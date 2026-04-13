//! Procedural memory store operations (Phase G).
//!
//! Procedures are learned workflows, policies, and recovery patterns
//! that can be retrieved and reused across sessions.

use anyhow::Context;
use chrono::Utc;
use memd_schema::{
    Procedure, ProcedureDetectRequest, ProcedureDetectResponse, ProcedureKind,
    ProcedureListRequest, ProcedureListResponse, ProcedureMatchRequest,
    ProcedureMatchResponse, ProcedurePromoteRequest, ProcedurePromoteResponse,
    ProcedureRecordRequest, ProcedureRecordResponse, ProcedureRetireRequest,
    ProcedureRetireResponse, ProcedureStatus, ProcedureUseRequest, ProcedureUseResponse,
};
use rusqlite::params;
use uuid::Uuid;

use crate::store::SqliteStore;

impl SqliteStore {
    /// List procedures with optional filters.
    pub(crate) fn list_procedures(
        &self,
        req: &ProcedureListRequest,
    ) -> anyhow::Result<ProcedureListResponse> {
        let conn = self.connect()?;
        let mut sql =
            String::from("SELECT id, payload_json FROM procedures WHERE 1=1");
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(project) = &req.project {
            sql.push_str(" AND project = ?");
            bind_values.push(project.clone());
        }
        if let Some(namespace) = &req.namespace {
            sql.push_str(" AND namespace = ?");
            bind_values.push(namespace.clone());
        }
        if let Some(kind) = &req.kind {
            let kind_str = serde_json::to_value(kind)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_default();
            sql.push_str(" AND kind = ?");
            bind_values.push(kind_str);
        }
        if let Some(status) = &req.status {
            let status_str = serde_json::to_value(status)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_default();
            sql.push_str(" AND status = ?");
            bind_values.push(status_str);
        }
        sql.push_str(" ORDER BY updated_at DESC");
        let limit = req.limit.unwrap_or(20);
        sql.push_str(&format!(" LIMIT {limit}"));

        let mut stmt = conn.prepare(&sql)?;
        let params: Vec<&dyn rusqlite::ToSql> = bind_values
            .iter()
            .map(|v| v as &dyn rusqlite::ToSql)
            .collect();
        let procedures = stmt
            .query_map(params.as_slice(), |row| {
                let payload: String = row.get(1)?;
                Ok(payload)
            })?
            .filter_map(|r| r.inspect_err(|e| eprintln!("warn: procedure row read: {e}")).ok())
            .filter_map(|payload| serde_json::from_str::<Procedure>(&payload).inspect_err(|e| eprintln!("warn: procedure json parse: {e}")).ok())
            .collect();

        Ok(ProcedureListResponse { procedures })
    }

    /// Record a new procedure (explicit capture).
    pub(crate) fn record_procedure(
        &self,
        req: &ProcedureRecordRequest,
    ) -> anyhow::Result<ProcedureRecordResponse> {
        let conn = self.connect()?;
        let now = Utc::now();
        let procedure = Procedure {
            id: Uuid::new_v4(),
            name: req.name.clone(),
            description: req.description.clone(),
            kind: req.kind,
            status: ProcedureStatus::Candidate,
            trigger: req.trigger.clone(),
            steps: req.steps.clone(),
            success_criteria: req.success_criteria.clone(),
            source_ids: req.source_ids.clone(),
            project: req.project.clone(),
            namespace: req.namespace.clone(),
            use_count: 0,
            confidence: 0.5,
            created_at: now,
            updated_at: now,
            tags: req.tags.clone(),
            session_count: 0,
            last_session: None,
            supersedes: req.supersedes,
        };
        let payload = serde_json::to_string(&procedure)?;
        let kind_str = serde_json::to_value(procedure.kind)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default();
        conn.execute(
            r#"
            INSERT INTO procedures (id, name, kind, status, project, namespace,
                                    use_count, confidence, created_at, updated_at, payload_json)
            VALUES (?1, ?2, ?3, 'candidate', ?4, ?5, 0, 0.5, ?6, ?7, ?8)
            "#,
            params![
                procedure.id.to_string(),
                procedure.name,
                kind_str,
                procedure.project,
                procedure.namespace,
                now.to_rfc3339(),
                now.to_rfc3339(),
                payload,
            ],
        )
        .context("insert procedure")?;

        // G7: Check for conflicting promoted procedures with overlapping triggers.
        let trigger_words: Vec<String> = procedure
            .trigger
            .to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() >= 3)
            .map(String::from)
            .collect();
        let mut conflicts = Vec::new();
        if !trigger_words.is_empty() {
            let conn2 = self.connect()?;
            let mut stmt2 = conn2.prepare(
                "SELECT payload_json FROM procedures WHERE status = 'promoted' AND id != ?1",
            )?;
            let promoted: Vec<Procedure> = stmt2
                .query_map(params![procedure.id.to_string()], |row| {
                    let p: String = row.get(0)?;
                    Ok(p)
                })?
                .filter_map(|r| r.inspect_err(|e| eprintln!("warn: procedure row read: {e}")).ok())
                .filter_map(|p| serde_json::from_str::<Procedure>(&p).inspect_err(|e| eprintln!("warn: procedure json parse: {e}")).ok())
                .collect();
            for existing in promoted {
                let existing_trigger = existing.trigger.to_lowercase();
                let overlap: usize = trigger_words
                    .iter()
                    .filter(|w| existing_trigger.contains(w.as_str()))
                    .count();
                if overlap >= 2 || (trigger_words.len() == 1 && overlap == 1) {
                    conflicts.push(existing);
                }
            }
        }

