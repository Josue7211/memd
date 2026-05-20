#[tokio::test]
async fn run_public_longmemeval_command_writes_artifacts_and_docs() {
    let dir = std::env::temp_dir().join(format!(
        "memd-public-benchmark-command-{}",
        uuid::Uuid::new_v4()
    ));
    let output = dir.join(".memd");
    let docs_root = dir.join("repo");
    fs::create_dir_all(&output).expect("create output dir");
    fs::create_dir_all(docs_root.join(".git")).expect("create git dir");

    let fixture = public_benchmark_fixture_path("longmemeval");
    let report = run_public_benchmark_command(&PublicBenchmarkArgs {
        dataset: "longmemeval".to_string(),
        mode: Some("raw".to_string()),
        retrieval_backend: None,
        rag_url: None,
        memd_url: None,
        top_k: Some(5),
        limit: Some(2),
        dataset_root: Some(fixture),
        reranker: None,
        write: false,
        json: false,
        community_standard: false,
        hypotheses_file: None,
        grader_model: None,
        full_eval: false,
        generator_model: None,
        sample: None,
        dry_run: false,
        dual: false,
        turn_diagnostics: false,
        all: false,
        out: output.clone(),
        ci: false,
        record: false,
        typed_ingest: None,
        distill_model: "gpt-5.4".to_string(),
        distill_budget_milli_usd: 100,
        distill_cache_dir: None,
        promotion_dry_run: false,
        compiler: "off".to_string(),
        depth_routing: "on".to_string(),
        max_depth_calls: 3,
        max_retrieval_tokens: 10_000,
        reasoning: "on".to_string(),
        max_reasoning_steps: 5,
        max_reasoning_tokens: 20_000,
        regenerate_report: false,
        regenerate_10star: false,
        allow_below_target: false,
    })
    .await
    .expect("run public benchmark");

    assert_eq!(report.manifest.benchmark_id, "longmemeval");
    assert_eq!(report.manifest.mode, "raw");
    assert_eq!(report.manifest.dataset_name, "LongMemEval");
    assert_eq!(report.item_count, 2);
    assert!(report.metrics.get("accuracy").copied().unwrap_or(0.0) > 0.0);

    let receipt = write_public_benchmark_run_artifacts(&output, &report).expect("write artifacts");
    assert!(receipt.manifest_path.exists());
    assert!(receipt.results_path.exists());
    assert!(receipt.results_jsonl_path.exists());
    assert!(receipt.report_path.exists());

    write_public_benchmark_docs(&docs_root, &output, &report).expect("write public benchmark docs");
    let docs = fs::read_to_string(public_benchmark_docs_path(&docs_root))
        .expect("read public benchmark docs");
    assert!(docs.contains("# memd public benchmark"));
    assert!(docs.contains("LongMemEval"));
    assert!(docs.contains("results"));
    assert!(docs.contains("## Target Inventory"));
    assert!(docs.contains("- longmemeval: implemented"));
    assert!(docs.contains("- locomo: implemented"));
    assert!(docs.contains("- convomem: implemented"));
    assert!(docs.contains("- membench: implemented"));
    // J3 contract: `write_public_benchmark_docs` no longer auto-overwrites
    // `PUBLIC_LEADERBOARD.md`. The leaderboard file is hand-curated per I3;
    // the runtime only writes the summary report above. See
    // `docs/handoff/2026-04-21-j3-complete-proxy-gap-deferred-next-k3.md`.
    assert!(
        !public_benchmark_leaderboard_docs_path(&docs_root).exists(),
        "leaderboard docs must not be auto-overwritten by the bench runtime"
    );

    fs::remove_dir_all(dir).expect("cleanup public benchmark command dir");
}

