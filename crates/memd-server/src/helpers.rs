use super::*;

pub(crate) fn enrich_with_entities(
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

pub(crate) fn build_context(
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
        .filter(|entry| matches_requested_project(&req.project, &entry.item))
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

pub(crate) fn apply_agent_profile_defaults(
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

pub(crate) fn filter_items(
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
        .filter(|entry| matches_requested_project(&req.project, &entry.item))
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
            req.project.as_ref(),
            req.namespace.as_ref(),
            plan,
        )
        .partial_cmp(&search_score(
            &a.item,
            a.entity.as_ref(),
            a.source_trust_score,
            &query,
            req.project.as_ref(),
            req.namespace.as_ref(),
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

pub(crate) fn compact_content(content: &str, max_chars: usize) -> String {
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

pub(crate) fn event_type_for_stage(stage: MemoryStage) -> &'static str {
    match stage {
        MemoryStage::Candidate => "candidate_created",
        MemoryStage::Canonical => "canonical_created",
    }
}

pub(crate) fn entity_context_frame(
    entity: &MemoryEntityRecord,
    item: &MemoryItem,
) -> MemoryContextFrame {
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

pub(crate) fn consolidation_content(
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

pub(crate) fn consolidation_scope(entity: &MemoryEntityRecord) -> MemoryScope {
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

pub(crate) fn consolidation_kind(entity_type: &str) -> MemoryKind {
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

pub(crate) fn consolidation_tags(entity: &MemoryEntityRecord, event_count: usize) -> Vec<String> {
    let mut tags = entity.tags.clone();
    tags.push("consolidated".to_string());
    tags.push(format!("events:{}", event_count));
    tags.push(entity.entity_type.clone());
    tags.sort();
    tags.dedup();
    tags
}

pub(crate) fn internal_error(error: anyhow::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

pub(crate) fn compact_record(item: &MemoryItem) -> String {
    let mut parts = Vec::new();
    parts.push(format!("id={}", item.id));
    parts.push(format!("stage={}", enum_label_stage(item.stage)));
    parts.push(format!("scope={}", enum_label_scope(item.scope)));
    parts.push(format!("kind={}", enum_label_kind(item.kind)));
    parts.push(format!("status={}", enum_label_status(item.status)));

    if let Some(project) = &item.project
        && !project.is_empty() {
            parts.push(format!("project={}", sanitize_value(project)));
        }
    if let Some(namespace) = &item.namespace
        && !namespace.is_empty() {
            parts.push(format!("ns={}", sanitize_value(namespace)));
        }
    if let Some(workspace) = &item.workspace
        && !workspace.is_empty() {
            parts.push(format!("ws={}", sanitize_value(workspace)));
        }
    parts.push(format!("vis={}", enum_label_visibility(item.visibility)));
    if let Some(branch) = &item.belief_branch
        && !branch.is_empty() {
            parts.push(format!("belief_branch={}", sanitize_value(branch)));
        }
    if let Some(agent) = &item.source_agent
        && !agent.is_empty() {
            parts.push(format!("agent={}", sanitize_value(agent)));
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

pub(crate) fn enum_label_route(route: RetrievalRoute) -> &'static str {
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

pub(crate) fn enum_label_intent(intent: RetrievalIntent) -> &'static str {
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

pub(crate) fn associative_recall_score(
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
        0.18
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

pub(crate) fn associative_recall_reasons(
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


pub(crate) fn sanitize_value(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace('|', "/")
}

pub(crate) fn enum_label_kind(kind: MemoryKind) -> &'static str {
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

pub(crate) fn enum_label_scope(scope: MemoryScope) -> &'static str {
    match scope {
        MemoryScope::Local => "local",
        MemoryScope::Synced => "synced",
        MemoryScope::Project => "project",
        MemoryScope::Global => "global",
    }
}

pub(crate) fn enum_label_visibility(visibility: MemoryVisibility) -> &'static str {
    match visibility {
        MemoryVisibility::Private => "private",
        MemoryVisibility::Workspace => "workspace",
        MemoryVisibility::Public => "public",
    }
}

pub(crate) fn enum_label_stage(stage: MemoryStage) -> &'static str {
    match stage {
        MemoryStage::Candidate => "candidate",
        MemoryStage::Canonical => "canonical",
    }
}

pub(crate) fn enum_label_status(status: MemoryStatus) -> &'static str {
    match status {
        MemoryStatus::Active => "active",
        MemoryStatus::Stale => "stale",
        MemoryStatus::Superseded => "superseded",
        MemoryStatus::Contested => "contested",
        MemoryStatus::Expired => "expired",
    }
}

pub(crate) fn context_score(
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
    score += project_scope_bonus(item, req.project.as_ref(), None);

    if let Some(project) = &req.project
        && item.project.as_ref() == Some(project) {
            score += 1.9;
        }

    if let Some(agent) = &req.agent
        && item.source_agent.as_ref() == Some(agent) {
            score += 0.75;
        }

    score += workspace_rank_adjustment(req.workspace.as_ref(), item.workspace.as_ref());
    score += durable_truth_rank_adjustment(item);

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

pub(crate) fn search_score(
    item: &MemoryItem,
    entity: Option<&MemoryEntityRecord>,
    source_trust_score: f32,
    query: &Option<String>,
    requested_project: Option<&String>,
    requested_namespace: Option<&String>,
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
    score += project_scope_bonus(item, requested_project, requested_namespace) * 0.9;
    score += trust_rank_adjustment(source_trust_score) * 0.8;
    score += epistemic_rank_adjustment(item) * 0.85;
    score += durable_truth_rank_adjustment(item) * 0.9;

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

pub(crate) fn trust_rank_adjustment(source_trust_score: f32) -> f32 {
    if source_trust_score < 0.35 {
        -1.2
    } else if source_trust_score < 0.5 {
        -0.75
    } else if source_trust_score < 0.6 {
        -0.4
    } else if source_trust_score >= 0.9 {
        0.3
    } else if source_trust_score >= 0.75 {
        0.18
    } else {
        0.0
    }
}

pub(crate) fn epistemic_rank_adjustment(item: &MemoryItem) -> f32 {
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

pub(crate) fn durable_truth_rank_adjustment(item: &MemoryItem) -> f32 {
    let mut score = 0.0;

    if item.tags.iter().any(|tag| tag == "correction") {
        score += 1.4;
    }
    if item.tags.iter().any(|tag| tag == "project_fact") {
        score += 1.0;
    }
    if item
        .source_system
        .as_deref()
        .is_some_and(|value| value == "correction")
    {
        score += 0.8;
    }

    let content = item.content.to_ascii_lowercase();
    if content.starts_with("corrected fact:") {
        score += 1.2;
    } else if content.starts_with("remembered project fact:") {
        score += 0.8;
    }

    score
}

pub(crate) fn workspace_rank_adjustment(
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

pub(crate) fn age_penalty(updated_at: chrono::DateTime<Utc>) -> f32 {
    let age_days = (Utc::now() - updated_at).num_days().max(0) as f32;
    (age_days / 21.0).min(2.0)
}

pub(crate) fn inbox_score(
    item: &MemoryItem,
    entity: Option<&MemoryEntityRecord>,
    requested_project: Option<&String>,
    requested_namespace: Option<&String>,
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
    score += project_scope_bonus(item, requested_project, requested_namespace);
    score -= age_penalty(item.updated_at) * 0.75;
    score
}

pub(crate) fn matches_requested_project(
    requested_project: &Option<String>,
    item: &MemoryItem,
) -> bool {
    let Some(project) = requested_project else {
        return true;
    };

    match item.project.as_ref() {
        Some(item_project) => item_project == project,
        None => item.scope == MemoryScope::Global,
    }
}

pub(crate) fn project_scope_bonus(
    item: &MemoryItem,
    requested_project: Option<&String>,
    requested_namespace: Option<&String>,
) -> f32 {
    let Some(project) = requested_project else {
        return 0.0;
    };

    let mut bonus = 0.0;
    match item.project.as_ref() {
        Some(item_project) if item_project == project => {
            bonus += 1.25;
            if item.scope == MemoryScope::Project {
                bonus += 0.45;
            }
        }
        None if item.scope == MemoryScope::Global => {
            bonus += 0.15;
        }
        _ => {
            bonus -= 1.0;
        }
    }

    if let Some(namespace) = requested_namespace
        && item.namespace.as_ref() == Some(namespace) {
            bonus += 0.2;
        }

    bonus
}

pub(crate) fn entity_attention_bonus(
    item: &MemoryItem,
    entity: Option<&MemoryEntityRecord>,
) -> f32 {
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
                bonus += 0.45;
            }
            if context.namespace.as_ref() == item.namespace.as_ref() {
                bonus += 0.15;
            }
            if context.agent.as_ref() == item.source_agent.as_ref() {
                bonus += 0.08;
            }
            bonus
        })
        .unwrap_or(0.0);

    salience * 0.75 + rehearsal * 0.08 + recency * 0.1 + state_alignment
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routing::RetrievalPlan;

    fn sample_item(content: &str, tags: Vec<&str>, source_system: Option<&str>) -> MemoryItem {
        MemoryItem {
            id: uuid::Uuid::new_v4(),
            content: content.to_string(),
            redundancy_key: None,
            belief_branch: None,
            preferred: true,
            kind: MemoryKind::Fact,
            scope: MemoryScope::Project,
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: MemoryVisibility::Workspace,
            source_agent: Some("codex".to_string()),
            source_system: source_system.map(|value| value.to_string()),
            source_path: None,
            source_quality: Some(SourceQuality::Canonical),
            confidence: 0.95,
            ttl_seconds: None,
            created_at: Utc::now(),
            status: MemoryStatus::Active,
            stage: MemoryStage::Canonical,
            last_verified_at: Some(Utc::now()),
            supersedes: Vec::new(),
            updated_at: Utc::now(),
            tags: tags.into_iter().map(|value| value.to_string()).collect(),
        }
    }

    fn sample_entity(
        project: Option<&str>,
        namespace: Option<&str>,
        salience_score: f32,
        rehearsal_count: u64,
        last_accessed_at: Option<chrono::DateTime<Utc>>,
    ) -> MemoryEntityRecord {
        MemoryEntityRecord {
            id: uuid::Uuid::new_v4(),
            entity_type: "repo".to_string(),
            aliases: vec!["memd".to_string()],
            current_state: Some("working memory".to_string()),
            state_version: 1,
            confidence: 0.9,
            salience_score,
            rehearsal_count,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_accessed_at,
            last_seen_at: Some(Utc::now()),
            valid_from: Some(Utc::now()),
            valid_to: None,
            tags: vec!["project".to_string()],
            context: Some(MemoryContextFrame {
                at: Some(Utc::now()),
                project: project.map(|value| value.to_string()),
                namespace: namespace.map(|value| value.to_string()),
                workspace: Some("core".to_string()),
                repo: Some("memd".to_string()),
                host: Some("laptop".to_string()),
                branch: Some("main".to_string()),
                agent: Some("codex".to_string()),
                location: Some("/tmp/memd".to_string()),
            }),
        }
    }

    #[test]
    fn corrected_fact_ranks_above_resume_noise_for_search() {
        let plan = RetrievalPlan::resolve(
            Some(RetrievalRoute::LocalFirst),
            Some(RetrievalIntent::CurrentTask),
        );
        let corrected = sample_item(
            "corrected fact: roadmap status is not proof of working memory recall",
            vec!["correction"],
            Some("correction"),
        );
        let noisy = MemoryItem {
            scope: MemoryScope::Synced,
            kind: MemoryKind::Status,
            content: "resume state noise: synced session snapshot".to_string(),
            tags: vec!["resume_state".to_string(), "session_state".to_string()],
            source_system: Some("memd-resume-state".to_string()),
            ..sample_item(
                "resume state noise: synced session snapshot",
                vec![],
                Some("memd"),
            )
        };

        assert!(
            search_score(&corrected, None, 0.95, &None, None, None, &plan)
                > search_score(&noisy, None, 0.95, &None, None, None, &plan)
        );
    }

    #[test]
    fn requested_project_blocks_foreign_project_items_even_for_global_scope() {
        let requested = Some("demo-b".to_string());
        let foreign_global = MemoryItem {
            scope: MemoryScope::Global,
            project: Some("demo-a".to_string()),
            ..sample_item("foreign global", vec![], Some("memd"))
        };
        let shared_global = MemoryItem {
            scope: MemoryScope::Global,
            project: None,
            ..sample_item("shared global", vec![], Some("memd"))
        };
        let local_project = MemoryItem {
            scope: MemoryScope::Project,
            project: Some("demo-b".to_string()),
            ..sample_item("local project", vec![], Some("memd"))
        };

        assert!(!matches_requested_project(&requested, &foreign_global));
        assert!(matches_requested_project(&requested, &shared_global));
        assert!(matches_requested_project(&requested, &local_project));
        assert!(matches_requested_project(&None, &foreign_global));
    }

    #[test]
    fn project_scope_bonus_prefers_project_over_shared_global_context() {
        let requested_project = Some("demo-b".to_string());
        let requested_namespace = Some("main".to_string());
        let project_item = MemoryItem {
            scope: MemoryScope::Project,
            project: Some("demo-b".to_string()),
            namespace: Some("main".to_string()),
            ..sample_item("project item", vec![], Some("memd"))
        };
        let shared_global = MemoryItem {
            scope: MemoryScope::Global,
            project: None,
            namespace: Some("main".to_string()),
            ..sample_item("shared global", vec![], Some("memd"))
        };
        let unrelated_global = MemoryItem {
            scope: MemoryScope::Global,
            project: Some("demo-a".to_string()),
            namespace: Some("main".to_string()),
            ..sample_item("unrelated global", vec![], Some("memd"))
        };

        assert!(
            project_scope_bonus(
                &project_item,
                requested_project.as_ref(),
                requested_namespace.as_ref()
            ) > project_scope_bonus(
                &shared_global,
                requested_project.as_ref(),
                requested_namespace.as_ref()
            )
        );
        assert!(
            project_scope_bonus(
                &shared_global,
                requested_project.as_ref(),
                requested_namespace.as_ref()
            ) > project_scope_bonus(
                &unrelated_global,
                requested_project.as_ref(),
                requested_namespace.as_ref()
            )
        );
    }

    #[test]
    fn entity_attention_bonus_prefers_project_aligned_recent_entity_without_popularity_spillover() {
        let now = Utc::now();
        let item = sample_item("entity bonus item", vec!["project"], Some("memd"));
        let aligned = sample_entity(Some("demo"), Some("main"), 0.85, 2, Some(now));
        let popular_but_unaligned = sample_entity(Some("other"), Some("main"), 1.0, 20, Some(now));

        assert!(
            entity_attention_bonus(&item, Some(&aligned)) > entity_attention_bonus(&item, None)
        );
        assert!(
            entity_attention_bonus(&item, Some(&aligned))
                > entity_attention_bonus(&item, Some(&popular_but_unaligned))
        );
    }

    #[test]
    fn trust_and_age_weights_keep_durable_project_truth_ahead_of_fresh_noise() {
        let plan = RetrievalPlan::resolve(
            Some(RetrievalRoute::ProjectFirst),
            Some(RetrievalIntent::CurrentTask),
        );
        let now = Utc::now();
        let durable = MemoryItem {
            scope: MemoryScope::Project,
            kind: MemoryKind::Fact,
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            confidence: 0.86,
            source_quality: Some(SourceQuality::Canonical),
            last_verified_at: Some(now - chrono::Duration::days(8)),
            updated_at: now - chrono::Duration::days(60),
            tags: vec!["project_fact".to_string()],
            content: "remembered project fact: use compact spill at boundaries".to_string(),
            ..sample_item("durable project truth", vec!["project_fact"], Some("memd"))
        };
        let fresh_noise = MemoryItem {
            scope: MemoryScope::Global,
            kind: MemoryKind::Status,
            project: None,
            namespace: Some("main".to_string()),
            confidence: 0.94,
            source_quality: Some(SourceQuality::Synthetic),
            last_verified_at: None,
            updated_at: now,
            tags: vec!["session_state".to_string()],
            content: "resume state noise: fresh but not durable".to_string(),
            ..sample_item("fresh noise", vec![], Some("memd"))
        };

        assert!(
            search_score(
                &durable,
                None,
                0.92,
                &Some("spill".to_string()),
                Some(&"demo".to_string()),
                Some(&"main".to_string()),
                &plan,
            ) > search_score(
                &fresh_noise,
                None,
                0.95,
                &Some("spill".to_string()),
                Some(&"demo".to_string()),
                Some(&"main".to_string()),
                &plan,
            )
        );
    }

    #[test]
    fn trust_rank_adjustment_rewards_strong_sources_and_penalizes_weak_ones() {
        assert!(trust_rank_adjustment(0.95) > trust_rank_adjustment(0.8));
        assert!(trust_rank_adjustment(0.8) > trust_rank_adjustment(0.55));
        assert!(trust_rank_adjustment(0.55) > trust_rank_adjustment(0.4));
        assert!(trust_rank_adjustment(0.4) > trust_rank_adjustment(0.2));
    }

    #[test]
    fn age_penalty_grows_with_age_but_is_bounded() {
        let now = Utc::now();
        let recent = now - chrono::Duration::days(7);
        let older = now - chrono::Duration::days(70);
        assert!(age_penalty(older) > age_penalty(recent));
        assert!(age_penalty(older) <= 2.0);
    }

    #[test]
    fn inbox_score_prefers_project_scoped_items_over_shared_noise() {
        let plan = RetrievalPlan::resolve(
            Some(RetrievalRoute::ProjectFirst),
            Some(RetrievalIntent::CurrentTask),
        );
        let project_item = sample_item("project inbox item", vec!["project"], Some("memd"));
        let shared_noise = MemoryItem {
            scope: MemoryScope::Global,
            project: None,
            kind: MemoryKind::Status,
            content: "shared inbox noise".to_string(),
            confidence: 0.95,
            updated_at: Utc::now(),
            ..sample_item("shared inbox noise", vec!["session_state"], Some("memd"))
        };
        let entity = sample_entity(Some("demo"), Some("main"), 0.8, 3, Some(Utc::now()));

        assert!(
            inbox_score(
                &project_item,
                Some(&entity),
                Some(&"demo".to_string()),
                Some(&"main".to_string()),
                &plan,
            ) > inbox_score(
                &shared_noise,
                Some(&entity),
                Some(&"demo".to_string()),
                Some(&"main".to_string()),
                &plan,
            )
        );
    }

    #[test]
    fn associative_recall_score_prefers_same_project_links_over_cross_project_noise() {
        let root = sample_entity(Some("demo"), Some("main"), 0.9, 3, Some(Utc::now()));
        let same_project = sample_entity(Some("demo"), Some("main"), 0.7, 2, Some(Utc::now()));
        let other_project = sample_entity(Some("other"), Some("main"), 1.0, 20, Some(Utc::now()));
        let link = MemoryEntityLinkRecord {
            id: uuid::Uuid::new_v4(),
            from_entity_id: root.id,
            to_entity_id: same_project.id,
            relation_kind: memd_schema::EntityRelationKind::Related,
            confidence: 0.85,
            created_at: Utc::now(),
            note: Some("related".to_string()),
            context: None,
            tags: vec!["project".to_string()],
        };
        let other_link = MemoryEntityLinkRecord {
            id: uuid::Uuid::new_v4(),
            from_entity_id: root.id,
            to_entity_id: other_project.id,
            relation_kind: memd_schema::EntityRelationKind::Related,
            confidence: 0.85,
            created_at: Utc::now(),
            note: Some("related".to_string()),
            context: None,
            tags: vec!["project".to_string()],
        };

        assert!(
            associative_recall_score(&same_project, &link, 1, &root)
                > associative_recall_score(&other_project, &other_link, 1, &root)
        );
    }
}

pub(crate) fn entity_context_bonus(
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

pub(crate) fn inbox_reasons(item: &MemoryItem) -> Vec<String> {
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
        reasons.push("inferred".to_string());
    }
    if item.source_quality == Some(SourceQuality::Synthetic) {
        reasons.push("rejected-source".to_string());
    }
    if item.last_verified_at.is_none()
        && item.status == MemoryStatus::Active
        && item.stage == MemoryStage::Canonical
    {
        reasons.push("claimed".to_string());
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

pub(crate) fn epistemic_state_label(item: &MemoryItem) -> &'static str {
    match item.status {
        MemoryStatus::Contested => "contested",
        MemoryStatus::Stale => "stale",
        MemoryStatus::Superseded => "superseded",
        MemoryStatus::Expired => "expired",
        MemoryStatus::Active => {
            if item.last_verified_at.is_some() {
                "verified"
            } else if item.source_quality == Some(SourceQuality::Derived) {
                "inferred"
            } else {
                "claimed"
            }
        }
    }
}

#[cfg(test)]
mod epistemic_state_tests {
    use super::*;
    use chrono::Utc;
    use memd_schema::{MemoryKind, MemoryScope, MemoryStage, MemoryStatus, MemoryVisibility};
    use uuid::Uuid;

    fn test_item(
        status: MemoryStatus,
        source_quality: Option<SourceQuality>,
        last_verified_at: Option<chrono::DateTime<Utc>>,
    ) -> MemoryItem {
        MemoryItem {
            id: Uuid::nil(),
            content: "memory".to_string(),
            redundancy_key: None,
            belief_branch: None,
            preferred: false,
            kind: MemoryKind::Fact,
            scope: MemoryScope::Project,
            project: Some("memd".into()),
            namespace: Some("test".into()),
            workspace: Some("core".into()),
            visibility: MemoryVisibility::Workspace,
            source_agent: Some("codex".into()),
            source_system: Some("memd".into()),
            source_path: None,
            source_quality,
            confidence: 0.9,
            ttl_seconds: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_verified_at,
            supersedes: Vec::new(),
            tags: vec![],
            status,
            stage: MemoryStage::Canonical,
        }
    }

    #[test]
    fn epistemic_state_labels_distinguish_claimed_inferred_verified_and_stale() {
        assert_eq!(
            epistemic_state_label(&test_item(
                MemoryStatus::Active,
                Some(SourceQuality::Canonical),
                None
            )),
            "claimed"
        );
        assert_eq!(
            epistemic_state_label(&test_item(
                MemoryStatus::Active,
                Some(SourceQuality::Derived),
                None
            )),
            "inferred"
        );
        assert_eq!(
            epistemic_state_label(&test_item(
                MemoryStatus::Active,
                Some(SourceQuality::Canonical),
                Some(Utc::now())
            )),
            "verified"
        );
        assert_eq!(
            epistemic_state_label(&test_item(
                MemoryStatus::Stale,
                Some(SourceQuality::Canonical),
                Some(Utc::now())
            )),
            "stale"
        );
        assert_eq!(
            epistemic_state_label(&test_item(
                MemoryStatus::Contested,
                Some(SourceQuality::Canonical),
                Some(Utc::now())
            )),
            "contested"
        );
    }

    #[test]
    fn inbox_reasons_surface_claimed_and_inferred_memory() {
        let claimed = test_item(MemoryStatus::Active, Some(SourceQuality::Canonical), None);
        let inferred = test_item(MemoryStatus::Active, Some(SourceQuality::Derived), None);
        let stale = test_item(
            MemoryStatus::Stale,
            Some(SourceQuality::Canonical),
            Some(Utc::now()),
        );

        assert!(
            inbox_reasons(&claimed)
                .iter()
                .any(|reason| reason == "claimed")
        );
        assert!(
            inbox_reasons(&inferred)
                .iter()
                .any(|reason| reason == "inferred")
        );
        assert!(inbox_reasons(&stale).iter().any(|reason| reason == "stale"));
    }
}
