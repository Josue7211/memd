use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod memory_surfaces;
pub mod skill;
pub use memory_surfaces::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryKind {
    Fact,
    Decision,
    Preference,
    Runbook,
    Procedural,
    SelfModel,
    Topology,
    Status,
    LiveTruth,
    Pattern,
    Constraint,
    Correction,
    Skill,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapabilityRecord {
    pub harness: String,
    pub kind: String,
    pub name: String,
    pub status: String,
    pub portability_class: String,
    pub source_path: String,
    pub bridge_hint: Option<String>,
    pub hash: Option<String>,
    #[serde(default)]
    pub notes: Vec<String>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub user_id: Option<String>,
    pub agent: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CapabilitySyncRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub user_id: Option<String>,
    pub agent: Option<String>,
    #[serde(default)]
    pub records: Vec<CapabilityRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CapabilityListRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub user_id: Option<String>,
    pub harness: Option<String>,
    pub kind: Option<String>,
    pub query: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySyncResponse {
    pub upserted: usize,
    pub total: usize,
    pub records: Vec<CapabilityRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityListResponse {
    pub total: usize,
    pub records: Vec<CapabilityRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccessRouteRecord {
    pub id: String,
    pub provider: String,
    pub status: String,
    pub scope: String,
    pub secret_values_stored: bool,
    pub guidance: String,
    pub source: String,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub user_id: Option<String>,
    pub agent: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccessRouteSyncRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub user_id: Option<String>,
    pub agent: Option<String>,
    #[serde(default)]
    pub routes: Vec<AccessRouteRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccessRouteListRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub user_id: Option<String>,
    pub provider: Option<String>,
    pub query: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessRouteSyncResponse {
    pub upserted: usize,
    pub total: usize,
    pub routes: Vec<AccessRouteRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessRouteListResponse {
    pub total: usize,
    pub routes: Vec<AccessRouteRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TokenSavingsRecord {
    pub id: Uuid,
    pub operation: String,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub user_id: Option<String>,
    pub agent: Option<String>,
    pub model_tier: Option<String>,
    pub intent: Option<String>,
    pub source_records: usize,
    pub baseline_input_tokens: usize,
    pub output_tokens: usize,
    pub tokens_saved: usize,
    #[serde(default)]
    pub wasted_tokens: usize,
    #[serde(default)]
    pub waste_kind: Option<String>,
    pub reason: String,
    pub ts: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenSavingsSyncRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub user_id: Option<String>,
    pub agent: Option<String>,
    #[serde(default)]
    pub records: Vec<TokenSavingsRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenSavingsListRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub user_id: Option<String>,
    pub agent: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSavingsSyncResponse {
    pub upserted: usize,
    pub total: usize,
    pub records: Vec<TokenSavingsRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSavingsListResponse {
    pub total: usize,
    pub measured_input_tokens: usize,
    pub measured_output_tokens: usize,
    pub measured_tokens_saved: usize,
    #[serde(default)]
    pub source_reuse_events: usize,
    #[serde(default)]
    pub source_reuse_tokens: usize,
    #[serde(default)]
    pub wasted_events: usize,
    #[serde(default)]
    pub wasted_tokens: usize,
    #[serde(default)]
    pub wasted_raw_reread_tokens: usize,
    #[serde(default)]
    pub wasted_giant_diff_tokens: usize,
    #[serde(default)]
    pub wasted_cache_exposure_tokens: usize,
    pub records: Vec<TokenSavingsRecord>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CaptureSource {
    Manual,
    HookAuto,
    Detector,
    Judge,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CorrectionMetadata {
    pub corrects_id: Option<Uuid>,
    pub source_turn: Option<String>,
    pub captured_by: Option<CaptureSource>,
    pub confidence: Option<f32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryScope {
    Local,
    Synced,
    Project,
    Global,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalRoute {
    Auto,
    LocalOnly,
    SyncedOnly,
    ProjectOnly,
    GlobalOnly,
    LocalFirst,
    SyncedFirst,
    ProjectFirst,
    GlobalFirst,
    All,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalIntent {
    General,
    CurrentTask,
    Decision,
    Runbook,
    Procedural,
    SelfModel,
    Topology,
    Preference,
    Fact,
    Pattern,
    Skill,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryStatus {
    Active,
    Stale,
    Superseded,
    Contested,
    Expired,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryStage {
    Candidate,
    Canonical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceQuality {
    Canonical,
    Derived,
    Synthetic,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
#[serde(rename_all = "snake_case")]
pub enum MemoryVisibility {
    #[default]
    Private,
    Workspace,
    Public,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VisibleMemoryStatus {
    Current,
    Candidate,
    Stale,
    Superseded,
    Conflicted,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VisibleMemoryProvenance {
    pub source_system: Option<String>,
    pub source_path: Option<String>,
    pub producer: Option<String>,
    pub trust_reason: String,
    pub last_verified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VisibleMemoryArtifact {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub artifact_kind: String,
    pub memory_kind: Option<MemoryKind>,
    pub scope: Option<MemoryScope>,
    pub visibility: Option<MemoryVisibility>,
    pub workspace: Option<String>,
    pub status: VisibleMemoryStatus,
    pub freshness: String,
    pub confidence: f32,
    pub provenance: VisibleMemoryProvenance,
    pub sources: Vec<String>,
    pub linked_artifact_ids: Vec<Uuid>,
    pub linked_sessions: Vec<String>,
    pub linked_agents: Vec<String>,
    pub repair_state: String,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VisibleMemoryHome {
    pub focus_artifact: VisibleMemoryArtifact,
    pub inbox_count: usize,
    pub repair_count: usize,
    pub awareness_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VisibleMemoryGraphNode {
    pub artifact_id: Uuid,
    pub title: String,
    pub artifact_kind: String,
    pub status: VisibleMemoryStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VisibleMemoryGraphEdge {
    pub from: Uuid,
    pub to: Uuid,
    pub relation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VisibleMemoryKnowledgeMap {
    pub nodes: Vec<VisibleMemoryGraphNode>,
    pub edges: Vec<VisibleMemoryGraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VisibleMemorySnapshotResponse {
    pub generated_at: DateTime<Utc>,
    pub home: VisibleMemoryHome,
    pub knowledge_map: VisibleMemoryKnowledgeMap,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VisibleMemoryUiActionKind {
    Inspect,
    Explain,
    VerifyCurrent,
    MarkStale,
    Promote,
    OpenSource,
    OpenInObsidian,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibleMemoryArtifactDetailResponse {
    pub generated_at: DateTime<Utc>,
    pub artifact: VisibleMemoryArtifact,
    pub explain: Option<ExplainMemoryResponse>,
    pub timeline: Option<TimelineMemoryResponse>,
    pub sources: SourceMemoryResponse,
    pub workspaces: WorkspaceMemoryResponse,
    pub sessions: HiveSessionsResponse,
    pub tasks: HiveTasksResponse,
    pub claims: HiveClaimsResponse,
    pub related_artifacts: Vec<VisibleMemoryArtifact>,
    pub related_map: VisibleMemoryKnowledgeMap,
    pub actions: Vec<VisibleMemoryUiActionKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibleMemoryUiActionRequest {
    pub id: Uuid,
    pub action: VisibleMemoryUiActionKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibleMemoryUiActionResponse {
    pub action: VisibleMemoryUiActionKind,
    pub artifact_id: Uuid,
    pub outcome: String,
    pub message: String,
    pub detail: Option<VisibleMemoryArtifactDetailResponse>,
    pub open_uri: Option<String>,
    pub source_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub id: Uuid,
    pub content: String,
    pub redundancy_key: Option<String>,
    pub belief_branch: Option<String>,
    pub preferred: bool,
    pub kind: MemoryKind,
    pub scope: MemoryScope,
    pub project: Option<String>,
    pub namespace: Option<String>,
    #[serde(default)]
    pub workspace: Option<String>,
    #[serde(default)]
    pub visibility: MemoryVisibility,
    pub source_agent: Option<String>,
    pub source_system: Option<String>,
    pub source_path: Option<String>,
    pub source_quality: Option<SourceQuality>,
    pub confidence: f32,
    pub ttl_seconds: Option<u64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub supersedes: Vec<Uuid>,
    pub tags: Vec<String>,
    pub status: MemoryStatus,
    pub stage: MemoryStage,
    #[serde(default)]
    pub lane: Option<String>,
    /// Lamport version — incremented on every server-side mutation.
    /// Foreign imports with `version <= stored.version` are rejected as
    /// `Conflict`, giving timestamp-independent resolution across harnesses.
    #[serde(default = "default_memory_item_version")]
    pub version: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correction_meta: Option<CorrectionMetadata>,
}

pub fn default_memory_item_version() -> u64 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreMemoryRequest {
    pub content: String,
    pub kind: MemoryKind,
    pub scope: MemoryScope,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub visibility: Option<MemoryVisibility>,
    pub belief_branch: Option<String>,
    pub source_agent: Option<String>,
    pub source_system: Option<String>,
    pub source_path: Option<String>,
    pub source_quality: Option<SourceQuality>,
    pub confidence: Option<f32>,
    pub ttl_seconds: Option<u64>,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub supersedes: Vec<Uuid>,
    pub tags: Vec<String>,
    pub status: Option<MemoryStatus>,
    #[serde(default)]
    pub lane: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreMemoryResponse {
    pub item: MemoryItem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateMemoryRequest {
    pub content: String,
    pub kind: MemoryKind,
    pub scope: MemoryScope,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub visibility: Option<MemoryVisibility>,
    pub belief_branch: Option<String>,
    pub source_agent: Option<String>,
    pub source_system: Option<String>,
    pub source_path: Option<String>,
    pub source_quality: Option<SourceQuality>,
    pub confidence: Option<f32>,
    pub ttl_seconds: Option<u64>,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub supersedes: Vec<Uuid>,
    pub tags: Vec<String>,
    #[serde(default)]
    pub lane: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateMemoryResponse {
    pub item: MemoryItem,
    pub duplicate_of: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromoteMemoryRequest {
    pub id: Uuid,
    pub scope: Option<MemoryScope>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub visibility: Option<MemoryVisibility>,
    pub belief_branch: Option<String>,
    pub confidence: Option<f32>,
    pub ttl_seconds: Option<u64>,
    pub tags: Option<Vec<String>>,
    pub status: Option<MemoryStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromoteMemoryResponse {
    pub item: MemoryItem,
    pub duplicate_of: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpireMemoryRequest {
    pub id: Uuid,
    pub status: Option<MemoryStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpireMemoryResponse {
    pub item: MemoryItem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyMemoryRequest {
    pub id: Uuid,
    pub confidence: Option<f32>,
    pub status: Option<MemoryStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyMemoryResponse {
    pub item: MemoryItem,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryRepairMode {
    Verify,
    Expire,
    Supersede,
    Contest,
    PreferBranch,
    CorrectMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairMemoryRequest {
    pub id: Uuid,
    pub mode: MemoryRepairMode,
    pub confidence: Option<f32>,
    pub status: Option<MemoryStatus>,
    pub workspace: Option<String>,
    pub visibility: Option<MemoryVisibility>,
    pub source_agent: Option<String>,
    pub source_system: Option<String>,
    pub source_path: Option<String>,
    pub source_quality: Option<SourceQuality>,
    pub content: Option<String>,
    pub tags: Option<Vec<String>>,
    pub supersedes: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairMemoryResponse {
    pub item: MemoryItem,
    pub mode: MemoryRepairMode,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectMemoryRequest {
    pub id: Uuid,
    pub content: String,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub confidence: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectMemoryResponse {
    pub old_item: MemoryItem,
    pub new_item: MemoryItem,
    pub contested: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchMemoryRequest {
    pub query: Option<String>,
    pub route: Option<RetrievalRoute>,
    pub intent: Option<RetrievalIntent>,
    pub scopes: Vec<MemoryScope>,
    pub kinds: Vec<MemoryKind>,
    pub statuses: Vec<MemoryStatus>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub visibility: Option<MemoryVisibility>,
    pub belief_branch: Option<String>,
    pub source_agent: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    pub tags: Vec<String>,
    pub stages: Vec<MemoryStage>,
    pub limit: Option<usize>,
    pub max_chars_per_item: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMemoryResponse {
    pub route: RetrievalRoute,
    pub intent: RetrievalIntent,
    pub items: Vec<MemoryItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace: Option<SearchRetrievalTrace>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRetrievalTrace {
    pub query: Option<String>,
    pub lanes: Vec<String>,
    pub items: Vec<SearchItemTrace>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchItemTrace {
    pub id: Uuid,
    pub final_rank: usize,
    pub final_score: f64,
    pub signals: Vec<SearchSignalTrace>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSignalTrace {
    pub lane: String,
    pub score: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextRequest {
    pub project: Option<String>,
    pub agent: Option<String>,
    pub workspace: Option<String>,
    pub visibility: Option<MemoryVisibility>,
    pub route: Option<RetrievalRoute>,
    pub intent: Option<RetrievalIntent>,
    pub limit: Option<usize>,
    pub max_chars_per_item: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextResponse {
    pub route: RetrievalRoute,
    pub intent: RetrievalIntent,
    pub retrieval_order: Vec<MemoryScope>,
    pub items: Vec<MemoryItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactMemoryRecord {
    pub id: Uuid,
    pub record: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactContextResponse {
    pub route: RetrievalRoute,
    pub intent: RetrievalIntent,
    pub retrieval_order: Vec<MemoryScope>,
    pub records: Vec<CompactMemoryRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextPacketRequest {
    pub project: Option<String>,
    pub agent: Option<String>,
    pub workspace: Option<String>,
    pub visibility: Option<MemoryVisibility>,
    pub route: Option<RetrievalRoute>,
    pub intent: Option<RetrievalIntent>,
    pub limit: Option<usize>,
    pub max_chars_per_item: Option<usize>,
    pub model_tier: Option<String>,
    pub safety: Option<String>,
    #[serde(default)]
    pub include_capabilities: bool,
    #[serde(default)]
    pub include_access: bool,
    #[serde(default)]
    pub include_hive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPacketSection {
    pub name: String,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPacketResponse {
    pub route: RetrievalRoute,
    pub intent: RetrievalIntent,
    pub retrieval_order: Vec<MemoryScope>,
    pub model_tier: String,
    pub safety_mode: String,
    pub packet: String,
    pub sections: Vec<ContextPacketSection>,
    pub source_ids: Vec<Uuid>,
    pub compact: CompactContextResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkingMemoryRequest {
    pub project: Option<String>,
    pub agent: Option<String>,
    pub workspace: Option<String>,
    pub visibility: Option<MemoryVisibility>,
    pub route: Option<RetrievalRoute>,
    pub intent: Option<RetrievalIntent>,
    pub limit: Option<usize>,
    pub max_chars_per_item: Option<usize>,
    pub max_total_chars: Option<usize>,
    pub rehydration_limit: Option<usize>,
    pub auto_consolidate: Option<bool>,
    /// Optional query text for lane-aware scoring (G2.2).
    /// When provided, items whose lane matches the query's detected lane
    /// get a higher boost than items with a different lane.
    pub query: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingMemoryResponse {
    pub route: RetrievalRoute,
    pub intent: RetrievalIntent,
    pub retrieval_order: Vec<MemoryScope>,
    pub budget_chars: usize,
    pub used_chars: usize,
    pub remaining_chars: usize,
    pub truncated: bool,
    pub policy: WorkingMemoryPolicyState,
    pub records: Vec<CompactMemoryRecord>,
    pub evicted: Vec<WorkingMemoryEvictionRecord>,
    pub rehydration_queue: Vec<MemoryRehydrationRecord>,
    pub traces: Vec<WorkingMemoryTraceRecord>,
    pub semantic_consolidation: Option<MemoryConsolidationResponse>,
    /// Procedures matched against current working context (Phase G).
    #[serde(default)]
    pub procedures: Vec<Procedure>,
    /// Admission cycle quality metrics for M3/J2 observability.
    #[serde(default)]
    pub compaction_quality: Option<CompactionQualityReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingMemoryPolicyState {
    pub admission_limit: usize,
    pub max_chars_per_item: usize,
    pub budget_chars: usize,
    pub rehydration_limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionQualityReport {
    pub admitted: usize,
    pub evicted: usize,
    pub per_kind_admitted: BTreeMap<String, usize>,
    pub per_kind_evicted: BTreeMap<String, usize>,
    /// Per-kind character counts for admitted items.
    #[serde(default)]
    pub chars_per_kind_admitted: BTreeMap<String, usize>,
    pub budget_chars: usize,
    pub used_chars: usize,
}

/// Per-kind character and item counts for a single operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerKindTokenMetrics {
    /// Map of kind name → character count consumed by that kind.
    pub chars_per_kind: BTreeMap<String, usize>,
    /// Map of kind name → item count for that kind.
    pub items_per_kind: BTreeMap<String, usize>,
    /// Total characters consumed across all kinds.
    pub total_chars: usize,
    /// Total items across all kinds.
    pub total_items: usize,
}

/// Token efficiency report for a single operation (wake, recall, handoff, working memory).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationTokenReport {
    /// Which operation produced this report.
    pub operation: String,
    /// Total character budget available for this operation.
    pub budget_chars: usize,
    /// Characters actually used.
    pub used_chars: usize,
    /// Budget utilization as a percentage (0.0–100.0).
    pub utilization_pct: f64,
    /// Per-kind breakdown.
    pub per_kind: PerKindTokenMetrics,
}

/// Combined token efficiency report across all measured operations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenEfficiencyReport {
    /// One entry per measured operation.
    pub operations: Vec<OperationTokenReport>,
    /// Timestamp of this report (seconds since epoch).
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingMemoryEvictionRecord {
    pub id: Uuid,
    pub record: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRehydrationRecord {
    pub id: Option<Uuid>,
    pub kind: String,
    pub label: String,
    pub summary: String,
    pub reason: Option<String>,
    pub source_agent: Option<String>,
    pub source_system: Option<String>,
    pub source_path: Option<String>,
    pub source_quality: Option<SourceQuality>,
    pub recorded_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentProfileRequest {
    pub agent: String,
    pub project: Option<String>,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAgentProfile {
    pub id: Uuid,
    pub agent: String,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub preferred_route: Option<RetrievalRoute>,
    pub preferred_intent: Option<RetrievalIntent>,
    pub summary_chars: Option<usize>,
    pub max_total_chars: Option<usize>,
    pub recall_depth: Option<usize>,
    pub source_trust_floor: Option<f32>,
    pub style_tags: Vec<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProfileResponse {
    pub profile: Option<MemoryAgentProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProfileUpsertRequest {
    pub agent: String,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub preferred_route: Option<RetrievalRoute>,
    pub preferred_intent: Option<RetrievalIntent>,
    pub summary_chars: Option<usize>,
    pub max_total_chars: Option<usize>,
    pub recall_depth: Option<usize>,
    pub source_trust_floor: Option<f32>,
    pub style_tags: Vec<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMemoryRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub visibility: Option<MemoryVisibility>,
    pub source_agent: Option<String>,
    pub source_system: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMemoryRecord {
    pub source_agent: Option<String>,
    pub source_system: Option<String>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub visibility: MemoryVisibility,
    pub item_count: usize,
    pub active_count: usize,
    pub candidate_count: usize,
    pub derived_count: usize,
    pub synthetic_count: usize,
    pub contested_count: usize,
    pub avg_confidence: f32,
    pub trust_score: f32,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMemoryResponse {
    pub sources: Vec<SourceMemoryRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMemoryRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub visibility: Option<MemoryVisibility>,
    pub source_agent: Option<String>,
    pub source_system: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMemoryRecord {
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub visibility: MemoryVisibility,
    pub item_count: usize,
    pub active_count: usize,
    pub candidate_count: usize,
    pub contested_count: usize,
    pub source_lane_count: usize,
    pub avg_confidence: f32,
    pub trust_score: f32,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMemoryResponse {
    pub workspaces: Vec<WorkspaceMemoryRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveMessageRecord {
    pub id: String,
    pub kind: String,
    pub from_session: String,
    pub from_agent: Option<String>,
    pub to_session: String,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveMessageSendRequest {
    pub kind: String,
    pub from_session: String,
    pub from_agent: Option<String>,
    pub to_session: String,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveMessageAckRequest {
    pub id: String,
    pub session: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HiveMessageInboxRequest {
    pub session: String,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub include_acknowledged: Option<bool>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveMessagesResponse {
    pub messages: Vec<HiveMessageRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveClaimRecord {
    pub scope: String,
    pub session: String,
    pub tab_id: Option<String>,
    pub agent: Option<String>,
    pub effective_agent: Option<String>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub host: Option<String>,
    pub pid: Option<u32>,
    pub acquired_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveClaimAcquireRequest {
    pub scope: String,
    pub session: String,
    pub tab_id: Option<String>,
    pub agent: Option<String>,
    pub effective_agent: Option<String>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub host: Option<String>,
    pub pid: Option<u32>,
    pub ttl_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveClaimReleaseRequest {
    pub scope: String,
    pub session: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveClaimTransferRequest {
    pub scope: String,
    pub from_session: String,
    pub to_session: String,
    pub to_tab_id: Option<String>,
    pub to_agent: Option<String>,
    pub to_effective_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveClaimRecoverRequest {
    pub scope: String,
    pub from_session: String,
    pub to_session: Option<String>,
    pub to_tab_id: Option<String>,
    pub to_agent: Option<String>,
    pub to_effective_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HiveClaimsRequest {
    pub session: Option<String>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub active_only: Option<bool>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveClaimsResponse {
    pub claims: Vec<HiveClaimRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HiveSessionRecord {
    pub session: String,
    pub tab_id: Option<String>,
    pub agent: Option<String>,
    pub effective_agent: Option<String>,
    pub hive_system: Option<String>,
    pub hive_role: Option<String>,
    #[serde(default)]
    pub worker_name: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub hive_groups: Vec<String>,
    #[serde(default)]
    pub lane_id: Option<String>,
    pub hive_group_goal: Option<String>,
    pub authority: Option<String>,
    pub heartbeat_model: Option<String>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub repo_root: Option<String>,
    pub worktree_root: Option<String>,
    pub branch: Option<String>,
    pub base_branch: Option<String>,
    pub visibility: Option<String>,
    pub base_url: Option<String>,
    pub base_url_healthy: Option<bool>,
    pub host: Option<String>,
    pub pid: Option<u32>,
    pub topic_claim: Option<String>,
    #[serde(default)]
    pub scope_claims: Vec<String>,
    pub task_id: Option<String>,
    pub focus: Option<String>,
    pub pressure: Option<String>,
    pub next_recovery: Option<String>,
    #[serde(default)]
    pub next_action: Option<String>,
    #[serde(default)]
    pub working: Option<String>,
    #[serde(default)]
    pub touches: Vec<String>,
    #[serde(default)]
    pub relationship_state: Option<String>,
    #[serde(default)]
    pub relationship_peer: Option<String>,
    #[serde(default)]
    pub relationship_reason: Option<String>,
    #[serde(default)]
    pub suggested_action: Option<String>,
    #[serde(default)]
    pub blocked_by: Vec<String>,
    #[serde(default)]
    pub cowork_with: Vec<String>,
    #[serde(default)]
    pub handoff_target: Option<String>,
    #[serde(default)]
    pub offered_to: Vec<String>,
    #[serde(default)]
    pub needs_help: bool,
    #[serde(default)]
    pub needs_review: bool,
    #[serde(default)]
    pub handoff_state: Option<String>,
    #[serde(default)]
    pub confidence: Option<String>,
    #[serde(default)]
    pub risk: Option<String>,
    pub status: String,
    pub last_seen: DateTime<Utc>,
    #[serde(default)]
    pub last_wake_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HiveSessionUpsertRequest {
    pub session: String,
    pub tab_id: Option<String>,
    pub agent: Option<String>,
    pub effective_agent: Option<String>,
    pub hive_system: Option<String>,
    pub hive_role: Option<String>,
    #[serde(default)]
    pub worker_name: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub hive_groups: Vec<String>,
    #[serde(default)]
    pub lane_id: Option<String>,
    pub hive_group_goal: Option<String>,
    pub authority: Option<String>,
    pub heartbeat_model: Option<String>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub repo_root: Option<String>,
    pub worktree_root: Option<String>,
    pub branch: Option<String>,
    pub base_branch: Option<String>,
    pub visibility: Option<String>,
    pub base_url: Option<String>,
    pub base_url_healthy: Option<bool>,
    pub host: Option<String>,
    pub pid: Option<u32>,
    pub topic_claim: Option<String>,
    #[serde(default)]
    pub scope_claims: Vec<String>,
    pub task_id: Option<String>,
    pub focus: Option<String>,
    pub pressure: Option<String>,
    pub next_recovery: Option<String>,
    pub status: Option<String>,
    #[serde(default)]
    pub next_action: Option<String>,
    #[serde(default)]
    pub working: Option<String>,
    #[serde(default)]
    pub touches: Vec<String>,
    #[serde(default)]
    pub blocked_by: Vec<String>,
    #[serde(default)]
    pub cowork_with: Vec<String>,
    #[serde(default)]
    pub handoff_target: Option<String>,
    #[serde(default)]
    pub offered_to: Vec<String>,
    #[serde(default)]
    pub needs_help: bool,
    #[serde(default)]
    pub needs_review: bool,
    #[serde(default)]
    pub handoff_state: Option<String>,
    #[serde(default)]
    pub confidence: Option<String>,
    #[serde(default)]
    pub risk: Option<String>,
    #[serde(default)]
    pub last_wake_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HiveSessionsRequest {
    pub session: Option<String>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub repo_root: Option<String>,
    pub worktree_root: Option<String>,
    pub branch: Option<String>,
    pub hive_system: Option<String>,
    pub hive_role: Option<String>,
    pub host: Option<String>,
    pub hive_group: Option<String>,
    pub active_only: Option<bool>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveSessionsResponse {
    pub sessions: Vec<HiveSessionRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveRosterResponse {
    pub project: String,
    pub namespace: String,
    pub queen_session: Option<String>,
    pub bees: Vec<HiveSessionRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HiveRosterRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HiveBoardRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveBoardResponse {
    pub queen_session: Option<String>,
    pub active_bees: Vec<HiveSessionRecord>,
    pub blocked_bees: Vec<String>,
    pub stale_bees: Vec<String>,
    pub review_queue: Vec<String>,
    pub overlap_risks: Vec<String>,
    pub lane_faults: Vec<String>,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HiveFollowRequest {
    pub session: String,
    pub current_session: Option<String>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveFollowResponse {
    pub current_session: Option<String>,
    pub target: HiveSessionRecord,
    pub work_summary: String,
    pub touch_points: Vec<String>,
    pub next_action: Option<String>,
    pub messages: Vec<HiveMessageRecord>,
    pub owned_tasks: Vec<HiveTaskRecord>,
    pub help_tasks: Vec<HiveTaskRecord>,
    pub review_tasks: Vec<HiveTaskRecord>,
    pub recent_receipts: Vec<HiveCoordinationReceiptRecord>,
    pub overlap_risk: Option<String>,
    pub recommended_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveSessionRetireRequest {
    pub session: String,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub repo_root: Option<String>,
    pub worktree_root: Option<String>,
    pub branch: Option<String>,
    pub agent: Option<String>,
    pub effective_agent: Option<String>,
    pub hive_system: Option<String>,
    pub hive_role: Option<String>,
    pub host: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveSessionRetireResponse {
    pub retired: usize,
    pub sessions: Vec<HiveSessionRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HiveSessionAutoRetireRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveSessionAutoRetireResponse {
    pub retired: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveQueenActionRequest {
    pub queen_session: String,
    pub target_session: Option<String>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub scope: Option<String>,
    pub note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_lane: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveQueenActionResponse {
    pub action: String,
    pub target_session: Option<String>,
    pub receipt: Option<HiveCoordinationReceiptRecord>,
    pub message_id: Option<String>,
    pub retired: Vec<String>,
    pub summary: String,
    pub follow_session: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveHandoffPacket {
    pub from_session: String,
    pub from_worker: Option<String>,
    pub to_session: String,
    pub to_worker: Option<String>,
    pub task_id: Option<String>,
    pub topic_claim: Option<String>,
    pub scope_claims: Vec<String>,
    pub next_action: Option<String>,
    pub blocker: Option<String>,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    /// L2.4: compact snapshot of outgoing-bee working memory, continuity
    /// fields, and unresolved procedures. Optional so legacy payloads
    /// decode; serialized only when populated so receipts stay compact.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_context: Option<WorkingContextSnapshot>,
}

/// L2.4: carrier for the outgoing bee's in-flight context on handoff.
///
/// Hard-capped at 8 working-memory slots to mirror the wake working-set
/// cap. Procedures are included only when `status != done` in the source
/// session so the receiver sees what is still open.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkingContextSnapshot {
    #[serde(default)]
    pub working_records: Vec<CompactMemoryRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doing: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left_off: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocker: Option<String>,
    #[serde(default)]
    pub unresolved_procedures: Vec<Procedure>,
    /// Lamport stamp of the snapshot — the outgoing session's `version` at
    /// capture time. Lets the receiver detect a stale handoff packet
    /// arriving after a newer one.
    #[serde(default)]
    pub version: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub captured_at: Option<DateTime<Utc>>,
}

/// L2.5: per-branch divergence summary. Kept bounded so the human
/// dashboard can render it without paging — at most 2 branches with 3
/// decisions each. Decisions are normalized (trim + lowercase + collapse
/// whitespace) and deduplicated so the dashboard sees the distinct set.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DivergenceSummary {
    pub branches: Vec<DivergenceBranch>,
    pub truncated_branches: bool,
}

impl DivergenceSummary {
    pub const MAX_BRANCHES: usize = 2;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DivergenceBranch {
    pub branch_name: String,
    pub decisions: Vec<DivergenceDecision>,
    pub truncated_decisions: bool,
}

impl DivergenceBranch {
    pub const MAX_DECISIONS: usize = 3;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DivergenceDecision {
    pub id: Uuid,
    /// Raw decision text (first 280 chars, untrimmed).
    pub text: String,
    /// Normalized form used for dedup + diff at the caller.
    pub normalized: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DivergenceRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
}

impl WorkingContextSnapshot {
    /// Cap on the working-records slice — matches the 8-slot working-set
    /// budget applied at retrieval time. Callers should pass already-trimmed
    /// data, but we enforce it here defensively so packets never balloon.
    pub const MAX_WORKING_RECORDS: usize = 8;

    pub fn truncate_to_cap(mut self) -> Self {
        if self.working_records.len() > Self::MAX_WORKING_RECORDS {
            self.working_records.truncate(Self::MAX_WORKING_RECORDS);
        }
        self
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CoordinationMode {
    #[default]
    ExclusiveWrite,
    SharedReview,
    HelpOnly,
    Solo,
}

impl CoordinationMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            CoordinationMode::ExclusiveWrite => "exclusive_write",
            CoordinationMode::SharedReview => "shared_review",
            CoordinationMode::HelpOnly => "help_only",
            CoordinationMode::Solo => "solo",
        }
    }
}

impl std::fmt::Display for CoordinationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for CoordinationMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "exclusive_write" => Ok(CoordinationMode::ExclusiveWrite),
            "shared_review" => Ok(CoordinationMode::SharedReview),
            "help_only" => Ok(CoordinationMode::HelpOnly),
            "solo" => Ok(CoordinationMode::Solo),
            other => Err(format!("unknown coordination_mode: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveTaskRecord {
    pub task_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    #[serde(default)]
    pub coordination_mode: CoordinationMode,
    pub session: Option<String>,
    pub agent: Option<String>,
    pub effective_agent: Option<String>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub claim_scopes: Vec<String>,
    pub help_requested: bool,
    pub review_requested: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveTaskUpsertRequest {
    pub task_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub coordination_mode: Option<CoordinationMode>,
    pub session: Option<String>,
    pub agent: Option<String>,
    pub effective_agent: Option<String>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub claim_scopes: Vec<String>,
    pub help_requested: Option<bool>,
    pub review_requested: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveTaskAssignRequest {
    pub task_id: String,
    pub from_session: Option<String>,
    pub to_session: String,
    pub to_agent: Option<String>,
    pub to_effective_agent: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HiveTasksRequest {
    pub session: Option<String>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub active_only: Option<bool>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveTasksResponse {
    pub tasks: Vec<HiveTaskRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HiveCoordinationInboxRequest {
    pub session: String,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveCoordinationInboxResponse {
    pub messages: Vec<HiveMessageRecord>,
    pub owned_tasks: Vec<HiveTaskRecord>,
    pub help_tasks: Vec<HiveTaskRecord>,
    pub review_tasks: Vec<HiveTaskRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveCoordinationReceiptRecord {
    pub id: String,
    pub kind: String,
    pub actor_session: String,
    pub actor_agent: Option<String>,
    pub target_session: Option<String>,
    pub task_id: Option<String>,
    pub scope: Option<String>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub summary: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveCoordinationReceiptRequest {
    pub kind: String,
    pub actor_session: String,
    pub actor_agent: Option<String>,
    pub target_session: Option<String>,
    pub task_id: Option<String>,
    pub scope: Option<String>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HiveCoordinationReceiptsRequest {
    pub session: Option<String>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveCoordinationReceiptsResponse {
    pub receipts: Vec<HiveCoordinationReceiptRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevServerLeaseRecord {
    pub scope: String,
    pub host: String,
    pub port: u16,
    pub url: String,
    pub repo_root: String,
    pub repo_hash: String,
    pub command: Vec<String>,
    pub session: String,
    pub tab_id: Option<String>,
    pub agent: Option<String>,
    pub effective_agent: Option<String>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub host_name: Option<String>,
    pub pid: Option<u32>,
    pub acquired_at: DateTime<Utc>,
    pub last_heartbeat_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevServerLeaseAcquireRequest {
    pub scope: String,
    pub host: String,
    pub port: u16,
    pub url: String,
    pub repo_root: String,
    pub repo_hash: String,
    pub command: Vec<String>,
    pub session: String,
    pub tab_id: Option<String>,
    pub agent: Option<String>,
    pub effective_agent: Option<String>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub host_name: Option<String>,
    pub pid: Option<u32>,
    pub ttl_seconds: u64,
    pub recover_stale: bool,
    pub stale_after_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevServerLeaseReleaseRequest {
    pub scope: String,
    pub session: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DevServerLeasesRequest {
    pub session: Option<String>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub repo_hash: Option<String>,
    pub active_only: Option<bool>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevServerLeasesResponse {
    pub leases: Vec<DevServerLeaseRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPolicyActivationRecord {
    pub harness: String,
    pub name: String,
    pub kind: String,
    pub portability_class: String,
    pub proposal: String,
    pub sandbox: String,
    pub sandbox_risk: f32,
    pub sandbox_reason: String,
    pub activation: String,
    pub activation_reason: String,
    pub source_path: String,
    pub target_path: Option<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPolicyApplyRequest {
    pub bundle_root: String,
    pub runtime_defaulted: bool,
    pub source_queue_path: String,
    pub applied_count: usize,
    pub skipped_count: usize,
    pub applied: Vec<SkillPolicyActivationRecord>,
    pub skipped: Vec<SkillPolicyActivationRecord>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPolicyApplyReceipt {
    pub id: String,
    pub bundle_root: String,
    pub runtime_defaulted: bool,
    pub source_queue_path: String,
    pub applied_count: usize,
    pub skipped_count: usize,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPolicyApplyResponse {
    pub receipt: SkillPolicyApplyReceipt,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillPolicyApplyReceiptsRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPolicyApplyReceiptsResponse {
    pub receipts: Vec<SkillPolicyApplyReceipt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPolicyActivationEntry {
    pub receipt_id: String,
    pub bundle_root: String,
    pub runtime_defaulted: bool,
    pub source_queue_path: String,
    pub record: SkillPolicyActivationRecord,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillPolicyActivationEntriesRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPolicyActivationEntriesResponse {
    pub activations: Vec<SkillPolicyActivationEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingMemoryTraceRecord {
    pub item_id: Uuid,
    pub entity_id: Option<Uuid>,
    #[serde(default = "default_working_memory_trace_kind")]
    pub memory_kind: MemoryKind,
    #[serde(default = "default_working_memory_trace_stage")]
    pub memory_stage: MemoryStage,
    #[serde(default = "default_working_memory_trace_typed_memory")]
    pub typed_memory: String,
    pub event_type: String,
    pub summary: String,
    pub occurred_at: DateTime<Utc>,
    pub salience_score: f32,
}

fn default_working_memory_trace_kind() -> MemoryKind {
    MemoryKind::Status
}

fn default_working_memory_trace_stage() -> MemoryStage {
    MemoryStage::Canonical
}

fn default_working_memory_trace_typed_memory() -> String {
    "session_continuity+canonical".to_string()
}

#[cfg(test)]
mod tests;
