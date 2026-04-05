use axum::http::StatusCode;
use memd_schema::{
    ExplainBranchSiblingRecord, ExplainMemoryRequest, ExplainMemoryResponse, MemoryEventRecord,
    MemoryItem, MemoryRehydrationRecord, MemoryStage, MemoryStatus, RetrievalFeedbackSummary,
    RetrievalFeedbackSurfaceCount, RetrievalIntent, RetrievalRoute, SourceMemoryRecord,
    SourceMemoryRequest,
};
use std::collections::BTreeMap;

use super::{AppState, canonical_key, internal_error, redundancy_key};

pub(crate) fn explain_memory(
    state: &AppState,
    req: ExplainMemoryRequest,
) -> Result<ExplainMemoryResponse, (StatusCode, String)> {
    let plan = super::routing::RetrievalPlan::resolve(req.route, req.intent);
    let item = state
        .store
        .get(req.id)
        .map_err(internal_error)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "memory item not found".to_string()))?;
    if req
        .belief_branch
        .as_ref()
        .is_some_and(|branch| item.belief_branch.as_ref() != Some(branch))
    {
        return Err((StatusCode::NOT_FOUND, "memory item not found".to_string()));
    }

    let reasons = explain_reasons(&item, &plan);
    let canonical = canonical_key(&item);
    let redundancy = redundancy_key(&item);
    state.rehearse_item(req.id, 0.06).map_err(internal_error)?;
    let entity = state
        .store
        .entity_for_item(item.id)
        .map_err(internal_error)?;
    let events = match &entity {
        Some(entity) => state
            .store
            .events_for_entity(entity.id, 8)
            .map_err(internal_error)?,
        None => Vec::new(),
    };
    let sources = state
        .store
        .source_memory(&SourceMemoryRequest {
            project: item.project.clone(),
            namespace: item.namespace.clone(),
            workspace: item.workspace.clone(),
            visibility: Some(item.visibility),
            source_agent: item.source_agent.clone(),
            source_system: item.source_system.clone(),
            limit: Some(5),
        })
        .map_err(internal_error)?
        .sources;
    let retrieval_feedback = build_retrieval_feedback(&events, &item);
    let branch_siblings = build_branch_siblings(state, &item).map_err(internal_error)?;
    let rehydration = build_rehydration(&item, &events, &sources);
    let policy_hooks = build_policy_hooks(&item, &plan, &sources, &branch_siblings);

    Ok(ExplainMemoryResponse {
        route: plan.route,
        intent: plan.intent,
        item,
        canonical_key: canonical,
        redundancy_key: redundancy,
        reasons,
        entity,
        events,
        sources,
        retrieval_feedback,
        branch_siblings,
        rehydration,
        policy_hooks,
    })
}

fn build_retrieval_feedback(
    events: &[MemoryEventRecord],
    item: &MemoryItem,
) -> RetrievalFeedbackSummary {
    let mut counts = BTreeMap::<String, usize>::new();
    let mut last_retrieved_at = None;
    let mut recent_policy_hooks = Vec::new();

    for event in events
        .iter()
        .filter(|event| event.event_type.starts_with("retrieved_"))
    {
        let surface = event
            .event_type
            .strip_prefix("retrieved_")
            .unwrap_or(event.event_type.as_str())
            .to_string();
        *counts.entry(surface).or_insert(0) += 1;
        if last_retrieved_at.is_none_or(|current| event.recorded_at > current) {
            last_retrieved_at = Some(event.recorded_at);
        }
        for tag in event.tags.iter().filter(|tag| {
            tag.starts_with("route:")
                || tag.starts_with("intent:")
                || tag.starts_with("belief_branch:")
        }) {
            if !recent_policy_hooks.iter().any(|existing| existing == tag) {
                recent_policy_hooks.push(tag.clone());
            }
        }
    }

    if let Some(branch) = &item.belief_branch {
        let branch_tag = format!("belief_branch:{branch}");
        if !recent_policy_hooks
            .iter()
            .any(|existing| existing == &branch_tag)
        {
            recent_policy_hooks.push(branch_tag);
        }
    }

    RetrievalFeedbackSummary {
        total_retrievals: counts.values().sum(),
        last_retrieved_at,
        by_surface: counts
            .into_iter()
            .map(|(surface, count)| RetrievalFeedbackSurfaceCount { surface, count })
            .collect(),
        recent_policy_hooks,
    }
}

fn build_branch_siblings(
    state: &AppState,
    item: &MemoryItem,
) -> anyhow::Result<Vec<ExplainBranchSiblingRecord>> {
    let canonical = canonical_key(item);
    let redundancy = redundancy_key(item);
    let mut siblings = state
        .snapshot()?
        .into_iter()
        .filter(|candidate| candidate.id != item.id)
        .filter(|candidate| candidate.kind == item.kind)
        .filter(|candidate| candidate.scope == item.scope)
        .filter(|candidate| candidate.project == item.project)
        .filter(|candidate| candidate.namespace == item.namespace)
        .filter(|candidate| candidate.redundancy_key.as_deref() == Some(redundancy.as_str()))
        .filter(|candidate| canonical_key(candidate) != canonical)
        .map(|candidate| ExplainBranchSiblingRecord {
            id: candidate.id,
            belief_branch: candidate.belief_branch,
            preferred: candidate.preferred,
            status: candidate.status,
            stage: candidate.stage,
            confidence: candidate.confidence,
            updated_at: candidate.updated_at,
        })
        .collect::<Vec<_>>();
    siblings.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    siblings.truncate(6);
    Ok(siblings)
}

