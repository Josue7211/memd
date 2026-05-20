use super::*;

#[test]
fn awareness_merge_prefers_fresher_local_session_metadata_over_stale_remote_row() {
    let entries = merge_project_awareness_entries(
        vec![ProjectAwarenessEntry {
            project_dir: "/tmp/projects/current".to_string(),
            bundle_root: "/tmp/projects/current/.memd".to_string(),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            base_branch: None,
            agent: Some("codex".to_string()),
            session: Some("session-2c2c883c".to_string()),
            tab_id: Some("tab-alpha".to_string()),
            effective_agent: Some("codex@session-2c2c883c".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("agent".to_string()),
            capabilities: vec!["memory".to_string(), "coordination".to_string()],
            hive_groups: vec!["project:memd".to_string()],
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            presence: "active".to_string(),
            host: None,
            pid: None,
            active_claims: 1,
            workspace: Some("memd".to_string()),
            visibility: Some("workspace".to_string()),
            topic_claim: None,
            scope_claims: Vec::new(),
            task_id: None,
            focus: Some("Ship the hive fix".to_string()),
            pressure: Some("Repair awareness".to_string()),
            next_recovery: None,
            last_updated: Some(Utc::now()),
        }],
        vec![ProjectAwarenessEntry {
            project_dir: "remote".to_string(),
            bundle_root: "remote:http://127.0.0.1:8787:session-2c2c883c".to_string(),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            base_branch: None,
            agent: Some("codex".to_string()),
            session: Some("session-2c2c883c".to_string()),
            tab_id: Some("tab-alpha".to_string()),
            effective_agent: Some("codex@session-2c2c883c".to_string()),
            hive_system: None,
            hive_role: None,
            capabilities: vec!["memory".to_string()],
            hive_groups: Vec::new(),
            hive_group_goal: None,
            authority: None,
            base_url: Some("http://127.0.0.1:8787".to_string()),
            presence: "stale".to_string(),
            host: None,
            pid: None,
            active_claims: 0,
            workspace: Some("memd".to_string()),
            visibility: Some("workspace".to_string()),
            topic_claim: None,
            scope_claims: Vec::new(),
            task_id: None,
            focus: None,
            pressure: None,
            next_recovery: None,
            last_updated: Some(Utc::now() - chrono::TimeDelta::minutes(10)),
        }],
    );

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].project_dir, "/tmp/projects/current");
    assert_eq!(entries[0].presence, "active");
    assert_eq!(entries[0].hive_system.as_deref(), Some("codex"));
    assert_eq!(entries[0].hive_groups, vec!["project:memd".to_string()]);
}

