use super::*;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_item_id: Option<Uuid>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_item_id: Option<Uuid>,
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

// ---------------------------------------------------------------------------
// Procedural memory types (Phase G)
// ---------------------------------------------------------------------------

/// Status of a procedure in the promotion pipeline.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProcedureStatus {
    /// Observed pattern, not yet validated.
    Candidate,
    /// Validated and promoted for reuse.
    Promoted,
    /// Manually or automatically retired.
    Retired,
}

/// What kind of procedure this is.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProcedureKind {
    /// A reusable workflow (build, deploy, review, etc.).
    Workflow,
    /// An operating policy (preference, convention).
    Policy,
    /// A recovery pattern (what to do when X breaks).
    Recovery,
}

/// A learned procedure that can be retrieved and reused.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Procedure {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub kind: ProcedureKind,
    pub status: ProcedureStatus,
    /// When this procedure applies (natural language trigger condition).
    pub trigger: String,
    /// Ordered steps to execute.
    pub steps: Vec<String>,
    /// How to know it worked.
    pub success_criteria: Option<String>,
    /// Source memory items that evidenced this procedure.
    pub source_ids: Vec<Uuid>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    /// How many times this procedure was successfully applied.
    pub use_count: usize,
    pub confidence: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
    /// Number of distinct sessions that have used this procedure.
    #[serde(default)]
    pub session_count: usize,
    /// Last session that used this procedure.
    #[serde(default)]
    pub last_session: Option<String>,
    /// ID of the procedure this one supersedes (X1: correction integration).
    #[serde(default)]
    pub supersedes: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProcedureListRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub kind: Option<ProcedureKind>,
    pub status: Option<ProcedureStatus>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureListResponse {
    pub procedures: Vec<Procedure>,
}

/// Request to retrieve procedures relevant to current context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureMatchRequest {
    pub context: String,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureMatchResponse {
    pub procedures: Vec<Procedure>,
}

/// Request to record a new procedure (explicit capture).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureRecordRequest {
    pub name: String,
    pub description: String,
    pub kind: ProcedureKind,
    pub trigger: String,
    pub steps: Vec<String>,
    pub success_criteria: Option<String>,
    pub source_ids: Vec<Uuid>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub tags: Vec<String>,
    /// ID of a procedure this one supersedes (X1: correction integration).
    #[serde(default)]
    pub supersedes: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureRecordResponse {
    pub procedure: Procedure,
    /// Existing promoted procedures with overlapping triggers (conflict detection).
    #[serde(default)]
    pub conflicts: Vec<Procedure>,
}

/// Request to promote a candidate procedure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedurePromoteRequest {
    pub procedure_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedurePromoteResponse {
    pub procedure: Procedure,
}

/// Request to record a successful use of a procedure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureUseRequest {
    pub procedure_id: Uuid,
    /// Session recording this use (for cross-session tracking).
    #[serde(default)]
    pub session: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureUseResponse {
    pub procedure: Procedure,
    /// Whether the procedure was auto-promoted by crossing use/session thresholds.
    #[serde(default)]
    pub auto_promoted: bool,
}

/// Request to retire a procedure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureRetireRequest {
    pub procedure_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureRetireResponse {
    pub procedure: Procedure,
}

/// Request to detect candidate procedures from episodic event patterns.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProcedureDetectRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
    /// Minimum event occurrences for an entity to be considered (default 3).
    pub min_events: Option<usize>,
    /// How many days back to scan (default 14).
    pub lookback_days: Option<i64>,
    /// Max candidate procedures to generate (default 5).
    pub max_candidates: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureDetectResponse {
    /// Entities scanned.
    pub scanned: usize,
    /// Candidate procedures created.
    pub created: usize,
    /// Procedures that were created.
    pub procedures: Vec<Procedure>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryDecayRequest {
    pub max_items: Option<usize>,
    pub inactive_days: Option<i64>,
    pub max_decay: Option<f32>,
    pub record_events: Option<bool>,
    /// Divisor controlling decay acceleration past inactive threshold.
    /// decay = (idle_days_over / decay_divisor).min(1.0) * max_decay
    /// Defaults to 14.0.
    #[serde(default)]
    pub decay_divisor: Option<f32>,
}

/// Age distribution of entities scanned during a decay pass.
/// Buckets are based on idle_days (days since last access/seen/updated).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DecayAgeDistribution {
    pub under_7d: usize,
    pub d7_to_14: usize,
    pub d14_to_21: usize,
    pub d21_to_30: usize,
    pub over_30d: usize,
}

/// Salience distribution: 10 buckets [0.0,0.1), [0.1,0.2), ..., [0.9,1.0].
/// Index i covers salience in [i*0.1, (i+1)*0.1).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SalienceHistogram {
    pub buckets: Vec<usize>,
}

impl SalienceHistogram {
    pub fn new() -> Self {
        Self {
            buckets: vec![0; 10],
        }
    }

