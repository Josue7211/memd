    fn high_scoring_eval(output: &Path) -> BundleEvalResponse {
        BundleEvalResponse {
            bundle_root: output.display().to_string(),
            project: Some("demo".to_string()),
            namespace: None,
            agent: Some("codex".to_string()),
            workspace: None,
            visibility: None,
            status: "strong".to_string(),
            score: 92,
            working_records: 2,
            context_records: 2,
            rehydration_items: 1,
            inbox_items: 0,
            workspace_lanes: 0,
            semantic_hits: 0,
            findings: Vec::new(),
            baseline_score: Some(88),
            score_delta: Some(4),
            changes: vec!["score 88 -> 92".to_string()],
            recommendations: vec!["keep the current lane tight".to_string()],
        }
    }

    fn high_scoring_scenario(output: &Path) -> ScenarioReport {
        ScenarioReport {
            bundle_root: output.display().to_string(),
            project: Some("demo".to_string()),
            namespace: None,
            agent: Some("codex".to_string()),
            session: None,
            workspace: None,
            visibility: None,
            scenario: "bundle_health".to_string(),
            score: 96,
            max_score: 100,
            checks: vec![
                ScenarioCheck {
                    name: "runtime_config".to_string(),
                    status: "pass".to_string(),
                    points: 28,
                    details: "bundle runtime config is available".to_string(),
                },
                ScenarioCheck {
                    name: "resume_signal".to_string(),
                    status: "pass".to_string(),
                    points: 22,
                    details: "resume signal pressure is low".to_string(),
                },
            ],
            passed_checks: 2,
            failed_checks: 0,
            findings: Vec::new(),
            next_actions: vec!["keep the current lane tight".to_string()],
            evidence: vec!["scenario baseline is healthy".to_string()],
            generated_at: Utc::now(),
            completed_at: Utc::now(),
        }
    }

    fn test_improvement_report(output: &Path, completed_at: DateTime<Utc>) -> ImprovementReport {
        ImprovementReport {
            bundle_root: output.display().to_string(),
            project: Some("demo".to_string()),
            namespace: None,
            agent: Some("codex".to_string()),
            session: None,
            workspace: None,
            visibility: None,
            max_iterations: 1,
            apply: false,
            started_at: completed_at,
            completed_at,
            converged: true,
            initial_gap: None,
            final_gap: None,
            final_changes: Vec::new(),
            iterations: Vec::new(),
        }
    }

    fn test_composite_report(
        output: &Path,
        score: u8,
        max_score: u8,
        completed_at: DateTime<Utc>,
    ) -> CompositeReport {
        CompositeReport {
            bundle_root: output.display().to_string(),
            project: Some("demo".to_string()),
            namespace: None,
            agent: Some("codex".to_string()),
            session: None,
            workspace: None,
            visibility: None,
            scenario: Some("self_evolution".to_string()),
            score,
            max_score,
            dimensions: Vec::new(),
            gates: Vec::new(),
            findings: Vec::new(),
            recommendations: Vec::new(),
            evidence: Vec::new(),
            generated_at: completed_at,
            completed_at,
        }
    }

    fn test_experiment_report(
        output: &Path,
        accepted: bool,
        restored: bool,
        score: u8,
        max_score: u8,
        completed_at: DateTime<Utc>,
    ) -> ExperimentReport {
        ExperimentReport {
            bundle_root: output.display().to_string(),
            project: Some("demo".to_string()),
            namespace: None,
            agent: Some("codex".to_string()),
            session: None,
            workspace: None,
            visibility: None,
            max_iterations: 1,
            accept_below: 80,
            apply: false,
            consolidate: false,
            accepted,
            restored,
            started_at: completed_at,
            completed_at,
            improvement: test_improvement_report(output, completed_at),
            composite: test_composite_report(output, score, max_score, completed_at),
            trail: Vec::new(),
            learnings: vec!["tighten the loop".to_string()],
            findings: Vec::new(),
            recommendations: Vec::new(),
            evidence: Vec::new(),
            evolution: None,
        }
    }

    fn write_test_bundle_config(output: &Path, base_url: &str) {
        fs::create_dir_all(output).expect("create bundle root");
        fs::write(
            output.join("config.json"),
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
        .expect("write bundle config");
    }

    fn test_benchmark_registry() -> BenchmarkRegistry {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let registry_path = manifest_dir
            .join("../..")
            .join("docs/verification/benchmark-registry.json");
        let registry_json = fs::read_to_string(&registry_path).expect("read benchmark registry");
        serde_json::from_str(&registry_json).expect("parse benchmark registry")
    }

    fn test_continuity_fixture_record() -> FixtureRecord {
        FixtureRecord {
            id: "fixture.continuity_bundle".to_string(),
            kind: "bundle_fixture".to_string(),
            description: "continuity".to_string(),
            seed_files: Vec::new(),
            seed_config: json!({
                "project": "memd",
                "namespace": "main",
                "agent": "codex"
            }),
            seed_memories: Vec::new(),
            seed_events: Vec::new(),
            seed_sessions: Vec::new(),
            seed_claims: Vec::new(),
            seed_vault: None,
            backend_mode: "normal".to_string(),
            isolation: "fresh_temp_dir".to_string(),
            cleanup_policy: "destroy".to_string(),
        }
    }

    fn test_failing_tier_zero_verifier() -> VerifierRecord {
        VerifierRecord {
            id: "verifier.feature.session_continuity.failing".to_string(),
            name: "Resume feature failing".to_string(),
            verifier_type: "feature_contract".to_string(),
            pillar: "memory-continuity".to_string(),
            family: "bundle-runtime".to_string(),
            subject_ids: vec!["feature.session_continuity".to_string()],
            fixture_id: "fixture.continuity_bundle".to_string(),
            baseline_modes: vec!["with_memd".to_string()],
            steps: Vec::new(),
            assertions: Vec::new(),
            metrics: vec!["prompt_tokens".to_string()],
            evidence_requirements: vec!["live_primary".to_string()],
            gate_target: "acceptable".to_string(),
            status: "declared".to_string(),
            lanes: vec!["nightly".to_string()],
            helper_hooks: vec!["force_fail".to_string()],
        }
    }

    fn test_failing_tier_two_verifier() -> VerifierRecord {
        VerifierRecord {
            id: "verifier.feature.noncritical.failing".to_string(),
            name: "Noncritical feature failing".to_string(),
            verifier_type: "feature_contract".to_string(),
            pillar: "visible-memory".to_string(),
            family: "visible-memory".to_string(),
            subject_ids: vec!["feature.noncritical.placeholder".to_string()],
            fixture_id: "fixture.continuity_bundle".to_string(),
            baseline_modes: vec!["with_memd".to_string()],
            steps: Vec::new(),
            assertions: Vec::new(),
            metrics: vec!["prompt_tokens".to_string()],
            evidence_requirements: vec!["live_primary".to_string()],
            gate_target: "acceptable".to_string(),
            status: "declared".to_string(),
            lanes: vec!["nightly".to_string()],
            helper_hooks: vec!["force_fail".to_string()],
        }
    }

    async fn run_verify_sweep_for_test(
        lane: &str,
        verifiers: Vec<VerifierRecord>,
    ) -> anyhow::Result<VerifySweepReport> {
        let dir = std::env::temp_dir().join(format!("memd-verify-sweep-{}", uuid::Uuid::new_v4()));
        let output = dir.join(".memd");
        fs::create_dir_all(&output).expect("create verify sweep output");

        let mut registry = test_benchmark_registry();
        registry.verifiers = verifiers;
        registry.fixtures = vec![test_continuity_fixture_record()];

        let report = execute_verify_sweep(&output, None, &registry, lane).await;
        fs::remove_dir_all(dir).expect("cleanup verify sweep dir");
        report
    }

    fn write_test_benchmark_registry(repo_root: &Path) {
        fs::create_dir_all(benchmark_registry_docs_dir(repo_root))
            .expect("create benchmark registry docs dir");
        fs::write(
            benchmark_registry_json_path(repo_root),
            serde_json::to_string_pretty(&test_benchmark_registry()).expect("serialize registry")
                + "\n",
        )
        .expect("write benchmark registry");
    }

    fn test_feature_benchmark_report(output: &Path) -> FeatureBenchmarkReport {
        FeatureBenchmarkReport {
            bundle_root: output.display().to_string(),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            session: Some("codex-a".to_string()),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            score: 88,
            max_score: 100,
            command_count: 4,
            skill_count: 2,
            pack_count: 2,
            memory_pages: 3,
            event_count: 5,
            areas: vec![FeatureBenchmarkArea {
                slug: "core_memory".to_string(),
                name: "Core Memory".to_string(),
                score: 90,
                max_score: 100,
                status: "pass".to_string(),
                implemented_commands: 4,
                expected_commands: 4,
                evidence: vec!["memory_quality=90".to_string()],
                recommendations: vec!["keep the current lane tight".to_string()],
            }],
            evidence: vec!["benchmark_registry root=repo".to_string()],
            recommendations: vec!["keep the current lane tight".to_string()],
            generated_at: Utc::now(),
            completed_at: Utc::now(),
        }
    }

    fn write_test_bundle_heartbeat(output: &Path, state: &BundleHeartbeatState) {
        fs::create_dir_all(output.join("state")).expect("create bundle state dir");
        fs::write(
            bundle_heartbeat_state_path(output),
            serde_json::to_string_pretty(state).expect("serialize heartbeat") + "\n",
        )
        .expect("write heartbeat");
    }

    fn init_test_git_repo(root: &Path) {
        let _ = Command::new("git")
            .arg("-C")
            .arg(root)
            .arg("init")
            .status()
            .expect("init git repo");
        let _ = Command::new("git")
            .arg("-C")
            .arg(root)
            .arg("config")
            .arg("user.email")
            .arg("memd@example.com")
            .status()
            .expect("git user email");
        let _ = Command::new("git")
            .arg("-C")
            .arg(root)
            .arg("config")
            .arg("user.name")
            .arg("memd")
            .status()
            .expect("git user name");
        let _ = Command::new("git")
            .arg("-C")
            .arg(root)
            .arg("add")
            .arg(".")
            .status()
            .expect("git add");
        let _ = Command::new("git")
            .arg("-C")
            .arg(root)
            .arg("commit")
            .arg("-m")
            .arg("init")
            .status()
            .expect("git commit");
    }

    fn checkout_test_branch(root: &Path, branch: &str) {
        let _ = Command::new("git")
            .arg("-C")
            .arg(root)
            .arg("checkout")
            .arg("-b")
            .arg(branch)
            .status()
            .expect("checkout branch");
    }

    fn test_hive_heartbeat_state(
        session: &str,
        agent: &str,
        tab_id: &str,
        status: &str,
        last_seen: DateTime<Utc>,
    ) -> BundleHeartbeatState {
        BundleHeartbeatState {
            session: Some(session.to_string()),
            agent: Some(agent.to_string()),
            effective_agent: Some(compose_agent_identity(agent, Some(session))),
            tab_id: Some(tab_id.to_string()),
            hive_system: Some(agent.to_string()),
            hive_role: Some("agent".to_string()),
            worker_name: Some(agent.to_string()),
            display_name: None,
            role: Some("agent".to_string()),
            capabilities: vec!["memory".to_string(), "coordination".to_string()],
            hive_groups: vec!["openclaw-stack".to_string()],
            lane_id: Some("/tmp/memd".to_string()),
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: Some(default_heartbeat_model()),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            repo_root: Some("/tmp/memd".to_string()),
            worktree_root: Some("/tmp/memd".to_string()),
            branch: Some("feature/test-bee".to_string()),
            base_branch: Some("main".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: None,
            base_url_healthy: None,
            host: Some("workstation".to_string()),
            pid: Some(4242),
            topic_claim: None,
            scope_claims: Vec::new(),
            task_id: None,
            focus: Some("Keep the shared hive healthy".to_string()),
            pressure: Some("Avoid claim collisions".to_string()),
            next_recovery: None,
            next_action: None,
            needs_help: false,
            needs_review: false,
            handoff_state: None,
            confidence: None,
            risk: None,
            status: status.to_string(),
            last_seen,
            authority_mode: Some("shared".to_string()),
            authority_degraded: false,
        }
    }

    fn test_autoresearch_snapshot(
        refresh_recommended: bool,
        change_summary: Vec<String>,
        recent_repo_changes: Vec<String>,
    ) -> ResumeSnapshot {
        ResumeSnapshot {
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex@codex-a".to_string()),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            context: memd_schema::CompactContextResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::CurrentTask,
                retrieval_order: vec![memd_schema::MemoryScope::Project],
                records: vec![memd_schema::CompactMemoryRecord {
                    id: uuid::Uuid::new_v4(),
                    record: "resume context".to_string(),
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
                    record: "resume focus".to_string(),
                }],
                evicted: Vec::new(),
                rehydration_queue: vec![memd_schema::MemoryRehydrationRecord {
                    id: None,
                    kind: "source".to_string(),
                    label: "handoff".to_string(),
                    summary: "resume next step".to_string(),
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
                        content: "resume pressure".to_string(),
                        redundancy_key: None,
                        belief_branch: None,
                        preferred: true,
                        kind: memd_schema::MemoryKind::Status,
                        scope: memd_schema::MemoryScope::Project,
                        project: Some("demo".to_string()),
                        namespace: Some("main".to_string()),
                        workspace: Some("shared".to_string()),
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
                    workspace: Some("shared".to_string()),
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
            recent_repo_changes,
            change_summary,
            resume_state_age_minutes: Some(1),
            refresh_recommended,
        }
    }

    fn seed_autoresearch_snapshot_cache(output: &Path, snapshot: &ResumeSnapshot) {
        seed_autoresearch_snapshot_cache_with_limits(output, snapshot, 8, 4, true);
    }

    fn seed_autoresearch_snapshot_cache_with_limits(
        output: &Path,
        snapshot: &ResumeSnapshot,
        limit: usize,
        rehydration_limit: usize,
        semantic: bool,
    ) {
        let runtime = read_bundle_runtime_config(output)
            .expect("read runtime")
            .expect("runtime config");
        let args = ResumeArgs {
            output: output.to_path_buf(),
            project: None,
            namespace: None,
            agent: None,
            workspace: None,
            visibility: None,
            route: None,
            intent: None,
            limit: Some(limit),
            rehydration_limit: Some(rehydration_limit),
            semantic,
            prompt: false,
            summary: false,
        };
        let base_url = resolve_bundle_command_base_url(
            runtime
                .base_url
                .as_deref()
                .unwrap_or("http://127.0.0.1:59999"),
            runtime.base_url.as_deref(),
        );
        let cache_key = build_resume_snapshot_cache_key(&args, Some(&runtime), &base_url);
        cache::write_resume_snapshot_cache(output, &cache_key, snapshot)
            .expect("write resume cache");
    }

    #[tokio::test]
    async fn run_capability_contract_loop_warns_when_registry_is_empty() {
        let output =
            std::env::temp_dir().join(format!("memd-cap-empty-output-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&output).expect("create temp output");
        write_test_bundle_config(&output, "http://127.0.0.1:59999");
        let snapshot = test_autoresearch_snapshot(false, Vec::new(), Vec::new());
        seed_autoresearch_snapshot_cache(&output, &snapshot);

        let descriptor = AUTORESEARCH_LOOPS
            .iter()
            .find(|descriptor| descriptor.slug == "autonomy-quality")
            .expect("autonomy quality descriptor");
        let record =
            run_autonomy_quality_loop(&output, "http://127.0.0.1:59999", descriptor, 0, None)
                .await
                .expect("run autonomy quality loop");

        assert_eq!(record.status.as_deref(), Some("warning"));
        assert_eq!(record.percent_improvement, Some(80.0));
        assert_eq!(record.token_savings, Some(descriptor.base_tokens * 0.8));
        assert!(
            record
                .summary
                .as_deref()
                .unwrap_or_default()
                .contains("autonomy quality score")
        );
        assert_eq!(
            record
                .metadata
                .get("warning_reasons")
                .and_then(serde_json::Value::as_array)
                .and_then(|reasons| reasons.first())
                .and_then(serde_json::Value::as_str),
            Some("no_change_signal")
        );

        fs::remove_dir_all(output).expect("cleanup temp output");
    }

    #[tokio::test]
    async fn run_capability_contract_loop_warns_on_trend_regression() {
        let project_root =
            std::env::temp_dir().join(format!("memd-cap-trend-{}", uuid::Uuid::new_v4()));
        let output = project_root.join(".memd");
        fs::create_dir_all(&output).expect("create temp output");
        fs::write(project_root.join("AGENTS.md"), "# agents\n").expect("write agents");
        write_test_bundle_config(&output, "http://127.0.0.1:59999");
        let snapshot = test_autoresearch_snapshot(
            true,
            vec!["summary".to_string()],
            vec!["changed file".to_string()],
        );
        seed_autoresearch_snapshot_cache(&output, &snapshot);

        let descriptor = AUTORESEARCH_LOOPS
            .iter()
            .find(|descriptor| descriptor.slug == "autonomy-quality")
            .expect("autonomy quality descriptor");
        let previous_entry = LoopSummaryEntry {
            slug: "autonomy-quality".to_string(),
            percent_improvement: Some(100.0),
            token_savings: Some(100.0),
            status: Some("success".to_string()),
            recorded_at: Utc::now(),
        };
        let record = run_autonomy_quality_loop(
            &output,
            "http://127.0.0.1:59999",
            descriptor,
            1,
            Some(&previous_entry),
        )
        .await
        .expect("run autonomy quality loop");

        assert_eq!(record.status.as_deref(), Some("warning"));
        assert!(
            record
                .metadata
                .get("trend")
                .and_then(|trend| trend.get("regressed"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
        );
        assert!(
            record
                .metadata
                .get("trend")
                .and_then(|trend| trend.get("warning_reasons"))
                .and_then(serde_json::Value::as_array)
                .is_some_and(|reasons| reasons
                    .iter()
                    .any(|reason| reason == "trend_token_regressed"))
        );

        fs::remove_dir_all(project_root).expect("cleanup temp project");
    }

    #[test]
    fn autoresearch_absolute_floor_requires_enough_evidence() {
        let descriptor = AUTORESEARCH_LOOPS
            .iter()
            .find(|descriptor| descriptor.slug == "prompt-efficiency")
            .expect("prompt efficiency descriptor");

        assert!(loop_meets_absolute_floor(
            descriptor,
            descriptor.base_percent,
            descriptor.base_tokens,
            AUTORESEARCH_MIN_EVIDENCE_SIGNALS,
        ));
        assert!(!loop_meets_absolute_floor(
            descriptor,
            descriptor.base_percent,
            descriptor.base_tokens,
            AUTORESEARCH_MIN_EVIDENCE_SIGNALS - 1,
        ));
        assert!(!loop_meets_absolute_floor(
            descriptor,
            descriptor.base_percent - 0.1,
            descriptor.base_tokens,
            AUTORESEARCH_MIN_EVIDENCE_SIGNALS,
        ));
        assert_eq!(
            loop_floor_warning_reasons(
                descriptor,
                descriptor.base_percent - 0.1,
                descriptor.base_tokens - 1.0,
                AUTORESEARCH_MIN_EVIDENCE_SIGNALS - 1,
            ),
            vec![
                "percent_below_floor".to_string(),
                "token_savings_below_floor".to_string(),
                "evidence_count_below_floor".to_string(),
            ]
        );
    }

    #[test]
    fn autoresearch_trend_reasons_split_percent_and_tokens() {
        let descriptor = AUTORESEARCH_LOOPS
            .iter()
            .find(|descriptor| descriptor.slug == "prompt-efficiency")
            .expect("prompt efficiency descriptor");
        let previous_entry = LoopSummaryEntry {
            slug: "prompt-efficiency".to_string(),
            percent_improvement: Some(10.0),
            token_savings: Some(100.0),
            status: Some("success".to_string()),
            recorded_at: Utc::now(),
        };

        assert_eq!(
            loop_trend_warning_reasons(
                descriptor,
                Some(&previous_entry),
                descriptor.trend_percent_floor - 1.0,
                descriptor.trend_token_floor - 1.0,
            ),
            vec![
                "trend_percent_regressed".to_string(),
                "trend_token_regressed".to_string(),
            ]
        );
    }

    #[test]
    fn autoresearch_loop_table_has_ten_unique_slugs() {
        let mut slugs = AUTORESEARCH_LOOPS
            .iter()
            .map(|descriptor| descriptor.slug)
            .collect::<Vec<_>>();
        slugs.sort_unstable();
        slugs.dedup();

        assert_eq!(slugs.len(), 10);
        assert!(slugs.contains(&"hive-health"));
        assert!(slugs.contains(&"memory-hygiene"));
        assert!(slugs.contains(&"autonomy-quality"));
        assert!(slugs.contains(&"prompt-efficiency"));
        assert!(slugs.contains(&"repair-rate"));
        assert!(slugs.contains(&"signal-freshness"));
        assert!(slugs.contains(&"cross-harness"));
        assert!(slugs.contains(&"self-evolution"));
        assert!(slugs.contains(&"branch-review-quality"));
        assert!(slugs.contains(&"docs-spec-drift"));
        assert!(loop_success_requires_second_signal(true, true, true, true));
    }

    #[test]
    fn autoresearch_sweep_signature_rounds_loop_metrics() {
        let descriptor = AUTORESEARCH_LOOPS
            .iter()
            .find(|descriptor| descriptor.slug == "prompt-efficiency")
            .expect("prompt efficiency descriptor");
        let record = LoopRecord {
            slug: Some("prompt-efficiency".to_string()),
            name: Some("Prompt Efficiency".to_string()),
            iteration: Some(1),
            percent_improvement: Some(31.915),
            token_savings: Some(709.004),
            status: Some("success".to_string()),
            summary: None,
            artifacts: None,
            created_at: Some(Utc::now()),
            metadata: serde_json::json!({}),
        };
        let signature = build_autoresearch_sweep_signature(&[
            (descriptor, record.clone()),
            (descriptor, record),
        ]);

        assert_eq!(signature.len(), 2);
        assert_eq!(signature[0].slug, "prompt-efficiency");
        assert_eq!(signature[0].status, "success");
        assert_eq!(signature[0].percent_bp, 3192);
        assert_eq!(signature[0].tokens_bp, 70900);
    }

    #[test]
    fn autoresearch_emits_ten_loop_records() {
        let output =
            std::env::temp_dir().join(format!("memd-10-loop-records-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&output).expect("create output");

        for (index, descriptor) in AUTORESEARCH_LOOPS.iter().enumerate() {
            let record = build_autoresearch_record(
                descriptor,
                index + 1,
                100.0,
                descriptor.base_tokens,
                format!("{} recorded", descriptor.slug),
                vec![descriptor.slug.to_string()],
                serde_json::json!({
                    "evidence": [format!("slug={}", descriptor.slug)],
                    "warning_reasons": [],
                }),
            );
            persist_loop_record(&output, &record).expect("persist loop record");
        }

        let outputs = read_loop_entries(&output).expect("read loop entries");
        assert_eq!(outputs.len(), 10);
        assert!(outputs.iter().any(|record| record.slug == "hive-health"));
        assert!(
            outputs
                .iter()
                .any(|record| record.slug == "docs-spec-drift")
        );
        assert!(outputs.iter().all(|record| record.record.status.is_some()));

        fs::remove_dir_all(output).expect("cleanup temp output");
    }

    #[tokio::test]
    async fn run_prompt_surface_loop_warns_when_refresh_is_recommended() {
        let output = std::env::temp_dir().join(format!(
            "memd-prompt-surface-refresh-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&output).expect("create temp output");
        write_test_bundle_config(&output, "http://127.0.0.1:59999");
        let snapshot = test_autoresearch_snapshot(
            true,
            vec!["summary".to_string()],
            vec!["changed file".to_string()],
        );
        seed_autoresearch_snapshot_cache(&output, &snapshot);

        let descriptor = AUTORESEARCH_LOOPS
            .iter()
            .find(|descriptor| descriptor.slug == "prompt-efficiency")
            .expect("prompt efficiency descriptor");
        let record =
            run_prompt_efficiency_loop(&output, "http://127.0.0.1:59999", descriptor, 0, None)
                .await
                .expect("run prompt efficiency loop");

        assert_eq!(record.status.as_deref(), Some("success"));
        assert!(
            record
                .metadata
                .get("refresh_recommended")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
        );

        fs::remove_dir_all(output).expect("cleanup temp output");
    }

    #[tokio::test]
    async fn run_live_truth_loop_warns_when_snapshot_has_no_signal() {
        let output =
            std::env::temp_dir().join(format!("memd-live-truth-empty-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&output).expect("create temp output");
        write_test_bundle_config(&output, "http://127.0.0.1:59999");
        let snapshot = test_autoresearch_snapshot(false, Vec::new(), Vec::new());
        seed_autoresearch_snapshot_cache_with_limits(&output, &snapshot, 4, 2, true);

        let descriptor = AUTORESEARCH_LOOPS
            .iter()
            .find(|descriptor| descriptor.slug == "signal-freshness")
            .expect("signal freshness descriptor");
        let record = run_live_truth_loop(&output, "http://127.0.0.1:59999", descriptor, 0, None)
            .await
            .expect("run live truth loop");

        assert_eq!(record.status.as_deref(), Some("warning"));
        assert!(
            record
                .summary
                .as_deref()
                .unwrap_or_default()
                .contains("0 change_summary")
        );

        fs::remove_dir_all(output).expect("cleanup temp output");
    }

    #[tokio::test]
    async fn run_live_truth_loop_succeeds_when_snapshot_has_fresh_signal() {
        let output =
            std::env::temp_dir().join(format!("memd-live-truth-fresh-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&output).expect("create temp output");
        write_test_bundle_config(&output, "http://127.0.0.1:59999");
        let snapshot = test_autoresearch_snapshot(
            true,
            vec![
                "summary".to_string(),
                "new fact".to_string(),
                "follow-up".to_string(),
                "checkpoint".to_string(),
            ],
            vec!["changed file".to_string()],
        );
        seed_autoresearch_snapshot_cache_with_limits(&output, &snapshot, 4, 2, true);

        let descriptor = AUTORESEARCH_LOOPS
            .iter()
            .find(|descriptor| descriptor.slug == "signal-freshness")
            .expect("signal freshness descriptor");
        let record = run_live_truth_loop(&output, "http://127.0.0.1:59999", descriptor, 0, None)
            .await
            .expect("run live truth loop");

        assert_eq!(record.status.as_deref(), Some("success"));
        assert!(
            record
                .metadata
                .get("confidence")
                .and_then(|confidence| confidence.get("absolute_floor_met"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
        );

        fs::remove_dir_all(output).expect("cleanup temp output");
    }

    #[tokio::test]
    async fn run_event_spine_loop_warns_when_snapshot_has_no_signal() {
        let output =
            std::env::temp_dir().join(format!("memd-event-spine-empty-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&output).expect("create temp output");
        write_test_bundle_config(&output, "http://127.0.0.1:59999");
        let snapshot = test_autoresearch_snapshot(false, Vec::new(), Vec::new());
        seed_autoresearch_snapshot_cache_with_limits(&output, &snapshot, 4, 2, true);

        let descriptor = AUTORESEARCH_LOOPS
            .iter()
            .find(|descriptor| descriptor.slug == "memory-hygiene")
            .expect("memory hygiene descriptor");
        let record =
            run_memory_hygiene_loop(&output, "http://127.0.0.1:59999", descriptor, 0, None)
                .await
                .expect("run memory hygiene loop");

        assert_eq!(record.status.as_deref(), Some("warning"));
        assert!(
            record
                .summary
                .as_deref()
                .unwrap_or_default()
                .contains("memory hygiene score")
        );

        fs::remove_dir_all(output).expect("cleanup temp output");
    }

    #[tokio::test]
    async fn execute_autoresearch_loop_dispatches_capability_contract_to_contract_logic() {
        let output = std::env::temp_dir().join(format!(
            "memd-autoresearch-capability-dispatch-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&output).expect("create temp output");
        write_test_bundle_config(&output, "http://127.0.0.1:59999");
        let snapshot = test_autoresearch_snapshot(
            false,
            vec!["summary".to_string()],
            vec!["changed file".to_string()],
        );
        seed_autoresearch_snapshot_cache(&output, &snapshot);

        let descriptor = AUTORESEARCH_LOOPS
            .iter()
            .find(|descriptor| descriptor.slug == "autonomy-quality")
            .expect("autonomy quality descriptor");
        execute_autoresearch_loop(&output, "http://127.0.0.1:59999", descriptor)
            .await
            .expect("execute autonomy quality loop");

        let raw = fs::read_to_string(output.join("loops/loop-autonomy-quality.json"))
            .expect("read autonomy quality record");
        let record: LoopRecord = serde_json::from_str(&raw).expect("parse autonomy quality record");

        assert_eq!(record.slug.as_deref(), Some("autonomy-quality"));
        assert!(
            record
                .summary
                .as_deref()
                .unwrap_or_default()
                .contains("autonomy quality score")
        );
        assert!(
            record
                .metadata
                .get("warning_pressure")
                .and_then(serde_json::Value::as_u64)
                .is_some()
        );

        fs::remove_dir_all(output).expect("cleanup temp output");
    }

    #[tokio::test]
    async fn execute_autoresearch_loop_dispatches_event_spine_to_event_spine_logic() {
        let output = std::env::temp_dir().join(format!(
            "memd-autoresearch-event-spine-dispatch-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&output).expect("create temp output");
        write_test_bundle_config(&output, "http://127.0.0.1:59999");
        let snapshot = test_autoresearch_snapshot(
            false,
            vec!["summary".to_string()],
            vec!["changed file".to_string()],
        );
        seed_autoresearch_snapshot_cache_with_limits(&output, &snapshot, 4, 2, true);

        let descriptor = AUTORESEARCH_LOOPS
            .iter()
            .find(|descriptor| descriptor.slug == "memory-hygiene")
            .expect("memory hygiene descriptor");
        execute_autoresearch_loop(&output, "http://127.0.0.1:59999", descriptor)
            .await
            .expect("execute memory hygiene loop");

        let raw = fs::read_to_string(output.join("loops/loop-memory-hygiene.json"))
            .expect("read memory hygiene record");
        let record: LoopRecord = serde_json::from_str(&raw).expect("parse memory hygiene record");

        assert_eq!(record.slug.as_deref(), Some("memory-hygiene"));
        assert!(
            record
                .summary
                .as_deref()
                .unwrap_or_default()
                .contains("memory hygiene score")
        );
        assert!(
            record
                .metadata
                .get("duplicates")
                .and_then(serde_json::Value::as_f64)
                .is_some()
        );
        assert!(record.metadata.get("event_spine_entries").is_none());

        fs::remove_dir_all(output).expect("cleanup temp output");
    }

    #[tokio::test]
    async fn run_branch_review_quality_loop_warns_on_dirty_branch() {
        let root =
            std::env::temp_dir().join(format!("memd-branch-review-{}", uuid::Uuid::new_v4()));
        let output = root.join(".memd");
        fs::create_dir_all(&output).expect("create temp output");
        write_test_bundle_config(&output, "http://127.0.0.1:59999");

        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("init")
            .status()
            .expect("init git repo");
        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("checkout")
            .arg("-b")
            .arg("feature/branch-review")
            .status()
            .expect("create branch");
        fs::write(root.join("dirty.txt"), "dirty branch").expect("write dirty file");

        let descriptor = AUTORESEARCH_LOOPS
            .iter()
            .find(|descriptor| descriptor.slug == "branch-review-quality")
            .expect("branch review quality descriptor");
        let record = run_branch_review_quality_loop(&output, descriptor, 0, None)
            .await
            .expect("run branch review quality loop");

        assert_eq!(record.status.as_deref(), Some("warning"));
        assert_eq!(record.slug.as_deref(), Some("branch-review-quality"));
        assert_eq!(
            record
                .metadata
                .get("branch")
                .and_then(serde_json::Value::as_str),
            Some("feature/branch-review")
        );
        assert_eq!(
            record
                .metadata
                .get("review_ready")
                .and_then(serde_json::Value::as_bool),
            Some(false)
        );

        fs::remove_dir_all(root).expect("cleanup temp branch review root");
    }

    #[tokio::test]
    async fn run_branch_review_quality_loop_succeeds_on_clean_branch() {
        let root =
            std::env::temp_dir().join(format!("memd-branch-review-clean-{}", uuid::Uuid::new_v4()));
        let output = root.join(".memd");
        fs::create_dir_all(&output).expect("create temp output");
        write_test_bundle_config(&output, "http://127.0.0.1:59999");

        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("init")
            .status()
            .expect("init git repo");
        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("checkout")
            .arg("-b")
            .arg("feature/branch-review-clean")
            .status()
            .expect("create branch");

        let descriptor = AUTORESEARCH_LOOPS
            .iter()
            .find(|descriptor| descriptor.slug == "branch-review-quality")
            .expect("branch review quality descriptor");
        let record = run_branch_review_quality_loop(&output, descriptor, 0, None)
            .await
            .expect("run branch review quality loop");

        assert_eq!(record.status.as_deref(), Some("success"));
        assert_eq!(
            record
                .metadata
                .get("review_ready")
                .and_then(serde_json::Value::as_bool),
            Some(true)
        );

        fs::remove_dir_all(root).expect("cleanup temp branch review root");
    }

    #[tokio::test]
    async fn run_docs_spec_drift_loop_succeeds_when_docs_match_runtime() {
        let root = std::env::temp_dir().join(format!("memd-docs-drift-{}", uuid::Uuid::new_v4()));
        let output = root.join(".memd");
        fs::create_dir_all(output.join("state")).expect("create bundle output");
        fs::create_dir_all(root.join("docs/superpowers/specs")).expect("create specs dir");
        fs::create_dir_all(root.join("docs/superpowers/plans")).expect("create plans dir");
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"temp-docs-drift\"\n",
        )
        .expect("write cargo");
        fs::write(
            root.join("docs/superpowers/specs/2026-04-08-memd-10-loop-design.md"),
            "# memd 10-loop autoresearch stack\n\n10-loop\n",
        )
        .expect("write spec");
        fs::write(
            root.join("docs/superpowers/plans/2026-04-08-memd-10-loop-stack.md"),
            "# memd 10-loop autoresearch stack implementation plan\n\nImplementation Plan\n",
        )
        .expect("write plan");
        let _ = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("init")
            .status()
            .expect("init git repo");

        let descriptor = AUTORESEARCH_LOOPS
            .iter()
            .find(|descriptor| descriptor.slug == "docs-spec-drift")
            .expect("docs spec drift descriptor");
        let record = run_docs_spec_drift_loop(&output, descriptor, 0, None)
            .await
            .expect("run docs spec drift loop");

        assert_eq!(record.status.as_deref(), Some("success"));
        assert_eq!(record.slug.as_deref(), Some("docs-spec-drift"));
        assert_eq!(
            record
                .metadata
                .get("spec_has_10_loop")
                .and_then(serde_json::Value::as_bool),
            Some(true)
        );
        assert!(
            record
                .metadata
                .get("evidence")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|evidence| evidence.iter().any(|entry| {
                    entry
                        .as_str()
                        .is_some_and(|value| value.starts_with("runtime_bytes="))
                }))
        );

        fs::remove_dir_all(root).expect("cleanup docs drift root");
    }

    #[tokio::test]
    async fn run_repair_rate_loop_succeeds_on_repair_signal() {
        let output =
            std::env::temp_dir().join(format!("memd-repair-rate-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&output).expect("create temp output");
        write_test_bundle_config(&output, "http://127.0.0.1:59999");
        let snapshot = test_autoresearch_snapshot(
            false,
            vec![
                "fix typo".to_string(),
                "ship feature".to_string(),
                "review notes".to_string(),
                "correct labels".to_string(),
                "cleanup docs".to_string(),
                "repair cache".to_string(),
                "deploy patch".to_string(),
                "verify change".to_string(),
                "close ticket".to_string(),
            ],
            vec!["changed file".to_string()],
        );
        seed_autoresearch_snapshot_cache(&output, &snapshot);

        let descriptor = AUTORESEARCH_LOOPS
            .iter()
            .find(|descriptor| descriptor.slug == "repair-rate")
            .expect("repair rate descriptor");
        let record = run_repair_rate_loop(&output, "http://127.0.0.1:59999", descriptor, 0, None)
            .await
            .expect("run repair rate loop");

        assert_eq!(record.status.as_deref(), Some("success"));
        assert_eq!(record.slug.as_deref(), Some("repair-rate"));
        assert!(
            record
                .metadata
                .get("corrections")
                .and_then(serde_json::Value::as_f64)
                .is_some_and(|value| value >= 3.0)
        );

        fs::remove_dir_all(output).expect("cleanup temp output");
    }

    #[tokio::test]
    async fn run_long_context_loop_warns_when_refresh_is_recommended() {
        let output = std::env::temp_dir().join(format!(
            "memd-long-context-refresh-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&output).expect("create temp output");
        write_test_bundle_config(&output, "http://127.0.0.1:59999");
        let snapshot = test_autoresearch_snapshot(
            true,
            vec!["summary".to_string()],
            vec!["changed file".to_string()],
        );
        seed_autoresearch_snapshot_cache(&output, &snapshot);

        let descriptor = AUTORESEARCH_LOOPS
            .iter()
            .find(|descriptor| descriptor.slug == "prompt-efficiency")
            .expect("prompt efficiency descriptor");
        let record =
            run_prompt_efficiency_loop(&output, "http://127.0.0.1:59999", descriptor, 0, None)
                .await
                .expect("run prompt efficiency loop");

        assert_eq!(record.status.as_deref(), Some("success"));
        assert!(
            record
                .metadata
                .get("refresh_recommended")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
        );

        fs::remove_dir_all(output).expect("cleanup temp output");
    }

    #[tokio::test]
    async fn run_prompt_surface_loop_warns_on_trend_regression() {
        let output = std::env::temp_dir().join(format!(
            "memd-prompt-surface-trend-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&output).expect("create temp output");
        write_test_bundle_config(&output, "http://127.0.0.1:59999");
        let snapshot = test_autoresearch_snapshot(false, vec!["summary".to_string()], Vec::new());
        seed_autoresearch_snapshot_cache(&output, &snapshot);
        let previous_entry = LoopSummaryEntry {
            slug: "prompt-efficiency".to_string(),
            percent_improvement: Some(99.0),
            token_savings: Some(1300.0),
            status: Some("success".to_string()),
            recorded_at: Utc::now(),
        };

        let descriptor = AUTORESEARCH_LOOPS
            .iter()
            .find(|descriptor| descriptor.slug == "prompt-efficiency")
            .expect("prompt efficiency descriptor");
        let record = run_prompt_efficiency_loop(
            &output,
            "http://127.0.0.1:59999",
            descriptor,
            1,
            Some(&previous_entry),
        )
        .await
        .expect("run prompt efficiency loop");

        assert_eq!(record.status.as_deref(), Some("warning"));
        assert!(
            record
                .metadata
                .get("trend")
                .and_then(|trend| trend.get("regressed"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
        );

        fs::remove_dir_all(output).expect("cleanup temp output");
    }

    #[tokio::test]
    async fn run_self_evolution_loop_warns_when_report_is_not_usable() {
        let output =
            std::env::temp_dir().join(format!("memd-self-evolution-{}", uuid::Uuid::new_v4()));
        let experiments = output.join("experiments");
        fs::create_dir_all(&experiments).expect("create experiments dir");
        let report = test_experiment_report(&output, false, true, 0, 10, Utc::now());
        fs::write(
            experiments.join("latest.json"),
            serde_json::to_string_pretty(&report).expect("serialize report"),
        )
        .expect("write latest report");

        let descriptor = AUTORESEARCH_LOOPS
            .iter()
            .find(|descriptor| descriptor.slug == "self-evolution")
            .expect("self-evolution descriptor");
        let record = run_self_evolution_loop(&output, descriptor, 0, None)
            .await
            .expect("run self evolution loop");

        assert_eq!(record.status.as_deref(), Some("warning"));
        assert!(
            record
                .summary
                .as_deref()
                .unwrap_or_default()
                .contains("not usable")
        );
        assert_eq!(record.percent_improvement, Some(0.0));
        assert_eq!(record.token_savings, Some(0.0));
        assert!(
            record
                .metadata
                .get("warning_reasons")
                .and_then(serde_json::Value::as_array)
                .is_some_and(
                    |reasons| reasons.iter().any(|reason| reason == "restored_report")
                        && reasons.iter().any(|reason| reason == "unaccepted_report")
                )
        );

        fs::remove_dir_all(output).expect("cleanup temp output");
    }

    #[tokio::test]
    async fn run_self_evolution_loop_surfaces_accepted_proposal_state() {
        let output = std::env::temp_dir().join(format!(
            "memd-self-evolution-proposal-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&output).expect("create output dir");

        let mut report = test_experiment_report(&output, true, false, 100, 100, Utc::now());
        report.improvement.final_changes = vec![
            "tighten evaluation score heuristic".to_string(),
            "lower loop threshold floor".to_string(),
        ];
        write_experiment_artifacts(&output, &report).expect("write experiment artifacts");

        let descriptor = AUTORESEARCH_LOOPS
            .iter()
            .find(|descriptor| descriptor.slug == "self-evolution")
            .expect("self-evolution descriptor");
        let record = run_self_evolution_loop(&output, descriptor, 0, None)
            .await
            .expect("run self evolution loop");

        assert_eq!(record.status.as_deref(), Some("success"));
        assert_eq!(
            record
                .metadata
                .get("proposal_state")
                .and_then(serde_json::Value::as_str),
            Some("accepted_proposal")
        );
        assert_eq!(
            record
                .metadata
                .get("scope_gate")
                .and_then(serde_json::Value::as_str),
            Some("auto_merge")
        );

        fs::remove_dir_all(output).expect("cleanup temp output");
    }

    #[tokio::test]
    async fn run_self_evolution_loop_refreshes_stale_proposal_classification() {
        let root = std::env::temp_dir().join(format!(
            "memd-self-evolution-refresh-{}",
            uuid::Uuid::new_v4()
        ));
        let output = root.join(".memd");
        fs::create_dir_all(output.join("experiments")).expect("create experiments dir");
        write_test_bundle_config(&output, "http://127.0.0.1:59999");

        let completed_at = Utc::now();
        let mut report = test_experiment_report(&output, true, false, 100, 100, completed_at);
        report.composite.scenario = Some("self_evolution".to_string());
        report.improvement.final_changes =
            vec!["retune pass/fail gate for self-evolution proposals".to_string()];
        fs::write(
            output.join("experiments").join("latest.json"),
            serde_json::to_string_pretty(&report).expect("serialize report"),
        )
        .expect("write latest report");

        let stale = EvolutionProposalReport {
            bundle_root: output.display().to_string(),
            project: Some("demo".to_string()),
            namespace: None,
            agent: Some("codex".to_string()),
            session: None,
            workspace: None,
            visibility: None,
            proposal_id: format!("self-evolution-{}", completed_at.format("%Y%m%dT%H%M%SZ")),
            scenario: Some("self_evolution".to_string()),
            topic: "self-evolution".to_string(),
            branch: format!(
                "auto/evolution/broader-implementation/self-evolution/{}",
                completed_at.format("%Y%m%d%H%M%S")
            ),
            state: "accepted_proposal".to_string(),
            scope_class: "broader_implementation".to_string(),
            scope_gate: "proposal_only".to_string(),
            authority_tier: "proposal_only".to_string(),
            allowed_write_surface: vec!["proposal-only".to_string()],
            merge_eligible: false,
            durable_truth: false,
            accepted: true,
            restored: false,
            composite_score: 100,
            composite_max: 100,
            evidence: vec!["accepted=true".to_string()],
            scope_reasons: vec!["scope unclear; keep on proposal branch".to_string()],
            generated_at: completed_at,
            durability_due_at: None,
        };
        write_evolution_proposal_artifacts(&output, &stale).expect("write stale proposal");

        let descriptor = AUTORESEARCH_LOOPS
            .iter()
            .find(|descriptor| descriptor.slug == "self-evolution")
            .expect("self-evolution descriptor");
        let _record = run_self_evolution_loop(&output, descriptor, 0, None)
            .await
            .expect("run self evolution loop");

        let refreshed = read_latest_evolution_proposal(&output)
            .expect("read refreshed proposal")
            .expect("refreshed proposal present");
        assert_eq!(refreshed.scope_class, "runtime_policy");
        assert_eq!(refreshed.scope_gate, "auto_merge");
        assert_eq!(refreshed.authority_tier, "phase1_auto_merge");

        fs::remove_dir_all(root).expect("cleanup temp root");
    }

    #[tokio::test]
    async fn run_hive_health_loop_warns_on_claim_collision() {
        let root = std::env::temp_dir().join(format!("memd-hive-health-{}", uuid::Uuid::new_v4()));
        let current_project = root.join("current");
        let sibling_project = root.join("sibling");
        let current_bundle = current_project.join(".memd");
        let sibling_bundle = sibling_project.join(".memd");
        fs::create_dir_all(&current_bundle).expect("create current bundle");
        fs::create_dir_all(&sibling_bundle).expect("create sibling bundle");

        fs::write(
            current_bundle.join("config.json"),
            r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "shared-session",
  "tab_id": "tab-a",
  "workspace": "shared",
  "visibility": "workspace",
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
  "agent": "claude-code",
  "session": "shared-session",
  "tab_id": "tab-a",
  "workspace": "shared",
  "visibility": "workspace",
  "route": "auto",
  "intent": "current_task"
}
"#,
        )
        .expect("write sibling config");
        write_test_bundle_heartbeat(
            &current_bundle,
            &test_hive_heartbeat_state("shared-session", "codex", "tab-a", "live", Utc::now()),
        );
        write_test_bundle_heartbeat(
            &sibling_bundle,
            &test_hive_heartbeat_state(
                "shared-session",
                "claude-code",
                "tab-a",
                "dead",
                Utc::now() - chrono::TimeDelta::minutes(30),
            ),
        );

        let descriptor = AUTORESEARCH_LOOPS
            .iter()
            .find(|descriptor| descriptor.slug == "hive-health")
            .expect("hive health descriptor");
        let record = run_hive_health_loop(&current_bundle, descriptor, 0, None)
            .await
            .expect("run hive health loop");

        assert_eq!(record.status.as_deref(), Some("warning"));
        assert_eq!(
            record
                .metadata
                .get("heartbeat_status")
                .and_then(serde_json::Value::as_str),
            Some("live")
        );
        assert!(
            record
                .metadata
                .get("evidence")
                .and_then(serde_json::Value::as_array)
                .is_some_and(
                    |evidence| evidence.iter().any(|entry| entry == "active_hives=2")
                        && evidence.iter().any(|entry| entry == "dead_hives=1")
                        && evidence.iter().any(|entry| entry == "claim_collisions=1")
                )
        );

        fs::remove_dir_all(root).expect("cleanup hive health root");
    }

    #[tokio::test]
    async fn run_hive_health_loop_warns_when_awareness_is_sparse() {
        let output =
            std::env::temp_dir().join(format!("memd-hive-health-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&output).expect("create temp output");
        write_test_bundle_config(&output, "http://127.0.0.1:59999");

        let descriptor = AUTORESEARCH_LOOPS
            .iter()
            .find(|descriptor| descriptor.slug == "hive-health")
            .expect("hive health descriptor");
        let record = run_hive_health_loop(&output, descriptor, 0, None)
            .await
            .expect("run hive health loop");

        assert_eq!(record.status.as_deref(), Some("warning"));
        assert_eq!(
            record
                .metadata
                .get("heartbeat_status")
                .and_then(serde_json::Value::as_str),
            Some("live")
        );

        fs::remove_dir_all(output).expect("cleanup temp output");
    }

    #[path = "evaluation_runtime_tests_evolution.rs"]
