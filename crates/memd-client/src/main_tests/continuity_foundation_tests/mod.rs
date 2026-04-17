//! A3 Part 1 (Continuity Foundation) integration tests: file-interaction
//! ledger hook, precompact seal, wake surfacing, and `memd prime-reads`.

use super::*;
use crate::MemdClient;
use crate::cli::{
    HookArgs, HookFileInteractionArgs, HookMode, HookSealLedgerArgs, PrimeReadsArgs,
    run_hook_mode, run_lifecycle_probe, run_prime_reads,
};
use memd_core::file_ledger::{
    FileInteractionLedger, FileOp, append_file_interaction, ledger_path, seal_session_ledger,
    session_dir,
};

pub(crate) fn seed_prior_session_ledger(output: &Path, session_id: &str, ops: &[(&str, FileOp)]) {
    for (idx, (path, op)) in ops.iter().enumerate() {
        let payload = serde_json::json!({
            "session_id": session_id,
            "tool_name": match op {
                FileOp::Read => "Read",
                FileOp::Edit => "Edit",
                FileOp::Write => "Write",
            },
            "tool_input": { "file_path": path },
        });
        append_file_interaction(&payload, None, output, (idx as i64) + 1).unwrap();
    }
    seal_session_ledger(session_id, output).unwrap();
}

fn dummy_client() -> MemdClient {
    MemdClient::new("http://127.0.0.1:1").expect("build memd client for tests")
}

#[tokio::test]
async fn hook_file_interaction_appends_ledger_entry() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().to_path_buf();
    let payload = serde_json::json!({
        "session_id": "sess-abc",
        "tool_name": "Read",
        "tool_input": {"file_path": "/tmp/foo.rs"}
    })
    .to_string();
    let args = HookArgs {
        mode: HookMode::FileInteraction(HookFileInteractionArgs {
            output: output.clone(),
            session_id: None,
            stdin: false,
            content: Some(payload),
        }),
    };
    run_hook_mode(&dummy_client(), "http://127.0.0.1:1", args)
        .await
        .expect("hook file-interaction run");

    let lp = ledger_path(&output, "sess-abc");
    assert!(lp.exists(), "ledger file should be created at {lp:?}");
    let ledger = FileInteractionLedger::load_from_path(&lp).unwrap();
    assert_eq!(ledger.entries.len(), 1);
    assert_eq!(ledger.entries[0].path, "/tmp/foo.rs");
    assert_eq!(ledger.entries[0].op, FileOp::Read);
}

#[tokio::test]
async fn hook_seal_ledger_copies_current_to_sealed_dir() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().to_path_buf();
    let payload = serde_json::json!({
        "session_id": "sess-seal",
        "tool_name": "Edit",
        "tool_input": {"file_path": "a.rs"}
    });
    append_file_interaction(&payload, None, &output, 1).unwrap();

    let args = HookArgs {
        mode: HookMode::SealLedger(HookSealLedgerArgs {
            output: output.clone(),
            session_id: "sess-seal".into(),
        }),
    };
    run_hook_mode(&dummy_client(), "http://127.0.0.1:1", args)
        .await
        .expect("hook seal-ledger run");

    let sealed_dir = session_dir(&output, "sess-seal").join("sealed");
    assert!(sealed_dir.exists(), "sealed dir missing: {sealed_dir:?}");
    let entries: Vec<_> = fs::read_dir(&sealed_dir).unwrap().collect();
    assert_eq!(entries.len(), 1, "expected exactly one sealed ledger");
}

#[tokio::test]
async fn hook_seal_ledger_is_noop_when_no_ledger_exists() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().to_path_buf();
    let args = HookArgs {
        mode: HookMode::SealLedger(HookSealLedgerArgs {
            output,
            session_id: "sess-missing".into(),
        }),
    };
    run_hook_mode(&dummy_client(), "http://127.0.0.1:1", args)
        .await
        .expect("seal-ledger on missing ledger should succeed silently");
}

