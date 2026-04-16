use axum::http::StatusCode;
use chrono::Utc;
use uuid::Uuid;

use crate::{AppState, BuildContextResult, build_context, compact_record, internal_error, store_entities::trust_rank};
use memd_schema::{
    AgentProfileRequest, CompactMemoryRecord, CompactionQualityReport, ContextRequest,
    MemoryConsolidationRequest, MemoryEntityRecord, MemoryKind, MemoryPolicyConsolidation,
    MemoryPolicyDecay, MemoryPolicyFeedback, MemoryPolicyPromotion, MemoryPolicyResponse,
    MemoryPolicyRouteDefault, MemoryPolicyWorkingMemory, MemoryRehydrationRecord, MemoryScope,
    MemoryStage, WorkingMemoryEvictionRecord, WorkingMemoryPolicyState, WorkingMemoryRequest,
    WorkingMemoryResponse, WorkingMemoryTraceRecord,
};

pub(crate) fn working_memory(
    state: &AppState,
    req: WorkingMemoryRequest,
) -> Result<WorkingMemoryResponse, (StatusCode, String)> {
    let policy = memory_policy_snapshot();
    let working_policy = &policy.working_memory;
    let consolidation_policy = &policy.consolidation;
    let source_trust_floor = policy.source_trust_floor;
    let req = apply_working_profile_defaults(state, req).map_err(internal_error)?;
    let admission_limit = req.limit.unwrap_or(working_policy.default_limit).min(32);
    let rehydration_limit = req
        .rehydration_limit
        .unwrap_or(working_policy.rehydration_limit)
        .clamp(1, 12);
    let candidate_window = (admission_limit + rehydration_limit).min(32);
    let compact_req = ContextRequest {
        project: req.project.clone(),
        agent: req.agent.clone(),
        workspace: req.workspace.clone(),
        visibility: req.visibility,
        route: req.route,
        intent: req.intent,
        limit: Some(candidate_window),
        max_chars_per_item: req.max_chars_per_item,
    };
    let BuildContextResult {
        plan,
        retrieval_order,
        items,
    } = build_context(state, &compact_req)?;
    let query_lane = req
        .query
        .as_deref()
        .and_then(|q| crate::helpers::detect_content_lane(q, None, &[]));
    state.rehearse_items(&items, 3).map_err(internal_error)?;
    state
        .record_retrieval_feedback(&items, 3, "retrieved_working", &plan)
        .map_err(internal_error)?;
    let now = Utc::now();
    let mut ranked_items = Vec::with_capacity(items.len());
    for item in items {
        let (entity, _) = state.entity_view(item.id, 1).map_err(internal_error)?;
        let source_trust_score = state
            .store
            .trust_score_for_item(&item)
            .map_err(internal_error)?;
        let (score, reasons) = working_item_priority(
            &item,
            entity.as_ref(),
            source_trust_score,
            source_trust_floor,
            now,
            query_lane.as_deref(),
        );
        ranked_items.push((score, reasons, item));
    }
    ranked_items.sort_by(|left, right| right.0.total_cmp(&left.0));

    // Cap status items to avoid noise flooding working memory.
    // Keep at most 2 status items; evict the rest with tracking.
    let max_status_items = 2usize;
    let mut status_count = 0usize;
    let mut capped_items = Vec::with_capacity(ranked_items.len());
    let mut status_evictions: Vec<WorkingMemoryEvictionRecord> = Vec::new();
    for entry in ranked_items {
        if entry.2.kind == memd_schema::MemoryKind::Status {
            status_count += 1;
            if status_count > max_status_items {
                let record = compact_record(&entry.2);
                status_evictions.push(WorkingMemoryEvictionRecord {
                    id: entry.2.id,
                    record,
                    reason: format!("evicted_by_status_cap;{}", entry.1.join(";")),
                });
                continue;
            }
        }
        capped_items.push(entry);
    }
    let ranked_items = capped_items;

    let selected_items = ranked_items
        .iter()
        .map(|(_, _, item)| item.clone())
        .collect::<Vec<_>>();

    let budget_chars = req.max_total_chars.unwrap_or(1600).clamp(400, 8000);
    let max_chars_per_item = req.max_chars_per_item.unwrap_or(220).clamp(80, 2000);
    let mut used_chars = 0usize;
    let mut truncated = false;
    let mut records = Vec::new();
    let mut evicted: Vec<WorkingMemoryEvictionRecord> = status_evictions;

    let compacted_records = ranked_items
        .iter()
        .map(|(_, reasons, item)| {
            let mut record = compact_record(item);
            if record.chars().count() > max_chars_per_item {
                record = record
                    .chars()
                    .take(max_chars_per_item.saturating_sub(3))
                    .collect();
                record.push_str("...");
            }
            (item.id, record, reasons.join(";"))
        })
        .collect::<Vec<_>>();

    // G2: lane diversity — track lane counts, defer items from overrepresented lanes.
    let lane_diversity_cap = 5usize; // max items from a single lane before deferral
    let mut lane_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut deferred: Vec<(usize, &str, &str)> = Vec::new(); // (index, record, reasons)

    for (index, (item_id, record, reasons)) in compacted_records.iter().enumerate() {
        // Check lane diversity: if this lane already has lane_diversity_cap items, defer it
        if let Some(item) = selected_items.iter().find(|i| i.id == *item_id) {
            if let Some(lane) = &item.lane {
                let count = lane_counts.get(lane.as_str()).copied().unwrap_or(0);
                if count >= lane_diversity_cap && records.len() < admission_limit {
                    deferred.push((index, record, reasons));
                    continue;
                }
            }
        }

        let record_chars = record.chars().count();
        if used_chars + record_chars > budget_chars {
            truncated = true;
            evicted.push(WorkingMemoryEvictionRecord {
                id: *item_id,
                record: record.clone(),
                reason: format!("evicted_by_budget;{reasons}"),
            });
            continue;
        }
        used_chars += record_chars;
        records.push(CompactMemoryRecord {
            id: *item_id,
            record: record.clone(),
        });

        // Track lane counts for admitted items
        if let Some(item) = selected_items.iter().find(|i| i.id == *item_id) {
            if let Some(lane) = &item.lane {
                *lane_counts.entry(lane.clone()).or_insert(0) += 1;
            }
        }

        if index + 1 >= admission_limit {
            truncated = compacted_records.len() > records.len();
        }
    }

    // Admit deferred items if there's still room
    for (index, record, reasons) in deferred {
        let (item_id, _, _) = &compacted_records[index];
        let record_chars = record.chars().count();
        if records.len() >= admission_limit || used_chars + record_chars > budget_chars {
            evicted.push(WorkingMemoryEvictionRecord {
                id: *item_id,
                record: record.to_string(),
                reason: format!("evicted_by_lane_diversity;{reasons}"),
            });
            truncated = true;
            continue;
        }
        used_chars += record_chars;
        records.push(CompactMemoryRecord {
            id: *item_id,
            record: record.to_string(),
        });
    }

    if records.len() > admission_limit {
        for record in records.drain(admission_limit..) {
            let reason = compacted_records
                .iter()
                .find(|(id, _, _)| *id == record.id)
                .map(|(_, _, reasons)| format!("evicted_by_admission_limit;{reasons}"))
                .unwrap_or_else(|| "evicted_by_admission_limit".to_string());
            evicted.push(WorkingMemoryEvictionRecord {
                id: record.id,
                record: record.record,
                reason,
            });
        }
        truncated = true;
    }

    let rehydration_queue = evicted
        .iter()
        .take(rehydration_limit)
        .map(|entry| {
            let source_item = selected_items.iter().find(|item| item.id == entry.id);
            build_rehydration_record(source_item, entry.id, &entry.record, &entry.reason)
        })
        .collect::<Vec<_>>();

    let traces = working_traces_for_items(state, &selected_items, 3).map_err(internal_error)?;
    let semantic_consolidation = if req.auto_consolidate.unwrap_or(false) {
        let auto_request = MemoryConsolidationRequest {
            project: req.project.clone(),
            namespace: req.agent.clone(),
            max_groups: Some(consolidation_policy.max_groups.min(8)),
            min_events: Some(consolidation_policy.min_events),
            lookback_days: Some(consolidation_policy.lookback_days),
            min_salience: Some(consolidation_policy.min_salience),
            record_events: Some(consolidation_policy.record_events),
        };
        Some(
            state
                .consolidate_semantic_memory(&auto_request)
                .map_err(internal_error)?,
        )
    } else {
        None
    };

    // Phase G: match procedures against current working context.
    let procedures = {
        let context: String = records
            .iter()
            .map(|r| r.record.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        if context.is_empty() {
            Vec::new()
        } else {
            state
                .store
                .match_procedures(&memd_schema::ProcedureMatchRequest {
                    context,
                    project: req.project.clone(),
                    namespace: req.agent.clone(),
                    limit: Some(3),
                })
                .map(|r| r.procedures)
                .unwrap_or_default()
        }
    };

    let compaction_quality = {
        use std::collections::BTreeMap;
        let kind_key = |kind: &MemoryKind| -> String {
            serde_json::to_value(kind)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| format!("{:?}", kind).to_lowercase())
        };
        let mut per_kind_admitted: BTreeMap<String, usize> = BTreeMap::new();
        for record in &records {
            if let Some(item) = selected_items.iter().find(|i| i.id == record.id) {
                *per_kind_admitted.entry(kind_key(&item.kind)).or_insert(0) += 1;
            }
        }
        let mut per_kind_evicted: BTreeMap<String, usize> = BTreeMap::new();
        for record in &evicted {
            if let Some(item) = selected_items.iter().find(|i| i.id == record.id) {
                *per_kind_evicted.entry(kind_key(&item.kind)).or_insert(0) += 1;
            }
        }
        CompactionQualityReport {
            admitted: records.len(),
            evicted: evicted.len(),
            per_kind_admitted,
            per_kind_evicted,
            budget_chars,
            used_chars,
        }
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
        procedures,
        compaction_quality: Some(compaction_quality),
    })
}

