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
    // a full top list (≥5 items), lexical fallback is dilution, not rescue.
    // 60Q probe decomposition showed 0 lexical rescues across 30 pref + 30
    // cross-type samples; 7 pref misses had server rank ≤5 but merged rank
    // ≥6 because lexical dragged unrelated items into the top.
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
    let leaderboard = fs::read_to_string(public_benchmark_leaderboard_docs_path(&docs_root))
        .expect("read public leaderboard docs");
    assert!(leaderboard.contains("# memd public leaderboard"));
    assert!(leaderboard.contains("fixture-backed"));
    assert!(leaderboard.contains("fixture-only"));
    assert!(
        leaderboard.contains("declared parity targets: longmemeval, locomo, convomem, membench")
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

    let leaderboard = fs::read_to_string(public_benchmark_leaderboard_docs_path(&docs_root))
        .expect("read public leaderboard docs");
    assert!(leaderboard.contains("| LongMemEval |"));
    assert!(leaderboard.contains("| LoCoMo |"));
    assert!(leaderboard.contains("| ConvoMem |"));
    assert!(leaderboard.contains("| MemBench |"));
    assert!(leaderboard.contains("| Verification |"));
    assert!(leaderboard.contains("| MemPalace |"));
    assert!(leaderboard.contains("| Commit |"));
    assert!(leaderboard.contains("| Rerun |"));

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

    for dataset in ["longmemeval", "locomo", "convomem", "membench"] {
        let source = public_benchmark_dataset_source(dataset).expect("catalog entry");
        let cache_path =
            public_benchmark_dataset_cache_path(&output, dataset, source.default_filename);
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent).expect("create dataset cache dir");
        }
        let fixture_path = match dataset {
            "convomem" => PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(".memd/benchmarks/datasets/convomem/convomem-evidence-sample.json"),
            "membench" => PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(".memd/benchmarks/datasets/membench/membench-firstagent.json"),
            _ => public_benchmark_fixture_path(dataset),
        };
        fs::copy(fixture_path, &cache_path).expect("seed cached fixture");
    }

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
                dataset_root: None,
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

    let leaderboard = fs::read_to_string(public_benchmark_leaderboard_docs_path(&docs_root))
        .expect("read public leaderboard docs");
    assert!(leaderboard.contains("| ConvoMem |"));
    assert!(leaderboard.contains("| MemBench |"));

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

fn parity_ranked_ids(
    bench_id: &str,
    backend: PublicBenchmarkBackend,
    query: &str,
) -> Vec<String> {
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
    assert!((cost - 2.50).abs() < 1e-6, "1M input tokens = $2.50, got {cost}");
    let cost = estimate_judge_cost_usd("gpt-4o-2024-08-06", 0, 1_000_000);
    assert!((cost - 10.00).abs() < 1e-6, "1M output tokens = $10.00, got {cost}");
    let cost = estimate_judge_cost_usd("gpt-4o-mini", 1_000_000, 0);
    assert!((cost - 0.15).abs() < 1e-6, "mini 1M input = $0.15, got {cost}");
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
    fs::write(&cache_path, serde_json::to_vec_pretty(&payload).unwrap())
        .expect("write cache file");
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
