use axum::http::StatusCode;
use memd_schema::{
    ExplainMemoryRequest, ExplainMemoryResponse, MemoryItem, MemoryStage, MemoryStatus,
    RetrievalIntent, RetrievalRoute, SourceMemoryRequest,
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
    })
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