fn build_rehydration_record(
    item: Option<&memd_schema::MemoryItem>,
    id: Uuid,
    record: &str,
    reason: &str,
) -> MemoryRehydrationRecord {
    MemoryRehydrationRecord {
        id: Some(id),
        kind: "working_memory_record".to_string(),
        label: item
            .map(|item| format!("{:?}", item.kind).to_ascii_lowercase())
            .unwrap_or_else(|| "evicted working-set item".to_string()),
        summary: record.to_string(),
        reason: Some(reason.to_string()),
        source_agent: item.and_then(|item| item.source_agent.clone()),
        source_system: item.and_then(|item| item.source_system.clone()),
        source_path: item.and_then(|item| item.source_path.clone()),
        source_quality: item.and_then(|item| item.source_quality),
        recorded_at: item.map(|item| item.updated_at),
    }
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
            memory_kind: item.kind,
            memory_stage: item.stage,
            typed_memory: typed_memory_label(item.kind, item.stage),
            event_type: event.event_type.clone(),
            summary: event.summary.clone(),
            occurred_at: event.occurred_at,
            salience_score: event.salience_score,
        });
    }
    Ok(traces)
}

fn typed_memory_label(kind: MemoryKind, stage: MemoryStage) -> String {
    let family = match kind {
        MemoryKind::Runbook | MemoryKind::Procedural => "procedural",
        MemoryKind::Status => "session_continuity",
        MemoryKind::Pattern => "episodic",
        MemoryKind::Fact
        | MemoryKind::Decision
        | MemoryKind::Preference
        | MemoryKind::SelfModel
        | MemoryKind::Topology
        | MemoryKind::LiveTruth
        | MemoryKind::Constraint => "semantic",
    };
    let stage = match stage {
        MemoryStage::Candidate => "candidate",
        MemoryStage::Canonical => "canonical",
    };
    format!("{family}+{stage}")
}

