use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context;
use memd_schema::{
    AgentProfileRequest, AgentProfileUpsertRequest, EntityLinkRequest, EntityLinksRequest,
    EntitySearchHit, EntitySearchRequest, HiveBoardRequest, HiveBoardResponse,
    HiveClaimAcquireRequest, HiveClaimRecord, HiveClaimRecoverRequest, HiveClaimReleaseRequest,
    HiveClaimTransferRequest, HiveClaimsRequest, HiveClaimsResponse, HiveCoordinationInboxRequest,
    HiveCoordinationInboxResponse, HiveCoordinationReceiptRecord, HiveCoordinationReceiptRequest,
    HiveCoordinationReceiptsRequest, HiveCoordinationReceiptsResponse, HiveMessageAckRequest,
    HiveMessageInboxRequest, HiveMessageRecord, HiveMessageSendRequest, HiveMessagesResponse,
    HiveRosterResponse, HiveSessionAutoRetireResponse, HiveSessionRecord, HiveSessionRetireRequest,
    HiveSessionRetireResponse, HiveSessionUpsertRequest, HiveSessionsRequest, HiveSessionsResponse,
    HiveTaskAssignRequest, HiveTaskRecord, HiveTaskUpsertRequest, HiveTasksRequest,
    HiveTasksResponse, MaintainReport, MaintainReportRequest, MemoryAgentProfile,
    MemoryConsolidationRequest, MemoryContextFrame, MemoryDecayRequest, MemoryEntityLinkRecord,
    MemoryEntityRecord, MemoryEventRecord, MemoryItem, MemoryMaintenanceReportRequest,
    SkillPolicyActivationEntriesRequest, SkillPolicyActivationEntriesResponse,
    SkillPolicyActivationEntry, SkillPolicyApplyReceipt, SkillPolicyApplyReceiptsRequest,
    SkillPolicyApplyReceiptsResponse, SkillPolicyApplyRequest, SkillPolicyApplyResponse,
    SourceMemoryRecord, SourceMemoryRequest, SourceMemoryResponse, SourceQuality,
    WorkspaceMemoryRecord, WorkspaceMemoryRequest, WorkspaceMemoryResponse,
};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
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
    db_path: Arc<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct RecordEventArgs {
    pub event_type: String,
    pub summary: String,
    pub occurred_at: chrono::DateTime<chrono::Utc>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub source_agent: Option<String>,
    pub source_system: Option<String>,
    pub source_path: Option<String>,
    pub related_entity_ids: Vec<Uuid>,
    pub tags: Vec<String>,
    pub context: Option<MemoryContextFrame>,
    pub confidence: f32,
    pub salience_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SkillPolicyApplyRecordPayload {
    receipt: SkillPolicyApplyReceipt,
    request: SkillPolicyApplyRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MaintainReportRecordPayload {
    request: MaintainReportRequest,
    response: MaintainReport,
}

impl SqliteStore {
    pub fn open<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let db_path = path.as_ref().to_path_buf();
        let mut conn = Connection::open(&db_path).context("open sqlite database")?;
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

            CREATE TABLE IF NOT EXISTS memory_agent_profiles (
              id TEXT PRIMARY KEY,
              agent TEXT NOT NULL,
              project TEXT,
              namespace TEXT,
              updated_at TEXT NOT NULL,
              payload_json TEXT NOT NULL,
              UNIQUE(agent, project, namespace)
            );
            CREATE INDEX IF NOT EXISTS idx_memory_agent_profiles_updated_at
              ON memory_agent_profiles(updated_at DESC);

            CREATE TABLE IF NOT EXISTS hive_messages (
              id TEXT PRIMARY KEY,
              to_session TEXT NOT NULL,
              project TEXT,
              namespace TEXT,
              workspace TEXT,
              created_at TEXT NOT NULL,
              acknowledged_at TEXT,
              payload_json TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_hive_messages_session
              ON hive_messages(to_session, created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_hive_messages_project_namespace
              ON hive_messages(project, namespace, created_at DESC);

            CREATE TABLE IF NOT EXISTS hive_claims (
              scope TEXT PRIMARY KEY,
              session TEXT NOT NULL,
              project TEXT,
              namespace TEXT,
              workspace TEXT,
              expires_at TEXT NOT NULL,
              payload_json TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_hive_claims_session
              ON hive_claims(session, expires_at DESC);
            CREATE INDEX IF NOT EXISTS idx_hive_claims_project_namespace
              ON hive_claims(project, namespace, expires_at DESC);

            CREATE TABLE IF NOT EXISTS hive_tasks (
              task_id TEXT PRIMARY KEY,
              session TEXT,
              project TEXT,
              namespace TEXT,
              workspace TEXT,
              status TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              payload_json TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_hive_tasks_session
              ON hive_tasks(session, updated_at DESC);
            CREATE INDEX IF NOT EXISTS idx_hive_tasks_project_namespace
              ON hive_tasks(project, namespace, updated_at DESC);

            CREATE TABLE IF NOT EXISTS hive_coordination_receipts (
              id TEXT PRIMARY KEY,
              actor_session TEXT NOT NULL,
              project TEXT,
              namespace TEXT,
              workspace TEXT,
              created_at TEXT NOT NULL,
              payload_json TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_hive_coordination_receipts_actor
              ON hive_coordination_receipts(actor_session, created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_hive_coordination_receipts_project_namespace
              ON hive_coordination_receipts(project, namespace, created_at DESC);

            CREATE TABLE IF NOT EXISTS skill_policy_apply_receipts (
              id TEXT PRIMARY KEY,
              project TEXT,
              namespace TEXT,
              workspace TEXT,
              created_at TEXT NOT NULL,
              payload_json TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_skill_policy_apply_receipts_project_namespace
              ON skill_policy_apply_receipts(project, namespace, created_at DESC);

            CREATE TABLE IF NOT EXISTS skill_policy_activations (
              id TEXT PRIMARY KEY,
              receipt_id TEXT NOT NULL,
              project TEXT,
              namespace TEXT,
              workspace TEXT,
              created_at TEXT NOT NULL,
              payload_json TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_skill_policy_activations_receipt
              ON skill_policy_activations(receipt_id, created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_skill_policy_activations_project_namespace
              ON skill_policy_activations(project, namespace, created_at DESC);

            CREATE TABLE IF NOT EXISTS runtime_maintenance_reports (
              receipt_id TEXT PRIMARY KEY,
              mode TEXT NOT NULL,
              project TEXT,
              namespace TEXT,
              workspace TEXT,
              session TEXT,
              created_at TEXT NOT NULL,
              payload_json TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_runtime_maintenance_reports_scope
              ON runtime_maintenance_reports(project, namespace, workspace, created_at DESC);

            CREATE TABLE IF NOT EXISTS hive_sessions (
              session_key TEXT PRIMARY KEY,
              session TEXT NOT NULL,
              project TEXT,
              namespace TEXT,
              workspace TEXT,
              repo_root TEXT,
              worktree_root TEXT,
              branch TEXT,
              hive_system TEXT,
              hive_role TEXT,
              host TEXT,
              status TEXT NOT NULL,
              last_seen TEXT NOT NULL,
              payload_json TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_hive_sessions_session
              ON hive_sessions(session, last_seen DESC);
            CREATE INDEX IF NOT EXISTS idx_hive_sessions_project_namespace
              ON hive_sessions(project, namespace, last_seen DESC);
            CREATE INDEX IF NOT EXISTS idx_hive_sessions_project_namespace_workspace
              ON hive_sessions(project, namespace, workspace, last_seen DESC);
            CREATE INDEX IF NOT EXISTS idx_hive_sessions_last_seen
              ON hive_sessions(last_seen DESC);
            CREATE TABLE IF NOT EXISTS hive_session_groups (
              session_key TEXT NOT NULL,
              hive_group TEXT NOT NULL,
              PRIMARY KEY (session_key, hive_group),
              FOREIGN KEY (session_key) REFERENCES hive_sessions(session_key) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_hive_session_groups_session
              ON hive_session_groups(session_key);
            CREATE INDEX IF NOT EXISTS idx_hive_session_groups_hive_group
              ON hive_session_groups(hive_group, session_key);
            "#,
        )
        .context("initialize sqlite schema")?;

        migrate_redundancy_key(&conn)?;
        migrate_hive_sessions_identity_columns(&mut conn)?;
        create_hive_session_identity_indexes(&conn)?;

        Ok(Self {
            db_path: Arc::new(db_path),
        })
    }

    fn connect(&self) -> anyhow::Result<Connection> {
        let conn = Connection::open(self.db_path.as_ref())
            .with_context(|| format!("open sqlite database {}", self.db_path.display()))?;
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;
            "#,
        )
        .context("configure sqlite connection")?;
        Ok(conn)
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

        let rows = {
            let conn = self.connect()?;
            conn.execute(
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
            )
        };

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

        let mut conn = self.connect()?;
        let update_result = conn.execute(
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
                &kind,
                &scope,
                &stage,
                item.project,
                item.namespace,
                item.source_agent,
                redundancy_key,
                &status,
                item.confidence,
                canonical_key,
                item.updated_at.to_rfc3339(),
                &payload_json,
            ],
        );

        match update_result {
            Ok(_) => Ok(()),
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE =>
            {
                let duplicate = self
                    .find_by_any_key(redundancy_key, canonical_key, &item.stage)?
                    .context("unique update conflict but no duplicate row found")?;
                if duplicate.id == item.id {
                    return Ok(());
                }

                let tx = conn
                    .transaction()
                    .context("begin duplicate merge transaction")?;
                tx.execute(
                    "DELETE FROM memory_items WHERE id = ?1",
                    params![item.id.to_string()],
                )
                .context("delete conflicting memory item before merge")?;
                tx.execute(
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
                        duplicate.id.to_string(),
                        &kind,
                        &scope,
                        &stage,
                        item.project,
                        item.namespace,
                        item.source_agent,
                        redundancy_key,
                        &status,
                        item.confidence,
                        canonical_key,
                        item.updated_at.to_rfc3339(),
                        &payload_json,
                    ],
                )
                .context("merge duplicate memory item")?;
                tx.commit().context("commit duplicate merge")?;
                Ok(())
            }
            Err(err) => Err(err).context("update memory item"),
        }
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
            let conn = self.connect()?;
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
                    workspace: entity
                        .context
                        .as_ref()
                        .and_then(|context| context.workspace.clone()),
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
            let conn = self.connect()?;
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
            let conn = self.connect()?;
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

    pub fn maintain_runtime(
        &self,
        request: &MaintainReportRequest,
    ) -> anyhow::Result<MaintainReport> {
        let mode = request.mode.trim();
        let mode = if mode.is_empty() { "scan" } else { mode };
        let maintenance_request = MemoryMaintenanceReportRequest {
            project: request.project.clone(),
            namespace: request.namespace.clone(),
            inactive_days: Some(7),
            lookback_days: Some(30),
            min_events: Some(2),
            max_decay: Some(0.5),
            mode: Some(mode.to_string()),
            apply: Some(request.apply),
        };
        let (
            reinforced_candidates,
            cooled_candidates,
            consolidated_candidates,
            stale_items,
            skipped,
            highlights,
        ) = self.maintenance_report(&maintenance_request)?;
        let receipt_id = Uuid::new_v4().to_string();
        let generated_at = chrono::Utc::now();
        let compacted_items = if mode == "compact" {
            consolidated_candidates
        } else {
            0
        };
        let refreshed_items = if mode == "refresh" {
            cooled_candidates
        } else {
            0
        };
        let repaired_items = if mode == "repair" {
            reinforced_candidates
        } else {
            0
        };
        let mut findings = vec![
            format!("memory maintain mode={mode}"),
            format!(
                "scope project={} namespace={} workspace={} session={}",
                request.project.as_deref().unwrap_or("none"),
                request.namespace.as_deref().unwrap_or("none"),
                request.workspace.as_deref().unwrap_or("none"),
                request.session.as_deref().unwrap_or("none")
            ),
            format!(
                "signals stale={} reinforced={} cooled={} consolidated={} skipped={}",
                stale_items,
                reinforced_candidates,
                cooled_candidates,
                consolidated_candidates,
                skipped
            ),
        ];
        if request.apply {
            findings.push("apply requested".to_string());
        }
        findings.extend(
            highlights
                .into_iter()
                .map(|value| format!("highlight: {value}")),
        );
        let response = MaintainReport {
            mode: mode.to_string(),
            receipt_id: Some(receipt_id.clone()),
            compacted_items,
            refreshed_items,
            repaired_items,
            findings,
            generated_at,
        };
        let payload = MaintainReportRecordPayload {
            request: request.clone(),
            response: response.clone(),
        };
        let payload_json =
            serde_json::to_string(&payload).context("serialize maintain report payload")?;
        let conn = self.connect()?;
        conn.execute(
            r#"
            INSERT INTO runtime_maintenance_reports (
              receipt_id, mode, project, namespace, workspace, session, created_at, payload_json
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                receipt_id,
                response.mode.as_str(),
                &request.project,
                &request.namespace,
                &request.workspace,
                &request.session,
                response.generated_at.to_rfc3339(),
                payload_json,
            ],
        )
        .context("insert runtime maintenance report")?;

        Ok(response)
    }

    pub fn entity_for_item(&self, item_id: Uuid) -> anyhow::Result<Option<MemoryEntityRecord>> {
        let conn = self.connect()?;
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
        let conn = self.connect()?;
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

    pub fn rehearse_entity_by_id(
        &self,
        entity_id: Uuid,
        salience_boost: f32,
    ) -> anyhow::Result<Option<MemoryEntityRecord>> {
        let row = {
            let conn = self.connect()?;
            conn.query_row(
                "SELECT entity_key, payload_json FROM memory_entities WHERE id = ?1",
                [entity_id.to_string()],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
        };

        let (entity_key, payload) = match row {
            Ok(row) => row,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(err) => return Err(err).context("fetch memory entity by id for rehearsal"),
        };

        let mut record: MemoryEntityRecord =
            serde_json::from_str(&payload).context("deserialize memory entity payload")?;
        let now = chrono::Utc::now();
        record.rehearsal_count = record.rehearsal_count.saturating_add(1);
        record.last_accessed_at = Some(now);
        record.salience_score = (record.salience_score + salience_boost).min(1.0);
        record.updated_at = now;
        self.upsert_entity(&entity_key, &record)?;
        Ok(Some(record))
    }

    pub fn upsert_entity_link(&self, link: &MemoryEntityLinkRecord) -> anyhow::Result<()> {
        let payload_json = serde_json::to_string(link).context("serialize entity link")?;
        let context_json =
            serde_json::to_string(&link.context).context("serialize entity link context")?;
        let tags_json = serde_json::to_string(&link.tags).context("serialize entity link tags")?;
        let relation_kind =
            serde_json::to_string(&link.relation_kind).context("serialize entity link relation")?;
        let conn = self.connect()?;
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
        let conn = self.connect()?;
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

    pub fn upsert_agent_profile(
        &self,
        request: &AgentProfileUpsertRequest,
    ) -> anyhow::Result<MemoryAgentProfile> {
        let now = chrono::Utc::now();
        let profile = MemoryAgentProfile {
            id: Uuid::new_v4(),
            agent: request.agent.trim().to_string(),
            project: request.project.clone(),
            namespace: request.namespace.clone(),
            preferred_route: request.preferred_route,
            preferred_intent: request.preferred_intent,
            summary_chars: request.summary_chars,
            max_total_chars: request.max_total_chars,
            recall_depth: request.recall_depth,
            source_trust_floor: request.source_trust_floor,
            style_tags: request.style_tags.clone(),
            notes: request.notes.clone(),
            created_at: now,
            updated_at: now,
        };
        let payload_json = serde_json::to_string(&profile).context("serialize agent profile")?;
        let conn = self.connect()?;
        conn.execute(
            r#"
            INSERT INTO memory_agent_profiles (
              id, agent, project, namespace, updated_at, payload_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(agent, project, namespace) DO UPDATE SET
              id = excluded.id,
              updated_at = excluded.updated_at,
              payload_json = excluded.payload_json
            "#,
            params![
                profile.id.to_string(),
                profile.agent,
                profile.project,
                profile.namespace,
                profile.updated_at.to_rfc3339(),
                payload_json,
            ],
        )
        .context("upsert agent profile")?;
        Ok(profile)
    }

    pub fn agent_profile(
        &self,
        request: &AgentProfileRequest,
    ) -> anyhow::Result<Option<MemoryAgentProfile>> {
        let conn = self.connect()?;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT payload_json
                FROM memory_agent_profiles
                ORDER BY updated_at DESC
                "#,
            )
            .context("prepare agent profile query")?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .context("query agent profiles")?;

        for row in rows {
            let payload = row.context("read agent profile row")?;
            let profile: MemoryAgentProfile =
                serde_json::from_str(&payload).context("deserialize agent profile")?;
            if profile.agent != request.agent {
                continue;
            }
            if request
                .project
                .as_ref()
                .is_some_and(|project| profile.project.as_ref() != Some(project))
            {
                continue;
            }
            if request
                .namespace
                .as_ref()
                .is_some_and(|namespace| profile.namespace.as_ref() != Some(namespace))
            {
                continue;
            }
            return Ok(Some(profile));
        }
        Ok(None)
    }

    pub fn source_memory(
        &self,
        request: &SourceMemoryRequest,
    ) -> anyhow::Result<SourceMemoryResponse> {
        let mut grouped: std::collections::BTreeMap<SourceKey, SourceAggregate> =
            std::collections::BTreeMap::new();

        for item in self.list()? {
            if request
                .project
                .as_ref()
                .is_some_and(|value| item.project.as_ref() != Some(value))
            {
                continue;
            }
            if request
                .namespace
                .as_ref()
                .is_some_and(|value| item.namespace.as_ref() != Some(value))
            {
                continue;
            }
            if request
                .workspace
                .as_ref()
                .is_some_and(|value| item.workspace.as_ref() != Some(value))
            {
                continue;
            }
            if request
                .visibility
                .is_some_and(|value| item.visibility != value)
            {
                continue;
            }
            if request
                .source_agent
                .as_ref()
                .is_some_and(|value| item.source_agent.as_ref() != Some(value))
            {
                continue;
            }
            if request
                .source_system
                .as_ref()
                .is_some_and(|value| item.source_system.as_ref() != Some(value))
            {
                continue;
            }

            let key = (
                item.source_agent.clone(),
                item.source_system.clone(),
                item.project.clone(),
                item.namespace.clone(),
                item.workspace.clone(),
                item.visibility,
            );
            let aggregate = grouped.entry(key).or_default();
            aggregate.observe(&item);
        }

        let mut sources = grouped
            .into_iter()
            .map(
                |(
                    (source_agent, source_system, project, namespace, workspace, visibility),
                    aggregate,
                )| {
                    SourceMemoryRecord {
                        source_agent,
                        source_system,
                        project,
                        namespace,
                        workspace,
                        visibility,
                        item_count: aggregate.item_count,
                        active_count: aggregate.active_count,
                        candidate_count: aggregate.candidate_count,
                        derived_count: aggregate.derived_count,
                        synthetic_count: aggregate.synthetic_count,
                        contested_count: aggregate.contested_count,
                        avg_confidence: aggregate.avg_confidence(),
                        trust_score: source_trust_score(
                            aggregate.item_count,
                            aggregate.active_count,
                            aggregate.candidate_count,
                            aggregate.derived_count,
                            aggregate.synthetic_count,
                            aggregate.contested_count,
                            aggregate.avg_confidence(),
                        ),
                        last_seen_at: aggregate.last_seen_at,
                        tags: aggregate.tags(6),
                    }
                },
            )
            .collect::<Vec<_>>();

        sources.sort_by(|a, b| {
            b.trust_score
                .partial_cmp(&a.trust_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.last_seen_at.cmp(&a.last_seen_at))
        });
        let limit = request.limit.unwrap_or(20).min(100);
        sources.truncate(limit);
        Ok(SourceMemoryResponse { sources })
    }

    pub fn workspace_memory(
        &self,
        request: &WorkspaceMemoryRequest,
    ) -> anyhow::Result<WorkspaceMemoryResponse> {
        let mut grouped: std::collections::BTreeMap<WorkspaceKey, WorkspaceAggregate> =
            std::collections::BTreeMap::new();

        for item in self.list()? {
            if request
                .project
                .as_ref()
                .is_some_and(|value| item.project.as_ref() != Some(value))
            {
                continue;
            }
            if request
                .namespace
                .as_ref()
                .is_some_and(|value| item.namespace.as_ref() != Some(value))
            {
                continue;
            }
            if request
                .workspace
                .as_ref()
                .is_some_and(|value| item.workspace.as_ref() != Some(value))
            {
                continue;
            }
            if request
                .visibility
                .is_some_and(|value| item.visibility != value)
            {
                continue;
            }
            if request
                .source_agent
                .as_ref()
                .is_some_and(|value| item.source_agent.as_ref() != Some(value))
            {
                continue;
            }
            if request
                .source_system
                .as_ref()
                .is_some_and(|value| item.source_system.as_ref() != Some(value))
            {
                continue;
            }

            let key = (
                item.project.clone(),
                item.namespace.clone(),
                item.workspace.clone(),
                item.visibility,
            );
            let aggregate = grouped.entry(key).or_default();
            aggregate.observe(&item);
        }

        let mut workspaces = grouped
            .into_iter()
            .map(
                |((project, namespace, workspace, visibility), aggregate)| WorkspaceMemoryRecord {
                    project,
                    namespace,
                    workspace,
                    visibility,
                    item_count: aggregate.source.item_count,
                    active_count: aggregate.source.active_count,
                    candidate_count: aggregate.source.candidate_count,
                    contested_count: aggregate.source.contested_count,
                    source_lane_count: aggregate.source_lanes.len(),
                    avg_confidence: aggregate.source.avg_confidence(),
                    trust_score: source_trust_score(
                        aggregate.source.item_count,
                        aggregate.source.active_count,
                        aggregate.source.candidate_count,
                        aggregate.source.derived_count,
                        aggregate.source.synthetic_count,
                        aggregate.source.contested_count,
                        aggregate.source.avg_confidence(),
                    ),
                    last_seen_at: aggregate.source.last_seen_at,
                    tags: aggregate.source.tags(6),
                },
            )
            .collect::<Vec<_>>();

        workspaces.sort_by(|a, b| {
            b.trust_score
                .total_cmp(&a.trust_score)
                .then_with(|| b.item_count.cmp(&a.item_count))
                .then_with(|| b.last_seen_at.cmp(&a.last_seen_at))
                .then_with(|| a.workspace.cmp(&b.workspace))
        });
        let limit = request.limit.unwrap_or(20).min(100);
        workspaces.truncate(limit);
        Ok(WorkspaceMemoryResponse { workspaces })
    }

    pub fn trust_score_for_item(&self, item: &MemoryItem) -> anyhow::Result<f32> {
        let response = self.source_memory(&SourceMemoryRequest {
            project: item.project.clone(),
            namespace: item.namespace.clone(),
            workspace: item.workspace.clone(),
            visibility: Some(item.visibility),
            source_agent: item.source_agent.clone(),
            source_system: item.source_system.clone(),
            limit: Some(1),
        })?;
        Ok(response
            .sources
            .first()
            .map(|source| source.trust_score)
            .unwrap_or(0.5))
    }

    pub fn list_entities(&self) -> anyhow::Result<Vec<MemoryEntityRecord>> {
        let conn = self.connect()?;
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
        let conn = self.connect()?;
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
        let conn = self.connect()?;
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
        args: RecordEventArgs,
    ) -> anyhow::Result<MemoryEventRecord> {
        let RecordEventArgs {
            event_type,
            summary,
            occurred_at,
            project,
            namespace,
            workspace,
            source_agent,
            source_system,
            source_path,
            related_entity_ids,
            tags,
            context,
            confidence,
            salience_score,
        } = args;
        let now = chrono::Utc::now();
        let event = MemoryEventRecord {
            id: Uuid::new_v4(),
            entity_id: Some(entity.id),
            event_type,
            summary,
            occurred_at,
            recorded_at: now,
            confidence,
            salience_score,
            project,
            namespace,
            workspace,
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
        let conn = self.connect()?;
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
        let conn = self.connect()?;
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
        let conn = self.connect()?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM memory_items", [], |row| row.get(0))
            .context("count memory items")?;
        Ok(count as usize)
    }

    pub fn send_hive_message(
        &self,
        request: &HiveMessageSendRequest,
    ) -> anyhow::Result<HiveMessagesResponse> {
        let message = HiveMessageRecord {
            id: Uuid::new_v4().to_string(),
            kind: request.kind.trim().to_string(),
            from_session: request.from_session.trim().to_string(),
            from_agent: request
                .from_agent
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            to_session: request.to_session.trim().to_string(),
            project: request.project.clone(),
            namespace: request.namespace.clone(),
            workspace: request.workspace.clone(),
            content: request.content.trim().to_string(),
            created_at: chrono::Utc::now(),
            acknowledged_at: None,
        };
        let payload_json =
            serde_json::to_string(&message).context("serialize hive message payload")?;

        let conn = self.connect()?;
        conn.execute(
            r#"
            INSERT INTO hive_messages (
              id, to_session, project, namespace, workspace, created_at, acknowledged_at, payload_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                &message.id,
                &message.to_session,
                &message.project,
                &message.namespace,
                &message.workspace,
                message.created_at.to_rfc3339(),
                Option::<String>::None,
                payload_json,
            ],
        )
        .context("insert hive message")?;

        Ok(HiveMessagesResponse {
            messages: vec![message],
        })
    }

    pub fn hive_inbox(
        &self,
        request: &HiveMessageInboxRequest,
    ) -> anyhow::Result<HiveMessagesResponse> {
        let include_acknowledged = request.include_acknowledged.unwrap_or(false);
        let limit = request.limit.unwrap_or(64).clamp(1, 512) as i64;

        let conn = self.connect()?;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT payload_json
                FROM hive_messages
                WHERE to_session = ?1
                  AND (?2 IS NULL OR project = ?2)
                  AND (?3 IS NULL OR namespace = ?3)
                  AND (?4 IS NULL OR workspace = ?4)
                  AND (?5 = 1 OR acknowledged_at IS NULL)
                ORDER BY created_at DESC
                LIMIT ?6
                "#,
            )
            .context("prepare hive inbox query")?;

        let rows = stmt
            .query_map(
                params![
                    request.session.trim(),
                    request.project.clone(),
                    request.namespace.clone(),
                    request.workspace.clone(),
                    if include_acknowledged { 1 } else { 0 },
                    limit,
                ],
                |row| row.get::<_, String>(0),
            )
            .context("query hive inbox")?;

        let mut messages = Vec::new();
        for row in rows {
            let payload = row.context("read hive inbox row")?;
            messages.push(
                serde_json::from_str::<HiveMessageRecord>(&payload)
                    .context("deserialize hive inbox payload")?,
            );
        }

        Ok(HiveMessagesResponse { messages })
    }

    pub fn ack_hive_message(
        &self,
        request: &HiveMessageAckRequest,
    ) -> anyhow::Result<HiveMessagesResponse> {
        let conn = self.connect()?;
        let payload = conn
            .query_row(
                "SELECT payload_json FROM hive_messages WHERE id = ?1 AND to_session = ?2",
                params![request.id.trim(), request.session.trim()],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .context("fetch hive message for ack")?;

        let Some(payload) = payload else {
            return Ok(HiveMessagesResponse {
                messages: Vec::new(),
            });
        };

        let mut message: HiveMessageRecord =
            serde_json::from_str(&payload).context("deserialize hive message for ack")?;
        message.acknowledged_at = Some(chrono::Utc::now());
        let updated_payload =
            serde_json::to_string(&message).context("serialize acked hive message")?;

        conn.execute(
            "UPDATE hive_messages SET acknowledged_at = ?2, payload_json = ?3 WHERE id = ?1",
            params![
                &message.id,
                message
                    .acknowledged_at
                    .as_ref()
                    .map(chrono::DateTime::to_rfc3339),
                updated_payload,
            ],
        )
        .context("ack hive message")?;

        Ok(HiveMessagesResponse {
            messages: vec![message],
        })
    }

    pub fn acquire_hive_claim(
        &self,
        request: &HiveClaimAcquireRequest,
    ) -> anyhow::Result<HiveClaimsResponse> {
        self.prune_expired_hive_claims()?;

        let expires_at =
            chrono::Utc::now() + chrono::TimeDelta::seconds(request.ttl_seconds.max(1) as i64);
        let claim = HiveClaimRecord {
            scope: request.scope.trim().to_string(),
            session: request.session.trim().to_string(),
            tab_id: request.tab_id.clone(),
            agent: request.agent.clone(),
            effective_agent: request.effective_agent.clone(),
            project: request.project.clone(),
            namespace: request.namespace.clone(),
            workspace: request.workspace.clone(),
            host: request.host.clone(),
            pid: request.pid,
            acquired_at: chrono::Utc::now(),
            expires_at,
        };

        let conn = self.connect()?;
        let existing = conn
            .query_row(
                "SELECT payload_json FROM hive_claims WHERE scope = ?1",
                params![claim.scope.as_str()],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .context("fetch existing hive claim")?;
        if let Some(payload) = existing {
            let existing_claim: HiveClaimRecord =
                serde_json::from_str(&payload).context("deserialize existing hive claim")?;
            if existing_claim.session != claim.session
                && existing_claim.expires_at > chrono::Utc::now()
            {
                anyhow::bail!(
                    "scope '{}' already claimed by {}",
                    claim.scope,
                    existing_claim
                        .effective_agent
                        .as_deref()
                        .unwrap_or(existing_claim.session.as_str())
                );
            }
        }

        let payload_json = serde_json::to_string(&claim).context("serialize hive claim")?;
        conn.execute(
            r#"
            INSERT INTO hive_claims (scope, session, project, namespace, workspace, expires_at, payload_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(scope) DO UPDATE SET
              session = excluded.session,
              project = excluded.project,
              namespace = excluded.namespace,
              workspace = excluded.workspace,
              expires_at = excluded.expires_at,
              payload_json = excluded.payload_json
            "#,
            params![
                claim.scope.as_str(),
                claim.session.as_str(),
                &claim.project,
                &claim.namespace,
                &claim.workspace,
                claim.expires_at.to_rfc3339(),
                payload_json,
            ],
        )
        .context("upsert hive claim")?;

        Ok(HiveClaimsResponse {
            claims: vec![claim],
        })
    }

    pub fn release_hive_claim(
        &self,
        request: &HiveClaimReleaseRequest,
    ) -> anyhow::Result<HiveClaimsResponse> {
        let conn = self.connect()?;
        let payload = conn
            .query_row(
                "SELECT payload_json FROM hive_claims WHERE scope = ?1 AND session = ?2",
                params![request.scope.trim(), request.session.trim()],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .context("fetch hive claim for release")?;
        let Some(payload) = payload else {
            return Ok(HiveClaimsResponse { claims: Vec::new() });
        };
        let claim: HiveClaimRecord =
            serde_json::from_str(&payload).context("deserialize hive claim for release")?;
        conn.execute(
            "DELETE FROM hive_claims WHERE scope = ?1 AND session = ?2",
            params![request.scope.trim(), request.session.trim()],
        )
        .context("release hive claim")?;
        Ok(HiveClaimsResponse {
            claims: vec![claim],
        })
    }

    pub fn transfer_hive_claim(
        &self,
        request: &HiveClaimTransferRequest,
    ) -> anyhow::Result<HiveClaimsResponse> {
        self.prune_expired_hive_claims()?;

        let conn = self.connect()?;
        let payload = conn
            .query_row(
                "SELECT payload_json FROM hive_claims WHERE scope = ?1 AND session = ?2",
                params![request.scope.trim(), request.from_session.trim()],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .context("fetch hive claim for transfer")?;
        let Some(payload) = payload else {
            return Ok(HiveClaimsResponse { claims: Vec::new() });
        };

        let mut claim: HiveClaimRecord =
            serde_json::from_str(&payload).context("deserialize hive claim for transfer")?;
        claim.session = request.to_session.trim().to_string();
        claim.tab_id = request.to_tab_id.clone();
        claim.agent = request.to_agent.clone();
        claim.effective_agent = request.to_effective_agent.clone();
        let updated_payload =
            serde_json::to_string(&claim).context("serialize transferred hive claim")?;
        conn.execute(
            r#"
            UPDATE hive_claims
            SET session = ?2, payload_json = ?3
            WHERE scope = ?1 AND session = ?4
            "#,
            params![
                request.scope.trim(),
                claim.session.as_str(),
                updated_payload,
                request.from_session.trim(),
            ],
        )
        .context("transfer hive claim")?;
        Ok(HiveClaimsResponse {
            claims: vec![claim],
        })
    }

    pub fn recover_hive_claim(
        &self,
        request: &HiveClaimRecoverRequest,
    ) -> anyhow::Result<HiveClaimsResponse> {
        self.prune_expired_hive_claims()?;

        let conn = self.connect()?;
        let payload = conn
            .query_row(
                "SELECT payload_json FROM hive_claims WHERE scope = ?1 AND session = ?2",
                params![request.scope.trim(), request.from_session.trim()],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .context("fetch hive claim for recovery")?;
        let Some(payload) = payload else {
            return Ok(HiveClaimsResponse { claims: Vec::new() });
        };

        let mut claim: HiveClaimRecord =
            serde_json::from_str(&payload).context("deserialize hive claim for recovery")?;

        if let Some(to_session) = request
            .to_session
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            claim.session = to_session.to_string();
            claim.tab_id = request.to_tab_id.clone();
            claim.agent = request.to_agent.clone();
            claim.effective_agent = request.to_effective_agent.clone();
            let updated_payload =
                serde_json::to_string(&claim).context("serialize recovered hive claim")?;
            conn.execute(
                r#"
                UPDATE hive_claims
                SET session = ?2, payload_json = ?3
                WHERE scope = ?1 AND session = ?4
                "#,
                params![
                    request.scope.trim(),
                    claim.session.as_str(),
                    updated_payload,
                    request.from_session.trim(),
                ],
            )
            .context("recover hive claim into new owner")?;
            Ok(HiveClaimsResponse {
                claims: vec![claim],
            })
        } else {
            conn.execute(
                "DELETE FROM hive_claims WHERE scope = ?1 AND session = ?2",
                params![request.scope.trim(), request.from_session.trim()],
            )
            .context("delete hive claim during recovery")?;
            Ok(HiveClaimsResponse {
                claims: vec![claim],
            })
        }
    }

    pub fn hive_claims(&self, request: &HiveClaimsRequest) -> anyhow::Result<HiveClaimsResponse> {
        self.prune_expired_hive_claims()?;

        let limit = request.limit.unwrap_or(256).clamp(1, 1024) as i64;
        let active_only = request.active_only.unwrap_or(true);
        let conn = self.connect()?;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT payload_json
                FROM hive_claims
                WHERE (?1 IS NULL OR session = ?1)
                  AND (?2 IS NULL OR project = ?2)
                  AND (?3 IS NULL OR namespace = ?3)
                  AND (?4 IS NULL OR workspace = ?4)
                  AND (?5 = 0 OR expires_at > ?6)
                ORDER BY expires_at DESC
                LIMIT ?7
                "#,
            )
            .context("prepare hive claims query")?;
        let now = chrono::Utc::now().to_rfc3339();
        let rows = stmt
            .query_map(
                params![
                    request.session.clone(),
                    request.project.clone(),
                    request.namespace.clone(),
                    request.workspace.clone(),
                    if active_only { 1 } else { 0 },
                    now,
                    limit,
                ],
                |row| row.get::<_, String>(0),
            )
            .context("query hive claims")?;

        let mut claims = Vec::new();
        for row in rows {
            let payload = row.context("read hive claim row")?;
            claims.push(
                serde_json::from_str::<HiveClaimRecord>(&payload)
                    .context("deserialize hive claim payload")?,
            );
        }
        Ok(HiveClaimsResponse { claims })
    }

    pub fn upsert_hive_session(
        &self,
        request: &HiveSessionUpsertRequest,
    ) -> anyhow::Result<HiveSessionsResponse> {
        self.prune_stale_hive_sessions()?;

        let now = chrono::Utc::now();
        let session = request.session.trim().to_string();
        let project = request
            .project
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let namespace = request
            .namespace
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let workspace = request
            .workspace
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let repo_root = request
            .repo_root
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let worktree_root = request
            .worktree_root
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let branch = request
            .branch
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let record = HiveSessionRecord {
            session: session.clone(),
            tab_id: request
                .tab_id
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            agent: request
                .agent
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            effective_agent: request
                .effective_agent
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            hive_system: request
                .hive_system
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            hive_role: request
                .hive_role
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            worker_name: request
                .worker_name
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            display_name: request
                .display_name
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            role: request
                .role
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            capabilities: request
                .capabilities
                .iter()
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect(),
            hive_groups: request
                .hive_groups
                .iter()
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect(),
            lane_id: request
                .lane_id
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            hive_group_goal: request
                .hive_group_goal
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            authority: request
                .authority
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            heartbeat_model: request
                .heartbeat_model
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            project: project.clone(),
            namespace: namespace.clone(),
            workspace: workspace.clone(),
            repo_root: repo_root.clone(),
            worktree_root: worktree_root.clone(),
            branch: branch.clone(),
            base_branch: request
                .base_branch
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            visibility: request
                .visibility
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            base_url: request
                .base_url
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            base_url_healthy: request.base_url_healthy,
            host: request
                .host
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            pid: request.pid,
            topic_claim: request
                .topic_claim
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            scope_claims: request
                .scope_claims
                .iter()
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect(),
            task_id: request
                .task_id
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            focus: request
                .focus
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            pressure: request
                .pressure
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            next_recovery: request
                .next_recovery
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            next_action: request
                .next_action
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            needs_help: request.needs_help,
            needs_review: request.needs_review,
            handoff_state: request
                .handoff_state
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            confidence: request
                .confidence
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            risk: request
                .risk
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            status: request
                .status
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("live")
                .to_string(),
            last_seen: now,
        };
        let payload_json = serde_json::to_string(&record).context("serialize hive session")?;
        let session_key = hive_session_key(
            &session,
            HiveSessionKeyArgs {
                project: project.as_deref(),
                namespace: namespace.as_deref(),
                workspace: workspace.as_deref(),
                repo_root: repo_root.as_deref(),
                worktree_root: worktree_root.as_deref(),
                branch: branch.as_deref(),
                agent: record.agent.as_deref(),
                effective_agent: record.effective_agent.as_deref(),
                hive_system: record.hive_system.as_deref(),
                hive_role: record.hive_role.as_deref(),
                host: record.host.as_deref(),
            },
        );

        let mut conn = self.connect()?;
        let tx = conn
            .transaction()
            .context("begin hive session upsert transaction")?;
        tx.execute(
            r#"
            INSERT INTO hive_sessions (
              session_key, session, project, namespace, workspace, repo_root, worktree_root, branch, hive_system, hive_role, host, status, last_seen, payload_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            ON CONFLICT(session_key) DO UPDATE SET
              session = excluded.session,
              project = excluded.project,
              namespace = excluded.namespace,
              workspace = excluded.workspace,
              repo_root = excluded.repo_root,
              worktree_root = excluded.worktree_root,
              branch = excluded.branch,
              hive_system = excluded.hive_system,
              hive_role = excluded.hive_role,
              host = excluded.host,
              status = excluded.status,
              last_seen = excluded.last_seen,
              payload_json = excluded.payload_json
            "#,
            params![
                session_key,
                record.session,
                &record.project,
                &record.namespace,
                &record.workspace,
                &record.repo_root,
                &record.worktree_root,
                &record.branch,
                &record.hive_system,
                &record.hive_role,
                &record.host,
                record.status,
                record.last_seen.to_rfc3339(),
                payload_json,
            ],
        )
        .context("upsert hive session")?;
        tx.execute(
            "DELETE FROM hive_session_groups WHERE session_key = ?1",
            params![session_key],
        )
        .context("clear hive session groups")?;

        {
            let mut insert_group_stmt = tx.prepare(
                "INSERT OR REPLACE INTO hive_session_groups (session_key, hive_group) VALUES (?1, ?2)",
            )?;
            for hive_group in record.hive_groups.iter() {
                insert_group_stmt.execute(params![session_key, hive_group])?;
            }
        }
        tx.commit()
            .context("commit hive session upsert transaction")?;

        Ok(HiveSessionsResponse {
            sessions: vec![record],
        })
    }

    pub fn hive_sessions(
        &self,
        request: &HiveSessionsRequest,
    ) -> anyhow::Result<HiveSessionsResponse> {
        self.prune_stale_hive_sessions()?;

        let active_only = request.active_only.unwrap_or(true);
        let limit = request.limit.unwrap_or(128).clamp(1, 1024) as i64;
        let active_cutoff = (chrono::Utc::now() - chrono::TimeDelta::minutes(15)).to_rfc3339();
        let repo_root = request
            .repo_root
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let worktree_root = request
            .worktree_root
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let branch = request
            .branch
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let hive_system = request
            .hive_system
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let hive_role = request
            .hive_role
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let host = request
            .host
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let hive_group = request
            .hive_group
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);

        let conn = self.connect()?;
        let session_filter = request
            .session
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let project_filter = request
            .project
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let namespace_filter = request
            .namespace
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let workspace_filter = request
            .workspace
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());

        let mut stmt = conn
            .prepare(
                r#"
                SELECT payload_json
                FROM hive_sessions
                WHERE (?1 IS NULL OR session = ?1)
                  AND (?2 IS NULL OR project = ?2)
                  AND (?3 IS NULL OR namespace = ?3)
                  AND (?4 IS NULL OR workspace = ?4)
                  AND (?5 IS NULL OR repo_root = ?5)
                  AND (?6 IS NULL OR worktree_root = ?6)
                  AND (?7 IS NULL OR branch = ?7)
                  AND (?8 = 0 OR last_seen >= ?9)
                  AND (?10 IS NULL OR hive_system = ?10)
                  AND (?11 IS NULL OR hive_role = ?11)
                  AND (?12 IS NULL OR host = ?12)
                  AND (
                    ?13 IS NULL OR EXISTS (
                      SELECT 1
                      FROM hive_session_groups
                      WHERE hive_session_groups.session_key = hive_sessions.session_key
                        AND hive_session_groups.hive_group = ?13
                    )
                  )
                ORDER BY last_seen DESC
                LIMIT ?14
                "#,
            )
            .context("prepare hive sessions query")?;
        let rows = stmt
            .query_map(
                params![
                    session_filter,
                    project_filter,
                    namespace_filter,
                    workspace_filter,
                    repo_root,
                    worktree_root,
                    branch,
                    if active_only { 1 } else { 0 },
                    active_cutoff,
                    hive_system,
                    hive_role,
                    host,
                    hive_group,
                    limit,
                ],
                |row| row.get::<_, String>(0),
            )
            .context("query hive sessions")?;

        let mut sessions = Vec::new();
        for row in rows {
            let payload = row.context("read hive session row")?;
            sessions.push(
                serde_json::from_str::<HiveSessionRecord>(&payload)
                    .context("deserialize hive session payload")?,
            );
        }

        Ok(HiveSessionsResponse { sessions })
    }

    pub fn hive_board(&self, request: &HiveBoardRequest) -> anyhow::Result<HiveBoardResponse> {
        let sessions = self
            .hive_sessions(&HiveSessionsRequest {
                session: None,
                project: request.project.clone(),
                namespace: request.namespace.clone(),
                workspace: request.workspace.clone(),
                repo_root: None,
                worktree_root: None,
                branch: None,
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(false),
                limit: Some(256),
            })?
            .sessions;
        let tasks = self
            .hive_tasks(&HiveTasksRequest {
                session: None,
                project: request.project.clone(),
                namespace: request.namespace.clone(),
                workspace: request.workspace.clone(),
                active_only: Some(true),
                limit: Some(256),
            })?
            .tasks;
        let receipts = self
            .hive_coordination_receipts(&HiveCoordinationReceiptsRequest {
                session: None,
                project: request.project.clone(),
                namespace: request.namespace.clone(),
                workspace: request.workspace.clone(),
                limit: Some(256),
            })?
            .receipts;

        let queen_session = sessions
            .iter()
            .find(|session| {
                matches!(
                    session.role.as_deref().or(session.hive_role.as_deref()),
                    Some("queen" | "orchestrator" | "memory-control-plane")
                ) || matches!(
                    session.authority.as_deref(),
                    Some("coordinator" | "canonical")
                )
            })
            .map(|session| session.session.clone());

        let active_bees = sessions
            .iter()
            .filter(|session| session.status == "active")
            .cloned()
            .collect::<Vec<_>>();
        let stale_bees = sessions
            .iter()
            .filter(|session| session.status != "active")
            .map(|session| session.session.clone())
            .collect::<Vec<_>>();
        let review_queue = tasks
            .iter()
            .filter(|task| task.review_requested || task.coordination_mode == "shared_review")
            .map(|task| {
                format!(
                    "{} -> {}",
                    task.task_id,
                    task.session.as_deref().unwrap_or("unassigned")
                )
            })
            .collect::<Vec<_>>();
        let lane_faults = receipts
            .iter()
            .filter(|receipt| receipt.kind.starts_with("lane_"))
            .map(|receipt| receipt.summary.clone())
            .collect::<Vec<_>>();
        let overlap_risks = receipts
            .iter()
            .filter(|receipt| {
                receipt.kind.contains("overlap")
                    || receipt.summary.contains("overlap")
                    || receipt.summary.contains("scope")
            })
            .map(|receipt| receipt.summary.clone())
            .collect::<Vec<_>>();
        let blocked_bees = receipts
            .iter()
            .filter(|receipt| receipt.kind == "queen_deny" || receipt.kind.starts_with("lane_"))
            .map(|receipt| receipt.summary.clone())
            .collect::<Vec<_>>();
        let mut recommended_actions = Vec::new();
        for session in &stale_bees {
            recommended_actions.push(format!("retire {}", session));
        }
        for receipt in receipts
            .iter()
            .filter(|receipt| receipt.kind == "queen_deny")
        {
            recommended_actions.push(format!("reroute {}", receipt.summary));
        }

        Ok(HiveBoardResponse {
            queen_session,
            active_bees,
            blocked_bees,
            stale_bees,
            review_queue,
            overlap_risks,
            lane_faults,
            recommended_actions,
        })
    }

    pub fn hive_roster(
        &self,
        request: &memd_schema::HiveRosterRequest,
    ) -> anyhow::Result<HiveRosterResponse> {
        let sessions = self
            .hive_sessions(&HiveSessionsRequest {
                session: None,
                project: request.project.clone(),
                namespace: request.namespace.clone(),
                workspace: request.workspace.clone(),
                repo_root: None,
                worktree_root: None,
                branch: None,
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(false),
                limit: Some(256),
            })?
            .sessions;
        let queen_session = sessions
            .iter()
            .find(|session| {
                matches!(
                    session.role.as_deref().or(session.hive_role.as_deref()),
                    Some("queen" | "orchestrator" | "memory-control-plane")
                ) || matches!(
                    session.authority.as_deref(),
                    Some("coordinator" | "canonical")
                )
            })
            .map(|session| session.session.clone());

        Ok(HiveRosterResponse {
            project: request
                .project
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            namespace: request
                .namespace
                .clone()
                .unwrap_or_else(|| "default".to_string()),
            queen_session,
            bees: sessions,
        })
    }

    pub fn hive_follow(
        &self,
        request: &memd_schema::HiveFollowRequest,
    ) -> anyhow::Result<memd_schema::HiveFollowResponse> {
        let sessions = self
            .hive_sessions(&HiveSessionsRequest {
                session: None,
                project: request.project.clone(),
                namespace: request.namespace.clone(),
                workspace: request.workspace.clone(),
                repo_root: None,
                worktree_root: None,
                branch: None,
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(false),
                limit: Some(256),
            })?
            .sessions;
        let target = sessions
            .iter()
            .find(|session| session.session == request.session)
            .cloned()
            .context("hive follow session not found")?;
        let inbox = self.hive_coordination_inbox(&HiveCoordinationInboxRequest {
            session: request.session.clone(),
            project: request.project.clone(),
            namespace: request.namespace.clone(),
            workspace: request.workspace.clone(),
            limit: Some(32),
        })?;
        let recent_receipts = self
            .hive_coordination_receipts(&HiveCoordinationReceiptsRequest {
                session: None,
                project: request.project.clone(),
                namespace: request.namespace.clone(),
                workspace: request.workspace.clone(),
                limit: Some(64),
            })?
            .receipts
            .into_iter()
            .filter(|receipt| {
                receipt.actor_session == request.session
                    || receipt.target_session.as_deref() == Some(request.session.as_str())
                    || receipt.task_id.as_deref().is_some_and(|task_id| {
                        inbox.owned_tasks.iter().any(|task| task.task_id == task_id)
                    })
            })
            .take(8)
            .collect::<Vec<_>>();

        let overlap_risk = request
            .current_session
            .as_deref()
            .and_then(|current_session| {
                let current = sessions
                    .iter()
                    .find(|session| session.session == current_session)?;
                hive_follow_overlap_risk(current, &target)
            });
        let recommended_action = if overlap_risk.is_some() {
            "coordinate_now".to_string()
        } else if !inbox.review_tasks.is_empty()
            || !inbox.help_tasks.is_empty()
            || !inbox.messages.is_empty()
        {
            "watch_and_coordinate".to_string()
        } else {
            "safe_to_continue".to_string()
        };

        Ok(memd_schema::HiveFollowResponse {
            current_session: request.current_session.clone(),
            target: target.clone(),
            work_summary: target
                .topic_claim
                .clone()
                .or_else(|| target.focus.clone())
                .unwrap_or_else(|| "none".to_string()),
            touch_points: if target.scope_claims.is_empty() {
                Vec::new()
            } else {
                target.scope_claims.clone()
            },
            next_action: target.next_action.clone(),
            messages: inbox.messages,
            owned_tasks: inbox.owned_tasks,
            help_tasks: inbox.help_tasks,
            review_tasks: inbox.review_tasks,
            recent_receipts,
            overlap_risk,
            recommended_action,
        })
    }

    pub fn retire_hive_session(
        &self,
        request: &HiveSessionRetireRequest,
    ) -> anyhow::Result<HiveSessionRetireResponse> {
        let session = request.session.trim();
        anyhow::ensure!(!session.is_empty(), "session must not be empty");

        let project = request
            .project
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let namespace = request
            .namespace
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let workspace = request
            .workspace
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let repo_root = request
            .repo_root
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let worktree_root = request
            .worktree_root
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let branch = request
            .branch
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let agent = request
            .agent
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let effective_agent = request
            .effective_agent
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let hive_system = request
            .hive_system
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let hive_role = request
            .hive_role
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let host = request
            .host
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());

        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT session_key, payload_json
            FROM hive_sessions
            WHERE session = ?1
              AND (?2 IS NULL OR project = ?2)
              AND (?3 IS NULL OR namespace = ?3)
              AND (?4 IS NULL OR workspace = ?4)
              AND (?5 IS NULL OR repo_root = ?5)
              AND (?6 IS NULL OR worktree_root = ?6)
              AND (?7 IS NULL OR branch = ?7)
              AND (?8 IS NULL OR json_extract(payload_json, '$.agent') = ?8)
              AND (?9 IS NULL OR json_extract(payload_json, '$.effective_agent') = ?9)
              AND (?10 IS NULL OR hive_system = ?10)
              AND (?11 IS NULL OR hive_role = ?11)
              AND (?12 IS NULL OR host = ?12)
            "#,
        )?;
        let rows = stmt.query_map(
            params![
                session,
                project,
                namespace,
                workspace,
                repo_root,
                worktree_root,
                branch,
                agent,
                effective_agent,
                hive_system,
                hive_role,
                host,
            ],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )?;

        let mut targets = Vec::new();
        for row in rows {
            let (session_key, payload) = row.context("read hive session retire row")?;
            targets.push((
                session_key,
                serde_json::from_str::<HiveSessionRecord>(&payload)
                    .context("deserialize retired hive session payload")?,
            ));
        }
        if targets.is_empty() {
            return Ok(HiveSessionRetireResponse {
                retired: 0,
                sessions: Vec::new(),
            });
        }

        let mut conn = self.connect()?;
        let tx = conn
            .transaction()
            .context("begin hive session retire transaction")?;
        for (session_key, _) in &targets {
            tx.execute(
                "DELETE FROM hive_session_groups WHERE session_key = ?1",
                params![session_key],
            )?;
            tx.execute(
                "DELETE FROM hive_sessions WHERE session_key = ?1",
                params![session_key],
            )?;
        }
        tx.commit()
            .context("commit hive session retire transaction")?;

        Ok(HiveSessionRetireResponse {
            retired: targets.len(),
            sessions: targets.into_iter().map(|(_, record)| record).collect(),
        })
    }

    pub fn auto_retire_stale_hive_sessions(
        &self,
        project: Option<&str>,
        namespace: Option<&str>,
        workspace: Option<&str>,
        now: chrono::DateTime<chrono::Utc>,
    ) -> anyhow::Result<HiveSessionAutoRetireResponse> {
        self.prune_expired_hive_claims()?;
        self.prune_stale_hive_sessions()?;

        let project = project
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let namespace = namespace
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let workspace = workspace
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);

        let sessions = self
            .hive_sessions(&HiveSessionsRequest {
                session: None,
                project: project.clone(),
                namespace: namespace.clone(),
                workspace: workspace.clone(),
                repo_root: None,
                worktree_root: None,
                branch: None,
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(false),
                limit: Some(512),
            })?
            .sessions;
        let tasks = self
            .hive_tasks(&HiveTasksRequest {
                session: None,
                project: project.clone(),
                namespace: namespace.clone(),
                workspace: workspace.clone(),
                active_only: Some(true),
                limit: Some(512),
            })?
            .tasks;
        let claims = self
            .hive_claims(&HiveClaimsRequest {
                session: None,
                project: project.clone(),
                namespace: namespace.clone(),
                workspace: workspace.clone(),
                active_only: Some(true),
                limit: Some(512),
            })?
            .claims;
        let receipts = self
            .hive_coordination_receipts(&HiveCoordinationReceiptsRequest {
                session: None,
                project: project.clone(),
                namespace: namespace.clone(),
                workspace: workspace.clone(),
                limit: Some(512),
            })?
            .receipts;

        let stale_cutoff = now - chrono::TimeDelta::minutes(15);
        let retireable = sessions
            .into_iter()
            .filter(|session| session.last_seen < stale_cutoff)
            .filter(|session| {
                !tasks.iter().any(|task| {
                    task.session.as_deref() == Some(session.session.as_str())
                        && task.status != "done"
                        && task.status != "closed"
                })
            })
            .filter(|session| !claims.iter().any(|claim| claim.session == session.session))
            .filter(|session| {
                !receipts.iter().any(|receipt| {
                    receipt.kind == "queen_handoff"
                        && (receipt.actor_session == session.session
                            || receipt.target_session.as_deref() == Some(session.session.as_str()))
                })
            })
            .collect::<Vec<_>>();

        let mut retired = Vec::new();
        for session in retireable {
            let response = self.retire_hive_session(&HiveSessionRetireRequest {
                session: session.session.clone(),
                project: session.project.clone(),
                namespace: session.namespace.clone(),
                workspace: session.workspace.clone(),
                repo_root: session.repo_root.clone(),
                worktree_root: session.worktree_root.clone(),
                branch: session.branch.clone(),
                agent: session.agent.clone(),
                effective_agent: session.effective_agent.clone(),
                hive_system: session.hive_system.clone(),
                hive_role: session.hive_role.clone(),
                host: session.host.clone(),
                reason: Some("auto_retire_stale_session".to_string()),
            })?;
            if response.retired > 0 {
                retired.push(session.session);
            }
        }

        Ok(HiveSessionAutoRetireResponse { retired })
    }

    pub fn upsert_hive_task(
        &self,
        request: &HiveTaskUpsertRequest,
    ) -> anyhow::Result<HiveTasksResponse> {
        let conn = self.connect()?;
        let existing = conn
            .query_row(
                "SELECT payload_json FROM hive_tasks WHERE task_id = ?1",
                params![request.task_id.trim()],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .context("fetch existing hive task")?;

        let now = chrono::Utc::now();
        let mut task = if let Some(payload) = existing {
            let mut task: HiveTaskRecord =
                serde_json::from_str(&payload).context("deserialize existing hive task")?;
            task.title = request.title.trim().to_string();
            task.description = request
                .description
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string);
            if let Some(mode) = request.coordination_mode.as_deref() {
                let trimmed = mode.trim();
                if !trimmed.is_empty() {
                    task.coordination_mode = trimmed.to_string();
                }
            }
            if let Some(status) = request.status.as_deref() {
                let trimmed = status.trim();
                if !trimmed.is_empty() {
                    task.status = trimmed.to_string();
                }
            }
            task.session = request.session.clone().or(task.session);
            task.agent = request.agent.clone().or(task.agent);
            task.effective_agent = request.effective_agent.clone().or(task.effective_agent);
            task.project = request.project.clone().or(task.project);
            task.namespace = request.namespace.clone().or(task.namespace);
            task.workspace = request.workspace.clone().or(task.workspace);
            if !request.claim_scopes.is_empty() {
                task.claim_scopes = request
                    .claim_scopes
                    .iter()
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
                    .collect();
            }
            if let Some(value) = request.help_requested {
                task.help_requested = value;
            }
            if let Some(value) = request.review_requested {
                task.review_requested = value;
            }
            task.updated_at = now;
            task
        } else {
            HiveTaskRecord {
                task_id: request.task_id.trim().to_string(),
                title: request.title.trim().to_string(),
                description: request
                    .description
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string),
                status: request
                    .status
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .unwrap_or("active")
                    .to_string(),
                coordination_mode: request
                    .coordination_mode
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .unwrap_or("exclusive_write")
                    .to_string(),
                session: request.session.clone(),
                agent: request.agent.clone(),
                effective_agent: request.effective_agent.clone(),
                project: request.project.clone(),
                namespace: request.namespace.clone(),
                workspace: request.workspace.clone(),
                claim_scopes: request
                    .claim_scopes
                    .iter()
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
                    .collect(),
                help_requested: request.help_requested.unwrap_or(false),
                review_requested: request.review_requested.unwrap_or(false),
                created_at: now,
                updated_at: now,
            }
        };

        if task.status.trim().is_empty() {
            task.status = "active".to_string();
        }

        let payload_json = serde_json::to_string(&task).context("serialize hive task")?;
        conn.execute(
            r#"
            INSERT INTO hive_tasks (task_id, session, project, namespace, workspace, status, updated_at, payload_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(task_id) DO UPDATE SET
              session = excluded.session,
              project = excluded.project,
              namespace = excluded.namespace,
              workspace = excluded.workspace,
              status = excluded.status,
              updated_at = excluded.updated_at,
              payload_json = excluded.payload_json
            "#,
            params![
                task.task_id.as_str(),
                &task.session,
                &task.project,
                &task.namespace,
                &task.workspace,
                task.status.as_str(),
                task.updated_at.to_rfc3339(),
                payload_json,
            ],
        )
        .context("upsert hive task")?;

        Ok(HiveTasksResponse { tasks: vec![task] })
    }

    pub fn assign_hive_task(
        &self,
        request: &HiveTaskAssignRequest,
    ) -> anyhow::Result<HiveTasksResponse> {
        let conn = self.connect()?;
        let payload = conn
            .query_row(
                "SELECT payload_json FROM hive_tasks WHERE task_id = ?1",
                params![request.task_id.trim()],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .context("fetch hive task for assignment")?;
        let Some(payload) = payload else {
            return Ok(HiveTasksResponse { tasks: Vec::new() });
        };

        let mut task: HiveTaskRecord =
            serde_json::from_str(&payload).context("deserialize hive task for assignment")?;
        if let Some(from_session) = request
            .from_session
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            if task.session.as_deref() != Some(from_session) {
                anyhow::bail!("task '{}' is not owned by {}", task.task_id, from_session);
            }
        }
        task.session = Some(request.to_session.trim().to_string());
        task.agent = request.to_agent.clone();
        task.effective_agent = request.to_effective_agent.clone();
        task.status = "assigned".to_string();
        task.updated_at = chrono::Utc::now();
        let payload_json = serde_json::to_string(&task).context("serialize assigned hive task")?;
        conn.execute(
            r#"
            UPDATE hive_tasks
            SET session = ?2, status = ?3, updated_at = ?4, payload_json = ?5
            WHERE task_id = ?1
            "#,
            params![
                task.task_id.as_str(),
                &task.session,
                task.status.as_str(),
                task.updated_at.to_rfc3339(),
                payload_json,
            ],
        )
        .context("assign hive task")?;
        Ok(HiveTasksResponse { tasks: vec![task] })
    }

    pub fn hive_tasks(&self, request: &HiveTasksRequest) -> anyhow::Result<HiveTasksResponse> {
        let limit = request.limit.unwrap_or(128).clamp(1, 1024) as i64;
        let active_only = request.active_only.unwrap_or(true);
        let conn = self.connect()?;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT payload_json
                FROM hive_tasks
                WHERE (?1 IS NULL OR session = ?1)
                  AND (?2 IS NULL OR project = ?2)
                  AND (?3 IS NULL OR namespace = ?3)
                  AND (?4 IS NULL OR workspace = ?4)
                  AND (?5 = 0 OR status NOT IN ('done', 'closed'))
                ORDER BY updated_at DESC
                LIMIT ?6
                "#,
            )
            .context("prepare hive tasks query")?;
        let rows = stmt
            .query_map(
                params![
                    request.session.clone(),
                    request.project.clone(),
                    request.namespace.clone(),
                    request.workspace.clone(),
                    if active_only { 1 } else { 0 },
                    limit,
                ],
                |row| row.get::<_, String>(0),
            )
            .context("query hive tasks")?;

        let mut tasks = Vec::new();
        for row in rows {
            let payload = row.context("read hive task row")?;
            tasks.push(
                serde_json::from_str::<HiveTaskRecord>(&payload)
                    .context("deserialize hive task payload")?,
            );
        }
        Ok(HiveTasksResponse { tasks })
    }

    pub fn hive_coordination_inbox(
        &self,
        request: &HiveCoordinationInboxRequest,
    ) -> anyhow::Result<HiveCoordinationInboxResponse> {
        let messages = self
            .hive_inbox(&HiveMessageInboxRequest {
                session: request.session.clone(),
                project: request.project.clone(),
                namespace: request.namespace.clone(),
                workspace: request.workspace.clone(),
                include_acknowledged: Some(false),
                limit: request.limit,
            })?
            .messages;

        let tasks = self
            .hive_tasks(&HiveTasksRequest {
                session: None,
                project: request.project.clone(),
                namespace: request.namespace.clone(),
                workspace: request.workspace.clone(),
                active_only: Some(true),
                limit: request.limit,
            })?
            .tasks;

        let mut owned_tasks = Vec::new();
        let mut help_tasks = Vec::new();
        let mut review_tasks = Vec::new();
        for task in tasks {
            if task.session.as_deref() == Some(request.session.as_str()) {
                owned_tasks.push(task.clone());
            }
            if task.help_requested {
                help_tasks.push(task.clone());
            }
            if task.review_requested {
                review_tasks.push(task);
            }
        }

        Ok(HiveCoordinationInboxResponse {
            messages,
            owned_tasks,
            help_tasks,
            review_tasks,
        })
    }

    pub fn record_hive_coordination_receipt(
        &self,
        request: &HiveCoordinationReceiptRequest,
    ) -> anyhow::Result<HiveCoordinationReceiptsResponse> {
        let receipt = HiveCoordinationReceiptRecord {
            id: Uuid::new_v4().to_string(),
            kind: request.kind.trim().to_string(),
            actor_session: request.actor_session.trim().to_string(),
            actor_agent: request.actor_agent.clone(),
            target_session: request.target_session.clone(),
            task_id: request.task_id.clone(),
            scope: request.scope.clone(),
            project: request.project.clone(),
            namespace: request.namespace.clone(),
            workspace: request.workspace.clone(),
            summary: request.summary.trim().to_string(),
            created_at: chrono::Utc::now(),
        };
        let payload_json =
            serde_json::to_string(&receipt).context("serialize coordination receipt")?;
        let conn = self.connect()?;
        conn.execute(
            r#"
            INSERT INTO hive_coordination_receipts (id, actor_session, project, namespace, workspace, created_at, payload_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                receipt.id.as_str(),
                receipt.actor_session.as_str(),
                &receipt.project,
                &receipt.namespace,
                &receipt.workspace,
                receipt.created_at.to_rfc3339(),
                payload_json,
            ],
        )
        .context("insert coordination receipt")?;
        Ok(HiveCoordinationReceiptsResponse {
            receipts: vec![receipt],
        })
    }

    pub fn hive_coordination_receipts(
        &self,
        request: &HiveCoordinationReceiptsRequest,
    ) -> anyhow::Result<HiveCoordinationReceiptsResponse> {
        let limit = request.limit.unwrap_or(128).clamp(1, 1024) as i64;
        let conn = self.connect()?;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT payload_json
                FROM hive_coordination_receipts
                WHERE (?1 IS NULL OR actor_session = ?1)
                  AND (?2 IS NULL OR project = ?2)
                  AND (?3 IS NULL OR namespace = ?3)
                  AND (?4 IS NULL OR workspace = ?4)
                ORDER BY created_at DESC
                LIMIT ?5
                "#,
            )
            .context("prepare coordination receipts query")?;
        let rows = stmt
            .query_map(
                params![
                    request.session.clone(),
                    request.project.clone(),
                    request.namespace.clone(),
                    request.workspace.clone(),
                    limit,
                ],
                |row| row.get::<_, String>(0),
            )
            .context("query coordination receipts")?;
        let mut receipts = Vec::new();
        for row in rows {
            let payload = row.context("read coordination receipt row")?;
            receipts.push(
                serde_json::from_str::<HiveCoordinationReceiptRecord>(&payload)
                    .context("deserialize coordination receipt payload")?,
            );
        }
        Ok(HiveCoordinationReceiptsResponse { receipts })
    }

    pub fn record_skill_policy_apply_receipt(
        &self,
        request: &SkillPolicyApplyRequest,
    ) -> anyhow::Result<SkillPolicyApplyResponse> {
        let receipt = SkillPolicyApplyReceipt {
            id: Uuid::new_v4().to_string(),
            bundle_root: request.bundle_root.trim().to_string(),
            runtime_defaulted: request.runtime_defaulted,
            source_queue_path: request.source_queue_path.trim().to_string(),
            applied_count: request.applied_count,
            skipped_count: request.skipped_count,
            project: request.project.clone(),
            namespace: request.namespace.clone(),
            workspace: request.workspace.clone(),
            created_at: chrono::Utc::now(),
        };
        let payload_json = serde_json::to_string(&SkillPolicyApplyRecordPayload {
            receipt: receipt.clone(),
            request: request.clone(),
        })
        .context("serialize skill policy apply receipt")?;
        let conn = self.connect()?;
        conn.execute(
            r#"
            INSERT INTO skill_policy_apply_receipts (id, project, namespace, workspace, created_at, payload_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                receipt.id.as_str(),
                &receipt.project,
                &receipt.namespace,
                &receipt.workspace,
                receipt.created_at.to_rfc3339(),
                payload_json,
            ],
        )
        .context("insert skill policy apply receipt")?;

        for record in request.applied.iter() {
            let activation = SkillPolicyActivationEntry {
                receipt_id: receipt.id.clone(),
                bundle_root: receipt.bundle_root.clone(),
                runtime_defaulted: receipt.runtime_defaulted,
                source_queue_path: receipt.source_queue_path.clone(),
                record: record.clone(),
                project: receipt.project.clone(),
                namespace: receipt.namespace.clone(),
                workspace: receipt.workspace.clone(),
                created_at: receipt.created_at,
            };
            let activation_json = serde_json::to_string(&activation)
                .context("serialize skill policy activation entry")?;
            conn.execute(
                r#"
                INSERT INTO skill_policy_activations (id, receipt_id, project, namespace, workspace, created_at, payload_json)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                "#,
                params![
                    Uuid::new_v4().to_string(),
                    receipt.id.as_str(),
                    &activation.project,
                    &activation.namespace,
                    &activation.workspace,
                    activation.created_at.to_rfc3339(),
                    activation_json,
                ],
            )
            .context("insert skill policy activation entry")?;
        }
        Ok(SkillPolicyApplyResponse { receipt })
    }

    pub fn skill_policy_apply_receipts(
        &self,
        request: &SkillPolicyApplyReceiptsRequest,
    ) -> anyhow::Result<SkillPolicyApplyReceiptsResponse> {
        let limit = request.limit.unwrap_or(128).clamp(1, 1024) as i64;
        let conn = self.connect()?;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT payload_json
                FROM skill_policy_apply_receipts
                WHERE (?1 IS NULL OR project = ?1)
                  AND (?2 IS NULL OR namespace = ?2)
                  AND (?3 IS NULL OR workspace = ?3)
                ORDER BY created_at DESC
                LIMIT ?4
                "#,
            )
            .context("prepare skill policy apply receipts query")?;
        let rows = stmt
            .query_map(
                params![
                    request.project.clone(),
                    request.namespace.clone(),
                    request.workspace.clone(),
                    limit,
                ],
                |row| row.get::<_, String>(0),
            )
            .context("query skill policy apply receipts")?;
        let mut receipts = Vec::new();
        for row in rows {
            let payload = row.context("read skill policy apply receipt row")?;
            let payload = serde_json::from_str::<SkillPolicyApplyRecordPayload>(&payload)
                .context("deserialize skill policy apply receipt payload")?;
            receipts.push(payload.receipt);
        }
        Ok(SkillPolicyApplyReceiptsResponse { receipts })
    }

    pub fn skill_policy_activations(
        &self,
        request: &SkillPolicyActivationEntriesRequest,
    ) -> anyhow::Result<SkillPolicyActivationEntriesResponse> {
        let limit = request.limit.unwrap_or(128).clamp(1, 1024) as i64;
        let conn = self.connect()?;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT payload_json
                FROM skill_policy_activations
                WHERE (?1 IS NULL OR project = ?1)
                  AND (?2 IS NULL OR namespace = ?2)
                  AND (?3 IS NULL OR workspace = ?3)
                ORDER BY created_at DESC
                LIMIT ?4
                "#,
            )
            .context("prepare skill policy activations query")?;
        let rows = stmt
            .query_map(
                params![
                    request.project.clone(),
                    request.namespace.clone(),
                    request.workspace.clone(),
                    limit,
                ],
                |row| row.get::<_, String>(0),
            )
            .context("query skill policy activations")?;
        let mut activations = Vec::new();
        for row in rows {
            let payload = row.context("read skill policy activation row")?;
            activations.push(
                serde_json::from_str::<SkillPolicyActivationEntry>(&payload)
                    .context("deserialize skill policy activation payload")?,
            );
        }
        Ok(SkillPolicyActivationEntriesResponse { activations })
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

    fn prune_expired_hive_claims(&self) -> anyhow::Result<()> {
        let conn = self.connect()?;
        conn.execute(
            "DELETE FROM hive_claims WHERE expires_at <= ?1",
            params![chrono::Utc::now().to_rfc3339()],
        )
        .context("prune expired hive claims")?;
        Ok(())
    }

    fn prune_stale_hive_sessions(&self) -> anyhow::Result<()> {
        let conn = self.connect()?;
        conn.execute(
            "DELETE FROM hive_sessions WHERE last_seen < ?1",
            params![(chrono::Utc::now() - chrono::TimeDelta::hours(24)).to_rfc3339()],
        )
        .context("prune stale hive sessions")?;
        Ok(())
    }

    fn find_by_any_key(
        &self,
        redundancy_key: &str,
        canonical_key: &str,
        stage: &memd_schema::MemoryStage,
    ) -> anyhow::Result<Option<DuplicateMatch>> {
        let stage = serde_json::to_string(stage).context("serialize lookup stage")?;
        let conn = self.connect()?;
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
        let conn = self.connect()?;
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
        let conn = self.connect()?;
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
        let conn = self.connect()?;
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

fn hive_follow_overlap_risk(
    current: &HiveSessionRecord,
    target: &HiveSessionRecord,
) -> Option<String> {
    let current_task = normalized_hive_text(current.task_id.as_deref());
    let target_task = normalized_hive_text(target.task_id.as_deref());
    let current_topic = normalized_hive_text(current.topic_claim.as_deref());
    let target_topic = normalized_hive_text(target.topic_claim.as_deref());
    let current_scopes = hive_overlap_scopes(&current.scope_claims);
    let target_scopes = hive_overlap_scopes(&target.scope_claims);
    let shared_scopes = current_scopes
        .iter()
        .filter(|scope| target_scopes.iter().any(|other| other == *scope))
        .cloned()
        .collect::<Vec<_>>();

    if let (Some(current_task), Some(target_task)) =
        (current_task.as_deref(), target_task.as_deref())
    {
        if current_task != target_task && !shared_scopes.is_empty() {
            return Some(format!(
                "confirmed hive overlap: target session {} already owns scope(s) for task {}",
                target.session, target_task
            ));
        }
    }

    if !shared_scopes.is_empty() {
        return Some(format!(
            "possible_work_overlap touches={}",
            shared_scopes.join(",")
        ));
    }

    if let (Some(current_task), Some(target_task)) =
        (current_task.as_deref(), target_task.as_deref())
    {
        if current_task == target_task {
            return Some(format!(
                "confirmed hive overlap: target session {} already owns task {}",
                target.session, target_task
            ));
        }
    }

    if let (Some(current_topic), Some(target_topic)) =
        (current_topic.as_deref(), target_topic.as_deref())
    {
        if current_topic == target_topic {
            return Some(format!(
                "confirmed hive overlap: target session {} already owns topic {}",
                target.session,
                target.topic_claim.as_deref().unwrap_or("none")
            ));
        }
    }

    None
}

fn hive_overlap_scopes(scopes: &[String]) -> Vec<String> {
    scopes
        .iter()
        .filter_map(|scope| normalized_hive_text(Some(scope.as_str())))
        .filter(|scope| !is_generic_hive_overlap_scope(scope))
        .collect()
}

fn normalized_hive_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn is_generic_hive_overlap_scope(value: &str) -> bool {
    matches!(
        value,
        "project" | "workspace" | "shared" | "none" | "unknown"
    )
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

struct HiveSessionKeyArgs<'a> {
    project: Option<&'a str>,
    namespace: Option<&'a str>,
    workspace: Option<&'a str>,
    repo_root: Option<&'a str>,
    worktree_root: Option<&'a str>,
    branch: Option<&'a str>,
    agent: Option<&'a str>,
    effective_agent: Option<&'a str>,
    hive_system: Option<&'a str>,
    hive_role: Option<&'a str>,
    host: Option<&'a str>,
}

fn hive_session_key(session: &str, args: HiveSessionKeyArgs<'_>) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        session.trim(),
        args.project.unwrap_or("").trim(),
        args.namespace.unwrap_or("").trim(),
        args.workspace.unwrap_or("").trim(),
        args.repo_root.unwrap_or("").trim(),
        args.worktree_root.unwrap_or("").trim(),
        args.branch.unwrap_or("").trim(),
        args.agent.unwrap_or("").trim(),
        args.effective_agent.unwrap_or("").trim(),
        args.hive_system.unwrap_or("").trim(),
        args.hive_role.unwrap_or("").trim(),
        args.host.unwrap_or("").trim()
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
            workspace: item.workspace.clone(),
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
        workspace: item
            .workspace
            .clone()
            .or(previous.and_then(|context| context.workspace)),
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

type SourceKey = (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    memd_schema::MemoryVisibility,
);

type WorkspaceKey = (
    Option<String>,
    Option<String>,
    Option<String>,
    memd_schema::MemoryVisibility,
);

#[derive(Default)]
struct SourceAggregate {
    item_count: usize,
    active_count: usize,
    candidate_count: usize,
    derived_count: usize,
    synthetic_count: usize,
    contested_count: usize,
    confidence_sum: f32,
    last_seen_at: Option<chrono::DateTime<chrono::Utc>>,
    tag_counts: std::collections::BTreeMap<String, usize>,
}

impl SourceAggregate {
    fn observe(&mut self, item: &MemoryItem) {
        self.item_count = self.item_count.saturating_add(1);
        if item.stage == memd_schema::MemoryStage::Canonical {
            self.active_count = self.active_count.saturating_add(1);
        } else {
            self.candidate_count = self.candidate_count.saturating_add(1);
        }
        if item.source_quality == Some(SourceQuality::Derived) {
            self.derived_count = self.derived_count.saturating_add(1);
        }
        if item.source_quality == Some(SourceQuality::Synthetic) {
            self.synthetic_count = self.synthetic_count.saturating_add(1);
        }
        if item.status == memd_schema::MemoryStatus::Contested {
            self.contested_count = self.contested_count.saturating_add(1);
        }
        self.confidence_sum += item.confidence.clamp(0.0, 1.0);
        self.last_seen_at = match self.last_seen_at {
            Some(current) if current >= item.updated_at => Some(current),
            _ => Some(item.updated_at),
        };
        for tag in &item.tags {
            *self.tag_counts.entry(tag.clone()).or_insert(0) += 1;
        }
    }

    fn avg_confidence(&self) -> f32 {
        if self.item_count == 0 {
            0.0
        } else {
            (self.confidence_sum / self.item_count as f32).clamp(0.0, 1.0)
        }
    }

    fn tags(&self, limit: usize) -> Vec<String> {
        let mut tags = self
            .tag_counts
            .iter()
            .map(|(tag, count)| (tag.clone(), *count))
            .collect::<Vec<_>>();
        tags.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        tags.into_iter().take(limit).map(|(tag, _)| tag).collect()
    }
}

#[derive(Default)]
struct WorkspaceAggregate {
    source: SourceAggregate,
    source_lanes: std::collections::BTreeSet<(Option<String>, Option<String>)>,
}

impl WorkspaceAggregate {
    fn observe(&mut self, item: &MemoryItem) {
        self.source.observe(item);
        self.source_lanes
            .insert((item.source_agent.clone(), item.source_system.clone()));
    }
}

fn source_trust_score(
    item_count: usize,
    active_count: usize,
    candidate_count: usize,
    derived_count: usize,
    synthetic_count: usize,
    contested_count: usize,
    avg_confidence: f32,
) -> f32 {
    if item_count == 0 {
        return 0.0;
    }

    let active_ratio = active_count as f32 / item_count as f32;
    let derived_ratio = derived_count as f32 / item_count as f32;
    let candidate_ratio = candidate_count as f32 / item_count as f32;
    let synthetic_ratio = synthetic_count as f32 / item_count as f32;
    let contested_ratio = contested_count as f32 / item_count as f32;

    let score = avg_confidence * 0.58 + active_ratio * 0.18 + derived_ratio * 0.12
        - candidate_ratio * 0.05
        - synthetic_ratio * 0.18
        - contested_ratio * 0.14;
    score.clamp(0.0, 1.0)
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

fn migrate_hive_sessions_identity_columns(conn: &mut Connection) -> anyhow::Result<()> {
    let columns = {
        let mut stmt = conn.prepare("PRAGMA table_info(hive_sessions)")?;
        stmt.query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>, _>>()?
    };

    let has_hive_system = columns.iter().any(|value| value == "hive_system");
    let has_hive_role = columns.iter().any(|value| value == "hive_role");
    let has_host = columns.iter().any(|value| value == "host");
    let has_repo_root = columns.iter().any(|value| value == "repo_root");
    let has_worktree_root = columns.iter().any(|value| value == "worktree_root");
    let has_branch = columns.iter().any(|value| value == "branch");

    if !has_hive_system {
        conn.execute_batch("ALTER TABLE hive_sessions ADD COLUMN hive_system TEXT;")?;
    }
    if !has_hive_role {
        conn.execute_batch("ALTER TABLE hive_sessions ADD COLUMN hive_role TEXT;")?;
    }
    if !has_host {
        conn.execute_batch("ALTER TABLE hive_sessions ADD COLUMN host TEXT;")?;
    }
    if !has_repo_root {
        conn.execute_batch("ALTER TABLE hive_sessions ADD COLUMN repo_root TEXT;")?;
    }
    if !has_worktree_root {
        conn.execute_batch("ALTER TABLE hive_sessions ADD COLUMN worktree_root TEXT;")?;
    }
    if !has_branch {
        conn.execute_batch("ALTER TABLE hive_sessions ADD COLUMN branch TEXT;")?;
    }

    let has_hive_session_groups = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'hive_session_groups'",
            [],
            |_| Ok(()),
        )
        .optional()?
        .is_some();
    if !has_hive_session_groups {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS hive_session_groups (
              session_key TEXT NOT NULL,
              hive_group TEXT NOT NULL,
              PRIMARY KEY (session_key, hive_group),
              FOREIGN KEY (session_key) REFERENCES hive_sessions(session_key) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_hive_session_groups_session
              ON hive_session_groups(session_key);
            CREATE INDEX IF NOT EXISTS idx_hive_session_groups_hive_group
              ON hive_session_groups(hive_group, session_key);
            "#,
        )?;
    }

    let hive_session_groups_empty = conn
        .query_row("SELECT 1 FROM hive_session_groups LIMIT 1", [], |_| Ok(()))
        .optional()?
        .is_none();

    if !has_hive_system
        || !has_hive_role
        || !has_host
        || !has_repo_root
        || !has_worktree_root
        || !has_branch
        || hive_session_groups_empty
    {
        let tx = conn
            .transaction()
            .context("begin hive session migration backfill")?;

        {
            let mut statement =
                tx.prepare("SELECT session_key, payload_json FROM hive_sessions")?;
            let mut rows = statement.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;

            let mut update = if !has_hive_system
                || !has_hive_role
                || !has_host
                || !has_repo_root
                || !has_worktree_root
                || !has_branch
            {
                Some(tx.prepare(
                    "UPDATE hive_sessions SET hive_system = ?1, hive_role = ?2, host = ?3, repo_root = ?4, worktree_root = ?5, branch = ?6 WHERE session_key = ?7",
                )?)
            } else {
                None
            };
            let mut insert_group = if hive_session_groups_empty {
                Some(tx.prepare(
                    "INSERT OR IGNORE INTO hive_session_groups (session_key, hive_group) VALUES (?1, ?2)",
                )?)
            } else {
                None
            };

            if hive_session_groups_empty {
                tx.execute("DELETE FROM hive_session_groups", [])?;
            }

            for row in rows.by_ref() {
                let (session_key, payload) = row?;
                let record: HiveSessionRecord =
                    serde_json::from_str(&payload).context("deserialize hive session record")?;
                if let Some(update) = update.as_mut() {
                    update.execute(params![
                        record.hive_system,
                        record.hive_role,
                        record.host,
                        record.repo_root,
                        record.worktree_root,
                        record.branch,
                        session_key
                    ])?;
                }
                if let Some(insert_group) = insert_group.as_mut() {
                    for hive_group in record.hive_groups.iter() {
                        insert_group.execute(params![session_key, hive_group])?;
                    }
                }
            }
        }
        tx.commit()
            .context("commit hive session migration backfill")?;
    }

    Ok(())
}

fn create_hive_session_identity_indexes(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch(
        r#"
        CREATE INDEX IF NOT EXISTS idx_hive_sessions_hive_system
          ON hive_sessions(hive_system);
        CREATE INDEX IF NOT EXISTS idx_hive_sessions_hive_role
          ON hive_sessions(hive_role);
        CREATE INDEX IF NOT EXISTS idx_hive_sessions_host
          ON hive_sessions(host);
        CREATE INDEX IF NOT EXISTS idx_hive_sessions_repo_root
          ON hive_sessions(repo_root);
        CREATE INDEX IF NOT EXISTS idx_hive_sessions_worktree_root_identity
          ON hive_sessions(worktree_root);
        CREATE INDEX IF NOT EXISTS idx_hive_sessions_branch_identity
          ON hive_sessions(branch);
        "#,
    )
    .context("create hive session identity indexes")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonical_key;
    use memd_schema::{MemoryKind, MemoryScope, MemoryStage, MemoryStatus, MemoryVisibility};

    fn open_temp_store(prefix: &str) -> (std::path::PathBuf, SqliteStore) {
        let dir = std::env::temp_dir().join(format!("{prefix}-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");
        (dir, store)
    }

    fn sample_memory_item() -> MemoryItem {
        let now = chrono::Utc::now();
        MemoryItem {
            id: Uuid::new_v4(),
            content: "hive resume state".to_string(),
            redundancy_key: None,
            belief_branch: None,
            preferred: false,
            kind: MemoryKind::Status,
            scope: MemoryScope::Project,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("core".to_string()),
            visibility: MemoryVisibility::Workspace,
            source_agent: Some("codex@test".to_string()),
            source_system: Some("memd-test".to_string()),
            source_path: None,
            confidence: 0.9,
            ttl_seconds: None,
            created_at: now,
            updated_at: now,
            last_verified_at: None,
            supersedes: Vec::new(),
            tags: vec!["resume_state".to_string()],
            status: MemoryStatus::Active,
            source_quality: Some(SourceQuality::Canonical),
            stage: MemoryStage::Canonical,
        }
    }

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
                workspace: Some("core".to_string()),
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

    #[test]
    fn insert_or_get_duplicate_returns_existing_item_without_deadlock() {
        let (dir, store) = open_temp_store("memd-duplicate-path");
        let item = sample_memory_item();
        let canonical_key = canonical_key(&item);
        let redundancy_key = redundancy_key(&item);

        assert!(
            store
                .insert_or_get_duplicate(&item, &canonical_key, &redundancy_key)
                .expect("insert first item")
                .is_none()
        );

        let duplicate = store
            .insert_or_get_duplicate(&item, &canonical_key, &redundancy_key)
            .expect("resolve duplicate");

        assert!(duplicate.is_some());
        assert_eq!(duplicate.as_ref().map(|found| found.id), Some(item.id));
        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn rehearse_entity_by_id_updates_entity_without_deadlock() {
        let (dir, store) = open_temp_store("memd-rehearse-entity");
        let item = sample_memory_item();
        let canonical_key = canonical_key(&item);
        let entity = store
            .resolve_entity_for_item(&item, &canonical_key)
            .expect("resolve entity");

        let rehearsed = store
            .rehearse_entity_by_id(entity.record.id, 0.15)
            .expect("rehearse entity")
            .expect("entity should exist");

        assert_eq!(rehearsed.id, entity.record.id);
        assert_eq!(
            rehearsed.rehearsal_count,
            entity.record.rehearsal_count.saturating_add(1)
        );
        assert!(rehearsed.salience_score >= entity.record.salience_score);
        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn concurrent_write_and_cross_workspace_reads_complete() {
        let (dir, store) = open_temp_store("memd-cross-workspace-concurrency");

        let mut seed = sample_memory_item();
        seed.project = Some("demo".to_string());
        seed.namespace = Some("main".to_string());
        seed.workspace = Some("shared".to_string());
        seed.visibility = MemoryVisibility::Workspace;
        seed.content = "seed item".to_string();
        seed.source_agent = Some("codex@test-a@session-a".to_string());
        seed.source_system = Some("memd".to_string());
        seed.tags = vec!["seed".to_string()];
        let seed_canonical_key = canonical_key(&seed);
        let seed_redundancy_key = redundancy_key(&seed);
        store
            .insert_or_get_duplicate(&seed, &seed_canonical_key, &seed_redundancy_key)
            .expect("insert seed item");
        let entity = store
            .resolve_entity_for_item(&seed, &seed_canonical_key)
            .expect("resolve seed entity");
        store
            .record_event(
                &entity.record,
                seed.id,
                RecordEventArgs {
                    event_type: "stored".to_string(),
                    summary: "seed stored".to_string(),
                    occurred_at: seed.updated_at,
                    project: seed.project.clone(),
                    namespace: seed.namespace.clone(),
                    workspace: seed.workspace.clone(),
                    source_agent: seed.source_agent.clone(),
                    source_system: seed.source_system.clone(),
                    source_path: seed.source_path.clone(),
                    related_entity_ids: Vec::new(),
                    tags: seed.tags.clone(),
                    context: None,
                    confidence: seed.confidence,
                    salience_score: entity.record.salience_score,
                },
            )
            .expect("record seed event");

        let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
        let (done_tx, done_rx) = std::sync::mpsc::channel::<&'static str>();

        let writer_store = store.clone();
        let writer_barrier = barrier.clone();
        let writer_tx = done_tx.clone();
        let writer = std::thread::spawn(move || {
            writer_barrier.wait();
            let mut item = sample_memory_item();
            item.project = Some("demo".to_string());
            item.namespace = Some("main".to_string());
            item.workspace = Some("shared".to_string());
            item.visibility = MemoryVisibility::Workspace;
            item.content = "concurrent item".to_string();
            item.source_agent = Some("codex@test-a@session-a".to_string());
            item.source_system = Some("memd".to_string());
            item.tags = vec!["repro".to_string()];
            let canonical_key = canonical_key(&item);
            let redundancy_key = redundancy_key(&item);
            writer_store
                .insert_or_get_duplicate(&item, &canonical_key, &redundancy_key)
                .expect("insert concurrent item");
            let entity = writer_store
                .resolve_entity_for_item(&item, &canonical_key)
                .expect("resolve concurrent entity");
            writer_store
                .record_event(
                    &entity.record,
                    item.id,
                    RecordEventArgs {
                        event_type: "stored".to_string(),
                        summary: "concurrent item stored".to_string(),
                        occurred_at: item.updated_at,
                        project: item.project.clone(),
                        namespace: item.namespace.clone(),
                        workspace: item.workspace.clone(),
                        source_agent: item.source_agent.clone(),
                        source_system: item.source_system.clone(),
                        source_path: item.source_path.clone(),
                        related_entity_ids: Vec::new(),
                        tags: item.tags.clone(),
                        context: None,
                        confidence: item.confidence,
                        salience_score: entity.record.salience_score,
                    },
                )
                .expect("record concurrent event");
            writer_tx.send("writer").expect("send writer completion");
        });

        let reader_store = store.clone();
        let reader_barrier = barrier.clone();
        let reader_tx = done_tx.clone();
        let reader = std::thread::spawn(move || {
            reader_barrier.wait();
            let workspaces = reader_store
                .workspace_memory(&WorkspaceMemoryRequest {
                    project: Some("demo".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("other".to_string()),
                    visibility: Some(MemoryVisibility::Workspace),
                    source_agent: None,
                    source_system: None,
                    limit: Some(6),
                })
                .expect("cross-workspace lanes");
            let sources = reader_store
                .source_memory(&SourceMemoryRequest {
                    project: Some("demo".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("other".to_string()),
                    visibility: Some(MemoryVisibility::Workspace),
                    source_agent: None,
                    source_system: None,
                    limit: Some(6),
                })
                .expect("cross-workspace sources");
            assert!(workspaces.workspaces.is_empty());
            assert!(sources.sources.is_empty());
            reader_tx.send("reader").expect("send reader completion");
        });

        barrier.wait();
        let first = done_rx
            .recv_timeout(std::time::Duration::from_secs(2))
            .expect("first concurrent operation should finish");
        let second = done_rx
            .recv_timeout(std::time::Duration::from_secs(2))
            .expect("second concurrent operation should finish");
        assert_ne!(first, second);

        writer.join().expect("join writer");
        reader.join().expect("join reader");
        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn hive_sessions_keep_same_named_sessions_separate_across_agents() {
        let dir = std::env::temp_dir().join(format!("memd-hive-sessions-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "shared-session".to_string(),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@shared-session".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("codex".to_string()),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string(), "coordination".to_string()],
                hive_groups: vec!["openclaw-stack".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some("llama-desktop/qwen".to_string()),
                tab_id: None,
                project: Some("repo-a".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("initiative-alpha".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: Some("laptop-a".to_string()),
                pid: Some(101),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: Some("work a".to_string()),
                pressure: None,
                next_recovery: None,
                next_action: None,
                needs_help: false,
                needs_review: false,
                handoff_state: None,
                confidence: None,
                risk: None,
                status: Some("live".to_string()),
            })
            .expect("insert codex session");
        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "shared-session".to_string(),
                agent: Some("claude-code".to_string()),
                effective_agent: Some("claude-code@shared-session".to_string()),
                hive_system: Some("claude-code".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("claude-code".to_string()),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string(), "coordination".to_string()],
                hive_groups: vec!["openclaw-stack".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some("claude-sonnet-4".to_string()),
                tab_id: None,
                project: Some("repo-b".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("initiative-alpha".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:9797".to_string()),
                base_url_healthy: Some(true),
                host: Some("laptop-b".to_string()),
                pid: Some(202),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: Some("work b".to_string()),
                pressure: None,
                next_recovery: None,
                next_action: None,
                needs_help: false,
                needs_review: false,
                handoff_state: None,
                confidence: None,
                risk: None,
                status: Some("live".to_string()),
            })
            .expect("insert claude session");

        let sessions = store
            .hive_sessions(&HiveSessionsRequest {
                session: Some("shared-session".to_string()),
                project: None,
                namespace: None,
                repo_root: None,
                worktree_root: None,
                branch: None,
                workspace: Some("initiative-alpha".to_string()),
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(false),
                limit: Some(16),
            })
            .expect("query sessions");
        assert_eq!(sessions.sessions.len(), 2);
        assert_eq!(
            sessions
                .sessions
                .iter()
                .filter(|session| session.agent.as_deref() == Some("codex"))
                .count(),
            1
        );
        assert_eq!(
            sessions
                .sessions
                .iter()
                .filter(|session| session.agent.as_deref() == Some("claude-code"))
                .count(),
            1
        );

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn hive_sessions_keep_same_named_sessions_separate_across_branches() {
        let dir = std::env::temp_dir().join(format!("memd-hive-branches-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "shared-session".to_string(),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@shared-session".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("codex".to_string()),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["project:demo".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some("llama-desktop/qwen".to_string()),
                tab_id: Some("tab-a".to_string()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                repo_root: Some("/tmp/repo".to_string()),
                worktree_root: Some("/tmp/repo-a".to_string()),
                branch: Some("feature/a".to_string()),
                base_branch: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: Some("workstation".to_string()),
                pid: Some(111),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: None,
                pressure: None,
                next_recovery: None,
                next_action: None,
                needs_help: false,
                needs_review: false,
                handoff_state: None,
                confidence: None,
                risk: None,
                status: Some("live".to_string()),
            })
            .expect("insert branch a session");
        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "shared-session".to_string(),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@shared-session".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("codex".to_string()),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["project:demo".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some("llama-desktop/qwen".to_string()),
                tab_id: Some("tab-b".to_string()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                repo_root: Some("/tmp/repo".to_string()),
                worktree_root: Some("/tmp/repo-b".to_string()),
                branch: Some("feature/b".to_string()),
                base_branch: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: Some("workstation".to_string()),
                pid: Some(222),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: None,
                pressure: None,
                next_recovery: None,
                next_action: None,
                needs_help: false,
                needs_review: false,
                handoff_state: None,
                confidence: None,
                risk: None,
                status: Some("live".to_string()),
            })
            .expect("insert branch b session");

        let sessions = store
            .hive_sessions(&HiveSessionsRequest {
                session: Some("shared-session".to_string()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                repo_root: Some("/tmp/repo".to_string()),
                worktree_root: None,
                branch: None,
                workspace: Some("shared".to_string()),
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(false),
                limit: Some(16),
            })
            .expect("query sessions");

        assert_eq!(sessions.sessions.len(), 2);
        assert!(
            sessions
                .sessions
                .iter()
                .any(|session| session.branch.as_deref() == Some("feature/a"))
        );
        assert!(
            sessions
                .sessions
                .iter()
                .any(|session| session.branch.as_deref() == Some("feature/b"))
        );

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn hive_sessions_preserve_service_hive_metadata() {
        let dir = std::env::temp_dir().join(format!("memd-hive-service-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "shell-a".to_string(),
                agent: Some("agent-shell".to_string()),
                effective_agent: Some("agent-shell@shell-a".to_string()),
                hive_system: Some("agent-shell".to_string()),
                hive_role: Some("runtime-shell".to_string()),
                worker_name: Some("agent-shell".to_string()),
                display_name: None,
                role: Some("runtime-shell".to_string()),
                capabilities: vec![
                    "shell".to_string(),
                    "exec".to_string(),
                    "workspace".to_string(),
                ],
                hive_groups: vec!["runtime-core".to_string(), "dependency-owners".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("worker".to_string()),
                heartbeat_model: Some("llama-desktop/qwen".to_string()),
                tab_id: None,
                project: Some("openclaw".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("stack-alpha".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: Some("vm-a".to_string()),
                pid: Some(333),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: Some("repair runtime dependency".to_string()),
                pressure: None,
                next_recovery: None,
                next_action: None,
                needs_help: false,
                needs_review: false,
                handoff_state: None,
                confidence: None,
                risk: None,
                status: Some("live".to_string()),
            })
            .expect("insert service hive");

        let sessions = store
            .hive_sessions(&HiveSessionsRequest {
                session: Some("shell-a".to_string()),
                project: Some("openclaw".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                workspace: Some("stack-alpha".to_string()),
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(false),
                limit: Some(8),
            })
            .expect("query service hive");

        assert_eq!(sessions.sessions.len(), 1);
        let hive = &sessions.sessions[0];
        assert_eq!(hive.hive_system.as_deref(), Some("agent-shell"));
        assert_eq!(hive.hive_role.as_deref(), Some("runtime-shell"));
        assert_eq!(hive.authority.as_deref(), Some("worker"));
        assert!(hive.capabilities.iter().any(|value| value == "shell"));
        assert!(hive.capabilities.iter().any(|value| value == "exec"));

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn retire_hive_session_removes_only_matching_identity() {
        let dir = std::env::temp_dir().join(format!("memd-hive-retire-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "shared-session".to_string(),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@shared-session".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("codex".to_string()),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["project:demo".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some("llama-desktop/qwen".to_string()),
                tab_id: Some("tab-a".to_string()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: Some("workstation".to_string()),
                pid: Some(111),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: None,
                pressure: None,
                next_recovery: None,
                next_action: None,
                needs_help: false,
                needs_review: false,
                handoff_state: None,
                confidence: None,
                risk: None,
                status: Some("live".to_string()),
            })
            .expect("insert codex session");
        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "shared-session".to_string(),
                agent: Some("claude-code".to_string()),
                effective_agent: Some("claude-code@shared-session".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("claude-code".to_string()),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["project:demo".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some("llama-desktop/qwen".to_string()),
                tab_id: Some("tab-b".to_string()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: Some("workstation".to_string()),
                pid: Some(222),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: None,
                pressure: None,
                next_recovery: None,
                next_action: None,
                needs_help: false,
                needs_review: false,
                handoff_state: None,
                confidence: None,
                risk: None,
                status: Some("live".to_string()),
            })
            .expect("insert claude session");

        let retired = store
            .retire_hive_session(&HiveSessionRetireRequest {
                session: "shared-session".to_string(),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                workspace: Some("shared".to_string()),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@shared-session".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                host: Some("workstation".to_string()),
                reason: Some("superseded".to_string()),
            })
            .expect("retire codex session");
        assert_eq!(retired.retired, 1);
        assert_eq!(retired.sessions[0].agent.as_deref(), Some("codex"));

        let sessions = store
            .hive_sessions(&HiveSessionsRequest {
                session: Some("shared-session".to_string()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                workspace: Some("shared".to_string()),
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(false),
                limit: Some(8),
            })
            .expect("query remaining sessions");
        assert_eq!(sessions.sessions.len(), 1);
        assert_eq!(sessions.sessions[0].agent.as_deref(), Some("claude-code"));

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn hive_coordination_auto_retires_stale_session_without_owned_work() {
        let dir =
            std::env::temp_dir().join(format!("memd-hive-auto-retire-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "session-old".to_string(),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@session-old".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("Lorentz".to_string()),
                display_name: None,
                role: Some("worker".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["project:memd".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: None,
                tab_id: None,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: Some("workstation".to_string()),
                pid: Some(111),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: None,
                pressure: None,
                next_recovery: None,
                next_action: None,
                needs_help: false,
                needs_review: false,
                handoff_state: None,
                confidence: None,
                risk: None,
                status: Some("active".to_string()),
            })
            .expect("insert stale session");

        let mut session = store
            .hive_sessions(&HiveSessionsRequest {
                session: Some("session-old".to_string()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                workspace: Some("shared".to_string()),
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(false),
                limit: Some(8),
            })
            .expect("load session")
            .sessions
            .into_iter()
            .next()
            .expect("session exists");
        session.last_seen = chrono::Utc::now() - chrono::TimeDelta::minutes(45);
        let conn = store.connect().expect("connect sqlite");
        conn.execute(
            "UPDATE hive_sessions SET last_seen = ?1, payload_json = ?2 WHERE session = ?3",
            params![
                session.last_seen.to_rfc3339(),
                serde_json::to_string(&session).expect("serialize stale session"),
                session.session.as_str(),
            ],
        )
        .expect("mark session stale");

        let sessions = store
            .hive_sessions(&HiveSessionsRequest {
                session: None,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                workspace: Some("shared".to_string()),
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(false),
                limit: Some(8),
            })
            .expect("list sessions");
        assert!(
            sessions
                .sessions
                .iter()
                .any(|session| session.session == "session-old")
        );

        let retired = store
            .auto_retire_stale_hive_sessions(
                Some("memd"),
                Some("main"),
                Some("shared"),
                chrono::Utc::now(),
            )
            .expect("auto retire");
        assert_eq!(retired.retired, vec!["session-old".to_string()]);

        let remaining = store
            .hive_sessions(&HiveSessionsRequest {
                session: None,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                workspace: Some("shared".to_string()),
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(false),
                limit: Some(8),
            })
            .expect("list sessions after retire");
        assert!(
            remaining
                .sessions
                .iter()
                .all(|session| session.session != "session-old")
        );

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn hive_sessions_filter_by_hive_identity() {
        let dir = std::env::temp_dir().join(format!("memd-hive-filter-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "shared-session".to_string(),
                agent: Some("agent-a".to_string()),
                effective_agent: Some("agent-a@shared-session".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("runtime-shell".to_string()),
                worker_name: Some("agent-a".to_string()),
                display_name: None,
                role: Some("runtime-shell".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["runtime-core".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("worker".to_string()),
                heartbeat_model: Some("llama-desktop/qwen".to_string()),
                tab_id: None,
                project: Some("repo-a".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: Some("vm-a".to_string()),
                pid: Some(111),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: None,
                pressure: None,
                next_recovery: None,
                next_action: None,
                needs_help: false,
                needs_review: false,
                handoff_state: None,
                confidence: None,
                risk: None,
                status: Some("live".to_string()),
            })
            .expect("insert codex runtime shell session");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "shared-session".to_string(),
                agent: Some("agent-b".to_string()),
                effective_agent: Some("agent-b@shared-session".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("orchestrator".to_string()),
                worker_name: Some("agent-b".to_string()),
                display_name: None,
                role: Some("orchestrator".to_string()),
                capabilities: vec!["coordination".to_string()],
                hive_groups: vec!["openclaw-stack".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("coordinator".to_string()),
                heartbeat_model: Some("llama-desktop/qwen".to_string()),
                tab_id: None,
                project: Some("repo-a".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:9797".to_string()),
                base_url_healthy: Some(true),
                host: Some("vm-b".to_string()),
                pid: Some(222),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: None,
                pressure: None,
                next_recovery: None,
                next_action: None,
                needs_help: false,
                needs_review: false,
                handoff_state: None,
                confidence: None,
                risk: None,
                status: Some("live".to_string()),
            })
            .expect("insert codex orchestrator session");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "shared-session".to_string(),
                agent: Some("agent-c".to_string()),
                effective_agent: Some("agent-c@shared-session".to_string()),
                hive_system: Some("claude-code".to_string()),
                hive_role: Some("runtime-shell".to_string()),
                worker_name: Some("agent-c".to_string()),
                display_name: None,
                role: Some("runtime-shell".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["runtime-core".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("worker".to_string()),
                heartbeat_model: Some("claude-opus".to_string()),
                tab_id: None,
                project: Some("repo-a".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:9898".to_string()),
                base_url_healthy: Some(true),
                host: Some("vm-a".to_string()),
                pid: Some(333),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: None,
                pressure: None,
                next_recovery: None,
                next_action: None,
                needs_help: false,
                needs_review: false,
                handoff_state: None,
                confidence: None,
                risk: None,
                status: Some("live".to_string()),
            })
            .expect("insert claude runtime shell session");

        let codex_sessions = store
            .hive_sessions(&HiveSessionsRequest {
                session: Some("shared-session".to_string()),
                project: Some("repo-a".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                workspace: Some("shared".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(false),
                limit: Some(16),
            })
            .expect("query sessions by hive system");
        assert_eq!(codex_sessions.sessions.len(), 2);

        let runtime_session = store
            .hive_sessions(&HiveSessionsRequest {
                session: Some("shared-session".to_string()),
                project: Some("repo-a".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                workspace: Some("shared".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("runtime-shell".to_string()),
                host: Some("vm-a".to_string()),
                hive_group: None,
                active_only: Some(false),
                limit: Some(16),
            })
            .expect("query sessions by hive role and host");
        assert_eq!(runtime_session.sessions.len(), 1);
        assert_eq!(
            runtime_session.sessions[0].hive_system.as_deref(),
            Some("codex")
        );
        assert_eq!(
            runtime_session.sessions[0].hive_role.as_deref(),
            Some("runtime-shell")
        );
        assert_eq!(runtime_session.sessions[0].host.as_deref(), Some("vm-a"));

        let host_a_sessions = store
            .hive_sessions(&HiveSessionsRequest {
                session: Some("shared-session".to_string()),
                project: Some("repo-a".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                workspace: Some("shared".to_string()),
                hive_system: None,
                hive_role: None,
                host: Some("vm-a".to_string()),
                hive_group: None,
                active_only: Some(false),
                limit: Some(16),
            })
            .expect("query sessions by host");
        assert_eq!(host_a_sessions.sessions.len(), 2);
        assert!(
            host_a_sessions
                .sessions
                .iter()
                .all(|session| session.host.as_deref() == Some("vm-a"))
        );

        let runtime_group_sessions = store
            .hive_sessions(&HiveSessionsRequest {
                session: Some("shared-session".to_string()),
                project: Some("repo-a".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                workspace: Some("shared".to_string()),
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: Some("runtime-core".to_string()),
                active_only: Some(false),
                limit: Some(16),
            })
            .expect("query sessions by hive group");
        assert_eq!(runtime_group_sessions.sessions.len(), 2);
        assert!(runtime_group_sessions.sessions.iter().all(|session| {
            session
                .hive_groups
                .iter()
                .any(|value| value == "runtime-core")
        }));

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn open_migrates_legacy_hive_sessions_before_identity_indexes() {
        let dir = std::env::temp_dir().join(format!("legacy-hive-sessions-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("state.sqlite");
        let conn = Connection::open(&path).expect("open sqlite database");

        conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;
            CREATE TABLE hive_sessions (
              session_key TEXT PRIMARY KEY,
              session TEXT NOT NULL,
              project TEXT,
              namespace TEXT,
              workspace TEXT,
              status TEXT NOT NULL,
              last_seen TEXT NOT NULL,
              payload_json TEXT NOT NULL
            );
            "#,
        )
        .expect("create legacy hive_sessions");

        drop(conn);

        let store = SqliteStore::open(&path).expect("open migrated sqlite store");
        let conn = store.connect().expect("connect migrated sqlite store");
        let columns = {
            let mut stmt = conn
                .prepare("PRAGMA table_info(hive_sessions)")
                .expect("prepare table info");
            stmt.query_map([], |row| row.get::<_, String>(1))
                .expect("query table info")
                .collect::<Result<Vec<_>, _>>()
                .expect("collect hive session columns")
        };
        assert!(columns.iter().any(|value| value == "hive_system"));
        assert!(columns.iter().any(|value| value == "hive_role"));
        assert!(columns.iter().any(|value| value == "host"));

        let indexes = {
            let mut stmt = conn
                .prepare("PRAGMA index_list(hive_sessions)")
                .expect("prepare index list");
            stmt.query_map([], |row| row.get::<_, String>(1))
                .expect("query index list")
                .collect::<Result<Vec<_>, _>>()
                .expect("collect hive session indexes")
        };
        assert!(
            indexes
                .iter()
                .any(|value| value == "idx_hive_sessions_hive_system")
        );
        assert!(
            indexes
                .iter()
                .any(|value| value == "idx_hive_sessions_hive_role")
        );
        assert!(
            indexes
                .iter()
                .any(|value| value == "idx_hive_sessions_host")
        );

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn maintain_runtime_persists_report_receipt() {
        let dir = std::env::temp_dir().join(format!("runtime-maintain-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

        let report = store
            .maintain_runtime(&MaintainReportRequest {
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                session: Some("session-a".to_string()),
                mode: "scan".to_string(),
                apply: false,
            })
            .expect("run maintain runtime");

        assert_eq!(report.mode, "scan");
        assert!(report.receipt_id.is_some());
        assert!(
            report
                .findings
                .iter()
                .any(|line| line.contains("memory maintain"))
        );

        let conn = store.connect().expect("connect sqlite store");
        let persisted: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM runtime_maintenance_reports",
                [],
                |row| row.get(0),
            )
            .expect("count persisted maintenance reports");
        assert_eq!(persisted, 1);

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }
}
