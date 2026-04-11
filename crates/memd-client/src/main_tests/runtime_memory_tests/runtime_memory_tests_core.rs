use super::*;

    let base_url = spawn_mock_runtime_server(state.clone(), false).await;
    let client = MemdClient::new(&base_url).expect("client");
    let req = SearchMemoryRequest {
        query: Some("what default response style should Codex use".to_string()),
        route: Some(memd_schema::RetrievalRoute::All),
        intent: Some(memd_schema::RetrievalIntent::General),
        scopes: vec![
            memd_schema::MemoryScope::Project,
            memd_schema::MemoryScope::Synced,
            memd_schema::MemoryScope::Global,
        ],
        kinds: vec![memd_schema::MemoryKind::Preference],
        statuses: vec![memd_schema::MemoryStatus::Active],
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        workspace: None,
        visibility: None,
        belief_branch: None,
        source_agent: None,
        tags: vec!["caveman-ultra".to_string()],
        stages: vec![memd_schema::MemoryStage::Canonical],
        limit: Some(6),
        max_chars_per_item: Some(280),
    };

    let response = lookup_with_fallbacks(
        &client,
        &req,
        "what default response style should Codex use",
    )
    .await
    .expect("lookup fallback");
    assert_eq!(response.items.len(), 1);
    assert_eq!(response.items[0].kind, memd_schema::MemoryKind::Preference);
    assert!(response.items[0].content.contains("caveman ultra"));
    assert!(
        *state.search_count.lock().expect("lock search count") >= 2,
        "expected fallback search attempts"
    );
}

#[test]
fn lookup_markdown_mentions_pre_answer_protocol() {
    let response = memd_schema::SearchMemoryResponse {
        route: memd_schema::RetrievalRoute::Auto,
        intent: memd_schema::RetrievalIntent::CurrentTask,
        items: vec![memd_schema::MemoryItem {
            id: uuid::Uuid::new_v4(),
            content: "decision: wake is the startup surface".to_string(),
            redundancy_key: None,
            belief_branch: None,
            preferred: false,
            kind: memd_schema::MemoryKind::Decision,
            scope: memd_schema::MemoryScope::Project,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            visibility: memd_schema::MemoryVisibility::Workspace,
            source_agent: Some("codex".to_string()),
            source_system: Some("memd".to_string()),
            source_path: None,
            source_quality: Some(memd_schema::SourceQuality::Canonical),
            confidence: 0.95,
            ttl_seconds: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_verified_at: None,
            supersedes: Vec::new(),
            tags: vec!["10-star".to_string()],
            status: memd_schema::MemoryStatus::Active,
            stage: memd_schema::MemoryStage::Canonical,
        }],
    };

    let markdown = render_lookup_markdown("startup surface", &response, false);
    assert!(markdown.contains("# memd lookup"));
    assert!(markdown.contains("decision: wake is the startup surface"));
    assert!(markdown.contains("Use recalled items before answering"));
}

