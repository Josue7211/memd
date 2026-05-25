use axum::http::StatusCode;
use chrono::Utc;
use memd_schema::{
    ExplainMemoryRequest, HiveClaimsRequest, HiveSessionsRequest, HiveTasksRequest,
    MemoryEventRecord, MemoryItem, MemoryRepairMode, MemoryStage, MemoryStatus, MemoryVisibility,
    RepairMemoryRequest, SourceMemoryRequest, TimelineMemoryResponse, VisibleMemoryArtifact,
    VisibleMemoryArtifactDetailResponse, VisibleMemoryGraphEdge, VisibleMemoryGraphNode,
    VisibleMemoryHome, VisibleMemoryKnowledgeMap, VisibleMemoryProvenance,
    VisibleMemorySnapshotResponse, VisibleMemoryStatus, VisibleMemoryUiActionKind,
    VisibleMemoryUiActionRequest, VisibleMemoryUiActionResponse, WorkspaceMemoryRecord,
    WorkspaceMemoryRequest,
};
use uuid::Uuid;

use crate::{AppState, canonical_key, errors::MemdError, internal_error, redundancy_key};

pub(crate) fn build_visible_memory_snapshot(
    state: &AppState,
) -> anyhow::Result<VisibleMemorySnapshotResponse> {
    let items = state.snapshot()?;
    let focus_item = items
        .iter()
        .find(|item| item.preferred)
        .or_else(|| items.first())
        .ok_or_else(|| anyhow::anyhow!("no memory items available"))?;

    let inbox_items = inbox_items(&items);
    let repair_items = repair_items(&items);
    let workspace_records = workspace_records(state)?;
    let timeline_events = timeline_events(state, focus_item.id, 4)?;
    let inbox_count = inbox_items.len();
    let repair_count = repair_items.len();
    let awareness_count = workspace_records.len();

    let focus_artifact = visible_artifact(focus_item);
    let knowledge_map =
        build_knowledge_map(focus_item, &items, &timeline_events, &workspace_records);

    Ok(VisibleMemorySnapshotResponse {
        generated_at: Utc::now(),
        home: VisibleMemoryHome {
            focus_artifact,
            inbox_count,
            repair_count,
            awareness_count,
        },
        knowledge_map,
    })
}

pub(crate) fn build_visible_memory_artifact_detail(
    state: &AppState,
    id: Uuid,
) -> Result<VisibleMemoryArtifactDetailResponse, (StatusCode, String)> {
    let items = state.snapshot().map_err(internal_error)?;
    let focus_item = items
        .iter()
        .find(|item| item.id == id)
        .cloned()
        .ok_or_else(|| MemdError::not_found("memory item", id).into_wire())?;
    build_visible_memory_artifact_detail_from_item(state, focus_item, &items)
}

