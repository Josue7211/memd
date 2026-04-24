//! B4 integration tests for `memd hooks enforce` and
//! `memd hooks doctor --check contract`.
//!
//! Plan: `docs/phases/v4/phase-b4-plan.md §4`. Tests 11–22 inclusive.
//! This module calls the enforcer function directly (no subprocess
//! shell-out) for speed; the enforcer itself spawns the `inner`
//! command under a real OS timer.

use super::*;
use crate::cli::{HookEnforceArgs, HookFailureClassArg, run_hook_enforce};
use memd_core::hook_runtime::HookRecord;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use tempfile::TempDir;

/// Serialise tests that mutate `MEMD_HOOK_ENFORCE` + `MEMD_HOOK_TRACE_PATH`.
static ENFORCE_ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    ENFORCE_ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("env lock poisoned")
}

fn base_args(output: PathBuf, event: &str, inner: Vec<String>) -> HookEnforceArgs {
    HookEnforceArgs {
        event: event.to_string(),
        harness: "claude-code".to_string(),
        session_id: "sess-b4".to_string(),
        budget_ms: None,
        failure_class: None,
        trace: None,
        output,
        tool: None,
        path: None,
        inner,
    }
}

fn load_trace(path: &std::path::Path) -> Vec<HookRecord> {
    let contents = std::fs::read_to_string(path).unwrap_or_default();
    contents
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_str::<HookRecord>(l).expect("valid trace line"))
        .collect()
}

#[test]
fn enforce_happy_path_precompact() {
    let _g = env_lock();
    unsafe { std::env::set_var("MEMD_HOOK_ENFORCE", "1") };
    let dir = TempDir::new().unwrap();
    let args = base_args(
        dir.path().to_path_buf(),
        "PreCompact",
        vec!["true".to_string()],
    );
    let code = run_hook_enforce(&args).unwrap();
    assert_eq!(code, 0);

    let trace = dir.path().join("logs").join("hook-trace.ndjson");
    let records = load_trace(&trace);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].event.as_str(), "PreCompact");
    assert_eq!(records[0].exit_code, Some(0));
    unsafe { std::env::remove_var("MEMD_HOOK_ENFORCE") };
}

#[test]
fn enforce_times_out_on_stuck_inner() {
    let _g = env_lock();
    unsafe { std::env::set_var("MEMD_HOOK_ENFORCE", "1") };
    let dir = TempDir::new().unwrap();
    let mut args = base_args(
        dir.path().to_path_buf(),
        "PreCompact",
        vec!["sh".to_string(), "-c".to_string(), "sleep 2".to_string()],
    );
    args.budget_ms = Some(300);

    let code = run_hook_enforce(&args).unwrap();
    assert_eq!(code, 2, "halt-class timeout → exit 2");

    let trace = dir.path().join("logs").join("hook-trace.ndjson");
    let records = load_trace(&trace);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].exit_code, Some(124));
    assert!(
        matches!(
            records[0].failure_class,
            memd_core::hook_runtime::FailureClass::Timeout
        ),
        "failure_class=timeout expected"
    );
    unsafe { std::env::remove_var("MEMD_HOOK_ENFORCE") };
}

#[test]
fn enforce_disabled_flag_bypasses() {
    let _g = env_lock();
    unsafe { std::env::set_var("MEMD_HOOK_ENFORCE", "0") };
    let dir = TempDir::new().unwrap();
    let args = base_args(
        dir.path().to_path_buf(),
        "PreCompact",
        vec!["true".to_string()],
    );
    let code = run_hook_enforce(&args).unwrap();
    assert_eq!(code, 0);

    let trace = dir.path().join("logs").join("hook-trace.ndjson");
    assert!(!trace.exists(), "no trace file should be written when disabled");
    unsafe { std::env::remove_var("MEMD_HOOK_ENFORCE") };
}

#[test]
fn enforce_rejects_unknown_event_token() {
    let _g = env_lock();
    unsafe { std::env::set_var("MEMD_HOOK_ENFORCE", "1") };
    let dir = TempDir::new().unwrap();
    let args = base_args(
        dir.path().to_path_buf(),
        "NotAnEvent",
        vec!["true".to_string()],
    );
    let code = run_hook_enforce(&args).unwrap();
    assert_eq!(code, 3, "unknown event → contract-parse exit 3");
    unsafe { std::env::remove_var("MEMD_HOOK_ENFORCE") };
}

