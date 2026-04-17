//! A3 Part 2 (Continuity Enforcement) integration tests: PreToolUse gate,
//! wake ## Continuity Gate block, preference replay (cross-process).

use super::*;
use crate::cli::cli_gate_runtime::run_gate;
use crate::cli::args::HookGateArgs;
use memd_core::enforcement::{EnforcementPolicy, gate_decision, format_gate_output};
use memd_core::file_ledger::{
    FileInteractionLedger, FileOp, append_file_interaction, ledger_path, seal_session_ledger,
};
use std::path::Path;

fn seed_sealed_paths(output: &Path, session: &str, paths: &[(&str, FileOp)]) {
    for (i, (p, op)) in paths.iter().enumerate() {
        let payload = serde_json::json!({
            "session_id": session,
            "tool_name": match op {
                FileOp::Read => "Read",
                FileOp::Edit => "Edit",
                FileOp::Write => "Write",
            },
            "tool_input": {"file_path": p},
        });
        append_file_interaction(&payload, None, output, (i as i64) + 1).unwrap();
    }
    seal_session_ledger(session, output).unwrap();
}

/// Pure-function test: gate_decision matrix (redundant with memd-core unit tests but kept for integration-level confidence).
#[test]
fn gate_decision_denies_when_block_and_sealed_path_not_fresh() {
    let sealed = vec!["a.rs".to_string()];
    let fresh: Vec<String> = vec![];
    assert!(matches!(
        gate_decision(EnforcementPolicy::Block, "a.rs", &sealed, &fresh),
        memd_core::enforcement::GateDecision::Deny { .. }
    ));
}

fn gate_args(out: &Path, policy: &str, payload: serde_json::Value) -> HookGateArgs {
    HookGateArgs {
        output: out.to_path_buf(),
        session_id: None,
        policy: Some(policy.into()),
        stdin: false,
        content: Some(payload.to_string()),
    }
}

#[tokio::test]
async fn hook_gate_denies_edit_on_sealed_path_without_fresh_read() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path();
    seed_sealed_paths(out, "sess-prev", &[("a.rs", FileOp::Edit)]);
    let args = gate_args(out, "block", serde_json::json!({
        "session_id": "sess-now",
        "tool_name": "Edit",
        "tool_input": {"file_path": "a.rs"}
    }));
    let stdout = run_gate(&args).await.unwrap().expect("deny emits output");
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(v["hookSpecificOutput"]["permissionDecision"], "deny");
    assert!(
        v["hookSpecificOutput"]["permissionDecisionReason"]
            .as_str()
            .unwrap()
            .contains("a.rs")
    );
}

#[tokio::test]
async fn hook_gate_allows_edit_when_path_freshly_read_in_current_session() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path();
    seed_sealed_paths(out, "sess-prev", &[("a.rs", FileOp::Edit)]);
    append_file_interaction(&serde_json::json!({
        "session_id": "sess-now",
        "tool_name": "Read",
        "tool_input": {"file_path": "a.rs"}
    }), None, out, 9).unwrap();

    let args = gate_args(out, "block", serde_json::json!({
        "session_id": "sess-now",
        "tool_name": "Edit",
        "tool_input": {"file_path": "a.rs"}
    }));
    assert!(run_gate(&args).await.unwrap().is_none());
}

#[tokio::test]
async fn hook_gate_warn_emits_system_message_not_deny() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path();
    seed_sealed_paths(out, "sess-prev", &[("a.rs", FileOp::Edit)]);
    let args = gate_args(out, "warn", serde_json::json!({
        "session_id": "sess-now",
        "tool_name": "Edit",
        "tool_input": {"file_path": "a.rs"}
    }));
    let stdout = run_gate(&args).await.unwrap().expect("warn emits output");
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(v["systemMessage"].as_str().unwrap().contains("a.rs"));
    assert_ne!(v["hookSpecificOutput"]["permissionDecision"], "deny");
}
