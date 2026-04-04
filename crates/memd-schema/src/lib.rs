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
    Topology,
    Status,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub id: Uuid,
    pub content: String,
    pub redundancy_key: Option<String>,
    pub kind: MemoryKind,
    pub scope: MemoryScope,
    pub project: Option<String>,
    pub namespace: Option<String>,
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
pub struct MemoryInboxRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
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
pub struct MemoryInboxResponse {
    pub route: RetrievalRoute,
    pub intent: RetrievalIntent,
    pub items: Vec<InboxMemoryItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainMemoryRequest {
    pub id: Uuid,
    pub route: Option<RetrievalRoute>,
    pub intent: Option<RetrievalIntent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainMemoryResponse {
    pub route: RetrievalRoute,
    pub intent: RetrievalIntent,
    pub item: MemoryItem,
    pub canonical_key: String,
    pub redundancy_key: String,
    pub reasons: Vec<String>,
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