#[tokio::test]
async fn run_public_longmemeval_hybrid_command_sets_metadata() {
    let dir = std::env::temp_dir().join(format!(
        "memd-public-benchmark-hybrid-{}",
        uuid::Uuid::new_v4()
    ));
    let output = dir.join(".memd");
    fs::create_dir_all(&output).expect("create output dir");

    let fixture = public_benchmark_fixture_path("longmemeval");
    let report = run_public_benchmark_command(&PublicBenchmarkArgs {
        dataset: "longmemeval".to_string(),
        mode: Some("hybrid".to_string()),
        retrieval_backend: None,
        rag_url: None,
        memd_url: None,
        top_k: Some(5),
        limit: Some(1),
        dataset_root: Some(fixture),
        reranker: Some("test-reranker".to_string()),
        write: false,
        json: false,
        community_standard: false,
        hypotheses_file: None,
        grader_model: None,
        full_eval: false,
        generator_model: None,
        sample: None,
        dry_run: false,
        dual: false,
        turn_diagnostics: false,
        all: false,
        out: output.clone(),
        ci: false,
        record: false,
        typed_ingest: None,
        distill_model: "gpt-5.4".to_string(),
        distill_budget_milli_usd: 100,
        distill_cache_dir: None,
        promotion_dry_run: false,
        compiler: "off".to_string(),
        depth_routing: "on".to_string(),
        max_depth_calls: 3,
        max_retrieval_tokens: 10_000,
        reasoning: "on".to_string(),
        max_reasoning_steps: 5,
        max_reasoning_tokens: 20_000,
        regenerate_report: false,
        regenerate_10star: false,
        allow_below_target: false,
    })
    .await
    .expect("run hybrid public benchmark");

    assert_eq!(report.manifest.mode, "hybrid");
    assert_eq!(
        report.manifest.reranker_id.as_deref(),
        Some("test-reranker")
    );
    assert_eq!(
        report.manifest.reranker_provider.as_deref(),
        Some("declared")
    );
    assert_eq!(
        report.manifest.token_usage,
        Some(json!({
            "prompt_tokens": 0,
            "completion_tokens": 0,
            "reranker_tokens": 0,
        }))
    );
    assert_eq!(report.manifest.cost_estimate_usd, Some(0.0));
    assert_eq!(report.items.len(), 1);
    assert_eq!(report.items[0].claim_class, "raw");

    fs::remove_dir_all(dir).expect("cleanup public benchmark hybrid dir");
}

#[tokio::test]
async fn render_public_leaderboard_marks_fixture_backed_partial_parity() {
    let dir =
        std::env::temp_dir().join(format!("memd-public-leaderboard-{}", uuid::Uuid::new_v4()));
    let output = dir.join(".memd");
    fs::create_dir_all(&output).expect("create output dir");

    let fixture = public_benchmark_fixture_path("longmemeval");
    let report = run_public_benchmark_command(&PublicBenchmarkArgs {
        dataset: "longmemeval".to_string(),
        mode: Some("raw".to_string()),
        retrieval_backend: None,
        rag_url: None,
        memd_url: None,
        top_k: Some(5),
        limit: Some(2),
        dataset_root: Some(fixture),
        reranker: None,
        write: false,
        json: false,
        community_standard: false,
        hypotheses_file: None,
        grader_model: None,
        full_eval: false,
        generator_model: None,
        sample: None,
        dry_run: false,
        dual: false,
        turn_diagnostics: false,
        all: false,
        out: output.clone(),
        ci: false,
        record: false,
        typed_ingest: None,
        distill_model: "gpt-5.4".to_string(),
        distill_budget_milli_usd: 100,
        distill_cache_dir: None,
        promotion_dry_run: false,
        compiler: "off".to_string(),
        depth_routing: "on".to_string(),
        max_depth_calls: 3,
        max_retrieval_tokens: 10_000,
        reasoning: "on".to_string(),
        max_reasoning_steps: 5,
        max_reasoning_tokens: 20_000,
        regenerate_report: false,
        regenerate_10star: false,
        allow_below_target: false,
    })
    .await
    .expect("run public benchmark");

    let leaderboard_report =
        build_public_benchmark_leaderboard_report(&dir, &output, std::slice::from_ref(&report));
    let markdown = render_public_leaderboard(&leaderboard_report);
    assert!(markdown.contains("# memd public leaderboard"));
    assert!(markdown.contains("fixture-backed"));
    assert!(markdown.contains("fixture-only"));
    assert!(markdown.contains("MemPalace"));
    assert!(markdown.contains("Verification"));
    assert!(markdown.contains("Regression"));
    assert!(markdown.contains("Commit"));
    assert!(markdown.contains("Rerun"));
    assert!(markdown.contains("run mode is benchmark execution mode"));
    assert!(
        markdown.contains("implemented mini adapters: longmemeval, locomo, convomem, membench")
    );
    assert!(markdown.contains("session_recall_any@5 (retrieval diagnostic)"));
    assert!(markdown.contains("declared parity targets: longmemeval, locomo, convomem, membench"));

    fs::remove_dir_all(dir).expect("cleanup public leaderboard dir");
}

