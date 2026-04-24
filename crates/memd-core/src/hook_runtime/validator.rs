//! Fire-order state machine — contract `docs/contracts/hook-order.md §2`.
//!
//! Tracks which events have been observed in a session. Each call to
//! [`FireOrderValidator::observe`] checks the incoming event against the
//! required-predecessor set from contract §2 column 4. Missing a
//! required predecessor produces [`ViolationKind::MissingPredecessor`].
//! Gaps are permitted (e.g. a turn that never compacts never emits
//! `PreCompact`).

use super::HookEvent;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViolationKind {
    /// Event fired without its required predecessor (contract §2 col 4).
    MissingPredecessor {
        event: HookEvent,
        required_any_of: Vec<HookEvent>,
    },
    /// PostCompact fired before PreCompact within the same session —
    /// the canonical halt-class swap described in contract §1.
    OrderSwap {
        event: HookEvent,
        before: HookEvent,
    },
}

#[derive(Debug, Default, Clone)]
pub struct FireOrderValidator {
    observed: HashSet<HookEvent>,
}

impl FireOrderValidator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an event and validate predecessors. Returns `Ok` on clean
    /// observation, `Err(ViolationKind)` on contract breach. The event
    /// is still recorded on error so downstream calls see the actual
    /// observed sequence.
    pub fn observe(&mut self, event: HookEvent) -> Result<(), ViolationKind> {
        let result = self.check(event);
        self.observed.insert(event);
        result
    }

    fn check(&self, event: HookEvent) -> Result<(), ViolationKind> {
        use HookEvent::*;

        // Canonical swap — PostCompact before PreCompact.
        if matches!(event, PostCompact) && !self.observed.contains(&PreCompact) {
            return Err(ViolationKind::OrderSwap {
                event: PostCompact,
                before: PreCompact,
            });
        }

        let predecessors: &[HookEvent] = match event {
            SessionStart => &[],
            UserPromptSubmit | PreRead | PreEdit | PreToolUse | PreCompact | Stop => {
                &[SessionStart]
            }
            PostToolUse => &[PreEdit, PreRead, PreToolUse],
            LedgerSeal => &[PreCompact],
            PostCompact => &[PreCompact],
            LedgerRestore => &[PostCompact],
            TruncationRequired => &[],
        };

        if predecessors.is_empty() {
            return Ok(());
        }
        if predecessors.iter().any(|p| self.observed.contains(p)) {
            return Ok(());
        }
        Err(ViolationKind::MissingPredecessor {
            event,
            required_any_of: predecessors.to_vec(),
        })
    }

    pub fn observed(&self) -> &HashSet<HookEvent> {
        &self.observed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_order_validator_accepts_canonical_sequence() {
        let mut v = FireOrderValidator::new();
        let sequence = [
            HookEvent::SessionStart,
            HookEvent::UserPromptSubmit,
            HookEvent::PreEdit,
            HookEvent::PostToolUse,
            HookEvent::PreCompact,
            HookEvent::LedgerSeal,
            HookEvent::PostCompact,
            HookEvent::LedgerRestore,
            HookEvent::Stop,
        ];
        for ev in sequence {
            v.observe(ev).expect("canonical sequence accepted");
        }
    }

    #[test]
    fn fire_order_validator_flags_swap() {
        let mut v = FireOrderValidator::new();
        v.observe(HookEvent::SessionStart).unwrap();
        let err = v
            .observe(HookEvent::PostCompact)
            .expect_err("PostCompact before PreCompact is a swap");
        assert!(matches!(err, ViolationKind::OrderSwap { .. }));
    }

    #[test]
    fn fire_order_validator_permits_gaps() {
        let mut v = FireOrderValidator::new();
        v.observe(HookEvent::SessionStart).unwrap();
        // Skip UserPromptSubmit + read/edit hooks entirely — permitted.
        v.observe(HookEvent::Stop)
            .expect("Stop after SessionStart with no gaps filled");
    }

    #[test]
    fn post_tool_use_requires_a_pre_probe() {
        let mut v = FireOrderValidator::new();
        v.observe(HookEvent::SessionStart).unwrap();
        let err = v.observe(HookEvent::PostToolUse).expect_err("no pre probe");
        assert!(matches!(
            err,
            ViolationKind::MissingPredecessor { .. }
        ));
    }

    #[test]
    fn ledger_restore_requires_postcompact() {
        let mut v = FireOrderValidator::new();
        v.observe(HookEvent::SessionStart).unwrap();
        v.observe(HookEvent::PreCompact).unwrap();
        let err = v.observe(HookEvent::LedgerRestore).expect_err("no postcompact");
        assert!(matches!(err, ViolationKind::MissingPredecessor { .. }));
    }
}
