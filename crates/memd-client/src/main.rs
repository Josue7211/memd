mod obsidian;
mod commands;
mod render;

use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use memd_client::MemdClient;
use memd_core::{
    build_compaction_packet, derive_compaction_spill, derive_compaction_spill_with_options,
    render_compaction_wire,
};
use memd_multimodal::{
    MultimodalChunk, MultimodalIngestPlan, build_ingest_plan, extract_chunks, to_sidecar_requests,
};
use memd_rag::{RagClient, RagIngestRequest, RagRetrieveMode, RagRetrieveRequest};
use memd_schema::{
    AgentProfileRequest, AgentProfileUpsertRequest, AssociativeRecallRequest, CandidateMemoryRequest,
    CompactionDecision, CompactionOpenLoop, CompactionPacket, CompactionReference,
    CompactionSession, CompactionSpillOptions, CompactionSpillResult, ContextRequest,
    EntityLinkRequest, EntityLinksRequest, ExpireMemoryRequest, ExplainMemoryRequest,
    EntitySearchRequest, MemoryConsolidationRequest, MemoryInboxRequest, MemoryKind,
    MemoryMaintenanceReportRequest, MemoryScope, MemoryStage, MemoryStatus, PromoteMemoryRequest,
    RepairMemoryRequest, RetrievalIntent, RetrievalRoute, SearchMemoryRequest, SourceMemoryRequest,
    StoreMemoryRequest, VerifyMemoryRequest, WorkingMemoryRequest,
};
use memd_sidecar::{SidecarClient, SidecarIngestRequest, SidecarIngestResponse};
use notify::{Config as NotifyConfig, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use obsidian::{ObsidianImportPreview, ObsidianSyncEntry};
use commands::{
    parse_entity_relation_kind, parse_memory_kind_value, parse_memory_scope_value,
    parse_retrieval_intent, parse_retrieval_route, parse_source_quality_value, parse_uuid_list,
};
use render::{
    render_consolidate_summary, render_entity_search_summary, render_entity_summary,
    render_explain_summary, render_maintenance_report_summary, render_obsidian_import_summary,
    render_obsidian_scan_summary, render_profile_summary, render_recall_summary,
    render_repair_summary, render_source_summary, render_timeline_summary, render_working_summary,
    short_uuid,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser)]
#[command(name = "memd")]
#[command(about = "Compact CLI for memd")]
struct Cli {
    #[arg(long, default_value = "http://127.0.0.1:8787")]
    base_url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Healthz,
    Status(StatusArgs),
    Attach(AttachArgs),
    Rag(RagArgs),
    Multimodal(MultimodalArgs),
    Ingest(IngestArgs),
    Store(RequestInput),
    Candidate(RequestInput),
    Promote(RequestInput),
    Expire(RequestInput),
    Verify(RequestInput),
    Repair(RepairArgs),
    Search(SearchArgs),
    Context(ContextArgs),
    Working(WorkingArgs),
    Profile(ProfileArgs),
    Source(SourceArgs),
    Inbox(InboxArgs),
    Explain(ExplainArgs),
    Entity(EntityArgs),
    EntitySearch(EntitySearchArgs),
    EntityLink(EntityLinkArgs),
    EntityLinks(EntityLinksArgs),
    Recall(RecallArgs),
    Timeline(TimelineArgs),
    Consolidate(ConsolidateArgs),
    MaintenanceReport(MaintenanceReportArgs),
    Policy,
    Compact(CompactArgs),
    Obsidian(ObsidianArgs),
    Hook(HookArgs),
    Init(InitArgs),
}

#[derive(Debug, Clone, Args)]
struct RequestInput {
    #[arg(long)]
    json: Option<String>,

    #[arg(long)]
    input: Option<PathBuf>,

    #[arg(long)]
    stdin: bool,
}

#[derive(Debug, Clone, Args)]
struct RepairArgs {
    #[arg(long)]
    id: String,

    #[arg(long)]
    mode: String,

    #[arg(long)]
    confidence: Option<f32>,

    #[arg(long)]
    status: Option<String>,

    #[arg(long)]
    source_agent: Option<String>,

    #[arg(long)]
    source_system: Option<String>,

    #[arg(long)]
    source_path: Option<String>,

    #[arg(long)]
    source_quality: Option<String>,

    #[arg(long)]
    content: Option<String>,

    #[arg(long, value_name = "TEXT")]
    tag: Vec<String>,

    #[arg(long, value_name = "UUID")]
    supersede: Vec<String>,

    #[arg(long)]
    summary: bool,

    #[arg(long)]
    follow: bool,
}

#[derive(Debug, Clone, Args)]
struct ContextArgs {
    #[arg(long)]
    project: Option<String>,

    #[arg(long)]
    agent: Option<String>,

    #[arg(long)]
    limit: Option<usize>,

    #[arg(long)]
    max_chars_per_item: Option<usize>,

    #[arg(long)]
    route: Option<String>,

    #[arg(long)]
    intent: Option<String>,

    #[arg(long)]
    compact: bool,

    #[arg(long)]
    json: Option<String>,

    #[arg(long)]
    input: Option<PathBuf>,

    #[arg(long)]
    stdin: bool,
}

#[derive(Debug, Clone, Args)]
struct WorkingArgs {
    #[arg(long)]
    project: Option<String>,

    #[arg(long)]
    agent: Option<String>,

    #[arg(long)]
    limit: Option<usize>,

    #[arg(long)]
    max_chars_per_item: Option<usize>,

    #[arg(long)]
    max_total_chars: Option<usize>,

    #[arg(long)]
    rehydration_limit: Option<usize>,

    #[arg(long)]
    route: Option<String>,

    #[arg(long)]
    intent: Option<String>,

    #[arg(long)]
    summary: bool,

    #[arg(long)]
    follow: bool,

    #[arg(long)]
    auto_consolidate: bool,
}

#[derive(Debug, Clone, Args)]
struct ProfileArgs {
    #[arg(long)]
    agent: String,

    #[arg(long)]
    project: Option<String>,

    #[arg(long)]
    namespace: Option<String>,

    #[arg(long)]
    set: bool,

    #[arg(long)]
    preferred_route: Option<String>,

    #[arg(long)]
    preferred_intent: Option<String>,

    #[arg(long)]
    summary_chars: Option<usize>,

    #[arg(long)]
    max_total_chars: Option<usize>,

    #[arg(long)]
    recall_depth: Option<usize>,

    #[arg(long)]
    source_trust_floor: Option<f32>,

    #[arg(long, value_name = "TEXT")]
    style_tag: Vec<String>,

    #[arg(long)]
    notes: Option<String>,

    #[arg(long)]
    summary: bool,

    #[arg(long)]
    follow: bool,
}

#[derive(Debug, Clone, Args)]
struct SourceArgs {
    #[arg(long)]
    project: Option<String>,

    #[arg(long)]
    namespace: Option<String>,

    #[arg(long)]
    source_agent: Option<String>,

    #[arg(long)]
    source_system: Option<String>,

    #[arg(long)]
    limit: Option<usize>,

    #[arg(long)]
    summary: bool,

    #[arg(long)]
    follow: bool,
}

#[derive(Debug, Clone, Args)]
struct InboxArgs {
    #[arg(long)]
    project: Option<String>,

    #[arg(long)]
    namespace: Option<String>,

    #[arg(long)]
    belief_branch: Option<String>,

    #[arg(long)]
    route: Option<String>,

    #[arg(long)]
    intent: Option<String>,

    #[arg(long)]
    limit: Option<usize>,
}

#[derive(Debug, Clone, Args)]
struct ExplainArgs {
    #[arg(long)]
    id: String,

    #[arg(long)]
    belief_branch: Option<String>,

    #[arg(long)]
    route: Option<String>,

    #[arg(long)]
    intent: Option<String>,

    #[arg(long)]
    summary: bool,

    #[arg(long)]
    follow: bool,
}

#[derive(Debug, Clone, Args)]
struct EntityArgs {
    #[arg(long)]
    id: String,

    #[arg(long)]
    route: Option<String>,

    #[arg(long)]
    intent: Option<String>,

    #[arg(long)]
    limit: Option<usize>,

    #[arg(long)]
    summary: bool,

    #[arg(long)]
    follow: bool,
}

#[derive(Debug, Clone, Args)]
struct EntitySearchArgs {
    #[arg(long)]
    query: String,

    #[arg(long)]
    project: Option<String>,

    #[arg(long)]
    namespace: Option<String>,

    #[arg(long)]
    at: Option<String>,

    #[arg(long)]
    host: Option<String>,

    #[arg(long)]
    branch: Option<String>,

    #[arg(long)]
    location: Option<String>,

    #[arg(long)]
    route: Option<String>,

    #[arg(long)]
    intent: Option<String>,

    #[arg(long)]
    limit: Option<usize>,

    #[arg(long)]
    summary: bool,

    #[arg(long)]
    follow: bool,
}

#[derive(Debug, Clone, Args)]
struct EntityLinkArgs {
    #[arg(long)]
    from_entity_id: String,

    #[arg(long)]
    to_entity_id: String,

    #[arg(long)]
    relation_kind: String,

    #[arg(long)]
    confidence: Option<f32>,

    #[arg(long)]
    note: Option<String>,
}

#[derive(Debug, Clone, Args)]
struct EntityLinksArgs {
    #[arg(long)]
    entity_id: String,
}

#[derive(Debug, Clone, Args)]
struct RecallArgs {
    #[arg(long)]
    entity_id: Option<String>,

    #[arg(long)]
    query: Option<String>,

    #[arg(long)]
    project: Option<String>,

    #[arg(long)]
    namespace: Option<String>,

    #[arg(long)]
    at: Option<String>,

    #[arg(long)]
    host: Option<String>,

    #[arg(long)]
    branch: Option<String>,

    #[arg(long)]
    location: Option<String>,

    #[arg(long)]
    depth: Option<usize>,

    #[arg(long)]
    limit: Option<usize>,

    #[arg(long)]
    summary: bool,

    #[arg(long)]
    follow: bool,
}

#[derive(Debug, Clone, Args)]
struct TimelineArgs {
    #[arg(long)]
    id: String,

    #[arg(long)]
    route: Option<String>,

    #[arg(long)]
    intent: Option<String>,

    #[arg(long)]
    limit: Option<usize>,

    #[arg(long)]
    summary: bool,

    #[arg(long)]
    follow: bool,
}

#[derive(Debug, Clone, Args)]
struct ConsolidateArgs {
    #[arg(long)]
    project: Option<String>,

    #[arg(long)]
    namespace: Option<String>,

    #[arg(long)]
    max_groups: Option<usize>,

    #[arg(long)]
    min_events: Option<usize>,

    #[arg(long)]
    lookback_days: Option<i64>,

    #[arg(long)]
    min_salience: Option<f32>,

    #[arg(long, default_value_t = true)]
    record_events: bool,

    #[arg(long)]
    summary: bool,

    #[arg(long)]
    follow: bool,
}

#[derive(Debug, Clone, Args)]
struct MaintenanceReportArgs {
    #[arg(long)]
    project: Option<String>,

    #[arg(long)]
    namespace: Option<String>,

    #[arg(long)]
    inactive_days: Option<i64>,

    #[arg(long)]
    lookback_days: Option<i64>,

    #[arg(long)]
    min_events: Option<usize>,

    #[arg(long)]
    max_decay: Option<f32>,

    #[arg(long)]
    summary: bool,

    #[arg(long)]
    follow: bool,
}

#[derive(Debug, Clone, Args)]
struct ObsidianArgs {
    #[arg(long)]
    vault: PathBuf,

    #[arg(long)]
    project: Option<String>,

    #[arg(long)]
    namespace: Option<String>,

    #[arg(long)]
    max_notes: Option<usize>,

