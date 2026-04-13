use super::*;
use axum::{
    body::{Body, to_bytes},
    http::Request,
};
use memd_schema::MemoryRepairMode;
use tower::util::ServiceExt;

fn sample_memory_item(workspace: Option<&str>) -> MemoryItem {
    let now = Utc::now();
    MemoryItem {
        id: uuid::Uuid::new_v4(),
        content: "workspace-ranked memory".to_string(),
        redundancy_key: None,
        belief_branch: None,
        preferred: false,
        kind: MemoryKind::Fact,
        scope: MemoryScope::Project,
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        workspace: workspace.map(|value| value.to_string()),
        visibility: MemoryVisibility::Workspace,
        source_agent: Some("codex".to_string()),
        source_system: Some("cli".to_string()),
        source_path: None,
        source_quality: Some(SourceQuality::Canonical),
        confidence: 0.9,
        ttl_seconds: None,
        created_at: now,
        updated_at: now,
        last_verified_at: Some(now),
        supersedes: Vec::new(),
        tags: vec!["workspace".to_string()],
        status: MemoryStatus::Active,
        stage: MemoryStage::Canonical,
    }
}

fn temp_state(name: &str) -> (std::path::PathBuf, AppState) {
    let dir = std::env::temp_dir().join(format!("{name}-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("create temp state dir");
    let db_path = dir.join("memd.db");
    let state = AppState {
        store: SqliteStore::open(&db_path).expect("open temp store"),
    };
    (dir, state)
}

fn test_hive_router(state: AppState) -> Router {
    Router::new()
        .route("/hive/board", get(get_hive_board))
        .route("/hive/roster", get(get_hive_roster))
        .route("/hive/follow", get(get_hive_follow))
        .with_state(state)
}

fn seed_hive_route_state(state: &AppState) {
    state
        .store
        .upsert_hive_session(&HiveSessionUpsertRequest {
            session: "queen-1".to_string(),
            agent: Some("codex".to_string()),
            effective_agent: Some("codex@queen-1".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("orchestrator".to_string()),
            worker_name: Some("Avicenna".to_string()),
            display_name: None,
            role: Some("queen".to_string()),
            capabilities: vec!["coordination".to_string()],
            hive_groups: vec!["project:memd".to_string()],
            lane_id: Some("queen-lane".to_string()),
            hive_group_goal: None,
            authority: Some("coordinator".to_string()),
            heartbeat_model: Some("gpt-5.4".to_string()),
            tab_id: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: Some("main".to_string()),
            base_branch: None,
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            base_url_healthy: Some(true),
            host: Some("workstation".to_string()),
            pid: Some(100),
            topic_claim: Some("Route hive work".to_string()),
            scope_claims: vec!["docs/hive.md".to_string()],
            task_id: Some("queen-routing".to_string()),
            focus: Some("Coordinate bee lanes".to_string()),
            pressure: None,
            next_recovery: None,
            next_action: Some("Review overlap alerts".to_string()),
            working: None,
            touches: Vec::new(),
            blocked_by: Vec::new(),
            cowork_with: Vec::new(),
            handoff_target: None,
            offered_to: Vec::new(),
            needs_help: false,
            needs_review: false,
            handoff_state: None,
            confidence: Some("0.95".to_string()),
            risk: None,
            status: Some("active".to_string()),
        })
        .expect("insert queen session");
    state
        .store
        .upsert_hive_session(&HiveSessionUpsertRequest {
            session: "bee-1".to_string(),
            agent: Some("codex".to_string()),
            effective_agent: Some("codex@bee-1".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("agent".to_string()),
            worker_name: Some("Lorentz".to_string()),
            display_name: None,
            role: Some("worker".to_string()),
            capabilities: vec!["coding".to_string()],
            hive_groups: vec!["project:memd".to_string()],
            lane_id: Some("parser-lane".to_string()),
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: Some("gpt-5.4".to_string()),
            tab_id: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: Some("/tmp/bee-1".to_string()),
            branch: Some("feature/parser".to_string()),
            base_branch: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            base_url_healthy: Some(true),
            host: Some("workstation".to_string()),
            pid: Some(101),
            topic_claim: Some("Parser lane refactor".to_string()),
            scope_claims: vec![
                "project".to_string(),
                "crates/memd-client/src/main.rs".to_string(),
                "task:parser-refactor".to_string(),
            ],
            task_id: Some("parser-refactor".to_string()),
            focus: Some("Refine parser overlap flow".to_string()),
            pressure: None,
            next_recovery: None,
            next_action: Some("Request review".to_string()),
            working: None,
            touches: Vec::new(),
            blocked_by: Vec::new(),
            cowork_with: Vec::new(),
            handoff_target: None,
            offered_to: Vec::new(),
            needs_help: false,
            needs_review: true,
            handoff_state: None,
            confidence: Some("0.9".to_string()),
            risk: None,
            status: Some("active".to_string()),
        })
        .expect("insert bee session");
    state
        .store
        .upsert_hive_task(&HiveTaskUpsertRequest {
            task_id: "parser-refactor".to_string(),
            title: "Refine parser overlap flow".to_string(),
            description: Some("narrow parser work".to_string()),
            status: Some("active".to_string()),
            coordination_mode: Some("exclusive_write".to_string()),
            session: Some("bee-1".to_string()),
            agent: Some("codex".to_string()),
            effective_agent: Some("codex@bee-1".to_string()),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            claim_scopes: vec!["crates/memd-client/src/main.rs".to_string()],
            help_requested: Some(false),
            review_requested: Some(true),
        })
        .expect("insert hive task");
    state
        .store
        .send_hive_message(&HiveMessageSendRequest {
            kind: "note".to_string(),
            from_session: "queen-1".to_string(),
            from_agent: Some("codex".to_string()),
            to_session: "bee-1".to_string(),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            content: "Stay on parser lane only.".to_string(),
        })
        .expect("insert hive message");
    state
        .store
        .record_hive_coordination_receipt(&HiveCoordinationReceiptRequest {
            kind: "queen_assign".to_string(),
            actor_session: "queen-1".to_string(),
            actor_agent: Some("codex".to_string()),
            target_session: Some("bee-1".to_string()),
            task_id: Some("parser-refactor".to_string()),
            scope: Some("crates/memd-client/src/main.rs".to_string()),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            summary: "Assigned parser lane".to_string(),
        })
        .expect("insert coordination receipt");
}

async fn decode_json<T: serde::de::DeserializeOwned>(response: axum::response::Response) -> T {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read response body");
    serde_json::from_slice(&body).expect("decode response json")
}

#[tokio::test]
async fn hive_board_route_returns_active_bees_and_review_queue() {
    let (dir, state) = temp_state("memd-hive-board-route");
    seed_hive_route_state(&state);
    let app = test_hive_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/hive/board?project=memd&namespace=main&workspace=shared")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("run hive board route");

    assert_eq!(response.status(), StatusCode::OK);
    let body: HiveBoardResponse = decode_json(response).await;
    assert_eq!(body.queen_session.as_deref(), Some("queen-1"));
    assert!(body.active_bees.iter().any(|bee| bee.session == "bee-1"));
    assert!(
        body.review_queue
            .iter()
            .any(|item| item.contains("parser-refactor"))
    );

    std::fs::remove_dir_all(dir).expect("cleanup temp dir");
}

#[tokio::test]
async fn hive_roster_route_returns_named_bees_and_queen() {
    let (dir, state) = temp_state("memd-hive-roster-route");
    seed_hive_route_state(&state);
    let app = test_hive_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/hive/roster?project=memd&namespace=main&workspace=shared")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("run hive roster route");

    assert_eq!(response.status(), StatusCode::OK);
    let body: HiveRosterResponse = decode_json(response).await;
    assert_eq!(body.queen_session.as_deref(), Some("queen-1"));
    assert!(
        body.bees
            .iter()
            .any(|bee| bee.worker_name.as_deref() == Some("Lorentz"))
    );

    std::fs::remove_dir_all(dir).expect("cleanup temp dir");
}

#[tokio::test]
async fn hive_follow_route_returns_messages_receipts_and_confirmed_overlap() {
    let (dir, state) = temp_state("memd-hive-follow-route");
    seed_hive_route_state(&state);
    state
        .store
        .upsert_hive_session(&HiveSessionUpsertRequest {
            session: "bee-2".to_string(),
            agent: Some("codex".to_string()),
            effective_agent: Some("codex@bee-2".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("agent".to_string()),
            worker_name: Some("Noether".to_string()),
            display_name: None,
            role: Some("worker".to_string()),
            capabilities: vec!["coding".to_string()],
            hive_groups: vec!["project:memd".to_string()],
            lane_id: Some("render-lane".to_string()),
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: Some("gpt-5.4".to_string()),
            tab_id: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: Some("/tmp/bee-2".to_string()),
            branch: Some("feature/render".to_string()),
            base_branch: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            base_url_healthy: Some(true),
            host: Some("workstation".to_string()),
            pid: Some(102),
            topic_claim: Some("Render lane polish".to_string()),
            scope_claims: vec![
                "project".to_string(),
                "crates/memd-client/src/main.rs".to_string(),
                "task:render-refresh".to_string(),
            ],
            task_id: Some("render-refresh".to_string()),
            focus: Some("Render lane polish".to_string()),
            pressure: None,
            next_recovery: None,
            next_action: Some("Wait for parser ack".to_string()),
            working: None,
            touches: Vec::new(),
            blocked_by: Vec::new(),
            cowork_with: Vec::new(),
            handoff_target: None,
            offered_to: Vec::new(),
            needs_help: false,
            needs_review: false,
            handoff_state: None,
            confidence: Some("0.82".to_string()),
            risk: None,
            status: Some("active".to_string()),
        })
        .expect("insert second bee");
    let app = test_hive_router(state);

    let response = app
            .oneshot(
                Request::builder()
                    .uri("/hive/follow?session=bee-1&current_session=bee-2&project=memd&namespace=main&workspace=shared")
                    .body(Body::empty())
                    .expect("build request"),
            )
            .await
            .expect("run hive follow route");

    assert_eq!(response.status(), StatusCode::OK);
    let body: HiveFollowResponse = decode_json(response).await;
    assert_eq!(body.target.session, "bee-1");
    assert_eq!(body.messages.len(), 1);
    assert_eq!(body.recent_receipts.len(), 1);
    assert_eq!(
        body.overlap_risk.as_deref(),
        Some(
            "confirmed hive overlap: target session bee-1 already owns scope(s) for task parser-refactor"
        )
    );
    assert_eq!(body.recommended_action, "coordinate_now");

    std::fs::remove_dir_all(dir).expect("cleanup temp dir");
}

#[tokio::test]
async fn hive_follow_route_rejects_empty_session() {
    let (dir, state) = temp_state("memd-hive-follow-bad-request");
    let app = test_hive_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/hive/follow?session=")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("run hive follow route");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    std::fs::remove_dir_all(dir).expect("cleanup temp dir");
}

#[tokio::test]
async fn hive_board_route_auto_retires_stale_sessions() {
    let (dir, state) = temp_state("memd-hive-board-auto-retire");
    state
        .store
        .upsert_hive_session(&HiveSessionUpsertRequest {
            session: "stale-bee".to_string(),
            agent: Some("codex".to_string()),
            effective_agent: Some("codex@stale-bee".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("agent".to_string()),
            worker_name: Some("StaleBee".to_string()),
            display_name: None,
            role: Some("worker".to_string()),
            capabilities: vec!["coding".to_string()],
            hive_groups: vec!["project:memd".to_string()],
            lane_id: Some("old-lane".to_string()),
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: Some("gpt-5.4".to_string()),
            tab_id: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: Some("/tmp/stale-bee".to_string()),
            branch: Some("feature/old".to_string()),
            base_branch: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            base_url_healthy: Some(true),
            host: Some("workstation".to_string()),
            pid: Some(303),
            topic_claim: Some("Old work".to_string()),
            scope_claims: vec!["crates/memd-client/src/old.rs".to_string()],
            task_id: Some("old-task".to_string()),
            focus: Some("stale work".to_string()),
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
            confidence: Some("0.6".to_string()),
            risk: None,
            status: Some("active".to_string()),
        })
        .expect("insert stale bee");

    let mut session = state
        .store
        .hive_sessions(&HiveSessionsRequest {
            session: Some("stale-bee".to_string()),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            workspace: Some("shared".to_string()),
            hive_system: None,
            hive_role: None,
            host: None,
            hive_group: None,
            active_only: Some(false),
            limit: Some(8),
        })
        .expect("load stale bee")
        .sessions
        .into_iter()
        .next()
        .expect("stale bee exists");
    session.last_seen = chrono::Utc::now() - chrono::TimeDelta::minutes(45);
    let conn = rusqlite::Connection::open(dir.join("memd.db")).expect("connect sqlite");
    conn.execute(
        "UPDATE hive_sessions SET last_seen = ?1, payload_json = ?2 WHERE session = ?3",
        rusqlite::params![
            session.last_seen.to_rfc3339(),
            serde_json::to_string(&session).expect("serialize stale session"),
            session.session.as_str(),
        ],
    )
    .expect("mark hive session stale");

    let app = test_hive_router(state.clone());
    let response = app
        .oneshot(
            Request::builder()
                .uri("/hive/board?project=memd&namespace=main&workspace=shared")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("run hive board route");

    assert_eq!(response.status(), StatusCode::OK);
    let body: HiveBoardResponse = decode_json(response).await;
    assert!(!body.stale_bees.iter().any(|session| session == "stale-bee"));

    let remaining = state
        .store
        .hive_sessions(&HiveSessionsRequest {
            session: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            workspace: Some("shared".to_string()),
            hive_system: None,
            hive_role: None,
            host: None,
            hive_group: None,
            active_only: Some(false),
            limit: Some(16),
        })
        .expect("list sessions after hive board");
    assert!(
        remaining
            .sessions
            .iter()
            .all(|session| session.session != "stale-bee")
    );

    std::fs::remove_dir_all(dir).expect("cleanup temp dir");
}

#[tokio::test]
async fn dashboard_route_surfaces_hive_controls_and_coordination_endpoints() {
    let (dir, state) = temp_state("memd-dashboard-hive-controls");
    crate::ui::test_insert_visible_item(&state, "runtime spine", true)
        .expect("seed visible memory");
    seed_hive_route_state(&state);
    let app = Router::new()
        .route("/", get(dashboard))
        .route(
            "/coordination/sessions/retire",
            post(post_hive_session_retire),
        )
        .route(
            "/coordination/sessions/auto-retire",
            post(post_hive_session_auto_retire),
        )
        .route("/hive/board", get(get_hive_board))
        .route("/hive/roster", get(get_hive_roster))
        .route("/hive/follow", get(get_hive_follow))
        .route("/hive/queen/deny", post(post_hive_queen_deny))
        .route("/hive/queen/reroute", post(post_hive_queen_reroute))
        .route("/hive/queen/handoff", post(post_hive_queen_handoff))
        .with_state(state.clone());

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/")
                .body(Body::empty())
                .expect("build dashboard request"),
        )
        .await
        .expect("run dashboard route");
    assert_eq!(response.status(), StatusCode::OK);
    let html = String::from_utf8(
        to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read dashboard body")
            .to_vec(),
    )
    .expect("decode dashboard html");
    assert!(html.contains("data-hive-queen-action=\"deny-focused\""));
    assert!(html.contains("data-hive-queen-action=\"reroute-focused\""));
    assert!(html.contains("data-hive-queen-action=\"handoff-focused\""));
    assert!(html.contains("/hive/queen/deny"));
    assert!(html.contains("/hive/queen/reroute"));
    assert!(html.contains("/hive/queen/handoff"));
    assert!(html.contains("queen auto-retire"));
    assert!(html.contains("<strong>action</strong>"));
    assert!(html.contains("<strong>latest message</strong>"));
    assert!(html.contains("<strong>latest receipt</strong>"));
    assert!(html.contains("window.setInterval(refreshHiveBoardIfVisible, hiveRefreshIntervalMs)"));
    assert!(html.contains("const hiveRefreshIntervalMs = 5000;"));

    let receipt_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/hive/queen/deny")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "queen_session": "queen-1",
                        "target_session": "bee-1",
                        "project": "memd",
                        "namespace": "main",
                        "workspace": "shared"
                    })
                    .to_string(),
                ))
                .expect("build receipt request"),
        )
        .await
        .expect("record receipt");
    assert_eq!(receipt_response.status(), StatusCode::OK);

    let message_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/hive/queen/handoff")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "queen_session": "queen-1",
                        "target_session": "bee-1",
                        "project": "memd",
                        "namespace": "main",
                        "workspace": "shared",
                        "scope": "crates/memd-client/src/main.rs"
                    })
                    .to_string(),
                ))
                .expect("build message request"),
        )
        .await
        .expect("record message");
    assert_eq!(message_response.status(), StatusCode::OK);

    let receipts = state
        .store
        .hive_coordination_receipts(&HiveCoordinationReceiptsRequest {
            session: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            limit: Some(16),
        })
        .expect("list receipts");
    assert!(
        receipts
            .receipts
            .iter()
            .any(|receipt| receipt.kind == "queen_deny")
    );
    let inbox = state
        .store
        .hive_inbox(&HiveMessageInboxRequest {
            session: "bee-1".to_string(),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            include_acknowledged: Some(true),
            limit: Some(8),
        })
        .expect("load hive inbox");
    assert!(
        inbox
            .messages
            .iter()
            .any(|message| message.kind == "handoff")
    );

    std::fs::remove_dir_all(dir).expect("cleanup temp dir");
}

