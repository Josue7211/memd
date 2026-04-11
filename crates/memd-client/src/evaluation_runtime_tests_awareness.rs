    use super::*;

    fn project_awareness_scans_sibling_bundles_without_current() {
        let root =
            std::env::temp_dir().join(format!("memd-awareness-root-{}", uuid::Uuid::new_v4()));
        let current_project = root.join("current");
        let sibling_project = root.join("sibling");
        fs::create_dir_all(current_project.join(".memd").join("state")).expect("create current");
        fs::create_dir_all(sibling_project.join(".memd").join("state")).expect("create sibling");

        fs::write(
            current_project.join(".memd").join("config.json"),
            r#"{
  "project": "current",
  "namespace": "main",
  "agent": "codex",
  "workspace": "current-lane",
  "visibility": "workspace"
}
"#,
        )
        .expect("write current config");
        fs::write(
            sibling_project.join(".memd").join("config.json"),
            r#"{
  "project": "sibling",
  "namespace": "main",
  "agent": "claude-code",
  "workspace": "research",
  "visibility": "workspace"
}
"#,
        )
        .expect("write sibling config");
        fs::write(
            sibling_project
                .join(".memd")
                .join("state")
                .join("last-resume.json"),
            r#"{
  "focus": "Finish the sibling task",
  "pressure": "Resolve review comments",
  "next_recovery": "Re-open the last handoff",
  "lane": "sibling / main / research",
  "working_records": 2,
  "inbox_items": 1,
  "rehydration_items": 1
}
"#,
        )
        .expect("write sibling state");

        let response = read_project_awareness_local(&AwarenessArgs {
            output: current_project.join(".memd"),
            root: Some(root.clone()),
            include_current: false,
            summary: false,
        })
        .expect("read awareness");

        assert_eq!(response.entries.len(), 1);
        let entry = &response.entries[0];
        assert_eq!(entry.project.as_deref(), Some("sibling"));
        assert_eq!(entry.agent.as_deref(), Some("claude-code"));
        assert_eq!(entry.workspace.as_deref(), Some("research"));
        assert_eq!(entry.focus.as_deref(), Some("Finish the sibling task"));
        assert_eq!(entry.pressure.as_deref(), Some("Resolve review comments"));

        fs::remove_dir_all(root).expect("cleanup awareness root");
    }

    #[tokio::test]
    async fn project_awareness_includes_current_bundle_when_session_exists() {
        let root =
            std::env::temp_dir().join(format!("memd-awareness-live-{}", uuid::Uuid::new_v4()));
        let current_project = root.join("current");
        let sibling_project = root.join("sibling");
        fs::create_dir_all(current_project.join(".memd").join("state")).expect("create current");
        fs::create_dir_all(sibling_project.join(".memd").join("state")).expect("create sibling");

        fs::write(
            current_project.join(".memd").join("config.json"),
            r#"{
  "project": "current",
  "namespace": "main",
  "agent": "codex",
  "session": "current-a",
  "workspace": "current-lane",
  "visibility": "workspace"
}
"#,
        )
        .expect("write current config");
        fs::write(
            sibling_project.join(".memd").join("config.json"),
            r#"{
  "project": "sibling",
  "namespace": "main",
  "agent": "claude-code",
  "workspace": "research",
  "visibility": "workspace"
}
"#,
        )
        .expect("write sibling config");
        fs::write(
            sibling_project
                .join(".memd")
                .join("state")
                .join("last-resume.json"),
            r#"{
  "focus": "Finish the sibling task",
  "pressure": "Resolve review comments",
  "next_recovery": "Re-open the last handoff",
  "lane": "sibling / main / research",
  "working_records": 2,
  "inbox_items": 1,
  "rehydration_items": 1
}
"#,
        )
        .expect("write sibling state");

        let response = read_project_awareness(&AwarenessArgs {
            output: current_project.join(".memd"),
            root: Some(root.clone()),
            include_current: false,
            summary: false,
        })
        .await
        .expect("read awareness");

        assert_eq!(response.entries.len(), 2);
        assert!(response.entries.iter().any(|entry| {
            entry.project.as_deref() == Some("current")
                && entry.session.as_deref() == Some("current-a")
        }));
        assert!(response.entries.iter().any(|entry| {
            entry.project.as_deref() == Some("sibling")
                && entry.agent.as_deref() == Some("claude-code")
        }));

        fs::remove_dir_all(root).expect("cleanup awareness root");
    }

    #[test]
    fn project_awareness_local_prunes_dead_sibling_without_active_claims() {
        let root =
            std::env::temp_dir().join(format!("memd-awareness-prune-{}", uuid::Uuid::new_v4()));
        let current_project = root.join("current");
        let sibling_project = root.join("sibling");
        let current_bundle = current_project.join(".memd");
        let sibling_bundle = sibling_project.join(".memd");
        fs::create_dir_all(current_bundle.join("state")).expect("create current");
        fs::create_dir_all(sibling_bundle.join("state")).expect("create sibling");

        fs::write(
            current_bundle.join("config.json"),
            r#"{
  "project": "current",
  "namespace": "main",
  "agent": "codex",
  "session": "current-a",
  "workspace": "shared",
  "visibility": "workspace"
}
"#,
        )
        .expect("write current config");
        fs::write(
            sibling_bundle.join("config.json"),
            r#"{
  "project": "sibling",
  "namespace": "main",
  "agent": "claude-code",
  "session": "sibling-dead",
  "workspace": "shared",
  "visibility": "workspace"
}
"#,
        )
        .expect("write sibling config");
        fs::write(
            bundle_heartbeat_state_path(&sibling_bundle),
            serde_json::to_string_pretty(&BundleHeartbeatState {
                session: Some("sibling-dead".to_string()),
                agent: Some("claude-code".to_string()),
                effective_agent: Some("claude-code@sibling-dead".to_string()),
                tab_id: None,
                hive_system: Some("claude-code".to_string()),
                hive_role: Some("agent".to_string()),
                lane_id: None,
                worker_name: Some("claude-code".to_string()),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["project:sibling".to_string()],
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some(default_heartbeat_model()),
                project: Some("sibling".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: Some(sibling_project.display().to_string()),
                branch: Some("feature/old".to_string()),
                base_branch: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some(SHARED_MEMD_BASE_URL.to_string()),
                base_url_healthy: Some(true),
                host: None,
                pid: None,
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
                status: "live".to_string(),
                last_seen: Utc::now() - chrono::TimeDelta::minutes(30),
                authority_mode: Some("shared".to_string()),
                authority_degraded: false,
            })
            .expect("serialize heartbeat")
                + "\n",
        )
        .expect("write sibling heartbeat");

        let response = read_project_awareness_local(&AwarenessArgs {
            output: current_bundle.clone(),
            root: Some(root.clone()),
            include_current: true,
            summary: false,
        })
        .expect("read awareness");

        assert!(
            response
                .entries
                .iter()
                .all(|entry| { entry.session.as_deref() != Some("sibling-dead") })
        );
        assert!(
            !bundle_heartbeat_state_path(&sibling_bundle).exists(),
            "dead sibling heartbeat should be pruned automatically"
        );

        fs::remove_dir_all(root).expect("cleanup awareness root");
    }

    #[test]
    fn project_awareness_summary_compacts_focus_and_pressure() {
        let now = Utc::now();
        let response = ProjectAwarenessResponse {
            root: "/tmp/projects".to_string(),
            current_bundle: "/tmp/projects/current/.memd".to_string(),
            collisions: Vec::new(),
            entries: vec![ProjectAwarenessEntry {
                project_dir: "/tmp/projects/sibling".to_string(),
                bundle_root: "/tmp/projects/sibling/.memd".to_string(),
                project: Some("sibling".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                agent: Some("claude-code".to_string()),
                session: Some("claude-a".to_string()),
                tab_id: Some("tab-a".to_string()),
                effective_agent: Some("claude-code@claude-a".to_string()),
                hive_system: Some("claude-code".to_string()),
                hive_role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string(), "coordination".to_string()],
                hive_groups: vec!["openclaw-stack".to_string()],
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                base_url: None,
                presence: "active".to_string(),
                host: None,
                pid: None,
                active_claims: 0,
                workspace: Some("research".to_string()),
                visibility: Some("workspace".to_string()),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: Some("Investigate whether the recall lane is still stale".to_string()),
                pressure: Some("Repair the shared lane before the next resume".to_string()),
                next_recovery: None,
                last_updated: Some(now),
            }],
        };

        let summary = render_project_awareness_summary(&response);
        assert!(
            summary.contains(
                "awareness root=/tmp/projects bundles=1 diagnostics=0 hidden_remote_dead=0 hidden_superseded_stale=0"
            )
        );
        assert!(summary.contains("active_hive_sessions:"));
        assert!(summary.contains("sibling [hive-session] | presence=active truth=fresh"));
        assert!(summary.contains("focus=\"Investigate whether the recall lane is still stale\""));
        assert!(summary.contains("pressure=\"Repair the shared lane before the next resume\""));
        assert!(summary.contains("tab=tab-a"));
    }

    #[test]
    fn project_awareness_surfaces_base_url_collisions() {
        let response = ProjectAwarenessResponse {
            root: "/tmp/projects".to_string(),
            current_bundle: "/tmp/projects/current/.memd".to_string(),
            collisions: vec!["base_url http://127.0.0.1:8787 used by 2 bundles".to_string()],
            entries: vec![
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
                    session: Some("codex-a".to_string()),
                    tab_id: Some("tab-a".to_string()),
                    effective_agent: Some("codex@codex-a".to_string()),
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
                    active_claims: 1,
                    workspace: Some("a".to_string()),
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
                    session: Some("claude-b".to_string()),
                    tab_id: Some("tab-b".to_string()),
                    effective_agent: Some("claude-code@claude-b".to_string()),
                    hive_system: Some("claude-code".to_string()),
                    hive_role: Some("agent".to_string()),
                    capabilities: vec!["memory".to_string(), "coordination".to_string()],
                    hive_groups: vec!["openclaw-stack".to_string()],
                    hive_group_goal: None,
                    authority: Some("participant".to_string()),
                    base_url: Some("http://127.0.0.1:8787".to_string()),
                    presence: "active".to_string(),
                    host: None,
                    pid: None,
                    active_claims: 1,
                    workspace: Some("b".to_string()),
                    visibility: Some("workspace".to_string()),
                    topic_claim: None,
                    scope_claims: Vec::new(),
                    task_id: None,
                    focus: None,
                    pressure: None,
                    next_recovery: None,
                    last_updated: None,
                },
            ],
        };

        let summary = render_project_awareness_summary(&response);
        assert!(summary.contains("! shared_hive_endpoint http://127.0.0.1:8787 sessions=2"));
    }

    #[test]
    fn project_awareness_summary_hides_dead_remote_rows_by_default() {
        let response = ProjectAwarenessResponse {
            root: "server:http://127.0.0.1:8787".to_string(),
            current_bundle: "/tmp/projects/current/.memd".to_string(),
            collisions: Vec::new(),
            entries: vec![
                ProjectAwarenessEntry {
                    project_dir: "/tmp/projects/current".to_string(),
                    bundle_root: "/tmp/projects/current/.memd".to_string(),
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    repo_root: None,
                    worktree_root: None,
                    branch: None,
                    base_branch: None,
                    agent: Some("codex".to_string()),
                    session: Some("session-live".to_string()),
                    tab_id: Some("tab-alpha".to_string()),
                    effective_agent: Some("codex@session-live".to_string()),
                    hive_system: Some("codex".to_string()),
                    hive_role: Some("agent".to_string()),
                    capabilities: vec!["memory".to_string()],
                    hive_groups: vec!["project:memd".to_string()],
                    hive_group_goal: None,
                    authority: Some("participant".to_string()),
                    base_url: Some("http://127.0.0.1:8787".to_string()),
                    presence: "active".to_string(),
                    host: None,
                    pid: None,
                    active_claims: 0,
                    workspace: None,
                    visibility: Some("all".to_string()),
                    topic_claim: None,
                    scope_claims: Vec::new(),
                    task_id: None,
                    focus: None,
                    pressure: None,
                    next_recovery: None,
                    last_updated: None,
                },
                ProjectAwarenessEntry {
                    project_dir: "remote".to_string(),
                    bundle_root: "remote:http://127.0.0.1:8787:session-dead".to_string(),
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    repo_root: None,
                    worktree_root: None,
                    branch: None,
                    base_branch: None,
                    agent: Some("codex".to_string()),
                    session: Some("session-dead".to_string()),
                    tab_id: None,
                    effective_agent: Some("codex@session-dead".to_string()),
                    hive_system: None,
                    hive_role: None,
                    capabilities: vec!["memory".to_string()],
                    hive_groups: Vec::new(),
                    hive_group_goal: None,
                    authority: Some("participant".to_string()),
                    base_url: Some("http://127.0.0.1:8787".to_string()),
                    presence: "dead".to_string(),
                    host: None,
                    pid: None,
                    active_claims: 0,
                    workspace: None,
                    visibility: Some("all".to_string()),
                    topic_claim: None,
                    scope_claims: Vec::new(),
                    task_id: None,
                    focus: None,
                    pressure: None,
                    next_recovery: None,
                    last_updated: None,
                },
            ],
        };

        let summary = render_project_awareness_summary(&response);
        assert!(summary.contains(
            "awareness root=server:http://127.0.0.1:8787 bundles=1 diagnostics=0 hidden_remote_dead=1 hidden_superseded_stale=0"
        ));
        assert!(summary.contains("current_session:"));
        assert!(summary.contains("session=session-live"));
        assert!(!summary.contains("session=session-dead"));
    }

    #[test]
    fn project_awareness_summary_calls_out_stale_remote_sessions() {
        let response = ProjectAwarenessResponse {
            root: "server:http://127.0.0.1:8787".to_string(),
            current_bundle: "/tmp/projects/current/.memd".to_string(),
            collisions: Vec::new(),
            entries: vec![
                ProjectAwarenessEntry {
                    project_dir: "/tmp/projects/current".to_string(),
                    bundle_root: "/tmp/projects/current/.memd".to_string(),
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    repo_root: None,
                    worktree_root: None,
                    branch: None,
                    base_branch: None,
                    agent: Some("codex".to_string()),
                    session: Some("session-live".to_string()),
                    tab_id: Some("tab-alpha".to_string()),
                    effective_agent: Some("codex@session-live".to_string()),
                    hive_system: Some("codex".to_string()),
                    hive_role: Some("agent".to_string()),
                    capabilities: vec!["memory".to_string()],
                    hive_groups: vec!["project:memd".to_string()],
                    hive_group_goal: None,
                    authority: Some("participant".to_string()),
                    base_url: Some("http://127.0.0.1:8787".to_string()),
                    presence: "active".to_string(),
                    host: None,
                    pid: None,
                    active_claims: 0,
                    workspace: None,
                    visibility: Some("all".to_string()),
                    topic_claim: None,
                    scope_claims: Vec::new(),
                    task_id: None,
                    focus: None,
                    pressure: None,
                    next_recovery: None,
                    last_updated: None,
                },
                ProjectAwarenessEntry {
                    project_dir: "remote".to_string(),
                    bundle_root: "remote:http://127.0.0.1:8787:session-stale".to_string(),
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    repo_root: None,
                    worktree_root: None,
                    branch: None,
                    base_branch: None,
                    agent: Some("claude-code".to_string()),
                    session: Some("session-stale".to_string()),
                    tab_id: None,
                    effective_agent: Some("claude-code@session-stale".to_string()),
                    hive_system: None,
                    hive_role: None,
                    capabilities: vec!["memory".to_string()],
                    hive_groups: Vec::new(),
                    hive_group_goal: None,
                    authority: Some("participant".to_string()),
                    base_url: Some("http://127.0.0.1:8787".to_string()),
                    presence: "stale".to_string(),
                    host: None,
                    pid: None,
                    active_claims: 0,
                    workspace: None,
                    visibility: Some("all".to_string()),
                    topic_claim: None,
                    scope_claims: Vec::new(),
                    task_id: None,
                    focus: None,
                    pressure: None,
                    next_recovery: None,
                    last_updated: None,
                },
            ],
        };

        let summary = render_project_awareness_summary(&response);
        assert!(summary.contains("! stale_remote_sessions=1 sessions=session-stale"));
        assert!(summary.contains("stale_sessions:"));
    }

    #[test]
    fn project_awareness_summary_marks_current_and_active_hive_sessions() {
        let now = Utc::now();
        let response = ProjectAwarenessResponse {
            root: "server:http://127.0.0.1:8787".to_string(),
            current_bundle: "/tmp/projects/current/.memd".to_string(),
            collisions: Vec::new(),
            entries: vec![
                ProjectAwarenessEntry {
                    project_dir: "/tmp/projects/current".to_string(),
                    bundle_root: "/tmp/projects/current/.memd".to_string(),
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    repo_root: None,
                    worktree_root: None,
                    branch: None,
                    base_branch: None,
                    agent: Some("codex".to_string()),
                    session: Some("session-current".to_string()),
                    tab_id: Some("tab-alpha".to_string()),
                    effective_agent: Some("codex@session-current".to_string()),
                    hive_system: Some("codex".to_string()),
                    hive_role: Some("agent".to_string()),
                    capabilities: vec!["memory".to_string()],
                    hive_groups: vec!["project:memd".to_string()],
                    hive_group_goal: None,
                    authority: Some("participant".to_string()),
                    base_url: Some("http://127.0.0.1:8787".to_string()),
                    presence: "active".to_string(),
                    host: None,
                    pid: None,
                    active_claims: 0,
                    workspace: None,
                    visibility: Some("all".to_string()),
                    topic_claim: None,
                    scope_claims: Vec::new(),
                    task_id: None,
                    focus: None,
                    pressure: None,
                    next_recovery: None,
                    last_updated: Some(now),
                },
                ProjectAwarenessEntry {
                    project_dir: "remote".to_string(),
                    bundle_root: "remote:http://127.0.0.1:8787:session-other".to_string(),
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    repo_root: None,
                    worktree_root: None,
                    branch: None,
                    base_branch: None,
                    agent: Some("codex".to_string()),
                    session: Some("session-other".to_string()),
                    tab_id: Some("tab-beta".to_string()),
                    effective_agent: Some("codex@session-other".to_string()),
                    hive_system: Some("codex".to_string()),
                    hive_role: Some("agent".to_string()),
                    capabilities: vec!["memory".to_string()],
                    hive_groups: vec!["project:memd".to_string()],
                    hive_group_goal: None,
                    authority: Some("participant".to_string()),
                    base_url: Some("http://127.0.0.1:8787".to_string()),
                    presence: "active".to_string(),
                    host: None,
                    pid: None,
                    active_claims: 0,
                    workspace: None,
                    visibility: Some("all".to_string()),
                    topic_claim: None,
                    scope_claims: Vec::new(),
                    task_id: None,
                    focus: None,
                    pressure: None,
                    next_recovery: None,
                    last_updated: Some(now),
                },
            ],
        };

        let summary = render_project_awareness_summary(&response);
        assert!(summary.contains("! active_hive_sessions=1 sessions=session-other"));
        assert!(summary.contains("current_session:"));
        assert!(summary.contains("active_hive_sessions:"));
        assert!(summary.contains("memd [current] | presence=active truth=current"));
        assert!(summary.contains("memd [hive-session] | presence=active truth=fresh"));
    }

    #[test]
    fn project_awareness_summary_hides_superseded_stale_session_rows() {
        let response = ProjectAwarenessResponse {
            root: "server:http://127.0.0.1:8787".to_string(),
            current_bundle: "/tmp/projects/current/.memd".to_string(),
            collisions: Vec::new(),
            entries: vec![
                ProjectAwarenessEntry {
                    project_dir: "/tmp/projects/current".to_string(),
                    bundle_root: "/tmp/projects/current/.memd".to_string(),
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    repo_root: None,
                    worktree_root: None,
                    branch: None,
                    base_branch: None,
                    agent: Some("codex".to_string()),
                    session: Some("session-fresh".to_string()),
                    tab_id: Some("tab-alpha".to_string()),
                    effective_agent: Some("codex@session-fresh".to_string()),
                    hive_system: Some("codex".to_string()),
                    hive_role: Some("agent".to_string()),
                    capabilities: vec!["memory".to_string()],
                    hive_groups: vec!["project:memd".to_string()],
                    hive_group_goal: None,
                    authority: Some("participant".to_string()),
                    base_url: Some("http://127.0.0.1:8787".to_string()),
                    presence: "active".to_string(),
                    host: None,
                    pid: None,
                    active_claims: 0,
                    workspace: None,
                    visibility: Some("all".to_string()),
                    topic_claim: None,
                    scope_claims: Vec::new(),
                    task_id: None,
                    focus: None,
                    pressure: None,
                    next_recovery: None,
                    last_updated: None,
                },
                ProjectAwarenessEntry {
                    project_dir: "remote".to_string(),
                    bundle_root: "remote:http://127.0.0.1:8787:session-stale".to_string(),
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    repo_root: None,
                    worktree_root: None,
                    branch: None,
                    base_branch: None,
                    agent: Some("codex".to_string()),
                    session: Some("session-stale".to_string()),
                    tab_id: None,
                    effective_agent: Some("codex@session-stale".to_string()),
                    hive_system: None,
                    hive_role: None,
                    capabilities: vec!["memory".to_string()],
                    hive_groups: Vec::new(),
                    hive_group_goal: None,
                    authority: Some("participant".to_string()),
                    base_url: Some("http://127.0.0.1:8787".to_string()),
                    presence: "stale".to_string(),
                    host: None,
                    pid: None,
                    active_claims: 0,
                    workspace: None,
                    visibility: Some("all".to_string()),
                    topic_claim: None,
                    scope_claims: Vec::new(),
                    task_id: None,
                    focus: None,
                    pressure: None,
                    next_recovery: None,
                    last_updated: None,
                },
            ],
        };

        let summary = render_project_awareness_summary(&response);
        assert!(summary.contains("hidden_superseded_stale=1"));
        assert!(summary.contains("! superseded_stale_sessions=1 sessions=session-stale"));
        assert!(!summary.contains("session=session-stale"));
        assert!(!summary.contains("stale_sessions:"));
    }

    #[test]
    fn project_awareness_summary_groups_sessions_into_current_active_stale_dead_sections() {
        let now = Utc::now();
        let response = ProjectAwarenessResponse {
            root: "server:http://127.0.0.1:8787".to_string(),
            current_bundle: "/tmp/projects/current/.memd".to_string(),
            collisions: Vec::new(),
            entries: vec![
                ProjectAwarenessEntry {
                    project_dir: "/tmp/projects/current".to_string(),
                    bundle_root: "/tmp/projects/current/.memd".to_string(),
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    repo_root: None,
                    worktree_root: None,
                    branch: None,
                    base_branch: None,
                    agent: Some("codex".to_string()),
                    session: Some("session-current".to_string()),
                    tab_id: Some("tab-current".to_string()),
                    effective_agent: Some("codex@session-current".to_string()),
                    hive_system: Some("codex".to_string()),
                    hive_role: Some("agent".to_string()),
                    capabilities: vec!["memory".to_string()],
                    hive_groups: vec!["project:memd".to_string()],
                    hive_group_goal: None,
                    authority: Some("participant".to_string()),
                    base_url: Some("http://127.0.0.1:8787".to_string()),
                    presence: "active".to_string(),
                    host: None,
                    pid: None,
                    active_claims: 1,
                    workspace: None,
                    visibility: Some("all".to_string()),
                    topic_claim: None,
                    scope_claims: Vec::new(),
                    task_id: None,
                    focus: None,
                    pressure: None,
                    next_recovery: None,
                    last_updated: Some(now),
                },
                ProjectAwarenessEntry {
                    project_dir: "remote".to_string(),
                    bundle_root: "remote:http://127.0.0.1:8787:session-active".to_string(),
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    repo_root: None,
                    worktree_root: None,
                    branch: None,
                    base_branch: None,
                    agent: Some("codex".to_string()),
                    session: Some("session-active".to_string()),
                    tab_id: Some("tab-active".to_string()),
                    effective_agent: Some("codex@session-active".to_string()),
                    hive_system: Some("codex".to_string()),
                    hive_role: Some("agent".to_string()),
                    capabilities: vec!["memory".to_string()],
                    hive_groups: vec!["project:memd".to_string()],
                    hive_group_goal: None,
                    authority: Some("participant".to_string()),
                    base_url: Some("http://127.0.0.1:8787".to_string()),
                    presence: "active".to_string(),
                    host: None,
                    pid: None,
                    active_claims: 0,
                    workspace: None,
                    visibility: Some("all".to_string()),
                    topic_claim: None,
                    scope_claims: Vec::new(),
                    task_id: None,
                    focus: None,
                    pressure: None,
                    next_recovery: None,
                    last_updated: Some(now),
                },
                ProjectAwarenessEntry {
                    project_dir: "remote".to_string(),
                    bundle_root: "remote:http://127.0.0.1:8787:session-stale".to_string(),
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    repo_root: None,
                    worktree_root: None,
                    branch: None,
                    base_branch: None,
                    agent: Some("codex".to_string()),
                    session: Some("session-stale".to_string()),
                    tab_id: None,
                    effective_agent: Some("codex@session-stale".to_string()),
                    hive_system: None,
                    hive_role: None,
                    capabilities: vec!["memory".to_string()],
                    hive_groups: Vec::new(),
                    hive_group_goal: None,
                    authority: Some("participant".to_string()),
                    base_url: Some("http://127.0.0.1:8787".to_string()),
                    presence: "stale".to_string(),
                    host: None,
                    pid: None,
                    active_claims: 0,
                    workspace: None,
                    visibility: Some("all".to_string()),
                    topic_claim: None,
                    scope_claims: Vec::new(),
                    task_id: None,
                    focus: None,
                    pressure: None,
                    next_recovery: None,
                    last_updated: Some(now - chrono::TimeDelta::minutes(7)),
                },
                ProjectAwarenessEntry {
                    project_dir: "remote".to_string(),
                    bundle_root: "remote:http://127.0.0.1:8787:session-stale-visible".to_string(),
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    repo_root: None,
                    worktree_root: None,
                    branch: None,
                    base_branch: None,
                    agent: Some("claude-code".to_string()),
                    session: Some("session-stale-visible".to_string()),
                    tab_id: None,
                    effective_agent: Some("claude-code@session-stale-visible".to_string()),
                    hive_system: None,
                    hive_role: None,
                    capabilities: vec!["memory".to_string()],
                    hive_groups: Vec::new(),
                    hive_group_goal: None,
                    authority: Some("participant".to_string()),
                    base_url: Some("http://127.0.0.1:8787".to_string()),
                    presence: "stale".to_string(),
                    host: None,
                    pid: None,
                    active_claims: 0,
                    workspace: None,
                    visibility: Some("all".to_string()),
                    topic_claim: None,
                    scope_claims: Vec::new(),
                    task_id: None,
                    focus: None,
                    pressure: None,
                    next_recovery: None,
                    last_updated: Some(now - chrono::TimeDelta::minutes(6)),
                },
                ProjectAwarenessEntry {
                    project_dir: "/tmp/projects/seen".to_string(),
                    bundle_root: "/tmp/projects/seen/.memd".to_string(),
                    project: Some("seen".to_string()),
                    namespace: Some("main".to_string()),
                    repo_root: None,
                    worktree_root: None,
                    branch: None,
                    base_branch: None,
                    agent: Some("codex".to_string()),
                    session: Some("session-seen".to_string()),
                    tab_id: None,
                    effective_agent: Some("codex@session-seen".to_string()),
                    hive_system: None,
                    hive_role: None,
                    capabilities: vec!["memory".to_string()],
                    hive_groups: Vec::new(),
                    hive_group_goal: None,
                    authority: None,
                    base_url: None,
                    presence: "unknown".to_string(),
                    host: None,
                    pid: None,
                    active_claims: 0,
                    workspace: None,
                    visibility: Some("all".to_string()),
                    topic_claim: None,
                    scope_claims: Vec::new(),
                    task_id: None,
                    focus: None,
                    pressure: None,
                    next_recovery: None,
                    last_updated: Some(now - chrono::TimeDelta::minutes(2)),
                },
                ProjectAwarenessEntry {
                    project_dir: "/tmp/projects/dead".to_string(),
                    bundle_root: "/tmp/projects/dead/.memd".to_string(),
                    project: Some("dead".to_string()),
                    namespace: Some("main".to_string()),
                    repo_root: None,
                    worktree_root: None,
                    branch: None,
                    base_branch: None,
                    agent: Some("codex".to_string()),
                    session: Some("session-dead".to_string()),
                    tab_id: None,
                    effective_agent: Some("codex@session-dead".to_string()),
                    hive_system: None,
                    hive_role: None,
                    capabilities: vec!["memory".to_string()],
                    hive_groups: Vec::new(),
                    hive_group_goal: None,
                    authority: None,
                    base_url: None,
                    presence: "dead".to_string(),
                    host: None,
                    pid: None,
                    active_claims: 0,
                    workspace: None,
                    visibility: Some("all".to_string()),
                    topic_claim: None,
                    scope_claims: Vec::new(),
                    task_id: None,
                    focus: None,
                    pressure: None,
                    next_recovery: None,
                    last_updated: Some(now - chrono::TimeDelta::minutes(30)),
                },
            ],
        };

        let summary = render_project_awareness_summary(&response);
        assert!(summary.starts_with("awareness root=server:http://127.0.0.1:8787"));
        assert!(summary.contains("current_session:"));
        assert!(summary.contains("active_hive_sessions:"));
        assert!(summary.contains("stale_sessions:"));
        assert!(summary.contains("dead_sessions:"));
        assert!(summary.contains("seen_sessions:"));
    }

    #[test]
    fn project_awareness_summary_hides_shadowed_seen_entry_for_current_lane() {
        let now = Utc::now();
        let response = ProjectAwarenessResponse {
            root: "server:http://127.0.0.1:8787".to_string(),
            current_bundle: "/tmp/projects/current/.memd".to_string(),
            collisions: Vec::new(),
            entries: vec![
                ProjectAwarenessEntry {
                    project_dir: "/tmp/projects/current".to_string(),
                    bundle_root: "/tmp/projects/current/.memd".to_string(),
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    repo_root: None,
                    worktree_root: Some("/tmp/projects/current".to_string()),
                    branch: Some("feature/live".to_string()),
                    base_branch: Some("main".to_string()),
                    agent: Some("codex".to_string()),
                    session: Some("session-live".to_string()),
                    tab_id: None,
                    effective_agent: Some("codex@session-live".to_string()),
                    hive_system: Some("codex".to_string()),
                    hive_role: Some("agent".to_string()),
                    capabilities: vec!["memory".to_string()],
                    hive_groups: vec!["project:memd".to_string()],
                    hive_group_goal: None,
                    authority: Some("participant".to_string()),
                    base_url: Some("http://127.0.0.1:8787".to_string()),
                    presence: "active".to_string(),
                    host: None,
                    pid: None,
                    active_claims: 0,
                    workspace: None,
                    visibility: Some("all".to_string()),
                    topic_claim: None,
                    scope_claims: Vec::new(),
                    task_id: None,
                    focus: Some("Finish the lane isolation fix".to_string()),
                    pressure: None,
                    next_recovery: None,
                    last_updated: Some(now),
                },
                ProjectAwarenessEntry {
                    project_dir: "/tmp/projects/old".to_string(),
                    bundle_root: "/tmp/projects/old/.memd".to_string(),
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    repo_root: None,
                    worktree_root: None,
                    branch: None,
                    base_branch: None,
                    agent: Some("codex".to_string()),
                    session: Some("session-old".to_string()),
                    tab_id: None,
                    effective_agent: Some("codex@session-old".to_string()),
                    hive_system: None,
                    hive_role: None,
                    capabilities: vec!["memory".to_string()],
                    hive_groups: Vec::new(),
                    hive_group_goal: None,
                    authority: Some("participant".to_string()),
                    base_url: Some("http://127.0.0.1:8787".to_string()),
                    presence: "active".to_string(),
                    host: None,
                    pid: None,
                    active_claims: 0,
                    workspace: None,
                    visibility: Some("all".to_string()),
                    topic_claim: None,
                    scope_claims: Vec::new(),
                    task_id: None,
                    focus: Some("Old stale local bundle".to_string()),
                    pressure: None,
                    next_recovery: None,
                    last_updated: Some(now - chrono::TimeDelta::minutes(5)),
                },
            ],
        };

        let summary = render_project_awareness_summary(&response);
        assert!(!summary.contains("session=session-old"));
        assert!(!summary.contains("Old stale local bundle"));
    }

    #[test]
    fn project_awareness_summary_surfaces_compact_work_quickview() {
        let now = Utc::now();
        let response = ProjectAwarenessResponse {
            root: "/tmp/projects".to_string(),
            current_bundle: "/tmp/projects/current/.memd".to_string(),
            collisions: Vec::new(),
            entries: vec![ProjectAwarenessEntry {
                project_dir: "/tmp/projects/current".to_string(),
                bundle_root: "/tmp/projects/current/.memd".to_string(),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: Some("/tmp/projects/current".to_string()),
                branch: Some("feature/live".to_string()),
                base_branch: Some("main".to_string()),
                agent: Some("codex".to_string()),
                session: Some("session-live".to_string()),
                tab_id: None,
                effective_agent: Some("codex@session-live".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["project:memd".to_string()],
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
                focus: Some("Finish the queen-bee quickview summary".to_string()),
                pressure: Some("file_edited: crates/memd-client/src/main.rs | scope=task:queen-bee-awareness | Avoid overlap in worker lane routing".to_string()),
                next_recovery: Some("publish overlap-safe hive quickview".to_string()),
                last_updated: Some(now),
            }],
        };

        let summary = render_project_awareness_summary(&response);
        assert!(summary.contains("work=\"Finish the queen-bee quickview summary\""));
        assert!(
            summary.contains("touches=crates/memd-client/src/main.rs,task:queen-bee-awareness")
        );
        assert!(summary.contains("next=\"publish overlap-safe hive quickview\""));
    }

    #[test]
    fn project_awareness_summary_surfaces_possible_work_overlap_diagnostics() {
        let now = Utc::now();
        let response = ProjectAwarenessResponse {
            root: "server:http://127.0.0.1:8787".to_string(),
            current_bundle: "/tmp/projects/current/.memd".to_string(),
            collisions: Vec::new(),
            entries: vec![
                ProjectAwarenessEntry {
                    project_dir: "/tmp/projects/current".to_string(),
                    bundle_root: "/tmp/projects/current/.memd".to_string(),
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    repo_root: None,
                    worktree_root: Some("/tmp/projects/current".to_string()),
                    branch: Some("feature/queen".to_string()),
                    base_branch: Some("main".to_string()),
                    agent: Some("codex".to_string()),
                    session: Some("session-current".to_string()),
                    tab_id: None,
                    effective_agent: Some("codex@session-current".to_string()),
                    hive_system: Some("codex".to_string()),
                    hive_role: Some("agent".to_string()),
                    capabilities: vec!["memory".to_string()],
                    hive_groups: vec!["project:memd".to_string()],
                    hive_group_goal: None,
                    authority: Some("participant".to_string()),
                    base_url: Some("http://127.0.0.1:8787".to_string()),
                    presence: "active".to_string(),
                    host: None,
                    pid: None,
                    active_claims: 1,
                    workspace: Some("shared".to_string()),
                    visibility: Some("workspace".to_string()),
                    topic_claim: None,
                    scope_claims: Vec::new(),
                    task_id: None,
                    focus: Some("Refine hive overlap awareness".to_string()),
                    pressure: Some("file_edited: crates/memd-client/src/main.rs | scope=task:queen-bee-awareness".to_string()),
                    next_recovery: None,
                    last_updated: Some(now),
                },
                ProjectAwarenessEntry {
                    project_dir: "remote".to_string(),
                    bundle_root: "remote:http://127.0.0.1:8787:session-peer".to_string(),
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    repo_root: None,
                    worktree_root: Some("/tmp/projects/peer".to_string()),
                    branch: Some("feature/peer".to_string()),
                    base_branch: Some("main".to_string()),
                    agent: Some("codex".to_string()),
                    session: Some("session-peer".to_string()),
                    tab_id: None,
                    effective_agent: Some("codex@session-peer".to_string()),
                    hive_system: Some("codex".to_string()),
                    hive_role: Some("agent".to_string()),
                    capabilities: vec!["memory".to_string()],
                    hive_groups: vec!["project:memd".to_string()],
                    hive_group_goal: None,
                    authority: Some("participant".to_string()),
                    base_url: Some("http://127.0.0.1:8787".to_string()),
                    presence: "active".to_string(),
                    host: None,
                    pid: None,
                    active_claims: 1,
                    workspace: Some("shared".to_string()),
                    visibility: Some("workspace".to_string()),
                    topic_claim: None,
                    scope_claims: Vec::new(),
                    task_id: None,
                    focus: Some("Finish peer awareness lane".to_string()),
                    pressure: Some("file_edited: crates/memd-client/src/main.rs | scope=task:peer-quickview".to_string()),
                    next_recovery: None,
                    last_updated: Some(now),
                },
            ],
        };

        let summary = render_project_awareness_summary(&response);
        assert!(summary.contains(
            "! possible_work_overlap touches=crates/memd-client/src/main.rs sessions=session-current,session-peer"
        ));
    }

    #[test]
    fn project_awareness_summary_ignores_generic_project_overlap_noise() {
        let now = Utc::now();
        let response = ProjectAwarenessResponse {
            root: "server:http://127.0.0.1:8787".to_string(),
            current_bundle: "/tmp/projects/current/.memd".to_string(),
            collisions: Vec::new(),
            entries: vec![
                ProjectAwarenessEntry {
                    project_dir: "remote".to_string(),
                    bundle_root: "remote:http://127.0.0.1:8787:session-a".to_string(),
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    repo_root: None,
                    worktree_root: None,
                    branch: None,
                    base_branch: None,
                    agent: Some("codex".to_string()),
                    session: Some("session-a".to_string()),
                    tab_id: None,
                    effective_agent: Some("codex@session-a".to_string()),
                    hive_system: Some("codex".to_string()),
                    hive_role: Some("agent".to_string()),
                    capabilities: vec!["memory".to_string()],
                    hive_groups: vec!["project:memd".to_string()],
                    hive_group_goal: None,
                    authority: Some("participant".to_string()),
                    base_url: Some("http://127.0.0.1:8787".to_string()),
                    presence: "active".to_string(),
                    host: None,
                    pid: None,
                    active_claims: 0,
                    workspace: Some("shared".to_string()),
                    visibility: Some("workspace".to_string()),
                    topic_claim: Some("Parser lane refactor".to_string()),
                    scope_claims: vec!["project".to_string()],
                    task_id: Some("bee-a".to_string()),
                    focus: Some("id=1 | scope=project | ws=shared".to_string()),
                    pressure: None,
                    next_recovery: None,
                    last_updated: Some(now),
                },
                ProjectAwarenessEntry {
                    project_dir: "remote".to_string(),
                    bundle_root: "remote:http://127.0.0.1:8787:session-b".to_string(),
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    repo_root: None,
                    worktree_root: None,
                    branch: None,
                    base_branch: None,
                    agent: Some("codex".to_string()),
                    session: Some("session-b".to_string()),
                    tab_id: None,
                    effective_agent: Some("codex@session-b".to_string()),
                    hive_system: Some("codex".to_string()),
                    hive_role: Some("agent".to_string()),
                    capabilities: vec!["memory".to_string()],
                    hive_groups: vec!["project:memd".to_string()],
                    hive_group_goal: None,
                    authority: Some("participant".to_string()),
                    base_url: Some("http://127.0.0.1:8787".to_string()),
                    presence: "active".to_string(),
                    host: None,
                    pid: None,
                    active_claims: 0,
                    workspace: Some("shared".to_string()),
                    visibility: Some("workspace".to_string()),
                    topic_claim: Some("Render lane polish".to_string()),
                    scope_claims: vec!["project".to_string()],
                    task_id: Some("bee-b".to_string()),
                    focus: Some("id=2 | scope=project | ws=shared".to_string()),
                    pressure: None,
                    next_recovery: None,
                    last_updated: Some(now),
                },
            ],
        };

        let summary = render_project_awareness_summary(&response);
        assert!(!summary.contains("possible_work_overlap"));
    }

    #[test]
