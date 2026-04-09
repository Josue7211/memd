mod inspection;
mod keys;
mod repair;
mod routing;
mod store;
mod ui;
mod working;

use std::collections::{HashSet, VecDeque};

use anyhow::Context;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::Html,
    routing::{get, post},
};
use chrono::Utc;
use keys::{apply_lifecycle, canonical_key, redundancy_key, validate_source_quality};
use memd_schema::{
    AgentProfileRequest, AgentProfileResponse, AgentProfileUpsertRequest, AssociativeRecallHit,
    AssociativeRecallRequest, AssociativeRecallResponse, CandidateMemoryRequest,
    CandidateMemoryResponse, CompactContextResponse, CompactMemoryRecord, ContextRequest,
    ContextResponse, EntityLinkRequest, EntityLinkResponse, EntityLinksRequest,
    EntityLinksResponse, EntityMemoryRequest, EntityMemoryResponse, EntitySearchHit,
    EntitySearchRequest, EntitySearchResponse, ExpireMemoryRequest, ExpireMemoryResponse,
    ExplainMemoryRequest, ExplainMemoryResponse, HealthResponse, HiveBoardRequest,
    HiveBoardResponse, HiveClaimAcquireRequest, HiveClaimRecoverRequest, HiveClaimReleaseRequest,
    HiveClaimTransferRequest, HiveClaimsRequest, HiveClaimsResponse, HiveCoordinationInboxRequest,
    HiveCoordinationInboxResponse, HiveCoordinationReceiptRequest, HiveCoordinationReceiptsRequest,
    HiveCoordinationReceiptsResponse, HiveFollowRequest, HiveFollowResponse, HiveMessageAckRequest,
    HiveMessageInboxRequest, HiveMessageSendRequest, HiveMessagesResponse, HiveRosterRequest,
    HiveRosterResponse, HiveSessionAutoRetireRequest, HiveSessionAutoRetireResponse,
    HiveSessionRetireRequest, HiveSessionRetireResponse, HiveSessionUpsertRequest,
    HiveSessionsRequest, HiveSessionsResponse, HiveTaskAssignRequest, HiveTaskUpsertRequest,
    HiveTasksRequest, HiveTasksResponse, InboxMemoryItem, MaintainReport, MaintainReportRequest,
    MemoryConsolidationRequest, MemoryConsolidationResponse, MemoryContextFrame,
    MemoryDecayRequest, MemoryDecayResponse, MemoryEntityLinkRecord, MemoryEntityRecord,
    MemoryEventRecord, MemoryInboxRequest, MemoryInboxResponse, MemoryItem, MemoryKind,
    MemoryMaintenanceReportRequest, MemoryMaintenanceReportResponse, MemoryPolicyResponse,
    MemoryScope, MemoryStage, MemoryStatus, MemoryVisibility, PromoteMemoryRequest,
    PromoteMemoryResponse, RepairMemoryRequest, RepairMemoryResponse, RetrievalIntent,
    RetrievalRoute, SearchMemoryRequest, SearchMemoryResponse, SkillPolicyActivationEntriesRequest,
    SkillPolicyActivationEntriesResponse, SkillPolicyApplyReceiptsRequest,
    SkillPolicyApplyReceiptsResponse, SkillPolicyApplyRequest, SkillPolicyApplyResponse,
    SourceMemoryRequest, SourceMemoryResponse, SourceQuality, StoreMemoryRequest,
    StoreMemoryResponse, TimelineMemoryRequest, TimelineMemoryResponse, VerifyMemoryRequest,
    VerifyMemoryResponse, VisibleMemoryArtifactDetailResponse, VisibleMemorySnapshotResponse,
    VisibleMemoryUiActionRequest, VisibleMemoryUiActionResponse, WorkingMemoryRequest,
    WorkingMemoryResponse, WorkspaceMemoryRequest, WorkspaceMemoryResponse,
};
use routing::RetrievalPlan;
use serde::Deserialize;
use store::{DuplicateMatch, RecordEventArgs, SqliteStore};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    store: SqliteStore,
}

impl AppState {
    fn store_item(
        &self,
        req: StoreMemoryRequest,
        stage: MemoryStage,
    ) -> anyhow::Result<(MemoryItem, Option<DuplicateMatch>)> {
        validate_source_quality(req.source_quality)?;
        let now = Utc::now();
        let item = MemoryItem {
            id: Uuid::new_v4(),
            content: req.content.trim().to_string(),
            redundancy_key: None,
            belief_branch: req.belief_branch,
            preferred: false,
            kind: req.kind,
            scope: req.scope,
            project: req.project,
            namespace: req.namespace,
            workspace: req.workspace,
            visibility: req.visibility.unwrap_or_default(),
            source_agent: req.source_agent,
            source_system: req.source_system,
            source_path: req.source_path,
            confidence: req.confidence.unwrap_or(0.7),
            ttl_seconds: req.ttl_seconds,
            created_at: now,
            updated_at: now,
            last_verified_at: req.last_verified_at,
            supersedes: req.supersedes,
            tags: req.tags,
            status: req.status.unwrap_or(MemoryStatus::Active),
            source_quality: req.source_quality.or(Some(SourceQuality::Canonical)),
            stage,
        };

        let canonical_key = canonical_key(&item);
        let redundancy_key = redundancy_key(&item);
        let item = MemoryItem {
            redundancy_key: Some(redundancy_key.clone()),
            ..item
        };
        let _entity = self.store.resolve_entity_for_item(&item, &canonical_key)?;
        let duplicate =
            self.store
                .insert_or_get_duplicate(&item, &canonical_key, &redundancy_key)?;
        if let Some(found) = duplicate.as_ref() {
            if let Some(revived) = self.revive_duplicate_on_explicit_store(
                found,
                &item,
                &canonical_key,
                &redundancy_key,
            )? {
                let _ = self.record_item_event(
                    &revived,
                    "restored",
                    "duplicate memory item restored to active canonical state".to_string(),
                );
                return Ok((revived, None));
            }
        }
        if duplicate.is_none() {
            let _ = self.record_item_event(
                &item,
                event_type_for_stage(stage),
                format!(
                    "{} memory item stored",
                    match stage {
                        MemoryStage::Candidate => "candidate",
                        MemoryStage::Canonical => "canonical",
                    }
                ),
            );
        }
        Ok((item, duplicate))
    }

    fn revive_duplicate_on_explicit_store(
        &self,
        duplicate: &DuplicateMatch,
        incoming: &MemoryItem,
        canonical_key: &str,
        redundancy_key: &str,
    ) -> anyhow::Result<Option<MemoryItem>> {
        if incoming.stage != MemoryStage::Canonical || incoming.status != MemoryStatus::Active {
            return Ok(None);
        }
        if duplicate.item.status == MemoryStatus::Active {
            return Ok(None);
        }

        let mut revived = duplicate.item.clone();
        revived.content = incoming.content.clone();
        revived.belief_branch = incoming.belief_branch.clone();
        revived.project = incoming.project.clone();
        revived.namespace = incoming.namespace.clone();
        revived.workspace = incoming.workspace.clone();
        revived.visibility = incoming.visibility;
        revived.source_agent = incoming.source_agent.clone();
        revived.source_system = incoming.source_system.clone();
        revived.source_path = incoming.source_path.clone();
        revived.source_quality = incoming.source_quality;
        revived.confidence = incoming.confidence;
        revived.ttl_seconds = incoming.ttl_seconds;
        revived.last_verified_at = incoming.last_verified_at;
        revived.tags = incoming.tags.clone();
        revived.status = MemoryStatus::Active;
        revived.updated_at = Utc::now();
        revived.supersedes.extend(incoming.supersedes.clone());
        revived.supersedes.retain(|id| *id != revived.id);
        revived.supersedes.sort_unstable();
        revived.supersedes.dedup();
        let revived = MemoryItem {
            redundancy_key: Some(redundancy_key.to_string()),
            ..revived
        };
        self.store.update(&revived, canonical_key, redundancy_key)?;
        Ok(Some(revived))
    }

    fn snapshot(&self) -> anyhow::Result<Vec<MemoryItem>> {
        let items = self.store.list()?;
        let mut hydrated = Vec::with_capacity(items.len());
        for item in items {
            let (item, changed) = apply_lifecycle(item);
            if changed {
                let canonical_key = canonical_key(&item);
                let redundancy_key = redundancy_key(&item);
                self.store.update(&item, &canonical_key, &redundancy_key)?;
            }
            hydrated.push(item);
        }
        Ok(hydrated)
    }

    fn promote_item(
        &self,
        req: PromoteMemoryRequest,
    ) -> anyhow::Result<(MemoryItem, Option<DuplicateMatch>)> {
        let mut item = self
            .store
            .get(req.id)?
            .ok_or_else(|| anyhow::anyhow!("memory item not found"))?;

        item.scope = req.scope.unwrap_or(item.scope);
        item.project = req.project.or(item.project);
        item.namespace = req.namespace.or(item.namespace);
        item.workspace = req.workspace.or(item.workspace);
        item.visibility = req.visibility.unwrap_or(item.visibility);
        item.belief_branch = req.belief_branch.or(item.belief_branch);
        item.confidence = req.confidence.unwrap_or(item.confidence);
        item.ttl_seconds = req.ttl_seconds.or(item.ttl_seconds);
        if let Some(tags) = req.tags {
            item.tags = tags;
        }
        item.status = req.status.unwrap_or(MemoryStatus::Active);
        item.stage = MemoryStage::Canonical;
        item.updated_at = Utc::now();

        let canonical_key = canonical_key(&item);
        let redundancy_key_value = redundancy_key(&item);
        if let Some(duplicate) = self.store.find_duplicate(
            &redundancy_key_value,
            &canonical_key,
            &item.stage,
            item.id,
        )? {
            return Ok((item, Some(duplicate)));
        }

        let item = MemoryItem {
            redundancy_key: Some(redundancy_key_value),
            ..item
        };
        let redundancy_key_value = redundancy_key(&item);
        self.store
            .update(&item, &canonical_key, &redundancy_key_value)?;
        let _ = self.record_item_event(
            &item,
            "promoted",
            "memory item promoted to canonical stage".to_string(),
        );
        Ok((item, None))
    }

    fn record_item_event(
        &self,
        item: &MemoryItem,
        event_type: &str,
        summary: String,
    ) -> anyhow::Result<MemoryEventRecord> {
        let canonical_key = canonical_key(item);
        let entity = self.store.resolve_entity_for_item(item, &canonical_key)?;
        let context = Some(entity_context_frame(&entity.record, item));
        self.store.record_event(
            &entity.record,
            item.id,
            RecordEventArgs {
                event_type: event_type.to_string(),
                summary,
                occurred_at: item.updated_at,
                project: item.project.clone(),
                namespace: item.namespace.clone(),
                workspace: item.workspace.clone(),
                source_agent: item.source_agent.clone(),
                source_system: item.source_system.clone(),
                source_path: item.source_path.clone(),
                related_entity_ids: Vec::new(),
                tags: item.tags.clone(),
                context,
                confidence: item.confidence,
                salience_score: entity.record.salience_score,
            },
        )
    }
}

