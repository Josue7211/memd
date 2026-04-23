use super::*;
#[test]
fn hive_board_ignores_handoff_scope_receipts_in_overlap_risks() {
    let dir =
        std::env::temp_dir().join(format!("memd-hive-overlap-noise-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

    store
        .upsert_hive_session(&HiveSessionUpsertRequest {
            session: "bee-1".to_string(),
            agent: Some("codex".to_string()),
            effective_agent: Some("codex@bee-1".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("worker".to_string()),
            worker_name: Some("Avicenna".to_string()),
            display_name: None,
            role: Some("worker".to_string()),
            capabilities: vec!["coordination".to_string()],
            hive_groups: vec!["project:memd".to_string()],
            lane_id: None,
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: None,
            tab_id: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            base_branch: None,
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            base_url_healthy: Some(true),
            host: None,
            pid: Some(101),
            topic_claim: Some("Parser lane".to_string()),
            scope_claims: vec!["crates/memd-client/src/main.rs".to_string()],
            task_id: Some("parser-refactor".to_string()),
            focus: None,
            pressure: None,
            next_recovery: None,
            next_action: None,
            working: None,
            touches: Vec::new(),
            blocked_by: Vec::new(),
            cowork_with: Vec::new(),
            handoff_target: None,
            offered_to: Vec::new(),
            needs_help: false,
            needs_review: false,
            handoff_state: None,
            confidence: None,
            risk: None,
            last_wake_at: None,
            status: Some("live".to_string()),
        })
        .expect("insert session");

    store
            .record_hive_coordination_receipt(&HiveCoordinationReceiptRequest {
                kind: "queen_handoff".to_string(),
                actor_session: "queen-1".to_string(),
                actor_agent: Some("queen".to_string()),
                target_session: Some("bee-1".to_string()),
                task_id: Some("parser-refactor".to_string()),
                scope: Some("crates/memd-client/src/main.rs".to_string()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                summary: "Handoff to Avicenna (bee-1) task=parser-refactor scopes=crates/memd-client/src/main.rs".to_string(),
            })
            .expect("record handoff receipt");
    store
        .record_hive_coordination_receipt(&HiveCoordinationReceiptRequest {
            kind: "possible_work_overlap".to_string(),
            actor_session: "queen-1".to_string(),
            actor_agent: Some("queen".to_string()),
            target_session: Some("bee-1".to_string()),
            task_id: Some("parser-refactor".to_string()),
            scope: Some("crates/memd-client/src/main.rs".to_string()),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            summary:
                "possible_work_overlap touches=crates/memd-client/src/main.rs sessions=bee-1,bee-2"
                    .to_string(),
        })
        .expect("record overlap receipt");

    let board = store
        .hive_board(&HiveBoardRequest {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
        })
        .expect("read hive board");

    assert_eq!(board.overlap_risks.len(), 1);
    assert!(board.overlap_risks[0].contains("possible_work_overlap"));

    std::fs::remove_dir_all(dir).expect("cleanup temp dir");
}

#[test]
fn hive_board_hides_low_signal_sender_sessions_without_active_tasks() {
    let dir = std::env::temp_dir().join(format!("memd-hive-sender-noise-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

    store
        .upsert_hive_session(&HiveSessionUpsertRequest {
            session: "sender-noise".to_string(),
            agent: Some("codex".to_string()),
            effective_agent: Some("codex@sender-noise".to_string()),
            hive_system: None,
            hive_role: None,
            worker_name: Some("codex".to_string()),
            display_name: None,
            role: None,
            capabilities: Vec::new(),
            hive_groups: vec!["project:memd".to_string()],
            lane_id: None,
            hive_group_goal: None,
            authority: None,
            heartbeat_model: None,
            tab_id: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            base_branch: None,
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            base_url_healthy: Some(true),
            host: None,
            pid: Some(301),
            topic_claim: Some("focus: task-current-noise".to_string()),
            scope_claims: Vec::new(),
            task_id: None,
            focus: Some("focus: task-current-noise".to_string()),
            pressure: Some("keep continuity tight".to_string()),
            next_recovery: Some("next: resume next step".to_string()),
            next_action: Some("focus: task-current-noise".to_string()),
            working: None,
            touches: Vec::new(),
            blocked_by: Vec::new(),
            cowork_with: Vec::new(),
            handoff_target: None,
            offered_to: Vec::new(),
            needs_help: false,
            needs_review: false,
            handoff_state: None,
            confidence: None,
            risk: None,
            last_wake_at: None,
            status: Some("live".to_string()),
        })
        .expect("insert low signal sender");

    store
        .upsert_hive_session(&HiveSessionUpsertRequest {
            session: "worker-1".to_string(),
            agent: Some("codex".to_string()),
            effective_agent: Some("codex@worker-1".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("worker".to_string()),
            worker_name: Some("Avicenna".to_string()),
            display_name: None,
            role: Some("worker".to_string()),
            capabilities: vec!["coordination".to_string()],
            hive_groups: vec!["project:memd".to_string()],
            lane_id: None,
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: None,
            tab_id: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            base_branch: None,
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            base_url_healthy: Some(true),
            host: None,
            pid: Some(302),
            topic_claim: Some("Parser lane".to_string()),
            scope_claims: vec!["crates/memd-client/src/main.rs".to_string()],
            task_id: Some("parser-refactor".to_string()),
            focus: None,
            pressure: None,
            next_recovery: None,
            next_action: None,
            working: None,
            touches: Vec::new(),
            blocked_by: Vec::new(),
            cowork_with: Vec::new(),
            handoff_target: None,
            offered_to: Vec::new(),
            needs_help: false,
            needs_review: false,
            handoff_state: None,
            confidence: None,
            risk: None,
            last_wake_at: None,
            status: Some("live".to_string()),
        })
        .expect("insert worker");

    let board = store
        .hive_board(&HiveBoardRequest {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
        })
        .expect("read hive board");
    let roster = store
        .hive_roster(&HiveRosterRequest {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
        })
        .expect("read hive roster");

    assert_eq!(board.active_bees.len(), 1);
    assert_eq!(board.active_bees[0].session, "worker-1");
    assert_eq!(roster.bees.len(), 1);
    assert_eq!(roster.bees[0].session, "worker-1");

    std::fs::remove_dir_all(dir).expect("cleanup temp dir");
}

#[test]
fn hive_sessions_mark_proof_bees_stale_on_shorter_window() {
    let dir =
        std::env::temp_dir().join(format!("memd-hive-proof-presence-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

    store
        .upsert_hive_session(&HiveSessionUpsertRequest {
            session: "session-live-proof".to_string(),
            agent: Some("openclaw".to_string()),
            effective_agent: Some("openclaw@session-live-proof".to_string()),
            hive_system: Some("openclaw".to_string()),
            hive_role: Some("agent".to_string()),
            worker_name: Some("Openclaw".to_string()),
            display_name: None,
            role: Some("agent".to_string()),
            capabilities: vec!["coordination".to_string()],
            hive_groups: vec!["project:memd".to_string()],
            lane_id: None,
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: None,
            tab_id: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            base_branch: None,
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            base_url_healthy: Some(true),
            host: None,
            pid: Some(611),
            topic_claim: None,
            scope_claims: Vec::new(),
            task_id: None,
            focus: None,
            pressure: None,
            next_recovery: None,
            next_action: None,
            working: None,
            touches: Vec::new(),
            blocked_by: Vec::new(),
            cowork_with: Vec::new(),
            handoff_target: None,
            offered_to: Vec::new(),
            needs_help: false,
            needs_review: false,
            handoff_state: None,
            confidence: None,
            risk: None,
            last_wake_at: None,
            status: Some("live".to_string()),
        })
        .expect("insert proof bee");

    let mut session = store
        .hive_sessions(&HiveSessionsRequest {
            session: Some("session-live-proof".to_string()),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            hive_system: None,
            hive_role: None,
            host: None,
            hive_group: None,
            active_only: Some(false),
            limit: Some(1),
        })
        .expect("load proof bee")
        .sessions
        .into_iter()
        .next()
        .expect("proof bee exists");
    session.last_seen = chrono::Utc::now() - chrono::TimeDelta::minutes(6);
    let conn = store.connect().expect("connect sqlite");
    conn.execute(
        "UPDATE hive_sessions SET last_seen = ?1, payload_json = ?2 WHERE session = ?3",
        params![
            session.last_seen.to_rfc3339(),
            serde_json::to_string(&session).expect("serialize proof bee"),
            session.session.as_str(),
        ],
    )
    .expect("age proof bee");

    let active = store
        .hive_sessions(&HiveSessionsRequest {
            session: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            hive_system: None,
            hive_role: None,
            host: None,
            hive_group: None,
            active_only: Some(true),
            limit: Some(8),
        })
        .expect("list active proof bees");
    assert!(
        active
            .sessions
            .iter()
            .all(|session| session.session != "session-live-proof")
    );

    let all = store
        .hive_sessions(&HiveSessionsRequest {
            session: Some("session-live-proof".to_string()),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            hive_system: None,
            hive_role: None,
            host: None,
            hive_group: None,
            active_only: Some(false),
            limit: Some(1),
        })
        .expect("load proof bee after aging");
    assert_eq!(all.sessions[0].status, "stale");

    std::fs::remove_dir_all(dir).expect("cleanup temp dir");
}

#[test]
fn hive_board_hides_sender_sessions_with_only_lane_path_and_no_task_signal() {
    let dir = std::env::temp_dir().join(format!(
        "memd-hive-sender-lane-only-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

    store
        .upsert_hive_session(&HiveSessionUpsertRequest {
            session: "sender-lane-only".to_string(),
            agent: Some("codex".to_string()),
            effective_agent: Some("codex@sender-lane-only".to_string()),
            hive_system: None,
            hive_role: None,
            worker_name: Some("codex".to_string()),
            display_name: None,
            role: None,
            capabilities: Vec::new(),
            hive_groups: vec!["project:memd".to_string()],
            lane_id: Some("/tmp/sessions".to_string()),
            hive_group_goal: None,
            authority: None,
            heartbeat_model: None,
            tab_id: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            repo_root: Some("/tmp/sessions".to_string()),
            worktree_root: Some("/tmp/sessions".to_string()),
            branch: Some("feature/hive-shared".to_string()),
            base_branch: None,
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            base_url_healthy: Some(true),
            host: None,
            pid: Some(303),
            topic_claim: Some("focus: task-current-noise".to_string()),
            scope_claims: Vec::new(),
            task_id: None,
            focus: Some("focus: task-current-noise".to_string()),
            pressure: Some("keep continuity tight".to_string()),
            next_recovery: Some("next: resume next step".to_string()),
            next_action: Some("focus: task-current-noise".to_string()),
            working: None,
            touches: Vec::new(),
            blocked_by: Vec::new(),
            cowork_with: Vec::new(),
            handoff_target: None,
            offered_to: Vec::new(),
            needs_help: false,
            needs_review: false,
            handoff_state: None,
            confidence: None,
            risk: None,
            last_wake_at: None,
            status: Some("live".to_string()),
        })
        .expect("insert lane-only sender");

    let board = store
        .hive_board(&HiveBoardRequest {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
        })
        .expect("read hive board");
    let roster = store
        .hive_roster(&HiveRosterRequest {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
        })
        .expect("read hive roster");

    assert!(board.active_bees.is_empty());
    assert!(roster.bees.is_empty());

    std::fs::remove_dir_all(dir).expect("cleanup temp dir");
}

#[test]
fn hive_board_hides_historical_lane_fault_noise_for_inactive_sessions() {
    let dir = std::env::temp_dir().join(format!(
        "memd-hive-lane-fault-noise-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

    store
        .upsert_hive_session(&HiveSessionUpsertRequest {
            session: "worker-1".to_string(),
            agent: Some("codex".to_string()),
            effective_agent: Some("codex@worker-1".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("worker".to_string()),
            worker_name: Some("Avicenna".to_string()),
            display_name: None,
            role: Some("worker".to_string()),
            capabilities: vec!["coordination".to_string()],
            hive_groups: vec!["project:memd".to_string()],
            lane_id: None,
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: None,
            tab_id: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            base_branch: None,
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            base_url_healthy: Some(true),
            host: None,
            pid: Some(401),
            topic_claim: Some("Parser lane".to_string()),
            scope_claims: vec!["crates/memd-client/src/main.rs".to_string()],
            task_id: Some("parser-refactor".to_string()),
            focus: None,
            pressure: None,
            next_recovery: None,
            next_action: None,
            working: None,
            touches: Vec::new(),
            blocked_by: Vec::new(),
            cowork_with: Vec::new(),
            handoff_target: None,
            offered_to: Vec::new(),
            needs_help: false,
            needs_review: false,
            handoff_state: None,
            confidence: None,
            risk: None,
            last_wake_at: None,
            status: Some("live".to_string()),
        })
        .expect("insert worker");

    store
        .record_hive_coordination_receipt(&HiveCoordinationReceiptRequest {
            kind: "lane_fault".to_string(),
            actor_session: "queen-1".to_string(),
            actor_agent: Some("queen".to_string()),
            target_session: Some("old-worker".to_string()),
            task_id: Some("legacy-task".to_string()),
            scope: Some("crates/memd-client/src/main.rs".to_string()),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            summary: "Old lane fault for old-worker".to_string(),
        })
        .expect("record stale lane fault");

    {
        let mut receipt = store
            .hive_coordination_receipts(&HiveCoordinationReceiptsRequest {
                session: None,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                limit: Some(8),
            })
            .expect("load receipts")
            .receipts
            .into_iter()
            .next()
            .expect("stale lane fault receipt");
        receipt.created_at = chrono::Utc::now() - chrono::TimeDelta::minutes(30);
        let payload_json = serde_json::to_string(&receipt).expect("serialize aged receipt");
        let conn = store.connect().expect("connect sqlite store");
        conn.execute(
                "UPDATE hive_coordination_receipts SET created_at = ?1, payload_json = ?2 WHERE id = ?3",
                rusqlite::params![
                    receipt.created_at.to_rfc3339(),
                    payload_json,
                    receipt.id
                ],
            )
            .expect("age receipt row");
    }

    let board = store
        .hive_board(&HiveBoardRequest {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
        })
        .expect("read hive board");

    assert!(board.blocked_bees.is_empty());
    assert!(board.lane_faults.is_empty());
    assert!(board.recommended_actions.is_empty());

    std::fs::remove_dir_all(dir).expect("cleanup temp dir");
}

#[test]
fn hive_board_hides_lane_faults_when_only_actor_session_is_active() {
    let dir = std::env::temp_dir().join(format!(
        "memd-hive-lane-fault-target-filter-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

    store
        .upsert_hive_session(&HiveSessionUpsertRequest {
            session: "worker-1".to_string(),
            agent: Some("codex".to_string()),
            effective_agent: Some("codex@worker-1".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("worker".to_string()),
            worker_name: Some("Avicenna".to_string()),
            display_name: None,
            role: Some("worker".to_string()),
            capabilities: vec!["coordination".to_string()],
            hive_groups: vec!["project:memd".to_string()],
            lane_id: None,
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: None,
            tab_id: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            base_branch: None,
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            base_url_healthy: Some(true),
            host: None,
            pid: Some(501),
            topic_claim: Some("Parser lane".to_string()),
            scope_claims: vec!["crates/memd-client/src/main.rs".to_string()],
            task_id: Some("parser-refactor".to_string()),
            focus: None,
            pressure: None,
            next_recovery: None,
            next_action: None,
            working: None,
            touches: Vec::new(),
            blocked_by: Vec::new(),
            cowork_with: Vec::new(),
            handoff_target: None,
            offered_to: Vec::new(),
            needs_help: false,
            needs_review: false,
            handoff_state: None,
            confidence: None,
            risk: None,
            last_wake_at: None,
            status: Some("live".to_string()),
        })
        .expect("insert active worker");

    store
        .record_hive_coordination_receipt(&HiveCoordinationReceiptRequest {
            kind: "queen_deny".to_string(),
            actor_session: "worker-1".to_string(),
            actor_agent: Some("codex".to_string()),
            target_session: Some("inactive-target".to_string()),
            task_id: Some("parser-refactor".to_string()),
            scope: Some("crates/memd-client/src/main.rs".to_string()),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            summary: "Queen denied inactive target".to_string(),
        })
        .expect("record deny receipt");

    let board = store
        .hive_board(&HiveBoardRequest {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
        })
        .expect("read hive board");

    assert!(board.blocked_bees.is_empty());
    assert!(board.lane_faults.is_empty());
    assert!(board.recommended_actions.is_empty());

    std::fs::remove_dir_all(dir).expect("cleanup temp dir");
}

#[test]
fn open_migrates_legacy_hive_sessions_before_identity_indexes() {
    let dir = std::env::temp_dir().join(format!("legacy-hive-sessions-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join("state.sqlite");
    let conn = Connection::open(&path).expect("open sqlite database");

    conn.execute_batch(
        r#"
            PRAGMA foreign_keys = ON;
            CREATE TABLE hive_sessions (
              session_key TEXT PRIMARY KEY,
              session TEXT NOT NULL,
              project TEXT,
              namespace TEXT,
              workspace TEXT,
              status TEXT NOT NULL,
              last_seen TEXT NOT NULL,
              payload_json TEXT NOT NULL
            );
            "#,
    )
    .expect("create legacy hive_sessions");

    drop(conn);

    let store = SqliteStore::open(&path).expect("open migrated sqlite store");
    let conn = store.connect().expect("connect migrated sqlite store");
    let columns = {
        let mut stmt = conn
            .prepare("PRAGMA table_info(hive_sessions)")
            .expect("prepare table info");
        stmt.query_map([], |row| row.get::<_, String>(1))
            .expect("query table info")
            .collect::<Result<Vec<_>, _>>()
            .expect("collect hive session columns")
    };
    assert!(columns.iter().any(|value| value == "hive_system"));
    assert!(columns.iter().any(|value| value == "hive_role"));
    assert!(columns.iter().any(|value| value == "host"));

    let indexes = {
        let mut stmt = conn
            .prepare("PRAGMA index_list(hive_sessions)")
            .expect("prepare index list");
        stmt.query_map([], |row| row.get::<_, String>(1))
            .expect("query index list")
            .collect::<Result<Vec<_>, _>>()
            .expect("collect hive session indexes")
    };
    assert!(
        indexes
            .iter()
            .any(|value| value == "idx_hive_sessions_hive_system")
    );
    assert!(
        indexes
            .iter()
            .any(|value| value == "idx_hive_sessions_hive_role")
    );
    assert!(
        indexes
            .iter()
            .any(|value| value == "idx_hive_sessions_host")
    );

    std::fs::remove_dir_all(dir).expect("cleanup temp dir");
}

#[test]
fn queen_deny_blocks_subsequent_task_upsert_for_denied_session() {
    let dir = std::env::temp_dir().join(format!("memd-queen-deny-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

    store
        .upsert_hive_task(&HiveTaskUpsertRequest {
            task_id: "task-alpha".to_string(),
            title: "Alpha".to_string(),
            description: None,
            status: Some("active".to_string()),
            coordination_mode: Some(CoordinationMode::Solo),
            session: Some("bee-1".to_string()),
            agent: None,
            effective_agent: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
            claim_scopes: Vec::new(),
            help_requested: None,
            review_requested: None,
        })
        .expect("seed task");

    store
        .record_queen_deny("task-alpha", "bee-1", Some("overlap"))
        .expect("record deny");

    let err = store
        .upsert_hive_task(&HiveTaskUpsertRequest {
            task_id: "task-alpha".to_string(),
            title: "Alpha v2".to_string(),
            description: Some("attempt after deny".to_string()),
            status: Some("active".to_string()),
            coordination_mode: Some(CoordinationMode::Solo),
            session: Some("bee-1".to_string()),
            agent: None,
            effective_agent: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
            claim_scopes: Vec::new(),
            help_requested: None,
            review_requested: None,
        })
        .expect_err("denied upsert must fail");
    assert!(
        format!("{err:#}").contains("queen_denied:"),
        "expected queen_denied marker, got: {err:#}"
    );

    store
        .upsert_hive_task(&HiveTaskUpsertRequest {
            task_id: "task-alpha".to_string(),
            title: "Alpha v2".to_string(),
            description: None,
            status: Some("active".to_string()),
            coordination_mode: Some(CoordinationMode::Solo),
            session: Some("bee-other".to_string()),
            agent: None,
            effective_agent: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
            claim_scopes: Vec::new(),
            help_requested: None,
            review_requested: None,
        })
        .expect("non-denied session upsert should succeed");

    let err = store
        .assign_hive_task(&HiveTaskAssignRequest {
            task_id: "task-alpha".to_string(),
            from_session: None,
            to_session: "bee-1".to_string(),
            to_agent: None,
            to_effective_agent: None,
            note: None,
        })
        .expect_err("denied assign must fail");
    assert!(
        format!("{err:#}").contains("queen_denied:"),
        "expected queen_denied marker on assign, got: {err:#}"
    );

    store
        .clear_queen_deny("task-alpha", "bee-1")
        .expect("clear deny");
    store
        .assign_hive_task(&HiveTaskAssignRequest {
            task_id: "task-alpha".to_string(),
            from_session: None,
            to_session: "bee-1".to_string(),
            to_agent: None,
            to_effective_agent: None,
            note: None,
        })
        .expect("assign after clear should succeed");

    std::fs::remove_dir_all(dir).expect("cleanup temp dir");
}

#[test]
fn queen_reroute_rewrites_session_lane_via_json_set() {
    let dir = std::env::temp_dir().join(format!("memd-queen-reroute-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

    store
        .upsert_hive_session(&HiveSessionUpsertRequest {
            session: "bee-42".to_string(),
            agent: Some("codex".to_string()),
            effective_agent: Some("codex@bee-42".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("worker".to_string()),
            worker_name: Some("Ada".to_string()),
            display_name: None,
            role: Some("worker".to_string()),
            capabilities: Vec::new(),
            hive_groups: Vec::new(),
            lane_id: Some("lane-alpha".to_string()),
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: None,
            tab_id: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            base_branch: None,
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            base_url_healthy: Some(true),
            host: None,
            pid: Some(42),
            topic_claim: None,
            scope_claims: Vec::new(),
            task_id: None,
            focus: None,
            pressure: None,
            next_recovery: None,
            next_action: None,
            working: None,
            touches: Vec::new(),
            blocked_by: Vec::new(),
            cowork_with: Vec::new(),
            handoff_target: None,
            offered_to: Vec::new(),
            needs_help: false,
            needs_review: false,
            handoff_state: None,
            confidence: None,
            risk: None,
            last_wake_at: None,
            status: Some("live".to_string()),
        })
        .expect("seed session");

    let touched = store
        .set_session_lane(
            "bee-42",
            Some("memd"),
            Some("main"),
            Some("shared"),
            Some("lane-beta"),
        )
        .expect("reroute");
    assert_eq!(touched, 1, "expected exactly one session row rewritten");

    let sessions = store
        .hive_sessions(&HiveSessionsRequest {
            session: Some("bee-42".to_string()),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            hive_system: None,
            hive_role: None,
            host: None,
            hive_group: None,
            active_only: Some(false),
            limit: Some(4),
        })
        .expect("read session back");
    assert_eq!(sessions.sessions.len(), 1);
    assert_eq!(
        sessions.sessions[0].lane_id.as_deref(),
        Some("lane-beta"),
        "lane_id must be persisted to payload_json"
    );

    let cleared = store
        .set_session_lane("bee-42", Some("memd"), Some("main"), Some("shared"), None)
        .expect("reroute clear");
    assert_eq!(cleared, 1);
    let sessions = store
        .hive_sessions(&HiveSessionsRequest {
            session: Some("bee-42".to_string()),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            hive_system: None,
            hive_role: None,
            host: None,
            hive_group: None,
            active_only: Some(false),
            limit: Some(4),
        })
        .expect("read session after clear");
    assert!(
        sessions.sessions[0].lane_id.is_none(),
        "clearing lane should null lane_id in payload"
    );

    std::fs::remove_dir_all(dir).expect("cleanup temp dir");
}

#[test]
fn queen_handoff_lock_bumps_version_and_blocks_other_sessions() {
    let dir = std::env::temp_dir().join(format!("memd-queen-handoff-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

    // First handoff — expect version 1 and only the locked session may mutate.
    let v1 = store
        .apply_handoff_lock("task-omega", "bee-2", Some("queen-1"))
        .expect("first handoff lock");
    assert_eq!(v1, 1, "first handoff must stamp version 1");

    // Locked session can still upsert.
    store
        .upsert_hive_task(&HiveTaskUpsertRequest {
            task_id: "task-omega".to_string(),
            title: "Omega".to_string(),
            description: None,
            status: Some("active".to_string()),
            coordination_mode: Some(CoordinationMode::Solo),
            session: Some("bee-2".to_string()),
            agent: None,
            effective_agent: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
            claim_scopes: Vec::new(),
            help_requested: None,
            review_requested: None,
        })
        .expect("locked session upsert");

    // Someone else tries → 409-class error.
    let err = store
        .upsert_hive_task(&HiveTaskUpsertRequest {
            task_id: "task-omega".to_string(),
            title: "Omega (other)".to_string(),
            description: None,
            status: Some("active".to_string()),
            coordination_mode: Some(CoordinationMode::Solo),
            session: Some("bee-3".to_string()),
            agent: None,
            effective_agent: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
            claim_scopes: Vec::new(),
            help_requested: None,
            review_requested: None,
        })
        .expect_err("unrelated session must be locked out");
    assert!(
        format!("{err:#}").contains("queen_handoff_locked:"),
        "expected queen_handoff_locked marker, got: {err:#}"
    );

    // Assign to unrelated session → also blocked.
    let err = store
        .assign_hive_task(&HiveTaskAssignRequest {
            task_id: "task-omega".to_string(),
            from_session: None,
            to_session: "bee-3".to_string(),
            to_agent: None,
            to_effective_agent: None,
            note: None,
        })
        .expect_err("assign to unrelated session must be locked out");
    assert!(format!("{err:#}").contains("queen_handoff_locked:"));

    // Second handoff (to a new session) — version monotonically bumps.
    let v2 = store
        .apply_handoff_lock("task-omega", "bee-3", Some("queen-1"))
        .expect("second handoff lock");
    assert_eq!(v2, 2, "second handoff must stamp version 2");

    // Previously denied session (bee-3) now owns the lock — handoff clears
    // any prior deny on that (task, session) pair.
    store
        .record_queen_deny("task-omega", "bee-3", Some("stale"))
        .expect("record deny pre-handoff");
    let v3 = store
        .apply_handoff_lock("task-omega", "bee-3", Some("queen-1"))
        .expect("handoff subsumes deny");
    assert_eq!(v3, 3);
    assert!(
        !store
            .is_task_denied_for_session("task-omega", "bee-3")
            .expect("check deny"),
        "handoff must subsume stale deny on the same (task, session)"
    );

    // Explicit clear unlocks.
    store.clear_handoff_lock("task-omega").expect("clear lock");
    assert!(
        store
            .handoff_lock_for_task("task-omega")
            .expect("read lock")
            .is_none()
    );
    store
        .upsert_hive_task(&HiveTaskUpsertRequest {
            task_id: "task-omega".to_string(),
            title: "Omega (unlocked)".to_string(),
            description: None,
            status: Some("active".to_string()),
            coordination_mode: Some(CoordinationMode::Solo),
            session: Some("bee-4".to_string()),
            agent: None,
            effective_agent: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
            claim_scopes: Vec::new(),
            help_requested: None,
            review_requested: None,
        })
        .expect("upsert succeeds once lock cleared");

    std::fs::remove_dir_all(dir).expect("cleanup temp dir");
}

#[test]
fn hive_divergence_caps_branches_and_decisions_and_dedups_by_normalized_text() {
    use memd_schema::DivergenceRequest;

    let dir = std::env::temp_dir().join(format!("memd-hive-divergence-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

    let base = chrono::Utc::now();
    // (branch, content, minutes_ago) — 3 branches so we trigger truncated_branches.
    let seeds: &[(&str, &str, i64)] = &[
        // branch main: 4 decisions, 2 are normalization-duplicates (different casing/whitespace).
        ("main", "Adopt FTS5 for search", 60),
        ("main", "adopt   fts5   for search", 55), // dedups with above
        ("main", "Cache embeddings in-memory", 50),
        ("main", "Use Lamport clocks for imports", 45),
        ("main", "Shard by project", 40), // pushes truncated_decisions after cap=3
        // branch alt-a: 2 decisions
        ("alt-a", "Prefer DuckDB over SQLite", 30),
        ("alt-a", "Compact with vacuum weekly", 25),
        // branch alt-b: 1 decision (oldest — should be evicted by MAX_BRANCHES=2)
        ("alt-b", "Write handoff to separate DB", 120),
    ];

    for (branch, content, minutes_ago) in seeds {
        let mut item = sample_memory_item();
        item.id = uuid::Uuid::new_v4();
        item.kind = memd_schema::MemoryKind::Decision;
        item.belief_branch = Some((*branch).to_string());
        item.content = (*content).to_string();
        let ts = base - chrono::Duration::minutes(*minutes_ago);
        item.created_at = ts;
        item.updated_at = ts;
        let ckey = canonical_key(&item);
        let rkey = redundancy_key(&item);
        store
            .insert_or_get_duplicate(&item, &ckey, &rkey)
            .expect("seed decision row");
    }

    let summary = store
        .hive_divergence(&DivergenceRequest {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
        })
        .expect("compute divergence");

    assert!(
        summary.truncated_branches,
        "3 branches → truncated flag set"
    );
    assert_eq!(summary.branches.len(), 2, "cap MAX_BRANCHES=2");

    // Most-recent-first: main (last update 40min ago) then alt-a (25min ago wins over 30min).
    // alt-a has its newest at 25min; main has newest at 40min. alt-a should come first.
    assert_eq!(summary.branches[0].branch_name, "alt-a");
    assert_eq!(summary.branches[1].branch_name, "main");

    let main_branch = &summary.branches[1];
    assert!(
        main_branch.truncated_decisions,
        "main had 4 unique decisions after dedup; cap=3"
    );
    assert_eq!(main_branch.decisions.len(), 3);
    // First dup ("adopt fts5 for search") must not appear twice.
    let normalized_set: std::collections::HashSet<&str> = main_branch
        .decisions
        .iter()
        .map(|d| d.normalized.as_str())
        .collect();
    assert_eq!(
        normalized_set.len(),
        main_branch.decisions.len(),
        "decisions must be unique after normalization"
    );

    let alt_a = &summary.branches[0];
    assert!(!alt_a.truncated_decisions);
    assert_eq!(alt_a.decisions.len(), 2);

    std::fs::remove_dir_all(dir).expect("cleanup temp dir");
}
