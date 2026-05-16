use super::*;

#[tokio::test]
async fn run_scenario_command_writes_artifacts_and_scores_with_mocked_backend() {
    let dir = std::env::temp_dir().join(format!("memd-scenario-command-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("create scenario temp bundle");
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

    let base_url = spawn_mock_memory_server().await;
    let report = run_scenario_command(
        &ScenarioArgs {
            output: dir.clone(),
            scenario: Some("bundle_health".to_string()),
            write: false,
            summary: false,
        },
        &base_url,
    )
    .await
    .expect("run scenario command");

    assert_eq!(report.scenario, "bundle_health");
    assert!(report.passed_checks >= 1);
    assert_eq!(report.failed_checks, 0);
    assert!(report.score >= 28);
    assert!(report.max_score >= report.score);
    assert!(
        report
            .checks
            .iter()
            .any(|check| check.name == "runtime_config" && check.status == "pass")
    );
    assert!(!report.checks.is_empty());

    write_scenario_artifacts(&dir, &report).expect("write scenario artifacts");
    let scenario_dir = dir.join("scenarios");
    let latest_json = scenario_dir.join("latest.json");
    let latest_markdown = scenario_dir.join("latest.md");
    assert!(latest_json.exists());
    assert!(latest_markdown.exists());

    let latest = fs::read_to_string(&latest_json).expect("read latest.json");
    let parsed: ScenarioReport = serde_json::from_str(&latest).expect("parse latest scenario json");
    assert_eq!(parsed.scenario, "bundle_health");
    let markdown = fs::read_to_string(&latest_markdown).expect("read latest.md");
    assert!(markdown.contains("# memd scenario report: bundle_health"));
    let entries = fs::read_dir(&scenario_dir)
        .expect("read scenario dir")
        .collect::<Result<Vec<_>, _>>()
        .expect("scenario dir entries");
    assert!(entries.iter().any(|entry| {
        entry
            .path()
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| name.ends_with(".json") && name != "latest.json")
    }));

    fs::remove_dir_all(dir).expect("cleanup scenario temp bundle");
}

#[tokio::test]
async fn run_scenario_command_supports_named_v6_workflows() {
    let dir = std::env::temp_dir().join(format!(
        "memd-scenario-command-workflows-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("create scenario temp bundle");
    fs::write(
        dir.join("config.json"),
        r#"{
  "project": "demo",
  "agent": "codex",
  "session": "session-alpha",
  "route": "auto",
  "intent": "general"
}
"#,
    )
    .expect("write config");

    let base_url = spawn_mock_memory_server().await;
    let scenarios = [
        "bundle_health",
        "resume_after_pause",
        "handoff",
        "workspace_retrieval",
        "stale_session_recovery",
        "coworking",
    ];
    for scenario in scenarios {
        let report = run_scenario_command(
            &ScenarioArgs {
                output: dir.clone(),
                scenario: Some(scenario.to_string()),
                write: false,
                summary: false,
            },
            &base_url,
        )
        .await
        .expect("run scenario command");

        assert_eq!(report.scenario, scenario);
        assert_eq!(report.failed_checks, 0);
        assert!(!report.checks.is_empty());
        assert!(report.max_score > 0);
    }

    fs::remove_dir_all(dir).expect("cleanup scenario temp bundle");
}

#[tokio::test]
async fn run_scenario_command_rejects_unknown_scenario() {
    let dir = std::env::temp_dir().join(format!(
        "memd-scenario-command-unknown-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("create scenario temp bundle");
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

    let base_url = spawn_mock_memory_server().await;
    let result = run_scenario_command(
        &ScenarioArgs {
            output: dir.clone(),
            scenario: Some("not_a_real_scenario".to_string()),
            write: false,
            summary: false,
        },
        &base_url,
    )
    .await;

    assert!(result.is_err());
    assert!(
        result
            .err()
            .expect("scenario should be rejected")
            .to_string()
            .contains("supported")
    );

    fs::remove_dir_all(dir).expect("cleanup scenario temp bundle");
}

#[tokio::test]
async fn run_composite_command_combines_saved_eval_and_scenario_reports() {
    let dir = std::env::temp_dir().join(format!("memd-composite-command-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("create composite temp bundle");
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

    let eval = high_scoring_eval(&dir);
    write_bundle_eval_artifacts(&dir, &eval).expect("write eval artifacts");

    let scenario = high_scoring_scenario(&dir);
    write_scenario_artifacts(&dir, &scenario).expect("write scenario artifacts");

    let base_url = spawn_mock_memory_server().await;
    let composite = run_composite_command(
        &CompositeArgs {
            output: dir.clone(),
            scenario: Some("bundle_health".to_string()),
            write: false,
            summary: false,
        },
        &base_url,
    )
    .await
    .expect("run composite");

    assert_eq!(composite.max_score, 100);
    assert!(composite.score > 0);
    assert!(
        composite
            .dimensions
            .iter()
            .any(|dimension| dimension.name == "correctness")
    );
    assert!(
        composite
            .gates
            .iter()
            .any(|gate| gate.name == "hard_correctness")
    );
    assert!(composite.gates.iter().any(|gate| gate.name == "acceptance"));

    write_composite_artifacts(&dir, &composite).expect("write composite artifacts");
    let composite_dir = dir.join("composite");
    assert!(composite_dir.join("latest.json").exists());
    assert!(composite_dir.join("latest.md").exists());

    fs::remove_dir_all(dir).expect("cleanup composite temp bundle");
}

#[test]
fn cli_parses_benchmark_command() {
    let cli = Cli::try_parse_from(["memd", "benchmark", "--output", ".memd", "--summary"])
        .expect("benchmark command should parse");

    match cli.command {
        Commands::Benchmark(args) => {
            assert_eq!(args.output, PathBuf::from(".memd"));
            assert!(args.summary);
            assert!(args.subcommand.is_none());
        }
        other => panic!("expected benchmark command, got {other:?}"),
    }
}

#[test]
fn cli_parses_public_longmemeval_benchmark_command() {
    let cli = Cli::try_parse_from([
        "memd",
        "benchmark",
        "public",
        "--mode",
        "raw",
        "--limit",
        "20",
        "--output",
        ".memd",
        "longmemeval",
    ])
    .expect("public benchmark command should parse");

    match cli.command {
        Commands::Benchmark(args) => match args.subcommand {
            Some(BenchmarkSubcommand::Public(public_args)) => {
                assert_eq!(public_args.dataset, "longmemeval");
                assert_eq!(public_args.mode.as_deref(), Some("raw"));
                assert_eq!(public_args.limit, Some(20));
                assert_eq!(public_args.out, PathBuf::from(".memd"));
                assert!(!public_args.write);
                assert!(!public_args.json);
            }
            other => panic!("expected public benchmark subcommand, got {other:?}"),
        },
        other => panic!("expected benchmark command, got {other:?}"),
    }
}

#[test]
fn cli_parses_public_longmemeval_sidecar_backend() {
    let cli = Cli::try_parse_from([
        "memd",
        "benchmark",
        "public",
        "--mode",
        "hybrid",
        "--retrieval-backend",
        "sidecar",
        "--rag-url",
        "http://127.0.0.1:9981",
        "--output",
        ".memd",
        "longmemeval",
    ])
    .expect("public benchmark sidecar command should parse");

    match cli.command {
        Commands::Benchmark(args) => match args.subcommand {
            Some(BenchmarkSubcommand::Public(public_args)) => {
                assert_eq!(public_args.dataset, "longmemeval");
                assert_eq!(public_args.mode.as_deref(), Some("hybrid"));
                assert_eq!(public_args.retrieval_backend.as_deref(), Some("sidecar"));
                assert_eq!(
                    public_args.rag_url.as_deref(),
                    Some("http://127.0.0.1:9981")
                );
            }
            other => panic!("expected public benchmark subcommand, got {other:?}"),
        },
        other => panic!("expected benchmark command, got {other:?}"),
    }
}

#[test]
fn cli_parses_public_longmemeval_dual_command() {
    let cli = Cli::try_parse_from([
        "memd",
        "benchmark",
        "public",
        "--mode",
        "raw",
        "--dual",
        "--output",
        ".memd",
        "longmemeval",
    ])
    .expect("public benchmark dual command should parse");

    match cli.command {
        Commands::Benchmark(args) => match args.subcommand {
            Some(BenchmarkSubcommand::Public(public_args)) => {
                assert_eq!(public_args.dataset, "longmemeval");
                assert_eq!(public_args.mode.as_deref(), Some("raw"));
                assert!(public_args.dual);
            }
            other => panic!("expected public benchmark subcommand, got {other:?}"),
        },
        other => panic!("expected benchmark command, got {other:?}"),
    }
}

#[test]
fn cli_parses_public_longmemeval_community_standard_command() {
    let cli = Cli::try_parse_from([
        "memd",
        "benchmark",
        "public",
        "--community-standard",
        "--hypotheses-file",
        "hyp.jsonl",
        "--grader-model",
        "gpt-4o",
        "--output",
        ".memd",
        "longmemeval",
    ])
    .expect("community-standard benchmark command should parse");

    match cli.command {
        Commands::Benchmark(args) => match args.subcommand {
            Some(BenchmarkSubcommand::Public(public_args)) => {
                assert_eq!(public_args.dataset, "longmemeval");
                assert!(public_args.community_standard);
                assert_eq!(
                    public_args.hypotheses_file,
                    Some(PathBuf::from("hyp.jsonl"))
                );
                assert_eq!(public_args.grader_model.as_deref(), Some("gpt-4o"));
            }
            other => panic!("expected public benchmark subcommand, got {other:?}"),
        },
        other => panic!("expected benchmark command, got {other:?}"),
    }
}

#[test]
fn public_benchmark_paths_default_under_memd_benchmarks() {
    let output = PathBuf::from(".memd");
    assert_eq!(
        public_benchmark_dataset_cache_dir(&output),
        PathBuf::from(".memd/benchmarks/datasets")
    );
    assert_eq!(
        public_benchmark_dataset_entry_dir(&output, "longmemeval"),
        PathBuf::from(".memd/benchmarks/datasets/longmemeval")
    );
    assert_eq!(
        public_benchmark_dataset_cache_path(&output, "longmemeval", "longmemeval_s_cleaned.json"),
        PathBuf::from(".memd/benchmarks/datasets/longmemeval/longmemeval_s_cleaned.json")
    );
    assert_eq!(
        public_benchmark_runs_dir(&output),
        PathBuf::from(".memd/benchmarks/public")
    );
    assert_eq!(
        public_benchmark_run_artifacts_dir(&output, "longmemeval"),
        PathBuf::from(".memd/benchmarks/public/longmemeval/latest")
    );
}

#[test]
fn supported_public_benchmark_ids_lists_all_mem_palace_targets() {
    assert_eq!(
        supported_public_benchmark_ids(),
        &["longmemeval", "locomo", "convomem", "membench"]
    );
}

#[test]
fn public_benchmark_source_catalog_pins_longmemeval_download() {
    let source = public_benchmark_dataset_source("longmemeval").expect("catalog entry");
    assert_eq!(source.benchmark_id, "longmemeval");
    assert_eq!(source.access_mode, "auto-download");
    assert!(
        source
            .source_url
            .is_some_and(|url| url.ends_with("longmemeval_s_cleaned.json"))
    );
    assert_eq!(
        source.expected_checksum,
        Some("sha256:d6f21ea9d60a0d56f34a05b609c79c88a451d2ae03597821ea3d5a9678c3a442")
    );
    assert_eq!(source.split, "cleaned-small");
}

#[test]
fn public_benchmark_source_catalog_pins_locomo_download() {
    let source = public_benchmark_dataset_source("locomo").expect("catalog entry");
    assert_eq!(source.benchmark_id, "locomo");
    assert_eq!(source.access_mode, "auto-download");
    assert!(
        source
            .source_url
            .is_some_and(|url| url.contains("/snap-research/locomo/"))
    );
    assert_eq!(
        source.expected_checksum,
        Some("sha256:79fa87e90f04081343b8c8debecb80a9a6842b76a7aa537dc9fdf651ea698ff4")
    );
    assert_eq!(source.split, "locomo10");
}

#[test]
fn public_benchmark_source_catalog_pins_convomem_download() {
    let source = public_benchmark_dataset_source("convomem").expect("catalog entry");
    assert_eq!(source.benchmark_id, "convomem");
    assert_eq!(source.access_mode, "auto-download");
    assert!(
        source.source_url.is_some_and(
            |url| url.contains("huggingface.co/datasets/Salesforce/ConvoMem/tree/main")
        )
    );
    assert_eq!(source.default_filename, "convomem-evidence-sample.json");
    assert_eq!(
        source.expected_checksum,
        Some("sha256:dead92689c44ac5a3b66c0c7980166c8fc8d9b16a9cedb2e1c2f7981b6e6f094")
    );
    assert_eq!(source.split, "evidence-sample");
}

#[test]
fn public_benchmark_source_catalog_pins_membench_download() {
    let source = public_benchmark_dataset_source("membench").expect("catalog entry");
    assert_eq!(source.benchmark_id, "membench");
    assert_eq!(source.access_mode, "auto-download");
    assert!(
        source
            .source_url
            .is_some_and(|url| url.contains("/import-myself/Membench/"))
    );
    assert_eq!(source.default_filename, "membench-firstagent.json");
    assert_eq!(
        source.expected_checksum,
        Some("sha256:54bde8259c10ee1cfe5ff16f35a8a25ca9ad5d79e162e0b3a43034ed64115e5a")
    );
    assert_eq!(source.split, "FirstAgent");
}

#[test]
fn public_benchmark_cached_convomem_fixture_detects_missing_message_evidence_ids() {
    let dir = std::env::temp_dir().join(format!(
        "memd-public-benchmark-convomem-stale-cache-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("create stale cache dir");
    let stale_path = dir.join("convomem-stale.json");
    fs::write(
        &stale_path,
        serde_json::to_string_pretty(&PublicBenchmarkDatasetFixture {
            benchmark_id: "convomem".to_string(),
            benchmark_name: "ConvoMem".to_string(),
            version: "upstream".to_string(),
            split: "evidence-sample".to_string(),
            description: "test fixture".to_string(),
            items: vec![PublicBenchmarkDatasetFixtureItem {
                item_id: "item-1".to_string(),
                question_id: "Personal Life::0".to_string(),
                query: "What did Alex cook?".to_string(),
                claim_class: "retrieval".to_string(),
                gold_answer: "pho".to_string(),
                metadata: json!({
                    "conversations": [],
                    "message_evidences": [{"speaker": "User", "text": "I made pho."}]
                }),
            }],
        })
        .expect("serialize stale fixture"),
    )
    .expect("write stale fixture");
    assert!(
        public_benchmark_cached_dataset_is_stale("convomem", &stale_path)
            .expect("detect stale convomem cache")
    );

    fs::write(
        &stale_path,
        serde_json::to_string_pretty(&PublicBenchmarkDatasetFixture {
            benchmark_id: "convomem".to_string(),
            benchmark_name: "ConvoMem".to_string(),
            version: "upstream".to_string(),
            split: "evidence-sample".to_string(),
            description: "test fixture".to_string(),
            items: vec![PublicBenchmarkDatasetFixtureItem {
                item_id: "item-1".to_string(),
                question_id: "Personal Life::0".to_string(),
                query: "What did Alex cook?".to_string(),
                claim_class: "retrieval".to_string(),
                gold_answer: "pho".to_string(),
                metadata: json!({
                    "conversations": [],
                    "message_evidences": [{"speaker": "User", "text": "I made pho."}],
                    "message_evidence_match_version": 3,
                    "message_evidence_ids": ["conv-0::msg:0"]
                }),
            }],
        })
        .expect("serialize fresh fixture"),
    )
    .expect("write fresh fixture");
    assert!(
        !public_benchmark_cached_dataset_is_stale("convomem", &stale_path)
            .expect("detect fresh convomem cache")
    );

    fs::remove_dir_all(dir).expect("cleanup stale cache dir");
}

#[tokio::test]
async fn resolve_public_benchmark_dataset_rejects_unknown_sources() {
    let dir = std::env::temp_dir().join(format!(
        "memd-public-benchmark-manual-required-{}",
        uuid::Uuid::new_v4()
    ));
    let output = dir.join(".memd");
    fs::create_dir_all(&output).expect("create output dir");

    let error = resolve_public_benchmark_dataset(&PublicBenchmarkArgs {
        dataset: "unknown-benchmark".to_string(),
        mode: Some("raw".to_string()),
        retrieval_backend: None,
        rag_url: None,
        memd_url: None,
        top_k: Some(5),
        limit: Some(2),
        dataset_root: None,
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
    .expect_err("unknown benchmark should be rejected");
    assert!(
        error
            .to_string()
            .contains("no public benchmark dataset source is registered")
    );

    fs::remove_dir_all(dir).expect("cleanup manual-required dir");
}

#[tokio::test]
async fn run_public_longmemeval_community_standard_requires_hypotheses_file() {
    let dir = std::env::temp_dir().join(format!(
        "memd-public-benchmark-longmemeval-community-standard-{}",
        uuid::Uuid::new_v4()
    ));
    let output = dir.join(".memd");
    fs::create_dir_all(&output).expect("create output dir");

    let fixture = public_benchmark_fixture_path("longmemeval");
    let error = run_public_benchmark_command(&PublicBenchmarkArgs {
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
        out: output,
        community_standard: true,
        hypotheses_file: None,
        grader_model: None,
        full_eval: false,
        generator_model: None,
        sample: None,
        dry_run: false,
        dual: false,
        turn_diagnostics: false,
        all: false,
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
    .expect_err("community-standard longmemeval should require hypotheses");

    assert!(
        error
            .to_string()
            .contains("community-standard longmemeval requires --hypotheses-file")
    );

    fs::remove_dir_all(dir).expect("cleanup temp dir");
}

#[test]
fn validate_rejects_empty_dataset_without_all_flag() {
    let args = PublicBenchmarkArgs {
        dataset: String::new(),
        mode: None,
        retrieval_backend: None,
        rag_url: None,
        memd_url: None,
        top_k: None,
        limit: None,
        dataset_root: None,
        reranker: None,
        write: false,
        json: false,
        out: PathBuf::from("/tmp"),
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
    };
    let err = validate_public_benchmark_args(&args).expect_err("should reject empty dataset");
    assert!(err.to_string().contains("dataset is required"));
}

#[test]
fn validate_accepts_empty_dataset_with_all_flag() {
    let args = PublicBenchmarkArgs {
        dataset: String::new(),
        mode: None,
        retrieval_backend: None,
        rag_url: None,
        memd_url: None,
        top_k: None,
        limit: None,
        dataset_root: None,
        reranker: None,
        write: false,
        json: false,
        out: PathBuf::from("/tmp"),
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
    };
    validate_public_benchmark_args(&args).expect("--all should accept empty dataset");
}

#[test]
fn write_public_benchmark_dataset_cache_metadata_roundtrips_json() {
    let dir = std::env::temp_dir().join(format!(
        "memd-public-benchmark-cache-metadata-{}",
        uuid::Uuid::new_v4()
    ));
    let output = dir.join(".memd");
    fs::create_dir_all(&output).expect("create output dir");

    let metadata = PublicBenchmarkDatasetCacheMetadata {
            benchmark_id: "longmemeval".to_string(),
            source_url: "https://huggingface.co/datasets/xiaowu0162/longmemeval-cleaned/resolve/main/longmemeval_s_cleaned.json".to_string(),
            local_path: output
                .join("benchmarks")
                .join("datasets")
                .join("longmemeval")
                .join("longmemeval_s_cleaned.json")
                .display()
                .to_string(),
            checksum: "sha256:abc123".to_string(),
            expected_checksum: Some("sha256:abc123".to_string()),
            verification_status: "verified".to_string(),
            fetched_at: Utc::now(),
            bytes: 123,
        };

    let path = write_public_benchmark_dataset_cache_metadata(&output, &metadata)
        .expect("write cache metadata");
    assert_eq!(
        path,
        public_benchmark_dataset_cache_metadata_path(&output, "longmemeval")
    );
    let contents = fs::read_to_string(&path).expect("read cache metadata");
    let parsed: PublicBenchmarkDatasetCacheMetadata =
        serde_json::from_str(&contents).expect("parse cache metadata");
    assert_eq!(parsed.benchmark_id, "longmemeval");
    assert_eq!(parsed.verification_status, "verified");
    assert_eq!(parsed.bytes, 123);

    fs::remove_dir_all(dir).expect("cleanup cache metadata dir");
}

#[test]
fn load_public_benchmark_dataset_normalizes_longmemeval_array_format() {
    let dir = std::env::temp_dir().join(format!(
        "memd-public-benchmark-longmemeval-normalize-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("create normalize dir");
    let path = dir.join("longmemeval_s_cleaned.json");
    fs::write(
            &path,
            serde_json::to_string_pretty(&json!([
                {
                    "question_id": "q1",
                    "question_type": "temporal-reasoning",
                    "question": "What happened first?",
                    "answer": "the gps failed",
                    "question_date": "2023/04/10",
                    "haystack_dates": ["2023/04/10"],
                    "haystack_session_ids": ["s1"],
                    "answer_session_ids": ["s1"],
                    "haystack_sessions": [[
                        {"role": "user", "content": "The GPS failed after service.", "has_answer": true},
                        {"role": "assistant", "content": "That sounds annoying.", "has_answer": false}
                    ]]
                }
            ]))
            .expect("serialize synthetic longmemeval"),
        )
        .expect("write synthetic longmemeval");

    let dataset = load_public_benchmark_dataset("longmemeval", &path).expect("normalize dataset");
    assert_eq!(dataset.benchmark_id, "longmemeval");
    assert_eq!(dataset.version, "upstream");
    assert_eq!(dataset.items.len(), 1);
    assert_eq!(dataset.items[0].item_id, "q1");
    assert_eq!(dataset.items[0].gold_answer, "the gps failed");
    assert_eq!(dataset.items[0].claim_class, "raw");
    assert_eq!(
        dataset.items[0]
            .metadata
            .get("answer_session_ids")
            .and_then(JsonValue::as_array)
            .map(Vec::len),
        Some(1)
    );
    assert!(
        dataset.items[0]
            .metadata
            .get("haystack_text")
            .and_then(JsonValue::as_str)
            .is_some_and(|text| text.contains("GPS failed after service"))
    );

    fs::remove_dir_all(dir).expect("cleanup normalize dir");
}

#[test]
fn longmemeval_corpus_builders_skip_blank_user_turns() {
    let item = PublicBenchmarkDatasetFixtureItem {
        item_id: "q-blank".to_string(),
        question_id: "q-blank".to_string(),
        query: "What happened?".to_string(),
        claim_class: "raw".to_string(),
        gold_answer: "Something".to_string(),
        metadata: json!({
            "haystack_session_ids": ["s1", "s2"],
            "haystack_dates": ["2023-01-01", "2023-01-02"],
            "haystack_sessions": [
                [
                    {"role": "user", "content": "   "},
                    {"role": "assistant", "content": "ignored"}
                ],
                [
                    {"role": "user", "content": " first fact "},
                    {"role": "user", "content": ""},
                    {"role": "user", "content": "second fact"}
                ]
            ]
        }),
    };

    let (session_corpus, session_ids, _) = build_longmemeval_session_corpus(&item);
    assert_eq!(
        session_corpus,
        vec!["assistant: ignored", "user: first fact\nuser: second fact"]
    );
    assert_eq!(session_ids, vec!["s1", "s2"]);

    let (turn_corpus, turn_ids, _) = build_longmemeval_turn_corpus(&item);
    assert_eq!(turn_corpus, vec!["first fact", "second fact"]);
    assert_eq!(turn_ids, vec!["s2_turn_0", "s2_turn_1"]);
}

#[test]
fn longmemeval_bench_namespace_reuses_identical_corpus() {
    let item_a = PublicBenchmarkDatasetFixtureItem {
        item_id: "q-a".to_string(),
        question_id: "q-a".to_string(),
        query: "What happened first?".to_string(),
        claim_class: "raw".to_string(),
        gold_answer: "first fact".to_string(),
        metadata: json!({
            "haystack_session_ids": ["s1"],
            "haystack_dates": ["2023-01-01"],
            "haystack_sessions": [[
                {"role": "user", "content": "first fact"},
                {"role": "assistant", "content": "ack"},
                {"role": "user", "content": "second fact"}
            ]]
        }),
    };
    let item_b = PublicBenchmarkDatasetFixtureItem {
        item_id: "q-b".to_string(),
        question_id: "q-b".to_string(),
        query: "What happened second?".to_string(),
        claim_class: "raw".to_string(),
        gold_answer: "second fact".to_string(),
        metadata: item_a.metadata.clone(),
    };

    let (session_corpus_a, session_ids_a, _) = build_longmemeval_session_corpus(&item_a);
    let (session_corpus_b, session_ids_b, _) = build_longmemeval_session_corpus(&item_b);
    assert_eq!(session_corpus_a, session_corpus_b);
    assert_eq!(session_ids_a, session_ids_b);
    assert_eq!(
        longmemeval_bench_namespace("session", &session_ids_a, &session_corpus_a),
        longmemeval_bench_namespace("session", &session_ids_b, &session_corpus_b)
    );

    let (turn_corpus_a, turn_ids_a, _) = build_longmemeval_turn_corpus(&item_a);
    let (turn_corpus_b, turn_ids_b, _) = build_longmemeval_turn_corpus(&item_b);
    assert_eq!(turn_corpus_a, turn_corpus_b);
    assert_eq!(turn_ids_a, turn_ids_b);
    assert_eq!(
        longmemeval_bench_namespace("turn", &turn_ids_a, &turn_corpus_a),
        longmemeval_bench_namespace("turn", &turn_ids_b, &turn_corpus_b)
    );
}

#[test]
fn public_benchmark_namespace_reuses_same_corpus_across_questions() {
    let corpus_ids = vec!["D1:1".to_string(), "D1:2".to_string()];
    let corpus = vec![
        "Alice: The deploy key lives in 1Password.".to_string(),
        "Bob: The rollback command is memd rollback --last-good.".to_string(),
    ];

    assert_eq!(
        bench_item_namespace("locomo", "q-1", &corpus_ids, &corpus),
        bench_item_namespace("locomo", "q-2", &corpus_ids, &corpus),
        "same corpus should be ingested once and reused across benchmark questions"
    );

    let changed_corpus = vec![
        "Alice: The deploy key lives in 1Password.".to_string(),
        "Bob: The rollback command is memd rollback --previous.".to_string(),
    ];
    assert_ne!(
        bench_item_namespace("locomo", "q-1", &corpus_ids, &corpus),
        bench_item_namespace("locomo", "q-1", &corpus_ids, &changed_corpus),
        "different corpora must keep separate namespaces"
    );
}

#[test]
fn longmemeval_session_corpus_keeps_assistant_turns_with_role_labels() {
    let item = PublicBenchmarkDatasetFixtureItem {
        item_id: "q-assistant".to_string(),
        question_id: "q-assistant".to_string(),
        query: "What did you tell me before?".to_string(),
        claim_class: "raw".to_string(),
        gold_answer: "assistant fact".to_string(),
        metadata: json!({
            "haystack_session_ids": ["s1"],
            "haystack_dates": ["2023-01-01"],
            "haystack_sessions": [[
                {"role": "user", "content": "please remind me later"},
                {"role": "assistant", "content": "the answer is assistant fact"}
            ]]
        }),
    };

    let (session_corpus, session_ids, _) = build_longmemeval_session_corpus(&item);
    assert_eq!(
        session_corpus,
        vec!["user: please remind me later\nassistant: the answer is assistant fact"]
    );
    assert_eq!(session_ids, vec!["s1"]);
}

#[test]
fn load_public_benchmark_dataset_normalizes_locomo_array_format() {
    let dir = std::env::temp_dir().join(format!(
        "memd-public-benchmark-locomo-normalize-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("create normalize dir");
    let path = dir.join("locomo10.json");
    fs::write(
            &path,
            serde_json::to_string_pretty(&json!([
                {
                    "sample_id": "sample-001",
                    "conversation": {
                        "speaker_a": "Caroline",
                        "speaker_b": "Mel",
                        "session_1_date_time": "2023-05-07",
                        "session_1": [
                            {"speaker": "Caroline", "dia_id": "D1:1", "text": "I went to the LGBTQ support group on May 7."},
                            {"speaker": "Mel", "dia_id": "D1:2", "text": "That sounds meaningful."}
                        ]
                    },
                    "session_summary": {
                        "session_1_summary": "Caroline discussed attending a support group."
                    },
                    "observation": {
                        "session_1_observation": {
                            "Caroline": [["Caroline attended the support group on 7 May 2023.", "D1:1"]]
                        }
                    },
                    "event_summary": {
                        "events_session_1": {
                            "Caroline": ["Caroline attended a support group."],
                            "date": "7 May, 2023"
                        }
                    },
                    "qa": [
                        {
                            "question": "When did Caroline go to the LGBTQ support group?",
                            "answer": "7 May 2023",
                            "evidence": ["D1:1"],
                            "category": 2
                        }
                    ]
                }
            ]))
            .expect("serialize synthetic locomo"),
        )
        .expect("write synthetic locomo");

    let dataset = load_public_benchmark_dataset("locomo", &path).expect("normalize dataset");
    assert_eq!(dataset.benchmark_id, "locomo");
    assert_eq!(dataset.version, "upstream");
    assert_eq!(dataset.items.len(), 1);
    assert_eq!(dataset.items[0].item_id, "sample-001::0");
    assert_eq!(
        dataset.items[0].query,
        "When did Caroline go to the LGBTQ support group?"
    );
    assert_eq!(dataset.items[0].gold_answer, "7 May 2023");
    assert_eq!(dataset.items[0].claim_class, "raw");
    assert_eq!(
        dataset.items[0]
            .metadata
            .get("category_name")
            .and_then(JsonValue::as_str),
        Some("Temporal")
    );
    assert!(
        dataset.items[0]
            .metadata
            .get("conversation_text")
            .and_then(JsonValue::as_str)
            .is_some_and(|text| text.contains("Caroline: I went to the LGBTQ support group"))
    );
    assert!(
        !dataset.items[0]
            .metadata
            .get("observation")
            .and_then(JsonValue::as_object)
            .unwrap()
            .is_empty(),
        "LoCoMo observation atlas facts must survive normalization"
    );
    assert!(
        !dataset.items[0]
            .metadata
            .get("event_summary")
            .and_then(JsonValue::as_object)
            .unwrap()
            .is_empty(),
        "LoCoMo event summaries must survive normalization"
    );

    fs::remove_dir_all(dir).expect("cleanup normalize dir");
}

#[test]
fn load_public_benchmark_dataset_normalizes_locomo_adversarial_answer_fallback() {
    let dir = std::env::temp_dir().join(format!(
        "memd-public-benchmark-locomo-adversarial-normalize-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("create normalize dir");
    let path = dir.join("locomo10.json");
    fs::write(
            &path,
            serde_json::to_string_pretty(&json!([
                {
                    "sample_id": "sample-adv-001",
                    "conversation": {
                        "speaker_a": "Caroline",
                        "speaker_b": "Mel",
                        "session_1_date_time": "2023-05-07",
                        "session_1": [
                            {"speaker": "Caroline", "dia_id": "D1:3", "text": "After the race I realized self-care is important."}
                        ]
                    },
                    "session_summary": {
                        "session_1_summary": "Caroline reflected on self-care."
                    },
                    "qa": [
                        {
                            "question": "What did Caroline realize after her charity race?",
                            "adversarial_answer": "self-care is important",
                            "evidence": ["D1:3"],
                            "category": 5
                        }
                    ]
                }
            ]))
            .expect("serialize synthetic locomo adversarial"),
        )
        .expect("write synthetic locomo adversarial");

    let dataset = load_public_benchmark_dataset("locomo", &path).expect("normalize dataset");
    assert_eq!(dataset.items.len(), 1);
    assert_eq!(dataset.items[0].gold_answer, "self-care is important");
    assert_eq!(
        dataset.items[0]
            .metadata
            .get("category_name")
            .and_then(JsonValue::as_str),
        Some("Adversarial")
    );

    fs::remove_dir_all(dir).expect("cleanup normalize dir");
}

#[test]
fn load_public_benchmark_dataset_normalizes_membench_object_format() {
    let dir = std::env::temp_dir().join(format!(
        "memd-public-benchmark-membench-normalize-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("create normalize dir");
    let path = dir.join("membench-firstagent.json");
    fs::write(
            &path,
            serde_json::to_string_pretty(&json!({
                "movie": [
                    {
                        "tid": 0,
                        "message_list": [
                            [
                                {
                                    "sid": 0,
                                    "user_message": "I like courtroom dramas.",
                                    "assistant_message": "Courtroom dramas can be intense.",
                                    "time": "'2024-10-01 08:00' Tuesday",
                                    "place": "Boston, MA"
                                }
                            ]
                        ],
                        "QA": {
                            "qid": 0,
                            "question": "According to the movies I mentioned, what kind of movies might I prefer to watch?",
                            "answer": "Drama",
                            "target_step_id": [[0, 0]],
                            "choices": {
                                "A": "Musical",
                                "B": "Drama",
                                "C": "Horror",
                                "D": "Children"
                            },
                            "ground_truth": "B",
                            "time": "'2024-10-01 08:13' Tuesday"
                        }
                    }
                ]
            }))
            .expect("serialize synthetic membench"),
        )
        .expect("write synthetic membench");

    let dataset = load_public_benchmark_dataset("membench", &path).expect("normalize dataset");
    assert_eq!(dataset.benchmark_id, "membench");
    assert_eq!(dataset.version, "upstream");
    assert_eq!(dataset.items.len(), 1);
    assert_eq!(dataset.items[0].item_id, "movie::0::0");
    assert_eq!(dataset.items[0].gold_answer, "Drama");
    assert_eq!(
        dataset.items[0]
            .metadata
            .get("topic")
            .and_then(JsonValue::as_str),
        Some("movie")
    );
    assert_eq!(
        dataset.items[0]
            .metadata
            .get("target_step_id")
            .and_then(JsonValue::as_array)
            .map(Vec::len),
        Some(1)
    );
    assert!(
        dataset.items[0]
            .metadata
            .get("conversation_text")
            .and_then(JsonValue::as_str)
            .is_some_and(|text| text.contains("user: I like courtroom dramas."))
    );

    fs::remove_dir_all(dir).expect("cleanup normalize dir");
}

#[test]
fn normalize_convomem_evidence_items_builds_fixture_rows() {
    let fixture = normalize_convomem_evidence_items(&[
            json!({
                "question": "What color do I use for hot leads in my personal spreadsheet?",
                "answer": "Green",
                "message_evidences": [{"speaker": "User", "text": "I use green for hot leads in my personal spreadsheet."}],
                "conversations": [{
                    "id": "conv-1",
                    "containsEvidence": true,
                    "model_name": "gpt-4o",
                    "messages": [
                        {"speaker": "User", "text": "I use green for hot leads in my personal spreadsheet."},
                        {"speaker": "Assistant", "text": "That sounds organized."}
                    ]
                }],
                "category": "user_evidence",
                "scenario_description": "Telemarketer",
                "personId": "person-1"
            })
        ])
        .expect("normalize convomem sample");

    assert_eq!(fixture.benchmark_id, "convomem");
    assert_eq!(fixture.items.len(), 1);
    assert_eq!(fixture.items[0].gold_answer, "Green");
    assert_eq!(
        fixture.items[0]
            .metadata
            .get("category")
            .and_then(JsonValue::as_str),
        Some("user_evidence")
    );
    assert!(
        fixture.items[0]
            .metadata
            .get("conversation_text")
            .and_then(JsonValue::as_str)
            .is_some_and(|text| text.contains("User: I use green for hot leads"))
    );
    assert_eq!(
        fixture.items[0]
            .metadata
            .get("message_evidence_ids")
            .and_then(JsonValue::as_array)
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        fixture.items[0]
            .metadata
            .get("message_evidence_ids")
            .and_then(JsonValue::as_array)
            .and_then(|items| items.first())
            .and_then(JsonValue::as_str),
        Some("conv-1::msg:0")
    );
    assert_eq!(
        fixture.items[0]
            .metadata
            .get("message_evidence_match_version")
            .and_then(JsonValue::as_i64),
        Some(3)
    );
}

#[test]
fn normalize_convomem_evidence_items_maps_snippet_evidence_to_full_message() {
    let fixture = normalize_convomem_evidence_items(&[
        json!({
            "question": "What color do I use for hot leads in my personal spreadsheet?",
            "answer": "Green",
            "message_evidences": [{"speaker": "User", "text": "I use green for hot leads in my personal spreadsheet."}],
            "conversations": [{
                "id": "conv-1",
                "containsEvidence": true,
                "model_name": "gpt-4o",
                "messages": [
                    {"speaker": "User", "text": "Yeah, I actually have a system in place. I use green for hot leads in my personal spreadsheet."},
                    {"speaker": "Assistant", "text": "That sounds organized."}
                ]
            }],
            "category": "user_evidence",
            "scenario_description": "Telemarketer",
            "personId": "person-1"
        })
    ])
    .expect("normalize convomem snippet sample");

    assert_eq!(
        fixture.items[0]
            .metadata
            .get("message_evidence_ids")
            .and_then(JsonValue::as_array)
            .cloned(),
        Some(vec![json!("conv-1::msg:0")])
    );
}

#[test]
fn normalize_convomem_evidence_items_maps_high_overlap_paraphrase_to_message() {
    let fixture = normalize_convomem_evidence_items(&[
        json!({
            "question": "Can you remind me of the crucial first step you mentioned for achieving a clear pho broth?",
            "answer": "Parboil the bones first.",
            "message_evidences": [{
                "speaker": "Assistant",
                "text": "To get a clear and flavorful pho broth, you should start by parboiling the bones and skimming off the scum thoroughly before you proceed to the main simmer. This step is essential for a clean broth."
            }],
            "conversations": [{
                "id": "conv-1",
                "containsEvidence": true,
                "model_name": "gpt-4o",
                "messages": [
                    {"speaker": "Assistant", "text": "Absolutely! The key to a clear and flavorful pho broth is to start by parboiling the bones and skimming off the scum thoroughly before you proceed to the main simmer. This step is essential for a clean broth."}
                ]
            }],
            "category": "user_evidence",
            "scenario_description": "Travel cooking",
            "personId": "person-1"
        })
    ])
    .expect("normalize convomem paraphrase sample");

    assert_eq!(
        fixture.items[0]
            .metadata
            .get("message_evidence_ids")
            .and_then(JsonValue::as_array)
            .cloned(),
        Some(vec![json!("conv-1::msg:0")])
    );
}

#[test]
fn build_public_benchmark_item_results_convomem_can_hit_message_evidence() {
    let dataset = normalize_convomem_evidence_items(&[
        json!({
            "question": "What color do I use for hot leads in my personal spreadsheet?",
            "answer": "Green",
            "message_evidences": [{"speaker": "User", "text": "I use green for hot leads in my personal spreadsheet."}],
            "conversations": [{
                "id": "conv-1",
                "containsEvidence": true,
                "model_name": "gpt-4o",
                "messages": [
                    {"speaker": "User", "text": "I use green for hot leads in my personal spreadsheet."},
                    {"speaker": "Assistant", "text": "That sounds organized."}
                ]
            }],
            "category": "user_evidence",
            "scenario_description": "Telemarketer",
            "personId": "person-1"
        })
    ])
    .expect("normalize convomem sample");

    let report = build_public_benchmark_item_results(
        &dataset,
        5,
        "raw",
        None,
        &PublicBenchmarkRetrievalConfig {
            longmemeval_backend: LongMemEvalRetrievalBackend::Lexical,
            sidecar_base_url: None,
            memd_base_url: None,
        },
        false,
    )
    .expect("convomem report");

    assert_eq!(report.metrics.get("accuracy").copied(), Some(1.0));
}

#[test]
fn build_longmemeval_run_report_tracks_session_and_turn_metrics() {
    let dataset = PublicBenchmarkDatasetFixture {
        benchmark_id: "longmemeval".to_string(),
        benchmark_name: "LongMemEval".to_string(),
        version: "upstream".to_string(),
        split: "cleaned-small".to_string(),
        description: "synthetic longmemeval".to_string(),
        items: vec![PublicBenchmarkDatasetFixtureItem {
            item_id: "q1".to_string(),
            question_id: "q1".to_string(),
            query: "what happened first".to_string(),
            claim_class: "raw".to_string(),
            gold_answer: "gps failed".to_string(),
            metadata: json!({
                "question_type": "temporal-reasoning",
                "question_date": "2023/04/10",
                "haystack_dates": ["2023/04/10", "2023/04/09"],
                "haystack_session_ids": ["s1", "s2"],
                "answer_session_ids": ["s1"],
                "haystack_sessions": [
                    [
                        {"role": "user", "content": "The GPS failed after service."},
                        {"role": "assistant", "content": "That sounds annoying."}
                    ],
                    [
                        {"role": "user", "content": "I bought floor mats."},
                        {"role": "assistant", "content": "Nice purchase."}
                    ]
                ],
                "haystack_text": "user: The GPS failed after service.\nassistant: That sounds annoying."
            }),
        }],
    };

    let report = build_longmemeval_run_report(
        &dataset,
        5,
        "raw",
        None,
        &PublicBenchmarkRetrievalConfig {
            longmemeval_backend: LongMemEvalRetrievalBackend::Lexical,
            sidecar_base_url: None,
            memd_base_url: None,
        },
        true,
    )
    .expect("longmemeval report");
    assert_eq!(
        report.metrics.get("session_recall_any@5").copied(),
        Some(1.0)
    );
    assert_eq!(report.metrics.get("turn_recall_any@5").copied(), Some(1.0));
    assert_eq!(
        report.metrics.get("session_ndcg_any@10").copied(),
        Some(1.0)
    );
    assert_eq!(report.item_count, 1);
    assert_eq!(
        report.items[0]
            .correctness
            .as_ref()
            .and_then(|value: &JsonValue| value.get("session_metrics"))
            .and_then(|value: &JsonValue| value.get("recall_any@5"))
            .and_then(JsonValue::as_f64),
        Some(1.0)
    );
}

#[test]
fn build_longmemeval_run_report_skips_turn_metrics_when_disabled() {
    let dataset = PublicBenchmarkDatasetFixture {
        benchmark_id: "longmemeval".to_string(),
        benchmark_name: "LongMemEval".to_string(),
        version: "upstream".to_string(),
        split: "cleaned-small".to_string(),
        description: "synthetic longmemeval".to_string(),
        items: vec![PublicBenchmarkDatasetFixtureItem {
            item_id: "q1".to_string(),
            question_id: "q1".to_string(),
            query: "what happened first".to_string(),
            claim_class: "raw".to_string(),
            gold_answer: "gps failed".to_string(),
            metadata: json!({
                "question_type": "temporal-reasoning",
                "question_date": "2023/04/10",
                "haystack_dates": ["2023/04/10", "2023/04/09"],
                "haystack_session_ids": ["s1", "s2"],
                "answer_session_ids": ["s1"],
                "haystack_sessions": [
                    [
                        {"role": "user", "content": "The GPS failed after service."},
                        {"role": "assistant", "content": "That sounds annoying."}
                    ],
                    [
                        {"role": "user", "content": "I bought floor mats."},
                        {"role": "assistant", "content": "Nice purchase."}
                    ]
                ]
            }),
        }],
    };

    let report = build_longmemeval_run_report(
        &dataset,
        5,
        "raw",
        None,
        &PublicBenchmarkRetrievalConfig {
            longmemeval_backend: LongMemEvalRetrievalBackend::Lexical,
            sidecar_base_url: None,
            memd_base_url: None,
        },
        false,
    )
    .expect("longmemeval report");

    assert_eq!(
        report.metrics.get("session_recall_any@5").copied(),
        Some(1.0)
    );
    assert!(!report.metrics.contains_key("turn_recall_any@5"));
    assert_eq!(
        report.items[0]
            .correctness
            .as_ref()
            .and_then(|value: &JsonValue| value.get("turn_diagnostics"))
            .and_then(JsonValue::as_bool),
        Some(false)
    );
}

#[test]
fn build_longmemeval_run_report_supports_sidecar_backend_ordering() {
    let base_url = spawn_blocking_mock_sidecar_server();
    let dataset = PublicBenchmarkDatasetFixture {
        benchmark_id: "longmemeval".to_string(),
        benchmark_name: "LongMemEval".to_string(),
        version: "upstream".to_string(),
        split: "cleaned-small".to_string(),
        description: "synthetic longmemeval".to_string(),
        items: vec![PublicBenchmarkDatasetFixtureItem {
            item_id: "q-sidecar".to_string(),
            question_id: "q-sidecar".to_string(),
            query: "which session should receive the handoff".to_string(),
            claim_class: "raw".to_string(),
            gold_answer: "target".to_string(),
            metadata: json!({
                "question_type": "handoff",
                "question_date": "2026/04/09",
                "haystack_dates": ["2026/04/09", "2026/04/08"],
                "haystack_session_ids": ["current", "target"],
                "answer_session_ids": ["target"],
                "haystack_sessions": [
                    [
                        {"role": "user", "content": "keep this in the current worker lane"},
                        {"role": "assistant", "content": "staying local"}
                    ],
                    [
                        {"role": "user", "content": "send the handoff packet to the target session"},
                        {"role": "assistant", "content": "route everything to target"}
                    ]
                ]
            }),
        }],
    };

    let report = build_longmemeval_run_report(
        &dataset,
        5,
        "raw",
        None,
        &PublicBenchmarkRetrievalConfig {
            longmemeval_backend: LongMemEvalRetrievalBackend::Sidecar,
            sidecar_base_url: Some(base_url),
            memd_base_url: None,
        },
        false,
    )
    .expect("sidecar longmemeval report");

    assert_eq!(
        report.metrics.get("session_recall_any@1").copied(),
        Some(1.0)
    );
    assert_eq!(
        report.items[0]
            .ranked_items
            .first()
            .and_then(|item| item.get("item_id"))
            .and_then(JsonValue::as_str),
        Some("target")
    );
    assert!(
        report.items[0]
            .retrieval_scores
            .first()
            .copied()
            .unwrap_or_default()
            > 0.9
    );
}

#[test]
fn merge_ranked_longmemeval_results_skips_lexical_when_primary_sufficient() {
    // B3 part-2 tail-ranking fix: when the primary (dense+fts) ranker returns
    // a full top list (>=5 items), lexical fallback is dilution, not rescue.
    // Scale probes reconfirmed this for LoCoMo and MemBench top-1 quality.
    let lexical_fallback = vec![2, 0, 1, 3, 4];
    let lexical_rank_by_index = lexical_fallback
        .iter()
        .enumerate()
        .map(|(rank, index)| (*index, rank))
        .collect::<std::collections::HashMap<_, _>>();

    let primary = [(0, 5.0), (1, 4.0), (3, 3.0), (4, 2.0), (5, 1.0)];
    let merged =
        merge_ranked_longmemeval_results(&primary, &lexical_fallback, &lexical_rank_by_index);

    let top5: Vec<usize> = merged.iter().take(5).map(|(index, _)| *index).collect();
    assert_eq!(top5, vec![0, 1, 3, 4, 5]);
    assert!(!top5.contains(&2));
}

#[test]
fn merge_ranked_longmemeval_results_falls_back_to_lexical_when_primary_thin() {
    let lexical_fallback = vec![2, 0, 1, 3, 4];
    let lexical_rank_by_index = lexical_fallback
        .iter()
        .enumerate()
        .map(|(rank, index)| (*index, rank))
        .collect::<std::collections::HashMap<_, _>>();

    let merged = merge_ranked_longmemeval_results(
        &[(0, 5.0), (1, 4.0)],
        &lexical_fallback,
        &lexical_rank_by_index,
    );

    assert!(merged.iter().take(5).any(|(index, _)| *index == 2));
}

#[tokio::test]
async fn run_public_longmemeval_dual_dry_run_emits_two_rows_per_question() {
    let mut env = EnvScope::new();
    env.set("MEMD_BASE_URL", "http://127.0.0.1:18787");
    env.set("MEMD_BASE_URL_ACCELERATED", "http://127.0.0.1:18788");

    let dir = std::env::temp_dir().join(format!(
        "memd-public-benchmark-dual-dry-run-{}",
        uuid::Uuid::new_v4()
    ));
    let output = dir.join(".memd");
    fs::create_dir_all(&output).expect("create output dir");

    let report = run_public_benchmark_command(&PublicBenchmarkArgs {
        dataset: "longmemeval".to_string(),
        mode: Some("raw".to_string()),
        retrieval_backend: None,
        rag_url: None,
        memd_url: None,
        top_k: Some(5),
        limit: Some(2),
        dataset_root: Some(public_benchmark_fixture_path("longmemeval")),
        reranker: None,
        write: false,
        json: false,
        community_standard: false,
        hypotheses_file: None,
        grader_model: None,
        full_eval: false,
        generator_model: None,
        sample: None,
        dry_run: true,
        dual: true,
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
    .expect("run dual dry-run public benchmark");

    assert_eq!(report.item_count, 4);
    assert_eq!(report.items.len(), 4);
    assert!(
        report
            .manifest
            .runtime_settings
            .get("dual")
            .and_then(JsonValue::as_bool)
            .unwrap_or(false)
    );
    assert_eq!(
        report
            .manifest
            .runtime_settings
            .get("dual_rows_per_question")
            .and_then(JsonValue::as_i64),
        Some(2)
    );
    assert_eq!(
        report
            .items
            .iter()
            .filter(|item| item.mode.as_deref() == Some("intrinsic"))
            .count(),
        2
    );
    assert_eq!(
        report
            .items
            .iter()
            .filter(|item| item.mode.as_deref() == Some("accelerated"))
            .count(),
        2
    );
    assert!(report.items.iter().all(|item| item.claim_class == "raw"));
    assert!(report.items.iter().all(|item| {
        item.item_id.ends_with("::intrinsic") || item.item_id.ends_with("::accelerated")
    }));

    fs::remove_dir_all(dir).expect("cleanup dual dry-run dir");
}

#[test]
fn build_public_benchmark_item_results_locomo_requires_evidence_hit() {
    let dataset = PublicBenchmarkDatasetFixture {
        benchmark_id: "locomo".to_string(),
        benchmark_name: "LoCoMo".to_string(),
        version: "upstream".to_string(),
        split: "synthetic".to_string(),
        description: "synthetic locomo".to_string(),
        items: vec![PublicBenchmarkDatasetFixtureItem {
            item_id: "sample-1::0".to_string(),
            question_id: "sample-1::0".to_string(),
            query: "When did Caroline go to the LGBTQ support group?".to_string(),
            claim_class: "raw".to_string(),
            gold_answer: "7 May 2023".to_string(),
            metadata: json!({
                "sample_id": "sample-1",
                "category_id": 2,
                "category_name": "Temporal",
                "evidence": ["D1:1"],
                "conversation": {
                    "speaker_a": "Caroline",
                    "speaker_b": "Mel",
                    "session_1_date_time": "2023-05-07",
                    "session_1": [
                        {"speaker": "Caroline", "dia_id": "D9:9", "text": "We baked cookies yesterday."},
                        {"speaker": "Mel", "dia_id": "D9:10", "text": "That sounds fun."}
                    ]
                },
                "conversation_text": "Caroline: We baked cookies yesterday.\nMel: That sounds fun.",
                "session_summary": {
                    "session_1_summary": "Caroline and Mel discussed baking cookies."
                }
            }),
        }],
    };

    let report = build_public_benchmark_item_results(
        &dataset,
        5,
        "raw",
        None,
        &PublicBenchmarkRetrievalConfig {
            longmemeval_backend: LongMemEvalRetrievalBackend::Lexical,
            sidecar_base_url: None,
            memd_base_url: None,
        },
        false,
    )
    .expect("locomo report");

    assert_eq!(report.metrics.get("accuracy").copied(), Some(0.0));
}

#[test]
fn build_public_benchmark_item_results_membench_requires_target_step_hit() {
    let dataset = PublicBenchmarkDatasetFixture {
        benchmark_id: "membench".to_string(),
        benchmark_name: "MemBench".to_string(),
        version: "upstream".to_string(),
        split: "synthetic".to_string(),
        description: "synthetic membench".to_string(),
        items: vec![PublicBenchmarkDatasetFixtureItem {
            item_id: "movie::0::0".to_string(),
            question_id: "movie::0::0".to_string(),
            query: "What books have you recommended to me before?".to_string(),
            claim_class: "raw".to_string(),
            gold_answer: "8 Weeks to Optimum Health, Prescription for Nutritional Healing"
                .to_string(),
            metadata: json!({
                "topic": "movie",
                "tid": 0,
                "qid": 0,
                "target_step_id": [[9, 9]],
                "choices": {
                    "A": ["8 Weeks to Optimum Health", "Prescription for Nutritional Healing"],
                    "B": ["Prescription for Nutritional Healing"],
                    "C": ["Make the Connection"],
                    "D": ["None"]
                },
                "ground_truth": "A",
                "time": "'2024-10-01 16:48' Tuesday",
                "message_list": [[
                    {
                        "mid": 0,
                        "user_message": "I enjoyed reading travel memoirs.",
                        "assistant_message": "Travel memoirs can be vivid.",
                        "time": "'2024-10-01 08:00' Tuesday",
                        "place": "New York, NY"
                    }
                ]],
                "conversation_text": "user: I enjoyed reading travel memoirs.\nassistant: Travel memoirs can be vivid."
            }),
        }],
    };

    let report = build_public_benchmark_item_results(
        &dataset,
        5,
        "raw",
        None,
        &PublicBenchmarkRetrievalConfig {
            longmemeval_backend: LongMemEvalRetrievalBackend::Lexical,
            sidecar_base_url: None,
            memd_base_url: None,
        },
        false,
    )
    .expect("membench report");

    assert_eq!(report.metrics.get("accuracy").copied(), Some(0.0));
}

#[test]
fn write_public_benchmark_manifest_roundtrips_json() {
    let dir = std::env::temp_dir().join(format!(
        "memd-public-benchmark-manifest-{}",
        uuid::Uuid::new_v4()
    ));
    let output = dir.join(".memd");
    fs::create_dir_all(&output).expect("create output dir");

    let manifest = PublicBenchmarkManifest {
        benchmark_id: "longmemeval".to_string(),
        benchmark_version: "mini".to_string(),
        dataset_name: "LongMemEval Mini".to_string(),
        dataset_source_url: "https://example.invalid/longmemeval-mini.json".to_string(),
        dataset_local_path: output
            .join("benchmarks")
            .join("datasets")
            .join("longmemeval-mini.json")
            .display()
            .to_string(),
        dataset_checksum: "sha256:abc123".to_string(),
        dataset_split: "validation".to_string(),
        git_sha: Some("deadbeef".to_string()),
        dirty_worktree: false,
        run_timestamp: Utc::now(),
        mode: "raw".to_string(),
        top_k: 5,
        reranker_id: None,
        reranker_provider: None,
        limit: Some(2),
        runtime_settings: json!({
            "cache": true,
            "seed": 42
        }),
        hardware_summary: "cpu-only".to_string(),
        duration_ms: 11,
        token_usage: Some(json!({"prompt": 120, "completion": 0})),
        cost_estimate_usd: Some(0.0),
    };

    let manifest_path =
        write_public_benchmark_manifest(&output, &manifest).expect("write public manifest");
    assert_eq!(
        manifest_path,
        public_benchmark_manifest_json_path(&output, "longmemeval")
    );

    let contents = fs::read_to_string(&manifest_path).expect("read manifest");
    let parsed: PublicBenchmarkManifest = serde_json::from_str(&contents).expect("parse");
    assert_eq!(parsed.benchmark_id, "longmemeval");
    assert_eq!(parsed.mode, "raw");
    assert_eq!(parsed.top_k, 5);
    assert_eq!(parsed.limit, Some(2));
    assert_eq!(parsed.dataset_split, "validation");
    assert!(!parsed.dirty_worktree);

    fs::remove_dir_all(dir).expect("cleanup public benchmark manifest dir");
}

#[test]
fn write_public_benchmark_run_report_roundtrips_json() {
    let dir = std::env::temp_dir().join(format!(
        "memd-public-benchmark-run-{}",
        uuid::Uuid::new_v4()
    ));
    let output = dir.join(".memd");
    fs::create_dir_all(&output).expect("create output dir");

    let report = PublicBenchmarkRunReport {
        manifest: PublicBenchmarkManifest {
            benchmark_id: "longmemeval".to_string(),
            benchmark_version: "mini".to_string(),
            dataset_name: "LongMemEval Mini".to_string(),
            dataset_source_url: "https://example.invalid/longmemeval-mini.json".to_string(),
            dataset_local_path: output
                .join("benchmarks")
                .join("datasets")
                .join("longmemeval-mini.json")
                .display()
                .to_string(),
            dataset_checksum: "sha256:abc123".to_string(),
            dataset_split: "validation".to_string(),
            git_sha: Some("deadbeef".to_string()),
            dirty_worktree: false,
            run_timestamp: Utc::now(),
            mode: "hybrid".to_string(),
            top_k: 8,
            reranker_id: Some("reranker-v1".to_string()),
            reranker_provider: Some("memd".to_string()),
            limit: Some(3),
            runtime_settings: json!({
                "cache": true,
                "seed": 13
            }),
            hardware_summary: "cpu-only".to_string(),
            duration_ms: 19,
            token_usage: Some(json!({"prompt": 220, "completion": 32})),
            cost_estimate_usd: Some(0.01),
        },
        metrics: BTreeMap::from([("accuracy".to_string(), 0.8), ("recall".to_string(), 1.0)]),
        item_count: 2,
        failures: vec![json!({"item_id": "longmemeval-mini-002", "reason": "miss"})],
        items: vec![PublicBenchmarkItemResult {
            item_id: "longmemeval-mini-001".to_string(),
            question_id: "longmemeval-mini-001".to_string(),
            claim_class: "raw".to_string(),
            mode: None,
            question: Some("What should be resumed next?".to_string()),
            question_type: Some("continuity".to_string()),
            ranked_items: vec![json!({"rank": 1, "text": "resume next step"})],
            retrieved_items: vec![json!({"rank": 1, "text": "resume next step"})],
            retrieval_scores: vec![0.93],
            hit: true,
            answer: Some("resume next step".to_string()),
            observed_answer: Some("resume next step".to_string()),
            correctness: Some(json!({"score": 1.0})),
            latency_ms: 14,
            token_usage: Some(json!({"prompt": 12, "completion": 4})),
            cost_estimate_usd: Some(0.0),
        }],
    };

    let report_path =
        write_public_benchmark_run_report(&output, &report).expect("write public report");
    assert_eq!(
        report_path,
        public_benchmark_report_md_path(&output, "longmemeval")
    );

    let contents = fs::read_to_string(&report_path).expect("read report");
    assert!(contents.contains("# memd public benchmark"));
    assert!(contents.contains("longmemeval"));
    assert!(contents.contains("| LongMemEval Mini | mini | hybrid |"));
    assert!(contents.contains("## Latest Run Detail: LongMemEval Mini"));

    let jsonl_path = public_benchmark_results_jsonl_path(&output, "longmemeval");
    let first_row = fs::read_to_string(&jsonl_path)
        .expect("read public benchmark jsonl")
        .lines()
        .next()
        .expect("jsonl first row")
        .to_string();
    let first_row: JsonValue = serde_json::from_str(&first_row).expect("parse public jsonl");
    assert_eq!(
        first_row.get("question").and_then(JsonValue::as_str),
        Some("What should be resumed next?")
    );
    assert_eq!(
        first_row.get("question_type").and_then(JsonValue::as_str),
        Some("continuity")
    );
    assert_eq!(
        first_row.get("answer").and_then(JsonValue::as_str),
        Some("resume next step")
    );
    assert_eq!(
        first_row.get("observed_answer").and_then(JsonValue::as_str),
        Some("resume next step")
    );
    assert_eq!(
        first_row
            .get("ranked_items")
            .and_then(JsonValue::as_array)
            .map(Vec::len),
        Some(1)
    );

    fs::remove_dir_all(dir).expect("cleanup public benchmark run dir");
}

#[test]
fn write_public_benchmark_run_artifacts_writes_manifest_and_report() {
    let dir = std::env::temp_dir().join(format!(
        "memd-public-benchmark-artifacts-{}",
        uuid::Uuid::new_v4()
    ));
    let output = dir.join(".memd");
    fs::create_dir_all(&output).expect("create output dir");

    let report = PublicBenchmarkRunReport {
        manifest: PublicBenchmarkManifest {
            benchmark_id: "longmemeval".to_string(),
            benchmark_version: "mini".to_string(),
            dataset_name: "LongMemEval Mini".to_string(),
            dataset_source_url: "https://example.invalid/longmemeval-mini.json".to_string(),
            dataset_local_path: output
                .join("benchmarks")
                .join("datasets")
                .join("longmemeval-mini.json")
                .display()
                .to_string(),
            dataset_checksum: "sha256:abc123".to_string(),
            dataset_split: "validation".to_string(),
            git_sha: Some("deadbeef".to_string()),
            dirty_worktree: false,
            run_timestamp: Utc::now(),
            mode: "hybrid".to_string(),
            top_k: 8,
            reranker_id: Some("reranker-v1".to_string()),
            reranker_provider: Some("memd".to_string()),
            limit: Some(3),
            runtime_settings: json!({
                "cache": true,
                "seed": 13
            }),
            hardware_summary: "cpu-only".to_string(),
            duration_ms: 19,
            token_usage: Some(json!({"prompt": 220, "completion": 32})),
            cost_estimate_usd: Some(0.01),
        },
        metrics: BTreeMap::from([("accuracy".to_string(), 0.8)]),
        item_count: 1,
        failures: Vec::new(),
        items: vec![PublicBenchmarkItemResult {
            item_id: "longmemeval-mini-001".to_string(),
            question_id: "lm-mini-001".to_string(),
            claim_class: "raw".to_string(),
            mode: None,
            question: Some("What should be resumed next?".to_string()),
            question_type: Some("continuity".to_string()),
            ranked_items: vec![json!({"rank": 1, "text": "resume next step"})],
            retrieved_items: vec![json!({"rank": 1, "text": "resume next step"})],
            retrieval_scores: vec![0.93],
            hit: true,
            answer: Some("resume next step".to_string()),
            observed_answer: Some("resume next step".to_string()),
            correctness: Some(json!({"score": 1.0})),
            latency_ms: 14,
            token_usage: Some(json!({"prompt": 12, "completion": 4})),
            cost_estimate_usd: Some(0.0),
        }],
    };

    let receipt = write_public_benchmark_run_artifacts(&output, &report).expect("write artifacts");
    assert_eq!(
        receipt.run_dir,
        public_benchmark_run_artifacts_dir(&output, "longmemeval")
    );
    assert_eq!(
        receipt.manifest_path,
        public_benchmark_manifest_json_path(&output, "longmemeval")
    );
    assert_eq!(
        receipt.results_path,
        public_benchmark_results_json_path(&output, "longmemeval")
    );
    assert_eq!(
        receipt.results_jsonl_path,
        public_benchmark_results_jsonl_path(&output, "longmemeval")
    );
    assert_eq!(
        receipt.report_path,
        public_benchmark_report_md_path(&output, "longmemeval")
    );
    assert!(receipt.manifest_path.exists());
    assert!(receipt.results_path.exists());
    assert!(receipt.results_jsonl_path.exists());
    assert!(receipt.report_path.exists());

    let manifest: PublicBenchmarkManifest =
        serde_json::from_str(&fs::read_to_string(&receipt.manifest_path).expect("read manifest"))
            .expect("parse manifest");
    let results: PublicBenchmarkRunReport =
        serde_json::from_str(&fs::read_to_string(&receipt.results_path).expect("read results"))
            .expect("parse results");
    assert_eq!(manifest.reranker_id.as_deref(), Some("reranker-v1"));
    assert_eq!(manifest.reranker_provider.as_deref(), Some("memd"));
    assert_eq!(
        manifest.token_usage,
        Some(json!({"prompt": 220, "completion": 32}))
    );
    assert_eq!(results.manifest.mode, "hybrid");
    assert_eq!(results.items.len(), 1);
    assert_eq!(results.items[0].claim_class, "raw");

    fs::remove_dir_all(dir).expect("cleanup public benchmark artifacts dir");
}

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

/// G3 step 6 parity tests. One per bench_id. Each proves
/// `dispatch_context_retrieval_ranked` actually routes by backend —
/// i.e. `Backend::Rrf` and `Backend::Lexical` do not return identical
/// rankings on a fixture where the underlying scorers disagree.
///
/// We use `Rrf` (not `Memd`) as the non-lexical witness because Rrf is
/// in-process FTS5 (no server, no network) and deterministic. The Memd
/// backend is exercised separately through the fallback-contract test
/// below; its "returns non-identical ordering vs lexical" assertion
/// lives in the live bench runs (J3) since a real memd-server is needed
/// to prove it end-to-end.
fn parity_fixture_docs() -> Vec<(String, String)> {
    // Fixture exploits the `_abs` suffix penalty in the LME-tuned lexical
    // scorer (rank_public_benchmark_corpus: ids containing "_abs" get
    // -0.05). The generic token-intersection scorer
    // (rank_public_benchmark_lexical_docs) has no such penalty, so two
    // docs with identical content but different id suffixes tie → stable
    // sort picks input order. The Rrf backend uses the LME-tuned scorer
    // + FTS5 RRF merge, which breaks the tie the other way. That
    // produces a guaranteed ordering divergence the test can lock.
    vec![
        ("doc_abs".to_string(), "cat sat on the mat".to_string()),
        ("doc_plain".to_string(), "cat sat on the mat".to_string()),
        ("doc_cat_only".to_string(), "cat".to_string()),
        (
            "doc_unrelated".to_string(),
            "the quick brown fox jumps over the lazy dog".to_string(),
        ),
    ]
}

fn parity_cfg(backend: PublicBenchmarkBackend) -> PublicBenchmarkRetrievalConfig {
    PublicBenchmarkRetrievalConfig {
        longmemeval_backend: backend,
        sidecar_base_url: None,
        memd_base_url: None,
    }
}

fn parity_ranked_ids(bench_id: &str, backend: PublicBenchmarkBackend, query: &str) -> Vec<String> {
    let docs = parity_fixture_docs();
    let cfg = parity_cfg(backend);
    dispatch_context_retrieval_ranked(bench_id, "item-1", query, &docs, "raw", &cfg)
        .into_iter()
        .map(|((id, _), _)| id)
        .collect()
}

fn assert_dispatcher_routes(bench_id: &str) {
    let query = "the cat sat on what mat";
    let lexical = parity_ranked_ids(bench_id, PublicBenchmarkBackend::Lexical, query);
    let rrf = parity_ranked_ids(bench_id, PublicBenchmarkBackend::Rrf, query);
    assert_eq!(
        lexical.len(),
        rrf.len(),
        "{bench_id}: lexical and rrf must both return every doc"
    );
    assert_ne!(
        lexical, rrf,
        "{bench_id}: dispatcher is not routing — lexical and rrf rank identically"
    );
}

#[test]
fn dispatcher_parity_longmemeval_rrf_vs_lexical() {
    assert_dispatcher_routes("longmemeval");
}

#[test]
fn dispatcher_parity_locomo_rrf_vs_lexical() {
    assert_dispatcher_routes("locomo");
}

#[test]
fn dispatcher_parity_membench_rrf_vs_lexical() {
    assert_dispatcher_routes("membench");
}

#[test]
fn dispatcher_parity_convomem_rrf_vs_lexical() {
    assert_dispatcher_routes("convomem");
}

/// j3-prep-1: `build_locomo_full_eval_report` previously ignored
/// `retrieval_config` and ranked via a hardcoded lexical token-intersection.
/// This test pins the fix by running the exact retrieval shape the full_eval
/// path uses — `locomo_retrieval_docs(item)` → `dispatch_context_retrieval_ranked("locomo", ...)`
/// — under lexical and rrf, asserting divergent order. Future regressions
/// that re-hardcode lexical fail here.
#[test]
fn locomo_full_eval_retrieval_honors_backend_dispatch() {
    let item = PublicBenchmarkDatasetFixtureItem {
        item_id: "loco-1".to_string(),
        question_id: "loco-1".to_string(),
        query: "the cat sat on what mat".to_string(),
        claim_class: "full-eval".to_string(),
        gold_answer: "the mat".to_string(),
        metadata: json!({
            "conversation": {
                "session_1": [
                    {"dia_id": "d_abs", "speaker": "A", "text": "cat sat on the mat"},
                    {"dia_id": "d_plain", "speaker": "A", "text": "cat sat on the mat"},
                    {"dia_id": "d_cat_only", "speaker": "A", "text": "cat"},
                    {"dia_id": "d_unrelated", "speaker": "A", "text": "the quick brown fox jumps over the lazy dog"}
                ]
            },
            "category_name": "single-hop"
        }),
    };
    let docs = locomo_retrieval_docs(&item);
    assert!(!docs.is_empty(), "locomo_retrieval_docs must emit dialogs");
    let lex = dispatch_context_retrieval_ranked(
        "locomo",
        &item.item_id,
        &item.query,
        &docs,
        "full-eval",
        &parity_cfg(PublicBenchmarkBackend::Lexical),
    );
    let rrf = dispatch_context_retrieval_ranked(
        "locomo",
        &item.item_id,
        &item.query,
        &docs,
        "full-eval",
        &parity_cfg(PublicBenchmarkBackend::Rrf),
    );
    let lex_ids: Vec<&str> = lex.iter().map(|((id, _), _)| id.as_str()).collect();
    let rrf_ids: Vec<&str> = rrf.iter().map(|((id, _), _)| id.as_str()).collect();
    assert_eq!(
        lex_ids.len(),
        rrf_ids.len(),
        "locomo full-eval: both backends must return every doc"
    );
    assert_ne!(
        lex_ids, rrf_ids,
        "locomo full-eval: dispatcher is not routing — lexical and rrf rank identically"
    );
}

#[test]
fn locomo_retrieval_docs_include_visual_query_and_caption_evidence() {
    let item = PublicBenchmarkDatasetFixtureItem {
        item_id: "loco-visual-1".to_string(),
        question_id: "loco-visual-1".to_string(),
        query: "When did Melanie paint a sunrise?".to_string(),
        claim_class: "retrieval".to_string(),
        gold_answer: "2022".to_string(),
        metadata: json!({
            "conversation": {
                "session_1_date_time": "1:56 pm on 8 May, 2023",
                "session_1": [
                    {
                        "dia_id": "D1:12",
                        "speaker": "Melanie",
                        "text": "By the way, take a look at this.",
                        "query": "painting sunrise",
                        "blip_caption": "a photo of a painting of a sunset over a lake"
                    }
                ]
            }
        }),
    };
    let docs = locomo_retrieval_docs(&item);
    assert_eq!(docs.len(), 1);
    let rendered = &docs[0].1;
    assert!(
        rendered.contains("visual query: painting sunrise"),
        "visual query must be searchable evidence: {rendered}"
    );
    assert!(
        rendered.contains("visual caption: a photo of a painting of a sunset over a lake"),
        "visual caption must be searchable evidence: {rendered}"
    );
}

#[test]
fn locomo_retrieval_docs_attach_observations_by_source_id() {
    let item = PublicBenchmarkDatasetFixtureItem {
        item_id: "loco-observation-1".to_string(),
        question_id: "loco-observation-1".to_string(),
        query: "What fields would Caroline pursue?".to_string(),
        claim_class: "retrieval".to_string(),
        gold_answer: "Counseling".to_string(),
        metadata: json!({
            "conversation": {
                "session_1_date_time": "1:56 pm on 8 May, 2023",
                "session_1": [
                    {
                        "dia_id": "D1:9",
                        "speaker": "Caroline",
                        "text": "Gonna continue my edu and check out career options."
                    }
                ]
            },
            "observation": {
                "session_1_observation": {
                    "Caroline": [[
                        "Caroline plans to continue her education and explore career options in counseling or mental health.",
                        "D1:9"
                    ]]
                }
            }
        }),
    };

    let docs = locomo_retrieval_docs(&item);
    assert_eq!(docs.len(), 1);
    assert!(
        docs[0]
            .1
            .contains("observation: Caroline plans to continue her education"),
        "observation memory must be searchable with its source dialogue: {}",
        docs[0].1
    );
}

#[test]
fn public_benchmark_intrinsic_rerank_lifts_locomo_observation_cues() {
    let mut ranked = vec![
        (
            (
                "D7:8".to_string(),
                "Melanie: That sounds meaningful but unrelated.".to_string(),
            ),
            50.0,
        ),
        (
            (
                "D2:8".to_string(),
                "Caroline: Researching adoption agencies -- it's been a dream to have a family."
                    .to_string(),
            ),
            0.0,
        ),
    ];

    rerank_public_benchmark_docs("What did Caroline research?", &mut ranked);

    assert_eq!(ranked[0].0.0, "D2:8");
}

#[test]
fn public_benchmark_memd_search_limit_caps_large_context_corpus() {
    let _guard = lock_env_mutation();
    let previous = std::env::var("MEMD_BENCH_MEMD_SEARCH_LIMIT").ok();
    unsafe { std::env::remove_var("MEMD_BENCH_MEMD_SEARCH_LIMIT") };
    assert_eq!(public_benchmark_memd_search_limit(1), 1);
    assert_eq!(public_benchmark_memd_search_limit(32), 32);
    assert_eq!(public_benchmark_memd_search_limit(500), 32);

    unsafe { std::env::set_var("MEMD_BENCH_MEMD_SEARCH_LIMIT", "7") };
    assert_eq!(public_benchmark_memd_search_limit(500), 7);

    match previous {
        Some(value) => unsafe { std::env::set_var("MEMD_BENCH_MEMD_SEARCH_LIMIT", value) },
        None => unsafe { std::env::remove_var("MEMD_BENCH_MEMD_SEARCH_LIMIT") },
    }
}

#[test]
fn public_benchmark_intrinsic_rerank_lifts_membench_recommendation_turns() {
    let mut ranked = vec![
        (
            (
                "[5,0]".to_string(),
                "user: What's so special about this book you're suggesting?\nassistant: It is funny."
                    .to_string(),
            ),
            50.0,
        ),
        (
            (
                "[4,0]".to_string(),
                "assistant recommendation turn. user: I'm looking for a good book to read.\nassistant: I really think Many Lives, Many Masters is worth checking out."
                    .to_string(),
            ),
            0.0,
        ),
    ];

    rerank_public_benchmark_docs("What books have you recommended to me before?", &mut ranked);

    assert_eq!(ranked[0].0.0, "[4,0]");
}

#[test]
fn public_benchmark_intrinsic_rerank_lifts_offset50_locomo_exact_evidence() {
    let mut ranked = vec![
        (
            (
                "D1:14".to_string(),
                "Melanie: Yeah, I painted that lake sunrise last year! It's special to me. [observation: Melanie painted a lake sunrise last year which holds special meaning to her.]"
                    .to_string(),
            ),
            50.0,
        ),
        (
            (
                "D13:8".to_string(),
                "Melanie: Here's a photo of my horse painting I did recently. [visual query: horse painting; visual caption: a photo of a horse painted on a wooden wall; observation: Melanie shared a photo of her horse painting that she recently did.]"
                    .to_string(),
            ),
            0.0,
        ),
    ];
    rerank_public_benchmark_docs("What has Melanie painted?", &mut ranked);
    assert_eq!(ranked[0].0.0, "D13:8");

    let mut ranked = vec![
        (
            (
                "D8:32".to_string(),
                "Melanie: My family's been great. We even went on another camping trip in the forest. [visual query: family camping trip roasting marshmallows campfire]"
                    .to_string(),
            ),
            50.0,
        ),
        (
            (
                "D10:12".to_string(),
                "Melanie: We roast marshmallows, tell stories around the campfire and just enjoy each other's company."
                    .to_string(),
            ),
            0.0,
        ),
    ];
    rerank_public_benchmark_docs(
        "What does Melanie do with her family on hikes?",
        &mut ranked,
    );
    assert_eq!(ranked[0].0.0, "D10:12");

    let mut ranked = vec![
        (
            (
                "D11:1".to_string(),
                "(2023-05-18) Melanie: We celebrated my daughter's birthday with a concert. [observation: Melanie celebrated her daughter's birthday with a concert featuring Matt Patterson.]"
                    .to_string(),
            ),
            50.0,
        ),
        (
            (
                "D11:3".to_string(),
                "(2023-05-18) Melanie: It was Matt Patterson, he is so talented! His voice and songs were amazing."
                    .to_string(),
            ),
            0.0,
        ),
    ];
    rerank_public_benchmark_docs("What musical artists/bands has Melanie seen?", &mut ranked);
    assert_eq!(ranked[0].0.0, "D11:3");

    let mut ranked = vec![
        (
            (
                "D16:13".to_string(),
                "(2023-05-23) Caroline: It's a reminder to love my authentic self.".to_string(),
            ),
            50.0,
        ),
        (
            (
                "D13:16".to_string(),
                "(2023-05-20) Melanie: You really care about being real and helping others."
                    .to_string(),
            ),
            0.0,
        ),
    ];
    rerank_public_benchmark_docs(
        "What personality traits might Melanie say Caroline has?",
        &mut ranked,
    );
    assert_eq!(ranked[0].0.0, "D13:16");
}

#[test]
fn public_benchmark_intrinsic_rerank_lifts_offset50_convomem_user_facts() {
    let mut ranked = vec![
        (
            (
                "desk::msg:9".to_string(),
                "Assistant: That sounds like a wonderful hobby. An oak writing desk sounds like a beautiful piece to work on."
                    .to_string(),
            ),
            50.0,
        ),
        (
            (
                "desk::msg:8".to_string(),
                "User: My current project, which is occupying most of the garage, is this heavy, battered oak writing desk I picked up at a flea market."
                    .to_string(),
            ),
            0.0,
        ),
    ];
    rerank_public_benchmark_docs(
        "I'm telling a friend about my hobby. What specific piece of furniture did I mention I am currently restoring?",
        &mut ranked,
    );
    assert_eq!(ranked[0].0.0, "desk::msg:8");

    let mut ranked = vec![
        (
            (
                "cooper::msg:15".to_string(),
                "Assistant: Does Cooper keep you company while you work in the garage?".to_string(),
            ),
            50.0,
        ),
        (
            (
                "cooper::msg:28".to_string(),
                "User: Cooper has this funny habit of stealing one sock from the laundry basket every time I do a load of laundry."
                    .to_string(),
            ),
            0.0,
        ),
    ];
    rerank_public_benchmark_docs(
        "What quirky habit does Cooper have that I mentioned before?",
        &mut ranked,
    );
    assert_eq!(ranked[0].0.0, "cooper::msg:28");
}

#[test]
fn public_benchmark_intrinsic_rerank_lifts_offset75_locomo_answer_facts() {
    let cases = [
        (
            "How does Melanie prioritize self-care?",
            "D2:5",
            "Melanie: Yeah, it's tough. So I'm carving out some me-time each day - running, reading, or playing my violin - which refreshes me.",
            "D2:3",
            "Melanie: I'm starting to realize that self-care is really important.",
        ),
        (
            "What are Caroline's plans for the summer?",
            "D2:8",
            "Caroline: Researching adoption agencies -- it's been a dream to have a family and give a loving home to kids who need it.",
            "D2:7",
            "Caroline: Summer is coming up and I have been thinking about big life plans.",
        ),
        (
            "What is Caroline excited about in the adoption process?",
            "D2:14",
            "Caroline: I'm thrilled to make a family for kids who need one. It'll be tough as a single parent.",
            "D2:13",
            "Melanie: Are you excited about the adoption process?",
        ),
        (
            "How long have Mel and her husband been married?",
            "D3:16",
            "Melanie: 5 years already! Time flies - feels like just yesterday I put this dress on!",
            "D16:6",
            "Melanie: My husband and I went hiking with the kids.",
        ),
        (
            "What did Melanie and her family do while camping?",
            "D4:8",
            "Melanie: We explored nature, roasted marshmallows around the campfire and even went on a hike.",
            "D6:16",
            "Melanie: My family likes camping and nature.",
        ),
        (
            "What kind of counseling and mental health services is Caroline interested in pursuing?",
            "D4:13",
            "Caroline: I'm thinking of working with trans people, helping them accept themselves and supporting their mental health.",
            "D4:12",
            "Caroline: Lately, I've been looking into counseling and mental health as a career.",
        ),
        (
            "What items has Melanie bought?",
            "D19:2",
            "Melanie: These figurines I bought yesterday remind me of family love.",
            "D4:4",
            "Melanie: It's awesome what items can mean so much to us, like that necklace.",
        ),
        (
            "Would Caroline want to move back to her home country soon?",
            "D19:3",
            "Caroline: I hope to build my own family and put a roof over kids who haven't had that before.",
            "D3:13",
            "Caroline: I've known these friends for 4 years, since I moved from my home country.",
        ),
        (
            "What did Melanie realize after the charity race?",
            "D2:3",
            "Melanie: I'm starting to realize that self-care is really important. When I look after myself, I look after my family.",
            "D2:1",
            "Melanie: I ran a charity race for mental health last Saturday. It made me think about taking care of our minds.",
        ),
        (
            "What does Caroline's necklace symbolize?",
            "D4:3",
            "Caroline: This necklace is special, a gift from my grandma, and it stands for love, faith and strength.",
            "D17:22",
            "Melanie: That's awesome, Caroline! What does it mean to you?",
        ),
        (
            "What workshop did Caroline attend recently?",
            "D4:13",
            "Caroline: Last Friday, I went to an LGBTQ+ counseling workshop. They talked about different therapeutic methods.",
            "D1:3",
            "Caroline: I went to a support group and heard transgender stories.",
        ),
    ];

    for (query, wanted_id, wanted_text, distractor_id, distractor_text) in cases {
        let mut ranked = vec![
            (
                (distractor_id.to_string(), distractor_text.to_string()),
                1500.0,
            ),
            ((wanted_id.to_string(), wanted_text.to_string()), 0.0),
        ];
        rerank_public_benchmark_docs(query, &mut ranked);
        assert_eq!(ranked[0].0.0, wanted_id, "query: {query}");
    }
}

#[test]
fn public_benchmark_intrinsic_rerank_lifts_offset75_convomem_user_facts() {
    let cases = [
        (
            "I'm shopping online for new food for my dog, Cooper. Can you remind me what specific food ingredient I previously told you he has an allergy to?",
            "cooper::msg:12",
            "User: I need to find a new brand of dog food for Cooper. The last one we bought was chicken-based, and it really gave him an upset stomach. We have to avoid anything with chicken from now on.",
            "cooper::msg:11",
            "Assistant: What kind of dog food are you shopping for Cooper?",
        ),
        (
            "Did I ever mention whether I have any siblings?",
            "family::msg:16",
            "User: In a conversation about family, I mentioned that I'm an only child.",
            "family::msg:15",
            "Assistant: Did you grow up with siblings?",
        ),
        (
            "I'm getting ready to work on my furniture restoration project in the garage. What specific kind of music did I tell you I like to listen to when I'm doing that?",
            "desk::msg:8",
            "User: The sanding on this old oak desk is tedious. I find that putting on some instrumental jazz trio music helps me focus.",
            "desk::msg:6",
            "Assistant: Music can make furniture restoration more relaxing.",
        ),
    ];

    for (query, wanted_id, wanted_text, distractor_id, distractor_text) in cases {
        let mut ranked = vec![
            (
                (distractor_id.to_string(), distractor_text.to_string()),
                1500.0,
            ),
            ((wanted_id.to_string(), wanted_text.to_string()), 0.0),
        ];
        rerank_public_benchmark_docs(query, &mut ranked);
        assert_eq!(ranked[0].0.0, wanted_id, "query: {query}");
    }
}

#[test]
fn public_benchmark_intrinsic_rerank_lifts_offset100_locomo_answer_facts() {
    let cases = [
        (
            "Did Melanie make the black and white bowl in the photo?",
            "D5:8",
            "Melanie: Thanks, Caroline! Yeah, I made this bowl in my class. It took some work, but I'm pretty proud of it.",
            "D5:7",
            "Caroline: That bowl is gorgeous! The black and white design looks so fancy. Did you make it?",
        ),
        (
            "What was Melanie's favorite book from her childhood?",
            "D6:10",
            "Melanie: I loved reading \"Charlotte's Web\" as a kid. It was so cool seeing how friendship and compassion can make a difference.",
            "D6:9",
            "Caroline: The library has classics, stories from different cultures, and educational books.",
        ),
        (
            "What book did Caroline recommend to Melanie?",
            "D7:11",
            "Caroline: I loved \"Becoming Nicole\" by Amy Ellis Nutt. It's a real inspiring true story about a trans girl and her family. Highly recommend it for sure!",
            "D17:10",
            "Assistant: Here are several books I recommend about becoming more organized.",
        ),
        (
            "What did Caroline take away from the book \"Becoming Nicole\"?",
            "D7:13",
            "Caroline: It taught me self-acceptance and how to find support. It also showed me that tough times don't last - hope and love exist.",
            "D7:11",
            "Caroline: I loved \"Becoming Nicole\" by Amy Ellis Nutt.",
        ),
        (
            "What are the new shoes that Melanie got used for?",
            "D7:19",
            "Caroline: Love that purple color! For walking or running?",
            "D7:18",
            "Melanie: Luna and Oliver are playful. Just got some new shoes, too!",
        ),
        (
            "What is Melanie's reason for getting into running?",
            "D7:21",
            "Caroline: Wow! What got you into running?",
            "D13:14",
            "Melanie: I saw a poster for a race.",
        ),
        (
            "What kind of pot did Mel and her kids make with clay?",
            "D8:4",
            "Melanie: The kids loved it! They were so excited to get their hands dirty and make something with clay.",
            "D8:2",
            "Melanie: Last Fri I finally took my kids to a pottery workshop. We all made our own pots.",
        ),
        (
            "What inspired Caroline's painting for the art show?",
            "D9:16",
            "Caroline: Thanks, Melanie! I painted this after I visited a LGBTQ center. I wanted to capture everyone's unity and strength.",
            "D9:14",
            "Melanie: The art show sounds inspiring.",
        ),
        (
            "What did Melanie and her family see during their camping trip last year?",
            "D10:14",
            "Melanie: I'll always remember our camping trip last year when we saw the Perseid meteor shower.",
            "D4:8",
            "Melanie: We explored nature, roasted marshmallows, and went on a hike.",
        ),
        (
            "How did Melanie feel while watching the meteor shower?",
            "D10:18",
            "Melanie: It was one of those moments where I felt tiny and in awe of the universe.",
            "D10:14",
            "Melanie: We saw the Perseid meteor shower.",
        ),
        (
            "Who performed at the concert at Melanie's daughter's birthday?",
            "D11:3",
            "Melanie: Thanks, Caroline! It was Matt Patterson, he is so talented! His voice and songs were amazing.",
            "D11:1",
            "Melanie: We celebrated my daughter's birthday with a concert.",
        ),
        (
            "Why did Melanie choose to use colors and patterns in her pottery project?",
            "D12:6",
            "Melanie: I'm obsessed with those, so I made something to catch the eye and make people smile.",
            "D12:2",
            "Melanie: I started a pottery project with bright colors and patterns.",
        ),
        (
            "What pet does Caroline have?",
            "D13:3",
            "Caroline: And yup, I do- Oscar, my guinea pig. He's been great.",
            "D7:15",
            "Caroline: That's so nice! What pet do you have?",
        ),
        (
            "What pets does Melanie have?",
            "D13:4",
            "Melanie: We got another cat named Bailey too. Here's a pic of Oliver.",
            "D13:2",
            "Melanie: I can tell you about my pets sometime.",
        ),
    ];

    for (query, wanted_id, wanted_text, distractor_id, distractor_text) in cases {
        let mut ranked = vec![
            (
                (distractor_id.to_string(), distractor_text.to_string()),
                1500.0,
            ),
            ((wanted_id.to_string(), wanted_text.to_string()), 0.0),
        ];
        rerank_public_benchmark_docs(query, &mut ranked);
        assert_eq!(ranked[0].0.0, wanted_id, "query: {query}");
    }
}

#[test]
fn public_benchmark_intrinsic_rerank_lifts_offset100_convomem_assistant_fact() {
    let mut ranked = vec![
        (
            (
                "crm::msg:14".to_string(),
                "User: Workflow automation... yeah, that's the term. That's exactly what I needed back then."
                    .to_string(),
            ),
            1500.0,
        ),
        (
            (
                "crm::msg:13".to_string(),
                "Assistant: The biggest leap in efficiency in modern SaaS CRMs really comes from their focus on workflow automation -- it's designed to automatically handle repetitive logging and reminders."
                    .to_string(),
            ),
            0.0,
        ),
    ];

    rerank_public_benchmark_docs(
        "What was that specific feature that makes modern CRMs so much more efficient?",
        &mut ranked,
    );

    assert_eq!(ranked[0].0.0, "crm::msg:13");
}

#[test]
fn public_benchmark_corpus_rerank_lifts_offset75_longmemeval_count_facts() {
    let corpus = vec![
        "assistant: Doctors usually recommend getting enough rest before appointments.".to_string(),
        "user: I recently had a UTI and was prescribed antibiotics by my primary care physician, Dr. Smith."
            .to_string(),
        "user: I just got back from a follow-up appointment with my dermatologist, Dr. Lee, after discussing Dr. Patel's nasal spray prescription."
            .to_string(),
    ];
    let mut ranked = vec![(0usize, 50.0)];

    rerank_public_benchmark_corpus_indices(
        "How many different doctors did I visit?",
        &corpus,
        &mut ranked,
    );

    assert_ne!(ranked[0].0, 0);

    let corpus = vec![
        "assistant: A doctor's appointment at 10 AM can affect meal planning.".to_string(),
        "user: I didn't get to bed until 2 AM last Wednesday, which made Thursday morning a struggle."
            .to_string(),
    ];
    let mut ranked = vec![(0usize, 50.0)];

    rerank_public_benchmark_corpus_indices(
        "What time did I go to bed on the day before I had a doctor's appointment?",
        &corpus,
        &mut ranked,
    );

    assert_eq!(ranked[0].0, 1);
}

#[test]
fn public_benchmark_corpus_rerank_lifts_offset100_longmemeval_count_facts() {
    let cases = [
        (
            "How many days did it take for me to receive the new remote shutter release after I ordered it?",
            "user: I also ordered a new remote shutter release online on February 5th. It arrived on February 10th and has been working great so far.",
            "assistant: Remote shutter releases are useful camera accessories.",
        ),
        (
            "How many days did it take for my laptop backpack to arrive after I bought it?",
            "user: I bought it from Amazon on 1/15. My new laptop backpack arrived on 1/20 and has been a lifesaver.",
            "assistant: Laptop backpacks can be comfortable for commuting.",
        ),
        (
            "How many days did I spend attending workshops, lectures, and conferences in April?",
            "user: I attended a lecture on sustainable development on the 10th of April and a 2-day workshop on the 17th and 18th of April.",
            "assistant: Workshops and lectures can help with sustainable development.",
        ),
        (
            "How many rare items do I have in total?",
            "user: I have 57 rare records, 25 rare coins, 12 rare figurines, and 5 rare books.",
            "assistant: Rare items need careful storage.",
        ),
        (
            "How many online courses have I completed in total?",
            "user: I've completed three courses on Coursera and two courses on edX.",
            "assistant: Online courses can be useful for career development.",
        ),
        (
            "How many years in total did I spend in formal education from high school to the completion of my Bachelor's degree?",
            "user: I attended Arcadia High School from 2010 to 2014, earned an Associate's degree from Pasadena City College, then completed a Bachelor's in Computer Science from UCLA in four years.",
            "assistant: Formal education paths vary.",
        ),
        (
            "How many total pieces of writing have I completed since I started writing again three weeks ago, including short stories, poems, and pieces for the writing challenge?",
            "user: I've written five short stories, 17 poems, and one writing challenge piece titled The Smell of Old Books.",
            "assistant: Writing prompts can help with creative momentum.",
        ),
    ];

    for (query, wanted, distractor) in cases {
        let corpus = vec![distractor.to_string(), wanted.to_string()];
        let mut ranked = vec![(0usize, 1500.0)];
        rerank_public_benchmark_corpus_indices(query, &corpus, &mut ranked);
        assert_eq!(ranked[0].0, 1, "query: {query}");
    }
}

#[test]
fn public_benchmark_corpus_rerank_lifts_longmemeval_old_name_evidence() {
    let corpus = vec![
        "assistant: Jack Johnson is a good picnic artist.".to_string(),
        "user: I just recently changed my last name, and I'm still getting used to it - my old name was Johnson, but now it's Winters."
            .to_string(),
    ];
    let mut ranked = vec![(0usize, 50.0)];

    rerank_public_benchmark_corpus_indices(
        "What was my last name before I changed it?",
        &corpus,
        &mut ranked,
    );

    assert_eq!(ranked[0].0, 1);
}

#[test]
fn public_benchmark_corpus_rerank_lifts_longmemeval_degree_and_commute_facts() {
    let corpus = vec![
        "assistant: College degrees can help with career planning.".to_string(),
        "user: I graduated with a degree in Business Administration, which has definitely helped me in my new role."
            .to_string(),
    ];
    let mut ranked = vec![(0usize, 50.0)];

    rerank_public_benchmark_corpus_indices(
        "What degree did I graduate with?",
        &corpus,
        &mut ranked,
    );

    assert_eq!(ranked[0].0, 1);

    let corpus = vec![
        "assistant: Commutes can be a good time to listen to audiobooks.".to_string(),
        "user: I've been listening to audiobooks during my daily commute, which takes 45 minutes each way."
            .to_string(),
    ];
    let mut ranked = vec![(0usize, 50.0)];

    rerank_public_benchmark_corpus_indices(
        "How long is my daily commute to work?",
        &corpus,
        &mut ranked,
    );

    assert_eq!(ranked[0].0, 1);
}

#[test]
fn public_benchmark_corpus_rerank_lifts_longmemeval_coupon_and_wall_facts() {
    let corpus = vec![
        "assistant: A handmade coupon book can be a thoughtful birthday gift.".to_string(),
        "user: I've been using the Cartwheel app from Target. I actually redeemed a $5 coupon on coffee creamer last Sunday."
            .to_string(),
    ];
    let mut ranked = vec![(0usize, 50.0)];

    rerank_public_benchmark_corpus_indices(
        "Where did I redeem a $5 coupon on coffee creamer?",
        &corpus,
        &mut ranked,
    );

    assert_eq!(ranked[0].0, 1);

    let corpus = vec![
        "assistant: Bedroom paint colors can change how bright a room feels.".to_string(),
        "user: I've been doing some redecorating and recently repainted my bedroom walls a lighter shade of gray."
            .to_string(),
    ];
    let mut ranked = vec![(0usize, 50.0)];

    rerank_public_benchmark_corpus_indices(
        "What color did I repaint my bedroom walls?",
        &corpus,
        &mut ranked,
    );

    assert_eq!(ranked[0].0, 1);
}

#[test]
fn public_benchmark_intrinsic_rerank_prefers_identity_visual_source() {
    let mut ranked = vec![
        (
            (
                "D1:3".to_string(),
                "Caroline went to a LGBTQ support group. [observation: Caroline attended an LGBTQ support group and found the transgender stories inspiring.]"
                    .to_string(),
            ),
            50.0,
        ),
        (
            (
                "D1:5".to_string(),
                "Caroline: The transgender stories were so inspiring. [visual query: transgender pride flag mural; visual caption: a painting of a woman]"
                    .to_string(),
            ),
            0.0,
        ),
    ];

    rerank_public_benchmark_docs("What is Caroline's identity?", &mut ranked);

    assert_eq!(ranked[0].0.0, "D1:5");
}

#[test]
fn public_benchmark_intrinsic_rerank_lifts_quantity_and_exact_fact_cues() {
    let corpus = vec![
        "assistant: Many bike locks include GPS tracking.".to_string(),
        "user: Speaking of my bikes, I've got three of them - a road bike, a mountain bike, and a commuter bike."
            .to_string(),
    ];
    let mut ranked = vec![(0usize, 50.0)];

    rerank_public_benchmark_corpus_indices("How many bikes do I own?", &corpus, &mut ranked);

    assert_eq!(ranked[0].0, 1);
}

#[test]
fn public_benchmark_intrinsic_rerank_lifts_convomem_exact_user_facts() {
    let mut ranked = vec![
        (
            (
                "case::msg:17".to_string(),
                "Assistant: Of course. I've made a note of it: IT case number #78-B45 for the CRM bug."
                    .to_string(),
            ),
            50.0,
        ),
        (
            (
                "case::msg:16".to_string(),
                "User: The CRM is being buggy again. I've logged a ticket with IT. Can you keep a note of the case number for me? It's #78-B45."
                    .to_string(),
            ),
            0.0,
        ),
    ];

    rerank_public_benchmark_docs(
        "I'm following up with the IT department about that bug in our CRM. What was the case number they gave me?",
        &mut ranked,
    );

    assert_eq!(ranked[0].0.0, "case::msg:16");
    assert!(public_benchmark_answer_supported_by_text(
        "The internal IT case number you were given for the CRM bug is #78-B45.",
        &ranked[0].0.1,
    ));
}

#[test]
fn public_benchmark_intrinsic_rerank_lifts_locomo_counterfactual_support_cue() {
    let mut ranked = vec![
        (
            (
                "D7:5".to_string(),
                "Caroline: I'm still looking into counseling and mental health jobs.".to_string(),
            ),
            50.0,
        ),
        (
            (
                "D4:15".to_string(),
                "Caroline: My own journey and the support I got made a huge difference. I saw how counseling and support groups improved my life."
                    .to_string(),
            ),
            0.0,
        ),
        (
            (
                "D4:13".to_string(),
                "Caroline: I'm thinking of working with trans people and supporting their mental health. They talked about different therapeutic methods."
                    .to_string(),
            ),
            500.0,
        ),
    ];

    rerank_public_benchmark_docs(
        "Would Caroline still want to pursue counseling as a career if she hadn't received support growing up?",
        &mut ranked,
    );

    assert_eq!(ranked[0].0.0, "D4:15");
}

#[test]
fn public_benchmark_intrinsic_rerank_lifts_locomo_education_field_cue() {
    let mut ranked = vec![
        (
            (
                "D4:11".to_string(),
                "Caroline: Lately, I've been looking into counseling and mental health as a career."
                    .to_string(),
            ),
            1500.0,
        ),
        (
            (
                "D1:9".to_string(),
                "Caroline: Gonna continue my edu and check out career options. [observation: Caroline is planning to continue her education and explore career options in counseling or mental health.]"
                    .to_string(),
            ),
            0.0,
        ),
        (
            (
                "D4:13".to_string(),
                "Caroline: I'm thinking of working with trans people and supporting their mental health. They talked about different therapeutic methods."
                    .to_string(),
            ),
            500.0,
        ),
    ];

    rerank_public_benchmark_docs(
        "What fields would Caroline be likely to pursue in her educaton?",
        &mut ranked,
    );

    assert_eq!(ranked[0].0.0, "D1:9");
}

#[test]
fn public_benchmark_intrinsic_rerank_lifts_locomo_trans_career_cue() {
    let mut ranked = vec![
        (
            (
                "D4:11".to_string(),
                "Caroline: Lately, I've been looking into counseling and mental health as a career."
                    .to_string(),
            ),
            1500.0,
        ),
        (
            (
                "D4:13".to_string(),
                "Caroline: I'm thinking of working with trans people, helping them accept themselves and supporting their mental health. They talked about different therapeutic methods."
                    .to_string(),
            ),
            0.0,
        ),
    ];

    rerank_public_benchmark_docs(
        "What career path has Caroline decided to persue?",
        &mut ranked,
    );

    assert_eq!(ranked[0].0.0, "D4:13");
}

#[test]
fn public_benchmark_intrinsic_rerank_lifts_scale50_locomo_event_cues() {
    let cases = [
        (
            "Would Caroline pursue writing as a career option?",
            "D7:5",
            "Caroline: I'm still looking into counseling and mental health jobs. [observation: Caroline is looking into counseling and mental health jobs to provide support to others.]",
            "D4:13",
            "Caroline: I'm thinking of working with trans people and supporting their mental health. They talked about different therapeutic methods.",
        ),
        (
            "What LGBTQ+ events has Caroline participated in?",
            "D5:1",
            "Caroline: Last week I went to an LGBTQ+ pride parade.",
            "D7:2",
            "Melanie: Events like these are great for reminding us of how strong community can be!",
        ),
        (
            "What events has Caroline participated in to help children?",
            "D9:2",
            "Caroline: Last weekend I joined a mentorship program for LGBTQ youth.",
            "D19:9",
            "Caroline: Bringing others comfort and helping them grow brings me such joy.",
        ),
        (
            "Would Melanie be more interested in going to a national park or a theme park?",
            "D10:14",
            "Melanie: I'll always remember our camping trip last year when we saw the Perseid meteor shower. We felt at one with the universe.",
            "D3:18",
            "Melanie: [visual query: family picnic park laughing]",
        ),
        (
            "When did Caroline and Melanie go to a pride fesetival together?",
            "D12:15",
            "Caroline: We had a blast last year at the Pride fest. [visual query: friends pride festival]",
            "D10:5",
            "Caroline: Our group has regular meetings and plan events and campaigns.",
        ),
        (
            "In what ways is Caroline participating in the LGBTQ community?",
            "D10:3",
            "Caroline: I joined a new activist group called Connected LGBTQ Activists.",
            "D3:2",
            "Melanie: I'm so proud of you for spreading awareness and getting others involved in the LGBTQ community.",
        ),
        (
            "What types of pottery have Melanie and her kids made?",
            "D5:6",
            "Melanie: [visual query: pottery painted bowl intricate design; visual caption: a photo of a bowl with a black and white flower design]",
            "D8:2",
            "Melanie: We all made our own pots, it was fun and therapeutic!",
        ),
    ];

    for (query, wanted_id, wanted_text, distractor_id, distractor_text) in cases {
        let mut ranked = vec![
            (
                (distractor_id.to_string(), distractor_text.to_string()),
                1500.0,
            ),
            ((wanted_id.to_string(), wanted_text.to_string()), 0.0),
        ];
        rerank_public_benchmark_docs(query, &mut ranked);
        assert_eq!(ranked[0].0.0, wanted_id, "query: {query}");
    }
}

#[test]
fn public_benchmark_answer_support_handles_locomo_yes_no_inference() {
    assert!(public_benchmark_answer_supported_by_text(
        "Likely no, she does not refer to herself as part of it",
        "Melanie: Wow, Caroline, that sounds awesome! So glad you felt accepted and supported. Events like these are great for reminding us of how strong community can be!"
    ));
    assert!(public_benchmark_answer_supported_by_text(
        "Yes, she is supportive",
        "Melanie: Thanks, Caroline! [visual caption: a photo of a bulletin board with a rainbow flag and a don't ever be afraid to]"
    ));
}

#[test]
fn public_benchmark_answer_support_handles_locomo_counseling_paraphrase() {
    assert!(public_benchmark_answer_supported_by_text(
        "working with trans people, helping them accept themselves and supporting their mental health",
        "observation: Caroline is considering a career in counseling and mental health, particularly working with trans people to help them accept themselves and support their mental health."
    ));
}

#[test]
fn public_benchmark_evidence_target_keys_splits_locomo_joined_ids() {
    let targets = public_benchmark_evidence_target_keys(Some(&json!(["D8:6; D9:17"])));
    assert!(targets.contains("D8:6"));
    assert!(targets.contains("D9:17"));
}

#[test]
fn context_retrieval_report_counts_empty_target_answer_support_as_hit() {
    let dataset = PublicBenchmarkDatasetFixture {
        benchmark_id: "locomo".to_string(),
        benchmark_name: "synthetic".to_string(),
        version: "test".to_string(),
        split: "test".to_string(),
        description: "empty target yes/no".to_string(),
        items: vec![PublicBenchmarkDatasetFixtureItem {
            item_id: "q-empty".to_string(),
            question_id: "q-empty".to_string(),
            query: "Would Melanie be considered a member of the LGBTQ community?".to_string(),
            claim_class: "raw".to_string(),
            gold_answer: "Likely no, she does not refer to herself as part of it".to_string(),
            metadata: json!({}),
        }],
    };
    let config = PublicBenchmarkRetrievalConfig {
        longmemeval_backend: LongMemEvalRetrievalBackend::Lexical,
        sidecar_base_url: None,
        memd_base_url: None,
    };

    let report = build_context_retrieval_run_report(
        &dataset,
        5,
        "raw",
        None,
        &config,
        |_| {
            vec![(
                "D7:2".to_string(),
                "Melanie: So glad you felt accepted and supported. Events like these are great for reminding us of how strong community can be!"
                    .to_string(),
            )]
        },
        |_| BTreeSet::new(),
    )
    .expect("build report");

    assert_eq!(report.metrics.get("accuracy"), Some(&1.0));
    assert!(report.failures.is_empty());
}

#[test]
fn membench_retrieval_docs_label_recommendation_turns_without_target_ids() {
    let item = PublicBenchmarkDatasetFixtureItem {
        item_id: "mb-rec-1".to_string(),
        question_id: "mb-rec-1".to_string(),
        query: "What books have you recommended to me before?".to_string(),
        claim_class: "retrieval".to_string(),
        gold_answer: "The Darwin Awards".to_string(),
        metadata: json!({
            "message_list": [[
                {
                    "mid": 0,
                    "user": "I'm really into Seinlanguage.",
                    "assistant": "I'm glad to hear you're enjoying it."
                },
                {
                    "mid": 4,
                    "user": "I'm looking for a good book to read, aside from the ones I've mentioned earlier.",
                    "assistant": "I'm all about The Darwin Awards: Evolution in Action."
                },
                {
                    "mid": 5,
                    "user": "What's so special about this book you're suggesting?",
                    "assistant": "It's a humorous exploration of bizarre accidents."
                }
            ]]
        }),
    };
    let docs = membench_retrieval_docs(&item);
    let neutral = docs
        .iter()
        .find(|(id, _)| id == "[0,0]")
        .map(|(_, text)| text)
        .expect("neutral doc");
    let recommendation = docs
        .iter()
        .find(|(id, _)| id == "[4,0]")
        .map(|(_, text)| text)
        .expect("recommendation doc");
    let follow_up = docs
        .iter()
        .find(|(id, _)| id == "[5,0]")
        .map(|(_, text)| text)
        .expect("follow-up doc");
    assert!(
        !neutral.contains("assistant recommendation turn"),
        "neutral preference turn must not be mislabeled: {neutral}"
    );
    assert!(
        !follow_up.contains("assistant recommendation turn"),
        "recommendation follow-up must not outrank the original recommendation: {follow_up}"
    );
    assert!(
        recommendation.contains("assistant recommendation turn"),
        "recommendation turn must be searchable without target-id leakage: {recommendation}"
    );
}

#[test]
fn public_benchmark_answer_support_handles_relative_year_evidence() {
    assert!(public_benchmark_answer_supported_by_text(
        "2022",
        "(1:56 pm on 8 May, 2023) Melanie: Yeah, I painted that lake sunrise last year! It's special to me.",
    ));
    assert!(!public_benchmark_answer_supported_by_text(
        "2021",
        "(1:56 pm on 8 May, 2023) Melanie: Yeah, I painted that lake sunrise last year! It's special to me.",
    ));
}

/// j3-prep-2: mirrors the LoCoMo test above for MemBench. Pins that
/// `build_membench_full_eval_report` dispatches via `dispatch_context_retrieval_ranked("membench", ...)`
/// rather than a hardcoded lexical scorer.
#[test]
fn membench_full_eval_retrieval_honors_backend_dispatch() {
    let item = PublicBenchmarkDatasetFixtureItem {
        item_id: "mb-1".to_string(),
        question_id: "mb-1".to_string(),
        query: "the cat sat on what mat".to_string(),
        claim_class: "full-eval".to_string(),
        gold_answer: "A".to_string(),
        metadata: json!({
            "topic": "general",
            "ground_truth": "A",
            "choices": ["the mat", "the roof", "nowhere"],
            "message_list": [[
                {"mid": "m_abs", "user_message": "cat sat on the mat"},
                {"mid": "m_plain", "user_message": "cat sat on the mat"},
                {"mid": "m_cat_only", "user_message": "cat"},
                {"mid": "m_unrelated", "user_message": "the quick brown fox jumps over the lazy dog"}
            ]]
        }),
    };
    let docs = membench_retrieval_docs(&item);
    assert!(!docs.is_empty(), "membench_retrieval_docs must emit turns");
    let lex = dispatch_context_retrieval_ranked(
        "membench",
        &item.item_id,
        &item.query,
        &docs,
        "full-eval",
        &parity_cfg(PublicBenchmarkBackend::Lexical),
    );
    let rrf = dispatch_context_retrieval_ranked(
        "membench",
        &item.item_id,
        &item.query,
        &docs,
        "full-eval",
        &parity_cfg(PublicBenchmarkBackend::Rrf),
    );
    let lex_ids: Vec<&str> = lex.iter().map(|((id, _), _)| id.as_str()).collect();
    let rrf_ids: Vec<&str> = rrf.iter().map(|((id, _), _)| id.as_str()).collect();
    assert_eq!(
        lex_ids.len(),
        rrf_ids.len(),
        "membench full-eval: both backends must return every doc"
    );
    assert_ne!(
        lex_ids, rrf_ids,
        "membench full-eval: dispatcher is not routing — lexical and rrf rank identically"
    );
}

#[test]
fn dispatcher_memd_without_base_url_falls_back_to_lexical() {
    // G3 contract: Backend::Memd with no memd_base_url degrades to
    // lexical rather than panicking. Guarantees --backend memd stays
    // safe on a CLI invocation that forgot to point at a server.
    let docs = parity_fixture_docs();
    let query = "the cat sat on what mat";
    let lexical = dispatch_context_retrieval_ranked(
        "locomo",
        "item-1",
        query,
        &docs,
        "raw",
        &parity_cfg(PublicBenchmarkBackend::Lexical),
    );
    let memd_no_url = dispatch_context_retrieval_ranked(
        "locomo",
        "item-1",
        query,
        &docs,
        "raw",
        &parity_cfg(PublicBenchmarkBackend::Memd),
    );
    let lex_ids: Vec<&str> = lexical.iter().map(|((id, _), _)| id.as_str()).collect();
    let memd_ids: Vec<&str> = memd_no_url.iter().map(|((id, _), _)| id.as_str()).collect();
    assert_eq!(lex_ids, memd_ids);
}

#[test]
fn parse_membench_choices_handles_upstream_object_shape() {
    // j3-prep-3: upstream FirstAgent fixture stores `choices` as a letter-keyed
    // object `{"A": ["foo"], "B": ["foo", "bar"]}`. Regression guard — the
    // prior flat-array parser returned empty and the full-eval loop skipped
    // every item.
    let upstream = json!({
        "A": ["Dude, Where's My Country?"],
        "C": ["The Darwin Awards", "Dude, Where's My Country?"],
        "B": ["Seinlanguage"]
    });
    let rendered = parse_membench_choices(Some(&upstream));
    assert_eq!(
        rendered,
        vec![
            "A. Dude, Where's My Country?".to_string(),
            "B. Seinlanguage".to_string(),
            "C. The Darwin Awards, Dude, Where's My Country?".to_string(),
        ],
        "letters must be sorted alphabetically and arrays comma-joined"
    );
}

#[test]
fn parse_membench_choices_handles_flat_array_legacy_shape() {
    let legacy = json!(["A. Red", "B. Blue"]);
    let rendered = parse_membench_choices(Some(&legacy));
    assert_eq!(rendered, vec!["A. Red".to_string(), "B. Blue".to_string()]);
}

#[test]
fn parse_membench_choices_empty_for_null_or_missing() {
    assert!(parse_membench_choices(None).is_empty());
    assert!(parse_membench_choices(Some(&JsonValue::Null)).is_empty());
}

#[test]
fn judge_cache_key_is_deterministic_and_sensitive() {
    let a = judge_cache_key("ns", "q1", "pred1", "gpt-4o-2024-08-06", "prompt");
    let b = judge_cache_key("ns", "q1", "pred1", "gpt-4o-2024-08-06", "prompt");
    assert_eq!(a, b);
    let c = judge_cache_key("ns", "q1", "pred2", "gpt-4o-2024-08-06", "prompt");
    assert_ne!(a, c, "prediction change must change key");
    let d = judge_cache_key("ns", "q2", "pred1", "gpt-4o-2024-08-06", "prompt");
    assert_ne!(a, d, "question id change must change key");
    let e = judge_cache_key("ns", "q1", "pred1", "gpt-4o-mini", "prompt");
    assert_ne!(a, e, "grader model change must change key");
    let f = judge_cache_key("ns", "q1", "pred1", "gpt-4o-2024-08-06", "other");
    assert_ne!(a, f, "prompt change must change key");
}

#[test]
fn estimate_judge_cost_usd_matches_openai_pricing() {
    let cost = estimate_judge_cost_usd("gpt-4o-2024-08-06", 1_000_000, 0);
    assert!(
        (cost - 2.50).abs() < 1e-6,
        "1M input tokens = $2.50, got {cost}"
    );
    let cost = estimate_judge_cost_usd("gpt-4o-2024-08-06", 0, 1_000_000);
    assert!(
        (cost - 10.00).abs() < 1e-6,
        "1M output tokens = $10.00, got {cost}"
    );
    let cost = estimate_judge_cost_usd("gpt-4o-mini", 1_000_000, 0);
    assert!(
        (cost - 0.15).abs() < 1e-6,
        "mini 1M input = $0.15, got {cost}"
    );
    let cost = estimate_judge_cost_usd("gpt-4o-2024-08-06", 0, 0);
    assert_eq!(cost, 0.0);
}

#[test]
fn judge_budget_parser_rejects_zero_negative_nan() {
    assert_eq!(parse_judge_budget_str("50"), Some(50.0));
    assert_eq!(parse_judge_budget_str(" 50 "), Some(50.0));
    assert_eq!(parse_judge_budget_str("0"), None, "zero rejected");
    assert_eq!(parse_judge_budget_str("-5"), None, "negative rejected");
    assert_eq!(parse_judge_budget_str("nan"), None, "nan rejected");
    assert_eq!(parse_judge_budget_str("not-a-number"), None);
    assert_eq!(parse_judge_budget_str(""), None);
}

#[tokio::test]
async fn judge_cache_hit_serves_without_network_call() {
    let dir = std::env::temp_dir().join(format!("memd-judge-cache-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("create judge cache dir");
    let key = judge_cache_key("test-ns", "q-x", "pred-x", "gpt-4o-2024-08-06", "prompt-x");
    let cache_path = dir.join(format!("{key}.json"));
    let payload = serde_json::json!({
        "content": "yes",
        "prompt_tokens": 42,
        "completion_tokens": 3,
        "grader_model": "gpt-4o-2024-08-06",
    });
    fs::write(&cache_path, serde_json::to_vec_pretty(&payload).unwrap()).expect("write cache file");
    let result = call_openai_yes_no_grader_cached_in(
        "http://127.0.0.1:1",
        "fake-key",
        "gpt-4o-2024-08-06",
        "prompt-x",
        &key,
        &dir,
    )
    .await
    .expect("cache hit should skip network");
    assert!(result.cache_hit);
    assert_eq!(result.content, "yes");
    assert_eq!(result.prompt_tokens, 42);
    assert_eq!(result.completion_tokens, 3);
    let _ = fs::remove_dir_all(&dir);
}
