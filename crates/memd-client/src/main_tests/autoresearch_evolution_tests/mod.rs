use super::*;

pub(super) fn test_autoresearch_snapshot(
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
    cache::write_resume_snapshot_cache(output, &cache_key, snapshot).expect("write resume cache");
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
    let record = run_autonomy_quality_loop(&output, "http://127.0.0.1:59999", descriptor, 0, None)
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
    let signature =
        build_autoresearch_sweep_signature(&[(descriptor, record.clone()), (descriptor, record)]);

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
    let record = run_prompt_efficiency_loop(&output, "http://127.0.0.1:59999", descriptor, 0, None)
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
    let record = run_memory_hygiene_loop(&output, "http://127.0.0.1:59999", descriptor, 0, None)
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
    let root = std::env::temp_dir().join(format!("memd-branch-review-{}", uuid::Uuid::new_v4()));
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
    let output = std::env::temp_dir().join(format!("memd-repair-rate-{}", uuid::Uuid::new_v4()));
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
    let record = run_prompt_efficiency_loop(&output, "http://127.0.0.1:59999", descriptor, 0, None)
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
    let output = std::env::temp_dir().join(format!("memd-self-evolution-{}", uuid::Uuid::new_v4()));
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
    let output = std::env::temp_dir().join(format!("memd-hive-health-{}", uuid::Uuid::new_v4()));
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

#[tokio::test]
async fn run_experiment_command_restores_bundle_when_rejected() {
    let dir = std::env::temp_dir().join(format!("memd-experiment-reject-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("create experiment temp bundle");
    fs::write(
        dir.join("config.json"),
        r#"{
  "project": "demo",
  "agent": "codex",
  "route": "auto",
  "intent": "general"
}
"#,
    )
    .expect("write config");
    fs::write(dir.join("MEMD_MEMORY.md"), "# sentinel memory\n").expect("write memory seed");
    fs::create_dir_all(dir.join("agents")).expect("create agents dir");
    fs::write(
        dir.join("agents").join("CODEX_MEMORY.md"),
        "# agent sentinel\n",
    )
    .expect("write agent memory seed");

    let base_url = spawn_mock_memory_server().await;
    let report = run_experiment_command(
        &ExperimentArgs {
            output: dir.clone(),
            max_iterations: 1,
            limit: Some(8),
            recent_commits: Some(1),
            accept_below: 99,
            apply: true,
            consolidate: false,
            write: false,
            summary: false,
        },
        &base_url,
    )
    .await
    .expect("run experiment");

    assert!(!report.accepted);
    assert!(report.restored);
    assert!(
        fs::read_to_string(dir.join("MEMD_MEMORY.md"))
            .expect("read restored memory")
            .contains("sentinel memory")
    );
    write_experiment_artifacts(&dir, &report).expect("write experiment artifacts");
    assert!(dir.join("experiments").join("latest.json").exists());

    fs::remove_dir_all(dir).expect("cleanup experiment temp bundle");
}

#[tokio::test]
async fn run_experiment_command_consolidates_accepted_learnings() {
    let dir = std::env::temp_dir().join(format!("memd-experiment-accept-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("create experiment temp bundle");
    fs::write(
        dir.join("config.json"),
        r#"{
  "project": "demo",
  "agent": "codex",
  "route": "auto",
  "intent": "general"
}
"#,
    )
    .expect("write config");
    fs::write(dir.join("MEMD_MEMORY.md"), "# baseline memory\n").expect("write memory seed");
    fs::create_dir_all(dir.join("agents")).expect("create agents dir");
    fs::write(
        dir.join("agents").join("CLAUDE_CODE_MEMORY.md"),
        "# agent baseline\n",
    )
    .expect("write agent memory seed");

    let eval = high_scoring_eval(&dir);
    write_bundle_eval_artifacts(&dir, &eval).expect("write eval artifacts");
    let scenario = high_scoring_scenario(&dir);
    write_scenario_artifacts(&dir, &scenario).expect("write scenario artifacts");

    let base_url = spawn_mock_memory_server().await;
    let report = run_experiment_command(
        &ExperimentArgs {
            output: dir.clone(),
            max_iterations: 0,
            limit: Some(8),
            recent_commits: Some(1),
            accept_below: 80,
            apply: false,
            consolidate: true,
            write: false,
            summary: false,
        },
        &base_url,
    )
    .await
    .expect("run experiment");

    assert!(report.accepted);
    assert!(!report.restored);
    assert!(!report.learnings.is_empty());
    assert!(
        fs::read_to_string(dir.join("MEMD_MEMORY.md"))
            .expect("read consolidated memory")
            .contains("Accepted Experiment")
    );
    write_experiment_artifacts(&dir, &report).expect("write experiment artifacts");
    let latest_json = dir.join("experiments").join("latest.json");
    assert!(latest_json.exists());
    let parsed: ExperimentReport =
        serde_json::from_str(&fs::read_to_string(&latest_json).expect("read experiment json"))
            .expect("parse experiment report");
    assert!(parsed.accepted);

    fs::remove_dir_all(dir).expect("cleanup experiment temp bundle");
}

#[tokio::test]
async fn run_experiment_command_forces_one_iteration_when_apply_true_and_max_iterations_zero() {
    let dir = std::env::temp_dir().join(format!(
        "memd-experiment-force-iteration-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("create experiment temp bundle");
    fs::write(
        dir.join("config.json"),
        r#"{
  "project": "demo",
  "agent": "codex",
  "route": "auto",
  "intent": "general"
}
"#,
    )
    .expect("write config");
    fs::write(dir.join("MEMD_MEMORY.md"), "# baseline memory\n").expect("write memory seed");
    fs::create_dir_all(dir.join("agents")).expect("create agents dir");
    fs::write(
        dir.join("agents").join("CLAUDE_CODE_MEMORY.md"),
        "# agent baseline\n",
    )
    .expect("write agent memory seed");

    let base_url = spawn_mock_memory_server().await;
    let report = run_experiment_command(
        &ExperimentArgs {
            output: dir.clone(),
            max_iterations: 0,
            limit: Some(8),
            recent_commits: Some(1),
            accept_below: 80,
            apply: true,
            consolidate: false,
            write: false,
            summary: false,
        },
        &base_url,
    )
    .await
    .expect("run experiment");

    assert_eq!(report.improvement.max_iterations, 1);
    assert_eq!(report.improvement.iterations.len(), 1);
    assert!(
        report
            .trail
            .iter()
            .any(|line| line.contains("max_iterations=1"))
    );

    fs::remove_dir_all(dir).expect("cleanup experiment temp bundle");
}

#[test]
fn write_experiment_artifacts_writes_evolution_proposal_and_ledger() {
    let dir =
        std::env::temp_dir().join(format!("memd-evolution-proposal-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("create temp bundle");

    let mut report = test_experiment_report(&dir, true, false, 92, 100, Utc::now());
    report.improvement.final_changes = vec![
        "tighten scoring heuristic for self-evolution loop".to_string(),
        "adjust loop threshold floor".to_string(),
    ];
    write_experiment_artifacts(&dir, &report).expect("write experiment artifacts");

    let proposal_path = dir.join("evolution").join("latest-proposal.json");
    let ledger_path = dir.join("evolution").join("durability-ledger.json");
    assert!(proposal_path.exists());
    assert!(ledger_path.exists());

    let proposal: EvolutionProposalReport =
        serde_json::from_str(&fs::read_to_string(&proposal_path).expect("read proposal"))
            .expect("parse proposal");
    assert_eq!(proposal.state, "accepted_proposal");
    assert_eq!(proposal.scope_gate, "auto_merge");
    assert!(proposal.merge_eligible);

    let ledger: EvolutionDurabilityLedger =
        serde_json::from_str(&fs::read_to_string(&ledger_path).expect("read ledger"))
            .expect("parse ledger");
    assert_eq!(ledger.entries.len(), 1);
    assert_eq!(ledger.entries[0].state, "accepted_proposal");

    fs::remove_dir_all(dir).expect("cleanup experiment temp bundle");
}

#[test]
fn write_experiment_artifacts_keeps_coordination_scope_on_proposal_branch() {
    let dir = std::env::temp_dir().join(format!(
        "memd-evolution-proposal-coordination-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("create temp bundle");

    let mut report = test_experiment_report(&dir, true, false, 91, 100, Utc::now());
    report.improvement.final_changes =
        vec!["change hive coordination protocol for task claims".to_string()];
    write_experiment_artifacts(&dir, &report).expect("write experiment artifacts");

    let proposal: EvolutionProposalReport = serde_json::from_str(
        &fs::read_to_string(dir.join("evolution").join("latest-proposal.json"))
            .expect("read proposal"),
    )
    .expect("parse proposal");
    assert_eq!(proposal.scope_class, "coordination_semantics");
    assert_eq!(proposal.scope_gate, "proposal_only");
    assert!(!proposal.merge_eligible);
    assert_eq!(proposal.state, "accepted_proposal");

    fs::remove_dir_all(dir).expect("cleanup experiment temp bundle");
}

#[test]
fn write_experiment_artifacts_creates_isolated_evolution_branch_and_authority_ledger() {
    let root = std::env::temp_dir().join(format!("memd-evolution-branch-{}", uuid::Uuid::new_v4()));
    let output = root.join(".memd");
    fs::create_dir_all(&output).expect("create temp bundle");
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
        .arg("config")
        .arg("user.email")
        .arg("memd@example.com")
        .status()
        .expect("git user email");
    let _ = Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("config")
        .arg("user.name")
        .arg("memd")
        .status()
        .expect("git user name");
    fs::write(root.join("README.md"), "# demo\n").expect("write readme");
    let _ = Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("add")
        .arg(".")
        .status()
        .expect("git add");
    let _ = Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("commit")
        .arg("-m")
        .arg("init")
        .status()
        .expect("git commit");

    let mut report = test_experiment_report(&output, true, false, 100, 100, Utc::now());
    report.improvement.final_changes = vec![
        "tighten evaluation score heuristic".to_string(),
        "adjust loop threshold floor".to_string(),
    ];
    write_experiment_artifacts(&output, &report).expect("write experiment artifacts");

    let branch_manifest: EvolutionBranchManifest = serde_json::from_str(
        &fs::read_to_string(output.join("evolution").join("latest-branch.json"))
            .expect("read branch manifest"),
    )
    .expect("parse branch manifest");
    assert!(matches!(
        branch_manifest.status.as_str(),
        "created" | "existing"
    ));
    assert!(branch_manifest.branch.starts_with("auto/evolution/"));

    let branch_exists = Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("show-ref")
        .arg("--verify")
        .arg(format!("refs/heads/{}", branch_manifest.branch))
        .status()
        .expect("show ref")
        .success();
    assert!(branch_exists);

    let authority: EvolutionAuthorityLedger = serde_json::from_str(
        &fs::read_to_string(output.join("evolution").join("authority-ledger.json"))
            .expect("read authority ledger"),
    )
    .expect("parse authority ledger");
    assert_eq!(authority.entries.len(), 1);
    assert_eq!(authority.entries[0].authority_tier, "phase1_auto_merge");

    let merge_queue: EvolutionMergeQueue = serde_json::from_str(
        &fs::read_to_string(output.join("evolution").join("merge-queue.json"))
            .expect("read merge queue"),
    )
    .expect("parse merge queue");
    assert_eq!(merge_queue.entries.len(), 1);
    assert_eq!(merge_queue.entries[0].status, "no_diff");

    let durability_queue: EvolutionDurabilityQueue = serde_json::from_str(
        &fs::read_to_string(output.join("evolution").join("durability-queue.json"))
            .expect("read durability queue"),
    )
    .expect("parse durability queue");
    assert_eq!(durability_queue.entries.len(), 1);
    assert_eq!(durability_queue.entries[0].status, "no_diff");

    fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn git_worktree_conflicts_with_branch_detects_overlap() {
    let root = std::env::temp_dir().join(format!("memd-evolution-dirty-{}", uuid::Uuid::new_v4()));
    let output = root.join(".memd");
    fs::create_dir_all(&output).expect("create temp bundle");
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
        .arg("config")
        .arg("user.email")
        .arg("memd@example.com")
        .status()
        .expect("git user email");
    let _ = Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("config")
        .arg("user.name")
        .arg("memd")
        .status()
        .expect("git user name");
    fs::write(root.join("README.md"), "# demo\n").expect("write readme");
    let _ = Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("add")
        .arg(".")
        .status()
        .expect("git add");
    let _ = Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("commit")
        .arg("-m")
        .arg("init")
        .status()
        .expect("git commit");

    let mut report = test_experiment_report(&output, true, false, 100, 100, Utc::now());
    report.improvement.final_changes = vec!["adjust loop threshold floor".to_string()];
    write_experiment_artifacts(&output, &report).expect("write experiment artifacts");

    let proposal: EvolutionProposalReport = serde_json::from_str(
        &fs::read_to_string(output.join("evolution").join("latest-proposal.json"))
            .expect("read proposal"),
    )
    .expect("parse proposal");
    let branch_manifest: EvolutionBranchManifest = serde_json::from_str(
        &fs::read_to_string(output.join("evolution").join("latest-branch.json"))
            .expect("read branch manifest"),
    )
    .expect("parse branch manifest");

    let _ = Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("checkout")
        .arg(&proposal.branch)
        .status()
        .expect("checkout proposal branch");
    fs::write(root.join("README.md"), "# demo\n\noverlap change\n").expect("write branch change");
    let _ = Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("add")
        .arg("README.md")
        .status()
        .expect("git add readme");
    let _ = Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("commit")
        .arg("-m")
        .arg("proposal change")
        .status()
        .expect("commit proposal change");
    let _ = Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("checkout")
        .arg(branch_manifest.base_branch.as_deref().unwrap_or("master"))
        .status()
        .expect("checkout base branch");
    fs::write(root.join("README.md"), "# demo\n\ndirty overlap\n").expect("write dirty readme");

    assert!(git_worktree_conflicts_with_branch(
        &root,
        branch_manifest.base_branch.as_deref().unwrap_or("master"),
        &proposal.branch
    ));

    fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn process_evolution_queues_merges_ready_auto_merge_branch() {
    let root = std::env::temp_dir().join(format!(
        "memd-evolution-merge-ready-{}",
        uuid::Uuid::new_v4()
    ));
    let output = root.join(".memd");
    fs::create_dir_all(&output).expect("create temp bundle");
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
        .arg("config")
        .arg("user.email")
        .arg("memd@example.com")
        .status()
        .expect("git user email");
    let _ = Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("config")
        .arg("user.name")
        .arg("memd")
        .status()
        .expect("git user name");
    fs::write(root.join("README.md"), "# demo\n").expect("write readme");
    let _ = Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("add")
        .arg(".")
        .status()
        .expect("git add");
    let _ = Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("commit")
        .arg("-m")
        .arg("init")
        .status()
        .expect("git commit");

    let mut report = test_experiment_report(&output, true, false, 100, 100, Utc::now());
    report.improvement.final_changes = vec!["adjust loop threshold floor".to_string()];
    write_experiment_artifacts(&output, &report).expect("write experiment artifacts");

    let proposal: EvolutionProposalReport = serde_json::from_str(
        &fs::read_to_string(output.join("evolution").join("latest-proposal.json"))
            .expect("read proposal"),
    )
    .expect("parse proposal");
    let branch_manifest: EvolutionBranchManifest = serde_json::from_str(
        &fs::read_to_string(output.join("evolution").join("latest-branch.json"))
            .expect("read branch manifest"),
    )
    .expect("parse branch manifest");
    assert_eq!(proposal.authority_tier, "phase1_auto_merge");
    assert!(proposal.merge_eligible);

    let _ = Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("checkout")
        .arg(&proposal.branch)
        .status()
        .expect("checkout proposal branch");
    fs::write(root.join("README.md"), "# demo\n\nmerged change\n").expect("write branch change");
    let _ = Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("add")
        .arg("README.md")
        .status()
        .expect("git add readme");
    let _ = Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("commit")
        .arg("-m")
        .arg("proposal change")
        .status()
        .expect("commit proposal change");
    let _ = Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("checkout")
        .arg(branch_manifest.base_branch.as_deref().unwrap_or("master"))
        .status()
        .expect("checkout base branch");
    fs::write(root.join("NOTES.txt"), "unrelated dirty file\n")
        .expect("write unrelated dirty file");

    process_evolution_queues(&output).expect("process evolution queues");

    let merge_queue: EvolutionMergeQueue = serde_json::from_str(
        &fs::read_to_string(output.join("evolution").join("merge-queue.json"))
            .expect("read merge queue"),
    )
    .expect("parse merge queue");
    assert_eq!(merge_queue.entries[0].status, "merged");

    let durability_queue: EvolutionDurabilityQueue = serde_json::from_str(
        &fs::read_to_string(output.join("evolution").join("durability-queue.json"))
            .expect("read durability queue"),
    )
    .expect("parse durability queue");
    assert_eq!(durability_queue.entries[0].status, "scheduled");

    let latest_proposal: EvolutionProposalReport = serde_json::from_str(
        &fs::read_to_string(output.join("evolution").join("latest-proposal.json"))
            .expect("read latest proposal"),
    )
    .expect("parse latest proposal");
    assert_eq!(latest_proposal.state, "merged");

    let latest_branch: EvolutionBranchManifest = serde_json::from_str(
        &fs::read_to_string(output.join("evolution").join("latest-branch.json"))
            .expect("read latest branch"),
    )
    .expect("parse latest branch");
    assert_eq!(latest_branch.status, "merged");

    let base_head = git_stdout(
        &root,
        &[
            "rev-parse",
            branch_manifest.base_branch.as_deref().unwrap_or("master"),
        ],
    )
    .expect("read base head");
    let proposal_head =
        git_stdout(&root, &["rev-parse", &proposal.branch]).expect("read proposal head");
    assert_eq!(base_head, proposal_head);

    fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn process_evolution_queues_promotes_merged_branch_to_durable_truth() {
    let root =
        std::env::temp_dir().join(format!("memd-evolution-durable-{}", uuid::Uuid::new_v4()));
    let output = root.join(".memd");
    fs::create_dir_all(&output).expect("create temp bundle");
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
        .arg("config")
        .arg("user.email")
        .arg("memd@example.com")
        .status()
        .expect("git user email");
    let _ = Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("config")
        .arg("user.name")
        .arg("memd")
        .status()
        .expect("git user name");
    fs::write(root.join("README.md"), "# demo\n").expect("write readme");
    let _ = Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("add")
        .arg(".")
        .status()
        .expect("git add");
    let _ = Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("commit")
        .arg("-m")
        .arg("init")
        .status()
        .expect("git commit");

    let mut report = test_experiment_report(&output, true, false, 100, 100, Utc::now());
    report.improvement.final_changes = vec!["adjust loop threshold floor".to_string()];
    write_experiment_artifacts(&output, &report).expect("write experiment artifacts");

    let proposal: EvolutionProposalReport = serde_json::from_str(
        &fs::read_to_string(output.join("evolution").join("latest-proposal.json"))
            .expect("read proposal"),
    )
    .expect("parse proposal");
    let branch_manifest: EvolutionBranchManifest = serde_json::from_str(
        &fs::read_to_string(output.join("evolution").join("latest-branch.json"))
            .expect("read branch manifest"),
    )
    .expect("parse branch manifest");

    let _ = Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("checkout")
        .arg(&proposal.branch)
        .status()
        .expect("checkout proposal branch");
    fs::write(root.join("README.md"), "# demo\n\ndurable change\n").expect("write branch change");
    let _ = Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("add")
        .arg("README.md")
        .status()
        .expect("git add readme");
    let _ = Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("commit")
        .arg("-m")
        .arg("proposal change")
        .status()
        .expect("commit proposal change");
    let _ = Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("checkout")
        .arg(branch_manifest.base_branch.as_deref().unwrap_or("master"))
        .status()
        .expect("checkout base branch");

    process_evolution_queues(&output).expect("merge proposal");

    let mut durability_queue: EvolutionDurabilityQueue = serde_json::from_str(
        &fs::read_to_string(output.join("evolution").join("durability-queue.json"))
            .expect("read durability queue"),
    )
    .expect("parse durability queue");
    durability_queue.entries[0].due_at = Some(Utc::now() - chrono::TimeDelta::minutes(1));
    write_evolution_durability_queue(&output, &durability_queue).expect("rewrite durability queue");

    process_evolution_queues(&output).expect("process durability");

    let final_queue: EvolutionDurabilityQueue = serde_json::from_str(
        &fs::read_to_string(output.join("evolution").join("durability-queue.json"))
            .expect("read final durability queue"),
    )
    .expect("parse final durability queue");
    assert_eq!(final_queue.entries[0].status, "verified");

    let latest_proposal: EvolutionProposalReport = serde_json::from_str(
        &fs::read_to_string(output.join("evolution").join("latest-proposal.json"))
            .expect("read latest proposal"),
    )
    .expect("parse latest proposal");
    assert_eq!(latest_proposal.state, "durable_truth");
    assert!(latest_proposal.durable_truth);

    let latest_branch: EvolutionBranchManifest = serde_json::from_str(
        &fs::read_to_string(output.join("evolution").join("latest-branch.json"))
            .expect("read latest branch"),
    )
    .expect("parse latest branch");
    assert_eq!(latest_branch.status, "durable_truth");
    assert!(latest_branch.durable_truth);

    fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn compute_evolution_authority_tier_expands_and_contracts() {
    let output =
        std::env::temp_dir().join(format!("memd-evolution-authority-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(output.join("evolution")).expect("create evolution dir");

    fs::write(
        output.join("evolution").join("authority-ledger.json"),
        serde_json::to_string_pretty(&EvolutionAuthorityLedger {
            entries: vec![
                EvolutionAuthorityEntry {
                    scope_class: "runtime_policy".to_string(),
                    authority_tier: "phase1_auto_merge".to_string(),
                    accepted: true,
                    merged: true,
                    durable_truth: true,
                    proposal_id: "p3".to_string(),
                    branch: "b3".to_string(),
                    recorded_at: Utc::now(),
                },
                EvolutionAuthorityEntry {
                    scope_class: "runtime_policy".to_string(),
                    authority_tier: "phase1_auto_merge".to_string(),
                    accepted: true,
                    merged: true,
                    durable_truth: true,
                    proposal_id: "p2".to_string(),
                    branch: "b2".to_string(),
                    recorded_at: Utc::now(),
                },
                EvolutionAuthorityEntry {
                    scope_class: "runtime_policy".to_string(),
                    authority_tier: "phase1_auto_merge".to_string(),
                    accepted: true,
                    merged: true,
                    durable_truth: true,
                    proposal_id: "p1".to_string(),
                    branch: "b1".to_string(),
                    recorded_at: Utc::now(),
                },
            ],
        })
        .expect("serialize authority"),
    )
    .expect("write authority");
    assert_eq!(
        compute_evolution_authority_tier(&output, "runtime_policy", "auto_merge"),
        "durable_auto_merge"
    );

    fs::write(
        output.join("evolution").join("authority-ledger.json"),
        serde_json::to_string_pretty(&EvolutionAuthorityLedger {
            entries: vec![
                EvolutionAuthorityEntry {
                    scope_class: "runtime_policy".to_string(),
                    authority_tier: "phase1_auto_merge".to_string(),
                    accepted: false,
                    merged: false,
                    durable_truth: false,
                    proposal_id: "p2".to_string(),
                    branch: "b2".to_string(),
                    recorded_at: Utc::now(),
                },
                EvolutionAuthorityEntry {
                    scope_class: "runtime_policy".to_string(),
                    authority_tier: "phase1_auto_merge".to_string(),
                    accepted: true,
                    merged: false,
                    durable_truth: false,
                    proposal_id: "p1".to_string(),
                    branch: "b1".to_string(),
                    recorded_at: Utc::now(),
                },
            ],
        })
        .expect("serialize authority"),
    )
    .expect("write authority");
    assert_eq!(
        compute_evolution_authority_tier(&output, "runtime_policy", "auto_merge"),
        "proposal_only"
    );

    fs::remove_dir_all(output).expect("cleanup authority output");
}

#[test]
fn classify_evolution_scope_biases_safe_self_evolution_changes_into_auto_merge_lanes() {
    let dir = std::env::temp_dir().join(format!(
        "memd-evolution-classifier-safe-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("create temp bundle");

    let mut report = test_experiment_report(&dir, true, false, 96, 100, Utc::now());
    report.composite.scenario = Some("self_evolution".to_string());
    report.improvement.final_changes =
        vec!["retune pass/fail gate for self-evolution proposals".to_string()];
    let scope = classify_evolution_scope(&report);
    assert_eq!(scope.scope_class, "runtime_policy");
    assert_eq!(scope.scope_gate, "auto_merge");

    report.improvement.final_changes =
        vec!["refine composite dimension calculation for self-evolution".to_string()];
    let scope = classify_evolution_scope(&report);
    assert_eq!(scope.scope_class, "low_risk_evaluation_code");
    assert_eq!(scope.scope_gate, "auto_merge");

    report.improvement.final_changes =
        vec!["adjust acceptance signal aggregation in proposal scoring".to_string()];
    let scope = classify_evolution_scope(&report);
    assert_eq!(scope.scope_class, "low_risk_evaluation_code");
    assert_eq!(scope.scope_gate, "auto_merge");

    fs::remove_dir_all(dir).expect("cleanup temp bundle");
}

#[test]
fn classify_evolution_scope_keeps_high_risk_changes_on_proposal_branch() {
    let dir = std::env::temp_dir().join(format!(
        "memd-evolution-classifier-risky-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("create temp bundle");

    let mut report = test_experiment_report(&dir, true, false, 96, 100, Utc::now());
    report.composite.scenario = Some("self_evolution".to_string());
    report.improvement.final_changes =
        vec!["change hive coordination protocol for task claims".to_string()];
    let scope = classify_evolution_scope(&report);
    assert_eq!(scope.scope_class, "coordination_semantics");
    assert_eq!(scope.scope_gate, "proposal_only");

    report.improvement.final_changes =
        vec!["update storage schema for evolution durability ledger".to_string()];
    let scope = classify_evolution_scope(&report);
    assert_eq!(scope.scope_class, "persistence_semantics");
    assert_eq!(scope.scope_gate, "proposal_only");

    fs::remove_dir_all(dir).expect("cleanup temp bundle");
}

#[test]
fn ui_commands_parse_artifact_and_map_modes() {
    let artifact_id = uuid::Uuid::new_v4().to_string();
    let cli = Cli::parse_from([
        "memd",
        "--base-url",
        "http://localhost:3000",
        "ui",
        "artifact",
        "--id",
        &artifact_id,
        "--json",
        "--follow",
    ]);

    assert_eq!(cli.base_url, "http://localhost:3000");
    match cli.command {
        Commands::Ui(args) => match args.mode {
            UiMode::Artifact(args) => {
                assert_eq!(args.id, artifact_id);
                assert!(args.json);
                assert!(args.follow);
            }
            _ => panic!("expected artifact mode"),
        },
        _ => panic!("expected ui command"),
    }

    let cli = Cli::parse_from(["memd", "ui", "map", "--json"]);
    match cli.command {
        Commands::Ui(args) => match args.mode {
            UiMode::Map(args) => {
                assert!(args.json);
                assert!(!args.follow);
            }
            _ => panic!("expected map mode"),
        },
        _ => panic!("expected ui command"),
    }
}
