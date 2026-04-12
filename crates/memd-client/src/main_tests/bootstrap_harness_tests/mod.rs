use super::*;

#[test]
fn derives_help_request_message_from_scope() {
    let message = derive_outbound_message(&MessagesArgs {
        output: PathBuf::from(".memd"),
        send: true,
        inbox: false,
        ack: None,
        target_session: Some("claude-b".to_string()),
        kind: None,
        request_help: true,
        request_review: false,
        assign_scope: None,
        scope: Some("file:src/main.rs".to_string()),
        content: None,
        summary: false,
    })
    .expect("derive help request");

    assert_eq!(message.0, "help_request");
    assert!(message.1.contains("file:src/main.rs"));
}

#[test]
fn derives_review_request_message_from_scope() {
    let message = derive_outbound_message(&MessagesArgs {
        output: PathBuf::from(".memd"),
        send: true,
        inbox: false,
        ack: None,
        target_session: Some("claude-b".to_string()),
        kind: None,
        request_help: false,
        request_review: true,
        assign_scope: None,
        scope: Some("task:parser-refactor".to_string()),
        content: None,
        summary: false,
    })
    .expect("derive review request");

    assert_eq!(message.0, "review_request");
    assert!(message.1.contains("task:parser-refactor"));
}

#[test]
fn derives_assignment_message_from_assign_scope() {
    let message = derive_outbound_message(&MessagesArgs {
        output: PathBuf::from(".memd"),
        send: true,
        inbox: false,
        ack: None,
        target_session: Some("claude-b".to_string()),
        kind: None,
        request_help: false,
        request_review: false,
        assign_scope: Some("task:parser-refactor".to_string()),
        scope: None,
        content: None,
        summary: false,
    })
    .expect("derive assignment");

    assert_eq!(message.0, "assignment");
    assert!(message.1.contains("task:parser-refactor"));
}

#[test]
fn resolves_nested_bundle_rag_config() {
    let config = BundleConfigFile {
        project: None,
        namespace: None,
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
        workspace: None,
        visibility: None,
        heartbeat_model: Some(default_heartbeat_model()),
        voice_mode: Some(default_voice_mode()),
        auto_short_term_capture: true,
        rag_url: None,
        authority_policy: BundleAuthorityPolicy::default(),
        authority_state: BundleAuthorityState::default(),
        backend: Some(BundleBackendConfigFile {
            rag: Some(BundleRagConfigFile {
                enabled: Some(true),
                url: Some("http://127.0.0.1:9000".to_string()),
            }),
        }),
    };

    let resolved = resolve_bundle_rag_config(config).expect("bundle rag config");
    assert!(resolved.enabled);
    assert!(resolved.configured);
    assert_eq!(resolved.url.as_deref(), Some("http://127.0.0.1:9000"));
    assert_eq!(resolved.source, "backend.rag");
}

#[test]
fn resolves_legacy_bundle_rag_url() {
    let config = BundleConfigFile {
        project: None,
        namespace: None,
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
        workspace: None,
        visibility: None,
        heartbeat_model: Some(default_heartbeat_model()),
        voice_mode: Some(default_voice_mode()),
        auto_short_term_capture: true,
        rag_url: Some("http://127.0.0.1:9000".to_string()),
        backend: None,
        authority_policy: BundleAuthorityPolicy::default(),
        authority_state: BundleAuthorityState::default(),
    };

    let resolved = resolve_bundle_rag_config(config).expect("bundle rag config");
    assert!(resolved.enabled);
    assert!(resolved.configured);
    assert_eq!(resolved.url.as_deref(), Some("http://127.0.0.1:9000"));
    assert_eq!(resolved.source, "rag_url");
}

#[test]
fn serializes_bundle_config_with_nested_rag_state() {
    let config = BundleConfig {
        schema_version: 2,
        project: "demo".to_string(),
        namespace: Some("main".to_string()),
        agent: "codex".to_string(),
        session: "session-demo".to_string(),
        tab_id: None,
        hive_system: Some("codex".to_string()),
        hive_role: Some("agent".to_string()),
        capabilities: vec!["memory".to_string(), "coordination".to_string()],
        hive_groups: vec!["openclaw-stack".to_string()],
        hive_group_goal: None,
        hive_project_enabled: false,
        hive_project_anchor: None,
        hive_project_joined_at: None,
        authority: Some("participant".to_string()),
        base_url: "http://127.0.0.1:8787".to_string(),
        route: "auto".to_string(),
        intent: "general".to_string(),
        workspace: Some("team-alpha".to_string()),
        visibility: Some("workspace".to_string()),
        heartbeat_model: default_heartbeat_model(),
        voice_mode: default_voice_mode(),
        auto_short_term_capture: true,
        authority_policy: BundleAuthorityPolicy::default(),
        authority_state: BundleAuthorityState::default(),
        backend: BundleBackendConfig {
            rag: BundleRagConfig {
                enabled: true,
                provider: "lightrag-compatible".to_string(),
                url: Some("http://127.0.0.1:9000".to_string()),
            },
        },
        hooks: BundleHooksConfig {
            context: "hooks/memd-context.sh".to_string(),
            capture: "hooks/memd-capture.sh".to_string(),
            spill: "hooks/memd-spill.sh".to_string(),
            context_ps1: "hooks/memd-context.ps1".to_string(),
            capture_ps1: "hooks/memd-capture.ps1".to_string(),
            spill_ps1: "hooks/memd-spill.ps1".to_string(),
        },
        rag_url: Some("http://127.0.0.1:9000".to_string()),
    };

    let json = serde_json::to_value(config).expect("serialize bundle config");
    assert_eq!(json["schema_version"], 2);
    assert_eq!(json["namespace"], "main");
    assert_eq!(json["backend"]["rag"]["enabled"], true);
    assert_eq!(json["backend"]["rag"]["provider"], "lightrag-compatible");
    assert_eq!(json["backend"]["rag"]["url"], "http://127.0.0.1:9000");
    assert_eq!(json["workspace"], "team-alpha");
    assert_eq!(json["visibility"], "workspace");
    assert_eq!(json["hooks"]["capture"], "hooks/memd-capture.sh");
    assert_eq!(json["hooks"]["capture_ps1"], "hooks/memd-capture.ps1");
    assert_eq!(json["rag_url"], "http://127.0.0.1:9000");
}