pub(crate) fn perform_visible_memory_action(
    state: &AppState,
    req: VisibleMemoryUiActionRequest,
) -> Result<VisibleMemoryUiActionResponse, (StatusCode, String)> {
    let item = state
        .store
        .get(req.id)
        .map_err(internal_error)?
        .ok_or_else(|| MemdError::not_found("memory item", req.id).into_wire())?;

    let action = req.action;
    let snapshot = state.snapshot().map_err(internal_error)?;
    let (artifact, outcome, message, detail, open_uri, source_path) = match action {
        VisibleMemoryUiActionKind::Inspect => {
            let detail =
                build_visible_memory_artifact_detail_from_item(state, item.clone(), &snapshot)?;
            (
                item,
                "inspected".to_string(),
                "built selected artifact detail".to_string(),
                Some(detail),
                None,
                None,
            )
        }
        VisibleMemoryUiActionKind::Explain => {
            let detail =
                build_visible_memory_artifact_detail_from_item(state, item.clone(), &snapshot)?;
            (
                item,
                "explained".to_string(),
                "reused explain, timeline, source, workspace, and coordination data".to_string(),
                Some(detail),
                None,
                None,
            )
        }
        VisibleMemoryUiActionKind::VerifyCurrent => {
            let updated = crate::repair::verify_item(
                state,
                memd_schema::VerifyMemoryRequest {
                    id: item.id,
                    confidence: Some(item.confidence),
                    status: Some(MemoryStatus::Active),
                },
            )?;
            let detail = build_visible_memory_artifact_detail_from_item(
                state,
                updated.clone(),
                &state.snapshot().map_err(internal_error)?,
            )?;
            (
                updated,
                "verified".to_string(),
                "marked memory current and updated verification state".to_string(),
                Some(detail),
                None,
                None,
            )
        }
        VisibleMemoryUiActionKind::MarkStale => {
            let updated = crate::repair::repair_item(
                state,
                RepairMemoryRequest {
                    id: item.id,
                    mode: MemoryRepairMode::Expire,
                    confidence: Some(item.confidence),
                    status: Some(MemoryStatus::Stale),
                    workspace: None,
                    visibility: None,
                    source_agent: None,
                    source_system: None,
                    source_path: None,
                    source_quality: None,
                    content: None,
                    tags: None,
                    supersedes: Vec::new(),
                },
            )?
            .item;
            let detail = build_visible_memory_artifact_detail_from_item(
                state,
                updated.clone(),
                &state.snapshot().map_err(internal_error)?,
            )?;
            (
                updated,
                "marked_stale".to_string(),
                "marked artifact stale".to_string(),
                Some(detail),
                None,
                None,
            )
        }
        VisibleMemoryUiActionKind::Promote => {
            let (updated, duplicate): (MemoryItem, Option<crate::DuplicateMatch>) = state
                .promote_item(memd_schema::PromoteMemoryRequest {
                    id: item.id,
                    scope: None,
                    project: None,
                    namespace: None,
                    workspace: None,
                    visibility: None,
                    belief_branch: None,
                    confidence: None,
                    ttl_seconds: None,
                    tags: None,
                    status: None,
                })
                .map_err(internal_error)?;
            let effective = duplicate
                .as_ref()
                .map_or(updated.clone(), |found| found.item.clone());
            let detail = build_visible_memory_artifact_detail_from_item(
                state,
                effective.clone(),
                &state.snapshot().map_err(internal_error)?,
            )?;
            (
                effective,
                "promoted".to_string(),
                if duplicate.is_some() {
                    "promoted artifact and resolved duplicate".to_string()
                } else {
                    "promoted artifact to canonical stage".to_string()
                },
                Some(detail),
                None,
                None,
            )
        }
        VisibleMemoryUiActionKind::OpenSource => {
            let detail =
                build_visible_memory_artifact_detail_from_item(state, item.clone(), &snapshot)?;
            (
                item.clone(),
                "metadata".to_string(),
                "returned source metadata for the selected artifact".to_string(),
                Some(detail),
                None,
                item.source_path.clone(),
            )
        }
        VisibleMemoryUiActionKind::OpenInObsidian => {
            let detail =
                build_visible_memory_artifact_detail_from_item(state, item.clone(), &snapshot)?;
            let open_uri = item
                .source_path
                .as_ref()
                .map(|path| build_obsidian_uri(path));
            (
                item.clone(),
                "metadata".to_string(),
                if open_uri.is_some() {
                    "generated Obsidian open URI".to_string()
                } else {
                    "no Obsidian path available".to_string()
                },
                Some(detail),
                open_uri,
                item.source_path.clone(),
            )
        }
    };

    Ok(VisibleMemoryUiActionResponse {
        action,
        artifact_id: artifact.id,
        outcome,
        message,
        detail,
        open_uri,
        source_path,
    })
}

fn workspace_records(state: &AppState) -> anyhow::Result<Vec<WorkspaceMemoryRecord>> {
    let response = state.store.workspace_memory(&WorkspaceMemoryRequest {
        project: None,
        namespace: None,
        workspace: None,
        visibility: None,
        source_agent: None,
        source_system: None,
        limit: Some(32),
    })?;
    Ok(response.workspaces)
}

fn timeline_events(
    state: &AppState,
    item_id: Uuid,
    limit: usize,
) -> anyhow::Result<Vec<MemoryEventRecord>> {
    let (entity, events) = state.entity_view(item_id, limit)?;
    Ok(if entity.is_some() { events } else { Vec::new() })
}