        Ok(ProcedureRecordResponse {
            procedure,
            conflicts,
        })
    }

    /// G6: Auto-retire procedures with 0 uses and stale for 30+ days.
    pub(crate) fn auto_retire_stale_procedures(&self) -> anyhow::Result<usize> {
        const STALE_DAYS: i64 = 30;
        let cutoff = Utc::now() - chrono::Duration::days(STALE_DAYS);
        let conn = self.connect()?;

        // Find stale candidates: promoted with 0 uses, not updated in 30 days.
        let mut stmt = conn.prepare(
            "SELECT id, payload_json FROM procedures WHERE status = 'promoted' AND use_count = 0 AND updated_at < ?1",
        )?;
        let stale: Vec<(String, String)> = stmt
            .query_map(params![cutoff.to_rfc3339()], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .filter_map(|r| r.inspect_err(|e| eprintln!("warn: stale procedure row read: {e}")).ok())
            .collect();
        drop(stmt);

        let now = Utc::now();
        let mut retired = 0;
        for (id_str, payload) in &stale {
            if let Ok(mut proc) = serde_json::from_str::<Procedure>(payload) {
                proc.status = ProcedureStatus::Retired;
                proc.updated_at = now;
                if let Ok(new_payload) = serde_json::to_string(&proc) {
                    if let Err(e) = conn.execute(
                        "UPDATE procedures SET status = 'retired', updated_at = ?1, payload_json = ?2 WHERE id = ?3",
                        params![now.to_rfc3339(), new_payload, id_str],
                    ) {
                        eprintln!("warn: auto_retire_stale_procedures update: {e:#}");
                    }
                    retired += 1;
                }
            }
        }
        Ok(retired)
    }

    /// Match procedures relevant to a context string.
    ///
    /// Matching is keyword-based against name, description, trigger, and tags.
    /// Only promoted procedures are returned by default.
    pub(crate) fn match_procedures(
        &self,
        req: &ProcedureMatchRequest,
    ) -> anyhow::Result<ProcedureMatchResponse> {
        // G6: Auto-retire stale procedures before matching.
        if let Err(e) = self.auto_retire_stale_procedures() {
            eprintln!("warn: auto_retire_stale_procedures: {e:#}");
        }
        let conn = self.connect()?;
        let mut sql = String::from(
            "SELECT id, payload_json FROM procedures WHERE status = 'promoted'",
        );
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(project) = &req.project {
            sql.push_str(" AND project = ?");
            bind_values.push(project.clone());
        }
        if let Some(namespace) = &req.namespace {
            sql.push_str(" AND namespace = ?");
            bind_values.push(namespace.clone());
        }
        sql.push_str(" ORDER BY use_count DESC, updated_at DESC");
        let limit = req.limit.unwrap_or(10);
        sql.push_str(&format!(" LIMIT {limit}"));

        let mut stmt = conn.prepare(&sql)?;
        let params: Vec<&dyn rusqlite::ToSql> = bind_values
            .iter()
            .map(|v| v as &dyn rusqlite::ToSql)
            .collect();
        let all: Vec<Procedure> = stmt
            .query_map(params.as_slice(), |row| {
                let payload: String = row.get(1)?;
                Ok(payload)
            })?
            .filter_map(|r| r.inspect_err(|e| eprintln!("warn: procedure row read: {e}")).ok())
            .filter_map(|payload| serde_json::from_str::<Procedure>(&payload).inspect_err(|e| eprintln!("warn: procedure json parse: {e}")).ok())
            .collect();

        // Score each procedure against the context string.
        let context_lower = req.context.to_lowercase();
        let context_words: Vec<&str> = context_lower.split_whitespace().collect();

        let mut scored: Vec<(f32, Procedure)> = all
            .into_iter()
            .map(|p| {
                let score = procedure_match_score(&p, &context_words);
                (score, p)
            })
            .filter(|(score, _)| *score > 0.0)
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let limit = req.limit.unwrap_or(5);
        scored.truncate(limit);

        Ok(ProcedureMatchResponse {
            procedures: scored.into_iter().map(|(_, p)| p).collect(),
        })
    }

    /// Promote a candidate procedure to promoted status.
    pub(crate) fn promote_procedure(
        &self,
        req: &ProcedurePromoteRequest,
    ) -> anyhow::Result<ProcedurePromoteResponse> {
        let conn = self.connect()?;
        let id_str = req.procedure_id.to_string();
        let now = Utc::now();

        // Load the current procedure.
        let payload: String = conn
            .query_row(
                "SELECT payload_json FROM procedures WHERE id = ?1",
                params![id_str],
                |row| row.get(0),
            )
            .context("procedure not found")?;

        let mut procedure: Procedure =
            serde_json::from_str(&payload).context("parse procedure")?;
        procedure.status = ProcedureStatus::Promoted;
        procedure.confidence = (procedure.confidence + 0.2).min(1.0);
        procedure.updated_at = now;

        let new_payload = serde_json::to_string(&procedure)?;
        conn.execute(
            r#"
            UPDATE procedures SET status = 'promoted', confidence = ?1,
                                  updated_at = ?2, payload_json = ?3
            WHERE id = ?4
            "#,
            params![
                procedure.confidence,
                now.to_rfc3339(),
                new_payload,
                id_str,
            ],
        )
        .context("promote procedure")?;

        Ok(ProcedurePromoteResponse { procedure })
    }

    /// Record a successful use of a procedure, incrementing use_count.
    pub(crate) fn use_procedure(
        &self,
        req: &ProcedureUseRequest,
    ) -> anyhow::Result<ProcedureUseResponse> {
        let conn = self.connect()?;
        let id_str = req.procedure_id.to_string();
        let now = Utc::now();

        let payload: String = conn
            .query_row(
                "SELECT payload_json FROM procedures WHERE id = ?1",
                params![id_str],
                |row| row.get(0),
            )
            .context("procedure not found")?;

        let mut procedure: Procedure =
            serde_json::from_str(&payload).context("parse procedure")?;
        procedure.use_count += 1;
        procedure.confidence = (procedure.confidence + 0.05).min(1.0);
        procedure.updated_at = now;

        // Cross-session tracking.
        if let Some(session) = &req.session {
            let is_new_session = procedure
                .last_session
                .as_ref()
                .is_none_or(|prev| prev != session);
            if is_new_session {
                procedure.session_count += 1;
            }
            procedure.last_session = Some(session.clone());
        }

        // G1: Auto-promote when use_count >= 3 AND session_count >= 2.
        const AUTO_PROMOTE_USE_COUNT: usize = 3;
        const AUTO_PROMOTE_SESSION_COUNT: usize = 2;
        let auto_promoted = procedure.status == ProcedureStatus::Candidate
            && procedure.use_count >= AUTO_PROMOTE_USE_COUNT
            && procedure.session_count >= AUTO_PROMOTE_SESSION_COUNT;
        if auto_promoted {
            procedure.status = ProcedureStatus::Promoted;
            procedure.confidence = (procedure.confidence + 0.2).min(1.0);
        }

        let new_payload = serde_json::to_string(&procedure)?;
        let status_str = if auto_promoted { "promoted" } else {
            match procedure.status {
                ProcedureStatus::Candidate => "candidate",
                ProcedureStatus::Promoted => "promoted",
                ProcedureStatus::Retired => "retired",
            }
        };
        conn.execute(
            r#"
            UPDATE procedures SET status = ?1, use_count = ?2, confidence = ?3,
                                  updated_at = ?4, payload_json = ?5
            WHERE id = ?6
            "#,
            params![
                status_str,
                procedure.use_count as i64,
                procedure.confidence,
                now.to_rfc3339(),
                new_payload,
                id_str,
            ],
        )
        .context("record procedure use")?;

        Ok(ProcedureUseResponse {
            procedure,
            auto_promoted,
        })
    }

    /// Retire a procedure (manual or automatic).
    pub(crate) fn retire_procedure(
        &self,
        req: &ProcedureRetireRequest,
    ) -> anyhow::Result<ProcedureRetireResponse> {
        let conn = self.connect()?;
        let id_str = req.procedure_id.to_string();
        let now = Utc::now();

        let payload: String = conn
            .query_row(
                "SELECT payload_json FROM procedures WHERE id = ?1",
                params![id_str],
                |row| row.get(0),
            )
            .context("procedure not found")?;

        let mut procedure: Procedure =
            serde_json::from_str(&payload).context("parse procedure")?;
        procedure.status = ProcedureStatus::Retired;
        procedure.updated_at = now;

        let new_payload = serde_json::to_string(&procedure)?;
        conn.execute(
            r#"
            UPDATE procedures SET status = 'retired', updated_at = ?1, payload_json = ?2
            WHERE id = ?3
            "#,
            params![now.to_rfc3339(), new_payload, id_str],
        )
        .context("retire procedure")?;

        Ok(ProcedureRetireResponse { procedure })
    }

    /// Detect candidate procedures from repeated episodic event patterns.
    ///
    /// Scans the event spine for entities with repeated events, extracts
    /// the event summaries as steps, and creates candidate procedures.
    pub(crate) fn detect_procedures(
        &self,
        req: &ProcedureDetectRequest,
    ) -> anyhow::Result<ProcedureDetectResponse> {
        let min_events = req.min_events.unwrap_or(3).max(2);
        let lookback_days = req.lookback_days.unwrap_or(14).max(1);
        let max_candidates = req.max_candidates.unwrap_or(5).min(20);
        let cutoff = Utc::now() - chrono::Duration::days(lookback_days);

        // Find entities with repeated events.
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT entity_id, COUNT(*) AS event_count
            FROM memory_events
            WHERE entity_id != ''
              AND recorded_at >= ?1
            GROUP BY entity_id
            HAVING COUNT(*) >= ?2
            ORDER BY event_count DESC
            LIMIT ?3
            "#,
        )?;
        let rows: Vec<(String, i64)> = stmt
            .query_map(
                params![
                    cutoff.to_rfc3339(),
                    min_events as i64,
                    (max_candidates * 3) as i64, // over-fetch to filter
                ],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
            )?
            .filter_map(|r| r.inspect_err(|e| eprintln!("warn: detect candidate row read: {e}")).ok())
            .collect();
        drop(stmt);
        drop(conn);

        let scanned = rows.len();
        let mut created = 0;
        let mut procedures = Vec::new();

        for (entity_id_str, _event_count) in rows {
            if created >= max_candidates {
                break;
            }
            let entity_id = match Uuid::parse_str(&entity_id_str) {
                Ok(id) => id,
                Err(_) => continue,
            };

            // Get the entity details.
            let entity = match self.entity_by_id(entity_id)? {
                Some(e) => e,
                None => continue,
            };

            // Check project/namespace filters.
            if let Some(project) = &req.project {
                let entity_project = entity
                    .context
                    .as_ref()
                    .and_then(|c| c.project.as_ref());
                if entity_project != Some(project) {
                    continue;
                }
            }
            if let Some(namespace) = &req.namespace {
                let entity_ns = entity
                    .context
                    .as_ref()
                    .and_then(|c| c.namespace.as_ref());
                if entity_ns != Some(namespace) {
                    continue;
                }
            }

            // Get recent events for this entity.
            let events = self.events_for_entity(entity_id, 20)?;
            if events.len() < min_events {
                continue;
            }

            // Derive a name from the entity.
            let procedure_name = entity
                .aliases
                .first()
                .cloned()
                .unwrap_or_else(|| format!("{}:{}", entity.entity_type, &entity.id.to_string()[..8]));

            // Check if a procedure already exists for this entity.
            let already_exists = {
                let conn = self.connect()?;
                let count: i64 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM procedures WHERE name = ?1 AND status != 'retired'",
                        params![procedure_name],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);
                count > 0
            };
            if already_exists {
                continue;
            }

            // Extract steps from event summaries (deduplicated, ordered).
            let mut seen_summaries = Vec::new();
            let mut steps = Vec::new();
            for event in events.iter().rev() {
                let summary = event.summary.trim();
                if !summary.is_empty() && !seen_summaries.contains(&summary.to_lowercase()) {
                    seen_summaries.push(summary.to_lowercase());
                    steps.push(summary.to_string());
                }
                if steps.len() >= 8 {
                    break;
                }
            }
            if steps.is_empty() {
                continue;
            }

            let entity_type = &entity.entity_type;
            let project = entity
                .context
                .as_ref()
                .and_then(|c| c.project.clone());
            let namespace = entity
                .context
                .as_ref()
                .and_then(|c| c.namespace.clone());

            // X1: Filter out events whose entities have been superseded.
            let source_ids: Vec<Uuid> = events
                .iter()
                .filter_map(|e| e.entity_id)
                .filter(|eid| {
                    self.entity_by_id(*eid)
                        .ok()
                        .flatten()
                        .is_some_and(|ent| ent.current_state.as_deref() != Some("superseded"))
                })
                .collect();

            let record_req = ProcedureRecordRequest {
                name: procedure_name.clone(),
                description: format!(
                    "Auto-detected from {} repeated events on {} entity",
                    events.len(),
                    entity_type
                ),
                kind: ProcedureKind::Workflow,
                trigger: format!("when working with {}", procedure_name),
                steps,
                success_criteria: None,
                source_ids,
                project,
                namespace,
                tags: vec!["auto-detected".to_string()],
                supersedes: None,
            };

            match self.record_procedure(&record_req) {
                Ok(resp) => {
                    procedures.push(resp.procedure);
                    created += 1;
                }
                Err(_) => continue,
            }
        }

        Ok(ProcedureDetectResponse {
            scanned,
            created,
            procedures,
        })
    }
}

