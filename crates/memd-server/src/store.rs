use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::Context;
use memd_schema::{
    MemoryConsolidationRequest, MemoryContextFrame, MemoryDecayRequest, MemoryEntityRecord,
    MemoryEventRecord, MemoryItem,
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
