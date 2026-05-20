#[test]
fn init_infers_service_hive_profile_for_claw_control() {
    let args = InitArgs {
        project: None,
        namespace: None,
        global: false,
        project_root: None,
        seed_existing: false,
        agent: "claw-control".to_string(),
        session: None,
        tab_id: None,
        hive_system: None,
        hive_role: None,
        capability: Vec::new(),
        hive_group: Vec::new(),
        hive_group_goal: None,
        authority: None,
        output: default_init_output_path(),
        base_url: "http://127.0.0.1:8787".to_string(),
        rag_url: None,
        route: "auto".to_string(),
        intent: "current_task".to_string(),
        voice_mode: None,
        workspace: None,
        visibility: None,
        allow_localhost_read_only_fallback: false,
        force: false,
    };

    let profile = resolve_hive_profile(&args, None);
    assert_eq!(profile.hive_system.as_deref(), Some("claw-control"));
    assert_eq!(profile.hive_role.as_deref(), Some("orchestrator"));
    assert_eq!(profile.authority.as_deref(), Some("coordinator"));
    assert!(profile.capabilities.iter().any(|value| value == "control"));
    assert!(
        profile
            .capabilities
            .iter()
            .any(|value| value == "coordination")
    );
}

#[test]
fn infer_service_agent_from_project_root_name() {
    assert_eq!(
        infer_service_agent_from_path(Path::new("/tmp/clawcontrol-rollout")).as_deref(),
        Some("claw-control")
    );
    assert_eq!(
        infer_service_agent_from_path(Path::new("/tmp/clawcontrol-agentshell")).as_deref(),
        Some("agent-shell")
    );
    assert_eq!(
        infer_service_agent_from_path(Path::new("/tmp/clawcontrol-agent-secrets-v2")).as_deref(),
        Some("agent-secrets")
    );
    assert_eq!(
        infer_service_agent_from_path(Path::new("/tmp/workspace")).as_deref(),
        Some("openclaw")
    );
}

#[test]
fn infer_worker_agent_from_env_prefers_explicit_worker_name() {
    unsafe {
        std::env::set_var("MEMD_WORKER_NAME", "Avicenna");
    }
    assert_eq!(infer_worker_agent_from_env().as_deref(), Some("Avicenna"));
    unsafe {
        std::env::remove_var("MEMD_WORKER_NAME");
    }
}

#[test]
fn default_bundle_worker_name_prefers_session_backed_label_for_generic_agents() {
    assert_eq!(
        default_bundle_worker_name("codex", Some("session-6d422e56")),
        "Codex 6d422e56"
    );
    assert_eq!(
        default_bundle_worker_name("claude-code", Some("session-review-a")),
        "Claude review-a"
    );
    assert_eq!(
        default_bundle_worker_name("openclaw", Some("lane-a")),
        "Openclaw"
    );
}

#[test]
fn default_bundle_worker_name_for_project_prefers_project_scoped_label_for_generic_agents() {
    assert_eq!(
        default_bundle_worker_name_for_project(Some("memd"), "codex", Some("session-6d422e56")),
        "Memd Codex 6d422e56"
    );
    assert_eq!(
        default_bundle_worker_name_for_project(
            Some("demo"),
            "claude-code",
            Some("session-review-a")
        ),
        "Demo Claude review-a"
    );
    assert_eq!(
        default_bundle_worker_name_for_project(Some("memd"), "openclaw", Some("lane-a")),
        "Openclaw"
    );
}

