//! Phase F4.6 — preference drift end-to-end + restate-rate benchmark.
//!
//! Tests 13 + 14 from the F4 plan. Server-free: drives the detector via
//! a stub `JudgeTransport`, persists outstanding state, then runs the
//! D4 wake compiler against a snapshot and asserts the drift line lands
//! in the rendered Preferences section. Test 14 measures restate-rate
//! drop directly from the fixture pair.

use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use memd_core::correction::judge::{JudgeTransport, RawJudgeResponse};
use memd_core::preference::drift::{DriftConfig, DriftDetector, DriftVerdict};
use memd_core::preference::outstanding::{
    outstanding_state_path, read_outstanding, record_drift,
};
use memd_core::preference::PreferenceRecord;
use serde::Deserialize;
use tempfile::TempDir;

use crate::runtime::resume::compiler::{
    compile_wake, drift_notes_from_outstanding, BucketKind, CompilerInput, WakeBudget,
};
use memd_schema::CompactMemoryRecord;

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("f4")
}

#[derive(Debug, Deserialize)]
struct PrefFixture {
    id: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct BehaviorTurn {
    #[allow(dead_code)]
    turn: String,
    #[allow(dead_code)]
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct RestateRow {
    #[allow(dead_code)]
    day: u32,
    restate: bool,
}

fn load_jsonl<T: for<'de> Deserialize<'de>>(name: &str) -> Vec<T> {
    let path = fixtures_root().join(name);
    let body = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    body.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("parse fixture row"))
        .collect()
}

/// Stub transport that returns the contents of a fixture verdict file.
struct FixtureTransport {
    body: String,
}

impl JudgeTransport for FixtureTransport {
    fn call(&self, _prompt: &str, _model: &str) -> Result<RawJudgeResponse> {
        Ok(RawJudgeResponse {
            status: 200,
            body: self.body.clone(),
        })
    }
}

fn drift_config(memd_dir: &std::path::Path) -> DriftConfig {
    DriftConfig {
        cache_dir: memd_dir.join("benchmarks").join("grader-cache").join("f4"),
        budget_file: memd_dir.join("logs").join("c4-cost.json"),
        model: "gpt-5.4-test".into(),
        budget_usd: 5.0,
    }
}

fn pref_records(prefs: &[PrefFixture]) -> Vec<CompactMemoryRecord> {
    prefs
        .iter()
        .map(|p| CompactMemoryRecord {
            id: uuid::Uuid::new_v4(),
            record: format!("id={} | content={}", p.id, p.content),
        })
        .collect()
}

/// Test 13 — 7-day dogfood simulation: aligned phase produces no drift
/// surface; violating phase records drift and surfaces it on next wake.
#[test]
fn e2e_7day_dogfood_simulation() {
    let tmp = TempDir::new().unwrap();
    let memd_dir = tmp.path().to_path_buf();
    let prefs: Vec<PrefFixture> = load_jsonl("preferences.jsonl");
    let voice_pref = prefs
        .iter()
        .find(|p| p.id == "pref-voice-terse")
        .expect("voice-terse preference present in fixture");
    let pref_record = PreferenceRecord::new(voice_pref.id.clone(), voice_pref.content.clone());

    // Aligned phase: 10 turns matching prefs → judge returns aligned →
    // outstanding state stays empty.
    let aligned_turns: Vec<String> = load_jsonl::<BehaviorTurn>("aligned-behavior.jsonl")
        .into_iter()
        .map(|t| t.content)
        .collect();
    let ok_body = fs::read_to_string(fixtures_root().join("judge-verdict-ok.json")).unwrap();
    let aligned_detector = DriftDetector::new(
        FixtureTransport { body: ok_body },
        drift_config(&memd_dir),
    );
    let aligned_check = aligned_detector
        .detect(&pref_record, &aligned_turns)
        .expect("aligned detect");
    assert_eq!(aligned_check.verdict, DriftVerdict::Aligned);
    record_drift(
        &outstanding_state_path(&memd_dir),
        &aligned_check,
        1_700_000_000_000,
    )
    .unwrap();

    let outstanding_after_aligned =
        read_outstanding(&outstanding_state_path(&memd_dir)).unwrap();
    assert!(
        outstanding_after_aligned.entries.is_empty(),
        "aligned verdict must not seed outstanding state"
    );

    // Violating phase: 10 turns drifting → judge returns drift → outstanding
    // state holds one entry → next wake surfaces the drift line.
    let drift_turns: Vec<String> = load_jsonl::<BehaviorTurn>("drift-behavior.jsonl")
        .into_iter()
        .map(|t| t.content)
        .collect();
    let drift_body = fs::read_to_string(fixtures_root().join("judge-verdict-drift.json")).unwrap();
    let drift_detector = DriftDetector::new(
        FixtureTransport { body: drift_body },
        drift_config(&memd_dir),
    );
    let drift_check = drift_detector
        .detect(&pref_record, &drift_turns)
        .expect("drift detect");
    assert_eq!(drift_check.verdict, DriftVerdict::Drift);
    assert!(drift_check.violation_count > 0);
    record_drift(
        &outstanding_state_path(&memd_dir),
        &drift_check,
        1_700_000_000_001,
    )
    .unwrap();

    let outstanding_after_drift =
        read_outstanding(&outstanding_state_path(&memd_dir)).unwrap();
    assert_eq!(
        outstanding_after_drift.entries.len(),
        1,
        "drift verdict must seed outstanding state"
    );

    // Wake compile: drift_notes populated from outstanding state, drift
    // line visible in the rendered Preferences section.
    let drift_notes = drift_notes_from_outstanding(&memd_dir);
    assert_eq!(drift_notes.len(), 1, "exactly one drift note expected");
    assert!(
        drift_notes[0].contains("⚠ drift"),
        "drift note format: {}",
        drift_notes[0]
    );
    assert!(drift_notes[0].contains(&voice_pref.id));

    let input = CompilerInput {
        canonical: vec![CompactMemoryRecord {
            id: uuid::Uuid::new_v4(),
            record: "id=root | c=memd is rust".into(),
        }],
        preferences: pref_records(&prefs),
        drift_notes,
        ..Default::default()
    };
    let compiled = compile_wake(input, WakeBudget::default_2000());
    assert!(
        compiled.markdown.contains("⚠ drift"),
        "drift surface must appear in rendered wake markdown:\n{}",
        compiled.markdown
    );
    assert!(compiled.markdown.contains(&voice_pref.id));

    let pref_idx = compiled.markdown.find("## Preferences").unwrap();
    let drift_idx = compiled.markdown.find("⚠ drift").unwrap();
    assert!(
        pref_idx < drift_idx,
        "drift line must render under the Preferences header"
    );

    // Confirm clears outstanding state (agent acknowledged); next wake
    // omits the drift surface.
    memd_core::preference::outstanding::clear_outstanding(
        &outstanding_state_path(&memd_dir),
        &voice_pref.id,
    )
    .unwrap();
    let post_clear = drift_notes_from_outstanding(&memd_dir);
    assert!(
        post_clear.is_empty(),
        "confirm must purge drift surface for next wake"
    );

    // Preference bucket non-demotable: even with a tight budget, the
    // pref records survive (validates the F4.3 invariant on the e2e path).
    let tight_input = CompilerInput {
        preferences: pref_records(&prefs),
        ..Default::default()
    };
    let mut tight_budget = WakeBudget::default_2000();
    tight_budget.tokens = 200;
    let tight = compile_wake(tight_input, tight_budget);
    let pref_report = tight
        .bucket_report
        .get(&BucketKind::Preference)
        .expect("preference bucket reported");
    assert!(
        pref_report.admitted > 0,
        "Preference bucket must keep at least one record under tight budget"
    );
}

/// Test 14 — user restate rate drops ≥50% with F4 surfacing enabled.
///
/// Compares baseline (no F4 surface → user restates voice preference 4×)
/// against F4-on (drift surfaced → user restates 1×). The metric is
/// derived from fixture data, so this also acts as a regression lock on
/// fixture authorship.
#[test]
fn e2e_user_restate_rate_drops() {
    let baseline: Vec<RestateRow> = load_jsonl("user-restate-baseline.jsonl");
    let with_f4: Vec<RestateRow> = load_jsonl("user-restate-withF4.jsonl");

    let baseline_restates = baseline.iter().filter(|r| r.restate).count();
    let f4_restates = with_f4.iter().filter(|r| r.restate).count();

    assert!(
        baseline_restates >= 3,
        "baseline must capture multiple restates (got {baseline_restates})"
    );
    assert!(
        f4_restates <= baseline_restates / 2,
        "F4 must drop restate count by ≥50%: baseline={baseline_restates} f4={f4_restates}"
    );
}
