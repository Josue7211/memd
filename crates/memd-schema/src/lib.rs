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
    pub rehydration_queue: Vec<WorkingMemoryRehydrationRecord>,
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
pub struct WorkingMemoryRehydrationRecord {
    pub id: Uuid,
    pub record: String,
    pub reason: String,
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
pub struct WorkingMemoryTraceRecord {
    pub item_id: Uuid,
    pub entity_id: Option<Uuid>,
    pub event_type: String,
    pub summary: String,
    pub occurred_at: DateTime<Utc>,
    pub salience_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryInboxRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMaintenanceReportResponse {
    pub reinforced_candidates: usize,
    pub cooled_candidates: usize,
    pub consolidated_candidates: usize,
    pub stale_items: usize,
    pub skipped: usize,
    pub highlights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPolicyResponse {
    pub retrieval_order: Vec<MemoryScope>,
    pub route_defaults: Vec<MemoryPolicyRouteDefault>,
    pub working_memory: MemoryPolicyWorkingMemory,
    pub retrieval_feedback: MemoryPolicyFeedback,
    pub source_trust_floor: f32,
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
    pub artifact_trail: Vec<ExplainArtifactRecord>,
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
pub struct ExplainArtifactRecord {
    pub kind: String,
    pub label: String,
    pub summary: String,
    pub source_agent: Option<String>,
    pub source_system: Option<String>,
    pub source_path: Option<String>,
    pub source_quality: Option<SourceQuality>,
    pub recorded_at: Option<DateTime<Utc>>,
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
            source_agent: Some("codex".to_string()),
            source_system: Some("cli".to_string()),
            source_path: Some("/tmp/memd".to_string()),
            related_entity_ids: vec![Uuid::new_v4()],
            tags: vec!["identity".to_string(), "timeline".to_string()],
            context: Some(MemoryContextFrame {
                at: Some(Utc::now()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
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
        };

        let response = MemoryMaintenanceReportResponse {
            reinforced_candidates: 9,
            cooled_candidates: 4,
            consolidated_candidates: 2,
            stale_items: 11,
            skipped: 2,
            highlights: vec!["repo:3 events".to_string()],
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
                source_agent: Some("codex".to_string()),
                source_system: Some("cli".to_string()),
                source_path: Some("/tmp/memd/docs/rag.md".to_string()),
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
            artifact_trail: vec![ExplainArtifactRecord {
                kind: "memory_item".to_string(),
                label: "canonical memory".to_string(),
                summary: "prefer bundle-first config".to_string(),
                source_agent: Some("codex".to_string()),
                source_system: Some("cli".to_string()),
                source_path: Some("/tmp/memd/docs/rag.md".to_string()),
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
        assert_eq!(decoded.artifact_trail.len(), 1);
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
        assert_eq!(decoded.decay.max_decay, 0.12);
        assert_eq!(decoded.consolidation.max_groups, 24);
    }

    #[test]
    fn working_memory_roundtrips() {
        let request = WorkingMemoryRequest {
            project: Some("memd".to_string()),
            agent: Some("codex".to_string()),
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
            rehydration_queue: vec![WorkingMemoryRehydrationRecord {
                id: Uuid::new_v4(),
                record: "older context left the hot set".to_string(),
                reason: "evicted_by_budget".to_string(),
            }],
            traces: vec![WorkingMemoryTraceRecord {
                item_id: Uuid::new_v4(),
                entity_id: Some(Uuid::new_v4()),
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
}
