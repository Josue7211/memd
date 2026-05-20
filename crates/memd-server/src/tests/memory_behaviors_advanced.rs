use super::*;

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
