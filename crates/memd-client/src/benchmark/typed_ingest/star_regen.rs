//! V6 / F6 — MEMD-10-STAR composite regenerator (scaffold-symmetric).
//!
//! Pure: takes a 7-axis score table, returns the weighted composite
//! and the verdict (publishable / refused / overridden). The gate
//! refuses to publish a 10-STAR claim below 7.0 unless
//! `MEMD_V6_ALLOW_BELOW_TARGET=1` (mirrors `--allow-below-target`).
//!
//! V6 milestone target is 4.45 (per `MILESTONE-v6.md`); the 10-STAR
//! gate is the *publishable claim* threshold, not the V6 milestone
//! threshold. They differ by design: V6 lifts the composite without
//! claiming the 10-star bar.
//!
//! Plan: `docs/phases/v6/phase-f6-plan.md` §1, §7.

pub(crate) const STAR_REGEN_VERSION: &str = "memd-10-star/v1";
pub(crate) const PUBLISH_THRESHOLD: f64 = 7.0;

/// Per-axis score with its weight (sums to 1.00 across the slice).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AxisScore {
    pub axis: &'static str,
    pub weight: f64,
    pub score: f64,
}

/// Verdict returned by the regen call. Tests assert on `Refused`
/// vs `Published` directly so the gate logic stays observable.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum StarVerdict {
    /// Composite ≥ 7.0 — publish.
    Published { composite: f64 },
    /// Composite < 7.0 and override not set — refuse.
    Refused { composite: f64 },
    /// Composite < 7.0 but override (`--allow-below-target` /
    /// `MEMD_V6_ALLOW_BELOW_TARGET=1`) was set — publish anyway.
    OverriddenBelowTarget { composite: f64 },
}

impl StarVerdict {
    pub(crate) fn composite(&self) -> f64 {
        match self {
            StarVerdict::Published { composite }
            | StarVerdict::Refused { composite }
            | StarVerdict::OverriddenBelowTarget { composite } => *composite,
        }
    }
}

/// Compute weighted composite. Pure.
pub(crate) fn composite(scores: &[AxisScore]) -> f64 {
    scores.iter().map(|a| a.weight * a.score).sum()
}

/// Apply the 7.0 gate. `allow_below` mirrors the CLI flag + env.
pub(crate) fn evaluate(scores: &[AxisScore], allow_below: bool) -> StarVerdict {
    let c = composite(scores);
    if c >= PUBLISH_THRESHOLD {
        StarVerdict::Published { composite: c }
    } else if allow_below {
        StarVerdict::OverriddenBelowTarget { composite: c }
    } else {
        StarVerdict::Refused { composite: c }
    }
}

/// Resolution of the `--allow-below-target` CLI flag and the env
/// override `MEMD_V6_ALLOW_BELOW_TARGET=1`. Env wins.
pub(crate) fn allow_below_target_active(cli_flag: bool) -> bool {
    if std::env::var("MEMD_V6_ALLOW_BELOW_TARGET").ok().as_deref() == Some("1") {
        return true;
    }
    cli_flag
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at_or_above() -> Vec<AxisScore> {
        vec![
            AxisScore {
                axis: "session_continuity",
                weight: 0.20,
                score: 7.0,
            },
            AxisScore {
                axis: "correction_retention",
                weight: 0.15,
                score: 7.0,
            },
            AxisScore {
                axis: "procedural_reuse",
                weight: 0.15,
                score: 7.0,
            },
            AxisScore {
                axis: "cross_harness",
                weight: 0.15,
                score: 7.0,
            },
            AxisScore {
                axis: "raw_retrieval",
                weight: 0.15,
                score: 7.0,
            },
            AxisScore {
                axis: "token_efficiency",
                weight: 0.10,
                score: 7.0,
            },
            AxisScore {
                axis: "trust_provenance",
                weight: 0.10,
                score: 7.0,
            },
        ]
    }

    #[test]
    fn composite_weighted_sum() {
        let c = composite(&at_or_above());
        assert!((c - 7.0).abs() < 1e-9, "composite={c}");
    }

    #[test]
    fn gate_refuses_below_threshold() {
        let mut s = at_or_above();
        for a in &mut s {
            a.score = 4.0;
        }
        let v = evaluate(&s, false);
        assert!(matches!(v, StarVerdict::Refused { .. }));
    }

    #[test]
    fn gate_publishes_at_threshold() {
        let v = evaluate(&at_or_above(), false);
        assert!(matches!(v, StarVerdict::Published { .. }));
    }

    #[test]
    fn override_publishes_below_threshold() {
        let mut s = at_or_above();
        for a in &mut s {
            a.score = 4.0;
        }
        let v = evaluate(&s, true);
        assert!(matches!(v, StarVerdict::OverriddenBelowTarget { .. }));
    }
}
