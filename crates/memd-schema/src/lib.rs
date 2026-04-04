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
pub struct ExplainMemoryResponse {
    pub route: RetrievalRoute,
    pub intent: RetrievalIntent,
    pub item: MemoryItem,
    pub canonical_key: String,
    pub redundancy_key: String,
    pub reasons: Vec<String>,
    pub entity: Option<MemoryEntityRecord>,
    pub events: Vec<MemoryEventRecord>,
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
        };

        let json = serde_json::to_string(&response).unwrap();
        let decoded: MemoryConsolidationResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.scanned, response.scanned);
        assert_eq!(decoded.consolidated, response.consolidated);
        assert_eq!(decoded.events, response.events);
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
}
