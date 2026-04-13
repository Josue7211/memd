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
            "mem.md"
        ));
        assert!(response.agents[0].launch_hint.contains("codex.sh"));
        assert!(
            response.agents[0]
                .native_hint
                .as_deref()
                .unwrap_or_default()
                .contains("wake.md")
        );
        assert!(
            response.agents.iter().any(|agent| path_text_ends_with(
                &agent.memory_file,
                "mem.md"
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
        assert!(path_text_ends_with(
            &response.agents[0].memory_file,
            "agents/CLAUDE_IMPORTS.md"
        ));
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
    fn attach_snippet_executes_wake_with_bundle_route_intent_and_env_defaults() {
        let root =
            std::env::temp_dir().join(format!("memd-attach-exec-{}", uuid::Uuid::new_v4()));
        let bundle = root.join(".memd");
        let bin_dir = root.join("bin");
        let log_path = root.join("memd.log");
        fs::create_dir_all(&bundle).expect("create bundle");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        fs::write(
            bundle.join("config.json"),
            r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "base_url": "http://127.0.0.1:8787",
  "route": "local_first",
  "intent": "general",
  "workspace": "shared",
  "visibility": "workspace"
}
"#,
        )
        .expect("write config");
        fs::write(
            bundle.join("env"),
            "MEMD_PROJECT=demo\nMEMD_NAMESPACE=main\nMEMD_WORKSPACE=shared\nMEMD_VISIBILITY=workspace\n",
        )
        .expect("write env");
        fs::write(bundle.join("env.ps1"), "").expect("write env.ps1");
        fs::write(
            bin_dir.join("memd"),
            format!(
                "#!/usr/bin/env bash\nprintf '%s|project=%s|workspace=%s|bundle=%s\\n' \"$*\" \"${{MEMD_PROJECT:-}}\" \"${{MEMD_WORKSPACE:-}}\" \"${{MEMD_BUNDLE_ROOT:-}}\" >> {}\n",
                shell_single_quote(log_path.to_string_lossy().as_ref())
            ),
        )
        .expect("write fake memd");
        let mut perms = fs::metadata(bin_dir.join("memd"))
            .expect("stat fake memd")
            .permissions();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            perms.set_mode(0o755);
            fs::set_permissions(bin_dir.join("memd"), perms).expect("chmod fake memd");
        }

        let attach = render_attach_snippet("bash", &bundle).expect("attach snippet");
        let shell_script = format!("{attach}\nsleep 0.2\n");
        let path = std::env::var("PATH").unwrap_or_default();
        let source = Command::new("bash")
            .arg("-lc")
            .arg(shell_script)
            .env("PATH", format!("{}:{}", bin_dir.display(), path))
            .output()
            .expect("run attach snippet");
        assert!(
            source.status.success(),
            "attach snippet failed: {}",
            String::from_utf8_lossy(&source.stderr)
        );

        let log = fs::read_to_string(&log_path).expect("read memd log");
        let wake_line = log
            .lines()
            .find(|line| line.contains("wake --output"))
            .expect("wake line");
        assert!(wake_line.contains("--route local_first --intent general --write"));
        assert!(wake_line.contains("project=demo"));
        assert!(wake_line.contains("workspace=shared"));
        assert!(wake_line.contains(&format!("bundle={}", bundle.display())));

        fs::remove_dir_all(root).expect("cleanup temp project");
    }

    #[test]
    fn codex_and_claude_profiles_execute_same_bundle_defaults() {
        let root =
            std::env::temp_dir().join(format!("memd-profile-exec-{}", uuid::Uuid::new_v4()));
        let bundle = root.join(".memd");
        let bin_dir = root.join("bin");
        let codex_log = root.join("codex.log");
        let claude_log = root.join("claude.log");
        fs::create_dir_all(&bundle).expect("create bundle");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        fs::write(
            bundle.join("config.json"),
            r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
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
        fs::write(bundle.join("wake.md"), "# codex wakeup\n").expect("write codex wakeup");
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

        let codex_profile = render_agent_shell_profile(&bundle, Some("codex"));
        let codex = Command::new("bash")
            .arg("-lc")
            .arg(codex_profile)
            .env("PATH", format!("{}:{}", bin_dir.display(), path))
            .env("MEMD_LOG_PATH", &codex_log)
            .output()
            .expect("run codex profile");
        assert!(
            codex.status.success(),
            "codex profile failed: {}",
            String::from_utf8_lossy(&codex.stderr)
        );

        let claude_profile = render_agent_shell_profile(&bundle, Some("claude-code"));
        let claude = Command::new("bash")
            .arg("-lc")
            .arg(claude_profile)
            .env("PATH", format!("{}:{}", bin_dir.display(), path))
            .env("MEMD_LOG_PATH", &claude_log)
            .output()
            .expect("run claude profile");
        assert!(
            claude.status.success(),
            "claude profile failed: {}",
            String::from_utf8_lossy(&claude.stderr)
        );

        let codex_log_text = fs::read_to_string(&codex_log).expect("read codex log");
        let claude_log_text = fs::read_to_string(&claude_log).expect("read claude log");
        let codex_wake = codex_log_text
            .lines()
            .find(|line| line.contains("wake --output"))
            .expect("codex wake line");
        let claude_wake = claude_log_text
            .lines()
            .find(|line| line.contains("wake --output"))
            .expect("claude wake line");

        assert!(codex_wake.contains("--route project_first --intent current_task --write"));
        assert!(claude_wake.contains("--route project_first --intent current_task --write"));
        assert!(codex_wake.contains("agent=codex"));
        assert!(claude_wake.contains("agent=claude-code"));
        assert!(codex_wake.contains("project=demo"));
        assert!(claude_wake.contains("project=demo"));
        assert!(codex_wake.contains("workspace=shared"));
        assert!(claude_wake.contains("workspace=shared"));

        fs::remove_dir_all(root).expect("cleanup temp project");
    }

    #[test]
    fn openclaw_profile_executes_same_bundle_defaults() {
        let root =
            std::env::temp_dir().join(format!("memd-openclaw-profile-{}", uuid::Uuid::new_v4()));
        let bundle = root.join(".memd");
        let bin_dir = root.join("bin");
        let log_path = root.join("openclaw.log");
        fs::create_dir_all(&bundle).expect("create bundle");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        fs::write(
            bundle.join("config.json"),
            r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "openclaw",
  "session": "openclaw-a",
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
            "MEMD_PROJECT=demo\nMEMD_NAMESPACE=main\nMEMD_WORKSPACE=shared\nMEMD_VISIBILITY=workspace\n",
        )
        .expect("write env");
        fs::write(bundle.join("backend.env"), "").expect("write backend env");
        fs::write(bundle.join("env.ps1"), "").expect("write env.ps1");
        fs::create_dir_all(bundle.join("agents")).expect("create agents dir");
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
        let profile = render_agent_shell_profile(&bundle, Some("openclaw"));
        let output = Command::new("bash")
            .arg("-lc")
            .arg(profile)
            .env("PATH", format!("{}:{}", bin_dir.display(), path))
            .env("MEMD_LOG_PATH", &log_path)
            .output()
            .expect("run openclaw profile");
        assert!(
            output.status.success(),
            "openclaw profile failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let log_text = fs::read_to_string(&log_path).expect("read openclaw log");
        let wake_line = log_text
            .lines()
            .find(|line| line.contains("wake --output"))
            .expect("openclaw wake line");
        assert!(wake_line.contains("--route project_first --intent current_task --write"));
        assert!(wake_line.contains("agent=openclaw"));
        assert!(wake_line.contains("project=demo"));
        assert!(wake_line.contains("workspace=shared"));

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
