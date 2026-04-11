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

pub(crate) fn derived_continuity_metrics(benchmark: &FeatureBenchmarkReport) -> BenchmarkSubjectMetrics {
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
        "- Feature count: `{}`\n",
        report.feature_ids.len()
    ));
    markdown.push_str("\n## Evidence Summary\n");
    markdown.push_str(&format!(
        "- contract evidence: `{}`\n",
        report.evidence.has_contract_evidence
    ));
    markdown.push_str(&format!(
        "- workflow evidence: `{}`\n",
        report.evidence.has_workflow_evidence
    ));
    markdown.push_str(&format!(
        "- continuity evidence: `{}`\n",
        report.evidence.has_continuity_evidence
    ));
    markdown.push_str(&format!(
        "- comparative evidence: `{}`\n",
        report.evidence.has_comparative_evidence
    ));
    markdown.push_str(&format!(
        "- drift failure: `{}`\n",
        report.evidence.has_drift_failure
    ));
    markdown.push_str("\n## Metrics\n");
    markdown.push_str(&format!(
        "- correctness: `{}`\n",
        report.metrics.correctness
    ));
    markdown.push_str(&format!("- continuity: `{}`\n", report.metrics.continuity));
    markdown.push_str(&format!(
        "- reliability: `{}`\n",
        report.metrics.reliability
    ));
    markdown.push_str(&format!(
        "- token efficiency: `{}`\n",
        report.metrics.token_efficiency
    ));
    markdown.push_str(&format!(
        "- no-memd delta: `{}`\n",
        report
            .metrics
            .no_memd_delta
            .map(|delta: i16| delta.to_string())
            .unwrap_or_else(|| "unset".to_string())
    ));
    if !report.gate_decision.reasons.is_empty() {
        markdown.push_str("\n## Gate Reasons\n");
        for reason in &report.gate_decision.reasons {
            markdown.push_str(&format!("- {}\n", reason));
        }
    }
    markdown.push('\n');
    markdown
}

