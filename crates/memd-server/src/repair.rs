use axum::http::StatusCode;
use chrono::Utc;
use memd_schema::{
    ExpireMemoryRequest, MemoryItem, MemoryRepairMode, MemoryStatus, RepairMemoryRequest,
    RepairMemoryResponse, VerifyMemoryRequest,
};

use super::{canonical_key, internal_error, redundancy_key, AppState};

pub(crate) fn expire_item(
    state: &AppState,
    req: ExpireMemoryRequest,
) -> Result<MemoryItem, (StatusCode, String)> {
    let mut item = state
        .store
        .get(req.id)
        .map_err(internal_error)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "memory item not found".to_string()))?;

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
    let _ = record_lifecycle_event(state, &item, "expired", "memory item marked expired");
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

pub(crate) fn repair_item(
    state: &AppState,
    req: RepairMemoryRequest,
) -> Result<RepairMemoryResponse, (StatusCode, String)> {
    let mut item = state
        .store
        .get(req.id)
        .map_err(internal_error)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "memory item not found".to_string()))?;
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
        MemoryRepairMode::CorrectMetadata => {
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
                    return Err((StatusCode::BAD_REQUEST, "content cannot be empty".to_string()));
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
    let _ = record_lifecycle_event(state, &item, event_type, &summary);
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
        event_type,
        summary.to_string(),
        item.updated_at,
        item.project.clone(),
        item.namespace.clone(),
        item.source_agent.clone(),
        item.source_system.clone(),
        item.source_path.clone(),
        vec![],
        item.tags.clone(),
        context,
        item.confidence,
        entity.record.salience_score,
    )?;
    Ok(())
}

fn format_mode(mode: MemoryRepairMode) -> &'static str {
    match mode {
        MemoryRepairMode::Verify => "verify",
        MemoryRepairMode::Expire => "expire",
        MemoryRepairMode::Supersede => "supersede",
        MemoryRepairMode::Contest => "contest",
        MemoryRepairMode::CorrectMetadata => "correct_metadata",
    }
}
