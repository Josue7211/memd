use super::*;

#[test]
fn write_benchmark_registry_docs_writes_expected_markdown_outputs() {
    let dir = std::env::temp_dir().join(format!(
        "memd-benchmark-registry-docs-{}",
        uuid::Uuid::new_v4()
    ));
    let repo_root = dir.join("repo");
    fs::create_dir_all(repo_root.join(".git")).expect("create git dir");
    write_test_benchmark_registry(&repo_root);
    let registry = test_benchmark_registry();
    let benchmark = test_feature_benchmark_report(&repo_root.join(".memd"));

    let report = build_benchmark_registry_docs_report(&repo_root, &registry, &benchmark);
    assert!(
        report
            .benchmarks_markdown
            .contains("# memd benchmark registry")
    );
    assert!(report.loops_markdown.contains("# memd benchmark loops"));
    assert!(report.loops_markdown.contains("Coverage Gaps"));
    assert!(
        report
            .coverage_markdown
            .contains("# memd benchmark coverage")
    );
    assert!(report.coverage_markdown.contains("Coverage Summary"));
    assert!(report.coverage_markdown.contains("Benchmark Gaps"));
    assert!(report.scores_markdown.contains("# memd benchmark scores"));
    assert!(report._comparative_report.is_some());
    assert!(report.scores_markdown.contains("Comparative Evidence"));

    write_benchmark_registry_docs(&repo_root, &registry, &benchmark)
        .expect("write benchmark registry docs");
    assert!(benchmark_registry_markdown_path(&repo_root, "BENCHMARKS.md").exists());
    assert!(benchmark_registry_markdown_path(&repo_root, "LOOPS.md").exists());
    assert!(benchmark_registry_markdown_path(&repo_root, "COVERAGE.md").exists());
    assert!(benchmark_registry_markdown_path(&repo_root, "SCORES.md").exists());
    assert!(benchmark_registry_markdown_path(&repo_root, "MORNING.md").exists());
    assert!(
        benchmark_telemetry_dir(Path::new(&benchmark.bundle_root))
            .join("latest.json")
            .exists()
    );
    assert!(
        benchmark_telemetry_dir(Path::new(&benchmark.bundle_root))
            .join("latest.md")
            .exists()
    );

    let benchmarks_md = fs::read_to_string(benchmark_registry_markdown_path(
        &repo_root,
        "BENCHMARKS.md",
    ))
    .expect("read benchmarks md");
    assert!(benchmarks_md.contains("Current benchmark score"));
    let morning_md = fs::read_to_string(benchmark_registry_markdown_path(&repo_root, "MORNING.md"))
        .expect("read morning md");
    assert!(morning_md.contains("# memd morning summary"));
    assert!(morning_md.contains("Continuity Failures"));
    assert!(morning_md.contains("Verification Regressions"));

    fs::remove_dir_all(dir).expect("cleanup benchmark registry docs dir");
}

#[test]
fn build_benchmark_gap_candidates_surfaces_unbenchmarked_continuity_feature() {
    let mut registry = test_benchmark_registry();
    registry.features = vec![BenchmarkFeatureRecord {
        id: "feature.session_continuity".to_string(),
        name: "Session Continuity".to_string(),
        pillar: "memory-continuity".to_string(),
        family: "bundle-runtime".to_string(),
        tier: "tier-0-continuity-critical".to_string(),
        continuity_critical: true,
        user_contract: "resume restores continuity".to_string(),
        source_contract_refs: Vec::new(),
        commands: vec!["memd resume".to_string()],
        routes: Vec::new(),
        files: vec!["crates/memd-client/src/main.rs".to_string()],
        journey_ids: Vec::new(),
        loop_ids: Vec::new(),
        quality_dimensions: vec!["continuity".to_string()],
        drift_risks: vec!["continuity-drift".to_string()],
        failure_modes: vec!["resume misses task state".to_string()],
        coverage_status: "unbenchmarked".to_string(),
        last_verified_at: None,
    }];

    let gaps = build_benchmark_gap_candidates(&registry);
    assert!(
        gaps.iter()
            .any(|gap| gap.id == "benchmark:unbenchmarked_continuity_feature")
    );
}

