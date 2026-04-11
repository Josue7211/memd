use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ProjectBootstrapBundle {
    pub markdown: String,
    pub registry: BootstrapSourceRegistry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CapabilityRegistry {
    pub generated_at: DateTime<Utc>,
    pub project_root: Option<String>,
    pub capabilities: Vec<CapabilityRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CapabilityBridgeRegistry {
    pub generated_at: DateTime<Utc>,
    pub actions: Vec<CapabilityBridgeAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CapabilityBridgeAction {
    pub harness: String,
    pub capability: String,
    pub status: String,
    pub source_path: String,
    pub target_path: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SkillLifecycleReport {
    pub generated_at: DateTime<Utc>,
    pub proposed: usize,
    pub sandbox_passed: usize,
    pub sandbox_review: usize,
    pub sandbox_blocked: usize,
    pub activation_candidates: usize,
    pub activated: usize,
    pub review_queue: Vec<SkillLifecycleRecord>,
    pub activate_queue: Vec<SkillLifecycleRecord>,
    pub records: Vec<SkillLifecycleRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SkillPolicyBatchArtifact {
    pub generated_at: DateTime<Utc>,
    pub bundle_root: String,
    pub runtime_defaulted: bool,
    pub report: SkillLifecycleReport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SkillPolicyQueueArtifact {
    pub generated_at: DateTime<Utc>,
    pub bundle_root: String,
    pub runtime_defaulted: bool,
    pub queue: String,
    pub records: Vec<SkillLifecycleRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SkillPolicyApplyArtifact {
    pub generated_at: DateTime<Utc>,
    pub bundle_root: String,
    pub runtime_defaulted: bool,
    pub source_queue_path: String,
    pub applied_count: usize,
    pub skipped_count: usize,
    pub applied: Vec<SkillLifecycleRecord>,
    pub skipped: Vec<SkillLifecycleRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SkillLifecycleRecord {
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
pub(crate) struct CapabilityRecord {
    pub harness: String,
    pub kind: String,
    pub name: String,
    pub status: String,
    pub portability_class: String,
    pub source_path: String,
    pub bridge_hint: Option<String>,
    pub hash: Option<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BundleMigrationManifest {
    pub generated_at: DateTime<Utc>,
    pub project_root: Option<String>,
    pub source_registry_hash: Option<String>,
    pub source_registry_path: Option<String>,
    pub layer_summary: Vec<BundleMigrationLayer>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BundleMigrationLayer {
    pub layer: String,
    pub sources: usize,
    pub summary: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BootstrapSourceRegistry {
    pub project: String,
    pub project_root: String,
    pub imported_at: DateTime<Utc>,
    pub sources: Vec<BootstrapSourceRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HarnessBridgeRegistry {
    pub generated_at: DateTime<Utc>,
    pub overall_portability_class: String,
    pub all_wired: bool,
    pub harnesses: Vec<HarnessBridgeRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HarnessBridgeRecord {
    pub harness: String,
    pub wired: bool,
    pub portability_class: String,
    pub required_surfaces: Vec<String>,
    pub missing_surfaces: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BootstrapSourceRecord {
    pub path: String,
    pub kind: String,
    pub hash: String,
    pub bytes: usize,
    pub lines: usize,
    #[serde(default)]
    pub present: bool,
    #[serde(default)]
    pub imported_at: DateTime<Utc>,
    #[serde(default)]
    pub modified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BundleEvalResponse {
    pub bundle_root: String,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub agent: Option<String>,
    pub workspace: Option<String>,
    pub visibility: Option<String>,
    pub status: String,
    pub score: u8,
    pub working_records: usize,
    pub context_records: usize,
    pub rehydration_items: usize,
    pub inbox_items: usize,
    pub workspace_lanes: usize,
    pub semantic_hits: usize,
    pub findings: Vec<String>,
    pub baseline_score: Option<u8>,
    pub score_delta: Option<i32>,
    pub changes: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct GapCandidate {
    pub id: String,
    pub area: String,
    pub priority: u8,
    pub severity: String,
    pub signal: String,
    pub evidence: Vec<String>,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ImprovementGapSnapshot {
    pub candidate_count: usize,
    pub high_priority_count: usize,
    pub eval_status: Option<String>,
    pub eval_score: Option<u8>,
    pub eval_score_delta: Option<i32>,
    pub top_priorities: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ImprovementAction {
    pub action: String,
    pub priority: String,
    pub target_session: Option<String>,
    pub scope: Option<String>,
    pub task_id: Option<String>,
    pub message_id: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ImprovementActionResult {
    pub action: String,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ImprovementIteration {
    pub iteration: usize,
    pub pre_gap: ImprovementGapSnapshot,
    pub planned_actions: Vec<ImprovementAction>,
    pub executed_actions: Vec<ImprovementActionResult>,
    pub post_gap: Option<ImprovementGapSnapshot>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ImprovementReport {
    pub bundle_root: String,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub agent: Option<String>,
    pub session: Option<String>,
    pub workspace: Option<String>,
    pub visibility: Option<String>,
    pub max_iterations: usize,
    pub apply: bool,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub converged: bool,
    pub initial_gap: Option<ImprovementGapSnapshot>,
    pub final_gap: Option<ImprovementGapSnapshot>,
    pub final_changes: Vec<String>,
    pub iterations: Vec<ImprovementIteration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ScenarioCheck {
    pub name: String,
    pub status: String,
    pub points: u16,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ScenarioReport {
    pub bundle_root: String,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub agent: Option<String>,
    pub session: Option<String>,
    pub workspace: Option<String>,
    pub visibility: Option<String>,
    pub scenario: String,
    pub score: u16,
    pub max_score: u16,
    pub checks: Vec<ScenarioCheck>,
    pub passed_checks: usize,
    pub failed_checks: usize,
    pub findings: Vec<String>,
    pub next_actions: Vec<String>,
    pub evidence: Vec<String>,
    pub generated_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CompositeDimension {
    pub name: String,
    pub weight: u8,
    pub score: u8,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CompositeGate {
    pub name: String,
    pub status: String,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CompositeReport {
    pub bundle_root: String,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub agent: Option<String>,
    pub session: Option<String>,
    pub workspace: Option<String>,
    pub visibility: Option<String>,
    pub scenario: Option<String>,
    pub score: u8,
    pub max_score: u8,
    pub dimensions: Vec<CompositeDimension>,
    pub gates: Vec<CompositeGate>,
    pub findings: Vec<String>,
    pub recommendations: Vec<String>,
    pub evidence: Vec<String>,
    pub generated_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FeatureBenchmarkArea {
    pub slug: String,
    pub name: String,
    pub score: u8,
    pub max_score: u8,
    pub status: String,
    pub implemented_commands: usize,
    pub expected_commands: usize,
    pub evidence: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FeatureBenchmarkReport {
    pub bundle_root: String,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub agent: Option<String>,
    pub session: Option<String>,
    pub workspace: Option<String>,
    pub visibility: Option<String>,
    pub score: u8,
    pub max_score: u8,
    pub command_count: usize,
    pub skill_count: usize,
    pub pack_count: usize,
    pub memory_pages: usize,
    pub event_count: usize,
    pub areas: Vec<FeatureBenchmarkArea>,
    pub evidence: Vec<String>,
    pub recommendations: Vec<String>,
    pub generated_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PublicBenchmarkManifest {
    pub benchmark_id: String,
    pub benchmark_version: String,
    pub dataset_name: String,
    pub dataset_source_url: String,
    pub dataset_local_path: String,
    pub dataset_checksum: String,
    pub dataset_split: String,
    pub git_sha: Option<String>,
    pub dirty_worktree: bool,
    pub run_timestamp: DateTime<Utc>,
    pub mode: String,
    pub top_k: usize,
    pub reranker_id: Option<String>,
    pub reranker_provider: Option<String>,
    pub limit: Option<usize>,
    pub runtime_settings: JsonValue,
    pub hardware_summary: String,
    pub duration_ms: u128,
    pub token_usage: Option<JsonValue>,
    pub cost_estimate_usd: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PublicBenchmarkDatasetFixture {
    pub benchmark_id: String,
    pub benchmark_name: String,
    pub version: String,
    pub split: String,
    pub description: String,
    pub items: Vec<PublicBenchmarkDatasetFixtureItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PublicBenchmarkDatasetFixtureItem {
    pub item_id: String,
    pub question_id: String,
    pub query: String,
    pub claim_class: String,
    pub gold_answer: String,
    pub metadata: JsonValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PublicBenchmarkDatasetCacheMetadata {
    pub benchmark_id: String,
    pub source_url: String,
    pub local_path: String,
    pub checksum: String,
    pub expected_checksum: Option<String>,
    pub verification_status: String,
    pub fetched_at: DateTime<Utc>,
    pub bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PublicBenchmarkItemResult {
    pub item_id: String,
    pub question_id: String,
    pub claim_class: String,
    #[serde(default)]
    pub question: Option<String>,
    #[serde(default)]
    pub question_type: Option<String>,
    #[serde(default)]
    pub ranked_items: Vec<JsonValue>,
    pub retrieved_items: Vec<JsonValue>,
    pub retrieval_scores: Vec<f64>,
    pub hit: bool,
    #[serde(default)]
    pub answer: Option<String>,
    #[serde(default)]
    pub observed_answer: Option<String>,
    #[serde(default)]
    pub correctness: Option<JsonValue>,
    pub latency_ms: u128,
    #[serde(default)]
    pub token_usage: Option<JsonValue>,
    #[serde(default)]
    pub cost_estimate_usd: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PublicBenchmarkRunReport {
    pub manifest: PublicBenchmarkManifest,
    pub metrics: BTreeMap<String, f64>,
    pub item_count: usize,
    pub failures: Vec<JsonValue>,
    pub items: Vec<PublicBenchmarkItemResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LongMemEvalRetrievalBackend {
    Lexical,
    Sidecar,
}

#[derive(Debug, Clone)]
pub(crate) struct PublicBenchmarkRetrievalConfig {
    pub longmemeval_backend: LongMemEvalRetrievalBackend,
    pub sidecar_base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PublicBenchmarkLeaderboardRow {
    pub benchmark_id: String,
    pub benchmark_name: String,
    pub benchmark_version: String,
    pub run_mode: String,
    pub item_claim_classes: Vec<String>,
    pub coverage_status: String,
    pub parity_status: String,
    pub accuracy: f64,
    pub item_count: usize,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PublicBenchmarkLeaderboardReport {
    pub generated_at: DateTime<Utc>,
    pub governance_notes: Vec<String>,
    pub rows: Vec<PublicBenchmarkLeaderboardRow>,
}

#[derive(Debug, Clone)]
pub(crate) struct PublicBenchmarkRunArtifactReceipt {
    pub run_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub results_path: PathBuf,
    pub results_jsonl_path: PathBuf,
    pub report_path: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct PublicBenchmarkDatasetSource {
    pub benchmark_id: &'static str,
    pub source_url: Option<&'static str>,
    pub default_filename: &'static str,
    pub expected_checksum: Option<&'static str>,
    pub split: &'static str,
    pub access_mode: &'static str,
    pub notes: &'static str,
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedPublicBenchmarkDataset {
    pub path: PathBuf,
    pub source_url: String,
    pub checksum: String,
    pub split: String,
    pub verification_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ExperimentReport {
    pub bundle_root: String,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub agent: Option<String>,
    pub session: Option<String>,
    pub workspace: Option<String>,
    pub visibility: Option<String>,
    pub max_iterations: usize,
    pub accept_below: u8,
    pub apply: bool,
    pub consolidate: bool,
    pub accepted: bool,
    pub restored: bool,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub improvement: ImprovementReport,
    pub composite: CompositeReport,
    pub trail: Vec<String>,
    pub learnings: Vec<String>,
    pub findings: Vec<String>,
    pub recommendations: Vec<String>,
    pub evidence: Vec<String>,
    pub evolution: Option<ExperimentEvolutionSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ExperimentEvolutionSummary {
    pub proposal_state: String,
    pub scope_class: String,
    pub scope_gate: String,
    pub authority_tier: String,
    pub merge_status: String,
    pub durability_status: String,
    pub branch: String,
    pub durable_truth: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EvolutionProposalReport {
    pub bundle_root: String,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub agent: Option<String>,
    pub session: Option<String>,
    pub workspace: Option<String>,
    pub visibility: Option<String>,
    pub proposal_id: String,
    pub scenario: Option<String>,
    pub topic: String,
    pub branch: String,
    pub state: String,
    pub scope_class: String,
    pub scope_gate: String,
    #[serde(default = "default_evolution_authority_tier")]
    pub authority_tier: String,
    pub allowed_write_surface: Vec<String>,
    pub merge_eligible: bool,
    pub durable_truth: bool,
    pub accepted: bool,
    pub restored: bool,
    pub composite_score: u8,
    pub composite_max: u8,
    pub evidence: Vec<String>,
    pub scope_reasons: Vec<String>,
    pub generated_at: DateTime<Utc>,
    pub durability_due_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EvolutionDurabilityEntry {
    pub proposal_id: String,
    pub branch: String,
    pub branch_prefix: String,
    pub state: String,
    pub scope_class: String,
    pub scope_gate: String,
    pub merge_eligible: bool,
    pub durable_truth: bool,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct EvolutionDurabilityLedger {
    pub entries: Vec<EvolutionDurabilityEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EvolutionBranchManifest {
    pub proposal_id: String,
    pub branch: String,
    pub branch_prefix: String,
    pub project_root: Option<String>,
    pub head_sha: Option<String>,
    pub base_branch: Option<String>,
    pub status: String,
    pub merge_eligible: bool,
    pub durable_truth: bool,
    pub scope_class: String,
    pub scope_gate: String,
    pub generated_at: DateTime<Utc>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EvolutionAuthorityEntry {
    pub scope_class: String,
    pub authority_tier: String,
    pub accepted: bool,
    pub merged: bool,
    pub durable_truth: bool,
    pub proposal_id: String,
    pub branch: String,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct EvolutionAuthorityLedger {
    pub entries: Vec<EvolutionAuthorityEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EvolutionMergeQueueEntry {
    pub proposal_id: String,
    pub branch: String,
    pub scope_class: String,
    pub scope_gate: String,
    pub authority_tier: String,
    pub status: String,
    pub merge_eligible: bool,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct EvolutionMergeQueue {
    pub entries: Vec<EvolutionMergeQueueEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EvolutionDurabilityQueueEntry {
    pub proposal_id: String,
    pub branch: String,
    pub state: String,
    pub status: String,
    pub due_at: Option<DateTime<Utc>>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct EvolutionDurabilityQueue {
    pub entries: Vec<EvolutionDurabilityQueueEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct GapReport {
    pub bundle_root: String,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub agent: Option<String>,
    pub session: Option<String>,
    pub workspace: Option<String>,
    pub visibility: Option<String>,
    pub limit: usize,
    pub commits_checked: usize,
    pub eval_status: Option<String>,
    pub eval_score: Option<u8>,
    pub eval_score_delta: Option<i32>,
    pub candidate_count: usize,
    pub high_priority_count: usize,
    pub top_priorities: Vec<String>,
    pub candidates: Vec<GapCandidate>,
    pub recommendations: Vec<String>,
    pub changes: Vec<String>,
    pub evidence: Vec<String>,
    pub generated_at: DateTime<Utc>,
    pub previous_candidate_count: Option<usize>,
}