#[tokio::test]
async fn hook_capture_can_supersede_stale_memory_after_promotion() {
    let state = MockRuntimeState::default();
    let base_url = spawn_mock_runtime_server(state.clone(), false).await;
    let dir = std::env::temp_dir().join(format!("memd-hook-supersede-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("create temp bundle");

    let args = HookCaptureArgs {
        output: dir.clone(),
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        workspace: Some("shared".to_string()),
        visibility: Some("workspace".to_string()),
        source_path: Some("hook-capture".to_string()),
        confidence: Some(0.8),
        ttl_seconds: Some(3600),
        content: None,
        input: None,
        stdin: true,
        tag: vec!["episodic".to_string()],
        promote_kind: Some("fact".to_string()),
        promote_scope: Some("project".to_string()),
        promote_supersede: vec!["123e4567-e89b-12d3-a456-426614174000".to_string()],
        promote_supersede_query: None,
        promote_tag: vec!["correction".to_string()],
        promote_confidence: Some(0.99),
        summary: true,
    };

    let (supersede_targets, diagnostics) = find_hook_capture_supersede_targets(
        &base_url,
        &args,
        "corrected fact: wake is the startup surface",
    )
    .await
    .expect("find supersede targets");
    assert_eq!(supersede_targets.len(), 1);

    let promoted = remember_with_bundle_defaults(
        &remember_args_from_effective_hook_capture(
            &args,
            "corrected fact: wake is the startup surface".to_string(),
            "fact".to_string(),
            supersede_targets.clone(),
        ),
        &base_url,
    )
    .await
    .expect("promote durable memory");
    let repaired =
        mark_hook_capture_supersede_targets(&base_url, &args, &supersede_targets, promoted.item.id)
            .await
            .expect("supersede stale memory");

    assert_eq!(repaired.len(), 1);
    let diagnostics_json = serde_json::to_value(&diagnostics).unwrap_or(JsonValue::Null);
    assert!(
        diagnostics_json
            .get("candidate_hits")
            .and_then(JsonValue::as_array)
            .is_some_and(|values| values.is_empty())
    );
    assert_eq!(promoted.item.supersedes, supersede_targets);
    let captured = state.repaired.lock().expect("lock repaired");
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].mode, memd_schema::MemoryRepairMode::Supersede);
    assert_eq!(
        captured[0].status,
        Some(memd_schema::MemoryStatus::Superseded)
    );
    assert!(captured[0].supersedes.is_empty());

    fs::remove_dir_all(dir).expect("cleanup temp bundle");
}

#[tokio::test]
async fn hook_capture_can_find_supersede_targets_by_query() {
    let state = MockRuntimeState::default();
    let base_url = spawn_mock_runtime_server(state.clone(), false).await;
    let args = HookCaptureArgs {
        output: PathBuf::from(".memd"),
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        workspace: Some("shared".to_string()),
        visibility: Some("workspace".to_string()),
        source_path: Some("hook-capture".to_string()),
        confidence: Some(0.8),
        ttl_seconds: None,
        content: None,
        input: None,
        stdin: true,
        tag: vec!["episodic".to_string()],
        promote_kind: Some("fact".to_string()),
        promote_scope: Some("project".to_string()),
        promote_supersede: Vec::new(),
        promote_supersede_query: Some("stale belief".to_string()),
        promote_tag: vec!["correction".to_string()],
        promote_confidence: Some(0.99),
        summary: true,
    };

    let (supersede_targets, diagnostics) =
        find_hook_capture_supersede_targets(&base_url, &args, "corrected fact: stale belief")
            .await
            .expect("supersede by query");
    let diagnostics_json = serde_json::to_value(&diagnostics).unwrap_or(JsonValue::Null);
    assert_eq!(supersede_targets.len(), 1);
    assert!(
        diagnostics_json
            .get("tried_queries")
            .and_then(JsonValue::as_array)
            .is_some_and(|values| !values.is_empty())
    );
    let candidate_hits = diagnostics_json
        .get("candidate_hits")
        .and_then(JsonValue::as_array)
        .expect("candidate hits json");
    assert!(!candidate_hits.is_empty());
    assert_eq!(
        candidate_hits[0].get("query").and_then(JsonValue::as_str),
        Some("stale belief")
    );
    assert_eq!(
        candidate_hits[0]
            .get("ids")
            .and_then(JsonValue::as_array)
            .map(|values| values.len()),
        Some(1)
    );
    assert_eq!(*state.search_count.lock().expect("lock search count"), 1);
    let promoted = remember_with_bundle_defaults(
        &remember_args_from_effective_hook_capture(
            &args,
            "corrected fact: stale belief".to_string(),
            "fact".to_string(),
            supersede_targets.clone(),
        ),
        &base_url,
    )
    .await
    .expect("promote corrected memory");
    assert_eq!(promoted.item.supersedes, supersede_targets);
    let repaired =
        mark_hook_capture_supersede_targets(&base_url, &args, &supersede_targets, promoted.item.id)
            .await
            .expect("mark stale target superseded");
    assert_eq!(repaired.len(), 1);
    let captured = state.repaired.lock().expect("lock repaired");
    assert_eq!(captured.len(), 1);
    assert!(captured[0].supersedes.is_empty());
}

