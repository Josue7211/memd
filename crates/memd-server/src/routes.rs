use super::*;

pub(crate) async fn healthz(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        items: state.store.count().unwrap_or(0),
    })
}

pub(crate) async fn dashboard(
    State(state): State<AppState>,
) -> Result<Html<String>, (StatusCode, String)> {
    let snapshot = ui::build_visible_memory_snapshot(&state).map_err(internal_error)?;
    Ok(Html(ui::dashboard_html(&snapshot, ui::UiPage::Home)))
}

pub(crate) async fn get_visible_memory_snapshot(
    State(state): State<AppState>,
) -> Result<Json<VisibleMemorySnapshotResponse>, (StatusCode, String)> {
    let response = ui::build_visible_memory_snapshot(&state).map_err(internal_error)?;
    Ok(Json(response))
}

#[derive(Deserialize)]
pub(crate) struct VisibleMemoryArtifactQuery {
    pub(crate) id: Uuid,
}

pub(crate) async fn get_visible_memory_artifact(
    State(state): State<AppState>,
    Query(req): Query<VisibleMemoryArtifactQuery>,
) -> Result<Json<VisibleMemoryArtifactDetailResponse>, (StatusCode, String)> {
    let response = ui::build_visible_memory_artifact_detail(&state, req.id)?;
    Ok(Json(response))
}

pub(crate) async fn post_visible_memory_action(
    State(state): State<AppState>,
    Json(req): Json<VisibleMemoryUiActionRequest>,
) -> Result<Json<VisibleMemoryUiActionResponse>, (StatusCode, String)> {
    let response = ui::perform_visible_memory_action(&state, req)?;
    Ok(Json(response))
}

pub(crate) async fn store_memory(
    State(state): State<AppState>,
    Json(req): Json<StoreMemoryRequest>,
) -> Result<Json<StoreMemoryResponse>, (StatusCode, String)> {
    if req.content.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "content must not be empty".to_string(),
        ));
    }

    let item = state
        .store_item(req, MemoryStage::Canonical)
        .map_err(internal_error)?;
    let (item, duplicate) = item;
    Ok(Json(StoreMemoryResponse {
        item: duplicate.map_or(item, |found| found.item),
    }))
}

pub(crate) async fn store_candidate(
    State(state): State<AppState>,
    Json(req): Json<CandidateMemoryRequest>,
) -> Result<Json<CandidateMemoryResponse>, (StatusCode, String)> {
    if req.content.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "content must not be empty".to_string(),
        ));
    }

    let store_req = StoreMemoryRequest {
        content: req.content,
        kind: req.kind,
        scope: req.scope,
        project: req.project,
        namespace: req.namespace,
        workspace: req.workspace,
        visibility: req.visibility,
        belief_branch: req.belief_branch,
        source_agent: req.source_agent,
        source_system: req.source_system,
        source_path: req.source_path,
        source_quality: req.source_quality,
        confidence: req.confidence,
        ttl_seconds: req.ttl_seconds,
        last_verified_at: req.last_verified_at,
        supersedes: req.supersedes,
        tags: req.tags,
        status: Some(MemoryStatus::Active),
    };

    let (item, duplicate) = state
        .store_item(store_req, MemoryStage::Candidate)
        .map_err(internal_error)?;
    Ok(Json(CandidateMemoryResponse {
        item: duplicate.as_ref().map_or(item, |found| found.item.clone()),
        duplicate_of: duplicate.map(|found| found.id),
    }))
}

pub(crate) async fn promote_memory(
    State(state): State<AppState>,
    Json(req): Json<PromoteMemoryRequest>,
) -> Result<Json<PromoteMemoryResponse>, (StatusCode, String)> {
    let (item, duplicate) = state.promote_item(req).map_err(internal_error)?;
    Ok(Json(PromoteMemoryResponse {
        item: duplicate.as_ref().map_or(item, |found| found.item.clone()),
        duplicate_of: duplicate.map(|found| found.id),
    }))
}

pub(crate) async fn expire_memory(
    State(state): State<AppState>,
    Json(req): Json<ExpireMemoryRequest>,
) -> Result<Json<ExpireMemoryResponse>, (StatusCode, String)> {
    let item = repair::expire_item(&state, req)?;
    Ok(Json(ExpireMemoryResponse { item }))
}

pub(crate) async fn verify_memory(
    State(state): State<AppState>,
    Json(req): Json<VerifyMemoryRequest>,
) -> Result<Json<VerifyMemoryResponse>, (StatusCode, String)> {
    let item = repair::verify_item(&state, req)?;
    Ok(Json(VerifyMemoryResponse { item }))
}

pub(crate) async fn repair_memory(
    State(state): State<AppState>,
    Json(req): Json<RepairMemoryRequest>,
) -> Result<Json<RepairMemoryResponse>, (StatusCode, String)> {
    let response = repair::repair_item(&state, req)?;
    Ok(Json(response))
}

pub(crate) async fn get_working_memory(
    State(state): State<AppState>,
    Query(req): Query<WorkingMemoryRequest>,
) -> Result<Json<WorkingMemoryResponse>, (StatusCode, String)> {
    let response = working::working_memory(&state, req)?;
    Ok(Json(response))
}

