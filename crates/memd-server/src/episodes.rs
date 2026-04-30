//! E3-D2: episode detection + consolidation.
//!
//! Pure-function core (session boundary detection, narrative synthesis,
//! deterministic session id) kept separate from store I/O so it can be
//! unit-tested without sqlite. Store methods in `store_episodes.rs` drive
//! this module.
//!
//! Idempotency contract: `session_id_for(project, namespace, started_at)` is
//! deterministic (UUIDv5). Re-running consolidation for the same window
//! produces the same session_ids; `episodes(session_id UNIQUE)` rejects the
//! re-insert and the response reports it as `idempotent_skipped`.

use chrono::{DateTime, Utc};
use memd_schema::{Episode, EpisodeFactRelation, MemoryItem, SessionSpan};
use uuid::Uuid;

/// UUID namespace for episode session ids — fixed seed so the mapping is
/// stable across processes and restarts.
const SESSION_NS: Uuid = Uuid::from_bytes([
    0xe3, 0xd2, 0x53, 0x45, 0x53, 0x53, 0x49, 0x4f, 0x4e, 0x5f, 0x65, 0x70, 0x69, 0x73, 0x6f, 0x64,
]);

pub const DEFAULT_SESSION_GAP_SECONDS: u64 = 1800;

#[derive(Debug, Clone)]
pub struct EventPoint {
    pub memory_id: Uuid,
    pub at: DateTime<Utc>,
}

/// Group a chronologically sorted event stream into sessions, splitting on
/// idle gaps greater than `gap_seconds`. Returns spans in the same order as
/// input.
pub fn detect_sessions(
    events: &[EventPoint],
    gap_seconds: u64,
    project: Option<&str>,
    namespace: Option<&str>,
) -> Vec<SessionSpan> {
    if events.is_empty() {
        return Vec::new();
    }

    let gap = chrono::Duration::seconds(gap_seconds as i64);
    let mut spans: Vec<SessionSpan> = Vec::new();
    let mut cur_start = events[0].at;
    let mut cur_end = events[0].at;
    let mut cur_ids: Vec<Uuid> = vec![events[0].memory_id];

    for win in events.windows(2) {
        let prev = &win[0];
        let next = &win[1];
        if next.at - prev.at > gap {
            spans.push(build_span(
                project,
                namespace,
                cur_start,
                cur_end,
                std::mem::take(&mut cur_ids),
            ));
            cur_start = next.at;
        }
        cur_end = next.at;
        cur_ids.push(next.memory_id);
    }
    spans.push(build_span(project, namespace, cur_start, cur_end, cur_ids));
    spans
}

fn build_span(
    project: Option<&str>,
    namespace: Option<&str>,
    started_at: DateTime<Utc>,
    ended_at: DateTime<Utc>,
    memory_ids: Vec<Uuid>,
) -> SessionSpan {
    SessionSpan {
        id: session_id_for(project, namespace, started_at),
        project: project.map(str::to_string),
        namespace: namespace.map(str::to_string),
        started_at,
        ended_at,
        event_count: memory_ids.len(),
        memory_ids,
    }
}

/// Deterministic UUIDv5 keyed on (project, namespace, started_at RFC3339
/// rounded to second). Re-detecting the same session window always produces
/// the same id, which is what drives idempotency at the DB layer.
pub fn session_id_for(
    project: Option<&str>,
    namespace: Option<&str>,
    started_at: DateTime<Utc>,
) -> Uuid {
    let key = format!(
        "{}|{}|{}",
        project.unwrap_or(""),
        namespace.unwrap_or(""),
        started_at.format("%Y-%m-%dT%H:%M:%SZ"),
    );
    Uuid::new_v5(&SESSION_NS, key.as_bytes())
}

/// Classify a memory item's relation to its containing episode. Kept
/// deterministic — a future LLM pass can upgrade this, but the default keeps
/// consolidation cheap.
pub fn classify_relation(item: &MemoryItem) -> EpisodeFactRelation {
    use memd_schema::MemoryKind::*;
    match item.kind {
        Decision => EpisodeFactRelation::Outcome,
        Fact | LiveTruth | Status | Topology => EpisodeFactRelation::Evidence,
        Runbook | Procedural | Pattern | Constraint | Preference | SelfModel => {
            EpisodeFactRelation::Reference
        }
        Correction => EpisodeFactRelation::Outcome,
        Skill => EpisodeFactRelation::Reference, // Phase 1: stub classification
    }
}

