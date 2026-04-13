use super::*;
use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "memd")]
#[command(about = "Compact CLI for memd")]
pub(crate) struct Cli {
    #[arg(long, default_value_t = default_base_url())]
    pub(crate) base_url: String,

    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    Healthz,
    Status(StatusArgs),
    Capabilities(CapabilitiesArgs),
    Session(SessionArgs),
    Wake(WakeArgs),
    Awareness(AwarenessArgs),
    Heartbeat(HeartbeatArgs),
    Claims(ClaimsArgs),
    Messages(MessagesArgs),
    Tasks(TasksArgs),
    Coordination(CoordinationArgs),
    Bundle(BundleArgs),
    Hive(HiveArgs),
    HiveProject(HiveProjectArgs),
    #[command(name = "hive-join", alias = "hive-fix")]
    HiveJoin(HiveJoinArgs),
    Eval(EvalArgs),
    Gap(GapArgs),
    Improve(ImproveArgs),
    Scenario(ScenarioArgs),
    Composite(CompositeArgs),
    Benchmark(BenchmarkArgs),
    Verify(VerifyArgs),
    Experiment(ExperimentArgs),
    Agent(AgentArgs),
    Attach(AttachArgs),
    Resume(ResumeArgs),
    #[command(visible_alias = "reload")]
    Refresh(ResumeArgs),
    Watch(WatchArgs),
    Handoff(HandoffArgs),
    Checkpoint(CheckpointArgs),
    Remember(RememberArgs),
    Rag(RagArgs),
    Multimodal(MultimodalArgs),
    Ingest(IngestArgs),
    Inspiration(InspirationArgs),
    Skills(SkillsArgs),
    Packs(PacksArgs),
    Commands(CommandCatalogArgs),
    Setup(SetupArgs),
    Doctor(DoctorArgs),
    Config(ConfigArgs),
    Memory(MemoryArgs),
    Store(RequestInput),
    Candidate(RequestInput),
    Promote(RequestInput),
    Expire(RequestInput),
    #[command(name = "memory-verify")]
    MemoryVerify(RequestInput),
    Repair(RepairArgs),
    Search(SearchArgs),
    Lookup(LookupArgs),
    Context(ContextArgs),
    Working(WorkingArgs),
    Profile(ProfileArgs),
    Source(SourceArgs),
    Workspaces(SourceArgs),
    Inbox(InboxArgs),
    Explain(ExplainArgs),
    Entity(EntityArgs),
    EntitySearch(EntitySearchArgs),
    EntityLink(EntityLinkArgs),
    EntityLinks(EntityLinksArgs),
    Recall(RecallArgs),
    Timeline(TimelineArgs),
    Atlas(AtlasArgs),
    Procedure(ProcedureArgs),
    Events(EventsArgs),
    Consolidate(ConsolidateArgs),
    MaintenanceReport(MaintenanceReportArgs),
    Maintain(MaintainArgs),
    Policy(PolicyArgs),
    SkillPolicy(PolicyArgs),
    Compact(CompactArgs),
    Obsidian(ObsidianArgs),
    Ui(UiArgs),
    Hook(HookArgs),
    Init(InitArgs),
    Loops(LoopsArgs),
    Telemetry(TelemetryArgs),
    Autoresearch(AutoresearchArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct RepairArgs {
    #[arg(long)]
    pub(crate) id: String,

    #[arg(long)]
    pub(crate) mode: String,

    #[arg(long)]
    pub(crate) confidence: Option<f32>,

    #[arg(long)]
    pub(crate) status: Option<String>,

    #[arg(long)]
    pub(crate) workspace: Option<String>,

    #[arg(long)]
    pub(crate) visibility: Option<String>,

    #[arg(long)]
    pub(crate) source_agent: Option<String>,

    #[arg(long)]
    pub(crate) source_system: Option<String>,

    #[arg(long)]
    pub(crate) source_path: Option<String>,

    #[arg(long)]
    pub(crate) source_quality: Option<String>,

    #[arg(long)]
    pub(crate) content: Option<String>,

    #[arg(long, value_name = "TEXT")]
    pub(crate) tag: Vec<String>,

    #[arg(long, value_name = "UUID")]
    pub(crate) supersede: Vec<String>,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(long)]
    pub(crate) follow: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ContextArgs {
    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) agent: Option<String>,

    #[arg(long)]
    pub(crate) workspace: Option<String>,

    #[arg(long)]
    pub(crate) visibility: Option<String>,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) max_chars_per_item: Option<usize>,

    #[arg(long)]
    pub(crate) route: Option<String>,

    #[arg(long)]
    pub(crate) intent: Option<String>,

    #[arg(long)]
    pub(crate) compact: bool,

    #[arg(long)]
    pub(crate) json: Option<String>,

    #[arg(long)]
    pub(crate) input: Option<PathBuf>,

    #[arg(long)]
    pub(crate) stdin: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct WorkingArgs {
    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) agent: Option<String>,

    #[arg(long)]
    pub(crate) workspace: Option<String>,

