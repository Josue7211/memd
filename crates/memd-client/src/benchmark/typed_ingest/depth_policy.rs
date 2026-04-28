//! V6 / E6 — escalation policy.
//!
//! Pure decision functions that decide whether the next depth tier
//! should be issued, given the prior call result. Two escalation
//! signals: empty wake result and low-confidence answer. Both are
//! observable on the bench side without requiring model self-report.
//!
//! Contract: `docs/contracts/bench-depth-routing.md` §2.

/// Confidence floor below which the answer is considered unreliable
/// and an escalation to `targeted` (or `resume`) is triggered.
/// Calibrated against LoCoMo multi-hop fixtures; tweakable via the
/// `MEMD_V6_DEPTH_CONFIDENCE_FLOOR` env override (read by the
/// runtime, not by the pure helper).
pub(crate) const DEFAULT_LOW_CONFIDENCE_FLOOR: f64 = 0.6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NextDepth {
    /// Stay on the existing answer.
    Stop,
    /// Re-issue at `targeted` depth.
    Targeted,
    /// Re-issue at `resume` (full-depth) tier.
    Resume,
}

/// Escalation 1: prior `wake` lookup returned no rows. Bench routes
/// to `targeted` (next depth tier) — V4 E4 surfaces this directly.
pub(crate) fn escalate_on_empty_wake(prior_depth: &str, prior_hits: usize) -> NextDepth {
    if prior_depth == "wake" && prior_hits == 0 {
        NextDepth::Targeted
    } else {
        NextDepth::Stop
    }
}

/// Escalation 2: model answer carries confidence < floor. Bench
/// re-issues at `resume` tier so the answer can re-ground against
/// the long-form record set.
pub(crate) fn escalate_on_low_confidence(
    answer_confidence: f64,
    floor: f64,
) -> NextDepth {
    if answer_confidence < floor {
        NextDepth::Resume
    } else {
        NextDepth::Stop
    }
}

/// Combined: empty-wake takes precedence, then low-confidence. This
/// matches the runtime evaluation order so tests can assert by
/// running either helper on its own and the combined function.
pub(crate) fn next_depth(
    prior_depth: &str,
    prior_hits: usize,
    answer_confidence: f64,
    floor: f64,
) -> NextDepth {
    let empty = escalate_on_empty_wake(prior_depth, prior_hits);
    if empty != NextDepth::Stop {
        return empty;
    }
    escalate_on_low_confidence(answer_confidence, floor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_wake_escalates_to_targeted() {
        assert_eq!(escalate_on_empty_wake("wake", 0), NextDepth::Targeted);
        assert_eq!(escalate_on_empty_wake("wake", 3), NextDepth::Stop);
        assert_eq!(escalate_on_empty_wake("targeted", 0), NextDepth::Stop);
    }

    #[test]
    fn low_confidence_escalates_to_resume() {
        assert_eq!(
            escalate_on_low_confidence(0.5, DEFAULT_LOW_CONFIDENCE_FLOOR),
            NextDepth::Resume
        );
        assert_eq!(
            escalate_on_low_confidence(0.9, DEFAULT_LOW_CONFIDENCE_FLOOR),
            NextDepth::Stop
        );
    }
}
