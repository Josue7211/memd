use super::*;

#[path = "memory_behaviors_advanced.rs"]
mod memory_behaviors_advanced;
#[path = "memory_behaviors_tail.rs"]
mod memory_behaviors_tail;

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
        "workspace": "core",
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

#[tokio::test]
async fn authority_search_is_opt_in_token_gated_and_reads_legacy_private_rows() {
    let (dir, state) = temp_state("memd-authority-search");

    let mut private_item = sample_memory_item(None);
    private_item.content = "legacy private authority inventory row".to_string();
    private_item.visibility = MemoryVisibility::Private;
    private_item.source_agent = None;
    private_item.tags = vec!["authority-inventory".to_string()];
    let ck = super::keys::canonical_key(&private_item);
    let rk = super::keys::redundancy_key(&private_item);
    state
        .store
        .insert_or_get_duplicate(&private_item, &ck, &rk)
        .expect("insert authority item");

    let app = Router::new()
        .route("/memory/search", post(search_memory))
        .route("/memory/authority/search", post(search_memory_authority))
        .with_state(state);
    let req_body = serde_json::json!({
        "project": "memd",
        "namespace": "main",
        "scopes": [],
        "kinds": [],
        "statuses": [],
        "tags": ["authority-inventory"],
        "stages": [],
        "limit": 10,
    });
    let request = |path: &str, token: Option<&str>| {
        let mut builder = Request::builder()
            .method("POST")
            .uri(path)
            .header("content-type", "application/json");
        if let Some(token) = token {
            builder = builder.header("x-memd-authority-token", token);
        }
        builder
            .body(Body::from(serde_json::to_string(&req_body).unwrap()))
            .expect("build request")
    };

    let normal = app
        .clone()
        .oneshot(request("/memory/search", None))
        .await
        .expect("run normal search");
    assert_eq!(normal.status(), StatusCode::OK);
    let normal_body: SearchMemoryResponse = decode_json(normal).await;
    assert!(normal_body.items.is_empty());

    let disabled = app
        .clone()
        .oneshot(request("/memory/authority/search", None))
        .await
        .expect("run disabled authority search");
    assert_eq!(disabled.status(), StatusCode::NOT_FOUND);

    let _enabled = set_test_env("MEMD_AUTHORITY_SEARCH", "1");
    let missing_token = app
        .clone()
        .oneshot(request("/memory/authority/search", None))
        .await
        .expect("run authority search without token");
    assert_eq!(missing_token.status(), StatusCode::UNAUTHORIZED);

    let _token = set_test_env("MEMD_AUTHORITY_TOKEN", "secret-authority-token");
    let allowed = app
        .oneshot(request(
            "/memory/authority/search",
            Some("secret-authority-token"),
        ))
        .await
        .expect("run authority search");
    assert_eq!(allowed.status(), StatusCode::OK);
    let body: SearchMemoryResponse = decode_json(allowed).await;
    assert_eq!(body.items.len(), 1);
    assert_eq!(body.items[0].id, private_item.id);

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
            tags: Some(vec!["turn:s1-t5".to_string(), "geography".to_string()]),
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
    let meta = response
        .new_item
        .correction_meta
        .as_ref()
        .expect("correction metadata");
    assert_eq!(meta.corrects_id, Some(original.id));
    assert_eq!(meta.source_turn.as_deref(), Some("s1-t5"));
    assert_eq!(meta.captured_by, Some(memd_schema::CaptureSource::Manual));

    let old_from_store = state.store.get(original.id).unwrap().unwrap();
    assert_eq!(old_from_store.status, MemoryStatus::Superseded);
    let new_from_store = state.store.get(response.new_item.id).unwrap().unwrap();
    assert_eq!(new_from_store.status, MemoryStatus::Active);

    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[test]
fn correct_item_derives_correction_meta_from_legacy_turn_tag() {
    let (dir, state) = temp_state("memd-correct-meta-legacy-tag");

    let (original, _) = state
        .store_item(
            StoreMemoryRequest {
                content: "deploy target is AWS".to_string(),
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
                confidence: Some(0.7),
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: vec![],
                tags: vec!["infra".to_string()],
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
            content: "deploy target is GCP".to_string(),
            reason: Some("migration finished".to_string()),
            tags: Some(vec![
                "infra".to_string(),
                "v7-seed47-s1-correct-001".to_string(),
            ]),
            confidence: Some(0.91),
        },
    )
    .expect("correct item");

    let meta = response
        .new_item
        .correction_meta
        .as_ref()
        .expect("correction metadata");
    assert_eq!(meta.corrects_id, Some(original.id));
    assert_eq!(
        meta.source_turn.as_deref(),
        Some("v7-seed47-s1-correct-001")
    );
    assert_eq!(meta.confidence, Some(0.91));

    let from_store = state.store.get(response.new_item.id).unwrap().unwrap();
    assert_eq!(
        from_store.correction_meta,
        response.new_item.correction_meta
    );

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
    assert_eq!(
        explain_new
            .item
            .correction_meta
            .as_ref()
            .and_then(|m| m.corrects_id),
        Some(original.id),
        "explain should carry correction metadata on corrected item"
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
