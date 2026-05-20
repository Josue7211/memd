use super::*;

#[derive(Debug, Clone, Args)]
pub(crate) struct BenchmarkArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long, default_value_t = false)]
    pub(crate) write: bool,

    #[arg(long, default_value_t = false)]
    pub(crate) summary: bool,

    #[command(subcommand)]
    pub(crate) subcommand: Option<BenchmarkSubcommand>,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum BenchmarkSubcommand {
    Public(PublicBenchmarkArgs),
    /// V5 substrate-native benchmark suites (cross-session-recall, correction-propagation, …).
    Substrate(SubstrateArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct SubstrateArgs {
    /// Suite name (e.g. cross-session-recall). Mutually exclusive with --all.
    #[arg(long)]
    pub(crate) suite: Option<String>,

    /// Run every registered substrate suite.
    #[arg(long, default_value_t = false)]
    pub(crate) all: bool,

    /// Path to bench spec YAML. Defaults to .memd/benchmarks/substrate/<suite>.yaml.
    #[arg(long)]
    pub(crate) spec: Option<PathBuf>,

    /// RNG seed override (defaults to spec value, then 42).
    #[arg(long)]
    pub(crate) seed: Option<u64>,

    /// Output dir for NDJSON results.
    #[arg(long, default_value = ".memd/benchmarks/substrate/results")]
    pub(crate) output: PathBuf,

    /// Markdown report path to append/regenerate.
    #[arg(long, default_value = "docs/verification/SUBSTRATE_BENCHMARKS.md")]
    pub(crate) report: PathBuf,

    /// Restrict to a subset of cut counts (comma-separated).
    #[arg(long)]
    pub(crate) only_cuts: Option<String>,

    /// Emit JSON to stdout instead of human summary.
    #[arg(long, default_value_t = false)]
    pub(crate) json: bool,

    /// Hard ceiling on LLM-judge spend (USD). Exit 2 if exceeded.
    #[arg(long)]
    pub(crate) max_budget_usd: Option<f64>,

    /// Regenerate locked fixtures under .memd/benchmarks/substrate/fixtures/<suite>/.
    #[arg(long, default_value_t = false)]
    pub(crate) emit_fixtures: bool,

    /// E5 only: inject a provenance hole to verify auditor catches it.
    #[arg(long, default_value_t = false)]
    pub(crate) inject_hole: bool,

    /// D5 only: restrict to a single depth class (wake, lookup, or resume).
    #[arg(long)]
    pub(crate) depth_only: Option<String>,

    /// G5 only: regenerate the canonical SUBSTRATE_BENCHMARKS.md after `--all`.
    #[arg(long, default_value_t = false)]
    pub(crate) regenerate_report: bool,

    /// G5 only: regenerate MEMD-10-STAR.md V5 axes after `--all`.
    #[arg(long, default_value_t = false)]
    pub(crate) regenerate_10star: bool,

    /// G5 only: halt on the first failing suite when running `--all`.
    #[arg(long, default_value_t = false)]
    pub(crate) fail_fast: bool,

    /// G5 only: allow regenerator to write composite below 4.20 target.
    #[arg(long, default_value_t = false)]
    pub(crate) allow_below_target: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct PublicBenchmarkArgs {
    #[arg(default_value = "")]
    pub(crate) dataset: String,

    #[arg(long, value_parser = ["raw", "hybrid"])]
    pub(crate) mode: Option<String>,

    #[arg(long, value_parser = ["lexical", "sidecar", "rrf", "memd"])]
    pub(crate) retrieval_backend: Option<String>,

    #[arg(long)]
    pub(crate) rag_url: Option<String>,

    /// B3 Part-2: base URL of a running memd-server when
    /// --retrieval-backend=memd. Defaults to http://127.0.0.1:8787.
    #[arg(long)]
    pub(crate) memd_url: Option<String>,

    #[arg(long)]
    pub(crate) top_k: Option<usize>,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) dataset_root: Option<PathBuf>,

    #[arg(long)]
    pub(crate) reranker: Option<String>,

    #[arg(long, default_value_t = false)]
    pub(crate) write: bool,

    #[arg(long, default_value_t = false)]
    pub(crate) json: bool,

    #[arg(long, default_value_t = false)]
    pub(crate) community_standard: bool,

    #[arg(long)]
    pub(crate) hypotheses_file: Option<PathBuf>,

    #[arg(long)]
    pub(crate) grader_model: Option<String>,

    #[arg(long, default_value_t = false)]
    pub(crate) full_eval: bool,

    #[arg(long)]
    pub(crate) generator_model: Option<String>,

    #[arg(long)]
    pub(crate) sample: Option<usize>,

    #[arg(long, default_value_t = false)]
    pub(crate) dry_run: bool,

    #[arg(long, default_value_t = false)]
    pub(crate) dual: bool,

    /// LongMemEval only: also compute turn-level retrieval diagnostics.
    /// Default off so the primary 500-Q gate pays only for the session metric.
    #[arg(long, default_value_t = false)]
    pub(crate) turn_diagnostics: bool,

    #[arg(long, default_value_t = false)]
    pub(crate) all: bool,

    #[arg(long, alias = "output", default_value_os_t = default_bundle_root_path())]
    pub(crate) out: PathBuf,

    /// CI gate mode: run all benchmarks, exit 1 if any drops below threshold.
    /// Thresholds: LongMemEval >= 80%, LoCoMo >= 41.5%, MemBench >= 30%.
    #[arg(long, default_value_t = false)]
    pub(crate) ci: bool,

    /// Record results to benchmark-registry.json with git SHA and timestamp.
    #[arg(long, default_value_t = false)]
    pub(crate) record: bool,

    /// V6 typed-ingest pipeline. Values:
    /// - `episodic` (A6) — per-bench `EpisodicAdapter`, ingests with
    ///   `EpisodicProvenance` metadata.
    /// - `episodic+semantic` (B6) — A6 + B6 semantic distillation,
    ///   emits `stage=candidate` records via the codex-lb judge.
    /// - `episodic+semantic+canonical` (C6) — B6 + C6 canonical
    ///   promotion under the rule card (corroboration ≥ 2, confidence
    ///   ≥ 0.8, session-age ≥ 3 turns, contradiction reuse via C4).
    /// Off by default unless the flag is passed. V6 close graduates this
    /// from calendar-gated scaffold to active public-bench typed ingest.
    #[arg(long, value_parser = ["episodic", "episodic+semantic", "episodic+semantic+canonical"])]
    pub(crate) typed_ingest: Option<String>,

    /// V6/B6 distillation judge model. Default `gpt-5.4` via codex-lb.
    /// Overridable per run; also overridable via `MEMD_V6_DISTILL_MODEL`.
    #[arg(long, default_value = "gpt-5.4")]
    pub(crate) distill_model: String,

    /// V6/B6 per-run distillation budget in milli-USD. The judge stops
    /// emitting candidates once spend ≥ budget; cache hits are free.
    #[arg(long, default_value_t = 100u64)]
    pub(crate) distill_budget_milli_usd: u64,

    /// V6/B6 distillation cache directory. Defaults to
    /// `.memd/benchmarks/public/cache/distill/` relative to the bundle.
    #[arg(long)]
    pub(crate) distill_cache_dir: Option<PathBuf>,

    /// V6/C6 promotion dry-run. Emits the same NDJSON telemetry as a
    /// real promotion but does not write to the canonical index. Also
    /// forced on by `MEMD_V6_PROMOTION_DRY_RUN=1`.
    #[arg(long, default_value_t = false)]
    pub(crate) promotion_dry_run: bool,

    /// V6/D6 bench-compiler A/B switch. `on` routes the answer prompt
    /// through `runtime::resume::compiler::compile_wake` with the
    /// per-bench budget profile from
    /// `.memd/benchmarks/public/compiler-budgets.json`. `off` (default)
    /// preserves the legacy flat-RAG prompt path verbatim. Also
    /// promoted to `on` by `MEMD_V6_COMPILER=1`.
    #[arg(long, value_parser = ["on", "off"], default_value = "off")]
    pub(crate) compiler: String,

    /// V6/E6 progressive-depth routing. `on` (default) enables the
    /// multi-call tool-call loop: model can re-query memd mid-answer
    /// across the wake/targeted/resume tiers, capped by
    /// `--max-depth-calls` and `--max-retrieval-tokens`. `off`
    /// preserves the single-call legacy path. Also forced off by
    /// `MEMD_V6_DEPTH_ROUTING=0`.
    #[arg(long, value_parser = ["on", "off"], default_value = "on")]
    pub(crate) depth_routing: String,

    /// V6/E6 hard cap on lookups per answer. Default 3. Override via
    /// `MEMD_V6_MAX_DEPTH_CALLS`.
    #[arg(long, default_value_t = 3usize)]
    pub(crate) max_depth_calls: usize,

    /// V6/E6 hard cap on retrieved-content tokens per answer
    /// (chars-as-tokens, V4 convention). Default 10000.
    #[arg(long, default_value_t = 10_000usize)]
    pub(crate) max_retrieval_tokens: usize,

    /// V6/F6 iterative-reasoning harness. `on` (default) chains up to
    /// `--max-reasoning-steps` depth-routed lookups into a single
    /// answer scratchpad. `off` preserves the E6 single-call path.
    /// Forced off by `MEMD_V6_REASONING=0`.
    #[arg(long, value_parser = ["on", "off"], default_value = "on")]
    pub(crate) reasoning: String,

    /// V6/F6 hard cap on reasoning steps per question. Default 5.
    /// Override via `MEMD_V6_MAX_REASONING_STEPS`.
    #[arg(long, default_value_t = 5usize)]
    pub(crate) max_reasoning_steps: usize,

    /// V6/F6 hard cap on retrieved-content tokens across the full
    /// reasoning chain. Default 20000 (above E6's per-answer cap so
    /// multi-step chains have slack).
    #[arg(long, default_value_t = 20_000usize)]
    pub(crate) max_reasoning_tokens: usize,

    /// V6/F6 regenerate `docs/verification/PUBLIC_BENCHMARKS.md`
    /// after running the canonical sweep. No-op when no per-bench
    /// scorecards have been written yet.
    #[arg(long, default_value_t = false)]
    pub(crate) regenerate_report: bool,

    /// V6/F6 regenerate `docs/verification/MEMD-10-STAR.md` from the
    /// V6 axis deltas. Refuses to publish the V6 milestone claim with
    /// composite < 4.45 unless `--allow-below-target` is set.
    #[arg(long, default_value_t = false)]
    pub(crate) regenerate_10star: bool,

    /// V6/F6 allow the 10-STAR regenerator to publish a composite
    /// below the 4.45 V6 milestone threshold. Also forced on by
    /// `MEMD_V6_ALLOW_BELOW_TARGET=1`.
    #[arg(long, default_value_t = false)]
    pub(crate) allow_below_target: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct VerifyArgs {
    #[command(subcommand)]
    pub(crate) command: VerifyCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum VerifyCommand {
    Feature(VerifyFeatureArgs),
    Journey(VerifyJourneyArgs),
    Adversarial(VerifyAdversarialArgs),
    Compare(VerifyCompareArgs),
    Sweep(VerifySweepArgs),
    Doctor(VerifyDoctorArgs),
    List(VerifyListArgs),
    Show(VerifyShowArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct VerifyFeatureArgs {
    pub(crate) feature_id: String,

    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long, default_value_t = false)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct VerifyJourneyArgs {
    pub(crate) journey_id: String,

    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long, default_value_t = false)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct VerifyAdversarialArgs {
    pub(crate) verifier_id: String,

    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long, default_value_t = false)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct VerifyCompareArgs {
    pub(crate) verifier_id: String,

    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long, default_value_t = false)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct VerifySweepArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long, default_value = "fast")]
    pub(crate) lane: String,

    #[arg(long, default_value_t = false)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct VerifyDoctorArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long, default_value_t = false)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct VerifyListArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) lane: Option<String>,

    #[arg(long, default_value_t = false)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct VerifyShowArgs {
    pub(crate) item_id: String,

    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long, default_value_t = false)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ExperimentArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long, default_value_t = 2)]
    pub(crate) max_iterations: usize,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) recent_commits: Option<usize>,

    #[arg(long, default_value_t = 80)]
    pub(crate) accept_below: u8,

    #[arg(long, default_value_t = true)]
    pub(crate) apply: bool,

    #[arg(long, default_value_t = true)]
    pub(crate) consolidate: bool,

    #[arg(long, default_value_t = false)]
    pub(crate) write: bool,

    #[arg(long, default_value_t = false)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct AttachArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) shell: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct AgentArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) name: Option<String>,

    #[arg(long)]
    pub(crate) shell: Option<String>,

    #[arg(long)]
    pub(crate) session: Option<String>,

    #[arg(long)]
    pub(crate) apply: bool,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ResumeArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) agent: Option<String>,

    #[arg(long)]
    pub(crate) workspace: Option<String>,

    #[arg(long)]
    pub(crate) visibility: Option<String>,

    #[arg(long)]
    pub(crate) route: Option<String>,

    #[arg(long)]
    pub(crate) intent: Option<String>,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) rehydration_limit: Option<usize>,

    #[arg(long)]
    pub(crate) semantic: bool,

    #[arg(long)]
    pub(crate) prompt: bool,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct WatchArgs {
    #[arg(long, default_value_os_t = std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))]
    pub(crate) root: PathBuf,

    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) agent: Option<String>,

    #[arg(long)]
    pub(crate) workspace: Option<String>,

    #[arg(long)]
    pub(crate) visibility: Option<String>,

    #[arg(long)]
    pub(crate) route: Option<String>,

    #[arg(long)]
    pub(crate) intent: Option<String>,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) rehydration_limit: Option<usize>,

    #[arg(long)]
    pub(crate) semantic: bool,

    #[arg(long, default_value_t = 750)]
    pub(crate) debounce_ms: u64,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct WakeArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) agent: Option<String>,

    #[arg(long)]
    pub(crate) workspace: Option<String>,

    #[arg(long)]
    pub(crate) visibility: Option<String>,

    #[arg(long)]
    pub(crate) route: Option<String>,

    #[arg(long)]
    pub(crate) intent: Option<String>,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) rehydration_limit: Option<usize>,

    #[arg(long)]
    pub(crate) semantic: bool,

    #[arg(long)]
    pub(crate) verbose: bool,

    #[arg(long)]
    pub(crate) write: bool,

    #[arg(long)]
    pub(crate) summary: bool,

    /// D4: bypass the wake-context compiler and emit the legacy raw render.
    #[arg(long)]
    pub(crate) raw: bool,

    /// D4: override `MEMD_WAKE_BUDGET_TOKENS` (chars). 0 = use env/default.
    #[arg(long, default_value_t = 0)]
    pub(crate) budget_tokens: usize,

    /// D4: force-include a bucket even when over budget. Repeatable.
    #[arg(long = "include-bucket")]
    pub(crate) include_bucket: Vec<String>,

    /// D4: force-exclude a bucket regardless of priority. Repeatable.
    #[arg(long = "exclude-bucket")]
    pub(crate) exclude_bucket: Vec<String>,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HandoffArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) target_session: Option<String>,

    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) agent: Option<String>,

    #[arg(long)]
    pub(crate) workspace: Option<String>,

    #[arg(long)]
    pub(crate) visibility: Option<String>,

    #[arg(long)]
    pub(crate) route: Option<String>,

    #[arg(long)]
    pub(crate) intent: Option<String>,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) rehydration_limit: Option<usize>,

    #[arg(long)]
    pub(crate) source_limit: Option<usize>,

    #[arg(long)]
    pub(crate) semantic: bool,

    #[arg(long)]
    pub(crate) prompt: bool,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct RememberArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) workspace: Option<String>,

    #[arg(long)]
    pub(crate) visibility: Option<String>,

    #[arg(long)]
    pub(crate) kind: Option<String>,

    #[arg(long)]
    pub(crate) scope: Option<String>,

    #[arg(long)]
    pub(crate) source_agent: Option<String>,

    #[arg(long)]
    pub(crate) source_system: Option<String>,

    #[arg(long)]
    pub(crate) source_path: Option<String>,

    #[arg(long)]
    pub(crate) source_quality: Option<String>,

    #[arg(long)]
    pub(crate) confidence: Option<f32>,

    #[arg(long)]
    pub(crate) ttl_seconds: Option<u64>,

    #[arg(long, value_name = "TEXT")]
    pub(crate) tag: Vec<String>,

    #[arg(long, value_name = "UUID")]
    pub(crate) supersede: Vec<String>,

    #[arg(long)]
    pub(crate) content: Option<String>,

    #[arg(long)]
    pub(crate) input: Option<PathBuf>,

    #[arg(long)]
    pub(crate) stdin: bool,
}
