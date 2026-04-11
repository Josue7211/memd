use super::*;

pub(crate) async fn execute_named_verifier_command(
    output: &Path,
    mode: &str,
    subject: Option<String>,
    baseline: Option<String>,
    verifier: &VerifierRecord,
) -> anyhow::Result<VerifyReport> {
    let (repo_root, registry) = load_benchmark_registry_for_output(output)?
        .context("verify command requires benchmark-registry.json")?;
    let fixture = registry
        .fixtures
        .iter()
        .find(|fixture| fixture.id == verifier.fixture_id)
        .with_context(|| format!("missing fixture {}", verifier.fixture_id))?;
    let runtime = read_bundle_runtime_config(output)?;
    let run = run_verifier_record(
        verifier,
        fixture,
        runtime
            .as_ref()
            .and_then(|config| config.base_url.as_deref()),
    )
    .await?;
    let evidence_payload = json!({
        "verifier_id": verifier.id,
        "fixture_id": fixture.id,
        "confidence_tier": "live_primary",
        "mode": mode,
        "subject": subject,
    });
    write_verifier_run_artifacts(output, &run, &evidence_payload)?;
    Ok(build_verify_report_from_run(
        mode, output, &repo_root, &registry, subject, baseline, &run,
    ))
}

pub(crate) fn build_verify_report(
    mode: &str,
    output: &Path,
    lane: Option<String>,
    subject: Option<String>,
    baseline: Option<String>,
) -> anyhow::Result<VerifyReport> {
    let loaded_registry = load_benchmark_registry_for_output(output)?;
    let (
        repo_root,
        registry_loaded,
        registry_version,
        registry_features,
        registry_journeys,
        registry_loops,
        registry_verifiers,
        registry_fixtures,
    ) = if let Some((repo_root, registry)) = loaded_registry {
        (
            Some(repo_root.display().to_string()),
            true,
            Some(registry.version),
            registry.features.len(),
            registry.journeys.len(),
            registry.loops.len(),
            registry.verifiers.len(),
            registry.fixtures.len(),
        )
    } else {
        (None, false, None, 0, 0, 0, 0, 0)
    };

    let mut findings = vec![format!("verify {} placeholder check", mode)];
    if registry_loaded {
        findings.push("benchmark registry loaded".to_string());
    } else {
        findings.push("benchmark registry unavailable".to_string());
    }
    if let Some(lane) = lane.as_ref() {
        findings.push(format!("lane={lane}"));
    }
    if let Some(subject) = subject.as_ref() {
        findings.push(format!("subject={subject}"));
    }

    let mut recommendations = vec!["replace stub with concrete verification checks".to_string()];
    if !registry_loaded {
        recommendations
            .push("add docs/verification/benchmark-registry.json to the repo root".to_string());
    }

    Ok(VerifyReport {
        mode: mode.to_string(),
        bundle_root: output.display().to_string(),
        repo_root,
        registry_loaded,
        registry_version,
        registry_features,
        registry_journeys,
        registry_loops,
        registry_verifiers,
        registry_fixtures,
        lane,
        subject,
        baseline,
        findings,
        recommendations,
        generated_at: Utc::now(),
    })
}

pub(crate) fn render_verify_summary(report: &VerifyReport) -> String {
    let mut summary = format!(
        "verify mode={} bundle={} registry_loaded={} version={} features={} journeys={} loops={} verifiers={} fixtures={}",
        report.mode,
        report.bundle_root,
        report.registry_loaded,
        report.registry_version.as_deref().unwrap_or("none"),
        report.registry_features,
        report.registry_journeys,
        report.registry_loops,
        report.registry_verifiers,
        report.registry_fixtures
    );
    if let Some(lane) = report.lane.as_deref() {
        summary.push_str(&format!(" lane={lane}"));
    }
    if let Some(subject) = report.subject.as_deref() {
        summary.push_str(&format!(" subject={subject}"));
    }
    if let Some(baseline) = report.baseline.as_deref() {
        summary.push_str(&format!(" baseline={baseline}"));
    }
    if !report.findings.is_empty() {
        summary.push_str(&format!(" findings={}", report.findings.join("|")));
    }
    summary
}

