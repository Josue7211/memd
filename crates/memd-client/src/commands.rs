use anyhow::Context;
use memd_schema::{
    EntityRelationKind, MemoryKind, MemoryRepairMode, MemoryScope, MemoryVisibility,
    RetrievalIntent, RetrievalRoute,
};

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
        "procedural" | "procedure" | "workflow" => Ok(MemoryKind::Procedural),
        "self_model" | "self" | "capability" | "capabilities" => Ok(MemoryKind::SelfModel),
        "topology" => Ok(MemoryKind::Topology),
        "status" => Ok(MemoryKind::Status),
        "pattern" => Ok(MemoryKind::Pattern),
        "constraint" => Ok(MemoryKind::Constraint),
        _ => anyhow::bail!(
            "invalid memory kind '{value}'; expected fact, decision, preference, runbook, procedural, self_model, topology, status, pattern, or constraint"
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

pub(crate) fn parse_memory_visibility_value(value: &str) -> anyhow::Result<MemoryVisibility> {
    let normalized = value.trim().to_ascii_lowercase().replace('-', "_");
    match normalized.as_str() {
        "private" => Ok(MemoryVisibility::Private),
        "workspace" | "shared" => Ok(MemoryVisibility::Workspace),
        "public" => Ok(MemoryVisibility::Public),
        _ => anyhow::bail!(
            "invalid visibility '{value}'; expected private, workspace, or public"
        ),
    }
}

pub(crate) fn parse_memory_status_value(value: &str) -> anyhow::Result<memd_schema::MemoryStatus> {
    let normalized = value.trim().to_ascii_lowercase().replace('-', "_");
    match normalized.as_str() {
        "active" => Ok(memd_schema::MemoryStatus::Active),
        "stale" => Ok(memd_schema::MemoryStatus::Stale),
        "superseded" => Ok(memd_schema::MemoryStatus::Superseded),
        "contested" => Ok(memd_schema::MemoryStatus::Contested),
        "expired" => Ok(memd_schema::MemoryStatus::Expired),
        _ => anyhow::bail!(
            "invalid memory status '{value}'; expected active, stale, superseded, contested, or expired"
        ),
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

pub(crate) fn parse_memory_repair_mode_value(value: &str) -> anyhow::Result<MemoryRepairMode> {
    let normalized = value.trim().to_ascii_lowercase().replace('-', "_");
    match normalized.as_str() {
        "verify" => Ok(MemoryRepairMode::Verify),
        "expire" => Ok(MemoryRepairMode::Expire),
        "supersede" => Ok(MemoryRepairMode::Supersede),
        "contest" => Ok(MemoryRepairMode::Contest),
        "prefer_branch" | "prefer" | "resolve" => Ok(MemoryRepairMode::PreferBranch),
        "correct_metadata" | "correct" | "repair" => Ok(MemoryRepairMode::CorrectMetadata),
        _ => anyhow::bail!(
            "invalid repair mode '{value}'; expected verify, expire, supersede, contest, prefer_branch, or correct_metadata"
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
        "procedural" | "procedure" | "workflow" => Ok(RetrievalIntent::Procedural),
        "self_model" | "self" | "capability" | "capabilities" => Ok(RetrievalIntent::SelfModel),
        "topology" => Ok(RetrievalIntent::Topology),
        "preference" => Ok(RetrievalIntent::Preference),
        "fact" => Ok(RetrievalIntent::Fact),
        "pattern" => Ok(RetrievalIntent::Pattern),
        _ => anyhow::bail!(
            "invalid retrieval intent '{value}'; expected general, current_task, decision, runbook, procedural, self_model, topology, preference, fact, or pattern"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_memory_kind_value, parse_retrieval_intent_value};
    use memd_schema::{MemoryKind, RetrievalIntent};

    #[test]
    fn parses_new_memory_kinds() {
        assert_eq!(
            parse_memory_kind_value("procedural").unwrap(),
            MemoryKind::Procedural
        );
        assert_eq!(
            parse_memory_kind_value("self-model").unwrap(),
            MemoryKind::SelfModel
        );
    }

    #[test]
    fn parses_new_retrieval_intents() {
        assert_eq!(
            parse_retrieval_intent_value("workflow").unwrap(),
            RetrievalIntent::Procedural
        );
        assert_eq!(
            parse_retrieval_intent_value("capabilities").unwrap(),
            RetrievalIntent::SelfModel
        );
    }
}