    #[arg(long)]
    max_attachments: Option<usize>,

    #[arg(long)]
    state_file: Option<PathBuf>,

    #[arg(long)]
    include_folder: Vec<String>,

    #[arg(long)]
    exclude_folder: Vec<String>,

    #[arg(long)]
    include_tag: Vec<String>,

    #[arg(long)]
    exclude_tag: Vec<String>,

    #[arg(long)]
    summary: bool,

    #[arg(long)]
    follow: bool,

    #[arg(long)]
    review_sensitive: bool,

    #[arg(long)]
    include_attachments: bool,

    #[arg(long)]
    apply: bool,

    #[arg(long)]
    link_notes: bool,

    #[arg(long)]
    output: Option<PathBuf>,

    #[arg(long)]
    overwrite: bool,

    #[arg(long)]
    open: bool,

    #[arg(long)]
    pane_type: Option<String>,

    #[arg(long)]
    note: Option<PathBuf>,

    #[arg(long)]
    query: Option<String>,

    #[arg(long)]
    route: Option<String>,

    #[arg(long)]
    intent: Option<String>,

    #[arg(long)]
    limit: Option<usize>,

    #[arg(long)]
    id: Option<String>,

    #[arg(long, default_value_t = 750)]
    debounce_ms: u64,

    #[command(subcommand)]
    mode: ObsidianMode,
}

#[derive(Debug, Clone, Subcommand)]
enum ObsidianMode {
    Scan,
    Import,
    Sync,
    Compile,
    Open,
    Writeback,
    Roundtrip,
    Watch,
    Status,
}

#[derive(Debug, Clone, Args)]
struct SearchArgs {
    #[command(flatten)]
    input: RequestInput,

    #[arg(long)]
    belief_branch: Option<String>,

    #[arg(long)]
    route: Option<String>,

    #[arg(long)]
    intent: Option<String>,
}

#[derive(Debug, Clone, Args)]
struct IngestArgs {
    #[arg(long)]
    project: Option<String>,

    #[arg(long)]
    namespace: Option<String>,

    #[arg(long)]
    kind: Option<String>,

    #[arg(long)]
    scope: Option<String>,

    #[arg(long)]
    source_agent: Option<String>,

    #[arg(long)]
    source_system: Option<String>,

    #[arg(long)]
    source_path: Option<String>,

    #[arg(long)]
    source_quality: Option<String>,

    #[arg(long)]
    confidence: Option<f32>,

    #[arg(long)]
    ttl_seconds: Option<u64>,

    #[arg(long, value_name = "TEXT")]
    tag: Vec<String>,

    #[arg(long, value_name = "TEXT")]
    supersede: Vec<String>,

    #[arg(long)]
    content: Option<String>,

    #[arg(long)]
    json: Option<String>,

    #[arg(long)]
    input: Option<PathBuf>,

    #[arg(long)]
    stdin: bool,

    #[arg(long)]
    apply: bool,

    #[arg(long)]
    route: Option<String>,

    #[arg(long)]
    intent: Option<String>,
}

#[derive(Debug, Clone, Args)]
struct CompactArgs {
    #[arg(long)]
    project: Option<String>,

    #[arg(long)]
    agent: Option<String>,

    #[arg(long)]
    task: String,

    #[arg(long)]
    goal: String,

    #[arg(long, value_name = "TEXT")]
    hard_constraint: Vec<String>,

    #[arg(long, value_name = "TEXT")]
    active_work: Vec<String>,

    #[arg(long, value_name = "TEXT")]
    decision: Vec<String>,

    #[arg(long, value_name = "TEXT")]
    open_loop: Vec<String>,

    #[arg(long, value_name = "TEXT")]
    next_action: Vec<String>,

    #[arg(long, value_name = "TEXT")]
    do_not_drop: Vec<String>,

    #[arg(long, value_name = "KIND=VALUE")]
    exact_ref: Vec<String>,

    #[arg(long)]
    limit: Option<usize>,

    #[arg(long)]
    max_chars_per_item: Option<usize>,

    #[arg(long)]
    route: Option<String>,

    #[arg(long)]
    intent: Option<String>,

    #[arg(long)]
    wire: bool,

    #[arg(long)]
    spill: bool,

    #[arg(long)]
    spill_transient: bool,

    #[arg(long)]
    apply: bool,
}

#[derive(Debug, Clone, Args)]
struct HookArgs {
    #[command(subcommand)]
    mode: HookMode,
}

#[derive(Debug, Clone, Subcommand)]
enum HookMode {
    Context(HookContextArgs),
    Spill(HookSpillArgs),
}

#[derive(Debug, Clone, Args)]
struct HookContextArgs {
    #[arg(long)]
    project: Option<String>,

    #[arg(long)]
    agent: Option<String>,

    #[arg(long)]
    limit: Option<usize>,

    #[arg(long)]
    max_chars_per_item: Option<usize>,

    #[arg(long)]
    route: Option<String>,

    #[arg(long)]
    intent: Option<String>,
}

#[derive(Debug, Clone, Args)]
struct HookSpillArgs {
    #[command(flatten)]
    input: RequestInput,

    #[arg(long)]
    apply: bool,

    #[arg(long)]
    spill_transient: bool,
}

#[derive(Debug, Clone, Args)]
struct InitArgs {
    #[arg(long)]
    project: String,

    #[arg(long)]
    agent: String,

    #[arg(long, default_value = ".memd")]
    output: PathBuf,

    #[arg(long, default_value = "http://127.0.0.1:8787")]
    base_url: String,

    #[arg(long)]
    rag_url: Option<String>,

    #[arg(long, default_value = "auto")]
    route: String,

    #[arg(long, default_value = "general")]
    intent: String,

    #[arg(long)]
    force: bool,
}

#[derive(Debug, Clone, Args)]
struct StatusArgs {
    #[arg(long, default_value = ".memd")]
    output: PathBuf,
}

#[derive(Debug, Clone, Args)]
struct AttachArgs {
    #[arg(long, default_value = ".memd")]
    output: PathBuf,

    #[arg(long)]
    shell: Option<String>,
}

#[derive(Debug, Clone, Args)]
struct RagArgs {
    #[arg(long)]
    rag_url: Option<String>,

    #[command(subcommand)]
    mode: RagMode,
}

#[derive(Debug, Clone, Subcommand)]
enum RagMode {
    Healthz,
    Sync(RagSyncArgs),
    Search(RagSearchArgs),
}

#[derive(Debug, Clone, Args)]
struct MultimodalArgs {
    #[arg(long)]
    rag_url: Option<String>,

    #[command(subcommand)]
    mode: MultimodalMode,
}

#[derive(Debug, Clone, Subcommand)]
enum MultimodalMode {
    Healthz,
    Plan(MultimodalPlanArgs),
    Ingest(MultimodalIngestArgs),
    Retrieve(MultimodalRetrieveArgs),
}

#[derive(Debug, Clone, Args)]
struct RagSyncArgs {
    #[arg(long)]
    project: Option<String>,

    #[arg(long)]
    namespace: Option<String>,

    #[arg(long)]
    limit: Option<usize>,

    #[arg(long)]
    dry_run: bool,
}

#[derive(Debug, Clone, Args)]
struct RagSearchArgs {
    #[arg(long)]
    query: String,

    #[arg(long)]
    project: Option<String>,

    #[arg(long)]
    namespace: Option<String>,

    #[arg(long)]
    mode: Option<String>,

    #[arg(long)]
    include_cross_modal: bool,

    #[arg(long)]
    limit: Option<usize>,
}

#[derive(Debug, Clone, Args)]
struct MultimodalPlanArgs {
    #[arg(long)]
    project: Option<String>,

    #[arg(long)]
    namespace: Option<String>,

    #[arg(long, value_name = "PATH")]
    path: Vec<PathBuf>,
}

#[derive(Debug, Clone, Args)]
struct MultimodalIngestArgs {
    #[arg(long)]
    project: Option<String>,

    #[arg(long)]
    namespace: Option<String>,

    #[arg(long, value_name = "PATH")]
    path: Vec<PathBuf>,

    #[arg(long)]
    apply: bool,
}

#[derive(Debug, Clone, Args)]
struct MultimodalRetrieveArgs {
    #[arg(long)]
    query: String,

    #[arg(long)]
    project: Option<String>,

    #[arg(long)]
    namespace: Option<String>,

    #[arg(long)]
    mode: Option<String>,

    #[arg(long)]
    include_cross_modal: bool,

    #[arg(long)]
    limit: Option<usize>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let client = MemdClient::new(&cli.base_url)?;
    let base_url = cli.base_url.clone();

    match cli.command {
        Commands::Healthz => print_json(&client.healthz().await?)?,
        Commands::Status(args) => print_json(&read_bundle_status(&args.output, &base_url).await?)?,
        Commands::Attach(args) => {
            let shell = args
                .shell
                .or_else(|| detect_shell())
                .unwrap_or_else(|| "bash".to_string());
            println!("{}", render_attach_snippet(&shell, &args.output)?);
        }
        Commands::Rag(args) => {
            let rag_url =
                resolve_rag_url(args.rag_url, resolve_default_bundle_root()?.as_deref())?;
            let rag = RagClient::new(&rag_url)?;
            match args.mode {
                RagMode::Healthz => print_json(&rag.healthz().await?)?,
                RagMode::Search(args) => {
                    let mode = args
                        .mode
                        .as_deref()
                        .map(parse_rag_retrieve_mode)
                        .transpose()?
                        .unwrap_or(RagRetrieveMode::Auto);
                    let query = RagRetrieveRequest {
                        query: args.query,
                        project: args.project,
                        namespace: args.namespace,
                        mode,
                        limit: args.limit,
                        include_cross_modal: args.include_cross_modal,
                    };
                    print_json(&rag.retrieve(&query).await?)?;
                }
                RagMode::Sync(args) => {
                    let summary = sync_to_rag(&client, &rag, args).await?;
                    print_json(&summary)?;
                }
            }
        }
        Commands::Multimodal(args) => {
            let rag_url =
                resolve_rag_url(args.rag_url, resolve_default_bundle_root()?.as_deref())?;
            let sidecar = SidecarClient::new(&rag_url)?;
            match args.mode {
                MultimodalMode::Healthz => print_json(&sidecar.healthz().await?)?,
                MultimodalMode::Plan(args) => {
                    let preview =
                        build_multimodal_preview(args.project, args.namespace, &args.path)?;
                    print_json(&preview)?;
                }
                MultimodalMode::Ingest(args) => {
                    let preview =
                        build_multimodal_preview(args.project, args.namespace, &args.path)?;
                    if args.apply {
                        let responses =
                            ingest_multimodal_preview(&sidecar, &preview.requests).await?;
                        let submitted = responses.len();
                        print_json(&MultimodalIngestOutput {
                            preview,
                            responses,
                            submitted,
                            dry_run: false,
                        })?;
                    } else {
                        print_json(&MultimodalIngestOutput {
                            preview,
                            responses: Vec::new(),
                            submitted: 0,
                            dry_run: true,
                        })?;
                    }
                }
                MultimodalMode::Retrieve(args) => {
                    let mut request = memd_multimodal::build_retrieve_request(
                        args.query,
                        args.project,
                        args.namespace,
                        args.limit,
                        args.include_cross_modal,
                    );
                    if let Some(mode) = args
                        .mode
                        .as_deref()
                        .map(parse_rag_retrieve_mode)
                        .transpose()?
                    {
                        request.mode = mode;
                    }
                    print_json(&sidecar.retrieve(&request).await?)?;
                }
            }
        }
        Commands::Ingest(args) => {
            let result = ingest_auto_route(&client, &args).await?;
            print_json(&result)?;
        }
        Commands::Store(input) => {
            let req = read_request::<StoreMemoryRequest>(&input)?;
            print_json(&client.store(&req).await?)?;
        }
        Commands::Candidate(input) => {
            let req = read_request::<CandidateMemoryRequest>(&input)?;
            print_json(&client.candidate(&req).await?)?;
        }
        Commands::Promote(input) => {
            let req = read_request::<PromoteMemoryRequest>(&input)?;
            print_json(&client.promote(&req).await?)?;
        }
        Commands::Expire(input) => {
            let req = read_request::<ExpireMemoryRequest>(&input)?;
            print_json(&client.expire(&req).await?)?;
        }
        Commands::Verify(input) => {
            let req = read_request::<VerifyMemoryRequest>(&input)?;
            print_json(&client.verify(&req).await?)?;
        }
        Commands::Repair(args) => {
            let mode = commands::parse_memory_repair_mode_value(&args.mode)?;
            let status = match args.status.as_deref() {
                Some(value) => Some(commands::parse_memory_status_value(value)?),
                None => None,
            };
            let source_quality = match args.source_quality.as_deref() {
                Some(value) => Some(parse_source_quality_value(value)?),
                None => None,
            };
            let supersedes = parse_uuid_list(&args.supersede)?;
            let response = client
                .repair(&RepairMemoryRequest {
                    id: args.id.parse()?,
                    mode,
                    confidence: args.confidence,
                    status,
                    source_agent: args.source_agent.clone(),
                    source_system: args.source_system.clone(),
                    source_path: args.source_path.clone(),
                    source_quality,
                    content: args.content.clone(),
                    tags: if args.tag.is_empty() {
                        None
                    } else {
                        Some(args.tag.clone())
                    },
                    supersedes,
                })
                .await?;
            if args.summary {
                println!("{}", render_repair_summary(&response, args.follow));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Search(args) => {
            let mut req = read_request::<SearchMemoryRequest>(&args.input)?;
            if args.route.is_some() || args.intent.is_some() {
                req.route = parse_retrieval_route(args.route)?;
                req.intent = parse_retrieval_intent(args.intent)?;
            }
            if args.belief_branch.is_some() {
                req.belief_branch = args.belief_branch.clone();
            }
            print_json(&client.search(&req).await?)?;
        }
        Commands::Context(args) => {
            let req = if args.json.is_some() || args.input.is_some() || args.stdin {
                read_request::<ContextRequest>(&RequestInput {
                    json: args.json.clone(),
                    input: args.input.clone(),
                    stdin: args.stdin,
                })?
            } else {
                ContextRequest {
                    project: args.project.clone(),
                    agent: args.agent.clone(),
                    route: parse_retrieval_route(args.route.clone())?,
                    intent: parse_retrieval_intent(args.intent.clone())?,
                    limit: args.limit,
                    max_chars_per_item: args.max_chars_per_item,
                }
            };

            if args.compact {
                print_json(&client.context_compact(&req).await?)?;
            } else {
                print_json(&client.context(&req).await?)?;
            }
        }
        Commands::Working(args) => {
            let response = client
                .working(&WorkingMemoryRequest {
                    project: args.project.clone(),
                    agent: args.agent.clone(),
                    route: parse_retrieval_route(args.route.clone())?,
                    intent: parse_retrieval_intent(args.intent.clone())?,
                    limit: args.limit,
                    max_chars_per_item: args.max_chars_per_item,
                    max_total_chars: args.max_total_chars,
                    rehydration_limit: args.rehydration_limit,
                    auto_consolidate: Some(args.auto_consolidate),
                })
                .await?;
            if args.summary {
                println!("{}", render_working_summary(&response, args.follow));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Profile(args) => {
            let should_set = args.set
                || args.preferred_route.is_some()
                || args.preferred_intent.is_some()
                || args.summary_chars.is_some()
                || args.max_total_chars.is_some()
                || args.recall_depth.is_some()
                || args.source_trust_floor.is_some()
                || !args.style_tag.is_empty()
                || args.notes.is_some();

            if should_set {
                let response = client
                    .upsert_agent_profile(&AgentProfileUpsertRequest {
                        agent: args.agent.clone(),
                        project: args.project.clone(),
                        namespace: args.namespace.clone(),
                        preferred_route: parse_retrieval_route(args.preferred_route.clone())?,
                        preferred_intent: parse_retrieval_intent(args.preferred_intent.clone())?,
                        summary_chars: args.summary_chars,
                        max_total_chars: args.max_total_chars,
                        recall_depth: args.recall_depth,
                        source_trust_floor: args.source_trust_floor,
                        style_tags: args.style_tag.clone(),
                        notes: args.notes.clone(),
                    })
                    .await?;
                if args.summary {
                    println!("{}", render_profile_summary(&response, args.follow));
                } else {
                    print_json(&response)?;
                }
            } else {
                let response = client
                    .agent_profile(&AgentProfileRequest {
                        agent: args.agent.clone(),
                        project: args.project.clone(),
                        namespace: args.namespace.clone(),
                    })
                    .await?;
                if args.summary {
                    println!("{}", render_profile_summary(&response, args.follow));
                } else {
                    print_json(&response)?;
                }
            }
        }
        Commands::Source(args) => {
            let response = client
                .source_memory(&SourceMemoryRequest {
                    project: args.project.clone(),
                    namespace: args.namespace.clone(),
                    source_agent: args.source_agent.clone(),
                    source_system: args.source_system.clone(),
                    limit: args.limit,
                })
                .await?;
            if args.summary {
                println!("{}", render_source_summary(&response, args.follow));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Inbox(args) => {
            let req = MemoryInboxRequest {
                project: args.project.clone(),
                namespace: args.namespace.clone(),
                belief_branch: args.belief_branch.clone(),
                route: parse_retrieval_route(args.route.clone())?,
                intent: parse_retrieval_intent(args.intent.clone())?,
                limit: args.limit,
            };
            print_json(&client.inbox(&req).await?)?;
        }
        Commands::Explain(args) => {
            let req = ExplainMemoryRequest {
                id: args.id.parse().context("parse memory id as uuid")?,
                belief_branch: args.belief_branch.clone(),
                route: parse_retrieval_route(args.route.clone())?,
                intent: parse_retrieval_intent(args.intent.clone())?,
            };
            let response = client.explain(&req).await?;
            if args.summary {
                println!("{}", render_explain_summary(&response, args.follow));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Entity(args) => {
            let req = memd_schema::EntityMemoryRequest {
                id: args.id.parse().context("parse memory id as uuid")?,
                route: parse_retrieval_route(args.route.clone())?,
                intent: parse_retrieval_intent(args.intent.clone())?,
                limit: args.limit,
            };
            let response = client.entity(&req).await?;
            if args.summary {
                println!("{}", render_entity_summary(&response, args.follow));
            } else {
                print_json(&response)?;
            }
        }
        Commands::EntitySearch(args) => {
            let response = client
                .entity_search(&EntitySearchRequest {
                    query: args.query.clone(),
                    project: args.project.clone(),
                    namespace: args.namespace.clone(),
                    at: parse_context_time(args.at.clone())?,
                    host: args.host.clone(),
                    branch: args.branch.clone(),
                    location: args.location.clone(),
                    route: parse_retrieval_route(args.route.clone())?,
                    intent: parse_retrieval_intent(args.intent.clone())?,
                    limit: args.limit,
                })
                .await?;
            if args.summary {
                println!("{}", render_entity_search_summary(&response, args.follow));
            } else {
                print_json(&response)?;
            }
        }
        Commands::EntityLink(args) => {
            let response = client
                .link_entity(&EntityLinkRequest {
                    from_entity_id: args
                        .from_entity_id
                        .parse()
                        .context("parse from_entity_id as uuid")?,
                    to_entity_id: args
                        .to_entity_id
                        .parse()
                        .context("parse to_entity_id as uuid")?,
                    relation_kind: parse_entity_relation_kind(&args.relation_kind)?,
                    confidence: args.confidence,
                    note: args.note,
                    context: None,
                    tags: Vec::new(),
                })
                .await?;
            print_json(&response)?;
        }
        Commands::EntityLinks(args) => {
            let response = client
                .entity_links(&EntityLinksRequest {
                    entity_id: args.entity_id.parse().context("parse entity_id as uuid")?,
                })
                .await?;
            print_json(&response)?;
        }
        Commands::Recall(args) => {
            let req = resolve_recall_request(&client, &args).await?;
            let response = client.associative_recall(&req).await?;
            if args.summary {
                println!("{}", render_recall_summary(&response, args.follow));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Timeline(args) => {
            let req = memd_schema::TimelineMemoryRequest {
                id: args.id.parse().context("parse memory id as uuid")?,
                route: parse_retrieval_route(args.route.clone())?,
                intent: parse_retrieval_intent(args.intent.clone())?,
                limit: args.limit,
            };
            let response = client.timeline(&req).await?;
            if args.summary {
                println!("{}", render_timeline_summary(&response, args.follow));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Consolidate(args) => {
            let response = client
                .consolidate(&MemoryConsolidationRequest {
                    project: args.project.clone(),
                    namespace: args.namespace.clone(),
                    max_groups: args.max_groups,
                    min_events: args.min_events,
                    lookback_days: args.lookback_days,
                    min_salience: args.min_salience,
                    record_events: Some(args.record_events),
                })
                .await?;
            if args.summary {
                println!("{}", render_consolidate_summary(&response, args.follow));
            } else {
                print_json(&response)?;
            }
        }
        Commands::MaintenanceReport(args) => {
            let response = client
                .maintenance_report(&MemoryMaintenanceReportRequest {
                    project: args.project.clone(),
                    namespace: args.namespace.clone(),
                    inactive_days: args.inactive_days,
                    lookback_days: args.lookback_days,
                    min_events: args.min_events,
                    max_decay: args.max_decay,
                })
                .await?;
            if args.summary {
                println!(
                    "{}",
                    render_maintenance_report_summary(&response, args.follow)
                );
            } else {
                print_json(&response)?;
            }
        }
        Commands::Policy => {
            print_json(&client.policy().await?)?;
        }
        Commands::Compact(args) => {
            if args.spill && args.wire {
                anyhow::bail!("use either --spill or --wire, not both");
            }

            let memory = client
                .context_compact(&ContextRequest {
                    project: args.project.clone(),
                    agent: args.agent.clone(),
                    route: parse_retrieval_route(args.route.clone())?,
                    intent: parse_retrieval_intent(args.intent.clone())?,
                    limit: args.limit,
                    max_chars_per_item: args.max_chars_per_item,
                })
                .await?;

            let packet = build_compaction_packet(
                CompactionSession {
                    project: args.project,
                    agent: args.agent,
                    task: args.task,
                },
                args.goal,
                args.hard_constraint,
                args.active_work,
                args.decision
                    .into_iter()
                    .enumerate()
                    .map(|(idx, text)| CompactionDecision {
                        id: format!("decision-{}", idx + 1),
                        text,
                    })
                    .collect(),
                args.open_loop
                    .into_iter()
                    .enumerate()
                    .map(|(idx, text)| CompactionOpenLoop {
                        id: format!("loop-{}", idx + 1),
                        text,
                        status: "open".to_string(),
                    })
                    .collect(),
                args.exact_ref
                    .into_iter()
                    .map(|value| {
                        let (kind, value) = value
                            .split_once('=')
                            .map(|(kind, value)| {
                                (kind.trim().to_string(), value.trim().to_string())
                            })
                            .unwrap_or_else(|| ("unknown".to_string(), value.trim().to_string()));
                        CompactionReference { kind, value }
                    })
                    .collect(),
                args.next_action,
                args.do_not_drop,
                memory,
            );

            if args.spill {
                let spill = if args.spill_transient {
                    derive_compaction_spill_with_options(
                        &packet,
                        CompactionSpillOptions {
                            include_transient_state: true,
                        },
                    )
                } else {
                    derive_compaction_spill(&packet)
                };
                if args.apply {
                    let responses = client.candidate_batch(&spill.items).await?;
                    let duplicates = responses
                        .iter()
                        .filter(|response| response.duplicate_of.is_some())
                        .count();
                    if let Some(rag) = maybe_rag_client_from_bundle_or_env()? {
                        sync_candidate_responses_to_rag(&rag, &responses).await?;
                    }
                    let submitted = responses.len();
                    let result = CompactionSpillResult {
                        submitted,
                        duplicates,
                        responses,
                        batch: spill,
                    };
                    print_json(&result)?;
                } else {
                    print_json(&spill)?;
                }
            } else if args.wire {
                println!("{}", render_compaction_wire(&packet));
            } else {
                print_json(&packet)?;
            }
        }
        Commands::Obsidian(args) => match args.mode {
            ObsidianMode::Scan => {
                let scan = obsidian::scan_vault(
                    &args.vault,
                    args.project.clone(),
                    args.namespace.clone(),
                    args.max_notes,
                    args.include_attachments,
                    args.max_attachments,
                    &args.include_folder,
                    &args.exclude_folder,
                    &args.include_tag,
                    &args.exclude_tag,
                )?;
                if args.review_sensitive {
                    println!("{}", obsidian::render_sensitive_review(&scan));
                    return Ok(());
                }
                if args.summary {
                    println!("{}", render_obsidian_scan_summary(&scan, args.follow));
                } else {
                    print_json(&scan)?;
                }
            }
            ObsidianMode::Import => {
                run_obsidian_import(&client, &args, false, false).await?;
            }
            ObsidianMode::Sync => {
                run_obsidian_import(&client, &args, true, false).await?;
            }
            ObsidianMode::Compile => {
                run_obsidian_compile(&client, &args).await?;
            }
            ObsidianMode::Writeback => {
                run_obsidian_writeback(&client, &args).await?;
            }
            ObsidianMode::Open => {
                run_obsidian_open(&client, &args).await?;
            }
            ObsidianMode::Roundtrip => {
                run_obsidian_import(&client, &args, true, true).await?;
            }
            ObsidianMode::Watch => {
                run_obsidian_watch(&client, &args).await?;
            }
            ObsidianMode::Status => {
                run_obsidian_status(&client, &args).await?;
            }
        },
        Commands::Hook(args) => match args.mode {
            HookMode::Context(args) => {
                let req = ContextRequest {
                    project: args.project,
                    agent: args.agent,
                    route: parse_retrieval_route(args.route)?,
                    intent: parse_retrieval_intent(args.intent)?,
                    limit: args.limit,
                    max_chars_per_item: args.max_chars_per_item,
                };
                print_json(&client.context_compact(&req).await?)?;
            }
            HookMode::Spill(args) => {
                let packet = read_request::<CompactionPacket>(&args.input)?;
                let spill = if args.spill_transient {
                    derive_compaction_spill_with_options(
                        &packet,
                        CompactionSpillOptions {
                            include_transient_state: true,
                        },
                    )
                } else {
                    derive_compaction_spill(&packet)
                };

                if args.apply {
                    let responses = client.candidate_batch(&spill.items).await?;
                    let duplicates = responses
                        .iter()
                        .filter(|response| response.duplicate_of.is_some())
                        .count();
                    if let Some(rag) = maybe_rag_client_from_bundle_or_env()? {
                        sync_candidate_responses_to_rag(&rag, &responses).await?;
                    }
                    let submitted = responses.len();
                    print_json(&CompactionSpillResult {
                        submitted,
                        duplicates,
                        responses,
                        batch: spill,
                    })?;
                } else {
                    print_json(&spill)?;
                }
            }
        },
        Commands::Init(args) => {
            write_init_bundle(&args)?;
            println!(
                "Initialized memd project bundle at {}",
                args.output.display()
            );
        }
    }

    Ok(())
}

async fn run_obsidian_import(
    client: &MemdClient,
    args: &ObsidianArgs,
    sync_mode: bool,
    mirror_mode: bool,
) -> anyhow::Result<()> {
    let include_attachments = args.include_attachments || sync_mode;
    let link_notes = args.link_notes || sync_mode;
    let apply = args.apply || sync_mode;

    let scan = obsidian::scan_vault(
        &args.vault,
        args.project.clone(),
        args.namespace.clone(),
        args.max_notes,
        include_attachments,
        args.max_attachments,
        &args.include_folder,
        &args.exclude_folder,
        &args.include_tag,
        &args.exclude_tag,
    )?;
    if args.review_sensitive {
        println!("{}", obsidian::render_sensitive_review(&scan));
        return Ok(());
    }
    let (state_path, sync_state) = obsidian::load_sync_state(&args.vault, args.state_file.clone())?;
    let (preview, _candidates, changed_notes) =
        obsidian::build_import_preview(scan, &sync_state, state_path.clone());
    let (attachment_assets, attachment_unchanged_count) = if include_attachments {
        obsidian::partition_changed_attachments(&preview.scan.attachments, &sync_state)
    } else {
        (Vec::new(), 0)
    };

    if apply {
        let mut next_state = sync_state.clone();
        let mut submitted = 0usize;
        let mut duplicates = 0usize;
        let mut note_failures = 0usize;
        for note in &changed_notes {
            let request = obsidian::build_note_request(
                note,
                args.project.clone(),
                args.namespace.clone(),
                preview.scan.vault.clone(),
                next_state
                    .entries
                    .get(&note.relative_path)
                    .and_then(|entry| entry.item_id),
            );
            let response = match client.candidate(&request).await {
                Ok(response) => response,
                Err(err) => {
                    note_failures += 1;
                    eprintln!(
                        "obsidian note import failed for {}: {err:#}",
                        note.relative_path
                    );
                    continue;
                }
            };
            let stored_id = response.duplicate_of.unwrap_or(response.item.id);
            let entity_id = match client
                .entity(&memd_schema::EntityMemoryRequest {
                    id: stored_id,
                    route: None,
                    intent: None,
                    limit: Some(4),
                })
                .await
            {
                Ok(entity) => entity.entity.as_ref().map(|entity| entity.id),
                Err(err) => {
                    note_failures += 1;
                    eprintln!(
                        "obsidian entity lookup failed for {}: {err:#}",
                        note.relative_path
                    );
                    None
                }
            };
            next_state.entries.insert(
                note.relative_path.clone(),
                ObsidianSyncEntry {
                    content_hash: note.content_hash.clone(),
                    bytes: note.bytes,
                    modified_at: note.modified_at,
                    item_id: Some(stored_id),
                    entity_id,
                },
            );
            submitted += 1;
            if response.duplicate_of.is_some() {
                duplicates += 1;
            }
            obsidian::save_sync_state(&state_path, &next_state)?;
        }

        let mut attachment_multimodal = None;
        let mut attachment_submitted = 0usize;
        let mut attachment_duplicates = 0usize;
        let mut attachment_failures = 0usize;
        if include_attachments && !attachment_assets.is_empty() {
            let attachment_paths = attachment_assets
                .iter()
                .map(|asset| asset.path.clone())
                .collect::<Vec<_>>();
            let multimodal_preview = build_multimodal_preview(
                args.project.clone(),
                args.namespace.clone(),
                &attachment_paths,
            )?;
            let rag_url = resolve_rag_url(None, resolve_default_bundle_root()?.as_deref())?;
            let sidecar = SidecarClient::new(&rag_url)?;
            let mut multimodal_responses = Vec::with_capacity(attachment_assets.len());
            let mut ingested_attachment_pairs = Vec::with_capacity(attachment_assets.len());
            for (asset, request) in attachment_assets
                .iter()
                .zip(multimodal_preview.requests.iter())
            {
                let response = match sidecar.ingest(request).await {
                    Ok(response) => response,
                    Err(err) => {
                        attachment_failures += 1;
                        eprintln!(
                            "obsidian attachment ingest failed for {}: {err:#}",
                            asset.relative_path
                        );
                        continue;
                    }
                };
                multimodal_responses.push(response.clone());
                ingested_attachment_pairs.push((asset, response));
            }
            attachment_multimodal = Some(MultimodalIngestOutput {
                preview: multimodal_preview,
                responses: multimodal_responses,
                submitted: ingested_attachment_pairs.len(),
                dry_run: false,
            });

            for (asset, response) in ingested_attachment_pairs {
                let match_ = obsidian::resolve_attachment_match(
                    asset,
                    &preview.scan.notes,
                    &preview.note_index,
                );
                let linked_note = match_
                    .as_ref()
                    .and_then(|association| preview.scan.notes.get(association.note_index));
                let attachment_candidate = obsidian::build_attachment_request(
                    asset,
                    args.project.clone(),
                    args.namespace.clone(),
                    preview.scan.vault.clone(),
                    linked_note,
                    Some(response.track_id),
                );
                let attachment_response = match client.candidate(&attachment_candidate).await {
                    Ok(response) => response,
                    Err(err) => {
                        attachment_failures += 1;
                        eprintln!(
                            "obsidian attachment import failed for {}: {err:#}",
                            asset.relative_path
                        );
                        continue;
                    }
                };
                attachment_submitted += 1;
                if attachment_response.duplicate_of.is_some() {
                    attachment_duplicates += 1;
                }
                let stored_id = attachment_response
                    .duplicate_of
                    .unwrap_or(attachment_response.item.id);
                let entity_id = match client
                    .entity(&memd_schema::EntityMemoryRequest {
                        id: stored_id,
                        route: None,
                        intent: None,
                        limit: Some(4),
                    })
                    .await
                {
                    Ok(entity) => entity.entity.as_ref().map(|entity| entity.id),
                    Err(err) => {
                        attachment_failures += 1;
                        eprintln!(
                            "obsidian attachment entity lookup failed for {}: {err:#}",
                            asset.relative_path
                        );
                        None
                    }
                };
                next_state.entries.insert(
                    asset.relative_path.clone(),
                    ObsidianSyncEntry {
                        content_hash: asset.content_hash.clone(),
                        bytes: asset.bytes,
                        modified_at: asset.modified_at,
                        item_id: Some(stored_id),
                        entity_id,
                    },
                );
                obsidian::save_sync_state(&state_path, &next_state)?;
            }
        }

        let mut entity_ids_by_item_id = std::collections::HashMap::new();
        for entry in next_state.entries.values() {
            let Some(item_id) = entry.item_id else {
                continue;
            };
            if entity_ids_by_item_id.contains_key(&item_id) {
                continue;
            }
            if let Some(entity_id) = entry.entity_id {
                entity_ids_by_item_id.insert(item_id, entity_id);
                continue;
            }
            let entity = client
                .entity(&memd_schema::EntityMemoryRequest {
                    id: item_id,
                    route: None,
                    intent: None,
                    limit: Some(4),
                })
                .await?;
            if let Some(entity) = entity.entity {
                entity_ids_by_item_id.insert(item_id, entity.id);
            }
        }

        obsidian::save_sync_state(&state_path, &next_state)?;

        let mut links_created = 0usize;
        if link_notes {
            for note in &preview.scan.notes {
                let Some(from_entity_id) = next_state
                    .entries
                    .get(&note.relative_path)
                    .and_then(|entry| entry.item_id)
                    .and_then(|item_id| entity_ids_by_item_id.get(&item_id).copied())
                else {
                    continue;
                };
                for target in &note.links {
                    let target_key = obsidian::normalized_title(target);
                    let Some(target_idx) = preview.note_index.get(&target_key) else {
                        continue;
                    };
                    let target_note = &preview.scan.notes[*target_idx];
                    let Some(to_entity_id) = next_state
                        .entries
                        .get(&target_note.relative_path)
                        .and_then(|entry| entry.item_id)
                        .and_then(|item_id| entity_ids_by_item_id.get(&item_id).copied())
                    else {
                        continue;
                    };
                    if from_entity_id == to_entity_id {
                        continue;
                    }
                    let request =
                        obsidian::build_entity_link_request(from_entity_id, to_entity_id, note);
                    let _ = client.link_entity(&request).await?;
                    links_created += 1;
                }
            }
        }

        let mut attachment_links_created = 0usize;
        if include_attachments && !attachment_assets.is_empty() {
            for asset in &attachment_assets {
                let Some(match_) = obsidian::resolve_attachment_match(
                    asset,
                    &preview.scan.notes,
                    &preview.note_index,
                ) else {
                    continue;
                };
                let Some(attachment_entry) = next_state.entries.get(&asset.relative_path) else {
                    continue;
                };
                let Some(attachment_item_id) = attachment_entry.item_id else {
                    continue;
                };
                let Some(attachment_entity_id) =
                    entity_ids_by_item_id.get(&attachment_item_id).copied()
                else {
                    continue;
                };
                let Some(note) = preview.scan.notes.get(match_.note_index) else {
                    continue;
                };
                let Some(note_entry) = next_state.entries.get(&note.relative_path) else {
                    continue;
                };
                let Some(note_item_id) = note_entry.item_id else {
                    continue;
                };
                let Some(note_entity_id) = entity_ids_by_item_id.get(&note_item_id).copied() else {
                    continue;
                };
                if attachment_entity_id == note_entity_id {
                    continue;
                }
                let request = memd_schema::EntityLinkRequest {
                    from_entity_id: attachment_entity_id,
                    to_entity_id: note_entity_id,
                    relation_kind: match_.relation_kind,
                    confidence: Some(0.78),
                    note: Some(format!("obsidian attachment from {}", asset.relative_path)),
                    context: Some(memd_schema::MemoryContextFrame {
                        at: Some(chrono::Utc::now()),
                        project: args.project.clone(),
                        namespace: args.namespace.clone(),
                        repo: Some("obsidian".to_string()),
                        host: None,
                        branch: None,
                        agent: Some("obsidian".to_string()),
                        location: Some(asset.relative_path.clone()),
                    }),
                    tags: vec![
                        "obsidian".to_string(),
                        "vault_attachment".to_string(),
                        format!("linked_note={}", note.normalized_title),
                        format!("reason={}", match_.reason),
                    ],
                };
                let _ = client.link_entity(&request).await?;
                attachment_links_created += 1;
            }
        }

        let mut mirrored_notes = 0usize;
        let mut mirrored_attachments = 0usize;
        if mirror_mode {
            for note in &preview.scan.notes {
                let Some(entry) = next_state.entries.get(&note.relative_path) else {
                    continue;
                };
                let Some(item_id) = entry.item_id else {
                    continue;
                };
                let entity_id = if let Some(entity_id) = entry.entity_id {
                    Some(entity_id)
                } else {
                    let entity = client
                        .entity(&memd_schema::EntityMemoryRequest {
                            id: item_id,
                            route: None,
                            intent: None,
                            limit: Some(4),
                        })
                        .await?;
                    entity.entity.as_ref().map(|entity| entity.id)
                };
                let block = obsidian::build_roundtrip_annotation(note, Some(item_id), entity_id);
                obsidian::annotate_note(&note.path, &block)?;
                let (_, mirror_markdown) =
                    obsidian::build_note_mirror_markdown(note, Some(item_id), entity_id);
                let mirror_path = obsidian::note_mirror_path(&preview.scan.vault, note);
                obsidian::write_markdown(&mirror_path, &mirror_markdown)?;
                mirrored_notes += 1;
            }

            if include_attachments {
                for asset in &attachment_assets {
                    let Some(entry) = next_state.entries.get(&asset.relative_path) else {
                        continue;
                    };
                    let Some(item_id) = entry.item_id else {
                        continue;
                    };
                    let entity_id = if let Some(entity_id) = entry.entity_id {
                        Some(entity_id)
                    } else {
                        let entity = client
                            .entity(&memd_schema::EntityMemoryRequest {
                                id: item_id,
                                route: None,
                                intent: None,
                                limit: Some(4),
                            })
                            .await?;
                        entity.entity.as_ref().map(|entity| entity.id)
                    };
                    let linked_note = obsidian::resolve_attachment_match(
                        asset,
                        &preview.scan.notes,
                        &preview.note_index,
                    )
                    .and_then(|association| preview.scan.notes.get(association.note_index));
                    let (_, mirror_markdown) = obsidian::build_attachment_mirror_markdown(
                        asset,
                        Some(item_id),
                        entity_id,
                        linked_note,
                        None,
                    );
                    let mirror_path = obsidian::attachment_mirror_path(&preview.scan.vault, asset);
                    obsidian::write_markdown(&mirror_path, &mirror_markdown)?;
                    mirrored_attachments += 1;
                }
            }
        }

        let output = ObsidianImportOutput {
            preview,
            submitted,
            attachment_submitted,
            duplicates,
            attachment_duplicates,
            note_failures,
            attachment_failures,
            links_created,
            attachment_links_created,
            mirrored_notes,
            mirrored_attachments,
            attachments: attachment_multimodal,
            attachment_unchanged_count,
            dry_run: false,
        };
        if args.summary {
            println!("{}", render_obsidian_import_summary(&output, args.follow));
        } else {
            print_json(&output)?;
        }
    } else {
        let output = ObsidianImportOutput {
            preview,
            submitted: 0,
            attachment_submitted: 0,
            duplicates: 0,
            attachment_duplicates: 0,
            note_failures: 0,
            attachment_failures: 0,
            links_created: 0,
            attachment_links_created: 0,
            mirrored_notes: 0,
            mirrored_attachments: 0,
            attachments: None,
            attachment_unchanged_count,
            dry_run: true,
        };
        if args.summary {
            println!("{}", render_obsidian_import_summary(&output, args.follow));
        } else {
            print_json(&output)?;
        }
    }

    Ok(())
}

async fn run_obsidian_writeback(client: &MemdClient, args: &ObsidianArgs) -> anyhow::Result<()> {
    let Some(id) = args.id.as_ref() else {
        anyhow::bail!("obsidian writeback requires --id <uuid>");
    };
    let id = id
        .parse::<uuid::Uuid>()
        .context("parse obsidian writeback id")?;
    let explain = client
        .explain(&ExplainMemoryRequest {
            id,
            belief_branch: None,
            route: None,
            intent: None,
        })
        .await?;

    let output_path = args
        .output
        .clone()
        .unwrap_or_else(|| obsidian::default_writeback_path(&args.vault, &explain));
    let (title, markdown) = obsidian::build_writeback_markdown(&explain, explain.entity.as_ref());

    let preview = serde_json::json!({
        "output_path": output_path.display().to_string(),
        "open_uri": obsidian::build_open_uri(&output_path, args.pane_type.as_deref())?,
        "title": title,
        "id": explain.item.id,
        "kind": format!("{:?}", explain.item.kind).to_lowercase(),
        "summary": explain.item.content.clone(),
        "reasons": explain.reasons.clone(),
        "entity": explain.entity.as_ref().map(|entity| entity.id),
        "events": explain.events.len(),
        "apply": args.apply,
    });

    if !args.apply {
        print_json(&preview)?;
        return Ok(());
    }

    if output_path.exists() && !args.overwrite {
        anyhow::bail!(
            "{} already exists; pass --overwrite to replace it",
            output_path.display()
        );
    }
    obsidian::write_markdown(&output_path, &markdown)?;
    if args.open {
        let uri = obsidian::build_open_uri(&output_path, args.pane_type.as_deref())?;
        obsidian::open_uri(&uri)?;
    }
    print_json(&preview)?;
    Ok(())
}

async fn run_obsidian_compile(client: &MemdClient, args: &ObsidianArgs) -> anyhow::Result<()> {
    let Some(query) = args.query.as_ref() else {
        anyhow::bail!("obsidian compile requires --query <text>");
    };
    let response = client
        .search(&SearchMemoryRequest {
            query: Some(query.clone()),
            route: parse_retrieval_route(args.route.clone())?,
            intent: parse_retrieval_intent(args.intent.clone())?,
            scopes: vec![MemoryScope::Project, MemoryScope::Global, MemoryScope::Synced],
            kinds: vec![],
            statuses: vec![MemoryStatus::Active, MemoryStatus::Stale, MemoryStatus::Contested],
            project: args.project.clone(),
            namespace: args.namespace.clone(),
            belief_branch: None,
            source_agent: None,
            tags: Vec::new(),
            stages: vec![MemoryStage::Canonical, MemoryStage::Candidate],
            limit: Some(args.limit.unwrap_or(12).clamp(1, 48)),
            max_chars_per_item: Some(800),
        })
        .await?;

    let output_path = args
        .output
        .clone()
        .unwrap_or_else(|| obsidian::default_compiled_note_path(&args.vault, query));
    let (title, markdown) = obsidian::build_compiled_note_markdown(query, &response);
    let preview = serde_json::json!({
        "output_path": output_path.display().to_string(),
        "open_uri": obsidian::build_open_uri(&output_path, args.pane_type.as_deref())?,
        "title": title,
        "query": query,
        "items": response.items.len(),
        "apply": args.apply,
    });

    if !args.apply {
        print_json(&preview)?;
        return Ok(());
    }

    if output_path.exists() && !args.overwrite {
        anyhow::bail!(
            "{} already exists; pass --overwrite to replace it",
            output_path.display()
        );
    }
    obsidian::write_markdown(&output_path, &markdown)?;
    if args.open {
        let uri = obsidian::build_open_uri(&output_path, args.pane_type.as_deref())?;
        obsidian::open_uri(&uri)?;
    }
    print_json(&preview)?;
    Ok(())
}

async fn run_obsidian_open(client: &MemdClient, args: &ObsidianArgs) -> anyhow::Result<()> {
    let target_path = if let Some(note) = args.note.as_ref() {
        obsidian::resolve_open_path(&args.vault, note)
    } else if let Some(id) = args.id.as_ref() {
        let id = id
            .parse::<uuid::Uuid>()
            .context("parse obsidian open id")?;
        let explain = client
            .explain(&ExplainMemoryRequest {
                id,
                belief_branch: None,
                route: None,
                intent: None,
            })
            .await?;
        args.output
            .clone()
            .unwrap_or_else(|| obsidian::default_writeback_path(&args.vault, &explain))
    } else if let Some(output) = args.output.as_ref() {
        obsidian::resolve_open_path(&args.vault, output)
    } else {
        args.vault.clone()
    };

    let uri = obsidian::build_open_uri(&target_path, args.pane_type.as_deref())?;
    let preview = serde_json::json!({
        "vault": args.vault.display().to_string(),
        "target_path": target_path.display().to_string(),
        "open_uri": uri,
        "apply": args.apply,
    });

    if !args.apply {
        print_json(&preview)?;
        return Ok(());
    }

    obsidian::open_uri(preview["open_uri"].as_str().unwrap_or_default())?;
    print_json(&preview)?;
    Ok(())
}

async fn run_obsidian_status(_client: &MemdClient, args: &ObsidianArgs) -> anyhow::Result<()> {
    let scan = obsidian::scan_vault(
        &args.vault,
        args.project.clone(),
        args.namespace.clone(),
        args.max_notes,
        args.include_attachments,
        args.max_attachments,
        &args.include_folder,
        &args.exclude_folder,
        &args.include_tag,
        &args.exclude_tag,
    )?;
    let (state_path, sync_state) = obsidian::load_sync_state(&args.vault, args.state_file.clone())?;
    let (preview, _, _) = obsidian::build_import_preview(scan, &sync_state, state_path.clone());
    let attachment_assets = if args.include_attachments {
        obsidian::partition_changed_attachments(&preview.scan.attachments, &sync_state).0
    } else {
        Vec::new()
    };
    let mirror_notes = count_obsidian_mirrors(&args.vault, "notes")?;
    let mirror_attachments = count_obsidian_mirrors(&args.vault, "attachments")?;
    let sync_state_entries = sync_state.entries.len();
    let changed_notes = preview.candidates.len();
    let unchanged_notes = preview.unchanged_count;
    let changed_attachments = attachment_assets.len();
    let unchanged_attachments = preview.scan.attachment_unchanged_count;
    let roundtrip_live = sync_state_entries > 0 || mirror_notes > 0 || mirror_attachments > 0;
    let mut summary = format!(
        "obsidian_status vault={} notes={} changed_notes={} unchanged_notes={} attachments={} changed_attachments={} unchanged_attachments={} sync_entries={} mirrors_notes={} mirrors_attachments={} roundtrip_live={} state={}",
        args.vault.display(),
        preview.scan.note_count,
        changed_notes,
        unchanged_notes,
        preview.scan.attachment_count,
        changed_attachments,
        unchanged_attachments,
        sync_state_entries,
        mirror_notes,
        mirror_attachments,
        roundtrip_live,
        state_path.display()
    );
    if args.follow {
        let trail = preview
            .scan
            .notes
            .iter()
            .take(3)
            .map(|note| note.title.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        if !trail.is_empty() {
            summary.push_str(&format!(" trail={trail}"));
        }
    }
    if args.summary {
        println!("{summary}");
    } else {
        print_json(&serde_json::json!({
            "vault": preview.scan.vault,
            "project": preview.scan.project,
            "namespace": preview.scan.namespace,
            "notes": preview.scan.note_count,
            "changed_notes": changed_notes,
            "unchanged_notes": unchanged_notes,
            "attachments": preview.scan.attachment_count,
            "changed_attachments": changed_attachments,
            "unchanged_attachments": unchanged_attachments,
            "sync_state_entries": sync_state_entries,
            "mirror_notes": mirror_notes,
            "mirror_attachments": mirror_attachments,
            "roundtrip_live": roundtrip_live,
            "state_path": state_path,
        }))?;
    }
    Ok(())
}

async fn run_obsidian_watch(client: &MemdClient, args: &ObsidianArgs) -> anyhow::Result<()> {
    println!(
        "obsidian_watch vault={} debounce_ms={}",
        args.vault.display(),
        args.debounce_ms
    );

    run_obsidian_import(client, args, true, true).await?;

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<()>();
    let mut watcher = RecommendedWatcher::new(
        move |result: notify::Result<notify::Event>| {
            if let Ok(event) = result {
                let should_trigger = matches!(
                    event.kind,
                    EventKind::Create(_)
                        | EventKind::Modify(_)
                        | EventKind::Remove(_)
                        | EventKind::Any
                ) && event
                    .paths
                    .iter()
                    .any(|path| !obsidian_path_is_internal(path));
                if should_trigger {
                    let _ = tx.send(());
                }
            }
        },
        NotifyConfig::default(),
    )
    .context("create obsidian watcher")?;
    watcher
        .watch(&args.vault, RecursiveMode::Recursive)
        .with_context(|| format!("watch {}", args.vault.display()))?;

    let debounce = Duration::from_millis(args.debounce_ms.max(100));
    loop {
        if rx.recv().await.is_none() {
            break;
        }

        let mut dirty = true;
        while dirty {
            dirty = false;
            tokio::time::sleep(debounce).await;
            while rx.try_recv().is_ok() {
                dirty = true;
            }
        }

        if let Err(err) = run_obsidian_import(client, args, true, true).await {
            eprintln!("obsidian watch sync failed: {err:#}");
        }
    }

    Ok(())
}

fn obsidian_path_is_internal(path: &Path) -> bool {
    path.components().any(|component| {
        matches!(
            component,
            std::path::Component::Normal(name)
                if name == ".memd" || name == ".obsidian" || name == ".git"
        )
    })
}

fn count_obsidian_mirrors(vault: &Path, kind: &str) -> anyhow::Result<usize> {
    let root = vault.join(".memd").join("writeback").join(kind);
    if !root.exists() {
        return Ok(0);
    }
    let mut count = 0usize;
    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if entry.file_type().is_file() {
            count += 1;
        }
    }
    Ok(count)
}

fn read_request<T>(input: &RequestInput) -> anyhow::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let json = if let Some(json) = &input.json {
        json.clone()
    } else if let Some(path) = &input.input {
        fs::read_to_string(path).with_context(|| format!("read request file {}", path.display()))?
    } else if input.stdin {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("read request from stdin")?;
        buffer
    } else {
        anyhow::bail!("provide --json, --input, or --stdin");
    };

    serde_json::from_str(&json).context("parse request json")
}

fn print_json<T>(value: &T) -> anyhow::Result<()>
where
    T: serde::Serialize,
{
    let json = serde_json::to_string_pretty(value).context("serialize response json")?;
    println!("{json}");
    Ok(())
}

#[derive(Debug, Serialize)]
struct ObsidianImportOutput {
    preview: ObsidianImportPreview,
    submitted: usize,
    attachment_submitted: usize,
    duplicates: usize,
    attachment_duplicates: usize,
    note_failures: usize,
    attachment_failures: usize,
    links_created: usize,
    attachment_links_created: usize,
    mirrored_notes: usize,
    mirrored_attachments: usize,
    attachments: Option<MultimodalIngestOutput>,
    attachment_unchanged_count: usize,
    dry_run: bool,
}

async fn resolve_recall_request(
    client: &MemdClient,
    args: &RecallArgs,
) -> anyhow::Result<AssociativeRecallRequest> {
    if let Some(entity_id) = &args.entity_id {
        return Ok(AssociativeRecallRequest {
            entity_id: entity_id.parse().context("parse entity id as uuid")?,
            depth: args.depth,
            limit: args.limit,
        });
    }

    let query = args
        .query
        .clone()
        .context("provide either --entity-id or --query")?;
    let response = client
        .entity_search(&EntitySearchRequest {
            query,
            project: args.project.clone(),
            namespace: args.namespace.clone(),
            at: parse_context_time(args.at.clone())?,
            host: args.host.clone(),
            branch: args.branch.clone(),
            location: args.location.clone(),
            route: None,
            intent: None,
            limit: Some(5),
        })
        .await
        .context("resolve recall target")?;

    let Some(best_match) = response.best_match else {
        anyhow::bail!("no entity matched the recall query");
    };
    if response.ambiguous {
        anyhow::bail!(
            "recall query was ambiguous; use --entity-id instead (best match {}::{})",
            short_uuid(best_match.entity.id),
            best_match.entity.entity_type,
        );
    }

    Ok(AssociativeRecallRequest {
        entity_id: best_match.entity.id,
        depth: args.depth,
        limit: args.limit,
    })
}

#[derive(Debug, Serialize)]
struct RagSyncSummary {
    fetched: usize,
    pushed: usize,
    dry_run: bool,
    project: Option<String>,
    namespace: Option<String>,
}

async fn sync_to_rag(
    memd: &MemdClient,
    rag: &RagClient,
    args: RagSyncArgs,
) -> anyhow::Result<RagSyncSummary> {
    let fetched = memd
        .search(&SearchMemoryRequest {
            query: None,
            route: Some(RetrievalRoute::All),
            intent: Some(RetrievalIntent::General),
            scopes: vec![MemoryScope::Project, MemoryScope::Global],
            kinds: vec![
                MemoryKind::Fact,
                MemoryKind::Decision,
                MemoryKind::Preference,
                MemoryKind::Runbook,
                MemoryKind::Procedural,
                MemoryKind::SelfModel,
                MemoryKind::Topology,
                MemoryKind::Status,
                MemoryKind::Pattern,
                MemoryKind::Constraint,
            ],
            statuses: vec![MemoryStatus::Active],
            project: args.project.clone(),
            namespace: args.namespace.clone(),
            belief_branch: None,
            source_agent: None,
            tags: Vec::new(),
            stages: vec![MemoryStage::Canonical],
            limit: args.limit,
            max_chars_per_item: Some(1000),
        })
        .await
        .context("load canonical memory for rag sync")?;

    let mut pushed = 0usize;
    for item in &fetched.items {
        if !args.dry_run {
            rag.ingest(&RagIngestRequest::from(item))
                .await
                .context("ingest rag record")?;
        }
        pushed += 1;
    }

    Ok(RagSyncSummary {
        fetched: fetched.items.len(),
        pushed,
        dry_run: args.dry_run,
        project: args.project,
        namespace: args.namespace,
    })
}

async fn sync_candidate_responses_to_rag(
    rag: &RagClient,
    responses: &[memd_schema::CandidateMemoryResponse],
) -> anyhow::Result<usize> {
    let mut pushed = 0usize;
    for response in responses {
        rag.ingest(&RagIngestRequest::from(&response.item))
            .await
            .context("ingest rag record from spill")?;
        pushed += 1;
    }
    Ok(pushed)
}

fn parse_rag_retrieve_mode(value: &str) -> anyhow::Result<RagRetrieveMode> {
    let normalized = value.trim().to_ascii_lowercase().replace('-', "_");
    match normalized.as_str() {
        "auto" => Ok(RagRetrieveMode::Auto),
        "text" => Ok(RagRetrieveMode::Text),
        "multimodal" => Ok(RagRetrieveMode::Multimodal),
        "graph" => Ok(RagRetrieveMode::Graph),
        _ => anyhow::bail!(
            "invalid rag retrieve mode '{value}'; expected auto, text, multimodal, or graph"
        ),
    }
}

#[derive(Debug, Serialize)]
struct IngestAutoRouteResult {
    route: String,
    request: Option<CandidateMemoryRequest>,
    candidate: Option<memd_schema::CandidateMemoryResponse>,
    multimodal: Option<MultimodalIngestOutput>,
}

async fn ingest_auto_route(
    client: &MemdClient,
    args: &IngestArgs,
) -> anyhow::Result<IngestAutoRouteResult> {
    if let Some(content) = &args.content {
        return ingest_text_memory(client, args, content.clone()).await;
    }

    if args.input.is_some() || args.stdin {
        let raw = read_ingest_payload(args)?;
        if looks_like_multimodal(&raw) {
            return ingest_multimodal_payload(args, &raw).await;
        }
        return ingest_text_memory(client, args, raw).await;
    }

    anyhow::bail!("provide --content, --input, or --stdin");
}

fn read_ingest_payload(args: &IngestArgs) -> anyhow::Result<String> {
    if let Some(json) = &args.json {
        return Ok(json.clone());
    }
    if let Some(path) = &args.input {
        return fs::read_to_string(path)
            .with_context(|| format!("read ingest input file {}", path.display()));
    }
    if args.stdin {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("read ingest payload from stdin")?;
        return Ok(buffer);
    }
    anyhow::bail!("no ingest payload");
}

fn looks_like_multimodal(input: &str) -> bool {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        return false;
    }
    trimmed.split_whitespace().any(|token| {
        let lowered = token
            .trim_matches(|c: char| c == ',' || c == ';')
            .to_ascii_lowercase();
        lowered.ends_with(".pdf")
            || lowered.ends_with(".png")
            || lowered.ends_with(".jpg")
            || lowered.ends_with(".jpeg")
            || lowered.ends_with(".webp")
            || lowered.ends_with(".heic")
            || lowered.ends_with(".mp4")
            || lowered.ends_with(".mov")
            || lowered.ends_with(".mkv")
            || lowered.ends_with(".webm")
            || lowered.ends_with(".csv")
            || lowered.ends_with(".tsv")
            || lowered.ends_with(".xlsx")
            || lowered.ends_with(".txt")
            || lowered.ends_with(".md")
    })
}

async fn ingest_text_memory(
    client: &MemdClient,
    args: &IngestArgs,
    content: String,
) -> anyhow::Result<IngestAutoRouteResult> {
    let kind = args
        .kind
        .as_deref()
        .map(parse_memory_kind_value)
        .transpose()?
        .unwrap_or(MemoryKind::Fact);
    let scope = args
        .scope
        .as_deref()
        .map(parse_memory_scope_value)
        .transpose()?
        .unwrap_or(MemoryScope::Project);
    let source_quality = args
        .source_quality
        .as_deref()
        .map(parse_source_quality_value)
        .transpose()?;
    let supersedes = parse_uuid_list(&args.supersede)?;
    let tags = args.tag.clone();

    let req = CandidateMemoryRequest {
        content,
        kind,
        scope,
        project: args.project.clone(),
        namespace: args.namespace.clone(),
        belief_branch: None,
        source_agent: args.source_agent.clone(),
        source_system: args.source_system.clone(),
        source_path: args.source_path.clone(),
        source_quality,
        confidence: args.confidence,
        ttl_seconds: args.ttl_seconds,
        last_verified_at: None,
        supersedes,
        tags,
    };

    if args.apply {
        let candidate = client.candidate(&req).await?;
        Ok(IngestAutoRouteResult {
            route: "memory".to_string(),
            request: Some(req),
            candidate: Some(candidate),
            multimodal: None,
        })
    } else {
        Ok(IngestAutoRouteResult {
            route: "memory".to_string(),
            request: Some(req),
            candidate: None,
            multimodal: None,
        })
    }
}

async fn ingest_multimodal_payload(
    args: &IngestArgs,
    payload: &str,
) -> anyhow::Result<IngestAutoRouteResult> {
    let paths = payload
        .lines()
        .flat_map(|line| line.split_whitespace())
        .map(|token| token.trim_matches(|c: char| c == ',' || c == ';'))
        .filter(|token| {
            token.ends_with(".pdf")
                || token.ends_with(".png")
                || token.ends_with(".jpg")
                || token.ends_with(".jpeg")
                || token.ends_with(".webp")
                || token.ends_with(".heic")
                || token.ends_with(".mp4")
                || token.ends_with(".mov")
                || token.ends_with(".mkv")
                || token.ends_with(".webm")
                || token.ends_with(".csv")
                || token.ends_with(".tsv")
                || token.ends_with(".xlsx")
                || token.ends_with(".txt")
                || token.ends_with(".md")
        })
        .map(PathBuf::from)
        .collect::<Vec<_>>();

    let preview = build_multimodal_preview(args.project.clone(), args.namespace.clone(), &paths)?;
    let multimodal = if args.apply {
        let rag_url = resolve_rag_url(None, resolve_default_bundle_root()?.as_deref())?;
        let sidecar = SidecarClient::new(&rag_url)?;
        let responses = ingest_multimodal_preview(&sidecar, &preview.requests).await?;
        let submitted = responses.len();
        MultimodalIngestOutput {
            preview,
            responses,
            submitted,
            dry_run: false,
        }
    } else {
        MultimodalIngestOutput {
            preview,
            responses: Vec::new(),
            submitted: 0,
            dry_run: true,
        }
    };

    Ok(IngestAutoRouteResult {
        route: "multimodal".to_string(),
        request: None,
        candidate: None,
        multimodal: Some(multimodal),
    })
}

fn parse_context_time(
    value: Option<String>,
) -> anyhow::Result<Option<chrono::DateTime<chrono::Utc>>> {
    match value {
        Some(value) => Ok(Some(
            chrono::DateTime::parse_from_rfc3339(&value)
                .context("parse context time as RFC3339")?
                .with_timezone(&chrono::Utc),
        )),
        None => Ok(None),
    }
}

#[derive(Debug, Serialize)]
struct MultimodalPreview {
    plan: MultimodalIngestPlan,
    chunks: Vec<MultimodalChunk>,
    requests: Vec<SidecarIngestRequest>,
}

#[derive(Debug, Serialize)]
struct MultimodalIngestOutput {
    preview: MultimodalPreview,
    responses: Vec<SidecarIngestResponse>,
    submitted: usize,
    dry_run: bool,
}

fn build_multimodal_preview(
    project: Option<String>,
    namespace: Option<String>,
    paths: &[PathBuf],
) -> anyhow::Result<MultimodalPreview> {
    if paths.is_empty() {
        anyhow::bail!("provide at least one --path");
    }

    let plan = build_ingest_plan(paths.iter(), project, namespace)?;
    let chunks = extract_chunks(&plan)?;
    let requests = to_sidecar_requests(&plan, &chunks);

    Ok(MultimodalPreview {
        plan,
        chunks,
        requests,
    })
}

async fn ingest_multimodal_preview(
    sidecar: &SidecarClient,
    requests: &[SidecarIngestRequest],
) -> anyhow::Result<Vec<SidecarIngestResponse>> {
    let mut responses = Vec::with_capacity(requests.len());
    for request in requests {
        responses.push(sidecar.ingest(request).await?);
    }
    Ok(responses)
}

fn resolve_default_bundle_root() -> anyhow::Result<Option<PathBuf>> {
    if let Ok(value) = std::env::var("MEMD_BUNDLE_ROOT") {
        let value = value.trim();
        if !value.is_empty() {
            return Ok(Some(PathBuf::from(value)));
        }
    }

    let cwd = std::env::current_dir().context("read current directory")?;
    let bundle_root = cwd.join(".memd");
    if bundle_root.join("config.json").exists() {
        return Ok(Some(bundle_root));
    }

    Ok(None)
}

fn resolve_rag_url(explicit: Option<String>, bundle_root: Option<&Path>) -> anyhow::Result<String> {
    if let Some(value) = explicit.map(|value| value.trim().to_string()).filter(|value| !value.is_empty()) {
        return Ok(value);
    }

    if let Some(bundle_root) = bundle_root {
        if let Some(config) = read_bundle_rag_config(bundle_root)? {
            if config.enabled {
                if let Some(url) = config.url {
                    return Ok(url);
                }
                anyhow::bail!(
                    "rag backend is enabled in {} but no url was configured",
                    bundle_root.display()
                );
            }
        }
    }

    if let Ok(value) = std::env::var("MEMD_RAG_URL") {
        let value = value.trim().to_string();
        if !value.is_empty() {
            return Ok(value);
        }
    }

    anyhow::bail!("provide --rag-url, configure rag_url in the bundle, or set MEMD_RAG_URL")
}

fn maybe_rag_client_from_bundle_or_env() -> anyhow::Result<Option<RagClient>> {
    if let Some(bundle_root) = resolve_default_bundle_root()? {
        if let Some(config) = read_bundle_rag_config(bundle_root.as_path())? {
            if config.enabled {
                let rag_url = config.url.with_context(|| {
                    format!(
                        "rag backend is enabled in {} but no url was configured",
                        bundle_root.display()
                    )
                })?;
                return Ok(Some(RagClient::new(rag_url)?));
            }
        }
    }

    match std::env::var("MEMD_RAG_URL") {
        Ok(value) if !value.trim().is_empty() => Ok(Some(RagClient::new(value)?)),
        _ => Ok(None),
    }
}

fn write_init_bundle(args: &InitArgs) -> anyhow::Result<()> {
    let output = &args.output;
    if output.exists() && !args.force {
        anyhow::bail!(
            "{} already exists; pass --force to overwrite",
            output.display()
        );
    }

    fs::create_dir_all(output.join("hooks"))
        .with_context(|| format!("create {}", output.join("hooks").display()))?;
    fs::create_dir_all(output.join("agents"))
        .with_context(|| format!("create {}", output.join("agents").display()))?;

    let config = BundleConfig {
        schema_version: 2,
        project: args.project.clone(),
        agent: args.agent.clone(),
        base_url: args.base_url.clone(),
        route: args.route.clone(),
        intent: args.intent.clone(),
        backend: BundleBackendConfig {
            rag: BundleRagConfig {
                enabled: args.rag_url.is_some(),
                provider: "lightrag-compatible".to_string(),
                url: args.rag_url.clone(),
            },
        },
        hooks: BundleHooksConfig {
            context: "hooks/memd-context.sh".to_string(),
            spill: "hooks/memd-spill.sh".to_string(),
            context_ps1: "hooks/memd-context.ps1".to_string(),
            spill_ps1: "hooks/memd-spill.ps1".to_string(),
        },
        rag_url: args.rag_url.clone(),
    };
    fs::write(
        output.join("config.json"),
        serde_json::to_string_pretty(&config)? + "\n",
    )
    .with_context(|| format!("write {}", output.join("config.json").display()))?;

    write_bundle_backend_env(output, &config)?;

    fs::write(
        output.join("env"),
        format!(
            "MEMD_BASE_URL={}\nMEMD_PROJECT={}\nMEMD_AGENT={}\nMEMD_ROUTE={}\nMEMD_INTENT={}\n{}",
            args.base_url,
            args.project,
            args.agent,
            args.route,
            args.intent,
            args.rag_url
                .as_ref()
                .map(|value| format!("MEMD_RAG_URL={value}\n"))
                .unwrap_or_default(),
        ),
    )
    .with_context(|| format!("write {}", output.join("env").display()))?;

    fs::write(
        output.join("env.ps1"),
        format!(
            "$env:MEMD_BASE_URL = \"{}\"\n$env:MEMD_PROJECT = \"{}\"\n$env:MEMD_AGENT = \"{}\"\n$env:MEMD_ROUTE = \"{}\"\n$env:MEMD_INTENT = \"{}\"\n{}",
            escape_ps1(&args.base_url),
            escape_ps1(&args.project),
            escape_ps1(&args.agent),
            escape_ps1(&args.route),
            escape_ps1(&args.intent),
            args.rag_url
                .as_ref()
                .map(|value| format!("$env:MEMD_RAG_URL = \"{}\"\n", escape_ps1(value)))
                .unwrap_or_default(),
        ),
    )
    .with_context(|| format!("write {}", output.join("env.ps1").display()))?;

    let hook_root = output.join("hooks");
    copy_hook_assets(Path::new(&hook_root))?;
    write_agent_profiles(output)?;

    fs::write(
        output.join("README.md"),
        format!(
            "# memd project bundle\n\nThis directory contains the local memd configuration for `{project}`.\n\n## Files\n\n- `config.json`\n- `env`\n- `env.ps1`\n- `hooks/`\n\n## Usage\n\nSource `env` or `env.ps1` before running the hook kit, or point your agent integration at these values directly.\n",
            project = args.project
        ),
    )
    .with_context(|| format!("write {}", output.join("README.md").display()))?;

    Ok(())
}

fn write_agent_profiles(output: &Path) -> anyhow::Result<()> {
    let agents_dir = output.join("agents");
    let shell_profile = format!(
        "#!/usr/bin/env bash\nset -euo pipefail\n\nexport MEMD_BUNDLE_ROOT=\"{}\"\nsource \"$MEMD_BUNDLE_ROOT/backend.env\" 2>/dev/null || true\nsource \"$MEMD_BUNDLE_ROOT/env\"\nexec memd hook context --project \"$MEMD_PROJECT\" --agent \"$MEMD_AGENT\" --route \"$MEMD_ROUTE\" --intent \"$MEMD_INTENT\" \"$@\"\n",
        compact_bundle_value(output.to_string_lossy().as_ref()),
    );
    fs::write(agents_dir.join("agent.sh"), shell_profile)
        .with_context(|| format!("write {}", agents_dir.join("agent.sh").display()))?;
    set_executable_if_shell_script(&agents_dir.join("agent.sh"), "agent.sh")?;

    let ps1_profile = format!(
        "$env:MEMD_BUNDLE_ROOT = \"{}\"\n$bundleBackendEnv = Join-Path $env:MEMD_BUNDLE_ROOT \"backend.env.ps1\"\nif (Test-Path $bundleBackendEnv) {{ . $bundleBackendEnv }}\n. (Join-Path $env:MEMD_BUNDLE_ROOT \"env.ps1\")\nmemd hook context --project $env:MEMD_PROJECT --agent $env:MEMD_AGENT --route $env:MEMD_ROUTE --intent $env:MEMD_INTENT\n",
        escape_ps1(output.to_string_lossy().as_ref()),
    );
    fs::write(agents_dir.join("agent.ps1"), ps1_profile)
        .with_context(|| format!("write {}", agents_dir.join("agent.ps1").display()))?;

    Ok(())
}

fn write_bundle_backend_env(output: &Path, config: &BundleConfig) -> anyhow::Result<()> {
    let backend_env = output.join("backend.env");
    let backend_env_ps1 = output.join("backend.env.ps1");
    let rag = &config.backend.rag;

    let mut shell = String::new();
    shell.push_str(&format!("MEMD_BUNDLE_SCHEMA_VERSION={}\n", config.schema_version));
    shell.push_str(&format!(
        "MEMD_BUNDLE_BACKEND_PROVIDER={}\n",
        rag.provider
    ));
    shell.push_str(&format!(
        "MEMD_BUNDLE_BACKEND_ENABLED={}\n",
        if rag.enabled { "true" } else { "false" }
    ));
    if let Some(url) = rag.url.as_deref() {
        shell.push_str(&format!("MEMD_RAG_URL={url}\n"));
    }
    fs::write(&backend_env, shell)
        .with_context(|| format!("write {}", backend_env.display()))?;

    let mut ps1 = String::new();
    ps1.push_str(&format!(
        "$env:MEMD_BUNDLE_SCHEMA_VERSION = \"{}\"\n",
        config.schema_version
    ));
    ps1.push_str(&format!(
        "$env:MEMD_BUNDLE_BACKEND_PROVIDER = \"{}\"\n",
        escape_ps1(&rag.provider)
    ));
    ps1.push_str(&format!(
        "$env:MEMD_BUNDLE_BACKEND_ENABLED = \"{}\"\n",
        if rag.enabled { "true" } else { "false" }
    ));
    if let Some(url) = rag.url.as_deref() {
        ps1.push_str(&format!(
            "$env:MEMD_RAG_URL = \"{}\"\n",
            escape_ps1(url)
        ));
    }
    fs::write(&backend_env_ps1, ps1)
        .with_context(|| format!("write {}", backend_env_ps1.display()))?;

    Ok(())
}

async fn read_bundle_status(output: &Path, base_url: &str) -> anyhow::Result<serde_json::Value> {
    let client = MemdClient::new(base_url)?;
    let health = client.healthz().await.ok();
    let rag_config = read_bundle_rag_config(output)?;
    let rag = match rag_config {
        Some(config) if config.enabled => {
            let source = config.source;
            let Some(url) = config.url.clone() else {
                return Ok(serde_json::json!({
                    "bundle": output,
                    "exists": output.exists(),
                    "config": output.join("config.json").exists(),
                    "env": output.join("env").exists(),
                    "env_ps1": output.join("env.ps1").exists(),
                    "hooks": output.join("hooks").exists(),
                    "agents": output.join("agents").exists(),
                    "server": health,
                    "rag": {
                        "configured": false,
                        "enabled": true,
                        "healthy": false,
                        "error": "rag backend enabled but no url configured",
                        "source": source,
                    },
                }));
            };
            let rag_result = RagClient::new(url.as_str())?.healthz().await;
            Some(match rag_result {
                Ok(health) => serde_json::json!({
                    "configured": true,
                    "enabled": true,
                    "url": url,
                    "healthy": true,
                    "health": health,
                    "source": source,
                }),
                Err(error) => serde_json::json!({
                    "configured": true,
                    "enabled": true,
                    "url": url,
                    "healthy": false,
                    "error": error.to_string(),
                    "source": source,
                }),
            })
        }
        Some(config) => Some(serde_json::json!({
            "configured": config.configured,
            "enabled": false,
            "url": config.url,
            "healthy": null,
            "source": config.source,
        })),
        None => None,
    };
    Ok(serde_json::json!({
        "bundle": output,
        "exists": output.exists(),
        "config": output.join("config.json").exists(),
        "env": output.join("env").exists(),
        "env_ps1": output.join("env.ps1").exists(),
        "hooks": output.join("hooks").exists(),
        "agents": output.join("agents").exists(),
        "server": health,
        "rag": rag.unwrap_or_else(|| serde_json::json!({
            "configured": false,
            "enabled": false,
            "healthy": null,
        })),
    }))
}

fn read_bundle_rag_config(output: &Path) -> anyhow::Result<Option<BundleRagConfigState>> {
    let config_path = output.join("config.json");
    if !config_path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let config: BundleConfigFile =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
    Ok(resolve_bundle_rag_config(config))
}

fn render_attach_snippet(shell: &str, bundle_path: &Path) -> anyhow::Result<String> {
    let shell = shell.trim().to_ascii_lowercase();
    match shell.as_str() {
        "bash" | "zsh" | "sh" => Ok(format!(
            r#"export MEMD_BUNDLE_ROOT="{bundle_path}"
source "$MEMD_BUNDLE_ROOT/env"
memd hook context --project "$MEMD_PROJECT" --agent "$MEMD_AGENT" --route "$MEMD_ROUTE" --intent "$MEMD_INTENT"
"#,
            bundle_path = bundle_path.display(),
        )),
        "powershell" | "pwsh" => Ok(format!(
            r#"$env:MEMD_BUNDLE_ROOT = "{bundle_path}"
. (Join-Path $env:MEMD_BUNDLE_ROOT "env.ps1")
memd hook context --project $env:MEMD_PROJECT --agent $env:MEMD_AGENT --route $env:MEMD_ROUTE --intent $env:MEMD_INTENT
"#,
            bundle_path = escape_ps1(&bundle_path.display().to_string()),
        )),
        other => anyhow::bail!(
            "unsupported shell '{other}'; expected bash, zsh, sh, powershell, or pwsh"
        ),
    }
}

fn detect_shell() -> Option<String> {
    std::env::var("SHELL")
        .ok()
        .and_then(|shell| {
            let shell = shell.rsplit('/').next()?.to_string();
            Some(shell)
        })
        .or_else(|| {
            std::env::var("PSModulePath")
                .ok()
                .map(|_| "powershell".to_string())
        })
}

fn copy_hook_assets(target: &Path) -> anyhow::Result<()> {
    let source_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("integrations")
        .join("hooks");

    for file in [
        "README.md",
        "install.sh",
        "install.ps1",
        "memd-context.sh",
        "memd-context.ps1",
        "memd-spill.sh",
        "memd-spill.ps1",
    ] {
        let src = source_dir.join(file);
        let dst = target.join(file);
        fs::copy(&src, &dst)
            .with_context(|| format!("copy {} to {}", src.display(), dst.display()))?;
        set_executable_if_shell_script(&dst, file)?;
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize)]
struct BundleConfig {
    schema_version: u32,
    project: String,
    agent: String,
    base_url: String,
    route: String,
    intent: String,
    backend: BundleBackendConfig,
    hooks: BundleHooksConfig,
    rag_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct BundleBackendConfig {
    rag: BundleRagConfig,
}

#[derive(Debug, Clone, Serialize)]
struct BundleRagConfig {
    enabled: bool,
    provider: String,
    url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct BundleHooksConfig {
    context: String,
    spill: String,
    context_ps1: String,
    spill_ps1: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct BundleConfigFile {
    #[serde(default)]
    rag_url: Option<String>,
    #[serde(default)]
    backend: Option<BundleBackendConfigFile>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct BundleBackendConfigFile {
    #[serde(default)]
    rag: Option<BundleRagConfigFile>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct BundleRagConfigFile {
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    url: Option<String>,
}

#[derive(Debug, Clone)]
struct BundleRagConfigState {
    configured: bool,
    enabled: bool,
    url: Option<String>,
    source: String,
}

fn resolve_bundle_rag_config(config: BundleConfigFile) -> Option<BundleRagConfigState> {
    if let Some(rag) = config.backend.and_then(|backend| backend.rag) {
        let url = rag
            .url
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let enabled = rag.enabled.unwrap_or(url.is_some());
        let configured = url.is_some();
        return Some(BundleRagConfigState {
            configured,
            enabled,
            url,
            source: "backend.rag".to_string(),
        });
    }

    let url = config
        .rag_url
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    if let Some(url) = url {
        return Some(BundleRagConfigState {
            configured: true,
            enabled: true,
            url: Some(url),
            source: "rag_url".to_string(),
        });
    }

    None
}

fn escape_ps1(value: &str) -> String {
    value.replace('\"', "`\"")
}

fn compact_bundle_value(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn set_executable_if_shell_script(path: &Path, file_name: &str) -> anyhow::Result<()> {
    if !file_name.ends_with(".sh") {
        return Ok(());
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let metadata = fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions)
            .with_context(|| format!("chmod +x {}", path.display()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_nested_bundle_rag_config() {
        let config = BundleConfigFile {
            rag_url: None,
            backend: Some(BundleBackendConfigFile {
                rag: Some(BundleRagConfigFile {
                    enabled: Some(true),
                    url: Some("http://127.0.0.1:9000".to_string()),
                }),
            }),
        };

        let resolved = resolve_bundle_rag_config(config).expect("bundle rag config");
        assert!(resolved.enabled);
        assert!(resolved.configured);
        assert_eq!(resolved.url.as_deref(), Some("http://127.0.0.1:9000"));
        assert_eq!(resolved.source, "backend.rag");
    }

    #[test]
    fn resolves_legacy_bundle_rag_url() {
        let config = BundleConfigFile {
            rag_url: Some("http://127.0.0.1:9000".to_string()),
            backend: None,
        };

        let resolved = resolve_bundle_rag_config(config).expect("bundle rag config");
        assert!(resolved.enabled);
        assert!(resolved.configured);
        assert_eq!(resolved.url.as_deref(), Some("http://127.0.0.1:9000"));
        assert_eq!(resolved.source, "rag_url");
    }

    #[test]
    fn serializes_bundle_config_with_nested_rag_state() {
        let config = BundleConfig {
            schema_version: 2,
            project: "demo".to_string(),
            agent: "codex".to_string(),
            base_url: "http://127.0.0.1:8787".to_string(),
            route: "auto".to_string(),
            intent: "general".to_string(),
            backend: BundleBackendConfig {
                rag: BundleRagConfig {
                    enabled: true,
                    provider: "lightrag-compatible".to_string(),
                    url: Some("http://127.0.0.1:9000".to_string()),
                },
            },
            hooks: BundleHooksConfig {
                context: "hooks/memd-context.sh".to_string(),
                spill: "hooks/memd-spill.sh".to_string(),
                context_ps1: "hooks/memd-context.ps1".to_string(),
                spill_ps1: "hooks/memd-spill.ps1".to_string(),
            },
            rag_url: Some("http://127.0.0.1:9000".to_string()),
        };

        let json = serde_json::to_value(config).expect("serialize bundle config");
        assert_eq!(json["schema_version"], 2);
        assert_eq!(json["backend"]["rag"]["enabled"], true);
        assert_eq!(json["backend"]["rag"]["provider"], "lightrag-compatible");
        assert_eq!(json["backend"]["rag"]["url"], "http://127.0.0.1:9000");
        assert_eq!(json["rag_url"], "http://127.0.0.1:9000");
    }
}