pub(crate) async fn run_verify_feature_command(
    args: &VerifyFeatureArgs,
) -> anyhow::Result<VerifyReport> {
    let (_, registry) = load_benchmark_registry_for_output(&args.output)?
        .context("verify feature requires benchmark-registry.json")?;
    let verifier = find_verifier_by_subject(&registry, "feature_contract", &args.feature_id)
        .with_context(|| format!("missing feature verifier for {}", args.feature_id))?
        .clone();
    execute_named_verifier_command(
        &args.output,
        "feature",
        Some(args.feature_id.clone()),
        None,
        &verifier,
    )
    .await
}

pub(crate) async fn run_verify_journey_command(
    args: &VerifyJourneyArgs,
) -> anyhow::Result<VerifyReport> {
    let (_, registry) = load_benchmark_registry_for_output(&args.output)?
        .context("verify journey requires benchmark-registry.json")?;
    let verifier = find_verifier_by_subject(&registry, "journey", &args.journey_id)
        .with_context(|| format!("missing journey verifier for {}", args.journey_id))?
        .clone();
    execute_named_verifier_command(
        &args.output,
        "journey",
        Some(args.journey_id.clone()),
        None,
        &verifier,
    )
    .await
}

pub(crate) async fn run_verify_adversarial_command(
    args: &VerifyAdversarialArgs,
) -> anyhow::Result<VerifyReport> {
    let (_, registry) = load_benchmark_registry_for_output(&args.output)?
        .context("verify adversarial requires benchmark-registry.json")?;
    let verifier = find_verifier_by_id(&registry, &args.verifier_id)
        .with_context(|| format!("missing adversarial verifier {}", args.verifier_id))?
        .clone();
    execute_named_verifier_command(
        &args.output,
        "adversarial",
        Some(args.verifier_id.clone()),
        None,
        &verifier,
    )
    .await
}

pub(crate) async fn run_verify_compare_command(
    args: &VerifyCompareArgs,
) -> anyhow::Result<VerifyReport> {
    let (_, registry) = load_benchmark_registry_for_output(&args.output)?
        .context("verify compare requires benchmark-registry.json")?;
    let verifier = find_verifier_by_id(&registry, &args.verifier_id)
        .with_context(|| format!("missing comparative verifier {}", args.verifier_id))?
        .clone();
    execute_named_verifier_command(
        &args.output,
        "compare",
        Some(args.verifier_id.clone()),
        Some(verifier.baseline_modes.join(",")),
        &verifier,
    )
    .await
}

pub(crate) async fn execute_verify_sweep(
    output: &Path,
    repo_root: Option<&Path>,
    registry: &BenchmarkRegistry,
    lane: &str,
) -> anyhow::Result<VerifySweepReport> {
    let runtime = read_bundle_runtime_config(output)?;
    let base_url_override = runtime
        .as_ref()
        .and_then(|config| config.base_url.as_deref());
    let selected = registry
        .verifiers
        .iter()
        .filter(|verifier| verifier.lanes.iter().any(|candidate| candidate == lane))
        .cloned()
        .collect::<Vec<_>>();

    let mut runs = Vec::new();
    let mut failures = Vec::new();
    for verifier in &selected {
        let fixture = registry
            .fixtures
            .iter()
            .find(|fixture| fixture.id == verifier.fixture_id)
            .with_context(|| format!("missing fixture {}", verifier.fixture_id))?;
        let run = run_verifier_record(verifier, fixture, base_url_override).await?;
        if run.status != "passing" {
            let failure = if verifier_is_tier_zero(verifier, registry) {
                format!("tier-0 failure {}", verifier.id)
            } else if verifier_is_critical_comparative_failure(verifier, &run) {
                format!("critical comparative regression {}", verifier.id)
            } else {
                format!("noncritical failure {}", verifier.id)
            };
            failures.push(failure);
        }
        runs.push(run);
    }

    let has_blocking_failure = failures.iter().any(|failure| {
        failure.starts_with("tier-0 failure")
            || failure.starts_with("critical comparative regression")
    });
    let ok = if lane == "nightly" {
        !has_blocking_failure
    } else {
        true
    };

    Ok(VerifySweepReport {
        lane: lane.to_string(),
        ok,
        total: runs.len(),
        passed: runs.iter().filter(|run| run.status == "passing").count(),
        failures,
        runs,
        bundle_root: output.display().to_string(),
        repo_root: repo_root.map(|root| root.display().to_string()),
    })
}

