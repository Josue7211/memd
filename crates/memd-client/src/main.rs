use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
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
    CandidateMemoryRequest, CompactionDecision, CompactionOpenLoop, CompactionPacket,
    CompactionReference, CompactionSession, CompactionSpillOptions, CompactionSpillResult,
    ContextRequest, ExpireMemoryRequest, ExplainMemoryRequest, MemoryInboxRequest, MemoryKind,
    MemoryScope, MemoryStage, MemoryStatus, PromoteMemoryRequest, RetrievalIntent, RetrievalRoute,
    SearchMemoryRequest, StoreMemoryRequest, VerifyMemoryRequest,
};
use memd_sidecar::{SidecarClient, SidecarIngestRequest, SidecarIngestResponse};
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
    Inbox(InboxArgs),
    Explain(ExplainArgs),
    Compact(CompactArgs),
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
                    json: args.json,
                    input: args.input,
                    stdin: args.stdin,
                })?
            } else {
                ContextRequest {
                    project: args.project,
                    agent: args.agent,
                    route: parse_retrieval_route(args.route)?,
                    intent: parse_retrieval_intent(args.intent)?,
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
        Commands::Inbox(args) => {
            let req = MemoryInboxRequest {
                project: args.project,
                namespace: args.namespace,
                route: parse_retrieval_route(args.route)?,
                intent: parse_retrieval_intent(args.intent)?,
                limit: args.limit,
            };
            print_json(&client.inbox(&req).await?)?;
        }
        Commands::Explain(args) => {
            let req = ExplainMemoryRequest {
                id: args.id.parse().context("parse memory id as uuid")?,
                route: parse_retrieval_route(args.route)?,
                intent: parse_retrieval_intent(args.intent)?,
            };
            print_json(&client.explain(&req).await?)?;
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
