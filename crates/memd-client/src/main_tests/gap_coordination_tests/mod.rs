use super::*;

#[test]
fn describes_eval_changes_against_baseline() {
    let baseline = BundleEvalResponse {
        bundle_root: ".memd".to_string(),
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        agent: Some("codex".to_string()),
        workspace: Some("team-alpha".to_string()),
        visibility: Some("workspace".to_string()),
        status: "usable".to_string(),
        score: 72,
        working_records: 2,
        context_records: 1,
        rehydration_items: 1,
        inbox_items: 3,
        workspace_lanes: 1,
        semantic_hits: 0,
        findings: Vec::new(),
        baseline_score: None,
        score_delta: None,
        changes: Vec::new(),
        recommendations: Vec::new(),
    };
    let snapshot = ResumeSnapshot {
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        agent: Some("codex".to_string()),
        workspace: Some("team-alpha".to_string()),
        visibility: Some("workspace".to_string()),
        route: "auto".to_string(),
        intent: "general".to_string(),
        context: memd_schema::CompactContextResponse {
            route: memd_schema::RetrievalRoute::Auto,
            intent: memd_schema::RetrievalIntent::General,
            retrieval_order: vec![memd_schema::MemoryScope::Project],
            records: vec![memd_schema::CompactMemoryRecord {
                id: uuid::Uuid::new_v4(),
                record: "ctx".to_string(),
            }],
        },
        working: memd_schema::WorkingMemoryResponse {
            route: memd_schema::RetrievalRoute::Auto,
            intent: memd_schema::RetrievalIntent::General,
            retrieval_order: vec![memd_schema::MemoryScope::Project],
            budget_chars: 1600,
            used_chars: 100,
            remaining_chars: 1500,
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
                    record: "one".to_string(),
                },
                memd_schema::CompactMemoryRecord {
                    id: uuid::Uuid::new_v4(),
                    record: "two".to_string(),
                },
                memd_schema::CompactMemoryRecord {
                    id: uuid::Uuid::new_v4(),
                    record: "three".to_string(),
                },
            ],
            evicted: Vec::new(),
            rehydration_queue: vec![
                memd_schema::MemoryRehydrationRecord {
                    id: None,
                    kind: "source".to_string(),
                    label: "artifact".to_string(),
                    summary: "more".to_string(),
                    reason: None,
                    source_agent: None,
                    source_system: None,
                    source_path: None,
                    source_quality: None,
                    recorded_at: None,
                },
                memd_schema::MemoryRehydrationRecord {
                    id: None,
                    kind: "source".to_string(),
                    label: "artifact-2".to_string(),
                    summary: "more".to_string(),
                    reason: None,
                    source_agent: None,
                    source_system: None,
                    source_path: None,
                    source_quality: None,
                    recorded_at: None,
                },
            ],
            traces: Vec::new(),
            semantic_consolidation: None,
            procedures: vec![],
        },
        inbox: memd_schema::MemoryInboxResponse {
            route: memd_schema::RetrievalRoute::Auto,
            intent: memd_schema::RetrievalIntent::General,
            items: vec![],
        },
        workspaces: memd_schema::WorkspaceMemoryResponse {
            workspaces: vec![
                memd_schema::WorkspaceMemoryRecord {
                    project: Some("demo".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("team-alpha".to_string()),
                    visibility: memd_schema::MemoryVisibility::Workspace,
                    item_count: 3,
                    active_count: 3,
                    candidate_count: 0,
                    contested_count: 0,
                    source_lane_count: 1,
                    avg_confidence: 0.9,
                    trust_score: 0.9,
                    last_seen_at: None,
                    tags: vec![],
                },
                memd_schema::WorkspaceMemoryRecord {
                    project: Some("demo".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("shared".to_string()),
                    visibility: memd_schema::MemoryVisibility::Workspace,
                    item_count: 2,
                    active_count: 2,
                    candidate_count: 0,
                    contested_count: 0,
                    source_lane_count: 1,
                    avg_confidence: 0.8,
                    trust_score: 0.8,
                    last_seen_at: None,
                    tags: vec![],
                },
            ],
        },
        sources: memd_schema::SourceMemoryResponse {
            sources: Vec::new(),
        },
        semantic: Some(memd_rag::RagRetrieveResponse {
            status: "ok".to_string(),
            mode: memd_rag::RagRetrieveMode::Auto,
            items: vec![memd_rag::RagRetrieveItem {
                content: "semantic".to_string(),
                source: Some("wiki/demo.md".to_string()),
                score: 0.9,
            }],
        }),
        claims: SessionClaimsState::default(),
        recent_repo_changes: vec!["repo clean".to_string()],
        change_summary: Vec::new(),
        resume_state_age_minutes: None,
        refresh_recommended: false,
    };

    let changes = describe_eval_changes(&baseline, 88, &snapshot);
    assert!(
        changes
            .iter()
            .any(|value: &String| value.contains("score 72 -> 88"))
    );
    assert!(
        changes
            .iter()
            .any(|value: &String| value.contains("working 2 -> 3"))
    );
    assert!(
        changes
            .iter()
            .any(|value: &String| value.contains("rehydration 1 -> 2"))
    );
    assert!(
        changes
            .iter()
            .any(|value: &String| value.contains("inbox 3 -> 0"))
    );
    assert!(
        changes
            .iter()
            .any(|value: &String| value.contains("lanes 1 -> 2"))
    );
    assert!(
        changes
            .iter()
            .any(|value: &String| value.contains("semantic 0 -> 1"))
    );
}

#[test]
fn eval_failure_reason_respects_score_threshold() {
    let response = BundleEvalResponse {
        bundle_root: ".memd".to_string(),
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        agent: Some("codex".to_string()),
        workspace: Some("team-alpha".to_string()),
        visibility: Some("workspace".to_string()),
        status: "weak".to_string(),
        score: 62,
        working_records: 0,
        context_records: 0,
        rehydration_items: 0,
        inbox_items: 0,
        workspace_lanes: 0,
        semantic_hits: 0,
        findings: vec!["no working memory".to_string()],
        baseline_score: Some(70),
        score_delta: Some(-8),
        changes: vec!["score 70 -> 62".to_string()],
        recommendations: vec!["capture durable memory".to_string()],
    };

    let reason = eval_failure_reason(&response, Some(70), false).expect("threshold failure");
    assert!(reason.contains("score 62"));
    assert!(reason.contains("threshold 70"));
}