/// A3 Part 1 acceptance gate: session A edits 5 files → precompact seal →
/// session B can surface all 5 via `collect_files_touched` + `prime-reads`.
/// This proves the **surfacing** flow only; enforcement is Part 2.
#[tokio::test]
async fn compaction_mid_edit_flow_surfaces_prior_session_files() {
    use crate::runtime::collect_files_touched;
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().to_path_buf();

    // Session A: 5 file-interaction hook invocations
    for file in ["a.rs", "b.rs", "c.rs", "d.rs", "e.rs"] {
        let payload = serde_json::json!({
            "session_id": "sess-A",
            "tool_name": "Edit",
            "tool_input": { "file_path": file },
        })
        .to_string();
        let args = HookArgs {
            mode: HookMode::FileInteraction(HookFileInteractionArgs {
                output: output.clone(),
                session_id: None,
                stdin: false,
                content: Some(payload),
            }),
        };
        run_hook_mode(&dummy_client(), "http://127.0.0.1:1", args)
            .await
            .expect("file-interaction hook");
    }

    // Precompact: seal the ledger
    let seal_args = HookArgs {
        mode: HookMode::SealLedger(HookSealLedgerArgs {
            output: output.clone(),
            session_id: "sess-A".into(),
        }),
    };
    run_hook_mode(&dummy_client(), "http://127.0.0.1:1", seal_args)
        .await
        .expect("seal-ledger");

    // Session B: collect_files_touched surfaces all 5
    let paths = collect_files_touched(&output);
    for f in ["a.rs", "b.rs", "c.rs", "d.rs", "e.rs"] {
        assert!(
            paths.iter().any(|p| p == f),
            "prior-session file {f} missing from collect_files_touched: {paths:?}"
        );
    }

    // prime-reads wires the same data — verify it runs without error
    let pr_args = PrimeReadsArgs {
        output,
        since_session: None,
    };
    run_prime_reads(&pr_args).expect("prime-reads after seal");
}

#[test]
fn prime_reads_runs_with_populated_ledger() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().to_path_buf();
    seed_prior_session_ledger(
        &output,
        "sess-prev",
        &[("a.rs", FileOp::Read), ("b.rs", FileOp::Edit)],
    );
    let args = PrimeReadsArgs {
        output,
        since_session: None,
    };
    run_prime_reads(&args).expect("prime-reads must not error on populated ledger");
}

#[test]
fn prime_reads_since_session_reads_live_ledger() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().to_path_buf();
    // seed a live ledger (no seal)
    let payload = serde_json::json!({
        "session_id": "sess-live",
        "tool_name": "Write",
        "tool_input": {"file_path": "only.rs"},
    });
    memd_core::file_ledger::append_file_interaction(&payload, None, &output, 1).unwrap();
    let args = PrimeReadsArgs {
        output,
        since_session: Some("sess-live".into()),
    };
    run_prime_reads(&args).expect("prime-reads --since-session must not error");
}

/// A3-D3 lifecycle probe: store → recall → expire → verify-expired against
/// a live memd server. Skips silently if the server at `MEMD_TEST_BASE_URL`
/// (default `http://127.0.0.1:8787`) is unreachable so `cargo test` stays
/// green in environments without a running daemon. Task 11's gate runs the
/// CLI path (`memd diagnostics lifecycle-probe`) for the authoritative check.
#[tokio::test]
async fn lifecycle_probe_reports_green_on_healthy_server() {
    let base_url = std::env::var("MEMD_TEST_BASE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8787".to_string());
    let client = MemdClient::new(&base_url).expect("build client");
    match client.healthz().await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("skipping lifecycle_probe test — server {base_url} unreachable: {e}");
            return;
        }
    }
    let report = run_lifecycle_probe(&client).await;
    assert_eq!(
        report.status, "green",
        "lifecycle probe red: {:#?}",
        report.steps
    );
    assert!(report.steps.iter().all(|s| s.ok));
    assert_eq!(report.steps.len(), 4);
    let names: Vec<_> = report.steps.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["store", "recall", "expire", "verify_expired"]);
}

#[test]
fn collect_files_touched_returns_distinct_paths_from_sealed_ledger() {
    use crate::runtime::collect_files_touched;
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path();
    seed_prior_session_ledger(
        output,
        "sess-prev",
        &[
            ("crates/memd-core/src/lib.rs", FileOp::Read),
            ("crates/memd-core/src/lib.rs", FileOp::Edit),
            ("ROADMAP.md", FileOp::Read),
        ],
    );
    let paths = collect_files_touched(output);
    assert!(paths.contains(&"crates/memd-core/src/lib.rs".to_string()));
    assert!(paths.contains(&"ROADMAP.md".to_string()));
    // distinct
    assert_eq!(paths.len(), 2);
}

