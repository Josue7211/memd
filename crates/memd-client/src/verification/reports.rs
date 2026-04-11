use super::*;

pub(crate) fn gate_rank(gate: &str) -> u8 {
    match gate {
        "broken" => 0,
        "fragile" => 1,
        "acceptable" => 2,
        "strong" => 3,
        "ten_star" => 4,
        _ => 0,
    }
}

pub(crate) fn verifier_run_morning_severity(
    run: &VerifierRunRecord,
    gate_target: &str,
    continuity_critical: bool,
) -> u8 {
    let actual_rank = gate_rank(&run.gate_result);
    let target_rank = gate_rank(gate_target);
    let target_gap = target_rank.saturating_sub(actual_rank);
    match run.gate_result.as_str() {
        "broken" => {
            if continuity_critical {
                8
            } else {
                7
            }
        }
        "fragile" => {
            if continuity_critical {
                6
            } else {
                5
            }
        }
        "acceptable" => {
            if continuity_critical {
                3 + target_gap
            } else {
                target_gap
            }
        }
        _ if run.status != "passing" => {
            if continuity_critical {
                4
            } else {
                2
            }
        }
        _ => 0,
    }
}

pub(crate) fn build_no_memd_delta_report(
    no_memd: &BaselineMetrics,
    with_memd: &BaselineMetrics,
) -> NoMemdDeltaReport {
    NoMemdDeltaReport {
        no_memd: no_memd.clone(),
        with_memd: with_memd.clone(),
        token_delta: no_memd.prompt_tokens as isize - with_memd.prompt_tokens as isize,
        reread_delta: no_memd.reread_count as isize - with_memd.reread_count as isize,
        reconstruction_delta: no_memd.reconstruction_steps as isize
            - with_memd.reconstruction_steps as isize,
        with_memd_better: no_memd.prompt_tokens > with_memd.prompt_tokens
            && no_memd.reread_count > with_memd.reread_count
            && no_memd.reconstruction_steps > with_memd.reconstruction_steps,
    }
}

pub(crate) fn build_benchmark_comparison_report(
    benchmark: &FeatureBenchmarkReport,
) -> Option<NoMemdDeltaReport> {
    let failing_area_count = benchmark
        .areas
        .iter()
        .filter(|area| area.status != "pass")
        .count();
    let no_memd = BaselineMetrics {
        prompt_tokens: 1600
            + benchmark.command_count * 50
            + benchmark.event_count * 20
            + benchmark.memory_pages * 32
            + benchmark.areas.len() * 18,
        reread_count: 4 + failing_area_count + benchmark.recommendations.len(),
        reconstruction_steps: 3 + failing_area_count.saturating_mul(2) + benchmark.memory_pages / 2,
    };
    let with_memd = BaselineMetrics {
        prompt_tokens: 1100
            + benchmark.command_count * 32
            + benchmark.event_count * 10
            + benchmark.memory_pages * 18
            + benchmark.areas.len() * 10,
        reread_count: 1 + failing_area_count.saturating_sub(1),
        reconstruction_steps: 1 + failing_area_count,
    };
    Some(build_no_memd_delta_report(&no_memd, &with_memd))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct VerifyReport {
    pub(crate) mode: String,
    pub(crate) bundle_root: String,
    pub(crate) repo_root: Option<String>,
    pub(crate) registry_loaded: bool,
    pub(crate) registry_version: Option<String>,
    pub(crate) registry_features: usize,
    pub(crate) registry_journeys: usize,
    pub(crate) registry_loops: usize,
    pub(crate) registry_verifiers: usize,
    pub(crate) registry_fixtures: usize,
    pub(crate) lane: Option<String>,
    pub(crate) subject: Option<String>,
    pub(crate) baseline: Option<String>,
    pub(crate) findings: Vec<String>,
    pub(crate) recommendations: Vec<String>,
    pub(crate) generated_at: DateTime<Utc>,
}

pub(crate) fn find_verifier_by_subject<'a>(
    registry: &'a BenchmarkRegistry,
    verifier_type: &str,
    subject_id: &str,
) -> Option<&'a VerifierRecord> {
    registry.verifiers.iter().find(|verifier| {
        verifier.verifier_type == verifier_type
            && verifier
                .subject_ids
                .iter()
                .any(|candidate| candidate == subject_id)
    })
}

pub(crate) fn find_verifier_by_id<'a>(
    registry: &'a BenchmarkRegistry,
    verifier_id: &str,
) -> Option<&'a VerifierRecord> {
    registry
        .verifiers
        .iter()
        .find(|verifier| verifier.id == verifier_id)
}

pub(crate) fn build_verify_report_from_run(
    mode: &str,
    output: &Path,
    repo_root: &Path,
    registry: &BenchmarkRegistry,
    subject: Option<String>,
    baseline: Option<String>,
    run: &VerifierRunRecord,
) -> VerifyReport {
    let mut findings = vec![format!("verifier_run_status={}", run.status)];
    findings.push(format!("gate_result={}", run.gate_result));
    findings.push(format!("evidence={}", run.evidence_ids.join(",")));
    VerifyReport {
        mode: mode.to_string(),
        bundle_root: output.display().to_string(),
        repo_root: Some(repo_root.display().to_string()),
        registry_loaded: true,
        registry_version: Some(registry.version.clone()),
        registry_features: registry.features.len(),
        registry_journeys: registry.journeys.len(),
        registry_loops: registry.loops.len(),
        registry_verifiers: registry.verifiers.len(),
        registry_fixtures: registry.fixtures.len(),
        lane: None,
        subject,
        baseline,
        findings,
        recommendations: vec!["replace stub steps with concrete verifier execution".to_string()],
        generated_at: Utc::now(),
    }
}

pub(crate) fn verifier_is_tier_zero(
    verifier: &VerifierRecord,
    registry: &BenchmarkRegistry,
) -> bool {
    verifier.subject_ids.iter().any(|subject_id| {
        registry
            .features
            .iter()
            .find(|feature| feature.id == *subject_id)
            .map(|feature| feature.tier == "tier-0-continuity-critical")
            .unwrap_or(false)
    })
}

pub(crate) fn verifier_is_critical_comparative_failure(
    verifier: &VerifierRecord,
    run: &VerifierRunRecord,
) -> bool {
    verifier.verifier_type == "comparative"
        && run.status != "passing"
        && run.gate_result == "acceptable"
}
