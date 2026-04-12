use super::*;

#[tokio::test]
async fn hive_join_forces_shared_base_url_for_stale_bundle() {
    let dir = std::env::temp_dir().join(format!("memd-hive-join-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("create temp bundle");
    fs::write(
        dir.join("config.json"),
        r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "current_task",
  "hive_groups": ["openclaw-stack"]
}
"#,
    )
    .expect("write config");

    let response = run_hive_join_command(&HiveJoinArgs {
        output: dir.clone(),
        base_url: default_hive_join_base_url(),
        all_active: false,
        all_local: false,
        publish_heartbeat: false,
        summary: false,
    })
    .await
    .expect("join hive");

    let response = match response {
        HiveJoinResponse::Single(response) => response,
        other => panic!("expected single response, got {other:?}"),
    };
    assert_eq!(response.base_url, SHARED_MEMD_BASE_URL);
    assert_eq!(response.session.as_deref(), Some("codex-a"));
    let config = fs::read_to_string(dir.join("config.json")).expect("read config");
    let env = fs::read_to_string(dir.join("env")).expect("read env");
    assert!(config.contains(r#""base_url": "http://100.104.154.24:8787""#));
    assert!(env.contains("MEMD_BASE_URL=http://100.104.154.24:8787"));

    fs::remove_dir_all(dir).expect("cleanup temp bundle");
}

#[tokio::test]
async fn hive_join_all_active_rewrites_live_bundles_in_project() {
    let root = std::env::temp_dir().join(format!("memd-hive-join-all-{}", uuid::Uuid::new_v4()));
    let current_project = root.join("alpha");
    let sibling_project = root.join("beta");
    let current_bundle = current_project.join(".memd");
    let sibling_bundle = sibling_project.join(".memd");
    fs::create_dir_all(&current_bundle).expect("create current bundle");
    fs::create_dir_all(&sibling_bundle).expect("create sibling bundle");

    for bundle_root in [&current_bundle, &sibling_bundle] {
        fs::create_dir_all(bundle_root.join("state")).expect("create state dir");
        fs::write(
            bundle_root.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "{}",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "http://127.0.0.1:8787",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task",
  "hive_groups": ["openclaw-stack"]
}}
"#,
                bundle_root
                    .parent()
                    .and_then(|path| path.file_name())
                    .and_then(|value| value.to_str())
                    .unwrap_or("bundle")
            ),
        )
        .expect("write config");
        let heartbeat = build_hive_heartbeat(bundle_root, None).expect("build heartbeat");
        fs::write(
            bundle_heartbeat_state_path(bundle_root),
            serde_json::to_string_pretty(&heartbeat).expect("serialize heartbeat") + "\n",
        )
        .expect("write heartbeat");
    }

    let response = run_hive_join_command(&HiveJoinArgs {
        output: current_bundle.clone(),
        base_url: default_hive_join_base_url(),
        all_active: true,
        all_local: false,
        publish_heartbeat: false,
        summary: false,
    })
    .await
    .expect("join all active");

    match response {
        HiveJoinResponse::Batch(batch) => {
            assert_eq!(batch.base_url, SHARED_MEMD_BASE_URL);
            assert_eq!(batch.joined.len(), 2);
        }
        other => panic!("expected batch response, got {other:?}"),
    }

    for bundle_root in [&current_bundle, &sibling_bundle] {
        let config = fs::read_to_string(bundle_root.join("config.json")).expect("read config");
        let env = fs::read_to_string(bundle_root.join("env")).expect("read env");
        assert!(config.contains(r#""base_url": "http://100.104.154.24:8787""#));
        assert!(env.contains("MEMD_BASE_URL=http://100.104.154.24:8787"));
    }

    fs::remove_dir_all(root).expect("cleanup project root");
}

#[tokio::test]
async fn hive_command_propagates_hive_metadata_to_active_sibling_bundles() {
    let root = std::env::temp_dir().join(format!("memd-hive-propagate-{}", uuid::Uuid::new_v4()));
    let current_project = root.join("alpha");
    let sibling_project = root.join("beta");
    let current_bundle = current_project.join(".memd");
    let sibling_bundle = sibling_project.join(".memd");
    fs::create_dir_all(current_bundle.join("state")).expect("create current state dir");
    fs::create_dir_all(sibling_bundle.join("state")).expect("create sibling state dir");

    fs::write(
        current_bundle.join("config.json"),
        r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "tab_id": "tab-a",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "http://127.0.0.1:8787",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task"
}
"#,
    )
    .expect("write current config");
    fs::write(
        sibling_bundle.join("config.json"),
        r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-b",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "http://127.0.0.1:8787",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task"
}
"#,
    )
    .expect("write sibling config");
    write_test_bundle_heartbeat(
        &current_bundle,
        &test_hive_heartbeat_state("codex-a", "codex", "tab-a", "live", Utc::now()),
    );
    write_test_bundle_heartbeat(
        &sibling_bundle,
        &test_hive_heartbeat_state("codex-b", "codex", "tab-b", "live", Utc::now()),
    );

    let response = run_hive_command(&HiveArgs {
        command: None,
        global: false,
        project_root: None,
        seed_existing: false,
        project: None,
        namespace: None,
        agent: None,
        session: None,
        tab_id: None,
        hive_system: Some("codex".to_string()),
        hive_role: Some("agent".to_string()),
        capability: Vec::new(),
        hive_group: vec!["project:demo".to_string()],
        hive_group_goal: None,
        authority: Some("participant".to_string()),
        output: current_bundle.clone(),
        base_url: SHARED_MEMD_BASE_URL.to_string(),
        rag_url: None,
        route: "auto".to_string(),
        intent: "current_task".to_string(),
        workspace: Some("shared".to_string()),
        visibility: Some("workspace".to_string()),
        publish_heartbeat: true,
        force: false,
        summary: false,
    })
    .await
    .expect("run hive command");

    assert_eq!(response.hive_system.as_deref(), Some("codex"));
    assert_eq!(response.hive_role.as_deref(), Some("agent"));

    let sibling_runtime = read_bundle_runtime_config(&sibling_bundle)
        .expect("read sibling runtime")
        .expect("sibling runtime config");
    assert_eq!(sibling_runtime.hive_system.as_deref(), Some("codex"));
    assert_eq!(sibling_runtime.hive_role.as_deref(), Some("agent"));
    assert_eq!(sibling_runtime.authority.as_deref(), Some("participant"));
    assert_eq!(
        sibling_runtime.base_url.as_deref(),
        Some(SHARED_MEMD_BASE_URL)
    );
    assert!(
        sibling_runtime
            .hive_groups
            .iter()
            .any(|group| group == "project:demo")
    );

    let sibling_heartbeat = read_bundle_heartbeat(&sibling_bundle)
        .expect("read sibling heartbeat")
        .expect("sibling heartbeat");
    assert_eq!(sibling_heartbeat.hive_system.as_deref(), Some("codex"));
    assert_eq!(sibling_heartbeat.hive_role.as_deref(), Some("agent"));
    assert_eq!(sibling_heartbeat.authority.as_deref(), Some("participant"));
    assert!(
        sibling_heartbeat
            .hive_groups
            .iter()
            .any(|group| group == "project:demo")
    );

    fs::remove_dir_all(root).expect("cleanup project root");
}