#[test]
fn build_public_leaderboard_prefers_local_mempalace_replay_artifacts() {
    let dir = std::env::temp_dir().join(format!(
        "memd-public-leaderboard-mempalace-replay-{}",
        uuid::Uuid::new_v4()
    ));
    let output = dir.join(".memd");
    fs::create_dir_all(output.join("benchmarks").join("baselines")).expect("create baselines dir");

    fs::write(
        default_baselines_path(&output),
        serde_json::to_string_pretty(&json!({
            "longmemeval": {
                "MemPalace": {
                    "accuracy": 98.4,
                    "source": "mempalace/benchmarks/BENCHMARKS.md",
                    "date": "2026-03-26",
                    "note": "published held-out baseline; local same-fixture replay pending"
                }
            }
        }))
        .expect("serialize published baselines"),
    )
    .expect("write published baselines");
    fs::write(
        default_mempalace_replays_path(&output),
        serde_json::to_string_pretty(&json!({
            "longmemeval": {
                "accuracy": 0.966,
                "source": ".memd/benchmarks/baselines/mempalace-replays/longmemeval/latest/summary.json",
                "note": "local same-fixture replay complete",
                "status": "replayed",
                "command": "python scripts/bench-mempalace.py --benchmark longmemeval",
                "artifact_path": ".memd/benchmarks/baselines/mempalace-replays/longmemeval/latest/"
            }
        }))
        .expect("serialize local replays"),
    )
    .expect("write local replays");

    let report = PublicBenchmarkRunReport {
        manifest: PublicBenchmarkManifest {
            benchmark_id: "longmemeval".to_string(),
            benchmark_version: "upstream".to_string(),
            dataset_name: "LongMemEval".to_string(),
            dataset_source_url:
                "https://huggingface.co/datasets/xiaowu0162/longmemeval-cleaned/resolve/main/longmemeval_s_cleaned.json"
                    .to_string(),
            dataset_local_path: ".memd/benchmarks/datasets/longmemeval/longmemeval_s_cleaned.json"
                .to_string(),
            dataset_checksum: "sha256:test".to_string(),
            dataset_split: "cleaned-small".to_string(),
            git_sha: Some("deadbeef".to_string()),
            dirty_worktree: false,
            run_timestamp: Utc::now(),
            mode: "raw".to_string(),
            top_k: 5,
            reranker_id: None,
            reranker_provider: None,
            limit: Some(2),
            runtime_settings: json!({"dataset_verification": "verified"}),
            hardware_summary: "cpu-only".to_string(),
            duration_ms: 1,
            token_usage: None,
            cost_estimate_usd: None,
        },
        metrics: BTreeMap::from([("session_recall_any@5".to_string(), 0.88)]),
        item_count: 2,
        failures: Vec::new(),
        items: Vec::new(),
    };

    let leaderboard =
        build_public_benchmark_leaderboard_report(&dir, &output, std::slice::from_ref(&report));
    let row = leaderboard.rows.first().expect("leaderboard row");
    assert_eq!(row.mempalace_score, Some(0.966));
    assert_eq!(row.mempalace_status, "replayed");
    assert_eq!(row.claim_class, "cross-replayed");
    assert_eq!(row.parity_status, "cross-replayed");
    assert!(row.notes.iter().any(|note| {
        note == "mempalace_command=python scripts/bench-mempalace.py --benchmark longmemeval"
    }));
    assert!(row.notes.iter().any(|note| {
        note == "mempalace_artifacts=.memd/benchmarks/baselines/mempalace-replays/longmemeval/latest/"
    }));

    fs::remove_dir_all(dir).expect("cleanup local replay dir");
}

