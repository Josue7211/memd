use axum::http::StatusCode;
use chrono::Utc;
use uuid::Uuid;

use crate::{AppState, build_context, compact_record, internal_error};
use memd_schema::{
    AgentProfileRequest, CompactMemoryRecord, ContextRequest, MemoryConsolidationRequest,
    MemoryPolicyConsolidation, MemoryPolicyDecay, MemoryPolicyPromotion, MemoryPolicyResponse,
    MemoryPolicyRouteDefault, MemoryPolicyWorkingMemory, MemoryScope,
    WorkingMemoryEvictionRecord, WorkingMemoryPolicyState, WorkingMemoryRehydrationRecord,
    WorkingMemoryRequest, WorkingMemoryResponse, WorkingMemoryTraceRecord,
};

pub(crate) fn working_memory(
    state: &AppState,
    req: WorkingMemoryRequest,
) -> Result<WorkingMemoryResponse, (StatusCode, String)> {
    let req = apply_working_profile_defaults(&state, req).map_err(internal_error)?;
    let admission_limit = req.limit.unwrap_or(8).min(32);
    let rehydration_limit = req.rehydration_limit.unwrap_or(3).clamp(1, 12);
    let candidate_window = (admission_limit + rehydration_limit).min(32);
    let compact_req = ContextRequest {
        project: req.project.clone(),
        agent: req.agent.clone(),
        route: req.route,
        intent: req.intent,
        limit: Some(candidate_window),
        max_chars_per_item: req.max_chars_per_item,
    };
    let (plan, retrieval_order, items) = build_context(&state, &compact_req)?;
    state.rehearse_items(&items, 3).map_err(internal_error)?;
    let now = Utc::now();
    let mut ranked_items = items
        .into_iter()
        .map(|item| {
            let score = working_item_priority(&item, now);
            (score, item)
        })
        .collect::<Vec<_>>();
    ranked_items.sort_by(|left, right| right.0.total_cmp(&left.0));
    let selected_items = ranked_items
        .into_iter()
        .map(|(_, item)| item)
        .collect::<Vec<_>>();

    let budget_chars = req.max_total_chars.unwrap_or(1600).clamp(400, 8000);
    let max_chars_per_item = req.max_chars_per_item.unwrap_or(220).clamp(80, 2000);
    let mut used_chars = 0usize;
    let mut truncated = false;
    let mut records = Vec::new();
    let mut evicted = Vec::new();

    let compacted_records = selected_items
        .iter()
        .map(|item| {
            let mut record = compact_record(item);
            if record.chars().count() > max_chars_per_item {
                record = record
                    .chars()
                    .take(max_chars_per_item.saturating_sub(3))
                    .collect();
                record.push_str("...");
            }
            (item.id, record)
        })
        .collect::<Vec<_>>();

    for (index, (item_id, record)) in compacted_records.iter().enumerate() {
        let record_chars = record.chars().count();
        if used_chars + record_chars > budget_chars {
            truncated = true;
            evicted.push(WorkingMemoryEvictionRecord {
                id: *item_id,
                record: record.clone(),
                reason: "evicted_by_budget".to_string(),
            });
            continue;
        }
        used_chars += record_chars;
        records.push(CompactMemoryRecord {
            id: *item_id,
            record: record.clone(),
        });
        if index + 1 >= admission_limit {
            truncated = compacted_records.len() > records.len();
        }
    }

    if records.len() > admission_limit {
        for record in records.drain(admission_limit..) {
            evicted.push(WorkingMemoryEvictionRecord {
                id: record.id,
                record: record.record,
                reason: "evicted_by_admission_limit".to_string(),
            });
        }
        truncated = true;
    }

    let rehydration_queue = evicted
        .iter()
        .take(rehydration_limit)
        .map(|entry| WorkingMemoryRehydrationRecord {
            id: entry.id,
            record: entry.record.clone(),
            reason: entry.reason.clone(),
        })
        .collect::<Vec<_>>();

    let traces = working_traces_for_items(&state, &selected_items, 3).map_err(internal_error)?;
    let semantic_consolidation = if req.auto_consolidate.unwrap_or(false) {
        let auto_request = MemoryConsolidationRequest {
            project: req.project.clone(),
            namespace: req.agent.clone(),
            max_groups: Some(8),
            min_events: Some(3),
            lookback_days: Some(14),
            min_salience: Some(0.22),
            record_events: Some(true),
        };
        Some(
            state
                .consolidate_semantic_memory(&auto_request)
                .map_err(internal_error)?,
        )
    } else {
        None
    };

    Ok(WorkingMemoryResponse {
        route: plan.route,
        intent: plan.intent,
        retrieval_order,
        budget_chars,
        used_chars,
        remaining_chars: budget_chars.saturating_sub(used_chars),
        truncated,
        policy: WorkingMemoryPolicyState {
            admission_limit,
            max_chars_per_item,
            budget_chars,
            rehydration_limit,
        },
        records,
        evicted,
        rehydration_queue,
        traces,
        semantic_consolidation,
    })
}

