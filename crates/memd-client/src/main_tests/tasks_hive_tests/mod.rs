use super::*;

#[tokio::test]
async fn run_tasks_command_supports_owned_view() {
    let dir = std::env::temp_dir().join(format!("memd-tasks-view-owned-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("create temp dir");
    let state = MockRuntimeState::default();
    {
        let mut tasks = state.task_records.lock().expect("lock task records");
        tasks.push(HiveTaskRecord {
            task_id: "owned-1".to_string(),
            title: "owned".to_string(),
            description: None,
            status: "in_progress".to_string(),
            coordination_mode: "exclusive_write".to_string(),
            session: Some("codex-a".to_string()),
            agent: Some("codex".to_string()),
            effective_agent: Some("codex@codex-a".to_string()),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            claim_scopes: vec!["src/main.rs".to_string()],
            help_requested: false,
            review_requested: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });
        tasks.push(HiveTaskRecord {
            task_id: "shared-2".to_string(),
            title: "other".to_string(),
            description: None,
            status: "needs_review".to_string(),
            coordination_mode: "shared_review".to_string(),
            session: Some("codex-b".to_string()),
            agent: Some("codex".to_string()),
            effective_agent: Some("codex@codex-b".to_string()),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            claim_scopes: Vec::new(),
            help_requested: false,
            review_requested: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });
    }
    let base_url = spawn_mock_runtime_server(state, false).await;
    fs::write(
        dir.join("config.json"),
        format!(
            r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "{}",
  "route": "auto",
  "intent": "current_task"
}}
"#,
            base_url
        ),
    )
    .expect("write config");

    let response = run_tasks_command(
        &TasksArgs {
            output: dir.clone(),
            upsert: false,
            assign_to_session: None,
            target_session: None,
            task_id: None,
            title: None,
            description: None,
            status: None,
            mode: None,
            scope: Vec::new(),
            request_help: false,
            request_review: false,
            all: false,
            view: Some("owned".to_string()),
            summary: true,
            json: false,
        },
        SHARED_MEMD_BASE_URL,
    )
    .await
    .expect("tasks response");

    assert_eq!(response.tasks.len(), 1);
    assert_eq!(response.tasks[0].task_id, "owned-1");

    fs::remove_dir_all(dir).expect("cleanup temp dir");
}