    pub fn record(&mut self, salience: f32) {
        let idx = ((salience * 10.0) as usize).min(9);
        self.buckets[idx] += 1;
    }
}

/// Metrics collected during a single decay run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayRunMetrics {
    /// Items read and evaluated.
    pub inspected: usize,
    /// Items that had salience reduced.
    pub decayed: usize,
    /// Items that reached salience == 0.0 after decay.
    pub zeroed: usize,
    /// Sum of all decay deltas applied across all items.
    pub total_decay_applied: f32,
    /// Distribution of idle age across inspected items.
    pub age_distribution: DecayAgeDistribution,
    /// Salience distribution before decay was applied.
    pub salience_pre: SalienceHistogram,
    /// Salience distribution after decay was applied (for items that changed).
    pub salience_post: SalienceHistogram,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDecayResponse {
    pub scanned: usize,
    pub updated: usize,
    pub events: usize,
    #[serde(default)]
    pub metrics: Option<DecayRunMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayDiagnosticsResponse {
    pub metrics: DecayRunMetrics,
    pub inactive_days: usize,
    pub max_decay: f32,
    pub decay_divisor: f32,
    pub max_items: usize,
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

/// Per-item quality score computed after a consolidation write.
/// Scores are in [0.0, 1.0] and reflect how well the generated item preserves
/// source fidelity across four dimensions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationQualityScore {
    /// Entity type present in consolidated content (semantic coherence).
    pub semantic_coherence: f32,
    /// Clause/sentence density vs event count (information preservation).
    pub information_preservation: f32,
    /// Consolidated kind matches expected kind from entity type (1.0 or 0.0).
    pub kind_preserved: f32,
    /// Consolidated visibility matches most-restrictive source visibility (1.0 or 0.0).
    pub visibility_preserved: f32,
    /// Average of the four dimension scores.
    pub overall: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConsolidationResponse {
    pub scanned: usize,
    pub groups: usize,
    pub consolidated: usize,
    pub duplicates: usize,
    pub events: usize,
    pub highlights: Vec<String>,
    /// Mean quality score across all consolidated items in this run.
    #[serde(default)]
    pub mean_quality: Option<f32>,
    /// Per-item quality scores (one per consolidated item, in consolidation order).
    #[serde(default)]
    pub quality_scores: Vec<ConsolidationQualityScore>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryDrainRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub max_items: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDrainResponse {
    pub deleted: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboxDismissRequest {
    pub ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboxDismissResponse {
    pub dismissed: usize,
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
    /// Divisor controlling decay acceleration past inactive threshold. Default: 14.0.
    pub decay_divisor: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPolicyConsolidation {
    pub max_groups: usize,
    pub min_events: usize,
    pub lookback_days: i64,
    pub min_salience: f32,
    pub record_events: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryPolicyRuntime {
    pub live_truth: MemoryPolicyLiveTruth,
    pub memory_compilation: MemoryPolicyMemoryCompilation,
    pub semantic_fallback: MemoryPolicySemanticFallback,
    pub skill_gating: MemoryPolicySkillGating,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryPolicyLiveTruth {
    pub read_once_sources: bool,
    pub raw_reopen_requires_change_or_doubt: bool,
    pub visible_memory_objects: bool,
    pub compile_from_events: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryPolicyMemoryCompilation {
    pub event_driven_updates: bool,
    pub patch_not_rewrite: bool,
    pub preserve_provenance: bool,
    pub source_on_demand: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryPolicySemanticFallback {
    pub enabled: bool,
    pub source_of_truth: bool,
    pub max_items_per_query: usize,
    pub rerank_with_visible_memory: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryPolicySkillGating {
    pub propose_from_repeated_patterns: bool,
    pub sandboxed_evaluation: bool,
    pub auto_activate_low_risk_only: bool,
    pub gated_activation: bool,
    pub require_evaluation: bool,
    pub require_policy_approval: bool,
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
    #[serde(default)]
    pub corrections_chain: Vec<CorrectionChainEntry>,
    #[serde(default)]
    pub confidence_timeline: Vec<ConfidenceSample>,
    #[serde(default)]
    pub trust_rank_history: Vec<TrustRankSample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionChainEntry {
    pub id: Uuid,
    pub content_preview: String,
    pub confidence: f32,
    pub stage: MemoryStage,
    pub status: MemoryStatus,
    pub updated_at: DateTime<Utc>,
    pub supersedes: Vec<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correction_source_turn: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceSample {
    pub at: DateTime<Utc>,
    pub confidence: f32,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustRankSample {
    pub at: DateTime<Utc>,
    pub source_agent: Option<String>,
    pub source_system: Option<String>,
    pub event_type: String,
    pub confidence: f32,
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
pub struct PressureMetrics {
    pub inbox: usize,
    pub candidates: usize,
    pub stale: usize,
    pub expired: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagHealthStatus {
    pub enabled: bool,
    pub reachable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indexed_count: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_sync_status: Option<String>,
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub recent_failures: u64,
}

fn is_zero_u64(v: &u64) -> bool {
    *v == 0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasHealthStatus {
    pub edges_total: usize,
    pub edges_active: usize,
    pub edges_dormant: usize,
    pub region_count: usize,
    pub edge_item_ratio: f64,
    pub dormant: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub items: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eval_score: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pressure: Option<PressureMetrics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rag: Option<RagHealthStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub atlas: Option<AtlasHealthStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryHealthBreakdown {
    pub total: usize,
    pub active: usize,
    pub stale: usize,
    pub superseded: usize,
    pub contested: usize,
    pub expired: usize,
    pub candidates: usize,
    pub canonical: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyBucket {
    pub upper_ms: u64,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyDiagnosticsResponse {
    pub surface: String,
    pub total: u64,
    #[serde(default)]
    pub recent_total: u64,
    pub mean_ms: f64,
    pub max_ms: u64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    #[serde(default)]
    pub recent_p95_ms: f64,
    pub buckets: Vec<LatencyBucket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpineViolation {
    pub entity_id: Uuid,
    pub earlier_event_id: Uuid,
    pub later_event_id: Uuid,
    pub earlier_recorded_at: DateTime<Utc>,
    pub later_recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpineVerifyResponse {
    pub scanned: u64,
    pub monotonic_violations: u64,
    pub first_violation: Option<SpineViolation>,
    pub rolling_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessStatus {
    pub git_branch: String,
    pub git_commit: String,
    pub git_dirty: String,
    pub memory: MemoryHealthBreakdown,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_p95_ms: Option<f64>,
    pub benchmark_gate: String,
    #[serde(default)]
    pub schema_version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub atlas: Option<AtlasHealthStatus>,
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

// ── Ingestion Pipeline (F2) ──────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestLanesRequest {
    pub root: String,
    pub project: Option<String>,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestLanesResponse {
    pub files_scanned: usize,
    pub files_ingested: usize,
    pub files_skipped: usize,
    pub files_stale: usize,
}

/// E3-D2: session boundaries derived from event-spine gaps.
/// A new session starts when the idle gap between consecutive events
/// exceeds `session_gap_seconds` (default 30min = 1800s).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSpan {
    pub id: Uuid,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub event_count: usize,
    pub memory_ids: Vec<Uuid>,
}

/// E3-D2: episode = a session's events consolidated into a narrative.
/// `narrative` is prose — subject to FTS5 index for cross-session recall.
/// `session_id` scopes the episode to one boundary; `fact_count` = number
/// of linked memory items in `episode_facts`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub id: Uuid,
    pub mind: Option<String>,
    pub title: String,
    pub narrative: String,
    pub session_id: Uuid,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub fact_count: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeFactRelation {
    Origin,
    Evidence,
    Reference,
    Outcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeFactLink {
    pub episode_id: Uuid,
    pub fact_id: Uuid,
    pub relation: EpisodeFactRelation,
}

/// E3-D2 request: consolidate recent sessions into episodes.
/// - `since`: only consider events after this timestamp (default = last 24h)
/// - `session_gap_seconds`: gap threshold for session boundaries (default 1800)
/// - `dry_run`: detect sessions but do not persist episodes
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConsolidateEpisodesRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub session_gap_seconds: Option<u64>,
    #[serde(default)]
    pub dry_run: bool,
}

/// E3-D2 response. `idempotent_skipped` = sessions already consolidated
/// (same session_id already has an episode). Must be non-zero on second
/// run of the same window for idempotency proof.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConsolidateEpisodesResponse {
    pub sessions_detected: usize,
    pub episodes_created: Vec<Episode>,
    pub idempotent_skipped: usize,
    pub total_events_scanned: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListEpisodesRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub limit: Option<usize>,
    pub query: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListEpisodesResponse {
    pub episodes: Vec<Episode>,
}

/// E3-D5 request: scan existing vectors in scope, cluster near-duplicates
/// by cosine distance, preview which rows would be merged under
/// `MEMD_STORE_DEDUP=1`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DedupScanRequest {
    pub project: Option<String>,
    pub namespace: Option<String>,
    /// Cosine distance threshold (default: 0.15).
    pub threshold_cosine_distance: Option<f32>,
    /// Cap on clusters returned (default: 50).
    pub limit: Option<usize>,
    /// Reserved: when false, the server would actually merge. For now
    /// D5 only supports dry_run=true.
    #[serde(default = "default_dry_run_true")]
    pub dry_run: bool,
}

fn default_dry_run_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DedupDuplicate {
    pub id: Uuid,
    pub similarity: f32,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DedupCluster {
    /// Richest survivor (highest confidence, ties broken by updated_at desc).
    pub survivor_id: Uuid,
    pub survivor_preview: String,
    pub duplicates: Vec<DedupDuplicate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DedupScanResponse {
    pub clusters: Vec<DedupCluster>,
    pub vectors_scanned: usize,
    pub threshold_cosine_distance: f32,
}