fn build_visible_memory_artifact_detail_from_item(
    state: &AppState,
    item: MemoryItem,
    items: &[MemoryItem],
) -> Result<VisibleMemoryArtifactDetailResponse, (StatusCode, String)> {
    let artifact = visible_artifact(&item);
    let explain = Some(crate::inspection::explain_memory(
        state,
        ExplainMemoryRequest {
            id: item.id,
            belief_branch: item.belief_branch.clone(),
            route: None,
            intent: None,
        },
    )?);

    let timeline: Option<TimelineMemoryResponse> = {
        let limit = 8;
        let (entity, events): (
            Option<memd_schema::MemoryEntityRecord>,
            Vec<MemoryEventRecord>,
        ) = state.entity_view(item.id, limit).map_err(internal_error)?;
        entity.map(|entity| TimelineMemoryResponse {
            route: memd_schema::RetrievalRoute::Auto,
            intent: memd_schema::RetrievalIntent::General,
            entity: Some(entity),
            events,
        })
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
            limit: Some(8),
        })
        .map_err(internal_error)?;

    let workspaces = state
        .store
        .workspace_memory(&WorkspaceMemoryRequest {
            project: item.project.clone(),
            namespace: item.namespace.clone(),
            workspace: item.workspace.clone(),
            visibility: Some(item.visibility),
            source_agent: item.source_agent.clone(),
            source_system: item.source_system.clone(),
            limit: Some(8),
        })
        .map_err(internal_error)?;

    let sessions = state
        .store
        .hive_sessions(&HiveSessionsRequest {
            session: None,
            project: item.project.clone(),
            namespace: item.namespace.clone(),
            repo_root: None,
            worktree_root: None,
            branch: None,
            workspace: item.workspace.clone(),
            hive_system: None,
            hive_role: None,
            host: None,
            hive_group: None,
            active_only: Some(true),
            limit: Some(8),
        })
        .map_err(internal_error)?;

    let tasks = state
        .store
        .hive_tasks(&HiveTasksRequest {
            session: None,
            project: item.project.clone(),
            namespace: item.namespace.clone(),
            workspace: item.workspace.clone(),
            active_only: Some(true),
            limit: Some(8),
        })
        .map_err(internal_error)?;

    let claims = state
        .store
        .hive_claims(&HiveClaimsRequest {
            session: None,
            project: item.project.clone(),
            namespace: item.namespace.clone(),
            workspace: item.workspace.clone(),
            active_only: Some(true),
            limit: Some(8),
        })
        .map_err(internal_error)?;

    let timeline_events = timeline
        .as_ref()
        .map(|entry| entry.events.clone())
        .unwrap_or_default();
    let related_map = build_knowledge_map(&item, items, &timeline_events, &workspaces.workspaces);
    let related_artifacts = related_visible_items(&item, items);

    Ok(VisibleMemoryArtifactDetailResponse {
        generated_at: Utc::now(),
        artifact,
        explain,
        timeline,
        sources,
        workspaces,
        sessions,
        tasks,
        claims,
        related_artifacts,
        related_map,
        actions: visible_actions_for_item(&item),
    })
}

fn related_visible_items(item: &MemoryItem, items: &[MemoryItem]) -> Vec<VisibleMemoryArtifact> {
    let mut related = items
        .iter()
        .filter(|candidate| candidate.id != item.id)
        .filter(|candidate| {
            candidate.workspace == item.workspace
                || candidate.project == item.project
                || candidate.supersedes.contains(&item.id)
                || item.supersedes.contains(&candidate.id)
                || candidate.source_path == item.source_path
                || candidate.source_system == item.source_system
                || candidate.source_agent == item.source_agent
        })
        .map(visible_artifact)
        .collect::<Vec<_>>();
    related.sort_by(|a, b| a.title.cmp(&b.title));
    related.truncate(12);
    related
}

fn visible_actions_for_item(item: &MemoryItem) -> Vec<VisibleMemoryUiActionKind> {
    let mut actions = vec![
        VisibleMemoryUiActionKind::Inspect,
        VisibleMemoryUiActionKind::Explain,
        VisibleMemoryUiActionKind::VerifyCurrent,
        VisibleMemoryUiActionKind::MarkStale,
        VisibleMemoryUiActionKind::Promote,
    ];
    if item.source_path.is_some() {
        actions.push(VisibleMemoryUiActionKind::OpenSource);
        actions.push(VisibleMemoryUiActionKind::OpenInObsidian);
    }
    actions
}

fn build_obsidian_uri(path: &str) -> String {
    format!("obsidian://open?path={}", percent_encode_path(path))
}

fn percent_encode_path(path: &str) -> String {
    let mut encoded = String::with_capacity(path.len());
    for byte in path.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                encoded.push(byte as char);
            }
            _ => {
                use std::fmt::Write as _;
                let _ = write!(encoded, "%{:02X}", byte);
            }
        }
    }
    encoded
}

