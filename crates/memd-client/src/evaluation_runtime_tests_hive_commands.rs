    use super::*;

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
        assert!(summary.contains("queen=session-queen"));
        assert!(summary.contains("reroute Lorentz"));
        assert!(summary.contains("queen_deny session-avicenna"));
        assert!(summary.contains("## Action Cards"));
        assert!(summary.contains("follow=memd hive follow --session session-lorentz --summary"));
        assert!(
            summary.contains("reroute=memd hive queen --reroute-session session-lorentz --summary")
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
            overlap_risks: vec![
                "Lorentz vs Avicenna on crates/memd-client/src/main.rs".to_string(),
            ],
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
        let dir = std::env::temp_dir().join(format!(
            "memd-hive-handoff-follow-{}",
            uuid::Uuid::new_v4()
        ));
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
        let dir =
            std::env::temp_dir().join(format!("memd-hive-board-retire-{}", uuid::Uuid::new_v4()));
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
        let dir =
            std::env::temp_dir().join(format!("memd-heartbeat-intent-{}", uuid::Uuid::new_v4()));
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