pub(crate) fn render_verify_sweep_summary(report: &VerifySweepReport) -> String {
    let mut summary = format!(
        "verify sweep lane={} ok={} total={} passed={}",
        report.lane, report.ok, report.total, report.passed
    );
    if !report.failures.is_empty() {
        summary.push_str(&format!(" failures={}", report.failures.join("|")));
    }
    summary
}

pub(crate) fn render_verifiers_markdown(
    registry: &BenchmarkRegistry,
    report: &VerifySweepReport,
) -> String {
    let mut markdown = String::from("# memd verifiers\n\n");
    markdown.push_str(&format!("- Lane: `{}`\n", report.lane));
    markdown.push_str(&format!("- Total: `{}`\n\n", registry.verifiers.len()));
    for verifier in &registry.verifiers {
        markdown.push_str(&format!(
            "- `{}` [{}] fixture=`{}` lanes=`{}`\n",
            verifier.id,
            verifier.verifier_type,
            verifier.fixture_id,
            verifier.lanes.join(",")
        ));
    }
    markdown
}

pub(crate) fn render_fixtures_markdown(registry: &BenchmarkRegistry) -> String {
    let mut markdown = String::from("# memd fixtures\n\n");
    markdown.push_str(&format!("- Total: `{}`\n\n", registry.fixtures.len()));
    for fixture in &registry.fixtures {
        markdown.push_str(&format!(
            "- `{}` kind=`{}` isolation=`{}` backend=`{}`\n",
            fixture.id, fixture.kind, fixture.isolation, fixture.backend_mode
        ));
    }
    markdown
}

pub(crate) fn render_verify_coverage_markdown(
    registry: &BenchmarkRegistry,
    report: &VerifySweepReport,
) -> String {
    let feature_contracts = registry
        .verifiers
        .iter()
        .filter(|verifier| verifier.verifier_type == "feature_contract")
        .count();
    let journeys = registry
        .verifiers
        .iter()
        .filter(|verifier| verifier.verifier_type == "journey")
        .count();
    let adversarials = registry
        .verifiers
        .iter()
        .filter(|verifier| verifier.verifier_type == "adversarial")
        .count();
    let comparatives = registry
        .verifiers
        .iter()
        .filter(|verifier| verifier.verifier_type == "comparative")
        .count();
    format!(
        "# memd verification coverage\n\n- Verifiers: `{}`\n- Fixtures: `{}`\n- Sweep lane: `{}`\n- Selected: `{}`\n- Passed: `{}`\n- Failures: `{}`\n- Feature contracts: `{}`\n- Journeys: `{}`\n- Adversarials: `{}`\n- Comparatives: `{}`\n",
        registry.verifiers.len(),
        registry.fixtures.len(),
        report.lane,
        report.total,
        report.passed,
        report.failures.len(),
        feature_contracts,
        journeys,
        adversarials,
        comparatives
    )
}

pub(crate) fn render_verify_scores_markdown(report: &VerifySweepReport) -> String {
    let strong = report
        .runs
        .iter()
        .filter(|run| run.gate_result == "strong")
        .count();
    let acceptable = report
        .runs
        .iter()
        .filter(|run| run.gate_result == "acceptable")
        .count();
    let fragile_or_broken = report
        .runs
        .iter()
        .filter(|run| matches!(run.gate_result.as_str(), "fragile" | "broken"))
        .count();
    let mut markdown = format!(
        "# memd verification scores\n\n- Lane: `{}`\n- OK: `{}`\n- Passed: `{}/{}`\n- Strong: `{}`\n- Acceptable: `{}`\n- Fragile/Broken: `{}`\n",
        report.lane, report.ok, report.passed, report.total, strong, acceptable, fragile_or_broken
    );
    if !report.runs.is_empty() {
        markdown.push_str("\n## Verifier Runs\n");
        for run in &report.runs {
            markdown.push_str(&format!(
                "- `{}` status=`{}` gate=`{}` evidence=`{}`\n",
                run.verifier_id,
                run.status,
                run.gate_result,
                run.evidence_ids.join(",")
            ));
        }
    }
    if !report.failures.is_empty() {
        markdown.push_str("\n## Failures\n");
        for failure in &report.failures {
            markdown.push_str(&format!("- {}\n", failure));
        }
    }
    markdown
}

