mod benchmark_runtime;
mod cli_runtime;
mod bundle_agent_profiles;
mod bundle_bootstrap_runtime;
mod bundle_config_runtime;
mod bundle_lane_runtime;
mod bundle_memory_surface;
mod bundle_models;
mod bundle_profile_runtime;
mod bundle_maintenance_runtime;
mod bundle_init_runtime;
mod bundle_runtime;
mod public_benchmark_runtime;
mod command_catalog;
mod commands;
mod compiled_event;
mod compiled_memory;
mod coordination_control;
mod coordination_runtime;
mod coordination_views;
mod evaluation_runtime;
mod evolution_runtime;
pub(crate) mod harness;
mod hive_commands_runtime;
mod hive_ops_runtime;
mod hive_runtime;
mod ingest_runtime;
mod inspiration_search;
mod improvement_runtime;
mod obsidian;
mod obsidian_commands;
mod obsidian_runtime;
mod obsidian_support;
mod render;
mod retrieval_runtime;
mod runtime_checkpoint;
mod runtime_resume;
mod autoresearch_runtime;
mod scenario_runtime;
mod skill_catalog;
mod verify_runtime;
mod verification_runtime;
mod workspace_runtime;

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    future::Future,
    io::{self, Read},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use crate::harness::cache;
use anyhow::{Context, anyhow};
use benchmark_runtime::*;
use bundle_bootstrap_runtime::*;
use bundle_config_runtime::*;
use bundle_lane_runtime::*;
pub(crate) use bundle_memory_surface::{
    MemoryObjectLane, bundle_compiled_memory_dir, bundle_compiled_memory_item_path,
    bundle_compiled_memory_path, memory_object_item_slug, memory_object_lane_item_key,
    read_memory_surface, render_bundle_memory_object_item_markdown,
    render_bundle_memory_object_markdown, render_current_task_bundle_snapshot,
    render_memory_surface_summary, short_hash_text,
};
pub(crate) use bundle_maintenance_runtime::*;
use bundle_models::*;
use bundle_profile_runtime::*;
use bundle_runtime::*;
pub(crate) use bundle_runtime::{
    BundleAuthorityPolicy, BundleAuthorityState, BundleBackendConfig, BundleConfig,
    BundleHeartbeatState, BundleHooksConfig, BundleRagConfig, BundleRuntimeConfig,
    CapabilitiesResponse, CapabilityHarnessSummary, ClaimsResponse,
    CoordinationRecoverySummary, CoordinationResponse, CoordinationSuggestion,
    MemorySurfaceResponse, MessagesResponse, ProjectAwarenessEntry, ProjectAwarenessResponse,
    SessionClaim, SessionClaimsState, SessionResponse, TasksResponse, bundle_heartbeat_state_path,
    derive_awareness_worker_name, detect_host_name, heartbeat_presence_label,
    project_awareness_entry_to_hive_session, read_bundle_claims, read_bundle_heartbeat,
};
use chrono::{DateTime, Utc};
use clap::{Args, CommandFactory, Parser, Subcommand};
use commands::{
    normalize_voice_mode_value, parse_entity_relation_kind, parse_memory_kind_value,
    parse_memory_scope_value, parse_memory_visibility_value, parse_retrieval_intent,
    parse_retrieval_route, parse_source_quality_value, parse_uuid_list,
};
pub(crate) use compiled_event::*;
pub(crate) use compiled_memory::*;
use coordination_control::*;
use coordination_runtime::*;
use coordination_views::*;
pub(crate) use evaluation_runtime::{
    append_experiment_learning_notes, append_text_to_memory_surface, build_gap_candidates,
    copy_dir_contents, derive_experiment_learnings, eval_bundle_memory, eval_failure_reason,
    eval_score_delta, evaluate_gap_changes, persist_loop_record, prioritize_gap_candidates,
    read_latest_bundle_eval, read_latest_gap_report, read_latest_scenario_report,
    read_loop_summary, render_bundle_eval_markdown, research_loops_doc_loop_count,
    restore_bundle_snapshot, simplify_awareness_work_text, write_gap_artifacts,
    write_public_benchmark_run_artifacts,
};
pub(crate) use evolution_runtime::*;
pub(crate) use bundle_init_runtime::*;
pub(crate) use autoresearch_runtime::*;
pub(crate) use improvement_runtime::*;
#[allow(unused_imports)]
pub(crate) use obsidian_support::*;
pub(crate) use public_benchmark_runtime::*;
pub(crate) use verification_runtime::*;
pub(crate) use workspace_runtime::*;
use hive_commands_runtime::*;
use hive_ops_runtime::*;
use hive_runtime::*;
pub(crate) use ingest_runtime::*;
pub(crate) use inspiration_search::*;
use memd_client::MemdClient;
use memd_core::{
    BuildCompactionPacketArgs, build_compaction_packet, derive_compaction_spill,
    derive_compaction_spill_with_options, render_compaction_wire,
};
use memd_rag::{
    RagClient, RagIngestRequest, RagIngestSource, RagRetrieveMode, RagRetrieveRequest,
    RagRetrieveResponse,
};
use memd_schema::{
    AgentProfileRequest, AgentProfileUpsertRequest, AssociativeRecallRequest,
    BenchmarkEvidenceSummary, BenchmarkGateDecision, BenchmarkRegistry, BenchmarkSubjectMetrics,
    CandidateMemoryRequest, CompactionDecision, CompactionOpenLoop, CompactionPacket,
    CompactionReference, CompactionSession, CompactionSpillOptions, CompactionSpillResult,
    ContextRequest, ContinuityJourneyReport, EntityLinkRequest, EntityLinksRequest,
    EntitySearchRequest, ExpireMemoryRequest, ExplainMemoryRequest, FixtureRecord,
    HiveBoardRequest, HiveBoardResponse, HiveClaimRecoverRequest, HiveClaimsRequest,
    HiveCoordinationInboxRequest, HiveCoordinationInboxResponse, HiveCoordinationReceiptRecord,
    HiveCoordinationReceiptRequest, HiveCoordinationReceiptsRequest, HiveFollowRequest,
    HiveFollowResponse, HiveHandoffPacket, HiveMessageAckRequest, HiveMessageInboxRequest,
    HiveMessageRecord, HiveMessageSendRequest, HiveRosterRequest, HiveRosterResponse,
    HiveSessionAutoRetireRequest, HiveTaskAssignRequest, HiveTaskRecord, HiveTaskUpsertRequest,
    HiveTasksRequest, MaintainReport, MemoryConsolidationRequest, MemoryInboxRequest, MemoryKind,
    MemoryMaintenanceReportRequest, MemoryPolicyResponse, MemoryRepairMode, MemoryScope,
    MemoryStage, MemoryStatus, PromoteMemoryRequest, RepairMemoryRequest, RetrievalIntent,
    RetrievalRoute, SearchMemoryRequest, SkillPolicyActivationEntriesRequest,
    SkillPolicyActivationEntriesResponse, SkillPolicyActivationRecord,
    SkillPolicyApplyReceiptsRequest, SkillPolicyApplyReceiptsResponse, SkillPolicyApplyRequest,
    SourceMemoryRequest, StoreMemoryRequest, VerifierRecord, VerifyMemoryRequest,
    WorkingMemoryRequest,
};
use memd_sidecar::SidecarClient;
use notify::{Config as NotifyConfig, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use obsidian::{ObsidianImportPreview, ObsidianSyncEntry};
use obsidian_commands::{
    run_obsidian_compile, run_obsidian_import, run_obsidian_open, run_obsidian_status,
    run_obsidian_watch,
};
use obsidian_runtime::{run_obsidian_handoff, run_obsidian_writeback};
use render::{
    is_default_runtime, render_bundle_status_summary, render_command_catalog_json,
    render_command_catalog_markdown, render_command_catalog_summary, render_composite_markdown,
    render_composite_summary, render_consolidate_summary, render_entity_search_summary,
    render_entity_summary, render_eval_summary, render_experiment_markdown,
    render_experiment_summary, render_explain_summary, render_feature_benchmark_markdown,
    render_feature_benchmark_summary, render_gap_summary, render_handoff_prompt,
    render_harness_pack_index_json, render_harness_pack_index_markdown,
    render_harness_pack_index_summary, render_improvement_markdown, render_improvement_summary,
    render_maintenance_report_summary, render_obsidian_import_summary,
    render_obsidian_scan_summary, render_policy_summary, render_profile_summary,
    render_recall_summary, render_repair_summary, render_resume_prompt, render_scenario_markdown,
    render_scenario_summary, render_skill_catalog_markdown, render_skill_catalog_match_markdown,
    render_skill_catalog_match_summary, render_skill_catalog_summary, render_skill_policy_summary,
    render_source_summary, render_timeline_summary, render_visible_memory_artifact_detail,
    render_visible_memory_home, render_visible_memory_knowledge_map, render_working_summary,
    render_workspace_summary, short_uuid,
};
use retrieval_runtime::*;
use runtime_checkpoint::*;
use runtime_resume::*;
pub(crate) use runtime_resume::{
    BundleResumeState, HandoffSnapshot, ResumeSnapshot, TruthRecordSummary, TruthSummary,
    build_truth_summary, read_bundle_resume_state, write_bundle_resume_state,
};
use scenario_runtime::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use serde_json::json;
use sha2::{Digest, Sha256};
pub(crate) use skill_catalog::*;
use tempfile::TempDir;
use tokio::task::JoinSet;
use verify_runtime::*;

#[derive(Debug, Parser)]
#[command(name = "memd")]
#[command(about = "Compact CLI for memd")]
struct Cli {
    #[arg(long, default_value_t = default_base_url())]
    base_url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
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
struct EventsArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    root: PathBuf,

    #[arg(long)]
    query: Option<String>,

    #[arg(long)]
    open: Option<String>,

    #[arg(long)]
    list: bool,

    #[arg(long, default_value_t = 12)]
    limit: usize,

    #[arg(long)]
    summary: bool,
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
struct MaintainArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,

    #[arg(long, default_value = "scan")]
    mode: String,

    #[arg(long)]
    apply: bool,

    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct PolicyArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,

    #[arg(long)]
    project: Option<String>,

    #[arg(long)]
    namespace: Option<String>,

    #[arg(long)]
    workspace: Option<String>,

    #[arg(long)]
    limit: Option<usize>,

    #[arg(long)]
    summary: bool,

    #[arg(long)]
    follow: bool,

    #[arg(long, help = "Query stored skill-policy receipts and activations")]
    query: bool,

    #[arg(long, help = "Write skill-policy batch artifacts to bundle state")]
    write: bool,

    #[arg(
        long,
        help = "Write the activate queue artifact for downstream apply flows"
    )]
    apply: bool,
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
struct UiArgs {
    #[command(subcommand)]
    mode: UiMode,
}