#[tokio::test]
async fn hook_capture_does_not_auto_supersede_active_query_matches() {
    let state = MockRuntimeState::default();
    let base_url = spawn_mock_runtime_server(state.clone(), false).await;
    let args = HookCaptureArgs {
        output: PathBuf::from(".memd"),
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        workspace: Some("shared".to_string()),
        visibility: Some("workspace".to_string()),
        source_path: Some("hook-capture".to_string()),
        confidence: Some(0.8),
        ttl_seconds: None,
        content: None,
        input: None,
        stdin: true,
        tag: vec!["episodic".to_string()],
        promote_kind: Some("fact".to_string()),
        promote_scope: Some("project".to_string()),
        promote_supersede: Vec::new(),
        promote_supersede_query: Some("hive resume state".to_string()),
        promote_tag: vec!["correction".to_string()],
        promote_confidence: Some(0.99),
        summary: true,
    };

    let (supersede_targets, diagnostics) =
        find_hook_capture_supersede_targets(&base_url, &args, "corrected fact: hive resume state")
            .await
            .expect("query repair");
    let diagnostics_json = serde_json::to_value(&diagnostics).unwrap_or(JsonValue::Null);
    assert!(supersede_targets.is_empty());
    assert!(
        diagnostics_json
            .get("candidate_hits")
            .and_then(JsonValue::as_array)
            .is_some_and(|values| !values.is_empty())
    );
    let captured = state.repaired.lock().expect("lock repaired");
    assert!(captured.is_empty());
}

