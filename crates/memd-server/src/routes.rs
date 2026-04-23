use super::errors::MemdError;
use super::*;
use memd_schema::PressureMetrics;

// B3-T6: atlas-at-recall 1-hop expansion flag. Default off.
fn atlas_recall_enabled() -> bool {
    match std::env::var("MEMD_RETRIEVAL_ATLAS_RECALL") {
        Ok(v) => {
            let v = v.trim().to_ascii_lowercase();
            matches!(v.as_str(), "1" | "true" | "on" | "yes")
        }
        Err(_) => false,
    }
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

fn intrinsic_rerank_enabled() -> bool {
    match std::env::var("MEMD_RETRIEVAL_RERANK") {
        Ok(value) => {
            let value = value.trim().to_ascii_lowercase();
            !matches!(value.as_str(), "0" | "false" | "off" | "no")
        }
        Err(_) => true,
    }
}

const INTRINSIC_RERANK_WINDOW: usize = 20;

pub(crate) fn intrinsic_rerank_search_candidates(
    items: &[MemoryViewItem],
    query: &str,
    base_ranks: &[(Uuid, f64)],
) -> Vec<(Uuid, f64)> {
    let query = query.trim();
    if query.is_empty() || base_ranks.len() < 2 {
        return base_ranks.to_vec();
    }

    let by_id: std::collections::HashMap<Uuid, &MemoryItem> = items
        .iter()
        .map(|entry| (entry.item.id, &entry.item))
        .collect();
    let head_len = base_ranks.len().min(INTRINSIC_RERANK_WINDOW);
    let (head, tail) = base_ranks.split_at(head_len);

    let mut reranked = head
        .iter()
        .enumerate()
        .map(|(index, (id, _base_score))| {
            let base_norm = 1.0 / (10.0 + index as f64);
            let local_score = by_id
                .get(id)
                .map(|item| intrinsic_local_rerank_score(item, query))
                .unwrap_or(0.0);
            let blended = local_score * 0.82 + base_norm * 0.18;
            (*id, blended, index)
        })
        .collect::<Vec<_>>();

    reranked.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.2.cmp(&b.2)));

    let mut ordered = reranked
        .into_iter()
        .map(|(id, score, _)| (id, score))
        .collect::<Vec<_>>();
    ordered.extend_from_slice(tail);
    ordered
}

async fn rerank_search_candidates(
    state: &AppState,
    items: &[MemoryViewItem],
    query: &str,
    base_ranks: &[(Uuid, f64)],
) -> Vec<(Uuid, f64)> {
    let query = query.trim();
    if query.is_empty() || base_ranks.len() < 2 {
        return base_ranks.to_vec();
    }

    let by_id: std::collections::HashMap<Uuid, &MemoryItem> = items
        .iter()
        .map(|entry| (entry.item.id, &entry.item))
        .collect();
    let head_len = base_ranks.len().min(INTRINSIC_RERANK_WINDOW);
    let (head, tail) = base_ranks.split_at(head_len);

    if let Some(rag) = state.rag.as_deref() {
        let candidates = head
            .iter()
            .filter_map(|(id, _)| by_id.get(id).map(|item| (*id, rerank_item_haystack(item))))
            .collect::<Vec<_>>();
        if !candidates.is_empty() {
            match crate::rag_bridge::rerank_candidates(rag, query, &candidates, candidates.len())
                .await
            {
                Ok(reranked) if !reranked.is_empty() => {
                    let mut ordered = reranked;
                    let seen = ordered
                        .iter()
                        .map(|(id, _)| *id)
                        .collect::<std::collections::HashSet<_>>();
                    for (index, (id, _base_score)) in head.iter().enumerate() {
                        if !seen.contains(id) {
                            ordered.push((*id, 1.0 / (10.0 + index as f64)));
                        }
                    }
                    ordered.extend_from_slice(tail);
                    return ordered;
                }
                Ok(_) => {}
                Err(error) => {
                    warn!(error = %format_args!("{error:#}"), "sidecar rerank failed; falling back to intrinsic rerank")
                }
            }
        }
    }

    intrinsic_rerank_search_candidates(items, query, base_ranks)
}

