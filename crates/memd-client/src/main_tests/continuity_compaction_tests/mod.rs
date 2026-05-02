//! A4 (Read-State Across Compaction) integration tests for `memd hook
//! restore`. Task A4.2 establishes tests 12–15 from `phase-a4-plan.md §4`.
//! Task A4.3 extends test 13 with breach-log assertions; Task A4.7 adds the
//! full E2E scenarios (18 + 19).

use super::*;
use crate::MemdClient;
use crate::cli::{
    HookArgs, HookDoctorArgs, HookDoctorCheck, HookMode, HookRestoreArgs, HookRestoreNoSealed,
    HookSealLedgerArgs, run_hook_doctor_ordering, run_hook_mode, run_hook_restore,
};
use memd_core::file_ledger::{
    FileInteractionLedger, append_file_interaction, ledger_path, restore::RestoreSource,
};

fn ordering_args(output: &Path, trace_inline: Option<&str>) -> HookDoctorArgs {
    HookDoctorArgs {
        project_root: None,
        json: false,
        check: Some(HookDoctorCheck::Ordering),
        trace: None,
        trace_inline: trace_inline.map(str::to_string),
        output: output.to_path_buf(),
    }
}

fn dummy_client() -> MemdClient {
    MemdClient::new("http://127.0.0.1:1").expect("build memd client for tests")
}

fn restore_args(output: &Path, session_id: &str, dry_run: bool) -> HookRestoreArgs {
    HookRestoreArgs {
        output: output.to_path_buf(),
        session_id: session_id.to_string(),
        latest_only: None,
        dry_run,
        json: false,
    }
}

async fn seal_via_hook(output: &Path, session_id: &str) {
    let args = HookArgs {
        mode: HookMode::SealLedger(HookSealLedgerArgs {
            output: output.to_path_buf(),
            session_id: session_id.to_string(),
        }),
    };
    run_hook_mode(&dummy_client(), "http://127.0.0.1:1", args)
        .await
        .expect("seal-ledger hook run");
}

fn seed_ledger(output: &Path, session_id: &str, paths: &[&str]) {
    for (i, p) in paths.iter().enumerate() {
        let payload = serde_json::json!({
            "session_id": session_id,
            "tool_name": "Read",
            "tool_input": { "file_path": p },
        });
        append_file_interaction(&payload, None, output, (i as i64) + 1).unwrap();
    }
}

/// Test 12: CLI seal-ledger then hook restore round-trips entries.
#[tokio::test]
async fn seal_then_restore_round_trips_ledger_entries() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path();
    let sid = "sess-roundtrip";
    seed_ledger(output, sid, &["a.rs", "b.rs", "c.rs"]);

    seal_via_hook(output, sid).await;
    // Simulate compaction wiping the active ledger.
    fs::remove_file(ledger_path(output, sid)).unwrap();
    assert!(!ledger_path(output, sid).exists());

    let report = run_hook_restore(&restore_args(output, sid, false)).expect("restore");
    assert!(report.ok);
    assert_eq!(report.entries, 3);
    assert_eq!(report.source, RestoreSource::Postcompact);

    let loaded = FileInteractionLedger::load_from_path(&ledger_path(output, sid)).unwrap();
    assert_eq!(loaded.distinct_paths(), vec!["a.rs", "b.rs", "c.rs"]);
}

/// Test 13: restore no-ops when no sealed ledger present.
/// A4.2 asserts report shape + sentinel error. A4.3 extends with breach-log
/// line assertion.
#[tokio::test]
async fn restore_noops_when_no_sealed_ledger_present() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path();
    let sid = "sess-nothing";

    let report = run_hook_restore(&restore_args(output, sid, false)).expect("restore call");
    assert!(!report.ok);
    assert_eq!(report.error.as_deref(), Some("no-sealed-ledger"));
    assert_eq!(report.entries, 0);
    assert!(report.sealed_path.is_none());
    assert!(!ledger_path(output, sid).exists());

    // A4.3: PostCompact source emits a breach line under <output>/logs/.
    let breach_log = output.join("logs/continuity-breach.log");
    assert!(
        breach_log.exists(),
        "breach log should be created at {breach_log:?}"
    );
    let text = fs::read_to_string(&breach_log).unwrap();
    let lines: Vec<&str> = text.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.len(), 1, "exactly one breach line expected");
    assert!(lines[0].contains("breach=no-sealed-ledger"));
    assert!(lines[0].contains(sid));

    // run_hook_mode should surface HookRestoreNoSealed so main.rs exits 2.
    let args = HookArgs {
        mode: HookMode::Restore(restore_args(output, sid, false)),
    };
    let err = run_hook_mode(&dummy_client(), "http://127.0.0.1:1", args)
        .await
        .expect_err("no-sealed must error");
    assert!(
        err.downcast_ref::<HookRestoreNoSealed>().is_some(),
        "expected HookRestoreNoSealed, got: {err:?}",
    );
}