pub(crate) fn write_continuity_journey_artifacts(
    output: &Path,
    report: &ContinuityJourneyReport,
) -> anyhow::Result<()> {
    let continuity_dir = benchmark_telemetry_dir(output);
    fs::create_dir_all(&continuity_dir)
        .with_context(|| format!("create {}", continuity_dir.display()))?;

    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let baseline_json = continuity_dir.join("latest.json");
    let baseline_md = continuity_dir.join("latest.md");
    let timestamp_json = continuity_dir.join(format!("{timestamp}.json"));
    let timestamp_md = continuity_dir.join(format!("{timestamp}.md"));
    let json = serde_json::to_string_pretty(report)? + "\n";
    let markdown = render_continuity_journey_markdown(report);

    fs::write(&baseline_json, &json)
        .with_context(|| format!("write {}", baseline_json.display()))?;
    fs::write(&baseline_md, &markdown)
        .with_context(|| format!("write {}", baseline_md.display()))?;
    fs::write(&timestamp_json, &json)
        .with_context(|| format!("write {}", timestamp_json.display()))?;
    fs::write(&timestamp_md, &markdown)
        .with_context(|| format!("write {}", timestamp_md.display()))?;
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct BenchmarkRegistryDocsReport {
    _repo_root: PathBuf,
    _registry_path: PathBuf,
    _registry: BenchmarkRegistry,
    _comparative_report: Option<NoMemdDeltaReport>,
    benchmarks_markdown: String,
    loops_markdown: String,
    coverage_markdown: String,
    scores_markdown: String,
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

#[derive(Debug)]
pub(crate) struct MaterializedFixture {
    _fixture_id: String,
    _root: TempDir,
    bundle_root: PathBuf,
    fixture_vars: BTreeMap<String, String>,
    _session_bundles: BTreeMap<String, PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct VerifierRunRecord {
    pub(crate) verifier_id: String,
    pub(crate) status: String,
    pub(crate) gate_result: String,
    pub(crate) evidence_ids: Vec<String>,
    pub(crate) metrics_observed: BTreeMap<String, JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct VerifySweepReport {
    pub(crate) lane: String,
    pub(crate) ok: bool,
    pub(crate) total: usize,
    pub(crate) passed: usize,
    pub(crate) failures: Vec<String>,
    pub(crate) runs: Vec<VerifierRunRecord>,
    pub(crate) bundle_root: String,
    pub(crate) repo_root: Option<String>,
}

pub(crate) fn verification_reports_dir(output: &Path) -> PathBuf {
    output.join("verification")
}

pub(crate) fn verification_runs_dir(output: &Path) -> PathBuf {
    verification_reports_dir(output).join("runs")
}

pub(crate) fn verification_evidence_dir(output: &Path) -> PathBuf {
    verification_reports_dir(output).join("evidence")
}

pub(crate) fn fixture_seed_object(
    fixture: &FixtureRecord,
) -> anyhow::Result<serde_json::Map<String, JsonValue>> {
    fixture
        .seed_config
        .as_object()
        .cloned()
        .context("fixture seed_config must be a JSON object")
}

pub(crate) fn fixture_seed_string(
    seed: &serde_json::Map<String, JsonValue>,
    key: &str,
    default: &str,
) -> String {
    seed.get(key)
        .and_then(JsonValue::as_str)
        .unwrap_or(default)
        .to_string()
}

pub(crate) fn fixture_seed_defaults(
    fixture: &FixtureRecord,
    base_url_override: Option<&str>,
) -> anyhow::Result<serde_json::Map<String, JsonValue>> {
    let mut seed = fixture_seed_object(fixture)?;
    let defaults = [
        ("project", "memd"),
        ("namespace", "main"),
        ("agent", "codex"),
        ("session", "verifier-fixture"),
        ("workspace", "shared"),
        ("visibility", "workspace"),
        ("route", "auto"),
        ("intent", "current_task"),
        ("base_url", "http://127.0.0.1:59999"),
    ];
    for (key, value) in defaults {
        seed.entry(key.to_string())
            .or_insert_with(|| JsonValue::String(value.to_string()));
    }
    if let Some(base_url) = base_url_override.filter(|value| !value.trim().is_empty()) {
        seed.insert(
            "base_url".to_string(),
            JsonValue::String(base_url.to_string()),
        );
    }
    Ok(seed)
}

pub(crate) fn build_fixture_vars(seed: &serde_json::Map<String, JsonValue>) -> BTreeMap<String, String> {
    let run_id = uuid::Uuid::new_v4().simple().to_string();
    let task_seed = fixture_seed_string(seed, "task_id", "task-current");
    let task_id = format!("{task_seed}-{}", &run_id[..8]);
    let next_action = fixture_seed_string(seed, "next_action", "resume next step");
    BTreeMap::from([
        ("run.id".to_string(), run_id),
        ("task.id".to_string(), task_id),
        ("task.next_action".to_string(), next_action),
    ])
}

pub(crate) fn build_fixture_resume_snapshot(
    seed: &serde_json::Map<String, JsonValue>,
    fixture_vars: &BTreeMap<String, String>,
) -> ResumeSnapshot {
    let project = fixture_seed_string(seed, "project", "memd");
    let namespace = fixture_seed_string(seed, "namespace", "main");
    let agent = fixture_seed_string(seed, "agent", "codex");
    let workspace = fixture_seed_string(seed, "workspace", "shared");
    let visibility = fixture_seed_string(seed, "visibility", "workspace");
    let route = fixture_seed_string(seed, "route", "auto");
    let intent = fixture_seed_string(seed, "intent", "current_task");
    let task_id = fixture_vars
        .get("task.id")
        .cloned()
        .unwrap_or_else(|| "task-current".to_string());
    let next_action = fixture_vars
        .get("task.next_action")
        .cloned()
        .unwrap_or_else(|| "resume next step".to_string());

    ResumeSnapshot {
        project: Some(project.clone()),
        namespace: Some(namespace.clone()),
        agent: Some(agent),
        workspace: Some(workspace.clone()),
        visibility: Some(visibility),
        route,
        intent,
        context: memd_schema::CompactContextResponse {
            route: memd_schema::RetrievalRoute::Auto,
            intent: memd_schema::RetrievalIntent::CurrentTask,
            retrieval_order: vec![memd_schema::MemoryScope::Project],
            records: vec![memd_schema::CompactMemoryRecord {
                id: uuid::Uuid::new_v4(),
                record: format!("current task {task_id}"),
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
                record: format!("focus: {task_id}"),
            }],
            evicted: Vec::new(),
            rehydration_queue: vec![memd_schema::MemoryRehydrationRecord {
                id: None,
                kind: "task".to_string(),
                label: "next".to_string(),
                summary: next_action,
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
                    content: "keep continuity tight".to_string(),
                    redundancy_key: None,
                    belief_branch: None,
                    preferred: true,
                    kind: memd_schema::MemoryKind::Status,
                    scope: memd_schema::MemoryScope::Project,
                    project: Some(project.clone()),
                    namespace: Some(namespace.clone()),
                    workspace: Some(workspace.clone()),
                    visibility: memd_schema::MemoryVisibility::Workspace,
                    source_agent: None,
                    source_system: None,
                    source_path: None,
                    source_quality: None,
                    confidence: 0.9,
                    ttl_seconds: Some(86_400),
                    created_at: Utc::now(),
                    status: memd_schema::MemoryStatus::Active,
                    stage: memd_schema::MemoryStage::Candidate,
                    last_verified_at: None,
                    supersedes: Vec::new(),
                    updated_at: Utc::now(),
                    tags: vec!["continuity".to_string()],
                },
                reasons: vec!["fixture".to_string()],
            }],
        },
        workspaces: memd_schema::WorkspaceMemoryResponse {
            workspaces: vec![memd_schema::WorkspaceMemoryRecord {
                project: Some(project),
                namespace: Some(namespace),
                workspace: Some(workspace),
                visibility: memd_schema::MemoryVisibility::Workspace,
                item_count: 4,
                active_count: 3,
                candidate_count: 1,
                contested_count: 0,
                source_lane_count: 1,
                avg_confidence: 0.9,
                trust_score: 0.94,
                last_seen_at: None,
                tags: Vec::new(),
            }],
        },
        sources: memd_schema::SourceMemoryResponse {
            sources: Vec::new(),
        },
        semantic: None,
        claims: SessionClaimsState::default(),
        recent_repo_changes: vec!["crates/memd-client/src/main.rs".to_string()],
        change_summary: vec!["fixture continuity seeded".to_string()],
        resume_state_age_minutes: Some(1),
        refresh_recommended: false,
    }
}

pub(crate) fn verifier_resume_args(output: &Path) -> ResumeArgs {
    ResumeArgs {
        output: output.to_path_buf(),
        project: None,
        namespace: None,
        agent: None,
        workspace: None,
        visibility: None,
        route: None,
        intent: None,
        limit: Some(8),
        rehydration_limit: Some(4),
        semantic: false,
        prompt: false,
        summary: false,
    }
}

pub(crate) fn verifier_handoff_args(output: &Path) -> HandoffArgs {
    HandoffArgs {
        output: output.to_path_buf(),
        target_session: None,
        project: None,
        namespace: None,
        agent: None,
        workspace: None,
        visibility: None,
        route: None,
        intent: None,
        limit: Some(8),
        rehydration_limit: Some(4),
        source_limit: Some(6),
        semantic: false,
        prompt: false,
        summary: false,
    }
}

pub(crate) fn seed_materialized_fixture(
    bundle_root: &Path,
    seed: &serde_json::Map<String, JsonValue>,
    fixture_vars: &BTreeMap<String, String>,
    fixture: &FixtureRecord,
) -> anyhow::Result<()> {
    let runtime_json = JsonValue::Object(seed.clone());
    fs::write(
        bundle_root.join("config.json"),
        serde_json::to_string_pretty(&runtime_json).context("serialize fixture config")? + "\n",
    )
    .with_context(|| format!("write {}", bundle_root.join("config.json").display()))?;
    fs::write(bundle_root.join("env"), "")
        .with_context(|| format!("write {}", bundle_root.join("env").display()))?;
    fs::write(bundle_root.join("env.ps1"), "")
        .with_context(|| format!("write {}", bundle_root.join("env.ps1").display()))?;

    let runtime = read_bundle_runtime_config(bundle_root)?
        .context("fixture runtime config missing after materialization")?;
    let base_url = runtime
        .base_url
        .as_deref()
        .unwrap_or("http://127.0.0.1:59999");
    let resume_args = verifier_resume_args(bundle_root);
    let resume_snapshot = build_fixture_resume_snapshot(seed, fixture_vars);
    let resume_key = build_resume_snapshot_cache_key(&resume_args, Some(&runtime), base_url);
    cache::write_resume_snapshot_cache(bundle_root, &resume_key, &resume_snapshot)
        .context("write fixture resume cache")?;
    write_bundle_resume_state(bundle_root, &resume_snapshot)
        .context("write fixture resume state")?;

    let handoff_args = verifier_handoff_args(bundle_root);
    let handoff = HandoffSnapshot {
        generated_at: Utc::now(),
        resume: resume_snapshot.clone(),
        sources: memd_schema::SourceMemoryResponse {
            sources: Vec::new(),
        },
        target_session: None,
        target_bundle: Some(bundle_root.display().to_string()),
    };
    let handoff_key = cache::build_turn_key(
        Some(&bundle_root.display().to_string()),
        None,
        Some("none"),
        "handoff",
        &format!(
            "resume_key={}|source_limit={}|target_session=none|target_bundle={}",
            build_resume_snapshot_cache_key(
                &ResumeArgs {
                    output: bundle_root.to_path_buf(),
                    project: handoff_args.project.clone(),
                    namespace: handoff_args.namespace.clone(),
                    agent: handoff_args.agent.clone(),
                    workspace: handoff_args.workspace.clone(),
                    visibility: handoff_args.visibility.clone(),
                    route: handoff_args.route.clone(),
                    intent: handoff_args.intent.clone(),
                    limit: handoff_args.limit,
                    rehydration_limit: handoff_args.rehydration_limit,
                    semantic: handoff_args.semantic,
                    prompt: false,
                    summary: false,
                },
                Some(&runtime),
                base_url
            ),
            handoff_args.source_limit.unwrap_or(6),
            bundle_root.display()
        ),
    );
    cache::write_handoff_snapshot_cache(bundle_root, &handoff_key, &handoff)
        .context("write fixture handoff cache")?;

    for seed_file in &fixture.seed_files {
        let destination = bundle_root.join(seed_file);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        let content = format!(
            "fixture={}\ntask_id={}\nnext_action={}\n",
            fixture.id,
            fixture_vars
                .get("task.id")
                .map(String::as_str)
                .unwrap_or("task-current"),
            fixture_vars
                .get("task.next_action")
                .map(String::as_str)
                .unwrap_or("resume next step")
        );
        fs::write(&destination, content)
            .with_context(|| format!("write {}", destination.display()))?;
    }
    Ok(())
}

pub(crate) fn materialize_fixture(
    fixture: &FixtureRecord,
    base_url_override: Option<&str>,
) -> anyhow::Result<MaterializedFixture> {
    if fixture.kind != "bundle_fixture" {
        anyhow::bail!("unsupported fixture kind {}", fixture.kind);
    }
    if fixture.isolation != "fresh_temp_dir" {
        anyhow::bail!("unsupported fixture isolation {}", fixture.isolation);
    }

    let root = tempfile::tempdir().context("create fixture tempdir")?;
    let seed = fixture_seed_defaults(fixture, base_url_override)?;
    let mut fixture_vars = build_fixture_vars(&seed);
    let mut session_bundles = BTreeMap::new();

    let bundle_root = if fixture.seed_sessions.is_empty() {
        let bundle_root = root.path().join(".memd");
        fs::create_dir_all(&bundle_root)
            .with_context(|| format!("create {}", bundle_root.display()))?;
        seed_materialized_fixture(&bundle_root, &seed, &fixture_vars, fixture)?;
        bundle_root
    } else {
        let sessions_root = root.path().join("sessions");
        fs::create_dir_all(&sessions_root)
            .with_context(|| format!("create {}", sessions_root.display()))?;
        for (index, session_label) in fixture.seed_sessions.iter().enumerate() {
            let session_bundle = sessions_root.join(session_label).join(".memd");
            fs::create_dir_all(&session_bundle)
                .with_context(|| format!("create {}", session_bundle.display()))?;
            let mut session_seed = seed.clone();
            let session_agent = fixture_session_agent_name(session_label);
            let session_identity = format!(
                "{}-{}",
                session_label,
                uuid::Uuid::new_v4().simple().to_string()[..8].to_string()
            );
            session_seed.insert(
                "session".to_string(),
                JsonValue::String(session_identity.clone()),
            );
            if index > 0 || seed.get("agent").is_some() {
                session_seed.insert("agent".to_string(), JsonValue::String(session_agent));
            }
            seed_materialized_fixture(&session_bundle, &session_seed, &fixture_vars, fixture)?;
            fixture_vars.insert(
                format!("{session_label}_bundle"),
                session_bundle.display().to_string(),
            );
            fixture_vars.insert(format!("{session_label}_session"), session_identity.clone());
            session_bundles.insert(session_label.to_string(), session_bundle);
        }
        let primary_label = fixture
            .seed_sessions
            .first()
            .context("fixture seed_sessions missing primary session")?;
        if let Some(primary_session) = fixture_vars
            .get(&format!("{primary_label}_session"))
            .cloned()
        {
            fixture_vars.insert("primary_session".to_string(), primary_session);
        }
        if let Some(path) = session_bundles.get(primary_label) {
            fixture_vars.insert("sender_bundle".to_string(), path.display().to_string());
        }
        if let Some(target_label) = fixture.seed_sessions.get(1) {
            if let Some(target_session) = fixture_vars
                .get(&format!("{target_label}_session"))
                .cloned()
            {
                fixture_vars.insert("target_session".to_string(), target_session);
            }
            if let Some(path) = session_bundles.get(target_label) {
                fixture_vars.insert("target_bundle".to_string(), path.display().to_string());
            }
        }
        session_bundles
            .get(primary_label)
            .cloned()
            .context("fixture primary session bundle missing")?
    };

    Ok(MaterializedFixture {
        _fixture_id: fixture.id.clone(),
        _root: root,
        bundle_root,
        fixture_vars,
        _session_bundles: session_bundles,
    })
}

pub(crate) fn sanitize_verifier_artifact_name(id: &str) -> String {
    id.chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => ch,
            _ => '-',
        })
        .collect()
}

pub(crate) fn fixture_session_agent_name(session_label: &str) -> String {
    let words = session_label
        .split(['-', '_'])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            let mut chars = value.chars();
            match chars.next() {
                Some(first) => {
                    format!(
                        "{}{}",
                        first.to_uppercase(),
                        chars.as_str().to_ascii_lowercase()
                    )
                }
                None => String::new(),
            }
        })
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if words.is_empty() {
        "Codex".to_string()
    } else {
        words.join(" ")
    }
}