pub(crate) async fn get_explain(
    State(state): State<AppState>,
    Query(req): Query<ExplainMemoryRequest>,
) -> Result<Json<ExplainMemoryResponse>, (StatusCode, String)> {
    let plan = RetrievalPlan::resolve(req.route, req.intent);
    let response = inspection::explain_memory(&state, req)?;
    state
        .record_retrieval_feedback(
            std::slice::from_ref(&response.item),
            1,
            "retrieved_explain",
            &plan,
        )
        .map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn get_memory_policy() -> Json<MemoryPolicyResponse> {
    Json(working::memory_policy_snapshot())
}

pub(crate) async fn search_memory(
    State(state): State<AppState>,
    Json(req): Json<SearchMemoryRequest>,
) -> Result<Json<SearchMemoryResponse>, (StatusCode, String)> {
    let policy = working::memory_policy_snapshot();
    let feedback_limit = policy.retrieval_feedback.max_items_per_request;
    let items = enrich_with_entities(&state, state.snapshot().map_err(internal_error)?)
        .map_err(internal_error)?;
    let plan = RetrievalPlan::resolve(req.route, req.intent);
    let items = filter_items(&items, &req, &plan);
    state
        .rehearse_items(&items, feedback_limit)
        .map_err(internal_error)?;
    state
        .record_retrieval_feedback(&items, feedback_limit, "retrieved_search", &plan)
        .map_err(internal_error)?;
    Ok(Json(SearchMemoryResponse {
        route: plan.route,
        intent: plan.intent,
        items,
    }))
}

pub(crate) async fn get_context(
    State(state): State<AppState>,
    Query(req): Query<ContextRequest>,
) -> Result<Json<ContextResponse>, (StatusCode, String)> {
    let policy = working::memory_policy_snapshot();
    let feedback_limit = policy.retrieval_feedback.max_items_per_request;
    let req = apply_agent_profile_defaults(&state, req).map_err(internal_error)?;
    let BuildContextResult {
        plan,
        retrieval_order,
        items,
    } = build_context(&state, &req)?;
    state
        .rehearse_items(&items, feedback_limit)
        .map_err(internal_error)?;
    state
        .record_retrieval_feedback(&items, feedback_limit, "retrieved_context", &plan)
        .map_err(internal_error)?;
    Ok(Json(ContextResponse {
        route: plan.route,
        intent: plan.intent,
        retrieval_order,
        items,
    }))
}

pub(crate) async fn get_compact_context(
    State(state): State<AppState>,
    Query(req): Query<ContextRequest>,
) -> Result<Json<CompactContextResponse>, (StatusCode, String)> {
    let policy = working::memory_policy_snapshot();
    let feedback_limit = policy.retrieval_feedback.max_items_per_request;
    let BuildContextResult {
        plan,
        retrieval_order,
        items,
    } = build_context(&state, &req)?;
    state
        .rehearse_items(&items, feedback_limit)
        .map_err(internal_error)?;
    state
        .record_retrieval_feedback(&items, feedback_limit, "retrieved_compact_context", &plan)
        .map_err(internal_error)?;
    let records = items
        .into_iter()
        .map(|item| CompactMemoryRecord {
            id: item.id,
            record: compact_record(&item),
        })
        .collect();

    Ok(Json(CompactContextResponse {
        route: plan.route,
        intent: plan.intent,
        retrieval_order,
        records,
    }))
}

pub(crate) async fn get_inbox(
    State(state): State<AppState>,
    Query(req): Query<MemoryInboxRequest>,
) -> Result<Json<MemoryInboxResponse>, (StatusCode, String)> {
    let plan = RetrievalPlan::resolve(req.route, req.intent);
    let limit = req.limit.unwrap_or(12).min(50);
    let items = enrich_with_entities(&state, state.snapshot().map_err(internal_error)?)
        .map_err(internal_error)?;
    let mut inbox = items
        .into_iter()
        .filter(|entry| {
            entry.item.stage == MemoryStage::Candidate || entry.item.status != MemoryStatus::Active
        })
        .filter(|entry| {
            req.project
                .as_ref()
                .is_none_or(|project| entry.item.project.as_ref() == Some(project))
        })
        .filter(|entry| {
            req.namespace
                .as_ref()
                .is_none_or(|namespace| entry.item.namespace.as_ref() == Some(namespace))
        })
        .filter(|entry| {
            req.workspace
                .as_ref()
                .is_none_or(|workspace| entry.item.workspace.as_ref() == Some(workspace))
        })
        .filter(|entry| {
            req.visibility
                .is_none_or(|visibility| entry.item.visibility == visibility)
        })
        .filter(|entry| {
            req.belief_branch
                .as_ref()
                .is_none_or(|branch| entry.item.belief_branch.as_ref() == Some(branch))
        })
        .collect::<Vec<_>>();

    inbox.sort_by(|a, b| {
        inbox_score(
            &b.item,
            b.entity.as_ref(),
            req.project.as_ref(),
            req.namespace.as_ref(),
            &plan,
        )
        .partial_cmp(&inbox_score(
            &a.item,
            a.entity.as_ref(),
            req.project.as_ref(),
            req.namespace.as_ref(),
            &plan,
        ))
        .unwrap_or(std::cmp::Ordering::Equal)
        .then_with(|| b.item.updated_at.cmp(&a.item.updated_at))
    });
    inbox.truncate(limit);
    let inbox = inbox
        .into_iter()
        .map(|entry| InboxMemoryItem {
            reasons: inbox_reasons(&entry.item),
            item: entry.item,
        })
        .filter(|entry| !entry.reasons.is_empty())
        .collect();

    Ok(Json(MemoryInboxResponse {
        route: plan.route,
        intent: plan.intent,
        items: inbox,
    }))
}

pub(crate) async fn get_entity(
    State(state): State<AppState>,
    Query(req): Query<EntityMemoryRequest>,
) -> Result<Json<EntityMemoryResponse>, (StatusCode, String)> {
    let plan = RetrievalPlan::resolve(req.route, req.intent);
    let limit = req.limit.unwrap_or(4).min(12);
    state.rehearse_item(req.id, 0.08).map_err(internal_error)?;
    let (entity, events) = state.entity_view(req.id, limit).map_err(internal_error)?;

    Ok(Json(EntityMemoryResponse {
        route: plan.route,
        intent: plan.intent,
        entity,
        events,
    }))
}

pub(crate) async fn get_entity_search(
    State(state): State<AppState>,
    Query(req): Query<EntitySearchRequest>,
) -> Result<Json<EntitySearchResponse>, (StatusCode, String)> {
    let plan = RetrievalPlan::resolve(req.route, req.intent);
    let query = req.query.trim().to_string();
    if query.is_empty() {
        return Ok(Json(EntitySearchResponse {
            route: plan.route,
            intent: plan.intent,
            query,
            best_match: None,
            candidates: Vec::new(),
            ambiguous: false,
        }));
    }

    let mut candidates = if let Ok(id) = Uuid::parse_str(&query) {
        match state.store.entity_by_id(id).map_err(internal_error)? {
            Some(entity) => vec![EntitySearchHit {
                entity,
                score: 1.0,
                reasons: vec!["exact entity id".to_string()],
            }],
            None => Vec::new(),
        }
    } else {
        state
            .store
            .search_entities(&EntitySearchRequest {
                query: query.clone(),
                project: req.project.clone(),
                namespace: req.namespace.clone(),
                at: req.at,
                host: req.host.clone(),
                branch: req.branch.clone(),
                location: req.location.clone(),
                route: req.route,
                intent: req.intent,
                limit: req.limit,
            })
            .map_err(internal_error)?
    };

    let best_match = candidates.first().cloned();
    let ambiguous = candidates.len() > 1
        && candidates
            .get(1)
            .map(|candidate| {
                best_match
                    .as_ref()
                    .is_some_and(|best| (best.score - candidate.score).abs() < 0.15)
            })
            .unwrap_or(false);

    Ok(Json(EntitySearchResponse {
        route: plan.route,
        intent: plan.intent,
        query,
        best_match,
        candidates: std::mem::take(&mut candidates),
        ambiguous,
    }))
}

pub(crate) async fn post_entity_link(
    State(state): State<AppState>,
    Json(req): Json<EntityLinkRequest>,
) -> Result<Json<EntityLinkResponse>, (StatusCode, String)> {
    let link = state.store.link_entity(&req).map_err(internal_error)?;
    Ok(Json(EntityLinkResponse { link }))
}

pub(crate) async fn get_entity_links(
    State(state): State<AppState>,
    Query(req): Query<EntityLinksRequest>,
) -> Result<Json<EntityLinksResponse>, (StatusCode, String)> {
    let links = state.store.links_for_entity(&req).map_err(internal_error)?;
    Ok(Json(EntityLinksResponse {
        entity_id: req.entity_id,
        links,
    }))
}

pub(crate) async fn get_entity_recall(
    State(state): State<AppState>,
    Query(req): Query<AssociativeRecallRequest>,
) -> Result<Json<AssociativeRecallResponse>, (StatusCode, String)> {
    let response = state.associative_recall(&req).map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn get_agent_profile(
    State(state): State<AppState>,
    Query(req): Query<AgentProfileRequest>,
) -> Result<Json<AgentProfileResponse>, (StatusCode, String)> {
    let profile = state.store.agent_profile(&req).map_err(internal_error)?;
    Ok(Json(AgentProfileResponse { profile }))
}

pub(crate) async fn post_agent_profile(
    State(state): State<AppState>,
    Json(req): Json<AgentProfileUpsertRequest>,
) -> Result<Json<AgentProfileResponse>, (StatusCode, String)> {
    let profile = state
        .store
        .upsert_agent_profile(&req)
        .map_err(internal_error)?;
    Ok(Json(AgentProfileResponse {
        profile: Some(profile),
    }))
}

pub(crate) async fn get_source_memory(
    State(state): State<AppState>,
    Query(req): Query<SourceMemoryRequest>,
) -> Result<Json<SourceMemoryResponse>, (StatusCode, String)> {
    let response = state.store.source_memory(&req).map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn get_workspace_memory(
    State(state): State<AppState>,
    Query(req): Query<WorkspaceMemoryRequest>,
) -> Result<Json<WorkspaceMemoryResponse>, (StatusCode, String)> {
    let response = state.store.workspace_memory(&req).map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_hive_message(
    State(state): State<AppState>,
    Json(req): Json<HiveMessageSendRequest>,
) -> Result<Json<HiveMessagesResponse>, (StatusCode, String)> {
    if req.from_session.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "from_session must not be empty".to_string(),
        ));
    }
    if req.to_session.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "to_session must not be empty".to_string(),
        ));
    }
    if req.content.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "content must not be empty".to_string(),
        ));
    }

    let response = state
        .store
        .send_hive_message(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn get_hive_inbox(
    State(state): State<AppState>,
    Query(req): Query<HiveMessageInboxRequest>,
) -> Result<Json<HiveMessagesResponse>, (StatusCode, String)> {
    if req.session.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "session must not be empty".to_string(),
        ));
    }
    let response = state.store.hive_inbox(&req).map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_hive_ack(
    State(state): State<AppState>,
    Json(req): Json<HiveMessageAckRequest>,
) -> Result<Json<HiveMessagesResponse>, (StatusCode, String)> {
    if req.session.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "session must not be empty".to_string(),
        ));
    }
    if req.id.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "id must not be empty".to_string()));
    }
    let response = state.store.ack_hive_message(&req).map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn get_hive_coordination_inbox(
    State(state): State<AppState>,
    Query(req): Query<HiveCoordinationInboxRequest>,
) -> Result<Json<HiveCoordinationInboxResponse>, (StatusCode, String)> {
    if req.session.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "session must not be empty".to_string(),
        ));
    }
    let response = state
        .store
        .hive_coordination_inbox(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_hive_coordination_receipt(
    State(state): State<AppState>,
    Json(req): Json<HiveCoordinationReceiptRequest>,
) -> Result<Json<HiveCoordinationReceiptsResponse>, (StatusCode, String)> {
    if req.kind.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "kind must not be empty".to_string(),
        ));
    }
    if req.actor_session.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "actor_session must not be empty".to_string(),
        ));
    }
    if req.summary.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "summary must not be empty".to_string(),
        ));
    }
    let response = state
        .store
        .record_hive_coordination_receipt(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn get_hive_coordination_receipts(
    State(state): State<AppState>,
    Query(req): Query<HiveCoordinationReceiptsRequest>,
) -> Result<Json<HiveCoordinationReceiptsResponse>, (StatusCode, String)> {
    let response = state
        .store
        .hive_coordination_receipts(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_skill_policy_apply_receipt(
    State(state): State<AppState>,
    Json(req): Json<SkillPolicyApplyRequest>,
) -> Result<Json<SkillPolicyApplyResponse>, (StatusCode, String)> {
    if req.bundle_root.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "bundle_root must not be empty".to_string(),
        ));
    }
    if req.source_queue_path.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "source_queue_path must not be empty".to_string(),
        ));
    }
    let response = state
        .store
        .record_skill_policy_apply_receipt(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn get_skill_policy_apply_receipts(
    State(state): State<AppState>,
    Query(req): Query<SkillPolicyApplyReceiptsRequest>,
) -> Result<Json<SkillPolicyApplyReceiptsResponse>, (StatusCode, String)> {
    let response = state
        .store
        .skill_policy_apply_receipts(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn get_skill_policy_activations(
    State(state): State<AppState>,
    Query(req): Query<SkillPolicyActivationEntriesRequest>,
) -> Result<Json<SkillPolicyActivationEntriesResponse>, (StatusCode, String)> {
    let response = state
        .store
        .skill_policy_activations(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_hive_claim_acquire(
    State(state): State<AppState>,
    Json(req): Json<HiveClaimAcquireRequest>,
) -> Result<Json<HiveClaimsResponse>, (StatusCode, String)> {
    if req.scope.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "scope must not be empty".to_string(),
        ));
    }
    if req.session.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "session must not be empty".to_string(),
        ));
    }
    let response = state
        .store
        .acquire_hive_claim(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_hive_claim_release(
    State(state): State<AppState>,
    Json(req): Json<HiveClaimReleaseRequest>,
) -> Result<Json<HiveClaimsResponse>, (StatusCode, String)> {
    if req.scope.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "scope must not be empty".to_string(),
        ));
    }
    if req.session.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "session must not be empty".to_string(),
        ));
    }
    let response = state
        .store
        .release_hive_claim(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_hive_claim_transfer(
    State(state): State<AppState>,
    Json(req): Json<HiveClaimTransferRequest>,
) -> Result<Json<HiveClaimsResponse>, (StatusCode, String)> {
    if req.scope.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "scope must not be empty".to_string(),
        ));
    }
    if req.from_session.trim().is_empty() || req.to_session.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "from_session and to_session must not be empty".to_string(),
        ));
    }
    let response = state
        .store
        .transfer_hive_claim(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_hive_claim_recover(
    State(state): State<AppState>,
    Json(req): Json<HiveClaimRecoverRequest>,
) -> Result<Json<HiveClaimsResponse>, (StatusCode, String)> {
    if req.scope.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "scope must not be empty".to_string(),
        ));
    }
    if req.from_session.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "from_session must not be empty".to_string(),
        ));
    }
    if let Some(to_session) = req.to_session.as_deref() {
        if to_session.trim().is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                "to_session must not be empty".to_string(),
            ));
        }
    }
    let response = state
        .store
        .recover_hive_claim(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn get_hive_claims(
    State(state): State<AppState>,
    Query(req): Query<HiveClaimsRequest>,
) -> Result<Json<HiveClaimsResponse>, (StatusCode, String)> {
    let response = state.store.hive_claims(&req).map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_hive_session_upsert(
    State(state): State<AppState>,
    Json(req): Json<HiveSessionUpsertRequest>,
) -> Result<Json<HiveSessionsResponse>, (StatusCode, String)> {
    if req.session.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "session must not be empty".to_string(),
        ));
    }
    let response = state
        .store
        .upsert_hive_session(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_hive_session_retire(
    State(state): State<AppState>,
    Json(req): Json<HiveSessionRetireRequest>,
) -> Result<Json<HiveSessionRetireResponse>, (StatusCode, String)> {
    if req.session.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "session must not be empty".to_string(),
        ));
    }
    let response = state
        .store
        .retire_hive_session(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_hive_session_auto_retire(
    State(state): State<AppState>,
    Json(req): Json<HiveSessionAutoRetireRequest>,
) -> Result<Json<HiveSessionAutoRetireResponse>, (StatusCode, String)> {
    let response = state
        .store
        .auto_retire_stale_hive_sessions(
            req.project.as_deref(),
            req.namespace.as_deref(),
            req.workspace.as_deref(),
            Utc::now(),
        )
        .map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn get_hive_sessions(
    State(state): State<AppState>,
    Query(req): Query<HiveSessionsRequest>,
) -> Result<Json<HiveSessionsResponse>, (StatusCode, String)> {
    let response = state.store.hive_sessions(&req).map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn get_hive_board(
    State(state): State<AppState>,
    Query(req): Query<HiveBoardRequest>,
) -> Result<Json<HiveBoardResponse>, (StatusCode, String)> {
    state
        .store
        .auto_retire_stale_hive_sessions(
            req.project.as_deref(),
            req.namespace.as_deref(),
            req.workspace.as_deref(),
            Utc::now(),
        )
        .map_err(internal_error)?;
    let response = state.store.hive_board(&req).map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn get_hive_roster(
    State(state): State<AppState>,
    Query(req): Query<HiveRosterRequest>,
) -> Result<Json<HiveRosterResponse>, (StatusCode, String)> {
    let response = state.store.hive_roster(&req).map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn get_hive_follow(
    State(state): State<AppState>,
    Query(req): Query<HiveFollowRequest>,
) -> Result<Json<HiveFollowResponse>, (StatusCode, String)> {
    if req.session.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "session must not be empty".to_string(),
        ));
    }
    let response = state.store.hive_follow(&req).map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_hive_queen_deny(
    State(state): State<AppState>,
    Json(req): Json<HiveQueenActionRequest>,
) -> Result<Json<HiveQueenActionResponse>, (StatusCode, String)> {
    let target_session = require_hive_queen_target(&req)?;
    let target = state
        .store
        .hive_follow(&HiveFollowRequest {
            session: target_session.clone(),
            current_session: Some(req.queen_session.clone()),
            project: req.project.clone(),
            namespace: req.namespace.clone(),
            workspace: req.workspace.clone(),
        })
        .map_err(internal_error)?;
    let receipt = state
        .store
        .record_hive_coordination_receipt(&HiveCoordinationReceiptRequest {
            kind: "queen_deny".to_string(),
            actor_session: req.queen_session.clone(),
            actor_agent: Some("dashboard".to_string()),
            target_session: Some(target_session.clone()),
            task_id: target.target.task_id.clone(),
            scope: target.touch_points.first().cloned(),
            project: req.project.clone(),
            namespace: req.namespace.clone(),
            workspace: req.workspace.clone(),
            summary: format!(
                "Queen denied overlapping lane or scope work for session {}.",
                target_session
            ),
        })
        .map_err(internal_error)?
        .receipts
        .into_iter()
        .next();
    Ok(Json(HiveQueenActionResponse {
        action: "deny".to_string(),
        target_session: Some(target_session.clone()),
        receipt,
        message_id: None,
        retired: Vec::new(),
        summary: format!("Denied focused bee: {}", target_session),
        follow_session: Some(target_session),
    }))
}

pub(crate) async fn post_hive_queen_reroute(
    State(state): State<AppState>,
    Json(req): Json<HiveQueenActionRequest>,
) -> Result<Json<HiveQueenActionResponse>, (StatusCode, String)> {
    let target_session = require_hive_queen_target(&req)?;
    let target = state
        .store
        .hive_follow(&HiveFollowRequest {
            session: target_session.clone(),
            current_session: Some(req.queen_session.clone()),
            project: req.project.clone(),
            namespace: req.namespace.clone(),
            workspace: req.workspace.clone(),
        })
        .map_err(internal_error)?;
    let receipt = state
        .store
        .record_hive_coordination_receipt(&HiveCoordinationReceiptRequest {
            kind: "queen_reroute".to_string(),
            actor_session: req.queen_session.clone(),
            actor_agent: Some("dashboard".to_string()),
            target_session: Some(target_session.clone()),
            task_id: target.target.task_id.clone(),
            scope: target.touch_points.first().cloned(),
            project: req.project.clone(),
            namespace: req.namespace.clone(),
            workspace: req.workspace.clone(),
            summary: format!(
                "Queen ordered session {} onto a new isolated lane.",
                target_session
            ),
        })
        .map_err(internal_error)?
        .receipts
        .into_iter()
        .next();
    Ok(Json(HiveQueenActionResponse {
        action: "reroute".to_string(),
        target_session: Some(target_session.clone()),
        receipt,
        message_id: None,
        retired: Vec::new(),
        summary: format!("Reroute recorded for: {}", target_session),
        follow_session: Some(target_session),
    }))
}

pub(crate) async fn post_hive_queen_handoff(
    State(state): State<AppState>,
    Json(req): Json<HiveQueenActionRequest>,
) -> Result<Json<HiveQueenActionResponse>, (StatusCode, String)> {
    let target_session = require_hive_queen_target(&req)?;
    let scope = req
        .scope
        .as_deref()
        .and_then(|value| {
            let value = value.trim();
            if value.is_empty() { None } else { Some(value) }
        })
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                "scope must not be empty".to_string(),
            )
        })?
        .to_string();
    let target = state
        .store
        .hive_follow(&HiveFollowRequest {
            session: target_session.clone(),
            current_session: Some(req.queen_session.clone()),
            project: req.project.clone(),
            namespace: req.namespace.clone(),
            workspace: req.workspace.clone(),
        })
        .map_err(internal_error)?;
    let receipt = state
        .store
        .record_hive_coordination_receipt(&HiveCoordinationReceiptRequest {
            kind: "queen_handoff".to_string(),
            actor_session: req.queen_session.clone(),
            actor_agent: Some("dashboard".to_string()),
            target_session: Some(target_session.clone()),
            task_id: target.target.task_id.clone(),
            scope: Some(scope.clone()),
            project: req.project.clone(),
            namespace: req.namespace.clone(),
            workspace: req.workspace.clone(),
            summary: format!(
                "Queen handed off scope {} to session {}.",
                scope, target_session
            ),
        })
        .map_err(internal_error)?
        .receipts
        .into_iter()
        .next();
    let message = state
        .store
        .send_hive_message(&HiveMessageSendRequest {
            kind: "handoff".to_string(),
            from_session: req.queen_session.clone(),
            from_agent: Some("dashboard".to_string()),
            to_session: target_session.clone(),
            project: req.project.clone(),
            namespace: req.namespace.clone(),
            workspace: req.workspace.clone(),
            content: req
                .note
                .as_deref()
                .and_then(|value| {
                    let value = value.trim();
                    if value.is_empty() { None } else { Some(value) }
                })
                .map(|note| format!("handoff_scope: {scope}\n{note}"))
                .unwrap_or_else(|| format!("handoff_scope: {scope}")),
        })
        .map_err(internal_error)?
        .messages
        .into_iter()
        .next();
    Ok(Json(HiveQueenActionResponse {
        action: "handoff".to_string(),
        target_session: Some(target_session.clone()),
        receipt,
        message_id: message.as_ref().map(|entry| entry.id.clone()),
        retired: Vec::new(),
        summary: format!("Handoff recorded for: {} on {}", target_session, scope),
        follow_session: Some(target_session),
    }))
}

