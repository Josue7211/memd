//! Budget + demotion — D4.4 stub.
//!
//! Filled in D4.4: hard cap on total chars, per-bucket floor preserved,
//! kinds-coverage caps (WC ≤25%, SC ≤20%, Canon ≤25%, Sem+Epi ≤20%,
//! Proc ≤10%), overflow emitted as DemotionHint.

#![allow(dead_code)]

use super::WakeBudget;
use super::buckets::{AdmittedBuckets, DedupedBuckets};

pub fn admit(input: DedupedBuckets, _budget: &WakeBudget) -> AdmittedBuckets {
    AdmittedBuckets {
        buckets: input.buckets,
        demoted: Vec::new(),
    }
}
