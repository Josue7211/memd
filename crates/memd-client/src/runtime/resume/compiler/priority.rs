//! Priority rules — D4.2.
//!
//! Order: Canonical > Preference > Focus > Correction > Episodic >
//! Semantic > Candidate.
//!
//! Rationale (phase-doc §2):
//! - Canonical durable truth must never be displaced.
//! - Preferences gate behavior — second-tier durability.
//! - Focus is the live "what am I doing" stub.
//! - Corrections trump plain Semantic facts on equal topic; the
//!   correction bucket appears earlier so render emits it before the
//!   stale-fact bucket and dedupe (D4.3) collapses overlaps onto the
//!   higher-priority survivor.
//! - Episodic > Semantic: the moment of learning beats the disembodied
//!   fact when both compete for budget.
//! - Candidates are the speculative tier — last admitted, first demoted.

use super::buckets::OrderedBuckets;
use super::{BucketKind, CompilerInput};

/// Canonical priority order. `BucketKind::ALL` is intentionally the
/// single source of truth: changing one changes the other.
pub const PRIORITY_ORDER: [BucketKind; 7] = BucketKind::ALL;

pub fn apply(input: &CompilerInput) -> OrderedBuckets {
    let buckets = PRIORITY_ORDER
        .iter()
        .map(|&kind| (kind, input.bucket(kind).clone()))
        .collect();
    OrderedBuckets { buckets }
}
