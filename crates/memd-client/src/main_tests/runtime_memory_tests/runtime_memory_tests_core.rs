use super::*;
use memd_schema::MemoryVisibility;

#[tokio::test]
async fn lookup_with_fallbacks_retries_until_match() {
    let state = MockRuntimeState::default();
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
        stages: vec![
            memd_schema::MemoryStage::Canonical,
            memd_schema::MemoryStage::Candidate,
        ],
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

#[tokio::test]
async fn source_memory_request_uses_repo_bundle_identity_defaults() {
    let state = MockRuntimeState::default();
    let base_url = spawn_mock_runtime_server(state.clone(), false).await;
    let client = MemdClient::new(&base_url).expect("client");
    let temp_root =
        std::env::temp_dir().join(format!("memd-source-defaults-{}", uuid::Uuid::new_v4()));
    let repo_root = temp_root.join("repo-b");
    let bundle_root = repo_root.join(".memd");

    fs::create_dir_all(repo_root.join(".git")).expect("create repo git dir");
    fs::create_dir_all(&bundle_root).expect("create bundle dir");
    let _cwd = crate::test_support::set_current_dir(&repo_root);

    let (project, namespace) = infer_bundle_identity_defaults(&bundle_root);
    let response = client
        .source_memory(&SourceMemoryRequest {
            project,
            namespace,
            workspace: Some("shared".to_string()),
            visibility: Some(MemoryVisibility::Workspace),
            source_agent: Some("codex".to_string()),
            source_system: Some("hook-capture".to_string()),
            limit: Some(5),
        })
        .await
        .expect("source memory request");

    assert!(response.sources.is_empty());
    let requests = state.source_requests.lock().expect("lock source requests");
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].project.as_deref(), Some("repo-b"));
    assert_eq!(requests[0].namespace.as_deref(), Some("main"));
    assert_eq!(requests[0].workspace.as_deref(), Some("shared"));
    assert_eq!(requests[0].visibility, Some(MemoryVisibility::Workspace));
    assert_eq!(requests[0].source_agent.as_deref(), Some("codex"));
    assert_eq!(requests[0].source_system.as_deref(), Some("hook-capture"));
    assert_eq!(requests[0].limit, Some(5));

    drop(requests);
    drop(_cwd);
    fs::remove_dir_all(temp_root).expect("cleanup temp root");
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

    let request = SearchMemoryRequest {
        query: Some("startup surface".to_string()),
        ..Default::default()
    };
    let markdown = render_lookup_markdown("startup surface", &request, &response, false);
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
                label: "dup".to_string(),
                summary: "Repeat this exact idea".to_string(),
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
                    content: "Repeat this exact idea".to_string(),
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
                    confidence: 0.9,
                    ttl_seconds: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    last_verified_at: None,
                    supersedes: Vec::new(),
                    tags: Vec::new(),
                    status: memd_schema::MemoryStatus::Active,
                    stage: memd_schema::MemoryStage::Candidate,
                },
                reasons: vec!["same".to_string()],
            }],
        },
        workspaces: memd_schema::WorkspaceMemoryResponse {
            workspaces: Vec::new(),
        },
        sources: memd_schema::SourceMemoryResponse {
            sources: Vec::new(),
        },
        semantic: None,
        claims: SessionClaimsState::default(),
        recent_repo_changes: vec!["repo clean".to_string()],
        change_summary: Vec::new(),
        resume_state_age_minutes: None,
        refresh_recommended: false,
    };

    let base = snapshot;
    assert!(base.redundant_context_items() >= 1);
    assert_eq!(base.context_pressure(), "medium");
    assert!(
        base.optimization_hints()
            .iter()
            .any(|hint: &String| hint.contains("collapse 1 repeated context item"))
    );
}

#[test]
fn working_summary_surfaces_typed_trace_trail() {
    let response = memd_schema::WorkingMemoryResponse {
        route: memd_schema::RetrievalRoute::ProjectFirst,
        intent: memd_schema::RetrievalIntent::CurrentTask,
        retrieval_order: vec![
            memd_schema::MemoryScope::Project,
            memd_schema::MemoryScope::Synced,
        ],
        budget_chars: 1600,
        used_chars: 240,
        remaining_chars: 1360,
        truncated: false,
        policy: memd_schema::WorkingMemoryPolicyState {
            admission_limit: 8,
            max_chars_per_item: 220,
            budget_chars: 1600,
            rehydration_limit: 4,
        },
        records: vec![memd_schema::CompactMemoryRecord {
            id: uuid::Uuid::new_v4(),
            record: "current task: lock typed trace families".to_string(),
        }],
        evicted: Vec::new(),
        rehydration_queue: Vec::new(),
        traces: vec![memd_schema::WorkingMemoryTraceRecord {
            item_id: uuid::Uuid::new_v4(),
            entity_id: Some(uuid::Uuid::new_v4()),
            memory_kind: memd_schema::MemoryKind::Status,
            memory_stage: memd_schema::MemoryStage::Candidate,
            typed_memory: "session_continuity+candidate".to_string(),
            event_type: "retrieved_context".to_string(),
            summary: "continuity state entered working set".to_string(),
            occurred_at: chrono::Utc::now(),
            salience_score: 0.82,
        }],
        semantic_consolidation: None,
    };

    let summary = render_working_summary(&response, true);
    assert!(summary.contains(
        "trace_trail=session_continuity+candidate:retrieved_context:continuity state entered working set"
    ));
}