#[test]
fn build_telemetry_benchmark_coverage_surfaces_registry_gaps() {
    let dir =
        std::env::temp_dir().join(format!("memd-benchmark-telemetry-{}", uuid::Uuid::new_v4()));
    let repo_root = dir.join("repo");
    let output = repo_root.join(".memd");
    fs::create_dir_all(repo_root.join(".git")).expect("create git dir");
    fs::create_dir_all(feature_benchmark_reports_dir(&output)).expect("create benchmark dir");
    write_test_benchmark_registry(&repo_root);
    let report = test_feature_benchmark_report(&output);
    fs::write(
        feature_benchmark_reports_dir(&output).join("latest.json"),
        serde_json::to_string_pretty(&report).expect("serialize report") + "\n",
    )
    .expect("write benchmark report");

    let coverage = build_telemetry_benchmark_coverage(&output)
        .expect("build telemetry coverage")
        .expect("telemetry coverage");
    assert_eq!(coverage.continuity_critical_total, 13);
    assert_eq!(coverage.continuity_critical_benchmarked, 1);
    assert_eq!(coverage.missing_loop_count, 12);
    assert!(
        coverage
            .gap_candidates
            .iter()
            .any(|gap| gap.id == "benchmark:unbenchmarked_continuity_feature")
    );

    fs::remove_dir_all(dir).expect("cleanup telemetry dir");
}

#[test]
fn render_morning_operator_summary_surfaces_top_regressions() {
    let summary = render_morning_operator_summary(&MorningOperatorSummary {
        current_benchmark_score: 88,
        current_benchmark_max_score: 100,
        top_continuity_failures: vec!["resume continuity drift".to_string()],
        top_verification_regressions: vec![
            "verifier.feature.session_continuity status=failing gate=fragile".to_string(),
        ],
        top_verification_pressure: vec![
            "verifier.feature.hive.messages-send-ack status=passing gate=acceptable target=acceptable continuity_critical=true".to_string(),
        ],
        top_drift_risks: vec!["surface drift in mem.md".to_string()],
        top_token_regressions: vec!["handoff packet +420 tokens".to_string()],
        top_no_memd_losses: vec!["resume still loses to no-memd baseline".to_string()],
        proposed_next_actions: vec!["fix resume journey before expanding registry".to_string()],
    });

    assert!(summary.contains("resume continuity drift"));
    assert!(summary.contains("Verification Regressions"));
    assert!(summary.contains("Verification Pressure"));
    assert!(summary.contains("verifier.feature.session_continuity status=failing gate=fragile"));
    assert!(
        summary.contains("verifier.feature.hive.messages-send-ack status=passing gate=acceptable")
    );
    assert!(summary.contains("handoff packet +420 tokens"));
    assert!(summary.contains("fix resume journey before expanding registry"));
    assert!(summary.contains("# memd morning summary"));
}

#[test]
fn build_morning_operator_summary_surfaces_acceptable_continuity_verifiers() {
    let registry = test_benchmark_registry();
    let benchmark = test_feature_benchmark_report(Path::new(".memd"));
    let verification_report = VerifySweepReport {
        lane: "nightly".to_string(),
        ok: true,
        total: 10,
        passed: 10,
        failures: Vec::new(),
        runs: vec![
            VerifierRunRecord {
                verifier_id: "verifier.journey.resume-handoff-attach".to_string(),
                status: "passing".to_string(),
                gate_result: "acceptable".to_string(),
                evidence_ids: vec![
                    "evidence:verifier.journey.resume-handoff-attach:latest".to_string(),
                ],
                metrics_observed: BTreeMap::new(),
            },
            VerifierRunRecord {
                verifier_id: "verifier.feature.hive.messages-send-ack".to_string(),
                status: "passing".to_string(),
                gate_result: "acceptable".to_string(),
                evidence_ids: vec![
                    "evidence:verifier.feature.hive.messages-send-ack:latest".to_string(),
                ],
                metrics_observed: BTreeMap::new(),
            },
        ],
        bundle_root: ".memd".to_string(),
        repo_root: None,
    };

    let summary = build_morning_operator_summary(
        &registry,
        &benchmark,
        None,
        None,
        Some(&verification_report),
    );

    assert!(summary.top_verification_regressions.iter().any(|item| {
        item.contains("verifier.journey.resume-handoff-attach") && item.contains("target=strong")
    }));
    assert!(
        !summary
            .top_verification_regressions
            .iter()
            .any(|item| item.contains("nightly verify lane nightly is green"))
    );
    let journey_index = summary
        .top_verification_regressions
        .iter()
        .position(|item| item.contains("verifier.journey.resume-handoff-attach"))
        .expect("journey verifier should be ranked");
    assert_eq!(journey_index, 0);
    assert!(
        summary
            .proposed_next_actions
            .iter()
            .any(|item| item.contains("upgrade verifier gates with highest target pressure"))
    );
    assert!(
        summary
            .proposed_next_actions
            .iter()
            .any(|item| item.contains("verifier.journey.resume-handoff-attach"))
    );
    assert!(
        !summary
            .proposed_next_actions
            .iter()
            .any(|item| item.contains("verifier.feature.hive.messages-send-ack"))
    );
    assert!(
        !summary
            .top_verification_regressions
            .iter()
            .any(|item| item.contains("verifier.feature.hive.messages-send-ack"))
    );
    assert!(
        summary
            .top_verification_pressure
            .iter()
            .any(|item| item.contains("verifier.feature.hive.messages-send-ack"))
    );
}