#[test]
fn resolve_hive_command_base_url_prefers_global_bundle_override() {
    let _home_lock = lock_home_mutation();
    let home = std::env::temp_dir().join(format!("memd-home-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(home.join(".memd")).expect("create fake home bundle");
    fs::write(
        home.join(".memd").join("config.json"),
        r#"{
  "project": "global",
  "namespace": "global",
  "agent": "codex",
  "session": "codex-a",
  "base_url": "http://100.104.154.24:8788"
}
"#,
    )
    .expect("write fake global config");
    let original_home = std::env::var_os("HOME");
    unsafe {
        std::env::set_var("HOME", &home);
    }

    let resolved = resolve_hive_command_base_url(SHARED_MEMD_BASE_URL);
    assert_eq!(resolved, "http://100.104.154.24:8788");

    if let Some(value) = original_home {
        unsafe {
            std::env::set_var("HOME", value);
        }
    } else {
        unsafe {
            std::env::remove_var("HOME");
        }
    }
    fs::remove_dir_all(home).expect("cleanup fake home");
}

#[test]
fn default_base_url_prefers_global_bundle_override() {
    let _home_lock = lock_home_mutation();
    let home = std::env::temp_dir().join(format!("memd-default-home-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(home.join(".memd")).expect("create fake home bundle");
    fs::write(
        home.join(".memd").join("config.json"),
        r#"{
  "project": "global",
  "namespace": "global",
  "agent": "codex",
  "session": "codex-a",
  "base_url": "http://100.104.154.24:8788"
}
"#,
    )
    .expect("write fake global config");
    let original_home = std::env::var_os("HOME");
    let original_base_url = std::env::var_os("MEMD_BASE_URL");
    unsafe {
        std::env::set_var("HOME", &home);
        std::env::remove_var("MEMD_BASE_URL");
    }

    let resolved = default_base_url();
    assert_eq!(resolved, "http://100.104.154.24:8788");

    if let Some(value) = original_home {
        unsafe {
            std::env::set_var("HOME", value);
        }
    } else {
        unsafe {
            std::env::remove_var("HOME");
        }
    }
    if let Some(value) = original_base_url {
        unsafe {
            std::env::set_var("MEMD_BASE_URL", value);
        }
    }
    fs::remove_dir_all(home).expect("cleanup fake home");
}

#[test]
fn resolve_bundle_command_base_url_prefers_runtime_over_env_default() {
    let original_base_url = std::env::var_os("MEMD_BASE_URL");
    unsafe {
        std::env::set_var("MEMD_BASE_URL", "http://127.0.0.1:8787");
    }

    let resolved = resolve_bundle_command_base_url(
        "http://127.0.0.1:8787",
        Some("http://100.104.154.24:8788"),
    );
    assert_eq!(resolved, "http://100.104.154.24:8788");

    if let Some(value) = original_base_url {
        unsafe {
            std::env::set_var("MEMD_BASE_URL", value);
        }
    } else {
        unsafe {
            std::env::remove_var("MEMD_BASE_URL");
        }
    }
}

#[test]
fn resolve_bundle_command_base_url_honors_explicit_non_default_request() {
    let original_base_url = std::env::var_os("MEMD_BASE_URL");
    unsafe {
        std::env::set_var("MEMD_BASE_URL", "http://127.0.0.1:8787");
    }

    let resolved = resolve_bundle_command_base_url(
        "http://127.0.0.1:9797",
        Some("http://100.104.154.24:8788"),
    );
    assert_eq!(resolved, "http://127.0.0.1:9797");

    if let Some(value) = original_base_url {
        unsafe {
            std::env::set_var("MEMD_BASE_URL", value);
        }
    } else {
        unsafe {
            std::env::remove_var("MEMD_BASE_URL");
        }
    }
}

#[test]
fn resolve_project_bundle_overlay_uses_local_bundle_from_global_root() {
    let temp_root =
        std::env::temp_dir().join(format!("memd-overlay-root-{}", uuid::Uuid::new_v4()));
    let global_root = temp_root.join("global");
    let repo_root = temp_root.join("repo");
    let local_bundle = repo_root.join(".memd");
    fs::create_dir_all(&local_bundle).expect("create local bundle");
    fs::write(
        local_bundle.join("config.json"),
        r#"{
  "project": "memd",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "base_url": "http://127.0.0.1:8787",
  "route": "lexical",
  "intent": "current_task",
  "workspace": "team-alpha",
  "visibility": "workspace"
}
"#,
    )
    .expect("write local config");

    let overlay = resolve_project_bundle_overlay(&global_root, &repo_root, &global_root)
        .expect("resolve overlay")
        .expect("overlay present");
    assert_eq!(overlay.project.as_deref(), Some("memd"));
    assert_eq!(overlay.namespace.as_deref(), Some("main"));
    assert_eq!(overlay.route.as_deref(), Some("lexical"));
    assert_eq!(overlay.intent.as_deref(), Some("current_task"));

    fs::remove_dir_all(temp_root).expect("cleanup overlay temp");
}

#[test]
fn resolve_live_session_overlay_uses_global_session_for_current_project_bundle() {
    let temp_root =
        std::env::temp_dir().join(format!("memd-live-overlay-{}", uuid::Uuid::new_v4()));
    let home = temp_root.join("home");
    let repo_root = temp_root.join("repo");
    let global_root = home.join(".memd");
    let local_bundle = repo_root.join(".memd");
    fs::create_dir_all(&global_root).expect("create global bundle");
    fs::create_dir_all(&local_bundle).expect("create local bundle");
    fs::write(
        global_root.join("config.json"),
        r#"{
  "project": "global",
  "namespace": "global",
  "agent": "codex",
  "session": "codex-fresh",
  "tab_id": "tab-alpha",
  "base_url": "http://100.104.154.24:8788"
}
"#,
    )
    .expect("write global config");
    fs::write(
        local_bundle.join("config.json"),
        r#"{
  "project": "memd",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-stale",
  "base_url": "http://100.104.154.24:8788",
  "workspace": "shared",
  "visibility": "workspace"
}
"#,
    )
    .expect("write local config");

    let _cwd = set_current_dir(&repo_root);

    let overlay = resolve_live_session_overlay(&local_bundle, &repo_root, &global_root)
        .expect("resolve live overlay")
        .expect("overlay present");
    assert_eq!(overlay.session.as_deref(), Some("codex-fresh"));
    assert_eq!(overlay.tab_id.as_deref(), Some("tab-alpha"));

    drop(_cwd);
    fs::remove_dir_all(temp_root).expect("cleanup live overlay temp");
}

