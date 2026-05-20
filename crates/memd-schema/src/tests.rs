use super::*;

#[test]
fn entity_record_roundtrips() {
    let record = MemoryEntityRecord {
        id: Uuid::new_v4(),
        entity_type: "repo".to_string(),
        aliases: vec!["memd".to_string(), "memd-core".to_string()],
        current_state: Some("main branch with multimodal stack".to_string()),
        state_version: 3,
        confidence: 0.93,
        salience_score: 0.82,
        rehearsal_count: 4,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_accessed_at: Some(Utc::now()),
        last_seen_at: Some(Utc::now()),
        valid_from: Some(Utc::now()),
        valid_to: None,
        tags: vec!["project".to_string(), "permanent".to_string()],
        context: Some(MemoryContextFrame {
            at: Some(Utc::now()),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("core".to_string()),
            repo: Some("memd".to_string()),
            host: Some("laptop".to_string()),
            branch: Some("main".to_string()),
            agent: Some("codex".to_string()),
            location: Some("home".to_string()),
        }),
    };

    let json = serde_json::to_string(&record).unwrap();
    let decoded: MemoryEntityRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.entity_type, "repo");
    assert_eq!(decoded.aliases.len(), 2);
}

#[test]
fn hive_session_roundtrips_worker_identity_fields() {
    let session = HiveSessionRecord {
        session: "session-lorentz".to_string(),
        tab_id: Some("tab-lorentz".to_string()),
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        workspace: Some("shared".to_string()),
        agent: Some("codex".to_string()),
        effective_agent: Some("codex@session-lorentz".to_string()),
        hive_system: Some("codex".to_string()),
        hive_role: Some("reviewer".to_string()),
        hive_groups: vec!["project:memd".to_string()],
        hive_group_goal: Some("review parser handoff".to_string()),
        authority: Some("participant".to_string()),
        heartbeat_model: Some("codex".to_string()),
        worker_name: Some("Lorentz".to_string()),
        display_name: Some("Parser Reviewer".to_string()),
        role: Some("reviewer".to_string()),
        capabilities: vec!["review".to_string(), "coordination".to_string()],
        lane_id: Some("lane-render-review".to_string()),
        repo_root: Some("/repo".to_string()),
        worktree_root: Some("/repo-review".to_string()),
        branch: Some("review/render".to_string()),
        base_branch: Some("main".to_string()),
        visibility: Some("workspace".to_string()),
        base_url: Some("http://127.0.0.1:8787".to_string()),
        base_url_healthy: Some(true),
        host: Some("workstation".to_string()),
        pid: Some(4242),
        topic_claim: Some("Review parser handoff".to_string()),
        scope_claims: vec!["crates/memd-client/src/main.rs".to_string()],
        task_id: Some("review-parser-handoff".to_string()),
        focus: Some("Review parser handoff".to_string()),
        pressure: Some("file_edited: crates/memd-client/src/main.rs".to_string()),
        next_recovery: Some("publish overlap-safe hive quickview".to_string()),
        next_action: Some("Review overlap guard output".to_string()),
        working: Some("yes".to_string()),
        touches: vec!["parser".to_string()],
        relationship_state: Some("coordinating".to_string()),
        relationship_peer: Some("hive".to_string()),
        relationship_reason: Some("parser handoff".to_string()),
        suggested_action: Some("review".to_string()),
        blocked_by: vec!["tests".to_string()],
        cowork_with: vec!["hive".to_string()],
        handoff_target: Some("codex".to_string()),
        offered_to: vec!["review".to_string()],
        status: "active".to_string(),
        needs_help: false,
        needs_review: false,
        handoff_state: Some("none".to_string()),
        confidence: Some("high".to_string()),
        risk: Some("low".to_string()),
        last_wake_at: None,
        last_seen: Utc::now(),
    };

    let json = serde_json::to_string(&session).expect("serialize session");
    let decoded: HiveSessionRecord = serde_json::from_str(&json).expect("deserialize session");
    assert_eq!(decoded.worker_name.as_deref(), Some("Lorentz"));
    assert_eq!(decoded.display_name.as_deref(), Some("Parser Reviewer"));
    assert_eq!(decoded.role.as_deref(), Some("reviewer"));
    assert_eq!(decoded.lane_id.as_deref(), Some("lane-render-review"));
    assert_eq!(
        decoded.next_action.as_deref(),
        Some("Review overlap guard output")
    );
    assert_eq!(decoded.risk.as_deref(), Some("low"));
}

#[test]
fn event_record_roundtrips() {
    let record = MemoryEventRecord {
        id: Uuid::new_v4(),
        entity_id: Some(Uuid::new_v4()),
        event_type: "rename".to_string(),
        summary: "repo renamed but entity stayed the same".to_string(),
        occurred_at: Utc::now(),
        recorded_at: Utc::now(),
        confidence: 0.88,
        salience_score: 0.74,
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        workspace: Some("core".to_string()),
        source_agent: Some("codex".to_string()),
        source_system: Some("cli".to_string()),
        source_path: Some("/tmp/memd".to_string()),
        related_entity_ids: vec![Uuid::new_v4()],
        tags: vec!["identity".to_string(), "timeline".to_string()],
        context: Some(MemoryContextFrame {
            at: Some(Utc::now()),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("core".to_string()),
            repo: Some("memd".to_string()),
            host: Some("laptop".to_string()),
            branch: Some("main".to_string()),
            agent: Some("codex".to_string()),
            location: Some("office".to_string()),
        }),
    };

    let json = serde_json::to_string(&record).unwrap();
    let decoded: MemoryEventRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.event_type, "rename");
    assert_eq!(decoded.tags.len(), 2);
}

#[test]
fn visible_memory_artifact_snapshot_round_trips() {
    let snapshot = VisibleMemorySnapshotResponse {
        generated_at: Utc::now(),
        home: VisibleMemoryHome {
            focus_artifact: VisibleMemoryArtifact {
                id: Uuid::new_v4(),
                title: "runtime spine".to_string(),
                body: "runtime spine is the canonical memory contract".to_string(),
                artifact_kind: "compiled_page".to_string(),
                memory_kind: Some(MemoryKind::Decision),
                scope: Some(MemoryScope::Project),
                visibility: Some(MemoryVisibility::Workspace),
                workspace: Some("team-alpha".to_string()),
                status: VisibleMemoryStatus::Current,
                freshness: "fresh".to_string(),
                confidence: 0.93,
                provenance: VisibleMemoryProvenance {
                    source_system: Some("obsidian".to_string()),
                    source_path: Some("wiki/runtime-spine.md".to_string()),
                    producer: Some("obsidian compile".to_string()),
                    trust_reason: "verified from compiled workspace page".to_string(),
                    last_verified_at: None,
                },
                sources: vec!["wiki/runtime-spine.md".to_string()],
                linked_artifact_ids: vec![],
                linked_sessions: vec!["codex-01".to_string()],
                linked_agents: vec!["codex".to_string()],
                repair_state: "healthy".to_string(),
                actions: vec!["inspect".to_string(), "verify_current".to_string()],
            },
            inbox_count: 3,
            repair_count: 1,
            awareness_count: 2,
        },
        knowledge_map: VisibleMemoryKnowledgeMap {
            nodes: vec![VisibleMemoryGraphNode {
                artifact_id: Uuid::new_v4(),
                title: "runtime spine".to_string(),
                artifact_kind: "compiled_page".to_string(),
                status: VisibleMemoryStatus::Current,
            }],
            edges: vec![VisibleMemoryGraphEdge {
                from: Uuid::new_v4(),
                to: Uuid::new_v4(),
                relation: "focus".to_string(),
            }],
        },
    };

    let json = serde_json::to_string(&snapshot).unwrap();
    let decoded: VisibleMemorySnapshotResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.home.focus_artifact.title, "runtime spine");
    assert_eq!(
        decoded.home.focus_artifact.status,
        VisibleMemoryStatus::Current
    );
    assert_eq!(decoded.home.inbox_count, 3);
    assert_eq!(decoded.home.repair_count, 1);
    assert_eq!(decoded.knowledge_map.nodes.len(), 1);
    assert_eq!(decoded.knowledge_map.edges.len(), 1);
}

