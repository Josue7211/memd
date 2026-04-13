use super::*;
use crate::canonical_key;
use crate::redundancy_key;
use memd_schema::{
    HiveRosterRequest, MaintainReportRequest, MemoryKind, MemoryScope, MemoryStage, MemoryStatus,
    MemoryVisibility, SourceQuality,
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
    }
}

#[path = "core.rs"]
mod core;
#[path = "hive_board.rs"]
mod hive_board;
#[path = "hive_sessions.rs"]
mod hive_sessions;
#[path = "maintenance.rs"]
mod maintenance;
