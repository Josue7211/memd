use anyhow::Context;
use memd_schema::{
    HiveClaimsRequest, HiveSessionAutoRetireResponse, HiveSessionRetireRequest, HiveSessionsRequest,
};
use rusqlite::params;

use crate::store::SqliteStore;
use crate::store_hive::{hive_session_is_active_at, is_ephemeral_proof_hive_session};

impl SqliteStore {
    pub(crate) fn prune_expired_hive_claims(&self) -> anyhow::Result<()> {
        let conn = self.connect()?;
        conn.execute(
            "DELETE FROM hive_claims WHERE expires_at <= ?1",
            params![chrono::Utc::now().to_rfc3339()],
        )
        .context("prune expired hive claims")?;
        Ok(())
    }

    pub(crate) fn prune_stale_hive_sessions(&self) -> anyhow::Result<()> {
        let conn = self.connect()?;
        conn.execute(
            "DELETE FROM hive_sessions WHERE last_seen < ?1",
            params![(chrono::Utc::now() - chrono::TimeDelta::hours(24)).to_rfc3339()],
        )
        .context("prune stale hive sessions")?;
        Ok(())
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
            .hive_tasks(&memd_schema::HiveTasksRequest {
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
            .hive_coordination_receipts(&memd_schema::HiveCoordinationReceiptsRequest {
                session: None,
                project: project.clone(),
                namespace: namespace.clone(),
                workspace: workspace.clone(),
                limit: Some(512),
            })?
            .receipts;

        let retireable = sessions
            .into_iter()
            .filter(|session| !hive_session_is_active_at(session, now))
            .filter(|session| {
                if is_ephemeral_proof_hive_session(session) {
                    return true;
                }
                !tasks.iter().any(|task| {
                    task.session.as_deref() == Some(session.session.as_str())
                        && task.status != "done"
                        && task.status != "closed"
                })
            })
            .filter(|session| {
                if is_ephemeral_proof_hive_session(session) {
                    return true;
                }
                !claims.iter().any(|claim| claim.session == session.session)
            })
            .filter(|session| {
                if is_ephemeral_proof_hive_session(session) {
                    return true;
                }
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
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;
    use memd_schema::{
        HiveClaimAcquireRequest, HiveClaimsRequest, HiveSessionUpsertRequest, HiveSessionsRequest,
        HiveTaskUpsertRequest,
    };
    use rusqlite::params;
    use uuid::Uuid;

    use super::*;

    #[test]
    fn auto_retire_stale_hive_sessions_prunes_expired_claims_and_stale_sessions() {
        let dir = std::env::temp_dir().join(format!("memd-hive-lifecycle-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "session-dogfood-proof".to_string(),
                agent: Some("Avicenna".to_string()),
                effective_agent: Some("Avicenna@session-dogfood-proof".to_string()),
                hive_system: Some("avicenna".to_string()),
                hive_role: Some("worker".to_string()),
                worker_name: Some("Avicenna".to_string()),
                display_name: None,
                role: Some("worker".to_string()),
                capabilities: vec!["coordination".to_string()],
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
                host: None,
                pid: Some(612),
                topic_claim: Some("Dogfood parser lane".to_string()),
                scope_claims: vec!["crates/memd-client/src/main.rs".to_string()],
                task_id: Some("dogfood-parser-proof".to_string()),
                focus: None,
                pressure: None,
                next_recovery: None,
                next_action: None,
                working: None,
                touches: Vec::new(),
                blocked_by: Vec::new(),
                cowork_with: Vec::new(),
                handoff_target: None,
                offered_to: Vec::new(),
                needs_help: false,
                needs_review: false,
                handoff_state: None,
                confidence: None,
                risk: None,
                status: Some("live".to_string()),
            })
            .expect("insert proof worker");

        store
            .upsert_hive_task(&HiveTaskUpsertRequest {
                task_id: "dogfood-parser-proof".to_string(),
                title: "Dogfood parser lane".to_string(),
                description: None,
                status: Some("active".to_string()),
                coordination_mode: Some("solo".to_string()),
                session: Some("session-dogfood-proof".to_string()),
                agent: Some("Avicenna".to_string()),
                effective_agent: Some("Avicenna@session-dogfood-proof".to_string()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                claim_scopes: vec!["crates/memd-client/src/main.rs".to_string()],
                help_requested: Some(false),
                review_requested: Some(false),
            })
            .expect("insert proof task");

        store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "session-stale-housekeeping".to_string(),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@session-stale-housekeeping".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("worker".to_string()),
                worker_name: Some("codex".to_string()),
                display_name: None,
                role: Some("worker".to_string()),
                capabilities: vec!["coordination".to_string()],
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
                host: None,
                pid: Some(613),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: None,
                pressure: None,
                next_recovery: None,
                next_action: None,
                working: None,
                touches: Vec::new(),
                blocked_by: Vec::new(),
                cowork_with: Vec::new(),
                handoff_target: None,
                offered_to: Vec::new(),
                needs_help: false,
                needs_review: false,
                handoff_state: None,
                confidence: None,
                risk: None,
                status: Some("live".to_string()),
            })
            .expect("insert stale session");

        store
            .acquire_hive_claim(&HiveClaimAcquireRequest {
                scope: "scope:memd:cleanup".to_string(),
                session: "session-dogfood-proof".to_string(),
                tab_id: None,
                agent: Some("Avicenna".to_string()),
                effective_agent: Some("Avicenna@session-dogfood-proof".to_string()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                host: None,
                pid: None,
                ttl_seconds: 60,
            })
            .expect("insert claim");

        let mut proof_session = store
            .hive_sessions(&HiveSessionsRequest {
                session: Some("session-dogfood-proof".to_string()),
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
                limit: Some(1),
            })
            .expect("load proof worker")
            .sessions
            .into_iter()
            .next()
            .expect("proof worker exists");
        proof_session.last_seen = chrono::Utc::now() - TimeDelta::minutes(6);

        let mut stale_session = store
            .hive_sessions(&HiveSessionsRequest {
                session: Some("session-stale-housekeeping".to_string()),
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
                limit: Some(1),
            })
            .expect("load stale worker")
            .sessions
            .into_iter()
            .next()
            .expect("stale worker exists");
        stale_session.last_seen = chrono::Utc::now() - chrono::TimeDelta::hours(25);

        let conn = store.connect().expect("connect sqlite");
        conn.execute(
            "UPDATE hive_sessions SET last_seen = ?1, payload_json = ?2 WHERE session = ?3",
            params![
                proof_session.last_seen.to_rfc3339(),
                serde_json::to_string(&proof_session).expect("serialize proof worker"),
                proof_session.session.as_str(),
            ],
        )
        .expect("age proof worker");
        conn.execute(
            "UPDATE hive_sessions SET last_seen = ?1, payload_json = ?2 WHERE session = ?3",
            params![
                stale_session.last_seen.to_rfc3339(),
                serde_json::to_string(&stale_session).expect("serialize stale worker"),
                stale_session.session.as_str(),
            ],
        )
        .expect("age stale worker");
        conn.execute(
            "UPDATE hive_claims SET expires_at = ?1 WHERE scope = ?2",
            params![
                (chrono::Utc::now() - TimeDelta::minutes(1)).to_rfc3339(),
                "scope:memd:cleanup",
            ],
        )
        .expect("expire claim");

        let retired = store
            .auto_retire_stale_hive_sessions(
                Some("memd"),
                Some("main"),
                Some("shared"),
                chrono::Utc::now(),
            )
            .expect("auto retire");
        assert_eq!(retired.retired, vec!["session-dogfood-proof".to_string()]);

        let remaining_sessions = store
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
            remaining_sessions
                .sessions
                .iter()
                .all(|session| session.session != "session-stale-housekeeping")
        );

        let remaining_claims = store
            .hive_claims(&HiveClaimsRequest {
                session: None,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                active_only: Some(false),
                limit: Some(8),
            })
            .expect("list claims after retire");
        assert!(
            remaining_claims
                .claims
                .iter()
                .all(|claim| claim.scope != "scope:memd:cleanup")
        );

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }
}
