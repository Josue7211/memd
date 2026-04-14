use super::*;
use crate::cli::command_catalog::build_command_catalog;
use crate::cli::skill_catalog::build_skill_catalog;

pub(crate) fn top_level_command_names() -> BTreeSet<String> {
    let mut root = Cli::command();
    root.build();
    root.get_subcommands()
        .map(|command| command.get_name().to_string())
        .collect()
}

pub(crate) fn benchmark_command_coverage(
    available: &BTreeSet<String>,
    required: &[&str],
) -> (usize, Vec<String>) {
    let missing = required
        .iter()
        .filter(|name| !available.contains(**name))
        .map(|name| (*name).to_string())
        .collect::<Vec<_>>();
    (required.len().saturating_sub(missing.len()), missing)
}

pub(crate) fn benchmark_status(score: u8) -> &'static str {
    if score >= 80 {
        "pass"
    } else if score >= 60 {
        "warn"
    } else {
        "fail"
    }
}

pub(crate) fn benchmark_dim_score(
    report: &CompiledMemoryQualityReport,
    name: &str,
    fallback: u8,
) -> u8 {
    report
        .dimensions
        .iter()
        .find(|dimension| dimension.name == name)
        .map(|dimension| dimension.score)
        .unwrap_or(fallback)
}

pub(crate) fn benchmark_area_from_commands(
    slug: &str,
    name: &str,
    available: &BTreeSet<String>,
    required: &[&str],
    evidence: Vec<String>,
    recommendations: Vec<String>,
    bonus_score: u8,
) -> FeatureBenchmarkArea {
    let (implemented_commands, missing) = benchmark_command_coverage(available, required);
    let coverage_score = if required.is_empty() {
        100
    } else {
        ((implemented_commands * 100) / required.len()) as u8
    };
    let score = ((coverage_score as u16 * 70) / 100 + (bonus_score as u16 * 30) / 100) as u8;
    let mut merged_evidence = evidence;
    merged_evidence.push(format!(
        "command_coverage={}/{}",
        implemented_commands,
        required.len()
    ));
    let mut merged_recommendations = recommendations;
    if !missing.is_empty() {
        merged_recommendations.push(format!(
            "fill missing command surfaces: {}",
            missing.join(", ")
        ));
    }
    FeatureBenchmarkArea {
        slug: slug.to_string(),
        name: name.to_string(),
        score,
        max_score: 100,
        status: benchmark_status(score).to_string(),
        implemented_commands,
        expected_commands: required.len(),
        evidence: merged_evidence,
        recommendations: merged_recommendations,
    }
}

pub(crate) fn build_benchmark_gap_candidates(registry: &BenchmarkRegistry) -> Vec<GapCandidate> {
    let mut candidates = Vec::new();

    let continuity_gaps = registry
        .features
        .iter()
        .filter(|feature| feature.continuity_critical && feature.coverage_status != "verified")
        .collect::<Vec<_>>();
    if !continuity_gaps.is_empty() {
        let evidence = continuity_gaps
            .iter()
            .take(5)
            .map(|feature| {
                format!(
                    "{} [{}] coverage={} loops={}",
                    feature.id,
                    feature.family,
                    feature.coverage_status,
                    feature.loop_ids.len()
                )
            })
            .collect::<Vec<_>>();
        candidates.push(GapCandidate {
            id: "benchmark:unbenchmarked_continuity_feature".to_string(),
            area: "benchmark".to_string(),
            priority: 98,
            severity: "high".to_string(),
            signal: "unbenchmarked_continuity_feature".to_string(),
            evidence,
            recommendation:
                "bench continuity-critical features before promoting the benchmark registry"
                    .to_string(),
        });
    }

    let missing_loop_ids = registry
        .features
        .iter()
        .filter(|feature| feature.loop_ids.is_empty())
        .collect::<Vec<_>>();
    if !missing_loop_ids.is_empty() {
        let evidence = missing_loop_ids
            .iter()
            .take(5)
            .map(|feature| {
                format!(
                    "{} [{}] coverage={} missing_loop_ids=1",
                    feature.id, feature.family, feature.coverage_status
                )
            })
            .collect::<Vec<_>>();
        candidates.push(GapCandidate {
            id: "benchmark:missing_loop_ids".to_string(),
            area: "benchmark".to_string(),
            priority: 86,
            severity: "medium".to_string(),
            signal: "missing_loop_ids".to_string(),
            evidence,
            recommendation: "assign loop IDs to every benchmarked feature".to_string(),
        });
    }

    candidates
}