    #[arg(long)]
    pub(crate) visibility: Option<String>,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) max_chars_per_item: Option<usize>,

    #[arg(long)]
    pub(crate) max_total_chars: Option<usize>,

    #[arg(long)]
    pub(crate) rehydration_limit: Option<usize>,

    #[arg(long)]
    pub(crate) route: Option<String>,

    #[arg(long)]
    pub(crate) intent: Option<String>,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(long)]
    pub(crate) follow: bool,

    #[arg(long)]
    pub(crate) auto_consolidate: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ProfileArgs {
    #[arg(long, default_value = "auto")]
    pub(crate) agent: String,

    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) set: bool,

    #[arg(long)]
    pub(crate) preferred_route: Option<String>,

    #[arg(long)]
    pub(crate) preferred_intent: Option<String>,

    #[arg(long)]
    pub(crate) summary_chars: Option<usize>,

    #[arg(long)]
    pub(crate) max_total_chars: Option<usize>,

    #[arg(long)]
    pub(crate) recall_depth: Option<usize>,

    #[arg(long)]
    pub(crate) source_trust_floor: Option<f32>,

    #[arg(long, value_name = "TEXT")]
    pub(crate) style_tag: Vec<String>,

    #[arg(long)]
    pub(crate) notes: Option<String>,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(long)]
    pub(crate) follow: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct SourceArgs {
    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) workspace: Option<String>,

    #[arg(long)]
    pub(crate) visibility: Option<String>,

    #[arg(long)]
    pub(crate) source_agent: Option<String>,

    #[arg(long)]
    pub(crate) source_system: Option<String>,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(long)]
    pub(crate) follow: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct InboxArgs {
    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) workspace: Option<String>,

    #[arg(long)]
    pub(crate) visibility: Option<String>,

    #[arg(long)]
    pub(crate) belief_branch: Option<String>,

    #[arg(long)]
    pub(crate) route: Option<String>,

    #[arg(long)]
    pub(crate) intent: Option<String>,

    #[arg(long)]
    pub(crate) limit: Option<usize>,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ExplainArgs {
    #[arg(long)]
    pub(crate) id: String,

    #[arg(long)]
    pub(crate) belief_branch: Option<String>,

    #[arg(long)]
    pub(crate) route: Option<String>,

    #[arg(long)]
    pub(crate) intent: Option<String>,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(long)]
    pub(crate) follow: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct EntityArgs {
    #[arg(long)]
    pub(crate) id: String,

    #[arg(long)]
    pub(crate) route: Option<String>,

    #[arg(long)]
    pub(crate) intent: Option<String>,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(long)]
    pub(crate) follow: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct EntitySearchArgs {
    #[arg(long)]
    pub(crate) query: String,

    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) at: Option<String>,

    #[arg(long)]
    pub(crate) host: Option<String>,

    #[arg(long)]
    pub(crate) branch: Option<String>,

    #[arg(long)]
    pub(crate) location: Option<String>,

    #[arg(long)]
    pub(crate) route: Option<String>,

    #[arg(long)]
    pub(crate) intent: Option<String>,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(long)]
    pub(crate) follow: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct EntityLinkArgs {
    #[arg(long)]
    pub(crate) from_entity_id: String,

    #[arg(long)]
    pub(crate) to_entity_id: String,

    #[arg(long)]
    pub(crate) relation_kind: String,

    #[arg(long)]
    pub(crate) confidence: Option<f32>,

    #[arg(long)]
    pub(crate) note: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct EntityLinksArgs {
    #[arg(long)]
    pub(crate) entity_id: String,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct RecallArgs {
    #[arg(long)]
    pub(crate) entity_id: Option<String>,

    #[arg(long)]
    pub(crate) query: Option<String>,

    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) at: Option<String>,

    #[arg(long)]
    pub(crate) host: Option<String>,

    #[arg(long)]
    pub(crate) branch: Option<String>,

    #[arg(long)]
    pub(crate) location: Option<String>,

    #[arg(long)]
    pub(crate) depth: Option<usize>,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(long)]
    pub(crate) follow: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct TimelineArgs {
    #[arg(long)]
    pub(crate) id: String,

    #[arg(long)]
    pub(crate) route: Option<String>,

    #[arg(long)]
    pub(crate) intent: Option<String>,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(long)]
    pub(crate) follow: bool,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum AtlasCommand {
    Regions(AtlasRegionsArgs),
    Explore(AtlasExploreArgs),
    Generate(AtlasRegionsArgs),
    Compile(AtlasCompileArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct AtlasArgs {
    #[command(subcommand)]
    pub(crate) command: AtlasCommand,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct AtlasRegionsArgs {
    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) lane: Option<String>,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct AtlasExploreArgs {
    #[arg(long)]
    pub(crate) region: Option<String>,

    #[arg(long)]
    pub(crate) node: Option<String>,

    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) lane: Option<String>,

    #[arg(long)]
    pub(crate) depth: Option<usize>,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) kind: Option<String>,

    #[arg(long)]
    pub(crate) scope: Option<String>,

    #[arg(long)]
    pub(crate) source_agent: Option<String>,

    #[arg(long)]
    pub(crate) source_system: Option<String>,

    #[arg(long)]
    pub(crate) min_trust: Option<f32>,

    #[arg(long)]
    pub(crate) min_salience: Option<f32>,

    #[arg(long)]
    pub(crate) include_evidence: bool,

    #[arg(long)]
    pub(crate) from_working: bool,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct AtlasCompileArgs {
    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) vault: Option<String>,

    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,
}

// ---------------------------------------------------------------------------
// Procedural memory CLI args (Phase G)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum ProcedureCommand {
    /// List procedures with optional filters.
    List(ProcedureListArgs),
    /// Record a new procedure.
    Record(ProcedureRecordArgs),
    /// Find procedures matching a context.
    Match(ProcedureMatchArgs),
    /// Promote a candidate procedure to promoted status.
    Promote(ProcedurePromoteArgs),
    /// Record a successful use of a procedure.
    Use(ProcedureUseArgs),
    /// Retire a procedure.
    Retire(ProcedureRetireArgs),
    /// Auto-detect procedures from episodic event patterns.
    Detect(ProcedureDetectArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ProcedureArgs {
    #[command(subcommand)]
    pub(crate) command: ProcedureCommand,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ProcedureListArgs {
    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) kind: Option<String>,

    #[arg(long)]
    pub(crate) status: Option<String>,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ProcedureRecordArgs {
    #[arg(long)]
    pub(crate) name: String,

    #[arg(long)]
    pub(crate) description: String,

    /// workflow, policy, or recovery
    #[arg(long, default_value = "workflow")]
    pub(crate) kind: String,

    #[arg(long)]
    pub(crate) trigger: String,

    /// Comma-separated steps
    #[arg(long)]
    pub(crate) steps: String,

    #[arg(long)]
    pub(crate) success_criteria: Option<String>,

    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    /// Comma-separated tags
    #[arg(long)]
    pub(crate) tags: Option<String>,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ProcedureMatchArgs {
    /// Context to match against
    pub(crate) context: String,

    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ProcedurePromoteArgs {
    /// Procedure ID to promote
    pub(crate) id: String,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ProcedureUseArgs {
    /// Procedure ID to record use for
    pub(crate) id: String,

    #[arg(long)]
    pub(crate) session: Option<String>,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ProcedureRetireArgs {
    /// Procedure ID to retire
    pub(crate) id: String,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ProcedureDetectArgs {
    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) min_events: Option<usize>,

    #[arg(long)]
    pub(crate) lookback_days: Option<i64>,

    #[arg(long)]
    pub(crate) max_candidates: Option<usize>,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct EventsArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) root: PathBuf,

    #[arg(long)]
    pub(crate) query: Option<String>,

    #[arg(long)]
    pub(crate) open: Option<String>,

    #[arg(long)]
    pub(crate) list: bool,

    #[arg(long, default_value_t = 12)]
    pub(crate) limit: usize,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ConsolidateArgs {
    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) max_groups: Option<usize>,

    #[arg(long)]
    pub(crate) min_events: Option<usize>,

    #[arg(long)]
    pub(crate) lookback_days: Option<i64>,

    #[arg(long)]
    pub(crate) min_salience: Option<f32>,

    #[arg(long, default_value_t = true)]
    pub(crate) record_events: bool,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(long)]
    pub(crate) follow: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct MaintenanceReportArgs {
    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) inactive_days: Option<i64>,

    #[arg(long)]
    pub(crate) lookback_days: Option<i64>,

    #[arg(long)]
    pub(crate) min_events: Option<usize>,

    #[arg(long)]
    pub(crate) max_decay: Option<f32>,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(long)]
    pub(crate) follow: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct MaintainArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long, default_value = "scan")]
    pub(crate) mode: String,

    #[arg(long)]
    pub(crate) apply: bool,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct PolicyArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) workspace: Option<String>,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(long)]
    pub(crate) follow: bool,

    #[arg(long, help = "Query stored skill-policy receipts and activations")]
    pub(crate) query: bool,

    #[arg(long, help = "Write skill-policy batch artifacts to bundle state")]
    pub(crate) write: bool,

    #[arg(
        long,
        help = "Write the activate queue artifact for downstream apply flows"
    )]
    pub(crate) apply: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ObsidianArgs {
    #[arg(long)]
    pub(crate) vault: PathBuf,

    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) workspace: Option<String>,

    #[arg(long)]
    pub(crate) visibility: Option<String>,

    #[arg(long)]
    pub(crate) max_notes: Option<usize>,

    #[arg(long)]
    pub(crate) max_attachments: Option<usize>,

    #[arg(long)]
    pub(crate) state_file: Option<PathBuf>,

    #[arg(long)]
    pub(crate) include_folder: Vec<String>,

    #[arg(long)]
    pub(crate) exclude_folder: Vec<String>,

    #[arg(long)]
    pub(crate) include_tag: Vec<String>,

    #[arg(long)]
    pub(crate) exclude_tag: Vec<String>,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(long)]
    pub(crate) follow: bool,

    #[arg(long)]
    pub(crate) review_sensitive: bool,

    #[arg(long)]
    pub(crate) include_attachments: bool,

    #[arg(long)]
    pub(crate) apply: bool,

    #[arg(long)]
    pub(crate) link_notes: bool,

    #[arg(long)]
    pub(crate) output: Option<PathBuf>,

    #[arg(long)]
    pub(crate) overwrite: bool,

    #[arg(long)]
    pub(crate) open: bool,

    #[arg(long)]
    pub(crate) pane_type: Option<String>,

    #[arg(long)]
    pub(crate) note: Option<PathBuf>,

    #[arg(long)]
    pub(crate) query: Option<String>,

    #[arg(long)]
    pub(crate) route: Option<String>,

    #[arg(long)]
    pub(crate) intent: Option<String>,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) id: Option<String>,

    #[arg(long, default_value_t = 750)]
    pub(crate) debounce_ms: u64,

    #[command(subcommand)]
    pub(crate) mode: ObsidianMode,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum ObsidianMode {
    Scan,
    Import,
    Sync,
    Compile,
    Handoff,
    Open,
    Writeback,
    Roundtrip,
    Watch,
    Status,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct UiArgs {
    #[command(subcommand)]
    pub(crate) mode: UiMode,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum UiMode {
    Home(UiHomeArgs),
    Artifact(UiArtifactArgs),
    Map(UiMapArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct UiHomeArgs {
    #[arg(long)]
    pub(crate) json: bool,

    #[arg(long)]
    pub(crate) follow: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct UiArtifactArgs {
    #[arg(long)]
    pub(crate) id: String,

    #[arg(long)]
    pub(crate) json: bool,

    #[arg(long)]
    pub(crate) follow: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct UiMapArgs {
    #[arg(long)]
    pub(crate) json: bool,

    #[arg(long)]
    pub(crate) follow: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct SearchArgs {
    #[command(flatten)]
    pub(crate) input: RequestInput,

    #[arg(long)]
    pub(crate) belief_branch: Option<String>,

    #[arg(long)]
    pub(crate) workspace: Option<String>,

    #[arg(long)]
    pub(crate) visibility: Option<String>,

    #[arg(long)]
    pub(crate) route: Option<String>,

    #[arg(long)]
    pub(crate) intent: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct LookupArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) query: String,

    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) workspace: Option<String>,

    #[arg(long)]
    pub(crate) visibility: Option<String>,

    #[arg(long)]
    pub(crate) route: Option<String>,

    #[arg(long)]
    pub(crate) intent: Option<String>,

    #[arg(long, value_name = "KIND")]
    pub(crate) kind: Vec<String>,

    #[arg(long, value_name = "TEXT")]
    pub(crate) tag: Vec<String>,

    #[arg(long)]
    pub(crate) include_stale: bool,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) verbose: bool,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct IngestArgs {
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

    #[arg(long, value_name = "TEXT")]
    pub(crate) supersede: Vec<String>,

    #[arg(long)]
    pub(crate) content: Option<String>,

    #[arg(long)]
    pub(crate) json: Option<String>,

    #[arg(long)]
    pub(crate) input: Option<PathBuf>,

    #[arg(long)]
    pub(crate) stdin: bool,

    #[arg(long)]
    pub(crate) apply: bool,

    #[arg(long)]
    pub(crate) route: Option<String>,

    #[arg(long)]
    pub(crate) intent: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct InspirationArgs {
    #[arg(long)]
    pub(crate) query: String,

    #[arg(long, default_value_t = 10)]
    pub(crate) limit: usize,

    #[arg(long)]
    pub(crate) root: Option<PathBuf>,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct SkillsArgs {
    #[arg(long)]
    pub(crate) root: Option<PathBuf>,

    #[arg(long)]
    pub(crate) query: Option<String>,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct PacksArgs {
    #[arg(long)]
    pub(crate) root: Option<PathBuf>,

    #[arg(long)]
    pub(crate) query: Option<String>,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct CommandCatalogArgs {
    #[arg(long)]
    pub(crate) root: Option<PathBuf>,

    #[arg(long)]
    pub(crate) query: Option<String>,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct SetupArgs {
    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long, default_value_t = false)]
    pub(crate) global: bool,

    #[arg(long)]
    pub(crate) project_root: Option<PathBuf>,

    #[arg(long, default_value_t = true)]
    pub(crate) seed_existing: bool,

    #[arg(long)]
    pub(crate) agent: Option<String>,

    #[arg(long)]
    pub(crate) session: Option<String>,

    #[arg(long)]
    pub(crate) tab_id: Option<String>,

    #[arg(long)]
    pub(crate) hive_system: Option<String>,

    #[arg(long)]
    pub(crate) hive_role: Option<String>,

    #[arg(long, value_name = "TEXT")]
    pub(crate) capability: Vec<String>,

    #[arg(long, value_name = "TEXT")]
    pub(crate) hive_group: Vec<String>,

    #[arg(long)]
    pub(crate) hive_group_goal: Option<String>,

    #[arg(long)]
    pub(crate) authority: Option<String>,

    #[arg(long)]
    pub(crate) output: Option<PathBuf>,

    #[arg(long)]
    pub(crate) base_url: Option<String>,

    #[arg(long)]
    pub(crate) rag_url: Option<String>,

    #[arg(long)]
    pub(crate) route: Option<String>,

    #[arg(long)]
    pub(crate) intent: Option<String>,

    #[arg(long)]
    pub(crate) workspace: Option<String>,

    #[arg(long)]
    pub(crate) visibility: Option<String>,

    #[arg(long)]
    pub(crate) voice_mode: Option<String>,

    #[arg(long)]
    pub(crate) force: bool,

    #[arg(long, default_value_t = false)]
    pub(crate) allow_localhost_read_only_fallback: bool,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct DoctorArgs {
    #[arg(long)]
    pub(crate) output: Option<PathBuf>,

    #[arg(long)]
    pub(crate) project_root: Option<PathBuf>,

    #[arg(long)]
    pub(crate) repair: bool,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ConfigArgs {
    #[arg(long)]
    pub(crate) output: Option<PathBuf>,

    #[arg(long)]
    pub(crate) project_root: Option<PathBuf>,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct MemoryArgs {
    #[arg(long)]
    pub(crate) root: Option<PathBuf>,

    #[arg(long)]
    pub(crate) query: Option<String>,

    #[arg(long)]
    pub(crate) open: Option<String>,

    #[arg(long)]
    pub(crate) lane: Option<String>,

    #[arg(long)]
    pub(crate) item: Option<String>,

    #[arg(long)]
    pub(crate) list: bool,

    #[arg(long)]
    pub(crate) lanes_only: bool,

    #[arg(long)]
    pub(crate) items_only: bool,

    #[arg(long)]
    pub(crate) filter: Option<String>,

    #[arg(long)]
    pub(crate) grouped: bool,

    #[arg(long)]
    pub(crate) expand_items: bool,

    #[arg(long)]
    pub(crate) json: bool,

    #[arg(long, default_value_t = 12)]
    pub(crate) limit: usize,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(long)]
    pub(crate) quality: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct CompactArgs {
    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) agent: Option<String>,

    #[arg(long)]
    pub(crate) task: String,

    #[arg(long)]
    pub(crate) goal: String,

    #[arg(long, value_name = "TEXT")]
    pub(crate) hard_constraint: Vec<String>,

    #[arg(long, value_name = "TEXT")]
    pub(crate) active_work: Vec<String>,

    #[arg(long, value_name = "TEXT")]
    pub(crate) decision: Vec<String>,

    #[arg(long, value_name = "TEXT")]
    pub(crate) open_loop: Vec<String>,

    #[arg(long, value_name = "TEXT")]
    pub(crate) next_action: Vec<String>,

    #[arg(long, value_name = "TEXT")]
    pub(crate) do_not_drop: Vec<String>,

    #[arg(long, value_name = "KIND=VALUE")]
    pub(crate) exact_ref: Vec<String>,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) max_chars_per_item: Option<usize>,

    #[arg(long)]
    pub(crate) route: Option<String>,

    #[arg(long)]
    pub(crate) intent: Option<String>,

    #[arg(long)]
    pub(crate) wire: bool,

    #[arg(long)]
    pub(crate) spill: bool,

    #[arg(long)]
    pub(crate) spill_transient: bool,

    #[arg(long)]
    pub(crate) apply: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HookArgs {
    #[command(subcommand)]
    pub(crate) mode: HookMode,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum HookMode {
    Context(HookContextArgs),
    Capture(HookCaptureArgs),
    Spill(HookSpillArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HookContextArgs {
    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) agent: Option<String>,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) max_chars_per_item: Option<usize>,

    #[arg(long)]
    pub(crate) route: Option<String>,

    #[arg(long)]
    pub(crate) intent: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HookSpillArgs {
    #[command(flatten)]
    pub(crate) input: RequestInput,

    #[arg(long)]
    pub(crate) apply: bool,

    #[arg(long)]
    pub(crate) spill_transient: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HookCaptureArgs {
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
    pub(crate) source_path: Option<String>,

    #[arg(long)]
    pub(crate) confidence: Option<f32>,

    #[arg(long)]
    pub(crate) ttl_seconds: Option<u64>,

    #[arg(long)]
    pub(crate) content: Option<String>,

    #[arg(long)]
    pub(crate) input: Option<PathBuf>,

    #[arg(long)]
    pub(crate) stdin: bool,

    #[arg(long, value_name = "TEXT")]
    pub(crate) tag: Vec<String>,

    #[arg(long)]
    pub(crate) promote_kind: Option<String>,

    #[arg(long)]
    pub(crate) promote_scope: Option<String>,

    #[arg(long, value_name = "UUID")]
    pub(crate) promote_supersede: Vec<String>,

    #[arg(long)]
    pub(crate) promote_supersede_query: Option<String>,

    #[arg(long, value_name = "TEXT")]
    pub(crate) promote_tag: Vec<String>,

    #[arg(long)]
    pub(crate) promote_confidence: Option<f32>,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct InitArgs {
    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long, default_value_t = false)]
    pub(crate) global: bool,

    #[arg(long)]
    pub(crate) project_root: Option<PathBuf>,

    #[arg(long, default_value_t = true)]
    pub(crate) seed_existing: bool,

    #[arg(long, default_value = "auto")]
    pub(crate) agent: String,

    #[arg(long)]
    pub(crate) session: Option<String>,

    #[arg(long)]
    pub(crate) tab_id: Option<String>,

    #[arg(long)]
    pub(crate) hive_system: Option<String>,

    #[arg(long)]
    pub(crate) hive_role: Option<String>,

    #[arg(long, value_name = "TEXT")]
    pub(crate) capability: Vec<String>,

    #[arg(long, value_name = "TEXT")]
    pub(crate) hive_group: Vec<String>,

    #[arg(long)]
    pub(crate) hive_group_goal: Option<String>,

    #[arg(long)]
    pub(crate) authority: Option<String>,

    #[arg(long, default_value_os_t = default_init_output_path())]
    pub(crate) output: PathBuf,

    #[arg(long, default_value_t = default_base_url())]
    pub(crate) base_url: String,

    #[arg(long)]
    pub(crate) rag_url: Option<String>,

    #[arg(long, default_value = "auto")]
    pub(crate) route: String,

    #[arg(long, default_value = "current_task")]
    pub(crate) intent: String,

    #[arg(long)]
    pub(crate) workspace: Option<String>,

    #[arg(long)]
    pub(crate) visibility: Option<String>,

    #[arg(long)]
    pub(crate) voice_mode: Option<String>,

    #[arg(long)]
    pub(crate) force: bool,

    #[arg(long, default_value_t = false)]
    pub(crate) allow_localhost_read_only_fallback: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct LoopsArgs {
    #[arg(long, default_value_os_t = default_init_output_path())]
    pub(crate) output: PathBuf,

    #[arg(
        long = "loop",
        value_name = "SLUG",
        help = "Show details for a recorded loop slug"
    )]
    pub(crate) loop_slug: Option<String>,

    #[arg(long, help = "Show aggregate loop metrics and improvements")]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct TelemetryArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long, help = "Emit telemetry JSON instead of text")]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct AutoresearchArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long, help = "Run all manifest loops")]
    pub(crate) auto: bool,

    #[arg(long, help = "Run a single loop by slug")]
    pub(crate) loop_slug: Option<String>,

    #[arg(long, help = "Print the manifest of available autoresearch loops")]
    pub(crate) manifest: bool,

    #[arg(
        long,
        default_value_t = 1,
        help = "Maximum number of autoresearch sweeps to run"
    )]
    pub(crate) max_sweeps: usize,

    #[arg(
        long,
        default_value_t = 0,
        help = "Stop after this many consecutive identical sweep signatures (0 disables plateau stopping)"
    )]
    pub(crate) plateau_sweeps: usize,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct StatusArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct CapabilitiesArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) harness: Option<String>,

    #[arg(long)]
    pub(crate) kind: Option<String>,

    #[arg(long)]
    pub(crate) portability: Option<String>,

    #[arg(long)]
    pub(crate) query: Option<String>,

    #[arg(long, default_value_t = 12)]
    pub(crate) limit: usize,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct AwarenessArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) root: Option<PathBuf>,

    #[arg(long, default_value_t = false)]
    pub(crate) include_current: bool,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HeartbeatArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) watch: bool,

    #[arg(long, default_value_t = 30)]
    pub(crate) interval_secs: u64,

    #[arg(long)]
    pub(crate) probe_base_url: bool,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct SessionArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) rebind: bool,

    #[arg(long)]
    pub(crate) reconcile: bool,

    #[arg(long)]
    pub(crate) retire_session: Option<String>,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ClaimsArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) acquire: bool,

    #[arg(long)]
    pub(crate) release: bool,

    #[arg(long)]
    pub(crate) transfer_to_session: Option<String>,

    #[arg(long)]
    pub(crate) scope: Option<String>,

    #[arg(long, default_value_t = 900)]
    pub(crate) ttl_secs: u64,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct MessagesArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) send: bool,

    #[arg(long)]
    pub(crate) inbox: bool,

    #[arg(long)]
    pub(crate) ack: Option<String>,

    #[arg(long)]
    pub(crate) target_session: Option<String>,

    #[arg(long)]
    pub(crate) kind: Option<String>,

    #[arg(long)]
    pub(crate) request_help: bool,

    #[arg(long)]
    pub(crate) request_review: bool,

    #[arg(long)]
    pub(crate) assign_scope: Option<String>,

    #[arg(long)]
    pub(crate) scope: Option<String>,

    #[arg(long)]
    pub(crate) content: Option<String>,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct TasksArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) upsert: bool,

    #[arg(long)]
    pub(crate) assign_to_session: Option<String>,

    #[arg(long)]
    pub(crate) target_session: Option<String>,

    #[arg(long)]
    pub(crate) task_id: Option<String>,

    #[arg(long)]
    pub(crate) title: Option<String>,

    #[arg(long)]
    pub(crate) description: Option<String>,

    #[arg(long)]
    pub(crate) status: Option<String>,

    #[arg(long)]
    pub(crate) mode: Option<String>,

    #[arg(long, value_name = "SCOPE")]
    pub(crate) scope: Vec<String>,

    #[arg(long)]
    pub(crate) request_help: bool,

    #[arg(long)]
    pub(crate) request_review: bool,

    #[arg(long)]
    pub(crate) all: bool,

    #[arg(long)]
    pub(crate) view: Option<String>,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct CoordinationArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) view: Option<String>,

    #[arg(long)]
    pub(crate) changes_only: bool,

    #[arg(long)]
    pub(crate) watch: bool,

    #[arg(long, default_value_t = 30)]
    pub(crate) interval_secs: u64,

    #[arg(long)]
    pub(crate) recover_session: Option<String>,

    #[arg(long)]
    pub(crate) retire_session: Option<String>,

    #[arg(long)]
    pub(crate) to_session: Option<String>,

    #[arg(long)]
    pub(crate) deny_session: Option<String>,

    #[arg(long)]
    pub(crate) reroute_session: Option<String>,

    #[arg(long)]
    pub(crate) handoff_scope: Option<String>,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct BundleArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) hive_system: Option<String>,

    #[arg(long)]
    pub(crate) hive_role: Option<String>,

    #[arg(long, value_name = "TEXT")]
    pub(crate) capability: Vec<String>,

    #[arg(long, value_name = "TEXT")]
    pub(crate) hive_group: Vec<String>,

    #[arg(long)]
    pub(crate) hive_group_goal: Option<String>,

    #[arg(long)]
    pub(crate) authority: Option<String>,

    #[arg(long)]
    pub(crate) base_url: Option<String>,

    #[arg(long)]
    pub(crate) route: Option<String>,

    #[arg(long)]
    pub(crate) intent: Option<String>,

    #[arg(long)]
    pub(crate) tab_id: Option<String>,

    #[arg(long)]
    pub(crate) auto_short_term_capture: Option<bool>,

    #[arg(long)]
    pub(crate) voice_mode: Option<String>,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HiveArgs {
    #[command(subcommand)]
    pub(crate) command: Option<HiveSubcommand>,

    #[arg(long)]
    pub(crate) agent: Option<String>,

    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long, default_value_t = false)]
    pub(crate) global: bool,

    #[arg(long)]
    pub(crate) project_root: Option<PathBuf>,

    #[arg(long, default_value_t = true)]
    pub(crate) seed_existing: bool,

    #[arg(long)]
    pub(crate) session: Option<String>,

    #[arg(long)]
    pub(crate) tab_id: Option<String>,

    #[arg(long)]
    pub(crate) hive_system: Option<String>,

    #[arg(long)]
    pub(crate) hive_role: Option<String>,

    #[arg(long, value_name = "TEXT")]
    pub(crate) capability: Vec<String>,

    #[arg(long, value_name = "TEXT")]
    pub(crate) hive_group: Vec<String>,

    #[arg(long)]
    pub(crate) hive_group_goal: Option<String>,

    #[arg(long)]
    pub(crate) authority: Option<String>,

    #[arg(long, default_value_os_t = default_init_output_path())]
    pub(crate) output: PathBuf,

    #[arg(long, default_value_t = default_base_url())]
    pub(crate) base_url: String,

    #[arg(long)]
    pub(crate) rag_url: Option<String>,

    #[arg(long, default_value = "auto")]
    pub(crate) route: String,

    #[arg(long, default_value = "current_task")]
    pub(crate) intent: String,

    #[arg(long)]
    pub(crate) workspace: Option<String>,

    #[arg(long)]
    pub(crate) visibility: Option<String>,

    #[arg(long, default_value_t = true)]
    pub(crate) publish_heartbeat: bool,

    #[arg(long)]
    pub(crate) force: bool,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum HiveSubcommand {
    Roster(HiveRosterArgs),
    Follow(HiveFollowArgs),
    Handoff(HiveHandoffArgs),
    Cowork {
        #[command(subcommand)]
        command: HiveCoworkSubcommand,
    },
    Queen(HiveQueenArgs),
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum HiveCoworkSubcommand {
    Request(HiveCoworkArgs),
    Ack(HiveCoworkArgs),
    Decline(HiveCoworkArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HiveRosterArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) json: bool,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HiveFollowArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) session: Option<String>,

    #[arg(long)]
    pub(crate) worker: Option<String>,

    #[arg(long)]
    pub(crate) watch: bool,

    #[arg(long, default_value_t = 5)]
    pub(crate) interval_secs: u64,

    #[arg(long)]
    pub(crate) json: bool,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HiveHandoffArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) to_session: Option<String>,

    #[arg(long)]
    pub(crate) to_worker: Option<String>,

    #[arg(long)]
    pub(crate) task_id: Option<String>,

    #[arg(long)]
    pub(crate) topic: Option<String>,

    #[arg(long, value_delimiter = ',')]
    pub(crate) scope: Vec<String>,

    #[arg(long)]
    pub(crate) next_action: Option<String>,

    #[arg(long)]
    pub(crate) blocker: Option<String>,

    #[arg(long)]
    pub(crate) note: Option<String>,

    #[arg(long)]
    pub(crate) json: bool,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HiveCoworkArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) to_session: Option<String>,

    #[arg(long)]
    pub(crate) to_worker: Option<String>,

    #[arg(long)]
    pub(crate) task_id: Option<String>,

    #[arg(long, value_delimiter = ',')]
    pub(crate) scope: Vec<String>,

    #[arg(long)]
    pub(crate) reason: Option<String>,

    #[arg(long)]
    pub(crate) note: Option<String>,

    #[arg(long)]
    pub(crate) json: bool,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HiveQueenArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) view: Option<String>,

    #[arg(long)]
    pub(crate) recover_session: Option<String>,

    #[arg(long)]
    pub(crate) retire_session: Option<String>,

    #[arg(long)]
    pub(crate) to_session: Option<String>,

    #[arg(long)]
    pub(crate) deny_session: Option<String>,

    #[arg(long)]
    pub(crate) reroute_session: Option<String>,

    #[arg(long)]
    pub(crate) handoff_scope: Option<String>,

    #[arg(long)]
    pub(crate) json: bool,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(long)]
    pub(crate) cowork_auto_send: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HiveJoinArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long, default_value_t = default_hive_join_base_url())]
    pub(crate) base_url: String,

    #[arg(long)]
    pub(crate) all_active: bool,

    #[arg(long)]
    pub(crate) all_local: bool,

    #[arg(long, default_value_t = true)]
    pub(crate) publish_heartbeat: bool,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HiveProjectArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) enable: bool,

    #[arg(long)]
    pub(crate) disable: bool,

    #[arg(long)]
    pub(crate) status: bool,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct EvalArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) rehydration_limit: Option<usize>,

    #[arg(long)]
    pub(crate) write: bool,

    #[arg(long)]
    pub(crate) fail_below: Option<u8>,

    #[arg(long)]
    pub(crate) fail_on_regression: bool,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct GapArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) recent_commits: Option<usize>,

    #[arg(long, default_value_t = false)]
    pub(crate) write: bool,

    #[arg(long, default_value_t = false)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ImproveArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long, default_value_t = 3)]
    pub(crate) max_iterations: usize,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) recent_commits: Option<usize>,

    #[arg(long, default_value_t = false)]
    pub(crate) write: bool,

    #[arg(long, default_value_t = false)]
    pub(crate) apply: bool,

    #[arg(long, default_value_t = false)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ScenarioArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) scenario: Option<String>,

    #[arg(long, default_value_t = false)]
    pub(crate) write: bool,

    #[arg(long, default_value_t = false)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct CompositeArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) scenario: Option<String>,

    #[arg(long, default_value_t = false)]
    pub(crate) write: bool,

    #[arg(long, default_value_t = false)]
    pub(crate) summary: bool,
}

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
}