#[test]
fn writes_bundle_memory_placeholder_with_hot_path_guidance() {
    let dir =
        std::env::temp_dir().join(format!("memd-bundle-placeholder-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("create temp bundle");
    let config = BundleConfig {
        schema_version: 2,
        project: "demo".to_string(),
        namespace: Some("main".to_string()),
        agent: "codex".to_string(),
        session: "session-demo".to_string(),
        tab_id: None,
        hive_system: Some("codex".to_string()),
        hive_role: Some("agent".to_string()),
        capabilities: vec!["memory".to_string(), "coordination".to_string()],
        hive_groups: vec!["openclaw-stack".to_string()],
        hive_group_goal: None,
        hive_project_enabled: false,
        hive_project_anchor: None,
        hive_project_joined_at: None,
        authority: Some("participant".to_string()),
        base_url: "http://127.0.0.1:8787".to_string(),
        route: "auto".to_string(),
        intent: "general".to_string(),
        workspace: Some("team-alpha".to_string()),
        visibility: Some("workspace".to_string()),
        heartbeat_model: default_heartbeat_model(),
        voice_mode: default_voice_mode(),
        auto_short_term_capture: true,
        authority_policy: BundleAuthorityPolicy::default(),
        authority_state: BundleAuthorityState::default(),
        backend: BundleBackendConfig {
            rag: BundleRagConfig {
                enabled: true,
                provider: "lightrag-compatible".to_string(),
                url: Some("http://127.0.0.1:9000".to_string()),
            },
        },
        hooks: BundleHooksConfig {
            context: "hooks/memd-context.sh".to_string(),
            capture: "hooks/memd-capture.sh".to_string(),
            spill: "hooks/memd-spill.sh".to_string(),
            context_ps1: "hooks/memd-context.ps1".to_string(),
            capture_ps1: "hooks/memd-capture.ps1".to_string(),
            spill_ps1: "hooks/memd-spill.ps1".to_string(),
        },
        rag_url: Some("http://127.0.0.1:9000".to_string()),
    };

    write_bundle_memory_placeholder(&dir, &config, None, None).expect("write placeholder");
    write_native_agent_bridge_files(&dir).expect("write native bridge");

    let markdown = fs::read_to_string(dir.join("MEMD_MEMORY.md")).expect("read placeholder");
    assert!(markdown.contains("memd resume --output"));
    assert!(markdown.contains("--semantic"));
    assert!(markdown.contains("fast local hot path"));
    assert!(markdown.contains("slower deep recall"));
    assert!(markdown.contains("installed `$gsd-*` skills as the primary GSD interface"));
    assert!(markdown.contains("standalone `gsd-*` shell binaries"));
    assert!(markdown.contains("`$gsd-autonomous` is installed as a skill"));
    let claude_imports = fs::read_to_string(dir.join("agents").join("CLAUDE_IMPORTS.md"))
        .expect("read claude imports");
    let codex_agents = fs::read_to_string(dir.join("agents").join("AGENTS.md.example"))
        .expect("read codex agents example");
    assert!(claude_imports.contains("@../MEMD_WAKEUP.md"));
    assert!(claude_imports.contains("@../MEMD_MEMORY.md"));
    assert!(claude_imports.contains("@CLAUDE_CODE_WAKEUP.md"));
    assert!(claude_imports.contains("@CLAUDE_CODE_MEMORY.md"));
    assert!(claude_imports.contains("/memory"));
    assert!(claude_imports.contains("use installed `$gsd-*` skills as the GSD interface"));
    assert!(claude_imports.contains("standalone `gsd-*` shell binaries"));
    assert!(claude_imports.contains("`$gsd-autonomous` is installed as a skill"));
    assert!(codex_agents.contains(".memd/agents/CODEX_WAKEUP.md"));
    assert!(codex_agents.contains(".memd/agents/CODEX_MEMORY.md"));
    assert!(codex_agents.contains("Durable truth beats transcript recall."));
    assert!(codex_agents.contains("memd lookup --output .memd --query"));
    assert!(codex_agents.contains("stay in `caveman-ultra`"));

    fs::remove_dir_all(dir).expect("cleanup temp bundle");
}

#[test]
fn writes_bundle_memory_placeholder_with_normal_voice_mode() {
    let dir = std::env::temp_dir().join(format!("memd-bundle-voice-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("create temp bundle");
    let config = BundleConfig {
        schema_version: 2,
        project: "demo".to_string(),
        namespace: Some("main".to_string()),
        agent: "codex".to_string(),
        session: "session-demo".to_string(),
        tab_id: None,
        hive_system: Some("codex".to_string()),
        hive_role: Some("agent".to_string()),
        capabilities: vec!["memory".to_string(), "coordination".to_string()],
        hive_groups: vec!["openclaw-stack".to_string()],
        hive_group_goal: None,
        hive_project_enabled: false,
        hive_project_anchor: None,
        hive_project_joined_at: None,
        authority: Some("participant".to_string()),
        base_url: "http://127.0.0.1:8787".to_string(),
        route: "auto".to_string(),
        intent: "general".to_string(),
        workspace: Some("team-alpha".to_string()),
        visibility: Some("workspace".to_string()),
        heartbeat_model: default_heartbeat_model(),
        voice_mode: "normal".to_string(),
        auto_short_term_capture: true,
        authority_policy: BundleAuthorityPolicy::default(),
        authority_state: BundleAuthorityState::default(),
        backend: BundleBackendConfig {
            rag: BundleRagConfig {
                enabled: true,
                provider: "lightrag-compatible".to_string(),
                url: Some("http://127.0.0.1:9000".to_string()),
            },
        },
        hooks: BundleHooksConfig {
            context: "hooks/memd-context.sh".to_string(),
            capture: "hooks/memd-capture.sh".to_string(),
            spill: "hooks/memd-spill.sh".to_string(),
            context_ps1: "hooks/memd-context.ps1".to_string(),
            capture_ps1: "hooks/memd-capture.ps1".to_string(),
            spill_ps1: "hooks/memd-spill.ps1".to_string(),
        },
        rag_url: Some("http://127.0.0.1:9000".to_string()),
    };

    write_bundle_memory_placeholder(&dir, &config, None, None).expect("write placeholder");
    write_bundle_config_file(
        &dir.join("config.json"),
        &BundleConfigFile {
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            session: Some("session-demo".to_string()),
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
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            heartbeat_model: Some(default_heartbeat_model()),
            voice_mode: Some("normal".to_string()),
            auto_short_term_capture: true,
            rag_url: Some("http://127.0.0.1:9000".to_string()),
            backend: Some(BundleBackendConfigFile {
                rag: Some(BundleRagConfigFile {
                    enabled: Some(true),
                    url: Some("http://127.0.0.1:9000".to_string()),
                }),
            }),
            authority_policy: BundleAuthorityPolicy::default(),
            authority_state: BundleAuthorityState::default(),
        },
    )
    .expect("write config");
    write_native_agent_bridge_files(&dir).expect("write native bridge");

    let memory = fs::read_to_string(dir.join("MEMD_MEMORY.md")).expect("read placeholder");
    let claude_imports = fs::read_to_string(dir.join("agents").join("CLAUDE_IMPORTS.md"))
        .expect("read claude imports");
    let codex_agents = fs::read_to_string(dir.join("agents").join("AGENTS.md.example"))
        .expect("read codex agents");
    assert!(memory.contains("default: normal"));
    assert!(memory.contains("avoid forced compression"));
    assert!(claude_imports.contains("default: normal"));
    assert!(claude_imports.contains("avoid forced compression"));
    assert!(codex_agents.contains("stay in `normal`"));
    assert!(codex_agents.contains("sets `voice_mode` to `normal`"));

    fs::remove_dir_all(dir).expect("cleanup temp bundle");
}

#[test]
fn writes_bundle_memory_placeholder_with_caveman_lite_voice_mode() {
    let dir = std::env::temp_dir().join(format!("memd-bundle-voice-lite-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("create temp bundle");
    let config = BundleConfig {
        schema_version: 2,
        project: "demo".to_string(),
        namespace: Some("main".to_string()),
        agent: "codex".to_string(),
        session: "session-demo".to_string(),
        tab_id: None,
        hive_system: Some("codex".to_string()),
        hive_role: Some("agent".to_string()),
        capabilities: vec!["memory".to_string(), "coordination".to_string()],
        hive_groups: vec!["openclaw-stack".to_string()],
        hive_group_goal: None,
        hive_project_enabled: false,
        hive_project_anchor: None,
        hive_project_joined_at: None,
        authority: Some("participant".to_string()),
        base_url: "http://127.0.0.1:8787".to_string(),
        route: "auto".to_string(),
        intent: "general".to_string(),
        workspace: Some("team-alpha".to_string()),
        visibility: Some("workspace".to_string()),
        heartbeat_model: default_heartbeat_model(),
        voice_mode: "caveman-lite".to_string(),
        auto_short_term_capture: true,
        authority_policy: BundleAuthorityPolicy::default(),
        authority_state: BundleAuthorityState::default(),
        backend: BundleBackendConfig {
            rag: BundleRagConfig {
                enabled: true,
                provider: "lightrag-compatible".to_string(),
                url: Some("http://127.0.0.1:9000".to_string()),
            },
        },
        hooks: BundleHooksConfig {
            context: "hooks/memd-context.sh".to_string(),
            capture: "hooks/memd-capture.sh".to_string(),
            spill: "hooks/memd-spill.sh".to_string(),
            context_ps1: "hooks/memd-context.ps1".to_string(),
            capture_ps1: "hooks/memd-capture.ps1".to_string(),
            spill_ps1: "hooks/memd-spill.ps1".to_string(),
        },
        rag_url: Some("http://127.0.0.1:9000".to_string()),
    };

    write_bundle_memory_placeholder(&dir, &config, None, None).expect("write placeholder");
    write_bundle_config_file(
        &dir.join("config.json"),
        &BundleConfigFile {
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            session: Some("session-demo".to_string()),
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
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            heartbeat_model: Some(default_heartbeat_model()),
            voice_mode: Some("caveman-lite".to_string()),
            auto_short_term_capture: true,
            rag_url: Some("http://127.0.0.1:9000".to_string()),
            backend: Some(BundleBackendConfigFile {
                rag: Some(BundleRagConfigFile {
                    enabled: Some(true),
                    url: Some("http://127.0.0.1:9000".to_string()),
                }),
            }),
            authority_policy: BundleAuthorityPolicy::default(),
            authority_state: BundleAuthorityState::default(),
        },
    )
    .expect("write config");
    write_native_agent_bridge_files(&dir).expect("write native bridge");

    let memory = fs::read_to_string(dir.join("MEMD_MEMORY.md")).expect("read placeholder");
    let claude_imports = fs::read_to_string(dir.join("agents").join("CLAUDE_IMPORTS.md"))
        .expect("read claude imports");
    let codex_agents = fs::read_to_string(dir.join("agents").join("AGENTS.md.example"))
        .expect("read codex agents");
    assert!(memory.contains("default: `caveman-lite`"));
    assert!(memory.contains("compress, but not to ultra level"));
    assert!(claude_imports.contains("default: `caveman-lite`"));
    assert!(
        claude_imports.contains("match `.memd/config.json` exactly if the user changes voice_mode")
    );
    assert!(
        codex_agents
            .contains("Valid repo voice modes are `normal`, `caveman-lite`, and `caveman-ultra`.")
    );
    assert!(codex_agents.contains("stay in `caveman-lite`"));
    assert!(codex_agents.contains("sets `voice_mode` to `caveman-lite`"));

    fs::remove_dir_all(dir).expect("cleanup temp bundle");
}

#[test]
fn wake_fallback_writes_placeholder_memory_and_wakeup_files() {
    let dir = std::env::temp_dir().join(format!("memd-wake-fallback-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("create temp bundle");

    let config = BundleConfigFile {
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        agent: Some("codex".to_string()),
        session: Some("session-demo".to_string()),
        tab_id: Some("tab-alpha".to_string()),
        workspace: Some("team-alpha".to_string()),
        visibility: Some("workspace".to_string()),
        route: Some("auto".to_string()),
        intent: Some("current_task".to_string()),
        ..Default::default()
    };
    write_bundle_config_file(&dir.join("config.json"), &config).expect("write config");

    let args = WakeArgs {
        output: dir.clone(),
        project: None,
        namespace: None,
        agent: None,
        workspace: None,
        visibility: None,
        route: None,
        intent: None,
        limit: None,
        rehydration_limit: None,
        semantic: false,
        verbose: false,
        write: true,
        summary: false,
    };

    write_bundle_turn_fallback_artifacts(
        &dir,
        args.project.as_deref(),
        args.namespace.as_deref(),
        args.agent.as_deref(),
        args.workspace.as_deref(),
        args.visibility.as_deref(),
        args.route.as_deref(),
        args.intent.as_deref(),
        "# memd wake-up\n\n- fallback\n",
    )
    .expect("write wake fallback");

    let memory = fs::read_to_string(dir.join("MEMD_MEMORY.md")).expect("read memory");
    let wakeup = fs::read_to_string(dir.join("MEMD_WAKEUP.md")).expect("read wakeup");
    assert!(memory.contains("## Bundle Defaults"));
    assert!(memory.contains("project: demo"));
    assert!(memory.contains("namespace: main"));
    assert!(memory.contains("agent: codex"));
    assert!(memory.contains("session: session-demo"));
    assert!(memory.contains("tab: tab-alpha"));
    assert!(memory.contains("## Voice"));
    assert!(memory.contains("caveman-ultra"));
    assert!(wakeup.contains("fallback"));
    assert!(dir.join("agents").join("CODEX_MEMORY.md").exists());
    assert!(dir.join("agents").join("CLAUDE_CODE_MEMORY.md").exists());

    fs::remove_dir_all(dir).expect("cleanup temp bundle");
}

#[test]
fn checkpoint_fallback_writes_placeholder_memory_without_agent() {
    let dir =
        std::env::temp_dir().join(format!("memd-checkpoint-fallback-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("create temp bundle");

    let config = BundleConfigFile {
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        session: Some("session-demo".to_string()),
        tab_id: Some("tab-alpha".to_string()),
        workspace: Some("team-alpha".to_string()),
        visibility: Some("workspace".to_string()),
        route: Some("auto".to_string()),
        intent: Some("current_task".to_string()),
        ..Default::default()
    };
    write_bundle_config_file(&dir.join("config.json"), &config).expect("write config");

    write_bundle_turn_placeholder_memory(
        &dir,
        Some("demo"),
        Some("main"),
        None,
        Some("team-alpha"),
        Some("workspace"),
        Some("auto"),
        Some("current_task"),
    )
    .expect("write placeholder memory");

    let memory = fs::read_to_string(dir.join("MEMD_MEMORY.md")).expect("read memory");
    assert!(memory.contains("project: demo"));
    assert!(memory.contains("namespace: main"));
    assert!(memory.contains("session: session-demo"));
    assert!(memory.contains("tab: tab-alpha"));
    assert!(memory.contains("agent: codex"));

    fs::remove_dir_all(dir).expect("cleanup temp bundle");
}

#[test]
fn fallback_memory_and_wakeup_surfaces_include_authority_warning() {
    let dir =
        std::env::temp_dir().join(format!("memd-authority-markdown-{}", uuid::Uuid::new_v4()));
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

    write_memory_markdown_files(&dir, "# memd memory\n\nbody\n").expect("write memory");
    write_wakeup_markdown_files(&dir, "# memd wake-up\n\nbody\n").expect("write wakeup");

    let memory = fs::read_to_string(dir.join("MEMD_MEMORY.md")).expect("read memory");
    let wakeup = fs::read_to_string(dir.join("MEMD_WAKEUP.md")).expect("read wakeup");
    assert!(memory.contains("## Session Start Warning"));
    assert!(memory.contains("shared authority unavailable"));
    assert!(wakeup.contains("## Session Start Warning"));
    assert!(wakeup.contains("localhost fallback is lower trust"));

    fs::remove_dir_all(dir).expect("cleanup temp bundle");
}

#[test]
fn copies_hook_assets_with_live_capture_scripts() {
    let dir = std::env::temp_dir().join(format!("memd-hook-assets-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("create hook temp dir");

    crate::bundle::agent_profiles::copy_hook_assets(&dir).expect("copy hook assets");

    assert!(dir.join("memd-context.sh").exists());
    assert!(dir.join("memd-context.ps1").exists());
    assert!(dir.join("memd-capture.sh").exists());
    assert!(dir.join("memd-capture.ps1").exists());
    assert!(dir.join("memd-spill.sh").exists());
    assert!(dir.join("memd-spill.ps1").exists());
    assert!(dir.join("memd-stop-save.sh").exists());
    assert!(dir.join("memd-stop-save.ps1").exists());
    assert!(dir.join("memd-precompact-save.sh").exists());
    assert!(dir.join("memd-precompact-save.ps1").exists());

    let install = fs::read_to_string(dir.join("install.sh")).expect("read install.sh");
    assert!(install.contains("memd-capture"));
    assert!(install.contains("memd-hook-capture"));
    assert!(install.contains("memd-hook-stop-save"));
    assert!(install.contains("memd-hook-precompact-save"));

    fs::remove_dir_all(dir).expect("cleanup hook temp dir");
}

#[test]
fn writes_command_catalog_markdown_into_bundle_root() {
    let dir = std::env::temp_dir().join(format!("memd-command-docs-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("create command docs bundle");

    write_bundle_command_catalog_files(&dir).expect("write command catalog");

    let commands = fs::read_to_string(dir.join("COMMANDS.md")).expect("read command catalog");
    assert!(commands.contains("# memd commands"));
    assert!(commands.contains("/memory"));
    assert!(commands.contains("$gsd-autonomous"));

    fs::remove_dir_all(dir).expect("cleanup command docs bundle");
}

#[test]
fn codex_pack_manifest_exposes_recall_capture_cache_and_files() {
    let bundle_root =
        std::env::temp_dir().join(format!("memd-codex-pack-test-{}", uuid::Uuid::new_v4()));
    let manifest = crate::harness::codex::build_codex_harness_pack(&bundle_root, "demo", "main");

    assert_eq!(manifest.agent, "codex");
    assert!(
        manifest
            .files
            .iter()
            .any(|path| path.ends_with("CODEX_WAKEUP.md"))
    );
    assert!(
        manifest
            .files
            .iter()
            .any(|path| path.ends_with("CODEX_MEMORY.md"))
    );
    assert!(
        manifest
            .commands
            .iter()
            .any(|cmd| { cmd.contains("memd wake --output .memd --write") })
    );
    assert!(
        manifest
            .commands
            .iter()
            .any(|cmd| cmd.contains("memd lookup --output .memd --query"))
    );
    assert!(
        manifest
            .commands
            .iter()
            .any(|cmd| cmd.contains("memd hook capture --output .memd"))
    );
    assert!(
        manifest
            .behaviors
            .iter()
            .any(|line| line.contains("pre-answer lookup before memory-dependent responses"))
    );
    assert!(
        manifest
            .behaviors
            .iter()
            .any(|line| line.contains("turn-scoped cache"))
    );

    let markdown = render_codex_harness_pack_markdown(&manifest);
    assert!(markdown.contains("CODEX_WAKEUP.md"));
    assert!(markdown.contains("CODEX_MEMORY.md"));
    assert!(markdown.contains("turn-scoped cache"));
    assert!(markdown.contains("memd lookup --output .memd --query"));
    assert!(markdown.contains("memd hook capture --output .memd --stdin --summary"));
}

#[test]
fn claude_code_pack_manifest_exposes_native_bridge_and_files() {
    let bundle_root = std::env::temp_dir().join(format!(
        "memd-claude-code-pack-test-{}",
        uuid::Uuid::new_v4()
    ));
    let manifest =
        crate::harness::claude_code::build_claude_code_harness_pack(&bundle_root, "demo", "main");

    assert_eq!(manifest.agent, "claude-code");
    assert!(
        manifest
            .files
            .iter()
            .any(|path| path.ends_with("CLAUDE_CODE_WAKEUP.md"))
    );
    assert!(
        manifest
            .files
            .iter()
            .any(|path| path.ends_with("CLAUDE_CODE_MEMORY.md"))
    );
    assert!(
        manifest
            .files
            .iter()
            .any(|path| path.ends_with("CLAUDE_IMPORTS.md"))
    );
    assert!(
        manifest
            .commands
            .iter()
            .any(|cmd| cmd.contains("memd lookup --output .memd"))
    );
    assert!(
        manifest
            .behaviors
            .iter()
            .any(|line| line.contains("native Claude import bridge"))
    );

    let markdown = render_claude_code_harness_pack_markdown(&manifest);
    assert!(markdown.contains("CLAUDE_CODE_WAKEUP.md"));
    assert!(markdown.contains("CLAUDE_IMPORTS.md"));
    assert!(markdown.contains("native Claude import bridge"));
    assert!(markdown.contains("memd lookup --output .memd --query"));
}

#[test]
fn command_catalog_includes_slash_and_skill_commands() {
    let bundle_root = std::env::temp_dir().join(format!(
        "memd-command-catalog-test-{}",
        uuid::Uuid::new_v4()
    ));
    let catalog = crate::build_command_catalog(&bundle_root);

    assert!(catalog.commands.iter().any(|entry| entry.name == "/memory"));
    assert!(
        catalog
            .commands
            .iter()
            .any(|entry| entry.name == "$gsd-autonomous")
    );
    assert!(
        catalog
            .commands
            .iter()
            .any(|entry| entry.name == ".memd/agents/claude-code.sh")
    );

    let summary = render_command_catalog_summary(&catalog, None);
    assert!(summary.contains("commands root="));
    assert!(summary.contains("commands="));
    assert!(summary.contains("/memory"));

    let markdown = render_command_catalog_markdown(&catalog);
    assert!(markdown.contains("# memd commands"));
    assert!(markdown.contains("## Native memd CLI"));
    assert!(markdown.contains("## Bridge surfaces"));
    assert!(markdown.contains("## Bundle helpers"));
    assert!(markdown.contains("/memory"));
    assert!(markdown.contains("$gsd-autonomous"));
    assert!(markdown.contains("bundle-root-present"));
    assert!(markdown.contains("codex-skill-installed"));
    assert!(markdown.contains(".memd/agents/claude-code.sh"));
}

#[test]
fn openclaw_pack_manifest_exposes_context_spill_cache_and_files() {
    let bundle_root =
        std::env::temp_dir().join(format!("memd-openclaw-pack-test-{}", uuid::Uuid::new_v4()));
    let manifest =
        crate::harness::openclaw::build_openclaw_harness_pack(&bundle_root, "demo", "main");

    assert_eq!(manifest.agent, "openclaw");
    assert!(
        manifest
            .files
            .iter()
            .any(|path| path.ends_with("OPENCLAW_WAKEUP.md"))
    );
    assert!(
        manifest
            .files
            .iter()
            .any(|path| path.ends_with("OPENCLAW_MEMORY.md"))
    );
    assert!(manifest.commands.iter().any(|cmd| {
        cmd.contains("memd context --project <project> --agent openclaw --compact")
    }));
    assert!(
        manifest
            .commands
            .iter()
            .any(|cmd| cmd.contains("memd hook spill --output .memd --stdin --apply"))
    );
    assert!(
        manifest
            .behaviors
            .iter()
            .any(|line| line.contains("turn-scoped cache"))
    );

    let markdown = render_openclaw_harness_pack_markdown(&manifest);
    assert!(markdown.contains("OPENCLAW_WAKEUP.md"));
    assert!(markdown.contains("OPENCLAW_MEMORY.md"));
    assert!(markdown.contains("turn-scoped cache"));
    assert!(markdown.contains("memd hook spill --output .memd --stdin --apply"));
}

#[test]
fn harness_registry_exposes_shared_preset_ids_and_defaults() {
    let registry = crate::harness::preset::HarnessPresetRegistry::default_registry();

    assert!(registry.packs.iter().any(|pack| pack.pack_id == "codex"));
    assert!(registry.packs.iter().any(|pack| pack.pack_id == "openclaw"));
    assert!(registry.packs.iter().any(|pack| pack.pack_id == "hermes"));
    assert!(registry.packs.iter().any(|pack| pack.pack_id == "opencode"));
    assert!(
        registry
            .packs
            .iter()
            .any(|pack| pack.pack_id == "agent-zero")
    );

    let codex = registry
        .get("codex")
        .expect("codex preset should exist in the shared registry");
    assert_eq!(
        codex.default_verbs,
        vec!["wake", "lookup", "checkpoint", "spill"]
    );
}

#[tokio::test]
async fn codex_pack_refreshes_wakeup_and_memory_files_after_capture() {
    let bundle_root =
        std::env::temp_dir().join(format!("memd-codex-refresh-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&bundle_root).expect("create bundle root");
    let snapshot = codex_test_snapshot("demo", "main", "codex");
    let manifest = crate::harness::codex::build_codex_harness_pack(&bundle_root, "demo", "main");

    let written = refresh_harness_pack_files(
        &bundle_root,
        &snapshot,
        &manifest,
        "codex",
        "refresh",
        &harness_pack_query_from_snapshot(&snapshot),
    )
    .await
    .expect("refresh codex pack files");

    assert!(written.iter().any(|path| path.ends_with("MEMD_WAKEUP.md")));
    assert!(written.iter().any(|path| path.ends_with("MEMD_MEMORY.md")));
    assert!(
        written
            .iter()
            .any(|path| path_text_ends_with(path, "agents/CODEX_WAKEUP.md"))
    );
    assert!(
        written
            .iter()
            .any(|path| path_text_ends_with(path, "agents/CODEX_MEMORY.md"))
    );
    assert!(bundle_root.join("MEMD_WAKEUP.md").exists());
    assert!(bundle_root.join("MEMD_MEMORY.md").exists());
    assert!(bundle_root.join("agents").join("CODEX_WAKEUP.md").exists());
    assert!(bundle_root.join("agents").join("CODEX_MEMORY.md").exists());

    fs::remove_dir_all(bundle_root).expect("cleanup codex refresh temp dir");
}

#[tokio::test]
async fn openclaw_pack_refreshes_wakeup_and_memory_files_after_capture() {
    let bundle_root =
        std::env::temp_dir().join(format!("memd-openclaw-refresh-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&bundle_root).expect("create bundle root");
    let snapshot = codex_test_snapshot("demo", "main", "openclaw");
    let manifest =
        crate::harness::openclaw::build_openclaw_harness_pack(&bundle_root, "demo", "main");

    let written = refresh_harness_pack_files(
        &bundle_root,
        &snapshot,
        &manifest,
        "openclaw",
        "refresh",
        &harness_pack_query_from_snapshot(&snapshot),
    )
    .await
    .expect("refresh openclaw pack files");

    assert!(written.iter().any(|path| path.ends_with("MEMD_WAKEUP.md")));
    assert!(written.iter().any(|path| path.ends_with("MEMD_MEMORY.md")));
    assert!(
        written
            .iter()
            .any(|path| path.ends_with("agents/OPENCLAW_WAKEUP.md"))
    );
    assert!(
        written
            .iter()
            .any(|path| path.ends_with("agents/OPENCLAW_MEMORY.md"))
    );
    assert!(
        bundle_root
            .join("state")
            .join("openclaw-turn-cache.json")
            .exists()
    );
    assert!(bundle_root.join("MEMD_WAKEUP.md").exists());
    assert!(bundle_root.join("MEMD_MEMORY.md").exists());
    assert!(
        bundle_root
            .join("agents")
            .join("OPENCLAW_WAKEUP.md")
            .exists()
    );
    assert!(
        bundle_root
            .join("agents")
            .join("OPENCLAW_MEMORY.md")
            .exists()
    );

    fs::remove_dir_all(bundle_root).expect("cleanup openclaw refresh temp dir");
}

#[tokio::test]
async fn hermes_pack_refreshes_wakeup_and_memory_files_after_capture() {
    let bundle_root =
        std::env::temp_dir().join(format!("memd-hermes-refresh-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&bundle_root).expect("create bundle root");
    let snapshot = codex_test_snapshot("demo", "main", "hermes");
    let manifest = crate::harness::hermes::build_hermes_harness_pack(&bundle_root, "demo", "main");

    let written = refresh_harness_pack_files(
        &bundle_root,
        &snapshot,
        &manifest,
        "hermes",
        "refresh",
        &harness_pack_query_from_snapshot(&snapshot),
    )
    .await
    .expect("refresh hermes pack files");

    assert!(written.iter().any(|path| path.ends_with("MEMD_WAKEUP.md")));
    assert!(written.iter().any(|path| path.ends_with("MEMD_MEMORY.md")));
    assert!(
        written
            .iter()
            .any(|path| path.ends_with("agents/HERMES_WAKEUP.md"))
    );
    assert!(
        written
            .iter()
            .any(|path| path.ends_with("agents/HERMES_MEMORY.md"))
    );
    assert!(bundle_root.join("MEMD_WAKEUP.md").exists());
    assert!(bundle_root.join("MEMD_MEMORY.md").exists());
    assert!(bundle_root.join("agents").join("HERMES_WAKEUP.md").exists());
    assert!(bundle_root.join("agents").join("HERMES_MEMORY.md").exists());

    fs::remove_dir_all(bundle_root).expect("cleanup hermes refresh temp dir");
}

#[tokio::test]
async fn provenance_source_path_survives_across_codex_and_openclaw_memory_surfaces() {
    let bundle_root =
        std::env::temp_dir().join(format!("memd-provenance-parity-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&bundle_root).expect("create bundle root");
    let mut snapshot = codex_test_snapshot("demo", "main", "codex");
    snapshot.inbox.items[0].item.source_agent = Some("codex@test".to_string());
    snapshot.inbox.items[0].item.source_system = Some("hook-capture".to_string());
    snapshot.inbox.items[0].item.source_path = Some("notes/provenance.md".to_string());
    snapshot.working.rehydration_queue[0].source_agent = Some("codex@test".to_string());
    snapshot.working.rehydration_queue[0].source_system = Some("hook-capture".to_string());
    snapshot.working.rehydration_queue[0].source_path = Some("notes/provenance.md".to_string());

    let manifest = crate::harness::codex::build_codex_harness_pack(&bundle_root, "demo", "main");
    refresh_harness_pack_files(
        &bundle_root,
        &snapshot,
        &manifest,
        "codex",
        "refresh",
        &harness_pack_query_from_snapshot(&snapshot),
    )
    .await
    .expect("refresh harness pack files");

    let codex_memory = fs::read_to_string(bundle_root.join("agents").join("CODEX_MEMORY.md"))
        .expect("read codex memory");
    let openclaw_memory = fs::read_to_string(bundle_root.join("agents").join("OPENCLAW_MEMORY.md"))
        .expect("read openclaw memory");

    let expected_source = "codex@test / hook-capture / notes/provenance.md";
    assert!(codex_memory.contains(expected_source));
    assert!(openclaw_memory.contains(expected_source));
    assert!(codex_memory.contains("notes/provenance.md"));
    assert!(openclaw_memory.contains("notes/provenance.md"));

    fs::remove_dir_all(bundle_root).expect("cleanup provenance parity temp dir");
}

#[test]
fn harness_pack_turn_key_is_stable_for_repeated_recall() {
    for agent in ["codex", "hermes", "openclaw", "opencode", "agent-zero"] {
        let first = harness_pack_turn_key(
            Some("demo"),
            Some("main"),
            Some(agent),
            "full",
            "What did we decide?",
        );
        let second = harness_pack_turn_key(
            Some("demo"),
            Some("main"),
            Some(agent),
            "full",
            "  What    did    we decide?  ",
        );

        assert_eq!(first, second, "turn key should be stable for {agent}");
    }
}

#[test]
fn codex_pack_backend_failure_falls_back_to_local_bundle_truth() {
    let bundle_root =
        std::env::temp_dir().join(format!("memd-codex-local-truth-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&bundle_root).expect("create bundle root");
    fs::write(bundle_root.join("MEMD_WAKEUP.md"), "# local wakeup\n").expect("seed wakeup");
    fs::write(bundle_root.join("MEMD_MEMORY.md"), "# local memory\n").expect("seed memory");

    let wakeup = read_codex_pack_local_markdown(&bundle_root, "MEMD_WAKEUP.md")
        .expect("read wakeup fallback")
        .expect("local wakeup fallback");
    let memory = read_codex_pack_local_markdown(&bundle_root, "MEMD_MEMORY.md")
        .expect("read memory fallback")
        .expect("local memory fallback");

    assert!(wakeup.contains("local wakeup"));
    assert!(memory.contains("local memory"));

    fs::remove_dir_all(bundle_root).expect("cleanup codex fallback temp dir");
}

#[test]
fn codex_pack_docs_cover_operational_flow_and_fallback() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../");
    let setup = fs::read_to_string(repo_root.join("docs/core/setup.md")).expect("read setup docs");
    let api = fs::read_to_string(repo_root.join("docs/core/api.md")).expect("read api docs");
    let positioning = fs::read_to_string(repo_root.join("docs/reference/oss-positioning.md"))
        .expect("read oss docs");
    let agent_zero = fs::read_to_string(repo_root.join("integrations/agent-zero/README.md"))
        .expect("read agent zero docs");
    let codex = fs::read_to_string(repo_root.join("integrations/codex/README.md"))
        .expect("read codex docs");
    let opencode = fs::read_to_string(repo_root.join("integrations/opencode/README.md"))
        .expect("read opencode docs");
    let hooks = fs::read_to_string(repo_root.join("integrations/hooks/README.md"))
        .expect("read hooks docs");

    assert!(setup.contains("Codex is the first harness pack"));
    assert!(setup.contains("reads compiled memory before the turn"));
    assert!(setup.contains("turn-scoped cache"));
    assert!(setup.contains(".memd/MEMD_WAKEUP.md"));
    assert!(setup.contains(".memd/agents/CODEX_MEMORY.md"));
    assert!(setup.contains("Hermes is the adoption-focused harness pack"));
    assert!(setup.contains("Agent Zero is the zero-friction harness pack"));
    assert!(setup.contains("OpenCode is the shared-lane harness pack"));
    assert!(setup.contains(".memd/agents/hermes.sh"));
    assert!(setup.contains(".memd/agents/agent-zero.sh"));
    assert!(setup.contains(".memd/agents/opencode.sh"));

    assert!(api.contains("bundle-local harness pack flow"));
    assert!(api.contains("memd checkpoint"));
    assert!(api.contains("turn-scoped cache"));
    assert!(api.contains(".memd/MEMD_MEMORY.md"));
    assert!(api.contains("Hermes is the adoption-focused harness pack"));
    assert!(api.contains("Agent Zero is the zero-friction harness pack"));
    assert!(api.contains("OpenCode is the shared-lane harness pack"));

    assert!(positioning.contains("Codex is the first harness pack"));
    assert!(positioning.contains("local-first fallback path"));
    assert!(positioning.contains("Hermes is the adoption-focused harness pack"));
    assert!(positioning.contains("Agent Zero is the zero-friction harness pack"));
    assert!(positioning.contains("OpenCode is the shared-lane harness pack"));

    assert!(codex.contains("turn-first recall/capture pack"));
    assert!(codex.contains("memd hook capture --output .memd --stdin --summary"));
    assert!(codex.contains("memd hook capture --output .memd --stdin --summary"));
    assert!(codex.contains("Keep using the local bundle markdown"));
    assert!(codex.contains(
        "turn cache is keyed from project, namespace, agent, mode, and normalized query"
    ));
    assert!(codex.contains("Hermes uses the same shared memory core"));

    assert!(agent_zero.contains("zero-friction lane"));
    assert!(agent_zero.contains("memd hook spill --output .memd --stdin --apply"));
    assert!(agent_zero.contains(".memd/agents/AGENT_ZERO_MEMORY.md"));
    assert!(agent_zero.contains("memd handoff --output .memd --prompt"));

    assert!(opencode.contains("shared continuity plane"));
    assert!(opencode.contains("memd hook spill --output .memd --stdin --apply"));
    assert!(opencode.contains(".memd/agents/OPENCODE_MEMORY.md"));
    assert!(opencode.contains("memd handoff --output .memd --prompt"));

    assert!(hooks.contains("pre-turn read step"));
    assert!(hooks.contains("memd hook capture --stdin --summary"));
    assert!(hooks.contains("existing local bundle truth"));
    assert!(hooks.contains("Hermes is the adoption-focused harness pack"));
    assert!(hooks.contains("Agent Zero is the zero-friction harness pack"));
    assert!(hooks.contains("OpenCode is the shared-lane harness pack"));
}

#[test]
fn init_bootstrap_summarizes_existing_project_files() {
    let project_root =
        std::env::temp_dir().join(format!("memd-init-bootstrap-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(project_root.join(".planning")).expect("create project root");
    fs::write(
        project_root.join("CLAUDE.md"),
        "# project instructions\n\nremember memd",
    )
    .expect("write claude");
    fs::write(
        project_root.join("DESIGN.md"),
        "# design\n\nuse clean typography",
    )
    .expect("write design");
    fs::write(
        project_root.join(".planning").join("STATE.md"),
        "# state\n\nactive",
    )
    .expect("write state");
    fs::write(project_root.join("README.md"), "# demo repo").expect("write readme");

    let args = InitArgs {
        project: None,
        namespace: None,
        global: false,
        project_root: Some(project_root.clone()),
        seed_existing: true,
        agent: "codex".to_string(),
        session: None,
        tab_id: None,
        hive_system: None,
        hive_role: None,
        capability: Vec::new(),
        hive_group: Vec::new(),
        hive_group_goal: None,
        authority: None,
        output: project_root.join(".memd"),
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

    let bootstrap = build_project_bootstrap_memory(Some(project_root.as_path()), "demo", &args)
        .expect("build bootstrap")
        .expect("bootstrap context");
    assert!(bootstrap.markdown.contains("CLAUDE.md"));
    assert!(bootstrap.markdown.contains("DESIGN.md"));
    assert!(bootstrap.markdown.contains(".planning/STATE.md"));
    assert!(bootstrap.markdown.contains("README.md"));
    assert!(bootstrap.markdown.contains("project instructions"));
    assert!(bootstrap.markdown.contains("clean typography"));
    assert_eq!(bootstrap.registry.project, "demo");
    assert!(
        bootstrap
            .registry
            .sources
            .iter()
            .any(|source| source.path.ends_with("CLAUDE.md"))
    );
    assert!(
        bootstrap
            .registry
            .sources
            .iter()
            .any(|source| source.kind == "design")
    );

    fs::remove_dir_all(project_root).expect("cleanup temp project");
}

#[test]
fn init_bootstrap_writes_source_registry() {
    let project_root =
        std::env::temp_dir().join(format!("memd-init-registry-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(project_root.join(".planning")).expect("create planning");
    fs::write(project_root.join("AGENTS.md"), "# agents\n\nmemd").expect("write agents");
    fs::write(project_root.join("README.md"), "# demo repo").expect("write readme");

    let output = project_root.join(".memd");
    let args = InitArgs {
        project: None,
        namespace: None,
        global: false,
        project_root: Some(project_root.clone()),
        seed_existing: true,
        agent: "codex".to_string(),
        session: None,
        tab_id: None,
        hive_system: None,
        hive_role: None,
        capability: Vec::new(),
        hive_group: Vec::new(),
        hive_group_goal: None,
        authority: None,
        output: output.clone(),
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

    let bootstrap = build_project_bootstrap_memory(Some(project_root.as_path()), "demo", &args)
        .expect("build bootstrap")
        .expect("bootstrap context");
    write_bundle_source_registry(&output, &bootstrap.registry).expect("write registry");

    let registry_path = bundle_source_registry_path(&output);
    let raw = fs::read_to_string(&registry_path).expect("read registry");
    let registry: BootstrapSourceRegistry = serde_json::from_str(&raw).expect("parse registry");

    assert_eq!(registry.project, "demo");
    assert!(
        registry
            .sources
            .iter()
            .any(|source| source.path == "AGENTS.md")
    );
    assert!(
        registry
            .sources
            .iter()
            .any(|source| source.path == "README.md")
    );
    assert!(registry.sources.len() >= 2);

    fs::remove_dir_all(project_root).expect("cleanup temp project");
}

#[test]
fn refresh_bootstrap_memory_detects_changed_source() {
    let project_root =
        std::env::temp_dir().join(format!("memd-refresh-registry-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(project_root.join(".planning")).expect("create planning");
    fs::write(project_root.join("README.md"), "# demo repo").expect("write readme");

    let output = project_root.join(".memd");
    let args = InitArgs {
        project: None,
        namespace: None,
        global: false,
        project_root: Some(project_root.clone()),
        seed_existing: true,
        agent: "codex".to_string(),
        session: None,
        tab_id: None,
        hive_system: None,
        hive_role: None,
        capability: Vec::new(),
        hive_group: Vec::new(),
        hive_group_goal: None,
        authority: None,
        output: output.clone(),
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

    let bootstrap = build_project_bootstrap_memory(Some(project_root.as_path()), "demo", &args)
        .expect("build bootstrap")
        .expect("bootstrap context");
    write_bundle_source_registry(&output, &bootstrap.registry).expect("write registry");

    fs::write(project_root.join("README.md"), "# demo repo\n\nchanged").expect("rewrite readme");

    let refreshed = tokio::runtime::Runtime::new()
        .expect("create runtime")
        .block_on(refresh_project_bootstrap_memory(&output))
        .expect("refresh bootstrap")
        .expect("changed sources");

    assert!(refreshed.0.contains("Project source refresh"));
    let registry = refreshed.1;
    assert!(
        registry
            .sources
            .iter()
            .any(|source| source.path == "README.md")
    );
    let refreshed_readme = registry
        .sources
        .iter()
        .find(|source| source.path == "README.md")
        .expect("refreshed readme source");
    let initial_readme = bootstrap
        .registry
        .sources
        .iter()
        .find(|source| source.path == "README.md")
        .expect("initial readme source");
    assert!(refreshed_readme.imported_at >= initial_readme.imported_at);

    fs::remove_dir_all(project_root).expect("cleanup temp project");
}

#[test]
fn capability_registry_detects_bridgeable_superpowers_plugin() {
    let _home_lock = lock_home_mutation();
    let home = std::env::temp_dir().join(format!("memd-cap-registry-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(home.join(".claude")).expect("create claude root");
    fs::create_dir_all(
        home.join(".codex")
            .join("plugins")
            .join("cache")
            .join("claude-plugins-official")
            .join("superpowers")
            .join("5.0.6")
            .join(".codex"),
    )
    .expect("create codex cache");
    fs::write(
        home.join(".claude").join("settings.json"),
        r#"{
  "enabledPlugins": {
    "superpowers@claude-plugins-official": true
  }
}
"#,
    )
    .expect("write settings");
    fs::write(
        home.join(".codex")
            .join("plugins")
            .join("cache")
            .join("claude-plugins-official")
            .join("superpowers")
            .join("5.0.6")
            .join(".codex")
            .join("INSTALL.md"),
        "# install\n",
    )
    .expect("write install doc");

    let original_home = std::env::var_os("HOME");
    unsafe {
        std::env::set_var("HOME", &home);
    }

    let registry = build_bundle_capability_registry(None);
    let record = registry
        .capabilities
        .iter()
        .find(|record| {
            record.harness == "claude" && record.kind == "plugin" && record.name == "superpowers"
        })
        .expect("superpowers plugin record");
    assert_eq!(record.status, "enabled");
    assert_eq!(record.portability_class, "bridgeable");
    assert!(path_text_contains(
        record.bridge_hint.as_deref().unwrap_or_default(),
        ".codex/INSTALL.md"
    ));

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
fn capability_registry_collects_harness_surface_artifacts() {
    let _home_lock = lock_home_mutation();
    let home = std::env::temp_dir().join(format!(
        "memd-cap-registry-artifacts-{}",
        uuid::Uuid::new_v4()
    ));
    let cache_root = home
        .join(".codex")
        .join("plugins")
        .join("cache")
        .join("claude-plugins-official")
        .join("superpowers")
        .join("5.0.6");
    fs::create_dir_all(home.join(".claude")).expect("create claude root");
    fs::create_dir_all(home.join(".claude").join("agents")).expect("create agents dir");
    fs::create_dir_all(home.join(".claude").join("teams")).expect("create teams dir");
    fs::create_dir_all(home.join(".claude").join("hooks")).expect("create hooks dir");
    fs::create_dir_all(home.join(".claude").join("command")).expect("create command dir");
    fs::create_dir_all(cache_root.join("command")).expect("create plugin command dir");
    fs::create_dir_all(cache_root.join("hooks")).expect("create plugin hook dir");
    fs::write(
        home.join(".claude").join("agents").join("ops.md"),
        "# ops agent\n",
    )
    .expect("write agent");
    fs::write(
        home.join(".claude").join("teams").join("platform.md"),
        "# team platform\n",
    )
    .expect("write team");
    fs::write(
        home.join(".claude")
            .join("hooks")
            .join("memd-session-context.js"),
        "module.exports = {};\n",
    )
    .expect("write hook");
    fs::write(
        home.join(".claude").join("command").join("memd.md"),
        "# command\n",
    )
    .expect("write command");
    fs::write(
        home.join(".claude").join("settings.json"),
        r#"{
  "enabledPlugins": {
    "superpowers@claude-plugins-official": true
  }
}
"#,
    )
    .expect("write settings");
    fs::create_dir_all(cache_root.join(".codex")).expect("create cache codex dir");
    fs::write(
        cache_root.join("command").join("plugin.md"),
        "# plugin command\n",
    )
    .expect("write plugin command");
    fs::write(cache_root.join("hooks").join("plugin.mjs"), "export {}\n")
        .expect("write plugin hook");

    let original_home = std::env::var_os("HOME");
    unsafe {
        std::env::set_var("HOME", &home);
    }

    let registry = build_bundle_capability_registry(None);
    let agent = registry
        .capabilities
        .iter()
        .find(|record| {
            record.harness == "claude" && record.kind == "agent" && record.name == "agent:ops.md"
        })
        .expect("claude agent record");
    assert_eq!(agent.portability_class, "universal");
    let team = registry
        .capabilities
        .iter()
        .find(|record| {
            record.harness == "claude" && record.kind == "team" && record.name == "team:platform.md"
        })
        .expect("claude team record");
    assert_eq!(team.portability_class, "universal");
    let hook = registry
        .capabilities
        .iter()
        .find(|record| {
            record.harness == "claude"
                && record.kind == "hook"
                && record.name == "hook:memd-session-context.js"
        })
        .expect("claude hook record");
    assert_eq!(hook.portability_class, "universal");
    let command = registry
        .capabilities
        .iter()
        .find(|record| {
            record.harness == "claude"
                && record.kind == "command"
                && record.name == "command:memd.md"
        })
        .expect("claude command record");
    assert_eq!(command.portability_class, "universal");
    let plugin_command = registry
        .capabilities
        .iter()
        .find(|record| {
            record.harness == "claude"
                && record.kind == "command"
                && record.name == "superpowers:plugin.md"
        })
        .expect("claude plugin command record");
    assert_eq!(plugin_command.status, "discovered");
    assert_eq!(plugin_command.portability_class, "harness-native");
    let plugin_hook = registry
        .capabilities
        .iter()
        .find(|record| {
            record.harness == "claude"
                && record.kind == "hook"
                && record.name == "superpowers:plugin.mjs"
        })
        .expect("claude plugin hook record");
    assert_eq!(plugin_hook.status, "discovered");
    assert_eq!(plugin_hook.portability_class, "harness-native");

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
fn capability_registry_collects_openclaw_and_opencode_artifacts() {
    let _home_lock = lock_home_mutation();
    let home = std::env::temp_dir().join(format!(
        "memd-cap-registry-openclaw-opencode-{}",
        uuid::Uuid::new_v4()
    ));
    let openclaw_workspace = home.join(".openclaw").join("workspace");
    let opencode_root = home.join(".config").join("opencode");

    fs::create_dir_all(openclaw_workspace.join("agents")).expect("create openclaw agents");
    fs::create_dir_all(openclaw_workspace.join("teams")).expect("create openclaw teams");
    fs::create_dir_all(openclaw_workspace.join("hooks")).expect("create openclaw hooks");
    fs::create_dir_all(openclaw_workspace.join("command")).expect("create openclaw command");
    fs::create_dir_all(opencode_root.join("agents")).expect("create opencode agents");
    fs::create_dir_all(opencode_root.join("teams")).expect("create opencode teams");
    fs::create_dir_all(opencode_root.join("hooks")).expect("create opencode hooks");
    fs::create_dir_all(opencode_root.join("command")).expect("create opencode command");
    fs::create_dir_all(opencode_root.join("plugins")).expect("create opencode plugins");

    fs::write(
        openclaw_workspace.join("agents").join("shift.md"),
        "# openclaw agent\n",
    )
    .expect("write openclaw agent");
    fs::write(
        openclaw_workspace.join("teams").join("core.md"),
        "# openclaw team\n",
    )
    .expect("write openclaw team");
    fs::write(
        openclaw_workspace
            .join("hooks")
            .join("memd-session-context.js"),
        "module.exports = {};\n",
    )
    .expect("write openclaw hook");
    fs::write(
        openclaw_workspace.join("command").join("memd.md"),
        "# openclaw command\n",
    )
    .expect("write openclaw command");

    fs::write(
        opencode_root.join("agents").join("ops.md"),
        "# opencode agent\n",
    )
    .expect("write opencode agent");
    fs::write(
        opencode_root.join("teams").join("dev.md"),
        "# opencode team\n",
    )
    .expect("write opencode team");
    fs::write(
        opencode_root.join("hooks").join("memd-session-context.js"),
        "module.exports = {};\n",
    )
    .expect("write opencode hook");
    fs::write(
        opencode_root.join("command").join("memd.md"),
        "# opencode command\n",
    )
    .expect("write opencode command");
    fs::write(
        opencode_root.join("plugins").join("memd-plugin.mjs"),
        "export {}\n",
    )
    .expect("write opencode plugin");

    let original_home = std::env::var_os("HOME");
    unsafe {
        std::env::set_var("HOME", &home);
    }

    let registry = build_bundle_capability_registry(None);

    let openclaw_agent = registry
        .capabilities
        .iter()
        .find(|record| {
            record.harness == "openclaw"
                && record.kind == "agent"
                && record.name == "agent:shift.md"
        })
        .expect("openclaw agent record");
    assert_eq!(openclaw_agent.portability_class, "harness-native");

    let openclaw_team = registry
        .capabilities
        .iter()
        .find(|record| {
            record.harness == "openclaw" && record.kind == "team" && record.name == "team:core.md"
        })
        .expect("openclaw team record");
    assert_eq!(openclaw_team.portability_class, "harness-native");

    let openclaw_hook = registry
        .capabilities
        .iter()
        .find(|record| {
            record.harness == "openclaw"
                && record.kind == "hook"
                && record.name == "hook:memd-session-context.js"
        })
        .expect("openclaw hook record");
    assert_eq!(openclaw_hook.portability_class, "harness-native");

    let openclaw_command = registry
        .capabilities
        .iter()
        .find(|record| {
            record.harness == "openclaw"
                && record.kind == "command"
                && record.name == "command:memd.md"
        })
        .expect("openclaw command record");
    assert_eq!(openclaw_command.portability_class, "harness-native");

    let opencode_agent = registry
        .capabilities
        .iter()
        .find(|record| {
            record.harness == "opencode" && record.kind == "agent" && record.name == "agent:ops.md"
        })
        .expect("opencode agent record");
    assert_eq!(opencode_agent.portability_class, "harness-native");

    let opencode_team = registry
        .capabilities
        .iter()
        .find(|record| {
            record.harness == "opencode" && record.kind == "team" && record.name == "team:dev.md"
        })
        .expect("opencode team record");
    assert_eq!(opencode_team.portability_class, "harness-native");

    let opencode_plugin = registry
        .capabilities
        .iter()
        .find(|record| record.harness == "opencode" && record.kind == "plugin")
        .expect("opencode plugin record");
    assert_eq!(opencode_plugin.portability_class, "universal");

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
fn capability_bridges_install_superpowers_into_agents_skills() {
    let _home_lock = lock_home_mutation();
    let home = std::env::temp_dir().join(format!("memd-cap-bridge-{}", uuid::Uuid::new_v4()));
    let cache_root = home
        .join(".codex")
        .join("plugins")
        .join("cache")
        .join("claude-plugins-official")
        .join("superpowers")
        .join("5.0.6");
    fs::create_dir_all(home.join(".claude")).expect("create claude root");
    fs::create_dir_all(cache_root.join("skills")).expect("create skills dir");
    fs::create_dir_all(home.join(".agents").join("skills")).expect("create agents dir");
    fs::write(
        home.join(".claude").join("settings.json"),
        r#"{
  "enabledPlugins": {
    "superpowers@claude-plugins-official": true
  }
}
"#,
    )
    .expect("write settings");
    fs::write(cache_root.join("skills").join("README.md"), "superpowers")
        .expect("write bridge marker");

    let original_home = std::env::var_os("HOME");
    unsafe {
        std::env::set_var("HOME", &home);
    }

    let bridges = apply_capability_bridges();
    let action = bridges
        .actions
        .iter()
        .find(|action| action.harness == "codex" && action.capability == "superpowers")
        .expect("superpowers bridge action");
    assert!(matches!(
        action.status.as_str(),
        "bridged" | "already-bridged"
    ));
    let target = home.join(".agents").join("skills").join("superpowers");
    assert!(target.exists());
    assert!(
        fs::symlink_metadata(&target)
            .expect("read target metadata")
            .file_type()
            .is_symlink()
    );

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
fn capability_bridge_inspection_reports_available_without_mutating_targets() {
    let _home_lock = lock_home_mutation();
    let home = std::env::temp_dir().join(format!("memd-cap-inspect-{}", uuid::Uuid::new_v4()));
    let source = home
        .join(".codex")
        .join("plugins")
        .join("cache")
        .join("claude-plugins-official")
        .join("superpowers")
        .join("5.0.6")
        .join("skills");
    let target = home.join(".agents").join("skills").join("superpowers");

    fs::create_dir_all(&source).expect("create source skills dir");
    fs::create_dir_all(target.parent().expect("target parent")).expect("create target parent");
    fs::write(source.join("README.md"), "superpowers").expect("write source marker");

    let action = inspect_directory_skill_bridge("codex", "superpowers", &source, &target);
    assert_eq!(action.status, "available");
    assert_eq!(action.target_path, target.display().to_string());
    assert!(!target.exists());

    fs::remove_dir_all(home).expect("cleanup fake home");
}

#[test]
fn capability_bridges_install_superpowers_into_opencode_plugin_roots() {
    let _home_lock = lock_home_mutation();
    let home =
        std::env::temp_dir().join(format!("memd-cap-bridge-opencode-{}", uuid::Uuid::new_v4()));
    let cache_root = home
        .join(".codex")
        .join("plugins")
        .join("cache")
        .join("claude-plugins-official")
        .join("superpowers")
        .join("5.0.6");
    let source_opencode_plugins = cache_root.join(".opencode").join("plugins");
    let target_modern = home
        .join(".config")
        .join("opencode")
        .join("plugins")
        .join("superpowers");
    let target_legacy = home.join(".opencode").join("plugins").join("superpowers");

    fs::create_dir_all(home.join(".claude")).expect("create claude root");
    fs::create_dir_all(source_opencode_plugins.join("superpowers"))
        .expect("create source opencode plugin directory");
    fs::write(
        home.join(".claude").join("settings.json"),
        r#"{
  "enabledPlugins": {
    "superpowers@claude-plugins-official": true
  }
}
"#,
    )
    .expect("write settings");
    fs::write(
        source_opencode_plugins
            .join("superpowers")
            .join("memd-plugin.mjs"),
        "export {}\n",
    )
    .expect("write bridge marker");

    let original_home = std::env::var_os("HOME");
    unsafe {
        std::env::set_var("HOME", &home);
    }

    let bridges = apply_capability_bridges();
    let actions: Vec<_> = bridges
        .actions
        .iter()
        .filter(|action| action.harness == "opencode" && action.capability == "superpowers")
        .collect();
    assert_eq!(actions.len(), 2);
    for action in &actions {
        assert!(matches!(
            action.status.as_str(),
            "bridged" | "already-bridged"
        ));
    }
    let summary = render_capability_bridge_summary(&bridges);
    assert!(summary.contains(&target_modern.display().to_string()));
    assert!(summary.contains(&target_legacy.display().to_string()));
    assert!(
        actions
            .iter()
            .any(|action| action.target_path == target_modern.display().to_string())
    );
    assert!(
        actions
            .iter()
            .any(|action| action.target_path == target_legacy.display().to_string())
    );

    for target in [&target_modern, &target_legacy] {
        assert!(target.exists());
        let metadata = fs::symlink_metadata(target).expect("read target metadata");
        assert!(metadata.file_type().is_symlink());
    }

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
fn render_capability_registry_summary_includes_claude_family_bridgeable_records() {
    let registry = CapabilityRegistry {
        generated_at: Utc::now(),
        project_root: None,
        capabilities: vec![
            CapabilityRecord {
                harness: "claude".to_string(),
                kind: "skill".to_string(),
                name: "universal-skill".to_string(),
                status: "enabled".to_string(),
                portability_class: "universal".to_string(),
                source_path: "src/universal.md".to_string(),
                bridge_hint: None,
                hash: None,
                notes: Vec::new(),
            },
            CapabilityRecord {
                harness: "clawcode".to_string(),
                kind: "plugin".to_string(),
                name: "bridge-plugin".to_string(),
                status: "enabled".to_string(),
                portability_class: "bridgeable".to_string(),
                source_path: "src/plugin.md".to_string(),
                bridge_hint: Some("link-to-plugin".to_string()),
                hash: None,
                notes: Vec::new(),
            },
            CapabilityRecord {
                harness: "clawcode".to_string(),
                kind: "plugin".to_string(),
                name: "cl-family-bridgeable".to_string(),
                status: "enabled".to_string(),
                portability_class: "claude-family-bridgeable".to_string(),
                source_path: "src/cl-fam.md".to_string(),
                bridge_hint: Some("link-to-fork".to_string()),
                hash: None,
                notes: Vec::new(),
            },
        ],
    };

    let summary = render_capability_registry_summary(&registry);
    assert!(summary.contains("bridgeable: 2"));
    assert!(summary.contains("### Bridgeable capabilities"));
    assert!(summary.contains("clawcode / plugin / bridge-plugin [bridgeable]"));
    assert!(summary.contains(
        "clawcode / plugin / cl-family-bridgeable [claude-family-bridgeable] -> link-to-fork"
    ));
}

#[test]
fn render_capability_bridge_summary_includes_opencode_targets() {
    let home =
        std::env::temp_dir().join(format!("memd-cap-bridge-summary-{}", uuid::Uuid::new_v4()));
    let registry = CapabilityBridgeRegistry {
        generated_at: Utc::now(),
        actions: vec![
            CapabilityBridgeAction {
                harness: "opencode".to_string(),
                capability: "superpowers".to_string(),
                status: "bridged".to_string(),
                source_path: home
                    .join(".codex")
                    .join("plugins")
                    .join("cache")
                    .join("claude-plugins-official")
                    .join("superpowers")
                    .join("5.0.6")
                    .join(".opencode")
                    .join("plugins")
                    .join("superpowers")
                    .display()
                    .to_string(),
                target_path: home
                    .join(".config")
                    .join("opencode")
                    .join("plugins")
                    .join("superpowers")
                    .display()
                    .to_string(),
                notes: vec!["created native skill bridge".to_string()],
            },
            CapabilityBridgeAction {
                harness: "opencode".to_string(),
                capability: "superpowers".to_string(),
                status: "already-bridged".to_string(),
                source_path: home
                    .join(".codex")
                    .join("plugins")
                    .join("cache")
                    .join("claude-plugins-official")
                    .join("superpowers")
                    .join("5.0.6")
                    .join(".opencode")
                    .join("plugins")
                    .join("superpowers")
                    .display()
                    .to_string(),
                target_path: home
                    .join(".opencode")
                    .join("plugins")
                    .join("superpowers")
                    .display()
                    .to_string(),
                notes: vec!["already-bridged".to_string()],
            },
        ],
    };

    let summary = render_capability_bridge_summary(&registry);
    assert!(summary.contains("## Capability Bridges"));
    assert!(summary.contains("bridged: 1"));
    assert!(summary.contains("already_bridged: 1"));
    assert!(summary.contains("- opencode / superpowers -> "));
    let modern_target = home
        .join(".config")
        .join("opencode")
        .join("plugins")
        .join("superpowers")
        .display()
        .to_string();
    let legacy_target = home
        .join(".opencode")
        .join("plugins")
        .join("superpowers")
        .display()
        .to_string();
    assert!(summary.contains(&modern_target));
    assert!(summary.contains(&legacy_target));
}

#[test]
fn detect_claude_family_harness_roots_finds_clawcode_shape() {
    let home = std::env::temp_dir().join(format!("memd-clawcode-home-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(home.join(".claude")).expect("create claude root");
    fs::create_dir_all(home.join(".clawcode")).expect("create clawcode root");
    fs::write(home.join(".claude").join("settings.json"), "{}").expect("write claude settings");
    fs::write(home.join(".clawcode").join("settings.json"), "{}").expect("write clawcode settings");

    let roots = detect_claude_family_harness_roots(&home);
    assert!(roots.iter().any(|root| root.harness == "claude"));
    assert!(roots.iter().any(|root| root.harness == "clawcode"));

    fs::remove_dir_all(home).expect("cleanup fake home");
}

#[test]
fn capability_registry_detects_claude_family_fork_plugin_state() {
    let _home_lock = lock_home_mutation();
    let home = std::env::temp_dir().join(format!("memd-clawcode-cap-{}", uuid::Uuid::new_v4()));
    let cache_root = home
        .join(".codex")
        .join("plugins")
        .join("cache")
        .join("claude-plugins-official")
        .join("superpowers")
        .join("5.0.6")
        .join(".codex");
    fs::create_dir_all(home.join(".clawcode")).expect("create clawcode root");
    fs::create_dir_all(&cache_root).expect("create codex cache");
    fs::write(
        home.join(".clawcode").join("settings.json"),
        r#"{
  "enabledPlugins": {
    "superpowers@claude-plugins-official": true
  }
}
"#,
    )
    .expect("write clawcode settings");
    fs::write(cache_root.join("INSTALL.md"), "# install\n").expect("write install");

    let original_home = std::env::var_os("HOME");
    unsafe {
        std::env::set_var("HOME", &home);
    }

    let registry = build_bundle_capability_registry(None);
    let record = registry
        .capabilities
        .iter()
        .find(|record| record.harness == "clawcode" && record.name == "superpowers")
        .expect("clawcode superpowers record");
    assert_eq!(record.kind, "plugin");
    assert_eq!(record.status, "enabled");
    assert_eq!(record.portability_class, "claude-family-bridgeable");

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
fn init_output_prefers_project_root_when_seeded_from_repo() {
    let project_root =
        std::env::temp_dir().join(format!("memd-init-output-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&project_root).expect("create temp project root");

    let args = InitArgs {
        project: None,
        namespace: None,
        global: false,
        project_root: Some(project_root.clone()),
        seed_existing: true,
        agent: "codex".to_string(),
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

    let resolved = resolve_init_output_path(&args, Some(project_root.as_path()));
    assert_eq!(resolved, project_root.join(".memd"));

    fs::remove_dir_all(project_root).expect("cleanup temp project");
}

#[test]
fn init_output_prefers_global_bundle_when_requested() {
    let project_root =
        std::env::temp_dir().join(format!("memd-init-global-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&project_root).expect("create temp project root");

    let args = InitArgs {
        project: None,
        namespace: None,
        global: true,
        project_root: Some(project_root.clone()),
        seed_existing: true,
        agent: "codex".to_string(),
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

    let resolved = resolve_init_output_path(&args, Some(project_root.as_path()));
    assert_eq!(resolved, default_global_bundle_root());

    fs::remove_dir_all(project_root).expect("cleanup temp project");
}

#[test]
fn checkpoint_translation_sets_short_term_defaults() {
    let args = CheckpointArgs {
        output: PathBuf::from(".memd"),
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        workspace: Some("team-alpha".to_string()),
        visibility: Some("workspace".to_string()),
        source_path: Some("notes/today.md".to_string()),
        confidence: None,
        ttl_seconds: None,
        tag: vec!["urgent".to_string()],
        content: Some("remember current blocker".to_string()),
        input: None,
        stdin: false,
    };

    let translated = checkpoint_as_remember_args(&args);
    assert_eq!(translated.kind.as_deref(), Some("status"));
    assert_eq!(translated.scope.as_deref(), Some("project"));
    assert_eq!(translated.source_system.as_deref(), Some("memd-short-term"));
    assert_eq!(translated.source_quality.as_deref(), Some("derived"));
    assert_eq!(translated.confidence, Some(0.8));
    assert_eq!(translated.ttl_seconds, Some(86_400));
    assert!(translated.tag.iter().any(|value| value == "checkpoint"));
    assert!(translated.tag.iter().any(|value| value == "current-task"));
    assert!(translated.tag.iter().any(|value| value == "urgent"));
}

#[test]
fn bundle_memory_markdown_surfaces_current_task_snapshot() {
    let snapshot = ResumeSnapshot {
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        agent: Some("codex".to_string()),
        workspace: Some("team-alpha".to_string()),
        visibility: Some("workspace".to_string()),
        route: "auto".to_string(),
        intent: "current_task".to_string(),
        context: memd_schema::CompactContextResponse {
            route: memd_schema::RetrievalRoute::Auto,
            intent: memd_schema::RetrievalIntent::CurrentTask,
            retrieval_order: vec![memd_schema::MemoryScope::Project],
            records: Vec::new(),
        },
        working: memd_schema::WorkingMemoryResponse {
            route: memd_schema::RetrievalRoute::Auto,
            intent: memd_schema::RetrievalIntent::CurrentTask,
            retrieval_order: vec![memd_schema::MemoryScope::Project],
            budget_chars: 1600,
            used_chars: 60,
            remaining_chars: 1540,
            truncated: false,
            policy: memd_schema::WorkingMemoryPolicyState {
                admission_limit: 8,
                max_chars_per_item: 220,
                budget_chars: 1600,
                rehydration_limit: 4,
            },
            records: vec![memd_schema::CompactMemoryRecord {
                id: uuid::Uuid::new_v4(),
                record: "Finish the resume snapshot renderer".to_string(),
            }],
            evicted: Vec::new(),
            rehydration_queue: vec![memd_schema::MemoryRehydrationRecord {
                id: None,
                kind: "source".to_string(),
                label: "artifact".to_string(),
                summary: "Check the latest handoff note".to_string(),
                reason: None,
                source_agent: None,
                source_system: None,
                source_path: None,
                source_quality: None,
                recorded_at: None,
            }],
            traces: Vec::new(),
            semantic_consolidation: None,
        },
        inbox: memd_schema::MemoryInboxResponse {
            route: memd_schema::RetrievalRoute::Auto,
            intent: memd_schema::RetrievalIntent::CurrentTask,
            items: vec![memd_schema::InboxMemoryItem {
                item: memd_schema::MemoryItem {
                    id: uuid::Uuid::new_v4(),
                    content: "Repair one stale workspace lane".to_string(),
                    redundancy_key: None,
                    belief_branch: None,
                    preferred: true,
                    kind: memd_schema::MemoryKind::Status,
                    scope: memd_schema::MemoryScope::Project,
                    project: Some("demo".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("team-alpha".to_string()),
                    visibility: memd_schema::MemoryVisibility::Workspace,
                    source_agent: None,
                    source_system: None,
                    source_path: None,
                    source_quality: None,
                    confidence: 0.8,
                    ttl_seconds: Some(86_400),
                    created_at: chrono::Utc::now(),
                    status: memd_schema::MemoryStatus::Active,
                    stage: memd_schema::MemoryStage::Candidate,
                    last_verified_at: None,
                    supersedes: Vec::new(),
                    updated_at: chrono::Utc::now(),
                    tags: vec!["checkpoint".to_string()],
                },
                reasons: vec!["stale".to_string()],
            }],
        },
        workspaces: memd_schema::WorkspaceMemoryResponse {
            workspaces: vec![memd_schema::WorkspaceMemoryRecord {
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("team-alpha".to_string()),
                visibility: memd_schema::MemoryVisibility::Workspace,
                item_count: 4,
                active_count: 3,
                candidate_count: 1,
                contested_count: 0,
                source_lane_count: 1,
                avg_confidence: 0.84,
                trust_score: 0.91,
                last_seen_at: None,
                tags: Vec::new(),
            }],
        },
        sources: memd_schema::SourceMemoryResponse {
            sources: Vec::new(),
        },
        semantic: None,
        claims: SessionClaimsState::default(),
        recent_repo_changes: vec!["status M crates/memd-client/src/main.rs".to_string()],
        change_summary: vec!["focus -> Finish the resume snapshot renderer".to_string()],
        resume_state_age_minutes: None,
        refresh_recommended: false,
    };

    let markdown = render_bundle_memory_markdown(Path::new(".memd"), &snapshot, None, None);
    assert!(markdown.contains("## Budget"));
    assert!(markdown.contains("drivers="));
    assert!(markdown.contains("action=\""));
    assert!(markdown.contains("## Durable Truth"));
    assert!(markdown.contains("- none"));
    assert!(markdown.contains("## Read First"));
    assert!(markdown.contains("## Memory Objects"));
    assert!(markdown.contains("- context none"));
    assert!(markdown.contains("- working id="));
    assert!(markdown.contains("- inbox id="));
    assert!(markdown.contains("- recovery id="));
    assert!(markdown.contains("- workspace project="));
    assert!(markdown.contains("## E+LT"));
    assert!(markdown.contains("Finish the resume snapshot renderer"));
    assert!(markdown.contains("Repair one stale workspace lane"));
    assert!(markdown.contains("Check the latest handoff note"));
    assert!(markdown.contains("status M crates/memd-client/src/main.rs"));
    assert!(markdown.contains("team-alpha"));
    assert!(path_text_contains(&markdown, "compiled/memory/working.md"));
}