#[derive(Debug, Clone, Subcommand)]
enum UiMode {
    Home(UiHomeArgs),
    Artifact(UiArtifactArgs),
    Map(UiMapArgs),
}

#[derive(Debug, Clone, Args)]
struct UiHomeArgs {
    #[arg(long)]
    json: bool,

    #[arg(long)]
    follow: bool,
}

#[derive(Debug, Clone, Args)]
struct UiArtifactArgs {
    #[arg(long)]
    id: String,

    #[arg(long)]
    json: bool,

    #[arg(long)]
    follow: bool,
}

#[derive(Debug, Clone, Args)]
struct UiMapArgs {
    #[arg(long)]
    json: bool,

    #[arg(long)]
    follow: bool,
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
struct LookupArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,

    #[arg(long)]
    query: String,

    #[arg(long)]
    project: Option<String>,

    #[arg(long)]
    namespace: Option<String>,

    #[arg(long)]
    workspace: Option<String>,

    #[arg(long)]
    visibility: Option<String>,

    #[arg(long)]
    route: Option<String>,

    #[arg(long)]
    intent: Option<String>,

    #[arg(long, value_name = "KIND")]
    kind: Vec<String>,

    #[arg(long, value_name = "TEXT")]
    tag: Vec<String>,

    #[arg(long)]
    include_stale: bool,

