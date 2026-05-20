use super::*;

#[tokio::test]
async fn context_packet_prioritizes_host_cli_auth_guidance_for_fresh_harnesses() {
    let (dir, state) = temp_state("memd-context-host-cli-auth-guidance");
    let project = "memd-context-host-cli-auth";
    let namespace = "main";

    let mut records = (0..20)
        .map(|index| CapabilityRecord {
            harness: "codex".to_string(),
            kind: "skill".to_string(),
            name: format!("skill-{index}"),
            status: "installed".to_string(),
            portability_class: "harness-native".to_string(),
            source_path: format!("/remote/skill-{index}.md"),
            bridge_hint: None,
            hash: None,
            notes: Vec::new(),
            project: None,
            namespace: None,
            workspace: None,
            user_id: None,
            agent: None,
            updated_at: None,
        })
        .collect::<Vec<_>>();
    records.push(CapabilityRecord {
        harness: "local".to_string(),
        kind: "cli".to_string(),
        name: "opencode".to_string(),
        status: "installed".to_string(),
        portability_class: "host-local".to_string(),
        source_path: "/opt/bin/opencode".to_string(),
        bridge_hint: Some("host-local CLI; auth state is machine-specific".to_string()),
        hash: None,
        notes: vec![
            "memd:host-auth-status:unauthenticated".to_string(),
            "memd:host-auth-check:opencode auth status".to_string(),
            "memd:host-auth-proof:local-probe".to_string(),
            "memd:host-auth-output-stored:false".to_string(),
        ],
        project: None,
        namespace: None,
        workspace: None,
        user_id: None,
        agent: None,
        updated_at: None,
    });
    records.push(CapabilityRecord {
        harness: "local".to_string(),
        kind: "cli".to_string(),
        name: "wrangler".to_string(),
        status: "installed".to_string(),
        portability_class: "host-local".to_string(),
        source_path: "/opt/bin/wrangler".to_string(),
        bridge_hint: Some("host-local CLI; auth state is machine-specific".to_string()),
        hash: None,
        notes: vec![
            "memd:host-auth-status:authenticated".to_string(),
            "memd:host-auth-check:wrangler whoami".to_string(),
            "memd:host-auth-proof:local-probe".to_string(),
            "memd:host-auth-output-stored:false".to_string(),
        ],
        project: None,
        namespace: None,
        workspace: None,
        user_id: None,
        agent: None,
        updated_at: None,
    });

    state
        .store
        .upsert_capabilities(&CapabilitySyncRequest {
            project: Some(project.to_string()),
            namespace: Some(namespace.to_string()),
            workspace: Some("shared".to_string()),
            user_id: None,
            agent: None,
            records,
        })
        .expect("seed capabilities");

    let Json(packet) = get_context_packet(
        State(state.clone()),
        Query(ContextPacketRequest {
            project: Some(project.to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("shared".to_string()),
            visibility: None,
            route: None,
            intent: Some(RetrievalIntent::CurrentTask),
            limit: Some(8),
            max_chars_per_item: Some(520),
            model_tier: Some("tiny".to_string()),
            safety: Some("strict".to_string()),
            include_capabilities: true,
            include_access: true,
            include_hive: false,
        }),
    )
    .await
    .expect("context packet");

    assert!(
        packet.packet.contains("local:cli `opencode`"),
        "fresh harness packet should surface host-local CLI inventory before skill overflow"
    );
    assert!(
        packet.packet.contains("auth_status=unauthenticated"),
        "fresh harness packet should expose auth status without secret output"
    );
    assert!(
        packet.packet.contains("auth_check=opencode auth status"),
        "fresh harness packet should say how to verify or ask for access"
    );
    assert!(packet.packet.contains("local:cli `wrangler`"));
    assert!(packet.packet.contains("auth_status=authenticated"));
    assert!(!packet.packet.contains("stdout="));
    assert!(!packet.packet.contains("stderr="));

    std::fs::remove_dir_all(dir).expect("cleanup context host CLI temp dir");
}

#[tokio::test]
async fn hive_handoff_reaches_target_context_packet() {
    let (dir, state) = temp_state("memd-hive-handoff-context-packet-25-5");
    seed_hive_route_state(&state);
    let app = Router::new()
        .route("/hive/queen/handoff", post(post_hive_queen_handoff))
        .with_state(state.clone());

    let response = app
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
                        "scope": "crates/memd-client/src/main.rs",
                        "task_id": "parser-refactor",
                        "note": "Continue parser refactor; preserve Source IDs and ask for review before merge."
                    })
                    .to_string(),
                ))
                .expect("build handoff request"),
        )
        .await
        .expect("record hive handoff");
    assert_eq!(response.status(), StatusCode::OK);
    let handoff: HiveQueenActionResponse = decode_json(response).await;
    assert_eq!(handoff.action, "handoff");
    assert!(handoff.message_id.is_some());

    let Json(packet) = get_context_packet(
        State(state),
        Query(ContextPacketRequest {
            project: Some("memd".to_string()),
            agent: Some("bee-1".to_string()),
            workspace: Some("shared".to_string()),
            visibility: None,
            route: None,
            intent: Some(RetrievalIntent::CurrentTask),
            limit: Some(4),
            max_chars_per_item: Some(420),
            model_tier: Some("small".to_string()),
            safety: Some("strict".to_string()),
            include_capabilities: false,
            include_access: false,
            include_hive: true,
        }),
    )
    .await
    .expect("target handoff context packet");

    assert!(packet.packet.contains("## Hive Board"));
    assert!(packet.packet.contains("inbox kind=handoff"));
    assert!(
        packet
            .packet
            .contains("handoff_scope: crates/memd-client/src/main.rs")
    );
    assert!(packet.packet.contains("Continue parser refactor"));
    assert!(packet.packet.contains("sync=server"));
    assert!(
        packet
            .sections
            .iter()
            .find(|section| section.name == "Hive Board")
            .is_some_and(|section| section
                .lines
                .iter()
                .any(|line| line.contains("inbox kind=handoff")))
    );

    std::fs::remove_dir_all(dir).expect("cleanup hive handoff context packet temp dir");
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

