    use super::*;

    fn harness_pack_index_lists_known_packs() {
        let root = std::env::temp_dir().join(format!("memd-pack-index-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).expect("create pack index root");

        let index =
            crate::harness::index::build_harness_pack_index(&root, Some("demo"), Some("main"));
        assert_eq!(index.pack_count, 6);
        assert!(index.packs.iter().any(|pack| pack.name == "Codex"));
        assert!(index.packs.iter().any(|pack| pack.name == "Claude Code"));
        assert!(index.packs.iter().any(|pack| pack.name == "Agent Zero"));
        assert!(index.packs.iter().any(|pack| pack.name == "Hermes"));
        assert!(index.packs.iter().any(|pack| pack.name == "OpenCode"));
        assert!(index.packs.iter().any(|pack| pack.name == "OpenClaw"));

        let summary = render_harness_pack_index_summary(&root, &index, None);
        assert!(summary.contains("pack index root="));
        assert!(summary.contains("packs=6"));
        assert!(summary.contains("Codex"));
        assert!(summary.contains("Claude Code"));
        assert!(summary.contains("Agent Zero"));
        assert!(summary.contains("Hermes"));
        assert!(summary.contains("OpenCode"));
        assert!(summary.contains("OpenClaw"));

        fs::remove_dir_all(root).expect("cleanup pack index temp dir");
    }

    #[test]
    fn hermes_pack_manifest_exposes_onboarding_wake_capture_and_files() {
        let bundle_root =
            std::env::temp_dir().join(format!("memd-hermes-pack-test-{}", uuid::Uuid::new_v4()));
        let manifest =
            crate::harness::hermes::build_hermes_harness_pack(&bundle_root, "demo", "main");

        assert_eq!(manifest.agent, "hermes");
        assert!(
            manifest
                .files
                .iter()
                .any(|path| path.ends_with("HERMES_WAKEUP.md"))
        );
        assert!(
            manifest
                .files
                .iter()
                .any(|path| path.ends_with("HERMES_MEMORY.md"))
        );
        assert!(
            manifest.commands.iter().any(|cmd| {
                cmd.contains("memd wake --output .memd --intent current_task --write")
            })
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
                .any(|line| line.contains("onboarding-friendly wake"))
        );

        let markdown = render_hermes_harness_pack_markdown(&manifest);
        assert!(markdown.contains("HERMES_WAKEUP.md"));
        assert!(markdown.contains("HERMES_MEMORY.md"));
        assert!(markdown.contains("onboarding-friendly wake"));
        assert!(markdown.contains("memd hook capture --output .memd --stdin --summary"));
    }

    #[test]
    fn agent_zero_pack_manifest_exposes_resume_handoff_and_files() {
        let bundle_root = std::env::temp_dir().join(format!(
            "memd-agent-zero-pack-test-{}",
            uuid::Uuid::new_v4()
        ));
        let manifest =
            crate::harness::agent_zero::build_agent_zero_harness_pack(&bundle_root, "demo", "main");

        assert_eq!(manifest.agent, "agent-zero");
        assert!(
            manifest
                .files
                .iter()
                .any(|path| path.ends_with("AGENT_ZERO_WAKEUP.md"))
        );
        assert!(
            manifest
                .files
                .iter()
                .any(|path| path.ends_with("AGENT_ZERO_MEMORY.md"))
        );
        assert!(
            manifest
                .commands
                .iter()
                .any(|cmd| cmd.contains("memd resume --output .memd"))
        );
        assert!(
            manifest
                .commands
                .iter()
                .any(|cmd| cmd.contains("memd handoff --output .memd --prompt"))
        );
        assert!(
            manifest
                .behaviors
                .iter()
                .any(|line| line.contains("zero-friction resume"))
        );

        let markdown = render_agent_zero_harness_pack_markdown(&manifest);
        assert!(markdown.contains("AGENT_ZERO_WAKEUP.md"));
        assert!(markdown.contains("AGENT_ZERO_MEMORY.md"));
        assert!(markdown.contains("zero-friction resume"));
        assert!(markdown.contains("memd remember --output .memd --kind decision"));
    }

    #[tokio::test]
    async fn agent_zero_pack_refreshes_visible_bundle_files_from_turn_state() {
        let bundle_root =
            std::env::temp_dir().join(format!("memd-agent-zero-refresh-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(bundle_root.join("agents")).expect("create agent-zero bundle dir");
        fs::write(
            bundle_root.join("MEMD_MEMORY.md"),
            "# Memory\n\nstale bundle\n",
        )
        .expect("seed memory file");
        fs::write(
            bundle_root.join("agents").join("AGENT_ZERO_MEMORY.md"),
            "# Agent Zero Memory\n\nstale agent bundle\n",
        )
        .expect("seed agent-zero agent memory file");

        let snapshot = codex_test_snapshot("demo", "main", "agent-zero");
        let manifest =
            crate::harness::agent_zero::build_agent_zero_harness_pack(&bundle_root, "demo", "main");

        let written = refresh_harness_pack_files(
            &bundle_root,
            &snapshot,
            &manifest,
            "agent-zero",
            "refresh",
            &harness_pack_query_from_snapshot(&snapshot),
        )
        .await
        .expect("refresh agent zero pack files");

        assert!(written.contains(&bundle_root.join("MEMD_MEMORY.md")));
        assert!(written.contains(&bundle_root.join("agents").join("AGENT_ZERO_MEMORY.md")));
        let refreshed = fs::read_to_string(bundle_root.join("MEMD_MEMORY.md"))
            .expect("read refreshed memory file");
        assert!(refreshed.contains("# memd memory"));
        assert!(refreshed.contains("keep the live wake surface current"));
    }

    #[test]
    fn opencode_pack_manifest_exposes_resume_handoff_and_files() {
        let bundle_root =
            std::env::temp_dir().join(format!("memd-opencode-pack-test-{}", uuid::Uuid::new_v4()));
        let manifest =
            crate::harness::opencode::build_opencode_harness_pack(&bundle_root, "demo", "main");

        assert_eq!(manifest.agent, "opencode");
        assert!(
            manifest
                .files
                .iter()
                .any(|path| path.ends_with("OPENCODE_WAKEUP.md"))
        );
        assert!(
            manifest
                .files
                .iter()
                .any(|path| path.ends_with("OPENCODE_MEMORY.md"))
        );
        assert!(
            manifest
                .commands
                .iter()
                .any(|cmd| cmd.contains("memd resume --output .memd"))
        );
        assert!(
            manifest
                .commands
                .iter()
                .any(|cmd| cmd.contains("memd handoff --output .memd --prompt"))
        );
        assert!(
            manifest
                .behaviors
                .iter()
                .any(|line| line.contains("write durable outcomes back"))
        );

        let markdown = render_opencode_harness_pack_markdown(&manifest);
        assert!(markdown.contains("OPENCODE_WAKEUP.md"));
        assert!(markdown.contains("OPENCODE_MEMORY.md"));
        assert!(markdown.contains("emit a shared handoff"));
        assert!(markdown.contains("memd remember --output .memd --kind decision"));
    }

    #[tokio::test]
    async fn opencode_pack_refreshes_visible_bundle_files_from_turn_state() {
        let bundle_root =
            std::env::temp_dir().join(format!("memd-opencode-refresh-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(bundle_root.join("agents")).expect("create opencode bundle dir");
        fs::write(
            bundle_root.join("MEMD_MEMORY.md"),
            "# Memory\n\nstale bundle\n",
        )
        .expect("seed memory file");
        fs::write(
            bundle_root.join("agents").join("OPENCODE_MEMORY.md"),
            "# OpenCode Memory\n\nstale agent bundle\n",
        )
        .expect("seed opencode agent memory file");

        let snapshot = codex_test_snapshot("demo", "main", "opencode");
        let manifest =
            crate::harness::opencode::build_opencode_harness_pack(&bundle_root, "demo", "main");

        let written = refresh_harness_pack_files(
            &bundle_root,
            &snapshot,
            &manifest,
            "opencode",
            "refresh",
            &harness_pack_query_from_snapshot(&snapshot),
        )
        .await
        .expect("refresh opencode pack files");

        assert!(written.contains(&bundle_root.join("MEMD_MEMORY.md")));
        assert!(written.contains(&bundle_root.join("agents").join("OPENCODE_MEMORY.md")));
        let refreshed = fs::read_to_string(bundle_root.join("MEMD_MEMORY.md"))
            .expect("read refreshed memory file");
        assert!(refreshed.contains("# memd memory"));
        assert!(refreshed.contains("keep the live wake surface current"));
    }

    #[test]
    fn harness_pack_index_query_matches_roles_commands_and_behaviors() {
        let root =
            std::env::temp_dir().join(format!("memd-pack-index-query-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).expect("create pack index root");

        let index =
            crate::harness::index::build_harness_pack_index(&root, Some("demo"), Some("main"));

        let spill = crate::harness::index::filter_harness_pack_index(index.clone(), Some("spill"));
        assert!(spill.packs.iter().any(|pack| pack.name == "OpenClaw"));
        assert!(!spill.packs.iter().any(|pack| pack.name == "Codex"));

        let capture =
            crate::harness::index::filter_harness_pack_index(index.clone(), Some("capture"));
        assert_eq!(capture.packs.len(), 2);
        assert!(capture.packs.iter().any(|pack| pack.name == "Codex"));
        assert!(capture.packs.iter().any(|pack| pack.name == "Hermes"));

        let compact =
            crate::harness::index::filter_harness_pack_index(index.clone(), Some("turn-scoped"));
        assert_eq!(compact.packs.len(), 3);

        let compact =
            crate::harness::index::filter_harness_pack_index(index, Some("compact context"));
        assert_eq!(compact.packs.len(), 1);
        assert_eq!(compact.packs[0].name, "OpenClaw");

        fs::remove_dir_all(root).expect("cleanup pack index query temp dir");
    }

    #[test]
    fn harness_pack_index_json_contains_structured_entries() {
        let root =
            std::env::temp_dir().join(format!("memd-pack-index-json-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).expect("create pack index root");

        let index =
            crate::harness::index::build_harness_pack_index(&root, Some("demo"), Some("main"));
        let json = render_harness_pack_index_json(&index);
        assert_eq!(json.root, root.display().to_string());
        assert_eq!(json.pack_count, 6);
        assert_eq!(json.packs.len(), 6);
        assert!(json.packs.iter().any(|pack| pack.name == "Codex"));
        assert!(json.packs.iter().any(|pack| pack.name == "Claude Code"));
        assert!(json.packs.iter().any(|pack| pack.name == "Agent Zero"));
        assert!(json.packs.iter().any(|pack| pack.name == "Hermes"));
        assert!(json.packs.iter().any(|pack| pack.name == "OpenCode"));
        assert!(json.packs.iter().any(|pack| pack.name == "OpenClaw"));

        fs::remove_dir_all(root).expect("cleanup pack index json temp dir");
    }

    #[tokio::test]
    async fn read_bundle_handoff_resolves_target_bundle_and_preserves_workspace_state() {
        let root =
            std::env::temp_dir().join(format!("memd-handoff-runtime-{}", uuid::Uuid::new_v4()));
        let current_project = root.join("current");
        let target_project = root.join("target");
        let current_bundle = current_project.join(".memd");
        let target_bundle = target_project.join(".memd");
        fs::create_dir_all(current_bundle.join("state")).expect("create current bundle state");
        fs::create_dir_all(target_bundle.join("state")).expect("create target bundle state");

        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state, false).await;

        fs::write(
            current_bundle.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
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
        .expect("write current config");

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

        fs::write(
            target_bundle.join("state").join("heartbeat.json"),
            serde_json::to_string_pretty(&BundleHeartbeatState {
                session: Some("claude-b".to_string()),
                agent: Some("claude-code".to_string()),
                effective_agent: Some("claude-code@claude-b".to_string()),
                tab_id: None,
                hive_system: Some("claude-code".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("claude-code".to_string()),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string(), "coordination".to_string()],
                hive_groups: vec!["openclaw-stack".to_string()],
                lane_id: Some(target_project.display().to_string()),
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some(default_heartbeat_model()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                repo_root: Some(root.display().to_string()),
                worktree_root: Some(target_project.display().to_string()),
                branch: Some("feature/claude-b".to_string()),
                base_branch: Some("main".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some(base_url.clone()),
                base_url_healthy: Some(true),
                host: Some("workstation".to_string()),
                pid: Some(2222),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: Some("resume delegated workspace".to_string()),
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
            })
            .expect("serialize target heartbeat"),
        )
        .expect("write target heartbeat");

        let snapshot = read_bundle_handoff(
            &HandoffArgs {
                output: current_bundle.clone(),
                target_session: Some("claude-b".to_string()),
                project: None,
                namespace: None,
                agent: None,
                workspace: None,
                visibility: None,
                route: None,
                intent: Some("current_task".to_string()),
                limit: Some(8),
                rehydration_limit: Some(4),
                source_limit: Some(6),
                semantic: false,
                prompt: false,
                summary: false,
            },
            &base_url,
        )
        .await
        .expect("read handoff");

        assert_eq!(snapshot.target_session.as_deref(), Some("claude-b"));
        assert_path_tail(
            snapshot.target_bundle.as_deref().unwrap_or_default(),
            &target_bundle,
        );
        assert_eq!(
            snapshot.resume.agent.as_deref(),
            Some("claude-code@claude-b")
        );
        assert_eq!(snapshot.resume.workspace.as_deref(), Some("shared"));
        assert_eq!(snapshot.resume.visibility.as_deref(), Some("workspace"));

        fs::remove_dir_all(root).expect("cleanup handoff runtime dir");
    }

    #[tokio::test]
    async fn read_bundle_resume_uses_cache_before_backend() {
        let dir = std::env::temp_dir().join(format!("memd-resume-cache-{}", uuid::Uuid::new_v4()));
        let output = dir.join(".memd");
        fs::create_dir_all(output.join("state")).expect("create bundle state dir");
        fs::write(
            output.join("config.json"),
            r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "tab_id": "tab-alpha",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "http://127.0.0.1:59999",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task"
}
"#,
        )
        .expect("write config");

        let args = ResumeArgs {
            output: output.clone(),
            project: None,
            namespace: None,
            agent: None,
            workspace: None,
            visibility: None,
            route: None,
            intent: Some("current_task".to_string()),
            limit: Some(8),
            rehydration_limit: Some(4),
            semantic: true,
            prompt: false,
            summary: false,
        };
        let runtime = read_bundle_runtime_config(&output)
            .expect("read runtime")
            .expect("runtime config");
        let base_url =
            resolve_bundle_command_base_url("http://127.0.0.1:59999", runtime.base_url.as_deref());
        let cache_key = build_resume_snapshot_cache_key(&args, Some(&runtime), &base_url);
        let snapshot = ResumeSnapshot {
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex@codex-a".to_string()),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            context: memd_schema::CompactContextResponse {
                route: RetrievalRoute::Auto,
                intent: RetrievalIntent::CurrentTask,
                retrieval_order: Vec::new(),
                records: Vec::new(),
            },
            working: memd_schema::WorkingMemoryResponse {
                route: RetrievalRoute::Auto,
                intent: RetrievalIntent::CurrentTask,
                retrieval_order: Vec::new(),
                budget_chars: 0,
                used_chars: 0,
                remaining_chars: 0,
                truncated: false,
                policy: memd_schema::WorkingMemoryPolicyState {
                    admission_limit: 0,
                    max_chars_per_item: 0,
                    budget_chars: 0,
                    rehydration_limit: 0,
                },
                records: Vec::new(),
                evicted: Vec::new(),
                rehydration_queue: Vec::new(),
                traces: Vec::new(),
                semantic_consolidation: None,
            },
            inbox: memd_schema::MemoryInboxResponse {
                route: RetrievalRoute::Auto,
                intent: RetrievalIntent::CurrentTask,
                items: Vec::new(),
            },
            workspaces: memd_schema::WorkspaceMemoryResponse {
                workspaces: Vec::new(),
            },
            sources: memd_schema::SourceMemoryResponse {
                sources: Vec::new(),
            },
            semantic: None,
            claims: SessionClaimsState::default(),
            recent_repo_changes: vec!["changed file".to_string()],
            change_summary: vec!["summary".to_string()],
            resume_state_age_minutes: Some(1),
            refresh_recommended: false,
        };
        cache::write_resume_snapshot_cache(&output, &cache_key, &snapshot)
            .expect("write resume cache");

        let resumed = read_bundle_resume(&args, "http://127.0.0.1:59999")
            .await
            .expect("resume from cache");

        assert_eq!(resumed.project.as_deref(), Some("demo"));
        assert_eq!(resumed.agent.as_deref(), Some("codex@codex-a"));
        assert_eq!(resumed.workspace.as_deref(), Some("shared"));
        assert!(resumed.semantic.is_none());
        assert_eq!(resumed.change_summary, vec!["summary".to_string()]);

        fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[tokio::test]
    async fn read_bundle_handoff_uses_cache_before_backend() {
        let dir = std::env::temp_dir().join(format!("memd-handoff-cache-{}", uuid::Uuid::new_v4()));
        let output = dir.join(".memd");
        fs::create_dir_all(output.join("state")).expect("create bundle state dir");
        fs::write(
            output.join("config.json"),
            r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "tab_id": "tab-alpha",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "http://127.0.0.1:59998",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task"
}
"#,
        )
        .expect("write config");

        let args = HandoffArgs {
            output: output.clone(),
            target_session: None,
            project: None,
            namespace: None,
            agent: None,
            workspace: None,
            visibility: None,
            route: None,
            intent: Some("current_task".to_string()),
            limit: Some(8),
            rehydration_limit: Some(4),
            source_limit: Some(6),
            semantic: false,
            prompt: false,
            summary: false,
        };
        let runtime = read_bundle_runtime_config(&output)
            .expect("read runtime")
            .expect("runtime config");
        let resolved_base_url =
            resolve_bundle_command_base_url("http://127.0.0.1:59998", runtime.base_url.as_deref());
        let resume_args = ResumeArgs {
            output: output.clone(),
            project: None,
            namespace: None,
            agent: None,
            workspace: None,
            visibility: None,
            route: None,
            intent: Some("current_task".to_string()),
            limit: Some(8),
            rehydration_limit: Some(4),
            semantic: false,
            prompt: false,
            summary: false,
        };
        let resume_key =
            build_resume_snapshot_cache_key(&resume_args, Some(&runtime), &resolved_base_url);
        let handoff_key = cache::build_turn_key(
            Some(&output.display().to_string()),
            None,
            Some("none"),
            "handoff",
            &format!(
                "resume_key={resume_key}|source_limit=6|target_session=none|target_bundle={}",
                output.display()
            ),
        );
        let resume_snapshot = ResumeSnapshot {
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex@codex-a".to_string()),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            context: memd_schema::CompactContextResponse {
                route: RetrievalRoute::Auto,
                intent: RetrievalIntent::CurrentTask,
                retrieval_order: Vec::new(),
                records: Vec::new(),
            },
            working: memd_schema::WorkingMemoryResponse {
                route: RetrievalRoute::Auto,
                intent: RetrievalIntent::CurrentTask,
                retrieval_order: Vec::new(),
                budget_chars: 0,
                used_chars: 0,
                remaining_chars: 0,
                truncated: false,
                policy: memd_schema::WorkingMemoryPolicyState {
                    admission_limit: 0,
                    max_chars_per_item: 0,
                    budget_chars: 0,
                    rehydration_limit: 0,
                },
                records: Vec::new(),
                evicted: Vec::new(),
                rehydration_queue: Vec::new(),
                traces: Vec::new(),
                semantic_consolidation: None,
            },
            inbox: memd_schema::MemoryInboxResponse {
                route: RetrievalRoute::Auto,
                intent: RetrievalIntent::CurrentTask,
                items: Vec::new(),
            },
            workspaces: memd_schema::WorkspaceMemoryResponse {
                workspaces: Vec::new(),
            },
            sources: memd_schema::SourceMemoryResponse {
                sources: Vec::new(),
            },
            semantic: None,
            claims: SessionClaimsState::default(),
            recent_repo_changes: vec!["changed file".to_string()],
            change_summary: vec!["summary".to_string()],
            resume_state_age_minutes: Some(1),
            refresh_recommended: false,
        };
        let handoff = HandoffSnapshot {
            generated_at: Utc::now(),
            resume: resume_snapshot,
            sources: memd_schema::SourceMemoryResponse {
                sources: Vec::new(),
            },
            target_session: None,
            target_bundle: Some(output.display().to_string()),
        };
        cache::write_handoff_snapshot_cache(&output, &handoff_key, &handoff)
            .expect("write handoff cache");

        let resumed = read_bundle_handoff(&args, "http://127.0.0.1:59998")
            .await
            .expect("handoff from cache");
        let expected_target_bundle = output.display().to_string();

        assert_eq!(
            resumed.target_bundle.as_deref(),
            Some(expected_target_bundle.as_str())
        );
        assert_eq!(resumed.resume.project.as_deref(), Some("demo"));
        assert_eq!(resumed.resume.agent.as_deref(), Some("codex@codex-a"));
        assert!(resumed.resume.semantic.is_none());

        fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn invalidate_bundle_runtime_caches_removes_resume_and_handoff_snapshots() {
        let dir =
            std::env::temp_dir().join(format!("memd-runtime-cache-prune-{}", uuid::Uuid::new_v4()));
        let output = dir.join(".memd");
        fs::create_dir_all(output.join("state")).expect("create state dir");
        fs::write(output.join("state/resume-snapshot-cache.json"), "{}\n")
            .expect("write resume cache");
        fs::write(output.join("state/handoff-snapshot-cache.json"), "{}\n")
            .expect("write handoff cache");

        invalidate_bundle_runtime_caches(&output).expect("invalidate bundle caches");

        assert!(!output.join("state/resume-snapshot-cache.json").exists());
        assert!(!output.join("state/handoff-snapshot-cache.json").exists());

        fs::remove_dir_all(dir).expect("cleanup runtime cache dir");
    }

    #[test]
    fn set_bundle_base_url_updates_config_and_env_files() {
        let dir =
            std::env::temp_dir().join(format!("memd-bundle-base-url-{}", uuid::Uuid::new_v4()));
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
        fs::write(dir.join("env"), "MEMD_BASE_URL=http://127.0.0.1:8787\n").expect("write env");
        fs::write(
            dir.join("env.ps1"),
            "$env:MEMD_BASE_URL = \"http://127.0.0.1:8787\"\n",
        )
        .expect("write env.ps1");

        set_bundle_base_url(&dir, "http://127.0.0.1:9797").expect("set bundle base url");

        let config = fs::read_to_string(dir.join("config.json")).expect("read config");
        let env = fs::read_to_string(dir.join("env")).expect("read env");
        let env_ps1 = fs::read_to_string(dir.join("env.ps1")).expect("read env.ps1");
        assert!(config.contains(r#""base_url": "http://127.0.0.1:9797""#));
        assert!(env.contains("MEMD_BASE_URL=http://127.0.0.1:9797"));
        assert!(env_ps1.contains("$env:MEMD_BASE_URL = \"http://127.0.0.1:9797\""));

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

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
