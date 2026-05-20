use super::*;

#[derive(Debug, Clone, Args)]
pub(crate) struct CorrectionArgs {
    #[command(subcommand)]
    pub(crate) command: CorrectionSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum CorrectionSubcommand {
    Detect(CorrectionDetectArgs),
    Capture(CorrectionCaptureArgs),
    List(CorrectionListArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct CorrectionDetectArgs {
    /// Single-turn text payload.
    #[arg(long)]
    pub(crate) turn: String,
    /// Session id stamped into the NDJSON row.
    #[arg(long)]
    pub(crate) session_id: Option<String>,
    /// JSON-encoded prior claims: [{"id":"...","turn":"...","content":"..."}].
    #[arg(long)]
    pub(crate) prior: Option<String>,
    /// Skip judge call even if proxy is reachable.
    #[arg(long, default_value_t = false)]
    pub(crate) no_judge: bool,
    /// Bundle root (defaults to .memd).
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,
    /// Print full JSON instead of summary line.
    #[arg(long, default_value_t = false)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct CorrectionCaptureArgs {
    #[arg(long)]
    pub(crate) content: String,
    #[arg(long = "corrects-id")]
    pub(crate) corrects_id: Option<String>,
    #[arg(long = "source-turn")]
    pub(crate) source_turn: Option<String>,
    #[arg(long, default_value_t = 0.85_f32)]
    pub(crate) confidence: f32,
    #[arg(long = "captured-by", default_value = "manual")]
    pub(crate) captured_by: String,
    #[arg(long)]
    pub(crate) session_id: Option<String>,
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct CorrectionListArgs {
    #[arg(long)]
    pub(crate) session_id: Option<String>,
    #[arg(long)]
    pub(crate) since: Option<String>,
    #[arg(long, default_value_t = 50)]
    pub(crate) limit: usize,
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,
    #[arg(long, default_value_t = false)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct PreferenceArgs {
    #[command(subcommand)]
    pub(crate) command: PreferenceSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum PreferenceSubcommand {
    /// Print outstanding drift state.
    List(PreferenceListArgs),
    /// Force-record a drift verdict (manual or test-injected).
    Drift(PreferenceDriftArgs),
    /// Acknowledge an outstanding drift entry.
    Confirm(PreferenceConfirmArgs),
    /// Promote a preference via the C4 correction-capture path.
    Promote(PreferencePromoteArgs),
    /// Per-turn drift tick. Invoked from PostToolUse hook; rate-limited by
    /// MEMD_F4_DRIFT_N_TURNS (default 10). Prints `fire=true` every Nth turn.
    Tick(PreferenceTickArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct OfflineArgs {
    #[command(subcommand)]
    pub(crate) command: OfflineSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum OfflineSubcommand {
    Status(OfflineQueueArgs),
    Replay(OfflineQueueArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct OfflineQueueArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct PreferenceListArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,
    #[arg(long, default_value_t = false)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct PreferenceDriftArgs {
    #[arg(long = "preference-id")]
    pub(crate) preference_id: String,
    /// Preference content (terse rule string). Required when calling the judge;
    /// optional when `--verdict` is supplied (manual override).
    #[arg(long = "preference-content")]
    pub(crate) preference_content: Option<String>,
    /// JSON array of recent agent turns (strings). Required when judge runs.
    #[arg(long = "turns-json")]
    pub(crate) turns_json: Option<String>,
    /// Manual override / test injection: `drift|aligned|unknown`. Skips judge.
    #[arg(long)]
    pub(crate) verdict: Option<String>,
    #[arg(long)]
    pub(crate) confidence: Option<f32>,
    #[arg(long = "violation-count")]
    pub(crate) violation_count: Option<u32>,
    #[arg(long)]
    pub(crate) rationale: Option<String>,
    /// Stamped into the outstanding entry as `checked_turns`.
    #[arg(long = "checked-turns")]
    pub(crate) checked_turns: Option<u32>,
    #[arg(long)]
    pub(crate) session_id: Option<String>,
    /// Skip judge call even if proxy is reachable; requires `--verdict`.
    #[arg(long, default_value_t = false)]
    pub(crate) no_judge: bool,
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,
    #[arg(long, default_value_t = false)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct PreferenceConfirmArgs {
    #[arg(long = "preference-id")]
    pub(crate) preference_id: String,
    /// Also promote via the C4 correction-capture path. Requires
    /// `--preference-content`.
    #[arg(long, default_value_t = false)]
    pub(crate) promote: bool,
    #[arg(long = "preference-content")]
    pub(crate) preference_content: Option<String>,
    #[arg(long, default_value_t = 0.95_f32)]
    pub(crate) confidence: f32,
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct PreferenceTickArgs {
    /// Override turn interval. Default reads `MEMD_F4_DRIFT_N_TURNS` (10).
    #[arg(long)]
    pub(crate) n_turns: Option<u32>,
    /// Force the gate open even when `MEMD_F4_PREF_DRIFT` is unset. Used by
    /// tests; production callers honor the env gate by default.
    #[arg(long, default_value_t = false)]
    pub(crate) force_enabled: bool,
    /// Session ID stamped into the `preference-drift.ndjson` row when
    /// the tick fires. Hooks pass this in from the harness so dogfood
    /// rows are joinable to the originating session.
    #[arg(long)]
    pub(crate) session_id: Option<String>,
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,
    #[arg(long, default_value_t = false)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct PreferencePromoteArgs {
    #[arg(long = "preference-id")]
    pub(crate) preference_id: String,
    #[arg(long = "preference-content")]
    pub(crate) preference_content: String,
    #[arg(long, default_value_t = 0.95_f32)]
    pub(crate) confidence: f32,
    #[arg(long)]
    pub(crate) session_id: Option<String>,
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ContractArgs {
    #[command(subcommand)]
    pub(crate) command: ContractCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum ContractCommand {
    /// Verify the current bundle against `.memd/contract.json`.
    Verify(ContractVerifyArgs),
    /// Write the default contract shape to `.memd/contract.json`.
    Generate(ContractGenerateArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ContractVerifyArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    /// Emit violations as JSON instead of human-readable text.
    #[arg(long, default_value_t = false)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ContractGenerateArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    /// Overwrite an existing contract.json.
    #[arg(long, default_value_t = false)]
    pub(crate) force: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct PrimeReadsArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,
    /// Read a specific session's live ledger instead of the newest sealed.
    #[arg(long)]
    pub(crate) since_session: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct DiagnosticsArgs {
    #[command(subcommand)]
    pub(crate) command: DiagnosticsCommand,

    #[arg(long, default_value_t = default_base_url())]
    pub(crate) base_url: String,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum DiagnosticsCommand {
    /// Combined measurement report: token efficiency, decay, compaction, benchmarks.
    Report(DiagnosticsReportArgs),
    /// Per-kind token efficiency for a given project context.
    TokenEfficiency(DiagnosticsTokenEfficiencyArgs),
    /// Working-memory lifecycle self-test: store -> recall -> expire -> verify.
    #[command(name = "lifecycle-probe")]
    LifecycleProbe(DiagnosticsLifecycleProbeArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct DiagnosticsLifecycleProbeArgs {
    /// Bundle output directory (unused today, reserved for probe logs).
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    /// Emit human-readable summary instead of JSON.
    #[arg(long, default_value_t = false)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct DiagnosticsReportArgs {
    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) agent: Option<String>,

    /// Bundle output directory (reads cached wake token metrics).
    #[arg(long)]
    pub(crate) output: Option<std::path::PathBuf>,

    /// Output as JSON instead of human-readable text.
    #[arg(long, default_value_t = false)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct DiagnosticsTokenEfficiencyArgs {
    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) agent: Option<String>,

    #[arg(long, default_value_t = false)]
    pub(crate) json: bool,
}
