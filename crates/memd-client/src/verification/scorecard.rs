use super::*;

pub(crate) fn benchmark_gate_rank(gate: &str) -> u8 {
    match gate {
        "ten-star" => 4,
        "strong" => 3,
        "acceptable" => 2,
        "fragile" => 1,
        _ => 0,
    }
}

pub(crate) fn cap_benchmark_gate(current: &str, cap: &str) -> String {
    if benchmark_gate_rank(current) > benchmark_gate_rank(cap) {
        cap.to_string()
    } else {
        current.to_string()
    }
}

pub(crate) fn gate_score(gate: &str) -> u8 {
    match gate {
        "ten-star" => 100,
        "strong" => 90,
        "acceptable" => 75,
        "fragile" => 40,
        _ => 0,
    }
}

pub(crate) fn derived_continuity_metrics(
    benchmark: &FeatureBenchmarkReport,
) -> BenchmarkSubjectMetrics {
    let area_scores = benchmark
        .areas
        .iter()
        .map(|area| area.score as u16)
        .collect::<Vec<_>>();
    let average_area_score = if area_scores.is_empty() {
        benchmark.score
    } else {
        (area_scores.iter().sum::<u16>() / area_scores.len() as u16) as u8
    };
    let continuity_score = benchmark
        .areas
        .iter()
        .find(|area| area.slug == "bundle_session" || area.slug == "core_memory")
        .map(|area| area.score)
        .unwrap_or(average_area_score);
    let reliability_score = benchmark
        .areas
        .iter()
        .map(|area| area.score)
        .min()
        .unwrap_or(benchmark.score);
    let token_efficiency_score = benchmark
        .areas
        .iter()
        .find(|area| area.slug == "retrieval_context" || area.slug == "visible_memory")
        .map(|area| area.score)
        .unwrap_or(average_area_score);

    BenchmarkSubjectMetrics {
        correctness: benchmark.score,
        continuity: continuity_score,
        reliability: reliability_score,
        token_efficiency: token_efficiency_score,
        no_memd_delta: None,
    }
}

pub(crate) fn evidence_summary_from_feature_benchmark(
    benchmark: &FeatureBenchmarkReport,
) -> BenchmarkEvidenceSummary {
    let has_contract_evidence = benchmark
        .evidence
        .iter()
        .any(|item| item.contains("benchmark_registry root="));
    let has_workflow_evidence = !benchmark.areas.is_empty() && benchmark.command_count > 0;
    let has_continuity_evidence = benchmark.memory_pages > 0
        || benchmark.event_count > 0
        || benchmark
            .evidence
            .iter()
            .any(|item| item.contains("memory_quality="));
    let has_comparative_evidence = benchmark
        .evidence
        .iter()
        .any(|item| item.contains("no_memd_delta=") || item.contains("baseline.no-memd"));
    let has_drift_failure = benchmark.areas.iter().any(|area| {
        area.status != "pass"
            && area
                .recommendations
                .iter()
                .any(|item| item.contains("drift"))
    }) || benchmark
        .recommendations
        .iter()
        .any(|item| item.contains("drift"));

    BenchmarkEvidenceSummary {
        has_contract_evidence,
        has_workflow_evidence,
        has_continuity_evidence,
        has_comparative_evidence,
        has_drift_failure,
    }
}