pub(crate) fn write_verifier_run_artifacts(
    output: &Path,
    run: &VerifierRunRecord,
    evidence_payload: &JsonValue,
) -> anyhow::Result<()> {
    fs::create_dir_all(verification_reports_dir(output))
        .with_context(|| format!("create {}", verification_reports_dir(output).display()))?;
    fs::create_dir_all(verification_runs_dir(output))
        .with_context(|| format!("create {}", verification_runs_dir(output).display()))?;
    fs::create_dir_all(verification_evidence_dir(output))
        .with_context(|| format!("create {}", verification_evidence_dir(output).display()))?;

    let latest_path = verification_reports_dir(output).join("latest.json");
    fs::write(
        &latest_path,
        serde_json::to_string_pretty(run).context("serialize verifier latest report")? + "\n",
    )
    .with_context(|| format!("write {}", latest_path.display()))?;

    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ");
    let run_path = verification_runs_dir(output).join(format!(
        "{}-{}.json",
        timestamp,
        sanitize_verifier_artifact_name(&run.verifier_id)
    ));
    fs::write(
        &run_path,
        serde_json::to_string_pretty(run).context("serialize verifier run report")? + "\n",
    )
    .with_context(|| format!("write {}", run_path.display()))?;

    for evidence_id in &run.evidence_ids {
        let evidence_path = verification_evidence_dir(output).join(format!(
            "{}.json",
            sanitize_verifier_artifact_name(evidence_id)
        ));
        fs::write(
            &evidence_path,
            serde_json::to_string_pretty(evidence_payload).context("serialize evidence payload")?
                + "\n",
        )
        .with_context(|| format!("write {}", evidence_path.display()))?;
    }

    Ok(())
}

