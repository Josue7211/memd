    use super::*;

    async fn resolve_bootstrap_authority_requires_explicit_localhost_fallback_consent() {
        let state = MockRuntimeState::default();
        let localhost_fallback_base_url = spawn_mock_runtime_server(state, false).await;
        let original = std::env::var_os("MEMD_LOCALHOST_FALLBACK_BASE_URL");
        unsafe {
            std::env::set_var(
                "MEMD_LOCALHOST_FALLBACK_BASE_URL",
                &localhost_fallback_base_url,
            );
        }

        let result = resolve_bootstrap_authority(InitArgs {
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            global: false,
            project_root: None,
            seed_existing: false,
            agent: "codex".to_string(),
            session: Some("codex-a".to_string()),
            tab_id: None,
            hive_system: Some("codex".to_string()),
            hive_role: Some("agent".to_string()),
            capability: vec!["memory".to_string()],
            hive_group: vec!["project:demo".to_string()],
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            output: std::env::temp_dir()
                .join(format!("memd-bootstrap-authority-{}", uuid::Uuid::new_v4())),
            base_url: "http://memd.invalid:8787".to_string(),
            rag_url: None,
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            voice_mode: Some(default_voice_mode()),
            force: false,
            allow_localhost_read_only_fallback: false,
        })
        .await;

        if let Some(value) = original {
            unsafe {
                std::env::set_var("MEMD_LOCALHOST_FALLBACK_BASE_URL", value);
            }
        } else {
            unsafe {
                std::env::remove_var("MEMD_LOCALHOST_FALLBACK_BASE_URL");
            }
        }

        let err = result.expect_err("missing consent should block localhost fallback");
        assert!(
            err.to_string()
                .contains("--allow-localhost-read-only-fallback")
        );
        assert!(err.to_string().contains(&localhost_fallback_base_url));
    }

    #[test]
    fn read_previous_maintain_report_uses_latest_timestamped_report() {
        let dir =
            std::env::temp_dir().join(format!("memd-maintain-history-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");
        let maintain_dir = dir.join("maintenance");
        fs::create_dir_all(&maintain_dir).expect("create maintenance dir");
        fs::write(
            maintain_dir.join("20260409T120000Z.json"),
            serde_json::to_string_pretty(&MaintainReport {
                mode: "compact".to_string(),
                receipt_id: Some("receipt-older".to_string()),
                compacted_items: 1,
                refreshed_items: 0,
                repaired_items: 0,
                findings: vec!["older".to_string()],
                generated_at: Utc::now(),
            })
            .expect("serialize older"),
        )
        .expect("write older report");
        fs::write(
            maintain_dir.join("20260409T130000Z.json"),
            serde_json::to_string_pretty(&MaintainReport {
                mode: "auto".to_string(),
                receipt_id: Some("receipt-newer".to_string()),
                compacted_items: 4,
                refreshed_items: 1,
                repaired_items: 1,
                findings: vec!["newer".to_string()],
                generated_at: Utc::now(),
            })
            .expect("serialize newer"),
        )
        .expect("write newer report");
        fs::write(
            maintain_dir.join("latest.json"),
            serde_json::to_string_pretty(&MaintainReport {
                mode: "auto".to_string(),
                receipt_id: Some("receipt-latest-link".to_string()),
                compacted_items: 4,
                refreshed_items: 1,
                repaired_items: 1,
                findings: vec!["latest".to_string()],
                generated_at: Utc::now(),
            })
            .expect("serialize latest"),
        )
        .expect("write latest report");

        let report = read_previous_maintain_report(&dir)
            .expect("read previous maintain report")
            .expect("expected previous maintain report");

        assert_eq!(report.receipt_id.as_deref(), Some("receipt-newer"));
        assert_eq!(report.mode, "auto");
        assert_eq!(report.compacted_items, 4);

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[tokio::test]
    async fn read_bundle_status_emits_truth_summary() {
        let dir = std::env::temp_dir().join(format!("memd-status-truth-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(dir.join("hooks")).expect("create hooks dir");
        fs::create_dir_all(dir.join("agents")).expect("create agents dir");
        let state = MockRuntimeState::default();
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
  "tab_id": "tab-alpha",
  "base_url": "{}",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task"
}}
"#,
                base_url
            ),
        )
        .expect("write config");
        fs::write(dir.join("env"), "MEMD_BASE_URL=test\n").expect("write env");
        fs::write(dir.join("env.ps1"), "$env:MEMD_BASE_URL='test'\n").expect("write env ps1");

        let status = read_bundle_status(&dir, SHARED_MEMD_BASE_URL)
            .await
            .expect("status via runtime base url");
        let truth = status.get("truth_summary").expect("truth summary present");
        assert_eq!(
            truth.get("retrieval_tier").and_then(JsonValue::as_str),
            Some("working")
        );
        assert!(
            truth
                .get("records")
                .and_then(JsonValue::as_array)
                .is_some_and(|records| !records.is_empty())
        );
        assert!(
            truth
                .get("source_count")
                .and_then(JsonValue::as_u64)
                .is_some()
        );

        fs::remove_dir_all(dir).expect("cleanup status dir");
    }

    #[tokio::test]
    async fn read_bundle_status_surfaces_evolution_summary() {
        let dir =
            std::env::temp_dir().join(format!("memd-status-evolution-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(dir.join("hooks")).expect("create hooks dir");
        fs::create_dir_all(dir.join("agents")).expect("create agents dir");
        let state = MockRuntimeState::default();
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
  "tab_id": "tab-alpha",
  "base_url": "{}",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task"
}}
"#,
                base_url
            ),
        )
        .expect("write config");
        fs::write(dir.join("env"), "MEMD_BASE_URL=test\n").expect("write env");
        fs::write(dir.join("env.ps1"), "$env:MEMD_BASE_URL='test'\n").expect("write env ps1");

        let mut report = test_experiment_report(&dir, true, false, 96, 100, Utc::now());
        report.composite.scenario = Some("self_evolution".to_string());
        report.improvement.final_changes =
            vec!["retune pass/fail gate for self-evolution proposals".to_string()];
        write_experiment_artifacts(&dir, &report).expect("write experiment artifacts");

        let status = read_bundle_status(&dir, SHARED_MEMD_BASE_URL)
            .await
            .expect("status via runtime base url");
        let evolution = status.get("evolution").expect("evolution summary present");
        assert_eq!(
            evolution.get("proposal_state").and_then(JsonValue::as_str),
            Some("accepted_proposal")
        );
        assert_eq!(
            evolution.get("scope_class").and_then(JsonValue::as_str),
            Some("runtime_policy")
        );
        assert_eq!(
            evolution.get("scope_gate").and_then(JsonValue::as_str),
            Some("auto_merge")
        );

        fs::remove_dir_all(dir).expect("cleanup status dir");
    }

    #[tokio::test]
    async fn write_bundle_heartbeat_times_out_slow_hive_publish() {
        let dir =
            std::env::temp_dir().join(format!("memd-heartbeat-timeout-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create heartbeat dir");
        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state.clone(), true).await;
        fs::write(
            dir.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "hive_system": "codex",
  "hive_role": "agent",
  "hive_groups": ["openclaw-stack", "runtime-core"],
  "workspace": "shared",
  "visibility": "workspace",
  "tab_id": "tab-alpha",
  "base_url": "{}",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task"
}}
"#,
                base_url
            ),
        )
        .expect("write config");

        let started = std::time::Instant::now();
        write_bundle_heartbeat(&dir, None, false)
            .await
            .expect("write heartbeat");
        assert!(started.elapsed() < std::time::Duration::from_secs(3));

        let heartbeat = read_bundle_heartbeat(&dir)
            .expect("read heartbeat")
            .expect("heartbeat present");
        assert_eq!(heartbeat.session.as_deref(), Some("codex-a"));
        assert_eq!(heartbeat.base_url.as_deref(), Some(base_url.as_str()));
        assert!(
            heartbeat
                .hive_groups
                .iter()
                .any(|group| group == "project:demo")
        );

        let session_upserts = state.session_upserts.lock().expect("lock session upserts");
        assert_eq!(session_upserts.len(), 1);
        assert!(
            session_upserts[0]
                .hive_groups
                .iter()
                .any(|group| group == "project:demo")
        );
        assert_eq!(
            session_upserts[0].worker_name.as_deref(),
            Some("Demo Codex a")
        );
        assert_eq!(session_upserts[0].role.as_deref(), Some("agent"));

        fs::remove_dir_all(dir).expect("cleanup heartbeat dir");
    }

    #[tokio::test]
    async fn write_bundle_heartbeat_retires_superseded_stale_sessions() {
        let dir =
            std::env::temp_dir().join(format!("memd-heartbeat-retire-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create heartbeat dir");
        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state.clone(), false).await;
        state
            .session_records
            .lock()
            .expect("lock session records")
            .push(memd_schema::HiveSessionRecord {
                session: "codex-stale".to_string(),
                tab_id: Some("tab-alpha".to_string()),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@codex-stale".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("codex".to_string()),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["project:demo".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some(default_heartbeat_model()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some(base_url.clone()),
                base_url_healthy: Some(true),
                host: Some("workstation".to_string()),
                pid: Some(4242),
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
                last_seen: Utc::now() - chrono::TimeDelta::minutes(8),
            });
        fs::write(
            dir.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-fresh",
  "hive_system": "codex",
  "hive_role": "agent",
  "workspace": "shared",
  "visibility": "workspace",
  "tab_id": "tab-alpha",
  "base_url": "{}",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task"
}}
"#,
                base_url
            ),
        )
        .expect("write config");

        write_bundle_heartbeat(&dir, None, false)
            .await
            .expect("write heartbeat");

        let retires = state.session_retires.lock().expect("lock session retires");
        assert_eq!(retires.len(), 1);
        assert_eq!(retires[0].session, "codex-stale");
        assert_eq!(retires[0].agent.as_deref(), Some("codex"));
        drop(retires);

        let records = state.session_records.lock().expect("lock session records");
        assert!(records.iter().all(|record| record.session != "codex-stale"));

        fs::remove_dir_all(dir).expect("cleanup heartbeat dir");
    }

    #[tokio::test]
    async fn write_bundle_memory_files_surfaces_hive_state_in_compiled_memory_pages() {
        let dir =
            std::env::temp_dir().join(format!("memd-memory-hive-pages-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp dir");
        let state = MockRuntimeState::default();
        push_mock_runtime_hive_session(
            &state,
            "queen-1",
            "Avicenna",
            "queen",
            Some("queen-routing"),
            Some("Route hive work"),
            vec!["docs/hive.md".to_string()],
        );
        push_mock_runtime_hive_session(
            &state,
            "bee-1",
            "Lorentz",
            "worker",
            Some("parser-refactor"),
            Some("Parser lane refactor"),
            vec![
                "task:parser-refactor".to_string(),
                "crates/memd-client/src/main.rs".to_string(),
            ],
        );
        state
            .task_records
            .lock()
            .expect("lock task records")
            .push(HiveTaskRecord {
                task_id: "parser-refactor".to_string(),
                title: "Refine parser overlap flow".to_string(),
                description: None,
                status: "active".to_string(),
                coordination_mode: "exclusive_write".to_string(),
                session: Some("bee-1".to_string()),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@bee-1".to_string()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                claim_scopes: vec!["crates/memd-client/src/main.rs".to_string()],
                help_requested: false,
                review_requested: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        state
            .receipts
            .lock()
            .expect("lock receipts")
            .push(HiveCoordinationReceiptRecord {
                id: "receipt-queen-deny".to_string(),
                kind: "queen_deny".to_string(),
                actor_session: "queen-1".to_string(),
                actor_agent: Some("dashboard".to_string()),
                target_session: Some("bee-1".to_string()),
                task_id: Some("parser-refactor".to_string()),
                scope: Some("crates/memd-client/src/main.rs".to_string()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                summary: "Queen denied overlapping lane or scope work for session bee-1."
                    .to_string(),
                created_at: Utc::now(),
            });
        state
            .messages
            .lock()
            .expect("lock messages")
            .push(HiveMessageRecord {
                id: "msg-handoff".to_string(),
                kind: "handoff".to_string(),
                from_session: "queen-1".to_string(),
                from_agent: Some("dashboard".to_string()),
                to_session: "bee-1".to_string(),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                content: "handoff_scope: crates/memd-client/src/main.rs".to_string(),
                created_at: Utc::now(),
                acknowledged_at: None,
            });
        let base_url = spawn_mock_runtime_server(state, false).await;
        fs::write(
            dir.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "bee-1",
  "workspace": "shared",
  "visibility": "workspace",
  "tab_id": "tab-alpha",
  "base_url": "{}",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task"
}}
"#,
                base_url
            ),
        )
        .expect("write config");

        let snapshot = codex_test_snapshot("demo", "main", "codex");
        write_bundle_memory_files(&dir, &snapshot, None, false)
            .await
            .expect("write bundle memory files");

        let memory =
            fs::read_to_string(dir.join("MEMD_MEMORY.md")).expect("read generated bundle memory");
        assert!(memory.contains("## Hive"));
        assert!(memory.contains("queen=queen-1"));
        assert!(memory.contains("active_bees=Avicenna(queen-1)/queen-routing"));
        assert!(memory.contains("focus=Lorentz"));

        let workspace_page = fs::read_to_string(dir.join("compiled/memory/workspace.md"))
            .expect("read workspace page");
        assert!(workspace_page.contains("## Hive"));
        assert!(workspace_page.contains("bee Lorentz (bee-1)"));

        let workspace_key = memory_object_lane_item_key(&snapshot, MemoryObjectLane::Workspace, 0)
            .expect("workspace key");
        let workspace_item_path =
            bundle_compiled_memory_item_path(&dir, MemoryObjectLane::Workspace, 0, &workspace_key);
        let workspace_item_page =
            fs::read_to_string(workspace_item_path).expect("read workspace item page");
        assert!(workspace_item_page.contains("## Hive"));
        assert!(
            workspace_item_page.contains("focus=Lorentz")
                || workspace_item_page.contains("focus=bee-1")
        );

        fs::remove_dir_all(dir).expect("cleanup memory hive page dir");
    }

    #[tokio::test]
    async fn write_bundle_memory_files_prunes_stale_compiled_memory_outputs() {
        let dir = std::env::temp_dir()
            .join(format!("memd-memory-prune-{}", uuid::Uuid::new_v4()));
        let compiled = dir.join("compiled").join("memory");
        let stale_item = compiled.join("items/working/working-99-deadbeef.md");
        let stale_lane = compiled.join("obsolete.md");
        fs::create_dir_all(stale_item.parent().expect("stale item parent"))
            .expect("create stale item dir");
        fs::write(&stale_item, "# stale compiled item\n").expect("write stale item");
        fs::write(&stale_lane, "# stale compiled lane\n").expect("write stale lane");

        let snapshot = codex_test_snapshot("demo", "main", "codex");
        write_bundle_memory_files(&dir, &snapshot, None, false)
            .await
            .expect("write bundle memory files");

        assert!(
            !stale_item.exists(),
            "stale compiled item page should be pruned on rewrite"
        );
        assert!(
            !stale_lane.exists(),
            "stale compiled lane page should be pruned on rewrite"
        );
        assert!(compiled.join("working.md").exists());
        assert!(compiled.join("context.md").exists());

        fs::remove_dir_all(dir).expect("cleanup memory prune dir");
    }

    #[tokio::test]
    async fn retire_hive_session_entry_uses_awareness_identity() {
        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state.clone(), false).await;
        state
            .session_records
            .lock()
            .expect("lock session records")
            .push(memd_schema::HiveSessionRecord {
                session: "codex-stale".to_string(),
                tab_id: Some("tab-alpha".to_string()),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@codex-stale".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("codex".to_string()),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["project:demo".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some(default_heartbeat_model()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some(base_url.clone()),
                base_url_healthy: Some(true),
                host: Some("workstation".to_string()),
                pid: Some(4242),
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
                last_seen: Utc::now() - chrono::TimeDelta::minutes(8),
            });

        let retired = retire_hive_session_entry(
            &MemdClient::new(&base_url).expect("client"),
            &ProjectAwarenessEntry {
                project_dir: "remote".to_string(),
                bundle_root: format!("remote:{base_url}:codex-stale"),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                agent: Some("codex".to_string()),
                session: Some("codex-stale".to_string()),
                tab_id: Some("tab-alpha".to_string()),
                effective_agent: Some("codex@codex-stale".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["project:demo".to_string()],
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                base_url: Some(base_url.clone()),
                presence: "stale".to_string(),
                host: Some("workstation".to_string()),
                pid: Some(4242),
                active_claims: 0,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: None,
                pressure: None,
                next_recovery: None,
                last_updated: Some(Utc::now() - chrono::TimeDelta::minutes(8)),
            },
            "recovered to codex-fresh",
        )
        .await
        .expect("retire stale entry");
        assert_eq!(retired, 1);

        let retires = state.session_retires.lock().expect("lock session retires");
        assert_eq!(retires.len(), 1);
        assert_eq!(retires[0].session, "codex-stale");
        assert_eq!(
            retires[0].reason.as_deref(),
            Some("recovered to codex-fresh")
        );
    }

    #[tokio::test]
    async fn read_bundle_resume_publishes_resume_state_and_hive_groups() {
        let dir =
            std::env::temp_dir().join(format!("memd-resume-runtime-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create resume dir");
        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state.clone(), false).await;
        fs::write(
            dir.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "hive_system": "codex",
  "hive_role": "agent",
  "hive_groups": ["openclaw-stack", "runtime-core"],
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
        .expect("write config");

        let snapshot = read_bundle_resume(
            &ResumeArgs {
                output: dir.clone(),
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
            },
            &base_url,
        )
        .await
        .expect("read bundle resume");
        write_bundle_memory_files(&dir, &snapshot, None, false)
            .await
            .expect("write bundle memory files");

        let stored = state.stored.lock().expect("lock stored");
        assert_eq!(stored.len(), 1);
        assert_eq!(
            stored[0].source_system.as_deref(),
            Some("memd-resume-state")
        );
        assert_eq!(stored[0].project.as_deref(), Some("demo"));
        assert_eq!(stored[0].workspace.as_deref(), Some("shared"));
        assert!(stored[0].tags.iter().any(|tag| tag == "resume_state"));
        drop(stored);

        let session_upserts = state.session_upserts.lock().expect("lock session upserts");
        assert!(!session_upserts.is_empty());
        let last = session_upserts.last().expect("session upsert recorded");
        assert_eq!(last.session, "codex-a");
        assert_eq!(
            last.hive_groups,
            vec![
                "openclaw-stack".to_string(),
                "project:demo".to_string(),
                "runtime-core".to_string()
            ]
        );
        assert_eq!(last.base_url.as_deref(), Some(base_url.as_str()));

        fs::remove_dir_all(dir).expect("cleanup resume dir");
    }

    #[tokio::test]
    async fn read_bundle_resume_keeps_recalled_project_fact_visible_in_bundle_memory() {
        let dir =
            std::env::temp_dir().join(format!("memd-resume-project-fact-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create resume dir");
        let state = MockRuntimeState::default();
        *state
            .context_compact_response
            .lock()
            .expect("lock context response") = Some(memd_schema::CompactContextResponse {
            route: memd_schema::RetrievalRoute::LocalFirst,
            intent: memd_schema::RetrievalIntent::CurrentTask,
            retrieval_order: vec![
                memd_schema::MemoryScope::Local,
                memd_schema::MemoryScope::Synced,
                memd_schema::MemoryScope::Project,
            ],
            records: vec![memd_schema::CompactMemoryRecord {
                id: uuid::Uuid::new_v4(),
                record: "remembered project fact: memd must preserve important user corrections"
                    .to_string(),
            }],
        });
        *state
            .working_response
            .lock()
            .expect("lock working response") = Some(memd_schema::WorkingMemoryResponse {
            route: memd_schema::RetrievalRoute::LocalFirst,
            intent: memd_schema::RetrievalIntent::CurrentTask,
            retrieval_order: vec![
                memd_schema::MemoryScope::Local,
                memd_schema::MemoryScope::Synced,
                memd_schema::MemoryScope::Project,
            ],
            budget_chars: 1600,
            used_chars: 220,
            remaining_chars: 1380,
            truncated: false,
            policy: memd_schema::WorkingMemoryPolicyState {
                admission_limit: 8,
                max_chars_per_item: 220,
                budget_chars: 1600,
                rehydration_limit: 4,
            },
            records: vec![
                memd_schema::CompactMemoryRecord {
                    id: uuid::Uuid::new_v4(),
                    record:
                        "remembered project fact: memd must preserve important user corrections"
                            .to_string(),
                },
                memd_schema::CompactMemoryRecord {
                    id: uuid::Uuid::new_v4(),
                    record: "resume state noise: synced session snapshot".to_string(),
                },
            ],
            evicted: Vec::new(),
            rehydration_queue: Vec::new(),
            traces: Vec::new(),
            semantic_consolidation: None,
        });
        let base_url = spawn_mock_runtime_server(state.clone(), false).await;
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
  "tab_id": "tab-alpha",
  "base_url": "{}",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task"
}}
"#,
                base_url
            ),
        )
        .expect("write config");

        let snapshot = read_bundle_resume(
            &ResumeArgs {
                output: dir.clone(),
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
            },
            &base_url,
        )
        .await
        .expect("read bundle resume");
        write_bundle_memory_files(&dir, &snapshot, None, false)
            .await
            .expect("write bundle memory files");
        refresh_live_bundle_event_pages(&dir, &snapshot, None).expect("refresh live event pages");

        assert!(
            snapshot.working.records[0]
                .record
                .contains("memd must preserve important user corrections")
        );

        let markdown =
            fs::read_to_string(dir.join("MEMD_MEMORY.md")).expect("read generated bundle memory");
        assert!(markdown.contains("## Scope"));
        assert!(markdown.contains("# memd memory [tab=tab-alpha]"));
        assert!(markdown.contains("session: `codex-a`"));
        assert!(markdown.contains("effective agent: `codex@codex-a`"));
        assert!(markdown.contains("memd must preserve important user corrections"));
        assert!(markdown.contains("resume state noise"));
        assert!(markdown.contains("MEMD_EVENTS.md"));
        let context_page = fs::read_to_string(dir.join("compiled/memory/context.md"))
            .expect("read compiled context page");
        assert!(context_page.contains("# memd memory object: Context [tab=tab-alpha]"));
        assert!(context_page.contains("session: `codex-a`"));
        assert!(context_page.contains("- id=") || context_page.contains("- none"));
        let working_page = fs::read_to_string(dir.join("compiled/memory/working.md"))
            .expect("read compiled working page");
        assert!(working_page.contains("# memd memory object: Working [tab=tab-alpha]"));
        assert!(working_page.contains("session: `codex-a`"));
        assert!(working_page.contains("memd must preserve important user corrections"));
        assert!(working_page.contains("items/working/"));
        let working_key = memory_object_lane_item_key(&snapshot, MemoryObjectLane::Working, 0)
            .expect("working key");
        let working_item_path =
            bundle_compiled_memory_item_path(&dir, MemoryObjectLane::Working, 0, &working_key);
        let working_item_page =
            fs::read_to_string(&working_item_path).expect("read compiled working item page");
        assert!(working_item_page.contains("# memd memory item: Working [tab=tab-alpha]"));
        assert!(working_item_page.contains("session: `codex-a`"));
        assert!(working_item_page.contains("memd must preserve important user corrections"));
        let inbox_page = fs::read_to_string(dir.join("compiled/memory/inbox.md"))
            .expect("read compiled inbox page");
        assert!(inbox_page.contains("# memd memory object: Inbox"));
        let recovery_page = fs::read_to_string(dir.join("compiled/memory/recovery.md"))
            .expect("read compiled recovery page");
        assert!(recovery_page.contains("# memd memory object: Recovery"));
        let semantic_page = fs::read_to_string(dir.join("compiled/memory/semantic.md"))
            .expect("read compiled semantic page");
        assert!(semantic_page.contains("# memd memory object: Semantic"));
        let workspace_page = fs::read_to_string(dir.join("compiled/memory/workspace.md"))
            .expect("read compiled workspace page");
        assert!(workspace_page.contains("# memd memory object: Workspace"));
        let event_log =
            fs::read_to_string(dir.join("MEMD_EVENTS.md")).expect("read generated event log");
        assert!(event_log.contains("# memd event log"));
        assert!(event_log.contains("event compiler"));
        assert!(event_log.contains("live_snapshot") || event_log.contains("resume_snapshot"));
        let event_index = fs::read_to_string(dir.join("compiled/events/latest.md"))
            .expect("read compiled event index");
        assert!(event_index.contains("# memd event index"));
        assert!(path_text_contains(&event_index, "compiled/events/items/"));
        let wakeup =
            fs::read_to_string(dir.join("MEMD_WAKEUP.md")).expect("read generated wakeup memory");
        assert!(wakeup.contains("# memd wake-up"));
        assert!(wakeup.contains("Read first."));
        assert!(wakeup.contains("memd must preserve important user corrections"));
        assert!(wakeup.contains("Default voice: caveman ultra"));
        let remember_decision = fs::read_to_string(dir.join("agents/remember-decision.sh"))
            .expect("read remember decision helper");
        let remember_short =
            fs::read_to_string(dir.join("agents/remember-short.sh")).expect("read short helper");
        let remember_long =
            fs::read_to_string(dir.join("agents/remember-long.sh")).expect("read long helper");
        let watch = fs::read_to_string(dir.join("agents/watch.sh")).expect("read watch helper");
        assert!(remember_decision.contains("args=(remember --output"));
        assert!(remember_decision.contains("--kind \"decision\""));
        assert!(remember_decision.contains("--tag \"basic-memory\""));
        assert!(remember_short.contains("args=(checkpoint --output"));
        assert!(remember_short.contains("--tag basic-memory --tag short-term"));
        assert!(remember_long.contains("--kind \"fact\""));
        assert!(remember_long.contains("--tag \"long-term\""));
        assert!(watch.contains("memd watch --root"));
        let capture_live =
            fs::read_to_string(dir.join("agents/capture-live.sh")).expect("read capture helper");
        assert!(capture_live.contains("args=(hook capture --output"));
        assert!(capture_live.contains("--tag basic-memory --tag live-capture"));
        let sync_semantic =
            fs::read_to_string(dir.join("agents/sync-semantic.sh")).expect("read semantic helper");
        assert!(sync_semantic.contains("args=(rag sync)"));
        assert!(sync_semantic.contains("MEMD_PROJECT"));
        let claude_imports =
            fs::read_to_string(dir.join("agents/CLAUDE_IMPORTS.md")).expect("read claude imports");
        assert!(claude_imports.contains(".memd/agents/remember-short.sh"));
        assert!(claude_imports.contains(".memd/agents/remember-decision.sh"));
        assert!(claude_imports.contains(".memd/agents/remember-long.sh"));
        assert!(claude_imports.contains(".memd/agents/correct-memory.sh"));
        assert!(claude_imports.contains(".memd/agents/sync-semantic.sh"));
        assert!(claude_imports.contains("@../MEMD_EVENTS.md"));
        assert!(claude_imports.contains("@CLAUDE_CODE_EVENTS.md"));

        fs::remove_dir_all(dir).expect("cleanup resume dir");
    }

    #[tokio::test]
    async fn checkpoint_refreshes_live_event_pages() {
        let dir = std::env::temp_dir().join(format!("memd-live-events-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");
        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state.clone(), false).await;
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
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task"
}}
"#,
                base_url
            ),
        )
        .expect("write config");

        checkpoint_with_bundle_defaults(
            &CheckpointArgs {
                output: dir.clone(),
                project: None,
                namespace: None,
                workspace: None,
                visibility: None,
                source_path: Some("checkpoint".to_string()),
                confidence: Some(0.9),
                ttl_seconds: Some(60),
                tag: vec!["checkpoint".to_string()],
                content: Some("refresh live event pages".to_string()),
                input: None,
                stdin: false,
            },
            &base_url,
        )
        .await
        .expect("checkpoint");

        let snapshot = read_bundle_resume(
            &autoresearch_resume_args_with_limits(&dir, 8, 4, true),
            &base_url,
        )
        .await
        .expect("read bundle resume");
        write_bundle_memory_files(&dir, &snapshot, None, false)
            .await
            .expect("write bundle memory files");
        refresh_live_bundle_event_pages(&dir, &snapshot, None).expect("refresh live event pages");

        let events = read_bundle_event_log(&dir).expect("read bundle event log");
        assert_eq!(events.len(), 1);
        assert!(events[0].summary.contains("project=demo"));
        assert!(events[0].summary.contains("tokens="));
        let root_events =
            fs::read_to_string(dir.join("MEMD_EVENTS.md")).expect("read generated event log");
        assert!(root_events.contains("# memd event log"));
        assert!(root_events.contains("event compiler"));
        assert!(root_events.contains("compiled/events/"));
        let compiled = fs::read_to_string(dir.join("compiled/events/latest.md"))
            .expect("read compiled event index");
        assert!(compiled.contains("# memd event index"));
        fs::remove_dir_all(dir).expect("cleanup live events dir");
    }

    #[test]
    fn compiled_memory_search_resolves_lane_and_item_pages() {
        let root = std::env::temp_dir().join(format!("memd-memory-query-{}", uuid::Uuid::new_v4()));
        let compiled = root.join("compiled").join("memory");
        let items = compiled.join("items").join("working");
        fs::create_dir_all(&items).expect("create compiled memory dir");
        fs::write(
            compiled.join("working.md"),
            "# memd memory object: Working\n\n- [open](items/working/working-01-abcd1234.md)\n",
        )
        .expect("write lane page");
        fs::write(
            items.join("working-01-abcd1234.md"),
            "# memd memory item: Working\n\n- id=abc123 record=\"current_task: keep memory visible\"\n",
        )
        .expect("write item page");

        let lane_hits = search_compiled_memory_pages(&root, "working", 8)
            .expect("search compiled memory pages");
        assert!(
            lane_hits
                .iter()
                .any(|hit| path_text_ends_with(&hit.path, "working.md"))
        );
        assert!(
            lane_hits
                .iter()
                .any(|hit| path_text_ends_with(&hit.path, "working-01-abcd1234.md"))
        );

        let resolved =
            resolve_compiled_memory_page(&root, "working").expect("resolve compiled memory page");
        assert!(path_text_ends_with(&resolved, "working.md"));

        let item_resolved = resolve_compiled_memory_page(&root, "working-01-abcd1234")
            .expect("resolve compiled memory item");
        assert!(path_text_ends_with(
            &item_resolved,
            "working-01-abcd1234.md"
        ));

        fs::remove_dir_all(root).expect("cleanup memory query temp dir");
    }

    #[test]
    fn compiled_memory_lane_shortcut_resolves_lane_page() {
        let root = std::env::temp_dir().join(format!("memd-memory-lane-{}", uuid::Uuid::new_v4()));
        let compiled = root.join("compiled").join("memory");
        fs::create_dir_all(&compiled).expect("create compiled memory dir");
        fs::write(
            compiled.join("working.md"),
            "# memd memory object: Working\n\n- [open](items/working/working-01-abcd1234.md)\n",
        )
        .expect("write lane page");

        let resolved =
            resolve_compiled_memory_page(&root, "working").expect("resolve lane shortcut");
        assert!(path_text_ends_with(&resolved, "working.md"));

        fs::remove_dir_all(root).expect("cleanup memory lane temp dir");
    }

    #[test]
    fn compiled_event_search_resolves_kind_and_item_pages() {
        let root = std::env::temp_dir().join(format!("memd-event-query-{}", uuid::Uuid::new_v4()));
        let compiled = root.join("compiled").join("events");
        let items = compiled.join("items").join("live_snapshot");
        fs::create_dir_all(&items).expect("create compiled event dir");
        fs::write(
            compiled.join("live_snapshot.md"),
            "# memd event lane: Live Snapshot\n\n- [event-01-abcd1234](items/live_snapshot/event-01-abcd1234.md)\n",
        )
        .expect("write event lane page");
        fs::write(
            items.join("event-01-abcd1234.md"),
            "# memd event item: Live Snapshot\n\n- summary: live_snapshot project=demo pressure=\"trim context\"\n",
        )
        .expect("write event item page");

        let hits =
            search_compiled_event_pages(&root, "pressure", 8).expect("search compiled event pages");
        assert!(!hits.is_empty());
        assert!(
            hits.iter()
                .any(|hit| hit.path.ends_with("event-01-abcd1234.md"))
        );

        let index = render_compiled_event_index(&root).expect("render compiled event index");
        assert!(index.kind_count >= 1);
        assert!(index.item_count >= 1);
        assert!(
            index
                .pages
                .iter()
                .any(|page| page.ends_with("live_snapshot.md"))
        );
        assert!(
            index
                .pages
                .iter()
                .any(|page| page.contains("event-01-abcd1234.md"))
        );

        let resolved = resolve_compiled_event_page(&root, "live_snapshot")
            .expect("resolve compiled event page");
        assert!(resolved.ends_with("live_snapshot.md"));

        let item_resolved = resolve_compiled_event_page(&root, "event-01-abcd1234")
            .expect("resolve compiled event item");
        assert!(item_resolved.ends_with("event-01-abcd1234.md"));

        fs::remove_dir_all(root).expect("cleanup event query temp dir");
    }

    #[test]
    fn compiled_event_index_summary_and_json_include_lane_counts() {
        let root = std::env::temp_dir().join(format!("memd-event-index-{}", uuid::Uuid::new_v4()));
        let compiled = root.join("compiled").join("events");
        let items = compiled.join("items").join("live_snapshot");
        fs::create_dir_all(&items).expect("create compiled event dir");
        fs::write(
            compiled.join("live_snapshot.md"),
            "# memd event lane: Live Snapshot\n\n- [event-01-abcd1234](items/live_snapshot/event-01-abcd1234.md)\n",
        )
        .expect("write event lane page");
        fs::write(
            items.join("event-01-abcd1234.md"),
            "# memd event item: Live Snapshot\n\n- summary: live_snapshot project=demo pressure=\"trim context\"\n",
        )
        .expect("write event item page");

        let index = render_compiled_event_index(&root).expect("render compiled event index");
        let summary = render_compiled_event_index_summary(&root, &index);
        assert!(summary.contains("event index"));
        assert!(summary.contains("kinds=1"));
        assert!(summary.contains("items=1"));
        let json = render_compiled_event_index_json(&root, &index);
        assert_eq!(json.root, root.display().to_string());
        assert_eq!(json.kind_count, 1);
        assert_eq!(json.item_count, 1);
        assert!(
            json.pages
                .iter()
                .any(|page| page.ends_with("live_snapshot.md"))
        );
        assert!(
            json.pages
                .iter()
                .any(|page| page.contains("event-01-abcd1234.md"))
        );

        fs::remove_dir_all(root).expect("cleanup event index temp dir");
    }

    #[test]
    fn compiled_memory_item_target_takes_precedence_over_lane_and_open() {
        let args = MemoryArgs {
            root: None,
            query: None,
            open: Some("working".to_string()),
            lane: Some("working".to_string()),
            item: Some("working-01-abcd1234".to_string()),
            list: false,
            lanes_only: false,
            items_only: false,
            filter: None,
            grouped: false,
            expand_items: false,
            json: false,
            limit: 12,
            summary: true,
            quality: false,
        };

        assert_eq!(
            compiled_memory_target(&args).as_deref(),
            Some("working-01-abcd1234")
        );
    }

    #[test]
    fn compiled_memory_index_lists_lane_and_item_pages() {
        let root = std::env::temp_dir().join(format!("memd-memory-index-{}", uuid::Uuid::new_v4()));
        let compiled = root.join("compiled").join("memory");
        let items = compiled.join("items").join("working");
        fs::create_dir_all(&items).expect("create compiled memory dir");
        fs::write(
            compiled.join("working.md"),
            "# memd memory object: Working\n\n- [open](items/working/working-01-abcd1234.md)\n",
        )
        .expect("write lane page");
        fs::write(
            items.join("working-01-abcd1234.md"),
            "# memd memory item: Working\n\n- id=abc123 record=\"current_task: keep memory visible\"\n",
        )
        .expect("write item page");

        let index = render_compiled_memory_index(&root).expect("render compiled memory index");
        assert!(index.lane_count >= 1);
        assert!(index.item_count >= 1);
        assert!(
            index
                .pages
                .iter()
                .any(|page| path_text_ends_with(page, "working.md"))
        );
        assert!(
            index
                .pages
                .iter()
                .any(|page| path_text_contains(page, "working-01-abcd1234.md"))
        );

        fs::remove_dir_all(root).expect("cleanup memory index temp dir");
    }

    #[test]
    fn compiled_memory_index_grouped_markdown_uses_lane_sections_and_links() {
        let root = std::env::temp_dir().join(format!(
            "memd-memory-index-grouped-{}",
            uuid::Uuid::new_v4()
        ));
        let compiled = root.join("compiled").join("memory");
        let items = compiled.join("items").join("working");
        fs::create_dir_all(&items).expect("create compiled memory dir");
        fs::write(compiled.join("working.md"), "# Working\n").expect("write lane page");
        fs::write(items.join("working-01-abcd1234.md"), "# Item\n").expect("write item page");

        let index = render_compiled_memory_index(&root).expect("render compiled memory index");
        let markdown = render_compiled_memory_index_grouped_markdown(&root, &index, true);
        assert!(markdown.contains("## Working"));
        assert!(markdown.contains("[Working]("));
        assert!(path_text_contains(&markdown, "compiled/memory/working.md"));
        assert!(markdown.contains("[working-01-abcd1234]("));
        assert!(path_text_contains(&markdown, "working-01-abcd1234.md"));

        fs::remove_dir_all(root).expect("cleanup memory index grouped temp dir");
    }

    #[test]
    fn compiled_memory_index_grouped_markdown_collapses_items_by_default() {
        let root = std::env::temp_dir().join(format!(
            "memd-memory-index-grouped-compact-{}",
            uuid::Uuid::new_v4()
        ));
        let compiled = root.join("compiled").join("memory");
        let items = compiled.join("items").join("working");
        fs::create_dir_all(&items).expect("create compiled memory dir");
        fs::write(compiled.join("working.md"), "# Working\n").expect("write lane page");
        fs::write(items.join("working-01-abcd1234.md"), "# Item\n").expect("write item page");
        fs::write(items.join("working-02-fedcba98.md"), "# Item 2\n").expect("write item page");

        let index = render_compiled_memory_index(&root).expect("render compiled memory index");
        let markdown = render_compiled_memory_index_grouped_markdown(&root, &index, false);
        assert!(markdown.contains("## Working"));
        assert!(markdown.contains("[Working]("));
        assert!(markdown.contains("+2 more item(s)") || markdown.contains("+1 more item(s)"));
        assert!(!markdown.contains("working-02-fedcba98"));

        fs::remove_dir_all(root).expect("cleanup memory index grouped compact temp dir");
    }

    #[test]
    fn compiled_memory_index_json_exports_paths_and_counts() {
        let root =
            std::env::temp_dir().join(format!("memd-memory-index-json-{}", uuid::Uuid::new_v4()));
        let compiled = root.join("compiled").join("memory");
        let items = compiled.join("items").join("working");
        fs::create_dir_all(&items).expect("create compiled memory dir");
        fs::write(compiled.join("working.md"), "# Working\n").expect("write lane page");
        fs::write(items.join("working-01-abcd1234.md"), "# Item\n").expect("write item page");

        let index = render_compiled_memory_index(&root).expect("render compiled memory index");
        let json = render_compiled_memory_index_json(&root, &index);
        assert_eq!(json.root, root.display().to_string());
        assert_eq!(json.tab_id, "none");
        assert_eq!(json.lane_count, 1);
        assert_eq!(json.item_count, 1);
        assert!(
            json.pages
                .iter()
                .any(|page| path_text_ends_with(page, "working.md"))
        );
        assert!(
            json.pages
                .iter()
                .any(|page| path_text_contains(page, "working-01-abcd1234.md"))
        );
        assert!(json.entries.iter().any(|entry| {
            entry.kind == "lane" && entry.lane == "working" && entry.relative_path == "working.md"
        }));
        assert!(json.entries.iter().any(|entry| {
            entry.kind == "item"
                && entry.lane == "working"
                && normalize_path_text(&entry.relative_path)
                    == "items/working/working-01-abcd1234.md"
        }));

        fs::remove_dir_all(root).expect("cleanup memory index json temp dir");
    }

    #[test]
    fn compiled_memory_index_summary_stays_compact() {
        let root = std::env::temp_dir().join(format!(
            "memd-memory-index-summary-{}",
            uuid::Uuid::new_v4()
        ));
        let compiled = root.join("compiled").join("memory");
        let items = compiled.join("items").join("working");
        fs::create_dir_all(&items).expect("create compiled memory dir");
        fs::write(compiled.join("working.md"), "# Working\n").expect("write lane page");
        fs::write(items.join("working-01-abcd1234.md"), "# Item\n").expect("write item page");

        let index = render_compiled_memory_index(&root).expect("render compiled memory index");
        let summary = render_compiled_memory_index_summary(&root, &index);
        assert!(summary.contains("memory index root="));
        assert!(summary.contains("lanes=1"));
        assert!(summary.contains("items=1"));
        assert!(summary.contains("pages=2"));

        fs::remove_dir_all(root).expect("cleanup memory index summary temp dir");
    }

    #[test]
    fn compiled_memory_search_ranks_exact_path_matches_before_generic_hits() {
        let root =
            std::env::temp_dir().join(format!("memd-memory-search-rank-{}", uuid::Uuid::new_v4()));
        let compiled = root.join("compiled").join("memory");
        fs::create_dir_all(&compiled).expect("create compiled memory dir");
        fs::write(
            compiled.join("working.md"),
            "# Working\n\nworking memory is the current lane.\n",
        )
        .expect("write working page");
        fs::write(
            compiled.join("notes.md"),
            "# Notes\n\nthis working note is more generic.\n",
        )
        .expect("write notes page");

        let hits = search_compiled_memory_pages(&root, "working", 2).expect("search memory");
        assert_eq!(hits.len(), 2);
        assert!(hits[0].score >= hits[1].score);
        assert!(path_text_ends_with(&hits[0].path, "working.md"));
        assert!(!hits[0].reasons.is_empty());

        fs::remove_dir_all(root).expect("cleanup memory search temp dir");
    }

    #[test]
    fn compiled_memory_quality_report_scores_scope_and_probes() {
        let root =
            std::env::temp_dir().join(format!("memd-memory-quality-{}", uuid::Uuid::new_v4()));
        let compiled = root.join("compiled").join("memory");
        fs::create_dir_all(&compiled).expect("create compiled memory dir");
        fs::write(
            root.join("config.json"),
            r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "session-alpha",
  "tab_id": "tab-alpha"
}
"#,
        )
        .expect("write runtime config");
        fs::write(
            root.join("MEMD_MEMORY.md"),
            "# memd memory\n\n## Scope\n\n- source_note: [[Working]]\n",
        )
        .expect("write memory surface");
        fs::write(
            compiled.join("working.md"),
            "# Working\n\nworking memory is the current lane.\n",
        )
        .expect("write working page");

        let report = build_compiled_memory_quality_report(&root).expect("build quality report");
        assert_eq!(report.benchmark_target, "supermemory");
        assert!(report.score > 0);
        assert!(report.page_count >= 1);
        assert!(
            report
                .dimensions
                .iter()
                .any(|dimension| dimension.name == "freshness")
        );
        assert!(
            report
                .dimensions
                .iter()
                .any(|dimension| dimension.name == "contradiction")
        );
        assert!(
            report
                .dimensions
                .iter()
                .any(|dimension| dimension.name == "token_efficiency")
        );
        assert!(
            report
                .probes
                .iter()
                .any(|probe| probe.query == "working" && probe.best_score > 0)
        );
        assert!(
            report
                .recommendations
                .iter()
                .any(|rec| rec.contains("surface")
                    || rec.contains("scope")
                    || rec.contains("rank"))
        );

        fs::remove_dir_all(root).expect("cleanup memory quality temp dir");
    }

    #[test]
    fn compiled_memory_index_filters_lanes_items_and_text() {
        let root =
            std::env::temp_dir().join(format!("memd-memory-index-filter-{}", uuid::Uuid::new_v4()));
        let compiled = root.join("compiled").join("memory");
        let items = compiled.join("items").join("working");
        fs::create_dir_all(&items).expect("create compiled memory dir");
        fs::write(compiled.join("working.md"), "# Working\n").expect("write lane page");
        fs::write(items.join("working-01-abcd1234.md"), "# Item\n").expect("write item page");

        let index = render_compiled_memory_index(&root).expect("render compiled memory index");
        let lanes_only = filter_compiled_memory_index(index.clone(), true, false, None);
        assert!(
            lanes_only
                .pages
                .iter()
                .all(|page| !path_text_contains(page, "/items/"))
        );
        assert_eq!(lanes_only.lane_count, 1);
        assert_eq!(lanes_only.item_count, 0);

        let items_only = filter_compiled_memory_index(index.clone(), false, true, None);
        assert!(
            items_only
                .pages
                .iter()
                .all(|page| path_text_contains(page, "/items/"))
        );
        assert_eq!(items_only.lane_count, 0);
        assert_eq!(items_only.item_count, 1);

        let filtered = filter_compiled_memory_index(index, false, false, Some("working-01"));
        assert!(
            filtered
                .pages
                .iter()
                .any(|page| path_text_contains(page, "working-01-abcd1234.md"))
        );
        assert_eq!(filtered.pages.len(), 1);

        fs::remove_dir_all(root).expect("cleanup memory index filter temp dir");
    }

    #[test]
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
    }

    #[tokio::test]
