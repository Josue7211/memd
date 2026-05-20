use super::*;

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
            "/coordination/dev-servers/acquire",
            post(post_dev_server_lease_acquire),
        )
        .route(
            "/coordination/dev-servers/release",
            post(post_dev_server_lease_release),
        )
        .route("/coordination/dev-servers", get(get_dev_server_leases))
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

pub(crate) fn store_test_item(state: &AppState) -> MemoryItem {
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
async fn dev_server_lease_same_session_renews_and_records_receipts() {
    let (dir, state) = temp_state("memd-dev-server-lease-renew");
    let scope = "resource:dev-server:repo1234:127.0.0.1:43210";
    let first = sample_dev_server_lease(scope, "bee-1");
    let first_response = state
        .store
        .acquire_dev_server_lease(&first)
        .expect("acquire dev server lease");
    assert_eq!(first_response.leases.len(), 1);

    let mut renewed = sample_dev_server_lease(scope, "bee-1");
    renewed.pid = Some(5252);
    let renewed_response = state
        .store
        .acquire_dev_server_lease(&renewed)
        .expect("renew dev server lease");
    assert_eq!(renewed_response.leases[0].pid, Some(5252));

    let leases = state
        .store
        .dev_server_leases(&DevServerLeasesRequest {
            session: Some("bee-1".to_string()),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            repo_hash: Some("repo1234".to_string()),
            active_only: Some(true),
            limit: Some(16),
        })
        .expect("list dev server leases");
    assert_eq!(leases.leases.len(), 1);

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
    assert!(receipts.receipts.iter().any(|receipt| {
        receipt.kind == "dev_server_acquire" && receipt.scope.as_deref() == Some(scope)
    }));
    assert!(receipts.receipts.iter().any(|receipt| {
        receipt.kind == "dev_server_heartbeat" && receipt.scope.as_deref() == Some(scope)
    }));

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[tokio::test]
async fn dev_server_lease_conflict_returns_409_and_receipt() {
    let (dir, state) = temp_state("memd-dev-server-lease-conflict");
    let scope = "resource:dev-server:repo1234:127.0.0.1:43210";
    state
        .store
        .acquire_dev_server_lease(&sample_dev_server_lease(scope, "bee-1"))
        .expect("acquire first dev server lease");
    let app = test_full_router(state);

    let mut contender = sample_dev_server_lease(scope, "bee-2");
    contender.stale_after_seconds = 600;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/coordination/dev-servers/acquire")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&contender).unwrap()))
                .unwrap(),
        )
        .await
        .expect("conflicting lease acquire route");
    assert_eq!(response.status(), StatusCode::CONFLICT);

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[tokio::test]
async fn dev_server_lease_expired_can_be_reclaimed() {
    let (dir, state) = temp_state("memd-dev-server-lease-recover");
    let scope = "resource:dev-server:repo1234:127.0.0.1:43210";
    let mut stale = sample_dev_server_lease(scope, "bee-1");
    stale.ttl_seconds = 1;
    state
        .store
        .acquire_dev_server_lease(&stale)
        .expect("acquire stale candidate lease");
    std::thread::sleep(std::time::Duration::from_millis(1200));

    let recovered = state
        .store
        .acquire_dev_server_lease(&sample_dev_server_lease(scope, "bee-2"))
        .expect("reclaim expired dev server lease");
    assert_eq!(recovered.leases[0].session, "bee-2");

    let leases = state
        .store
        .dev_server_leases(&DevServerLeasesRequest {
            session: None,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            repo_hash: Some("repo1234".to_string()),
            active_only: Some(true),
            limit: Some(16),
        })
        .expect("list recovered dev server leases");
    assert_eq!(leases.leases.len(), 1);
    assert_eq!(leases.leases[0].session, "bee-2");

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[tokio::test]
async fn dev_server_stale_heartbeat_can_be_recovered_before_ttl_expires() {
    let (dir, state) = temp_state("memd-dev-server-lease-stale-recover");
    let scope = "resource:dev-server:repo1234:127.0.0.1:43210";
    let mut first = sample_dev_server_lease(scope, "bee-1");
    first.ttl_seconds = 600;
    first.stale_after_seconds = 1;
    state
        .store
        .acquire_dev_server_lease(&first)
        .expect("acquire first dev server lease");
    std::thread::sleep(std::time::Duration::from_millis(1200));

    let mut contender = sample_dev_server_lease(scope, "bee-2");
    contender.ttl_seconds = 600;
    contender.stale_after_seconds = 1;
    let recovered = state
        .store
        .acquire_dev_server_lease(&contender)
        .expect("recover stale dev server heartbeat");
    assert_eq!(recovered.leases[0].session, "bee-2");

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
    assert!(receipts.receipts.iter().any(|receipt| {
        receipt.kind == "dev_server_recover" && receipt.scope.as_deref() == Some(scope)
    }));

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[tokio::test]
async fn dev_server_release_by_non_owner_conflicts() {
    let (dir, state) = temp_state("memd-dev-server-release-denied");
    let scope = "resource:dev-server:repo1234:127.0.0.1:43210";
    state
        .store
        .acquire_dev_server_lease(&sample_dev_server_lease(scope, "bee-1"))
        .expect("acquire dev server lease");
    let app = test_full_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/coordination/dev-servers/release")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&DevServerLeaseReleaseRequest {
                        scope: scope.to_string(),
                        session: "bee-2".to_string(),
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .expect("release denied route");
    assert_eq!(response.status(), StatusCode::CONFLICT);

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[tokio::test]
async fn hive_board_surfaces_dev_server_owner_and_conflict() {
    let (dir, state) = temp_state("memd-dev-server-board-visible");
    seed_hive_route_state(&state);
    let scope = "resource:dev-server:repo1234:127.0.0.1:43210";
    state
        .store
        .acquire_dev_server_lease(&sample_dev_server_lease(scope, "bee-1"))
        .expect("acquire dev server lease");
    let mut contender = sample_dev_server_lease(scope, "bee-2");
    contender.stale_after_seconds = 600;
    assert!(
        state
            .store
            .acquire_dev_server_lease(&contender)
            .expect_err("conflicting lease should fail")
            .to_string()
            .contains("dev_server_conflict")
    );

    let board = state
        .store
        .hive_board(&HiveBoardRequest {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
        })
        .expect("load hive board");
    assert!(
        board
            .lane_faults
            .iter()
            .any(|fault| fault.contains("dev-server http://127.0.0.1:43210 owner=bee-1"))
    );
    assert!(
        board
            .blocked_bees
            .iter()
            .any(|blocked| blocked.contains("already leased by bee-1"))
    );
    assert!(
        board
            .recommended_actions
            .iter()
            .any(|action| action.contains("reuse http://127.0.0.1:43210 owned by bee-1"))
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