fn apply_working_profile_defaults(
    state: &AppState,
    mut req: WorkingMemoryRequest,
) -> anyhow::Result<WorkingMemoryRequest> {
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
        if req.max_total_chars.is_none() {
            req.max_total_chars = profile.max_total_chars;
        }
        if req.limit.is_none() && profile.recall_depth.is_some() {
            req.limit = profile.recall_depth;
        }
    }

    Ok(req)
}

fn working_traces_for_items(
    state: &AppState,
    items: &[memd_schema::MemoryItem],
    limit: usize,
) -> anyhow::Result<Vec<WorkingMemoryTraceRecord>> {
    let mut traces = Vec::new();
    for item in items.iter().take(limit) {
        let (entity, events) = state.entity_view(item.id, 1)?;
        let Some(event) = events.first() else {
            continue;
        };
        traces.push(WorkingMemoryTraceRecord {
            item_id: item.id,
            entity_id: entity.as_ref().map(|entity| entity.id),
            event_type: event.event_type.clone(),
            summary: event.summary.clone(),
            occurred_at: event.occurred_at,
            salience_score: event.salience_score,
        });
    }
    Ok(traces)
}

fn working_item_priority(item: &memd_schema::MemoryItem, now: chrono::DateTime<Utc>) -> f32 {
    let confidence = item.confidence.clamp(0.0, 1.0);
    let age_days = now
        .signed_duration_since(item.updated_at)
        .num_days()
        .max(0) as f32;
    let verification_days = item
        .last_verified_at
        .map(|verified| now.signed_duration_since(verified).num_days().max(0) as f32)
        .unwrap_or(45.0);

    let status_score = match item.status {
        memd_schema::MemoryStatus::Active => 0.22,
        memd_schema::MemoryStatus::Stale => -0.10,
        memd_schema::MemoryStatus::Superseded => -0.20,
        memd_schema::MemoryStatus::Contested => -0.18,
        memd_schema::MemoryStatus::Expired => -0.28,
    };
    let source_score = match item.source_quality {
        Some(memd_schema::SourceQuality::Canonical) => 0.14,
        Some(memd_schema::SourceQuality::Derived) => 0.06,
        Some(memd_schema::SourceQuality::Synthetic) => -0.08,
        None => 0.0,
    };
    let stage_score = match item.stage {
        memd_schema::MemoryStage::Canonical => 0.08,
        memd_schema::MemoryStage::Candidate => -0.02,
    };
    let freshness_score = if age_days <= 2.0 {
        0.06
    } else if age_days >= 30.0 {
        -0.05
    } else {
        0.0
    };
    let verification_score = if verification_days <= 7.0 {
        0.08
    } else if verification_days >= 60.0 {
        -0.06
    } else {
        0.0
    };
    let ttl_score = match item.ttl_seconds {
        Some(ttl) if ttl <= 86_400 => -0.04,
        Some(ttl) if ttl >= 7 * 86_400 => 0.02,
        Some(_) => 0.0,
        None => 0.03,
    };

    (confidence * 0.48
        + status_score
        + source_score
        + stage_score
        + freshness_score
        + verification_score
        + ttl_score)
        .clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_item(
        status: memd_schema::MemoryStatus,
        source_quality: Option<memd_schema::SourceQuality>,
        confidence: f32,
        last_verified_at: Option<chrono::DateTime<Utc>>,
        updated_at: chrono::DateTime<Utc>,
    ) -> memd_schema::MemoryItem {
        let now = updated_at;
        memd_schema::MemoryItem {
            id: Uuid::new_v4(),
            content: "content".to_string(),
            redundancy_key: None,
            kind: memd_schema::MemoryKind::Fact,
            scope: memd_schema::MemoryScope::Project,
            project: Some("proj".to_string()),
            namespace: Some("ns".to_string()),
            source_agent: Some("agent".to_string()),
            source_system: Some("system".to_string()),
            source_path: Some("path".to_string()),
            source_quality,
            confidence,
            ttl_seconds: None,
            created_at: now,
            updated_at,
            last_verified_at,
            supersedes: vec![],
            tags: vec![],
            status,
            stage: memd_schema::MemoryStage::Canonical,
        }
    }

    #[test]
    fn active_recent_canonical_items_rank_above_stale_contested_items() {
        let now = Utc::now();
        let good = sample_item(
            memd_schema::MemoryStatus::Active,
            Some(memd_schema::SourceQuality::Canonical),
            0.95,
            Some(now),
            now,
        );
        let weak = sample_item(
            memd_schema::MemoryStatus::Contested,
            Some(memd_schema::SourceQuality::Synthetic),
            0.35,
            Some(now - chrono::Duration::days(90)),
            now - chrono::Duration::days(45),
        );

        assert!(working_item_priority(&good, now) > working_item_priority(&weak, now));
    }

    #[test]
    fn recently_verified_items_rank_above_unverified_items() {
        let now = Utc::now();
        let verified = sample_item(
            memd_schema::MemoryStatus::Active,
            Some(memd_schema::SourceQuality::Derived),
            0.8,
            Some(now),
            now - chrono::Duration::days(3),
        );
        let unverified = sample_item(
            memd_schema::MemoryStatus::Active,
            Some(memd_schema::SourceQuality::Derived),
            0.8,
            Some(now - chrono::Duration::days(80)),
            now - chrono::Duration::days(3),
        );

        assert!(working_item_priority(&verified, now) > working_item_priority(&unverified, now));
    }
}

