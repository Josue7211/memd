    use super::*;

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

    #[test]
    fn agent_and_attach_scripts_default_to_current_task_intent() {
        let dir = std::env::temp_dir().join(format!("memd-hive-launcher-{}", uuid::Uuid::new_v4()));
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
        set_bundle_hive_project_state(&dir, true, Some("project:demo"), Some(Utc::now()))
            .expect("enable hive project");

        let shell = render_agent_shell_profile(&dir, Some("codex"));
        let ps1 = render_agent_ps1_profile(&dir, Some("codex"));
        let attach = render_attach_snippet("bash", &dir).expect("attach snippet");
        let lookup = render_lookup_shell_profile(&dir, &[], &[]);
        let recall_decisions = render_lookup_shell_profile(&dir, &["decision", "constraint"], &[]);
        let recall_design = render_lookup_shell_profile(
            &dir,
            &["preference", "constraint", "decision"],
            &["design-memory"],
        );
        let remember_decision = render_remember_shell_profile(&dir, "decision", &["basic-memory"]);
        let remember_short = render_checkpoint_shell_profile(&dir);
        let remember_long =
            render_remember_shell_profile(&dir, "fact", &["basic-memory", "long-term"]);
        let watch = render_watch_shell_profile(&dir);
        let capture_live = render_capture_shell_profile(&dir, "capture-live");
        let sync_semantic = render_rag_sync_shell_profile(&dir);

        assert!(shell.contains("--intent current_task"));
        assert!(ps1.contains("--intent current_task"));
        assert!(shell.contains("MEMD_TAB_ID"));
        assert!(ps1.contains("MEMD_TAB_ID"));
        assert!(shell.contains("MEMD_WORKER_NAME"));
        assert!(ps1.contains("MEMD_WORKER_NAME"));
        assert!(shell.contains("nohup \"$MEMD_BUNDLE_ROOT/agents/watch.sh\""));
        assert!(ps1.contains("Start-Process -WindowStyle Hidden"));
        assert!(attach.contains("--intent current_task"));
        assert!(shell.contains("memd wake --output \"$MEMD_BUNDLE_ROOT\" --intent current_task --write >/dev/null 2>&1 || true"));
        assert!(ps1.contains("try { memd wake --output $env:MEMD_BUNDLE_ROOT --intent current_task --write | Out-Null } catch { }"));
        assert!(shell.contains("export MEMD_BASE_URL=\"http://100.104.154.24:8787\""));
        assert!(ps1.contains("$env:MEMD_BASE_URL = \"http://100.104.154.24:8787\""));
        assert!(attach.contains("export MEMD_BASE_URL=\"http://100.104.154.24:8787\""));
        assert!(shell.contains("memd heartbeat --output \"$MEMD_BUNDLE_ROOT\" --watch"));
        assert!(ps1.contains("FilePath memd -ArgumentList @('heartbeat'"));
        assert!(attach.contains("memd heartbeat --output \"$MEMD_BUNDLE_ROOT\" --watch"));
        assert!(
            shell
                .contains("memd hive --output \"$MEMD_BUNDLE_ROOT\" --publish-heartbeat --summary")
        );
        assert!(
            ps1.contains("memd hive --output $env:MEMD_BUNDLE_ROOT --publish-heartbeat --summary")
        );
        assert!(shell.contains("exec memd wake"));
        assert!(ps1.contains("memd wake"));
        assert!(attach.contains("memd wake"));
        assert!(lookup.contains("lookup --output \"$MEMD_BUNDLE_ROOT\""));
        assert!(lookup.contains("--route project_first"));
        assert!(recall_decisions.contains("--kind \"decision\""));
        assert!(recall_decisions.contains("--kind \"constraint\""));
        assert!(recall_design.contains("--tag \"design-memory\""));
        assert!(remember_decision.contains("remember --output"));
        assert!(remember_decision.contains("--kind \"decision\""));
        assert!(remember_short.contains("checkpoint --output"));
        assert!(remember_short.contains("--tag basic-memory --tag short-term"));
        assert!(remember_long.contains("--kind \"fact\""));
        assert!(remember_long.contains("--tag \"long-term\""));
        assert!(watch.contains("memd watch --root"));
        assert!(capture_live.contains("hook capture --output"));
        assert!(capture_live.contains("--tag basic-memory"));
        assert!(sync_semantic.contains("rag sync"));
        assert!(sync_semantic.contains("MEMD_PROJECT"));
        assert!(workspace_path_should_trigger(Path::new("src/main.rs")));
        assert!(workspace_path_should_trigger(Path::new("docs/guide.md")));
        assert!(workspace_path_should_trigger(Path::new("Cargo.toml")));
        assert!(!workspace_path_should_trigger(Path::new(
            ".watch-smoke-new"
        )));
        assert!(!workspace_path_should_trigger(Path::new(
            ".memd/config.json"
        )));

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn hook_capture_can_build_promoted_memory_args() {
        let args = HookCaptureArgs {
            output: PathBuf::from(".memd"),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            source_path: Some("hook-capture".to_string()),
            confidence: Some(0.6),
            ttl_seconds: Some(3600),
            content: None,
            input: None,
            stdin: true,
            tag: vec!["episodic".to_string()],
            promote_kind: Some("decision".to_string()),
            promote_scope: Some("project".to_string()),
            promote_supersede: vec!["123e4567-e89b-12d3-a456-426614174000".to_string()],
            promote_supersede_query: None,
            promote_tag: vec!["10-star".to_string(), "product-direction".to_string()],
            promote_confidence: Some(0.95),
            summary: true,
        };

        let remember = remember_args_from_hook_capture(&args, "ship wake first".to_string());
        assert_eq!(remember.kind.as_deref(), Some("decision"));
        assert_eq!(remember.scope.as_deref(), Some("project"));
        assert_eq!(remember.project.as_deref(), Some("memd"));
        assert_eq!(remember.namespace.as_deref(), Some("main"));
        assert_eq!(remember.workspace.as_deref(), Some("shared"));
        assert_eq!(remember.visibility.as_deref(), Some("workspace"));
        assert_eq!(remember.content.as_deref(), Some("ship wake first"));
        assert_eq!(remember.confidence, Some(0.95));
        assert_eq!(remember.source_path.as_deref(), Some("hook-capture"));
        assert_eq!(remember.tag, vec!["10-star", "product-direction"]);
        assert_eq!(
            remember.supersede,
            vec!["123e4567-e89b-12d3-a456-426614174000"]
        );
    }

    #[test]
    fn hook_capture_can_infer_promote_kind_from_prefix() {
        assert_eq!(
            infer_promote_kind_from_capture("decision: keep wake"),
            Some("decision")
        );
        assert_eq!(
            infer_promote_kind_from_capture("Preference: bold UI"),
            Some("preference")
        );
        assert_eq!(infer_promote_kind_from_capture("random note"), None);
    }

    #[test]
    fn hook_capture_can_infer_supersede_query_from_correction_text() {
        assert_eq!(
            infer_supersede_query_from_capture(
                "corrected fact: hosted backend health does not prove usable agent memory"
            )
            .as_deref(),
            Some("hosted backend health does not prove usable agent memory")
        );
        assert_eq!(
            infer_supersede_query_from_capture("correction: stale recall ranking"),
            Some("stale recall ranking".to_string())
        );
        assert_eq!(
            infer_supersede_query_from_capture("decision: keep wake"),
            None
        );
    }

    #[test]
    fn hook_capture_uses_inferred_supersede_query_when_none_provided() {
        let args = HookCaptureArgs {
            output: PathBuf::from(".memd"),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
            visibility: None,
            source_path: None,
            confidence: Some(0.6),
            ttl_seconds: None,
            content: None,
            input: None,
            stdin: true,
            tag: vec!["episodic".to_string()],
            promote_kind: Some("fact".to_string()),
            promote_scope: Some("project".to_string()),
            promote_supersede: Vec::new(),
            promote_supersede_query: None,
            promote_tag: Vec::new(),
            promote_confidence: Some(0.9),
            summary: true,
        };

        assert_eq!(
            effective_hook_capture_supersede_query(
                &args,
                "corrected fact: hosted backend health does not prove usable agent memory",
            )
            .as_deref(),
            Some("hosted backend health does not prove usable agent memory")
        );
    }

    #[test]
    fn hook_capture_builds_condensed_supersede_query() {
        assert_eq!(
            condensed_supersede_query("hosted backend health does not prove usable agent memory")
                .as_deref(),
            Some("hosted backend health")
        );
    }

    #[test]
    fn hook_capture_supersede_query_candidates_include_fallback() {
        let queries =
            supersede_query_candidates("hosted backend health does not prove usable agent memory");
        assert_eq!(
            queries,
            vec![
                "hosted backend health does not prove usable agent memory".to_string(),
                "hosted backend health".to_string(),
                "hosted backend".to_string(),
                "backend health".to_string(),
            ]
        );
    }

    #[test]
    fn hook_capture_formats_supersede_candidate_hits() {
        let hit = SupersedeCandidateHit {
            query: "backend health".to_string(),
            ids: vec!["12345678-1234-1234-1234-123456789abc".to_string()],
            statuses: vec!["stale".to_string()],
            kinds: vec!["fact".to_string()],
            previews: vec!["hosted backend health looked good".to_string()],
        };

        assert_eq!(
            format_supersede_candidate_hit(&hit),
            "backend health=>12345678:stale:fact:hosted backend health looked good"
        );
    }

    #[test]
    fn hook_capture_recent_scan_ranks_overlap_and_prefers_stale() {
        let matched = rank_recent_supersede_candidates(
            "hosted backend health does not prove usable agent memory",
            vec![
                memd_schema::MemoryItem {
                    id: uuid::Uuid::new_v4(),
                    content: "backend health looked good".to_string(),
                    redundancy_key: None,
                    belief_branch: None,
                    preferred: false,
                    kind: memd_schema::MemoryKind::Fact,
                    scope: memd_schema::MemoryScope::Project,
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("shared".to_string()),
                    visibility: memd_schema::MemoryVisibility::Workspace,
                    source_agent: None,
                    source_system: Some("memd".to_string()),
                    source_path: None,
                    source_quality: Some(memd_schema::SourceQuality::Canonical),
                    confidence: 0.6,
                    ttl_seconds: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    last_verified_at: None,
                    supersedes: Vec::new(),
                    tags: Vec::new(),
                    status: memd_schema::MemoryStatus::Active,
                    stage: memd_schema::MemoryStage::Canonical,
                },
                memd_schema::MemoryItem {
                    id: uuid::Uuid::new_v4(),
                    content: "hosted backend health was enough proof".to_string(),
                    redundancy_key: None,
                    belief_branch: None,
                    preferred: false,
                    kind: memd_schema::MemoryKind::Fact,
                    scope: memd_schema::MemoryScope::Project,
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("shared".to_string()),
                    visibility: memd_schema::MemoryVisibility::Workspace,
                    source_agent: None,
                    source_system: Some("memd".to_string()),
                    source_path: None,
                    source_quality: Some(memd_schema::SourceQuality::Canonical),
                    confidence: 0.6,
                    ttl_seconds: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    last_verified_at: None,
                    supersedes: Vec::new(),
                    tags: Vec::new(),
                    status: memd_schema::MemoryStatus::Stale,
                    stage: memd_schema::MemoryStage::Canonical,
                },
            ],
        );

        assert_eq!(matched.len(), 2);
        assert_eq!(matched[0].status, memd_schema::MemoryStatus::Stale);
        assert!(matched[0].content.contains("enough proof"));
    }

    #[test]
    fn hook_capture_auto_promotion_sets_inferred_kind() {
        let args = HookCaptureArgs {
            output: PathBuf::from(".memd"),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
            visibility: None,
            source_path: None,
            confidence: Some(0.6),
            ttl_seconds: None,
            content: None,
            input: None,
            stdin: true,
            tag: vec!["episodic".to_string()],
            promote_kind: None,
            promote_scope: Some("project".to_string()),
            promote_supersede: Vec::new(),
            promote_supersede_query: None,
            promote_tag: Vec::new(),
            promote_confidence: Some(0.9),
            summary: true,
        };

        let kind =
            effective_hook_capture_promote_kind(&args, "decision: keep wake").expect("infer kind");
        let remember = remember_args_from_effective_hook_capture(
            &args,
            "decision: keep wake".to_string(),
            kind,
            Vec::new(),
        );
        assert_eq!(remember.kind.as_deref(), Some("decision"));
        assert!(remember.tag.iter().any(|tag| tag == "auto-promoted"));
        assert!(remember.tag.iter().any(|tag| tag == "decision"));
    }

    #[test]
    fn hook_capture_auto_promotion_infers_design_and_correction_tags() {
        let args = HookCaptureArgs {
            output: PathBuf::from(".memd"),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
            visibility: None,
            source_path: None,
            confidence: Some(0.6),
            ttl_seconds: None,
            content: None,
            input: None,
            stdin: true,
            tag: vec!["episodic".to_string()],
            promote_kind: None,
            promote_scope: Some("project".to_string()),
            promote_supersede: vec!["123e4567-e89b-12d3-a456-426614174000".to_string()],
            promote_supersede_query: None,
            promote_tag: Vec::new(),
            promote_confidence: Some(0.9),
            summary: true,
        };

        let remember = remember_args_from_effective_hook_capture(
            &args,
            "preference: design memory should preserve UX/UI taste".to_string(),
            "preference".to_string(),
            vec![uuid::Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").expect("uuid")],
        );
        assert!(remember.tag.iter().any(|tag| tag == "design-memory"));
        assert!(remember.tag.iter().any(|tag| tag == "correction"));
        assert!(remember.tag.iter().any(|tag| tag == "preference"));
    }

    #[test]
    fn lookup_request_uses_bundle_defaults_and_active_canonical_filters() {
        let args = LookupArgs {
            output: PathBuf::from(".memd"),
            query: "what did we decide?".to_string(),
            project: None,
            namespace: None,
            workspace: None,
            visibility: None,
            route: None,
            intent: None,
            kind: Vec::new(),
            tag: vec!["10-star".to_string()],
            include_stale: false,
            limit: None,
            verbose: false,
            json: false,
        };
        let runtime = BundleRuntimeConfig {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
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
            route: Some("auto".to_string()),
            intent: Some("current_task".to_string()),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            heartbeat_model: None,
            auto_short_term_capture: true,
            authority_policy: BundleAuthorityPolicy::default(),
            authority_state: BundleAuthorityState::default(),
        };

        let req = build_lookup_request(&args, Some(&runtime)).expect("build lookup request");
        assert_eq!(req.query.as_deref(), Some("what did we decide?"));
        assert_eq!(req.project.as_deref(), Some("memd"));
        assert_eq!(req.namespace.as_deref(), Some("main"));
        assert_eq!(req.workspace.as_deref(), Some("shared"));
        assert_eq!(
            req.visibility,
            Some(memd_schema::MemoryVisibility::Workspace)
        );
        assert_eq!(req.route, Some(memd_schema::RetrievalRoute::ProjectFirst));
        assert_eq!(req.intent, Some(memd_schema::RetrievalIntent::General));
        assert_eq!(req.statuses, vec![memd_schema::MemoryStatus::Active]);
        assert_eq!(req.stages, vec![memd_schema::MemoryStage::Canonical]);
        assert!(req.kinds.contains(&memd_schema::MemoryKind::Decision));
        assert!(req.kinds.contains(&memd_schema::MemoryKind::Preference));
    }

    #[tokio::test]
    async fn lookup_with_fallbacks_recovers_tagged_memory_when_query_is_too_fuzzy() {
        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state.clone(), false).await;
        let client = MemdClient::new(&base_url).expect("client");
        let req = SearchMemoryRequest {
            query: Some("what default response style should Codex use".to_string()),
            route: Some(memd_schema::RetrievalRoute::All),
            intent: Some(memd_schema::RetrievalIntent::General),
            scopes: vec![
                memd_schema::MemoryScope::Project,
                memd_schema::MemoryScope::Synced,
                memd_schema::MemoryScope::Global,
            ],
            kinds: vec![memd_schema::MemoryKind::Preference],
            statuses: vec![memd_schema::MemoryStatus::Active],
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
            visibility: None,
            belief_branch: None,
            source_agent: None,
            tags: vec!["caveman-ultra".to_string()],
            stages: vec![memd_schema::MemoryStage::Canonical],
            limit: Some(6),
            max_chars_per_item: Some(280),
        };

        let response = lookup_with_fallbacks(
            &client,
            &req,
            "what default response style should Codex use",
        )
        .await
        .expect("lookup fallback");
        assert_eq!(response.items.len(), 1);
        assert_eq!(response.items[0].kind, memd_schema::MemoryKind::Preference);
        assert!(response.items[0].content.contains("caveman ultra"));
        assert!(
            *state.search_count.lock().expect("lock search count") >= 2,
            "expected fallback search attempts"
        );
    }

    #[test]
    fn lookup_markdown_mentions_pre_answer_protocol() {
        let response = memd_schema::SearchMemoryResponse {
            route: memd_schema::RetrievalRoute::Auto,
            intent: memd_schema::RetrievalIntent::CurrentTask,
            items: vec![memd_schema::MemoryItem {
                id: uuid::Uuid::new_v4(),
                content: "decision: wake is the startup surface".to_string(),
                redundancy_key: None,
                belief_branch: None,
                preferred: false,
                kind: memd_schema::MemoryKind::Decision,
                scope: memd_schema::MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                visibility: memd_schema::MemoryVisibility::Workspace,
                source_agent: Some("codex".to_string()),
                source_system: Some("memd".to_string()),
                source_path: None,
                source_quality: Some(memd_schema::SourceQuality::Canonical),
                confidence: 0.95,
                ttl_seconds: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec!["10-star".to_string()],
                status: memd_schema::MemoryStatus::Active,
                stage: memd_schema::MemoryStage::Canonical,
            }],
        };

        let markdown = render_lookup_markdown("startup surface", &response, false);
        assert!(markdown.contains("# memd lookup"));
        assert!(markdown.contains("decision: wake is the startup surface"));
        assert!(markdown.contains("Use recalled items before answering"));
    }

    #[tokio::test]
    async fn hook_capture_can_supersede_stale_memory_after_promotion() {
        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state.clone(), false).await;
        let dir =
            std::env::temp_dir().join(format!("memd-hook-supersede-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");

        let args = HookCaptureArgs {
            output: dir.clone(),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            source_path: Some("hook-capture".to_string()),
            confidence: Some(0.8),
            ttl_seconds: Some(3600),
            content: None,
            input: None,
            stdin: true,
            tag: vec!["episodic".to_string()],
            promote_kind: Some("fact".to_string()),
            promote_scope: Some("project".to_string()),
            promote_supersede: vec!["123e4567-e89b-12d3-a456-426614174000".to_string()],
            promote_supersede_query: None,
            promote_tag: vec!["correction".to_string()],
            promote_confidence: Some(0.99),
            summary: true,
        };

        let (supersede_targets, diagnostics) = find_hook_capture_supersede_targets(
            &base_url,
            &args,
            "corrected fact: wake is the startup surface",
        )
        .await
        .expect("find supersede targets");
        assert_eq!(supersede_targets.len(), 1);

        let promoted = remember_with_bundle_defaults(
            &remember_args_from_effective_hook_capture(
                &args,
                "corrected fact: wake is the startup surface".to_string(),
                "fact".to_string(),
                supersede_targets.clone(),
            ),
            &base_url,
        )
        .await
        .expect("promote durable memory");
        let repaired = mark_hook_capture_supersede_targets(
            &base_url,
            &args,
            &supersede_targets,
            promoted.item.id,
        )
        .await
        .expect("supersede stale memory");

        assert_eq!(repaired.len(), 1);
        assert!(diagnostics.candidate_hits.is_empty());
        assert_eq!(promoted.item.supersedes, supersede_targets);
        let captured = state.repaired.lock().expect("lock repaired");
        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0].mode, memd_schema::MemoryRepairMode::Supersede);
        assert_eq!(
            captured[0].status,
            Some(memd_schema::MemoryStatus::Superseded)
        );
        assert!(captured[0].supersedes.is_empty());

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[tokio::test]
    async fn hook_capture_can_find_supersede_targets_by_query() {
        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state.clone(), false).await;
        let args = HookCaptureArgs {
            output: PathBuf::from(".memd"),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            source_path: Some("hook-capture".to_string()),
            confidence: Some(0.8),
            ttl_seconds: None,
            content: None,
            input: None,
            stdin: true,
            tag: vec!["episodic".to_string()],
            promote_kind: Some("fact".to_string()),
            promote_scope: Some("project".to_string()),
            promote_supersede: Vec::new(),
            promote_supersede_query: Some("stale belief".to_string()),
            promote_tag: vec!["correction".to_string()],
            promote_confidence: Some(0.99),
            summary: true,
        };

        let (supersede_targets, diagnostics) =
            find_hook_capture_supersede_targets(&base_url, &args, "corrected fact: stale belief")
                .await
                .expect("supersede by query");
        assert_eq!(supersede_targets.len(), 1);
        assert!(!diagnostics.tried_queries.is_empty());
        assert!(!diagnostics.candidate_hits.is_empty());
        assert_eq!(diagnostics.candidate_hits[0].query, "stale belief");
        assert_eq!(diagnostics.candidate_hits[0].ids.len(), 1);
        assert_eq!(*state.search_count.lock().expect("lock search count"), 1);
        let promoted = remember_with_bundle_defaults(
            &remember_args_from_effective_hook_capture(
                &args,
                "corrected fact: stale belief".to_string(),
                "fact".to_string(),
                supersede_targets.clone(),
            ),
            &base_url,
        )
        .await
        .expect("promote corrected memory");
        assert_eq!(promoted.item.supersedes, supersede_targets);
        let repaired = mark_hook_capture_supersede_targets(
            &base_url,
            &args,
            &supersede_targets,
            promoted.item.id,
        )
        .await
        .expect("mark stale target superseded");
        assert_eq!(repaired.len(), 1);
        let captured = state.repaired.lock().expect("lock repaired");
        assert_eq!(captured.len(), 1);
        assert!(captured[0].supersedes.is_empty());
    }

    #[tokio::test]
    async fn hook_capture_does_not_auto_supersede_active_query_matches() {
        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state.clone(), false).await;
        let args = HookCaptureArgs {
            output: PathBuf::from(".memd"),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            source_path: Some("hook-capture".to_string()),
            confidence: Some(0.8),
            ttl_seconds: None,
            content: None,
            input: None,
            stdin: true,
            tag: vec!["episodic".to_string()],
            promote_kind: Some("fact".to_string()),
            promote_scope: Some("project".to_string()),
            promote_supersede: Vec::new(),
            promote_supersede_query: Some("hive resume state".to_string()),
            promote_tag: vec!["correction".to_string()],
            promote_confidence: Some(0.99),
            summary: true,
        };

        let (supersede_targets, diagnostics) = find_hook_capture_supersede_targets(
            &base_url,
            &args,
            "corrected fact: hive resume state",
        )
        .await
        .expect("query repair");
        assert!(supersede_targets.is_empty());
        assert!(!diagnostics.candidate_hits.is_empty());
        let captured = state.repaired.lock().expect("lock repaired");
        assert!(captured.is_empty());
    }

    #[test]
    fn resume_prompt_surfaces_current_task_snapshot() {
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
                    record: "Follow the active current-task lane".to_string(),
                }],
                evicted: Vec::new(),
                rehydration_queue: vec![memd_schema::MemoryRehydrationRecord {
                    id: None,
                    kind: "source".to_string(),
                    label: "handoff".to_string(),
                    summary: "Reload the shared workspace handoff".to_string(),
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
                        content: "One review item is still open".to_string(),
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
            recent_repo_changes: vec!["status M crates/memd-client/src/render.rs".to_string()],
            change_summary: vec!["focus -> Follow the active current-task lane".to_string()],
            resume_state_age_minutes: None,
            refresh_recommended: false,
        };

        let prompt = crate::render::render_resume_prompt(&snapshot);
        assert!(prompt.contains("## Context Budget"));
        assert!(prompt.contains("tok="));
        assert!(prompt.contains("dup=0"));
        assert!(prompt.contains("p=low"));
        assert!(prompt.contains("ref=false"));
        assert!(prompt.contains("## T"));
        assert!(prompt.contains("- t="));
        assert!(prompt.contains("## E+LT"));
        assert!(prompt.contains("Follow the active current-task lane"));
        assert!(prompt.contains("One review item is still open"));
        assert!(prompt.contains("Reload the shared workspace handoff"));
        assert!(prompt.contains("status M crates/memd-client/src/render.rs"));
        assert!(prompt.contains("team-alpha"));
    }

    #[test]
    fn resume_snapshot_detects_redundant_context_items() {
        let base = ResumeSnapshot {
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
                records: vec![memd_schema::CompactMemoryRecord {
                    id: uuid::Uuid::new_v4(),
                    record: "Repeat this exact idea".to_string(),
                }],
            },
            working: memd_schema::WorkingMemoryResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::CurrentTask,
                retrieval_order: vec![memd_schema::MemoryScope::Project],
                budget_chars: 1600,
                used_chars: 120,
                remaining_chars: 1480,
                truncated: false,
                policy: memd_schema::WorkingMemoryPolicyState {
                    admission_limit: 8,
                    max_chars_per_item: 220,
                    budget_chars: 1600,
                    rehydration_limit: 4,
                },
                records: vec![memd_schema::CompactMemoryRecord {
                    id: uuid::Uuid::new_v4(),
                    record: "Repeat this exact idea".to_string(),
                }],
                evicted: Vec::new(),
                rehydration_queue: vec![memd_schema::MemoryRehydrationRecord {
                    id: None,
                    kind: "source".to_string(),
                    label: "dup".to_string(),
                    summary: "Repeat this exact idea".to_string(),
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
                        content: "Repeat this exact idea".to_string(),
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
                        confidence: 0.9,
                        ttl_seconds: None,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        last_verified_at: None,
                        supersedes: Vec::new(),
                        tags: Vec::new(),
                        status: memd_schema::MemoryStatus::Active,
                        stage: memd_schema::MemoryStage::Candidate,
                    },
                    reasons: vec!["same".to_string()],
                }],
            },
            workspaces: memd_schema::WorkspaceMemoryResponse {
                workspaces: Vec::new(),
            },
            sources: memd_schema::SourceMemoryResponse {
                sources: Vec::new(),
            },
            semantic: None,
            claims: SessionClaimsState::default(),
            recent_repo_changes: vec!["repo clean".to_string()],
            change_summary: Vec::new(),
            resume_state_age_minutes: None,
            refresh_recommended: false,
        };

        assert!(base.redundant_context_items() >= 3);
        assert_eq!(base.context_pressure(), "high");
        assert!(
            base.optimization_hints()
                .iter()
                .any(|hint| hint.contains("collapse 3 repeated context item"))
        );
    }

    #[test]
    fn resume_state_changes_capture_hot_lane_deltas() {
        let previous = BundleResumeState {
            focus: Some("old focus".to_string()),
            pressure: Some("old pressure".to_string()),
            next_recovery: Some("artifact: old".to_string()),
            lane: Some("demo / main / alpha".to_string()),
            working_records: 2,
            inbox_items: 1,
            rehydration_items: 1,
            recorded_at: Utc::now(),
        };
        let current = BundleResumeState {
            focus: Some("new focus".to_string()),
            pressure: Some("new pressure".to_string()),
            next_recovery: Some("artifact: new".to_string()),
            lane: Some("demo / main / beta".to_string()),
            working_records: 4,
            inbox_items: 0,
            rehydration_items: 2,
            recorded_at: Utc::now(),
        };

        let changes = describe_resume_state_changes(Some(&previous), &current);
        assert!(
            changes
                .iter()
                .any(|value| value.contains("focus -> new focus"))
        );
        assert!(
            changes
                .iter()
                .any(|value| value.contains("pressure -> new pressure"))
        );
        assert!(
            changes
                .iter()
                .any(|value| value.contains("next_recovery -> artifact: new"))
        );
        assert!(
            changes
                .iter()
                .any(|value| value.contains("lane -> demo / main / beta"))
        );
        assert!(changes.iter().any(|value| value.contains("working 2 -> 4")));
        assert!(changes.iter().any(|value| value.contains("inbox 1 -> 0")));
        assert!(
            changes
                .iter()
                .any(|value| value.contains("rehydration 1 -> 2"))
        );
    }

    #[test]
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