    #[arg(long)]
    limit: Option<usize>,

    #[arg(long)]
    verbose: bool,

    #[arg(long)]
    json: bool,
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
struct InspirationArgs {
    #[arg(long)]
    query: String,

    #[arg(long, default_value_t = 10)]
    limit: usize,

    #[arg(long)]
    root: Option<PathBuf>,

    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct SkillsArgs {
    #[arg(long)]
    root: Option<PathBuf>,

    #[arg(long)]
    query: Option<String>,

    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct PacksArgs {
    #[arg(long)]
    root: Option<PathBuf>,

    #[arg(long)]
    query: Option<String>,

    #[arg(long)]
    summary: bool,

    #[arg(long)]
    json: bool,
}

#[derive(Debug, Clone, Args)]
struct CommandCatalogArgs {
    #[arg(long)]
    root: Option<PathBuf>,

    #[arg(long)]
    query: Option<String>,

    #[arg(long)]
    summary: bool,

    #[arg(long)]
    json: bool,
}

#[derive(Debug, Clone, Args)]
struct SetupArgs {
    #[arg(long)]
    project: Option<String>,

    #[arg(long)]
    namespace: Option<String>,

    #[arg(long, default_value_t = false)]
    global: bool,

    #[arg(long)]
    project_root: Option<PathBuf>,

    #[arg(long, default_value_t = true)]
    seed_existing: bool,

    #[arg(long)]
    agent: Option<String>,

    #[arg(long)]
    session: Option<String>,

    #[arg(long)]
    tab_id: Option<String>,

    #[arg(long)]
    hive_system: Option<String>,

    #[arg(long)]
    hive_role: Option<String>,

    #[arg(long, value_name = "TEXT")]
    capability: Vec<String>,

    #[arg(long, value_name = "TEXT")]
    hive_group: Vec<String>,

    #[arg(long)]
    hive_group_goal: Option<String>,

    #[arg(long)]
    authority: Option<String>,

    #[arg(long)]
    output: Option<PathBuf>,

    #[arg(long)]
    base_url: Option<String>,

    #[arg(long)]
    rag_url: Option<String>,

    #[arg(long)]
    route: Option<String>,

    #[arg(long)]
    intent: Option<String>,

    #[arg(long)]
    workspace: Option<String>,

    #[arg(long)]
    visibility: Option<String>,

    #[arg(long)]
    voice_mode: Option<String>,

    #[arg(long)]
    force: bool,

    #[arg(long, default_value_t = false)]
    allow_localhost_read_only_fallback: bool,

    #[arg(long)]
    summary: bool,

    #[arg(long)]
    json: bool,
}

#[derive(Debug, Clone, Args)]
struct DoctorArgs {
    #[arg(long)]
    output: Option<PathBuf>,

    #[arg(long)]
    project_root: Option<PathBuf>,

    #[arg(long)]
    repair: bool,

    #[arg(long)]
    summary: bool,

    #[arg(long)]
    json: bool,
}

#[derive(Debug, Clone, Args)]
struct ConfigArgs {
    #[arg(long)]
    output: Option<PathBuf>,

    #[arg(long)]
    project_root: Option<PathBuf>,

    #[arg(long)]
    summary: bool,

    #[arg(long)]
    json: bool,
}

#[derive(Debug, Clone, Args)]
struct MemoryArgs {
    #[arg(long)]
    root: Option<PathBuf>,

    #[arg(long)]
    query: Option<String>,

    #[arg(long)]
    open: Option<String>,

    #[arg(long)]
    lane: Option<String>,

    #[arg(long)]
    item: Option<String>,

    #[arg(long)]
    list: bool,

    #[arg(long)]
    lanes_only: bool,

    #[arg(long)]
    items_only: bool,

    #[arg(long)]
    filter: Option<String>,

    #[arg(long)]
    grouped: bool,

    #[arg(long)]
    expand_items: bool,

    #[arg(long)]
    json: bool,

    #[arg(long, default_value_t = 12)]
    limit: usize,

    #[arg(long)]
    summary: bool,

    #[arg(long)]
    quality: bool,
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
    Capture(HookCaptureArgs),
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
struct HookCaptureArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
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

    #[arg(long)]
    content: Option<String>,

    #[arg(long)]
    input: Option<PathBuf>,

    #[arg(long)]
    stdin: bool,

    #[arg(long, value_name = "TEXT")]
    tag: Vec<String>,

    #[arg(long)]
    promote_kind: Option<String>,

    #[arg(long)]
    promote_scope: Option<String>,

    #[arg(long, value_name = "UUID")]
    promote_supersede: Vec<String>,

    #[arg(long)]
    promote_supersede_query: Option<String>,

    #[arg(long, value_name = "TEXT")]
    promote_tag: Vec<String>,

    #[arg(long)]
    promote_confidence: Option<f32>,

    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct InitArgs {
    #[arg(long)]
    project: Option<String>,

    #[arg(long)]
    namespace: Option<String>,

    #[arg(long, default_value_t = false)]
    global: bool,

    #[arg(long)]
    project_root: Option<PathBuf>,

    #[arg(long, default_value_t = true)]
    seed_existing: bool,

    #[arg(long)]
    agent: String,

    #[arg(long)]
    session: Option<String>,

    #[arg(long)]
    tab_id: Option<String>,

    #[arg(long)]
    hive_system: Option<String>,

    #[arg(long)]
    hive_role: Option<String>,

    #[arg(long, value_name = "TEXT")]
    capability: Vec<String>,

    #[arg(long, value_name = "TEXT")]
    hive_group: Vec<String>,

    #[arg(long)]
    hive_group_goal: Option<String>,

    #[arg(long)]
    authority: Option<String>,

    #[arg(long, default_value_os_t = default_init_output_path())]
    output: PathBuf,

    #[arg(long, default_value_t = default_base_url())]
    base_url: String,

    #[arg(long)]
    rag_url: Option<String>,

    #[arg(long, default_value = "auto")]
    route: String,

    #[arg(long, default_value = "current_task")]
    intent: String,

    #[arg(long)]
    workspace: Option<String>,

    #[arg(long)]
    visibility: Option<String>,

    #[arg(long)]
    voice_mode: Option<String>,

    #[arg(long)]
    force: bool,

    #[arg(long, default_value_t = false)]
    allow_localhost_read_only_fallback: bool,
}

#[derive(Debug, Clone, Args)]
struct LoopsArgs {
    #[arg(long, default_value_os_t = default_init_output_path())]
    output: PathBuf,

    #[arg(
        long = "loop",
        value_name = "SLUG",
        help = "Show details for a recorded loop slug"
    )]
    loop_slug: Option<String>,

    #[arg(long, help = "Show aggregate loop metrics and improvements")]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct TelemetryArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,

    #[arg(long, help = "Emit telemetry JSON instead of text")]
    json: bool,
}

#[derive(Debug, Clone, Args)]
struct AutoresearchArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,