pub(crate) fn memory_policy_snapshot() -> MemoryPolicyResponse {
    MemoryPolicyResponse {
        retrieval_order: vec![
            MemoryScope::Local,
            MemoryScope::Synced,
            MemoryScope::Project,
            MemoryScope::Global,
        ],
        route_defaults: vec![
            MemoryPolicyRouteDefault {
                intent: memd_schema::RetrievalIntent::General,
                route: memd_schema::RetrievalRoute::All,
            },
            MemoryPolicyRouteDefault {
                intent: memd_schema::RetrievalIntent::CurrentTask,
                route: memd_schema::RetrievalRoute::LocalFirst,
            },
            MemoryPolicyRouteDefault {
                intent: memd_schema::RetrievalIntent::Decision,
                route: memd_schema::RetrievalRoute::ProjectFirst,
            },
            MemoryPolicyRouteDefault {
                intent: memd_schema::RetrievalIntent::Runbook,
                route: memd_schema::RetrievalRoute::ProjectFirst,
            },
            MemoryPolicyRouteDefault {
                intent: memd_schema::RetrievalIntent::Topology,
                route: memd_schema::RetrievalRoute::ProjectFirst,
            },
            MemoryPolicyRouteDefault {
                intent: memd_schema::RetrievalIntent::Preference,
                route: memd_schema::RetrievalRoute::GlobalFirst,
            },
            MemoryPolicyRouteDefault {
                intent: memd_schema::RetrievalIntent::Fact,
                route: memd_schema::RetrievalRoute::All,
            },
            MemoryPolicyRouteDefault {
                intent: memd_schema::RetrievalIntent::Pattern,
                route: memd_schema::RetrievalRoute::GlobalFirst,
            },
        ],
        working_memory: MemoryPolicyWorkingMemory {
            budget_chars: 1600,
            max_chars_per_item: 220,
            default_limit: 8,
        },
        promotion: MemoryPolicyPromotion {
            min_salience: 0.22,
            min_events: 3,
            lookback_days: 14,
            default_ttl_days: 90,
        },
        decay: MemoryPolicyDecay {
            max_items: 128,
            inactive_days: 21,
            max_decay: 0.12,
            record_events: true,
        },
        consolidation: MemoryPolicyConsolidation {
            max_groups: 24,
            min_events: 3,
            lookback_days: 14,
            min_salience: 0.22,
            record_events: true,
        },
    }
}
