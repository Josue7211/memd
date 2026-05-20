use super::*;
use memd_schema::CompactionQualityReport;
use std::collections::BTreeMap;

fn report(
    facts: usize,
    decisions: usize,
    extras: usize,
    used_chars: usize,
) -> CompactionQualityReport {
    let mut per_kind_admitted = BTreeMap::new();
    if facts > 0 {
        per_kind_admitted.insert("\"fact\"".to_string(), facts);
    }
    if decisions > 0 {
        per_kind_admitted.insert("\"decision\"".to_string(), decisions);
    }
    if extras > 0 {
        per_kind_admitted.insert("\"status\"".to_string(), extras);
    }
    CompactionQualityReport {
        admitted: facts + decisions + extras,
        evicted: 0,
        per_kind_admitted,
        per_kind_evicted: BTreeMap::new(),
        chars_per_kind_admitted: BTreeMap::new(),
        budget_chars: 4000,
        used_chars,
    }
}

#[test]
fn rich_handoff_meets_08_threshold() {
    let score = HandoffQualityScore::from_report(&report(3, 2, 3, 3000));
    assert!(
        score.is_acceptable(),
        "rich handoff should pass: composite={}",
        score.composite
    );
    assert!(score.fact_coverage >= 0.99);
    assert!(score.decision_coverage >= 0.99);
}

#[test]
fn sparse_handoff_fails_08_threshold() {
    let score = HandoffQualityScore::from_report(&report(1, 0, 0, 200));
    assert!(
        !score.is_acceptable(),
        "sparse handoff should fail: composite={}",
        score.composite
    );
    assert!(score.fact_coverage < 0.5);
    assert_eq!(score.decision_coverage, 0.0);
}

#[test]
fn preference_retrieval_decisions_raise_handoff_quality() {
    let mut score = HandoffQualityScore::from_report(&report(3, 0, 4, 3000));
    assert_eq!(score.decision_coverage, 0.0);
    assert!(!score.is_acceptable());

    score.include_decision_signals(count_decision_records(&[
        "id=a | kind=decision | c=first".to_string(),
        "decision: second".to_string(),
    ]));

    assert!(score.decision_coverage >= 0.99);
    assert!(score.is_acceptable());
}

#[test]
fn recoverable_rehydration_decisions_raise_handoff_quality() {
    let mut snapshot = ResumeSnapshot::empty();
    snapshot
        .working
        .rehydration_queue
        .push(memd_schema::MemoryRehydrationRecord {
            id: None,
            kind: "working_memory_record".to_string(),
            label: "decision".to_string(),
            summary: "id=handoff | kind=decision | c=fix native recovery next".to_string(),
            reason: Some("evicted_by_budget".to_string()),
            source_agent: None,
            source_system: None,
            source_path: None,
            source_quality: None,
            recorded_at: None,
        });
    let preferences = Vec::new();
    let decision_count =
        count_recoverable_decision_records(&snapshot.context, &snapshot.working, &preferences);

    let mut score = HandoffQualityScore::from_report(&report(3, 0, 4, 3000));
    score.include_decision_signals(decision_count);

    assert_eq!(decision_count, 1);
    assert!(score.decision_coverage >= 0.49);
    assert!(score.is_acceptable());
}

#[test]
fn recoverable_fact_and_decision_signals_raise_handoff_quality() {
    let mut snapshot = ResumeSnapshot::empty();
    snapshot
        .working
        .records
        .push(memd_schema::CompactMemoryRecord {
            id: uuid::Uuid::new_v4(),
            record: "id=status-a | kind=status | c=Fact: memd tree clean".to_string(),
        });
    snapshot
        .working
        .records
        .push(memd_schema::CompactMemoryRecord {
            id: uuid::Uuid::new_v4(),
            record: "id=status-b | kind=status | c=Decision: commit producer bridge".to_string(),
        });
    snapshot
        .working
        .rehydration_queue
        .push(memd_schema::MemoryRehydrationRecord {
            id: None,
            kind: "working_memory_record".to_string(),
            label: "fact".to_string(),
            summary: "Fact: installed memd supports live-state".to_string(),
            reason: Some("evicted_by_status_cap".to_string()),
            source_agent: None,
            source_system: None,
            source_path: None,
            source_quality: None,
            recorded_at: None,
        });
    let preferences = vec![
        "Fact: proof blockers remain external".to_string(),
        "Decision: memd live-state is the authority path".to_string(),
        "Procedure: verify clean tree before completion".to_string(),
    ];
    let signals = recoverable_signal_counts(&snapshot.context, &snapshot.working, &preferences);
    let mut score = HandoffQualityScore::from_report(&report(0, 0, 2, 1100));

    score.include_recoverable_signals(signals.facts, signals.decisions, signals.total);

    assert_eq!(signals.facts, 3);
    assert_eq!(signals.decisions, 2);
    assert_eq!(signals.total, 6);
    assert!(score.fact_coverage >= 0.99);
    assert!(score.decision_coverage >= 0.99);
    assert!(score.working_depth >= 0.74);
    assert!(score.is_acceptable());
}