/// Test 14: restore emits exactly one valid ndjson record.
#[tokio::test]
async fn restore_emits_ndjson_record() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path();
    let sid = "sess-ndjson";
    seed_ledger(output, sid, &["only.rs"]);
    seal_via_hook(output, sid).await;

    let _ = run_hook_restore(&restore_args(output, sid, false)).expect("restore");

    let log = output.join("logs/ledger-restore.ndjson");
    assert!(log.exists(), "ndjson log should be created at {log:?}");
    let text = fs::read_to_string(&log).unwrap();
    let lines: Vec<&str> = text.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.len(), 1, "expected exactly one ndjson line");

    let value: serde_json::Value = serde_json::from_str(lines[0]).expect("valid json");
    assert_eq!(value["session_id"], sid);
    assert_eq!(value["ok"], true);
    assert_eq!(value["source"], "postcompact-hook");
    assert_eq!(value["entries"], 1);
    assert!(value["ts_ms"].is_i64());
    assert!(value["restored_path"].is_string());
    assert!(value["sealed_path"].is_string());
}

/// Test 15: dry-run must not mutate the active ledger or emit ndjson telemetry.
#[tokio::test]
async fn cli_dry_run_does_not_mutate_disk() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path();
    let sid = "sess-dry";
    seed_ledger(output, sid, &["a.rs", "b.rs"]);
    seal_via_hook(output, sid).await;
    // Wipe active ledger to verify dry-run leaves it absent.
    fs::remove_file(ledger_path(output, sid)).unwrap();

    let report = run_hook_restore(&restore_args(output, sid, true)).expect("dry run");
    assert!(report.ok, "dry-run with sealed present reports ok");
    assert_eq!(report.entries, 2, "dry-run loads sealed to report count");
    assert!(report.sealed_path.is_some());

    // Side effects: none.
    assert!(
        !ledger_path(output, sid).exists(),
        "dry-run must not write active ledger"
    );
    assert!(
        !output.join("logs/ledger-restore.ndjson").exists(),
        "dry-run must not append ndjson telemetry"
    );
}

/// Test 8: canonical trace (PostCompact → LedgerRestore → PreToolUse) is clean.
#[tokio::test]
async fn ordering_check_passes_on_canonical_trace() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path();
    let trace = r#"[
        {"event":"PostCompact","ts_ms":1,"session_id":"s1"},
        {"event":"LedgerRestore","ts_ms":2,"session_id":"s1"},
        {"event":"PreToolUse","ts_ms":3,"tool":"Read","path":"a.rs","session_id":"s1"}
    ]"#;
    run_hook_doctor_ordering(&ordering_args(output, Some(trace))).expect("clean trace");

    // No breach line should be written.
    let breach = output.join("logs/continuity-breach.log");
    assert!(!breach.exists(), "clean trace must not emit breach log");
}

/// Test 9: PreToolUse fires after PostCompact with no LedgerRestore between.
#[tokio::test]
async fn ordering_check_flags_tool_before_restore() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path();
    let trace = r#"[
        {"event":"PostCompact","ts_ms":1,"session_id":"s2"},
        {"event":"PreToolUse","ts_ms":2,"tool":"Edit","path":"b.rs","session_id":"s2"}
    ]"#;
    let err = run_hook_doctor_ordering(&ordering_args(output, Some(trace)))
        .expect_err("tool-before-restore must error");
    let msg = format!("{err}");
    assert!(msg.contains("breach"), "expected breach in error: {msg}");

    let breach = output.join("logs/continuity-breach.log");
    assert!(breach.exists(), "breach log must be written");
    let text = fs::read_to_string(&breach).unwrap();
    assert!(text.contains("breach=tool-before-restore"), "got: {text}");
    assert!(text.contains("tool=Edit"), "got: {text}");
    assert!(text.contains("path=b.rs"), "got: {text}");
    assert!(text.contains("s2"), "got: {text}");
}

/// Test 10: PostCompact at end of trace with no matching LedgerRestore.
#[tokio::test]
async fn ordering_check_flags_missing_restore() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path();
    let trace = r#"[
        {"event":"PostCompact","ts_ms":1,"session_id":"s3"}
    ]"#;
    let err = run_hook_doctor_ordering(&ordering_args(output, Some(trace)))
        .expect_err("missing-restore must error");
    assert!(format!("{err}").contains("breach"));

    let text = fs::read_to_string(output.join("logs/continuity-breach.log")).unwrap();
    assert!(text.contains("breach=missing-restore"), "got: {text}");
    assert!(text.contains("s3"), "got: {text}");
}