fn build_knowledge_map(
    focus_item: &MemoryItem,
    items: &[MemoryItem],
    timeline_events: &[MemoryEventRecord],
    workspace_records: &[WorkspaceMemoryRecord],
) -> VisibleMemoryKnowledgeMap {
    let mut nodes = items
        .iter()
        .map(|item| VisibleMemoryGraphNode {
            artifact_id: item.id,
            title: artifact_title(item),
            artifact_kind: artifact_kind(item),
            status: visible_status(item),
        })
        .collect::<Vec<_>>();

    let mut edges = Vec::new();
    for artifact in items.iter() {
        if artifact.id == focus_item.id {
            continue;
        }
        if artifact.workspace == focus_item.workspace || artifact.project == focus_item.project {
            edges.push(VisibleMemoryGraphEdge {
                from: focus_item.id,
                to: artifact.id,
                relation: "related".to_string(),
            });
        }
    }

    for event in timeline_events {
        edges.push(VisibleMemoryGraphEdge {
            from: focus_item.id,
            to: event.id,
            relation: "timeline".to_string(),
        });
    }

    for workspace in workspace_records.iter().take(6) {
        let node_id = workspace_node_id(workspace);
        edges.push(VisibleMemoryGraphEdge {
            from: focus_item.id,
            to: node_id,
            relation: format!("workspace:{}", workspace_title(workspace)),
        });
    }

    for event in timeline_events {
        nodes.push(VisibleMemoryGraphNode {
            artifact_id: event.id,
            title: event.summary.clone(),
            artifact_kind: "timeline_event".to_string(),
            status: if event.confidence >= 0.75 {
                VisibleMemoryStatus::Current
            } else {
                VisibleMemoryStatus::Candidate
            },
        });
    }

    for workspace in workspace_records.iter().take(6) {
        let node_id = workspace_node_id(workspace);
        nodes.push(VisibleMemoryGraphNode {
            artifact_id: node_id,
            title: workspace_title(workspace),
            artifact_kind: "workspace_lane".to_string(),
            status: workspace_status(workspace),
        });
    }

    VisibleMemoryKnowledgeMap { nodes, edges }
}

fn visible_artifact(item: &MemoryItem) -> VisibleMemoryArtifact {
    let status = visible_status(item);
    let freshness = freshness_label(item, status);
    let repair_state = repair_state_label(item, status);
    let title = artifact_title(item);
    let mut actions = vec![
        "inspect".to_string(),
        "explain".to_string(),
        "verify_current".to_string(),
        "mark_stale".to_string(),
        "supersede".to_string(),
        "promote".to_string(),
    ];
    if item.source_path.is_some() {
        actions.push("open_in_obsidian".to_string());
    }

    VisibleMemoryArtifact {
        id: item.id,
        title,
        body: item.content.clone(),
        artifact_kind: artifact_kind(item),
        memory_kind: Some(item.kind),
        scope: Some(item.scope),
        visibility: Some(item.visibility),
        workspace: item.workspace.clone(),
        status,
        freshness,
        confidence: item.confidence,
        provenance: VisibleMemoryProvenance {
            source_system: item.source_system.clone(),
            source_path: item.source_path.clone(),
            producer: item.source_agent.clone(),
            trust_reason: trust_reason(item, status),
            last_verified_at: item.last_verified_at,
        },
        sources: item.source_path.clone().into_iter().collect(),
        linked_artifact_ids: item.supersedes.clone(),
        linked_sessions: item.workspace.clone().into_iter().collect(),
        linked_agents: item.source_agent.clone().into_iter().collect(),
        repair_state,
        actions,
    }
}

fn artifact_title(item: &MemoryItem) -> String {
    let candidate = item
        .source_path
        .as_deref()
        .and_then(|path| std::path::Path::new(path).file_stem())
        .and_then(|stem| stem.to_str())
        .unwrap_or_else(|| item.content.lines().next().unwrap_or("memory item"));
    let title = candidate.trim();
    if title.is_empty() {
        "memory item".to_string()
    } else {
        title.to_string()
    }
}

fn artifact_kind(item: &MemoryItem) -> String {
    if item.stage == MemoryStage::Candidate {
        "candidate_memory".to_string()
    } else {
        "memory_item".to_string()
    }
}

fn visible_status(item: &MemoryItem) -> VisibleMemoryStatus {
    if item.stage == MemoryStage::Candidate {
        return VisibleMemoryStatus::Candidate;
    }
    match item.status {
        MemoryStatus::Active => VisibleMemoryStatus::Current,
        MemoryStatus::Stale => VisibleMemoryStatus::Stale,
        MemoryStatus::Superseded => VisibleMemoryStatus::Superseded,
        MemoryStatus::Contested => VisibleMemoryStatus::Conflicted,
        MemoryStatus::Expired => VisibleMemoryStatus::Archived,
    }
}

fn freshness_label(item: &MemoryItem, status: VisibleMemoryStatus) -> String {
    match status {
        VisibleMemoryStatus::Current => {
            if item.last_verified_at.is_some() {
                "verified".to_string()
            } else if item.source_quality == Some(memd_schema::SourceQuality::Derived) {
                "inferred".to_string()
            } else {
                "claimed".to_string()
            }
        }
        VisibleMemoryStatus::Candidate => "candidate".to_string(),
        VisibleMemoryStatus::Stale => "stale".to_string(),
        VisibleMemoryStatus::Superseded => "superseded".to_string(),
        VisibleMemoryStatus::Conflicted => "conflicted".to_string(),
        VisibleMemoryStatus::Archived => "archived".to_string(),
    }
}

