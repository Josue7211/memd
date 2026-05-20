use super::*;

fn offline_test_request(content: &str) -> memd_schema::StoreMemoryRequest {
    memd_schema::StoreMemoryRequest {
        content: content.to_string(),
        kind: MemoryKind::Fact,
        scope: MemoryScope::Project,
        project: Some("memd-offline-proof".to_string()),
        namespace: Some("main".to_string()),
        workspace: Some("shared".to_string()),
        visibility: Some(memd_schema::MemoryVisibility::Workspace),
        belief_branch: None,
        source_agent: Some("codex@test".to_string()),
        source_system: Some("offline-test".to_string()),
        source_path: None,
        source_quality: Some(memd_schema::SourceQuality::Canonical),
        confidence: Some(0.9),
        ttl_seconds: None,
        last_verified_at: None,
        supersedes: Vec::new(),
        tags: vec!["offline".to_string()],
        status: Some(MemoryStatus::Active),
        lane: None,
    }
}

fn offline_candidate_request(content: &str) -> memd_schema::CandidateMemoryRequest {
    memd_schema::CandidateMemoryRequest {
        content: content.to_string(),
        kind: MemoryKind::Status,
        scope: MemoryScope::Project,
        project: Some("memd-offline-proof".to_string()),
        namespace: Some("main".to_string()),
        workspace: Some("shared".to_string()),
        visibility: Some(memd_schema::MemoryVisibility::Workspace),
        belief_branch: None,
        source_agent: Some("codex@test".to_string()),
        source_system: Some("hook-spill".to_string()),
        source_path: Some("compaction-packet".to_string()),
        source_quality: Some(memd_schema::SourceQuality::Derived),
        confidence: Some(0.6),
        ttl_seconds: Some(86_400),
        last_verified_at: None,
        supersedes: Vec::new(),
        tags: vec!["hook-spill".to_string(), "offline".to_string()],
        lane: None,
    }
}

fn capability_record_fixture(name: &str) -> memd_schema::CapabilityRecord {
    memd_schema::CapabilityRecord {
        harness: "codex".to_string(),
        kind: "skill".to_string(),
        name: name.to_string(),
        status: "installed".to_string(),
        portability_class: "harness-native".to_string(),
        source_path: format!("/tmp/{name}/SKILL.md"),
        bridge_hint: None,
        hash: None,
        notes: Vec::new(),
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        workspace: None,
        user_id: None,
        agent: Some("codex".to_string()),
        updated_at: None,
    }
}

fn offline_remember_args(output: PathBuf, content: &str) -> RememberArgs {
    RememberArgs {
        output,
        project: Some("memd-offline-proof".to_string()),
        namespace: Some("main".to_string()),
        workspace: Some("shared".to_string()),
        visibility: Some("workspace".to_string()),
        kind: Some("fact".to_string()),
        scope: Some("project".to_string()),
        source_agent: Some("codex@test".to_string()),
        source_system: Some("offline-test".to_string()),
        source_path: None,
        source_quality: Some("canonical".to_string()),
        confidence: Some(0.9),
        ttl_seconds: None,
        tag: vec!["offline".to_string()],
        supersede: Vec::new(),
        content: Some(content.to_string()),
        input: None,
        stdin: false,
    }
}

async fn spawn_offline_store_server() -> String {
    use axum::{Json, Router, routing::post};

    async fn store(
        Json(req): Json<memd_schema::StoreMemoryRequest>,
    ) -> Json<memd_schema::StoreMemoryResponse> {
        let now = chrono::Utc::now();
        Json(memd_schema::StoreMemoryResponse {
            item: memd_schema::MemoryItem {
                id: Uuid::new_v4(),
                content: req.content,
                redundancy_key: None,
                belief_branch: req.belief_branch,
                preferred: false,
                kind: req.kind,
                scope: req.scope,
                project: req.project,
                namespace: req.namespace,
                workspace: req.workspace,
                visibility: req.visibility.unwrap_or_default(),
                source_agent: req.source_agent,
                source_system: req.source_system,
                source_path: req.source_path,
                source_quality: req.source_quality,
                confidence: req.confidence.unwrap_or(0.8),
                ttl_seconds: req.ttl_seconds,
                created_at: now,
                updated_at: now,
                last_verified_at: req.last_verified_at,
                supersedes: req.supersedes,
                tags: req.tags,
                status: req.status.unwrap_or(MemoryStatus::Active),
                stage: memd_schema::MemoryStage::Canonical,
                lane: req.lane,
                version: 1,
                correction_meta: None,
            },
        })
    }

    let app = Router::new().route("/memory/store", post(store));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind offline store server");
    let addr = listener.local_addr().expect("offline store addr");
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("serve offline store server");
    });
    format!("http://{}", addr)
}