    #[arg(long, help = "Run all manifest loops")]
    auto: bool,

    #[arg(long, help = "Run a single loop by slug")]
    loop_slug: Option<String>,

    #[arg(long, help = "Print the manifest of available autoresearch loops")]
    manifest: bool,

    #[arg(
        long,
        default_value_t = 1,
        help = "Maximum number of autoresearch sweeps to run"
    )]
    max_sweeps: usize,

    #[arg(
        long,
        default_value_t = 0,
        help = "Stop after this many consecutive identical sweep signatures (0 disables plateau stopping)"
    )]
    plateau_sweeps: usize,
}

#[derive(Debug, Clone, Args)]
struct StatusArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,

    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct CapabilitiesArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,

    #[arg(long)]
    harness: Option<String>,

    #[arg(long)]
    kind: Option<String>,

    #[arg(long)]
    portability: Option<String>,

    #[arg(long)]
    query: Option<String>,

    #[arg(long, default_value_t = 12)]
    limit: usize,

    #[arg(long)]
    summary: bool,

    #[arg(long)]
    json: bool,
}

#[derive(Debug, Clone, Args)]
struct AwarenessArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
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
    #[arg(long, default_value_os_t = default_bundle_root_path())]
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
struct SessionArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,

    #[arg(long)]
    rebind: bool,

    #[arg(long)]
    reconcile: bool,

    #[arg(long)]
    retire_session: Option<String>,

    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct ClaimsArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
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
    #[arg(long, default_value_os_t = default_bundle_root_path())]
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
    #[arg(long, default_value_os_t = default_bundle_root_path())]
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
    view: Option<String>,

    #[arg(long)]
    summary: bool,

    #[arg(long)]
    json: bool,
}