#[tokio::test]
async fn search_memory_with_rag_acceptance_boosts_semantic_recall_and_outage_falls_back() {
    let (dir, state) = temp_state("memd-rag-25-5-acceptance");
    let project = "memd-25-5-with-rag";
    let namespace = "acceptance";

    let target = state
        .store_item(
            test_store_request(
                "Canonical memory: Mnemosyne is the sidecar vector mirror for palace-style semantic recall.",
                project,
                namespace,
            ),
            MemoryStage::Canonical,
        )
        .expect("seed semantic target")
        .0;
    let lexical = state
        .store_item(
            test_store_request(
                "Canonical memory: local fallback recall survives sidecar outage through FTS and fuzzy lanes.",
                project,
                namespace,
            ),
            MemoryStage::Canonical,
        )
        .expect("seed local fallback target")
        .0;

    let no_rag_search = test_search_request(
        "what stores conceptual echoes across conversations",
        project,
        namespace,
    );
    let Json(no_rag) = search_memory(State(state.clone()), Json(no_rag_search))
        .await
        .expect("search without rag");
    assert!(
        !no_rag.items.iter().any(|item| item.id == target.id),
        "sidecar-only semantic candidate should not appear before RAG contributes it"
    );

    let rag_url = spawn_mock_rag_search_server(
        RagRetrieveResponse {
            status: "ok".to_string(),
            mode: RagRetrieveMode::Text,
            items: vec![RagRetrieveItem {
                content: "semantic sidecar candidate".to_string(),
                source: Some(target.id.to_string()),
                score: 0.97,
            }],
        },
        RagRerankResponse {
            status: "ok".to_string(),
            model: "bge-reranker-base".to_string(),
            items: vec![RagRerankItem {
                id: target.id.to_string(),
                score: 0.93,
                text: None,
            }],
        },
    )
    .await;
    let _dense_guard = set_test_env("MEMD_RETRIEVAL_RAG_DENSE", "1");
    let rag_state = AppState {
        store: state.store.clone(),
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: Some(Arc::new(
            RagClient::new(&rag_url).expect("build acceptance rag client"),
        )),
        embedder: None,
    };
    let Json(with_rag) = search_memory(
        State(rag_state),
        Json(test_search_request(
            "what stores conceptual echoes across conversations",
            project,
            namespace,
        )),
    )
    .await
    .expect("search with rag");
    assert_eq!(with_rag.items.first().map(|item| item.id), Some(target.id));
    let trace = with_rag.trace.expect("with-rag trace");
    assert!(trace.lanes.iter().any(|lane| lane == "rag_dense"));
    assert!(
        trace
            .items
            .iter()
            .find(|item| item.id == target.id)
            .expect("target trace")
            .signals
            .iter()
            .any(|signal| signal.lane == "rag_dense")
    );

    let before_failures = crate::rag_bridge::rag_failure_count();
    let _timeout_guard = set_test_env("MEMD_RAG_TIMEOUT_MS", "1");
    let outage_state = AppState {
        store: state.store.clone(),
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: Some(Arc::new(
            RagClient::new("http://127.0.0.1:1").expect("build unreachable rag client"),
        )),
        embedder: None,
    };
    let Json(outage) = search_memory(
        State(outage_state),
        Json(test_search_request(
            "local fallback recall sidecar outage",
            project,
            namespace,
        )),
    )
    .await
    .expect("search during rag outage");
    assert_eq!(outage.items.first().map(|item| item.id), Some(lexical.id));
    assert!(
        crate::rag_bridge::rag_failure_count() > before_failures,
        "sidecar outage should be visible in failure telemetry"
    );
    let outage_trace = outage.trace.expect("outage trace");
    assert!(!outage_trace.lanes.iter().any(|lane| lane == "rag_dense"));

    std::fs::remove_dir_all(dir).expect("cleanup with-rag acceptance temp dir");
}