#[test]
fn matching_workspace_ranks_above_other_shared_workspace() {
    let req = ContextRequest {
        project: Some("memd".to_string()),
        agent: Some("codex".to_string()),
        workspace: Some("team-alpha".to_string()),
        visibility: Some(MemoryVisibility::Workspace),
        route: Some(RetrievalRoute::ProjectFirst),
        intent: Some(RetrievalIntent::General),
        limit: Some(8),
        max_chars_per_item: Some(220),
    };
    let plan = RetrievalPlan::resolve(req.route, req.intent);
    let matching = sample_memory_item(Some("team-alpha"));
    let unrelated = sample_memory_item(Some("team-beta"));

    assert!(
        context_score(&matching, None, 0.9, &req, &plan)
            > context_score(&unrelated, None, 0.9, &req, &plan)
    );
}

#[test]
fn verified_canonical_memory_ranks_above_unverified_synthetic_memory() {
    let req = ContextRequest {
        project: Some("memd".to_string()),
        agent: Some("codex".to_string()),
        workspace: Some("team-alpha".to_string()),
        visibility: Some(MemoryVisibility::Workspace),
        route: Some(RetrievalRoute::ProjectFirst),
        intent: Some(RetrievalIntent::CurrentTask),
        limit: Some(8),
        max_chars_per_item: Some(220),
    };
    let plan = RetrievalPlan::resolve(req.route, req.intent);
    let mut verified = sample_memory_item(Some("team-alpha"));
    verified.source_quality = Some(SourceQuality::Canonical);
    verified.last_verified_at = Some(Utc::now());
    verified.confidence = 0.88;

    let mut inferred = sample_memory_item(Some("team-alpha"));
    inferred.source_quality = Some(SourceQuality::Synthetic);
    inferred.last_verified_at = None;
    inferred.confidence = 0.88;

    assert!(
        context_score(&verified, None, 0.7, &req, &plan)
            > context_score(&inferred, None, 0.7, &req, &plan)
    );
    assert!(
        search_score(
            &verified,
            None,
            0.7,
            &Some("workspace".to_string()),
            req.project.as_ref(),
            None,
            &plan,
        ) > search_score(
            &inferred,
            None,
            0.7,
            &Some("workspace".to_string()),
            req.project.as_ref(),
            None,
            &plan,
        )
    );
}