fn repair_state_label(item: &MemoryItem, status: VisibleMemoryStatus) -> String {
    match status {
        VisibleMemoryStatus::Current if item.stage == MemoryStage::Canonical => {
            "healthy".to_string()
        }
        VisibleMemoryStatus::Current => "needs_review".to_string(),
        VisibleMemoryStatus::Candidate => "needs_promotion".to_string(),
        VisibleMemoryStatus::Stale
        | VisibleMemoryStatus::Superseded
        | VisibleMemoryStatus::Conflicted
        | VisibleMemoryStatus::Archived => "needs_attention".to_string(),
    }
}

fn trust_reason(item: &MemoryItem, status: VisibleMemoryStatus) -> String {
    let origin = item.source_system.as_deref().unwrap_or("memd").to_string();
    let epistemic = match status {
        VisibleMemoryStatus::Stale => "stale",
        VisibleMemoryStatus::Superseded => "superseded",
        VisibleMemoryStatus::Conflicted => "contested",
        VisibleMemoryStatus::Archived => "archived",
        VisibleMemoryStatus::Candidate => {
            if item.source_quality == Some(memd_schema::SourceQuality::Derived) {
                "inferred"
            } else {
                "claimed"
            }
        }
        VisibleMemoryStatus::Current => crate::helpers::epistemic_state_label(item),
    };
    format!("{origin} {epistemic}")
}

fn is_inbox_item(item: &MemoryItem) -> bool {
    item.stage == MemoryStage::Candidate || item.status != MemoryStatus::Active
}

fn is_repair_item(item: &MemoryItem) -> bool {
    matches!(
        item.status,
        MemoryStatus::Stale
            | MemoryStatus::Superseded
            | MemoryStatus::Contested
            | MemoryStatus::Expired
    ) || item.stage == MemoryStage::Candidate
}

fn inbox_items(items: &[MemoryItem]) -> Vec<&MemoryItem> {
    items.iter().filter(|item| is_inbox_item(item)).collect()
}

fn repair_items(items: &[MemoryItem]) -> Vec<&MemoryItem> {
    items
        .iter()
        .filter(|item| {
            is_repair_item(item)
                || item.last_verified_at.is_none()
                || item.source_quality == Some(memd_schema::SourceQuality::Derived)
        })
        .collect()
}

fn workspace_title(record: &WorkspaceMemoryRecord) -> String {
    let project = record.project.as_deref().unwrap_or("shared");
    let namespace = record.namespace.as_deref().unwrap_or("default");
    let workspace = record.workspace.as_deref().unwrap_or("workspace");
    format!("{project} / {namespace} / {workspace}")
}

fn workspace_status(record: &WorkspaceMemoryRecord) -> VisibleMemoryStatus {
    if record.contested_count > 0 {
        VisibleMemoryStatus::Conflicted
    } else if record.candidate_count > 0 {
        VisibleMemoryStatus::Candidate
    } else {
        VisibleMemoryStatus::Current
    }
}