#[test]
fn eval_failure_reason_respects_regression_gate() {
    let response = BundleEvalResponse {
        bundle_root: ".memd".to_string(),
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        agent: Some("codex".to_string()),
        workspace: Some("team-alpha".to_string()),
        visibility: Some("workspace".to_string()),
        status: "usable".to_string(),
        score: 79,
        working_records: 3,
        context_records: 2,
        rehydration_items: 2,
        inbox_items: 1,
        workspace_lanes: 1,
        semantic_hits: 2,
        findings: Vec::new(),
        baseline_score: Some(83),
        score_delta: Some(-4),
        changes: vec!["score 83 -> 79".to_string()],
        recommendations: vec!["write a fresh baseline".to_string()],
    };

    let reason = eval_failure_reason(&response, None, true).expect("regression failure");
    assert!(reason.contains("baseline 83"));
    assert!(reason.contains("to 79"));
    assert!(reason.contains("delta -4"));
}

#[test]
fn eval_failure_reason_passes_when_gates_are_clear() {
    let response = BundleEvalResponse {
        bundle_root: ".memd".to_string(),
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        agent: Some("codex".to_string()),
        workspace: Some("team-alpha".to_string()),
        visibility: Some("workspace".to_string()),
        status: "strong".to_string(),
        score: 91,
        working_records: 4,
        context_records: 3,
        rehydration_items: 2,
        inbox_items: 0,
        workspace_lanes: 2,
        semantic_hits: 3,
        findings: Vec::new(),
        baseline_score: Some(89),
        score_delta: Some(2),
        changes: vec!["score 89 -> 91".to_string()],
        recommendations: Vec::new(),
    };

    assert!(eval_failure_reason(&response, Some(80), true).is_none());
}

#[test]
fn build_eval_recommendations_surfaces_actionable_followups() {
    let snapshot = ResumeSnapshot {
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        agent: Some("codex".to_string()),
        workspace: Some("team-alpha".to_string()),
        visibility: Some("workspace".to_string()),
        route: "auto".to_string(),
        intent: "general".to_string(),
        context: memd_schema::CompactContextResponse {
            route: memd_schema::RetrievalRoute::Auto,
            intent: memd_schema::RetrievalIntent::General,
            retrieval_order: vec![memd_schema::MemoryScope::Project],
            records: Vec::new(),
        },
        working: memd_schema::WorkingMemoryResponse {
            route: memd_schema::RetrievalRoute::Auto,
            intent: memd_schema::RetrievalIntent::General,
            retrieval_order: vec![memd_schema::MemoryScope::Project],
            budget_chars: 1600,
            used_chars: 0,
            remaining_chars: 1600,
            truncated: false,
            policy: memd_schema::WorkingMemoryPolicyState {
                admission_limit: 8,
                max_chars_per_item: 220,
                budget_chars: 1600,
                rehydration_limit: 4,
            },
            records: Vec::new(),
            evicted: Vec::new(),
            rehydration_queue: Vec::new(),
            traces: Vec::new(),
            semantic_consolidation: None,
            procedures: vec![],
        },
        inbox: memd_schema::MemoryInboxResponse {
            route: memd_schema::RetrievalRoute::Auto,
            intent: memd_schema::RetrievalIntent::General,
            items: vec![
                memd_schema::InboxMemoryItem {
                    item: memd_schema::MemoryItem {
                        id: uuid::Uuid::new_v4(),
                        content: "one".to_string(),
                        redundancy_key: None,
                        belief_branch: None,
                        preferred: true,
                        kind: memd_schema::MemoryKind::Decision,
                        scope: memd_schema::MemoryScope::Project,
                        project: Some("demo".to_string()),
                        namespace: Some("main".to_string()),
                        workspace: Some("team-alpha".to_string()),
                        visibility: memd_schema::MemoryVisibility::Workspace,
                        source_agent: None,
                        source_system: None,
                        source_path: None,
                        source_quality: None,
                        confidence: 0.6,
                        ttl_seconds: None,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        last_verified_at: None,
                        supersedes: Vec::new(),
                        tags: Vec::new(),
                        status: memd_schema::MemoryStatus::Active,
                        stage: memd_schema::MemoryStage::Candidate,
                    },
                    reasons: Vec::new(),
                };
                6
            ],
        },
        workspaces: memd_schema::WorkspaceMemoryResponse {
            workspaces: Vec::new(),
        },
        sources: memd_schema::SourceMemoryResponse {
            sources: Vec::new(),
        },
        semantic: Some(memd_rag::RagRetrieveResponse {
            status: "ok".to_string(),
            mode: memd_rag::RagRetrieveMode::Auto,
            items: Vec::new(),
        }),
        claims: SessionClaimsState::default(),
        recent_repo_changes: vec!["repo clean".to_string()],
        change_summary: Vec::new(),
        resume_state_age_minutes: None,
        refresh_recommended: false,
    };

    let recommendations = build_eval_recommendations(&snapshot, 62);
    assert!(
        recommendations
            .iter()
            .any(|value: &String| value.contains("memd remember"))
    );
    assert!(
        recommendations
            .iter()
            .any(|value: &String| value.contains("compact context"))
    );
    assert!(
        recommendations
            .iter()
            .any(|value: &String| value.contains("rehydrate deeper context"))
    );
    assert!(
        recommendations
            .iter()
            .any(|value: &String| value.contains("workspace or visibility"))
    );
    assert!(
        recommendations
            .iter()
            .any(|value: &String| value.contains("inbox pressure"))
    );
    assert!(
        recommendations
            .iter()
            .any(|value| value.contains("LightRAG"))
    );
    assert!(
        recommendations
            .iter()
            .any(|value| value.contains("write a fresh baseline"))
    );
}