fn intrinsic_local_rerank_score(item: &MemoryItem, query: &str) -> f64 {
    let query_terms = rerank_tokenize(query);
    if query_terms.is_empty() {
        return 0.0;
    }
    let query_term_set = query_terms
        .iter()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    let query_keywords = rerank_keyword_tokens(&query_terms);
    let query_bigrams = rerank_query_bigrams(&query_terms);
    let haystack = rerank_item_haystack(item);
    let record_tokens = rerank_tokenize(&haystack);
    let name_tokens = rerank_extract_name_tokens(&item.content);
    let record_term_set = record_tokens
        .iter()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    let overlap = query_term_set.intersection(&record_term_set).count() as f64;
    let lexical = overlap / query_term_set.len().max(1) as f64;
    let token_frequency =
        record_tokens
            .iter()
            .fold(std::collections::HashMap::new(), |mut acc, token| {
                *acc.entry(token.as_str()).or_insert(0usize) += 1;
                acc
            });
    let bm25ish = if query_keywords.is_empty() {
        0.0
    } else {
        query_keywords
            .iter()
            .map(|keyword| {
                let frequency = token_frequency.get(keyword.as_str()).copied().unwrap_or(0) as f64;
                if frequency == 0.0 {
                    0.0
                } else {
                    frequency / (frequency + 1.2)
                }
            })
            .sum::<f64>()
            / query_keywords.len() as f64
    };
    let semantic = rerank_cosine_similarity(
        &rerank_semantic_terms(query),
        &rerank_semantic_terms(&haystack),
    );
    let phrase_bonus = if query_terms.len() >= 2 && haystack.contains(&query_terms.join(" ")) {
        0.35
    } else {
        0.0
    };
    let keyword_bonus = if query_keywords.is_empty() {
        0.0
    } else {
        query_keywords
            .iter()
            .filter(|keyword| haystack.contains(keyword.as_str()))
            .count() as f64
            / query_keywords.len() as f64
    };
    let bigram_bonus = if query_bigrams.is_empty() {
        0.0
    } else {
        query_bigrams
            .iter()
            .filter(|bigram| haystack.contains(bigram.as_str()))
            .count() as f64
            / query_bigrams.len() as f64
    };
    let name_bonus = if query_keywords.is_empty() {
        0.0
    } else {
        query_keywords
            .iter()
            .filter(|keyword| name_tokens.contains(keyword.as_str()))
            .count() as f64
            / query_keywords.len() as f64
    };
    let path_lower = item
        .source_path
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let path_bonus = if query_keywords.is_empty() {
        0.0
    } else {
        query_keywords
            .iter()
            .filter(|keyword| path_lower.contains(keyword.as_str()))
            .count() as f64
            / query_keywords.len() as f64
    };
    let tag_bonus = if query_keywords.is_empty() {
        0.0
    } else {
        query_keywords
            .iter()
            .filter(|keyword| {
                item.tags
                    .iter()
                    .any(|tag| tag.to_ascii_lowercase().contains(keyword.as_str()))
            })
            .count() as f64
            / query_keywords.len() as f64
    };

    let score = lexical * 0.22
        + bm25ish * 0.18
        + semantic * 0.25
        + phrase_bonus
        + keyword_bonus * 0.20
        + bigram_bonus * 0.18
        + name_bonus * 0.08
        + path_bonus * 0.14
        + tag_bonus * 0.10;
    score.clamp(0.0, 1.0)
}

fn rerank_item_haystack(item: &MemoryItem) -> String {
    let mut haystack = item.content.to_ascii_lowercase();
    haystack.push(' ');
    haystack.push_str(format!("{:?}", item.kind).to_ascii_lowercase().as_str());
    haystack.push(' ');
    haystack.push_str(&item.tags.join(" ").to_ascii_lowercase());
    if let Some(path) = item.source_path.as_deref() {
        haystack.push(' ');
        haystack.push_str(&path.to_ascii_lowercase());
    }
    if let Some(agent) = item.source_agent.as_deref() {
        haystack.push(' ');
        haystack.push_str(&agent.to_ascii_lowercase());
    }
    haystack
}