#[test]
fn visible_memory_artifact_detail_round_trips() {
    let detail = VisibleMemoryArtifactDetailResponse {
        generated_at: Utc::now(),
        artifact: VisibleMemoryArtifact {
            id: Uuid::new_v4(),
            title: "runtime spine".to_string(),
            body: "runtime spine is the canonical memory contract".to_string(),
            artifact_kind: "compiled_page".to_string(),
            memory_kind: Some(MemoryKind::Decision),
            scope: Some(MemoryScope::Project),
            visibility: Some(MemoryVisibility::Workspace),
            workspace: Some("team-alpha".to_string()),
            status: VisibleMemoryStatus::Current,
            freshness: "fresh".to_string(),
            confidence: 0.93,
            provenance: VisibleMemoryProvenance {
                source_system: Some("obsidian".to_string()),
                source_path: Some("wiki/runtime-spine.md".to_string()),
                producer: Some("obsidian compile".to_string()),
                trust_reason: "verified from compiled workspace page".to_string(),
                last_verified_at: None,
            },
            sources: vec!["wiki/runtime-spine.md".to_string()],
            linked_artifact_ids: vec![],
            linked_sessions: vec!["codex-01".to_string()],
            linked_agents: vec!["codex".to_string()],
            repair_state: "healthy".to_string(),
            actions: vec!["inspect".to_string(), "verify_current".to_string()],
        },
        explain: None,
        timeline: None,
        sources: SourceMemoryResponse { sources: vec![] },
        workspaces: WorkspaceMemoryResponse { workspaces: vec![] },
        sessions: HiveSessionsResponse { sessions: vec![] },
        tasks: HiveTasksResponse { tasks: vec![] },
        claims: HiveClaimsResponse { claims: vec![] },
        related_artifacts: vec![],
        related_map: VisibleMemoryKnowledgeMap {
            nodes: vec![],
            edges: vec![],
        },
        actions: vec![
            VisibleMemoryUiActionKind::Inspect,
            VisibleMemoryUiActionKind::Explain,
            VisibleMemoryUiActionKind::VerifyCurrent,
        ],
    };

    let json = serde_json::to_string(&detail).unwrap();
    let decoded: VisibleMemoryArtifactDetailResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.artifact.title, "runtime spine");
    assert_eq!(decoded.actions.len(), 3);
}

#[test]
fn visible_memory_ui_action_round_trips() {
    let request = VisibleMemoryUiActionRequest {
        id: Uuid::new_v4(),
        action: VisibleMemoryUiActionKind::OpenInObsidian,
    };
    let response = VisibleMemoryUiActionResponse {
        action: VisibleMemoryUiActionKind::OpenInObsidian,
        artifact_id: request.id,
        outcome: "opened".to_string(),
        message: "generated obsidian uri".to_string(),
        detail: None,
        open_uri: Some("obsidian://open?path=wiki/runtime-spine.md".to_string()),
        source_path: Some("wiki/runtime-spine.md".to_string()),
    };

    let request_json = serde_json::to_string(&request).unwrap();
    let response_json = serde_json::to_string(&response).unwrap();
    let decoded_request: VisibleMemoryUiActionRequest =
        serde_json::from_str(&request_json).unwrap();
    let decoded_response: VisibleMemoryUiActionResponse =
        serde_json::from_str(&response_json).unwrap();

    assert_eq!(
        decoded_request.action,
        VisibleMemoryUiActionKind::OpenInObsidian
    );
    assert_eq!(decoded_response.artifact_id, request.id);
    assert_eq!(
        decoded_response.open_uri.as_deref(),
        response.open_uri.as_deref()
    );
}

#[test]
fn entity_search_roundtrips() {
    let entity = MemoryEntityRecord {
        id: Uuid::new_v4(),
        entity_type: "repo".to_string(),
        aliases: vec!["memd".to_string()],
        current_state: Some("main".to_string()),
        state_version: 1,
        confidence: 0.8,
        salience_score: 0.7,
        rehearsal_count: 2,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_accessed_at: Some(Utc::now()),
        last_seen_at: Some(Utc::now()),
        valid_from: Some(Utc::now()),
        valid_to: None,
        tags: vec!["project".to_string()],
        context: Some(MemoryContextFrame {
            at: Some(Utc::now()),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("core".to_string()),
            repo: Some("memd".to_string()),
            host: Some("laptop".to_string()),
            branch: Some("main".to_string()),
            agent: Some("codex".to_string()),
            location: Some("/tmp/memd".to_string()),
        }),
    };
    let request = EntitySearchRequest {
        query: "memd repo".to_string(),
        project: Some("memd".to_string()),
        namespace: None,
        at: Some(Utc::now()),
        host: Some("laptop".to_string()),
        branch: Some("main".to_string()),
        location: Some("/tmp/memd".to_string()),
        route: Some(RetrievalRoute::ProjectFirst),
        intent: Some(RetrievalIntent::Fact),
        limit: Some(5),
    };
    let response = EntitySearchResponse {
        route: RetrievalRoute::ProjectFirst,
        intent: RetrievalIntent::Fact,
        query: request.query.clone(),
        best_match: Some(EntitySearchHit {
            entity,
            score: 0.93,
            reasons: vec!["alias match".to_string()],
        }),
        candidates: Vec::new(),
        ambiguous: false,
    };

    let request_json = serde_json::to_string(&request).unwrap();
    let response_json = serde_json::to_string(&response).unwrap();
    let decoded_request: EntitySearchRequest = serde_json::from_str(&request_json).unwrap();
    let decoded_response: EntitySearchResponse = serde_json::from_str(&response_json).unwrap();
    assert_eq!(decoded_request.query, request.query);
    assert_eq!(decoded_response.best_match.unwrap().score, 0.93);
}