#[derive(Debug, Clone, Args)]
struct CoordinationArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
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
    retire_session: Option<String>,

    #[arg(long)]
    to_session: Option<String>,

    #[arg(long)]
    deny_session: Option<String>,

    #[arg(long)]
    reroute_session: Option<String>,

    #[arg(long)]
    handoff_scope: Option<String>,

    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct BundleArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,

    #[arg(long)]
    hive_system: Option<String>,

    #[arg(long)]
    hive_role: Option<String>,

    #[arg(long, value_name = "TEXT")]
    capability: Vec<String>,

    #[arg(long, value_name = "TEXT")]
    hive_group: Vec<String>,

    #[arg(long)]
    hive_group_goal: Option<String>,

    #[arg(long)]
    authority: Option<String>,

    #[arg(long)]
    base_url: Option<String>,

    #[arg(long)]
    route: Option<String>,

    #[arg(long)]
    intent: Option<String>,

    #[arg(long)]
    tab_id: Option<String>,

    #[arg(long)]
    auto_short_term_capture: Option<bool>,

    #[arg(long)]
    voice_mode: Option<String>,

    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct HiveArgs {
    #[command(subcommand)]
    command: Option<HiveSubcommand>,

    #[arg(long)]
    agent: Option<String>,

    #[arg(long)]
    project: Option<String>,

    #[arg(long)]
    namespace: Option<String>,

    #[arg(long, default_value_t = false)]
    global: bool,

    #[arg(long)]
    project_root: Option<PathBuf>,

    #[arg(long, default_value_t = true)]
    seed_existing: bool,

    #[arg(long)]
    session: Option<String>,

    #[arg(long)]
    tab_id: Option<String>,

    #[arg(long)]
    hive_system: Option<String>,

    #[arg(long)]
    hive_role: Option<String>,

    #[arg(long, value_name = "TEXT")]
    capability: Vec<String>,

    #[arg(long, value_name = "TEXT")]
    hive_group: Vec<String>,

    #[arg(long)]
    hive_group_goal: Option<String>,

    #[arg(long)]
    authority: Option<String>,

    #[arg(long, default_value_os_t = default_init_output_path())]
    output: PathBuf,

    #[arg(long, default_value_t = default_base_url())]
    base_url: String,

    #[arg(long)]
    rag_url: Option<String>,

    #[arg(long, default_value = "auto")]
    route: String,

    #[arg(long, default_value = "current_task")]
    intent: String,

    #[arg(long)]
    workspace: Option<String>,

    #[arg(long)]
    visibility: Option<String>,

    #[arg(long, default_value_t = true)]
    publish_heartbeat: bool,

    #[arg(long)]
    force: bool,

    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Clone, Subcommand)]
