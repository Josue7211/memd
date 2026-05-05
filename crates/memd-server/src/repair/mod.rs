use axum::http::StatusCode;
use chrono::Utc;
use memd_schema::{
    CaptureSource, CorrectMemoryRequest, CorrectMemoryResponse, CorrectionMetadata,
    ExpireMemoryRequest, MemoryItem, MemoryRepairMode, MemoryStage, MemoryStatus,
    RepairMemoryRequest, RepairMemoryResponse, StoreMemoryRequest, VerifyMemoryRequest,
};
use tracing::warn;

use crate::{
    AppState, RecordEventArgs, canonical_key, errors::MemdError, internal_error, redundancy_key,
};

pub(crate) fn expire_item(
    state: &AppState,
    req: ExpireMemoryRequest,
) -> Result<MemoryItem, (StatusCode, String)> {
    let mut item = state
        .store
        .get(req.id)
        .map_err(internal_error)?
        .ok_or_else(|| MemdError::not_found("memory item", req.id).into_wire())?;

    item.status = req.status.unwrap_or(MemoryStatus::Expired);
    item.updated_at = Utc::now();
    let canonical_key = canonical_key(&item);
    let redundancy_key = redundancy_key(&item);
    let item = MemoryItem {
        redundancy_key: Some(redundancy_key.clone()),
        ..item
    };
    state
        .store
        .update(&item, &canonical_key, &redundancy_key)
        .map_err(internal_error)?;
    if let Err(e) = record_lifecycle_event(state, &item, "expired", "memory item marked expired") {
        warn!(error = %format_args!("{e:#}"), "record_lifecycle_event (expired)");
    }
    Ok(item)
}

pub(crate) fn verify_item(
    state: &AppState,
    req: VerifyMemoryRequest,
) -> Result<MemoryItem, (StatusCode, String)> {
    repair_item(
        state,
        RepairMemoryRequest {
            id: req.id,
            mode: MemoryRepairMode::Verify,
            confidence: req.confidence,
            status: req.status,
            workspace: None,
            visibility: None,
            source_agent: None,
            source_system: None,
            source_path: None,
            source_quality: None,
            content: None,
            tags: None,
            supersedes: vec![],
        },
    )
    .map(|response| response.item)
}