#[test]
fn entity_link_roundtrips() {
    let link = MemoryEntityLinkRecord {
        id: Uuid::new_v4(),
        from_entity_id: Uuid::new_v4(),
        to_entity_id: Uuid::new_v4(),
        relation_kind: EntityRelationKind::DerivedFrom,
        confidence: 0.84,
        created_at: Utc::now(),
        valid_from: Some(Utc::now()),
        valid_to: None,
        source_item_id: Some(Uuid::new_v4()),
        note: Some("rolled up from repeated traces".to_string()),
        context: Some(MemoryContextFrame {
            at: Some(Utc::now()),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("core".to_string()),
            repo: Some("memd".to_string()),
            host: Some("laptop".to_string()),
            branch: Some("main".to_string()),
            agent: Some("codex".to_string()),
            location: Some("/tmp/memd".to_string()),
        }),
        tags: vec!["semantic".to_string()],
    };
    let request = EntityLinkRequest {
        from_entity_id: link.from_entity_id,
        to_entity_id: link.to_entity_id,
        relation_kind: link.relation_kind,
        confidence: Some(link.confidence),
        valid_from: link.valid_from,
        valid_to: link.valid_to,
        source_item_id: link.source_item_id,
        note: link.note.clone(),
        context: link.context.clone(),
        tags: link.tags.clone(),
    };
    let response = EntityLinkResponse { link: link.clone() };
    let links = EntityLinksResponse {
        entity_id: link.from_entity_id,
        links: vec![link],
    };

    let request_json = serde_json::to_string(&request).unwrap();
    let response_json = serde_json::to_string(&response).unwrap();
    let links_json = serde_json::to_string(&links).unwrap();
    let decoded_request: EntityLinkRequest = serde_json::from_str(&request_json).unwrap();
    let decoded_response: EntityLinkResponse = serde_json::from_str(&response_json).unwrap();
    let decoded_links: EntityLinksResponse = serde_json::from_str(&links_json).unwrap();
    assert_eq!(decoded_request.relation_kind, request.relation_kind);
    assert_eq!(decoded_response.link.confidence, response.link.confidence);
    assert_eq!(decoded_links.links.len(), 1);
}

#[test]
fn associative_recall_roundtrips() {
    let root = MemoryEntityRecord {
        id: Uuid::new_v4(),
        entity_type: "repo".to_string(),
        aliases: vec!["memd".to_string()],
        current_state: Some("working memory".to_string()),
        state_version: 2,
        confidence: 0.89,
        salience_score: 0.77,
        rehearsal_count: 5,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_accessed_at: Some(Utc::now()),
        last_seen_at: Some(Utc::now()),
        valid_from: Some(Utc::now()),
        valid_to: None,
        tags: vec!["project".to_string()],
        context: Some(MemoryContextFrame {
            at: Some(Utc::now()),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("core".to_string()),
            repo: Some("memd".to_string()),
            host: Some("laptop".to_string()),
            branch: Some("main".to_string()),
            agent: Some("codex".to_string()),
            location: Some("/tmp/memd".to_string()),
        }),
    };
    let link = MemoryEntityLinkRecord {
        id: Uuid::new_v4(),
        from_entity_id: root.id,
        to_entity_id: Uuid::new_v4(),
        relation_kind: EntityRelationKind::Related,
        confidence: 0.7,
        created_at: Utc::now(),
        valid_from: Some(Utc::now()),
        valid_to: None,
        source_item_id: Some(Uuid::new_v4()),
        note: Some("adjacent memory".to_string()),
        context: root.context.clone(),
        tags: vec!["graph".to_string()],
    };
    let request = AssociativeRecallRequest {
        entity_id: root.id,
        depth: Some(2),
        limit: Some(6),
    };
    let response = AssociativeRecallResponse {
        root_entity: Some(root.clone()),
        hits: vec![AssociativeRecallHit {
            entity: root,
            depth: 0,
            via: None,
            score: 1.0,
            reasons: vec!["root".to_string()],
        }],
        links: vec![link],
        truncated: false,
    };

    let request_json = serde_json::to_string(&request).unwrap();
    let response_json = serde_json::to_string(&response).unwrap();
    let decoded_request: AssociativeRecallRequest = serde_json::from_str(&request_json).unwrap();
    let decoded_response: AssociativeRecallResponse = serde_json::from_str(&response_json).unwrap();
    assert_eq!(decoded_request.depth, request.depth);
    assert_eq!(decoded_response.links.len(), 1);
    assert_eq!(decoded_response.hits.len(), 1);
    assert_eq!(decoded_response.hits[0].score, 1.0);
}

#[test]
fn consolidation_request_roundtrips() {
    let request = MemoryConsolidationRequest {
        project: Some("memd".to_string()),
        namespace: Some("agent".to_string()),
        max_groups: Some(12),
        min_events: Some(3),
        lookback_days: Some(14),
        min_salience: Some(0.25),
        record_events: Some(true),
    };

    let json = serde_json::to_string(&request).unwrap();
    let decoded: MemoryConsolidationRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.project, request.project);
    assert_eq!(decoded.min_events, request.min_events);
    assert_eq!(decoded.record_events, request.record_events);
}

#[test]
fn consolidation_response_roundtrips() {
    let response = MemoryConsolidationResponse {
        scanned: 42,
        groups: 7,
        consolidated: 3,
        duplicates: 1,
        events: 3,
        highlights: vec!["repo:3 events".to_string()],
        mean_quality: None,
        quality_scores: vec![],
    };

    let json = serde_json::to_string(&response).unwrap();
    let decoded: MemoryConsolidationResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.scanned, response.scanned);
    assert_eq!(decoded.consolidated, response.consolidated);
    assert_eq!(decoded.events, response.events);
    assert_eq!(decoded.highlights, response.highlights);
}

#[test]
fn maintenance_report_roundtrips() {
    let request = MemoryMaintenanceReportRequest {
        project: Some("memd".to_string()),
        namespace: Some("agent".to_string()),
        inactive_days: Some(21),
        lookback_days: Some(14),
        min_events: Some(3),
        max_decay: Some(0.12),
        mode: Some("scan".to_string()),
        apply: Some(false),
    };

    let response = MemoryMaintenanceReportResponse {
        reinforced_candidates: 9,
        cooled_candidates: 4,
        consolidated_candidates: 2,
        stale_items: 11,
        skipped: 2,
        highlights: vec!["repo:3 events".to_string()],
        receipt_id: Some("receipt-1".to_string()),
        mode: Some("scan".to_string()),
        compacted_items: 2,
        refreshed_items: 4,
        repaired_items: 1,
        generated_at: Utc::now(),
    };

    let request_json = serde_json::to_string(&request).unwrap();
    let response_json = serde_json::to_string(&response).unwrap();
    let decoded_request: MemoryMaintenanceReportRequest =
        serde_json::from_str(&request_json).unwrap();
    let decoded_response: MemoryMaintenanceReportResponse =
        serde_json::from_str(&response_json).unwrap();
    assert_eq!(decoded_request.project, request.project);
    assert_eq!(decoded_response.stale_items, response.stale_items);
    assert_eq!(decoded_response.skipped, response.skipped);
    assert_eq!(decoded_response.highlights, response.highlights);
    assert_eq!(decoded_response.mode, response.mode);
    assert_eq!(decoded_response.receipt_id, response.receipt_id);
    assert_eq!(decoded_response.compacted_items, response.compacted_items);
    assert_eq!(decoded_response.refreshed_items, response.refreshed_items);
    assert_eq!(decoded_response.repaired_items, response.repaired_items);
}