fn working_item_priority(
    item: &memd_schema::MemoryItem,
    entity: Option<&MemoryEntityRecord>,
    source_trust_score: f32,
    source_trust_floor: f32,
    now: chrono::DateTime<Utc>,
    query_lane: Option<&str>,
) -> (f32, Vec<String>) {
    let confidence = item.confidence.clamp(0.0, 1.0);
    let age_days = now.signed_duration_since(item.updated_at).num_days().max(0) as f32;
    let verification_days = item
        .last_verified_at
        .map(|verified| now.signed_duration_since(verified).num_days().max(0) as f32)
        .unwrap_or(45.0);
    let recent_use_days = entity
        .and_then(|entity| entity.last_accessed_at)
        .map(|last_accessed_at| {
            now.signed_duration_since(last_accessed_at)
                .num_days()
                .max(0) as f32
        })
        .unwrap_or(45.0);
    let rehearsal_count = entity.map(|entity| entity.rehearsal_count).unwrap_or(0);

    // B2: kind is the dominant admission factor. Facts/decisions must always
    // outrank status regardless of confidence. Spread: 0.55 (Fact→Status).
    let kind_score = match item.kind {
        memd_schema::MemoryKind::Fact | memd_schema::MemoryKind::Decision => 0.30,
        memd_schema::MemoryKind::Preference => 0.25,
        memd_schema::MemoryKind::Procedural => 0.20,
        memd_schema::MemoryKind::Constraint | memd_schema::MemoryKind::Runbook => 0.15,
        memd_schema::MemoryKind::Pattern
        | memd_schema::MemoryKind::Topology
        | memd_schema::MemoryKind::SelfModel => 0.08,
        memd_schema::MemoryKind::Status => -0.25,
        _ => 0.0,
    };
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
        Some(ttl) if ttl <= 3_600 => -0.08,
        Some(ttl) if ttl <= 86_400 => -0.04,
        Some(ttl) if ttl >= 7 * 86_400 => 0.02,
        Some(_) => 0.0,
        None => 0.03,
    };
    let recent_use_score = if recent_use_days <= 2.0 {
        0.08
    } else if recent_use_days >= 30.0 {
        -0.06
    } else {
        0.0
    };
    let rehearsal_score = if rehearsal_count >= 5 {
        0.06
    } else if rehearsal_count == 0 {
        -0.04
    } else {
        0.02
    };
    let contradiction_score = match item.status {
        memd_schema::MemoryStatus::Contested | memd_schema::MemoryStatus::Superseded => -0.12,
        _ => {
            if item.source_quality == Some(memd_schema::SourceQuality::Synthetic) {
                -0.05
            } else {
                0.0
            }
        }
    };
    // G2.2: query-aware lane boost. Same-lane items rank above different-lane.
    // When no query context, preserve backward-compat flat +0.06.
    let lane_score = match (&item.lane, query_lane) {
        (Some(lane), Some(ql)) if lane == ql => 0.08, // same-lane match
        (Some(_), Some(_)) => 0.02,                   // has lane, different from query
        (Some(_), None) => 0.06,                      // has lane, no query context
        (None, _) => 0.0,
    };
    let correction_boost = if item.tags.iter().any(|t| t == "correction") {
        0.10
    } else {
        0.0
    };
    // Trust hierarchy tiebreaker: rank 0-4 × 0.012 = max 0.048.
    // Only matters when items score within ~0.05 of each other.
    let trust_hierarchy_score = trust_rank(item) as f32 * 0.012;
    let trust_score = if source_trust_score < source_trust_floor * 0.6 {
        -0.18
    } else if source_trust_score < source_trust_floor * 0.83 {
        -0.12
    } else if source_trust_score < source_trust_floor {
        -0.06
    } else if source_trust_score >= (source_trust_floor + 0.3).min(1.0) {
        0.08
    } else if source_trust_score >= (source_trust_floor + 0.15).min(1.0) {
        0.04
    } else {
        0.0
    };

    let mut reasons = vec![
        format!("kind={:?}", item.kind),
        format!("status={}", format_status(item.status)),
        format!("source={}", format_source_quality(item.source_quality)),
        format!("source_trust={source_trust_score:.2}"),
        format!("freshness_days={age_days:.0}"),
        format!("verified_days={verification_days:.0}"),
        format!("recent_use_days={recent_use_days:.0}"),
        format!("rehearsals={rehearsal_count}"),
    ];
    if source_trust_score < source_trust_floor {
        reasons.push("trust_below_floor".to_string());
    }
    if source_trust_score >= (source_trust_floor + 0.15).min(1.0) {
        reasons.push("trust_boost".to_string());
    }
    if contradiction_score < 0.0 {
        reasons.push("contradiction_state".to_string());
    }
    if item.status == memd_schema::MemoryStatus::Contested {
        reasons.push("contested".to_string());
    }
    if item.status == memd_schema::MemoryStatus::Superseded {
        reasons.push("superseded".to_string());
    }
    if item.source_quality == Some(memd_schema::SourceQuality::Canonical) {
        reasons.push("trusted_source".to_string());
    }
    if let Some(lane) = &item.lane {
        reasons.push(format!("lane={lane}"));
        match query_lane {
            Some(ql) if lane == ql => reasons.push("lane_match".to_string()),
            Some(_) => reasons.push("lane_mismatch".to_string()),
            None => {}
        }
    }
    if correction_boost > 0.0 {
        reasons.push("correction_boost".to_string());
    }
    if trust_hierarchy_score > 0.0 {
        reasons.push(format!("trust_rank={}", trust_rank(item)));
    }
    (
        // B2: reduced confidence weight (0.35 from 0.48) so kind_score dominates.
        (confidence * 0.35
            + kind_score
            + status_score
            + source_score
            + stage_score
            + freshness_score
            + verification_score
            + ttl_score
            + recent_use_score
            + rehearsal_score
            + trust_score
            + contradiction_score
            + lane_score
            + correction_boost
            + trust_hierarchy_score)
            .clamp(0.0, 1.0),
        reasons,
    )
}