async fn spawn_offline_sync_server(
    seen: std::sync::Arc<std::sync::Mutex<Vec<&'static str>>>,
) -> String {
    use axum::{Json, Router, extract::State, routing::post};

    #[derive(Clone)]
    struct SyncState {
        seen: std::sync::Arc<std::sync::Mutex<Vec<&'static str>>>,
    }

    async fn capabilities(
        State(state): State<SyncState>,
        Json(req): Json<memd_schema::CapabilitySyncRequest>,
    ) -> Json<memd_schema::CapabilitySyncResponse> {
        state.seen.lock().expect("seen lock").push("capabilities");
        Json(memd_schema::CapabilitySyncResponse {
            upserted: req.records.len(),
            total: req.records.len(),
            records: req.records,
        })
    }

    async fn access_routes(
        State(state): State<SyncState>,
        Json(req): Json<memd_schema::AccessRouteSyncRequest>,
    ) -> Json<memd_schema::AccessRouteSyncResponse> {
        state.seen.lock().expect("seen lock").push("access_routes");
        Json(memd_schema::AccessRouteSyncResponse {
            upserted: req.routes.len(),
            total: req.routes.len(),
            routes: req.routes,
        })
    }

    async fn token_savings(
        State(state): State<SyncState>,
        Json(req): Json<memd_schema::TokenSavingsSyncRequest>,
    ) -> Json<memd_schema::TokenSavingsSyncResponse> {
        state.seen.lock().expect("seen lock").push("token_savings");
        Json(memd_schema::TokenSavingsSyncResponse {
            upserted: req.records.len(),
            total: req.records.len(),
            records: req.records,
        })
    }

    async fn candidate(
        State(state): State<SyncState>,
        Json(req): Json<memd_schema::CandidateMemoryRequest>,
    ) -> Json<memd_schema::CandidateMemoryResponse> {
        let now = chrono::Utc::now();
        state.seen.lock().expect("seen lock").push("candidates");
        Json(memd_schema::CandidateMemoryResponse {
            item: memd_schema::MemoryItem {
                id: Uuid::new_v4(),
                content: req.content,
                redundancy_key: Some("candidate".to_string()),
                belief_branch: req.belief_branch,
                preferred: false,
                kind: req.kind,
                scope: req.scope,
                project: req.project,
                namespace: req.namespace,
                workspace: req.workspace,
                visibility: req.visibility.unwrap_or_default(),
                source_agent: req.source_agent,
                source_system: req.source_system,
                source_path: req.source_path,
                source_quality: req.source_quality,
                confidence: req.confidence.unwrap_or(0.6),
                ttl_seconds: req.ttl_seconds,
                created_at: now,
                updated_at: now,
                last_verified_at: req.last_verified_at,
                supersedes: req.supersedes,
                tags: req.tags,
                status: MemoryStatus::Active,
                stage: memd_schema::MemoryStage::Candidate,
                lane: req.lane,
                version: 1,
                correction_meta: None,
            },
            duplicate_of: None,
        })
    }

    let app = Router::new()
        .route("/capabilities/sync", post(capabilities))
        .route("/access/routes/sync", post(access_routes))
        .route("/tokens/savings/sync", post(token_savings))
        .route("/memory/candidates", post(candidate))
        .with_state(SyncState { seen });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind offline sync server");
    let addr = listener.local_addr().expect("offline sync addr");
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("serve offline sync server");
    });
    format!("http://{}", addr)
}

