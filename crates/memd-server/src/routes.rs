use super::errors::MemdError;
use super::*;
use memd_schema::{PressureMetrics, SearchItemTrace, SearchRetrievalTrace, SearchSignalTrace};
#[path = "routes_context_packet.rs"]
mod routes_context_packet;
#[path = "routes_prompt_firewall.rs"]
mod routes_prompt_firewall;

use routes_context_packet::*;
use routes_prompt_firewall::*;

// B3-T6: atlas-at-recall 1-hop expansion flag. Default on because atlas/entity
// recall is part of the mandatory in-house retrieval core.
fn atlas_recall_enabled() -> bool {
    parse_atlas_recall_enabled(std::env::var("MEMD_RETRIEVAL_ATLAS_RECALL").ok().as_deref())
}

fn parse_atlas_recall_enabled(value: Option<&str>) -> bool {
    value
        .map(|value| {
            let value = value.trim().to_ascii_lowercase();
            !matches!(value.as_str(), "0" | "false" | "off" | "no")
        })
        .unwrap_or(true)
}

pub(crate) fn resolve_region_member_filter(
    state: &AppState,
    region: Option<&str>,
    project: Option<&str>,
    namespace: Option<&str>,
) -> anyhow::Result<Option<std::collections::HashSet<Uuid>>> {
    let Some(region) = region.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };

    let mut regions = state
        .store
        .list_atlas_regions(&memd_schema::AtlasRegionsRequest {
            project: project.map(str::to_string),
            namespace: namespace.map(str::to_string),
            lane: None,
            limit: None,
        })?
        .regions;
    if regions.is_empty() {
        regions = state
            .store
            .generate_regions_for_project(project, namespace, None)?;
    }

    let needle = region.to_ascii_lowercase();
    let matched = regions.into_iter().find(|candidate| {
        candidate.name.eq_ignore_ascii_case(region)
            || candidate.id.to_string() == region
            || candidate.id.to_string().starts_with(&needle)
    });
    let Some(region) = matched else {
        let fallback_members = state
            .store
            .list()?
            .into_iter()
            .filter(|item| item.status == MemoryStatus::Active)
            .filter(|item| project.is_none_or(|value| item.project.as_deref() == Some(value)))
            .filter(|item| namespace.is_none_or(|value| item.namespace.as_deref() == Some(value)))
            .filter(|item| {
                crate::atlas::region_bucket_key(item, None)
                    .is_some_and(|bucket| bucket.eq_ignore_ascii_case(region))
            })
            .map(|item| item.id)
            .collect::<std::collections::HashSet<_>>();
        return Ok(Some(fallback_members));
    };
    let member_ids = state
        .store
        .get_region_member_ids(region.id)?
        .into_iter()
        .collect::<std::collections::HashSet<_>>();
    Ok(Some(member_ids))
}

include!("routes_search.rs");
fn authority_search_enabled() -> bool {
    match std::env::var("MEMD_AUTHORITY_SEARCH") {
        Ok(value) => {
            let value = value.trim().to_ascii_lowercase();
            matches!(value.as_str(), "1" | "true" | "on" | "yes")
        }
        Err(_) => false,
    }
}