/// Score how well a procedure matches a set of context words.
fn procedure_match_score(p: &Procedure, context_words: &[&str]) -> f32 {
    let haystack = format!(
        "{} {} {} {} {}",
        p.name.to_lowercase(),
        p.description.to_lowercase(),
        p.trigger.to_lowercase(),
        p.steps.join(" ").to_lowercase(),
        p.tags.join(" ").to_lowercase(),
    );
    let mut hits = 0usize;
    for word in context_words {
        if word.len() >= 3 && haystack.contains(word) {
            hits += 1;
        }
    }
    if context_words.is_empty() {
        return 0.0;
    }
    let ratio = hits as f32 / context_words.len() as f32;
    // Boost by use_count and confidence.
    ratio * (1.0 + (p.use_count as f32).ln().max(0.0)) * p.confidence
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store() -> SqliteStore {
        let path = std::env::temp_dir()
            .join(format!("memd-proc-test-{}.db", uuid::Uuid::new_v4()));
        SqliteStore::open(&path).unwrap()
    }

    #[test]
    fn record_and_list_procedure() {
        let store = temp_store();
        let req = ProcedureRecordRequest {
            name: "deploy to prod".into(),
            description: "Standard production deployment workflow".into(),
            kind: ProcedureKind::Workflow,
            trigger: "when deploying to production".into(),
            steps: vec![
                "run tests".into(),
                "build release".into(),
                "deploy via portainer".into(),
                "verify health".into(),
            ],
            success_criteria: Some("health check returns 200".into()),
            source_ids: vec![],
            project: Some("memd".into()),
            namespace: Some("main".into()),
            tags: vec!["deploy".into(), "production".into()],
            supersedes: None,
        };
        let resp = store.record_procedure(&req).unwrap();
        assert_eq!(resp.procedure.name, "deploy to prod");
        assert_eq!(resp.procedure.status, ProcedureStatus::Candidate);
        assert_eq!(resp.procedure.use_count, 0);

        let list = store
            .list_procedures(&ProcedureListRequest {
                project: Some("memd".into()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(list.procedures.len(), 1);
        assert_eq!(list.procedures[0].steps.len(), 4);
    }

    #[test]
    fn promote_procedure_changes_status() {
        let store = temp_store();
        let rec = store
            .record_procedure(&ProcedureRecordRequest {
                name: "fix ssh ban".into(),
                description: "Unban IP from fail2ban via proxmox".into(),
                kind: ProcedureKind::Recovery,
                trigger: "when SSH is blocked on a VM".into(),
                steps: vec!["qm guest exec to unban IP".into()],
                success_criteria: Some("SSH connects again".into()),
                source_ids: vec![],
                project: None,
                namespace: None,
                tags: vec!["ssh".into(), "recovery".into()],
                supersedes: None,
            })
            .unwrap();
        assert_eq!(rec.procedure.status, ProcedureStatus::Candidate);

        let promoted = store
            .promote_procedure(&ProcedurePromoteRequest {
                procedure_id: rec.procedure.id,
            })
            .unwrap();
        assert_eq!(promoted.procedure.status, ProcedureStatus::Promoted);
        assert!(promoted.procedure.confidence > rec.procedure.confidence);
    }

    #[test]
    fn use_procedure_increments_count() {
        let store = temp_store();
        let rec = store
            .record_procedure(&ProcedureRecordRequest {
                name: "build memd".into(),
                description: "cargo build workflow".into(),
                kind: ProcedureKind::Workflow,
                trigger: "when building memd".into(),
                steps: vec!["cargo build --release".into()],
                success_criteria: None,
                source_ids: vec![],
                project: Some("memd".into()),
                namespace: None,
                tags: vec![],
                supersedes: None,
            })
            .unwrap();

        // Promote first so it shows up in match results.
        store
            .promote_procedure(&ProcedurePromoteRequest {
                procedure_id: rec.procedure.id,
            })
            .unwrap();

        let used = store
            .use_procedure(&ProcedureUseRequest {
                procedure_id: rec.procedure.id,
                session: None,
            })
            .unwrap();
        assert_eq!(used.procedure.use_count, 1);

        let used2 = store
            .use_procedure(&ProcedureUseRequest {
                procedure_id: rec.procedure.id,
                session: None,
            })
            .unwrap();
        assert_eq!(used2.procedure.use_count, 2);
    }

    #[test]
    fn match_procedures_returns_promoted_only() {
        let store = temp_store();
        // Create two procedures, promote only one.
        let rec1 = store
            .record_procedure(&ProcedureRecordRequest {
                name: "deploy workflow".into(),
                description: "production deploy steps".into(),
                kind: ProcedureKind::Workflow,
                trigger: "deploying to production".into(),
                steps: vec!["build".into(), "deploy".into()],
                success_criteria: None,
                source_ids: vec![],
                project: Some("memd".into()),
                namespace: None,
                tags: vec!["deploy".into()],
                supersedes: None,
            })
            .unwrap();
        store
            .record_procedure(&ProcedureRecordRequest {
                name: "candidate workflow".into(),
                description: "still being validated".into(),
                kind: ProcedureKind::Workflow,
                trigger: "deploying".into(),
                steps: vec!["test".into()],
                success_criteria: None,
                source_ids: vec![],
                project: Some("memd".into()),
                namespace: None,
                tags: vec![],
                supersedes: None,
            })
            .unwrap();

        store
            .promote_procedure(&ProcedurePromoteRequest {
                procedure_id: rec1.procedure.id,
            })
            .unwrap();

        let matches = store
            .match_procedures(&ProcedureMatchRequest {
                context: "I need to deploy to production".into(),
                project: Some("memd".into()),
                namespace: None,
                limit: None,
            })
            .unwrap();
        assert_eq!(matches.procedures.len(), 1);
        assert_eq!(matches.procedures[0].name, "deploy workflow");
    }

    #[test]
    fn filter_by_kind_and_status() {
        let store = temp_store();
        store
            .record_procedure(&ProcedureRecordRequest {
                name: "recovery pattern".into(),
                description: "fix broken thing".into(),
                kind: ProcedureKind::Recovery,
                trigger: "when thing breaks".into(),
                steps: vec!["fix it".into()],
                success_criteria: None,
                source_ids: vec![],
                project: None,
                namespace: None,
                tags: vec![],
                supersedes: None,
            })
            .unwrap();
        store
            .record_procedure(&ProcedureRecordRequest {
                name: "workflow pattern".into(),
                description: "do thing".into(),
                kind: ProcedureKind::Workflow,
                trigger: "when doing thing".into(),
                steps: vec!["do it".into()],
                success_criteria: None,
                source_ids: vec![],
                project: None,
                namespace: None,
                tags: vec![],
                supersedes: None,
            })
            .unwrap();

        let recovery_only = store
            .list_procedures(&ProcedureListRequest {
                kind: Some(ProcedureKind::Recovery),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(recovery_only.procedures.len(), 1);
        assert_eq!(recovery_only.procedures[0].name, "recovery pattern");
    }

    #[test]
    fn retire_procedure_sets_retired_status() {
        let store = temp_store();
        let rec = store
            .record_procedure(&ProcedureRecordRequest {
                name: "old workflow".into(),
                description: "no longer useful".into(),
                kind: ProcedureKind::Workflow,
                trigger: "never".into(),
                steps: vec!["step1".into()],
                success_criteria: None,
                source_ids: vec![],
                project: None,
                namespace: None,
                tags: vec![],
                supersedes: None,
            })
            .unwrap();

        // Promote first so it's visible in match.
        store
            .promote_procedure(&ProcedurePromoteRequest {
                procedure_id: rec.procedure.id,
            })
            .unwrap();
        let before = store
            .match_procedures(&ProcedureMatchRequest {
                context: "old workflow".into(),
                project: None,
                namespace: None,
                limit: None,
            })
            .unwrap();
        assert_eq!(before.procedures.len(), 1);

        // Now retire — should disappear from match results.
        let retired = store
            .retire_procedure(&ProcedureRetireRequest {
                procedure_id: rec.procedure.id,
            })
            .unwrap();
        assert_eq!(retired.procedure.status, ProcedureStatus::Retired);

        let after = store
            .match_procedures(&ProcedureMatchRequest {
                context: "old workflow".into(),
                project: None,
                namespace: None,
                limit: None,
            })
            .unwrap();
        assert_eq!(after.procedures.len(), 0);
    }

    #[test]
    fn use_procedure_tracks_sessions() {
        let store = temp_store();
        let rec = store
            .record_procedure(&ProcedureRecordRequest {
                name: "session tracked".into(),
                description: "tracks sessions".into(),
                kind: ProcedureKind::Workflow,
                trigger: "always".into(),
                steps: vec!["do thing".into()],
                success_criteria: None,
                source_ids: vec![],
                project: None,
                namespace: None,
                tags: vec![],
                supersedes: None,
            })
            .unwrap();

        // First use with session A.
        let used1 = store
            .use_procedure(&ProcedureUseRequest {
                procedure_id: rec.procedure.id,
                session: Some("session-aaa".into()),
            })
            .unwrap();
        assert_eq!(used1.procedure.session_count, 1);
        assert_eq!(
            used1.procedure.last_session.as_deref(),
            Some("session-aaa")
        );

        // Second use same session — session_count stays 1.
        let used2 = store
            .use_procedure(&ProcedureUseRequest {
                procedure_id: rec.procedure.id,
                session: Some("session-aaa".into()),
            })
            .unwrap();
        assert_eq!(used2.procedure.session_count, 1);
        assert_eq!(used2.procedure.use_count, 2);

        // Third use different session — session_count goes to 2.
        let used3 = store
            .use_procedure(&ProcedureUseRequest {
                procedure_id: rec.procedure.id,
                session: Some("session-bbb".into()),
            })
            .unwrap();
        assert_eq!(used3.procedure.session_count, 2);
        assert_eq!(used3.procedure.use_count, 3);
        assert_eq!(
            used3.procedure.last_session.as_deref(),
            Some("session-bbb")
        );
    }

    #[test]
    fn detect_procedures_from_events() {
        let store = temp_store();

        // Create an entity and record events for it.
        let now = chrono::Utc::now();
        let entity = memd_schema::MemoryEntityRecord {
            id: Uuid::new_v4(),
            entity_type: "workflow".to_string(),
            aliases: vec!["deploy-workflow".to_string()],
            current_state: None,
            state_version: 0,
            confidence: 0.8,
            salience_score: 0.7,
            rehearsal_count: 0,
            created_at: now,
            updated_at: now,
            last_accessed_at: None,
            last_seen_at: None,
            valid_from: None,
            valid_to: None,
            tags: vec![],
            context: Some(memd_schema::MemoryContextFrame {
                at: None,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                repo: None,
                host: None,
                branch: None,
                agent: None,
                location: None,
            }),
        };
        // Insert entity manually.
        {
            let conn = store.connect().unwrap();
            let payload = serde_json::to_string(&entity).unwrap();
            conn.execute(
                "INSERT INTO memory_entities (id, entity_key, entity_type, updated_at, payload_json) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    entity.id.to_string(),
                    "deploy-workflow",
                    entity.entity_type,
                    entity.updated_at.to_rfc3339(),
                    payload,
                ],
            ).unwrap();
        }

        // Insert a memory item to link events to.
        let item_id = Uuid::new_v4();
        {
            let conn = store.connect().unwrap();
            let now = chrono::Utc::now();
            conn.execute(
                "INSERT INTO memory_items (id, kind, scope, stage, project, namespace, source_agent, redundancy_key, status, confidence, canonical_key, updated_at, payload_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                rusqlite::params![
                    item_id.to_string(),
                    "procedural",
                    "project",
                    "canonical",
                    "memd",
                    "main",
                    "",
                    "",
                    "active",
                    0.8f32,
                    format!("procedural::{}", item_id),
                    now.to_rfc3339(),
                    "{}",
                ],
            ).unwrap();
        }

        // Record 4 events for this entity.
        use crate::store::RecordEventArgs;
        for (i, summary) in ["run tests", "build release", "deploy via portainer", "verify health"]
            .iter()
            .enumerate()
        {
            store
                .record_event(
                    &entity,
                    item_id,
                    RecordEventArgs {
                        event_type: "workflow_step".to_string(),
                        summary: summary.to_string(),
                        occurred_at: chrono::Utc::now()
                            + chrono::Duration::seconds(i as i64),
                        project: Some("memd".to_string()),
                        namespace: Some("main".to_string()),
                        workspace: None,
                        source_agent: None,
                        source_system: None,
                        source_path: None,
                        related_entity_ids: vec![],
                        tags: vec![],
                        context: None,
                        confidence: 0.8,
                        salience_score: 0.7,
                    },
                )
                .unwrap();
        }

        // Detect procedures.
        let detected = store
            .detect_procedures(&ProcedureDetectRequest {
                project: Some("memd".to_string()),
                namespace: None,
                min_events: Some(3),
                lookback_days: Some(1),
                max_candidates: Some(5),
            })
            .unwrap();

        assert!(detected.scanned > 0);
        assert_eq!(detected.created, 1);
        assert_eq!(detected.procedures[0].name, "deploy-workflow");
        assert_eq!(detected.procedures[0].status, ProcedureStatus::Candidate);
        assert!(detected.procedures[0].tags.contains(&"auto-detected".to_string()));
        assert!(detected.procedures[0].steps.len() >= 3);

        // Running detect again should NOT create duplicates.
        let detected2 = store
            .detect_procedures(&ProcedureDetectRequest {
                project: Some("memd".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(detected2.created, 0);
    }

    #[test]
    fn auto_promote_after_threshold() {
        let store = temp_store();
        let rec = store
            .record_procedure(&ProcedureRecordRequest {
                name: "auto promote candidate".into(),
                description: "should auto-promote".into(),
                kind: ProcedureKind::Workflow,
                trigger: "when auto-promoting".into(),
                steps: vec!["step1".into()],
                success_criteria: None,
                source_ids: vec![],
                project: None,
                namespace: None,
                tags: vec![],
                supersedes: None,
            })
            .unwrap();
        assert_eq!(rec.procedure.status, ProcedureStatus::Candidate);

        // Use across 2 sessions, 3+ times total.
        store
            .use_procedure(&ProcedureUseRequest {
                procedure_id: rec.procedure.id,
                session: Some("sess-a".into()),
            })
            .unwrap();
        store
            .use_procedure(&ProcedureUseRequest {
                procedure_id: rec.procedure.id,
                session: Some("sess-a".into()),
            })
            .unwrap();
        // 2 uses, 1 session — still candidate.
        let after2 = store
            .use_procedure(&ProcedureUseRequest {
                procedure_id: rec.procedure.id,
                session: Some("sess-b".into()),
            })
            .unwrap();
        // 3 uses, 2 sessions — should auto-promote.
        assert_eq!(after2.procedure.status, ProcedureStatus::Promoted);
        assert!(after2.auto_promoted);
    }

    #[test]
    fn auto_retire_stale_procedures() {
        let store = temp_store();
        let rec = store
            .record_procedure(&ProcedureRecordRequest {
                name: "stale procedure".into(),
                description: "will be stale".into(),
                kind: ProcedureKind::Workflow,
                trigger: "never used".into(),
                steps: vec!["step1".into()],
                success_criteria: None,
                source_ids: vec![],
                project: None,
                namespace: None,
                tags: vec![],
                supersedes: None,
            })
            .unwrap();

        // Promote it.
        store
            .promote_procedure(&ProcedurePromoteRequest {
                procedure_id: rec.procedure.id,
            })
            .unwrap();

        // Manually backdate updated_at to 31 days ago.
        let stale_date = Utc::now() - chrono::Duration::days(31);
        {
            let conn = store.connect().unwrap();
            conn.execute(
                "UPDATE procedures SET updated_at = ?1 WHERE id = ?2",
                params![stale_date.to_rfc3339(), rec.procedure.id.to_string()],
            )
            .unwrap();
        }

        let retired = store.auto_retire_stale_procedures().unwrap();
        assert_eq!(retired, 1);

        // Should not appear in match results.
        let matches = store
            .match_procedures(&ProcedureMatchRequest {
                context: "stale procedure".into(),
                project: None,
                namespace: None,
                limit: None,
            })
            .unwrap();
        assert_eq!(matches.procedures.len(), 0);
    }

    #[test]
    fn conflict_detection_on_record() {
        let store = temp_store();
        // Create and promote a procedure.
        let rec1 = store
            .record_procedure(&ProcedureRecordRequest {
                name: "deploy to production".into(),
                description: "production deploy".into(),
                kind: ProcedureKind::Workflow,
                trigger: "when deploying to production servers".into(),
                steps: vec!["build".into(), "deploy".into()],
                success_criteria: None,
                source_ids: vec![],
                project: None,
                namespace: None,
                tags: vec![],
                supersedes: None,
            })
            .unwrap();
        store
            .promote_procedure(&ProcedurePromoteRequest {
                procedure_id: rec1.procedure.id,
            })
            .unwrap();

        // Record a new procedure with overlapping trigger.
        let rec2 = store
            .record_procedure(&ProcedureRecordRequest {
                name: "deploy v2".into(),
                description: "new deploy".into(),
                kind: ProcedureKind::Workflow,
                trigger: "when deploying to production environment".into(),
                steps: vec!["new step".into()],
                success_criteria: None,
                source_ids: vec![],
                project: None,
                namespace: None,
                tags: vec![],
                supersedes: None,
            })
            .unwrap();

        // Should detect conflict with the promoted procedure.
        assert!(!rec2.conflicts.is_empty());
        assert_eq!(rec2.conflicts[0].name, "deploy to production");
    }

    #[test]
    fn supersedes_field_persists() {
        let store = temp_store();
        let rec1 = store
            .record_procedure(&ProcedureRecordRequest {
                name: "original workflow".into(),
                description: "v1".into(),
                kind: ProcedureKind::Workflow,
                trigger: "original".into(),
                steps: vec!["old step".into()],
                success_criteria: None,
                source_ids: vec![],
                project: None,
                namespace: None,
                tags: vec![],
                supersedes: None,
            })
            .unwrap();

        // Record a new procedure that supersedes the first.
        let rec2 = store
            .record_procedure(&ProcedureRecordRequest {
                name: "updated workflow".into(),
                description: "v2".into(),
                kind: ProcedureKind::Workflow,
                trigger: "updated".into(),
                steps: vec!["new step".into()],
                success_criteria: None,
                source_ids: vec![],
                project: None,
                namespace: None,
                tags: vec![],
                supersedes: Some(rec1.procedure.id),
            })
            .unwrap();

        assert_eq!(rec2.procedure.supersedes, Some(rec1.procedure.id));

        // Verify it persists through list.
        let list = store
            .list_procedures(&ProcedureListRequest::default())
            .unwrap();
        let found = list.procedures.iter().find(|p| p.name == "updated workflow").unwrap();
        assert_eq!(found.supersedes, Some(rec1.procedure.id));
    }
}
