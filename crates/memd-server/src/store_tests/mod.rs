use super::*;
use crate::canonical_key;
use crate::redundancy_key;
use memd_schema::{
    HiveRosterRequest, MaintainReportRequest, MemoryKind, MemoryScope, MemoryStage, MemoryStatus,
    MemoryVisibility, SourceQuality, TokenSavingsListRequest, TokenSavingsRecord,
    TokenSavingsSyncRequest,
};

fn open_temp_store(prefix: &str) -> (std::path::PathBuf, SqliteStore) {
    let dir = std::env::temp_dir().join(format!("{prefix}-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");
    (dir, store)
}

fn sample_memory_item() -> MemoryItem {
    let now = chrono::Utc::now();
    MemoryItem {
        id: Uuid::new_v4(),
        content: "hive resume state".to_string(),
        redundancy_key: None,
        belief_branch: None,
        preferred: false,
        kind: MemoryKind::Status,
        scope: MemoryScope::Project,
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        workspace: Some("core".to_string()),
        visibility: MemoryVisibility::Workspace,
        source_agent: Some("codex@test".to_string()),
        source_system: Some("memd-test".to_string()),
        source_path: None,
        confidence: 0.9,
        ttl_seconds: None,
        created_at: now,
        updated_at: now,
        last_verified_at: None,
        supersedes: Vec::new(),
        tags: vec!["resume_state".to_string()],
        status: MemoryStatus::Active,
        source_quality: Some(SourceQuality::Canonical),
        stage: MemoryStage::Canonical,
        lane: None,
        version: 1,
        correction_meta: None,
    }
}

#[test]
fn token_savings_upsert_and_list_summarizes_cross_harness_records() {
    let (dir, store) = open_temp_store("memd-token-savings-store");
    let now = chrono::Utc::now();
    let record = TokenSavingsRecord {
        id: Uuid::new_v4(),
        operation: "context_packet".to_string(),
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        workspace: Some("core".to_string()),
        user_id: None,
        agent: Some("codex".to_string()),
        model_tier: Some("tiny".to_string()),
        intent: Some("CurrentTask".to_string()),
        source_records: 3,
        baseline_input_tokens: 1000,
        output_tokens: 250,
        tokens_saved: 750,
        wasted_tokens: 0,
        waste_kind: None,
        reason: "compiled packet avoided reread".to_string(),
        ts: now,
        updated_at: None,
    };
    let waste_record = TokenSavingsRecord {
        id: Uuid::new_v4(),
        operation: "token_waste_observed".to_string(),
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        workspace: Some("core".to_string()),
        user_id: None,
        agent: Some("codex".to_string()),
        model_tier: None,
        intent: Some("TokenWaste".to_string()),
        source_records: 0,
        baseline_input_tokens: 2000,
        output_tokens: 0,
        tokens_saved: 0,
        wasted_tokens: 2000,
        waste_kind: Some("giant_diff".to_string()),
        reason: "giant diff entered context".to_string(),
        ts: now,
        updated_at: None,
    };

    let sync = store
        .upsert_token_savings(&TokenSavingsSyncRequest {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("core".to_string()),
            user_id: None,
            agent: None,
            records: vec![record, waste_record],
        })
        .expect("upsert token savings");
    let list = store
        .list_token_savings(&TokenSavingsListRequest {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("core".to_string()),
            user_id: None,
            agent: None,
            since: None,
            limit: None,
        })
        .expect("list token savings");

    assert_eq!(sync.upserted, 2);
    assert_eq!(list.total, 2);
    assert_eq!(list.measured_input_tokens, 1000);
    assert_eq!(list.measured_output_tokens, 250);
    assert_eq!(list.measured_tokens_saved, 750);
    assert_eq!(list.wasted_events, 1);
    assert_eq!(list.wasted_tokens, 2000);
    assert_eq!(list.wasted_giant_diff_tokens, 2000);

    std::fs::remove_dir_all(dir).expect("cleanup token savings store");
}

#[path = "core.rs"]
mod core;
#[path = "hive_board.rs"]
mod hive_board;
#[path = "hive_sessions.rs"]
mod hive_sessions;
#[path = "maintenance.rs"]
mod maintenance;
#[path = "multi_user.rs"]
mod multi_user;