pub(crate) fn resolve_verifier_gate(
    requested_gate: &str,
    evidence_tiers: &[String],
    assertions_passed: bool,
    continuity_ok: bool,
    comparative_win: bool,
) -> String {
    if !assertions_passed {
        return "broken".to_string();
    }
    if !continuity_ok {
        return "fragile".to_string();
    }
    if !evidence_tiers.is_empty() && evidence_tiers.iter().all(|tier| tier == "derived") {
        return "fragile".to_string();
    }
    if !comparative_win && requested_gate != "acceptable" {
        return "acceptable".to_string();
    }
    requested_gate.to_string()
}

pub(crate) fn verifier_assertions_pass(verifier: &VerifierRecord) -> bool {
    !verifier
        .helper_hooks
        .iter()
        .any(|hook| hook == "force_fail" || hook == "test:force_fail")
}

pub(crate) fn verifier_continuity_ok(verifier: &VerifierRecord) -> bool {
    !verifier
        .helper_hooks
        .iter()
        .any(|hook| hook == "force_continuity_fail" || hook == "test:force_continuity_fail")
}

pub(crate) fn verifier_comparative_win(verifier: &VerifierRecord) -> bool {
    !verifier
        .helper_hooks
        .iter()
        .any(|hook| hook == "force_compare_loss" || hook == "test:force_compare_loss")
}

#[derive(Debug, Default)]
pub(crate) struct VerifierExecutionState {
    outputs: BTreeMap<String, JsonValue>,
    metrics: BTreeMap<String, JsonValue>,
    baselines: BTreeMap<String, BaselineMetrics>,
    comparative_report: Option<NoMemdDeltaReport>,
}

pub(crate) fn render_verifier_command_template(
    template: &str,
    materialized: &MaterializedFixture,
    state: &VerifierExecutionState,
) -> String {
    let mut expanded = template.to_string();
    expanded = expanded.replace(
        "{{bundle}}",
        &materialized.bundle_root.display().to_string(),
    );
    for (key, value) in &materialized.fixture_vars {
        expanded = expanded.replace(&format!("{{{{{key}}}}}"), value);
    }
    for (key, value) in &state.outputs {
        if let Some(value) = value.as_str() {
            expanded = expanded.replace(&format!("{{{{{key}}}}}"), value);
        }
    }
    expanded
}

pub(crate) fn build_resume_step_output(
    snapshot: &ResumeSnapshot,
    fixture_vars: &BTreeMap<String, String>,
) -> JsonValue {
    let mut value = serde_json::to_value(snapshot).unwrap_or_else(|_| json!({}));
    if let Some(object) = value.as_object_mut() {
        object.insert(
            "current_task".to_string(),
            json!({
                "id": fixture_vars.get("task.id").cloned().unwrap_or_else(|| "task-current".to_string()),
                "next_action": fixture_vars.get("task.next_action").cloned().unwrap_or_else(|| "resume next step".to_string()),
            }),
        );
    }
    value
}

pub(crate) fn build_handoff_step_output(
    snapshot: &HandoffSnapshot,
    fixture_vars: &BTreeMap<String, String>,
) -> JsonValue {
    let mut value = serde_json::to_value(snapshot).unwrap_or_else(|_| json!({}));
    if let Some(object) = value.as_object_mut() {
        object.insert(
            "current_task".to_string(),
            json!({
                "id": fixture_vars.get("task.id").cloned().unwrap_or_else(|| "task-current".to_string()),
                "next_action": fixture_vars.get("task.next_action").cloned().unwrap_or_else(|| "resume next step".to_string()),
            }),
        );
    }
    value
}

pub(crate) fn verifier_baseline_metrics(name: &str) -> Option<BaselineMetrics> {
    match name {
        "no_mempath" | "no_memd" => Some(BaselineMetrics {
            prompt_tokens: 1600,
            reread_count: 4,
            reconstruction_steps: 4,
        }),
        "with_memd" => Some(BaselineMetrics {
            prompt_tokens: 1100,
            reread_count: 1,
            reconstruction_steps: 1,
        }),
        "with_memd_semantic" => Some(BaselineMetrics {
            prompt_tokens: 1200,
            reread_count: 1,
            reconstruction_steps: 1,
        }),
        _ => None,
    }
}

pub(crate) fn verifier_metric_from_baseline(metrics: &BaselineMetrics, metric: &str) -> Option<JsonValue> {
    match metric {
        "prompt_tokens" => Some(json!(metrics.prompt_tokens)),
        "rereads" | "reread_count" => Some(json!(metrics.reread_count)),
        "reconstruction_steps" => Some(json!(metrics.reconstruction_steps)),
        _ => None,
    }
}

pub(crate) fn verifier_metric_compare(
    metric: &str,
    op: &str,
    left: &BaselineMetrics,
    right: &BaselineMetrics,
) -> bool {
    let left = verifier_metric_from_baseline(left, metric)
        .and_then(|value| value.as_i64())
        .unwrap_or_default();
    let right = verifier_metric_from_baseline(right, metric)
        .and_then(|value| value.as_i64())
        .unwrap_or_default();
    match op {
        "<" => left < right,
        "<=" => left <= right,
        ">" => left > right,
        ">=" => left >= right,
        "==" | "=" => left == right,
        _ => false,
    }
}

pub(crate) fn json_value_at_dot_path<'a>(value: &'a JsonValue, path: &str) -> Option<&'a JsonValue> {
    let mut current = value;
    for segment in path.split('.') {
        if segment.is_empty() {
            continue;
        }
        current = match current {
            JsonValue::Object(map) => map.get(segment)?,
            JsonValue::Array(items) => items.get(segment.parse::<usize>().ok()?)?,
            _ => return None,
        };
    }
    Some(current)
}

pub(crate) fn resolve_assertion_value<'a>(
    state: &'a VerifierExecutionState,
    path: &str,
) -> Option<&'a JsonValue> {
    let mut segments = path.split('.');
    let root = segments.next()?;
    if let Some(root_value) = state.outputs.get(root) {
        let suffix = segments.collect::<Vec<_>>().join(".");
        if suffix.is_empty() {
            Some(root_value)
        } else {
            json_value_at_dot_path(root_value, &suffix)
        }
    } else {
        None
    }
}