pub(crate) fn write_verify_operator_docs(
    repo_root: &Path,
    registry: &BenchmarkRegistry,
    report: &VerifySweepReport,
) -> anyhow::Result<()> {
    fs::create_dir_all(benchmark_registry_docs_dir(repo_root)).with_context(|| {
        format!(
            "create {}",
            benchmark_registry_docs_dir(repo_root).display()
        )
    })?;
    fs::write(
        benchmark_registry_markdown_path(repo_root, "VERIFIERS.md"),
        render_verifiers_markdown(registry, report),
    )
    .with_context(|| "write VERIFIERS.md".to_string())?;
    fs::write(
        benchmark_registry_markdown_path(repo_root, "FIXTURES.md"),
        render_fixtures_markdown(registry),
    )
    .with_context(|| "write FIXTURES.md".to_string())?;
    fs::write(
        benchmark_registry_markdown_path(repo_root, "COVERAGE.md"),
        render_verify_coverage_markdown(registry, report),
    )
    .with_context(|| "write COVERAGE.md".to_string())?;
    fs::write(
        benchmark_registry_markdown_path(repo_root, "SCORES.md"),
        render_verify_scores_markdown(report),
    )
    .with_context(|| "write SCORES.md".to_string())?;
    Ok(())
}

pub(crate) fn write_verify_sweep_artifacts(
    output: &Path,
    report: &VerifySweepReport,
) -> anyhow::Result<()> {
    fs::create_dir_all(verification_reports_dir(output))
        .with_context(|| format!("create {}", verification_reports_dir(output).display()))?;
    let latest_path = verification_reports_dir(output).join("latest.json");
    fs::write(
        &latest_path,
        serde_json::to_string_pretty(report).context("serialize verify sweep latest")? + "\n",
    )
    .with_context(|| format!("write {}", latest_path.display()))?;
    let latest_md = verification_reports_dir(output).join("latest.md");
    fs::write(&latest_md, render_verify_sweep_summary(report))
        .with_context(|| format!("write {}", latest_md.display()))?;
    Ok(())
}

pub(crate) fn read_latest_verify_sweep_report(output: &Path) -> Option<VerifySweepReport> {
    let path = verification_reports_dir(output).join("latest.json");
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

pub(crate) async fn run_verify_sweep_command(
    args: &VerifySweepArgs,
) -> anyhow::Result<VerifyReport> {
    let (repo_root, registry) = load_benchmark_registry_for_output(&args.output)?
        .context("verify sweep requires benchmark-registry.json")?;
    let report =
        execute_verify_sweep(&args.output, Some(&repo_root), &registry, &args.lane).await?;
    write_verify_sweep_artifacts(&args.output, &report)?;
    write_verify_operator_docs(&repo_root, &registry, &report)?;
    Ok(VerifyReport {
        mode: "sweep".to_string(),
        bundle_root: report.bundle_root.clone(),
        repo_root: report.repo_root.clone(),
        registry_loaded: true,
        registry_version: Some(registry.version),
        registry_features: registry.features.len(),
        registry_journeys: registry.journeys.len(),
        registry_loops: registry.loops.len(),
        registry_verifiers: registry.verifiers.len(),
        registry_fixtures: registry.fixtures.len(),
        lane: Some(report.lane.clone()),
        subject: None,
        baseline: None,
        findings: if report.failures.is_empty() {
            vec!["verify sweep completed".to_string()]
        } else {
            report.failures.clone()
        },
        recommendations: vec!["promote real verifier execution over placeholders".to_string()],
        generated_at: Utc::now(),
    })
}

pub(crate) fn run_verify_doctor_command(args: &VerifyDoctorArgs) -> anyhow::Result<VerifyReport> {
    build_verify_report("doctor", &args.output, None, None, None)
}

pub(crate) fn run_verify_list_command(args: &VerifyListArgs) -> anyhow::Result<VerifyReport> {
    build_verify_report("list", &args.output, args.lane.clone(), None, None)
}

pub(crate) fn run_verify_show_command(args: &VerifyShowArgs) -> anyhow::Result<VerifyReport> {
    build_verify_report("show", &args.output, None, Some(args.item_id.clone()), None)
}
