//! Budget-bounded command wrapper.
//!
//! `run_with_budget` spawns a child, polls `try_wait` on a short
//! interval, and sends SIGKILL (or the platform equivalent) if the
//! budget expires. Hand-rolled rather than depending on `wait_timeout`
//! to keep the workspace dep surface minimal — the poll interval is
//! small enough (10 ms) that wall-clock overhead is negligible against
//! the smallest event budget (500 ms).

use super::FailureClass;
use std::process::{Command, ExitStatus};
use std::thread;
use std::time::{Duration, Instant};

/// Default per-poll sleep for [`run_with_budget`].
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Outcome of a budgeted inner command.
#[derive(Debug, Clone)]
pub struct BudgetOutcome {
    pub elapsed_ms: u64,
    pub exit_code: i32,
    pub failure_class: FailureClass,
    pub timed_out: bool,
}

impl BudgetOutcome {
    pub fn ok(&self) -> bool {
        matches!(self.failure_class, FailureClass::None)
    }
}

/// Optional per-event budget. `None` means "no budget" — the caller
/// (e.g. LedgerSeal) inherits its surrounding event's timer.
#[derive(Debug, Clone, Copy)]
pub struct HookBudget {
    pub limit: Option<Duration>,
}

impl HookBudget {
    pub fn from_ms(ms: Option<u64>) -> Self {
        Self {
            limit: ms.map(Duration::from_millis),
        }
    }

    pub fn limit_ms(&self) -> Option<u64> {
        self.limit.map(|d| d.as_millis() as u64)
    }
}

/// Run `cmd` with the given budget. Always returns — never panics.
/// On timeout, kills the child and returns `FailureClass::Timeout`
/// (exit code 124, matching GNU `timeout` convention).
pub fn run_with_budget(mut cmd: Command, budget: HookBudget) -> std::io::Result<BudgetOutcome> {
    let start = Instant::now();
    let mut child = cmd.spawn()?;

    let deadline = budget.limit.map(|d| start + d);

    loop {
        match child.try_wait()? {
            Some(status) => {
                let elapsed_ms = start.elapsed().as_millis() as u64;
                return Ok(finish(status, elapsed_ms));
            }
            None => {
                if let Some(d) = deadline
                    && Instant::now() >= d
                {
                    let _ = child.kill();
                    let _ = child.wait();
                    let elapsed_ms = start.elapsed().as_millis() as u64;
                    return Ok(BudgetOutcome {
                        elapsed_ms,
                        exit_code: 124,
                        failure_class: FailureClass::Timeout,
                        timed_out: true,
                    });
                }
                thread::sleep(POLL_INTERVAL);
            }
        }
    }
}

fn finish(status: ExitStatus, elapsed_ms: u64) -> BudgetOutcome {
    let exit_code = status.code().unwrap_or({
        // On unix, signal-terminated children return None. Treat as 128+sig
        // to match shell convention — but keep generic for portability.
        if status.success() { 0 } else { 1 }
    });
    let failure_class = if exit_code == 0 {
        FailureClass::None
    } else {
        FailureClass::InnerNonzero
    };
    BudgetOutcome {
        elapsed_ms,
        exit_code,
        failure_class,
        timed_out: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sleep_cmd(ms: u64) -> Command {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(format!("sleep {}", ms as f64 / 1000.0));
        cmd
    }

    #[test]
    fn budget_passes_through_on_success() {
        let mut cmd = Command::new("true");
        let outcome =
            run_with_budget(std::mem::replace(&mut cmd, Command::new("true")), HookBudget::from_ms(Some(500))).unwrap();
        assert_eq!(outcome.exit_code, 0);
        assert_eq!(outcome.failure_class, FailureClass::None);
        assert!(!outcome.timed_out);
    }

    #[test]
    fn budget_wraps_command_and_respects_timeout() {
        let outcome = run_with_budget(sleep_cmd(500), HookBudget::from_ms(Some(200))).unwrap();
        assert!(outcome.timed_out);
        assert_eq!(outcome.exit_code, 124);
        assert_eq!(outcome.failure_class, FailureClass::Timeout);
        // Elapsed should be close to the budget, not the sleep.
        assert!(outcome.elapsed_ms < 400, "elapsed {} ms", outcome.elapsed_ms);
    }

    #[test]
    fn budget_none_means_no_timer() {
        let outcome = run_with_budget(sleep_cmd(50), HookBudget { limit: None }).unwrap();
        assert_eq!(outcome.exit_code, 0);
    }

    #[test]
    fn inner_nonzero_is_classified() {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg("exit 3");
        let outcome = run_with_budget(cmd, HookBudget::from_ms(Some(500))).unwrap();
        assert_eq!(outcome.exit_code, 3);
        assert_eq!(outcome.failure_class, FailureClass::InnerNonzero);
    }
}
