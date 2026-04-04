mod keys;
mod routing;
mod store;

use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
};
use chrono::Utc;
use keys::{apply_lifecycle, canonical_key, redundancy_key, validate_source_quality};
use memd_schema::{
    CandidateMemoryRequest, CandidateMemoryResponse, CompactContextResponse, CompactMemoryRecord,
    ContextRequest, ContextResponse, ExpireMemoryRequest, ExpireMemoryResponse,
    ExplainMemoryRequest, ExplainMemoryResponse, HealthResponse, InboxMemoryItem,
    MemoryInboxRequest, MemoryInboxResponse, MemoryItem, MemoryKind, MemoryScope, MemoryStage,
    MemoryStatus, PromoteMemoryRequest, PromoteMemoryResponse, SearchMemoryRequest,
    SearchMemoryResponse, SourceQuality, StoreMemoryRequest, StoreMemoryResponse,
    VerifyMemoryRequest, VerifyMemoryResponse,
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
        let duplicate =
            self.store
                .insert_or_get_duplicate(&item, &canonical_key, &redundancy_key)?;
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
        Ok((item, None))
    }

    fn expire_item(&self, req: ExpireMemoryRequest) -> anyhow::Result<MemoryItem> {
        let mut item = self
            .store
            .get(req.id)?
            .ok_or_else(|| anyhow::anyhow!("memory item not found"))?;

        item.status = req.status.unwrap_or(MemoryStatus::Expired);
        item.updated_at = Utc::now();
        let canonical_key = canonical_key(&item);
        let redundancy_key = redundancy_key(&item);
        let item = MemoryItem {
            redundancy_key: Some(redundancy_key.clone()),
            ..item
        };
        self.store.update(&item, &canonical_key, &redundancy_key)?;
        Ok(item)
    }

    fn verify_item(&self, req: VerifyMemoryRequest) -> anyhow::Result<MemoryItem> {
        let mut item = self
            .store
            .get(req.id)?
            .ok_or_else(|| anyhow::anyhow!("memory item not found"))?;

        item.last_verified_at = Some(Utc::now());
        if let Some(confidence) = req.confidence {
            item.confidence = confidence.clamp(0.0, 1.0);
        }
        item.status = req.status.unwrap_or(MemoryStatus::Active);
        item.updated_at = Utc::now();
        let canonical_key = canonical_key(&item);
        let redundancy_key = redundancy_key(&item);
        let item = MemoryItem {
            redundancy_key: Some(redundancy_key.clone()),
            ..item
        };
        self.store.update(&item, &canonical_key, &redundancy_key)?;
        Ok(item)
    }
}

#[tokio::main]
async fn main() {
    let db_path = std::env::var("MEMD_DB_PATH").unwrap_or_else(|_| "memd.db".to_string());
    let state = AppState {
        store: SqliteStore::open(&db_path).expect("open memd sqlite store"),
    };
    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/memory/store", post(store_memory))
        .route("/memory/candidates", post(store_candidate))
        .route("/memory/promote", post(promote_memory))
        .route("/memory/expire", post(expire_memory))
        .route("/memory/verify", post(verify_memory))
        .route("/memory/search", post(search_memory))
        .route("/memory/context", get(get_context))
        .route("/memory/context/compact", get(get_compact_context))
        .route("/memory/inbox", get(get_inbox))
        .route("/memory/explain", get(get_explain))
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
    let item = state.expire_item(req).map_err(internal_error)?;
    Ok(Json(ExpireMemoryResponse { item }))
}

async fn verify_memory(
    State(state): State<AppState>,
    Json(req): Json<VerifyMemoryRequest>,
) -> Result<Json<VerifyMemoryResponse>, (StatusCode, String)> {
    let item = state.verify_item(req).map_err(internal_error)?;
    Ok(Json(VerifyMemoryResponse { item }))
}

async fn search_memory(
    State(state): State<AppState>,
    Json(req): Json<SearchMemoryRequest>,
) -> Result<Json<SearchMemoryResponse>, (StatusCode, String)> {
    let items = state.snapshot().map_err(internal_error)?;
    let plan = RetrievalPlan::resolve(req.route, req.intent);
    let items = filter_items(&items, &req, &plan);
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
    let (plan, retrieval_order, items) = build_context(&state, &req)?;
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
    let items = state.snapshot().map_err(internal_error)?;
    let mut inbox = items
        .into_iter()
        .filter(|item| item.stage == MemoryStage::Candidate || item.status != MemoryStatus::Active)
        .filter(|item| {
            req.project
                .as_ref()
                .is_none_or(|project| item.project.as_ref() == Some(project))
        })
        .filter(|item| {
            req.namespace
                .as_ref()
                .is_none_or(|namespace| item.namespace.as_ref() == Some(namespace))
        })
        .map(|item| InboxMemoryItem {
            reasons: inbox_reasons(&item),
            item,
        })
        .filter(|entry| !entry.reasons.is_empty())
        .collect::<Vec<_>>();

    inbox.sort_by(|a, b| {
        inbox_score(&b.item, &plan)
            .partial_cmp(&inbox_score(&a.item, &plan))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.item.updated_at.cmp(&a.item.updated_at))
    });
    inbox.truncate(limit);

    Ok(Json(MemoryInboxResponse {
        route: plan.route,
        intent: plan.intent,
        items: inbox,
    }))
}