#[tokio::test]
async fn remember_queues_offline_store_when_backend_down_and_dedupes() {
    let temp = tempfile::tempdir().expect("tempdir");
    let bundle = temp.path().join(".memd");
    fs::create_dir_all(&bundle).expect("create bundle");
    let args = offline_remember_args(
        bundle.clone(),
        "Offline queued memory survives backend outage.",
    );

    let first = remember_with_bundle_defaults(&args, "http://127.0.0.1:1")
        .await
        .expect("queue offline remember");
    let second = remember_with_bundle_defaults(&args, "http://127.0.0.1:1")
        .await
        .expect("dedupe offline remember");
    let entries = read_offline_store_queue(&bundle).expect("read offline queue");

    assert!(is_offline_queued_response(&first));
    assert!(is_offline_queued_response(&second));
    assert_eq!(first.item.id, second.item.id);
    assert_eq!(entries.len(), 1, "same offline write should dedupe");
    assert_eq!(entries[0].status, "pending");
}

#[tokio::test]
async fn replay_offline_store_queue_syncs_pending_and_skips_synced() {
    let temp = tempfile::tempdir().expect("tempdir");
    let bundle = temp.path().join(".memd");
    fs::create_dir_all(&bundle).expect("create bundle");
    let req = offline_test_request("Replay this offline memory once.");
    let queued = queue_offline_store_request(&bundle, &req, "backend unavailable").expect("queue");
    let base_url = spawn_offline_store_server().await;
    let client = MemdClient::new(&base_url).expect("client");

    let report = replay_offline_store_queue(&bundle, &client)
        .await
        .expect("replay queue");
    let entries = read_offline_store_queue(&bundle).expect("read replayed queue");
    let second = replay_offline_store_queue(&bundle, &client)
        .await
        .expect("second replay");

    assert_eq!(report.attempted, 1);
    assert_eq!(report.synced, 1);
    assert_eq!(report.failed, 0);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].id, queued.id);
    assert_eq!(entries[0].status, "synced");
    assert!(entries[0].synced_item_id.is_some());
    assert_eq!(
        second.attempted, 0,
        "synced entries should not replay twice"
    );
}

#[tokio::test]
async fn replay_offline_sync_queue_replays_candidate_spill_payloads() {
    let temp = tempfile::tempdir().expect("tempdir");
    let bundle = temp.path().join(".memd");
    fs::create_dir_all(&bundle).expect("create bundle");
    let req = offline_candidate_request("Replay this offline hook spill candidate once.");
    queue_offline_sync_payload(
        &bundle,
        OfflineSyncPayload::Candidates(vec![req.clone()]),
        "backend unavailable",
    )
    .expect("queue candidate spill");
    let status = offline_queue_status(&bundle).expect("offline status");
    assert_eq!(status.sync.total, 1);
    assert_eq!(status.sync.by_kind["candidates"].pending, 1);

    let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let base_url = spawn_offline_sync_server(seen.clone()).await;
    let client = MemdClient::new(&base_url).expect("client");
    let report = replay_offline_sync_queue(&bundle, &client)
        .await
        .expect("replay candidate queue");
    let replayed = read_offline_sync_queue(&bundle).expect("read replayed candidate queue");
    let second = replay_offline_sync_queue(&bundle, &client)
        .await
        .expect("second candidate replay");

    assert_eq!(report.attempted, 1);
    assert_eq!(report.synced, 1);
    assert_eq!(report.failed, 0);
    assert_eq!(replayed.len(), 1);
    assert_eq!(replayed[0].status, "synced");
    assert_eq!(second.attempted, 0);
    assert_eq!(seen.lock().expect("seen lock").as_slice(), &["candidates"]);
}

#[test]
fn offline_sync_queue_dedupes_and_reports_status() {
    let temp = tempfile::tempdir().expect("tempdir");
    let bundle = temp.path().join(".memd");
    fs::create_dir_all(&bundle).expect("create bundle");
    let payload = OfflineSyncPayload::Capabilities(memd_schema::CapabilitySyncRequest {
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        workspace: None,
        user_id: None,
        agent: Some("codex".to_string()),
        records: vec![memd_schema::CapabilityRecord {
            harness: "codex".to_string(),
            kind: "skill".to_string(),
            name: "browser".to_string(),
            status: "installed".to_string(),
            portability_class: "harness-native".to_string(),
            source_path: "/tmp/SKILL.md".to_string(),
            bridge_hint: None,
            hash: None,
            notes: Vec::new(),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
            user_id: None,
            agent: Some("codex".to_string()),
            updated_at: None,
        }],
    });

    let first = queue_offline_sync_payload(&bundle, payload.clone(), "server down")
        .expect("queue sync payload");
    let second =
        queue_offline_sync_payload(&bundle, payload, "still down").expect("dedupe sync payload");
    let status = offline_queue_status(&bundle).expect("offline status");

    assert_eq!(first.id, second.id);
    assert_eq!(status.store.total, 0);
    assert_eq!(status.sync.total, 1);
    assert_eq!(status.sync.pending, 1);
}