fn format_status(status: memd_schema::MemoryStatus) -> &'static str {
    match status {
        memd_schema::MemoryStatus::Active => "active",
        memd_schema::MemoryStatus::Stale => "stale",
        memd_schema::MemoryStatus::Superseded => "superseded",
        memd_schema::MemoryStatus::Contested => "contested",
        memd_schema::MemoryStatus::Expired => "expired",
    }
}

fn format_source_quality(source_quality: Option<memd_schema::SourceQuality>) -> &'static str {
    match source_quality {
        Some(memd_schema::SourceQuality::Canonical) => "canonical",
        Some(memd_schema::SourceQuality::Derived) => "derived",
        Some(memd_schema::SourceQuality::Synthetic) => "synthetic",
        None => "unknown",
    }
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
            belief_branch: None,
            preferred: false,
            kind: memd_schema::MemoryKind::Fact,
            scope: memd_schema::MemoryScope::Project,
            project: Some("proj".to_string()),
            namespace: Some("ns".to_string()),
            workspace: Some("core".to_string()),
            visibility: memd_schema::MemoryVisibility::Workspace,
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
                    lane: None,
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

        assert!(
            working_item_priority(&good, None, 0.95, 0.6, now, None).0
                > working_item_priority(&weak, None, 0.22, 0.6, now, None).0
        );
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

        assert!(
            working_item_priority(&verified, None, 0.8, 0.6, now, None).0
                > working_item_priority(&unverified, None, 0.8, 0.6, now, None).0
        );
    }

    #[test]
    fn b2_fact_always_outranks_status_regardless_of_confidence() {
        let now = Utc::now();
        // High-confidence status item (0.9 — typical checkpoint)
        let mut status_item = sample_item(
            memd_schema::MemoryStatus::Active,
            Some(memd_schema::SourceQuality::Canonical),
            0.9,
            Some(now),
            now,
        );
        status_item.kind = memd_schema::MemoryKind::Status;

        // Medium-confidence fact (0.6 — freshly stored)
        let fact_item = sample_item(
            memd_schema::MemoryStatus::Active,
            Some(memd_schema::SourceQuality::Canonical),
            0.6,
            Some(now),
            now,
        );

        let status_score = working_item_priority(&status_item, None, 0.9, 0.6, now, None).0;
        let fact_score = working_item_priority(&fact_item, None, 0.7, 0.6, now, None).0;
        assert!(
            fact_score > status_score,
            "fact ({fact_score:.3}) must outrank status ({status_score:.3})"
        );
    }

    #[test]
    fn contested_synthetic_items_collect_policy_reasons() {
        let now = Utc::now();
        let item = sample_item(
            memd_schema::MemoryStatus::Contested,
            Some(memd_schema::SourceQuality::Synthetic),
            0.4,
            Some(now - chrono::Duration::days(80)),
            now - chrono::Duration::days(40),
        );

        let (_, reasons) = working_item_priority(&item, None, 0.28, 0.6, now, None);
        assert!(reasons.iter().any(|reason| reason == "contested"));
        assert!(reasons.iter().any(|reason| reason == "contradiction_state"));
        assert!(reasons.iter().any(|reason| reason == "trust_below_floor"));
        assert!(
            reasons
                .iter()
                .any(|reason| reason.starts_with("recent_use_days="))
        );
    }

    #[test]
    fn g2_2_same_lane_items_outrank_different_lane_with_query() {
        let now = Utc::now();
        // Use moderate scores so the total doesn't clamp to 1.0
        let mut same_lane = sample_item(
            memd_schema::MemoryStatus::Active,
            Some(memd_schema::SourceQuality::Derived),
            0.5,
            None,
            now - chrono::Duration::days(10),
        );
        same_lane.lane = Some("architecture".to_string());

        let mut diff_lane = sample_item(
            memd_schema::MemoryStatus::Active,
            Some(memd_schema::SourceQuality::Derived),
            0.5,
            None,
            now - chrono::Duration::days(10),
        );
        diff_lane.lane = Some("design".to_string());

        let query_lane = Some("architecture");

        let (same_score, same_reasons) =
            working_item_priority(&same_lane, None, 0.7, 0.6, now, query_lane);
        let (diff_score, diff_reasons) =
            working_item_priority(&diff_lane, None, 0.7, 0.6, now, query_lane);

        assert!(
            same_score > diff_score,
            "same-lane ({same_score:.3}) must outrank different-lane ({diff_score:.3})"
        );
        assert!(same_reasons.iter().any(|r| r == "lane_match"));
        assert!(diff_reasons.iter().any(|r| r == "lane_mismatch"));
    }

    #[test]
    fn g2_2_no_query_preserves_flat_lane_boost() {
        let now = Utc::now();
        let mut with_lane = sample_item(
            memd_schema::MemoryStatus::Active,
            Some(memd_schema::SourceQuality::Derived),
            0.5,
            None,
            now - chrono::Duration::days(10),
        );
        with_lane.lane = Some("architecture".to_string());

        let without_lane = sample_item(
            memd_schema::MemoryStatus::Active,
            Some(memd_schema::SourceQuality::Derived),
            0.5,
            None,
            now - chrono::Duration::days(10),
        );

        let (lane_score, _) =
            working_item_priority(&with_lane, None, 0.7, 0.6, now, None);
        let (no_lane_score, _) =
            working_item_priority(&without_lane, None, 0.7, 0.6, now, None);

        // With no query context, lane items still get the flat +0.06 boost
        assert!(
            lane_score > no_lane_score,
            "lane item ({lane_score:.3}) must outrank no-lane ({no_lane_score:.3}) without query"
        );
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
                route: memd_schema::RetrievalRoute::ProjectFirst,
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
            rehydration_limit: 3,
        },
        retrieval_feedback: MemoryPolicyFeedback {
            enabled: true,
            tracked_surfaces: vec![
                "search".to_string(),
                "context".to_string(),
                "compact_context".to_string(),
                "working".to_string(),
                "explain".to_string(),
                "timeline".to_string(),
            ],
            max_items_per_request: 3,
        },
        source_trust_floor: 0.6,
        runtime: memd_schema::MemoryPolicyRuntime {
            live_truth: memd_schema::MemoryPolicyLiveTruth {
                read_once_sources: true,
                raw_reopen_requires_change_or_doubt: true,
                visible_memory_objects: true,
                compile_from_events: true,
            },
            memory_compilation: memd_schema::MemoryPolicyMemoryCompilation {
                event_driven_updates: true,
                patch_not_rewrite: true,
                preserve_provenance: true,
                source_on_demand: true,
            },
            semantic_fallback: memd_schema::MemoryPolicySemanticFallback {
                enabled: true,
                source_of_truth: false,
                max_items_per_query: 3,
                rerank_with_visible_memory: true,
            },
            skill_gating: memd_schema::MemoryPolicySkillGating {
                propose_from_repeated_patterns: true,
                sandboxed_evaluation: true,
                auto_activate_low_risk_only: true,
                gated_activation: true,
                require_evaluation: true,
                require_policy_approval: true,
            },
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
            decay_divisor: 14.0,
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