enum HiveSubcommand {
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
enum HiveCoworkSubcommand {
    Request(HiveCoworkArgs),
    Ack(HiveCoworkArgs),
    Decline(HiveCoworkArgs),
}

#[derive(Debug, Clone, Args)]
struct HiveRosterArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,

    #[arg(long)]
    json: bool,

    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct HiveFollowArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,

    #[arg(long)]
    session: Option<String>,

    #[arg(long)]
    worker: Option<String>,

    #[arg(long)]
    watch: bool,

    #[arg(long, default_value_t = 5)]
    interval_secs: u64,

    #[arg(long)]
    json: bool,

    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct HiveHandoffArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,

    #[arg(long)]
    to_session: Option<String>,

    #[arg(long)]
    to_worker: Option<String>,

    #[arg(long)]
    task_id: Option<String>,

    #[arg(long)]
    topic: Option<String>,

    #[arg(long, value_delimiter = ',')]
    scope: Vec<String>,

    #[arg(long)]
    next_action: Option<String>,

    #[arg(long)]
    blocker: Option<String>,

    #[arg(long)]
    note: Option<String>,

    #[arg(long)]
    json: bool,

    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct HiveCoworkArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,

    #[arg(long)]
    to_session: Option<String>,

    #[arg(long)]
    to_worker: Option<String>,