#[tokio::test]
async fn write_public_benchmark_docs_aggregates_all_latest_runs() {
    let dir = std::env::temp_dir().join(format!(
        "memd-public-benchmark-suite-docs-{}",
        uuid::Uuid::new_v4()
    ));
    let output = dir.join(".memd");
    let docs_root = dir.join("repo");
    fs::create_dir_all(&output).expect("create output dir");
    fs::create_dir_all(docs_root.join(".git")).expect("create git dir");

    for dataset in ["longmemeval", "locomo", "convomem", "membench"] {
        let report = run_public_benchmark_command(&PublicBenchmarkArgs {
            dataset: dataset.to_string(),
            mode: Some("raw".to_string()),
            retrieval_backend: None,
            rag_url: None,
            memd_url: None,
            top_k: Some(5),
            limit: Some(2),
            dataset_root: Some(public_benchmark_fixture_path(dataset)),
            reranker: None,
            write: false,
            json: false,
            community_standard: false,
            hypotheses_file: None,
            grader_model: None,
            full_eval: false,
            generator_model: None,
            sample: None,
            dry_run: false,
            dual: false,
            turn_diagnostics: false,
            all: false,
            out: output.clone(),
            ci: false,
            record: false,
            typed_ingest: None,
            distill_model: "gpt-5.4".to_string(),
            distill_budget_milli_usd: 100,
            distill_cache_dir: None,
            promotion_dry_run: false,
            compiler: "off".to_string(),
            depth_routing: "on".to_string(),
            max_depth_calls: 3,
            max_retrieval_tokens: 10_000,
            reasoning: "on".to_string(),
            max_reasoning_steps: 5,
            max_reasoning_tokens: 20_000,
            regenerate_report: false,
            regenerate_10star: false,
            allow_below_target: false,
        })
        .await
        .expect("run public benchmark");
        write_public_benchmark_run_artifacts(&output, &report).expect("write public artifacts");
    }

    let latest_report = run_public_benchmark_command(&PublicBenchmarkArgs {
        dataset: "locomo".to_string(),
        mode: Some("hybrid".to_string()),
        retrieval_backend: None,
        rag_url: None,
        memd_url: None,
        top_k: Some(5),
        limit: Some(2),
        dataset_root: Some(public_benchmark_fixture_path("locomo")),
        reranker: Some("test-reranker".to_string()),
        write: false,
        json: false,
        community_standard: false,
        hypotheses_file: None,
        grader_model: None,
        full_eval: false,
        generator_model: None,
        sample: None,
        dry_run: false,
        dual: false,
        turn_diagnostics: false,
        all: false,
        out: output.clone(),
        ci: false,
        record: false,
        typed_ingest: None,
        distill_model: "gpt-5.4".to_string(),
        distill_budget_milli_usd: 100,
        distill_cache_dir: None,
        promotion_dry_run: false,
        compiler: "off".to_string(),
        depth_routing: "on".to_string(),
        max_depth_calls: 3,
        max_retrieval_tokens: 10_000,
        reasoning: "on".to_string(),
        max_reasoning_steps: 5,
        max_reasoning_tokens: 20_000,
        regenerate_report: false,
        regenerate_10star: false,
        allow_below_target: false,
    })
    .await
    .expect("run latest public benchmark");
    write_public_benchmark_run_artifacts(&output, &latest_report)
        .expect("write latest public artifacts");
    write_public_benchmark_docs(&docs_root, &output, &latest_report)
        .expect("write public benchmark docs");

    let docs = fs::read_to_string(public_benchmark_docs_path(&docs_root))
        .expect("read public benchmark docs");
    assert!(docs.contains("# memd public benchmark suite"));
    assert!(docs.contains("| LongMemEval |"));
    assert!(docs.contains("| LoCoMo |"));
    assert!(docs.contains("| ConvoMem |"));
    assert!(docs.contains("| MemBench |"));
    assert!(docs.contains("## Latest Run Detail: LoCoMo"));

    // J3 contract: `write_public_benchmark_docs` skips `PUBLIC_LEADERBOARD.md`.
    // Hand-curated per I3 (method cards + retraction log + gaming-audit rule).
    assert!(
        !public_benchmark_leaderboard_docs_path(&docs_root).exists(),
        "leaderboard docs must not be auto-overwritten by the bench runtime"
    );

    fs::remove_dir_all(dir).expect("cleanup public benchmark suite docs dir");
}