#[test]
fn resolve_live_session_overlay_skips_plain_local_bundle_without_hive_scope() {
    let temp_root = std::env::temp_dir().join(format!(
        "memd-live-overlay-no-scope-{}",
        uuid::Uuid::new_v4()
    ));
    let home = temp_root.join("home");
    let repo_root = temp_root.join("repo");
    let global_root = home.join(".memd");
    let local_bundle = repo_root.join(".memd");
    fs::create_dir_all(&global_root).expect("create global bundle");
    fs::create_dir_all(&local_bundle).expect("create local bundle");
    fs::write(
        global_root.join("config.json"),
        r#"{
  "project": "global",
  "namespace": "global",
  "agent": "codex",
  "session": "codex-fresh",
  "tab_id": "tab-alpha",
  "base_url": "http://100.104.154.24:8788"
}
"#,
    )
    .expect("write global config");
    fs::write(
        local_bundle.join("config.json"),
        r#"{
  "project": "memd",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-stale",
  "base_url": "http://100.104.154.24:8788",
  "route": "auto",
  "intent": "current_task"
}
"#,
    )
    .expect("write local config");

    let _cwd = set_current_dir(&repo_root);

    let overlay = resolve_live_session_overlay(&local_bundle, &repo_root, &global_root)
        .expect("resolve live overlay");
    assert!(overlay.is_none());

    drop(_cwd);
    fs::remove_dir_all(temp_root).expect("cleanup live overlay temp");
}

