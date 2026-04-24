//! Hook runtime primitives for B4.
//!
//! Pure-Rust types that power the `memd hooks enforce` wrapper:
//!
//! - [`HookEvent`] — normative fire-order token (mirrors the table in
//!   `docs/contracts/hook-order.md §1`).
//! - [`HookRecord`] — NDJSON trace line shape (see contract §3).
//! - [`FailureClass`] — per-event halt vs log posture (contract §2).
//! - [`HookTrace`] — append-only NDJSON writer backed by `OpenOptions`.
//! - [`HookBudget`] — timeout wrapper around `std::process::Command`.
//! - [`FireOrderValidator`] — state machine that enforces contract §2
//!   predecessors.

pub mod budget;
pub mod trace;
pub mod validator;

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

pub use budget::{BudgetOutcome, HookBudget, run_with_budget};
pub use trace::{HookRecord, HookTrace};
pub use validator::{FireOrderValidator, ViolationKind};

/// Normative event tokens from `docs/contracts/hook-order.md §1`.
///
/// Only these strings are accepted by `memd hooks enforce` and
/// `memd hooks doctor --check contract`. Unknown tokens trigger the
/// `contract-parse` failure class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum HookEvent {
    SessionStart,
    UserPromptSubmit,
    PreRead,
    PreEdit,
    PreToolUse,
    PostToolUse,
    PreCompact,
    LedgerSeal,
    PostCompact,
    LedgerRestore,
    Stop,
    /// Sentinel appended when the trace file exceeds the 100 MiB cap.
    TruncationRequired,
}

impl HookEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            HookEvent::SessionStart => "SessionStart",
            HookEvent::UserPromptSubmit => "UserPromptSubmit",
            HookEvent::PreRead => "PreRead",
            HookEvent::PreEdit => "PreEdit",
            HookEvent::PreToolUse => "PreToolUse",
            HookEvent::PostToolUse => "PostToolUse",
            HookEvent::PreCompact => "PreCompact",
            HookEvent::LedgerSeal => "LedgerSeal",
            HookEvent::PostCompact => "PostCompact",
            HookEvent::LedgerRestore => "LedgerRestore",
            HookEvent::Stop => "Stop",
            HookEvent::TruncationRequired => "truncation-required",
        }
    }

    /// Contract §2 — halt-class events abort the turn on inner failure.
    pub fn failure_class_default(&self) -> FailureClass {
        match self {
            HookEvent::PreCompact | HookEvent::PostCompact | HookEvent::LedgerRestore => {
                FailureClass::Halt
            }
            _ => FailureClass::Log,
        }
    }

    /// Contract §2 column 2 — default budget in milliseconds.
    /// `None` means the inner command runs without an explicit budget
    /// (the caller's event absorbs the timer).
    pub fn default_budget_ms(&self) -> Option<u64> {
        match self {
            HookEvent::SessionStart => Some(2_000),
            HookEvent::UserPromptSubmit => Some(5_000),
            HookEvent::PreRead | HookEvent::PreEdit | HookEvent::PreToolUse => Some(500),
            HookEvent::PostToolUse => Some(500),
            HookEvent::PreCompact => Some(5_000),
            HookEvent::PostCompact => Some(2_000),
            HookEvent::Stop => Some(3_000),
            HookEvent::LedgerSeal
            | HookEvent::LedgerRestore
            | HookEvent::TruncationRequired => None,
        }
    }
}

impl fmt::Display for HookEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for HookEvent {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "SessionStart" => HookEvent::SessionStart,
            "UserPromptSubmit" => HookEvent::UserPromptSubmit,
            "PreRead" => HookEvent::PreRead,
            "PreEdit" => HookEvent::PreEdit,
            "PreToolUse" => HookEvent::PreToolUse,
            "PostToolUse" => HookEvent::PostToolUse,
            "PreCompact" => HookEvent::PreCompact,
            "LedgerSeal" => HookEvent::LedgerSeal,
            "PostCompact" => HookEvent::PostCompact,
            "LedgerRestore" => HookEvent::LedgerRestore,
            "Stop" => HookEvent::Stop,
            "truncation-required" => HookEvent::TruncationRequired,
            _ => return Err(()),
        })
    }
}

/// Contract §3 — failure-class taxonomy written into trace lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FailureClass {
    /// Exit 0, within budget, order ok.
    None,
    /// Inner exceeded `budget_ms` — SIGTERM sent, exit 124.
    Timeout,
    /// Inner exited non-zero within budget.
    InnerNonzero,
    /// Inner emitted malformed output where schema was expected.
    BadOutput,
    /// Predecessor of contract §2 column 4 missing in observed sequence.
    OrderViolation,
    /// Manifest or contract file invalid.
    ContractParse,
    /// Halt-class decision posture — used by budget/order wrappers before
    /// converting into the concrete failure value above.
    #[serde(rename = "halt")]
    Halt,
    /// Log-class decision posture — wrapper returns 0 despite failure.
    #[serde(rename = "log")]
    Log,
}

impl FailureClass {
    /// Is this a decision posture (halt/log) rather than an observed outcome?
    pub fn is_posture(&self) -> bool {
        matches!(self, FailureClass::Halt | FailureClass::Log)
    }
}

#[cfg(test)]
mod event_tests {
    use super::*;

    #[test]
    fn parses_every_contract_token() {
        for tok in [
            "SessionStart",
            "UserPromptSubmit",
            "PreRead",
            "PreEdit",
            "PreToolUse",
            "PostToolUse",
            "PreCompact",
            "LedgerSeal",
            "PostCompact",
            "LedgerRestore",
            "Stop",
            "truncation-required",
        ] {
            let parsed: HookEvent = tok.parse().expect("contract token parses");
            assert_eq!(parsed.as_str(), tok);
        }
    }

    #[test]
    fn unknown_token_rejected() {
        assert!("NotAnEvent".parse::<HookEvent>().is_err());
    }

    #[test]
    fn failure_class_defaults_track_contract() {
        assert_eq!(
            HookEvent::PreCompact.failure_class_default(),
            FailureClass::Halt
        );
        assert_eq!(
            HookEvent::PostCompact.failure_class_default(),
            FailureClass::Halt
        );
        assert_eq!(
            HookEvent::LedgerRestore.failure_class_default(),
            FailureClass::Halt
        );
        assert_eq!(HookEvent::Stop.failure_class_default(), FailureClass::Log);
        assert_eq!(
            HookEvent::PreRead.failure_class_default(),
            FailureClass::Log
        );
    }

    #[test]
    fn budgets_match_contract_table() {
        assert_eq!(HookEvent::PreCompact.default_budget_ms(), Some(5_000));
        assert_eq!(HookEvent::PostCompact.default_budget_ms(), Some(2_000));
        assert_eq!(HookEvent::PreRead.default_budget_ms(), Some(500));
        assert_eq!(HookEvent::LedgerSeal.default_budget_ms(), None);
    }
}