#[test]
fn live_truth_precedes_project_memory() {
    let db_path = std::env::temp_dir().join(format!("memd-live-truth-{}.db", uuid::Uuid::new_v4()));
    let state = AppState {
        store: SqliteStore::open(&db_path).expect("open temp db"),
    };

    let _ = state
        .store_item(
            StoreMemoryRequest {
                content: "recent repo change: update live truth".to_string(),
                kind: MemoryKind::LiveTruth,
                scope: MemoryScope::Local,
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("team-alpha".to_string()),
                visibility: Some(MemoryVisibility::Workspace),
                belief_branch: None,
                source_agent: Some("memd".to_string()),
                source_system: Some("memd-live-truth".to_string()),
                source_path: Some("/tmp/demo".to_string()),
                source_quality: Some(SourceQuality::Derived),
                confidence: Some(0.98),
                ttl_seconds: Some(3_600),
                last_verified_at: Some(Utc::now()),
                supersedes: Vec::new(),
                tags: vec!["live_truth".to_string()],
                status: Some(MemoryStatus::Active),
            },
            MemoryStage::Canonical,
        )
        .expect("store live truth");

    let _ = state
        .store_item(
            StoreMemoryRequest {
                content: "older project fact".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("team-alpha".to_string()),
                visibility: Some(MemoryVisibility::Workspace),
                belief_branch: None,
                source_agent: Some("codex".to_string()),
                source_system: Some("cli".to_string()),
                source_path: Some("notes.md".to_string()),
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.9),
                ttl_seconds: None,
                last_verified_at: Some(Utc::now()),
                supersedes: Vec::new(),
                tags: vec!["fact".to_string()],
                status: Some(MemoryStatus::Active),
            },
            MemoryStage::Canonical,
        )
        .expect("store fact");

    let BuildContextResult { items, .. } = build_context(
        &state,
        &ContextRequest {
            project: Some("demo".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some(MemoryVisibility::Workspace),
            route: Some(RetrievalRoute::ProjectFirst),
            intent: Some(RetrievalIntent::CurrentTask),
            limit: Some(4),
            max_chars_per_item: Some(220),
        },
    )
    .expect("build context");

    assert_eq!(
        items.first().map(|item| item.kind),
        Some(MemoryKind::LiveTruth)
    );
    assert!(
        items
            .iter()
            .any(|item| item.content.contains("older project fact"))
    );
}