fn authority_search_token() -> Option<String> {
    std::env::var("MEMD_AUTHORITY_TOKEN")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn bearer_token(value: &str) -> Option<&str> {
    value
        .trim()
        .strip_prefix("Bearer ")
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn authority_header_allowed(headers: &axum::http::HeaderMap) -> bool {
    let Some(expected) = authority_search_token() else {
        return false;
    };
    let direct = headers
        .get("x-memd-authority-token")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value == expected);
    let bearer = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(bearer_token)
        .is_some_and(|value| value == expected);
    direct || bearer
}

pub(crate) async fn search_memory_authority(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<SearchMemoryRequest>,
) -> Result<Json<SearchMemoryResponse>, (StatusCode, String)> {
    if !authority_search_enabled() {
        return Err((
            StatusCode::NOT_FOUND,
            "memd authority search is not enabled".to_string(),
        ));
    }
    if !authority_header_allowed(&headers) {
        return Err((
            StatusCode::UNAUTHORIZED,
            "memd authority token required".to_string(),
        ));
    }

    let mut items = state
        .snapshot_for_scope(req.project.as_deref(), req.namespace.as_deref())
        .map_err(internal_error)?;
    let region_member_ids = resolve_region_member_filter(
        &state,
        req.region.as_deref(),
        req.project.as_deref(),
        req.namespace.as_deref(),
    )
    .map_err(internal_error)?;
    if let Some(allowed_ids) = region_member_ids.as_ref() {
        items.retain(|item| allowed_ids.contains(&item.id));
    }
    let plan = RetrievalPlan::resolve(req.route, req.intent);
    let items = filter_raw_items_authority(&items, &req, &plan);

    Ok(Json(SearchMemoryResponse {
        route: plan.route,
        intent: plan.intent,
        items,
        trace: None,
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
    if crate::store_runtime_maintenance::expired_item_gc_enabled() {
        let _ = state.store.gc_expired_items(3600);
    }
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

pub(crate) async fn get_context_packet(
    State(state): State<AppState>,
    Query(req): Query<ContextPacketRequest>,
) -> Result<Json<ContextPacketResponse>, (StatusCode, String)> {
    if crate::store_runtime_maintenance::expired_item_gc_enabled() {
        let _ = state.store.gc_expired_items(3600);
    }
    let context_req = ContextRequest {
        project: req.project.clone(),
        agent: req.agent.clone(),
        workspace: req.workspace.clone(),
        visibility: req.visibility,
        route: req.route,
        intent: req.intent,
        limit: req.limit,
        max_chars_per_item: req.max_chars_per_item,
    };
    let context_req = apply_agent_profile_defaults(&state, context_req).map_err(internal_error)?;
    let policy = working::memory_policy_snapshot();
    let feedback_limit = policy.retrieval_feedback.max_items_per_request;
    let BuildContextResult {
        plan,
        retrieval_order,
        items,
    } = build_context(&state, &context_req)?;
    state
        .rehearse_items(&items, feedback_limit)
        .map_err(internal_error)?;
    state
        .record_retrieval_feedback(&items, feedback_limit, "retrieved_context_packet", &plan)
        .map_err(internal_error)?;

    let records = items
        .into_iter()
        .map(|item| CompactMemoryRecord {
            id: item.id,
            record: compact_record(&item),
        })
        .collect::<Vec<_>>();
    let compact = CompactContextResponse {
        route: plan.route,
        intent: plan.intent,
        retrieval_order: retrieval_order.clone(),
        records,
    };
    let model_tier = req.model_tier.as_deref().unwrap_or("cloud");
    let safety = req.safety.as_deref().unwrap_or("strict");
    let sections = build_server_context_packet_sections(&state, &context_req, &compact, &req);
    let packet = render_server_context_packet(&sections, model_tier);
    let source_ids = compact.records.iter().map(|record| record.id).collect();
    record_server_context_packet_token_savings(
        &state,
        &context_req,
        &compact,
        model_tier,
        packet.chars().count(),
    )
    .map_err(internal_error)?;

    Ok(Json(ContextPacketResponse {
        route: compact.route,
        intent: compact.intent,
        retrieval_order,
        model_tier: model_tier.to_string(),
        safety_mode: if server_context_packet_strict(safety) {
            "strict".to_string()
        } else {
            safety.to_string()
        },
        packet,
        sections,
        source_ids,
        compact,
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
        .filter(|entry| entry.item.status != MemoryStatus::Expired)
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
        .filter(|entry| crate::helpers::visibility_allows(&None, &entry.item))
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
        return Err(MemdError::validation("from_session", "must not be empty").into_wire());
    }
    if req.to_session.trim().is_empty() {
        return Err(MemdError::validation("to_session", "must not be empty").into_wire());
    }
    if req.content.trim().is_empty() {
        return Err(MemdError::validation("content", "must not be empty").into_wire());
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
        return Err(MemdError::validation("session", "must not be empty").into_wire());
    }
    let response = state.store.hive_inbox(&req).map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_hive_ack(
    State(state): State<AppState>,
    Json(req): Json<HiveMessageAckRequest>,
) -> Result<Json<HiveMessagesResponse>, (StatusCode, String)> {
    if req.session.trim().is_empty() {
        return Err(MemdError::validation("session", "must not be empty").into_wire());
    }
    if req.id.trim().is_empty() {
        return Err(MemdError::validation("id", "must not be empty").into_wire());
    }
    let response = state.store.ack_hive_message(&req).map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn get_hive_coordination_inbox(
    State(state): State<AppState>,
    Query(req): Query<HiveCoordinationInboxRequest>,
) -> Result<Json<HiveCoordinationInboxResponse>, (StatusCode, String)> {
    if req.session.trim().is_empty() {
        return Err(MemdError::validation("session", "must not be empty").into_wire());
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
        return Err(MemdError::validation("kind", "must not be empty").into_wire());
    }
    if req.actor_session.trim().is_empty() {
        return Err(MemdError::validation("actor_session", "must not be empty").into_wire());
    }
    if req.summary.trim().is_empty() {
        return Err(MemdError::validation("summary", "must not be empty").into_wire());
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
        return Err(MemdError::validation("bundle_root", "must not be empty").into_wire());
    }
    if req.source_queue_path.trim().is_empty() {
        return Err(MemdError::validation("source_queue_path", "must not be empty").into_wire());
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
        return Err(MemdError::validation("scope", "must not be empty").into_wire());
    }
    if req.session.trim().is_empty() {
        return Err(MemdError::validation("session", "must not be empty").into_wire());
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
        return Err(MemdError::validation("scope", "must not be empty").into_wire());
    }
    if req.session.trim().is_empty() {
        return Err(MemdError::validation("session", "must not be empty").into_wire());
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
        return Err(MemdError::validation("scope", "must not be empty").into_wire());
    }
    if req.from_session.trim().is_empty() || req.to_session.trim().is_empty() {
        return Err(
            MemdError::validation("from_session and to_session", "must not be empty").into_wire(),
        );
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
        return Err(MemdError::validation("scope", "must not be empty").into_wire());
    }
    if req.from_session.trim().is_empty() {
        return Err(MemdError::validation("from_session", "must not be empty").into_wire());
    }
    if let Some(to_session) = req.to_session.as_deref()
        && to_session.trim().is_empty()
    {
        return Err(MemdError::validation("to_session", "must not be empty").into_wire());
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

pub(crate) async fn post_capabilities_sync(
    State(state): State<AppState>,
    Json(req): Json<CapabilitySyncRequest>,
) -> Result<Json<CapabilitySyncResponse>, (StatusCode, String)> {
    if req.records.len() > 1000 {
        return Err(
            MemdError::validation("records", "must contain at most 1000 items").into_wire(),
        );
    }
    let response = state
        .store
        .upsert_capabilities(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn get_capabilities(
    State(state): State<AppState>,
    Query(req): Query<CapabilityListRequest>,
) -> Result<Json<CapabilityListResponse>, (StatusCode, String)> {
    let response = state
        .store
        .list_capabilities(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_access_routes_sync(
    State(state): State<AppState>,
    Json(req): Json<AccessRouteSyncRequest>,
) -> Result<Json<AccessRouteSyncResponse>, (StatusCode, String)> {
    if req.routes.len() > 1000 {
        return Err(MemdError::validation("routes", "must contain at most 1000 items").into_wire());
    }
    if req.routes.iter().any(|route| route.secret_values_stored) {
        return Err(MemdError::validation("routes", "must not contain secret values").into_wire());
    }
    let response = state
        .store
        .upsert_access_routes(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn get_access_routes(
    State(state): State<AppState>,
    Query(req): Query<AccessRouteListRequest>,
) -> Result<Json<AccessRouteListResponse>, (StatusCode, String)> {
    let response = state
        .store
        .list_access_routes(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_token_savings_sync(
    State(state): State<AppState>,
    Json(req): Json<TokenSavingsSyncRequest>,
) -> Result<Json<TokenSavingsSyncResponse>, (StatusCode, String)> {
    if req.records.len() > 5000 {
        return Err(
            MemdError::validation("records", "must contain at most 5000 items").into_wire(),
        );
    }
    let response = state
        .store
        .upsert_token_savings(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn get_token_savings(
    State(state): State<AppState>,
    Query(req): Query<TokenSavingsListRequest>,
) -> Result<Json<TokenSavingsListResponse>, (StatusCode, String)> {
    let response = state
        .store
        .list_token_savings(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_dev_server_lease_acquire(
    State(state): State<AppState>,
    Json(req): Json<DevServerLeaseAcquireRequest>,
) -> Result<Json<DevServerLeasesResponse>, (StatusCode, String)> {
    if req.scope.trim().is_empty() {
        return Err(MemdError::validation("scope", "must not be empty").into_wire());
    }
    if req.session.trim().is_empty() {
        return Err(MemdError::validation("session", "must not be empty").into_wire());
    }
    if req.host.trim().is_empty() {
        return Err(MemdError::validation("host", "must not be empty").into_wire());
    }
    if req.repo_hash.trim().is_empty() {
        return Err(MemdError::validation("repo_hash", "must not be empty").into_wire());
    }
    let response = state
        .store
        .acquire_dev_server_lease(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_dev_server_lease_release(
    State(state): State<AppState>,
    Json(req): Json<DevServerLeaseReleaseRequest>,
) -> Result<Json<DevServerLeasesResponse>, (StatusCode, String)> {
    if req.scope.trim().is_empty() {
        return Err(MemdError::validation("scope", "must not be empty").into_wire());
    }
    if req.session.trim().is_empty() {
        return Err(MemdError::validation("session", "must not be empty").into_wire());
    }
    let response = state
        .store
        .release_dev_server_lease(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn get_dev_server_leases(
    State(state): State<AppState>,
    Query(req): Query<DevServerLeasesRequest>,
) -> Result<Json<DevServerLeasesResponse>, (StatusCode, String)> {
    let response = state
        .store
        .dev_server_leases(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_hive_session_upsert(
    State(state): State<AppState>,
    Json(req): Json<HiveSessionUpsertRequest>,
) -> Result<Json<HiveSessionsResponse>, (StatusCode, String)> {
    if req.session.trim().is_empty() {
        return Err(MemdError::validation("session", "must not be empty").into_wire());
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
        return Err(MemdError::validation("session", "must not be empty").into_wire());
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
        return Err(MemdError::validation("session", "must not be empty").into_wire());
    }
    let response = state.store.hive_follow(&req).map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn get_hive_divergence(
    State(state): State<AppState>,
    Query(req): Query<DivergenceRequest>,
) -> Result<Json<DivergenceSummary>, (StatusCode, String)> {
    let response = state.store.hive_divergence(&req).map_err(internal_error)?;
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
    let effective_task_id = req
        .task_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| target.target.task_id.clone());
    if let Some(task_id) = effective_task_id.as_deref() {
        state
            .store
            .record_queen_deny(task_id, &target_session, req.note.as_deref())
            .map_err(internal_error)?;
    }
    let receipt = state
        .store
        .record_hive_coordination_receipt(&HiveCoordinationReceiptRequest {
            kind: "queen_deny".to_string(),
            actor_session: req.queen_session.clone(),
            actor_agent: Some("dashboard".to_string()),
            target_session: Some(target_session.clone()),
            task_id: effective_task_id.clone(),
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
    let new_lane = req
        .new_lane
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let affected = state
        .store
        .set_session_lane(
            &target_session,
            req.project.as_deref(),
            req.namespace.as_deref(),
            req.workspace.as_deref(),
            new_lane.as_deref(),
        )
        .map_err(internal_error)?;
    let lane_summary = new_lane
        .as_deref()
        .map(|lane| format!("lane={}", lane))
        .unwrap_or_else(|| "lane cleared".to_string());
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
                "Queen rerouted session {} ({lane_summary}, {affected} row(s) updated).",
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
        summary: format!("Reroute applied to {}: {lane_summary}", target_session),
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
        .ok_or_else(|| MemdError::validation("scope", "must not be empty").into_wire())?
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
    let effective_task_id = req
        .task_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| target.target.task_id.clone());
    let lock_version = if let Some(task_id) = effective_task_id.as_deref() {
        Some(
            state
                .store
                .apply_handoff_lock(task_id, &target_session, Some(&req.queen_session))
                .map_err(internal_error)?,
        )
    } else {
        None
    };
    let lock_fragment = match (effective_task_id.as_deref(), lock_version) {
        (Some(task), Some(version)) => {
            format!(" Lock v{} on task {}.", version, task)
        }
        _ => String::new(),
    };
    let receipt = state
        .store
        .record_hive_coordination_receipt(&HiveCoordinationReceiptRequest {
            kind: "queen_handoff".to_string(),
            actor_session: req.queen_session.clone(),
            actor_agent: Some("dashboard".to_string()),
            target_session: Some(target_session.clone()),
            task_id: effective_task_id.clone(),
            scope: Some(scope.clone()),
            project: req.project.clone(),
            namespace: req.namespace.clone(),
            workspace: req.workspace.clone(),
            summary: format!(
                "Queen handed off scope {} to session {}.{lock_fragment}",
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
        return Err(MemdError::validation("queen_session", "must not be empty").into_wire());
    }
    req.target_session
        .as_deref()
        .and_then(|value| {
            let value = value.trim();
            if value.is_empty() { None } else { Some(value) }
        })
        .map(str::to_string)
        .ok_or_else(|| MemdError::validation("target_session", "must not be empty").into_wire())
}

pub(crate) async fn post_hive_task_upsert(
    State(state): State<AppState>,
    Json(req): Json<HiveTaskUpsertRequest>,
) -> Result<Json<HiveTasksResponse>, (StatusCode, String)> {
    if req.task_id.trim().is_empty() {
        return Err(MemdError::validation("task_id", "must not be empty").into_wire());
    }
    if req.title.trim().is_empty() {
        return Err(MemdError::validation("title", "must not be empty").into_wire());
    }
    let response = state.store.upsert_hive_task(&req).map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_hive_task_assign(
    State(state): State<AppState>,
    Json(req): Json<HiveTaskAssignRequest>,
) -> Result<Json<HiveTasksResponse>, (StatusCode, String)> {
    if req.task_id.trim().is_empty() {
        return Err(MemdError::validation("task_id", "must not be empty").into_wire());
    }
    if req.to_session.trim().is_empty() {
        return Err(MemdError::validation("to_session", "must not be empty").into_wire());
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
    let (scanned, updated, events, metrics) =
        state.store.decay_entities(&req).map_err(internal_error)?;
    Ok(Json(MemoryDecayResponse {
        scanned,
        updated,
        events,
        metrics: Some(metrics),
    }))
}

pub(crate) async fn decay_diagnostics(
    State(state): State<AppState>,
    Json(req): Json<MemoryDecayRequest>,
) -> Result<Json<DecayDiagnosticsResponse>, (StatusCode, String)> {
    let inactive_days = req.inactive_days.unwrap_or(21).max(1) as usize;
    let max_decay = req.max_decay.unwrap_or(0.12).clamp(0.01, 0.5);
    let decay_divisor = req.decay_divisor.unwrap_or(14.0).max(1.0);
    let max_items = req.max_items.unwrap_or(128).min(1_000);
    let metrics = state
        .store
        .decay_diagnostics(&req)
        .map_err(internal_error)?;
    Ok(Json(DecayDiagnosticsResponse {
        metrics,
        inactive_days,
        max_decay,
        decay_divisor,
        max_items,
    }))
}

/// Token efficiency diagnostics — computes per-kind character breakdown for
/// the working memory of a given project/namespace/agent context.
pub(crate) async fn token_efficiency_diagnostics(
    State(state): State<AppState>,
    Json(req): Json<WorkingMemoryRequest>,
) -> Result<Json<memd_schema::OperationTokenReport>, (StatusCode, String)> {
    let response = crate::working::working_memory(&state, req)?;

    // Build the report from compaction quality (already computed in working memory)
    let cq = response
        .compaction_quality
        .as_ref()
        .cloned()
        .unwrap_or_else(|| memd_schema::CompactionQualityReport {
            admitted: response.records.len(),
            evicted: 0,
            per_kind_admitted: Default::default(),
            per_kind_evicted: Default::default(),
            chars_per_kind_admitted: Default::default(),
            budget_chars: response.budget_chars,
            used_chars: response.used_chars,
        });

    let utilization_pct = if cq.budget_chars > 0 {
        (cq.used_chars as f64 / cq.budget_chars as f64) * 100.0
    } else {
        0.0
    };

    Ok(Json(memd_schema::OperationTokenReport {
        operation: "working_memory".to_string(),
        budget_chars: cq.budget_chars,
        used_chars: cq.used_chars,
        utilization_pct,
        per_kind: memd_schema::PerKindTokenMetrics {
            chars_per_kind: cq.chars_per_kind_admitted,
            items_per_kind: cq.per_kind_admitted,
            total_chars: cq.used_chars,
            total_items: cq.admitted,
        },
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

pub(crate) async fn drain_memory(
    State(state): State<AppState>,
    Json(req): Json<MemoryDrainRequest>,
) -> Result<Json<MemoryDrainResponse>, (StatusCode, String)> {
    let max_items = req.max_items.unwrap_or(500).min(5000);
    let deleted = state
        .store
        .drain_expired(req.project.as_deref(), req.namespace.as_deref(), max_items)
        .map_err(internal_error)?;
    Ok(Json(MemoryDrainResponse { deleted }))
}

pub(crate) async fn dismiss_inbox(
    State(state): State<AppState>,
    Json(req): Json<InboxDismissRequest>,
) -> Result<Json<InboxDismissResponse>, (StatusCode, String)> {
    if req.ids.is_empty() {
        return Err(MemdError::validation("ids", "must not be empty").into_wire());
    }
    if req.ids.len() > 100 {
        return Err(MemdError::validation("ids", "max 100 items per dismiss").into_wire());
    }
    let dismissed = state
        .store
        .dismiss_items(&req.ids)
        .map_err(internal_error)?;
    Ok(Json(InboxDismissResponse { dismissed }))
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
        let mut quality_scores: Vec<memd_schema::ConsolidationQualityScore> = Vec::new();

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

            // Inherit the most restrictive visibility from source items.
            // Private < Workspace < Public — min() gives the strictest.
            let inherited_visibility = self
                .store
                .items_for_entity(candidate.entity.id)
                .unwrap_or_default()
                .iter()
                .map(|item| item.visibility)
                .min()
                .unwrap_or(MemoryVisibility::Workspace);

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
                    visibility: Some(inherited_visibility),
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
                    lane: None,
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
            let quality = score_consolidation_quality(
                &candidate.entity,
                &item,
                inherited_visibility,
                candidate.event_count,
            );
            quality_scores.push(quality);
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

        let mean_quality = if quality_scores.is_empty() {
            None
        } else {
            Some(
                quality_scores.iter().map(|q| q.overall).sum::<f32>() / quality_scores.len() as f32,
            )
        };
        Ok(MemoryConsolidationResponse {
            scanned,
            groups,
            consolidated,
            duplicates,
            events,
            highlights,
            mean_quality,
            quality_scores,
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
        let mut report = self.store.maintain_runtime(req)?;
        // E2: backfill entity links on full/repair maintain
        let mode = req.mode.trim();
        let mode = if mode.is_empty() { "scan" } else { mode };
        if req.apply && (mode == "full" || mode == "repair") {
            let linked = self.backfill_entity_links().unwrap_or(0);
            if linked > 0 {
                report
                    .findings
                    .push(format!("entity_links: backfilled {linked} links"));
            }
        }
        Ok(report)
    }

    /// Re-run auto_link_entity for each entity to backfill missing links.
    fn backfill_entity_links(&self) -> anyhow::Result<usize> {
        let entities = self.store.list_entities()?;
        let mut created = 0usize;
        for entity in &entities {
            // Find one representative item for this entity to get project context
            let items = self.store.items_for_entity(entity.id)?;
            let Some(item) = items.first() else {
                continue;
            };
            let before = self
                .store
                .links_for_entity(&memd_schema::EntityLinksRequest {
                    entity_id: entity.id,
                })?
                .len();
            self.auto_link_entity(entity, item)?;
            let after = self
                .store
                .links_for_entity(&memd_schema::EntityLinksRequest {
                    entity_id: entity.id,
                })?
                .len();
            created += after.saturating_sub(before);
        }
        Ok(created)
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
    let mut response = state
        .store
        .list_atlas_regions(&req)
        .map_err(internal_error)?;
    if response.regions.is_empty() {
        let generated = state
            .store
            .generate_regions_for_project(
                req.project.as_deref(),
                req.namespace.as_deref(),
                req.lane.as_deref(),
            )
            .map_err(internal_error)?;
        let limit = req.limit.unwrap_or(generated.len());
        response.regions = generated.into_iter().take(limit).collect();
    }
    Ok(Json(response))
}

pub(crate) async fn post_atlas_explore(
    State(state): State<AppState>,
    Json(req): Json<AtlasExploreRequest>,
) -> Result<Json<AtlasExploreResponse>, (StatusCode, String)> {
    let response = state.store.explore_atlas(&req).map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_atlas_trail_save(
    State(state): State<AppState>,
    Json(req): Json<AtlasSaveTrailRequest>,
) -> Result<Json<AtlasSaveTrailResponse>, (StatusCode, String)> {
    let response = state.store.save_atlas_trail(&req).map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn get_atlas_trails(
    State(state): State<AppState>,
    Query(req): Query<AtlasListTrailsRequest>,
) -> Result<Json<AtlasListTrailsResponse>, (StatusCode, String)> {
    let response = state
        .store
        .list_atlas_trails(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_atlas_rename(
    State(state): State<AppState>,
    Json(req): Json<AtlasRenameRegionRequest>,
) -> Result<Json<AtlasRenameRegionResponse>, (StatusCode, String)> {
    let response = state
        .store
        .rename_atlas_region(&req)
        .map_err(internal_error)?;
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

// ---------------------------------------------------------------------------
// Procedural memory routes (Phase G)
// ---------------------------------------------------------------------------

pub(crate) async fn get_procedures(
    State(state): State<AppState>,
    Query(req): Query<ProcedureListRequest>,
) -> Result<Json<ProcedureListResponse>, (StatusCode, String)> {
    let response = state.store.list_procedures(&req).map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_procedure_record(
    State(state): State<AppState>,
    Json(req): Json<ProcedureRecordRequest>,
) -> Result<Json<ProcedureRecordResponse>, (StatusCode, String)> {
    let response = state.store.record_procedure(&req).map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_procedure_match(
    State(state): State<AppState>,
    Json(req): Json<ProcedureMatchRequest>,
) -> Result<Json<ProcedureMatchResponse>, (StatusCode, String)> {
    let response = state.store.match_procedures(&req).map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_procedure_promote(
    State(state): State<AppState>,
    Json(req): Json<ProcedurePromoteRequest>,
) -> Result<Json<ProcedurePromoteResponse>, (StatusCode, String)> {
    let response = state
        .store
        .promote_procedure(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_procedure_use(
    State(state): State<AppState>,
    Json(req): Json<ProcedureUseRequest>,
) -> Result<Json<ProcedureUseResponse>, (StatusCode, String)> {
    let response = state.store.use_procedure(&req).map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_procedure_retire(
    State(state): State<AppState>,
    Json(req): Json<ProcedureRetireRequest>,
) -> Result<Json<ProcedureRetireResponse>, (StatusCode, String)> {
    let response = state.store.retire_procedure(&req).map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_procedure_detect(
    State(state): State<AppState>,
    Json(req): Json<ProcedureDetectRequest>,
) -> Result<Json<ProcedureDetectResponse>, (StatusCode, String)> {
    let response = state
        .store
        .detect_procedures(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

pub(crate) async fn post_ingest_lanes(
    State(state): State<AppState>,
    Json(req): Json<IngestLanesRequest>,
) -> Result<Json<IngestLanesResponse>, (StatusCode, String)> {
    let root = std::path::Path::new(&req.root);
    if !root.is_dir() {
        return Err(
            MemdError::validation("root", format!("is not a directory: {}", req.root)).into_wire(),
        );
    }
    let summary = crate::store_ingestion::ingest_lane_files(
        &state,
        root,
        req.project.as_deref(),
        req.namespace.as_deref(),
    )
    .map_err(internal_error)?;
    Ok(Json(IngestLanesResponse {
        files_scanned: summary.files_scanned,
        files_ingested: summary.files_ingested,
        files_skipped: summary.files_skipped,
        files_stale: summary.files_stale,
    }))
}

pub(crate) async fn consolidate_episodes_handler(
    State(state): State<AppState>,
    Json(req): Json<ConsolidateEpisodesRequest>,
) -> Result<Json<ConsolidateEpisodesResponse>, (StatusCode, String)> {
    let now = chrono::Utc::now();
    state
        .store
        .consolidate_episodes(&req, now)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub(crate) async fn list_episodes_handler(
    State(state): State<AppState>,
    Query(req): Query<ListEpisodesRequest>,
) -> Result<Json<ListEpisodesResponse>, (StatusCode, String)> {
    state
        .store
        .list_episodes(&req)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub(crate) async fn dedup_scan_handler(
    State(state): State<AppState>,
    Json(req): Json<DedupScanRequest>,
) -> Result<Json<DedupScanResponse>, (StatusCode, String)> {
    let model = state
        .embedder
        .as_deref()
        .map(|e| e.model_code().to_string())
        .ok_or_else(|| {
            (
                StatusCode::PRECONDITION_FAILED,
                "embedder not configured; dedup scan unavailable".to_string(),
            )
        })?;
    state
        .store
        .scan_duplicates(&req, &model)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[derive(Clone)]
pub(crate) struct MemoryViewItem {
    pub(crate) item: MemoryItem,
    pub(crate) entity: Option<MemoryEntityRecord>,
    pub(crate) source_trust_score: f32,
}

#[cfg(test)]
#[path = "routes_tests.rs"]
mod search_fabric_tests;
