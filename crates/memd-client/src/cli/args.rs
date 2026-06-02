use super::*;
use clap::{Args, Parser, Subcommand};

#[path = "args_coordination.rs"]
mod args_coordination;
#[path = "args_lanes.rs"]
mod args_lanes;
#[path = "args_runtime.rs"]
mod args_runtime;
#[path = "args_skill.rs"]
mod args_skill;

pub(crate) use args_coordination::*;
pub(crate) use args_lanes::*;
pub(crate) use args_runtime::*;
pub(crate) use args_skill::*;

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
    /// Show local bundle readiness, server health, memory context, and next actions.
    Status(StatusArgs),
    State(StateArgs),
    Claim(ClaimArgs),
    Capabilities(CapabilitiesArgs),
    Session(SessionArgs),
    /// Refresh/rehydrate the current agent session from saved memd context.
    Wake(WakeArgs),
    Awareness(AwarenessArgs),
    Heartbeat(HeartbeatArgs),
    Features(FeaturesArgs),
    Health(HealthArgs),
    Access(AccessArgs),
    #[command(name = "live-state")]
    LiveState(LiveStateArgs),
    Secrets(SecretsArgs),
    Tokens(TokensArgs),
    #[command(name = "dev-server")]
    DevServer(DevServerArgs),
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
    /// Refresh the active session context; alias: reload.
    #[command(visible_alias = "reload")]
    Refresh(ResumeArgs),
    Watch(WatchArgs),
    Handoff(HandoffArgs),
    Checkpoint(CheckpointArgs),
    /// Save a durable memory item with project/workspace metadata.
    Remember(RememberArgs),
    /// Teach memd a canonical fact, preference, or procedure for future recall.
    Teach(TeachArgs),
    Embed(EmbedArgs),
    Rag(RagArgs),
    Offline(OfflineArgs),
    Sync(OfflineArgs),
    Multimodal(MultimodalArgs),
    Ingest(IngestArgs),
    #[command(name = "ingest-sources")]
    IngestSources(IngestSourcesArgs),
    Inspiration(InspirationArgs),
    Skill(SkillArgs),
    /// Browse installed skills and skill catalog entries.
    Skills(SkillsArgs),
    Packs(PacksArgs),
    Commands(CommandCatalogArgs),
    /// Configure memd for a local project, provider, and harness.
    Setup(SetupArgs),
    /// Run an isolated setup proof without changing the current repository.
    #[command(name = "setup-demo")]
    SetupDemo(SetupDemoArgs),
    /// Check local memd health and print actionable repair guidance.
    Doctor(DoctorArgs),
    Device(DeviceArgs),
    Dogfood(DogfoodArgs),
    /// View or edit runtime settings; aliases: configure, settings.
    #[command(visible_alias = "configure", visible_alias = "settings")]
    Config(ConfigArgs),
    /// Browse lanes, items, and local memory artifacts.
    Memory(MemoryArgs),
    Store(RequestInput),
    Candidate(RequestInput),
    Promote(RequestInput),
    Expire(RequestInput),
    #[command(name = "memory-verify")]
    MemoryVerify(RequestInput),
    Repair(RepairArgs),
    Correct(CorrectArgs),
    Search(SearchArgs),
    /// Retrieve relevant memories for a query with filters and optional JSON.
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
    Routines(RoutinesArgs),
    Audit(AuditArgs),
    Events(EventsArgs),
    Consolidate(ConsolidateArgs),
    /// E3-D5: scan existing memory vectors for near-duplicates (cosine).
    Dedup(DedupArgs),
    MaintenanceReport(MaintenanceReportArgs),
    Maintain(MaintainArgs),
    Policy(PolicyArgs),
    SkillPolicy(PolicyArgs),
    Compact(CompactArgs),
    Obsidian(ObsidianArgs),
    Ui(UiArgs),
    /// Run/install memd hook helpers for context, capture, gate, and repair flows.
    #[command(visible_alias = "hooks")]
    Hook(HookArgs),
    /// Initialize memd in a project; equivalent onboarding entrypoint to setup.
    Init(InitArgs),
    Loops(LoopsArgs),
    Telemetry(TelemetryArgs),
    Compiler(CompilerArgs),
    Autoresearch(AutoresearchArgs),
    Diagnostics(DiagnosticsArgs),
    #[command(name = "prime-reads")]
    PrimeReads(PrimeReadsArgs),
    /// Live memory contract (A3-D5): shape, verify, generate default.
    Contract(ContractArgs),
    /// Phase C4 correction lane: detect, capture, list.
    Correction(CorrectionArgs),
    /// Phase F4 preference lane: list, drift, confirm, promote.
    Preference(PreferenceArgs),
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
    pub(crate) region: Option<String>,

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
pub(crate) struct CorrectArgs {
    #[arg(long)]
    pub(crate) id: String,

    #[arg(long)]
    pub(crate) content: String,

    #[arg(long)]
    pub(crate) reason: Option<String>,

    #[arg(long, value_name = "TEXT")]
    pub(crate) tag: Vec<String>,

    #[arg(long)]
    pub(crate) confidence: Option<f32>,
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
    pub(crate) region: Option<String>,

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
    pub(crate) model_tier: Option<String>,

    #[arg(long)]
    pub(crate) include_capabilities: bool,

    #[arg(long)]
    pub(crate) include_access: bool,

    #[arg(long)]
    pub(crate) include_hive: bool,

    #[arg(long)]
    pub(crate) format: Option<String>,

    #[arg(long, default_value = "strict")]
    pub(crate) safety: String,

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
pub(crate) struct FeaturesArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HealthArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct AccessArgs {
    #[command(subcommand)]
    pub(crate) command: AccessSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum AccessSubcommand {
    Status(AccessStatusArgs),
    Route(AccessRouteArgs),
    Sync(AccessSyncArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct AccessStatusArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct AccessRouteArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) resource: Option<String>,

    #[arg(long)]
    pub(crate) purpose: Option<String>,

    #[arg(long)]
    pub(crate) provider: Option<String>,

    #[arg(long)]
    pub(crate) agent: Option<String>,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct AccessSyncArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct SecretsArgs {
    #[command(subcommand)]
    pub(crate) command: SecretsSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum SecretsSubcommand {
    Status(SecretsStatusArgs),
    Providers(SecretsStatusArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct SecretsStatusArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct TokensArgs {
    #[command(subcommand)]
    pub(crate) command: TokensSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum TokensSubcommand {
    Saved(TokensSavedArgs),
    Sync(TokensSavedArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct TokensSavedArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) since: Option<String>,

    #[arg(long)]
    pub(crate) json: bool,
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
    pub(crate) region: Option<String>,

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

    /// Query text for lane-aware scoring (G2.2)
    #[arg(long)]
    pub(crate) query: Option<String>,
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

include!("args_memory.rs");
