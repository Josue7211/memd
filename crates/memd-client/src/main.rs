mod commands;
mod obsidian;
mod render;

use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use anyhow::Context;
use chrono::{DateTime, Utc};
use clap::{Args, Parser, Subcommand};
use commands::{
    parse_entity_relation_kind, parse_memory_kind_value, parse_memory_scope_value,
    parse_memory_visibility_value, parse_retrieval_intent, parse_retrieval_route,
    parse_source_quality_value, parse_uuid_list,
};
use memd_client::MemdClient;
use memd_core::{
    build_compaction_packet, derive_compaction_spill, derive_compaction_spill_with_options,
    render_compaction_wire,
};
use memd_multimodal::{
    MultimodalChunk, MultimodalIngestPlan, build_ingest_plan, extract_chunks, to_sidecar_requests,
};
use memd_rag::{
    RagClient, RagIngestRequest, RagRetrieveMode, RagRetrieveRequest, RagRetrieveResponse,
};
use memd_schema::{
    AgentProfileRequest, AgentProfileUpsertRequest, AssociativeRecallRequest,
    CandidateMemoryRequest, CompactionDecision, CompactionOpenLoop, CompactionPacket,
    CompactionReference, CompactionSession, CompactionSpillOptions, CompactionSpillResult,
    ContextRequest, EntityLinkRequest, EntityLinksRequest, EntitySearchRequest,
    ExpireMemoryRequest, ExplainMemoryRequest, MemoryConsolidationRequest, MemoryInboxRequest,
    MemoryKind, MemoryMaintenanceReportRequest, MemoryScope, MemoryStage, MemoryStatus,
    PeerClaimRecoverRequest, PeerClaimsRequest, PeerCoordinationInboxRequest,
    PeerCoordinationInboxResponse, PeerCoordinationReceiptRecord, PeerCoordinationReceiptRequest,
    PeerCoordinationReceiptsRequest, PeerMessageAckRequest, PeerMessageInboxRequest,
    PeerMessageRecord, PeerMessageSendRequest, PeerTaskAssignRequest, PeerTaskRecord,
    PeerTaskUpsertRequest, PeerTasksRequest, PromoteMemoryRequest, RepairMemoryRequest,
    RetrievalIntent, RetrievalRoute, SearchMemoryRequest, SourceMemoryRequest, StoreMemoryRequest,
    VerifyMemoryRequest, WorkingMemoryRequest,
};
use memd_sidecar::{SidecarClient, SidecarIngestRequest, SidecarIngestResponse};
use notify::{Config as NotifyConfig, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use obsidian::{ObsidianImportPreview, ObsidianSyncEntry};
use render::{
    render_consolidate_summary, render_entity_search_summary, render_entity_summary,
    render_eval_summary, render_explain_summary, render_gap_summary, render_handoff_prompt,
    render_improvement_markdown, render_improvement_summary, render_maintenance_report_summary,
    render_obsidian_import_summary, render_obsidian_scan_summary, render_profile_summary,
    render_recall_summary, render_repair_summary, render_resume_prompt, render_source_summary,
    render_timeline_summary, render_working_summary, render_workspace_summary, short_uuid,
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
    Awareness(AwarenessArgs),
    Heartbeat(HeartbeatArgs),
    Claims(ClaimsArgs),
    Messages(MessagesArgs),
    Tasks(TasksArgs),
    Coordination(CoordinationArgs),
    Bundle(BundleArgs),
    Eval(EvalArgs),
    Gap(GapArgs),
    Improve(ImproveArgs),
    Agent(AgentArgs),
    Attach(AttachArgs),
    Resume(ResumeArgs),
    Handoff(HandoffArgs),
    Checkpoint(CheckpointArgs),
    Remember(RememberArgs),
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
    Workspaces(SourceArgs),
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
    workspace: Option<String>,

    #[arg(long)]
    visibility: Option<String>,

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
    workspace: Option<String>,

    #[arg(long)]
    visibility: Option<String>,

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
    workspace: Option<String>,

    #[arg(long)]
    visibility: Option<String>,

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
    workspace: Option<String>,

    #[arg(long)]
    visibility: Option<String>,

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
    workspace: Option<String>,

    #[arg(long)]
    visibility: Option<String>,

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
    workspace: Option<String>,

    #[arg(long)]
    visibility: Option<String>,

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
    Handoff,
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
    workspace: Option<String>,

    #[arg(long)]
    visibility: Option<String>,

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
    workspace: Option<String>,

    #[arg(long)]
    visibility: Option<String>,

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
    namespace: Option<String>,

    #[arg(long)]
    agent: String,

    #[arg(long)]
    session: Option<String>,

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
    workspace: Option<String>,

    #[arg(long)]
    visibility: Option<String>,

    #[arg(long)]
    force: bool,
}

#[derive(Debug, Clone, Args)]
struct StatusArgs {
    #[arg(long, default_value = ".memd")]
    output: PathBuf,
}

#[derive(Debug, Clone, Args)]
struct AwarenessArgs {
    #[arg(long, default_value = ".memd")]
    output: PathBuf,

    #[arg(long)]
    root: Option<PathBuf>,

    #[arg(long, default_value_t = false)]
    include_current: bool,

    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct HeartbeatArgs {
    #[arg(long, default_value = ".memd")]
    output: PathBuf,

    #[arg(long)]
    watch: bool,

    #[arg(long, default_value_t = 30)]
    interval_secs: u64,

    #[arg(long)]
    probe_base_url: bool,

    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct ClaimsArgs {
    #[arg(long, default_value = ".memd")]
    output: PathBuf,

    #[arg(long)]
    acquire: bool,

    #[arg(long)]
    release: bool,

    #[arg(long)]
    transfer_to_session: Option<String>,

    #[arg(long)]
    scope: Option<String>,

    #[arg(long, default_value_t = 900)]
    ttl_secs: u64,

    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct MessagesArgs {
    #[arg(long, default_value = ".memd")]
    output: PathBuf,

    #[arg(long)]
    send: bool,

    #[arg(long)]
    inbox: bool,

    #[arg(long)]
    ack: Option<String>,

    #[arg(long)]
    target_session: Option<String>,

    #[arg(long)]
    kind: Option<String>,

    #[arg(long)]
    request_help: bool,

    #[arg(long)]
    request_review: bool,

    #[arg(long)]
    assign_scope: Option<String>,

    #[arg(long)]
    scope: Option<String>,

    #[arg(long)]
    content: Option<String>,

    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct TasksArgs {
    #[arg(long, default_value = ".memd")]
    output: PathBuf,

    #[arg(long)]
    upsert: bool,

    #[arg(long)]
    assign_to_session: Option<String>,

    #[arg(long)]
    target_session: Option<String>,

    #[arg(long)]
    task_id: Option<String>,

    #[arg(long)]
    title: Option<String>,

    #[arg(long)]
    description: Option<String>,

    #[arg(long)]
    status: Option<String>,

    #[arg(long)]
    mode: Option<String>,

    #[arg(long, value_name = "SCOPE")]
    scope: Vec<String>,

    #[arg(long)]
    request_help: bool,

    #[arg(long)]
    request_review: bool,

    #[arg(long)]
    all: bool,

    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct CoordinationArgs {
    #[arg(long, default_value = ".memd")]
    output: PathBuf,

    #[arg(long)]
    view: Option<String>,

    #[arg(long)]
    changes_only: bool,

    #[arg(long)]
    watch: bool,

    #[arg(long, default_value_t = 30)]
    interval_secs: u64,

    #[arg(long)]
    recover_session: Option<String>,

    #[arg(long)]
    to_session: Option<String>,

    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct BundleArgs {
    #[arg(long, default_value = ".memd")]
    output: PathBuf,

    #[arg(long)]
    base_url: Option<String>,

    #[arg(long)]
    auto_short_term_capture: Option<bool>,

    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct EvalArgs {
    #[arg(long, default_value = ".memd")]
    output: PathBuf,

    #[arg(long)]
    limit: Option<usize>,

    #[arg(long)]
    rehydration_limit: Option<usize>,

    #[arg(long)]
    write: bool,

    #[arg(long)]
    fail_below: Option<u8>,

    #[arg(long)]
    fail_on_regression: bool,

    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct GapArgs {
    #[arg(long, default_value = ".memd")]
    output: PathBuf,

    #[arg(long)]
    limit: Option<usize>,

    #[arg(long)]
    recent_commits: Option<usize>,

    #[arg(long, default_value_t = false)]
    write: bool,

    #[arg(long, default_value_t = false)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct ImproveArgs {
    #[arg(long, default_value = ".memd")]
    output: PathBuf,

    #[arg(long, default_value_t = 3)]
    max_iterations: usize,

    #[arg(long)]
    limit: Option<usize>,

    #[arg(long)]
    recent_commits: Option<usize>,

    #[arg(long, default_value_t = false)]
    write: bool,

    #[arg(long, default_value_t = false)]
    apply: bool,

    #[arg(long, default_value_t = false)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct AttachArgs {
    #[arg(long, default_value = ".memd")]
    output: PathBuf,

    #[arg(long)]
    shell: Option<String>,
}

#[derive(Debug, Clone, Args)]
struct AgentArgs {
    #[arg(long, default_value = ".memd")]
    output: PathBuf,

    #[arg(long)]
    name: Option<String>,

    #[arg(long)]
    shell: Option<String>,

    #[arg(long)]
    session: Option<String>,

    #[arg(long)]
    apply: bool,

    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct ResumeArgs {
    #[arg(long, default_value = ".memd")]
    output: PathBuf,

    #[arg(long)]
    project: Option<String>,

    #[arg(long)]
    namespace: Option<String>,

    #[arg(long)]
    agent: Option<String>,

    #[arg(long)]
    workspace: Option<String>,

    #[arg(long)]
    visibility: Option<String>,

    #[arg(long)]
    route: Option<String>,

    #[arg(long)]
    intent: Option<String>,

    #[arg(long)]
    limit: Option<usize>,

    #[arg(long)]
    rehydration_limit: Option<usize>,

    #[arg(long)]
    semantic: bool,

    #[arg(long)]
    prompt: bool,

    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct HandoffArgs {
    #[arg(long, default_value = ".memd")]
    output: PathBuf,

    #[arg(long)]
    target_session: Option<String>,

    #[arg(long)]
    project: Option<String>,

    #[arg(long)]
    namespace: Option<String>,

    #[arg(long)]
    agent: Option<String>,

    #[arg(long)]
    workspace: Option<String>,

    #[arg(long)]
    visibility: Option<String>,

    #[arg(long)]
    route: Option<String>,

    #[arg(long)]
    intent: Option<String>,

    #[arg(long)]
    limit: Option<usize>,

    #[arg(long)]
    rehydration_limit: Option<usize>,

    #[arg(long)]
    source_limit: Option<usize>,

    #[arg(long)]
    semantic: bool,

    #[arg(long)]
    prompt: bool,

    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct RememberArgs {
    #[arg(long, default_value = ".memd")]
    output: PathBuf,

    #[arg(long)]
    project: Option<String>,

    #[arg(long)]
    namespace: Option<String>,

    #[arg(long)]
    workspace: Option<String>,

    #[arg(long)]
    visibility: Option<String>,

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

    #[arg(long)]
    content: Option<String>,

    #[arg(long)]
    input: Option<PathBuf>,

    #[arg(long)]
    stdin: bool,
}

#[derive(Debug, Clone, Args)]
struct CheckpointArgs {
    #[arg(long, default_value = ".memd")]
    output: PathBuf,

    #[arg(long)]
    project: Option<String>,

    #[arg(long)]
    namespace: Option<String>,

    #[arg(long)]
    workspace: Option<String>,

    #[arg(long)]
    visibility: Option<String>,

    #[arg(long)]
    source_path: Option<String>,

    #[arg(long)]
    confidence: Option<f32>,

    #[arg(long)]
    ttl_seconds: Option<u64>,

    #[arg(long, value_name = "TEXT")]
    tag: Vec<String>,

    #[arg(long)]
    content: Option<String>,

    #[arg(long)]
    input: Option<PathBuf>,

    #[arg(long)]
    stdin: bool,
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
        Commands::Awareness(args) => {
            let response = read_project_awareness(&args)?;
            if args.summary {
                println!("{}", render_project_awareness_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Heartbeat(args) => {
            if args.watch {
                let interval = Duration::from_secs(args.interval_secs.max(1));
                loop {
                    let response =
                        refresh_bundle_heartbeat(&args.output, None, args.probe_base_url).await?;
                    if args.summary {
                        println!("{}", render_bundle_heartbeat_summary(&response));
                    } else {
                        print_json(&response)?;
                    }
                    tokio::time::sleep(interval).await;
                }
            } else {
                let response =
                    refresh_bundle_heartbeat(&args.output, None, args.probe_base_url).await?;
                if args.summary {
                    println!("{}", render_bundle_heartbeat_summary(&response));
                } else {
                    print_json(&response)?;
                }
            }
        }
        Commands::Claims(args) => {
            let response = run_claims_command(&args, &base_url).await?;
            if args.summary {
                println!("{}", render_claims_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Messages(args) => {
            let response = run_messages_command(&args, &base_url).await?;
            if args.summary {
                println!("{}", render_messages_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Tasks(args) => {
            let response = run_tasks_command(&args, &base_url).await?;
            if args.summary {
                println!("{}", render_tasks_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Coordination(args) => {
            if args.watch {
                let interval = Duration::from_secs(args.interval_secs.max(1));
                let mut previous: Option<CoordinationResponse> = None;
                loop {
                    let response = run_coordination_command(&args, &base_url).await?;
                    if args.summary {
                        let alerts = render_coordination_alerts(
                            previous.as_ref(),
                            &response,
                            args.view.as_deref(),
                        );
                        if previous.is_none() || !alerts.is_empty() {
                            println!("[{}]", Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
                            for line in alerts {
                                println!("{line}");
                            }
                            println!(
                                "{}",
                                render_coordination_summary(&response, args.view.as_deref())
                            );
                            println!();
                        }
                    } else {
                        print_json(&response)?;
                    }
                    previous = Some(response);
                    tokio::time::sleep(interval).await;
                }
            } else if args.changes_only {
                let response = run_coordination_command(&args, &base_url).await?;
                let changes = build_coordination_change_response(
                    &args.output,
                    &response,
                    args.view.as_deref(),
                )?;
                if args.summary {
                    println!("{}", render_coordination_change_summary(&changes));
                } else {
                    print_json(&changes)?;
                }
            } else {
                let response = run_coordination_command(&args, &base_url).await?;
                if args.summary {
                    println!(
                        "{}",
                        render_coordination_summary(&response, args.view.as_deref())
                    );
                } else {
                    print_json(&response)?;
                }
            }
        }
        Commands::Bundle(args) => {
            if let Some(value) = args.base_url.as_deref() {
                set_bundle_base_url(&args.output, value)?;
            }
            if let Some(value) = args.auto_short_term_capture {
                set_bundle_auto_short_term_capture(&args.output, value)?;
            }
            let status = read_bundle_status(&args.output, &base_url).await?;
            if args.summary {
                let base_url = status
                    .get("defaults")
                    .and_then(|value| value.get("base_url"))
                    .and_then(|value| value.as_str())
                    .unwrap_or("none");
                let enabled = status
                    .get("defaults")
                    .and_then(|value| value.get("auto_short_term_capture"))
                    .and_then(|value| value.as_bool())
                    .unwrap_or(true);
                println!(
                    "bundle={} base_url={} auto_short_term_capture={}",
                    args.output.display(),
                    base_url,
                    if enabled { "true" } else { "false" }
                );
            } else {
                print_json(&status)?;
            }
        }
        Commands::Eval(args) => {
            let response = eval_bundle_memory(&args, &base_url).await?;
            if args.write {
                write_bundle_eval_artifacts(&args.output, &response)?;
            }
            if args.summary {
                println!("{}", render_eval_summary(&response));
            } else {
                print_json(&response)?;
            }
            if let Some(reason) =
                eval_failure_reason(&response, args.fail_below, args.fail_on_regression)
            {
                anyhow::bail!(reason);
            }
        }
        Commands::Gap(args) => {
            let response = gap_report(&args).await?;
            if args.write {
                write_gap_artifacts(&args.output, &response)?;
            }
            if args.summary {
                println!("{}", render_gap_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Improve(args) => {
            let response = run_improvement_loop(&args, &base_url).await?;
            if args.write {
                write_improvement_artifacts(&args.output, &response)?;
            }
            if args.summary {
                println!("{}", render_improvement_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Agent(args) => {
            if args.apply {
                let Some(name) = args.name.as_deref() else {
                    anyhow::bail!("memd agent --apply requires --name <agent>");
                };
                set_bundle_agent(&args.output, name)?;
                if let Some(session) = args.session.as_deref() {
                    set_bundle_session(&args.output, session)?;
                }
                let snapshot = read_bundle_resume(
                    &ResumeArgs {
                        output: args.output.clone(),
                        project: None,
                        namespace: None,
                        agent: Some(name.to_string()),
                        workspace: None,
                        visibility: None,
                        route: None,
                        intent: None,
                        limit: Some(8),
                        rehydration_limit: Some(4),
                        semantic: false,
                        prompt: false,
                        summary: false,
                    },
                    &base_url,
                )
                .await?;
                write_bundle_memory_files(&args.output, &snapshot, None).await?;
            } else if let Some(session) = args.session.as_deref() {
                set_bundle_session(&args.output, session)?;
            }
            let response = build_bundle_agent_profiles(
                &args.output,
                args.name.as_deref(),
                args.shell.as_deref(),
            )?;
            if args.summary {
                println!("{}", render_bundle_agent_profiles_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Attach(args) => {
            let shell = args
                .shell
                .or_else(|| detect_shell())
                .unwrap_or_else(|| "bash".to_string());
            println!("{}", render_attach_snippet(&shell, &args.output)?);
        }
        Commands::Resume(args) => {
            let snapshot = read_bundle_resume(&args, &base_url).await?;
            write_bundle_memory_files(&args.output, &snapshot, None).await?;
            if args.prompt {
                println!("{}", render_resume_prompt(&snapshot));
            } else if args.summary {
                let focus = snapshot
                    .working
                    .records
                    .first()
                    .map(|record| compact_inline(&record.record, 72))
                    .unwrap_or_else(|| "none".to_string());
                let pressure = snapshot
                    .inbox
                    .items
                    .first()
                    .map(|item| compact_inline(&item.item.content, 72))
                    .unwrap_or_else(|| "none".to_string());
                println!(
                    "resume project={} namespace={} agent={} workspace={} visibility={} context={} working={} inbox={} workspaces={} changes={} est_tokens={} context_pressure={} redundant_items={} refresh_recommended={} focus=\"{}\" pressure=\"{}\"",
                    snapshot.project.as_deref().unwrap_or("none"),
                    snapshot.namespace.as_deref().unwrap_or("none"),
                    snapshot.agent.as_deref().unwrap_or("none"),
                    snapshot.workspace.as_deref().unwrap_or("none"),
                    snapshot.visibility.as_deref().unwrap_or("all"),
                    snapshot.context.records.len(),
                    snapshot.working.records.len(),
                    snapshot.inbox.items.len(),
                    snapshot.workspaces.workspaces.len(),
                    snapshot.change_summary.len(),
                    snapshot.estimated_prompt_tokens(),
                    snapshot.context_pressure(),
                    snapshot.redundant_context_items(),
                    snapshot.refresh_recommended,
                    focus,
                    pressure,
                );
            } else {
                print_json(&snapshot)?;
            }
        }
        Commands::Handoff(args) => {
            let snapshot = read_bundle_handoff(&args, &base_url).await?;
            write_bundle_memory_files(&args.output, &snapshot.resume, Some(&snapshot)).await?;
            if args.prompt {
                println!("{}", render_handoff_prompt(&snapshot));
            } else if args.summary {
                println!(
                    "handoff project={} namespace={} agent={} workspace={} visibility={} working={} inbox={} workspaces={} sources={} rehydration={} target_session={} target_bundle={}",
                    snapshot.resume.project.as_deref().unwrap_or("none"),
                    snapshot.resume.namespace.as_deref().unwrap_or("none"),
                    snapshot.resume.agent.as_deref().unwrap_or("none"),
                    snapshot.resume.workspace.as_deref().unwrap_or("none"),
                    snapshot.resume.visibility.as_deref().unwrap_or("all"),
                    snapshot.resume.working.records.len(),
                    snapshot.resume.inbox.items.len(),
                    snapshot.resume.workspaces.workspaces.len(),
                    snapshot.sources.sources.len(),
                    snapshot.resume.working.rehydration_queue.len(),
                    snapshot.target_session.as_deref().unwrap_or("none"),
                    snapshot.target_bundle.as_deref().unwrap_or("none"),
                );
            } else {
                print_json(&snapshot)?;
            }
        }
        Commands::Checkpoint(args) => {
            let response = checkpoint_with_bundle_defaults(&args, &base_url).await?;
            let snapshot = read_bundle_resume(
                &ResumeArgs {
                    output: args.output.clone(),
                    project: args.project.clone(),
                    namespace: args.namespace.clone(),
                    agent: None,
                    workspace: args.workspace.clone(),
                    visibility: args.visibility.clone(),
                    route: None,
                    intent: Some("current_task".to_string()),
                    limit: Some(8),
                    rehydration_limit: Some(4),
                    semantic: false,
                    prompt: false,
                    summary: false,
                },
                &base_url,
            )
            .await?;
            write_bundle_memory_files(&args.output, &snapshot, None).await?;
            print_json(&response)?;
        }
        Commands::Remember(args) => {
            let response = remember_with_bundle_defaults(&args, &base_url).await?;
            let snapshot = read_bundle_resume(
                &ResumeArgs {
                    output: args.output.clone(),
                    project: args.project.clone(),
                    namespace: args.namespace.clone(),
                    agent: None,
                    workspace: args.workspace.clone(),
                    visibility: args.visibility.clone(),
                    route: None,
                    intent: Some("current_task".to_string()),
                    limit: Some(8),
                    rehydration_limit: Some(4),
                    semantic: false,
                    prompt: false,
                    summary: false,
                },
                &base_url,
            )
            .await?;
            write_bundle_memory_files(&args.output, &snapshot, None).await?;
            print_json(&response)?;
        }
        Commands::Rag(args) => {
            let rag_url = resolve_rag_url(args.rag_url, resolve_default_bundle_root()?.as_deref())?;
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
            let rag_url = resolve_rag_url(args.rag_url, resolve_default_bundle_root()?.as_deref())?;
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
                    workspace: args.workspace.clone(),
                    visibility: args
                        .visibility
                        .as_deref()
                        .map(parse_memory_visibility_value)
                        .transpose()?,
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
            if args.workspace.is_some() {
                req.workspace = args.workspace.clone();
            }
            if let Some(visibility) = args.visibility.as_deref() {
                req.visibility = Some(parse_memory_visibility_value(visibility)?);
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
                    workspace: args.workspace.clone(),
                    visibility: args
                        .visibility
                        .as_deref()
                        .map(parse_memory_visibility_value)
                        .transpose()?,
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
                    workspace: args.workspace.clone(),
                    visibility: args
                        .visibility
                        .as_deref()
                        .map(parse_memory_visibility_value)
                        .transpose()?,
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
                    workspace: args.workspace.clone(),
                    visibility: args
                        .visibility
                        .as_deref()
                        .map(parse_memory_visibility_value)
                        .transpose()?,
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
        Commands::Workspaces(args) => {
            let response = client
                .workspace_memory(&memd_schema::WorkspaceMemoryRequest {
                    project: args.project.clone(),
                    namespace: args.namespace.clone(),
                    workspace: args.workspace.clone(),
                    visibility: args
                        .visibility
                        .as_deref()
                        .map(parse_memory_visibility_value)
                        .transpose()?,
                    source_agent: args.source_agent.clone(),
                    source_system: args.source_system.clone(),
                    limit: args.limit,
                })
                .await?;
            if args.summary {
                println!("{}", render_workspace_summary(&response, args.follow));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Inbox(args) => {
            let req = MemoryInboxRequest {
                project: args.project.clone(),
                namespace: args.namespace.clone(),
                workspace: args.workspace.clone(),
                visibility: args
                    .visibility
                    .as_deref()
                    .map(parse_memory_visibility_value)
                    .transpose()?,
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
                    workspace: None,
                    visibility: None,
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
                    auto_checkpoint_compaction_packet(&packet, &base_url).await?;
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
                    args.workspace.clone(),
                    args.visibility
                        .as_deref()
                        .map(parse_memory_visibility_value)
                        .transpose()?,
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
            ObsidianMode::Handoff => {
                run_obsidian_handoff(&args, &base_url).await?;
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
                    workspace: None,
                    visibility: None,
                    route: parse_retrieval_route(args.route)?,
                    intent: parse_retrieval_intent(
                        args.intent.or(Some("current_task".to_string())),
                    )?,
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
                    auto_checkpoint_compaction_packet(&packet, &base_url).await?;
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
        args.workspace.clone(),
        args.visibility
            .as_deref()
            .map(parse_memory_visibility_value)
            .transpose()?,
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
                args.workspace.clone(),
                args.visibility
                    .as_deref()
                    .map(parse_memory_visibility_value)
                    .transpose()?,
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
                    args.workspace.clone(),
                    args.visibility
                        .as_deref()
                        .map(parse_memory_visibility_value)
                        .transpose()?,
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
                        workspace: args.workspace.clone(),
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
                let block = obsidian::build_roundtrip_annotation(
                    note,
                    Some(item_id),
                    entity_id,
                    args.workspace.as_deref(),
                    args.visibility
                        .as_deref()
                        .map(parse_memory_visibility_value)
                        .transpose()?,
                );
                obsidian::annotate_note(&note.path, &block)?;
                let (_, mirror_markdown) = obsidian::build_note_mirror_markdown(
                    note,
                    Some(item_id),
                    entity_id,
                    args.workspace.as_deref(),
                    args.visibility
                        .as_deref()
                        .map(parse_memory_visibility_value)
                        .transpose()?,
                );
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
                        args.workspace.as_deref(),
                        args.visibility
                            .as_deref()
                            .map(parse_memory_visibility_value)
                            .transpose()?,
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
    let (title, markdown) =
        obsidian::build_writeback_markdown(&args.vault, &explain, explain.entity.as_ref());

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

async fn run_obsidian_handoff(args: &ObsidianArgs, base_url: &str) -> anyhow::Result<()> {
    let snapshot = read_bundle_handoff(
        &HandoffArgs {
            output: resolve_default_bundle_root()?.unwrap_or_else(|| PathBuf::from(".memd")),
            target_session: None,
            project: args.project.clone(),
            namespace: args.namespace.clone(),
            agent: None,
            workspace: args.workspace.clone(),
            visibility: args.visibility.clone(),
            route: args.route.clone(),
            intent: args.intent.clone(),
            limit: args.limit,
            rehydration_limit: Some(4),
            source_limit: Some(6),
            semantic: true,
            prompt: false,
            summary: false,
        },
        base_url,
    )
    .await?;

    let output_path = args
        .output
        .clone()
        .unwrap_or_else(|| obsidian::default_handoff_path(&args.vault, &snapshot.resume));
    let (title, markdown) =
        obsidian::build_handoff_markdown(&args.vault, &snapshot.resume, &snapshot.sources);
    let preview = serde_json::json!({
        "output_path": output_path.display().to_string(),
        "open_uri": obsidian::build_open_uri(&output_path, args.pane_type.as_deref())?,
        "title": title,
        "project": snapshot.resume.project,
        "namespace": snapshot.resume.namespace,
        "workspace": snapshot.resume.workspace,
        "visibility": snapshot.resume.visibility,
        "working": snapshot.resume.working.records.len(),
        "inbox": snapshot.resume.inbox.items.len(),
        "workspaces": snapshot.resume.workspaces.workspaces.len(),
        "semantic_hits": snapshot.resume.semantic.as_ref().map(|semantic| semantic.items.len()).unwrap_or(0),
        "sources": snapshot.sources.sources.len(),
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
    let (title, markdown, output_path, preview, index_items, index_kind) = if let Some(id) =
        args.id.as_ref()
    {
        let id = id
            .parse::<uuid::Uuid>()
            .context("parse obsidian compile id")?;
        let explain = client
            .explain(&ExplainMemoryRequest {
                id,
                belief_branch: None,
                route: parse_retrieval_route(args.route.clone())?,
                intent: parse_retrieval_intent(args.intent.clone())?,
            })
            .await?;
        let output_path = args
            .output
            .clone()
            .unwrap_or_else(|| obsidian::default_compiled_memory_path(&args.vault, &explain));
        let (title, markdown) = obsidian::build_compiled_memory_markdown(&args.vault, &explain);
        let preview = serde_json::json!({
            "output_path": output_path.display().to_string(),
            "open_uri": obsidian::build_open_uri(&output_path, args.pane_type.as_deref())?,
            "title": title,
            "id": explain.item.id,
            "kind": format!("{:?}", explain.item.kind).to_lowercase(),
            "rehydration": explain.rehydration.len(),
            "apply": args.apply,
        });
        (title, markdown, output_path, preview, 1usize, "memory")
    } else {
        let Some(query) = args.query.as_ref() else {
            anyhow::bail!("obsidian compile requires --query <text> or --id <uuid>");
        };
        let route = parse_retrieval_route(args.route.clone())?;
        let intent = parse_retrieval_intent(args.intent.clone())?;
        let response = client
            .search(&SearchMemoryRequest {
                query: Some(query.clone()),
                route,
                intent,
                scopes: vec![
                    MemoryScope::Project,
                    MemoryScope::Global,
                    MemoryScope::Synced,
                ],
                kinds: vec![],
                statuses: vec![
                    MemoryStatus::Active,
                    MemoryStatus::Stale,
                    MemoryStatus::Contested,
                ],
                project: args.project.clone(),
                namespace: args.namespace.clone(),
                workspace: args.workspace.clone(),
                visibility: args
                    .visibility
                    .as_deref()
                    .map(parse_memory_visibility_value)
                    .transpose()?,
                belief_branch: None,
                source_agent: None,
                tags: Vec::new(),
                stages: vec![MemoryStage::Canonical, MemoryStage::Candidate],
                limit: Some(args.limit.unwrap_or(12).clamp(1, 48)),
                max_chars_per_item: Some(800),
            })
            .await?;
        let semantic = if let Some(rag) = maybe_rag_client_from_bundle_or_env()? {
            rag.retrieve(&RagRetrieveRequest {
                query: query.clone(),
                project: args.project.clone(),
                namespace: args.namespace.clone(),
                mode: RagRetrieveMode::Auto,
                limit: Some(6),
                include_cross_modal: false,
            })
            .await
            .ok()
            .filter(|response| !response.items.is_empty())
        } else {
            None
        };

        let output_path = args
            .output
            .clone()
            .unwrap_or_else(|| obsidian::default_compiled_note_path(&args.vault, query));
        let (title, markdown) = obsidian::build_compiled_note_markdown(
            &args.vault,
            query,
            &response,
            semantic.as_ref(),
        );
        let preview = serde_json::json!({
            "output_path": output_path.display().to_string(),
            "open_uri": obsidian::build_open_uri(&output_path, args.pane_type.as_deref())?,
            "title": title,
            "query": query,
            "items": response.items.len(),
            "semantic_hits": semantic.as_ref().map(|response| response.items.len()).unwrap_or(0),
            "apply": args.apply,
        });
        (
            title,
            markdown,
            output_path,
            preview,
            response.items.len(),
            "query",
        )
    };

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
    let index_path = obsidian::default_compiled_index_path(&args.vault);
    let existing_index = fs::read_to_string(&index_path).ok();
    let index_markdown = obsidian::build_compiled_index_markdown(
        existing_index.as_deref(),
        index_kind,
        &title,
        &output_path,
        index_items,
    );
    obsidian::write_markdown(&index_path, &index_markdown)?;
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
        let id = id.parse::<uuid::Uuid>().context("parse obsidian open id")?;
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
        args.workspace.clone(),
        args.visibility
            .as_deref()
            .map(parse_memory_visibility_value)
            .transpose()?,
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
            workspace: None,
            visibility: None,
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
        workspace: args.workspace.clone(),
        visibility: args
            .visibility
            .as_deref()
            .map(parse_memory_visibility_value)
            .transpose()?,
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
    if let Some(value) = explicit
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
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

fn maybe_rag_client_for_bundle(output: &Path) -> anyhow::Result<Option<RagClient>> {
    if let Some(config) = read_bundle_rag_config(output)? {
        if config.enabled {
            let rag_url = config.url.with_context(|| {
                format!(
                    "rag backend is enabled in {} but no url was configured",
                    output.display()
                )
            })?;
            return Ok(Some(RagClient::new(rag_url)?));
        }
    }

    match std::env::var("MEMD_RAG_URL") {
        Ok(value) if !value.trim().is_empty() => Ok(Some(RagClient::new(value)?)),
        _ => Ok(None),
    }
}

fn build_resume_rag_query(
    project: Option<&str>,
    workspace: Option<&str>,
    intent: &str,
    working: &memd_schema::WorkingMemoryResponse,
    context: &memd_schema::CompactContextResponse,
) -> String {
    let mut parts = Vec::new();

    if let Some(project) = project.filter(|value| !value.trim().is_empty()) {
        parts.push(format!("project: {project}"));
    }
    if let Some(workspace) = workspace.filter(|value| !value.trim().is_empty()) {
        parts.push(format!("workspace: {workspace}"));
    }
    if !intent.trim().is_empty() {
        parts.push(format!("intent: {intent}"));
    }

    for record in working.records.iter().take(2) {
        let value = compact_resume_rag_text(&record.record, 180);
        if !value.is_empty() {
            parts.push(format!("working: {value}"));
        }
    }

    for record in context.records.iter().take(2) {
        let value = compact_resume_rag_text(&record.record, 180);
        if !value.is_empty() {
            parts.push(format!("context: {value}"));
        }
    }

    compact_resume_rag_text(&parts.join(" | "), 700)
}

fn compact_resume_rag_text(input: &str, max_chars: usize) -> String {
    let collapsed = input.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.len() <= max_chars {
        return collapsed;
    }

    let mut output = String::new();
    for ch in collapsed.chars() {
        if output.chars().count() >= max_chars.saturating_sub(1) {
            break;
        }
        output.push(ch);
    }
    output.push('…');
    output
}

fn default_auto_short_term_capture() -> bool {
    true
}

fn default_bundle_session() -> String {
    format!(
        "session-{}",
        &uuid::Uuid::new_v4().simple().to_string()[..8]
    )
}

fn compose_agent_identity(agent: &str, session: Option<&str>) -> String {
    let agent = agent.trim();
    let session = session.map(str::trim).filter(|value| !value.is_empty());
    match session {
        Some(session) => format!("{agent}@{session}"),
        None => agent.to_string(),
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

    let session = args
        .session
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(default_bundle_session);

    let config = BundleConfig {
        schema_version: 2,
        project: args.project.clone(),
        namespace: args.namespace.clone(),
        agent: args.agent.clone(),
        session: session.clone(),
        base_url: args.base_url.clone(),
        route: args.route.clone(),
        intent: args.intent.clone(),
        workspace: args.workspace.clone(),
        visibility: args.visibility.clone(),
        auto_short_term_capture: true,
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
            "MEMD_BASE_URL={}\nMEMD_PROJECT={}\n{}MEMD_AGENT={}\nMEMD_SESSION={}\nMEMD_ROUTE={}\nMEMD_INTENT={}\nMEMD_AUTO_SHORT_TERM_CAPTURE={}\n{}{}{}",
            args.base_url,
            args.project,
            args.namespace
                .as_ref()
                .map(|value| format!("MEMD_NAMESPACE={value}\n"))
                .unwrap_or_default(),
            compose_agent_identity(&args.agent, Some(&session)),
            session,
            args.route,
            args.intent,
            if config.auto_short_term_capture { "true" } else { "false" },
            args.workspace
                .as_ref()
                .map(|value| format!("MEMD_WORKSPACE={value}\n"))
                .unwrap_or_default(),
            args.visibility
                .as_ref()
                .map(|value| format!("MEMD_VISIBILITY={value}\n"))
                .unwrap_or_default(),
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
            "$env:MEMD_BASE_URL = \"{}\"\n$env:MEMD_PROJECT = \"{}\"\n{}$env:MEMD_AGENT = \"{}\"\n$env:MEMD_SESSION = \"{}\"\n$env:MEMD_ROUTE = \"{}\"\n$env:MEMD_INTENT = \"{}\"\n$env:MEMD_AUTO_SHORT_TERM_CAPTURE = \"{}\"\n{}{}{}",
            escape_ps1(&args.base_url),
            escape_ps1(&args.project),
            args.namespace
                .as_ref()
                .map(|value| format!("$env:MEMD_NAMESPACE = \"{}\"\n", escape_ps1(value)))
                .unwrap_or_default(),
            escape_ps1(&compose_agent_identity(&args.agent, Some(&session))),
            escape_ps1(&session),
            escape_ps1(&args.route),
            escape_ps1(&args.intent),
            if config.auto_short_term_capture { "true" } else { "false" },
            args.workspace
                .as_ref()
                .map(|value| format!("$env:MEMD_WORKSPACE = \"{}\"\n", escape_ps1(value)))
                .unwrap_or_default(),
            args.visibility
                .as_ref()
                .map(|value| format!("$env:MEMD_VISIBILITY = \"{}\"\n", escape_ps1(value)))
                .unwrap_or_default(),
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
    write_bundle_memory_placeholder(output, &config)?;
    write_native_agent_bridge_files(output)?;

    fs::write(
        output.join("README.md"),
        format!(
            "# memd project bundle\n\nThis directory contains the local memd configuration for `{project}`.\n\n## Files\n\n- `config.json`\n- `env`\n- `env.ps1`\n- `MEMD_MEMORY.md`\n- `state/last-resume.json`\n- `agents/CODEX_MEMORY.md`\n- `agents/CLAUDE_CODE_MEMORY.md`\n- `agents/CLAUDE_IMPORTS.md`\n- `agents/CLAUDE.md.example`\n- `agents/OPENCLAW_MEMORY.md`\n- `agents/OPENCODE_MEMORY.md`\n- `agents/codex.sh`\n- `agents/claude-code.sh`\n- `agents/openclaw.sh`\n- `agents/opencode.sh`\n- `hooks/`\n\n## Usage\n\nSource `env` or `env.ps1` before running the hook kit, or point your agent integration at these values directly. Run `memd resume --output {bundle} --intent current_task` or `memd handoff --output {bundle}` for the fast local short-term memory path. Add `--semantic` only when you want deeper LightRAG fallback. Automatic short-term capture is enabled by default for compaction spill boundaries and writes bundle state under `state/last-resume.json`. Use the agent-specific scripts in `agents/` when switching between clients on the same bundle. For Claude Code, import `.memd/agents/CLAUDE_IMPORTS.md` from your project `CLAUDE.md`, then use `/memory` to verify the memd files are loaded.\n",
            project = args.project,
            bundle = output.display(),
        ),
    )
    .with_context(|| format!("write {}", output.join("README.md").display()))?;

    Ok(())
}

fn write_agent_profiles(output: &Path) -> anyhow::Result<()> {
    let agents_dir = output.join("agents");
    fs::create_dir_all(&agents_dir).with_context(|| format!("create {}", agents_dir.display()))?;
    for (slug, env_agent) in [
        ("agent", None),
        ("codex", Some("codex")),
        ("claude-code", Some("claude-code")),
        ("openclaw", Some("openclaw")),
        ("opencode", Some("opencode")),
    ] {
        let shell_profile = render_agent_shell_profile(output, env_agent);
        let shell_path = agents_dir.join(format!("{slug}.sh"));
        fs::write(&shell_path, shell_profile)
            .with_context(|| format!("write {}", shell_path.display()))?;
        set_executable_if_shell_script(
            &shell_path,
            shell_path
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or("agent.sh"),
        )?;

        let ps1_profile = render_agent_ps1_profile(output, env_agent);
        let ps1_path = agents_dir.join(format!("{slug}.ps1"));
        fs::write(&ps1_path, ps1_profile)
            .with_context(|| format!("write {}", ps1_path.display()))?;
    }

    Ok(())
}

fn write_bundle_memory_placeholder(output: &Path, config: &BundleConfig) -> anyhow::Result<()> {
    let mut markdown = String::new();
    markdown.push_str("# memd memory\n\n");
    markdown.push_str("This file is maintained by `memd` for agents that do not have built-in durable memory.\n\n");
    markdown.push_str("Refresh it with:\n\n");
    markdown.push_str(&format!(
        "- `memd resume --output {} --intent current_task`\n- `memd resume --output {} --intent current_task --semantic`\n- `memd handoff --output {}`\n- `memd handoff --output {} --semantic`\n\n",
        output.display(),
        output.display(),
        output.display(),
        output.display()
    ));
    markdown.push_str("## Bundle Defaults\n\n");
    markdown.push_str(&format!(
        "- project: {}\n- namespace: {}\n- agent: {}\n- workspace: {}\n- visibility: {}\n- route: {}\n- intent: {}\n- auto_short_term_capture: {}\n",
        config.project,
        config.namespace.as_deref().unwrap_or("none"),
        config.agent,
        config.workspace.as_deref().unwrap_or("none"),
        config.visibility.as_deref().unwrap_or("all"),
        config.route,
        config.intent,
        if config.auto_short_term_capture { "true" } else { "false" },
    ));
    markdown.push_str("\n## Notes\n\n");
    markdown
        .push_str("- `resume` keeps the active working memory fresh on the fast local hot path.\n");
    markdown.push_str("- `handoff` adds shared workspace, source-lane, and delegation state.\n");
    markdown.push_str("- automatic short-term capture runs on compaction spill boundaries unless disabled in the bundle env/config.\n");
    markdown.push_str(
        "- add `--semantic` only when you want slower deep recall from the semantic backend.\n",
    );
    markdown.push_str(
        "- future dream/consolidation output should flow back into this same memory surface.\n",
    );
    write_memory_markdown_files(output, &markdown)
}

async fn write_bundle_memory_files(
    output: &Path,
    snapshot: &ResumeSnapshot,
    handoff: Option<&HandoffSnapshot>,
) -> anyhow::Result<()> {
    let markdown = render_bundle_memory_markdown(snapshot, handoff);
    write_memory_markdown_files(output, &markdown)?;
    write_bundle_resume_state(output, snapshot)?;
    write_bundle_heartbeat(output, Some(snapshot), false).await
}

fn bundle_resume_state_path(output: &Path) -> PathBuf {
    output.join("state").join("last-resume.json")
}

fn bundle_heartbeat_state_path(output: &Path) -> PathBuf {
    output.join("state").join("heartbeat.json")
}

fn bundle_claims_state_path(output: &Path) -> PathBuf {
    output.join("state").join("claims.json")
}

fn read_bundle_resume_state(output: &Path) -> anyhow::Result<Option<BundleResumeState>> {
    let path = bundle_resume_state_path(output);
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let state = serde_json::from_str::<BundleResumeState>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(state))
}

fn write_bundle_resume_state(output: &Path, snapshot: &ResumeSnapshot) -> anyhow::Result<()> {
    let path = bundle_resume_state_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let state = BundleResumeState::from_snapshot(snapshot);
    fs::write(&path, serde_json::to_string_pretty(&state)? + "\n")
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn read_bundle_heartbeat(output: &Path) -> anyhow::Result<Option<BundleHeartbeatState>> {
    let path = bundle_heartbeat_state_path(output);
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let state = serde_json::from_str::<BundleHeartbeatState>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(state))
}

fn read_bundle_claims(output: &Path) -> anyhow::Result<SessionClaimsState> {
    let path = bundle_claims_state_path(output);
    if !path.exists() {
        return Ok(SessionClaimsState::default());
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let state = serde_json::from_str::<SessionClaimsState>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(state)
}

fn write_bundle_claims(output: &Path, state: &SessionClaimsState) -> anyhow::Result<()> {
    let path = bundle_claims_state_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&path, serde_json::to_string_pretty(state)? + "\n")
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn detect_host_name() -> Option<String> {
    std::env::var("HOSTNAME")
        .ok()
        .or_else(|| std::env::var("COMPUTERNAME").ok())
        .filter(|value| !value.trim().is_empty())
}

fn heartbeat_presence_label(last_seen: DateTime<Utc>) -> &'static str {
    let age = Utc::now() - last_seen;
    if age.num_seconds() <= 120 {
        "active"
    } else if age.num_minutes() <= 15 {
        "stale"
    } else {
        "dead"
    }
}

fn build_bundle_heartbeat(
    output: &Path,
    snapshot: Option<&ResumeSnapshot>,
) -> anyhow::Result<BundleHeartbeatState> {
    let runtime = read_bundle_runtime_config(output)?.unwrap_or(BundleRuntimeConfig {
        project: None,
        namespace: None,
        agent: None,
        session: None,
        base_url: None,
        route: None,
        intent: None,
        workspace: None,
        visibility: None,
        auto_short_term_capture: true,
    });
    let session = runtime.session.clone();
    let agent = runtime.agent.clone();
    let effective_agent = agent
        .as_deref()
        .map(|value| compose_agent_identity(value, session.as_deref()));
    let focus = snapshot
        .and_then(|value| {
            value
                .working
                .records
                .first()
                .map(|record| record.record.clone())
        })
        .or_else(|| {
            read_bundle_resume_state(output)
                .ok()
                .flatten()
                .and_then(|value| value.focus)
        });
    let pressure = snapshot
        .and_then(|value| {
            value
                .inbox
                .items
                .first()
                .map(|item| item.item.content.clone())
        })
        .or_else(|| {
            read_bundle_resume_state(output)
                .ok()
                .flatten()
                .and_then(|value| value.pressure)
        });
    let next_recovery = snapshot
        .and_then(|value| {
            value
                .working
                .rehydration_queue
                .first()
                .map(|item| format!("{}: {}", item.label, item.summary))
        })
        .or_else(|| {
            read_bundle_resume_state(output)
                .ok()
                .flatten()
                .and_then(|value| value.next_recovery)
        });

    Ok(BundleHeartbeatState {
        session,
        agent,
        effective_agent,
        project: snapshot
            .and_then(|value| value.project.clone())
            .or(runtime.project),
        namespace: snapshot
            .and_then(|value| value.namespace.clone())
            .or(runtime.namespace),
        workspace: snapshot
            .and_then(|value| value.workspace.clone())
            .or(runtime.workspace),
        visibility: snapshot
            .and_then(|value| value.visibility.clone())
            .or(runtime.visibility),
        base_url: runtime.base_url,
        base_url_healthy: None,
        host: detect_host_name(),
        pid: Some(std::process::id()),
        focus,
        pressure,
        next_recovery,
        status: "live".to_string(),
        last_seen: Utc::now(),
    })
}

async fn write_bundle_heartbeat(
    output: &Path,
    snapshot: Option<&ResumeSnapshot>,
    probe_base_url: bool,
) -> anyhow::Result<()> {
    let path = bundle_heartbeat_state_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let mut state = build_bundle_heartbeat(output, snapshot)?;
    if probe_base_url && let Some(url) = state.base_url.as_deref() {
        state.base_url_healthy = Some(MemdClient::new(url)?.healthz().await.is_ok());
    }
    fs::write(&path, serde_json::to_string_pretty(&state)? + "\n")
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

async fn refresh_bundle_heartbeat(
    output: &Path,
    snapshot: Option<&ResumeSnapshot>,
    probe_base_url: bool,
) -> anyhow::Result<BundleHeartbeatState> {
    write_bundle_heartbeat(output, snapshot, probe_base_url).await?;
    read_bundle_heartbeat(output)?.context("reload bundle heartbeat after write")
}

fn render_bundle_heartbeat_summary(state: &BundleHeartbeatState) -> String {
    format!(
        "heartbeat project={} agent={} session={} presence={} base_url={} focus=\"{}\" pressure=\"{}\"",
        state.project.as_deref().unwrap_or("none"),
        state
            .effective_agent
            .as_deref()
            .or(state.agent.as_deref())
            .unwrap_or("none"),
        state.session.as_deref().unwrap_or("none"),
        heartbeat_presence_label(state.last_seen),
        state.base_url.as_deref().unwrap_or("none"),
        state
            .focus
            .as_deref()
            .map(|value| compact_inline(value, 72))
            .unwrap_or_else(|| "none".to_string()),
        state
            .pressure
            .as_deref()
            .map(|value| compact_inline(value, 72))
            .unwrap_or_else(|| "none".to_string())
    )
}

async fn run_claims_command(args: &ClaimsArgs, base_url: &str) -> anyhow::Result<ClaimsResponse> {
    let runtime = read_bundle_runtime_config(&args.output)?;
    let heartbeat = read_bundle_heartbeat(&args.output)?;
    let current_session = runtime
        .as_ref()
        .and_then(|config| config.session.clone())
        .filter(|value| !value.trim().is_empty());
    let current_base_url = runtime
        .as_ref()
        .and_then(|config| config.base_url.clone())
        .unwrap_or_else(|| base_url.to_string());
    let current_agent = runtime.as_ref().and_then(|config| config.agent.clone());
    let current_effective_agent = runtime.as_ref().and_then(|config| {
        config
            .agent
            .as_deref()
            .map(|agent| compose_agent_identity(agent, config.session.as_deref()))
    });
    let client = MemdClient::new(&current_base_url)?;

    if args.acquire {
        let scope = args
            .scope
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .context("claims --acquire requires --scope")?;
        let session = current_session
            .as_deref()
            .context("claims --acquire requires a configured bundle session")?;
        let response = client
            .acquire_peer_claim(&memd_schema::PeerClaimAcquireRequest {
                scope: scope.to_string(),
                session: session.to_string(),
                agent: current_agent,
                effective_agent: current_effective_agent,
                project: runtime.as_ref().and_then(|config| config.project.clone()),
                namespace: runtime.as_ref().and_then(|config| config.namespace.clone()),
                workspace: runtime.as_ref().and_then(|config| config.workspace.clone()),
                host: heartbeat.as_ref().and_then(|value| value.host.clone()),
                pid: heartbeat.as_ref().and_then(|value| value.pid),
                ttl_seconds: args.ttl_secs,
            })
            .await?;
        auto_checkpoint_bundle_event(
            &args.output,
            &current_base_url,
            "claims",
            format!("Claimed scope {scope} for active work."),
            vec!["claims".to_string(), "auto-checkpoint".to_string()],
            0.82,
        )
        .await?;
        let cache = SessionClaimsState {
            claims: client
                .peer_claims(&memd_schema::PeerClaimsRequest {
                    session: None,
                    project: runtime.as_ref().and_then(|config| config.project.clone()),
                    namespace: runtime.as_ref().and_then(|config| config.namespace.clone()),
                    workspace: None,
                    active_only: Some(true),
                    limit: Some(512),
                })
                .await?
                .claims
                .into_iter()
                .map(session_claim_from_record)
                .collect(),
        };
        write_bundle_claims(&args.output, &cache)?;
        return Ok(ClaimsResponse {
            bundle_root: args.output.display().to_string(),
            current_session,
            claims: response
                .claims
                .into_iter()
                .map(session_claim_from_record)
                .collect(),
        });
    } else if let Some(target_session) = args
        .transfer_to_session
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let scope = args
            .scope
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .context("claims --transfer-to-session requires --scope")?;
        let session = current_session
            .as_deref()
            .context("claims --transfer-to-session requires a configured bundle session")?;
        let target = resolve_target_session_bundle(&args.output, target_session)?;
        let response = client
            .transfer_peer_claim(&memd_schema::PeerClaimTransferRequest {
                scope: scope.to_string(),
                from_session: session.to_string(),
                to_session: target_session.to_string(),
                to_agent: target.as_ref().and_then(|entry| entry.agent.clone()),
                to_effective_agent: target
                    .as_ref()
                    .and_then(|entry| entry.effective_agent.clone()),
            })
            .await?;
        auto_checkpoint_bundle_event(
            &args.output,
            &current_base_url,
            "claims",
            format!("Transferred scope {scope} to session {target_session}."),
            vec![
                "claims".to_string(),
                "assignment".to_string(),
                "auto-checkpoint".to_string(),
            ],
            0.84,
        )
        .await?;
        let cache = SessionClaimsState {
            claims: client
                .peer_claims(&memd_schema::PeerClaimsRequest {
                    session: None,
                    project: runtime.as_ref().and_then(|config| config.project.clone()),
                    namespace: runtime.as_ref().and_then(|config| config.namespace.clone()),
                    workspace: None,
                    active_only: Some(true),
                    limit: Some(512),
                })
                .await?
                .claims
                .into_iter()
                .map(session_claim_from_record)
                .collect(),
        };
        write_bundle_claims(&args.output, &cache)?;
        return Ok(ClaimsResponse {
            bundle_root: args.output.display().to_string(),
            current_session,
            claims: response
                .claims
                .into_iter()
                .map(session_claim_from_record)
                .collect(),
        });
    } else if args.release {
        let scope = args
            .scope
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .context("claims --release requires --scope")?;
        let session = current_session
            .as_deref()
            .context("claims --release requires a configured bundle session")?;
        let response = client
            .release_peer_claim(&memd_schema::PeerClaimReleaseRequest {
                scope: scope.to_string(),
                session: session.to_string(),
            })
            .await?;
        auto_checkpoint_bundle_event(
            &args.output,
            &current_base_url,
            "claims",
            format!("Released scope {scope} after finishing or handing off work."),
            vec!["claims".to_string(), "auto-checkpoint".to_string()],
            0.78,
        )
        .await?;
        let cache = SessionClaimsState {
            claims: client
                .peer_claims(&memd_schema::PeerClaimsRequest {
                    session: None,
                    project: runtime.as_ref().and_then(|config| config.project.clone()),
                    namespace: runtime.as_ref().and_then(|config| config.namespace.clone()),
                    workspace: None,
                    active_only: Some(true),
                    limit: Some(512),
                })
                .await?
                .claims
                .into_iter()
                .map(session_claim_from_record)
                .collect(),
        };
        write_bundle_claims(&args.output, &cache)?;
        return Ok(ClaimsResponse {
            bundle_root: args.output.display().to_string(),
            current_session,
            claims: response
                .claims
                .into_iter()
                .map(session_claim_from_record)
                .collect(),
        });
    }

    let response = client
        .peer_claims(&memd_schema::PeerClaimsRequest {
            session: None,
            project: runtime.as_ref().and_then(|config| config.project.clone()),
            namespace: runtime.as_ref().and_then(|config| config.namespace.clone()),
            workspace: None,
            active_only: Some(true),
            limit: Some(512),
        })
        .await?;
    let claims = response
        .claims
        .into_iter()
        .map(session_claim_from_record)
        .collect::<Vec<_>>();
    write_bundle_claims(
        &args.output,
        &SessionClaimsState {
            claims: claims.clone(),
        },
    )?;

    Ok(ClaimsResponse {
        bundle_root: args.output.display().to_string(),
        current_session,
        claims,
    })
}

fn render_claims_summary(response: &ClaimsResponse) -> String {
    let mut lines = vec![format!(
        "claims bundle={} current_session={} active={}",
        response.bundle_root,
        response.current_session.as_deref().unwrap_or("none"),
        response.claims.len()
    )];
    for claim in &response.claims {
        lines.push(format!(
            "- {} | holder={} | workspace={} | expires_at={}",
            claim.scope,
            claim
                .effective_agent
                .as_deref()
                .or(claim.session.as_deref())
                .unwrap_or("none"),
            claim.workspace.as_deref().unwrap_or("none"),
            claim.expires_at.to_rfc3339(),
        ));
    }
    lines.join("\n")
}

async fn run_messages_command(
    args: &MessagesArgs,
    base_url: &str,
) -> anyhow::Result<MessagesResponse> {
    let runtime = read_bundle_runtime_config(&args.output)?;
    let current_session = runtime
        .as_ref()
        .and_then(|config| config.session.clone())
        .filter(|value| !value.trim().is_empty());
    let current_agent = runtime.as_ref().and_then(|config| {
        config
            .agent
            .as_deref()
            .map(|agent| compose_agent_identity(agent, config.session.as_deref()))
    });
    let current_project = runtime.as_ref().and_then(|config| config.project.clone());
    let current_namespace = runtime.as_ref().and_then(|config| config.namespace.clone());
    let current_workspace = runtime.as_ref().and_then(|config| config.workspace.clone());
    let current_base_url = runtime
        .as_ref()
        .and_then(|config| config.base_url.clone())
        .unwrap_or_else(|| base_url.to_string());

    if args.send {
        let target_session = args
            .target_session
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .context("messages --send requires --target-session")?;
        let from_session = current_session
            .as_deref()
            .context("messages --send requires a configured bundle session")?;
        let (kind, content) = derive_outbound_message(args)
            .context("messages --send requires --content or a request helper")?;
        let target = resolve_target_session_bundle(&args.output, target_session)?
            .context("target session not found in awareness")?;
        let target_runtime = read_bundle_runtime_config(Path::new(&target.bundle_root))?;
        let target_base_url = target_runtime
            .as_ref()
            .and_then(|config| config.base_url.clone())
            .or(target.base_url.clone())
            .unwrap_or_else(|| current_base_url.clone());
        let client = MemdClient::new(&target_base_url)?;
        if let Some(assign_scope) = args
            .assign_scope
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            let transfer_client = MemdClient::new(&current_base_url)?;
            transfer_client
                .transfer_peer_claim(&memd_schema::PeerClaimTransferRequest {
                    scope: assign_scope.to_string(),
                    from_session: from_session.to_string(),
                    to_session: target_session.to_string(),
                    to_agent: target.agent.clone(),
                    to_effective_agent: target.effective_agent.clone(),
                })
                .await?;
        }
        let response = client
            .send_peer_message(&PeerMessageSendRequest {
                kind,
                from_session: from_session.to_string(),
                from_agent: current_agent.clone(),
                to_session: target_session.to_string(),
                project: current_project.clone(),
                namespace: current_namespace.clone(),
                workspace: current_workspace.clone(),
                content,
            })
            .await?;
        let summary = if let Some(assign_scope) = args
            .assign_scope
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            format!("Assigned scope {assign_scope} to session {target_session}.")
        } else if args.request_help {
            format!("Requested help from session {target_session}.")
        } else if args.request_review {
            format!("Requested review from session {target_session}.")
        } else {
            format!(
                "Sent {} message to session {target_session}.",
                response.messages[0].kind
            )
        };
        let mut tags = vec!["messages".to_string(), "auto-checkpoint".to_string()];
        if args.request_help {
            tags.push("help-request".to_string());
        }
        if args.request_review {
            tags.push("review-request".to_string());
        }
        if args.assign_scope.is_some() {
            tags.push("assignment".to_string());
        }
        auto_checkpoint_bundle_event(
            &args.output,
            &current_base_url,
            "messages",
            summary,
            tags,
            0.8,
        )
        .await?;
        emit_coordination_receipt(
            &client,
            if args.assign_scope.is_some() {
                "assignment"
            } else if args.request_help {
                "help_request"
            } else if args.request_review {
                "review_request"
            } else {
                "message"
            },
            from_session,
            current_agent.clone(),
            Some(target_session.to_string()),
            None,
            args.assign_scope.clone().or(args.scope.clone()),
            current_project.clone(),
            current_namespace.clone(),
            current_workspace.clone(),
            response
                .messages
                .first()
                .map(|message| message.content.clone())
                .unwrap_or_else(|| "coordination message sent".to_string()),
        )
        .await?;
        return Ok(MessagesResponse {
            bundle_root: args.output.display().to_string(),
            current_session,
            messages: response.messages,
        });
    }

    let client = MemdClient::new(&current_base_url)?;
    let messages = if let Some(ack) = args.ack.as_deref() {
        let session = current_session
            .as_deref()
            .context("messages --ack requires a configured bundle session")?;
        client
            .ack_peer_message(&PeerMessageAckRequest {
                id: ack.trim().to_string(),
                session: session.to_string(),
            })
            .await?
            .messages
    } else {
        let session = current_session
            .as_deref()
            .context("messages --inbox requires a configured bundle session")?;
        client
            .peer_inbox(&PeerMessageInboxRequest {
                session: session.to_string(),
                project: current_project,
                namespace: current_namespace,
                workspace: current_workspace,
                include_acknowledged: Some(false),
                limit: Some(128),
            })
            .await?
            .messages
    };

    Ok(MessagesResponse {
        bundle_root: args.output.display().to_string(),
        current_session,
        messages,
    })
}

fn derive_outbound_message(args: &MessagesArgs) -> Option<(String, String)> {
    let assign_scope = args
        .assign_scope
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let scope = args
        .scope
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let explicit_content = args
        .content
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    if args.request_help {
        let content = explicit_content.or_else(|| {
            scope
                .map(|scope| format!("Need help on {scope}. Please coordinate before changing it."))
        })?;
        return Some(("help_request".to_string(), content));
    }

    if args.request_review {
        let content = explicit_content.or_else(|| {
            scope.map(|scope| {
                format!("Need review on {scope}. Please inspect before I hand it off.")
            })
        })?;
        return Some(("review_request".to_string(), content));
    }

    if let Some(assign_scope) = assign_scope {
        let content = explicit_content.or_else(|| {
            Some(format!(
                "Assigned scope {assign_scope}. Take ownership and continue from there."
            ))
        })?;
        return Some(("assignment".to_string(), content));
    }

    let content = explicit_content?;
    Some((
        args.kind.clone().unwrap_or_else(|| "handoff".to_string()),
        content,
    ))
}

fn render_messages_summary(response: &MessagesResponse) -> String {
    let mut lines = vec![format!(
        "messages bundle={} current_session={} count={}",
        response.bundle_root,
        response.current_session.as_deref().unwrap_or("none"),
        response.messages.len()
    )];
    for message in &response.messages {
        lines.push(format!(
            "- {} [{}] {} -> {} | acked={} | {}",
            &message.id[..8.min(message.id.len())],
            message.kind,
            message.from_agent.as_deref().unwrap_or("unknown"),
            message.to_session,
            if message.acknowledged_at.is_some() {
                "yes"
            } else {
                "no"
            },
            compact_inline(&message.content, 80)
        ));
    }
    lines.join("\n")
}

async fn emit_coordination_receipt(
    client: &MemdClient,
    kind: &str,
    actor_session: &str,
    actor_agent: Option<String>,
    target_session: Option<String>,
    task_id: Option<String>,
    scope: Option<String>,
    project: Option<String>,
    namespace: Option<String>,
    workspace: Option<String>,
    summary: String,
) -> anyhow::Result<()> {
    client
        .record_peer_coordination_receipt(&PeerCoordinationReceiptRequest {
            kind: kind.to_string(),
            actor_session: actor_session.to_string(),
            actor_agent,
            target_session,
            task_id,
            scope,
            project,
            namespace,
            workspace,
            summary,
        })
        .await?;
    Ok(())
}

async fn run_tasks_command(args: &TasksArgs, base_url: &str) -> anyhow::Result<TasksResponse> {
    let runtime = read_bundle_runtime_config(&args.output)?;
    let current_session = runtime
        .as_ref()
        .and_then(|config| config.session.clone())
        .filter(|value| !value.trim().is_empty());
    let current_agent = runtime.as_ref().and_then(|config| config.agent.clone());
    let current_effective_agent = runtime.as_ref().and_then(|config| {
        config
            .agent
            .as_deref()
            .map(|agent| compose_agent_identity(agent, config.session.as_deref()))
    });
    let current_project = runtime.as_ref().and_then(|config| config.project.clone());
    let current_namespace = runtime.as_ref().and_then(|config| config.namespace.clone());
    let current_workspace = runtime.as_ref().and_then(|config| config.workspace.clone());
    let current_base_url = runtime
        .as_ref()
        .and_then(|config| config.base_url.clone())
        .unwrap_or_else(|| base_url.to_string());
    let client = MemdClient::new(&current_base_url)?;

    if args.upsert {
        let task_id = args
            .task_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .context("tasks --upsert requires --task-id")?;
        let title = args
            .title
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .context("tasks --upsert requires --title")?;
        let response = client
            .upsert_peer_task(&PeerTaskUpsertRequest {
                task_id: task_id.to_string(),
                title: title.to_string(),
                description: args.description.clone(),
                status: args.status.clone(),
                coordination_mode: args.mode.clone(),
                session: current_session.clone(),
                agent: current_agent.clone(),
                effective_agent: current_effective_agent.clone(),
                project: current_project.clone(),
                namespace: current_namespace.clone(),
                workspace: current_workspace.clone(),
                claim_scopes: args.scope.clone(),
                help_requested: Some(false),
                review_requested: Some(false),
            })
            .await?;
        auto_checkpoint_bundle_event(
            &args.output,
            &current_base_url,
            "tasks",
            format!("Updated shared task {task_id}."),
            vec!["tasks".to_string(), "auto-checkpoint".to_string()],
            0.83,
        )
        .await?;
        emit_coordination_receipt(
            &client,
            "task_update",
            current_session.as_deref().unwrap_or("unknown"),
            current_effective_agent.clone(),
            None,
            Some(task_id.to_string()),
            args.scope.first().cloned(),
            current_project.clone(),
            current_namespace.clone(),
            current_workspace.clone(),
            format!("Updated shared task {task_id}."),
        )
        .await?;
        return Ok(TasksResponse {
            bundle_root: args.output.display().to_string(),
            current_session,
            tasks: response.tasks,
        });
    }

    if let Some(target_session) = args
        .assign_to_session
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let task_id = args
            .task_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .context("tasks --assign-to-session requires --task-id")?;
        let session = current_session
            .as_deref()
            .context("tasks --assign-to-session requires a configured bundle session")?;
        let target = resolve_target_session_bundle(&args.output, target_session)?
            .context("target session not found in awareness")?;

        let existing = client
            .peer_tasks(&PeerTasksRequest {
                session: None,
                project: current_project.clone(),
                namespace: current_namespace.clone(),
                workspace: current_workspace.clone(),
                active_only: Some(false),
                limit: Some(256),
            })
            .await?;
        if let Some(task) = existing.tasks.iter().find(|task| task.task_id == task_id) {
            for scope in &task.claim_scopes {
                let _ = client
                    .transfer_peer_claim(&memd_schema::PeerClaimTransferRequest {
                        scope: scope.clone(),
                        from_session: session.to_string(),
                        to_session: target_session.to_string(),
                        to_agent: target.agent.clone(),
                        to_effective_agent: target.effective_agent.clone(),
                    })
                    .await;
            }
        }

        let response = client
            .assign_peer_task(&PeerTaskAssignRequest {
                task_id: task_id.to_string(),
                from_session: Some(session.to_string()),
                to_session: target_session.to_string(),
                to_agent: target.agent.clone(),
                to_effective_agent: target.effective_agent.clone(),
                note: None,
            })
            .await?;
        auto_checkpoint_bundle_event(
            &args.output,
            &current_base_url,
            "tasks",
            format!("Assigned shared task {task_id} to session {target_session}."),
            vec![
                "tasks".to_string(),
                "assignment".to_string(),
                "auto-checkpoint".to_string(),
            ],
            0.85,
        )
        .await?;
        emit_coordination_receipt(
            &client,
            "task_assignment",
            session,
            current_effective_agent.clone(),
            Some(target_session.to_string()),
            Some(task_id.to_string()),
            None,
            current_project.clone(),
            current_namespace.clone(),
            current_workspace.clone(),
            format!("Assigned shared task {task_id} to session {target_session}."),
        )
        .await?;
        return Ok(TasksResponse {
            bundle_root: args.output.display().to_string(),
            current_session,
            tasks: response.tasks,
        });
    }

    if args.request_help || args.request_review {
        let task_id = args
            .task_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .context("tasks help/review requires --task-id")?;
        let target_session = args
            .target_session
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .context("tasks help/review requires --target-session")?;
        let from_session = current_session
            .as_deref()
            .context("tasks help/review requires a configured bundle session")?;
        let target = resolve_target_session_bundle(&args.output, target_session)?
            .context("target session not found in awareness")?;
        let target_runtime = read_bundle_runtime_config(Path::new(&target.bundle_root))?;
        let target_base_url = target_runtime
            .as_ref()
            .and_then(|config| config.base_url.clone())
            .or(target.base_url.clone())
            .unwrap_or_else(|| current_base_url.clone());
        let target_client = MemdClient::new(&target_base_url)?;

        let tasks = client
            .upsert_peer_task(&PeerTaskUpsertRequest {
                task_id: task_id.to_string(),
                title: args
                    .title
                    .clone()
                    .unwrap_or_else(|| format!("Shared task {task_id}")),
                description: args.description.clone(),
                status: Some(if args.request_help {
                    "needs_help".to_string()
                } else {
                    "needs_review".to_string()
                }),
                coordination_mode: Some(if args.request_help {
                    "help_only".to_string()
                } else {
                    "shared_review".to_string()
                }),
                session: current_session.clone(),
                agent: current_agent.clone(),
                effective_agent: current_effective_agent.clone(),
                project: current_project.clone(),
                namespace: current_namespace.clone(),
                workspace: current_workspace.clone(),
                claim_scopes: args.scope.clone(),
                help_requested: Some(args.request_help),
                review_requested: Some(args.request_review),
            })
            .await?;
        let kind = if args.request_help {
            "help_request"
        } else {
            "review_request"
        };
        let content = if args.request_help {
            format!(
                "Need help on shared task {task_id}. Please coordinate before changing overlapping work."
            )
        } else {
            format!("Need review on shared task {task_id}. Please inspect the task before handoff.")
        };
        target_client
            .send_peer_message(&PeerMessageSendRequest {
                kind: kind.to_string(),
                from_session: from_session.to_string(),
                from_agent: current_effective_agent.clone(),
                to_session: target_session.to_string(),
                project: current_project.clone(),
                namespace: current_namespace.clone(),
                workspace: current_workspace.clone(),
                content,
            })
            .await?;
        auto_checkpoint_bundle_event(
            &args.output,
            &current_base_url,
            "tasks",
            format!(
                "{} requested on shared task {task_id} from session {target_session}.",
                if args.request_help { "Help" } else { "Review" }
            ),
            vec![
                "tasks".to_string(),
                if args.request_help {
                    "help-request".to_string()
                } else {
                    "review-request".to_string()
                },
                "auto-checkpoint".to_string(),
            ],
            0.81,
        )
        .await?;
        emit_coordination_receipt(
            &client,
            if args.request_help {
                "task_help_request"
            } else {
                "task_review_request"
            },
            from_session,
            current_effective_agent.clone(),
            Some(target_session.to_string()),
            Some(task_id.to_string()),
            args.scope.first().cloned(),
            current_project.clone(),
            current_namespace.clone(),
            current_workspace.clone(),
            format!(
                "{} requested on shared task {task_id} from session {target_session}.",
                if args.request_help { "Help" } else { "Review" }
            ),
        )
        .await?;
        return Ok(TasksResponse {
            bundle_root: args.output.display().to_string(),
            current_session,
            tasks: tasks.tasks,
        });
    }

    let response = client
        .peer_tasks(&PeerTasksRequest {
            session: None,
            project: current_project,
            namespace: current_namespace,
            workspace: current_workspace,
            active_only: Some(!args.all),
            limit: Some(256),
        })
        .await?;
    Ok(TasksResponse {
        bundle_root: args.output.display().to_string(),
        current_session,
        tasks: response.tasks,
    })
}

fn render_tasks_summary(response: &TasksResponse) -> String {
    let mut lines = vec![format!(
        "tasks bundle={} current_session={} count={}",
        response.bundle_root,
        response.current_session.as_deref().unwrap_or("none"),
        response.tasks.len()
    )];
    for task in &response.tasks {
        lines.push(format!(
            "- {} [{}:{}] owner={} scopes={} help={} review={} | {}",
            task.task_id,
            task.status,
            task.coordination_mode,
            task.effective_agent
                .as_deref()
                .or(task.session.as_deref())
                .unwrap_or("none"),
            if task.claim_scopes.is_empty() {
                "none".to_string()
            } else {
                task.claim_scopes.join(",")
            },
            if task.help_requested { "yes" } else { "no" },
            if task.review_requested { "yes" } else { "no" },
            compact_inline(&task.title, 80)
        ));
    }
    lines.join("\n")
}

async fn run_coordination_command(
    args: &CoordinationArgs,
    base_url: &str,
) -> anyhow::Result<CoordinationResponse> {
    let runtime = read_bundle_runtime_config(&args.output)?;
    let current_session = runtime
        .as_ref()
        .and_then(|config| config.session.clone())
        .filter(|value| !value.trim().is_empty())
        .context("coordination requires a configured bundle session")?;
    let current_base_url = runtime
        .as_ref()
        .and_then(|config| config.base_url.clone())
        .unwrap_or_else(|| base_url.to_string());
    let current_effective_agent = runtime.as_ref().and_then(|config| {
        config
            .agent
            .as_deref()
            .map(|agent| compose_agent_identity(agent, config.session.as_deref()))
    });
    let client = MemdClient::new(&current_base_url)?;
    let awareness = read_project_awareness(&AwarenessArgs {
        output: args.output.clone(),
        root: None,
        include_current: true,
        summary: false,
    })?;
    let current_project = runtime.as_ref().and_then(|config| config.project.clone());
    let current_namespace = runtime.as_ref().and_then(|config| config.namespace.clone());
    let current_workspace = runtime.as_ref().and_then(|config| config.workspace.clone());
    let claims = client
        .peer_claims(&PeerClaimsRequest {
            session: None,
            project: current_project.clone(),
            namespace: current_namespace.clone(),
            workspace: current_workspace.clone(),
            active_only: Some(true),
            limit: Some(512),
        })
        .await?
        .claims
        .into_iter()
        .map(session_claim_from_record)
        .collect::<Vec<_>>();
    let tasks = client
        .peer_tasks(&PeerTasksRequest {
            session: None,
            project: current_project.clone(),
            namespace: current_namespace.clone(),
            workspace: current_workspace.clone(),
            active_only: Some(true),
            limit: Some(512),
        })
        .await?
        .tasks;

    let stale_peers = awareness
        .entries
        .iter()
        .filter(|entry| entry.session.as_deref() != Some(current_session.as_str()))
        .filter(|entry| entry.presence == "stale" || entry.presence == "dead")
        .cloned()
        .collect::<Vec<_>>();

    if let Some(recover_session) = args
        .recover_session
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let stale_entry = stale_peers
            .iter()
            .find(|entry| entry.session.as_deref() == Some(recover_session))
            .cloned()
            .context("recover_session must target a stale or dead session")?;
        let destination = if let Some(to_session) = args
            .to_session
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            awareness
                .entries
                .iter()
                .find(|entry| entry.session.as_deref() == Some(to_session))
                .cloned()
                .context("to_session not found in awareness")?
        } else {
            awareness
                .entries
                .iter()
                .find(|entry| entry.session.as_deref() == Some(current_session.as_str()))
                .cloned()
                .context("current session missing from awareness")?
        };

        let recover_claims = claims
            .iter()
            .filter(|claim| claim.session.as_deref() == Some(recover_session))
            .cloned()
            .collect::<Vec<_>>();
        let recover_tasks = tasks
            .iter()
            .filter(|task| task.session.as_deref() == Some(recover_session))
            .cloned()
            .collect::<Vec<_>>();

        for claim in &recover_claims {
            client
                .recover_peer_claim(&PeerClaimRecoverRequest {
                    scope: claim.scope.clone(),
                    from_session: recover_session.to_string(),
                    to_session: destination.session.clone(),
                    to_agent: destination.agent.clone(),
                    to_effective_agent: destination.effective_agent.clone(),
                })
                .await?;
        }
        for task in &recover_tasks {
            client
                .assign_peer_task(&PeerTaskAssignRequest {
                    task_id: task.task_id.clone(),
                    from_session: Some(recover_session.to_string()),
                    to_session: destination
                        .session
                        .clone()
                        .context("destination session missing for recovery")?,
                    to_agent: destination.agent.clone(),
                    to_effective_agent: destination.effective_agent.clone(),
                    note: Some(format!(
                        "Recovered from {} session {}",
                        stale_entry.presence, recover_session
                    )),
                })
                .await?;
        }
        auto_checkpoint_bundle_event(
            &args.output,
            &current_base_url,
            "coordination",
            format!(
                "Recovered {} claims and {} tasks from {} session {}.",
                recover_claims.len(),
                recover_tasks.len(),
                stale_entry.presence,
                recover_session
            ),
            vec![
                "coordination".to_string(),
                "recovery".to_string(),
                "auto-checkpoint".to_string(),
            ],
            0.86,
        )
        .await?;
        emit_coordination_receipt(
            &client,
            "stale_session_recovery",
            &current_session,
            current_effective_agent.clone(),
            destination.session.clone(),
            None,
            None,
            runtime.as_ref().and_then(|config| config.project.clone()),
            runtime.as_ref().and_then(|config| config.namespace.clone()),
            runtime.as_ref().and_then(|config| config.workspace.clone()),
            format!(
                "Recovered {} claims and {} tasks from {} session {}.",
                recover_claims.len(),
                recover_tasks.len(),
                stale_entry.presence,
                recover_session
            ),
        )
        .await?;
    }

    let response = client
        .peer_coordination_inbox(&PeerCoordinationInboxRequest {
            session: current_session.clone(),
            project: current_project.clone(),
            namespace: current_namespace.clone(),
            workspace: current_workspace.clone(),
            limit: Some(128),
        })
        .await?;
    let claims = client
        .peer_claims(&PeerClaimsRequest {
            session: None,
            project: current_project,
            namespace: current_namespace,
            workspace: current_workspace,
            active_only: Some(true),
            limit: Some(512),
        })
        .await?
        .claims
        .into_iter()
        .map(session_claim_from_record)
        .collect::<Vec<_>>();
    let tasks = client
        .peer_tasks(&PeerTasksRequest {
            session: None,
            project: runtime.as_ref().and_then(|config| config.project.clone()),
            namespace: runtime.as_ref().and_then(|config| config.namespace.clone()),
            workspace: runtime.as_ref().and_then(|config| config.workspace.clone()),
            active_only: Some(true),
            limit: Some(512),
        })
        .await?
        .tasks;
    let receipts = client
        .peer_coordination_receipts(&PeerCoordinationReceiptsRequest {
            session: None,
            project: runtime.as_ref().and_then(|config| config.project.clone()),
            namespace: runtime.as_ref().and_then(|config| config.namespace.clone()),
            workspace: runtime.as_ref().and_then(|config| config.workspace.clone()),
            limit: Some(32),
        })
        .await?
        .receipts;
    let policy_conflicts = tasks
        .iter()
        .filter(|task| task.coordination_mode == "exclusive_write")
        .flat_map(|task| {
            task.claim_scopes.iter().filter_map(|scope| {
                claims
                    .iter()
                    .find(|claim| claim.scope == *scope)
                    .and_then(|claim| {
                        let claim_owner = claim.session.as_deref();
                        let task_owner = task.session.as_deref();
                        if claim_owner.is_some() && claim_owner != task_owner {
                            Some(format!(
                                "task {} requires exclusive_write but scope {} is held by {}",
                                task.task_id,
                                scope,
                                claim
                                    .effective_agent
                                    .as_deref()
                                    .or(claim.session.as_deref())
                                    .unwrap_or("none")
                            ))
                        } else {
                            None
                        }
                    })
            })
        })
        .collect::<Vec<_>>();
    let stale_sessions = stale_peers
        .iter()
        .filter_map(|entry| entry.session.as_deref())
        .collect::<Vec<_>>();
    let active_peers = awareness
        .entries
        .iter()
        .filter(|entry| entry.session.as_deref() != Some(current_session.as_str()))
        .filter(|entry| entry.presence == "active")
        .filter_map(|entry| entry.session.as_deref())
        .collect::<Vec<_>>();
    let suggestions = suggest_coordination_actions(
        &response,
        &stale_sessions,
        &active_peers,
        &claims,
        &tasks,
        &current_session,
        &policy_conflicts,
    );
    Ok(CoordinationResponse {
        bundle_root: args.output.display().to_string(),
        current_session: current_session.clone(),
        inbox: response,
        recovery: CoordinationRecoverySummary {
            stale_peers: stale_peers.clone(),
            reclaimable_claims: claims
                .clone()
                .into_iter()
                .filter(|claim| {
                    claim.session.as_deref().is_some_and(|session| {
                        stale_peers
                            .iter()
                            .any(|entry| entry.session.as_deref() == Some(session))
                    })
                })
                .collect(),
            stalled_tasks: tasks
                .clone()
                .into_iter()
                .filter(|task| {
                    task.session.as_deref().is_some_and(|session| {
                        stale_peers
                            .iter()
                            .any(|entry| entry.session.as_deref() == Some(session))
                    })
                })
                .collect(),
        },
        policy_conflicts,
        suggestions,
        boundary_recommendations: suggest_boundary_recommendations(
            &tasks,
            &claims,
            &current_session,
        ),
        receipts,
    })
}

fn render_coordination_summary(response: &CoordinationResponse, view: Option<&str>) -> String {
    let view = view.unwrap_or("all");
    let mut lines = vec![
        format!(
            "coordination bundle={} session={}",
            response.bundle_root, response.current_session
        ),
        format!(
            "pressure messages={} owned={} help={} review={}",
            response.inbox.messages.len(),
            response.inbox.owned_tasks.len(),
            response.inbox.help_tasks.len(),
            response.inbox.review_tasks.len(),
        ),
        format!(
            "recovery stale_peers={} reclaimable_claims={} stalled_tasks={}",
            response.recovery.stale_peers.len(),
            response.recovery.reclaimable_claims.len(),
            response.recovery.stalled_tasks.len(),
        ),
        format!(
            "policy conflicts={} recommendations={} suggestions={} receipts={}",
            response.policy_conflicts.len(),
            response.boundary_recommendations.len(),
            response.suggestions.len(),
            response.receipts.len(),
        ),
    ];
    if matches!(view, "all" | "overview" | "inbox") {
        lines.push("".to_string());
        lines.push("## Inbox".to_string());
    }
    append_coordination_sections(&mut lines, response, view);
    lines.join("\n")
}

fn suggest_coordination_actions(
    inbox: &PeerCoordinationInboxResponse,
    stale_sessions: &[&str],
    active_peer_sessions: &[&str],
    claims: &[SessionClaim],
    tasks: &[PeerTaskRecord],
    current_session: &str,
    policy_conflicts: &[String],
) -> Vec<CoordinationSuggestion> {
    let mut suggestions = Vec::new();
    let mut emitted = Vec::<(
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
    )>::new();

    let is_stale_session = |session: &str, stale_sessions: &[&str]| {
        stale_sessions.iter().any(|entry| entry == &session)
    };
    let has_scope_conflict = |task_id: &str,
                              scope: &str,
                              list: &Vec<(
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
    )>| {
        list.iter()
            .any(|item| item.0 == "assign_scope" && item.1 == task_id && item.2 == scope)
    };
    let push_unique = |suggestion: CoordinationSuggestion,
                       seen: &mut Vec<(
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
    )>,
                       out: &mut Vec<CoordinationSuggestion>| {
        let key = (
            suggestion.action.clone(),
            suggestion.task_id.clone().unwrap_or_else(String::new),
            suggestion.scope.clone().unwrap_or_else(String::new),
            suggestion.target_session.clone(),
            suggestion.message_id.clone(),
            suggestion.stale_session.clone(),
        );
        if !seen.contains(&key) {
            seen.push(key);
            out.push(suggestion);
        }
    };

    if !inbox.messages.is_empty() {
        for message in inbox.messages.iter().take(3) {
            let suggestion = CoordinationSuggestion {
                action: "ack_message".to_string(),
                priority: "high".to_string(),
                target_session: None,
                task_id: None,
                scope: None,
                message_id: Some(message.id.clone()),
                reason: format!(
                    "Acknowledge {} message from {}.",
                    message.kind,
                    message
                        .from_agent
                        .clone()
                        .unwrap_or_else(|| message.from_session.clone())
                ),
                stale_session: None,
            };
            push_unique(suggestion, &mut emitted, &mut suggestions);
        }
    }

    if !stale_sessions.is_empty() {
        for stale_session in stale_sessions.iter().copied() {
            let reclaimable_claims = claims
                .iter()
                .filter(|claim| claim.session.as_deref() == Some(stale_session))
                .count();
            let stalled_tasks = tasks
                .iter()
                .filter(|task| task.session.as_deref() == Some(stale_session))
                .count();
            if reclaimable_claims == 0 && stalled_tasks == 0 {
                continue;
            }
            let suggestion = CoordinationSuggestion {
                action: "recover_session".to_string(),
                priority: "high".to_string(),
                target_session: None,
                task_id: None,
                scope: None,
                message_id: None,
                reason: format!(
                    "Recover {} claim(s) and {} stalled task(s) from stale session {}.",
                    reclaimable_claims, stalled_tasks, stale_session
                ),
                stale_session: Some(stale_session.to_string()),
            };
            push_unique(suggestion, &mut emitted, &mut suggestions);
        }
    }

    for task in tasks
        .iter()
        .filter(|task| task.coordination_mode == "exclusive_write")
    {
        for scope in &task.claim_scopes {
            let Some(task_owner) = task.session.as_deref() else {
                continue;
            };
            let Some(claim) = claims.iter().find(|claim| {
                claim.scope.as_str() == scope.as_str()
                    && claim
                        .session
                        .as_deref()
                        .is_some_and(|claim_owner| !is_stale_session(claim_owner, stale_sessions))
            }) else {
                continue;
            };
            let Some(claim_owner) = claim.session.as_deref() else {
                continue;
            };
            if claim_owner == task_owner {
                continue;
            }
            if has_scope_conflict(&task.task_id, scope, &emitted) {
                continue;
            }
            let suggestion = CoordinationSuggestion {
                action: "assign_scope".to_string(),
                priority: "medium".to_string(),
                target_session: Some(task_owner.to_string()),
                task_id: Some(task.task_id.clone()),
                scope: Some(scope.clone()),
                message_id: None,
                reason: format!(
                    "Resolve exclusivity conflict for {scope} by moving it to task owner {}.",
                    task_owner
                ),
                stale_session: None,
            };
            push_unique(suggestion, &mut emitted, &mut suggestions);
        }
    }

    if !policy_conflicts.is_empty() && suggestions.len() < 6 && !tasks.is_empty() {
        let Some(peer_session) = active_peer_sessions.first().copied() else {
            return suggestions;
        };
        for task in tasks
            .iter()
            .filter(|task| task.session.as_deref() == Some(current_session))
            .take(2)
        {
            let action = if task.coordination_mode == "shared_review" {
                "request_review"
            } else if task.coordination_mode == "help_only" {
                "request_help"
            } else {
                continue;
            };
            if action == "request_review" && task.review_requested {
                continue;
            }
            if action == "request_help" && task.help_requested {
                continue;
            }
            let suggestion = CoordinationSuggestion {
                action: action.to_string(),
                priority: "low".to_string(),
                target_session: Some(peer_session.to_string()),
                task_id: Some(task.task_id.clone()),
                scope: None,
                message_id: None,
                reason: format!(
                    "Ask {} for collaboration support on task {} before heavy overlap grows.",
                    peer_session, task.task_id
                ),
                stale_session: None,
            };
            push_unique(suggestion, &mut emitted, &mut suggestions);
        }
    }

    suggestions
}

fn append_coordination_sections(
    lines: &mut Vec<String>,
    response: &CoordinationResponse,
    view: &str,
) {
    let show_all = matches!(view, "all" | "overview");
    let show_inbox = show_all || view == "inbox";
    let show_requests = show_all || view == "requests";
    let show_recovery = show_all || view == "recovery";
    let show_policy = show_all || view == "policy";
    let show_suggestions = show_all || view == "suggestions";
    let show_history = show_all || view == "history";

    if show_inbox {
        for message in response.inbox.messages.iter().take(6) {
            lines.push(format!(
                "- msg {} [{}] {}",
                &message.id[..8.min(message.id.len())],
                message.kind,
                compact_inline(&message.content, 90)
            ));
        }
        for task in response.inbox.owned_tasks.iter().take(6) {
            lines.push(format!(
                "- own {} [{}] {}",
                task.task_id,
                task.status,
                compact_inline(&task.title, 90)
            ));
        }
    }
    if show_requests
        && (!response.inbox.help_tasks.is_empty() || !response.inbox.review_tasks.is_empty())
    {
        lines.push("".to_string());
        lines.push("## Requests".to_string());
        for task in response.inbox.help_tasks.iter().take(6) {
            lines.push(format!(
                "- help {} [{}] owner={}",
                task.task_id,
                task.status,
                task.effective_agent
                    .as_deref()
                    .or(task.session.as_deref())
                    .unwrap_or("none")
            ));
        }
        for task in response.inbox.review_tasks.iter().take(6) {
            lines.push(format!(
                "- review {} [{}] owner={}",
                task.task_id,
                task.status,
                task.effective_agent
                    .as_deref()
                    .or(task.session.as_deref())
                    .unwrap_or("none")
            ));
        }
    }
    if show_recovery
        && (!response.recovery.stale_peers.is_empty()
            || !response.recovery.reclaimable_claims.is_empty()
            || !response.recovery.stalled_tasks.is_empty())
    {
        lines.push("".to_string());
        lines.push("## Recovery".to_string());
        for entry in response.recovery.stale_peers.iter().take(6) {
            lines.push(format!(
                "- stale session={} agent={} presence={} focus=\"{}\"",
                entry.session.as_deref().unwrap_or("none"),
                entry
                    .effective_agent
                    .as_deref()
                    .or(entry.agent.as_deref())
                    .unwrap_or("none"),
                entry.presence,
                compact_inline(entry.focus.as_deref().unwrap_or("none"), 72),
            ));
        }
        for claim in response.recovery.reclaimable_claims.iter().take(6) {
            lines.push(format!(
                "- reclaimable claim {} owner={}",
                claim.scope,
                claim
                    .effective_agent
                    .as_deref()
                    .or(claim.session.as_deref())
                    .unwrap_or("none")
            ));
        }
        for task in response.recovery.stalled_tasks.iter().take(6) {
            lines.push(format!(
                "- stalled task {} [{}] owner={}",
                task.task_id,
                task.status,
                task.effective_agent
                    .as_deref()
                    .or(task.session.as_deref())
                    .unwrap_or("none")
            ));
        }
    }
    if show_policy
        && (!response.policy_conflicts.is_empty() || !response.boundary_recommendations.is_empty())
    {
        lines.push("".to_string());
        lines.push("## Policy".to_string());
        for conflict in response.policy_conflicts.iter().take(6) {
            lines.push(format!("- policy {}", compact_inline(conflict, 96)));
        }
        for recommendation in response.boundary_recommendations.iter().take(6) {
            lines.push(format!(
                "- recommend {}",
                compact_inline(recommendation, 96)
            ));
        }
    }
    if show_suggestions && !response.suggestions.is_empty() {
        lines.push("".to_string());
        lines.push("## Suggestions".to_string());
        for suggestion in response.suggestions.iter().take(6) {
            lines.push(format!(
                "- {} [{}] {}",
                suggestion.priority,
                suggestion.action,
                compact_inline(&suggestion.reason, 110),
            ));
        }
    }
    if show_history && !response.receipts.is_empty() {
        lines.push("".to_string());
        lines.push("## History".to_string());
        for receipt in response.receipts.iter().take(8) {
            lines.push(format!(
                "- receipt {} [{}] {}",
                &receipt.id[..8.min(receipt.id.len())],
                receipt.kind,
                compact_inline(&receipt.summary, 96)
            ));
        }
    }
}

fn render_coordination_alerts(
    previous: Option<&CoordinationResponse>,
    current: &CoordinationResponse,
    view: Option<&str>,
) -> Vec<String> {
    let Some(previous) = previous else {
        return vec!["alert initial coordination snapshot".to_string()];
    };

    let view = view.unwrap_or("all");
    let show_all = matches!(view, "all" | "overview");
    let show_suggestions = matches!(view, "all" | "overview" | "suggestions");
    let mut alerts = Vec::new();

    if show_all || view == "inbox" {
        let prev_messages = previous.inbox.messages.len();
        let curr_messages = current.inbox.messages.len();
        let prev_owned = previous.inbox.owned_tasks.len();
        let curr_owned = current.inbox.owned_tasks.len();
        if prev_messages != curr_messages || prev_owned != curr_owned {
            alerts.push(format!(
                "alert inbox messages {}->{} owned {}->{}",
                prev_messages, curr_messages, prev_owned, curr_owned
            ));
        }
    }
    if show_all || view == "requests" {
        let prev_help = previous.inbox.help_tasks.len();
        let curr_help = current.inbox.help_tasks.len();
        let prev_review = previous.inbox.review_tasks.len();
        let curr_review = current.inbox.review_tasks.len();
        if prev_help != curr_help || prev_review != curr_review {
            alerts.push(format!(
                "alert requests help {}->{} review {}->{}",
                prev_help, curr_help, prev_review, curr_review
            ));
        }
    }
    if show_all || view == "recovery" {
        let prev_stale = previous.recovery.stale_peers.len();
        let curr_stale = current.recovery.stale_peers.len();
        let prev_claims = previous.recovery.reclaimable_claims.len();
        let curr_claims = current.recovery.reclaimable_claims.len();
        let prev_tasks = previous.recovery.stalled_tasks.len();
        let curr_tasks = current.recovery.stalled_tasks.len();
        if prev_stale != curr_stale || prev_claims != curr_claims || prev_tasks != curr_tasks {
            alerts.push(format!(
                "alert recovery stale {}->{} reclaimable {}->{} stalled {}->{}",
                prev_stale, curr_stale, prev_claims, curr_claims, prev_tasks, curr_tasks
            ));
        }
    }
    if show_all || view == "policy" {
        let prev_conflicts = previous.policy_conflicts.len();
        let curr_conflicts = current.policy_conflicts.len();
        let prev_recs = previous.boundary_recommendations.len();
        let curr_recs = current.boundary_recommendations.len();
        let prev_suggestions = previous.suggestions.len();
        let curr_suggestions = current.suggestions.len();
        if prev_conflicts != curr_conflicts
            || prev_recs != curr_recs
            || prev_suggestions != curr_suggestions
        {
            alerts.push(format!(
                "alert policy conflicts {}->{} recommendations {}->{} suggestions {}->{}",
                prev_conflicts,
                curr_conflicts,
                prev_recs,
                curr_recs,
                prev_suggestions,
                curr_suggestions
            ));
        }
    }
    if show_suggestions && !show_all {
        let prev_suggestions = previous.suggestions.len();
        let curr_suggestions = current.suggestions.len();
        if prev_suggestions != curr_suggestions {
            alerts.push(format!(
                "alert suggestions {}->{}",
                prev_suggestions, curr_suggestions
            ));
        }
    }
    if show_all || view == "history" {
        let prev_receipts = previous.receipts.first().map(|receipt| receipt.id.as_str());
        let curr_receipts = current.receipts.first().map(|receipt| receipt.id.as_str());
        if prev_receipts != curr_receipts {
            alerts.push(format!(
                "alert history latest_receipt={}",
                curr_receipts.unwrap_or("none")
            ));
        }
    }

    alerts
}

fn coordination_snapshot_path(output: &Path) -> PathBuf {
    output.join("state").join("coordination-snapshot.json")
}

fn read_coordination_snapshot(output: &Path) -> anyhow::Result<Option<CoordinationSnapshotState>> {
    let path = coordination_snapshot_path(output);
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let state =
        serde_json::from_str(&content).with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(state))
}

fn write_coordination_snapshot(
    output: &Path,
    state: &CoordinationSnapshotState,
) -> anyhow::Result<()> {
    let path = coordination_snapshot_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&path, serde_json::to_string_pretty(state)? + "\n")
        .with_context(|| format!("write {}", path.display()))
}

fn build_coordination_alert_snapshot(response: &CoordinationResponse) -> CoordinationAlertSnapshot {
    CoordinationAlertSnapshot {
        message_count: response.inbox.messages.len(),
        owned_count: response.inbox.owned_tasks.len(),
        help_count: response.inbox.help_tasks.len(),
        review_count: response.inbox.review_tasks.len(),
        stale_peer_count: response.recovery.stale_peers.len(),
        reclaimable_claim_count: response.recovery.reclaimable_claims.len(),
        stalled_task_count: response.recovery.stalled_tasks.len(),
        policy_conflict_count: response.policy_conflicts.len(),
        recommendation_count: response.boundary_recommendations.len(),
        suggestion_count: response.suggestions.len(),
        latest_receipt_id: response.receipts.first().map(|receipt| receipt.id.clone()),
    }
}

fn render_coordination_snapshot_alerts(
    previous: Option<&CoordinationAlertSnapshot>,
    current: &CoordinationAlertSnapshot,
    view: &str,
) -> Vec<String> {
    let Some(previous) = previous else {
        return vec!["alert initial coordination snapshot".to_string()];
    };

    let show_all = matches!(view, "all" | "overview");
    let mut alerts = Vec::new();

    if show_all || view == "inbox" {
        if previous.message_count != current.message_count
            || previous.owned_count != current.owned_count
        {
            alerts.push(format!(
                "alert inbox messages {}->{} owned {}->{}",
                previous.message_count,
                current.message_count,
                previous.owned_count,
                current.owned_count
            ));
        }
    }
    if show_all || view == "requests" {
        if previous.help_count != current.help_count
            || previous.review_count != current.review_count
        {
            alerts.push(format!(
                "alert requests help {}->{} review {}->{}",
                previous.help_count,
                current.help_count,
                previous.review_count,
                current.review_count
            ));
        }
    }
    if show_all || view == "recovery" {
        if previous.stale_peer_count != current.stale_peer_count
            || previous.reclaimable_claim_count != current.reclaimable_claim_count
            || previous.stalled_task_count != current.stalled_task_count
        {
            alerts.push(format!(
                "alert recovery stale {}->{} reclaimable {}->{} stalled {}->{}",
                previous.stale_peer_count,
                current.stale_peer_count,
                previous.reclaimable_claim_count,
                current.reclaimable_claim_count,
                previous.stalled_task_count,
                current.stalled_task_count
            ));
        }
    }
    if show_all || view == "policy" {
        if previous.policy_conflict_count != current.policy_conflict_count
            || previous.recommendation_count != current.recommendation_count
            || previous.suggestion_count != current.suggestion_count
        {
            alerts.push(format!(
                "alert policy conflicts {}->{} recommendations {}->{} suggestions {}->{}",
                previous.policy_conflict_count,
                current.policy_conflict_count,
                previous.recommendation_count,
                current.recommendation_count,
                previous.suggestion_count,
                current.suggestion_count
            ));
        }
    }
    if show_all || view == "history" {
        if previous.latest_receipt_id != current.latest_receipt_id {
            alerts.push(format!(
                "alert history latest_receipt={}",
                current.latest_receipt_id.as_deref().unwrap_or("none")
            ));
        }
    }

    alerts
}

fn build_coordination_change_response(
    output: &Path,
    response: &CoordinationResponse,
    view: Option<&str>,
) -> anyhow::Result<CoordinationChangeResponse> {
    let view = view.unwrap_or("all").to_string();
    let previous = read_coordination_snapshot(output)?;
    let snapshot = build_coordination_alert_snapshot(response);
    let alerts = render_coordination_snapshot_alerts(
        previous.as_ref().map(|state| &state.snapshot),
        &snapshot,
        &view,
    );
    let change = CoordinationChangeResponse {
        bundle_root: response.bundle_root.clone(),
        current_session: response.current_session.clone(),
        view: view.clone(),
        changed: !alerts.is_empty(),
        alerts,
        snapshot: snapshot.clone(),
        generated_at: Utc::now(),
        previous_generated_at: previous.as_ref().map(|state| state.generated_at),
    };
    write_coordination_snapshot(
        output,
        &CoordinationSnapshotState {
            generated_at: change.generated_at,
            view,
            snapshot,
        },
    )?;
    Ok(change)
}

fn render_coordination_change_summary(response: &CoordinationChangeResponse) -> String {
    let mut lines = vec![
        format!(
            "coordination_changes bundle={} session={} view={} changed={}",
            response.bundle_root, response.current_session, response.view, response.changed
        ),
        format!(
            "snapshot messages={} owned={} help={} review={} stale={} reclaimable={} stalled={} conflicts={} recommendations={} suggestions={} latest_receipt={}",
            response.snapshot.message_count,
            response.snapshot.owned_count,
            response.snapshot.help_count,
            response.snapshot.review_count,
            response.snapshot.stale_peer_count,
            response.snapshot.reclaimable_claim_count,
            response.snapshot.stalled_task_count,
            response.snapshot.policy_conflict_count,
            response.snapshot.recommendation_count,
            response.snapshot.suggestion_count,
            response
                .snapshot
                .latest_receipt_id
                .as_deref()
                .unwrap_or("none"),
        ),
    ];
    for alert in &response.alerts {
        lines.push(format!("- {alert}"));
    }
    lines.join("\n")
}

fn suggest_boundary_recommendations(
    tasks: &[PeerTaskRecord],
    claims: &[SessionClaim],
    current_session: &str,
) -> Vec<String> {
    tasks
        .iter()
        .map(|task| {
            let branch_prefix = match task.coordination_mode.as_str() {
                "shared_review" => "review",
                "help_only" => "help",
                _ => "feat",
            };
            let branch_suffix = task
                .task_id
                .chars()
                .map(|ch| {
                    if ch.is_ascii_alphanumeric() {
                        ch.to_ascii_lowercase()
                    } else {
                        '-'
                    }
                })
                .collect::<String>()
                .split('-')
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join("-");
            let owned_scopes = claims
                .iter()
                .filter(|claim| claim.session.as_deref() == task.session.as_deref())
                .filter(|claim| {
                    task.claim_scopes.is_empty() || task.claim_scopes.contains(&claim.scope)
                })
                .map(|claim| claim.scope.clone())
                .collect::<Vec<_>>();
            let scope_hint = if !task.claim_scopes.is_empty() {
                task.claim_scopes.join(", ")
            } else if !owned_scopes.is_empty() {
                owned_scopes.join(", ")
            } else {
                "define a narrower scope".to_string()
            };
            let ownership_hint = if task.session.as_deref() == Some(current_session) {
                "active owner"
            } else {
                "remote owner"
            };
            format!(
                "task {} [{}] -> branch {}/{} | {} | scopes {}",
                task.task_id,
                task.coordination_mode,
                branch_prefix,
                if branch_suffix.is_empty() {
                    "task"
                } else {
                    &branch_suffix
                },
                ownership_hint,
                scope_hint
            )
        })
        .collect()
}

fn describe_resume_state_changes(
    previous: Option<&BundleResumeState>,
    current: &BundleResumeState,
) -> Vec<String> {
    let Some(previous) = previous else {
        return Vec::new();
    };

    let mut changes = Vec::new();

    if previous.focus != current.focus
        && let Some(focus) = current.focus.as_deref()
    {
        changes.push(format!("focus -> {}", compact_inline(focus, 120)));
    }
    if previous.pressure != current.pressure
        && let Some(pressure) = current.pressure.as_deref()
    {
        changes.push(format!("pressure -> {}", compact_inline(pressure, 120)));
    }
    if previous.next_recovery != current.next_recovery
        && let Some(next_recovery) = current.next_recovery.as_deref()
    {
        changes.push(format!(
            "next_recovery -> {}",
            compact_inline(next_recovery, 120)
        ));
    }
    if previous.lane != current.lane
        && let Some(lane) = current.lane.as_deref()
    {
        changes.push(format!("lane -> {}", compact_inline(lane, 120)));
    }
    if previous.working_records != current.working_records {
        changes.push(format!(
            "working {} -> {}",
            previous.working_records, current.working_records
        ));
    }
    if previous.inbox_items != current.inbox_items {
        changes.push(format!(
            "inbox {} -> {}",
            previous.inbox_items, current.inbox_items
        ));
    }
    if previous.rehydration_items != current.rehydration_items {
        changes.push(format!(
            "rehydration {} -> {}",
            previous.rehydration_items, current.rehydration_items
        ));
    }

    changes
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

fn write_native_agent_bridge_files(output: &Path) -> anyhow::Result<()> {
    let agents_dir = output.join("agents");
    fs::create_dir_all(&agents_dir).with_context(|| format!("create {}", agents_dir.display()))?;

    let claude_imports = agents_dir.join("CLAUDE_IMPORTS.md");
    fs::write(
        &claude_imports,
        format!(
            "# memd imports for Claude Code\n\nUse this file as the single import target from your project `CLAUDE.md`.\n\nAdd this line to the root `CLAUDE.md` for the workspace:\n\n`@.memd/agents/CLAUDE_IMPORTS.md`\n\nThen run `/memory` inside Claude Code to verify the imported memd files are loaded.\n\n## Imported memd memory files\n\n@../MEMD_MEMORY.md\n@CLAUDE_CODE_MEMORY.md\n\n## Notes\n\n- `memd resume --output {bundle} --intent current_task` refreshes the hot short-term lane.\n- `memd checkpoint --output {bundle} --content \"...\"` writes short-term state back into the same lane.\n- `memd handoff --output {bundle} --prompt` refreshes the shared handoff view.\n- dream and autodream output should flow back through `memd`, then Claude should pick it up through this import chain.\n- keep `memd` as the source of truth; treat this Claude import surface as a generated bridge.\n",
            bundle = output.display(),
        ),
    )
    .with_context(|| format!("write {}", claude_imports.display()))?;

    let claude_example = agents_dir.join("CLAUDE.md.example");
    fs::write(
        &claude_example,
        "# Claude Code project memory\n\n@.memd/agents/CLAUDE_IMPORTS.md\n",
    )
    .with_context(|| format!("write {}", claude_example.display()))?;

    Ok(())
}

fn write_memory_markdown_files(output: &Path, markdown: &str) -> anyhow::Result<()> {
    let root_memory = output.join("MEMD_MEMORY.md");
    fs::write(&root_memory, markdown)
        .with_context(|| format!("write {}", root_memory.display()))?;

    let agents_dir = output.join("agents");
    if let Some(parent) = agents_dir.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::create_dir_all(&agents_dir).with_context(|| format!("create {}", agents_dir.display()))?;
    for file_name in [
        "CODEX_MEMORY.md",
        "CLAUDE_CODE_MEMORY.md",
        "OPENCLAW_MEMORY.md",
        "OPENCODE_MEMORY.md",
    ] {
        let path = agents_dir.join(file_name);
        fs::write(&path, markdown).with_context(|| format!("write {}", path.display()))?;
    }
    Ok(())
}

fn write_bundle_eval_artifacts(output: &Path, response: &BundleEvalResponse) -> anyhow::Result<()> {
    let evals_dir = output.join("evals");
    fs::create_dir_all(&evals_dir).with_context(|| format!("create {}", evals_dir.display()))?;

    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let json = serde_json::to_string_pretty(response)? + "\n";
    let markdown = render_bundle_eval_markdown(response);

    let latest_json = evals_dir.join("latest.json");
    let latest_md = evals_dir.join("latest.md");
    let timestamped_json = evals_dir.join(format!("{timestamp}.json"));
    let timestamped_md = evals_dir.join(format!("{timestamp}.md"));

    fs::write(&latest_json, &json).with_context(|| format!("write {}", latest_json.display()))?;
    fs::write(&latest_md, &markdown).with_context(|| format!("write {}", latest_md.display()))?;
    fs::write(&timestamped_json, &json)
        .with_context(|| format!("write {}", timestamped_json.display()))?;
    fs::write(&timestamped_md, &markdown)
        .with_context(|| format!("write {}", timestamped_md.display()))?;

    Ok(())
}

fn gap_reports_dir(output: &Path) -> PathBuf {
    output.join("gaps")
}

fn improvement_reports_dir(output: &Path) -> PathBuf {
    output.join("improvements")
}

fn project_root_from_bundle(output: &Path) -> &Path {
    output.parent().unwrap_or_else(|| Path::new("."))
}

fn read_text_file(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn read_recent_commits(root: &Path, limit: usize) -> Vec<String> {
    let limit = limit.max(1).min(64);
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("log")
        .arg(format!("-n{limit}"))
        .arg("--oneline")
        .output();

    let Ok(output) = output else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    let raw = String::from_utf8_lossy(&output.stdout);
    raw.lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .take(limit)
        .collect()
}

fn gap_to_improvement_snapshot(response: &GapReport) -> ImprovementGapSnapshot {
    ImprovementGapSnapshot {
        candidate_count: response.candidate_count,
        high_priority_count: response.high_priority_count,
        eval_status: response.eval_status.clone(),
        eval_score: response.eval_score,
        eval_score_delta: response.eval_score_delta,
        top_priorities: response.top_priorities.clone(),
        generated_at: response.generated_at,
    }
}

fn improvement_progress(previous: &GapReport, current: &GapReport) -> bool {
    if current.candidate_count < previous.candidate_count {
        return true;
    }
    if current.high_priority_count < previous.high_priority_count {
        return true;
    }
    if let (Some(previous_score), Some(current_score)) = (previous.eval_score, current.eval_score) {
        if current_score > previous_score {
            return true;
        }
    } else if current.eval_score.is_some() && previous.eval_score.is_none() {
        return true;
    }
    previous.top_priorities != current.top_priorities
}

fn build_improvement_actions(
    gap: &GapReport,
    coordination: Option<&CoordinationResponse>,
) -> Vec<ImprovementAction> {
    let mut actions = Vec::new();
    let mut seen = std::collections::HashSet::<(
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )>::new();

    let add = |actions: &mut Vec<ImprovementAction>,
               seen: &mut std::collections::HashSet<(
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )>,
               action: &str,
               priority: &str,
               target_session: Option<String>,
               scope: Option<String>,
               task_id: Option<String>,
               message_id: Option<String>,
               reason: &str| {
        let key = (
            action.to_string(),
            target_session.clone(),
            scope.clone(),
            task_id.clone(),
            message_id.clone(),
            Some(reason.to_string()),
        );
        if seen.insert(key) {
            actions.push(ImprovementAction {
                action: action.to_string(),
                priority: priority.to_string(),
                target_session,
                scope,
                task_id,
                message_id,
                reason: reason.to_string(),
            });
        }
    };

    for candidate in &gap.candidates {
        match candidate.id.as_str() {
            "memory:low_eval_score"
            | "memory:below_target_eval_score"
            | "memory:no_eval_snapshot" => {
                add(
                    &mut actions,
                    &mut seen,
                    "refresh_eval",
                    "high",
                    None,
                    None,
                    None,
                    None,
                    &candidate.recommendation,
                );
            }
            "memory:weak_working_lane"
            | "memory:empty_context_lane"
            | "memory:empty_rehydration_queue"
            | "memory:missing_active_workspace_lane"
            | "memory:inbox_growth"
            | "memory:resume_state_weak"
            | "memory:resume_state_inbox_backlog"
            | "memory:missing_resume_state" => {
                add(
                    &mut actions,
                    &mut seen,
                    "refresh_resume",
                    "high",
                    None,
                    None,
                    None,
                    None,
                    &candidate.recommendation,
                );
            }
            "coordination:message_backlog" => {
                add(
                    &mut actions,
                    &mut seen,
                    "refresh_resume",
                    "medium",
                    None,
                    None,
                    None,
                    None,
                    &candidate.recommendation,
                );
            }
            _ => {}
        }
    }

    if let Some(coordination) = coordination {
        for suggestion in &coordination.suggestions {
            match suggestion.action.as_str() {
                "ack_message" => {
                    if let Some(message_id) = suggestion.message_id.clone() {
                        add(
                            &mut actions,
                            &mut seen,
                            "ack_message",
                            &suggestion.priority,
                            None,
                            None,
                            None,
                            Some(message_id),
                            &suggestion.reason,
                        );
                    }
                }
                "recover_session" => {
                    if let Some(session) = suggestion.stale_session.clone() {
                        add(
                            &mut actions,
                            &mut seen,
                            "recover_session",
                            &suggestion.priority,
                            Some(session),
                            None,
                            None,
                            None,
                            &suggestion.reason,
                        );
                    }
                }
                "assign_scope" => {
                    if suggestion.task_id.is_some() && suggestion.scope.is_some() {
                        add(
                            &mut actions,
                            &mut seen,
                            "assign_scope",
                            &suggestion.priority,
                            suggestion.target_session.clone(),
                            suggestion.scope.clone(),
                            suggestion.task_id.clone(),
                            None,
                            &suggestion.reason,
                        );
                    }
                }
                "request_help" | "request_review" => {
                    if suggestion.task_id.is_some() && suggestion.target_session.is_some() {
                        add(
                            &mut actions,
                            &mut seen,
                            &suggestion.action,
                            &suggestion.priority,
                            suggestion.target_session.clone(),
                            None,
                            suggestion.task_id.clone(),
                            None,
                            &suggestion.reason,
                        );
                    }
                }
                _ => {}
            }
        }
    }

    if actions.len() > 8 {
        actions.truncate(8);
    }
    actions
}

async fn apply_improvement_action(
    action: &ImprovementAction,
    output: &Path,
    base_url: &str,
) -> anyhow::Result<String> {
    match action.action.as_str() {
        "refresh_eval" => {
            let response = eval_bundle_memory(
                &EvalArgs {
                    output: output.to_path_buf(),
                    limit: None,
                    rehydration_limit: None,
                    write: false,
                    fail_below: None,
                    fail_on_regression: false,
                    summary: false,
                },
                base_url,
            )
            .await?;
            Ok(format!(
                "status={} score={}",
                response.status, response.score
            ))
        }
        "refresh_resume" => {
            let runtime = read_bundle_runtime_config(output)?;
            let snapshot = read_bundle_resume(
                &ResumeArgs {
                    output: output.to_path_buf(),
                    project: runtime.as_ref().and_then(|value| value.project.clone()),
                    namespace: runtime.as_ref().and_then(|value| value.namespace.clone()),
                    agent: runtime.as_ref().and_then(|value| value.agent.clone()),
                    workspace: runtime.as_ref().and_then(|value| value.workspace.clone()),
                    visibility: runtime.as_ref().and_then(|value| value.visibility.clone()),
                    route: runtime
                        .as_ref()
                        .and_then(|value| value.route.clone())
                        .or(Some("auto".to_string())),
                    intent: runtime
                        .as_ref()
                        .and_then(|value| value.intent.clone())
                        .or(Some("current_task".to_string())),
                    limit: Some(8),
                    rehydration_limit: Some(4),
                    semantic: false,
                    prompt: false,
                    summary: false,
                },
                base_url,
            )
            .await?;
            write_bundle_memory_files(output, &snapshot, None).await?;
            Ok(format!(
                "working={} inbox={} rehydration={}",
                snapshot.working.records.len(),
                snapshot.inbox.items.len(),
                snapshot.working.rehydration_queue.len(),
            ))
        }
        "ack_message" => {
            let response = run_messages_command(
                &MessagesArgs {
                    output: output.to_path_buf(),
                    send: false,
                    inbox: true,
                    ack: action.message_id.clone(),
                    target_session: None,
                    kind: None,
                    request_help: false,
                    request_review: false,
                    assign_scope: None,
                    scope: None,
                    content: None,
                    summary: false,
                },
                base_url,
            )
            .await?;
            Ok(format!("acked {} message(s)", response.messages.len()))
        }
        "recover_session" => {
            let response = run_coordination_command(
                &CoordinationArgs {
                    output: output.to_path_buf(),
                    view: Some("all".to_string()),
                    changes_only: false,
                    watch: false,
                    interval_secs: 30,
                    recover_session: action.target_session.clone(),
                    to_session: None,
                    summary: false,
                },
                base_url,
            )
            .await?;
            Ok(format!(
                "recovered stale session pressure (stale_peers={})",
                response.recovery.stale_peers.len()
            ))
        }
        "assign_scope" => {
            let response = run_tasks_command(
                &TasksArgs {
                    output: output.to_path_buf(),
                    upsert: false,
                    assign_to_session: action.target_session.clone(),
                    target_session: None,
                    task_id: action.task_id.clone(),
                    title: None,
                    description: None,
                    status: None,
                    mode: None,
                    scope: Vec::new(),
                    request_help: false,
                    request_review: false,
                    all: false,
                    summary: false,
                },
                base_url,
            )
            .await?;
            Ok(format!("assigned task count={}", response.tasks.len()))
        }
        "request_help" => {
            let response = run_tasks_command(
                &TasksArgs {
                    output: output.to_path_buf(),
                    upsert: false,
                    assign_to_session: None,
                    target_session: action.target_session.clone(),
                    task_id: action.task_id.clone(),
                    title: None,
                    description: None,
                    status: None,
                    mode: None,
                    scope: Vec::new(),
                    request_help: true,
                    request_review: false,
                    all: false,
                    summary: false,
                },
                base_url,
            )
            .await?;
            Ok(format!(
                "requested help on {} task(s)",
                response.tasks.len()
            ))
        }
        "request_review" => {
            let response = run_tasks_command(
                &TasksArgs {
                    output: output.to_path_buf(),
                    upsert: false,
                    assign_to_session: None,
                    target_session: action.target_session.clone(),
                    task_id: action.task_id.clone(),
                    title: None,
                    description: None,
                    status: None,
                    mode: None,
                    scope: Vec::new(),
                    request_help: false,
                    request_review: true,
                    all: false,
                    summary: false,
                },
                base_url,
            )
            .await?;
            Ok(format!(
                "requested review on {} task(s)",
                response.tasks.len()
            ))
        }
        _ => anyhow::bail!("unknown improvement action: {}", action.action),
    }
}

fn collect_gap_plan_evidence(project_root: &Path) -> Vec<String> {
    let planning_root = project_root.join(".planning");
    let mut evidence = Vec::new();
    let roadmap = read_text_file(&planning_root.join("ROADMAP.md"));
    let state = read_text_file(&planning_root.join("STATE.md"));
    let project = read_text_file(&planning_root.join("PROJECT.md"));

    if let Some(roadmap) = roadmap {
        let lines = roadmap
            .lines()
            .filter(|value| value.contains("Phase") && value.contains("v6"))
            .take(4)
            .collect::<Vec<_>>();
        if !lines.is_empty() {
            evidence.push(format!("roadmap phases: {}", lines.join(" | ")));
        }
    }
    if let Some(state) = state {
        if let Some(open_loops) = state
            .lines()
            .filter(|line| line.starts_with("- ") && line.contains("phase"))
            .next()
        {
            evidence.push(format!("state signal: {open_loops}"));
        }
        if let Some(open_block) = state.split("## Open Loops").nth(1) {
            let next = open_block
                .lines()
                .take(3)
                .filter(|value| value.starts_with("- "))
                .collect::<Vec<_>>();
            if !next.is_empty() {
                evidence.push(format!("state open loops: {}", next.join(" | ")));
            }
        }
    }
    if let Some(project) = project {
        if let Some(core) = project
            .lines()
            .find(|line| line.starts_with("##") && line.contains("Core"))
        {
            evidence.push(format!("project: {core}"));
        }
    }

    evidence
}

async fn gap_report(args: &GapArgs) -> anyhow::Result<GapReport> {
    let runtime = read_bundle_runtime_config(&args.output)?;
    let project_root = project_root_from_bundle(&args.output);
    let base_url = runtime
        .as_ref()
        .and_then(|value| value.base_url.clone())
        .unwrap_or_else(|| "http://127.0.0.1:8787".to_string());
    let limit = args.limit.unwrap_or(8);
    let recent_commits = read_recent_commits(project_root, args.recent_commits.unwrap_or(8));
    let mut evidence = collect_gap_plan_evidence(project_root);

    if evidence.is_empty() {
        evidence.push("planning evidence unavailable in .planning".to_string());
    }

    let baseline = read_latest_gap_report(&args.output).ok().flatten();
    let eval = read_latest_bundle_eval(&args.output).ok().flatten();
    if let Some(eval) = &eval {
        evidence.push(format!(
            "eval baseline score: {} ({})",
            eval.score, eval.status
        ));
    } else {
        evidence.push("no previous memd eval snapshot in .memd/evals/latest.json".to_string());
    }

    let resume = read_bundle_resume(
        &ResumeArgs {
            output: args.output.clone(),
            project: runtime.as_ref().and_then(|value| value.project.clone()),
            namespace: runtime.as_ref().and_then(|value| value.namespace.clone()),
            agent: runtime.as_ref().and_then(|value| value.agent.clone()),
            workspace: runtime.as_ref().and_then(|value| value.workspace.clone()),
            visibility: runtime.as_ref().and_then(|value| value.visibility.clone()),
            route: runtime
                .as_ref()
                .and_then(|value| value.route.clone())
                .or(Some("auto".to_string())),
            intent: runtime
                .as_ref()
                .and_then(|value| value.intent.clone())
                .or(Some("current_task".to_string())),
            limit: Some(limit),
            rehydration_limit: Some(4),
            semantic: false,
            prompt: false,
            summary: false,
        },
        &base_url,
    )
    .await
    .ok();

    let snapshot_state = read_bundle_resume_state(&args.output).ok().flatten();

    let runtime_session = runtime.as_ref().and_then(|value| value.session.clone());
    let coordination = if runtime_session.is_some() {
        run_coordination_command(
            &CoordinationArgs {
                output: args.output.clone(),
                view: None,
                changes_only: false,
                watch: false,
                interval_secs: 30,
                recover_session: None,
                to_session: None,
                summary: false,
            },
            &base_url,
        )
        .await
        .ok()
    } else {
        None
    };

    let candidates = build_gap_candidates(
        &args.output,
        &runtime,
        &resume,
        snapshot_state.as_ref(),
        eval.as_ref(),
        coordination.as_ref(),
        &recent_commits,
        &mut evidence,
    );
    let candidates = prioritize_gap_candidates(candidates, limit);

    let mut recommendations = candidates
        .iter()
        .take(3)
        .map(|candidate| candidate.recommendation.clone())
        .collect::<Vec<_>>();
    if recommendations.is_empty() {
        recommendations
            .push("run memd gap after collecting 12+ recent commits and a fresh eval".to_string());
    }

    if !recent_commits.is_empty() {
        evidence.push(format!("recent_commits={} checked", recent_commits.len()));
    }

    let high_priorities = candidates
        .iter()
        .filter(|candidate| candidate.severity == "high")
        .map(|candidate| candidate.id.clone())
        .collect::<Vec<_>>();
    let mut response = GapReport {
        bundle_root: args.output.display().to_string(),
        project: runtime.as_ref().and_then(|value| value.project.clone()),
        namespace: runtime.as_ref().and_then(|value| value.namespace.clone()),
        agent: runtime.as_ref().and_then(|value| value.agent.clone()),
        session: runtime_session,
        workspace: runtime.as_ref().and_then(|value| value.workspace.clone()),
        visibility: runtime.as_ref().and_then(|value| value.visibility.clone()),
        limit,
        commits_checked: recent_commits.len(),
        eval_status: eval.as_ref().map(|value| value.status.clone()),
        eval_score: eval.as_ref().map(|value| value.score),
        eval_score_delta: baseline
            .as_ref()
            .and_then(|value| value.eval_score)
            .and_then(|value| eval_score_delta(value, eval.as_ref())),
        candidate_count: candidates.len(),
        high_priority_count: high_priorities.len(),
        candidates,
        top_priorities: high_priorities,
        recommendations,
        changes: Vec::new(),
        evidence,
        generated_at: Utc::now(),
        previous_candidate_count: baseline.as_ref().map(|value| value.candidate_count),
    };
    response.changes = evaluate_gap_changes(&response, baseline.as_ref());
    Ok(response)
}

async fn run_improvement_loop(
    args: &ImproveArgs,
    base_url: &str,
) -> anyhow::Result<ImprovementReport> {
    let runtime = read_bundle_runtime_config(&args.output)?;
    let started_at = Utc::now();
    let mut iterations = Vec::new();
    let mut converged = false;

    let initial_report = gap_report(&GapArgs {
        output: args.output.clone(),
        limit: args.limit,
        recent_commits: args.recent_commits,
        write: false,
        summary: false,
    })
    .await?;

    let initial_snapshot = gap_to_improvement_snapshot(&initial_report);
    let mut current_gap = initial_report.clone();
    let mut final_changes = initial_report.changes.clone();
    let mut previous_gap: Option<GapReport> = Some(initial_report.clone());
    let mut final_gap: Option<GapReport> = Some(initial_report);

    for iteration in 0..args.max_iterations {
        let coordination = if runtime
            .as_ref()
            .and_then(|value| value.session.as_ref())
            .is_some()
        {
            run_coordination_command(
                &CoordinationArgs {
                    output: args.output.clone(),
                    view: Some("all".to_string()),
                    changes_only: false,
                    watch: false,
                    interval_secs: 30,
                    recover_session: None,
                    to_session: None,
                    summary: false,
                },
                base_url,
            )
            .await
            .ok()
        } else {
            None
        };

        let mut planned_actions = build_improvement_actions(&current_gap, coordination.as_ref());
        planned_actions.truncate(6);

        let pre_gap = gap_to_improvement_snapshot(&current_gap);
        let mut executed_actions = Vec::new();

        if !args.apply || planned_actions.is_empty() {
            final_gap = Some(current_gap.clone());
            final_changes = current_gap.changes.clone();
            converged = true;
            iterations.push(ImprovementIteration {
                iteration,
                pre_gap,
                planned_actions,
                executed_actions,
                post_gap: None,
                generated_at: Utc::now(),
            });
            break;
        }

        let mut stop_due_to_failure = false;
        for action in &planned_actions {
            let result = match apply_improvement_action(action, &args.output, base_url).await {
                Ok(detail) => ImprovementActionResult {
                    action: action.action.clone(),
                    status: "applied".to_string(),
                    detail,
                },
                Err(error) => {
                    stop_due_to_failure = true;
                    ImprovementActionResult {
                        action: action.action.clone(),
                        status: "failed".to_string(),
                        detail: error.to_string(),
                    }
                }
            };
            executed_actions.push(result);
        }

        if stop_due_to_failure {
            final_gap = Some(current_gap.clone());
            final_changes = current_gap.changes.clone();
            iterations.push(ImprovementIteration {
                iteration,
                pre_gap,
                planned_actions,
                executed_actions,
                post_gap: None,
                generated_at: Utc::now(),
            });
            break;
        }

        current_gap = gap_report(&GapArgs {
            output: args.output.clone(),
            limit: args.limit,
            recent_commits: args.recent_commits,
            write: false,
            summary: false,
        })
        .await?;
        final_changes = current_gap.changes.clone();
        let post_gap = gap_to_improvement_snapshot(&current_gap);
        final_gap = Some(current_gap.clone());

        iterations.push(ImprovementIteration {
            iteration,
            pre_gap,
            planned_actions,
            executed_actions,
            post_gap: Some(post_gap),
            generated_at: Utc::now(),
        });

        if let Some(previous_gap) = previous_gap.as_ref() {
            if !improvement_progress(previous_gap, &current_gap) {
                converged = true;
                break;
            }
        }
        previous_gap = Some(current_gap.clone());

        if iteration + 1 >= args.max_iterations {
            break;
        }
    }

    let final_snapshot = final_gap
        .as_ref()
        .map(gap_to_improvement_snapshot)
        .or_else(|| Some(initial_snapshot.clone()));

    if iterations.is_empty() {
        iterations.push(ImprovementIteration {
            iteration: 0,
            pre_gap: initial_snapshot.clone(),
            planned_actions: Vec::new(),
            executed_actions: Vec::new(),
            post_gap: Some(initial_snapshot.clone()),
            generated_at: Utc::now(),
        });
        final_gap = Some(current_gap);
        final_changes = final_gap
            .as_ref()
            .map_or_else(Vec::new, |gap| gap.changes.clone());
    }
    if final_changes.is_empty()
        && let Some(gap) = final_gap.as_ref()
    {
        final_changes = gap.changes.clone();
    }

    Ok(ImprovementReport {
        bundle_root: args.output.display().to_string(),
        project: runtime.as_ref().and_then(|value| value.project.clone()),
        namespace: runtime.as_ref().and_then(|value| value.namespace.clone()),
        agent: runtime.as_ref().and_then(|value| value.agent.clone()),
        session: runtime.as_ref().and_then(|value| value.session.clone()),
        workspace: runtime.as_ref().and_then(|value| value.workspace.clone()),
        visibility: runtime.as_ref().and_then(|value| value.visibility.clone()),
        max_iterations: args.max_iterations,
        apply: args.apply,
        started_at,
        completed_at: Utc::now(),
        converged,
        initial_gap: Some(initial_snapshot),
        final_gap: final_snapshot,
        final_changes,
        iterations,
    })
}

fn write_improvement_artifacts(output: &Path, response: &ImprovementReport) -> anyhow::Result<()> {
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

fn build_gap_candidates(
    output: &Path,
    runtime: &Option<BundleRuntimeConfig>,
    resume: &Option<ResumeSnapshot>,
    state: Option<&BundleResumeState>,
    eval: Option<&BundleEvalResponse>,
    coordination: Option<&CoordinationResponse>,
    recent_commits: &[String],
    evidence: &mut Vec<String>,
) -> Vec<GapCandidate> {
    let mut candidates = Vec::new();
    let add = |candidates: &mut Vec<GapCandidate>,
               area: &str,
               signal: &str,
               priority: u8,
               evidence: Vec<String>,
               recommendation: &str| {
        candidates.push(GapCandidate {
            id: format!("{}:{signal}", area),
            area: area.to_string(),
            priority,
            severity: if priority >= 85 {
                "high".to_string()
            } else if priority >= 65 {
                "medium".to_string()
            } else {
                "low".to_string()
            },
            signal: signal.to_string(),
            evidence,
            recommendation: recommendation.to_string(),
        });
    };

    if let Some(eval) = eval {
        if eval.score < 70 {
            add(
                &mut candidates,
                "memory",
                "low_eval_score",
                95,
                vec![format!(
                    "memd eval score {} with status {}",
                    eval.score, eval.status
                )],
                "run `memd eval --write --summary` and address top recommendations before the next context switch",
            );
        } else if eval.score < 82 {
            add(
                &mut candidates,
                "memory",
                "below_target_eval_score",
                76,
                vec![format!(
                    "eval score {} indicates medium risk, status {}",
                    eval.score, eval.status
                )],
                "close immediate resume-pressure gaps (context, rehydration, inbox pressure) and rerun `memd eval`",
            );
        }
        if eval.inbox_items >= 6 {
            add(
                &mut candidates,
                "memory",
                "inbox_pressure",
                72,
                vec![format!(
                    "eval inbox_items={} indicates pressure",
                    eval.inbox_items
                )],
                "triage/ack backlog with `memd coordination` then rerun resume",
            );
        }
    } else {
        add(
            &mut candidates,
            "memory",
            "no_eval_snapshot",
            82,
            vec!["no .memd/evals/latest.json was available".to_string()],
            "run `memd eval --write --summary` to establish a baseline before gap scoring",
        );
    }

    if let Some(snapshot) = resume {
        if snapshot.working.records.len() <= 1 {
            add(
                &mut candidates,
                "memory",
                "weak_working_lane",
                86,
                vec![format!(
                    "working.records={}",
                    snapshot.working.records.len()
                )],
                "capture durable and short-term lane before resuming high-cost tasks",
            );
        }
        if snapshot.context.records.is_empty() {
            add(
                &mut candidates,
                "memory",
                "empty_context_lane",
                84,
                vec!["compact context returned no records for current route/intent".to_string()],
                "verify active project/namespace and reset route/intent defaults",
            );
        }
        if snapshot.working.rehydration_queue.is_empty() {
            add(
                &mut candidates,
                "memory",
                "empty_rehydration_queue",
                66,
                vec!["working.rehydration_queue empty".to_string()],
                "write a checkpointable deep-context item and rerun handoff/resume",
            );
        }
        if snapshot.workspace.is_some() && snapshot.workspaces.workspaces.is_empty() {
            add(
                &mut candidates,
                "memory",
                "missing_active_workspace_lane",
                70,
                vec!["active workspace had no workspace lane visibility".to_string()],
                "repair workspace visibility and rehydrate shared lane state",
            );
        }
        if snapshot.inbox.items.len() >= 7 {
            add(
                &mut candidates,
                "memory",
                "inbox_growth",
                68,
                vec![format!("inbox items={}", snapshot.inbox.items.len())],
                "drain high-urgency items and clear stale messages before the next decision",
            );
        }
    } else if let Some(state) = state {
        if state.working_records <= 1 {
            add(
                &mut candidates,
                "memory",
                "resume_state_weak",
                80,
                vec![
                    "resume snapshot unavailable; using last saved state".to_string(),
                    format!("working_records={}", state.working_records),
                ],
                "resume the bundle and immediately run `memd eval --write --summary`",
            );
        }
        if state.inbox_items >= 7 {
            add(
                &mut candidates,
                "memory",
                "resume_state_inbox_backlog",
                64,
                vec![format!("saved inbox_items={}", state.inbox_items)],
                "refresh resume and inspect backlog with `memd coordination --summary`",
            );
        }
    } else {
        add(
            &mut candidates,
            "memory",
            "missing_resume_state",
            74,
            vec!["resume state was not available locally".to_string()],
            "run `memd resume` once so gap reports get live lane evidence",
        );
    }

    if let Some(coordination) = coordination {
        if !coordination.recovery.stale_peers.is_empty() {
            add(
                &mut candidates,
                "coordination",
                "stale_peers_recovery",
                90,
                vec![format!(
                    "stale peers={}",
                    coordination.recovery.stale_peers.len()
                )],
                "recover stale sessions before assigning new claims",
            );
        }
        if !coordination.policy_conflicts.is_empty() {
            add(
                &mut candidates,
                "coordination",
                "policy_conflicts",
                84,
                vec![format!(
                    "policy_conflicts={}",
                    coordination.policy_conflicts.len()
                )],
                "resolve conflicts by explicit assign/recover actions",
            );
        }
        if coordination.inbox.messages.len() >= 6 {
            add(
                &mut candidates,
                "coordination",
                "message_backlog",
                76,
                vec![format!(
                    "inbox messages={}",
                    coordination.inbox.messages.len()
                )],
                "ack now and reduce queue churn before adding new tasks",
            );
        }
        if coordination.suggestions.len() >= 3 {
            add(
                &mut candidates,
                "coordination",
                "stale_action_pressure",
                62,
                vec![format!(
                    "coordination suggestions={} pending",
                    coordination.suggestions.len()
                )],
                "execute highest-priority coordination suggestion via bounded actions",
            );
        }
    } else if !coordination_exists(output) {
        add(
            &mut candidates,
            "coordination",
            "coordination_unreachable",
            60,
            vec!["coordination snapshot was unavailable".to_string()],
            "configure bundle session/base_url and rerun `memd gap`",
        );
    }

    if let Some(runtime) = runtime {
        if let Some(agent) = runtime.agent.as_ref() {
            let mut session_hint = String::new();
            if let Some(session) = runtime.session.as_ref() {
                session_hint.push_str(session);
            }
            if !recent_commits.is_empty() {
                evidence.push(format!("agent={agent} session={session_hint}"));
            }
        }
    }

    if recent_commits.is_empty() {
        add(
            &mut candidates,
            "research_loop",
            "no_recent_commits",
            58,
            vec!["no local commits discovered for configured limit".to_string()],
            "run `git log` with commits available and compare gap deltas across windows",
        );
    } else {
        evidence.push(format!(
            "recent commits: {}",
            recent_commits.first().map_or("none", |value| value)
        ));
    }

    candidates
}

fn evaluate_gap_changes(current: &GapReport, baseline: Option<&GapReport>) -> Vec<String> {
    let mut changes = Vec::new();
    let current_top = current.candidate_count;
    if let Some(baseline) = baseline {
        if baseline.candidate_count != current_top {
            changes.push(format!(
                "candidate_count {} -> {}",
                baseline.candidate_count, current_top
            ));
        }
        if baseline.eval_score != current.eval_score {
            changes.push(format!(
                "eval_score {:?} -> {:?}",
                baseline.eval_score, current.eval_score
            ));
        }
    }
    if current.eval_status.is_some() {
        changes.push(format!(
            "eval_status={}",
            current.eval_status.as_deref().unwrap_or("none")
        ));
    }
    changes
}

fn eval_score_delta(previous: u8, current: Option<&BundleEvalResponse>) -> Option<i32> {
    current.map(|value| i32::from(value.score) - i32::from(previous))
}

fn prioritize_gap_candidates(mut candidates: Vec<GapCandidate>, limit: usize) -> Vec<GapCandidate> {
    candidates.sort_by(|left, right| {
        right
            .priority
            .cmp(&left.priority)
            .then_with(|| left.area.cmp(&right.area))
    });
    candidates.into_iter().take(limit).collect()
}

fn coordination_exists(output: &Path) -> bool {
    output
        .join("state")
        .join("coordination-snapshot.json")
        .exists()
}

fn gap_artifact_paths(output: &Path, name: &str) -> PathBuf {
    gap_reports_dir(output).join(name)
}

fn write_gap_artifacts(output: &Path, response: &GapReport) -> anyhow::Result<()> {
    let gap_dir = gap_reports_dir(output);
    fs::create_dir_all(&gap_dir).with_context(|| format!("create {}", gap_dir.display()))?;

    let baseline_json = gap_artifact_paths(output, "latest.json");
    let baseline_md = gap_artifact_paths(output, "latest.md");
    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let timestamp_json = gap_artifact_paths(output, &format!("{timestamp}.json"));
    let timestamp_md = gap_artifact_paths(output, &format!("{timestamp}.md"));
    let markdown = render_gap_markdown(response);
    let json = serde_json::to_string_pretty(response)? + "\n";

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

fn read_latest_gap_report(output: &Path) -> anyhow::Result<Option<GapReport>> {
    let path = gap_artifact_paths(output, "latest.json");
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let report = serde_json::from_str::<GapReport>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(report))
}

fn render_gap_markdown(response: &GapReport) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd gap report\n\n");
    markdown.push_str(&format!(
        "- bundle: {}\n- project: {}\n- namespace: {}\n- agent: {}\n- session: {}\n- workspace: {}\n- visibility: {}\n- eval_status: {}\n- eval_score: {}\n- eval_score_delta: {}\n- candidate_count: {}\n- high_priority_count: {}\n- previous_candidate_count: {}\n- commits_checked: {}\n- generated_at: {}\n",
        response.bundle_root,
        response.project.as_deref().unwrap_or("none"),
        response.namespace.as_deref().unwrap_or("none"),
        response.agent.as_deref().unwrap_or("none"),
        response.session.as_deref().unwrap_or("none"),
        response.workspace.as_deref().unwrap_or("none"),
        response.visibility.as_deref().unwrap_or("all"),
        response.eval_status.clone().unwrap_or_else(|| "none".to_string()),
        response
            .eval_score
            .map(|value: u8| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        response.eval_score_delta
            .map(|value: i32| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        response.candidate_count,
        response.high_priority_count,
        response.previous_candidate_count.unwrap_or(0),
        response.commits_checked,
        response.generated_at,
    ));

    markdown.push_str("\n## Evidence\n\n");
    if response.evidence.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for item in &response.evidence {
            markdown.push_str(&format!("- {}\n", item));
        }
    }

    markdown.push_str("\n## Candidates\n\n");
    if response.candidates.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for candidate in &response.candidates {
            markdown.push_str(&format!(
                "- [{}] {} {} (priority={})\n",
                candidate.severity, candidate.area, candidate.signal, candidate.priority
            ));
            markdown.push_str(&format!("  - action: {}\n", candidate.recommendation));
            for entry in &candidate.evidence {
                markdown.push_str(&format!("  - evidence: {}\n", entry));
            }
        }
    }

    markdown.push_str("\n## Recommendations\n\n");
    if response.recommendations.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for recommendation in &response.recommendations {
            markdown.push_str(&format!("- {}\n", recommendation));
        }
    }

    markdown.push_str("\n## Priorities\n\n");
    if response.top_priorities.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for item in &response.top_priorities {
            markdown.push_str(&format!("- {}\n", item));
        }
    }

    markdown.push_str("\n## Changes\n\n");
    if response.changes.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for change in &response.changes {
            markdown.push_str(&format!("- {}\n", change));
        }
    }

    markdown
}

fn render_bundle_eval_markdown(response: &BundleEvalResponse) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd bundle evaluation\n\n");
    markdown.push_str(&format!(
        "- bundle: {}\n- status: {}\n- score: {}\n- baseline_score: {}\n- score_delta: {}\n- agent: {}\n- workspace: {}\n- visibility: {}\n",
        response.bundle_root,
        response.status,
        response.score,
        response
            .baseline_score
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        response
            .score_delta
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        response.agent.as_deref().unwrap_or("none"),
        response.workspace.as_deref().unwrap_or("none"),
        response.visibility.as_deref().unwrap_or("none"),
    ));
    markdown.push_str(&format!(
        "- working_records: {}\n- context_records: {}\n- rehydration_items: {}\n- inbox_items: {}\n- workspace_lanes: {}\n- semantic_hits: {}\n",
        response.working_records,
        response.context_records,
        response.rehydration_items,
        response.inbox_items,
        response.workspace_lanes,
        response.semantic_hits,
    ));

    markdown.push_str("\n## Findings\n\n");
    if response.findings.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for finding in &response.findings {
            markdown.push_str(&format!("- {}\n", finding));
        }
    }

    markdown.push_str("\n## Changes\n\n");
    if response.changes.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for change in &response.changes {
            markdown.push_str(&format!("- {}\n", change));
        }
    }

    markdown.push_str("\n## Recommendations\n\n");
    if response.recommendations.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for recommendation in &response.recommendations {
            markdown.push_str(&format!("- {}\n", recommendation));
        }
    }

    markdown
}

fn render_agent_shell_profile(output: &Path, env_agent: Option<&str>) -> String {
    let mut script = format!(
        "#!/usr/bin/env bash\nset -euo pipefail\n\nexport MEMD_BUNDLE_ROOT=\"{}\"\nsource \"$MEMD_BUNDLE_ROOT/backend.env\" 2>/dev/null || true\nsource \"$MEMD_BUNDLE_ROOT/env\"\n",
        compact_bundle_value(output.to_string_lossy().as_ref()),
    );
    if let Some(env_agent) = env_agent {
        script.push_str(&format!(
            "export MEMD_AGENT=\"{}\"\n",
            compact_bundle_value(env_agent)
        ));
    }
    script
        .push_str("exec memd resume --output \"$MEMD_BUNDLE_ROOT\" --intent current_task \"$@\"\n");
    script
}

fn render_agent_ps1_profile(output: &Path, env_agent: Option<&str>) -> String {
    let mut script = format!(
        "$env:MEMD_BUNDLE_ROOT = \"{}\"\n$bundleBackendEnv = Join-Path $env:MEMD_BUNDLE_ROOT \"backend.env.ps1\"\nif (Test-Path $bundleBackendEnv) {{ . $bundleBackendEnv }}\n. (Join-Path $env:MEMD_BUNDLE_ROOT \"env.ps1\")\n",
        escape_ps1(output.to_string_lossy().as_ref()),
    );
    if let Some(env_agent) = env_agent {
        script.push_str(&format!(
            "$env:MEMD_AGENT = \"{}\"\n",
            escape_ps1(env_agent)
        ));
    }
    script.push_str("memd resume --output $env:MEMD_BUNDLE_ROOT --intent current_task\n");
    script
}

fn render_bundle_memory_markdown(
    snapshot: &ResumeSnapshot,
    handoff: Option<&HandoffSnapshot>,
) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd memory\n\n");
    markdown.push_str(&format!(
        "- project: {}\n- namespace: {}\n- agent: {}\n- workspace: {}\n- visibility: {}\n- route: {}\n- intent: {}\n",
        snapshot.project.as_deref().unwrap_or("none"),
        snapshot.namespace.as_deref().unwrap_or("none"),
        snapshot.agent.as_deref().unwrap_or("none"),
        snapshot.workspace.as_deref().unwrap_or("none"),
        snapshot.visibility.as_deref().unwrap_or("all"),
        snapshot.route,
        snapshot.intent,
    ));

    let current_task = render_current_task_bundle_snapshot(snapshot);
    if !current_task.is_empty() {
        markdown.push_str("\n## Current Task Snapshot\n\n");
        markdown.push_str(&current_task);
    }

    if !snapshot.change_summary.is_empty() {
        markdown.push_str("\n## Since Last Resume\n\n");
        for change in snapshot.change_summary.iter().take(6) {
            markdown.push_str("- ");
            markdown.push_str(change.trim());
            markdown.push('\n');
        }
    }

    markdown.push_str("\n## Working Memory\n\n");
    if snapshot.working.records.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for record in snapshot.working.records.iter().take(10) {
            markdown.push_str("- ");
            markdown.push_str(record.record.trim());
            markdown.push('\n');
        }
    }

    if !snapshot.working.rehydration_queue.is_empty() {
        markdown.push_str("\n## Rehydration Queue\n\n");
        for artifact in snapshot.working.rehydration_queue.iter().take(6) {
            markdown.push_str(&format!(
                "- {}: {}\n",
                artifact.label,
                artifact.summary.trim()
            ));
        }
    }

    if !snapshot.inbox.items.is_empty() {
        markdown.push_str("\n## Inbox\n\n");
        for item in snapshot.inbox.items.iter().take(6) {
            markdown.push_str(&format!(
                "- {:?} {:?}: {}\n",
                item.item.kind,
                item.item.status,
                item.item.content.trim()
            ));
            if !item.reasons.is_empty() {
                markdown.push_str(&format!("  - reasons: {}\n", item.reasons.join(", ")));
            }
        }
    }

    if !snapshot.workspaces.workspaces.is_empty() {
        markdown.push_str("\n## Workspace Lanes\n\n");
        for workspace in snapshot.workspaces.workspaces.iter().take(6) {
            markdown.push_str(&format!(
                "- {} / {} / {} | visibility {} | items {} | trust {:.2}\n",
                workspace.project.as_deref().unwrap_or("none"),
                workspace.namespace.as_deref().unwrap_or("none"),
                workspace.workspace.as_deref().unwrap_or("none"),
                memory_visibility_label(workspace.visibility),
                workspace.item_count,
                workspace.trust_score
            ));
        }
    }

    if let Some(semantic) = snapshot
        .semantic
        .as_ref()
        .filter(|semantic| !semantic.items.is_empty())
    {
        markdown.push_str("\n## Semantic Recall\n\n");
        for item in semantic.items.iter().take(5) {
            markdown.push_str(&format!(
                "- {}{}{}\n",
                compact_resume_rag_text(&item.content, 220),
                item.source
                    .as_deref()
                    .map(|source| format!(" | source {source}"))
                    .unwrap_or_default(),
                format!(" | score {:.2}", item.score)
            ));
        }
    }

    if let Some(handoff) = handoff {
        markdown.push_str("\n## Source Lanes\n\n");
        if handoff.sources.sources.is_empty() {
            markdown.push_str("- none\n");
        } else {
            for source in handoff.sources.sources.iter().take(6) {
                markdown.push_str(&format!(
                    "- {} / {} | workspace {} | visibility {} | items {} | trust {:.2} | confidence {:.2}\n",
                    source.source_agent.as_deref().unwrap_or("none"),
                    source.source_system.as_deref().unwrap_or("none"),
                    source.workspace.as_deref().unwrap_or("none"),
                    memory_visibility_label(source.visibility),
                    source.item_count,
                    source.trust_score,
                    source.avg_confidence
                ));
            }
        }
        markdown.push_str("\n## Handoff Notes\n\n");
        markdown.push_str("- this file was refreshed from a shared handoff bundle\n");
        markdown.push_str("- dream/consolidation output should feed this same file so durable memory and distilled memory stay aligned\n");
    }

    markdown
}

fn render_current_task_bundle_snapshot(snapshot: &ResumeSnapshot) -> String {
    let mut markdown = String::new();

    if let Some(focus) = snapshot.working.records.first() {
        markdown.push_str("- focus: ");
        markdown.push_str(focus.record.trim());
        markdown.push('\n');
    }

    if let Some(blocker) = snapshot.inbox.items.first() {
        markdown.push_str(&format!(
            "- pressure: {:?} {:?}: {}\n",
            blocker.item.kind,
            blocker.item.status,
            blocker.item.content.trim()
        ));
    }

    if let Some(next) = snapshot.working.rehydration_queue.first() {
        markdown.push_str(&format!(
            "- next_recovery: {}: {}\n",
            next.label,
            next.summary.trim()
        ));
    }

    if let Some(lane) = snapshot.workspaces.workspaces.first() {
        markdown.push_str(&format!(
            "- lane: {} / {} / {} | visibility {} | trust {:.2}\n",
            lane.project.as_deref().unwrap_or("none"),
            lane.namespace.as_deref().unwrap_or("none"),
            lane.workspace.as_deref().unwrap_or("none"),
            memory_visibility_label(lane.visibility),
            lane.trust_score
        ));
    }

    markdown
}

fn write_bundle_backend_env(output: &Path, config: &BundleConfig) -> anyhow::Result<()> {
    let backend_env = output.join("backend.env");
    let backend_env_ps1 = output.join("backend.env.ps1");
    let rag = &config.backend.rag;

    let mut shell = String::new();
    shell.push_str(&format!(
        "MEMD_BUNDLE_SCHEMA_VERSION={}\n",
        config.schema_version
    ));
    shell.push_str(&format!("MEMD_BUNDLE_BACKEND_PROVIDER={}\n", rag.provider));
    shell.push_str(&format!(
        "MEMD_BUNDLE_BACKEND_ENABLED={}\n",
        if rag.enabled { "true" } else { "false" }
    ));
    if let Some(url) = rag.url.as_deref() {
        shell.push_str(&format!("MEMD_RAG_URL={url}\n"));
    }
    fs::write(&backend_env, shell).with_context(|| format!("write {}", backend_env.display()))?;

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
        ps1.push_str(&format!("$env:MEMD_RAG_URL = \"{}\"\n", escape_ps1(url)));
    }
    fs::write(&backend_env_ps1, ps1)
        .with_context(|| format!("write {}", backend_env_ps1.display()))?;

    Ok(())
}

async fn read_bundle_status(output: &Path, base_url: &str) -> anyhow::Result<serde_json::Value> {
    let client = MemdClient::new(base_url)?;
    let health = client.healthz().await.ok();
    let runtime = read_bundle_runtime_config(output)?;
    let heartbeat = read_bundle_heartbeat(output)?;
    let resume_preview = if output.join("config.json").exists() && health.is_some() {
        let preview = read_bundle_resume(
            &ResumeArgs {
                output: output.to_path_buf(),
                project: None,
                namespace: None,
                agent: None,
                workspace: None,
                visibility: None,
                route: None,
                intent: Some("current_task".to_string()),
                limit: Some(4),
                rehydration_limit: Some(2),
                semantic: false,
                prompt: false,
                summary: false,
            },
            base_url,
        )
        .await
        .ok();
        preview.map(|snapshot| {
            serde_json::json!({
                "project": snapshot.project,
                "namespace": snapshot.namespace,
                "agent": snapshot.agent,
                "session": runtime.as_ref().and_then(|config| config.session.clone()),
                "workspace": snapshot.workspace,
                "visibility": snapshot.visibility,
                "route": snapshot.route,
                "intent": snapshot.intent,
                "context_records": snapshot.context.records.len(),
                "working_records": snapshot.working.records.len(),
                "inbox_items": snapshot.inbox.items.len(),
                "workspace_lanes": snapshot.workspaces.workspaces.len(),
                "rehydration_queue": snapshot.working.rehydration_queue.len(),
                "semantic_hits": snapshot.semantic.as_ref().map(|semantic| semantic.items.len()).unwrap_or(0),
                "change_summary": snapshot.change_summary,
                "focus": snapshot.working.records.first().map(|record| record.record.clone()),
                "pressure": snapshot.inbox.items.first().map(|item| item.item.content.clone()),
                "next_recovery": snapshot.working.rehydration_queue.first().map(|item| format!("{}: {}", item.label, item.summary)),
            })
        })
    } else {
        None
    };
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
        "active_agent": runtime.as_ref().and_then(|config| config.agent.clone()),
        "defaults": runtime.as_ref().map(|config| serde_json::json!({
            "project": config.project,
            "namespace": config.namespace,
            "agent": config.agent,
            "session": config.session,
            "effective_agent": config.agent.as_ref().map(|agent| compose_agent_identity(agent, config.session.as_deref())),
            "base_url": config.base_url,
            "route": config.route,
            "intent": config.intent,
            "workspace": config.workspace,
            "visibility": config.visibility,
            "auto_short_term_capture": config.auto_short_term_capture,
        })),
        "heartbeat": heartbeat.as_ref().map(|value| serde_json::json!({
            "session": value.session,
            "agent": value.agent,
            "effective_agent": value.effective_agent,
            "presence": heartbeat_presence_label(value.last_seen),
            "base_url": value.base_url,
            "host": value.host,
            "pid": value.pid,
            "focus": value.focus,
            "pressure": value.pressure,
            "next_recovery": value.next_recovery,
            "last_seen": value.last_seen,
        })),
        "resume_preview": resume_preview,
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

fn read_bundle_runtime_config(output: &Path) -> anyhow::Result<Option<BundleRuntimeConfig>> {
    let config_path = output.join("config.json");
    if !config_path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let config: BundleConfigFile =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
    Ok(Some(BundleRuntimeConfig {
        project: config.project,
        namespace: config.namespace,
        agent: config.agent,
        session: config.session,
        base_url: config.base_url,
        route: config.route,
        intent: config.intent,
        workspace: config.workspace,
        visibility: config.visibility,
        auto_short_term_capture: config.auto_short_term_capture,
    }))
}

fn bundle_auto_short_term_capture_enabled(output: &Path) -> anyhow::Result<bool> {
    if let Ok(value) = std::env::var("MEMD_AUTO_SHORT_TERM_CAPTURE") {
        let value = value.trim().to_ascii_lowercase();
        return Ok(matches!(value.as_str(), "1" | "true" | "yes" | "on"));
    }

    Ok(read_bundle_runtime_config(output)?
        .map(|config| config.auto_short_term_capture)
        .unwrap_or(true))
}

fn read_project_awareness(args: &AwarenessArgs) -> anyhow::Result<ProjectAwarenessResponse> {
    let current_bundle = if args.output.is_absolute() {
        args.output.clone()
    } else {
        std::env::current_dir()?.join(&args.output)
    };
    let current_bundle = fs::canonicalize(&current_bundle).unwrap_or(current_bundle);
    let current_project = current_bundle
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let scan_root = if let Some(root) = args.root.as_ref() {
        if root.is_absolute() {
            root.clone()
        } else {
            std::env::current_dir()?.join(root)
        }
    } else {
        current_project
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| current_project.clone())
    };
    let scan_root = fs::canonicalize(&scan_root).unwrap_or(scan_root);

    let mut entries = Vec::new();
    let mut base_url_counts = std::collections::BTreeMap::<String, usize>::new();
    for child in fs::read_dir(&scan_root)
        .with_context(|| format!("read awareness root {}", scan_root.display()))?
    {
        let child = child?;
        if !child.file_type()?.is_dir() {
            continue;
        }

        let project_dir = child.path();
        let bundle_root = project_dir.join(".memd");
        let config_path = bundle_root.join("config.json");
        if !config_path.exists() {
            continue;
        }

        let canonical_bundle = fs::canonicalize(&bundle_root).unwrap_or(bundle_root.clone());
        if !args.include_current && canonical_bundle == current_bundle {
            continue;
        }

        let runtime = read_bundle_runtime_config(&bundle_root)?.unwrap_or(BundleRuntimeConfig {
            project: None,
            namespace: None,
            agent: None,
            session: None,
            base_url: None,
            route: None,
            intent: None,
            workspace: None,
            visibility: None,
            auto_short_term_capture: true,
        });
        let state = read_bundle_resume_state(&bundle_root)?;
        let heartbeat = read_bundle_heartbeat(&bundle_root)?;
        let claims = read_bundle_claims(&bundle_root)?;
        let state_path = bundle_resume_state_path(&bundle_root);
        let heartbeat_path = bundle_heartbeat_state_path(&bundle_root);
        let last_updated = if heartbeat_path.exists() {
            fs::metadata(&heartbeat_path)
                .ok()
                .and_then(|metadata| metadata.modified().ok())
                .map(DateTime::<Utc>::from)
        } else if state_path.exists() {
            fs::metadata(&state_path)
                .ok()
                .and_then(|metadata| metadata.modified().ok())
                .map(DateTime::<Utc>::from)
        } else {
            fs::metadata(&config_path)
                .ok()
                .and_then(|metadata| metadata.modified().ok())
                .map(DateTime::<Utc>::from)
        };

        entries.push(ProjectAwarenessEntry {
            project_dir: project_dir.display().to_string(),
            bundle_root: bundle_root.display().to_string(),
            project: runtime.project,
            namespace: runtime.namespace,
            effective_agent: runtime
                .agent
                .as_deref()
                .map(|agent| compose_agent_identity(agent, runtime.session.as_deref())),
            agent: runtime.agent,
            session: runtime.session,
            base_url: runtime.base_url.clone(),
            presence: heartbeat
                .as_ref()
                .map(|value| heartbeat_presence_label(value.last_seen).to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            host: heartbeat.as_ref().and_then(|value| value.host.clone()),
            pid: heartbeat.as_ref().and_then(|value| value.pid),
            active_claims: claims
                .claims
                .iter()
                .filter(|claim| claim.expires_at > Utc::now())
                .count(),
            workspace: heartbeat
                .as_ref()
                .and_then(|value| value.workspace.clone())
                .or(runtime.workspace),
            visibility: heartbeat
                .as_ref()
                .and_then(|value| value.visibility.clone())
                .or(runtime.visibility),
            focus: heartbeat
                .as_ref()
                .and_then(|value| value.focus.clone())
                .or_else(|| state.as_ref().and_then(|value| value.focus.clone())),
            pressure: heartbeat
                .as_ref()
                .and_then(|value| value.pressure.clone())
                .or_else(|| state.as_ref().and_then(|value| value.pressure.clone())),
            next_recovery: heartbeat
                .as_ref()
                .and_then(|value| value.next_recovery.clone())
                .or_else(|| state.as_ref().and_then(|value| value.next_recovery.clone())),
            last_updated,
        });
        if let Some(url) = entries
            .last()
            .and_then(|entry| entry.base_url.as_ref())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
        {
            *base_url_counts.entry(url).or_insert(0) += 1;
        }
    }

    entries.sort_by(|left, right| left.project_dir.cmp(&right.project_dir));
    let collisions = base_url_counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(url, count)| format!("base_url {} used by {} bundles", url, count))
        .collect::<Vec<_>>();

    Ok(ProjectAwarenessResponse {
        root: scan_root.display().to_string(),
        current_bundle: current_bundle.display().to_string(),
        collisions,
        entries,
    })
}

fn render_project_awareness_summary(response: &ProjectAwarenessResponse) -> String {
    let mut lines = vec![format!(
        "awareness root={} bundles={} collisions={}",
        response.root,
        response.entries.len(),
        response.collisions.len()
    )];
    for collision in &response.collisions {
        lines.push(format!("! {}", collision));
    }
    for entry in &response.entries {
        let focus = entry
            .focus
            .as_deref()
            .map(|value| compact_inline(value, 56))
            .unwrap_or_else(|| "none".to_string());
        let pressure = entry
            .pressure
            .as_deref()
            .map(|value| compact_inline(value, 56))
            .unwrap_or_else(|| "none".to_string());
        lines.push(format!(
            "- {} | presence={} claims={} ns={} agent={} session={} base_url={} workspace={} visibility={} focus=\"{}\" pressure=\"{}\"",
            entry.project.as_deref().unwrap_or("unknown"),
            entry.presence,
            entry.active_claims,
            entry.namespace.as_deref().unwrap_or("none"),
            entry.effective_agent
                .as_deref()
                .or(entry.agent.as_deref())
                .unwrap_or("none"),
            entry.session.as_deref().unwrap_or("none"),
            entry.base_url.as_deref().unwrap_or("none"),
            entry.workspace.as_deref().unwrap_or("none"),
            entry.visibility.as_deref().unwrap_or("all"),
            focus,
            pressure,
        ));
    }
    lines.join("\n")
}

fn resolve_target_session_bundle(
    output: &Path,
    target_session: &str,
) -> anyhow::Result<Option<ProjectAwarenessEntry>> {
    let current_project = if output.is_absolute() {
        output.to_path_buf()
    } else {
        std::env::current_dir()?.join(output)
    };
    let awareness = read_project_awareness(&AwarenessArgs {
        output: current_project,
        root: None,
        include_current: true,
        summary: false,
    })?;

    Ok(awareness.entries.into_iter().find(|entry| {
        entry.session.as_deref() == Some(target_session)
            || entry.effective_agent.as_deref() == Some(target_session)
    }))
}

async fn read_bundle_resume(args: &ResumeArgs, base_url: &str) -> anyhow::Result<ResumeSnapshot> {
    let runtime = read_bundle_runtime_config(&args.output)?;
    let base_agent = args
        .agent
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.agent.clone()));
    let session = runtime.as_ref().and_then(|config| config.session.clone());
    let project = args
        .project
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.project.clone()));
    let namespace = args
        .namespace
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.namespace.clone()));
    let agent = base_agent
        .as_deref()
        .map(|value| compose_agent_identity(value, session.as_deref()));
    let workspace = args
        .workspace
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.workspace.clone()));
    let visibility_raw = args.visibility.clone().or_else(|| {
        runtime
            .as_ref()
            .and_then(|config| config.visibility.clone())
    });
    let route_raw = args
        .route
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.route.clone()))
        .unwrap_or_else(|| "auto".to_string());
    let intent_raw = args
        .intent
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.intent.clone()))
        .unwrap_or_else(|| "general".to_string());
    let base_url = runtime
        .as_ref()
        .and_then(|config| config.base_url.clone())
        .unwrap_or_else(|| base_url.to_string());

    let visibility = visibility_raw
        .as_deref()
        .map(parse_memory_visibility_value)
        .transpose()?;
    let route = parse_retrieval_route(Some(route_raw.clone()))?;
    let intent = parse_retrieval_intent(Some(intent_raw.clone()))?;
    let limit = args.limit.or(Some(8));
    let rehydration_limit = args.rehydration_limit.or(Some(4));

    let client = MemdClient::new(&base_url)?;
    let context = client
        .context_compact(&memd_schema::ContextRequest {
            project: project.clone(),
            agent: agent.clone(),
            workspace: workspace.clone(),
            visibility,
            route,
            intent,
            limit,
            max_chars_per_item: Some(220),
        })
        .await?;
    let working = client
        .working(&WorkingMemoryRequest {
            project: project.clone(),
            agent: agent.clone(),
            workspace: workspace.clone(),
            visibility,
            route,
            intent,
            limit,
            max_chars_per_item: Some(220),
            max_total_chars: Some(1600),
            rehydration_limit,
            auto_consolidate: Some(false),
        })
        .await?;
    let inbox = client
        .inbox(&memd_schema::MemoryInboxRequest {
            project: project.clone(),
            namespace: namespace.clone(),
            workspace: workspace.clone(),
            visibility,
            belief_branch: None,
            route,
            intent,
            limit: Some(6),
        })
        .await?;
    let workspaces = client
        .workspace_memory(&memd_schema::WorkspaceMemoryRequest {
            project: project.clone(),
            namespace: namespace.clone(),
            workspace: workspace.clone(),
            visibility,
            source_agent: None,
            source_system: None,
            limit: Some(6),
        })
        .await?;
    let semantic = if let Some(rag) = maybe_rag_client_for_bundle(&args.output)? {
        if args.semantic {
            let query = build_resume_rag_query(
                project.as_deref(),
                workspace.as_deref(),
                &intent_raw,
                &working,
                &context,
            );
            if query.trim().is_empty() {
                None
            } else {
                rag.retrieve(&RagRetrieveRequest {
                    query,
                    project: project.clone(),
                    namespace: namespace.clone(),
                    mode: RagRetrieveMode::Auto,
                    limit: Some(4),
                    include_cross_modal: false,
                })
                .await
                .ok()
                .filter(|response| !response.items.is_empty())
            }
        } else {
            None
        }
    } else {
        None
    };

    let current_state = BundleResumeState {
        focus: working.records.first().map(|record| record.record.clone()),
        pressure: inbox.items.first().map(|item| item.item.content.clone()),
        next_recovery: working
            .rehydration_queue
            .first()
            .map(|item| format!("{}: {}", item.label, item.summary)),
        lane: workspaces.workspaces.first().map(|lane| {
            format!(
                "{} / {} / {}",
                lane.project.as_deref().unwrap_or("none"),
                lane.namespace.as_deref().unwrap_or("none"),
                lane.workspace.as_deref().unwrap_or("none")
            )
        }),
        working_records: working.records.len(),
        inbox_items: inbox.items.len(),
        rehydration_items: working.rehydration_queue.len(),
        recorded_at: Utc::now(),
    };
    let previous_state = read_bundle_resume_state(&args.output)?;
    let change_summary = describe_resume_state_changes(previous_state.as_ref(), &current_state);
    let resume_state_age_minutes = previous_state.as_ref().map(BundleResumeState::age_minutes);
    let refresh_recommended = resume_state_age_minutes.is_some_and(|age_minutes| age_minutes >= 15)
        || working.truncated
        || working.remaining_chars <= 200
        || working.records.len() >= 8
        || inbox.items.len() >= 5
        || working.rehydration_queue.len() >= 4
        || context.records.len() >= 6;

    Ok(ResumeSnapshot {
        project,
        namespace,
        agent,
        workspace,
        visibility: visibility_raw,
        route: route_raw,
        intent: intent_raw,
        context,
        working,
        inbox,
        workspaces,
        semantic,
        change_summary,
        resume_state_age_minutes,
        refresh_recommended,
    })
}

async fn read_bundle_handoff(
    args: &HandoffArgs,
    base_url: &str,
) -> anyhow::Result<HandoffSnapshot> {
    let target = if let Some(target_session) = args.target_session.as_deref() {
        resolve_target_session_bundle(&args.output, target_session)?
    } else {
        None
    };
    let target_bundle = target
        .as_ref()
        .map(|entry| PathBuf::from(&entry.bundle_root))
        .unwrap_or_else(|| args.output.clone());

    let resume = read_bundle_resume(
        &ResumeArgs {
            output: target_bundle.clone(),
            project: args.project.clone(),
            namespace: args.namespace.clone(),
            agent: args.agent.clone(),
            workspace: args.workspace.clone(),
            visibility: args.visibility.clone(),
            route: args.route.clone(),
            intent: args.intent.clone(),
            limit: args.limit,
            rehydration_limit: args.rehydration_limit,
            semantic: args.semantic,
            prompt: false,
            summary: false,
        },
        base_url,
    )
    .await?;

    let runtime = read_bundle_runtime_config(&target_bundle)?;
    let base_url = runtime
        .as_ref()
        .and_then(|config| config.base_url.clone())
        .unwrap_or_else(|| base_url.to_string());
    let client = MemdClient::new(&base_url)?;
    let sources = client
        .source_memory(&SourceMemoryRequest {
            project: resume.project.clone(),
            namespace: resume.namespace.clone(),
            workspace: resume.workspace.clone(),
            visibility: resume
                .visibility
                .as_deref()
                .map(parse_memory_visibility_value)
                .transpose()?,
            source_agent: None,
            source_system: None,
            limit: args.source_limit.or(Some(6)),
        })
        .await?;

    Ok(HandoffSnapshot {
        generated_at: Utc::now(),
        resume,
        sources,
        target_session: target.and_then(|entry| entry.session),
        target_bundle: Some(target_bundle.display().to_string()),
    })
}

async fn eval_bundle_memory(args: &EvalArgs, base_url: &str) -> anyhow::Result<BundleEvalResponse> {
    let baseline = read_latest_bundle_eval(&args.output)?;
    let snapshot = read_bundle_resume(
        &ResumeArgs {
            output: args.output.clone(),
            project: None,
            namespace: None,
            agent: None,
            workspace: None,
            visibility: None,
            route: None,
            intent: None,
            limit: args.limit.or(Some(8)),
            rehydration_limit: args.rehydration_limit.or(Some(4)),
            semantic: true,
            prompt: false,
            summary: false,
        },
        base_url,
    )
    .await?;

    let runtime = read_bundle_runtime_config(&args.output)?;
    let mut score = 100i32;
    let mut findings = Vec::new();

    if snapshot.working.records.is_empty() {
        score -= 30;
        findings.push("no working memory records returned from bundle resume".to_string());
    }
    if snapshot.context.records.is_empty() {
        score -= 15;
        findings.push("no compact context records returned from bundle resume".to_string());
    }
    if snapshot.working.rehydration_queue.is_empty() {
        score -= 10;
        findings.push("rehydration queue is empty; deeper evidence recovery is weak".to_string());
    }
    if snapshot.workspace.is_some() && snapshot.workspaces.workspaces.is_empty() {
        score -= 15;
        findings.push("active workspace is set but no workspace lanes were returned".to_string());
    }
    if snapshot
        .semantic
        .as_ref()
        .is_some_and(|semantic| semantic.items.is_empty())
    {
        score -= 5;
        findings.push("semantic recall is configured but returned no items".to_string());
    }
    if snapshot.inbox.items.len() >= 6 {
        score -= 10;
        findings.push("inbox pressure is high; resume lane may need maintenance".to_string());
    }

    let score = score.clamp(0, 100) as u8;
    let status = if score >= 85 {
        "strong"
    } else if score >= 65 {
        "usable"
    } else {
        "weak"
    };

    let baseline_score = baseline.as_ref().map(|value| value.score);
    let score_delta = baseline_score.map(|baseline| score as i32 - baseline as i32);
    let changes = baseline
        .as_ref()
        .map(|baseline| describe_eval_changes(baseline, score, &snapshot))
        .unwrap_or_default();
    let recommendations = build_eval_recommendations(&snapshot, score);

    Ok(BundleEvalResponse {
        bundle_root: args.output.display().to_string(),
        project: snapshot.project.clone(),
        namespace: snapshot.namespace.clone(),
        agent: snapshot
            .agent
            .clone()
            .or_else(|| runtime.as_ref().and_then(|config| config.agent.clone())),
        workspace: snapshot.workspace.clone(),
        visibility: snapshot.visibility.clone(),
        status: status.to_string(),
        score,
        working_records: snapshot.working.records.len(),
        context_records: snapshot.context.records.len(),
        rehydration_items: snapshot.working.rehydration_queue.len(),
        inbox_items: snapshot.inbox.items.len(),
        workspace_lanes: snapshot.workspaces.workspaces.len(),
        semantic_hits: snapshot
            .semantic
            .as_ref()
            .map(|semantic| semantic.items.len())
            .unwrap_or(0),
        findings,
        baseline_score,
        score_delta,
        changes,
        recommendations,
    })
}

fn read_latest_bundle_eval(output: &Path) -> anyhow::Result<Option<BundleEvalResponse>> {
    let path = output.join("evals").join("latest.json");
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let eval = serde_json::from_str::<BundleEvalResponse>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(eval))
}

fn describe_eval_changes(
    baseline: &BundleEvalResponse,
    score: u8,
    snapshot: &ResumeSnapshot,
) -> Vec<String> {
    let mut changes = Vec::new();

    if baseline.score != score {
        changes.push(format!("score {} -> {}", baseline.score, score));
    }

    let working_records = snapshot.working.records.len();
    if baseline.working_records != working_records {
        changes.push(format!(
            "working {} -> {}",
            baseline.working_records, working_records
        ));
    }

    let context_records = snapshot.context.records.len();
    if baseline.context_records != context_records {
        changes.push(format!(
            "context {} -> {}",
            baseline.context_records, context_records
        ));
    }

    let rehydration_items = snapshot.working.rehydration_queue.len();
    if baseline.rehydration_items != rehydration_items {
        changes.push(format!(
            "rehydration {} -> {}",
            baseline.rehydration_items, rehydration_items
        ));
    }

    let inbox_items = snapshot.inbox.items.len();
    if baseline.inbox_items != inbox_items {
        changes.push(format!("inbox {} -> {}", baseline.inbox_items, inbox_items));
    }

    let workspace_lanes = snapshot.workspaces.workspaces.len();
    if baseline.workspace_lanes != workspace_lanes {
        changes.push(format!(
            "lanes {} -> {}",
            baseline.workspace_lanes, workspace_lanes
        ));
    }

    let semantic_hits = snapshot
        .semantic
        .as_ref()
        .map(|semantic| semantic.items.len())
        .unwrap_or(0);
    if baseline.semantic_hits != semantic_hits {
        changes.push(format!(
            "semantic {} -> {}",
            baseline.semantic_hits, semantic_hits
        ));
    }

    changes
}

fn eval_failure_reason(
    response: &BundleEvalResponse,
    fail_below: Option<u8>,
    fail_on_regression: bool,
) -> Option<String> {
    if let Some(threshold) = fail_below {
        if response.score < threshold {
            return Some(format!(
                "bundle evaluation score {} fell below required threshold {}",
                response.score, threshold
            ));
        }
    }

    if fail_on_regression && response.score_delta.is_some_and(|delta| delta < 0) {
        let baseline = response
            .baseline_score
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let delta = response.score_delta.unwrap_or_default();
        return Some(format!(
            "bundle evaluation regressed from baseline {} to {} (delta {})",
            baseline, response.score, delta
        ));
    }

    None
}

fn build_eval_recommendations(snapshot: &ResumeSnapshot, score: u8) -> Vec<String> {
    let mut recommendations = Vec::new();

    if snapshot.working.records.is_empty() {
        recommendations.push(
            "capture durable memory with `memd remember --output .memd ...` before relying on resume"
                .to_string(),
        );
    }
    if snapshot.context.records.is_empty() {
        recommendations.push(
            "review bundle route/intent defaults and verify compact context retrieval for the active lane"
                .to_string(),
        );
    }
    if snapshot.working.rehydration_queue.is_empty() {
        recommendations.push(
            "promote richer evidence or inspect key items with `memd explain --follow` so resume can rehydrate deeper context"
                .to_string(),
        );
    }
    if snapshot.workspace.is_some() && snapshot.workspaces.workspaces.is_empty() {
        recommendations.push(
            "repair workspace or visibility lanes so shared memory is visible to the active bundle"
                .to_string(),
        );
    }
    if snapshot.inbox.items.len() >= 6 {
        recommendations.push(
            "drain inbox pressure with repair or promotion passes before the next handoff or resume"
                .to_string(),
        );
    }
    if snapshot
        .semantic
        .as_ref()
        .is_some_and(|semantic| semantic.items.is_empty())
    {
        recommendations.push(
            "check the LightRAG index or sync path before depending on semantic fallback"
                .to_string(),
        );
    }
    if score < 85 {
        recommendations.push(
            "write a fresh baseline with `memd eval --output .memd --write --summary` after corrective changes"
                .to_string(),
        );
    }

    recommendations
}

async fn remember_with_bundle_defaults(
    args: &RememberArgs,
    base_url: &str,
) -> anyhow::Result<memd_schema::StoreMemoryResponse> {
    let runtime = read_bundle_runtime_config(&args.output)?;
    let session = runtime.as_ref().and_then(|config| config.session.clone());
    let project = args
        .project
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.project.clone()));
    let namespace = args
        .namespace
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.namespace.clone()));
    let workspace = args
        .workspace
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.workspace.clone()));
    let visibility_raw = args.visibility.clone().or_else(|| {
        runtime
            .as_ref()
            .and_then(|config| config.visibility.clone())
    });
    let base_url = runtime
        .as_ref()
        .and_then(|config| config.base_url.clone())
        .unwrap_or_else(|| base_url.to_string());
    let source_agent = args
        .source_agent
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.agent.clone()))
        .as_deref()
        .map(|value| compose_agent_identity(value, session.as_deref()));

    let content = if let Some(content) = &args.content {
        content.clone()
    } else if let Some(path) = &args.input {
        fs::read_to_string(path)
            .with_context(|| format!("read remember input file {}", path.display()))?
    } else if args.stdin {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("read remember payload from stdin")?;
        buffer
    } else {
        anyhow::bail!("provide --content, --input, or --stdin");
    };

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
        .unwrap_or_else(|| {
            if project.is_some() {
                MemoryScope::Project
            } else {
                MemoryScope::Synced
            }
        });
    let source_quality = args
        .source_quality
        .as_deref()
        .map(parse_source_quality_value)
        .transpose()?;
    let visibility = visibility_raw
        .as_deref()
        .map(parse_memory_visibility_value)
        .transpose()?;

    let client = MemdClient::new(&base_url)?;
    client
        .store(&memd_schema::StoreMemoryRequest {
            content,
            kind,
            scope,
            project,
            namespace,
            workspace,
            visibility,
            belief_branch: None,
            source_agent,
            source_system: args.source_system.clone().or(Some("memd".to_string())),
            source_path: args.source_path.clone(),
            source_quality,
            confidence: args.confidence,
            ttl_seconds: args.ttl_seconds,
            last_verified_at: None,
            supersedes: Vec::new(),
            tags: args.tag.clone(),
            status: Some(MemoryStatus::Active),
        })
        .await
}

async fn checkpoint_with_bundle_defaults(
    args: &CheckpointArgs,
    base_url: &str,
) -> anyhow::Result<memd_schema::StoreMemoryResponse> {
    let translated = checkpoint_as_remember_args(args);
    remember_with_bundle_defaults(&translated, base_url).await
}

async fn auto_checkpoint_bundle_event(
    output: &Path,
    base_url: &str,
    source_path: &str,
    content: String,
    tags: Vec<String>,
    confidence: f32,
) -> anyhow::Result<()> {
    if read_bundle_runtime_config(output)?.is_none() {
        return Ok(());
    }
    if !bundle_auto_short_term_capture_enabled(output)? {
        return Ok(());
    }
    if content.trim().is_empty() {
        return Ok(());
    }

    checkpoint_with_bundle_defaults(
        &CheckpointArgs {
            output: output.to_path_buf(),
            project: None,
            namespace: None,
            workspace: None,
            visibility: None,
            source_path: Some(source_path.to_string()),
            confidence: Some(confidence),
            ttl_seconds: Some(86_400),
            tag: tags,
            content: Some(content),
            input: None,
            stdin: false,
        },
        base_url,
    )
    .await?;

    let snapshot = read_bundle_resume(
        &ResumeArgs {
            output: output.to_path_buf(),
            project: None,
            namespace: None,
            agent: None,
            workspace: None,
            visibility: None,
            route: None,
            intent: Some("current_task".to_string()),
            limit: Some(8),
            rehydration_limit: Some(4),
            semantic: false,
            prompt: false,
            summary: false,
        },
        base_url,
    )
    .await?;
    write_bundle_memory_files(output, &snapshot, None).await?;
    Ok(())
}

async fn auto_checkpoint_compaction_packet(
    packet: &CompactionPacket,
    base_url: &str,
) -> anyhow::Result<()> {
    let Some(output) = resolve_default_bundle_root()? else {
        return Ok(());
    };
    if read_bundle_runtime_config(&output)?.is_none() {
        return Ok(());
    }
    if !bundle_auto_short_term_capture_enabled(&output)? {
        return Ok(());
    }

    let Some(content) = render_compaction_checkpoint_content(packet) else {
        return Ok(());
    };

    let response = checkpoint_with_bundle_defaults(
        &CheckpointArgs {
            output: output.clone(),
            project: packet.session.project.clone(),
            namespace: None,
            workspace: None,
            visibility: None,
            source_path: Some("compaction".to_string()),
            confidence: Some(0.85),
            ttl_seconds: Some(86_400),
            tag: vec!["compaction".to_string(), "auto-checkpoint".to_string()],
            content: Some(content),
            input: None,
            stdin: false,
        },
        base_url,
    )
    .await?;

    let snapshot = read_bundle_resume(
        &ResumeArgs {
            output,
            project: packet.session.project.clone(),
            namespace: None,
            agent: packet.session.agent.clone(),
            workspace: None,
            visibility: None,
            route: None,
            intent: Some("current_task".to_string()),
            limit: Some(8),
            rehydration_limit: Some(4),
            semantic: false,
            prompt: false,
            summary: false,
        },
        base_url,
    )
    .await?;
    write_bundle_memory_files(&snapshot_bundle_root(&response, &snapshot), &snapshot, None).await?;
    Ok(())
}

fn snapshot_bundle_root(
    _response: &memd_schema::StoreMemoryResponse,
    _snapshot: &ResumeSnapshot,
) -> PathBuf {
    resolve_default_bundle_root()
        .ok()
        .flatten()
        .unwrap_or_else(|| PathBuf::from(".memd"))
}

fn render_compaction_checkpoint_content(packet: &CompactionPacket) -> Option<String> {
    let mut lines = Vec::new();

    if !packet.session.task.trim().is_empty() {
        lines.push(format!("task: {}", packet.session.task.trim()));
    }
    if !packet.goal.trim().is_empty() {
        lines.push(format!("goal: {}", packet.goal.trim()));
    }
    if !packet.active_work.is_empty() {
        lines.push(format!(
            "active: {}",
            packet
                .active_work
                .iter()
                .take(3)
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    if !packet.next_actions.is_empty() {
        lines.push(format!(
            "next: {}",
            packet
                .next_actions
                .iter()
                .take(3)
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    if !packet.do_not_drop.is_empty() {
        lines.push(format!(
            "keep: {}",
            packet
                .do_not_drop
                .iter()
                .take(2)
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }

    let content = lines.join("\n");
    if content.trim().is_empty() {
        None
    } else {
        Some(content)
    }
}

fn checkpoint_as_remember_args(args: &CheckpointArgs) -> RememberArgs {
    let mut tags = vec!["checkpoint".to_string(), "current-task".to_string()];
    for tag in &args.tag {
        if !tags.iter().any(|existing| existing == tag) {
            tags.push(tag.clone());
        }
    }

    RememberArgs {
        output: args.output.clone(),
        project: args.project.clone(),
        namespace: args.namespace.clone(),
        workspace: args.workspace.clone(),
        visibility: args.visibility.clone(),
        kind: Some("status".to_string()),
        scope: Some("project".to_string()),
        source_agent: None,
        source_system: Some("memd-short-term".to_string()),
        source_path: args.source_path.clone(),
        source_quality: Some("derived".to_string()),
        confidence: args.confidence.or(Some(0.8)),
        ttl_seconds: args.ttl_seconds.or(Some(86_400)),
        tag: tags,
        content: args.content.clone(),
        input: args.input.clone(),
        stdin: args.stdin,
    }
}

fn render_attach_snippet(shell: &str, bundle_path: &Path) -> anyhow::Result<String> {
    let shell = shell.trim().to_ascii_lowercase();
    match shell.as_str() {
        "bash" | "zsh" | "sh" => Ok(format!(
            r#"export MEMD_BUNDLE_ROOT="{bundle_path}"
source "$MEMD_BUNDLE_ROOT/env"
memd resume --output "$MEMD_BUNDLE_ROOT" --intent current_task
"#,
            bundle_path = bundle_path.display(),
        )),
        "powershell" | "pwsh" => Ok(format!(
            r#"$env:MEMD_BUNDLE_ROOT = "{bundle_path}"
. (Join-Path $env:MEMD_BUNDLE_ROOT "env.ps1")
memd resume --output $env:MEMD_BUNDLE_ROOT --intent current_task
"#,
            bundle_path = escape_ps1(&bundle_path.display().to_string()),
        )),
        other => anyhow::bail!(
            "unsupported shell '{other}'; expected bash, zsh, sh, powershell, or pwsh"
        ),
    }
}

fn set_bundle_agent(output: &Path, agent: &str) -> anyhow::Result<()> {
    let config_path = output.join("config.json");
    if !config_path.exists() {
        anyhow::bail!(
            "{} does not exist; initialize the bundle first",
            config_path.display()
        );
    }

    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let mut config: BundleConfigFile =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
    config.agent = Some(agent.to_string());
    fs::write(&config_path, serde_json::to_string_pretty(&config)? + "\n")
        .with_context(|| format!("write {}", config_path.display()))?;

    let session = config.session.clone();
    let effective_agent = compose_agent_identity(agent, session.as_deref());
    rewrite_env_assignment(
        &output.join("env"),
        "MEMD_AGENT=",
        &format!("MEMD_AGENT={effective_agent}\n"),
    )?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_AGENT = ",
        &format!("$env:MEMD_AGENT = \"{}\"\n", escape_ps1(&effective_agent)),
    )?;

    Ok(())
}

fn set_bundle_session(output: &Path, session: &str) -> anyhow::Result<()> {
    let config_path = output.join("config.json");
    if !config_path.exists() {
        anyhow::bail!(
            "{} does not exist; initialize the bundle first",
            config_path.display()
        );
    }

    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let mut config: BundleConfigFile =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
    config.session = Some(session.to_string());
    fs::write(&config_path, serde_json::to_string_pretty(&config)? + "\n")
        .with_context(|| format!("write {}", config_path.display()))?;

    let agent = config.agent.as_deref().unwrap_or("unknown");
    let effective_agent = compose_agent_identity(agent, Some(session));
    rewrite_env_assignment(
        &output.join("env"),
        "MEMD_SESSION=",
        &format!("MEMD_SESSION={session}\n"),
    )?;
    rewrite_env_assignment(
        &output.join("env"),
        "MEMD_AGENT=",
        &format!("MEMD_AGENT={effective_agent}\n"),
    )?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_SESSION = ",
        &format!("$env:MEMD_SESSION = \"{}\"\n", escape_ps1(session)),
    )?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_AGENT = ",
        &format!("$env:MEMD_AGENT = \"{}\"\n", escape_ps1(&effective_agent)),
    )?;

    Ok(())
}

fn set_bundle_base_url(output: &Path, base_url: &str) -> anyhow::Result<()> {
    let config_path = output.join("config.json");
    if !config_path.exists() {
        anyhow::bail!(
            "{} does not exist; initialize the bundle first",
            config_path.display()
        );
    }

    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let mut config: BundleConfigFile =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
    config.base_url = Some(base_url.to_string());
    fs::write(&config_path, serde_json::to_string_pretty(&config)? + "\n")
        .with_context(|| format!("write {}", config_path.display()))?;

    rewrite_env_assignment(
        &output.join("env"),
        "MEMD_BASE_URL=",
        &format!("MEMD_BASE_URL={base_url}\n"),
    )?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_BASE_URL = ",
        &format!("$env:MEMD_BASE_URL = \"{}\"\n", escape_ps1(base_url)),
    )?;

    Ok(())
}

fn set_bundle_auto_short_term_capture(output: &Path, enabled: bool) -> anyhow::Result<()> {
    let config_path = output.join("config.json");
    if !config_path.exists() {
        anyhow::bail!(
            "{} does not exist; initialize the bundle first",
            config_path.display()
        );
    }

    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let mut config: BundleConfigFile =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
    config.auto_short_term_capture = enabled;
    fs::write(&config_path, serde_json::to_string_pretty(&config)? + "\n")
        .with_context(|| format!("write {}", config_path.display()))?;

    rewrite_env_assignment(
        &output.join("env"),
        "MEMD_AUTO_SHORT_TERM_CAPTURE=",
        &format!(
            "MEMD_AUTO_SHORT_TERM_CAPTURE={}\n",
            if enabled { "true" } else { "false" }
        ),
    )?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_AUTO_SHORT_TERM_CAPTURE = ",
        &format!(
            "$env:MEMD_AUTO_SHORT_TERM_CAPTURE = \"{}\"\n",
            if enabled { "true" } else { "false" }
        ),
    )?;

    Ok(())
}

fn rewrite_env_assignment(path: &Path, prefix: &str, replacement: &str) -> anyhow::Result<()> {
    let mut lines = if path.exists() {
        fs::read_to_string(path)
            .with_context(|| format!("read {}", path.display()))?
            .lines()
            .map(|line| format!("{line}\n"))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let mut replaced = false;
    for line in &mut lines {
        if line.starts_with(prefix) {
            *line = replacement.to_string();
            replaced = true;
        }
    }
    if !replaced {
        lines.push(replacement.to_string());
    }

    let mut output = String::new();
    for line in lines {
        output.push_str(&line);
    }
    fs::write(path, output).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
struct BundleAgentProfile {
    name: String,
    env_agent: String,
    session: Option<String>,
    effective_agent: String,
    memory_file: String,
    shell_entrypoint: String,
    powershell_entrypoint: String,
    launch_hint: String,
    native_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct BundleAgentProfilesResponse {
    bundle_root: String,
    shell: String,
    current: Option<String>,
    current_session: Option<String>,
    selected: Option<String>,
    agents: Vec<BundleAgentProfile>,
}

fn build_bundle_agent_profiles(
    output: &Path,
    name: Option<&str>,
    shell: Option<&str>,
) -> anyhow::Result<BundleAgentProfilesResponse> {
    let runtime = read_bundle_runtime_config(output)?;
    let current = runtime.as_ref().and_then(|config| config.agent.clone());
    let current_session = runtime.as_ref().and_then(|config| config.session.clone());
    let shell = shell
        .map(|value| value.trim().to_ascii_lowercase())
        .or_else(detect_shell)
        .unwrap_or_else(|| "bash".to_string());
    let mut agents = vec![
        ("codex", "codex", "CODEX_MEMORY.md"),
        ("claude-code", "claude-code", "CLAUDE_CODE_MEMORY.md"),
        ("openclaw", "openclaw", "OPENCLAW_MEMORY.md"),
        ("opencode", "opencode", "OPENCODE_MEMORY.md"),
    ]
    .into_iter()
    .map(|(name, env_agent, memory_file)| BundleAgentProfile {
        name: name.to_string(),
        env_agent: env_agent.to_string(),
        session: current_session.clone(),
        effective_agent: compose_agent_identity(env_agent, current_session.as_deref()),
        memory_file: output
            .join("agents")
            .join(memory_file)
            .display()
            .to_string(),
        shell_entrypoint: output
            .join("agents")
            .join(format!("{name}.sh"))
            .display()
            .to_string(),
        powershell_entrypoint: output
            .join("agents")
            .join(format!("{name}.ps1"))
            .display()
            .to_string(),
        launch_hint: String::new(),
        native_hint: None,
    })
    .collect::<Vec<_>>();

    for agent in &mut agents {
        agent.launch_hint = match shell.as_str() {
            "powershell" | "pwsh" => format!(". \"{}\"", agent.powershell_entrypoint),
            _ => format!("\"{}\"", agent.shell_entrypoint),
        };
        if agent.name == "claude-code" {
            agent.native_hint = Some(format!(
                "import @.memd/agents/CLAUDE_IMPORTS.md into CLAUDE.md, then verify with /memory"
            ));
        }
    }

    let selected = name.map(|value| value.trim().to_ascii_lowercase());
    if let Some(selected_name) = selected.as_deref() {
        agents.retain(|agent| agent.name == selected_name);
        if agents.is_empty() {
            anyhow::bail!("unknown agent profile '{selected_name}'");
        }
    }

    Ok(BundleAgentProfilesResponse {
        bundle_root: output.display().to_string(),
        shell,
        current,
        current_session,
        selected,
        agents,
    })
}

fn render_bundle_agent_profiles_summary(response: &BundleAgentProfilesResponse) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "bundle={} shell={} current={} session={}\n",
        response.bundle_root,
        response.shell,
        response.current.as_deref().unwrap_or("none"),
        response.current_session.as_deref().unwrap_or("none")
    ));
    for agent in &response.agents {
        output.push_str(&format!(
            "- {}{} | effective {} | memory {} | launch {}\n",
            agent.name,
            if response.current.as_deref() == Some(agent.name.as_str()) {
                " [active]"
            } else {
                ""
            },
            agent.effective_agent,
            agent.memory_file,
            agent.launch_hint
        ));
        if let Some(native_hint) = agent.native_hint.as_deref() {
            output.push_str(&format!("  native {}\n", native_hint));
        }
    }
    output.trim_end().to_string()
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
    namespace: Option<String>,
    agent: String,
    session: String,
    base_url: String,
    route: String,
    intent: String,
    workspace: Option<String>,
    visibility: Option<String>,
    auto_short_term_capture: bool,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct BundleConfigFile {
    #[serde(default)]
    project: Option<String>,
    #[serde(default)]
    namespace: Option<String>,
    #[serde(default)]
    agent: Option<String>,
    #[serde(default)]
    session: Option<String>,
    #[serde(default)]
    base_url: Option<String>,
    #[serde(default)]
    route: Option<String>,
    #[serde(default)]
    intent: Option<String>,
    #[serde(default)]
    workspace: Option<String>,
    #[serde(default)]
    visibility: Option<String>,
    #[serde(default = "default_auto_short_term_capture")]
    auto_short_term_capture: bool,
    #[serde(default)]
    rag_url: Option<String>,
    #[serde(default)]
    backend: Option<BundleBackendConfigFile>,
}

#[derive(Debug, Clone)]
struct BundleRuntimeConfig {
    project: Option<String>,
    namespace: Option<String>,
    agent: Option<String>,
    session: Option<String>,
    base_url: Option<String>,
    route: Option<String>,
    intent: Option<String>,
    workspace: Option<String>,
    visibility: Option<String>,
    auto_short_term_capture: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BundleResumeState {
    focus: Option<String>,
    pressure: Option<String>,
    next_recovery: Option<String>,
    lane: Option<String>,
    working_records: usize,
    inbox_items: usize,
    rehydration_items: usize,
    #[serde(default = "Utc::now")]
    recorded_at: DateTime<Utc>,
}

impl BundleResumeState {
    fn from_snapshot(snapshot: &ResumeSnapshot) -> Self {
        Self {
            focus: snapshot
                .working
                .records
                .first()
                .map(|record| record.record.clone()),
            pressure: snapshot
                .inbox
                .items
                .first()
                .map(|item| item.item.content.clone()),
            next_recovery: snapshot
                .working
                .rehydration_queue
                .first()
                .map(|item| format!("{}: {}", item.label, item.summary)),
            lane: snapshot.workspaces.workspaces.first().map(|lane| {
                format!(
                    "{} / {} / {}",
                    lane.project.as_deref().unwrap_or("none"),
                    lane.namespace.as_deref().unwrap_or("none"),
                    lane.workspace.as_deref().unwrap_or("none")
                )
            }),
            working_records: snapshot.working.records.len(),
            inbox_items: snapshot.inbox.items.len(),
            rehydration_items: snapshot.working.rehydration_queue.len(),
            recorded_at: Utc::now(),
        }
    }

    fn age_minutes(&self) -> i64 {
        (Utc::now() - self.recorded_at).num_minutes()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResumeSnapshot {
    project: Option<String>,
    namespace: Option<String>,
    agent: Option<String>,
    workspace: Option<String>,
    visibility: Option<String>,
    route: String,
    intent: String,
    context: memd_schema::CompactContextResponse,
    working: memd_schema::WorkingMemoryResponse,
    inbox: memd_schema::MemoryInboxResponse,
    workspaces: memd_schema::WorkspaceMemoryResponse,
    semantic: Option<RagRetrieveResponse>,
    change_summary: Vec<String>,
    resume_state_age_minutes: Option<i64>,
    refresh_recommended: bool,
}

impl ResumeSnapshot {
    fn normalized_memory_text(value: &str) -> String {
        value
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase()
    }

    fn redundant_context_items(&self) -> usize {
        let mut seen = std::collections::HashSet::<String>::new();
        let mut duplicates = 0usize;

        for value in self
            .context
            .records
            .iter()
            .map(|record| record.record.as_str())
        {
            let normalized = Self::normalized_memory_text(value);
            if !normalized.is_empty() && !seen.insert(normalized) {
                duplicates += 1;
            }
        }
        for value in self
            .working
            .records
            .iter()
            .map(|record| record.record.as_str())
        {
            let normalized = Self::normalized_memory_text(value);
            if !normalized.is_empty() && !seen.insert(normalized) {
                duplicates += 1;
            }
        }
        for value in self
            .working
            .rehydration_queue
            .iter()
            .map(|item| item.summary.as_str())
        {
            let normalized = Self::normalized_memory_text(value);
            if !normalized.is_empty() && !seen.insert(normalized) {
                duplicates += 1;
            }
        }
        for value in self
            .inbox
            .items
            .iter()
            .map(|item| item.item.content.as_str())
        {
            let normalized = Self::normalized_memory_text(value);
            if !normalized.is_empty() && !seen.insert(normalized) {
                duplicates += 1;
            }
        }
        for value in self
            .semantic
            .iter()
            .flat_map(|semantic| semantic.items.iter())
            .map(|item| item.content.as_str())
        {
            let normalized = Self::normalized_memory_text(value);
            if !normalized.is_empty() && !seen.insert(normalized) {
                duplicates += 1;
            }
        }

        duplicates
    }

    fn estimated_prompt_chars(&self) -> usize {
        let header_chars = self.project.as_deref().map_or(0, str::len)
            + self.namespace.as_deref().map_or(0, str::len)
            + self.agent.as_deref().map_or(0, str::len)
            + self.workspace.as_deref().map_or(0, str::len)
            + self.visibility.as_deref().map_or(0, str::len)
            + self.route.len()
            + self.intent.len();
        let context_chars: usize = self
            .context
            .records
            .iter()
            .map(|record| record.record.len())
            .sum();
        let working_chars: usize = self
            .working
            .records
            .iter()
            .map(|record| record.record.len())
            .sum();
        let rehydration_chars: usize = self
            .working
            .rehydration_queue
            .iter()
            .map(|item| item.label.len() + item.summary.len())
            .sum();
        let inbox_chars: usize = self
            .inbox
            .items
            .iter()
            .map(|item| {
                let reasons_len: usize = item.reasons.iter().map(|reason| reason.len()).sum();
                item.item.content.len() + reasons_len
            })
            .sum();
        let workspace_chars: usize = self
            .workspaces
            .workspaces
            .iter()
            .map(|lane| {
                lane.project.as_deref().map_or(0, str::len)
                    + lane.namespace.as_deref().map_or(0, str::len)
                    + lane.workspace.as_deref().map_or(0, str::len)
                    + lane.tags.iter().map(|tag| tag.len()).sum::<usize>()
            })
            .sum();
        let semantic_chars: usize = self.semantic.as_ref().map_or(0, |semantic| {
            semantic
                .items
                .iter()
                .map(|item| item.content.len() + item.source.as_deref().map_or(0, str::len))
                .sum()
        });
        let change_chars: usize = self.change_summary.iter().map(|change| change.len()).sum();
        header_chars
            + context_chars
            + working_chars
            + rehydration_chars
            + inbox_chars
            + workspace_chars
            + semantic_chars
            + change_chars
    }

    fn estimated_prompt_tokens(&self) -> usize {
        self.estimated_prompt_chars().div_ceil(4)
    }

    fn context_pressure(&self) -> &'static str {
        let tokens = self.estimated_prompt_tokens();
        if self.working.truncated
            || tokens >= 1_800
            || self.inbox.items.len() >= 5
            || self.redundant_context_items() >= 3
            || self
                .semantic
                .as_ref()
                .is_some_and(|semantic| semantic.items.len() >= 4)
        {
            "high"
        } else if tokens >= 1_000
            || self.working.remaining_chars <= 200
            || self.inbox.items.len() >= 3
            || self.working.rehydration_queue.len() >= 4
            || self.redundant_context_items() >= 1
        {
            "medium"
        } else {
            "low"
        }
    }

    fn optimization_hints(&self) -> Vec<String> {
        let mut hints = Vec::new();
        if self.refresh_recommended {
            hints.push(
                "prefer a fresh session resumed from the bundle instead of carrying a stale long transcript"
                    .to_string(),
            );
        }
        if self.inbox.items.len() >= 3 {
            hints.push("triage inbox pressure before pulling in more context".to_string());
        }
        let redundant = self.redundant_context_items();
        if redundant > 0 {
            hints.push(format!(
                "collapse {} repeated context item(s) before continuing the session",
                redundant
            ));
        }
        if self
            .semantic
            .as_ref()
            .is_some_and(|semantic| !semantic.items.is_empty())
        {
            hints.push(
                "keep semantic recall off unless deep context is actually required".to_string(),
            );
        }
        if self.estimated_prompt_tokens() >= 1_200 || self.context.records.len() >= 6 {
            hints.push(
                "promote stable facts into compiled or typed artifacts before rereading raw files"
                    .to_string(),
            );
        }
        if self.working.rehydration_queue.len() >= 3 {
            hints.push(
                "resolve the top rehydration items instead of loading every deferred artifact"
                    .to_string(),
            );
        }
        hints
    }
}

#[derive(Debug, Clone, Serialize)]
struct HandoffSnapshot {
    generated_at: DateTime<Utc>,
    resume: ResumeSnapshot,
    sources: memd_schema::SourceMemoryResponse,
    target_session: Option<String>,
    target_bundle: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BundleEvalResponse {
    bundle_root: String,
    project: Option<String>,
    namespace: Option<String>,
    agent: Option<String>,
    workspace: Option<String>,
    visibility: Option<String>,
    status: String,
    score: u8,
    working_records: usize,
    context_records: usize,
    rehydration_items: usize,
    inbox_items: usize,
    workspace_lanes: usize,
    semantic_hits: usize,
    findings: Vec<String>,
    baseline_score: Option<u8>,
    score_delta: Option<i32>,
    changes: Vec<String>,
    recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GapCandidate {
    id: String,
    area: String,
    priority: u8,
    severity: String,
    signal: String,
    evidence: Vec<String>,
    recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ImprovementGapSnapshot {
    candidate_count: usize,
    high_priority_count: usize,
    eval_status: Option<String>,
    eval_score: Option<u8>,
    eval_score_delta: Option<i32>,
    top_priorities: Vec<String>,
    generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ImprovementAction {
    action: String,
    priority: String,
    target_session: Option<String>,
    scope: Option<String>,
    task_id: Option<String>,
    message_id: Option<String>,
    reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ImprovementActionResult {
    action: String,
    status: String,
    detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ImprovementIteration {
    iteration: usize,
    pre_gap: ImprovementGapSnapshot,
    planned_actions: Vec<ImprovementAction>,
    executed_actions: Vec<ImprovementActionResult>,
    post_gap: Option<ImprovementGapSnapshot>,
    generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ImprovementReport {
    bundle_root: String,
    project: Option<String>,
    namespace: Option<String>,
    agent: Option<String>,
    session: Option<String>,
    workspace: Option<String>,
    visibility: Option<String>,
    max_iterations: usize,
    apply: bool,
    started_at: DateTime<Utc>,
    completed_at: DateTime<Utc>,
    converged: bool,
    initial_gap: Option<ImprovementGapSnapshot>,
    final_gap: Option<ImprovementGapSnapshot>,
    final_changes: Vec<String>,
    iterations: Vec<ImprovementIteration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GapReport {
    bundle_root: String,
    project: Option<String>,
    namespace: Option<String>,
    agent: Option<String>,
    session: Option<String>,
    workspace: Option<String>,
    visibility: Option<String>,
    limit: usize,
    commits_checked: usize,
    eval_status: Option<String>,
    eval_score: Option<u8>,
    eval_score_delta: Option<i32>,
    candidate_count: usize,
    high_priority_count: usize,
    top_priorities: Vec<String>,
    candidates: Vec<GapCandidate>,
    recommendations: Vec<String>,
    changes: Vec<String>,
    evidence: Vec<String>,
    generated_at: DateTime<Utc>,
    previous_candidate_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectAwarenessEntry {
    project_dir: String,
    bundle_root: String,
    project: Option<String>,
    namespace: Option<String>,
    agent: Option<String>,
    session: Option<String>,
    effective_agent: Option<String>,
    base_url: Option<String>,
    presence: String,
    host: Option<String>,
    pid: Option<u32>,
    active_claims: usize,
    workspace: Option<String>,
    visibility: Option<String>,
    focus: Option<String>,
    pressure: Option<String>,
    next_recovery: Option<String>,
    last_updated: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectAwarenessResponse {
    root: String,
    current_bundle: String,
    collisions: Vec<String>,
    entries: Vec<ProjectAwarenessEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BundleHeartbeatState {
    session: Option<String>,
    agent: Option<String>,
    effective_agent: Option<String>,
    project: Option<String>,
    namespace: Option<String>,
    workspace: Option<String>,
    visibility: Option<String>,
    base_url: Option<String>,
    base_url_healthy: Option<bool>,
    host: Option<String>,
    pid: Option<u32>,
    focus: Option<String>,
    pressure: Option<String>,
    next_recovery: Option<String>,
    status: String,
    last_seen: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionClaim {
    scope: String,
    session: Option<String>,
    agent: Option<String>,
    effective_agent: Option<String>,
    project: Option<String>,
    workspace: Option<String>,
    host: Option<String>,
    pid: Option<u32>,
    acquired_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SessionClaimsState {
    claims: Vec<SessionClaim>,
}

fn session_claim_from_record(record: memd_schema::PeerClaimRecord) -> SessionClaim {
    SessionClaim {
        scope: record.scope,
        session: Some(record.session),
        agent: record.agent,
        effective_agent: record.effective_agent,
        project: record.project,
        workspace: record.workspace,
        host: record.host,
        pid: record.pid,
        acquired_at: record.acquired_at,
        expires_at: record.expires_at,
    }
}

#[derive(Debug, Clone, Serialize)]
struct ClaimsResponse {
    bundle_root: String,
    current_session: Option<String>,
    claims: Vec<SessionClaim>,
}

#[derive(Debug, Clone, Serialize)]
struct MessagesResponse {
    bundle_root: String,
    current_session: Option<String>,
    messages: Vec<PeerMessageRecord>,
}

#[derive(Debug, Clone, Serialize)]
struct TasksResponse {
    bundle_root: String,
    current_session: Option<String>,
    tasks: Vec<PeerTaskRecord>,
}

#[derive(Debug, Clone, Serialize)]
struct CoordinationResponse {
    bundle_root: String,
    current_session: String,
    inbox: PeerCoordinationInboxResponse,
    recovery: CoordinationRecoverySummary,
    policy_conflicts: Vec<String>,
    suggestions: Vec<CoordinationSuggestion>,
    boundary_recommendations: Vec<String>,
    receipts: Vec<PeerCoordinationReceiptRecord>,
}

#[derive(Debug, Clone, Serialize)]
struct CoordinationRecoverySummary {
    stale_peers: Vec<ProjectAwarenessEntry>,
    reclaimable_claims: Vec<SessionClaim>,
    stalled_tasks: Vec<PeerTaskRecord>,
}

#[derive(Debug, Clone, Serialize)]
struct CoordinationChangeResponse {
    bundle_root: String,
    current_session: String,
    view: String,
    changed: bool,
    alerts: Vec<String>,
    snapshot: CoordinationAlertSnapshot,
    generated_at: DateTime<Utc>,
    previous_generated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CoordinationSnapshotState {
    generated_at: DateTime<Utc>,
    view: String,
    snapshot: CoordinationAlertSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct CoordinationAlertSnapshot {
    message_count: usize,
    owned_count: usize,
    help_count: usize,
    review_count: usize,
    stale_peer_count: usize,
    reclaimable_claim_count: usize,
    stalled_task_count: usize,
    policy_conflict_count: usize,
    recommendation_count: usize,
    suggestion_count: usize,
    latest_receipt_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct CoordinationSuggestion {
    action: String,
    priority: String,
    target_session: Option<String>,
    task_id: Option<String>,
    scope: Option<String>,
    message_id: Option<String>,
    reason: String,
    stale_session: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct BundleBackendConfigFile {
    #[serde(default)]
    rag: Option<BundleRagConfigFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

fn memory_visibility_label(value: memd_schema::MemoryVisibility) -> &'static str {
    match value {
        memd_schema::MemoryVisibility::Private => "private",
        memd_schema::MemoryVisibility::Workspace => "workspace",
        memd_schema::MemoryVisibility::Public => "public",
    }
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
    use std::sync::{Arc, Mutex};

    use axum::{
        Json, Router,
        extract::{Query, State},
        routing::{get, post},
    };
    use memd_schema::{
        PeerClaimAcquireRequest, PeerClaimRecord, PeerClaimReleaseRequest,
        PeerClaimTransferRequest, PeerClaimsRequest, PeerClaimsResponse,
        PeerCoordinationInboxResponse, PeerCoordinationReceiptRecord,
        PeerCoordinationReceiptRequest, PeerCoordinationReceiptsResponse, PeerMessageAckRequest,
        PeerMessageInboxRequest, PeerMessageRecord, PeerMessageSendRequest, PeerMessagesResponse,
        PeerTaskRecord,
    };

    #[derive(Clone, Default)]
    struct MockPeerState {
        messages: Arc<Mutex<Vec<PeerMessageRecord>>>,
        claims: Arc<Mutex<Vec<PeerClaimRecord>>>,
        receipts: Arc<Mutex<Vec<PeerCoordinationReceiptRecord>>>,
    }

    async fn mock_send_peer_message(
        State(state): State<MockPeerState>,
        Json(req): Json<PeerMessageSendRequest>,
    ) -> Json<PeerMessagesResponse> {
        let message = PeerMessageRecord {
            id: uuid::Uuid::new_v4().to_string(),
            kind: req.kind,
            from_session: req.from_session,
            from_agent: req.from_agent,
            to_session: req.to_session,
            project: req.project,
            namespace: req.namespace,
            workspace: req.workspace,
            content: req.content,
            created_at: Utc::now(),
            acknowledged_at: None,
        };
        state
            .messages
            .lock()
            .expect("lock messages")
            .push(message.clone());
        Json(PeerMessagesResponse {
            messages: vec![message],
        })
    }

    async fn mock_peer_inbox(
        State(state): State<MockPeerState>,
        Query(req): Query<PeerMessageInboxRequest>,
    ) -> Json<PeerMessagesResponse> {
        let messages = state
            .messages
            .lock()
            .expect("lock messages")
            .iter()
            .filter(|message| {
                message.to_session == req.session
                    && req
                        .project
                        .as_ref()
                        .is_none_or(|project| message.project.as_ref() == Some(project))
                    && req
                        .namespace
                        .as_ref()
                        .is_none_or(|namespace| message.namespace.as_ref() == Some(namespace))
                    && req
                        .workspace
                        .as_ref()
                        .is_none_or(|workspace| message.workspace.as_ref() == Some(workspace))
                    && (req.include_acknowledged.unwrap_or(false)
                        || message.acknowledged_at.is_none())
            })
            .cloned()
            .collect();
        Json(PeerMessagesResponse { messages })
    }

    async fn mock_peer_ack(
        State(state): State<MockPeerState>,
        Json(req): Json<PeerMessageAckRequest>,
    ) -> Json<PeerMessagesResponse> {
        let mut messages = state.messages.lock().expect("lock messages");
        let mut acked = Vec::new();
        for message in messages.iter_mut() {
            if message.id == req.id && message.to_session == req.session {
                message.acknowledged_at = Some(Utc::now());
                acked.push(message.clone());
            }
        }
        Json(PeerMessagesResponse { messages: acked })
    }

    async fn mock_claim_acquire(
        State(state): State<MockPeerState>,
        Json(req): Json<PeerClaimAcquireRequest>,
    ) -> Json<PeerClaimsResponse> {
        let mut claims = state.claims.lock().expect("lock claims");
        claims.retain(|claim| claim.expires_at > Utc::now());
        if let Some(existing) = claims
            .iter()
            .find(|claim| claim.scope == req.scope && claim.session != req.session)
            .cloned()
        {
            return Json(PeerClaimsResponse {
                claims: vec![existing],
            });
        }
        claims.retain(|claim| !(claim.scope == req.scope && claim.session == req.session));
        let claim = PeerClaimRecord {
            scope: req.scope,
            session: req.session,
            agent: req.agent,
            effective_agent: req.effective_agent,
            project: req.project,
            namespace: req.namespace,
            workspace: req.workspace,
            host: req.host,
            pid: req.pid,
            acquired_at: Utc::now(),
            expires_at: Utc::now() + chrono::TimeDelta::seconds(req.ttl_seconds as i64),
        };
        claims.push(claim.clone());
        Json(PeerClaimsResponse {
            claims: vec![claim],
        })
    }

    async fn mock_claim_release(
        State(state): State<MockPeerState>,
        Json(req): Json<PeerClaimReleaseRequest>,
    ) -> Json<PeerClaimsResponse> {
        let mut claims = state.claims.lock().expect("lock claims");
        let mut released = Vec::new();
        claims.retain(|claim| {
            let matches = claim.scope == req.scope && claim.session == req.session;
            if matches {
                released.push(claim.clone());
            }
            !matches
        });
        Json(PeerClaimsResponse { claims: released })
    }

    async fn mock_claim_transfer(
        State(state): State<MockPeerState>,
        Json(req): Json<PeerClaimTransferRequest>,
    ) -> Json<PeerClaimsResponse> {
        let mut claims = state.claims.lock().expect("lock claims");
        let mut transferred = Vec::new();
        for claim in claims.iter_mut() {
            if claim.scope == req.scope && claim.session == req.from_session {
                claim.session = req.to_session.clone();
                claim.agent = req.to_agent.clone();
                claim.effective_agent = req.to_effective_agent.clone();
                transferred.push(claim.clone());
            }
        }
        Json(PeerClaimsResponse {
            claims: transferred,
        })
    }

    async fn mock_claims(
        State(state): State<MockPeerState>,
        Query(req): Query<PeerClaimsRequest>,
    ) -> Json<PeerClaimsResponse> {
        let claims = state
            .claims
            .lock()
            .expect("lock claims")
            .iter()
            .filter(|claim| {
                req.session
                    .as_ref()
                    .is_none_or(|session| &claim.session == session)
                    && req
                        .project
                        .as_ref()
                        .is_none_or(|project| claim.project.as_ref() == Some(project))
                    && req
                        .namespace
                        .as_ref()
                        .is_none_or(|namespace| claim.namespace.as_ref() == Some(namespace))
                    && req
                        .workspace
                        .as_ref()
                        .is_none_or(|workspace| claim.workspace.as_ref() == Some(workspace))
                    && (!req.active_only.unwrap_or(true) || claim.expires_at > Utc::now())
            })
            .cloned()
            .collect();
        Json(PeerClaimsResponse { claims })
    }

    async fn mock_record_receipt(
        State(state): State<MockPeerState>,
        Json(req): Json<PeerCoordinationReceiptRequest>,
    ) -> Json<PeerCoordinationReceiptsResponse> {
        let receipt = PeerCoordinationReceiptRecord {
            id: uuid::Uuid::new_v4().to_string(),
            kind: req.kind,
            actor_session: req.actor_session,
            actor_agent: req.actor_agent,
            target_session: req.target_session,
            task_id: req.task_id,
            scope: req.scope,
            project: req.project,
            namespace: req.namespace,
            workspace: req.workspace,
            summary: req.summary,
            created_at: Utc::now(),
        };
        state
            .receipts
            .lock()
            .expect("lock receipts")
            .push(receipt.clone());
        Json(PeerCoordinationReceiptsResponse {
            receipts: vec![receipt],
        })
    }

    async fn spawn_mock_peer_server() -> String {
        let state = MockPeerState::default();
        let app = Router::new()
            .route("/coordination/messages/send", post(mock_send_peer_message))
            .route("/coordination/messages/inbox", get(mock_peer_inbox))
            .route("/coordination/messages/ack", post(mock_peer_ack))
            .route("/coordination/receipts/record", post(mock_record_receipt))
            .route("/coordination/claims/acquire", post(mock_claim_acquire))
            .route("/coordination/claims/release", post(mock_claim_release))
            .route("/coordination/claims/transfer", post(mock_claim_transfer))
            .route("/coordination/claims", get(mock_claims))
            .with_state(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind mock peer server");
        let addr = listener.local_addr().expect("mock peer server addr");
        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("serve mock peer server");
        });
        tokio::time::sleep(Duration::from_millis(25)).await;
        format!("http://{}", addr)
    }

    #[test]
    fn derives_help_request_message_from_scope() {
        let message = derive_outbound_message(&MessagesArgs {
            output: PathBuf::from(".memd"),
            send: true,
            inbox: false,
            ack: None,
            target_session: Some("claude-b".to_string()),
            kind: None,
            request_help: true,
            request_review: false,
            assign_scope: None,
            scope: Some("file:src/main.rs".to_string()),
            content: None,
            summary: false,
        })
        .expect("derive help request");

        assert_eq!(message.0, "help_request");
        assert!(message.1.contains("file:src/main.rs"));
    }

    #[test]
    fn derives_review_request_message_from_scope() {
        let message = derive_outbound_message(&MessagesArgs {
            output: PathBuf::from(".memd"),
            send: true,
            inbox: false,
            ack: None,
            target_session: Some("claude-b".to_string()),
            kind: None,
            request_help: false,
            request_review: true,
            assign_scope: None,
            scope: Some("task:parser-refactor".to_string()),
            content: None,
            summary: false,
        })
        .expect("derive review request");

        assert_eq!(message.0, "review_request");
        assert!(message.1.contains("task:parser-refactor"));
    }

    #[test]
    fn derives_assignment_message_from_assign_scope() {
        let message = derive_outbound_message(&MessagesArgs {
            output: PathBuf::from(".memd"),
            send: true,
            inbox: false,
            ack: None,
            target_session: Some("claude-b".to_string()),
            kind: None,
            request_help: false,
            request_review: false,
            assign_scope: Some("task:parser-refactor".to_string()),
            scope: None,
            content: None,
            summary: false,
        })
        .expect("derive assignment");

        assert_eq!(message.0, "assignment");
        assert!(message.1.contains("task:parser-refactor"));
    }

    #[test]
    fn resolves_nested_bundle_rag_config() {
        let config = BundleConfigFile {
            project: None,
            namespace: None,
            agent: None,
            session: None,
            base_url: None,
            route: None,
            intent: None,
            workspace: None,
            visibility: None,
            auto_short_term_capture: true,
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
            project: None,
            namespace: None,
            agent: None,
            session: None,
            base_url: None,
            route: None,
            intent: None,
            workspace: None,
            visibility: None,
            auto_short_term_capture: true,
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
            namespace: Some("main".to_string()),
            agent: "codex".to_string(),
            session: "session-demo".to_string(),
            base_url: "http://127.0.0.1:8787".to_string(),
            route: "auto".to_string(),
            intent: "general".to_string(),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            auto_short_term_capture: true,
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
        assert_eq!(json["namespace"], "main");
        assert_eq!(json["backend"]["rag"]["enabled"], true);
        assert_eq!(json["backend"]["rag"]["provider"], "lightrag-compatible");
        assert_eq!(json["backend"]["rag"]["url"], "http://127.0.0.1:9000");
        assert_eq!(json["workspace"], "team-alpha");
        assert_eq!(json["visibility"], "workspace");
        assert_eq!(json["rag_url"], "http://127.0.0.1:9000");
    }

    #[test]
    fn writes_bundle_memory_placeholder_with_hot_path_guidance() {
        let dir =
            std::env::temp_dir().join(format!("memd-bundle-placeholder-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");
        let config = BundleConfig {
            schema_version: 2,
            project: "demo".to_string(),
            namespace: Some("main".to_string()),
            agent: "codex".to_string(),
            session: "session-demo".to_string(),
            base_url: "http://127.0.0.1:8787".to_string(),
            route: "auto".to_string(),
            intent: "general".to_string(),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            auto_short_term_capture: true,
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

        write_bundle_memory_placeholder(&dir, &config).expect("write placeholder");
        write_native_agent_bridge_files(&dir).expect("write native bridge");

        let markdown = fs::read_to_string(dir.join("MEMD_MEMORY.md")).expect("read placeholder");
        assert!(markdown.contains("memd resume --output"));
        assert!(markdown.contains("--semantic"));
        assert!(markdown.contains("fast local hot path"));
        assert!(markdown.contains("slower deep recall"));
        let claude_imports = fs::read_to_string(dir.join("agents").join("CLAUDE_IMPORTS.md"))
            .expect("read claude imports");
        assert!(claude_imports.contains("@../MEMD_MEMORY.md"));
        assert!(claude_imports.contains("@CLAUDE_CODE_MEMORY.md"));
        assert!(claude_imports.contains("/memory"));

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn checkpoint_translation_sets_short_term_defaults() {
        let args = CheckpointArgs {
            output: PathBuf::from(".memd"),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            source_path: Some("notes/today.md".to_string()),
            confidence: None,
            ttl_seconds: None,
            tag: vec!["urgent".to_string()],
            content: Some("remember current blocker".to_string()),
            input: None,
            stdin: false,
        };

        let translated = checkpoint_as_remember_args(&args);
        assert_eq!(translated.kind.as_deref(), Some("status"));
        assert_eq!(translated.scope.as_deref(), Some("project"));
        assert_eq!(translated.source_system.as_deref(), Some("memd-short-term"));
        assert_eq!(translated.source_quality.as_deref(), Some("derived"));
        assert_eq!(translated.confidence, Some(0.8));
        assert_eq!(translated.ttl_seconds, Some(86_400));
        assert!(translated.tag.iter().any(|value| value == "checkpoint"));
        assert!(translated.tag.iter().any(|value| value == "current-task"));
        assert!(translated.tag.iter().any(|value| value == "urgent"));
    }

    #[test]
    fn bundle_memory_markdown_surfaces_current_task_snapshot() {
        let snapshot = ResumeSnapshot {
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            context: memd_schema::CompactContextResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::CurrentTask,
                retrieval_order: vec![memd_schema::MemoryScope::Project],
                records: Vec::new(),
            },
            working: memd_schema::WorkingMemoryResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::CurrentTask,
                retrieval_order: vec![memd_schema::MemoryScope::Project],
                budget_chars: 1600,
                used_chars: 60,
                remaining_chars: 1540,
                truncated: false,
                policy: memd_schema::WorkingMemoryPolicyState {
                    admission_limit: 8,
                    max_chars_per_item: 220,
                    budget_chars: 1600,
                    rehydration_limit: 4,
                },
                records: vec![memd_schema::CompactMemoryRecord {
                    id: uuid::Uuid::new_v4(),
                    record: "Finish the resume snapshot renderer".to_string(),
                }],
                evicted: Vec::new(),
                rehydration_queue: vec![memd_schema::MemoryRehydrationRecord {
                    id: None,
                    kind: "source".to_string(),
                    label: "artifact".to_string(),
                    summary: "Check the latest handoff note".to_string(),
                    reason: None,
                    source_agent: None,
                    source_system: None,
                    source_path: None,
                    source_quality: None,
                    recorded_at: None,
                }],
                traces: Vec::new(),
                semantic_consolidation: None,
            },
            inbox: memd_schema::MemoryInboxResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::CurrentTask,
                items: vec![memd_schema::InboxMemoryItem {
                    item: memd_schema::MemoryItem {
                        id: uuid::Uuid::new_v4(),
                        content: "Repair one stale workspace lane".to_string(),
                        redundancy_key: None,
                        belief_branch: None,
                        preferred: true,
                        kind: memd_schema::MemoryKind::Status,
                        scope: memd_schema::MemoryScope::Project,
                        project: Some("demo".to_string()),
                        namespace: Some("main".to_string()),
                        workspace: Some("team-alpha".to_string()),
                        visibility: memd_schema::MemoryVisibility::Workspace,
                        source_agent: None,
                        source_system: None,
                        source_path: None,
                        source_quality: None,
                        confidence: 0.8,
                        ttl_seconds: Some(86_400),
                        created_at: chrono::Utc::now(),
                        status: memd_schema::MemoryStatus::Active,
                        stage: memd_schema::MemoryStage::Candidate,
                        last_verified_at: None,
                        supersedes: Vec::new(),
                        updated_at: chrono::Utc::now(),
                        tags: vec!["checkpoint".to_string()],
                    },
                    reasons: vec!["stale".to_string()],
                }],
            },
            workspaces: memd_schema::WorkspaceMemoryResponse {
                workspaces: vec![memd_schema::WorkspaceMemoryRecord {
                    project: Some("demo".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("team-alpha".to_string()),
                    visibility: memd_schema::MemoryVisibility::Workspace,
                    item_count: 4,
                    active_count: 3,
                    candidate_count: 1,
                    contested_count: 0,
                    source_lane_count: 1,
                    avg_confidence: 0.84,
                    trust_score: 0.91,
                    last_seen_at: None,
                    tags: Vec::new(),
                }],
            },
            semantic: None,
            change_summary: vec!["focus -> Finish the resume snapshot renderer".to_string()],
            resume_state_age_minutes: None,
            refresh_recommended: false,
        };

        let markdown = render_bundle_memory_markdown(&snapshot, None);
        assert!(markdown.contains("## Current Task Snapshot"));
        assert!(markdown.contains("## Since Last Resume"));
        assert!(markdown.contains("Finish the resume snapshot renderer"));
        assert!(markdown.contains("Repair one stale workspace lane"));
        assert!(markdown.contains("Check the latest handoff note"));
        assert!(markdown.contains("team-alpha"));
    }

    #[test]
    fn agent_and_attach_scripts_default_to_current_task_intent() {
        let shell = render_agent_shell_profile(Path::new(".memd"), Some("codex"));
        let ps1 = render_agent_ps1_profile(Path::new(".memd"), Some("codex"));
        let attach = render_attach_snippet("bash", Path::new(".memd")).expect("attach snippet");

        assert!(shell.contains("--intent current_task"));
        assert!(ps1.contains("--intent current_task"));
        assert!(attach.contains("--intent current_task"));
    }

    #[test]
    fn resume_prompt_surfaces_current_task_snapshot() {
        let snapshot = ResumeSnapshot {
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            context: memd_schema::CompactContextResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::CurrentTask,
                retrieval_order: vec![memd_schema::MemoryScope::Project],
                records: Vec::new(),
            },
            working: memd_schema::WorkingMemoryResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::CurrentTask,
                retrieval_order: vec![memd_schema::MemoryScope::Project],
                budget_chars: 1600,
                used_chars: 60,
                remaining_chars: 1540,
                truncated: false,
                policy: memd_schema::WorkingMemoryPolicyState {
                    admission_limit: 8,
                    max_chars_per_item: 220,
                    budget_chars: 1600,
                    rehydration_limit: 4,
                },
                records: vec![memd_schema::CompactMemoryRecord {
                    id: uuid::Uuid::new_v4(),
                    record: "Follow the active current-task lane".to_string(),
                }],
                evicted: Vec::new(),
                rehydration_queue: vec![memd_schema::MemoryRehydrationRecord {
                    id: None,
                    kind: "source".to_string(),
                    label: "handoff".to_string(),
                    summary: "Reload the shared workspace handoff".to_string(),
                    reason: None,
                    source_agent: None,
                    source_system: None,
                    source_path: None,
                    source_quality: None,
                    recorded_at: None,
                }],
                traces: Vec::new(),
                semantic_consolidation: None,
            },
            inbox: memd_schema::MemoryInboxResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::CurrentTask,
                items: vec![memd_schema::InboxMemoryItem {
                    item: memd_schema::MemoryItem {
                        id: uuid::Uuid::new_v4(),
                        content: "One review item is still open".to_string(),
                        redundancy_key: None,
                        belief_branch: None,
                        preferred: true,
                        kind: memd_schema::MemoryKind::Status,
                        scope: memd_schema::MemoryScope::Project,
                        project: Some("demo".to_string()),
                        namespace: Some("main".to_string()),
                        workspace: Some("team-alpha".to_string()),
                        visibility: memd_schema::MemoryVisibility::Workspace,
                        source_agent: None,
                        source_system: None,
                        source_path: None,
                        source_quality: None,
                        confidence: 0.8,
                        ttl_seconds: Some(86_400),
                        created_at: chrono::Utc::now(),
                        status: memd_schema::MemoryStatus::Active,
                        stage: memd_schema::MemoryStage::Candidate,
                        last_verified_at: None,
                        supersedes: Vec::new(),
                        updated_at: chrono::Utc::now(),
                        tags: vec!["checkpoint".to_string()],
                    },
                    reasons: vec!["stale".to_string()],
                }],
            },
            workspaces: memd_schema::WorkspaceMemoryResponse {
                workspaces: vec![memd_schema::WorkspaceMemoryRecord {
                    project: Some("demo".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("team-alpha".to_string()),
                    visibility: memd_schema::MemoryVisibility::Workspace,
                    item_count: 4,
                    active_count: 3,
                    candidate_count: 1,
                    contested_count: 0,
                    source_lane_count: 1,
                    avg_confidence: 0.84,
                    trust_score: 0.91,
                    last_seen_at: None,
                    tags: Vec::new(),
                }],
            },
            semantic: None,
            change_summary: vec!["focus -> Follow the active current-task lane".to_string()],
            resume_state_age_minutes: None,
            refresh_recommended: false,
        };

        let prompt = crate::render::render_resume_prompt(&snapshot);
        assert!(prompt.contains("## Context Budget"));
        assert!(prompt.contains("estimated_tokens"));
        assert!(prompt.contains("redundant_items: 0"));
        assert!(prompt.contains("pressure: low"));
        assert!(prompt.contains("## Current Task Snapshot"));
        assert!(prompt.contains("## Since Last Resume"));
        assert!(prompt.contains("Follow the active current-task lane"));
        assert!(prompt.contains("One review item is still open"));
        assert!(prompt.contains("Reload the shared workspace handoff"));
        assert!(prompt.contains("team-alpha"));
    }

    #[test]
    fn resume_snapshot_detects_redundant_context_items() {
        let base = ResumeSnapshot {
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            context: memd_schema::CompactContextResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::CurrentTask,
                retrieval_order: vec![memd_schema::MemoryScope::Project],
                records: vec![memd_schema::CompactMemoryRecord {
                    id: uuid::Uuid::new_v4(),
                    record: "Repeat this exact idea".to_string(),
                }],
            },
            working: memd_schema::WorkingMemoryResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::CurrentTask,
                retrieval_order: vec![memd_schema::MemoryScope::Project],
                budget_chars: 1600,
                used_chars: 120,
                remaining_chars: 1480,
                truncated: false,
                policy: memd_schema::WorkingMemoryPolicyState {
                    admission_limit: 8,
                    max_chars_per_item: 220,
                    budget_chars: 1600,
                    rehydration_limit: 4,
                },
                records: vec![memd_schema::CompactMemoryRecord {
                    id: uuid::Uuid::new_v4(),
                    record: "Repeat this exact idea".to_string(),
                }],
                evicted: Vec::new(),
                rehydration_queue: vec![memd_schema::MemoryRehydrationRecord {
                    id: None,
                    kind: "source".to_string(),
                    label: "dup".to_string(),
                    summary: "Repeat this exact idea".to_string(),
                    reason: None,
                    source_agent: None,
                    source_system: None,
                    source_path: None,
                    source_quality: None,
                    recorded_at: None,
                }],
                traces: Vec::new(),
                semantic_consolidation: None,
            },
            inbox: memd_schema::MemoryInboxResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::CurrentTask,
                items: vec![memd_schema::InboxMemoryItem {
                    item: memd_schema::MemoryItem {
                        id: uuid::Uuid::new_v4(),
                        content: "Repeat this exact idea".to_string(),
                        redundancy_key: None,
                        belief_branch: None,
                        preferred: true,
                        kind: memd_schema::MemoryKind::Status,
                        scope: memd_schema::MemoryScope::Project,
                        project: Some("demo".to_string()),
                        namespace: Some("main".to_string()),
                        workspace: Some("team-alpha".to_string()),
                        visibility: memd_schema::MemoryVisibility::Workspace,
                        source_agent: None,
                        source_system: None,
                        source_path: None,
                        source_quality: None,
                        confidence: 0.9,
                        ttl_seconds: None,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        last_verified_at: None,
                        supersedes: Vec::new(),
                        tags: Vec::new(),
                        status: memd_schema::MemoryStatus::Active,
                        stage: memd_schema::MemoryStage::Candidate,
                    },
                    reasons: vec!["same".to_string()],
                }],
            },
            workspaces: memd_schema::WorkspaceMemoryResponse {
                workspaces: Vec::new(),
            },
            semantic: None,
            change_summary: Vec::new(),
            resume_state_age_minutes: None,
            refresh_recommended: false,
        };

        assert!(base.redundant_context_items() >= 3);
        assert_eq!(base.context_pressure(), "high");
        assert!(
            base.optimization_hints()
                .iter()
                .any(|hint| hint.contains("collapse 3 repeated context item"))
        );
    }

    #[test]
    fn resume_state_changes_capture_hot_lane_deltas() {
        let previous = BundleResumeState {
            focus: Some("old focus".to_string()),
            pressure: Some("old pressure".to_string()),
            next_recovery: Some("artifact: old".to_string()),
            lane: Some("demo / main / alpha".to_string()),
            working_records: 2,
            inbox_items: 1,
            rehydration_items: 1,
            recorded_at: Utc::now(),
        };
        let current = BundleResumeState {
            focus: Some("new focus".to_string()),
            pressure: Some("new pressure".to_string()),
            next_recovery: Some("artifact: new".to_string()),
            lane: Some("demo / main / beta".to_string()),
            working_records: 4,
            inbox_items: 0,
            rehydration_items: 2,
            recorded_at: Utc::now(),
        };

        let changes = describe_resume_state_changes(Some(&previous), &current);
        assert!(
            changes
                .iter()
                .any(|value| value.contains("focus -> new focus"))
        );
        assert!(
            changes
                .iter()
                .any(|value| value.contains("pressure -> new pressure"))
        );
        assert!(
            changes
                .iter()
                .any(|value| value.contains("next_recovery -> artifact: new"))
        );
        assert!(
            changes
                .iter()
                .any(|value| value.contains("lane -> demo / main / beta"))
        );
        assert!(changes.iter().any(|value| value.contains("working 2 -> 4")));
        assert!(changes.iter().any(|value| value.contains("inbox 1 -> 0")));
        assert!(
            changes
                .iter()
                .any(|value| value.contains("rehydration 1 -> 2"))
        );
    }

    #[test]
    fn builds_bundle_agent_profiles_for_known_agents() {
        let dir =
            std::env::temp_dir().join(format!("memd-agent-profiles-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");
        fs::write(
            dir.join("config.json"),
            r#"{
  "project": "demo",
  "agent": "codex",
  "session": "codex-a",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "general"
}
"#,
        )
        .expect("write config");
        let response =
            build_bundle_agent_profiles(&dir, None, Some("bash")).expect("agent profiles");
        assert_eq!(response.agents.len(), 4);
        assert_eq!(response.shell, "bash");
        assert_eq!(response.current.as_deref(), Some("codex"));
        assert_eq!(response.current_session.as_deref(), Some("codex-a"));
        assert_eq!(response.agents[0].name, "codex");
        assert_eq!(response.agents[0].effective_agent, "codex@codex-a");
        assert!(
            response.agents[0]
                .memory_file
                .ends_with("agents/CODEX_MEMORY.md")
        );
        assert!(response.agents[0].launch_hint.contains("codex.sh"));
        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn filters_bundle_agent_profiles_by_name() {
        let dir =
            std::env::temp_dir().join(format!("memd-agent-selected-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");
        fs::write(
            dir.join("config.json"),
            r#"{
  "project": "demo",
  "agent": "claude-code",
  "session": "claude-a",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "general"
}
"#,
        )
        .expect("write config");
        let response = build_bundle_agent_profiles(&dir, Some("claude-code"), Some("pwsh"))
            .expect("agent profiles");
        assert_eq!(response.agents.len(), 1);
        assert_eq!(response.current.as_deref(), Some("claude-code"));
        assert_eq!(response.selected.as_deref(), Some("claude-code"));
        assert_eq!(response.agents[0].name, "claude-code");
        assert!(response.agents[0].launch_hint.contains("claude-code.ps1"));
        assert!(
            response.agents[0]
                .native_hint
                .as_deref()
                .unwrap_or_default()
                .contains("CLAUDE_IMPORTS.md")
        );
        let summary = render_bundle_agent_profiles_summary(&response);
        assert!(summary.contains("current=claude-code"));
        assert!(summary.contains("session=claude-a"));
        assert!(summary.contains("claude-code [active]"));
        assert!(summary.contains("effective claude-code@claude-a"));
        assert!(summary.contains("/memory"));
        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn set_bundle_agent_updates_config_and_env_files() {
        let dir = std::env::temp_dir().join(format!("memd-agent-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");
        fs::write(
            dir.join("config.json"),
            r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "general"
}
"#,
        )
        .expect("write config");
        fs::write(
            dir.join("env"),
            "MEMD_AGENT=codex@codex-a\nMEMD_SESSION=codex-a\n",
        )
        .expect("write env");
        fs::write(
            dir.join("env.ps1"),
            "$env:MEMD_AGENT = \"codex@codex-a\"\n$env:MEMD_SESSION = \"codex-a\"\n",
        )
        .expect("write env.ps1");

        set_bundle_agent(&dir, "openclaw").expect("set bundle agent");

        let config = fs::read_to_string(dir.join("config.json")).expect("read config");
        let env = fs::read_to_string(dir.join("env")).expect("read env");
        let env_ps1 = fs::read_to_string(dir.join("env.ps1")).expect("read env.ps1");
        assert!(config.contains(r#""agent": "openclaw""#));
        assert!(env.contains("MEMD_AGENT=openclaw@codex-a"));
        assert!(env_ps1.contains("$env:MEMD_AGENT = \"openclaw@codex-a\""));

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn set_bundle_auto_short_term_capture_updates_config_and_env_files() {
        let dir = std::env::temp_dir().join(format!("memd-bundle-policy-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");
        fs::write(
            dir.join("config.json"),
            r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "general",
  "auto_short_term_capture": true
}
"#,
        )
        .expect("write config");
        fs::write(
            dir.join("env"),
            "MEMD_AGENT=codex@codex-a\nMEMD_SESSION=codex-a\nMEMD_AUTO_SHORT_TERM_CAPTURE=true\n",
        )
        .expect("write env");
        fs::write(
            dir.join("env.ps1"),
            "$env:MEMD_AGENT = \"codex@codex-a\"\n$env:MEMD_SESSION = \"codex-a\"\n$env:MEMD_AUTO_SHORT_TERM_CAPTURE = \"true\"\n",
        )
        .expect("write env.ps1");

        set_bundle_auto_short_term_capture(&dir, false).expect("set bundle policy");

        let config = fs::read_to_string(dir.join("config.json")).expect("read config");
        let env = fs::read_to_string(dir.join("env")).expect("read env");
        let env_ps1 = fs::read_to_string(dir.join("env.ps1")).expect("read env.ps1");
        assert!(config.contains(r#""auto_short_term_capture": false"#));
        assert!(env.contains("MEMD_AUTO_SHORT_TERM_CAPTURE=false"));
        assert!(env_ps1.contains("$env:MEMD_AUTO_SHORT_TERM_CAPTURE = \"false\""));

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn project_awareness_scans_sibling_bundles_without_current() {
        let root =
            std::env::temp_dir().join(format!("memd-awareness-root-{}", uuid::Uuid::new_v4()));
        let current_project = root.join("current");
        let sibling_project = root.join("sibling");
        fs::create_dir_all(current_project.join(".memd").join("state")).expect("create current");
        fs::create_dir_all(sibling_project.join(".memd").join("state")).expect("create sibling");

        fs::write(
            current_project.join(".memd").join("config.json"),
            r#"{
  "project": "current",
  "namespace": "main",
  "agent": "codex",
  "workspace": "current-lane",
  "visibility": "workspace"
}
"#,
        )
        .expect("write current config");
        fs::write(
            sibling_project.join(".memd").join("config.json"),
            r#"{
  "project": "sibling",
  "namespace": "main",
  "agent": "claude-code",
  "workspace": "research",
  "visibility": "workspace"
}
"#,
        )
        .expect("write sibling config");
        fs::write(
            sibling_project
                .join(".memd")
                .join("state")
                .join("last-resume.json"),
            r#"{
  "focus": "Finish the sibling task",
  "pressure": "Resolve review comments",
  "next_recovery": "Re-open the last handoff",
  "lane": "sibling / main / research",
  "working_records": 2,
  "inbox_items": 1,
  "rehydration_items": 1
}
"#,
        )
        .expect("write sibling state");

        let response = read_project_awareness(&AwarenessArgs {
            output: current_project.join(".memd"),
            root: Some(root.clone()),
            include_current: false,
            summary: false,
        })
        .expect("read awareness");

        assert_eq!(response.entries.len(), 1);
        let entry = &response.entries[0];
        assert_eq!(entry.project.as_deref(), Some("sibling"));
        assert_eq!(entry.agent.as_deref(), Some("claude-code"));
        assert_eq!(entry.workspace.as_deref(), Some("research"));
        assert_eq!(entry.focus.as_deref(), Some("Finish the sibling task"));
        assert_eq!(entry.pressure.as_deref(), Some("Resolve review comments"));

        fs::remove_dir_all(root).expect("cleanup awareness root");
    }

    #[test]
    fn project_awareness_summary_compacts_focus_and_pressure() {
        let response = ProjectAwarenessResponse {
            root: "/tmp/projects".to_string(),
            current_bundle: "/tmp/projects/current/.memd".to_string(),
            collisions: Vec::new(),
            entries: vec![ProjectAwarenessEntry {
                project_dir: "/tmp/projects/sibling".to_string(),
                bundle_root: "/tmp/projects/sibling/.memd".to_string(),
                project: Some("sibling".to_string()),
                namespace: Some("main".to_string()),
                agent: Some("claude-code".to_string()),
                session: Some("claude-a".to_string()),
                effective_agent: Some("claude-code@claude-a".to_string()),
                base_url: None,
                presence: "active".to_string(),
                host: None,
                pid: None,
                active_claims: 0,
                workspace: Some("research".to_string()),
                visibility: Some("workspace".to_string()),
                focus: Some("Investigate whether the recall lane is still stale".to_string()),
                pressure: Some("Repair the shared lane before the next resume".to_string()),
                next_recovery: None,
                last_updated: None,
            }],
        };

        let summary = render_project_awareness_summary(&response);
        assert!(summary.contains("awareness root=/tmp/projects bundles=1 collisions=0"));
        assert!(summary.contains(
            "sibling | presence=active claims=0 ns=main agent=claude-code@claude-a session=claude-a base_url=none workspace=research"
        ));
        assert!(summary.contains("focus=\"Investigate whether the recall lane is still stale\""));
        assert!(summary.contains("pressure=\"Repair the shared lane before the next resume\""));
    }

    #[test]
    fn project_awareness_surfaces_base_url_collisions() {
        let response = ProjectAwarenessResponse {
            root: "/tmp/projects".to_string(),
            current_bundle: "/tmp/projects/current/.memd".to_string(),
            collisions: vec!["base_url http://127.0.0.1:8787 used by 2 bundles".to_string()],
            entries: vec![
                ProjectAwarenessEntry {
                    project_dir: "/tmp/projects/a".to_string(),
                    bundle_root: "/tmp/projects/a/.memd".to_string(),
                    project: Some("a".to_string()),
                    namespace: Some("main".to_string()),
                    agent: Some("codex".to_string()),
                    session: Some("codex-a".to_string()),
                    effective_agent: Some("codex@codex-a".to_string()),
                    base_url: Some("http://127.0.0.1:8787".to_string()),
                    presence: "active".to_string(),
                    host: None,
                    pid: None,
                    active_claims: 1,
                    workspace: Some("a".to_string()),
                    visibility: Some("workspace".to_string()),
                    focus: None,
                    pressure: None,
                    next_recovery: None,
                    last_updated: None,
                },
                ProjectAwarenessEntry {
                    project_dir: "/tmp/projects/b".to_string(),
                    bundle_root: "/tmp/projects/b/.memd".to_string(),
                    project: Some("b".to_string()),
                    namespace: Some("main".to_string()),
                    agent: Some("claude-code".to_string()),
                    session: Some("claude-b".to_string()),
                    effective_agent: Some("claude-code@claude-b".to_string()),
                    base_url: Some("http://127.0.0.1:8787".to_string()),
                    presence: "active".to_string(),
                    host: None,
                    pid: None,
                    active_claims: 1,
                    workspace: Some("b".to_string()),
                    visibility: Some("workspace".to_string()),
                    focus: None,
                    pressure: None,
                    next_recovery: None,
                    last_updated: None,
                },
            ],
        };

        let summary = render_project_awareness_summary(&response);
        assert!(summary.contains("! base_url http://127.0.0.1:8787 used by 2 bundles"));
    }

    #[test]
    fn heartbeat_presence_labels_age_bands() {
        assert_eq!(heartbeat_presence_label(Utc::now()), "active");
        assert_eq!(
            heartbeat_presence_label(Utc::now() - chrono::TimeDelta::minutes(5)),
            "stale"
        );
        assert_eq!(
            heartbeat_presence_label(Utc::now() - chrono::TimeDelta::minutes(30)),
            "dead"
        );
    }

    #[test]
    fn render_bundle_heartbeat_summary_surfaces_presence_and_focus() {
        let state = BundleHeartbeatState {
            session: Some("codex-a".to_string()),
            agent: Some("codex".to_string()),
            effective_agent: Some("codex@codex-a".to_string()),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            base_url_healthy: Some(true),
            host: Some("workstation".to_string()),
            pid: Some(4242),
            focus: Some("Finish the live heartbeat lane".to_string()),
            pressure: Some("Avoid memory drift".to_string()),
            next_recovery: None,
            status: "live".to_string(),
            last_seen: Utc::now(),
        };

        let summary = render_bundle_heartbeat_summary(&state);
        assert!(summary.contains("heartbeat project=demo"));
        assert!(summary.contains("agent=codex@codex-a"));
        assert!(summary.contains("session=codex-a"));
        assert!(summary.contains("presence=active"));
        assert!(summary.contains("focus=\"Finish the live heartbeat lane\""));
        assert!(summary.contains("pressure=\"Avoid memory drift\""));
    }

    #[test]
    fn resolve_target_session_bundle_finds_matching_session() {
        let root =
            std::env::temp_dir().join(format!("memd-target-session-{}", uuid::Uuid::new_v4()));
        let current_project = root.join("current");
        let target_project = root.join("target");
        fs::create_dir_all(current_project.join(".memd").join("state")).expect("create current");
        fs::create_dir_all(target_project.join(".memd").join("state")).expect("create target");

        fs::write(
            current_project.join(".memd").join("config.json"),
            r#"{
  "project": "current",
  "agent": "codex",
  "session": "codex-a",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "general"
}
"#,
        )
        .expect("write current config");
        fs::write(
            target_project.join(".memd").join("config.json"),
            r#"{
  "project": "target",
  "agent": "claude-code",
  "session": "claude-b",
  "base_url": "http://127.0.0.1:9797",
  "route": "auto",
  "intent": "general"
}
"#,
        )
        .expect("write target config");
        fs::write(
            target_project
                .join(".memd")
                .join("state")
                .join("heartbeat.json"),
            serde_json::to_string_pretty(&BundleHeartbeatState {
                session: Some("claude-b".to_string()),
                agent: Some("claude-code".to_string()),
                effective_agent: Some("claude-code@claude-b".to_string()),
                project: Some("target".to_string()),
                namespace: None,
                workspace: Some("research".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:9797".to_string()),
                base_url_healthy: Some(true),
                host: Some("workstation".to_string()),
                pid: Some(4242),
                focus: Some("Handle the delegated task".to_string()),
                pressure: None,
                next_recovery: None,
                status: "live".to_string(),
                last_seen: Utc::now(),
            })
            .expect("serialize heartbeat"),
        )
        .expect("write heartbeat");

        let resolved = resolve_target_session_bundle(&current_project.join(".memd"), "claude-b")
            .expect("resolve target")
            .expect("matching session");
        assert_eq!(resolved.project.as_deref(), Some("target"));
        assert_eq!(resolved.session.as_deref(), Some("claude-b"));
        assert_eq!(
            resolved.bundle_root,
            target_project.join(".memd").display().to_string()
        );

        fs::remove_dir_all(root).expect("cleanup target-session root");
    }

    #[tokio::test]
    async fn claims_acquire_and_release_scope() {
        let dir = std::env::temp_dir().join(format!("memd-claims-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(dir.join("state")).expect("create claims dir");
        let base_url = spawn_mock_peer_server().await;
        fs::write(
            dir.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "agent": "codex",
  "session": "codex-a",
  "workspace": "shared",
  "base_url": "{}",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "general"
}}
"#,
                base_url
            ),
        )
        .expect("write config");
        fs::write(
            dir.join("state").join("heartbeat.json"),
            serde_json::to_string_pretty(&BundleHeartbeatState {
                session: Some("codex-a".to_string()),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@codex-a".to_string()),
                project: Some("demo".to_string()),
                namespace: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some(base_url.clone()),
                base_url_healthy: Some(true),
                host: Some("workstation".to_string()),
                pid: Some(1111),
                focus: None,
                pressure: None,
                next_recovery: None,
                status: "live".to_string(),
                last_seen: Utc::now(),
            })
            .expect("serialize heartbeat"),
        )
        .expect("write heartbeat");

        let acquired = run_claims_command(
            &ClaimsArgs {
                output: dir.clone(),
                acquire: true,
                release: false,
                transfer_to_session: None,
                scope: Some("file:src/main.rs".to_string()),
                ttl_secs: 900,
                summary: false,
            },
            &base_url,
        )
        .await
        .expect("acquire claim");
        assert_eq!(acquired.claims.len(), 1);
        assert_eq!(acquired.claims[0].scope, "file:src/main.rs");

        let released = run_claims_command(
            &ClaimsArgs {
                output: dir.clone(),
                acquire: false,
                release: true,
                transfer_to_session: None,
                scope: Some("file:src/main.rs".to_string()),
                ttl_secs: 900,
                summary: false,
            },
            &base_url,
        )
        .await
        .expect("release claim");
        assert_eq!(released.claims.len(), 1);
        assert_eq!(released.claims[0].scope, "file:src/main.rs");

        fs::remove_dir_all(dir).expect("cleanup claims dir");
    }

    #[tokio::test]
    async fn claims_transfer_scope_to_target_session() {
        let root =
            std::env::temp_dir().join(format!("memd-claim-transfer-{}", uuid::Uuid::new_v4()));
        let current_project = root.join("current");
        let target_project = root.join("target");
        let current_bundle = current_project.join(".memd");
        let target_bundle = target_project.join(".memd");
        fs::create_dir_all(current_bundle.join("state")).expect("create current claims dir");
        fs::create_dir_all(target_bundle.join("state")).expect("create target claims dir");
        let base_url = spawn_mock_peer_server().await;

        fs::write(
            current_bundle.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "agent": "codex",
  "session": "codex-a",
  "workspace": "shared",
  "base_url": "{}",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "general"
}}
"#,
                base_url
            ),
        )
        .expect("write current config");
        fs::write(
            current_bundle.join("state").join("heartbeat.json"),
            serde_json::to_string_pretty(&BundleHeartbeatState {
                session: Some("codex-a".to_string()),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@codex-a".to_string()),
                project: Some("demo".to_string()),
                namespace: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some(base_url.clone()),
                base_url_healthy: Some(true),
                host: Some("workstation".to_string()),
                pid: Some(1111),
                focus: None,
                pressure: None,
                next_recovery: None,
                status: "live".to_string(),
                last_seen: Utc::now(),
            })
            .expect("serialize current heartbeat"),
        )
        .expect("write current heartbeat");

        fs::write(
            target_bundle.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "agent": "claude-code",
  "session": "claude-b",
  "workspace": "shared",
  "base_url": "{}",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "general"
}}
"#,
                base_url
            ),
        )
        .expect("write target config");
        fs::write(
            target_bundle.join("state").join("heartbeat.json"),
            serde_json::to_string_pretty(&BundleHeartbeatState {
                session: Some("claude-b".to_string()),
                agent: Some("claude-code".to_string()),
                effective_agent: Some("claude-code@claude-b".to_string()),
                project: Some("demo".to_string()),
                namespace: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some(base_url.clone()),
                base_url_healthy: Some(true),
                host: Some("workstation".to_string()),
                pid: Some(2222),
                focus: None,
                pressure: None,
                next_recovery: None,
                status: "live".to_string(),
                last_seen: Utc::now(),
            })
            .expect("serialize target heartbeat"),
        )
        .expect("write target heartbeat");

        let acquired = run_claims_command(
            &ClaimsArgs {
                output: current_bundle.clone(),
                acquire: true,
                release: false,
                transfer_to_session: None,
                scope: Some("task:parser-refactor".to_string()),
                ttl_secs: 900,
                summary: false,
            },
            &base_url,
        )
        .await
        .expect("acquire claim");
        assert_eq!(acquired.claims[0].session.as_deref(), Some("codex-a"));

        let transferred = run_claims_command(
            &ClaimsArgs {
                output: current_bundle.clone(),
                acquire: false,
                release: false,
                transfer_to_session: Some("claude-b".to_string()),
                scope: Some("task:parser-refactor".to_string()),
                ttl_secs: 900,
                summary: false,
            },
            &base_url,
        )
        .await
        .expect("transfer claim");
        assert_eq!(transferred.claims.len(), 1);
        assert_eq!(transferred.claims[0].session.as_deref(), Some("claude-b"));
        assert_eq!(
            transferred.claims[0].effective_agent.as_deref(),
            Some("claude-code@claude-b")
        );

        fs::remove_dir_all(root).expect("cleanup transfer dir");
    }

    #[tokio::test]
    async fn messages_send_and_ack_for_target_session() {
        let root = std::env::temp_dir().join(format!("memd-messages-{}", uuid::Uuid::new_v4()));
        let current_project = root.join("current");
        let target_project = root.join("target");
        let current_bundle = current_project.join(".memd");
        let target_bundle = target_project.join(".memd");
        fs::create_dir_all(&current_bundle).expect("create current bundle");
        fs::create_dir_all(&target_bundle).expect("create target bundle");
        let current_base_url = spawn_mock_peer_server().await;
        let target_base_url = spawn_mock_peer_server().await;

        fs::write(
            current_bundle.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "agent": "codex",
  "session": "codex-a",
  "workspace": "shared",
  "base_url": "{}",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "general"
}}
"#,
                current_base_url
            ),
        )
        .expect("write config");
        fs::write(
            target_bundle.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "agent": "claude-code",
  "session": "claude-b",
  "workspace": "shared",
  "base_url": "{}",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "general"
}}
"#,
                target_base_url
            ),
        )
        .expect("write target config");

        let sent = run_messages_command(
            &MessagesArgs {
                output: current_bundle.clone(),
                send: true,
                inbox: false,
                ack: None,
                target_session: Some("claude-b".to_string()),
                kind: Some("handoff".to_string()),
                request_help: false,
                request_review: false,
                assign_scope: None,
                scope: None,
                content: Some("Pick up the parser refactor".to_string()),
                summary: false,
            },
            &current_base_url,
        )
        .await
        .expect("send message");
        assert_eq!(sent.messages.len(), 1);
        assert_eq!(sent.messages[0].to_session, "claude-b");

        let inbox = run_messages_command(
            &MessagesArgs {
                output: target_bundle.clone(),
                send: false,
                inbox: true,
                ack: None,
                target_session: None,
                kind: None,
                request_help: false,
                request_review: false,
                assign_scope: None,
                scope: None,
                content: None,
                summary: false,
            },
            &target_base_url,
        )
        .await
        .expect("read inbox");
        assert_eq!(inbox.messages.len(), 1);
        let message_id = inbox.messages[0].id.clone();

        let acked = run_messages_command(
            &MessagesArgs {
                output: target_bundle.clone(),
                send: false,
                inbox: true,
                ack: Some(message_id),
                target_session: None,
                kind: None,
                request_help: false,
                request_review: false,
                assign_scope: None,
                scope: None,
                content: None,
                summary: false,
            },
            &target_base_url,
        )
        .await
        .expect("ack message");
        assert!(acked.messages[0].acknowledged_at.is_some());

        fs::remove_dir_all(root).expect("cleanup messages dir");
    }

    #[test]
    fn set_bundle_base_url_updates_config_and_env_files() {
        let dir =
            std::env::temp_dir().join(format!("memd-bundle-base-url-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");
        fs::write(
            dir.join("config.json"),
            r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "general"
}
"#,
        )
        .expect("write config");
        fs::write(dir.join("env"), "MEMD_BASE_URL=http://127.0.0.1:8787\n").expect("write env");
        fs::write(
            dir.join("env.ps1"),
            "$env:MEMD_BASE_URL = \"http://127.0.0.1:8787\"\n",
        )
        .expect("write env.ps1");

        set_bundle_base_url(&dir, "http://127.0.0.1:9797").expect("set bundle base url");

        let config = fs::read_to_string(dir.join("config.json")).expect("read config");
        let env = fs::read_to_string(dir.join("env")).expect("read env");
        let env_ps1 = fs::read_to_string(dir.join("env.ps1")).expect("read env.ps1");
        assert!(config.contains(r#""base_url": "http://127.0.0.1:9797""#));
        assert!(env.contains("MEMD_BASE_URL=http://127.0.0.1:9797"));
        assert!(env_ps1.contains("$env:MEMD_BASE_URL = \"http://127.0.0.1:9797\""));

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn describes_eval_changes_against_baseline() {
        let baseline = BundleEvalResponse {
            bundle_root: ".memd".to_string(),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            status: "usable".to_string(),
            score: 72,
            working_records: 2,
            context_records: 1,
            rehydration_items: 1,
            inbox_items: 3,
            workspace_lanes: 1,
            semantic_hits: 0,
            findings: Vec::new(),
            baseline_score: None,
            score_delta: None,
            changes: Vec::new(),
            recommendations: Vec::new(),
        };
        let snapshot = ResumeSnapshot {
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            route: "auto".to_string(),
            intent: "general".to_string(),
            context: memd_schema::CompactContextResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::General,
                retrieval_order: vec![memd_schema::MemoryScope::Project],
                records: vec![memd_schema::CompactMemoryRecord {
                    id: uuid::Uuid::new_v4(),
                    record: "ctx".to_string(),
                }],
            },
            working: memd_schema::WorkingMemoryResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::General,
                retrieval_order: vec![memd_schema::MemoryScope::Project],
                budget_chars: 1600,
                used_chars: 100,
                remaining_chars: 1500,
                truncated: false,
                policy: memd_schema::WorkingMemoryPolicyState {
                    admission_limit: 8,
                    max_chars_per_item: 220,
                    budget_chars: 1600,
                    rehydration_limit: 4,
                },
                records: vec![
                    memd_schema::CompactMemoryRecord {
                        id: uuid::Uuid::new_v4(),
                        record: "one".to_string(),
                    },
                    memd_schema::CompactMemoryRecord {
                        id: uuid::Uuid::new_v4(),
                        record: "two".to_string(),
                    },
                    memd_schema::CompactMemoryRecord {
                        id: uuid::Uuid::new_v4(),
                        record: "three".to_string(),
                    },
                ],
                evicted: Vec::new(),
                rehydration_queue: vec![
                    memd_schema::MemoryRehydrationRecord {
                        id: None,
                        kind: "source".to_string(),
                        label: "artifact".to_string(),
                        summary: "more".to_string(),
                        reason: None,
                        source_agent: None,
                        source_system: None,
                        source_path: None,
                        source_quality: None,
                        recorded_at: None,
                    },
                    memd_schema::MemoryRehydrationRecord {
                        id: None,
                        kind: "source".to_string(),
                        label: "artifact-2".to_string(),
                        summary: "more".to_string(),
                        reason: None,
                        source_agent: None,
                        source_system: None,
                        source_path: None,
                        source_quality: None,
                        recorded_at: None,
                    },
                ],
                traces: Vec::new(),
                semantic_consolidation: None,
            },
            inbox: memd_schema::MemoryInboxResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::General,
                items: vec![],
            },
            workspaces: memd_schema::WorkspaceMemoryResponse {
                workspaces: vec![
                    memd_schema::WorkspaceMemoryRecord {
                        project: Some("demo".to_string()),
                        namespace: Some("main".to_string()),
                        workspace: Some("team-alpha".to_string()),
                        visibility: memd_schema::MemoryVisibility::Workspace,
                        item_count: 3,
                        active_count: 3,
                        candidate_count: 0,
                        contested_count: 0,
                        source_lane_count: 1,
                        avg_confidence: 0.9,
                        trust_score: 0.9,
                        last_seen_at: None,
                        tags: vec![],
                    },
                    memd_schema::WorkspaceMemoryRecord {
                        project: Some("demo".to_string()),
                        namespace: Some("main".to_string()),
                        workspace: Some("shared".to_string()),
                        visibility: memd_schema::MemoryVisibility::Workspace,
                        item_count: 2,
                        active_count: 2,
                        candidate_count: 0,
                        contested_count: 0,
                        source_lane_count: 1,
                        avg_confidence: 0.8,
                        trust_score: 0.8,
                        last_seen_at: None,
                        tags: vec![],
                    },
                ],
            },
            semantic: Some(memd_rag::RagRetrieveResponse {
                status: "ok".to_string(),
                mode: memd_rag::RagRetrieveMode::Auto,
                items: vec![memd_rag::RagRetrieveItem {
                    content: "semantic".to_string(),
                    source: Some("wiki/demo.md".to_string()),
                    score: 0.9,
                }],
            }),
            change_summary: Vec::new(),
            resume_state_age_minutes: None,
            refresh_recommended: false,
        };

        let changes = describe_eval_changes(&baseline, 88, &snapshot);
        assert!(changes.iter().any(|value| value.contains("score 72 -> 88")));
        assert!(changes.iter().any(|value| value.contains("working 2 -> 3")));
        assert!(
            changes
                .iter()
                .any(|value| value.contains("rehydration 1 -> 2"))
        );
        assert!(changes.iter().any(|value| value.contains("inbox 3 -> 0")));
        assert!(changes.iter().any(|value| value.contains("lanes 1 -> 2")));
        assert!(
            changes
                .iter()
                .any(|value| value.contains("semantic 0 -> 1"))
        );
    }

    #[test]
    fn eval_failure_reason_respects_score_threshold() {
        let response = BundleEvalResponse {
            bundle_root: ".memd".to_string(),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            status: "weak".to_string(),
            score: 62,
            working_records: 0,
            context_records: 0,
            rehydration_items: 0,
            inbox_items: 0,
            workspace_lanes: 0,
            semantic_hits: 0,
            findings: vec!["no working memory".to_string()],
            baseline_score: Some(70),
            score_delta: Some(-8),
            changes: vec!["score 70 -> 62".to_string()],
            recommendations: vec!["capture durable memory".to_string()],
        };

        let reason = eval_failure_reason(&response, Some(70), false).expect("threshold failure");
        assert!(reason.contains("score 62"));
        assert!(reason.contains("threshold 70"));
    }

    #[test]
    fn eval_failure_reason_respects_regression_gate() {
        let response = BundleEvalResponse {
            bundle_root: ".memd".to_string(),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            status: "usable".to_string(),
            score: 79,
            working_records: 3,
            context_records: 2,
            rehydration_items: 2,
            inbox_items: 1,
            workspace_lanes: 1,
            semantic_hits: 2,
            findings: Vec::new(),
            baseline_score: Some(83),
            score_delta: Some(-4),
            changes: vec!["score 83 -> 79".to_string()],
            recommendations: vec!["write a fresh baseline".to_string()],
        };

        let reason = eval_failure_reason(&response, None, true).expect("regression failure");
        assert!(reason.contains("baseline 83"));
        assert!(reason.contains("to 79"));
        assert!(reason.contains("delta -4"));
    }

    #[test]
    fn eval_failure_reason_passes_when_gates_are_clear() {
        let response = BundleEvalResponse {
            bundle_root: ".memd".to_string(),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            status: "strong".to_string(),
            score: 91,
            working_records: 4,
            context_records: 3,
            rehydration_items: 2,
            inbox_items: 0,
            workspace_lanes: 2,
            semantic_hits: 3,
            findings: Vec::new(),
            baseline_score: Some(89),
            score_delta: Some(2),
            changes: vec!["score 89 -> 91".to_string()],
            recommendations: Vec::new(),
        };

        assert!(eval_failure_reason(&response, Some(80), true).is_none());
    }

    #[test]
    fn build_eval_recommendations_surfaces_actionable_followups() {
        let snapshot = ResumeSnapshot {
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            route: "auto".to_string(),
            intent: "general".to_string(),
            context: memd_schema::CompactContextResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::General,
                retrieval_order: vec![memd_schema::MemoryScope::Project],
                records: Vec::new(),
            },
            working: memd_schema::WorkingMemoryResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::General,
                retrieval_order: vec![memd_schema::MemoryScope::Project],
                budget_chars: 1600,
                used_chars: 0,
                remaining_chars: 1600,
                truncated: false,
                policy: memd_schema::WorkingMemoryPolicyState {
                    admission_limit: 8,
                    max_chars_per_item: 220,
                    budget_chars: 1600,
                    rehydration_limit: 4,
                },
                records: Vec::new(),
                evicted: Vec::new(),
                rehydration_queue: Vec::new(),
                traces: Vec::new(),
                semantic_consolidation: None,
            },
            inbox: memd_schema::MemoryInboxResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::General,
                items: vec![
                    memd_schema::InboxMemoryItem {
                        item: memd_schema::MemoryItem {
                            id: uuid::Uuid::new_v4(),
                            content: "one".to_string(),
                            redundancy_key: None,
                            belief_branch: None,
                            preferred: true,
                            kind: memd_schema::MemoryKind::Decision,
                            scope: memd_schema::MemoryScope::Project,
                            project: Some("demo".to_string()),
                            namespace: Some("main".to_string()),
                            workspace: Some("team-alpha".to_string()),
                            visibility: memd_schema::MemoryVisibility::Workspace,
                            source_agent: None,
                            source_system: None,
                            source_path: None,
                            source_quality: None,
                            confidence: 0.6,
                            ttl_seconds: None,
                            created_at: Utc::now(),
                            updated_at: Utc::now(),
                            last_verified_at: None,
                            supersedes: Vec::new(),
                            tags: Vec::new(),
                            status: memd_schema::MemoryStatus::Active,
                            stage: memd_schema::MemoryStage::Candidate,
                        },
                        reasons: Vec::new(),
                    };
                    6
                ],
            },
            workspaces: memd_schema::WorkspaceMemoryResponse {
                workspaces: Vec::new(),
            },
            semantic: Some(memd_rag::RagRetrieveResponse {
                status: "ok".to_string(),
                mode: memd_rag::RagRetrieveMode::Auto,
                items: Vec::new(),
            }),
            change_summary: Vec::new(),
            resume_state_age_minutes: None,
            refresh_recommended: false,
        };

        let recommendations = build_eval_recommendations(&snapshot, 62);
        assert!(
            recommendations
                .iter()
                .any(|value| value.contains("memd remember"))
        );
        assert!(
            recommendations
                .iter()
                .any(|value| value.contains("compact context"))
        );
        assert!(
            recommendations
                .iter()
                .any(|value| value.contains("rehydrate deeper context"))
        );
        assert!(
            recommendations
                .iter()
                .any(|value| value.contains("workspace or visibility"))
        );
        assert!(
            recommendations
                .iter()
                .any(|value| value.contains("inbox pressure"))
        );
        assert!(
            recommendations
                .iter()
                .any(|value| value.contains("LightRAG"))
        );
        assert!(
            recommendations
                .iter()
                .any(|value| value.contains("write a fresh baseline"))
        );
    }

    #[test]
    fn suggest_coordination_actions_emits_multi_priority_output() {
        let now = Utc::now();
        let inbox = PeerCoordinationInboxResponse {
            messages: vec![
                PeerMessageRecord {
                    id: "m-1".to_string(),
                    kind: "status_check".to_string(),
                    from_session: "peer-a".to_string(),
                    from_agent: None,
                    to_session: "codex".to_string(),
                    project: None,
                    namespace: None,
                    workspace: None,
                    content: "review this artifact".to_string(),
                    created_at: now,
                    acknowledged_at: None,
                },
                PeerMessageRecord {
                    id: "m-2".to_string(),
                    kind: "help_request".to_string(),
                    from_session: "peer-b".to_string(),
                    from_agent: None,
                    to_session: "codex".to_string(),
                    project: None,
                    namespace: None,
                    workspace: None,
                    content: "another request".to_string(),
                    created_at: now,
                    acknowledged_at: None,
                },
            ],
            owned_tasks: vec![],
            help_tasks: vec![],
            review_tasks: vec![],
        };

        let stale_sessions = vec!["peer-stale"];
        let active_peers = vec!["peer-helper"];
        let claims = vec![
            SessionClaim {
                scope: "shared/src.rs".to_string(),
                session: Some("peer-stale".to_string()),
                agent: Some("claude".to_string()),
                effective_agent: Some("codex".to_string()),
                project: None,
                workspace: None,
                host: None,
                pid: None,
                acquired_at: now,
                expires_at: now,
            },
            SessionClaim {
                scope: "shared/src.rs".to_string(),
                session: Some("peer-contender".to_string()),
                agent: None,
                effective_agent: None,
                project: None,
                workspace: None,
                host: None,
                pid: None,
                acquired_at: now,
                expires_at: now,
            },
        ];
        let tasks = vec![
            PeerTaskRecord {
                task_id: "task-exclusive".to_string(),
                title: "edit shared".to_string(),
                description: None,
                status: "assigned".to_string(),
                coordination_mode: "exclusive_write".to_string(),
                session: Some("peer-owner".to_string()),
                agent: Some("peer-owner".to_string()),
                effective_agent: None,
                project: None,
                namespace: None,
                workspace: None,
                claim_scopes: vec!["shared/src.rs".to_string()],
                help_requested: false,
                review_requested: false,
                created_at: now,
                updated_at: now,
            },
            PeerTaskRecord {
                task_id: "task-review".to_string(),
                title: "run review".to_string(),
                description: None,
                status: "in_progress".to_string(),
                coordination_mode: "shared_review".to_string(),
                session: Some("codex".to_string()),
                agent: Some("coder".to_string()),
                effective_agent: None,
                project: None,
                namespace: None,
                workspace: None,
                claim_scopes: vec![],
                help_requested: false,
                review_requested: false,
                created_at: now,
                updated_at: now,
            },
            PeerTaskRecord {
                task_id: "task-help".to_string(),
                title: "parallel assist".to_string(),
                description: None,
                status: "in_progress".to_string(),
                coordination_mode: "help_only".to_string(),
                session: Some("codex".to_string()),
                agent: Some("coder".to_string()),
                effective_agent: None,
                project: None,
                namespace: None,
                workspace: None,
                claim_scopes: vec![],
                help_requested: false,
                review_requested: false,
                created_at: now,
                updated_at: now,
            },
        ];
        let policy_conflicts = vec!["policy conflict for shared scope".to_string()];

        let suggestions = suggest_coordination_actions(
            &inbox,
            &stale_sessions,
            &active_peers,
            &claims,
            &tasks,
            "codex",
            &policy_conflicts,
        );

        assert_eq!(
            suggestions
                .iter()
                .filter(|s| s.action == "ack_message")
                .count(),
            2,
            "each inbox message should produce its own ack suggestion"
        );
        assert!(suggestions.iter().any(|s| s.action == "recover_session"));
        assert!(suggestions.iter().any(|s| s.action == "assign_scope"));
        assert!(suggestions.iter().any(|s| s.action == "request_review"));
        assert!(suggestions.iter().any(|s| s.action == "request_help"));
        assert!(
            suggestions
                .iter()
                .any(|s| s.stale_session.as_deref() == Some("peer-stale"))
        );
    }

    #[test]
    fn build_gap_candidates_generates_core_gap_signals() {
        let output = std::env::temp_dir().join(format!("memd-gap-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&output).expect("create temp output");

        let runtime = None;
        let resume = None;
        let commits = vec!["abc".to_string(), "def".to_string()];
        let mut evidence = Vec::new();

        let candidates = build_gap_candidates(
            &output,
            &runtime,
            &resume,
            None,
            None,
            None,
            &commits,
            &mut evidence,
        );

        assert!(
            candidates
                .iter()
                .any(|value| value.id == "memory:no_eval_snapshot"),
            "baseline eval signal should be present when no eval exists"
        );
        assert!(
            candidates
                .iter()
                .any(|value| value.id == "memory:missing_resume_state"),
            "resume signal should be present when resume and state are missing"
        );
        assert!(
            candidates
                .iter()
                .any(|value| value.id == "coordination:coordination_unreachable"),
            "coordination signal should be present when coordination snapshot is unavailable"
        );
        assert!(
            !evidence.is_empty(),
            "recent commits should generate at least one evidence string"
        );

        fs::remove_dir_all(&output).expect("cleanup temp output");
    }

    #[test]
    fn prioritize_gap_candidates_orders_high_to_low_priority() {
        let candidates = vec![
            GapCandidate {
                id: "memory:a".to_string(),
                area: "memory".to_string(),
                priority: 40,
                severity: "low".to_string(),
                signal: "low".to_string(),
                evidence: Vec::new(),
                recommendation: "low-priority".to_string(),
            },
            GapCandidate {
                id: "coordination:b".to_string(),
                area: "coordination".to_string(),
                priority: 90,
                severity: "high".to_string(),
                signal: "high".to_string(),
                evidence: Vec::new(),
                recommendation: "high-priority".to_string(),
            },
            GapCandidate {
                id: "memory:c".to_string(),
                area: "memory".to_string(),
                priority: 70,
                severity: "medium".to_string(),
                signal: "medium".to_string(),
                evidence: Vec::new(),
                recommendation: "medium-priority".to_string(),
            },
        ];
        let sorted = prioritize_gap_candidates(candidates, 2);
        assert_eq!(sorted[0].priority, 90);
        assert_eq!(sorted[1].priority, 70);
    }

    #[test]
    fn evaluate_gap_changes_detects_count_and_status_shift() {
        let baseline = GapReport {
            bundle_root: ".memd".to_string(),
            project: None,
            namespace: None,
            agent: None,
            session: None,
            workspace: None,
            visibility: None,
            limit: 8,
            commits_checked: 0,
            eval_status: Some("usable".to_string()),
            eval_score: Some(70),
            eval_score_delta: Some(-5),
            candidate_count: 6,
            high_priority_count: 2,
            top_priorities: Vec::new(),
            candidates: Vec::new(),
            recommendations: Vec::new(),
            changes: Vec::new(),
            evidence: Vec::new(),
            generated_at: Utc::now(),
            previous_candidate_count: None,
        };

        let current = GapReport {
            bundle_root: ".memd".to_string(),
            project: None,
            namespace: None,
            agent: None,
            session: None,
            workspace: None,
            visibility: None,
            limit: 8,
            commits_checked: 2,
            eval_status: Some("weak".to_string()),
            eval_score: Some(66),
            eval_score_delta: Some(-10),
            candidate_count: 2,
            high_priority_count: 1,
            top_priorities: Vec::new(),
            candidates: Vec::new(),
            recommendations: Vec::new(),
            changes: Vec::new(),
            evidence: Vec::new(),
            generated_at: Utc::now(),
            previous_candidate_count: None,
        };

        let changes = evaluate_gap_changes(&current, Some(&baseline));
        assert!(
            changes
                .iter()
                .any(|value| value.contains("candidate_count 6 -> 2"))
        );
        assert!(
            changes
                .iter()
                .any(|value| value.contains("eval_score Some(70) -> Some(66)"))
        );
        assert!(
            changes
                .iter()
                .any(|value| value.contains("eval_status=weak"))
        );
    }

    fn test_gap_report(
        candidate_count: usize,
        high_priority_count: usize,
        eval_score: Option<u8>,
        top_priorities: Vec<String>,
    ) -> GapReport {
        GapReport {
            bundle_root: ".memd".to_string(),
            project: None,
            namespace: None,
            agent: None,
            session: None,
            workspace: None,
            visibility: None,
            limit: 8,
            commits_checked: 0,
            eval_status: None,
            eval_score,
            eval_score_delta: None,
            candidate_count,
            high_priority_count,
            top_priorities,
            candidates: Vec::new(),
            recommendations: Vec::new(),
            changes: Vec::new(),
            evidence: Vec::new(),
            generated_at: Utc::now(),
            previous_candidate_count: None,
        }
    }

    #[test]
    fn build_improvement_actions_dedupes_and_limits() {
        let mut gap = test_gap_report(3, 2, Some(61), vec!["memory:low_eval_score".to_string()]);
        gap.candidates.push(GapCandidate {
            id: "memory:low_eval_score".to_string(),
            area: "memory".to_string(),
            priority: 95,
            severity: "high".to_string(),
            signal: "low_eval_score".to_string(),
            evidence: vec!["evidence".to_string()],
            recommendation: "refresh eval".to_string(),
        });
        let coordination = CoordinationResponse {
            bundle_root: ".memd".to_string(),
            current_session: "codex".to_string(),
            inbox: PeerCoordinationInboxResponse {
                messages: Vec::new(),
                owned_tasks: Vec::new(),
                help_tasks: Vec::new(),
                review_tasks: Vec::new(),
            },
            recovery: CoordinationRecoverySummary {
                stale_peers: Vec::new(),
                reclaimable_claims: Vec::new(),
                stalled_tasks: Vec::new(),
            },
            policy_conflicts: Vec::new(),
            suggestions: (0..10)
                .map(|index| CoordinationSuggestion {
                    action: "ack_message".to_string(),
                    priority: "medium".to_string(),
                    target_session: None,
                    task_id: None,
                    scope: None,
                    message_id: Some(format!("dup-{index}")),
                    reason: "dedupe check".to_string(),
                    stale_session: None,
                })
                .chain(std::iter::once(CoordinationSuggestion {
                    action: "ack_message".to_string(),
                    priority: "high".to_string(),
                    target_session: None,
                    task_id: None,
                    scope: None,
                    message_id: Some("dup-0".to_string()),
                    reason: "dedupe check".to_string(),
                    stale_session: None,
                }))
                .collect(),
            boundary_recommendations: Vec::new(),
            receipts: Vec::new(),
        };
        let actions = build_improvement_actions(&gap, Some(&coordination));
        assert!(
            actions.len() <= 8,
            "action list should be bounded by apply_improvement cap"
        );
        assert!(
            actions
                .iter()
                .filter(|value| value.action == "refresh_eval")
                .count()
                == 1,
            "low_eval_score only yields one refresh_eval action"
        );
        assert!(
            actions
                .iter()
                .filter(|value| value.message_id.as_deref() == Some("dup-0"))
                .count()
                <= 1,
            "duplicate suggestion keys should dedupe"
        );
    }

    #[test]
    fn improvement_progress_tracks_candidate_score_and_priority_change() {
        let baseline = test_gap_report(10, 3, Some(82), vec!["a".to_string(), "b".to_string()]);
        let fewer_candidates =
            test_gap_report(8, 3, Some(82), vec!["a".to_string(), "b".to_string()]);
        let better_score = test_gap_report(10, 3, Some(84), vec!["a".to_string(), "b".to_string()]);
        let changed_priorities =
            test_gap_report(10, 3, Some(82), vec!["x".to_string(), "a".to_string()]);
        let no_change = test_gap_report(10, 3, Some(82), vec!["a".to_string(), "b".to_string()]);

        assert!(improvement_progress(&baseline, &fewer_candidates));
        assert!(improvement_progress(&baseline, &better_score));
        assert!(improvement_progress(&baseline, &changed_priorities));
        assert!(!improvement_progress(&baseline, &no_change));
    }
}
