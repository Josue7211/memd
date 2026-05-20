use super::*;

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