/// Synthesize (title, narrative) from a session's items. Deterministic —
/// same inputs produce byte-identical output. Title = first non-empty
/// content line trimmed to 80 chars, or a dated fallback. Narrative = bullet
/// list of item summaries, oldest first.
pub fn synthesize_narrative(items: &[MemoryItem], started_at: DateTime<Utc>) -> (String, String) {
    let title = items
        .iter()
        .find_map(|it| it.content.lines().find(|l| !l.trim().is_empty()))
        .map(|first| {
            let trimmed: String = first.trim().chars().take(80).collect();
            if trimmed.is_empty() {
                fallback_title(started_at)
            } else {
                trimmed
            }
        })
        .unwrap_or_else(|| fallback_title(started_at));

    let mut lines = Vec::with_capacity(items.len() + 2);
    lines.push(format!(
        "Session {} — {} items",
        started_at.format("%Y-%m-%d %H:%M UTC"),
        items.len()
    ));
    lines.push(String::new());
    for it in items {
        let first_line = it.content.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
        let summary: String = first_line.trim().chars().take(160).collect();
        if summary.is_empty() {
            continue;
        }
        lines.push(format!(
            "- [{}] {}",
            kind_tag(it.kind),
            summary,
        ));
    }
    let narrative = lines.join("\n");
    (title, narrative)
}

fn fallback_title(started_at: DateTime<Utc>) -> String {
    format!("Session {}", started_at.format("%Y-%m-%d %H:%M UTC"))
}

fn kind_tag(kind: memd_schema::MemoryKind) -> &'static str {
    use memd_schema::MemoryKind::*;
    match kind {
        Fact => "fact",
        Decision => "decision",
        Preference => "pref",
        Runbook => "runbook",
        Procedural => "proc",
        SelfModel => "self",
        Topology => "topology",
        Status => "status",
        LiveTruth => "live",
        Pattern => "pattern",
        Constraint => "constraint",
        Correction => "correction",
        Skill => "skill", // Phase 1: stub tag
    }
}

