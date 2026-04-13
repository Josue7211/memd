use super::*;
use crate::test_support::{EnvScope, set_current_dir};
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::{get, post},
};
use memd_schema::{
    CompactContextResponse, CompactMemoryRecord, CompactionDecision, CompactionOpenLoop,
    CompactionPacket, CompactionReference, CompactionSession, MemoryItem, MemoryScope, MemoryStage,
    MemoryStatus, MemoryVisibility, RetrievalIntent, RetrievalRoute, SearchMemoryResponse,
    SourceMemoryRequest, SourceQuality, WorkingMemoryPolicyState, WorkingMemoryResponse,
};
use std::sync::{Arc, Mutex, OnceLock};

static E2E_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn lock_e2e() -> std::sync::MutexGuard<'static, ()> {
    E2E_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
}

#[derive(Clone, Default)]
struct LookupState {
    requests: Arc<Mutex<Vec<SearchMemoryRequest>>>,
}

#[derive(Clone, Default)]
struct SpillState {
    candidates: Arc<Mutex<Vec<memd_schema::CandidateMemoryRequest>>>,
    stored: Arc<Mutex<Vec<memd_schema::StoreMemoryRequest>>>,
    source_requests: Arc<Mutex<Vec<memd_schema::SourceMemoryRequest>>>,
    context_compact_response: Arc<Mutex<Option<CompactContextResponse>>>,
    working_response: Arc<Mutex<Option<WorkingMemoryResponse>>>,
}

async fn healthz() -> &'static str {
    "ok"
}

async fn search_memory(
    State(state): State<LookupState>,
    Json(req): Json<SearchMemoryRequest>,
) -> Json<SearchMemoryResponse> {
    state
        .requests
        .lock()
        .expect("lock lookup requests")
        .push(req);
    Json(SearchMemoryResponse {
        route: RetrievalRoute::Auto,
        intent: RetrievalIntent::CurrentTask,
        items: vec![MemoryItem {
            id: uuid::Uuid::new_v4(),
            content: "repo-b answer".to_string(),
            redundancy_key: None,
            belief_branch: None,
            preferred: true,
            kind: MemoryKind::Fact,
            scope: MemoryScope::Project,
            project: Some("repo-b".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            visibility: MemoryVisibility::Workspace,
            source_agent: Some("codex".to_string()),
            source_system: Some("memd".to_string()),
            source_path: None,
            source_quality: Some(SourceQuality::Canonical),
            confidence: 0.99,
            ttl_seconds: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_verified_at: None,
            supersedes: Vec::new(),
            tags: vec!["repo-b".to_string()],
            status: MemoryStatus::Active,
            stage: MemoryStage::Canonical,
        }],
    })
}

async fn spawn_lookup_server(state: LookupState) -> String {
    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/memory/search", post(search_memory))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind mock lookup server");
    let addr = listener.local_addr().expect("mock server addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve mock lookup");
    });
    format!("http://{addr}")
}

async fn spawn_spill_server(state: SpillState) -> String {
    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/memory/search", post(mock_search_memory_for_spill))
        .route("/memory/candidates", post(mock_candidate_memory))
        .route("/memory/store", post(mock_store_memory))
        .route(
            "/memory/context/compact",
            get(mock_context_compact_for_spill),
        )
        .route("/memory/working", get(mock_working_memory_for_spill))
        .route("/memory/inbox", get(mock_inbox_for_spill))
        .route("/memory/workspaces", get(mock_workspace_memory_for_spill))
        .route("/memory/source", get(mock_source_memory_for_spill))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind spill server");
    let addr = listener.local_addr().expect("spill server addr");
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("serve spill server");
    });
    format!("http://{addr}")
}

async fn mock_search_memory_for_spill(
    State(_state): State<SpillState>,
    Json(_req): Json<SearchMemoryRequest>,
) -> Json<SearchMemoryResponse> {
    Json(SearchMemoryResponse {
        route: RetrievalRoute::Auto,
        intent: RetrievalIntent::CurrentTask,
        items: Vec::new(),
    })
}

