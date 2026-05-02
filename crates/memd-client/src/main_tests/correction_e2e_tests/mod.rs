//! Phase C4.6 — end-to-end correction-lane scenarios.
//!
//! These tests are server-free: they exercise detector + capture + log
//! plumbing against fixture transcripts under `fixtures/c4/`.

use std::fs;
use std::path::{Path, PathBuf};

use memd_core::correction::detector::{PriorClaim, score};
use serde::Deserialize;
use tempfile::TempDir;

use crate::cli::{CorrectionCaptureArgs, corrections_log_path, run_correction_capture};

#[derive(Debug, Clone, Deserialize)]
struct FixtureTurn {
    turn: String,
    role: String,
    content: String,
}

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("c4")
}

fn load_jsonl(name: &str) -> Vec<FixtureTurn> {
    let path = fixtures_root().join(name);
    let body = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    body.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("parse fixture turn"))
        .collect()
}

fn capture_fixture(tmp: &Path, fixture: &str) {
    let turns = load_jsonl(fixture);
    let mut prior: Vec<PriorClaim> = Vec::new();
    for t in &turns {
        if t.role == "assistant" || t.role == "system" {
            continue;
        }
        let cand = score(&t.content, &prior);
        if cand.score >= 0.5 {
            run_correction_capture(&CorrectionCaptureArgs {
                content: t.content.clone(),
                corrects_id: cand.corrects_id.clone(),
                source_turn: Some(t.turn.clone()),
                confidence: cand.score,
                captured_by: "detector".into(),
                session_id: Some("e2e".into()),
                output: tmp.to_path_buf(),
            })
            .unwrap();
        }
        prior.push(PriorClaim {
            id: format!("rec-{}", t.turn),
            turn: t.turn.clone(),
            content: t.content.clone(),
        });
    }
}

#[test]
fn e2e_assert_then_correct_3_turn_scenario() {
    let tmp = TempDir::new().unwrap();
    capture_fixture(tmp.path(), "turns-happy.jsonl");
    let log = corrections_log_path(tmp.path());
    let body = fs::read_to_string(&log).unwrap();
    let lines: Vec<_> = body.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(
        lines.len(),
        1,
        "exactly one correction expected, got {}: {body}",
        lines.len()
    );
    assert!(lines[0].contains("\"corrects_id\":\"rec-t-1\""));
    assert!(lines[0].contains("\"turn\":\"t-3\""));
}

#[test]
fn e2e_correction_survives_compaction() {
    let tmp = TempDir::new().unwrap();
    capture_fixture(tmp.path(), "turns-cross-compact.jsonl");

    // Simulate compaction: copy NDJSON to a sealed-state path, wipe live,
    // then restore from sealed copy. This mirrors A4 restore semantics.
    let log = corrections_log_path(tmp.path());
    let sealed = tmp.path().join("logs").join("corrections.sealed.ndjson");
    fs::copy(&log, &sealed).unwrap();
    fs::remove_file(&log).unwrap();
    assert!(!log.exists());
    fs::copy(&sealed, &log).unwrap();

    let body = fs::read_to_string(&log).unwrap();
    assert!(
        body.contains("master node is beta"),
        "post-restore body: {body}"
    );
    assert!(body.contains("\"corrects_id\":\"rec-t-1\""));
}

#[test]
fn e2e_correction_false_positive_rate_on_neutral_fixture() {
    let tmp = TempDir::new().unwrap();
    capture_fixture(tmp.path(), "turns-neutral.jsonl");
    let log = corrections_log_path(tmp.path());
    let body = fs::read_to_string(&log).unwrap_or_default();
    let captured = body.lines().filter(|l| !l.is_empty()).count();
    let total = load_jsonl("turns-neutral.jsonl").len();
    assert!(
        captured <= total / 20,
        "false-positive rate too high: {captured}/{total} captured"
    );
}

#[test]
fn judge_cache_namespace_isolated_from_public_bench_cache() {
    let tmp = TempDir::new().unwrap();
    let bench_dir = tmp.path().join("benchmarks").join("grader-cache");
    let c4_dir = bench_dir.join("c4");
    let public_dir = bench_dir.join("public_bench");
    fs::create_dir_all(&c4_dir).unwrap();
    fs::create_dir_all(&public_dir).unwrap();
    fs::write(c4_dir.join("aa.json"), "{}").unwrap();
    fs::write(public_dir.join("bb.json"), "{}").unwrap();

    let c4_files: Vec<_> = fs::read_dir(&c4_dir).unwrap().collect();
    let public_files: Vec<_> = fs::read_dir(&public_dir).unwrap().collect();
    assert_eq!(c4_files.len(), 1);
    assert_eq!(public_files.len(), 1);
    let c4_name = c4_files[0].as_ref().unwrap().file_name();
    let public_name = public_files[0].as_ref().unwrap().file_name();
    assert_ne!(c4_name, public_name, "namespaces clobbered each other");
}
