//! A3 Part 1 (Continuity Foundation) integration tests: file-interaction
//! ledger hook, precompact seal, wake surfacing, and `memd prime-reads`.

use super::*;
use crate::MemdClient;
use crate::cli::{
    HookArgs, HookFileInteractionArgs, HookMode, HookSealLedgerArgs, PrimeReadsArgs,
    run_hook_mode, run_prime_reads,
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