#[test]
fn continuity_status_records_raise_handoff_quality() {
    let mut snapshot = ResumeSnapshot::empty();
    for record in [
        "id=handoff | kind=status | c=CURRENT NEXT ACTION after commit 98fb520: continue live-state authority work",
        "id=proof | kind=status | c=proof_blockers=supermemory:missing_requirements=approved_supermemory_access_route_or_process_credential",
        "id=live | kind=status | c=live_state_blockers=clawcontrol:status=auth_required missing=calendar",
        "id=tree | kind=status | c=repo tree clean after hook commit",
    ] {
        snapshot
            .working
            .records
            .push(memd_schema::CompactMemoryRecord {
                id: uuid::Uuid::new_v4(),
                record: record.to_string(),
            });
    }
    let preferences = vec![
        "Decision: keep ClawControl access route explicit".to_string(),
        "Fact: source probe refreshed auth_required state".to_string(),
    ];
    let signals = recoverable_signal_counts(&snapshot.context, &snapshot.working, &preferences);
    let mut score = HandoffQualityScore::from_report(&report(0, 0, 2, 1100));

    score.include_recoverable_signals(signals.facts, signals.decisions, signals.total);

    assert!(signals.facts >= 3, "{signals:?}");
    assert!(signals.decisions >= 2, "{signals:?}");
    assert!(signals.total >= 6, "{signals:?}");
    assert!(score.fact_coverage >= 0.99);
    assert!(score.decision_coverage >= 0.99);
    assert!(score.is_acceptable());
}

#[test]
fn bundle_refresh_status_records_raise_handoff_quality() {
    let mut snapshot = ResumeSnapshot::empty();
    for record in [
        "id=resume | kind=status | tags=resume_state,session_state | c=compact session continuity state",
        "id=wake-a | kind=status | tags=checkpoint,current-task,auto-short-term,bundle-refresh,resume | c=status: wake project=memd working=2",
        "id=wake-b | kind=status | tags=checkpoint,current-task,auto-short-term,bundle-refresh,refresh | c=status: wake project=memd focus=current task",
        "id=wake-c | kind=status | tags=checkpoint,current-task,auto-short-term,bundle-refresh,resume | c=status: wake project=memd repo clean",
    ] {
        snapshot
            .working
            .records
            .push(memd_schema::CompactMemoryRecord {
                id: uuid::Uuid::new_v4(),
                record: record.to_string(),
            });
    }
    let preferences =
        vec!["id=next | kind=decision | c=CURRENT NEXT ACTION after commit 98fb520".to_string()];
    let signals = recoverable_signal_counts(&snapshot.context, &snapshot.working, &preferences);
    let mut score = HandoffQualityScore::from_report(&report(0, 0, 2, 1100));

    score.include_recoverable_signals(signals.facts, signals.decisions, signals.total);

    assert!(signals.facts >= 3, "{signals:?}");
    assert!(signals.decisions >= 2, "{signals:?}");
    assert!(signals.total >= 5, "{signals:?}");
    assert!(score.is_acceptable());
}

#[test]
fn cached_resume_refresh_restores_handoff_quality_from_working_report() {
    let mut snapshot = ResumeSnapshot::empty();
    snapshot.working.compaction_quality = Some(report(3, 2, 3, 900));
    snapshot
        .working
        .records
        .push(memd_schema::CompactMemoryRecord {
            id: uuid::Uuid::new_v4(),
            record:
                "id=handoff | kind=status | c=CURRENT NEXT ACTION: continue live-state authority"
                    .to_string(),
        });
    snapshot
        .preferences
        .push("Decision: keep approved communications privacy gates explicit".to_string());

    refresh_resume_local_recovery_state(None, &mut snapshot);

    let score = snapshot.handoff_quality.expect("handoff quality restored");
    assert!(score.is_acceptable(), "score={score:?}");
}

#[test]
fn truncated_handoff_penalizes_trust_score() {
    // Utilization near 1.0 → trust proxy below 1.0.
    let score = HandoffQualityScore::from_report(&report(3, 2, 2, 4000));
    assert!(score.budget_utilization >= 0.99);
    // composite still can pass if coverage is maxed
    assert!(score.is_acceptable());
}

#[test]
fn unknown_kind_spellings_still_match() {
    // upstream sometimes emits bare keys (no JSON quotes).
    let mut per_kind_admitted = BTreeMap::new();
    per_kind_admitted.insert("fact".to_string(), 3);
    per_kind_admitted.insert("decision".to_string(), 2);
    let report = CompactionQualityReport {
        admitted: 5,
        evicted: 0,
        per_kind_admitted,
        per_kind_evicted: BTreeMap::new(),
        chars_per_kind_admitted: BTreeMap::new(),
        budget_chars: 4000,
        used_chars: 2000,
    };
    let score = HandoffQualityScore::from_report(&report);
    assert!(score.fact_coverage >= 0.99);
    assert!(score.decision_coverage >= 0.99);
    assert!(score.is_acceptable());
}