    #[arg(long)]
    task_id: Option<String>,

    #[arg(long, value_delimiter = ',')]
    scope: Vec<String>,

    #[arg(long)]
    reason: Option<String>,

    #[arg(long)]
    note: Option<String>,

    #[arg(long)]
    json: bool,

    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct HiveQueenArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,

    #[arg(long)]
    view: Option<String>,

    #[arg(long)]
    recover_session: Option<String>,

    #[arg(long)]
    retire_session: Option<String>,

    #[arg(long)]
    to_session: Option<String>,

    #[arg(long)]
    deny_session: Option<String>,

    #[arg(long)]
    reroute_session: Option<String>,

    #[arg(long)]
    handoff_scope: Option<String>,

    #[arg(long)]
    json: bool,

    #[arg(long)]
    summary: bool,

    #[arg(long)]
    cowork_auto_send: bool,
}

#[derive(Debug, Clone, Args)]
struct HiveJoinArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,

    #[arg(long, default_value_t = default_hive_join_base_url())]
    base_url: String,

    #[arg(long)]
    all_active: bool,

    #[arg(long)]
    all_local: bool,

    #[arg(long, default_value_t = true)]
    publish_heartbeat: bool,

    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct HiveProjectArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,

    #[arg(long)]
    enable: bool,

    #[arg(long)]
    disable: bool,

    #[arg(long)]
    status: bool,

    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct EvalArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
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
    #[arg(long, default_value_os_t = default_bundle_root_path())]
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
    #[arg(long, default_value_os_t = default_bundle_root_path())]
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
struct ScenarioArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,

    #[arg(long)]
    scenario: Option<String>,

    #[arg(long, default_value_t = false)]
    write: bool,

    #[arg(long, default_value_t = false)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct CompositeArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,

    #[arg(long)]
    scenario: Option<String>,

    #[arg(long, default_value_t = false)]
    write: bool,

    #[arg(long, default_value_t = false)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct BenchmarkArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,

    #[arg(long, default_value_t = false)]
    write: bool,

    #[arg(long, default_value_t = false)]
    summary: bool,

    #[command(subcommand)]
    subcommand: Option<BenchmarkSubcommand>,
}

#[derive(Debug, Clone, Subcommand)]
enum BenchmarkSubcommand {
    Public(PublicBenchmarkArgs),
}

#[derive(Debug, Clone, Args)]
struct PublicBenchmarkArgs {
    dataset: String,

    #[arg(long, value_parser = ["raw", "hybrid"])]
    mode: Option<String>,

    #[arg(long, value_parser = ["lexical", "sidecar"])]
    retrieval_backend: Option<String>,

    #[arg(long)]
    rag_url: Option<String>,

    #[arg(long)]
    top_k: Option<usize>,

    #[arg(long)]
    limit: Option<usize>,

    #[arg(long)]
    dataset_root: Option<PathBuf>,

    #[arg(long)]
    reranker: Option<String>,

    #[arg(long, default_value_t = false)]
    write: bool,

    #[arg(long, default_value_t = false)]
    json: bool,

    #[arg(long, alias = "output", default_value_os_t = default_bundle_root_path())]
    out: PathBuf,
}

#[derive(Debug, Clone, Args)]
struct VerifyArgs {
    #[command(subcommand)]
    command: VerifyCommand,
}

#[derive(Debug, Clone, Subcommand)]
enum VerifyCommand {
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
struct VerifyFeatureArgs {
    feature_id: String,

    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,

    #[arg(long, default_value_t = false)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct VerifyJourneyArgs {
    journey_id: String,

    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,

    #[arg(long, default_value_t = false)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct VerifyAdversarialArgs {
    verifier_id: String,

    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,

    #[arg(long, default_value_t = false)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct VerifyCompareArgs {
    verifier_id: String,

    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,

    #[arg(long, default_value_t = false)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct VerifySweepArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,

    #[arg(long, default_value = "fast")]
    lane: String,

    #[arg(long, default_value_t = false)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct VerifyDoctorArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,