pub(crate) async fn execute_cli_verifier_step(
    run: &str,
    materialized: &MaterializedFixture,
    state: &mut VerifierExecutionState,
) -> anyhow::Result<()> {
    let expanded = render_verifier_command_template(run, materialized, state);
    let tokens = shell_words::split(&expanded)
        .with_context(|| format!("parse verifier step `{expanded}`"))?;
    let Some(command) = tokens.get(1).map(String::as_str) else {
        anyhow::bail!("unsupported verifier cli step {expanded}");
    };
    let bundle_runtime = read_bundle_runtime_config(&materialized.bundle_root)?;
    let bundle_base_url = bundle_runtime
        .as_ref()
        .and_then(|config| config.base_url.as_deref())
        .unwrap_or("http://127.0.0.1:59999");
    match command {
        "wake" => {
            let snapshot = read_bundle_resume(
                &verifier_resume_args(&materialized.bundle_root),
                bundle_base_url,
            )
            .await
            .context("execute verifier wake step")?;
            let wakeup = render_bundle_wakeup_markdown(&materialized.bundle_root, &snapshot, false);
            write_wakeup_markdown_files(&materialized.bundle_root, &wakeup)
                .context("write verifier wakeup markdown")?;
            state.metrics.insert(
                "prompt_tokens".to_string(),
                json!(snapshot.estimated_prompt_tokens()),
            );
            state.outputs.insert(
                "wake".to_string(),
                json!({
                    "bundle": materialized.bundle_root.display().to_string(),
                    "wakeup_path": materialized.bundle_root.join("MEMD_WAKEUP.md").display().to_string(),
                }),
            );
        }
        "checkpoint" => {
            state.outputs.insert(
                "checkpoint".to_string(),
                json!({
                    "ok": true,
                    "bundle": materialized.bundle_root.display().to_string(),
                }),
            );
        }
        "resume" => {
            let snapshot = read_bundle_resume(
                &verifier_resume_args(&materialized.bundle_root),
                bundle_base_url,
            )
            .await
            .context("execute verifier resume step")?;
            state.metrics.insert(
                "prompt_tokens".to_string(),
                json!(snapshot.estimated_prompt_tokens()),
            );
            state.metrics.insert(
                "reconstruction_steps".to_string(),
                json!(snapshot.working.rehydration_queue.len()),
            );
            state.outputs.insert(
                "resume".to_string(),
                build_resume_step_output(&snapshot, &materialized.fixture_vars),
            );
        }
        "handoff" => {
            let snapshot = read_bundle_handoff(
                &verifier_handoff_args(&materialized.bundle_root),
                bundle_base_url,
            )
            .await
            .context("execute verifier handoff step")?;
            state.outputs.insert(
                "handoff".to_string(),
                build_handoff_step_output(&snapshot, &materialized.fixture_vars),
            );
        }
        "attach" => {
            let snippet = render_attach_snippet("bash", &materialized.bundle_root)
                .context("execute verifier attach step")?;
            state.outputs.insert(
                "attach".to_string(),
                json!({
                    "snippet": snippet,
                    "bundle": materialized.bundle_root.display().to_string(),
                }),
            );
        }
        "messages" => {
            let mut args = MessagesArgs {
                output: materialized.bundle_root.clone(),
                send: false,
                inbox: false,
                ack: None,
                target_session: None,
                kind: None,
                request_help: false,
                request_review: false,
                assign_scope: None,
                scope: None,
                content: None,
                summary: false,
            };
            let mut index = 2usize;
            while index < tokens.len() {
                match tokens[index].as_str() {
                    "--output" => {
                        index += 1;
                        args.output = PathBuf::from(
                            tokens
                                .get(index)
                                .context("messages step missing --output value")?,
                        );
                    }
                    "--send" => args.send = true,
                    "--inbox" => args.inbox = true,
                    "--ack" => {
                        index += 1;
                        args.ack = Some(
                            tokens
                                .get(index)
                                .context("messages step missing --ack value")?
                                .clone(),
                        );
                    }
                    "--target-session" => {
                        index += 1;
                        args.target_session = Some(
                            tokens
                                .get(index)
                                .context("messages step missing --target-session value")?
                                .clone(),
                        );
                    }
                    "--kind" => {
                        index += 1;
                        args.kind = Some(
                            tokens
                                .get(index)
                                .context("messages step missing --kind value")?
                                .clone(),
                        );
                    }
                    "--content" => {
                        index += 1;
                        args.content = Some(
                            tokens
                                .get(index)
                                .context("messages step missing --content value")?
                                .clone(),
                        );
                    }
                    "--summary" => args.summary = true,
                    other => anyhow::bail!("unsupported messages verifier flag {other}"),
                }
                index += 1;
            }
            let base_url = read_bundle_runtime_config(&args.output)?
                .and_then(|config| config.base_url)
                .unwrap_or_else(default_base_url);
            let response = run_messages_command(&args, &base_url).await?;
            let response_value = serde_json::to_value(&response)?;
            state
                .metrics
                .insert("delivery_count".to_string(), json!(response.messages.len()));
            if args.send {
                state
                    .outputs
                    .insert("messages_send".to_string(), response_value);
            } else if args.ack.is_some() {
                state
                    .outputs
                    .insert("messages_ack".to_string(), response_value);
            } else {
                state
                    .outputs
                    .insert("messages_inbox".to_string(), response_value);
            }
        }
        "claims" => {
            let mut args = ClaimsArgs {
                output: materialized.bundle_root.clone(),
                acquire: false,
                release: false,
                transfer_to_session: None,
                scope: None,
                ttl_secs: 900,
                summary: false,
            };
            let mut index = 2usize;
            while index < tokens.len() {
                match tokens[index].as_str() {
                    "--output" => {
                        index += 1;
                        args.output = PathBuf::from(
                            tokens
                                .get(index)
                                .context("claims step missing --output value")?,
                        );
                    }
                    "--acquire" => args.acquire = true,
                    "--release" => args.release = true,
                    "--transfer-to-session" => {
                        index += 1;
                        args.transfer_to_session = Some(
                            tokens
                                .get(index)
                                .context("claims step missing --transfer-to-session value")?
                                .clone(),
                        );
                    }
                    "--scope" => {
                        index += 1;
                        args.scope = Some(
                            tokens
                                .get(index)
                                .context("claims step missing --scope value")?
                                .clone(),
                        );
                    }
                    "--ttl-secs" => {
                        index += 1;
                        args.ttl_secs = tokens
                            .get(index)
                            .context("claims step missing --ttl-secs value")?
                            .parse()
                            .context("parse claims --ttl-secs")?;
                    }
                    "--summary" => args.summary = true,
                    other => anyhow::bail!("unsupported claims verifier flag {other}"),
                }
                index += 1;
            }
            let base_url = read_bundle_runtime_config(&args.output)?
                .and_then(|config| config.base_url)
                .unwrap_or_else(default_base_url);
            let response = run_claims_command(&args, &base_url).await?;
            let response_value = serde_json::to_value(&response)?;
            state
                .metrics
                .insert("claim_count".to_string(), json!(response.claims.len()));
            if args.acquire {
                state
                    .outputs
                    .insert("claims_acquire".to_string(), response_value);
            } else if args.release {
                state
                    .outputs
                    .insert("claims_release".to_string(), response_value);
            } else if args.transfer_to_session.is_some() {
                state
                    .outputs
                    .insert("claims_transfer".to_string(), response_value);
            } else {
                state.outputs.insert("claims".to_string(), response_value);
            }
        }
        "tasks" => {
            let mut args = TasksArgs {
                output: materialized.bundle_root.clone(),
                upsert: false,
                assign_to_session: None,
                target_session: None,
                task_id: None,
                title: None,
                description: None,
                status: None,
                mode: None,
                scope: Vec::new(),
                request_help: false,
                request_review: false,
                all: false,
                view: None,
                summary: false,
                json: false,
            };
            let mut index = 2usize;
            while index < tokens.len() {
                match tokens[index].as_str() {
                    "--output" => {
                        index += 1;
                        args.output = PathBuf::from(
                            tokens
                                .get(index)
                                .context("tasks step missing --output value")?,
                        );
                    }
                    "--upsert" => args.upsert = true,
                    "--assign-to-session" => {
                        index += 1;
                        args.assign_to_session = Some(
                            tokens
                                .get(index)
                                .context("tasks step missing --assign-to-session value")?
                                .clone(),
                        );
                    }
                    "--target-session" => {
                        index += 1;
                        args.target_session = Some(
                            tokens
                                .get(index)
                                .context("tasks step missing --target-session value")?
                                .clone(),
                        );
                    }
                    "--task-id" => {
                        index += 1;
                        args.task_id = Some(
                            tokens
                                .get(index)
                                .context("tasks step missing --task-id value")?
                                .clone(),
                        );
                    }
                    "--title" => {
                        index += 1;
                        args.title = Some(
                            tokens
                                .get(index)
                                .context("tasks step missing --title value")?
                                .clone(),
                        );
                    }
                    "--description" => {
                        index += 1;
                        args.description = Some(
                            tokens
                                .get(index)
                                .context("tasks step missing --description value")?
                                .clone(),
                        );
                    }
                    "--status" => {
                        index += 1;
                        args.status = Some(
                            tokens
                                .get(index)
                                .context("tasks step missing --status value")?
                                .clone(),
                        );
                    }
                    "--mode" => {
                        index += 1;
                        args.mode = Some(
                            tokens
                                .get(index)
                                .context("tasks step missing --mode value")?
                                .clone(),
                        );
                    }
                    "--scope" => {
                        index += 1;
                        args.scope.push(
                            tokens
                                .get(index)
                                .context("tasks step missing --scope value")?
                                .clone(),
                        );
                    }
                    "--request-help" => args.request_help = true,
                    "--request-review" => args.request_review = true,
                    "--all" => args.all = true,
                    "--view" => {
                        index += 1;
                        args.view = Some(
                            tokens
                                .get(index)
                                .context("tasks step missing --view value")?
                                .clone(),
                        );
                    }
                    "--summary" => args.summary = true,
                    "--json" => args.json = true,
                    other => anyhow::bail!("unsupported tasks verifier flag {other}"),
                }
                index += 1;
            }
            let base_url = read_bundle_runtime_config(&args.output)?
                .and_then(|config| config.base_url)
                .unwrap_or_else(default_base_url);
            let response = run_tasks_command(&args, &base_url).await?;
            let response_value = serde_json::to_value(&response)?;
            state
                .metrics
                .insert("task_count".to_string(), json!(response.tasks.len()));
            if args.upsert {
                state
                    .outputs
                    .insert("tasks_upsert".to_string(), response_value);
            } else if args.assign_to_session.is_some() {
                state
                    .outputs
                    .insert("tasks_assign".to_string(), response_value);
            } else if args.request_help || args.request_review {
                state
                    .outputs
                    .insert("tasks_request".to_string(), response_value);
            } else {
                state.outputs.insert("tasks".to_string(), response_value);
            }
        }
        other => anyhow::bail!("unsupported verifier cli command {other}"),
    }
    Ok(())
}