#[tokio::test]
async fn run_maintain_command_persists_scan_report() {
    let dir = std::env::temp_dir().join(format!("memd-maintain-scan-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("create temp bundle");
    fs::write(
            dir.join("config.json"),
            r#"{"project":"demo","namespace":"main","agent":"codex","session":"session-a","auto_short_term_capture":false}"#,
        )
        .expect("write config");

    let base_url = spawn_mock_memory_server().await;
    let report = run_maintain_command(
        &MaintainArgs {
            output: dir.clone(),
            mode: "scan".to_string(),
            apply: false,
            summary: false,
        },
        &base_url,
    )
    .await
    .expect("run maintain scan");

    assert_eq!(report.mode.as_str(), "scan");
    assert!(report.receipt_id.is_some());
    assert!(report.findings.iter().any(|value| value.contains("memory")));
    assert!(dir.join("maintenance").join("latest.json").exists());
    assert!(dir.join("maintenance").join("latest.md").exists());

    fs::remove_dir_all(dir).expect("cleanup temp bundle");
}

#[test]
fn render_maintain_summary_surfaces_receipt_and_counts() {
    let summary = render_maintain_summary(&MaintainReport {
        mode: "compact".to_string(),
        receipt_id: Some("receipt-1".to_string()),
        compacted_items: 3,
        refreshed_items: 0,
        repaired_items: 1,
        findings: vec!["compacted stale duplicates".to_string()],
        generated_at: Utc::now(),
    });
    assert!(summary.contains("maintain mode=compact"));
    assert!(summary.contains("receipt=receipt-1"));
    assert!(summary.contains("compacted=3"));
    assert!(summary.contains("repaired=1"));
}

#[test]
fn suggest_coordination_actions_emits_multi_priority_output() {
    let now = Utc::now();
    let inbox = HiveCoordinationInboxResponse {
        messages: vec![
            HiveMessageRecord {
                id: "m-1".to_string(),
                kind: "status_check".to_string(),
                from_session: "hive-a".to_string(),
                from_agent: None,
                to_session: "codex".to_string(),
                project: None,
                namespace: None,
                workspace: None,
                content: "review this artifact".to_string(),
                created_at: now,
                acknowledged_at: None,
            },
            HiveMessageRecord {
                id: "m-2".to_string(),
                kind: "help_request".to_string(),
                from_session: "hive-b".to_string(),
                from_agent: None,
                to_session: "codex".to_string(),
                project: None,
                namespace: None,
                workspace: None,
                content: "another request".to_string(),
                created_at: now,
                acknowledged_at: None,
            },
        ],
        owned_tasks: vec![],
        help_tasks: vec![],
        review_tasks: vec![],
    };

    let stale_sessions = vec!["hive-stale"];
    let active_hives = vec![ProjectAwarenessEntry {
        project_dir: "remote".to_string(),
        bundle_root: "remote:http://127.0.0.1:8787:hive-helper".to_string(),
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        repo_root: None,
        worktree_root: None,
        branch: None,
        base_branch: None,
        agent: Some("agent-shell".to_string()),
        session: Some("hive-helper".to_string()),
        tab_id: Some("tab-helper".to_string()),
        effective_agent: Some("agent-shell@hive-helper".to_string()),
        hive_system: Some("agent-shell".to_string()),
        hive_role: Some("runtime-shell".to_string()),
        capabilities: vec!["shell".to_string(), "exec".to_string()],
        hive_groups: vec!["runtime-core".to_string(), "dependency-owners".to_string()],
        hive_group_goal: Some("stabilize runtime execution".to_string()),
        authority: Some("worker".to_string()),
        base_url: Some("http://127.0.0.1:8787".to_string()),
        presence: "active".to_string(),
        host: Some("vm-a".to_string()),
        pid: Some(42),
        active_claims: 0,
        workspace: Some("shared".to_string()),
        visibility: Some("workspace".to_string()),
        topic_claim: None,
        scope_claims: Vec::new(),
        task_id: None,
        focus: Some("repair runtime dependencies".to_string()),
        pressure: None,
        next_recovery: None,
        last_updated: Some(now),
    }];
    let claims = vec![
        SessionClaim {
            scope: "shared/src.rs".to_string(),
            session: Some("hive-stale".to_string()),
            tab_id: None,
            agent: Some("claude".to_string()),
            effective_agent: Some("codex".to_string()),
            project: None,
            workspace: None,
            host: None,
            pid: None,
            acquired_at: now,
            expires_at: now,
        },
        SessionClaim {
            scope: "shared/src.rs".to_string(),
            session: Some("hive-contender".to_string()),
            tab_id: None,
            agent: None,
            effective_agent: None,
            project: None,
            workspace: None,
            host: None,
            pid: None,
            acquired_at: now,
            expires_at: now,
        },
    ];
    let tasks = vec![
        HiveTaskRecord {
            task_id: "task-exclusive".to_string(),
            title: "edit shared".to_string(),
            description: None,
            status: "assigned".to_string(),
            coordination_mode: "exclusive_write".to_string(),
            session: Some("hive-owner".to_string()),
            agent: Some("hive-owner".to_string()),
            effective_agent: None,
            project: None,
            namespace: None,
            workspace: None,
            claim_scopes: vec!["shared/src.rs".to_string()],
            help_requested: false,
            review_requested: false,
            created_at: now,
            updated_at: now,
        },
        HiveTaskRecord {
            task_id: "task-review".to_string(),
            title: "run review".to_string(),
            description: None,
            status: "in_progress".to_string(),
            coordination_mode: "shared_review".to_string(),
            session: Some("codex".to_string()),
            agent: Some("coder".to_string()),
            effective_agent: None,
            project: None,
            namespace: None,
            workspace: None,
            claim_scopes: vec![],
            help_requested: false,
            review_requested: false,
            created_at: now,
            updated_at: now,
        },
        HiveTaskRecord {
            task_id: "task-help".to_string(),
            title: "parallel assist".to_string(),
            description: None,
            status: "in_progress".to_string(),
            coordination_mode: "help_only".to_string(),
            session: Some("codex".to_string()),
            agent: Some("coder".to_string()),
            effective_agent: None,
            project: None,
            namespace: None,
            workspace: None,
            claim_scopes: vec![],
            help_requested: false,
            review_requested: false,
            created_at: now,
            updated_at: now,
        },
    ];
    let policy_conflicts = vec!["runtime dependency conflict for shared scope".to_string()];

    let suggestions = suggest_coordination_actions(
        &inbox,
        &stale_sessions,
        &active_hives,
        &claims,
        &tasks,
        "codex",
        &policy_conflicts,
        None,
        &[],
    );

    assert_eq!(
        suggestions
            .iter()
            .filter(|s| s.action == "ack_message")
            .count(),
        2,
        "each inbox message should produce its own ack suggestion"
    );
    assert!(suggestions.iter().any(|s| s.action == "recover_session"));
    assert!(suggestions.iter().any(|s| s.action == "assign_scope"));
    assert!(suggestions.iter().any(|s| s.action == "request_review"));
    assert!(suggestions.iter().any(|s| s.action == "request_help"));
    assert!(
        suggestions
            .iter()
            .filter(|s| matches!(s.action.as_str(), "request_review" | "request_help"))
            .all(|s| s.target_session.as_deref() == Some("hive-helper"))
    );
    assert!(
        suggestions
            .iter()
            .any(|s| s.stale_session.as_deref() == Some("hive-stale"))
    );
}

#[test]
fn suggest_coordination_actions_retires_stale_session_without_owned_work() {
    let suggestions = suggest_coordination_actions(
        &HiveCoordinationInboxResponse {
            messages: Vec::new(),
            owned_tasks: Vec::new(),
            help_tasks: Vec::new(),
            review_tasks: Vec::new(),
        },
        &["hive-stale-empty"],
        &[],
        &[],
        &[],
        "codex",
        &[],
        None,
        &[],
    );

    assert!(
        suggestions
            .iter()
            .any(|value| value.action == "retire_session"
                && value.stale_session.as_deref() == Some("hive-stale-empty"))
    );
    assert!(
        !suggestions
            .iter()
            .any(|value| value.action == "recover_session"),
        "stale sessions without owned work should retire instead of recover"
    );
}

#[test]
fn build_gap_candidates_generates_core_gap_signals() {
    let output = std::env::temp_dir().join(format!("memd-gap-test-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&output).expect("create temp output");

    let runtime = None;
    let resume = None;
    let commits = vec!["abc".to_string(), "def".to_string()];
    let mut evidence = Vec::new();

    let candidates = build_gap_candidates(
        &output,
        &runtime,
        &resume,
        None,
        None,
        None,
        None,
        None,
        &commits,
        &mut evidence,
        None,
    );

    assert!(
        candidates
            .iter()
            .any(|value| value.id == "memory:no_eval_snapshot"),
        "baseline eval signal should be present when no eval exists"
    );
    assert!(
        candidates
            .iter()
            .any(|value| value.id == "memory:missing_resume_state"),
        "resume signal should be present when resume and state are missing"
    );
    assert!(
        candidates
            .iter()
            .any(|value| value.id == "coordination:coordination_unreachable"),
        "coordination signal should be present when coordination snapshot is unavailable"
    );
    assert!(
        !evidence.is_empty(),
        "recent commits should generate at least one evidence string"
    );

    fs::remove_dir_all(&output).expect("cleanup temp output");
}

#[test]
fn build_gap_candidates_surfaces_unhived_active_sessions() {
    let output =
        std::env::temp_dir().join(format!("memd-gap-awareness-test-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&output).expect("create temp output");

    let awareness = ProjectAwarenessResponse {
        root: "server:http://127.0.0.1:8787".to_string(),
        current_bundle: output.display().to_string(),
        collisions: vec!["base_url http://127.0.0.1:8787 used by 2 bundles".to_string()],
        entries: vec![
            ProjectAwarenessEntry {
                project_dir: output.display().to_string(),
                bundle_root: output.display().to_string(),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                agent: Some("codex".to_string()),
                session: Some("session-a".to_string()),
                tab_id: Some("tab-a".to_string()),
                effective_agent: Some("codex@session-a".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["project:memd".to_string()],
                hive_group_goal: Some("coordinate memd".to_string()),
                authority: Some("participant".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                presence: "active".to_string(),
                host: None,
                pid: None,
                active_claims: 0,
                workspace: None,
                visibility: None,
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: None,
                pressure: None,
                next_recovery: None,
                last_updated: Some(Utc::now()),
            },
            ProjectAwarenessEntry {
                project_dir: "/tmp/other".to_string(),
                bundle_root: "/tmp/other/.memd".to_string(),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                agent: Some("codex".to_string()),
                session: Some("session-b".to_string()),
                tab_id: None,
                effective_agent: Some("codex@session-b".to_string()),
                hive_system: None,
                hive_role: None,
                capabilities: Vec::new(),
                hive_groups: Vec::new(),
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                presence: "active".to_string(),
                host: None,
                pid: None,
                active_claims: 0,
                workspace: None,
                visibility: None,
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: None,
                pressure: None,
                next_recovery: None,
                last_updated: Some(Utc::now()),
            },
        ],
    };

    let mut evidence = Vec::new();
    let candidates = build_gap_candidates(
        &output,
        &None,
        &None,
        None,
        None,
        None,
        Some(&awareness),
        None,
        &[],
        &mut evidence,
        None,
    );

    assert!(
        candidates
            .iter()
            .any(|value| value.id == "coordination:unhived_active_sessions")
    );
    assert!(
        candidates
            .iter()
            .any(|value| value.id == "coordination:awareness_collisions")
    );

    fs::remove_dir_all(&output).expect("cleanup temp output");
}

#[test]
fn build_gap_candidates_does_not_surface_superseded_stale_remote_sessions() {
    let output = std::env::temp_dir().join(format!(
        "memd-gap-stale-sessions-test-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&output).expect("create temp output");

    let awareness = ProjectAwarenessResponse {
        root: "server:http://127.0.0.1:8787".to_string(),
        current_bundle: output.display().to_string(),
        collisions: Vec::new(),
        entries: vec![ProjectAwarenessEntry {
            project_dir: "remote".to_string(),
            bundle_root: "remote:http://127.0.0.1:8787:session-dead".to_string(),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            base_branch: None,
            agent: Some("codex".to_string()),
            session: Some("session-dead".to_string()),
            tab_id: None,
            effective_agent: Some("codex@session-dead".to_string()),
            hive_system: None,
            hive_role: None,
            capabilities: vec!["memory".to_string()],
            hive_groups: Vec::new(),
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            presence: "stale".to_string(),
            host: None,
            pid: None,
            active_claims: 0,
            workspace: None,
            visibility: None,
            topic_claim: None,
            scope_claims: Vec::new(),
            task_id: None,
            focus: None,
            pressure: None,
            next_recovery: None,
            last_updated: Some(Utc::now()),
        }],
    };

    let mut evidence = Vec::new();
    let candidates = build_gap_candidates(
        &output,
        &None,
        &None,
        None,
        None,
        None,
        Some(&awareness),
        None,
        &[],
        &mut evidence,
        None,
    );

    assert!(
        candidates
            .iter()
            .all(|value| value.id != "coordination:stale_remote_sessions")
    );

    fs::remove_dir_all(&output).expect("cleanup temp output");
}

#[test]
fn build_gap_candidates_surfaces_loop_manifest_drift() {
    let output =
        std::env::temp_dir().join(format!("memd-gap-docs-drift-test-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&output).expect("create temp output");

    let mut evidence = Vec::new();
    let candidates = build_gap_candidates(
        &output,
        &None,
        &None,
        None,
        None,
        None,
        None,
        Some(8),
        &[],
        &mut evidence,
        None,
    );

    assert!(
        candidates
            .iter()
            .any(|value| value.id == "docs:loop_manifest_drift")
    );

    fs::remove_dir_all(&output).expect("cleanup temp output");
}

#[test]
fn collect_gap_repo_evidence_surfaces_repo_docs_and_runtime_signals() {
    let root =
        std::env::temp_dir().join(format!("memd-gap-repo-evidence-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(root.join("docs")).expect("create docs dir");
    fs::create_dir_all(root.join("docs").join("core")).expect("create docs core dir");
    fs::create_dir_all(root.join(".planning")).expect("create planning dir");
    fs::write(
        root.join("README.md"),
        "# memd\n\nThis repo uses memd for memory and setup.",
    )
    .expect("write readme");
    fs::write(
        root.join("ROADMAP.md"),
        "# roadmap\n\n## v6\nmemd gap research loop.",
    )
    .expect("write roadmap");
    fs::write(
        root.join("docs").join("core").join("setup.md"),
        "memd init and codex bootstrap",
    )
    .expect("write setup");
    fs::write(
        root.join(".planning").join("STATE.md"),
        "## Open Loops\n- gap research loop",
    )
    .expect("write state");

    let evidence = collect_gap_repo_evidence(&root);
    assert!(
        evidence.iter().any(|line| line.contains("git branch:")),
        "expected git branch evidence"
    );
    assert!(
        evidence.iter().any(|line| line.contains("README.md:")),
        "expected README evidence"
    );
    assert!(
        evidence.iter().any(|line| line.contains("ROADMAP.md:")),
        "expected ROADMAP evidence"
    );
    assert!(
        evidence
            .iter()
            .any(|line| line.contains("docs/core/setup.md:")),
        "expected setup doc evidence"
    );
    assert!(
        evidence.iter().any(|line| line.contains("runtime wiring:")),
        "expected runtime wiring evidence"
    );

    fs::remove_dir_all(root).expect("cleanup temp repo");
}

#[test]
fn prioritize_gap_candidates_orders_high_to_low_priority() {
    let candidates = vec![
        GapCandidate {
            id: "memory:a".to_string(),
            area: "memory".to_string(),
            priority: 40,
            severity: "low".to_string(),
            signal: "low".to_string(),
            evidence: Vec::new(),
            recommendation: "low-priority".to_string(),
        },
        GapCandidate {
            id: "coordination:b".to_string(),
            area: "coordination".to_string(),
            priority: 90,
            severity: "high".to_string(),
            signal: "high".to_string(),
            evidence: Vec::new(),
            recommendation: "high-priority".to_string(),
        },
        GapCandidate {
            id: "memory:c".to_string(),
            area: "memory".to_string(),
            priority: 70,
            severity: "medium".to_string(),
            signal: "medium".to_string(),
            evidence: Vec::new(),
            recommendation: "medium-priority".to_string(),
        },
    ];
    let sorted = prioritize_gap_candidates(candidates, 2);
    assert_eq!(sorted[0].priority, 90);
    assert_eq!(sorted[1].priority, 70);
}

#[test]
fn evaluate_gap_changes_detects_count_and_status_shift() {
    let baseline = GapReport {
        bundle_root: ".memd".to_string(),
        project: None,
        namespace: None,
        agent: None,
        session: None,
        workspace: None,
        visibility: None,
        limit: 8,
        commits_checked: 0,
        eval_status: Some("usable".to_string()),
        eval_score: Some(70),
        eval_score_delta: Some(-5),
        candidate_count: 6,
        high_priority_count: 2,
        top_priorities: Vec::new(),
        candidates: Vec::new(),
        recommendations: Vec::new(),
        changes: Vec::new(),
        evidence: Vec::new(),
        generated_at: Utc::now(),
        previous_candidate_count: None,
    };

    let current = GapReport {
        bundle_root: ".memd".to_string(),
        project: None,
        namespace: None,
        agent: None,
        session: None,
        workspace: None,
        visibility: None,
        limit: 8,
        commits_checked: 2,
        eval_status: Some("weak".to_string()),
        eval_score: Some(66),
        eval_score_delta: Some(-10),
        candidate_count: 2,
        high_priority_count: 1,
        top_priorities: Vec::new(),
        candidates: Vec::new(),
        recommendations: Vec::new(),
        changes: Vec::new(),
        evidence: Vec::new(),
        generated_at: Utc::now(),
        previous_candidate_count: None,
    };

    let changes = evaluate_gap_changes(&current, Some(&baseline));
    assert!(
        changes
            .iter()
            .any(|value| value.contains("candidate_count 6 -> 2"))
    );
    assert!(
        changes
            .iter()
            .any(|value| value.contains("eval_score Some(70) -> Some(66)"))
    );
    assert!(
        changes
            .iter()
            .any(|value| value.contains("eval_status=weak"))
    );
}

fn test_gap_report(
    candidate_count: usize,
    high_priority_count: usize,
    eval_score: Option<u8>,
    top_priorities: Vec<String>,
) -> GapReport {
    GapReport {
        bundle_root: ".memd".to_string(),
        project: None,
        namespace: None,
        agent: None,
        session: None,
        workspace: None,
        visibility: None,
        limit: 8,
        commits_checked: 0,
        eval_status: None,
        eval_score,
        eval_score_delta: None,
        candidate_count,
        high_priority_count,
        top_priorities,
        candidates: Vec::new(),
        recommendations: Vec::new(),
        changes: Vec::new(),
        evidence: Vec::new(),
        generated_at: Utc::now(),
        previous_candidate_count: None,
    }
}

#[test]
fn build_improvement_actions_dedupes_and_limits() {
    let mut gap = test_gap_report(3, 2, Some(61), vec!["memory:low_eval_score".to_string()]);
    gap.candidates.push(GapCandidate {
        id: "memory:low_eval_score".to_string(),
        area: "memory".to_string(),
        priority: 95,
        severity: "high".to_string(),
        signal: "low_eval_score".to_string(),
        evidence: vec!["evidence".to_string()],
        recommendation: "refresh eval".to_string(),
    });
    let coordination = CoordinationResponse {
        bundle_root: ".memd".to_string(),
        current_session: "codex".to_string(),
        inbox: HiveCoordinationInboxResponse {
            messages: Vec::new(),
            owned_tasks: Vec::new(),
            help_tasks: Vec::new(),
            review_tasks: Vec::new(),
        },
        active_hives: Vec::new(),
        recovery: CoordinationRecoverySummary {
            stale_hives: Vec::new(),
            reclaimable_claims: Vec::new(),
            stalled_tasks: Vec::new(),
            retireable_sessions: Vec::new(),
        },
        lane_fault: None,
        lane_receipts: Vec::new(),
        policy_conflicts: Vec::new(),
        suggestions: (0..10)
            .map(|index| CoordinationSuggestion {
                action: "ack_message".to_string(),
                priority: "medium".to_string(),
                target_session: None,
                task_id: None,
                scope: None,
                message_id: Some(format!("dup-{index}")),
                reason: "dedupe check".to_string(),
                stale_session: None,
            })
            .chain(std::iter::once(CoordinationSuggestion {
                action: "ack_message".to_string(),
                priority: "high".to_string(),
                target_session: None,
                task_id: None,
                scope: None,
                message_id: Some("dup-0".to_string()),
                reason: "dedupe check".to_string(),
                stale_session: None,
            }))
            .collect(),
        boundary_recommendations: Vec::new(),
        receipts: Vec::new(),
    };
    let actions = build_improvement_actions(&gap, Some(&coordination));
    assert!(
        actions.len() <= 8,
        "action list should be bounded by apply_improvement cap"
    );
    assert!(
        actions
            .iter()
            .filter(|value| value.action == "refresh_eval")
            .count()
            == 1,
        "low_eval_score only yields one refresh_eval action"
    );
    assert!(
        actions
            .iter()
            .filter(|value| value.message_id.as_deref() == Some("dup-0"))
            .count()
            <= 1,
        "duplicate suggestion keys should dedupe"
    );
}

#[test]
fn build_improvement_actions_includes_retire_session_suggestions() {
    let gap = test_gap_report(
        2,
        1,
        Some(70),
        vec!["coordination:stale_remote_sessions".to_string()],
    );
    let coordination = CoordinationResponse {
        bundle_root: ".memd".to_string(),
        current_session: "codex".to_string(),
        inbox: HiveCoordinationInboxResponse {
            messages: Vec::new(),
            owned_tasks: Vec::new(),
            help_tasks: Vec::new(),
            review_tasks: Vec::new(),
        },
        active_hives: Vec::new(),
        recovery: CoordinationRecoverySummary {
            stale_hives: Vec::new(),
            reclaimable_claims: Vec::new(),
            stalled_tasks: Vec::new(),
            retireable_sessions: Vec::new(),
        },
        lane_fault: None,
        lane_receipts: Vec::new(),
        policy_conflicts: Vec::new(),
        suggestions: vec![CoordinationSuggestion {
            action: "retire_session".to_string(),
            priority: "medium".to_string(),
            target_session: None,
            task_id: None,
            scope: None,
            message_id: None,
            reason: "retire stale session".to_string(),
            stale_session: Some("session-stale".to_string()),
        }],
        boundary_recommendations: Vec::new(),
        receipts: Vec::new(),
    };

    let actions = build_improvement_actions(&gap, Some(&coordination));
    assert!(actions.iter().any(|value| {
        value.action == "retire_session" && value.target_session.as_deref() == Some("session-stale")
    }));
}

#[test]
fn render_coordination_summary_surfaces_retireable_sessions() {
    let summary = render_coordination_summary(
        &CoordinationResponse {
            bundle_root: ".memd".to_string(),
            current_session: "codex".to_string(),
            inbox: HiveCoordinationInboxResponse {
                messages: Vec::new(),
                owned_tasks: Vec::new(),
                help_tasks: Vec::new(),
                review_tasks: Vec::new(),
            },
            active_hives: vec![ProjectAwarenessEntry {
                project_dir: "remote".to_string(),
                bundle_root: "remote:http://127.0.0.1:8787:active".to_string(),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: Some("/tmp/peer".to_string()),
                branch: Some("feature/peer".to_string()),
                base_branch: Some("main".to_string()),
                agent: Some("claude-code".to_string()),
                session: Some("active".to_string()),
                tab_id: None,
                effective_agent: Some("claude-code@active".to_string()),
                hive_system: Some("claude-code".to_string()),
                hive_role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["project:demo".to_string()],
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                presence: "active".to_string(),
                host: Some("workstation".to_string()),
                pid: Some(2),
                active_claims: 1,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                topic_claim: Some("Refine parser overlap flow".to_string()),
                scope_claims: vec![
                    "task:parser-refactor".to_string(),
                    "crates/memd-client/src/main.rs".to_string(),
                ],
                task_id: Some("parser-refactor".to_string()),
                focus: None,
                pressure: None,
                next_recovery: None,
                last_updated: Some(Utc::now()),
            }],
            recovery: CoordinationRecoverySummary {
                stale_hives: Vec::new(),
                reclaimable_claims: Vec::new(),
                stalled_tasks: Vec::new(),
                retireable_sessions: vec![ProjectAwarenessEntry {
                    project_dir: "remote".to_string(),
                    bundle_root: "remote:http://127.0.0.1:8787:stale".to_string(),
                    project: Some("demo".to_string()),
                    namespace: Some("main".to_string()),
                    repo_root: None,
                    worktree_root: None,
                    branch: None,
                    base_branch: None,
                    agent: Some("codex".to_string()),
                    session: Some("stale".to_string()),
                    tab_id: None,
                    effective_agent: Some("codex@stale".to_string()),
                    hive_system: Some("codex".to_string()),
                    hive_role: Some("agent".to_string()),
                    capabilities: vec!["memory".to_string()],
                    hive_groups: vec!["project:demo".to_string()],
                    hive_group_goal: None,
                    authority: Some("participant".to_string()),
                    base_url: Some("http://127.0.0.1:8787".to_string()),
                    presence: "stale".to_string(),
                    host: Some("workstation".to_string()),
                    pid: Some(1),
                    active_claims: 0,
                    workspace: Some("shared".to_string()),
                    visibility: Some("workspace".to_string()),
                    topic_claim: None,
                    scope_claims: Vec::new(),
                    task_id: None,
                    focus: None,
                    pressure: None,
                    next_recovery: None,
                    last_updated: Some(Utc::now()),
                }],
            },
            lane_fault: Some(serde_json::json!({
                "kind": "unsafe_same_branch",
                "session": "claude-b",
                "branch": "feature/hive-shared",
                "worktree_root": "/tmp/worktree"
            })),
            lane_receipts: vec![HiveCoordinationReceiptRecord {
                id: "lane-1".to_string(),
                kind: "queen_deny".to_string(),
                actor_session: "queen".to_string(),
                actor_agent: Some("codex@queen".to_string()),
                target_session: Some("claude-b".to_string()),
                task_id: None,
                scope: None,
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                summary: "Queen denied overlap".to_string(),
                created_at: Utc::now(),
            }],
            policy_conflicts: Vec::new(),
            suggestions: Vec::new(),
            boundary_recommendations: Vec::new(),
            receipts: Vec::new(),
        },
        Some("overview"),
    );

    assert!(summary.contains("retireable_sessions=1"));
    assert!(summary.contains("lane_fault=yes"));
    assert!(summary.contains("lane_receipts=1"));
    assert!(summary.contains("## Active Hive"));
    assert!(summary.contains("task=parser-refactor"));
    assert!(summary.contains("work=\"Refine parser overlap flow\""));
}

#[test]
fn render_session_summary_surfaces_rebind_and_retire_state() {
    let summary = render_session_summary(&SessionResponse {
        action: "rebind+retire".to_string(),
        bundle_root: ".memd".to_string(),
        bundle_session: Some("codex-fresh".to_string()),
        live_session: Some("codex-fresh".to_string()),
        rebased_from: Some("codex-stale".to_string()),
        tab_id: Some("tab-alpha".to_string()),
        reconciled: true,
        reconciled_retired_sessions: 2,
        retired_sessions: 1,
        retire_target: Some("codex-old".to_string()),
        heartbeat: Some(serde_json::json!({"status":"active"})),
    });

    assert!(summary.contains("action=rebind+retire"));
    assert!(summary.contains("bundle_session=codex-fresh"));
    assert!(summary.contains("live_session=codex-fresh"));
    assert!(summary.contains("rebased_from=codex-stale"));
    assert!(summary.contains("reconciled=yes"));
    assert!(summary.contains("reconciled_retired=2"));
    assert!(summary.contains("retired=1"));
    assert!(summary.contains("retire_target=codex-old"));
    assert!(summary.contains("heartbeat=published"));
}

#[test]
fn render_tasks_summary_surfaces_task_taxonomy_counts() {
    let now = Utc::now();
    let summary = render_tasks_summary(&TasksResponse {
        bundle_root: ".memd".to_string(),
        current_session: Some("codex-a".to_string()),
        tasks: vec![
            HiveTaskRecord {
                task_id: "t1".to_string(),
                title: "exclusive open".to_string(),
                description: None,
                status: "in_progress".to_string(),
                coordination_mode: "exclusive_write".to_string(),
                session: Some("codex-a".to_string()),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@codex-a".to_string()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                claim_scopes: vec!["src/main.rs".to_string()],
                help_requested: true,
                review_requested: false,
                created_at: now,
                updated_at: now,
            },
            HiveTaskRecord {
                task_id: "t2".to_string(),
                title: "shared review".to_string(),
                description: None,
                status: "needs_review".to_string(),
                coordination_mode: "shared_review".to_string(),
                session: Some("codex-b".to_string()),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@codex-b".to_string()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                claim_scopes: vec![],
                help_requested: false,
                review_requested: true,
                created_at: now,
                updated_at: now,
            },
            HiveTaskRecord {
                task_id: "t3".to_string(),
                title: "closed".to_string(),
                description: None,
                status: "done".to_string(),
                coordination_mode: "shared_review".to_string(),
                session: Some("codex-c".to_string()),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@codex-c".to_string()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                claim_scopes: vec![],
                help_requested: false,
                review_requested: false,
                created_at: now,
                updated_at: now,
            },
        ],
    });

    assert!(summary.contains("count=3"));
    assert!(summary.contains("open=2"));
    assert!(summary.contains("help=1"));
    assert!(summary.contains("review=1"));
    assert!(summary.contains("exclusive=1"));
    assert!(summary.contains("shared=2"));
    assert!(summary.contains("active_sessions=2"));
    assert!(summary.contains("owned=1"));
}

#[test]
fn render_capabilities_runtime_summary_surfaces_harness_breakdown() {
    let summary = render_capabilities_runtime_summary(&CapabilitiesResponse {
        bundle_root: ".memd".to_string(),
        generated_at: Utc::now(),
        discovered: 7,
        universal: 2,
        bridgeable: 3,
        harness_native: 2,
        bridge_actions: 4,
        wired_harnesses: 2,
        filters: serde_json::json!({}),
        harnesses: vec![
            CapabilityHarnessSummary {
                harness: "codex".to_string(),
                capabilities: 3,
                installed: 2,
                bridge_actions: 1,
            },
            CapabilityHarnessSummary {
                harness: "claude-code".to_string(),
                capabilities: 4,
                installed: 4,
                bridge_actions: 3,
            },
        ],
        records: Vec::new(),
    });

    assert!(summary.contains("discovered=7"));
    assert!(summary.contains("bridge_actions=4"));
    assert!(summary.contains("wired_harnesses=2"));
    assert!(summary.contains("shown=0"));
    assert!(summary.contains("codex:3/2/1"));
    assert!(summary.contains("claude-code:4/4/3"));
}

#[test]
fn render_memory_surface_summary_surfaces_truth_and_tiers() {
    let summary = render_memory_surface_summary(&MemorySurfaceResponse {
        bundle_root: ".memd".to_string(),
        truth_summary: TruthSummary {
            retrieval_tier: RetrievalTier::Hot,
            truth: "current".to_string(),
            epistemic_state: "verified".to_string(),
            freshness: "fresh".to_string(),
            confidence: 0.97,
            action_hint: "keep current truth hot".to_string(),
            source_count: 2,
            contested_sources: 0,
            compact_records: 5,
            records: vec![TruthRecordSummary {
                lane: "live_truth".to_string(),
                truth: "current".to_string(),
                epistemic_state: "verified".to_string(),
                freshness: "fresh".to_string(),
                retrieval_tier: RetrievalTier::Hot,
                confidence: 0.97,
                provenance: "event_spine / compact".to_string(),
                preview: "Current live truth head".to_string(),
            }],
        },
        context_records: 2,
        working_records: 3,
        inbox_items: 1,
        source_lanes: 2,
        rehydration_queue: 1,
        semantic_hits: 2,
        change_summary: 1,
        estimated_prompt_tokens: 180,
        refresh_recommended: false,
        contradiction_pressure: 2,
        superseded_pressure: 1,
        contradiction_reasons: vec!["live_truth:current:fresh".to_string()],
        superseded_reasons: vec!["refresh_recommended".to_string()],
        records: vec![TruthRecordSummary {
            lane: "live_truth".to_string(),
            truth: "current".to_string(),
            epistemic_state: "verified".to_string(),
            freshness: "fresh".to_string(),
            retrieval_tier: RetrievalTier::Hot,
            confidence: 0.97,
            provenance: "event_spine / compact".to_string(),
            preview: "Current live truth head".to_string(),
        }],
    });

    assert!(summary.contains("truth=current"));
    assert!(summary.contains("epistemic=verified"));
    assert!(summary.contains("freshness=fresh"));
    assert!(summary.contains("retrieval=hot"));
    assert!(summary.contains("working:3"));
    assert!(summary.contains("sources:2"));
    assert!(summary.contains("tok=180"));
    assert!(summary.contains("contradictions=2"));
    assert!(summary.contains("superseded=1"));
    assert!(summary.contains("head=live_truth"));
}

#[test]
fn render_claims_summary_surfaces_continuity_overlay() {
    let summary = render_claims_summary(&ClaimsResponse {
        bundle_root: ".memd".to_string(),
        bundle_session: Some("codex-stale".to_string()),
        live_session: Some("codex-fresh".to_string()),
        rebased_from: Some("codex-stale".to_string()),
        current_session: Some("codex-fresh".to_string()),
        current_tab_id: Some("tab-a".to_string()),
        claims: Vec::new(),
    });

    assert!(summary.contains("bundle_session=codex-stale"));
    assert!(summary.contains("live_session=codex-fresh"));
    assert!(summary.contains("rebased_from=codex-stale"));
}

#[test]
fn render_messages_summary_surfaces_continuity_overlay() {
    let summary = render_messages_summary(&MessagesResponse {
        bundle_root: ".memd".to_string(),
        bundle_session: Some("codex-stale".to_string()),
        live_session: Some("codex-fresh".to_string()),
        rebased_from: Some("codex-stale".to_string()),
        current_session: Some("codex-fresh".to_string()),
        messages: Vec::new(),
    });

    assert!(summary.contains("bundle_session=codex-stale"));
    assert!(summary.contains("live_session=codex-fresh"));
    assert!(summary.contains("rebased_from=codex-stale"));
}

#[test]
fn run_capabilities_command_filters_records() {
    let output =
        std::env::temp_dir().join(format!("memd-capabilities-filter-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&output).expect("create output");

    let response = run_capabilities_command(&CapabilitiesArgs {
        output: output.clone(),
        harness: Some("codex".to_string()),
        kind: None,
        portability: None,
        query: Some("memory".to_string()),
        limit: 8,
        summary: true,
        json: false,
    })
    .expect("capabilities response");

    assert!(
        response
            .records
            .iter()
            .all(|record| record.harness == "codex")
    );
    assert!(response.records.len() <= 8);

    fs::remove_dir_all(output).expect("cleanup output");
}

#[test]
fn cli_accepts_reload_as_refresh_alias() {
    let cli = Cli::try_parse_from(["memd", "reload", "--output", ".memd", "--summary"])
        .expect("reload alias should parse");

    match cli.command {
        Commands::Refresh(args) => {
            assert_eq!(args.output, PathBuf::from(".memd"));
            assert!(args.summary);
        }
        other => panic!("expected refresh command, got {other:?}"),
    }
}

#[test]
fn improvement_progress_tracks_candidate_score_and_priority_change() {
    let baseline = test_gap_report(10, 3, Some(82), vec!["a".to_string(), "b".to_string()]);
    let fewer_candidates = test_gap_report(8, 3, Some(82), vec!["a".to_string(), "b".to_string()]);
    let better_score = test_gap_report(10, 3, Some(84), vec!["a".to_string(), "b".to_string()]);
    let changed_priorities =
        test_gap_report(10, 3, Some(82), vec!["x".to_string(), "a".to_string()]);
    let no_change = test_gap_report(10, 3, Some(82), vec!["a".to_string(), "b".to_string()]);

    assert!(improvement_progress(&baseline, &fewer_candidates));
    assert!(improvement_progress(&baseline, &better_score));
    assert!(improvement_progress(&baseline, &changed_priorities));
    assert!(!improvement_progress(&baseline, &no_change));
}
