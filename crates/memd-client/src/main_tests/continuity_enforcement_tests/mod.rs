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

#[test]
fn collect_un_read_paths_returns_sealed_minus_fresh_reads() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path();
    seed_sealed_paths(out, "sess-prev", &[("a.rs", FileOp::Edit), ("b.rs", FileOp::Read)]);
    // sess-now has read a.rs but NOT b.rs
    let read_payload = serde_json::json!({
        "session_id": "sess-now",
        "tool_name": "Read",
        "tool_input": {"file_path": "a.rs"}
    });
    append_file_interaction(&read_payload, None, out, 9).unwrap();
    let un_read = crate::runtime::collect_un_read_paths(out, "sess-now");
    assert!(!un_read.contains(&"a.rs".to_string()), "a.rs is fresh-read");
    assert!(un_read.contains(&"b.rs".to_string()), "b.rs is un-read");
}

#[test]
fn render_continuity_gate_block_lists_un_read_paths() {
    let block = crate::runtime::render_continuity_gate_block(&["a.rs".into(), "b.rs".into()], false);
    assert!(block.contains("## Continuity Gate"));
    assert!(block.contains("a.rs"));
    assert!(block.contains("b.rs"));
}

#[test]
fn render_continuity_gate_block_is_empty_when_un_read_list_is_empty() {
    assert_eq!(crate::runtime::render_continuity_gate_block(&[], false), String::new());
}

fn seed_file_interaction(out: &Path, session: &str, tool: &str, path: &str, ts: i64) {
    append_file_interaction(
        &serde_json::json!({
            "session_id": session,
            "tool_name": tool,
            "tool_input": {"file_path": path}
        }),
        None,
        out,
        ts,
    )
    .unwrap();
}

#[tokio::test]
async fn enforcement_end_to_end_seal_deny_read_allow() {
    use crate::runtime::collect_un_read_paths;
    use crate::runtime::render_continuity_gate_block;
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path();

    // Session A: edit 3 files, then seal.
    for (i, f) in ["a.rs", "b.rs", "c.rs"].iter().enumerate() {
        seed_file_interaction(out, "sess-A", "Edit", f, (i as i64) + 1);
    }
    seal_session_ledger("sess-A", out).unwrap();

    // Session B wake-block should list all three un-read paths.
    let un_read = collect_un_read_paths(out, "sess-B");
    let block = render_continuity_gate_block(&un_read, false);
    assert!(block.contains("## Continuity Gate"));
    for f in ["a.rs", "b.rs", "c.rs"] {
        assert!(block.contains(f), "gate block missing {f}");
    }

    // Gate denies Edit on a.rs in sess-B.
    let deny_args = gate_args(out, "block", serde_json::json!({
        "session_id": "sess-B",
        "tool_name": "Edit",
        "tool_input": {"file_path": "a.rs"}
    }));
    let deny = run_gate(&deny_args).await.unwrap().expect("deny emits JSON");
    let v: serde_json::Value = serde_json::from_str(&deny).unwrap();
    assert_eq!(v["hookSpecificOutput"]["permissionDecision"], "deny");

    // Simulate Read of a.rs in sess-B.
    seed_file_interaction(out, "sess-B", "Read", "a.rs", 100);

    // Gate now allows (None = no output).
    let allow_args = gate_args(out, "block", serde_json::json!({
        "session_id": "sess-B",
        "tool_name": "Edit",
        "tool_input": {"file_path": "a.rs"}
    }));
    assert!(run_gate(&allow_args).await.unwrap().is_none());
}

#[test]
fn contract_verify_exits_nonzero_when_policy_configured_but_hook_not_wired() {
    use crate::cli::run_contract_verify;
    use crate::cli::args::ContractVerifyArgs;
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().to_path_buf();
    // Write config.json enabling enforcement but no PreToolUse script in the bundle.
    std::fs::write(
        out.join("config.json"),
        serde_json::json!({"continuity":{"enforcement":"block"}}).to_string()
    ).unwrap();
    // Seed a default contract.json so verify has something to read.
    let contract = memd_core::contract::MemdContract::default();
    std::fs::write(
        out.join("contract.json"),
        serde_json::to_string_pretty(&contract).unwrap(),
    ).unwrap();
    // No hooks/memd-pretool-gate.sh created under `out/hooks/` — simulating a bundle with
    // enforcement-policy configured but the gate hook NOT wired.

    let args = ContractVerifyArgs {
        output: out,
        json: false,
    };
    let result = run_contract_verify(&args);
    // run_contract_verify returns Err when violations exist.
    assert!(result.is_err(), "should fail when enforcement_policy_configured=true but enforcement_hook_wired=false");
}