#[tokio::test]
async fn run_coordination_command_records_queen_decisions() {
    let dir =
        std::env::temp_dir().join(format!("memd-coordination-queen-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(dir.join("state")).expect("create temp dir");
    let state = MockRuntimeState::default();
    let base_url = spawn_mock_runtime_server(state.clone(), false).await;
    fs::write(
        dir.join("config.json"),
        format!(
            r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "queen-a",
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

    let response = run_coordination_command(
        &CoordinationArgs {
            output: dir.clone(),
            view: Some("lanes".to_string()),
            changes_only: false,
            watch: false,
            interval_secs: 30,
            recover_session: None,
            retire_session: None,
            to_session: Some("bee-b".to_string()),
            deny_session: Some("bee-b".to_string()),
            reroute_session: Some("bee-c".to_string()),
            handoff_scope: Some("file:src/main.rs".to_string()),
            summary: false,
        },
        &base_url,
    )
    .await
    .expect("coordination response");

    assert!(
        response
            .lane_receipts
            .iter()
            .any(|receipt| receipt.kind == "queen_deny")
    );
    assert!(
        response
            .lane_receipts
            .iter()
            .any(|receipt| receipt.kind == "queen_reroute")
    );
    assert!(
        response
            .lane_receipts
            .iter()
            .any(|receipt| receipt.kind == "queen_handoff")
    );

    fs::remove_dir_all(dir).expect("cleanup temp dir");
}

#[tokio::test]
async fn run_coordination_command_falls_back_to_local_truth_when_backend_unreachable() {
    let dir = std::env::temp_dir().join(format!(
        "memd-coordination-offline-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(dir.join("state")).expect("create temp dir");
    fs::write(
        dir.join("config.json"),
        r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "queen-a",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "http://127.0.0.1:9",
  "route": "auto",
  "intent": "current_task"
}
"#,
    )
    .expect("write config");

    let response = run_coordination_command(
        &CoordinationArgs {
            output: dir.clone(),
            view: Some("overview".to_string()),
            changes_only: false,
            watch: false,
            interval_secs: 30,
            recover_session: None,
            retire_session: None,
            to_session: None,
            deny_session: None,
            reroute_session: None,
            handoff_scope: None,
            summary: false,
        },
        "http://127.0.0.1:9",
    )
    .await
    .expect("offline coordination response");

    assert_eq!(response.current_session, "queen-a");
    assert!(response.inbox.messages.is_empty());
    assert!(response.inbox.owned_tasks.is_empty());
    assert!(response.receipts.is_empty());

    fs::remove_dir_all(dir).expect("cleanup temp dir");
}

#[tokio::test]
async fn run_coordination_command_fails_fast_for_mutations_when_backend_unreachable() {
    let dir = std::env::temp_dir().join(format!(
        "memd-coordination-offline-mutation-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(dir.join("state")).expect("create temp dir");
    fs::write(
        dir.join("config.json"),
        r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "queen-a",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "http://127.0.0.1:9",
  "route": "auto",
  "intent": "current_task"
}
"#,
    )
    .expect("write config");

    let err = run_coordination_command(
        &CoordinationArgs {
            output: dir.clone(),
            view: Some("overview".to_string()),
            changes_only: false,
            watch: false,
            interval_secs: 30,
            recover_session: None,
            retire_session: None,
            to_session: None,
            deny_session: Some("bee-b".to_string()),
            reroute_session: None,
            handoff_scope: None,
            summary: false,
        },
        "http://127.0.0.1:9",
    )
    .await
    .expect_err("offline mutation should fail fast");

    assert!(
        err.to_string().contains("coordination backend unreachable"),
        "{err}"
    );

    fs::remove_dir_all(dir).expect("cleanup temp dir");
}

#[tokio::test]
async fn read_bundle_status_reports_live_session_rebind() {
    let _home_lock = lock_home_mutation();
    let temp_root =
        std::env::temp_dir().join(format!("memd-status-rebind-{}", uuid::Uuid::new_v4()));
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
    unsafe {
        std::env::set_var("HOME", &home);
    }
    let _cwd = set_current_dir(&repo_root);

    let status = read_bundle_status(&local_bundle, SHARED_MEMD_BASE_URL)
        .await
        .expect("read bundle status");
    let overlay = status
        .get("session_overlay")
        .expect("session overlay present");
    assert_eq!(
        overlay
            .get("bundle_session")
            .and_then(serde_json::Value::as_str),
        Some("codex-stale")
    );
    assert!(
        overlay
            .get("live_session")
            .and_then(serde_json::Value::as_str)
            .is_some(),
        "expected live session overlay"
    );
    assert_eq!(
        overlay
            .get("rebased_from")
            .and_then(serde_json::Value::as_str),
        Some("codex-stale")
    );

    drop(_cwd);
    if let Some(value) = original_home {
        unsafe {
            std::env::set_var("HOME", value);
        }
    } else {
        unsafe {
            std::env::remove_var("HOME");
        }
    }
    cleanup_temp_dir(temp_root, "cleanup status rebind temp");
}
