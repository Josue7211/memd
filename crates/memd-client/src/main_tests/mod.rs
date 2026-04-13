use super::*;
use std::sync::{Arc, Mutex, OnceLock};

use crate::render::{
    render_agent_zero_harness_pack_markdown, render_claude_code_harness_pack_markdown,
    render_codex_harness_pack_markdown, render_command_catalog_markdown,
    render_command_catalog_summary, render_hermes_harness_pack_markdown,
    render_openclaw_harness_pack_markdown, render_opencode_harness_pack_markdown,
};
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
};
use memd_schema::{
    BenchmarkEvidenceSummary, BenchmarkFeatureRecord, BenchmarkGateDecision,
    BenchmarkSubjectMetrics, ContinuityJourneyReport, HiveClaimAcquireRequest, HiveClaimRecord,
    HiveClaimReleaseRequest, HiveClaimTransferRequest, HiveClaimsRequest, HiveClaimsResponse,
    HiveCoordinationInboxResponse, HiveCoordinationReceiptRecord, HiveCoordinationReceiptRequest,
    HiveCoordinationReceiptsResponse, HiveMessageAckRequest, HiveMessageInboxRequest,
    HiveMessageRecord, HiveMessageSendRequest, HiveMessagesResponse, HiveTaskRecord,
    SkillPolicyActivationRecord, SkillPolicyApplyReceipt, SkillPolicyApplyReceiptsRequest,
    SkillPolicyApplyReceiptsResponse, SkillPolicyApplyRequest, SkillPolicyApplyResponse,
    VerifierAssertionRecord, VerifierStepRecord,
};

static HOME_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn lock_home_mutation() -> std::sync::MutexGuard<'static, ()> {
    HOME_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("HOME mutation lock poisoned")
}

fn lock_env_mutation() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("env mutation lock poisoned")
}

fn normalize_path_text(value: impl AsRef<Path>) -> String {
    value.as_ref().to_string_lossy().replace('\\', "/")
}

fn path_text_contains(value: impl AsRef<Path>, needle: &str) -> bool {
    normalize_path_text(value).contains(needle)
}

fn public_benchmark_fixture_path(dataset: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../fixtures/{dataset}-mini.json"))
}

#[allow(dead_code)]
fn assert_path_tail(actual: &str, expected: &Path) {
    let expected = fs::canonicalize(expected).unwrap_or_else(|_| expected.to_path_buf());
    assert!(
        Path::new(actual).ends_with(&expected),
        "path {actual:?} did not end with {expected:?}"
    );
}

fn codex_test_snapshot(project: &str, namespace: &str, agent: &str) -> ResumeSnapshot {
    ResumeSnapshot {
        project: Some(project.to_string()),
        namespace: Some(namespace.to_string()),
        agent: Some(agent.to_string()),
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
                record: "keep the live wake surface current".to_string(),
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
                record: "follow the codex pack turn boundary".to_string(),
            }],
            evicted: Vec::new(),
            rehydration_queue: vec![memd_schema::MemoryRehydrationRecord {
                id: None,
                kind: "source".to_string(),
                label: "handoff".to_string(),
                summary: "reload the bundled wake and memory files".to_string(),
                reason: None,
                source_agent: None,
                source_system: None,
                source_path: None,
                source_quality: None,
                recorded_at: None,
            }],
            traces: Vec::new(),
            semantic_consolidation: None,
            procedures: vec![],
        },
        inbox: memd_schema::MemoryInboxResponse {
            route: memd_schema::RetrievalRoute::Auto,
            intent: memd_schema::RetrievalIntent::CurrentTask,
            items: vec![memd_schema::InboxMemoryItem {
                item: memd_schema::MemoryItem {
                    id: uuid::Uuid::new_v4(),
                    content: "capture the latest turn result".to_string(),
                    redundancy_key: None,
                    belief_branch: None,
                    preferred: true,
                    kind: memd_schema::MemoryKind::Status,
                    scope: memd_schema::MemoryScope::Project,
                    project: Some(project.to_string()),
                    namespace: Some(namespace.to_string()),
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
                reasons: vec!["current-turn".to_string()],
            }],
        },
        workspaces: memd_schema::WorkspaceMemoryResponse {
            workspaces: vec![memd_schema::WorkspaceMemoryRecord {
                project: Some(project.to_string()),
                namespace: Some(namespace.to_string()),
                workspace: Some("team-alpha".to_string()),
                visibility: memd_schema::MemoryVisibility::Workspace,
                item_count: 3,
                active_count: 2,
                candidate_count: 1,
                contested_count: 0,
                source_lane_count: 1,
                avg_confidence: 0.84,
                trust_score: 0.91,
                last_seen_at: None,
                tags: Vec::new(),
            }],
        },
        sources: memd_schema::SourceMemoryResponse {
            sources: Vec::new(),
        },
        semantic: None,
        claims: SessionClaimsState::default(),
        recent_repo_changes: vec!["status M crates/memd-client/src/main.rs".to_string()],
        change_summary: vec!["focus -> follow the codex pack turn boundary".to_string()],
        resume_state_age_minutes: None,
        refresh_recommended: false,
    }
}

mod mock_server_support;
pub(crate) use self::mock_server_support::*;
mod autoresearch_evolution_tests;
mod awareness_hive_tests;
mod benchmark_runtime_tests;
mod bootstrap_harness_tests;
mod gap_coordination_tests;
mod hive_coordination_tests;
mod public_benchmark_tests;
mod runtime_memory_tests;
mod runtime_verification_tests;
mod skill_workflow_tests;
mod tasks_hive_tests;
mod test_support;
use self::autoresearch_evolution_tests::test_autoresearch_snapshot;
pub(crate) use self::test_support::*;
