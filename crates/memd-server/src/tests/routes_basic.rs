use super::*;

#[tokio::test]
async fn capability_and_access_sync_routes_round_trip_and_reject_secret_values() {
    let (dir, state) = temp_state("memd-capability-access-sync-route");
    let app = Router::new()
        .route("/capabilities", get(get_capabilities))
        .route("/capabilities/sync", post(post_capabilities_sync))
        .route("/access/routes", get(get_access_routes))
        .route("/access/routes/sync", post(post_access_routes_sync))
        .with_state(state);

    let capability_req = CapabilitySyncRequest {
        project: Some("memd-sync-proof".to_string()),
        namespace: Some("main".to_string()),
        workspace: Some("shared".to_string()),
        user_id: Some("user-a".to_string()),
        agent: Some("codex".to_string()),
        records: vec![
            CapabilityRecord {
                harness: "codex".to_string(),
                kind: "skill".to_string(),
                name: "browser-use:browser".to_string(),
                status: "installed".to_string(),
                portability_class: "harness-native".to_string(),
                source_path: "/Users/aparcedodev/.codex/plugins/cache/browser/SKILL.md".to_string(),
                bridge_hint: Some("use browser plugin for local web checks".to_string()),
                hash: Some("sha256:test".to_string()),
                notes: vec!["synced from PC-A".to_string()],
                project: None,
                namespace: None,
                workspace: None,
                user_id: None,
                agent: None,
                updated_at: None,
            },
            CapabilityRecord {
                harness: "claude-code".to_string(),
                kind: "plugin".to_string(),
                name: "cloud-browser-session".to_string(),
                status: "unavailable".to_string(),
                portability_class: "target-equivalent".to_string(),
                source_path: "/Users/aparcedodev/.claude/plugins/cloud-browser.json".to_string(),
                bridge_hint: Some(
                    "surface as missing tool; suggest Codex browser-use equivalent".to_string(),
                ),
                hash: Some("sha256:missing".to_string()),
                notes: vec!["synced from PC-A; unavailable on PC-B must remain listed".to_string()],
                project: None,
                namespace: None,
                workspace: None,
                user_id: None,
                agent: None,
                updated_at: None,
            },
        ],
    };
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/capabilities/sync")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&capability_req).expect("serialize capability sync"),
                ))
                .expect("build capability sync request"),
        )
        .await
        .expect("capability sync route");
    assert_eq!(response.status(), StatusCode::OK);
    let synced: CapabilitySyncResponse = decode_json(response).await;
    assert_eq!(synced.upserted, 2);
    assert_eq!(
        synced.records[0].project.as_deref(),
        Some("memd-sync-proof")
    );
    assert_eq!(synced.records[0].agent.as_deref(), Some("codex"));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/capabilities?project=memd-sync-proof&workspace=shared&harness=codex&kind=skill&query=browser")
                .body(Body::empty())
                .expect("build capability list request"),
        )
        .await
        .expect("capability list route");
    assert_eq!(response.status(), StatusCode::OK);
    let listed: CapabilityListResponse = decode_json(response).await;
    assert_eq!(listed.total, 1);
    assert_eq!(listed.records[0].name, "browser-use:browser");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/capabilities?project=memd-sync-proof&workspace=shared&query=pc-a")
                .body(Body::empty())
                .expect("build pc-b capability list request"),
        )
        .await
        .expect("capability list from pc-b route");
    assert_eq!(response.status(), StatusCode::OK);
    let listed: CapabilityListResponse = decode_json(response).await;
    assert_eq!(listed.total, 2);
    assert!(
        listed
            .records
            .iter()
            .any(|record| record.name == "cloud-browser-session"
                && record.status == "unavailable"
                && record
                    .bridge_hint
                    .as_deref()
                    .is_some_and(|hint| hint.contains("equivalent"))),
        "PC-B capability list should retain unavailable PC-A tool with equivalent guidance"
    );

    let access_req = AccessRouteSyncRequest {
        project: Some("memd-sync-proof".to_string()),
        namespace: Some("main".to_string()),
        workspace: Some("shared".to_string()),
        user_id: Some("user-a".to_string()),
        agent: Some("codex".to_string()),
        routes: vec![
            AccessRouteRecord {
                id: "bitwarden-login".to_string(),
                provider: "bitwarden".to_string(),
                status: "locked".to_string(),
                scope: "user/project".to_string(),
                secret_values_stored: false,
                guidance: "Ask user to unlock Bitwarden before workaround; resolve refs only."
                    .to_string(),
                source: "bw status".to_string(),
                project: None,
                namespace: None,
                workspace: None,
                user_id: None,
                agent: None,
                updated_at: None,
            },
            AccessRouteRecord {
                id: "agent-secrets-broker".to_string(),
                provider: "agent-secrets".to_string(),
                status: "unavailable".to_string(),
                scope: "user/project".to_string(),
                secret_values_stored: false,
                guidance: "agent-secrets broker unavailable on PC-B; keep refs listed and ask user for approved route."
                    .to_string(),
                source: "agent-secrets status".to_string(),
                project: None,
                namespace: None,
                workspace: None,
                user_id: None,
                agent: None,
                updated_at: None,
            },
        ],
    };
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/access/routes/sync")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&access_req).expect("serialize access sync"),
                ))
                .expect("build access sync request"),
        )
        .await
        .expect("access sync route");
    assert_eq!(response.status(), StatusCode::OK);
    let synced: AccessRouteSyncResponse = decode_json(response).await;
    assert_eq!(synced.upserted, 2);
    assert!(!synced.routes[0].secret_values_stored);
    assert_eq!(synced.routes[0].project.as_deref(), Some("memd-sync-proof"));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/access/routes?project=memd-sync-proof&workspace=shared&provider=bitwarden&query=unlock")
                .body(Body::empty())
                .expect("build access list request"),
        )
        .await
        .expect("access list route");
    assert_eq!(response.status(), StatusCode::OK);
    let listed: AccessRouteListResponse = decode_json(response).await;
    assert_eq!(listed.total, 1);
    assert_eq!(listed.routes[0].provider, "bitwarden");
    assert!(!listed.routes[0].secret_values_stored);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/access/routes?project=memd-sync-proof&workspace=shared&provider=agent-secrets&query=pc-b")
                .body(Body::empty())
                .expect("build pc-b access route list request"),
        )
        .await
        .expect("access route list from pc-b route");
    assert_eq!(response.status(), StatusCode::OK);
    let listed: AccessRouteListResponse = decode_json(response).await;
    assert_eq!(listed.total, 1);
    assert_eq!(listed.routes[0].provider, "agent-secrets");
    assert_eq!(listed.routes[0].status, "unavailable");
    assert!(!listed.routes[0].secret_values_stored);
    assert!(listed.routes[0].guidance.contains("ask user"));

    let mut bad_access_req = access_req;
    bad_access_req.routes[0].secret_values_stored = true;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/access/routes/sync")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&bad_access_req).expect("serialize bad access sync"),
                ))
                .expect("build bad access sync request"),
        )
        .await
        .expect("bad access sync route");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    std::fs::remove_dir_all(dir).expect("cleanup capability/access sync temp dir");
}