#[test]
fn maintain_report_roundtrips() {
    let request = MaintainReportRequest {
        project: Some("memd".to_string()),
        namespace: Some("agent".to_string()),
        workspace: Some("shared".to_string()),
        session: Some("session-a".to_string()),
        mode: "scan".to_string(),
        apply: false,
    };

    let response = MaintainReport {
        mode: "scan".to_string(),
        receipt_id: Some("receipt-1".to_string()),
        compacted_items: 3,
        refreshed_items: 2,
        repaired_items: 1,
        findings: vec!["memory scan complete".to_string()],
        generated_at: Utc::now(),
    };

    let request_json = serde_json::to_string(&request).unwrap();
    let response_json = serde_json::to_string(&response).unwrap();
    let decoded_request: MaintainReportRequest = serde_json::from_str(&request_json).unwrap();
    let decoded_response: MaintainReport = serde_json::from_str(&response_json).unwrap();
    assert_eq!(decoded_request.mode, request.mode);
    assert_eq!(decoded_request.apply, request.apply);
    assert_eq!(decoded_request.workspace, request.workspace);
    assert_eq!(decoded_response.mode, response.mode);
    assert_eq!(decoded_response.receipt_id, response.receipt_id);
    assert_eq!(decoded_response.compacted_items, response.compacted_items);
    assert_eq!(decoded_response.findings, response.findings);
}

#[test]
fn explain_response_roundtrips() {
    let now = Utc::now();
    let response = ExplainMemoryResponse {
        route: RetrievalRoute::ProjectFirst,
        intent: RetrievalIntent::Decision,
        item: MemoryItem {
            id: Uuid::new_v4(),
            content: "prefer bundle-first config".to_string(),
            redundancy_key: Some("decision:bundle-first".to_string()),
            belief_branch: Some("mainline".to_string()),
            preferred: true,
            kind: MemoryKind::Decision,
            scope: MemoryScope::Project,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("core".to_string()),
            visibility: MemoryVisibility::Workspace,
            source_agent: Some("codex".to_string()),
            source_system: Some("cli".to_string()),
            source_path: Some("/tmp/memd/docs/core/rag.md".to_string()),
            source_quality: Some(SourceQuality::Canonical),
            confidence: 0.92,
            ttl_seconds: None,
            created_at: now,
            updated_at: now,
            last_verified_at: Some(now),
            supersedes: vec![],
            tags: vec!["decision".to_string()],
            status: MemoryStatus::Active,
            stage: MemoryStage::Canonical,
            lane: None,
            version: 1,
            correction_meta: None,
        },
        canonical_key: "decision:bundle-first".to_string(),
        redundancy_key: "decision:bundle-first".to_string(),
        reasons: vec!["route=project_first".to_string()],
        entity: None,
        events: vec![],
        sources: vec![SourceMemoryRecord {
            source_agent: Some("codex".to_string()),
            source_system: Some("cli".to_string()),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("core".to_string()),
            visibility: MemoryVisibility::Workspace,
            item_count: 4,
            active_count: 4,
            candidate_count: 0,
            derived_count: 0,
            synthetic_count: 0,
            contested_count: 0,
            avg_confidence: 0.91,
            trust_score: 0.95,
            last_seen_at: Some(now),
            tags: vec!["docs".to_string()],
        }],
        retrieval_feedback: RetrievalFeedbackSummary {
            total_retrievals: 4,
            last_retrieved_at: Some(now),
            by_surface: vec![
                RetrievalFeedbackSurfaceCount {
                    surface: "explain".to_string(),
                    count: 2,
                },
                RetrievalFeedbackSurfaceCount {
                    surface: "working".to_string(),
                    count: 2,
                },
            ],
            recent_policy_hooks: vec![
                "route=project_first".to_string(),
                "intent=decision".to_string(),
            ],
        },
        branch_siblings: vec![ExplainBranchSiblingRecord {
            id: Uuid::new_v4(),
            belief_branch: Some("fallback".to_string()),
            preferred: false,
            status: MemoryStatus::Contested,
            stage: MemoryStage::Canonical,
            confidence: 0.71,
            updated_at: now,
        }],
        rehydration: vec![MemoryRehydrationRecord {
            id: Some(Uuid::new_v4()),
            kind: "memory_item".to_string(),
            label: "canonical memory".to_string(),
            summary: "prefer bundle-first config".to_string(),
            reason: Some("rehydrate_primary_memory".to_string()),
            source_agent: Some("codex".to_string()),
            source_system: Some("cli".to_string()),
            source_path: Some("/tmp/memd/docs/core/rag.md".to_string()),
            source_quality: Some(SourceQuality::Canonical),
            recorded_at: Some(now),
        }],
        policy_hooks: vec![
            "route=project_first".to_string(),
            "intent=decision".to_string(),
            "source_trust_floor=0.60".to_string(),
        ],
        corrections_chain: vec![CorrectionChainEntry {
            id: Uuid::new_v4(),
            content_preview: "prior revision of decision".to_string(),
            confidence: 0.81,
            stage: MemoryStage::Canonical,
            status: MemoryStatus::Superseded,
            updated_at: now,
            supersedes: vec![],
            correction_source_turn: Some("t-12".to_string()),
        }],
        confidence_timeline: vec![ConfidenceSample {
            at: now,
            confidence: 0.92,
            source: "created".to_string(),
        }],
        trust_rank_history: vec![TrustRankSample {
            at: now,
            source_agent: Some("codex".to_string()),
            source_system: Some("cli".to_string()),
            event_type: "verify".to_string(),
            confidence: 0.9,
        }],
    };

    let json = serde_json::to_string(&response).unwrap();
    let decoded: ExplainMemoryResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.item.belief_branch.as_deref(), Some("mainline"));
    assert_eq!(decoded.retrieval_feedback.total_retrievals, 4);
    assert_eq!(decoded.branch_siblings.len(), 1);
    assert_eq!(decoded.rehydration.len(), 1);
    assert_eq!(decoded.policy_hooks.len(), 3);
    assert_eq!(decoded.sources[0].trust_score, 0.95);
    assert_eq!(decoded.corrections_chain.len(), 1);
    assert_eq!(decoded.confidence_timeline.len(), 1);
    assert_eq!(decoded.trust_rank_history.len(), 1);
}