#[derive(Debug, Clone, Args)]
pub(crate) struct PublicBenchmarkArgs {
    pub(crate) dataset: String,

    #[arg(long, value_parser = ["raw", "hybrid"])]
    pub(crate) mode: Option<String>,

    #[arg(long, value_parser = ["lexical", "sidecar"])]
    pub(crate) retrieval_backend: Option<String>,

    #[arg(long)]
    pub(crate) rag_url: Option<String>,

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

    #[arg(long, alias = "output", default_value_os_t = default_bundle_root_path())]
    pub(crate) out: PathBuf,
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

#[derive(Debug, Clone, Args)]
pub(crate) struct CheckpointArgs {
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
    pub(crate) source_path: Option<String>,

    #[arg(long)]
    pub(crate) confidence: Option<f32>,

    #[arg(long)]
    pub(crate) ttl_seconds: Option<u64>,

    #[arg(long, value_name = "TEXT")]
    pub(crate) tag: Vec<String>,

    #[arg(long)]
    pub(crate) content: Option<String>,

    #[arg(long)]
    pub(crate) input: Option<PathBuf>,

    #[arg(long)]
    pub(crate) stdin: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct RagArgs {
    #[arg(long)]
    pub(crate) rag_url: Option<String>,

    #[command(subcommand)]
    pub(crate) mode: RagMode,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum RagMode {
    Healthz,
    Sync(RagSyncArgs),
    Search(RagSearchArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct MultimodalArgs {
    #[arg(long)]
    pub(crate) rag_url: Option<String>,

    #[command(subcommand)]
    pub(crate) mode: MultimodalMode,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum MultimodalMode {
    Healthz,
    Plan(MultimodalPlanArgs),
    Ingest(MultimodalIngestArgs),
    Retrieve(MultimodalRetrieveArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct RagSyncArgs {
    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) dry_run: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct RagSearchArgs {
    #[arg(long)]
    pub(crate) query: String,

    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) mode: Option<String>,

    #[arg(long)]
    pub(crate) include_cross_modal: bool,

    #[arg(long)]
    pub(crate) limit: Option<usize>,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct MultimodalPlanArgs {
    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long, value_name = "PATH")]
    pub(crate) path: Vec<PathBuf>,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct MultimodalIngestArgs {
    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long, value_name = "PATH")]
    pub(crate) path: Vec<PathBuf>,

    #[arg(long)]
    pub(crate) apply: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct MultimodalRetrieveArgs {
    #[arg(long)]
    pub(crate) query: String,

    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) mode: Option<String>,

    #[arg(long)]
    pub(crate) include_cross_modal: bool,

    #[arg(long)]
    pub(crate) limit: Option<usize>,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct RequestInput {
    #[arg(long)]
    pub(crate) json: Option<String>,

    #[arg(long)]
    pub(crate) input: Option<PathBuf>,

    #[arg(long)]
    pub(crate) stdin: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct ObsidianImportOutput {
    pub(crate) preview: ObsidianImportPreview,
    pub(crate) submitted: usize,
    pub(crate) attachment_submitted: usize,
    pub(crate) duplicates: usize,
    pub(crate) attachment_duplicates: usize,
    pub(crate) note_failures: usize,
    pub(crate) attachment_failures: usize,
    pub(crate) links_created: usize,
    pub(crate) attachment_links_created: usize,
    pub(crate) mirrored_notes: usize,
    pub(crate) mirrored_attachments: usize,
    pub(crate) attachments: Option<MultimodalIngestOutput>,
    pub(crate) attachment_unchanged_count: usize,
    pub(crate) dry_run: bool,
}