async fn get_explain(
    State(state): State<AppState>,
    Query(req): Query<ExplainMemoryRequest>,
) -> Result<Json<ExplainMemoryResponse>, (StatusCode, String)> {
    let plan = RetrievalPlan::resolve(req.route, req.intent);
    let item = state
        .store
        .get(req.id)
        .map_err(internal_error)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "memory item not found".to_string()))?;

    let reasons = explain_reasons(&item, &plan);
    let canonical = canonical_key(&item);
    let redundancy = redundancy_key(&item);

    Ok(Json(ExplainMemoryResponse {
        route: plan.route,
        intent: plan.intent,
        item,
        canonical_key: canonical,
        redundancy_key: redundancy,
        reasons,
    }))
}

fn build_context(
    state: &AppState,
    req: &ContextRequest,
) -> Result<(RetrievalPlan, Vec<MemoryScope>, Vec<MemoryItem>), (StatusCode, String)> {
    let plan = RetrievalPlan::resolve(req.route, req.intent);
    let limit = req.limit.unwrap_or(8).min(32);
    let max_chars = req.max_chars_per_item.unwrap_or(280).clamp(80, 2000);
    let items = state.snapshot().map_err(internal_error)?;
    let retrieval_order = plan.scopes();

    let mut scoped: Vec<MemoryItem> = Vec::new();
    for scope in retrieval_order.iter().copied() {
        let mut bucket: Vec<MemoryItem> = items
            .iter()
            .filter(|item| plan.allows(item.scope))
            .filter(|item| item.scope == scope)
            .filter(|item| item.status == MemoryStatus::Active)
            .filter(|item| match (&req.project, &item.project, scope) {
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
            context_score(b, req, &plan)
                .partial_cmp(&context_score(a, req, &plan))
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.updated_at.cmp(&a.updated_at))
        });

        scoped.extend(bucket);
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

fn filter_items(
    items: &[MemoryItem],
    req: &SearchMemoryRequest,
    plan: &RetrievalPlan,
) -> Vec<MemoryItem> {
    let query = req.query.as_ref().map(|q| q.to_ascii_lowercase());
    let limit = req.limit.unwrap_or(10).min(100);
    let max_chars = req.max_chars_per_item.unwrap_or(420).clamp(120, 4000);

    let mut filtered: Vec<MemoryItem> = items
        .iter()
        .filter(|item| req.scopes.is_empty() || req.scopes.contains(&item.scope))
        .filter(|item| plan.allows(item.scope))
        .filter(|item| req.kinds.is_empty() || req.kinds.contains(&item.kind))
        .filter(|item| req.statuses.is_empty() || req.statuses.contains(&item.status))
        .filter(|item| req.stages.is_empty() || req.stages.contains(&item.stage))
        .filter(|item| {
            req.project
                .as_ref()
                .is_none_or(|project| item.project.as_ref() == Some(project))
        })
        .filter(|item| {
            req.namespace
                .as_ref()
                .is_none_or(|namespace| item.namespace.as_ref() == Some(namespace))
        })
        .filter(|item| {
            req.source_agent
                .as_ref()
                .is_none_or(|agent| item.source_agent.as_ref() == Some(agent))
        })
        .filter(|item| {
            req.tags.is_empty()
                || req
                    .tags
                    .iter()
                    .all(|tag| item.tags.iter().any(|item_tag| item_tag == tag))
        })
        .filter(|item| {
            query.as_ref().is_none_or(|query| {
                item.content.to_ascii_lowercase().contains(query)
                    || item
                        .tags
                        .iter()
                        .any(|tag| tag.to_ascii_lowercase().contains(query))
            })
        })
        .cloned()
        .collect();

    filtered.sort_by(|a, b| {
        search_score(b, &query, plan)
            .partial_cmp(&search_score(a, &query, plan))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                b.confidence
                    .partial_cmp(&a.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| b.updated_at.cmp(&a.updated_at))
    });
    for item in &mut filtered {
        item.content = compact_content(&item.content, max_chars);
    }
    filtered.truncate(limit);
    filtered
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

fn context_score(item: &MemoryItem, req: &ContextRequest, plan: &RetrievalPlan) -> f32 {
    let mut score = item.confidence;

    score += match item.stage {
        MemoryStage::Canonical => 1.25,
        MemoryStage::Candidate => 0.25,
    };

    score += plan.scope_rank_bonus(item.scope);
    score += plan.intent_scope_bonus(item.scope);

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

    if item.status == MemoryStatus::Stale {
        score -= 1.5;
    }

    if item.status == MemoryStatus::Contested {
        score -= 2.0;
    }

    score -= age_penalty(item.updated_at);
    score
}

fn search_score(item: &MemoryItem, query: &Option<String>, plan: &RetrievalPlan) -> f32 {
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

fn age_penalty(updated_at: chrono::DateTime<Utc>) -> f32 {
    let age_days = (Utc::now() - updated_at).num_days().max(0) as f32;
    (age_days / 14.0).min(3.0)
}

fn inbox_score(item: &MemoryItem, plan: &RetrievalPlan) -> f32 {
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
    score -= age_penalty(item.updated_at) * 0.75;
    score
}

fn inbox_reasons(item: &MemoryItem) -> Vec<String> {
    let mut reasons = Vec::new();
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
    reasons
}

fn explain_reasons(item: &MemoryItem, plan: &RetrievalPlan) -> Vec<String> {
    let mut reasons = Vec::new();
    reasons.push(format!("route={}", format_route(plan.route)));
    reasons.push(format!("intent={}", format_intent(plan.intent)));
    reasons.push(format!("scope={}", format_scope(item.scope)));
    reasons.push(format!("stage={}", format_stage(item.stage)));
    reasons.push(format!("status={}", format_status(item.status)));
    if let Some(project) = &item.project {
        reasons.push(format!("project={project}"));
    }
    if let Some(namespace) = &item.namespace {
        reasons.push(format!("namespace={namespace}"));
    }
    if let Some(agent) = &item.source_agent {
        reasons.push(format!("source_agent={agent}"));
    }
    if let Some(path) = &item.source_path {
        reasons.push(format!("source_path={path}"));
    }
    if let Some(key) = &item.redundancy_key {
        reasons.push(format!("redundancy_key={key}"));
    }
    if !item.supersedes.is_empty() {
        reasons.push(format!("supersedes={}", item.supersedes.len()));
    }
    if item.status == MemoryStatus::Stale {
        reasons.push("needs_verification".to_string());
    }
    if item.stage == MemoryStage::Candidate {
        reasons.push("candidate_memory".to_string());
    }
    reasons
}

fn format_route(route: memd_schema::RetrievalRoute) -> &'static str {
    match route {
        memd_schema::RetrievalRoute::Auto => "auto",
        memd_schema::RetrievalRoute::LocalOnly => "local_only",
        memd_schema::RetrievalRoute::SyncedOnly => "synced_only",
        memd_schema::RetrievalRoute::ProjectOnly => "project_only",
        memd_schema::RetrievalRoute::GlobalOnly => "global_only",
        memd_schema::RetrievalRoute::LocalFirst => "local_first",
        memd_schema::RetrievalRoute::SyncedFirst => "synced_first",
        memd_schema::RetrievalRoute::ProjectFirst => "project_first",
        memd_schema::RetrievalRoute::GlobalFirst => "global_first",
        memd_schema::RetrievalRoute::All => "all",
    }
}

fn format_intent(intent: memd_schema::RetrievalIntent) -> &'static str {
    match intent {
        memd_schema::RetrievalIntent::General => "general",
        memd_schema::RetrievalIntent::CurrentTask => "current_task",
        memd_schema::RetrievalIntent::Decision => "decision",
        memd_schema::RetrievalIntent::Runbook => "runbook",
        memd_schema::RetrievalIntent::Topology => "topology",
        memd_schema::RetrievalIntent::Preference => "preference",
        memd_schema::RetrievalIntent::Fact => "fact",
        memd_schema::RetrievalIntent::Pattern => "pattern",
    }
}

fn format_scope(scope: MemoryScope) -> &'static str {
    match scope {
        MemoryScope::Local => "local",
        MemoryScope::Synced => "synced",
        MemoryScope::Project => "project",
        MemoryScope::Global => "global",
    }
}

fn format_stage(stage: MemoryStage) -> &'static str {
    match stage {
        MemoryStage::Candidate => "candidate",
        MemoryStage::Canonical => "canonical",
    }
}

fn format_status(status: MemoryStatus) -> &'static str {
    match status {
        MemoryStatus::Active => "active",
        MemoryStatus::Stale => "stale",
        MemoryStatus::Superseded => "superseded",
        MemoryStatus::Contested => "contested",
        MemoryStatus::Expired => "expired",
    }
}
