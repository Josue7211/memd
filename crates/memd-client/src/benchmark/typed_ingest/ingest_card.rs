//! V6 typed-ingest card + baseline lock — task A6.8.
//!
//! Locks the deterministic turn/session counts produced by the episodic
//! adapter on the canonical A6 fixtures. A regression here means a bench
//! adapter changed shape: either a real bug, or an intentional schema
//! shift that needs a baseline bump in the same PR. The card is the
//! human-readable summary appended to ingest logs and (in A6.9) to the
//! nightly substrate report.

use super::TypedIngestReport;

/// Locked baseline for the LME 10-turn fixture
/// (`tests/fixtures/typed_ingest/a6/lme-sample-10turn.json`). Drift means
/// the LME adapter changed; bump deliberately.
pub(crate) const BASELINE_LME_10TURN: TypedIngestReport = TypedIngestReport {
    bench_id: "longmemeval",
    turn_count: 10,
    session_count: 2,
};

/// Comparator. Returns `Ok(())` if `actual` matches the locked baseline
/// exactly. Episodic counts are deterministic for a given fixture, so any
/// difference is a regression — the loose ±1% from the plan applies to
/// downstream retrieval scores, not to ingest counts.
pub(crate) fn assert_baseline(actual: &TypedIngestReport, baseline: &TypedIngestReport) -> Result<(), String> {
    if actual.bench_id != baseline.bench_id {
        return Err(format!(
            "bench_id drift: baseline `{}` got `{}`",
            baseline.bench_id, actual.bench_id
        ));
    }
    if actual.turn_count != baseline.turn_count {
        return Err(format!(
            "turn_count drift on `{}`: baseline {} got {}",
            baseline.bench_id, baseline.turn_count, actual.turn_count
        ));
    }
    if actual.session_count != baseline.session_count {
        return Err(format!(
            "session_count drift on `{}`: baseline {} got {}",
            baseline.bench_id, baseline.session_count, actual.session_count
        ));
    }
    Ok(())
}

/// Render a typed-ingest card as markdown. Stable shape — A6.9 nightly
/// gate parses the `bench_id` / `turn_count` / `session_count` lines.
pub(crate) fn render_ingest_card(report: &TypedIngestReport) -> String {
    let mut s = String::new();
    s.push_str("## Typed Ingest Card\n\n");
    s.push_str(&format!("- bench_id: `{}`\n", report.bench_id));
    s.push_str(&format!("- turn_count: {}\n", report.turn_count));
    s.push_str(&format!("- session_count: {}\n", report.session_count));
    s.push_str("- pipeline: episodic\n");
    s.push_str("- schema: `MemoryKind::Fact` + `EpisodicProvenance` sidecar\n");
    s
}