#[test]
fn policy_response_roundtrips() {
    let response = MemoryPolicyResponse {
        retrieval_order: vec![
            MemoryScope::Local,
            MemoryScope::Synced,
            MemoryScope::Project,
            MemoryScope::Global,
        ],
        route_defaults: vec![
            MemoryPolicyRouteDefault {
                intent: RetrievalIntent::CurrentTask,
                route: RetrievalRoute::LocalFirst,
            },
            MemoryPolicyRouteDefault {
                intent: RetrievalIntent::Preference,
                route: RetrievalRoute::GlobalFirst,
            },
        ],
        working_memory: MemoryPolicyWorkingMemory {
            budget_chars: 1600,
            max_chars_per_item: 220,
            default_limit: 8,
            rehydration_limit: 3,
        },
        retrieval_feedback: MemoryPolicyFeedback {
            enabled: true,
            tracked_surfaces: vec![
                "search".to_string(),
                "context".to_string(),
                "working".to_string(),
                "explain".to_string(),
            ],
            max_items_per_request: 3,
        },
        source_trust_floor: 0.6,
        runtime: MemoryPolicyRuntime {
            live_truth: MemoryPolicyLiveTruth {
                read_once_sources: true,
                raw_reopen_requires_change_or_doubt: true,
                visible_memory_objects: true,
                compile_from_events: true,
            },
            memory_compilation: MemoryPolicyMemoryCompilation {
                event_driven_updates: true,
                patch_not_rewrite: true,
                preserve_provenance: true,
                source_on_demand: true,
            },
            semantic_fallback: MemoryPolicySemanticFallback {
                enabled: true,
                source_of_truth: false,
                max_items_per_query: 3,
                rerank_with_visible_memory: true,
            },
            skill_gating: MemoryPolicySkillGating {
                propose_from_repeated_patterns: true,
                sandboxed_evaluation: true,
                auto_activate_low_risk_only: true,
                gated_activation: true,
                require_evaluation: true,
                require_policy_approval: true,
            },
        },
        promotion: MemoryPolicyPromotion {
            min_salience: 0.22,
            min_events: 3,
            lookback_days: 14,
            default_ttl_days: 90,
        },
        decay: MemoryPolicyDecay {
            max_items: 128,
            inactive_days: 21,
            max_decay: 0.12,
            decay_divisor: 14.0,
            record_events: true,
        },
        consolidation: MemoryPolicyConsolidation {
            max_groups: 24,
            min_events: 3,
            lookback_days: 14,
            min_salience: 0.22,
            record_events: true,
        },
    };

    let json = serde_json::to_string(&response).unwrap();
    let decoded: MemoryPolicyResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.retrieval_order, response.retrieval_order);
    assert_eq!(decoded.working_memory.default_limit, 8);
    assert!(decoded.retrieval_feedback.enabled);
    assert_eq!(decoded.source_trust_floor, 0.6);
    assert!(decoded.runtime.live_truth.read_once_sources);
    assert!(decoded.runtime.skill_gating.gated_activation);
    assert!(decoded.runtime.skill_gating.sandboxed_evaluation);
    assert_eq!(decoded.decay.max_decay, 0.12);
    assert_eq!(decoded.consolidation.max_groups, 24);
}

#[test]
fn working_memory_roundtrips() {
    let request = WorkingMemoryRequest {
        project: Some("memd".to_string()),
        agent: Some("codex".to_string()),
        workspace: Some("core".to_string()),
        visibility: Some(MemoryVisibility::Workspace),
        route: Some(RetrievalRoute::ProjectFirst),
        intent: Some(RetrievalIntent::CurrentTask),
        limit: Some(4),
        max_chars_per_item: Some(180),
        max_total_chars: Some(900),
        rehydration_limit: Some(2),
        auto_consolidate: Some(true),
        query: Some("system architecture".to_string()),
    };

    let response = WorkingMemoryResponse {
        route: RetrievalRoute::ProjectFirst,
        intent: RetrievalIntent::CurrentTask,
        retrieval_order: vec![MemoryScope::Project, MemoryScope::Synced],
        budget_chars: 900,
        used_chars: 612,
        remaining_chars: 288,
        truncated: false,
        policy: WorkingMemoryPolicyState {
            admission_limit: 4,
            max_chars_per_item: 180,
            budget_chars: 900,
            rehydration_limit: 2,
        },
        records: vec![CompactMemoryRecord {
            id: Uuid::new_v4(),
            record: "focus on the working set".to_string(),
        }],
        evicted: vec![WorkingMemoryEvictionRecord {
            id: Uuid::new_v4(),
            record: "older context left the hot set".to_string(),
            reason: "evicted_by_budget".to_string(),
        }],
        rehydration_queue: vec![MemoryRehydrationRecord {
            id: Some(Uuid::new_v4()),
            kind: "working_memory_record".to_string(),
            label: "evicted working-set item".to_string(),
            summary: "older context left the hot set".to_string(),
            reason: Some("evicted_by_budget".to_string()),
            source_agent: Some("codex".to_string()),
            source_system: Some("cli".to_string()),
            source_path: Some("/tmp/memd/notes.md".to_string()),
            source_quality: Some(SourceQuality::Derived),
            recorded_at: Some(Utc::now()),
        }],
        traces: vec![WorkingMemoryTraceRecord {
            item_id: Uuid::new_v4(),
            entity_id: Some(Uuid::new_v4()),
            memory_kind: MemoryKind::Decision,
            memory_stage: MemoryStage::Canonical,
            typed_memory: "semantic+canonical".to_string(),
            event_type: "retrieved".to_string(),
            summary: "working set refreshed".to_string(),
            occurred_at: Utc::now(),
            salience_score: 0.81,
        }],
        semantic_consolidation: Some(MemoryConsolidationResponse {
            scanned: 3,
            groups: 1,
            consolidated: 1,
            duplicates: 0,
            events: 1,
            highlights: vec!["working-set replay".to_string()],
            mean_quality: None,
            quality_scores: vec![],
        }),
        procedures: vec![],
        compaction_quality: None,
    };

    let request_json = serde_json::to_string(&request).unwrap();
    let response_json = serde_json::to_string(&response).unwrap();
    let decoded_request: WorkingMemoryRequest = serde_json::from_str(&request_json).unwrap();
    let decoded_response: WorkingMemoryResponse = serde_json::from_str(&response_json).unwrap();
    assert_eq!(decoded_request.limit, request.limit);
    assert_eq!(decoded_request.rehydration_limit, request.rehydration_limit);
    assert_eq!(decoded_response.budget_chars, response.budget_chars);
    assert_eq!(decoded_response.policy.admission_limit, 4);
    assert_eq!(decoded_response.records.len(), 1);
    assert_eq!(decoded_response.evicted.len(), 1);
    assert_eq!(decoded_response.rehydration_queue.len(), 1);
    assert_eq!(decoded_response.traces.len(), 1);
    assert!(decoded_response.semantic_consolidation.is_some());
}

