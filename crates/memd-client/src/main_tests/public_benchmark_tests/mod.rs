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
    assert_eq!(source.expected_checksum, None);
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
    assert_eq!(source.expected_checksum, None);
    assert_eq!(source.split, "FirstAgent");
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
        top_k: Some(5),
        limit: Some(2),
        dataset_root: None,
        reranker: None,
        write: false,
        json: false,
        out: output.clone(),
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
        },
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
        },
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
        top_k: Some(5),
        limit: Some(2),
        dataset_root: Some(fixture),
        reranker: None,
        write: false,
        json: false,
        out: output.clone(),
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
    assert!(leaderboard.contains("dataset-grade / retrieval-local"));
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
        top_k: Some(5),
        limit: Some(1),
        dataset_root: Some(fixture),
        reranker: Some("test-reranker".to_string()),
        write: false,
        json: false,
        out: output.clone(),
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
        top_k: Some(5),
        limit: Some(2),
        dataset_root: Some(fixture),
        reranker: None,
        write: false,
        json: false,
        out: output.clone(),
    })
    .await
    .expect("run public benchmark");

    let leaderboard_report =
        build_public_benchmark_leaderboard_report(std::slice::from_ref(&report));
    let markdown = render_public_leaderboard(&leaderboard_report);
    assert!(markdown.contains("# memd public leaderboard"));
    assert!(markdown.contains("fixture-backed"));
    assert!(markdown.contains("dataset-grade / retrieval-local"));
    assert!(markdown.contains("not a full MemPalace parity claim"));
    assert!(markdown.contains("run mode is benchmark execution mode"));
    assert!(
        markdown.contains("implemented mini adapters: longmemeval, locomo, convomem, membench")
    );
    assert!(markdown.contains("| LongMemEval | upstream | raw | raw |"));
    assert!(markdown.contains("declared parity targets: longmemeval, locomo, convomem, membench"));

    fs::remove_dir_all(dir).expect("cleanup public leaderboard dir");
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
            top_k: Some(5),
            limit: Some(2),
            dataset_root: Some(public_benchmark_fixture_path(dataset)),
            reranker: None,
            write: false,
            json: false,
            out: output.clone(),
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
        top_k: Some(5),
        limit: Some(2),
        dataset_root: Some(public_benchmark_fixture_path("locomo")),
        reranker: Some("test-reranker".to_string()),
        write: false,
        json: false,
        out: output.clone(),
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

    fs::remove_dir_all(dir).expect("cleanup public benchmark suite docs dir");
}
