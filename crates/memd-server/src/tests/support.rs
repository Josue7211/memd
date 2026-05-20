use super::*;

pub(super) fn sample_memory_item(workspace: Option<&str>) -> MemoryItem {
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

pub(super) fn temp_state(name: &str) -> (std::path::PathBuf, AppState) {
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

pub(super) fn temp_state_with_rag(
    name: &str,
    rag_url: Option<&str>,
) -> (std::path::PathBuf, AppState) {
    let (dir, mut state) = temp_state(name);
    state.rag = rag_url.map(|url| Arc::new(RagClient::new(url).expect("build test rag client")));
    (dir, state)
}

pub(super) fn test_store_request(
    content: &str,
    project: &str,
    namespace: &str,
) -> StoreMemoryRequest {
    StoreMemoryRequest {
        content: content.to_string(),
        kind: MemoryKind::Fact,
        scope: MemoryScope::Project,
        project: Some(project.to_string()),
        namespace: Some(namespace.to_string()),
        workspace: Some("shared".to_string()),
        visibility: Some(MemoryVisibility::Workspace),
        belief_branch: None,
        source_agent: Some("codex".to_string()),
        source_system: Some("test".to_string()),
        source_path: None,
        source_quality: Some(SourceQuality::Canonical),
        confidence: Some(0.86),
        ttl_seconds: None,
        last_verified_at: Some(Utc::now()),
        supersedes: Vec::new(),
        tags: Vec::new(),
        status: Some(MemoryStatus::Active),
        lane: None,
    }
}

pub(super) fn test_search_request(
    query: &str,
    project: &str,
    namespace: &str,
) -> SearchMemoryRequest {
    SearchMemoryRequest {
        query: Some(query.to_string()),
        route: None,
        intent: Some(RetrievalIntent::CurrentTask),
        scopes: vec![MemoryScope::Project],
        kinds: Vec::new(),
        statuses: Vec::new(),
        project: Some(project.to_string()),
        namespace: Some(namespace.to_string()),
        workspace: Some("shared".to_string()),
        visibility: None,
        belief_branch: None,
        source_agent: Some("codex".to_string()),
        region: None,
        tags: Vec::new(),
        stages: vec![MemoryStage::Canonical],
        limit: Some(10),
        max_chars_per_item: None,
    }
}

pub(super) fn set_env(name: &str, value: &str) {
    unsafe {
        std::env::set_var(name, value);
    }
}

pub(super) fn remove_env(name: &str) {
    unsafe {
        std::env::remove_var(name);
    }
}

pub(super) struct EnvGuard(&'static str);

impl Drop for EnvGuard {
    fn drop(&mut self) {
        remove_env(self.0);
    }
}

pub(super) fn set_test_env(name: &'static str, value: &str) -> EnvGuard {
    set_env(name, value);
    EnvGuard(name)
}

#[derive(Clone)]
pub(super) struct RagIngestCaptureState {
    ingest_tx: Arc<Mutex<Option<tokio::sync::oneshot::Sender<RagIngestRequest>>>>,
}

pub(super) async fn mock_rag_healthz() -> Json<RagBackendHealthResponse> {
    Json(RagBackendHealthResponse {
        status: "ok".to_string(),
        backend: RagBackendHealth {
            connected: true,
            name: Some("rag-sidecar".to_string()),
            multimodal: true,
            profile: Some("sparse".to_string()),
            indexed_count: Some(0),
        },
    })
}

pub(super) async fn mock_rag_ingest(
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

pub(super) async fn mock_rag_retrieve(
    State(response): State<RagRetrieveResponse>,
) -> Json<RagRetrieveResponse> {
    Json(response)
}

pub(super) async fn mock_rag_retrieve_from_state(
    State(state): State<MockRagSearchState>,
) -> Json<RagRetrieveResponse> {
    Json(state.retrieve)
}

#[derive(Clone)]
pub(super) struct MockRagSearchState {
    retrieve: RagRetrieveResponse,
    rerank: RagRerankResponse,
}

pub(super) async fn mock_rag_rerank(
    State(state): State<MockRagSearchState>,
) -> Json<RagRerankResponse> {
    Json(state.rerank)
}

#[derive(Clone)]
pub(super) struct MockRagQueryCorpusState {
    by_query: Arc<std::collections::BTreeMap<String, Vec<RagRetrieveItem>>>,
}

pub(super) async fn mock_rag_retrieve_query_corpus(
    State(state): State<MockRagQueryCorpusState>,
    Json(req): Json<RagRetrieveRequest>,
) -> Json<RagRetrieveResponse> {
    let items = state
        .by_query
        .get(req.query.trim())
        .cloned()
        .unwrap_or_default();
    Json(RagRetrieveResponse {
        status: "ok".to_string(),
        mode: RagRetrieveMode::Text,
        items,
    })
}

pub(super) async fn mock_rag_rerank_query_corpus(
    State(state): State<MockRagQueryCorpusState>,
    Json(req): Json<RagRerankRequest>,
) -> Json<RagRerankResponse> {
    let preferred = state
        .by_query
        .get(req.query.trim())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.source.as_deref())
                .collect::<std::collections::BTreeSet<_>>()
        })
        .unwrap_or_default();
    let mut items = req
        .candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| RagRerankItem {
            id: candidate.id.clone(),
            score: if preferred.contains(candidate.id.as_str()) {
                1.0 - (index as f32 * 0.001)
            } else {
                0.5 - (index as f32 * 0.001)
            },
            text: None,
        })
        .collect::<Vec<_>>();
    items.sort_by(|a, b| b.score.total_cmp(&a.score));
    Json(RagRerankResponse {
        status: "ok".to_string(),
        model: "mock-query-corpus-reranker".to_string(),
        items,
    })
}