async fn mock_candidate_memory(
    State(state): State<SpillState>,
    Json(req): Json<memd_schema::CandidateMemoryRequest>,
) -> Json<memd_schema::CandidateMemoryResponse> {
    state
        .candidates
        .lock()
        .expect("lock spill candidates")
        .push(req.clone());
    Json(memd_schema::CandidateMemoryResponse {
        item: memd_schema::MemoryItem {
            id: uuid::Uuid::new_v4(),
            content: req.content,
            redundancy_key: Some("candidate".to_string()),
            belief_branch: req.belief_branch,
            preferred: false,
            kind: req.kind,
            scope: req.scope,
            project: req.project,
            namespace: req.namespace,
            workspace: req.workspace,
            visibility: req.visibility.unwrap_or(MemoryVisibility::Private),
            source_agent: req.source_agent,
            source_system: req.source_system,
            source_path: req.source_path,
            source_quality: req.source_quality,
            confidence: req.confidence.unwrap_or(0.7),
            ttl_seconds: req.ttl_seconds,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_verified_at: req.last_verified_at,
            supersedes: req.supersedes,
            tags: req.tags,
            status: MemoryStatus::Active,
            stage: MemoryStage::Candidate,
        },
        duplicate_of: None,
    })
}

async fn mock_store_memory(
    State(state): State<SpillState>,
    Json(req): Json<memd_schema::StoreMemoryRequest>,
) -> Json<memd_schema::StoreMemoryResponse> {
    state
        .stored
        .lock()
        .expect("lock spill stored")
        .push(req.clone());
    Json(memd_schema::StoreMemoryResponse {
        item: memd_schema::MemoryItem {
            id: uuid::Uuid::new_v4(),
            content: req.content,
            redundancy_key: Some("stored".to_string()),
            belief_branch: req.belief_branch,
            preferred: false,
            kind: req.kind,
            scope: req.scope,
            project: req.project,
            namespace: req.namespace,
            workspace: req.workspace,
            visibility: req.visibility.unwrap_or(MemoryVisibility::Private),
            source_agent: req.source_agent,
            source_system: req.source_system,
            source_path: req.source_path,
            source_quality: req.source_quality,
            confidence: req.confidence.unwrap_or(0.7),
            ttl_seconds: req.ttl_seconds,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_verified_at: req.last_verified_at,
            supersedes: req.supersedes,
            tags: req.tags,
            status: req.status.unwrap_or(MemoryStatus::Active),
            stage: MemoryStage::Canonical,
        },
    })
}

async fn mock_context_compact_for_spill(
    State(state): State<SpillState>,
) -> Json<CompactContextResponse> {
    Json(
        state
            .context_compact_response
            .lock()
            .expect("lock spill context response")
            .clone()
            .unwrap_or(CompactContextResponse {
                route: RetrievalRoute::ProjectFirst,
                intent: RetrievalIntent::CurrentTask,
                retrieval_order: vec![MemoryScope::Project, MemoryScope::Synced],
                records: vec![CompactMemoryRecord {
                    id: uuid::Uuid::new_v4(),
                    record: "compact context: keep startup surfaces tight".to_string(),
                }],
            }),
    )
}

async fn mock_working_memory_for_spill(
    State(state): State<SpillState>,
) -> Json<WorkingMemoryResponse> {
    Json(
        state
            .working_response
            .lock()
            .expect("lock spill working response")
            .clone()
            .unwrap_or(WorkingMemoryResponse {
                route: RetrievalRoute::ProjectFirst,
                intent: RetrievalIntent::CurrentTask,
                retrieval_order: vec![MemoryScope::Project, MemoryScope::Synced],
                budget_chars: 1600,
                used_chars: 220,
                remaining_chars: 1380,
                truncated: false,
                policy: WorkingMemoryPolicyState {
                    admission_limit: 8,
                    max_chars_per_item: 220,
                    budget_chars: 1600,
                    rehydration_limit: 4,
                },
                records: vec![CompactMemoryRecord {
                    id: uuid::Uuid::new_v4(),
                    record: "working record: keep startup surfaces tight".to_string(),
                }],
                evicted: Vec::new(),
                rehydration_queue: Vec::new(),
                traces: Vec::new(),
                semantic_consolidation: None,
            }),
    )
}

async fn mock_inbox_for_spill() -> Json<memd_schema::MemoryInboxResponse> {
    Json(memd_schema::MemoryInboxResponse {
        route: RetrievalRoute::ProjectFirst,
        intent: RetrievalIntent::CurrentTask,
        items: Vec::new(),
    })
}

async fn mock_workspace_memory_for_spill() -> Json<memd_schema::WorkspaceMemoryResponse> {
    Json(memd_schema::WorkspaceMemoryResponse {
        workspaces: vec![memd_schema::WorkspaceMemoryRecord {
            project: Some("repo".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            visibility: MemoryVisibility::Workspace,
            item_count: 1,
            active_count: 1,
            candidate_count: 0,
            contested_count: 0,
            source_lane_count: 1,
            avg_confidence: 0.9,
            trust_score: 0.95,
            last_seen_at: Some(chrono::Utc::now()),
            tags: vec!["spill".to_string()],
        }],
    })
}

async fn mock_source_memory_for_spill(
    State(state): State<SpillState>,
    Query(req): Query<memd_schema::SourceMemoryRequest>,
) -> Json<memd_schema::SourceMemoryResponse> {
    state
        .source_requests
        .lock()
        .expect("lock source requests")
        .push(req);
    Json(memd_schema::SourceMemoryResponse {
        sources: Vec::new(),
    })
}

#[tokio::test]
async fn lookup_cli_defaults_stay_on_repo_b_bundle_against_live_memory_server() {
    let _guard = lock_e2e();
    let mut env = EnvScope::new();
    let temp_root = std::env::temp_dir().join(format!("memd-lookup-e2e-{}", uuid::Uuid::new_v4()));
    let home = temp_root.join("home");
    let repo_a = temp_root.join("repo-a");
    let repo_b = temp_root.join("repo-b");
    let repo_b_nested = repo_b.join("src").join("feature");
    let global_root = home.join(".memd");
    let local_bundle = repo_b.join(".memd");
    let state = LookupState::default();
    let base_url = spawn_lookup_server(state.clone()).await;

    fs::create_dir_all(global_root.join("state")).expect("create global state");
    fs::create_dir_all(repo_a.join(".git")).expect("create repo a");
    fs::create_dir_all(repo_b.join(".git")).expect("create repo b");
    fs::create_dir_all(&repo_b_nested).expect("create nested repo b dir");
    fs::create_dir_all(local_bundle.join("state")).expect("create local state");
    fs::write(
        global_root.join("config.json"),
        format!(
            r#"{{
  "project": "repo-a",
  "namespace": "main",
  "agent": "codex",
  "session": "repo-a-session",
  "base_url": "{base_url}"
}}
"#
        ),
    )
    .expect("write global config");
    fs::write(
        local_bundle.join("config.json"),
        format!(
            r#"{{
  "project": "repo-b",
  "namespace": "main",
  "agent": "codex",
  "session": "repo-b-session",
  "base_url": "{base_url}"
}}
"#
        ),
    )
    .expect("write local config");

    env.set("HOME", &home);
    env.remove("MEMD_BUNDLE_ROOT");
    let _cwd = set_current_dir(&repo_b_nested);
    let output = default_bundle_root_path();
    assert_eq!(output, local_bundle);
    let args = LookupArgs {
        output,
        query: "repo bleed".to_string(),
        project: None,
        namespace: None,
        workspace: None,
        visibility: None,
        route: None,
        intent: None,
        kind: Vec::new(),
        tag: Vec::new(),
        include_stale: false,
        limit: None,
        verbose: false,
        json: false,
    };

    let client = MemdClient::new(&base_url).expect("client");
    run_lookup_command(&client, args).await.expect("run lookup");

    let search_requests = state.requests.lock().expect("lock lookup requests");
    assert_eq!(search_requests.len(), 1);
    assert_eq!(search_requests[0].project.as_deref(), Some("repo-b"));
    assert_eq!(search_requests[0].namespace.as_deref(), Some("main"));
    assert_ne!(search_requests[0].project.as_deref(), Some("repo-a"));

    drop(_cwd);
    drop(env);
    fs::remove_dir_all(temp_root).expect("cleanup lookup e2e temp");
}