pub(crate) fn render_benchmark_registry_benchmarks_markdown(
    repo_root: &Path,
    registry: &BenchmarkRegistry,
    benchmark: &FeatureBenchmarkReport,
) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd benchmark registry\n\n");
    markdown.push_str(&format!("- Root: `{}`\n", repo_root.display()));
    markdown.push_str(&"- Registry: `docs/verification/benchmark-registry.json`\n".to_string());
    markdown.push_str(&format!("- Version: `{}`\n", registry.version));
    markdown.push_str(&format!("- App goal: {}\n", registry.app_goal));
    markdown.push_str(&format!(
        "- Current benchmark score: `{}/{}`\n",
        benchmark.score, benchmark.max_score
    ));
    markdown.push_str(&format!(
        "- Quality dimensions: `{}`\n",
        registry.quality_dimensions.len()
    ));
    markdown.push_str(&format!("- Pillars: `{}`\n", registry.pillars.len()));
    markdown.push_str(&format!("- Families: `{}`\n", registry.families.len()));
    markdown.push_str(&format!("- Features: `{}`\n", registry.features.len()));
    markdown.push_str(&format!("- Journeys: `{}`\n", registry.journeys.len()));
    markdown.push_str(&format!("- Loops: `{}`\n", registry.loops.len()));
    markdown.push_str(&format!("- Scorecards: `{}`\n", registry.scorecards.len()));
    markdown.push_str(&format!(
        "- Evidence records: `{}`\n",
        registry.evidence.len()
    ));
    markdown.push_str(&format!("- Gates: `{}`\n", registry.gates.len()));
    markdown.push_str(&format!(
        "- Baseline modes: `{}`\n",
        registry.baseline_modes.len()
    ));
    markdown.push_str(&format!(
        "- Runtime policies: `{}`\n\n",
        registry.runtime_policies.len()
    ));

    markdown.push_str("## Pillars\n");
    for pillar in &registry.pillars {
        let family_count = registry
            .families
            .iter()
            .filter(|family| family.pillar == pillar.id)
            .count();
        let feature_count = registry
            .features
            .iter()
            .filter(|feature| feature.pillar == pillar.id)
            .count();
        markdown.push_str(&format!(
            "- `{}`: {} family surfaces, {} features\n",
            pillar.id, family_count, feature_count
        ));
    }
    markdown.push('\n');

    markdown.push_str("## Feature Coverage Snapshot\n");
    for feature in registry.features.iter().take(12) {
        markdown.push_str(&format!(
            "- `{}` [{}] {} | continuity={} | loops={}\n",
            feature.id,
            feature.family,
            feature.coverage_status,
            feature.continuity_critical,
            feature.loop_ids.len()
        ));
    }
    if registry.features.len() > 12 {
        markdown.push_str(&format!(
            "- ... and {} more features\n",
            registry.features.len() - 12
        ));
    }
    markdown.push('\n');

    markdown.push_str("## Quality Dimensions\n");
    for dimension in &registry.quality_dimensions {
        markdown.push_str(&format!(
            "- `{}` weight `{}`\n",
            dimension.id, dimension.weight
        ));
    }
    markdown.push('\n');
    markdown
}

pub(crate) fn render_benchmark_registry_loops_markdown(
    registry: &BenchmarkRegistry,
    coverage: &BenchmarkCoverageTelemetry,
) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd benchmark loops\n\n");
    markdown.push_str(&format!("- Loops: `{}`\n", registry.loops.len()));
    markdown.push_str(&format!("- Journeys: `{}`\n\n", registry.journeys.len()));
    markdown.push_str("| Loop | Type | Family | Baseline | Features | Journeys | Status |\n");
    markdown.push_str("| --- | --- | --- | --- | --- | --- | --- |\n");
    for loop_record in &registry.loops {
        markdown.push_str(&format!(
            "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` |\n",
            loop_record.id,
            loop_record.loop_type,
            loop_record.family,
            loop_record.baseline_mode,
            loop_record.covers_features.len(),
            loop_record.journey_ids.len(),
            loop_record.status
        ));
    }
    markdown.push('\n');
    markdown.push_str("## Coverage Gaps\n");
    if coverage.gap_candidates.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for candidate in &coverage.gap_candidates {
            markdown.push_str(&format!(
                "- `{}` [{}] {}\n",
                candidate.id, candidate.severity, candidate.recommendation
            ));
            for entry in &candidate.evidence {
                markdown.push_str(&format!("  - evidence: {}\n", entry));
            }
        }
    }
    markdown.push('\n');
    markdown.push_str("## Loop Coverage Notes\n");
    for loop_record in &registry.loops {
        markdown.push_str(&format!(
            "- `{}` probes `{}` and `{}`\n",
            loop_record.id, loop_record.workflow_probe, loop_record.adversarial_probe
        ));
    }
    markdown.push('\n');
    markdown
}

pub(crate) fn render_benchmark_registry_coverage_markdown(
    registry: &BenchmarkRegistry,
    benchmark: &FeatureBenchmarkReport,
    coverage: &BenchmarkCoverageTelemetry,
) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd benchmark coverage\n\n");
    markdown.push_str(&format!(
        "- Current benchmark score: `{}/{}`\n",
        benchmark.score, benchmark.max_score
    ));
    markdown.push_str(&format!(
        "- Feature coverage records: `{}`\n",
        registry.features.len()
    ));
    markdown.push_str(&format!(
        "- Journey coverage records: `{}`\n\n",
        registry.journeys.len()
    ));
    markdown.push_str("## Coverage Summary\n");
    markdown.push_str(&format!(
        "- continuity-critical total: `{}`\n",
        coverage.continuity_critical_total
    ));
    markdown.push_str(&format!(
        "- continuity-critical benchmarked: `{}`\n",
        coverage.continuity_critical_benchmarked
    ));
    markdown.push_str(&format!(
        "- missing loop count: `{}`\n",
        coverage.missing_loop_count
    ));
    markdown.push_str(&format!(
        "- with-memd losses: `{}`\n",
        coverage.with_memd_losses
    ));
    markdown.push('\n');
    markdown.push_str("## Continuity-Critical Features\n");
    let mut continuity_features = registry
        .features
        .iter()
        .filter(|feature| feature.continuity_critical)
        .collect::<Vec<_>>();
    continuity_features.sort_by(|left, right| {
        left.coverage_status
            .cmp(&right.coverage_status)
            .then_with(|| left.id.cmp(&right.id))
    });
    for feature in continuity_features {
        markdown.push_str(&format!(
            "- `{}` [{}] loops={} drift={}\n",
            feature.id,
            feature.coverage_status,
            feature.loop_ids.len(),
            feature.drift_risks.join("|")
        ));
    }
    markdown.push('\n');
    markdown.push_str("## Benchmark Gaps\n");
    if coverage.gap_candidates.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for candidate in &coverage.gap_candidates {
            markdown.push_str(&format!(
                "- `{}` [{}] {}\n",
                candidate.id, candidate.severity, candidate.recommendation
            ));
        }
    }
    markdown.push('\n');
    markdown.push_str("## Missing Loop IDs\n");
    let missing_loop_features = registry
        .features
        .iter()
        .filter(|feature| feature.loop_ids.is_empty())
        .collect::<Vec<_>>();
    if missing_loop_features.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for feature in missing_loop_features {
            markdown.push_str(&format!(
                "- `{}` [{}] missing loop IDs\n",
                feature.id, feature.coverage_status
            ));
        }
    }
    markdown.push('\n');
    markdown.push_str("## Journeys\n");
    for journey in &registry.journeys {
        markdown.push_str(&format!(
            "- `{}` [{}] features={} loops={} gate={}\n",
            journey.id,
            journey.goal,
            journey.feature_ids.len(),
            journey.loop_ids.len(),
            journey.gate_target
        ));
    }
    markdown.push('\n');
    markdown.push_str("## Current Benchmark Areas\n");
    for area in &benchmark.areas {
        markdown.push_str(&format!(
            "- `{}`: `{}/{}`\n",
            area.slug, area.score, area.max_score
        ));
    }
    markdown.push('\n');
    markdown
}