#[test]
fn offline_sync_queue_status_ignores_superseded_pending_kind() {
    let temp = tempfile::tempdir().expect("tempdir");
    let bundle = temp.path().join(".memd");
    fs::create_dir_all(&bundle).expect("create bundle");
    let old_payload = OfflineSyncPayload::Capabilities(memd_schema::CapabilitySyncRequest {
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        workspace: None,
        user_id: None,
        agent: Some("codex".to_string()),
        records: vec![capability_record_fixture("old")],
    });
    let new_payload = OfflineSyncPayload::Capabilities(memd_schema::CapabilitySyncRequest {
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        workspace: None,
        user_id: None,
        agent: Some("codex".to_string()),
        records: vec![capability_record_fixture("new")],
    });
    let queued_at = chrono::Utc::now();
    let entries = vec![
        OfflineSyncQueueEntry {
            id: Uuid::new_v4(),
            dedup_key: offline_sync_dedup_key(&new_payload).expect("new dedup"),
            queued_at,
            attempts: 1,
            status: "synced".to_string(),
            last_error: None,
            payload: new_payload,
            synced_at: Some(queued_at + chrono::Duration::seconds(1)),
        },
        OfflineSyncQueueEntry {
            id: Uuid::new_v4(),
            dedup_key: offline_sync_dedup_key(&old_payload).expect("old dedup"),
            queued_at: queued_at + chrono::Duration::seconds(60),
            attempts: 0,
            status: "pending".to_string(),
            last_error: Some("server not reachable at http://127.0.0.1:9".to_string()),
            payload: old_payload,
            synced_at: None,
        },
    ];
    write_offline_sync_queue(&bundle, &entries).expect("write sync queue");

    let status = offline_queue_status(&bundle).expect("offline status");

    assert_eq!(status.sync.total, 2);
    assert_eq!(status.sync.pending, 0);
    assert_eq!(status.sync.failed, 0);
    assert_eq!(status.sync.synced, 1);
    let capabilities = status
        .sync
        .by_kind
        .get("capabilities")
        .expect("capabilities status");
    assert_eq!(capabilities.total, 2);
    assert_eq!(capabilities.pending, 0);
    assert_eq!(capabilities.synced, 1);
}

#[test]
fn offline_capability_sync_replay_chunks_large_payloads() {
    let mut req = memd_schema::CapabilitySyncRequest {
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        workspace: Some("shared".to_string()),
        user_id: None,
        agent: Some("codex".to_string()),
        records: (0..10)
            .map(|index| memd_schema::CapabilityRecord {
                harness: "codex".to_string(),
                kind: "plugin-skill".to_string(),
                name: format!("skill-{index}"),
                status: "installed".to_string(),
                portability_class: "harness-native".to_string(),
                source_path: format!("/tmp/skill-{index}.md"),
                bridge_hint: None,
                hash: None,
                notes: Vec::new(),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                user_id: None,
                agent: Some("codex".to_string()),
                updated_at: None,
            })
            .collect(),
    };
    for record in &mut req.records {
        record.notes = vec!["x".repeat(2048)];
    }

    let chunks = offline_capability_sync_request_chunks(&req, 100, 10 * 1024);

    assert!(chunks.len() > 1);
    assert_eq!(
        chunks
            .iter()
            .map(|chunk| chunk.records.len())
            .sum::<usize>(),
        req.records.len()
    );
    assert!(
        chunks.iter().all(|chunk| {
            serde_json::to_vec(chunk).expect("serialize chunk").len() <= 10 * 1024
        })
    );
}

