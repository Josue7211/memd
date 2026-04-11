    use super::*;

    fn builds_bundle_agent_profiles_for_known_agents() {
        let dir =
            std::env::temp_dir().join(format!("memd-agent-profiles-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");
        fs::write(
            dir.join("config.json"),
            r#"{
  "project": "demo",
  "agent": "codex",
  "session": "codex-a",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "general"
}
"#,
        )
        .expect("write config");
        let response =
            build_bundle_agent_profiles(&dir, None, Some("bash")).expect("agent profiles");
        assert_eq!(response.agents.len(), 6);
        assert_eq!(response.shell, "bash");
        assert_eq!(response.current.as_deref(), Some("codex"));
        assert_eq!(response.current_session.as_deref(), Some("codex-a"));
        assert_eq!(response.agents[0].name, "codex");
        assert_eq!(response.agents[0].effective_agent, "codex@codex-a");
        assert!(path_text_ends_with(
            &response.agents[0].memory_file,
            "agents/CODEX_MEMORY.md"
        ));
        assert!(response.agents[0].launch_hint.contains("codex.sh"));
        assert!(
            response.agents.iter().any(|agent| path_text_ends_with(
                &agent.memory_file,
                "agents/AGENT_ZERO_MEMORY.md"
            ))
        );
        assert!(
            response
                .agents
                .iter()
                .any(|agent| agent.name == "agent-zero")
        );
        assert!(response.agents.iter().any(|agent| agent.name == "hermes"));
        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn filters_bundle_agent_profiles_by_name() {
        let dir =
            std::env::temp_dir().join(format!("memd-agent-selected-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");
        fs::write(
            dir.join("config.json"),
            r#"{
  "project": "demo",
  "agent": "claude-code",
  "session": "claude-a",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "general"
}
"#,
        )
        .expect("write config");
        let response = build_bundle_agent_profiles(&dir, Some("claude-code"), Some("pwsh"))
            .expect("agent profiles");
        assert_eq!(response.agents.len(), 1);
        assert_eq!(response.current.as_deref(), Some("claude-code"));
        assert_eq!(response.selected.as_deref(), Some("claude-code"));
        assert_eq!(response.agents[0].name, "claude-code");
        assert!(response.agents[0].launch_hint.contains("claude-code.ps1"));
        assert!(
            response.agents[0]
                .native_hint
                .as_deref()
                .unwrap_or_default()
                .contains("CLAUDE_IMPORTS.md")
        );
        let summary = render_bundle_agent_profiles_summary(&response);
        assert!(summary.contains("current=claude-code"));
        assert!(summary.contains("session=claude-a"));
        assert!(summary.contains("claude-code [active]"));
        assert!(summary.contains("effective claude-code@claude-a"));
        assert!(summary.contains("/memory"));
        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn set_bundle_agent_updates_config_and_env_files() {
        let dir = std::env::temp_dir().join(format!("memd-agent-test-{}", uuid::Uuid::new_v4()));
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
        fs::write(
            dir.join("env"),
            "MEMD_AGENT=codex@codex-a\nMEMD_SESSION=codex-a\n",
        )
        .expect("write env");
        fs::write(
            dir.join("env.ps1"),
            "$env:MEMD_AGENT = \"codex@codex-a\"\n$env:MEMD_SESSION = \"codex-a\"\n",
        )
        .expect("write env.ps1");

        set_bundle_agent(&dir, "openclaw").expect("set bundle agent");

        let config = fs::read_to_string(dir.join("config.json")).expect("read config");
        let env = fs::read_to_string(dir.join("env")).expect("read env");
        let env_ps1 = fs::read_to_string(dir.join("env.ps1")).expect("read env.ps1");
        assert!(config.contains(r#""agent": "openclaw""#));
        assert!(env.contains("MEMD_AGENT=openclaw@codex-a"));
        assert!(env.contains("MEMD_WORKER_NAME='Openclaw'"));
        assert!(env_ps1.contains("$env:MEMD_AGENT = \"openclaw@codex-a\""));
        assert!(env_ps1.contains("$env:MEMD_WORKER_NAME = \"Openclaw\""));

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn repair_bundle_worker_name_env_backfills_missing_worker_name_assignments() {
        let dir =
            std::env::temp_dir().join(format!("memd-worker-env-repair-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");
        fs::write(
            dir.join("config.json"),
            r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "openclaw",
  "session": "session-live-openclaw",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "general"
}
"#,
        )
        .expect("write config");
        fs::write(
            dir.join("env"),
            "MEMD_AGENT=openclaw@session-live-openclaw\nMEMD_SESSION=session-live-openclaw\n",
        )
        .expect("write env");
        fs::write(
            dir.join("env.ps1"),
            "$env:MEMD_AGENT = \"openclaw@session-live-openclaw\"\n$env:MEMD_SESSION = \"session-live-openclaw\"\n",
        )
        .expect("write env.ps1");

        let repaired = repair_bundle_worker_name_env(&dir).expect("repair env");
        assert!(repaired);

        let env = fs::read_to_string(dir.join("env")).expect("read env");
        let env_ps1 = fs::read_to_string(dir.join("env.ps1")).expect("read env.ps1");
        assert!(env.contains("MEMD_WORKER_NAME='Openclaw'"));
        assert!(env_ps1.contains("$env:MEMD_WORKER_NAME = \"Openclaw\""));

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn write_init_bundle_quotes_worker_name_in_shell_env_for_launcher_source() {
        let root =
            std::env::temp_dir().join(format!("memd-worker-source-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).expect("create project root");
        let output = root.join(".memd");

        write_init_bundle(&InitArgs {
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            global: false,
            project_root: Some(root.clone()),
            seed_existing: false,
            agent: "codex".to_string(),
            session: Some("session-proof-alpha".to_string()),
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
            workspace: None,
            visibility: None,
            voice_mode: Some(default_voice_mode()),
            force: true,
            allow_localhost_read_only_fallback: false,
        })
        .expect("write init bundle");

        let env_path = output.join("env");
        let env_contents = fs::read_to_string(&env_path).expect("read env");
        assert!(env_contents.contains("MEMD_WORKER_NAME='Demo Codex proof-alpha'"));

        let shell_script = format!(
            ". {}\nprintf '%s' \"$MEMD_WORKER_NAME\"\n",
            shell_single_quote(env_path.to_string_lossy().as_ref())
        );
        let source = Command::new("bash")
            .arg("-lc")
            .arg(shell_script)
            .output()
            .expect("source env in bash");
        assert!(
            source.status.success(),
            "bash source failed: {}",
            String::from_utf8_lossy(&source.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&source.stdout),
            "Demo Codex proof-alpha"
        );

        fs::remove_dir_all(root).expect("cleanup temp project");
    }

    #[test]
    fn set_bundle_tab_id_updates_config_and_env_files() {
        let dir = std::env::temp_dir().join(format!("memd-tab-test-{}", uuid::Uuid::new_v4()));
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
        fs::write(
            dir.join("env"),
            "MEMD_AGENT=codex@codex-a\nMEMD_SESSION=codex-a\n",
        )
        .expect("write env");
        fs::write(
            dir.join("env.ps1"),
            "$env:MEMD_AGENT = \"codex@codex-a\"\n$env:MEMD_SESSION = \"codex-a\"\n",
        )
        .expect("write env.ps1");

        set_bundle_tab_id(&dir, "tab-a").expect("set bundle tab id");

        let config = fs::read_to_string(dir.join("config.json")).expect("read config");
        let env = fs::read_to_string(dir.join("env")).expect("read env");
        let env_ps1 = fs::read_to_string(dir.join("env.ps1")).expect("read env.ps1");
        assert!(config.contains(r#""tab_id": "tab-a""#));
        assert!(env.contains("MEMD_TAB_ID=tab-a"));
        assert!(env_ps1.contains("$env:MEMD_TAB_ID = \"tab-a\""));

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn set_bundle_auto_short_term_capture_updates_config_and_env_files() {
        let dir = std::env::temp_dir().join(format!("memd-bundle-policy-{}", uuid::Uuid::new_v4()));
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
  "intent": "general",
  "auto_short_term_capture": true
}
"#,
        )
        .expect("write config");
        fs::write(
            dir.join("env"),
            "MEMD_AGENT=codex@codex-a\nMEMD_SESSION=codex-a\nMEMD_AUTO_SHORT_TERM_CAPTURE=true\n",
        )
        .expect("write env");
        fs::write(
            dir.join("env.ps1"),
            "$env:MEMD_AGENT = \"codex@codex-a\"\n$env:MEMD_SESSION = \"codex-a\"\n$env:MEMD_AUTO_SHORT_TERM_CAPTURE = \"true\"\n",
        )
        .expect("write env.ps1");

        set_bundle_auto_short_term_capture(&dir, false).expect("set bundle policy");

        let config = fs::read_to_string(dir.join("config.json")).expect("read config");
        let env = fs::read_to_string(dir.join("env")).expect("read env");
        let env_ps1 = fs::read_to_string(dir.join("env.ps1")).expect("read env.ps1");
        assert!(config.contains(r#""auto_short_term_capture": false"#));
        assert!(env.contains("MEMD_AUTO_SHORT_TERM_CAPTURE=false"));
        assert!(env_ps1.contains("$env:MEMD_AUTO_SHORT_TERM_CAPTURE = \"false\""));

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }
