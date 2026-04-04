mod obsidian;

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
    AssociativeRecallRequest, AssociativeRecallResponse, CandidateMemoryRequest,
    CompactionDecision, CompactionOpenLoop, CompactionPacket, CompactionReference,
    CompactionSession, CompactionSpillOptions, CompactionSpillResult, ContextRequest,
    EntityLinkRequest, EntityLinksRequest, EntitySearchRequest, EntitySearchResponse,
    ExpireMemoryRequest, ExplainMemoryRequest, MemoryConsolidationRequest, MemoryInboxRequest,
    MemoryKind, MemoryMaintenanceReportRequest, MemoryMaintenanceReportResponse, MemoryScope,
    MemoryStage, MemoryStatus, PromoteMemoryRequest, RetrievalIntent, RetrievalRoute,
    SearchMemoryRequest, StoreMemoryRequest, VerifyMemoryRequest, WorkingMemoryRequest,
    WorkingMemoryResponse,
};
use memd_sidecar::{SidecarClient, SidecarIngestRequest, SidecarIngestResponse};
use notify::{Config as NotifyConfig, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use obsidian::{ObsidianImportPreview, ObsidianSyncEntry, ObsidianVaultScan};
use serde::Serialize;

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
    Search(SearchArgs),
    Context(ContextArgs),
    Working(WorkingArgs),
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
struct InboxArgs {
    #[arg(long)]
    project: Option<String>,

    #[arg(long)]
    namespace: Option<String>,

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
    route: Option<String>,

    #[arg(long)]
    intent: Option<String>,
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
            let rag_url = args
                .rag_url
                .or_else(|| std::env::var("MEMD_RAG_URL").ok())
                .context("provide --rag-url or set MEMD_RAG_URL")?;
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
            let rag_url = args
                .rag_url
                .or_else(|| std::env::var("MEMD_RAG_URL").ok())
                .context("provide --rag-url or set MEMD_RAG_URL")?;
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
        Commands::Search(args) => {
            let mut req = read_request::<SearchMemoryRequest>(&args.input)?;
            if args.route.is_some() || args.intent.is_some() {
                req.route = parse_retrieval_route(args.route)?;
                req.intent = parse_retrieval_intent(args.intent)?;
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
                    auto_consolidate: Some(args.auto_consolidate),
                })
                .await?;
            if args.summary {
                println!("{}", render_working_summary(&response, args.follow));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Inbox(args) => {
            let req = MemoryInboxRequest {
                project: args.project.clone(),
                namespace: args.namespace.clone(),
                route: parse_retrieval_route(args.route.clone())?,
                intent: parse_retrieval_intent(args.intent.clone())?,
                limit: args.limit,
            };
            print_json(&client.inbox(&req).await?)?;
        }
        Commands::Explain(args) => {
            let req = ExplainMemoryRequest {
                id: args.id.parse().context("parse memory id as uuid")?,
                route: parse_retrieval_route(args.route.clone())?,
                intent: parse_retrieval_intent(args.intent.clone())?,
            };
            print_json(&client.explain(&req).await?)?;
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
                    if let Some(rag) = maybe_rag_client_from_env()? {
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
            ObsidianMode::Writeback => {
                run_obsidian_writeback(&client, &args).await?;
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
                    if let Some(rag) = maybe_rag_client_from_env()? {
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
    let (preview, candidates, changed_notes) =
        obsidian::build_import_preview(scan, &sync_state, state_path.clone());
    let (attachment_assets, attachment_unchanged_count) = if include_attachments {
        obsidian::partition_changed_attachments(&preview.scan.attachments, &sync_state)
    } else {
        (Vec::new(), 0)
    };

    if apply {
        let responses = client.candidate_batch(&candidates).await?;
        let mut next_state = sync_state.clone();
        for (note, response) in changed_notes.iter().zip(&responses) {
            let stored_id = response.duplicate_of.unwrap_or(response.item.id);
            let entity = client
                .entity(&memd_schema::EntityMemoryRequest {
                    id: stored_id,
                    route: None,
                    intent: None,
                    limit: Some(4),
                })
                .await?;
            next_state.entries.insert(
                note.relative_path.clone(),
                ObsidianSyncEntry {
                    content_hash: note.content_hash.clone(),
                    bytes: note.bytes,
                    modified_at: note.modified_at,
                    item_id: Some(stored_id),
                    entity_id: entity.entity.as_ref().map(|entity| entity.id),
                },
            );
        }

        let mut attachment_multimodal = None;
        let mut attachment_submitted = 0usize;
        let mut attachment_duplicates = 0usize;
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
            let rag_url = std::env::var("MEMD_RAG_URL")
                .context("set MEMD_RAG_URL for obsidian attachments")?;
            let sidecar = SidecarClient::new(&rag_url)?;
            let multimodal_responses =
                ingest_multimodal_preview(&sidecar, &multimodal_preview.requests).await?;
            attachment_multimodal = Some(MultimodalIngestOutput {
                preview: multimodal_preview,
                responses: multimodal_responses,
                submitted: attachment_assets.len(),
                dry_run: false,
            });

            let attachment_candidates = attachment_assets
                .iter()
                .zip(
                    attachment_multimodal
                        .as_ref()
                        .into_iter()
                        .flat_map(|output| output.responses.iter()),
                )
                .map(|(asset, response)| {
                    let match_ = obsidian::resolve_attachment_match(
                        asset,
                        &preview.scan.notes,
                        &preview.note_index,
                    );
                    let linked_note = match_
                        .as_ref()
                        .and_then(|association| preview.scan.notes.get(association.note_index));
                    obsidian::build_attachment_request(
                        asset,
                        args.project.clone(),
                        args.namespace.clone(),
                        preview.scan.vault.clone(),
                        linked_note,
                        Some(response.track_id),
                    )
                })
                .collect::<Vec<_>>();
            let attachment_responses = client.candidate_batch(&attachment_candidates).await?;
            attachment_submitted = attachment_responses.len();
            attachment_duplicates = attachment_responses
                .iter()
                .filter(|response| response.duplicate_of.is_some())
                .count();
            for (asset, response) in attachment_assets.iter().zip(&attachment_responses) {
                let stored_id = response.duplicate_of.unwrap_or(response.item.id);
                let entity = client
                    .entity(&memd_schema::EntityMemoryRequest {
                        id: stored_id,
                        route: None,
                        intent: None,
                        limit: Some(4),
                    })
                    .await?;
                next_state.entries.insert(
                    asset.relative_path.clone(),
                    ObsidianSyncEntry {
                        content_hash: asset.content_hash.clone(),
                        bytes: asset.bytes,
                        modified_at: asset.modified_at,
                        item_id: Some(stored_id),
                        entity_id: entity.entity.as_ref().map(|entity| entity.id),
                    },
                );
            }
        }

        for note in &preview.scan.notes {
            if let Some(entry) = next_state.entries.get_mut(&note.relative_path) {
                entry.content_hash = note.content_hash.clone();
                entry.bytes = note.bytes;
                entry.modified_at = note.modified_at;
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
            submitted: responses.len(),
            attachment_submitted,
            duplicates: responses
                .iter()
                .filter(|response| response.duplicate_of.is_some())
                .count(),
            attachment_duplicates,
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
    links_created: usize,
    attachment_links_created: usize,
    mirrored_notes: usize,
    mirrored_attachments: usize,
    attachments: Option<MultimodalIngestOutput>,
    attachment_unchanged_count: usize,
    dry_run: bool,
}

fn render_obsidian_scan_summary(scan: &ObsidianVaultScan, follow: bool) -> String {
    let mut output = format!(
        "vault={} notes={} sensitive={} unchanged={} backlinks={} attachments={} attachment_sensitive={} skipped={} project={}",
        scan.vault.display(),
        scan.note_count,
        scan.sensitive_count,
        scan.unchanged_count,
        scan.backlink_count,
        scan.attachment_count,
        scan.attachment_sensitive_count,
        scan.skipped_count,
        scan.project.as_deref().unwrap_or("none")
    );
    if follow {
        let trail = scan
            .notes
            .iter()
            .take(3)
            .map(|note| note.title.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        if !trail.is_empty() {
            output.push_str(&format!(" trail={trail}"));
        }
    }
    output
}

fn render_obsidian_import_summary(output: &ObsidianImportOutput, follow: bool) -> String {
    let attachment_submitted = output
        .attachments
        .as_ref()
        .map(|attachments| attachments.submitted)
        .unwrap_or(0);
    let mut summary = format!(
        "obsidian_import vault={} notes={} sensitive={} unchanged={} backlinks={} attachments={} attachment_sensitive={} attachment_unchanged={} submitted={} attachment_submitted={} duplicates={} attachment_duplicates={} links={} attachment_links={} mirrored={} mirrored_attachments={} dry_run={}",
        output.preview.scan.vault.display(),
        output.preview.scan.note_count,
        output.preview.scan.sensitive_count,
        output.preview.scan.unchanged_count,
        output.preview.scan.backlink_count,
        output.preview.scan.attachment_count,
        output.preview.scan.attachment_sensitive_count,
        output.attachment_unchanged_count,
        output.submitted,
        attachment_submitted,
        output.duplicates,
        output.attachment_duplicates,
        output.links_created,
        output.attachment_links_created,
        output.mirrored_notes,
        output.mirrored_attachments,
        output.dry_run
    );
    if follow {
        let trail = output
            .preview
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
    if let Some(attachments) = output.attachments.as_ref() {
        summary.push_str(&format!(
            " attachments_submitted={} attachments_dry_run={}",
            attachments.submitted, attachments.dry_run
        ));
    }
    summary
}

fn render_entity_summary(response: &memd_schema::EntityMemoryResponse, follow: bool) -> String {
    let Some(entity) = response.entity.as_ref() else {
        return format!(
            "entity=none route={} intent={}",
            route_label(response.route),
            intent_label(response.intent)
        );
    };

    let state = entity
        .current_state
        .as_deref()
        .map(|value| compact_inline(value, 72))
        .unwrap_or_else(|| "no-state".to_string());
    let last_seen = entity
        .last_seen_at
        .map(|value| value.to_rfc3339())
        .unwrap_or_else(|| "unknown".to_string());

    let mut output = format!(
        "entity={} type={} salience={:.2} rehearsal={} state_v={} last_seen={} state=\"{}\" events={}",
        short_uuid(entity.id),
        entity.entity_type,
        entity.salience_score,
        entity.rehearsal_count,
        entity.state_version,
        last_seen,
        state,
        response.events.len()
    );

    if follow && let Some(event) = response.events.first() {
        output.push_str(&format!(
            " latest={}::{}",
            event.event_type,
            compact_inline(&event.summary, 48)
        ));
    }

    output
}

fn render_entity_search_summary(response: &EntitySearchResponse, follow: bool) -> String {
    let mut output = format!(
        "entity-search query=\"{}\" candidates={} ambiguous={}",
        compact_inline(&response.query, 48),
        response.candidates.len(),
        response.ambiguous
    );

    if let Some(best) = response.best_match.as_ref() {
        output.push_str(&format!(
            " best={} type={} score={:.2} reasons={}",
            short_uuid(best.entity.id),
            best.entity.entity_type,
            best.score,
            compact_inline(&best.reasons.join(","), 64)
        ));
    }

    if follow {
        let trail = response
            .candidates
            .iter()
            .take(3)
            .map(|candidate| {
                format!(
                    "{}:{}:{:.2}",
                    short_uuid(candidate.entity.id),
                    candidate.entity.entity_type,
                    candidate.score
                )
            })
            .collect::<Vec<_>>();
        if !trail.is_empty() {
            output.push_str(&format!(" trail={}", trail.join(" | ")));
        }
    }

    output
}

fn render_recall_summary(response: &AssociativeRecallResponse, follow: bool) -> String {
    let root = response
        .root_entity
        .as_ref()
        .map(|entity| format!("root={} type={}", short_uuid(entity.id), entity.entity_type))
        .unwrap_or_else(|| "root=none".to_string());

    let mut output = format!(
        "recall {} hits={} links={} truncated={}",
        root,
        response.hits.len(),
        response.links.len(),
        response.truncated
    );

    if follow {
        let hit_trail = response
            .hits
            .iter()
            .take(3)
            .map(|hit| {
                format!(
                    "d{}:{}:{}",
                    hit.depth,
                    short_uuid(hit.entity.id),
                    compact_inline(
                        hit.entity
                            .current_state
                            .as_deref()
                            .unwrap_or(&hit.entity.entity_type),
                        28
                    )
                )
            })
            .collect::<Vec<_>>();
        if !hit_trail.is_empty() {
            output.push_str(&format!(" trail={}", hit_trail.join(" | ")));
        }

        let link_trail = response
            .links
            .iter()
            .take(3)
            .map(|link| {
                format!(
                    "{}:{}->{}",
                    format!("{:?}", link.relation_kind).to_ascii_lowercase(),
                    short_uuid(link.from_entity_id),
                    short_uuid(link.to_entity_id)
                )
            })
            .collect::<Vec<_>>();
        if !link_trail.is_empty() {
            output.push_str(&format!(" links={}", link_trail.join(" | ")));
        }
    }

    output
}

fn render_timeline_summary(response: &memd_schema::TimelineMemoryResponse, follow: bool) -> String {
    let entity = response
        .entity
        .as_ref()
        .map(|entity| {
            format!(
                "entity={} type={}",
                short_uuid(entity.id),
                entity.entity_type
            )
        })
        .unwrap_or_else(|| "entity=none".to_string());
    let latest = response
        .events
        .first()
        .map(|event| {
            format!(
                "{}:{}",
                event.event_type,
                compact_inline(&event.summary, 56)
            )
        })
        .unwrap_or_else(|| "no-events".to_string());

    let mut output = format!(
        "timeline {} route={} intent={} events={} latest={}",
        entity,
        route_label(response.route),
        intent_label(response.intent),
        response.events.len(),
        latest
    );

    if follow {
        let trail = response
            .events
            .iter()
            .take(3)
            .map(|event| {
                format!(
                    "{}:{}",
                    event.event_type,
                    compact_inline(&event.summary, 40)
                )
            })
            .collect::<Vec<_>>();
        if !trail.is_empty() {
            output.push_str(&format!(" trail={}", trail.join(" | ")));
        }
    }

    output
}

fn render_working_summary(response: &WorkingMemoryResponse, follow: bool) -> String {
    let mut output = format!(
        "working route={} intent={} budget={} used={} remaining={} truncated={} records={} traces={} semantic={}",
        route_label(response.route),
        intent_label(response.intent),
        response.budget_chars,
        response.used_chars,
        response.remaining_chars,
        response.truncated,
        response.records.len(),
        response.traces.len(),
        response
            .semantic_consolidation
            .as_ref()
            .map(|value| value.consolidated.to_string())
            .unwrap_or_else(|| "off".to_string())
    );

    if follow {
        let trail = response
            .records
            .iter()
            .take(3)
            .map(|record| compact_inline(&record.record, 48))
            .collect::<Vec<_>>();
        if !trail.is_empty() {
            output.push_str(&format!(" trail={}", trail.join(" | ")));
        }

        let trace_trail = response
            .traces
            .iter()
            .take(3)
            .map(|trace| {
                format!(
                    "{}:{}",
                    trace.event_type,
                    compact_inline(&trace.summary, 40)
                )
            })
            .collect::<Vec<_>>();
        if !trace_trail.is_empty() {
            output.push_str(&format!(" trace_trail={}", trace_trail.join(" | ")));
        }

        if let Some(semantic) = response.semantic_consolidation.as_ref() {
            let trail = semantic
                .highlights
                .iter()
                .take(3)
                .map(|value| compact_inline(value, 40))
                .collect::<Vec<_>>();
            if !trail.is_empty() {
                output.push_str(&format!(" semantic_trail={}", trail.join(" | ")));
            }
        }
    }

    output
}

fn render_consolidate_summary(
    response: &memd_schema::MemoryConsolidationResponse,
    follow: bool,
) -> String {
    let mut output = format!(
        "consolidate scanned={} groups={} consolidated={} duplicates={} events={}",
        response.scanned,
        response.groups,
        response.consolidated,
        response.duplicates,
        response.events
    );

    if follow && !response.highlights.is_empty() {
        let highlights = response
            .highlights
            .iter()
            .take(3)
            .map(|value| compact_inline(value, 40))
            .collect::<Vec<_>>();
        output.push_str(&format!(" trail={}", highlights.join(" | ")));
    }

    output
}

fn render_maintenance_report_summary(
    response: &MemoryMaintenanceReportResponse,
    follow: bool,
) -> String {
    let mut output = format!(
        "learning report reinforced={} cooled={} consolidated={} stale_checked={} skipped={}",
        response.reinforced_candidates,
        response.cooled_candidates,
        response.consolidated_candidates,
        response.stale_items,
        response.skipped
    );

    if follow && !response.highlights.is_empty() {
        let highlights = response
            .highlights
            .iter()
            .take(3)
            .map(|value| compact_inline(value, 40))
            .collect::<Vec<_>>();
        output.push_str(&format!(" trail={}", highlights.join(" | ")));
    }

    output
}

fn compact_inline(value: &str, max_chars: usize) -> String {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.chars().count() <= max_chars {
        return normalized;
    }

    let mut compact = String::with_capacity(max_chars + 3);
    for ch in normalized.chars().take(max_chars.saturating_sub(3)) {
        compact.push(ch);
    }
    compact.push_str("...");
    compact
}

fn short_uuid(id: uuid::Uuid) -> String {
    id.to_string().chars().take(8).collect()
}

fn route_label(route: RetrievalRoute) -> &'static str {
    match route {
        RetrievalRoute::Auto => "auto",
        RetrievalRoute::LocalOnly => "local_only",
        RetrievalRoute::SyncedOnly => "synced_only",
        RetrievalRoute::ProjectOnly => "project_only",
        RetrievalRoute::GlobalOnly => "global_only",
        RetrievalRoute::LocalFirst => "local_first",
        RetrievalRoute::SyncedFirst => "synced_first",
        RetrievalRoute::ProjectFirst => "project_first",
        RetrievalRoute::GlobalFirst => "global_first",
        RetrievalRoute::All => "all",
    }
}

fn intent_label(intent: RetrievalIntent) -> &'static str {
    match intent {
        RetrievalIntent::General => "general",
        RetrievalIntent::CurrentTask => "current_task",
        RetrievalIntent::Decision => "decision",
        RetrievalIntent::Runbook => "runbook",
        RetrievalIntent::Topology => "topology",
        RetrievalIntent::Preference => "preference",
        RetrievalIntent::Fact => "fact",
        RetrievalIntent::Pattern => "pattern",
    }
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
                MemoryKind::Topology,
                MemoryKind::Status,
                MemoryKind::Pattern,
                MemoryKind::Constraint,
            ],
            statuses: vec![MemoryStatus::Active],
            project: args.project.clone(),
            namespace: args.namespace.clone(),
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
        let rag_url =
            std::env::var("MEMD_RAG_URL").context("set MEMD_RAG_URL for multimodal ingest")?;
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

fn parse_uuid_list(values: &[String]) -> anyhow::Result<Vec<uuid::Uuid>> {
    values
        .iter()
        .map(|value| value.parse::<uuid::Uuid>().context("parse uuid"))
        .collect()
}

fn parse_memory_kind_value(value: &str) -> anyhow::Result<MemoryKind> {
    let normalized = value.trim().to_ascii_lowercase().replace('-', "_");
    match normalized.as_str() {
        "fact" => Ok(MemoryKind::Fact),
        "decision" => Ok(MemoryKind::Decision),
        "preference" => Ok(MemoryKind::Preference),
        "runbook" => Ok(MemoryKind::Runbook),
        "topology" => Ok(MemoryKind::Topology),
        "status" => Ok(MemoryKind::Status),
        "pattern" => Ok(MemoryKind::Pattern),
        "constraint" => Ok(MemoryKind::Constraint),
        _ => anyhow::bail!(
            "invalid memory kind '{value}'; expected fact, decision, preference, runbook, topology, status, pattern, or constraint"
        ),
    }
}

fn parse_memory_scope_value(value: &str) -> anyhow::Result<MemoryScope> {
    let normalized = value.trim().to_ascii_lowercase().replace('-', "_");
    match normalized.as_str() {
        "local" => Ok(MemoryScope::Local),
        "synced" => Ok(MemoryScope::Synced),
        "project" => Ok(MemoryScope::Project),
        "global" => Ok(MemoryScope::Global),
        _ => anyhow::bail!("invalid scope '{value}'; expected local, synced, project, or global"),
    }
}

fn parse_source_quality_value(value: &str) -> anyhow::Result<memd_schema::SourceQuality> {
    let normalized = value.trim().to_ascii_lowercase().replace('-', "_");
    match normalized.as_str() {
        "canonical" => Ok(memd_schema::SourceQuality::Canonical),
        "derived" => Ok(memd_schema::SourceQuality::Derived),
        "synthetic" => Ok(memd_schema::SourceQuality::Synthetic),
        _ => anyhow::bail!(
            "invalid source quality '{value}'; expected canonical, derived, or synthetic"
        ),
    }
}

fn parse_entity_relation_kind(value: &str) -> anyhow::Result<memd_schema::EntityRelationKind> {
    let normalized = value.trim().to_ascii_lowercase().replace('-', "_");
    match normalized.as_str() {
        "same_as" | "same" => Ok(memd_schema::EntityRelationKind::SameAs),
        "derived_from" | "derived" => Ok(memd_schema::EntityRelationKind::DerivedFrom),
        "supersedes" => Ok(memd_schema::EntityRelationKind::Supersedes),
        "contradicts" => Ok(memd_schema::EntityRelationKind::Contradicts),
        "related" => Ok(memd_schema::EntityRelationKind::Related),
        _ => anyhow::bail!(
            "invalid relation kind '{value}'; expected same_as, derived_from, supersedes, contradicts, or related"
        ),
    }
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

fn maybe_rag_client_from_env() -> anyhow::Result<Option<RagClient>> {
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

    let config = serde_json::json!({
        "project": args.project,
        "agent": args.agent,
        "base_url": args.base_url,
        "rag_url": args.rag_url,
        "route": args.route,
        "intent": args.intent,
        "hook_kit": {
            "context": "hooks/memd-context.sh",
            "spill": "hooks/memd-spill.sh",
            "context_ps1": "hooks/memd-context.ps1",
            "spill_ps1": "hooks/memd-spill.ps1"
        }
    });
    fs::write(
        output.join("config.json"),
        serde_json::to_string_pretty(&config)? + "\n",
    )
    .with_context(|| format!("write {}", output.join("config.json").display()))?;

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
        "#!/usr/bin/env bash\nset -euo pipefail\n\nexport MEMD_BUNDLE_ROOT=\"{}\"\nsource \"$MEMD_BUNDLE_ROOT/env\"\nexec memd hook context --project \"$MEMD_PROJECT\" --agent \"$MEMD_AGENT\" --route \"$MEMD_ROUTE\" --intent \"$MEMD_INTENT\" \"$@\"\n",
        compact_bundle_value(output.to_string_lossy().as_ref()),
    );
    fs::write(agents_dir.join("agent.sh"), shell_profile)
        .with_context(|| format!("write {}", agents_dir.join("agent.sh").display()))?;
    set_executable_if_shell_script(&agents_dir.join("agent.sh"), "agent.sh")?;

    let ps1_profile = format!(
        "$env:MEMD_BUNDLE_ROOT = \"{}\"\n. (Join-Path $env:MEMD_BUNDLE_ROOT \"env.ps1\")\nmemd hook context --project $env:MEMD_PROJECT --agent $env:MEMD_AGENT --route $env:MEMD_ROUTE --intent $env:MEMD_INTENT\n",
        escape_ps1(output.to_string_lossy().as_ref()),
    );
    fs::write(agents_dir.join("agent.ps1"), ps1_profile)
        .with_context(|| format!("write {}", agents_dir.join("agent.ps1").display()))?;

    Ok(())
}

async fn read_bundle_status(output: &Path, base_url: &str) -> anyhow::Result<serde_json::Value> {
    let client = MemdClient::new(base_url)?;
    let health = client.healthz().await.ok();
    let rag_url = read_bundle_rag_url(output)?;
    let rag = match rag_url {
        Some(ref url) => {
            let client = RagClient::new(url)?;
            Some(
                client
                    .healthz()
                    .await
                    .map(|health| {
                        serde_json::json!({
                            "enabled": true,
                            "url": url,
                            "healthy": true,
                            "health": health,
                        })
                    })
                    .unwrap_or_else(|error| {
                        serde_json::json!({
                            "enabled": true,
                            "url": url,
                            "healthy": false,
                            "error": error.to_string(),
                        })
                    }),
            )
        }
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
            "enabled": false,
            "healthy": null,
        })),
    }))
}

fn read_bundle_rag_url(output: &Path) -> anyhow::Result<Option<String>> {
    let config_path = output.join("config.json");
    if !config_path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let config: serde_json::Value =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;

    let rag_url = config
        .get("rag_url")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    Ok(rag_url)
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

fn parse_retrieval_route(value: Option<String>) -> anyhow::Result<Option<RetrievalRoute>> {
    value
        .map(|value| parse_retrieval_route_value(&value))
        .transpose()
}

fn parse_retrieval_intent(value: Option<String>) -> anyhow::Result<Option<RetrievalIntent>> {
    value
        .map(|value| parse_retrieval_intent_value(&value))
        .transpose()
}

fn parse_retrieval_route_value(value: &str) -> anyhow::Result<RetrievalRoute> {
    let normalized = value.trim().to_ascii_lowercase().replace('-', "_");
    match normalized.as_str() {
        "auto" => Ok(RetrievalRoute::Auto),
        "local_only" | "local" => Ok(RetrievalRoute::LocalOnly),
        "synced_only" | "synced" => Ok(RetrievalRoute::SyncedOnly),
        "project_only" | "project" => Ok(RetrievalRoute::ProjectOnly),
        "global_only" | "global" => Ok(RetrievalRoute::GlobalOnly),
        "local_first" => Ok(RetrievalRoute::LocalFirst),
        "synced_first" => Ok(RetrievalRoute::SyncedFirst),
        "project_first" => Ok(RetrievalRoute::ProjectFirst),
        "global_first" => Ok(RetrievalRoute::GlobalFirst),
        "all" => Ok(RetrievalRoute::All),
        _ => anyhow::bail!(
            "invalid retrieval route '{value}'; expected auto, local_only, synced_only, project_only, global_only, local_first, synced_first, project_first, global_first, or all"
        ),
    }
}

fn parse_retrieval_intent_value(value: &str) -> anyhow::Result<RetrievalIntent> {
    let normalized = value.trim().to_ascii_lowercase().replace('-', "_");
    match normalized.as_str() {
        "general" => Ok(RetrievalIntent::General),
        "current_task" | "task" => Ok(RetrievalIntent::CurrentTask),
        "decision" => Ok(RetrievalIntent::Decision),
        "runbook" => Ok(RetrievalIntent::Runbook),
        "topology" => Ok(RetrievalIntent::Topology),
        "preference" => Ok(RetrievalIntent::Preference),
        "fact" => Ok(RetrievalIntent::Fact),
        "pattern" => Ok(RetrievalIntent::Pattern),
        _ => anyhow::bail!(
            "invalid retrieval intent '{value}'; expected general, current_task, decision, runbook, topology, preference, fact, or pattern"
        ),
    }
}