#[test]
fn capability_list_supports_full_fresh_machine_pull() {
    let (dir, state) = temp_state("memd-capability-full-pull");
    let records = (0..750)
        .map(|index| CapabilityRecord {
            harness: "local".to_string(),
            kind: "cli".to_string(),
            name: format!("tool-{index}"),
            status: "installed".to_string(),
            portability_class: "host-local".to_string(),
            source_path: format!("host-cli:tool-{index}"),
            bridge_hint: Some("sync host-local install guidance".to_string()),
            hash: None,
            notes: vec![format!(
                "memd:host-cli-install-plan:#!/bin/sh\necho install tool-{index}\n"
            )],
            project: None,
            namespace: None,
            workspace: None,
            user_id: None,
            agent: None,
            updated_at: None,
        })
        .collect::<Vec<_>>();

    state
        .store
        .upsert_capabilities(&CapabilitySyncRequest {
            project: Some("memd-sync-proof".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            user_id: None,
            agent: Some("codex".to_string()),
            records,
        })
        .expect("sync capability inventory");

    let pulled = state
        .store
        .list_capabilities(&CapabilityListRequest {
            project: Some("memd-sync-proof".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            user_id: None,
            harness: None,
            kind: None,
            query: None,
            limit: Some(5_000),
        })
        .expect("list full capability inventory");

    assert_eq!(pulled.records.len(), 750);
    assert!(
        pulled
            .records
            .iter()
            .any(|record| record.name == "tool-749")
    );

    std::fs::remove_dir_all(dir).ok();
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
async fn dashboard_root_redirects_to_react_dashboard_and_coordination_endpoints_remain_live() {
    let (dir, state) = temp_state("memd-dashboard-hive-controls");
    crate::ui::test_insert_visible_item(&state, "runtime spine", true)
        .expect("seed visible memory");
    seed_hive_route_state(&state);
    let app = Router::new()
        .route("/", get(dashboard_redirect))
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
    assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
    assert_eq!(
        response
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok()),
        Some("/dashboard/")
    );

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
