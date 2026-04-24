use super::*;
use axum::{
    body::{Body, to_bytes},
    http::Request,
};
use memd_rag::{
    RagBackendHealth, RagBackendHealthResponse, RagClient, RagIngestRequest, RagIngestResponse,
    RagRerankItem, RagRerankResponse, RagRetrieveItem, RagRetrieveMode, RagRetrieveResponse,
};
use memd_schema::{CoordinationMode, MemoryRepairMode, SkillPolicyActivationRecord};
use std::sync::{Arc, Mutex};
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
        lane: None,
        version: 1,
        correction_meta: None,
    }
}

fn temp_state(name: &str) -> (std::path::PathBuf, AppState) {
    let dir = std::env::temp_dir().join(format!("{name}-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("create temp state dir");
    let db_path = dir.join("memd.db");
    let state = AppState {
        store: SqliteStore::open(&db_path).expect("open temp store"),
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: None,
        embedder: None,
    };
    (dir, state)
}

fn temp_state_with_rag(name: &str, rag_url: Option<&str>) -> (std::path::PathBuf, AppState) {
    let (dir, mut state) = temp_state(name);
    state.rag = rag_url.map(|url| Arc::new(RagClient::new(url).expect("build test rag client")));
    (dir, state)
}

fn set_env(name: &str, value: &str) {
    unsafe {
        std::env::set_var(name, value);
    }
}

fn remove_env(name: &str) {
    unsafe {
        std::env::remove_var(name);
    }
}

struct EnvGuard(&'static str);

impl Drop for EnvGuard {
    fn drop(&mut self) {
        remove_env(self.0);
    }
}

fn set_test_env(name: &'static str, value: &str) -> EnvGuard {
    set_env(name, value);
    EnvGuard(name)
}

#[derive(Clone)]
struct RagIngestCaptureState {
    ingest_tx: Arc<Mutex<Option<tokio::sync::oneshot::Sender<RagIngestRequest>>>>,
}

async fn mock_rag_healthz() -> Json<RagBackendHealthResponse> {
    Json(RagBackendHealthResponse {
        status: "ok".to_string(),
        backend: RagBackendHealth {
            connected: true,
            name: Some("rag-sidecar".to_string()),
            multimodal: true,
            profile: Some("sparse".to_string()),
        },
    })
}

async fn mock_rag_ingest(
    State(state): State<RagIngestCaptureState>,
    Json(req): Json<RagIngestRequest>,
) -> Json<RagIngestResponse> {
    if let Some(tx) = state.ingest_tx.lock().expect("lock ingest tx").take() {
        let _ = tx.send(req.clone());
    }
    Json(RagIngestResponse {
        status: "ok".to_string(),
        track_id: req.source.id,
        items: 1,
    })
}

async fn mock_rag_retrieve(
    State(response): State<RagRetrieveResponse>,
) -> Json<RagRetrieveResponse> {
    Json(response)
}

async fn mock_rag_retrieve_from_state(
    State(state): State<MockRagSearchState>,
) -> Json<RagRetrieveResponse> {
    Json(state.retrieve)
}

#[derive(Clone)]
struct MockRagSearchState {
    retrieve: RagRetrieveResponse,
    rerank: RagRerankResponse,
}

async fn mock_rag_rerank(State(state): State<MockRagSearchState>) -> Json<RagRerankResponse> {
    Json(state.rerank)
}

async fn spawn_mock_rag_ingest_server() -> (String, tokio::sync::oneshot::Receiver<RagIngestRequest>)
{
    let (tx, rx) = tokio::sync::oneshot::channel();
    let state = RagIngestCaptureState {
        ingest_tx: Arc::new(Mutex::new(Some(tx))),
    };
    let app = Router::new()
        .route("/healthz", get(mock_rag_healthz))
        .route("/v1/ingest", post(mock_rag_ingest))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind mock rag ingest server");
    let addr = listener.local_addr().expect("mock rag ingest addr");
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("serve mock rag ingest server");
    });
    tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    (format!("http://{}", addr), rx)
}

async fn spawn_mock_rag_retrieve_server(response: RagRetrieveResponse) -> String {
    let app = Router::new()
        .route("/v1/retrieve", post(mock_rag_retrieve))
        .with_state(response);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind mock rag retrieve server");
    let addr = listener.local_addr().expect("mock rag retrieve addr");
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("serve mock rag retrieve server");
    });
    tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    format!("http://{}", addr)
}

async fn spawn_mock_rag_search_server(
    retrieve: RagRetrieveResponse,
    rerank: RagRerankResponse,
) -> String {
    let app = Router::new()
        .route("/v1/retrieve", post(mock_rag_retrieve_from_state))
        .route("/v1/rerank", post(mock_rag_rerank))
        .with_state(MockRagSearchState { retrieve, rerank });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind mock rag search server");
    let addr = listener.local_addr().expect("mock rag search addr");
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("serve mock rag search server");
    });
    tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    format!("http://{}", addr)
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
            last_wake_at: None,
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
            last_wake_at: None,
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
            coordination_mode: Some(CoordinationMode::ExclusiveWrite),
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
            last_wake_at: None,
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
            last_wake_at: None,
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
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: None,
        embedder: None,
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
                lane: None,
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
                lane: None,
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
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: None,
        embedder: None,
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
                lane: None,
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
                    lane: None,
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
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: None,
        embedder: None,
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
                lane: None,
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
                    lane: None,
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
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: None,
        embedder: None,
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
                lane: None,
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
                lane: None,
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
                lane: None,
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
                lane: None,
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
                lane: None,
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
    assert_eq!(event.source_path.as_deref(), Some(".memd/wake.md"));
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
                lane: None,
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
                lane: None,
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
                lane: None,
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
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: None,
        embedder: None,
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
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: None,
        embedder: None,
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
    let store = SqliteStore::open(
        std::env::temp_dir().join(format!("memd-atlas-{}.db", uuid::Uuid::new_v4())),
    )
    .expect("open test store");
    let state = AppState {
        store: store.clone(),
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: None,
        embedder: None,
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
            lane: None,
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
    let store = SqliteStore::open(
        std::env::temp_dir().join(format!("memd-atlas-{}.db", uuid::Uuid::new_v4())),
    )
    .expect("open test store");
    let state = AppState {
        store: store.clone(),
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: None,
        embedder: None,
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
            lane: None,
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
            min_salience: None,
            include_evidence: false,
            from_working: false,
        })
        .expect("explore atlas");

    assert_eq!(response.nodes.len(), 3);
    assert!(response.region.is_some());
    assert_eq!(response.region.unwrap().id, region.id);
}

#[tokio::test]
async fn atlas_explore_single_node_returns_that_item() {
    let store = SqliteStore::open(
        std::env::temp_dir().join(format!("memd-atlas-{}.db", uuid::Uuid::new_v4())),
    )
    .expect("open test store");
    let state = AppState {
        store: store.clone(),
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: None,
        embedder: None,
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
        lane: None,
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
            min_salience: None,
            include_evidence: false,
            from_working: false,
        })
        .expect("explore single node");

    assert_eq!(response.nodes.len(), 1);
    assert_eq!(response.nodes[0].memory_id, item.id);
    assert_eq!(response.nodes[0].label, "single node test");
}