pub(crate) fn correct_item(
    state: &AppState,
    req: CorrectMemoryRequest,
) -> Result<CorrectMemoryResponse, (StatusCode, String)> {
    if req.content.trim().is_empty() {
        return Err(MemdError::validation("content", "cannot be empty").into_wire());
    }
    let old_item = state
        .store
        .get(req.id)
        .map_err(internal_error)?
        .ok_or_else(|| MemdError::not_found("memory item", req.id).into_wire())?;

    // 1. Mark old item Superseded
    let mut superseded = old_item.clone();
    superseded.status = MemoryStatus::Superseded;
    superseded.updated_at = Utc::now();
    let old_canonical = canonical_key(&superseded);
    let old_redundancy = redundancy_key(&superseded);
    let superseded = MemoryItem {
        redundancy_key: Some(old_redundancy.clone()),
        ..superseded
    };
    state
        .store
        .update(&superseded, &old_canonical, &old_redundancy)
        .map_err(internal_error)?;
    state
        .store
        .close_links_for_source_item(old_item.id, superseded.updated_at)
        .map_err(internal_error)?;
    if let Err(e) = record_lifecycle_event(
        state,
        &superseded,
        "superseded_by_correction",
        &format!("memory item superseded by correction"),
    ) {
        warn!(error = %format_args!("{e:#}"), "record_lifecycle_event (superseded_by_correction)");
    }

    // 2. Create new item with corrected content
    let mut new_tags = req.tags.unwrap_or_else(|| old_item.tags.clone());
    if !new_tags.contains(&"correction".to_string()) {
        new_tags.push("correction".to_string());
    }
    let correction_confidence = req
        .confidence
        .unwrap_or(old_item.confidence)
        .clamp(0.0, 1.0);
    let correction_meta = CorrectionMetadata {
        corrects_id: Some(old_item.id),
        source_turn: correction_source_turn(&new_tags),
        captured_by: Some(CaptureSource::Manual),
        confidence: Some(correction_confidence),
    };
    let store_req = StoreMemoryRequest {
        content: req.content.trim().to_string(),
        kind: old_item.kind,
        scope: old_item.scope,
        project: old_item.project.clone(),
        namespace: old_item.namespace.clone(),
        workspace: old_item.workspace.clone(),
        visibility: Some(old_item.visibility),
        belief_branch: old_item.belief_branch.clone(),
        source_agent: old_item.source_agent.clone(),
        source_system: old_item.source_system.clone(),
        source_path: old_item.source_path.clone(),
        source_quality: old_item.source_quality,
        confidence: Some(correction_confidence),
        ttl_seconds: old_item.ttl_seconds,
        last_verified_at: Some(Utc::now()),
        supersedes: vec![old_item.id],
        tags: new_tags,
        status: Some(MemoryStatus::Active),
        lane: old_item.lane.clone(),
    };
    let (mut new_item, _duplicate) = state
        .store_item(store_req, MemoryStage::Canonical)
        .map_err(internal_error)?;

    // Corrections outrank the original in retrieval
    new_item.preferred = true;
    new_item.correction_meta = Some(correction_meta);
    new_item.updated_at = Utc::now();
    let pref_canonical = canonical_key(&new_item);
    let pref_redundancy = redundancy_key(&new_item);
    let new_item = MemoryItem {
        redundancy_key: Some(pref_redundancy.clone()),
        ..new_item
    };
    state
        .store
        .update(&new_item, &pref_canonical, &pref_redundancy)
        .map_err(internal_error)?;

    if let Err(e) = record_lifecycle_event(
        state,
        &new_item,
        "correction_created",
        &format!(
            "correction of item {} — {}",
            old_item.id,
            req.reason.as_deref().unwrap_or("content corrected")
        ),
    ) {
        warn!(error = %format_args!("{e:#}"), "record_lifecycle_event (correction_created)");
    }

    // 3. Contradiction detection: entity-based matching.
    //    Look up the OLD item's entity (not new — new has different content so
    //    canonical_key yields a different entity). The old item's entity is the
    //    one that siblings share. Mark Active siblings with different content Contested.
    let mut contested = Vec::new();
    if let Ok(Some(entity)) = state.store.entity_for_item(old_item.id) {
        if let Ok(siblings) = state.store.items_for_entity(entity.id) {
            for mut sibling in siblings {
                if sibling.id == new_item.id || sibling.id == old_item.id {
                    continue;
                }
                if sibling.status != MemoryStatus::Active {
                    continue;
                }
                if sibling.kind != new_item.kind
                    || sibling.scope != new_item.scope
                    || sibling.project != new_item.project
                {
                    continue;
                }
                if sibling.content != new_item.content {
                    sibling.status = MemoryStatus::Contested;
                    sibling.updated_at = Utc::now();
                    let sib_canonical = canonical_key(&sibling);
                    let sib_redundancy = redundancy_key(&sibling);
                    let sibling = MemoryItem {
                        redundancy_key: Some(sib_redundancy.clone()),
                        ..sibling
                    };
                    state
                        .store
                        .update(&sibling, &sib_canonical, &sib_redundancy)
                        .map_err(internal_error)?;
                    contested.push(sibling.id);
                }
            }
        }
    }

    Ok(CorrectMemoryResponse {
        old_item: superseded,
        new_item,
        contested,
    })
}

fn correction_source_turn(tags: &[String]) -> Option<String> {
    tags.iter()
        .find_map(|tag| {
            tag.strip_prefix("turn:")
                .filter(|value| !value.trim().is_empty())
                .map(|value| value.to_string())
        })
        .or_else(|| {
            tags.iter()
                .find(|tag| tag.as_str() != "correction" && tag.contains("correct"))
                .cloned()
        })
}