#[tokio::main]
async fn main() {
    let db_path = std::env::var("MEMD_DB_PATH").unwrap_or_else(|_| "memd.db".to_string());
    let bind_addr =
        std::env::var("MEMD_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8787".to_string());
    let state = AppState {
        store: SqliteStore::open(&db_path).expect("open memd sqlite store"),
    };
    let app = Router::new()
        .route("/", get(dashboard))
        .route("/ui/snapshot", get(get_visible_memory_snapshot))
        .route("/ui/artifact", get(get_visible_memory_artifact))
        .route("/ui/action", post(post_visible_memory_action))
        .route("/healthz", get(healthz))
        .route("/memory/store", post(store_memory))
        .route("/memory/candidates", post(store_candidate))
        .route("/memory/promote", post(promote_memory))
        .route("/memory/expire", post(expire_memory))
        .route("/memory/verify", post(verify_memory))
        .route("/memory/repair", post(repair_memory))
        .route("/memory/search", post(search_memory))
        .route("/memory/context", get(get_context))
        .route("/memory/context/compact", get(get_compact_context))
        .route("/memory/working", get(get_working_memory))
        .route("/memory/inbox", get(get_inbox))
        .route("/memory/entity", get(get_entity))
        .route("/memory/entity/search", get(get_entity_search))
        .route("/memory/entity/link", post(post_entity_link))
        .route("/memory/entity/links", get(get_entity_links))
        .route("/memory/entity/recall", get(get_entity_recall))
        .route("/memory/timeline", get(get_timeline))
        .route(
            "/memory/profile",
            get(get_agent_profile).post(post_agent_profile),
        )
        .route("/memory/source", get(get_source_memory))
        .route("/memory/workspaces", get(get_workspace_memory))
        .route("/memory/explain", get(get_explain))
        .route("/coordination/messages/send", post(post_hive_message))
        .route("/coordination/messages/inbox", get(get_hive_inbox))
        .route("/coordination/messages/ack", post(post_hive_ack))
        .route("/coordination/inbox", get(get_hive_coordination_inbox))
        .route(
            "/coordination/receipts/record",
            post(post_hive_coordination_receipt),
        )
        .route(
            "/coordination/receipts",
            get(get_hive_coordination_receipts),
        )
        .route(
            "/coordination/skill-policy/apply",
            post(post_skill_policy_apply_receipt).get(get_skill_policy_apply_receipts),
        )
        .route(
            "/coordination/skill-policy/activations",
            get(get_skill_policy_activations),
        )
        .route(
            "/coordination/claims/acquire",
            post(post_hive_claim_acquire),
        )
        .route(
            "/coordination/claims/release",
            post(post_hive_claim_release),
        )
        .route(
            "/coordination/claims/transfer",
            post(post_hive_claim_transfer),
        )
        .route(
            "/coordination/claims/recover",
            post(post_hive_claim_recover),
        )
        .route("/coordination/claims", get(get_hive_claims))
        .route(
            "/coordination/sessions/upsert",
            post(post_hive_session_upsert),
        )
        .route(
            "/coordination/sessions/retire",
            post(post_hive_session_retire),
        )
        .route(
            "/coordination/sessions/auto-retire",
            post(post_hive_session_auto_retire),
        )
        .route("/coordination/sessions", get(get_hive_sessions))
        .route("/hive/board", get(get_hive_board))
        .route("/hive/roster", get(get_hive_roster))
        .route("/hive/follow", get(get_hive_follow))
        .route("/coordination/tasks/upsert", post(post_hive_task_upsert))
        .route("/coordination/tasks/assign", post(post_hive_task_assign))
        .route("/coordination/tasks", get(get_hive_tasks))
        .route("/memory/maintenance/decay", post(decay_memory))
        .route("/memory/maintenance/consolidate", post(consolidate_memory))
        .route("/memory/maintenance/report", get(get_maintenance_report))
        .route("/runtime/maintain", post(post_runtime_maintain))
        .route("/memory/policy", get(get_memory_policy))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|_| panic!("bind memd to {}", bind_addr));
    axum::serve(listener, app).await.expect("serve memd");
}

async fn healthz(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        items: state.store.count().unwrap_or(0),
    })
}

async fn dashboard(State(state): State<AppState>) -> Result<Html<String>, (StatusCode, String)> {
    let snapshot = ui::build_visible_memory_snapshot(&state).map_err(internal_error)?;
    Ok(Html(dashboard_html(&snapshot)))
}

async fn get_visible_memory_snapshot(
    State(state): State<AppState>,
) -> Result<Json<VisibleMemorySnapshotResponse>, (StatusCode, String)> {
    let response = ui::build_visible_memory_snapshot(&state).map_err(internal_error)?;
    Ok(Json(response))
}

#[derive(Deserialize)]
struct VisibleMemoryArtifactQuery {
    id: Uuid,
}

async fn get_visible_memory_artifact(
    State(state): State<AppState>,
    Query(req): Query<VisibleMemoryArtifactQuery>,
) -> Result<Json<VisibleMemoryArtifactDetailResponse>, (StatusCode, String)> {
    let response = ui::build_visible_memory_artifact_detail(&state, req.id)?;
    Ok(Json(response))
}

async fn post_visible_memory_action(
    State(state): State<AppState>,
    Json(req): Json<VisibleMemoryUiActionRequest>,
) -> Result<Json<VisibleMemoryUiActionResponse>, (StatusCode, String)> {
    let response = ui::perform_visible_memory_action(&state, req)?;
    Ok(Json(response))
}