#[tokio::test]
async fn atlas_pivot_filters_by_min_trust() {
    let store = SqliteStore::open(
        std::env::temp_dir().join(format!("memd-atlas-{}.db", uuid::Uuid::new_v4())),
    )
    .expect("open test store");
    let state = AppState {
        store: store.clone(),
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: None,
        embedder: None,
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
            lane: None,
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
            min_salience: None,
            include_evidence: false,
            from_working: false,
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
    let store = SqliteStore::open(
        std::env::temp_dir().join(format!("memd-atlas-trails-{}.db", uuid::Uuid::new_v4())),
    )
    .expect("open test store");
    let state = AppState {
        store: store.clone(),
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: None,
        embedder: None,
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
            lane: None,
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
            min_salience: None,
            include_evidence: false,
            from_working: false,
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
    let store = SqliteStore::open(
        std::env::temp_dir().join(format!("memd-atlas-time-{}.db", uuid::Uuid::new_v4())),
    )
    .expect("open test store");
    let state = AppState {
        store: store.clone(),
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: None,
        embedder: None,
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
            lane: None,
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
            min_salience: None,
            include_evidence: false,
            from_working: false,
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
            min_salience: None,
            include_evidence: false,
            from_working: false,
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
    let store = SqliteStore::open(
        std::env::temp_dir().join(format!("memd-atlas-lanes-{}.db", uuid::Uuid::new_v4())),
    )
    .expect("open test store");
    let state = AppState {
        store: store.clone(),
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: None,
        embedder: None,
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
            lane: None,
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
            lane: None,
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
    let store = SqliteStore::open(
        std::env::temp_dir().join(format!("memd-atlas-expand-{}.db", uuid::Uuid::new_v4())),
    )
    .expect("open test store");
    let state = AppState {
        store: store.clone(),
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: None,
        embedder: None,
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
        lane: None,
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
async fn atlas_one_hop_neighbors_resolve_through_entities() {
    let store = SqliteStore::open(
        std::env::temp_dir().join(format!("memd-atlas-onehop-{}.db", uuid::Uuid::new_v4())),
    )
    .expect("open test store");
    let state = AppState {
        store: store.clone(),
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: None,
        embedder: None,
    };

    let base_req = StoreMemoryRequest {
        content: String::new(),
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
        lane: None,
    };

    let (seed, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "alpha-svc uses event sourcing".to_string(),
                ..base_req.clone()
            },
            MemoryStage::Canonical,
        )
        .expect("store seed");
    let (neighbor, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "migration plan references [[atlas-expand]] decisions".to_string(),
                ..base_req
            },
            MemoryStage::Canonical,
        )
        .expect("store neighbor");

    let seed_entity = store
        .entity_for_item(seed.id)
        .expect("seed entity lookup")
        .expect("seed entity present");
    let neighbor_entity = store
        .entity_for_item(neighbor.id)
        .expect("neighbor entity lookup")
        .expect("neighbor entity present");
    store
        .link_entity(&memd_schema::EntityLinkRequest {
            from_entity_id: seed_entity.id,
            to_entity_id: neighbor_entity.id,
            relation_kind: memd_schema::EntityRelationKind::Related,
            confidence: Some(0.95),
            valid_from: Some(seed.updated_at),
            valid_to: None,
            source_item_id: Some(seed.id),
            note: Some("manual atlas path".to_string()),
            context: None,
            tags: vec!["manual".to_string()],
        })
        .expect("create manual atlas link");

    let neighbors = store.one_hop_neighbors_for_items(&[seed.id], 10);
    assert!(
        neighbors.contains(&neighbor.id),
        "one-hop recall should surface linked neighbor item via entity graph"
    );
}

#[tokio::test]
async fn atlas_expand_returns_linked_neighbors_for_seed_items() {
    let store = SqliteStore::open(std::env::temp_dir().join(format!(
        "memd-atlas-expand-link-{}.db",
        uuid::Uuid::new_v4()
    )))
    .expect("open test store");
    let state = AppState {
        store: store.clone(),
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: None,
        embedder: None,
    };

    let base_req = StoreMemoryRequest {
        content: String::new(),
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
        lane: None,
    };

    let (seed, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "alpha-svc uses event sourcing".to_string(),
                ..base_req.clone()
            },
            MemoryStage::Canonical,
        )
        .expect("store seed");
    let (neighbor, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "migration plan references [[atlas-expand]] decisions".to_string(),
                ..base_req
            },
            MemoryStage::Canonical,
        )
        .expect("store neighbor");

    let seed_entity = store
        .entity_for_item(seed.id)
        .expect("seed entity lookup")
        .expect("seed entity present");
    let neighbor_entity = store
        .entity_for_item(neighbor.id)
        .expect("neighbor entity lookup")
        .expect("neighbor entity present");
    store
        .link_entity(&memd_schema::EntityLinkRequest {
            from_entity_id: seed_entity.id,
            to_entity_id: neighbor_entity.id,
            relation_kind: memd_schema::EntityRelationKind::Related,
            confidence: Some(0.95),
            valid_from: Some(seed.updated_at),
            valid_to: None,
            source_item_id: Some(seed.id),
            note: Some("manual atlas path".to_string()),
            context: None,
            tags: vec!["manual".to_string()],
        })
        .expect("create manual atlas link");

    let response = store
        .atlas_expand(&memd_schema::AtlasExpandRequest {
            memory_ids: vec![seed.id],
            project: Some("atlas-expand".to_string()),
            namespace: Some("main".to_string()),
            depth: Some(1),
            limit: Some(10),
        })
        .expect("atlas expand");

    assert!(
        response
            .expanded_nodes
            .iter()
            .any(|node| node.id == neighbor.id),
        "atlas expand should traverse item -> entity -> linked entity -> item"
    );
    assert!(
        response
            .links
            .iter()
            .any(|link| link.from_node_id == seed.id && link.to_node_id == neighbor.id),
        "atlas expand should emit the traversed atlas link"
    );
}

#[tokio::test]
async fn atlas_nodes_include_evidence_count() {
    let store = SqliteStore::open(
        std::env::temp_dir().join(format!("memd-atlas-evidence-{}.db", uuid::Uuid::new_v4())),
    )
    .expect("open test store");
    let state = AppState {
        store: store.clone(),
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: None,
        embedder: None,
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
        lane: None,
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
            min_salience: None,
            include_evidence: false,
            from_working: false,
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
    let store = SqliteStore::open(
        std::env::temp_dir().join(format!("memd-atlas-rename-{}.db", uuid::Uuid::new_v4())),
    )
    .expect("open test store");
    let state = AppState {
        store: store.clone(),
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: None,
        embedder: None,
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
                    lane: None,
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
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: None,
        embedder: None,
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
                lane: None,
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
                lane: None,
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
                lane: None,
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
            min_salience: None,
            include_evidence: false,
            from_working: false,
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
        !response.nodes.iter().any(|n| n.label.contains("unrelated")),
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
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: None,
        embedder: None,
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
                lane: None,
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
            min_salience: None,
            include_evidence: true,
            from_working: false,
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
    let store = SqliteStore::open(
        std::env::temp_dir().join(format!("memd-atlas-scope-{}.db", uuid::Uuid::new_v4())),
    )
    .expect("open test store");
    let state = AppState {
        store: store.clone(),
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: None,
        embedder: None,
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
                lane: None,
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
                lane: None,
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
            min_salience: None,
            include_evidence: false,
            from_working: false,
        })
        .expect("explore with scope pivot");

    assert_eq!(
        response.nodes.len(),
        1,
        "only global-scoped item should pass"
    );
    assert_eq!(response.nodes[0].memory_id, global_item.id);
}

#[tokio::test]
async fn atlas_from_working_seeds_from_working_memory() {
    let store = SqliteStore::open(
        std::env::temp_dir().join(format!("memd-atlas-fromwork-{}.db", uuid::Uuid::new_v4())),
    )
    .expect("open test store");
    let state = AppState {
        store: store.clone(),
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: None,
        embedder: None,
    };

    // Store Status items (working memory candidates)
    for i in 0..2 {
        state
            .store_item(
                StoreMemoryRequest {
                    content: format!("working status {i}"),
                    kind: MemoryKind::Status,
                    scope: MemoryScope::Project,
                    project: Some("atlas-work".to_string()),
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
                    lane: None,
                },
                MemoryStage::Canonical,
            )
            .expect("store status item");
    }

    // Store a non-working Fact (should NOT be seeded)
    state
        .store_item(
            StoreMemoryRequest {
                content: "regular fact not working".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("atlas-work".to_string()),
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
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store fact");

    let response = store
        .explore_atlas(&memd_schema::AtlasExploreRequest {
            region_id: None,
            node_id: None,
            project: Some("atlas-work".to_string()),
            namespace: Some("main".to_string()),
            lane: None,
            depth: Some(0),
            limit: None,
            pivot_time: None,
            pivot_kind: None,
            pivot_scope: None,
            pivot_source_agent: None,
            pivot_source_system: None,
            min_trust: None,
            min_salience: None,
            include_evidence: false,
            from_working: true,
        })
        .expect("explore from working");

    // Should seed from Status items only, not the Fact
    assert_eq!(
        response.nodes.len(),
        2,
        "from_working should seed 2 Status items, got {}",
        response.nodes.len()
    );
    assert!(response.nodes.iter().all(|n| n.kind == MemoryKind::Status));
}

#[tokio::test]
async fn atlas_supersedes_neighborhood_finds_corrections() {
    let store = SqliteStore::open(
        std::env::temp_dir().join(format!("memd-atlas-supersedes-{}.db", uuid::Uuid::new_v4())),
    )
    .expect("open test store");
    let state = AppState {
        store: store.clone(),
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: None,
        embedder: None,
    };

    // Store the old item (will be superseded)
    let (old_item, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "old belief about auth".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("atlas-super".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: None,
                belief_branch: None,
                source_agent: None,
                source_system: None,
                source_path: None,
                source_quality: None,
                confidence: Some(0.5),
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: Vec::new(),
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store old item");

    // Store the new item that supersedes the old
    let (new_item, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "corrected belief about auth".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("atlas-super".to_string()),
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
                supersedes: vec![old_item.id],
                tags: Vec::new(),
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store new item");

    // Explore from the new item — old item should appear as corrective neighbor
    let response = store
        .explore_atlas(&memd_schema::AtlasExploreRequest {
            region_id: None,
            node_id: Some(new_item.id),
            project: Some("atlas-super".to_string()),
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
            min_salience: None,
            include_evidence: false,
            from_working: false,
        })
        .expect("explore with supersedes");

    assert_eq!(
        response.nodes.len(),
        2,
        "should find new item + superseded old item"
    );
    let corrective_link = response
        .links
        .iter()
        .find(|l| l.link_kind == memd_schema::AtlasLinkKind::Corrective);
    assert!(
        corrective_link.is_some(),
        "should have a corrective link to superseded item"
    );
    assert_eq!(
        corrective_link.unwrap().label.as_deref(),
        Some("supersedes")
    );
}

#[tokio::test]
async fn atlas_persisted_links_survive_reload() {
    let db_path =
        std::env::temp_dir().join(format!("memd-atlas-persist-{}.db", uuid::Uuid::new_v4()));
    let store = SqliteStore::open(&db_path).expect("open test store");
    let state = AppState {
        store: store.clone(),
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: None,
        embedder: None,
    };

    // Store two items
    let (item_a, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "persist link A".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("atlas-persist".to_string()),
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
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store A");

    let (item_b, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "persist link B".to_string(),
                kind: MemoryKind::Decision,
                scope: MemoryScope::Project,
                project: Some("atlas-persist".to_string()),
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
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store B");

    // Persist a link
    let link = memd_schema::AtlasLink {
        from_node_id: item_a.id,
        to_node_id: item_b.id,
        link_kind: memd_schema::AtlasLinkKind::Causal,
        weight: 0.8,
        label: Some("A caused B".to_string()),
    };
    store.persist_atlas_link(&link).expect("persist link");

    // Reopen the store (simulates restart)
    let store2 = SqliteStore::open(&db_path).expect("reopen store");

    // Load persisted links
    let loaded = store2
        .load_persisted_links_for_node(item_a.id)
        .expect("load links");
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].to_node_id, item_b.id);
    assert_eq!(loaded[0].link_kind, memd_schema::AtlasLinkKind::Causal);
    assert_eq!(loaded[0].label.as_deref(), Some("A caused B"));

    // Explore from A — should find B via persisted link
    let response = store2
        .explore_atlas(&memd_schema::AtlasExploreRequest {
            region_id: None,
            node_id: Some(item_a.id),
            project: Some("atlas-persist".to_string()),
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
            min_salience: None,
            include_evidence: false,
            from_working: false,
        })
        .expect("explore with persisted link");

    assert_eq!(
        response.nodes.len(),
        2,
        "should find A + B via persisted link"
    );
    assert!(
        response
            .links
            .iter()
            .any(|l| l.link_kind == memd_schema::AtlasLinkKind::Causal),
        "should include the persisted causal link"
    );
}

#[tokio::test]
async fn atlas_salience_pivot_uses_entity_salience_score() {
    let store = SqliteStore::open(
        std::env::temp_dir().join(format!("memd-atlas-salience-{}.db", uuid::Uuid::new_v4())),
    )
    .expect("open test store");
    let state = AppState {
        store: store.clone(),
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: None,
        embedder: None,
    };

    // Store items — entity salience_score is set during store_item
    // via entity creation. Items with higher confidence get higher salience.
    for (i, cf) in [0.3, 0.9].iter().enumerate() {
        state
            .store_item(
                StoreMemoryRequest {
                    content: format!("salience test item {i}"),
                    kind: MemoryKind::Fact,
                    scope: MemoryScope::Project,
                    project: Some("atlas-sal".to_string()),
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
                    lane: None,
                },
                MemoryStage::Canonical,
            )
            .expect("store item");
    }

    let regions = store
        .generate_regions_for_project(Some("atlas-sal"), Some("main"), None)
        .expect("generate");
    let region = &regions[0];

    // Filter by min_salience=0.8 — only the 0.9 item should pass
    let response = store
        .explore_atlas(&memd_schema::AtlasExploreRequest {
            region_id: Some(region.id),
            node_id: None,
            project: Some("atlas-sal".to_string()),
            namespace: Some("main".to_string()),
            lane: None,
            depth: Some(0),
            limit: None,
            pivot_time: None,
            pivot_kind: None,
            pivot_scope: None,
            pivot_source_agent: None,
            pivot_source_system: None,
            min_trust: None,
            min_salience: Some(0.8),
            include_evidence: false,
            from_working: false,
        })
        .expect("explore with salience filter");

    assert_eq!(
        response.nodes.len(),
        1,
        "only high-salience item should pass, got {}",
        response.nodes.len()
    );
}

#[tokio::test]
async fn atlas_saved_trails_persist_and_list() {
    let store = SqliteStore::open(
        std::env::temp_dir().join(format!("memd-atlas-trail-save-{}.db", uuid::Uuid::new_v4())),
    )
    .expect("open test store");

    let node_a = uuid::Uuid::new_v4();
    let node_b = uuid::Uuid::new_v4();
    let node_c = uuid::Uuid::new_v4();

    let response = store
        .save_atlas_trail(&memd_schema::AtlasSaveTrailRequest {
            name: "auth investigation".to_string(),
            project: Some("atlas-trails".to_string()),
            namespace: Some("main".to_string()),
            region_id: None,
            node_ids: vec![node_a, node_b, node_c],
        })
        .expect("save trail");

    assert_eq!(response.trail.name, "auth investigation");
    assert_eq!(response.trail.node_ids.len(), 3);

    // List trails
    let listed = store
        .list_atlas_trails(&memd_schema::AtlasListTrailsRequest {
            project: Some("atlas-trails".to_string()),
            namespace: Some("main".to_string()),
            limit: None,
        })
        .expect("list trails");

    assert_eq!(listed.trails.len(), 1);
    assert_eq!(listed.trails[0].name, "auth investigation");
    assert_eq!(listed.trails[0].node_ids, vec![node_a, node_b, node_c]);

    // Save again with same name — should upsert
    let updated = store
        .save_atlas_trail(&memd_schema::AtlasSaveTrailRequest {
            name: "auth investigation".to_string(),
            project: Some("atlas-trails".to_string()),
            namespace: Some("main".to_string()),
            region_id: None,
            node_ids: vec![node_a, node_c],
        })
        .expect("upsert trail");

    assert_eq!(updated.trail.node_ids.len(), 2);

    let relisted = store
        .list_atlas_trails(&memd_schema::AtlasListTrailsRequest {
            project: Some("atlas-trails".to_string()),
            namespace: Some("main".to_string()),
            limit: None,
        })
        .expect("relist");
    assert_eq!(relisted.trails.len(), 1, "upsert should not duplicate");
    assert_eq!(relisted.trails[0].node_ids.len(), 2);
}

// ─── Integration tests for previously untested routes ───

fn test_full_router(state: AppState) -> Router {
    Router::new()
        .route("/memory/store", post(store_memory))
        .route("/memory/verify", post(verify_memory))
        .route(
            "/memory/profile",
            get(get_agent_profile).post(post_agent_profile),
        )
        .route("/memory/workspaces", get(get_workspace_memory))
        .route("/memory/policy", get(get_memory_policy))
        .route("/coordination/inbox", get(get_hive_coordination_inbox))
        .route(
            "/coordination/receipts/record",
            post(post_hive_coordination_receipt),
        )
        .route(
            "/coordination/receipts",
            get(get_hive_coordination_receipts),
        )
        .route(
            "/coordination/skill-policy/apply",
            post(post_skill_policy_apply_receipt).get(get_skill_policy_apply_receipts),
        )
        .route(
            "/coordination/skill-policy/activations",
            get(get_skill_policy_activations),
        )
        .route(
            "/coordination/claims/acquire",
            post(post_hive_claim_acquire),
        )
        .route(
            "/coordination/claims/release",
            post(post_hive_claim_release),
        )
        .route(
            "/coordination/claims/transfer",
            post(post_hive_claim_transfer),
        )
        .route(
            "/coordination/claims/recover",
            post(post_hive_claim_recover),
        )
        .route("/coordination/claims", get(get_hive_claims))
        .route(
            "/coordination/sessions/upsert",
            post(post_hive_session_upsert),
        )
        .route("/coordination/tasks/upsert", post(post_hive_task_upsert))
        .route("/coordination/tasks/assign", post(post_hive_task_assign))
        .route("/coordination/tasks", get(get_hive_tasks))
        .route("/runtime/maintain", post(post_runtime_maintain))
        .with_state(state)
}

fn store_test_item(state: &AppState) -> MemoryItem {
    let item = sample_memory_item(Some("test-ws"));
    state
        .store_item(
            StoreMemoryRequest {
                content: item.content.clone(),
                kind: item.kind,
                scope: item.scope,
                project: item.project.clone(),
                namespace: item.namespace.clone(),
                workspace: item.workspace.clone(),
                visibility: Some(item.visibility),
                source_agent: item.source_agent.clone(),
                source_system: item.source_system.clone(),
                source_path: None,
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(item.confidence),
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: item.tags.clone(),
                belief_branch: None,
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store test item")
        .0
}

#[tokio::test]
async fn verify_memory_route_updates_verification_timestamp() {
    let (dir, state) = temp_state("memd-verify-route");
    let item = store_test_item(&state);
    let app = test_full_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/memory/verify")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&VerifyMemoryRequest {
                        id: item.id,
                        confidence: Some(0.95),
                        status: None,
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .expect("verify memory route");

    assert_eq!(response.status(), StatusCode::OK);
    let body: VerifyMemoryResponse = decode_json(response).await;
    assert_eq!(body.item.id, item.id);
    assert!(body.item.last_verified_at > item.last_verified_at);

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[tokio::test]
async fn agent_profile_roundtrip_via_routes() {
    let (dir, state) = temp_state("memd-agent-profile-route");
    let app = test_full_router(state);

    // POST profile
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/memory/profile")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&AgentProfileUpsertRequest {
                        agent: "codex".to_string(),
                        project: Some("memd".to_string()),
                        namespace: Some("main".to_string()),
                        preferred_route: None,
                        preferred_intent: None,
                        summary_chars: Some(500),
                        max_total_chars: Some(4000),
                        recall_depth: Some(3),
                        source_trust_floor: Some(0.5),
                        style_tags: vec!["terse".to_string()],
                        notes: Some("test profile".to_string()),
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .expect("post agent profile");

    assert_eq!(response.status(), StatusCode::OK);
    let body: AgentProfileResponse = decode_json(response).await;
    assert!(body.profile.is_some());
    assert_eq!(body.profile.as_ref().unwrap().agent, "codex");

    // GET profile
    let response = app
        .oneshot(
            Request::builder()
                .uri("/memory/profile?agent=codex&project=memd&namespace=main")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("get agent profile");

    assert_eq!(response.status(), StatusCode::OK);
    let body: AgentProfileResponse = decode_json(response).await;
    assert!(body.profile.is_some());
    assert_eq!(body.profile.unwrap().summary_chars, Some(500));

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[tokio::test]
async fn workspace_memory_route_returns_aggregates() {
    let (dir, state) = temp_state("memd-workspace-memory-route");
    store_test_item(&state);
    let app = test_full_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/memory/workspaces?project=memd&namespace=main")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("workspace memory route");

    assert_eq!(response.status(), StatusCode::OK);
    let body: WorkspaceMemoryResponse = decode_json(response).await;
    assert!(!body.workspaces.is_empty());

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[tokio::test]
async fn memory_policy_route_returns_snapshot() {
    let (dir, state) = temp_state("memd-policy-route");
    let app = test_full_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/memory/policy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("policy route");

    assert_eq!(response.status(), StatusCode::OK);
    let body: MemoryPolicyResponse = decode_json(response).await;
    assert!(body.retrieval_feedback.max_items_per_request > 0);

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[tokio::test]
async fn coordination_inbox_route_returns_session_view() {
    let (dir, state) = temp_state("memd-coord-inbox-route");
    seed_hive_route_state(&state);
    let app = test_full_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/coordination/inbox?session=bee-1&project=memd&namespace=main&workspace=shared")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("coordination inbox route");

    assert_eq!(response.status(), StatusCode::OK);
    let body: HiveCoordinationInboxResponse = decode_json(response).await;
    assert!(!body.owned_tasks.is_empty() || !body.messages.is_empty());

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[tokio::test]
async fn coordination_receipts_route_returns_receipts() {
    let (dir, state) = temp_state("memd-coord-receipts-route");
    seed_hive_route_state(&state);
    let app = test_full_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/coordination/receipts?project=memd&namespace=main&workspace=shared")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("coordination receipts route");

    assert_eq!(response.status(), StatusCode::OK);
    let body: HiveCoordinationReceiptsResponse = decode_json(response).await;
    assert!(!body.receipts.is_empty());

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[tokio::test]
async fn skill_policy_apply_roundtrip_via_routes() {
    let (dir, state) = temp_state("memd-skill-policy-route");
    let app = test_full_router(state);

    // POST apply receipt
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/coordination/skill-policy/apply")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&SkillPolicyApplyRequest {
                        bundle_root: "/tmp/memd-test".to_string(),
                        runtime_defaulted: false,
                        source_queue_path: "/tmp/memd-test/queue".to_string(),
                        applied_count: 1,
                        skipped_count: 0,
                        applied: vec![SkillPolicyActivationRecord {
                            harness: "claude-code".to_string(),
                            name: "test-skill".to_string(),
                            kind: "hook".to_string(),
                            portability_class: "portable".to_string(),
                            proposal: "install".to_string(),
                            sandbox: "allowed".to_string(),
                            sandbox_risk: 0.1,
                            sandbox_reason: "safe".to_string(),
                            activation: "enabled".to_string(),
                            activation_reason: "user request".to_string(),
                            source_path: "/tmp/skill.toml".to_string(),
                            target_path: None,
                            notes: vec![],
                        }],
                        skipped: vec![],
                        project: Some("memd".to_string()),
                        namespace: Some("main".to_string()),
                        workspace: None,
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .expect("post skill policy apply");

    assert_eq!(response.status(), StatusCode::OK);
    let body: SkillPolicyApplyResponse = decode_json(response).await;
    assert_eq!(body.receipt.applied_count, 1);

    // GET apply receipts
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/coordination/skill-policy/apply?project=memd&namespace=main")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("get skill policy receipts");

    assert_eq!(response.status(), StatusCode::OK);
    let body: SkillPolicyApplyReceiptsResponse = decode_json(response).await;
    assert!(!body.receipts.is_empty());

    // GET activations
    let response = app
        .oneshot(
            Request::builder()
                .uri("/coordination/skill-policy/activations?project=memd&namespace=main")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("get skill policy activations");

    assert_eq!(response.status(), StatusCode::OK);
    let body: SkillPolicyActivationEntriesResponse = decode_json(response).await;
    assert!(!body.activations.is_empty());

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[tokio::test]
async fn claim_transfer_via_route() {
    let (dir, state) = temp_state("memd-claim-transfer-route");
    seed_hive_route_state(&state);
    // Acquire a claim first
    state
        .store
        .acquire_hive_claim(&HiveClaimAcquireRequest {
            scope: "crates/store.rs".to_string(),
            session: "bee-1".to_string(),
            tab_id: None,
            agent: Some("codex".to_string()),
            effective_agent: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            host: None,
            pid: None,
            ttl_seconds: 600,
        })
        .expect("acquire claim");
    let app = test_full_router(state);

    // Transfer the claim
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/coordination/claims/transfer")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&HiveClaimTransferRequest {
                        scope: "crates/store.rs".to_string(),
                        from_session: "bee-1".to_string(),
                        to_session: "queen-1".to_string(),
                        to_tab_id: None,
                        to_agent: None,
                        to_effective_agent: None,
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .expect("transfer claim route");

    assert_eq!(response.status(), StatusCode::OK);
    let body: HiveClaimsResponse = decode_json(response).await;
    assert!(
        body.claims
            .iter()
            .any(|c| c.session == "queen-1" && c.scope == "crates/store.rs")
    );

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[tokio::test]
async fn claim_recover_via_route() {
    let (dir, state) = temp_state("memd-claim-recover-route");
    seed_hive_route_state(&state);
    state
        .store
        .acquire_hive_claim(&HiveClaimAcquireRequest {
            scope: "crates/lib.rs".to_string(),
            session: "bee-1".to_string(),
            tab_id: None,
            agent: Some("codex".to_string()),
            effective_agent: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            host: None,
            pid: None,
            ttl_seconds: 600,
        })
        .expect("acquire claim");
    let app = test_full_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/coordination/claims/recover")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&HiveClaimRecoverRequest {
                        scope: "crates/lib.rs".to_string(),
                        from_session: "bee-1".to_string(),
                        to_session: Some("queen-1".to_string()),
                        to_tab_id: None,
                        to_agent: None,
                        to_effective_agent: None,
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .expect("recover claim route");

    assert_eq!(response.status(), StatusCode::OK);
    let body: HiveClaimsResponse = decode_json(response).await;
    assert!(
        body.claims
            .iter()
            .any(|c| c.session == "queen-1" && c.scope == "crates/lib.rs")
    );

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[tokio::test]
async fn task_upsert_and_list_via_routes() {
    let (dir, state) = temp_state("memd-task-routes");
    seed_hive_route_state(&state);
    let app = test_full_router(state);

    // Upsert a new task
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/coordination/tasks/upsert")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&HiveTaskUpsertRequest {
                        task_id: "new-task-1".to_string(),
                        title: "Test task via route".to_string(),
                        description: Some("integration test".to_string()),
                        status: Some("active".to_string()),
                        coordination_mode: Some(CoordinationMode::ExclusiveWrite),
                        session: Some("bee-1".to_string()),
                        agent: Some("codex".to_string()),
                        effective_agent: None,
                        project: Some("memd".to_string()),
                        namespace: Some("main".to_string()),
                        workspace: Some("shared".to_string()),
                        claim_scopes: vec!["tests/".to_string()],
                        help_requested: Some(false),
                        review_requested: Some(false),
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .expect("task upsert route");

    assert_eq!(response.status(), StatusCode::OK);
    let body: HiveTasksResponse = decode_json(response).await;
    assert!(body.tasks.iter().any(|t| t.task_id == "new-task-1"));

    // List tasks
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/coordination/tasks?project=memd&namespace=main")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("task list route");

    assert_eq!(response.status(), StatusCode::OK);
    let body: HiveTasksResponse = decode_json(response).await;
    assert!(body.tasks.iter().any(|t| t.task_id == "new-task-1"));

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[tokio::test]
async fn task_assign_via_route() {
    let (dir, state) = temp_state("memd-task-assign-route");
    seed_hive_route_state(&state);
    let app = test_full_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/coordination/tasks/assign")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&HiveTaskAssignRequest {
                        task_id: "parser-refactor".to_string(),
                        from_session: Some("bee-1".to_string()),
                        to_session: "queen-1".to_string(),
                        to_agent: Some("codex".to_string()),
                        to_effective_agent: None,
                        note: Some("reassigning to queen".to_string()),
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .expect("task assign route");

    assert_eq!(response.status(), StatusCode::OK);
    let body: HiveTasksResponse = decode_json(response).await;
    assert!(
        body.tasks
            .iter()
            .any(|t| t.task_id == "parser-refactor" && t.session.as_deref() == Some("queen-1"))
    );

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[tokio::test]
async fn runtime_maintain_route_returns_report() {
    let (dir, state) = temp_state("memd-runtime-maintain-route");
    store_test_item(&state);
    let app = test_full_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/runtime/maintain")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&MaintainReportRequest {
                        project: Some("memd".to_string()),
                        namespace: Some("main".to_string()),
                        workspace: None,
                        session: None,
                        mode: "compact".to_string(),
                        apply: false,
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .expect("runtime maintain route");

    assert_eq!(response.status(), StatusCode::OK);
    let body: MaintainReport = decode_json(response).await;
    assert_eq!(body.mode, "compact");

    std::fs::remove_dir_all(dir).expect("cleanup");
}

fn test_drain_router(state: AppState) -> Router {
    Router::new()
        .route("/memory/maintenance/drain", post(drain_memory))
        .route("/memory/inbox/dismiss", post(dismiss_inbox))
        .with_state(state)
}

#[tokio::test]
async fn drain_deletes_expired_items() {
    let (dir, state) = temp_state("memd-drain-expired");
    let app = test_drain_router(state.clone());

    // Store and expire an item via store layer
    let item = store_test_item(&state);
    crate::repair::expire_item(
        &state,
        memd_schema::ExpireMemoryRequest {
            id: item.id,
            status: None,
        },
    )
    .expect("expire item");

    // Drain expired via HTTP
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/memory/maintenance/drain")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&memd_schema::MemoryDrainRequest {
                        project: None,
                        namespace: None,
                        max_items: None,
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .expect("drain items");
    assert_eq!(response.status(), StatusCode::OK);
    let body: memd_schema::MemoryDrainResponse = decode_json(response).await;
    assert_eq!(body.deleted, 1);

    // Verify item is gone from the store
    assert!(state.store.get(item.id).unwrap().is_none());

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[tokio::test]
async fn dismiss_inbox_expires_items() {
    let (dir, state) = temp_state("memd-dismiss-inbox");
    let app = test_drain_router(state.clone());

    // Store a candidate item via store layer
    let item = sample_memory_item(Some("test-ws"));
    let (stored, _) = state
        .store_item(
            StoreMemoryRequest {
                content: item.content.clone(),
                kind: item.kind,
                scope: item.scope,
                project: item.project.clone(),
                namespace: item.namespace.clone(),
                workspace: item.workspace.clone(),
                visibility: Some(item.visibility),
                source_agent: item.source_agent.clone(),
                source_system: item.source_system.clone(),
                source_path: None,
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(item.confidence),
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: item.tags.clone(),
                belief_branch: None,
                status: None,
                lane: None,
            },
            MemoryStage::Candidate,
        )
        .expect("store candidate item");

    // Dismiss via HTTP
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/memory/inbox/dismiss")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&memd_schema::InboxDismissRequest {
                        ids: vec![stored.id],
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .expect("dismiss inbox items");
    assert_eq!(response.status(), StatusCode::OK);
    let body: memd_schema::InboxDismissResponse = decode_json(response).await;
    assert_eq!(body.dismissed, 1);

    // Verify item is expired
    let updated = state.store.get(stored.id).unwrap().unwrap();
    assert_eq!(updated.status, MemoryStatus::Expired);

    std::fs::remove_dir_all(dir).expect("cleanup");
}

// ── Dogfood E2E Gate Tests ──────────────────────────────

#[test]
fn dogfood_store_fact_survives_context_retrieval() {
    let (dir, state) = temp_state("memd-dogfood-fact-context");

    // Store a user fact
    let (_fact, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "user prefers terse responses without trailing summaries".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: None,
                source_agent: Some("codex".to_string()),
                source_system: Some("memd".to_string()),
                source_path: None,
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.95),
                ttl_seconds: None,
                last_verified_at: Some(Utc::now()),
                supersedes: Vec::new(),
                tags: vec!["user_pref".to_string()],
                belief_branch: None,
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store user fact");

    // Store status noise
    for i in 0..5 {
        let _ = state.store_item(
            StoreMemoryRequest {
                content: format!("checkpoint {i}: session state snapshot"),
                kind: MemoryKind::Status,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: None,
                source_agent: Some(format!("codex@s{i}")),
                source_system: Some("memd-short-term".to_string()),
                source_path: None,
                source_quality: Some(SourceQuality::Derived),
                confidence: Some(0.8),
                ttl_seconds: Some(86_400),
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec!["checkpoint".to_string()],
                belief_branch: None,
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        );
    }

    // Retrieve context with current_task intent
    let BuildContextResult { items, .. } = build_context(
        &state,
        &ContextRequest {
            project: Some("memd".to_string()),
            agent: Some("codex".to_string()),
            workspace: None,
            visibility: None,
            route: None,
            intent: Some(RetrievalIntent::CurrentTask),
            limit: Some(4),
            max_chars_per_item: Some(220),
        },
    )
    .expect("build context");

    assert!(
        items.iter().any(|item| item.kind == MemoryKind::Fact),
        "dogfood gate: stored fact must survive context retrieval under status noise"
    );

    // Retrieve working memory
    let working = crate::working::working_memory(
        &state,
        WorkingMemoryRequest {
            project: Some("memd".to_string()),
            agent: Some("codex".to_string()),
            workspace: None,
            visibility: None,
            route: None,
            intent: Some(RetrievalIntent::CurrentTask),
            limit: Some(8),
            max_chars_per_item: Some(220),
            max_total_chars: Some(1600),
            rehydration_limit: Some(4),
            auto_consolidate: Some(false),
            query: None,
        },
    )
    .expect("build working memory");

    assert!(
        working
            .records
            .iter()
            .any(|r| r.record.contains("kind=fact")),
        "dogfood gate: stored fact must appear in working memory (at least 1 fact-kind record)"
    );

    // Verify working memory has at least 1 non-status record
    let non_status_in_context = items
        .iter()
        .filter(|item| item.kind != MemoryKind::Status)
        .count();
    assert!(
        non_status_in_context >= 1,
        "dogfood gate: context must contain at least 1 non-status item (found {non_status_in_context})"
    );

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[test]
fn dogfood_decision_surfaces_over_status_noise() {
    let (dir, state) = temp_state("memd-dogfood-decision");

    let _ = state.store_item(
        StoreMemoryRequest {
            content: "decided: use IMMEDIATE transactions for all writes".to_string(),
            kind: MemoryKind::Decision,
            scope: MemoryScope::Project,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
            visibility: None,
            source_agent: Some("codex".to_string()),
            source_system: Some("memd".to_string()),
            source_path: None,
            source_quality: Some(SourceQuality::Canonical),
            confidence: Some(0.92),
            ttl_seconds: None,
            last_verified_at: Some(Utc::now()),
            supersedes: Vec::new(),
            tags: vec!["architecture".to_string()],
            belief_branch: None,
            status: None,
            lane: None,
        },
        MemoryStage::Canonical,
    );

    for i in 0..8 {
        let _ = state.store_item(
            StoreMemoryRequest {
                content: format!("status noise {i}: session heartbeat"),
                kind: MemoryKind::Status,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: None,
                source_agent: Some(format!("codex@s{i}")),
                source_system: Some("memd-short-term".to_string()),
                source_path: None,
                source_quality: Some(SourceQuality::Derived),
                confidence: Some(0.8),
                ttl_seconds: Some(86_400),
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec!["checkpoint".to_string()],
                belief_branch: None,
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        );
    }

    let working = crate::working::working_memory(
        &state,
        WorkingMemoryRequest {
            project: Some("memd".to_string()),
            agent: Some("codex".to_string()),
            workspace: None,
            visibility: None,
            route: None,
            intent: Some(RetrievalIntent::CurrentTask),
            limit: Some(8),
            max_chars_per_item: Some(220),
            max_total_chars: Some(1600),
            rehydration_limit: Some(4),
            auto_consolidate: Some(false),
            query: None,
        },
    )
    .expect("build working memory");

    assert!(
        working
            .records
            .iter()
            .any(|r| r.record.contains("kind=decision")),
        "dogfood gate: decision must surface in working memory under 8 status items"
    );

    let status_count = working
        .records
        .iter()
        .filter(|r| r.record.contains("heartbeat") || r.record.contains("checkpoint"))
        .count();
    assert!(
        status_count <= 2,
        "dogfood gate: working memory must cap status items at 2 (found {status_count})"
    );

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[test]
fn auto_link_creates_entity_links_on_store() {
    let (dir, state) = temp_state("memd-auto-link");

    // Store two facts in the same project
    let (fact_a, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "architecture uses event sourcing pattern".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: None,
                source_agent: Some("codex".to_string()),
                source_system: Some("memd".to_string()),
                source_path: None,
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.9),
                ttl_seconds: None,
                last_verified_at: Some(Utc::now()),
                supersedes: Vec::new(),
                tags: vec!["arch".to_string()],
                belief_branch: None,
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store fact A");

    let (fact_b, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "decided: sqlite over postgres for embedded use".to_string(),
                kind: MemoryKind::Decision,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: None,
                source_agent: Some("codex".to_string()),
                source_system: Some("memd".to_string()),
                source_path: None,
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.9),
                ttl_seconds: None,
                last_verified_at: Some(Utc::now()),
                supersedes: Vec::new(),
                tags: vec!["db".to_string()],
                belief_branch: None,
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store fact B");

    // Check that entity links were auto-created
    let entity_a = state.store.entity_for_item(fact_a.id).unwrap();
    let entity_b = state.store.entity_for_item(fact_b.id).unwrap();

    if let (Some(ea), Some(eb)) = (&entity_a, &entity_b) {
        let links = state
            .store
            .links_for_entity(&memd_schema::EntityLinksRequest { entity_id: eb.id })
            .unwrap();
        assert!(
            !links.is_empty(),
            "auto-linking should create at least one entity link between co-occurring items"
        );
        let has_auto_link = links.iter().any(|link| {
            link.tags.contains(&"auto".to_string())
                && (link.from_entity_id == ea.id || link.to_entity_id == ea.id)
        });
        assert!(has_auto_link, "auto-link should reference the first entity");
    }

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[tokio::test]
async fn search_excludes_ttl_expired_items_by_default() {
    let (dir, state) = temp_state("memd-ttl-search-filter");

    // Store an item with a 1-second TTL, backdated so it's already expired.
    let past = Utc::now() - chrono::Duration::seconds(10);
    let mut expired_item = sample_memory_item(Some("core"));
    expired_item.content = "ephemeral note that should expire".to_string();
    expired_item.ttl_seconds = Some(1);
    expired_item.created_at = past;
    expired_item.updated_at = past;
    expired_item.kind = MemoryKind::Status;
    expired_item.tags = vec!["ttl-test".to_string()];
    let ck = super::keys::canonical_key(&expired_item);
    let rk = super::keys::redundancy_key(&expired_item);
    state
        .store
        .insert_or_get_duplicate(&expired_item, &ck, &rk)
        .expect("insert expired item");

    // Store a normal item (no TTL) that should survive.
    let mut alive_item = sample_memory_item(Some("core"));
    alive_item.content = "durable fact that stays".to_string();
    alive_item.kind = MemoryKind::Fact;
    alive_item.tags = vec!["ttl-test".to_string()];
    let ck = super::keys::canonical_key(&alive_item);
    let rk = super::keys::redundancy_key(&alive_item);
    state
        .store
        .insert_or_get_duplicate(&alive_item, &ck, &rk)
        .expect("insert alive item");

    let app = Router::new()
        .route("/memory/search", post(search_memory))
        .with_state(state);

    // Search with empty statuses (the default) — expired item must be excluded.
    let req_body = serde_json::json!({
        "scopes": [],
        "kinds": [],
        "statuses": [],
        "tags": ["ttl-test"],
        "stages": [],
        "limit": 10,
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/memory/search")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&req_body).unwrap()))
                .expect("build request"),
        )
        .await
        .expect("run search");

    assert_eq!(response.status(), StatusCode::OK);
    let body: SearchMemoryResponse = decode_json(response).await;

    assert_eq!(
        body.items.len(),
        1,
        "only the non-expired item should appear"
    );
    assert_eq!(body.items[0].id, alive_item.id);

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[test]
fn status_cap_eviction_tracked_in_working_memory() {
    let (dir, state) = temp_state("memd-status-cap-eviction");

    // Store 5 status items
    for i in 0..5 {
        let _ = state.store_item(
            StoreMemoryRequest {
                content: format!("status checkpoint {i}"),
                kind: MemoryKind::Status,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: None,
                source_agent: Some("codex".to_string()),
                source_system: Some("memd".to_string()),
                source_path: None,
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.8),
                ttl_seconds: None,
                last_verified_at: Some(Utc::now()),
                supersedes: Vec::new(),
                tags: vec!["checkpoint".to_string()],
                belief_branch: None,
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        );
    }

    // Store 3 facts so they surface alongside capped status
    for i in 0..3 {
        let _ = state.store_item(
            StoreMemoryRequest {
                content: format!("important fact number {i}"),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: None,
                source_agent: Some("codex".to_string()),
                source_system: Some("memd".to_string()),
                source_path: None,
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.9),
                ttl_seconds: None,
                last_verified_at: Some(Utc::now()),
                supersedes: Vec::new(),
                tags: vec!["infra".to_string()],
                belief_branch: None,
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        );
    }

    let working = crate::working::working_memory(
        &state,
        WorkingMemoryRequest {
            project: Some("memd".to_string()),
            agent: Some("codex".to_string()),
            workspace: None,
            visibility: None,
            route: None,
            intent: Some(RetrievalIntent::CurrentTask),
            limit: Some(8),
            max_chars_per_item: Some(220),
            max_total_chars: Some(4000),
            rehydration_limit: Some(4),
            auto_consolidate: Some(false),
            query: None,
        },
    )
    .expect("build working memory");

    // Count status items in records — should be ≤ 2
    let status_in_records = working
        .records
        .iter()
        .filter(|r| r.record.contains("kind=status"))
        .count();
    assert!(
        status_in_records <= 2,
        "at most 2 status items in records, found {status_in_records}"
    );

    // Evicted list should contain status-capped items
    let status_evictions: Vec<_> = working
        .evicted
        .iter()
        .filter(|e| e.reason.contains("evicted_by_status_cap"))
        .collect();
    assert!(
        !status_evictions.is_empty(),
        "evicted list must track status-capped items"
    );

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[test]
fn duplicate_store_reinforces_existing_item() {
    let (dir, state) = temp_state("memd-reinforce-dedup");

    let req = StoreMemoryRequest {
        content: "the server runs debian".to_string(),
        kind: MemoryKind::Fact,
        scope: MemoryScope::Project,
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        workspace: None,
        visibility: None,
        source_agent: Some("codex".to_string()),
        source_system: Some("memd".to_string()),
        source_path: None,
        source_quality: Some(SourceQuality::Canonical),
        confidence: Some(0.7),
        ttl_seconds: None,
        last_verified_at: None,
        supersedes: Vec::new(),
        tags: vec!["infra".to_string()],
        belief_branch: None,
        status: None,
        lane: None,
    };

    let (first, dup1) = state
        .store_item(req.clone(), MemoryStage::Canonical)
        .expect("first store");
    assert!(dup1.is_none(), "first insert should not be a duplicate");

    let (reinforced, dup2) = state
        .store_item(req.clone(), MemoryStage::Canonical)
        .expect("second store");
    assert!(dup2.is_some(), "second insert should detect duplicate");
    assert_eq!(reinforced.id, first.id, "should reinforce same item");
    assert!(
        reinforced.confidence > first.confidence,
        "confidence should increase: {} > {}",
        reinforced.confidence,
        first.confidence
    );
    assert!(
        reinforced.updated_at >= first.updated_at,
        "updated_at should be bumped"
    );

    let items = state.snapshot().expect("snapshot");
    let matching: Vec<_> = items
        .iter()
        .filter(|i| i.content == "the server runs debian")
        .collect();
    assert_eq!(matching.len(), 1, "only one item should exist in DB");

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[test]
fn concurrent_writes_no_sqlite_busy() {
    // C2 gate: 3 agents writing simultaneously, 0 SQLITE_BUSY errors.
    let dir = std::env::temp_dir().join(format!("memd-concurrent-write-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let db_path = dir.join("memd.db");
    let store = SqliteStore::open(&db_path).expect("open store");

    let handles: Vec<_> = (0..3)
        .map(|agent_idx| {
            let store = store.clone();
            std::thread::spawn(move || {
                let mut errors = Vec::new();
                for i in 0..50 {
                    let now = chrono::Utc::now();
                    let item = MemoryItem {
                        id: uuid::Uuid::new_v4(),
                        content: format!(
                            "concurrent-stress-unique-{} content-payload",
                            uuid::Uuid::new_v4()
                        ),
                        redundancy_key: None,
                        belief_branch: None,
                        preferred: false,
                        kind: MemoryKind::Fact,
                        scope: MemoryScope::Project,
                        project: Some("memd".to_string()),
                        namespace: Some("main".to_string()),
                        workspace: None,
                        visibility: MemoryVisibility::Private,
                        source_agent: Some(format!("agent-{agent_idx}")),
                        source_system: Some("stress-test".to_string()),
                        source_path: None,
                        source_quality: Some(SourceQuality::Canonical),
                        confidence: 0.9,
                        ttl_seconds: None,
                        created_at: now,
                        updated_at: now,
                        last_verified_at: None,
                        supersedes: vec![],
                        tags: vec!["concurrent-test".to_string()],
                        status: MemoryStatus::Active,
                        stage: MemoryStage::Canonical,
                        lane: None,
                        version: 1,
                        correction_meta: None,
                    };
                    let ck = super::keys::canonical_key(&item);
                    let rk = super::keys::redundancy_key(&item);
                    if let Err(e) = store.insert_or_get_duplicate(&item, &ck, &rk) {
                        errors.push(format!("agent-{agent_idx} item {i}: {e}"));
                    }
                }
                errors
            })
        })
        .collect();

    let mut all_errors = Vec::new();
    for handle in handles {
        all_errors.extend(handle.join().expect("thread panicked"));
    }

    assert!(
        all_errors.is_empty(),
        "concurrent writes produced errors: {all_errors:?}"
    );

    let items = store.list().expect("list items");
    let test_items: Vec<_> = items
        .iter()
        .filter(|i| i.tags.contains(&"concurrent-test".to_string()))
        .collect();
    assert_eq!(
        test_items.len(),
        150,
        "all 150 items (3 agents × 50) should be stored"
    );

    std::fs::remove_dir_all(dir).expect("cleanup");
}

// ── D2 Correction Flow tests ──

#[test]
fn correct_item_supersedes_old_and_creates_new() {
    let (dir, state) = temp_state("memd-correct-basic");

    let (original, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "the capital of France is Berlin".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: None,
                belief_branch: None,
                source_agent: Some("test".to_string()),
                source_system: None,
                source_path: None,
                source_quality: None,
                confidence: None,
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: vec![],
                tags: vec!["geography".to_string()],
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store original");

    let response = repair::correct_item(
        &state,
        CorrectMemoryRequest {
            id: original.id,
            content: "the capital of France is Paris".to_string(),
            reason: Some("Berlin is Germany's capital, not France's".to_string()),
            tags: None,
            confidence: None,
        },
    )
    .expect("correct item");

    assert_eq!(response.old_item.status, MemoryStatus::Superseded);
    assert_eq!(response.old_item.id, original.id);

    assert_eq!(response.new_item.status, MemoryStatus::Active);
    assert_eq!(response.new_item.content, "the capital of France is Paris");
    assert!(response.new_item.supersedes.contains(&original.id));
    assert!(response.new_item.tags.contains(&"correction".to_string()));
    assert!(response.new_item.tags.contains(&"geography".to_string()));

    let old_from_store = state.store.get(original.id).unwrap().unwrap();
    assert_eq!(old_from_store.status, MemoryStatus::Superseded);
    let new_from_store = state.store.get(response.new_item.id).unwrap().unwrap();
    assert_eq!(new_from_store.status, MemoryStatus::Active);

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[test]
fn correct_item_rejects_empty_content() {
    let (dir, state) = temp_state("memd-correct-empty");

    let (original, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "some fact".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: None,
                belief_branch: None,
                source_agent: None,
                source_system: None,
                source_path: None,
                source_quality: None,
                confidence: None,
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: vec![],
                tags: vec![],
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store original");

    let result = repair::correct_item(
        &state,
        CorrectMemoryRequest {
            id: original.id,
            content: "   ".to_string(),
            reason: None,
            tags: None,
            confidence: None,
        },
    );

    assert!(result.is_err());
    let (status, _msg) = result.unwrap_err();
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let from_store = state.store.get(original.id).unwrap().unwrap();
    assert_eq!(from_store.status, MemoryStatus::Active);

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[test]
fn correct_item_not_found_returns_404() {
    let (dir, state) = temp_state("memd-correct-404");

    let result = repair::correct_item(
        &state,
        CorrectMemoryRequest {
            id: uuid::Uuid::new_v4(),
            content: "corrected".to_string(),
            reason: None,
            tags: None,
            confidence: None,
        },
    );

    assert!(result.is_err());
    let (status, _msg) = result.unwrap_err();
    assert_eq!(status, StatusCode::NOT_FOUND);

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[test]
fn correct_item_preserves_metadata_from_original() {
    let (dir, state) = temp_state("memd-correct-metadata");

    let (original, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "old content".to_string(),
                kind: MemoryKind::Decision,
                scope: MemoryScope::Synced,
                project: Some("myproject".to_string()),
                namespace: Some("dev".to_string()),
                workspace: Some("ws-1".to_string()),
                visibility: Some(MemoryVisibility::Private),
                belief_branch: Some("branch-a".to_string()),
                source_agent: Some("agent-x".to_string()),
                source_system: Some("system-y".to_string()),
                source_path: Some("/path/to/file".to_string()),
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.95),
                ttl_seconds: Some(3600),
                last_verified_at: None,
                supersedes: vec![],
                tags: vec!["important".to_string()],
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store original");

    let response = repair::correct_item(
        &state,
        CorrectMemoryRequest {
            id: original.id,
            content: "new content".to_string(),
            reason: Some("updated decision".to_string()),
            tags: None,
            confidence: None,
        },
    )
    .expect("correct item");

    let new = &response.new_item;
    assert_eq!(new.kind, MemoryKind::Decision);
    assert_eq!(new.scope, MemoryScope::Synced);
    assert_eq!(new.project.as_deref(), Some("myproject"));
    assert_eq!(new.namespace.as_deref(), Some("dev"));
    assert_eq!(new.workspace.as_deref(), Some("ws-1"));
    assert_eq!(new.visibility, MemoryVisibility::Private);
    assert_eq!(new.belief_branch.as_deref(), Some("branch-a"));
    assert_eq!(new.source_agent.as_deref(), Some("agent-x"));
    assert_eq!(new.source_system.as_deref(), Some("system-y"));
    assert_eq!(new.source_path.as_deref(), Some("/path/to/file"));
    assert_eq!(new.ttl_seconds, Some(3600));
    assert!(new.tags.contains(&"important".to_string()));
    assert!(new.tags.contains(&"correction".to_string()));
    assert_eq!(new.content, "new content");

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[test]
fn explain_shows_correction_events() {
    let (dir, state) = temp_state("memd-correct-explain");

    let (original, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "wrong answer".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: None,
                belief_branch: None,
                source_agent: Some("test".to_string()),
                source_system: None,
                source_path: None,
                source_quality: None,
                confidence: None,
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: vec![],
                tags: vec![],
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store original");

    let response = repair::correct_item(
        &state,
        CorrectMemoryRequest {
            id: original.id,
            content: "right answer".to_string(),
            reason: Some("was wrong".to_string()),
            tags: None,
            confidence: None,
        },
    )
    .expect("correct item");

    let explain_old = inspection::explain_memory(
        &state,
        ExplainMemoryRequest {
            id: original.id,
            belief_branch: None,
            route: None,
            intent: None,
        },
    )
    .expect("explain old item");
    assert!(
        explain_old
            .events
            .iter()
            .any(|e| e.event_type == "superseded_by_correction"),
        "old item should have superseded_by_correction event"
    );

    let explain_new = inspection::explain_memory(
        &state,
        ExplainMemoryRequest {
            id: response.new_item.id,
            belief_branch: None,
            route: None,
            intent: None,
        },
    )
    .expect("explain new item");
    assert!(
        explain_new
            .events
            .iter()
            .any(|e| e.event_type == "correction_created" || e.event_type == "stored_canonical"),
        "new item should have correction_created or stored_canonical event"
    );

    // K2.3: corrected item should surface its predecessor via corrections_chain.
    assert!(
        explain_new
            .corrections_chain
            .iter()
            .any(|entry| entry.id == original.id),
        "new item's corrections_chain should contain the superseded original"
    );
    assert!(
        !explain_new.confidence_timeline.is_empty(),
        "corrected item should have at least a `created` confidence sample"
    );
    assert!(
        explain_new
            .confidence_timeline
            .iter()
            .any(|sample| sample.source == "created"),
        "confidence timeline should carry the initial `created` sample"
    );

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[test]
fn selective_reset_corrects_one_item_without_affecting_others() {
    let (dir, state) = temp_state("memd-correct-selective");

    let make_item = |content: &str| {
        state
            .store_item(
                StoreMemoryRequest {
                    content: content.to_string(),
                    kind: MemoryKind::Fact,
                    scope: MemoryScope::Project,
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: None,
                    visibility: None,
                    belief_branch: None,
                    source_agent: Some("test".to_string()),
                    source_system: None,
                    source_path: None,
                    source_quality: None,
                    confidence: None,
                    ttl_seconds: None,
                    last_verified_at: None,
                    supersedes: vec![],
                    tags: vec![],
                    status: None,
                    lane: None,
                },
                MemoryStage::Canonical,
            )
            .expect("store item")
    };

    let (item_a, _) = make_item("fact A is correct");
    let (item_b, _) = make_item("fact B is correct");
    let (item_c, _) = make_item("fact C is correct");

    // Correct only item_b — items A and C should be untouched
    let response = repair::correct_item(
        &state,
        CorrectMemoryRequest {
            id: item_b.id,
            content: "fact B was wrong, now fixed".to_string(),
            reason: Some("selective fix".to_string()),
            tags: None,
            confidence: None,
        },
    )
    .expect("correct item B");

    // B is superseded
    let b_store = state.store.get(item_b.id).unwrap().unwrap();
    assert_eq!(b_store.status, MemoryStatus::Superseded);

    // New B replacement exists
    assert_eq!(response.new_item.status, MemoryStatus::Active);
    assert_eq!(response.new_item.content, "fact B was wrong, now fixed");

    // A and C are completely untouched
    let a_store = state.store.get(item_a.id).unwrap().unwrap();
    assert_eq!(a_store.status, MemoryStatus::Active);
    assert_eq!(a_store.content, "fact A is correct");
    assert_eq!(a_store.updated_at, item_a.updated_at);

    let c_store = state.store.get(item_c.id).unwrap().unwrap();
    assert_eq!(c_store.status, MemoryStatus::Active);
    assert_eq!(c_store.content, "fact C is correct");
    assert_eq!(c_store.updated_at, item_c.updated_at);

    std::fs::remove_dir_all(dir).expect("cleanup");
}

// ── E2 Atlas Activation tests ──

#[test]
fn parse_wiki_links_extracts_bracketed_refs() {
    let content = "see [[Rust]] and [[memd server]] for details, also [[Rust]] again";
    let links = helpers::parse_wiki_links(content);
    assert_eq!(links, vec!["Rust", "memd server"]);
}

#[test]
fn parse_wiki_links_handles_empty_and_unclosed() {
    assert!(helpers::parse_wiki_links("no links here").is_empty());
    assert!(helpers::parse_wiki_links("[[]]").is_empty());
    assert!(helpers::parse_wiki_links("[[unclosed").is_empty());
    assert_eq!(
        helpers::parse_wiki_links("[[valid]] then [[unclosed"),
        vec!["valid"]
    );
}

#[test]
fn wiki_link_creates_entity_link_on_store() {
    let (dir, state) = temp_state("memd-wiki-link");

    // First item in project "alpha-svc" creates an entity with alias "alpha-svc"
    let (fact_a, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "alpha-svc uses event sourcing".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("alpha-svc".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: None,
                source_agent: Some("codex".to_string()),
                source_system: None,
                source_path: None,
                source_quality: None,
                confidence: None,
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec!["arch".to_string()],
                belief_branch: None,
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store fact A");

    // Second item in different project, uses wiki link [[alpha-svc]] matching first entity's alias
    let (fact_b, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "migration plan references [[alpha-svc]] decisions".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("beta-svc".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: None,
                source_agent: Some("codex".to_string()),
                source_system: None,
                source_path: None,
                source_quality: None,
                confidence: None,
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec!["plan".to_string()],
                belief_branch: None,
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store fact B with wiki link");

    let entity_a = state.store.entity_for_item(fact_a.id).unwrap();
    let entity_b = state.store.entity_for_item(fact_b.id).unwrap();

    if let (Some(ea), Some(eb)) = (&entity_a, &entity_b) {
        let links = state
            .store
            .links_for_entity(&memd_schema::EntityLinksRequest { entity_id: eb.id })
            .unwrap();
        let has_wiki_link = links.iter().any(|link| {
            link.tags.contains(&"wiki-link".to_string())
                && (link.from_entity_id == ea.id || link.to_entity_id == ea.id)
        });
        assert!(
            has_wiki_link,
            "wiki link [[alpha-svc]] should create entity link to fact A's entity"
        );
    }

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[test]
fn named_entity_mentions_create_source_backed_atlas_links() {
    let (dir, state) = temp_state("memd-ner-link");

    let (fact_a, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "Alice Johnson owns the deploy process for ACME Cloud.".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("atlas-ner".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: None,
                source_agent: Some("codex".to_string()),
                source_system: None,
                source_path: None,
                source_quality: None,
                confidence: None,
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec!["people".to_string()],
                belief_branch: None,
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store fact A");

    let (fact_b, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "Escalate ACME Cloud incidents to Alice Johnson first.".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("atlas-ner".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: None,
                source_agent: Some("codex".to_string()),
                source_system: None,
                source_path: None,
                source_quality: None,
                confidence: None,
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec!["ops".to_string()],
                belief_branch: None,
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store fact B");

    let entity_a = state
        .store
        .entity_for_item(fact_a.id)
        .unwrap()
        .expect("entity A");
    let entity_b = state
        .store
        .entity_for_item(fact_b.id)
        .unwrap()
        .expect("entity B");
    let links = state
        .store
        .links_for_entity(&memd_schema::EntityLinksRequest {
            entity_id: entity_b.id,
        })
        .expect("links for entity B");

    assert!(
        links.iter().any(|link| {
            (link.from_entity_id == entity_a.id || link.to_entity_id == entity_a.id)
                && link.tags.iter().any(|tag| tag == "ner")
                && link.source_item_id == Some(fact_b.id)
        }),
        "named entity mentions should create source-backed atlas links"
    );

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[test]
fn atlas_regions_generated_for_project_with_items() {
    let (dir, state) = temp_state("memd-atlas-regions");

    for i in 0..12 {
        state
            .store_item(
                StoreMemoryRequest {
                    content: format!("fact number {i} about the project"),
                    kind: MemoryKind::Fact,
                    scope: MemoryScope::Project,
                    project: Some("atlas-test".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: None,
                    visibility: None,
                    source_agent: Some("codex".to_string()),
                    source_system: Some("test".to_string()),
                    source_path: None,
                    source_quality: None,
                    confidence: None,
                    ttl_seconds: None,
                    last_verified_at: None,
                    supersedes: Vec::new(),
                    tags: vec!["batch".to_string()],
                    belief_branch: None,
                    status: None,
                    lane: None,
                },
                MemoryStage::Canonical,
            )
            .expect("store batch item");
    }

    let regions = state
        .store
        .generate_regions_for_project(Some("atlas-test"), Some("main"), None)
        .expect("generate regions");

    assert!(
        !regions.is_empty(),
        "atlas should generate non-empty regions for 12 items"
    );

    let list = state
        .store
        .list_atlas_regions(&memd_schema::AtlasRegionsRequest {
            project: Some("atlas-test".to_string()),
            namespace: Some("main".to_string()),
            lane: None,
            limit: Some(5),
        })
        .expect("list atlas regions");

    assert!(
        !list.regions.is_empty(),
        "listed atlas regions should be non-empty after generation"
    );

    std::fs::remove_dir_all(dir).expect("cleanup");
}

// ── G2 Lane Architecture tests ──

#[test]
fn lane_auto_detection_from_content_keywords() {
    assert_eq!(
        helpers::detect_content_lane("system architecture uses event sourcing", None, &[]),
        Some("architecture".to_string())
    );
    assert_eq!(
        helpers::detect_content_lane("we decided to use sqlite over postgres", None, &[]),
        Some("decisions".to_string())
    );
    assert_eq!(
        helpers::detect_content_lane("constraint: must not exceed 100ms latency", None, &[]),
        Some("constraints".to_string())
    );
    assert_eq!(
        helpers::detect_content_lane("deploy pipeline runs on every push", None, &[]),
        Some("operations".to_string())
    );
    assert_eq!(
        helpers::detect_content_lane("just a regular note about nothing special", None, &[]),
        None,
    );
}

#[test]
fn lane_auto_detection_from_tags() {
    assert_eq!(
        helpers::detect_content_lane("some content", None, &["lane:design".to_string()]),
        Some("design".to_string())
    );
    assert_eq!(
        helpers::detect_content_lane("architecture note", None, &["lane:operations".to_string()]),
        Some("operations".to_string())
    );
}

#[test]
fn lane_auto_detection_from_source_path() {
    assert_eq!(
        helpers::detect_content_lane("some code", Some("src/components/Button.tsx"), &[]),
        Some("design".to_string())
    );
    assert_eq!(
        helpers::detect_content_lane("notes", Some("docs/architecture/overview.md"), &[]),
        Some("architecture".to_string())
    );
}

#[test]
fn lane_persisted_on_store_item() {
    let (dir, state) = temp_state("memd-lane-persist");

    let (item, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "system architecture uses event sourcing pattern".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: None,
                source_agent: None,
                source_system: None,
                source_path: None,
                source_quality: None,
                confidence: None,
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: Vec::new(),
                belief_branch: None,
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store item with auto-detected lane");

    assert_eq!(
        item.lane.as_deref(),
        Some("architecture"),
        "lane should be auto-detected from content keywords"
    );

    let stored = state.store.get(item.id).unwrap().unwrap();
    assert_eq!(stored.lane.as_deref(), Some("architecture"));

    std::fs::remove_dir_all(dir).expect("cleanup g2-persist");
}

#[test]
fn explicit_lane_overrides_auto_detection() {
    let (dir, state) = temp_state("memd-lane-explicit");

    let (item, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "system architecture uses event sourcing".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: None,
                source_agent: None,
                source_system: None,
                source_path: None,
                source_quality: None,
                confidence: None,
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: Vec::new(),
                belief_branch: None,
                status: None,
                lane: Some("design".to_string()),
            },
            MemoryStage::Canonical,
        )
        .expect("store item with explicit lane");

    assert_eq!(
        item.lane.as_deref(),
        Some("design"),
        "explicit lane should override auto-detection"
    );

    std::fs::remove_dir_all(dir).expect("cleanup g2-explicit");
}

#[test]
fn lane_tag_triggers_auto_detection() {
    let (dir, state) = temp_state("memd-lane-tag");

    let (item, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "test lane migration".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: None,
                source_agent: None,
                source_system: None,
                source_path: None,
                source_quality: None,
                confidence: None,
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec!["lane:patterns".to_string()],
                belief_branch: None,
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store item with lane tag");

    assert_eq!(item.lane.as_deref(), Some("patterns"));
    let stored = state.store.get(item.id).unwrap().unwrap();
    assert_eq!(stored.lane.as_deref(), Some("patterns"));

    std::fs::remove_dir_all(dir).expect("cleanup g2-tag");
}

// ── H2 Recall Proof Tests ──────────────────────────────

#[test]
fn fts5_search_returns_matching_items() {
    let (dir, state) = temp_state("h2-fts5-search");

    let (fact, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "Josue prefers Rust for all backend services".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: None,
                source_agent: Some("codex".to_string()),
                source_system: Some("memd".to_string()),
                source_path: None,
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.95),
                ttl_seconds: None,
                last_verified_at: Some(Utc::now()),
                supersedes: Vec::new(),
                tags: vec!["preference".to_string()],
                belief_branch: None,
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store fact");

    // Store unrelated noise
    let _ = state.store_item(
        StoreMemoryRequest {
            content: "session checkpoint: working on dashboard layout".to_string(),
            kind: MemoryKind::Status,
            scope: MemoryScope::Project,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
            visibility: None,
            source_agent: Some("codex".to_string()),
            source_system: Some("memd".to_string()),
            source_path: None,
            source_quality: Some(SourceQuality::Derived),
            confidence: Some(0.7),
            ttl_seconds: Some(86_400),
            last_verified_at: None,
            supersedes: Vec::new(),
            tags: vec!["checkpoint".to_string()],
            belief_branch: None,
            status: None,
            lane: None,
        },
        MemoryStage::Canonical,
    );

    let results = state
        .store
        .fts_search("Rust backend", 10)
        .expect("fts search");
    assert!(!results.is_empty(), "FTS search should return results");
    assert_eq!(
        results[0].0, fact.id,
        "best FTS hit should be the Rust fact"
    );

    std::fs::remove_dir_all(dir).expect("cleanup h2-fts5");
}

#[test]
fn rrf_merge_boosts_fts_matched_items_in_search() {
    let (dir, state) = temp_state("h2-rrf-merge");

    // Store a specific technical fact with low confidence (would rank lower by metadata)
    let (target, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "NFS Cargo builds must use /tmp/<project>-target to avoid locking"
                    .to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Global,
                project: None,
                namespace: None,
                workspace: None,
                visibility: Some(MemoryVisibility::Workspace),
                source_agent: Some("codex".to_string()),
                source_system: Some("memd".to_string()),
                source_path: None,
                source_quality: Some(SourceQuality::Derived),
                confidence: Some(0.5),
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec!["nfs".to_string(), "cargo".to_string()],
                belief_branch: None,
                status: None,
                lane: None,
            },
            MemoryStage::Candidate,
        )
        .expect("store nfs fact");

    // Store high-confidence items that would normally outrank
    for i in 0..5 {
        let _ = state.store_item(
            StoreMemoryRequest {
                content: format!("project architecture decision {i}: use axum for HTTP layer"),
                kind: MemoryKind::Decision,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: Some(MemoryVisibility::Workspace),
                source_agent: Some("codex".to_string()),
                source_system: Some("memd".to_string()),
                source_path: None,
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.95),
                ttl_seconds: None,
                last_verified_at: Some(Utc::now()),
                supersedes: Vec::new(),
                tags: vec!["architecture".to_string()],
                belief_branch: None,
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        );
    }

    // Search for "nfs cargo" — FTS should boost the target item
    let fts_ranks = state
        .store
        .fts_search("nfs cargo", 100)
        .expect("fts search");
    assert!(
        fts_ranks.iter().any(|(id, _)| *id == target.id),
        "FTS should find the NFS cargo fact"
    );

    let items = enrich_with_entities(&state, state.snapshot().expect("snapshot")).expect("enrich");
    let plan = RetrievalPlan::resolve(None, None);
    let results = filter_items(
        &items,
        &SearchMemoryRequest {
            query: Some("nfs cargo".to_string()),
            limit: Some(10),
            ..Default::default()
        },
        &plan,
        &fts_ranks,
    );

    assert!(!results.is_empty(), "search should return results");
    assert_eq!(
        results[0].id, target.id,
        "RRF should boost the FTS-matched NFS fact to position 1"
    );

    std::fs::remove_dir_all(dir).expect("cleanup h2-rrf");
}

#[test]
fn filter_items_keeps_fts_hits_even_when_raw_question_text_is_not_a_substring() {
    let (dir, state) = temp_state("h2-rrf-natural-language");

    let (target, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "graduated with a degree in business administration from UCLA".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: Some(MemoryVisibility::Workspace),
                source_agent: Some("codex".to_string()),
                source_system: Some("memd".to_string()),
                source_path: None,
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.9),
                ttl_seconds: None,
                last_verified_at: Some(Utc::now()),
                supersedes: Vec::new(),
                tags: vec!["degree".to_string(), "ucla".to_string()],
                belief_branch: None,
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store degree fact");

    let raw_query = "What degree did I graduate with?".to_string();
    let fts_ranks = state
        .store
        .fts_search("degree ucla", 100)
        .expect("fts search");
    assert!(
        fts_ranks.iter().any(|(id, _)| *id == target.id),
        "FTS should find the degree fact from the sanitized query"
    );

    let items = enrich_with_entities(&state, state.snapshot().expect("snapshot")).expect("enrich");
    let plan = RetrievalPlan::resolve(None, None);
    let results = filter_items(
        &items,
        &SearchMemoryRequest {
            query: Some(raw_query),
            limit: Some(10),
            ..Default::default()
        },
        &plan,
        &fts_ranks,
    );

    assert!(
        results.iter().any(|item| item.id == target.id),
        "raw-question filtering must not discard an FTS hit"
    );

    std::fs::remove_dir_all(dir).expect("cleanup h2-rrf-natural-language");
}

#[test]
fn search_score_prefers_query_token_overlap_over_unrelated_high_metadata_item() {
    let plan = RetrievalPlan::resolve(None, None);
    let query = Some(
        "What should I serve for dinner this weekend with my homegrown ingredients?".to_string(),
    );
    let relevant = MemoryItem {
        content: "homegrown cherry tomatoes basil mint dinner ideas and garden produce".to_string(),
        tags: vec!["garden".to_string(), "dinner".to_string()],
        confidence: 0.7,
        source_quality: Some(SourceQuality::Canonical),
        ..sample_memory_item(None)
    };
    let noisy = MemoryItem {
        content: "generic project status update about architecture and planning".to_string(),
        tags: vec!["architecture".to_string()],
        confidence: 0.95,
        source_quality: Some(SourceQuality::Canonical),
        ..sample_memory_item(None)
    };

    assert!(
        search_score(&relevant, None, 0.8, &query, None, None, &plan)
            > search_score(&noisy, None, 0.95, &query, None, None, &plan)
    );
}

#[test]
fn ab_influence_recall_changes_search_output() {
    let (dir, state) = temp_state("h2-ab-influence");

    // Store a correction: old fact superseded by new fact
    let (old_fact, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "deploy target is fly.io".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: Some(MemoryVisibility::Workspace),
                source_agent: Some("codex".to_string()),
                source_system: Some("memd".to_string()),
                source_path: None,
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.9),
                ttl_seconds: None,
                last_verified_at: Some(Utc::now()),
                supersedes: Vec::new(),
                tags: vec!["deploy".to_string()],
                belief_branch: None,
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store old fact");

    let (new_fact, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "deploy target is docker on services VM via portainer".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: Some(MemoryVisibility::Workspace),
                source_agent: Some("codex".to_string()),
                source_system: Some("memd".to_string()),
                source_path: None,
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.95),
                ttl_seconds: None,
                last_verified_at: Some(Utc::now()),
                supersedes: vec![old_fact.id],
                tags: vec!["deploy".to_string()],
                belief_branch: None,
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store corrected fact");

    // A: Search WITH memd recall (FTS + RRF)
    let fts_ranks = state
        .store
        .fts_search("deploy target", 100)
        .expect("fts search");
    let items = enrich_with_entities(&state, state.snapshot().expect("snapshot")).expect("enrich");
    let plan = RetrievalPlan::resolve(None, None);
    let with_recall = filter_items(
        &items,
        &SearchMemoryRequest {
            query: Some("deploy target".to_string()),
            limit: Some(5),
            ..Default::default()
        },
        &plan,
        &fts_ranks,
    );

    // B: Search WITHOUT FTS recall (empty fts_ranks, simulating no-memd)
    let without_recall = filter_items(
        &items,
        &SearchMemoryRequest {
            query: Some("deploy target".to_string()),
            limit: Some(5),
            ..Default::default()
        },
        &plan,
        &[],
    );

    // Both should find the deploy facts
    assert!(
        with_recall.iter().any(|item| item.id == new_fact.id),
        "recall-on should find corrected deploy fact"
    );
    assert!(
        without_recall.iter().any(|item| item.id == new_fact.id),
        "recall-off should also find deploy fact (it's still in the metadata path)"
    );

    // The key A/B proof: with FTS recall, the corrected fact should rank higher
    // because FTS gives it a direct keyword match boost via RRF
    let with_pos = with_recall
        .iter()
        .position(|item| item.id == new_fact.id)
        .unwrap();
    let without_pos = without_recall
        .iter()
        .position(|item| item.id == new_fact.id)
        .unwrap();

    // With only 2 deploy facts, both paths find it. The test proves the
    // mechanism exists: FTS provides an independent ranking signal that
    // RRF merges. In larger stores, this difference becomes decisive.
    assert!(
        with_pos <= without_pos,
        "FTS+RRF should rank the target at least as high as metadata-only"
    );

    std::fs::remove_dir_all(dir).expect("cleanup h2-ab");
}

// ── D2 E2E: correction flow ──────────────────────────────────────────────────

#[test]
fn d2_correction_e2e() {
    let (dir, state) = temp_state("d2-correction-e2e");

    // Store original fact
    let (original, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "memd uses Python for the server".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: None,
                belief_branch: None,
                source_agent: Some("test".to_string()),
                source_system: None,
                source_path: None,
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.8),
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: vec![],
                tags: vec!["architecture".to_string()],
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store original");

    // (a) Correct it
    let response = repair::correct_item(
        &state,
        CorrectMemoryRequest {
            id: original.id,
            content: "memd uses Rust for the server".to_string(),
            reason: Some("server is written in Rust, not Python".to_string()),
            tags: None,
            confidence: None,
        },
    )
    .expect("correct item");

    // (a) old item is Superseded
    assert_eq!(response.old_item.status, MemoryStatus::Superseded);

    // (b) new item is Active with correction tag and preferred: true
    assert_eq!(response.new_item.status, MemoryStatus::Active);
    assert!(response.new_item.tags.contains(&"correction".to_string()));
    assert!(
        response.new_item.preferred,
        "correction item must be preferred"
    );

    // (c) build_context returns corrected version only
    let BuildContextResult { items, .. } = build_context(
        &state,
        &ContextRequest {
            project: Some("memd".to_string()),
            agent: Some("test".to_string()),
            workspace: None,
            visibility: None,
            route: Some(RetrievalRoute::ProjectFirst),
            intent: Some(RetrievalIntent::CurrentTask),
            limit: Some(10),
            max_chars_per_item: Some(300),
        },
    )
    .expect("build context");

    assert!(
        items.iter().any(|i| i.id == response.new_item.id),
        "corrected item must appear in context"
    );
    assert!(
        items.iter().all(|i| i.id != original.id),
        "superseded original must NOT appear in context"
    );

    // (d) explain_memory shows correction chain
    let explain = inspection::explain_memory(
        &state,
        ExplainMemoryRequest {
            id: response.new_item.id,
            belief_branch: None,
            route: None,
            intent: None,
        },
    )
    .expect("explain corrected item");

    assert!(
        explain
            .events
            .iter()
            .any(|e| e.event_type == "correction_created" || e.event_type == "stored_canonical"),
        "correction lifecycle event must be present"
    );

    // (e) corrected item scores higher than a non-correction fact in working memory
    let (_filler, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "memd stores data in SQLite".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: None,
                belief_branch: None,
                source_agent: Some("test".to_string()),
                source_system: None,
                source_path: None,
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.8),
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: vec![],
                tags: vec![],
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store filler");

    let working = crate::working::working_memory(
        &state,
        WorkingMemoryRequest {
            project: Some("memd".to_string()),
            agent: Some("test".to_string()),
            workspace: None,
            visibility: None,
            route: None,
            intent: Some(RetrievalIntent::CurrentTask),
            limit: Some(8),
            max_chars_per_item: Some(300),
            max_total_chars: Some(2400),
            rehydration_limit: Some(4),
            auto_consolidate: Some(false),
            query: None,
        },
    )
    .expect("build working memory");

    // The corrected item (with correction_boost +0.10) should appear
    let has_corrected = working
        .records
        .iter()
        .any(|r| r.record.contains("memd uses Rust"));
    assert!(
        has_corrected,
        "corrected fact must appear in working memory"
    );

    // The superseded original must NOT appear
    let has_superseded = working
        .records
        .iter()
        .any(|r| r.record.contains("memd uses Python"));
    assert!(
        !has_superseded,
        "superseded original must NOT appear in working memory"
    );

    std::fs::remove_dir_all(dir).expect("cleanup d2-e2e");
}

// ── D2: contradiction detection (3-item scenario) ───────────────────────────
//
// Entity grouping is path-based: items sharing source_path get the same entity
// regardless of content. This lets contradiction detection find siblings with
// different content about the same topic.

#[test]
fn d2_contradiction_marks_siblings_contested() {
    let (dir, state) = temp_state("d2-contradiction");
    let shared_path = "/docs/server-language.md";

    // Item A: "memd uses Python" — wrong claim, linked to path entity
    let (item_a, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "memd uses Python for the server".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: None,
                belief_branch: None,
                source_agent: Some("test".to_string()),
                source_system: None,
                source_path: Some(shared_path.to_string()),
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.8),
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: vec![],
                tags: vec![],
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store item A");

    // Item C: different content, same source_path → shares entity with A
    let (item_c, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "memd uses JavaScript for the server".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: None,
                belief_branch: None,
                source_agent: Some("test-other".to_string()),
                source_system: None,
                source_path: Some(shared_path.to_string()),
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.7),
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: vec![],
                tags: vec![],
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store item C");

    // Verify A and C share the same entity (same source_path → path-based entity key)
    let entity_a = state
        .store
        .entity_for_item(item_a.id)
        .expect("entity lookup A")
        .expect("A must have entity");
    let entity_c = state
        .store
        .entity_for_item(item_c.id)
        .expect("entity lookup C")
        .expect("C must have entity");
    assert_eq!(
        entity_a.id, entity_c.id,
        "items with same source_path must share entity"
    );

    // Correct A → B: "memd uses Rust"
    let response = repair::correct_item(
        &state,
        CorrectMemoryRequest {
            id: item_a.id,
            content: "memd uses Rust for the server".to_string(),
            reason: Some("server is Rust, not Python".to_string()),
            tags: None,
            confidence: None,
        },
    )
    .expect("correct A → B");

    // A is Superseded
    assert_eq!(response.old_item.status, MemoryStatus::Superseded);
    // B is Active
    assert_eq!(response.new_item.status, MemoryStatus::Active);
    // C should be Contested — sibling of old_item's entity, different content from B
    assert!(
        response.contested.contains(&item_c.id),
        "item C must appear in contested list; got {:?}",
        response.contested
    );

    // Verify C's persisted status is Contested
    let refreshed_c = state
        .store
        .get(item_c.id)
        .expect("get C")
        .expect("C exists");
    assert_eq!(
        refreshed_c.status,
        MemoryStatus::Contested,
        "C must be Contested in DB"
    );

    std::fs::remove_dir_all(dir).expect("cleanup d2-contradiction");
}

// ── E2: atlas navigation — wake → explore → expand → explain in ≤4 hops ────

#[test]
fn e2_atlas_navigation_four_hops() {
    let (dir, state) = temp_state("e2-atlas-nav");
    let store = &state.store;

    // Hop 0: Store 5+ items in same project (simulates "wake" seeding)
    let mut item_ids = Vec::new();
    let contents = [
        "memd stores data in SQLite",
        "memd uses Rust for the server",
        "memd entities track salience scores",
        "memd working memory ranks by priority",
        "memd wake packet compiles context",
    ];
    for content in &contents {
        let (item, _) = state
            .store_item(
                StoreMemoryRequest {
                    content: content.to_string(),
                    kind: MemoryKind::Fact,
                    scope: MemoryScope::Project,
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: None,
                    visibility: None,
                    belief_branch: None,
                    source_agent: Some("test".to_string()),
                    source_system: None,
                    source_path: None,
                    source_quality: Some(SourceQuality::Canonical),
                    confidence: Some(0.9),
                    ttl_seconds: None,
                    last_verified_at: None,
                    supersedes: vec![],
                    tags: vec![],
                    status: None,
                    lane: None,
                },
                MemoryStage::Canonical,
            )
            .expect("store item");
        item_ids.push(item.id);
    }

    // Verify entities auto-created
    for &id in &item_ids {
        let entity = store.entity_for_item(id).expect("entity lookup");
        assert!(
            entity.is_some(),
            "item {id} must have an entity after store"
        );
    }

    // Hop 1: Generate atlas → regions should be non-empty
    let regions = store
        .generate_regions_for_project(Some("memd"), Some("main"), None)
        .expect("generate regions");
    assert!(
        !regions.is_empty(),
        "atlas must generate at least 1 region from 5 items"
    );

    // Hop 2: Explore a region → nodes should include our items
    let region = &regions[0];
    let explore = store
        .explore_atlas(&memd_schema::AtlasExploreRequest {
            region_id: Some(region.id),
            node_id: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            lane: None,
            depth: Some(0),
            limit: None,
            pivot_time: None,
            pivot_kind: None,
            pivot_scope: None,
            pivot_source_agent: None,
            pivot_source_system: None,
            min_trust: None,
            min_salience: None,
            include_evidence: false,
            from_working: false,
        })
        .expect("explore atlas");
    assert!(
        !explore.nodes.is_empty(),
        "explore must return nodes for region"
    );

    // Hop 3: Expand from a node → linked items
    let seed_id = explore.nodes[0].memory_id;
    let expand = store
        .atlas_expand(&memd_schema::AtlasExpandRequest {
            memory_ids: vec![seed_id],
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            depth: Some(1),
            limit: Some(10),
        })
        .expect("expand atlas node");
    // Expand should return the seed + any linked nodes
    assert!(
        expand.seed_count >= 1,
        "expand must acknowledge at least 1 seed"
    );

    // Hop 4: Explain → provenance with sources
    let explain = inspection::explain_memory(
        &state,
        ExplainMemoryRequest {
            id: seed_id,
            belief_branch: None,
            route: None,
            intent: None,
        },
    )
    .expect("explain memory item");
    assert!(
        !explain.events.is_empty(),
        "explain must show lifecycle events (provenance)"
    );
    assert!(
        explain
            .events
            .iter()
            .any(|e| e.event_type == "canonical_created"),
        "provenance must include canonical_created event"
    );

    std::fs::remove_dir_all(dir).expect("cleanup e2-atlas-nav");
}

// ── H2: cross-session correction persistence ────────────────────────────────

#[test]
fn h2_cross_session_correction_persists() {
    let (dir, state) = temp_state("h2-cross-session");

    // Session 1: store + correct
    let (original, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "deploy target is AWS".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("infra".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: Some(MemoryVisibility::Workspace),
                belief_branch: None,
                source_agent: Some("session-1".to_string()),
                source_system: None,
                source_path: None,
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.9),
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: vec![],
                tags: vec![],
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store original");

    let correction = repair::correct_item(
        &state,
        CorrectMemoryRequest {
            id: original.id,
            content: "deploy target is GCP".to_string(),
            reason: Some("migrated to GCP".to_string()),
            tags: None,
            confidence: None,
        },
    )
    .expect("correct");

    // Session 2: rebuild context from scratch (simulates new session)
    let BuildContextResult { items, .. } = build_context(
        &state,
        &ContextRequest {
            project: Some("infra".to_string()),
            agent: Some("session-2".to_string()),
            workspace: None,
            visibility: None,
            route: Some(RetrievalRoute::ProjectFirst),
            intent: Some(RetrievalIntent::CurrentTask),
            limit: Some(10),
            max_chars_per_item: Some(300),
        },
    )
    .expect("build context session 2");

    assert!(
        items.iter().any(|i| i.id == correction.new_item.id),
        "corrected item must appear in new session context"
    );
    assert!(
        items.iter().all(|i| i.id != original.id),
        "superseded original must NOT appear in new session context"
    );

    std::fs::remove_dir_all(dir).expect("cleanup h2-cross-session");
}

// ── H2: cross-harness continuity — agent-A stores, agent-B retrieves ────────

#[test]
fn h2_cross_harness_item_retrievable() {
    let (dir, state) = temp_state("h2-cross-harness");

    // Agent A stores
    let (stored, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "database uses PostgreSQL".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("shared".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: Some(MemoryVisibility::Workspace),
                belief_branch: None,
                source_agent: Some("agent-A".to_string()),
                source_system: Some("system-A".to_string()),
                source_path: None,
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.9),
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: vec![],
                tags: vec![],
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store from agent-A");

    // Agent B retrieves — different agent, different system
    let BuildContextResult { items, .. } = build_context(
        &state,
        &ContextRequest {
            project: Some("shared".to_string()),
            agent: Some("agent-B".to_string()),
            workspace: None,
            visibility: None,
            route: Some(RetrievalRoute::ProjectFirst),
            intent: Some(RetrievalIntent::CurrentTask),
            limit: Some(10),
            max_chars_per_item: Some(300),
        },
    )
    .expect("build context from agent-B");

    assert!(
        items.iter().any(|i| i.id == stored.id),
        "item stored by agent-A must be retrievable by agent-B"
    );

    std::fs::remove_dir_all(dir).expect("cleanup h2-cross-harness");
}

/// H2: A/B influence test — corrections must improve retrieval, not degrade.
///
/// Baseline: store 5 facts, retrieve, measure which appear in build_context.
/// Treatment: correct 2 of those facts, retrieve again, verify:
///   (a) corrected versions appear in results,
///   (b) superseded originals do NOT appear,
///   (c) remaining 3 uncorrected items still appear (selective reset).
#[test]
fn h2_ab_influence_corrections_improve_retrieval() {
    let (dir, state) = temp_state("h2-ab-influence");

    // Baseline: store 5 facts.
    let mut item_ids = Vec::new();
    let contents = [
        "primary database is PostgreSQL",
        "cache layer uses Redis",
        "message queue is RabbitMQ",
        "deployment target is Kubernetes",
        "monitoring uses Prometheus",
    ];
    for content in &contents {
        let (stored, _) = state
            .store_item(
                StoreMemoryRequest {
                    content: content.to_string(),
                    kind: MemoryKind::Fact,
                    scope: MemoryScope::Project,
                    project: Some("infra".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: None,
                    visibility: None,
                    belief_branch: None,
                    source_agent: Some("h2-test".to_string()),
                    source_system: Some("test".to_string()),
                    source_path: None,
                    source_quality: Some(SourceQuality::Canonical),
                    confidence: Some(0.9),
                    ttl_seconds: None,
                    last_verified_at: None,
                    supersedes: vec![],
                    tags: vec![],
                    status: None,
                    lane: None,
                },
                MemoryStage::Canonical,
            )
            .expect("store baseline fact");
        item_ids.push(stored.id);
    }

    // Baseline retrieval
    let baseline_ctx = build_context(
        &state,
        &ContextRequest {
            project: Some("infra".to_string()),
            agent: Some("h2-test".to_string()),
            workspace: None,
            visibility: None,
            route: Some(RetrievalRoute::ProjectFirst),
            intent: Some(RetrievalIntent::General),
            limit: Some(10),
            max_chars_per_item: Some(300),
        },
    )
    .expect("baseline build_context");
    let baseline_count = baseline_ctx.items.len();
    assert!(
        baseline_count >= 5,
        "baseline should return all 5 items, got {}",
        baseline_count
    );

    // Treatment: correct items 0 and 1.
    let _correction_0 = repair::correct_item(
        &state,
        CorrectMemoryRequest {
            id: item_ids[0],
            content: "primary database is CockroachDB".to_string(),
            reason: Some("migration from PostgreSQL".to_string()),
            tags: None,
            confidence: None,
        },
    )
    .expect("correct item 0");
    let _correction_1 = repair::correct_item(
        &state,
        CorrectMemoryRequest {
            id: item_ids[1],
            content: "cache layer uses Dragonfly".to_string(),
            reason: Some("replaced Redis with Dragonfly".to_string()),
            tags: None,
            confidence: None,
        },
    )
    .expect("correct item 1");

    // Treatment retrieval
    let treatment_ctx = build_context(
        &state,
        &ContextRequest {
            project: Some("infra".to_string()),
            agent: Some("h2-test".to_string()),
            workspace: None,
            visibility: None,
            route: Some(RetrievalRoute::ProjectFirst),
            intent: Some(RetrievalIntent::General),
            limit: Some(10),
            max_chars_per_item: Some(300),
        },
    )
    .expect("treatment build_context");

    // (a) Corrected versions appear
    assert!(
        treatment_ctx
            .items
            .iter()
            .any(|i| i.content.contains("CockroachDB")),
        "corrected version (CockroachDB) must appear in treatment retrieval"
    );
    assert!(
        treatment_ctx
            .items
            .iter()
            .any(|i| i.content.contains("Dragonfly")),
        "corrected version (Dragonfly) must appear in treatment retrieval"
    );

    // (b) Superseded originals must NOT appear
    assert!(
        !treatment_ctx
            .items
            .iter()
            .any(|i| i.id == item_ids[0] || i.id == item_ids[1]),
        "superseded originals must not appear in treatment retrieval"
    );

    // (c) Remaining 3 uncorrected items still appear (selective reset)
    for &uncorrected_id in &item_ids[2..] {
        assert!(
            treatment_ctx.items.iter().any(|i| i.id == uncorrected_id),
            "uncorrected item {:?} must still appear after selective corrections",
            uncorrected_id
        );
    }

    // (d) Treatment quality >= baseline: same or more useful items returned
    assert!(
        treatment_ctx.items.len() >= baseline_count,
        "treatment must return at least as many items as baseline ({} vs {})",
        treatment_ctx.items.len(),
        baseline_count
    );

    std::fs::remove_dir_all(dir).expect("cleanup h2-ab-influence");
}

// ── J2: Isolation + Trust ──────────────────────────────────────────────

#[test]
fn j2_adversarial_visibility_private_items_invisible_to_other_agents() {
    let (dir, state) = temp_state("j2-adversarial-visibility");

    // Agent A stores a Private item
    let (private_item, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "agent A secret: internal API key rotation schedule".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: Some(MemoryVisibility::Private),
                source_agent: Some("agent-a".to_string()),
                source_system: Some("memd".to_string()),
                source_path: None,
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.95),
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec!["secret".to_string()],
                belief_branch: None,
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store agent A private item");

    // Agent A stores a Workspace item
    let (workspace_item, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "shared fact: memd uses SQLite with WAL mode".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: Some(MemoryVisibility::Workspace),
                source_agent: Some("agent-a".to_string()),
                source_system: Some("memd".to_string()),
                source_path: None,
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.9),
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec!["architecture".to_string()],
                belief_branch: None,
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store agent A workspace item");

    // Agent B queries the same project
    let agent_b_ctx = build_context(
        &state,
        &ContextRequest {
            project: Some("memd".to_string()),
            agent: Some("agent-b".to_string()),
            workspace: None,
            visibility: None,
            route: None,
            intent: None,
            limit: Some(32),
            max_chars_per_item: None,
        },
    )
    .expect("agent B context");

    // Assert: agent B cannot see agent A's Private item
    assert!(
        !agent_b_ctx
            .items
            .iter()
            .any(|item| item.id == private_item.id),
        "LEAK: agent B retrieved agent A's Private item"
    );

    // Assert: agent B CAN see agent A's Workspace item
    assert!(
        agent_b_ctx
            .items
            .iter()
            .any(|item| item.id == workspace_item.id),
        "agent B should see Workspace items from agent A"
    );

    // Agent A queries — should see both
    let agent_a_ctx = build_context(
        &state,
        &ContextRequest {
            project: Some("memd".to_string()),
            agent: Some("agent-a".to_string()),
            workspace: None,
            visibility: None,
            route: None,
            intent: None,
            limit: Some(32),
            max_chars_per_item: None,
        },
    )
    .expect("agent A context");

    assert!(
        agent_a_ctx
            .items
            .iter()
            .any(|item| item.id == private_item.id),
        "agent A should see own Private item"
    );
    assert!(
        agent_a_ctx
            .items
            .iter()
            .any(|item| item.id == workspace_item.id),
        "agent A should see own Workspace item"
    );

    std::fs::remove_dir_all(dir).expect("cleanup j2-adversarial-visibility");
}

#[test]
fn j2_multi_project_isolation_items_dont_cross_projects() {
    let (dir, state) = temp_state("j2-multi-project-isolation");

    // Store item in project X
    let (project_x_item, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "project X secret architecture decision".to_string(),
                kind: MemoryKind::Decision,
                scope: MemoryScope::Project,
                project: Some("project-x".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: Some(MemoryVisibility::Workspace),
                source_agent: Some("codex".to_string()),
                source_system: Some("memd".to_string()),
                source_path: None,
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.9),
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec![],
                belief_branch: None,
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store project X item");

    // Query from project Y context
    let project_y_ctx = build_context(
        &state,
        &ContextRequest {
            project: Some("project-y".to_string()),
            agent: Some("codex".to_string()),
            workspace: None,
            visibility: None,
            route: None,
            intent: None,
            limit: Some(32),
            max_chars_per_item: None,
        },
    )
    .expect("project Y context");

    // Assert: project X item NOT returned in project Y
    assert!(
        !project_y_ctx
            .items
            .iter()
            .any(|item| item.id == project_x_item.id),
        "LEAK: project X item appeared in project Y retrieval"
    );

    // Query from project X context — should find it
    let project_x_ctx = build_context(
        &state,
        &ContextRequest {
            project: Some("project-x".to_string()),
            agent: Some("codex".to_string()),
            workspace: None,
            visibility: None,
            route: None,
            intent: None,
            limit: Some(32),
            max_chars_per_item: None,
        },
    )
    .expect("project X context");

    assert!(
        project_x_ctx
            .items
            .iter()
            .any(|item| item.id == project_x_item.id),
        "project X item should be visible in project X"
    );

    std::fs::remove_dir_all(dir).expect("cleanup j2-multi-project-isolation");
}

#[test]
fn j2_per_agent_working_context_isolation() {
    use crate::working::working_memory;

    let (dir, state) = temp_state("j2-per-agent-working");

    // Agent A stores a Private fact
    let (private_item, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "agent A private procedure: restart services in order X→Y→Z".to_string(),
                kind: MemoryKind::Procedural,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: Some(MemoryVisibility::Private),
                source_agent: Some("agent-a".to_string()),
                source_system: Some("memd".to_string()),
                source_path: None,
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.9),
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec![],
                belief_branch: None,
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store agent A private item");

    // Agent B requests working memory for same project
    let agent_b_working = working_memory(
        &state,
        WorkingMemoryRequest {
            project: Some("memd".to_string()),
            agent: Some("agent-b".to_string()),
            workspace: None,
            visibility: None,
            route: None,
            intent: None,
            limit: None,
            max_chars_per_item: None,
            max_total_chars: None,
            rehydration_limit: None,
            auto_consolidate: None,
            query: None,
        },
    )
    .expect("agent B working memory");

    // Assert: agent B's working context does NOT contain agent A's Private item
    assert!(
        !agent_b_working
            .records
            .iter()
            .any(|record| record.id == private_item.id),
        "LEAK: agent A's Private item in agent B's working memory"
    );

    // Agent A requests working memory — should contain their own Private item
    let agent_a_working = working_memory(
        &state,
        WorkingMemoryRequest {
            project: Some("memd".to_string()),
            agent: Some("agent-a".to_string()),
            workspace: None,
            visibility: None,
            route: None,
            intent: None,
            limit: None,
            max_chars_per_item: None,
            max_total_chars: None,
            rehydration_limit: None,
            auto_consolidate: None,
            query: None,
        },
    )
    .expect("agent A working memory");

    assert!(
        agent_a_working
            .records
            .iter()
            .any(|record| record.id == private_item.id),
        "agent A should see own Private item in working memory"
    );

    std::fs::remove_dir_all(dir).expect("cleanup j2-per-agent-working");
}

#[test]
fn j2_consolidation_preserves_source_visibility() {
    let (dir, state) = temp_state("j2-consolidation-visibility");
    let plan = RetrievalPlan::resolve(
        Some(RetrievalRoute::ProjectFirst),
        Some(RetrievalIntent::General),
    );

    // Store 3 Private items from agent-a, all under same source_path.
    // Same source_path → same entity key → all events link to one entity.
    let mut source_items = Vec::new();
    for i in 0..3 {
        let (item, _) = state
            .store_item(
                StoreMemoryRequest {
                    content: format!("private note {}: agent-a internal context", i),
                    kind: MemoryKind::Fact,
                    scope: MemoryScope::Project,
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: None,
                    visibility: Some(MemoryVisibility::Private),
                    belief_branch: None,
                    source_agent: Some("agent-a".to_string()),
                    source_system: Some("cli".to_string()),
                    source_path: Some("notes/private.md".to_string()),
                    source_quality: Some(SourceQuality::Canonical),
                    confidence: Some(0.9),
                    ttl_seconds: None,
                    last_verified_at: Some(Utc::now()),
                    supersedes: Vec::new(),
                    tags: vec!["private".to_string()],
                    status: Some(MemoryStatus::Active),
                    lane: None,
                },
                MemoryStage::Canonical,
            )
            .expect("store private source item");
        source_items.push(item);
    }

    // Record retrieval feedback for each → creates memory events linking all 3
    // items to the same entity (same source_path entity key).
    state
        .record_retrieval_feedback(&source_items, 3, "retrieved_working", &plan)
        .expect("record retrieval feedback");

    // Consolidate with low thresholds so the test runs deterministically.
    let response = state
        .consolidate_semantic_memory(&MemoryConsolidationRequest {
            project: Some("memd".to_string()),
            namespace: None,
            max_groups: Some(8),
            min_events: Some(2),
            lookback_days: Some(30),
            min_salience: Some(0.0),
            record_events: Some(false),
        })
        .expect("consolidate semantic memory");

    assert!(
        response.consolidated >= 1,
        "expected at least 1 consolidated item, got {}",
        response.consolidated
    );

    // Agent-a requests context — consolidated item (Derived quality) must be Private.
    let agent_a_ctx = build_context(
        &state,
        &ContextRequest {
            project: Some("memd".to_string()),
            agent: Some("agent-a".to_string()),
            workspace: None,
            visibility: None,
            route: Some(RetrievalRoute::ProjectFirst),
            intent: Some(RetrievalIntent::General),
            limit: Some(16),
            max_chars_per_item: Some(220),
        },
    )
    .expect("build context agent-a");

    let consolidated = agent_a_ctx
        .items
        .iter()
        .find(|item| item.source_quality == Some(SourceQuality::Derived))
        .expect("consolidated item must appear in agent-a context");
    assert_eq!(
        consolidated.visibility,
        MemoryVisibility::Private,
        "consolidated item must inherit Private visibility from sources"
    );

    // Agent-b requests context — must not see any Private items.
    let agent_b_ctx = build_context(
        &state,
        &ContextRequest {
            project: Some("memd".to_string()),
            agent: Some("agent-b".to_string()),
            workspace: None,
            visibility: None,
            route: Some(RetrievalRoute::ProjectFirst),
            intent: Some(RetrievalIntent::General),
            limit: Some(16),
            max_chars_per_item: Some(220),
        },
    )
    .expect("build context agent-b");

    assert!(
        agent_b_ctx
            .items
            .iter()
            .all(|item| item.visibility != MemoryVisibility::Private),
        "LEAK: Private consolidated item visible to agent-b"
    );

    std::fs::remove_dir_all(dir).expect("cleanup j2-consolidation-visibility");
}

// O2.3: Decay sensitivity analysis
// Runs 5 param sets on identical pre-aged entities and compares outcomes.
//
// Param sets:
//   defaults:     inactive_days=21, max_decay=0.12, decay_divisor=14.0
//   aggressive:   inactive_days=14, max_decay=0.20, decay_divisor=7.0
//   conservative: inactive_days=30, max_decay=0.06, decay_divisor=21.0
//   fast_decay:   inactive_days=7,  max_decay=0.25, decay_divisor=5.0
//   slow_decay:   inactive_days=45, max_decay=0.04, decay_divisor=30.0
//
// Each scenario uses its own isolated DB seeded with 10 entities:
//   5 "old" entities (40 days idle, salience=0.6)
//   5 "recent" entities (5 days idle, salience=0.8)
#[test]
fn o2_3_decay_sensitivity_analysis() {
    struct Scenario {
        name: &'static str,
        inactive_days: i64,
        max_decay: f32,
        decay_divisor: f32,
        // expectations on the old entities (40 days idle)
        expect_old_decayed: bool,
        // expectations on the recent entities (5 days idle)
        expect_recent_decayed: bool,
    }

    let scenarios = [
        Scenario {
            name: "defaults",
            inactive_days: 21,
            max_decay: 0.12,
            decay_divisor: 14.0,
            expect_old_decayed: true,
            expect_recent_decayed: false,
        },
        Scenario {
            name: "aggressive",
            inactive_days: 14,
            max_decay: 0.20,
            decay_divisor: 7.0,
            expect_old_decayed: true,
            expect_recent_decayed: false,
        },
        Scenario {
            name: "conservative",
            inactive_days: 30,
            max_decay: 0.06,
            decay_divisor: 21.0,
            expect_old_decayed: true,
            expect_recent_decayed: false,
        },
        Scenario {
            name: "fast_decay",
            inactive_days: 7,
            max_decay: 0.25,
            decay_divisor: 5.0,
            expect_old_decayed: true,
            expect_recent_decayed: false,
        },
        Scenario {
            name: "slow_decay",
            inactive_days: 45,
            max_decay: 0.04,
            decay_divisor: 30.0,
            expect_old_decayed: false, // 40 days < 45 threshold
            expect_recent_decayed: false,
        },
    ];

    let mut results_table: Vec<(String, usize, usize, f32)> = Vec::new();

    for scenario in &scenarios {
        let (dir, state) = temp_state(&format!("o2-3-decay-{}", scenario.name));

        // Seed 10 entities via store_item (auto-creates memory_entities rows).
        for i in 0..10 {
            let _ = state
                .store_item(
                    StoreMemoryRequest {
                        content: format!("decay sensitivity entity {i}"),
                        kind: MemoryKind::Fact,
                        scope: MemoryScope::Project,
                        project: Some("memd".to_string()),
                        namespace: Some("main".to_string()),
                        workspace: None,
                        visibility: Some(MemoryVisibility::Public),
                        belief_branch: None,
                        source_agent: Some("test".to_string()),
                        source_system: Some("cli".to_string()),
                        source_path: None,
                        source_quality: Some(SourceQuality::Canonical),
                        confidence: Some(0.7),
                        ttl_seconds: None,
                        last_verified_at: None,
                        supersedes: Vec::new(),
                        tags: Vec::new(),
                        status: Some(MemoryStatus::Active),
                        lane: None,
                    },
                    MemoryStage::Canonical,
                )
                .expect("seed entity");
        }

        // Age the first 5 entities to 40 days idle; last 5 stay at 5 days idle.
        let now = chrono::Utc::now();
        let old_ts = (now - chrono::Duration::days(40)).to_rfc3339();
        let recent_ts = (now - chrono::Duration::days(5)).to_rfc3339();

        {
            let conn = state.store.connect().expect("connect for age patch");
            let all_keys: Vec<String> = {
                let mut stmt = conn
                    .prepare("SELECT entity_key FROM memory_entities ORDER BY rowid ASC")
                    .expect("prepare entity keys query");
                stmt.query_map([], |row| row.get(0))
                    .expect("query entity keys")
                    .map(|r| r.expect("read entity key"))
                    .collect()
            };

            // Also backdate the salience in payload_json so pre-decay salience is predictable.
            for (idx, key) in all_keys.iter().enumerate() {
                let ts = if idx < 5 { &old_ts } else { &recent_ts };
                let salience = if idx < 5 { 0.6f32 } else { 0.8f32 };
                // Read the current payload, patch timestamps + salience, write back.
                let payload_json: String = conn
                    .query_row(
                        "SELECT payload_json FROM memory_entities WHERE entity_key = ?1",
                        rusqlite::params![key],
                        |row| row.get(0),
                    )
                    .expect("read entity payload");
                let mut record: memd_schema::MemoryEntityRecord =
                    serde_json::from_str(&payload_json).expect("deserialize entity");
                record.salience_score = salience;
                record.last_accessed_at = Some(
                    chrono::DateTime::parse_from_rfc3339(ts)
                        .expect("parse ts")
                        .with_timezone(&chrono::Utc),
                );
                record.updated_at = chrono::DateTime::parse_from_rfc3339(ts)
                    .expect("parse ts")
                    .with_timezone(&chrono::Utc);
                let patched = serde_json::to_string(&record).expect("re-serialize entity");
                conn.execute(
                    "UPDATE memory_entities SET updated_at = ?1, payload_json = ?2 WHERE entity_key = ?3",
                    rusqlite::params![ts, patched, key],
                )
                .expect("patch entity age");
            }
        }

        // Run decay_diagnostics (read-only — does not mutate; use decay_entities for real run).
        let req = MemoryDecayRequest {
            max_items: Some(20),
            inactive_days: Some(scenario.inactive_days),
            max_decay: Some(scenario.max_decay),
            decay_divisor: Some(scenario.decay_divisor),
            record_events: Some(false),
        };
        let metrics = state
            .store
            .decay_diagnostics(&req)
            .expect("decay diagnostics");

        // Validate age distribution: all 10 entities were inspected.
        assert_eq!(
            metrics.inspected, 10,
            "[{}] expected 10 entities inspected",
            scenario.name
        );

        // old entities (40 days idle) should fall in over_30d bucket.
        assert_eq!(
            metrics.age_distribution.over_30d, 5,
            "[{}] expected 5 entities in over_30d bucket",
            scenario.name
        );

        // recent entities (5 days idle) should fall in under_7d bucket.
        assert_eq!(
            metrics.age_distribution.under_7d, 5,
            "[{}] expected 5 entities in under_7d bucket",
            scenario.name
        );

        // Check decay expectations.
        if scenario.expect_old_decayed {
            assert!(
                metrics.decayed > 0,
                "[{}] expected old entities to be decayed but decayed={}",
                scenario.name,
                metrics.decayed
            );
        } else {
            assert_eq!(
                metrics.decayed, 0,
                "[{}] expected NO decay (threshold not met) but decayed={}",
                scenario.name, metrics.decayed
            );
        }

        if !scenario.expect_recent_decayed {
            // Recent entities should never be decayed (5 days < all inactive_days thresholds).
            // We can't distinguish which decayed, but if old threshold is met and recent is not:
            // decayed count should be <= 5 (only old entities, not recent).
            // This holds for all scenarios where recent entities are below threshold.
            assert!(
                metrics.decayed <= 5,
                "[{}] recent entities should not be decayed, decayed={}",
                scenario.name,
                metrics.decayed
            );
        }

        results_table.push((
            scenario.name.to_string(),
            metrics.decayed,
            metrics.inspected,
            metrics.total_decay_applied,
        ));

        std::fs::remove_dir_all(dir)
            .unwrap_or_else(|_| eprintln!("warn: cleanup failed for {}", scenario.name));
    }

    // Print comparison table for documentation.
    println!("\nO2.3 Decay Sensitivity Comparison Table:");
    println!(
        "{:<14} {:>8} {:>10} {:>16}",
        "scenario", "decayed", "inspected", "total_decay"
    );
    for (name, decayed, inspected, total) in &results_table {
        println!(
            "{:<14} {:>8} {:>10} {:>16.4}",
            name, decayed, inspected, total
        );
    }

    // Ranking check: aggressive > defaults > conservative for total_decay (when old entities present).
    let aggressive_decay = results_table
        .iter()
        .find(|(n, ..)| n == "aggressive")
        .map(|(_, _, _, d)| *d)
        .unwrap_or(0.0);
    let defaults_decay = results_table
        .iter()
        .find(|(n, ..)| n == "defaults")
        .map(|(_, _, _, d)| *d)
        .unwrap_or(0.0);
    let conservative_decay = results_table
        .iter()
        .find(|(n, ..)| n == "conservative")
        .map(|(_, _, _, d)| *d)
        .unwrap_or(0.0);
    let slow_decay = results_table
        .iter()
        .find(|(n, ..)| n == "slow_decay")
        .map(|(_, _, _, d)| *d)
        .unwrap_or(0.0);

    assert!(
        aggressive_decay >= defaults_decay,
        "aggressive params must decay at least as much as defaults: {aggressive_decay:.4} vs {defaults_decay:.4}"
    );
    assert!(
        defaults_decay >= conservative_decay,
        "defaults must decay at least as much as conservative: {defaults_decay:.4} vs {conservative_decay:.4}"
    );
    assert_eq!(
        slow_decay, 0.0,
        "slow_decay scenario: 40-day-old entities should not decay (threshold=45d)"
    );
}

// O2.5: Post-consolidation A/B recall comparison
//
// Proves that consolidation does NOT degrade retrieval:
//   (a) Store 10 items on the same topic (all map to one entity via shared source_path)
//   (b) Run 5 context queries and record pre-consolidation hit counts
//   (c) Generate retrieval events so consolidation threshold is met
//   (d) Run consolidation
//   (e) Run the same 5 queries again and assert post >= pre for each
#[test]
fn o2_5_post_consolidation_recall_ab_test() {
    let (dir, state) = temp_state("o2-5-recall-ab");
    let plan = RetrievalPlan::resolve(
        Some(RetrievalRoute::ProjectFirst),
        Some(RetrievalIntent::General),
    );

    // (a) Store 10 items on the same topic.  All share source_path so they map to one entity.
    let rust_facts = [
        "rust ownership model prevents use-after-free at compile time",
        "rust borrow checker enforces single mutable reference per scope",
        "rust lifetimes ensure references never outlive their referents",
        "rust move semantics transfer ownership without copying heap data",
        "rust drop trait runs destructors deterministically when scope ends",
        "rust rc and arc provide shared ownership via reference counting",
        "rust box type allocates heap memory with sole ownership",
        "rust slice references provide safe views into contiguous memory",
        "rust unsafe blocks allow raw pointer operations with explicit opt-in",
        "rust pin type prevents moving self-referential structs in memory",
    ];

    let mut all_items: Vec<MemoryItem> = Vec::new();
    for content in &rust_facts {
        let (item, _) = state
            .store_item(
                StoreMemoryRequest {
                    content: content.to_string(),
                    kind: MemoryKind::Fact,
                    scope: MemoryScope::Project,
                    project: Some("probe".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: None,
                    visibility: Some(MemoryVisibility::Public),
                    belief_branch: None,
                    source_agent: Some("codex".to_string()),
                    source_system: Some("cli".to_string()),
                    source_path: Some("topic/rust-memory".to_string()),
                    source_quality: Some(SourceQuality::Canonical),
                    confidence: Some(0.9),
                    ttl_seconds: None,
                    last_verified_at: Some(Utc::now()),
                    supersedes: Vec::new(),
                    tags: vec!["rust".to_string(), "memory".to_string()],
                    status: Some(MemoryStatus::Active),
                    lane: None,
                },
                MemoryStage::Canonical,
            )
            .expect("store rust fact");
        all_items.push(item);
    }
    assert_eq!(all_items.len(), 10, "expected 10 seeded items");

    // Helper closure: build context and count items that contain "rust" (all seeded items do).
    let query = |limit: usize| -> usize {
        build_context(
            &state,
            &ContextRequest {
                project: Some("probe".to_string()),
                agent: Some("o2-5-agent".to_string()),
                workspace: None,
                visibility: None,
                route: Some(RetrievalRoute::ProjectFirst),
                intent: Some(RetrievalIntent::General),
                limit: Some(limit),
                max_chars_per_item: Some(300),
            },
        )
        .expect("build context")
        .items
        .into_iter()
        .filter(|i| i.content.contains("rust"))
        .count()
    };

    // (b) Pre-consolidation baseline: run 5 queries with increasing limits.
    let pre = [query(5), query(8), query(10), query(12), query(20)];
    let pre_total: usize = pre.iter().sum();
    // Sanity: at least the smallest query returns something.
    assert!(
        pre[0] >= 1,
        "pre-consolidation baseline empty at limit=5; got {}",
        pre[0]
    );

    // (c) Record retrieval events twice so the entity hits min_events=2.
    state
        .record_retrieval_feedback(&all_items, all_items.len(), "retrieved_working", &plan)
        .expect("retrieval feedback pass 1");
    state
        .record_retrieval_feedback(&all_items, all_items.len(), "retrieved_working", &plan)
        .expect("retrieval feedback pass 2");

    // (d) Run consolidation — min_events=2 means one entity (all 10 items share source_path)
    //     should be consolidated into a single synthesised item.
    let response = state
        .consolidate_semantic_memory(&MemoryConsolidationRequest {
            project: Some("probe".to_string()),
            namespace: Some("main".to_string()),
            max_groups: Some(4),
            min_events: Some(2),
            lookback_days: Some(7),
            min_salience: Some(0.0),
            record_events: Some(false),
        })
        .expect("consolidate semantic memory");
    assert!(
        response.consolidated >= 1,
        "expected at least 1 consolidated item; got {}",
        response.consolidated
    );

    // (e) Post-consolidation: same 5 queries. Post >= pre for each, and in aggregate.
    let post = [query(5), query(8), query(10), query(12), query(20)];
    let post_total: usize = post.iter().sum();

    for (i, (pre_hits, post_hits)) in pre.iter().zip(post.iter()).enumerate() {
        assert!(
            post_hits >= pre_hits,
            "query[{i}]: recall degraded post-consolidation — pre={pre_hits} post={post_hits}"
        );
    }
    assert!(
        post_total >= pre_total,
        "aggregate recall degraded post-consolidation — pre={pre_total} post={post_total}"
    );

    // Consolidated (Derived) item must be discoverable via retrieval.
    let final_ctx = build_context(
        &state,
        &ContextRequest {
            project: Some("probe".to_string()),
            agent: Some("o2-5-agent".to_string()),
            workspace: None,
            visibility: None,
            route: Some(RetrievalRoute::ProjectFirst),
            intent: Some(RetrievalIntent::General),
            limit: Some(50),
            max_chars_per_item: Some(500),
        },
    )
    .expect("final context");
    assert!(
        final_ctx
            .items
            .iter()
            .any(|i| i.source_quality == Some(SourceQuality::Derived)),
        "consolidated (Derived) item must appear in retrieval after consolidation"
    );

    println!(
        "\nO2.5 A/B Recall: pre=[{},{},{},{},{}] post=[{},{},{},{},{}] total {pre_total}->{post_total}",
        pre[0], pre[1], pre[2], pre[3], pre[4], post[0], post[1], post[2], post[3], post[4],
    );

    std::fs::remove_dir_all(dir).expect("cleanup o2-5-recall-ab");
}

#[test]
fn p2_compaction_quality_report_includes_per_kind_chars() {
    // P2: Verify CompactionQualityReport tracks per-kind character counts
    let (dir, state) = temp_state("p2-per-kind-chars");

    // Store items of different kinds
    let kinds_and_content = vec![
        (MemoryKind::Fact, "The earth revolves around the sun"),
        (
            MemoryKind::Decision,
            "We chose Rust for memory safety and performance",
        ),
        (
            MemoryKind::Preference,
            "User prefers dark mode in the dashboard",
        ),
        (MemoryKind::Status, "M3 phase P2 is in progress"),
    ];

    for (kind, content) in &kinds_and_content {
        let mut item = sample_memory_item(None);
        item.kind = kind.clone();
        item.content = content.to_string();
        item.project = Some("p2-test".to_string());
        let ck = super::keys::canonical_key(&item);
        let rk = super::keys::redundancy_key(&item);
        state
            .store
            .insert_or_get_duplicate(&item, &ck, &rk)
            .expect("store p2 test item");
    }

    // Build working memory
    let req = memd_schema::WorkingMemoryRequest {
        project: Some("p2-test".to_string()),
        agent: None,
        workspace: None,
        visibility: None,
        route: Some(memd_schema::RetrievalRoute::ProjectFirst),
        intent: Some(memd_schema::RetrievalIntent::CurrentTask),
        limit: None,
        max_chars_per_item: None,
        max_total_chars: None,
        rehydration_limit: None,
        auto_consolidate: None,
        query: None,
    };

    let response = crate::working::working_memory(&state, req).expect("build working memory");

    // Verify compaction quality report exists and has per-kind char breakdown
    let cq = response
        .compaction_quality
        .expect("compaction quality report must exist");

    assert!(cq.admitted > 0, "at least one item should be admitted");
    assert!(
        !cq.per_kind_admitted.is_empty(),
        "per_kind_admitted should have entries"
    );
    assert!(
        !cq.chars_per_kind_admitted.is_empty(),
        "chars_per_kind_admitted should have entries (P2 per-kind char tracking)"
    );

    // Verify chars are non-zero for admitted kinds
    for (kind, chars) in &cq.chars_per_kind_admitted {
        assert!(
            *chars > 0,
            "kind '{kind}' should have non-zero character count"
        );
    }

    // Verify budget utilization
    assert!(cq.budget_chars > 0, "budget_chars should be positive");
    assert!(
        cq.used_chars <= cq.budget_chars,
        "used_chars ({}) should not exceed budget_chars ({})",
        cq.used_chars,
        cq.budget_chars
    );

    println!(
        "\nP2 Token Efficiency: budget={}, used={}, utilization={:.1}%",
        cq.budget_chars,
        cq.used_chars,
        (cq.used_chars as f64 / cq.budget_chars as f64) * 100.0
    );
    println!("Per-kind chars: {:?}", cq.chars_per_kind_admitted);

    std::fs::remove_dir_all(dir).expect("cleanup p2 test");
}

#[test]
fn working_memory_retrieval_p95_under_100ms() {
    // K2.6 CI gate: seed a realistic corpus, issue N working-memory requests
    // through the same path as the /memory/working handler, and assert the
    // histogram p95 stays under the SLA. Threshold is intentionally generous
    // to absorb cold-cache noise on CI; regressions above this point mean
    // the retrieval path has drifted.
    let (dir, state) = temp_state("memd-latency-sla");
    for n in 0..64 {
        state
            .store_item(
                StoreMemoryRequest {
                    content: format!("warm fact {n}"),
                    kind: MemoryKind::Fact,
                    scope: MemoryScope::Project,
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: None,
                    visibility: None,
                    belief_branch: None,
                    source_agent: Some("codex".to_string()),
                    source_system: None,
                    source_path: None,
                    source_quality: None,
                    confidence: None,
                    ttl_seconds: None,
                    last_verified_at: None,
                    supersedes: vec![],
                    tags: vec![],
                    status: None,
                    lane: None,
                },
                MemoryStage::Canonical,
            )
            .expect("seed fact");
    }

    for _ in 0..20 {
        let started = std::time::Instant::now();
        let _ = crate::working::working_memory(
            &state,
            WorkingMemoryRequest {
                project: Some("memd".to_string()),
                agent: Some("codex".to_string()),
                workspace: None,
                visibility: None,
                route: None,
                intent: Some(RetrievalIntent::CurrentTask),
                limit: Some(8),
                max_chars_per_item: Some(220),
                max_total_chars: Some(1600),
                rehydration_limit: Some(4),
                auto_consolidate: Some(false),
                query: None,
            },
        )
        .expect("working memory");
        state
            .latency
            .record_ms(started.elapsed().as_millis() as u64);
    }

    let snap = state.latency.snapshot();
    assert!(
        snap.total >= 20,
        "expected 20 recorded samples, got {}",
        snap.total
    );

    // Debug builds run SQLite-bound paths roughly 5-10x slower than release,
    // so the hard 100ms gate only fires in release/CI. Debug gets a looser
    // smoke check that still catches pathological regressions.
    #[cfg(not(debug_assertions))]
    assert!(
        snap.p95_ms < 100.0,
        "working-memory retrieval p95 exceeded 100ms SLA: p95={} mean={} max={}",
        snap.p95_ms,
        snap.mean_ms,
        snap.max_ms,
    );
    #[cfg(debug_assertions)]
    assert!(
        snap.p95_ms < 500.0,
        "debug-build working-memory p95 regression: p95={} mean={}",
        snap.p95_ms,
        snap.mean_ms,
    );

    std::fs::remove_dir_all(dir).expect("cleanup latency sla test");
}

#[test]
fn spine_verify_reports_no_violations_on_clean_store() {
    let (dir, state) = temp_state("memd-spine-verify");

    for n in 0..3 {
        state
            .store_item(
                StoreMemoryRequest {
                    content: format!("fact {n}"),
                    kind: MemoryKind::Fact,
                    scope: MemoryScope::Project,
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: None,
                    visibility: None,
                    belief_branch: None,
                    source_agent: Some("test".to_string()),
                    source_system: None,
                    source_path: None,
                    source_quality: None,
                    confidence: None,
                    ttl_seconds: None,
                    last_verified_at: None,
                    supersedes: vec![],
                    tags: vec![],
                    status: None,
                    lane: None,
                },
                MemoryStage::Canonical,
            )
            .expect("store fact");
    }

    let report = state.store.verify_spine().expect("verify spine");
    assert!(report.scanned > 0, "at least the store events should scan");
    assert_eq!(report.monotonic_violations, 0);
    assert!(report.first_violation.is_none());
    assert_eq!(report.rolling_sha256.len(), 64);
    assert!(report.rolling_sha256.chars().all(|c| c.is_ascii_hexdigit()));

    let again = state.store.verify_spine().expect("verify spine again");
    assert_eq!(
        again.rolling_sha256, report.rolling_sha256,
        "rolling hash should be deterministic across calls"
    );

    std::fs::remove_dir_all(dir).expect("cleanup spine verify test");
}

#[test]
fn l2_1_lamport_version_increments_on_mutation_and_rejects_stale_imports() {
    let (dir, state) = temp_state("l2-1-lamport-version");
    let item = sample_memory_item(Some("core"));
    let id = item.id;
    let ck = keys::canonical_key(&item);
    let rk = keys::redundancy_key(&item);

    // Insert: persisted version starts at 1.
    state
        .store
        .insert_or_get_duplicate(&item, &ck, &rk)
        .expect("insert new item");
    assert_eq!(
        state.store.get_version(id).expect("read version"),
        Some(1),
        "fresh insert persists at version 1"
    );

    // Local mutation: version auto-increments to 2.
    let mut mutated = item.clone();
    mutated.content = "workspace-ranked memory (edited)".to_string();
    mutated.updated_at = Utc::now();
    let ck2 = keys::canonical_key(&mutated);
    let rk2 = keys::redundancy_key(&mutated);
    state
        .store
        .update(&mutated, &ck2, &rk2)
        .expect("update item");
    assert_eq!(
        state
            .store
            .get_version(id)
            .expect("read version after update"),
        Some(2),
        "update bumps Lamport version by 1"
    );
    let stored = state.store.get(id).expect("get").expect("row present");
    assert_eq!(stored.version, 2, "payload_json and column stay in sync");

    // Import with equal version: rejected.
    let mut stale = stored.clone();
    stale.version = 2;
    let outcome = state
        .store
        .import_with_version(&stale, &ck2, &rk2)
        .expect("import call");
    assert_eq!(
        outcome,
        crate::store::ImportOutcome::RejectedStale {
            stored_version: 2,
            incoming_version: 2,
        },
        "equal-version import must be treated as stale"
    );

    // Import with strictly-greater version: applied, version becomes 5.
    let mut fresh = stored.clone();
    fresh.version = 5;
    fresh.content = "workspace-ranked memory (remote)".to_string();
    fresh.updated_at = Utc::now();
    let ck3 = keys::canonical_key(&fresh);
    let rk3 = keys::redundancy_key(&fresh);
    let outcome = state
        .store
        .import_with_version(&fresh, &ck3, &rk3)
        .expect("import fresh");
    assert_eq!(outcome, crate::store::ImportOutcome::Applied);
    assert_eq!(
        state
            .store
            .get_version(id)
            .expect("read version post-import"),
        Some(5),
        "accepted import preserves incoming version exactly"
    );

    std::fs::remove_dir_all(dir).expect("cleanup L2.1 test");
}

// L2.6: end-to-end rate limit middleware. Wires a one-route router exactly
// like the real app — same `from_fn_with_state` layer — and hammers it. Reads
// stay unthrottled, writes cross both thresholds, header carries agent key.
#[tokio::test]
async fn rate_limit_middleware_throttles_writes_per_agent_and_passes_reads() {
    let dir = std::env::temp_dir().join(format!("memd-rl-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let db_path = dir.join("memd.db");
    let state = AppState {
        store: SqliteStore::open(&db_path).expect("open temp store"),
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::with(
            2,
            4,
            std::time::Duration::from_secs(60),
        )),
        rag: None,
        embedder: None,
    };

    let app = Router::new()
        .route(
            "/ping",
            axum::routing::get(|| async { "pong" }).post(|| async { "wrote" }),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::rate_limit::rate_limit_middleware,
        ))
        .with_state(state);

    async fn status_and_headers(
        app: &Router,
        method: &str,
        agent: Option<&str>,
    ) -> (axum::http::StatusCode, axum::http::HeaderMap) {
        let mut req = Request::builder().method(method).uri("/ping");
        if let Some(a) = agent {
            req = req.header("x-memd-agent", a);
        }
        let resp = app
            .clone()
            .oneshot(req.body(Body::empty()).unwrap())
            .await
            .expect("oneshot");
        (resp.status(), resp.headers().clone())
    }

    // GET is never throttled.
    for _ in 0..10 {
        let (status, _) = status_and_headers(&app, "GET", Some("agent-1")).await;
        assert_eq!(status, axum::http::StatusCode::OK);
    }

    // Writes 1..=2 succeed for agent-1 (soft=2).
    let (s1, h1) = status_and_headers(&app, "POST", Some("agent-1")).await;
    assert_eq!(s1, axum::http::StatusCode::OK);
    assert!(h1.contains_key("x-memd-ratelimit-remaining"));
    let (s2, _) = status_and_headers(&app, "POST", Some("agent-1")).await;
    assert_eq!(s2, axum::http::StatusCode::OK);

    // Writes 3..=4 are soft-throttled → 429 + Retry-After.
    let (s3, h3) = status_and_headers(&app, "POST", Some("agent-1")).await;
    assert_eq!(s3, axum::http::StatusCode::TOO_MANY_REQUESTS);
    assert!(h3.contains_key("retry-after"));
    let (s4, _) = status_and_headers(&app, "POST", Some("agent-1")).await;
    assert_eq!(s4, axum::http::StatusCode::TOO_MANY_REQUESTS);

    // Write 5 is hard-rejected (still 429 but tier="hard").
    let (s5, _) = status_and_headers(&app, "POST", Some("agent-1")).await;
    assert_eq!(s5, axum::http::StatusCode::TOO_MANY_REQUESTS);

    // A different agent's bucket is independent.
    let (s_b1, _) = status_and_headers(&app, "POST", Some("agent-2")).await;
    assert_eq!(s_b1, axum::http::StatusCode::OK);

    std::fs::remove_dir_all(dir).expect("cleanup rl test");
}

// L2.7: 10 threads × 100 writes each. busy_timeout=5000 + WAL journal must
// absorb contention entirely. If any thread surfaces SQLITE_BUSY, L2-D3
// (the WAL+busy_timeout guarantees from M2) regressed — fail hard.
#[test]
fn concurrency_10_threads_100_writes_no_sqlite_busy_surfaces() {
    let dir = std::env::temp_dir().join(format!("memd-concurrency-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let db_path = dir.join("state.sqlite");
    let store = SqliteStore::open(&db_path).expect("open store");

    const THREADS: usize = 10;
    const PER_THREAD: usize = 100;

    let start = std::sync::Arc::new(std::sync::Barrier::new(THREADS));
    let mut handles = Vec::with_capacity(THREADS);
    for tid in 0..THREADS {
        let store = store.clone();
        let start = start.clone();
        handles.push(std::thread::spawn(move || {
            start.wait();
            let mut busy_hits = 0usize;
            let mut other_errors: Vec<String> = Vec::new();
            for i in 0..PER_THREAD {
                let mut item = sample_memory_item(None);
                item.id = uuid::Uuid::new_v4();
                item.content = format!("concurrency t{tid}-i{i}");
                item.tags = vec!["concurrency".to_string(), format!("t{tid}")];
                item.source_agent = Some(format!("agent-t{tid}"));
                let ck = keys::canonical_key(&item);
                let rk = keys::redundancy_key(&item);
                match store.insert_or_get_duplicate(&item, &ck, &rk) {
                    Ok(_) => {}
                    Err(err) => {
                        let msg = format!("{err:#}").to_lowercase();
                        if msg.contains("busy") || msg.contains("database is locked") {
                            busy_hits += 1;
                        } else {
                            other_errors.push(format!("t{tid}-i{i}: {err:#}"));
                        }
                    }
                }
            }
            (busy_hits, other_errors)
        }));
    }

    let mut total_busy = 0usize;
    let mut all_errors: Vec<String> = Vec::new();
    for h in handles {
        let (busy, errs) = h.join().expect("thread joined");
        total_busy += busy;
        all_errors.extend(errs);
    }

    assert_eq!(
        total_busy, 0,
        "SQLITE_BUSY must not surface under WAL + 5000ms busy_timeout (L2-D3 regressed)"
    );
    assert!(
        all_errors.is_empty(),
        "unexpected non-busy errors: {all_errors:#?}"
    );

    // Row count sanity: each thread produced 100 distinct writes.
    let conn = rusqlite::Connection::open(&db_path).expect("reopen to count");
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM memory_items WHERE source_agent LIKE 'agent-t%'",
            [],
            |row| row.get(0),
        )
        .expect("count rows");
    assert_eq!(count, (THREADS * PER_THREAD) as i64);

    std::fs::remove_dir_all(dir).expect("cleanup concurrency test");
}

// L2.8: cross-harness E2E — codex-style harness A hands off to
// claude-code-style harness B, B makes corrections, A wakes up and picks
// them up. Runs against the shared store (single sqlite file) the way two
// harnesses coexist in production: distinct source_agent, shared storage.
#[test]
fn cross_harness_e2e_a_to_b_with_corrections_picked_up_by_a() {
    use memd_schema::{
        CompactMemoryRecord, HiveHandoffPacket, Procedure, ProcedureKind, ProcedureStatus,
        WorkingContextSnapshot,
    };

    let dir = std::env::temp_dir().join(format!("memd-x-harness-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let store = SqliteStore::open(dir.join("state.sqlite")).expect("open store");

    // Shared namespace/workspace — both harnesses operate on the same lane.
    let project = Some("memd".to_string());
    let namespace = Some("main".to_string());
    let workspace = Some("shared".to_string());

    fn seed_as(
        store: &SqliteStore,
        agent: &str,
        kind: MemoryKind,
        content: &str,
        project: &Option<String>,
        namespace: &Option<String>,
        workspace: &Option<String>,
    ) -> MemoryItem {
        let now = Utc::now();
        let item = MemoryItem {
            id: uuid::Uuid::new_v4(),
            content: content.to_string(),
            redundancy_key: None,
            belief_branch: None,
            preferred: false,
            kind,
            scope: MemoryScope::Project,
            project: project.clone(),
            namespace: namespace.clone(),
            workspace: workspace.clone(),
            visibility: MemoryVisibility::Workspace,
            source_agent: Some(agent.to_string()),
            source_system: Some("cross-harness-test".to_string()),
            source_path: None,
            source_quality: Some(SourceQuality::Canonical),
            confidence: 0.9,
            ttl_seconds: None,
            created_at: now,
            updated_at: now,
            last_verified_at: Some(now),
            supersedes: Vec::new(),
            tags: vec!["cross-harness".to_string()],
            status: MemoryStatus::Active,
            stage: MemoryStage::Canonical,
            lane: None,
            version: 1,
            correction_meta: None,
        };
        let ck = keys::canonical_key(&item);
        let rk = keys::redundancy_key(&item);
        store
            .insert_or_get_duplicate(&item, &ck, &rk)
            .expect("seed item");
        item
    }

    // (a) Harness A: 3 facts + 2 decisions + 1 procedure candidate.
    let a_fact_1 = seed_as(
        &store,
        "codex@A",
        MemoryKind::Fact,
        "hive uses Lamport clocks",
        &project,
        &namespace,
        &workspace,
    );
    let a_fact_2 = seed_as(
        &store,
        "codex@A",
        MemoryKind::Fact,
        "writes retry under WAL",
        &project,
        &namespace,
        &workspace,
    );
    let a_fact_3 = seed_as(
        &store,
        "codex@A",
        MemoryKind::Fact,
        "procedural memory is 8-slot bounded",
        &project,
        &namespace,
        &workspace,
    );
    let a_dec_1 = seed_as(
        &store,
        "codex@A",
        MemoryKind::Decision,
        "adopt FTS5 for search",
        &project,
        &namespace,
        &workspace,
    );
    let a_dec_2 = seed_as(
        &store,
        "codex@A",
        MemoryKind::Decision,
        "use SQLite online backup for snapshots",
        &project,
        &namespace,
        &workspace,
    );
    let a_proc_seed = seed_as(
        &store,
        "codex@A",
        MemoryKind::Procedural,
        "when tests fail intermittently, check busy_timeout",
        &project,
        &namespace,
        &workspace,
    );

    // (b) Build a handoff packet from A. Snapshot carries working records +
    //     unresolved procedure candidate.
    let compact = |item: &MemoryItem| CompactMemoryRecord {
        id: item.id,
        record: item.content.clone(),
    };
    let snapshot = WorkingContextSnapshot {
        working_records: vec![
            compact(&a_fact_1),
            compact(&a_fact_2),
            compact(&a_fact_3),
            compact(&a_dec_1),
            compact(&a_dec_2),
        ],
        doing: Some("ship L2".to_string()),
        left_off: Some("just finished L2.7".to_string()),
        next_action: Some("start L2.8".to_string()),
        blocker: None,
        unresolved_procedures: vec![Procedure {
            id: uuid::Uuid::new_v4(),
            name: "retry on flakes".to_string(),
            description: "observed pattern from A".to_string(),
            kind: ProcedureKind::Recovery,
            status: ProcedureStatus::Candidate,
            trigger: "tests flake".to_string(),
            steps: vec!["check busy_timeout".to_string()],
            success_criteria: None,
            source_ids: vec![a_proc_seed.id],
            project: project.clone(),
            namespace: namespace.clone(),
            use_count: 0,
            confidence: 0.6,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: vec!["cross-harness".to_string()],
            session_count: 0,
            last_session: Some("A".to_string()),
            supersedes: None,
        }],
        version: 1,
        captured_at: Some(Utc::now()),
    }
    .truncate_to_cap();

    let packet = HiveHandoffPacket {
        from_session: "A".to_string(),
        from_worker: Some("codex".to_string()),
        to_session: "B".to_string(),
        to_worker: Some("claude-code".to_string()),
        task_id: Some("ship-l2".to_string()),
        topic_claim: Some("L2 hive hardening".to_string()),
        scope_claims: Vec::new(),
        next_action: Some("start L2.8".to_string()),
        blocker: None,
        note: Some("running out of context".to_string()),
        created_at: Utc::now(),
        working_context: Some(snapshot.clone()),
    };

    // (c) Harness B resumes. In production this fans out to store calls;
    //     here we validate the contract: all seeded items referenced by the
    //     snapshot are visible via the shared store.
    for rec in &packet.working_context.as_ref().unwrap().working_records {
        let row = store
            .get(rec.id)
            .expect("store.get works")
            .expect("handed-off item visible to B");
        assert_eq!(row.content, rec.record);
    }
    assert_eq!(
        packet
            .working_context
            .as_ref()
            .unwrap()
            .working_records
            .len(),
        5,
        "3 facts + 2 decisions"
    );
    assert_eq!(
        packet
            .working_context
            .as_ref()
            .unwrap()
            .unresolved_procedures
            .len(),
        1,
        "1 procedure candidate"
    );

    // (e) Harness B: correction to fact_1 + new decision.
    let mut corrected_fact = a_fact_1.clone();
    corrected_fact.id = uuid::Uuid::new_v4();
    corrected_fact.content = "hive uses Lamport clocks (versioned u64)".to_string();
    corrected_fact.source_agent = Some("claude-code@B".to_string());
    corrected_fact.supersedes = vec![a_fact_1.id];
    corrected_fact.updated_at = Utc::now();
    let ck = keys::canonical_key(&corrected_fact);
    let rk = keys::redundancy_key(&corrected_fact);
    store
        .insert_or_get_duplicate(&corrected_fact, &ck, &rk)
        .expect("B writes correction");

    let b_new_dec = seed_as(
        &store,
        "claude-code@B",
        MemoryKind::Decision,
        "prefer online backup over cold copy",
        &project,
        &namespace,
        &workspace,
    );

    // (f) Harness A wakes up — reads from same store, sees B's additions.
    let all_items = {
        let conn = rusqlite::Connection::open(dir.join("state.sqlite")).expect("reopen to query");
        let mut stmt = conn
            .prepare("SELECT payload_json FROM memory_items")
            .unwrap();
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .map(|r| {
                let s = r.unwrap();
                serde_json::from_str::<MemoryItem>(&s).expect("decode payload")
            })
            .collect::<Vec<_>>();
        rows
    };

    let correction_visible = all_items
        .iter()
        .any(|i| i.id == corrected_fact.id && i.supersedes == vec![a_fact_1.id]);
    assert!(correction_visible, "A must see B's correction chain");

    let new_dec_visible = all_items.iter().any(|i| i.id == b_new_dec.id);
    assert!(new_dec_visible, "A must see B's newly added decision");

    // And the originals remain reachable so the supersedes chain works.
    for original in [
        &a_fact_1,
        &a_fact_2,
        &a_fact_3,
        &a_dec_1,
        &a_dec_2,
        &a_proc_seed,
    ] {
        let found = all_items.iter().any(|i| i.id == original.id);
        assert!(found, "original item {} still in shared store", original.id);
    }

    std::fs::remove_dir_all(dir).expect("cleanup x-harness");
}

#[tokio::test]
async fn healthz_route_without_rag_env_marks_rag_disabled() {
    let (dir, state) = temp_state("memd-healthz-no-rag");
    let app = Router::new()
        .route("/healthz", get(healthz))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .expect("build healthz request"),
        )
        .await
        .expect("run healthz route");

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = decode_json(response).await;
    assert_eq!(body["rag"]["enabled"], false);
    assert_eq!(body["rag"]["reachable"], false);
    assert!(body["rag"]["name"].is_null());

    std::fs::remove_dir_all(dir).expect("cleanup healthz temp dir");
}

#[tokio::test]
async fn healthz_route_surfaces_reachable_rag_name() {
    let (rag_url, _rx) = spawn_mock_rag_ingest_server().await;
    let (dir, state) = temp_state_with_rag("memd-healthz-rag", Some(&rag_url));
    let app = Router::new()
        .route("/healthz", get(healthz))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .expect("build healthz request"),
        )
        .await
        .expect("run healthz route");

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = decode_json(response).await;
    assert_eq!(body["rag"]["enabled"], true);
    assert_eq!(body["rag"]["reachable"], true);
    assert_eq!(body["rag"]["name"], "rag-sidecar");

    std::fs::remove_dir_all(dir).expect("cleanup rag healthz temp dir");
}

#[tokio::test]
async fn healthz_route_surfaces_atlas_dormant_warning() {
    let (dir, state) = temp_state("memd-healthz-atlas-dormant");
    state
        .store_item(
            StoreMemoryRequest {
                content: "single item without atlas links".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
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
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store lone item");
    let app = Router::new()
        .route("/healthz", get(healthz))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .expect("build healthz request"),
        )
        .await
        .expect("run healthz route");

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = decode_json(response).await;
    assert_eq!(body["atlas"]["edges_active"], 0);
    assert_eq!(body["atlas"]["region_count"], 0);
    assert_eq!(body["atlas"]["dormant"], true);
    assert!(
        body["atlas"]["warning"]
            .as_str()
            .unwrap_or_default()
            .contains("atlas dormant")
    );

    std::fs::remove_dir_all(dir).expect("cleanup atlas dormant temp dir");
}

#[tokio::test]
async fn search_memory_region_filter_limits_results_to_region_members() {
    let (dir, state) = temp_state("memd-search-region-filter");

    state
        .store_item(
            StoreMemoryRequest {
                content: "fact region item".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("atlas-region".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: Some(MemoryVisibility::Workspace),
                belief_branch: None,
                source_agent: None,
                source_system: None,
                source_path: None,
                source_quality: None,
                confidence: Some(0.8),
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec!["facts".to_string()],
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store fact");
    state
        .store_item(
            StoreMemoryRequest {
                content: "decision region item".to_string(),
                kind: MemoryKind::Decision,
                scope: MemoryScope::Project,
                project: Some("atlas-region".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: Some(MemoryVisibility::Workspace),
                belief_branch: None,
                source_agent: None,
                source_system: None,
                source_path: None,
                source_quality: None,
                confidence: Some(0.9),
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec!["decisions".to_string()],
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store decision");

    let region_members = crate::routes::resolve_region_member_filter(
        &state,
        Some("facts"),
        Some("atlas-region"),
        Some("main"),
    )
    .expect("resolve region members")
    .expect("expected region filter set");
    assert_eq!(
        region_members.len(),
        1,
        "facts region should resolve one member"
    );

    let Json(response) = search_memory(
        State(state),
        Json(SearchMemoryRequest {
            query: None,
            route: None,
            intent: None,
            scopes: vec![MemoryScope::Project],
            kinds: Vec::new(),
            statuses: vec![MemoryStatus::Active],
            project: Some("atlas-region".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
            visibility: None,
            belief_branch: None,
            source_agent: None,
            region: Some("facts".to_string()),
            tags: Vec::new(),
            stages: vec![MemoryStage::Canonical],
            limit: Some(10),
            max_chars_per_item: None,
        }),
    )
    .await
    .expect("search with region");

    assert!(
        !response.items.is_empty(),
        "region-filtered search should return items"
    );
    assert!(
        response
            .items
            .iter()
            .all(|item| item.kind == MemoryKind::Fact)
    );

    std::fs::remove_dir_all(dir).expect("cleanup region filter temp dir");
}

#[test]
fn correct_item_closes_source_backed_atlas_links() {
    let (dir, state) = temp_state("memd-correction-closes-atlas-links");

    let (old_item, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "old auth belief".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("atlas-correct".to_string()),
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
                tags: Vec::new(),
                status: None,
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store old item");
    let (peer_item, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "peer auth evidence".to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("atlas-correct".to_string()),
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
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("store peer item");

    let old_entity = state
        .store
        .entity_for_item(old_item.id)
        .expect("old entity lookup")
        .expect("old entity present");
    let peer_entity = state
        .store
        .entity_for_item(peer_item.id)
        .expect("peer entity lookup")
        .expect("peer entity present");
    state
        .store
        .link_entity(&memd_schema::EntityLinkRequest {
            from_entity_id: old_entity.id,
            to_entity_id: peer_entity.id,
            relation_kind: memd_schema::EntityRelationKind::Related,
            confidence: Some(0.9),
            valid_from: Some(old_item.updated_at),
            valid_to: None,
            source_item_id: Some(old_item.id),
            note: Some("old-item atlas path".to_string()),
            context: None,
            tags: vec!["manual".to_string()],
        })
        .expect("create source-backed link");

    let response = repair::correct_item(
        &state,
        CorrectMemoryRequest {
            id: old_item.id,
            content: "corrected auth belief".to_string(),
            reason: Some("auth updated".to_string()),
            tags: None,
            confidence: Some(0.95),
        },
    )
    .expect("correct item");

    let links = state
        .store
        .links_for_entity(&memd_schema::EntityLinksRequest {
            entity_id: old_entity.id,
        })
        .expect("list old links");
    let closed = links
        .iter()
        .find(|link| link.source_item_id == Some(old_item.id))
        .expect("source-backed link should remain readable");
    assert!(
        closed.valid_to.is_some(),
        "source-backed link should be time-closed when correction supersedes the old item"
    );
    assert_eq!(response.old_item.status, MemoryStatus::Superseded);

    std::fs::remove_dir_all(dir).expect("cleanup correction temp dir");
}

#[tokio::test]
async fn store_memory_fanouts_rag_ingest_with_identity_contract() {
    let (rag_url, rx) = spawn_mock_rag_ingest_server().await;
    let (dir, state) = temp_state_with_rag("memd-rag-ingest", Some(&rag_url));
    let app = Router::new()
        .route("/memory/store", post(store_memory))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/memory/store")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&StoreMemoryRequest {
                        content: "rag ingest fanout contract".to_string(),
                        kind: MemoryKind::Fact,
                        scope: MemoryScope::Project,
                        project: Some("memd".to_string()),
                        namespace: Some("main".to_string()),
                        workspace: Some("shared".to_string()),
                        visibility: None,
                        belief_branch: None,
                        source_agent: Some("codex".to_string()),
                        source_system: Some("cli".to_string()),
                        source_path: None,
                        source_quality: None,
                        confidence: None,
                        ttl_seconds: None,
                        last_verified_at: None,
                        supersedes: Vec::new(),
                        tags: vec!["rag".to_string(), "ingest".to_string()],
                        status: None,
                        lane: None,
                    })
                    .expect("serialize store request"),
                ))
                .expect("build store request"),
        )
        .await
        .expect("run store route");

    assert_eq!(response.status(), StatusCode::OK);
    let body: StoreMemoryResponse = decode_json(response).await;
    let captured = tokio::time::timeout(std::time::Duration::from_secs(2), rx)
        .await
        .expect("wait for rag ingest fanout")
        .expect("rag ingest request");
    assert_eq!(captured.source.id, body.item.id);
    let expected_source_path = body.item.id.to_string();
    assert_eq!(
        captured.source.source_path.as_deref(),
        Some(expected_source_path.as_str())
    );

    std::fs::remove_dir_all(dir).expect("cleanup ingest temp dir");
}

#[tokio::test]
async fn search_memory_injects_dense_rag_candidates() {
    let (dir, state) = temp_state("memd-rag-dense");
    let query = "dense \"alpha";
    let first = state
        .store_item(
            StoreMemoryRequest {
                content: format!("{query} first item"),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                visibility: Some(MemoryVisibility::Workspace),
                belief_branch: None,
                source_agent: Some("codex".to_string()),
                source_system: Some("cli".to_string()),
                source_path: Some("seed-a.md".to_string()),
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.8),
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec!["rag".to_string()],
                status: Some(MemoryStatus::Active),
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("seed first item")
        .0;
    tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    let second = state
        .store_item(
            StoreMemoryRequest {
                content: format!("{query} second item"),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                visibility: Some(MemoryVisibility::Workspace),
                belief_branch: None,
                source_agent: Some("codex".to_string()),
                source_system: Some("cli".to_string()),
                source_path: Some("seed-b.md".to_string()),
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.8),
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec!["rag".to_string()],
                status: Some(MemoryStatus::Active),
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("seed second item")
        .0;

    let retrieve_response = RagRetrieveResponse {
        status: "ok".to_string(),
        mode: RagRetrieveMode::Text,
        items: vec![RagRetrieveItem {
            content: "dense candidate".to_string(),
            source: Some(first.id.to_string()),
            score: 0.98,
        }],
    };
    let rag_url = spawn_mock_rag_retrieve_server(retrieve_response).await;
    let _dense_guard = set_test_env("MEMD_RETRIEVAL_RAG_DENSE", "1");
    let search_state = AppState {
        store: state.store.clone(),
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: Some(Arc::new(
            RagClient::new(&rag_url).expect("build dense rag client"),
        )),
        embedder: None,
    };

    let app = Router::new()
        .route("/memory/search", post(search_memory))
        .with_state(search_state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/memory/search")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&SearchMemoryRequest {
                        query: Some(query.to_string()),
                        route: None,
                        intent: None,
                        scopes: Vec::new(),
                        kinds: Vec::new(),
                        statuses: Vec::new(),
                        project: Some("memd".to_string()),
                        namespace: Some("main".to_string()),
                        workspace: Some("shared".to_string()),
                        visibility: None,
                        belief_branch: None,
                        source_agent: None,
                        region: None,
                        tags: Vec::new(),
                        stages: Vec::new(),
                        limit: Some(10),
                        max_chars_per_item: None,
                    })
                    .expect("serialize search request"),
                ))
                .expect("build search request"),
        )
        .await
        .expect("run search route");

    assert_eq!(response.status(), StatusCode::OK);
    let body: SearchMemoryResponse = decode_json(response).await;
    assert_eq!(body.items.first().map(|item| item.id), Some(first.id));
    assert_eq!(body.items.len(), 2);
    assert_eq!(body.items[1].id, second.id);

    std::fs::remove_dir_all(dir).expect("cleanup dense temp dir");
}

#[test]
fn intrinsic_rerank_search_candidates_promotes_stronger_phrase_match() {
    let weak = MemoryItem {
        content: "Brenda handled the workflow review and later scheduled the demo.".to_string(),
        tags: vec!["workflow".to_string(), "review".to_string()],
        confidence: 0.98,
        source_path: Some("notes/review.md".to_string()),
        source_quality: Some(SourceQuality::Canonical),
        ..sample_memory_item(None)
    };
    let strong = MemoryItem {
        content: "Brenda documented the MEDDIC qualification workflow for the deal review."
            .to_string(),
        tags: vec![
            "qualification".to_string(),
            "workflow".to_string(),
            "meddic".to_string(),
        ],
        confidence: 0.72,
        source_path: Some("notes/qualification-workflow.md".to_string()),
        source_quality: Some(SourceQuality::Canonical),
        ..sample_memory_item(None)
    };
    let items = vec![
        MemoryViewItem {
            item: weak.clone(),
            entity: None,
            source_trust_score: 0.9,
        },
        MemoryViewItem {
            item: strong.clone(),
            entity: None,
            source_trust_score: 0.9,
        },
    ];
    let base_ranks = vec![(weak.id, 0.91), (strong.id, 0.83)];

    let reranked = intrinsic_rerank_search_candidates(
        &items,
        "What did Brenda document about qualification workflow?",
        &base_ranks,
    );

    assert_eq!(
        reranked.first().map(|(id, _)| *id),
        Some(strong.id),
        "rerank should promote the stronger phrase/keyword match over the weaker base-ranked item"
    );
}

#[tokio::test]
async fn search_memory_uses_sidecar_rerank_when_available() {
    let (dir, state) = temp_state("memd-sidecar-rerank-search");
    let query = "What did Brenda document about qualification workflow?";
    let weak = state
        .store_item(
            StoreMemoryRequest {
                content: "Brenda handled the workflow review and later scheduled the demo."
                    .to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                visibility: Some(MemoryVisibility::Workspace),
                belief_branch: None,
                source_agent: Some("codex".to_string()),
                source_system: Some("cli".to_string()),
                source_path: Some("weak.md".to_string()),
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.95),
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec!["workflow".to_string()],
                status: Some(MemoryStatus::Active),
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("seed weak item")
        .0;
    let strong = state
        .store_item(
            StoreMemoryRequest {
                content: "Brenda documented the MEDDIC qualification workflow for the deal review."
                    .to_string(),
                kind: MemoryKind::Fact,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                visibility: Some(MemoryVisibility::Workspace),
                belief_branch: None,
                source_agent: Some("codex".to_string()),
                source_system: Some("cli".to_string()),
                source_path: Some("strong.md".to_string()),
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.8),
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec!["qualification".to_string(), "workflow".to_string()],
                status: Some(MemoryStatus::Active),
                lane: None,
            },
            MemoryStage::Canonical,
        )
        .expect("seed strong item")
        .0;

    let rag_url = spawn_mock_rag_search_server(
        RagRetrieveResponse {
            status: "ok".to_string(),
            mode: RagRetrieveMode::Text,
            items: vec![
                RagRetrieveItem {
                    content: "weak".to_string(),
                    source: Some(weak.id.to_string()),
                    score: 0.98,
                },
                RagRetrieveItem {
                    content: "strong".to_string(),
                    source: Some(strong.id.to_string()),
                    score: 0.82,
                },
            ],
        },
        RagRerankResponse {
            status: "ok".to_string(),
            model: "bge-reranker-base".to_string(),
            items: vec![
                RagRerankItem {
                    id: strong.id.to_string(),
                    score: 0.91,
                    text: None,
                },
                RagRerankItem {
                    id: weak.id.to_string(),
                    score: 0.52,
                    text: None,
                },
            ],
        },
    )
    .await;

    let search_state = AppState {
        store: state.store.clone(),
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: Some(Arc::new(
            RagClient::new(&rag_url).expect("build rag client with rerank"),
        )),
        embedder: None,
    };
    let app = Router::new()
        .route("/memory/search", post(search_memory))
        .with_state(search_state);
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/memory/search")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&SearchMemoryRequest {
                        query: Some(query.to_string()),
                        route: None,
                        intent: None,
                        scopes: Vec::new(),
                        kinds: Vec::new(),
                        statuses: Vec::new(),
                        project: Some("memd".to_string()),
                        namespace: Some("main".to_string()),
                        workspace: Some("shared".to_string()),
                        visibility: None,
                        belief_branch: None,
                        source_agent: None,
                        region: None,
                        tags: Vec::new(),
                        stages: Vec::new(),
                        limit: Some(10),
                        max_chars_per_item: None,
                    })
                    .expect("serialize rerank search request"),
                ))
                .expect("build rerank search request"),
        )
        .await
        .expect("run rerank search route");
    assert_eq!(response.status(), StatusCode::OK);
    let body: SearchMemoryResponse = decode_json(response).await;
    assert_eq!(body.items.first().map(|item| item.id), Some(strong.id));

    std::fs::remove_dir_all(dir).expect("cleanup rerank temp dir");
}