pub(crate) fn repair_item(
    state: &AppState,
    req: RepairMemoryRequest,
) -> Result<RepairMemoryResponse, (StatusCode, String)> {
    let mut item = state
        .store
        .get(req.id)
        .map_err(internal_error)?
        .ok_or_else(|| MemdError::not_found("memory item", req.id).into_wire())?;
    let mut reasons = vec![format!("mode={}", format_mode(req.mode))];
    let event_type = match req.mode {
        MemoryRepairMode::Verify => {
            item.last_verified_at = Some(Utc::now());
            item.status = req.status.unwrap_or(MemoryStatus::Active);
            if let Some(confidence) = req.confidence {
                item.confidence = confidence.clamp(0.0, 1.0);
                reasons.push("confidence_updated".to_string());
            }
            "verified"
        }
        MemoryRepairMode::Expire => {
            item.status = req.status.unwrap_or(MemoryStatus::Expired);
            if let Some(confidence) = req.confidence {
                item.confidence = confidence.clamp(0.0, 1.0);
                reasons.push("confidence_updated".to_string());
            }
            "expired"
        }
        MemoryRepairMode::Supersede => {
            item.status = req.status.unwrap_or(MemoryStatus::Superseded);
            if !req.supersedes.is_empty() {
                item.supersedes.extend(req.supersedes.clone());
                item.supersedes.sort_unstable();
                item.supersedes.dedup();
                reasons.push(format!("supersedes={}", req.supersedes.len()));
            }
            if let Some(confidence) = req.confidence {
                item.confidence = confidence.clamp(0.0, 1.0);
                reasons.push("confidence_updated".to_string());
            }
            "superseded"
        }
        MemoryRepairMode::Contest => {
            item.status = req.status.unwrap_or(MemoryStatus::Contested);
            if let Some(confidence) = req.confidence {
                item.confidence = confidence.clamp(0.0, 1.0);
                reasons.push("confidence_updated".to_string());
            }
            "contested"
        }
        MemoryRepairMode::PreferBranch => {
            item.preferred = true;
            item.status = req.status.unwrap_or(MemoryStatus::Active);
            if let Some(confidence) = req.confidence {
                item.confidence = confidence.clamp(0.0, 1.0);
                reasons.push("confidence_updated".to_string());
            }
            let siblings = state.snapshot().map_err(internal_error)?;
            let canonical = canonical_key(&item);
            let redundancy = redundancy_key(&item);
            let mut demoted = 0usize;
            for mut sibling in siblings {
                if sibling.id == item.id {
                    continue;
                }
                if sibling.kind != item.kind
                    || sibling.scope != item.scope
                    || sibling.project != item.project
                    || sibling.namespace != item.namespace
                    || sibling.redundancy_key.as_deref() != Some(redundancy.as_str())
                    || canonical_key(&sibling) == canonical
                    || !sibling.preferred
                {
                    continue;
                }
                sibling.preferred = false;
                sibling.updated_at = Utc::now();
                let sibling_canonical_key = canonical_key(&sibling);
                let sibling_redundancy_key = redundancy_key(&sibling);
                let sibling = MemoryItem {
                    redundancy_key: Some(sibling_redundancy_key.clone()),
                    ..sibling
                };
                state
                    .store
                    .update(&sibling, &sibling_canonical_key, &sibling_redundancy_key)
                    .map_err(internal_error)?;
                demoted += 1;
            }
            reasons.push("preferred_branch_selected".to_string());
            if demoted > 0 {
                reasons.push(format!("preferred_branch_cleared={demoted}"));
            }
            "preferred_branch"
        }
        MemoryRepairMode::CorrectMetadata => {
            if let Some(workspace) = req.workspace {
                let workspace = workspace.trim().to_string();
                item.workspace = if workspace.is_empty() {
                    None
                } else {
                    Some(workspace)
                };
                reasons.push("workspace_updated".to_string());
            }
            if let Some(visibility) = req.visibility {
                item.visibility = visibility;
                reasons.push("visibility_updated".to_string());
            }
            if let Some(source_agent) = req.source_agent {
                item.source_agent = Some(source_agent);
                reasons.push("source_agent_updated".to_string());
            }
            if let Some(source_system) = req.source_system {
                item.source_system = Some(source_system);
                reasons.push("source_system_updated".to_string());
            }
            if let Some(source_path) = req.source_path {
                item.source_path = Some(source_path);
                reasons.push("source_path_updated".to_string());
            }
            if let Some(source_quality) = req.source_quality {
                item.source_quality = Some(source_quality);
                reasons.push("source_quality_updated".to_string());
            }
            if let Some(tags) = req.tags {
                item.tags = tags;
                reasons.push("tags_updated".to_string());
            }
            if let Some(content) = req.content {
                let content = content.trim().to_string();
                if content.is_empty() {
                    return Err(MemdError::validation("content", "cannot be empty").into_wire());
                }
                item.content = content;
                reasons.push("content_repaired".to_string());
            }
            if let Some(confidence) = req.confidence {
                item.confidence = confidence.clamp(0.0, 1.0);
                reasons.push("confidence_updated".to_string());
            }
            if let Some(status) = req.status {
                item.status = status;
                reasons.push("status_updated".to_string());
            }
            if !req.supersedes.is_empty() {
                item.supersedes.extend(req.supersedes.clone());
                item.supersedes.sort_unstable();
                item.supersedes.dedup();
                reasons.push(format!("supersedes={}", req.supersedes.len()));
            }
            "metadata_corrected"
        }
    };

    item.updated_at = Utc::now();
    let canonical_key = canonical_key(&item);
    let redundancy_key = redundancy_key(&item);
    let item = MemoryItem {
        redundancy_key: Some(redundancy_key.clone()),
        ..item
    };
    state
        .store
        .update(&item, &canonical_key, &redundancy_key)
        .map_err(internal_error)?;
    let summary = format!("memory item {} via {}", event_type, format_mode(req.mode));
    if let Err(e) = record_lifecycle_event(state, &item, event_type, &summary) {
        warn!(error = %format_args!("{e:#}"), %event_type, "record_lifecycle_event");
    }
    Ok(RepairMemoryResponse {
        item,
        mode: req.mode,
        reasons,
    })
}