pub(crate) fn render_benchmark_registry_scores_markdown(
    registry: &BenchmarkRegistry,
    benchmark: &FeatureBenchmarkReport,
    continuity_journey_report: Option<&ContinuityJourneyReport>,
    comparative_report: Option<&NoMemdDeltaReport>,
) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd benchmark scores\n\n");
    markdown.push_str(&format!(
        "- Current benchmark score: `{}/{}`\n",
        benchmark.score, benchmark.max_score
    ));
    markdown.push_str(&format!(
        "- Area score count: `{}`\n\n",
        benchmark.areas.len()
    ));
    markdown.push_str("## Quality Dimension Weights\n");
    for dimension in &registry.quality_dimensions {
        markdown.push_str(&format!("- `{}` -> `{}`\n", dimension.id, dimension.weight));
    }
    markdown.push('\n');
    markdown.push_str("## Current Benchmark Areas\n");
    for area in &benchmark.areas {
        markdown.push_str(&format!(
            "- `{}`: `{}/{}`\n",
            area.name, area.score, area.max_score
        ));
    }
    if let Some(report) = comparative_report {
        markdown.push_str("\n## Comparative Evidence\n");
        markdown.push_str(&format!(
            "- no-memd prompt tokens: `{}`\n",
            report.no_memd.prompt_tokens
        ));
        markdown.push_str(&format!(
            "- with-memd prompt tokens: `{}`\n",
            report.with_memd.prompt_tokens
        ));
        markdown.push_str(&format!("- token delta: `{}`\n", report.token_delta));
        markdown.push_str(&format!("- reread delta: `{}`\n", report.reread_delta));
        markdown.push_str(&format!(
            "- reconstruction delta: `{}`\n",
            report.reconstruction_delta
        ));
        markdown.push_str(&format!(
            "- with memd better: `{}`\n",
            report.with_memd_better
        ));
    }
    if let Some(report) = continuity_journey_report {
        markdown.push_str("\n## Continuity Gate\n");
        markdown.push_str(&format!("- Journey: `{}`\n", report.journey_id));
        markdown.push_str(&format!(
            "- Gate: `{}` (score `{}`)\n",
            report.gate_decision.gate, report.gate_decision.resolved_score
        ));
        markdown.push_str(&format!(
            "- Baseline modes: `{}`\n",
            report.baseline_modes.join("`, `")
        ));
        if !report.gate_decision.reasons.is_empty() {
            markdown.push_str("- Reasons:\n");
            for reason in &report.gate_decision.reasons {
                markdown.push_str(&format!("  - {}\n", reason));
            }
        }
    }
    markdown.push('\n');
    markdown
}