#[tokio::test]
async fn hook_spill_apply_writes_candidates_and_compaction_checkpoint() {
    let _guard = lock_e2e();
    let mut env = EnvScope::new();
    let temp_root = std::env::temp_dir().join(format!("memd-spill-e2e-{}", uuid::Uuid::new_v4()));
    let home = temp_root.join("home");
    let repo_root = temp_root.join("repo");
    let bundle_root = repo_root.join(".memd");
    let spill_state = SpillState::default();
    let base_url = spawn_spill_server(spill_state.clone()).await;

    fs::create_dir_all(repo_root.join(".git")).expect("create repo");
    fs::create_dir_all(bundle_root.join("state")).expect("create bundle state");
    fs::write(
        bundle_root.join("config.json"),
        format!(
            r#"{{
  "project": "repo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-live",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "{base_url}",
  "auto_short_term_capture": true,
  "route": "auto",
  "intent": "current_task"
}}
"#
        ),
    )
    .expect("write bundle config");

    let packet = CompactionPacket {
        session: CompactionSession {
            project: Some("repo".to_string()),
            agent: Some("codex".to_string()),
            task: "finish spill proof".to_string(),
        },
        goal: "prove compaction spill writes".to_string(),
        hard_constraints: vec!["keep the proof live".to_string()],
        active_work: vec!["wire spill end to end".to_string()],
        decisions: vec![CompactionDecision {
            id: "decision-1".to_string(),
            text: "use live checkpoint".to_string(),
        }],
        open_loops: vec![CompactionOpenLoop {
            id: "loop-1".to_string(),
            text: "confirm resume refresh".to_string(),
            status: "open".to_string(),
        }],
        exact_refs: vec![CompactionReference {
            kind: "file".to_string(),
            value: "crates/memd-client/src/cli/cli_hook_runtime.rs".to_string(),
        }],
        next_actions: vec!["write the regression".to_string()],
        do_not_drop: vec!["config change must stay one edit".to_string()],
        memory: CompactContextResponse {
            route: RetrievalRoute::ProjectFirst,
            intent: RetrievalIntent::CurrentTask,
            retrieval_order: vec![MemoryScope::Project, MemoryScope::Synced],
            records: vec![CompactMemoryRecord {
                id: uuid::Uuid::new_v4(),
                record: "compact context: keep startup surfaces tight".to_string(),
            }],
        },
    };

    env.set("HOME", &home);
    env.set("MEMD_BUNDLE_ROOT", &bundle_root);
    let _cwd = set_current_dir(&repo_root);

    let client = MemdClient::new(&base_url).expect("client");
    run_hook_mode(
        &client,
        &base_url,
        HookArgs {
            mode: HookMode::Spill(HookSpillArgs {
                input: RequestInput {
                    json: Some(serde_json::to_string_pretty(&packet).expect("serialize packet")),
                    input: None,
                    stdin: false,
                },
                apply: true,
                spill_transient: true,
            }),
        },
    )
    .await
    .expect("run hook spill");

    let candidates = spill_state.candidates.lock().expect("lock candidates");
    assert!(
        !candidates.is_empty(),
        "spill should submit candidate writes"
    );
    assert_eq!(candidates[0].namespace.as_deref(), Some("compaction"));
    assert!(
        candidates
            .iter()
            .any(|req| req.tags.iter().any(|tag| tag == "compaction"))
    );
    drop(candidates);

    let stored = spill_state.stored.lock().expect("lock stored");
    assert!(
        !stored.is_empty(),
        "spill apply should checkpoint the compaction packet"
    );
    assert_eq!(stored[0].source_path.as_deref(), Some("compaction"));
    assert_eq!(stored[0].source_system.as_deref(), Some("memd-short-term"));
    assert!(
        stored[0].content.contains("finish spill proof")
            || stored[0]
                .content
                .contains("goal: prove compaction spill writes")
    );
    drop(stored);

    let memory = fs::read_to_string(bundle_root.join("mem.md"))
        .expect("read generated bundle memory");
    let wakeup = fs::read_to_string(bundle_root.join("wake.md"))
        .expect("read generated bundle wakeup");
    assert!(memory.contains("compact context: keep startup surfaces tight"));
    assert!(memory.contains("working record: keep startup surfaces tight"));
    assert!(wakeup.contains("compact context: keep startup surfaces tight"));
    assert!(wakeup.contains("working record: keep startup surfaces tight"));

    drop(_cwd);
    drop(env);
    fs::remove_dir_all(temp_root).expect("cleanup spill e2e temp");
}