#[test]
fn current_task_context_keeps_project_fact_visible_under_synced_noise() {
    let db_path = std::env::temp_dir().join(format!(
        "memd-current-task-project-fact-{}.db",
        uuid::Uuid::new_v4()
    ));
    let state = AppState {
        store: SqliteStore::open(&db_path).expect("open temp db"),
    };

    let _ = state
        .store_item(
            StoreMemoryRequest {
                content: "remembered project fact: memd must preserve important user corrections"
                    .to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("team-alpha".to_string()),
                visibility: Some(MemoryVisibility::Workspace),
                belief_branch: None,
                source_agent: Some("codex".to_string()),
                source_system: Some("memd".to_string()),
                source_path: Some("notes.md".to_string()),
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.98),
                ttl_seconds: None,
                last_verified_at: Some(Utc::now()),
                supersedes: Vec::new(),
                tags: vec!["project_fact".to_string()],
                status: Some(MemoryStatus::Active),
            },
            MemoryStage::Canonical,
        )
        .expect("store durable project fact");

    for index in 0..5 {
        let _ = state
            .store_item(
                StoreMemoryRequest {
                    content: format!("resume state noise {index}: synced session snapshot"),
                    kind: MemoryKind::Status,
                    scope: MemoryScope::Synced,
                    project: Some("demo".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("team-alpha".to_string()),
                    visibility: Some(MemoryVisibility::Workspace),
                    belief_branch: None,
                    source_agent: Some(format!("codex@session-{index}")),
                    source_system: Some("memd-resume-state".to_string()),
                    source_path: None,
                    source_quality: Some(SourceQuality::Derived),
                    confidence: Some(0.94),
                    ttl_seconds: Some(86_400),
                    last_verified_at: Some(Utc::now()),
                    supersedes: Vec::new(),
                    tags: vec!["resume_state".to_string(), "session_state".to_string()],
                    status: Some(MemoryStatus::Active),
                },
                MemoryStage::Canonical,
            )
            .expect("store synced session noise");
    }

    let BuildContextResult { items, .. } = build_context(
        &state,
        &ContextRequest {
            project: Some("demo".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some(MemoryVisibility::Workspace),
            route: Some(RetrievalRoute::LocalFirst),
            intent: Some(RetrievalIntent::CurrentTask),
            limit: Some(4),
            max_chars_per_item: Some(220),
        },
    )
    .expect("build current-task context");

    assert!(
        items
            .iter()
            .any(|item| item.content.contains("remembered project fact")),
        "durable project fact should survive current-task retrieval even when synced session-state noise exists"
    );
}

#[test]
fn current_task_context_prefers_matching_workspace_memory_under_cross_workspace_noise() {
    let db_path = std::env::temp_dir().join(format!(
        "memd-current-task-workspace-fact-{}.db",
        uuid::Uuid::new_v4()
    ));
    let state = AppState {
        store: SqliteStore::open(&db_path).expect("open temp db"),
    };

    let _ = state
        .store_item(
            StoreMemoryRequest {
                content: "shared workspace handoff: team-alpha owns the memory audit".to_string(),
                kind: MemoryKind::Status,
                scope: MemoryScope::Synced,
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("team-alpha".to_string()),
                visibility: Some(MemoryVisibility::Workspace),
                belief_branch: None,
                source_agent: Some("codex@shared-a".to_string()),
                source_system: Some("handoff".to_string()),
                source_path: Some("handoff.md".to_string()),
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.96),
                ttl_seconds: Some(86_400),
                last_verified_at: Some(Utc::now()),
                supersedes: Vec::new(),
                tags: vec!["handoff".to_string(), "workspace".to_string()],
                status: Some(MemoryStatus::Active),
            },
            MemoryStage::Canonical,
        )
        .expect("store matching workspace memory");

    for index in 0..5 {
        let _ = state
            .store_item(
                StoreMemoryRequest {
                    content: format!("team-beta session noise {index}: unrelated workspace state"),
                    kind: MemoryKind::Status,
                    scope: MemoryScope::Synced,
                    project: Some("demo".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("team-beta".to_string()),
                    visibility: Some(MemoryVisibility::Workspace),
                    belief_branch: None,
                    source_agent: Some(format!("codex@team-beta-{index}")),
                    source_system: Some("memd-resume-state".to_string()),
                    source_path: None,
                    source_quality: Some(SourceQuality::Derived),
                    confidence: Some(0.94),
                    ttl_seconds: Some(86_400),
                    last_verified_at: Some(Utc::now()),
                    supersedes: Vec::new(),
                    tags: vec!["resume_state".to_string(), "session_state".to_string()],
                    status: Some(MemoryStatus::Active),
                },
                MemoryStage::Canonical,
            )
            .expect("store unrelated workspace noise");
    }

    let BuildContextResult { items, .. } = build_context(
        &state,
        &ContextRequest {
            project: Some("demo".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some(MemoryVisibility::Workspace),
            route: Some(RetrievalRoute::LocalFirst),
            intent: Some(RetrievalIntent::CurrentTask),
            limit: Some(4),
            max_chars_per_item: Some(220),
        },
    )
    .expect("build current-task context");

    assert!(
        items
            .iter()
            .any(|item| item.content.contains("team-alpha owns the memory audit")),
        "matching workspace memory should survive current-task retrieval even when unrelated synced workspace noise exists"
    );
    assert!(
        items
            .iter()
            .any(|item| item.workspace.as_deref() == Some("team-alpha")),
        "matching workspace should remain represented in the retrieved set"
    );
}

#[test]
fn superseded_memory_drops_out_after_manual_correction_loop() {
    let db_path =
        std::env::temp_dir().join(format!("memd-correction-loop-{}.db", uuid::Uuid::new_v4()));
    let state = AppState {
        store: SqliteStore::open(&db_path).expect("open temp db"),
    };

    let (old_item, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "stale belief: roadmap completion proves memd functionality".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("team-alpha".to_string()),
                visibility: Some(MemoryVisibility::Workspace),
                belief_branch: None,
                source_agent: Some("codex".to_string()),
                source_system: Some("memd".to_string()),
                source_path: Some("notes.md".to_string()),
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.92),
                ttl_seconds: None,
                last_verified_at: Some(Utc::now()),
                supersedes: Vec::new(),
                tags: vec!["fact".to_string()],
                status: Some(MemoryStatus::Active),
            },
            MemoryStage::Canonical,
        )
        .expect("store stale belief");

    repair::repair_item(
        &state,
        RepairMemoryRequest {
            id: old_item.id,
            mode: MemoryRepairMode::Supersede,
            confidence: Some(0.25),
            status: Some(MemoryStatus::Superseded),
            workspace: None,
            visibility: None,
            source_agent: None,
            source_system: None,
            source_path: None,
            source_quality: None,
            content: None,
            tags: None,
            supersedes: vec![],
        },
    )
    .expect("supersede stale belief");

    let _ = state
        .store_item(
            StoreMemoryRequest {
                content: "corrected fact: roadmap status is not proof of working memory recall"
                    .to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("team-alpha".to_string()),
                visibility: Some(MemoryVisibility::Workspace),
                belief_branch: None,
                source_agent: Some("user".to_string()),
                source_system: Some("correction".to_string()),
                source_path: Some("conversation".to_string()),
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.99),
                ttl_seconds: None,
                last_verified_at: Some(Utc::now()),
                supersedes: vec![old_item.id],
                tags: vec!["correction".to_string()],
                status: Some(MemoryStatus::Active),
            },
            MemoryStage::Canonical,
        )
        .expect("store corrected fact");

    let BuildContextResult { items, .. } = build_context(
        &state,
        &ContextRequest {
            project: Some("demo".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some(MemoryVisibility::Workspace),
            route: Some(RetrievalRoute::ProjectFirst),
            intent: Some(RetrievalIntent::CurrentTask),
            limit: Some(4),
            max_chars_per_item: Some(220),
        },
    )
    .expect("build corrected context");

    assert!(
        items.iter().any(|item| {
            item.content
                .contains("roadmap status is not proof of working memory recall")
        }),
        "corrected fact should be visible after the correction loop"
    );
    assert!(
        !items.iter().any(|item| item
            .content
            .contains("roadmap completion proves memd functionality")),
        "superseded stale belief should not remain in active current-task context"
    );
}

#[test]
fn explicit_store_revives_superseded_canonical_duplicate() {
    let (dir, state) = temp_state("memd-revive-duplicate");
    let (old_item, duplicate) = state
        .store_item(
            StoreMemoryRequest {
                content: "corrected fact: hosted backend health does not prove usable agent memory"
                    .to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: Some(MemoryVisibility::Private),
                belief_branch: None,
                source_agent: Some("codex".to_string()),
                source_system: Some("memd".to_string()),
                source_path: Some("hook-capture-promotion".to_string()),
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.25),
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec!["correction".to_string()],
                status: Some(MemoryStatus::Superseded),
            },
            MemoryStage::Canonical,
        )
        .expect("store old superseded correction");
    assert!(duplicate.is_none());

    let (revived, duplicate) = state
        .store_item(
            StoreMemoryRequest {
                content: "corrected fact: hosted backend health does not prove usable agent memory"
                    .to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: Some(MemoryVisibility::Private),
                belief_branch: None,
                source_agent: Some("codex".to_string()),
                source_system: Some("memd".to_string()),
                source_path: Some("hook-capture-promotion".to_string()),
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.99),
                ttl_seconds: None,
                last_verified_at: Some(Utc::now()),
                supersedes: vec![uuid::Uuid::new_v4()],
                tags: vec!["correction".to_string(), "product-direction".to_string()],
                status: Some(MemoryStatus::Active),
            },
            MemoryStage::Canonical,
        )
        .expect("revive duplicate");

    assert!(duplicate.is_none());
    assert_eq!(revived.id, old_item.id);
    assert_eq!(revived.status, MemoryStatus::Active);
    assert_eq!(revived.confidence, 0.99);
    assert!(revived.tags.iter().any(|tag| tag == "product-direction"));
    assert!(!revived.supersedes.contains(&revived.id));
    std::fs::remove_dir_all(dir).expect("cleanup temp state dir");
}