fn rerank_tokenize(text: &str) -> Vec<String> {
    text.split(|ch: char| !ch.is_alphanumeric())
        .filter(|token| token.len() > 1)
        .map(|token| token.to_ascii_lowercase())
        .collect()
}

fn rerank_keyword_tokens(query_terms: &[String]) -> Vec<String> {
    let stop_words = [
        "what", "when", "where", "who", "how", "which", "did", "do", "was", "were", "have", "has",
        "had", "is", "are", "the", "a", "an", "my", "me", "i", "you", "your", "their", "it", "its",
        "in", "on", "at", "to", "for", "of", "with", "by", "from", "ago", "last", "that", "this",
        "there", "about", "get", "got", "give", "gave", "buy", "bought", "made", "make", "said",
        "would", "could", "should", "might", "can", "will", "shall", "kind", "type", "like",
        "prefer", "enjoy", "think", "feel",
    ]
    .into_iter()
    .collect::<std::collections::BTreeSet<_>>();
    query_terms
        .iter()
        .filter(|token| token.len() >= 3 && !stop_words.contains(token.as_str()))
        .cloned()
        .collect()
}

fn rerank_query_bigrams(query_terms: &[String]) -> Vec<String> {
    query_terms
        .windows(2)
        .map(|pair| format!("{} {}", pair[0], pair[1]))
        .collect()
}

fn rerank_semantic_terms(text: &str) -> Vec<String> {
    let tokens = rerank_tokenize(text);
    let mut features = Vec::new();
    for token in &tokens {
        features.push(format!("tok:{token}"));
        if token.len() >= 4 {
            for trigram in token.as_bytes().windows(3) {
                if let Ok(fragment) = std::str::from_utf8(trigram) {
                    features.push(format!("tri:{fragment}"));
                }
            }
        }
    }
    for pair in tokens.windows(2) {
        features.push(format!("bi:{}_{}", pair[0], pair[1]));
    }
    features
}

fn rerank_cosine_similarity(left: &[String], right: &[String]) -> f64 {
    if left.is_empty() || right.is_empty() {
        return 0.0;
    }
    let left_freq = rerank_feature_frequency(left);
    let right_freq = rerank_feature_frequency(right);
    let mut dot = 0.0f64;
    for (feature, left_weight) in &left_freq {
        if let Some(right_weight) = right_freq.get(feature) {
            dot += left_weight * right_weight;
        }
    }
    if dot == 0.0 {
        return 0.0;
    }
    let left_norm = left_freq
        .values()
        .map(|weight| weight * weight)
        .sum::<f64>()
        .sqrt();
    let right_norm = right_freq
        .values()
        .map(|weight| weight * weight)
        .sum::<f64>()
        .sqrt();
    if left_norm == 0.0 || right_norm == 0.0 {
        0.0
    } else {
        dot / (left_norm * right_norm)
    }
}

fn rerank_feature_frequency(features: &[String]) -> std::collections::HashMap<&str, f64> {
    let mut frequency = std::collections::HashMap::new();
    for feature in features {
        *frequency.entry(feature.as_str()).or_insert(0.0) += 1.0;
    }
    frequency
}

fn rerank_extract_name_tokens(content: &str) -> std::collections::BTreeSet<String> {
    content
        .split(|ch: char| !ch.is_ascii_alphabetic())
        .filter(|token| {
            token.len() >= 3
                && token
                    .chars()
                    .next()
                    .is_some_and(|first| first.is_ascii_uppercase())
        })
        .map(|token| token.to_ascii_lowercase())
        .collect()
}