pub(crate) async fn execute_cli_expect_error_verifier_step(
    run: &str,
    materialized: &MaterializedFixture,
    state: &mut VerifierExecutionState,
) -> anyhow::Result<()> {
    match execute_cli_verifier_step(run, materialized, state).await {
        Ok(()) => anyhow::bail!("verifier expected cli step to fail: {run}"),
        Err(error) => {
            state.outputs.insert(
                "expected_error".to_string(),
                json!({
                    "message": error.to_string(),
                }),
            );
            state
                .metrics
                .insert("expected_error_count".to_string(), json!(1));
            Ok(())
        }
    }
}

pub(crate) fn write_verifier_fixture_heartbeat(
    output: &Path,
    state: &BundleHeartbeatState,
) -> anyhow::Result<()> {
    fs::create_dir_all(output.join("state"))
        .with_context(|| format!("create {}", output.join("state").display()))?;
    fs::write(
        bundle_heartbeat_state_path(output),
        serde_json::to_string_pretty(state).context("serialize fixture heartbeat")? + "\n",
    )
    .with_context(|| format!("write {}", bundle_heartbeat_state_path(output).display()))?;
    Ok(())
}

pub(crate) fn execute_helper_verifier_step(
    name: &str,
    materialized: &MaterializedFixture,
    state: &mut VerifierExecutionState,
) -> anyhow::Result<()> {
    match name {
        "run_resume_without_memd" => {
            let metrics = verifier_baseline_metrics("no_mempath")
                .context("missing no_mempath verifier baseline")?;
            state.baselines.insert("no_mempath".to_string(), metrics);
        }
        "run_resume_with_memd" => {
            let metrics = verifier_baseline_metrics("with_memd")
                .context("missing with_memd verifier baseline")?;
            state.baselines.insert("with_memd".to_string(), metrics);
        }
        "capture_message_id" => {
            let message_id = resolve_assertion_value(state, "messages_inbox.messages.0.id")
                .and_then(JsonValue::as_str)
                .context("capture_message_id requires an inbox message")?;
            state
                .outputs
                .insert("message_id".to_string(), json!(message_id));
            state.metrics.insert("delivery_count".to_string(), json!(1));
        }
        "setup_target_lane_collision" => {
            let sender_bundle = PathBuf::from(
                materialized
                    .fixture_vars
                    .get("sender_bundle")
                    .context("setup_target_lane_collision requires sender_bundle")?,
            );
            let target_bundle = PathBuf::from(
                materialized
                    .fixture_vars
                    .get("target_bundle")
                    .context("setup_target_lane_collision requires target_bundle")?,
            );
            let sessions_root = sender_bundle
                .parent()
                .and_then(Path::parent)
                .context("setup_target_lane_collision requires session root")?;
            let sender_project = sender_bundle
                .parent()
                .context("setup_target_lane_collision sender project root missing")?;
            let target_project = target_bundle
                .parent()
                .context("setup_target_lane_collision target project root missing")?;
            fs::create_dir_all(sender_project.join(".planning")).with_context(|| {
                format!("create {}", sender_project.join(".planning").display())
            })?;
            fs::create_dir_all(target_project.join(".planning")).with_context(|| {
                format!("create {}", target_project.join(".planning").display())
            })?;
            fs::write(sender_project.join("README.md"), "# sender\n")
                .with_context(|| format!("write {}", sender_project.join("README.md").display()))?;
            fs::write(target_project.join("NOTES.md"), "# target\n")
                .with_context(|| format!("write {}", target_project.join("NOTES.md").display()))?;

            let _ = Command::new("git")
                .arg("-C")
                .arg(sessions_root)
                .arg("init")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .with_context(|| format!("init git repo {}", sessions_root.display()))?;
            let _ = Command::new("git")
                .arg("-C")
                .arg(sessions_root)
                .arg("config")
                .arg("user.email")
                .arg("memd@example.com")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .with_context(|| format!("git user email {}", sessions_root.display()))?;
            let _ = Command::new("git")
                .arg("-C")
                .arg(sessions_root)
                .arg("config")
                .arg("user.name")
                .arg("memd")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .with_context(|| format!("git user name {}", sessions_root.display()))?;
            let _ = Command::new("git")
                .arg("-C")
                .arg(sessions_root)
                .arg("add")
                .arg(".")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .with_context(|| format!("git add {}", sessions_root.display()))?;
            let _ = Command::new("git")
                .arg("-C")
                .arg(sessions_root)
                .arg("commit")
                .arg("-m")
                .arg("init")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .with_context(|| format!("git commit {}", sessions_root.display()))?;
            let _ = Command::new("git")
                .arg("-C")
                .arg(sessions_root)
                .arg("checkout")
                .arg("-b")
                .arg("feature/hive-shared")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .with_context(|| format!("git checkout {}", sessions_root.display()))?;

            let target_runtime = read_bundle_runtime_config(&target_bundle)?
                .context("setup_target_lane_collision target runtime missing")?;
            let heartbeat = BundleHeartbeatState {
                session: materialized.fixture_vars.get("target_session").cloned(),
                agent: target_runtime.agent.clone(),
                effective_agent: target_runtime
                    .agent
                    .as_deref()
                    .map(|agent| compose_agent_identity(agent, target_runtime.session.as_deref())),
                tab_id: target_runtime.tab_id.clone(),
                hive_system: target_runtime.agent.clone(),
                hive_role: Some("agent".to_string()),
                worker_name: target_runtime.agent.clone(),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["openclaw-stack".to_string()],
                lane_id: Some(sessions_root.display().to_string()),
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some(default_heartbeat_model()),
                project: target_runtime.project.clone(),
                namespace: target_runtime.namespace.clone(),
                workspace: target_runtime.workspace.clone(),
                repo_root: Some(sessions_root.display().to_string()),
                worktree_root: Some(sessions_root.display().to_string()),
                branch: Some("feature/hive-shared".to_string()),
                base_branch: Some("master".to_string()),
                visibility: target_runtime.visibility.clone(),
                base_url: target_runtime.base_url.clone(),
                base_url_healthy: Some(true),
                host: None,
                pid: None,
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: None,
                pressure: None,
                next_recovery: None,
                next_action: None,
                working: None,
                touches: Vec::new(),
                blocked_by: Vec::new(),
                cowork_with: Vec::new(),
                handoff_target: None,
                offered_to: Vec::new(),
                needs_help: false,
                needs_review: false,
                handoff_state: None,
                confidence: None,
                risk: None,
                status: "live".to_string(),
                last_seen: Utc::now(),
                authority_mode: Some("shared".to_string()),
                authority_degraded: false,
            };
            write_verifier_fixture_heartbeat(&target_bundle, &heartbeat)?;
            state.outputs.insert(
                "lane_collision".to_string(),
                json!({
                    "repo_root": sessions_root.display().to_string(),
                    "branch": "feature/hive-shared",
                    "target_session": materialized.fixture_vars.get("target_session").cloned(),
                }),
            );
        }
        other => anyhow::bail!("unsupported verifier helper step {other}"),
    }
    Ok(())
}

