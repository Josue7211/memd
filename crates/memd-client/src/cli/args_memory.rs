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

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum RoutinesCommand {
    /// Browse the curated routine library.
    Browse(RoutinesBrowseArgs),
    /// Edit a routine into a new active revision.
    Edit(RoutinesEditArgs),
    /// Merge duplicate routines into one active routine.
    Merge(RoutinesMergeArgs),
    /// Compose two routines into one active routine.
    Compose(RoutinesComposeArgs),
    /// Deprecate a routine.
    Deprecate(RoutinesDeprecateArgs),
    /// Export a workspace routine library.
    Export(RoutinesExportArgs),
    /// Import a workspace routine library.
    Import(RoutinesImportArgs),
    /// Search, browse, or install marketplace routines.
    Marketplace(RoutinesMarketplaceArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct RoutinesArgs {
    #[command(subcommand)]
    pub(crate) command: RoutinesCommand,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct RoutinesBrowseArgs {
    #[arg(long)]
    pub(crate) status: Option<String>,

    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long, default_value_t = false)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct RoutinesEditArgs {
    pub(crate) id: String,

    #[arg(long)]
    pub(crate) name: String,

    #[arg(long)]
    pub(crate) summary: String,

    #[arg(long = "steps-file")]
    pub(crate) steps_file: PathBuf,

    #[arg(long, default_value = "codex")]
    pub(crate) updated_by: String,

    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long, default_value_t = false)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct RoutinesMergeArgs {
    pub(crate) ids: Vec<String>,

    #[arg(long)]
    pub(crate) name: String,

    #[arg(long)]
    pub(crate) summary: String,

    #[arg(long, default_value = "codex")]
    pub(crate) updated_by: String,

    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long, default_value_t = false)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct RoutinesComposeArgs {
    pub(crate) left: String,
    pub(crate) right: String,

    #[arg(long)]
    pub(crate) name: String,

    #[arg(long)]
    pub(crate) summary: String,

    #[arg(long, default_value = "codex")]
    pub(crate) updated_by: String,

    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long, default_value_t = false)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct RoutinesDeprecateArgs {
    pub(crate) id: String,

    #[arg(long)]
    pub(crate) reason: String,

    #[arg(long, default_value = "codex")]
    pub(crate) updated_by: String,

    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long, default_value_t = false)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct RoutinesExportArgs {
    #[arg(long)]
    pub(crate) file: PathBuf,

    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct RoutinesImportArgs {
    #[arg(long = "from")]
    pub(crate) from: PathBuf,

    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum RoutinesMarketplaceCommand {
    /// Search marketplace routines.
    Search(RoutinesMarketplaceSearchArgs),
    /// Browse marketplace routines.
    Browse(RoutinesMarketplaceBrowseArgs),
    /// Install a marketplace routine into the local library.
    Install(RoutinesMarketplaceInstallArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct RoutinesMarketplaceArgs {
    #[command(subcommand)]
    pub(crate) command: RoutinesMarketplaceCommand,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct RoutinesMarketplaceSearchArgs {
    pub(crate) query: String,

    #[arg(long, default_value_t = false)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct RoutinesMarketplaceBrowseArgs {
    #[arg(long, default_value_t = false)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct RoutinesMarketplaceInstallArgs {
    pub(crate) name: String,

    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long, default_value_t = false)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum AuditCommand {
    /// Browse signed audit entries from an export.
    Browse(AuditBrowseArgs),
    /// Explain an item chain from an export.
    Explain(AuditExplainArgs),
    /// Verify a signed audit export.
    Verify(AuditVerifyArgs),
    /// Verify a V19 correction-applied proof.
    VerifyZk(AuditVerifyZkArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct AuditArgs {
    #[command(subcommand)]
    pub(crate) command: AuditCommand,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct AuditBrowseArgs {
    #[arg(long)]
    pub(crate) export: PathBuf,

    #[arg(long)]
    pub(crate) since: Option<String>,

    #[arg(long, default_value_t = false)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct AuditExplainArgs {
    pub(crate) item_id: String,

    #[arg(long)]
    pub(crate) export: PathBuf,

    #[arg(long, default_value_t = false)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct AuditVerifyArgs {
    #[arg(long)]
    pub(crate) export: PathBuf,

    #[arg(long, default_value_t = false)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct AuditVerifyZkArgs {
    pub(crate) proof: PathBuf,

    #[arg(long, default_value_t = false)]
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
pub(crate) struct DedupArgs {
    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    /// Cosine distance threshold (default: 0.15).
    #[arg(long)]
    pub(crate) threshold: Option<f32>,

    /// Max clusters to emit.
    #[arg(long, default_value_t = 50)]
    pub(crate) limit: usize,

    /// Preview only. Future: set false to apply merges.
    #[arg(long, default_value_t = true)]
    pub(crate) dry_run: bool,

    /// Emit JSON.
    #[arg(long, default_value_t = false)]
    pub(crate) json: bool,
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

    #[arg(long)]
    pub(crate) trace: bool,
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
    pub(crate) region: Option<String>,

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

    /// E4: select recall depth — `wake` for compiled overview,
    /// `lookup` (default) for 1–3 targeted records, `resume` for full
    /// task-state reconstruction.
    #[arg(long, value_enum, default_value_t = crate::runtime::recall::RecallDepth::Lookup)]
    pub(crate) depth: crate::runtime::recall::RecallDepth,

    /// E4: print the chosen depth + rationale alongside the result.
    #[arg(long)]
    pub(crate) explain_depth: bool,

    /// F5: emit routed_kinds and router_rationale in JSON output.
    #[arg(long)]
    pub(crate) explain_route: bool,
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
pub(crate) struct IngestSourcesArgs {
    /// Directory containing markdown files to ingest (e.g. .memd/lanes/architecture)
    #[arg(long)]
    pub(crate) dir: PathBuf,

    /// Lane tag applied to all ingested items (e.g. "architecture", "inspiration")
    #[arg(long)]
    pub(crate) lane: String,

    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    /// Memory kind for ingested items (default: fact)
    #[arg(long, default_value = "fact")]
    pub(crate) kind: String,

    /// Memory scope for ingested items (default: project)
    #[arg(long, default_value = "project")]
    pub(crate) scope: String,

    /// Extra tags applied to all items
    #[arg(long, value_name = "TEXT")]
    pub(crate) tag: Vec<String>,

    /// Actually write to the DB (dry-run without this)
    #[arg(long)]
    pub(crate) apply: bool,
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

    /// Print the beginner guided setup path and exact proof commands.
    #[arg(long, default_value_t = false)]
    pub(crate) guided: bool,

    #[arg(long, default_value_t = false)]
    pub(crate) allow_localhost_read_only_fallback: bool,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(long)]
    pub(crate) json: bool,
}


#[derive(Debug, Clone, Args)]
pub(crate) struct SetupDemoArgs {
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
pub(crate) struct DeviceArgs {
    #[command(subcommand)]
    pub(crate) command: DeviceCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum DeviceCommand {
    /// Register this machine as a dogfood/evidence device.
    Add(DeviceAddArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct DeviceAddArgs {
    #[arg(long)]
    pub(crate) output: Option<PathBuf>,

    #[arg(long)]
    pub(crate) name: Option<String>,

    #[arg(long)]
    pub(crate) user: Option<String>,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct DogfoodArgs {
    #[command(subcommand)]
    pub(crate) command: DogfoodCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum DogfoodCommand {
    /// Enroll a real dogfood user/harness/device lane.
    Enroll(DogfoodEnrollArgs),
    /// Show local dogfood enrollment state.
    Status(DogfoodStatusArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct DogfoodEnrollArgs {
    #[arg(long)]
    pub(crate) output: Option<PathBuf>,

    #[arg(long)]
    pub(crate) user_id: Option<String>,

    #[arg(long)]
    pub(crate) device_id: Option<String>,

    #[arg(long, value_name = "HARNESS")]
    pub(crate) harness: Vec<String>,

    #[arg(long)]
    pub(crate) consent: bool,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct DogfoodStatusArgs {
    #[arg(long)]
    pub(crate) output: Option<PathBuf>,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ConfigArgs {
    #[command(subcommand)]
    pub(crate) command: Option<ConfigCommand>,

    #[arg(value_name = "KEY=VALUE")]
    pub(crate) set: Vec<String>,

    #[arg(long)]
    pub(crate) output: Option<PathBuf>,

    #[arg(long)]
    pub(crate) project_root: Option<PathBuf>,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum ConfigCommand {
    /// List canonical runtime settings.
    List(ConfigListArgs),
    /// Print one setting.
    Get(ConfigKeyArgs),
    /// Set one setting as KEY=VALUE.
    Set(ConfigSetArgs),
    /// Reset one setting, or all V8 settings when no key is passed.
    Reset(ConfigResetArgs),
    /// Emit the canonical settings JSON schema.
    #[command(name = "show-schema")]
    ShowSchema(ConfigSchemaArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ConfigListArgs {
    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ConfigKeyArgs {
    pub(crate) key: String,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ConfigSetArgs {
    pub(crate) setting: String,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ConfigResetArgs {
    pub(crate) key: Option<String>,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ConfigSchemaArgs {
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
    FileInteraction(HookFileInteractionArgs),
    SealLedger(HookSealLedgerArgs),
    /// A4: PostCompact restore — copy the newest sealed ledger back to the
    /// active `ledger_path` and append an ndjson restore record.
    Restore(HookRestoreArgs),
    Gate(HookGateArgs),
    /// A3 Part 3: verify `.memd/hooks/MANIFEST.json` against on-disk hooks.
    Doctor(HookDoctorArgs),
    /// B4: wrap an inner hook command with fire-order, budget, and trace
    /// enforcement per `docs/contracts/hook-order.md`.
    Enforce(HookEnforceArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HookEnforceArgs {
    /// Event token from `docs/contracts/hook-order.md §1`. Rejected if
    /// not a known token (exit 3).
    #[arg(long)]
    pub(crate) event: String,

    /// Harness label recorded in the trace line (`claude-code`, `codex`, …).
    #[arg(long, default_value = "unknown")]
    pub(crate) harness: String,

    /// Session id recorded in every trace line + used for per-session
    /// serialization. Required.
    #[arg(long)]
    pub(crate) session_id: String,

    /// Override the per-event default budget from contract §2.
    #[arg(long)]
    pub(crate) budget_ms: Option<u64>,

    /// Override the per-event default failure class.
    #[arg(long, value_enum)]
    pub(crate) failure_class: Option<HookFailureClassArg>,

    /// Override the trace file path (defaults to
    /// `<bundle>/logs/hook-trace.ndjson` or `MEMD_HOOK_TRACE_PATH`).
    #[arg(long)]
    pub(crate) trace: Option<PathBuf>,

    /// Bundle root — defaults to `.memd`.
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    /// Optional `tool` field recorded on the trace line.
    #[arg(long)]
    pub(crate) tool: Option<String>,

    /// Optional `path` field recorded on the trace line.
    #[arg(long)]
    pub(crate) path: Option<String>,

    /// Trailing args after `--` are the inner command. When empty, the
    /// enforcer emits a trace line for the event itself and exits 0.
    #[arg(last = true)]
    pub(crate) inner: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub(crate) enum HookFailureClassArg {
    Halt,
    Log,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HookDoctorArgs {
    /// Project root that contains `.memd/hooks/`; defaults to current working dir.
    #[arg(long)]
    pub(crate) project_root: Option<PathBuf>,

    /// Emit JSON instead of human-readable text.
    #[arg(long, default_value_t = false)]
    pub(crate) json: bool,

    /// A4.5: run a dedicated check instead of the default manifest verify.
    /// Currently supports `ordering` (PreCompact → PostCompact → tool-use
    /// trace audit). When absent, the legacy manifest check runs.
    #[arg(long, value_enum)]
    pub(crate) check: Option<HookDoctorCheck>,

    /// Path to the hook trace NDJSON file. Defaults to
    /// `<output>/logs/hook-trace.ndjson` (written by B4).
    #[arg(long)]
    pub(crate) trace: Option<PathBuf>,

    /// Inline trace payload used in tests and dry-runs when no trace file
    /// is available. Accepts either NDJSON lines or a JSON array of events.
    #[arg(long)]
    pub(crate) trace_inline: Option<String>,

    /// Bundle root — used to resolve default trace path and to read the
    /// breach log. Defaults to `.memd`.
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub(crate) enum HookDoctorCheck {
    /// Audit PreCompact → PostCompact → tool-use fire order against
    /// `docs/contracts/hook-handoff.md`.
    Ordering,
    /// B4: audit hook trace against `docs/contracts/hook-order.md` —
    /// event-token validity, budget overruns, silent swallows, and
    /// manifest completeness.
    Contract,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HookFileInteractionArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) session_id: Option<String>,

    #[arg(long)]
    pub(crate) stdin: bool,

    #[arg(long)]
    pub(crate) content: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HookSealLedgerArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) session_id: String,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HookRestoreArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) session_id: String,

    /// Restore only the newest sealed ledger. Currently the only supported
    /// mode; flag reserved for future multi-ledger strategies. Default: true.
    #[arg(long)]
    pub(crate) latest_only: Option<bool>,

    /// Print what would be restored without writing to disk or emitting
    /// telemetry.
    #[arg(long, default_value_t = false)]
    pub(crate) dry_run: bool,

    /// Emit the `LedgerRestoreReport` as JSON on stdout.
    #[arg(long, default_value_t = false)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HookGateArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    /// Override session id; otherwise read from hook payload.
    #[arg(long)]
    pub(crate) session_id: Option<String>,

    /// Policy override; otherwise read from .memd/config.json.
    #[arg(long)]
    pub(crate) policy: Option<String>,

    #[arg(long)]
    pub(crate) stdin: bool,

    #[arg(long)]
    pub(crate) content: Option<String>,
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

    /// C4: route as `correction` kind, append to corrections.ndjson, set provenance.
    #[arg(long, value_name = "KIND")]
    pub(crate) kind: Option<String>,

    /// C4: id of the prior memory record this correction supersedes.
    #[arg(long = "corrects-id")]
    pub(crate) corrects_id: Option<String>,

    /// C4: source-turn id (e.g. "t-12") for provenance trail.
    #[arg(long = "source-turn")]
    pub(crate) source_turn: Option<String>,
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
    #[command(subcommand)]
    pub(crate) command: Option<TelemetryCommand>,

    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long, help = "Emit telemetry JSON instead of text")]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum TelemetryCommand {
    /// Opt in to local-first telemetry.
    Enable,
    /// Opt out and stop writing telemetry events.
    Disable,
    /// Show telemetry config and local event count.
    Status,
    /// Append one telemetry event. Used by harnesses and proof scripts.
    Record(TelemetryRecordArgs),
    /// Print per-user per-harness token cost breakdown.
    Report(TelemetryReportArgs),
    /// Export anonymized telemetry NDJSON.
    Export(TelemetryExportArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct TelemetryRecordArgs {
    #[arg(long)]
    pub(crate) user: Option<String>,

    #[arg(long)]
    pub(crate) harness: Option<String>,

    #[arg(long, default_value = "manual")]
    pub(crate) source: String,

    #[arg(long, default_value = "usage")]
    pub(crate) event_kind: String,

    #[arg(long, default_value_t = 0)]
    pub(crate) tokens: u64,

    #[arg(long, default_value_t = 0.0)]
    pub(crate) cost_usd: f64,

    #[arg(long)]
    pub(crate) session_id: Option<String>,

    #[arg(long)]
    pub(crate) model_family: Option<String>,

    #[arg(long)]
    pub(crate) metadata_json: Option<String>,

    #[arg(long, default_value_t = false)]
    pub(crate) force: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct TelemetryReportArgs {
    #[arg(long, default_value = "30d")]
    pub(crate) window: String,

    #[arg(long, default_value_t = false)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct TelemetryExportArgs {
    #[arg(long)]
    pub(crate) output_file: Option<PathBuf>,

    #[arg(long, default_value = "local")]
    pub(crate) scope: String,

    #[arg(long, default_value = "30d")]
    pub(crate) window: String,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct CompilerArgs {
    #[command(subcommand)]
    pub(crate) command: Option<CompilerCommand>,

    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long, help = "Emit compiler JSON instead of text")]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum CompilerCommand {
    /// Build per-user per-harness self-tuning profiles from V14 telemetry.
    Tune(CompilerTuneArgs),
    /// Print persisted self-tuning profiles.
    Profiles(CompilerProfilesArgs),
    /// Compare static, dynamic, and self-tuning budgets.
    #[command(name = "ab-bench")]
    AbBench(CompilerAbBenchArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct CompilerTuneArgs {
    #[arg(long, default_value_t = 1500)]
    pub(crate) baseline_budget: u64,

    #[arg(long, default_value_t = 3)]
    pub(crate) min_samples: usize,

    #[arg(long, default_value_t = 0.90)]
    pub(crate) min_quality_score: f64,

    #[arg(long, default_value_t = 1.10)]
    pub(crate) tuning_headroom: f64,

    #[arg(long, default_value_t = false)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct CompilerProfilesArgs {
    #[arg(long, default_value_t = false)]
    pub(crate) accepted_only: bool,

    #[arg(long, default_value_t = false)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct CompilerAbBenchArgs {
    #[arg(long, default_value_t = 4000)]
    pub(crate) static_budget: u64,

    #[arg(long, default_value_t = 1500)]
    pub(crate) dynamic_budget: u64,

    #[arg(long, default_value_t = false)]
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
pub(crate) struct StateArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct LiveStateArgs {
    #[command(subcommand)]
    pub(crate) command: LiveStateSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum LiveStateSubcommand {
    Ingest(LiveStateIngestArgs),
    IngestBatch(LiveStateIngestBatchArgs),
    Import(LiveStateImportArgs),
    Sync(LiveStateSyncArgs),
    Status(LiveStateStatusArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct LiveStateIngestArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) source: String,

    #[arg(long)]
    pub(crate) module: String,

    #[arg(long, default_value = "default")]
    pub(crate) scope: String,

    #[arg(long, default_value = "private")]
    pub(crate) visibility: String,

    #[arg(long, default_value = "metadata")]
    pub(crate) privacy: String,

    #[arg(long)]
    pub(crate) approved: bool,

    #[arg(long = "agentsecrets-approved")]
    pub(crate) agentsecrets_approved: bool,

    #[arg(long, default_value_t = 86_400)]
    pub(crate) freshness_secs: i64,

    #[arg(long = "label")]
    pub(crate) label: Vec<String>,

    #[arg(long)]
    pub(crate) summary: String,

    #[arg(long)]
    pub(crate) payload_json: Option<String>,

    #[arg(long)]
    pub(crate) payload_file: Option<PathBuf>,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct LiveStateIngestBatchArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    /// Read a ClawControl-style {"records":[...]} live-state batch from stdin.
    #[arg(long)]
    pub(crate) stdin: bool,

    /// ClawControl-style {"records":[...]} live-state batch JSON.
    #[arg(long)]
    pub(crate) input_json: Option<String>,

    /// File containing a ClawControl-style {"records":[...]} live-state batch.
    #[arg(long)]
    pub(crate) input_file: Option<PathBuf>,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct LiveStateImportArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    /// Import records from another .memd output directory.
    #[arg(long)]
    pub(crate) from_output: PathBuf,

    /// Only import records from this source app.
    #[arg(long)]
    pub(crate) source: Option<String>,

    /// Skip records that are already expired in the source map.
    #[arg(long)]
    pub(crate) fresh_only: bool,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct LiveStateSyncArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    /// Import from this .memd output directory when the authority map is missing, stale, or due.
    #[arg(long)]
    pub(crate) from_output: PathBuf,

    /// Only sync records from this source app. Use "all" for a composite authority import.
    #[arg(long, default_value = "memd")]
    pub(crate) source: String,

    /// Also sync when next_refresh_at is within this many seconds.
    #[arg(long, default_value_t = 0)]
    pub(crate) due_within_secs: i64,

    /// Allow importing stale source records. Fresh-only is the default.
    #[arg(long)]
    pub(crate) allow_stale: bool,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct LiveStateStatusArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) json: bool,

    #[arg(long)]
    pub(crate) summary: bool,

    /// Print one producer task per line for shell schedulers.
    #[arg(long)]
    pub(crate) tasks: bool,

    /// Print shell command templates for each pending producer task.
    #[arg(long)]
    pub(crate) commands: bool,

    /// Print a ClawControl-style {"records":[...]} producer batch template.
    #[arg(long)]
    pub(crate) batch_template: bool,

    /// Exit non-zero when a producer sync is required.
    #[arg(long)]
    pub(crate) check: bool,

    /// With --check, also exit non-zero when next_refresh_at is within this many seconds.
    #[arg(long, default_value_t = 0)]
    pub(crate) due_within_secs: i64,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct CapabilitiesArgs {
    #[command(subcommand)]
    pub(crate) command: Option<CapabilitiesSubcommand>,

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

    #[arg(long)]
    pub(crate) materialize_plan: bool,

    #[arg(long)]
    pub(crate) materialize: bool,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum CapabilitiesSubcommand {
    Pull(CapabilitiesPullArgs),
    Status(CapabilitiesStatusArgs),
    Sync(CapabilitiesSyncArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct CapabilitiesPullArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) json: bool,

    #[arg(long)]
    pub(crate) materialize_plan: bool,

    #[arg(long)]
    pub(crate) materialize: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct CapabilitiesStatusArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct CapabilitiesSyncArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

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
pub(crate) struct DevServerArgs {
    #[command(subcommand)]
    pub(crate) command: DevServerSubcommand,
}