#[tokio::test]
async fn run_tasks_command_rejects_colliding_assignment_target_lane() {
    let root = std::env::temp_dir().join(format!("memd-tasks-collision-{}", uuid::Uuid::new_v4()));
    let current_project = root.join("current");
    let target_project = root.join("target");
    let current_bundle = current_project.join(".memd");
    let target_bundle = target_project.join(".memd");
    fs::create_dir_all(&current_bundle).expect("create current bundle");
    fs::create_dir_all(&target_bundle).expect("create target bundle");
    fs::create_dir_all(current_project.join(".planning")).expect("create current planning");
    fs::create_dir_all(target_project.join(".planning")).expect("create target planning");
    let state = MockRuntimeState::default();
    let base_url = spawn_mock_runtime_server(state, false).await;

    write_test_bundle_config(&current_bundle, &base_url);
    fs::write(
        target_bundle.join("config.json"),
        format!(
            r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "claude-code",
  "session": "claude-b",
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
    .expect("write target config");
    fs::write(current_project.join("README.md"), "# current\n").expect("write readme");
    fs::write(target_project.join("NOTES.md"), "# target\n").expect("write notes");
    init_test_git_repo(&root);
    checkout_test_branch(&root, "feature/hive-shared");

    write_test_bundle_heartbeat(
        &target_bundle,
        &BundleHeartbeatState {
            session: Some("claude-b".to_string()),
            agent: Some("claude-code".to_string()),
            effective_agent: Some("claude-code@claude-b".to_string()),
            tab_id: None,
            hive_system: Some("claude-code".to_string()),
            hive_role: Some("agent".to_string()),
            worker_name: Some("claude-code".to_string()),
            display_name: None,
            role: Some("agent".to_string()),
            capabilities: vec!["memory".to_string()],
            hive_groups: vec!["openclaw-stack".to_string()],
            lane_id: Some(root.display().to_string()),
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: Some(default_heartbeat_model()),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            repo_root: Some(root.display().to_string()),
            worktree_root: Some(root.display().to_string()),
            branch: Some("feature/hive-shared".to_string()),
            base_branch: Some("master".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some(base_url.clone()),
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
            last_seen: Utc::now(),
            authority_mode: Some("shared".to_string()),
            authority_degraded: false,
        },
    );

    let err = run_tasks_command(
        &TasksArgs {
            output: current_bundle.clone(),
            upsert: false,
            assign_to_session: Some("claude-b".to_string()),
            target_session: None,
            task_id: Some("task-1".to_string()),
            title: None,
            description: None,
            status: None,
            mode: None,
            scope: Vec::new(),
            request_help: false,
            request_review: false,
            all: false,
            view: None,
            summary: false,
            json: false,
        },
        SHARED_MEMD_BASE_URL,
    )
    .await
    .expect_err("colliding task assignment should fail");
    assert!(
        err.to_string()
            .contains("unsafe hive cowork target collision")
    );

    fs::remove_dir_all(root).expect("cleanup tasks collision dir");
}

#[tokio::test]
async fn run_tasks_command_rejects_colliding_help_target_lane() {
    let root = std::env::temp_dir().join(format!(
        "memd-tasks-help-collision-{}",
        uuid::Uuid::new_v4()
    ));
    let current_project = root.join("current");
    let target_project = root.join("target");
    let current_bundle = current_project.join(".memd");
    let target_bundle = target_project.join(".memd");
    fs::create_dir_all(&current_bundle).expect("create current bundle");
    fs::create_dir_all(&target_bundle).expect("create target bundle");
    fs::create_dir_all(current_project.join(".planning")).expect("create current planning");
    fs::create_dir_all(target_project.join(".planning")).expect("create target planning");
    let state = MockRuntimeState::default();
    let base_url = spawn_mock_runtime_server(state.clone(), false).await;

    write_test_bundle_config(&current_bundle, &base_url);
    fs::write(
        target_bundle.join("config.json"),
        format!(
            r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "claude-code",
  "session": "claude-b",
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
    .expect("write target config");
    fs::write(current_project.join("README.md"), "# current\n").expect("write readme");
    fs::write(target_project.join("NOTES.md"), "# target\n").expect("write notes");
    init_test_git_repo(&root);
    checkout_test_branch(&root, "feature/hive-shared");

    write_test_bundle_heartbeat(
        &target_bundle,
        &BundleHeartbeatState {
            session: Some("claude-b".to_string()),
            agent: Some("claude-code".to_string()),
            effective_agent: Some("claude-code@claude-b".to_string()),
            tab_id: None,
            hive_system: Some("claude-code".to_string()),
            hive_role: Some("agent".to_string()),
            worker_name: Some("claude-code".to_string()),
            display_name: None,
            role: Some("agent".to_string()),
            capabilities: vec!["memory".to_string()],
            hive_groups: vec!["openclaw-stack".to_string()],
            lane_id: Some(root.display().to_string()),
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: Some(default_heartbeat_model()),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            repo_root: Some(root.display().to_string()),
            worktree_root: Some(root.display().to_string()),
            branch: Some("feature/hive-shared".to_string()),
            base_branch: Some("master".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some(base_url.clone()),
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
            last_seen: Utc::now(),
            authority_mode: Some("shared".to_string()),
            authority_degraded: false,
        },
    );

    let err = run_tasks_command(
        &TasksArgs {
            output: current_bundle.clone(),
            upsert: false,
            assign_to_session: None,
            target_session: Some("claude-b".to_string()),
            task_id: Some("task-1".to_string()),
            title: Some("need help".to_string()),
            description: None,
            status: None,
            mode: None,
            scope: vec!["src/main.rs".to_string()],
            request_help: true,
            request_review: false,
            all: false,
            view: None,
            summary: false,
            json: false,
        },
        SHARED_MEMD_BASE_URL,
    )
    .await
    .expect_err("colliding help request should fail");
    assert!(
        err.to_string()
            .contains("unsafe hive cowork target collision")
    );

    fs::remove_dir_all(root).expect("cleanup tasks help collision dir");
}

#[tokio::test]
async fn run_tasks_command_rejects_colliding_review_target_lane() {
    let root = std::env::temp_dir().join(format!(
        "memd-tasks-review-collision-{}",
        uuid::Uuid::new_v4()
    ));
    let current_project = root.join("current");
    let target_project = root.join("target");
    let current_bundle = current_project.join(".memd");
    let target_bundle = target_project.join(".memd");
    fs::create_dir_all(&current_bundle).expect("create current bundle");
    fs::create_dir_all(&target_bundle).expect("create target bundle");
    fs::create_dir_all(current_project.join(".planning")).expect("create current planning");
    fs::create_dir_all(target_project.join(".planning")).expect("create target planning");
    let base_url = spawn_mock_runtime_server(MockRuntimeState::default(), false).await;

    write_test_bundle_config(&current_bundle, &base_url);
    fs::write(
        target_bundle.join("config.json"),
        format!(
            r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "claude-code",
  "session": "claude-b",
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
    .expect("write target config");
    fs::write(current_project.join("README.md"), "# current\n").expect("write readme");
    fs::write(target_project.join("NOTES.md"), "# target\n").expect("write notes");
    init_test_git_repo(&root);
    checkout_test_branch(&root, "feature/hive-shared");

    write_test_bundle_heartbeat(
        &target_bundle,
        &BundleHeartbeatState {
            session: Some("claude-b".to_string()),
            agent: Some("claude-code".to_string()),
            effective_agent: Some("claude-code@claude-b".to_string()),
            tab_id: None,
            hive_system: Some("claude-code".to_string()),
            hive_role: Some("agent".to_string()),
            worker_name: Some("claude-code".to_string()),
            display_name: None,
            role: Some("agent".to_string()),
            capabilities: vec!["memory".to_string()],
            hive_groups: vec!["openclaw-stack".to_string()],
            lane_id: Some(root.display().to_string()),
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: Some(default_heartbeat_model()),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            repo_root: Some(root.display().to_string()),
            worktree_root: Some(root.display().to_string()),
            branch: Some("feature/hive-shared".to_string()),
            base_branch: Some("master".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some(base_url.clone()),
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
            last_seen: Utc::now(),
            authority_mode: Some("shared".to_string()),
            authority_degraded: false,
        },
    );

    let err = run_tasks_command(
        &TasksArgs {
            output: current_bundle.clone(),
            upsert: false,
            assign_to_session: None,
            target_session: Some("claude-b".to_string()),
            task_id: Some("task-1".to_string()),
            title: Some("need review".to_string()),
            description: None,
            status: None,
            mode: None,
            scope: vec!["src/main.rs".to_string()],
            request_help: false,
            request_review: true,
            all: false,
            view: None,
            summary: false,
            json: false,
        },
        SHARED_MEMD_BASE_URL,
    )
    .await
    .expect_err("colliding review request should fail");
    assert!(
        err.to_string()
            .contains("unsafe hive cowork target collision")
    );

    fs::remove_dir_all(root).expect("cleanup tasks review collision dir");
}

#[tokio::test]
async fn hive_join_reroutes_colliding_worker_lane_into_new_worktree() {
    let root =
        std::env::temp_dir().join(format!("memd-hive-lane-reroute-{}", uuid::Uuid::new_v4()));
    let current_project = root.join("current");
    let target_project = root.join("target");
    let current_bundle = current_project.join(".memd");
    let target_bundle = target_project.join(".memd");
    fs::create_dir_all(&current_bundle).expect("create current bundle");
    fs::create_dir_all(&target_bundle).expect("create target bundle");
    fs::create_dir_all(current_project.join(".planning")).expect("create current planning");
    fs::create_dir_all(target_project.join(".planning")).expect("create target planning");
    let state = MockRuntimeState::default();
    let base_url = spawn_mock_runtime_server(state.clone(), false).await;

    write_test_bundle_config(&current_bundle, &base_url);
    fs::write(
        target_bundle.join("config.json"),
        format!(
            r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "claude-code",
  "session": "claude-b",
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
    .expect("write target config");
    fs::write(current_project.join("README.md"), "# current\n").expect("write readme");
    fs::write(target_project.join("NOTES.md"), "# target\n").expect("write notes");
    init_test_git_repo(&root);
    checkout_test_branch(&root, "feature/hive-shared");

    write_test_bundle_heartbeat(
        &target_bundle,
        &BundleHeartbeatState {
            session: Some("claude-b".to_string()),
            agent: Some("claude-code".to_string()),
            effective_agent: Some("claude-code@claude-b".to_string()),
            tab_id: None,
            hive_system: Some("claude-code".to_string()),
            hive_role: Some("agent".to_string()),
            worker_name: Some("claude-code".to_string()),
            display_name: None,
            role: Some("agent".to_string()),
            capabilities: vec!["memory".to_string()],
            hive_groups: vec!["openclaw-stack".to_string()],
            lane_id: Some(root.display().to_string()),
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: Some(default_heartbeat_model()),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            repo_root: Some(root.display().to_string()),
            worktree_root: Some(root.display().to_string()),
            branch: Some("feature/hive-shared".to_string()),
            base_branch: Some("master".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some(base_url.clone()),
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
            last_seen: Utc::now(),
            authority_mode: Some("shared".to_string()),
            authority_degraded: false,
        },
    );
    let conflict = detect_bundle_lane_collision(&current_bundle, Some("codex-a"))
        .await
        .expect("detect lane collision");
    assert!(conflict.is_some(), "expected lane collision before join");

    let response = run_hive_join_command(&HiveJoinArgs {
        output: current_bundle.clone(),
        base_url: base_url.clone(),
        all_active: false,
        all_local: false,
        publish_heartbeat: false,
        summary: false,
    })
    .await
    .expect("reroute join");

    let response = match response {
        HiveJoinResponse::Single(response) => response,
        other => panic!("expected single response, got {other:?}"),
    };
    let rerouted_output = PathBuf::from(&response.output);
    assert!(response.lane_rerouted);
    assert!(response.lane_created);
    assert!(response.lane_surface.is_some());
    assert_ne!(rerouted_output, current_bundle);
    assert!(rerouted_output.join("config.json").exists());

    let rerouted_project = rerouted_output
        .parent()
        .expect("rerouted bundle parent")
        .to_path_buf();
    let rerouted_branch =
        git_stdout(&rerouted_project, &["branch", "--show-current"]).expect("rerouted branch");
    assert_ne!(rerouted_branch, "feature/hive-shared");
    assert_ne!(
        detect_git_worktree_root(&rerouted_project).expect("rerouted worktree root"),
        detect_git_worktree_root(&current_project).expect("current worktree root")
    );

    let rerouted_runtime = read_bundle_runtime_config_raw(&rerouted_output)
        .expect("read rerouted runtime")
        .expect("rerouted runtime config");
    assert_ne!(rerouted_runtime.session.as_deref(), Some("codex-a"));
    let status = read_bundle_status(&rerouted_output, SHARED_MEMD_BASE_URL)
        .await
        .expect("read rerouted status");
    let lane = status.get("lane_surface").expect("lane surface present");
    assert_eq!(
        lane.get("action").and_then(JsonValue::as_str),
        Some("auto_reroute")
    );
    assert_eq!(
        lane.get("conflict_session").and_then(JsonValue::as_str),
        Some("claude-b")
    );
    let receipts = state.receipts.lock().expect("lock receipts");
    assert!(
        receipts
            .iter()
            .any(|receipt| receipt.kind == "lane_reroute")
    );

    fs::remove_dir_all(root).expect("cleanup hive reroute dir");
}

#[tokio::test]
async fn run_hive_command_reports_live_session_rebind() {
    let _home_lock = lock_home_mutation();
    let temp_root =
        std::env::temp_dir().join(format!("memd-hive-rebind-{}", uuid::Uuid::new_v4()));
    let home = temp_root.join("home");
    let repo_root = temp_root.join("repo");
    let global_root = home.join(".memd");
    let local_bundle = repo_root.join(".memd");
    fs::create_dir_all(global_root.join("state")).expect("create global state");
    fs::create_dir_all(local_bundle.join("state")).expect("create local state");
    fs::write(
        global_root.join("config.json"),
        format!(
            r#"{{
  "project": "global",
  "namespace": "global",
  "agent": "codex",
  "session": "codex-fresh",
  "tab_id": "tab-alpha",
  "base_url": "{SHARED_MEMD_BASE_URL}"
}}
"#
        ),
    )
    .expect("write global config");
    fs::write(
        local_bundle.join("config.json"),
        format!(
            r#"{{
  "project": "memd",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-stale",
  "base_url": "{SHARED_MEMD_BASE_URL}",
  "route": "auto",
  "intent": "current_task",
  "workspace": "shared",
  "visibility": "workspace"
}}
"#
        ),
    )
    .expect("write local config");

    let original_home = std::env::var_os("HOME");
    let original_dir = std::env::current_dir().expect("read cwd");
    unsafe {
        std::env::set_var("HOME", &home);
    }
    std::env::set_current_dir(&repo_root).expect("set repo cwd");

    let response = run_hive_command(&HiveArgs {
        command: None,
        agent: None,
        project: None,
        namespace: None,
        global: false,
        project_root: Some(repo_root.clone()),
        seed_existing: false,
        session: None,
        tab_id: None,
        hive_system: Some("codex".to_string()),
        hive_role: Some("agent".to_string()),
        capability: Vec::new(),
        hive_group: vec!["project:memd".to_string()],
        hive_group_goal: None,
        authority: Some("participant".to_string()),
        output: local_bundle.clone(),
        base_url: SHARED_MEMD_BASE_URL.to_string(),
        rag_url: None,
        route: "auto".to_string(),
        intent: "current_task".to_string(),
        workspace: Some("shared".to_string()),
        visibility: Some("workspace".to_string()),
        publish_heartbeat: false,
        force: false,
        summary: false,
    })
    .await
    .expect("run hive command");

    assert_eq!(response.bundle_session.as_deref(), Some("codex-stale"));
    assert_eq!(response.live_session.as_deref(), Some("codex-fresh"));
    assert_eq!(response.session.as_deref(), Some("codex-fresh"));
    assert_eq!(
        response.rebased_from_session.as_deref(),
        Some("codex-stale")
    );

    let summary = render_hive_wire_summary(&response);
    assert!(summary.contains("bundle_session=codex-stale"));
    assert!(summary.contains("live_session=codex-fresh"));
    assert!(summary.contains("rebased_from=codex-stale"));

    std::env::set_current_dir(&original_dir).expect("restore cwd");
    if let Some(value) = original_home {
        unsafe {
            std::env::set_var("HOME", value);
        }
    } else {
        unsafe {
            std::env::remove_var("HOME");
        }
    }
    fs::remove_dir_all(temp_root).expect("cleanup hive rebind temp");
}

#[tokio::test]
async fn run_hive_command_surfaces_lane_reroute() {
    let root =
        std::env::temp_dir().join(format!("memd-hive-wire-reroute-{}", uuid::Uuid::new_v4()));
    let current_project = root.join("current");
    let target_project = root.join("target");
    let current_bundle = current_project.join(".memd");
    let target_bundle = target_project.join(".memd");
    fs::create_dir_all(&current_bundle).expect("create current bundle");
    fs::create_dir_all(&target_bundle).expect("create target bundle");
    fs::create_dir_all(current_project.join(".planning")).expect("create current planning");
    fs::create_dir_all(target_project.join(".planning")).expect("create target planning");
    let state = MockRuntimeState::default();
    let base_url = spawn_mock_runtime_server(state.clone(), false).await;

    write_test_bundle_config(&current_bundle, &base_url);
    fs::write(
        target_bundle.join("config.json"),
        format!(
            r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "claude-code",
  "session": "claude-b",
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
    .expect("write target config");
    fs::write(current_project.join("README.md"), "# current\n").expect("write readme");
    fs::write(target_project.join("NOTES.md"), "# target\n").expect("write notes");
    init_test_git_repo(&root);
    checkout_test_branch(&root, "feature/hive-shared");
    write_test_bundle_heartbeat(
        &target_bundle,
        &BundleHeartbeatState {
            session: Some("claude-b".to_string()),
            agent: Some("claude-code".to_string()),
            effective_agent: Some("claude-code@claude-b".to_string()),
            tab_id: None,
            hive_system: Some("claude-code".to_string()),
            hive_role: Some("agent".to_string()),
            worker_name: Some("claude-code".to_string()),
            display_name: None,
            role: Some("agent".to_string()),
            capabilities: vec!["memory".to_string()],
            hive_groups: vec!["openclaw-stack".to_string()],
            lane_id: Some(root.display().to_string()),
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: Some(default_heartbeat_model()),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            repo_root: Some(root.display().to_string()),
            worktree_root: Some(root.display().to_string()),
            branch: Some("feature/hive-shared".to_string()),
            base_branch: Some("master".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some(base_url.clone()),
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
            last_seen: Utc::now(),
            authority_mode: Some("shared".to_string()),
            authority_degraded: false,
        },
    );

    let response = run_hive_command(&HiveArgs {
        command: None,
        agent: None,
        project: None,
        namespace: None,
        global: false,
        project_root: Some(current_project.clone()),
        seed_existing: false,
        session: None,
        tab_id: None,
        hive_system: Some("codex".to_string()),
        hive_role: Some("agent".to_string()),
        capability: Vec::new(),
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
        publish_heartbeat: false,
        force: false,
        summary: false,
    })
    .await
    .expect("run hive command");

    assert!(response.lane_rerouted);
    assert!(response.lane_created);
    assert!(response.lane_surface.is_some());
    assert_ne!(PathBuf::from(&response.output), current_bundle);

    let receipts = state.receipts.lock().expect("lock receipts");
    assert!(
        receipts
            .iter()
            .any(|receipt| receipt.kind == "lane_reroute")
    );

    fs::remove_dir_all(root).expect("cleanup hive reroute root");
}

#[tokio::test]
async fn run_tasks_command_blocks_mutating_writes_in_localhost_read_only_mode() {
    let dir = std::env::temp_dir().join(format!("memd-tasks-ro-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("create temp bundle");
    fs::write(
        dir.join("config.json"),
        format!(
            r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "{}",
  "route": "auto",
  "intent": "current_task",
  "authority_policy": {{
    "shared_primary": true,
    "localhost_fallback_policy": "allow_read_only"
  }},
  "authority_state": {{
    "mode": "localhost_read_only",
    "degraded": true,
    "shared_base_url": "{}",
    "fallback_base_url": "http://127.0.0.1:8787",
    "reason": "tailscale is unavailable"
  }}
}}
"#,
            SHARED_MEMD_BASE_URL, SHARED_MEMD_BASE_URL
        ),
    )
    .expect("write config");

    let err = run_tasks_command(
        &TasksArgs {
            output: dir.clone(),
            upsert: true,
            assign_to_session: None,
            target_session: None,
            task_id: Some("task-1".to_string()),
            title: Some("keep work moving".to_string()),
            description: Some("refresh the plan".to_string()),
            status: Some("open".to_string()),
            mode: None,
            scope: vec!["src/main.rs".to_string()],
            request_help: false,
            request_review: false,
            all: false,
            view: None,
            summary: false,
            json: false,
        },
        SHARED_MEMD_BASE_URL,
    )
    .await
    .expect_err("shared write in localhost read-only mode should fail");
    assert!(
        err.to_string()
            .contains("localhost read-only fallback active")
    );

    fs::remove_dir_all(dir).expect("cleanup temp bundle");
}

#[tokio::test]
async fn run_hive_command_auto_creates_isolated_worker_lane_for_new_bundle() {
    let root = std::env::temp_dir().join(format!("memd-hive-auto-create-{}", uuid::Uuid::new_v4()));
    let current_project = root.join("current");
    fs::create_dir_all(current_project.join(".planning")).expect("create current planning");
    fs::write(current_project.join("README.md"), "# current\n").expect("write readme");
    init_test_git_repo(&root);
    checkout_test_branch(&root, "feature/hive-shared");

    let state = MockRuntimeState::default();
    let base_url = spawn_mock_runtime_server(state.clone(), false).await;
    let response = run_hive_command(&HiveArgs {
        command: None,
        agent: Some("codex".to_string()),
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        global: false,
        project_root: Some(current_project.clone()),
        seed_existing: false,
        session: Some("codex-a".to_string()),
        tab_id: None,
        hive_system: Some("codex".to_string()),
        hive_role: Some("agent".to_string()),
        capability: Vec::new(),
        hive_group: vec!["project:demo".to_string()],
        hive_group_goal: None,
        authority: Some("participant".to_string()),
        output: default_init_output_path(),
        base_url: base_url.clone(),
        rag_url: None,
        route: "auto".to_string(),
        intent: "current_task".to_string(),
        workspace: Some("shared".to_string()),
        visibility: Some("workspace".to_string()),
        publish_heartbeat: false,
        force: false,
        summary: false,
    })
    .await
    .expect("run hive command");

    assert!(!response.lane_rerouted);
    assert!(response.lane_created);
    let lane = response.lane_surface.expect("lane surface");
    assert_eq!(
        lane.get("action").and_then(JsonValue::as_str),
        Some("auto_create")
    );
    let output = PathBuf::from(&response.output);
    assert_ne!(output, current_project.join(".memd"));
    assert!(output.join("config.json").exists());

    let receipts = state.receipts.lock().expect("lock receipts");
    assert!(receipts.iter().any(|receipt| receipt.kind == "lane_create"));

    fs::remove_dir_all(root).expect("cleanup hive auto create root");
}
