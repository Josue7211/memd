use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingMemoryPolicyState {
    pub admission_limit: usize,
    pub max_chars_per_item: usize,
    pub budget_chars: usize,
    pub rehydration_limit: usize,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveTaskRecord {
    pub task_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    #[serde(default = "default_coordination_mode")]
    pub coordination_mode: String,
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
    pub coordination_mode: Option<String>,
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

fn default_coordination_mode() -> String {
    "exclusive_write".to_string()
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryInboxRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub visibility: Option<MemoryVisibility>,
    pub belief_branch: Option<String>,
    pub route: Option<RetrievalRoute>,
    pub intent: Option<RetrievalIntent>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboxMemoryItem {
    pub item: MemoryItem,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryContextFrame {
    pub at: Option<DateTime<Utc>>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub host: Option<String>,
    pub branch: Option<String>,
    pub agent: Option<String>,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntityRecord {
    pub id: Uuid,
    pub entity_type: String,
    pub aliases: Vec<String>,
    pub current_state: Option<String>,
    pub state_version: u64,
    pub confidence: f32,
    pub salience_score: f32,
    pub rehearsal_count: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_accessed_at: Option<DateTime<Utc>>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub context: Option<MemoryContextFrame>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEventRecord {
    pub id: Uuid,
    pub entity_id: Option<Uuid>,
    pub event_type: String,
    pub summary: String,
    pub occurred_at: DateTime<Utc>,
    pub recorded_at: DateTime<Utc>,
    pub confidence: f32,
    pub salience_score: f32,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub source_agent: Option<String>,
    pub source_system: Option<String>,
    pub source_path: Option<String>,
    pub related_entity_ids: Vec<Uuid>,
    pub tags: Vec<String>,
    pub context: Option<MemoryContextFrame>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInboxResponse {
    pub route: RetrievalRoute,
    pub intent: RetrievalIntent,
    pub items: Vec<InboxMemoryItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainMemoryRequest {
    pub id: Uuid,
    pub belief_branch: Option<String>,
    pub route: Option<RetrievalRoute>,
    pub intent: Option<RetrievalIntent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMemoryRequest {
    pub id: Uuid,
    pub route: Option<RetrievalRoute>,
    pub intent: Option<RetrievalIntent>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMemoryResponse {
    pub route: RetrievalRoute,
    pub intent: RetrievalIntent,
    pub entity: Option<MemoryEntityRecord>,
    pub events: Vec<MemoryEventRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntitySearchRequest {
    pub query: String,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub at: Option<DateTime<Utc>>,
    pub host: Option<String>,
    pub branch: Option<String>,
    pub location: Option<String>,
    pub route: Option<RetrievalRoute>,
    pub intent: Option<RetrievalIntent>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySearchHit {
    pub entity: MemoryEntityRecord,
    pub score: f32,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySearchResponse {
    pub route: RetrievalRoute,
    pub intent: RetrievalIntent,
    pub query: String,
    pub best_match: Option<EntitySearchHit>,
    pub candidates: Vec<EntitySearchHit>,
    pub ambiguous: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EntityRelationKind {
    SameAs,
    DerivedFrom,
    Supersedes,
    Contradicts,
    Related,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntityLinkRecord {
    pub id: Uuid,
    pub from_entity_id: Uuid,
    pub to_entity_id: Uuid,
    pub relation_kind: EntityRelationKind,
    pub confidence: f32,
    pub created_at: DateTime<Utc>,
    pub note: Option<String>,
    pub context: Option<MemoryContextFrame>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityLinkRequest {
    pub from_entity_id: Uuid,
    pub to_entity_id: Uuid,
    pub relation_kind: EntityRelationKind,
    pub confidence: Option<f32>,
    pub note: Option<String>,
    pub context: Option<MemoryContextFrame>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityLinkResponse {
    pub link: MemoryEntityLinkRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityLinksRequest {
    pub entity_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityLinksResponse {
    pub entity_id: Uuid,
    pub links: Vec<MemoryEntityLinkRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AssociativeRecallRequest {
    pub entity_id: Uuid,
    pub depth: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssociativeRecallHit {
    pub entity: MemoryEntityRecord,
    pub depth: usize,
    pub via: Option<MemoryEntityLinkRecord>,
    pub score: f32,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssociativeRecallResponse {
    pub root_entity: Option<MemoryEntityRecord>,
    pub hits: Vec<AssociativeRecallHit>,
    pub links: Vec<MemoryEntityLinkRecord>,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineMemoryRequest {
    pub id: Uuid,
    pub route: Option<RetrievalRoute>,
    pub intent: Option<RetrievalIntent>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineMemoryResponse {
    pub route: RetrievalRoute,
    pub intent: RetrievalIntent,
    pub entity: Option<MemoryEntityRecord>,
    pub events: Vec<MemoryEventRecord>,
}

// ---------------------------------------------------------------------------
// Atlas types (Phase F)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AtlasLinkKind {
    Temporal,
    Causal,
    Procedural,
    Semantic,
    Corrective,
    Ownership,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasRegion {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub lane: Option<String>,
    pub auto_generated: bool,
    pub node_count: usize,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasNode {
    pub id: Uuid,
    pub region_id: Option<Uuid>,
    pub memory_id: Uuid,
    pub entity_id: Option<Uuid>,
    pub label: String,
    pub kind: MemoryKind,
    pub stage: MemoryStage,
    pub lane: Option<String>,
    pub confidence: f32,
    pub salience: f32,
    pub depth: usize,
    #[serde(default)]
    pub evidence_count: usize,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasLink {
    pub from_node_id: Uuid,
    pub to_node_id: Uuid,
    pub link_kind: AtlasLinkKind,
    pub weight: f32,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasTrail {
    pub name: String,
    pub nodes: Vec<Uuid>,
    pub links: Vec<AtlasLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AtlasRegionsRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub lane: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasRegionsResponse {
    pub regions: Vec<AtlasRegion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasExploreRequest {
    pub region_id: Option<Uuid>,
    pub node_id: Option<Uuid>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub lane: Option<String>,
    pub depth: Option<usize>,
    pub limit: Option<usize>,
    pub pivot_time: Option<DateTime<Utc>>,
    pub pivot_kind: Option<MemoryKind>,
    pub pivot_scope: Option<MemoryScope>,
    pub pivot_source_agent: Option<String>,
    pub pivot_source_system: Option<String>,
    pub min_trust: Option<f32>,
    pub min_salience: Option<f32>,
    #[serde(default)]
    pub include_evidence: bool,
    #[serde(default)]
    pub from_working: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasExploreResponse {
    pub region: Option<AtlasRegion>,
    pub nodes: Vec<AtlasNode>,
    pub links: Vec<AtlasLink>,
    pub trails: Vec<AtlasTrail>,
    #[serde(default)]
    pub evidence: Vec<MemoryEventRecord>,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasSaveTrailRequest {
    pub name: String,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub region_id: Option<Uuid>,
    pub node_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasSavedTrail {
    pub id: Uuid,
    pub name: String,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub region_id: Option<Uuid>,
    pub node_ids: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasSaveTrailResponse {
    pub trail: AtlasSavedTrail,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AtlasListTrailsRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasListTrailsResponse {
    pub trails: Vec<AtlasSavedTrail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasRenameRegionRequest {
    pub region_id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasRenameRegionResponse {
    pub region: AtlasRegion,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AtlasExpandRequest {
    pub memory_ids: Vec<Uuid>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub depth: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasExpandResponse {
    pub seed_count: usize,
    pub expanded_nodes: Vec<AtlasNode>,
    pub links: Vec<AtlasLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryDecayRequest {
    pub max_items: Option<usize>,
    pub inactive_days: Option<i64>,
    pub max_decay: Option<f32>,
    pub record_events: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDecayResponse {
    pub scanned: usize,
    pub updated: usize,
    pub events: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryConsolidationRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub max_groups: Option<usize>,
    pub min_events: Option<usize>,
    pub lookback_days: Option<i64>,
    pub min_salience: Option<f32>,
    pub record_events: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConsolidationResponse {
    pub scanned: usize,
    pub groups: usize,
    pub consolidated: usize,
    pub duplicates: usize,
    pub events: usize,
    pub highlights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryMaintenanceReportRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub inactive_days: Option<i64>,
    pub lookback_days: Option<i64>,
    pub min_events: Option<usize>,
    pub max_decay: Option<f32>,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub apply: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMaintenanceReportResponse {
    pub reinforced_candidates: usize,
    pub cooled_candidates: usize,
    pub consolidated_candidates: usize,
    pub stale_items: usize,
    pub skipped: usize,
    pub highlights: Vec<String>,
    #[serde(default)]
    pub receipt_id: Option<String>,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub compacted_items: usize,
    #[serde(default)]
    pub refreshed_items: usize,
    #[serde(default)]
    pub repaired_items: usize,
    #[serde(default = "Utc::now")]
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MaintainReportRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub session: Option<String>,
    pub mode: String,
    pub apply: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintainReport {
    pub mode: String,
    pub receipt_id: Option<String>,
    pub compacted_items: usize,
    pub refreshed_items: usize,
    pub repaired_items: usize,
    pub findings: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPolicyResponse {
    pub retrieval_order: Vec<MemoryScope>,
    pub route_defaults: Vec<MemoryPolicyRouteDefault>,
    pub working_memory: MemoryPolicyWorkingMemory,
    pub retrieval_feedback: MemoryPolicyFeedback,
    pub source_trust_floor: f32,
    #[serde(default)]
    pub runtime: MemoryPolicyRuntime,
    pub promotion: MemoryPolicyPromotion,
    pub decay: MemoryPolicyDecay,
    pub consolidation: MemoryPolicyConsolidation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPolicyRouteDefault {
    pub intent: RetrievalIntent,
    pub route: RetrievalRoute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPolicyWorkingMemory {
    pub budget_chars: usize,
    pub max_chars_per_item: usize,
    pub default_limit: usize,
    #[serde(default = "default_rehydration_limit")]
    pub rehydration_limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPolicyFeedback {
    pub enabled: bool,
    pub tracked_surfaces: Vec<String>,
    pub max_items_per_request: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPolicyPromotion {
    pub min_salience: f32,
    pub min_events: usize,
    pub lookback_days: i64,
    pub default_ttl_days: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPolicyDecay {
    pub max_items: usize,
    pub inactive_days: i64,
    pub max_decay: f32,
    pub record_events: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPolicyConsolidation {
    pub max_groups: usize,
    pub min_events: usize,
    pub lookback_days: i64,
    pub min_salience: f32,
    pub record_events: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPolicyRuntime {
    pub live_truth: MemoryPolicyLiveTruth,
    pub memory_compilation: MemoryPolicyMemoryCompilation,
    pub semantic_fallback: MemoryPolicySemanticFallback,
    pub skill_gating: MemoryPolicySkillGating,
}

impl Default for MemoryPolicyRuntime {
    fn default() -> Self {
        Self {
            live_truth: MemoryPolicyLiveTruth::default(),
            memory_compilation: MemoryPolicyMemoryCompilation::default(),
            semantic_fallback: MemoryPolicySemanticFallback::default(),
            skill_gating: MemoryPolicySkillGating::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPolicyLiveTruth {
    pub read_once_sources: bool,
    pub raw_reopen_requires_change_or_doubt: bool,
    pub visible_memory_objects: bool,
    pub compile_from_events: bool,
}

impl Default for MemoryPolicyLiveTruth {
    fn default() -> Self {
        Self {
            read_once_sources: false,
            raw_reopen_requires_change_or_doubt: false,
            visible_memory_objects: false,
            compile_from_events: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPolicyMemoryCompilation {
    pub event_driven_updates: bool,
    pub patch_not_rewrite: bool,
    pub preserve_provenance: bool,
    pub source_on_demand: bool,
}

impl Default for MemoryPolicyMemoryCompilation {
    fn default() -> Self {
        Self {
            event_driven_updates: false,
            patch_not_rewrite: false,
            preserve_provenance: false,
            source_on_demand: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPolicySemanticFallback {
    pub enabled: bool,
    pub source_of_truth: bool,
    pub max_items_per_query: usize,
    pub rerank_with_visible_memory: bool,
}

impl Default for MemoryPolicySemanticFallback {
    fn default() -> Self {
        Self {
            enabled: false,
            source_of_truth: false,
            max_items_per_query: 0,
            rerank_with_visible_memory: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPolicySkillGating {
    pub propose_from_repeated_patterns: bool,
    pub sandboxed_evaluation: bool,
    pub auto_activate_low_risk_only: bool,
    pub gated_activation: bool,
    pub require_evaluation: bool,
    pub require_policy_approval: bool,
}

impl Default for MemoryPolicySkillGating {
    fn default() -> Self {
        Self {
            propose_from_repeated_patterns: false,
            sandboxed_evaluation: false,
            auto_activate_low_risk_only: false,
            gated_activation: false,
            require_evaluation: false,
            require_policy_approval: false,
        }
    }
}

fn default_rehydration_limit() -> usize {
    3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainMemoryResponse {
    pub route: RetrievalRoute,
    pub intent: RetrievalIntent,
    pub item: MemoryItem,
    pub canonical_key: String,
    pub redundancy_key: String,
    pub reasons: Vec<String>,
    pub entity: Option<MemoryEntityRecord>,
    pub events: Vec<MemoryEventRecord>,
    pub sources: Vec<SourceMemoryRecord>,
    pub retrieval_feedback: RetrievalFeedbackSummary,
    pub branch_siblings: Vec<ExplainBranchSiblingRecord>,
    pub rehydration: Vec<MemoryRehydrationRecord>,
    pub policy_hooks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainBranchSiblingRecord {
    pub id: Uuid,
    pub belief_branch: Option<String>,
    pub preferred: bool,
    pub status: MemoryStatus,
    pub stage: MemoryStage,
    pub confidence: f32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalFeedbackSummary {
    pub total_retrievals: usize,
    pub last_retrieved_at: Option<DateTime<Utc>>,
    pub by_surface: Vec<RetrievalFeedbackSurfaceCount>,
    pub recent_policy_hooks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalFeedbackSurfaceCount {
    pub surface: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionSession {
    pub project: Option<String>,
    pub agent: Option<String>,
    pub task: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionDecision {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionOpenLoop {
    pub id: String,
    pub text: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionReference {
    pub kind: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionPacket {
    pub session: CompactionSession,
    pub goal: String,
    pub hard_constraints: Vec<String>,
    pub active_work: Vec<String>,
    pub decisions: Vec<CompactionDecision>,
    pub open_loops: Vec<CompactionOpenLoop>,
    pub exact_refs: Vec<CompactionReference>,
    pub next_actions: Vec<String>,
    pub do_not_drop: Vec<String>,
    pub memory: CompactContextResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionSpillBatch {
    pub items: Vec<CandidateMemoryRequest>,
    pub dropped: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionSpillOptions {
    pub include_transient_state: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionSpillResult {
    pub batch: CompactionSpillBatch,
    pub submitted: usize,
    pub duplicates: usize,
    pub responses: Vec<CandidateMemoryResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub items: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkRegistry {
    pub version: String,
    pub app_goal: String,
    pub quality_dimensions: Vec<QualityDimensionRecord>,
    pub tiers: Vec<TierRecord>,
    pub pillars: Vec<PillarRecord>,
    pub families: Vec<FamilyRecord>,
    pub features: Vec<BenchmarkFeatureRecord>,
    pub journeys: Vec<BenchmarkJourneyRecord>,
    pub loops: Vec<BenchmarkLoopRecord>,
    pub verifiers: Vec<VerifierRecord>,
    pub fixtures: Vec<FixtureRecord>,
    pub evidence_policies: Vec<EvidencePolicyRecord>,
    pub schedules: Vec<ScheduleRecord>,
    pub scorecards: Vec<BenchmarkScorecardRecord>,
    pub evidence: Vec<BenchmarkEvidenceRecord>,
    pub gates: Vec<BenchmarkGateRecord>,
    pub baseline_modes: Vec<BaselineModeRecord>,
    pub runtime_policies: Vec<RuntimePolicyRecord>,
    pub generated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QualityDimensionRecord {
    pub id: String,
    pub weight: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TierRecord {
    pub id: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PillarRecord {
    pub id: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FamilyRecord {
    pub id: String,
    pub pillar: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkFeatureRecord {
    pub id: String,
    pub name: String,
    pub pillar: String,
    pub family: String,
    pub tier: String,
    pub continuity_critical: bool,
    pub user_contract: String,
    pub source_contract_refs: Vec<String>,
    pub commands: Vec<String>,
    pub routes: Vec<String>,
    pub files: Vec<String>,
    pub journey_ids: Vec<String>,
    pub loop_ids: Vec<String>,
    pub quality_dimensions: Vec<String>,
    pub drift_risks: Vec<String>,
    pub failure_modes: Vec<String>,
    pub coverage_status: String,
    pub last_verified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkJourneyRecord {
    pub id: String,
    pub name: String,
    pub goal: String,
    pub feature_ids: Vec<String>,
    pub loop_ids: Vec<String>,
    pub quality_dimensions: Vec<String>,
    pub baseline_mode_ids: Vec<String>,
    pub drift_risks: Vec<String>,
    pub gate_target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkLoopRecord {
    pub id: String,
    pub name: String,
    pub pillar: String,
    pub family: String,
    pub loop_type: String,
    pub covers_features: Vec<String>,
    pub journey_ids: Vec<String>,
    pub quality_dimensions: Vec<String>,
    pub baseline_mode: String,
    pub workflow_probe: String,
    pub adversarial_probe: String,
    pub cross_harness_probe: Option<String>,
    pub metrics: Vec<String>,
    pub guardrails: Vec<String>,
    pub stop_condition: String,
    pub artifacts_written: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerifierRecord {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub verifier_type: String,
    pub pillar: String,
    pub family: String,
    pub subject_ids: Vec<String>,
    pub fixture_id: String,
    pub baseline_modes: Vec<String>,
    pub steps: Vec<VerifierStepRecord>,
    pub assertions: Vec<VerifierAssertionRecord>,
    pub metrics: Vec<String>,
    pub evidence_requirements: Vec<String>,
    pub gate_target: String,
    pub status: String,
    pub lanes: Vec<String>,
    pub helper_hooks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerifierStepRecord {
    pub kind: String,
    #[serde(default)]
    pub run: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub left: Option<String>,
    #[serde(default)]
    pub right: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerifierAssertionRecord {
    pub kind: String,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub equals_fixture: Option<String>,
    #[serde(default)]
    pub contains_fixture: Option<String>,
    #[serde(default)]
    pub exists: Option<bool>,
    #[serde(default)]
    pub metric: Option<String>,
    #[serde(default)]
    pub op: Option<String>,
    #[serde(default)]
    pub left: Option<String>,
    #[serde(default)]
    pub right: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FixtureRecord {
    pub id: String,
    pub kind: String,
    pub description: String,
    pub seed_files: Vec<String>,
    pub seed_config: serde_json::Value,
    pub seed_memories: Vec<String>,
    pub seed_events: Vec<String>,
    pub seed_sessions: Vec<String>,
    pub seed_claims: Vec<String>,
    pub seed_vault: Option<String>,
    pub backend_mode: String,
    pub isolation: String,
    pub cleanup_policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvidencePolicyRecord {
    pub id: String,
    pub applies_to: Vec<String>,
    pub required_tiers: Vec<String>,
    pub max_gate_without_live_primary: String,
    pub comparative_required: bool,
    pub freshness_window: String,
    pub contradiction_rule: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScheduleRecord {
    pub id: String,
    pub lane: String,
    pub max_tokens: usize,
    pub max_duration_ms: u64,
    pub tiers: Vec<String>,
    pub default_types: Vec<String>,
    pub retry_policy: String,
    pub quarantine_policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkScorecardRecord {
    pub id: String,
    pub subject_type: String,
    pub subject_id: String,
    pub scores: Vec<ScoreDimensionRecord>,
    pub overall: u8,
    pub coverage: BenchmarkCoverageRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScoreDimensionRecord {
    pub id: String,
    pub score: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkCoverageRecord {
    pub required_loops: usize,
    pub passing_loops: usize,
    pub missing_loops: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkEvidenceRecord {
    pub id: String,
    pub subject_type: String,
    pub subject_id: String,
    pub kind: String,
    pub path_or_ref: String,
    pub captured_at: DateTime<Utc>,
    pub baseline_mode: String,
    pub supports_dimensions: Vec<String>,
    pub supports_loops: Vec<String>,
    pub summary: String,
    pub verdict: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkGateRecord {
    pub id: String,
    pub subject_type: String,
    pub subject_id: String,
    pub gate: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BaselineModeRecord {
    pub id: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuntimePolicyRecord {
    pub id: String,
    pub name: String,
    pub cli_surface: String,
    pub default_value: String,
    pub allowed_range: String,
    pub quality_dimensions_affected: Vec<String>,
    pub risk_level: String,
    pub loop_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScoreResolutionRules {
    pub cap_on_continuity_failure: String,
    pub cap_on_missing_required_evidence: String,
    pub cap_on_no_memd_loss: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkSubjectMetrics {
    pub correctness: u8,
    pub continuity: u8,
    pub reliability: u8,
    pub token_efficiency: u8,
    pub no_memd_delta: Option<i16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkEvidenceSummary {
    pub has_contract_evidence: bool,
    pub has_workflow_evidence: bool,
    pub has_continuity_evidence: bool,
    pub has_comparative_evidence: bool,
    pub has_drift_failure: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkGateDecision {
    pub gate: String,
    pub resolved_score: u8,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContinuityJourneyReport {
    pub journey_id: String,
    pub journey_name: String,
    pub gate_decision: BenchmarkGateDecision,
    pub metrics: BenchmarkSubjectMetrics,
    pub evidence: BenchmarkEvidenceSummary,
    pub baseline_modes: Vec<String>,
    pub feature_ids: Vec<String>,
    pub artifact_paths: Vec<String>,
    pub summary: String,
    pub generated_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_record_roundtrips() {
        let record = MemoryEntityRecord {
            id: Uuid::new_v4(),
            entity_type: "repo".to_string(),
            aliases: vec!["memd".to_string(), "memd-core".to_string()],
            current_state: Some("main branch with multimodal stack".to_string()),
            state_version: 3,
            confidence: 0.93,
            salience_score: 0.82,
            rehearsal_count: 4,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_accessed_at: Some(Utc::now()),
            last_seen_at: Some(Utc::now()),
            valid_from: Some(Utc::now()),
            valid_to: None,
            tags: vec!["project".to_string(), "permanent".to_string()],
            context: Some(MemoryContextFrame {
                at: Some(Utc::now()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("core".to_string()),
                repo: Some("memd".to_string()),
                host: Some("laptop".to_string()),
                branch: Some("main".to_string()),
                agent: Some("codex".to_string()),
                location: Some("home".to_string()),
            }),
        };

        let json = serde_json::to_string(&record).unwrap();
        let decoded: MemoryEntityRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.entity_type, "repo");
        assert_eq!(decoded.aliases.len(), 2);
    }

    #[test]
    fn hive_session_roundtrips_worker_identity_fields() {
        let session = HiveSessionRecord {
            session: "session-lorentz".to_string(),
            tab_id: Some("tab-lorentz".to_string()),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            agent: Some("codex".to_string()),
            effective_agent: Some("codex@session-lorentz".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("reviewer".to_string()),
            hive_groups: vec!["project:memd".to_string()],
            hive_group_goal: Some("review parser handoff".to_string()),
            authority: Some("participant".to_string()),
            heartbeat_model: Some("codex".to_string()),
            worker_name: Some("Lorentz".to_string()),
            display_name: Some("Parser Reviewer".to_string()),
            role: Some("reviewer".to_string()),
            capabilities: vec!["review".to_string(), "coordination".to_string()],
            lane_id: Some("lane-render-review".to_string()),
            repo_root: Some("/repo".to_string()),
            worktree_root: Some("/repo-review".to_string()),
            branch: Some("review/render".to_string()),
            base_branch: Some("main".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            base_url_healthy: Some(true),
            host: Some("workstation".to_string()),
            pid: Some(4242),
            topic_claim: Some("Review parser handoff".to_string()),
            scope_claims: vec!["crates/memd-client/src/main.rs".to_string()],
            task_id: Some("review-parser-handoff".to_string()),
            focus: Some("Review parser handoff".to_string()),
            pressure: Some("file_edited: crates/memd-client/src/main.rs".to_string()),
            next_recovery: Some("publish overlap-safe hive quickview".to_string()),
            next_action: Some("Review overlap guard output".to_string()),
            working: Some("yes".to_string()),
            touches: vec!["parser".to_string()],
            relationship_state: Some("coordinating".to_string()),
            relationship_peer: Some("hive".to_string()),
            relationship_reason: Some("parser handoff".to_string()),
            suggested_action: Some("review".to_string()),
            blocked_by: vec!["tests".to_string()],
            cowork_with: vec!["hive".to_string()],
            handoff_target: Some("codex".to_string()),
            offered_to: vec!["review".to_string()],
            status: "active".to_string(),
            needs_help: false,
            needs_review: false,
            handoff_state: Some("none".to_string()),
            confidence: Some("high".to_string()),
            risk: Some("low".to_string()),
            last_seen: Utc::now(),
        };

        let json = serde_json::to_string(&session).expect("serialize session");
        let decoded: HiveSessionRecord = serde_json::from_str(&json).expect("deserialize session");
        assert_eq!(decoded.worker_name.as_deref(), Some("Lorentz"));
        assert_eq!(decoded.display_name.as_deref(), Some("Parser Reviewer"));
        assert_eq!(decoded.role.as_deref(), Some("reviewer"));
        assert_eq!(decoded.lane_id.as_deref(), Some("lane-render-review"));
        assert_eq!(
            decoded.next_action.as_deref(),
            Some("Review overlap guard output")
        );
        assert_eq!(decoded.risk.as_deref(), Some("low"));
    }

    #[test]
    fn event_record_roundtrips() {
        let record = MemoryEventRecord {
            id: Uuid::new_v4(),
            entity_id: Some(Uuid::new_v4()),
            event_type: "rename".to_string(),
            summary: "repo renamed but entity stayed the same".to_string(),
            occurred_at: Utc::now(),
            recorded_at: Utc::now(),
            confidence: 0.88,
            salience_score: 0.74,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("core".to_string()),
            source_agent: Some("codex".to_string()),
            source_system: Some("cli".to_string()),
            source_path: Some("/tmp/memd".to_string()),
            related_entity_ids: vec![Uuid::new_v4()],
            tags: vec!["identity".to_string(), "timeline".to_string()],
            context: Some(MemoryContextFrame {
                at: Some(Utc::now()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("core".to_string()),
                repo: Some("memd".to_string()),
                host: Some("laptop".to_string()),
                branch: Some("main".to_string()),
                agent: Some("codex".to_string()),
                location: Some("office".to_string()),
            }),
        };

        let json = serde_json::to_string(&record).unwrap();
        let decoded: MemoryEventRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.event_type, "rename");
        assert_eq!(decoded.tags.len(), 2);
    }

    #[test]
    fn visible_memory_artifact_snapshot_round_trips() {
        let snapshot = VisibleMemorySnapshotResponse {
            generated_at: Utc::now(),
            home: VisibleMemoryHome {
                focus_artifact: VisibleMemoryArtifact {
                    id: Uuid::new_v4(),
                    title: "runtime spine".to_string(),
                    body: "runtime spine is the canonical memory contract".to_string(),
                    artifact_kind: "compiled_page".to_string(),
                    memory_kind: Some(MemoryKind::Decision),
                    scope: Some(MemoryScope::Project),
                    visibility: Some(MemoryVisibility::Workspace),
                    workspace: Some("team-alpha".to_string()),
                    status: VisibleMemoryStatus::Current,
                    freshness: "fresh".to_string(),
                    confidence: 0.93,
                    provenance: VisibleMemoryProvenance {
                        source_system: Some("obsidian".to_string()),
                        source_path: Some("wiki/runtime-spine.md".to_string()),
                        producer: Some("obsidian compile".to_string()),
                        trust_reason: "verified from compiled workspace page".to_string(),
                        last_verified_at: None,
                    },
                    sources: vec!["wiki/runtime-spine.md".to_string()],
                    linked_artifact_ids: vec![],
                    linked_sessions: vec!["codex-01".to_string()],
                    linked_agents: vec!["codex".to_string()],
                    repair_state: "healthy".to_string(),
                    actions: vec!["inspect".to_string(), "verify_current".to_string()],
                },
                inbox_count: 3,
                repair_count: 1,
                awareness_count: 2,
            },
            knowledge_map: VisibleMemoryKnowledgeMap {
                nodes: vec![VisibleMemoryGraphNode {
                    artifact_id: Uuid::new_v4(),
                    title: "runtime spine".to_string(),
                    artifact_kind: "compiled_page".to_string(),
                    status: VisibleMemoryStatus::Current,
                }],
                edges: vec![VisibleMemoryGraphEdge {
                    from: Uuid::new_v4(),
                    to: Uuid::new_v4(),
                    relation: "focus".to_string(),
                }],
            },
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        let decoded: VisibleMemorySnapshotResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.home.focus_artifact.title, "runtime spine");
        assert_eq!(
            decoded.home.focus_artifact.status,
            VisibleMemoryStatus::Current
        );
        assert_eq!(decoded.home.inbox_count, 3);
        assert_eq!(decoded.home.repair_count, 1);
        assert_eq!(decoded.knowledge_map.nodes.len(), 1);
        assert_eq!(decoded.knowledge_map.edges.len(), 1);
    }

    #[test]
    fn visible_memory_artifact_detail_round_trips() {
        let detail = VisibleMemoryArtifactDetailResponse {
            generated_at: Utc::now(),
            artifact: VisibleMemoryArtifact {
                id: Uuid::new_v4(),
                title: "runtime spine".to_string(),
                body: "runtime spine is the canonical memory contract".to_string(),
                artifact_kind: "compiled_page".to_string(),
                memory_kind: Some(MemoryKind::Decision),
                scope: Some(MemoryScope::Project),
                visibility: Some(MemoryVisibility::Workspace),
                workspace: Some("team-alpha".to_string()),
                status: VisibleMemoryStatus::Current,
                freshness: "fresh".to_string(),
                confidence: 0.93,
                provenance: VisibleMemoryProvenance {
                    source_system: Some("obsidian".to_string()),
                    source_path: Some("wiki/runtime-spine.md".to_string()),
                    producer: Some("obsidian compile".to_string()),
                    trust_reason: "verified from compiled workspace page".to_string(),
                    last_verified_at: None,
                },
                sources: vec!["wiki/runtime-spine.md".to_string()],
                linked_artifact_ids: vec![],
                linked_sessions: vec!["codex-01".to_string()],
                linked_agents: vec!["codex".to_string()],
                repair_state: "healthy".to_string(),
                actions: vec!["inspect".to_string(), "verify_current".to_string()],
            },
            explain: None,
            timeline: None,
            sources: SourceMemoryResponse { sources: vec![] },
            workspaces: WorkspaceMemoryResponse { workspaces: vec![] },
            sessions: HiveSessionsResponse { sessions: vec![] },
            tasks: HiveTasksResponse { tasks: vec![] },
            claims: HiveClaimsResponse { claims: vec![] },
            related_artifacts: vec![],
            related_map: VisibleMemoryKnowledgeMap {
                nodes: vec![],
                edges: vec![],
            },
            actions: vec![
                VisibleMemoryUiActionKind::Inspect,
                VisibleMemoryUiActionKind::Explain,
                VisibleMemoryUiActionKind::VerifyCurrent,
            ],
        };

        let json = serde_json::to_string(&detail).unwrap();
        let decoded: VisibleMemoryArtifactDetailResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.artifact.title, "runtime spine");
        assert_eq!(decoded.actions.len(), 3);
    }

    #[test]
    fn visible_memory_ui_action_round_trips() {
        let request = VisibleMemoryUiActionRequest {
            id: Uuid::new_v4(),
            action: VisibleMemoryUiActionKind::OpenInObsidian,
        };
        let response = VisibleMemoryUiActionResponse {
            action: VisibleMemoryUiActionKind::OpenInObsidian,
            artifact_id: request.id,
            outcome: "opened".to_string(),
            message: "generated obsidian uri".to_string(),
            detail: None,
            open_uri: Some("obsidian://open?path=wiki/runtime-spine.md".to_string()),
            source_path: Some("wiki/runtime-spine.md".to_string()),
        };

        let request_json = serde_json::to_string(&request).unwrap();
        let response_json = serde_json::to_string(&response).unwrap();
        let decoded_request: VisibleMemoryUiActionRequest =
            serde_json::from_str(&request_json).unwrap();
        let decoded_response: VisibleMemoryUiActionResponse =
            serde_json::from_str(&response_json).unwrap();

        assert_eq!(
            decoded_request.action,
            VisibleMemoryUiActionKind::OpenInObsidian
        );
        assert_eq!(decoded_response.artifact_id, request.id);
        assert_eq!(
            decoded_response.open_uri.as_deref(),
            response.open_uri.as_deref()
        );
    }

    #[test]
    fn entity_search_roundtrips() {
        let entity = MemoryEntityRecord {
            id: Uuid::new_v4(),
            entity_type: "repo".to_string(),
            aliases: vec!["memd".to_string()],
            current_state: Some("main".to_string()),
            state_version: 1,
            confidence: 0.8,
            salience_score: 0.7,
            rehearsal_count: 2,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_accessed_at: Some(Utc::now()),
            last_seen_at: Some(Utc::now()),
            valid_from: Some(Utc::now()),
            valid_to: None,
            tags: vec!["project".to_string()],
            context: Some(MemoryContextFrame {
                at: Some(Utc::now()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("core".to_string()),
                repo: Some("memd".to_string()),
                host: Some("laptop".to_string()),
                branch: Some("main".to_string()),
                agent: Some("codex".to_string()),
                location: Some("/tmp/memd".to_string()),
            }),
        };
        let request = EntitySearchRequest {
            query: "memd repo".to_string(),
            project: Some("memd".to_string()),
            namespace: None,
            at: Some(Utc::now()),
            host: Some("laptop".to_string()),
            branch: Some("main".to_string()),
            location: Some("/tmp/memd".to_string()),
            route: Some(RetrievalRoute::ProjectFirst),
            intent: Some(RetrievalIntent::Fact),
            limit: Some(5),
        };
        let response = EntitySearchResponse {
            route: RetrievalRoute::ProjectFirst,
            intent: RetrievalIntent::Fact,
            query: request.query.clone(),
            best_match: Some(EntitySearchHit {
                entity,
                score: 0.93,
                reasons: vec!["alias match".to_string()],
            }),
            candidates: Vec::new(),
            ambiguous: false,
        };

        let request_json = serde_json::to_string(&request).unwrap();
        let response_json = serde_json::to_string(&response).unwrap();
        let decoded_request: EntitySearchRequest = serde_json::from_str(&request_json).unwrap();
        let decoded_response: EntitySearchResponse = serde_json::from_str(&response_json).unwrap();
        assert_eq!(decoded_request.query, request.query);
        assert_eq!(decoded_response.best_match.unwrap().score, 0.93);
    }

    #[test]
    fn entity_link_roundtrips() {
        let link = MemoryEntityLinkRecord {
            id: Uuid::new_v4(),
            from_entity_id: Uuid::new_v4(),
            to_entity_id: Uuid::new_v4(),
            relation_kind: EntityRelationKind::DerivedFrom,
            confidence: 0.84,
            created_at: Utc::now(),
            note: Some("rolled up from repeated traces".to_string()),
            context: Some(MemoryContextFrame {
                at: Some(Utc::now()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("core".to_string()),
                repo: Some("memd".to_string()),
                host: Some("laptop".to_string()),
                branch: Some("main".to_string()),
                agent: Some("codex".to_string()),
                location: Some("/tmp/memd".to_string()),
            }),
            tags: vec!["semantic".to_string()],
        };
        let request = EntityLinkRequest {
            from_entity_id: link.from_entity_id,
            to_entity_id: link.to_entity_id,
            relation_kind: link.relation_kind,
            confidence: Some(link.confidence),
            note: link.note.clone(),
            context: link.context.clone(),
            tags: link.tags.clone(),
        };
        let response = EntityLinkResponse { link: link.clone() };
        let links = EntityLinksResponse {
            entity_id: link.from_entity_id,
            links: vec![link],
        };

        let request_json = serde_json::to_string(&request).unwrap();
        let response_json = serde_json::to_string(&response).unwrap();
        let links_json = serde_json::to_string(&links).unwrap();
        let decoded_request: EntityLinkRequest = serde_json::from_str(&request_json).unwrap();
        let decoded_response: EntityLinkResponse = serde_json::from_str(&response_json).unwrap();
        let decoded_links: EntityLinksResponse = serde_json::from_str(&links_json).unwrap();
        assert_eq!(decoded_request.relation_kind, request.relation_kind);
        assert_eq!(decoded_response.link.confidence, response.link.confidence);
        assert_eq!(decoded_links.links.len(), 1);
    }

    #[test]
    fn associative_recall_roundtrips() {
        let root = MemoryEntityRecord {
            id: Uuid::new_v4(),
            entity_type: "repo".to_string(),
            aliases: vec!["memd".to_string()],
            current_state: Some("working memory".to_string()),
            state_version: 2,
            confidence: 0.89,
            salience_score: 0.77,
            rehearsal_count: 5,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_accessed_at: Some(Utc::now()),
            last_seen_at: Some(Utc::now()),
            valid_from: Some(Utc::now()),
            valid_to: None,
            tags: vec!["project".to_string()],
            context: Some(MemoryContextFrame {
                at: Some(Utc::now()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("core".to_string()),
                repo: Some("memd".to_string()),
                host: Some("laptop".to_string()),
                branch: Some("main".to_string()),
                agent: Some("codex".to_string()),
                location: Some("/tmp/memd".to_string()),
            }),
        };
        let link = MemoryEntityLinkRecord {
            id: Uuid::new_v4(),
            from_entity_id: root.id,
            to_entity_id: Uuid::new_v4(),
            relation_kind: EntityRelationKind::Related,
            confidence: 0.7,
            created_at: Utc::now(),
            note: Some("adjacent memory".to_string()),
            context: root.context.clone(),
            tags: vec!["graph".to_string()],
        };
        let request = AssociativeRecallRequest {
            entity_id: root.id,
            depth: Some(2),
            limit: Some(6),
        };
        let response = AssociativeRecallResponse {
            root_entity: Some(root.clone()),
            hits: vec![AssociativeRecallHit {
                entity: root,
                depth: 0,
                via: None,
                score: 1.0,
                reasons: vec!["root".to_string()],
            }],
            links: vec![link],
            truncated: false,
        };

        let request_json = serde_json::to_string(&request).unwrap();
        let response_json = serde_json::to_string(&response).unwrap();
        let decoded_request: AssociativeRecallRequest =
            serde_json::from_str(&request_json).unwrap();
        let decoded_response: AssociativeRecallResponse =
            serde_json::from_str(&response_json).unwrap();
        assert_eq!(decoded_request.depth, request.depth);
        assert_eq!(decoded_response.links.len(), 1);
        assert_eq!(decoded_response.hits.len(), 1);
        assert_eq!(decoded_response.hits[0].score, 1.0);
    }

    #[test]
    fn consolidation_request_roundtrips() {
        let request = MemoryConsolidationRequest {
            project: Some("memd".to_string()),
            namespace: Some("agent".to_string()),
            max_groups: Some(12),
            min_events: Some(3),
            lookback_days: Some(14),
            min_salience: Some(0.25),
            record_events: Some(true),
        };

        let json = serde_json::to_string(&request).unwrap();
        let decoded: MemoryConsolidationRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.project, request.project);
        assert_eq!(decoded.min_events, request.min_events);
        assert_eq!(decoded.record_events, request.record_events);
    }

    #[test]
    fn consolidation_response_roundtrips() {
        let response = MemoryConsolidationResponse {
            scanned: 42,
            groups: 7,
            consolidated: 3,
            duplicates: 1,
            events: 3,
            highlights: vec!["repo:3 events".to_string()],
        };

        let json = serde_json::to_string(&response).unwrap();
        let decoded: MemoryConsolidationResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.scanned, response.scanned);
        assert_eq!(decoded.consolidated, response.consolidated);
        assert_eq!(decoded.events, response.events);
        assert_eq!(decoded.highlights, response.highlights);
    }

    #[test]
    fn maintenance_report_roundtrips() {
        let request = MemoryMaintenanceReportRequest {
            project: Some("memd".to_string()),
            namespace: Some("agent".to_string()),
            inactive_days: Some(21),
            lookback_days: Some(14),
            min_events: Some(3),
            max_decay: Some(0.12),
            mode: Some("scan".to_string()),
            apply: Some(false),
        };

        let response = MemoryMaintenanceReportResponse {
            reinforced_candidates: 9,
            cooled_candidates: 4,
            consolidated_candidates: 2,
            stale_items: 11,
            skipped: 2,
            highlights: vec!["repo:3 events".to_string()],
            receipt_id: Some("receipt-1".to_string()),
            mode: Some("scan".to_string()),
            compacted_items: 2,
            refreshed_items: 4,
            repaired_items: 1,
            generated_at: Utc::now(),
        };

        let request_json = serde_json::to_string(&request).unwrap();
        let response_json = serde_json::to_string(&response).unwrap();
        let decoded_request: MemoryMaintenanceReportRequest =
            serde_json::from_str(&request_json).unwrap();
        let decoded_response: MemoryMaintenanceReportResponse =
            serde_json::from_str(&response_json).unwrap();
        assert_eq!(decoded_request.project, request.project);
        assert_eq!(decoded_response.stale_items, response.stale_items);
        assert_eq!(decoded_response.skipped, response.skipped);
        assert_eq!(decoded_response.highlights, response.highlights);
        assert_eq!(decoded_response.mode, response.mode);
        assert_eq!(decoded_response.receipt_id, response.receipt_id);
        assert_eq!(decoded_response.compacted_items, response.compacted_items);
        assert_eq!(decoded_response.refreshed_items, response.refreshed_items);
        assert_eq!(decoded_response.repaired_items, response.repaired_items);
    }

    #[test]
    fn maintain_report_roundtrips() {
        let request = MaintainReportRequest {
            project: Some("memd".to_string()),
            namespace: Some("agent".to_string()),
            workspace: Some("shared".to_string()),
            session: Some("session-a".to_string()),
            mode: "scan".to_string(),
            apply: false,
        };

        let response = MaintainReport {
            mode: "scan".to_string(),
            receipt_id: Some("receipt-1".to_string()),
            compacted_items: 3,
            refreshed_items: 2,
            repaired_items: 1,
            findings: vec!["memory scan complete".to_string()],
            generated_at: Utc::now(),
        };

        let request_json = serde_json::to_string(&request).unwrap();
        let response_json = serde_json::to_string(&response).unwrap();
        let decoded_request: MaintainReportRequest = serde_json::from_str(&request_json).unwrap();
        let decoded_response: MaintainReport = serde_json::from_str(&response_json).unwrap();
        assert_eq!(decoded_request.mode, request.mode);
        assert_eq!(decoded_request.apply, request.apply);
        assert_eq!(decoded_request.workspace, request.workspace);
        assert_eq!(decoded_response.mode, response.mode);
        assert_eq!(decoded_response.receipt_id, response.receipt_id);
        assert_eq!(decoded_response.compacted_items, response.compacted_items);
        assert_eq!(decoded_response.findings, response.findings);
    }

    #[test]
    fn explain_response_roundtrips() {
        let now = Utc::now();
        let response = ExplainMemoryResponse {
            route: RetrievalRoute::ProjectFirst,
            intent: RetrievalIntent::Decision,
            item: MemoryItem {
                id: Uuid::new_v4(),
                content: "prefer bundle-first config".to_string(),
                redundancy_key: Some("decision:bundle-first".to_string()),
                belief_branch: Some("mainline".to_string()),
                preferred: true,
                kind: MemoryKind::Decision,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("core".to_string()),
                visibility: MemoryVisibility::Workspace,
                source_agent: Some("codex".to_string()),
                source_system: Some("cli".to_string()),
                source_path: Some("/tmp/memd/docs/core/rag.md".to_string()),
                source_quality: Some(SourceQuality::Canonical),
                confidence: 0.92,
                ttl_seconds: None,
                created_at: now,
                updated_at: now,
                last_verified_at: Some(now),
                supersedes: vec![],
                tags: vec!["decision".to_string()],
                status: MemoryStatus::Active,
                stage: MemoryStage::Canonical,
            },
            canonical_key: "decision:bundle-first".to_string(),
            redundancy_key: "decision:bundle-first".to_string(),
            reasons: vec!["route=project_first".to_string()],
            entity: None,
            events: vec![],
            sources: vec![SourceMemoryRecord {
                source_agent: Some("codex".to_string()),
                source_system: Some("cli".to_string()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("core".to_string()),
                visibility: MemoryVisibility::Workspace,
                item_count: 4,
                active_count: 4,
                candidate_count: 0,
                derived_count: 0,
                synthetic_count: 0,
                contested_count: 0,
                avg_confidence: 0.91,
                trust_score: 0.95,
                last_seen_at: Some(now),
                tags: vec!["docs".to_string()],
            }],
            retrieval_feedback: RetrievalFeedbackSummary {
                total_retrievals: 4,
                last_retrieved_at: Some(now),
                by_surface: vec![
                    RetrievalFeedbackSurfaceCount {
                        surface: "explain".to_string(),
                        count: 2,
                    },
                    RetrievalFeedbackSurfaceCount {
                        surface: "working".to_string(),
                        count: 2,
                    },
                ],
                recent_policy_hooks: vec![
                    "route=project_first".to_string(),
                    "intent=decision".to_string(),
                ],
            },
            branch_siblings: vec![ExplainBranchSiblingRecord {
                id: Uuid::new_v4(),
                belief_branch: Some("fallback".to_string()),
                preferred: false,
                status: MemoryStatus::Contested,
                stage: MemoryStage::Canonical,
                confidence: 0.71,
                updated_at: now,
            }],
            rehydration: vec![MemoryRehydrationRecord {
                id: Some(Uuid::new_v4()),
                kind: "memory_item".to_string(),
                label: "canonical memory".to_string(),
                summary: "prefer bundle-first config".to_string(),
                reason: Some("rehydrate_primary_memory".to_string()),
                source_agent: Some("codex".to_string()),
                source_system: Some("cli".to_string()),
                source_path: Some("/tmp/memd/docs/core/rag.md".to_string()),
                source_quality: Some(SourceQuality::Canonical),
                recorded_at: Some(now),
            }],
            policy_hooks: vec![
                "route=project_first".to_string(),
                "intent=decision".to_string(),
                "source_trust_floor=0.60".to_string(),
            ],
        };

        let json = serde_json::to_string(&response).unwrap();
        let decoded: ExplainMemoryResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.item.belief_branch.as_deref(), Some("mainline"));
        assert_eq!(decoded.retrieval_feedback.total_retrievals, 4);
        assert_eq!(decoded.branch_siblings.len(), 1);
        assert_eq!(decoded.rehydration.len(), 1);
        assert_eq!(decoded.policy_hooks.len(), 3);
        assert_eq!(decoded.sources[0].trust_score, 0.95);
    }

    #[test]
    fn policy_response_roundtrips() {
        let response = MemoryPolicyResponse {
            retrieval_order: vec![
                MemoryScope::Local,
                MemoryScope::Synced,
                MemoryScope::Project,
                MemoryScope::Global,
            ],
            route_defaults: vec![
                MemoryPolicyRouteDefault {
                    intent: RetrievalIntent::CurrentTask,
                    route: RetrievalRoute::LocalFirst,
                },
                MemoryPolicyRouteDefault {
                    intent: RetrievalIntent::Preference,
                    route: RetrievalRoute::GlobalFirst,
                },
            ],
            working_memory: MemoryPolicyWorkingMemory {
                budget_chars: 1600,
                max_chars_per_item: 220,
                default_limit: 8,
                rehydration_limit: 3,
            },
            retrieval_feedback: MemoryPolicyFeedback {
                enabled: true,
                tracked_surfaces: vec![
                    "search".to_string(),
                    "context".to_string(),
                    "working".to_string(),
                    "explain".to_string(),
                ],
                max_items_per_request: 3,
            },
            source_trust_floor: 0.6,
            runtime: MemoryPolicyRuntime {
                live_truth: MemoryPolicyLiveTruth {
                    read_once_sources: true,
                    raw_reopen_requires_change_or_doubt: true,
                    visible_memory_objects: true,
                    compile_from_events: true,
                },
                memory_compilation: MemoryPolicyMemoryCompilation {
                    event_driven_updates: true,
                    patch_not_rewrite: true,
                    preserve_provenance: true,
                    source_on_demand: true,
                },
                semantic_fallback: MemoryPolicySemanticFallback {
                    enabled: true,
                    source_of_truth: false,
                    max_items_per_query: 3,
                    rerank_with_visible_memory: true,
                },
                skill_gating: MemoryPolicySkillGating {
                    propose_from_repeated_patterns: true,
                    sandboxed_evaluation: true,
                    auto_activate_low_risk_only: true,
                    gated_activation: true,
                    require_evaluation: true,
                    require_policy_approval: true,
                },
            },
            promotion: MemoryPolicyPromotion {
                min_salience: 0.22,
                min_events: 3,
                lookback_days: 14,
                default_ttl_days: 90,
            },
            decay: MemoryPolicyDecay {
                max_items: 128,
                inactive_days: 21,
                max_decay: 0.12,
                record_events: true,
            },
            consolidation: MemoryPolicyConsolidation {
                max_groups: 24,
                min_events: 3,
                lookback_days: 14,
                min_salience: 0.22,
                record_events: true,
            },
        };

        let json = serde_json::to_string(&response).unwrap();
        let decoded: MemoryPolicyResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.retrieval_order, response.retrieval_order);
        assert_eq!(decoded.working_memory.default_limit, 8);
        assert!(decoded.retrieval_feedback.enabled);
        assert_eq!(decoded.source_trust_floor, 0.6);
        assert!(decoded.runtime.live_truth.read_once_sources);
        assert!(decoded.runtime.skill_gating.gated_activation);
        assert!(decoded.runtime.skill_gating.sandboxed_evaluation);
        assert_eq!(decoded.decay.max_decay, 0.12);
        assert_eq!(decoded.consolidation.max_groups, 24);
    }

    #[test]
    fn working_memory_roundtrips() {
        let request = WorkingMemoryRequest {
            project: Some("memd".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("core".to_string()),
            visibility: Some(MemoryVisibility::Workspace),
            route: Some(RetrievalRoute::ProjectFirst),
            intent: Some(RetrievalIntent::CurrentTask),
            limit: Some(4),
            max_chars_per_item: Some(180),
            max_total_chars: Some(900),
            rehydration_limit: Some(2),
            auto_consolidate: Some(true),
        };

        let response = WorkingMemoryResponse {
            route: RetrievalRoute::ProjectFirst,
            intent: RetrievalIntent::CurrentTask,
            retrieval_order: vec![MemoryScope::Project, MemoryScope::Synced],
            budget_chars: 900,
            used_chars: 612,
            remaining_chars: 288,
            truncated: false,
            policy: WorkingMemoryPolicyState {
                admission_limit: 4,
                max_chars_per_item: 180,
                budget_chars: 900,
                rehydration_limit: 2,
            },
            records: vec![CompactMemoryRecord {
                id: Uuid::new_v4(),
                record: "focus on the working set".to_string(),
            }],
            evicted: vec![WorkingMemoryEvictionRecord {
                id: Uuid::new_v4(),
                record: "older context left the hot set".to_string(),
                reason: "evicted_by_budget".to_string(),
            }],
            rehydration_queue: vec![MemoryRehydrationRecord {
                id: Some(Uuid::new_v4()),
                kind: "working_memory_record".to_string(),
                label: "evicted working-set item".to_string(),
                summary: "older context left the hot set".to_string(),
                reason: Some("evicted_by_budget".to_string()),
                source_agent: Some("codex".to_string()),
                source_system: Some("cli".to_string()),
                source_path: Some("/tmp/memd/notes.md".to_string()),
                source_quality: Some(SourceQuality::Derived),
                recorded_at: Some(Utc::now()),
            }],
            traces: vec![WorkingMemoryTraceRecord {
                item_id: Uuid::new_v4(),
                entity_id: Some(Uuid::new_v4()),
                memory_kind: MemoryKind::Decision,
                memory_stage: MemoryStage::Canonical,
                typed_memory: "semantic+canonical".to_string(),
                event_type: "retrieved".to_string(),
                summary: "working set refreshed".to_string(),
                occurred_at: Utc::now(),
                salience_score: 0.81,
            }],
            semantic_consolidation: Some(MemoryConsolidationResponse {
                scanned: 3,
                groups: 1,
                consolidated: 1,
                duplicates: 0,
                events: 1,
                highlights: vec!["working-set replay".to_string()],
            }),
        };

        let request_json = serde_json::to_string(&request).unwrap();
        let response_json = serde_json::to_string(&response).unwrap();
        let decoded_request: WorkingMemoryRequest = serde_json::from_str(&request_json).unwrap();
        let decoded_response: WorkingMemoryResponse = serde_json::from_str(&response_json).unwrap();
        assert_eq!(decoded_request.limit, request.limit);
        assert_eq!(decoded_request.rehydration_limit, request.rehydration_limit);
        assert_eq!(decoded_response.budget_chars, response.budget_chars);
        assert_eq!(decoded_response.policy.admission_limit, 4);
        assert_eq!(decoded_response.records.len(), 1);
        assert_eq!(decoded_response.evicted.len(), 1);
        assert_eq!(decoded_response.rehydration_queue.len(), 1);
        assert_eq!(decoded_response.traces.len(), 1);
        assert_eq!(decoded_response.semantic_consolidation.is_some(), true);
    }

    #[test]
    fn agent_profile_roundtrips() {
        let request = AgentProfileUpsertRequest {
            agent: "codex".to_string(),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            preferred_route: Some(RetrievalRoute::ProjectFirst),
            preferred_intent: Some(RetrievalIntent::CurrentTask),
            summary_chars: Some(160),
            max_total_chars: Some(1200),
            recall_depth: Some(2),
            source_trust_floor: Some(0.6),
            style_tags: vec!["concise".to_string(), "token-cheap".to_string()],
            notes: Some("prefer tight working sets".to_string()),
        };
        let profile = MemoryAgentProfile {
            id: Uuid::new_v4(),
            agent: "codex".to_string(),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            preferred_route: Some(RetrievalRoute::ProjectFirst),
            preferred_intent: Some(RetrievalIntent::CurrentTask),
            summary_chars: Some(160),
            max_total_chars: Some(1200),
            recall_depth: Some(2),
            source_trust_floor: Some(0.6),
            style_tags: vec!["concise".to_string(), "token-cheap".to_string()],
            notes: Some("prefer tight working sets".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let response = AgentProfileResponse {
            profile: Some(profile),
        };

        let request_json = serde_json::to_string(&request).unwrap();
        let response_json = serde_json::to_string(&response).unwrap();
        let decoded_request: AgentProfileUpsertRequest =
            serde_json::from_str(&request_json).unwrap();
        let decoded_response: AgentProfileResponse = serde_json::from_str(&response_json).unwrap();
        assert_eq!(decoded_request.summary_chars, request.summary_chars);
        assert_eq!(
            decoded_response.profile.as_ref().unwrap().summary_chars,
            Some(160)
        );
    }

    #[test]
    fn source_memory_roundtrips() {
        let request = SourceMemoryRequest {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("core".to_string()),
            visibility: Some(MemoryVisibility::Workspace),
            source_agent: Some("codex".to_string()),
            source_system: Some("cli".to_string()),
            limit: Some(10),
        };
        let response = SourceMemoryResponse {
            sources: vec![SourceMemoryRecord {
                source_agent: Some("codex".to_string()),
                source_system: Some("cli".to_string()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("core".to_string()),
                visibility: MemoryVisibility::Workspace,
                item_count: 12,
                active_count: 10,
                candidate_count: 2,
                derived_count: 4,
                synthetic_count: 0,
                contested_count: 1,
                avg_confidence: 0.84,
                trust_score: 0.91,
                last_seen_at: Some(Utc::now()),
                tags: vec!["agent".to_string(), "cli".to_string()],
            }],
        };

        let request_json = serde_json::to_string(&request).unwrap();
        let response_json = serde_json::to_string(&response).unwrap();
        let decoded_request: SourceMemoryRequest = serde_json::from_str(&request_json).unwrap();
        let decoded_response: SourceMemoryResponse = serde_json::from_str(&response_json).unwrap();
        assert_eq!(decoded_request.source_agent, request.source_agent);
        assert_eq!(decoded_response.sources[0].trust_score, 0.91);
    }

    #[test]
    fn workspace_memory_roundtrips() {
        let request = WorkspaceMemoryRequest {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("core".to_string()),
            visibility: Some(MemoryVisibility::Workspace),
            source_agent: Some("obsidian".to_string()),
            source_system: Some("obsidian".to_string()),
            limit: Some(8),
        };
        let response = WorkspaceMemoryResponse {
            workspaces: vec![WorkspaceMemoryRecord {
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("core".to_string()),
                visibility: MemoryVisibility::Workspace,
                item_count: 12,
                active_count: 10,
                candidate_count: 1,
                contested_count: 1,
                source_lane_count: 2,
                avg_confidence: 0.86,
                trust_score: 0.9,
                last_seen_at: Some(Utc::now()),
                tags: vec!["obsidian".to_string(), "shared".to_string()],
            }],
        };

        let request_json = serde_json::to_string(&request).unwrap();
        let response_json = serde_json::to_string(&response).unwrap();
        let decoded_request: WorkspaceMemoryRequest = serde_json::from_str(&request_json).unwrap();
        let decoded_response: WorkspaceMemoryResponse =
            serde_json::from_str(&response_json).unwrap();
        assert_eq!(decoded_request.workspace, request.workspace);
        assert_eq!(decoded_response.workspaces.len(), 1);
        assert_eq!(decoded_response.workspaces[0].source_lane_count, 2);
    }

    #[test]
    fn procedural_and_self_model_enums_roundtrip() {
        let kind_json = serde_json::to_string(&MemoryKind::Procedural).unwrap();
        let intent_json = serde_json::to_string(&RetrievalIntent::SelfModel).unwrap();

        assert_eq!(kind_json, "\"procedural\"");
        assert_eq!(intent_json, "\"self_model\"");
        assert_eq!(
            serde_json::from_str::<MemoryKind>(&kind_json).unwrap(),
            MemoryKind::Procedural
        );
        assert_eq!(
            serde_json::from_str::<RetrievalIntent>(&intent_json).unwrap(),
            RetrievalIntent::SelfModel
        );
    }

    #[test]
    fn repair_contract_roundtrips() {
        let request = RepairMemoryRequest {
            id: Uuid::new_v4(),
            mode: MemoryRepairMode::CorrectMetadata,
            confidence: Some(0.91),
            status: Some(MemoryStatus::Active),
            workspace: Some("team-alpha".to_string()),
            visibility: Some(MemoryVisibility::Workspace),
            source_agent: Some("codex".to_string()),
            source_system: Some("cli".to_string()),
            source_path: Some("/tmp/memd".to_string()),
            source_quality: Some(SourceQuality::Canonical),
            content: Some("repaired memory content".to_string()),
            tags: Some(vec!["repair".to_string(), "audit".to_string()]),
            supersedes: vec![Uuid::new_v4(), Uuid::new_v4()],
        };
        let response = RepairMemoryResponse {
            item: MemoryItem {
                id: request.id,
                content: "repaired memory content".to_string(),
                redundancy_key: Some("dedupe:key".to_string()),
                belief_branch: Some("mainline".to_string()),
                preferred: false,
                kind: MemoryKind::Decision,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("core".to_string()),
                visibility: MemoryVisibility::Workspace,
                source_agent: request.source_agent.clone(),
                source_system: request.source_system.clone(),
                source_path: request.source_path.clone(),
                source_quality: request.source_quality,
                confidence: 0.91,
                ttl_seconds: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                last_verified_at: Some(Utc::now()),
                supersedes: request.supersedes.clone(),
                tags: vec!["repair".to_string(), "audit".to_string()],
                status: MemoryStatus::Active,
                stage: MemoryStage::Canonical,
            },
            mode: request.mode,
            reasons: vec![
                "mode=correct_metadata".to_string(),
                "source_agent_updated".to_string(),
                "content_repaired".to_string(),
            ],
        };

        let request_json = serde_json::to_string(&request).unwrap();
        let response_json = serde_json::to_string(&response).unwrap();
        let decoded_request: RepairMemoryRequest = serde_json::from_str(&request_json).unwrap();
        let decoded_response: RepairMemoryResponse = serde_json::from_str(&response_json).unwrap();
        assert_eq!(decoded_request.mode, MemoryRepairMode::CorrectMetadata);
        assert_eq!(decoded_request.tags.as_ref().unwrap().len(), 2);
        assert_eq!(decoded_response.mode, MemoryRepairMode::CorrectMetadata);
        assert_eq!(decoded_response.item.status, MemoryStatus::Active);
        assert_eq!(decoded_response.reasons.len(), 3);
    }

    #[test]
    fn benchmark_registry_roundtrips_minimal_continuity_slice() {
        let registry = BenchmarkRegistry {
            version: "v1".to_string(),
            app_goal: "seamless memory and continuity".to_string(),
            quality_dimensions: vec![
                QualityDimensionRecord {
                    id: "continuity".to_string(),
                    weight: 25,
                },
                QualityDimensionRecord {
                    id: "correctness".to_string(),
                    weight: 20,
                },
            ],
            tiers: vec![TierRecord {
                id: "tier-0-continuity-critical".to_string(),
                description: "continuity-critical surfaces".to_string(),
            }],
            pillars: vec![PillarRecord {
                id: "memory-continuity".to_string(),
                description: "core continuity promise".to_string(),
            }],
            families: vec![FamilyRecord {
                id: "bundle-runtime".to_string(),
                pillar: "memory-continuity".to_string(),
                description: "bundle continuity surfaces".to_string(),
            }],
            features: vec![BenchmarkFeatureRecord {
                id: "feature.bundle.resume".to_string(),
                name: "Resume".to_string(),
                pillar: "memory-continuity".to_string(),
                family: "bundle-runtime".to_string(),
                tier: "tier-0-continuity-critical".to_string(),
                continuity_critical: true,
                user_contract: "resume restores usable continuity".to_string(),
                source_contract_refs: vec!["FEATURE-V1-WORKING-CONTEXT".to_string()],
                commands: vec!["memd resume".to_string()],
                routes: vec![],
                files: vec!["crates/memd-client/src/main.rs".to_string()],
                journey_ids: vec!["journey.continuity.resume-handoff-attach".to_string()],
                loop_ids: vec!["loop.resume.correctness".to_string()],
                quality_dimensions: vec!["continuity".to_string(), "correctness".to_string()],
                drift_risks: vec!["continuity-drift".to_string()],
                failure_modes: vec!["resume misses current task state".to_string()],
                coverage_status: "auditing".to_string(),
                last_verified_at: None,
            }],
            journeys: vec![],
            loops: vec![],
            verifiers: vec![VerifierRecord {
                id: "verifier.journey.resume-handoff-attach".to_string(),
                name: "Resume handoff attach continuity".to_string(),
                verifier_type: "journey".to_string(),
                pillar: "memory-continuity".to_string(),
                family: "bundle-runtime".to_string(),
                subject_ids: vec!["journey.continuity.resume-handoff-attach".to_string()],
                fixture_id: "fixture.continuity_bundle".to_string(),
                baseline_modes: vec!["with_memd".to_string()],
                steps: vec![],
                assertions: vec![],
                metrics: vec!["prompt_tokens".to_string()],
                evidence_requirements: vec!["live_primary".to_string()],
                gate_target: "acceptable".to_string(),
                status: "declared".to_string(),
                lanes: vec!["fast".to_string()],
                helper_hooks: vec![],
            }],
            fixtures: vec![FixtureRecord {
                id: "fixture.continuity_bundle".to_string(),
                kind: "bundle_fixture".to_string(),
                description: "continuity bundle".to_string(),
                seed_files: vec![],
                seed_config: serde_json::json!({"project":"memd"}),
                seed_memories: vec![],
                seed_events: vec![],
                seed_sessions: vec![],
                seed_claims: vec![],
                seed_vault: None,
                backend_mode: "normal".to_string(),
                isolation: "fresh_temp_dir".to_string(),
                cleanup_policy: "destroy".to_string(),
            }],
            evidence_policies: vec![],
            schedules: vec![],
            scorecards: vec![],
            evidence: vec![],
            gates: vec![],
            baseline_modes: vec![],
            runtime_policies: vec![],
            generated_at: None,
        };

        let json = serde_json::to_string(&registry).unwrap();
        let decoded: BenchmarkRegistry = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.version, "v1");
        assert_eq!(decoded.features[0].id, "feature.bundle.resume");
        assert!(decoded.features[0].continuity_critical);
        assert_eq!(
            decoded.verifiers[0].id,
            "verifier.journey.resume-handoff-attach"
        );
        assert_eq!(decoded.fixtures[0].id, "fixture.continuity_bundle");
    }

    #[test]
    fn verifier_registry_roundtrips_minimal_resume_verifier() {
        let registry = BenchmarkRegistry {
            version: "v1".to_string(),
            app_goal: "seamless memory and continuity".to_string(),
            quality_dimensions: vec![],
            tiers: vec![],
            pillars: vec![],
            families: vec![],
            features: vec![],
            journeys: vec![],
            loops: vec![],
            verifiers: vec![VerifierRecord {
                id: "verifier.journey.resume-handoff-attach".to_string(),
                name: "Resume handoff attach continuity".to_string(),
                verifier_type: "journey".to_string(),
                pillar: "memory-continuity".to_string(),
                family: "bundle-runtime".to_string(),
                subject_ids: vec!["journey.continuity.resume-handoff-attach".to_string()],
                fixture_id: "fixture.continuity_bundle".to_string(),
                baseline_modes: vec!["with_memd".to_string()],
                steps: vec![],
                assertions: vec![],
                metrics: vec!["prompt_tokens".to_string()],
                evidence_requirements: vec!["live_primary".to_string()],
                gate_target: "acceptable".to_string(),
                status: "declared".to_string(),
                lanes: vec!["fast".to_string()],
                helper_hooks: vec![],
            }],
            fixtures: vec![FixtureRecord {
                id: "fixture.continuity_bundle".to_string(),
                kind: "bundle_fixture".to_string(),
                description: "continuity bundle".to_string(),
                seed_files: vec![],
                seed_config: serde_json::json!({"project":"memd"}),
                seed_memories: vec![],
                seed_events: vec![],
                seed_sessions: vec![],
                seed_claims: vec![],
                seed_vault: None,
                backend_mode: "normal".to_string(),
                isolation: "fresh_temp_dir".to_string(),
                cleanup_policy: "destroy".to_string(),
            }],
            evidence_policies: vec![],
            schedules: vec![],
            scorecards: vec![],
            evidence: vec![],
            gates: vec![],
            baseline_modes: vec![],
            runtime_policies: vec![],
            generated_at: None,
        };

        let json = serde_json::to_string(&registry).unwrap();
        let decoded: BenchmarkRegistry = serde_json::from_str(&json).unwrap();
        assert_eq!(
            decoded.verifiers[0].id,
            "verifier.journey.resume-handoff-attach"
        );
        assert_eq!(decoded.fixtures[0].id, "fixture.continuity_bundle");
    }

    #[test]
    fn benchmark_score_resolution_rules_roundtrip() {
        let rules = ScoreResolutionRules {
            cap_on_continuity_failure: "fragile".to_string(),
            cap_on_missing_required_evidence: "fragile".to_string(),
            cap_on_no_memd_loss: "acceptable".to_string(),
        };

        let json = serde_json::to_string(&rules).unwrap();
        let decoded: ScoreResolutionRules = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.cap_on_continuity_failure, "fragile");
        assert_eq!(decoded.cap_on_no_memd_loss, "acceptable");
    }

    #[test]
    fn continuity_journey_report_roundtrips() {
        let report = ContinuityJourneyReport {
            journey_id: "journey.continuity.resume-handoff-attach".to_string(),
            journey_name: "Resume To Handoff To Attach".to_string(),
            gate_decision: BenchmarkGateDecision {
                gate: "acceptable".to_string(),
                resolved_score: 75,
                reasons: vec!["continuity evidence present".to_string()],
            },
            metrics: BenchmarkSubjectMetrics {
                correctness: 90,
                continuity: 85,
                reliability: 80,
                token_efficiency: 78,
                no_memd_delta: Some(9),
            },
            evidence: BenchmarkEvidenceSummary {
                has_contract_evidence: true,
                has_workflow_evidence: true,
                has_continuity_evidence: true,
                has_comparative_evidence: true,
                has_drift_failure: false,
            },
            baseline_modes: vec![
                "baseline.no-memd".to_string(),
                "baseline.with-memd".to_string(),
            ],
            feature_ids: vec![
                "feature.bundle.resume".to_string(),
                "feature.bundle.handoff".to_string(),
            ],
            artifact_paths: vec![".memd/telemetry/continuity/latest.json".to_string()],
            summary: "resume continuity evidence".to_string(),
            generated_at: None,
        };

        let json = serde_json::to_string(&report).unwrap();
        let decoded: ContinuityJourneyReport = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.gate_decision.gate, "acceptable");
        assert!(decoded.evidence.has_continuity_evidence);
        assert_eq!(decoded.feature_ids.len(), 2);
    }

    #[test]
    fn working_memory_trace_record_accepts_legacy_entries_without_new_fields() {
        let raw = r#"{
          "item_id": "31433b72-abfd-486c-b19b-8a66dd20654d",
          "entity_id": "e70b4828-e1d6-480c-9145-bfd384c27383",
          "event_type": "retrieved_working",
          "summary": "retrieved_working route=local_first intent=current_task",
          "occurred_at": "2026-04-12T14:04:19Z",
          "salience_score": 1.0
        }"#;

        let decoded: WorkingMemoryTraceRecord = serde_json::from_str(raw).unwrap();

        assert_eq!(decoded.memory_kind, MemoryKind::Status);
        assert_eq!(decoded.memory_stage, MemoryStage::Canonical);
        assert_eq!(decoded.typed_memory, "session_continuity+canonical");
    }
}