fn workspace_node_id(record: &WorkspaceMemoryRecord) -> Uuid {
    use std::hash::{Hash, Hasher};

    let title = workspace_title(record);
    let mut left = std::collections::hash_map::DefaultHasher::new();
    title.hash(&mut left);
    let mut right = std::collections::hash_map::DefaultHasher::new();
    "memd-workspace".hash(&mut right);
    title.hash(&mut right);
    let high = left.finish() as u128;
    let low = right.finish() as u128;
    Uuid::from_u128((high << 64) | low)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn test_insert_visible_item(
    state: &AppState,
    content: &str,
    preferred: bool,
) -> anyhow::Result<MemoryItem> {
    let req = memd_schema::StoreMemoryRequest {
        content: content.to_string(),
        kind: memd_schema::MemoryKind::Decision,
        scope: memd_schema::MemoryScope::Project,
        project: Some("memd".to_string()),
        namespace: Some("core".to_string()),
        workspace: Some("team-alpha".to_string()),
        visibility: Some(MemoryVisibility::Workspace),
        belief_branch: None,
        source_agent: Some("codex".to_string()),
        source_system: Some("obsidian".to_string()),
        source_path: Some(format!("wiki/{}.md", content.replace(' ', "-"))),
        source_quality: Some(memd_schema::SourceQuality::Derived),
        confidence: Some(0.9),
        ttl_seconds: None,
        last_verified_at: Some(Utc::now()),
        supersedes: Vec::new(),
        tags: vec!["visible-memory".to_string()],
        status: Some(MemoryStatus::Active),
        lane: None,
    };
    let (mut item, _) = state.store_item(req, MemoryStage::Canonical)?;
    item.preferred = preferred;
    item.updated_at = Utc::now();
    let canonical_key = canonical_key(&item);
    let redundancy_key = redundancy_key(&item);
    state.store.update(&item, &canonical_key, &redundancy_key)?;
    Ok(item)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_state() -> AppState {
        let path = std::env::temp_dir().join(format!("memd-visible-ui-{}.db", Uuid::new_v4()));
        AppState {
            store: crate::SqliteStore::open(&path).unwrap(),
            latency: crate::latency::LatencyHistogram::new(),
            rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new()),
            rag: None,
            embedder: None,
        }
    }

    #[test]
    fn builds_visible_memory_snapshot_from_stored_state() {
        let state = test_state();
        let preferred = test_insert_visible_item(&state, "runtime spine", true).unwrap();
        let newer = test_insert_visible_item(&state, "awareness lane", false).unwrap();
        let candidate = test_insert_candidate_item(&state, "candidate note").unwrap();
        let stale = test_insert_stale_item(&state, "stale belief").unwrap();
        assert!(newer.updated_at >= preferred.updated_at);

        let snapshot = build_visible_memory_snapshot(&state).unwrap();
        assert_eq!(snapshot.home.focus_artifact.id, preferred.id);
        assert_eq!(snapshot.home.focus_artifact.body, "runtime spine");
        assert_eq!(
            snapshot.home.focus_artifact.status,
            VisibleMemoryStatus::Current
        );
        assert_eq!(snapshot.home.focus_artifact.repair_state, "healthy");
        assert_eq!(
            snapshot.home.focus_artifact.provenance.source_system,
            Some("obsidian".to_string())
        );
        assert_eq!(snapshot.home.inbox_count, 2);
        assert_eq!(snapshot.home.repair_count, 4);
        assert_eq!(snapshot.home.awareness_count, 1);
        assert!(
            snapshot
                .knowledge_map
                .nodes
                .iter()
                .any(|node| node.artifact_kind == "timeline_event")
        );
        assert!(
            snapshot
                .knowledge_map
                .nodes
                .iter()
                .any(|node| node.artifact_kind == "workspace_lane")
        );
        assert!(snapshot.knowledge_map.nodes.len() >= 5);
        assert!(snapshot.knowledge_map.edges.len() >= 4);
        assert_eq!(candidate.status, MemoryStatus::Active);
        assert_eq!(stale.status, MemoryStatus::Stale);
    }

    #[test]
    fn visible_memory_snapshot_contains_dashboard_sections() {
        let state = test_state();
        test_insert_visible_item(&state, "runtime spine", true).unwrap();
        test_insert_candidate_item(&state, "candidate note").unwrap();
        let snapshot = build_visible_memory_snapshot(&state).unwrap();

        assert_eq!(snapshot.home.focus_artifact.title, "runtime-spine");
        assert_eq!(
            snapshot
                .home
                .focus_artifact
                .provenance
                .source_path
                .as_deref(),
            Some("wiki/runtime-spine.md")
        );
        assert!(
            snapshot
                .knowledge_map
                .nodes
                .iter()
                .any(|node| node.artifact_kind == "timeline_event")
        );
        assert!(
            snapshot
                .knowledge_map
                .nodes
                .iter()
                .any(|node| node.artifact_kind == "workspace_lane")
        );
        assert!(snapshot.home.inbox_count > 0);
        assert!(snapshot.home.repair_count > 0);
    }

    #[test]
    fn builds_visible_memory_artifact_detail_from_stored_state() {
        let state = test_state();
        let item = test_insert_visible_item(&state, "runtime spine", true).unwrap();
        let _related = test_insert_candidate_item(&state, "candidate note").unwrap();

        let detail = build_visible_memory_artifact_detail(&state, item.id).unwrap();

        assert_eq!(detail.artifact.id, item.id);
        assert_eq!(detail.artifact.title, "runtime-spine");
        assert!(detail.explain.is_some());
        assert!(!detail.sources.sources.is_empty());
        assert_eq!(detail.workspaces.workspaces.len(), 1);
        assert_eq!(detail.sessions.sessions.len(), 0);
        assert_eq!(detail.tasks.tasks.len(), 0);
        assert_eq!(detail.claims.claims.len(), 0);
        assert!(
            detail
                .related_artifacts
                .iter()
                .any(|artifact| artifact.title == "candidate-note")
        );
        assert!(!detail.actions.is_empty());
    }

    #[test]
    fn visible_memory_action_response_builds_obsidian_metadata() {
        let state = test_state();
        let item = test_insert_visible_item(&state, "runtime spine", true).unwrap();

        let response = perform_visible_memory_action(
            &state,
            VisibleMemoryUiActionRequest {
                id: item.id,
                action: VisibleMemoryUiActionKind::OpenInObsidian,
            },
        )
        .unwrap();

        assert_eq!(response.artifact_id, item.id);
        assert_eq!(response.action, VisibleMemoryUiActionKind::OpenInObsidian);
        assert_eq!(response.outcome, "metadata");
        assert!(
            response
                .open_uri
                .as_deref()
                .is_some_and(|uri| uri.starts_with("obsidian://open?path="))
        );
        assert_eq!(
            response.source_path.as_deref(),
            Some("wiki/runtime-spine.md")
        );
        assert!(response.detail.is_some());
    }

    #[test]
    fn visible_memory_action_can_verify_and_mark_stale() {
        let state = test_state();
        let item = test_insert_stale_item(&state, "stale belief").unwrap();

        let verified = perform_visible_memory_action(
            &state,
            VisibleMemoryUiActionRequest {
                id: item.id,
                action: VisibleMemoryUiActionKind::VerifyCurrent,
            },
        )
        .unwrap();
        assert_eq!(verified.outcome, "verified");
        assert!(verified.detail.is_some());

        let marked_stale = perform_visible_memory_action(
            &state,
            VisibleMemoryUiActionRequest {
                id: item.id,
                action: VisibleMemoryUiActionKind::MarkStale,
            },
        )
        .unwrap();
        assert_eq!(marked_stale.outcome, "marked_stale");
        assert!(marked_stale.detail.is_some());
    }

    #[test]
    fn visible_memory_action_can_promote() {
        let state = test_state();
        let item = test_insert_candidate_item(&state, "candidate note").unwrap();

        let promoted = perform_visible_memory_action(
            &state,
            VisibleMemoryUiActionRequest {
                id: item.id,
                action: VisibleMemoryUiActionKind::Promote,
            },
        )
        .unwrap();

        assert_eq!(promoted.outcome, "promoted");
        assert!(promoted.detail.is_some());
        let stored = state
            .store
            .get(item.id)
            .unwrap()
            .expect("stored promoted item");
        assert_eq!(stored.stage, MemoryStage::Canonical);
    }

    #[test]
    fn visible_memory_provenance_trust_reason_exposes_epistemic_state() {
        let state = test_state();
        let verified = test_insert_visible_item(&state, "runtime spine", true).unwrap();
        let inferred = test_insert_candidate_item(&state, "candidate note").unwrap();
        let stale = test_insert_stale_item(&state, "stale belief").unwrap();

        assert!(trust_reason(&verified, VisibleMemoryStatus::Current).contains("verified"));
        assert!(trust_reason(&inferred, VisibleMemoryStatus::Candidate).contains("inferred"));
        assert!(trust_reason(&stale, VisibleMemoryStatus::Stale).contains("stale"));
    }

    #[test]
    fn visible_memory_detail_explain_exposes_epistemic_state() {
        let state = test_state();
        let item = state
            .store_item(
                memd_schema::StoreMemoryRequest {
                    content: "claimed spine".to_string(),
                    kind: memd_schema::MemoryKind::Decision,
                    scope: memd_schema::MemoryScope::Project,
                    project: Some("memd".to_string()),
                    namespace: Some("core".to_string()),
                    workspace: Some("team-alpha".to_string()),
                    visibility: Some(MemoryVisibility::Workspace),
                    belief_branch: None,
                    source_agent: Some("codex".to_string()),
                    source_system: Some("obsidian".to_string()),
                    source_path: Some("wiki/claimed-spine.md".to_string()),
                    source_quality: Some(memd_schema::SourceQuality::Canonical),
                    confidence: Some(0.82),
                    ttl_seconds: None,
                    last_verified_at: None,
                    supersedes: Vec::new(),
                    tags: vec!["visible-memory".to_string()],
                    status: Some(MemoryStatus::Active),
                    lane: None,
                },
                MemoryStage::Canonical,
            )
            .unwrap()
            .0;

        let detail = build_visible_memory_artifact_detail(&state, item.id).unwrap();
        let explain = detail.explain.expect("explain payload");
        assert!(
            explain
                .reasons
                .iter()
                .any(|reason| reason == "epistemic_state=claimed")
        );
        assert!(
            explain
                .reasons
                .iter()
                .any(|reason| reason == "claimed_memory")
        );
    }

    #[test]
    fn visible_memory_freshness_labels_distinguish_claimed_inferred_and_verified() {
        let state = test_state();
        let verified = test_insert_visible_item(&state, "runtime spine", true).unwrap();
        let inferred = state
            .store_item(
                memd_schema::StoreMemoryRequest {
                    content: "inferred spine".to_string(),
                    kind: memd_schema::MemoryKind::Decision,
                    scope: memd_schema::MemoryScope::Project,
                    project: Some("memd".to_string()),
                    namespace: Some("core".to_string()),
                    workspace: Some("team-alpha".to_string()),
                    visibility: Some(MemoryVisibility::Workspace),
                    belief_branch: None,
                    source_agent: Some("codex".to_string()),
                    source_system: Some("obsidian".to_string()),
                    source_path: Some("wiki/inferred-spine.md".to_string()),
                    source_quality: Some(memd_schema::SourceQuality::Derived),
                    confidence: Some(0.84),
                    ttl_seconds: None,
                    last_verified_at: None,
                    supersedes: Vec::new(),
                    tags: vec!["visible-memory".to_string()],
                    status: Some(MemoryStatus::Active),
                    lane: None,
                },
                MemoryStage::Canonical,
            )
            .unwrap()
            .0;
        let claimed = state
            .store_item(
                memd_schema::StoreMemoryRequest {
                    content: "claimed spine".to_string(),
                    kind: memd_schema::MemoryKind::Decision,
                    scope: memd_schema::MemoryScope::Project,
                    project: Some("memd".to_string()),
                    namespace: Some("core".to_string()),
                    workspace: Some("team-alpha".to_string()),
                    visibility: Some(MemoryVisibility::Workspace),
                    belief_branch: None,
                    source_agent: Some("codex".to_string()),
                    source_system: Some("obsidian".to_string()),
                    source_path: Some("wiki/claimed-spine.md".to_string()),
                    source_quality: Some(memd_schema::SourceQuality::Canonical),
                    confidence: Some(0.82),
                    ttl_seconds: None,
                    last_verified_at: None,
                    supersedes: Vec::new(),
                    tags: vec!["visible-memory".to_string()],
                    status: Some(MemoryStatus::Active),
                    lane: None,
                },
                MemoryStage::Canonical,
            )
            .unwrap()
            .0;

        assert_eq!(
            freshness_label(&verified, VisibleMemoryStatus::Current),
            "verified"
        );
        assert_eq!(
            freshness_label(&inferred, VisibleMemoryStatus::Current),
            "inferred"
        );
        assert_eq!(
            freshness_label(&claimed, VisibleMemoryStatus::Current),
            "claimed"
        );
    }

    fn test_insert_candidate_item(state: &AppState, content: &str) -> anyhow::Result<MemoryItem> {
        let mut item = state
            .store_item(
                memd_schema::StoreMemoryRequest {
                    content: content.to_string(),
                    kind: memd_schema::MemoryKind::Decision,
                    scope: memd_schema::MemoryScope::Project,
                    project: Some("memd".to_string()),
                    namespace: Some("core".to_string()),
                    workspace: Some("team-alpha".to_string()),
                    visibility: Some(MemoryVisibility::Workspace),
                    belief_branch: None,
                    source_agent: Some("codex".to_string()),
                    source_system: Some("obsidian".to_string()),
                    source_path: Some(format!("wiki/{}.md", content.replace(' ', "-"))),
                    source_quality: Some(memd_schema::SourceQuality::Derived),
                    confidence: Some(0.72),
                    ttl_seconds: None,
                    last_verified_at: None,
                    supersedes: Vec::new(),
                    tags: vec!["visible-memory".to_string()],
                    status: Some(MemoryStatus::Active),
                    lane: None,
                },
                MemoryStage::Candidate,
            )?
            .0;
        item.updated_at = Utc::now();
        let canonical_key = canonical_key(&item);
        let redundancy_key = redundancy_key(&item);
        state.store.update(&item, &canonical_key, &redundancy_key)?;
        Ok(item)
    }

    fn test_insert_stale_item(state: &AppState, content: &str) -> anyhow::Result<MemoryItem> {
        let mut item = state
            .store_item(
                memd_schema::StoreMemoryRequest {
                    content: content.to_string(),
                    kind: memd_schema::MemoryKind::Decision,
                    scope: memd_schema::MemoryScope::Project,
                    project: Some("memd".to_string()),
                    namespace: Some("core".to_string()),
                    workspace: Some("team-alpha".to_string()),
                    visibility: Some(MemoryVisibility::Workspace),
                    belief_branch: None,
                    source_agent: Some("codex".to_string()),
                    source_system: Some("obsidian".to_string()),
                    source_path: Some(format!("wiki/{}.md", content.replace(' ', "-"))),
                    source_quality: Some(memd_schema::SourceQuality::Derived),
                    confidence: Some(0.6),
                    ttl_seconds: None,
                    last_verified_at: None,
                    supersedes: Vec::new(),
                    tags: vec!["visible-memory".to_string()],
                    status: Some(MemoryStatus::Stale),
                    lane: None,
                },
                MemoryStage::Canonical,
            )?
            .0;
        item.updated_at = Utc::now();
        let canonical_key = canonical_key(&item);
        let redundancy_key = redundancy_key(&item);
        state.store.update(&item, &canonical_key, &redundancy_key)?;
        Ok(item)
    }
}
