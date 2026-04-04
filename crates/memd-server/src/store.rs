use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::Context;
use memd_schema::{
    EntityLinkRequest, EntityLinksRequest, EntitySearchHit, EntitySearchRequest,
    MemoryConsolidationRequest, MemoryContextFrame, MemoryDecayRequest, MemoryEntityLinkRecord,
    MemoryEntityRecord, MemoryEventRecord, MemoryItem, MemoryMaintenanceReportRequest,
};
use rusqlite::{Connection, params};
use uuid::Uuid;

use crate::keys::redundancy_key;

#[derive(Debug, Clone)]
pub struct DuplicateMatch {
    pub id: Uuid,
    pub item: MemoryItem,
}

#[derive(Debug, Clone)]
pub struct EntityMatch {
    pub record: MemoryEntityRecord,
}

#[derive(Debug, Clone)]
pub struct ConsolidationCandidate {
    pub entity: MemoryEntityRecord,
    pub event_count: usize,
    pub first_recorded_at: chrono::DateTime<chrono::Utc>,
    pub last_recorded_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
pub struct SqliteStore {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteStore {
    pub fn open<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let conn = Connection::open(path).context("open sqlite database")?;
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS memory_items (
              id TEXT PRIMARY KEY,
              kind TEXT NOT NULL,
              scope TEXT NOT NULL,
              stage TEXT NOT NULL,
              project TEXT,
              namespace TEXT,
              source_agent TEXT,
              redundancy_key TEXT,
              status TEXT NOT NULL,
              confidence REAL NOT NULL,
              canonical_key TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              payload_json TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_memory_scope ON memory_items(scope);
            CREATE INDEX IF NOT EXISTS idx_memory_stage ON memory_items(stage);
            CREATE INDEX IF NOT EXISTS idx_memory_project ON memory_items(project);
            CREATE INDEX IF NOT EXISTS idx_memory_namespace ON memory_items(namespace);
            CREATE INDEX IF NOT EXISTS idx_memory_source_agent ON memory_items(source_agent);
            CREATE INDEX IF NOT EXISTS idx_memory_status ON memory_items(status);
            CREATE UNIQUE INDEX IF NOT EXISTS idx_memory_redundancy_key_stage
              ON memory_items(redundancy_key, stage);
            CREATE UNIQUE INDEX IF NOT EXISTS idx_memory_canonical_key_stage
              ON memory_items(canonical_key, stage);
            CREATE INDEX IF NOT EXISTS idx_memory_updated_at ON memory_items(updated_at DESC);

            CREATE TABLE IF NOT EXISTS memory_entities (
              id TEXT PRIMARY KEY,
              entity_key TEXT NOT NULL UNIQUE,
              entity_type TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              payload_json TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_memory_entities_updated_at
              ON memory_entities(updated_at DESC);

            CREATE TABLE IF NOT EXISTS memory_events (
              id TEXT PRIMARY KEY,
              memory_item_id TEXT,
              entity_id TEXT NOT NULL,
              event_type TEXT NOT NULL,
              occurred_at TEXT NOT NULL,
              recorded_at TEXT NOT NULL,
              payload_json TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_memory_events_entity_id
              ON memory_events(entity_id, recorded_at DESC);
            CREATE INDEX IF NOT EXISTS idx_memory_events_memory_item_id
              ON memory_events(memory_item_id);

            CREATE TABLE IF NOT EXISTS memory_entity_links (
              id TEXT PRIMARY KEY,
              from_entity_id TEXT NOT NULL,
              to_entity_id TEXT NOT NULL,
              relation_kind TEXT NOT NULL,
              confidence REAL NOT NULL,
              created_at TEXT NOT NULL,
              note TEXT,
              context_json TEXT,
              tags_json TEXT NOT NULL,
              payload_json TEXT NOT NULL
            );

            CREATE UNIQUE INDEX IF NOT EXISTS idx_memory_entity_links_unique
              ON memory_entity_links(from_entity_id, to_entity_id, relation_kind);
            CREATE INDEX IF NOT EXISTS idx_memory_entity_links_from
              ON memory_entity_links(from_entity_id, created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_memory_entity_links_to
              ON memory_entity_links(to_entity_id, created_at DESC);
            "#,
        )
        .context("initialize sqlite schema")?;

        migrate_redundancy_key(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn insert_or_get_duplicate(
        &self,
        item: &MemoryItem,
        canonical_key: &str,
        redundancy_key: &str,
    ) -> anyhow::Result<Option<DuplicateMatch>> {
        let payload_json = serde_json::to_string(item).context("serialize memory item")?;
        let kind = serde_json::to_string(&item.kind).context("serialize memory kind")?;
        let scope = serde_json::to_string(&item.scope).context("serialize memory scope")?;
        let stage = serde_json::to_string(&item.stage).context("serialize memory stage")?;
        let status = serde_json::to_string(&item.status).context("serialize memory status")?;

        let conn = self.conn.lock().expect("sqlite mutex poisoned");
        let rows = conn.execute(
            r#"
            INSERT INTO memory_items (
              id, kind, scope, stage, project, namespace, source_agent, redundancy_key, status, confidence, canonical_key, updated_at, payload_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            "#,
            params![
                item.id.to_string(),
                kind,
                scope,
                stage,
                item.project,
                item.namespace,
                item.source_agent,
                redundancy_key,
                status,
                item.confidence,
                canonical_key,
                item.updated_at.to_rfc3339(),
                payload_json,
            ],
        );

        match rows {
            Ok(_) => Ok(None),
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE =>
            {
                let duplicate = self
                    .find_by_any_key(redundancy_key, canonical_key, &item.stage)?
                    .context("duplicate key reported but no row found")?;
                Ok(Some(duplicate))
            }
            Err(err) => Err(err).context("insert memory item"),
        }
    }

    pub fn update(
        &self,
        item: &MemoryItem,
        canonical_key: &str,
        redundancy_key: &str,
    ) -> anyhow::Result<()> {
        let payload_json = serde_json::to_string(item).context("serialize updated memory item")?;
        let kind = serde_json::to_string(&item.kind).context("serialize memory kind")?;
        let scope = serde_json::to_string(&item.scope).context("serialize memory scope")?;
        let stage = serde_json::to_string(&item.stage).context("serialize memory stage")?;
        let status = serde_json::to_string(&item.status).context("serialize memory status")?;

        let conn = self.conn.lock().expect("sqlite mutex poisoned");
        conn.execute(
            r#"
            UPDATE memory_items
            SET kind = ?2,
                scope = ?3,
                stage = ?4,
                project = ?5,
                namespace = ?6,
                source_agent = ?7,
                redundancy_key = ?8,
                status = ?9,
                confidence = ?10,
                canonical_key = ?11,
                updated_at = ?12,
                payload_json = ?13
            WHERE id = ?1
            "#,
            params![
                item.id.to_string(),
                kind,
                scope,
                stage,
                item.project,
                item.namespace,
                item.source_agent,
                redundancy_key,
                status,
                item.confidence,
                canonical_key,
                item.updated_at.to_rfc3339(),
                payload_json,
            ],
        )
        .context("update memory item")?;

        Ok(())
    }

    pub fn resolve_entity_for_item(
        &self,
        item: &MemoryItem,
        canonical_key: &str,
    ) -> anyhow::Result<EntityMatch> {
        let entity_key = derive_entity_key(item, canonical_key);
        if let Some(record) = self.get_entity_by_key(&entity_key)? {
            let record = update_entity_record(record, item);
            self.upsert_entity(&entity_key, &record)?;
            return Ok(EntityMatch { record });
        }

        let record = new_entity_record(item);
        self.upsert_entity(&entity_key, &record)?;
        Ok(EntityMatch { record })
    }

    pub fn rehearse_entity_for_item(
        &self,
        item: &MemoryItem,
        canonical_key: &str,
        salience_boost: f32,
    ) -> anyhow::Result<Option<MemoryEntityRecord>> {
        let entity_key = derive_entity_key(item, canonical_key);
        let Some(mut record) = self.get_entity_by_key(&entity_key)? else {
            return Ok(None);
        };

        let now = chrono::Utc::now();
        record.rehearsal_count = record.rehearsal_count.saturating_add(1);
        record.last_accessed_at = Some(now);
        record.salience_score = (record.salience_score + salience_boost).min(1.0);
        record.updated_at = now;
        self.upsert_entity(&entity_key, &record)?;
        Ok(Some(record))
    }

    pub fn decay_entities(
        &self,
        request: &MemoryDecayRequest,
    ) -> anyhow::Result<(usize, usize, usize)> {
        let max_items = request.max_items.unwrap_or(128).min(1_000);
        let inactive_days = request.inactive_days.unwrap_or(21).max(1);
        let max_decay = request.max_decay.unwrap_or(0.12).clamp(0.01, 0.5);
        let record_events = request.record_events.unwrap_or(true);

        let rows: Vec<(String, MemoryEntityRecord)> = {
            let conn = self.conn.lock().expect("sqlite mutex poisoned");
            let mut stmt = conn
                .prepare(
                    r#"
                    SELECT entity_key, payload_json
                    FROM memory_entities
                    ORDER BY updated_at ASC
                    LIMIT ?1
                    "#,
                )
                .context("prepare decay entity query")?;
            let rows = stmt
                .query_map(params![max_items as i64], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .context("query decay entities")?;

            let mut decoded = Vec::new();
            for row in rows {
                let (entity_key, payload) = row.context("read decay entity row")?;
                let entity: MemoryEntityRecord =
                    serde_json::from_str(&payload).context("deserialize decay entity payload")?;
                decoded.push((entity_key, entity));
            }
            decoded
        };

        let mut scanned = 0usize;
        let mut updated = 0usize;
        let mut events = 0usize;
        let now = chrono::Utc::now();

        for (entity_key, mut entity) in rows {
            scanned += 1;

            let reference = entity
                .last_accessed_at
                .or(entity.last_seen_at)
                .unwrap_or(entity.updated_at);
            let idle_days = (now - reference).num_days().max(0);
            if idle_days < inactive_days {
                continue;
            }

            let inactive_days_over = (idle_days - inactive_days) as f32;
            let rehearsal_factor = 1.0 / ((entity.rehearsal_count as f32 + 1.0).ln_1p() + 1.0);
            let decay = (inactive_days_over / 14.0).min(1.0) * max_decay * rehearsal_factor;
            if decay <= 0.001 {
                continue;
            }

            let original_salience = entity.salience_score;
            entity.salience_score = (entity.salience_score - decay).max(0.0);
            if (entity.salience_score - original_salience).abs() < f32::EPSILON {
                continue;
            }

            entity.updated_at = now;
            self.upsert_entity(&entity_key, &entity)?;
            updated += 1;

            if record_events {
                let event = MemoryEventRecord {
                    id: Uuid::new_v4(),
                    entity_id: Some(entity.id),
                    event_type: "decayed".to_string(),
                    summary: format!(
                        "salience decayed from {:.3} to {:.3} after {} idle days",
                        original_salience, entity.salience_score, idle_days
                    ),
                    occurred_at: now,
                    recorded_at: now,
                    confidence: entity.confidence,
                    salience_score: entity.salience_score,
                    project: entity
                        .context
                        .as_ref()
                        .and_then(|context| context.project.clone()),
                    namespace: entity
                        .context
                        .as_ref()
                        .and_then(|context| context.namespace.clone()),
                    source_agent: entity
                        .context
                        .as_ref()
                        .and_then(|context| context.agent.clone()),
                    source_system: entity
                        .context
                        .as_ref()
                        .and_then(|context| context.repo.clone()),
                    source_path: entity
                        .context
                        .as_ref()
                        .and_then(|context| context.location.clone()),
                    related_entity_ids: Vec::new(),
                    tags: entity.tags.clone(),
                    context: entity.context.clone(),
                };
                self.insert_event(&event, None)?;
                events += 1;
            }
        }

        Ok((scanned, updated, events))
    }

    pub fn decay_candidate_count(&self, request: &MemoryDecayRequest) -> anyhow::Result<usize> {
        let max_items = request.max_items.unwrap_or(128).min(1_000);
        let inactive_days = request.inactive_days.unwrap_or(21).max(1);
        let max_decay = request.max_decay.unwrap_or(0.12).clamp(0.01, 0.5);

        let rows: Vec<MemoryEntityRecord> = {
            let conn = self.conn.lock().expect("sqlite mutex poisoned");
            let mut stmt = conn
                .prepare(
                    r#"
                    SELECT payload_json
                    FROM memory_entities
                    ORDER BY updated_at ASC
                    LIMIT ?1
                    "#,
                )
                .context("prepare decay count query")?;
            let rows = stmt
                .query_map(params![max_items as i64], |row| row.get::<_, String>(0))
                .context("query decay count entities")?;

            let mut decoded = Vec::new();
            for row in rows {
                let payload = row.context("read decay count entity row")?;
                let entity: MemoryEntityRecord = serde_json::from_str(&payload)
                    .context("deserialize decay count entity payload")?;
                decoded.push(entity);
            }
            decoded
        };

        let now = chrono::Utc::now();
        let mut updated = 0usize;

        for entity in rows {
            let reference = entity
                .last_accessed_at
                .or(entity.last_seen_at)
                .unwrap_or(entity.updated_at);
            let idle_days = (now - reference).num_days().max(0);
            if idle_days < inactive_days {
                continue;
            }

            let inactive_days_over = (idle_days - inactive_days) as f32;
            let rehearsal_factor = 1.0 / ((entity.rehearsal_count as f32 + 1.0).ln_1p() + 1.0);
            let decay = (inactive_days_over / 14.0).min(1.0) * max_decay * rehearsal_factor;
            if decay > 0.001 {
                updated += 1;
            }
        }

        Ok(updated)
    }

    pub fn consolidation_candidates(
        &self,
        request: &MemoryConsolidationRequest,
    ) -> anyhow::Result<Vec<ConsolidationCandidate>> {
        let max_groups = request.max_groups.unwrap_or(24).min(128);
        let min_events = request.min_events.unwrap_or(3).max(2);
        let lookback_days = request.lookback_days.unwrap_or(14).max(1);
        let cutoff = chrono::Utc::now() - chrono::Duration::days(lookback_days);

        let rows: Vec<(String, i64, String, String)> = {
            let conn = self.conn.lock().expect("sqlite mutex poisoned");
            let mut stmt = conn
                .prepare(
                    r#"
                    SELECT entity_id, COUNT(*) AS event_count, MIN(recorded_at) AS first_at, MAX(recorded_at) AS last_at
                    FROM memory_events
                    WHERE entity_id != ''
                      AND recorded_at >= ?1
                    GROUP BY entity_id
                    HAVING COUNT(*) >= ?2
                    ORDER BY event_count DESC, last_at DESC
                    LIMIT ?3
                    "#,
                )
                .context("prepare consolidation query")?;
            let rows = stmt
                .query_map(
                    params![cutoff.to_rfc3339(), min_events as i64, max_groups as i64],
                    |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, i64>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                        ))
                    },
                )
                .context("query consolidation candidates")?;

            let mut decoded = Vec::new();
            for row in rows {
                decoded.push(row.context("read consolidation candidate row")?);
            }
            decoded
        };

        let mut candidates = Vec::new();
        for (entity_id, event_count, first_at, last_at) in rows {
            let entity = match self.entity_by_id(
                Uuid::parse_str(&entity_id).context("parse consolidation entity id")?,
            )? {
                Some(entity) => entity,
                None => continue,
            };

            let passes_project_filter = request.project.as_ref().is_none_or(|project| {
                entity
                    .context
                    .as_ref()
                    .and_then(|context| context.project.as_ref())
                    == Some(project)
            });
            let passes_namespace_filter = request.namespace.as_ref().is_none_or(|namespace| {
                entity
                    .context
                    .as_ref()
                    .and_then(|context| context.namespace.as_ref())
                    == Some(namespace)
            });
            if !passes_project_filter || !passes_namespace_filter {
                continue;
            }

            let first_recorded_at =
                chrono::DateTime::parse_from_rfc3339(&first_at)?.with_timezone(&chrono::Utc);
            let last_recorded_at =
                chrono::DateTime::parse_from_rfc3339(&last_at)?.with_timezone(&chrono::Utc);

            candidates.push(ConsolidationCandidate {
                entity,
                event_count: event_count as usize,
                first_recorded_at,
                last_recorded_at,
            });
        }

        Ok(candidates)
    }

    pub fn maintenance_report(
        &self,
        request: &MemoryMaintenanceReportRequest,
    ) -> anyhow::Result<(usize, usize, usize, usize, usize, Vec<String>)> {
        let stale_items = self.stale_item_count(request)?;
        let reinforced_candidates = self.reinforced_candidate_count(request)?;
        let cooled_candidates = self.decay_candidate_count(&MemoryDecayRequest {
            max_items: Some(256),
            inactive_days: request.inactive_days,
            max_decay: request.max_decay,
            record_events: Some(false),
        })?;
        let consolidated_candidates =
            self.consolidation_candidates(&MemoryConsolidationRequest {
                project: request.project.clone(),
                namespace: request.namespace.clone(),
                max_groups: Some(256),
                min_events: request.min_events,
                lookback_days: request.lookback_days,
                min_salience: None,
                record_events: Some(false),
            })?;
        let consolidated_candidates_count = consolidated_candidates.len();
        let highlights = consolidated_candidates
            .into_iter()
            .take(3)
            .map(|candidate| {
                format!(
                    "{}:{} events salience={:.2}",
                    candidate.entity.entity_type,
                    candidate.event_count,
                    candidate.entity.salience_score
                )
            })
            .collect::<Vec<_>>();
        let skipped = stale_items.saturating_sub(reinforced_candidates);

        Ok((
            reinforced_candidates,
            cooled_candidates,
            consolidated_candidates_count,
            stale_items,
            skipped,
            highlights,
        ))
    }

    pub fn entity_for_item(&self, item_id: Uuid) -> anyhow::Result<Option<MemoryEntityRecord>> {
        let conn = self.conn.lock().expect("sqlite mutex poisoned");
        let payload = conn.query_row(
            r#"
            SELECT e.payload_json
            FROM memory_events ev
            JOIN memory_entities e ON e.id = ev.entity_id
            WHERE ev.memory_item_id = ?1
            ORDER BY ev.recorded_at DESC
            LIMIT 1
            "#,
            [item_id.to_string()],
            |row| row.get::<_, String>(0),
        );

        match payload {
            Ok(payload) => {
                let entity: MemoryEntityRecord =
                    serde_json::from_str(&payload).context("deserialize memory entity payload")?;
                Ok(Some(entity))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(err).context("fetch memory entity by item"),
        }
    }

    pub fn entity_by_id(&self, entity_id: Uuid) -> anyhow::Result<Option<MemoryEntityRecord>> {
        let conn = self.conn.lock().expect("sqlite mutex poisoned");
        let payload = conn.query_row(
            "SELECT payload_json FROM memory_entities WHERE id = ?1",
            [entity_id.to_string()],
            |row| row.get::<_, String>(0),
        );

        match payload {
            Ok(payload) => {
                let entity: MemoryEntityRecord =
                    serde_json::from_str(&payload).context("deserialize memory entity payload")?;
                Ok(Some(entity))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(err).context("fetch memory entity by id"),
        }
    }

    pub fn upsert_entity_link(&self, link: &MemoryEntityLinkRecord) -> anyhow::Result<()> {
        let payload_json = serde_json::to_string(link).context("serialize entity link")?;
        let context_json =
            serde_json::to_string(&link.context).context("serialize entity link context")?;
        let tags_json = serde_json::to_string(&link.tags).context("serialize entity link tags")?;
        let relation_kind =
            serde_json::to_string(&link.relation_kind).context("serialize entity link relation")?;
        let conn = self.conn.lock().expect("sqlite mutex poisoned");
        conn.execute(
            r#"
            INSERT INTO memory_entity_links (
              id, from_entity_id, to_entity_id, relation_kind, confidence, created_at,
              note, context_json, tags_json, payload_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(from_entity_id, to_entity_id, relation_kind) DO UPDATE SET
              id = excluded.id,
              confidence = excluded.confidence,
              created_at = excluded.created_at,
              note = excluded.note,
              context_json = excluded.context_json,
              tags_json = excluded.tags_json,
              payload_json = excluded.payload_json
            "#,
            params![
                link.id.to_string(),
                link.from_entity_id.to_string(),
                link.to_entity_id.to_string(),
                relation_kind,
                link.confidence,
                link.created_at.to_rfc3339(),
                link.note,
                context_json,
                tags_json,
                payload_json,
            ],
        )
        .context("upsert entity link")?;
        Ok(())
    }

    pub fn link_entity(
        &self,
        request: &EntityLinkRequest,
    ) -> anyhow::Result<MemoryEntityLinkRecord> {
        let now = chrono::Utc::now();
        let link = MemoryEntityLinkRecord {
            id: Uuid::new_v4(),
            from_entity_id: request.from_entity_id,
            to_entity_id: request.to_entity_id,
            relation_kind: request.relation_kind,
            confidence: request.confidence.unwrap_or(0.8).clamp(0.0, 1.0),
            created_at: now,
            note: request.note.clone(),
            context: request.context.clone(),
            tags: request.tags.clone(),
        };
        self.upsert_entity_link(&link)?;
        Ok(link)
    }

    pub fn links_for_entity(
        &self,
        request: &EntityLinksRequest,
    ) -> anyhow::Result<Vec<MemoryEntityLinkRecord>> {
        let conn = self.conn.lock().expect("sqlite mutex poisoned");
        let mut stmt = conn
            .prepare(
                r#"
                SELECT payload_json
                FROM memory_entity_links
                WHERE from_entity_id = ?1 OR to_entity_id = ?1
                ORDER BY created_at DESC
                "#,
            )
            .context("prepare entity links query")?;
        let rows = stmt
            .query_map([request.entity_id.to_string()], |row| {
                row.get::<_, String>(0)
            })
            .context("query entity links")?;

        let mut links = Vec::new();
        for row in rows {
            let payload = row.context("read entity link row")?;
            let link: MemoryEntityLinkRecord =
                serde_json::from_str(&payload).context("deserialize entity link payload")?;
            links.push(link);
        }
        Ok(links)
    }

    pub fn list_entities(&self) -> anyhow::Result<Vec<MemoryEntityRecord>> {
        let conn = self.conn.lock().expect("sqlite mutex poisoned");
        let mut stmt = conn
            .prepare("SELECT payload_json FROM memory_entities ORDER BY updated_at DESC")
            .context("prepare entity list query")?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .context("query memory entities")?;

        let mut entities = Vec::new();
        for row in rows {
            let payload = row.context("read entity row")?;
            let entity: MemoryEntityRecord =
                serde_json::from_str(&payload).context("deserialize memory entity payload")?;
            entities.push(entity);
        }
        Ok(entities)
    }

    pub fn search_entities(
        &self,
        request: &EntitySearchRequest,
    ) -> anyhow::Result<Vec<EntitySearchHit>> {
        let query = normalize_search_text(&request.query);
        if query.is_empty() {
            return Ok(Vec::new());
        }

        let query_tokens = tokenize_search_text(&query);
        let limit = request.limit.unwrap_or(5).min(20);
        let mut hits = Vec::new();
        for entity in self.list_entities()? {
            if !entity_matches_context(&entity, request) {
                continue;
            }

            let (score, reasons) = score_entity_search(request, &query, &query_tokens, &entity);
            if score <= 0.0 {
                continue;
            }

            hits.push(EntitySearchHit {
                entity,
                score,
                reasons,
            });
        }

        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.entity.rehearsal_count.cmp(&a.entity.rehearsal_count))
                .then_with(|| {
                    b.entity
                        .salience_score
                        .partial_cmp(&a.entity.salience_score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .then_with(|| b.entity.updated_at.cmp(&a.entity.updated_at))
        });
        hits.truncate(limit);
        Ok(hits)
    }

    fn stale_item_count(&self, request: &MemoryMaintenanceReportRequest) -> anyhow::Result<usize> {
        let conn = self.conn.lock().expect("sqlite mutex poisoned");
        let mut stmt = conn
            .prepare(
                r#"
                SELECT COUNT(*)
                FROM memory_items
                WHERE status = ?1
                  AND (?2 IS NULL OR project = ?2)
                  AND (?3 IS NULL OR namespace = ?3)
                "#,
            )
            .context("prepare stale item count query")?;
        let count: i64 = stmt
            .query_row(
                params![
                    serde_json::to_string(&memd_schema::MemoryStatus::Stale)?,
                    request.project.as_deref(),
                    request.namespace.as_deref(),
                ],
                |row| row.get(0),
            )
            .context("count stale memory items")?;
        Ok(count as usize)
    }

    fn reinforced_candidate_count(
        &self,
        request: &MemoryMaintenanceReportRequest,
    ) -> anyhow::Result<usize> {
        let items = self.list()?;
        let mut count = 0usize;
        for item in items {
            if item.status != memd_schema::MemoryStatus::Stale {
                continue;
            }
            if request
                .project
                .as_ref()
                .is_some_and(|project| item.project.as_ref() != Some(project))
            {
                continue;
            }
            if request
                .namespace
                .as_ref()
                .is_some_and(|namespace| item.namespace.as_ref() != Some(namespace))
            {
                continue;
            }
            if let Some(source_path) = &item.source_path {
                if Path::new(source_path).exists() {
                    count += 1;
                }
            }
        }
        Ok(count)
    }

    pub fn events_for_entity(
        &self,
        entity_id: Uuid,
        limit: usize,
    ) -> anyhow::Result<Vec<MemoryEventRecord>> {
        let conn = self.conn.lock().expect("sqlite mutex poisoned");
        let mut stmt = conn
            .prepare(
                r#"
                SELECT payload_json
                FROM memory_events
                WHERE entity_id = ?1
                ORDER BY recorded_at DESC
                LIMIT ?2
                "#,
            )
            .context("prepare entity event query")?;
        let rows = stmt
            .query_map(params![entity_id.to_string(), limit as i64], |row| {
                row.get::<_, String>(0)
            })
            .context("query entity events")?;

        let mut events = Vec::new();
        for row in rows {
            let payload = row.context("read entity event row")?;
            let event: MemoryEventRecord =
                serde_json::from_str(&payload).context("deserialize memory event payload")?;
            events.push(event);
        }
        Ok(events)
    }

    pub fn record_event(
        &self,
        entity: &MemoryEntityRecord,
        memory_item_id: Uuid,
        event_type: &str,
        summary: String,
        occurred_at: chrono::DateTime<chrono::Utc>,
        project: Option<String>,
        namespace: Option<String>,
        source_agent: Option<String>,
        source_system: Option<String>,
        source_path: Option<String>,
        related_entity_ids: Vec<Uuid>,
        tags: Vec<String>,
        context: Option<MemoryContextFrame>,
        confidence: f32,
        salience_score: f32,
    ) -> anyhow::Result<MemoryEventRecord> {
        let now = chrono::Utc::now();
        let event = MemoryEventRecord {
            id: Uuid::new_v4(),
            entity_id: Some(entity.id),
            event_type: event_type.to_string(),
            summary,
            occurred_at,
            recorded_at: now,
            confidence,
            salience_score,
            project,
            namespace,
            source_agent,
            source_system,
            source_path,
            related_entity_ids,
            tags,
            context,
        };

        let payload_json = serde_json::to_string(&event).context("serialize memory event")?;
        self.insert_event_payload(&event, Some(memory_item_id), payload_json)?;

        Ok(event)
    }

    pub fn get(&self, id: Uuid) -> anyhow::Result<Option<MemoryItem>> {
        let conn = self.conn.lock().expect("sqlite mutex poisoned");
        let payload = conn.query_row(
            "SELECT payload_json FROM memory_items WHERE id = ?1",
            [id.to_string()],
            |row| row.get::<_, String>(0),
        );

        match payload {
            Ok(payload) => {
                let item: MemoryItem =
                    serde_json::from_str(&payload).context("deserialize memory item payload")?;
                Ok(Some(item))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(err).context("fetch memory item by id"),
        }
    }

    pub fn list(&self) -> anyhow::Result<Vec<MemoryItem>> {
        let conn = self.conn.lock().expect("sqlite mutex poisoned");
        let mut stmt = conn
            .prepare("SELECT payload_json FROM memory_items ORDER BY updated_at DESC")
            .context("prepare list query")?;

        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .context("query memory items")?;

        let mut items = Vec::new();
        for row in rows {
            let payload = row.context("read memory row")?;
            let item: MemoryItem =
                serde_json::from_str(&payload).context("deserialize memory item payload")?;
            items.push(item);
        }

        Ok(items)
    }

    pub fn count(&self) -> anyhow::Result<usize> {
        let conn = self.conn.lock().expect("sqlite mutex poisoned");
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM memory_items", [], |row| row.get(0))
            .context("count memory items")?;
        Ok(count as usize)
    }

    pub fn find_duplicate(
        &self,
        redundancy_key: &str,
        canonical_key: &str,
        stage: &memd_schema::MemoryStage,
        exclude_id: Uuid,
    ) -> anyhow::Result<Option<DuplicateMatch>> {
        self.find_by_any_key(redundancy_key, canonical_key, stage)
            .map(|found| found.filter(|duplicate| duplicate.id != exclude_id))
    }

    fn find_by_any_key(
        &self,
        redundancy_key: &str,
        canonical_key: &str,
        stage: &memd_schema::MemoryStage,
    ) -> anyhow::Result<Option<DuplicateMatch>> {
        let stage = serde_json::to_string(stage).context("serialize lookup stage")?;
        let conn = self.conn.lock().expect("sqlite mutex poisoned");
        let row = conn.query_row(
            r#"
            SELECT id, payload_json
            FROM memory_items
            WHERE (redundancy_key = ?1 OR canonical_key = ?2) AND stage = ?3
            "#,
            params![redundancy_key, canonical_key, stage],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        );

        match row {
            Ok((id, payload)) => {
                let item: MemoryItem =
                    serde_json::from_str(&payload).context("deserialize duplicate memory item")?;
                Ok(Some(DuplicateMatch {
                    id: Uuid::parse_str(&id).context("parse duplicate id")?,
                    item,
                }))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(err).context("lookup duplicate excluding current item"),
        }
    }

    fn get_entity_by_key(&self, entity_key: &str) -> anyhow::Result<Option<MemoryEntityRecord>> {
        let conn = self.conn.lock().expect("sqlite mutex poisoned");
        let payload = conn.query_row(
            "SELECT payload_json FROM memory_entities WHERE entity_key = ?1",
            [entity_key],
            |row| row.get::<_, String>(0),
        );

        match payload {
            Ok(payload) => {
                let entity: MemoryEntityRecord =
                    serde_json::from_str(&payload).context("deserialize memory entity payload")?;
                Ok(Some(entity))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(err).context("fetch memory entity by key"),
        }
    }

    fn upsert_entity(&self, entity_key: &str, record: &MemoryEntityRecord) -> anyhow::Result<()> {
        let payload_json = serde_json::to_string(record).context("serialize memory entity")?;
        let conn = self.conn.lock().expect("sqlite mutex poisoned");
        conn.execute(
            r#"
            INSERT INTO memory_entities (id, entity_key, entity_type, updated_at, payload_json)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(entity_key) DO UPDATE SET
              entity_type = excluded.entity_type,
              updated_at = excluded.updated_at,
              payload_json = excluded.payload_json
            "#,
            params![
                record.id.to_string(),
                entity_key,
                record.entity_type,
                record.updated_at.to_rfc3339(),
                payload_json,
            ],
        )
        .context("upsert memory entity")?;
        Ok(())
    }

    fn insert_event(
        &self,
        event: &MemoryEventRecord,
        memory_item_id: Option<Uuid>,
    ) -> anyhow::Result<()> {
        let payload_json = serde_json::to_string(event).context("serialize memory event")?;
        self.insert_event_payload(event, memory_item_id, payload_json)
    }

    fn insert_event_payload(
        &self,
        event: &MemoryEventRecord,
        memory_item_id: Option<Uuid>,
        payload_json: String,
    ) -> anyhow::Result<()> {
        let conn = self.conn.lock().expect("sqlite mutex poisoned");
        conn.execute(
            r#"
            INSERT INTO memory_events (
              id, memory_item_id, entity_id, event_type, occurred_at, recorded_at, payload_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                event.id.to_string(),
                memory_item_id.map(|value| value.to_string()),
                event
                    .entity_id
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                event.event_type,
                event.occurred_at.to_rfc3339(),
                event.recorded_at.to_rfc3339(),
                payload_json,
            ],
        )
        .context("insert memory event")?;
        Ok(())
    }
}

fn derive_entity_key(item: &MemoryItem, canonical_key: &str) -> String {
    if let Some(source_path) = item.source_path.as_deref() {
        return format!(
            "path|{:?}|{:?}|{}",
            item.project.as_deref().unwrap_or(""),
            item.namespace.as_deref().unwrap_or(""),
            source_path
        );
    }

    if let Some(source_system) = item.source_system.as_deref() {
        return format!(
            "system|{:?}|{:?}|{:?}|{}",
            item.project.as_deref().unwrap_or(""),
            item.namespace.as_deref().unwrap_or(""),
            source_system,
            canonical_key
        );
    }

    format!(
        "entity|{:?}|{:?}|{:?}|{}",
        item.project.as_deref().unwrap_or(""),
        item.namespace.as_deref().unwrap_or(""),
        item.kind,
        canonical_key
    )
}

fn new_entity_record(item: &MemoryItem) -> MemoryEntityRecord {
    let now = chrono::Utc::now();
    MemoryEntityRecord {
        id: Uuid::new_v4(),
        entity_type: format!("{:?}", item.kind).to_lowercase(),
        aliases: entity_aliases(item),
        current_state: Some(compact_entity_state(item)),
        state_version: 1,
        confidence: item.confidence,
        salience_score: item.confidence.clamp(0.0, 1.0),
        rehearsal_count: 1,
        created_at: now,
        updated_at: now,
        last_accessed_at: Some(now),
        last_seen_at: Some(item.updated_at),
        valid_from: Some(item.updated_at),
        valid_to: None,
        tags: item.tags.clone(),
        context: Some(MemoryContextFrame {
            at: Some(item.updated_at),
            project: item.project.clone(),
            namespace: item.namespace.clone(),
            repo: item.source_system.clone(),
            host: None,
            branch: None,
            agent: item.source_agent.clone(),
            location: item.source_path.clone(),
        }),
    }
}

fn update_entity_record(mut record: MemoryEntityRecord, item: &MemoryItem) -> MemoryEntityRecord {
    let now = chrono::Utc::now();
    let previous = record.context.clone();
    let previous_project = previous
        .as_ref()
        .and_then(|context| context.project.clone());
    let previous_namespace = previous
        .as_ref()
        .and_then(|context| context.namespace.clone());
    let previous_repo = previous.as_ref().and_then(|context| context.repo.clone());
    let previous_host = previous.as_ref().and_then(|context| context.host.clone());
    let previous_branch = previous.as_ref().and_then(|context| context.branch.clone());
    let previous_agent = previous.as_ref().and_then(|context| context.agent.clone());
    let previous_location = previous
        .as_ref()
        .and_then(|context| context.location.clone());

    record.aliases = merge_aliases(&record.aliases, &entity_aliases(item));
    record.current_state = Some(compact_entity_state(item));
    record.state_version = record.state_version.saturating_add(1);
    record.confidence = record.confidence.max(item.confidence).clamp(0.0, 1.0);
    record.salience_score = (record.salience_score + 0.05).min(1.0);
    record.rehearsal_count = record.rehearsal_count.saturating_add(1);
    record.updated_at = now;
    record.last_accessed_at = Some(now);
    record.last_seen_at = Some(item.updated_at);
    if record.valid_from.is_none() {
        record.valid_from = Some(item.updated_at);
    }
    record.valid_to = None;
    record.tags = merge_tags(&record.tags, &item.tags);
    record.context = Some(MemoryContextFrame {
        at: Some(item.updated_at),
        project: item.project.clone().or(previous_project),
        namespace: item.namespace.clone().or(previous_namespace),
        repo: item.source_system.clone().or(previous_repo),
        host: previous_host,
        branch: previous_branch,
        agent: item.source_agent.clone().or(previous_agent),
        location: item.source_path.clone().or(previous_location),
    });
    record
}

fn entity_aliases(item: &MemoryItem) -> Vec<String> {
    let mut aliases = Vec::new();
    if let Some(project) = &item.project {
        aliases.push(project.clone());
    }
    if let Some(namespace) = &item.namespace {
        aliases.push(namespace.clone());
    }
    if let Some(agent) = &item.source_agent {
        aliases.push(agent.clone());
    }
    if let Some(system) = &item.source_system {
        aliases.push(system.clone());
    }
    if let Some(path) = &item.source_path {
        aliases.push(path.clone());
        if let Some(file_name) = Path::new(path).file_name().and_then(|value| value.to_str()) {
            aliases.push(file_name.to_string());
        }
    }
    aliases.push(format!("{:?}", item.kind).to_lowercase());
    aliases.sort();
    aliases.dedup();
    aliases
}

fn merge_aliases(existing: &[String], incoming: &[String]) -> Vec<String> {
    let mut aliases = existing.to_vec();
    aliases.extend(incoming.iter().cloned());
    aliases.sort();
    aliases.dedup();
    aliases
}

fn merge_tags(existing: &[String], incoming: &[String]) -> Vec<String> {
    let mut tags = existing.to_vec();
    tags.extend(incoming.iter().cloned());
    tags.sort();
    tags.dedup();
    tags
}

fn normalize_search_text(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn tokenize_search_text(value: &str) -> Vec<String> {
    value
        .split_whitespace()
        .map(|value| value.to_string())
        .filter(|value| !value.is_empty())
        .collect()
}

fn score_entity_search(
    request: &EntitySearchRequest,
    query: &str,
    query_tokens: &[String],
    entity: &MemoryEntityRecord,
) -> (f32, Vec<String>) {
    let mut score = 0.0f32;
    let mut reasons = Vec::new();
    let haystacks = entity_search_haystacks(entity);

    for haystack in &haystacks {
        if haystack == query {
            score += 1.0;
            reasons.push("exact match".to_string());
        } else if haystack.starts_with(query) {
            score += 0.7;
            reasons.push("prefix match".to_string());
        } else if haystack.contains(query) {
            score += 0.5;
            reasons.push("substring match".to_string());
        }
    }

    for token in query_tokens {
        if haystacks.iter().any(|haystack| haystack.contains(token)) {
            score += 0.2;
            reasons.push(format!("token:{token}"));
        }
    }

    if query_tokens.len() > 1 {
        let joined = query_tokens.join(" ");
        if haystacks.iter().any(|haystack| haystack.contains(&joined)) {
            score += 0.28;
            reasons.push("phrase match".to_string());
        }
    }

    if entity.salience_score > 0.0 {
        score += entity.salience_score * 0.08;
    }
    if entity.rehearsal_count > 0 {
        score += (entity.rehearsal_count as f32).ln_1p() * 0.03;
    }
    if entity.valid_from.is_some() {
        score += 0.08;
        reasons.push("validity window".to_string());
    }
    if request.project.is_some()
        && entity
            .context
            .as_ref()
            .and_then(|context| context.project.as_ref())
            .is_some()
    {
        score += 0.05;
        reasons.push("project context".to_string());
    }
    if request.namespace.is_some()
        && entity
            .context
            .as_ref()
            .and_then(|context| context.namespace.as_ref())
            .is_some()
    {
        score += 0.05;
        reasons.push("namespace context".to_string());
    }
    if request.host.is_some()
        && entity
            .context
            .as_ref()
            .and_then(|context| context.host.as_ref())
            .is_some()
    {
        score += 0.05;
        reasons.push("host context".to_string());
    }
    if request.branch.is_some()
        && entity
            .context
            .as_ref()
            .and_then(|context| context.branch.as_ref())
            .is_some()
    {
        score += 0.05;
        reasons.push("branch context".to_string());
    }
    if request.location.is_some()
        && entity
            .context
            .as_ref()
            .and_then(|context| context.location.as_ref())
            .is_some()
    {
        score += 0.05;
        reasons.push("location context".to_string());
    }
    if request.at.is_some() {
        score += 0.05;
        reasons.push("timestamp context".to_string());
    }

    score = score.min(1.0);
    reasons.sort();
    reasons.dedup();
    (score, reasons)
}

fn entity_search_haystacks(entity: &MemoryEntityRecord) -> Vec<String> {
    let mut haystacks = Vec::new();
    haystacks.push(normalize_search_text(&entity.entity_type));
    haystacks.extend(
        entity
            .aliases
            .iter()
            .map(|alias| normalize_search_text(alias)),
    );
    if let Some(state) = &entity.current_state {
        haystacks.push(normalize_search_text(state));
    }
    if let Some(context) = &entity.context {
        if let Some(project) = &context.project {
            haystacks.push(normalize_search_text(project));
        }
        if let Some(namespace) = &context.namespace {
            haystacks.push(normalize_search_text(namespace));
        }
        if let Some(repo) = &context.repo {
            haystacks.push(normalize_search_text(repo));
        }
        if let Some(agent) = &context.agent {
            haystacks.push(normalize_search_text(agent));
        }
        if let Some(location) = &context.location {
            haystacks.push(normalize_search_text(location));
            if let Some(file_name) = Path::new(location)
                .file_name()
                .and_then(|value| value.to_str())
            {
                haystacks.push(normalize_search_text(file_name));
            }
        }
    }
    haystacks.extend(entity.tags.iter().map(|tag| normalize_search_text(tag)));
    haystacks.sort();
    haystacks.dedup();
    haystacks
}

fn entity_matches_context(entity: &MemoryEntityRecord, request: &EntitySearchRequest) -> bool {
    if let Some(at) = request.at {
        if entity.valid_from.is_some_and(|valid_from| at < valid_from) {
            return false;
        }
        if entity.valid_to.is_some_and(|valid_to| at > valid_to) {
            return false;
        }
    }

    let context = entity.context.as_ref();
    if request.project.as_ref().is_some_and(|project| {
        context
            .and_then(|context| context.project.as_ref())
            .is_none_or(|entity_project| entity_project != project)
    }) {
        return false;
    }
    if request.namespace.as_ref().is_some_and(|namespace| {
        context
            .and_then(|context| context.namespace.as_ref())
            .is_none_or(|entity_namespace| entity_namespace != namespace)
    }) {
        return false;
    }
    if request.host.as_ref().is_some_and(|host| {
        context
            .and_then(|context| context.host.as_ref())
            .is_none_or(|entity_host| entity_host != host)
    }) {
        return false;
    }
    if request.branch.as_ref().is_some_and(|branch| {
        context
            .and_then(|context| context.branch.as_ref())
            .is_none_or(|entity_branch| entity_branch != branch)
    }) {
        return false;
    }
    if request.location.as_ref().is_some_and(|location| {
        context
            .and_then(|context| context.location.as_ref())
            .is_none_or(|entity_location| entity_location != location)
    }) {
        return false;
    }

    true
}

fn compact_entity_state(item: &MemoryItem) -> String {
    let mut state = item
        .content
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if state.len() > 240 {
        state.truncate(240);
        state.push('…');
    }
    state
}

fn migrate_redundancy_key(conn: &Connection) -> anyhow::Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(memory_items)")?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;

    if !columns.iter().any(|column| column == "redundancy_key") {
        conn.execute_batch("ALTER TABLE memory_items ADD COLUMN redundancy_key TEXT;")?;
        let mut stmt = conn.prepare("SELECT id, payload_json FROM memory_items")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (id, payload) = row?;
            let item: MemoryItem = serde_json::from_str(&payload)?;
            let key = redundancy_key(&item);
            conn.execute(
                "UPDATE memory_items SET redundancy_key = ?1 WHERE id = ?2",
                params![key, id],
            )?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_entity_search_scores_alias_hits_highest() {
        let entity = MemoryEntityRecord {
            id: Uuid::new_v4(),
            entity_type: "repo".to_string(),
            aliases: vec!["memd".to_string(), "memory manager".to_string()],
            current_state: Some("main branch with smart memory".to_string()),
            state_version: 1,
            confidence: 0.9,
            salience_score: 0.8,
            rehearsal_count: 3,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_accessed_at: Some(chrono::Utc::now()),
            last_seen_at: Some(chrono::Utc::now()),
            valid_from: Some(chrono::Utc::now()),
            valid_to: None,
            tags: vec!["project".to_string()],
            context: Some(MemoryContextFrame {
                at: Some(chrono::Utc::now()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo: Some("memd".to_string()),
                host: Some("laptop".to_string()),
                branch: Some("main".to_string()),
                agent: Some("codex".to_string()),
                location: Some("/tmp/memd".to_string()),
            }),
        };
        let request = EntitySearchRequest {
            query: "memd repo".to_string(),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            at: Some(chrono::Utc::now()),
            host: Some("laptop".to_string()),
            branch: Some("main".to_string()),
            location: Some("/tmp/memd".to_string()),
            route: None,
            intent: None,
            limit: Some(5),
        };

        let (score, reasons) = score_entity_search(
            &request,
            &normalize_search_text("memd repo"),
            &tokenize_search_text("memd repo"),
            &entity,
        );

        assert!(score > 0.5);
        assert!(reasons.iter().any(|reason| reason.contains("token:memd")));
    }
}