/// Build an Episode record from a session span + its items.
pub fn build_episode(
    span: &SessionSpan,
    items: &[MemoryItem],
    now: DateTime<Utc>,
) -> Episode {
    let (title, narrative) = synthesize_narrative(items, span.started_at);
    Episode {
        id: Uuid::new_v4(),
        mind: None,
        title,
        narrative,
        session_id: span.id,
        project: span.project.clone(),
        namespace: span.namespace.clone(),
        started_at: span.started_at,
        ended_at: span.ended_at,
        fact_count: items.len(),
        created_at: now,
        updated_at: now,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use memd_schema::{MemoryKind, MemoryScope, MemoryStage, MemoryStatus, MemoryVisibility};

    fn ev(mins: i64, id: &str) -> EventPoint {
        let at = Utc.with_ymd_and_hms(2026, 4, 20, 10, 0, 0).unwrap()
            + chrono::Duration::minutes(mins);
        EventPoint {
            memory_id: Uuid::parse_str(id).unwrap(),
            at,
        }
    }

    #[test]
    fn detect_sessions_empty_returns_empty() {
        let spans = detect_sessions(&[], 1800, None, None);
        assert!(spans.is_empty());
    }

    #[test]
    fn detect_sessions_single_event_yields_single_span() {
        let events = vec![ev(0, "00000000-0000-0000-0000-000000000001")];
        let spans = detect_sessions(&events, 1800, Some("memd"), Some("main"));
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].event_count, 1);
        assert_eq!(spans[0].project.as_deref(), Some("memd"));
    }

    #[test]
    fn detect_sessions_splits_on_gap_exceeding_threshold() {
        let events = vec![
            ev(0, "00000000-0000-0000-0000-000000000001"),
            ev(10, "00000000-0000-0000-0000-000000000002"),
            // 40min gap — > 30min threshold
            ev(50, "00000000-0000-0000-0000-000000000003"),
            ev(55, "00000000-0000-0000-0000-000000000004"),
        ];
        let spans = detect_sessions(&events, 1800, None, None);
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].event_count, 2);
        assert_eq!(spans[1].event_count, 2);
    }

    #[test]
    fn detect_sessions_keeps_close_events_in_one_span() {
        let events = vec![
            ev(0, "00000000-0000-0000-0000-000000000001"),
            ev(5, "00000000-0000-0000-0000-000000000002"),
            ev(10, "00000000-0000-0000-0000-000000000003"),
            ev(25, "00000000-0000-0000-0000-000000000004"),
        ];
        let spans = detect_sessions(&events, 1800, None, None);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].event_count, 4);
    }

    #[test]
    fn session_id_is_deterministic() {
        let t = Utc.with_ymd_and_hms(2026, 4, 20, 10, 0, 0).unwrap();
        let a = session_id_for(Some("memd"), Some("main"), t);
        let b = session_id_for(Some("memd"), Some("main"), t);
        assert_eq!(a, b);
    }

    #[test]
    fn session_id_differs_on_different_windows() {
        let t1 = Utc.with_ymd_and_hms(2026, 4, 20, 10, 0, 0).unwrap();
        let t2 = Utc.with_ymd_and_hms(2026, 4, 20, 11, 0, 0).unwrap();
        assert_ne!(
            session_id_for(Some("memd"), Some("main"), t1),
            session_id_for(Some("memd"), Some("main"), t2)
        );
    }

    fn mem(content: &str, kind: MemoryKind) -> MemoryItem {
        MemoryItem {
            id: Uuid::new_v4(),
            content: content.to_string(),
            redundancy_key: None,
            belief_branch: None,
            preferred: false,
            kind,
            scope: MemoryScope::Project,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
            visibility: MemoryVisibility::default(),
            source_agent: None,
            source_system: None,
            source_path: None,
            source_quality: None,
            confidence: 0.8,
            ttl_seconds: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_verified_at: None,
            supersedes: Vec::new(),
            tags: Vec::new(),
            status: MemoryStatus::Active,
            stage: MemoryStage::Canonical,
            lane: None,
            version: 1,
            correction_meta: None,
        }
    }

    #[test]
    fn narrative_title_is_first_nonempty_line() {
        let items = vec![mem("first line\ndetails", MemoryKind::Fact)];
        let (title, _) = synthesize_narrative(&items, Utc::now());
        assert_eq!(title, "first line");
    }

    #[test]
    fn narrative_is_deterministic() {
        let a_items = vec![
            mem("a", MemoryKind::Fact),
            mem("b", MemoryKind::Decision),
        ];
        let b_items = a_items.clone();
        let t = Utc.with_ymd_and_hms(2026, 4, 20, 10, 0, 0).unwrap();
        let (t1, n1) = synthesize_narrative(&a_items, t);
        let (t2, n2) = synthesize_narrative(&b_items, t);
        assert_eq!(t1, t2);
        assert_eq!(n1, n2);
    }

    #[test]
    fn narrative_includes_all_nonempty_items() {
        let items = vec![
            mem("alpha content", MemoryKind::Fact),
            mem("beta content", MemoryKind::Decision),
            mem("", MemoryKind::Fact), // dropped
        ];
        let (_, narrative) = synthesize_narrative(&items, Utc::now());
        assert!(narrative.contains("alpha content"));
        assert!(narrative.contains("beta content"));
        assert!(narrative.contains("[fact]"));
        assert!(narrative.contains("[decision]"));
    }

    #[test]
    fn classify_relation_maps_kinds() {
        assert_eq!(
            classify_relation(&mem("x", MemoryKind::Decision)),
            EpisodeFactRelation::Outcome
        );
        assert_eq!(
            classify_relation(&mem("x", MemoryKind::Fact)),
            EpisodeFactRelation::Evidence
        );
        assert_eq!(
            classify_relation(&mem("x", MemoryKind::Runbook)),
            EpisodeFactRelation::Reference
        );
    }
}