#[tokio::test]
async fn replay_offline_sync_queue_reconciles_payloads_with_server_authority() {
    let temp = tempfile::tempdir().expect("tempdir");
    let bundle = temp.path().join(".memd");
    fs::create_dir_all(&bundle).expect("create bundle");
    let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let base_url = spawn_offline_sync_server(seen.clone()).await;
    let client = MemdClient::new(&base_url).expect("client");

    queue_offline_sync_payload(
        &bundle,
        OfflineSyncPayload::Capabilities(memd_schema::CapabilitySyncRequest {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            user_id: None,
            agent: Some("codex@pc-a".to_string()),
            records: vec![memd_schema::CapabilityRecord {
                harness: "codex".to_string(),
                kind: "skill".to_string(),
                name: "browser-use:browser".to_string(),
                status: "installed".to_string(),
                portability_class: "harness-native".to_string(),
                source_path: "/Users/test/.codex/skills/browser/SKILL.md".to_string(),
                bridge_hint: Some("PC-B can use this through target equivalent".to_string()),
                hash: Some("sha256:cap".to_string()),
                notes: vec!["queued while backend down".to_string()],
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                user_id: None,
                agent: Some("codex@pc-a".to_string()),
                updated_at: None,
            }],
        }),
        "backend down",
    )
    .expect("queue capability sync");
    queue_offline_sync_payload(
        &bundle,
        OfflineSyncPayload::AccessRoutes(memd_schema::AccessRouteSyncRequest {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            user_id: None,
            agent: Some("codex@pc-a".to_string()),
            routes: vec![memd_schema::AccessRouteRecord {
                id: "bitwarden-login".to_string(),
                provider: "bitwarden".to_string(),
                status: "locked".to_string(),
                scope: "user/project".to_string(),
                secret_values_stored: false,
                guidance: "Ask user to unlock Bitwarden; store refs only.".to_string(),
                source: "bw status".to_string(),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                user_id: None,
                agent: Some("codex@pc-a".to_string()),
                updated_at: None,
            }],
        }),
        "backend down",
    )
    .expect("queue access route sync");
    queue_offline_sync_payload(
        &bundle,
        OfflineSyncPayload::TokenSavings(memd_schema::TokenSavingsSyncRequest {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            user_id: None,
            agent: Some("codex@pc-a".to_string()),
            records: vec![memd_schema::TokenSavingsRecord {
                id: uuid::Uuid::new_v4(),
                operation: "context_packet".to_string(),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                user_id: None,
                agent: Some("codex@pc-a".to_string()),
                model_tier: Some("tiny".to_string()),
                intent: Some("CurrentTask".to_string()),
                source_records: 3,
                baseline_input_tokens: 1200,
                output_tokens: 280,
                tokens_saved: 920,
                wasted_tokens: 0,
                waste_kind: None,
                reason: "offline packet compile avoided reread".to_string(),
                ts: chrono::Utc::now(),
                updated_at: None,
            }],
        }),
        "backend down",
    )
    .expect("queue token savings sync");

    let report = replay_offline_sync_queue(&bundle, &client)
        .await
        .expect("replay sync queue");
    let second = replay_offline_sync_queue(&bundle, &client)
        .await
        .expect("second replay skips synced");
    let status = offline_queue_status(&bundle).expect("offline status after sync");
    let seen = seen.lock().expect("seen lock").clone();

    assert_eq!(report.attempted, 3);
    assert_eq!(report.synced, 3);
    assert_eq!(report.failed, 0);
    assert_eq!(report.pending, 0);
    assert_eq!(second.attempted, 0);
    assert_eq!(status.sync.pending, 0);
    assert_eq!(status.sync.by_kind["capabilities"].synced, 1);
    assert_eq!(status.sync.by_kind["access_routes"].synced, 1);
    assert_eq!(status.sync.by_kind["token_savings"].synced, 1);
    assert!(seen.contains(&"capabilities"));
    assert!(seen.contains(&"access_routes"));
    assert!(seen.contains(&"token_savings"));
}

#[test]
fn infer_bundle_identity_defaults_bind_repo_without_runtime_config() {
    let temp_root =
        std::env::temp_dir().join(format!("memd-checkpoint-defaults-{}", uuid::Uuid::new_v4()));
    let repo_root = temp_root.join("repo-b");
    let bundle_root = repo_root.join(".memd");

    fs::create_dir_all(repo_root.join(".git")).expect("create repo git dir");

    let (project, namespace) = infer_bundle_identity_defaults(&bundle_root);
    assert_eq!(project.as_deref(), Some("repo-b"));
    assert_eq!(namespace.as_deref(), Some("main"));

    fs::remove_dir_all(temp_root).expect("cleanup temp root");
}