#[test]
fn agent_profile_roundtrips() {
    let request = AgentProfileUpsertRequest {
        agent: "codex".to_string(),
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        preferred_route: Some(RetrievalRoute::ProjectFirst),
        preferred_intent: Some(RetrievalIntent::CurrentTask),
        summary_chars: Some(160),
        max_total_chars: Some(1200),
        recall_depth: Some(2),
        source_trust_floor: Some(0.6),
        style_tags: vec!["concise".to_string(), "token-cheap".to_string()],
        notes: Some("prefer tight working sets".to_string()),
    };
    let profile = MemoryAgentProfile {
        id: Uuid::new_v4(),
        agent: "codex".to_string(),
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        preferred_route: Some(RetrievalRoute::ProjectFirst),
        preferred_intent: Some(RetrievalIntent::CurrentTask),
        summary_chars: Some(160),
        max_total_chars: Some(1200),
        recall_depth: Some(2),
        source_trust_floor: Some(0.6),
        style_tags: vec!["concise".to_string(), "token-cheap".to_string()],
        notes: Some("prefer tight working sets".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let response = AgentProfileResponse {
        profile: Some(profile),
    };

    let request_json = serde_json::to_string(&request).unwrap();
    let response_json = serde_json::to_string(&response).unwrap();
    let decoded_request: AgentProfileUpsertRequest = serde_json::from_str(&request_json).unwrap();
    let decoded_response: AgentProfileResponse = serde_json::from_str(&response_json).unwrap();
    assert_eq!(decoded_request.summary_chars, request.summary_chars);
    assert_eq!(
        decoded_response.profile.as_ref().unwrap().summary_chars,
        Some(160)
    );
}

#[test]
fn source_memory_roundtrips() {
    let request = SourceMemoryRequest {
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        workspace: Some("core".to_string()),
        visibility: Some(MemoryVisibility::Workspace),
        source_agent: Some("codex".to_string()),
        source_system: Some("cli".to_string()),
        limit: Some(10),
    };
    let response = SourceMemoryResponse {
        sources: vec![SourceMemoryRecord {
            source_agent: Some("codex".to_string()),
            source_system: Some("cli".to_string()),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("core".to_string()),
            visibility: MemoryVisibility::Workspace,
            item_count: 12,
            active_count: 10,
            candidate_count: 2,
            derived_count: 4,
            synthetic_count: 0,
            contested_count: 1,
            avg_confidence: 0.84,
            trust_score: 0.91,
            last_seen_at: Some(Utc::now()),
            tags: vec!["agent".to_string(), "cli".to_string()],
        }],
    };

    let request_json = serde_json::to_string(&request).unwrap();
    let response_json = serde_json::to_string(&response).unwrap();
    let decoded_request: SourceMemoryRequest = serde_json::from_str(&request_json).unwrap();
    let decoded_response: SourceMemoryResponse = serde_json::from_str(&response_json).unwrap();
    assert_eq!(decoded_request.source_agent, request.source_agent);
    assert_eq!(decoded_response.sources[0].trust_score, 0.91);
}

#[test]
fn workspace_memory_roundtrips() {
    let request = WorkspaceMemoryRequest {
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        workspace: Some("core".to_string()),
        visibility: Some(MemoryVisibility::Workspace),
        source_agent: Some("obsidian".to_string()),
        source_system: Some("obsidian".to_string()),
        limit: Some(8),
    };
    let response = WorkspaceMemoryResponse {
        workspaces: vec![WorkspaceMemoryRecord {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("core".to_string()),
            visibility: MemoryVisibility::Workspace,
            item_count: 12,
            active_count: 10,
            candidate_count: 1,
            contested_count: 1,
            source_lane_count: 2,
            avg_confidence: 0.86,
            trust_score: 0.9,
            last_seen_at: Some(Utc::now()),
            tags: vec!["obsidian".to_string(), "shared".to_string()],
        }],
    };

    let request_json = serde_json::to_string(&request).unwrap();
    let response_json = serde_json::to_string(&response).unwrap();
    let decoded_request: WorkspaceMemoryRequest = serde_json::from_str(&request_json).unwrap();
    let decoded_response: WorkspaceMemoryResponse = serde_json::from_str(&response_json).unwrap();
    assert_eq!(decoded_request.workspace, request.workspace);
    assert_eq!(decoded_response.workspaces.len(), 1);
    assert_eq!(decoded_response.workspaces[0].source_lane_count, 2);
}

#[test]
fn procedural_and_self_model_enums_roundtrip() {
    let kind_json = serde_json::to_string(&MemoryKind::Procedural).unwrap();
    let intent_json = serde_json::to_string(&RetrievalIntent::SelfModel).unwrap();

    assert_eq!(kind_json, "\"procedural\"");
    assert_eq!(intent_json, "\"self_model\"");
    assert_eq!(
        serde_json::from_str::<MemoryKind>(&kind_json).unwrap(),
        MemoryKind::Procedural
    );
    assert_eq!(
        serde_json::from_str::<RetrievalIntent>(&intent_json).unwrap(),
        RetrievalIntent::SelfModel
    );
}

#[test]
fn repair_contract_roundtrips() {
    let request = RepairMemoryRequest {
        id: Uuid::new_v4(),
        mode: MemoryRepairMode::CorrectMetadata,
        confidence: Some(0.91),
        status: Some(MemoryStatus::Active),
        workspace: Some("team-alpha".to_string()),
        visibility: Some(MemoryVisibility::Workspace),
        source_agent: Some("codex".to_string()),
        source_system: Some("cli".to_string()),
        source_path: Some("/tmp/memd".to_string()),
        source_quality: Some(SourceQuality::Canonical),
        content: Some("repaired memory content".to_string()),
        tags: Some(vec!["repair".to_string(), "audit".to_string()]),
        supersedes: vec![Uuid::new_v4(), Uuid::new_v4()],
    };
    let response = RepairMemoryResponse {
        item: MemoryItem {
            id: request.id,
            content: "repaired memory content".to_string(),
            redundancy_key: Some("dedupe:key".to_string()),
            belief_branch: Some("mainline".to_string()),
            preferred: false,
            kind: MemoryKind::Decision,
            scope: MemoryScope::Project,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("core".to_string()),
            visibility: MemoryVisibility::Workspace,
            source_agent: request.source_agent.clone(),
            source_system: request.source_system.clone(),
            source_path: request.source_path.clone(),
            source_quality: request.source_quality,
            confidence: 0.91,
            ttl_seconds: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_verified_at: Some(Utc::now()),
            supersedes: request.supersedes.clone(),
            tags: vec!["repair".to_string(), "audit".to_string()],
            status: MemoryStatus::Active,
            stage: MemoryStage::Canonical,
            lane: None,
            version: 1,
            correction_meta: None,
        },
        mode: request.mode,
        reasons: vec![
            "mode=correct_metadata".to_string(),
            "source_agent_updated".to_string(),
            "content_repaired".to_string(),
        ],
    };

    let request_json = serde_json::to_string(&request).unwrap();
    let response_json = serde_json::to_string(&response).unwrap();
    let decoded_request: RepairMemoryRequest = serde_json::from_str(&request_json).unwrap();
    let decoded_response: RepairMemoryResponse = serde_json::from_str(&response_json).unwrap();
    assert_eq!(decoded_request.mode, MemoryRepairMode::CorrectMetadata);
    assert_eq!(decoded_request.tags.as_ref().unwrap().len(), 2);
    assert_eq!(decoded_response.mode, MemoryRepairMode::CorrectMetadata);
    assert_eq!(decoded_response.item.status, MemoryStatus::Active);
    assert_eq!(decoded_response.reasons.len(), 3);
}

#[test]
fn benchmark_registry_roundtrips_minimal_continuity_slice() {
    let registry = BenchmarkRegistry {
        version: "v1".to_string(),
        app_goal: "seamless memory and continuity".to_string(),
        quality_dimensions: vec![
            QualityDimensionRecord {
                id: "continuity".to_string(),
                weight: 25,
            },
            QualityDimensionRecord {
                id: "correctness".to_string(),
                weight: 20,
            },
        ],
        tiers: vec![TierRecord {
            id: "tier-0-continuity-critical".to_string(),
            description: "continuity-critical surfaces".to_string(),
        }],
        pillars: vec![PillarRecord {
            id: "memory-continuity".to_string(),
            description: "core continuity promise".to_string(),
        }],
        families: vec![FamilyRecord {
            id: "bundle-runtime".to_string(),
            pillar: "memory-continuity".to_string(),
            description: "bundle continuity surfaces".to_string(),
        }],
        features: vec![BenchmarkFeatureRecord {
            id: "feature.bundle.resume".to_string(),
            name: "Resume".to_string(),
            pillar: "memory-continuity".to_string(),
            family: "bundle-runtime".to_string(),
            tier: "tier-0-continuity-critical".to_string(),
            continuity_critical: true,
            user_contract: "resume restores usable continuity".to_string(),
            source_contract_refs: vec!["FEATURE-V1-WORKING-CONTEXT".to_string()],
            commands: vec!["memd resume".to_string()],
            routes: vec![],
            files: vec!["crates/memd-client/src/main.rs".to_string()],
            journey_ids: vec!["journey.continuity.resume-handoff-attach".to_string()],
            loop_ids: vec!["loop.resume.correctness".to_string()],
            quality_dimensions: vec!["continuity".to_string(), "correctness".to_string()],
            drift_risks: vec!["continuity-drift".to_string()],
            failure_modes: vec!["resume misses current task state".to_string()],
            coverage_status: "auditing".to_string(),
            last_verified_at: None,
        }],
        journeys: vec![],
        loops: vec![],
        verifiers: vec![VerifierRecord {
            id: "verifier.journey.resume-handoff-attach".to_string(),
            name: "Resume handoff attach continuity".to_string(),
            verifier_type: "journey".to_string(),
            pillar: "memory-continuity".to_string(),
            family: "bundle-runtime".to_string(),
            subject_ids: vec!["journey.continuity.resume-handoff-attach".to_string()],
            fixture_id: "fixture.continuity_bundle".to_string(),
            baseline_modes: vec!["with_memd".to_string()],
            steps: vec![],
            assertions: vec![],
            metrics: vec!["prompt_tokens".to_string()],
            evidence_requirements: vec!["live_primary".to_string()],
            gate_target: "acceptable".to_string(),
            status: "declared".to_string(),
            lanes: vec!["fast".to_string()],
            helper_hooks: vec![],
        }],
        fixtures: vec![FixtureRecord {
            id: "fixture.continuity_bundle".to_string(),
            kind: "bundle_fixture".to_string(),
            description: "continuity bundle".to_string(),
            seed_files: vec![],
            seed_config: serde_json::json!({"project":"memd"}),
            seed_memories: vec![],
            seed_events: vec![],
            seed_sessions: vec![],
            seed_claims: vec![],
            seed_vault: None,
            backend_mode: "normal".to_string(),
            isolation: "fresh_temp_dir".to_string(),
            cleanup_policy: "destroy".to_string(),
        }],
        evidence_policies: vec![],
        schedules: vec![],
        scorecards: vec![],
        evidence: vec![],
        gates: vec![],
        baseline_modes: vec![],
        runtime_policies: vec![],
        generated_at: None,
    };

    let json = serde_json::to_string(&registry).unwrap();
    let decoded: BenchmarkRegistry = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.version, "v1");
    assert_eq!(decoded.features[0].id, "feature.bundle.resume");
    assert!(decoded.features[0].continuity_critical);
    assert_eq!(
        decoded.verifiers[0].id,
        "verifier.journey.resume-handoff-attach"
    );
    assert_eq!(decoded.fixtures[0].id, "fixture.continuity_bundle");
}

#[test]
fn verifier_registry_roundtrips_minimal_resume_verifier() {
    let registry = BenchmarkRegistry {
        version: "v1".to_string(),
        app_goal: "seamless memory and continuity".to_string(),
        quality_dimensions: vec![],
        tiers: vec![],
        pillars: vec![],
        families: vec![],
        features: vec![],
        journeys: vec![],
        loops: vec![],
        verifiers: vec![VerifierRecord {
            id: "verifier.journey.resume-handoff-attach".to_string(),
            name: "Resume handoff attach continuity".to_string(),
            verifier_type: "journey".to_string(),
            pillar: "memory-continuity".to_string(),
            family: "bundle-runtime".to_string(),
            subject_ids: vec!["journey.continuity.resume-handoff-attach".to_string()],
            fixture_id: "fixture.continuity_bundle".to_string(),
            baseline_modes: vec!["with_memd".to_string()],
            steps: vec![],
            assertions: vec![],
            metrics: vec!["prompt_tokens".to_string()],
            evidence_requirements: vec!["live_primary".to_string()],
            gate_target: "acceptable".to_string(),
            status: "declared".to_string(),
            lanes: vec!["fast".to_string()],
            helper_hooks: vec![],
        }],
        fixtures: vec![FixtureRecord {
            id: "fixture.continuity_bundle".to_string(),
            kind: "bundle_fixture".to_string(),
            description: "continuity bundle".to_string(),
            seed_files: vec![],
            seed_config: serde_json::json!({"project":"memd"}),
            seed_memories: vec![],
            seed_events: vec![],
            seed_sessions: vec![],
            seed_claims: vec![],
            seed_vault: None,
            backend_mode: "normal".to_string(),
            isolation: "fresh_temp_dir".to_string(),
            cleanup_policy: "destroy".to_string(),
        }],
        evidence_policies: vec![],
        schedules: vec![],
        scorecards: vec![],
        evidence: vec![],
        gates: vec![],
        baseline_modes: vec![],
        runtime_policies: vec![],
        generated_at: None,
    };

    let json = serde_json::to_string(&registry).unwrap();
    let decoded: BenchmarkRegistry = serde_json::from_str(&json).unwrap();
    assert_eq!(
        decoded.verifiers[0].id,
        "verifier.journey.resume-handoff-attach"
    );
    assert_eq!(decoded.fixtures[0].id, "fixture.continuity_bundle");
}

#[test]
fn benchmark_score_resolution_rules_roundtrip() {
    let rules = ScoreResolutionRules {
        cap_on_continuity_failure: "fragile".to_string(),
        cap_on_missing_required_evidence: "fragile".to_string(),
        cap_on_no_memd_loss: "acceptable".to_string(),
    };

    let json = serde_json::to_string(&rules).unwrap();
    let decoded: ScoreResolutionRules = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.cap_on_continuity_failure, "fragile");
    assert_eq!(decoded.cap_on_no_memd_loss, "acceptable");
}