pub(crate) fn require_hive_queen_target(
    req: &HiveQueenActionRequest,
) -> Result<String, (StatusCode, String)> {
    if req.queen_session.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "queen_session must not be empty".to_string(),
        ));
    }
    req.target_session
        .as_deref()
        .and_then(|value| {
            let value = value.trim();
            if value.is_empty() { None } else { Some(value) }
        })
        .map(str::to_string)
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                "target_session must not be empty".to_string(),
            )
        })
}

pub(crate) async fn post_hive_task_upsert(
    State(state): State<AppState>,
    Json(req): Json<HiveTaskUpsertRequest>,
) -> Result<Json<HiveTasksResponse>, (StatusCode, String)> {
    if req.task_id.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "task_id must not be empty".to_string(),
        ));
    }
    if req.title.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "title must not be empty".to_string(),
        ));
    }
    let response = state.store.upsert_hive_task(&req).map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_hive_task_assign(
    State(state): State<AppState>,
    Json(req): Json<HiveTaskAssignRequest>,
) -> Result<Json<HiveTasksResponse>, (StatusCode, String)> {
    if req.task_id.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "task_id must not be empty".to_string(),
        ));
    }
    if req.to_session.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "to_session must not be empty".to_string(),
        ));
    }
    let response = state.store.assign_hive_task(&req).map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn get_hive_tasks(
    State(state): State<AppState>,
    Query(req): Query<HiveTasksRequest>,
) -> Result<Json<HiveTasksResponse>, (StatusCode, String)> {
    let response = state.store.hive_tasks(&req).map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn get_timeline(
    State(state): State<AppState>,
    Query(req): Query<TimelineMemoryRequest>,
) -> Result<Json<TimelineMemoryResponse>, (StatusCode, String)> {
    let plan = RetrievalPlan::resolve(req.route, req.intent);
    let limit = req.limit.unwrap_or(12).min(32);
    state
        .record_retrieval_feedback_for_item(req.id, 0.05, "retrieved_timeline", &plan)
        .map_err(internal_error)?;
    let (entity, events) = state.entity_view(req.id, limit).map_err(internal_error)?;

    Ok(Json(TimelineMemoryResponse {
        route: plan.route,
        intent: plan.intent,
        entity,
        events,
    }))
}

pub(crate) async fn decay_memory(
    State(state): State<AppState>,
    Json(req): Json<MemoryDecayRequest>,
) -> Result<Json<MemoryDecayResponse>, (StatusCode, String)> {
    let (scanned, updated, events) = state.store.decay_entities(&req).map_err(internal_error)?;
    Ok(Json(MemoryDecayResponse {
        scanned,
        updated,
        events,
    }))
}

pub(crate) async fn consolidate_memory(
    State(state): State<AppState>,
    Json(req): Json<MemoryConsolidationRequest>,
) -> Result<Json<MemoryConsolidationResponse>, (StatusCode, String)> {
    let response = state
        .consolidate_semantic_memory(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn get_maintenance_report(
    State(state): State<AppState>,
    Query(req): Query<MemoryMaintenanceReportRequest>,
) -> Result<Json<MemoryMaintenanceReportResponse>, (StatusCode, String)> {
    let response = state.maintenance_report(&req).map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_runtime_maintain(
    State(state): State<AppState>,
    Json(req): Json<MaintainReportRequest>,
) -> Result<Json<MaintainReport>, (StatusCode, String)> {
    let response = state.maintain_runtime(&req).map_err(internal_error)?;
    Ok(Json(response))
}

impl AppState {
    pub(crate) fn rehearse_items(&self, items: &[MemoryItem], limit: usize) -> anyhow::Result<()> {
        for item in items.iter().take(limit) {
            let canonical_key = canonical_key(item);
            let _ = self
                .store
                .rehearse_entity_for_item(item, &canonical_key, 0.02)?;
        }
        Ok(())
    }

    pub(crate) fn rehearse_item(&self, item_id: Uuid, salience_boost: f32) -> anyhow::Result<()> {
        if let Some(item) = self.store.get(item_id)? {
            let canonical_key = canonical_key(&item);
            let _ = self
                .store
                .rehearse_entity_for_item(&item, &canonical_key, salience_boost)?;
        }
        Ok(())
    }

    pub(crate) fn record_retrieval_feedback(
        &self,
        items: &[MemoryItem],
        limit: usize,
        event_type: &str,
        plan: &RetrievalPlan,
    ) -> anyhow::Result<()> {
        for item in items.iter().take(limit) {
            let canonical_key = canonical_key(item);
            let entity = self.store.resolve_entity_for_item(item, &canonical_key)?;
            let mut tags = vec![
                "retrieval_feedback".to_string(),
                format!("route:{}", enum_label_route(plan.route)),
                format!("intent:{}", enum_label_intent(plan.intent)),
            ];
            if let Some(branch) = &item.belief_branch {
                tags.push(format!("belief_branch:{branch}"));
            }
            let context = Some(entity_context_frame(&entity.record, item));
            self.store.record_event(
                &entity.record,
                item.id,
                RecordEventArgs {
                    event_type: event_type.to_string(),
                    summary: format!(
                        "{} route={} intent={}",
                        event_type,
                        enum_label_route(plan.route),
                        enum_label_intent(plan.intent)
                    ),
                    occurred_at: Utc::now(),
                    project: item.project.clone(),
                    namespace: item.namespace.clone(),
                    workspace: item.workspace.clone(),
                    source_agent: item.source_agent.clone(),
                    source_system: item.source_system.clone(),
                    source_path: item.source_path.clone(),
                    related_entity_ids: Vec::new(),
                    tags,
                    context,
                    confidence: item.confidence,
                    salience_score: entity.record.salience_score,
                },
            )?;
        }
        Ok(())
    }

    pub(crate) fn record_retrieval_feedback_for_item(
        &self,
        item_id: Uuid,
        salience_boost: f32,
        event_type: &str,
        plan: &RetrievalPlan,
    ) -> anyhow::Result<()> {
        self.rehearse_item(item_id, salience_boost)?;
        if let Some(item) = self.store.get(item_id)? {
            self.record_retrieval_feedback(std::slice::from_ref(&item), 1, event_type, plan)?;
        }
        Ok(())
    }

    pub(crate) fn consolidate_semantic_memory(
        &self,
        req: &MemoryConsolidationRequest,
    ) -> anyhow::Result<MemoryConsolidationResponse> {
        let policy = working::memory_policy_snapshot();
        let consolidation_policy = &policy.consolidation;
        let candidates = self
            .store
            .consolidation_candidates(req)
            .context("load consolidation candidates")?;

        let min_salience = req
            .min_salience
            .unwrap_or(consolidation_policy.min_salience)
            .clamp(0.0, 1.0);
        let record_events = req
            .record_events
            .unwrap_or(consolidation_policy.record_events);

        let mut scanned = 0usize;
        let mut groups = 0usize;
        let mut consolidated = 0usize;
        let mut duplicates = 0usize;
        let mut events = 0usize;
        let mut highlights = Vec::new();

        for candidate in candidates {
            scanned += candidate.event_count;
            groups += 1;

            if candidate.entity.salience_score < min_salience
                && candidate.entity.rehearsal_count < candidate.event_count as u64
            {
                continue;
            }

            let content = consolidation_content(
                &candidate.entity,
                candidate.event_count,
                candidate.first_recorded_at,
                candidate.last_recorded_at,
            );
            let scope = consolidation_scope(&candidate.entity);
            let kind = consolidation_kind(&candidate.entity.entity_type);
            let confidence =
                (candidate.entity.confidence + (candidate.event_count as f32 * 0.05)).min(1.0);
            let tags = consolidation_tags(&candidate.entity, candidate.event_count);
            let source_system = candidate
                .entity
                .context
                .as_ref()
                .and_then(|context| context.repo.clone())
                .or_else(|| {
                    candidate
                        .entity
                        .context
                        .as_ref()
                        .and_then(|context| context.location.clone())
                });

            let (item, duplicate) = self.store_item(
                StoreMemoryRequest {
                    content,
                    kind,
                    scope,
                    project: candidate
                        .entity
                        .context
                        .as_ref()
                        .and_then(|context| context.project.clone()),
                    namespace: candidate
                        .entity
                        .context
                        .as_ref()
                        .and_then(|context| context.namespace.clone()),
                    workspace: candidate
                        .entity
                        .context
                        .as_ref()
                        .and_then(|context| context.workspace.clone()),
                    visibility: Some(MemoryVisibility::Workspace),
                    belief_branch: None,
                    source_agent: candidate
                        .entity
                        .context
                        .as_ref()
                        .and_then(|context| context.agent.clone()),
                    source_system: source_system.clone(),
                    source_path: candidate
                        .entity
                        .context
                        .as_ref()
                        .and_then(|context| context.location.clone()),
                    source_quality: Some(SourceQuality::Derived),
                    confidence: Some(confidence),
                    ttl_seconds: Some(60 * 60 * 24 * 90),
                    last_verified_at: Some(candidate.last_recorded_at),
                    supersedes: Vec::new(),
                    tags,
                    status: Some(MemoryStatus::Active),
                },
                MemoryStage::Canonical,
            )?;

            if duplicate.is_some() {
                duplicates += 1;
                continue;
            }

            if highlights.len() < 3 {
                highlights.push(format!(
                    "{}:{} events salience={:.2}",
                    candidate.entity.entity_type,
                    candidate.event_count,
                    candidate.entity.salience_score
                ));
            }
            consolidated += 1;
            if record_events {
                let context = Some(entity_context_frame(&candidate.entity, &item));
                let _ = self.store.record_event(
                    &candidate.entity,
                    item.id,
                    RecordEventArgs {
                        event_type: "consolidated".to_string(),
                        summary: format!(
                            "episodic traces consolidated after {} events into semantic memory",
                            candidate.event_count
                        ),
                        occurred_at: candidate.last_recorded_at,
                        project: candidate
                            .entity
                            .context
                            .as_ref()
                            .and_then(|context| context.project.clone()),
                        namespace: candidate
                            .entity
                            .context
                            .as_ref()
                            .and_then(|context| context.namespace.clone()),
                        workspace: candidate
                            .entity
                            .context
                            .as_ref()
                            .and_then(|context| context.workspace.clone()),
                        source_agent: candidate
                            .entity
                            .context
                            .as_ref()
                            .and_then(|context| context.agent.clone()),
                        source_system: candidate
                            .entity
                            .context
                            .as_ref()
                            .and_then(|context| context.repo.clone())
                            .or_else(|| {
                                candidate
                                    .entity
                                    .context
                                    .as_ref()
                                    .and_then(|context| context.location.clone())
                            }),
                        source_path: candidate
                            .entity
                            .context
                            .as_ref()
                            .and_then(|context| context.location.clone()),
                        related_entity_ids: vec![item.id],
                        tags: consolidation_tags(&candidate.entity, candidate.event_count),
                        context,
                        confidence: item.confidence,
                        salience_score: candidate.entity.salience_score,
                    },
                )?;
                events += 1;
            }
        }

        Ok(MemoryConsolidationResponse {
            scanned,
            groups,
            consolidated,
            duplicates,
            events,
            highlights,
        })
    }

    pub(crate) fn maintenance_report(
        &self,
        req: &MemoryMaintenanceReportRequest,
    ) -> anyhow::Result<MemoryMaintenanceReportResponse> {
        let (
            reinforced_candidates,
            cooled_candidates,
            consolidated_candidates,
            stale_items,
            skipped,
            highlights,
        ) = self.store.maintenance_report(req)?;

        Ok(MemoryMaintenanceReportResponse {
            reinforced_candidates,
            cooled_candidates,
            consolidated_candidates,
            stale_items,
            skipped,
            highlights,
            receipt_id: Some(uuid::Uuid::new_v4().to_string()),
            mode: req.mode.clone().or_else(|| Some("scan".to_string())),
            compacted_items: if req.mode.as_deref() == Some("compact") {
                consolidated_candidates
            } else {
                0
            },
            refreshed_items: if req.mode.as_deref() == Some("refresh") {
                reinforced_candidates
            } else {
                0
            },
            repaired_items: if req.mode.as_deref() == Some("repair") {
                cooled_candidates
            } else {
                0
            },
            generated_at: Utc::now(),
        })
    }

    pub(crate) fn maintain_runtime(
        &self,
        req: &MaintainReportRequest,
    ) -> anyhow::Result<MaintainReport> {
        self.store.maintain_runtime(req)
    }

    pub(crate) fn entity_view(
        &self,
        item_id: Uuid,
        limit: usize,
    ) -> anyhow::Result<(Option<MemoryEntityRecord>, Vec<MemoryEventRecord>)> {
        let entity = self.store.entity_for_item(item_id)?;
        let events = match &entity {
            Some(entity) => self.store.events_for_entity(entity.id, limit)?,
            None => Vec::new(),
        };
        Ok((entity, events))
    }

    pub(crate) fn associative_recall(
        &self,
        req: &AssociativeRecallRequest,
    ) -> anyhow::Result<AssociativeRecallResponse> {
        let depth_limit = req.depth.unwrap_or(2).clamp(1, 4);
        let hit_limit = req.limit.unwrap_or(8).clamp(1, 24);
        let Some(root) = self.store.entity_by_id(req.entity_id)? else {
            return Ok(AssociativeRecallResponse {
                root_entity: None,
                hits: Vec::new(),
                links: Vec::new(),
                truncated: false,
            });
        };

        let mut hits = vec![AssociativeRecallHit {
            entity: root.clone(),
            depth: 0,
            via: None,
            score: 1.0,
            reasons: vec!["root".to_string()],
        }];
        let mut links = Vec::new();
        let mut seen_entities = HashSet::from([root.id]);
        let mut seen_links = HashSet::new();
        let mut queue = VecDeque::from([(root.id, 0usize)]);
        let mut truncated = false;

        while let Some((entity_id, depth)) = queue.pop_front() {
            if depth >= depth_limit || hits.len() >= hit_limit {
                continue;
            }

            let entity_links = self
                .store
                .links_for_entity(&EntityLinksRequest { entity_id })?;
            for link in entity_links {
                if seen_links.insert(link.id) && links.len() < hit_limit.saturating_mul(2) {
                    links.push(link.clone());
                }

                let next_id = if link.from_entity_id == entity_id {
                    link.to_entity_id
                } else {
                    link.from_entity_id
                };

                if !seen_entities.insert(next_id) {
                    continue;
                }

                let Some(entity) = self.store.entity_by_id(next_id)? else {
                    continue;
                };

                let _ = self.store.rehearse_entity_by_id(entity.id, 0.04)?;
                let score = associative_recall_score(&entity, &link, depth + 1, &root);
                let reasons = associative_recall_reasons(&entity, &link, depth + 1);
                hits.push(AssociativeRecallHit {
                    entity: entity.clone(),
                    depth: depth + 1,
                    via: Some(link.clone()),
                    score,
                    reasons,
                });
                queue.push_back((next_id, depth + 1));

                if hits.len() >= hit_limit {
                    truncated = true;
                    break;
                }
            }

            if hits.len() >= hit_limit {
                break;
            }
        }

        let _ = self.store.rehearse_entity_by_id(root.id, 0.05)?;
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.depth.cmp(&b.depth))
                .then_with(|| b.entity.updated_at.cmp(&a.entity.updated_at))
        });

        Ok(AssociativeRecallResponse {
            root_entity: Some(root),
            hits,
            links,
            truncated,
        })
    }
}

// ---------------------------------------------------------------------------
// Atlas routes
// ---------------------------------------------------------------------------

pub(crate) async fn get_atlas_regions(
    State(state): State<AppState>,
    Query(req): Query<AtlasRegionsRequest>,
) -> Result<Json<AtlasRegionsResponse>, (StatusCode, String)> {
    let response = state.store.list_atlas_regions(&req).map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_atlas_explore(
    State(state): State<AppState>,
    Json(req): Json<AtlasExploreRequest>,
) -> Result<Json<AtlasExploreResponse>, (StatusCode, String)> {
    let response = state.store.explore_atlas(&req).map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_atlas_expand(
    State(state): State<AppState>,
    Json(req): Json<AtlasExpandRequest>,
) -> Result<Json<AtlasExpandResponse>, (StatusCode, String)> {
    let response = state.store.atlas_expand(&req).map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_atlas_generate(
    State(state): State<AppState>,
    Json(req): Json<AtlasRegionsRequest>,
) -> Result<Json<AtlasRegionsResponse>, (StatusCode, String)> {
    let regions = state
        .store
        .generate_regions_for_project(
            req.project.as_deref(),
            req.namespace.as_deref(),
            req.lane.as_deref(),
        )
        .map_err(internal_error)?;
    Ok(Json(AtlasRegionsResponse { regions }))
}

#[derive(Clone)]
pub(crate) struct MemoryViewItem {
    pub(crate) item: MemoryItem,
    pub(crate) entity: Option<MemoryEntityRecord>,
    pub(crate) source_trust_score: f32,
}