#[test]
fn git_auto_commit_clean_tree_returns_none() {
    let temp_root =
        std::env::temp_dir().join(format!("memd-auto-commit-clean-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_root).expect("create temp dir");

    // Init a git repo with one commit
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&temp_root)
        .output()
        .expect("git init");
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(&temp_root)
        .output()
        .expect("git config email");
    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(&temp_root)
        .output()
        .expect("git config name");
    fs::write(temp_root.join("file.txt"), "content").expect("write file");
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&temp_root)
        .output()
        .expect("git add");
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(&temp_root)
        .output()
        .expect("git commit");

    // Pass explicit repo root — no set_current_dir needed
    let result = git_auto_commit_if_dirty_in("test: should not commit", Some(&temp_root));

    assert!(result.is_ok());
    assert!(result.unwrap().is_none(), "clean tree should return None");

    fs::remove_dir_all(temp_root).expect("cleanup temp root");
}

#[test]
fn git_auto_commit_dirty_tree_commits_and_returns_hash() {
    let temp_root =
        std::env::temp_dir().join(format!("memd-auto-commit-test-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_root).expect("create temp dir");

    // Init a git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&temp_root)
        .output()
        .expect("git init");

    // Configure git user for the test repo
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(&temp_root)
        .output()
        .expect("git config email");
    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(&temp_root)
        .output()
        .expect("git config name");

    // Create and commit an initial file (need at least one commit)
    let file_path = temp_root.join("tracked.txt");
    fs::write(&file_path, "initial").expect("write file");
    std::process::Command::new("git")
        .args(["add", "tracked.txt"])
        .current_dir(&temp_root)
        .output()
        .expect("git add");
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(&temp_root)
        .output()
        .expect("git commit initial");

    // Modify the tracked file (makes tree dirty)
    fs::write(&file_path, "modified").expect("modify file");

    // Pass explicit repo root — no set_current_dir needed
    let result = git_auto_commit_if_dirty_in("test: auto-commit dirty tree", Some(&temp_root));

    assert!(result.is_ok());
    let hash = result.unwrap();
    assert!(hash.is_some(), "should have committed and returned a hash");
    assert!(!hash.unwrap().is_empty(), "hash should not be empty");

    fs::remove_dir_all(temp_root).expect("cleanup temp root");
}

#[test]
fn git_auto_commit_refuses_broad_dirty_tree() {
    let temp_root =
        std::env::temp_dir().join(format!("memd-auto-commit-broad-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_root).expect("create temp dir");

    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&temp_root)
        .output()
        .expect("git init");
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(&temp_root)
        .output()
        .expect("git config email");
    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(&temp_root)
        .output()
        .expect("git config name");

    for index in 0..6 {
        fs::write(temp_root.join(format!("tracked-{index}.txt")), "initial")
            .expect("write initial file");
    }
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&temp_root)
        .output()
        .expect("git add");
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(&temp_root)
        .output()
        .expect("git commit initial");

    for index in 0..6 {
        fs::write(temp_root.join(format!("tracked-{index}.txt")), "modified")
            .expect("modify tracked file");
    }

    let err = git_auto_commit_if_dirty_in("test: should refuse", Some(&temp_root))
        .expect_err("broad dirty tree should be rejected");
    assert!(
        err.to_string().contains("refusing memd auto-commit"),
        "unexpected error: {err}"
    );

    let staged = std::process::Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .current_dir(&temp_root)
        .status()
        .expect("git diff cached");
    assert!(
        staged.success(),
        "broad auto-commit guard should not stage files"
    );

    fs::remove_dir_all(temp_root).expect("cleanup temp root");
}

#[test]
fn git_auto_commit_refuses_multi_scope_dirty_tree_under_file_limit() {
    let temp_root = std::env::temp_dir().join(format!(
        "memd-auto-commit-multi-scope-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(temp_root.join("crates/memd-client")).expect("create crate dir");
    fs::create_dir_all(temp_root.join("docs/contracts")).expect("create docs dir");

    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&temp_root)
        .output()
        .expect("git init");
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(&temp_root)
        .output()
        .expect("git config email");
    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(&temp_root)
        .output()
        .expect("git config name");

    fs::write(temp_root.join("crates/memd-client/lib.rs"), "initial\n").expect("write crate file");
    fs::write(temp_root.join("docs/contracts/auto-commit.md"), "initial\n")
        .expect("write docs file");
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&temp_root)
        .output()
        .expect("git add");
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(&temp_root)
        .output()
        .expect("git commit initial");

    fs::write(temp_root.join("crates/memd-client/lib.rs"), "modified\n")
        .expect("modify crate file");
    fs::write(
        temp_root.join("docs/contracts/auto-commit.md"),
        "modified\n",
    )
    .expect("modify docs file");

    let err = git_auto_commit_if_dirty_in("test: should refuse", Some(&temp_root))
        .expect_err("multi-scope dirty tree should be rejected");
    assert!(
        err.to_string().contains("multiple atomic scopes"),
        "unexpected error: {err}"
    );

    let staged = std::process::Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .current_dir(&temp_root)
        .status()
        .expect("git diff cached");
    assert!(
        staged.success(),
        "multi-scope auto-commit guard should not stage files"
    );

    fs::remove_dir_all(temp_root).expect("cleanup temp root");
}

#[test]
fn update_roadmap_state_patches_existing_keys() {
    let temp_root =
        std::env::temp_dir().join(format!("memd-roadmap-state-test-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_root).expect("create temp dir");

    // Write a ROADMAP.md with a state block (no git needed — pass explicit path)
    let roadmap = r#"# Roadmap

<!-- ROADMAP_STATE
current_phase: O2
phase_status: verified
next_step: P2 — do the thing: with colons
note: O2 done
-->

## Content
"#;
    fs::write(temp_root.join("ROADMAP.md"), roadmap).expect("write roadmap");

    let updates = vec![
        ("current_phase".to_string(), "P2".to_string()),
        ("phase_status".to_string(), "in_progress".to_string()),
    ];

    let result = update_roadmap_state_in(&updates, Some(&temp_root));

    assert!(result.is_ok());
    assert!(result.unwrap(), "should report changes made");

    let updated = fs::read_to_string(temp_root.join("ROADMAP.md")).expect("read updated");
    assert!(
        updated.contains("current_phase: P2"),
        "phase should be updated"
    );
    assert!(
        updated.contains("phase_status: in_progress"),
        "status should be updated"
    );
    // Colons in values must survive
    assert!(
        updated.contains("next_step: P2 — do the thing: with colons"),
        "colon-bearing values must be preserved"
    );
    assert!(updated.contains("## Content"), "rest of file preserved");

    fs::remove_dir_all(temp_root).expect("cleanup");
}

#[test]
fn update_roadmap_state_appends_new_keys() {
    let temp_root =
        std::env::temp_dir().join(format!("memd-roadmap-append-test-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_root).expect("create temp dir");

    let roadmap = "<!-- ROADMAP_STATE\ncurrent_phase: O2\n-->\n";
    fs::write(temp_root.join("ROADMAP.md"), roadmap).expect("write roadmap");

    let updates = vec![("new_key".to_string(), "new_value".to_string())];
    let result = update_roadmap_state_in(&updates, Some(&temp_root));

    assert!(result.is_ok());
    assert!(result.unwrap(), "should report changes");

    let updated = fs::read_to_string(temp_root.join("ROADMAP.md")).expect("read");
    assert!(updated.contains("new_key: new_value"), "new key appended");
    assert!(
        updated.contains("current_phase: O2"),
        "existing key preserved"
    );

    fs::remove_dir_all(temp_root).expect("cleanup");
}

#[test]
fn update_roadmap_state_no_changes_returns_false() {
    let temp_root =
        std::env::temp_dir().join(format!("memd-roadmap-noop-test-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_root).expect("create temp dir");

    let roadmap = "<!-- ROADMAP_STATE\ncurrent_phase: O2\n-->\n";
    fs::write(temp_root.join("ROADMAP.md"), roadmap).expect("write roadmap");

    // Same value — no change
    let updates = vec![("current_phase".to_string(), "O2".to_string())];
    let result = update_roadmap_state_in(&updates, Some(&temp_root));

    assert!(result.is_ok());
    assert!(!result.unwrap(), "no changes should return false");

    fs::remove_dir_all(temp_root).expect("cleanup");
}