pub(crate) fn resolve_benchmark_scorecard(
    metrics: &BenchmarkSubjectMetrics,
    evidence: &BenchmarkEvidenceSummary,
    continuity_critical: bool,
) -> BenchmarkGateDecision {
    let mut gate = if metrics.correctness >= 95
        && metrics.continuity >= 95
        && metrics.reliability >= 90
        && metrics.token_efficiency >= 80
    {
        "ten-star"
    } else if metrics.correctness >= 90
        && metrics.continuity >= 90
        && metrics.reliability >= 85
        && metrics.token_efficiency >= 70
    {
        "strong"
    } else if metrics.correctness >= 70
        && metrics.continuity >= 70
        && metrics.reliability >= 65
        && metrics.token_efficiency >= 50
    {
        "acceptable"
    } else {
        "fragile"
    }
    .to_string();

    let mut reasons = Vec::new();
    if continuity_critical && !evidence.has_continuity_evidence {
        gate = cap_benchmark_gate(&gate, "fragile");
        reasons.push("continuity-critical subject is missing continuity evidence".to_string());
    }
    if !evidence.has_contract_evidence || !evidence.has_workflow_evidence {
        gate = cap_benchmark_gate(&gate, "fragile");
        reasons.push("contract or workflow evidence is missing".to_string());
    }
    if evidence.has_drift_failure {
        gate = cap_benchmark_gate(&gate, "fragile");
        reasons.push("drift failure detected".to_string());
    }
    if metrics.no_memd_delta.unwrap_or_default() < 0 {
        gate = cap_benchmark_gate(&gate, "acceptable");
        reasons.push("with-memd underperforms no-memd; cap at acceptable".to_string());
    }
    if continuity_critical && !evidence.has_comparative_evidence {
        reasons.push("comparative evidence not yet available".to_string());
    }

    BenchmarkGateDecision {
        resolved_score: gate_score(&gate),
        gate,
        reasons,
    }
}

pub(crate) fn build_continuity_journey_report(
    output: &Path,
    registry: &BenchmarkRegistry,
    benchmark: &FeatureBenchmarkReport,
) -> Option<ContinuityJourneyReport> {
    let journey = registry.journeys.iter().find(|journey| {
        journey.gate_target == "acceptable"
            || journey.feature_ids.iter().any(|feature_id| {
                registry
                    .features
                    .iter()
                    .find(|feature| feature.id == feature_id.as_str())
                    .is_some_and(|feature| feature.continuity_critical)
            })
    })?;

    let metrics = derived_continuity_metrics(benchmark);
    let evidence = evidence_summary_from_feature_benchmark(benchmark);
    let gate_decision = resolve_benchmark_scorecard(&metrics, &evidence, true);
    let gate_label = gate_decision.gate.clone();
    let artifact_dir = benchmark_telemetry_dir(output);

    Some(ContinuityJourneyReport {
        journey_id: journey.id.clone(),
        journey_name: journey.name.clone(),
        gate_decision,
        metrics,
        evidence,
        baseline_modes: journey.baseline_mode_ids.clone(),
        feature_ids: journey.feature_ids.clone(),
        artifact_paths: vec![
            artifact_dir.join("latest.json").display().to_string(),
            artifact_dir.join("latest.md").display().to_string(),
        ],
        summary: format!(
            "{} resolves to {} with {} evidence signals",
            journey.name,
            gate_label,
            benchmark.evidence.len()
        ),
        generated_at: Some(benchmark.completed_at),
    })
}

pub(crate) fn render_continuity_journey_markdown(report: &ContinuityJourneyReport) -> String {
    let mut markdown = String::new();
    markdown.push_str("# continuity journey evidence\n\n");
    markdown.push_str(&format!("- Journey: `{}`\n", report.journey_id));
    markdown.push_str(&format!("- Name: {}\n", report.journey_name));
    markdown.push_str(&format!(
        "- Gate: `{}` (score `{}`)\n",
        report.gate_decision.gate, report.gate_decision.resolved_score
    ));
    markdown.push_str(&format!(
        "- Baseline modes: `{}`\n",
        report.baseline_modes.join("`, `")
    ));
    markdown.push_str(&format!(
        "- Features: `{}`\n",
        report.feature_ids.join("`, `")
    ));
    markdown.push_str(&format!("- Metrics: {:?}\n", report.metrics));
    markdown.push_str(&format!("- Evidence: {:?}\n", report.evidence));
    markdown.push_str(&format!("- Summary: {}\n", report.summary));
    markdown
}

pub(crate) fn write_continuity_journey_artifacts(
    output: &Path,
    report: &ContinuityJourneyReport,
) -> anyhow::Result<()> {
    let dir = benchmark_telemetry_dir(output);
    fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    let json_path = dir.join("latest.json");
    let md_path = dir.join("latest.md");
    let json = serde_json::to_string_pretty(report)?;
    let markdown = render_continuity_journey_markdown(report);
    fs::write(&json_path, json).with_context(|| format!("write {}", json_path.display()))?;
    fs::write(&md_path, markdown).with_context(|| format!("write {}", md_path.display()))?;
    Ok(())
}