pub(crate) async fn run_feature_benchmark_command(
    args: &BenchmarkArgs,
    base_url: &str,
) -> anyhow::Result<FeatureBenchmarkReport> {
    let started_at = Utc::now();
    let runtime = read_bundle_runtime_config(&args.output)?;
    let benchmark_registry = load_benchmark_registry_for_output(&args.output)?;
    let status = tokio::time::timeout(
        Duration::from_secs(2),
        read_bundle_status(&args.output, base_url),
    )
    .await
    .ok()
    .and_then(Result::ok);
    let command_names = top_level_command_names();
    let command_catalog = build_command_catalog(&args.output);
    let skill_catalog = build_skill_catalog(&args.output.join("skills"))?;
    let pack_index = crate::harness::index::build_harness_pack_index(
        &args.output,
        runtime
            .as_ref()
            .and_then(|config| config.project.as_deref()),
        runtime
            .as_ref()
            .and_then(|config| config.namespace.as_deref()),
    );
    let compiled_index = render_compiled_memory_index(&args.output).ok();
    let memory_quality = build_compiled_memory_quality_report(&args.output).ok();
    let eval = read_latest_bundle_eval(&args.output).ok().flatten();
    let scenario = read_latest_scenario_report(&args.output).ok().flatten();
    let maintain = read_latest_maintain_report(&args.output).ok().flatten();
    let experiment = read_latest_experiment_report(&args.output).ok().flatten();
    let evolution_proposal = read_latest_evolution_proposal(&args.output).ok().flatten();
    let evolution_branch = read_latest_evolution_branch_manifest(&args.output)
        .ok()
        .flatten();
    let event_log = read_bundle_event_log(&args.output).unwrap_or_default();

    let setup_ready = status
        .as_ref()
        .and_then(|value| value.get("setup_ready"))
        .and_then(|value| value.as_bool())
        .unwrap_or(runtime.is_some());
    let memory_pages = compiled_index
        .as_ref()
        .map(|index| index.pages.len())
        .unwrap_or(0);
    let launcher_count = fs::read_dir(args.output.join("agents"))
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|value| value.to_str()) == Some("sh"))
        .count();
    let has_commands_file = args.output.join("COMMANDS.md").exists();
    let has_memory_file = args.output.join("mem.md").exists();
    let has_wakeup_file = args.output.join("wake.md").exists();
    let has_events_file = args.output.join("events.md").exists();
    let has_compiled_events = compiled_event_dir(&args.output).join("latest.md").exists();

    let quality_score = memory_quality
        .as_ref()
        .map(|report| report.score)
        .unwrap_or(50);
    let retrieval_score = memory_quality
        .as_ref()
        .map(|report| benchmark_dim_score(report, "retrieval", 50))
        .unwrap_or(50);
    let freshness_score = memory_quality
        .as_ref()
        .map(|report| benchmark_dim_score(report, "freshness", 50))
        .unwrap_or(50);
    let provenance_score = memory_quality
        .as_ref()
        .map(|report| benchmark_dim_score(report, "provenance", 50))
        .unwrap_or(50);
    let token_efficiency_score = memory_quality
        .as_ref()
        .map(|report| benchmark_dim_score(report, "token_efficiency", 50))
        .unwrap_or(50);

    let mut areas = Vec::new();

    areas.push(benchmark_area_from_commands(
        "core_memory",
        "Core Memory",
        &command_names,
        &[
            "memory",
            "store",
            "candidate",
            "promote",
            "expire",
            "verify",
            "repair",
            "search",
            "lookup",
            "working",
            "source",
            "inbox",
            "explain",
            "recall",
            "timeline",
            "events",
        ],
        vec![
            format!("memory_quality={quality_score}"),
            format!("freshness={freshness_score} provenance={provenance_score}"),
            format!(
                "memory_file={} event_log={}",
                has_memory_file,
                !event_log.is_empty()
            ),
        ],
        Vec::new(),
        ((quality_score as u16 + freshness_score as u16 + provenance_score as u16) / 3) as u8,
    ));

    areas.push(benchmark_area_from_commands(
        "retrieval_context",
        "Retrieval And Context",
        &command_names,
        &[
            "lookup", "context", "recall", "search", "memory", "resume", "explain", "working",
        ],
        vec![
            format!("retrieval_score={retrieval_score} token_efficiency={token_efficiency_score}"),
            format!(
                "route={} intent={}",
                runtime
                    .as_ref()
                    .and_then(|config| config.route.as_deref())
                    .unwrap_or("none"),
                runtime
                    .as_ref()
                    .and_then(|config| config.intent.as_deref())
                    .unwrap_or("none")
            ),
            format!(
                "eval_snapshot={} scenario_snapshot={}",
                eval.is_some(),
                scenario.is_some()
            ),
        ],
        Vec::new(),
        ((retrieval_score as u16 + token_efficiency_score as u16) / 2) as u8,
    ));

    let visible_bonus = if has_memory_file && has_wakeup_file && memory_pages > 0 {
        100
    } else if has_memory_file || memory_pages > 0 {
        75
    } else {
        45
    };
    areas.push(benchmark_area_from_commands(
        "visible_memory",
        "Visible Memory And Inspection",
        &command_names,
        &[
            "memory",
            "explain",
            "entity",
            "entity-search",
            "timeline",
            "source",
            "workspaces",
            "ui",
        ],
        vec![
            format!(
                "memory_pages={} wakeup={} memory={} commands_md={}",
                memory_pages, has_wakeup_file, has_memory_file, has_commands_file
            ),
            format!("compiled_events={has_compiled_events}"),
        ],
        Vec::new(),
        visible_bonus,
    ));

    let session_bonus = if setup_ready {
        (100u16
            + (launcher_count.min(6) as u16 * 100 / 6)
            + if runtime
                .as_ref()
                .and_then(|config| config.tab_id.as_ref())
                .is_some()
            {
                100
            } else {
                60
            })
            / 3
    } else {
        30
    } as u8;
    areas.push(benchmark_area_from_commands(
        "bundle_session",
        "Bundle And Session Workflow",
        &command_names,
        &[
            "setup",
            "init",
            "status",
            "doctor",
            "config",
            "agent",
            "attach",
            "resume",
            "refresh",
            "wake",
            "watch",
            "handoff",
            "checkpoint",
            "session",
            "bundle",
        ],
        vec![
            format!("setup_ready={setup_ready}"),
            format!("launcher_scripts={launcher_count}"),
            format!(
                "session={} tab={}",
                runtime
                    .as_ref()
                    .and_then(|config| config.session.as_deref())
                    .unwrap_or("none"),
                runtime
                    .as_ref()
                    .and_then(|config| config.tab_id.as_deref())
                    .unwrap_or("none")
            ),
        ],
        Vec::new(),
        session_bonus,
    ));

    let capture_bonus = if has_events_file && !event_log.is_empty() {
        100
    } else if has_events_file {
        75
    } else {
        45
    };
    areas.push(benchmark_area_from_commands(
        "capture_compaction_events",
        "Capture, Compaction, And Events",
        &command_names,
        &[
            "hook",
            "compact",
            "checkpoint",
            "consolidate",
            "maintain",
            "events",
            "watch",
            "remember",
        ],
        vec![
            format!("event_count={}", event_log.len()),
            format!(
                "events_file={} compiled_events={}",
                has_events_file, has_compiled_events
            ),
            format!("maintain_snapshot={}", maintain.is_some()),
        ],
        Vec::new(),
        capture_bonus,
    ));

    let coordination_bonus = status
        .as_ref()
        .and_then(|value| value.get("cowork_surface"))
        .map(|_| 100)
        .or_else(|| {
            status
                .as_ref()
                .and_then(|value| value.get("live_session"))
                .map(|_| 80)
        })
        .unwrap_or(60);
    areas.push(benchmark_area_from_commands(
        "coordination_hive",
        "Multi-Agent Coordination And Hive",
        &command_names,
        &[
            "hive",
            "hive-project",
            "hive-join",
            "awareness",
            "heartbeat",
            "claims",
            "messages",
            "tasks",
            "coordination",
            "session",
        ],
        vec![
            format!(
                "live_session_overlay={}",
                status
                    .as_ref()
                    .and_then(|value| value.get("live_session"))
                    .is_some()
            ),
            format!(
                "cowork_surface={}",
                status
                    .as_ref()
                    .and_then(|value| value.get("cowork_surface"))
                    .is_some()
            ),
            format!(
                "scenario_coworking={}",
                scenario
                    .as_ref()
                    .map(|value| value.scenario == "coworking")
                    .unwrap_or(false)
            ),
        ],
        vec!["keep lane/worktree arbitration hot to prevent shared-branch drift".to_string()],
        coordination_bonus,
    ));

    let obsidian_bonus = if command_names.contains("obsidian") {
        85
    } else {
        20
    };
    areas.push(benchmark_area_from_commands(
        "obsidian",
        "Obsidian Integration",
        &command_names,
        &["obsidian", "memory", "search", "explain", "handoff"],
        vec![
            "obsidian command surface present".to_string(),
            format!("vault_artifacts_active={}", false),
        ],
        vec![
            "run obsidian scan/import/compile in a real vault to benchmark roundtrip quality"
                .to_string(),
        ],
        obsidian_bonus,
    ));

    let semantic_bonus = if command_names.contains("rag") && command_names.contains("multimodal") {
        if memory_pages > 0 { 90 } else { 75 }
    } else {
        30
    };
    areas.push(benchmark_area_from_commands(
        "semantic_multimodal",
        "Semantic And Multimodal Backend",
        &command_names,
        &["rag", "multimodal", "ingest", "memory", "context", "recall"],
        vec![
            format!(
                "semantic_lane_present={}",
                compiled_index
                    .as_ref()
                    .map(|index| {
                        index.entries.iter().any(|entry| {
                            entry.kind == "lane" && entry.lane == MemoryObjectLane::Semantic.slug()
                        })
                    })
                    .unwrap_or(false)
            ),
            format!("backend_env={}", args.output.join("backend.env").exists()),
        ],
        Vec::new(),
        semantic_bonus,
    ));

    let skill_count = skill_catalog.builtins.len() + skill_catalog.custom.len();
    let evolution_bonus = match (&experiment, &evolution_proposal, &evolution_branch) {
        (Some(_), Some(_), Some(_)) => 100,
        (Some(_), _, _) => 80,
        _ => 60,
    };
    areas.push(benchmark_area_from_commands(
        "policy_skills_evolution",
        "Policy, Skills, And Self-Evolution",
        &command_names,
        &[
            "policy",
            "skill-policy",
            "skills",
            "loops",
            "telemetry",
            "autoresearch",
            "experiment",
            "composite",
            "improve",
            "gap",
        ],
        vec![
            format!("skills={skill_count}"),
            format!(
                "experiment={} proposal={} branch={}",
                experiment.is_some(),
                evolution_proposal.is_some(),
                evolution_branch.is_some()
            ),
        ],
        Vec::new(),
        evolution_bonus,
    ));

    let diagnostics_bonus = if has_commands_file { 100 } else { 70 };
    areas.push(benchmark_area_from_commands(
        "diagnostics_admin",
        "Diagnostics And Admin Surfaces",
        &command_names,
        &[
            "healthz",
            "status",
            "doctor",
            "config",
            "commands",
            "packs",
            "skills",
            "inspiration",
            "maintenance-report",
            "maintain",
            "ui",
        ],
        vec![
            format!("commands_md={has_commands_file}"),
            format!("pack_count={}", pack_index.pack_count),
            format!("catalog_count={}", command_catalog.command_count),
        ],
        Vec::new(),
        diagnostics_bonus,
    ));

    let score = if areas.is_empty() {
        0
    } else {
        (areas.iter().map(|area| area.score as u16).sum::<u16>() / areas.len() as u16) as u8
    };

    let mut evidence = vec![
        format!("setup_ready={setup_ready}"),
        format!("commands={}", command_catalog.command_count),
        format!("skills={skill_count} packs={}", pack_index.pack_count),
        format!("memory_pages={memory_pages} events={}", event_log.len()),
    ];
    if let Some(report) = memory_quality.as_ref() {
        evidence.push(format!(
            "memory_quality score={}/{} latency_ms={}",
            report.score, report.max_score, report.latency_ms
        ));
        if let Some(probe) = report
            .probes
            .iter()
            .find(|probe| probe.query == "session_continuity+candidate")
            .or_else(|| {
                report
                    .probes
                    .iter()
                    .find(|probe| probe.query == "session_continuity")
            })
        {
            let query_label = if probe.query == "session_continuity" {
                "session_continuity+candidate"
            } else {
                probe.query.as_str()
            };
            evidence.push(format!(
                "typed_retrieval_probe={} best_score={} best_path={} best_section={}",
                query_label,
                probe.best_score,
                probe.best_path.as_deref().unwrap_or("none"),
                probe.best_section.as_deref().unwrap_or("none")
            ));
        }
        for typed_query in ["procedural", "canonical"] {
            if let Some(probe) = report
                .probes
                .iter()
                .find(|probe| probe.query == typed_query)
            {
                evidence.push(format!(
                    "typed_retrieval_probe={} best_score={} best_path={} best_section={}",
                    probe.query,
                    probe.best_score,
                    probe.best_path.as_deref().unwrap_or("none"),
                    probe.best_section.as_deref().unwrap_or("none")
                ));
            }
        }
    }
    if let Some((repo_root, registry)) = benchmark_registry.as_ref() {
        evidence.push(format!(
            "benchmark_registry root={} version={} features={} journeys={} loops={} scorecards={} baseline_modes={} runtime_policies={}",
            repo_root.display(),
            registry.version,
            registry.features.len(),
            registry.journeys.len(),
            registry.loops.len(),
            registry.scorecards.len(),
            registry.baseline_modes.len(),
            registry.runtime_policies.len()
        ));
    } else {
        evidence.push("benchmark_registry root=unavailable".to_string());
    }

    let recommendations = areas
        .iter()
        .filter(|area| area.status != "pass")
        .flat_map(|area| area.recommendations.clone())
        .collect::<Vec<_>>();

    Ok(FeatureBenchmarkReport {
        bundle_root: args.output.display().to_string(),
        project: runtime.as_ref().and_then(|config| config.project.clone()),
        namespace: runtime.as_ref().and_then(|config| config.namespace.clone()),
        agent: runtime.as_ref().and_then(|config| config.agent.clone()),
        session: runtime.as_ref().and_then(|config| config.session.clone()),
        workspace: runtime.as_ref().and_then(|config| config.workspace.clone()),
        visibility: runtime
            .as_ref()
            .and_then(|config| config.visibility.clone()),
        score,
        max_score: 100,
        command_count: command_catalog.command_count,
        skill_count,
        pack_count: pack_index.pack_count,
        memory_pages,
        event_count: event_log.len(),
        areas,
        evidence,
        recommendations,
        generated_at: started_at,
        completed_at: Utc::now(),
    })
}