#[tokio::test]
async fn resume_command_surfaces_compact_working_state_from_live_server() {
    let _guard = lock_e2e();
    let temp_root = std::env::temp_dir().join(format!("memd-resume-e2e-{}", uuid::Uuid::new_v4()));
    let home = temp_root.join("home");
    let repo_root = temp_root.join("repo");
    let bundle_root = repo_root.join(".memd");
    let spill_state = SpillState::default();
    let base_url = spawn_spill_server(spill_state.clone()).await;
    let mut env = EnvScope::new();

    fs::create_dir_all(repo_root.join(".git")).expect("create repo");
    fs::create_dir_all(bundle_root.join("state")).expect("create bundle state");
    fs::write(
        bundle_root.join("config.json"),
        format!(
            r#"{{
  "project": "repo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-live",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "{base_url}",
  "auto_short_term_capture": true,
  "route": "auto",
  "intent": "current_task"
}}
"#
        ),
    )
    .expect("write bundle config");

    *spill_state
        .context_compact_response
        .lock()
        .expect("lock resume context response") = Some(CompactContextResponse {
        route: RetrievalRoute::ProjectFirst,
        intent: RetrievalIntent::CurrentTask,
        retrieval_order: vec![MemoryScope::Project, MemoryScope::Synced],
        records: vec![CompactMemoryRecord {
            id: uuid::Uuid::new_v4(),
            record: "compact context: resume should surface live compact state".to_string(),
        }],
    });
    *spill_state
        .working_response
        .lock()
        .expect("lock resume working response") = Some(WorkingMemoryResponse {
        route: RetrievalRoute::ProjectFirst,
        intent: RetrievalIntent::CurrentTask,
        retrieval_order: vec![MemoryScope::Project, MemoryScope::Synced],
        budget_chars: 1600,
        used_chars: 260,
        remaining_chars: 1340,
        truncated: false,
        policy: WorkingMemoryPolicyState {
            admission_limit: 8,
            max_chars_per_item: 220,
            budget_chars: 1600,
            rehydration_limit: 4,
        },
        records: vec![CompactMemoryRecord {
            id: uuid::Uuid::new_v4(),
            record: "working record: resume should surface live compact state".to_string(),
        }],
        evicted: Vec::new(),
        rehydration_queue: Vec::new(),
        traces: Vec::new(),
        semantic_consolidation: None,
    });

    env.set("HOME", &home);
    let _cwd = set_current_dir(&repo_root);

    let args = ResumeArgs {
        output: bundle_root.clone(),
        project: None,
        namespace: None,
        agent: None,
        workspace: None,
        visibility: None,
        route: None,
        intent: None,
        limit: None,
        rehydration_limit: None,
        semantic: false,
        prompt: false,
        summary: true,
    };
    let snapshot = crate::runtime::read_bundle_resume(&args, &base_url)
        .await
        .expect("read resume");
    assert_eq!(snapshot.context.records.len(), 1);
    assert_eq!(snapshot.working.records.len(), 1);
    assert_eq!(
        snapshot.context.records[0].record,
        "compact context: resume should surface live compact state"
    );
    assert_eq!(
        snapshot.working.records[0].record,
        "working record: resume should surface live compact state"
    );

    let prompt = crate::render::render_resume_prompt(&snapshot);
    assert!(prompt.contains("working record: resume should surface live compact state"));
    assert!(prompt.contains("## W"));
    assert!(prompt.contains("- doing="));
    assert!(prompt.contains("- left_off="));
    assert!(prompt.contains("- changed="));

    drop(_cwd);
    drop(env);
    fs::remove_dir_all(temp_root).expect("cleanup resume e2e temp");
}

#[tokio::test]
async fn source_command_uses_live_source_route_and_bundle_defaults() {
    let _guard = lock_e2e();
    let mut env = EnvScope::new();
    let temp_root = std::env::temp_dir().join(format!("memd-source-e2e-{}", uuid::Uuid::new_v4()));
    let home = temp_root.join("home");
    let repo_root = temp_root.join("repo");
    let bundle_root = repo_root.join(".memd");
    let spill_state = SpillState::default();
    let base_url = spawn_spill_server(spill_state.clone()).await;

    fs::create_dir_all(repo_root.join(".git")).expect("create repo");
    fs::create_dir_all(bundle_root.join("state")).expect("create bundle state");
    fs::write(
        bundle_root.join("config.json"),
        format!(
            r#"{{
  "project": "repo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-live",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "{base_url}",
  "auto_short_term_capture": true,
  "route": "auto",
  "intent": "current_task"
}}
"#
        ),
    )
    .expect("write bundle config");

    *spill_state
        .source_requests
        .lock()
        .expect("lock source requests") = Vec::new();

    env.set("HOME", &home);
    let _cwd = set_current_dir(&repo_root);

    let client = MemdClient::new(&base_url).expect("client");
    run_source_command(
        &client,
        SourceArgs {
            project: None,
            namespace: None,
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            source_agent: Some("codex".to_string()),
            source_system: Some("memd".to_string()),
            limit: Some(5),
            summary: true,
            follow: true,
        },
    )
    .await
    .expect("run source");

    let requests = spill_state
        .source_requests
        .lock()
        .expect("lock source requests");
    assert_eq!(
        requests.len(),
        1,
        "source command should hit live source route"
    );
    assert_eq!(requests[0].project.as_deref(), Some("repo"));
    assert_eq!(requests[0].namespace.as_deref(), Some("main"));
    assert_eq!(requests[0].workspace.as_deref(), Some("shared"));
    assert_eq!(requests[0].visibility, Some(MemoryVisibility::Workspace),);
    assert_eq!(requests[0].source_agent.as_deref(), Some("codex"));
    assert_eq!(requests[0].source_system.as_deref(), Some("memd"));
    assert_eq!(requests[0].limit, Some(5));

    drop(_cwd);
    drop(env);
    fs::remove_dir_all(temp_root).expect("cleanup source e2e temp");
}