#[tokio::test]
async fn hive_join_all_local_rewrites_all_local_bundles_in_project() {
    let root = std::env::temp_dir().join(format!("memd-hive-join-local-{}", uuid::Uuid::new_v4()));
    let current_project = root.join("alpha");
    let sibling_project = root.join("beta");
    let current_bundle = current_project.join(".memd");
    let sibling_bundle = sibling_project.join(".memd");
    fs::create_dir_all(&current_bundle).expect("create current bundle");
    fs::create_dir_all(&sibling_bundle).expect("create sibling bundle");

    for (bundle_root, session) in [(&current_bundle, "codex-a"), (&sibling_bundle, "codex-b")] {
        fs::write(
            bundle_root.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "{}",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "http://127.0.0.1:8787",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task",
  "hive_groups": ["openclaw-stack"]
}}
"#,
                session
            ),
        )
        .expect("write config");
    }

    let response = run_hive_join_command(&HiveJoinArgs {
        output: current_bundle.clone(),
        base_url: default_hive_join_base_url(),
        all_active: false,
        all_local: true,
        publish_heartbeat: false,
        summary: false,
    })
    .await
    .expect("join all local");

    let batch = match response {
        HiveJoinResponse::Batch(batch) => batch,
        other => panic!("expected batch response, got {other:?}"),
    };
    assert_eq!(batch.base_url, SHARED_MEMD_BASE_URL);
    assert_eq!(batch.mode, "all-local");
    assert_eq!(batch.joined.len(), 2);

    for bundle_root in [&current_bundle, &sibling_bundle] {
        let config = fs::read_to_string(bundle_root.join("config.json")).expect("read config");
        let env = fs::read_to_string(bundle_root.join("env")).expect("read env");
        assert!(config.contains(r#""base_url": "http://100.104.154.24:8787""#));
        assert!(env.contains("MEMD_BASE_URL=http://100.104.154.24:8787"));
    }

    fs::remove_dir_all(root).expect("cleanup project root");
}

#[test]
fn set_bundle_route_and_intent_update_config_and_env_files() {
    let dir = std::env::temp_dir().join(format!("memd-bundle-intent-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("create temp bundle");
    fs::write(
        dir.join("config.json"),
        r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "general"
}
"#,
    )
    .expect("write config");
    fs::write(dir.join("env"), "MEMD_ROUTE=auto\nMEMD_INTENT=general\n").expect("write env");
    fs::write(
        dir.join("env.ps1"),
        "$env:MEMD_ROUTE = \"auto\"\n$env:MEMD_INTENT = \"general\"\n",
    )
    .expect("write env.ps1");

    set_bundle_route(&dir, "lexical").expect("set bundle route");
    set_bundle_intent(&dir, "current_task").expect("set bundle intent");

    let config = fs::read_to_string(dir.join("config.json")).expect("read config");
    let env = fs::read_to_string(dir.join("env")).expect("read env");
    let env_ps1 = fs::read_to_string(dir.join("env.ps1")).expect("read env.ps1");
    assert!(config.contains(r#""route": "lexical""#));
    assert!(config.contains(r#""intent": "current_task""#));
    assert!(env.contains("MEMD_ROUTE=lexical"));
    assert!(env.contains("MEMD_INTENT=current_task"));
    assert!(env_ps1.contains("$env:MEMD_ROUTE = \"lexical\""));
    assert!(env_ps1.contains("$env:MEMD_INTENT = \"current_task\""));

    fs::remove_dir_all(dir).expect("cleanup temp bundle");
}

#[test]
fn set_bundle_hive_metadata_updates_config_and_env_files() {
    let dir = std::env::temp_dir().join(format!("memd-bundle-hive-meta-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("create temp bundle");
    fs::write(
        dir.join("config.json"),
        r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "general"
}
"#,
    )
    .expect("write config");
    fs::write(dir.join("env"), "").expect("write env");
    fs::write(dir.join("env.ps1"), "").expect("write env.ps1");

    set_bundle_hive_system(&dir, "agent-shell").expect("set hive system");
    set_bundle_hive_role(&dir, "runtime-shell").expect("set hive role");
    set_bundle_capabilities(&dir, &["shell".to_string(), "exec".to_string()])
        .expect("set capabilities");
    set_bundle_hive_groups(
        &dir,
        &["runtime-core".to_string(), "dependency-owners".to_string()],
    )
    .expect("set hive groups");
    set_bundle_hive_group_goal(&dir, "stabilize runtime dependencies").expect("set group goal");
    set_bundle_authority(&dir, "worker").expect("set authority");

    let config = fs::read_to_string(dir.join("config.json")).expect("read config");
    let env = fs::read_to_string(dir.join("env")).expect("read env");
    let env_ps1 = fs::read_to_string(dir.join("env.ps1")).expect("read env.ps1");
    assert!(config.contains(r#""hive_system": "agent-shell""#));
    assert!(config.contains(r#""hive_role": "runtime-shell""#));
    assert!(config.contains(r#""hive_group_goal": "stabilize runtime dependencies""#));
    assert!(config.contains(r#""authority": "worker""#));
    assert!(config.contains(r#""hive_groups": ["#) || config.contains("\"hive_groups\": ["));
    assert!(env.contains("MEMD_PEER_SYSTEM=agent-shell"));
    assert!(env.contains("MEMD_PEER_ROLE=runtime-shell"));
    assert!(
        env.contains("MEMD_PEER_GROUPS=dependency-owners,runtime-core")
            || env.contains("MEMD_PEER_GROUPS=runtime-core,dependency-owners")
    );
    assert!(env.contains("MEMD_PEER_GROUP_GOAL=stabilize runtime dependencies"));
    assert!(env.contains("MEMD_PEER_AUTHORITY=worker"));
    assert!(env_ps1.contains("$env:MEMD_PEER_SYSTEM = \"agent-shell\""));
    assert!(env_ps1.contains("$env:MEMD_PEER_ROLE = \"runtime-shell\""));
    assert!(env_ps1.contains("$env:MEMD_PEER_GROUP_GOAL = \"stabilize runtime dependencies\""));
    assert!(env_ps1.contains("$env:MEMD_PEER_AUTHORITY = \"worker\""));

    fs::remove_dir_all(dir).expect("cleanup temp bundle");
}

#[test]
fn hive_project_command_is_exposed_in_cli_help() {
    use clap::CommandFactory;

    let mut command = Cli::command();
    let help = command
        .find_subcommand_mut("hive-project")
        .expect("hive-project command")
        .render_long_help()
        .to_string();
    assert!(help.contains("hive-project"));
    assert!(help.contains("--enable"));
    assert!(help.contains("--disable"));
    assert!(help.contains("--status"));
}

#[tokio::test]
async fn hive_project_enable_and_disable_update_bundle_state() {
    let dir = std::env::temp_dir().join(format!("memd-hive-project-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("create temp bundle");
    fs::write(
        dir.join("config.json"),
        r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "current_task"
}
"#,
    )
    .expect("write config");
    fs::write(dir.join("env"), "").expect("write env");
    fs::write(dir.join("env.ps1"), "").expect("write env.ps1");

    let enabled = run_hive_project_command(&HiveProjectArgs {
        output: dir.clone(),
        enable: true,
        disable: false,
        status: false,
        summary: false,
    })
    .await
    .expect("enable hive project");
    assert!(enabled.enabled);
    assert_eq!(enabled.anchor.as_deref(), Some("project:demo"));

    let config = fs::read_to_string(dir.join("config.json")).expect("read config");
    assert!(config.contains("\"hive_project_enabled\": true"));
    assert!(config.contains("\"hive_project_anchor\": \"project:demo\""));

    let disabled = run_hive_project_command(&HiveProjectArgs {
        output: dir.clone(),
        enable: false,
        disable: true,
        status: false,
        summary: false,
    })
    .await
    .expect("disable hive project");
    assert!(!disabled.enabled);
    assert!(disabled.anchor.is_none());

    let config = fs::read_to_string(dir.join("config.json")).expect("read config");
    assert!(config.contains("\"hive_project_enabled\": false"));
    assert!(!config.contains("\"hive_project_anchor\": \"project:demo\""));

    fs::remove_dir_all(dir).expect("cleanup temp bundle");
}

#[tokio::test]
async fn hive_project_enable_then_hive_join_then_hive_fix_all_work_together() {
    let dir = std::env::temp_dir().join(format!("memd-hive-project-e2e-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("create temp bundle");
    fs::write(
        dir.join("config.json"),
        r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "current_task"
}
"#,
    )
    .expect("write config");
    fs::write(dir.join("env"), "").expect("write env");
    fs::write(dir.join("env.ps1"), "").expect("write env.ps1");

    let enabled = run_hive_project_command(&HiveProjectArgs {
        output: dir.clone(),
        enable: true,
        disable: false,
        status: false,
        summary: false,
    })
    .await
    .expect("enable hive project");
    assert!(enabled.enabled);
    assert_eq!(enabled.anchor.as_deref(), Some("project:demo"));
    assert_eq!(enabled.live_session.as_deref(), Some("codex-a"));

    let shell = render_agent_shell_profile(&dir, Some("codex"));
    let attach = render_attach_snippet("bash", &dir).expect("attach snippet");
    assert!(shell.contains(SHARED_MEMD_BASE_URL));
    assert!(attach.contains(SHARED_MEMD_BASE_URL));

    let joined = run_hive_join_command(&HiveJoinArgs {
        output: dir.clone(),
        base_url: "http://127.0.0.1:8787".to_string(),
        all_active: false,
        all_local: false,
        publish_heartbeat: false,
        summary: false,
    })
    .await
    .expect("join hive");
    let single = match joined {
        HiveJoinResponse::Single(response) => response,
        other => panic!("expected single response, got {other:?}"),
    };
    assert_eq!(single.base_url, SHARED_MEMD_BASE_URL);

    let runtime = read_bundle_runtime_config(&dir)
        .expect("reload bundle runtime config")
        .expect("bundle runtime config");
    assert!(runtime.hive_project_enabled);
    assert_eq!(runtime.hive_project_anchor.as_deref(), Some("project:demo"));
    assert_eq!(runtime.base_url.as_deref(), Some(SHARED_MEMD_BASE_URL));

    let config = fs::read_to_string(dir.join("config.json")).expect("read config");
    assert!(config.contains(r#""hive_project_enabled": true"#));
    assert!(config.contains(r#""base_url": "http://100.104.154.24:8787""#));

    fs::remove_dir_all(dir).expect("cleanup temp bundle");
}

#[test]
fn set_bundle_scope_metadata_updates_config_and_env_files() {
    let dir = std::env::temp_dir().join(format!("memd-bundle-scope-meta-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("create temp bundle");
    fs::write(
        dir.join("config.json"),
        r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "general"
}
"#,
    )
    .expect("write config");
    fs::write(dir.join("env"), "").expect("write env");
    fs::write(dir.join("env.ps1"), "").expect("write env.ps1");

    set_bundle_project(&dir, "clawcontrol-rollout").expect("set project");
    set_bundle_namespace(&dir, "main").expect("set namespace");
    set_bundle_workspace(&dir, "openclaw-stack").expect("set workspace");
    set_bundle_visibility(&dir, "workspace").expect("set visibility");

    let config = fs::read_to_string(dir.join("config.json")).expect("read config");
    let env = fs::read_to_string(dir.join("env")).expect("read env");
    let env_ps1 = fs::read_to_string(dir.join("env.ps1")).expect("read env.ps1");
    assert!(config.contains(r#""project": "clawcontrol-rollout""#));
    assert!(config.contains(r#""namespace": "main""#));
    assert!(config.contains(r#""workspace": "openclaw-stack""#));
    assert!(config.contains(r#""visibility": "workspace""#));
    assert!(env.contains("MEMD_PROJECT=clawcontrol-rollout"));
    assert!(env.contains("MEMD_NAMESPACE=main"));
    assert!(env.contains("MEMD_WORKSPACE=openclaw-stack"));
    assert!(env.contains("MEMD_VISIBILITY=workspace"));
    assert!(env_ps1.contains("$env:MEMD_PROJECT = \"clawcontrol-rollout\""));
    assert!(env_ps1.contains("$env:MEMD_NAMESPACE = \"main\""));
    assert!(env_ps1.contains("$env:MEMD_WORKSPACE = \"openclaw-stack\""));
    assert!(env_ps1.contains("$env:MEMD_VISIBILITY = \"workspace\""));

    fs::remove_dir_all(dir).expect("cleanup temp bundle");
}

#[test]
fn write_init_bundle_persists_authority_policy_state_and_env_files() {
    let project_root =
        std::env::temp_dir().join(format!("memd-init-root-{}", uuid::Uuid::new_v4()));
    let output = std::env::temp_dir().join(format!("memd-init-output-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&project_root).expect("create project root");

    write_init_bundle(&InitArgs {
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        global: false,
        project_root: Some(project_root.clone()),
        seed_existing: false,
        agent: "codex".to_string(),
        session: Some("codex-a".to_string()),
        tab_id: None,
        hive_system: Some("codex".to_string()),
        hive_role: Some("agent".to_string()),
        capability: vec!["memory".to_string()],
        hive_group: vec!["project:demo".to_string()],
        hive_group_goal: Some("keep the bundle safe".to_string()),
        authority: Some("participant".to_string()),
        output: output.clone(),
        base_url: SHARED_MEMD_BASE_URL.to_string(),
        rag_url: None,
        route: "auto".to_string(),
        intent: "current_task".to_string(),
        voice_mode: None,
        workspace: Some("shared".to_string()),
        visibility: Some("workspace".to_string()),
        allow_localhost_read_only_fallback: false,
        force: true,
    })
    .expect("write init bundle");

    let config = fs::read_to_string(output.join("config.json")).expect("read config");
    let env = fs::read_to_string(output.join("env")).expect("read env");
    let env_ps1 = fs::read_to_string(output.join("env.ps1")).expect("read env.ps1");
    assert!(config.contains(r#""authority_policy""#));
    assert!(config.contains(r#""localhost_fallback_policy": "deny""#));
    assert!(config.contains(r#""authority_state""#));
    assert!(config.contains(r#""mode": "shared""#));
    assert!(config.contains(&format!(r#""shared_base_url": "{}""#, SHARED_MEMD_BASE_URL)));
    assert!(env.contains("MEMD_AUTHORITY_MODE=shared"));
    assert!(env.contains("MEMD_LOCALHOST_FALLBACK_POLICY=deny"));
    assert!(env.contains(&format!("MEMD_SHARED_BASE_URL={SHARED_MEMD_BASE_URL}")));
    assert!(env.contains("MEMD_AUTHORITY_DEGRADED=false"));
    assert!(env_ps1.contains("$env:MEMD_AUTHORITY_MODE = \"shared\""));
    assert!(env_ps1.contains("$env:MEMD_LOCALHOST_FALLBACK_POLICY = \"deny\""));
    assert!(env_ps1.contains(&format!(
        "$env:MEMD_SHARED_BASE_URL = \"{}\"",
        SHARED_MEMD_BASE_URL
    )));
    assert!(env_ps1.contains("$env:MEMD_AUTHORITY_DEGRADED = \"false\""));

    fs::remove_dir_all(project_root).expect("cleanup init project root");
    fs::remove_dir_all(output).expect("cleanup init output");
}

#[test]
fn render_agent_profiles_surface_authority_warning_when_localhost_fallback_is_active() {
    let dir = std::env::temp_dir().join(format!("memd-authority-profile-{}", uuid::Uuid::new_v4()));
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
    fs::write(dir.join("env"), "").expect("write env");
    fs::write(dir.join("env.ps1"), "").expect("write env.ps1");

    let shell = render_agent_shell_profile(&dir, Some("codex"));
    let ps1 = render_agent_ps1_profile(&dir, Some("codex"));
    assert!(shell.contains("memd authority warning:"));
    assert!(shell.contains("localhost fallback is lower trust"));
    assert!(ps1.contains("memd authority warning:"));
    assert!(ps1.contains("localhost fallback is lower trust"));

    fs::remove_dir_all(dir).expect("cleanup temp bundle");
}

#[test]
fn wake_packet_cross_harness_profiles_keep_same_bundle_defaults() {
    let root = std::env::temp_dir().join(format!("memd-wake-cross-harness-{}", uuid::Uuid::new_v4()));
    let bundle = root.join(".memd");
    let bin_dir = root.join("bin");
    fs::create_dir_all(&bundle).expect("create bundle");
    fs::create_dir_all(&bin_dir).expect("create bin dir");
    fs::write(
        bundle.join("config.json"),
        r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "session-alpha",
  "base_url": "http://127.0.0.1:8787",
  "route": "project_first",
  "intent": "current_task",
  "workspace": "shared",
  "visibility": "workspace"
}
"#,
    )
    .expect("write config");
    fs::write(
        bundle.join("env"),
        "MEMD_PROJECT=demo\nMEMD_NAMESPACE=main\nMEMD_WORKSPACE=shared\nMEMD_VISIBILITY=workspace\nMEMD_VOICE_MODE=normal\n",
    )
    .expect("write env");
    fs::write(bundle.join("backend.env"), "").expect("write backend env");
    fs::write(bundle.join("env.ps1"), "").expect("write env.ps1");
    fs::create_dir_all(bundle.join("agents")).expect("create agents dir");
    fs::write(
        bundle.join("agents").join("CODEX_WAKEUP.md"),
        "# codex wakeup\n",
    )
    .expect("write codex wakeup");
    fs::write(
        bundle.join("agents").join("watch.sh"),
        "#!/usr/bin/env bash\nexit 0\n",
    )
    .expect("write watch helper");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(bundle.join("agents").join("watch.sh"))
            .expect("stat watch helper")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(bundle.join("agents").join("watch.sh"), perms)
            .expect("chmod watch helper");
    }
    fs::write(
        bin_dir.join("memd"),
        "#!/usr/bin/env bash\nprintf '%s|agent=%s|project=%s|workspace=%s\\n' \"$*\" \"${MEMD_AGENT:-}\" \"${MEMD_PROJECT:-}\" \"${MEMD_WORKSPACE:-}\" >> \"$MEMD_LOG_PATH\"\n",
    )
    .expect("write fake memd");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(bin_dir.join("memd"))
            .expect("stat fake memd")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(bin_dir.join("memd"), perms).expect("chmod fake memd");
    }

    let path = std::env::var("PATH").unwrap_or_default();
    for (agent, log_name) in [
        ("codex", "codex.log"),
        ("claude-code", "claude.log"),
        ("openclaw", "openclaw.log"),
    ] {
        let profile = render_agent_shell_profile(&bundle, Some(agent));
        let log_path = root.join(log_name);
        let output = Command::new("bash")
            .arg("-c")
            .arg(profile)
            .env("PATH", format!("{}:{}", bin_dir.display(), path))
            .env("MEMD_LOG_PATH", &log_path)
            .output()
            .expect("run harness profile");
        assert!(
            output.status.success(),
            "{agent} profile failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let log_text = fs::read_to_string(&log_path).expect("read harness log");
        let wake_line = log_text
            .lines()
            .find(|line| line.contains("wake --output"))
            .expect("wake line");
        assert!(
            wake_line.contains(&bundle.display().to_string()),
            "wake_line={wake_line}"
        );
        assert!(
            wake_line.contains(&format!("agent={agent}")),
            "wake_line={wake_line}"
        );
    }

    fs::remove_dir_all(root).expect("cleanup temp project");
}

#[test]
fn hive_project_state_round_trips_through_bundle_runtime_config() {
    let runtime = BundleRuntimeConfig {
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        agent: Some("codex".to_string()),
        session: Some("codex-a".to_string()),
        tab_id: Some("tab-alpha".to_string()),
        hive_system: Some("codex".to_string()),
        hive_role: Some("agent".to_string()),
        capabilities: vec!["claim".to_string()],
        hive_groups: vec!["openclaw-stack".to_string()],
        hive_group_goal: Some("coordinate the project hive".to_string()),
        authority: Some("participant".to_string()),
        base_url: Some("http://100.104.154.24:8787".to_string()),
        route: Some("auto".to_string()),
        intent: Some("current_task".to_string()),
        voice_mode: None,
        workspace: Some("shared".to_string()),
        visibility: Some("workspace".to_string()),
        heartbeat_model: Some("gpt-4.1-mini".to_string()),
        auto_short_term_capture: true,
        hive_project_enabled: true,
        hive_project_anchor: Some("project:demo".to_string()),
        hive_project_joined_at: Some(Utc::now()),
        authority_policy: BundleAuthorityPolicy::default(),
        authority_state: BundleAuthorityState::default(),
    };
    let json = serde_json::to_string(&runtime).unwrap();
    assert!(json.contains("\"hive_project_enabled\":true"));
    assert!(json.contains("\"hive_project_anchor\":\"project:demo\""));
}

#[test]
fn merge_bundle_runtime_config_prefers_overlay_scope() {
    let runtime = BundleRuntimeConfig {
        project: Some("global".to_string()),
        namespace: Some("global".to_string()),
        agent: Some("codex".to_string()),
        session: Some("codex-a".to_string()),
        tab_id: None,
        hive_system: Some("codex".to_string()),
        hive_role: Some("agent".to_string()),
        capabilities: vec!["memory".to_string(), "coordination".to_string()],
        hive_groups: vec!["openclaw-stack".to_string()],
        hive_group_goal: None,
        authority: Some("participant".to_string()),
        hive_project_enabled: false,
        hive_project_anchor: None,
        hive_project_joined_at: None,
        base_url: Some("http://127.0.0.1:8787".to_string()),
        route: Some("auto".to_string()),
        intent: Some("general".to_string()),
        voice_mode: None,
        workspace: Some("global".to_string()),
        visibility: Some("private".to_string()),
        heartbeat_model: Some("llama-desktop/qwen".to_string()),
        auto_short_term_capture: true,
        authority_policy: BundleAuthorityPolicy::default(),
        authority_state: BundleAuthorityState::default(),
    };
    let overlay = BundleRuntimeConfig {
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        agent: None,
        session: None,
        tab_id: None,
        hive_system: Some("claw-control".to_string()),
        hive_role: Some("orchestrator".to_string()),
        capabilities: vec!["control".to_string(), "coordination".to_string()],
        hive_groups: vec!["openclaw-stack".to_string(), "control-plane".to_string()],
        hive_group_goal: None,
        authority: Some("coordinator".to_string()),
        hive_project_enabled: false,
        hive_project_anchor: None,
        hive_project_joined_at: None,
        base_url: None,
        route: Some("lexical".to_string()),
        intent: Some("current_task".to_string()),
        voice_mode: None,
        workspace: Some("team-alpha".to_string()),
        visibility: Some("workspace".to_string()),
        heartbeat_model: None,
        auto_short_term_capture: false,
        authority_policy: BundleAuthorityPolicy::default(),
        authority_state: BundleAuthorityState::default(),
    };

    let merged = merge_bundle_runtime_config(runtime, overlay);
    assert_eq!(merged.project.as_deref(), Some("memd"));
    assert_eq!(merged.namespace.as_deref(), Some("main"));
    assert_eq!(merged.session.as_deref(), Some("codex-a"));
    assert_eq!(merged.hive_system.as_deref(), Some("claw-control"));
    assert_eq!(merged.hive_role.as_deref(), Some("orchestrator"));
    assert_eq!(merged.route.as_deref(), Some("lexical"));
    assert_eq!(merged.intent.as_deref(), Some("current_task"));
    assert_eq!(merged.workspace.as_deref(), Some("team-alpha"));
    assert_eq!(merged.visibility.as_deref(), Some("workspace"));
    assert_eq!(merged.base_url.as_deref(), Some("http://127.0.0.1:8787"));
    assert!(merged.auto_short_term_capture);
}

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
  "base_url": "http://100.104.154.24:8787"
}
"#,
    )
    .expect("write fake global config");
    let original_home = std::env::var_os("HOME");
    unsafe {
        std::env::set_var("HOME", &home);
    }

    let resolved = resolve_hive_command_base_url(SHARED_MEMD_BASE_URL);
    assert_eq!(resolved, "http://100.104.154.24:8787");

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
  "base_url": "http://100.104.154.24:8787"
}
"#,
    )
    .expect("write fake global config");
    let original_home = std::env::var_os("HOME");
    unsafe {
        std::env::set_var("HOME", &home);
    }

    let resolved = default_base_url();
    assert_eq!(resolved, "http://100.104.154.24:8787");

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
fn resolve_bundle_command_base_url_honors_env_override() {
    let original_base_url = std::env::var_os("MEMD_BASE_URL");
    unsafe {
        std::env::set_var("MEMD_BASE_URL", "http://127.0.0.1:8787");
    }

    let resolved = resolve_bundle_command_base_url(
        "http://127.0.0.1:8787",
        Some("http://100.104.154.24:8787"),
    );
    assert_eq!(resolved, "http://127.0.0.1:8787");

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
  "base_url": "http://100.104.154.24:8787"
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
  "base_url": "http://100.104.154.24:8787",
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
  "base_url": "http://100.104.154.24:8787"
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
  "base_url": "http://100.104.154.24:8787",
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
    fs::remove_dir_all(temp_root).expect("cleanup status rebind temp");
}