#[test]
fn continuity_journey_report_roundtrips() {
    let report = ContinuityJourneyReport {
        journey_id: "journey.continuity.resume-handoff-attach".to_string(),
        journey_name: "Resume To Handoff To Attach".to_string(),
        gate_decision: BenchmarkGateDecision {
            gate: "acceptable".to_string(),
            resolved_score: 75,
            reasons: vec!["continuity evidence present".to_string()],
        },
        metrics: BenchmarkSubjectMetrics {
            correctness: 90,
            continuity: 85,
            reliability: 80,
            token_efficiency: 78,
            no_memd_delta: Some(9),
        },
        evidence: BenchmarkEvidenceSummary {
            has_contract_evidence: true,
            has_workflow_evidence: true,
            has_continuity_evidence: true,
            has_comparative_evidence: true,
            has_drift_failure: false,
        },
        baseline_modes: vec![
            "baseline.no-memd".to_string(),
            "baseline.with-memd".to_string(),
        ],
        feature_ids: vec![
            "feature.bundle.resume".to_string(),
            "feature.bundle.handoff".to_string(),
        ],
        artifact_paths: vec![".memd/telemetry/continuity/latest.json".to_string()],
        summary: "resume continuity evidence".to_string(),
        generated_at: None,
    };

    let json = serde_json::to_string(&report).unwrap();
    let decoded: ContinuityJourneyReport = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.gate_decision.gate, "acceptable");
    assert!(decoded.evidence.has_continuity_evidence);
    assert_eq!(decoded.feature_ids.len(), 2);
}

