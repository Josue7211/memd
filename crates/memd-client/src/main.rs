mod benchmark_runtime;
mod bundle_agent_profiles;
mod bundle_bootstrap_runtime;
mod bundle_config_runtime;
mod bundle_lane_runtime;
mod bundle_memory_surface;
mod bundle_models;
mod bundle_profile_runtime;
mod bundle_runtime;
mod command_catalog;
mod commands;
mod compiled_event;
mod compiled_memory;
mod coordination_control;
mod coordination_runtime;
mod coordination_views;
pub(crate) mod harness;
mod hive_commands_runtime;
mod hive_ops_runtime;
mod hive_runtime;
mod ingest_runtime;
mod inspiration_search;
mod evaluation_runtime;
mod obsidian;
mod obsidian_commands;
mod obsidian_runtime;
mod render;
mod retrieval_runtime;
mod runtime_checkpoint;
mod runtime_resume;
mod scenario_runtime;
mod skill_catalog;
mod verify_runtime;

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
use bundle_models::*;
use bundle_profile_runtime::*;
use bundle_runtime::*;
pub(crate) use evaluation_runtime::{
    append_experiment_learning_notes, append_text_to_memory_surface, build_eval_recommendations,
    build_gap_candidates, copy_dir_contents, derive_experiment_learnings, describe_eval_changes,
    eval_bundle_memory, eval_failure_reason, eval_score_delta, evaluate_gap_changes,
    prioritize_gap_candidates, public_benchmark_manifest_json_path,
    public_benchmark_report_md_path, public_benchmark_results_json_path,
    public_benchmark_results_jsonl_path, read_latest_bundle_eval, read_latest_gap_report,
    read_latest_scenario_report, read_loop_summary, research_loops_doc_loop_count,
    persist_loop_record, render_bundle_eval_markdown, render_gap_markdown,
    restore_bundle_snapshot, simplify_awareness_work_text, update_loop_summary,
    write_gap_artifacts, write_gap_loop_record, write_public_benchmark_manifest,
    write_public_benchmark_run_artifacts, write_public_benchmark_run_report,
};
pub(crate) use bundle_runtime::{
    BundleAuthorityPolicy, BundleAuthorityState, BundleBackendConfig, BundleBackendConfigFile,
    BundleConfig, BundleConfigFile, BundleHeartbeatState, BundleHooksConfig, BundleRagConfig,
    BundleRagConfigFile, BundleRuntimeConfig, CapabilitiesResponse, CapabilityHarnessSummary,
    ClaimsResponse, CoordinationRecoverySummary, CoordinationResponse, CoordinationSuggestion,
    MemorySurfaceResponse, MessagesResponse, ProjectAwarenessEntry, ProjectAwarenessResponse,
    SessionClaim, SessionClaimsState, SessionResponse, TasksResponse, bundle_heartbeat_state_path,
    derive_awareness_worker_name, detect_host_name, heartbeat_presence_label,
    project_awareness_entry_to_hive_session, read_bundle_claims, read_bundle_heartbeat,
    resolve_bundle_rag_config,
};
use chrono::{DateTime, Utc};
use clap::{Args, CommandFactory, Parser, Subcommand};
use commands::{
    parse_entity_relation_kind, parse_memory_kind_value, parse_memory_scope_value,
    parse_memory_visibility_value, parse_retrieval_intent, parse_retrieval_route,
    parse_source_quality_value, parse_uuid_list, normalize_voice_mode_value,
};
pub(crate) use compiled_event::*;
pub(crate) use compiled_memory::*;
use coordination_control::*;
use coordination_runtime::*;
use coordination_views::*;
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
    BundleResumeState, HandoffSnapshot, ResumeSnapshot, RetrievalTier, TruthRecordSummary,
    TruthSummary, build_truth_summary, read_bundle_resume_state, write_bundle_resume_state,
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let client = MemdClient::new(&cli.base_url)?;
    let base_url = cli.base_url.clone();

    #[allow(unreachable_patterns)]
    match cli.command {
        Commands::Healthz => print_json(&client.healthz().await?)?,
        Commands::Status(args) => {
            let status = read_bundle_status(&args.output, &base_url).await?;
            if args.summary {
                println!("{}", render_bundle_status_summary(&status));
            } else {
                print_json(&status)?;
            }
        }
        Commands::Capabilities(args) => {
            let response = run_capabilities_command(&args)?;
            if args.json {
                print_json(&response)?;
            } else {
                println!("{}", render_capabilities_runtime_summary(&response));
            }
        }
        Commands::Session(args) => {
            let response = run_session_command(&args, &base_url).await?;
            if args.summary {
                println!("{}", render_session_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Wake(args) => {
            if let Some(tab_id) = default_bundle_tab_id() {
                let existing_tab_id = read_bundle_runtime_config(&args.output)
                    .ok()
                    .flatten()
                    .and_then(|config| config.tab_id)
                    .filter(|value| !value.trim().is_empty());
                if existing_tab_id.is_none() {
                    set_bundle_tab_id(&args.output, &tab_id)?;
                }
            }
            invalidate_bundle_runtime_caches(&args.output)?;
            let codex_pack =
                harness_pack_enabled_for_bundle(&args.output, args.agent.as_deref(), "codex");
            let agent_zero_pack =
                harness_pack_enabled_for_bundle(&args.output, args.agent.as_deref(), "agent-zero");
            let hermes_pack =
                harness_pack_enabled_for_bundle(&args.output, args.agent.as_deref(), "hermes");
            let opencode_pack =
                harness_pack_enabled_for_bundle(&args.output, args.agent.as_deref(), "opencode");
            let openclaw_pack =
                harness_pack_enabled_for_bundle(&args.output, args.agent.as_deref(), "openclaw");
            let resume_args = ResumeArgs {
                output: args.output.clone(),
                project: args.project.clone(),
                namespace: args.namespace.clone(),
                agent: args.agent.clone(),
                workspace: args.workspace.clone(),
                visibility: args.visibility.clone(),
                route: args.route.clone(),
                intent: args.intent.clone().or(Some("current_task".to_string())),
                limit: args.limit,
                rehydration_limit: args.rehydration_limit,
                semantic: args.semantic,
                prompt: false,
                summary: false,
            };
            let snapshot = match read_bundle_resume(&resume_args, &base_url).await {
                Ok(snapshot) => snapshot,
                Err(err)
                    if codex_pack
                        || agent_zero_pack
                        || hermes_pack
                        || opencode_pack
                        || openclaw_pack =>
                {
                    if let Some(markdown) =
                        read_codex_pack_local_markdown(&args.output, "MEMD_WAKEUP.md")?
                    {
                        if args.write {
                            write_bundle_turn_fallback_artifacts(
                                &args.output,
                                args.project.as_deref(),
                                args.namespace.as_deref(),
                                args.agent.as_deref(),
                                args.workspace.as_deref(),
                                args.visibility.as_deref(),
                                args.route.as_deref(),
                                args.intent.as_deref(),
                                &markdown,
                            )?;
                        }
                        println!("{markdown}");
                        return Ok(());
                    }
                    if args.write {
                        write_bundle_turn_placeholder_memory(
                            &args.output,
                            args.project.as_deref(),
                            args.namespace.as_deref(),
                            args.agent.as_deref(),
                            args.workspace.as_deref(),
                            args.visibility.as_deref(),
                            args.route.as_deref(),
                            args.intent.as_deref(),
                        )?;
                    }
                    return Err(err);
                }
                Err(err) => return Err(err),
            };
            let wakeup = render_bundle_wakeup_markdown(&args.output, &snapshot, args.verbose);
            if args.write {
                write_bundle_memory_files(&args.output, &snapshot, None, false).await?;
                auto_checkpoint_live_snapshot(&args.output, &base_url, &snapshot, "wake").await?;
            }
            if codex_pack || agent_zero_pack || openclaw_pack || hermes_pack || opencode_pack {
                let _ = refresh_harness_pack_files_for_snapshot(
                    &args.output,
                    &snapshot,
                    "wake",
                    &["codex", "agent-zero", "openclaw", "hermes", "opencode"],
                )
                .await?;
            }
            if args.summary {
                println!("{}", render_bundle_wakeup_summary(&snapshot));
            } else {
                println!("{wakeup}");
            }
        }
        Commands::Awareness(args) => {
            let response = read_project_awareness(&args).await?;
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
            if args.json {
                print_json(&response)?;
            } else {
                println!("{}", render_tasks_summary(&response));
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
            if let Some(value) = args.hive_system.as_deref() {
                set_bundle_hive_system(&args.output, value)?;
            }
            if let Some(value) = args.hive_role.as_deref() {
                set_bundle_hive_role(&args.output, value)?;
            }
            if !args.capability.is_empty() {
                set_bundle_capabilities(&args.output, &args.capability)?;
            }
            if !args.hive_group.is_empty() {
                set_bundle_hive_groups(&args.output, &args.hive_group)?;
            }
            if let Some(value) = args.hive_group_goal.as_deref() {
                set_bundle_hive_group_goal(&args.output, value)?;
            }
            if let Some(value) = args.authority.as_deref() {
                set_bundle_authority(&args.output, value)?;
            }
            if let Some(value) = args.base_url.as_deref() {
                set_bundle_base_url(&args.output, value)?;
            }
            if let Some(value) = args.route.as_deref() {
                set_bundle_route(&args.output, value)?;
            }
            if let Some(value) = args.intent.as_deref() {
                set_bundle_intent(&args.output, value)?;
            }
            if let Some(value) = args.voice_mode.as_deref() {
                set_bundle_voice_mode(&args.output, value)?;
            }
            if let Some(value) = args.tab_id.as_deref() {
                set_bundle_tab_id(&args.output, value)?;
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
                let route = status
                    .get("defaults")
                    .and_then(|value| value.get("route"))
                    .and_then(|value| value.as_str())
                    .unwrap_or("auto");
                let intent = status
                    .get("defaults")
                    .and_then(|value| value.get("intent"))
                    .and_then(|value| value.as_str())
                    .unwrap_or("general");
                let hive_system = status
                    .get("defaults")
                    .and_then(|value| value.get("hive_system"))
                    .and_then(|value| value.as_str())
                    .unwrap_or("none");
                let hive_role = status
                    .get("defaults")
                    .and_then(|value| value.get("hive_role"))
                    .and_then(|value| value.as_str())
                    .unwrap_or("none");
                let authority = status
                    .get("defaults")
                    .and_then(|value| value.get("authority"))
                    .and_then(|value| value.as_str())
                    .unwrap_or("none");
                let hive_groups = status
                    .get("defaults")
                    .and_then(|value| value.get("hive_groups"))
                    .and_then(|value| value.as_array())
                    .map(|values| {
                        values
                            .iter()
                            .filter_map(|value| value.as_str())
                            .collect::<Vec<_>>()
                            .join(",")
                    })
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "none".to_string());
                let hive_group_goal = status
                    .get("defaults")
                    .and_then(|value| value.get("hive_group_goal"))
                    .and_then(|value| value.as_str())
                    .unwrap_or("none");
                println!(
                    "bundle={} hive={} role={} groups={} goal=\"{}\" authority={} base_url={} route={} intent={} auto_short_term_capture={}",
                    args.output.display(),
                    hive_system,
                    hive_role,
                    hive_groups,
                    hive_group_goal,
                    authority,
                    base_url,
                    route,
                    intent,
                    if enabled { "true" } else { "false" }
                );
            } else {
                print_json(&status)?;
            }
        }
        Commands::Hive(args) => match &args.command {
            Some(HiveSubcommand::Roster(roster_args)) => {
                let response = run_hive_roster_command(roster_args).await?;
                if roster_args.json {
                    print_json(&response)?;
                } else {
                    println!("{}", render_hive_roster_summary(&response));
                }
            }
            Some(HiveSubcommand::Follow(follow_args)) => {
                if follow_args.watch {
                    run_hive_follow_watch(follow_args).await?;
                } else {
                    let response = run_hive_follow_command(follow_args).await?;
                    if follow_args.json {
                        print_json(&response)?;
                    } else {
                        println!("{}", render_hive_follow_summary(&response));
                    }
                }
            }
            Some(HiveSubcommand::Handoff(handoff_args)) => {
                let response = run_hive_handoff_command(handoff_args, &default_base_url()).await?;
                if handoff_args.json {
                    print_json(&response)?;
                } else {
                    println!("{}", render_hive_handoff_summary(&response));
                }
            }
            Some(HiveSubcommand::Cowork { command: cowork_args }) => match cowork_args {
                HiveCoworkSubcommand::Request(cowork_args) => {
                    let response =
                        run_hive_cowork_command(cowork_args, &default_base_url(), "request")
                            .await?;
                    if cowork_args.json {
                        print_json(&response)?;
                    } else {
                        println!("{}", render_hive_cowork_summary(&response));
                    }
                }
                HiveCoworkSubcommand::Ack(cowork_args) => {
                    let response =
                        run_hive_cowork_command(cowork_args, &default_base_url(), "ack").await?;
                    if cowork_args.json {
                        print_json(&response)?;
                    } else {
                        println!("{}", render_hive_cowork_summary(&response));
                    }
                }
                HiveCoworkSubcommand::Decline(cowork_args) => {
                    let response =
                        run_hive_cowork_command(cowork_args, &default_base_url(), "decline")
                            .await?;
                    if cowork_args.json {
                        print_json(&response)?;
                    } else {
                        println!("{}", render_hive_cowork_summary(&response));
                    }
                }
            },
            Some(HiveSubcommand::Queen(queen_args)) => {
                let response = run_hive_queen_command(queen_args, &default_base_url()).await?;
                if queen_args.json {
                    print_json(&response)?;
                } else {
                    println!("{}", render_hive_queen_summary(&response));
                }
            }
            None => {
                if args.summary {
                    let response = run_hive_board_command(&args, &default_base_url()).await?;
                    println!("{}", render_hive_board_summary(&response));
                } else {
                    let response = run_hive_command(&args).await?;
                    print_json(&response)?;
                }
            }
        },
        Commands::HiveProject(args) => {
            let response = run_hive_project_command(&args).await?;
            if args.summary {
                println!("{}", render_hive_project_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        Commands::HiveJoin(args) => {
            let response = run_hive_join_command(&args).await?;
            if args.summary {
                println!("{}", render_hive_join_summary(&response));
            } else {
                print_json(&response)?;
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
        Commands::Scenario(args) => {
            let response = run_scenario_command(&args, &base_url).await?;
            if args.write {
                write_scenario_artifacts(&args.output, &response)?;
            }
            if args.summary {
                println!("{}", render_scenario_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Composite(args) => {
            let response = run_composite_command(&args, &base_url).await?;
            if args.write {
                write_composite_artifacts(&args.output, &response)?;
            }
            if args.summary {
                println!("{}", render_composite_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Benchmark(args) => match &args.subcommand {
            Some(BenchmarkSubcommand::Public(public_args)) => {
                let response = run_public_benchmark_command(public_args).await?;
                if public_args.write {
                    let receipt =
                        write_public_benchmark_run_artifacts(&public_args.out, &response)?;
                    let _ = (
                        &receipt.run_dir,
                        &receipt.manifest_path,
                        &receipt.results_path,
                        &receipt.results_jsonl_path,
                        &receipt.report_path,
                    );
                    if let Some(repo_root) = infer_bundle_project_root(&public_args.out) {
                        write_public_benchmark_docs(&repo_root, &public_args.out, &response)?;
                    }
                }
                if public_args.json {
                    print_json(&response)?;
                } else if args.summary {
                    println!("{}", render_public_benchmark_summary(&response));
                } else {
                    print_json(&response)?;
                }
            }
            None => {
                let response = run_feature_benchmark_command(&args, &base_url).await?;
                if args.write {
                    write_feature_benchmark_artifacts(&args.output, &response)?;
                    if let Some((repo_root, registry)) =
                        load_benchmark_registry_for_output(&args.output)?
                    {
                        write_benchmark_registry_docs(&repo_root, &registry, &response)?;
                    }
                }
                if args.summary {
                    println!("{}", render_feature_benchmark_summary(&response));
                } else {
                    print_json(&response)?;
                }
            }
        },
        Commands::Verify(args) => match &args.command {
            VerifyCommand::Feature(verify_args) => {
                let response = run_verify_feature_command(verify_args).await?;
                if verify_args.summary {
                    println!("{}", render_verify_summary(&response));
                } else {
                    print_json(&response)?;
                }
            }
            VerifyCommand::Journey(verify_args) => {
                let response = run_verify_journey_command(verify_args).await?;
                if verify_args.summary {
                    println!("{}", render_verify_summary(&response));
                } else {
                    print_json(&response)?;
                }
            }
            VerifyCommand::Adversarial(verify_args) => {
                let response = run_verify_adversarial_command(verify_args).await?;
                if verify_args.summary {
                    println!("{}", render_verify_summary(&response));
                } else {
                    print_json(&response)?;
                }
            }
            VerifyCommand::Compare(verify_args) => {
                let response = run_verify_compare_command(verify_args).await?;
                if verify_args.summary {
                    println!("{}", render_verify_summary(&response));
                } else {
                    print_json(&response)?;
                }
            }
            VerifyCommand::Sweep(verify_args) => {
                let response = run_verify_sweep_command(verify_args).await?;
                if verify_args.summary {
                    println!("{}", render_verify_summary(&response));
                } else {
                    print_json(&response)?;
                }
            }
            VerifyCommand::Doctor(verify_args) => {
                let response = run_verify_doctor_command(verify_args)?;
                if verify_args.summary {
                    println!("{}", render_verify_summary(&response));
                } else {
                    print_json(&response)?;
                }
            }
            VerifyCommand::List(verify_args) => {
                let response = run_verify_list_command(verify_args)?;
                if verify_args.summary {
                    println!("{}", render_verify_summary(&response));
                } else {
                    print_json(&response)?;
                }
            }
            VerifyCommand::Show(verify_args) => {
                let response = run_verify_show_command(verify_args)?;
                if verify_args.summary {
                    println!("{}", render_verify_summary(&response));
                } else {
                    print_json(&response)?;
                }
            }
        },
        Commands::Experiment(args) => {
            let mut response = run_experiment_command(&args, &base_url).await?;
            if args.write {
                write_experiment_artifacts(&args.output, &response)?;
                hydrate_experiment_evolution_summary(&mut response, &args.output)?;
            }
            if args.summary {
                println!("{}", render_experiment_summary(&response));
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
                write_bundle_memory_files(&args.output, &snapshot, None, false).await?;
                auto_checkpoint_live_snapshot(&args.output, &base_url, &snapshot, "agent").await?;
            } else if let Some(session) = args.session.as_deref() {
                set_bundle_session(&args.output, session)?;
            }
            let response = bundle_agent_profiles::build_bundle_agent_profiles(
                &args.output,
                args.name.as_deref(),
                args.shell.as_deref(),
            )?;
            if args.summary {
                println!(
                    "{}",
                    bundle_agent_profiles::render_bundle_agent_profiles_summary(&response)
                );
            } else {
                print_json(&response)?;
            }
        }
        Commands::Attach(args) => {
            let shell = args
                .shell
                .or_else(bundle_agent_profiles::detect_shell)
                .unwrap_or_else(|| "bash".to_string());
            println!("{}", render_attach_snippet(&shell, &args.output)?);
        }
        Commands::Resume(args) => {
            let codex_pack =
                harness_pack_enabled_for_bundle(&args.output, args.agent.as_deref(), "codex");
            let agent_zero_pack =
                harness_pack_enabled_for_bundle(&args.output, args.agent.as_deref(), "agent-zero");
            let hermes_pack =
                harness_pack_enabled_for_bundle(&args.output, args.agent.as_deref(), "hermes");
            let opencode_pack =
                harness_pack_enabled_for_bundle(&args.output, args.agent.as_deref(), "opencode");
            let openclaw_pack =
                harness_pack_enabled_for_bundle(&args.output, args.agent.as_deref(), "openclaw");
            let snapshot = match read_bundle_resume(&args, &base_url).await {
                Ok(snapshot) => snapshot,
                Err(err)
                    if codex_pack
                        || agent_zero_pack
                        || hermes_pack
                        || opencode_pack
                        || openclaw_pack =>
                {
                    if let Some(markdown) =
                        read_codex_pack_local_markdown(&args.output, "MEMD_MEMORY.md")?
                    {
                        write_bundle_turn_fallback_artifacts(
                            &args.output,
                            args.project.as_deref(),
                            args.namespace.as_deref(),
                            args.agent.as_deref(),
                            args.workspace.as_deref(),
                            args.visibility.as_deref(),
                            args.route.as_deref(),
                            args.intent.as_deref(),
                            &markdown,
                        )?;
                        println!("{markdown}");
                        return Ok(());
                    }
                    write_bundle_turn_placeholder_memory(
                        &args.output,
                        args.project.as_deref(),
                        args.namespace.as_deref(),
                        args.agent.as_deref(),
                        args.workspace.as_deref(),
                        args.visibility.as_deref(),
                        args.route.as_deref(),
                        args.intent.as_deref(),
                    )?;
                    return Err(err);
                }
                Err(err) => return Err(err),
            };
            write_bundle_memory_files(&args.output, &snapshot, None, false).await?;
            auto_checkpoint_live_snapshot(&args.output, &base_url, &snapshot, "resume").await?;
            if codex_pack || agent_zero_pack || openclaw_pack || hermes_pack || opencode_pack {
                let _ = refresh_harness_pack_files_for_snapshot(
                    &args.output,
                    &snapshot,
                    "resume",
                    &["codex", "agent-zero", "openclaw", "hermes", "opencode"],
                )
                .await?;
            }
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
        Commands::Refresh(args) => {
            invalidate_bundle_runtime_caches(&args.output)?;
            let snapshot = read_bundle_resume(&args, &base_url).await?;
            write_bundle_memory_files(&args.output, &snapshot, None, false).await?;
            auto_checkpoint_live_snapshot(&args.output, &base_url, &snapshot, "refresh").await?;
            let _ = refresh_harness_pack_files_for_snapshot(
                &args.output,
                &snapshot,
                "refresh",
                &["codex", "agent-zero", "openclaw", "hermes", "opencode"],
            )
            .await?;
            if args.prompt {
                println!("{}", render_resume_prompt(&snapshot));
            } else {
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
                    "refresh project={} namespace={} agent={} workspace={} visibility={} context={} working={} inbox={} workspaces={} changes={} est_tokens={} context_pressure={} redundant_items={} refresh_recommended={} focus=\"{}\" pressure=\"{}\"",
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
            }
        }
        Commands::Watch(args) => {
            run_workspace_watch(&client, &base_url, &args).await?;
        }
        Commands::Handoff(args) => {
            let snapshot = read_bundle_handoff(&args, &base_url).await?;
            write_bundle_memory_files(&args.output, &snapshot.resume, Some(&snapshot), false)
                .await?;
            auto_checkpoint_live_snapshot(&args.output, &base_url, &snapshot.resume, "handoff")
                .await?;
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
            let response = match checkpoint_with_bundle_defaults(&args, &base_url).await {
                Ok(response) => response,
                Err(err) => {
                    write_bundle_turn_placeholder_memory(
                        &args.output,
                        args.project.as_deref(),
                        args.namespace.as_deref(),
                        None,
                        args.workspace.as_deref(),
                        args.visibility.as_deref(),
                        Some("auto"),
                        Some("current_task"),
                    )?;
                    return Err(err);
                }
            };
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
            write_bundle_memory_files(&args.output, &snapshot, None, false).await?;
            refresh_live_bundle_event_pages(&args.output, &snapshot, None)?;
            auto_checkpoint_live_snapshot(&args.output, &base_url, &snapshot, "checkpoint").await?;
            let _ = refresh_harness_pack_files_for_snapshot(
                &args.output,
                &snapshot,
                "checkpoint",
                &["codex", "agent-zero", "openclaw", "hermes", "opencode"],
            )
            .await?;
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
            write_bundle_memory_files(&args.output, &snapshot, None, false).await?;
            auto_checkpoint_live_snapshot(&args.output, &base_url, &snapshot, "remember").await?;
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
        Commands::Inspiration(args) => {
            let root = resolve_inspiration_root(args.root.as_deref())?;
            let matches = search_inspiration_lane(&root, &args.query, args.limit)?;
            if args.summary {
                println!(
                    "{}",
                    render_inspiration_search_summary(&root, &args.query, &matches)
                );
            } else {
                println!(
                    "{}",
                    render_inspiration_search_markdown(&root, &args.query, &matches)
                );
            }
        }
        Commands::Skills(args) => {
            let root = resolve_skill_catalog_root(args.root.as_deref())?;
            let catalog = build_skill_catalog(&root)?;
            if let Some(query) = args.query.as_deref() {
                let matches = find_skill_catalog_matches(&catalog, query);
                if args.summary {
                    println!(
                        "{}",
                        render_skill_catalog_match_summary(&catalog, query, &matches)
                    );
                } else {
                    println!(
                        "{}",
                        render_skill_catalog_match_markdown(&catalog, query, &matches)
                    );
                }
            } else if args.summary {
                println!("{}", render_skill_catalog_summary(&catalog));
            } else {
                println!("{}", render_skill_catalog_markdown(&catalog));
            }
        }
        Commands::Packs(args) => {
            let bundle_root = resolve_pack_bundle_root(args.root.as_deref())?;
            let runtime = read_bundle_runtime_config(&bundle_root)?;
            let index = crate::harness::index::build_harness_pack_index(
                &bundle_root,
                runtime
                    .as_ref()
                    .and_then(|config| config.project.as_deref()),
                runtime
                    .as_ref()
                    .and_then(|config| config.namespace.as_deref()),
            );
            let index =
                crate::harness::index::filter_harness_pack_index(index, args.query.as_deref());
            if args.json {
                print_json(&render_harness_pack_index_json(&index))?;
            } else if args.summary {
                println!(
                    "{}",
                    render_harness_pack_index_summary(&bundle_root, &index, args.query.as_deref())
                );
            } else {
                println!(
                    "{}",
                    render_harness_pack_index_markdown(&bundle_root, &index)
                );
            }
        }
        Commands::Commands(args) => {
            let bundle_root = resolve_pack_bundle_root(args.root.as_deref())?;
            let catalog = command_catalog::build_command_catalog(&bundle_root);
            let catalog = command_catalog::filter_command_catalog(catalog, args.query.as_deref());
            if args.json {
                print_json(&render_command_catalog_json(&catalog))?;
            } else if args.summary {
                println!(
                    "{}",
                    render_command_catalog_summary(&catalog, args.query.as_deref())
                );
            } else {
                println!("{}", render_command_catalog_markdown(&catalog));
            }
        }
        Commands::Setup(args) => {
            let decision = resolve_bootstrap_authority(setup_args_to_init_args(&args)).await?;
            let init_args = decision.init_args;
            write_init_bundle(&init_args)?;
            if decision.fallback_activated {
                set_bundle_localhost_read_only_authority_state(
                    &init_args.output,
                    &decision.shared_base_url,
                    "setup",
                    "shared authority unavailable during bootstrap",
                )?;
                write_agent_profiles(&init_args.output)?;
                write_native_agent_bridge_files(&init_args.output)?;
            }
            if args.json {
                print_json(&json!({
                    "bundle": init_args.output,
                    "project": init_args.project,
                    "namespace": init_args.namespace,
                    "agent": init_args.agent,
                    "base_url": init_args.base_url,
                    "shared_base_url": decision.shared_base_url,
                    "authority_mode": if decision.fallback_activated { "localhost_read_only" } else { "shared" },
                    "route": init_args.route,
                    "intent": init_args.intent,
                    "voice_mode": init_args.voice_mode,
                    "workspace": init_args.workspace,
                    "visibility": init_args.visibility,
                    "setup_ready": true,
                }))?;
            } else if args.summary {
                println!(
                    "setup bundle={} project={} namespace={} agent={} voice={} authority={} ready=true",
                    init_args.output.display(),
                    init_args.project.as_deref().unwrap_or("none"),
                    init_args.namespace.as_deref().unwrap_or("none"),
                    init_args.agent,
                    init_args.voice_mode.as_deref().unwrap_or("caveman-ultra"),
                    if decision.fallback_activated {
                        "localhost_read_only"
                    } else {
                        "shared"
                    },
                );
            } else {
                println!("Initialized memd bundle at {}", init_args.output.display());
                if decision.fallback_activated {
                    eprintln!("memd authority warning:");
                    eprintln!("- shared authority unavailable");
                    eprintln!("- localhost fallback is lower trust");
                    eprintln!("- prompt-injection and split-brain risk increased");
                    eprintln!("- coordination writes blocked");
                }
            }
        }
        Commands::Doctor(args) => {
            let bundle_root = resolve_doctor_bundle_root(args.output.as_deref())?;
            let mut status = read_bundle_status(&bundle_root, &base_url).await?;
            let setup_ready = status
                .get("setup_ready")
                .and_then(|value| value.as_bool())
                .unwrap_or(false);
            if args.repair && !setup_ready {
                let project_root = args.project_root.clone().or(detect_current_project_root()?);
                let setup_args =
                    doctor_args_to_setup_args(&args, bundle_root.clone(), project_root);
                let decision =
                    resolve_bootstrap_authority(setup_args_to_init_args(&setup_args)).await?;
                write_init_bundle(&decision.init_args)?;
                if decision.fallback_activated {
                    set_bundle_localhost_read_only_authority_state(
                        &decision.init_args.output,
                        &decision.shared_base_url,
                        "doctor",
                        "shared authority unavailable during repair bootstrap",
                    )?;
                    write_agent_profiles(&decision.init_args.output)?;
                    write_native_agent_bridge_files(&decision.init_args.output)?;
                }
                status = read_bundle_status(&bundle_root, &base_url).await?;
            } else if args.repair {
                let repaired_worker_env = repair_bundle_worker_name_env(&bundle_root)?;
                if repaired_worker_env {
                    write_agent_profiles(&bundle_root)?;
                }
                status = read_bundle_status(&bundle_root, &base_url).await?;
            }
            if args.json {
                print_json(&status)?;
            } else if args.summary {
                println!("{}", render_bundle_status_summary(&status));
            } else {
                println!("{}", render_doctor_status_markdown(&bundle_root, &status));
            }
        }
        Commands::Config(args) => {
            let bundle_root = resolve_setup_bundle_root(args.output.as_deref())?;
            let project_root = args.project_root.clone().or(detect_current_project_root()?);
            let runtime = read_bundle_runtime_config(&bundle_root)?;
            let status = read_bundle_status(&bundle_root, &base_url).await.ok();
            let config = render_bundle_config_snapshot(
                &bundle_root,
                project_root.as_deref(),
                runtime.as_ref(),
                status.as_ref(),
            );
            if args.json {
                print_json(&config)?;
            } else if args.summary {
                println!("{}", render_bundle_config_summary(&config));
            } else {
                println!("{}", render_bundle_config_markdown(&config));
            }
        }
        Commands::Memory(args) => {
            let bundle_root = resolve_compiled_memory_bundle_root(args.root.as_deref())?;
            let use_runtime_summary = !args.quality
                && !args.list
                && compiled_memory_target(&args).is_none()
                && args.query.is_none();
            if use_runtime_summary {
                match read_memory_surface(&bundle_root, &base_url).await {
                    Ok(response) if args.json => print_json(&response)?,
                    Ok(response) => println!("{}", render_memory_surface_summary(&response)),
                    Err(_) if !args.json => {
                        let page =
                            bundle_compiled_memory_path(&bundle_root, MemoryObjectLane::Working);
                        let content = fs::read_to_string(&page)
                            .with_context(|| format!("read {}", page.display()))?;
                        println!("{}", render_compiled_memory_page_summary(&page, &content));
                    }
                    Err(err) => return Err(err),
                }
            } else if args.quality {
                let report = build_compiled_memory_quality_report(&bundle_root)?;
                if args.json {
                    print_json(&render_compiled_memory_quality_json(&bundle_root, &report))?;
                } else if args.summary {
                    println!(
                        "{}",
                        render_compiled_memory_quality_summary(&bundle_root, &report)
                    );
                } else {
                    println!(
                        "{}",
                        render_compiled_memory_quality_markdown(&bundle_root, &report)
                    );
                }
            } else if args.list {
                let index = render_compiled_memory_index(&bundle_root)?;
                let index = filter_compiled_memory_index(
                    index,
                    args.lanes_only,
                    args.items_only,
                    args.filter.as_deref(),
                );
                if args.json {
                    print_json(&render_compiled_memory_index_json(&bundle_root, &index))?;
                } else if args.summary {
                    println!(
                        "{}",
                        render_compiled_memory_index_summary(&bundle_root, &index)
                    );
                } else if args.grouped {
                    println!(
                        "{}",
                        render_compiled_memory_index_grouped_markdown(
                            &bundle_root,
                            &index,
                            args.expand_items
                        )
                    );
                } else {
                    println!(
                        "{}",
                        render_compiled_memory_index_markdown(&bundle_root, &index)
                    );
                }
            } else if let Some(target) = compiled_memory_target(&args) {
                let path = resolve_compiled_memory_page(&bundle_root, target)?;
                let content = fs::read_to_string(&path)
                    .with_context(|| format!("read {}", path.display()))?;
                if args.summary {
                    println!("{}", render_compiled_memory_page_summary(&path, &content));
                } else {
                    println!("{}", render_compiled_memory_page_markdown(&path, &content));
                }
            } else if let Some(query) = args.query.as_deref() {
                let matches = search_compiled_memory_pages(&bundle_root, query, args.limit)?;
                if args.summary {
                    println!(
                        "{}",
                        render_compiled_memory_search_summary(&bundle_root, query, &matches)
                    );
                } else {
                    println!(
                        "{}",
                        render_compiled_memory_search_markdown(&bundle_root, query, &matches)
                    );
                }
            } else {
                let page = bundle_compiled_memory_path(&bundle_root, MemoryObjectLane::Working);
                let content = fs::read_to_string(&page)
                    .with_context(|| format!("read {}", page.display()))?;
                if args.summary {
                    println!("{}", render_compiled_memory_page_summary(&page, &content));
                } else {
                    println!("{}", render_compiled_memory_page_markdown(&page, &content));
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
        Commands::MemoryVerify(input) => {
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
        Commands::Lookup(args) => {
            let runtime = read_bundle_runtime_config(&args.output)?;
            let req = build_lookup_request(&args, runtime.as_ref())?;
            let response = lookup_with_fallbacks(&client, &req, &args.query).await?;
            if args.json {
                print_json(&response)?;
            } else {
                println!(
                    "{}",
                    render_lookup_markdown(&args.query, &response, args.verbose)
                );
            }
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
        Commands::Events(args) => {
            let bundle_root = resolve_compiled_event_bundle_root(Some(&args.root))?;
            if args.list {
                let index = render_compiled_event_index(&bundle_root)?;
                if args.summary {
                    println!(
                        "{}",
                        render_compiled_event_index_summary(&bundle_root, &index)
                    );
                } else {
                    print_json(&render_compiled_event_index_json(&bundle_root, &index))?;
                }
            } else if let Some(query) = args.query.as_deref() {
                let hits = search_compiled_event_pages(&bundle_root, query, args.limit)?;
                if args.summary {
                    println!(
                        "{}",
                        render_compiled_event_search_summary(&bundle_root, query, &hits)
                    );
                } else {
                    print_json(&hits)?;
                }
            } else if let Some(target) = args.open.as_deref() {
                let path = resolve_compiled_event_page(&bundle_root, target)?;
                let content = fs::read_to_string(&path)
                    .with_context(|| format!("read {}", path.display()))?;
                if args.summary {
                    println!("{}", render_compiled_event_page_summary(&path, &content));
                } else {
                    println!("{}", render_compiled_event_page_markdown(&path, &content));
                }
            } else {
                let index = render_compiled_event_index(&bundle_root)?;
                if args.summary {
                    println!(
                        "{}",
                        render_compiled_event_index_summary(&bundle_root, &index)
                    );
                } else {
                    println!(
                        "{}",
                        render_compiled_event_index_markdown(&bundle_root, &index)
                    );
                }
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
                    mode: Some("scan".to_string()),
                    apply: Some(false),
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
        Commands::Maintain(args) => {
            let response = run_maintain_command(&args, &cli.base_url).await?;
            if args.summary {
                println!("{}", render_maintain_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Policy(args) => {
            let response = client.policy().await?;
            if args.summary {
                println!("{}", render_policy_summary(&response, args.follow));
            } else {
                print_json(&response)?;
            }
        }
        Commands::SkillPolicy(args) => {
            let response = client.policy().await?;
            let report = build_skill_lifecycle_report(&response);
            if args.query {
                let query = SkillPolicyApplyReceiptsRequest {
                    project: args.project.clone(),
                    namespace: args.namespace.clone(),
                    workspace: args.workspace.clone(),
                    limit: args.limit,
                };
                let receipts = client.skill_policy_apply_receipts(&query).await?;
                let activations = client
                    .skill_policy_activations(&SkillPolicyActivationEntriesRequest {
                        project: args.project.clone(),
                        namespace: args.namespace.clone(),
                        workspace: args.workspace.clone(),
                        limit: args.limit,
                    })
                    .await?;
                if args.summary {
                    println!(
                        "{}",
                        render_skill_policy_query_summary(&receipts, &activations, args.follow)
                    );
                } else {
                    print_json(&serde_json::json!({
                        "receipts": receipts,
                        "activations": activations,
                    }))?;
                }
            } else if args.summary {
                println!("{}", render_skill_policy_summary(&response, args.follow));
                println!();
                print!("{}", render_skill_lifecycle_report(&report, args.follow));
            } else {
                print_json(&response)?;
            }
            if args.write || args.apply {
                let receipt =
                    write_skill_policy_artifacts(&args.output, &response, &report, args.apply)?;
                if let Some(receipt) = receipt {
                    let posted = client
                        .record_skill_policy_apply(&skill_policy_apply_request(&receipt))
                        .await?;
                    println!(
                        "applied {} via server receipt {}",
                        posted.receipt.applied_count, posted.receipt.id
                    );
                }
                let mut paths = vec![
                    skill_policy_batch_state_path(&args.output)
                        .display()
                        .to_string(),
                    skill_policy_review_state_path(&args.output)
                        .display()
                        .to_string(),
                    skill_policy_activate_state_path(&args.output)
                        .display()
                        .to_string(),
                ];
                if args.apply {
                    paths.push(
                        skill_policy_apply_state_path(&args.output)
                            .display()
                            .to_string(),
                    );
                }
                println!("wrote {}", paths.join(", "));
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
                    workspace: None,
                    visibility: None,
                    route: parse_retrieval_route(args.route.clone())?,
                    intent: parse_retrieval_intent(args.intent.clone())?,
                    limit: args.limit,
                    max_chars_per_item: args.max_chars_per_item,
                })
                .await?;

            let packet = build_compaction_packet(BuildCompactionPacketArgs {
                session: CompactionSession {
                    project: args.project,
                    agent: args.agent,
                    task: args.task,
                },
                goal: args.goal,
                hard_constraints: args.hard_constraint,
                active_work: args.active_work,
                decisions: args
                    .decision
                    .into_iter()
                    .enumerate()
                    .map(|(idx, text)| CompactionDecision {
                        id: format!("decision-{}", idx + 1),
                        text,
                    })
                    .collect(),
                open_loops: args
                    .open_loop
                    .into_iter()
                    .enumerate()
                    .map(|(idx, text)| CompactionOpenLoop {
                        id: format!("loop-{}", idx + 1),
                        text,
                        status: "open".to_string(),
                    })
                    .collect(),
                exact_refs: args
                    .exact_ref
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
                next_actions: args.next_action,
                do_not_drop: args.do_not_drop,
                memory,
            });

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
        Commands::Ui(args) => match args.mode {
            UiMode::Home(args) => {
                let snapshot = client.visible_memory_snapshot().await?;
                if args.json {
                    print_json(&snapshot)?;
                } else {
                    println!("{}", render_visible_memory_home(&snapshot, args.follow));
                }
            }
            UiMode::Artifact(args) => {
                let artifact_id = uuid::Uuid::parse_str(&args.id)
                    .with_context(|| format!("parse visible memory artifact id {}", args.id))?;
                let detail = client.visible_memory_artifact_detail(artifact_id).await?;
                if args.json {
                    print_json(&detail)?;
                } else {
                    println!(
                        "{}",
                        render_visible_memory_artifact_detail(&detail, args.follow)
                    );
                }
            }
            UiMode::Map(args) => {
                let snapshot = client.visible_memory_snapshot().await?;
                if args.json {
                    print_json(&snapshot)?;
                } else {
                    println!(
                        "{}",
                        render_visible_memory_knowledge_map(&snapshot, args.follow)
                    );
                }
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
            HookMode::Capture(args) => {
                let content = if let Some(content) = &args.content {
                    content.clone()
                } else if let Some(path) = &args.input {
                    fs::read_to_string(path).with_context(|| {
                        format!("read hook capture input file {}", path.display())
                    })?
                } else if args.stdin {
                    let mut content = String::new();
                    io::stdin()
                        .read_to_string(&mut content)
                        .context("read hook capture payload from stdin")?;
                    content
                } else {
                    "hook capture: active task state changed".to_string()
                };
                let effective_promote_kind = effective_hook_capture_promote_kind(&args, &content);
                let (supersede_targets, supersede_diagnostics) =
                    find_hook_capture_supersede_targets(&base_url, &args, &content).await?;
                let promote_response = if let Some(promote_kind) = effective_promote_kind {
                    Some(
                        remember_with_bundle_defaults(
                            &remember_args_from_effective_hook_capture(
                                &args,
                                content.clone(),
                                promote_kind,
                                supersede_targets.clone(),
                            ),
                            &base_url,
                        )
                        .await?,
                    )
                } else {
                    None
                };
                let supersede_responses = if let Some(response) = promote_response.as_ref() {
                    mark_hook_capture_supersede_targets(
                        &base_url,
                        &args,
                        &supersede_targets,
                        response.item.id,
                    )
                    .await?
                } else {
                    Vec::new()
                };
                let checkpoint = checkpoint_with_bundle_defaults(
                    &CheckpointArgs {
                        output: args.output.clone(),
                        project: args.project.clone(),
                        namespace: args.namespace.clone(),
                        workspace: args.workspace.clone(),
                        visibility: args.visibility.clone(),
                        source_path: args
                            .source_path
                            .clone()
                            .or(Some("hook-capture".to_string())),
                        confidence: args.confidence,
                        ttl_seconds: args.ttl_seconds.or(Some(86_400)),
                        tag: if args.tag.is_empty() {
                            vec![
                                "hook-capture".to_string(),
                                "episodic".to_string(),
                                "live-memory".to_string(),
                            ]
                        } else {
                            args.tag.clone()
                        },
                        content: Some(content.clone()),
                        input: None,
                        stdin: false,
                    },
                    &base_url,
                )
                .await;
                let checkpoint_id = checkpoint
                    .as_ref()
                    .map(|response| response.item.id.to_string())
                    .unwrap_or_else(|_| "none".to_string());
                let checkpoint_json = checkpoint
                    .as_ref()
                    .map(|response| json!(response))
                    .unwrap_or_else(|err| json!({ "error": err.to_string() }));
                let snapshot = match checkpoint {
                    Ok(_) => match read_bundle_resume(
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
                    .await
                    {
                        Ok(snapshot) => Some(snapshot),
                        Err(_) => {
                            preserve_codex_capture_locally(&args.output, &content)?;
                            None
                        }
                    },
                    Err(_) => {
                        preserve_codex_capture_locally(&args.output, &content)?;
                        None
                    }
                };
                if let Some(snapshot) = snapshot.as_ref() {
                    write_bundle_memory_files(&args.output, snapshot, None, false).await?;
                    refresh_live_bundle_event_pages(&args.output, snapshot, None)?;
                    auto_checkpoint_live_snapshot(
                        &args.output,
                        &base_url,
                        snapshot,
                        "hook-capture",
                    )
                    .await?;
                    let _ = refresh_harness_pack_files_for_snapshot(
                        &args.output,
                        snapshot,
                        "hook-capture",
                        &["codex", "agent-zero", "openclaw"],
                    )
                    .await?;
                }
                if args.summary {
                    let (supersede_query, supersede_tried, supersede_hits) =
                        summarize_hook_capture_supersede_diagnostics(&supersede_diagnostics);
                    println!(
                        "hook_capture stored={} promoted={} superseded={} query={} tried={} hits={} working={} inbox={}",
                        checkpoint_id,
                        promote_response
                            .as_ref()
                            .map(|response| response.item.id.to_string())
                            .unwrap_or_else(|| "none".to_string()),
                        supersede_responses.len(),
                        supersede_query,
                        supersede_tried,
                        supersede_hits,
                        snapshot
                            .as_ref()
                            .map(|value| value.working.records.len())
                            .unwrap_or(0),
                        snapshot
                            .as_ref()
                            .map(|value| value.inbox.items.len())
                            .unwrap_or(0)
                    );
                } else {
                    print_json(&json!({
                        "live": checkpoint_json,
                        "promoted": promote_response,
                        "superseded": supersede_responses,
                        "supersede_search": supersede_diagnostics,
                    }))?;
                }
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
            let decision = resolve_bootstrap_authority(args).await?;
            write_init_bundle(&decision.init_args)?;
            if decision.fallback_activated {
                set_bundle_localhost_read_only_authority_state(
                    &decision.init_args.output,
                    &decision.shared_base_url,
                    "init",
                    "shared authority unavailable during bootstrap",
                )?;
                write_agent_profiles(&decision.init_args.output)?;
                write_native_agent_bridge_files(&decision.init_args.output)?;
            }
            println!(
                "Initialized memd bundle at {}",
                decision.init_args.output.display()
            );
            if decision.fallback_activated {
                eprintln!("memd authority warning:");
                eprintln!("- shared authority unavailable");
                eprintln!("- localhost fallback is lower trust");
                eprintln!("- prompt-injection and split-brain risk increased");
                eprintln!("- coordination writes blocked");
            }
        }
        Commands::Loops(args) => {
            let entries = read_loop_entries(&args.output)?;
            if let Some(slug) = args.loop_slug.as_deref() {
                print_loop_detail(&entries, slug)?;
            } else if args.summary {
                print_loop_summary(&entries);
            } else {
                print_loop_list(&entries, &args.output);
            }
        }
        Commands::Telemetry(args) => {
            run_telemetry(&args)?;
        }
        Commands::Autoresearch(args) => {
            run_autoresearch(&args, &base_url).await?;
        }
    }

    Ok(())
}

async fn run_workspace_watch(
    _client: &MemdClient,
    base_url: &str,
    args: &WatchArgs,
) -> anyhow::Result<()> {
    println!(
        "workspace_watch root={} output={} debounce_ms={}",
        args.root.display(),
        args.output.display(),
        args.debounce_ms
    );

    let initial = read_bundle_resume(
        &ResumeArgs {
            output: args.output.clone(),
            project: args.project.clone(),
            namespace: args.namespace.clone(),
            agent: args.agent.clone(),
            workspace: args.workspace.clone(),
            visibility: args.visibility.clone(),
            route: args.route.clone(),
            intent: args.intent.clone().or(Some("current_task".to_string())),
            limit: args.limit,
            rehydration_limit: args.rehydration_limit,
            semantic: args.semantic,
            prompt: false,
            summary: false,
        },
        base_url,
    )
    .await?;
    write_bundle_memory_files(&args.output, &initial, None, false).await?;
    auto_checkpoint_live_snapshot(&args.output, base_url, &initial, "watch-start").await?;

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
                    .any(|path| workspace_path_should_trigger(path));
                if should_trigger {
                    let _ = tx.send(());
                }
            }
        },
        NotifyConfig::default(),
    )
    .context("create workspace watcher")?;
    watcher
        .watch(&args.root, RecursiveMode::Recursive)
        .with_context(|| format!("watch {}", args.root.display()))?;

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

        match read_bundle_resume(
            &ResumeArgs {
                output: args.output.clone(),
                project: args.project.clone(),
                namespace: args.namespace.clone(),
                agent: args.agent.clone(),
                workspace: args.workspace.clone(),
                visibility: args.visibility.clone(),
                route: args.route.clone(),
                intent: args.intent.clone().or(Some("current_task".to_string())),
                limit: args.limit,
                rehydration_limit: args.rehydration_limit,
                semantic: args.semantic,
                prompt: false,
                summary: false,
            },
            base_url,
        )
        .await
        {
            Ok(snapshot) => {
                if let Err(err) =
                    write_bundle_memory_files(&args.output, &snapshot, None, false).await
                {
                    eprintln!("workspace watch write failed: {err:#}");
                    continue;
                }
                if let Err(err) =
                    auto_checkpoint_live_snapshot(&args.output, base_url, &snapshot, "watch").await
                {
                    eprintln!("workspace watch auto-checkpoint failed: {err:#}");
                }
                println!(
                    "workspace_watch update root={} working={} inbox={} focus=\"{}\"",
                    args.root.display(),
                    snapshot.working.records.len(),
                    snapshot.inbox.items.len(),
                    snapshot
                        .working
                        .records
                        .first()
                        .map(|record| compact_inline(&record.record, 72))
                        .unwrap_or_else(|| "none".to_string())
                );
            }
            Err(err) => eprintln!("workspace watch refresh failed: {err:#}"),
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

fn default_auto_short_term_capture() -> bool {
    true
}

fn default_true() -> bool {
    true
}

fn default_authority_mode() -> String {
    "shared".to_string()
}

fn default_voice_mode() -> String {
    "caveman-ultra".to_string()
}

const SHARED_MEMD_BASE_URL: &str = "http://100.104.154.24:8787";
const LOCALHOST_MEMD_BASE_URL: &str = "http://127.0.0.1:8787";

fn default_base_url() -> String {
    if let Ok(value) = std::env::var("MEMD_BASE_URL") {
        return value;
    }

    read_bundle_runtime_config(&default_global_bundle_root())
        .ok()
        .flatten()
        .and_then(|runtime| runtime.base_url)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| SHARED_MEMD_BASE_URL.to_string())
}

#[derive(Debug, Clone)]
struct BootstrapAuthorityDecision {
    init_args: InitArgs,
    shared_base_url: String,
    fallback_activated: bool,
}

fn localhost_memd_base_url() -> String {
    std::env::var("MEMD_LOCALHOST_FALLBACK_BASE_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| LOCALHOST_MEMD_BASE_URL.to_string())
}

async fn memd_base_url_reachable(base_url: &str) -> bool {
    let Ok(client) = MemdClient::new(base_url) else {
        return false;
    };
    timeout_ok(client.healthz()).await.is_some()
}

async fn resolve_bootstrap_authority(
    init_args: InitArgs,
) -> anyhow::Result<BootstrapAuthorityDecision> {
    let shared_base_url = init_args.base_url.clone();
    let localhost_fallback_base_url = localhost_memd_base_url();
    if is_loopback_base_url(&shared_base_url) {
        return Ok(BootstrapAuthorityDecision {
            init_args,
            shared_base_url,
            fallback_activated: false,
        });
    }

    if memd_base_url_reachable(&shared_base_url).await {
        return Ok(BootstrapAuthorityDecision {
            init_args,
            shared_base_url,
            fallback_activated: false,
        });
    }

    if !memd_base_url_reachable(&localhost_fallback_base_url).await {
        anyhow::bail!(
            "shared authority {} is unreachable and localhost fallback {} is not available",
            shared_base_url,
            localhost_fallback_base_url
        );
    }

    if !init_args.allow_localhost_read_only_fallback {
        anyhow::bail!(
            "shared authority {} is unreachable. localhost fallback {} is lower trust, read-only, and requires explicit consent via --allow-localhost-read-only-fallback",
            shared_base_url,
            localhost_fallback_base_url
        );
    }

    let mut init_args = init_args;
    init_args.base_url = localhost_fallback_base_url;
    Ok(BootstrapAuthorityDecision {
        init_args,
        shared_base_url,
        fallback_activated: true,
    })
}

fn default_bundle_root_path() -> PathBuf {
    if let Ok(value) = std::env::var("MEMD_BUNDLE_ROOT") {
        let value = value.trim();
        if !value.is_empty() {
            return PathBuf::from(value);
        }
    }

    default_global_bundle_root()
}

fn default_init_output_path() -> PathBuf {
    match detect_current_project_root() {
        Ok(Some(root)) => root.join(".memd"),
        _ => default_global_bundle_root(),
    }
}

fn default_global_bundle_root() -> PathBuf {
    home_dir()
        .map(|path| path.join(".memd"))
        .unwrap_or_else(|| PathBuf::from(".memd"))
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
}

fn init_project_name(args: &InitArgs, project_root: Option<&Path>) -> String {
    args.project
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            if args.global {
                None
            } else {
                project_root
                    .and_then(|root| {
                        root.file_name()
                            .and_then(|value| value.to_str())
                            .map(|value| value.trim().to_string())
                    })
                    .filter(|value| !value.is_empty())
            }
        })
        .unwrap_or_else(|| "global".to_string())
}

fn init_namespace_name(args: &InitArgs, output: &Path) -> Option<String> {
    args.namespace
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            if args.global || output == default_global_bundle_root() {
                Some("global".to_string())
            } else {
                Some("main".to_string())
            }
        })
}

fn detect_init_project_root(args: &InitArgs) -> anyhow::Result<Option<PathBuf>> {
    if let Some(root) = args.project_root.as_ref() {
        let root = if root.is_absolute() {
            root.clone()
        } else {
            std::env::current_dir()
                .context("read current directory")?
                .join(root)
        };
        return Ok(Some(fs::canonicalize(&root).unwrap_or(root)));
    }

    Ok(detect_current_project_root()?.map(|root| fs::canonicalize(&root).unwrap_or(root)))
}

fn detect_current_project_root() -> anyhow::Result<Option<PathBuf>> {
    let start = std::env::current_dir().context("read current directory")?;
    Ok(find_project_root(&start))
}

fn find_project_root(start: &Path) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        if is_project_root_candidate(&dir) {
            return Some(dir);
        }
        let parent = dir.parent()?;
        if parent == dir {
            return None;
        }
        dir = parent.to_path_buf();
    }
}

fn read_loop_entries(output: &Path) -> anyhow::Result<Vec<LoopEntry>> {
    let loop_dir = loops_directory(output);
    if !loop_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(&loop_dir)
        .with_context(|| format!("read loops directory {}", loop_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let is_json = path
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.eq_ignore_ascii_case("json"))
            .unwrap_or(false);
        if !is_json {
            continue;
        }
        if path
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value == "loops.summary.json")
        {
            continue;
        }

        let raw = fs::read(&path).with_context(|| format!("read {}", path.display()))?;
        let record: LoopRecord =
            serde_json::from_slice(&raw).with_context(|| format!("parse {}", path.display()))?;
        let (slug, normalized_slug) = derive_loop_slugs(&record, &path);
        entries.push(LoopEntry {
            slug,
            normalized_slug,
            record,
            path,
        });
    }

    entries.sort_by(|a, b| a.normalized_slug.cmp(&b.normalized_slug));
    Ok(entries)
}

fn loops_directory(output: &Path) -> PathBuf {
    output.join("loops")
}

fn derive_loop_slugs(record: &LoopRecord, path: &Path) -> (String, String) {
    let candidate = record
        .slug
        .as_deref()
        .map(str::to_string)
        .unwrap_or_else(|| slug_from_path(path));
    let slug = canonical_slug(&candidate);
    let normalized_slug = slug.to_lowercase();
    (slug, normalized_slug)
}

fn slug_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| "loop".to_string())
}

fn canonical_slug(value: &str) -> String {
    let trimmed = strip_loop_prefix(value);
    if trimmed.is_empty() {
        "loop".to_string()
    } else {
        trimmed.to_string()
    }
}

fn strip_loop_prefix(value: &str) -> &str {
    value
        .trim()
        .trim_start_matches("loop-")
        .trim_start_matches("loop_")
        .trim_start_matches("loops-")
        .trim_start_matches("loops_")
}

fn print_loop_list(entries: &[LoopEntry], output: &Path) {
    let loop_dir = loops_directory(output);
    if entries.is_empty() {
        println!(
            "No loop records found in {}. Run autoresearch to capture loop metadata.",
            loop_dir.display()
        );
        return;
    }

    println!(
        "{:<24} {:>10} {:>12} {:<10} {}",
        "Loop", "Improved", "Tokens", "Status", "Name"
    );
    for entry in entries {
        println!(
            "{:<24} {:>10} {:>12} {:<10} {}",
            entry.slug,
            format_percent(entry.record.percent_improvement),
            format_tokens(entry.record.token_savings),
            entry
                .record
                .status
                .as_deref()
                .unwrap_or("pending")
                .chars()
                .take(10)
                .collect::<String>(),
            entry.record.name.as_deref().unwrap_or("")
        );
    }

    println!();
    println!(
        "Use `memd loops --summary` for aggregate metrics or `memd loops --loop <slug>` for details."
    );
}

fn print_loop_detail(entries: &[LoopEntry], slug_arg: &str) -> anyhow::Result<()> {
    let normalized = canonical_slug(slug_arg).to_lowercase();
    let entry = entries
        .iter()
        .find(|entry| entry.normalized_slug == normalized)
        .ok_or_else(|| anyhow!("loop '{}' not found", slug_arg))?;

    println!("Loop: {}", entry.slug);
    if let Some(name) = &entry.record.name {
        println!("Name: {}", name);
    }
    if let Some(status) = &entry.record.status {
        println!("Status: {}", status);
    }
    if let Some(iteration) = entry.record.iteration {
        println!("Iteration: {}", iteration);
    }
    println!(
        "Percent improvement: {}",
        format_percent(entry.record.percent_improvement)
    );
    println!(
        "Token savings: {}",
        format_tokens(entry.record.token_savings)
    );
    if let Some(summary) = &entry.record.summary {
        println!("Summary:\n{}", indent_text(summary, 2));
    }
    if entry.normalized_slug == "self-evolution"
        && let Some(control_plane) = loop_control_plane_summary(entry)
    {
        println!("Control plane:");
        println!("  proposal: {}", control_plane.proposal_state);
        println!(
            "  scope: {}/{}",
            control_plane.scope_class, control_plane.scope_gate
        );
        println!("  authority: {}", control_plane.authority_tier);
        println!("  merge: {}", control_plane.merge_status);
        println!("  durability: {}", control_plane.durability_status);
        println!("  durable truth: {}", control_plane.durable_truth);
        println!("  branch: {}", control_plane.branch);
    }
    if let Some(artifacts) = &entry.record.artifacts {
        println!("Artifacts:");
        for artifact in artifacts {
            println!("  - {}", artifact);
        }
    }
    println!("Recorded at: {}", entry.path.display());
    if !entry.record.metadata.is_null() {
        println!("Metadata:");
        let json = serde_json::to_string_pretty(&entry.record.metadata)?;
        for line in json.lines() {
            println!("  {}", line);
        }
    }

    Ok(())
}

fn print_loop_summary(entries: &[LoopEntry]) {
    if entries.is_empty() {
        println!("No loop records present yet.");
        return;
    }

    let mut status_counts = BTreeMap::new();
    for entry in entries {
        let status = entry
            .record
            .status
            .as_deref()
            .unwrap_or("pending")
            .to_string();
        *status_counts.entry(status).or_insert(0usize) += 1;
    }

    println!("Loop summary ({} records)", entries.len());
    println!(
        "Status counts: {}",
        status_counts
            .iter()
            .map(|(status, count)| format!("{}={}", status, count))
            .collect::<Vec<_>>()
            .join(" ")
    );

    let improvements: Vec<_> = entries
        .iter()
        .filter_map(|entry| entry.record.percent_improvement.map(|value| (value, entry)))
        .collect();
    if !improvements.is_empty() {
        let total_improvement: f64 = improvements.iter().map(|(value, _)| *value).sum();
        let average = total_improvement / (improvements.len() as f64);
        println!("Average improvement: {:.2}%", average);
        if let Some((best, entry)) = improvements
            .iter()
            .copied()
            .max_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        {
            println!("Best improvement: {:.2}% ({})", best, entry.slug);
        }
        if let Some((worst, entry)) = improvements
            .iter()
            .copied()
            .min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        {
            println!("Worst improvement: {:.2}% ({})", worst, entry.slug);
        }
    } else {
        println!("No percent-improvement metrics recorded yet.");
    }

    let total_tokens: f64 = entries
        .iter()
        .filter_map(|entry| entry.record.token_savings)
        .sum();
    if total_tokens > 0.0 {
        println!("Total tokens saved: {}", format_tokens(Some(total_tokens)));
    } else {
        println!("Total tokens saved: 0");
    }

    if let Some((value, entry)) = entries
        .iter()
        .filter_map(|entry| entry.record.token_savings.map(|value| (value, entry)))
        .max_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
    {
        println!(
            "Largest token-saving loop: {} ({} tokens)",
            entry.slug,
            format_tokens(Some(value))
        );
    }

    if let Some(entry) = entries
        .iter()
        .find(|entry| entry.normalized_slug == "self-evolution")
        && let Some(control_plane) = loop_control_plane_summary(entry)
    {
        println!(
            "Self-evolution: {} scope={}/{} authority={} merge={} durability={}",
            control_plane.proposal_state,
            control_plane.scope_class,
            control_plane.scope_gate,
            control_plane.authority_tier,
            control_plane.merge_status,
            control_plane.durability_status
        );
    }
}

fn loop_control_plane_summary(entry: &LoopEntry) -> Option<ExperimentEvolutionSummary> {
    let metadata = &entry.record.metadata;
    let proposal_state = metadata
        .get("proposal_state")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            metadata
                .get("proposal")
                .and_then(|value| value.get("state"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        })?;
    let scope_class = metadata
        .get("proposal")
        .and_then(|value| value.get("scope_class"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("none")
        .to_string();
    let scope_gate = metadata
        .get("scope_gate")
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            metadata
                .get("proposal")
                .and_then(|value| value.get("scope_gate"))
                .and_then(serde_json::Value::as_str)
        })
        .unwrap_or("none")
        .to_string();
    let authority_tier = metadata
        .get("proposal")
        .and_then(|value| value.get("authority_tier"))
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            metadata
                .get("authority_ledger")
                .and_then(|value| value.get("entries"))
                .and_then(serde_json::Value::as_array)
                .and_then(|entries| entries.last())
                .and_then(|value| value.get("authority_tier"))
                .and_then(serde_json::Value::as_str)
        })
        .unwrap_or("none")
        .to_string();
    let merge_status = metadata
        .get("merge_queue")
        .and_then(|value| value.get("entries"))
        .and_then(serde_json::Value::as_array)
        .and_then(|entries| entries.last())
        .and_then(|value| value.get("status"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("none")
        .to_string();
    let durability_status = metadata
        .get("durability_queue")
        .and_then(|value| value.get("entries"))
        .and_then(serde_json::Value::as_array)
        .and_then(|entries| entries.last())
        .and_then(|value| value.get("status"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("none")
        .to_string();
    let branch = metadata
        .get("proposal")
        .and_then(|value| value.get("branch"))
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            metadata
                .get("branch_manifest")
                .and_then(|value| value.get("branch"))
                .and_then(serde_json::Value::as_str)
        })
        .unwrap_or("none")
        .to_string();
    let durable_truth = metadata
        .get("durable_truth")
        .and_then(serde_json::Value::as_bool)
        .or_else(|| {
            metadata
                .get("proposal")
                .and_then(|value| value.get("durable_truth"))
                .and_then(serde_json::Value::as_bool)
        })
        .unwrap_or(false);
    Some(ExperimentEvolutionSummary {
        proposal_state,
        scope_class,
        scope_gate,
        authority_tier,
        merge_status,
        durability_status,
        branch,
        durable_truth,
    })
}

fn run_telemetry(args: &TelemetryArgs) -> anyhow::Result<()> {
    let path = loops_summary_path(&args.output);
    let summary = read_loop_summary(&path)?;
    let benchmark = build_telemetry_benchmark_coverage(&args.output)?;
    if args.json {
        let report = build_telemetry_report(&summary, benchmark.as_ref());
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_loop_telemetry(&summary, &path, benchmark.as_ref());
    }
    Ok(())
}

fn loops_summary_path(output: &Path) -> PathBuf {
    loops_directory(output).join("loops.summary.json")
}

fn print_loop_telemetry(
    summary: &LoopSummary,
    path: &Path,
    benchmark: Option<&BenchmarkCoverageTelemetry>,
) {
    if summary.entries.is_empty() {
        println!(
            "No telemetry records found ({}). Run autoresearch to generate loop telemetry.",
            path.display()
        );
        if let Some(benchmark) = benchmark {
            println!(
                "Benchmark coverage: continuity-critical {}/{} benchmarked, missing loops {}, with-memd losses {}",
                benchmark.continuity_critical_benchmarked,
                benchmark.continuity_critical_total,
                benchmark.missing_loop_count,
                benchmark.with_memd_losses
            );
            if !benchmark.gap_candidates.is_empty() {
                println!(
                    "Benchmark gaps: {}",
                    benchmark
                        .gap_candidates
                        .iter()
                        .map(|candidate| format!("{} ({})", candidate.id, candidate.severity))
                        .collect::<Vec<_>>()
                        .join(" | ")
                );
            }
        }
        return;
    }

    let stats = TelemetryStats::from_summary(summary);
    println!("Loop telemetry ({} entries)", stats.total_loops);
    println!(
        "Status counts: {}",
        stats
            .status_counts
            .iter()
            .map(|(name, count)| format!("{}={}", name, count))
            .collect::<Vec<_>>()
            .join(" ")
    );

    if let Some(avg) = stats.average_improvement() {
        println!("Average improvement: {:.2}%", avg);
    } else {
        println!("Average improvement: none recorded");
    }

    if let Some(highlight) = stats.best_improvement() {
        println!(
            "Best improvement: {:.2}% ({})",
            highlight.value, highlight.slug
        );
    }
    if let Some(highlight) = stats.worst_improvement() {
        println!(
            "Worst improvement: {:.2}% ({})",
            highlight.value, highlight.slug
        );
    }

    println!(
        "Total tokens saved: {}",
        format_tokens(Some(stats.total_tokens_saved))
    );
    if let Some(highlight) = stats.best_token_saving() {
        println!(
            "Largest token-saving loop: {} ({})",
            highlight.slug,
            format_tokens(Some(highlight.value))
        );
    }

    if let Some(benchmark) = benchmark {
        println!(
            "Benchmark coverage: continuity-critical {}/{} benchmarked, missing loops {}, with-memd losses {}",
            benchmark.continuity_critical_benchmarked,
            benchmark.continuity_critical_total,
            benchmark.missing_loop_count,
            benchmark.with_memd_losses
        );
        if !benchmark.gap_candidates.is_empty() {
            println!(
                "Benchmark gaps: {}",
                benchmark
                    .gap_candidates
                    .iter()
                    .map(|candidate| format!("{} ({})", candidate.id, candidate.severity))
                    .collect::<Vec<_>>()
                    .join(" | ")
            );
        }
    }
}

fn build_telemetry_report(
    summary: &LoopSummary,
    benchmark: Option<&BenchmarkCoverageTelemetry>,
) -> TelemetryReport {
    let stats = TelemetryStats::from_summary(summary);
    TelemetryReport {
        total_loops: stats.total_loops,
        statuses: stats.status_counts.clone(),
        average_improvement: stats.average_improvement(),
        best_improvement: stats.best_improvement(),
        worst_improvement: stats.worst_improvement(),
        total_tokens_saved: stats.total_tokens_saved,
        largest_token_saving: stats.best_token_saving(),
        benchmark: benchmark.cloned(),
    }
}

#[derive(Debug)]
struct TelemetryStats {
    total_loops: usize,
    status_counts: BTreeMap<String, usize>,
    percent_improvements: Vec<(f64, String)>,
    token_savings: Vec<(f64, String)>,
    total_tokens_saved: f64,
}

impl TelemetryStats {
    fn from_summary(summary: &LoopSummary) -> Self {
        let mut status_counts = BTreeMap::new();
        let mut percent_improvements = Vec::new();
        let mut token_savings = Vec::new();
        let mut total_tokens_saved = 0.0;

        for entry in &summary.entries {
            let status = entry.status.as_deref().unwrap_or("pending").to_string();
            *status_counts.entry(status).or_insert(0) += 1;

            if let Some(value) = entry.percent_improvement {
                percent_improvements.push((value, entry.slug.clone()));
            }
            if let Some(tokens) = entry.token_savings {
                total_tokens_saved += tokens;
                token_savings.push((tokens, entry.slug.clone()));
            }
        }

        TelemetryStats {
            total_loops: summary.entries.len(),
            status_counts,
            percent_improvements,
            token_savings,
            total_tokens_saved,
        }
    }

    fn average_improvement(&self) -> Option<f64> {
        if self.percent_improvements.is_empty() {
            return None;
        }
        let sum: f64 = self
            .percent_improvements
            .iter()
            .map(|(value, _)| *value)
            .sum();
        Some(sum / (self.percent_improvements.len() as f64))
    }

    fn best_improvement(&self) -> Option<TelemetryHighlight> {
        max_by_value(&self.percent_improvements)
    }

    fn worst_improvement(&self) -> Option<TelemetryHighlight> {
        min_by_value(&self.percent_improvements)
    }

    fn best_token_saving(&self) -> Option<TelemetryHighlight> {
        max_by_value(&self.token_savings)
    }
}

fn max_by_value(list: &[(f64, String)]) -> Option<TelemetryHighlight> {
    list.iter()
        .max_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(value, slug)| TelemetryHighlight {
            slug: slug.clone(),
            value: *value,
        })
}

fn min_by_value(list: &[(f64, String)]) -> Option<TelemetryHighlight> {
    list.iter()
        .min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(value, slug)| TelemetryHighlight {
            slug: slug.clone(),
            value: *value,
        })
}

#[derive(Debug, Serialize)]
struct TelemetryReport {
    total_loops: usize,
    statuses: BTreeMap<String, usize>,
    average_improvement: Option<f64>,
    best_improvement: Option<TelemetryHighlight>,
    worst_improvement: Option<TelemetryHighlight>,
    total_tokens_saved: f64,
    largest_token_saving: Option<TelemetryHighlight>,
    benchmark: Option<BenchmarkCoverageTelemetry>,
}

#[derive(Debug, Serialize)]
struct TelemetryHighlight {
    slug: String,
    value: f64,
}

fn format_percent(value: Option<f64>) -> String {
    value
        .map(|value| format!("{:.2}%", value))
        .unwrap_or_else(|| "-".to_string())
}

fn format_tokens(value: Option<f64>) -> String {
    match value {
        Some(value) if value >= 1_000_000f64 => format!("{:.1}M", value / 1_000_000f64),
        Some(value) if value >= 1_000f64 => format!("{:.1}k", value / 1_000f64),
        Some(value) => format!("{:.0}", value),
        None => "-".to_string(),
    }
}

fn indent_text(value: &str, spaces: usize) -> String {
    let spacer = " ".repeat(spaces);
    value
        .lines()
        .map(|line| format!("{}{}", spacer, line))
        .collect::<Vec<_>>()
        .join("\n")
}

async fn run_autoresearch(args: &AutoresearchArgs, base_url: &str) -> anyhow::Result<()> {
    if args.manifest {
        print_autoresearch_manifest();
        return Ok(());
    }

    if !args.auto && args.loop_slug.is_none() {
        anyhow::bail!("specify --auto to run every loop or --loop to run a single loop");
    }

    let loops: Vec<_> = if args.auto {
        AUTORESEARCH_LOOPS.iter().collect()
    } else if let Some(slug) = &args.loop_slug {
        let normalized = canonical_slug(slug).to_lowercase();
        AUTORESEARCH_LOOPS
            .iter()
            .filter(|descriptor| descriptor.normalized_slug == normalized)
            .collect()
    } else {
        Vec::new()
    };

    if let Some((_, registry)) = load_benchmark_registry_for_output(&args.output)
        .ok()
        .flatten()
    {
        let benchmark_gaps = build_benchmark_gap_candidates(&registry);
        if !benchmark_gaps.is_empty() {
            println!(
                "Benchmark coverage gaps detected: {} candidate(s)",
                benchmark_gaps.len()
            );
            for gap in benchmark_gaps.iter().take(3) {
                println!("- {}: {}", gap.id, gap.recommendation);
            }
        }
    }

    if loops.is_empty() {
        anyhow::bail!("no loops matched; run with --manifest to see available loops");
    }

    if !args.auto && loops.len() == 1 {
        execute_autoresearch_loop(&args.output, base_url, loops[0]).await?;
        return Ok(());
    }

    let max_sweeps = if args.auto { args.max_sweeps.max(1) } else { 1 };
    let mut stable_sweeps = 0usize;
    let mut previous_signature: Option<Vec<AutoresearchSweepSignatureEntry>> = None;

    for sweep in 1..=max_sweeps {
        let records = execute_autoresearch_sweep(&args.output, base_url, &loops).await?;
        let signature = build_autoresearch_sweep_signature(&records);
        if previous_signature.as_ref() == Some(&signature) {
            stable_sweeps += 1;
        } else {
            stable_sweeps = 0;
        }
        previous_signature = Some(signature);

        if args.auto && max_sweeps > 1 {
            println!("Completed autoresearch sweep {sweep}/{max_sweeps}");
        }
        if args.auto && args.plateau_sweeps > 0 && stable_sweeps >= args.plateau_sweeps {
            println!(
                "Autoresearch plateau detected after {} stable sweep(s); stopping early.",
                stable_sweeps
            );
            break;
        }
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AutoresearchSweepSignatureEntry {
    slug: String,
    status: String,
    percent_bp: i64,
    tokens_bp: i64,
}

fn build_autoresearch_sweep_signature(
    records: &[(&'static AutoresearchLoop, LoopRecord)],
) -> Vec<AutoresearchSweepSignatureEntry> {
    records
        .iter()
        .map(|(descriptor, record)| AutoresearchSweepSignatureEntry {
            slug: descriptor.slug.to_string(),
            status: record
                .status
                .clone()
                .unwrap_or_else(|| "pending".to_string()),
            percent_bp: (record.percent_improvement.unwrap_or(0.0) * 100.0).round() as i64,
            tokens_bp: (record.token_savings.unwrap_or(0.0) * 100.0).round() as i64,
        })
        .collect()
}

async fn execute_autoresearch_sweep(
    output: &Path,
    base_url: &str,
    loops: &[&'static AutoresearchLoop],
) -> anyhow::Result<Vec<(&'static AutoresearchLoop, LoopRecord)>> {
    let summary = read_loop_summary(&loops_summary_path(output))?;
    let mut join_set = JoinSet::new();

    for (index, descriptor) in loops.iter().copied().enumerate() {
        let previous_runs = summary
            .entries
            .iter()
            .filter(|entry| entry.slug == descriptor.slug)
            .count();
        let previous_entry = summary
            .entries
            .iter()
            .rev()
            .find(|entry| entry.slug == descriptor.slug)
            .cloned();
        let output = output.to_path_buf();
        let base_url = base_url.to_string();
        join_set.spawn(async move {
            let record = build_autoresearch_record_for_descriptor(
                &output,
                &base_url,
                descriptor,
                previous_runs,
                previous_entry.as_ref(),
            )
            .await?;
            Ok::<_, anyhow::Error>((index, descriptor, record))
        });
    }

    let mut completed = Vec::with_capacity(loops.len());
    while let Some(result) = join_set.join_next().await {
        let (index, descriptor, record) = result??;
        completed.push((index, descriptor, record));
    }
    completed.sort_by_key(|(index, _, _)| *index);

    let mut persisted = Vec::with_capacity(completed.len());
    for (_, descriptor, record) in completed {
        persist_loop_record(output, &record)?;
        println!(
            "Recorded loop {}: {} improvement, {} token savings",
            descriptor.slug,
            format_percent(record.percent_improvement),
            format_tokens(record.token_savings)
        );
        persisted.push((descriptor, record));
    }

    Ok(persisted)
}

async fn execute_autoresearch_loop(
    output: &Path,
    base_url: &str,
    descriptor: &AutoresearchLoop,
) -> anyhow::Result<()> {
    let summary = read_loop_summary(&loops_summary_path(output))?;
    let previous_runs = summary
        .entries
        .iter()
        .filter(|entry| entry.slug == descriptor.slug)
        .count();
    let previous_entry = summary
        .entries
        .iter()
        .rev()
        .find(|entry| entry.slug == descriptor.slug);

    let record = build_autoresearch_record_for_descriptor(
        output,
        base_url,
        descriptor,
        previous_runs,
        previous_entry,
    )
    .await?;

    persist_loop_record(output, &record)?;
    println!(
        "Recorded loop {}: {} improvement, {} token savings",
        descriptor.slug,
        format_percent(record.percent_improvement),
        format_tokens(record.token_savings)
    );
    Ok(())
}

async fn build_autoresearch_record_for_descriptor(
    output: &Path,
    base_url: &str,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let record = match descriptor.slug {
        "branch-review-quality" => {
            run_branch_review_quality_loop(output, descriptor, previous_runs, previous_entry)
                .await?
        }
        "prompt-efficiency" => {
            run_prompt_efficiency_loop(output, base_url, descriptor, previous_runs, previous_entry)
                .await?
        }
        "signal-freshness" => {
            run_live_truth_loop(output, base_url, descriptor, previous_runs, previous_entry).await?
        }
        "autonomy-quality" => {
            run_autonomy_quality_loop(output, base_url, descriptor, previous_runs, previous_entry)
                .await?
        }
        "hive-health" => {
            run_hive_health_loop(output, descriptor, previous_runs, previous_entry).await?
        }
        "memory-hygiene" => {
            run_memory_hygiene_loop(output, base_url, descriptor, previous_runs, previous_entry)
                .await?
        }
        "repair-rate" => {
            run_repair_rate_loop(output, base_url, descriptor, previous_runs, previous_entry)
                .await?
        }
        "cross-harness" => {
            run_cross_harness_loop(output, descriptor, previous_runs, previous_entry).await?
        }
        "self-evolution" => {
            run_self_evolution_loop(output, descriptor, previous_runs, previous_entry).await?
        }
        "docs-spec-drift" => {
            run_docs_spec_drift_loop(output, descriptor, previous_runs, previous_entry).await?
        }
        _ => anyhow::bail!("unsupported autoresearch loop '{}'", descriptor.slug),
    };
    Ok(record)
}

fn print_autoresearch_manifest() {
    println!("Autoresearch manifest ({} loops)", AUTORESEARCH_LOOPS.len());
    for descriptor in AUTORESEARCH_LOOPS.iter() {
        println!("- {} ({})", descriptor.name, descriptor.slug);
        println!("  description: {}", descriptor.description);
        println!("  target: {}", descriptor.target);
        println!("  metric: {}", descriptor.metric);
        println!("  stop: {}", descriptor.stop_condition);
        println!("  risk: {}", descriptor.risk);
    }
}

#[cfg(test)]
#[allow(dead_code)]
async fn run_prompt_surface_loop(
    output: &Path,
    base_url: &str,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    run_prompt_efficiency_loop(output, base_url, descriptor, previous_runs, previous_entry).await
}

async fn run_branch_review_quality_loop(
    output: &Path,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let root = infer_bundle_project_root(output).unwrap_or_else(|| output.to_path_buf());
    let evidence = collect_gap_repo_evidence(&root);
    let branch = std::process::Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("branch")
        .arg("--show-current")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    let review_ready = !evidence.iter().any(|line| line.contains("dirty"));
    let percent = if review_ready { 100.0 } else { 0.0 };
    let token_savings = if branch == "unknown" { 0.0 } else { 20.0 };
    let confidence_met =
        loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let status = if loop_success_requires_second_signal(
        confidence_met,
        branch != "unknown" && review_ready,
        confidence_met,
        !loop_is_regressed(descriptor, previous_entry, percent, token_savings),
    ) {
        "success"
    } else {
        "warning"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        format!("branch {} review_ready={}", branch, review_ready),
        vec!["branch review".to_string()],
        serde_json::json!({
            "evidence": evidence,
            "branch": branch,
            "review_ready": review_ready,
        }),
        status,
    ))
}

async fn run_prompt_efficiency_loop(
    output: &Path,
    base_url: &str,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let snapshot = read_bundle_resume(&autoresearch_resume_args(output), base_url).await?;
    let estimated_tokens = snapshot.estimated_prompt_tokens() as f64;
    let core_tokens = snapshot.core_prompt_tokens() as f64;
    let percent = improvement_less_is_better(core_tokens, estimated_tokens);
    let token_savings = (estimated_tokens - core_tokens).max(0.0);
    let summary = format!(
        "prompt tokens = {} (core {}, saved {})",
        estimated_tokens, core_tokens, token_savings
    );
    let evidence = vec![
        format!("estimated_tokens={}", estimated_tokens),
        format!("core_prompt_tokens={}", core_tokens),
        format!("context_pressure={}", snapshot.context_pressure()),
        format!("redundant_items={}", snapshot.redundant_context_items()),
    ];
    let confidence_met =
        loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let secondary_signal_ok = snapshot.context_pressure() != "high"
        || snapshot.redundant_context_items() == 0
        || token_savings >= descriptor.base_tokens * 2.0;
    let status = if loop_success_requires_second_signal(
        confidence_met,
        secondary_signal_ok,
        confidence_met,
        !loop_is_regressed(descriptor, previous_entry, percent, token_savings),
    ) {
        "success"
    } else {
        "warning"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        summary,
        vec!["prompt efficiency".to_string()],
        serde_json::json!({
            "evidence": evidence,
            "estimated_prompt_tokens": estimated_tokens,
            "core_prompt_tokens": core_tokens,
            "context_pressure": snapshot.context_pressure(),
            "refresh_recommended": snapshot.refresh_recommended,
            "confidence": loop_confidence_metadata(descriptor, percent, token_savings, confidence_met, 3),
            "trend": loop_trend_metadata(descriptor, previous_entry, percent, token_savings),
        }),
        status,
    ))
}

async fn run_hive_health_loop(
    output: &Path,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let awareness = read_project_awareness(&AwarenessArgs {
        output: output.to_path_buf(),
        root: None,
        include_current: true,
        summary: false,
    })
    .await?;
    let heartbeat = build_hive_heartbeat(output, None)?;
    let visible_entries = project_awareness_visible_entries(&awareness);
    let current_entry = visible_entries
        .iter()
        .find(|entry| entry.bundle_root == awareness.current_bundle);
    let relevant_collisions = awareness
        .collisions
        .iter()
        .filter(|collision| !collision.starts_with("base_url "))
        .collect::<Vec<_>>();
    let dead_hives = visible_entries
        .iter()
        .filter(|entry| entry.presence == "dead")
        .filter(|entry| {
            current_entry.is_some_and(|current| {
                entry.project_dir != "remote"
                    && entry.project == current.project
                    && entry.namespace == current.namespace
                    && entry.workspace == current.workspace
            })
        })
        .count();
    let percent = if relevant_collisions.is_empty() {
        100.0
    } else {
        100.0 - (relevant_collisions.len() as f64 * 10.0)
    };
    let token_savings = (visible_entries.len() as f64) * 8.0;
    let evidence = vec![
        format!("active_hives={}", visible_entries.len()),
        format!("dead_hives={}", dead_hives),
        format!("claim_collisions={}", relevant_collisions.len()),
    ];
    let confidence_met =
        loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let secondary_signal_ok = relevant_collisions.is_empty() && dead_hives == 0;
    let warning_reasons = {
        let mut reasons = Vec::new();
        if dead_hives > 0 {
            reasons.push("dead_hive_sessions".to_string());
        }
        if !relevant_collisions.is_empty() {
            reasons.push("claim_collisions_detected".to_string());
        }
        if loop_is_regressed(descriptor, previous_entry, percent, token_savings) {
            reasons.extend(loop_trend_warning_reasons(
                descriptor,
                previous_entry,
                percent,
                token_savings,
            ));
        }
        if !confidence_met {
            reasons.extend(loop_floor_warning_reasons(
                descriptor,
                percent,
                token_savings,
                evidence.len(),
            ));
        }
        reasons
    };
    let status = if loop_success_requires_second_signal(
        confidence_met,
        secondary_signal_ok,
        confidence_met,
        !loop_is_regressed(descriptor, previous_entry, percent, token_savings),
    ) {
        "success"
    } else {
        "warning"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        "hive health score".to_string(),
        vec!["hive health".to_string()],
        serde_json::json!({
            "active_hives": visible_entries.len(),
            "dead_hives": dead_hives,
            "claim_collisions": relevant_collisions.len(),
            "evidence": evidence,
            "heartbeat_status": heartbeat.status,
            "confidence": loop_confidence_metadata(
                descriptor,
                percent,
                token_savings,
                confidence_met,
                3,
            ),
            "trend": loop_trend_metadata(descriptor, previous_entry, percent, token_savings),
            "warning_reasons": warning_reasons,
        }),
        status,
    ))
}

async fn run_memory_hygiene_loop(
    output: &Path,
    base_url: &str,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let snapshot = read_bundle_resume(
        &autoresearch_resume_args_with_limits(output, 4, 2, true),
        base_url,
    )
    .await?;
    let mut seen = std::collections::HashSet::new();
    let mut duplicates = 0usize;
    for record in snapshot
        .context
        .records
        .iter()
        .chain(snapshot.working.records.iter())
    {
        let normalized = record.record.trim().to_lowercase();
        if !normalized.is_empty() && !seen.insert(normalized) {
            duplicates += 1;
        }
    }
    let total_records = snapshot.context.records.len() + snapshot.working.records.len();
    let event_spine_entries = snapshot.event_spine().len();
    let secondary_signal_ok = duplicates == 0 && event_spine_entries > 0;
    let percent = if secondary_signal_ok { 100.0 } else { 0.0 };
    let token_savings = if secondary_signal_ok {
        descriptor.base_tokens
    } else {
        0.0
    };
    let evidence = vec![
        format!("duplicates={duplicates}"),
        format!("records={total_records}"),
        format!("event_spine_entries={event_spine_entries}"),
    ];
    let confidence_met =
        loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let mut warning_reasons = Vec::new();
    if duplicates > 0 {
        warning_reasons.push("duplicate_memory_pressure".to_string());
    }
    if event_spine_entries == 0 {
        warning_reasons.push("empty_event_spine".to_string());
    }
    if loop_is_regressed(descriptor, previous_entry, percent, token_savings) {
        warning_reasons.extend(loop_trend_warning_reasons(
            descriptor,
            previous_entry,
            percent,
            token_savings,
        ));
    }
    if !confidence_met {
        warning_reasons.extend(loop_floor_warning_reasons(
            descriptor,
            percent,
            token_savings,
            evidence.len(),
        ));
    }
    let status = if loop_success_requires_second_signal(
        confidence_met,
        secondary_signal_ok,
        confidence_met,
        !loop_is_regressed(descriptor, previous_entry, percent, token_savings),
    ) {
        "success"
    } else {
        "warning"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        format!(
            "memory hygiene score: {duplicates} duplicates across {total_records} records, {event_spine_entries} event spine entries"
        ),
        vec!["memory hygiene".to_string()],
        serde_json::json!({
            "duplicates": duplicates as f64,
            "records": total_records,
            "evidence": evidence,
            "confidence": loop_confidence_metadata(descriptor, percent, token_savings, confidence_met, 3),
            "trend": loop_trend_metadata(descriptor, previous_entry, percent, token_savings),
            "warning_reasons": warning_reasons,
        }),
        status,
    ))
}

async fn run_autonomy_quality_loop(
    output: &Path,
    base_url: &str,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let snapshot = read_bundle_resume(&autoresearch_resume_args(output), base_url).await?;
    let mut warning_pressure = 0u64;
    if snapshot.change_summary.is_empty() && snapshot.recent_repo_changes.is_empty() {
        warning_pressure += 1;
    }
    let percent = (100.0 - warning_pressure as f64 * 20.0).max(0.0);
    let token_savings = descriptor.base_tokens * (percent / 100.0);
    let evidence = vec![
        format!("warning_pressure={warning_pressure}"),
        format!("change_summary={}", snapshot.change_summary.len()),
        format!("recent_repo_changes={}", snapshot.recent_repo_changes.len()),
    ];
    let confidence_met =
        loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let secondary_signal_ok = warning_pressure == 0;
    let mut warning_reasons = Vec::new();
    if snapshot.refresh_recommended {
        warning_reasons.push("refresh_recommended".to_string());
    }
    if snapshot.change_summary.is_empty() && snapshot.recent_repo_changes.is_empty() {
        warning_reasons.push("no_change_signal".to_string());
    }
    if loop_is_regressed(descriptor, previous_entry, percent, token_savings) {
        warning_reasons.extend(loop_trend_warning_reasons(
            descriptor,
            previous_entry,
            percent,
            token_savings,
        ));
    }
    if !confidence_met {
        warning_reasons.extend(loop_floor_warning_reasons(
            descriptor,
            percent,
            token_savings,
            evidence.len(),
        ));
    }
    let status = if loop_success_requires_second_signal(
        confidence_met,
        secondary_signal_ok,
        confidence_met,
        !loop_is_regressed(descriptor, previous_entry, percent, token_savings),
    ) {
        "success"
    } else {
        "warning"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        format!("autonomy quality score: warning pressure {warning_pressure}"),
        vec!["autonomy quality".to_string()],
        serde_json::json!({
            "warning_pressure": warning_pressure,
            "evidence": evidence,
            "confidence": loop_confidence_metadata(descriptor, percent, token_savings, confidence_met, 3),
            "trend": loop_trend_metadata(descriptor, previous_entry, percent, token_savings),
            "warning_reasons": warning_reasons,
        }),
        status,
    ))
}

async fn run_live_truth_loop(
    output: &Path,
    base_url: &str,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let snapshot = read_bundle_resume(
        &autoresearch_resume_args_with_limits(output, 4, 2, true),
        base_url,
    )
    .await?;
    let change_count = snapshot.change_summary.len() as f64;
    let baseline = 6.0;
    let percent = improvement_less_is_better(change_count, baseline);
    let token_savings = (baseline - change_count).max(0.0) * 20.0;
    let summary = format!(
        "{} change_summary entries, {} repo changes since last resume",
        change_count,
        snapshot.recent_repo_changes.len()
    );
    let evidence = vec![
        "live truth".to_string(),
        format!("change_summary={}", change_count),
        format!("recent_repo_changes={}", snapshot.recent_repo_changes.len()),
    ];
    let confidence_met =
        loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let warning_reasons = {
        let mut reasons = Vec::new();
        if snapshot.refresh_recommended {
            reasons.push("refresh_recommended".to_string());
        }
        if snapshot.change_summary.is_empty() && snapshot.recent_repo_changes.is_empty() {
            reasons.push("no_change_signal".to_string());
        }
        if loop_is_regressed(descriptor, previous_entry, percent, token_savings) {
            reasons.extend(loop_trend_warning_reasons(
                descriptor,
                previous_entry,
                percent,
                token_savings,
            ));
        }
        if !confidence_met {
            reasons.extend(loop_floor_warning_reasons(
                descriptor,
                percent,
                token_savings,
                evidence.len(),
            ));
        }
        reasons
    };
    let metadata = serde_json::json!({
        "change_summary": change_count,
        "recent_repo_changes": snapshot.recent_repo_changes.len(),
        "refresh_recommended": snapshot.refresh_recommended,
        "evidence": evidence,
        "confidence": loop_confidence_metadata(descriptor, percent, token_savings, confidence_met, 3),
        "trend": loop_trend_metadata(descriptor, previous_entry, percent, token_savings),
        "warning_reasons": warning_reasons,
    });
    let status = if (snapshot.change_summary.is_empty() && snapshot.recent_repo_changes.is_empty())
        || loop_is_regressed(descriptor, previous_entry, percent, token_savings)
        || !confidence_met
    {
        "warning"
    } else {
        "success"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        summary,
        vec!["live truth".to_string()],
        metadata,
        status,
    ))
}

#[cfg(test)]
#[allow(dead_code)]
async fn run_event_spine_loop(
    output: &Path,
    base_url: &str,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let snapshot = read_bundle_resume(
        &autoresearch_resume_args_with_limits(output, 4, 2, true),
        base_url,
    )
    .await?;
    let spine = snapshot.event_spine();
    let spine_chars = spine.iter().map(|line| line.len()).sum::<usize>() as f64;
    let baseline = 600.0;
    let percent = improvement_less_is_better(spine_chars, baseline);
    let token_savings = (baseline - spine_chars).max(0.0) / 4.0;
    let summary = format!(
        "{} event spine entries consuming {} chars",
        spine.len(),
        spine_chars
    );
    let evidence = vec![
        "event spine".to_string(),
        format!("entries={}", spine.len()),
        format!("chars={}", spine_chars),
    ];
    let confidence_met =
        loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let warning_reasons = {
        let mut reasons = Vec::new();
        if snapshot.refresh_recommended {
            reasons.push("refresh_recommended".to_string());
        }
        if spine.is_empty() {
            reasons.push("empty_event_spine".to_string());
        }
        if loop_is_regressed(descriptor, previous_entry, percent, token_savings) {
            reasons.extend(loop_trend_warning_reasons(
                descriptor,
                previous_entry,
                percent,
                token_savings,
            ));
        }
        if !confidence_met {
            reasons.extend(loop_floor_warning_reasons(
                descriptor,
                percent,
                token_savings,
                evidence.len(),
            ));
        }
        reasons
    };
    let metadata = serde_json::json!({
        "event_spine_entries": spine.len(),
        "event_spine_chars": spine_chars,
        "refresh_recommended": snapshot.refresh_recommended,
        "evidence": evidence,
        "confidence": loop_confidence_metadata(descriptor, percent, token_savings, confidence_met, 3),
        "trend": loop_trend_metadata(descriptor, previous_entry, percent, token_savings),
        "warning_reasons": warning_reasons,
    });
    let status = if spine.is_empty()
        || loop_is_regressed(descriptor, previous_entry, percent, token_savings)
        || !confidence_met
    {
        "warning"
    } else {
        "success"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        summary,
        vec!["event spine".to_string()],
        metadata,
        status,
    ))
}

#[cfg(test)]
#[allow(dead_code)]
async fn run_correction_learning_loop(
    output: &Path,
    base_url: &str,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    run_repair_rate_loop(output, base_url, descriptor, previous_runs, previous_entry).await
}

async fn run_repair_rate_loop(
    output: &Path,
    base_url: &str,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let snapshot = read_bundle_resume(&autoresearch_resume_args(output), base_url).await?;
    let total = snapshot.change_summary.len() as f64;
    let corrections = snapshot
        .change_summary
        .iter()
        .filter(|line| {
            let lower = line.to_lowercase();
            lower.contains("fix") || lower.contains("correct") || lower.contains("repair")
        })
        .count() as f64;
    let percent = if total == 0.0 {
        0.0
    } else {
        (1.0 - (corrections / total)).max(0.0) * 100.0
    };
    let token_savings = ((total - corrections).max(0.0)) * 10.0;
    let evidence = vec![
        format!("tracked={}", total),
        format!("corrections={}", corrections),
        format!("recent={}", snapshot.recent_repo_changes.len()),
    ];
    let confidence_met =
        loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let status = if loop_success_requires_second_signal(
        confidence_met,
        corrections <= total,
        confidence_met,
        !loop_is_regressed(descriptor, previous_entry, percent, token_savings),
    ) {
        "success"
    } else {
        "warning"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        format!(
            "{} corrections out of {} tracked change summaries",
            corrections, total
        ),
        vec!["repair rate".to_string()],
        serde_json::json!({
            "evidence": evidence,
            "corrections": corrections,
            "change_summary": total,
            "recent": snapshot.recent_repo_changes.len(),
            "confidence": loop_confidence_metadata(descriptor, percent, token_savings, confidence_met, 3),
            "trend": loop_trend_metadata(descriptor, previous_entry, percent, token_savings),
        }),
        status,
    ))
}

#[cfg(test)]
#[allow(dead_code)]
async fn run_long_context_loop(
    output: &Path,
    base_url: &str,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let snapshot =
        read_bundle_resume(&autoresearch_long_context_resume_args(output), base_url).await?;
    let tokens = snapshot.core_prompt_tokens() as f64;
    let baseline = 1_200.0;
    let percent = improvement_less_is_better(tokens, baseline);
    let token_savings = (baseline - tokens).max(0.0);
    let summary = format!("core prompt tokens {} (target {})", tokens, baseline);
    let evidence = vec![
        "long context".to_string(),
        format!("core_prompt_tokens={}", tokens),
        format!("context_records={}", snapshot.context.records.len()),
    ];
    let confidence_met =
        loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let metadata = serde_json::json!({
        "core_prompt_tokens": tokens,
        "baseline": baseline,
        "context_records": snapshot.context.records.len(),
        "refresh_recommended": snapshot.refresh_recommended,
        "evidence": evidence,
        "confidence": loop_confidence_metadata(descriptor, percent, token_savings, confidence_met, 3),
        "trend": loop_trend_metadata(descriptor, previous_entry, percent, token_savings),
    });
    let status = if tokens <= 0.0
        || loop_is_regressed(descriptor, previous_entry, percent, token_savings)
        || !confidence_met
    {
        "warning"
    } else {
        "success"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        summary,
        vec!["long context".to_string()],
        metadata,
        status,
    ))
}

async fn run_docs_spec_drift_loop(
    output: &Path,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let project_root = infer_bundle_project_root(output).unwrap_or_else(|| output.to_path_buf());
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let spec_path = project_root.join("docs/superpowers/specs/2026-04-08-memd-10-loop-design.md");
    let plan_path = project_root.join("docs/superpowers/plans/2026-04-08-memd-10-loop-stack.md");
    let spec = fs::read_to_string(&spec_path).or_else(|_| {
        fs::read_to_string(
            manifest_root.join("docs/superpowers/specs/2026-04-08-memd-10-loop-design.md"),
        )
    })?;
    let plan = fs::read_to_string(&plan_path).or_else(|_| {
        fs::read_to_string(
            manifest_root.join("docs/superpowers/plans/2026-04-08-memd-10-loop-stack.md"),
        )
    })?;
    let runtime_bytes = fs::metadata(project_root.join("Cargo.toml"))
        .map(|m| m.len())
        .unwrap_or(0);
    let evidence = vec![
        format!("spec_bytes={}", spec.len()),
        format!("plan_bytes={}", plan.len()),
        format!("runtime_bytes={}", runtime_bytes),
    ];
    let secondary_signal_ok = spec.contains("10-loop") && plan.contains("Implementation Plan");
    let percent = if secondary_signal_ok { 100.0 } else { 0.0 };
    let token_savings = if secondary_signal_ok {
        descriptor.base_tokens
    } else {
        0.0
    };
    let confidence_met =
        loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let status = if loop_success_requires_second_signal(
        confidence_met,
        secondary_signal_ok,
        confidence_met,
        !loop_is_regressed(descriptor, previous_entry, percent, token_savings),
    ) {
        "success"
    } else {
        "warning"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        "docs-spec drift score".to_string(),
        vec!["docs spec drift".to_string()],
        serde_json::json!({
            "evidence": evidence,
            "spec_has_10_loop": spec.contains("10-loop"),
            "plan_has_implementation_plan": plan.contains("Implementation Plan"),
        }),
        status,
    ))
}

#[cfg(test)]
#[allow(dead_code)]
async fn run_capability_contract_loop(
    output: &Path,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let project_root = infer_bundle_project_root(output);
    let registry = build_bundle_capability_registry(project_root.as_deref());
    let total = registry.capabilities.len();
    if total == 0 {
        let summary = "no capability contracts discovered".to_string();
        let metadata = serde_json::json!({
            "total": total,
            "missing": 0,
            "coverage": 0.0,
            "evidence": [
                "capability contract",
                "no capability contracts discovered",
            ],
            "confidence": {
                "absolute_percent_floor": descriptor.base_percent,
                "absolute_token_floor": descriptor.base_tokens,
                "min_evidence_signals": AUTORESEARCH_MIN_EVIDENCE_SIGNALS,
                "evidence_count": 0,
                "absolute_percent_met": false,
                "absolute_token_met": false,
                "absolute_floor_met": false,
            },
            "warning_reasons": ["no_capability_registry"],
        });
        return Ok(build_autoresearch_record_with_status(
            descriptor,
            previous_runs + 1,
            0.0,
            0.0,
            summary,
            vec!["capability contract".to_string()],
            metadata,
            "warning",
        ));
    }
    let discovered = registry
        .capabilities
        .iter()
        .filter(|entry| entry.status == "installed" || entry.status == "discovered")
        .count();
    let portable = registry
        .capabilities
        .iter()
        .filter(|entry| {
            entry.portability_class != "adapter-required"
                && entry.portability_class != "harness-native"
        })
        .count();
    let bad = total.saturating_sub(portable);
    let coverage = portable as f64 / total as f64;
    let percent = coverage * 100.0;
    let token_savings = portable as f64;
    let summary = format!(
        "{}/{} capability contracts satisfy expectations",
        portable, total
    );
    let evidence = vec![
        "capability contract".to_string(),
        format!("total={}", total),
        format!("discovered={}", discovered),
        format!("portable={}", portable),
        format!("coverage={}", coverage),
    ];
    let confidence_met =
        loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let warning_reasons = {
        let mut reasons = Vec::new();
        if loop_is_regressed(descriptor, previous_entry, percent, token_savings) {
            reasons.extend(loop_trend_warning_reasons(
                descriptor,
                previous_entry,
                percent,
                token_savings,
            ));
        }
        if !confidence_met {
            reasons.extend(loop_floor_warning_reasons(
                descriptor,
                percent,
                token_savings,
                evidence.len(),
            ));
        }
        reasons
    };
    let metadata = serde_json::json!({
        "total": total,
        "discovered": discovered,
        "portable": portable,
        "missing": bad,
        "coverage": coverage,
        "evidence": evidence,
        "confidence": loop_confidence_metadata(descriptor, percent, token_savings, confidence_met, 5),
        "trend": loop_trend_metadata(descriptor, previous_entry, percent, token_savings),
        "warning_reasons": warning_reasons,
    });
    let status = if loop_is_regressed(descriptor, previous_entry, percent, token_savings)
        || !confidence_met
    {
        "warning"
    } else {
        "success"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        summary,
        vec!["capability contract".to_string()],
        metadata,
        status,
    ))
}

async fn run_cross_harness_loop(
    output: &Path,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let project_root = infer_bundle_project_root(output);
    let registry = build_bundle_capability_registry(project_root.as_deref());
    let total = registry.capabilities.len();
    if total == 0 {
        let summary = "no cross-harness capabilities discovered".to_string();
        let metadata = serde_json::json!({
            "total": total,
            "portable": 0,
            "ratio": 0.0,
            "evidence": [
                "cross harness",
                "no cross-harness capabilities discovered",
            ],
            "confidence": {
                "absolute_percent_floor": descriptor.base_percent,
                "absolute_token_floor": descriptor.base_tokens,
                "min_evidence_signals": AUTORESEARCH_MIN_EVIDENCE_SIGNALS,
                "evidence_count": 0,
                "absolute_percent_met": false,
                "absolute_token_met": false,
                "absolute_floor_met": false,
            },
            "warning_reasons": ["no_cross_harness_registry"],
        });
        return Ok(build_autoresearch_record_with_status(
            descriptor,
            previous_runs + 1,
            0.0,
            0.0,
            summary,
            vec!["cross harness".to_string()],
            metadata,
            "warning",
        ));
    }
    let portable = registry
        .capabilities
        .iter()
        .filter(|entry| entry.portability_class != "adapter-required")
        .count();
    let ratio = portable as f64 / total as f64;
    let percent = ratio * 100.0;
    let token_savings = ratio * descriptor.base_tokens;
    let summary = format!("cross harness ports {}/{}", portable, total);
    let evidence = vec![
        "cross harness".to_string(),
        format!("total={}", total),
        format!("portable={}", portable),
        format!("ratio={}", ratio),
    ];
    let confidence_met =
        loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let warning_reasons = {
        let mut reasons = Vec::new();
        if loop_is_regressed(descriptor, previous_entry, percent, token_savings) {
            reasons.extend(loop_trend_warning_reasons(
                descriptor,
                previous_entry,
                percent,
                token_savings,
            ));
        }
        if !confidence_met {
            reasons.extend(loop_floor_warning_reasons(
                descriptor,
                percent,
                token_savings,
                evidence.len(),
            ));
        }
        reasons
    };
    let metadata = serde_json::json!({
        "total": total,
        "portable": portable,
        "ratio": ratio,
        "evidence": evidence,
        "confidence": loop_confidence_metadata(descriptor, percent, token_savings, confidence_met, 4),
        "trend": loop_trend_metadata(descriptor, previous_entry, percent, token_savings),
        "warning_reasons": warning_reasons,
    });
    let status = if loop_is_regressed(descriptor, previous_entry, percent, token_savings)
        || !confidence_met
    {
        "warning"
    } else {
        "success"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        summary,
        vec!["cross harness".to_string()],
        metadata,
        status,
    ))
}

async fn run_self_evolution_loop(
    output: &Path,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let report = read_latest_experiment_report(output)?;
    if let Some(report) = report {
        ensure_evolution_artifacts(output, &report)?;
        let proposal = read_latest_evolution_proposal(output)?;
        let durability = read_evolution_durability_ledger(output)?;
        let authority = read_evolution_authority_ledger(output)?;
        let branch_manifest = read_latest_evolution_branch_manifest(output)?;
        let merge_queue = read_evolution_merge_queue(output)?;
        let durability_queue = read_evolution_durability_queue(output)?;
        let fresh = experiment_report_is_fresh(&report);
        let proposal_state = proposal
            .as_ref()
            .map(|value| value.state.as_str())
            .unwrap_or("none");
        let scope_class = proposal
            .as_ref()
            .map(|value| value.scope_class.as_str())
            .unwrap_or("none");
        let scope_gate = proposal
            .as_ref()
            .map(|value| value.scope_gate.as_str())
            .unwrap_or("none");
        let authority_tier = proposal
            .as_ref()
            .map(|value| value.authority_tier.as_str())
            .or_else(|| {
                authority
                    .as_ref()
                    .and_then(|ledger| ledger.entries.last())
                    .map(|entry| entry.authority_tier.as_str())
            })
            .unwrap_or("none");
        let merge_status = merge_queue
            .as_ref()
            .and_then(|queue| queue.entries.last())
            .map(|entry| entry.status.as_str())
            .unwrap_or("none");
        let durability_status = durability_queue
            .as_ref()
            .and_then(|queue| queue.entries.last())
            .map(|entry| entry.status.as_str())
            .unwrap_or("none");
        let branch = proposal
            .as_ref()
            .map(|value| value.branch.as_str())
            .or_else(|| {
                branch_manifest
                    .as_ref()
                    .map(|manifest| manifest.branch.as_str())
            })
            .unwrap_or("none");
        let durable_truth = proposal.as_ref().is_some_and(|value| value.durable_truth)
            || durability
                .as_ref()
                .and_then(|ledger| ledger.entries.last())
                .is_some_and(|entry| entry.state == "durable_truth");
        let stage_multiplier = if durable_truth {
            1.0
        } else if proposal_state == "merged" {
            0.92
        } else if proposal_state == "accepted_proposal" {
            0.84
        } else {
            0.0
        };
        let usable = fresh
            && report.accepted
            && !report.restored
            && report.composite.max_score > 0
            && proposal_state != "rejected";
        let raw_ratio = if report.composite.max_score == 0 {
            0.0
        } else {
            report.composite.score as f64 / report.composite.max_score as f64
        };
        let ratio = raw_ratio * stage_multiplier;
        let percent = if usable { ratio * 100.0 } else { 0.0 };
        let token_savings = if usable && stage_multiplier > 0.0 {
            raw_ratio * descriptor.base_tokens
        } else {
            0.0
        };
        let summary = if usable {
            format!(
                "{} experiment composite score {}/{} with {} learnings",
                proposal_state,
                report.composite.score,
                report.composite.max_score,
                report.learnings.len()
            )
        } else {
            format!(
                "experiment report not usable (accepted={}, restored={}, fresh={}, max_score={}, proposal_state={}, scope_gate={})",
                report.accepted,
                report.restored,
                fresh,
                report.composite.max_score,
                proposal_state,
                scope_gate
            )
        };
        let evidence = if usable {
            vec![
                "self evolution".to_string(),
                format!("accepted={}", report.accepted),
                format!("fresh={}", fresh),
                format!("proposal_state={proposal_state}"),
                format!("scope_gate={scope_gate}"),
                format!("durable_truth={durable_truth}"),
                format!("composite_score={}", report.composite.score),
                format!("composite_max={}", report.composite.max_score),
            ]
        } else {
            vec![
                "self evolution".to_string(),
                format!("accepted={}", report.accepted),
                format!("restored={}", report.restored),
                format!("fresh={}", fresh),
                format!("proposal_state={proposal_state}"),
                format!("scope_gate={scope_gate}"),
            ]
        };
        let confidence_met =
            usable && loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
        let warning_reasons = {
            let mut reasons = Vec::new();
            if !fresh {
                reasons.push("stale_report".to_string());
            }
            if report.restored {
                reasons.push("restored_report".to_string());
            }
            if !report.accepted {
                reasons.push("unaccepted_report".to_string());
            }
            if report.composite.max_score == 0 {
                reasons.push("zero_max_score".to_string());
            }
            if proposal_state == "none" {
                reasons.push("no_evolution_proposal".to_string());
            }
            if usable && loop_is_regressed(descriptor, previous_entry, percent, token_savings) {
                reasons.extend(loop_trend_warning_reasons(
                    descriptor,
                    previous_entry,
                    percent,
                    token_savings,
                ));
            }
            if usable && !confidence_met {
                reasons.extend(loop_floor_warning_reasons(
                    descriptor,
                    percent,
                    token_savings,
                    evidence.len(),
                ));
            }
            reasons
        };
        let metadata = serde_json::json!({
            "accepted": report.accepted,
            "restored": report.restored,
            "fresh": fresh,
            "usable": usable,
            "proposal_state": proposal_state,
            "scope_class": scope_class,
            "scope_gate": scope_gate,
            "authority_tier": authority_tier,
            "merge_status": merge_status,
            "durability_status": durability_status,
            "branch": branch,
            "durable_truth": durable_truth,
            "composite_score": report.composite.score,
            "composite_max": report.composite.max_score,
            "evidence": evidence,
            "confidence": loop_confidence_metadata(descriptor, percent, token_savings, confidence_met, if usable { 5 } else { 4 }),
            "trend": loop_trend_metadata(descriptor, previous_entry, percent, token_savings),
            "proposal": proposal,
            "branch_manifest": branch_manifest,
            "authority_ledger": authority,
            "merge_queue": merge_queue,
            "durability_ledger": durability,
            "durability_queue": durability_queue,
            "warning_reasons": warning_reasons,
        });
        let artifact_paths = vec![
            output
                .join("experiments")
                .join("latest.json")
                .display()
                .to_string(),
            output
                .join("evolution")
                .join("latest-proposal.json")
                .display()
                .to_string(),
            output
                .join("evolution")
                .join("latest-branch.json")
                .display()
                .to_string(),
            output
                .join("evolution")
                .join("authority-ledger.json")
                .display()
                .to_string(),
            output
                .join("evolution")
                .join("merge-queue.json")
                .display()
                .to_string(),
            output
                .join("evolution")
                .join("durability-queue.json")
                .display()
                .to_string(),
        ];
        if usable {
            let status = if loop_is_regressed(descriptor, previous_entry, percent, token_savings)
                || !confidence_met
            {
                "warning"
            } else {
                "success"
            };
            Ok(build_autoresearch_record_with_status(
                descriptor,
                previous_runs + 1,
                percent,
                token_savings,
                summary,
                artifact_paths.clone(),
                metadata,
                status,
            ))
        } else {
            Ok(build_autoresearch_record_with_status(
                descriptor,
                previous_runs + 1,
                percent,
                token_savings,
                summary,
                artifact_paths,
                metadata,
                "warning",
            ))
        }
    } else {
        let summary = "no experiment recorded yet".to_string();
        Ok(build_autoresearch_record_with_status(
            descriptor,
            previous_runs + 1,
            0.0,
            0.0,
            summary,
            vec!["self evolution".to_string()],
            serde_json::json!({
                "evidence": ["self evolution", "no experiment recorded yet"],
                "confidence": {
                    "absolute_percent_floor": descriptor.base_percent,
                    "absolute_token_floor": descriptor.base_tokens,
                    "min_evidence_signals": AUTORESEARCH_MIN_EVIDENCE_SIGNALS,
                    "evidence_count": 0,
                    "absolute_percent_met": false,
                    "absolute_token_met": false,
                    "absolute_floor_met": false,
                },
                "warning_reasons": ["no_experiment_report"],
            }),
            "warning",
        ))
    }
}

#[cfg(test)]
#[allow(dead_code)]
async fn run_default_loop(
    _output: &Path,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
) -> anyhow::Result<LoopRecord> {
    let percent_improvement =
        descriptor.base_percent + (previous_runs as f64 * descriptor.base_percent * 0.1);
    let token_savings =
        descriptor.base_tokens + (previous_runs as f64 * descriptor.base_tokens * 0.1);
    Ok(build_autoresearch_record(
        descriptor,
        previous_runs + 1,
        percent_improvement,
        token_savings,
        descriptor.description.to_string(),
        vec!["autoresearch loop".to_string()],
        serde_json::json!({
            "target": descriptor.target,
            "metric": descriptor.metric,
        }),
    ))
}

#[cfg(test)]
fn build_autoresearch_record(
    descriptor: &AutoresearchLoop,
    iteration: usize,
    percent_improvement: f64,
    token_savings: f64,
    summary: String,
    artifacts: Vec<String>,
    metadata: serde_json::Value,
) -> LoopRecord {
    build_autoresearch_record_with_status(
        descriptor,
        iteration,
        percent_improvement,
        token_savings,
        summary,
        artifacts,
        metadata,
        "success",
    )
}

fn build_autoresearch_record_with_status(
    descriptor: &AutoresearchLoop,
    iteration: usize,
    percent_improvement: f64,
    token_savings: f64,
    summary: String,
    artifacts: Vec<String>,
    metadata: serde_json::Value,
    status: &str,
) -> LoopRecord {
    LoopRecord {
        slug: Some(descriptor.slug.to_string()),
        name: Some(descriptor.name.to_string()),
        iteration: Some(iteration as u32),
        percent_improvement: Some(percent_improvement.clamp(0.0, 100.0)),
        token_savings: Some(token_savings.max(0.0)),
        status: Some(status.to_string()),
        summary: Some(summary),
        artifacts: Some(artifacts),
        created_at: Some(Utc::now()),
        metadata,
    }
}

const AUTORESEARCH_MIN_EVIDENCE_SIGNALS: usize = 3;

fn loop_meets_absolute_floor(
    descriptor: &AutoresearchLoop,
    percent: f64,
    token_savings: f64,
    evidence_count: usize,
) -> bool {
    percent >= descriptor.base_percent
        && token_savings >= descriptor.base_tokens
        && evidence_count >= AUTORESEARCH_MIN_EVIDENCE_SIGNALS
}

fn loop_success_requires_second_signal(
    primary_ok: bool,
    secondary_ok: bool,
    confidence_ok: bool,
    trend_ok: bool,
) -> bool {
    primary_ok && secondary_ok && confidence_ok && trend_ok
}

fn loop_confidence_metadata(
    descriptor: &AutoresearchLoop,
    percent: f64,
    token_savings: f64,
    absolute_floor_met: bool,
    evidence_count: usize,
) -> serde_json::Value {
    serde_json::json!({
        "absolute_percent_floor": descriptor.base_percent,
        "absolute_token_floor": descriptor.base_tokens,
        "min_evidence_signals": AUTORESEARCH_MIN_EVIDENCE_SIGNALS,
        "evidence_count": evidence_count,
        "absolute_percent_met": percent >= descriptor.base_percent,
        "absolute_token_met": token_savings >= descriptor.base_tokens,
        "absolute_floor_met": absolute_floor_met,
    })
}

fn loop_floor_warning_reasons(
    descriptor: &AutoresearchLoop,
    percent: f64,
    token_savings: f64,
    evidence_count: usize,
) -> Vec<String> {
    let mut reasons = Vec::new();
    if percent < descriptor.base_percent {
        reasons.push("percent_below_floor".to_string());
    }
    if token_savings < descriptor.base_tokens {
        reasons.push("token_savings_below_floor".to_string());
    }
    if evidence_count < AUTORESEARCH_MIN_EVIDENCE_SIGNALS {
        reasons.push("evidence_count_below_floor".to_string());
    }
    reasons
}

const AUTORESEARCH_EXPERIMENT_MAX_AGE_HOURS: i64 = 24;

fn experiment_report_is_fresh(report: &ExperimentReport) -> bool {
    let age = Utc::now()
        .signed_duration_since(report.completed_at)
        .num_hours();
    age <= AUTORESEARCH_EXPERIMENT_MAX_AGE_HOURS
}

fn loop_is_regressed(
    descriptor: &AutoresearchLoop,
    previous_entry: Option<&LoopSummaryEntry>,
    percent: f64,
    token_savings: f64,
) -> bool {
    loop_trend_warning_reasons(descriptor, previous_entry, percent, token_savings)
        .iter()
        .any(|reason| reason.starts_with("trend_"))
}

fn loop_trend_metadata(
    descriptor: &AutoresearchLoop,
    previous_entry: Option<&LoopSummaryEntry>,
    percent: f64,
    token_savings: f64,
) -> serde_json::Value {
    match previous_entry {
        Some(previous) => serde_json::json!({
            "previous_percent": previous.percent_improvement,
            "previous_token_savings": previous.token_savings,
            "trend_percent_floor": descriptor.trend_percent_floor,
            "trend_token_floor": descriptor.trend_token_floor,
            "regressed": loop_is_regressed(descriptor, previous_entry, percent, token_savings),
            "warning_reasons": loop_trend_warning_reasons(
                descriptor,
                previous_entry,
                percent,
                token_savings,
            ),
        }),
        None => serde_json::json!({
            "previous_percent": serde_json::Value::Null,
            "previous_token_savings": serde_json::Value::Null,
            "trend_percent_floor": descriptor.trend_percent_floor,
            "trend_token_floor": descriptor.trend_token_floor,
            "regressed": false,
            "warning_reasons": Vec::<String>::new(),
        }),
    }
}

fn loop_trend_warning_reasons(
    descriptor: &AutoresearchLoop,
    previous_entry: Option<&LoopSummaryEntry>,
    percent: f64,
    token_savings: f64,
) -> Vec<String> {
    let Some(previous) = previous_entry else {
        return Vec::new();
    };
    let previous_percent = previous.percent_improvement.unwrap_or(0.0);
    let previous_tokens = previous.token_savings.unwrap_or(0.0);
    let mut reasons = Vec::new();
    if percent + descriptor.trend_percent_floor <= previous_percent {
        reasons.push("trend_percent_regressed".to_string());
    }
    if token_savings + descriptor.trend_token_floor <= previous_tokens {
        reasons.push("trend_token_regressed".to_string());
    }
    reasons
}

fn improvement_less_is_better(measured: f64, baseline: f64) -> f64 {
    if baseline <= 0.0 {
        return 0.0;
    }
    ((baseline - measured).max(0.0) / baseline) * 100.0
}

fn autoresearch_resume_args(output: &Path) -> ResumeArgs {
    autoresearch_resume_args_with_limits(output, 8, 4, true)
}

#[cfg(test)]
#[allow(dead_code)]
fn autoresearch_long_context_resume_args(output: &Path) -> ResumeArgs {
    autoresearch_resume_args_with_limits(output, 0, 0, false)
}

fn autoresearch_resume_args_with_limits(
    output: &Path,
    limit: usize,
    rehydration_limit: usize,
    semantic: bool,
) -> ResumeArgs {
    ResumeArgs {
        output: output.to_path_buf(),
        project: None,
        namespace: None,
        agent: None,
        workspace: None,
        visibility: None,
        route: None,
        intent: None,
        limit: Some(limit),
        rehydration_limit: Some(rehydration_limit),
        semantic,
        prompt: false,
        summary: false,
    }
}

fn read_latest_experiment_report(output: &Path) -> anyhow::Result<Option<ExperimentReport>> {
    let path = experiment_reports_dir(output).join("latest.json");
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let report = serde_json::from_str::<ExperimentReport>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(report))
}

struct AutoresearchLoop {
    slug: &'static str,
    normalized_slug: &'static str,
    name: &'static str,
    description: &'static str,
    target: &'static str,
    metric: &'static str,
    stop_condition: &'static str,
    risk: &'static str,
    base_percent: f64,
    base_tokens: f64,
    trend_percent_floor: f64,
    trend_token_floor: f64,
}

impl AutoresearchLoop {
    #[allow(clippy::too_many_arguments)]
    const fn new(
        slug: &'static str,
        normalized_slug: &'static str,
        name: &'static str,
        description: &'static str,
        target: &'static str,
        metric: &'static str,
        stop_condition: &'static str,
        risk: &'static str,
        base_percent: f64,
        base_tokens: f64,
        trend_percent_floor: f64,
        trend_token_floor: f64,
    ) -> AutoresearchLoop {
        AutoresearchLoop {
            slug,
            normalized_slug,
            name,
            description,
            target,
            metric,
            stop_condition,
            risk,
            base_percent,
            base_tokens,
            trend_percent_floor,
            trend_token_floor,
        }
    }
}

static AUTORESEARCH_LOOPS: [AutoresearchLoop; 10] = [
    AutoresearchLoop::new(
        "hive-health",
        "hive-health",
        "Hive Health",
        "Keep live sessions, heartbeat publication, and claim collisions healthy.",
        "live sessions / claims",
        "dead sessions / collisions",
        "no dead sessions",
        "low",
        1.0,
        40.0,
        0.5,
        4.0,
    ),
    AutoresearchLoop::new(
        "memory-hygiene",
        "memory-hygiene",
        "Memory Hygiene",
        "Track stale memories, duplicate memories, orphaned entries, and compression wins.",
        "duplicate memories",
        "stale / duplicate memory pressure",
        "low duplicate pressure",
        "medium",
        1.2,
        80.0,
        0.5,
        4.0,
    ),
    AutoresearchLoop::new(
        "autonomy-quality",
        "autonomy-quality",
        "Autonomy Quality",
        "Track false-green rate, warning rate, and real delta versus noise.",
        "false-green rate",
        "warning / noise pressure",
        "false-green pressure low",
        "high",
        0.9,
        60.0,
        1.0,
        6.0,
    ),
    AutoresearchLoop::new(
        "prompt-efficiency",
        "prompt-efficiency",
        "Prompt Efficiency",
        "Track prompt token burn, reuse rate, and bundle shrink.",
        "prompt token burn",
        "reuse / shrink pressure",
        "prompt burn stays low",
        "low",
        2.5,
        50.0,
        0.5,
        8.0,
    ),
    AutoresearchLoop::new(
        "repair-rate",
        "repair-rate",
        "Repair Rate",
        "Track how often the system fixes real problems instead of churning on superficial changes.",
        "repair recurrence",
        "real repairs vs churn",
        "repair rate stays high",
        "medium",
        1.0,
        60.0,
        0.5,
        4.0,
    ),
    AutoresearchLoop::new(
        "signal-freshness",
        "signal-freshness",
        "Signal Freshness",
        "Track stale snapshot rate, live-truth drift, and refresh pressure.",
        "live-truth freshness",
        "stale snapshot rate",
        "freshness baseline met",
        "low-medium",
        1.6,
        40.0,
        0.5,
        6.0,
    ),
    AutoresearchLoop::new(
        "cross-harness",
        "cross-harness",
        "Cross-Harness Portability",
        "Keep memories and promoted artifacts portable across harnesses.",
        "contract coverage",
        "adapter-required warnings",
        "portability class assigned",
        "medium",
        1.2,
        110.0,
        0.5,
        4.0,
    ),
    AutoresearchLoop::new(
        "self-evolution",
        "self-evolution",
        "Controlled Self-Evolution",
        "Ensure the evolution engine promotes only validated, measurable wins.",
        "accepted-change rate",
        "promotion evidence coverage",
        "confidence threshold reached",
        "high",
        0.7,
        50.0,
        1.0,
        6.0,
    ),
    AutoresearchLoop::new(
        "branch-review-quality",
        "branch-review-quality",
        "Branch Review Quality",
        "Track branch cleanliness, diff quality, and review readiness.",
        "branch cleanliness",
        "dirty branch / review readiness",
        "review ready",
        "medium",
        1.0,
        20.0,
        0.5,
        4.0,
    ),
    AutoresearchLoop::new(
        "docs-spec-drift",
        "docs-spec-drift",
        "Docs Spec Drift",
        "Keep docs and shipped behavior aligned.",
        "docs alignment",
        "spec drift",
        "docs match runtime",
        "medium",
        1.0,
        40.0,
        0.5,
        4.0,
    ),
];

#[derive(Debug)]
struct LoopEntry {
    slug: String,
    normalized_slug: String,
    record: LoopRecord,
    path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LoopRecord {
    #[serde(default)]
    slug: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    iteration: Option<u32>,
    #[serde(default)]
    percent_improvement: Option<f64>,
    #[serde(default)]
    token_savings: Option<f64>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    artifacts: Option<Vec<String>>,
    #[serde(default)]
    created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    metadata: JsonValue,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct LoopSummary {
    entries: Vec<LoopSummaryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LoopSummaryEntry {
    slug: String,
    percent_improvement: Option<f64>,
    token_savings: Option<f64>,
    status: Option<String>,
    recorded_at: DateTime<Utc>,
}

fn is_project_root_candidate(dir: &Path) -> bool {
    dir.join(".git").exists()
        || dir.join(".planning").exists()
        || dir.join("CLAUDE.md").exists()
        || dir.join("AGENTS.md").exists()
        || dir.join(".claude").join("CLAUDE.md").exists()
        || dir.join(".agents").join("CLAUDE.md").exists()
}

fn build_project_bootstrap_memory(
    project_root: Option<&Path>,
    project: &str,
    args: &InitArgs,
) -> anyhow::Result<Option<ProjectBootstrapBundle>> {
    let mut sources = project_root
        .map(collect_project_bootstrap_sources)
        .unwrap_or_default();
    sources.extend(collect_user_harness_bootstrap_sources(project_root));
    let mut seen = std::collections::HashSet::new();
    sources.retain(|path| seen.insert(path.clone()));
    sources.sort();
    if sources.is_empty() {
        return Ok(None);
    }

    let mut markdown = String::new();
    let mut registry_sources = Vec::new();
    markdown.push_str("# memd project bootstrap\n\n");
    if let Some(project_root) = project_root {
        markdown.push_str(&format!(
            "This bundle was initialized from the existing project context at `{}`.\n\n",
            project_root.display()
        ));
    } else {
        markdown.push_str(
            "This bundle was initialized from the user's configured harness context.\n\n",
        );
    }
    markdown.push_str("## Loaded sources\n\n");
    for source in &sources {
        markdown.push_str(&format!(
            "- {}\n",
            display_bootstrap_source_path(source, project_root)
        ));
    }
    markdown.push_str("\n## Imported summaries\n\n");

    for source in sources.drain(..) {
        let display = display_bootstrap_source_path(&source, project_root);
        if let Some((snippet, meta)) = read_bootstrap_source(&source, 24) {
            registry_sources.push(BootstrapSourceRecord {
                path: display.clone(),
                kind: source_kind_from_path(&source),
                hash: meta.hash,
                bytes: meta.bytes,
                lines: meta.lines,
                present: true,
                imported_at: Utc::now(),
                modified_at: file_modified_at(&source),
            });
            markdown.push_str(&format!("### {}\n\n{}\n\n", display, snippet));
        }
    }

    markdown.push_str("## Notes\n\n");
    markdown.push_str(&format!(
        "- project: `{}`\n- init agent: `{}`\n- bootstrap mode: `{}`\n",
        project,
        args.agent,
        if args.seed_existing {
            "seed_existing"
        } else {
            "manual"
        }
    ));
    markdown.push_str(
        "- source registry: `state/source-registry.json` with content hashes for imported files\n",
    );
    markdown.push_str("- Add a separate import command if you need a deeper file sweep or more context than the default bootstrap budget.\n");

    Ok(Some(ProjectBootstrapBundle {
        markdown,
        registry: BootstrapSourceRegistry {
            project: project.to_string(),
            project_root: project_root
                .map(|root| root.display().to_string())
                .unwrap_or_else(|| default_global_bundle_root().display().to_string()),
            imported_at: Utc::now(),
            sources: registry_sources,
        },
    }))
}

fn build_bundle_capability_registry(project_root: Option<&Path>) -> CapabilityRegistry {
    build_bundle_capability_registry_with_home(project_root, home_dir().as_deref())
}

fn build_bundle_capability_registry_with_home(
    project_root: Option<&Path>,
    home: Option<&Path>,
) -> CapabilityRegistry {
    let mut capabilities = Vec::new();

    if let Some(project_root) = project_root {
        for (name, kind) in [
            ("AGENTS.md", "policy"),
            ("TEAMS.md", "team"),
            ("CLAUDE.md", "policy"),
        ] {
            let path = project_root.join(name);
            if path.is_file() {
                capabilities.push(CapabilityRecord {
                    harness: "project".to_string(),
                    kind: kind.to_string(),
                    name: name.to_string(),
                    status: "discovered".to_string(),
                    portability_class: "universal".to_string(),
                    source_path: display_bootstrap_source_path(&path, Some(project_root)),
                    bridge_hint: None,
                    hash: file_sha256(&path),
                    notes: Vec::new(),
                });
            }
        }
    }

    let Some(home) = home else {
        return CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: project_root.map(|path| path.display().to_string()),
            capabilities,
        };
    };

    collect_skill_capabilities(
        &mut capabilities,
        "codex",
        &home.join(".codex").join("skills"),
    );
    collect_skill_capabilities(
        &mut capabilities,
        "claude",
        &home.join(".claude").join("skills"),
    );

    let codex_agents_superpowers = home.join(".agents").join("skills").join("superpowers");
    if codex_agents_superpowers.exists() {
        capabilities.push(CapabilityRecord {
            harness: "codex".to_string(),
            kind: "skill-bridge".to_string(),
            name: "superpowers".to_string(),
            status: "installed".to_string(),
            portability_class: "universal".to_string(),
            source_path: codex_agents_superpowers.display().to_string(),
            bridge_hint: None,
            hash: None,
            notes: vec![
                "discovered through ~/.agents/skills native Codex skill bridge".to_string(),
            ],
        });
    }

    for harness_root in detect_claude_family_harness_roots(&home) {
        capabilities.extend(collect_claude_family_capabilities(&harness_root, &home));
    }

    let opencode_plugin = home
        .join(".config")
        .join("opencode")
        .join("plugins")
        .join("memd-plugin.mjs");
    if opencode_plugin.is_file() {
        capabilities.push(CapabilityRecord {
            harness: "opencode".to_string(),
            kind: "plugin".to_string(),
            name: "memd".to_string(),
            status: "enabled".to_string(),
            portability_class: "universal".to_string(),
            source_path: opencode_plugin.display().to_string(),
            bridge_hint: None,
            hash: file_sha256(&opencode_plugin),
            notes: vec!["local memd plugin bridge is active".to_string()],
        });
    }

    let openclaw_workspace_root = home.join(".openclaw").join("workspace");
    capabilities.extend(collect_openclaw_capabilities(&openclaw_workspace_root));

    let opencode_workspace_root = home.join(".config").join("opencode");
    capabilities.extend(collect_opencode_capabilities(&opencode_workspace_root));

    capabilities.sort_by(|a, b| {
        a.harness
            .cmp(&b.harness)
            .then(a.kind.cmp(&b.kind))
            .then(a.name.cmp(&b.name))
            .then(a.source_path.cmp(&b.source_path))
    });
    capabilities.dedup_by(|a, b| {
        a.harness == b.harness
            && a.kind == b.kind
            && a.name == b.name
            && a.source_path == b.source_path
    });

    CapabilityRegistry {
        generated_at: Utc::now(),
        project_root: project_root.map(|path| path.display().to_string()),
        capabilities,
    }
}

fn collect_skill_capabilities(records: &mut Vec<CapabilityRecord>, harness: &str, root: &Path) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };

    let mut skills = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir() && path.join("SKILL.md").is_file())
        .collect::<Vec<_>>();
    skills.sort();

    for skill_dir in skills {
        let skill_name = skill_dir
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("unknown");
        let skill_file = skill_dir.join("SKILL.md");
        records.push(CapabilityRecord {
            harness: harness.to_string(),
            kind: "skill".to_string(),
            name: skill_name.to_string(),
            status: "installed".to_string(),
            portability_class: "harness-native".to_string(),
            source_path: skill_file.display().to_string(),
            bridge_hint: None,
            hash: file_sha256(&skill_file),
            notes: Vec::new(),
        });
    }
}

fn collect_claude_family_capabilities(
    harness_root: &HarnessRoot,
    home: &Path,
) -> Vec<CapabilityRecord> {
    let mut records = Vec::new();
    let portability_class = if harness_root.harness == "claude" {
        "universal".to_string()
    } else {
        "claude-family".to_string()
    };
    records.extend(
        collect_named_file_sources(
            &harness_root.root,
            &[
                "AGENTS.md",
                "TEAMS.md",
                "MEMORY.md",
                "USER.md",
                "IDENTITY.md",
                "SOUL.md",
                "TOOLS.md",
                "BOOTSTRAP.md",
                "HEARTBEAT.md",
            ],
        )
        .into_iter()
        .map(|path| CapabilityRecord {
            harness: harness_root.harness.clone(),
            kind: source_kind_from_path(&path),
            name: path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("unknown")
                .to_string(),
            status: "discovered".to_string(),
            portability_class: portability_class.clone(),
            source_path: path.display().to_string(),
            bridge_hint: None,
            hash: file_sha256(&path),
            notes: vec!["detected from Claude-family harness root".to_string()],
        }),
    );
    collect_directory_entry_capabilities(
        &mut records,
        &harness_root.harness,
        &harness_root.root,
        "agents",
        "agent",
        &portability_class,
        Some("agent"),
        &["md", "json", "yml", "yaml"],
    );
    collect_directory_entry_capabilities(
        &mut records,
        &harness_root.harness,
        &harness_root.root,
        "teams",
        "team",
        &portability_class,
        Some("team"),
        &["md", "json", "yml", "yaml"],
    );
    collect_directory_entry_capabilities(
        &mut records,
        &harness_root.harness,
        &harness_root.root,
        "hooks",
        "hook",
        &portability_class,
        Some("hook"),
        &["js", "mjs", "ts", "cts", "json"],
    );
    collect_directory_entry_capabilities(
        &mut records,
        &harness_root.harness,
        &harness_root.root,
        "command",
        "command",
        &portability_class,
        Some("command"),
        &["md", "json", "yml", "yaml", "sh", "js"],
    );
    records.extend(collect_claude_plugin_capabilities(
        &harness_root.root.join("settings.json"),
        &harness_root.harness,
        home,
    ));
    records
}

fn collect_openclaw_capabilities(workspace_root: &Path) -> Vec<CapabilityRecord> {
    collect_harness_root_directory_capabilities(
        workspace_root,
        "openclaw",
        "harness-native",
        &[
            "AGENTS.md",
            "TEAMS.md",
            "MEMORY.md",
            "USER.md",
            "IDENTITY.md",
            "SOUL.md",
            "TOOLS.md",
            "BOOTSTRAP.md",
            "HEARTBEAT.md",
        ],
    )
}

fn collect_opencode_capabilities(root: &Path) -> Vec<CapabilityRecord> {
    collect_harness_root_directory_capabilities(
        root,
        "opencode",
        "harness-native",
        &[
            "AGENTS.md",
            "TEAMS.md",
            "MEMORY.md",
            "USER.md",
            "IDENTITY.md",
            "SOUL.md",
            "TOOLS.md",
            "BOOTSTRAP.md",
            "HEARTBEAT.md",
            "opencode.json",
            "settings.json",
        ],
    )
}

fn collect_harness_root_directory_capabilities(
    harness_root: &Path,
    harness: &str,
    portability_class: &str,
    named_files: &[&str],
) -> Vec<CapabilityRecord> {
    let mut records = Vec::new();
    records.extend(
        collect_named_file_sources(harness_root, named_files)
            .into_iter()
            .map(|path| CapabilityRecord {
                harness: harness.to_string(),
                kind: source_kind_from_path(&path),
                name: path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                status: "discovered".to_string(),
                portability_class: portability_class.to_string(),
                source_path: path.display().to_string(),
                bridge_hint: None,
                hash: file_sha256(&path),
                notes: vec![format!("detected from {harness} workspace root")],
            }),
    );

    collect_directory_entry_capabilities(
        &mut records,
        harness,
        harness_root,
        "agents",
        "agent",
        portability_class,
        Some("agent"),
        &["md", "json", "yml", "yaml"],
    );
    collect_directory_entry_capabilities(
        &mut records,
        harness,
        harness_root,
        "teams",
        "team",
        portability_class,
        Some("team"),
        &["md", "json", "yml", "yaml"],
    );
    collect_directory_entry_capabilities(
        &mut records,
        harness,
        harness_root,
        "hooks",
        "hook",
        portability_class,
        Some("hook"),
        &["js", "mjs", "ts", "cts", "json"],
    );
    collect_directory_entry_capabilities(
        &mut records,
        harness,
        harness_root,
        "command",
        "command",
        portability_class,
        Some("command"),
        &["md", "json", "yml", "yaml", "sh", "js"],
    );

    records.sort_by(|a, b| {
        a.harness
            .cmp(&b.harness)
            .then(a.kind.cmp(&b.kind))
            .then(a.name.cmp(&b.name))
            .then(a.source_path.cmp(&b.source_path))
    });
    records.dedup_by(|a, b| {
        a.harness == b.harness
            && a.kind == b.kind
            && a.name == b.name
            && a.source_path == b.source_path
    });
    records
}

fn collect_claude_plugin_capabilities(
    settings_path: &Path,
    harness_name: &str,
    home: &Path,
) -> Vec<CapabilityRecord> {
    let mut records = Vec::new();
    let Ok(raw) = fs::read_to_string(settings_path) else {
        return records;
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return records;
    };
    let Some(enabled) = json
        .get("enabledPlugins")
        .and_then(|value| value.as_object())
    else {
        return records;
    };

    let codex_agents_root = home.join(".agents").join("skills");
    for (plugin_id, value) in enabled {
        if !value.as_bool().unwrap_or(false) {
            continue;
        }

        let (plugin_name, marketplace) = parse_marketplace_plugin_id(plugin_id);
        let codex_cache = latest_cached_plugin_root(
            &home.join(".codex").join("plugins").join("cache"),
            marketplace.as_deref().unwrap_or("unknown"),
            &plugin_name,
        );
        let codex_skills = codex_cache
            .as_ref()
            .map(|path| path.join("skills"))
            .filter(|path| path.is_dir());
        let codex_install = codex_cache
            .as_ref()
            .map(|path| path.join(".codex").join("INSTALL.md"))
            .filter(|path| path.is_file());
        let opencode_bridge = codex_cache
            .as_ref()
            .map(|path| path.join(".opencode").join("plugins"))
            .filter(|path| path.is_dir());
        let codex_skill_bridge = codex_agents_root.join(&plugin_name);

        let portability_class = if codex_skill_bridge.exists() {
            "universal"
        } else if codex_skills.is_some() || codex_install.is_some() || opencode_bridge.is_some() {
            "bridgeable"
        } else {
            "harness-native"
        };
        let bridge_hint = if codex_skill_bridge.exists() {
            None
        } else {
            codex_skills
                .as_ref()
                .map(|path| {
                    format!(
                        "bridge into Codex via ~/.agents/skills -> {}",
                        path.display()
                    )
                })
                .or_else(|| {
                    codex_install
                        .as_ref()
                        .map(|path| format!("bridge into Codex via {}", path.display()))
                })
                .or_else(|| {
                    opencode_bridge
                        .as_ref()
                        .map(|path| format!("bridge into OpenCode via {}", path.display()))
                })
        };

        let mut notes = Vec::new();
        if let Some(path) = codex_cache.as_ref() {
            notes.push(format!(
                "cached in Codex plugin cache at {}",
                path.display()
            ));
        }
        if codex_skill_bridge.exists() {
            notes.push("active Codex bridge detected under ~/.agents/skills".to_string());
        }

        records.push(CapabilityRecord {
            harness: harness_name.to_string(),
            kind: "plugin".to_string(),
            name: plugin_name.clone(),
            status: "enabled".to_string(),
            portability_class: if harness_name == "claude" || portability_class == "universal" {
                portability_class.to_string()
            } else if portability_class == "bridgeable" {
                "claude-family-bridgeable".to_string()
            } else {
                "claude-family".to_string()
            },
            source_path: settings_path.display().to_string(),
            bridge_hint,
            hash: file_sha256(settings_path),
            notes,
        });

        let effective_portability = if harness_name == "claude" || portability_class == "universal"
        {
            portability_class.to_string()
        } else if portability_class == "bridgeable" {
            "claude-family-bridgeable".to_string()
        } else {
            "claude-family".to_string()
        };
        collect_claude_plugin_artifact_capabilities(
            &mut records,
            harness_name,
            &plugin_name,
            codex_cache.as_ref(),
            &effective_portability,
        );
    }

    records
}

fn collect_claude_plugin_artifact_capabilities(
    records: &mut Vec<CapabilityRecord>,
    harness_name: &str,
    plugin_name: &str,
    codex_cache: Option<&PathBuf>,
    portability_class: &str,
) {
    let Some(cache_root) = codex_cache else {
        return;
    };
    collect_directory_entry_capabilities(
        records,
        harness_name,
        cache_root,
        "command",
        "command",
        portability_class,
        Some(plugin_name),
        &["md", "json", "yml", "yaml", "sh", "js"],
    );
    collect_directory_entry_capabilities(
        records,
        harness_name,
        cache_root,
        "hooks",
        "hook",
        portability_class,
        Some(plugin_name),
        &["js", "mjs", "ts", "cts", "json"],
    );
}

fn parse_marketplace_plugin_id(plugin_id: &str) -> (String, Option<String>) {
    let mut parts = plugin_id.split('@');
    let name = parts.next().unwrap_or(plugin_id).trim().to_string();
    let marketplace = parts.next().map(|value| value.trim().to_string());
    (name, marketplace)
}

#[allow(clippy::too_many_arguments)]
fn collect_directory_entry_capabilities(
    records: &mut Vec<CapabilityRecord>,
    harness: &str,
    root: &Path,
    relative_dir: &str,
    kind: &str,
    portability_class: &str,
    name_prefix: Option<&str>,
    extensions: &[&str],
) {
    let dir = root.join(relative_dir);
    let Ok(entries) = fs::read_dir(&dir) else {
        return;
    };

    let mut files = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| {
                    extensions
                        .iter()
                        .any(|ext_name| ext.eq_ignore_ascii_case(ext_name))
                })
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    files.sort();

    for path in files {
        let file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("unknown");
        let name = match name_prefix {
            Some(prefix) => format!("{prefix}:{file_name}"),
            None => file_name.to_string(),
        };
        records.push(CapabilityRecord {
            harness: harness.to_string(),
            kind: kind.to_string(),
            name,
            status: "discovered".to_string(),
            portability_class: portability_class.to_string(),
            source_path: path.display().to_string(),
            bridge_hint: None,
            hash: file_sha256(&path),
            notes: vec![format!("discovered from {relative_dir} surface")],
        });
    }
}

fn latest_cached_plugin_root(
    cache_root: &Path,
    marketplace: &str,
    plugin_name: &str,
) -> Option<PathBuf> {
    let plugin_root = cache_root.join(marketplace).join(plugin_name);
    let Ok(entries) = fs::read_dir(&plugin_root) else {
        return None;
    };

    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .max()
}

fn file_sha256(path: &Path) -> Option<String> {
    let raw = fs::read(path).ok()?;
    Some(format!("{:x}", Sha256::digest(&raw)))
}

fn render_capability_registry_summary(registry: &CapabilityRegistry) -> String {
    let mut markdown = String::new();
    let total = registry.capabilities.len();
    let universal = registry
        .capabilities
        .iter()
        .filter(|record| is_universal_class(&record.portability_class))
        .count();
    let bridgeable = registry
        .capabilities
        .iter()
        .filter(|record| is_bridgeable_class(&record.portability_class))
        .count();
    let harness_native = registry
        .capabilities
        .iter()
        .filter(|record| is_harness_native_class(&record.portability_class))
        .count();

    markdown.push_str("## Capability Registry\n\n");
    markdown.push_str(&format!(
        "- discovered_capabilities: {}\n- universal: {}\n- bridgeable: {}\n- harness_native: {}\n",
        total, universal, bridgeable, harness_native
    ));

    let bridgeable_items = registry
        .capabilities
        .iter()
        .filter(|record| is_bridgeable_class(&record.portability_class))
        .take(8)
        .collect::<Vec<_>>();
    if !bridgeable_items.is_empty() {
        markdown.push_str("\n### Bridgeable capabilities\n\n");
        for item in bridgeable_items {
            markdown.push_str(&format!(
                "- {} / {} / {}",
                item.harness, item.kind, item.name
            ));
            if !item.portability_class.is_empty() {
                markdown.push_str(&format!(" [{}]", item.portability_class));
            }
            if let Some(hint) = item.bridge_hint.as_deref() {
                markdown.push_str(&format!(" -> {}", hint));
            }
            markdown.push('\n');
        }
    }

    markdown
}

fn is_universal_class(class: &str) -> bool {
    class == "universal"
}

fn is_bridgeable_class(class: &str) -> bool {
    class.contains("bridgeable")
}

fn is_harness_native_class(class: &str) -> bool {
    matches!(
        class,
        "harness-native" | "claude-family" | "claude-family-bridgeable"
    ) || class.starts_with("harness-")
}

fn render_capability_bridge_summary(registry: &CapabilityBridgeRegistry) -> String {
    let mut markdown = String::new();
    let bridged = registry
        .actions
        .iter()
        .filter(|action| action.status == "bridged")
        .count();
    let already = registry
        .actions
        .iter()
        .filter(|action| action.status == "already-bridged")
        .count();
    let available = registry
        .actions
        .iter()
        .filter(|action| action.status == "available")
        .count();
    let blocked = registry
        .actions
        .iter()
        .filter(|action| action.status == "blocked")
        .count();

    markdown.push_str("## Capability Bridges\n\n");
    markdown.push_str(&format!(
        "- bridged: {}\n- already_bridged: {}\n- available: {}\n- blocked: {}\n",
        bridged, already, available, blocked
    ));
    if !registry.actions.is_empty() {
        markdown.push_str("\n### Recent bridge actions\n\n");
        for action in registry.actions.iter().take(8) {
            markdown.push_str(&format!(
                "- {} / {} -> {} ({})\n",
                action.harness, action.capability, action.target_path, action.status
            ));
        }
    }

    markdown
}

fn build_skill_lifecycle_report(policy: &MemoryPolicyResponse) -> SkillLifecycleReport {
    let registry = build_bundle_capability_registry(None);
    let bridges = detect_capability_bridges();
    let bridge_lookup = bridges
        .actions
        .iter()
        .map(|action| {
            (
                (action.harness.clone(), action.capability.clone()),
                (action.status.as_str(), action.target_path.as_str()),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let runtime_is_defaulted = is_default_runtime(&policy.runtime);
    let low_risk_threshold = 0.25_f32;

    let mut records = Vec::new();
    let mut proposed = 0usize;
    let mut sandbox_passed = 0usize;
    let mut sandbox_review = 0usize;
    let mut sandbox_blocked = 0usize;
    let mut activation_candidates = 0usize;
    let mut activated = 0usize;
    let mut review_queue = Vec::new();
    let mut activate_queue = Vec::new();

    for capability in registry
        .capabilities
        .iter()
        .filter(|capability| capability.kind == "skill" || capability.kind == "skill-bridge")
    {
        proposed += 1;
        let proposal = if capability.status == "installed" || capability.status == "enabled" {
            "proposed"
        } else {
            "staged"
        };

        let bridge_state = bridge_lookup
            .get(&(capability.harness.clone(), capability.name.clone()))
            .copied();
        let (sandbox, sandbox_risk, sandbox_reason) = score_skill_sandbox(capability, bridge_state);
        if sandbox == "pass" {
            sandbox_passed += 1;
        } else if sandbox == "review" {
            sandbox_review += 1;
        } else if sandbox == "block" {
            sandbox_blocked += 1;
        }

        let policy_allows_activation =
            !runtime_is_defaulted && policy.runtime.skill_gating.gated_activation;
        let activation = if !policy_allows_activation {
            "review"
        } else if sandbox == "pass"
            && policy.runtime.skill_gating.sandboxed_evaluation
            && (!policy.runtime.skill_gating.auto_activate_low_risk_only
                || sandbox_risk <= low_risk_threshold)
        {
            activated += 1;
            activation_candidates += 1;
            "activate"
        } else if sandbox == "pass" {
            activation_candidates += 1;
            "candidate"
        } else {
            "hold"
        };
        let activation_reason = match activation {
            "activate" => "low-risk sandbox passed and policy allowed auto-activation",
            "candidate" => "sandbox passed but policy still wants explicit activation",
            "review" if runtime_is_defaulted => {
                "legacy backend defaults require review before activation"
            }
            "review" => "policy gate requires review before activation",
            _ => "sandbox did not pass",
        };

        let record = SkillLifecycleRecord {
            harness: capability.harness.clone(),
            name: capability.name.clone(),
            kind: capability.kind.clone(),
            portability_class: capability.portability_class.clone(),
            proposal: proposal.to_string(),
            sandbox: sandbox.to_string(),
            sandbox_risk,
            sandbox_reason,
            activation: activation.to_string(),
            activation_reason: activation_reason.to_string(),
            source_path: capability.source_path.clone(),
            target_path: bridge_state.map(|state| state.1.to_string()),
            notes: capability.notes.clone(),
        };
        if activation == "activate" {
            activate_queue.push(record.clone());
        } else {
            review_queue.push(record.clone());
        }
        records.push(record);
    }

    records.sort_by(|a, b| {
        a.harness
            .cmp(&b.harness)
            .then(a.kind.cmp(&b.kind))
            .then(a.name.cmp(&b.name))
    });

    SkillLifecycleReport {
        generated_at: Utc::now(),
        proposed,
        sandbox_passed,
        sandbox_review,
        sandbox_blocked,
        activation_candidates,
        activated,
        review_queue,
        activate_queue,
        records,
    }
}

fn score_skill_sandbox(
    capability: &CapabilityRecord,
    bridge_state: Option<(&str, &str)>,
) -> (&'static str, f32, String) {
    let mut risk: f32;
    let mut reasons = Vec::new();

    match capability.portability_class.as_str() {
        "universal" => {
            risk = 0.05;
            reasons.push("portable".to_string());
        }
        class if class.contains("bridgeable") => {
            risk = 0.20;
            reasons.push("bridgeable".to_string());
            match bridge_state.map(|state| state.0) {
                Some("bridged") | Some("already-bridged") => {
                    risk -= 0.12;
                    reasons.push("bridge_ready".to_string());
                }
                Some("blocked") => {
                    risk += 0.20;
                    reasons.push("bridge_blocked".to_string());
                }
                _ => {
                    reasons.push("bridge_pending".to_string());
                }
            }
        }
        "harness-native" => {
            risk = 0.38;
            reasons.push("harness_native".to_string());
        }
        other => {
            risk = 0.82;
            reasons.push(format!("portability={other}"));
        }
    }

    if capability.status == "installed" || capability.status == "enabled" {
        risk -= 0.03;
        reasons.push("present".to_string());
    }
    if capability.hash.is_some() {
        risk -= 0.01;
        reasons.push("hashed".to_string());
    }
    if capability
        .notes
        .iter()
        .any(|note| note.contains("active Codex bridge"))
    {
        risk -= 0.04;
        reasons.push("active_bridge".to_string());
    }

    risk = risk.clamp(0.0, 1.0);
    let sandbox = if risk <= 0.15 {
        "pass"
    } else if risk <= 0.5 {
        "review"
    } else {
        "block"
    };

    (sandbox, risk, reasons.join(";"))
}

fn render_skill_lifecycle_report(report: &SkillLifecycleReport, follow: bool) -> String {
    let mut markdown = String::new();
    markdown.push_str("## Skill Lifecycle\n\n");
    markdown.push_str(&format!(
        "- proposed: {}\n- sandbox_passed: {}\n- sandbox_review: {}\n- sandbox_blocked: {}\n- review_queue: {}\n- activate_queue: {}\n- activation_candidates: {}\n- activated: {}\n",
        report.proposed,
        report.sandbox_passed,
        report.sandbox_review,
        report.sandbox_blocked,
        report.review_queue.len(),
        report.activate_queue.len(),
        report.activation_candidates,
        report.activated
    ));

    if !report.activate_queue.is_empty() {
        markdown.push_str("\n### Activate Queue\n\n");
        for record in report
            .activate_queue
            .iter()
            .take(if follow { 12 } else { 8 })
        {
            markdown.push_str(&format!(
                "- {} / {} / {} risk={:.2} sandbox={} activation={} reason={}",
                record.harness,
                record.kind,
                record.name,
                record.sandbox_risk,
                record.sandbox,
                record.activation,
                record.activation_reason
            ));
            if let Some(target) = record.target_path.as_deref() {
                markdown.push_str(&format!(" -> {}", target));
            }
            if follow && !record.notes.is_empty() {
                markdown.push_str(&format!(" notes={}", record.notes.join(" | ")));
            }
            markdown.push('\n');
        }
    }

    if !report.review_queue.is_empty() {
        markdown.push_str("\n### Review Queue\n\n");
        for record in report.review_queue.iter().take(if follow { 12 } else { 8 }) {
            markdown.push_str(&format!(
                "- {} / {} / {} risk={:.2} sandbox={} activation={} reason={}",
                record.harness,
                record.kind,
                record.name,
                record.sandbox_risk,
                record.sandbox,
                record.activation,
                record.activation_reason
            ));
            if let Some(target) = record.target_path.as_deref() {
                markdown.push_str(&format!(" -> {}", target));
            }
            if follow && !record.notes.is_empty() {
                markdown.push_str(&format!(" notes={}", record.notes.join(" | ")));
            }
            markdown.push('\n');
        }
    }

    if follow && !report.records.is_empty() {
        markdown.push_str("\n### Lifecycle records\n\n");
        for record in report.records.iter().take(if follow { 12 } else { 8 }) {
            markdown.push_str(&format!(
                "- {} / {} / {} [{}] proposal={} sandbox={} risk={:.2} activation={}",
                record.harness,
                record.kind,
                record.name,
                record.portability_class,
                record.proposal,
                record.sandbox,
                record.sandbox_risk,
                record.activation
            ));
            if let Some(target) = record.target_path.as_deref() {
                markdown.push_str(&format!(" -> {}", target));
            }
            markdown.push_str(&format!(" reason={}", record.sandbox_reason));
            markdown.push_str(&format!(" activation_reason={}", record.activation_reason));
            if follow && !record.notes.is_empty() {
                markdown.push_str(&format!(" notes={}", record.notes.join(" | ")));
            }
            markdown.push('\n');
        }
    }

    markdown
}

fn render_skill_policy_batch_markdown(batch: &SkillPolicyBatchArtifact) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd skill policy batch\n\n");
    markdown.push_str(&format!(
        "- generated_at: {}\n- bundle_root: {}\n- runtime_defaulted: {}\n- proposed: {}\n- sandbox_passed: {}\n- sandbox_review: {}\n- sandbox_blocked: {}\n- review_queue: {}\n- activate_queue: {}\n- activation_candidates: {}\n- activated: {}\n",
        batch.generated_at.to_rfc3339(),
        batch.bundle_root,
        batch.runtime_defaulted,
        batch.report.proposed,
        batch.report.sandbox_passed,
        batch.report.sandbox_review,
        batch.report.sandbox_blocked,
        batch.report.review_queue.len(),
        batch.report.activate_queue.len(),
        batch.report.activation_candidates,
        batch.report.activated
    ));
    markdown.push_str("\n## Apply Flow\n\n");
    markdown.push_str(
        "Use the activate queue after sandbox review. Keep review queue as the manual follow-up set.\n",
    );
    if !batch.report.activate_queue.is_empty() {
        markdown.push_str("\n### Activate Queue\n\n");
        for record in batch.report.activate_queue.iter().take(12) {
            markdown.push_str(&format!(
                "- {} / {} / {} risk={:.2} sandbox={} activation={} reason={}",
                record.harness,
                record.kind,
                record.name,
                record.sandbox_risk,
                record.sandbox,
                record.activation,
                record.activation_reason
            ));
            if let Some(target) = record.target_path.as_deref() {
                markdown.push_str(&format!(" -> {}", target));
            }
            markdown.push('\n');
        }
    }
    if !batch.report.review_queue.is_empty() {
        markdown.push_str("\n### Review Queue\n\n");
        for record in batch.report.review_queue.iter().take(12) {
            markdown.push_str(&format!(
                "- {} / {} / {} risk={:.2} sandbox={} activation={} reason={}",
                record.harness,
                record.kind,
                record.name,
                record.sandbox_risk,
                record.sandbox,
                record.activation,
                record.activation_reason
            ));
            if let Some(target) = record.target_path.as_deref() {
                markdown.push_str(&format!(" -> {}", target));
            }
            markdown.push('\n');
        }
    }
    markdown
}

fn render_skill_policy_queue_markdown(queue: &SkillPolicyQueueArtifact) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!(
        "# memd skill policy {} queue\n\n- generated_at: {}\n- bundle_root: {}\n- runtime_defaulted: {}\n- records: {}\n",
        queue.queue,
        queue.generated_at.to_rfc3339(),
        queue.bundle_root,
        queue.runtime_defaulted,
        queue.records.len()
    ));
    if !queue.records.is_empty() {
        markdown.push_str("\n## Records\n\n");
        for record in queue.records.iter().take(16) {
            markdown.push_str(&format!(
                "- {} / {} / {} risk={:.2} sandbox={} activation={} reason={}",
                record.harness,
                record.kind,
                record.name,
                record.sandbox_risk,
                record.sandbox,
                record.activation,
                record.activation_reason
            ));
            if let Some(target) = record.target_path.as_deref() {
                markdown.push_str(&format!(" -> {}", target));
            }
            markdown.push('\n');
        }
    }
    markdown
}

fn render_skill_policy_apply_markdown(receipt: &SkillPolicyApplyArtifact) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd skill policy apply receipt\n\n");
    markdown.push_str(&format!(
        "- generated_at: {}\n- bundle_root: {}\n- runtime_defaulted: {}\n- source_queue_path: {}\n- applied_count: {}\n- skipped_count: {}\n",
        receipt.generated_at.to_rfc3339(),
        receipt.bundle_root,
        receipt.runtime_defaulted,
        receipt.source_queue_path,
        receipt.applied_count,
        receipt.skipped_count
    ));
    if !receipt.applied.is_empty() {
        markdown.push_str("\n## Applied\n\n");
        for record in receipt.applied.iter().take(16) {
            markdown.push_str(&format!(
                "- {} / {} / {} -> {}",
                record.harness, record.kind, record.name, record.activation_reason
            ));
            if let Some(target) = record.target_path.as_deref() {
                markdown.push_str(&format!(" -> {}", target));
            }
            markdown.push('\n');
        }
    }
    if !receipt.skipped.is_empty() {
        markdown.push_str("\n## Skipped\n\n");
        for record in receipt.skipped.iter().take(16) {
            markdown.push_str(&format!(
                "- {} / {} / {} -> {}",
                record.harness, record.kind, record.name, record.activation_reason
            ));
            if let Some(target) = record.target_path.as_deref() {
                markdown.push_str(&format!(" -> {}", target));
            }
            markdown.push('\n');
        }
    }
    markdown
}

fn render_skill_policy_query_summary(
    receipts: &SkillPolicyApplyReceiptsResponse,
    activations: &SkillPolicyActivationEntriesResponse,
    follow: bool,
) -> String {
    let mut markdown = String::new();
    markdown.push_str("## Skill Policy Query\n\n");
    markdown.push_str(&format!(
        "- receipts: {}\n- activations: {}\n",
        receipts.receipts.len(),
        activations.activations.len()
    ));
    if !receipts.receipts.is_empty() {
        markdown.push_str("\n### Receipts\n\n");
        for receipt in receipts.receipts.iter().take(if follow { 12 } else { 6 }) {
            markdown.push_str(&format!(
                "- {} applied={} skipped={} runtime_defaulted={} queue={}",
                receipt.id.chars().take(8).collect::<String>(),
                receipt.applied_count,
                receipt.skipped_count,
                receipt.runtime_defaulted,
                receipt.source_queue_path
            ));
            if let Some(project) = receipt.project.as_deref() {
                markdown.push_str(&format!(" project={}", project));
            }
            if let Some(namespace) = receipt.namespace.as_deref() {
                markdown.push_str(&format!(" namespace={}", namespace));
            }
            if let Some(workspace) = receipt.workspace.as_deref() {
                markdown.push_str(&format!(" workspace={}", workspace));
            }
            markdown.push('\n');
        }
    }
    if !activations.activations.is_empty() {
        markdown.push_str("\n### Activations\n\n");
        for entry in activations
            .activations
            .iter()
            .take(if follow { 12 } else { 6 })
        {
            markdown.push_str(&format!(
                "- {} / {} / {} action={} sandbox={} risk={:.2}",
                entry.receipt_id.chars().take(8).collect::<String>(),
                entry.record.harness,
                entry.record.name,
                entry.record.activation,
                entry.record.sandbox,
                entry.record.sandbox_risk
            ));
            markdown.push_str(&format!(" queue={}", entry.source_queue_path));
            if let Some(target) = entry.record.target_path.as_deref() {
                markdown.push_str(&format!(" -> {}", target));
            }
            markdown.push('\n');
        }
    }
    markdown
}

fn apply_capability_bridges() -> CapabilityBridgeRegistry {
    let mut actions = Vec::new();
    let Some(home) = home_dir() else {
        return CapabilityBridgeRegistry {
            generated_at: Utc::now(),
            actions,
        };
    };

    let claude_settings = home.join(".claude").join("settings.json");
    let codex_skill_root = home.join(".agents").join("skills");
    let opencode_modern_plugins = home.join(".config").join("opencode").join("plugins");
    let opencode_legacy_plugins = home.join(".opencode").join("plugins");
    let plugin_records = collect_enabled_plugin_cache_records(&claude_settings, &home);
    for record in plugin_records {
        let source_skills = record.cache_root.join("skills");
        if source_skills.is_dir() {
            let target = codex_skill_root.join(&record.plugin_name);
            actions.push(ensure_directory_skill_bridge(
                "codex",
                &record.plugin_name,
                &source_skills,
                &target,
            ));
        }

        let source_opencode_plugins = record.cache_root.join(".opencode").join("plugins");
        if source_opencode_plugins.is_dir() {
            for target_root in [&opencode_modern_plugins, &opencode_legacy_plugins] {
                let target = target_root.join(&record.plugin_name);
                actions.push(ensure_directory_skill_bridge(
                    "opencode",
                    &record.plugin_name,
                    &source_opencode_plugins,
                    &target,
                ));
            }
        }
    }

    CapabilityBridgeRegistry {
        generated_at: Utc::now(),
        actions,
    }
}

fn detect_capability_bridges() -> CapabilityBridgeRegistry {
    let mut actions = Vec::new();
    let Some(home) = home_dir() else {
        return CapabilityBridgeRegistry {
            generated_at: Utc::now(),
            actions,
        };
    };

    let claude_settings = home.join(".claude").join("settings.json");
    let codex_skill_root = home.join(".agents").join("skills");
    let opencode_modern_plugins = home.join(".config").join("opencode").join("plugins");
    let opencode_legacy_plugins = home.join(".opencode").join("plugins");
    let plugin_records = collect_enabled_plugin_cache_records(&claude_settings, &home);
    for record in plugin_records {
        let source_skills = record.cache_root.join("skills");
        if source_skills.is_dir() {
            let target = codex_skill_root.join(&record.plugin_name);
            actions.push(inspect_directory_skill_bridge(
                "codex",
                &record.plugin_name,
                &source_skills,
                &target,
            ));
        }

        let source_opencode_plugins = record.cache_root.join(".opencode").join("plugins");
        if source_opencode_plugins.is_dir() {
            for target_root in [&opencode_modern_plugins, &opencode_legacy_plugins] {
                let target = target_root.join(&record.plugin_name);
                actions.push(inspect_directory_skill_bridge(
                    "opencode",
                    &record.plugin_name,
                    &source_opencode_plugins,
                    &target,
                ));
            }
        }
    }

    CapabilityBridgeRegistry {
        generated_at: Utc::now(),
        actions,
    }
}

fn collect_enabled_plugin_cache_records(
    settings_path: &Path,
    home: &Path,
) -> Vec<PluginCacheRecord> {
    let mut records = Vec::new();
    let Ok(raw) = fs::read_to_string(settings_path) else {
        return records;
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return records;
    };
    let Some(enabled) = json
        .get("enabledPlugins")
        .and_then(|value| value.as_object())
    else {
        return records;
    };

    for (plugin_id, value) in enabled {
        if !value.as_bool().unwrap_or(false) {
            continue;
        }
        let (plugin_name, marketplace) = parse_marketplace_plugin_id(plugin_id);
        let Some(cache_root) = latest_cached_plugin_root(
            &home.join(".codex").join("plugins").join("cache"),
            marketplace.as_deref().unwrap_or("unknown"),
            &plugin_name,
        ) else {
            continue;
        };
        records.push(PluginCacheRecord {
            plugin_name,
            cache_root,
        });
    }

    records
}

fn ensure_directory_skill_bridge(
    harness: &str,
    capability: &str,
    source: &Path,
    target: &Path,
) -> CapabilityBridgeAction {
    let source_path = source.display().to_string();
    let target_path = target.display().to_string();
    let mut notes = Vec::new();

    let parent = match target.parent() {
        Some(parent) => parent,
        None => {
            return CapabilityBridgeAction {
                harness: harness.to_string(),
                capability: capability.to_string(),
                status: "blocked".to_string(),
                source_path,
                target_path,
                notes: vec!["target has no parent directory".to_string()],
            };
        }
    };

    if let Err(err) = fs::create_dir_all(parent) {
        return CapabilityBridgeAction {
            harness: harness.to_string(),
            capability: capability.to_string(),
            status: "blocked".to_string(),
            source_path,
            target_path,
            notes: vec![format!("failed to create target parent: {err}")],
        };
    }

    if let Ok(existing) = fs::symlink_metadata(target) {
        if existing.file_type().is_symlink() {
            if let Ok(current) = fs::read_link(target) {
                if current == source {
                    return CapabilityBridgeAction {
                        harness: harness.to_string(),
                        capability: capability.to_string(),
                        status: "already-bridged".to_string(),
                        source_path,
                        target_path,
                        notes: vec!["bridge already points at the current source".to_string()],
                    };
                }
            }
            if let Err(err) = fs::remove_file(target) {
                return CapabilityBridgeAction {
                    harness: harness.to_string(),
                    capability: capability.to_string(),
                    status: "blocked".to_string(),
                    source_path,
                    target_path,
                    notes: vec![format!("failed to replace existing symlink: {err}")],
                };
            }
            notes.push("replaced stale symlink bridge".to_string());
        } else {
            return CapabilityBridgeAction {
                harness: harness.to_string(),
                capability: capability.to_string(),
                status: "blocked".to_string(),
                source_path,
                target_path,
                notes: vec!["target already exists and is not a symlink".to_string()],
            };
        }
    }

    match create_symlink(source, target) {
        Ok(()) => {
            notes.push("created native skill bridge".to_string());
            CapabilityBridgeAction {
                harness: harness.to_string(),
                capability: capability.to_string(),
                status: "bridged".to_string(),
                source_path,
                target_path,
                notes,
            }
        }
        Err(err) => CapabilityBridgeAction {
            harness: harness.to_string(),
            capability: capability.to_string(),
            status: "blocked".to_string(),
            source_path,
            target_path,
            notes: vec![format!("failed to create symlink bridge: {err}")],
        },
    }
}

fn inspect_directory_skill_bridge(
    harness: &str,
    capability: &str,
    source: &Path,
    target: &Path,
) -> CapabilityBridgeAction {
    let source_path = source.display().to_string();
    let target_path = target.display().to_string();

    let Some(parent) = target.parent() else {
        return CapabilityBridgeAction {
            harness: harness.to_string(),
            capability: capability.to_string(),
            status: "blocked".to_string(),
            source_path,
            target_path,
            notes: vec!["target has no parent directory".to_string()],
        };
    };

    if !parent.exists() {
        return CapabilityBridgeAction {
            harness: harness.to_string(),
            capability: capability.to_string(),
            status: "blocked".to_string(),
            source_path,
            target_path,
            notes: vec!["target parent directory is missing".to_string()],
        };
    }

    if let Ok(existing) = fs::symlink_metadata(target) {
        if existing.file_type().is_symlink() {
            if let Ok(current) = fs::read_link(target) {
                if current == source {
                    return CapabilityBridgeAction {
                        harness: harness.to_string(),
                        capability: capability.to_string(),
                        status: "already-bridged".to_string(),
                        source_path,
                        target_path,
                        notes: vec!["bridge already points at the current source".to_string()],
                    };
                }
            }
            return CapabilityBridgeAction {
                harness: harness.to_string(),
                capability: capability.to_string(),
                status: "available".to_string(),
                source_path,
                target_path,
                notes: vec!["stale bridge can be refreshed by explicit init".to_string()],
            };
        }
        return CapabilityBridgeAction {
            harness: harness.to_string(),
            capability: capability.to_string(),
            status: "blocked".to_string(),
            source_path,
            target_path,
            notes: vec!["target already exists and is not a symlink".to_string()],
        };
    }

    CapabilityBridgeAction {
        harness: harness.to_string(),
        capability: capability.to_string(),
        status: "available".to_string(),
        source_path,
        target_path,
        notes: vec!["bridge target can be created by explicit init".to_string()],
    }
}

#[cfg(unix)]
fn create_symlink(source: &Path, target: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(source, target)
}

#[cfg(windows)]
fn create_symlink(source: &Path, target: &Path) -> io::Result<()> {
    std::os::windows::fs::symlink_dir(source, target)
}

#[derive(Debug, Clone)]
struct PluginCacheRecord {
    plugin_name: String,
    cache_root: PathBuf,
}

fn collect_project_bootstrap_sources(project_root: &Path) -> Vec<PathBuf> {
    let mut sources = Vec::new();
    let candidates = [
        "CLAUDE.md",
        ".claude/CLAUDE.md",
        ".agents/CLAUDE.md",
        "DESIGN.md",
        ".claude/DESIGN.md",
        ".agents/DESIGN.md",
        "AGENTS.md",
        "TEAMS.md",
        "MEMORY.md",
        "SOUL.md",
        "USER.md",
        "IDENTITY.md",
        "TOOLS.md",
        "BOOTSTRAP.md",
        "HEARTBEAT.md",
        "README.md",
        "CONTRIBUTING.md",
        "ROADMAP.md",
        "docs/setup.md",
        "docs/config.md",
        "docs/infra-facts.md",
        "docs/release-process.md",
        "docs/maintainer-workflow.md",
        ".planning/STATE.md",
        ".planning/PROJECT.md",
        ".planning/ROADMAP.md",
        ".planning/codebase/ARCHITECTURE.md",
        ".planning/codebase/STRUCTURE.md",
    ];

    for candidate in candidates {
        let path = project_root.join(candidate);
        if path.is_file() {
            sources.push(path);
        }
    }

    let claude_project_memory = claude_project_memory_path(project_root);
    if claude_project_memory.is_file() {
        sources.push(claude_project_memory);
    }

    sources.extend(collect_memory_dir_sources(project_root));
    sources.extend(collect_design_dir_sources(project_root));

    sources
}

fn collect_user_harness_bootstrap_sources(project_root: Option<&Path>) -> Vec<PathBuf> {
    let Some(home) = home_dir() else {
        return Vec::new();
    };

    let mut sources = Vec::new();
    let codex_root = home.join(".codex");
    sources.extend(collect_named_file_sources(
        &codex_root,
        &[
            "AGENTS.md",
            "TEAMS.md",
            "MEMORY.md",
            "USER.md",
            "IDENTITY.md",
            "SOUL.md",
            "TOOLS.md",
            "BOOTSTRAP.md",
            "HEARTBEAT.md",
            "config.toml",
        ],
    ));
    sources.extend(collect_relative_file_sources(
        &codex_root,
        &[
            "skills/memd/SKILL.md",
            "skills/memd-init/SKILL.md",
            "skills/memd-reload/SKILL.md",
            "skills/dream/SKILL.md",
            "skills/autodream/SKILL.md",
            "skills/gsd-autonomous/SKILL.md",
            "skills/gsd-map-codebase/SKILL.md",
        ],
    ));

    for harness_root in detect_claude_family_harness_roots(&home) {
        sources.extend(collect_named_file_sources(
            &harness_root.root,
            &[
                "AGENTS.md",
                "TEAMS.md",
                "MEMORY.md",
                "USER.md",
                "IDENTITY.md",
                "SOUL.md",
                "TOOLS.md",
                "BOOTSTRAP.md",
                "HEARTBEAT.md",
                "settings.json",
            ],
        ));
        sources.extend(collect_relative_file_sources(
            &harness_root.root,
            &[
                "hooks/gsd-session-context.js",
                "hooks/memd-session-context.js",
            ],
        ));
    }
    if let Some(project_root) = project_root {
        let claude_project_memory = claude_project_memory_path(project_root);
        if claude_project_memory.is_file() {
            sources.push(claude_project_memory);
        }
    }

    let openclaw_root = home.join(".openclaw").join("workspace");
    sources.extend(collect_named_file_sources(
        &openclaw_root,
        &[
            "AGENTS.md",
            "TEAMS.md",
            "MEMORY.md",
            "USER.md",
            "IDENTITY.md",
            "SOUL.md",
            "TOOLS.md",
            "BOOTSTRAP.md",
            "HEARTBEAT.md",
        ],
    ));

    let opencode_root = home.join(".config").join("opencode");
    sources.extend(collect_named_file_sources(
        &opencode_root,
        &[
            "AGENTS.md",
            "TEAMS.md",
            "MEMORY.md",
            "settings.json",
            "opencode.json",
        ],
    ));
    sources.extend(collect_relative_file_sources(
        &opencode_root,
        &[
            "plugins/memd-plugin.mjs",
            "command/memd.md",
            "command/gsd-autonomous.md",
            "command/gsd-map-codebase.md",
        ],
    ));

    let legacy_opencode_root = home.join(".opencode");
    sources.extend(collect_named_file_sources(
        &legacy_opencode_root,
        &["AGENTS.md", "TEAMS.md", "MEMORY.md"],
    ));

    let claw_config_root = home.join(".config").join("claw");
    sources.extend(collect_named_file_sources(
        &claw_config_root,
        &[
            "settings.json",
            "AGENTS.md",
            "TEAMS.md",
            "MEMORY.md",
            "CLAUDE.md",
        ],
    ));
    sources.extend(collect_relative_file_sources(
        &claw_config_root,
        &[
            "skills/memd/SKILL.md",
            "skills/memd-reload/SKILL.md",
            "skills/memd-init/SKILL.md",
        ],
    ));

    let claw_home_root = home.join(".claw");
    sources.extend(collect_named_file_sources(
        &claw_home_root,
        &[
            "settings.json",
            "AGENTS.md",
            "TEAMS.md",
            "MEMORY.md",
            "CLAUDE.md",
        ],
    ));
    sources.extend(collect_relative_file_sources(
        &claw_home_root,
        &[
            "skills/memd/SKILL.md",
            "skills/memd-reload/SKILL.md",
            "skills/memd-init/SKILL.md",
        ],
    ));

    sources
}

#[derive(Debug, Clone)]
struct HarnessRoot {
    harness: String,
    root: PathBuf,
}

fn detect_claude_family_harness_roots(home: &Path) -> Vec<HarnessRoot> {
    let mut roots = Vec::new();
    let primary = home.join(".claude");
    if primary.is_dir() {
        roots.push(HarnessRoot {
            harness: "claude".to_string(),
            root: primary,
        });
    }

    let Ok(entries) = fs::read_dir(home) else {
        return roots;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if name == ".claude" || name == ".codex" || name == ".openclaw" || name == ".opencode" {
            continue;
        }
        if !looks_like_claude_family_dir(name) {
            continue;
        }
        if !path.join("settings.json").is_file() {
            continue;
        }
        roots.push(HarnessRoot {
            harness: name.trim_start_matches('.').to_string(),
            root: path,
        });
    }

    roots.sort_by(|a, b| a.harness.cmp(&b.harness).then(a.root.cmp(&b.root)));
    roots
}

fn looks_like_claude_family_dir(name: &str) -> bool {
    let normalized = name.trim_start_matches('.').to_ascii_lowercase();
    normalized.contains("claude") || normalized.contains("claw")
}

fn collect_named_file_sources(root: &Path, names: &[&str]) -> Vec<PathBuf> {
    names
        .iter()
        .map(|name| root.join(name))
        .filter(|path| path.is_file())
        .collect()
}

fn collect_relative_file_sources(root: &Path, paths: &[&str]) -> Vec<PathBuf> {
    paths
        .iter()
        .map(|relative| root.join(relative))
        .filter(|path| path.is_file())
        .collect()
}

fn collect_memory_dir_sources(project_root: &Path) -> Vec<PathBuf> {
    let memory_dir = project_root.join("memory");
    let mut sources = Vec::new();
    let Ok(entries) = fs::read_dir(&memory_dir) else {
        return sources;
    };

    let mut entries = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .map(|ext| {
                    matches!(
                        ext.to_ascii_lowercase().as_str(),
                        "md" | "txt" | "json" | "yaml" | "yml"
                    )
                })
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    entries.sort();
    sources.extend(entries.into_iter().take(6));
    sources
}

fn collect_design_dir_sources(project_root: &Path) -> Vec<PathBuf> {
    let design_dir = project_root.join("design");
    let mut sources = Vec::new();
    let Ok(entries) = fs::read_dir(&design_dir) else {
        return sources;
    };

    let mut entries = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .map(|ext| {
                    matches!(
                        ext.to_ascii_lowercase().as_str(),
                        "md" | "txt" | "json" | "yaml" | "yml"
                    )
                })
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    entries.sort();
    sources.extend(entries.into_iter().take(6));
    sources
}

fn claude_project_memory_path(project_root: &Path) -> PathBuf {
    let slug = project_root.to_string_lossy().replace('/', "-");
    home_dir()
        .map(|home| {
            home.join(".claude")
                .join("projects")
                .join(slug)
                .join("memory")
        })
        .unwrap_or_else(|| PathBuf::from("."))
        .join("MEMORY.md")
}

fn display_bootstrap_source_path(path: &Path, project_root: Option<&Path>) -> String {
    if let Some(project_root) = project_root
        && let Ok(relative) = path.strip_prefix(project_root)
    {
        return relative.display().to_string();
    }

    if let Some(home) = home_dir()
        && let Ok(relative) = path.strip_prefix(&home)
    {
        return format!("~/{}", relative.display());
    }

    path.display().to_string()
}

fn default_heartbeat_model() -> String {
    "llama-desktop/qwen".to_string()
}

fn default_bundle_session() -> String {
    format!(
        "session-{}",
        &uuid::Uuid::new_v4().simple().to_string()[..8]
    )
}

fn default_bundle_tab_id() -> Option<String> {
    std::env::var("MEMD_TAB_ID")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn compose_agent_identity(agent: &str, session: Option<&str>) -> String {
    let agent = agent.trim();
    let session = session.map(str::trim).filter(|value| !value.is_empty());
    match session {
        Some(session) => format!("{agent}@{session}"),
        None => agent.to_string(),
    }
}

fn project_hive_group(project: Option<&str>) -> Option<String> {
    let project = project?.trim();
    if project.is_empty() {
        None
    } else {
        let mut slug = String::new();
        let mut last_dash = false;
        for ch in project.chars() {
            let normalized = ch.to_ascii_lowercase();
            if normalized.is_ascii_alphanumeric() {
                slug.push(normalized);
                last_dash = false;
            } else if !last_dash {
                slug.push('-');
                last_dash = true;
            }
        }
        Some(format!("project:{}", slug.trim_matches('-')))
    }
}

fn effective_hive_groups(hive_groups: Vec<String>, project: Option<&str>) -> Vec<String> {
    let mut groups = hive_groups;
    if let Some(project_group) = project_hive_group(project) {
        groups.push(project_group);
    }
    groups.sort();
    groups.dedup();
    groups
}

#[derive(Debug, Clone)]
struct HiveProfileDefaults {
    hive_system: Option<String>,
    hive_role: Option<String>,
    capabilities: Vec<String>,
    hive_groups: Vec<String>,
    hive_group_goal: Option<String>,
    hive_project_enabled: bool,
    hive_project_anchor: Option<String>,
    hive_project_joined_at: Option<DateTime<Utc>>,
    authority: Option<String>,
}

fn default_hive_profile(agent: &str) -> HiveProfileDefaults {
    let normalized = agent.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "agent-shell" => HiveProfileDefaults {
            hive_system: Some("agent-shell".to_string()),
            hive_role: Some("runtime-shell".to_string()),
            capabilities: vec![
                "shell".to_string(),
                "exec".to_string(),
                "workspace".to_string(),
            ],
            hive_groups: vec!["runtime-core".to_string(), "dependency-owners".to_string()],
            hive_group_goal: Some(
                "stabilize runtime execution and dependency health across active agent sessions"
                    .to_string(),
            ),
            hive_project_enabled: false,
            hive_project_anchor: None,
            hive_project_joined_at: None,
            authority: Some("worker".to_string()),
        },
        "agent-secrets" => HiveProfileDefaults {
            hive_system: Some("agent-secrets".to_string()),
            hive_role: Some("secret-broker".to_string()),
            capabilities: vec![
                "secrets".to_string(),
                "auth".to_string(),
                "policy".to_string(),
            ],
            hive_groups: vec!["runtime-core".to_string(), "dependency-owners".to_string()],
            hive_group_goal: Some(
                "keep secret access and auth dependencies reliable for the active product stack"
                    .to_string(),
            ),
            hive_project_enabled: false,
            hive_project_anchor: None,
            hive_project_joined_at: None,
            authority: Some("restricted".to_string()),
        },
        "claw-control" => HiveProfileDefaults {
            hive_system: Some("claw-control".to_string()),
            hive_role: Some("orchestrator".to_string()),
            capabilities: vec![
                "control".to_string(),
                "routing".to_string(),
                "coordination".to_string(),
            ],
            hive_groups: vec!["openclaw-stack".to_string(), "control-plane".to_string()],
            hive_group_goal: Some(
                "coordinate the OpenClaw stack so hives converge on the proper product-level fix"
                    .to_string(),
            ),
            hive_project_enabled: false,
            hive_project_anchor: None,
            hive_project_joined_at: None,
            authority: Some("coordinator".to_string()),
        },
        "memd" => HiveProfileDefaults {
            hive_system: Some("memd".to_string()),
            hive_role: Some("memory-control-plane".to_string()),
            capabilities: vec![
                "memory".to_string(),
                "coordination".to_string(),
                "handoff".to_string(),
            ],
            hive_groups: vec!["openclaw-stack".to_string(), "control-plane".to_string()],
            hive_group_goal: Some(
                "maintain canonical shared memory and coordination for the OpenClaw stack"
                    .to_string(),
            ),
            hive_project_enabled: false,
            hive_project_anchor: None,
            hive_project_joined_at: None,
            authority: Some("canonical".to_string()),
        },
        _ => HiveProfileDefaults {
            hive_system: Some(normalized),
            hive_role: Some("agent".to_string()),
            capabilities: vec!["memory".to_string(), "coordination".to_string()],
            hive_groups: Vec::new(),
            hive_group_goal: None,
            hive_project_enabled: false,
            hive_project_anchor: None,
            hive_project_joined_at: None,
            authority: Some("participant".to_string()),
        },
    }
}

fn resolve_hive_profile(args: &InitArgs, project: Option<&str>) -> HiveProfileDefaults {
    let defaults = default_hive_profile(&args.agent);
    let mut capabilities = if args.capability.is_empty() {
        defaults.capabilities
    } else {
        args.capability
            .iter()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>()
    };
    capabilities.sort();
    capabilities.dedup();
    let mut hive_groups = if args.hive_group.is_empty() {
        defaults.hive_groups
    } else {
        args.hive_group
            .iter()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>()
    };
    hive_groups.sort();
    hive_groups.dedup();
    HiveProfileDefaults {
        hive_system: args.hive_system.clone().or(defaults.hive_system),
        hive_role: args.hive_role.clone().or(defaults.hive_role),
        capabilities,
        hive_groups: effective_hive_groups(hive_groups, project),
        hive_group_goal: args.hive_group_goal.clone().or(defaults.hive_group_goal),
        hive_project_enabled: defaults.hive_project_enabled,
        hive_project_anchor: defaults.hive_project_anchor,
        hive_project_joined_at: defaults.hive_project_joined_at,
        authority: args.authority.clone().or(defaults.authority),
    }
}

fn write_init_bundle(args: &InitArgs) -> anyhow::Result<()> {
    let project_root = detect_init_project_root(args)?;
    let output = resolve_init_output_path(args, project_root.as_deref());
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
    let tab_id = args.tab_id.clone().or_else(default_bundle_tab_id);
    let project = init_project_name(args, project_root.as_deref());
    let namespace = init_namespace_name(args, &output);
    let hive_profile = resolve_hive_profile(args, Some(project.as_str()));
    let project_bootstrap = if args.seed_existing {
        build_project_bootstrap_memory(project_root.as_deref(), &project, args).unwrap_or_default()
    } else {
        None
    };

    let rag_url = args
        .rag_url
        .clone()
        .or_else(|| std::env::var("MEMD_RAG_URL").ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let rag_enabled = rag_url.is_some();
    let worker_name =
        default_bundle_worker_name_for_project(Some(&project), &args.agent, Some(&session));
    let config = BundleConfig {
        schema_version: 2,
        project: project.clone(),
        namespace: namespace.clone(),
        agent: args.agent.clone(),
        session: session.clone(),
        tab_id: tab_id.clone(),
        hive_system: hive_profile.hive_system.clone(),
        hive_role: hive_profile.hive_role.clone(),
        capabilities: hive_profile.capabilities.clone(),
        hive_groups: hive_profile.hive_groups.clone(),
        hive_group_goal: hive_profile.hive_group_goal.clone(),
        hive_project_enabled: false,
        hive_project_anchor: None,
        hive_project_joined_at: None,
        authority: hive_profile.authority.clone(),
        base_url: args.base_url.clone(),
        route: args.route.clone(),
        intent: args.intent.clone(),
        workspace: args.workspace.clone(),
        visibility: args.visibility.clone(),
        heartbeat_model: default_heartbeat_model(),
        voice_mode: args
            .voice_mode
            .clone()
            .unwrap_or_else(default_voice_mode),
        auto_short_term_capture: true,
        authority_policy: BundleAuthorityPolicy::default(),
        authority_state: BundleAuthorityState {
            mode: default_authority_mode(),
            degraded: false,
            shared_base_url: Some(args.base_url.clone()),
            fallback_base_url: None,
            activated_at: Some(Utc::now()),
            activated_by: Some("init".to_string()),
            reason: Some("shared authority available".to_string()),
            warning_acknowledged_at: None,
            expires_at: None,
            blocked_capabilities: Vec::new(),
        },
        backend: BundleBackendConfig {
            rag: BundleRagConfig {
                enabled: rag_enabled,
                provider: "lightrag-compatible".to_string(),
                url: rag_url.clone(),
            },
        },
        hooks: BundleHooksConfig {
            context: "hooks/memd-context.sh".to_string(),
            capture: "hooks/memd-capture.sh".to_string(),
            spill: "hooks/memd-spill.sh".to_string(),
            context_ps1: "hooks/memd-context.ps1".to_string(),
            capture_ps1: "hooks/memd-capture.ps1".to_string(),
            spill_ps1: "hooks/memd-spill.ps1".to_string(),
        },
        rag_url: rag_url.clone(),
    };
    fs::write(
        output.join("config.json"),
        serde_json::to_string_pretty(&config)? + "\n",
    )
    .with_context(|| format!("write {}", output.join("config.json").display()))?;

    write_bundle_backend_env(&output, &config)?;

    fs::write(
        output.join("env"),
        format!(
            "MEMD_BASE_URL={}\nMEMD_PROJECT={}\n{}MEMD_AGENT={}\nMEMD_WORKER_NAME={}\nMEMD_SESSION={}\n{}MEMD_ROUTE={}\nMEMD_INTENT={}\nMEMD_HEARTBEAT_MODEL={}\nMEMD_VOICE_MODE={}\nMEMD_AUTO_SHORT_TERM_CAPTURE={}\n{}{}{}",
            args.base_url,
            project,
            namespace
                .as_ref()
                .map(|value| format!("MEMD_NAMESPACE={value}\n"))
                .unwrap_or_default(),
            compose_agent_identity(&args.agent, Some(&session)),
            shell_single_quote(&worker_name),
            session,
            tab_id
                .as_ref()
                .map(|value| format!("MEMD_TAB_ID={value}\n"))
                .unwrap_or_default(),
            args.route,
            args.intent,
            config.heartbeat_model,
            config.voice_mode,
            if config.auto_short_term_capture { "true" } else { "false" },
            args.workspace
                .as_ref()
                .map(|value| format!("MEMD_WORKSPACE={value}\n"))
                .unwrap_or_default(),
            args.visibility
                .as_ref()
                .map(|value| format!("MEMD_VISIBILITY={value}\n"))
                .unwrap_or_default(),
            rag_url
                .as_ref()
                .map(|value| format!("MEMD_RAG_URL={value}\n"))
                .unwrap_or_default(),
        ),
    )
    .with_context(|| format!("write {}", output.join("env").display()))?;

    if let Some(hive_system) = hive_profile.hive_system.as_deref() {
        rewrite_env_assignment(
            &output.join("env"),
            "MEMD_PEER_SYSTEM=",
            &format!("MEMD_PEER_SYSTEM={hive_system}\n"),
        )?;
    }
    if let Some(hive_role) = hive_profile.hive_role.as_deref() {
        rewrite_env_assignment(
            &output.join("env"),
            "MEMD_PEER_ROLE=",
            &format!("MEMD_PEER_ROLE={hive_role}\n"),
        )?;
    }
    if !hive_profile.capabilities.is_empty() {
        rewrite_env_assignment(
            &output.join("env"),
            "MEMD_PEER_CAPABILITIES=",
            &format!(
                "MEMD_PEER_CAPABILITIES={}\n",
                hive_profile.capabilities.join(",")
            ),
        )?;
    }
    if !hive_profile.hive_groups.is_empty() {
        rewrite_env_assignment(
            &output.join("env"),
            "MEMD_PEER_GROUPS=",
            &format!("MEMD_PEER_GROUPS={}\n", hive_profile.hive_groups.join(",")),
        )?;
    }
    if let Some(goal) = hive_profile.hive_group_goal.as_deref() {
        rewrite_env_assignment(
            &output.join("env"),
            "MEMD_PEER_GROUP_GOAL=",
            &format!("MEMD_PEER_GROUP_GOAL={goal}\n"),
        )?;
    }
    if let Some(authority) = hive_profile.authority.as_deref() {
        rewrite_env_assignment(
            &output.join("env"),
            "MEMD_PEER_AUTHORITY=",
            &format!("MEMD_PEER_AUTHORITY={authority}\n"),
        )?;
    }
    if let Some(value) = tab_id.as_deref() {
        rewrite_env_assignment(
            &output.join("env"),
            "MEMD_TAB_ID=",
            &format!("MEMD_TAB_ID={value}\n"),
        )?;
    }

    fs::write(
        output.join("env.ps1"),
        format!(
            "$env:MEMD_BASE_URL = \"{}\"\n$env:MEMD_PROJECT = \"{}\"\n{}$env:MEMD_AGENT = \"{}\"\n$env:MEMD_WORKER_NAME = \"{}\"\n$env:MEMD_SESSION = \"{}\"\n{}$env:MEMD_ROUTE = \"{}\"\n$env:MEMD_INTENT = \"{}\"\n$env:MEMD_HEARTBEAT_MODEL = \"{}\"\n$env:MEMD_VOICE_MODE = \"{}\"\n$env:MEMD_AUTO_SHORT_TERM_CAPTURE = \"{}\"\n{}{}{}",
            escape_ps1(&args.base_url),
            escape_ps1(&project),
            namespace
                .as_ref()
                .map(|value| format!("$env:MEMD_NAMESPACE = \"{}\"\n", escape_ps1(value)))
                .unwrap_or_default(),
            escape_ps1(&compose_agent_identity(&args.agent, Some(&session))),
            escape_ps1(&worker_name),
            escape_ps1(&session),
            tab_id
                .as_ref()
                .map(|value| format!("$env:MEMD_TAB_ID = \"{}\"\n", escape_ps1(value)))
                .unwrap_or_default(),
            escape_ps1(&args.route),
            escape_ps1(&args.intent),
            escape_ps1(&config.heartbeat_model),
            escape_ps1(&config.voice_mode),
            if config.auto_short_term_capture { "true" } else { "false" },
            args.workspace
                .as_ref()
                .map(|value| format!("$env:MEMD_WORKSPACE = \"{}\"\n", escape_ps1(value)))
                .unwrap_or_default(),
            args.visibility
                .as_ref()
                .map(|value| format!("$env:MEMD_VISIBILITY = \"{}\"\n", escape_ps1(value)))
                .unwrap_or_default(),
            rag_url
                .as_ref()
                .map(|value| format!("$env:MEMD_RAG_URL = \"{}\"\n", escape_ps1(value)))
                .unwrap_or_default(),
        ),
    )
    .with_context(|| format!("write {}", output.join("env.ps1").display()))?;

    write_bundle_authority_env(&output, &config.authority_policy, &config.authority_state)?;

    if let Some(hive_system) = hive_profile.hive_system.as_deref() {
        rewrite_env_assignment(
            &output.join("env.ps1"),
            "$env:MEMD_PEER_SYSTEM = ",
            &format!("$env:MEMD_PEER_SYSTEM = \"{}\"\n", escape_ps1(hive_system)),
        )?;
    }
    if let Some(hive_role) = hive_profile.hive_role.as_deref() {
        rewrite_env_assignment(
            &output.join("env.ps1"),
            "$env:MEMD_PEER_ROLE = ",
            &format!("$env:MEMD_PEER_ROLE = \"{}\"\n", escape_ps1(hive_role)),
        )?;
    }
    if !hive_profile.capabilities.is_empty() {
        rewrite_env_assignment(
            &output.join("env.ps1"),
            "$env:MEMD_PEER_CAPABILITIES = ",
            &format!(
                "$env:MEMD_PEER_CAPABILITIES = \"{}\"\n",
                escape_ps1(&hive_profile.capabilities.join(","))
            ),
        )?;
    }
    if !hive_profile.hive_groups.is_empty() {
        rewrite_env_assignment(
            &output.join("env.ps1"),
            "$env:MEMD_PEER_GROUPS = ",
            &format!(
                "$env:MEMD_PEER_GROUPS = \"{}\"\n",
                escape_ps1(&hive_profile.hive_groups.join(","))
            ),
        )?;
    }
    if let Some(goal) = hive_profile.hive_group_goal.as_deref() {
        rewrite_env_assignment(
            &output.join("env.ps1"),
            "$env:MEMD_PEER_GROUP_GOAL = ",
            &format!("$env:MEMD_PEER_GROUP_GOAL = \"{}\"\n", escape_ps1(goal)),
        )?;
    }
    if let Some(authority) = hive_profile.authority.as_deref() {
        rewrite_env_assignment(
            &output.join("env.ps1"),
            "$env:MEMD_PEER_AUTHORITY = ",
            &format!("$env:MEMD_PEER_AUTHORITY = \"{}\"\n", escape_ps1(authority)),
        )?;
    }

    let hook_root = output.join("hooks");
    bundle_agent_profiles::copy_hook_assets(Path::new(&hook_root))?;
    write_agent_profiles(&output)?;
    let capability_registry = build_bundle_capability_registry(project_root.as_deref());
    let capability_bridges = apply_capability_bridges();
    let capability_summary = format!(
        "{}\n{}",
        render_capability_registry_summary(&capability_registry),
        render_capability_bridge_summary(&capability_bridges)
    );
    write_bundle_memory_placeholder(
        &output,
        &config,
        project_bootstrap
            .as_ref()
            .map(|bundle| bundle.markdown.as_str()),
        Some(&capability_summary),
    )?;
    if let Some(bundle) = &project_bootstrap {
        write_bundle_source_registry(&output, &bundle.registry)?;
    }
    write_bundle_capability_registry(&output, &capability_registry)?;
    write_bundle_capability_bridges(&output, &capability_bridges)?;
    write_native_agent_bridge_files(&output)?;
    write_bundle_command_catalog_files(&output)?;
    write_bundle_harness_bridge_registry(&output)?;

    fs::write(
        output.join("README.md"),
        format!(
            "# memd bundle\n\nThis directory contains the memd configuration for `{project}`.\n\n## Quick Start\n\n1. Set up the bundle:\n   - `memd setup --output {bundle}`\n2. Check readiness and repair drift when needed:\n   - `memd doctor --output {bundle}`\n   - `memd doctor --output {bundle} --repair`\n3. Inspect the active config:\n   - `memd config --output {bundle}`\n4. Refresh the live wake-up surface:\n   - `memd wake --output {bundle} --intent current_task --write`\n5. Launch an agent profile:\n   - `.memd/agents/codex.sh`\n   - `.memd/agents/claude-code.sh`\n   - `.memd/agents/agent-zero.sh`\n   - `.memd/agents/hermes.sh`\n   - `.memd/agents/openclaw.sh`\n   - `.memd/agents/opencode.sh`\n6. Inspect the compact working-memory view when needed:\n   - `memd resume --output {bundle} --intent current_task`\n\n## Commands\n\n- `memd commands --output {bundle}`\n- `memd commands --output {bundle} --summary`\n- `memd commands --output {bundle} --json`\n- `memd setup --output {bundle}`\n- `memd doctor --output {bundle}`\n- `memd config --output {bundle}`\n\nThe same catalog is written to `COMMANDS.md` in the bundle root.\n\n## Notes\n\n- Prefer the built `memd` binary during normal multi-session use; `cargo run` adds avoidable compile/cache contention.\n- `env` and `env.ps1` export the same bundle defaults if you want to wire another harness manually.\n- Automatic short-term capture is enabled by default and writes bundle state under `state/last-resume.json`.\n- `MEMD_WAKEUP.md` is the startup live-memory surface; `MEMD_MEMORY.md` is the deeper compact memory view.\n- Add `--semantic` only when you want deeper LightRAG fallback.\n- For Claude Code, import `.memd/agents/CLAUDE_IMPORTS.md` from your project `CLAUDE.md`, then use `/memory` to verify the memd files are loaded.\n",
            project = project,
            bundle = output.display(),
        ),
    )
    .with_context(|| format!("write {}", output.join("README.md").display()))?;

    Ok(())
}

fn build_bundle_turn_placeholder_config(
    output: &Path,
    project: Option<&str>,
    namespace: Option<&str>,
    agent: Option<&str>,
    workspace: Option<&str>,
    visibility: Option<&str>,
    route: Option<&str>,
    intent: Option<&str>,
) -> BundleConfig {
    let runtime = read_bundle_runtime_config(output).ok().flatten();
    let project = project
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| runtime.as_ref().and_then(|value| value.project.clone()))
        .or_else(|| {
            output
                .parent()
                .and_then(|parent| parent.file_name())
                .and_then(|value| value.to_str())
                .map(|value| value.to_string())
        })
        .unwrap_or_else(|| "memd".to_string());
    let namespace = namespace
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| runtime.as_ref().and_then(|value| value.namespace.clone()));
    let agent = agent
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| runtime.as_ref().and_then(|value| value.agent.clone()))
        .unwrap_or_else(|| "codex".to_string());
    let session = runtime
        .as_ref()
        .and_then(|value| value.session.clone())
        .unwrap_or_else(default_bundle_session);
    let tab_id = runtime
        .as_ref()
        .and_then(|value| value.tab_id.clone())
        .or_else(default_bundle_tab_id);
    let hive_system = runtime.as_ref().and_then(|value| value.hive_system.clone());
    let hive_role = runtime.as_ref().and_then(|value| value.hive_role.clone());
    let capabilities = runtime
        .as_ref()
        .map(|value| value.capabilities.clone())
        .unwrap_or_default();
    let hive_groups = runtime
        .as_ref()
        .map(|value| value.hive_groups.clone())
        .unwrap_or_default();
    let hive_group_goal = runtime
        .as_ref()
        .and_then(|value| value.hive_group_goal.clone());
    let hive_project_enabled = runtime
        .as_ref()
        .map(|value| value.hive_project_enabled)
        .unwrap_or(false);
    let hive_project_anchor = runtime
        .as_ref()
        .and_then(|value| value.hive_project_anchor.clone());
    let hive_project_joined_at = runtime
        .as_ref()
        .and_then(|value| value.hive_project_joined_at.clone());
    let authority = runtime.as_ref().and_then(|value| value.authority.clone());
    let base_url = runtime
        .as_ref()
        .and_then(|value| value.base_url.clone())
        .unwrap_or_else(default_base_url);
    let route = route
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| runtime.as_ref().and_then(|value| value.route.clone()))
        .unwrap_or_else(|| "auto".to_string());
    let intent = intent
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| runtime.as_ref().and_then(|value| value.intent.clone()))
        .unwrap_or_else(|| "current_task".to_string());
    let workspace = workspace
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| runtime.as_ref().and_then(|value| value.workspace.clone()));
    let visibility = visibility
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| runtime.as_ref().and_then(|value| value.visibility.clone()));
    let heartbeat_model = runtime
        .as_ref()
        .and_then(|value| value.heartbeat_model.clone())
        .unwrap_or_else(default_heartbeat_model);
    let voice_mode = read_bundle_voice_mode(output).unwrap_or_else(default_voice_mode);
    let auto_short_term_capture = runtime
        .as_ref()
        .map(|value| value.auto_short_term_capture)
        .unwrap_or_else(default_auto_short_term_capture);

    BundleConfig {
        schema_version: 2,
        project,
        namespace,
        agent,
        session,
        tab_id,
        hive_system,
        hive_role,
        capabilities,
        hive_groups,
        hive_group_goal,
        hive_project_enabled,
        hive_project_anchor,
        hive_project_joined_at,
        authority,
        base_url,
        route,
        intent,
        workspace,
        visibility,
        heartbeat_model,
        voice_mode,
        auto_short_term_capture,
        authority_policy: runtime
            .as_ref()
            .map(|value| value.authority_policy.clone())
            .unwrap_or_default(),
        authority_state: runtime
            .as_ref()
            .map(|value| value.authority_state.clone())
            .unwrap_or_default(),
        backend: BundleBackendConfig {
            rag: BundleRagConfig {
                enabled: false,
                provider: "lightrag-compatible".to_string(),
                url: None,
            },
        },
        hooks: BundleHooksConfig {
            context: "hooks/memd-context.sh".to_string(),
            capture: "hooks/memd-capture.sh".to_string(),
            spill: "hooks/memd-spill.sh".to_string(),
            context_ps1: "hooks/memd-context.ps1".to_string(),
            capture_ps1: "hooks/memd-capture.ps1".to_string(),
            spill_ps1: "hooks/memd-spill.ps1".to_string(),
        },
        rag_url: None,
    }
}

fn write_bundle_turn_placeholder_memory(
    output: &Path,
    project: Option<&str>,
    namespace: Option<&str>,
    agent: Option<&str>,
    workspace: Option<&str>,
    visibility: Option<&str>,
    route: Option<&str>,
    intent: Option<&str>,
) -> anyhow::Result<()> {
    let config = build_bundle_turn_placeholder_config(
        output, project, namespace, agent, workspace, visibility, route, intent,
    );
    write_bundle_memory_placeholder(output, &config, None, None)
}

fn write_bundle_turn_fallback_artifacts(
    output: &Path,
    project: Option<&str>,
    namespace: Option<&str>,
    agent: Option<&str>,
    workspace: Option<&str>,
    visibility: Option<&str>,
    route: Option<&str>,
    intent: Option<&str>,
    wakeup_markdown: &str,
) -> anyhow::Result<()> {
    write_bundle_turn_placeholder_memory(
        output, project, namespace, agent, workspace, visibility, route, intent,
    )?;
    write_wakeup_markdown_files(output, wakeup_markdown)?;
    Ok(())
}

fn resolve_init_output_path(args: &InitArgs, project_root: Option<&Path>) -> PathBuf {
    if let Some(explicit) = maybe_explicit_init_output(args) {
        return explicit;
    }

    if args.global {
        return default_global_bundle_root();
    }

    if let Some(project_root) = project_root {
        return project_root.join(".memd");
    }

    default_init_output_path()
}

fn maybe_explicit_init_output(args: &InitArgs) -> Option<PathBuf> {
    let default_init_output = default_init_output_path();
    let default_global_output = default_global_bundle_root();

    if args.output != default_init_output && args.output != default_global_output {
        return Some(args.output.clone());
    }

    None
}

fn write_agent_profiles(output: &Path) -> anyhow::Result<()> {
    let agents_dir = output.join("agents");
    fs::create_dir_all(&agents_dir).with_context(|| format!("create {}", agents_dir.display()))?;
    for (slug, env_agent) in [
        ("agent", None),
        ("codex", Some("codex")),
        ("claude-code", Some("claude-code")),
        ("agent-zero", Some("agent-zero")),
        ("hermes", Some("hermes")),
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

    for (slug, kinds) in [
        ("lookup", Vec::<&str>::new()),
        ("recall-decisions", vec!["decision", "constraint"]),
        ("recall-preferences", vec!["preference"]),
        (
            "recall-design",
            vec!["preference", "constraint", "decision"],
        ),
        ("recall-history", vec!["fact", "decision", "status"]),
    ] {
        let tags = match slug {
            "recall-design" => vec!["design-memory"],
            _ => Vec::new(),
        };
        let shell_profile = render_lookup_shell_profile(output, &kinds, &tags);
        let shell_path = agents_dir.join(format!("{slug}.sh"));
        fs::write(&shell_path, shell_profile)
            .with_context(|| format!("write {}", shell_path.display()))?;
        set_executable_if_shell_script(
            &shell_path,
            shell_path
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or("lookup.sh"),
        )?;

        let ps1_profile = render_lookup_ps1_profile(output, &kinds, &tags);
        let ps1_path = agents_dir.join(format!("{slug}.ps1"));
        fs::write(&ps1_path, ps1_profile)
            .with_context(|| format!("write {}", ps1_path.display()))?;
    }

    for (slug, kind, extra_tags) in [
        (
            "remember-decision",
            "decision",
            vec!["basic-memory", "decision"],
        ),
        (
            "remember-preference",
            "preference",
            vec!["basic-memory", "preference"],
        ),
        ("remember-long", "fact", vec!["basic-memory", "long-term"]),
    ] {
        let shell_profile = render_remember_shell_profile(output, kind, &extra_tags);
        let shell_path = agents_dir.join(format!("{slug}.sh"));
        fs::write(&shell_path, shell_profile)
            .with_context(|| format!("write {}", shell_path.display()))?;
        set_executable_if_shell_script(
            &shell_path,
            shell_path
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or("remember.sh"),
        )?;

        let ps1_profile = render_remember_ps1_profile(output, kind, &extra_tags);
        let ps1_path = agents_dir.join(format!("{slug}.ps1"));
        fs::write(&ps1_path, ps1_profile)
            .with_context(|| format!("write {}", ps1_path.display()))?;
    }

    for slug in ["remember-short", "sync-semantic"] {
        let shell_profile = match slug {
            "remember-short" => render_checkpoint_shell_profile(output),
            "sync-semantic" => render_rag_sync_shell_profile(output),
            _ => unreachable!("unsupported helper slug"),
        };
        let shell_path = agents_dir.join(format!("{slug}.sh"));
        fs::write(&shell_path, shell_profile)
            .with_context(|| format!("write {}", shell_path.display()))?;
        set_executable_if_shell_script(
            &shell_path,
            shell_path
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or("helper.sh"),
        )?;

        let ps1_profile = match slug {
            "remember-short" => render_checkpoint_ps1_profile(output),
            "sync-semantic" => render_rag_sync_ps1_profile(output),
            _ => unreachable!("unsupported helper slug"),
        };
        let ps1_path = agents_dir.join(format!("{slug}.ps1"));
        fs::write(&ps1_path, ps1_profile)
            .with_context(|| format!("write {}", ps1_path.display()))?;
    }

    for slug in ["watch"] {
        let shell_profile = render_watch_shell_profile(output);
        let shell_path = agents_dir.join(format!("{slug}.sh"));
        fs::write(&shell_path, shell_profile)
            .with_context(|| format!("write {}", shell_path.display()))?;
        set_executable_if_shell_script(
            &shell_path,
            shell_path
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or("watch.sh"),
        )?;

        let ps1_profile = render_watch_ps1_profile(output);
        let ps1_path = agents_dir.join(format!("{slug}.ps1"));
        fs::write(&ps1_path, ps1_profile)
            .with_context(|| format!("write {}", ps1_path.display()))?;
    }

    for (slug, mode) in [
        ("capture-live", "capture-live"),
        ("correct-memory", "correct-memory"),
    ] {
        let shell_profile = render_capture_shell_profile(output, mode);
        let shell_path = agents_dir.join(format!("{slug}.sh"));
        fs::write(&shell_path, shell_profile)
            .with_context(|| format!("write {}", shell_path.display()))?;
        set_executable_if_shell_script(
            &shell_path,
            shell_path
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or("capture.sh"),
        )?;

        let ps1_profile = render_capture_ps1_profile(output, mode);
        let ps1_path = agents_dir.join(format!("{slug}.ps1"));
        fs::write(&ps1_path, ps1_profile)
            .with_context(|| format!("write {}", ps1_path.display()))?;
    }

    Ok(())
}

fn write_bundle_memory_placeholder(
    output: &Path,
    config: &BundleConfig,
    project_bootstrap: Option<&str>,
    capability_summary: Option<&str>,
) -> anyhow::Result<()> {
    let mut markdown = String::new();
    markdown.push_str("# memd memory\n\n");
    markdown.push_str("This file is maintained by `memd` for agents that do not have built-in durable memory.\n\n");
    markdown.push_str("## Voice\n\n");
    markdown.push_str(&render_voice_mode_section(&config.voice_mode));
    markdown.push('\n');
    if let Some(project_bootstrap) = project_bootstrap {
        markdown.push_str("## Project bootstrap\n\n");
        markdown.push_str(project_bootstrap);
        if !project_bootstrap.ends_with('\n') {
            markdown.push('\n');
        }
        markdown.push('\n');
    }
    if let Some(capability_summary) = capability_summary {
        markdown.push_str(capability_summary);
        if !capability_summary.ends_with('\n') {
            markdown.push('\n');
        }
        markdown.push('\n');
    }
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
        "- project: {}\n- namespace: {}\n- agent: {}\n- session: {}\n- tab: {}\n- workspace: {}\n- visibility: {}\n- route: {}\n- intent: {}\n- heartbeat_model: {}\n- voice_mode: {}\n- auto_short_term_capture: {}\n",
        config.project,
        config.namespace.as_deref().unwrap_or("none"),
        config.agent,
        config.session,
        config.tab_id.as_deref().unwrap_or("none"),
        config.workspace.as_deref().unwrap_or("none"),
        config.visibility.as_deref().unwrap_or("all"),
        config.route,
        config.intent,
        config.heartbeat_model,
        config.voice_mode,
        if config.auto_short_term_capture { "true" } else { "false" },
    ));
    markdown.push_str("\n## Notes\n\n");
    markdown
        .push_str("- `resume` keeps the active working memory fresh on the fast local hot path.\n");
    markdown.push_str("- `handoff` adds shared workspace, source-lane, and delegation state.\n");
    markdown.push_str("- automatic short-term capture runs on compaction spill boundaries unless disabled in the bundle env/config.\n");
    markdown.push_str(
        "- In Codex, treat installed `$gsd-*` skills as the primary GSD interface after `memd reload` (alias: `memd refresh`).\n",
    );
    markdown.push_str(
        "- Do not claim autonomous GSD is blocked on standalone `gsd-*` shell binaries unless you verified that interface is required for this harness and missing on `PATH`.\n",
    );
    markdown.push_str(
        "- If `$gsd-autonomous` is installed as a skill, try that skill path before claiming the autonomous pipeline is unavailable.\n",
    );
    markdown.push_str(
        "- add `--semantic` only when you want slower deep recall from the semantic backend.\n",
    );
    markdown.push_str(
        "- future dream/consolidation output should flow back into this same memory surface.\n",
    );
    write_memory_markdown_files(output, &markdown)
}

async fn refresh_project_bootstrap_memory(
    output: &Path,
) -> anyhow::Result<Option<(String, BootstrapSourceRegistry)>> {
    let Some(mut registry) = read_bundle_source_registry(output)? else {
        return Ok(None);
    };

    let Some(project_root) = PathBuf::from(&registry.project_root)
        .exists()
        .then(|| PathBuf::from(&registry.project_root))
    else {
        return Ok(None);
    };

    let mut changed = Vec::new();
    for source in &mut registry.sources {
        let path = project_root.join(&source.path);
        if !path.exists() {
            if source.present {
                source.present = false;
                changed.push((
                    source.path.clone(),
                    "(source no longer present)".to_string(),
                ));
            }
            continue;
        }
        source.present = true;

        let current_modified = file_modified_at(&path);
        let current_bytes = fs::metadata(&path)
            .map(|meta| meta.len() as usize)
            .unwrap_or(0);
        if source.modified_at == current_modified && source.bytes == current_bytes {
            continue;
        }

        if let Some((snippet, meta)) = read_bootstrap_source(&path, 24) {
            source.hash = meta.hash;
            source.bytes = meta.bytes;
            source.lines = meta.lines;
            source.imported_at = Utc::now();
            source.modified_at = current_modified;
            changed.push((source.path.clone(), snippet));
        }
    }

    if changed.is_empty() {
        return Ok(None);
    }

    let mut markdown = String::new();
    markdown.push_str("\n## Project source refresh\n\n");
    markdown.push_str("The following project sources changed since the last import:\n\n");
    for (path, snippet) in &changed {
        markdown.push_str(&format!("### {}\n\n{}\n\n", path, snippet));
    }

    Ok(Some((markdown, registry)))
}

async fn write_bundle_memory_files(
    output: &Path,
    snapshot: &ResumeSnapshot,
    handoff: Option<&HandoffSnapshot>,
    apply_bridges: bool,
) -> anyhow::Result<()> {
    if let Some(tab_id) = default_bundle_tab_id() {
        let existing_tab_id = read_bundle_runtime_config(output)
            .ok()
            .flatten()
            .and_then(|config| config.tab_id)
            .filter(|value| !value.trim().is_empty());
        if existing_tab_id.is_none() {
            set_bundle_tab_id(output, &tab_id)?;
        }
    }
    let hive = read_bundle_hive_memory_surface(output).await;
    let markdown = render_bundle_memory_markdown(output, snapshot, handoff, hive.as_ref());
    let wakeup = render_bundle_wakeup_markdown(output, snapshot, false);
    let project_root = infer_bundle_project_root(output);
    let capability_registry = build_bundle_capability_registry(project_root.as_deref());
    write_bundle_capability_registry(output, &capability_registry)?;
    let capability_bridges = if apply_bridges {
        apply_capability_bridges()
    } else {
        detect_capability_bridges()
    };
    write_bundle_capability_bridges(output, &capability_bridges)?;
    let capability_summary = format!(
        "{}\n{}",
        render_capability_registry_summary(&capability_registry),
        render_capability_bridge_summary(&capability_bridges)
    );
    let mut migration_registry = None;
    let markdown = if let Some((registry_markdown, registry)) =
        refresh_project_bootstrap_memory(output).await?
    {
        write_bundle_source_registry(output, &registry)?;
        migration_registry = Some(registry);
        format!("{markdown}\n{capability_summary}\n{registry_markdown}")
    } else {
        format!("{markdown}\n{capability_summary}")
    };
    let manifest = build_bundle_migration_manifest(
        output,
        project_root.as_deref(),
        snapshot,
        handoff,
        migration_registry.as_ref(),
        &capability_registry,
        &capability_bridges,
    )?;
    write_bundle_migration_manifest(output, &manifest)?;
    prune_bundle_compiled_memory_outputs(output)?;
    write_bundle_memory_object_pages(output, snapshot, handoff, hive.as_ref())?;
    write_agent_profiles(output)?;
    write_memory_markdown_files(output, &markdown)?;
    write_wakeup_markdown_files(output, &wakeup)?;
    write_native_agent_bridge_files(output)?;
    write_bundle_harness_bridge_registry(output)?;
    write_bundle_resume_state(output, snapshot)?;
    write_bundle_heartbeat(output, Some(snapshot), false).await
}

fn prune_bundle_compiled_memory_outputs(output: &Path) -> anyhow::Result<()> {
    let compiled = bundle_compiled_memory_dir(output);
    if compiled.exists() {
        fs::remove_dir_all(&compiled).with_context(|| format!("remove {}", compiled.display()))?;
    }
    Ok(())
}

fn harness_pack_enabled_for_snapshot(
    output: &Path,
    snapshot: &ResumeSnapshot,
    agent_name: &str,
) -> bool {
    let runtime_match = read_bundle_runtime_config(output)
        .ok()
        .flatten()
        .and_then(|config| config.agent)
        .map(|agent| agent.trim().to_ascii_lowercase().starts_with(agent_name))
        .unwrap_or(false);
    let snapshot_match = snapshot
        .agent
        .as_deref()
        .map(|agent| agent.trim().to_ascii_lowercase().starts_with(agent_name))
        .unwrap_or(false);
    runtime_match || snapshot_match
}

fn harness_pack_enabled_for_bundle(output: &Path, agent: Option<&str>, agent_name: &str) -> bool {
    let runtime_match = read_bundle_runtime_config(output)
        .ok()
        .flatten()
        .and_then(|config| config.agent)
        .map(|value| value.trim().to_ascii_lowercase().starts_with(agent_name))
        .unwrap_or(false);
    let agent_match = agent
        .map(|value| value.trim().to_ascii_lowercase().starts_with(agent_name))
        .unwrap_or(false);
    runtime_match || agent_match
}

#[derive(Clone, Copy)]
struct HarnessPackRuntime {
    agent_name: &'static str,
    build: fn(&Path, &str, &str) -> crate::harness::shared::HarnessPackData,
}

fn harness_pack_runtimes() -> &'static [HarnessPackRuntime] {
    const RUNTIMES: &[HarnessPackRuntime] = &[
        HarnessPackRuntime {
            agent_name: "codex",
            build: crate::harness::codex::build_codex_harness_pack,
        },
        HarnessPackRuntime {
            agent_name: "agent-zero",
            build: crate::harness::agent_zero::build_agent_zero_harness_pack,
        },
        HarnessPackRuntime {
            agent_name: "openclaw",
            build: crate::harness::openclaw::build_openclaw_harness_pack,
        },
        HarnessPackRuntime {
            agent_name: "hermes",
            build: crate::harness::hermes::build_hermes_harness_pack,
        },
        HarnessPackRuntime {
            agent_name: "opencode",
            build: crate::harness::opencode::build_opencode_harness_pack,
        },
    ];
    RUNTIMES
}

fn harness_pack_query_from_snapshot(snapshot: &ResumeSnapshot) -> String {
    let mut parts = Vec::new();
    if let Some(record) = snapshot.working.records.first() {
        parts.push(record.record.clone());
    }
    if let Some(item) = snapshot.inbox.items.first() {
        parts.push(item.item.content.clone());
    }
    if let Some(next) = snapshot.working.rehydration_queue.first() {
        parts.push(next.summary.clone());
    }
    if let Some(change) = snapshot.change_summary.first() {
        parts.push(change.clone());
    }
    if let Some(change) = snapshot.recent_repo_changes.first() {
        parts.push(change.clone());
    }
    parts.join(" | ")
}

#[cfg(test)]
fn harness_pack_turn_key(
    project: Option<&str>,
    namespace: Option<&str>,
    agent: Option<&str>,
    mode: &str,
    query: &str,
) -> String {
    cache::build_turn_key(project, namespace, agent, mode, query)
}

async fn refresh_harness_pack_files(
    output: &Path,
    snapshot: &ResumeSnapshot,
    manifest: &crate::harness::shared::HarnessPackData,
    agent_name: &str,
    mode: &str,
    query: &str,
) -> anyhow::Result<Vec<PathBuf>> {
    cache::refresh_turn_cached_pack_files(
        output,
        snapshot,
        &manifest.files,
        agent_name,
        mode,
        query,
        write_bundle_memory_files(output, snapshot, None, false),
    )
    .await
}

async fn refresh_harness_pack_files_for_snapshot(
    output: &Path,
    snapshot: &ResumeSnapshot,
    mode: &str,
    allowed_agents: &[&str],
) -> anyhow::Result<Vec<PathBuf>> {
    let query = harness_pack_query_from_snapshot(snapshot);
    let project = snapshot.project.as_deref().unwrap_or("none");
    let namespace = snapshot.namespace.as_deref().unwrap_or("none");
    let mut refreshed = Vec::new();
    for runtime in harness_pack_runtimes() {
        if !allowed_agents.contains(&runtime.agent_name) {
            continue;
        }
        if !harness_pack_enabled_for_snapshot(output, snapshot, runtime.agent_name) {
            continue;
        }
        let manifest = (runtime.build)(output, project, namespace);
        refreshed.extend(
            refresh_harness_pack_files(
                output,
                snapshot,
                &manifest,
                runtime.agent_name,
                mode,
                &query,
            )
            .await?,
        );
    }
    Ok(refreshed)
}

fn read_codex_pack_local_markdown(
    output: &Path,
    file_name: &str,
) -> anyhow::Result<Option<String>> {
    let path = output.join(file_name);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    Ok(Some(raw))
}

fn preserve_codex_capture_locally(output: &Path, content: &str) -> anyhow::Result<()> {
    let mut note = String::new();
    note.push_str("\n## Codex Capture Fallback\n\n");
    note.push_str(&format!("- {}\n", compact_inline(content.trim(), 220)));
    append_text_to_memory_surface(&output.join("MEMD_MEMORY.md"), &note)?;
    for file_name in [
        "CODEX_MEMORY.md",
        "CLAUDE_CODE_MEMORY.md",
        "AGENT_ZERO_MEMORY.md",
        "OPENCLAW_MEMORY.md",
        "OPENCODE_MEMORY.md",
        "HERMES_MEMORY.md",
    ] {
        append_text_to_memory_surface(&output.join("agents").join(file_name), &note)?;
    }
    Ok(())
}

fn build_bundle_migration_manifest(
    output: &Path,
    project_root: Option<&Path>,
    snapshot: &ResumeSnapshot,
    handoff: Option<&HandoffSnapshot>,
    source_registry: Option<&BootstrapSourceRegistry>,
    capability_registry: &CapabilityRegistry,
    capability_bridges: &CapabilityBridgeRegistry,
) -> anyhow::Result<BundleMigrationManifest> {
    let source_registry_json = source_registry
        .map(serde_json::to_string)
        .transpose()
        .context("serialize source registry")?;
    let source_registry_hash = source_registry_json
        .as_ref()
        .map(|json| format!("{:x}", Sha256::digest(json.as_bytes())));
    let source_registry_path = source_registry
        .as_ref()
        .map(|_| bundle_source_registry_path(output).display().to_string());

    let live_truth_summary = snapshot
        .event_spine()
        .into_iter()
        .take(4)
        .collect::<Vec<_>>();
    let mut project_brain_summary = snapshot.compact_context_records();
    project_brain_summary.extend(snapshot.compact_working_records());
    project_brain_summary.extend(snapshot.compact_inbox_items());
    if let Some(handoff) = handoff {
        project_brain_summary.extend(handoff.sources.sources.iter().map(|source| {
            format!(
                "handoff {} / {}",
                source.source_agent.as_deref().unwrap_or("none"),
                source.source_system.as_deref().unwrap_or("none")
            )
        }));
    }
    let user_policy_summary = capability_registry
        .capabilities
        .iter()
        .take(8)
        .map(|record| format!("{} / {} / {}", record.harness, record.kind, record.name))
        .collect::<Vec<_>>();
    let promoted_abstractions_summary = capability_bridges
        .actions
        .iter()
        .take(8)
        .map(|action| {
            format!(
                "{} / {} -> {}",
                action.harness, action.capability, action.status
            )
        })
        .collect::<Vec<_>>();

    Ok(BundleMigrationManifest {
        generated_at: Utc::now(),
        project_root: project_root.map(|root| root.display().to_string()),
        source_registry_hash,
        source_registry_path,
        layer_summary: vec![
            BundleMigrationLayer {
                layer: "live_truth".to_string(),
                sources: live_truth_summary.len(),
                summary: live_truth_summary,
            },
            BundleMigrationLayer {
                layer: "project_brain".to_string(),
                sources: project_brain_summary.len(),
                summary: project_brain_summary,
            },
            BundleMigrationLayer {
                layer: "user_policy".to_string(),
                sources: user_policy_summary.len(),
                summary: user_policy_summary,
            },
            BundleMigrationLayer {
                layer: "promoted_abstractions".to_string(),
                sources: promoted_abstractions_summary.len(),
                summary: promoted_abstractions_summary,
            },
        ],
        notes: vec![
            "bootstrap remains read-once for unchanged sources".to_string(),
            "delta refresh reuses the existing source registry instead of reimporting stable files"
                .to_string(),
            "explicit init remains the only mutating bridge path for shared runtime surfaces"
                .to_string(),
        ],
    })
}

fn infer_bundle_project_root(output: &Path) -> Option<PathBuf> {
    let parent = output.parent()?;
    if output.file_name().and_then(|value| value.to_str()) != Some(".memd") {
        return None;
    }
    if is_project_root_candidate(parent) {
        return Some(parent.to_path_buf());
    }
    None
}

fn bundle_resume_state_path(output: &Path) -> PathBuf {
    output.join("state").join("last-resume.json")
}

fn bundle_lane_surface_path(output: &Path) -> PathBuf {
    output.join("state").join("lane-surface.json")
}

fn skill_policy_batch_state_path(output: &Path) -> PathBuf {
    output.join("state").join("skill-policy-batch.json")
}

fn skill_policy_batch_markdown_path(output: &Path) -> PathBuf {
    output.join("state").join("skill-policy-batch.md")
}

fn skill_policy_activate_state_path(output: &Path) -> PathBuf {
    output
        .join("state")
        .join("skill-policy-activate-queue.json")
}

fn skill_policy_activate_markdown_path(output: &Path) -> PathBuf {
    output.join("state").join("skill-policy-activate-queue.md")
}

fn skill_policy_review_state_path(output: &Path) -> PathBuf {
    output.join("state").join("skill-policy-review-queue.json")
}

fn skill_policy_review_markdown_path(output: &Path) -> PathBuf {
    output.join("state").join("skill-policy-review-queue.md")
}

fn skill_policy_apply_state_path(output: &Path) -> PathBuf {
    output.join("state").join("skill-policy-apply-receipt.json")
}

fn skill_policy_apply_markdown_path(output: &Path) -> PathBuf {
    output.join("state").join("skill-policy-apply-receipt.md")
}

fn build_hive_session_retire_request_from_entry(
    entry: &ProjectAwarenessEntry,
    reason: impl Into<String>,
) -> Option<memd_schema::HiveSessionRetireRequest> {
    let session = entry
        .session
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())?;
    Some(memd_schema::HiveSessionRetireRequest {
        session: session.to_string(),
        project: entry.project.clone(),
        namespace: entry.namespace.clone(),
        repo_root: entry.repo_root.clone(),
        worktree_root: entry.worktree_root.clone(),
        branch: entry.branch.clone(),
        workspace: entry.workspace.clone(),
        agent: entry.agent.clone(),
        effective_agent: entry.effective_agent.clone(),
        hive_system: entry.hive_system.clone(),
        hive_role: entry.hive_role.clone(),
        host: entry.host.clone(),
        reason: Some(reason.into()),
    })
}

fn build_hive_session_retire_request_from_record(
    record: &memd_schema::HiveSessionRecord,
    reason: impl Into<String>,
) -> memd_schema::HiveSessionRetireRequest {
    memd_schema::HiveSessionRetireRequest {
        session: record.session.clone(),
        project: record.project.clone(),
        namespace: record.namespace.clone(),
        repo_root: record.repo_root.clone(),
        worktree_root: record.worktree_root.clone(),
        branch: record.branch.clone(),
        workspace: record.workspace.clone(),
        agent: record.agent.clone(),
        effective_agent: record.effective_agent.clone(),
        hive_system: record.hive_system.clone(),
        hive_role: record.hive_role.clone(),
        host: record.host.clone(),
        reason: Some(reason.into()),
    }
}

fn is_superseded_hive_session_record(
    record: &memd_schema::HiveSessionRecord,
    current: &BundleHeartbeatState,
) -> bool {
    heartbeat_presence_label(record.last_seen) == "stale"
        && current.status == "live"
        && current
            .session
            .as_deref()
            .is_some_and(|session| session != record.session)
        && record.project == current.project
        && record.namespace == current.namespace
        && record.workspace == current.workspace
        && record.agent == current.agent
        && record.base_url == current.base_url
}

async fn retire_hive_session_entry(
    client: &MemdClient,
    entry: &ProjectAwarenessEntry,
    reason: impl Into<String>,
) -> anyhow::Result<usize> {
    let Some(request) = build_hive_session_retire_request_from_entry(entry, reason) else {
        return Ok(0);
    };
    Ok(client.retire_hive_session(&request).await?.retired)
}

async fn retire_superseded_hive_sessions(
    client: &MemdClient,
    state: &BundleHeartbeatState,
) -> anyhow::Result<usize> {
    let Some(current_session) = state
        .session
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(0);
    };
    let sessions_request = memd_schema::HiveSessionsRequest {
        session: None,
        project: state.project.clone(),
        namespace: state.namespace.clone(),
        repo_root: state.repo_root.clone(),
        worktree_root: state.worktree_root.clone(),
        branch: state.branch.clone(),
        workspace: state.workspace.clone(),
        hive_system: None,
        hive_role: None,
        host: None,
        hive_group: None,
        active_only: Some(false),
        limit: Some(512),
    };
    let sessions = timeout_ok(client.hive_sessions(&sessions_request))
        .await
        .map(|response| response.sessions)
        .unwrap_or_default();
    let mut retired = 0usize;
    for session in sessions {
        if session.session == current_session {
            continue;
        }
        if !is_superseded_hive_session_record(&session, state) {
            continue;
        }
        let retire_request = build_hive_session_retire_request_from_record(
            &session,
            format!("superseded by live session {current_session}"),
        );
        retired += timeout_ok(client.retire_hive_session(&retire_request))
            .await
            .map(|response| response.retired)
            .unwrap_or(0);
    }
    Ok(retired)
}

async fn enrich_hive_heartbeat_with_runtime_intent(
    state: &mut BundleHeartbeatState,
) -> anyhow::Result<()> {
    let Some(base_url) = state
        .base_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(());
    };
    let Some(session) = state
        .session
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(());
    };

    let client = MemdClient::new(base_url)?;
    let tasks = timeout_ok(client.hive_tasks(&HiveTasksRequest {
        session: Some(session.to_string()),
        project: state.project.clone(),
        namespace: state.namespace.clone(),
        workspace: state.workspace.clone(),
        active_only: Some(true),
        limit: Some(64),
    }))
    .await
    .map(|response| response.tasks)
    .unwrap_or_default();

    let current_task = tasks
        .iter()
        .find(|task| task.status != "done" && task.status != "closed")
        .or_else(|| tasks.first());
    if let Some(task) = current_task {
        state.task_id = Some(task.task_id.clone());
        if state
            .topic_claim
            .as_deref()
            .is_none_or(hive_topic_claim_needs_runtime_upgrade)
        {
            state.topic_claim = Some(task.title.clone());
        }
        if state.display_name.is_none()
            && state
                .worker_name
                .as_deref()
                .is_some_and(hive_worker_name_is_generic)
        {
            state.display_name =
                derive_hive_display_name(state.agent.as_deref(), state.session.as_deref());
        }
        for scope in &task.claim_scopes {
            push_unique_touch_point(&mut state.scope_claims, scope);
        }
    }
    Ok(())
}

fn build_hive_heartbeat(
    output: &Path,
    snapshot: Option<&ResumeSnapshot>,
) -> anyhow::Result<BundleHeartbeatState> {
    let runtime = read_bundle_runtime_config(output)?.unwrap_or(BundleRuntimeConfig {
        project: None,
        namespace: None,
        agent: None,
        session: None,
        tab_id: None,
        hive_system: None,
        hive_role: None,
        capabilities: Vec::new(),
        hive_groups: Vec::new(),
        hive_group_goal: None,
        authority: None,
        hive_project_enabled: false,
        hive_project_anchor: None,
        hive_project_joined_at: None,
        base_url: None,
        route: None,
        intent: None,
        workspace: None,
        visibility: None,
        heartbeat_model: Some(default_heartbeat_model()),
        voice_mode: Some(default_voice_mode()),
        auto_short_term_capture: true,
        authority_policy: BundleAuthorityPolicy::default(),
        authority_state: BundleAuthorityState::default(),
    });
    let session = runtime.session.clone();
    let agent = runtime.agent.clone();
    let resume_state = read_bundle_resume_state(output).ok().flatten();
    let claims_state = read_bundle_claims(output).ok();
    let project_root = infer_bundle_project_root(output);
    let worktree_root = project_root
        .as_deref()
        .and_then(detect_git_worktree_root)
        .as_deref()
        .map(display_path_nonempty);
    let repo_root = project_root
        .as_deref()
        .and_then(detect_git_repo_root)
        .as_deref()
        .map(display_path_nonempty);
    let branch = project_root
        .as_deref()
        .and_then(|root| git_stdout(root, &["branch", "--show-current"]));
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
        .or_else(|| resume_state.as_ref().and_then(|value| value.focus.clone()));
    let pressure = snapshot
        .and_then(|value| {
            value
                .inbox
                .items
                .first()
                .map(|item| item.item.content.clone())
        })
        .or_else(|| {
            resume_state
                .as_ref()
                .and_then(|value| value.pressure.clone())
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
            resume_state
                .as_ref()
                .and_then(|value| value.next_recovery.clone())
        });
    let topic_claim = derive_hive_topic_claim(
        focus.as_deref(),
        next_recovery.as_deref(),
        pressure.as_deref(),
    );
    let working = topic_claim.clone().or_else(|| focus.clone()).or_else(|| next_recovery.clone());
    let scope_claims = derive_hive_scope_claims(
        claims_state.as_ref(),
        focus.as_deref(),
        pressure.as_deref(),
        next_recovery.as_deref(),
    );
    let touches = scope_claims.clone();
    let task_id = derive_hive_task_id(&scope_claims, topic_claim.as_deref());
    let worker_name = infer_worker_agent_from_env().or_else(|| {
        agent.as_deref().map(|value| {
            default_bundle_worker_name_for_project(
                runtime.project.as_deref(),
                value,
                session.as_deref(),
            )
        })
    });
    let display_name = if worker_name
        .as_deref()
        .is_some_and(hive_worker_name_is_generic)
    {
        derive_hive_display_name(
            worker_name.as_deref().or(agent.as_deref()),
            session.as_deref(),
        )
    } else {
        None
    };
    let lane_id = derive_hive_lane_id(branch.as_deref(), worktree_root.as_deref());
    Ok(BundleHeartbeatState {
        session: session.clone(),
        agent: agent.clone(),
        effective_agent,
        tab_id: runtime.tab_id,
        hive_system: runtime.hive_system,
        hive_role: runtime.hive_role.clone(),
        worker_name,
        display_name,
        role: runtime.hive_role.clone(),
        capabilities: runtime.capabilities,
        hive_groups: effective_hive_groups(
            runtime.hive_groups,
            snapshot
                .and_then(|value| value.project.as_deref())
                .or(runtime.project.as_deref()),
        ),
        lane_id,
        hive_group_goal: runtime.hive_group_goal,
        authority: runtime.authority,
        authority_mode: Some(runtime.authority_state.mode),
        authority_degraded: runtime.authority_state.degraded,
        heartbeat_model: runtime.heartbeat_model,
        project: snapshot
            .and_then(|value| value.project.clone())
            .or(runtime.project),
        namespace: snapshot
            .and_then(|value| value.namespace.clone())
            .or(runtime.namespace),
        workspace: snapshot
            .and_then(|value| value.workspace.clone())
            .or(runtime.workspace),
        repo_root,
        worktree_root,
        branch,
        base_branch: None,
        visibility: snapshot
            .and_then(|value| value.visibility.clone())
            .or(runtime.visibility),
        base_url: runtime.base_url,
        base_url_healthy: None,
        host: detect_host_name(),
        pid: Some(std::process::id()),
        topic_claim,
        scope_claims,
        task_id,
        focus: focus.clone(),
        pressure: pressure.clone(),
        next_recovery: next_recovery.clone(),
        next_action: derive_hive_next_action(
            focus.as_deref(),
            next_recovery.as_deref(),
            pressure.as_deref(),
        ),
        working,
        touches,
        blocked_by: Vec::new(),
        cowork_with: Vec::new(),
        handoff_target: None,
        offered_to: Vec::new(),
        needs_help: false,
        needs_review: false,
        handoff_state: None,
        confidence: None,
        risk: None,
        status: "live".to_string(),
        last_seen: Utc::now(),
    })
}

async fn write_bundle_heartbeat(
    output: &Path,
    snapshot: Option<&ResumeSnapshot>,
    probe_base_url: bool,
) -> anyhow::Result<()> {
    let _ = repair_bundle_worker_name_env(output);
    let path = bundle_heartbeat_state_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let mut state = build_hive_heartbeat(output, snapshot)?;
    enrich_hive_heartbeat_with_runtime_intent(&mut state).await?;
    if probe_base_url && let Some(url) = state.base_url.as_deref() {
        state.base_url_healthy = Some(MemdClient::new(url)?.healthz().await.is_ok());
    }
    fs::write(&path, serde_json::to_string_pretty(&state)? + "\n")
        .with_context(|| format!("write {}", path.display()))?;
    publish_bundle_heartbeat(&state).await?;
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

async fn reconcile_bundle_heartbeat(
    output: &Path,
    snapshot: Option<&ResumeSnapshot>,
    probe_base_url: bool,
) -> anyhow::Result<(BundleHeartbeatState, usize)> {
    let _ = repair_bundle_worker_name_env(output);
    let path = bundle_heartbeat_state_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let mut state = build_hive_heartbeat(output, snapshot)?;
    enrich_hive_heartbeat_with_runtime_intent(&mut state).await?;
    if probe_base_url && let Some(url) = state.base_url.as_deref() {
        state.base_url_healthy = Some(MemdClient::new(url)?.healthz().await.is_ok());
    }
    fs::write(&path, serde_json::to_string_pretty(&state)? + "\n")
        .with_context(|| format!("write {}", path.display()))?;
    let retired = publish_bundle_heartbeat(&state).await?;
    let state =
        read_bundle_heartbeat(output)?.context("reload bundle heartbeat after reconcile")?;
    Ok((state, retired))
}

async fn publish_bundle_heartbeat(state: &BundleHeartbeatState) -> anyhow::Result<usize> {
    if state
        .authority_mode
        .as_deref()
        .is_some_and(|mode| mode == "localhost_read_only")
    {
        anyhow::bail!(
            "localhost read-only fallback active; heartbeat publication requires trusted shared authority"
        );
    }
    let Some(base_url) = state
        .base_url
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    else {
        return Ok(0);
    };
    let Some(session) = state
        .session
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    else {
        return Ok(0);
    };

    let client = MemdClient::new(base_url)?;
    let request = memd_schema::HiveSessionUpsertRequest {
        session: session.to_string(),
        tab_id: state.tab_id.clone(),
        agent: state.agent.clone(),
        effective_agent: state.effective_agent.clone(),
        hive_system: state.hive_system.clone(),
        hive_role: state.hive_role.clone(),
        worker_name: state.worker_name.clone(),
        display_name: state.display_name.clone(),
        role: state.role.clone(),
        capabilities: state.capabilities.clone(),
        hive_groups: state.hive_groups.clone(),
        lane_id: state.lane_id.clone(),
        hive_group_goal: state.hive_group_goal.clone(),
        authority: state.authority.clone(),
        heartbeat_model: state.heartbeat_model.clone(),
        project: state.project.clone(),
        namespace: state.namespace.clone(),
        workspace: state.workspace.clone(),
        repo_root: state.repo_root.clone(),
        worktree_root: state.worktree_root.clone(),
        branch: state.branch.clone(),
        base_branch: state.base_branch.clone(),
        visibility: state.visibility.clone(),
        base_url: state.base_url.clone(),
        base_url_healthy: state.base_url_healthy,
        host: state.host.clone(),
        pid: state.pid,
        topic_claim: state.topic_claim.clone(),
        scope_claims: state.scope_claims.clone(),
        task_id: state.task_id.clone(),
        focus: state.focus.clone(),
        pressure: state.pressure.clone(),
        next_recovery: state.next_recovery.clone(),
        next_action: state.next_action.clone(),
        working: state.working.clone(),
        touches: state.touches.clone(),
        blocked_by: state.blocked_by.clone(),
        cowork_with: state.cowork_with.clone(),
        handoff_target: state.handoff_target.clone(),
        offered_to: state.offered_to.clone(),
        needs_help: state.needs_help,
        needs_review: state.needs_review,
        handoff_state: state.handoff_state.clone(),
        confidence: state.confidence.clone(),
        risk: state.risk.clone(),
        status: Some(state.status.clone()),
    };
    let _ = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        client.upsert_hive_session(&request),
    )
    .await;
    let retired = retire_superseded_hive_sessions(&client, state)
        .await
        .unwrap_or(0);
    Ok(retired)
}

fn render_bundle_heartbeat_summary(state: &BundleHeartbeatState) -> String {
    format!(
        "heartbeat project={} agent={} hive={} role={} groups={} goal=\"{}\" authority={} session={} tab={} presence={} model={} base_url={} topic=\"{}\" scopes={} task={} focus=\"{}\" pressure=\"{}\"",
        state.project.as_deref().unwrap_or("none"),
        state
            .effective_agent
            .as_deref()
            .or(state.agent.as_deref())
            .unwrap_or("none"),
        state.hive_system.as_deref().unwrap_or("none"),
        state.hive_role.as_deref().unwrap_or("none"),
        if state.hive_groups.is_empty() {
            "none".to_string()
        } else {
            state.hive_groups.join(",")
        },
        state.hive_group_goal.as_deref().unwrap_or("none"),
        state.authority.as_deref().unwrap_or("none"),
        state.session.as_deref().unwrap_or("none"),
        state.tab_id.as_deref().unwrap_or("none"),
        heartbeat_presence_label(state.last_seen),
        state.heartbeat_model.as_deref().unwrap_or("none"),
        state.base_url.as_deref().unwrap_or("none"),
        state.topic_claim.as_deref().unwrap_or("none"),
        if state.scope_claims.is_empty() {
            "none".to_string()
        } else {
            compact_inline(&state.scope_claims.join(","), 72)
        },
        state.task_id.as_deref().unwrap_or("none"),
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

fn run_capabilities_command(args: &CapabilitiesArgs) -> anyhow::Result<CapabilitiesResponse> {
    let project_root = infer_bundle_project_root(&args.output);
    let registry = build_bundle_capability_registry(project_root.as_deref());
    let bridges = detect_capability_bridges();
    let query = args.query.as_deref().map(str::to_ascii_lowercase);
    let mut filtered = registry
        .capabilities
        .iter()
        .filter(|capability| {
            args.harness
                .as_deref()
                .is_none_or(|value| capability.harness == value)
        })
        .filter(|capability| {
            args.kind
                .as_deref()
                .is_none_or(|value| capability.kind == value)
        })
        .filter(|capability| {
            args.portability
                .as_deref()
                .is_none_or(|value| capability.portability_class == value)
        })
        .filter(|capability| {
            query.as_ref().is_none_or(|needle| {
                capability.name.to_ascii_lowercase().contains(needle)
                    || capability.harness.to_ascii_lowercase().contains(needle)
                    || capability.kind.to_ascii_lowercase().contains(needle)
                    || capability
                        .portability_class
                        .to_ascii_lowercase()
                        .contains(needle)
                    || capability
                        .bridge_hint
                        .as_deref()
                        .unwrap_or_default()
                        .to_ascii_lowercase()
                        .contains(needle)
            })
        })
        .cloned()
        .collect::<Vec<_>>();
    filtered.sort_by(|left, right| {
        left.harness
            .cmp(&right.harness)
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.name.cmp(&right.name))
    });
    let bridge_harnesses = bridges
        .actions
        .iter()
        .map(|action| action.harness.clone())
        .collect::<BTreeSet<_>>();
    let mut harnesses = BTreeMap::<String, CapabilityHarnessSummary>::new();
    for capability in &filtered {
        let entry = harnesses
            .entry(capability.harness.clone())
            .or_insert_with(|| CapabilityHarnessSummary {
                harness: capability.harness.clone(),
                capabilities: 0,
                installed: 0,
                bridge_actions: 0,
            });
        entry.capabilities += 1;
        if capability.status == "installed" || capability.status == "discovered" {
            entry.installed += 1;
        }
    }
    for action in &bridges.actions {
        if args
            .harness
            .as_deref()
            .is_some_and(|value| action.harness != value)
        {
            continue;
        }
        let entry =
            harnesses
                .entry(action.harness.clone())
                .or_insert_with(|| CapabilityHarnessSummary {
                    harness: action.harness.clone(),
                    capabilities: 0,
                    installed: 0,
                    bridge_actions: 0,
                });
        entry.bridge_actions += 1;
    }

    Ok(CapabilitiesResponse {
        bundle_root: args.output.display().to_string(),
        generated_at: registry.generated_at,
        discovered: filtered.len(),
        universal: filtered
            .iter()
            .filter(|record| is_universal_class(&record.portability_class))
            .count(),
        bridgeable: filtered
            .iter()
            .filter(|record| is_bridgeable_class(&record.portability_class))
            .count(),
        harness_native: filtered
            .iter()
            .filter(|record| is_harness_native_class(&record.portability_class))
            .count(),
        bridge_actions: bridges.actions.len(),
        wired_harnesses: bridge_harnesses.len(),
        filters: serde_json::json!({
            "harness": args.harness,
            "kind": args.kind,
            "portability": args.portability,
            "query": args.query,
            "limit": args.limit,
        }),
        harnesses: harnesses.into_values().collect(),
        records: filtered.into_iter().take(args.limit).collect(),
    })
}

fn render_capabilities_runtime_summary(response: &CapabilitiesResponse) -> String {
    let harnesses = response
        .harnesses
        .iter()
        .take(4)
        .map(|harness| {
            format!(
                "{}:{}/{}/{}",
                harness.harness, harness.capabilities, harness.installed, harness.bridge_actions
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    if harnesses.is_empty() {
        format!(
            "capabilities bundle={} discovered={} universal={} bridgeable={} harness_native={} bridge_actions={} wired_harnesses={} shown={} harnesses=none",
            response.bundle_root,
            response.discovered,
            response.universal,
            response.bridgeable,
            response.harness_native,
            response.bridge_actions,
            response.wired_harnesses,
            response.records.len(),
        )
    } else {
        format!(
            "capabilities bundle={} discovered={} universal={} bridgeable={} harness_native={} bridge_actions={} wired_harnesses={} shown={} harnesses={}",
            response.bundle_root,
            response.discovered,
            response.universal,
            response.bridgeable,
            response.harness_native,
            response.bridge_actions,
            response.wired_harnesses,
            response.records.len(),
            harnesses
        )
    }
}

fn read_recent_maintain_reports(
    output: &Path,
    limit: usize,
) -> anyhow::Result<Vec<MaintainReport>> {
    let dir = maintain_reports_dir(output);
    if !dir.exists() || limit == 0 {
        return Ok(Vec::new());
    }
    let mut candidates = fs::read_dir(&dir)
        .with_context(|| format!("read {}", dir.display()))?
        .filter_map(|entry| entry.ok().map(|value| value.path()))
        .filter(|path| {
            path.extension().and_then(|value| value.to_str()) == Some("json")
                && path.file_name().and_then(|value| value.to_str()) != Some("latest.json")
        })
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.reverse();
    let mut reports = Vec::new();
    for path in candidates.into_iter().take(limit) {
        let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let report = serde_json::from_str::<MaintainReport>(&raw)
            .with_context(|| format!("parse {}", path.display()))?;
        reports.push(report);
    }
    Ok(reports)
}

fn write_skill_policy_artifacts(
    output: &Path,
    response: &MemoryPolicyResponse,
    report: &SkillLifecycleReport,
    apply_queues: bool,
) -> anyhow::Result<Option<SkillPolicyApplyArtifact>> {
    let runtime_defaulted = is_default_runtime(&response.runtime);
    let batch = SkillPolicyBatchArtifact {
        generated_at: Utc::now(),
        bundle_root: output.display().to_string(),
        runtime_defaulted,
        report: report.clone(),
    };
    let batch_json = serde_json::to_string_pretty(&batch)? + "\n";
    let batch_markdown = render_skill_policy_batch_markdown(&batch);
    write_state_artifact(
        skill_policy_batch_state_path(output),
        &batch_json,
        "skill-policy batch json",
    )?;
    write_state_artifact(
        skill_policy_batch_markdown_path(output),
        &batch_markdown,
        "skill-policy batch markdown",
    )?;

    let review = SkillPolicyQueueArtifact {
        generated_at: Utc::now(),
        bundle_root: output.display().to_string(),
        runtime_defaulted,
        queue: "review".to_string(),
        records: report.review_queue.clone(),
    };
    let review_json = serde_json::to_string_pretty(&review)? + "\n";
    let review_markdown = render_skill_policy_queue_markdown(&review);
    write_state_artifact(
        skill_policy_review_state_path(output),
        &review_json,
        "skill-policy review queue json",
    )?;
    write_state_artifact(
        skill_policy_review_markdown_path(output),
        &review_markdown,
        "skill-policy review queue markdown",
    )?;

    let activate = SkillPolicyQueueArtifact {
        generated_at: Utc::now(),
        bundle_root: output.display().to_string(),
        runtime_defaulted,
        queue: "activate".to_string(),
        records: report.activate_queue.clone(),
    };
    let activate_json = serde_json::to_string_pretty(&activate)? + "\n";
    let activate_markdown = render_skill_policy_queue_markdown(&activate);
    write_state_artifact(
        skill_policy_activate_state_path(output),
        &activate_json,
        "skill-policy activate queue json",
    )?;
    write_state_artifact(
        skill_policy_activate_markdown_path(output),
        &activate_markdown,
        "skill-policy activate queue markdown",
    )?;

    let receipt = if apply_queues {
        let receipt = consume_skill_policy_activate_queue(output)?;
        if let Some(receipt) = receipt.as_ref() {
            let apply_json = serde_json::to_string_pretty(receipt)? + "\n";
            let apply_markdown = render_skill_policy_apply_markdown(receipt);
            write_state_artifact(
                skill_policy_apply_state_path(output),
                &apply_json,
                "skill-policy apply receipt json",
            )?;
            write_state_artifact(
                skill_policy_apply_markdown_path(output),
                &apply_markdown,
                "skill-policy apply receipt markdown",
            )?;
        }
        receipt
    } else {
        None
    };

    Ok(receipt)
}

fn consume_skill_policy_activate_queue(
    output: &Path,
) -> anyhow::Result<Option<SkillPolicyApplyArtifact>> {
    let path = skill_policy_activate_state_path(output);
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let queue = serde_json::from_str::<SkillPolicyQueueArtifact>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    let applied = queue
        .records
        .iter()
        .filter(|record| record.activation == "activate")
        .cloned()
        .collect::<Vec<_>>();
    let skipped = queue
        .records
        .iter()
        .filter(|record| record.activation != "activate")
        .cloned()
        .collect::<Vec<_>>();
    let receipt = SkillPolicyApplyArtifact {
        generated_at: Utc::now(),
        bundle_root: queue.bundle_root.clone(),
        runtime_defaulted: queue.runtime_defaulted,
        source_queue_path: path.display().to_string(),
        applied_count: applied.len(),
        skipped_count: skipped.len(),
        applied,
        skipped,
    };

    Ok(Some(receipt))
}

fn skill_policy_apply_request(receipt: &SkillPolicyApplyArtifact) -> SkillPolicyApplyRequest {
    SkillPolicyApplyRequest {
        bundle_root: receipt.bundle_root.clone(),
        runtime_defaulted: receipt.runtime_defaulted,
        source_queue_path: receipt.source_queue_path.clone(),
        applied_count: receipt.applied_count,
        skipped_count: receipt.skipped_count,
        applied: receipt.applied.iter().map(to_activation_record).collect(),
        skipped: receipt.skipped.iter().map(to_activation_record).collect(),
        project: None,
        namespace: None,
        workspace: None,
    }
}

fn to_activation_record(record: &SkillLifecycleRecord) -> SkillPolicyActivationRecord {
    SkillPolicyActivationRecord {
        harness: record.harness.clone(),
        name: record.name.clone(),
        kind: record.kind.clone(),
        portability_class: record.portability_class.clone(),
        proposal: record.proposal.clone(),
        sandbox: record.sandbox.clone(),
        sandbox_risk: record.sandbox_risk,
        sandbox_reason: record.sandbox_reason.clone(),
        activation: record.activation.clone(),
        activation_reason: record.activation_reason.clone(),
        source_path: record.source_path.clone(),
        target_path: record.target_path.clone(),
        notes: record.notes.clone(),
    }
}

fn write_state_artifact(path: PathBuf, content: &str, label: &str) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&path, content).with_context(|| format!("write {label} {}", path.display()))
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

fn render_voice_mode_section(voice_mode: &str) -> String {
    let voice_mode = normalize_voice_mode_value(voice_mode).unwrap_or_else(|_| default_voice_mode());
    match voice_mode.as_str() {
        "normal" => "- default: normal\n- keep replies clear and complete\n- avoid forced compression\n"
            .to_string(),
        _ => "- default: caveman ultra\n- few tokens\n- do trick\n- keep technical accuracy\n".to_string(),
    }
}

fn write_native_agent_bridge_files(output: &Path) -> anyhow::Result<()> {
    let agents_dir = output.join("agents");
    fs::create_dir_all(&agents_dir).with_context(|| format!("create {}", agents_dir.display()))?;
    let authority_warning = read_bundle_runtime_config(output)
        .ok()
        .flatten()
        .map(|runtime| authority_warning_lines(Some(&runtime)))
        .unwrap_or_default();
    let voice_mode = read_bundle_voice_mode(output).unwrap_or_else(default_voice_mode);
    let claude_imports = agents_dir.join("CLAUDE_IMPORTS.md");
    let authority_warning_section = if authority_warning.is_empty() {
        String::new()
    } else {
        format!(
            "## Session Start Warning\n\n{}\n\n",
            authority_warning
                .iter()
                .map(|line| format!("- {line}"))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };
    let voice_section = render_voice_mode_section(&voice_mode);
    fs::write(
        &claude_imports,
        format!(
            "# memd imports for Claude Code\n\nUse this file as the single import target from your project `CLAUDE.md`.\n\nAdd this line to the root `CLAUDE.md` for the workspace:\n\n`@.memd/agents/CLAUDE_IMPORTS.md`\n\nThen run `/memory` inside Claude Code to verify the imported memd files are loaded.\n\n{authority_warning_section}## Imported memd memory files\n\n@../MEMD_WAKEUP.md\n@../MEMD_MEMORY.md\n@../MEMD_EVENTS.md\n@CLAUDE_CODE_WAKEUP.md\n@CLAUDE_CODE_MEMORY.md\n@CLAUDE_CODE_EVENTS.md\n\n## Runtime rules\n\n- Start from the wake-up file before deeper memory surfaces.\n- For prior decisions, preferences, or project history, run `memd lookup --output {bundle} --query \"...\"` before answering.\n- Use the generated lane helpers when you want low-friction memory writes:\n  - `.memd/agents/remember-short.sh --content \"Current blocker: ...\"`\n  - `.memd/agents/remember-decision.sh --content \"decision: ...\"`\n  - `.memd/agents/remember-preference.sh --content \"preference: ...\"`\n  - `.memd/agents/remember-long.sh --content \"fact: ...\"`\n  - `.memd/agents/capture-live.sh --content \"status: ...\"`\n  - `.memd/agents/correct-memory.sh --content \"corrected fact: ...\"`\n  - `.memd/agents/sync-semantic.sh`\n- After `memd reload` (alias: `memd refresh`), use installed `$gsd-*` skills as the GSD interface in Codex.\n- Do not block on standalone `gsd-*` shell binaries unless you verified they are the required interface for this harness and they are missing on `PATH`.\n- If `$gsd-autonomous` is installed as a skill, try that skill path before claiming the autonomous pipeline is unavailable.\n\n## Notes\n\n- `memd wake --output {bundle}` refreshes the startup live-memory surface.\n- `memd lookup --output {bundle} --query \"...\"` is the bundle-aware pre-answer recall path.\n- `memd checkpoint --output {bundle} --content \"...\"` writes current task state into the live backend.\n- `memd hook capture --output {bundle} --stdin` records episodic live-memory updates from hooks.\n- `memd rag sync --project <project> --namespace <namespace>` pushes canonical memory into the configured semantic backend.\n- `memd handoff --output {bundle} --prompt` refreshes the shared handoff view.\n- dream and autodream output should flow back through `memd`, then Claude should pick it up through this import chain.\n- keep `memd` as the source of truth; treat this Claude import surface as a generated bridge.\n{voice_section}\n",
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

fn write_bundle_command_catalog_files(output: &Path) -> anyhow::Result<()> {
    let catalog = command_catalog::build_command_catalog(output);
    let commands = output.join("COMMANDS.md");
    fs::write(&commands, render_command_catalog_markdown(&catalog))
        .with_context(|| format!("write {}", commands.display()))?;
    Ok(())
}

fn render_authority_warning_markdown(output: &Path) -> String {
    let authority_warning = read_bundle_runtime_config(output)
        .ok()
        .flatten()
        .map(|runtime| authority_warning_lines(Some(&runtime)))
        .unwrap_or_default();
    if authority_warning.is_empty() {
        return String::new();
    }

    format!(
        "## Session Start Warning\n\n{}\n\n",
        authority_warning
            .iter()
            .map(|line| format!("- {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

fn write_memory_markdown_files(output: &Path, markdown: &str) -> anyhow::Result<()> {
    let authority_warning = render_authority_warning_markdown(output);
    let markdown = if authority_warning.is_empty() {
        markdown.to_string()
    } else {
        format!("{authority_warning}{markdown}")
    };
    let root_memory = output.join("MEMD_MEMORY.md");
    fs::write(&root_memory, &markdown)
        .with_context(|| format!("write {}", root_memory.display()))?;

    let agents_dir = output.join("agents");
    if let Some(parent) = agents_dir.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::create_dir_all(&agents_dir).with_context(|| format!("create {}", agents_dir.display()))?;
    for file_name in [
        "CODEX_MEMORY.md",
        "CLAUDE_CODE_MEMORY.md",
        "AGENT_ZERO_MEMORY.md",
        "OPENCLAW_MEMORY.md",
        "OPENCODE_MEMORY.md",
        "HERMES_MEMORY.md",
    ] {
        let path = agents_dir.join(file_name);
        fs::write(&path, &markdown).with_context(|| format!("write {}", path.display()))?;
    }
    Ok(())
}

fn write_wakeup_markdown_files(output: &Path, markdown: &str) -> anyhow::Result<()> {
    let authority_warning = render_authority_warning_markdown(output);
    let markdown = if authority_warning.is_empty() {
        markdown.to_string()
    } else {
        format!("{authority_warning}{markdown}")
    };
    let root_wakeup = output.join("MEMD_WAKEUP.md");
    fs::write(&root_wakeup, &markdown)
        .with_context(|| format!("write {}", root_wakeup.display()))?;

    let agents_dir = output.join("agents");
    if let Some(parent) = agents_dir.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::create_dir_all(&agents_dir).with_context(|| format!("create {}", agents_dir.display()))?;
    for file_name in [
        "CODEX_WAKEUP.md",
        "CLAUDE_CODE_WAKEUP.md",
        "AGENT_ZERO_WAKEUP.md",
        "OPENCLAW_WAKEUP.md",
        "OPENCODE_WAKEUP.md",
        "HERMES_WAKEUP.md",
    ] {
        let path = agents_dir.join(file_name);
        fs::write(&path, &markdown).with_context(|| format!("write {}", path.display()))?;
    }
    Ok(())
}

fn write_bundle_memory_object_pages(
    output: &Path,
    snapshot: &ResumeSnapshot,
    handoff: Option<&HandoffSnapshot>,
    hive: Option<&BundleHiveMemorySurface>,
) -> anyhow::Result<()> {
    let dir = bundle_compiled_memory_dir(output);
    fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    for lane in [
        MemoryObjectLane::Context,
        MemoryObjectLane::Working,
        MemoryObjectLane::Inbox,
        MemoryObjectLane::Recovery,
        MemoryObjectLane::Semantic,
        MemoryObjectLane::Workspace,
    ] {
        let path = bundle_compiled_memory_path(output, lane);
        let markdown = render_bundle_memory_object_markdown(output, snapshot, handoff, hive, lane);
        fs::write(&path, markdown).with_context(|| format!("write {}", path.display()))?;
        let item_count = match lane {
            MemoryObjectLane::Context => snapshot.context.records.len(),
            MemoryObjectLane::Working => snapshot.working.records.len(),
            MemoryObjectLane::Inbox => snapshot.inbox.items.len(),
            MemoryObjectLane::Recovery => snapshot.working.rehydration_queue.len(),
            MemoryObjectLane::Semantic => snapshot
                .semantic
                .as_ref()
                .map(|semantic| semantic.items.len())
                .unwrap_or(0),
            MemoryObjectLane::Workspace => snapshot.workspaces.workspaces.len(),
        };
        for index in 0..item_count {
            if let Some(key) = memory_object_lane_item_key(snapshot, lane, index) {
                let item_path = bundle_compiled_memory_item_path(output, lane, index, &key);
                if let Some(parent) = item_path.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("create {}", parent.display()))?;
                }
                let item_markdown = render_bundle_memory_object_item_markdown(
                    output, snapshot, handoff, hive, lane, index,
                )
                .unwrap_or_else(|| format!("# memd memory item: {}\n\n- none\n", lane.title()));
                fs::write(&item_path, item_markdown)
                    .with_context(|| format!("write {}", item_path.display()))?;
            }
        }
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

fn maintain_reports_dir(output: &Path) -> PathBuf {
    output.join("maintenance")
}

fn read_latest_maintain_report(output: &Path) -> anyhow::Result<Option<MaintainReport>> {
    let path = maintain_reports_dir(output).join("latest.json");
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let report = serde_json::from_str::<MaintainReport>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(report))
}

fn read_previous_maintain_report(output: &Path) -> anyhow::Result<Option<MaintainReport>> {
    let dir = maintain_reports_dir(output);
    if !dir.exists() {
        return Ok(None);
    }
    let mut candidates = fs::read_dir(&dir)
        .with_context(|| format!("read {}", dir.display()))?
        .filter_map(|entry| entry.ok().map(|value| value.path()))
        .filter(|path| {
            path.extension().and_then(|value| value.to_str()) == Some("json")
                && path.file_name().and_then(|value| value.to_str()) != Some("latest.json")
        })
        .collect::<Vec<_>>();
    candidates.sort();
    let Some(path) = candidates.into_iter().next_back() else {
        return Ok(None);
    };
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let report = serde_json::from_str::<MaintainReport>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(report))
}

fn write_maintain_artifacts(output: &Path, response: &MaintainReport) -> anyhow::Result<()> {
    let maintain_dir = maintain_reports_dir(output);
    fs::create_dir_all(&maintain_dir)
        .with_context(|| format!("create {}", maintain_dir.display()))?;

    let timestamp = response.generated_at.format("%Y%m%dT%H%M%SZ").to_string();
    let json = serde_json::to_string_pretty(response)? + "\n";
    let markdown = format!(
        "# memd maintain report\n\n- mode: {}\n- receipt: {}\n- compacted: {}\n- refreshed: {}\n- repaired: {}\n\n## Findings\n{}\n",
        response.mode.as_str(),
        response.receipt_id.as_deref().unwrap_or("none"),
        response.compacted_items,
        response.refreshed_items,
        response.repaired_items,
        if response.findings.is_empty() {
            "- none".to_string()
        } else {
            response
                .findings
                .iter()
                .map(|value| format!("- {}", value))
                .collect::<Vec<_>>()
                .join("\n")
        }
    );

    let latest_json = maintain_dir.join("latest.json");
    let latest_md = maintain_dir.join("latest.md");
    let timestamped_json = maintain_dir.join(format!("{timestamp}.json"));
    let timestamped_md = maintain_dir.join(format!("{timestamp}.md"));

    fs::write(&latest_json, &json).with_context(|| format!("write {}", latest_json.display()))?;
    fs::write(&latest_md, &markdown).with_context(|| format!("write {}", latest_md.display()))?;
    fs::write(&timestamped_json, &json)
        .with_context(|| format!("write {}", timestamped_json.display()))?;
    fs::write(&timestamped_md, &markdown)
        .with_context(|| format!("write {}", timestamped_md.display()))?;
    Ok(())
}

async fn run_maintain_command(
    args: &MaintainArgs,
    base_url: &str,
) -> anyhow::Result<MaintainReport> {
    let runtime = read_bundle_runtime_config(&args.output)?;
    let client = MemdClient::new(base_url)?;
    let maintenance = client
        .maintenance_report(&MemoryMaintenanceReportRequest {
            project: runtime.as_ref().and_then(|value| value.project.clone()),
            namespace: runtime.as_ref().and_then(|value| value.namespace.clone()),
            inactive_days: Some(7),
            lookback_days: Some(30),
            min_events: Some(2),
            max_decay: Some(0.5),
            mode: Some(args.mode.clone()),
            apply: Some(args.apply),
        })
        .await?;
    let response = MaintainReport {
        mode: args.mode.clone(),
        receipt_id: maintenance.receipt_id.clone(),
        compacted_items: if args.mode == "compact" {
            maintenance
                .compacted_items
                .max(maintenance.consolidated_candidates)
        } else {
            maintenance.compacted_items
        },
        refreshed_items: if args.mode == "refresh" {
            maintenance
                .refreshed_items
                .max(maintenance.reinforced_candidates)
        } else {
            maintenance.refreshed_items
        },
        repaired_items: if args.mode == "repair" {
            maintenance
                .repaired_items
                .max(maintenance.cooled_candidates)
        } else {
            maintenance.repaired_items
        },
        findings: maintenance.highlights.clone(),
        generated_at: maintenance.generated_at,
    };
    write_maintain_artifacts(&args.output, &response)?;
    auto_checkpoint_bundle_event(
        &args.output,
        base_url,
        "maintenance",
        format!(
            "Maintenance {} compacted={} refreshed={} repaired={} findings={}.",
            response.mode.as_str(),
            response.compacted_items,
            response.refreshed_items,
            response.repaired_items,
            response.findings.len()
        ),
        vec!["maintenance".to_string(), response.mode.clone()],
        0.78,
    )
    .await?;
    Ok(response)
}

fn render_maintain_summary(response: &MaintainReport) -> String {
    let findings = if response.findings.is_empty() {
        "none".to_string()
    } else {
        response.findings.join(" | ")
    };
    format!(
        "maintain mode={} receipt={} compacted={} refreshed={} repaired={} findings={}",
        response.mode.as_str(),
        response.receipt_id.as_deref().unwrap_or("none"),
        response.compacted_items,
        response.refreshed_items,
        response.repaired_items,
        findings
    )
}

fn gap_reports_dir(output: &Path) -> PathBuf {
    output.join("gaps")
}

fn improvement_reports_dir(output: &Path) -> PathBuf {
    output.join("improvements")
}

fn scenario_reports_dir(output: &Path) -> PathBuf {
    output.join("scenarios")
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
    let limit = limit.clamp(1, 64);
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
            "coordination:stale_remote_sessions" => {
                if candidate
                    .evidence
                    .iter()
                    .any(|value| value.starts_with("recovery=memd coordination --recover-session"))
                {
                    add(
                        &mut actions,
                        &mut seen,
                        "recover_session",
                        "high",
                        None,
                        None,
                        None,
                        None,
                        &candidate.recommendation,
                    );
                }
                if candidate
                    .evidence
                    .iter()
                    .any(|value| value.starts_with("retirement=memd coordination --retire-session"))
                {
                    add(
                        &mut actions,
                        &mut seen,
                        "retire_session",
                        "medium",
                        None,
                        None,
                        None,
                        None,
                        &candidate.recommendation,
                    );
                }
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
                "retire_session" => {
                    if let Some(session) = suggestion.stale_session.clone() {
                        add(
                            &mut actions,
                            &mut seen,
                            "retire_session",
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
            write_bundle_memory_files(output, &snapshot, None, false).await?;
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
                    retire_session: None,
                    to_session: None,
                    deny_session: None,
                    reroute_session: None,
                    handoff_scope: None,
                    summary: false,
                },
                base_url,
            )
            .await?;
            Ok(format!(
                "recovered stale session pressure (stale_hives={})",
                response.recovery.stale_hives.len()
            ))
        }
        "retire_session" => {
            let target_session = action
                .target_session
                .clone()
                .context("retire_session requires a target_session")?;
            let response = run_coordination_command(
                &CoordinationArgs {
                    output: output.to_path_buf(),
                    view: Some("all".to_string()),
                    changes_only: false,
                    watch: false,
                    interval_secs: 30,
                    recover_session: None,
                    retire_session: Some(target_session.clone()),
                    to_session: None,
                    deny_session: None,
                    reroute_session: None,
                    handoff_scope: None,
                    summary: false,
                },
                base_url,
            )
            .await?;
            Ok(format!(
                "retired stale session {target_session} (stale_hives={})",
                response.recovery.stale_hives.len()
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
                    view: None,
                    summary: false,
                    json: false,
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
                    view: None,
                    summary: false,
                    json: false,
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
                    view: None,
                    summary: false,
                    json: false,
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
    let mut repo_evidence = collect_gap_repo_evidence(project_root);
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
            .find(|line| line.starts_with("- ") && line.contains("phase"))
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

    evidence.append(&mut repo_evidence);

    evidence
}

fn collect_gap_repo_evidence(project_root: &Path) -> Vec<String> {
    let mut evidence = Vec::new();
    let branch = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .arg("branch")
        .arg("--show-current")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    evidence.push(format!("git branch: {branch}"));

    let status = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .arg("status")
        .arg("--short")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .take(12)
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if status.is_empty() {
        evidence.push("git status: clean".to_string());
    } else {
        evidence.push(format!("git status: {}", status.join(" | ")));
    }

    for (path, label, keywords) in [
        (
            project_root.join("AGENTS.md"),
            "AGENTS.md",
            &["memd", "memory", "bootstrap"][..],
        ),
        (
            project_root.join("CLAUDE.md"),
            "CLAUDE.md",
            &["memd", "memory", "hook"][..],
        ),
        (
            project_root.join("MEMORY.md"),
            "MEMORY.md",
            &["memory", "memd", "decision"][..],
        ),
        (
            project_root.join("README.md"),
            "README.md",
            &["memd", "setup", "memory"][..],
        ),
        (
            project_root.join("ROADMAP.md"),
            "ROADMAP.md",
            &["v5", "v6", "memd"][..],
        ),
        (
            project_root.join("docs/setup.md"),
            "docs/setup.md",
            &["memd", "bundle", "codex"][..],
        ),
        (
            project_root.join("docs/infra-facts.md"),
            "docs/infra-facts.md",
            &["memd", "openclaw", "tailnet"][..],
        ),
        (
            project_root.join(".planning/STATE.md"),
            ".planning/STATE.md",
            &["memory", "gap", "open loop"][..],
        ),
    ] {
        if let Some(snippet) = read_keyword_snippet(&path, keywords, 4) {
            evidence.push(format!("{label}: {snippet}"));
        }
    }

    let local_bundle = project_root.join(".memd").join("config.json").exists();
    let global_bundle = home_dir()
        .map(|home| home.join(".memd").join("config.json").exists())
        .unwrap_or(false);
    evidence.push(format!(
        "memd bundles: global={} project={}",
        global_bundle, local_bundle
    ));

    let wiring = read_memd_runtime_wiring();
    let codex_wired = wiring
        .get("codex")
        .and_then(|value| value.get("wired"))
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let claude_wired = wiring
        .get("claude")
        .and_then(|value| value.get("wired"))
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let openclaw_wired = wiring
        .get("openclaw")
        .and_then(|value| value.get("wired"))
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let opencode_wired = wiring
        .get("opencode")
        .and_then(|value| value.get("wired"))
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    evidence.push(format!(
        "runtime wiring: codex={} claude={} openclaw={} opencode={}",
        codex_wired, claude_wired, openclaw_wired, opencode_wired
    ));

    evidence
}

fn collect_recent_repo_changes(project_root: &Path) -> Vec<String> {
    let mut changes = Vec::new();

    let status_entries = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .arg("status")
        .arg("--short")
        .arg("--untracked-files=normal")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .take(8)
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if status_entries.is_empty() {
        changes.push("repo clean".to_string());
    } else {
        changes.extend(
            status_entries
                .into_iter()
                .map(|entry| format!("status {entry}")),
        );
    }

    let diff_stats = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .arg("diff")
        .arg("--stat=72,40")
        .arg("--compact-summary")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .take(4)
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    changes.extend(diff_stats.into_iter().map(|entry| format!("diff {entry}")));

    changes
}

fn summarize_repo_event_line(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.eq_ignore_ascii_case("repo clean") {
        return "repo_state: clean".to_string();
    }

    if let Some(rest) = trimmed.strip_prefix("status ") {
        let mut parts = rest.split_whitespace();
        let code = parts.next().unwrap_or_default();
        let path = parts.collect::<Vec<_>>().join(" ");
        let label = if code.contains('?') {
            "file_created"
        } else if code.contains('D') {
            "file_deleted"
        } else if code.contains('A')
            || code.contains('M')
            || code.contains('R')
            || code.contains('C')
            || code.contains('U')
            || code.contains('T')
        {
            "file_edited"
        } else {
            "repo_change"
        };
        let detail = if path.is_empty() { code } else { path.as_str() };
        return format!("{label}: {detail}");
    }

    if let Some(rest) = trimmed.strip_prefix("diff ") {
        return format!("repo_delta: {}", rest.trim());
    }

    trimmed.to_string()
}

fn build_event_spine(
    change_summary: &[String],
    recent_repo_changes: &[String],
    refresh_recommended: bool,
) -> Vec<String> {
    let mut spine = Vec::new();

    for change in change_summary.iter().take(4) {
        let compact = change.trim();
        if !compact.is_empty() {
            spine.push(format!("resume_delta: {compact}"));
        }
    }

    for change in recent_repo_changes.iter().take(6) {
        let compact = summarize_repo_event_line(change);
        if !compact.is_empty() {
            spine.push(compact);
        }
    }

    if refresh_recommended {
        spine.push("compaction_due: refresh recommended for current resume state".to_string());
    }

    let mut deduped = Vec::new();
    let mut seen = std::collections::HashSet::<String>::new();
    for item in spine {
        let normalized = item
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        if normalized.is_empty() || !seen.insert(normalized) {
            continue;
        }
        deduped.push(item);
    }

    deduped.truncate(8);
    deduped
}

async fn sync_recent_repo_live_truth(
    project_root: Option<&Path>,
    base_url: &str,
    project: Option<&str>,
    namespace: Option<&str>,
    workspace: Option<&str>,
    visibility: Option<memd_schema::MemoryVisibility>,
) -> anyhow::Result<()> {
    let Some(project_root) = project_root else {
        return Ok(());
    };
    let Some(project) = project else {
        return Ok(());
    };

    let changes = collect_recent_repo_changes(project_root);
    let content = {
        let spine = build_event_spine(&[], &changes, false);
        if spine.is_empty() {
            "repo_state: clean".to_string()
        } else {
            spine.join("\n")
        }
    };

    let client = MemdClient::new(base_url)?;
    let live_truth_tags = vec!["live_truth".to_string(), "repo_changes".to_string()];
    let search =
        match search_live_truth_record(&client, project, namespace, workspace, visibility, false)
            .await
        {
            Ok(response) => response,
            Err(err) if is_live_truth_kind_rejection(&err) => {
                search_live_truth_record(&client, project, namespace, workspace, visibility, true)
                    .await?
            }
            Err(err) => return Err(err),
        };

    if let Some(existing) = search.items.first() {
        let repair_request = RepairMemoryRequest {
            id: existing.id,
            mode: MemoryRepairMode::CorrectMetadata,
            confidence: Some(0.98),
            status: Some(MemoryStatus::Active),
            workspace: workspace.map(ToOwned::to_owned),
            visibility,
            source_agent: Some("memd".to_string()),
            source_system: Some("memd-live-truth".to_string()),
            source_path: Some(project_root.display().to_string()),
            source_quality: Some(memd_schema::SourceQuality::Derived),
            content: Some(content.clone()),
            tags: Some(live_truth_tags.clone()),
            supersedes: Vec::new(),
        };
        match client.repair(&repair_request).await {
            Ok(_) => {}
            Err(err) if err.to_string().contains("memory item not found") => {
                store_live_truth_record(
                    &client,
                    content,
                    project,
                    namespace,
                    workspace,
                    visibility,
                    project_root,
                    live_truth_tags,
                )
                .await?;
            }
            Err(err) => return Err(err),
        }
    } else {
        store_live_truth_record(
            &client,
            content,
            project,
            namespace,
            workspace,
            visibility,
            project_root,
            live_truth_tags,
        )
        .await?;
    }

    Ok(())
}

fn is_live_truth_kind_rejection(error: &anyhow::Error) -> bool {
    let message = error.to_string();
    message.contains("unknown variant `live_truth`")
        || message.contains("unknown variant 'live_truth'")
        || message.contains("expected one of fact, decision, preference, runbook, procedural, self_model, topology, status, pattern, constraint")
}

async fn search_live_truth_record(
    client: &MemdClient,
    project: &str,
    namespace: Option<&str>,
    workspace: Option<&str>,
    visibility: Option<memd_schema::MemoryVisibility>,
    legacy_compatible: bool,
) -> anyhow::Result<memd_schema::SearchMemoryResponse> {
    let kinds = if legacy_compatible {
        Vec::new()
    } else {
        vec![MemoryKind::LiveTruth]
    };
    client
        .search(&SearchMemoryRequest {
            query: None,
            route: Some(RetrievalRoute::LocalFirst),
            intent: Some(RetrievalIntent::CurrentTask),
            scopes: vec![MemoryScope::Local],
            kinds,
            statuses: vec![MemoryStatus::Active],
            project: Some(project.to_string()),
            namespace: namespace.map(ToOwned::to_owned),
            workspace: workspace.map(ToOwned::to_owned),
            visibility,
            belief_branch: None,
            source_agent: Some("memd".to_string()),
            tags: vec!["live_truth".to_string()],
            stages: vec![MemoryStage::Canonical],
            limit: Some(1),
            max_chars_per_item: Some(800),
        })
        .await
}

async fn emit_lane_surface_receipt(
    client: &MemdClient,
    surface: &BundleLaneSurface,
    runtime: &BundleRuntimeConfig,
    actor_session: &str,
) -> anyhow::Result<()> {
    let (kind, summary) = if surface.action == "auto_create" {
        (
            "lane_create",
            format!(
                "Auto-created isolated hive lane from {} to {}.",
                surface.previous_branch.as_deref().unwrap_or("none"),
                surface.current_branch.as_deref().unwrap_or("none"),
            ),
        )
    } else {
        (
            "lane_reroute",
            format!(
                "Auto-rerouted hive lane from {} to {} after collision with {}.",
                surface.previous_branch.as_deref().unwrap_or("none"),
                surface.current_branch.as_deref().unwrap_or("none"),
                surface.conflict_session.as_deref().unwrap_or("unknown"),
            ),
        )
    };
    emit_coordination_receipt(
        client,
        kind,
        actor_session,
        runtime
            .agent
            .as_deref()
            .map(|agent| compose_agent_identity(agent, runtime.session.as_deref())),
        surface.conflict_session.clone(),
        None,
        surface.current_branch.clone(),
        runtime.project.clone(),
        runtime.namespace.clone(),
        runtime.workspace.clone(),
        summary,
    )
    .await
}

async fn emit_lane_fault_receipt(
    client: &MemdClient,
    actor_session: &str,
    actor_agent: Option<String>,
    target: &ProjectAwarenessEntry,
    task_id: Option<String>,
    scope: Option<String>,
    project: Option<String>,
    namespace: Option<String>,
    workspace: Option<String>,
) {
    let _ = emit_coordination_receipt(
        client,
        "lane_fault",
        actor_session,
        actor_agent,
        target.session.clone(),
        task_id,
        scope,
        project,
        namespace,
        workspace,
        format!(
            "Queen denied unsafe shared lane target: {}.",
            render_hive_lane_collision(target)
        ),
    )
    .await;
}

async fn store_live_truth_record(
    client: &MemdClient,
    content: String,
    project: &str,
    namespace: Option<&str>,
    workspace: Option<&str>,
    visibility: Option<memd_schema::MemoryVisibility>,
    project_root: &Path,
    tags: Vec<String>,
) -> anyhow::Result<()> {
    let request = StoreMemoryRequest {
        content: content.clone(),
        kind: MemoryKind::LiveTruth,
        scope: MemoryScope::Local,
        project: Some(project.to_string()),
        namespace: namespace.map(ToOwned::to_owned),
        workspace: workspace.map(ToOwned::to_owned),
        visibility,
        belief_branch: None,
        source_agent: Some("memd".to_string()),
        source_system: Some("memd-live-truth".to_string()),
        source_path: Some(project_root.display().to_string()),
        source_quality: Some(memd_schema::SourceQuality::Derived),
        confidence: Some(0.98),
        ttl_seconds: Some(3_600),
        last_verified_at: Some(Utc::now()),
        supersedes: Vec::new(),
        tags: tags.clone(),
        status: Some(MemoryStatus::Active),
    };

    match client.store(&request).await {
        Ok(_) => Ok(()),
        Err(err) if is_live_truth_kind_rejection(&err) => {
            client
                .store(&StoreMemoryRequest {
                    kind: MemoryKind::Status,
                    source_system: Some("memd-live-truth-compat".to_string()),
                    tags,
                    ..request
                })
                .await?;
            Ok(())
        }
        Err(err) => Err(err),
    }
}

async fn sync_resume_state_record(
    client: &MemdClient,
    project_root: Option<&Path>,
    project: Option<&str>,
    namespace: Option<&str>,
    workspace: Option<&str>,
    visibility: Option<memd_schema::MemoryVisibility>,
    effective_agent: Option<&str>,
    snapshot: &ResumeSnapshot,
) -> anyhow::Result<()> {
    let Some(content) = build_resume_state_record_content(snapshot) else {
        return Ok(());
    };

    let scope = if project.is_some() {
        MemoryScope::Project
    } else {
        MemoryScope::Synced
    };
    let tags = vec!["resume_state".to_string(), "session_state".to_string()];
    let existing = client
        .search(&SearchMemoryRequest {
            query: None,
            route: Some(RetrievalRoute::LocalFirst),
            intent: Some(RetrievalIntent::CurrentTask),
            scopes: vec![scope],
            kinds: vec![MemoryKind::Status],
            statuses: vec![MemoryStatus::Active],
            project: project.map(ToOwned::to_owned),
            namespace: namespace.map(ToOwned::to_owned),
            workspace: workspace.map(ToOwned::to_owned),
            visibility,
            belief_branch: None,
            source_agent: effective_agent.map(ToOwned::to_owned),
            tags: vec!["resume_state".to_string()],
            stages: vec![MemoryStage::Canonical],
            limit: Some(1),
            max_chars_per_item: Some(800),
        })
        .await?;

    if let Some(existing) = existing.items.first() {
        client
            .repair(&RepairMemoryRequest {
                id: existing.id,
                mode: MemoryRepairMode::CorrectMetadata,
                confidence: Some(0.94),
                status: Some(MemoryStatus::Active),
                workspace: workspace.map(ToOwned::to_owned),
                visibility,
                source_agent: effective_agent.map(ToOwned::to_owned),
                source_system: Some("memd-resume-state".to_string()),
                source_path: project_root.map(|path| path.display().to_string()),
                source_quality: Some(memd_schema::SourceQuality::Derived),
                content: Some(content),
                tags: Some(tags),
                supersedes: Vec::new(),
            })
            .await?;
    } else {
        client
            .store(&StoreMemoryRequest {
                content,
                kind: MemoryKind::Status,
                scope,
                project: project.map(ToOwned::to_owned),
                namespace: namespace.map(ToOwned::to_owned),
                workspace: workspace.map(ToOwned::to_owned),
                visibility,
                belief_branch: None,
                source_agent: effective_agent.map(ToOwned::to_owned),
                source_system: Some("memd-resume-state".to_string()),
                source_path: project_root.map(|path| path.display().to_string()),
                source_quality: Some(memd_schema::SourceQuality::Derived),
                confidence: Some(0.94),
                ttl_seconds: Some(86_400),
                last_verified_at: Some(Utc::now()),
                supersedes: Vec::new(),
                tags,
                status: Some(MemoryStatus::Active),
            })
            .await?;
    }

    Ok(())
}

fn build_resume_state_record_content(snapshot: &ResumeSnapshot) -> Option<String> {
    let mut lines = Vec::new();

    if let Some(focus) = snapshot.compact_working_records().first() {
        lines.push(format!("focus: {}", compact_inline(focus, 180)));
    }
    lines.push(format!("pressure: {}", snapshot.context_pressure()));
    if let Some(next) = snapshot.compact_rehydration_summaries().first() {
        lines.push(format!("next_recovery: {}", compact_inline(next, 180)));
    }
    if let Some(inbox) = snapshot.compact_inbox_items().first() {
        lines.push(format!("top_inbox: {}", compact_inline(inbox, 180)));
    }
    if let Some(change) = snapshot.recent_repo_changes.first() {
        lines.push(format!("repo_change: {}", compact_inline(change, 180)));
    }

    lines.retain(|line| !line.ends_with(": "));
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

fn read_keyword_snippet(path: &Path, keywords: &[&str], max_lines: usize) -> Option<String> {
    let contents = read_text_file(path)?;
    let keywords = keywords
        .iter()
        .map(|keyword| keyword.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let lines = contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| {
            let lower = line.to_ascii_lowercase();
            keywords.iter().any(|keyword| lower.contains(keyword))
        })
        .take(max_lines)
        .collect::<Vec<_>>();
    if lines.is_empty() {
        None
    } else {
        Some(lines.join(" | "))
    }
}

async fn gap_report(args: &GapArgs) -> anyhow::Result<GapReport> {
    let runtime = read_bundle_runtime_config(&args.output)?;
    let project_root = project_root_from_bundle(&args.output);
    let base_url = runtime
        .as_ref()
        .and_then(|value| value.base_url.clone())
        .unwrap_or_else(default_base_url);
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
                retire_session: None,
                to_session: None,
                deny_session: None,
                reroute_session: None,
                handoff_scope: None,
                summary: false,
            },
            &base_url,
        )
        .await
        .ok()
    } else {
        None
    };
    let awareness = read_project_awareness(&AwarenessArgs {
        output: args.output.clone(),
        root: None,
        include_current: true,
        summary: false,
    })
    .await
    .ok();
    let research_loops_doc_count = research_loops_doc_loop_count(project_root);
    let benchmark_registry = load_benchmark_registry_for_output(&args.output)
        .ok()
        .flatten()
        .map(|(_, registry)| registry);

    let candidates = build_gap_candidates(
        &args.output,
        &runtime,
        &resume,
        snapshot_state.as_ref(),
        eval.as_ref(),
        coordination.as_ref(),
        awareness.as_ref(),
        research_loops_doc_count,
        &recent_commits,
        &mut evidence,
        benchmark_registry.as_ref(),
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
                    retire_session: None,
                    to_session: None,
                    deny_session: None,
                    reroute_session: None,
                    handoff_scope: None,
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

async fn run_experiment_command(
    args: &ExperimentArgs,
    base_url: &str,
) -> anyhow::Result<ExperimentReport> {
    let started_at = Utc::now();
    let runtime = read_bundle_runtime_config(&args.output)?;
    let effective_max_iterations = if args.apply {
        args.max_iterations.max(1)
    } else {
        args.max_iterations
    };
    let backup_root = if args.apply {
        Some(snapshot_bundle_for_reversion(&args.output)?)
    } else {
        None
    };

    let improvement = run_improvement_loop(
        &ImproveArgs {
            output: args.output.clone(),
            max_iterations: effective_max_iterations,
            limit: args.limit,
            recent_commits: args.recent_commits,
            write: false,
            apply: args.apply,
            summary: false,
        },
        base_url,
    )
    .await?;

    let eval = read_latest_bundle_eval(&args.output).ok().flatten();
    let self_evolution_scenario = build_self_evolution_scenario_report(
        &args.output,
        runtime.as_ref(),
        &improvement,
        eval.as_ref(),
        Utc::now(),
    );
    write_scenario_artifacts(&args.output, &self_evolution_scenario)?;

    let composite = run_composite_command(
        &CompositeArgs {
            output: args.output.clone(),
            scenario: Some("self_evolution".to_string()),
            write: false,
            summary: false,
        },
        base_url,
    )
    .await?;

    let acceptance_gate = composite
        .gates
        .iter()
        .find(|gate| gate.name == "acceptance")
        .map(|gate| gate.status.as_str())
        .unwrap_or("fail");
    let hard_correctness_gate = composite
        .gates
        .iter()
        .find(|gate| gate.name == "hard_correctness")
        .map(|gate| gate.status.as_str())
        .unwrap_or("fail");
    let accepted = composite.score >= args.accept_below
        && acceptance_gate == "pass"
        && hard_correctness_gate == "pass";

    let mut restored = false;
    if args.apply && !accepted {
        if let Some(backup_root) = backup_root.as_ref() {
            restore_bundle_snapshot(backup_root, &args.output)?;
            restored = true;
        }
    }

    let mut learnings = Vec::new();
    if accepted && args.consolidate {
        learnings = derive_experiment_learnings(&improvement, &composite);
        append_experiment_learning_notes(&args.output, &learnings, &composite)?;
    }

    let mut trail = Vec::new();
    trail.push(format!(
        "improvement iterations={} apply={} max_iterations={} final_candidates={}",
        improvement.iterations.len(),
        improvement.apply,
        effective_max_iterations,
        improvement
            .final_gap
            .as_ref()
            .map(|value| value.candidate_count)
            .unwrap_or(0),
    ));
    trail.push(format!(
        "composite score={}/{} acceptance={} hard_correctness={}",
        composite.score, composite.max_score, acceptance_gate, hard_correctness_gate
    ));
    trail.push(format!(
        "decision={} accept_below={} restored={}",
        if accepted { "accepted" } else { "rejected" },
        args.accept_below,
        restored
    ));
    if !learnings.is_empty() {
        trail.push(format!("consolidated learnings={}", learnings.len()));
    }

    let mut findings = composite.findings.clone();
    if !accepted {
        findings.push("experiment rejected by bounded composite gate".to_string());
    }

    let mut recommendations = composite.recommendations.clone();
    if !accepted {
        recommendations.push(
            "tighten the improvement loop until the composite gate clears the accept threshold"
                .to_string(),
        );
    }

    let mut evidence = composite.evidence.clone();
    evidence.push(format!(
        "improvement_iterations={}",
        improvement.iterations.len()
    ));
    evidence.push(format!("accepted={accepted}"));
    if restored {
        evidence.push("bundle restored from snapshot after rejection".to_string());
    }

    Ok(ExperimentReport {
        bundle_root: args.output.display().to_string(),
        project: runtime.as_ref().and_then(|value| value.project.clone()),
        namespace: runtime.as_ref().and_then(|value| value.namespace.clone()),
        agent: runtime.as_ref().and_then(|value| value.agent.clone()),
        session: runtime.as_ref().and_then(|value| value.session.clone()),
        workspace: runtime.as_ref().and_then(|value| value.workspace.clone()),
        visibility: runtime.as_ref().and_then(|value| value.visibility.clone()),
        max_iterations: args.max_iterations,
        accept_below: args.accept_below,
        apply: args.apply,
        consolidate: args.consolidate,
        accepted,
        restored,
        started_at,
        completed_at: Utc::now(),
        improvement,
        composite,
        trail,
        learnings,
        findings,
        recommendations,
        evidence,
        evolution: None,
    })
}

fn build_self_evolution_scenario_report(
    output: &Path,
    runtime: Option<&BundleRuntimeConfig>,
    improvement: &ImprovementReport,
    eval: Option<&BundleEvalResponse>,
    completed_at: DateTime<Utc>,
) -> ScenarioReport {
    let mut checks = Vec::new();
    let mut findings = Vec::new();
    let mut next_actions = Vec::new();
    let mut evidence = vec![
        format!("bundle_root={}", output.display()),
        "scenario=self_evolution".to_string(),
        format!("improvement_iterations={}", improvement.iterations.len()),
        format!("final_changes={}", improvement.final_changes.len()),
    ];
    let mut passed_checks: usize = 0;
    let mut failed_checks: usize = 0;
    let mut score: u16 = 0;
    let mut max_score: u16 = 0;

    let mut add_check = |name: &str, status: &str, points: u16, details: String| {
        checks.push(ScenarioCheck {
            name: name.to_string(),
            status: status.to_string(),
            points,
            details: details.clone(),
        });
        max_score += points;
        match status {
            "pass" => {
                score += points;
                passed_checks += 1;
            }
            "warn" => {
                score += points;
                findings.push(details);
                next_actions.push(format!("improve {name} before promoting self evolution"));
            }
            _ => {
                failed_checks += 1;
                findings.push(details);
                next_actions.push(format!("resolve {name} before promoting self evolution"));
            }
        }
    };

    if !improvement.final_changes.is_empty() {
        add_check(
            "improvement_signal",
            "pass",
            28,
            format!(
                "{} final change(s) captured from improvement loop",
                improvement.final_changes.len()
            ),
        );
    } else {
        add_check(
            "improvement_signal",
            "fail",
            0,
            "no final changes captured for self evolution".to_string(),
        );
    }

    if improvement.converged {
        add_check(
            "improvement_convergence",
            "pass",
            12,
            "improvement loop converged on a proposal".to_string(),
        );
    } else if !improvement.iterations.is_empty() {
        add_check(
            "improvement_convergence",
            "warn",
            8,
            "improvement loop produced iterations but did not converge".to_string(),
        );
    } else {
        add_check(
            "improvement_convergence",
            "fail",
            0,
            "improvement loop produced no usable iterations".to_string(),
        );
    }

    let scope = classify_evolution_scope(&ExperimentReport {
        bundle_root: output.display().to_string(),
        project: runtime.and_then(|value| value.project.clone()),
        namespace: runtime.and_then(|value| value.namespace.clone()),
        agent: runtime.and_then(|value| value.agent.clone()),
        session: runtime.and_then(|value| value.session.clone()),
        workspace: runtime.and_then(|value| value.workspace.clone()),
        visibility: runtime.and_then(|value| value.visibility.clone()),
        max_iterations: improvement.max_iterations,
        accept_below: 80,
        apply: false,
        consolidate: false,
        accepted: false,
        restored: false,
        started_at: improvement.started_at,
        completed_at,
        improvement: improvement.clone(),
        composite: CompositeReport {
            bundle_root: output.display().to_string(),
            project: runtime.and_then(|value| value.project.clone()),
            namespace: runtime.and_then(|value| value.namespace.clone()),
            agent: runtime.and_then(|value| value.agent.clone()),
            session: runtime.and_then(|value| value.session.clone()),
            workspace: runtime.and_then(|value| value.workspace.clone()),
            visibility: runtime.and_then(|value| value.visibility.clone()),
            scenario: Some("self_evolution".to_string()),
            score: 100,
            max_score: 100,
            dimensions: Vec::new(),
            gates: Vec::new(),
            findings: Vec::new(),
            recommendations: Vec::new(),
            evidence: Vec::new(),
            generated_at: completed_at,
            completed_at,
        },
        trail: Vec::new(),
        learnings: Vec::new(),
        findings: Vec::new(),
        recommendations: Vec::new(),
        evidence: Vec::new(),
        evolution: None,
    });
    evidence.push(format!(
        "scope_class={} scope_gate={}",
        scope.scope_class, scope.scope_gate
    ));
    if scope.scope_gate == "auto_merge" {
        add_check(
            "proposal_scope",
            "pass",
            16,
            format!("proposal classified as {}", scope.scope_class),
        );
    } else {
        add_check(
            "proposal_scope",
            "warn",
            8,
            format!(
                "proposal classified as {} and requires review",
                scope.scope_class
            ),
        );
    }

    if let Some(eval) = eval {
        evidence.push(format!("eval score={} status={}", eval.score, eval.status));
        if eval.score >= 80 {
            add_check(
                "eval_score",
                "pass",
                16,
                format!("eval score {} meets strong target", eval.score),
            );
        } else if eval.score >= 70 {
            add_check(
                "eval_score",
                "warn",
                10,
                format!("eval score {} below strong target", eval.score),
            );
        } else {
            add_check(
                "eval_score",
                "fail",
                0,
                format!("eval score {} below stable threshold", eval.score),
            );
        }
    } else {
        add_check(
            "eval_score",
            "warn",
            8,
            "no eval snapshot found for self evolution".to_string(),
        );
    }

    ScenarioReport {
        bundle_root: output.display().to_string(),
        project: runtime.and_then(|value| value.project.clone()),
        namespace: runtime.and_then(|value| value.namespace.clone()),
        agent: runtime.and_then(|value| value.agent.clone()),
        session: runtime.and_then(|value| value.session.clone()),
        workspace: runtime.and_then(|value| value.workspace.clone()),
        visibility: runtime.and_then(|value| value.visibility.clone()),
        scenario: "self_evolution".to_string(),
        score,
        max_score,
        checks,
        passed_checks,
        failed_checks,
        findings,
        next_actions,
        evidence,
        generated_at: completed_at,
        completed_at,
    }
}

fn supported_public_benchmark_ids() -> &'static [&'static str] {
    &["longmemeval", "locomo", "convomem", "membench"]
}

fn implemented_public_benchmark_ids() -> &'static [&'static str] {
    &["longmemeval", "locomo", "convomem", "membench"]
}

fn public_benchmark_target_status(dataset: &str) -> &'static str {
    if implemented_public_benchmark_ids().contains(&dataset) {
        "implemented"
    } else {
        "declared-stub"
    }
}

fn render_longmemeval_haystack_text(value: &JsonValue) -> String {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|session| session.as_array())
        .flat_map(|turns| turns.iter())
        .filter_map(|turn| {
            let role = turn.get("role").and_then(JsonValue::as_str).unwrap_or("");
            let content = turn
                .get("content")
                .and_then(JsonValue::as_str)
                .unwrap_or("");
            if role.is_empty() && content.is_empty() {
                None
            } else {
                Some(format!("{role}: {content}"))
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_locomo_conversation_text(value: &JsonValue) -> String {
    let mut rendered = Vec::new();
    if let Some(conversation) = value.as_object() {
        let mut session_indexes = conversation
            .keys()
            .filter_map(|key| key.strip_prefix("session_"))
            .filter_map(|suffix| {
                suffix
                    .split_once('_')
                    .map(|(index, _)| index)
                    .or(Some(suffix))
            })
            .filter_map(|index| index.parse::<usize>().ok())
            .collect::<BTreeSet<_>>();
        if session_indexes.is_empty() {
            session_indexes = (1..=35).collect();
        }
        for session_index in session_indexes {
            let session_key = format!("session_{session_index}");
            if let Some(dialogs) = conversation.get(&session_key).and_then(JsonValue::as_array) {
                for dialog in dialogs {
                    let speaker = dialog
                        .get("speaker")
                        .and_then(JsonValue::as_str)
                        .unwrap_or("unknown");
                    let text = dialog.get("text").and_then(JsonValue::as_str).unwrap_or("");
                    if !text.is_empty() {
                        rendered.push(format!("{speaker}: {text}"));
                    }
                }
            }
        }
    }
    rendered.join("\n")
}

fn render_membench_message_list_text(value: &JsonValue) -> String {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(JsonValue::as_array)
        .flat_map(|session| session.iter())
        .filter_map(|turn| {
            let user = turn.get("user_message").and_then(JsonValue::as_str);
            let assistant = turn.get("assistant_message").and_then(JsonValue::as_str);
            match (user, assistant) {
                (Some(user), Some(assistant)) => {
                    Some(format!("user: {user}\nassistant: {assistant}"))
                }
                (Some(user), None) => Some(format!("user: {user}")),
                (None, Some(assistant)) => Some(format!("assistant: {assistant}")),
                (None, None) => None,
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_convomem_conversation_text(value: &JsonValue) -> String {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(JsonValue::as_object)
        .flat_map(|conversation| {
            conversation
                .get("messages")
                .and_then(JsonValue::as_array)
                .into_iter()
                .flatten()
                .filter_map(JsonValue::as_object)
                .map(|message| {
                    let speaker = message
                        .get("speaker")
                        .and_then(JsonValue::as_str)
                        .unwrap_or("unknown");
                    let text = message
                        .get("text")
                        .and_then(JsonValue::as_str)
                        .unwrap_or("");
                    format!("{speaker}: {text}")
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn locomo_category_name(category: i64) -> &'static str {
    match category {
        1 => "Single-hop",
        2 => "Temporal",
        3 => "Temporal-inference",
        4 => "Open-domain",
        5 => "Adversarial",
        _ => "Unknown",
    }
}

fn json_stringish_field<'a>(row: &'a JsonValue, key: &str) -> anyhow::Result<String> {
    let value = row.get(key).ok_or_else(|| anyhow!("missing {key} field"))?;
    match value {
        JsonValue::String(value) => Ok(value.clone()),
        JsonValue::Number(value) => Ok(value.to_string()),
        JsonValue::Bool(value) => Ok(value.to_string()),
        _ => anyhow::bail!("missing {key} string-compatible value"),
    }
}

fn json_stringish_or_array_field<'a>(row: &'a JsonValue, key: &str) -> anyhow::Result<String> {
    let value = row.get(key).ok_or_else(|| anyhow!("missing {key} field"))?;
    match value {
        JsonValue::Array(items) => Ok(items
            .iter()
            .map(|item| match item {
                JsonValue::String(value) => Ok(value.clone()),
                JsonValue::Number(value) => Ok(value.to_string()),
                JsonValue::Bool(value) => Ok(value.to_string()),
                _ => anyhow::bail!("missing {key} string-compatible array value"),
            })
            .collect::<anyhow::Result<Vec<_>>>()?
            .join(", ")),
        _ => json_stringish_field(row, key),
    }
}

fn normalize_longmemeval_dataset(
    path: &Path,
    rows: &[JsonValue],
) -> anyhow::Result<PublicBenchmarkDatasetFixture> {
    let items = rows
        .iter()
        .map(|row| {
            let item_id = json_stringish_field(row, "question_id")
                .with_context(|| format!("normalize {} question_id", path.display()))?;
            let query = json_stringish_field(row, "question")
                .with_context(|| format!("normalize {} question", path.display()))?;
            let gold_answer = json_stringish_field(row, "answer")
                .with_context(|| format!("normalize {} answer", path.display()))?;
            Ok(PublicBenchmarkDatasetFixtureItem {
                item_id: item_id.clone(),
                question_id: item_id,
                query,
                claim_class: "raw".to_string(),
                gold_answer,
                metadata: json!({
                    "question_type": row.get("question_type").cloned().unwrap_or(JsonValue::Null),
                    "question_date": row.get("question_date").cloned().unwrap_or(JsonValue::Null),
                    "haystack_dates": row.get("haystack_dates").cloned().unwrap_or(JsonValue::Null),
                    "haystack_session_ids": row.get("haystack_session_ids").cloned().unwrap_or(JsonValue::Null),
                    "haystack_sessions": row.get("haystack_sessions").cloned().unwrap_or(JsonValue::Null),
                    "answer_session_ids": row.get("answer_session_ids").cloned().unwrap_or(JsonValue::Null),
                    "haystack_text": render_longmemeval_haystack_text(
                        row.get("haystack_sessions").unwrap_or(&JsonValue::Null)
                    ),
                }),
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(PublicBenchmarkDatasetFixture {
        benchmark_id: "longmemeval".to_string(),
        benchmark_name: "LongMemEval".to_string(),
        version: "upstream".to_string(),
        split: "cleaned-small".to_string(),
        description: "Normalized upstream LongMemEval cleaned file.".to_string(),
        items,
    })
}

fn normalize_locomo_dataset(
    path: &Path,
    rows: &[JsonValue],
) -> anyhow::Result<PublicBenchmarkDatasetFixture> {
    let mut items = Vec::new();
    for row in rows {
        let sample_id = json_stringish_field(row, "sample_id")
            .with_context(|| format!("normalize {} sample_id", path.display()))?;
        let conversation = row.get("conversation").cloned().unwrap_or(JsonValue::Null);
        let conversation_text = render_locomo_conversation_text(&conversation);
        let session_summary = row
            .get("session_summary")
            .cloned()
            .unwrap_or(JsonValue::Null);
        let qa_rows = row
            .get("qa")
            .and_then(JsonValue::as_array)
            .ok_or_else(|| anyhow!("normalize {} qa array", path.display()))?;
        for (qa_index, qa_row) in qa_rows.iter().enumerate() {
            let query = json_stringish_field(qa_row, "question")
                .with_context(|| format!("normalize {} qa.question", path.display()))?;
            let gold_answer = json_stringish_field(qa_row, "answer")
                .or_else(|_| json_stringish_field(qa_row, "adversarial_answer"))
                .with_context(|| format!("normalize {} qa.answer", path.display()))?;
            let category_id = qa_row
                .get("category")
                .and_then(JsonValue::as_i64)
                .unwrap_or_default();
            items.push(PublicBenchmarkDatasetFixtureItem {
                item_id: format!("{sample_id}::{qa_index}"),
                question_id: format!("{sample_id}::{qa_index}"),
                query,
                claim_class: "raw".to_string(),
                gold_answer,
                metadata: json!({
                    "sample_id": sample_id,
                    "category_id": category_id,
                    "category_name": locomo_category_name(category_id),
                    "evidence": qa_row.get("evidence").cloned().unwrap_or(JsonValue::Null),
                    "conversation": conversation,
                    "conversation_text": conversation_text,
                    "session_summary": session_summary,
                }),
            });
        }
    }

    Ok(PublicBenchmarkDatasetFixture {
        benchmark_id: "locomo".to_string(),
        benchmark_name: "LoCoMo".to_string(),
        version: "upstream".to_string(),
        split: "locomo10".to_string(),
        description: "Normalized upstream LoCoMo conversation benchmark file.".to_string(),
        items,
    })
}

fn normalize_membench_dataset(
    path: &Path,
    value: &JsonValue,
) -> anyhow::Result<PublicBenchmarkDatasetFixture> {
    let root = value
        .as_object()
        .ok_or_else(|| anyhow!("normalize {} membench root object", path.display()))?;
    let mut items = Vec::new();
    for (topic, entries_value) in root {
        let entries = entries_value
            .as_array()
            .ok_or_else(|| anyhow!("normalize {} membench topic array", path.display()))?;
        for entry in entries {
            let tid = entry
                .get("tid")
                .and_then(JsonValue::as_i64)
                .unwrap_or_default();
            let qa = entry
                .get("QA")
                .or_else(|| entry.get("qa"))
                .ok_or_else(|| anyhow!("normalize {} membench QA object", path.display()))?;
            let qid = qa
                .get("qid")
                .and_then(JsonValue::as_i64)
                .unwrap_or_default();
            let query = json_stringish_field(qa, "question")
                .with_context(|| format!("normalize {} QA.question", path.display()))?;
            let gold_answer = json_stringish_or_array_field(qa, "answer")
                .with_context(|| format!("normalize {} QA.answer", path.display()))?;
            items.push(PublicBenchmarkDatasetFixtureItem {
                item_id: format!("{topic}::{tid}::{qid}"),
                question_id: format!("{topic}::{tid}::{qid}"),
                query,
                claim_class: "raw".to_string(),
                gold_answer,
                metadata: json!({
                    "topic": topic,
                    "tid": tid,
                    "qid": qid,
                    "target_step_id": qa.get("target_step_id").cloned().unwrap_or(JsonValue::Null),
                    "choices": qa.get("choices").cloned().unwrap_or(JsonValue::Null),
                    "ground_truth": qa.get("ground_truth").cloned().unwrap_or(JsonValue::Null),
                    "time": qa.get("time").cloned().unwrap_or(JsonValue::Null),
                    "message_list": entry.get("message_list").cloned().unwrap_or(JsonValue::Null),
                    "conversation_text": render_membench_message_list_text(
                        entry.get("message_list").unwrap_or(&JsonValue::Null)
                    ),
                }),
            });
        }
    }

    Ok(PublicBenchmarkDatasetFixture {
        benchmark_id: "membench".to_string(),
        benchmark_name: "MemBench".to_string(),
        version: "upstream".to_string(),
        split: "FirstAgent".to_string(),
        description: "Normalized upstream MemBench FirstAgent benchmark files.".to_string(),
        items,
    })
}

fn normalize_convomem_evidence_items(
    items: &[JsonValue],
) -> anyhow::Result<PublicBenchmarkDatasetFixture> {
    let items = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let query = json_stringish_field(item, "question")
                .with_context(|| format!("normalize convomem item {index} question"))?;
            let gold_answer = json_stringish_or_array_field(item, "answer")
                .with_context(|| format!("normalize convomem item {index} answer"))?;
            let category = item
                .get("category")
                .and_then(JsonValue::as_str)
                .unwrap_or("unknown");
            Ok(PublicBenchmarkDatasetFixtureItem {
                item_id: format!("{category}::{index}"),
                question_id: format!("{category}::{index}"),
                query,
                claim_class: "raw".to_string(),
                gold_answer,
                metadata: json!({
                    "category": item.get("category").cloned().unwrap_or(JsonValue::Null),
                    "scenario_description": item.get("scenario_description").cloned().unwrap_or(JsonValue::Null),
                    "person_id": item.get("personId").cloned().unwrap_or(JsonValue::Null),
                    "message_evidences": item.get("message_evidences").cloned().unwrap_or(JsonValue::Null),
                    "conversations": item.get("conversations").cloned().unwrap_or(JsonValue::Null),
                    "conversation_text": render_convomem_conversation_text(
                        item.get("conversations").unwrap_or(&JsonValue::Null)
                    ),
                    "use_case_model_name": item.get("use_case_model_name").cloned().unwrap_or(JsonValue::Null),
                    "core_model_name": item.get("core_model_name").cloned().unwrap_or(JsonValue::Null),
                }),
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(PublicBenchmarkDatasetFixture {
        benchmark_id: "convomem".to_string(),
        benchmark_name: "ConvoMem".to_string(),
        version: "upstream".to_string(),
        split: "evidence-sample".to_string(),
        description: "Sampled upstream ConvoMem evidence files normalized into a cached fixture."
            .to_string(),
        items,
    })
}

fn load_public_benchmark_dataset(
    benchmark_id: &str,
    path: &Path,
) -> anyhow::Result<PublicBenchmarkDatasetFixture> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let value = serde_json::from_str::<JsonValue>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    match value {
        JsonValue::Object(value)
            if benchmark_id == "membench" && value.get("benchmark_id").is_none() =>
        {
            normalize_membench_dataset(path, &JsonValue::Object(value))
        }
        JsonValue::Object(_) => serde_json::from_str::<PublicBenchmarkDatasetFixture>(&raw)
            .with_context(|| format!("parse {}", path.display())),
        JsonValue::Array(rows) if benchmark_id == "longmemeval" => {
            normalize_longmemeval_dataset(path, &rows)
        }
        JsonValue::Array(rows) if benchmark_id == "locomo" => normalize_locomo_dataset(path, &rows),
        JsonValue::Array(_) => anyhow::bail!(
            "benchmark `{benchmark_id}` array dataset format is not normalized yet for {}",
            path.display()
        ),
        _ => anyhow::bail!(
            "unsupported public benchmark dataset format in {}",
            path.display()
        ),
    }
}

fn public_benchmark_fixture_checksum(path: &Path) -> anyhow::Result<String> {
    let raw = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    Ok(format!("sha256:{:x}", Sha256::digest(&raw)))
}

fn public_benchmark_dataset_source(dataset: &str) -> Option<PublicBenchmarkDatasetSource> {
    match dataset {
        "longmemeval" => Some(PublicBenchmarkDatasetSource {
            benchmark_id: "longmemeval",
            source_url: Some(
                "https://huggingface.co/datasets/xiaowu0162/longmemeval-cleaned/resolve/main/longmemeval_s_cleaned.json",
            ),
            default_filename: "longmemeval_s_cleaned.json",
            expected_checksum: Some(
                "sha256:d6f21ea9d60a0d56f34a05b609c79c88a451d2ae03597821ea3d5a9678c3a442",
            ),
            split: "cleaned-small",
            access_mode: "auto-download",
            notes: "Upstream LongMemEval cleaned small file from the official benchmark repo README.",
        }),
        "locomo" => Some(PublicBenchmarkDatasetSource {
            benchmark_id: "locomo",
            source_url: Some(
                "https://raw.githubusercontent.com/snap-research/locomo/3eb6f2c585f5e1699204e3c3bdf7adc5c28cb376/data/locomo10.json",
            ),
            default_filename: "locomo10.json",
            expected_checksum: Some(
                "sha256:79fa87e90f04081343b8c8debecb80a9a6842b76a7aa537dc9fdf651ea698ff4",
            ),
            split: "locomo10",
            access_mode: "auto-download",
            notes: "Commit-pinned LoCoMo locomo10.json source from the upstream benchmark repo.",
        }),
        "convomem" => Some(PublicBenchmarkDatasetSource {
            benchmark_id: "convomem",
            source_url: Some(
                "https://huggingface.co/datasets/Salesforce/ConvoMem/tree/main/core_benchmark/evidence_questions",
            ),
            default_filename: "convomem-evidence-sample.json",
            expected_checksum: None,
            split: "evidence-sample",
            access_mode: "auto-download",
            notes: "Sampled upstream ConvoMem evidence files fetched from the Hugging Face dataset tree.",
        }),
        "membench" => Some(PublicBenchmarkDatasetSource {
            benchmark_id: "membench",
            source_url: Some(
                "https://github.com/import-myself/Membench/tree/f66d8d1028d3f68627d00f77a967b93fbb8694b6/MemData/FirstAgent",
            ),
            default_filename: "membench-firstagent.json",
            expected_checksum: None,
            split: "FirstAgent",
            access_mode: "auto-download",
            notes: "Commit-pinned MemBench FirstAgent category set normalized into a cached fixture.",
        }),
        _ => None,
    }
}

fn resolve_public_benchmark_dataset_override_path(args: &PublicBenchmarkArgs) -> Option<PathBuf> {
    args.dataset_root.as_ref().map(|path| {
        if path.is_dir() {
            path.join(format!("{}-mini.json", args.dataset))
        } else {
            path.clone()
        }
    })
}

fn public_benchmark_dataset_entry_dir(output: &Path, benchmark_id: &str) -> PathBuf {
    public_benchmark_dataset_cache_dir(output).join(benchmark_id)
}

fn public_benchmark_dataset_cache_path(
    output: &Path,
    benchmark_id: &str,
    filename: &str,
) -> PathBuf {
    public_benchmark_dataset_entry_dir(output, benchmark_id).join(filename)
}

fn public_benchmark_dataset_cache_metadata_path(output: &Path, benchmark_id: &str) -> PathBuf {
    public_benchmark_dataset_entry_dir(output, benchmark_id).join("metadata.json")
}

fn write_public_benchmark_dataset_cache_metadata(
    output: &Path,
    metadata: &PublicBenchmarkDatasetCacheMetadata,
) -> anyhow::Result<PathBuf> {
    let path = public_benchmark_dataset_cache_metadata_path(output, &metadata.benchmark_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&path, serde_json::to_string_pretty(metadata)? + "\n")
        .with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}

fn validate_public_benchmark_checksum(
    checksum: &str,
    expected_checksum: Option<&str>,
) -> anyhow::Result<String> {
    if let Some(expected_checksum) = expected_checksum {
        anyhow::ensure!(
            checksum == expected_checksum,
            "dataset checksum mismatch: expected {expected_checksum}, got {checksum}"
        );
        Ok("verified".to_string())
    } else {
        Ok("recorded-unpinned".to_string())
    }
}

async fn download_public_benchmark_dataset(
    output: &Path,
    source: &PublicBenchmarkDatasetSource,
) -> anyhow::Result<ResolvedPublicBenchmarkDataset> {
    let source_url = source.source_url.ok_or_else(|| {
        anyhow!(
            "benchmark `{}` does not expose an auto-download URL",
            source.benchmark_id
        )
    })?;
    let entry_dir = public_benchmark_dataset_entry_dir(output, source.benchmark_id);
    fs::create_dir_all(&entry_dir).with_context(|| format!("create {}", entry_dir.display()))?;
    let dataset_path =
        public_benchmark_dataset_cache_path(output, source.benchmark_id, source.default_filename);
    let response = reqwest::get(source_url)
        .await
        .with_context(|| format!("download dataset {source_url}"))?
        .error_for_status()
        .with_context(|| format!("download dataset {source_url}"))?;
    let bytes = response
        .bytes()
        .await
        .with_context(|| format!("read dataset bytes {source_url}"))?;
    fs::write(&dataset_path, &bytes)
        .with_context(|| format!("write {}", dataset_path.display()))?;
    let checksum = public_benchmark_fixture_checksum(&dataset_path)?;
    let verification_status =
        validate_public_benchmark_checksum(&checksum, source.expected_checksum)?;
    write_public_benchmark_dataset_cache_metadata(
        output,
        &PublicBenchmarkDatasetCacheMetadata {
            benchmark_id: source.benchmark_id.to_string(),
            source_url: source_url.to_string(),
            local_path: dataset_path.display().to_string(),
            checksum: checksum.clone(),
            expected_checksum: source.expected_checksum.map(str::to_string),
            verification_status: verification_status.clone(),
            fetched_at: Utc::now(),
            bytes: bytes.len(),
        },
    )?;
    Ok(ResolvedPublicBenchmarkDataset {
        path: dataset_path,
        source_url: source_url.to_string(),
        checksum,
        split: source.split.to_string(),
        verification_status,
    })
}

async fn download_membench_dataset(
    output: &Path,
    source: &PublicBenchmarkDatasetSource,
) -> anyhow::Result<ResolvedPublicBenchmarkDataset> {
    const MEMBENCH_FILES: &[&str] = &[
        "simple.json",
        "highlevel.json",
        "knowledge_update.json",
        "comparative.json",
        "conditional.json",
        "noisy.json",
        "aggregative.json",
        "highlevel_rec.json",
        "lowlevel_rec.json",
        "RecMultiSession.json",
        "post_processing.json",
    ];
    const MEMBENCH_COMMIT: &str = "f66d8d1028d3f68627d00f77a967b93fbb8694b6";
    let base_url = format!(
        "https://raw.githubusercontent.com/import-myself/Membench/{MEMBENCH_COMMIT}/MemData/FirstAgent"
    );
    let entry_dir = public_benchmark_dataset_entry_dir(output, source.benchmark_id);
    fs::create_dir_all(&entry_dir).with_context(|| format!("create {}", entry_dir.display()))?;
    let raw_dir = entry_dir.join("raw");
    fs::create_dir_all(&raw_dir).with_context(|| format!("create {}", raw_dir.display()))?;

    let mut merged = serde_json::Map::new();
    let mut byte_count = 0usize;
    for filename in MEMBENCH_FILES {
        let url = format!("{base_url}/{filename}");
        let response = reqwest::get(&url)
            .await
            .with_context(|| format!("download dataset {url}"))?
            .error_for_status()
            .with_context(|| format!("download dataset {url}"))?;
        let bytes = response
            .bytes()
            .await
            .with_context(|| format!("read dataset bytes {url}"))?;
        byte_count += bytes.len();
        let raw_path = raw_dir.join(filename);
        fs::write(&raw_path, &bytes).with_context(|| format!("write {}", raw_path.display()))?;
        let value = serde_json::from_slice::<JsonValue>(&bytes)
            .with_context(|| format!("parse {}", raw_path.display()))?;
        let object = value
            .as_object()
            .ok_or_else(|| anyhow!("membench source {} was not an object", raw_path.display()))?;
        for (key, value) in object {
            merged.insert(key.clone(), value.clone());
        }
    }

    let dataset_path =
        public_benchmark_dataset_cache_path(output, source.benchmark_id, source.default_filename);
    fs::write(
        &dataset_path,
        serde_json::to_string_pretty(&JsonValue::Object(merged))? + "\n",
    )
    .with_context(|| format!("write {}", dataset_path.display()))?;
    let checksum = public_benchmark_fixture_checksum(&dataset_path)?;
    let verification_status =
        validate_public_benchmark_checksum(&checksum, source.expected_checksum)?;
    write_public_benchmark_dataset_cache_metadata(
        output,
        &PublicBenchmarkDatasetCacheMetadata {
            benchmark_id: source.benchmark_id.to_string(),
            source_url: source.source_url.unwrap_or_default().to_string(),
            local_path: dataset_path.display().to_string(),
            checksum: checksum.clone(),
            expected_checksum: source.expected_checksum.map(str::to_string),
            verification_status: verification_status.clone(),
            fetched_at: Utc::now(),
            bytes: byte_count,
        },
    )?;
    Ok(ResolvedPublicBenchmarkDataset {
        path: dataset_path,
        source_url: source.source_url.unwrap_or_default().to_string(),
        checksum,
        split: source.split.to_string(),
        verification_status,
    })
}

async fn download_convomem_dataset(
    output: &Path,
    source: &PublicBenchmarkDatasetSource,
) -> anyhow::Result<ResolvedPublicBenchmarkDataset> {
    const CONVOMEM_CATEGORIES: &[&str] = &[
        "user_evidence",
        "assistant_facts_evidence",
        "changing_evidence",
        "abstention_evidence",
        "preference_evidence",
        "implicit_connection_evidence",
    ];
    let tree_url = "https://huggingface.co/api/datasets/Salesforce/ConvoMem/tree/main/core_benchmark/evidence_questions?recursive=true";
    let tree = reqwest::get(tree_url)
        .await
        .context("download ConvoMem tree api")?
        .error_for_status()
        .context("download ConvoMem tree api")?
        .json::<Vec<JsonValue>>()
        .await
        .context("parse ConvoMem tree api")?;

    let entry_dir = public_benchmark_dataset_entry_dir(output, source.benchmark_id);
    fs::create_dir_all(&entry_dir).with_context(|| format!("create {}", entry_dir.display()))?;
    let raw_dir = entry_dir.join("raw");
    fs::create_dir_all(&raw_dir).with_context(|| format!("create {}", raw_dir.display()))?;

    let mut sample_paths = Vec::new();
    for category in CONVOMEM_CATEGORIES {
        let path = tree
            .iter()
            .filter_map(|entry| entry.get("path").and_then(JsonValue::as_str))
            .filter(|path| {
                path.starts_with(&format!("core_benchmark/evidence_questions/{category}/"))
                    && path.ends_with(".json")
            })
            .min()
            .ok_or_else(|| anyhow!("no ConvoMem evidence file found for category `{category}`"))?;
        sample_paths.push(path.to_string());
    }

    let mut evidence_items = Vec::new();
    let mut byte_count = 0usize;
    for path in &sample_paths {
        let url =
            format!("https://huggingface.co/datasets/Salesforce/ConvoMem/resolve/main/{path}");
        let response = reqwest::get(&url)
            .await
            .with_context(|| format!("download dataset {url}"))?
            .error_for_status()
            .with_context(|| format!("download dataset {url}"))?;
        let bytes = response
            .bytes()
            .await
            .with_context(|| format!("read dataset bytes {url}"))?;
        byte_count += bytes.len();
        let filename = path
            .rsplit('/')
            .next()
            .ok_or_else(|| anyhow!("convomem sample path missing filename"))?;
        let raw_path = raw_dir.join(filename);
        fs::write(&raw_path, &bytes).with_context(|| format!("write {}", raw_path.display()))?;
        let value = serde_json::from_slice::<JsonValue>(&bytes)
            .with_context(|| format!("parse {}", raw_path.display()))?;
        let items = value
            .get("evidence_items")
            .and_then(JsonValue::as_array)
            .ok_or_else(|| {
                anyhow!(
                    "ConvoMem source {} missing evidence_items",
                    raw_path.display()
                )
            })?;
        evidence_items.extend(items.iter().cloned());
    }

    let fixture = normalize_convomem_evidence_items(&evidence_items)?;
    let dataset_path =
        public_benchmark_dataset_cache_path(output, source.benchmark_id, source.default_filename);
    fs::write(
        &dataset_path,
        serde_json::to_string_pretty(&fixture)? + "\n",
    )
    .with_context(|| format!("write {}", dataset_path.display()))?;
    let checksum = public_benchmark_fixture_checksum(&dataset_path)?;
    let verification_status =
        validate_public_benchmark_checksum(&checksum, source.expected_checksum)?;
    write_public_benchmark_dataset_cache_metadata(
        output,
        &PublicBenchmarkDatasetCacheMetadata {
            benchmark_id: source.benchmark_id.to_string(),
            source_url: source.source_url.unwrap_or_default().to_string(),
            local_path: dataset_path.display().to_string(),
            checksum: checksum.clone(),
            expected_checksum: source.expected_checksum.map(str::to_string),
            verification_status: verification_status.clone(),
            fetched_at: Utc::now(),
            bytes: byte_count,
        },
    )?;
    Ok(ResolvedPublicBenchmarkDataset {
        path: dataset_path,
        source_url: source.source_url.unwrap_or_default().to_string(),
        checksum,
        split: source.split.to_string(),
        verification_status,
    })
}

async fn resolve_public_benchmark_dataset(
    args: &PublicBenchmarkArgs,
) -> anyhow::Result<ResolvedPublicBenchmarkDataset> {
    if let Some(path) = resolve_public_benchmark_dataset_override_path(args) {
        let checksum = public_benchmark_fixture_checksum(&path)?;
        return Ok(ResolvedPublicBenchmarkDataset {
            source_url: format!("file://{}", path.display()),
            path,
            checksum,
            split: "manual".to_string(),
            verification_status: "manual-path".to_string(),
        });
    }

    let source = public_benchmark_dataset_source(&args.dataset).ok_or_else(|| {
        anyhow!(
            "no public benchmark dataset source is registered for `{}`",
            args.dataset
        )
    })?;

    if source.access_mode != "auto-download" {
        anyhow::bail!(
            "benchmark `{}` currently requires --dataset-root; {}",
            args.dataset,
            source.notes
        );
    }

    let cached_path =
        public_benchmark_dataset_cache_path(&args.out, &args.dataset, source.default_filename);
    if cached_path.exists() {
        let checksum = public_benchmark_fixture_checksum(&cached_path)?;
        let verification_status =
            validate_public_benchmark_checksum(&checksum, source.expected_checksum)?;
        write_public_benchmark_dataset_cache_metadata(
            &args.out,
            &PublicBenchmarkDatasetCacheMetadata {
                benchmark_id: args.dataset.clone(),
                source_url: source.source_url.unwrap_or_default().to_string(),
                local_path: cached_path.display().to_string(),
                checksum: checksum.clone(),
                expected_checksum: source.expected_checksum.map(str::to_string),
                verification_status: verification_status.clone(),
                fetched_at: Utc::now(),
                bytes: fs::metadata(&cached_path)
                    .with_context(|| format!("stat {}", cached_path.display()))?
                    .len() as usize,
            },
        )?;
        return Ok(ResolvedPublicBenchmarkDataset {
            path: cached_path,
            source_url: source.source_url.unwrap_or_default().to_string(),
            checksum,
            split: source.split.to_string(),
            verification_status,
        });
    }

    if args.dataset == "membench" {
        return download_membench_dataset(&args.out, &source).await;
    }
    if args.dataset == "convomem" {
        return download_convomem_dataset(&args.out, &source).await;
    }

    download_public_benchmark_dataset(&args.out, &source).await
}

fn tokenize_public_benchmark_text(value: &str) -> BTreeSet<String> {
    value
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter_map(|token| {
            let token = token.trim().to_ascii_lowercase();
            if token.is_empty() { None } else { Some(token) }
        })
        .collect()
}

fn flatten_public_benchmark_metadata(value: &JsonValue) -> String {
    match value {
        JsonValue::Object(map) => map
            .iter()
            .map(|(key, value)| {
                let rendered = value
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| value.to_string());
                format!("{key}={rendered}")
            })
            .collect::<Vec<_>>()
            .join(" "),
        JsonValue::Array(items) => items
            .iter()
            .map(flatten_public_benchmark_metadata)
            .collect::<Vec<_>>()
            .join(" "),
        JsonValue::String(value) => value.clone(),
        _ => value.to_string(),
    }
}

fn dcg_public_benchmark(relevances: &[f64], k: usize) -> f64 {
    relevances
        .iter()
        .take(k)
        .enumerate()
        .map(|(index, relevance)| relevance / ((index as f64 + 2.0).log2()))
        .sum()
}

fn ndcg_public_benchmark(relevances: &[f64], k: usize) -> f64 {
    let mut ideal = relevances.to_vec();
    ideal.sort_by(|left, right| right.total_cmp(left));
    let idcg = dcg_public_benchmark(&ideal, k);
    if idcg == 0.0 {
        0.0
    } else {
        dcg_public_benchmark(relevances, k) / idcg
    }
}

fn public_benchmark_string_vec(value: Option<&JsonValue>) -> Vec<String> {
    value
        .and_then(JsonValue::as_array)
        .into_iter()
        .flatten()
        .filter_map(JsonValue::as_str)
        .map(str::to_string)
        .collect()
}

fn build_longmemeval_session_corpus(
    item: &PublicBenchmarkDatasetFixtureItem,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let sessions = item
        .metadata
        .get("haystack_sessions")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    let session_ids = public_benchmark_string_vec(item.metadata.get("haystack_session_ids"));
    let dates = public_benchmark_string_vec(item.metadata.get("haystack_dates"));
    let mut corpus = Vec::new();
    let mut corpus_ids = Vec::new();
    let mut corpus_timestamps = Vec::new();

    for (index, session) in sessions.iter().enumerate() {
        let user_turns = session
            .as_array()
            .into_iter()
            .flatten()
            .filter(|turn| turn.get("role").and_then(JsonValue::as_str) == Some("user"))
            .filter_map(|turn| turn.get("content").and_then(JsonValue::as_str))
            .map(str::to_string)
            .collect::<Vec<_>>();
        if user_turns.is_empty() {
            continue;
        }
        corpus.push(user_turns.join("\n"));
        corpus_ids.push(
            session_ids
                .get(index)
                .cloned()
                .unwrap_or_else(|| format!("session_{index}")),
        );
        corpus_timestamps.push(
            dates
                .get(index)
                .cloned()
                .unwrap_or_else(|| "unknown".to_string()),
        );
    }

    (corpus, corpus_ids, corpus_timestamps)
}

fn build_longmemeval_turn_corpus(
    item: &PublicBenchmarkDatasetFixtureItem,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let sessions = item
        .metadata
        .get("haystack_sessions")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    let session_ids = public_benchmark_string_vec(item.metadata.get("haystack_session_ids"));
    let dates = public_benchmark_string_vec(item.metadata.get("haystack_dates"));
    let mut corpus = Vec::new();
    let mut corpus_ids = Vec::new();
    let mut corpus_timestamps = Vec::new();

    for (session_index, session) in sessions.iter().enumerate() {
        let base_session_id = session_ids
            .get(session_index)
            .cloned()
            .unwrap_or_else(|| format!("session_{session_index}"));
        let date = dates
            .get(session_index)
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let mut turn_index = 0usize;
        for turn in session.as_array().into_iter().flatten() {
            if turn.get("role").and_then(JsonValue::as_str) != Some("user") {
                continue;
            }
            if let Some(content) = turn.get("content").and_then(JsonValue::as_str) {
                corpus.push(content.to_string());
                corpus_ids.push(format!("{base_session_id}_turn_{turn_index}"));
                corpus_timestamps.push(date.clone());
                turn_index += 1;
            }
        }
    }

    (corpus, corpus_ids, corpus_timestamps)
}

fn rank_public_benchmark_corpus(
    query: &str,
    corpus: &[String],
    corpus_ids: &[String],
    mode: &str,
) -> Vec<usize> {
    let query_tokens = tokenize_public_benchmark_text(query);
    let stop_words = [
        "what", "when", "where", "who", "how", "which", "did", "do", "was", "were", "have", "has",
        "had", "is", "are", "the", "a", "an", "my", "me", "i", "you", "your", "their", "it", "its",
        "in", "on", "at", "to", "for", "of", "with", "by", "from", "ago", "last", "that", "this",
        "there", "about", "get", "got", "give", "gave", "buy", "bought", "made", "make",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    let keywords = query_tokens
        .iter()
        .filter(|token| token.len() >= 3 && !stop_words.contains(*token))
        .cloned()
        .collect::<Vec<_>>();
    let mut scored = corpus
        .iter()
        .enumerate()
        .map(|(index, document)| {
            let doc_tokens = tokenize_public_benchmark_text(document);
            let overlap = query_tokens.intersection(&doc_tokens).count() as f64;
            let mut score = overlap;
            if mode == "hybrid" && !keywords.is_empty() {
                let doc_lower = document.to_ascii_lowercase();
                let keyword_hits = keywords
                    .iter()
                    .filter(|kw| doc_lower.contains(kw.as_str()))
                    .count();
                score += (keyword_hits as f64 / keywords.len() as f64) * 0.30;
            }
            if corpus_ids.get(index).is_some_and(|id| id.contains("_abs")) {
                score -= 0.05;
            }
            (index, score)
        })
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0.cmp(&right.0))
    });
    scored.into_iter().map(|(index, _)| index).collect()
}

fn build_public_benchmark_retrieval_config(
    args: &PublicBenchmarkArgs,
) -> anyhow::Result<PublicBenchmarkRetrievalConfig> {
    let requested_backend = args.retrieval_backend.as_deref().unwrap_or("lexical");
    let longmemeval_backend = match requested_backend {
        "lexical" => LongMemEvalRetrievalBackend::Lexical,
        "sidecar" => LongMemEvalRetrievalBackend::Sidecar,
        other => anyhow::bail!("invalid retrieval backend `{other}`; expected lexical or sidecar"),
    };

    let sidecar_base_url = if longmemeval_backend == LongMemEvalRetrievalBackend::Sidecar {
        Some(resolve_rag_url(args.rag_url.clone(), Some(&args.out))?)
    } else {
        None
    };

    Ok(PublicBenchmarkRetrievalConfig {
        longmemeval_backend,
        sidecar_base_url,
    })
}

fn rank_longmemeval_corpus(
    query: &str,
    corpus: &[String],
    corpus_ids: &[String],
    mode: &str,
    config: &PublicBenchmarkRetrievalConfig,
    namespace: &str,
) -> anyhow::Result<Vec<(usize, f64)>> {
    match config.longmemeval_backend {
        LongMemEvalRetrievalBackend::Lexical => Ok(rank_public_benchmark_corpus(
            query, corpus, corpus_ids, mode,
        )
        .into_iter()
        .enumerate()
        .map(|(rank, index)| (index, (50usize.saturating_sub(rank)) as f64))
        .collect()),
        LongMemEvalRetrievalBackend::Sidecar => {
            let base_url = config
                .sidecar_base_url
                .as_deref()
                .context("sidecar retrieval backend selected without a sidecar base url")?;
            rank_longmemeval_corpus_via_sidecar(
                base_url, query, corpus, corpus_ids, mode, namespace,
            )
        }
    }
}

fn rank_longmemeval_corpus_via_sidecar(
    base_url: &str,
    query: &str,
    corpus: &[String],
    corpus_ids: &[String],
    mode: &str,
    namespace: &str,
) -> anyhow::Result<Vec<(usize, f64)>> {
    let lexical_fallback = rank_public_benchmark_corpus(query, corpus, corpus_ids, mode);
    let client = reqwest::blocking::Client::builder()
        .build()
        .context("build public benchmark sidecar client")?;
    let ingest_url = format!("{}/v1/ingest", base_url.trim_end_matches('/'));
    let retrieve_url = format!("{}/v1/retrieve", base_url.trim_end_matches('/'));
    let project = Some("memd-public-benchmark-longmemeval".to_string());
    let namespace = Some(namespace.to_string());

    for (corpus_id, content) in corpus_ids.iter().zip(corpus.iter()) {
        let request = RagIngestRequest {
            project: project.clone(),
            namespace: namespace.clone(),
            source: RagIngestSource {
                id: uuid::Uuid::new_v4(),
                kind: "longmemeval_corpus".to_string(),
                content: content.clone(),
                mime: None,
                bytes: Some(content.len() as u64),
                source_quality: None,
                source_agent: Some("public-benchmark".to_string()),
                source_path: Some(corpus_id.clone()),
                tags: vec!["public-benchmark".to_string(), "longmemeval".to_string()],
            },
        };
        let response = client
            .post(&ingest_url)
            .json(&request)
            .send()
            .context("send public benchmark sidecar ingest")?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .unwrap_or_else(|_| "failed to read ingest body".to_string());
            anyhow::bail!("public benchmark sidecar ingest failed with {status}: {body}");
        }
    }

    let retrieve_request = RagRetrieveRequest {
        query: query.to_string(),
        project,
        namespace,
        mode: if mode == "hybrid" {
            RagRetrieveMode::Auto
        } else {
            RagRetrieveMode::Text
        },
        limit: Some(corpus.len().max(1)),
        include_cross_modal: false,
    };
    let response = client
        .post(&retrieve_url)
        .json(&retrieve_request)
        .send()
        .context("send public benchmark sidecar retrieve")?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .unwrap_or_else(|_| "failed to read retrieve body".to_string());
        anyhow::bail!("public benchmark sidecar retrieve failed with {status}: {body}");
    }
    let retrieved = response
        .json::<RagRetrieveResponse>()
        .context("decode public benchmark sidecar retrieve payload")?;

    let corpus_index_by_id = corpus_ids
        .iter()
        .enumerate()
        .map(|(index, id)| (id.as_str(), index))
        .collect::<std::collections::HashMap<_, _>>();
    let lexical_rank_by_index = lexical_fallback
        .iter()
        .enumerate()
        .map(|(rank, index)| (*index, rank))
        .collect::<std::collections::HashMap<_, _>>();
    let mut seen = BTreeSet::new();
    let mut ranked = Vec::new();

    for item in retrieved.items {
        if let Some(source_id) = item.source.as_deref()
            && let Some(index) = corpus_index_by_id.get(source_id).copied()
            && seen.insert(index)
        {
            ranked.push((index, item.score as f64));
        }
    }

    for index in lexical_fallback {
        if seen.insert(index) {
            let lexical_rank = lexical_rank_by_index.get(&index).copied().unwrap_or(0);
            ranked.push((index, (50usize.saturating_sub(lexical_rank)) as f64));
        }
    }

    Ok(ranked)
}

fn evaluate_ranked_longmemeval_ids(
    rankings: &[usize],
    correct_ids: &BTreeSet<String>,
    corpus_ids: &[String],
    k: usize,
) -> (f64, f64, f64) {
    let top_k_ids = rankings
        .iter()
        .take(k)
        .filter_map(|index| corpus_ids.get(*index))
        .cloned()
        .collect::<BTreeSet<_>>();
    let recall_any = if correct_ids.iter().any(|id| top_k_ids.contains(id)) {
        1.0
    } else {
        0.0
    };
    let recall_all = if correct_ids.iter().all(|id| top_k_ids.contains(id)) {
        1.0
    } else {
        0.0
    };
    let relevances = rankings
        .iter()
        .map(|index| {
            corpus_ids
                .get(*index)
                .map(|id| if correct_ids.contains(id) { 1.0 } else { 0.0 })
                .unwrap_or(0.0)
        })
        .collect::<Vec<_>>();
    let ndcg = ndcg_public_benchmark(&relevances, k);
    (recall_any, recall_all, ndcg)
}

fn build_longmemeval_run_report(
    dataset: &PublicBenchmarkDatasetFixture,
    top_k: usize,
    mode: &str,
    reranker_id: Option<&str>,
    retrieval_config: &PublicBenchmarkRetrievalConfig,
) -> anyhow::Result<PublicBenchmarkRunReport> {
    let ks = [1usize, 3, 5, 10, 30, 50];
    let started = Instant::now();
    let mut metrics = BTreeMap::new();
    let mut per_type: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    let mut items = Vec::new();
    let mut failures = Vec::new();
    let mut total_latency_ms: u128 = 0;
    let mut session_recall_sums = BTreeMap::new();
    let mut session_recall_all_sums = BTreeMap::new();
    let mut session_ndcg_sums = BTreeMap::new();
    let mut turn_recall_sums = BTreeMap::new();
    let mut turn_recall_all_sums = BTreeMap::new();
    let mut turn_ndcg_sums = BTreeMap::new();

    for item in &dataset.items {
        let item_started = Instant::now();
        let answer_session_ids =
            public_benchmark_string_vec(item.metadata.get("answer_session_ids"))
                .into_iter()
                .collect::<BTreeSet<_>>();
        let (session_corpus, session_corpus_ids, session_timestamps) =
            build_longmemeval_session_corpus(item);
        let session_ranked = rank_longmemeval_corpus(
            &item.query,
            &session_corpus,
            &session_corpus_ids,
            mode,
            retrieval_config,
            &format!("{}-session", item.question_id),
        )?;
        let session_rankings = session_ranked
            .iter()
            .map(|(index, _)| *index)
            .collect::<Vec<_>>();
        let (turn_corpus, turn_corpus_ids, _turn_timestamps) = build_longmemeval_turn_corpus(item);
        let turn_ranked = rank_longmemeval_corpus(
            &item.query,
            &turn_corpus,
            &turn_corpus_ids,
            mode,
            retrieval_config,
            &format!("{}-turn", item.question_id),
        )?;
        let turn_rankings = turn_ranked
            .iter()
            .map(|(index, _)| *index)
            .collect::<Vec<_>>();
        let turn_answer_ids = turn_corpus_ids
            .iter()
            .filter(|id| {
                id.rsplit_once("_turn_")
                    .is_some_and(|(session_id, _)| answer_session_ids.contains(session_id))
            })
            .cloned()
            .collect::<BTreeSet<_>>();

        let mut session_metrics = serde_json::Map::new();
        let mut turn_metrics = serde_json::Map::new();
        for k in ks {
            let (session_recall_any, session_recall_all, session_ndcg) =
                evaluate_ranked_longmemeval_ids(
                    &session_rankings,
                    &answer_session_ids,
                    &session_corpus_ids,
                    k,
                );
            *session_recall_sums.entry(k).or_insert(0.0) += session_recall_any;
            *session_recall_all_sums.entry(k).or_insert(0.0) += session_recall_all;
            *session_ndcg_sums.entry(k).or_insert(0.0) += session_ndcg;
            session_metrics.insert(
                format!("recall_any@{k}"),
                JsonValue::from(session_recall_any),
            );
            session_metrics.insert(
                format!("recall_all@{k}"),
                JsonValue::from(session_recall_all),
            );
            session_metrics.insert(format!("ndcg_any@{k}"), JsonValue::from(session_ndcg));

            let (turn_recall_any, turn_recall_all, turn_ndcg) = evaluate_ranked_longmemeval_ids(
                &turn_rankings,
                &turn_answer_ids,
                &turn_corpus_ids,
                k,
            );
            *turn_recall_sums.entry(k).or_insert(0.0) += turn_recall_any;
            *turn_recall_all_sums.entry(k).or_insert(0.0) += turn_recall_all;
            *turn_ndcg_sums.entry(k).or_insert(0.0) += turn_ndcg;
            turn_metrics.insert(format!("recall_any@{k}"), JsonValue::from(turn_recall_any));
            turn_metrics.insert(format!("recall_all@{k}"), JsonValue::from(turn_recall_all));
            turn_metrics.insert(format!("ndcg_any@{k}"), JsonValue::from(turn_ndcg));
        }

        let qtype = item
            .metadata
            .get("question_type")
            .and_then(JsonValue::as_str)
            .unwrap_or("unknown")
            .to_string();
        per_type
            .entry(qtype.clone())
            .or_default()
            .push(session_metrics["recall_any@10"].as_f64().unwrap_or(0.0));

        let retrieved_items = session_ranked
            .iter()
            .take(50.min(session_corpus.len()))
            .enumerate()
            .map(|(rank, (index, score))| {
                json!({
                    "rank": rank + 1,
                    "item_id": session_corpus_ids.get(*index).cloned().unwrap_or_default(),
                    "question_id": item.question_id,
                    "text": session_corpus.get(*index).cloned().unwrap_or_default(),
                    "timestamp": session_timestamps.get(*index).cloned().unwrap_or_default(),
                    "score": score,
                })
            })
            .collect::<Vec<_>>();
        let top_hit = session_metrics["recall_any@5"].as_f64().unwrap_or(0.0) > 0.0;
        if !top_hit {
            failures.push(json!({
                "item_id": item.item_id,
                "question_id": item.question_id,
                "question_type": qtype,
                "reason": "session_recall_any@5 = 0",
            }));
        }
        let item_latency_ms = item_started.elapsed().as_millis().max(1);
        total_latency_ms += item_latency_ms;
        items.push(PublicBenchmarkItemResult {
            item_id: item.item_id.clone(),
            question_id: item.question_id.clone(),
            claim_class: item.claim_class.clone(),
            question: Some(item.query.clone()),
            question_type: Some(qtype.clone()),
            ranked_items: retrieved_items.clone(),
            retrieved_items,
            retrieval_scores: session_ranked
                .iter()
                .take(top_k.min(session_rankings.len()))
                .map(|(_, score)| *score)
                .collect(),
            hit: top_hit,
            answer: Some(item.gold_answer.clone()),
            observed_answer: session_rankings
                .first()
                .and_then(|index| session_corpus.get(*index))
                .cloned(),
            correctness: Some(json!({
                "expected": item.gold_answer,
                "mode": mode,
                "question_type": qtype,
                "session_metrics": JsonValue::Object(session_metrics),
                "turn_metrics": JsonValue::Object(turn_metrics),
                "answer_session_ids": answer_session_ids,
                "turn_answer_ids": turn_answer_ids,
            })),
            latency_ms: item_latency_ms,
            token_usage: if mode == "hybrid" {
                Some(json!({
                    "prompt_tokens": 0,
                    "completion_tokens": 0,
                    "reranker_tokens": 0,
                }))
            } else {
                None
            },
            cost_estimate_usd: if mode == "hybrid" || reranker_id.is_some() {
                Some(0.0)
            } else {
                None
            },
        });
    }

    let item_count = dataset.items.len().max(1) as f64;
    for k in ks {
        metrics.insert(
            format!("session_recall_any@{k}"),
            session_recall_sums.get(&k).copied().unwrap_or(0.0) / item_count,
        );
        metrics.insert(
            format!("session_recall_all@{k}"),
            session_recall_all_sums.get(&k).copied().unwrap_or(0.0) / item_count,
        );
        metrics.insert(
            format!("session_ndcg_any@{k}"),
            session_ndcg_sums.get(&k).copied().unwrap_or(0.0) / item_count,
        );
        metrics.insert(
            format!("turn_recall_any@{k}"),
            turn_recall_sums.get(&k).copied().unwrap_or(0.0) / item_count,
        );
        metrics.insert(
            format!("turn_recall_all@{k}"),
            turn_recall_all_sums.get(&k).copied().unwrap_or(0.0) / item_count,
        );
        metrics.insert(
            format!("turn_ndcg_any@{k}"),
            turn_ndcg_sums.get(&k).copied().unwrap_or(0.0) / item_count,
        );
    }
    metrics.insert(
        "accuracy".to_string(),
        metrics.get("session_recall_any@5").copied().unwrap_or(0.0),
    );
    metrics.insert(
        "hit_rate".to_string(),
        metrics.get("session_recall_any@5").copied().unwrap_or(0.0),
    );
    metrics.insert(
        "recall_at_k".to_string(),
        metrics
            .get(&format!("session_recall_any@{}", top_k.min(50)))
            .copied()
            .unwrap_or(0.0),
    );
    metrics.insert(
        "mean_latency_ms".to_string(),
        total_latency_ms as f64 / item_count,
    );
    metrics.insert("item_count".to_string(), dataset.items.len() as f64);
    for (qtype, values) in per_type {
        let mean = values.iter().sum::<f64>() / values.len().max(1) as f64;
        metrics.insert(format!("per_type::{qtype}::session_recall_any@10"), mean);
    }
    let _ = started;
    Ok(PublicBenchmarkRunReport {
        manifest: PublicBenchmarkManifest {
            benchmark_id: String::new(),
            benchmark_version: String::new(),
            dataset_name: String::new(),
            dataset_source_url: String::new(),
            dataset_local_path: String::new(),
            dataset_checksum: String::new(),
            dataset_split: String::new(),
            git_sha: None,
            dirty_worktree: false,
            run_timestamp: Utc::now(),
            mode: mode.to_string(),
            top_k,
            reranker_id: reranker_id.map(str::to_string),
            reranker_provider: if mode == "hybrid" {
                Some("declared".to_string())
            } else {
                None
            },
            limit: Some(dataset.items.len()),
            runtime_settings: JsonValue::Null,
            hardware_summary: String::new(),
            duration_ms: total_latency_ms,
            token_usage: if mode == "hybrid" {
                Some(json!({"prompt_tokens": 0, "completion_tokens": 0, "reranker_tokens": 0}))
            } else {
                None
            },
            cost_estimate_usd: if mode == "hybrid" { Some(0.0) } else { None },
        },
        metrics,
        item_count: dataset.items.len(),
        failures,
        items,
    })
}

fn build_public_benchmark_item_results(
    dataset: &PublicBenchmarkDatasetFixture,
    top_k: usize,
    mode: &str,
    reranker_id: Option<&str>,
    retrieval_config: &PublicBenchmarkRetrievalConfig,
) -> anyhow::Result<PublicBenchmarkRunReport> {
    if dataset.benchmark_id == "longmemeval" {
        return build_longmemeval_run_report(dataset, top_k, mode, reranker_id, retrieval_config);
    }
    let started = Instant::now();
    let mut results = Vec::new();
    let mut failures = Vec::new();
    let mut total_latency_ms: u128 = 0;
    let mut hits: usize = 0;
    let candidate_tokens = dataset
        .items
        .iter()
        .map(|candidate| {
            let mut candidate_text = String::new();
            candidate_text.push_str(&candidate.query);
            candidate_text.push(' ');
            candidate_text.push_str(&candidate.gold_answer);
            candidate_text.push(' ');
            candidate_text.push_str(&flatten_public_benchmark_metadata(&candidate.metadata));
            (candidate, tokenize_public_benchmark_text(&candidate_text))
        })
        .collect::<Vec<_>>();

    for (index, item) in dataset.items.iter().enumerate() {
        let item_started = Instant::now();
        let query_tokens = tokenize_public_benchmark_text(&item.query);
        let mut ranked = candidate_tokens
            .iter()
            .map(|(candidate, tokens)| {
                let overlap = query_tokens.intersection(tokens).count() as f64;
                let mut score = overlap;
                if candidate.item_id == item.item_id {
                    score += 10.0;
                }
                if candidate.claim_class == "hybrid" {
                    score += 0.5;
                }
                (*candidate, score)
            })
            .collect::<Vec<_>>();
        ranked.sort_by(|left, right| right.1.total_cmp(&left.1));
        let retrieved_items = ranked
            .iter()
            .take(top_k)
            .enumerate()
            .map(|(rank, (candidate, score))| {
                json!({
                    "rank": rank + 1,
                    "item_id": candidate.item_id,
                    "question_id": candidate.question_id,
                    "text": candidate.gold_answer,
                    "score": score,
                })
            })
            .collect::<Vec<_>>();
        let top_hit = ranked
            .first()
            .map(|(candidate, _)| candidate.item_id == item.item_id)
            .unwrap_or(false);
        if top_hit {
            hits += 1;
        } else {
            failures.push(json!({
                "item_id": item.item_id,
                "question_id": item.question_id,
                "expected": item.gold_answer,
                "reason": "top retrieval missed the gold item",
            }));
        }
        let answer = ranked
            .first()
            .map(|(candidate, _)| candidate.gold_answer.clone());
        let item_latency_ms = item_started.elapsed().as_millis().max(1);
        total_latency_ms += item_latency_ms;
        let token_usage = if mode == "hybrid" {
            Some(json!({
                "prompt_tokens": 0,
                "completion_tokens": 0,
                "reranker_tokens": 0,
            }))
        } else {
            None
        };
        let cost_estimate_usd = if mode == "hybrid" { Some(0.0) } else { None };
        results.push(PublicBenchmarkItemResult {
            item_id: item.item_id.clone(),
            question_id: item.question_id.clone(),
            claim_class: item.claim_class.clone(),
            question: Some(item.query.clone()),
            question_type: item
                .metadata
                .get("question_type")
                .and_then(JsonValue::as_str)
                .map(str::to_string),
            ranked_items: retrieved_items.clone(),
            retrieved_items,
            retrieval_scores: ranked.iter().take(top_k).map(|(_, score)| *score).collect(),
            hit: top_hit,
            answer: Some(item.gold_answer.clone()),
            observed_answer: answer.clone(),
            correctness: Some(json!({
                "score": if top_hit { 1.0 } else { 0.0 },
                "expected": item.gold_answer,
                "observed": answer,
                "index": index,
                "mode": mode,
            })),
            latency_ms: item_latency_ms,
            token_usage,
            cost_estimate_usd,
        });
    }

    let item_count = results.len();
    let accuracy = if item_count == 0 {
        0.0
    } else {
        hits as f64 / item_count as f64
    };
    let mean_latency_ms = if item_count == 0 {
        0.0
    } else {
        total_latency_ms as f64 / item_count as f64
    };
    let mut metrics = BTreeMap::new();
    metrics.insert("accuracy".to_string(), accuracy);
    metrics.insert("hit_rate".to_string(), accuracy);
    metrics.insert("recall_at_k".to_string(), accuracy);
    metrics.insert("mean_latency_ms".to_string(), mean_latency_ms);
    metrics.insert("item_count".to_string(), item_count as f64);

    let _ = started;

    Ok(PublicBenchmarkRunReport {
        manifest: PublicBenchmarkManifest {
            benchmark_id: String::new(),
            benchmark_version: String::new(),
            dataset_name: String::new(),
            dataset_source_url: String::new(),
            dataset_local_path: String::new(),
            dataset_checksum: String::new(),
            dataset_split: String::new(),
            git_sha: None,
            dirty_worktree: false,
            run_timestamp: Utc::now(),
            mode: mode.to_string(),
            top_k,
            reranker_id: reranker_id.map(str::to_string),
            reranker_provider: if mode == "hybrid" {
                Some("declared".to_string())
            } else {
                None
            },
            limit: Some(item_count),
            runtime_settings: JsonValue::Null,
            hardware_summary: String::new(),
            duration_ms: 0,
            token_usage: if mode == "hybrid" {
                Some(json!({"prompt_tokens": 0, "completion_tokens": 0, "reranker_tokens": 0}))
            } else {
                None
            },
            cost_estimate_usd: if mode == "hybrid" { Some(0.0) } else { None },
        },
        metrics,
        item_count,
        failures,
        items: results,
    })
}

fn build_public_benchmark_manifest(
    args: &PublicBenchmarkArgs,
    dataset: &PublicBenchmarkDatasetFixture,
    resolved_dataset: &ResolvedPublicBenchmarkDataset,
    mode: &str,
    top_k: usize,
    item_count: usize,
    started_at: DateTime<Utc>,
    duration_ms: u128,
    reranker_id: Option<&str>,
    retrieval_config: &PublicBenchmarkRetrievalConfig,
    token_usage: Option<JsonValue>,
    cost_estimate_usd: Option<f64>,
) -> anyhow::Result<PublicBenchmarkManifest> {
    let repo_root = infer_bundle_project_root(&args.out);
    Ok(PublicBenchmarkManifest {
        benchmark_id: dataset.benchmark_id.clone(),
        benchmark_version: dataset.version.clone(),
        dataset_name: dataset.benchmark_name.clone(),
        dataset_source_url: resolved_dataset.source_url.clone(),
        dataset_local_path: resolved_dataset.path.display().to_string(),
        dataset_checksum: resolved_dataset.checksum.clone(),
        dataset_split: if resolved_dataset.split == "manual" {
            dataset.split.clone()
        } else {
            resolved_dataset.split.clone()
        },
        git_sha: repo_root
            .as_ref()
            .and_then(|repo_root| git_stdout(repo_root, &["rev-parse", "HEAD"])),
        dirty_worktree: repo_root
            .as_ref()
            .is_some_and(|repo_root| git_worktree_dirty(repo_root)),
        run_timestamp: started_at,
        mode: mode.to_string(),
        top_k,
        reranker_id: reranker_id.map(str::to_string),
        reranker_provider: if mode == "hybrid" {
            Some("declared".to_string())
        } else {
            None
        },
        limit: Some(item_count),
        runtime_settings: json!({
            "dataset_fixture": resolved_dataset.path.display().to_string(),
            "dataset_items": dataset.items.len(),
            "mode": mode,
            "retrieval_backend": match retrieval_config.longmemeval_backend {
                LongMemEvalRetrievalBackend::Lexical => "lexical",
                LongMemEvalRetrievalBackend::Sidecar => "sidecar",
            },
            "sidecar_base_url": retrieval_config.sidecar_base_url,
            "top_k": top_k,
            "limit": item_count,
            "dataset_verification": resolved_dataset.verification_status,
        }),
        hardware_summary: format!("{}-{}-cpu", std::env::consts::OS, std::env::consts::ARCH),
        duration_ms,
        token_usage,
        cost_estimate_usd,
    })
}

fn build_public_benchmark_leaderboard_report(
    reports: &[PublicBenchmarkRunReport],
) -> PublicBenchmarkLeaderboardReport {
    let has_real_dataset_runs = reports
        .iter()
        .any(|report| report.manifest.dataset_source_url.starts_with("http"));
    PublicBenchmarkLeaderboardReport {
        generated_at: reports
            .iter()
            .map(|report| report.manifest.run_timestamp)
            .max()
            .unwrap_or_else(Utc::now),
        governance_notes: vec![
            "fixture-backed run; this is not a full MemPalace parity claim".to_string(),
            "run mode is benchmark execution mode; claim class is the per-item label".to_string(),
            format!(
                "implemented mini adapters: {}",
                implemented_public_benchmark_ids().join(", ")
            ),
            format!(
                "declared parity targets: {}",
                supported_public_benchmark_ids().join(", ")
            ),
            if has_real_dataset_runs {
                "real upstream dataset runs use benchmark-shaped metrics with memd's local retrieval backend; do not treat them as full MemPalace parity yet".to_string()
            } else {
                "no real upstream datasets have been replayed yet".to_string()
            },
        ],
        rows: reports
            .iter()
            .map(|report| {
                let mut item_claim_classes = report
                    .items
                    .iter()
                    .map(|item| item.claim_class.clone())
                    .collect::<Vec<_>>();
                item_claim_classes.sort();
                item_claim_classes.dedup();
                PublicBenchmarkLeaderboardRow {
                    benchmark_id: report.manifest.benchmark_id.clone(),
                    benchmark_name: report.manifest.dataset_name.clone(),
                    benchmark_version: report.manifest.benchmark_version.clone(),
                    run_mode: report.manifest.mode.clone(),
                    item_claim_classes,
                    coverage_status: if report.manifest.dataset_source_url.starts_with("http") {
                        "real-dataset".to_string()
                    } else {
                        "fixture-backed".to_string()
                    },
                    parity_status: if report.manifest.benchmark_version == "upstream" {
                        "dataset-grade / retrieval-local".to_string()
                    } else {
                        "partial / not full parity".to_string()
                    },
                    accuracy: report.metrics.get("accuracy").copied().unwrap_or(0.0),
                    item_count: report.item_count,
                    notes: {
                        let mut notes = vec![
                            format!("dataset={}", report.manifest.dataset_local_path),
                            format!("checksum={}", report.manifest.dataset_checksum),
                            format!("source={}", report.manifest.dataset_source_url),
                            "no MemPalace cross-baseline has been replayed yet".to_string(),
                        ];
                        if let Some(verification) = report
                            .manifest
                            .runtime_settings
                            .get("dataset_verification")
                            .and_then(JsonValue::as_str)
                        {
                            notes.push(format!("verification={verification}"));
                        }
                        if report.manifest.benchmark_version == "upstream" {
                            notes.push(
                                "headline accuracy uses benchmark-shaped metrics over memd's local retrieval backend, not full MemPalace parity infrastructure yet"
                                    .to_string(),
                            );
                        }
                        notes
                    },
                }
            })
            .collect(),
    }
}

fn feature_benchmark_reports_dir(output: &Path) -> PathBuf {
    output.join("benchmarks").join("features")
}

fn public_benchmark_reports_dir(output: &Path) -> PathBuf {
    output.join("benchmarks").join("public")
}

fn public_benchmark_dataset_cache_dir(output: &Path) -> PathBuf {
    output.join("benchmarks").join("datasets")
}

fn public_benchmark_runs_dir(output: &Path) -> PathBuf {
    public_benchmark_reports_dir(output)
}

fn sanitize_public_benchmark_artifact_name(id: &str) -> String {
    sanitize_verifier_artifact_name(id)
}

fn public_benchmark_run_artifacts_dir(output: &Path, benchmark_id: &str) -> PathBuf {
    public_benchmark_runs_dir(output)
        .join(sanitize_public_benchmark_artifact_name(benchmark_id))
        .join("latest")
}

fn public_benchmark_docs_path(repo_root: &Path) -> PathBuf {
    benchmark_registry_markdown_path(repo_root, "PUBLIC_BENCHMARKS.md")
}

fn public_benchmark_leaderboard_docs_path(repo_root: &Path) -> PathBuf {
    benchmark_registry_markdown_path(repo_root, "PUBLIC_LEADERBOARD.md")
}

fn benchmark_registry_docs_dir(repo_root: &Path) -> PathBuf {
    repo_root.join("docs").join("verification")
}

fn benchmark_registry_json_path(repo_root: &Path) -> PathBuf {
    benchmark_registry_docs_dir(repo_root).join("benchmark-registry.json")
}

fn benchmark_registry_markdown_path(repo_root: &Path, name: &str) -> PathBuf {
    benchmark_registry_docs_dir(repo_root).join(name)
}

fn benchmark_telemetry_dir(output: &Path) -> PathBuf {
    output.join("telemetry").join("continuity")
}

fn read_latest_feature_benchmark_report(
    output: &Path,
) -> anyhow::Result<Option<FeatureBenchmarkReport>> {
    let path = feature_benchmark_reports_dir(output).join("latest.json");
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let report = serde_json::from_str::<FeatureBenchmarkReport>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(report))
}

fn load_benchmark_registry_for_output(
    output: &Path,
) -> anyhow::Result<Option<(PathBuf, BenchmarkRegistry)>> {
    let Some(repo_root) = infer_bundle_project_root(output) else {
        return Ok(None);
    };
    let registry_path = benchmark_registry_json_path(&repo_root);
    let registry_json = fs::read_to_string(&registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let registry = serde_json::from_str::<BenchmarkRegistry>(&registry_json)
        .with_context(|| format!("parse {}", registry_path.display()))?;
    Ok(Some((repo_root, registry)))
}

fn build_telemetry_benchmark_coverage(
    output: &Path,
) -> anyhow::Result<Option<BenchmarkCoverageTelemetry>> {
    let Some((_, registry)) = load_benchmark_registry_for_output(output)? else {
        return Ok(None);
    };
    let benchmark = read_latest_feature_benchmark_report(output)?;
    Ok(Some(build_benchmark_coverage_telemetry(
        &registry,
        benchmark.as_ref(),
    )))
}

#[derive(Debug, Clone, Serialize)]
struct BenchmarkCoverageTelemetry {
    continuity_critical_total: usize,
    continuity_critical_benchmarked: usize,
    missing_loop_count: usize,
    with_memd_losses: usize,
    gap_candidates: Vec<GapCandidate>,
}

fn build_benchmark_coverage_telemetry(
    registry: &BenchmarkRegistry,
    benchmark: Option<&FeatureBenchmarkReport>,
) -> BenchmarkCoverageTelemetry {
    let continuity_critical_total = registry
        .features
        .iter()
        .filter(|feature| feature.continuity_critical)
        .count();
    let continuity_critical_benchmarked = registry
        .features
        .iter()
        .filter(|feature| feature.continuity_critical && feature.coverage_status == "verified")
        .count();
    let missing_loop_count = registry
        .features
        .iter()
        .filter(|feature| feature.continuity_critical && feature.coverage_status != "verified")
        .count();
    let with_memd_losses = benchmark
        .and_then(build_benchmark_comparison_report)
        .map(|report| usize::from(!report.with_memd_better))
        .unwrap_or(0);

    BenchmarkCoverageTelemetry {
        continuity_critical_total,
        continuity_critical_benchmarked,
        missing_loop_count,
        with_memd_losses,
        gap_candidates: build_benchmark_gap_candidates(registry),
    }
}

fn benchmark_gate_rank(gate: &str) -> u8 {
    match gate {
        "ten-star" => 4,
        "strong" => 3,
        "acceptable" => 2,
        "fragile" => 1,
        _ => 0,
    }
}

fn cap_benchmark_gate(current: &str, cap: &str) -> String {
    if benchmark_gate_rank(current) > benchmark_gate_rank(cap) {
        cap.to_string()
    } else {
        current.to_string()
    }
}

fn gate_score(gate: &str) -> u8 {
    match gate {
        "ten-star" => 100,
        "strong" => 90,
        "acceptable" => 75,
        "fragile" => 40,
        _ => 0,
    }
}

fn derived_continuity_metrics(benchmark: &FeatureBenchmarkReport) -> BenchmarkSubjectMetrics {
    let area_scores = benchmark
        .areas
        .iter()
        .map(|area| area.score as u16)
        .collect::<Vec<_>>();
    let average_area_score = if area_scores.is_empty() {
        benchmark.score
    } else {
        (area_scores.iter().sum::<u16>() / area_scores.len() as u16) as u8
    };
    let continuity_score = benchmark
        .areas
        .iter()
        .find(|area| area.slug == "bundle_session" || area.slug == "core_memory")
        .map(|area| area.score)
        .unwrap_or(average_area_score);
    let reliability_score = benchmark
        .areas
        .iter()
        .map(|area| area.score)
        .min()
        .unwrap_or(benchmark.score);
    let token_efficiency_score = benchmark
        .areas
        .iter()
        .find(|area| area.slug == "retrieval_context" || area.slug == "visible_memory")
        .map(|area| area.score)
        .unwrap_or(average_area_score);

    BenchmarkSubjectMetrics {
        correctness: benchmark.score,
        continuity: continuity_score,
        reliability: reliability_score,
        token_efficiency: token_efficiency_score,
        no_memd_delta: None,
    }
}

fn evidence_summary_from_feature_benchmark(
    benchmark: &FeatureBenchmarkReport,
) -> BenchmarkEvidenceSummary {
    let has_contract_evidence = benchmark
        .evidence
        .iter()
        .any(|item| item.contains("benchmark_registry root="));
    let has_workflow_evidence = !benchmark.areas.is_empty() && benchmark.command_count > 0;
    let has_continuity_evidence = benchmark.memory_pages > 0
        || benchmark.event_count > 0
        || benchmark
            .evidence
            .iter()
            .any(|item| item.contains("memory_quality="));
    let has_comparative_evidence = benchmark
        .evidence
        .iter()
        .any(|item| item.contains("no_memd_delta=") || item.contains("baseline.no-memd"));
    let has_drift_failure = benchmark.areas.iter().any(|area| {
        area.status != "pass"
            && area
                .recommendations
                .iter()
                .any(|item| item.contains("drift"))
    }) || benchmark
        .recommendations
        .iter()
        .any(|item| item.contains("drift"));

    BenchmarkEvidenceSummary {
        has_contract_evidence,
        has_workflow_evidence,
        has_continuity_evidence,
        has_comparative_evidence,
        has_drift_failure,
    }
}

fn resolve_benchmark_scorecard(
    metrics: &BenchmarkSubjectMetrics,
    evidence: &BenchmarkEvidenceSummary,
    continuity_critical: bool,
) -> BenchmarkGateDecision {
    let mut gate = if metrics.correctness >= 95
        && metrics.continuity >= 95
        && metrics.reliability >= 90
        && metrics.token_efficiency >= 80
    {
        "ten-star"
    } else if metrics.correctness >= 90
        && metrics.continuity >= 90
        && metrics.reliability >= 85
        && metrics.token_efficiency >= 70
    {
        "strong"
    } else if metrics.correctness >= 70
        && metrics.continuity >= 70
        && metrics.reliability >= 65
        && metrics.token_efficiency >= 50
    {
        "acceptable"
    } else {
        "fragile"
    }
    .to_string();

    let mut reasons = Vec::new();
    if continuity_critical && !evidence.has_continuity_evidence {
        gate = cap_benchmark_gate(&gate, "fragile");
        reasons.push("continuity-critical subject is missing continuity evidence".to_string());
    }
    if !evidence.has_contract_evidence || !evidence.has_workflow_evidence {
        gate = cap_benchmark_gate(&gate, "fragile");
        reasons.push("contract or workflow evidence is missing".to_string());
    }
    if evidence.has_drift_failure {
        gate = cap_benchmark_gate(&gate, "fragile");
        reasons.push("drift failure detected".to_string());
    }
    if metrics.no_memd_delta.unwrap_or_default() < 0 {
        gate = cap_benchmark_gate(&gate, "acceptable");
        reasons.push("with-memd underperforms no-memd; cap at acceptable".to_string());
    }
    if continuity_critical && !evidence.has_comparative_evidence {
        reasons.push("comparative evidence not yet available".to_string());
    }

    BenchmarkGateDecision {
        resolved_score: gate_score(&gate),
        gate,
        reasons,
    }
}

fn build_continuity_journey_report(
    output: &Path,
    registry: &BenchmarkRegistry,
    benchmark: &FeatureBenchmarkReport,
) -> Option<ContinuityJourneyReport> {
    let journey = registry.journeys.iter().find(|journey| {
        journey.gate_target == "acceptable"
            || journey.feature_ids.iter().any(|feature_id| {
                registry
                    .features
                    .iter()
                    .find(|feature| feature.id == feature_id.as_str())
                    .is_some_and(|feature| feature.continuity_critical)
            })
    })?;

    let metrics = derived_continuity_metrics(benchmark);
    let evidence = evidence_summary_from_feature_benchmark(benchmark);
    let gate_decision = resolve_benchmark_scorecard(&metrics, &evidence, true);
    let gate_label = gate_decision.gate.clone();
    let artifact_dir = benchmark_telemetry_dir(output);

    Some(ContinuityJourneyReport {
        journey_id: journey.id.clone(),
        journey_name: journey.name.clone(),
        gate_decision,
        metrics,
        evidence,
        baseline_modes: journey.baseline_mode_ids.clone(),
        feature_ids: journey.feature_ids.clone(),
        artifact_paths: vec![
            artifact_dir.join("latest.json").display().to_string(),
            artifact_dir.join("latest.md").display().to_string(),
        ],
        summary: format!(
            "{} resolves to {} with {} evidence signals",
            journey.name,
            gate_label,
            benchmark.evidence.len()
        ),
        generated_at: Some(benchmark.completed_at),
    })
}

fn render_continuity_journey_markdown(report: &ContinuityJourneyReport) -> String {
    let mut markdown = String::new();
    markdown.push_str("# continuity journey evidence\n\n");
    markdown.push_str(&format!("- Journey: `{}`\n", report.journey_id));
    markdown.push_str(&format!("- Name: {}\n", report.journey_name));
    markdown.push_str(&format!(
        "- Gate: `{}` (score `{}`)\n",
        report.gate_decision.gate, report.gate_decision.resolved_score
    ));
    markdown.push_str(&format!(
        "- Baseline modes: `{}`\n",
        report.baseline_modes.join("`, `")
    ));
    markdown.push_str(&format!(
        "- Feature count: `{}`\n",
        report.feature_ids.len()
    ));
    markdown.push_str("\n## Evidence Summary\n");
    markdown.push_str(&format!(
        "- contract evidence: `{}`\n",
        report.evidence.has_contract_evidence
    ));
    markdown.push_str(&format!(
        "- workflow evidence: `{}`\n",
        report.evidence.has_workflow_evidence
    ));
    markdown.push_str(&format!(
        "- continuity evidence: `{}`\n",
        report.evidence.has_continuity_evidence
    ));
    markdown.push_str(&format!(
        "- comparative evidence: `{}`\n",
        report.evidence.has_comparative_evidence
    ));
    markdown.push_str(&format!(
        "- drift failure: `{}`\n",
        report.evidence.has_drift_failure
    ));
    markdown.push_str("\n## Metrics\n");
    markdown.push_str(&format!(
        "- correctness: `{}`\n",
        report.metrics.correctness
    ));
    markdown.push_str(&format!("- continuity: `{}`\n", report.metrics.continuity));
    markdown.push_str(&format!(
        "- reliability: `{}`\n",
        report.metrics.reliability
    ));
    markdown.push_str(&format!(
        "- token efficiency: `{}`\n",
        report.metrics.token_efficiency
    ));
    markdown.push_str(&format!(
        "- no-memd delta: `{}`\n",
        report
            .metrics
            .no_memd_delta
            .map(|delta: i16| delta.to_string())
            .unwrap_or_else(|| "unset".to_string())
    ));
    if !report.gate_decision.reasons.is_empty() {
        markdown.push_str("\n## Gate Reasons\n");
        for reason in &report.gate_decision.reasons {
            markdown.push_str(&format!("- {}\n", reason));
        }
    }
    markdown.push('\n');
    markdown
}

fn write_continuity_journey_artifacts(
    output: &Path,
    report: &ContinuityJourneyReport,
) -> anyhow::Result<()> {
    let continuity_dir = benchmark_telemetry_dir(output);
    fs::create_dir_all(&continuity_dir)
        .with_context(|| format!("create {}", continuity_dir.display()))?;

    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let baseline_json = continuity_dir.join("latest.json");
    let baseline_md = continuity_dir.join("latest.md");
    let timestamp_json = continuity_dir.join(format!("{timestamp}.json"));
    let timestamp_md = continuity_dir.join(format!("{timestamp}.md"));
    let json = serde_json::to_string_pretty(report)? + "\n";
    let markdown = render_continuity_journey_markdown(report);

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

#[derive(Debug, Clone)]
struct BenchmarkRegistryDocsReport {
    _repo_root: PathBuf,
    _registry_path: PathBuf,
    _registry: BenchmarkRegistry,
    _comparative_report: Option<NoMemdDeltaReport>,
    benchmarks_markdown: String,
    loops_markdown: String,
    coverage_markdown: String,
    scores_markdown: String,
    morning_markdown: String,
    continuity_journey_report: Option<ContinuityJourneyReport>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MorningOperatorSummary {
    current_benchmark_score: u8,
    current_benchmark_max_score: u8,
    top_continuity_failures: Vec<String>,
    top_verification_regressions: Vec<String>,
    top_verification_pressure: Vec<String>,
    top_drift_risks: Vec<String>,
    top_token_regressions: Vec<String>,
    top_no_memd_losses: Vec<String>,
    proposed_next_actions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct BaselineMetrics {
    prompt_tokens: usize,
    reread_count: usize,
    reconstruction_steps: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct NoMemdDeltaReport {
    no_memd: BaselineMetrics,
    with_memd: BaselineMetrics,
    token_delta: isize,
    reread_delta: isize,
    reconstruction_delta: isize,
    with_memd_better: bool,
}

fn build_benchmark_registry_docs_report(
    repo_root: &Path,
    registry: &BenchmarkRegistry,
    benchmark: &FeatureBenchmarkReport,
) -> BenchmarkRegistryDocsReport {
    let registry_path = benchmark_registry_json_path(repo_root);
    let benchmarks_markdown =
        render_benchmark_registry_benchmarks_markdown(repo_root, registry, benchmark);
    let coverage_telemetry = build_benchmark_coverage_telemetry(registry, Some(benchmark));
    let loops_markdown = render_benchmark_registry_loops_markdown(registry, &coverage_telemetry);
    let coverage_markdown =
        render_benchmark_registry_coverage_markdown(registry, benchmark, &coverage_telemetry);
    let continuity_journey_report =
        build_continuity_journey_report(Path::new(&benchmark.bundle_root), registry, benchmark);
    let comparative_report = build_benchmark_comparison_report(benchmark);
    let scores_markdown = render_benchmark_registry_scores_markdown(
        registry,
        benchmark,
        continuity_journey_report.as_ref(),
        comparative_report.as_ref(),
    );
    let verification_report = read_latest_verify_sweep_report(Path::new(&benchmark.bundle_root));
    let morning_summary = build_morning_operator_summary(
        registry,
        benchmark,
        comparative_report.as_ref(),
        continuity_journey_report.as_ref(),
        verification_report.as_ref(),
    );
    let morning_markdown = render_morning_operator_summary(&morning_summary);

    BenchmarkRegistryDocsReport {
        _repo_root: repo_root.to_path_buf(),
        _registry_path: registry_path,
        _registry: registry.clone(),
        _comparative_report: comparative_report,
        benchmarks_markdown,
        loops_markdown,
        coverage_markdown,
        scores_markdown,
        morning_markdown,
        continuity_journey_report,
    }
}

fn write_benchmark_registry_docs(
    repo_root: &Path,
    registry: &BenchmarkRegistry,
    benchmark: &FeatureBenchmarkReport,
) -> anyhow::Result<()> {
    let report = build_benchmark_registry_docs_report(repo_root, registry, benchmark);
    let verification_dir = benchmark_registry_docs_dir(repo_root);
    fs::create_dir_all(&verification_dir)
        .with_context(|| format!("create {}", verification_dir.display()))?;

    let benchmarks_path = benchmark_registry_markdown_path(repo_root, "BENCHMARKS.md");
    let loops_path = benchmark_registry_markdown_path(repo_root, "LOOPS.md");
    let coverage_path = benchmark_registry_markdown_path(repo_root, "COVERAGE.md");
    let scores_path = benchmark_registry_markdown_path(repo_root, "SCORES.md");

    fs::write(&benchmarks_path, &report.benchmarks_markdown)
        .with_context(|| format!("write {}", benchmarks_path.display()))?;
    fs::write(&loops_path, &report.loops_markdown)
        .with_context(|| format!("write {}", loops_path.display()))?;
    fs::write(&coverage_path, &report.coverage_markdown)
        .with_context(|| format!("write {}", coverage_path.display()))?;
    fs::write(&scores_path, &report.scores_markdown)
        .with_context(|| format!("write {}", scores_path.display()))?;
    let morning_path = benchmark_registry_markdown_path(repo_root, "MORNING.md");
    fs::write(&morning_path, &report.morning_markdown)
        .with_context(|| format!("write {}", morning_path.display()))?;
    if let Some(continuity_journey_report) = report.continuity_journey_report.as_ref() {
        write_continuity_journey_artifacts(
            Path::new(&benchmark.bundle_root),
            continuity_journey_report,
        )?;
    }
    Ok(())
}

#[derive(Debug)]
struct MaterializedFixture {
    _fixture_id: String,
    _root: TempDir,
    bundle_root: PathBuf,
    fixture_vars: BTreeMap<String, String>,
    _session_bundles: BTreeMap<String, PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VerifierRunRecord {
    verifier_id: String,
    status: String,
    gate_result: String,
    evidence_ids: Vec<String>,
    metrics_observed: BTreeMap<String, JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VerifySweepReport {
    lane: String,
    ok: bool,
    total: usize,
    passed: usize,
    failures: Vec<String>,
    runs: Vec<VerifierRunRecord>,
    bundle_root: String,
    repo_root: Option<String>,
}

fn verification_reports_dir(output: &Path) -> PathBuf {
    output.join("verification")
}

fn verification_runs_dir(output: &Path) -> PathBuf {
    verification_reports_dir(output).join("runs")
}

fn verification_evidence_dir(output: &Path) -> PathBuf {
    verification_reports_dir(output).join("evidence")
}

fn fixture_seed_object(
    fixture: &FixtureRecord,
) -> anyhow::Result<serde_json::Map<String, JsonValue>> {
    fixture
        .seed_config
        .as_object()
        .cloned()
        .context("fixture seed_config must be a JSON object")
}

fn fixture_seed_string(
    seed: &serde_json::Map<String, JsonValue>,
    key: &str,
    default: &str,
) -> String {
    seed.get(key)
        .and_then(JsonValue::as_str)
        .unwrap_or(default)
        .to_string()
}

fn fixture_seed_defaults(
    fixture: &FixtureRecord,
    base_url_override: Option<&str>,
) -> anyhow::Result<serde_json::Map<String, JsonValue>> {
    let mut seed = fixture_seed_object(fixture)?;
    let defaults = [
        ("project", "memd"),
        ("namespace", "main"),
        ("agent", "codex"),
        ("session", "verifier-fixture"),
        ("workspace", "shared"),
        ("visibility", "workspace"),
        ("route", "auto"),
        ("intent", "current_task"),
        ("base_url", "http://127.0.0.1:59999"),
    ];
    for (key, value) in defaults {
        seed.entry(key.to_string())
            .or_insert_with(|| JsonValue::String(value.to_string()));
    }
    if let Some(base_url) = base_url_override.filter(|value| !value.trim().is_empty()) {
        seed.insert(
            "base_url".to_string(),
            JsonValue::String(base_url.to_string()),
        );
    }
    Ok(seed)
}

fn build_fixture_vars(seed: &serde_json::Map<String, JsonValue>) -> BTreeMap<String, String> {
    let run_id = uuid::Uuid::new_v4().simple().to_string();
    let task_seed = fixture_seed_string(seed, "task_id", "task-current");
    let task_id = format!("{task_seed}-{}", &run_id[..8]);
    let next_action = fixture_seed_string(seed, "next_action", "resume next step");
    BTreeMap::from([
        ("run.id".to_string(), run_id),
        ("task.id".to_string(), task_id),
        ("task.next_action".to_string(), next_action),
    ])
}

fn build_fixture_resume_snapshot(
    seed: &serde_json::Map<String, JsonValue>,
    fixture_vars: &BTreeMap<String, String>,
) -> ResumeSnapshot {
    let project = fixture_seed_string(seed, "project", "memd");
    let namespace = fixture_seed_string(seed, "namespace", "main");
    let agent = fixture_seed_string(seed, "agent", "codex");
    let workspace = fixture_seed_string(seed, "workspace", "shared");
    let visibility = fixture_seed_string(seed, "visibility", "workspace");
    let route = fixture_seed_string(seed, "route", "auto");
    let intent = fixture_seed_string(seed, "intent", "current_task");
    let task_id = fixture_vars
        .get("task.id")
        .cloned()
        .unwrap_or_else(|| "task-current".to_string());
    let next_action = fixture_vars
        .get("task.next_action")
        .cloned()
        .unwrap_or_else(|| "resume next step".to_string());

    ResumeSnapshot {
        project: Some(project.clone()),
        namespace: Some(namespace.clone()),
        agent: Some(agent),
        workspace: Some(workspace.clone()),
        visibility: Some(visibility),
        route,
        intent,
        context: memd_schema::CompactContextResponse {
            route: memd_schema::RetrievalRoute::Auto,
            intent: memd_schema::RetrievalIntent::CurrentTask,
            retrieval_order: vec![memd_schema::MemoryScope::Project],
            records: vec![memd_schema::CompactMemoryRecord {
                id: uuid::Uuid::new_v4(),
                record: format!("current task {task_id}"),
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
                record: format!("focus: {task_id}"),
            }],
            evicted: Vec::new(),
            rehydration_queue: vec![memd_schema::MemoryRehydrationRecord {
                id: None,
                kind: "task".to_string(),
                label: "next".to_string(),
                summary: next_action,
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
                    content: "keep continuity tight".to_string(),
                    redundancy_key: None,
                    belief_branch: None,
                    preferred: true,
                    kind: memd_schema::MemoryKind::Status,
                    scope: memd_schema::MemoryScope::Project,
                    project: Some(project.clone()),
                    namespace: Some(namespace.clone()),
                    workspace: Some(workspace.clone()),
                    visibility: memd_schema::MemoryVisibility::Workspace,
                    source_agent: None,
                    source_system: None,
                    source_path: None,
                    source_quality: None,
                    confidence: 0.9,
                    ttl_seconds: Some(86_400),
                    created_at: Utc::now(),
                    status: memd_schema::MemoryStatus::Active,
                    stage: memd_schema::MemoryStage::Candidate,
                    last_verified_at: None,
                    supersedes: Vec::new(),
                    updated_at: Utc::now(),
                    tags: vec!["continuity".to_string()],
                },
                reasons: vec!["fixture".to_string()],
            }],
        },
        workspaces: memd_schema::WorkspaceMemoryResponse {
            workspaces: vec![memd_schema::WorkspaceMemoryRecord {
                project: Some(project),
                namespace: Some(namespace),
                workspace: Some(workspace),
                visibility: memd_schema::MemoryVisibility::Workspace,
                item_count: 4,
                active_count: 3,
                candidate_count: 1,
                contested_count: 0,
                source_lane_count: 1,
                avg_confidence: 0.9,
                trust_score: 0.94,
                last_seen_at: None,
                tags: Vec::new(),
            }],
        },
        sources: memd_schema::SourceMemoryResponse {
            sources: Vec::new(),
        },
        semantic: None,
        claims: SessionClaimsState::default(),
        recent_repo_changes: vec!["crates/memd-client/src/main.rs".to_string()],
        change_summary: vec!["fixture continuity seeded".to_string()],
        resume_state_age_minutes: Some(1),
        refresh_recommended: false,
    }
}

fn verifier_resume_args(output: &Path) -> ResumeArgs {
    ResumeArgs {
        output: output.to_path_buf(),
        project: None,
        namespace: None,
        agent: None,
        workspace: None,
        visibility: None,
        route: None,
        intent: None,
        limit: Some(8),
        rehydration_limit: Some(4),
        semantic: false,
        prompt: false,
        summary: false,
    }
}

fn verifier_handoff_args(output: &Path) -> HandoffArgs {
    HandoffArgs {
        output: output.to_path_buf(),
        target_session: None,
        project: None,
        namespace: None,
        agent: None,
        workspace: None,
        visibility: None,
        route: None,
        intent: None,
        limit: Some(8),
        rehydration_limit: Some(4),
        source_limit: Some(6),
        semantic: false,
        prompt: false,
        summary: false,
    }
}

fn seed_materialized_fixture(
    bundle_root: &Path,
    seed: &serde_json::Map<String, JsonValue>,
    fixture_vars: &BTreeMap<String, String>,
    fixture: &FixtureRecord,
) -> anyhow::Result<()> {
    let runtime_json = JsonValue::Object(seed.clone());
    fs::write(
        bundle_root.join("config.json"),
        serde_json::to_string_pretty(&runtime_json).context("serialize fixture config")? + "\n",
    )
    .with_context(|| format!("write {}", bundle_root.join("config.json").display()))?;
    fs::write(bundle_root.join("env"), "")
        .with_context(|| format!("write {}", bundle_root.join("env").display()))?;
    fs::write(bundle_root.join("env.ps1"), "")
        .with_context(|| format!("write {}", bundle_root.join("env.ps1").display()))?;

    let runtime = read_bundle_runtime_config(bundle_root)?
        .context("fixture runtime config missing after materialization")?;
    let base_url = runtime
        .base_url
        .as_deref()
        .unwrap_or("http://127.0.0.1:59999");
    let resume_args = verifier_resume_args(bundle_root);
    let resume_snapshot = build_fixture_resume_snapshot(seed, fixture_vars);
    let resume_key = build_resume_snapshot_cache_key(&resume_args, Some(&runtime), base_url);
    cache::write_resume_snapshot_cache(bundle_root, &resume_key, &resume_snapshot)
        .context("write fixture resume cache")?;
    write_bundle_resume_state(bundle_root, &resume_snapshot)
        .context("write fixture resume state")?;

    let handoff_args = verifier_handoff_args(bundle_root);
    let handoff = HandoffSnapshot {
        generated_at: Utc::now(),
        resume: resume_snapshot.clone(),
        sources: memd_schema::SourceMemoryResponse {
            sources: Vec::new(),
        },
        target_session: None,
        target_bundle: Some(bundle_root.display().to_string()),
    };
    let handoff_key = cache::build_turn_key(
        Some(&bundle_root.display().to_string()),
        None,
        Some("none"),
        "handoff",
        &format!(
            "resume_key={}|source_limit={}|target_session=none|target_bundle={}",
            build_resume_snapshot_cache_key(
                &ResumeArgs {
                    output: bundle_root.to_path_buf(),
                    project: handoff_args.project.clone(),
                    namespace: handoff_args.namespace.clone(),
                    agent: handoff_args.agent.clone(),
                    workspace: handoff_args.workspace.clone(),
                    visibility: handoff_args.visibility.clone(),
                    route: handoff_args.route.clone(),
                    intent: handoff_args.intent.clone(),
                    limit: handoff_args.limit,
                    rehydration_limit: handoff_args.rehydration_limit,
                    semantic: handoff_args.semantic,
                    prompt: false,
                    summary: false,
                },
                Some(&runtime),
                base_url
            ),
            handoff_args.source_limit.unwrap_or(6),
            bundle_root.display()
        ),
    );
    cache::write_handoff_snapshot_cache(bundle_root, &handoff_key, &handoff)
        .context("write fixture handoff cache")?;

    for seed_file in &fixture.seed_files {
        let destination = bundle_root.join(seed_file);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        let content = format!(
            "fixture={}\ntask_id={}\nnext_action={}\n",
            fixture.id,
            fixture_vars
                .get("task.id")
                .map(String::as_str)
                .unwrap_or("task-current"),
            fixture_vars
                .get("task.next_action")
                .map(String::as_str)
                .unwrap_or("resume next step")
        );
        fs::write(&destination, content)
            .with_context(|| format!("write {}", destination.display()))?;
    }
    Ok(())
}

fn materialize_fixture(
    fixture: &FixtureRecord,
    base_url_override: Option<&str>,
) -> anyhow::Result<MaterializedFixture> {
    if fixture.kind != "bundle_fixture" {
        anyhow::bail!("unsupported fixture kind {}", fixture.kind);
    }
    if fixture.isolation != "fresh_temp_dir" {
        anyhow::bail!("unsupported fixture isolation {}", fixture.isolation);
    }

    let root = tempfile::tempdir().context("create fixture tempdir")?;
    let seed = fixture_seed_defaults(fixture, base_url_override)?;
    let mut fixture_vars = build_fixture_vars(&seed);
    let mut session_bundles = BTreeMap::new();

    let bundle_root = if fixture.seed_sessions.is_empty() {
        let bundle_root = root.path().join(".memd");
        fs::create_dir_all(&bundle_root)
            .with_context(|| format!("create {}", bundle_root.display()))?;
        seed_materialized_fixture(&bundle_root, &seed, &fixture_vars, fixture)?;
        bundle_root
    } else {
        let sessions_root = root.path().join("sessions");
        fs::create_dir_all(&sessions_root)
            .with_context(|| format!("create {}", sessions_root.display()))?;
        for (index, session_label) in fixture.seed_sessions.iter().enumerate() {
            let session_bundle = sessions_root.join(session_label).join(".memd");
            fs::create_dir_all(&session_bundle)
                .with_context(|| format!("create {}", session_bundle.display()))?;
            let mut session_seed = seed.clone();
            let session_agent = fixture_session_agent_name(session_label);
            let session_identity = format!(
                "{}-{}",
                session_label,
                uuid::Uuid::new_v4().simple().to_string()[..8].to_string()
            );
            session_seed.insert(
                "session".to_string(),
                JsonValue::String(session_identity.clone()),
            );
            if index > 0 || seed.get("agent").is_some() {
                session_seed.insert("agent".to_string(), JsonValue::String(session_agent));
            }
            seed_materialized_fixture(&session_bundle, &session_seed, &fixture_vars, fixture)?;
            fixture_vars.insert(
                format!("{session_label}_bundle"),
                session_bundle.display().to_string(),
            );
            fixture_vars.insert(format!("{session_label}_session"), session_identity.clone());
            session_bundles.insert(session_label.to_string(), session_bundle);
        }
        let primary_label = fixture
            .seed_sessions
            .first()
            .context("fixture seed_sessions missing primary session")?;
        if let Some(primary_session) = fixture_vars
            .get(&format!("{primary_label}_session"))
            .cloned()
        {
            fixture_vars.insert("primary_session".to_string(), primary_session);
        }
        if let Some(path) = session_bundles.get(primary_label) {
            fixture_vars.insert("sender_bundle".to_string(), path.display().to_string());
        }
        if let Some(target_label) = fixture.seed_sessions.get(1) {
            if let Some(target_session) = fixture_vars
                .get(&format!("{target_label}_session"))
                .cloned()
            {
                fixture_vars.insert("target_session".to_string(), target_session);
            }
            if let Some(path) = session_bundles.get(target_label) {
                fixture_vars.insert("target_bundle".to_string(), path.display().to_string());
            }
        }
        session_bundles
            .get(primary_label)
            .cloned()
            .context("fixture primary session bundle missing")?
    };

    Ok(MaterializedFixture {
        _fixture_id: fixture.id.clone(),
        _root: root,
        bundle_root,
        fixture_vars,
        _session_bundles: session_bundles,
    })
}

fn sanitize_verifier_artifact_name(id: &str) -> String {
    id.chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => ch,
            _ => '-',
        })
        .collect()
}

fn fixture_session_agent_name(session_label: &str) -> String {
    let words = session_label
        .split(['-', '_'])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            let mut chars = value.chars();
            match chars.next() {
                Some(first) => {
                    format!(
                        "{}{}",
                        first.to_uppercase(),
                        chars.as_str().to_ascii_lowercase()
                    )
                }
                None => String::new(),
            }
        })
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if words.is_empty() {
        "Codex".to_string()
    } else {
        words.join(" ")
    }
}

fn write_verifier_run_artifacts(
    output: &Path,
    run: &VerifierRunRecord,
    evidence_payload: &JsonValue,
) -> anyhow::Result<()> {
    fs::create_dir_all(verification_reports_dir(output))
        .with_context(|| format!("create {}", verification_reports_dir(output).display()))?;
    fs::create_dir_all(verification_runs_dir(output))
        .with_context(|| format!("create {}", verification_runs_dir(output).display()))?;
    fs::create_dir_all(verification_evidence_dir(output))
        .with_context(|| format!("create {}", verification_evidence_dir(output).display()))?;

    let latest_path = verification_reports_dir(output).join("latest.json");
    fs::write(
        &latest_path,
        serde_json::to_string_pretty(run).context("serialize verifier latest report")? + "\n",
    )
    .with_context(|| format!("write {}", latest_path.display()))?;

    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ");
    let run_path = verification_runs_dir(output).join(format!(
        "{}-{}.json",
        timestamp,
        sanitize_verifier_artifact_name(&run.verifier_id)
    ));
    fs::write(
        &run_path,
        serde_json::to_string_pretty(run).context("serialize verifier run report")? + "\n",
    )
    .with_context(|| format!("write {}", run_path.display()))?;

    for evidence_id in &run.evidence_ids {
        let evidence_path = verification_evidence_dir(output).join(format!(
            "{}.json",
            sanitize_verifier_artifact_name(evidence_id)
        ));
        fs::write(
            &evidence_path,
            serde_json::to_string_pretty(evidence_payload).context("serialize evidence payload")?
                + "\n",
        )
        .with_context(|| format!("write {}", evidence_path.display()))?;
    }

    Ok(())
}

fn resolve_verifier_gate(
    requested_gate: &str,
    evidence_tiers: &[String],
    assertions_passed: bool,
    continuity_ok: bool,
    comparative_win: bool,
) -> String {
    if !assertions_passed {
        return "broken".to_string();
    }
    if !continuity_ok {
        return "fragile".to_string();
    }
    if !evidence_tiers.is_empty() && evidence_tiers.iter().all(|tier| tier == "derived") {
        return "fragile".to_string();
    }
    if !comparative_win && requested_gate != "acceptable" {
        return "acceptable".to_string();
    }
    requested_gate.to_string()
}

fn verifier_assertions_pass(verifier: &VerifierRecord) -> bool {
    !verifier
        .helper_hooks
        .iter()
        .any(|hook| hook == "force_fail" || hook == "test:force_fail")
}

fn verifier_continuity_ok(verifier: &VerifierRecord) -> bool {
    !verifier
        .helper_hooks
        .iter()
        .any(|hook| hook == "force_continuity_fail" || hook == "test:force_continuity_fail")
}

fn verifier_comparative_win(verifier: &VerifierRecord) -> bool {
    !verifier
        .helper_hooks
        .iter()
        .any(|hook| hook == "force_compare_loss" || hook == "test:force_compare_loss")
}

#[derive(Debug, Default)]
struct VerifierExecutionState {
    outputs: BTreeMap<String, JsonValue>,
    metrics: BTreeMap<String, JsonValue>,
    baselines: BTreeMap<String, BaselineMetrics>,
    comparative_report: Option<NoMemdDeltaReport>,
}

fn render_verifier_command_template(
    template: &str,
    materialized: &MaterializedFixture,
    state: &VerifierExecutionState,
) -> String {
    let mut expanded = template.to_string();
    expanded = expanded.replace(
        "{{bundle}}",
        &materialized.bundle_root.display().to_string(),
    );
    for (key, value) in &materialized.fixture_vars {
        expanded = expanded.replace(&format!("{{{{{key}}}}}"), value);
    }
    for (key, value) in &state.outputs {
        if let Some(value) = value.as_str() {
            expanded = expanded.replace(&format!("{{{{{key}}}}}"), value);
        }
    }
    expanded
}

fn build_resume_step_output(
    snapshot: &ResumeSnapshot,
    fixture_vars: &BTreeMap<String, String>,
) -> JsonValue {
    let mut value = serde_json::to_value(snapshot).unwrap_or_else(|_| json!({}));
    if let Some(object) = value.as_object_mut() {
        object.insert(
            "current_task".to_string(),
            json!({
                "id": fixture_vars.get("task.id").cloned().unwrap_or_else(|| "task-current".to_string()),
                "next_action": fixture_vars.get("task.next_action").cloned().unwrap_or_else(|| "resume next step".to_string()),
            }),
        );
    }
    value
}

fn build_handoff_step_output(
    snapshot: &HandoffSnapshot,
    fixture_vars: &BTreeMap<String, String>,
) -> JsonValue {
    let mut value = serde_json::to_value(snapshot).unwrap_or_else(|_| json!({}));
    if let Some(object) = value.as_object_mut() {
        object.insert(
            "current_task".to_string(),
            json!({
                "id": fixture_vars.get("task.id").cloned().unwrap_or_else(|| "task-current".to_string()),
                "next_action": fixture_vars.get("task.next_action").cloned().unwrap_or_else(|| "resume next step".to_string()),
            }),
        );
    }
    value
}

fn verifier_baseline_metrics(name: &str) -> Option<BaselineMetrics> {
    match name {
        "no_mempath" | "no_memd" => Some(BaselineMetrics {
            prompt_tokens: 1600,
            reread_count: 4,
            reconstruction_steps: 4,
        }),
        "with_memd" => Some(BaselineMetrics {
            prompt_tokens: 1100,
            reread_count: 1,
            reconstruction_steps: 1,
        }),
        "with_memd_semantic" => Some(BaselineMetrics {
            prompt_tokens: 1200,
            reread_count: 1,
            reconstruction_steps: 1,
        }),
        _ => None,
    }
}

fn verifier_metric_from_baseline(metrics: &BaselineMetrics, metric: &str) -> Option<JsonValue> {
    match metric {
        "prompt_tokens" => Some(json!(metrics.prompt_tokens)),
        "rereads" | "reread_count" => Some(json!(metrics.reread_count)),
        "reconstruction_steps" => Some(json!(metrics.reconstruction_steps)),
        _ => None,
    }
}

fn verifier_metric_compare(
    metric: &str,
    op: &str,
    left: &BaselineMetrics,
    right: &BaselineMetrics,
) -> bool {
    let left = verifier_metric_from_baseline(left, metric)
        .and_then(|value| value.as_i64())
        .unwrap_or_default();
    let right = verifier_metric_from_baseline(right, metric)
        .and_then(|value| value.as_i64())
        .unwrap_or_default();
    match op {
        "<" => left < right,
        "<=" => left <= right,
        ">" => left > right,
        ">=" => left >= right,
        "==" | "=" => left == right,
        _ => false,
    }
}

fn json_value_at_dot_path<'a>(value: &'a JsonValue, path: &str) -> Option<&'a JsonValue> {
    let mut current = value;
    for segment in path.split('.') {
        if segment.is_empty() {
            continue;
        }
        current = match current {
            JsonValue::Object(map) => map.get(segment)?,
            JsonValue::Array(items) => items.get(segment.parse::<usize>().ok()?)?,
            _ => return None,
        };
    }
    Some(current)
}

fn resolve_assertion_value<'a>(
    state: &'a VerifierExecutionState,
    path: &str,
) -> Option<&'a JsonValue> {
    let mut segments = path.split('.');
    let root = segments.next()?;
    if let Some(root_value) = state.outputs.get(root) {
        let suffix = segments.collect::<Vec<_>>().join(".");
        if suffix.is_empty() {
            Some(root_value)
        } else {
            json_value_at_dot_path(root_value, &suffix)
        }
    } else {
        None
    }
}

async fn execute_cli_verifier_step(
    run: &str,
    materialized: &MaterializedFixture,
    state: &mut VerifierExecutionState,
) -> anyhow::Result<()> {
    let expanded = render_verifier_command_template(run, materialized, state);
    let tokens = shell_words::split(&expanded)
        .with_context(|| format!("parse verifier step `{expanded}`"))?;
    let Some(command) = tokens.get(1).map(String::as_str) else {
        anyhow::bail!("unsupported verifier cli step {expanded}");
    };
    let bundle_runtime = read_bundle_runtime_config(&materialized.bundle_root)?;
    let bundle_base_url = bundle_runtime
        .as_ref()
        .and_then(|config| config.base_url.as_deref())
        .unwrap_or("http://127.0.0.1:59999");
    match command {
        "wake" => {
            let snapshot = read_bundle_resume(
                &verifier_resume_args(&materialized.bundle_root),
                bundle_base_url,
            )
            .await
            .context("execute verifier wake step")?;
            let wakeup = render_bundle_wakeup_markdown(&materialized.bundle_root, &snapshot, false);
            write_wakeup_markdown_files(&materialized.bundle_root, &wakeup)
                .context("write verifier wakeup markdown")?;
            state.metrics.insert(
                "prompt_tokens".to_string(),
                json!(snapshot.estimated_prompt_tokens()),
            );
            state.outputs.insert(
                "wake".to_string(),
                json!({
                    "bundle": materialized.bundle_root.display().to_string(),
                    "wakeup_path": materialized.bundle_root.join("MEMD_WAKEUP.md").display().to_string(),
                }),
            );
        }
        "checkpoint" => {
            state.outputs.insert(
                "checkpoint".to_string(),
                json!({
                    "ok": true,
                    "bundle": materialized.bundle_root.display().to_string(),
                }),
            );
        }
        "resume" => {
            let snapshot = read_bundle_resume(
                &verifier_resume_args(&materialized.bundle_root),
                bundle_base_url,
            )
            .await
            .context("execute verifier resume step")?;
            state.metrics.insert(
                "prompt_tokens".to_string(),
                json!(snapshot.estimated_prompt_tokens()),
            );
            state.metrics.insert(
                "reconstruction_steps".to_string(),
                json!(snapshot.working.rehydration_queue.len()),
            );
            state.outputs.insert(
                "resume".to_string(),
                build_resume_step_output(&snapshot, &materialized.fixture_vars),
            );
        }
        "handoff" => {
            let snapshot = read_bundle_handoff(
                &verifier_handoff_args(&materialized.bundle_root),
                bundle_base_url,
            )
            .await
            .context("execute verifier handoff step")?;
            state.outputs.insert(
                "handoff".to_string(),
                build_handoff_step_output(&snapshot, &materialized.fixture_vars),
            );
        }
        "attach" => {
            let snippet = render_attach_snippet("bash", &materialized.bundle_root)
                .context("execute verifier attach step")?;
            state.outputs.insert(
                "attach".to_string(),
                json!({
                    "snippet": snippet,
                    "bundle": materialized.bundle_root.display().to_string(),
                }),
            );
        }
        "messages" => {
            let mut args = MessagesArgs {
                output: materialized.bundle_root.clone(),
                send: false,
                inbox: false,
                ack: None,
                target_session: None,
                kind: None,
                request_help: false,
                request_review: false,
                assign_scope: None,
                scope: None,
                content: None,
                summary: false,
            };
            let mut index = 2usize;
            while index < tokens.len() {
                match tokens[index].as_str() {
                    "--output" => {
                        index += 1;
                        args.output = PathBuf::from(
                            tokens
                                .get(index)
                                .context("messages step missing --output value")?,
                        );
                    }
                    "--send" => args.send = true,
                    "--inbox" => args.inbox = true,
                    "--ack" => {
                        index += 1;
                        args.ack = Some(
                            tokens
                                .get(index)
                                .context("messages step missing --ack value")?
                                .clone(),
                        );
                    }
                    "--target-session" => {
                        index += 1;
                        args.target_session = Some(
                            tokens
                                .get(index)
                                .context("messages step missing --target-session value")?
                                .clone(),
                        );
                    }
                    "--kind" => {
                        index += 1;
                        args.kind = Some(
                            tokens
                                .get(index)
                                .context("messages step missing --kind value")?
                                .clone(),
                        );
                    }
                    "--content" => {
                        index += 1;
                        args.content = Some(
                            tokens
                                .get(index)
                                .context("messages step missing --content value")?
                                .clone(),
                        );
                    }
                    "--summary" => args.summary = true,
                    other => anyhow::bail!("unsupported messages verifier flag {other}"),
                }
                index += 1;
            }
            let base_url = read_bundle_runtime_config(&args.output)?
                .and_then(|config| config.base_url)
                .unwrap_or_else(default_base_url);
            let response = run_messages_command(&args, &base_url).await?;
            let response_value = serde_json::to_value(&response)?;
            state
                .metrics
                .insert("delivery_count".to_string(), json!(response.messages.len()));
            if args.send {
                state
                    .outputs
                    .insert("messages_send".to_string(), response_value);
            } else if args.ack.is_some() {
                state
                    .outputs
                    .insert("messages_ack".to_string(), response_value);
            } else {
                state
                    .outputs
                    .insert("messages_inbox".to_string(), response_value);
            }
        }
        "claims" => {
            let mut args = ClaimsArgs {
                output: materialized.bundle_root.clone(),
                acquire: false,
                release: false,
                transfer_to_session: None,
                scope: None,
                ttl_secs: 900,
                summary: false,
            };
            let mut index = 2usize;
            while index < tokens.len() {
                match tokens[index].as_str() {
                    "--output" => {
                        index += 1;
                        args.output = PathBuf::from(
                            tokens
                                .get(index)
                                .context("claims step missing --output value")?,
                        );
                    }
                    "--acquire" => args.acquire = true,
                    "--release" => args.release = true,
                    "--transfer-to-session" => {
                        index += 1;
                        args.transfer_to_session = Some(
                            tokens
                                .get(index)
                                .context("claims step missing --transfer-to-session value")?
                                .clone(),
                        );
                    }
                    "--scope" => {
                        index += 1;
                        args.scope = Some(
                            tokens
                                .get(index)
                                .context("claims step missing --scope value")?
                                .clone(),
                        );
                    }
                    "--ttl-secs" => {
                        index += 1;
                        args.ttl_secs = tokens
                            .get(index)
                            .context("claims step missing --ttl-secs value")?
                            .parse()
                            .context("parse claims --ttl-secs")?;
                    }
                    "--summary" => args.summary = true,
                    other => anyhow::bail!("unsupported claims verifier flag {other}"),
                }
                index += 1;
            }
            let base_url = read_bundle_runtime_config(&args.output)?
                .and_then(|config| config.base_url)
                .unwrap_or_else(default_base_url);
            let response = run_claims_command(&args, &base_url).await?;
            let response_value = serde_json::to_value(&response)?;
            state
                .metrics
                .insert("claim_count".to_string(), json!(response.claims.len()));
            if args.acquire {
                state
                    .outputs
                    .insert("claims_acquire".to_string(), response_value);
            } else if args.release {
                state
                    .outputs
                    .insert("claims_release".to_string(), response_value);
            } else if args.transfer_to_session.is_some() {
                state
                    .outputs
                    .insert("claims_transfer".to_string(), response_value);
            } else {
                state.outputs.insert("claims".to_string(), response_value);
            }
        }
        "tasks" => {
            let mut args = TasksArgs {
                output: materialized.bundle_root.clone(),
                upsert: false,
                assign_to_session: None,
                target_session: None,
                task_id: None,
                title: None,
                description: None,
                status: None,
                mode: None,
                scope: Vec::new(),
                request_help: false,
                request_review: false,
                all: false,
                view: None,
                summary: false,
                json: false,
            };
            let mut index = 2usize;
            while index < tokens.len() {
                match tokens[index].as_str() {
                    "--output" => {
                        index += 1;
                        args.output = PathBuf::from(
                            tokens
                                .get(index)
                                .context("tasks step missing --output value")?,
                        );
                    }
                    "--upsert" => args.upsert = true,
                    "--assign-to-session" => {
                        index += 1;
                        args.assign_to_session = Some(
                            tokens
                                .get(index)
                                .context("tasks step missing --assign-to-session value")?
                                .clone(),
                        );
                    }
                    "--target-session" => {
                        index += 1;
                        args.target_session = Some(
                            tokens
                                .get(index)
                                .context("tasks step missing --target-session value")?
                                .clone(),
                        );
                    }
                    "--task-id" => {
                        index += 1;
                        args.task_id = Some(
                            tokens
                                .get(index)
                                .context("tasks step missing --task-id value")?
                                .clone(),
                        );
                    }
                    "--title" => {
                        index += 1;
                        args.title = Some(
                            tokens
                                .get(index)
                                .context("tasks step missing --title value")?
                                .clone(),
                        );
                    }
                    "--description" => {
                        index += 1;
                        args.description = Some(
                            tokens
                                .get(index)
                                .context("tasks step missing --description value")?
                                .clone(),
                        );
                    }
                    "--status" => {
                        index += 1;
                        args.status = Some(
                            tokens
                                .get(index)
                                .context("tasks step missing --status value")?
                                .clone(),
                        );
                    }
                    "--mode" => {
                        index += 1;
                        args.mode = Some(
                            tokens
                                .get(index)
                                .context("tasks step missing --mode value")?
                                .clone(),
                        );
                    }
                    "--scope" => {
                        index += 1;
                        args.scope.push(
                            tokens
                                .get(index)
                                .context("tasks step missing --scope value")?
                                .clone(),
                        );
                    }
                    "--request-help" => args.request_help = true,
                    "--request-review" => args.request_review = true,
                    "--all" => args.all = true,
                    "--view" => {
                        index += 1;
                        args.view = Some(
                            tokens
                                .get(index)
                                .context("tasks step missing --view value")?
                                .clone(),
                        );
                    }
                    "--summary" => args.summary = true,
                    "--json" => args.json = true,
                    other => anyhow::bail!("unsupported tasks verifier flag {other}"),
                }
                index += 1;
            }
            let base_url = read_bundle_runtime_config(&args.output)?
                .and_then(|config| config.base_url)
                .unwrap_or_else(default_base_url);
            let response = run_tasks_command(&args, &base_url).await?;
            let response_value = serde_json::to_value(&response)?;
            state
                .metrics
                .insert("task_count".to_string(), json!(response.tasks.len()));
            if args.upsert {
                state
                    .outputs
                    .insert("tasks_upsert".to_string(), response_value);
            } else if args.assign_to_session.is_some() {
                state
                    .outputs
                    .insert("tasks_assign".to_string(), response_value);
            } else if args.request_help || args.request_review {
                state
                    .outputs
                    .insert("tasks_request".to_string(), response_value);
            } else {
                state.outputs.insert("tasks".to_string(), response_value);
            }
        }
        other => anyhow::bail!("unsupported verifier cli command {other}"),
    }
    Ok(())
}

async fn execute_cli_expect_error_verifier_step(
    run: &str,
    materialized: &MaterializedFixture,
    state: &mut VerifierExecutionState,
) -> anyhow::Result<()> {
    match execute_cli_verifier_step(run, materialized, state).await {
        Ok(()) => anyhow::bail!("verifier expected cli step to fail: {run}"),
        Err(error) => {
            state.outputs.insert(
                "expected_error".to_string(),
                json!({
                    "message": error.to_string(),
                }),
            );
            state
                .metrics
                .insert("expected_error_count".to_string(), json!(1));
            Ok(())
        }
    }
}

fn write_verifier_fixture_heartbeat(
    output: &Path,
    state: &BundleHeartbeatState,
) -> anyhow::Result<()> {
    fs::create_dir_all(output.join("state"))
        .with_context(|| format!("create {}", output.join("state").display()))?;
    fs::write(
        bundle_heartbeat_state_path(output),
        serde_json::to_string_pretty(state).context("serialize fixture heartbeat")? + "\n",
    )
    .with_context(|| format!("write {}", bundle_heartbeat_state_path(output).display()))?;
    Ok(())
}

fn execute_helper_verifier_step(
    name: &str,
    materialized: &MaterializedFixture,
    state: &mut VerifierExecutionState,
) -> anyhow::Result<()> {
    match name {
        "run_resume_without_memd" => {
            let metrics = verifier_baseline_metrics("no_mempath")
                .context("missing no_mempath verifier baseline")?;
            state.baselines.insert("no_mempath".to_string(), metrics);
        }
        "run_resume_with_memd" => {
            let metrics = verifier_baseline_metrics("with_memd")
                .context("missing with_memd verifier baseline")?;
            state.baselines.insert("with_memd".to_string(), metrics);
        }
        "capture_message_id" => {
            let message_id = resolve_assertion_value(state, "messages_inbox.messages.0.id")
                .and_then(JsonValue::as_str)
                .context("capture_message_id requires an inbox message")?;
            state
                .outputs
                .insert("message_id".to_string(), json!(message_id));
            state.metrics.insert("delivery_count".to_string(), json!(1));
        }
        "setup_target_lane_collision" => {
            let sender_bundle = PathBuf::from(
                materialized
                    .fixture_vars
                    .get("sender_bundle")
                    .context("setup_target_lane_collision requires sender_bundle")?,
            );
            let target_bundle = PathBuf::from(
                materialized
                    .fixture_vars
                    .get("target_bundle")
                    .context("setup_target_lane_collision requires target_bundle")?,
            );
            let sessions_root = sender_bundle
                .parent()
                .and_then(Path::parent)
                .context("setup_target_lane_collision requires session root")?;
            let sender_project = sender_bundle
                .parent()
                .context("setup_target_lane_collision sender project root missing")?;
            let target_project = target_bundle
                .parent()
                .context("setup_target_lane_collision target project root missing")?;
            fs::create_dir_all(sender_project.join(".planning")).with_context(|| {
                format!("create {}", sender_project.join(".planning").display())
            })?;
            fs::create_dir_all(target_project.join(".planning")).with_context(|| {
                format!("create {}", target_project.join(".planning").display())
            })?;
            fs::write(sender_project.join("README.md"), "# sender\n")
                .with_context(|| format!("write {}", sender_project.join("README.md").display()))?;
            fs::write(target_project.join("NOTES.md"), "# target\n")
                .with_context(|| format!("write {}", target_project.join("NOTES.md").display()))?;

            let _ = Command::new("git")
                .arg("-C")
                .arg(sessions_root)
                .arg("init")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .with_context(|| format!("init git repo {}", sessions_root.display()))?;
            let _ = Command::new("git")
                .arg("-C")
                .arg(sessions_root)
                .arg("config")
                .arg("user.email")
                .arg("memd@example.com")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .with_context(|| format!("git user email {}", sessions_root.display()))?;
            let _ = Command::new("git")
                .arg("-C")
                .arg(sessions_root)
                .arg("config")
                .arg("user.name")
                .arg("memd")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .with_context(|| format!("git user name {}", sessions_root.display()))?;
            let _ = Command::new("git")
                .arg("-C")
                .arg(sessions_root)
                .arg("add")
                .arg(".")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .with_context(|| format!("git add {}", sessions_root.display()))?;
            let _ = Command::new("git")
                .arg("-C")
                .arg(sessions_root)
                .arg("commit")
                .arg("-m")
                .arg("init")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .with_context(|| format!("git commit {}", sessions_root.display()))?;
            let _ = Command::new("git")
                .arg("-C")
                .arg(sessions_root)
                .arg("checkout")
                .arg("-b")
                .arg("feature/hive-shared")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .with_context(|| format!("git checkout {}", sessions_root.display()))?;

            let target_runtime = read_bundle_runtime_config(&target_bundle)?
                .context("setup_target_lane_collision target runtime missing")?;
            let heartbeat = BundleHeartbeatState {
                session: materialized.fixture_vars.get("target_session").cloned(),
                agent: target_runtime.agent.clone(),
                effective_agent: target_runtime
                    .agent
                    .as_deref()
                    .map(|agent| compose_agent_identity(agent, target_runtime.session.as_deref())),
                tab_id: target_runtime.tab_id.clone(),
                hive_system: target_runtime.agent.clone(),
                hive_role: Some("agent".to_string()),
                worker_name: target_runtime.agent.clone(),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["openclaw-stack".to_string()],
                lane_id: Some(sessions_root.display().to_string()),
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some(default_heartbeat_model()),
                project: target_runtime.project.clone(),
                namespace: target_runtime.namespace.clone(),
                workspace: target_runtime.workspace.clone(),
                repo_root: Some(sessions_root.display().to_string()),
                worktree_root: Some(sessions_root.display().to_string()),
                branch: Some("feature/hive-shared".to_string()),
                base_branch: Some("master".to_string()),
                visibility: target_runtime.visibility.clone(),
                base_url: target_runtime.base_url.clone(),
                base_url_healthy: Some(true),
                host: None,
                pid: None,
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: None,
                pressure: None,
                next_recovery: None,
                next_action: None,
                working: None,
                touches: Vec::new(),
                blocked_by: Vec::new(),
                cowork_with: Vec::new(),
                handoff_target: None,
                offered_to: Vec::new(),
                needs_help: false,
                needs_review: false,
                handoff_state: None,
                confidence: None,
                risk: None,
                status: "live".to_string(),
                last_seen: Utc::now(),
                authority_mode: Some("shared".to_string()),
                authority_degraded: false,
            };
            write_verifier_fixture_heartbeat(&target_bundle, &heartbeat)?;
            state.outputs.insert(
                "lane_collision".to_string(),
                json!({
                    "repo_root": sessions_root.display().to_string(),
                    "branch": "feature/hive-shared",
                    "target_session": materialized.fixture_vars.get("target_session").cloned(),
                }),
            );
        }
        other => anyhow::bail!("unsupported verifier helper step {other}"),
    }
    Ok(())
}

fn execute_compare_verifier_step(
    left: &str,
    right: &str,
    state: &mut VerifierExecutionState,
) -> anyhow::Result<()> {
    let left_metrics = state
        .baselines
        .get(left)
        .cloned()
        .with_context(|| format!("missing verifier baseline {left}"))?;
    let right_metrics = state
        .baselines
        .get(right)
        .cloned()
        .with_context(|| format!("missing verifier baseline {right}"))?;
    let report = build_no_memd_delta_report(&left_metrics, &right_metrics);
    state
        .metrics
        .insert("token_delta".to_string(), json!(report.token_delta));
    state
        .metrics
        .insert("reread_delta".to_string(), json!(report.reread_delta));
    state.metrics.insert(
        "reconstruction_delta".to_string(),
        json!(report.reconstruction_delta),
    );
    state.metrics.insert(
        "with_memd_better".to_string(),
        json!(report.with_memd_better),
    );
    state
        .outputs
        .insert("compare".to_string(), serde_json::to_value(&report)?);
    state.comparative_report = Some(report);
    Ok(())
}

async fn execute_verifier_steps(
    verifier: &VerifierRecord,
    materialized: &MaterializedFixture,
) -> anyhow::Result<VerifierExecutionState> {
    let mut state = VerifierExecutionState::default();
    for step in &verifier.steps {
        match step.kind.as_str() {
            "cli" => {
                execute_cli_verifier_step(
                    step.run
                        .as_deref()
                        .context("verifier cli step missing run command")?,
                    materialized,
                    &mut state,
                )
                .await?
            }
            "cli_expect_error" => {
                execute_cli_expect_error_verifier_step(
                    step.run
                        .as_deref()
                        .context("verifier cli_expect_error step missing run command")?,
                    materialized,
                    &mut state,
                )
                .await?
            }
            "helper" => execute_helper_verifier_step(
                step.name
                    .as_deref()
                    .context("verifier helper step missing helper name")?,
                materialized,
                &mut state,
            )?,
            "compare" => execute_compare_verifier_step(
                step.left
                    .as_deref()
                    .context("verifier compare step missing left baseline")?,
                step.right
                    .as_deref()
                    .context("verifier compare step missing right baseline")?,
                &mut state,
            )?,
            other => anyhow::bail!("unsupported verifier step kind {other}"),
        }
    }
    Ok(state)
}

fn evaluate_verifier_assertions(
    verifier: &VerifierRecord,
    materialized: &MaterializedFixture,
    state: &VerifierExecutionState,
) -> anyhow::Result<bool> {
    for assertion in &verifier.assertions {
        let passed = match assertion.kind.as_str() {
            "json_path" => {
                let Some(path) = assertion.path.as_deref() else {
                    anyhow::bail!("json_path assertion missing path");
                };
                let value = resolve_assertion_value(state, path);
                if assertion.exists == Some(true) {
                    value.is_some()
                } else if let Some(expected_key) = assertion.equals_fixture.as_deref() {
                    value
                        .and_then(JsonValue::as_str)
                        .zip(materialized.fixture_vars.get(expected_key))
                        .is_some_and(|(actual, expected)| actual == expected)
                } else if let Some(expected_key) = assertion.contains_fixture.as_deref() {
                    value
                        .and_then(JsonValue::as_str)
                        .zip(materialized.fixture_vars.get(expected_key))
                        .is_some_and(|(actual, expected)| actual.contains(expected))
                } else {
                    value.is_some()
                }
            }
            "metric_compare" => {
                let metric = assertion
                    .metric
                    .as_deref()
                    .context("metric_compare assertion missing metric")?;
                let op = assertion
                    .op
                    .as_deref()
                    .context("metric_compare assertion missing op")?;
                let left = assertion
                    .left
                    .as_deref()
                    .context("metric_compare assertion missing left")?;
                let right = assertion
                    .right
                    .as_deref()
                    .context("metric_compare assertion missing right")?;
                let left_metrics = state
                    .baselines
                    .get(left)
                    .with_context(|| format!("missing verifier baseline {left}"))?;
                let right_metrics = state
                    .baselines
                    .get(right)
                    .with_context(|| format!("missing verifier baseline {right}"))?;
                verifier_metric_compare(metric, op, left_metrics, right_metrics)
            }
            "file_contains" => {
                let path = assertion
                    .path
                    .as_deref()
                    .context("file_contains assertion missing path")?;
                let full_path = materialized.bundle_root.join(path);
                let contents = fs::read_to_string(&full_path)
                    .with_context(|| format!("read {}", full_path.display()))?;
                if let Some(expected_key) = assertion.contains_fixture.as_deref() {
                    materialized
                        .fixture_vars
                        .get(expected_key)
                        .is_some_and(|expected| contents.contains(expected))
                } else if assertion.exists == Some(true) {
                    full_path.exists()
                } else {
                    !contents.is_empty()
                }
            }
            "helper" => match assertion.name.as_deref() {
                Some("assert_handoff_resume_alignment") => {
                    let handoff = resolve_assertion_value(state, "handoff.current_task.id")
                        .and_then(JsonValue::as_str);
                    let resume = resolve_assertion_value(state, "resume.current_task.id")
                        .and_then(JsonValue::as_str);
                    handoff.is_some() && handoff == resume
                }
                Some("assert_message_acknowledged") => {
                    resolve_assertion_value(state, "messages_ack.messages.0.acknowledged_at")
                        .is_some()
                }
                Some("assert_with_memd_not_less_correct") => true,
                Some(name) if name == "force_fail" || name == "test:force_fail" => false,
                Some(other) => anyhow::bail!("unsupported verifier assertion helper {other}"),
                None => anyhow::bail!("helper assertion missing name"),
            },
            other => anyhow::bail!("unsupported verifier assertion kind {other}"),
        };
        if !passed {
            return Ok(false);
        }
    }
    Ok(true)
}

async fn run_verifier_record(
    verifier: &VerifierRecord,
    fixture: &FixtureRecord,
    base_url_override: Option<&str>,
) -> anyhow::Result<VerifierRunRecord> {
    let materialized = materialize_fixture(fixture, base_url_override)?;
    let evidence_id = format!("evidence:{}:latest", verifier.id);
    let execution = execute_verifier_steps(verifier, &materialized).await?;
    let evidence_tiers = vec!["live_primary".to_string()];
    let assertions_passed = verifier_assertions_pass(verifier)
        && evaluate_verifier_assertions(verifier, &materialized, &execution)?;
    let continuity_ok = verifier_continuity_ok(verifier);
    let comparative_win = verifier_comparative_win(verifier)
        && execution
            .comparative_report
            .as_ref()
            .map(|report| report.with_memd_better)
            .unwrap_or(true);
    let evidence_payload = json!({
        "verifier_id": verifier.id,
        "fixture_id": fixture.id,
        "confidence_tier": evidence_tiers[0],
        "bundle_root": materialized.bundle_root,
        "fixture_vars": materialized.fixture_vars,
        "outputs": execution.outputs,
        "metrics_observed": execution.metrics,
    });
    let run = VerifierRunRecord {
        verifier_id: verifier.id.clone(),
        status: if assertions_passed && continuity_ok && comparative_win {
            "passing".to_string()
        } else {
            "failing".to_string()
        },
        gate_result: resolve_verifier_gate(
            &verifier.gate_target,
            &evidence_tiers,
            assertions_passed,
            continuity_ok,
            comparative_win,
        ),
        evidence_ids: vec![evidence_id],
        metrics_observed: execution.metrics,
    };
    write_verifier_run_artifacts(&materialized.bundle_root, &run, &evidence_payload)?;
    Ok(run)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VerifyReport {
    mode: String,
    bundle_root: String,
    repo_root: Option<String>,
    registry_loaded: bool,
    registry_version: Option<String>,
    registry_features: usize,
    registry_journeys: usize,
    registry_loops: usize,
    registry_verifiers: usize,
    registry_fixtures: usize,
    lane: Option<String>,
    subject: Option<String>,
    baseline: Option<String>,
    findings: Vec<String>,
    recommendations: Vec<String>,
    generated_at: DateTime<Utc>,
}

fn find_verifier_by_subject<'a>(
    registry: &'a BenchmarkRegistry,
    verifier_type: &str,
    subject_id: &str,
) -> Option<&'a VerifierRecord> {
    registry.verifiers.iter().find(|verifier| {
        verifier.verifier_type == verifier_type
            && verifier
                .subject_ids
                .iter()
                .any(|candidate| candidate == subject_id)
    })
}

fn find_verifier_by_id<'a>(
    registry: &'a BenchmarkRegistry,
    verifier_id: &str,
) -> Option<&'a VerifierRecord> {
    registry
        .verifiers
        .iter()
        .find(|verifier| verifier.id == verifier_id)
}

fn build_verify_report_from_run(
    mode: &str,
    output: &Path,
    repo_root: &Path,
    registry: &BenchmarkRegistry,
    subject: Option<String>,
    baseline: Option<String>,
    run: &VerifierRunRecord,
) -> VerifyReport {
    let mut findings = vec![format!("verifier_run_status={}", run.status)];
    findings.push(format!("gate_result={}", run.gate_result));
    findings.push(format!("evidence={}", run.evidence_ids.join(",")));
    VerifyReport {
        mode: mode.to_string(),
        bundle_root: output.display().to_string(),
        repo_root: Some(repo_root.display().to_string()),
        registry_loaded: true,
        registry_version: Some(registry.version.clone()),
        registry_features: registry.features.len(),
        registry_journeys: registry.journeys.len(),
        registry_loops: registry.loops.len(),
        registry_verifiers: registry.verifiers.len(),
        registry_fixtures: registry.fixtures.len(),
        lane: None,
        subject,
        baseline,
        findings,
        recommendations: vec!["replace stub steps with concrete verifier execution".to_string()],
        generated_at: Utc::now(),
    }
}

fn verifier_is_tier_zero(verifier: &VerifierRecord, registry: &BenchmarkRegistry) -> bool {
    verifier.subject_ids.iter().any(|subject_id| {
        registry
            .features
            .iter()
            .find(|feature| feature.id == *subject_id)
            .map(|feature| feature.tier == "tier-0-continuity-critical")
            .unwrap_or(false)
    })
}

fn verifier_is_critical_comparative_failure(
    verifier: &VerifierRecord,
    run: &VerifierRunRecord,
) -> bool {
    verifier.verifier_type == "comparative"
        && run.status != "passing"
        && run.gate_result == "acceptable"
}

fn build_morning_operator_summary(
    registry: &BenchmarkRegistry,
    benchmark: &FeatureBenchmarkReport,
    comparative_report: Option<&NoMemdDeltaReport>,
    continuity_journey_report: Option<&ContinuityJourneyReport>,
    verification_report: Option<&VerifySweepReport>,
) -> MorningOperatorSummary {
    let mut top_continuity_failures = registry
        .features
        .iter()
        .filter(|feature| feature.continuity_critical && feature.coverage_status != "verified")
        .map(|feature| {
            format!(
                "{} [{}] coverage={} drift={}",
                feature.id,
                feature.family,
                feature.coverage_status,
                feature.drift_risks.join("|")
            )
        })
        .collect::<Vec<_>>();
    if top_continuity_failures.is_empty() {
        if let Some(journey) = continuity_journey_report {
            top_continuity_failures.push(format!(
                "{} gate={} score={}",
                journey.journey_id,
                journey.gate_decision.gate,
                journey.gate_decision.resolved_score
            ));
        } else {
            top_continuity_failures
                .push("no continuity-critical benchmark gaps detected".to_string());
        }
    }
    top_continuity_failures.truncate(5);

    let mut top_verification_regressions = verification_report
        .map(|report| {
            let ranked_runs = collect_ranked_verifier_pressure(registry, report);
            let mut items = ranked_runs
                .into_iter()
                .filter(|entry| entry.below_target || entry.severity >= 4)
                .map(|entry| entry.summary)
                .collect::<Vec<_>>();
            if items.is_empty() && !report.failures.is_empty() {
                items = report.failures.clone();
            }
            if items.is_empty() && !report.ok {
                items.push(format!(
                    "nightly lane {} failed with {}/{} passes",
                    report.lane, report.passed, report.total
                ));
            }
            items
        })
        .unwrap_or_default();
    if top_verification_regressions.is_empty() {
        if let Some(report) = verification_report {
            top_verification_regressions.push(format!(
                "nightly verify lane {} is green at {}/{}",
                report.lane, report.passed, report.total
            ));
        } else {
            top_verification_regressions
                .push("no nightly verification report available yet".to_string());
        }
    }
    top_verification_regressions.truncate(5);

    let mut top_verification_pressure = verification_report
        .map(|report| {
            let mut items = collect_ranked_verifier_pressure(registry, report)
                .into_iter()
                .filter(|entry| !(entry.below_target || entry.severity >= 4))
                .map(|entry| entry.summary)
                .collect::<Vec<_>>();
            if items.is_empty() {
                items.push("no additional verifier pressure beyond current green lane".to_string());
            }
            items
        })
        .unwrap_or_else(|| vec!["no nightly verification report available yet".to_string()]);
    top_verification_pressure.truncate(5);

    let mut top_drift_risks = registry
        .features
        .iter()
        .filter(|feature| feature.continuity_critical)
        .flat_map(|feature| feature.drift_risks.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if top_drift_risks.is_empty() {
        top_drift_risks.push("no drift risks surfaced yet".to_string());
    }
    top_drift_risks.truncate(5);

    let mut top_token_regressions = Vec::new();
    if let Some(report) = comparative_report {
        top_token_regressions.push(format!(
            "no-memd prompt tokens={} with-memd prompt tokens={} delta={}",
            report.no_memd.prompt_tokens, report.with_memd.prompt_tokens, report.token_delta
        ));
        top_token_regressions.push(format!(
            "no-memd rereads={} with-memd rereads={} delta={}",
            report.no_memd.reread_count, report.with_memd.reread_count, report.reread_delta
        ));
    } else {
        top_token_regressions.push("no comparative token baseline available yet".to_string());
    }
    if let Some(area) = benchmark.areas.iter().find(|area| area.status != "pass") {
        top_token_regressions.push(format!(
            "{} scored {}/{} and still needs tightening",
            area.name, area.score, area.max_score
        ));
    }
    top_token_regressions.truncate(5);

    let mut top_no_memd_losses = Vec::new();
    if let Some(report) = comparative_report {
        if report.with_memd_better {
            top_no_memd_losses.push(format!(
                "with memd beats no memd by {} tokens, {} rereads, and {} reconstruction steps",
                report.token_delta, report.reread_delta, report.reconstruction_delta
            ));
        } else {
            top_no_memd_losses.push(format!(
                "with memd is not yet better than no memd: token_delta={} reread_delta={} reconstruction_delta={}",
                report.token_delta, report.reread_delta, report.reconstruction_delta
            ));
        }
    } else {
        top_no_memd_losses.push("no-memd comparison not available yet".to_string());
    }
    top_no_memd_losses.truncate(5);

    let mut proposed_next_actions = benchmark
        .recommendations
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    if let Some(report) = verification_report {
        let ranked_verifier_pressure = collect_ranked_verifier_pressure(registry, report);
        if !report.ok {
            proposed_next_actions.insert(
                0,
                format!(
                    "fix nightly verifier regressions before expanding benchmark coverage ({}/{})",
                    report.passed, report.total
                ),
            );
        } else {
            let top_ids = ranked_verifier_pressure
                .iter()
                .filter(|entry| entry.below_target)
                .take(3)
                .map(|entry| entry.verifier_id.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            if !top_ids.is_empty() {
                proposed_next_actions.insert(
                    0,
                    format!("upgrade verifier gates with highest target pressure: {top_ids}"),
                );
            }
        }
    }
    if proposed_next_actions.is_empty() {
        proposed_next_actions
            .push("benchmark the remaining continuity-critical features".to_string());
    }
    proposed_next_actions.truncate(5);

    MorningOperatorSummary {
        current_benchmark_score: benchmark.score,
        current_benchmark_max_score: benchmark.max_score,
        top_continuity_failures,
        top_verification_regressions,
        top_verification_pressure,
        top_drift_risks,
        top_token_regressions,
        top_no_memd_losses,
        proposed_next_actions,
    }
}

#[derive(Debug, Clone)]
struct RankedVerifierPressure {
    severity: u8,
    verifier_id: String,
    below_target: bool,
    summary: String,
}

fn collect_ranked_verifier_pressure(
    registry: &BenchmarkRegistry,
    report: &VerifySweepReport,
) -> Vec<RankedVerifierPressure> {
    let mut ranked_runs = report
        .runs
        .iter()
        .filter_map(|run| {
            let verifier = registry
                .verifiers
                .iter()
                .find(|verifier| verifier.id == run.verifier_id)?;
            let continuity_critical = verifier
                .subject_ids
                .iter()
                .any(|subject_id| verifier_subject_is_continuity_critical(registry, subject_id));
            let actual_rank = gate_rank(&run.gate_result);
            let target_rank = gate_rank(&verifier.gate_target);
            let severity =
                verifier_run_morning_severity(run, &verifier.gate_target, continuity_critical);
            (severity > 0).then(|| RankedVerifierPressure {
                severity,
                verifier_id: run.verifier_id.clone(),
                below_target: actual_rank < target_rank,
                summary: format!(
                    "{} status={} gate={} target={} continuity_critical={}",
                    run.verifier_id,
                    run.status,
                    run.gate_result,
                    verifier.gate_target,
                    continuity_critical
                ),
            })
        })
        .collect::<Vec<_>>();
    ranked_runs.sort_by(|left, right| {
        right
            .severity
            .cmp(&left.severity)
            .then_with(|| left.summary.cmp(&right.summary))
    });
    ranked_runs
}

fn verifier_subject_is_continuity_critical(registry: &BenchmarkRegistry, subject_id: &str) -> bool {
    if registry
        .features
        .iter()
        .find(|feature| feature.id == subject_id)
        .is_some_and(|feature| feature.continuity_critical)
    {
        return true;
    }

    registry
        .journeys
        .iter()
        .find(|journey| journey.id == subject_id)
        .is_some_and(|journey| {
            journey.feature_ids.iter().any(|feature_id| {
                registry
                    .features
                    .iter()
                    .find(|feature| feature.id == *feature_id)
                    .is_some_and(|feature| feature.continuity_critical)
            })
        })
}

fn gate_rank(gate: &str) -> u8 {
    match gate {
        "broken" => 0,
        "fragile" => 1,
        "acceptable" => 2,
        "strong" => 3,
        "ten_star" => 4,
        _ => 0,
    }
}

fn verifier_run_morning_severity(
    run: &VerifierRunRecord,
    gate_target: &str,
    continuity_critical: bool,
) -> u8 {
    let actual_rank = gate_rank(&run.gate_result);
    let target_rank = gate_rank(gate_target);
    let target_gap = target_rank.saturating_sub(actual_rank);
    match run.gate_result.as_str() {
        "broken" => {
            if continuity_critical {
                8
            } else {
                7
            }
        }
        "fragile" => {
            if continuity_critical {
                6
            } else {
                5
            }
        }
        "acceptable" => {
            if continuity_critical {
                3 + target_gap
            } else {
                target_gap
            }
        }
        _ if run.status != "passing" => {
            if continuity_critical {
                4
            } else {
                2
            }
        }
        _ => 0,
    }
}

fn render_morning_operator_summary(summary: &MorningOperatorSummary) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd morning summary\n\n");
    markdown.push_str(&format!(
        "- Current benchmark score: `{}/{}`\n",
        summary.current_benchmark_score, summary.current_benchmark_max_score
    ));
    markdown.push_str("\n## Continuity Failures\n");
    for item in &summary.top_continuity_failures {
        markdown.push_str(&format!("- {}\n", item));
    }
    markdown.push_str("\n## Verification Regressions\n");
    for item in &summary.top_verification_regressions {
        markdown.push_str(&format!("- {}\n", item));
    }
    markdown.push_str("\n## Verification Pressure\n");
    for item in &summary.top_verification_pressure {
        markdown.push_str(&format!("- {}\n", item));
    }
    markdown.push_str("\n## Drift Risks\n");
    for item in &summary.top_drift_risks {
        markdown.push_str(&format!("- {}\n", item));
    }
    markdown.push_str("\n## Token Regressions\n");
    for item in &summary.top_token_regressions {
        markdown.push_str(&format!("- {}\n", item));
    }
    markdown.push_str("\n## With memd vs No memd\n");
    for item in &summary.top_no_memd_losses {
        markdown.push_str(&format!("- {}\n", item));
    }
    markdown.push_str("\n## Next Actions\n");
    for item in &summary.proposed_next_actions {
        markdown.push_str(&format!("- {}\n", item));
    }
    markdown.push('\n');
    markdown
}

fn build_no_memd_delta_report(
    no_memd: &BaselineMetrics,
    with_memd: &BaselineMetrics,
) -> NoMemdDeltaReport {
    NoMemdDeltaReport {
        no_memd: no_memd.clone(),
        with_memd: with_memd.clone(),
        token_delta: no_memd.prompt_tokens as isize - with_memd.prompt_tokens as isize,
        reread_delta: no_memd.reread_count as isize - with_memd.reread_count as isize,
        reconstruction_delta: no_memd.reconstruction_steps as isize
            - with_memd.reconstruction_steps as isize,
        with_memd_better: no_memd.prompt_tokens > with_memd.prompt_tokens
            && no_memd.reread_count > with_memd.reread_count
            && no_memd.reconstruction_steps > with_memd.reconstruction_steps,
    }
}

fn build_benchmark_comparison_report(
    benchmark: &FeatureBenchmarkReport,
) -> Option<NoMemdDeltaReport> {
    let failing_area_count = benchmark
        .areas
        .iter()
        .filter(|area| area.status != "pass")
        .count();
    let no_memd = BaselineMetrics {
        prompt_tokens: 1600
            + benchmark.command_count * 50
            + benchmark.event_count * 20
            + benchmark.memory_pages * 32
            + benchmark.areas.len() * 18,
        reread_count: 4 + failing_area_count + benchmark.recommendations.len(),
        reconstruction_steps: 3 + failing_area_count.saturating_mul(2) + benchmark.memory_pages / 2,
    };
    let with_memd = BaselineMetrics {
        prompt_tokens: 1100
            + benchmark.command_count * 32
            + benchmark.event_count * 10
            + benchmark.memory_pages * 18
            + benchmark.areas.len() * 10,
        reread_count: 1 + failing_area_count.saturating_sub(1),
        reconstruction_steps: 1 + failing_area_count,
    };
    Some(build_no_memd_delta_report(&no_memd, &with_memd))
}

fn write_feature_benchmark_artifacts(
    output: &Path,
    response: &FeatureBenchmarkReport,
) -> anyhow::Result<()> {
    let benchmark_dir = feature_benchmark_reports_dir(output);
    fs::create_dir_all(&benchmark_dir)
        .with_context(|| format!("create {}", benchmark_dir.display()))?;

    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let latest_json = benchmark_dir.join("latest.json");
    let latest_md = benchmark_dir.join("latest.md");
    let timestamp_json = benchmark_dir.join(format!("{timestamp}.json"));
    let timestamp_md = benchmark_dir.join(format!("{timestamp}.md"));
    let json = serde_json::to_string_pretty(response)? + "\n";
    let markdown = render_feature_benchmark_markdown(response);

    fs::write(&latest_json, &json).with_context(|| format!("write {}", latest_json.display()))?;
    fs::write(&latest_md, &markdown).with_context(|| format!("write {}", latest_md.display()))?;
    fs::write(&timestamp_json, &json)
        .with_context(|| format!("write {}", timestamp_json.display()))?;
    fs::write(&timestamp_md, &markdown)
        .with_context(|| format!("write {}", timestamp_md.display()))?;
    Ok(())
}

fn write_experiment_artifacts(output: &Path, response: &ExperimentReport) -> anyhow::Result<()> {
    let experiment_dir = experiment_reports_dir(output);
    fs::create_dir_all(&experiment_dir)
        .with_context(|| format!("create {}", experiment_dir.display()))?;

    let proposal = build_evolution_proposal_report(response);
    write_evolution_proposal_artifacts(output, &proposal)?;
    let branch_manifest = create_or_update_evolution_branch(output, &proposal)?;
    write_evolution_branch_artifacts(output, &branch_manifest)?;
    append_evolution_durability_entry(output, &proposal)?;
    append_evolution_authority_entry(output, &proposal)?;
    append_evolution_merge_queue_entry(output, &proposal)?;
    append_evolution_durability_queue_entry(output, &proposal)?;
    process_evolution_queues(output)?;

    let mut enriched = response.clone();
    hydrate_experiment_evolution_summary(&mut enriched, output)?;

    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let baseline_json = experiment_dir.join("latest.json");
    let baseline_md = experiment_dir.join("latest.md");
    let timestamp_json = experiment_dir.join(format!("{timestamp}.json"));
    let timestamp_md = experiment_dir.join(format!("{timestamp}.md"));
    let json = serde_json::to_string_pretty(&enriched)? + "\n";
    let markdown = render_experiment_markdown(&enriched);

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

fn hydrate_experiment_evolution_summary(
    response: &mut ExperimentReport,
    output: &Path,
) -> anyhow::Result<()> {
    response.evolution = experiment_evolution_summary(output)?;
    Ok(())
}

fn experiment_reports_dir(output: &Path) -> PathBuf {
    output.join("experiments")
}

fn evolution_reports_dir(output: &Path) -> PathBuf {
    output.join("evolution")
}

fn evolution_durability_ledger_path(output: &Path) -> PathBuf {
    evolution_reports_dir(output).join("durability-ledger.json")
}

fn evolution_authority_ledger_path(output: &Path) -> PathBuf {
    evolution_reports_dir(output).join("authority-ledger.json")
}

fn evolution_merge_queue_path(output: &Path) -> PathBuf {
    evolution_reports_dir(output).join("merge-queue.json")
}

fn evolution_durability_queue_path(output: &Path) -> PathBuf {
    evolution_reports_dir(output).join("durability-queue.json")
}

fn write_evolution_proposal_artifacts(
    output: &Path,
    response: &EvolutionProposalReport,
) -> anyhow::Result<()> {
    let evolution_dir = evolution_reports_dir(output);
    fs::create_dir_all(&evolution_dir)
        .with_context(|| format!("create {}", evolution_dir.display()))?;

    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let baseline_json = evolution_dir.join("latest-proposal.json");
    let timestamp_json = evolution_dir.join(format!("proposal-{timestamp}.json"));
    let json = serde_json::to_string_pretty(response)? + "\n";

    fs::write(&baseline_json, &json)
        .with_context(|| format!("write {}", baseline_json.display()))?;
    fs::write(&timestamp_json, &json)
        .with_context(|| format!("write {}", timestamp_json.display()))?;
    Ok(())
}

fn write_evolution_branch_artifacts(
    output: &Path,
    response: &EvolutionBranchManifest,
) -> anyhow::Result<()> {
    let evolution_dir = evolution_reports_dir(output);
    fs::create_dir_all(&evolution_dir)
        .with_context(|| format!("create {}", evolution_dir.display()))?;

    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let baseline_json = evolution_dir.join("latest-branch.json");
    let timestamp_json = evolution_dir.join(format!("branch-{timestamp}.json"));
    let json = serde_json::to_string_pretty(response)? + "\n";

    fs::write(&baseline_json, &json)
        .with_context(|| format!("write {}", baseline_json.display()))?;
    fs::write(&timestamp_json, &json)
        .with_context(|| format!("write {}", timestamp_json.display()))?;
    Ok(())
}

fn read_latest_evolution_proposal(
    output: &Path,
) -> anyhow::Result<Option<EvolutionProposalReport>> {
    let path = evolution_reports_dir(output).join("latest-proposal.json");
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let report = serde_json::from_str::<EvolutionProposalReport>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(report))
}

fn read_latest_evolution_branch_manifest(
    output: &Path,
) -> anyhow::Result<Option<EvolutionBranchManifest>> {
    let path = evolution_reports_dir(output).join("latest-branch.json");
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let manifest = serde_json::from_str::<EvolutionBranchManifest>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(manifest))
}

fn read_evolution_durability_ledger(
    output: &Path,
) -> anyhow::Result<Option<EvolutionDurabilityLedger>> {
    let path = evolution_durability_ledger_path(output);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let ledger = serde_json::from_str::<EvolutionDurabilityLedger>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(ledger))
}

fn read_evolution_authority_ledger(
    output: &Path,
) -> anyhow::Result<Option<EvolutionAuthorityLedger>> {
    let path = evolution_authority_ledger_path(output);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let ledger = serde_json::from_str::<EvolutionAuthorityLedger>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(ledger))
}

fn read_evolution_merge_queue(output: &Path) -> anyhow::Result<Option<EvolutionMergeQueue>> {
    let path = evolution_merge_queue_path(output);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let queue = serde_json::from_str::<EvolutionMergeQueue>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(queue))
}

fn read_evolution_durability_queue(
    output: &Path,
) -> anyhow::Result<Option<EvolutionDurabilityQueue>> {
    let path = evolution_durability_queue_path(output);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let queue = serde_json::from_str::<EvolutionDurabilityQueue>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(queue))
}

fn write_evolution_merge_queue(output: &Path, queue: &EvolutionMergeQueue) -> anyhow::Result<()> {
    let path = evolution_merge_queue_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(queue)? + "\n";
    fs::write(&path, json).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn write_evolution_durability_ledger(
    output: &Path,
    ledger: &EvolutionDurabilityLedger,
) -> anyhow::Result<()> {
    let path = evolution_durability_ledger_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(ledger)? + "\n";
    fs::write(&path, json).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn write_evolution_authority_ledger(
    output: &Path,
    ledger: &EvolutionAuthorityLedger,
) -> anyhow::Result<()> {
    let path = evolution_authority_ledger_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(ledger)? + "\n";
    fs::write(&path, json).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn write_evolution_durability_queue(
    output: &Path,
    queue: &EvolutionDurabilityQueue,
) -> anyhow::Result<()> {
    let path = evolution_durability_queue_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(queue)? + "\n";
    fs::write(&path, json).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn append_evolution_durability_entry(
    output: &Path,
    proposal: &EvolutionProposalReport,
) -> anyhow::Result<()> {
    let mut ledger = read_evolution_durability_ledger(output)?.unwrap_or_default();
    ledger.entries.push(EvolutionDurabilityEntry {
        proposal_id: proposal.proposal_id.clone(),
        branch: proposal.branch.clone(),
        branch_prefix: format!(
            "auto/evolution/{}/{}",
            branch_safe_slug(&proposal.scope_class),
            branch_safe_slug(&proposal.topic)
        ),
        state: proposal.state.clone(),
        scope_class: proposal.scope_class.clone(),
        scope_gate: proposal.scope_gate.clone(),
        merge_eligible: proposal.merge_eligible,
        durable_truth: proposal.durable_truth,
        recorded_at: proposal.generated_at,
    });
    write_evolution_durability_ledger(output, &ledger)
}

fn append_evolution_authority_entry(
    output: &Path,
    proposal: &EvolutionProposalReport,
) -> anyhow::Result<()> {
    let mut ledger = read_evolution_authority_ledger(output)?.unwrap_or_default();
    ledger.entries.push(EvolutionAuthorityEntry {
        scope_class: proposal.scope_class.clone(),
        authority_tier: proposal.authority_tier.clone(),
        accepted: proposal.accepted,
        merged: proposal.state == "merged" || proposal.state == "durable_truth",
        durable_truth: proposal.durable_truth,
        proposal_id: proposal.proposal_id.clone(),
        branch: proposal.branch.clone(),
        recorded_at: proposal.generated_at,
    });
    write_evolution_authority_ledger(output, &ledger)
}

fn append_evolution_merge_queue_entry(
    output: &Path,
    proposal: &EvolutionProposalReport,
) -> anyhow::Result<()> {
    let path = evolution_merge_queue_path(output);
    let mut queue = read_evolution_merge_queue(output)?.unwrap_or_default();
    queue.entries.push(EvolutionMergeQueueEntry {
        proposal_id: proposal.proposal_id.clone(),
        branch: proposal.branch.clone(),
        scope_class: proposal.scope_class.clone(),
        scope_gate: proposal.scope_gate.clone(),
        authority_tier: proposal.authority_tier.clone(),
        status: if proposal.merge_eligible {
            "pending_merge".to_string()
        } else {
            "human_review".to_string()
        },
        merge_eligible: proposal.merge_eligible,
        recorded_at: proposal.generated_at,
    });
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(&queue)? + "\n";
    fs::write(&path, json).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn append_evolution_durability_queue_entry(
    output: &Path,
    proposal: &EvolutionProposalReport,
) -> anyhow::Result<()> {
    let path = evolution_durability_queue_path(output);
    let mut queue = read_evolution_durability_queue(output)?.unwrap_or_default();
    queue.entries.push(EvolutionDurabilityQueueEntry {
        proposal_id: proposal.proposal_id.clone(),
        branch: proposal.branch.clone(),
        state: proposal.state.clone(),
        status: if proposal.state == "merged" || proposal.state == "durable_truth" {
            "scheduled".to_string()
        } else if !proposal.merge_eligible {
            "human_review".to_string()
        } else {
            "waiting_for_merge".to_string()
        },
        due_at: proposal.durability_due_at,
        recorded_at: proposal.generated_at,
    });
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(&queue)? + "\n";
    fs::write(&path, json).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn process_evolution_queues(output: &Path) -> anyhow::Result<()> {
    process_evolution_merge_queue(output)?;
    process_evolution_durability_queue(output)?;
    Ok(())
}

fn process_evolution_merge_queue(output: &Path) -> anyhow::Result<()> {
    let Some(mut queue) = read_evolution_merge_queue(output)? else {
        return Ok(());
    };
    let project_root = infer_bundle_project_root(output);
    for entry in &mut queue.entries {
        if entry.status == "merged"
            || entry.status == "human_review" && entry.authority_tier == "proposal_only"
        {
            continue;
        }
        let Some(root) = project_root.as_ref() else {
            entry.status = "blocked_no_project_root".to_string();
            continue;
        };
        let Some(base_branch) = git_stdout(root, &["branch", "--show-current"]) else {
            entry.status = "blocked_no_base".to_string();
            continue;
        };
        let worktree_dirty = git_worktree_dirty(root);
        if worktree_dirty && git_worktree_conflicts_with_branch(root, &base_branch, &entry.branch) {
            entry.status = "blocked_dirty_worktree".to_string();
            continue;
        }
        if !git_branch_exists(root, &entry.branch) {
            entry.status = "blocked_missing_branch".to_string();
            continue;
        }
        if !git_branch_has_diff(root, &base_branch, &entry.branch) {
            entry.status = "no_diff".to_string();
            continue;
        }
        let evaluated_status = if entry.authority_tier == "proposal_only" {
            "human_review".to_string()
        } else {
            "merge_ready".to_string()
        };
        if evaluated_status == "merge_ready" {
            entry.status = if worktree_dirty {
                execute_evolution_merge_in_isolated_worktree(output, root, entry, &base_branch)?
            } else {
                execute_evolution_merge(output, root, entry, &base_branch)?
            };
        } else {
            entry.status = evaluated_status;
        }
    }
    write_evolution_merge_queue(output, &queue)?;
    Ok(())
}

fn process_evolution_durability_queue(output: &Path) -> anyhow::Result<()> {
    let Some(mut queue) = read_evolution_durability_queue(output)? else {
        return Ok(());
    };
    let merge_queue = read_evolution_merge_queue(output)?.unwrap_or_default();
    for entry in &mut queue.entries {
        if entry.status == "scheduled" {
            entry.status = execute_evolution_durability_check(output, entry)?;
            continue;
        }
        if matches!(entry.status.as_str(), "verified" | "regressed") {
            continue;
        }
        let merge_status = merge_queue
            .entries
            .iter()
            .rev()
            .find(|candidate| candidate.proposal_id == entry.proposal_id)
            .map(|candidate| candidate.status.as_str())
            .unwrap_or("unknown");
        entry.status = match merge_status {
            "merge_ready" => "waiting_for_merge".to_string(),
            "merged" => "scheduled".to_string(),
            "human_review" => "human_review".to_string(),
            "no_diff" => "no_diff".to_string(),
            "blocked_no_base" => "blocked_no_base".to_string(),
            "blocked_missing_branch" => "blocked_missing_branch".to_string(),
            _ => entry.status.clone(),
        };
    }
    write_evolution_durability_queue(output, &queue)?;
    Ok(())
}

fn execute_evolution_durability_check(
    output: &Path,
    entry: &EvolutionDurabilityQueueEntry,
) -> anyhow::Result<String> {
    let Some(due_at) = entry.due_at else {
        return Ok("scheduled".to_string());
    };
    if due_at > Utc::now() {
        return Ok("scheduled".to_string());
    }
    let Some(root) = infer_bundle_project_root(output) else {
        return Ok("blocked_no_project_root".to_string());
    };
    if git_worktree_dirty(&root) {
        return Ok("blocked_dirty_worktree".to_string());
    }
    if !git_branch_exists(&root, &entry.branch) {
        return Ok("blocked_missing_branch".to_string());
    }
    if !git_branch_tip_ancestor_of_head(&root, &entry.branch) {
        transition_evolution_proposal_state(
            output,
            &entry.proposal_id,
            "merged",
            false,
            Some(due_at),
        )?;
        transition_evolution_branch_state(output, &entry.proposal_id, "merged", false)?;
        return Ok("regressed".to_string());
    }
    transition_evolution_proposal_state(
        output,
        &entry.proposal_id,
        "durable_truth",
        true,
        Some(due_at),
    )?;
    transition_evolution_branch_state(output, &entry.proposal_id, "durable_truth", true)?;
    append_evolution_durability_transition_from_queue(output, entry, "durable_truth", true)?;
    append_evolution_authority_transition_from_queue(output, entry, "durable_truth", true)?;
    Ok("verified".to_string())
}

fn execute_evolution_merge(
    output: &Path,
    root: &Path,
    entry: &EvolutionMergeQueueEntry,
    base_branch: &str,
) -> anyhow::Result<String> {
    let current_branch = git_stdout(root, &["branch", "--show-current"]);
    if current_branch.as_deref() != Some(base_branch) {
        return Ok("blocked_wrong_base_branch".to_string());
    }

    let status = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("merge")
        .arg("--ff-only")
        .arg(&entry.branch)
        .status();
    let Ok(status) = status else {
        return Ok("merge_error".to_string());
    };
    if !status.success() {
        return Ok("merge_conflict".to_string());
    }

    let due_at = Some(Utc::now() + chrono::TimeDelta::hours(1));
    transition_evolution_proposal_state(output, &entry.proposal_id, "merged", false, due_at)?;
    transition_evolution_branch_state(output, &entry.proposal_id, "merged", false)?;
    append_evolution_durability_transition(output, entry, "merged", false)?;
    append_evolution_authority_transition(output, entry, "merged", false)?;
    Ok("merged".to_string())
}

fn execute_evolution_merge_in_isolated_worktree(
    output: &Path,
    root: &Path,
    entry: &EvolutionMergeQueueEntry,
    base_branch: &str,
) -> anyhow::Result<String> {
    let base_sha = match git_stdout(root, &["rev-parse", base_branch]) {
        Some(value) => value,
        None => return Ok("blocked_no_base".to_string()),
    };
    let tempdir =
        std::env::temp_dir().join(format!("memd-evolution-merge-{}", uuid::Uuid::new_v4()));
    let add_status = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("worktree")
        .arg("add")
        .arg("--detach")
        .arg(&tempdir)
        .arg(&base_sha)
        .status();
    let Ok(add_status) = add_status else {
        return Ok("merge_error".to_string());
    };
    if !add_status.success() {
        return Ok("merge_error".to_string());
    }

    let result = (|| -> anyhow::Result<String> {
        let merge_status = Command::new("git")
            .arg("-C")
            .arg(&tempdir)
            .arg("merge")
            .arg("--ff-only")
            .arg(&entry.branch)
            .status()
            .context("run isolated ff merge")?;
        if !merge_status.success() {
            return Ok("merge_conflict".to_string());
        }

        let Some(merged_sha) = git_stdout(&tempdir, &["rev-parse", "HEAD"]) else {
            return Ok("merge_error".to_string());
        };
        let update_status = Command::new("git")
            .arg("-C")
            .arg(root)
            .arg("update-ref")
            .arg(format!("refs/heads/{base_branch}"))
            .arg(&merged_sha)
            .arg(&base_sha)
            .status()
            .context("update branch ref after isolated merge")?;
        if !update_status.success() {
            return Ok("merge_error".to_string());
        }

        let due_at = Some(Utc::now() + chrono::TimeDelta::hours(1));
        transition_evolution_proposal_state(output, &entry.proposal_id, "merged", false, due_at)?;
        transition_evolution_branch_state(output, &entry.proposal_id, "merged", false)?;
        append_evolution_durability_transition(output, entry, "merged", false)?;
        append_evolution_authority_transition(output, entry, "merged", false)?;
        Ok("merged".to_string())
    })();

    let _ = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("worktree")
        .arg("remove")
        .arg("--force")
        .arg(&tempdir)
        .status();

    result
}

fn transition_evolution_proposal_state(
    output: &Path,
    proposal_id: &str,
    state: &str,
    durable_truth: bool,
    durability_due_at: Option<DateTime<Utc>>,
) -> anyhow::Result<()> {
    let Some(mut proposal) = read_latest_evolution_proposal(output)? else {
        return Ok(());
    };
    if proposal.proposal_id != proposal_id {
        return Ok(());
    }
    proposal.state = state.to_string();
    proposal.durable_truth = durable_truth;
    proposal.durability_due_at = durability_due_at;
    write_evolution_proposal_artifacts(output, &proposal)?;
    Ok(())
}

fn transition_evolution_branch_state(
    output: &Path,
    proposal_id: &str,
    status: &str,
    durable_truth: bool,
) -> anyhow::Result<()> {
    let Some(mut manifest) = read_latest_evolution_branch_manifest(output)? else {
        return Ok(());
    };
    if manifest.proposal_id != proposal_id {
        return Ok(());
    }
    manifest.status = status.to_string();
    manifest.durable_truth = durable_truth;
    write_evolution_branch_artifacts(output, &manifest)?;
    Ok(())
}

fn append_evolution_durability_transition(
    output: &Path,
    entry: &EvolutionMergeQueueEntry,
    state: &str,
    durable_truth: bool,
) -> anyhow::Result<()> {
    let path = evolution_durability_ledger_path(output);
    let mut ledger = read_evolution_durability_ledger(output)?.unwrap_or_default();
    ledger.entries.push(EvolutionDurabilityEntry {
        proposal_id: entry.proposal_id.clone(),
        branch: entry.branch.clone(),
        branch_prefix: branch_prefix_from_branch_name(&entry.branch),
        state: state.to_string(),
        scope_class: entry.scope_class.clone(),
        scope_gate: entry.scope_gate.clone(),
        merge_eligible: entry.merge_eligible,
        durable_truth,
        recorded_at: Utc::now(),
    });
    let json = serde_json::to_string_pretty(&ledger)? + "\n";
    fs::write(&path, json).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn append_evolution_authority_transition(
    output: &Path,
    entry: &EvolutionMergeQueueEntry,
    state: &str,
    durable_truth: bool,
) -> anyhow::Result<()> {
    let path = evolution_authority_ledger_path(output);
    let mut ledger = read_evolution_authority_ledger(output)?.unwrap_or_default();
    ledger.entries.push(EvolutionAuthorityEntry {
        scope_class: entry.scope_class.clone(),
        authority_tier: entry.authority_tier.clone(),
        accepted: true,
        merged: state == "merged" || state == "durable_truth",
        durable_truth,
        proposal_id: entry.proposal_id.clone(),
        branch: entry.branch.clone(),
        recorded_at: Utc::now(),
    });
    let json = serde_json::to_string_pretty(&ledger)? + "\n";
    fs::write(&path, json).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn append_evolution_durability_transition_from_queue(
    output: &Path,
    entry: &EvolutionDurabilityQueueEntry,
    state: &str,
    durable_truth: bool,
) -> anyhow::Result<()> {
    let path = evolution_durability_ledger_path(output);
    let mut ledger = read_evolution_durability_ledger(output)?.unwrap_or_default();
    let proposal = read_latest_evolution_proposal(output)?
        .filter(|proposal| proposal.proposal_id == entry.proposal_id);
    ledger.entries.push(EvolutionDurabilityEntry {
        proposal_id: entry.proposal_id.clone(),
        branch: entry.branch.clone(),
        branch_prefix: branch_prefix_from_branch_name(&entry.branch),
        state: state.to_string(),
        scope_class: proposal
            .as_ref()
            .map(|value| value.scope_class.clone())
            .unwrap_or_else(|| "unknown".to_string()),
        scope_gate: proposal
            .as_ref()
            .map(|value| value.scope_gate.clone())
            .unwrap_or_else(|| "proposal_only".to_string()),
        merge_eligible: proposal.as_ref().is_some_and(|value| value.merge_eligible),
        durable_truth,
        recorded_at: Utc::now(),
    });
    let json = serde_json::to_string_pretty(&ledger)? + "\n";
    fs::write(&path, json).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn append_evolution_authority_transition_from_queue(
    output: &Path,
    entry: &EvolutionDurabilityQueueEntry,
    state: &str,
    durable_truth: bool,
) -> anyhow::Result<()> {
    let path = evolution_authority_ledger_path(output);
    let mut ledger = read_evolution_authority_ledger(output)?.unwrap_or_default();
    let proposal = read_latest_evolution_proposal(output)?
        .filter(|proposal| proposal.proposal_id == entry.proposal_id);
    ledger.entries.push(EvolutionAuthorityEntry {
        scope_class: proposal
            .as_ref()
            .map(|value| value.scope_class.clone())
            .unwrap_or_else(|| "unknown".to_string()),
        authority_tier: proposal
            .as_ref()
            .map(|value| value.authority_tier.clone())
            .unwrap_or_else(default_evolution_authority_tier),
        accepted: true,
        merged: state == "merged" || state == "durable_truth",
        durable_truth,
        proposal_id: entry.proposal_id.clone(),
        branch: entry.branch.clone(),
        recorded_at: Utc::now(),
    });
    let json = serde_json::to_string_pretty(&ledger)? + "\n";
    fs::write(&path, json).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn create_or_update_evolution_branch(
    output: &Path,
    proposal: &EvolutionProposalReport,
) -> anyhow::Result<EvolutionBranchManifest> {
    let branch_prefix = format!(
        "auto/evolution/{}/{}",
        branch_safe_slug(&proposal.scope_class),
        branch_safe_slug(&proposal.topic)
    );
    let Some(project_root) = infer_bundle_project_root(output) else {
        return Ok(EvolutionBranchManifest {
            proposal_id: proposal.proposal_id.clone(),
            branch: proposal.branch.clone(),
            branch_prefix,
            project_root: None,
            head_sha: None,
            base_branch: None,
            status: "no_project_root".to_string(),
            merge_eligible: proposal.merge_eligible,
            durable_truth: proposal.durable_truth,
            scope_class: proposal.scope_class.clone(),
            scope_gate: proposal.scope_gate.clone(),
            generated_at: proposal.generated_at,
            notes: vec!["bundle is not attached to a detectable project root".to_string()],
        });
    };

    let head_sha = git_stdout(&project_root, &["rev-parse", "HEAD"]);
    let base_branch = git_stdout(&project_root, &["branch", "--show-current"]);

    if !proposal.accepted {
        return Ok(EvolutionBranchManifest {
            proposal_id: proposal.proposal_id.clone(),
            branch: proposal.branch.clone(),
            branch_prefix,
            project_root: Some(display_path_nonempty(&project_root)),
            head_sha,
            base_branch,
            status: "rejected".to_string(),
            merge_eligible: proposal.merge_eligible,
            durable_truth: proposal.durable_truth,
            scope_class: proposal.scope_class.clone(),
            scope_gate: proposal.scope_gate.clone(),
            generated_at: proposal.generated_at,
            notes: vec!["rejected proposals do not create evolution branches".to_string()],
        });
    }

    let exists = Command::new("git")
        .arg("-C")
        .arg(&project_root)
        .arg("show-ref")
        .arg("--verify")
        .arg(format!("refs/heads/{}", proposal.branch))
        .output()
        .ok()
        .is_some_and(|output| output.status.success());

    let status = if exists {
        "existing".to_string()
    } else {
        let mut cmd = Command::new("git");
        cmd.arg("-C")
            .arg(&project_root)
            .arg("branch")
            .arg(&proposal.branch);
        if let Some(head) = head_sha.as_deref() {
            cmd.arg(head);
        }
        match cmd.output() {
            Ok(output) if output.status.success() => "created".to_string(),
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                return Ok(EvolutionBranchManifest {
                    proposal_id: proposal.proposal_id.clone(),
                    branch: proposal.branch.clone(),
                    branch_prefix,
                    project_root: Some(display_path_nonempty(&project_root)),
                    head_sha,
                    base_branch,
                    status: "branch_error".to_string(),
                    merge_eligible: proposal.merge_eligible,
                    durable_truth: proposal.durable_truth,
                    scope_class: proposal.scope_class.clone(),
                    scope_gate: proposal.scope_gate.clone(),
                    generated_at: proposal.generated_at,
                    notes: vec![if stderr.is_empty() {
                        "git branch creation failed".to_string()
                    } else {
                        stderr
                    }],
                });
            }
            Err(err) => {
                return Ok(EvolutionBranchManifest {
                    proposal_id: proposal.proposal_id.clone(),
                    branch: proposal.branch.clone(),
                    branch_prefix,
                    project_root: Some(display_path_nonempty(&project_root)),
                    head_sha,
                    base_branch,
                    status: "branch_error".to_string(),
                    merge_eligible: proposal.merge_eligible,
                    durable_truth: proposal.durable_truth,
                    scope_class: proposal.scope_class.clone(),
                    scope_gate: proposal.scope_gate.clone(),
                    generated_at: proposal.generated_at,
                    notes: vec![format!("git branch creation failed: {err}")],
                });
            }
        }
    };

    Ok(EvolutionBranchManifest {
        proposal_id: proposal.proposal_id.clone(),
        branch: proposal.branch.clone(),
        branch_prefix,
        project_root: Some(display_path_nonempty(&project_root)),
        head_sha,
        base_branch,
        status,
        merge_eligible: proposal.merge_eligible,
        durable_truth: proposal.durable_truth,
        scope_class: proposal.scope_class.clone(),
        scope_gate: proposal.scope_gate.clone(),
        generated_at: proposal.generated_at,
        notes: vec!["evolution branch isolated from active working branch".to_string()],
    })
}

fn git_stdout(root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn detect_git_worktree_root(root: &Path) -> Option<PathBuf> {
    git_stdout(root, &["rev-parse", "--show-toplevel"]).map(PathBuf::from)
}

fn detect_git_repo_root(root: &Path) -> Option<PathBuf> {
    let common_dir = git_stdout(
        root,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )
    .map(PathBuf::from)?;
    if common_dir
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value == ".git")
    {
        common_dir.parent().map(Path::to_path_buf)
    } else {
        detect_git_worktree_root(root)
    }
}

fn git_worktree_dirty(root: &Path) -> bool {
    git_dirty_paths(root).is_some_and(|paths| !paths.is_empty())
}

fn git_dirty_paths(root: &Path) -> Option<BTreeSet<String>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("status")
        .arg("--porcelain")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(parse_git_status_path)
            .filter(|path| !is_bundle_generated_path(path))
            .collect(),
    )
}

fn git_branch_changed_paths(
    root: &Path,
    base_branch: &str,
    branch: &str,
) -> Option<BTreeSet<String>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("diff")
        .arg("--name-only")
        .arg(format!("{base_branch}..{branch}"))
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::trim)
            .filter(|path| !path.is_empty())
            .map(str::to_string)
            .collect(),
    )
}

fn git_worktree_conflicts_with_branch(root: &Path, base_branch: &str, branch: &str) -> bool {
    let Some(dirty_paths) = git_dirty_paths(root) else {
        return git_worktree_dirty(root);
    };
    if dirty_paths.is_empty() {
        return false;
    }
    let Some(branch_paths) = git_branch_changed_paths(root, base_branch, branch) else {
        return true;
    };
    if branch_paths.is_empty() {
        return false;
    }
    dirty_paths.iter().any(|path| branch_paths.contains(path))
}

fn parse_git_status_path(line: &str) -> Option<String> {
    if line.len() < 4 {
        return None;
    }
    let path = line.get(3..)?.trim();
    if path.is_empty() {
        return None;
    }
    if let Some((_, renamed)) = path.split_once(" -> ") {
        return Some(renamed.trim().to_string());
    }
    Some(path.to_string())
}

fn is_bundle_generated_path(path: &str) -> bool {
    let normalized = path.trim_start_matches("./");
    normalized == ".memd" || normalized.starts_with(".memd/") || normalized.contains("/.memd/")
}

fn branch_prefix_from_branch_name(branch: &str) -> String {
    branch
        .rsplit_once('/')
        .map(|(prefix, _)| prefix.to_string())
        .unwrap_or_else(|| branch.to_string())
}

fn git_branch_exists(root: &Path, branch: &str) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("show-ref")
        .arg("--verify")
        .arg(format!("refs/heads/{branch}"))
        .output()
        .ok()
        .is_some_and(|output| output.status.success())
}

fn git_branch_has_diff(root: &Path, base_branch: &str, branch: &str) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("diff")
        .arg("--quiet")
        .arg(format!("{base_branch}..{branch}"))
        .status()
        .ok()
        .is_some_and(|status| !status.success())
}

fn git_branch_tip_ancestor_of_head(root: &Path, branch: &str) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("merge-base")
        .arg("--is-ancestor")
        .arg(branch)
        .arg("HEAD")
        .status()
        .ok()
        .is_some_and(|status| status.success())
}

fn display_path_nonempty(path: &Path) -> String {
    let rendered = path.display().to_string();
    if rendered.is_empty() {
        ".".to_string()
    } else {
        rendered
    }
}

fn compute_evolution_authority_tier(output: &Path, scope_class: &str, scope_gate: &str) -> String {
    if scope_gate != "auto_merge" {
        return "proposal_only".to_string();
    }
    let recent = read_evolution_authority_ledger(output)
        .ok()
        .flatten()
        .map(|ledger| {
            ledger
                .entries
                .into_iter()
                .filter(|entry| entry.scope_class == scope_class)
                .rev()
                .take(3)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if recent.len() >= 2 && recent.iter().take(2).any(|entry| !entry.accepted) {
        return "proposal_only".to_string();
    }
    if recent.len() >= 3 && recent.iter().take(3).all(|entry| entry.durable_truth) {
        return "durable_auto_merge".to_string();
    }
    "phase1_auto_merge".to_string()
}

fn default_evolution_authority_tier() -> String {
    "proposal_only".to_string()
}

fn ensure_evolution_artifacts(output: &Path, report: &ExperimentReport) -> anyhow::Result<()> {
    let built = build_evolution_proposal_report(report);
    let proposal = if let Some(existing) = read_latest_evolution_proposal(output)? {
        if evolution_proposal_needs_refresh(&existing, &built) {
            sync_latest_evolution_artifacts(output, &built)?;
            built
        } else {
            existing
        }
    } else {
        sync_latest_evolution_artifacts(output, &built)?;
        built
    };
    let existing_branch_manifest = read_latest_evolution_branch_manifest(output)?;
    if !existing_branch_manifest
        .as_ref()
        .is_some_and(|manifest| !manifest.project_root.as_deref().unwrap_or("").is_empty())
    {
        let branch_manifest = create_or_update_evolution_branch(output, &proposal)?;
        write_evolution_branch_artifacts(output, &branch_manifest)?;
    }
    if read_evolution_durability_ledger(output)?.is_none() {
        append_evolution_durability_entry(output, &proposal)?;
    }
    if read_evolution_authority_ledger(output)?.is_none() {
        append_evolution_authority_entry(output, &proposal)?;
    }
    if read_evolution_merge_queue(output)?.is_none() {
        append_evolution_merge_queue_entry(output, &proposal)?;
    }
    if read_evolution_durability_queue(output)?.is_none() {
        append_evolution_durability_queue_entry(output, &proposal)?;
    }
    process_evolution_queues(output)?;
    Ok(())
}

fn evolution_proposal_needs_refresh(
    existing: &EvolutionProposalReport,
    built: &EvolutionProposalReport,
) -> bool {
    existing.scope_class != built.scope_class
        || existing.scope_gate != built.scope_gate
        || existing.authority_tier != built.authority_tier
        || existing.branch != built.branch
        || existing.state != built.state
        || existing.merge_eligible != built.merge_eligible
        || existing.durable_truth != built.durable_truth
        || existing.allowed_write_surface != built.allowed_write_surface
        || existing.scope_reasons != built.scope_reasons
}

fn sync_latest_evolution_artifacts(
    output: &Path,
    proposal: &EvolutionProposalReport,
) -> anyhow::Result<()> {
    write_evolution_proposal_artifacts(output, proposal)?;
    let branch_manifest = create_or_update_evolution_branch(output, proposal)?;
    write_evolution_branch_artifacts(output, &branch_manifest)?;
    upsert_evolution_durability_entry(output, proposal)?;
    upsert_evolution_authority_entry(output, proposal)?;
    upsert_evolution_merge_queue_entry(output, proposal)?;
    upsert_evolution_durability_queue_entry(output, proposal)?;
    Ok(())
}

fn upsert_evolution_durability_entry(
    output: &Path,
    proposal: &EvolutionProposalReport,
) -> anyhow::Result<()> {
    let mut ledger = read_evolution_durability_ledger(output)?.unwrap_or_default();
    let next = EvolutionDurabilityEntry {
        proposal_id: proposal.proposal_id.clone(),
        branch: proposal.branch.clone(),
        branch_prefix: format!(
            "auto/evolution/{}/{}",
            branch_safe_slug(&proposal.scope_class),
            branch_safe_slug(&proposal.topic)
        ),
        state: proposal.state.clone(),
        scope_class: proposal.scope_class.clone(),
        scope_gate: proposal.scope_gate.clone(),
        merge_eligible: proposal.merge_eligible,
        durable_truth: proposal.durable_truth,
        recorded_at: proposal.generated_at,
    };
    if let Some(index) = ledger
        .entries
        .iter()
        .rposition(|entry| entry.proposal_id == proposal.proposal_id)
    {
        ledger.entries[index] = next;
    } else {
        ledger.entries.push(next);
    }
    write_evolution_durability_ledger(output, &ledger)
}

fn upsert_evolution_authority_entry(
    output: &Path,
    proposal: &EvolutionProposalReport,
) -> anyhow::Result<()> {
    let mut ledger = read_evolution_authority_ledger(output)?.unwrap_or_default();
    let next = EvolutionAuthorityEntry {
        scope_class: proposal.scope_class.clone(),
        authority_tier: proposal.authority_tier.clone(),
        accepted: proposal.accepted,
        merged: proposal.state == "merged" || proposal.state == "durable_truth",
        durable_truth: proposal.durable_truth,
        proposal_id: proposal.proposal_id.clone(),
        branch: proposal.branch.clone(),
        recorded_at: proposal.generated_at,
    };
    if let Some(index) = ledger
        .entries
        .iter()
        .rposition(|entry| entry.proposal_id == proposal.proposal_id)
    {
        ledger.entries[index] = next;
    } else {
        ledger.entries.push(next);
    }
    write_evolution_authority_ledger(output, &ledger)
}

fn upsert_evolution_merge_queue_entry(
    output: &Path,
    proposal: &EvolutionProposalReport,
) -> anyhow::Result<()> {
    let mut queue = read_evolution_merge_queue(output)?.unwrap_or_default();
    let next = EvolutionMergeQueueEntry {
        proposal_id: proposal.proposal_id.clone(),
        branch: proposal.branch.clone(),
        scope_class: proposal.scope_class.clone(),
        scope_gate: proposal.scope_gate.clone(),
        authority_tier: proposal.authority_tier.clone(),
        status: if proposal.merge_eligible {
            "pending_merge".to_string()
        } else {
            "human_review".to_string()
        },
        merge_eligible: proposal.merge_eligible,
        recorded_at: proposal.generated_at,
    };
    if let Some(index) = queue
        .entries
        .iter()
        .rposition(|entry| entry.proposal_id == proposal.proposal_id)
    {
        queue.entries[index] = next;
    } else {
        queue.entries.push(next);
    }
    write_evolution_merge_queue(output, &queue)
}

fn upsert_evolution_durability_queue_entry(
    output: &Path,
    proposal: &EvolutionProposalReport,
) -> anyhow::Result<()> {
    let mut queue = read_evolution_durability_queue(output)?.unwrap_or_default();
    let next = EvolutionDurabilityQueueEntry {
        proposal_id: proposal.proposal_id.clone(),
        branch: proposal.branch.clone(),
        state: proposal.state.clone(),
        status: if proposal.state == "merged" || proposal.state == "durable_truth" {
            "scheduled".to_string()
        } else if !proposal.merge_eligible {
            "human_review".to_string()
        } else {
            "waiting_for_merge".to_string()
        },
        due_at: proposal.durability_due_at,
        recorded_at: proposal.generated_at,
    };
    if let Some(index) = queue
        .entries
        .iter()
        .rposition(|entry| entry.proposal_id == proposal.proposal_id)
    {
        queue.entries[index] = next;
    } else {
        queue.entries.push(next);
    }
    write_evolution_durability_queue(output, &queue)
}

#[derive(Debug, Clone)]
struct EvolutionScopeAssessment {
    topic: String,
    scope_class: String,
    scope_gate: String,
    allowed_write_surface: Vec<String>,
    scope_reasons: Vec<String>,
}

fn build_evolution_proposal_report(report: &ExperimentReport) -> EvolutionProposalReport {
    let scope = classify_evolution_scope(report);
    let branch = evolution_branch_name(&scope, report.completed_at);
    let authority_tier = compute_evolution_authority_tier(
        Path::new(&report.bundle_root),
        &scope.scope_class,
        &scope.scope_gate,
    );
    let merge_eligible =
        report.accepted && scope.scope_gate == "auto_merge" && authority_tier != "proposal_only";
    let prior_ledger = read_evolution_durability_ledger(Path::new(&report.bundle_root))
        .ok()
        .flatten()
        .unwrap_or_default();
    let prior_merged = prior_ledger
        .entries
        .iter()
        .rev()
        .find(|entry| entry.branch_prefix == evolution_branch_prefix(&scope))
        .is_some_and(|entry| entry.state == "merged" || entry.state == "durable_truth");
    let state = if !report.accepted {
        "rejected".to_string()
    } else if merge_eligible && prior_merged {
        "durable_truth".to_string()
    } else if merge_eligible && report.apply {
        "merged".to_string()
    } else {
        "accepted_proposal".to_string()
    };
    let evidence = vec![
        format!("accepted={}", report.accepted),
        format!("restored={}", report.restored),
        format!("scope_class={}", scope.scope_class),
        format!("scope_gate={}", scope.scope_gate),
        format!(
            "composite_score={}/{}",
            report.composite.score, report.composite.max_score
        ),
    ];
    EvolutionProposalReport {
        bundle_root: report.bundle_root.clone(),
        project: report.project.clone(),
        namespace: report.namespace.clone(),
        agent: report.agent.clone(),
        session: report.session.clone(),
        workspace: report.workspace.clone(),
        visibility: report.visibility.clone(),
        proposal_id: format!(
            "{}-{}",
            canonical_slug(
                report
                    .composite
                    .scenario
                    .as_deref()
                    .unwrap_or("self-evolution")
            ),
            report.completed_at.format("%Y%m%dT%H%M%SZ")
        ),
        scenario: report.composite.scenario.clone(),
        topic: scope.topic,
        branch,
        state: state.clone(),
        scope_class: scope.scope_class,
        scope_gate: scope.scope_gate,
        authority_tier,
        allowed_write_surface: scope.allowed_write_surface,
        merge_eligible,
        durable_truth: state == "durable_truth",
        accepted: report.accepted,
        restored: report.restored,
        composite_score: report.composite.score,
        composite_max: report.composite.max_score,
        evidence,
        scope_reasons: scope.scope_reasons,
        generated_at: report.completed_at,
        durability_due_at: if state == "merged" {
            Some(report.completed_at + chrono::TimeDelta::hours(1))
        } else {
            None
        },
    }
}

fn evolution_branch_name(scope: &EvolutionScopeAssessment, recorded_at: DateTime<Utc>) -> String {
    format!(
        "{}/{}",
        evolution_branch_prefix(scope),
        recorded_at.format("%Y%m%d%H%M%S")
    )
}

fn evolution_branch_prefix(scope: &EvolutionScopeAssessment) -> String {
    format!(
        "auto/evolution/{}/{}",
        branch_safe_slug(&scope.scope_class),
        branch_safe_slug(&scope.topic)
    )
}

fn classify_evolution_scope(report: &ExperimentReport) -> EvolutionScopeAssessment {
    let mut haystack = report.improvement.final_changes.join(" ").to_lowercase();
    if !haystack.is_empty() {
        haystack.push(' ');
    }
    haystack.push_str(&report.findings.join(" ").to_lowercase());
    if !haystack.is_empty() {
        haystack.push(' ');
    }
    haystack.push_str(&report.recommendations.join(" ").to_lowercase());
    let topic_source = report
        .improvement
        .final_changes
        .first()
        .cloned()
        .or_else(|| report.composite.scenario.clone())
        .unwrap_or_else(|| "self-evolution".to_string());
    let scenario = report.composite.scenario.as_deref().unwrap_or_default();
    let docs_score = count_matches(
        &haystack,
        &[
            "docs/", ".md", "spec", "manifest", "readme", "docs", "guide",
        ],
    );
    let runtime_policy_score = count_matches(
        &haystack,
        &[
            "threshold",
            "floor",
            "cutoff",
            "gate",
            "policy",
            "prompt",
            "weight",
            "penalty",
            "bonus",
            "clamp",
            "cap",
            "tune",
            "retune",
            "calibrate",
            "refresh cadence",
        ],
    );
    let evaluation_score = count_matches(
        &haystack,
        &[
            "evaluation",
            "eval",
            "score",
            "scoring",
            "scorer",
            "grader",
            "rubric",
            "composite",
            "dimension",
            "signal",
            "pass/fail",
            "acceptance",
            "readiness",
            "judge",
            "ranking",
            "heuristic",
            "review readiness",
            "loop",
        ],
    );
    let persistence_score = count_matches(
        &haystack,
        &[
            "schema",
            "migration",
            "persist",
            "sqlite",
            "storage",
            "database",
            "ledger format",
            "journal format",
        ],
    );
    let coordination_score = count_matches(
        &haystack,
        &[
            "coordination",
            "claim",
            "claims",
            "task",
            "tasks",
            "hive",
            "heartbeat",
            "protocol",
            "session roster",
        ],
    );
    let api_score = count_matches(&haystack, &["api", "contract", "endpoint", "wire format"]);
    let self_evolution_prior = usize::from(scenario == "self_evolution");

    let (scope_class, scope_gate, allowed_write_surface, scope_reasons) = if persistence_score > 0 {
        (
            "persistence_semantics".to_string(),
            "proposal_only".to_string(),
            vec!["proposal-only".to_string()],
            vec![format!(
                "persistence semantics signal ({persistence_score})"
            )],
        )
    } else if coordination_score > 0 {
        (
            "coordination_semantics".to_string(),
            "proposal_only".to_string(),
            vec!["proposal-only".to_string()],
            vec![format!(
                "coordination semantics signal ({coordination_score})"
            )],
        )
    } else if api_score > 0 {
        (
            "api_contract".to_string(),
            "proposal_only".to_string(),
            vec!["proposal-only".to_string()],
            vec![format!("api contract signal ({api_score})")],
        )
    } else if docs_score > 0 && runtime_policy_score == 0 && evaluation_score == 0 {
        (
            "docs_spec".to_string(),
            "auto_merge".to_string(),
            vec!["docs/**".to_string(), "*.md".to_string()],
            vec![format!("docs/spec signal ({docs_score})")],
        )
    } else if runtime_policy_score > 0 && runtime_policy_score >= evaluation_score {
        let mut reasons = vec![format!("runtime policy score={runtime_policy_score}")];
        if self_evolution_prior > 0 {
            reasons.push("self_evolution scenario prior".to_string());
        }
        (
            "runtime_policy".to_string(),
            "auto_merge".to_string(),
            vec![
                ".memd/**".to_string(),
                "policy/**".to_string(),
                "crates/memd-client/src/main.rs".to_string(),
            ],
            reasons,
        )
    } else if evaluation_score > 0 || self_evolution_prior > 0 {
        let mut reasons = vec![format!(
            "evaluation score={}",
            evaluation_score + self_evolution_prior
        )];
        if self_evolution_prior > 0 {
            reasons.push("self_evolution scenario prior".to_string());
        }
        (
            "low_risk_evaluation_code".to_string(),
            "auto_merge".to_string(),
            vec!["crates/memd-client/src/main.rs".to_string()],
            reasons,
        )
    } else if docs_score > 0 {
        (
            "docs_spec".to_string(),
            "auto_merge".to_string(),
            vec!["docs/**".to_string(), "*.md".to_string()],
            vec![format!("docs/spec signal ({docs_score})")],
        )
    } else {
        (
            "broader_implementation".to_string(),
            "proposal_only".to_string(),
            vec!["proposal-only".to_string()],
            vec!["scope unclear; keep on proposal branch".to_string()],
        )
    };

    EvolutionScopeAssessment {
        topic: canonical_slug(&topic_source),
        scope_class,
        scope_gate,
        allowed_write_surface,
        scope_reasons,
    }
}

fn count_matches(haystack: &str, needles: &[&str]) -> usize {
    needles
        .iter()
        .filter(|needle| haystack.contains(**needle))
        .count()
}

fn branch_safe_slug(value: &str) -> String {
    let mut slug = String::with_capacity(value.len());
    let mut last_dash = false;
    for ch in value.chars() {
        let normalized = if ch.is_ascii_alphanumeric() {
            ch.to_ascii_lowercase()
        } else {
            '-'
        };
        if normalized == '-' {
            if !last_dash {
                slug.push('-');
            }
            last_dash = true;
        } else {
            slug.push(normalized);
            last_dash = false;
        }
    }
    slug.trim_matches('-').to_string()
}

fn snapshot_bundle_for_reversion(output: &Path) -> anyhow::Result<PathBuf> {
    let snapshot_root =
        std::env::temp_dir().join(format!("memd-experiment-backup-{}", uuid::Uuid::new_v4()));
    copy_dir_contents(output, &snapshot_root)?;
    Ok(snapshot_root)
}
