use anyhow::Context;
use memd_schema::{EntityRelationKind, MemoryKind, MemoryScope, RetrievalIntent, RetrievalRoute};

pub(crate) fn parse_uuid_list(values: &[String]) -> anyhow::Result<Vec<uuid::Uuid>> {
    values
        .iter()
        .map(|value| value.parse::<uuid::Uuid>().context("parse uuid"))
        .collect()
}

pub(crate) fn parse_memory_kind_value(value: &str) -> anyhow::Result<MemoryKind> {
    let normalized = value.trim().to_ascii_lowercase().replace('-', "_");
    match normalized.as_str() {
        "fact" => Ok(MemoryKind::Fact),
        "decision" => Ok(MemoryKind::Decision),
        "preference" => Ok(MemoryKind::Preference),
        "runbook" => Ok(MemoryKind::Runbook),
        "topology" => Ok(MemoryKind::Topology),
        "status" => Ok(MemoryKind::Status),
        "pattern" => Ok(MemoryKind::Pattern),
        "constraint" => Ok(MemoryKind::Constraint),
        _ => anyhow::bail!(
            "invalid memory kind '{value}'; expected fact, decision, preference, runbook, topology, status, pattern, or constraint"
        ),
    }
}

pub(crate) fn parse_memory_scope_value(value: &str) -> anyhow::Result<MemoryScope> {
    let normalized = value.trim().to_ascii_lowercase().replace('-', "_");
    match normalized.as_str() {
        "local" => Ok(MemoryScope::Local),
        "synced" => Ok(MemoryScope::Synced),
        "project" => Ok(MemoryScope::Project),
        "global" => Ok(MemoryScope::Global),
        _ => anyhow::bail!("invalid scope '{value}'; expected local, synced, project, or global"),
    }
}

pub(crate) fn parse_source_quality_value(
    value: &str,
) -> anyhow::Result<memd_schema::SourceQuality> {
    let normalized = value.trim().to_ascii_lowercase().replace('-', "_");
    match normalized.as_str() {
        "canonical" => Ok(memd_schema::SourceQuality::Canonical),
        "derived" => Ok(memd_schema::SourceQuality::Derived),
        "synthetic" => Ok(memd_schema::SourceQuality::Synthetic),
        _ => anyhow::bail!(
            "invalid source quality '{value}'; expected canonical, derived, or synthetic"
        ),
    }
}

pub(crate) fn parse_entity_relation_kind(
    value: &str,
) -> anyhow::Result<EntityRelationKind> {
    let normalized = value.trim().to_ascii_lowercase().replace('-', "_");
    match normalized.as_str() {
        "same_as" | "same" => Ok(EntityRelationKind::SameAs),
        "derived_from" | "derived" => Ok(EntityRelationKind::DerivedFrom),
        "supersedes" => Ok(EntityRelationKind::Supersedes),
        "contradicts" => Ok(EntityRelationKind::Contradicts),
        "related" => Ok(EntityRelationKind::Related),
        _ => anyhow::bail!(
            "invalid relation kind '{value}'; expected same_as, derived_from, supersedes, contradicts, or related"
        ),
    }
}

pub(crate) fn parse_retrieval_route(value: Option<String>) -> anyhow::Result<Option<RetrievalRoute>> {
    match value {
        Some(value) => Ok(Some(parse_retrieval_route_value(&value)?)),
        None => Ok(None),
    }
}

pub(crate) fn parse_retrieval_intent(
    value: Option<String>,
) -> anyhow::Result<Option<RetrievalIntent>> {
    match value {
        Some(value) => Ok(Some(parse_retrieval_intent_value(&value)?)),
        None => Ok(None),
    }
}

pub(crate) fn parse_retrieval_route_value(value: &str) -> anyhow::Result<RetrievalRoute> {
    let normalized = value.trim().to_ascii_lowercase().replace('-', "_");
    match normalized.as_str() {
        "auto" => Ok(RetrievalRoute::Auto),
        "local_only" => Ok(RetrievalRoute::LocalOnly),
        "synced_only" => Ok(RetrievalRoute::SyncedOnly),
        "project_only" => Ok(RetrievalRoute::ProjectOnly),
        "global_only" => Ok(RetrievalRoute::GlobalOnly),
        "local_first" => Ok(RetrievalRoute::LocalFirst),
        "synced_first" => Ok(RetrievalRoute::SyncedFirst),
        "project_first" => Ok(RetrievalRoute::ProjectFirst),
        "global_first" => Ok(RetrievalRoute::GlobalFirst),
        "all" => Ok(RetrievalRoute::All),
        _ => anyhow::bail!(
            "invalid retrieval route '{value}'; expected auto, local_only, synced_only, project_only, global_only, local_first, synced_first, project_first, global_first, or all"
        ),
    }
}

pub(crate) fn parse_retrieval_intent_value(value: &str) -> anyhow::Result<RetrievalIntent> {
    let normalized = value.trim().to_ascii_lowercase().replace('-', "_");
    match normalized.as_str() {
        "general" => Ok(RetrievalIntent::General),
        "current_task" => Ok(RetrievalIntent::CurrentTask),
        "decision" => Ok(RetrievalIntent::Decision),
        "runbook" => Ok(RetrievalIntent::Runbook),
        "topology" => Ok(RetrievalIntent::Topology),
        "preference" => Ok(RetrievalIntent::Preference),
        "fact" => Ok(RetrievalIntent::Fact),
        "pattern" => Ok(RetrievalIntent::Pattern),
        _ => anyhow::bail!(
            "invalid retrieval intent '{value}'; expected general, current_task, decision, runbook, topology, preference, fact, or pattern"
        ),
    }
}