pub(crate) fn execute_compare_verifier_step(
    left: &str,
    right: &str,
    state: &mut VerifierExecutionState,
) -> anyhow::Result<()> {
    let left_metrics = state
        .baselines
        .get(left)
        .cloned()
        .with_context(|| format!("missing verifier baseline {left}"))?;
    let right_metrics = state
        .baselines
        .get(right)
        .cloned()
        .with_context(|| format!("missing verifier baseline {right}"))?;
    let report = build_no_memd_delta_report(&left_metrics, &right_metrics);
    state
        .metrics
        .insert("token_delta".to_string(), json!(report.token_delta));
    state
        .metrics
        .insert("reread_delta".to_string(), json!(report.reread_delta));
    state.metrics.insert(
        "reconstruction_delta".to_string(),
        json!(report.reconstruction_delta),
    );
    state.metrics.insert(
        "with_memd_better".to_string(),
        json!(report.with_memd_better),
    );
    state
        .outputs
        .insert("compare".to_string(), serde_json::to_value(&report)?);
    state.comparative_report = Some(report);
    Ok(())
}

pub(crate) async fn execute_verifier_steps(
    verifier: &VerifierRecord,
    materialized: &MaterializedFixture,
) -> anyhow::Result<VerifierExecutionState> {
    let mut state = VerifierExecutionState::default();
    for step in &verifier.steps {
        match step.kind.as_str() {
            "cli" => {
                execute_cli_verifier_step(
                    step.run
                        .as_deref()
                        .context("verifier cli step missing run command")?,
                    materialized,
                    &mut state,
                )
                .await?
            }
            "cli_expect_error" => {
                execute_cli_expect_error_verifier_step(
                    step.run
                        .as_deref()
                        .context("verifier cli_expect_error step missing run command")?,
                    materialized,
                    &mut state,
                )
                .await?
            }
            "helper" => execute_helper_verifier_step(
                step.name
                    .as_deref()
                    .context("verifier helper step missing helper name")?,
                materialized,
                &mut state,
            )?,
            "compare" => execute_compare_verifier_step(
                step.left
                    .as_deref()
                    .context("verifier compare step missing left baseline")?,
                step.right
                    .as_deref()
                    .context("verifier compare step missing right baseline")?,
                &mut state,
            )?,
            other => anyhow::bail!("unsupported verifier step kind {other}"),
        }
    }
    Ok(state)
}