#[test]
fn resume_prompt_surfaces_current_task_snapshot() {
    let snapshot = ResumeSnapshot {
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        agent: Some("codex".to_string()),
        workspace: Some("team-alpha".to_string()),
        visibility: Some("workspace".to_string()),
        route: "auto".to_string(),
        intent: "current_task".to_string(),
        context: memd_schema::CompactContextResponse {
            route: memd_schema::RetrievalRoute::Auto,
            intent: memd_schema::RetrievalIntent::CurrentTask,
            retrieval_order: vec![memd_schema::MemoryScope::Project],
            records: Vec::new(),
        },
        working: memd_schema::WorkingMemoryResponse {
            route: memd_schema::RetrievalRoute::Auto,
            intent: memd_schema::RetrievalIntent::CurrentTask,
            retrieval_order: vec![memd_schema::MemoryScope::Project],
            budget_chars: 1600,
            used_chars: 60,
            remaining_chars: 1540,
            truncated: false,
            policy: memd_schema::WorkingMemoryPolicyState {
                admission_limit: 8,
                max_chars_per_item: 220,
                budget_chars: 1600,
                rehydration_limit: 4,
            },
            records: vec![memd_schema::CompactMemoryRecord {
                id: uuid::Uuid::new_v4(),
                record: "Follow the active current-task lane".to_string(),
            }],
            evicted: Vec::new(),
            rehydration_queue: vec![memd_schema::MemoryRehydrationRecord {
                id: None,
                kind: "source".to_string(),
                label: "handoff".to_string(),
                summary: "Reload the shared workspace handoff".to_string(),
                reason: None,
                source_agent: None,
                source_system: None,
                source_path: None,
                source_quality: None,
                recorded_at: None,
            }],
            traces: Vec::new(),
            semantic_consolidation: None,
        },
        inbox: memd_schema::MemoryInboxResponse {
            route: memd_schema::RetrievalRoute::Auto,
            intent: memd_schema::RetrievalIntent::CurrentTask,
            items: vec![memd_schema::InboxMemoryItem {
                item: memd_schema::MemoryItem {
                    id: uuid::Uuid::new_v4(),
                    content: "One review item is still open".to_string(),
                    redundancy_key: None,
                    belief_branch: None,
                    preferred: true,
                    kind: memd_schema::MemoryKind::Status,
                    scope: memd_schema::MemoryScope::Project,
                    project: Some("demo".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("team-alpha".to_string()),
                    visibility: memd_schema::MemoryVisibility::Workspace,
                    source_agent: None,
                    source_system: None,
                    source_path: None,
                    source_quality: None,
                    confidence: 0.8,
                    ttl_seconds: Some(86_400),
                    created_at: chrono::Utc::now(),
                    status: memd_schema::MemoryStatus::Active,
                    stage: memd_schema::MemoryStage::Candidate,
                    last_verified_at: None,
                    supersedes: Vec::new(),
                    updated_at: chrono::Utc::now(),
                    tags: vec!["checkpoint".to_string()],
                },
                reasons: vec!["stale".to_string()],
            }],
        },
        workspaces: memd_schema::WorkspaceMemoryResponse {
            workspaces: vec![memd_schema::WorkspaceMemoryRecord {
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("team-alpha".to_string()),
                visibility: memd_schema::MemoryVisibility::Workspace,
                item_count: 4,
                active_count: 3,
                candidate_count: 1,
                contested_count: 0,
                source_lane_count: 1,
                avg_confidence: 0.84,
                trust_score: 0.91,
                last_seen_at: None,
                tags: Vec::new(),
            }],
        },
        sources: memd_schema::SourceMemoryResponse {
            sources: Vec::new(),
        },
        semantic: None,
        claims: SessionClaimsState::default(),
        recent_repo_changes: vec!["status M crates/memd-client/src/render.rs".to_string()],
        change_summary: vec!["focus -> Follow the active current-task lane".to_string()],
        resume_state_age_minutes: None,
        refresh_recommended: false,
    };

    let prompt = crate::render::render_resume_prompt(&snapshot);
    assert!(prompt.contains("## Context Budget"));
    assert!(prompt.contains("tok="));
    assert!(prompt.contains("dup=0"));
    assert!(prompt.contains("p=low"));
    assert!(prompt.contains("ref=false"));
    assert!(prompt.contains("## T"));
    assert!(prompt.contains("- t="));
    assert!(prompt.contains("## E+LT"));
    assert!(prompt.contains("Follow the active current-task lane"));
    assert!(prompt.contains("One review item is still open"));
    assert!(prompt.contains("Reload the shared workspace handoff"));
    assert!(prompt.contains("status M crates/memd-client/src/render.rs"));
    assert!(prompt.contains("team-alpha"));
}

#[test]
fn resume_snapshot_detects_redundant_context_items() {
    let base = ResumeSnapshot {
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        agent: Some("codex".to_string()),
        workspace: Some("team-alpha".to_string()),
        visibility: Some("workspace".to_string()),
        route: "auto".to_string(),
        intent: "current_task".to_string(),
        context: memd_schema::CompactContextResponse {
            route: memd_schema::RetrievalRoute::Auto,
            intent: memd_schema::RetrievalIntent::CurrentTask,
            retrieval_order: vec![memd_schema::MemoryScope::Project],
            records: vec![memd_schema::CompactMemoryRecord {
                id: uuid::Uuid::new_v4(),
                record: "Repeat this exact idea".to_string(),
            }],
        },
        working: memd_schema::WorkingMemoryResponse {
            route: memd_schema::RetrievalRoute::Auto,
            intent: memd_schema::RetrievalIntent::CurrentTask,
            retrieval_order: vec![memd_schema::MemoryScope::Project],
            budget_chars: 1600,
            used_chars: 120,
            remaining_chars: 1480,
            truncated: false,
            policy: memd_schema::WorkingMemoryPolicyState {
                admission_limit: 8,
                max_chars_per_item: 220,
                budget_chars: 1600,
                rehydration_limit: 4,
            },
            records: vec![memd_schema::CompactMemoryRecord {
                id: uuid::Uuid::new_v4(),
                record: "Repeat this exact idea".to_string(),
            }],
            evicted: Vec::new(),
            rehydration_queue: vec![memd_schema::MemoryRehydrationRecord {
