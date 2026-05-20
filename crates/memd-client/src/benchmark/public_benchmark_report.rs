use super::*;

pub(crate) fn compute_latency_percentiles(latencies: &[u128]) -> (f64, f64) {
    if latencies.is_empty() {
        return (0.0, 0.0);
    }
    let mut sorted = latencies.to_vec();
    sorted.sort();
    let p50_index = (sorted.len() as f64 * 0.5) as usize;
    let p95_index = (sorted.len() as f64 * 0.95) as usize;
    (
        sorted[p50_index.min(sorted.len() - 1)] as f64,
        sorted[p95_index.min(sorted.len() - 1)] as f64,
    )
}

pub(crate) fn build_public_benchmark_item_results(
    dataset: &PublicBenchmarkDatasetFixture,
    top_k: usize,
    mode: &str,
    reranker_id: Option<&str>,
    retrieval_config: &PublicBenchmarkRetrievalConfig,
    include_turn_diagnostics: bool,
) -> anyhow::Result<PublicBenchmarkRunReport> {
    if dataset.benchmark_id == "longmemeval" {
        return build_longmemeval_run_report(
            dataset,
            top_k,
            mode,
            reranker_id,
            retrieval_config,
            include_turn_diagnostics,
        );
    }
    if dataset.benchmark_id == "locomo" {
        return build_context_retrieval_run_report(
            dataset,
            top_k,
            mode,
            reranker_id,
            retrieval_config,
            locomo_retrieval_docs,
            |item| public_benchmark_evidence_target_keys(item.metadata.get("evidence")),
        );
    }
    if dataset.benchmark_id == "membench" {
        return build_context_retrieval_run_report(
            dataset,
            top_k,
            mode,
            reranker_id,
            retrieval_config,
            membench_retrieval_docs,
            |item| {
                item.metadata
                    .get("target_step_id")
                    .and_then(JsonValue::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(public_benchmark_target_key)
                    .collect()
            },
        );
    }
    if dataset.benchmark_id == "convomem" {
        return build_context_retrieval_run_report(
            dataset,
            top_k,
            mode,
            reranker_id,
            retrieval_config,
            |item| {
                convomem_message_docs(
                    item.metadata
                        .get("conversations")
                        .unwrap_or(&JsonValue::Null),
                )
            },
            |item| {
                let ids = public_benchmark_string_vec(item.metadata.get("message_evidence_ids"));
                if ids.is_empty() {
                    convomem_message_evidence_ids(
                        item.metadata
                            .get("message_evidences")
                            .unwrap_or(&JsonValue::Null),
                        item.metadata
                            .get("conversations")
                            .unwrap_or(&JsonValue::Null),
                    )
                    .into_iter()
                    .collect()
                } else {
                    ids.into_iter().collect()
                }
            },
        );
    }
    anyhow::bail!(
        "no retrieval-only benchmark path for dataset `{}`; use --full-eval or a supported dataset (longmemeval, locomo, membench, convomem)",
        dataset.benchmark_id
    );
}

pub(crate) fn build_public_benchmark_manifest(
    args: &PublicBenchmarkArgs,
    dataset: &PublicBenchmarkDatasetFixture,
    resolved_dataset: &ResolvedPublicBenchmarkDataset,
    mode: &str,
    top_k: usize,
    item_count: usize,
    started_at: DateTime<Utc>,
    duration_ms: u128,
    reranker_id: Option<&str>,
    retrieval_config: &PublicBenchmarkRetrievalConfig,
    token_usage: Option<JsonValue>,
    cost_estimate_usd: Option<f64>,
) -> anyhow::Result<PublicBenchmarkManifest> {
    let repo_root = infer_bundle_project_root(&args.out);
    Ok(PublicBenchmarkManifest {
        benchmark_id: dataset.benchmark_id.clone(),
        benchmark_version: dataset.version.clone(),
        dataset_name: dataset.benchmark_name.clone(),
        dataset_source_url: resolved_dataset.source_url.clone(),
        dataset_local_path: resolved_dataset.path.display().to_string(),
        dataset_checksum: resolved_dataset.checksum.clone(),
        dataset_split: if resolved_dataset.split == "manual" {
            dataset.split.clone()
        } else {
            resolved_dataset.split.clone()
        },
        git_sha: repo_root
            .as_ref()
            .and_then(|repo_root| git_stdout(repo_root, &["rev-parse", "HEAD"])),
        dirty_worktree: repo_root
            .as_ref()
            .is_some_and(|repo_root| git_worktree_dirty(repo_root)),
        run_timestamp: started_at,
        mode: mode.to_string(),
        top_k,
        reranker_id: reranker_id.map(str::to_string),
        reranker_provider: if mode == "hybrid" {
            Some("declared".to_string())
        } else {
            None
        },
        limit: Some(item_count),
        runtime_settings: json!({
            "dataset_fixture": resolved_dataset.path.display().to_string(),
            "dataset_items": dataset.items.len(),
            "mode": mode,
            "turn_diagnostics": args.turn_diagnostics,
            "community_standard": args.community_standard,
            "hypotheses_file": args.hypotheses_file.as_ref().map(|path| path.display().to_string()),
            "grader_model": args.grader_model,
            "retrieval_backend": match retrieval_config.longmemeval_backend {
                LongMemEvalRetrievalBackend::Lexical => "lexical",
                LongMemEvalRetrievalBackend::Sidecar => "sidecar",
                LongMemEvalRetrievalBackend::Rrf => "rrf",
                LongMemEvalRetrievalBackend::Memd => "memd",
            },
            "sidecar_base_url": retrieval_config.sidecar_base_url,
            "memd_base_url": retrieval_config.memd_base_url,
            "top_k": top_k,
            "limit": item_count,
            "dataset_verification": resolved_dataset.verification_status,
        }),
        hardware_summary: format!("{}-{}-cpu", std::env::consts::OS, std::env::consts::ARCH),
        duration_ms,
        token_usage,
        cost_estimate_usd,
    })
}

pub(crate) fn build_public_benchmark_leaderboard_report(
    repo_root: &Path,
    output: &Path,
    reports: &[PublicBenchmarkRunReport],
) -> PublicBenchmarkLeaderboardReport {
    let has_real_dataset_runs = reports
        .iter()
        .any(|report| report.manifest.dataset_source_url.starts_with("http"));
    let history = load_public_benchmark_history(output);
    let published_baselines =
        load_published_baselines(&default_baselines_path(output)).unwrap_or_default();
    let mempalace_replays =
        load_mempalace_replays(&default_mempalace_replays_path(output)).unwrap_or_default();
    PublicBenchmarkLeaderboardReport {
        generated_at: reports
            .iter()
            .map(|report| report.manifest.run_timestamp)
            .max()
            .unwrap_or_else(Utc::now),
        governance_notes: vec![
            "claim class, verification, regression budget, commit, rerun command, and MemPalace baseline are first-class row fields".to_string(),
            "run mode is benchmark execution mode; item mode is intrinsic/accelerated when dual is active; claim class stays dataset-native".to_string(),
            format!(
                "implemented mini adapters: {}",
                implemented_public_benchmark_ids().join(", ")
            ),
            format!(
                "declared parity targets: {}",
                supported_public_benchmark_ids().join(", ")
            ),
            format!(
                "default regression budget: {:.3}",
                PUBLIC_BENCHMARK_REGRESSION_BUDGET
            ),
            if has_real_dataset_runs {
                "real upstream dataset runs use benchmark-shaped metrics with memd's local retrieval backend; MemPalace status is surfaced per row".to_string()
            } else {
                "no real upstream datasets have been replayed yet".to_string()
            },
        ],
        rows: reports
            .iter()
            .map(|report| {
                let (primary_metric_label, primary_metric_value) =
                    public_benchmark_primary_metric(report);
                let (intrinsic_score, accelerated_score) =
                    public_benchmark_dual_scores(report);
                let score_delta = intrinsic_score
                    .zip(accelerated_score)
                    .map(|(intrinsic, accelerated)| accelerated - intrinsic);
                let mempalace_baseline = resolve_mempalace_baseline(
                    &published_baselines,
                    &mempalace_replays,
                    &report.manifest.benchmark_id,
                );
                let verification_status = public_benchmark_verification_status(report);
                let prior_verified = latest_verified_history_entry(
                    &history,
                    &report.manifest.benchmark_id,
                    report.manifest.run_timestamp,
                );
                let regression_delta = prior_verified
                    .as_ref()
                    .map(|entry| primary_metric_value - entry.primary_value);
                let commit_sha = resolve_public_benchmark_commit_sha(report);
                let commit_url = commit_sha
                    .as_deref()
                    .and_then(|sha| public_benchmark_commit_url(repo_root, sha));
                let mut item_modes = report
                    .items
                    .iter()
                    .filter_map(|item| item.mode.clone())
                    .collect::<Vec<_>>();
                item_modes.sort();
                item_modes.dedup();
                let mut item_claim_classes = report
                    .items
                    .iter()
                    .map(|item| item.claim_class.clone())
                    .collect::<Vec<_>>();
                item_claim_classes.sort();
                item_claim_classes.dedup();
                PublicBenchmarkLeaderboardRow {
                    benchmark_id: report.manifest.benchmark_id.clone(),
                    benchmark_name: report.manifest.dataset_name.clone(),
                    benchmark_version: report.manifest.benchmark_version.clone(),
                    run_mode: report.manifest.mode.clone(),
                    item_modes,
                    item_claim_classes,
                    coverage_status: if report.manifest.dataset_source_url.starts_with("http") {
                        "real-dataset".to_string()
                    } else {
                        "fixture-backed".to_string()
                    },
                    claim_class: public_benchmark_claim_class(report, mempalace_baseline.as_ref()),
                    parity_status: public_benchmark_parity_status(
                        report,
                        mempalace_baseline.as_ref(),
                    ),
                    verification_status,
                    primary_metric_label: primary_metric_label.to_string(),
                    accuracy: primary_metric_value,
                    intrinsic_score,
                    accelerated_score,
                    score_delta,
                    mempalace_score: mempalace_baseline.as_ref().and_then(|entry| entry.accuracy),
                    mempalace_status: mempalace_baseline
                        .as_ref()
                        .map(|entry| entry.status.clone())
                        .unwrap_or_else(|| "replay-pending".to_string()),
                    regression_delta,
                    regression_budget: Some(PUBLIC_BENCHMARK_REGRESSION_BUDGET),
                    commit_sha,
                    commit_url,
                    rerun_command: Some(public_benchmark_rerun_command(report, output)),
                    artifact_path: Some(public_benchmark_artifact_path(report)),
                    item_count: report.item_count,
                    notes: {
                        let mut notes = vec![
                            format!("dataset={}", report.manifest.dataset_local_path),
                            format!("checksum={}", report.manifest.dataset_checksum),
                            format!("source={}", report.manifest.dataset_source_url),
                            format!("artifacts={}", public_benchmark_artifact_path(report)),
                        ];
                        if let Some(entry) = mempalace_baseline.as_ref() {
                            notes.push(format!("mempalace_source={}", entry.source));
                            if let Some(note) = entry.note.as_deref() {
                                notes.push(format!("mempalace_note={note}"));
                            }
                            if let Some(command) = entry.command.as_deref() {
                                notes.push(format!("mempalace_command={command}"));
                            }
                            if let Some(path) = entry.artifact_path.as_deref() {
                                notes.push(format!("mempalace_artifacts={path}"));
                            }
                        } else {
                            notes.push("mempalace_source=missing".to_string());
                        }
                        if let Some(entry) = prior_verified.as_ref() {
                            if let Some(sha) = entry.git_sha.as_deref() {
                                notes.push(format!("last_verified_commit={sha}"));
                            }
                            notes.push(format!("last_verified_value={:.3}", entry.primary_value));
                        }
                        if report.manifest.benchmark_version == "upstream" {
                            notes.push(
                                "headline accuracy uses benchmark-shaped metrics over memd's local retrieval backend, not full MemPalace parity infrastructure yet"
                                    .to_string(),
                            );
                        }
                        notes
                    },
                }
            })
            .collect(),
    }
}

#[derive(Debug, Clone)]
struct MempalaceBaselineEntry {
    accuracy: Option<f64>,
    source: String,
    note: Option<String>,
    status: String,
    command: Option<String>,
    artifact_path: Option<String>,
}

fn load_public_benchmark_history(output: &Path) -> Vec<PublicBenchmarkHistoryEntry> {
    let history_path = output
        .join("benchmarks")
        .join("history")
        .join("benchmark-runs.jsonl");
    let Ok(raw) = fs::read_to_string(&history_path) else {
        return Vec::new();
    };
    raw.lines()
        .filter_map(|line| serde_json::from_str::<PublicBenchmarkHistoryEntry>(line).ok())
        .collect()
}

fn latest_verified_history_entry(
    entries: &[PublicBenchmarkHistoryEntry],
    benchmark_id: &str,
    current_timestamp: DateTime<Utc>,
) -> Option<PublicBenchmarkHistoryEntry> {
    entries
        .iter()
        .filter(|entry| entry.benchmark_id == benchmark_id)
        .filter(|entry| entry.timestamp <= current_timestamp)
        .filter(|entry| {
            entry
                .verification_status
                .as_deref()
                .map(|status| status.starts_with("verified"))
                .unwrap_or(true)
        })
        .max_by_key(|entry| entry.timestamp)
        .cloned()
}

fn resolve_mempalace_baseline(
    baselines: &BTreeMap<String, BTreeMap<String, BaselineEntry>>,
    replays: &BTreeMap<String, MempalaceReplayEntry>,
    benchmark_id: &str,
) -> Option<MempalaceBaselineEntry> {
    if let Some(entry) = replays.get(benchmark_id) {
        return Some(MempalaceBaselineEntry {
            accuracy: entry
                .accuracy
                .map(|value| if value > 1.0 { value / 100.0 } else { value }),
            source: entry.source.clone(),
            note: entry.note.clone(),
            status: entry
                .status
                .clone()
                .unwrap_or_else(|| "replayed".to_string()),
            command: entry.command.clone(),
            artifact_path: entry.artifact_path.clone(),
        });
    }
    baselines
        .get(benchmark_id)
        .and_then(|systems| {
            systems
                .iter()
                .find(|(name, _)| name.to_ascii_lowercase().contains("mempal"))
        })
        .map(|(_, entry)| MempalaceBaselineEntry {
            accuracy: entry
                .accuracy
                .map(|value| if value > 1.0 { value / 100.0 } else { value }),
            source: entry.source.clone(),
            note: entry.note.clone(),
            status: entry
                .note
                .as_deref()
                .map(public_benchmark_mempalace_status_from_note)
                .unwrap_or_else(|| "published-baseline".to_string()),
            command: None,
            artifact_path: None,
        })
}

fn public_benchmark_mempalace_status_from_note(note: &str) -> String {
    let note = note.to_ascii_lowercase();
    if note.contains("pending") {
        "replay-pending".to_string()
    } else if note.contains("replayed") || note.contains("same-fixture replay complete") {
        "replayed".to_string()
    } else {
        "published-baseline".to_string()
    }
}

fn public_benchmark_claim_class(
    report: &PublicBenchmarkRunReport,
    mempalace_baseline: Option<&MempalaceBaselineEntry>,
) -> String {
    if !report.manifest.dataset_source_url.starts_with("http") {
        return "fixture-only".to_string();
    }
    if mempalace_baseline
        .map(|entry| entry.status == "replayed")
        .unwrap_or(false)
    {
        return "cross-replayed".to_string();
    }
    "dataset-native / memd-local".to_string()
}

fn public_benchmark_parity_status(
    report: &PublicBenchmarkRunReport,
    mempalace_baseline: Option<&MempalaceBaselineEntry>,
) -> String {
    if !report.manifest.dataset_source_url.starts_with("http") {
        return "fixture-backed".to_string();
    }
    if mempalace_baseline
        .map(|entry| entry.status == "replayed")
        .unwrap_or(false)
    {
        "cross-replayed".to_string()
    } else {
        "dataset-native / memd-local".to_string()
    }
}

fn public_benchmark_verification_status(report: &PublicBenchmarkRunReport) -> String {
    report
        .manifest
        .runtime_settings
        .get("dataset_verification")
        .and_then(JsonValue::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| {
            if report.manifest.dataset_source_url.starts_with("http") {
                "recorded-unpinned".to_string()
            } else {
                "fixture-only".to_string()
            }
        })
}

fn resolve_public_benchmark_commit_sha(report: &PublicBenchmarkRunReport) -> Option<String> {
    report.manifest.git_sha.clone().or_else(|| {
        std::process::Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

fn public_benchmark_commit_url(repo_root: &Path, sha: &str) -> Option<String> {
    let repo_root = repo_root.to_string_lossy().to_string();
    let remote = std::process::Command::new("git")
        .args(["-C", &repo_root, "config", "--get", "remote.origin.url"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())?;
    let base = if let Some(rest) = remote.strip_prefix("https://github.com/") {
        format!("https://github.com/{}", rest.trim_end_matches(".git"))
    } else if let Some(rest) = remote.strip_prefix("git@github.com:") {
        format!("https://github.com/{}", rest.trim_end_matches(".git"))
    } else {
        return None;
    };
    Some(format!("{base}/commit/{sha}"))
}

fn public_benchmark_rerun_command(report: &PublicBenchmarkRunReport, output: &Path) -> String {
    format!(
        "cargo run -p memd-client -- benchmark public {} --mode {} --top-k {} --write --record --out {} --dataset-root {}",
        report.manifest.benchmark_id,
        report.manifest.mode,
        report.manifest.top_k,
        output.display(),
        report.manifest.dataset_local_path,
    )
}

fn public_benchmark_artifact_path(report: &PublicBenchmarkRunReport) -> String {
    format!(
        ".memd/benchmarks/public/{}/latest/",
        report.manifest.benchmark_id
    )
}

pub(crate) fn public_benchmark_primary_metric(
    report: &PublicBenchmarkRunReport,
) -> (&'static str, f64) {
    let (label, candidates) = public_benchmark_primary_metric_candidates(report);
    (label, resolve_public_benchmark_metric(report, &candidates))
}

fn public_benchmark_primary_metric_candidates(
    report: &PublicBenchmarkRunReport,
) -> (&'static str, Vec<&'static str>) {
    let is_full_eval = report
        .manifest
        .runtime_settings
        .get("full_eval")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false);

    match (report.manifest.benchmark_id.as_str(), is_full_eval) {
        ("longmemeval", true) => ("accuracy (LLM-judge, industry standard)", vec!["accuracy"]),
        ("locomo", true) => ("F1 (token-level, industry standard)", vec!["f1"]),
        ("membench", true) => ("MC accuracy (industry standard)", vec!["mc_accuracy"]),
        ("longmemeval", false) => {
            let is_community = report
                .manifest
                .runtime_settings
                .get("community_standard")
                .and_then(JsonValue::as_bool)
                .unwrap_or(false);
            if is_community {
                (
                    "official_qa_accuracy (community standard)",
                    vec!["qa_accuracy", "accuracy"],
                )
            } else {
                (
                    "session_recall_any@5 (retrieval diagnostic)",
                    vec!["session_recall_any@5", "accuracy"],
                )
            }
        }
        ("locomo", false) => (
            "evidence_hit_rate@5 (retrieval diagnostic)",
            vec!["accuracy"],
        ),
        ("membench", false) => ("target_hit_rate@5 (retrieval diagnostic)", vec!["accuracy"]),
        _ => ("accuracy (retrieval diagnostic)", vec!["accuracy"]),
    }
}

fn resolve_public_benchmark_metric(report: &PublicBenchmarkRunReport, candidates: &[&str]) -> f64 {
    candidates
        .iter()
        .find_map(|key| report.metrics.get(*key).copied())
        .unwrap_or(0.0)
}

fn public_benchmark_dual_scores(report: &PublicBenchmarkRunReport) -> (Option<f64>, Option<f64>) {
    let is_dual = report
        .manifest
        .runtime_settings
        .get("dual")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false);
    if !is_dual {
        return (None, None);
    }
    let (_, candidates) = public_benchmark_primary_metric_candidates(report);
    let intrinsic = candidates
        .iter()
        .find_map(|key| report.metrics.get(&format!("intrinsic::{key}")).copied());
    let accelerated = candidates
        .iter()
        .find_map(|key| report.metrics.get(&format!("accelerated::{key}")).copied());
    (intrinsic, accelerated)
}

pub(crate) fn check_benchmark_threshold(
    benchmark_id: &str,
    mode: &str,
    metrics: &BTreeMap<String, f64>,
    thresholds_path: &Path,
) -> anyhow::Result<bool> {
    if !thresholds_path.exists() {
        return Ok(true);
    }
    let raw = fs::read_to_string(thresholds_path)?;
    let thresholds: JsonValue = serde_json::from_str(&raw)?;
    let bench_thresholds = thresholds
        .get(benchmark_id)
        .and_then(|b| b.get(mode))
        .and_then(JsonValue::as_object);
    if let Some(checks) = bench_thresholds {
        for (metric_name, min_value) in checks {
            let min = min_value.as_f64().unwrap_or(0.0);
            let actual = metrics.get(metric_name).copied().unwrap_or(0.0);
            if actual < min {
                eprintln!(
                    "REGRESSION: {benchmark_id} {metric_name} = {actual:.3} < threshold {min:.3}"
                );
                return Ok(false);
            }
        }
    }
    Ok(true)
}

pub(crate) fn feature_benchmark_reports_dir(output: &Path) -> PathBuf {
    output.join("benchmarks").join("features")
}

pub(crate) fn public_benchmark_dataset_cache_dir(output: &Path) -> PathBuf {
    output.join("benchmarks").join("datasets")
}

pub(crate) fn public_benchmark_docs_path(repo_root: &Path) -> PathBuf {
    benchmark_registry_markdown_path(repo_root, "PUBLIC_BENCHMARKS.md")
}

pub(crate) fn public_benchmark_leaderboard_docs_path(repo_root: &Path) -> PathBuf {
    benchmark_registry_markdown_path(repo_root, "PUBLIC_LEADERBOARD.md")
}

pub(crate) fn benchmark_registry_docs_dir(repo_root: &Path) -> PathBuf {
    repo_root.join("docs").join("verification")
}

pub(crate) fn benchmark_registry_json_path(repo_root: &Path) -> PathBuf {
    benchmark_registry_docs_dir(repo_root).join("benchmark-registry.json")
}

pub(crate) fn benchmark_registry_markdown_path(repo_root: &Path, name: &str) -> PathBuf {
    benchmark_registry_docs_dir(repo_root).join(name)
}

pub(crate) fn benchmark_telemetry_dir(output: &Path) -> PathBuf {
    output.join("telemetry").join("continuity")
}

pub(crate) fn read_latest_feature_benchmark_report(
    output: &Path,
) -> anyhow::Result<Option<FeatureBenchmarkReport>> {
    let path = feature_benchmark_reports_dir(output).join("latest.json");
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let report = serde_json::from_str::<FeatureBenchmarkReport>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(report))
}

pub(crate) fn load_benchmark_registry_for_output(
    output: &Path,
) -> anyhow::Result<Option<(PathBuf, BenchmarkRegistry)>> {
    let Some(repo_root) = infer_bundle_project_root(output) else {
        return Ok(None);
    };
    let registry_path = benchmark_registry_json_path(&repo_root);
    let registry_json = fs::read_to_string(&registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let registry = serde_json::from_str::<BenchmarkRegistry>(&registry_json)
        .with_context(|| format!("parse {}", registry_path.display()))?;
    Ok(Some((repo_root, registry)))
}

pub(crate) fn build_telemetry_benchmark_coverage(
    output: &Path,
) -> anyhow::Result<Option<BenchmarkCoverageTelemetry>> {
    let Some((_, registry)) = load_benchmark_registry_for_output(output)? else {
        return Ok(None);
    };
    let benchmark = read_latest_feature_benchmark_report(output)?;
    Ok(Some(build_benchmark_coverage_telemetry(
        &registry,
        benchmark.as_ref(),
    )))
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchmarkCoverageTelemetry {
    pub(crate) continuity_critical_total: usize,
    pub(crate) continuity_critical_benchmarked: usize,
    pub(crate) missing_loop_count: usize,
    pub(crate) with_memd_losses: usize,
    pub(crate) gap_candidates: Vec<GapCandidate>,
}

pub(crate) fn build_benchmark_coverage_telemetry(
    registry: &BenchmarkRegistry,
    benchmark: Option<&FeatureBenchmarkReport>,
) -> BenchmarkCoverageTelemetry {
    let continuity_critical_total = registry
        .features
        .iter()
        .filter(|feature| feature.continuity_critical)
        .count();
    let continuity_critical_benchmarked = registry
        .features
        .iter()
        .filter(|feature| feature.continuity_critical && feature.coverage_status == "verified")
        .count();
    let missing_loop_count = registry
        .features
        .iter()
        .filter(|feature| feature.continuity_critical && feature.coverage_status != "verified")
        .count();
    let with_memd_losses = benchmark
        .and_then(build_benchmark_comparison_report)
        .map(|report| usize::from(!report.with_memd_better))
        .unwrap_or(0);

    BenchmarkCoverageTelemetry {
        continuity_critical_total,
        continuity_critical_benchmarked,
        missing_loop_count,
        with_memd_losses,
        gap_candidates: build_benchmark_gap_candidates(registry),
    }
}
