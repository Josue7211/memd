use super::*;
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
fn retire_hive_session_removes_scope_sibling_rows_for_same_session() {
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
        .expect("insert claude session");
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
            tab_id: Some("tab-c".to_string()),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            base_branch: None,
            workspace: Some("other".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            base_url_healthy: Some(true),
            host: Some("workstation".to_string()),
            pid: Some(333),
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
        .expect("insert other workspace session");

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
    assert_eq!(retired.retired, 2);
    assert!(
        retired
            .sessions
            .iter()
            .any(|record| record.agent.as_deref() == Some("codex"))
    );
    assert!(
        retired
            .sessions
            .iter()
            .any(|record| record.agent.as_deref() == Some("claude-code"))
    );

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
    assert_eq!(sessions.sessions.len(), 0);

    let other_workspace_sessions = store
        .hive_sessions(&HiveSessionsRequest {
            session: Some("shared-session".to_string()),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            workspace: Some("other".to_string()),
            hive_system: None,
            hive_role: None,
            host: None,
            hive_group: None,
            active_only: Some(false),
            limit: Some(8),
        })
        .expect("query other workspace sessions");
    assert_eq!(other_workspace_sessions.sessions.len(), 1);
    assert_eq!(
        other_workspace_sessions.sessions[0].workspace.as_deref(),
        Some("other")
    );

    std::fs::remove_dir_all(dir).expect("cleanup temp dir");
}

#[test]
fn hive_coordination_auto_retires_stale_session_without_owned_work() {
    let dir = std::env::temp_dir().join(format!("memd-hive-auto-retire-{}", uuid::Uuid::new_v4()));
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
fn hive_sessions_collapse_duplicate_rows_per_session_and_preserve_richer_identity() {
    let dir = std::env::temp_dir().join(format!("memd-hive-dedupe-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

    store
        .upsert_hive_session(&HiveSessionUpsertRequest {
            session: "codex-fresh".to_string(),
            agent: Some("codex".to_string()),
            effective_agent: Some("codex@codex-fresh".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("agent".to_string()),
            worker_name: Some("codex".to_string()),
            display_name: None,
            role: Some("agent".to_string()),
            capabilities: vec!["coordination".to_string(), "memory".to_string()],
            hive_groups: vec!["project:memd".to_string()],
            lane_id: None,
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: None,
            tab_id: Some("tab-alpha".to_string()),
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
            pid: Some(123),
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
        .expect("insert richer session row");

    store
        .upsert_hive_session(&HiveSessionUpsertRequest {
            session: "codex-fresh".to_string(),
            agent: Some("codex".to_string()),
            effective_agent: Some("codex@codex-fresh".to_string()),
            hive_system: None,
            hive_role: None,
            worker_name: Some("codex".to_string()),
            display_name: None,
            role: None,
            capabilities: Vec::new(),
            hive_groups: vec!["project:memd".to_string()],
            lane_id: None,
            hive_group_goal: None,
            authority: None,
            heartbeat_model: None,
            tab_id: Some("tab-alpha".to_string()),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            base_branch: None,
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            base_url_healthy: None,
            host: None,
            pid: Some(123),
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
        .expect("insert newer sparse session row");

    let response = store
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
            limit: Some(16),
        })
        .expect("query deduped sessions");

    assert_eq!(response.sessions.len(), 1);
    let session = &response.sessions[0];
    assert_eq!(session.session, "codex-fresh");
    assert_eq!(session.hive_system.as_deref(), Some("codex"));
    assert_eq!(session.hive_role.as_deref(), Some("agent"));
    assert_eq!(session.role.as_deref(), Some("agent"));
    assert_eq!(session.authority.as_deref(), Some("participant"));
    assert_eq!(
        session.capabilities,
        vec!["coordination".to_string(), "memory".to_string()]
    );

    std::fs::remove_dir_all(dir).expect("cleanup temp dir");
}

#[test]
fn hive_sessions_collapse_duplicate_rows_prefers_stronger_newer_worker_identity() {
    let dir = std::env::temp_dir().join(format!("memd-hive-identity-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

    store
        .upsert_hive_session(&HiveSessionUpsertRequest {
            session: "session-live-openclaw".to_string(),
            agent: Some("openclaw".to_string()),
            effective_agent: Some("openclaw@session-live-openclaw".to_string()),
            hive_system: Some("openclaw".to_string()),
            hive_role: Some("agent".to_string()),
            worker_name: Some("openclaw".to_string()),
            display_name: None,
            role: Some("agent".to_string()),
            capabilities: vec!["coordination".to_string(), "memory".to_string()],
            hive_groups: vec!["project:memd".to_string()],
            lane_id: None,
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: None,
            tab_id: Some("tab-alpha".to_string()),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            base_branch: None,
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some("http://100.104.154.24:8787".to_string()),
            base_url_healthy: Some(true),
            host: None,
            pid: Some(123),
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
        .expect("insert older generic identity row");

    store
        .upsert_hive_session(&HiveSessionUpsertRequest {
            session: "session-live-openclaw".to_string(),
            agent: Some("openclaw".to_string()),
            effective_agent: Some("openclaw@session-live-openclaw".to_string()),
            hive_system: Some("openclaw".to_string()),
            hive_role: Some("agent".to_string()),
            worker_name: Some("Openclaw".to_string()),
            display_name: Some("Openclaw".to_string()),
            role: Some("agent".to_string()),
            capabilities: vec!["coordination".to_string(), "memory".to_string()],
            hive_groups: vec!["project:memd".to_string()],
            lane_id: None,
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: None,
            tab_id: Some("tab-alpha".to_string()),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            base_branch: None,
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some("http://100.104.154.24:8787".to_string()),
            base_url_healthy: Some(true),
            host: None,
            pid: Some(456),
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
        .expect("insert newer human identity row");

    let response = store
        .hive_sessions(&HiveSessionsRequest {
            session: Some("session-live-openclaw".to_string()),
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
            limit: Some(16),
        })
        .expect("query merged session");

    assert_eq!(response.sessions.len(), 1);
    let session = &response.sessions[0];
    assert_eq!(session.worker_name.as_deref(), Some("Openclaw"));
    assert_eq!(session.display_name.as_deref(), Some("Openclaw"));

    std::fs::remove_dir_all(dir).expect("cleanup temp dir");
}

#[test]
fn hive_sessions_collapse_does_not_backfill_generic_display_for_named_worker() {
    let dir = std::env::temp_dir().join(format!("memd-hive-display-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

    store
        .upsert_hive_session(&HiveSessionUpsertRequest {
            session: "session-live-openclaw".to_string(),
            agent: Some("codex".to_string()),
            effective_agent: Some("codex@session-live-openclaw".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("agent".to_string()),
            worker_name: Some("codex".to_string()),
            display_name: Some("Codex live-openclaw".to_string()),
            role: Some("agent".to_string()),
            capabilities: vec!["coordination".to_string(), "memory".to_string()],
            hive_groups: vec!["project:memd".to_string()],
            lane_id: None,
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: None,
            tab_id: Some("tab-alpha".to_string()),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            base_branch: None,
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some("http://100.104.154.24:8787".to_string()),
            base_url_healthy: Some(true),
            host: None,
            pid: Some(123),
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
        .expect("insert older generic row");

    store
        .upsert_hive_session(&HiveSessionUpsertRequest {
            session: "session-live-openclaw".to_string(),
            agent: Some("openclaw".to_string()),
            effective_agent: Some("openclaw@session-live-openclaw".to_string()),
            hive_system: Some("openclaw".to_string()),
            hive_role: Some("agent".to_string()),
            worker_name: Some("Openclaw".to_string()),
            display_name: None,
            role: Some("agent".to_string()),
            capabilities: vec!["coordination".to_string(), "memory".to_string()],
            hive_groups: vec!["project:memd".to_string()],
            lane_id: None,
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: None,
            tab_id: Some("tab-alpha".to_string()),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            base_branch: None,
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some("http://100.104.154.24:8787".to_string()),
            base_url_healthy: Some(true),
            host: None,
            pid: Some(456),
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
        .expect("insert newer named row");

    let response = store
        .hive_sessions(&HiveSessionsRequest {
            session: Some("session-live-openclaw".to_string()),
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
            limit: Some(16),
        })
        .expect("query merged session");

    assert_eq!(response.sessions.len(), 1);
    let session = &response.sessions[0];
    assert_eq!(session.worker_name.as_deref(), Some("Openclaw"));
    assert_eq!(session.display_name.as_deref(), None);

    std::fs::remove_dir_all(dir).expect("cleanup temp dir");
}