fn record_lifecycle_event(
    state: &AppState,
    item: &MemoryItem,
    event_type: &str,
    summary: &str,
) -> anyhow::Result<()> {
    let canonical_key = canonical_key(item);
    let entity = state.store.resolve_entity_for_item(item, &canonical_key)?;
    let context = Some(super::entity_context_frame(&entity.record, item));
    state.store.record_event(
        &entity.record,
        item.id,
        RecordEventArgs {
            event_type: event_type.to_string(),
            summary: summary.to_string(),
            occurred_at: item.updated_at,
            project: item.project.clone(),
            namespace: item.namespace.clone(),
            workspace: item.workspace.clone(),
            source_agent: item.source_agent.clone(),
            source_system: item.source_system.clone(),
            source_path: item.source_path.clone(),
            related_entity_ids: Vec::new(),
            tags: item.tags.clone(),
            context,
            confidence: item.confidence,
            salience_score: entity.record.salience_score,
        },
    )?;
    Ok(())
}

fn format_mode(mode: MemoryRepairMode) -> &'static str {
    match mode {
        MemoryRepairMode::Verify => "verify",
        MemoryRepairMode::Expire => "expire",
        MemoryRepairMode::Supersede => "supersede",
        MemoryRepairMode::Contest => "contest",
        MemoryRepairMode::PreferBranch => "prefer_branch",
        MemoryRepairMode::CorrectMetadata => "correct_metadata",
    }
}