#[tokio::test]
async fn benchmark_public_all_write_refreshes_each_latest_artifact() {
    let dir = std::env::temp_dir().join(format!(
        "memd-public-benchmark-all-write-{}",
        uuid::Uuid::new_v4()
    ));
    let docs_root = dir.join("repo");
    let output = docs_root.join(".memd");
    fs::create_dir_all(docs_root.join(".git")).expect("create git dir");
    fs::create_dir_all(&output).expect("create output dir");

    let fixture_root = public_benchmark_fixture_path("longmemeval")
        .parent()
        .expect("fixture path has parent")
        .to_path_buf();

    run_benchmark_command(
        &BenchmarkArgs {
            output: output.clone(),
            write: false,
            summary: false,
            subcommand: Some(BenchmarkSubcommand::Public(PublicBenchmarkArgs {
                dataset: String::new(),
                mode: Some("raw".to_string()),
                retrieval_backend: None,
                rag_url: None,
                memd_url: None,
                top_k: Some(5),
                limit: Some(2),
                dataset_root: Some(fixture_root),
                reranker: None,
                write: true,
                json: false,
                community_standard: false,
                hypotheses_file: None,
                grader_model: None,
                full_eval: false,
                generator_model: None,
                sample: None,
                dry_run: false,
                dual: false,
                turn_diagnostics: false,
                all: true,
                out: output.clone(),
                ci: false,
                record: false,
                typed_ingest: None,
                distill_model: "gpt-5.4".to_string(),
                distill_budget_milli_usd: 100,
                distill_cache_dir: None,
                promotion_dry_run: false,
                compiler: "off".to_string(),
                depth_routing: "on".to_string(),
                max_depth_calls: 3,
                max_retrieval_tokens: 10_000,
                reasoning: "on".to_string(),
                max_reasoning_steps: 5,
                max_reasoning_tokens: 20_000,
                regenerate_report: false,
                regenerate_10star: false,
                allow_below_target: false,
            })),
        },
        "http://127.0.0.1:8787",
    )
    .await
    .expect("run benchmark public --all --write");

    for dataset in ["convomem", "membench"] {
        assert!(
            public_benchmark_manifest_json_path(&output, dataset).exists(),
            "missing manifest for {dataset}"
        );
        assert!(
            public_benchmark_results_json_path(&output, dataset).exists(),
            "missing results for {dataset}"
        );
        assert!(
            public_benchmark_report_md_path(&output, dataset).exists(),
            "missing report for {dataset}"
        );
    }

    // J3 contract: runtime skips leaderboard writes.
    assert!(
        !public_benchmark_leaderboard_docs_path(&docs_root).exists(),
        "leaderboard docs must not be auto-overwritten by the bench runtime"
    );

    fs::remove_dir_all(dir).expect("cleanup benchmark public --all dir");
}

#[test]
fn primary_metric_returns_full_eval_labels_when_runtime_settings_has_full_eval() {
    let make_report = |benchmark_id: &str, full_eval: bool, metric_key: &str| {
        let mut metrics = BTreeMap::new();
        metrics.insert(metric_key.to_string(), 0.75);
        PublicBenchmarkRunReport {
            manifest: PublicBenchmarkManifest {
                benchmark_id: benchmark_id.to_string(),
                benchmark_version: "upstream".to_string(),
                dataset_name: benchmark_id.to_string(),
                dataset_source_url: String::new(),
                dataset_local_path: String::new(),
                dataset_checksum: String::new(),
                dataset_split: String::new(),
                git_sha: None,
                dirty_worktree: false,
                run_timestamp: Utc::now(),
                mode: "raw".to_string(),
                top_k: 5,
                reranker_id: None,
                reranker_provider: None,
                limit: Some(10),
                runtime_settings: json!({"full_eval": full_eval}),
                hardware_summary: String::new(),
                duration_ms: 0,
                token_usage: None,
                cost_estimate_usd: None,
            },
            metrics,
            item_count: 10,
            failures: Vec::new(),
            items: Vec::new(),
        }
    };

    // full_eval=true should yield industry-standard labels
    let (label, val) =
        public_benchmark_primary_metric(&make_report("longmemeval", true, "accuracy"));
    assert!(
        label.contains("LLM-judge"),
        "longmemeval full_eval label: {label}"
    );
    assert!((val - 0.75).abs() < 0.001);

    let (label, _) = public_benchmark_primary_metric(&make_report("locomo", true, "f1"));
    assert!(
        label.contains("token-level"),
        "locomo full_eval label: {label}"
    );

    let (label, _) = public_benchmark_primary_metric(&make_report("membench", true, "mc_accuracy"));
    assert!(
        label.contains("MC accuracy"),
        "membench full_eval label: {label}"
    );

    // full_eval=false should yield retrieval diagnostic labels
    let (label, _) =
        public_benchmark_primary_metric(&make_report("longmemeval", false, "session_recall_any@5"));
    assert!(
        label.contains("retrieval diagnostic"),
        "longmemeval retrieval label: {label}"
    );
}
