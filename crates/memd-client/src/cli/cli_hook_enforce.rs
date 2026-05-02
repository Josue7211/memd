//! `memd hooks enforce` — B4 wrapper around an inner hook command.
//!
//! Contract: `docs/contracts/hook-order.md`.
//!
//! Flow:
//! 1. Parse the `--event` token; unknown → exit 3 (`contract-parse`).
//! 2. Resolve budget (override → contract default → unbounded).
//! 3. Resolve failure class (override → contract default).
//! 4. Spawn inner command under `run_with_budget`.
//! 5. Append a trace line with outcome.
//! 6. Halt-class failure → exit 1 (inner nonzero) or exit 2 (timeout).
//! 7. Log-class failure → exit 0 (trace line records `failure_class`).

use super::args::{HookEnforceArgs, HookFailureClassArg};
use anyhow::{Context, Result};
use memd_core::hook_runtime::{
    BudgetOutcome, DEFAULT_WAIT_MS, FailureClass, FireOrderValidator, HookBudget, HookEvent,
    HookRecord, HookSessionLock, HookTrace, ViolationKind, run_with_budget,
};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::time::Duration;

/// Exit codes as documented in `hook-order.md §4`.
pub(crate) const EXIT_OK: i32 = 0;
pub(crate) const EXIT_HALT_INNER: i32 = 1;
pub(crate) const EXIT_HALT_TIMEOUT: i32 = 2;
pub(crate) const EXIT_CONTRACT_PARSE: i32 = 3;
/// Lock contention past `MEMD_HOOK_LOCK_WAIT_MS`. Treated as halt-class.
pub(crate) const EXIT_HALT_CONTENDED: i32 = 4;

/// Sentinel error: `memd hooks enforce` wants a specific exit code.
/// `main.rs` downcasts this and `process::exit`s with the code.
#[derive(Debug)]
pub(crate) struct HookEnforceExitCode(pub i32);

impl std::fmt::Display for HookEnforceExitCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "memd hooks enforce: exit code {}", self.0)
    }
}

impl std::error::Error for HookEnforceExitCode {}

pub(crate) fn run_hook_enforce(args: &HookEnforceArgs) -> Result<i32> {
    // Opt-out: MEMD_HOOK_ENFORCE=0 → run inner directly, no trace.
    let enforce_flag = std::env::var("MEMD_HOOK_ENFORCE").unwrap_or_else(|_| "0".to_string());
    if enforce_flag == "0" {
        if args.inner.is_empty() {
            return Ok(EXIT_OK);
        }
        let status = Command::new(&args.inner[0])
            .args(&args.inner[1..])
            .status()
            .with_context(|| format!("spawn inner: {:?}", args.inner))?;
        return Ok(status.code().unwrap_or(EXIT_HALT_INNER));
    }

    let event = match HookEvent::from_str(&args.event) {
        Ok(ev) => ev,
        Err(_) => {
            eprintln!(
                "memd hooks enforce: unknown event token `{}` — not in docs/contracts/hook-order.md §1",
                args.event
            );
            return Ok(EXIT_CONTRACT_PARSE);
        }
    };

    let budget_ms = args
        .budget_ms
        .or_else(|| event.default_budget_ms())
        .or_else(|| {
            std::env::var("MEMD_HOOK_BUDGET_OVERRIDE")
                .ok()
                .and_then(|raw| parse_budget_override(&raw, &event))
        });
    let budget = HookBudget::from_ms(budget_ms);

    let posture = args
        .failure_class
        .map(|c| match c {
            HookFailureClassArg::Halt => FailureClass::Halt,
            HookFailureClassArg::Log => FailureClass::Log,
        })
        .unwrap_or_else(|| event.failure_class_default());

    let trace = HookTrace::new(resolve_trace_path(args));

    // Fire-order enforcement: replay prior trace lines for this session,
    // then check the current event against the validator. Halt-class
    // violation → exit 1 with OrderViolation trace line.
    if let Some(violation) = validate_fire_order(trace.path(), &args.session_id, event) {
        let record = build_record(event, args, budget_ms, 0, 0, FailureClass::OrderViolation);
        trace.append(&record)?;
        eprintln!(
            "memd hooks enforce: fire-order violation for session={} event={}: {}",
            args.session_id, event, violation
        );
        return Ok(EXIT_HALT_INNER);
    }

    // Per-(session,event) advisory lock. Drops at function return.
    let lock_wait = Duration::from_millis(
        std::env::var("MEMD_HOOK_LOCK_WAIT_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_WAIT_MS),
    );
    let _lock = match HookSessionLock::acquire(&args.output, &args.session_id, event, lock_wait) {
        Ok(g) => g,
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
            let record = build_record(event, args, budget_ms, 0, 0, FailureClass::OrderViolation);
            trace.append(&record)?;
            eprintln!(
                "memd hooks enforce: lock contended for session={} event={} (waited {:?})",
                args.session_id, event, lock_wait
            );
            return Ok(EXIT_HALT_CONTENDED);
        }
        Err(e) => return Err(e.into()),
    };

    // Empty inner → observability beacon only (no command to wrap).
    if args.inner.is_empty() {
        let record = build_record(event, args, budget_ms, 0, 0, FailureClass::None);
        trace.append(&record)?;
        return Ok(EXIT_OK);
    }

    let mut cmd = Command::new(&args.inner[0]);
    cmd.args(&args.inner[1..])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let outcome =
        run_with_budget(cmd, budget).with_context(|| format!("spawn inner: {:?}", args.inner))?;

    let class = outcome.failure_class;
    let record = build_record(
        event,
        args,
        budget_ms,
        outcome.elapsed_ms,
        outcome.exit_code,
        class,
    );
    trace.append(&record)?;

    Ok(map_exit(&outcome, posture))
}