#[tokio::test]
async fn bundle_voice_mode_changes_in_one_edit_and_updates_runtime_surfaces() {
    let _guard = lock_e2e();
    let temp_root = std::env::temp_dir().join(format!("memd-voice-e2e-{}", uuid::Uuid::new_v4()));
    let bundle_root = temp_root.join(".memd");

    fs::create_dir_all(bundle_root.join("state")).expect("create bundle state");
    fs::write(
        bundle_root.join("config.json"),
        "{\n  \"voice_mode\": \"caveman-ultra\"\n}\n",
    )
    .expect("seed config");

    crate::bundle::set_bundle_voice_mode(&bundle_root, "normal").expect("set voice mode");

    let config = fs::read_to_string(bundle_root.join("config.json")).expect("read config");
    assert!(config.contains("\"voice_mode\": \"normal\""));

    let env = fs::read_to_string(bundle_root.join("env")).expect("read env");
    assert!(env.contains("MEMD_VOICE_MODE=normal"));

    let env_ps1 = fs::read_to_string(bundle_root.join("env.ps1")).expect("read env.ps1");
    assert!(env_ps1.contains("$env:MEMD_VOICE_MODE = \"normal\""));

    let wakeup = render_bundle_wakeup_markdown(&bundle_root, &sample_snapshot_for_voice(), false);
    assert!(wakeup.contains("Default voice: normal"));
    assert!(wakeup.contains("Reply in `normal`"));

    fs::remove_dir_all(temp_root).expect("cleanup voice e2e temp");
}

fn sample_snapshot_for_voice() -> ResumeSnapshot {
    ResumeSnapshot {
        project: Some("repo".to_string()),
        namespace: Some("main".to_string()),
        agent: Some("codex".to_string()),
        workspace: Some("shared".to_string()),
        visibility: Some("workspace".to_string()),
        route: "auto".to_string(),
        intent: "current_task".to_string(),
        context: CompactContextResponse {
            route: RetrievalRoute::ProjectFirst,
            intent: RetrievalIntent::CurrentTask,
            retrieval_order: vec![MemoryScope::Project, MemoryScope::Synced],
            records: vec![CompactMemoryRecord {
                id: uuid::Uuid::new_v4(),
                record: "startup truth".to_string(),
            }],
        },
        working: WorkingMemoryResponse {
            route: RetrievalRoute::ProjectFirst,
            intent: RetrievalIntent::CurrentTask,
            retrieval_order: vec![MemoryScope::Project, MemoryScope::Synced],
            budget_chars: 1600,
            used_chars: 120,
            remaining_chars: 1480,
            truncated: false,
            policy: WorkingMemoryPolicyState {
                admission_limit: 8,
                max_chars_per_item: 220,
                budget_chars: 1600,
                rehydration_limit: 4,
            },
            records: vec![CompactMemoryRecord {
                id: uuid::Uuid::new_v4(),
                record: "working truth".to_string(),
            }],
            evicted: Vec::new(),
            rehydration_queue: Vec::new(),
            traces: Vec::new(),
            semantic_consolidation: None,
        },
        inbox: memd_schema::MemoryInboxResponse {
            route: RetrievalRoute::ProjectFirst,
            intent: RetrievalIntent::CurrentTask,
            items: Vec::new(),
        },
        workspaces: memd_schema::WorkspaceMemoryResponse {
            workspaces: Vec::new(),
        },
        sources: memd_schema::SourceMemoryResponse {
            sources: Vec::new(),
        },
        semantic: None,
        claims: SessionClaimsState::default(),
        recent_repo_changes: Vec::new(),
        change_summary: Vec::new(),
        resume_state_age_minutes: None,
        refresh_recommended: false,
    }
}