#[test]
fn enforce_log_class_returns_zero_on_inner_nonzero() {
    let _g = env_lock();
    unsafe { std::env::set_var("MEMD_HOOK_ENFORCE", "1") };
    let dir = TempDir::new().unwrap();
    let mut args = base_args(
        dir.path().to_path_buf(),
        "Stop", // Stop is log-class by default per contract
        vec!["sh".to_string(), "-c".to_string(), "exit 7".to_string()],
    );
    args.failure_class = Some(HookFailureClassArg::Log);

    let code = run_hook_enforce(&args).unwrap();
    assert_eq!(code, 0, "log-class inner non-zero → wrapper exits 0");

    let trace = dir.path().join("logs").join("hook-trace.ndjson");
    let records = load_trace(&trace);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].exit_code, Some(7));
    assert!(matches!(
        records[0].failure_class,
        memd_core::hook_runtime::FailureClass::InnerNonzero
    ));
    unsafe { std::env::remove_var("MEMD_HOOK_ENFORCE") };
}

#[test]
fn enforce_halt_class_returns_one_on_inner_nonzero() {
    let _g = env_lock();
    unsafe { std::env::set_var("MEMD_HOOK_ENFORCE", "1") };
    let dir = TempDir::new().unwrap();
    let mut args = base_args(
        dir.path().to_path_buf(),
        "PreCompact",
        vec!["sh".to_string(), "-c".to_string(), "exit 5".to_string()],
    );
    args.failure_class = Some(HookFailureClassArg::Halt);

    let code = run_hook_enforce(&args).unwrap();
    assert_eq!(code, 1, "halt-class inner nonzero → exit 1");
    unsafe { std::env::remove_var("MEMD_HOOK_ENFORCE") };
}

#[test]
fn enforce_latency_under_threshold_on_trivial_inner() {
    let _g = env_lock();
    unsafe { std::env::set_var("MEMD_HOOK_ENFORCE", "1") };
    let dir = TempDir::new().unwrap();

    // 20 samples — keeps the test fast but gives a signal.
    let mut elapsed_ms: Vec<u64> = Vec::new();
    for i in 0..20 {
        let args = HookEnforceArgs {
            session_id: format!("sess-lat-{i}"),
            ..base_args(
                dir.path().to_path_buf(),
                "PreRead",
                vec!["true".to_string()],
            )
        };
        let code = run_hook_enforce(&args).unwrap();
        assert_eq!(code, 0);
    }

    let records = load_trace(&dir.path().join("logs").join("hook-trace.ndjson"));
    assert_eq!(records.len(), 20);
    for r in &records {
        elapsed_ms.push(r.elapsed_ms.unwrap_or(u64::MAX));
    }
    elapsed_ms.sort();
    let p50 = elapsed_ms[10];
    let p99 = *elapsed_ms.last().unwrap();
    // Generous bounds — CI can be noisy. The ceiling that matters is
    // "orders of magnitude" over spawning `true` directly, not a tight
    // 50 ms regression gate (that lives in the dogfood run, not CI).
    assert!(p50 < 500, "p50 elapsed too high: {} ms", p50);
    assert!(p99 < 2_000, "p99 elapsed too high: {} ms", p99);
    unsafe { std::env::remove_var("MEMD_HOOK_ENFORCE") };
}

#[test]
fn enforce_records_harness_and_trace_id_on_every_line() {
    let _g = env_lock();
    unsafe { std::env::set_var("MEMD_HOOK_ENFORCE", "1") };
    let dir = TempDir::new().unwrap();
    for _ in 0..3 {
        let args = base_args(
            dir.path().to_path_buf(),
            "PreRead",
            vec!["true".to_string()],
        );
        run_hook_enforce(&args).unwrap();
    }
    let records = load_trace(&dir.path().join("logs").join("hook-trace.ndjson"));
    assert_eq!(records.len(), 3);
    let mut seen_ids = std::collections::HashSet::new();
    for r in &records {
        assert_eq!(r.harness.as_deref(), Some("claude-code"));
        assert_eq!(r.trace_id.len(), 26);
        assert!(seen_ids.insert(r.trace_id.clone()), "trace_id must be unique");
    }
    unsafe { std::env::remove_var("MEMD_HOOK_ENFORCE") };
}