#[tokio::test]
async fn search_memory_with_rag_public_corpus_scores_boost_acl_and_truth_guard() {
    let (dir, state) = temp_state("memd-rag-public-corpus-25-5");
    let project = "memd-25-5-with-rag-corpus";
    let namespace = "public-corpus";

    let corpus = [
        (
            "restart",
            "Canonical memory: Aster capsules hold restart breadcrumbs for interrupted agent work.",
            "continue after crash without losing conversation state",
        ),
        (
            "harness",
            "Canonical memory: Boreal adapter matrix binds Claude Code, Codex, OpenCode, OpenClaw, Hermes, and Ollama to one authority.",
            "switch between AI harnesses while sharing memory",
        ),
        (
            "ollama",
            "Canonical memory: Cedar packets carry only labeled evidence and source ids into local LLM prompts.",
            "give Ollama safe compact context instead of raw dumps",
        ),
        (
            "sidecar",
            "Canonical memory: Elara mirror keeps vector recall additive while SQLite remains source of truth.",
            "semantic database can help recall but cannot override permissions",
        ),
        (
            "offline",
            "Canonical memory: Fjord queue stores failed writes locally and replays them after backend recovery.",
            "capture memories when server is down then sync later",
        ),
        (
            "aliases",
            "Canonical memory: Garnet aliases connect names, paths, commands, and project entities for recall.",
            "find misspelled owner file command identifiers",
        ),
        (
            "trace",
            "Canonical memory: Helio trace lists lexical fuzzy atlas dense trust recency and rerank evidence.",
            "explain why search result was chosen",
        ),
        (
            "multimodal",
            "Canonical memory: Ion intake can mirror compact canonical records for text and future multimodal sidecar recall.",
            "retrieve meaning from screenshots and notes without making rag required",
        ),
    ];

    let mut ids = std::collections::BTreeMap::new();
    let mut rag_by_query = std::collections::BTreeMap::new();
    for (key, content, query) in corpus {
        let mut req = test_store_request(content, project, namespace);
        req.tags = vec![format!("rag-corpus:{key}")];
        let item = state
            .store_item(req, MemoryStage::Canonical)
            .expect("store rag corpus item")
            .0;
        ids.insert(key.to_string(), item.id);
        rag_by_query.insert(
            query.to_string(),
            vec![RagRetrieveItem {
                content: format!("semantic candidate for {key}"),
                source: Some(item.id.to_string()),
                score: 0.98,
            }],
        );
    }

    let mut stale_req = test_store_request(
        "Stale fact: Icarus says relaxed packet mode owns local model safety.",
        project,
        namespace,
    );
    stale_req.status = Some(MemoryStatus::Stale);
    stale_req.source_quality = Some(SourceQuality::Derived);
    let stale = state
        .store_item(stale_req, MemoryStage::Canonical)
        .expect("store rag stale item")
        .0;
    let mut correction_req = test_store_request(
        "Corrected fact: Juno says strict packet mode owns local model safety.",
        project,
        namespace,
    );
    correction_req.source_system = Some("correction".to_string());
    correction_req.tags = vec!["correction".to_string()];
    correction_req.confidence = Some(0.96);
    let correction = state
        .store_item(correction_req, MemoryStage::Canonical)
        .expect("store rag correction item")
        .0;
    let truth_query = "who owns local model safety mode";
    rag_by_query.insert(
        truth_query.to_string(),
        vec![
            RagRetrieveItem {
                content: "sidecar stale candidate".to_string(),
                source: Some(stale.id.to_string()),
                score: 0.99,
            },
            RagRetrieveItem {
                content: "sidecar correction candidate".to_string(),
                source: Some(correction.id.to_string()),
                score: 0.80,
            },
        ],
    );

    let mut private_req = test_store_request(
        "Private Claude note: confidential sidecar candidate must not leak to Codex.",
        project,
        namespace,
    );
    private_req.visibility = Some(MemoryVisibility::Private);
    private_req.source_agent = Some("claude-code".to_string());
    let private = state
        .store_item(private_req, MemoryStage::Canonical)
        .expect("store rag private item")
        .0;
    let public_acl = state
        .store_item(
            test_store_request(
                "Canonical memory: Kilo public evidence proves sidecar candidates still pass memd visibility filters.",
                project,
                namespace,
            ),
            MemoryStage::Canonical,
        )
        .expect("store rag public acl item")
        .0;
    let acl_query = "retrieve confidential sidecar candidate safely";
    rag_by_query.insert(
        acl_query.to_string(),
        vec![
            RagRetrieveItem {
                content: "private high score candidate".to_string(),
                source: Some(private.id.to_string()),
                score: 0.99,
            },
            RagRetrieveItem {
                content: "public fallback candidate".to_string(),
                source: Some(public_acl.id.to_string()),
                score: 0.70,
            },
        ],
    );

    let queries = corpus
        .iter()
        .map(|(key, _, query)| (*key, *query))
        .collect::<Vec<_>>();
    let mut no_rag_top1 = 0usize;
    for (expected_key, query) in &queries {
        let expected_id = ids[*expected_key];
        let Json(response) = search_memory(
            State(state.clone()),
            Json(test_search_request(query, project, namespace)),
        )
        .await
        .expect("search rag corpus without sidecar");
        if response.items.first().map(|item| item.id) == Some(expected_id) {
            no_rag_top1 += 1;
        }
        if let Some(trace) = response.trace {
            assert!(
                !trace.lanes.iter().any(|lane| lane == "rag_dense"),
                "no-rag trace must not include rag_dense for {expected_key}"
            );
        }
    }

    let rag_url = spawn_mock_rag_query_corpus_server(rag_by_query).await;
    let _dense_guard = set_test_env("MEMD_RETRIEVAL_RAG_DENSE", "1");
    let rag_state = AppState {
        store: state.store.clone(),
        latency: crate::latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
        rag: Some(Arc::new(
            RagClient::new(&rag_url).expect("build rag corpus client"),
        )),
        embedder: None,
    };

    let mut rag_top1 = 0usize;
    for (expected_key, query) in &queries {
        let expected_id = ids[*expected_key];
        let Json(response) = search_memory(
            State(rag_state.clone()),
            Json(test_search_request(query, project, namespace)),
        )
        .await
        .expect("search rag corpus with sidecar");
        if response.items.first().map(|item| item.id) == Some(expected_id) {
            rag_top1 += 1;
        }
        let trace = response.trace.expect("rag corpus trace");
        assert!(trace.lanes.iter().any(|lane| lane == "rag_dense"));
        assert!(trace.lanes.iter().any(|lane| lane == "truth"));
        assert!(
            trace
                .items
                .iter()
                .find(|item| item.id == expected_id)
                .is_some_and(|item| item.signals.iter().any(|signal| signal.lane == "rag_dense")),
            "expected rag_dense trace for {expected_key}"
        );
    }

    assert!(
        rag_top1 > no_rag_top1,
        "RAG corpus should improve recall@1: no_rag={no_rag_top1} rag={rag_top1}"
    );
    assert_eq!(
        rag_top1,
        queries.len(),
        "RAG corpus should hit every mapped qrel at rank 1"
    );

    let Json(truth_response) = search_memory(
        State(rag_state.clone()),
        Json(test_search_request(truth_query, project, namespace)),
    )
    .await
    .expect("search rag truth guard");
    assert_eq!(
        truth_response.items.first().map(|item| item.id),
        Some(correction.id),
        "memd truth guard must rank correction over stale sidecar candidate"
    );
    assert!(
        truth_response
            .trace
            .expect("truth trace")
            .lanes
            .iter()
            .any(|lane| lane == "truth")
    );

    let Json(acl_response) = search_memory(
        State(rag_state),
        Json(test_search_request(acl_query, project, namespace)),
    )
    .await
    .expect("search rag acl guard");
    assert!(
        !acl_response.items.iter().any(|item| item.id == private.id),
        "private sidecar candidate must be filtered by memd ACL"
    );
    assert_eq!(
        acl_response.items.first().map(|item| item.id),
        Some(public_acl.id),
        "public candidate should survive after private candidate is filtered"
    );

    std::fs::remove_dir_all(dir).expect("cleanup rag public corpus temp dir");
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
