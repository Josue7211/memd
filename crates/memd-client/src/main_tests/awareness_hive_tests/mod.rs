use super::*;

mod awareness_hive_tail;

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
            ..BundleHeartbeatState::default()
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
fn project_awareness_summary_surfaces_hive_goal_mismatch() {
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
                branch: Some("main".to_string()),
                base_branch: None,
                agent: Some("codex".to_string()),
                session: Some("session-current".to_string()),
                tab_id: None,
                effective_agent: Some("codex@session-current".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["project:memd".to_string()],
                hive_group_goal: Some("finish continuity".to_string()),
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
                focus: Some("Finish continuity".to_string()),
                pressure: None,
                next_recovery: None,
                last_updated: Some(now),
            },
            ProjectAwarenessEntry {
                project_dir: "/tmp/projects/peer".to_string(),
                bundle_root: "/tmp/projects/peer/.memd".to_string(),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: Some("/tmp/projects/peer".to_string()),
                branch: Some("main".to_string()),
                base_branch: None,
                agent: Some("codex".to_string()),
                session: Some("session-peer".to_string()),
                tab_id: None,
                effective_agent: Some("codex@session-peer".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                capabilities: vec!["coordination".to_string()],
                hive_groups: vec!["project:memd".to_string()],
                hive_group_goal: Some("ship dashboard".to_string()),
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
                focus: Some("Ship dashboard".to_string()),
                pressure: None,
                next_recovery: None,
                last_updated: Some(now),
            },
        ],
    };

    let summary = render_project_awareness_summary(&response);
    assert!(summary.contains("hive_goal_mismatch group=project:memd"));
    assert!(summary.contains("finish continuity|ship dashboard"));
    assert!(summary.contains("align_hive_group_goal_before_handoff"));
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
            ..memd_schema::HiveSessionRecord::default()
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
            ..memd_schema::HiveSessionRecord::default()
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
        "--allow-ephemeral",
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
                assert!(handoff.allow_ephemeral);
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
            working_context: None,
        },
        receipt_kind: "queen_handoff".to_string(),
        receipt_summary: "Handoff to Avicenna (session-avicenna) task=parser-refactor".to_string(),
        message_id: Some("msg-1".to_string()),
        recommended_follow: "memd hive follow --session session-avicenna --summary".to_string(),
        next_agent_prompt: "You are taking over a memd hive handoff.\nBefore changing files or running shared dev/build commands, publish heartbeat and check hive board for collisions.\nBefore broad Git, Cargo, test, or repo-scan work, run scripts/memd-host-io-guard.sh; exit 75 means wait and report the blocker scope/project_hint.\nUse .memd/state/codebase-live-map.json and .memd/state/codebase-live-map-events.ndjson as the live diff surface; record hook/file paths before waiting for heartbeat.".to_string(),
    };

    let summary = render_hive_handoff_summary(&response);
    assert!(summary.contains("hive_handoff from=Anscombe (session-anscombe)"));
    assert!(summary.contains("to=Avicenna (session-avicenna)"));
    assert!(summary.contains("task=parser-refactor"));
    assert!(summary.contains("scopes=task:parser-refactor,crates/memd-client/src/main.rs"));
    assert!(summary.contains("receipt_kind=queen_handoff"));
    assert!(summary.contains("follow=\"memd hive follow --session session-avicenna --summary\""));
    assert!(summary.contains("next_agent_prompt:"));
    assert!(summary.contains("check hive board for collisions"));
    assert!(summary.contains("scripts/memd-host-io-guard.sh"));
    assert!(summary.contains("codebase-live-map-events.ndjson"));
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
            ..memd_schema::HiveSessionRecord::default()
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
            coordination_mode: CoordinationMode::SharedReview,
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
            ..memd_schema::HiveSessionRecord::default()
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
        None,
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
            cowork_command: None,
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
    assert!(summary.contains("deny=`memd hive queen --deny-session session-lorentz --summary`"));
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
            ..memd_schema::HiveSessionRecord::default()
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
            ..memd_schema::HiveSessionRecord::default()
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

include!("awareness_hive_messages.rs");