pub(super) async fn spawn_mock_rag_ingest_server()
-> (String, tokio::sync::oneshot::Receiver<RagIngestRequest>) {
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

pub(super) async fn spawn_mock_rag_retrieve_server(response: RagRetrieveResponse) -> String {
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

pub(super) async fn spawn_mock_rag_search_server(
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

pub(super) async fn spawn_mock_rag_query_corpus_server(
    by_query: std::collections::BTreeMap<String, Vec<RagRetrieveItem>>,
) -> String {
    let app = Router::new()
        .route("/v1/retrieve", post(mock_rag_retrieve_query_corpus))
        .route("/v1/rerank", post(mock_rag_rerank_query_corpus))
        .with_state(MockRagQueryCorpusState {
            by_query: Arc::new(by_query),
        });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind mock rag query corpus server");
    let addr = listener.local_addr().expect("mock rag query corpus addr");
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("serve mock rag query corpus server");
    });
    tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    format!("http://{}", addr)
}

pub(super) fn test_hive_router(state: AppState) -> Router {
    Router::new()
        .route("/hive/board", get(get_hive_board))
        .route("/hive/roster", get(get_hive_roster))
        .route("/hive/follow", get(get_hive_follow))
        .with_state(state)
}

pub(super) fn seed_hive_route_state(state: &AppState) {
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

pub(super) fn sample_dev_server_lease(scope: &str, session: &str) -> DevServerLeaseAcquireRequest {
    DevServerLeaseAcquireRequest {
        scope: scope.to_string(),
        host: "127.0.0.1".to_string(),
        port: 43210,
        url: "http://127.0.0.1:43210".to_string(),
        repo_root: "/tmp/memd".to_string(),
        repo_hash: "repo1234".to_string(),
        command: vec!["npm".to_string(), "run".to_string(), "dev".to_string()],
        session: session.to_string(),
        tab_id: None,
        agent: Some("codex".to_string()),
        effective_agent: Some(session.to_string()),
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        workspace: Some("shared".to_string()),
        host_name: Some("workstation".to_string()),
        pid: Some(4242),
        ttl_seconds: 600,
        recover_stale: true,
        stale_after_seconds: 30,
    }
}

pub(super) async fn decode_json<T: serde::de::DeserializeOwned>(
    response: axum::response::Response,
) -> T {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read response body");
    serde_json::from_slice(&body).expect("decode response json")
}
