//! A4 (Read-State Across Compaction) integration tests for `memd hook
//! restore`. Task A4.2 establishes tests 12–15 from `phase-a4-plan.md §4`.
//! Task A4.3 extends test 13 with breach-log assertions; Task A4.7 adds the
//! full E2E scenarios (18 + 19).

use super::*;
use crate::cli::{
    HookArgs, HookMode, HookRestoreArgs, HookRestoreNoSealed, HookSealLedgerArgs,
    run_hook_mode, run_hook_restore,
};
use crate::MemdClient;
use memd_core::file_ledger::{
    append_file_interaction, ledger_path, restore::RestoreSource, FileInteractionLedger,
};

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
/// line assertion and CLI exit-code coverage.
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