#[test]
fn store_item_records_source_linked_event_for_canonical_memory() {
    let (dir, state) = temp_state("memd-store-event-canonical");

    let (item, duplicate) = state
        .store_item(
            StoreMemoryRequest {
                content: "raw truth: user corrected deployment target".to_string(),
                kind: MemoryKind::Decision,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("core".to_string()),
                visibility: Some(MemoryVisibility::Workspace),
                belief_branch: None,
                source_agent: Some("codex@test".to_string()),
                source_system: Some("hook-capture".to_string()),
                source_path: Some(".memd/wake.md".to_string()),
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.91),
                ttl_seconds: Some(86_400),
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec!["raw-spine".to_string(), "correction".to_string()],
                status: Some(MemoryStatus::Active),
            },
            MemoryStage::Canonical,
        )
        .expect("store canonical memory");

    assert!(duplicate.is_none());

    let (entity, events) = state
        .entity_view(item.id, 10)
        .expect("load entity timeline");
    assert!(entity.is_some());
    assert!(!events.is_empty(), "expected canonical timeline event");

    let event = &events[0];
    assert_eq!(event.event_type, "canonical_created");
    assert_eq!(event.source_agent.as_deref(), Some("codex@test"));
    assert_eq!(event.source_system.as_deref(), Some("hook-capture"));
    assert_eq!(
        event.source_path.as_deref(),
        Some(".memd/wake.md")
    );
    assert_eq!(
        event.tags,
        vec!["raw-spine".to_string(), "correction".to_string()]
    );
    assert_eq!(
        event
            .context
            .as_ref()
            .and_then(|context| context.repo.as_deref()),
        Some("hook-capture")
    );
    assert_eq!(
        event
            .context
            .as_ref()
            .and_then(|context| context.location.as_deref()),
        Some(".memd/wake.md")
    );
    std::fs::remove_dir_all(dir).expect("cleanup temp state dir");
}

#[test]
fn store_item_records_source_linked_event_for_candidate_memory() {
    let (dir, state) = temp_state("memd-store-event-candidate");

    let (item, duplicate) = state
        .store_item(
            StoreMemoryRequest {
                content: "checkpoint: parser lane blocked by stale resume packet".to_string(),
                kind: MemoryKind::Status,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("core".to_string()),
                visibility: Some(MemoryVisibility::Workspace),
                belief_branch: None,
                source_agent: Some("codex@test".to_string()),
                source_system: Some("checkpoint".to_string()),
                source_path: Some("checkpoint".to_string()),
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.78),
                ttl_seconds: Some(86_400),
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec!["checkpoint".to_string(), "raw-spine".to_string()],
                status: Some(MemoryStatus::Active),
            },
            MemoryStage::Candidate,
        )
        .expect("store candidate memory");

    assert!(duplicate.is_none());

    let (entity, events) = state
        .entity_view(item.id, 10)
        .expect("load entity timeline");
    assert!(entity.is_some());
    assert!(!events.is_empty(), "expected candidate timeline event");

    let event = &events[0];
    assert_eq!(event.event_type, "candidate_created");
    assert_eq!(event.source_agent.as_deref(), Some("codex@test"));
    assert_eq!(event.source_system.as_deref(), Some("checkpoint"));
    assert_eq!(event.source_path.as_deref(), Some("checkpoint"));
    assert_eq!(
        event.tags,
        vec!["checkpoint".to_string(), "raw-spine".to_string()]
    );
    assert_eq!(
        event
            .context
            .as_ref()
            .and_then(|context| context.repo.as_deref()),
        Some("checkpoint")
    );
    assert_eq!(
        event
            .context
            .as_ref()
            .and_then(|context| context.location.as_deref()),
        Some("checkpoint")
    );
    std::fs::remove_dir_all(dir).expect("cleanup temp state dir");
}

#[tokio::test]
async fn source_memory_route_returns_provenance_aggregates_for_filtered_source() {
    let (dir, state) = temp_state("memd-source-memory-route");

    state
        .store_item(
            StoreMemoryRequest {
                content: "raw truth: deployment target corrected to staging".to_string(),
                kind: MemoryKind::Decision,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("core".to_string()),
                visibility: Some(MemoryVisibility::Workspace),
                belief_branch: None,
                source_agent: Some("codex@test".to_string()),
                source_system: Some("hook-capture".to_string()),
                source_path: Some(".memd/wake.md".to_string()),
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.91),
                ttl_seconds: Some(86_400),
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec!["raw-spine".to_string(), "correction".to_string()],
                status: Some(MemoryStatus::Active),
            },
            MemoryStage::Canonical,
        )
        .expect("store filtered provenance item");

    state
        .store_item(
            StoreMemoryRequest {
                content: "other lane memory should not match filtered source".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("core".to_string()),
                visibility: Some(MemoryVisibility::Workspace),
                belief_branch: None,
                source_agent: Some("other@test".to_string()),
                source_system: Some("checkpoint".to_string()),
                source_path: Some("checkpoint".to_string()),
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.65),
                ttl_seconds: Some(86_400),
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec!["checkpoint".to_string()],
                status: Some(MemoryStatus::Active),
            },
            MemoryStage::Candidate,
        )
        .expect("store non-matching provenance item");

    let app = Router::new()
        .route("/memory/source", get(get_source_memory))
        .with_state(state);

    let response = app
            .oneshot(
                Request::builder()
                    .uri("/memory/source?project=memd&namespace=main&workspace=core&source_agent=codex%40test&source_system=hook-capture&limit=5")
                    .body(Body::empty())
                    .expect("build request"),
            )
            .await
            .expect("run source memory route");

    assert_eq!(response.status(), StatusCode::OK);
    let body: SourceMemoryResponse = decode_json(response).await;
    assert_eq!(body.sources.len(), 1);

    let source = &body.sources[0];
    assert_eq!(source.source_agent.as_deref(), Some("codex@test"));
    assert_eq!(source.source_system.as_deref(), Some("hook-capture"));
    assert_eq!(source.project.as_deref(), Some("memd"));
    assert_eq!(source.namespace.as_deref(), Some("main"));
    assert_eq!(source.workspace.as_deref(), Some("core"));
    assert_eq!(source.visibility, MemoryVisibility::Workspace);
    assert_eq!(source.item_count, 1);
    assert_eq!(source.active_count, 1);
    assert_eq!(source.candidate_count, 0);
    assert_eq!(source.contested_count, 0);
    assert!(
        source.tags.iter().any(|tag| tag == "raw-spine"),
        "expected raw truth tag in provenance aggregate"
    );
    assert!(
        source.tags.iter().any(|tag| tag == "correction"),
        "expected correction tag in provenance aggregate"
    );

    std::fs::remove_dir_all(dir).expect("cleanup temp state dir");
}

#[tokio::test]
async fn ui_artifact_handler_returns_detail_response() {
    let state = AppState {
        store: SqliteStore::open(
            std::env::temp_dir().join(format!("memd-ui-detail-{}.db", uuid::Uuid::new_v4())),
        )
        .expect("open temp db"),
    };
    let item = ui::test_insert_visible_item(&state, "runtime spine", true).unwrap();

    let response = super::get_visible_memory_artifact(
        State(state),
        Query(super::VisibleMemoryArtifactQuery { id: item.id }),
    )
    .await
    .expect("build artifact detail")
    .0;

    assert_eq!(response.artifact.id, item.id);
    assert!(response.explain.is_some());
}

#[tokio::test]
async fn ui_action_handler_returns_open_metadata() {
    let state = AppState {
        store: SqliteStore::open(
            std::env::temp_dir().join(format!("memd-ui-action-{}.db", uuid::Uuid::new_v4())),
        )
        .expect("open temp db"),
    };
    let item = ui::test_insert_visible_item(&state, "runtime spine", true).unwrap();

    let response = super::post_visible_memory_action(
        State(state),
        Json(VisibleMemoryUiActionRequest {
            id: item.id,
            action: memd_schema::VisibleMemoryUiActionKind::OpenInObsidian,
        }),
    )
    .await
    .expect("build action response")
    .0;

    assert_eq!(response.artifact_id, item.id);
    assert_eq!(
        response.open_uri.as_deref(),
        Some("obsidian://open?path=wiki/runtime-spine.md")
    );
}

