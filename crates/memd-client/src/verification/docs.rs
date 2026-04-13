use super::*;

#[derive(Debug, Clone)]
pub(crate) struct BenchmarkRegistryDocsReport {
    _repo_root: PathBuf,
    _registry_path: PathBuf,
    _registry: BenchmarkRegistry,
    pub(crate) _comparative_report: Option<NoMemdDeltaReport>,
    pub(crate) benchmarks_markdown: String,
    pub(crate) loops_markdown: String,
    pub(crate) coverage_markdown: String,
    pub(crate) scores_markdown: String,
    morning_markdown: String,
    continuity_journey_report: Option<ContinuityJourneyReport>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MorningOperatorSummary {
    pub(crate) current_benchmark_score: u8,
    pub(crate) current_benchmark_max_score: u8,
    pub(crate) top_continuity_failures: Vec<String>,
    pub(crate) top_verification_regressions: Vec<String>,
    pub(crate) top_verification_pressure: Vec<String>,
    pub(crate) top_drift_risks: Vec<String>,
    pub(crate) top_token_regressions: Vec<String>,
    pub(crate) top_no_memd_losses: Vec<String>,
    pub(crate) proposed_next_actions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct BaselineMetrics {
    pub(crate) prompt_tokens: usize,
    pub(crate) reread_count: usize,
    pub(crate) reconstruction_steps: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct NoMemdDeltaReport {
    pub(crate) no_memd: BaselineMetrics,
    pub(crate) with_memd: BaselineMetrics,
    pub(crate) token_delta: isize,
    pub(crate) reread_delta: isize,
    pub(crate) reconstruction_delta: isize,
    pub(crate) with_memd_better: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct RankedVerifierPressure {
    severity: u8,
    verifier_id: String,
    below_target: bool,
    summary: String,
}

pub(crate) fn build_benchmark_registry_docs_report(
    repo_root: &Path,
    registry: &BenchmarkRegistry,
    benchmark: &FeatureBenchmarkReport,
) -> BenchmarkRegistryDocsReport {
    let registry_path = benchmark_registry_json_path(repo_root);
    let benchmarks_markdown =
        render_benchmark_registry_benchmarks_markdown(repo_root, registry, benchmark);
    let coverage_telemetry = build_benchmark_coverage_telemetry(registry, Some(benchmark));
    let loops_markdown = render_benchmark_registry_loops_markdown(registry, &coverage_telemetry);
    let coverage_markdown =
        render_benchmark_registry_coverage_markdown(registry, benchmark, &coverage_telemetry);
    let continuity_journey_report =
        build_continuity_journey_report(Path::new(&benchmark.bundle_root), registry, benchmark);
    let comparative_report = build_benchmark_comparison_report(benchmark);
    let scores_markdown = render_benchmark_registry_scores_markdown(
        registry,
        benchmark,
        continuity_journey_report.as_ref(),
        comparative_report.as_ref(),
    );
    let verification_report = read_latest_verify_sweep_report(Path::new(&benchmark.bundle_root));
    let morning_summary = build_morning_operator_summary(
        registry,
        benchmark,
        comparative_report.as_ref(),
        continuity_journey_report.as_ref(),
        verification_report.as_ref(),
    );
    let morning_markdown = render_morning_operator_summary(&morning_summary);

    BenchmarkRegistryDocsReport {
        _repo_root: repo_root.to_path_buf(),
        _registry_path: registry_path,
        _registry: registry.clone(),
        _comparative_report: comparative_report,
        benchmarks_markdown,
        loops_markdown,
        coverage_markdown,
        scores_markdown,
        morning_markdown,
        continuity_journey_report,
    }
}

pub(crate) fn write_benchmark_registry_docs(
    repo_root: &Path,
    registry: &BenchmarkRegistry,
    benchmark: &FeatureBenchmarkReport,
) -> anyhow::Result<()> {
    let report = build_benchmark_registry_docs_report(repo_root, registry, benchmark);
    let verification_dir = benchmark_registry_docs_dir(repo_root);
    fs::create_dir_all(&verification_dir)
        .with_context(|| format!("create {}", verification_dir.display()))?;

    let benchmarks_path = benchmark_registry_markdown_path(repo_root, "BENCHMARKS.md");
    let loops_path = benchmark_registry_markdown_path(repo_root, "LOOPS.md");
    let coverage_path = benchmark_registry_markdown_path(repo_root, "COVERAGE.md");
    let scores_path = benchmark_registry_markdown_path(repo_root, "SCORES.md");

    fs::write(&benchmarks_path, &report.benchmarks_markdown)
        .with_context(|| format!("write {}", benchmarks_path.display()))?;
    fs::write(&loops_path, &report.loops_markdown)
        .with_context(|| format!("write {}", loops_path.display()))?;
    fs::write(&coverage_path, &report.coverage_markdown)
        .with_context(|| format!("write {}", coverage_path.display()))?;
    fs::write(&scores_path, &report.scores_markdown)
        .with_context(|| format!("write {}", scores_path.display()))?;
    let morning_path = benchmark_registry_markdown_path(repo_root, "MORNING.md");
    fs::write(&morning_path, &report.morning_markdown)
        .with_context(|| format!("write {}", morning_path.display()))?;
    if let Some(continuity_journey_report) = report.continuity_journey_report.as_ref() {
        write_continuity_journey_artifacts(
            Path::new(&benchmark.bundle_root),
            continuity_journey_report,
        )?;
    }
    Ok(())
}

pub(crate) fn build_morning_operator_summary(
    registry: &BenchmarkRegistry,
    benchmark: &FeatureBenchmarkReport,
    comparative_report: Option<&NoMemdDeltaReport>,
    continuity_journey_report: Option<&ContinuityJourneyReport>,
    verification_report: Option<&VerifySweepReport>,
) -> MorningOperatorSummary {
    let mut top_continuity_failures = registry
        .features
        .iter()
        .filter(|feature| feature.continuity_critical && feature.coverage_status != "verified")
        .map(|feature| {
            format!(
                "{} [{}] coverage={} drift={}",
                feature.id,
                feature.family,
                feature.coverage_status,
                feature.drift_risks.join("|")
            )
        })
        .collect::<Vec<_>>();
    if top_continuity_failures.is_empty() {
        if let Some(journey) = continuity_journey_report {
            top_continuity_failures.push(format!(
                "{} gate={} score={}",
                journey.journey_id,
                journey.gate_decision.gate,
                journey.gate_decision.resolved_score
            ));
        } else {
            top_continuity_failures
                .push("no continuity-critical benchmark gaps detected".to_string());
        }
    }
    top_continuity_failures.truncate(5);

    let mut top_verification_regressions = verification_report
        .map(|report| {
            let ranked_runs = collect_ranked_verifier_pressure(registry, report);
            let mut items = ranked_runs
                .into_iter()
                .filter(|entry| entry.below_target || entry.severity >= 4)
                .map(|entry| entry.summary)
                .collect::<Vec<_>>();
            if items.is_empty() && !report.failures.is_empty() {
                items = report.failures.clone();
            }
            if items.is_empty() && !report.ok {
                items.push(format!(
                    "nightly lane {} failed with {}/{} passes",
                    report.lane, report.passed, report.total
                ));
            }
            items
        })
        .unwrap_or_default();
    if top_verification_regressions.is_empty() {
        if let Some(report) = verification_report {
            top_verification_regressions.push(format!(
                "nightly verify lane {} is green at {}/{}",
                report.lane, report.passed, report.total
            ));
        } else {
            top_verification_regressions
                .push("no nightly verification report available yet".to_string());
        }
    }
    top_verification_regressions.truncate(5);

    let mut top_verification_pressure = verification_report
        .map(|report| {
            let mut items = collect_ranked_verifier_pressure(registry, report)
                .into_iter()
                .filter(|entry| !(entry.below_target || entry.severity >= 4))
                .map(|entry| entry.summary)
                .collect::<Vec<_>>();
            if items.is_empty() {
                items.push("no additional verifier pressure beyond current green lane".to_string());
            }
            items
        })
        .unwrap_or_else(|| vec!["no nightly verification report available yet".to_string()]);
    top_verification_pressure.truncate(5);

    let mut top_drift_risks = registry
        .features
        .iter()
        .filter(|feature| feature.continuity_critical)
        .flat_map(|feature| feature.drift_risks.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if top_drift_risks.is_empty() {
        top_drift_risks.push("no drift risks surfaced yet".to_string());
    }
    top_drift_risks.truncate(5);

    let mut top_token_regressions = Vec::new();
    if let Some(report) = comparative_report {
        top_token_regressions.push(format!(
            "no-memd prompt tokens={} with-memd prompt tokens={} delta={}",
            report.no_memd.prompt_tokens, report.with_memd.prompt_tokens, report.token_delta
        ));
        top_token_regressions.push(format!(
            "no-memd rereads={} with-memd rereads={} delta={}",
            report.no_memd.reread_count, report.with_memd.reread_count, report.reread_delta
        ));
    } else {
        top_token_regressions.push("no comparative token baseline available yet".to_string());
    }
    if let Some(area) = benchmark.areas.iter().find(|area| area.status != "pass") {
        top_token_regressions.push(format!(
            "{} scored {}/{} and still needs tightening",
            area.name, area.score, area.max_score
        ));
    }
    top_token_regressions.truncate(5);

    let mut top_no_memd_losses = Vec::new();
    if let Some(report) = comparative_report {
        if report.with_memd_better {
            top_no_memd_losses.push(format!(
                "with memd beats no memd by {} tokens, {} rereads, and {} reconstruction steps",
                report.token_delta, report.reread_delta, report.reconstruction_delta
            ));
        } else {
            top_no_memd_losses.push(format!(
                "with memd is not yet better than no memd: token_delta={} reread_delta={} reconstruction_delta={}",
                report.token_delta, report.reread_delta, report.reconstruction_delta
            ));
        }
    } else {
        top_no_memd_losses.push("no-memd comparison not available yet".to_string());
    }
    top_no_memd_losses.truncate(5);

    let mut proposed_next_actions = benchmark
        .recommendations.to_vec();
    if let Some(report) = verification_report {
        let ranked_verifier_pressure = collect_ranked_verifier_pressure(registry, report);
        if !report.ok {
            proposed_next_actions.insert(
                0,
                format!(
                    "fix nightly verifier regressions before expanding benchmark coverage ({}/{})",
                    report.passed, report.total
                ),
            );
        } else {
            let top_ids = ranked_verifier_pressure
                .iter()
                .filter(|entry| entry.below_target)
                .take(3)
                .map(|entry| entry.verifier_id.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            if !top_ids.is_empty() {
                proposed_next_actions.insert(
                    0,
                    format!("upgrade verifier gates with highest target pressure: {top_ids}"),
                );
            }
        }
    }
    if proposed_next_actions.is_empty() {
        proposed_next_actions
            .push("benchmark the remaining continuity-critical features".to_string());
    }
    proposed_next_actions.truncate(5);

    MorningOperatorSummary {
        current_benchmark_score: benchmark.score,
        current_benchmark_max_score: benchmark.max_score,
        top_continuity_failures,
        top_verification_regressions,
        top_verification_pressure,
        top_drift_risks,
        top_token_regressions,
        top_no_memd_losses,
        proposed_next_actions,
    }
}

pub(crate) fn collect_ranked_verifier_pressure(
    registry: &BenchmarkRegistry,
    report: &VerifySweepReport,
) -> Vec<RankedVerifierPressure> {
    let mut ranked_runs = report
        .runs
        .iter()
        .filter_map(|run| {
            let verifier = registry
                .verifiers
                .iter()
                .find(|verifier| verifier.id == run.verifier_id)?;
            let continuity_critical = verifier
                .subject_ids
                .iter()
                .any(|subject_id| verifier_subject_is_continuity_critical(registry, subject_id));
            let actual_rank = gate_rank(&run.gate_result);
            let target_rank = gate_rank(&verifier.gate_target);
            let severity =
                verifier_run_morning_severity(run, &verifier.gate_target, continuity_critical);
            (severity > 0).then(|| RankedVerifierPressure {
                severity,
                verifier_id: run.verifier_id.clone(),
                below_target: actual_rank < target_rank,
                summary: format!(
                    "{} status={} gate={} target={} continuity_critical={}",
                    run.verifier_id,
                    run.status,
                    run.gate_result,
                    verifier.gate_target,
                    continuity_critical
                ),
            })
        })
        .collect::<Vec<_>>();
    ranked_runs.sort_by(|left, right| {
        right
            .severity
            .cmp(&left.severity)
            .then_with(|| left.summary.cmp(&right.summary))
    });
    ranked_runs
}

pub(crate) fn verifier_subject_is_continuity_critical(
    registry: &BenchmarkRegistry,
    subject_id: &str,
) -> bool {
    if registry
        .features
        .iter()
        .find(|feature| feature.id == subject_id)
        .is_some_and(|feature| feature.continuity_critical)
    {
        return true;
    }

    registry
        .journeys
        .iter()
        .find(|journey| journey.id == subject_id)
        .is_some_and(|journey| {
            journey.feature_ids.iter().any(|feature_id| {
                registry
                    .features
                    .iter()
                    .find(|feature| feature.id == *feature_id)
                    .is_some_and(|feature| feature.continuity_critical)
            })
        })
}

pub(crate) fn render_morning_operator_summary(summary: &MorningOperatorSummary) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd morning summary\n\n");
    markdown.push_str(&format!(
        "- Current benchmark score: `{}/{}`\n",
        summary.current_benchmark_score, summary.current_benchmark_max_score
    ));
    markdown.push_str("\n## Continuity Failures\n");
    for item in &summary.top_continuity_failures {
        markdown.push_str(&format!("- {}\n", item));
    }
    markdown.push_str("\n## Verification Regressions\n");
    for item in &summary.top_verification_regressions {
        markdown.push_str(&format!("- {}\n", item));
    }
    markdown.push_str("\n## Verification Pressure\n");
    for item in &summary.top_verification_pressure {
        markdown.push_str(&format!("- {}\n", item));
    }
    markdown.push_str("\n## Drift Risks\n");
    for item in &summary.top_drift_risks {
        markdown.push_str(&format!("- {}\n", item));
    }
    markdown.push_str("\n## Token Regressions\n");
    for item in &summary.top_token_regressions {
        markdown.push_str(&format!("- {}\n", item));
    }
    markdown.push_str("\n## With memd vs No memd\n");
    for item in &summary.top_no_memd_losses {
        markdown.push_str(&format!("- {}\n", item));
    }
    markdown.push_str("\n## Next Actions\n");
    for item in &summary.proposed_next_actions {
        markdown.push_str(&format!("- {}\n", item));
    }
    markdown.push('\n');
    markdown
}