/// Test 11: without --trace-inline or a default trace file, bail with
/// `trace-unavailable` diagnostic (non-green, but not a breach — the CLI
/// simply cannot audit without a trace).
#[tokio::test]
async fn ordering_check_requires_trace_file() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path();
    let err = run_hook_doctor_ordering(&ordering_args(output, None))
        .expect_err("missing trace must error");
    let msg = format!("{err}");
    assert!(
        msg.contains("trace-unavailable"),
        "expected trace-unavailable, got: {msg}"
    );
}

// ---------------------------------------------------------------------------
// A4.7 — E2E fixture-driven scenarios 18 + 19.
// Fixtures live at crates/memd-client/fixtures/a4/.
// ---------------------------------------------------------------------------

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("a4")
        .join(name)
}

fn load_fixture_ledger(name: &str, session_id: &str) -> FileInteractionLedger {
    let raw = fs::read_to_string(fixture_path(name)).expect("fixture exists");
    let mut value: serde_json::Value = serde_json::from_str(&raw).expect("fixture is JSON");
    // Rename session_id in-place so fixture is reusable across tests.
    if let Some(obj) = value.as_object_mut() {
        obj.insert(
            "session_id".into(),
            serde_json::Value::String(session_id.to_string()),
        );
    }
    serde_json::from_value(value).expect("fixture matches ledger schema")
}

/// Scenario 18: 5-file synthetic session, PreCompact seal, simulated wipe,
/// PostCompact restore. Assert all 5 paths retrievable byte-for-byte from
/// the fixture's expected ledger.
#[tokio::test]
async fn a4_compaction_survival_5_files() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path();
    let sid = "a4-scenario-18";

    // 1. Seed active ledger from fixture.
    let ledger = load_fixture_ledger("pre-compact-ledger.json", sid);
    ledger
        .save_to_path(&ledger_path(output, sid))
        .expect("seed pre-compact ledger");

    // 2. Seal via CLI hook (PreCompact).
    seal_via_hook(output, sid).await;

    // 3. Simulate compaction collapsing the active ledger.
    fs::remove_file(ledger_path(output, sid)).unwrap();
    assert!(!ledger_path(output, sid).exists());

    // 4. Restore via CLI (PostCompact).
    let report = run_hook_restore(&restore_args(output, sid, false)).expect("restore");
    assert!(report.ok);
    assert_eq!(report.entries, 5);
    assert_eq!(report.source, RestoreSource::Postcompact);

    // 5. Assert restored ledger has all 5 paths and matches the expected
    //    fixture byte-level (modulo session_id rename).
    let restored = FileInteractionLedger::load_from_path(&ledger_path(output, sid)).unwrap();
    let paths = restored.distinct_paths();
    assert_eq!(
        paths,
        vec![
            "src/a.rs".to_string(),
            "src/b.rs".to_string(),
            "src/c.rs".to_string(),
            "src/d.rs".to_string(),
            "src/e.rs".to_string(),
        ]
    );
    let expected = load_fixture_ledger("post-compact-expected.json", sid);
    assert_eq!(restored.entries, expected.entries);

    // 6. No breach log should exist on a healthy run.
    assert!(
        !output.join("logs/continuity-breach.log").exists(),
        "happy path must not write breach log"
    );
}

/// Scenario 19: same setup minus the restore call. An ordering audit driven
/// by `breach-transcript.jsonl` must flag `tool-before-restore` on the first
/// PreToolUse that hits after the PostCompact event.
#[tokio::test]
async fn a4_compaction_breach_detection() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path();
    let trace = fs::read_to_string(fixture_path("breach-transcript.jsonl"))
        .expect("breach transcript fixture");

    let err = run_hook_doctor_ordering(&ordering_args(output, Some(&trace)))
        .expect_err("breach transcript must fail ordering check");
    assert!(format!("{err}").contains("breach"));

    // Two breaches: tool-before-restore on the Edit that follows PostCompact
    // with no LedgerRestore between, and missing-restore because no
    // LedgerRestore event ever arrives before the trace ends.
    let text =
        fs::read_to_string(output.join("logs/continuity-breach.log")).expect("breach log exists");
    let lines: Vec<&str> = text.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.len(), 2, "expected two breach lines: {lines:?}");
    assert!(
        lines
            .iter()
            .any(|l| l.contains("breach=tool-before-restore")
                && l.contains("tool=Edit")
                && l.contains("path=src/a.rs")
                && l.contains("a4-breach")),
        "tool-before-restore line missing: {lines:?}"
    );
    assert!(
        lines
            .iter()
            .any(|l| l.contains("breach=missing-restore") && l.contains("a4-breach")),
        "missing-restore line missing: {lines:?}"
    );
}