fn build_rehydration(
    item: &MemoryItem,
    events: &[MemoryEventRecord],
    sources: &[SourceMemoryRecord],
) -> Vec<MemoryRehydrationRecord> {
    let mut trail = vec![MemoryRehydrationRecord {
        id: Some(item.id),
        kind: "memory_item".to_string(),
        label: "canonical memory".to_string(),
        summary: item.content.clone(),
        reason: Some("rehydrate_primary_memory".to_string()),
        source_agent: item.source_agent.clone(),
        source_system: item.source_system.clone(),
        source_path: item.source_path.clone(),
        source_quality: item.source_quality,
        recorded_at: Some(item.updated_at),
    }];

    trail.extend(events.iter().take(3).map(|event| MemoryRehydrationRecord {
        id: None,
        kind: "event".to_string(),
        label: event.event_type.clone(),
        summary: event.summary.clone(),
        reason: Some("rehydrate_event_context".to_string()),
        source_agent: event.source_agent.clone(),
        source_system: event.source_system.clone(),
        source_path: event.source_path.clone(),
        source_quality: None,
        recorded_at: Some(event.occurred_at),
    }));

    trail.extend(
        sources
            .iter()
            .take(3)
            .map(|source| MemoryRehydrationRecord {
                id: None,
                kind: "source_memory".to_string(),
                label: format!(
                    "{}:{}",
                    source.source_agent.as_deref().unwrap_or("none"),
                    source.source_system.as_deref().unwrap_or("none")
                ),
                summary: format!(
                    "items={} trust={:.2} avg_confidence={:.2}",
                    source.item_count, source.trust_score, source.avg_confidence
                ),
                reason: Some("rehydrate_source_lane".to_string()),
                source_agent: source.source_agent.clone(),
                source_system: source.source_system.clone(),
                source_path: None,
                source_quality: None,
                recorded_at: source.last_seen_at,
            }),
    );

    trail
}

fn build_policy_hooks(
    item: &MemoryItem,
    plan: &super::routing::RetrievalPlan,
    sources: &[SourceMemoryRecord],
    branch_siblings: &[ExplainBranchSiblingRecord],
) -> Vec<String> {
    let mut hooks = vec![
        format!("route={}", format_route(plan.route)),
        format!("intent={}", format_intent(plan.intent)),
        "source_trust_floor=0.60".to_string(),
    ];

    match item.stage {
        MemoryStage::Candidate => hooks.push("promotion_review".to_string()),
        MemoryStage::Canonical => hooks.push("canonical_retention".to_string()),
    }

    match item.kind {
        memd_schema::MemoryKind::Procedural => hooks.push("procedural_memory".to_string()),
        memd_schema::MemoryKind::SelfModel => hooks.push("self_model_memory".to_string()),
        _ => {}
    }

    match item.status {
        MemoryStatus::Stale => hooks.push("verification_queue".to_string()),
        MemoryStatus::Contested => hooks.push("conflict_resolution".to_string()),
        MemoryStatus::Superseded => hooks.push("supersession_cleanup".to_string()),
        MemoryStatus::Expired => hooks.push("cold_storage".to_string()),
        MemoryStatus::Active => {}
    }

    if let Some(best_source) = sources.first() {
        hooks.push(format!("top_source_trust={:.2}", best_source.trust_score));
        if best_source.trust_score < 0.6 {
            hooks.push("trust_below_floor".to_string());
        } else if best_source.trust_score >= 0.75 {
            hooks.push("trust_boost".to_string());
        }
    }
    if item.preferred {
        hooks.push("preferred_branch".to_string());
    } else if !branch_siblings.is_empty()
        && !branch_siblings.iter().any(|sibling| sibling.preferred)
    {
        hooks.push("unresolved_contradiction".to_string());
    }
    if let Some(branch) = &item.belief_branch {
        hooks.push(format!("belief_branch={branch}"));
    }

    hooks
}

fn explain_reasons(item: &MemoryItem, plan: &super::routing::RetrievalPlan) -> Vec<String> {
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
    if let Some(branch) = &item.belief_branch {
        reasons.push(format!("belief_branch={branch}"));
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

fn format_route(route: RetrievalRoute) -> &'static str {
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

fn format_intent(intent: RetrievalIntent) -> &'static str {
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

fn format_scope(scope: super::MemoryScope) -> &'static str {
    match scope {
        super::MemoryScope::Local => "local",
        super::MemoryScope::Synced => "synced",
        super::MemoryScope::Project => "project",
        super::MemoryScope::Global => "global",
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