pub(crate) fn write_improvement_artifacts(
    output: &Path,
    response: &ImprovementReport,
) -> anyhow::Result<()> {
    let improvement_dir = improvement_reports_dir(output);
    fs::create_dir_all(&improvement_dir)
        .with_context(|| format!("create {}", improvement_dir.display()))?;

    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let baseline_json = improvement_dir.join("latest.json");
    let baseline_md = improvement_dir.join("latest.md");
    let timestamp_json = improvement_dir.join(format!("{timestamp}.json"));
    let timestamp_md = improvement_dir.join(format!("{timestamp}.md"));
    let json = serde_json::to_string_pretty(response)? + "\n";
    let markdown = render_improvement_markdown(response);

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

pub(crate) fn read_latest_public_benchmark_reports(
    output: &Path,
) -> anyhow::Result<Vec<PublicBenchmarkRunReport>> {
    let public_root = output.join("benchmarks").join("public");
    if !public_root.exists() {
        return Ok(Vec::new());
    }

    let mut reports = Vec::new();
    for benchmark_id in supported_public_benchmark_ids() {
        let results_path = public_root
            .join(benchmark_id)
            .join("latest")
            .join("results.json");
        if !results_path.exists() {
            continue;
        }
        let raw = fs::read_to_string(&results_path)
            .with_context(|| format!("read {}", results_path.display()))?;
        let report = serde_json::from_str::<PublicBenchmarkRunReport>(&raw)
            .with_context(|| format!("parse {}", results_path.display()))?;
        reports.push(report);
    }
    reports.sort_by(|left, right| left.manifest.benchmark_id.cmp(&right.manifest.benchmark_id));
    Ok(reports)
}

pub(crate) fn render_public_benchmark_markdown(reports: &[PublicBenchmarkRunReport]) -> String {
    let supported_targets = supported_public_benchmark_ids();
    let implemented_targets = implemented_public_benchmark_ids();
    let latest = reports
        .iter()
        .max_by_key(|report| report.manifest.run_timestamp);
    let mut lines = vec![
        "# memd public benchmark suite".to_string(),
        String::new(),
        format!("- latest_runs: {}", reports.len()),
        format!("- supported_targets: {}", supported_targets.join(", ")),
        format!("- implemented_adapters: {}", implemented_targets.join(", ")),
    ];
    if let Some(report) = latest {
        lines.push(format!(
            "- newest_run: {} mode={} at {}",
            report.manifest.benchmark_id,
            report.manifest.mode,
            report.manifest.run_timestamp.to_rfc3339()
        ));
    }
    lines.push(String::new());
    lines.push("## Target Inventory".to_string());
    for dataset in supported_targets {
        lines.push(format!(
            "- {}: {}",
            dataset,
            public_benchmark_target_status(dataset)
        ));
    }
    lines.push(format!(
        "- implemented adapters: {}",
        implemented_targets.join(", ")
    ));
    lines.push(String::new());
    lines.push("## Latest Runs".to_string());
    lines.push(
        "| Benchmark | Version | Mode | Primary Metric | Value | Items | Dataset | Checksum | Artifacts |"
            .to_string(),
    );
    lines.push("| --- | --- | --- | --- | --- | --- | --- | --- | --- |".to_string());
    for report in reports {
        let (metric_label, metric_value) = public_benchmark_primary_metric(report);
        lines.push(format!(
            "| {} | {} | {} | {} | {:.3} | {} | {} | {} | `.memd/benchmarks/public/{}/latest/` |",
            report.manifest.dataset_name,
            report.manifest.benchmark_version,
            report.manifest.mode,
            metric_label,
            metric_value,
            report.item_count,
            report.manifest.dataset_local_path,
            report.manifest.dataset_checksum,
            report.manifest.benchmark_id,
        ));
    }
    lines.push(String::new());
    lines.push("## Artifacts".to_string());
    for report in reports {
        lines.push(format!(
            "- {}: `.memd/benchmarks/public/{}/latest/manifest.json`, `.memd/benchmarks/public/{}/latest/results.json`, `.memd/benchmarks/public/{}/latest/results.jsonl`, `.memd/benchmarks/public/{}/latest/report.md`",
            report.manifest.benchmark_id,
            report.manifest.benchmark_id,
            report.manifest.benchmark_id,
            report.manifest.benchmark_id,
            report.manifest.benchmark_id,
        ));
    }
    if let Some(report) = latest {
        lines.push(String::new());
        lines.push(format!(
            "## Latest Run Detail: {}",
            report.manifest.dataset_name
        ));
        lines.push("| Item | Question | Claim | Hit | Answer | Latency ms |".to_string());
        lines.push("| --- | --- | --- | --- | --- | --- |".to_string());
        for item in &report.items {
            lines.push(format!(
                "| {} | {} | {} | {} | {} | {} |",
                item.item_id,
                item.question.as_deref().unwrap_or(&item.question_id),
                item.claim_class,
                item.hit,
                item.answer.as_deref().unwrap_or("-"),
                item.latency_ms,
            ));
        }
    }
    lines.join("\n")
}

pub(crate) fn render_public_leaderboard(report: &PublicBenchmarkLeaderboardReport) -> String {
    let mut lines = vec![
        "# memd public leaderboard".to_string(),
        String::new(),
        format!("- generated_at: {}", report.generated_at.to_rfc3339()),
        format!("- rows: {}", report.rows.len()),
        String::new(),
        "## Claim Governance".to_string(),
    ];
    for note in &report.governance_notes {
        lines.push(format!("- {note}"));
    }
    lines.push(String::new());
    lines.push(
        "| Benchmark | Version | Run mode | Item claim classes | Coverage | Parity claim | Primary Metric | Value | Items | Notes |"
            .to_string(),
    );
    lines.push("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |".to_string());
    for row in &report.rows {
        lines.push(format!(
            "| {} | {} | {} | {} | {} | {} | {} | {:.3} | {} | {} |",
            row.benchmark_name,
            row.benchmark_version,
            row.run_mode,
            row.item_claim_classes.join(", "),
            row.coverage_status,
            row.parity_status,
            row.notes
                .iter()
                .find_map(|note| note.strip_prefix("primary_metric="))
                .unwrap_or("retrieval-local proxy"),
            row.accuracy,
            row.item_count,
            row.notes.join("; "),
        ));
    }
    lines.join("\n")
}

pub(crate) fn render_public_benchmark_summary(report: &PublicBenchmarkRunReport) -> String {
    let (metric_label, metric_value) = public_benchmark_primary_metric(report);
    format!(
        "public benchmark {} mode={} items={} primary_metric={} value={:.3} artifacts=.memd/benchmarks/public/{}/latest",
        report.manifest.benchmark_id,
        report.manifest.mode,
        report.item_count,
        metric_label,
        metric_value,
        report.manifest.benchmark_id,
    )
}

pub(crate) fn write_public_benchmark_docs(
    repo_root: &Path,
    output: &Path,
    report: &PublicBenchmarkRunReport,
) -> anyhow::Result<()> {
    let mut reports = read_latest_public_benchmark_reports(output)?;
    if !reports
        .iter()
        .any(|existing| existing.manifest.benchmark_id == report.manifest.benchmark_id)
    {
        reports.push(report.clone());
        reports.sort_by(|left, right| left.manifest.benchmark_id.cmp(&right.manifest.benchmark_id));
    }
    let path = public_benchmark_docs_path(repo_root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&path, render_public_benchmark_markdown(&reports))
        .with_context(|| format!("write {}", path.display()))?;

    let leaderboard_path = public_benchmark_leaderboard_docs_path(repo_root);
    if let Some(parent) = leaderboard_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let leaderboard = build_public_benchmark_leaderboard_report(&reports);
    fs::write(&leaderboard_path, render_public_leaderboard(&leaderboard))
        .with_context(|| format!("write {}", leaderboard_path.display()))?;
    Ok(())
}

pub(crate) async fn run_public_benchmark_command(
    args: &PublicBenchmarkArgs,
) -> anyhow::Result<PublicBenchmarkRunReport> {
    validate_public_benchmark_args(args)?;

    // --all: run all implemented benchmarks sequentially, return the last report
    if args.all {
        let mut last_report = None;
        for dataset_id in implemented_public_benchmark_ids() {
            let mut sub_args = args.clone();
            sub_args.dataset = dataset_id.to_string();
            sub_args.all = false;
            match Box::pin(run_public_benchmark_command(&sub_args)).await {
                Ok(report) => {
                    last_report = Some(report);
                }
                Err(err) => {
                    eprintln!("warning: {dataset_id} benchmark failed: {err}");
                }
            }
        }
        return last_report.ok_or_else(|| anyhow!("no benchmarks completed"));
    }

    let supported_targets = supported_public_benchmark_ids();
    anyhow::ensure!(
        supported_targets.contains(&args.dataset.as_str()),
        "unknown public benchmark dataset `{}`; supported targets: {}",
        args.dataset,
        supported_targets.join(", ")
    );
    let started_at = Utc::now();
    let duration_started = Instant::now();
    let resolved_dataset = resolve_public_benchmark_dataset(args).await?;
    let dataset = load_public_benchmark_dataset(&args.dataset, &resolved_dataset.path)?;
    anyhow::ensure!(
        dataset.benchmark_id == args.dataset,
        "public benchmark fixture `{}` does not match requested dataset `{}`",
        dataset.benchmark_id,
        args.dataset
    );

    let mode = args.mode.as_deref().unwrap_or("raw");
    let retrieval_config = build_public_benchmark_retrieval_config(args)?;
    let top_k = args.top_k.unwrap_or(5).max(1);
    let item_count = args
        .sample
        .or(args.limit)
        .unwrap_or(dataset.items.len())
        .max(1)
        .min(dataset.items.len());
    let selected_items = dataset
        .items
        .iter()
        .take(item_count)
        .cloned()
        .collect::<Vec<_>>();
    let reranker_id = if mode == "hybrid" {
        Some(args.reranker.as_deref().unwrap_or("declared-reranker"))
    } else {
        None
    };
    let selected_dataset = PublicBenchmarkDatasetFixture {
        items: selected_items,
        ..dataset.clone()
    };
    // Dry-run: estimate cost without running
    if args.dry_run && args.full_eval {
        let est_calls =
            item_count * if args.dataset == "longmemeval" { 2 } else { 1 };
        let est_tokens = est_calls as f64 * 2200.0;
        let est_cost_4o_mini = est_tokens * 0.00000015;
        let est_cost_4o = est_tokens * 0.0000025;
        eprintln!(
            "Dry run: {item_count} items, ~{est_calls} API calls"
        );
        eprintln!(
            "Estimated cost: ${:.2} (gpt-4o-mini) / ${:.2} (gpt-4o)",
            est_cost_4o_mini, est_cost_4o
        );
        let mut dry_manifest = build_public_benchmark_manifest(
            args,
            &dataset,
            &resolved_dataset,
            mode,
            top_k,
            item_count,
            started_at,
            duration_started.elapsed().as_millis(),
            reranker_id,
            &retrieval_config,
            None,
            None,
        )?;
        if let Some(obj) = dry_manifest.runtime_settings.as_object_mut() {
            obj.insert("full_eval".to_string(), json!(true));
            obj.insert(
                "generator_model".to_string(),
                json!(args.generator_model.as_deref().unwrap_or("gpt-4o-mini")),
            );
        }
        return Ok(PublicBenchmarkRunReport {
            manifest: dry_manifest,
            metrics: BTreeMap::new(),
            item_count: 0,
            failures: Vec::new(),
            items: Vec::new(),
        });
    }

    let evaluation = if args.full_eval {
        let generator_config = resolve_generator_config(args)?;
        match args.dataset.as_str() {
            "longmemeval" => build_longmemeval_full_eval_report(
                &selected_dataset,
                top_k,
                mode,
                &retrieval_config,
                &generator_config,
            )
            .await?,
            "locomo" => build_locomo_full_eval_report(
                &selected_dataset,
                top_k,
                mode,
                &retrieval_config,
                &generator_config,
            )
            .await?,
            "membench" => build_membench_full_eval_report(
                &selected_dataset,
                top_k,
                mode,
                &retrieval_config,
                &generator_config,
            )
            .await?,
            other => anyhow::bail!("--full-eval not yet supported for {other}"),
        }
    } else if args.community_standard {
        let grader_model = args.grader_model.as_deref().unwrap_or("gpt-4o");
        build_longmemeval_community_standard_run_report(
            &selected_dataset,
            args.hypotheses_file
                .as_deref()
                .context("community-standard longmemeval requires --hypotheses-file")?,
            grader_model,
            mode,
        )
        .await?
    } else {
        build_public_benchmark_item_results(
            &selected_dataset,
            top_k,
            mode,
            reranker_id,
            &retrieval_config,
        )?
    };
    let token_usage = if mode == "hybrid" {
        Some(json!({
            "prompt_tokens": 0,
            "completion_tokens": 0,
            "reranker_tokens": 0,
        }))
    } else {
        None
    };
    let cost_estimate_usd = if mode == "hybrid" { Some(0.0) } else { None };
    let mut manifest = build_public_benchmark_manifest(
        args,
        &dataset,
        &resolved_dataset,
        mode,
        top_k,
        item_count,
        started_at,
        duration_started.elapsed().as_millis(),
        reranker_id,
        &retrieval_config,
        token_usage.clone(),
        cost_estimate_usd,
    )?;
    if args.full_eval {
        if let Some(obj) = manifest.runtime_settings.as_object_mut() {
            obj.insert("full_eval".to_string(), json!(true));
            obj.insert(
                "generator_model".to_string(),
                json!(args.generator_model.as_deref().unwrap_or("gpt-4o-mini")),
            );
        }
    }
    Ok(PublicBenchmarkRunReport {
        manifest,
        metrics: evaluation.metrics,
        item_count: evaluation.item_count,
        failures: evaluation.failures,
        items: evaluation.items,
    })
}