pub(crate) async fn healthz(
    State(state): State<AppState>,
) -> Result<Json<HealthResponse>, (StatusCode, String)> {
    // C2: opportunistic GC on health check — keeps expired count accurate.
    let _ = state.store.gc_expired_items(3600);
    let items = state.store.count().map_err(internal_error)?;

    // Compute pressure metrics from store
    let all_items = state.store.list().unwrap_or_default();
    let inbox_count = all_items
        .iter()
        .filter(|i| i.stage == MemoryStage::Candidate || i.status != MemoryStatus::Active)
        .filter(|i| i.status != MemoryStatus::Expired)
        .count();
    let candidates = all_items
        .iter()
        .filter(|i| i.stage == MemoryStage::Candidate)
        .count();
    let stale = all_items
        .iter()
        .filter(|i| i.status == MemoryStatus::Stale)
        .count();
    let expired = all_items
        .iter()
        .filter(|i| i.status == MemoryStatus::Expired)
        .count();
    let rag = state.rag_health_surface().await;
    let atlas = crate::status::atlas_health_surface(&state, items).map_err(internal_error)?;

    Ok(Json(HealthResponse {
        status: "ok".to_string(),
        items,
        eval_score: None,
        pressure: Some(PressureMetrics {
            inbox: inbox_count,
            candidates,
            stale,
            expired,
        }),
        rag: Some(rag),
        atlas: Some(atlas),
    }))
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
        return Err(MemdError::validation("content", "must not be empty").into_wire());
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
        return Err(MemdError::validation("content", "must not be empty").into_wire());
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
        lane: req.lane,
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

pub(crate) async fn correct_memory(
    State(state): State<AppState>,
    Json(req): Json<CorrectMemoryRequest>,
) -> Result<Json<CorrectMemoryResponse>, (StatusCode, String)> {
    let response = repair::correct_item(&state, req)?;
    Ok(Json(response))
}

pub(crate) async fn get_working_memory(
    State(state): State<AppState>,
    Query(req): Query<WorkingMemoryRequest>,
) -> Result<Json<WorkingMemoryResponse>, (StatusCode, String)> {
    // K2.6: record wall-clock ms into the process-global histogram so
    // /api/diagnostics/latency and HarnessStatus.latency_p95_ms reflect
    // the true tail of working-memory retrieval.
    let started = std::time::Instant::now();
    let result = working::working_memory(&state, req);
    state
        .latency
        .record_ms(started.elapsed().as_millis() as u64);
    let response = result?;
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

#[derive(Deserialize)]
pub(crate) struct GetSearchQuery {
    pub(crate) query: Option<String>,
    pub(crate) tag: Option<String>,
    pub(crate) kind: Option<MemoryKind>,
    pub(crate) status: Option<MemoryStatus>,
    pub(crate) stage: Option<MemoryStage>,
    pub(crate) project: Option<String>,
    pub(crate) namespace: Option<String>,
    pub(crate) workspace: Option<String>,
    pub(crate) limit: Option<usize>,
}

// K2.4: GET surface for ergonomic curl/dashboard search.
// Delegates to the same filter pipeline as POST /memory/search so ranking,
// FTS, and retrieval-feedback stay consistent between the two entry points.
pub(crate) async fn search_memory_get(
    state: State<AppState>,
    Query(q): Query<GetSearchQuery>,
) -> Result<Json<SearchMemoryResponse>, (StatusCode, String)> {
    let req = SearchMemoryRequest {
        query: q.query,
        tags: q.tag.into_iter().collect(),
        kinds: q.kind.into_iter().collect(),
        statuses: q.status.into_iter().collect(),
        stages: q.stage.into_iter().collect(),
        project: q.project,
        namespace: q.namespace,
        workspace: q.workspace,
        limit: q.limit,
        ..SearchMemoryRequest::default()
    };
    search_memory(state, Json(req)).await
}

pub(crate) async fn search_memory(
    State(state): State<AppState>,
    Json(req): Json<SearchMemoryRequest>,
) -> Result<Json<SearchMemoryResponse>, (StatusCode, String)> {
    let policy = working::memory_policy_snapshot();
    let feedback_limit = policy.retrieval_feedback.max_items_per_request;
    // B3-part2 prereq: scope snapshot to project+namespace when set so
    // per-question bench namespaces don't trigger a full-corpus scan.
    let snapshot = state
        .snapshot_for_scope(req.project.as_deref(), req.namespace.as_deref())
        .map_err(internal_error)?;
    let region_member_ids = resolve_region_member_filter(
        &state,
        req.region.as_deref(),
        req.project.as_deref(),
        req.namespace.as_deref(),
    )
    .map_err(internal_error)?;
    let mut items = enrich_with_entities(&state, snapshot).map_err(internal_error)?;
    if let Some(allowed_ids) = region_member_ids.as_ref() {
        items.retain(|entry| allowed_ids.contains(&entry.item.id));
    }
    let plan = RetrievalPlan::resolve(req.route, req.intent);
    // B3-T2: sanitize + atlas-synonym expand before FTS.
    let mut fts_ranks = req
        .query
        .as_ref()
        .and_then(|q| {
            let sanitized = crate::query_sanitize::sanitize_query(q);
            let aliases = state
                .store
                .entity_aliases_for_query(&sanitized.clean, 4)
                .unwrap_or_default();
            let fts_expr = crate::query_sanitize::build_fts_match(&sanitized.clean, &aliases);
            state.store.fts_search(&fts_expr, 100).ok()
        })
        .unwrap_or_default();
    if let Some(allowed_ids) = region_member_ids.as_ref() {
        fts_ranks.retain(|(id, _)| allowed_ids.contains(id));
    }

    // B3-T6: atlas-at-recall 1-hop entity expansion (item-space). Top K
    // FTS hits seed a 1-hop neighbor lookup; neighbors are injected into
    // fts_ranks with a small score bonus so they survive filter_items.
    // Gate: MEMD_RETRIEVAL_ATLAS_RECALL=1 (default off).
    if atlas_recall_enabled() && !fts_ranks.is_empty() {
        let seed_k = fts_ranks.len().min(5);
        let seeds: Vec<uuid::Uuid> = fts_ranks.iter().take(seed_k).map(|(id, _)| *id).collect();
        let neighbors = state.store.one_hop_neighbors_for_items(&seeds, 10);
        if !neighbors.is_empty() {
            // Bonus is small (0.15) so direct FTS matches retain priority.
            let tail_score = fts_ranks
                .last()
                .map(|(_, s)| *s * 0.5)
                .unwrap_or(0.15)
                .max(0.15);
            let existing: std::collections::HashSet<uuid::Uuid> =
                fts_ranks.iter().map(|(id, _)| *id).collect();
            for nid in neighbors {
                if !existing.contains(&nid) {
                    fts_ranks.push((nid, tail_score));
                }
            }
        }
    }
    if state.rag.is_some() && rag_dense_enabled() {
        match state.rag_dense_candidates(&req).await {
            Ok(dense) if !dense.is_empty() => {
                let tail_score = fts_ranks
                    .last()
                    .map(|(_, score)| *score * 0.5)
                    .unwrap_or(0.15)
                    .max(0.15);
                let mut existing: std::collections::HashSet<Uuid> =
                    fts_ranks.iter().map(|(id, _)| *id).collect();
                for (id, _) in dense {
                    if region_member_ids
                        .as_ref()
                        .is_some_and(|allowed_ids| !allowed_ids.contains(&id))
                    {
                        continue;
                    }
                    if existing.insert(id) {
                        fts_ranks.push((id, tail_score));
                    }
                }
            }
            Ok(_) => {}
            Err(error) => warn!(error = %format_args!("{error:#}"), "rag dense retrieval failed"),
        }
    }

    // B3-Part2: intrinsic dense blend (in-process fastembed). Gated via
    // MEMD_INTRINSIC_DENSE=1. Scopes the vector scan to the same project+
    // namespace slice as `snapshot_for_scope`, so per-question bench
    // namespaces never trigger a global-corpus vector scan. Dense scores
    // replace fts scores rather than tail-append, so purely-semantic matches
    // (zero lexical overlap) surface into the top-K.
    if let Some(embedder) = state.embedder.as_deref()
        && crate::embed::intrinsic_dense_enabled()
        && let Some(query_text) = req
            .query
            .as_deref()
            .map(str::trim)
            .filter(|q| !q.is_empty())
    {
        match embedder.embed_query_normalized(query_text) {
            Ok(q_vec) => {
                match state.store.list_vectors_for_scope(
                    req.project.as_deref(),
                    req.namespace.as_deref(),
                    embedder.model_code(),
                ) {
                    Ok(candidates) if !candidates.is_empty() => {
                        // Per-chunk scoring; one memory_id can appear multiple
                        // times if content was chunked on store. Group by
                        // memory_id taking MAX so a single strong chunk
                        // lifts the session it belongs to.
                        let mut by_id: std::collections::HashMap<Uuid, f64> =
                            std::collections::HashMap::with_capacity(candidates.len());
                        for (id, bytes) in candidates {
                            let v = crate::embed::bytes_to_vec(&bytes);
                            let score = crate::embed::cosine_on_unit(&q_vec, &v) as f64;
                            let entry = by_id.entry(id).or_insert(f64::NEG_INFINITY);
                            if score > *entry {
                                *entry = score;
                            }
                        }
                        let mut scored: Vec<(Uuid, f64)> = by_id.into_iter().collect();
                        scored.sort_by(|a, b| {
                            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
                        });
                        scored.truncate(200);
                        let existing: std::collections::HashMap<Uuid, f64> =
                            fts_ranks.iter().copied().collect();
                        let mut blended: Vec<(Uuid, f64)> =
                            Vec::with_capacity(scored.len() + fts_ranks.len());
                        let mut seen: std::collections::HashSet<Uuid> =
                            std::collections::HashSet::new();
                        for (id, dense_score) in scored {
                            if region_member_ids
                                .as_ref()
                                .is_some_and(|allowed_ids| !allowed_ids.contains(&id))
                            {
                                continue;
                            }
                            seen.insert(id);
                            let fts_score = existing.get(&id).copied().unwrap_or(0.0);
                            // Heavy dense weight — the gate exists because
                            // FTS alone is at 0.828, and we need pure
                            // semantic hits to win when lexical overlap is
                            // absent. Fts contributes as a tie-breaker.
                            let blended_score = dense_score + 0.1 * fts_score;
                            blended.push((id, blended_score));
                        }
                        for (id, fts_score) in fts_ranks.iter() {
                            if region_member_ids
                                .as_ref()
                                .is_some_and(|allowed_ids| !allowed_ids.contains(id))
                            {
                                continue;
                            }
                            if !seen.contains(id) {
                                blended.push((*id, 0.1 * *fts_score));
                            }
                        }
                        blended.sort_by(|a, b| {
                            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
                        });
                        fts_ranks = blended;
                    }
                    Ok(_) => {}
                    Err(error) => {
                        warn!(error = %format_args!("{error:#}"), "list_vectors_for_scope failed")
                    }
                }
            }
            Err(error) => warn!(error = %format_args!("{error:#}"), "query embed failed"),
        }
    }
    if intrinsic_rerank_enabled()
        && let Some(query_text) = req
            .query
            .as_deref()
            .map(str::trim)
            .filter(|q| !q.is_empty())
        && fts_ranks.len() > 1
    {
        fts_ranks = rerank_search_candidates(&state, &items, query_text, &fts_ranks).await;
    }
    let items = filter_items(&items, &req, &plan, &fts_ranks);
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
    // C2: opportunistic GC — remove expired items past 1h grace on every wake.
    let _ = state.store.gc_expired_items(3600);
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
    let response = crate::working::working_memory(&state, req).map_err(|e| e)?;

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
