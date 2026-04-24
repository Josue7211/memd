//! Markdown emitter — D4.5 stub.
//!
//! Filled in D4.5: section header per bucket in priority order, demotion
//! hint section appended when overflow non-empty, returned token count
//! must round-trip with `compute_wake_token_metrics`.

#![allow(dead_code)]

use std::collections::HashMap;

use super::buckets::AdmittedBuckets;
use super::{BucketKind, BucketReport, CompiledWake, WakeBudget};

pub fn emit(_admitted: AdmittedBuckets, _budget: &WakeBudget) -> CompiledWake {
    CompiledWake {
        markdown: String::new(),
        tokens: 0,
        bucket_report: HashMap::<BucketKind, BucketReport>::new(),
        demotion_hints: Vec::new(),
    }
}
