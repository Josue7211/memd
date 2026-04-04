use axum::http::StatusCode;
use memd_schema::{
    ExplainArtifactRecord, ExplainMemoryRequest, ExplainMemoryResponse, MemoryEventRecord,
    MemoryItem, MemoryStage, MemoryStatus, RetrievalIntent, RetrievalRoute, SourceMemoryRecord,
    SourceMemoryRequest,
};

use super::{canonical_key, internal_error, redundancy_key, AppState};

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
            source_agent: item.source_agent.clone(),
            source_system: item.source_system.clone(),
            limit: Some(5),
        })
        .map_err(internal_error)?
        .sources;
    let artifact_trail = build_artifact_trail(&item, &events, &sources);
    let policy_hooks = build_policy_hooks(&item, &plan, &sources);

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
        artifact_trail,
        policy_hooks,
    })
}

fn build_artifact_trail(
    item: &MemoryItem,
    events: &[MemoryEventRecord],
    sources: &[SourceMemoryRecord],
) -> Vec<ExplainArtifactRecord> {
    let mut trail = vec![ExplainArtifactRecord {
        kind: "memory_item".to_string(),
        label: "canonical memory".to_string(),
        summary: item.content.clone(),
        source_agent: item.source_agent.clone(),
        source_system: item.source_system.clone(),
        source_path: item.source_path.clone(),
        source_quality: item.source_quality,
        recorded_at: Some(item.updated_at),
    }];

    trail.extend(events.iter().take(3).map(|event| ExplainArtifactRecord {
        kind: "event".to_string(),
        label: event.event_type.clone(),
        summary: event.summary.clone(),
        source_agent: event.source_agent.clone(),
        source_system: event.source_system.clone(),
        source_path: event.source_path.clone(),
        source_quality: None,
        recorded_at: Some(event.occurred_at),
    }));

    trail.extend(sources.iter().take(3).map(|source| ExplainArtifactRecord {
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
        source_agent: source.source_agent.clone(),
        source_system: source.source_system.clone(),
        source_path: None,
        source_quality: None,
        recorded_at: source.last_seen_at,
    }));

    trail
}

fn build_policy_hooks(
    item: &MemoryItem,
    plan: &super::routing::RetrievalPlan,
    sources: &[SourceMemoryRecord],
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

    match item.status {
        MemoryStatus::Stale => hooks.push("verification_queue".to_string()),
        MemoryStatus::Contested => hooks.push("conflict_resolution".to_string()),
        MemoryStatus::Superseded => hooks.push("supersession_cleanup".to_string()),
        MemoryStatus::Expired => hooks.push("cold_storage".to_string()),
        MemoryStatus::Active => {}
    }

    if let Some(best_source) = sources.first() {
        hooks.push(format!("top_source_trust={:.2}", best_source.trust_score));
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