async fn store_memory(
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

async fn store_candidate(
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

async fn promote_memory(
    State(state): State<AppState>,
    Json(req): Json<PromoteMemoryRequest>,
) -> Result<Json<PromoteMemoryResponse>, (StatusCode, String)> {
    let (item, duplicate) = state.promote_item(req).map_err(internal_error)?;
    Ok(Json(PromoteMemoryResponse {
        item: duplicate.as_ref().map_or(item, |found| found.item.clone()),
        duplicate_of: duplicate.map(|found| found.id),
    }))
}

async fn expire_memory(
    State(state): State<AppState>,
    Json(req): Json<ExpireMemoryRequest>,
) -> Result<Json<ExpireMemoryResponse>, (StatusCode, String)> {
    let item = repair::expire_item(&state, req)?;
    Ok(Json(ExpireMemoryResponse { item }))
}

async fn verify_memory(
    State(state): State<AppState>,
    Json(req): Json<VerifyMemoryRequest>,
) -> Result<Json<VerifyMemoryResponse>, (StatusCode, String)> {
    let item = repair::verify_item(&state, req)?;
    Ok(Json(VerifyMemoryResponse { item }))
}

async fn repair_memory(
    State(state): State<AppState>,
    Json(req): Json<RepairMemoryRequest>,
) -> Result<Json<RepairMemoryResponse>, (StatusCode, String)> {
    let response = repair::repair_item(&state, req)?;
    Ok(Json(response))
}

async fn get_working_memory(
    State(state): State<AppState>,
    Query(req): Query<WorkingMemoryRequest>,
) -> Result<Json<WorkingMemoryResponse>, (StatusCode, String)> {
    let response = working::working_memory(&state, req)?;
    Ok(Json(response))
}

async fn get_explain(
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

async fn get_memory_policy() -> Json<MemoryPolicyResponse> {
    Json(working::memory_policy_snapshot())
}

async fn search_memory(
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

async fn get_context(
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

async fn get_compact_context(
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

async fn get_inbox(
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
        inbox_score(&b.item, b.entity.as_ref(), &plan)
            .partial_cmp(&inbox_score(&a.item, a.entity.as_ref(), &plan))
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

async fn get_entity(
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

async fn get_entity_search(
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

async fn post_entity_link(
    State(state): State<AppState>,
    Json(req): Json<EntityLinkRequest>,
) -> Result<Json<EntityLinkResponse>, (StatusCode, String)> {
    let link = state.store.link_entity(&req).map_err(internal_error)?;
    Ok(Json(EntityLinkResponse { link }))
}

async fn get_entity_links(
    State(state): State<AppState>,
    Query(req): Query<EntityLinksRequest>,
) -> Result<Json<EntityLinksResponse>, (StatusCode, String)> {
    let links = state.store.links_for_entity(&req).map_err(internal_error)?;
    Ok(Json(EntityLinksResponse {
        entity_id: req.entity_id,
        links,
    }))
}

async fn get_entity_recall(
    State(state): State<AppState>,
    Query(req): Query<AssociativeRecallRequest>,
) -> Result<Json<AssociativeRecallResponse>, (StatusCode, String)> {
    let response = state.associative_recall(&req).map_err(internal_error)?;
    Ok(Json(response))
}

async fn get_agent_profile(
    State(state): State<AppState>,
    Query(req): Query<AgentProfileRequest>,
) -> Result<Json<AgentProfileResponse>, (StatusCode, String)> {
    let profile = state.store.agent_profile(&req).map_err(internal_error)?;
    Ok(Json(AgentProfileResponse { profile }))
}

async fn post_agent_profile(
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

async fn get_source_memory(
    State(state): State<AppState>,
    Query(req): Query<SourceMemoryRequest>,
) -> Result<Json<SourceMemoryResponse>, (StatusCode, String)> {
    let response = state.store.source_memory(&req).map_err(internal_error)?;
    Ok(Json(response))
}

async fn get_workspace_memory(
    State(state): State<AppState>,
    Query(req): Query<WorkspaceMemoryRequest>,
) -> Result<Json<WorkspaceMemoryResponse>, (StatusCode, String)> {
    let response = state.store.workspace_memory(&req).map_err(internal_error)?;
    Ok(Json(response))
}

async fn post_hive_message(
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

async fn get_hive_inbox(
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

async fn post_hive_ack(
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

async fn get_hive_coordination_inbox(
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

async fn post_hive_coordination_receipt(
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

async fn get_hive_coordination_receipts(
    State(state): State<AppState>,
    Query(req): Query<HiveCoordinationReceiptsRequest>,
) -> Result<Json<HiveCoordinationReceiptsResponse>, (StatusCode, String)> {
    let response = state
        .store
        .hive_coordination_receipts(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

async fn post_skill_policy_apply_receipt(
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

async fn get_skill_policy_apply_receipts(
    State(state): State<AppState>,
    Query(req): Query<SkillPolicyApplyReceiptsRequest>,
) -> Result<Json<SkillPolicyApplyReceiptsResponse>, (StatusCode, String)> {
    let response = state
        .store
        .skill_policy_apply_receipts(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

async fn get_skill_policy_activations(
    State(state): State<AppState>,
    Query(req): Query<SkillPolicyActivationEntriesRequest>,
) -> Result<Json<SkillPolicyActivationEntriesResponse>, (StatusCode, String)> {
    let response = state
        .store
        .skill_policy_activations(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

async fn post_hive_claim_acquire(
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

async fn post_hive_claim_release(
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

async fn post_hive_claim_transfer(
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

async fn post_hive_claim_recover(
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

async fn get_hive_claims(
    State(state): State<AppState>,
    Query(req): Query<HiveClaimsRequest>,
) -> Result<Json<HiveClaimsResponse>, (StatusCode, String)> {
    let response = state.store.hive_claims(&req).map_err(internal_error)?;
    Ok(Json(response))
}

async fn post_hive_session_upsert(
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

async fn post_hive_session_retire(
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

async fn post_hive_session_auto_retire(
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

async fn get_hive_sessions(
    State(state): State<AppState>,
    Query(req): Query<HiveSessionsRequest>,
) -> Result<Json<HiveSessionsResponse>, (StatusCode, String)> {
    let response = state.store.hive_sessions(&req).map_err(internal_error)?;
    Ok(Json(response))
}

async fn get_hive_board(
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

async fn get_hive_roster(
    State(state): State<AppState>,
    Query(req): Query<HiveRosterRequest>,
) -> Result<Json<HiveRosterResponse>, (StatusCode, String)> {
    let response = state.store.hive_roster(&req).map_err(internal_error)?;
    Ok(Json(response))
}

async fn get_hive_follow(
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

async fn post_hive_task_upsert(
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

async fn post_hive_task_assign(
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

async fn get_hive_tasks(
    State(state): State<AppState>,
    Query(req): Query<HiveTasksRequest>,
) -> Result<Json<HiveTasksResponse>, (StatusCode, String)> {
    let response = state.store.hive_tasks(&req).map_err(internal_error)?;
    Ok(Json(response))
}

async fn get_timeline(
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

async fn decay_memory(
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

async fn consolidate_memory(
    State(state): State<AppState>,
    Json(req): Json<MemoryConsolidationRequest>,
) -> Result<Json<MemoryConsolidationResponse>, (StatusCode, String)> {
    let response = state
        .consolidate_semantic_memory(&req)
        .map_err(internal_error)?;
    Ok(Json(response))
}

async fn get_maintenance_report(
    State(state): State<AppState>,
    Query(req): Query<MemoryMaintenanceReportRequest>,
) -> Result<Json<MemoryMaintenanceReportResponse>, (StatusCode, String)> {
    let response = state.maintenance_report(&req).map_err(internal_error)?;
    Ok(Json(response))
}

async fn post_runtime_maintain(
    State(state): State<AppState>,
    Json(req): Json<MaintainReportRequest>,
) -> Result<Json<MaintainReport>, (StatusCode, String)> {
    let response = state.maintain_runtime(&req).map_err(internal_error)?;
    Ok(Json(response))
}

impl AppState {
    fn rehearse_items(&self, items: &[MemoryItem], limit: usize) -> anyhow::Result<()> {
        for item in items.iter().take(limit) {
            let canonical_key = canonical_key(item);
            let _ = self
                .store
                .rehearse_entity_for_item(item, &canonical_key, 0.02)?;
        }
        Ok(())
    }

    fn rehearse_item(&self, item_id: Uuid, salience_boost: f32) -> anyhow::Result<()> {
        if let Some(item) = self.store.get(item_id)? {
            let canonical_key = canonical_key(&item);
            let _ = self
                .store
                .rehearse_entity_for_item(&item, &canonical_key, salience_boost)?;
        }
        Ok(())
    }

    fn record_retrieval_feedback(
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

    fn record_retrieval_feedback_for_item(
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

    fn consolidate_semantic_memory(
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

    fn maintenance_report(
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

    fn maintain_runtime(&self, req: &MaintainReportRequest) -> anyhow::Result<MaintainReport> {
        self.store.maintain_runtime(req)
    }

    fn entity_view(
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

    fn associative_recall(
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

#[derive(Clone)]
struct MemoryViewItem {
    item: MemoryItem,
    entity: Option<MemoryEntityRecord>,
    source_trust_score: f32,
}

fn enrich_with_entities(
    state: &AppState,
    items: Vec<MemoryItem>,
) -> anyhow::Result<Vec<MemoryViewItem>> {
    items
        .into_iter()
        .map(|item| {
            let entity = state.store.entity_for_item(item.id)?;
            let source_trust_score = state.store.trust_score_for_item(&item)?;
            Ok(MemoryViewItem {
                item,
                entity,
                source_trust_score,
            })
        })
        .collect()
}

pub(crate) struct BuildContextResult {
    pub plan: RetrievalPlan,
    pub retrieval_order: Vec<MemoryScope>,
    pub items: Vec<MemoryItem>,
}

fn build_context(
    state: &AppState,
    req: &ContextRequest,
) -> Result<BuildContextResult, (StatusCode, String)> {
    let plan = RetrievalPlan::resolve(req.route, req.intent);
    let limit = req.limit.unwrap_or(8).min(32);
    let max_chars = req.max_chars_per_item.unwrap_or(280).clamp(80, 2000);
    let items = enrich_with_entities(state, state.snapshot().map_err(internal_error)?)
        .map_err(internal_error)?;
    let retrieval_order = plan.scopes();

    let mut scoped: Vec<MemoryItem> = Vec::new();
    let mut live_truth: Vec<MemoryViewItem> = items
        .iter()
        .filter(|entry| entry.item.kind == MemoryKind::LiveTruth)
        .filter(|entry| entry.item.status == MemoryStatus::Active)
        .filter(|entry| plan.allows(entry.item.scope))
        .filter(
            |entry| match (&req.project, &entry.item.project, entry.item.scope) {
                (Some(project), Some(item_project), MemoryScope::Project | MemoryScope::Synced) => {
                    item_project == project
                }
                (Some(_), None, MemoryScope::Project | MemoryScope::Synced) => false,
                _ => true,
            },
        )
        .filter(|entry| {
            req.visibility
                .is_none_or(|visibility| entry.item.visibility == visibility)
        })
        .cloned()
        .collect();
    live_truth.sort_by(|a, b| b.item.updated_at.cmp(&a.item.updated_at));

    for entry in live_truth {
        let mut item = entry.item;
        item.content = compact_content(&item.content, max_chars);
        scoped.push(item);
        if scoped.len() >= limit {
            scoped.truncate(limit);
            return Ok(BuildContextResult {
                plan,
                retrieval_order,
                items: scoped,
            });
        }
    }

    let mut ranked_items: Vec<MemoryViewItem> = items
        .iter()
        .filter(|entry| plan.allows(entry.item.scope))
        .filter(|entry| entry.item.kind != MemoryKind::LiveTruth)
        .filter(|entry| entry.item.status == MemoryStatus::Active)
        .filter(
            |entry| match (&req.project, &entry.item.project, entry.item.scope) {
                (Some(project), Some(item_project), MemoryScope::Project) => {
                    item_project == project
                }
                (Some(project), Some(item_project), MemoryScope::Synced) => item_project == project,
                (Some(_), None, MemoryScope::Project | MemoryScope::Synced) => false,
                _ => true,
            },
        )
        .filter(|entry| {
            req.visibility
                .is_none_or(|visibility| entry.item.visibility == visibility)
        })
        .cloned()
        .collect();

    ranked_items.sort_by(|a, b| {
        context_score(&b.item, b.entity.as_ref(), b.source_trust_score, req, &plan)
            .partial_cmp(&context_score(
                &a.item,
                a.entity.as_ref(),
                a.source_trust_score,
                req,
                &plan,
            ))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.item.updated_at.cmp(&a.item.updated_at))
    });

    scoped.extend(ranked_items.into_iter().map(|entry| entry.item));

    for item in &mut scoped {
        item.content = compact_content(&item.content, max_chars);
    }
    scoped.truncate(limit);

    Ok(BuildContextResult {
        plan,
        retrieval_order,
        items: scoped,
    })
}

fn apply_agent_profile_defaults(
    state: &AppState,
    mut req: ContextRequest,
) -> anyhow::Result<ContextRequest> {
    let Some(agent) = req.agent.clone() else {
        return Ok(req);
    };

    let profile = state.store.agent_profile(&AgentProfileRequest {
        agent,
        project: req.project.clone(),
        namespace: None,
    })?;
    if let Some(profile) = profile {
        if req.route.is_none() {
            req.route = profile.preferred_route;
        }
        if req.intent.is_none() {
            req.intent = profile.preferred_intent;
        }
        if req.max_chars_per_item.is_none() {
            req.max_chars_per_item = profile.summary_chars;
        }
        if req.limit.is_none() && profile.recall_depth.is_some() {
            req.limit = profile.recall_depth;
        }
    }

    Ok(req)
}

fn filter_items(
    items: &[MemoryViewItem],
    req: &SearchMemoryRequest,
    plan: &RetrievalPlan,
) -> Vec<MemoryItem> {
    let query = req.query.as_ref().map(|q| q.to_ascii_lowercase());
    let limit = req.limit.unwrap_or(10).min(100);
    let max_chars = req.max_chars_per_item.unwrap_or(420).clamp(120, 4000);

    let mut filtered: Vec<MemoryViewItem> = items
        .iter()
        .filter(|entry| req.scopes.is_empty() || req.scopes.contains(&entry.item.scope))
        .filter(|entry| plan.allows(entry.item.scope))
        .filter(|entry| req.kinds.is_empty() || req.kinds.contains(&entry.item.kind))
        .filter(|entry| req.statuses.is_empty() || req.statuses.contains(&entry.item.status))
        .filter(|entry| req.stages.is_empty() || req.stages.contains(&entry.item.stage))
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
        .filter(|entry| {
            req.source_agent
                .as_ref()
                .is_none_or(|agent| entry.item.source_agent.as_ref() == Some(agent))
        })
        .filter(|entry| {
            req.tags.is_empty()
                || req
                    .tags
                    .iter()
                    .all(|tag| entry.item.tags.iter().any(|item_tag| item_tag == tag))
        })
        .filter(|entry| {
            query.as_ref().is_none_or(|query| {
                entry.item.content.to_ascii_lowercase().contains(query)
                    || entry
                        .item
                        .tags
                        .iter()
                        .any(|tag| tag.to_ascii_lowercase().contains(query))
            })
        })
        .cloned()
        .collect();

    filtered.sort_by(|a, b| {
        search_score(
            &b.item,
            b.entity.as_ref(),
            b.source_trust_score,
            &query,
            plan,
        )
        .partial_cmp(&search_score(
            &a.item,
            a.entity.as_ref(),
            a.source_trust_score,
            &query,
            plan,
        ))
        .unwrap_or(std::cmp::Ordering::Equal)
        .then_with(|| {
            b.item
                .confidence
                .partial_cmp(&a.item.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .then_with(|| b.item.updated_at.cmp(&a.item.updated_at))
    });
    for item in &mut filtered {
        item.item.content = compact_content(&item.item.content, max_chars);
    }
    filtered.truncate(limit);
    filtered.into_iter().map(|entry| entry.item).collect()
}

fn compact_content(content: &str, max_chars: usize) -> String {
    let normalized = content.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.chars().count() <= max_chars {
        return normalized;
    }

    let mut compact = String::with_capacity(max_chars + 3);
    for ch in normalized.chars().take(max_chars.saturating_sub(3)) {
        compact.push(ch);
    }
    compact.push_str("...");
    compact
}

fn event_type_for_stage(stage: MemoryStage) -> &'static str {
    match stage {
        MemoryStage::Candidate => "candidate_created",
        MemoryStage::Canonical => "canonical_created",
    }
}

fn entity_context_frame(entity: &MemoryEntityRecord, item: &MemoryItem) -> MemoryContextFrame {
    entity.context.clone().unwrap_or(MemoryContextFrame {
        at: Some(item.updated_at),
        project: item.project.clone(),
        namespace: item.namespace.clone(),
        workspace: item.workspace.clone(),
        repo: item.source_system.clone(),
        host: None,
        branch: None,
        agent: item.source_agent.clone(),
        location: item.source_path.clone(),
    })
}

fn consolidation_content(
    entity: &MemoryEntityRecord,
    event_count: usize,
    first_recorded_at: chrono::DateTime<chrono::Utc>,
    last_recorded_at: chrono::DateTime<chrono::Utc>,
) -> String {
    let state = compact_content(
        entity
            .current_state
            .as_deref()
            .unwrap_or("state unavailable"),
        220,
    );
    let span_days = (last_recorded_at - first_recorded_at).num_days().max(0);
    format!(
        "stable {} state after {} events over {}d: {}",
        entity.entity_type, event_count, span_days, state
    )
}

fn consolidation_scope(entity: &MemoryEntityRecord) -> MemoryScope {
    let context = entity.context.as_ref();
    if context
        .and_then(|context| context.project.as_ref())
        .is_some()
    {
        MemoryScope::Project
    } else if context
        .and_then(|context| context.namespace.as_ref())
        .is_some()
    {
        MemoryScope::Synced
    } else {
        MemoryScope::Local
    }
}

fn consolidation_kind(entity_type: &str) -> MemoryKind {
    match entity_type {
        "fact" => MemoryKind::Fact,
        "decision" => MemoryKind::Decision,
        "preference" => MemoryKind::Preference,
        "runbook" => MemoryKind::Runbook,
        "procedural" => MemoryKind::Procedural,
        "self_model" => MemoryKind::SelfModel,
        "topology" => MemoryKind::Topology,
        "status" => MemoryKind::Status,
        "live_truth" => MemoryKind::LiveTruth,
        "pattern" => MemoryKind::Pattern,
        "constraint" => MemoryKind::Constraint,
        _ => MemoryKind::Pattern,
    }
}

fn consolidation_tags(entity: &MemoryEntityRecord, event_count: usize) -> Vec<String> {
    let mut tags = entity.tags.clone();
    tags.push("consolidated".to_string());
    tags.push(format!("events:{}", event_count));
    tags.push(entity.entity_type.clone());
    tags.sort();
    tags.dedup();
    tags
}

fn internal_error(error: anyhow::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

fn compact_record(item: &MemoryItem) -> String {
    let mut parts = Vec::new();
    parts.push(format!("id={}", item.id));
    parts.push(format!("stage={}", enum_label_stage(item.stage)));
    parts.push(format!("scope={}", enum_label_scope(item.scope)));
    parts.push(format!("kind={}", enum_label_kind(item.kind)));
    parts.push(format!("status={}", enum_label_status(item.status)));

    if let Some(project) = &item.project {
        if !project.is_empty() {
            parts.push(format!("project={}", sanitize_value(project)));
        }
    }
    if let Some(namespace) = &item.namespace {
        if !namespace.is_empty() {
            parts.push(format!("ns={}", sanitize_value(namespace)));
        }
    }
    if let Some(workspace) = &item.workspace {
        if !workspace.is_empty() {
            parts.push(format!("ws={}", sanitize_value(workspace)));
        }
    }
    parts.push(format!("vis={}", enum_label_visibility(item.visibility)));
    if let Some(branch) = &item.belief_branch {
        if !branch.is_empty() {
            parts.push(format!("belief_branch={}", sanitize_value(branch)));
        }
    }
    if let Some(agent) = &item.source_agent {
        if !agent.is_empty() {
            parts.push(format!("agent={}", sanitize_value(agent)));
        }
    }
    if !item.tags.is_empty() {
        let tags = item
            .tags
            .iter()
            .map(|tag| sanitize_value(tag))
            .collect::<Vec<_>>()
            .join(",");
        parts.push(format!("tags={}", tags));
    }
    parts.push(format!("cf={:.2}", item.confidence));
    parts.push(format!("upd={}", item.updated_at.timestamp()));
    parts.push(format!("c={}", sanitize_value(&item.content)));

    parts.join(" | ")
}

fn enum_label_route(route: RetrievalRoute) -> &'static str {
    match route {
        RetrievalRoute::Auto => "auto",
        RetrievalRoute::LocalOnly => "local_only",
        RetrievalRoute::SyncedOnly => "synced_only",
        RetrievalRoute::ProjectOnly => "project_only",
        RetrievalRoute::GlobalOnly => "global_only",
        RetrievalRoute::LocalFirst => "local_first",
        RetrievalRoute::SyncedFirst => "synced_first",
        RetrievalRoute::ProjectFirst => "project_first",
        RetrievalRoute::GlobalFirst => "global_first",
        RetrievalRoute::All => "all",
    }
}

fn enum_label_intent(intent: RetrievalIntent) -> &'static str {
    match intent {
        RetrievalIntent::General => "general",
        RetrievalIntent::CurrentTask => "current_task",
        RetrievalIntent::Decision => "decision",
        RetrievalIntent::Runbook => "runbook",
        RetrievalIntent::Procedural => "procedural",
        RetrievalIntent::SelfModel => "self_model",
        RetrievalIntent::Topology => "topology",
        RetrievalIntent::Preference => "preference",
        RetrievalIntent::Fact => "fact",
        RetrievalIntent::Pattern => "pattern",
    }
}

fn associative_recall_score(
    entity: &MemoryEntityRecord,
    link: &MemoryEntityLinkRecord,
    depth: usize,
    root: &MemoryEntityRecord,
) -> f32 {
    let relation_weight = match link.relation_kind {
        memd_schema::EntityRelationKind::SameAs => 1.0,
        memd_schema::EntityRelationKind::Supersedes => 0.92,
        memd_schema::EntityRelationKind::DerivedFrom => 0.88,
        memd_schema::EntityRelationKind::Related => 0.7,
        memd_schema::EntityRelationKind::Contradicts => 0.62,
    };
    let depth_penalty = 1.0 / (depth as f32 + 1.0);
    let salience = entity.salience_score.clamp(0.0, 1.0);
    let rehearsal = (entity.rehearsal_count as f32).ln_1p().min(3.0) / 3.0;
    let context_bonus = if entity
        .context
        .as_ref()
        .and_then(|context| context.project.as_ref())
        == root
            .context
            .as_ref()
            .and_then(|context| context.project.as_ref())
    {
        0.08
    } else {
        0.0
    };
    ((relation_weight * 0.42)
        + (salience * 0.34)
        + (rehearsal * 0.12)
        + (depth_penalty * 0.08)
        + context_bonus)
        .clamp(0.0, 1.0)
}

fn associative_recall_reasons(
    entity: &MemoryEntityRecord,
    link: &MemoryEntityLinkRecord,
    depth: usize,
) -> Vec<String> {
    let mut reasons = Vec::new();
    reasons.push(format!("{:?}", link.relation_kind).to_lowercase());
    reasons.push(format!("depth={depth}"));
    reasons.push(format!("salience={:.2}", entity.salience_score));
    if entity.rehearsal_count > 1 {
        reasons.push(format!("rehearsal={}", entity.rehearsal_count));
    }
    if !entity.aliases.is_empty() {
        reasons.push(format!("aliases={}", entity.aliases.len()));
    }
    reasons
}

#[allow(dead_code)]
fn legacy_dashboard_html() -> String {
    r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>memd</title>
  <style>
    :root {
      color-scheme: dark;
      --bg: #0b0d10;
      --panel: #11151b;
      --panel-2: #161b23;
      --text: #e7eef8;
      --muted: #93a4ba;
      --line: #243041;
      --accent: #69a8ff;
      --accent-2: #7bf1c8;
      --warn: #ffbd59;
      --bad: #ff6b6b;
    }
    * { box-sizing: border-box; }
    body {
      margin: 0;
      font: 14px/1.5 Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, sans-serif;
      background:
        radial-gradient(circle at top left, rgba(105,168,255,0.12), transparent 32%),
        radial-gradient(circle at top right, rgba(123,241,200,0.10), transparent 28%),
        linear-gradient(180deg, #090b0e, var(--bg));
      color: var(--text);
    }
    header {
      padding: 28px 24px 16px;
      border-bottom: 1px solid var(--line);
      background: rgba(10, 13, 17, 0.92);
      position: sticky;
      top: 0;
      backdrop-filter: blur(14px);
      z-index: 2;
    }
    .shell {
      max-width: 1400px;
      margin: 0 auto;
    }
    h1 {
      margin: 0 0 6px;
      font-size: 28px;
      letter-spacing: -0.02em;
    }
    .sub {
      color: var(--muted);
      margin: 0;
    }
    main {
      max-width: 1400px;
      margin: 0 auto;
      padding: 20px 24px 32px;
      display: grid;
      grid-template-columns: 360px 1fr;
      gap: 18px;
      align-items: start;
    }
    .panel {
      background: linear-gradient(180deg, rgba(17,21,27,0.95), rgba(13,17,22,0.95));
      border: 1px solid var(--line);
      border-radius: 18px;
      box-shadow: 0 24px 60px rgba(0,0,0,0.25);
      overflow: hidden;
    }
    .panel h2 {
      margin: 0;
      padding: 16px 16px 12px;
      border-bottom: 1px solid var(--line);
      font-size: 14px;
      letter-spacing: 0.08em;
      text-transform: uppercase;
      color: var(--muted);
    }
    .content {
      padding: 16px;
    }
    label {
      display: block;
      margin: 0 0 10px;
      color: var(--muted);
      font-size: 12px;
      text-transform: uppercase;
      letter-spacing: 0.08em;
    }
    input, select, textarea, button {
      width: 100%;
      border-radius: 12px;
      border: 1px solid var(--line);
      background: var(--panel-2);
      color: var(--text);
      padding: 11px 12px;
      font: inherit;
    }
    textarea {
      min-height: 120px;
      resize: vertical;
    }
    button {
      cursor: pointer;
      background: linear-gradient(180deg, rgba(105,168,255,0.95), rgba(76,131,245,0.95));
      border: 0;
      font-weight: 650;
    }
    button.secondary {
      background: var(--panel-2);
      border: 1px solid var(--line);
      color: var(--text);
      font-weight: 600;
    }
    .stack {
      display: grid;
      gap: 10px;
    }
    .grid-2 {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 10px;
    }
    .meta {
      display: flex;
      flex-wrap: wrap;
      gap: 8px;
      color: var(--muted);
      font-size: 12px;
      margin-bottom: 12px;
    }
    .pill {
      border: 1px solid var(--line);
      border-radius: 999px;
      padding: 6px 10px;
      background: rgba(255,255,255,0.02);
    }
    pre {
      margin: 0;
      white-space: pre-wrap;
      word-break: break-word;
      color: #dce7f4;
      background: #0b0f14;
      border: 1px solid var(--line);
      border-radius: 14px;
      padding: 14px;
      min-height: 240px;
      max-height: 68vh;
      overflow: auto;
    }
    .section {
      display: grid;
      gap: 12px;
    }
    .toolbar {
      display: flex;
      gap: 8px;
      flex-wrap: wrap;
    }
    .toolbar button {
      width: auto;
      padding: 10px 14px;
    }
    .note {
      color: var(--muted);
      font-size: 12px;
    }
    @media (max-width: 1040px) {
      main { grid-template-columns: 1fr; }
    }
  </style>
</head>
<body>
  <header>
    <div class="shell">
      <h1>memd</h1>
      <p class="sub">Memory manager, retrieval router, inbox, and explain surface.</p>
    </div>
  </header>
  <main>
    <section class="panel">
      <h2>Controls</h2>
      <div class="content stack">
        <div class="grid-2">
          <div>
            <label>Project</label>
            <input id="project" placeholder="demo">
          </div>
          <div>
            <label>Agent</label>
            <input id="agent" placeholder="codex">
          </div>
        </div>
        <div class="grid-2">
          <div>
            <label>Workspace</label>
            <input id="workspace" placeholder="team-alpha">
          </div>
          <div>
            <label>Visibility</label>
            <select id="visibility">
              <option value="">all</option>
              <option value="private">private</option>
              <option value="workspace">workspace</option>
              <option value="public">public</option>
            </select>
          </div>
        </div>
        <div class="grid-2">
          <div>
            <label>Route</label>
            <select id="route">
              <option value="auto">auto</option>
              <option value="local_only">local_only</option>
              <option value="synced_only">synced_only</option>
              <option value="project_only">project_only</option>
              <option value="global_only">global_only</option>
              <option value="local_first">local_first</option>
              <option value="synced_first">synced_first</option>
              <option value="project_first">project_first</option>
              <option value="global_first">global_first</option>
              <option value="all">all</option>
            </select>
          </div>
          <div>
            <label>Intent</label>
            <select id="intent">
              <option value="general">general</option>
              <option value="current_task">current_task</option>
              <option value="decision">decision</option>
              <option value="runbook">runbook</option>
              <option value="procedural">procedural</option>
              <option value="self_model">self_model</option>
              <option value="topology">topology</option>
              <option value="preference">preference</option>
              <option value="fact">fact</option>
              <option value="pattern">pattern</option>
            </select>
          </div>
        </div>
        <div>
          <label>Search query</label>
          <input id="query" placeholder="postgres, routing, memory, etc.">
        </div>
        <div>
          <label>Explain id</label>
          <input id="id" placeholder="UUID">
        </div>
        <div class="toolbar">
          <button onclick="loadHealth()">Refresh health</button>
          <button onclick="loadContext()">Load context</button>
          <button onclick="loadInbox()">Load inbox</button>
          <button onclick="loadSearch()">Search</button>
          <button onclick="loadWorkspaces()">Workspaces</button>
          <button class="secondary" onclick="loadSources()">Sources</button>
          <button class="secondary" onclick="loadExplain()">Explain</button>
        </div>
        <div class="note" id="healthNote">Loading health...</div>
      </div>
    </section>
    <section class="panel">
      <h2>Output</h2>
      <div class="content section">
        <pre id="output">{}</pre>
      </div>
    </section>
  </main>
  <script>
    const output = document.getElementById('output');
    const healthNote = document.getElementById('healthNote');
    const qs = () => ({
      project: document.getElementById('project').value.trim(),
      workspace: document.getElementById('workspace').value.trim(),
      visibility: document.getElementById('visibility').value,
      agent: document.getElementById('agent').value.trim(),
      route: document.getElementById('route').value,
      intent: document.getElementById('intent').value,
      query: document.getElementById('query').value.trim(),
      id: document.getElementById('id').value.trim(),
    });
    function pretty(data) {
      output.textContent = JSON.stringify(data, null, 2);
    }
    async function loadHealth() {
      const res = await fetch('/healthz');
      const data = await res.json();
      healthNote.textContent = `status=${data.status} items=${data.items}`;
      pretty(data);
    }
    async function loadContext() {
      const q = qs();
      const params = new URLSearchParams();
      if (q.project) params.set('project', q.project);
      if (q.workspace) params.set('workspace', q.workspace);
      if (q.visibility) params.set('visibility', q.visibility);
      if (q.agent) params.set('agent', q.agent);
      if (q.route !== 'auto') params.set('route', q.route);
      if (q.intent !== 'general') params.set('intent', q.intent);
      const res = await fetch('/memory/context/compact?' + params.toString());
      pretty(await res.json());
    }
    async function loadInbox() {
      const q = qs();
      const params = new URLSearchParams();
      if (q.project) params.set('project', q.project);
      if (q.workspace) params.set('workspace', q.workspace);
      if (q.visibility) params.set('visibility', q.visibility);
      if (q.agent) params.set('agent', q.agent);
      if (q.route !== 'auto') params.set('route', q.route);
      if (q.intent !== 'general') params.set('intent', q.intent);
      const res = await fetch('/memory/inbox?' + params.toString());
      pretty(await res.json());
    }
    async function loadSearch() {
      const q = qs();
      const body = {
        query: q.query || undefined,
        project: q.project || undefined,
        workspace: q.workspace || undefined,
        visibility: q.visibility || undefined,
        route: q.route,
        intent: q.intent,
      };
      const res = await fetch('/memory/search', {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify(body),
      });
      pretty(await res.json());
    }
    async function loadWorkspaces() {
      const q = qs();
      const params = new URLSearchParams();
      if (q.project) params.set('project', q.project);
      if (q.workspace) params.set('workspace', q.workspace);
      if (q.visibility) params.set('visibility', q.visibility);
      const res = await fetch('/memory/workspaces?' + params.toString());
      pretty(await res.json());
    }
    async function loadSources() {
      const q = qs();
      const params = new URLSearchParams();
      if (q.project) params.set('project', q.project);
      if (q.workspace) params.set('workspace', q.workspace);
      if (q.visibility) params.set('visibility', q.visibility);
      const res = await fetch('/memory/source?' + params.toString());
      pretty(await res.json());
    }
    async function loadExplain() {
      const q = qs();
      const params = new URLSearchParams();
      params.set('id', q.id);
      if (q.route !== 'auto') params.set('route', q.route);
      if (q.intent !== 'general') params.set('intent', q.intent);
      const res = await fetch('/memory/explain?' + params.toString());
      pretty(await res.json());
    }
    loadHealth().catch(err => {
      healthNote.textContent = `health check failed: ${err}`;
      output.textContent = JSON.stringify({error: String(err)}, null, 2);
    });
    setInterval(() => { loadHealth().catch(() => {}); }, 5000);
  </script>
</body>
</html>"#
        .to_string()
}

fn dashboard_html(snapshot: &VisibleMemorySnapshotResponse) -> String {
    ui::dashboard_html(snapshot)
}

fn sanitize_value(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace('|', "/")
}

fn enum_label_kind(kind: MemoryKind) -> &'static str {
    match kind {
        MemoryKind::Fact => "fact",
        MemoryKind::Decision => "decision",
        MemoryKind::Preference => "preference",
        MemoryKind::Runbook => "runbook",
        MemoryKind::Procedural => "procedural",
        MemoryKind::SelfModel => "self_model",
        MemoryKind::Topology => "topology",
        MemoryKind::Status => "status",
        MemoryKind::LiveTruth => "live_truth",
        MemoryKind::Pattern => "pattern",
        MemoryKind::Constraint => "constraint",
    }
}

fn enum_label_scope(scope: MemoryScope) -> &'static str {
    match scope {
        MemoryScope::Local => "local",
        MemoryScope::Synced => "synced",
        MemoryScope::Project => "project",
        MemoryScope::Global => "global",
    }
}

fn enum_label_visibility(visibility: MemoryVisibility) -> &'static str {
    match visibility {
        MemoryVisibility::Private => "private",
        MemoryVisibility::Workspace => "workspace",
        MemoryVisibility::Public => "public",
    }
}

fn enum_label_stage(stage: MemoryStage) -> &'static str {
    match stage {
        MemoryStage::Candidate => "candidate",
        MemoryStage::Canonical => "canonical",
    }
}

fn enum_label_status(status: MemoryStatus) -> &'static str {
    match status {
        MemoryStatus::Active => "active",
        MemoryStatus::Stale => "stale",
        MemoryStatus::Superseded => "superseded",
        MemoryStatus::Contested => "contested",
        MemoryStatus::Expired => "expired",
    }
}

fn context_score(
    item: &MemoryItem,
    entity: Option<&MemoryEntityRecord>,
    source_trust_score: f32,
    req: &ContextRequest,
    plan: &RetrievalPlan,
) -> f32 {
    let mut score = item.confidence;

    score += match item.stage {
        MemoryStage::Canonical => 1.25,
        MemoryStage::Candidate => 0.25,
    };

    score += plan.scope_rank_bonus(item.scope);
    score += plan.intent_scope_bonus(item.scope);
    score += entity_attention_bonus(item, entity);

    if let Some(project) = &req.project {
        if item.project.as_ref() == Some(project) {
            score += 1.5;
        }
    }

    if let Some(agent) = &req.agent {
        if item.source_agent.as_ref() == Some(agent) {
            score += 0.75;
        }
    }

    score += workspace_rank_adjustment(req.workspace.as_ref(), item.workspace.as_ref());

    score += entity_context_bonus(entity, req.project.as_ref(), req.agent.as_ref());
    score += trust_rank_adjustment(source_trust_score);
    score += epistemic_rank_adjustment(item);

    if item.status == MemoryStatus::Stale {
        score -= 1.5;
    }

    if item.status == MemoryStatus::Contested {
        score -= 2.0;
    }

    score -= age_penalty(item.updated_at);
    score
}

fn search_score(
    item: &MemoryItem,
    entity: Option<&MemoryEntityRecord>,
    source_trust_score: f32,
    query: &Option<String>,
    plan: &RetrievalPlan,
) -> f32 {
    let mut score = item.confidence;

    score += match item.stage {
        MemoryStage::Canonical => 1.0,
        MemoryStage::Candidate => 0.2,
    };

    score += match item.status {
        MemoryStatus::Active => 1.0,
        MemoryStatus::Stale => -1.0,
        MemoryStatus::Superseded => -2.0,
        MemoryStatus::Contested => -1.5,
        MemoryStatus::Expired => -4.0,
    };

    score += match item.scope {
        MemoryScope::Project => 0.75,
        MemoryScope::Synced => 0.5,
        MemoryScope::Local => 0.4,
        MemoryScope::Global => 0.1,
    };
    score += plan.scope_rank_bonus(item.scope) * 0.5;
    score += plan.intent_scope_bonus(item.scope) * 0.75;
    score += entity_attention_bonus(item, entity) * 0.75;
    score += trust_rank_adjustment(source_trust_score) * 0.8;
    score += epistemic_rank_adjustment(item) * 0.85;

    if let Some(query) = query {
        let content = item.content.to_ascii_lowercase();
        if content.contains(query) {
            score += 2.0;
        }
        let tag_hits = item
            .tags
            .iter()
            .filter(|tag| tag.to_ascii_lowercase().contains(query))
            .count();
        score += tag_hits as f32 * 0.5;
    }

    score -= age_penalty(item.updated_at);
    score
}

fn trust_rank_adjustment(source_trust_score: f32) -> f32 {
    if source_trust_score < 0.35 {
        -1.1
    } else if source_trust_score < 0.5 {
        -0.65
    } else if source_trust_score < 0.6 {
        -0.3
    } else if source_trust_score >= 0.9 {
        0.22
    } else if source_trust_score >= 0.75 {
        0.12
    } else {
        0.0
    }
}

fn epistemic_rank_adjustment(item: &MemoryItem) -> f32 {
    let mut score = match item.source_quality {
        Some(SourceQuality::Canonical) => 0.4,
        Some(SourceQuality::Derived) => 0.1,
        Some(SourceQuality::Synthetic) => -0.4,
        None => 0.0,
    };

    score += match item.last_verified_at {
        Some(verified_at) => {
            let verified_days = Utc::now()
                .signed_duration_since(verified_at)
                .num_days()
                .max(0);
            if verified_days <= 7 {
                0.45
            } else if verified_days <= 30 {
                0.2
            } else if verified_days <= 90 {
                0.05
            } else {
                -0.15
            }
        }
        None => -0.2,
    };

    if item.confidence < 0.6 {
        score -= 0.25;
    } else if item.confidence >= 0.9 {
        score += 0.08;
    }

    score
}

fn workspace_rank_adjustment(
    requested_workspace: Option<&String>,
    item_workspace: Option<&String>,
) -> f32 {
    match (requested_workspace, item_workspace) {
        (Some(requested), Some(item)) if requested == item => 0.85,
        (Some(_), Some(_)) => -0.18,
        (Some(_), None) => -0.08,
        _ => 0.0,
    }
}

fn age_penalty(updated_at: chrono::DateTime<Utc>) -> f32 {
    let age_days = (Utc::now() - updated_at).num_days().max(0) as f32;
    (age_days / 14.0).min(3.0)
}

fn inbox_score(
    item: &MemoryItem,
    entity: Option<&MemoryEntityRecord>,
    plan: &RetrievalPlan,
) -> f32 {
    let mut score = item.confidence;
    score += plan.scope_rank_bonus(item.scope);
    score += plan.intent_scope_bonus(item.scope);
    score += match item.stage {
        MemoryStage::Candidate => 2.0,
        MemoryStage::Canonical => 0.5,
    };
    score += match item.status {
        MemoryStatus::Contested => 2.5,
        MemoryStatus::Stale => 2.0,
        MemoryStatus::Superseded => 1.5,
        MemoryStatus::Expired => 1.0,
        MemoryStatus::Active => 0.0,
    };
    score += entity_attention_bonus(item, entity);
    score -= age_penalty(item.updated_at) * 0.75;
    score
}

fn entity_attention_bonus(item: &MemoryItem, entity: Option<&MemoryEntityRecord>) -> f32 {
    let Some(entity) = entity else {
        return 0.0;
    };

    let salience = entity.salience_score.clamp(0.0, 1.0);
    let rehearsal = (entity.rehearsal_count as f32 + 1.0).ln_1p();
    let recency = entity
        .last_accessed_at
        .map(|at| {
            let age_days = (Utc::now() - at).num_days().max(0) as f32;
            (1.0 - (age_days / 30.0)).clamp(0.0, 1.0)
        })
        .unwrap_or(0.0);
    let state_alignment = entity
        .context
        .as_ref()
        .map(|context| {
            let mut bonus = 0.0;
            if context.project.as_ref() == item.project.as_ref() {
                bonus += 0.2;
            }
            if context.namespace.as_ref() == item.namespace.as_ref() {
                bonus += 0.1;
            }
            if context.agent.as_ref() == item.source_agent.as_ref() {
                bonus += 0.1;
            }
            bonus
        })
        .unwrap_or(0.0);

    salience * 0.9 + rehearsal * 0.25 + recency * 0.25 + state_alignment
}

fn entity_context_bonus(
    entity: Option<&MemoryEntityRecord>,
    project: Option<&String>,
    agent: Option<&String>,
) -> f32 {
    let Some(entity) = entity else {
        return 0.0;
    };

    let mut bonus = 0.0;
    if let Some(context) = &entity.context {
        if context.project.as_ref() == project {
            bonus += 0.35;
        }
        if context.agent.as_ref() == agent {
            bonus += 0.2;
        }
    }
    bonus
}

fn inbox_reasons(item: &MemoryItem) -> Vec<String> {
    let mut reasons = Vec::new();
    if item.preferred {
        reasons.push("preferred-branch".to_string());
    }
    if item.stage == MemoryStage::Candidate {
        reasons.push("candidate".to_string());
    }
    match item.status {
        MemoryStatus::Contested => reasons.push("contested".to_string()),
        MemoryStatus::Stale => reasons.push("stale".to_string()),
        MemoryStatus::Superseded => reasons.push("superseded".to_string()),
        MemoryStatus::Expired => reasons.push("expired".to_string()),
        MemoryStatus::Active => {}
    }
    if item.source_quality == Some(SourceQuality::Derived) {
        reasons.push("derived".to_string());
    }
    if item.source_quality == Some(SourceQuality::Synthetic) {
        reasons.push("rejected-source".to_string());
    }
    if item.confidence < 0.75 {
        reasons.push("low-confidence".to_string());
    }
    if item.ttl_seconds.is_some() {
        reasons.push("ttl".to_string());
    }
    if item.belief_branch.is_some() && !item.preferred && item.status == MemoryStatus::Contested {
        reasons.push("unresolved-contradiction".to_string());
    }
    reasons
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use memd_schema::MemoryRepairMode;
    use tower::util::ServiceExt;

    fn sample_memory_item(workspace: Option<&str>) -> MemoryItem {
        let now = Utc::now();
        MemoryItem {
            id: uuid::Uuid::new_v4(),
            content: "workspace-ranked memory".to_string(),
            redundancy_key: None,
            belief_branch: None,
            preferred: false,
            kind: MemoryKind::Fact,
            scope: MemoryScope::Project,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: workspace.map(|value| value.to_string()),
            visibility: MemoryVisibility::Workspace,
            source_agent: Some("codex".to_string()),
            source_system: Some("cli".to_string()),
            source_path: None,
            source_quality: Some(SourceQuality::Canonical),
            confidence: 0.9,
            ttl_seconds: None,
            created_at: now,
            updated_at: now,
            last_verified_at: Some(now),
            supersedes: Vec::new(),
            tags: vec!["workspace".to_string()],
            status: MemoryStatus::Active,
            stage: MemoryStage::Canonical,
        }
    }

    fn temp_state(name: &str) -> (std::path::PathBuf, AppState) {
        let dir = std::env::temp_dir().join(format!("{name}-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp state dir");
        let db_path = dir.join("memd.db");
        let state = AppState {
            store: SqliteStore::open(&db_path).expect("open temp store"),
        };
        (dir, state)
    }

    fn test_hive_router(state: AppState) -> Router {
        Router::new()
            .route("/hive/board", get(get_hive_board))
            .route("/hive/roster", get(get_hive_roster))
            .route("/hive/follow", get(get_hive_follow))
            .with_state(state)
    }

    fn seed_hive_route_state(state: &AppState) {
        state
            .store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "queen-1".to_string(),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@queen-1".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("orchestrator".to_string()),
                worker_name: Some("Avicenna".to_string()),
                display_name: None,
                role: Some("queen".to_string()),
                capabilities: vec!["coordination".to_string()],
                hive_groups: vec!["project:memd".to_string()],
                lane_id: Some("queen-lane".to_string()),
                hive_group_goal: None,
                authority: Some("coordinator".to_string()),
                heartbeat_model: Some("gpt-5.4".to_string()),
                tab_id: None,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: Some("main".to_string()),
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: Some("workstation".to_string()),
                pid: Some(100),
                topic_claim: Some("Route hive work".to_string()),
                scope_claims: vec!["docs/hive.md".to_string()],
                task_id: Some("queen-routing".to_string()),
                focus: Some("Coordinate bee lanes".to_string()),
                pressure: None,
                next_recovery: None,
                next_action: Some("Review overlap alerts".to_string()),
                needs_help: false,
                needs_review: false,
                handoff_state: None,
                confidence: Some("0.95".to_string()),
                risk: None,
                status: Some("active".to_string()),
            })
            .expect("insert queen session");
        state
            .store
            .upsert_hive_session(&HiveSessionUpsertRequest {
                session: "bee-1".to_string(),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@bee-1".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("Lorentz".to_string()),
                display_name: None,
                role: Some("worker".to_string()),
                capabilities: vec!["coding".to_string()],
                hive_groups: vec!["project:memd".to_string()],
                lane_id: Some("parser-lane".to_string()),
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some("gpt-5.4".to_string()),
                tab_id: None,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: Some("/tmp/bee-1".to_string()),
                branch: Some("feature/parser".to_string()),
                base_branch: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: Some("workstation".to_string()),
                pid: Some(101),
                topic_claim: Some("Parser lane refactor".to_string()),
                scope_claims: vec![
                    "project".to_string(),
                    "crates/memd-client/src/main.rs".to_string(),
                    "task:parser-refactor".to_string(),
                ],
                task_id: Some("parser-refactor".to_string()),
                focus: Some("Refine parser overlap flow".to_string()),
                pressure: None,
                next_recovery: None,
                next_action: Some("Request review".to_string()),
                needs_help: false,
                needs_review: true,
                handoff_state: None,
                confidence: Some("0.9".to_string()),
                risk: None,
                status: Some("active".to_string()),
            })
            .expect("insert bee session");
        state
            .store
            .upsert_hive_task(&HiveTaskUpsertRequest {
                task_id: "parser-refactor".to_string(),
                title: "Refine parser overlap flow".to_string(),
                description: Some("narrow parser work".to_string()),
                status: Some("active".to_string()),
                coordination_mode: Some("exclusive_write".to_string()),
                session: Some("bee-1".to_string()),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@bee-1".to_string()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                claim_scopes: vec!["crates/memd-client/src/main.rs".to_string()],
                help_requested: Some(false),
                review_requested: Some(true),
            })
            .expect("insert hive task");
        state
            .store
            .send_hive_message(&HiveMessageSendRequest {
                kind: "note".to_string(),
                from_session: "queen-1".to_string(),
                from_agent: Some("codex".to_string()),
                to_session: "bee-1".to_string(),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                content: "Stay on parser lane only.".to_string(),
            })
            .expect("insert hive message");
        state
            .store
            .record_hive_coordination_receipt(&HiveCoordinationReceiptRequest {
                kind: "queen_assign".to_string(),
                actor_session: "queen-1".to_string(),
                actor_agent: Some("codex".to_string()),
                target_session: Some("bee-1".to_string()),
                task_id: Some("parser-refactor".to_string()),
                scope: Some("crates/memd-client/src/main.rs".to_string()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                summary: "Assigned parser lane".to_string(),
            })
            .expect("insert coordination receipt");
    }

    async fn decode_json<T: serde::de::DeserializeOwned>(response: axum::response::Response) -> T {
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read response body");
        serde_json::from_slice(&body).expect("decode response json")
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
                needs_help: false,
                needs_review: false,
                handoff_state: None,
                confidence: Some("0.82".to_string()),
                risk: None,
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
                needs_help: false,
                needs_review: false,
                handoff_state: None,
                confidence: Some("0.6".to_string()),
                risk: None,
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

    #[test]
    fn matching_workspace_ranks_above_other_shared_workspace() {
        let req = ContextRequest {
            project: Some("memd".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some(MemoryVisibility::Workspace),
            route: Some(RetrievalRoute::ProjectFirst),
            intent: Some(RetrievalIntent::General),
            limit: Some(8),
            max_chars_per_item: Some(220),
        };
        let plan = RetrievalPlan::resolve(req.route, req.intent);
        let matching = sample_memory_item(Some("team-alpha"));
        let unrelated = sample_memory_item(Some("team-beta"));

        assert!(
            context_score(&matching, None, 0.9, &req, &plan)
                > context_score(&unrelated, None, 0.9, &req, &plan)
        );
    }

    #[test]
    fn verified_canonical_memory_ranks_above_unverified_synthetic_memory() {
        let req = ContextRequest {
            project: Some("memd".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some(MemoryVisibility::Workspace),
            route: Some(RetrievalRoute::ProjectFirst),
            intent: Some(RetrievalIntent::CurrentTask),
            limit: Some(8),
            max_chars_per_item: Some(220),
        };
        let plan = RetrievalPlan::resolve(req.route, req.intent);
        let mut verified = sample_memory_item(Some("team-alpha"));
        verified.source_quality = Some(SourceQuality::Canonical);
        verified.last_verified_at = Some(Utc::now());
        verified.confidence = 0.88;

        let mut inferred = sample_memory_item(Some("team-alpha"));
        inferred.source_quality = Some(SourceQuality::Synthetic);
        inferred.last_verified_at = None;
        inferred.confidence = 0.88;

        assert!(
            context_score(&verified, None, 0.7, &req, &plan)
                > context_score(&inferred, None, 0.7, &req, &plan)
        );
        assert!(
            search_score(&verified, None, 0.7, &Some("workspace".to_string()), &plan)
                > search_score(&inferred, None, 0.7, &Some("workspace".to_string()), &plan)
        );
    }

    #[test]
    fn live_truth_precedes_project_memory() {
        let db_path =
            std::env::temp_dir().join(format!("memd-live-truth-{}.db", uuid::Uuid::new_v4()));
        let state = AppState {
            store: SqliteStore::open(&db_path).expect("open temp db"),
        };

        let _ = state
            .store_item(
                StoreMemoryRequest {
                    content: "recent repo change: update live truth".to_string(),
                    kind: MemoryKind::LiveTruth,
                    scope: MemoryScope::Local,
                    project: Some("demo".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("team-alpha".to_string()),
                    visibility: Some(MemoryVisibility::Workspace),
                    belief_branch: None,
                    source_agent: Some("memd".to_string()),
                    source_system: Some("memd-live-truth".to_string()),
                    source_path: Some("/tmp/demo".to_string()),
                    source_quality: Some(SourceQuality::Derived),
                    confidence: Some(0.98),
                    ttl_seconds: Some(3_600),
                    last_verified_at: Some(Utc::now()),
                    supersedes: Vec::new(),
                    tags: vec!["live_truth".to_string()],
                    status: Some(MemoryStatus::Active),
                },
                MemoryStage::Canonical,
            )
            .expect("store live truth");

        let _ = state
            .store_item(
                StoreMemoryRequest {
                    content: "older project fact".to_string(),
                    kind: MemoryKind::Fact,
                    scope: MemoryScope::Project,
                    project: Some("demo".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("team-alpha".to_string()),
                    visibility: Some(MemoryVisibility::Workspace),
                    belief_branch: None,
                    source_agent: Some("codex".to_string()),
                    source_system: Some("cli".to_string()),
                    source_path: Some("notes.md".to_string()),
                    source_quality: Some(SourceQuality::Canonical),
                    confidence: Some(0.9),
                    ttl_seconds: None,
                    last_verified_at: Some(Utc::now()),
                    supersedes: Vec::new(),
                    tags: vec!["fact".to_string()],
                    status: Some(MemoryStatus::Active),
                },
                MemoryStage::Canonical,
            )
            .expect("store fact");

        let BuildContextResult { items, .. } = build_context(
            &state,
            &ContextRequest {
                project: Some("demo".to_string()),
                agent: Some("codex".to_string()),
                workspace: Some("team-alpha".to_string()),
                visibility: Some(MemoryVisibility::Workspace),
                route: Some(RetrievalRoute::ProjectFirst),
                intent: Some(RetrievalIntent::CurrentTask),
                limit: Some(4),
                max_chars_per_item: Some(220),
            },
        )
        .expect("build context");

        assert_eq!(
            items.first().map(|item| item.kind),
            Some(MemoryKind::LiveTruth)
        );
        assert!(
            items
                .iter()
                .any(|item| item.content.contains("older project fact"))
        );
    }

    #[test]
    fn current_task_context_keeps_project_fact_visible_under_synced_noise() {
        let db_path = std::env::temp_dir().join(format!(
            "memd-current-task-project-fact-{}.db",
            uuid::Uuid::new_v4()
        ));
        let state = AppState {
            store: SqliteStore::open(&db_path).expect("open temp db"),
        };

        let _ = state
            .store_item(
                StoreMemoryRequest {
                    content:
                        "remembered project fact: memd must preserve important user corrections"
                            .to_string(),
                    kind: MemoryKind::Fact,
                    scope: MemoryScope::Project,
                    project: Some("demo".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("team-alpha".to_string()),
                    visibility: Some(MemoryVisibility::Workspace),
                    belief_branch: None,
                    source_agent: Some("codex".to_string()),
                    source_system: Some("memd".to_string()),
                    source_path: Some("notes.md".to_string()),
                    source_quality: Some(SourceQuality::Canonical),
                    confidence: Some(0.98),
                    ttl_seconds: None,
                    last_verified_at: Some(Utc::now()),
                    supersedes: Vec::new(),
                    tags: vec!["project_fact".to_string()],
                    status: Some(MemoryStatus::Active),
                },
                MemoryStage::Canonical,
            )
            .expect("store durable project fact");

        for index in 0..5 {
            let _ = state
                .store_item(
                    StoreMemoryRequest {
                        content: format!("resume state noise {index}: synced session snapshot"),
                        kind: MemoryKind::Status,
                        scope: MemoryScope::Synced,
                        project: Some("demo".to_string()),
                        namespace: Some("main".to_string()),
                        workspace: Some("team-alpha".to_string()),
                        visibility: Some(MemoryVisibility::Workspace),
                        belief_branch: None,
                        source_agent: Some(format!("codex@session-{index}")),
                        source_system: Some("memd-resume-state".to_string()),
                        source_path: None,
                        source_quality: Some(SourceQuality::Derived),
                        confidence: Some(0.94),
                        ttl_seconds: Some(86_400),
                        last_verified_at: Some(Utc::now()),
                        supersedes: Vec::new(),
                        tags: vec!["resume_state".to_string(), "session_state".to_string()],
                        status: Some(MemoryStatus::Active),
                    },
                    MemoryStage::Canonical,
                )
                .expect("store synced session noise");
        }

        let BuildContextResult { items, .. } = build_context(
            &state,
            &ContextRequest {
                project: Some("demo".to_string()),
                agent: Some("codex".to_string()),
                workspace: Some("team-alpha".to_string()),
                visibility: Some(MemoryVisibility::Workspace),
                route: Some(RetrievalRoute::LocalFirst),
                intent: Some(RetrievalIntent::CurrentTask),
                limit: Some(4),
                max_chars_per_item: Some(220),
            },
        )
        .expect("build current-task context");

        assert!(
            items
                .iter()
                .any(|item| item.content.contains("remembered project fact")),
            "durable project fact should survive current-task retrieval even when synced session-state noise exists"
        );
    }

    #[test]
    fn current_task_context_prefers_matching_workspace_memory_under_cross_workspace_noise() {
        let db_path = std::env::temp_dir().join(format!(
            "memd-current-task-workspace-fact-{}.db",
            uuid::Uuid::new_v4()
        ));
        let state = AppState {
            store: SqliteStore::open(&db_path).expect("open temp db"),
        };

        let _ = state
            .store_item(
                StoreMemoryRequest {
                    content: "shared workspace handoff: team-alpha owns the memory audit"
                        .to_string(),
                    kind: MemoryKind::Status,
                    scope: MemoryScope::Synced,
                    project: Some("demo".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("team-alpha".to_string()),
                    visibility: Some(MemoryVisibility::Workspace),
                    belief_branch: None,
                    source_agent: Some("codex@shared-a".to_string()),
                    source_system: Some("handoff".to_string()),
                    source_path: Some("handoff.md".to_string()),
                    source_quality: Some(SourceQuality::Canonical),
                    confidence: Some(0.96),
                    ttl_seconds: Some(86_400),
                    last_verified_at: Some(Utc::now()),
                    supersedes: Vec::new(),
                    tags: vec!["handoff".to_string(), "workspace".to_string()],
                    status: Some(MemoryStatus::Active),
                },
                MemoryStage::Canonical,
            )
            .expect("store matching workspace memory");

        for index in 0..5 {
            let _ = state
                .store_item(
                    StoreMemoryRequest {
                        content: format!(
                            "team-beta session noise {index}: unrelated workspace state"
                        ),
                        kind: MemoryKind::Status,
                        scope: MemoryScope::Synced,
                        project: Some("demo".to_string()),
                        namespace: Some("main".to_string()),
                        workspace: Some("team-beta".to_string()),
                        visibility: Some(MemoryVisibility::Workspace),
                        belief_branch: None,
                        source_agent: Some(format!("codex@team-beta-{index}")),
                        source_system: Some("memd-resume-state".to_string()),
                        source_path: None,
                        source_quality: Some(SourceQuality::Derived),
                        confidence: Some(0.94),
                        ttl_seconds: Some(86_400),
                        last_verified_at: Some(Utc::now()),
                        supersedes: Vec::new(),
                        tags: vec!["resume_state".to_string(), "session_state".to_string()],
                        status: Some(MemoryStatus::Active),
                    },
                    MemoryStage::Canonical,
                )
                .expect("store unrelated workspace noise");
        }

        let BuildContextResult { items, .. } = build_context(
            &state,
            &ContextRequest {
                project: Some("demo".to_string()),
                agent: Some("codex".to_string()),
                workspace: Some("team-alpha".to_string()),
                visibility: Some(MemoryVisibility::Workspace),
                route: Some(RetrievalRoute::LocalFirst),
                intent: Some(RetrievalIntent::CurrentTask),
                limit: Some(4),
                max_chars_per_item: Some(220),
            },
        )
        .expect("build current-task context");

        assert!(
            items
                .iter()
                .any(|item| item.content.contains("team-alpha owns the memory audit")),
            "matching workspace memory should survive current-task retrieval even when unrelated synced workspace noise exists"
        );
        assert!(
            items
                .iter()
                .any(|item| item.workspace.as_deref() == Some("team-alpha")),
            "matching workspace should remain represented in the retrieved set"
        );
    }

    #[test]
    fn superseded_memory_drops_out_after_manual_correction_loop() {
        let db_path =
            std::env::temp_dir().join(format!("memd-correction-loop-{}.db", uuid::Uuid::new_v4()));
        let state = AppState {
            store: SqliteStore::open(&db_path).expect("open temp db"),
        };

        let (old_item, _) = state
            .store_item(
                StoreMemoryRequest {
                    content: "stale belief: roadmap completion proves memd functionality"
                        .to_string(),
                    kind: MemoryKind::Fact,
                    scope: MemoryScope::Project,
                    project: Some("demo".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("team-alpha".to_string()),
                    visibility: Some(MemoryVisibility::Workspace),
                    belief_branch: None,
                    source_agent: Some("codex".to_string()),
                    source_system: Some("memd".to_string()),
                    source_path: Some("notes.md".to_string()),
                    source_quality: Some(SourceQuality::Canonical),
                    confidence: Some(0.92),
                    ttl_seconds: None,
                    last_verified_at: Some(Utc::now()),
                    supersedes: Vec::new(),
                    tags: vec!["fact".to_string()],
                    status: Some(MemoryStatus::Active),
                },
                MemoryStage::Canonical,
            )
            .expect("store stale belief");

        repair::repair_item(
            &state,
            RepairMemoryRequest {
                id: old_item.id,
                mode: MemoryRepairMode::Supersede,
                confidence: Some(0.25),
                status: Some(MemoryStatus::Superseded),
                workspace: None,
                visibility: None,
                source_agent: None,
                source_system: None,
                source_path: None,
                source_quality: None,
                content: None,
                tags: None,
                supersedes: vec![],
            },
        )
        .expect("supersede stale belief");

        let _ = state
            .store_item(
                StoreMemoryRequest {
                    content: "corrected fact: roadmap status is not proof of working memory recall"
                        .to_string(),
                    kind: MemoryKind::Fact,
                    scope: MemoryScope::Project,
                    project: Some("demo".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("team-alpha".to_string()),
                    visibility: Some(MemoryVisibility::Workspace),
                    belief_branch: None,
                    source_agent: Some("user".to_string()),
                    source_system: Some("correction".to_string()),
                    source_path: Some("conversation".to_string()),
                    source_quality: Some(SourceQuality::Canonical),
                    confidence: Some(0.99),
                    ttl_seconds: None,
                    last_verified_at: Some(Utc::now()),
                    supersedes: vec![old_item.id],
                    tags: vec!["correction".to_string()],
                    status: Some(MemoryStatus::Active),
                },
                MemoryStage::Canonical,
            )
            .expect("store corrected fact");

        let BuildContextResult { items, .. } = build_context(
            &state,
            &ContextRequest {
                project: Some("demo".to_string()),
                agent: Some("codex".to_string()),
                workspace: Some("team-alpha".to_string()),
                visibility: Some(MemoryVisibility::Workspace),
                route: Some(RetrievalRoute::ProjectFirst),
                intent: Some(RetrievalIntent::CurrentTask),
                limit: Some(4),
                max_chars_per_item: Some(220),
            },
        )
        .expect("build corrected context");

        assert!(
            items.iter().any(|item| {
                item.content
                    .contains("roadmap status is not proof of working memory recall")
            }),
            "corrected fact should be visible after the correction loop"
        );
        assert!(
            !items.iter().any(|item| item
                .content
                .contains("roadmap completion proves memd functionality")),
            "superseded stale belief should not remain in active current-task context"
        );
    }

    #[test]
    fn explicit_store_revives_superseded_canonical_duplicate() {
        let (dir, state) = temp_state("memd-revive-duplicate");
        let (old_item, duplicate) = state
            .store_item(
                StoreMemoryRequest {
                    content:
                        "corrected fact: hosted backend health does not prove usable agent memory"
                            .to_string(),
                    kind: MemoryKind::Fact,
                    scope: MemoryScope::Project,
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: None,
                    visibility: Some(MemoryVisibility::Private),
                    belief_branch: None,
                    source_agent: Some("codex".to_string()),
                    source_system: Some("memd".to_string()),
                    source_path: Some("hook-capture-promotion".to_string()),
                    source_quality: Some(SourceQuality::Canonical),
                    confidence: Some(0.25),
                    ttl_seconds: None,
                    last_verified_at: None,
                    supersedes: Vec::new(),
                    tags: vec!["correction".to_string()],
                    status: Some(MemoryStatus::Superseded),
                },
                MemoryStage::Canonical,
            )
            .expect("store old superseded correction");
        assert!(duplicate.is_none());

        let (revived, duplicate) = state
            .store_item(
                StoreMemoryRequest {
                    content:
                        "corrected fact: hosted backend health does not prove usable agent memory"
                            .to_string(),
                    kind: MemoryKind::Fact,
                    scope: MemoryScope::Project,
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: None,
                    visibility: Some(MemoryVisibility::Private),
                    belief_branch: None,
                    source_agent: Some("codex".to_string()),
                    source_system: Some("memd".to_string()),
                    source_path: Some("hook-capture-promotion".to_string()),
                    source_quality: Some(SourceQuality::Canonical),
                    confidence: Some(0.99),
                    ttl_seconds: None,
                    last_verified_at: Some(Utc::now()),
                    supersedes: vec![uuid::Uuid::new_v4()],
                    tags: vec!["correction".to_string(), "product-direction".to_string()],
                    status: Some(MemoryStatus::Active),
                },
                MemoryStage::Canonical,
            )
            .expect("revive duplicate");

        assert!(duplicate.is_none());
        assert_eq!(revived.id, old_item.id);
        assert_eq!(revived.status, MemoryStatus::Active);
        assert_eq!(revived.confidence, 0.99);
        assert!(revived.tags.iter().any(|tag| tag == "product-direction"));
        assert!(!revived.supersedes.contains(&revived.id));
        std::fs::remove_dir_all(dir).expect("cleanup temp state dir");
    }

    #[tokio::test]
    async fn ui_artifact_handler_returns_detail_response() {
        let state = AppState {
            store: SqliteStore::open(
                std::env::temp_dir().join(format!("memd-ui-detail-{}.db", uuid::Uuid::new_v4())),
            )
            .expect("open temp db"),
        };
        let item = ui::test_insert_visible_item(&state, "runtime spine", true).unwrap();

        let response = super::get_visible_memory_artifact(
            State(state),
            Query(super::VisibleMemoryArtifactQuery { id: item.id }),
        )
        .await
        .expect("build artifact detail")
        .0;

        assert_eq!(response.artifact.id, item.id);
        assert!(response.explain.is_some());
    }

    #[tokio::test]
    async fn ui_action_handler_returns_open_metadata() {
        let state = AppState {
            store: SqliteStore::open(
                std::env::temp_dir().join(format!("memd-ui-action-{}.db", uuid::Uuid::new_v4())),
            )
            .expect("open temp db"),
        };
        let item = ui::test_insert_visible_item(&state, "runtime spine", true).unwrap();

        let response = super::post_visible_memory_action(
            State(state),
            Json(VisibleMemoryUiActionRequest {
                id: item.id,
                action: memd_schema::VisibleMemoryUiActionKind::OpenInObsidian,
            }),
        )
        .await
        .expect("build action response")
        .0;

        assert_eq!(response.artifact_id, item.id);
        assert_eq!(
            response.open_uri.as_deref(),
            Some("obsidian://open?path=wiki/runtime-spine.md")
        );
    }
}
