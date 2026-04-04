use axum::http::StatusCode;
use chrono::Utc;
use memd_schema::{ExpireMemoryRequest, MemoryItem, MemoryStatus, VerifyMemoryRequest};

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
    let mut item = state
        .store
        .get(req.id)
        .map_err(internal_error)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "memory item not found".to_string()))?;

    item.last_verified_at = Some(Utc::now());
    if let Some(confidence) = req.confidence {
        item.confidence = confidence.clamp(0.0, 1.0);
    }
    item.status = req.status.unwrap_or(MemoryStatus::Active);
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
    let _ = record_lifecycle_event(state, &item, "verified", "memory item reverified");
    Ok(item)
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
