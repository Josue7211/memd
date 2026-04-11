use super::*;

#[test]
fn project_awareness_scans_sibling_bundles_without_current() {
    let root = std::env::temp_dir().join(format!("memd-awareness-root-{}", uuid::Uuid::new_v4()));
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
    let root = std::env::temp_dir().join(format!("memd-awareness-live-{}", uuid::Uuid::new_v4()));
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
        entry.project.as_deref() == Some("current") && entry.session.as_deref() == Some("current-a")
    }));
    assert!(response.entries.iter().any(|entry| {
        entry.project.as_deref() == Some("sibling") && entry.agent.as_deref() == Some("claude-code")
    }));

    fs::remove_dir_all(root).expect("cleanup awareness root");
}

#[test]
fn project_awareness_local_prunes_dead_sibling_without_active_claims() {
    let root = std::env::temp_dir().join(format!("memd-awareness-prune-{}", uuid::Uuid::new_v4()));
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
    assert!(summary.contains("touches=crates/memd-client/src/main.rs,task:queen-bee-awareness"));
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
                pressure: Some(
                    "file_edited: crates/memd-client/src/main.rs | scope=task:queen-bee-awareness"
                        .to_string(),
                ),
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
                pressure: Some(
                    "file_edited: crates/memd-client/src/main.rs | scope=task:peer-quickview"
                        .to_string(),
                ),
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
fn render_awareness_entry_line_prefers_first_class_topic_scope_and_task_fields() {
    let now = Utc::now();
    let entry = ProjectAwarenessEntry {
        project_dir: "/tmp/projects/current".to_string(),
        bundle_root: "/tmp/projects/current/.memd".to_string(),
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        repo_root: None,
        worktree_root: Some("/tmp/projects/current".to_string()),
        branch: Some("feature/queen".to_string()),
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
        active_claims: 1,
        workspace: Some("shared".to_string()),
        visibility: Some("workspace".to_string()),
        topic_claim: Some("Refine hive overlap awareness".to_string()),
        scope_claims: vec![
            "task:queen-bee-awareness".to_string(),
            "crates/memd-client/src/main.rs".to_string(),
        ],
        task_id: Some("queen-bee-awareness".to_string()),
        focus: Some("stale focus fallback".to_string()),
        pressure: Some("stale pressure fallback".to_string()),
        next_recovery: None,
        last_updated: Some(now),
    };

    let line = render_awareness_entry_line(&entry, "current", &entry.bundle_root);
    assert!(line.contains("task=queen-bee-awareness"));
    assert!(line.contains("work=\"Refine hive overlap awareness\""));
    assert!(line.contains("touches=task:queen-bee-awareness,crates/memd-client/src/main.rs"));
}

#[test]
fn render_hive_roster_summary_prefers_worker_names_and_role_lane_task() {
    let response = HiveRosterResponse {
        project: "memd".to_string(),
        namespace: "main".to_string(),
        queen_session: Some("session-queen".to_string()),
        bees: vec![memd_schema::HiveSessionRecord {
            session: "session-lorentz".to_string(),
            tab_id: None,
            agent: Some("codex".to_string()),
            effective_agent: Some("Lorentz@session-lorentz".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("reviewer".to_string()),
            worker_name: Some("Lorentz".to_string()),
            display_name: None,
            role: Some("reviewer".to_string()),
            capabilities: vec!["review".to_string(), "coordination".to_string()],
            hive_groups: vec!["project:memd".to_string()],
            lane_id: Some("lane-review".to_string()),
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            repo_root: Some("/repo".to_string()),
            worktree_root: Some("/repo-review".to_string()),
            branch: Some("review/parser".to_string()),
            base_branch: Some("main".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            base_url_healthy: Some(true),
            host: None,
            pid: None,
            topic_claim: Some("Review parser handoff".to_string()),
            scope_claims: vec!["crates/memd-client/src/main.rs".to_string()],
            task_id: Some("review-parser".to_string()),
            focus: Some("Review overlap guard output".to_string()),
            pressure: None,
            next_recovery: None,
            next_action: Some("Review overlap guard output".to_string()),
            needs_help: false,
            needs_review: true,
            handoff_state: None,
            confidence: None,
            risk: None,
            status: "active".to_string(),
            last_seen: Utc::now(),
        }],
    };

    let summary = render_hive_roster_summary(&response);
    assert!(summary.contains("Lorentz (session-lorentz)"));
    assert!(summary.contains("role=reviewer"));
    assert!(summary.contains("lane=lane-review"));
    assert!(summary.contains("task=review-parser"));
    assert!(summary.contains("caps=review,coordination"));
}

#[test]
fn render_hive_roster_summary_prefers_display_name_for_generic_workers() {
    let response = HiveRosterResponse {
        project: "memd".to_string(),
        namespace: "main".to_string(),
        queen_session: Some("session-queen".to_string()),
        bees: vec![memd_schema::HiveSessionRecord {
            session: "session-6d422e56".to_string(),
            tab_id: None,
            agent: Some("codex".to_string()),
            effective_agent: Some("codex@session-6d422e56".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("agent".to_string()),
            worker_name: Some("codex".to_string()),
            display_name: Some("Codex 6d422e56".to_string()),
            role: Some("agent".to_string()),
            capabilities: vec!["memory".to_string()],
            hive_groups: vec!["project:memd".to_string()],
            lane_id: Some("lane-main".to_string()),
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            repo_root: Some("/repo".to_string()),
            worktree_root: Some("/repo".to_string()),
            branch: Some("main".to_string()),
            base_branch: Some("main".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            base_url_healthy: Some(true),
            host: None,
            pid: None,
            topic_claim: Some("Parser refactor".to_string()),
            scope_claims: vec!["crates/memd-client/src/main.rs".to_string()],
            task_id: Some("parser-refactor".to_string()),
            focus: None,
            pressure: None,
            next_recovery: None,
            next_action: None,
            needs_help: false,
            needs_review: false,
            handoff_state: None,
            confidence: None,
            risk: None,
            status: "active".to_string(),
            last_seen: Utc::now(),
        }],
    };

    let summary = render_hive_roster_summary(&response);
    assert!(summary.contains("Codex 6d422e56 (session-6d422e56)"));
    assert!(!summary.contains("- codex (session-6d422e56)"));
}

#[test]
fn cli_parses_hive_follow_subcommand() {
    let cli = Cli::try_parse_from([
        "memd",
        "hive",
        "follow",
        "--output",
        ".memd",
        "--worker",
        "Lorentz",
        "--summary",
    ])
    .expect("hive follow command should parse");

    match cli.command {
        Commands::Hive(args) => match args.command {
            Some(HiveSubcommand::Follow(follow)) => {
                assert_eq!(follow.output, PathBuf::from(".memd"));
                assert_eq!(follow.worker.as_deref(), Some("Lorentz"));
                assert!(follow.summary);
            }
            other => panic!("expected hive follow subcommand, got {other:?}"),
        },
        other => panic!("expected hive command, got {other:?}"),
    }
}

#[test]
fn cli_parses_hive_handoff_subcommand() {
    let cli = Cli::try_parse_from([
        "memd",
        "hive",
        "handoff",
        "--output",
        ".memd",
        "--to-worker",
        "Avicenna",
        "--task-id",
        "parser-refactor",
        "--scope",
        "crates/memd-client/src/main.rs,task:parser-refactor",
        "--summary",
    ])
    .expect("hive handoff command should parse");

    match cli.command {
        Commands::Hive(args) => match args.command {
            Some(HiveSubcommand::Handoff(handoff)) => {
                assert_eq!(handoff.output, PathBuf::from(".memd"));
                assert_eq!(handoff.to_worker.as_deref(), Some("Avicenna"));
                assert_eq!(handoff.task_id.as_deref(), Some("parser-refactor"));
                assert_eq!(
                    handoff.scope,
                    vec![
                        "crates/memd-client/src/main.rs".to_string(),
                        "task:parser-refactor".to_string()
                    ]
                );
                assert!(handoff.summary);
            }
            other => panic!("expected hive handoff subcommand, got {other:?}"),
        },
        other => panic!("expected hive command, got {other:?}"),
    }
}

#[test]
fn cli_parses_hive_follow_watch_subcommand() {
    let cli = Cli::try_parse_from([
        "memd",
        "hive",
        "follow",
        "--output",
        ".memd",
        "--worker",
        "Lorentz",
        "--watch",
        "--interval-secs",
        "2",
    ])
    .expect("hive follow watch command should parse");

    match cli.command {
        Commands::Hive(args) => match args.command {
            Some(HiveSubcommand::Follow(follow)) => {
                assert_eq!(follow.output, PathBuf::from(".memd"));
                assert_eq!(follow.worker.as_deref(), Some("Lorentz"));
                assert!(follow.watch);
                assert_eq!(follow.interval_secs, 2);
            }
            other => panic!("expected hive follow subcommand, got {other:?}"),
        },
        other => panic!("expected hive command, got {other:?}"),
    }
}

#[test]
fn render_hive_handoff_summary_surfaces_packet_fields() {
    let response = HiveHandoffResponse {
        packet: HiveHandoffPacket {
            from_session: "session-anscombe".to_string(),
            from_worker: Some("Anscombe".to_string()),
            to_session: "session-avicenna".to_string(),
            to_worker: Some("Avicenna".to_string()),
            task_id: Some("parser-refactor".to_string()),
            topic_claim: Some("Parser overlap cleanup".to_string()),
            scope_claims: vec![
                "task:parser-refactor".to_string(),
                "crates/memd-client/src/main.rs".to_string(),
            ],
            next_action: Some("Finish overlap guard cleanup".to_string()),
            blocker: Some("render lane is about to converge".to_string()),
            note: Some("Keep render.rs out of scope".to_string()),
            created_at: Utc::now(),
        },
        receipt_kind: "queen_handoff".to_string(),
        receipt_summary: "Handoff to Avicenna (session-avicenna) task=parser-refactor".to_string(),
        message_id: Some("msg-1".to_string()),
        recommended_follow: "memd hive follow --session session-avicenna --summary".to_string(),
    };

    let summary = render_hive_handoff_summary(&response);
    assert!(summary.contains("hive_handoff from=Anscombe (session-anscombe)"));
    assert!(summary.contains("to=Avicenna (session-avicenna)"));
    assert!(summary.contains("task=parser-refactor"));
    assert!(summary.contains("scopes=task:parser-refactor,crates/memd-client/src/main.rs"));
    assert!(summary.contains("receipt_kind=queen_handoff"));
    assert!(summary.contains("follow=\"memd hive follow --session session-avicenna --summary\""));
}

#[test]
fn render_hive_follow_summary_surfaces_messages_receipts_and_overlap_risk() {
    let response = HiveFollowResponse {
        current_session: Some("session-current".to_string()),
        target: memd_schema::HiveSessionRecord {
            session: "session-lorentz".to_string(),
            tab_id: None,
            agent: Some("codex".to_string()),
            effective_agent: Some("Lorentz@session-lorentz".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("reviewer".to_string()),
            worker_name: Some("Lorentz".to_string()),
            display_name: None,
            role: Some("reviewer".to_string()),
            capabilities: vec!["review".to_string()],
            hive_groups: vec!["project:memd".to_string()],
            lane_id: Some("lane-review".to_string()),
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            repo_root: Some("/repo".to_string()),
            worktree_root: Some("/repo-review".to_string()),
            branch: Some("review/parser".to_string()),
            base_branch: Some("main".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            base_url_healthy: Some(true),
            host: None,
            pid: None,
            topic_claim: Some("Review parser handoff".to_string()),
            scope_claims: vec![
                "task:review-parser".to_string(),
                "crates/memd-client/src/main.rs".to_string(),
            ],
            task_id: Some("review-parser".to_string()),
            focus: Some("Review overlap guard output".to_string()),
            pressure: None,
            next_recovery: None,
            next_action: Some("Reply with review notes".to_string()),
            needs_help: false,
            needs_review: true,
            handoff_state: None,
            confidence: None,
            risk: Some("medium".to_string()),
            status: "active".to_string(),
            last_seen: Utc::now(),
        },
        work_summary: "Review parser handoff".to_string(),
        touch_points: vec![
            "task:review-parser".to_string(),
            "crates/memd-client/src/main.rs".to_string(),
        ],
        next_action: Some("Reply with review notes".to_string()),
        messages: vec![HiveMessageRecord {
            id: "msg-1".to_string(),
            kind: "note".to_string(),
            from_session: "session-queen".to_string(),
            from_agent: Some("Anscombe".to_string()),
            to_session: "session-lorentz".to_string(),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            content: "Stay on parser review and avoid render.rs.".to_string(),
            created_at: Utc::now(),
            acknowledged_at: None,
        }],
        owned_tasks: vec![HiveTaskRecord {
            task_id: "review-parser".to_string(),
            title: "Review parser handoff".to_string(),
            description: None,
            status: "active".to_string(),
            coordination_mode: "shared_review".to_string(),
            session: Some("session-lorentz".to_string()),
            agent: Some("codex".to_string()),
            effective_agent: Some("Lorentz@session-lorentz".to_string()),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            claim_scopes: vec!["crates/memd-client/src/main.rs".to_string()],
            help_requested: false,
            review_requested: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }],
        help_tasks: Vec::new(),
        review_tasks: Vec::new(),
        recent_receipts: vec![HiveCoordinationReceiptRecord {
            id: "receipt-1".to_string(),
            kind: "queen_handoff".to_string(),
            actor_session: "session-queen".to_string(),
            actor_agent: Some("Anscombe".to_string()),
            target_session: Some("session-lorentz".to_string()),
            task_id: Some("review-parser".to_string()),
            scope: Some("crates/memd-client/src/main.rs".to_string()),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            summary: "Queen handed off parser review scope to Lorentz.".to_string(),
            created_at: Utc::now(),
        }],
        overlap_risk: Some(
            "confirmed hive overlap: target session session-lorentz already claims crates/memd-client/src/main.rs".to_string(),
        ),
        recommended_action: "coordinate_now".to_string(),
    };

    let summary = render_hive_follow_summary(&response);
    assert!(summary.contains("hive_follow worker=Lorentz session=session-lorentz"));
    assert!(summary.contains("recommended_action=coordinate_now"));
    assert!(summary.contains("overlap_risk=confirmed hive overlap"));
    assert!(summary.contains("## Messages"));
    assert!(summary.contains("Stay on parser review and avoid render.rs."));
    assert!(summary.contains("## Tasks"));
    assert!(summary.contains("owned review-parser status=active"));
    assert!(summary.contains("## Receipts"));
    assert!(summary.contains("queen_handoff actor=Anscombe"));
}

#[test]
fn render_hive_follow_watch_frame_includes_timestamp_and_summary() {
    let response = HiveFollowResponse {
        current_session: Some("session-current".to_string()),
        target: memd_schema::HiveSessionRecord {
            session: "session-lorentz".to_string(),
            tab_id: None,
            agent: Some("codex".to_string()),
            effective_agent: Some("Lorentz@session-lorentz".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("reviewer".to_string()),
            worker_name: Some("Lorentz".to_string()),
            display_name: None,
            role: Some("reviewer".to_string()),
            capabilities: vec!["review".to_string()],
            hive_groups: vec!["project:memd".to_string()],
            lane_id: Some("lane-review".to_string()),
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            repo_root: Some("/repo".to_string()),
            worktree_root: Some("/repo-review".to_string()),
            branch: Some("review/parser".to_string()),
            base_branch: Some("main".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            base_url_healthy: Some(true),
            host: None,
            pid: None,
            topic_claim: Some("Review parser handoff".to_string()),
            scope_claims: vec!["crates/memd-client/src/main.rs".to_string()],
            task_id: Some("review-parser".to_string()),
            focus: Some("Review overlap guard output".to_string()),
            pressure: None,
            next_recovery: None,
            next_action: Some("Reply with review notes".to_string()),
            needs_help: false,
            needs_review: true,
            handoff_state: None,
            confidence: None,
            risk: Some("medium".to_string()),
            status: "active".to_string(),
            last_seen: Utc::now(),
        },
        work_summary: "Review parser handoff".to_string(),
        touch_points: vec!["crates/memd-client/src/main.rs".to_string()],
        next_action: Some("Reply with review notes".to_string()),
        messages: Vec::new(),
        owned_tasks: Vec::new(),
        help_tasks: Vec::new(),
        review_tasks: Vec::new(),
        recent_receipts: Vec::new(),
        overlap_risk: None,
        recommended_action: "safe_to_continue".to_string(),
    };

    let frame = render_hive_follow_watch_frame(
        &response,
        DateTime::parse_from_rfc3339("2026-04-09T22:30:00Z")
            .expect("parse timestamp")
            .with_timezone(&Utc),
    );
    assert!(frame.contains("== hive follow 2026-04-09T22:30:00+00:00 =="));
    assert!(frame.contains("hive_follow worker=Lorentz session=session-lorentz"));
}

#[test]
fn build_hive_heartbeat_prefers_explicit_worker_name_env() {
    let _env_lock = lock_env_mutation();
    let dir = std::env::temp_dir().join(format!("memd-heartbeat-worker-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("create temp bundle");
    fs::write(
        dir.join("config.json"),
        r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "openclaw",
  "session": "session-openclaw",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "current_task"
}
"#,
    )
    .expect("write config");

    unsafe {
        std::env::set_var("MEMD_WORKER_NAME", "Openclaw");
    }
    let heartbeat = build_hive_heartbeat(&dir, None).expect("build heartbeat");
    unsafe {
        std::env::remove_var("MEMD_WORKER_NAME");
    }

    assert_eq!(heartbeat.worker_name.as_deref(), Some("Openclaw"));
    assert!(heartbeat.display_name.is_none());

    fs::remove_dir_all(dir).expect("cleanup temp bundle");
}

#[test]
fn build_hive_heartbeat_uses_project_scoped_worker_name_for_generic_agents() {
    let _env_lock = lock_env_mutation();
    let dir = std::env::temp_dir().join(format!(
        "memd-heartbeat-generic-worker-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("create temp bundle");
    fs::write(
        dir.join("config.json"),
        r#"{
  "project": "memd",
  "namespace": "main",
  "agent": "codex",
  "session": "session-6d422e56",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "current_task"
}
"#,
    )
    .expect("write config");

    let heartbeat = build_hive_heartbeat(&dir, None).expect("build heartbeat");

    assert_eq!(
        heartbeat.worker_name.as_deref(),
        Some("Memd Codex 6d422e56")
    );
    assert!(heartbeat.display_name.is_none());

    fs::remove_dir_all(dir).expect("cleanup temp bundle");
}

#[test]
fn cli_parses_hive_queen_subcommand() {
    let cli = Cli::try_parse_from([
        "memd",
        "hive",
        "queen",
        "--output",
        ".memd",
        "--deny-session",
        "session-avicenna",
        "--summary",
    ])
    .expect("hive queen command should parse");

    match cli.command {
        Commands::Hive(args) => match args.command {
            Some(HiveSubcommand::Queen(queen)) => {
                assert_eq!(queen.output, PathBuf::from(".memd"));
                assert_eq!(queen.deny_session.as_deref(), Some("session-avicenna"));
                assert!(queen.summary);
            }
            other => panic!("expected hive queen subcommand, got {other:?}"),
        },
        other => panic!("expected hive command, got {other:?}"),
    }
}

#[test]
fn render_hive_queen_summary_surfaces_explicit_actions() {
    let response = HiveQueenResponse {
        queen_session: "session-queen".to_string(),
        suggested_actions: vec![
            "reroute Lorentz off crates/memd-client/src/main.rs".to_string(),
            "retire stale bee session-old".to_string(),
        ],
        action_cards: vec![HiveQueenActionCard {
            action: "reroute".to_string(),
            priority: "high".to_string(),
            target_session: Some("session-lorentz".to_string()),
            target_worker: Some("Lorentz".to_string()),
            task_id: Some("review-parser".to_string()),
            scope: Some("crates/memd-client/src/main.rs".to_string()),
            reason: "shared scope is colliding".to_string(),
            follow_command: Some(
                "memd hive follow --session session-lorentz --summary".to_string(),
            ),
            deny_command: Some(
                "memd hive queen --deny-session session-lorentz --summary".to_string(),
            ),
            reroute_command: Some(
                "memd hive queen --reroute-session session-lorentz --summary".to_string(),
            ),
            retire_command: None,
        }],
        recent_receipts: vec![
            "queen_assign session-lorentz review-parser".to_string(),
            "queen_deny session-avicenna overlap-main-rs".to_string(),
        ],
    };

    let summary = render_hive_queen_summary(&response);
    assert!(summary.contains("hive_queen queen=session-queen suggested=2 cards=1 receipts=2"));
    assert!(summary.contains("- reroute Lorentz off crates/memd-client/src/main.rs"));
    assert!(summary.contains("queen_deny session-avicenna"));
    assert!(summary.contains("## Action Cards"));
    assert!(summary.contains(
        "- [high] reroute target=Lorentz task=review-parser scope=crates/memd-client/src/main.rs"
    ));
    assert!(summary.contains("reason=\"shared scope is colliding\""));
    assert!(summary.contains("commands:"));
    assert!(summary.contains("follow=`memd hive follow --session session-lorentz --summary`"));
    assert!(
        summary.contains("reroute=`memd hive queen --reroute-session session-lorentz --summary`")
    );
    assert!(
        summary.contains("deny=`memd hive queen --deny-session session-lorentz --summary`")
    );
}

#[test]
fn render_hive_board_summary_surfaces_board_sections() {
    let response = HiveBoardResponse {
        queen_session: Some("session-queen".to_string()),
        active_bees: vec![memd_schema::HiveSessionRecord {
            session: "session-lorentz".to_string(),
            tab_id: None,
            agent: Some("codex".to_string()),
            effective_agent: Some("Lorentz@session-lorentz".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("reviewer".to_string()),
            worker_name: Some("Lorentz".to_string()),
            display_name: None,
            role: Some("reviewer".to_string()),
            capabilities: vec!["review".to_string()],
            hive_groups: vec!["project:memd".to_string()],
            lane_id: Some("lane-review".to_string()),
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            repo_root: Some("/repo".to_string()),
            worktree_root: Some("/repo-review".to_string()),
            branch: Some("review/parser".to_string()),
            base_branch: Some("main".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            base_url_healthy: Some(true),
            host: None,
            pid: None,
            topic_claim: Some("Review parser handoff".to_string()),
            scope_claims: vec!["crates/memd-client/src/main.rs".to_string()],
            task_id: Some("review-parser".to_string()),
            focus: None,
            pressure: None,
            next_recovery: None,
            next_action: None,
            needs_help: false,
            needs_review: true,
            handoff_state: None,
            confidence: None,
            risk: None,
            status: "active".to_string(),
            last_seen: Utc::now(),
        }],
        blocked_bees: vec!["Avicenna overlap on crates/memd-client/src/main.rs".to_string()],
        stale_bees: vec!["session-old".to_string()],
        review_queue: vec!["review-parser -> Lorentz".to_string()],
        overlap_risks: vec!["Lorentz vs Avicenna on crates/memd-client/src/main.rs".to_string()],
        lane_faults: vec!["lane_fault session-avicenna shared worktree".to_string()],
        recommended_actions: vec!["reroute Avicenna".to_string()],
    };

    let summary = render_hive_board_summary(&response);
    assert!(summary.contains("## Active Bees"));
    assert!(summary.contains("## Review Queue"));
    assert!(summary.contains("## Recommended Actions"));
    assert!(summary.contains("Lorentz (session-lorentz)"));
}

#[test]
fn hive_board_response_includes_dashboard_panels() {
    let response = HiveBoardResponse {
        queen_session: Some("session-queen".to_string()),
        active_bees: vec![memd_schema::HiveSessionRecord {
            session: "session-lorentz".to_string(),
            tab_id: None,
            agent: Some("codex".to_string()),
            effective_agent: Some("Lorentz@session-lorentz".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("reviewer".to_string()),
            worker_name: Some("Lorentz".to_string()),
            display_name: None,
            role: Some("reviewer".to_string()),
            capabilities: vec!["review".to_string()],
            hive_groups: vec!["project:memd".to_string()],
            lane_id: Some("lane-review".to_string()),
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: Some("review/parser".to_string()),
            base_branch: Some("main".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            base_url_healthy: Some(true),
            host: None,
            pid: None,
            topic_claim: Some("Review parser handoff".to_string()),
            scope_claims: vec!["crates/memd-client/src/main.rs".to_string()],
            task_id: Some("review-parser".to_string()),
            focus: None,
            pressure: None,
            next_recovery: None,
            next_action: None,
            needs_help: false,
            needs_review: true,
            handoff_state: None,
            confidence: None,
            risk: None,
            status: "active".to_string(),
            last_seen: Utc::now(),
        }],
        blocked_bees: vec!["Avicenna overlap".to_string()],
        stale_bees: vec!["session-old".to_string()],
        review_queue: vec!["review-parser -> Lorentz".to_string()],
        overlap_risks: vec!["Lorentz vs Avicenna".to_string()],
        lane_faults: vec!["lane_fault session-avicenna".to_string()],
        recommended_actions: vec!["reroute Avicenna".to_string()],
    };

    let json = serde_json::to_value(&response).expect("serialize board");
    assert!(json.get("active_bees").is_some());
    assert!(json.get("review_queue").is_some());
    assert!(json.get("lane_faults").is_some());
    assert!(json.get("recommended_actions").is_some());
}

#[tokio::test]
async fn run_hive_handoff_command_emits_message_and_receipt_for_target_worker() {
    let dir = std::env::temp_dir().join(format!("memd-hive-handoff-{}", uuid::Uuid::new_v4()));
    let output = dir.join(".memd");
    fs::create_dir_all(&output).expect("create output dir");

    let state = MockRuntimeState::default();
    let base_url = spawn_mock_runtime_server(state.clone(), false).await;
    write_test_bundle_config(&output, &base_url);
    fs::write(
        output.join("config.json"),
        format!(
            r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "session-anscombe",
  "workspace": null,
  "visibility": "workspace",
  "base_url": "{}",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task"
}}
"#,
            base_url
        ),
    )
    .expect("rewrite bundle config");

    {
        let mut sessions = state.session_records.lock().expect("lock session records");
        sessions.push(memd_schema::HiveSessionRecord {
            session: "session-anscombe".to_string(),
            tab_id: None,
            agent: Some("codex".to_string()),
            effective_agent: Some("Anscombe@session-anscombe".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("orchestrator".to_string()),
            worker_name: Some("Anscombe".to_string()),
            display_name: None,
            role: Some("queen".to_string()),
            capabilities: vec!["coordination".to_string()],
            hive_groups: vec!["project:memd".to_string()],
            lane_id: Some("lane-queen".to_string()),
            hive_group_goal: None,
            authority: Some("coordinator".to_string()),
            heartbeat_model: None,
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            repo_root: None,
            worktree_root: Some("/tmp/queen".to_string()),
            branch: Some("queen".to_string()),
            base_branch: Some("main".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some(base_url.clone()),
            base_url_healthy: Some(true),
            host: None,
            pid: None,
            topic_claim: Some("Parser overlap cleanup".to_string()),
            scope_claims: vec![
                "task:parser-refactor".to_string(),
                "crates/memd-client/src/main.rs".to_string(),
            ],
            task_id: Some("parser-refactor".to_string()),
            focus: Some("handoff parser lane".to_string()),
            pressure: None,
            next_recovery: Some("finish overlap guard cleanup".to_string()),
            next_action: Some("finish overlap guard cleanup".to_string()),
            needs_help: false,
            needs_review: false,
            handoff_state: None,
            confidence: None,
            risk: None,
            status: "active".to_string(),
            last_seen: Utc::now(),
        });
        sessions.push(memd_schema::HiveSessionRecord {
            session: "session-avicenna".to_string(),
            tab_id: None,
            agent: Some("codex".to_string()),
            effective_agent: Some("Avicenna@session-avicenna".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("worker".to_string()),
            worker_name: Some("Avicenna".to_string()),
            display_name: None,
            role: Some("worker".to_string()),
            capabilities: vec!["refactor".to_string()],
            hive_groups: vec!["project:memd".to_string()],
            lane_id: Some("lane-parser".to_string()),
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: None,
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            repo_root: None,
            worktree_root: Some("/tmp/parser".to_string()),
            branch: Some("feature/parser".to_string()),
            base_branch: Some("main".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some(base_url.clone()),
            base_url_healthy: Some(true),
            host: None,
            pid: None,
            topic_claim: Some("Receive parser handoff".to_string()),
            scope_claims: vec!["task:parser-refactor".to_string()],
            task_id: Some("parser-refactor".to_string()),
            focus: None,
            pressure: None,
            next_recovery: None,
            next_action: None,
            needs_help: false,
            needs_review: false,
            handoff_state: None,
            confidence: None,
            risk: None,
            status: "active".to_string(),
            last_seen: Utc::now(),
        });
    }

    let response = run_hive_handoff_command(
        &HiveHandoffArgs {
            output: output.clone(),
            to_session: None,
            to_worker: Some("Avicenna".to_string()),
            task_id: Some("parser-refactor".to_string()),
            topic: Some("Parser overlap cleanup".to_string()),
            scope: vec![
                "task:parser-refactor".to_string(),
                "crates/memd-client/src/main.rs".to_string(),
            ],
            next_action: Some("Finish overlap guard cleanup".to_string()),
            blocker: Some("render lane is converging".to_string()),
            note: Some("Keep render.rs out of scope".to_string()),
            json: false,
            summary: false,
        },
        &base_url,
    )
    .await
    .expect("run hive handoff");

    assert_eq!(response.packet.to_session, "session-avicenna");
    assert_eq!(response.packet.to_worker.as_deref(), Some("Avicenna"));
    assert_eq!(response.packet.task_id.as_deref(), Some("parser-refactor"));
    assert_eq!(
        response.packet.scope_claims,
        vec![
            "task:parser-refactor".to_string(),
            "crates/memd-client/src/main.rs".to_string()
        ]
    );
    assert!(response.message_id.is_some());

    let messages = state.messages.lock().expect("lock runtime messages");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].kind, "handoff");
    assert_eq!(messages[0].to_session, "session-avicenna");
    assert_eq!(messages[0].workspace.as_deref(), Some("shared"));
    assert!(messages[0].content.contains("handoff_packet"));
    assert!(messages[0].content.contains("task=parser-refactor"));

    let receipts = state.receipts.lock().expect("lock runtime receipts");
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].kind, "queen_handoff");
    assert_eq!(
        receipts[0].target_session.as_deref(),
        Some("session-avicenna")
    );
    assert_eq!(receipts[0].task_id.as_deref(), Some("parser-refactor"));

    fs::remove_dir_all(&dir).expect("cleanup handoff temp dir");
}

#[tokio::test]
async fn hive_handoff_is_visible_in_target_inbox_and_follow_surfaces() {
    let dir =
        std::env::temp_dir().join(format!("memd-hive-handoff-follow-{}", uuid::Uuid::new_v4()));
    let sender_output = dir.join("sender/.memd");
    let target_output = dir.join("target/.memd");
    fs::create_dir_all(&sender_output).expect("create sender output dir");
    fs::create_dir_all(&target_output).expect("create target output dir");

    let state = MockRuntimeState::default();
    let base_url = spawn_mock_runtime_server(state.clone(), false).await;
    write_test_bundle_config(&sender_output, &base_url);
    write_test_bundle_config(&target_output, &base_url);
    fs::write(
        sender_output.join("config.json"),
        format!(
            r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "avicenna",
  "session": "session-avicenna",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "{}",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task"
}}
"#,
            base_url
        ),
    )
    .expect("rewrite sender bundle config");
    fs::write(
        target_output.join("config.json"),
        format!(
            r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "noether",
  "session": "session-noether",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "{}",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task"
}}
"#,
            base_url
        ),
    )
    .expect("rewrite target bundle config");

    {
        let mut sessions = state.session_records.lock().expect("lock session records");
        sessions.push(memd_schema::HiveSessionRecord {
            session: "session-avicenna".to_string(),
            tab_id: None,
            agent: Some("avicenna".to_string()),
            effective_agent: Some("avicenna@session-avicenna".to_string()),
            hive_system: Some("avicenna".to_string()),
            hive_role: Some("agent".to_string()),
            worker_name: Some("Avicenna".to_string()),
            display_name: None,
            role: Some("worker".to_string()),
            capabilities: vec!["coordination".to_string()],
            hive_groups: vec!["project:demo".to_string()],
            lane_id: Some("lane-parser".to_string()),
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: None,
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            repo_root: None,
            worktree_root: Some("/tmp/parser".to_string()),
            branch: Some("feature/parser".to_string()),
            base_branch: Some("main".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some(base_url.clone()),
            base_url_healthy: Some(true),
            host: None,
            pid: None,
            topic_claim: Some("Parser handoff".to_string()),
            scope_claims: vec!["crates/memd-client/src/main.rs".to_string()],
            task_id: Some("review-parser".to_string()),
            focus: None,
            pressure: None,
            next_recovery: None,
            next_action: Some("Send parser handoff".to_string()),
            needs_help: false,
            needs_review: false,
            handoff_state: None,
            confidence: None,
            risk: None,
            status: "active".to_string(),
            last_seen: Utc::now(),
        });
        sessions.push(memd_schema::HiveSessionRecord {
            session: "session-noether".to_string(),
            tab_id: None,
            agent: Some("noether".to_string()),
            effective_agent: Some("noether@session-noether".to_string()),
            hive_system: Some("noether".to_string()),
            hive_role: Some("agent".to_string()),
            worker_name: Some("Noether".to_string()),
            display_name: None,
            role: Some("worker".to_string()),
            capabilities: vec!["review".to_string()],
            hive_groups: vec!["project:demo".to_string()],
            lane_id: Some("lane-review".to_string()),
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: None,
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            repo_root: None,
            worktree_root: Some("/tmp/review".to_string()),
            branch: Some("review/parser".to_string()),
            base_branch: Some("main".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some(base_url.clone()),
            base_url_healthy: Some(true),
            host: None,
            pid: None,
            topic_claim: Some("Receive parser handoff".to_string()),
            scope_claims: vec!["crates/memd-client/src/main.rs".to_string()],
            task_id: Some("review-parser".to_string()),
            focus: None,
            pressure: None,
            next_recovery: None,
            next_action: Some("Review parser handoff".to_string()),
            needs_help: false,
            needs_review: true,
            handoff_state: None,
            confidence: None,
            risk: None,
            status: "active".to_string(),
            last_seen: Utc::now(),
        });
    }

    let handoff = run_hive_handoff_command(
        &HiveHandoffArgs {
            output: sender_output.clone(),
            to_session: None,
            to_worker: Some("Noether".to_string()),
            task_id: Some("review-parser".to_string()),
            topic: Some("Review parser handoff".to_string()),
            scope: vec!["crates/memd-client/src/main.rs".to_string()],
            next_action: Some("Reply with review notes".to_string()),
            blocker: None,
            note: Some("Stay on parser review.".to_string()),
            json: false,
            summary: false,
        },
        &base_url,
    )
    .await
    .expect("run hive handoff");

    assert!(handoff.message_id.is_some());

    let inbox = run_messages_command(
        &MessagesArgs {
            output: target_output.clone(),
            send: false,
            inbox: true,
            ack: None,
            target_session: None,
            kind: None,
            request_help: false,
            request_review: false,
            assign_scope: None,
            scope: None,
            content: None,
            summary: false,
        },
        &base_url,
    )
    .await
    .expect("read target inbox");
    assert_eq!(inbox.messages.len(), 1);
    assert_eq!(inbox.messages[0].kind, "handoff");
    assert_eq!(inbox.messages[0].to_session, "session-noether");
    assert!(inbox.messages[0].content.contains("task=review-parser"));

    let follow = run_hive_follow_command(&HiveFollowArgs {
        output: target_output.clone(),
        session: Some("session-noether".to_string()),
        worker: None,
        watch: false,
        interval_secs: 5,
        json: false,
        summary: false,
    })
    .await
    .expect("run hive follow");
    assert_eq!(follow.target.session, "session-noether");
    assert_eq!(follow.messages.len(), 1);
    assert_eq!(follow.messages[0].id, inbox.messages[0].id);
    assert_eq!(follow.recent_receipts.len(), 1);
    assert_eq!(follow.recent_receipts[0].kind, "queen_handoff");
    assert_eq!(follow.recommended_action, "watch_and_coordinate");

    fs::remove_dir_all(&dir).expect("cleanup handoff follow temp dir");
}

#[tokio::test]
async fn run_hive_board_command_prunes_retired_stale_bees_from_default_view() {
    let dir = std::env::temp_dir().join(format!("memd-hive-board-retire-{}", uuid::Uuid::new_v4()));
    let output = dir.join(".memd");
    fs::create_dir_all(&output).expect("create output dir");

    let state = MockRuntimeState::default();
    {
        let mut sessions = state.session_records.lock().expect("lock session records");
        sessions.push(memd_schema::HiveSessionRecord {
            session: "codex-a".to_string(),
            tab_id: None,
            agent: Some("codex".to_string()),
            effective_agent: Some("Anscombe@codex-a".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("orchestrator".to_string()),
            worker_name: Some("Anscombe".to_string()),
            display_name: None,
            role: Some("queen".to_string()),
            capabilities: vec!["coordination".to_string()],
            hive_groups: vec!["project:memd".to_string()],
            lane_id: Some("lane-queen".to_string()),
            hive_group_goal: None,
            authority: Some("coordinator".to_string()),
            heartbeat_model: None,
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: Some("feature/queen".to_string()),
            base_branch: Some("main".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: None,
            base_url_healthy: Some(true),
            host: None,
            pid: None,
            topic_claim: Some("Route hive board".to_string()),
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
            status: "active".to_string(),
            last_seen: Utc::now(),
        });
        sessions.push(memd_schema::HiveSessionRecord {
            session: "session-stale".to_string(),
            tab_id: None,
            agent: Some("codex".to_string()),
            effective_agent: Some("Lorentz@session-stale".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("agent".to_string()),
            worker_name: Some("Lorentz".to_string()),
            display_name: None,
            role: Some("worker".to_string()),
            capabilities: vec!["review".to_string()],
            hive_groups: vec!["project:memd".to_string()],
            lane_id: Some("lane-review".to_string()),
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: None,
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: Some("feature/review".to_string()),
            base_branch: Some("main".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: None,
            base_url_healthy: Some(true),
            host: None,
            pid: None,
            topic_claim: Some("Old stale work".to_string()),
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
            status: "active".to_string(),
            last_seen: Utc::now() - chrono::TimeDelta::minutes(45),
        });
    }

    let base_url = spawn_mock_runtime_server(state.clone(), false).await;
    write_test_bundle_config(&output, &base_url);
    let board = run_hive_board_command(
        &HiveArgs {
            command: None,
            agent: None,
            project: None,
            namespace: None,
            global: false,
            project_root: None,
            seed_existing: true,
            session: None,
            tab_id: None,
            hive_system: None,
            hive_role: None,
            capability: Vec::new(),
            hive_group: Vec::new(),
            hive_group_goal: None,
            authority: None,
            output: output.clone(),
            base_url: base_url.clone(),
            rag_url: None,
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            workspace: None,
            visibility: None,
            publish_heartbeat: true,
            force: false,
            summary: true,
        },
        &base_url,
    )
    .await
    .expect("board");

    assert!(board.stale_bees.is_empty());
    let sessions = state.session_records.lock().expect("lock session records");
    assert!(
        sessions
            .iter()
            .all(|session| session.session != "session-stale")
    );

    fs::remove_dir_all(dir).expect("cleanup board retire dir");
}

#[test]
fn build_hive_heartbeat_derives_first_class_intent_fields() {
    let dir = std::env::temp_dir().join(format!("memd-heartbeat-intent-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(dir.join("state")).expect("create temp dir");
    std::fs::write(
        dir.join("state/claims.json"),
        serde_json::to_string_pretty(&SessionClaimsState {
            claims: vec![SessionClaim {
                scope: "task:queen-bee-awareness".to_string(),
                session: Some("session-live".to_string()),
                tab_id: None,
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@session-live".to_string()),
                project: Some("memd".to_string()),
                workspace: Some("shared".to_string()),
                host: None,
                pid: None,
                acquired_at: Utc::now(),
                expires_at: Utc::now() + chrono::TimeDelta::minutes(15),
            }],
        })
        .expect("serialize claims"),
    )
    .expect("write claims");

    let snapshot = BundleResumeState {
        focus: Some("Refine hive overlap awareness".to_string()),
        pressure: Some(
            "file_edited: crates/memd-client/src/main.rs | scope=task:queen-bee-awareness"
                .to_string(),
        ),
        next_recovery: Some("publish overlap-safe hive quickview".to_string()),
        lane: None,
        working_records: 0,
        inbox_items: 0,
        rehydration_items: 0,
        recorded_at: Utc::now(),
    };
    std::fs::write(
        dir.join("state/last-resume.json"),
        serde_json::to_string_pretty(&snapshot).expect("serialize resume"),
    )
    .expect("write resume");

    let heartbeat = build_hive_heartbeat(&dir, None).expect("build heartbeat");
    assert_eq!(
        heartbeat.topic_claim.as_deref(),
        Some("Refine hive overlap awareness")
    );
    assert!(
        heartbeat
            .scope_claims
            .iter()
            .any(|scope| scope == "task:queen-bee-awareness")
    );
    assert!(
        heartbeat
            .scope_claims
            .iter()
            .any(|scope| scope == "crates/memd-client/src/main.rs")
    );
    assert_eq!(heartbeat.task_id.as_deref(), Some("queen-bee-awareness"));

    std::fs::remove_dir_all(dir).expect("cleanup temp dir");
}

#[test]
fn derive_hive_display_name_uses_session_for_generic_agents() {
    assert_eq!(
        derive_hive_display_name(Some("codex"), Some("session-6d422e56")).as_deref(),
        Some("Codex 6d422e56")
    );
    assert_eq!(
        derive_hive_display_name(Some("claude-code"), Some("codex-fresh")).as_deref(),
        Some("Claude fresh")
    );
    assert_eq!(
        derive_hive_display_name(Some("Lorentz"), Some("session-x")),
        None
    );
}

#[test]
fn project_awareness_summary_marks_freshness_and_supersession_from_last_updated() {
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
                last_updated: Some(now),
            },
            ProjectAwarenessEntry {
                project_dir: "remote".to_string(),
                bundle_root: "remote:http://127.0.0.1:8787:session-aging".to_string(),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                agent: Some("codex".to_string()),
                session: Some("session-aging".to_string()),
                tab_id: Some("tab-beta".to_string()),
                effective_agent: Some("codex@session-aging".to_string()),
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
                last_updated: Some(now - chrono::TimeDelta::minutes(10)),
            },
            ProjectAwarenessEntry {
                project_dir: "remote".to_string(),
                bundle_root: "remote:http://127.0.0.1:8787:session-superseded".to_string(),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                agent: Some("codex".to_string()),
                session: Some("session-superseded".to_string()),
                tab_id: None,
                effective_agent: Some("codex@session-superseded".to_string()),
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
                workspace: None,
                visibility: Some("all".to_string()),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: None,
                pressure: None,
                next_recovery: None,
                last_updated: Some(now - chrono::TimeDelta::minutes(9)),
            },
        ],
    };

    let summary = render_project_awareness_summary(&response);
    assert!(summary.contains("memd [current] | presence=active truth=current"));
    assert!(summary.contains("memd [hive-session] | presence=active truth=aging"));
    assert!(summary.contains("! superseded_stale_sessions=1 sessions=session-superseded"));
    assert!(summary.contains("hidden_superseded_stale=1"));
}

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
        workspace: Some("initiative-alpha".to_string()),
        visibility: None,
        heartbeat_model: None,
        auto_short_term_capture: true,
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
            coordination_mode: "exclusive_write".to_string(),
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
            coordination_mode: "exclusive_write".to_string(),
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