#[test]
fn build_no_memd_delta_report_surfaces_token_and_reconstruction_improvement() {
    let report = build_no_memd_delta_report(
        &BaselineMetrics {
            prompt_tokens: 2200,
            reread_count: 5,
            reconstruction_steps: 4,
        },
        &BaselineMetrics {
            prompt_tokens: 1200,
            reread_count: 2,
            reconstruction_steps: 1,
        },
    );

    assert_eq!(report.token_delta, 1000);
    assert_eq!(report.reread_delta, 3);
    assert_eq!(report.reconstruction_delta, 3);
    assert!(report.with_memd_better);
}

#[test]
fn continuity_failure_caps_gate_at_fragile() {
    let scorecard = resolve_benchmark_scorecard(
        &BenchmarkSubjectMetrics {
            correctness: 92,
            continuity: 35,
            reliability: 88,
            token_efficiency: 70,
            no_memd_delta: Some(12),
        },
        &BenchmarkEvidenceSummary {
            has_contract_evidence: true,
            has_workflow_evidence: true,
            has_continuity_evidence: false,
            has_comparative_evidence: true,
            has_drift_failure: false,
        },
        true,
    );

    assert_eq!(scorecard.gate, "fragile");
}

#[test]
fn no_memd_loss_caps_feature_at_acceptable() {
    let scorecard = resolve_benchmark_scorecard(
        &BenchmarkSubjectMetrics {
            correctness: 95,
            continuity: 90,
            reliability: 90,
            token_efficiency: 65,
            no_memd_delta: Some(-4),
        },
        &BenchmarkEvidenceSummary {
            has_contract_evidence: true,
            has_workflow_evidence: true,
            has_continuity_evidence: true,
            has_comparative_evidence: true,
            has_drift_failure: false,
        },
        true,
    );

    assert_eq!(scorecard.gate, "acceptable");
}

#[test]
fn write_continuity_journey_artifacts_writes_expected_outputs() {
    let dir = std::env::temp_dir().join(format!(
        "memd-continuity-journey-artifacts-{}",
        uuid::Uuid::new_v4()
    ));
    let output = dir.join(".memd");
    fs::create_dir_all(&output).expect("create output dir");
    let report = ContinuityJourneyReport {
        journey_id: "journey.continuity.resume-handoff-attach".to_string(),
        journey_name: "Resume To Handoff To Attach".to_string(),
        gate_decision: BenchmarkGateDecision {
            gate: "acceptable".to_string(),
            resolved_score: 75,
            reasons: vec!["continuity evidence present".to_string()],
        },
        metrics: BenchmarkSubjectMetrics {
            correctness: 90,
            continuity: 85,
            reliability: 80,
            token_efficiency: 78,
            no_memd_delta: Some(9),
        },
        evidence: BenchmarkEvidenceSummary {
            has_contract_evidence: true,
            has_workflow_evidence: true,
            has_continuity_evidence: true,
            has_comparative_evidence: true,
            has_drift_failure: false,
        },
        baseline_modes: vec![
            "baseline.no-memd".to_string(),
            "baseline.with-memd".to_string(),
        ],
        feature_ids: vec![
            "feature.session_continuity".to_string(),
            "feature.bundle.handoff".to_string(),
        ],
        artifact_paths: Vec::new(),
        summary: "resume continuity evidence".to_string(),
        generated_at: Some(Utc::now()),
    };

    write_continuity_journey_artifacts(&output, &report)
        .expect("write continuity journey artifacts");
    let continuity_dir = benchmark_telemetry_dir(&output);
    assert!(continuity_dir.join("latest.json").exists());
    assert!(continuity_dir.join("latest.md").exists());

    let markdown =
        fs::read_to_string(continuity_dir.join("latest.md")).expect("read continuity markdown");
    assert!(markdown.contains("Resume To Handoff To Attach"));
    assert!(markdown.contains("Gate: `acceptable`"));

    fs::remove_dir_all(dir).expect("cleanup continuity journey artifacts dir");
}