#[tokio::test]
async fn atlas_generate_creates_regions_from_stored_memory() {
    let store = SqliteStore::open(std::env::temp_dir().join(format!("memd-atlas-{}.db", uuid::Uuid::new_v4()))).expect("open test store");
    let state = AppState {
        store: store.clone(),
    };

    // Store several memory items of different kinds
    for (i, kind) in [
        MemoryKind::Fact,
        MemoryKind::Fact,
        MemoryKind::Decision,
        MemoryKind::Decision,
        MemoryKind::Procedural,
        MemoryKind::Procedural,
    ]
    .iter()
    .enumerate()
    {
        let req = StoreMemoryRequest {
            content: format!("test memory item {i}"),
            kind: *kind,
            scope: MemoryScope::Project,
            project: Some("atlas-test".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
            visibility: None,
            belief_branch: None,
            source_agent: None,
            source_system: None,
            source_path: None,
            source_quality: None,
            confidence: Some(0.9),
            ttl_seconds: None,
            last_verified_at: None,
            supersedes: Vec::new(),
            tags: Vec::new(),
            status: None,
        };
        state
            .store_item(req, MemoryStage::Canonical)
            .expect("store test item");
    }

    let regions = store
        .generate_regions_for_project(Some("atlas-test"), Some("main"), None)
        .expect("generate regions");

    assert!(
        regions.len() >= 2,
        "should generate at least 2 regions (facts, decisions), got {}",
        regions.len()
    );

    // Regions should be persisted
    let listed = store
        .list_atlas_regions(&memd_schema::AtlasRegionsRequest {
            project: Some("atlas-test".to_string()),
            namespace: Some("main".to_string()),
            lane: None,
            limit: None,
        })
        .expect("list regions");
    assert!(!listed.regions.is_empty());
}

#[tokio::test]
async fn atlas_explore_returns_nodes_for_region() {
    let store = SqliteStore::open(std::env::temp_dir().join(format!("memd-atlas-{}.db", uuid::Uuid::new_v4()))).expect("open test store");
    let state = AppState {
        store: store.clone(),
    };

    // Store items
    let mut stored_ids = Vec::new();
    for i in 0..3 {
        let req = StoreMemoryRequest {
            content: format!("explore test item {i}"),
            kind: MemoryKind::Fact,
            scope: MemoryScope::Project,
            project: Some("atlas-explore".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
            visibility: None,
            belief_branch: None,
            source_agent: None,
            source_system: None,
            source_path: None,
            source_quality: None,
            confidence: Some(0.8),
            ttl_seconds: None,
            last_verified_at: None,
            supersedes: Vec::new(),
            tags: Vec::new(),
            status: None,
        };
        let (item, _) = state
            .store_item(req, MemoryStage::Canonical)
            .expect("store test item");
        stored_ids.push(item.id);
    }

    // Generate regions
    let regions = store
        .generate_regions_for_project(Some("atlas-explore"), Some("main"), None)
        .expect("generate regions");
    assert!(!regions.is_empty());

    let region = &regions[0];

    // Explore the region
    let response = store
        .explore_atlas(&memd_schema::AtlasExploreRequest {
            region_id: Some(region.id),
            node_id: None,
            project: Some("atlas-explore".to_string()),
            namespace: Some("main".to_string()),
            lane: None,
            depth: Some(0),
            limit: None,
            pivot_time: None,
            pivot_kind: None,
            min_trust: None,
            pivot_scope: None,
            pivot_source_agent: None,
            pivot_source_system: None,
            include_evidence: false,
        })
        .expect("explore atlas");

    assert_eq!(response.nodes.len(), 3);
    assert!(response.region.is_some());
    assert_eq!(response.region.unwrap().id, region.id);
}

#[tokio::test]
async fn atlas_explore_single_node_returns_that_item() {
    let store = SqliteStore::open(std::env::temp_dir().join(format!("memd-atlas-{}.db", uuid::Uuid::new_v4()))).expect("open test store");
    let state = AppState {
        store: store.clone(),
    };

    let req = StoreMemoryRequest {
        content: "single node test".to_string(),
        kind: MemoryKind::Decision,
        scope: MemoryScope::Project,
        project: Some("atlas-single".to_string()),
        namespace: Some("main".to_string()),
        workspace: None,
        visibility: None,
        belief_branch: None,
        source_agent: None,
        source_system: None,
        source_path: None,
        source_quality: None,
        confidence: Some(0.95),
        ttl_seconds: None,
        last_verified_at: None,
        supersedes: Vec::new(),
        tags: Vec::new(),
        status: None,
    };
    let (item, _) = state
        .store_item(req, MemoryStage::Canonical)
        .expect("store test item");

    let response = store
        .explore_atlas(&memd_schema::AtlasExploreRequest {
            region_id: None,
            node_id: Some(item.id),
            project: None,
            namespace: None,
            lane: None,
            depth: Some(0),
            limit: None,
            pivot_time: None,
            pivot_kind: None,
            min_trust: None,
            pivot_scope: None,
            pivot_source_agent: None,
            pivot_source_system: None,
            include_evidence: false,
        })
        .expect("explore single node");

    assert_eq!(response.nodes.len(), 1);
    assert_eq!(response.nodes[0].memory_id, item.id);
    assert_eq!(response.nodes[0].label, "single node test");
}

#[tokio::test]
async fn atlas_pivot_filters_by_min_trust() {
    let store = SqliteStore::open(std::env::temp_dir().join(format!("memd-atlas-{}.db", uuid::Uuid::new_v4()))).expect("open test store");
    let state = AppState {
        store: store.clone(),
    };

    // Store items with different confidence
    for (i, cf) in [0.3, 0.5, 0.9].iter().enumerate() {
        let req = StoreMemoryRequest {
            content: format!("trust filter item {i}"),
            kind: MemoryKind::Fact,
            scope: MemoryScope::Project,
            project: Some("atlas-trust".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
            visibility: None,
            belief_branch: None,
            source_agent: None,
            source_system: None,
            source_path: None,
            source_quality: None,
            confidence: Some(*cf),
            ttl_seconds: None,
            last_verified_at: None,
            supersedes: Vec::new(),
            tags: Vec::new(),
            status: None,
        };
        state
            .store_item(req, MemoryStage::Canonical)
            .expect("store test item");
    }

    let regions = store
        .generate_regions_for_project(Some("atlas-trust"), Some("main"), None)
        .expect("generate regions");
    let region = &regions[0];

    // Explore with min_trust filter
    let response = store
        .explore_atlas(&memd_schema::AtlasExploreRequest {
            region_id: Some(region.id),
            node_id: None,
            project: Some("atlas-trust".to_string()),
            namespace: Some("main".to_string()),
            lane: None,
            depth: Some(0),
            limit: None,
            pivot_time: None,
            pivot_kind: None,
            min_trust: Some(0.8),
            pivot_scope: None,
            pivot_source_agent: None,
            pivot_source_system: None,
            include_evidence: false,
        })
        .expect("explore with trust filter");

    assert_eq!(
        response.nodes.len(),
        1,
        "only the 0.9 confidence item should pass"
    );
}

#[tokio::test]
async fn atlas_explore_generates_trails_for_multi_node_regions() {
    let store = SqliteStore::open(std::env::temp_dir().join(format!("memd-atlas-trails-{}.db", uuid::Uuid::new_v4()))).expect("open test store");
    let state = AppState {
        store: store.clone(),
    };

    // Store items with varying confidence
    for (i, cf) in [0.5, 0.9, 0.7].iter().enumerate() {
        let req = StoreMemoryRequest {
            content: format!("trail item {i}"),
            kind: MemoryKind::Fact,
            scope: MemoryScope::Project,
            project: Some("atlas-trails".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
            visibility: None,
            belief_branch: None,
            source_agent: None,
            source_system: None,
            source_path: None,
            source_quality: None,
            confidence: Some(*cf),
            ttl_seconds: None,
            last_verified_at: None,
            supersedes: Vec::new(),
            tags: Vec::new(),
            status: None,
        };
        state
            .store_item(req, MemoryStage::Canonical)
            .expect("store test item");
    }

    let regions = store
        .generate_regions_for_project(Some("atlas-trails"), Some("main"), None)
        .expect("generate regions");
    let region = &regions[0];

    let response = store
        .explore_atlas(&memd_schema::AtlasExploreRequest {
            region_id: Some(region.id),
            node_id: None,
            project: Some("atlas-trails".to_string()),
            namespace: Some("main".to_string()),
            lane: None,
            depth: Some(0),
            limit: None,
            pivot_time: None,
            pivot_kind: None,
            min_trust: None,
            pivot_scope: None,
            pivot_source_agent: None,
            pivot_source_system: None,
            include_evidence: false,
        })
        .expect("explore atlas with trails");

    // Should have at least a salience trail
    assert!(
        !response.trails.is_empty(),
        "should generate at least one trail for 3+ nodes"
    );
    let salience_trail = response
        .trails
        .iter()
        .find(|t| t.name == "salience")
        .expect("salience trail should exist");
    assert_eq!(salience_trail.nodes.len(), 3);
    // First node in salience trail should be the highest confidence (0.9)
    let first_node = response
        .nodes
        .iter()
        .find(|n| n.id == salience_trail.nodes[0])
        .expect("first trail node should exist in nodes");
    assert!(
        first_node.confidence >= 0.9,
        "salience trail should start with highest confidence node"
    );
}

#[tokio::test]
async fn atlas_explore_time_pivot_filters_recent_items() {
    let store = SqliteStore::open(std::env::temp_dir().join(format!("memd-atlas-time-{}.db", uuid::Uuid::new_v4()))).expect("open test store");
    let state = AppState {
        store: store.clone(),
    };

    // Store items
    for i in 0..3 {
        let req = StoreMemoryRequest {
            content: format!("time pivot item {i}"),
            kind: MemoryKind::Fact,
            scope: MemoryScope::Project,
            project: Some("atlas-time".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
            visibility: None,
            belief_branch: None,
            source_agent: None,
            source_system: None,
            source_path: None,
            source_quality: None,
            confidence: Some(0.8),
            ttl_seconds: None,
            last_verified_at: None,
            supersedes: Vec::new(),
            tags: Vec::new(),
            status: None,
        };
        state
            .store_item(req, MemoryStage::Canonical)
            .expect("store test item");
    }

    let regions = store
        .generate_regions_for_project(Some("atlas-time"), Some("main"), None)
        .expect("generate regions");
    let region = &regions[0];

    // Use a pivot_time far in the past — should filter out all items
    let old_time = chrono::DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);

    let response = store
        .explore_atlas(&memd_schema::AtlasExploreRequest {
            region_id: Some(region.id),
            node_id: None,
            project: Some("atlas-time".to_string()),
            namespace: Some("main".to_string()),
            lane: None,
            depth: Some(0),
            limit: None,
            pivot_time: Some(old_time),
            pivot_kind: None,
            min_trust: None,
            pivot_scope: None,
            pivot_source_agent: None,
            pivot_source_system: None,
            include_evidence: false,
        })
        .expect("explore with time pivot");

    assert_eq!(
        response.nodes.len(),
        0,
        "all items created after 2020 should be filtered out"
    );

    // Now use a pivot_time in the future — should keep all items
    let future_time = chrono::DateTime::parse_from_rfc3339("2030-01-01T00:00:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);

    let response = store
        .explore_atlas(&memd_schema::AtlasExploreRequest {
            region_id: Some(region.id),
            node_id: None,
            project: Some("atlas-time".to_string()),
            namespace: Some("main".to_string()),
            lane: None,
            depth: Some(0),
            limit: None,
            pivot_time: Some(future_time),
            pivot_kind: None,
            min_trust: None,
            pivot_scope: None,
            pivot_source_agent: None,
            pivot_source_system: None,
            include_evidence: false,
        })
        .expect("explore with future time pivot");

    assert_eq!(
        response.nodes.len(),
        3,
        "all items should pass future time pivot"
    );
}

#[tokio::test]
async fn atlas_lane_tags_create_lane_specific_regions() {
    let store = SqliteStore::open(std::env::temp_dir().join(format!("memd-atlas-lanes-{}.db", uuid::Uuid::new_v4()))).expect("open test store");
    let state = AppState {
        store: store.clone(),
    };

    // Store items with lane tags
    for i in 0..3 {
        let req = StoreMemoryRequest {
            content: format!("design item {i}"),
            kind: MemoryKind::Fact,
            scope: MemoryScope::Project,
            project: Some("atlas-lanes".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
            visibility: None,
            belief_branch: None,
            source_agent: None,
            source_system: None,
            source_path: None,
            source_quality: None,
            confidence: Some(0.8),
            ttl_seconds: None,
            last_verified_at: None,
            supersedes: Vec::new(),
            tags: vec!["lane:design".to_string()],
            status: None,
        };
        state
            .store_item(req, MemoryStage::Canonical)
            .expect("store test item");
    }
    // Also store non-lane items
    for i in 0..2 {
        let req = StoreMemoryRequest {
            content: format!("untagged item {i}"),
            kind: MemoryKind::Decision,
            scope: MemoryScope::Project,
            project: Some("atlas-lanes".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
            visibility: None,
            belief_branch: None,
            source_agent: None,
            source_system: None,
            source_path: None,
            source_quality: None,
            confidence: Some(0.8),
            ttl_seconds: None,
            last_verified_at: None,
            supersedes: Vec::new(),
            tags: Vec::new(),
            status: None,
        };
        state
            .store_item(req, MemoryStage::Canonical)
            .expect("store test item");
    }

    // Generate all regions
    let all_regions = store
        .generate_regions_for_project(Some("atlas-lanes"), Some("main"), None)
        .expect("generate all regions");
    let design_region = all_regions
        .iter()
        .find(|r| r.name == "design")
        .expect("design lane region should exist");
    assert_eq!(design_region.node_count, 3);

    // Filter by lane
    let lane_regions = store
        .generate_regions_for_project(Some("atlas-lanes"), Some("main"), Some("design"))
        .expect("generate lane regions");
    assert_eq!(lane_regions.len(), 1);
    assert_eq!(lane_regions[0].name, "design");
}

#[tokio::test]
async fn atlas_expand_returns_neighborhood_for_seed_items() {
    let store = SqliteStore::open(std::env::temp_dir().join(format!(
        "memd-atlas-expand-{}.db",
        uuid::Uuid::new_v4()
    )))
    .expect("open test store");
    let state = AppState {
        store: store.clone(),
    };

    let req = StoreMemoryRequest {
        content: "expand seed item".to_string(),
        kind: MemoryKind::Fact,
        scope: MemoryScope::Project,
        project: Some("atlas-expand".to_string()),
        namespace: Some("main".to_string()),
        workspace: None,
        visibility: None,
        belief_branch: None,
        source_agent: None,
        source_system: None,
        source_path: None,
        source_quality: None,
        confidence: Some(0.9),
        ttl_seconds: None,
        last_verified_at: None,
        supersedes: Vec::new(),
        tags: Vec::new(),
        status: None,
    };
    let (item, _) = state
        .store_item(req, MemoryStage::Canonical)
        .expect("store test item");

    // Expand from the stored item (no entity links exist, so expansion should
    // return empty but not error)
    let response = store
        .atlas_expand(&memd_schema::AtlasExpandRequest {
            memory_ids: vec![item.id],
            project: Some("atlas-expand".to_string()),
            namespace: Some("main".to_string()),
            depth: Some(1),
            limit: Some(10),
        })
        .expect("atlas expand");

    assert_eq!(response.seed_count, 1);
    // No entity links → no expanded nodes
    assert!(response.expanded_nodes.is_empty());
    assert!(response.links.is_empty());
}

#[tokio::test]
async fn atlas_nodes_include_evidence_count() {
    let store = SqliteStore::open(std::env::temp_dir().join(format!(
        "memd-atlas-evidence-{}.db",
        uuid::Uuid::new_v4()
    )))
    .expect("open test store");
    let state = AppState {
        store: store.clone(),
    };

    let req = StoreMemoryRequest {
        content: "evidence count item".to_string(),
        kind: MemoryKind::Decision,
        scope: MemoryScope::Project,
        project: Some("atlas-evidence".to_string()),
        namespace: Some("main".to_string()),
        workspace: None,
        visibility: None,
        belief_branch: None,
        source_agent: None,
        source_system: None,
        source_path: None,
        source_quality: None,
        confidence: Some(0.85),
        ttl_seconds: None,
        last_verified_at: None,
        supersedes: Vec::new(),
        tags: Vec::new(),
        status: None,
    };
    let (item, _) = state
        .store_item(req, MemoryStage::Canonical)
        .expect("store test item");

    // Explore single node — evidence_count should be populated
    let response = store
        .explore_atlas(&memd_schema::AtlasExploreRequest {
            region_id: None,
            node_id: Some(item.id),
            project: None,
            namespace: None,
            lane: None,
            depth: Some(0),
            limit: None,
            pivot_time: None,
            pivot_kind: None,
            min_trust: None,
            pivot_scope: None,
            pivot_source_agent: None,
            pivot_source_system: None,
            include_evidence: false,
        })
        .expect("explore single node");

    assert_eq!(response.nodes.len(), 1);
    // store_item records an event, so evidence_count >= 1
    assert!(
        response.nodes[0].evidence_count >= 1,
        "node should have at least 1 evidence event from store, got {}",
        response.nodes[0].evidence_count
    );
}

#[tokio::test]
async fn atlas_rename_region_persists_new_name() {
    let store = SqliteStore::open(std::env::temp_dir().join(format!(
        "memd-atlas-rename-{}.db",
        uuid::Uuid::new_v4()
    )))
    .expect("open test store");
    let state = AppState {
        store: store.clone(),
    };

    // Create items so regions can be generated
    for i in 0..3 {
        state
            .store_item(
                StoreMemoryRequest {
                    content: format!("rename test {i}"),
                    kind: MemoryKind::Fact,
                    scope: MemoryScope::Project,
                    project: Some("atlas-rename".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: None,
                    visibility: None,
                    belief_branch: None,
                    source_agent: None,
                    source_system: None,
                    source_path: None,
                    source_quality: None,
                    confidence: Some(0.8),
                    ttl_seconds: None,
                    last_verified_at: None,
                    supersedes: Vec::new(),
                    tags: Vec::new(),
                    status: None,
                },
                MemoryStage::Canonical,
            )
            .expect("store item");
    }

    let regions = store
        .generate_regions_for_project(Some("atlas-rename"), Some("main"), None)
        .expect("generate");
    let region = &regions[0];

    let response = store
        .rename_atlas_region(&memd_schema::AtlasRenameRegionRequest {
            region_id: region.id,
            name: "Custom Region Name".to_string(),
            description: Some("user-curated region".to_string()),
        })
        .expect("rename region");

    assert_eq!(response.region.name, "Custom Region Name");
    assert_eq!(
        response.region.description.as_deref(),
        Some("user-curated region")
    );
    assert!(!response.region.auto_generated);

    // Verify persistence
    let listed = store
        .list_atlas_regions(&memd_schema::AtlasRegionsRequest {
            project: Some("atlas-rename".to_string()),
            namespace: Some("main".to_string()),
            lane: None,
            limit: None,
        })
        .expect("list");
    let found = listed
        .regions
        .iter()
        .find(|r| r.id == region.id)
        .expect("region should still exist");
    assert_eq!(found.name, "Custom Region Name");
}

#[tokio::test]
async fn atlas_tag_overlap_fallback_finds_neighbors() {
    let store = SqliteStore::open(std::env::temp_dir().join(format!(
        "memd-atlas-tagfallback-{}.db",
        uuid::Uuid::new_v4()
    )))
    .expect("open test store");
    let state = AppState {
        store: store.clone(),
    };

    // Store a seed item with tags
    let (seed, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "seed with tags".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("atlas-tagfb".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: None,
                belief_branch: None,
                source_agent: None,
                source_system: None,
                source_path: None,
                source_quality: None,
                confidence: Some(0.9),
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec!["auth".to_string(), "security".to_string()],
                status: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store seed");

    // Store a neighbor sharing a tag
    state
        .store_item(
            StoreMemoryRequest {
                content: "neighbor sharing auth tag".to_string(),
                kind: MemoryKind::Decision,
                scope: MemoryScope::Project,
                project: Some("atlas-tagfb".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: None,
                belief_branch: None,
                source_agent: None,
                source_system: None,
                source_path: None,
                source_quality: None,
                confidence: Some(0.85),
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec!["auth".to_string(), "migration".to_string()],
                status: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store neighbor");

    // Store an unrelated item
    state
        .store_item(
            StoreMemoryRequest {
                content: "unrelated item no shared tags".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("atlas-tagfb".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: None,
                belief_branch: None,
                source_agent: None,
                source_system: None,
                source_path: None,
                source_quality: None,
                confidence: Some(0.7),
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec!["unrelated".to_string()],
                status: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store unrelated");

    // Explore from seed with depth=1, no entity links exist so tag fallback kicks in
    let response = store
        .explore_atlas(&memd_schema::AtlasExploreRequest {
            region_id: None,
            node_id: Some(seed.id),
            project: Some("atlas-tagfb".to_string()),
            namespace: Some("main".to_string()),
            lane: None,
            depth: Some(1),
            limit: None,
            pivot_time: None,
            pivot_kind: None,
            pivot_scope: None,
            pivot_source_agent: None,
            pivot_source_system: None,
            min_trust: None,
            include_evidence: false,
        })
        .expect("explore with tag fallback");

    // Should find the seed + the neighbor (shares "auth" tag), but not the unrelated item
    assert_eq!(
        response.nodes.len(),
        2,
        "should find seed + 1 tag-overlap neighbor, got {}",
        response.nodes.len()
    );
    assert!(
        response
            .nodes
            .iter()
            .any(|n| n.label.contains("neighbor sharing auth")),
        "should include tag-overlap neighbor"
    );
    assert!(
        !response
            .nodes
            .iter()
            .any(|n| n.label.contains("unrelated")),
        "should NOT include unrelated item"
    );
}

#[tokio::test]
async fn atlas_explore_with_evidence_returns_events() {
    let store = SqliteStore::open(std::env::temp_dir().join(format!(
        "memd-atlas-evidence-drill-{}.db",
        uuid::Uuid::new_v4()
    )))
    .expect("open test store");
    let state = AppState {
        store: store.clone(),
    };

    let (item, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "evidence drill test".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("atlas-ev".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: None,
                belief_branch: None,
                source_agent: None,
                source_system: None,
                source_path: None,
                source_quality: None,
                confidence: Some(0.9),
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: Vec::new(),
                status: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store item");

    // Explore with include_evidence=true
    let response = store
        .explore_atlas(&memd_schema::AtlasExploreRequest {
            region_id: None,
            node_id: Some(item.id),
            project: None,
            namespace: None,
            lane: None,
            depth: Some(0),
            limit: None,
            pivot_time: None,
            pivot_kind: None,
            pivot_scope: None,
            pivot_source_agent: None,
            pivot_source_system: None,
            min_trust: None,
            include_evidence: true,
        })
        .expect("explore with evidence");

    assert_eq!(response.nodes.len(), 1);
    // store_item records events, so evidence should be non-empty
    assert!(
        !response.evidence.is_empty(),
        "evidence should contain events from store"
    );
}

#[tokio::test]
async fn atlas_scope_pivot_filters_by_scope() {
    let store = SqliteStore::open(std::env::temp_dir().join(format!(
        "memd-atlas-scope-{}.db",
        uuid::Uuid::new_v4()
    )))
    .expect("open test store");
    let state = AppState {
        store: store.clone(),
    };

    // Store project-scoped and global-scoped items
    state
        .store_item(
            StoreMemoryRequest {
                content: "project scoped".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("atlas-scope".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: None,
                belief_branch: None,
                source_agent: None,
                source_system: None,
                source_path: None,
                source_quality: None,
                confidence: Some(0.8),
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: Vec::new(),
                status: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store project item");

    let (global_item, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "global scoped".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Global,
                project: Some("atlas-scope".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: None,
                belief_branch: None,
                source_agent: None,
                source_system: None,
                source_path: None,
                source_quality: None,
                confidence: Some(0.8),
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: Vec::new(),
                status: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store global item");

    let regions = store
        .generate_regions_for_project(Some("atlas-scope"), Some("main"), None)
        .expect("generate");
    let region = &regions[0];

    // Pivot by global scope
    let response = store
        .explore_atlas(&memd_schema::AtlasExploreRequest {
            region_id: Some(region.id),
            node_id: None,
            project: Some("atlas-scope".to_string()),
            namespace: Some("main".to_string()),
            lane: None,
            depth: Some(0),
            limit: None,
            pivot_time: None,
            pivot_kind: None,
            pivot_scope: Some(MemoryScope::Global),
            pivot_source_agent: None,
            pivot_source_system: None,
            min_trust: None,
            include_evidence: false,
        })
        .expect("explore with scope pivot");

    assert_eq!(
        response.nodes.len(),
        1,
        "only global-scoped item should pass"
    );
    assert_eq!(response.nodes[0].memory_id, global_item.id);
}