#[test]
fn awareness_merge_keeps_distinct_sessions_when_remote_rows_are_not_duplicates() {
    let entries = merge_project_awareness_entries(
        vec![ProjectAwarenessEntry {
            project_dir: "/tmp/projects/current".to_string(),
            bundle_root: "/tmp/projects/current/.memd".to_string(),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            base_branch: None,
            agent: Some("codex".to_string()),
            session: Some("session-2c2c883c".to_string()),
            tab_id: Some("tab-alpha".to_string()),
            effective_agent: Some("codex@session-2c2c883c".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("agent".to_string()),
            capabilities: vec!["memory".to_string(), "coordination".to_string()],
            hive_groups: vec!["project:memd".to_string()],
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            presence: "active".to_string(),
            host: None,
            pid: None,
            active_claims: 1,
            workspace: Some("memd".to_string()),
            visibility: Some("workspace".to_string()),
            topic_claim: None,
            scope_claims: Vec::new(),
            task_id: None,
            focus: None,
            pressure: None,
            next_recovery: None,
            last_updated: Some(Utc::now()),
        }],
        vec![ProjectAwarenessEntry {
            project_dir: "remote".to_string(),
            bundle_root: "remote:http://127.0.0.1:8787:session-6d422e56".to_string(),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            base_branch: None,
            agent: Some("codex".to_string()),
            session: Some("session-6d422e56".to_string()),
            tab_id: Some("tab-beta".to_string()),
            effective_agent: Some("codex@session-6d422e56".to_string()),
            hive_system: None,
            hive_role: None,
            capabilities: vec!["memory".to_string()],
            hive_groups: Vec::new(),
            hive_group_goal: None,
            authority: None,
            base_url: Some("http://127.0.0.1:8787".to_string()),
            presence: "active".to_string(),
            host: None,
            pid: None,
            active_claims: 0,
            workspace: Some("memd".to_string()),
            visibility: Some("workspace".to_string()),
            topic_claim: None,
            scope_claims: Vec::new(),
            task_id: None,
            focus: None,
            pressure: None,
            next_recovery: None,
            last_updated: Some(Utc::now()),
        }],
    );

    assert_eq!(entries.len(), 2);
    assert!(
        entries
            .iter()
            .any(|entry| entry.session.as_deref() == Some("session-2c2c883c"))
    );
    assert!(
        entries
            .iter()
            .any(|entry| entry.session.as_deref() == Some("session-6d422e56"))
    );
}

#[test]
fn session_collision_warnings_surface_shared_session_reuse() {
    let warnings = session_collision_warnings(&[
        ProjectAwarenessEntry {
            project_dir: "/tmp/projects/a".to_string(),
            bundle_root: "/tmp/projects/a/.memd".to_string(),
            project: Some("a".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            base_branch: None,
            agent: Some("codex".to_string()),
            session: Some("shared-session".to_string()),
            tab_id: Some("tab-a".to_string()),
            effective_agent: Some("codex@shared-session".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("agent".to_string()),
            capabilities: vec!["memory".to_string(), "coordination".to_string()],
            hive_groups: vec!["openclaw-stack".to_string()],
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            presence: "active".to_string(),
            host: None,
            pid: None,
            active_claims: 0,
            workspace: Some("initiative-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            topic_claim: None,
            scope_claims: Vec::new(),
            task_id: None,
            focus: None,
            pressure: None,
            next_recovery: None,
            last_updated: None,
        },
        ProjectAwarenessEntry {
            project_dir: "/tmp/projects/b".to_string(),
            bundle_root: "/tmp/projects/b/.memd".to_string(),
            project: Some("b".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            base_branch: None,
            agent: Some("claude-code".to_string()),
            session: Some("shared-session".to_string()),
            tab_id: Some("tab-a".to_string()),
            effective_agent: Some("claude-code@shared-session".to_string()),
            hive_system: Some("claude-code".to_string()),
            hive_role: Some("agent".to_string()),
            capabilities: vec!["memory".to_string(), "coordination".to_string()],
            hive_groups: vec!["openclaw-stack".to_string()],
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            base_url: Some("http://127.0.0.1:9797".to_string()),
            presence: "active".to_string(),
            host: None,
            pid: None,
            active_claims: 0,
            workspace: Some("initiative-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            topic_claim: None,
            scope_claims: Vec::new(),
            task_id: None,
            focus: None,
            pressure: None,
            next_recovery: None,
            last_updated: None,
        },
    ]);

    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("session shared-session tab tab-a in workspace initiative-alpha"));
    assert!(warnings[0].contains("2 bundles"));
    assert!(warnings[0].contains("2 agents"));
    assert!(warnings[0].contains("2 endpoints"));
}

#[test]
fn branch_collision_warnings_surface_same_branch_and_worktree_faults() {
    let warnings = branch_collision_warnings(&[
        ProjectAwarenessEntry {
            project_dir: "/tmp/projects/a".to_string(),
            bundle_root: "/tmp/projects/a/.memd".to_string(),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            repo_root: Some("/tmp/repo".to_string()),
            worktree_root: Some("/tmp/repo".to_string()),
            branch: Some("feature/hive-a".to_string()),
            base_branch: Some("main".to_string()),
            agent: Some("codex".to_string()),
            session: Some("session-a".to_string()),
            tab_id: Some("tab-a".to_string()),
            effective_agent: Some("codex@session-a".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("agent".to_string()),
            capabilities: vec!["memory".to_string()],
            hive_groups: vec!["project:demo".to_string()],
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            presence: "active".to_string(),
            host: None,
            pid: None,
            active_claims: 0,
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            topic_claim: None,
            scope_claims: Vec::new(),
            task_id: None,
            focus: None,
            pressure: None,
            next_recovery: None,
            last_updated: None,
        },
        ProjectAwarenessEntry {
            project_dir: "/tmp/projects/b".to_string(),
            bundle_root: "/tmp/projects/b/.memd".to_string(),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            repo_root: Some("/tmp/repo".to_string()),
            worktree_root: Some("/tmp/repo".to_string()),
            branch: Some("feature/hive-a".to_string()),
            base_branch: Some("main".to_string()),
            agent: Some("claude-code".to_string()),
            session: Some("session-b".to_string()),
            tab_id: Some("tab-b".to_string()),
            effective_agent: Some("claude-code@session-b".to_string()),
            hive_system: Some("claude-code".to_string()),
            hive_role: Some("agent".to_string()),
            capabilities: vec!["memory".to_string()],
            hive_groups: vec!["project:demo".to_string()],
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            base_url: Some("http://127.0.0.1:9797".to_string()),
            presence: "active".to_string(),
            host: None,
            pid: None,
            active_claims: 0,
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            topic_claim: None,
            scope_claims: Vec::new(),
            task_id: None,
            focus: None,
            pressure: None,
            next_recovery: None,
            last_updated: None,
        },
    ]);

    assert!(warnings.iter().any(|value| {
        value.contains("unsafe_same_branch repo=/tmp/repo branch=feature/hive-a")
    }));
    assert!(
        warnings
            .iter()
            .any(|value| value.contains("unsafe_same_worktree worktree=/tmp/repo"))
    );
}

#[test]
fn heartbeat_presence_labels_age_bands() {
    assert_eq!(heartbeat_presence_label(Utc::now()), "active");
    assert_eq!(
        heartbeat_presence_label(Utc::now() - chrono::TimeDelta::minutes(5)),
        "stale"
    );
    assert_eq!(
        heartbeat_presence_label(Utc::now() - chrono::TimeDelta::minutes(30)),
        "dead"
    );
}

#[test]
fn render_bundle_heartbeat_summary_surfaces_presence_and_focus() {
    let state = BundleHeartbeatState {
        session: Some("codex-a".to_string()),
        agent: Some("codex".to_string()),
        effective_agent: Some("codex@codex-a".to_string()),
        tab_id: None,
        hive_system: Some("codex".to_string()),
        hive_role: Some("agent".to_string()),
        worker_name: Some("codex".to_string()),
        display_name: None,
        role: Some("agent".to_string()),
        capabilities: vec!["memory".to_string(), "coordination".to_string()],
        hive_groups: vec!["openclaw-stack".to_string()],
        lane_id: Some("/tmp/demo".to_string()),
        hive_group_goal: None,
        authority: Some("participant".to_string()),
        heartbeat_model: Some(default_heartbeat_model()),
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        workspace: Some("team-alpha".to_string()),
        repo_root: Some("/tmp/demo".to_string()),
        worktree_root: Some("/tmp/demo".to_string()),
        branch: Some("feature/test-bee".to_string()),
        base_branch: Some("main".to_string()),
        visibility: Some("workspace".to_string()),
        base_url: Some("http://127.0.0.1:8787".to_string()),
        base_url_healthy: Some(true),
        host: Some("workstation".to_string()),
        pid: Some(4242),
        topic_claim: None,
        scope_claims: Vec::new(),
        task_id: None,
        focus: Some("Finish the live heartbeat lane".to_string()),
        pressure: Some("Avoid memory drift".to_string()),
        next_recovery: None,
        next_action: None,
        needs_help: false,
        needs_review: false,
        handoff_state: None,
        confidence: None,
        risk: None,
        status: "live".to_string(),
        last_seen: Utc::now(),
        authority_mode: Some("shared".to_string()),
        authority_degraded: false,
        ..BundleHeartbeatState::default()
    };

    let summary = render_bundle_heartbeat_summary(&state);
    assert!(summary.contains("heartbeat project=demo"));
    assert!(summary.contains("agent=codex@codex-a"));
    assert!(summary.contains("session=codex-a"));
    assert!(summary.contains("presence=active"));
    assert!(summary.contains("focus=\"Finish the live heartbeat lane\""));
    assert!(summary.contains("pressure=\"Avoid memory drift\""));
}

#[test]
fn render_hive_wire_summary_marks_rebased_live_session() {
    let summary = render_hive_wire_summary(&HiveWireResponse {
        action: "updated".to_string(),
        output: ".memd".to_string(),
        project_root: Some("/tmp/demo".to_string()),
        agent: "codex".to_string(),
        bundle_session: Some("codex-stale".to_string()),
        live_session: Some("codex-fresh".to_string()),
        rebased_from_session: Some("codex-stale".to_string()),
        session: Some("codex-fresh".to_string()),
        tab_id: Some("tab-alpha".to_string()),
        hive_system: Some("codex".to_string()),
        hive_role: Some("agent".to_string()),
        hive_groups: vec!["project:memd".to_string()],
        hive_group_goal: None,
        authority: Some("participant".to_string()),
        heartbeat: None,
        lane_rerouted: false,
        lane_created: false,
        lane_surface: None,
    });

    assert!(summary.contains("bundle_session=codex-stale"));
    assert!(summary.contains("live_session=codex-fresh"));
    assert!(summary.contains("session=codex-fresh"));
    assert!(summary.contains("rebased_from=codex-stale"));
}

#[test]
fn shared_awareness_scope_prefers_workspace_over_project_filters() {
    let runtime = BundleRuntimeConfig {
        project: Some("repo-a".to_string()),
        namespace: Some("main".to_string()),
        agent: None,
        session: None,
        tab_id: None,
        hive_system: None,
        hive_role: None,
        capabilities: Vec::new(),
        hive_groups: Vec::new(),
        hive_group_goal: None,
        authority: None,
        hive_project_enabled: false,
        hive_project_anchor: None,
        hive_project_joined_at: None,
        base_url: None,
        route: None,
        intent: None,
        voice_mode: None,
        workspace: Some("initiative-alpha".to_string()),
        visibility: None,
        heartbeat_model: None,
        auto_short_term_capture: true,
        auto_commit: BundleAutoCommitConfig::default(),
        authority_policy: BundleAuthorityPolicy::default(),
        authority_state: BundleAuthorityState::default(),
    };

    let (project, namespace) = shared_awareness_scope(Some(&runtime));
    assert!(project.is_none());
    assert!(namespace.is_none());

    let runtime = BundleRuntimeConfig {
        workspace: None,
        ..runtime
    };
    let (project, namespace) = shared_awareness_scope(Some(&runtime));
    assert_eq!(project.as_deref(), Some("repo-a"));
    assert_eq!(namespace.as_deref(), Some("main"));
}

#[test]
fn confirmed_hive_overlap_reason_detects_scope_and_topic_conflicts() {
    let target = ProjectAwarenessEntry {
        project_dir: "remote".to_string(),
        bundle_root: "remote:http://127.0.0.1:8787:active".to_string(),
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        repo_root: None,
        worktree_root: Some("/tmp/peer".to_string()),
        branch: Some("feature/peer".to_string()),
        base_branch: Some("main".to_string()),
        agent: Some("claude-code".to_string()),
        session: Some("active".to_string()),
        tab_id: None,
        effective_agent: Some("claude-code@active".to_string()),
        hive_system: Some("claude-code".to_string()),
        hive_role: Some("agent".to_string()),
        capabilities: vec!["memory".to_string()],
        hive_groups: vec!["project:demo".to_string()],
        hive_group_goal: None,
        authority: Some("participant".to_string()),
        base_url: Some("http://127.0.0.1:8787".to_string()),
        presence: "active".to_string(),
        host: Some("workstation".to_string()),
        pid: Some(2),
        active_claims: 1,
        workspace: Some("shared".to_string()),
        visibility: Some("workspace".to_string()),
        topic_claim: Some("Refine parser overlap flow".to_string()),
        scope_claims: vec![
            "task:parser-refactor".to_string(),
            "crates/memd-client/src/main.rs".to_string(),
        ],
        task_id: Some("parser-refactor".to_string()),
        focus: None,
        pressure: None,
        next_recovery: None,
        last_updated: Some(Utc::now()),
    };

    let scope_conflict = confirmed_hive_overlap_reason(
        &target,
        Some("queen-refactor"),
        Some("Different task"),
        &["crates/memd-client/src/main.rs".to_string()],
    )
    .expect("scope conflict");
    assert!(scope_conflict.contains("already owns scope"));

    let topic_conflict = confirmed_hive_overlap_reason(
        &ProjectAwarenessEntry {
            scope_claims: Vec::new(),
            task_id: None,
            ..target
        },
        None,
        Some("Refine parser overlap flow"),
        &[],
    )
    .expect("topic conflict");
    assert!(topic_conflict.contains("already owns topic"));
}

#[test]
fn confirmed_hive_overlap_reason_ignores_generic_project_scope() {
    let target = ProjectAwarenessEntry {
        project_dir: "remote".to_string(),
        bundle_root: "remote:http://127.0.0.1:8787:peer".to_string(),
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        repo_root: None,
        worktree_root: Some("/tmp/peer".to_string()),
        branch: Some("feature/peer".to_string()),
        base_branch: Some("main".to_string()),
        agent: Some("codex".to_string()),
        session: Some("peer".to_string()),
        tab_id: None,
        effective_agent: Some("Peer@peer".to_string()),
        hive_system: Some("codex".to_string()),
        hive_role: Some("worker".to_string()),
        capabilities: vec!["coordination".to_string()],
        hive_groups: vec!["project:demo".to_string()],
        hive_group_goal: None,
        authority: Some("participant".to_string()),
        base_url: Some("http://127.0.0.1:8787".to_string()),
        presence: "active".to_string(),
        host: Some("workstation".to_string()),
        pid: Some(3),
        active_claims: 1,
        workspace: Some("shared".to_string()),
        visibility: Some("workspace".to_string()),
        topic_claim: Some("Review parser handoff".to_string()),
        scope_claims: vec!["project".to_string()],
        task_id: Some("peer-review".to_string()),
        focus: None,
        pressure: None,
        next_recovery: None,
        last_updated: Some(Utc::now()),
    };

    assert!(
        confirmed_hive_overlap_reason(
            &target,
            Some("current-task"),
            Some("Different topic"),
            &["project".to_string()],
        )
        .is_none()
    );
}

#[tokio::test]
async fn enrich_hive_heartbeat_with_runtime_intent_prefers_owned_task_state() {
    let state = MockRuntimeState::default();
    {
        let mut tasks = state.task_records.lock().expect("lock task records");
        tasks.push(HiveTaskRecord {
            task_id: "parser-refactor".to_string(),
            title: "Refine parser overlap flow".to_string(),
            description: None,
            status: "in_progress".to_string(),
            coordination_mode: CoordinationMode::ExclusiveWrite,
            session: Some("codex-a".to_string()),
            agent: Some("codex".to_string()),
            effective_agent: Some("codex@codex-a".to_string()),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            claim_scopes: vec![
                "task:parser-refactor".to_string(),
                "crates/memd-client/src/main.rs".to_string(),
            ],
            help_requested: false,
            review_requested: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });
    }
    let base_url = spawn_mock_runtime_server(state, false).await;
    let mut heartbeat = test_hive_heartbeat_state("codex-a", "codex", "tab-a", "live", Utc::now());
    heartbeat.base_url = Some(base_url);
    heartbeat.project = Some("demo".to_string());
    heartbeat.namespace = Some("main".to_string());
    heartbeat.workspace = Some("shared".to_string());
    heartbeat.session = Some("codex-a".to_string());
    heartbeat.topic_claim = Some("editing fallback".to_string());

    enrich_hive_heartbeat_with_runtime_intent(&mut heartbeat)
        .await
        .expect("enrich heartbeat");

    assert_eq!(heartbeat.task_id.as_deref(), Some("parser-refactor"));
    assert_eq!(
        heartbeat.topic_claim.as_deref(),
        Some("Refine parser overlap flow")
    );
    assert_eq!(heartbeat.display_name.as_deref(), Some("Codex a"));
    assert!(
        heartbeat
            .scope_claims
            .iter()
            .any(|scope| scope == "task:parser-refactor")
    );
}

#[tokio::test]
async fn enrich_hive_heartbeat_with_runtime_intent_overrides_workspace_topic_placeholder() {
    let state = MockRuntimeState::default();
    {
        let mut tasks = state.task_records.lock().expect("lock task records");
        tasks.push(HiveTaskRecord {
            task_id: "remote-proof-refactor".to_string(),
            title: "Remote proof overlap flow".to_string(),
            description: None,
            status: "in_progress".to_string(),
            coordination_mode: CoordinationMode::ExclusiveWrite,
            session: Some("codex-a".to_string()),
            agent: Some("codex".to_string()),
            effective_agent: Some("codex@codex-a".to_string()),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            claim_scopes: vec![
                "task:remote-proof-refactor".to_string(),
                "crates/memd-client/src/main.rs".to_string(),
            ],
            help_requested: false,
            review_requested: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });
    }
    let base_url = spawn_mock_runtime_server(state, false).await;
    let mut heartbeat = test_hive_heartbeat_state("codex-a", "codex", "tab-a", "live", Utc::now());
    heartbeat.base_url = Some(base_url);
    heartbeat.project = Some("demo".to_string());
    heartbeat.namespace = Some("main".to_string());
    heartbeat.workspace = Some("shared".to_string());
    heartbeat.session = Some("codex-a".to_string());
    heartbeat.topic_claim = Some("ws=shared".to_string());

    enrich_hive_heartbeat_with_runtime_intent(&mut heartbeat)
        .await
        .expect("enrich heartbeat");

    assert_eq!(heartbeat.task_id.as_deref(), Some("remote-proof-refactor"));
    assert_eq!(
        heartbeat.topic_claim.as_deref(),
        Some("Remote proof overlap flow")
    );
    assert_eq!(heartbeat.display_name.as_deref(), Some("Codex a"));
}

#[tokio::test]
async fn propagate_hive_metadata_does_not_overwrite_sibling_worker_identity() {
    let _env_lock = lock_env_mutation();
    let root = std::env::temp_dir().join(format!("memd-hive-propagate-{}", uuid::Uuid::new_v4()));
    let current_project = root.join("current");
    let sibling_project = root.join("sibling");
    fs::create_dir_all(&current_project).expect("create current project");
    fs::create_dir_all(&sibling_project).expect("create sibling project");

    let current_bundle = current_project.join(".memd");
    let sibling_bundle = sibling_project.join(".memd");
    let base_url = "http://127.0.0.1:9".to_string();

    write_init_bundle(&InitArgs {
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        global: false,
        project_root: Some(current_project.clone()),
        seed_existing: false,
        agent: "openclaw".to_string(),
        session: Some("session-live-openclaw".to_string()),
        tab_id: None,
        hive_system: Some("openclaw".to_string()),
        hive_role: Some("agent".to_string()),
        capability: vec!["coordination".to_string(), "memory".to_string()],
        hive_group: vec!["project:demo".to_string()],
        hive_group_goal: None,
        authority: Some("participant".to_string()),
        output: current_bundle.clone(),
        base_url: base_url.clone(),
        rag_url: None,
        route: "auto".to_string(),
        intent: "current_task".to_string(),
        voice_mode: None,
        workspace: Some("shared".to_string()),
        visibility: Some("workspace".to_string()),
        allow_localhost_read_only_fallback: false,
        force: true,
    })
    .expect("write current bundle");
    set_bundle_hive_project_state(
        &current_bundle,
        true,
        Some("project:demo"),
        Some(Utc::now()),
    )
    .expect("enable current project hive");

    write_init_bundle(&InitArgs {
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        global: false,
        project_root: Some(sibling_project.clone()),
        seed_existing: false,
        agent: "hermes".to_string(),
        session: Some("session-live-hermes".to_string()),
        tab_id: None,
        hive_system: Some("hermes".to_string()),
        hive_role: Some("agent".to_string()),
        capability: vec!["coordination".to_string(), "memory".to_string()],
        hive_group: vec!["project:demo".to_string()],
        hive_group_goal: None,
        authority: Some("participant".to_string()),
        output: sibling_bundle.clone(),
        base_url: base_url.clone(),
        rag_url: None,
        route: "auto".to_string(),
        intent: "current_task".to_string(),
        voice_mode: None,
        workspace: Some("shared".to_string()),
        visibility: Some("workspace".to_string()),
        allow_localhost_read_only_fallback: false,
        force: true,
    })
    .expect("write sibling bundle");
    set_bundle_hive_project_state(
        &sibling_bundle,
        true,
        Some("project:demo"),
        Some(Utc::now()),
    )
    .expect("enable sibling project hive");

    unsafe {
        std::env::set_var("MEMD_WORKER_NAME", "Hermes");
    }
    refresh_bundle_heartbeat(&sibling_bundle, None, false)
        .await
        .expect("refresh sibling heartbeat");

    unsafe {
        std::env::set_var("MEMD_WORKER_NAME", "Openclaw");
    }
    refresh_bundle_heartbeat(&current_bundle, None, false)
        .await
        .expect("refresh current heartbeat");

    let runtime = read_bundle_runtime_config(&current_bundle)
        .expect("read current runtime")
        .expect("current runtime present");

    propagate_hive_metadata_to_active_project_bundles(&current_bundle, &runtime, true)
        .await
        .expect("propagate hive metadata");

    let sibling_heartbeat =
        read_bundle_heartbeat(&sibling_bundle).expect("read sibling heartbeat file");
    let sibling_heartbeat = sibling_heartbeat.expect("sibling heartbeat present");
    assert_eq!(sibling_heartbeat.agent.as_deref(), Some("hermes"));
    assert_eq!(sibling_heartbeat.worker_name.as_deref(), Some("Hermes"));

    unsafe {
        std::env::remove_var("MEMD_WORKER_NAME");
    }
    fs::remove_dir_all(root).expect("cleanup propagation test root");
}