/// Replay trace lines for `session_id` into a fresh validator, then
/// observe `event`. Returns `Some(msg)` on contract breach, `None` on
/// clean observation. Replay errors are swallowed — we're reconstructing
/// state, not re-validating past fires.
fn validate_fire_order(
    trace_path: &std::path::Path,
    session_id: &str,
    event: HookEvent,
) -> Option<String> {
    let mut validator = FireOrderValidator::new();
    if let Ok(file) = std::fs::File::open(trace_path) {
        for line in BufReader::new(file).lines().map_while(Result::ok) {
            let Ok(record) = serde_json::from_str::<HookRecord>(&line) else {
                continue;
            };
            if record.session_id != session_id {
                continue;
            }
            // Skip order-violation markers so we don't double-count the
            // rejected event when the wrapper is retried.
            if matches!(record.failure_class, FailureClass::OrderViolation) {
                continue;
            }
            let _ = validator.observe(record.event);
        }
    }
    match validator.observe(event) {
        Ok(()) => None,
        // Runtime only halts on the canonical swap (PostCompact before
        // PreCompact). MissingPredecessor is left to `hooks doctor
        // --check contract` so tests + bootstrap paths that skip
        // SessionStart don't cascade-fail at runtime.
        Err(ViolationKind::OrderSwap { .. }) => {
            Some("OrderSwap: PostCompact before PreCompact".to_string())
        }
        Err(ViolationKind::MissingPredecessor { .. }) => None,
    }
}

fn resolve_trace_path(args: &HookEnforceArgs) -> std::path::PathBuf {
    if let Some(explicit) = &args.trace {
        return explicit.clone();
    }
    if let Ok(env) = std::env::var("MEMD_HOOK_TRACE_PATH") {
        return std::path::PathBuf::from(env);
    }
    args.output.join("logs").join("hook-trace.ndjson")
}

fn build_record(
    event: HookEvent,
    args: &HookEnforceArgs,
    budget_ms: Option<u64>,
    elapsed_ms: u64,
    exit_code: i32,
    class: FailureClass,
) -> HookRecord {
    let mut record = HookRecord::new(event, &args.session_id).with_harness(&args.harness);
    if let Some(ms) = budget_ms {
        record = record.with_budget_ms(ms);
    }
    record = record.with_outcome(elapsed_ms, exit_code, class);
    if let Some(tool) = &args.tool {
        record = record.with_tool(tool);
    }
    if let Some(path) = &args.path {
        record = record.with_path(path);
    }
    record
}

fn map_exit(outcome: &BudgetOutcome, posture: FailureClass) -> i32 {
    // posture: Halt or Log (decision). outcome: concrete failure class.
    match (posture, &outcome.failure_class) {
        (_, FailureClass::None) => EXIT_OK,
        (FailureClass::Halt, FailureClass::Timeout) => EXIT_HALT_TIMEOUT,
        (FailureClass::Halt, _) => EXIT_HALT_INNER,
        (FailureClass::Log, _) => EXIT_OK, // trace records the failure_class
        _ => EXIT_OK,
    }
}

fn parse_budget_override(raw: &str, event: &HookEvent) -> Option<u64> {
    let needle = event.as_str();
    for pair in raw.split(',') {
        let mut it = pair.splitn(2, '=');
        let k = it.next()?.trim();
        let v = it.next()?.trim();
        if k.eq_ignore_ascii_case(needle) {
            return v.parse::<u64>().ok();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use memd_core::hook_runtime::FailureClass;

    #[test]
    fn map_exit_halt_class_returns_exit_1_on_inner_failure() {
        let outcome = BudgetOutcome {
            elapsed_ms: 100,
            exit_code: 3,
            failure_class: FailureClass::InnerNonzero,
            timed_out: false,
        };
        assert_eq!(map_exit(&outcome, FailureClass::Halt), EXIT_HALT_INNER);
    }

    #[test]
    fn map_exit_halt_class_returns_exit_2_on_timeout() {
        let outcome = BudgetOutcome {
            elapsed_ms: 600,
            exit_code: 124,
            failure_class: FailureClass::Timeout,
            timed_out: true,
        };
        assert_eq!(map_exit(&outcome, FailureClass::Halt), EXIT_HALT_TIMEOUT);
    }

    #[test]
    fn map_exit_log_class_returns_exit_0_on_inner_failure() {
        let outcome = BudgetOutcome {
            elapsed_ms: 100,
            exit_code: 3,
            failure_class: FailureClass::InnerNonzero,
            timed_out: false,
        };
        assert_eq!(map_exit(&outcome, FailureClass::Log), EXIT_OK);
    }

    #[test]
    fn map_exit_success_returns_exit_0_either_posture() {
        let outcome = BudgetOutcome {
            elapsed_ms: 10,
            exit_code: 0,
            failure_class: FailureClass::None,
            timed_out: false,
        };
        assert_eq!(map_exit(&outcome, FailureClass::Halt), EXIT_OK);
        assert_eq!(map_exit(&outcome, FailureClass::Log), EXIT_OK);
    }

    #[test]
    fn parses_env_budget_override() {
        let raw = "PreCompact=10000,PostCompact=3000";
        assert_eq!(
            parse_budget_override(raw, &HookEvent::PreCompact),
            Some(10_000)
        );
        assert_eq!(
            parse_budget_override(raw, &HookEvent::PostCompact),
            Some(3_000)
        );
        assert_eq!(parse_budget_override(raw, &HookEvent::Stop), None);
    }
}
