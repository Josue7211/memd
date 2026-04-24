//! Priority rules — D4.2.
//!
//! Order: Canonical > Preference > Focus > Correction > Episodic >
//! Semantic > Candidate. Corrections sort BEFORE plain Semantic facts
//! per phase-doc §2 ("corrections trump plain facts on equal topic").

#![allow(dead_code)]

use super::buckets::OrderedBuckets;
use super::{BucketKind, CompilerInput};

pub fn apply(input: &CompilerInput) -> OrderedBuckets {
    let mut buckets = Vec::with_capacity(BucketKind::ALL.len());
    for kind in BucketKind::ALL {
        let records = input.bucket(kind).clone();
        buckets.push((kind, records));
    }
    OrderedBuckets { buckets }
}