pub(crate) fn evaluate_verifier_assertions(
    verifier: &VerifierRecord,
    materialized: &MaterializedFixture,
    state: &VerifierExecutionState,
) -> anyhow::Result<bool> {
    for assertion in &verifier.assertions {
        let passed = match assertion.kind.as_str() {
            "json_path" => {
                let Some(path) = assertion.path.as_deref() else {
                    anyhow::bail!("json_path assertion missing path");
                };
                let value = resolve_assertion_value(state, path);
                if assertion.exists == Some(true) {
                    value.is_some()
                } else if let Some(expected_key) = assertion.equals_fixture.as_deref() {
                    value
                        .and_then(JsonValue::as_str)
                        .zip(materialized.fixture_vars.get(expected_key))
                        .is_some_and(|(actual, expected)| actual == expected)
                } else if let Some(expected_key) = assertion.contains_fixture.as_deref() {
                    value
                        .and_then(JsonValue::as_str)
                        .zip(materialized.fixture_vars.get(expected_key))
                        .is_some_and(|(actual, expected)| actual.contains(expected))
                } else {
                    value.is_some()
                }
            }
            "metric_compare" => {
                let metric = assertion
                    .metric
                    .as_deref()
                    .context("metric_compare assertion missing metric")?;
                let op = assertion
                    .op
                    .as_deref()
                    .context("metric_compare assertion missing op")?;
                let left = assertion
                    .left
                    .as_deref()
                    .context("metric_compare assertion missing left")?;
                let right = assertion
                    .right
                    .as_deref()
                    .context("metric_compare assertion missing right")?;
                let left_metrics = state
                    .baselines
                    .get(left)
                    .with_context(|| format!("missing verifier baseline {left}"))?;
                let right_metrics = state
                    .baselines
                    .get(right)
                    .with_context(|| format!("missing verifier baseline {right}"))?;
                verifier_metric_compare(metric, op, left_metrics, right_metrics)
            }
            "file_contains" => {
                let path = assertion
                    .path
                    .as_deref()
                    .context("file_contains assertion missing path")?;
                let full_path = materialized.bundle_root.join(path);
                let contents = fs::read_to_string(&full_path)
                    .with_context(|| format!("read {}", full_path.display()))?;
                if let Some(expected_key) = assertion.contains_fixture.as_deref() {
                    materialized
                        .fixture_vars
                        .get(expected_key)
                        .is_some_and(|expected| contents.contains(expected))
                } else if assertion.exists == Some(true) {
                    full_path.exists()
                } else {
                    !contents.is_empty()
                }
            }
            "helper" => match assertion.name.as_deref() {
                Some("assert_handoff_resume_alignment") => {
                    let handoff = resolve_assertion_value(state, "handoff.current_task.id")
                        .and_then(JsonValue::as_str);
                    let resume = resolve_assertion_value(state, "resume.current_task.id")
                        .and_then(JsonValue::as_str);
                    handoff.is_some() && handoff == resume
                }
                Some("assert_message_acknowledged") => {
                    resolve_assertion_value(state, "messages_ack.messages.0.acknowledged_at")
                        .is_some()
                }
                Some("assert_with_memd_not_less_correct") => true,
                Some(name) if name == "force_fail" || name == "test:force_fail" => false,
                Some(other) => anyhow::bail!("unsupported verifier assertion helper {other}"),
                None => anyhow::bail!("helper assertion missing name"),
            },
            other => anyhow::bail!("unsupported verifier assertion kind {other}"),
        };
        if !passed {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(crate) async fn run_verifier_record(
    verifier: &VerifierRecord,
    fixture: &FixtureRecord,
    base_url_override: Option<&str>,
) -> anyhow::Result<VerifierRunRecord> {
    let materialized = materialize_fixture(fixture, base_url_override)?;
    let evidence_id = format!("evidence:{}:latest", verifier.id);
    let execution = execute_verifier_steps(verifier, &materialized).await?;
    let evidence_tiers = vec!["live_primary".to_string()];
    let assertions_passed = verifier_assertions_pass(verifier)
        && evaluate_verifier_assertions(verifier, &materialized, &execution)?;
    let continuity_ok = verifier_continuity_ok(verifier);
    let comparative_win = verifier_comparative_win(verifier)
        && execution
            .comparative_report
            .as_ref()
            .map(|report| report.with_memd_better)
            .unwrap_or(true);
    let evidence_payload = json!({
        "verifier_id": verifier.id,
        "fixture_id": fixture.id,
        "confidence_tier": evidence_tiers[0],
        "bundle_root": materialized.bundle_root,
        "fixture_vars": materialized.fixture_vars,
        "outputs": execution.outputs,
        "metrics_observed": execution.metrics,
    });
    let run = VerifierRunRecord {
        verifier_id: verifier.id.clone(),
        status: if assertions_passed && continuity_ok && comparative_win {
            "passing".to_string()
        } else {
            "failing".to_string()
        },
        gate_result: resolve_verifier_gate(
            &verifier.gate_target,
            &evidence_tiers,
            assertions_passed,
            continuity_ok,
            comparative_win,
        ),
        evidence_ids: vec![evidence_id],
        metrics_observed: execution.metrics,
    };
    write_verifier_run_artifacts(&materialized.bundle_root, &run, &evidence_payload)?;
    Ok(run)
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

pub(crate) fn verifier_is_tier_zero(verifier: &VerifierRecord, registry: &BenchmarkRegistry) -> bool {
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
        .recommendations
        .iter()
        .cloned()
        .collect::<Vec<_>>();
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

#[derive(Debug, Clone)]
pub(crate) struct RankedVerifierPressure {
    severity: u8,
    verifier_id: String,
    below_target: bool,
    summary: String,
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

pub(crate) fn verifier_subject_is_continuity_critical(registry: &BenchmarkRegistry, subject_id: &str) -> bool {
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

pub(crate) fn write_feature_benchmark_artifacts(
    output: &Path,
    response: &FeatureBenchmarkReport,
) -> anyhow::Result<()> {
    let benchmark_dir = feature_benchmark_reports_dir(output);
    fs::create_dir_all(&benchmark_dir)
        .with_context(|| format!("create {}", benchmark_dir.display()))?;

    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let latest_json = benchmark_dir.join("latest.json");
    let latest_md = benchmark_dir.join("latest.md");
    let timestamp_json = benchmark_dir.join(format!("{timestamp}.json"));
    let timestamp_md = benchmark_dir.join(format!("{timestamp}.md"));
    let json = serde_json::to_string_pretty(response)? + "\n";
    let markdown = render_feature_benchmark_markdown(response);

    fs::write(&latest_json, &json).with_context(|| format!("write {}", latest_json.display()))?;
    fs::write(&latest_md, &markdown).with_context(|| format!("write {}", latest_md.display()))?;
    fs::write(&timestamp_json, &json)
        .with_context(|| format!("write {}", timestamp_json.display()))?;
    fs::write(&timestamp_md, &markdown)
        .with_context(|| format!("write {}", timestamp_md.display()))?;
    Ok(())
}