#[test]
fn working_memory_trace_record_accepts_legacy_entries_without_new_fields() {
    let raw = r#"{
          "item_id": "31433b72-abfd-486c-b19b-8a66dd20654d",
          "entity_id": "e70b4828-e1d6-480c-9145-bfd384c27383",
          "event_type": "retrieved_working",
          "summary": "retrieved_working route=local_first intent=current_task",
          "occurred_at": "2026-04-12T14:04:19Z",
          "salience_score": 1.0
        }"#;

    let decoded: WorkingMemoryTraceRecord = serde_json::from_str(raw).unwrap();

    assert_eq!(decoded.memory_kind, MemoryKind::Status);
    assert_eq!(decoded.memory_stage, MemoryStage::Canonical);
    assert_eq!(decoded.typed_memory, "session_continuity+canonical");
}

#[test]
fn coordination_mode_wire_format_is_snake_case() {
    use std::str::FromStr;
    for (variant, wire) in [
        (CoordinationMode::ExclusiveWrite, "exclusive_write"),
        (CoordinationMode::SharedReview, "shared_review"),
        (CoordinationMode::HelpOnly, "help_only"),
        (CoordinationMode::Solo, "solo"),
    ] {
        let encoded = serde_json::to_string(&variant).unwrap();
        assert_eq!(encoded, format!("\"{wire}\""));
        let decoded: CoordinationMode = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, variant);
        assert_eq!(variant.as_str(), wire);
        assert_eq!(variant.to_string(), wire);
        assert_eq!(CoordinationMode::from_str(wire).unwrap(), variant);
    }
    assert_eq!(
        CoordinationMode::default(),
        CoordinationMode::ExclusiveWrite
    );
    assert!(CoordinationMode::from_str("bogus").is_err());
}

#[test]
fn hive_task_record_default_coordination_mode_matches_legacy_payload() {
    let legacy = serde_json::json!({
        "task_id": "legacy-task",
        "title": "legacy",
        "description": null,
        "status": "open",
        "session": null,
        "agent": null,
        "effective_agent": null,
        "project": null,
        "namespace": null,
        "workspace": null,
        "claim_scopes": [],
        "help_requested": false,
        "review_requested": false,
        "created_at": "2025-01-01T00:00:00Z",
        "updated_at": "2025-01-01T00:00:00Z",
    });
    let decoded: HiveTaskRecord = serde_json::from_value(legacy).unwrap();
    assert_eq!(decoded.coordination_mode, CoordinationMode::ExclusiveWrite);
}

#[test]
fn hive_handoff_packet_without_working_context_decodes_as_legacy() {
    let legacy = serde_json::json!({
        "from_session": "bee-1",
        "from_worker": null,
        "to_session": "bee-2",
        "to_worker": null,
        "task_id": null,
        "topic_claim": null,
        "scope_claims": [],
        "next_action": null,
        "blocker": null,
        "note": null,
        "created_at": "2025-01-01T00:00:00Z",
    });
    let decoded: HiveHandoffPacket = serde_json::from_value(legacy).unwrap();
    assert!(decoded.working_context.is_none());
    let re_encoded = serde_json::to_value(&decoded).unwrap();
    assert!(
        re_encoded.get("working_context").is_none(),
        "absent working_context must serialize-skip"
    );
}

#[test]
fn working_context_snapshot_truncates_to_cap() {
    let bulky: Vec<CompactMemoryRecord> = (0..16)
        .map(|n| CompactMemoryRecord {
            id: Uuid::new_v4(),
            record: format!("row-{n}"),
        })
        .collect();
    let snap = WorkingContextSnapshot {
        working_records: bulky,
        doing: Some("building L2.4".to_string()),
        version: 7,
        ..Default::default()
    }
    .truncate_to_cap();
    assert_eq!(
        snap.working_records.len(),
        WorkingContextSnapshot::MAX_WORKING_RECORDS
    );

    let packet = HiveHandoffPacket {
        from_session: "bee-a".to_string(),
        from_worker: None,
        to_session: "bee-b".to_string(),
        to_worker: None,
        task_id: Some("task-1".to_string()),
        topic_claim: None,
        scope_claims: Vec::new(),
        next_action: None,
        blocker: None,
        note: None,
        created_at: DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc),
        working_context: Some(snap),
    };
    let round: HiveHandoffPacket =
        serde_json::from_str(&serde_json::to_string(&packet).unwrap()).unwrap();
    let wc = round.working_context.expect("working_context survives");
    assert_eq!(
        wc.working_records.len(),
        WorkingContextSnapshot::MAX_WORKING_RECORDS
    );
    assert_eq!(wc.version, 7);
    assert_eq!(wc.doing.as_deref(), Some("building L2.4"));
}

fn base_memory_item(kind: MemoryKind) -> MemoryItem {
    let now = Utc::now();
    MemoryItem {
        id: Uuid::new_v4(),
        content: "sample".to_string(),
        redundancy_key: None,
        belief_branch: None,
        preferred: false,
        kind,
        scope: MemoryScope::Project,
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        workspace: None,
        visibility: MemoryVisibility::Private,
        source_agent: None,
        source_system: None,
        source_path: None,
        source_quality: None,
        confidence: 0.8,
        ttl_seconds: None,
        created_at: now,
        updated_at: now,
        last_verified_at: None,
        supersedes: vec![],
        tags: vec![],
        status: MemoryStatus::Active,
        stage: MemoryStage::Canonical,
        lane: None,
        version: 1,
        correction_meta: None,
    }
}

#[test]
fn memory_kind_correction_round_trips_json() {
    let json = serde_json::to_string(&MemoryKind::Correction).unwrap();
    assert_eq!(json, "\"correction\"");
    let decoded: MemoryKind = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, MemoryKind::Correction);
}

#[test]
fn memory_item_with_correction_meta_serializes_without_extra_nulls_when_absent() {
    let item = base_memory_item(MemoryKind::Fact);
    let json = serde_json::to_string(&item).unwrap();
    assert!(
        !json.contains("correction_meta"),
        "absent correction_meta must be skipped; got: {json}"
    );

    let meta = CorrectionMetadata {
        corrects_id: Some(Uuid::new_v4()),
        source_turn: Some("t-12".to_string()),
        captured_by: Some(CaptureSource::Detector),
        confidence: Some(0.88),
    };
    let mut with_meta = base_memory_item(MemoryKind::Correction);
    with_meta.correction_meta = Some(meta.clone());
    let json = serde_json::to_string(&with_meta).unwrap();
    assert!(json.contains("correction_meta"));
    assert!(json.contains("detector"));

    let decoded: MemoryItem = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.correction_meta, Some(meta));
}

#[test]
fn legacy_memory_item_deserializes_with_none_correction_meta() {
    let legacy_json = serde_json::to_string(&base_memory_item(MemoryKind::Fact)).unwrap();
    assert!(!legacy_json.contains("correction_meta"));
    let decoded: MemoryItem = serde_json::from_str(&legacy_json).unwrap();
    assert!(decoded.correction_meta.is_none());
}

#[test]
fn memory_kind_skill_serializes_snake_case() {
    let json = serde_json::to_string(&MemoryKind::Skill).unwrap();
    assert_eq!(json, "\"skill\"");
}

#[test]
fn memory_kind_skill_round_trips() {
    let parsed: MemoryKind = serde_json::from_str("\"skill\"").unwrap();
    assert_eq!(parsed, MemoryKind::Skill);
}

#[test]
fn retrieval_intent_skill_serializes() {
    let json = serde_json::to_string(&RetrievalIntent::Skill).unwrap();
    assert_eq!(json, "\"skill\"");
}