    #[arg(long, default_value_t = false)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct VerifyListArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,

    #[arg(long)]
    lane: Option<String>,

    #[arg(long, default_value_t = false)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct VerifyShowArgs {
    item_id: String,

    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,

    #[arg(long, default_value_t = false)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct ExperimentArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,

    #[arg(long, default_value_t = 2)]
    max_iterations: usize,

    #[arg(long)]
    limit: Option<usize>,

    #[arg(long)]
    recent_commits: Option<usize>,

    #[arg(long, default_value_t = 80)]
    accept_below: u8,

    #[arg(long, default_value_t = true)]
    apply: bool,

    #[arg(long, default_value_t = true)]
    consolidate: bool,

    #[arg(long, default_value_t = false)]
    write: bool,

    #[arg(long, default_value_t = false)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct AttachArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,

    #[arg(long)]
    shell: Option<String>,
}

#[derive(Debug, Clone, Args)]
struct AgentArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
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
    #[arg(long, default_value_os_t = default_bundle_root_path())]
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
struct WatchArgs {
    #[arg(long, default_value_os_t = std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))]
    root: PathBuf,

    #[arg(long, default_value_os_t = default_bundle_root_path())]
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

    #[arg(long, default_value_t = 750)]
    debounce_ms: u64,
}

#[derive(Debug, Clone, Args)]
struct WakeArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
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
    verbose: bool,

    #[arg(long)]
    write: bool,

    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Clone, Args)]
struct HandoffArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
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
    #[arg(long, default_value_os_t = default_bundle_root_path())]
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

    #[arg(long, value_name = "UUID")]
    supersede: Vec<String>,

    #[arg(long)]
    content: Option<String>,

    #[arg(long)]
    input: Option<PathBuf>,

    #[arg(long)]
    stdin: bool,
}

#[derive(Debug, Clone, Args)]
struct CheckpointArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
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

#[derive(Debug, Clone, Args)]
struct RequestInput {
    #[arg(long)]
    json: Option<String>,

    #[arg(long)]
    input: Option<PathBuf>,

    #[arg(long)]
    stdin: bool,
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

fn obsidian_path_is_internal(path: &Path) -> bool {
    path.components().any(|component| {
        matches!(
            component,
            std::path::Component::Normal(name)
                if name == ".memd" || name == ".obsidian" || name == ".git"
        )
    })
}

fn workspace_path_is_internal(path: &Path) -> bool {
    path.components().any(|component| {
        matches!(
            component,
            std::path::Component::Normal(name)
                if name == ".memd"
                    || name == ".git"
                    || name == "target"
                    || name == "node_modules"
                    || name == "watch.out"
                    || name == "memd-watch.log"
                    || name == "memd-watch.err"
        )
    })
}

fn workspace_path_should_trigger(path: &Path) -> bool {
    if workspace_path_is_internal(path) {
        return false;
    }

    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    if matches!(
        file_name,
        "Cargo.toml"
            | "Cargo.lock"
            | "Makefile"
            | "Dockerfile"
            | "README"
            | "README.md"
            | "AGENTS.md"
            | "CLAUDE.md"
            | "ROADMAP.md"
            | "DESIGN.md"
            | "CONTRIBUTING.md"
            | "CHANGELOG.md"
    ) {
        return true;
    }

    match path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
    {
        Some(ext)
            if matches!(
                ext.as_str(),
                "rs" | "toml"
                    | "md"
                    | "sh"
                    | "ps1"
                    | "json"
                    | "yml"
                    | "yaml"
                    | "js"
                    | "ts"
                    | "tsx"
                    | "py"
                    | "go"
                    | "c"
                    | "h"
                    | "cpp"
                    | "css"
                    | "html"
                    | "txt"
                    | "lock"
            ) =>
        {
            true
        }
        _ => false,
    }
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

fn resolve_pack_bundle_root(explicit: Option<&Path>) -> anyhow::Result<PathBuf> {
    if let Some(explicit) = explicit {
        return Ok(explicit.to_path_buf());
    }

    Ok(resolve_default_bundle_root()?.unwrap_or_else(default_bundle_root_path))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cli_runtime::run_cli(Cli::parse()).await
}
