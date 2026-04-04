mod inspection;
mod keys;
mod repair;
mod routing;
mod store;
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
    EntitySearchRequest, EntitySearchResponse, HealthResponse, InboxMemoryItem,
    MemoryConsolidationRequest, MemoryConsolidationResponse, MemoryContextFrame,
    MemoryDecayRequest, MemoryDecayResponse, MemoryEntityLinkRecord, MemoryEntityRecord,
    MemoryEventRecord, MemoryInboxRequest, MemoryInboxResponse, MemoryItem, MemoryKind,
    MemoryMaintenanceReportRequest, MemoryMaintenanceReportResponse, MemoryPolicyResponse,
    MemoryScope, MemoryStage, MemoryStatus, PromoteMemoryRequest, PromoteMemoryResponse,
    RepairMemoryRequest, RepairMemoryResponse, RetrievalIntent, RetrievalRoute,
    SearchMemoryRequest, SearchMemoryResponse, SourceMemoryRequest, SourceMemoryResponse,
    SourceQuality, StoreMemoryRequest, StoreMemoryResponse, TimelineMemoryRequest,
    TimelineMemoryResponse, ExpireMemoryRequest, ExpireMemoryResponse, ExplainMemoryRequest,
    ExplainMemoryResponse, VerifyMemoryRequest, VerifyMemoryResponse, WorkingMemoryRequest,
    WorkingMemoryResponse,
};
use routing::RetrievalPlan;
use store::{DuplicateMatch, SqliteStore};
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
            event_type,
            summary,
            item.updated_at,
            item.project.clone(),
            item.namespace.clone(),
            item.source_agent.clone(),
            item.source_system.clone(),
            item.source_path.clone(),
            vec![],
            item.tags.clone(),
            context,
            item.confidence,
            entity.record.salience_score,
        )
    }
}

#[tokio::main]
async fn main() {
    let db_path = std::env::var("MEMD_DB_PATH").unwrap_or_else(|_| "memd.db".to_string());
    let state = AppState {
        store: SqliteStore::open(&db_path).expect("open memd sqlite store"),
    };
    let app = Router::new()
        .route("/", get(dashboard))
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
        .route("/memory/explain", get(get_explain))
        .route("/memory/maintenance/decay", post(decay_memory))
        .route("/memory/maintenance/consolidate", post(consolidate_memory))
        .route("/memory/maintenance/report", get(get_maintenance_report))
        .route("/memory/policy", get(get_memory_policy))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8787")
        .await
        .expect("bind memd");
    axum::serve(listener, app).await.expect("serve memd");
}

async fn healthz(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        items: state.store.count().unwrap_or(0),
    })
}

async fn dashboard() -> Html<String> {
    Html(dashboard_html())
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
    let items = enrich_with_entities(&state, state.snapshot().map_err(internal_error)?)
        .map_err(internal_error)?;
    let plan = RetrievalPlan::resolve(req.route, req.intent);
    let items = filter_items(&items, &req, &plan);
    state.rehearse_items(&items, 3).map_err(internal_error)?;
    state
        .record_retrieval_feedback(&items, 3, "retrieved_search", &plan)
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
    let req = apply_agent_profile_defaults(&state, req).map_err(internal_error)?;
    let (plan, retrieval_order, items) = build_context(&state, &req)?;
    state.rehearse_items(&items, 3).map_err(internal_error)?;
    state
        .record_retrieval_feedback(&items, 3, "retrieved_context", &plan)
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
    let (plan, retrieval_order, items) = build_context(&state, &req)?;
    state.rehearse_items(&items, 3).map_err(internal_error)?;
    state
        .record_retrieval_feedback(&items, 3, "retrieved_compact_context", &plan)
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
                at: req.at.clone(),
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
            self.store.record_event(
                &entity.record,
                item.id,
                event_type,
                format!(
                    "{} route={} intent={}",
                    event_type,
                    enum_label_route(plan.route),
                    enum_label_intent(plan.intent)
                ),
                Utc::now(),
                item.project.clone(),
                item.namespace.clone(),
                item.source_agent.clone(),
                item.source_system.clone(),
                item.source_path.clone(),
                vec![],
                tags,
                Some(entity_context_frame(&entity.record, item)),
                item.confidence,
                entity.record.salience_score,
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
        let candidates = self
            .store
            .consolidation_candidates(req)
            .context("load consolidation candidates")?;

        let min_salience = req.min_salience.unwrap_or(0.22).clamp(0.0, 1.0);
        let record_events = req.record_events.unwrap_or(true);

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
                let _ = self.store.record_event(
                    &candidate.entity,
                    item.id,
                    "consolidated",
                    format!(
                        "episodic traces consolidated after {} events into semantic memory",
                        candidate.event_count
                    ),
                    candidate.last_recorded_at,
                    candidate
                        .entity
                        .context
                        .as_ref()
                        .and_then(|context| context.project.clone()),
                    candidate
                        .entity
                        .context
                        .as_ref()
                        .and_then(|context| context.namespace.clone()),
                    candidate
                        .entity
                        .context
                        .as_ref()
                        .and_then(|context| context.agent.clone()),
                    candidate
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
                    candidate
                        .entity
                        .context
                        .as_ref()
                        .and_then(|context| context.location.clone()),
                    vec![item.id],
                    consolidation_tags(&candidate.entity, candidate.event_count),
                    Some(entity_context_frame(&candidate.entity, &item)),
                    item.confidence,
                    candidate.entity.salience_score,
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
        })
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

fn build_context(
    state: &AppState,
    req: &ContextRequest,
) -> Result<(RetrievalPlan, Vec<MemoryScope>, Vec<MemoryItem>), (StatusCode, String)> {
    let plan = RetrievalPlan::resolve(req.route, req.intent);
    let limit = req.limit.unwrap_or(8).min(32);
    let max_chars = req.max_chars_per_item.unwrap_or(280).clamp(80, 2000);
    let items = enrich_with_entities(&state, state.snapshot().map_err(internal_error)?)
        .map_err(internal_error)?;
    let retrieval_order = plan.scopes();

    let mut scoped: Vec<MemoryItem> = Vec::new();
    for scope in retrieval_order.iter().copied() {
        let mut bucket: Vec<MemoryViewItem> = items
            .iter()
            .filter(|entry| plan.allows(entry.item.scope))
            .filter(|entry| entry.item.scope == scope)
            .filter(|entry| entry.item.status == MemoryStatus::Active)
            .filter(|entry| match (&req.project, &entry.item.project, scope) {
                (Some(project), Some(item_project), MemoryScope::Project) => {
                    item_project == project
                }
                (Some(project), Some(item_project), MemoryScope::Synced) => item_project == project,
                (Some(_), None, MemoryScope::Project | MemoryScope::Synced) => false,
                _ => true,
            })
            .cloned()
            .collect();

        bucket.sort_by(|a, b| {
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

        scoped.extend(bucket.into_iter().map(|entry| entry.item));
        if scoped.len() >= limit {
            break;
        }
    }

    for item in &mut scoped {
        item.content = compact_content(&item.content, max_chars);
    }
    scoped.truncate(limit);

    Ok((plan, retrieval_order, scoped))
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
        search_score(&b.item, b.entity.as_ref(), b.source_trust_score, &query, plan)
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
        "topology" => MemoryKind::Topology,
        "status" => MemoryKind::Status,
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

fn dashboard_html() -> String {
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
        MemoryKind::Topology => "topology",
        MemoryKind::Status => "status",
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

    score += entity_context_bonus(entity, req.project.as_ref(), req.agent.as_ref());
    score += trust_rank_adjustment(source_trust_score);

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
