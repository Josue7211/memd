//! Cross-bucket dedupe — D4.3 stub.
//!
//! Filled in D4.3: canonicalize content, hash, drop lower-priority
//! duplicates, merge provenance lines into highest-priority survivor.

#![allow(dead_code)]

use super::buckets::{DedupedBuckets, OrderedBuckets};

pub fn merge(input: OrderedBuckets) -> DedupedBuckets {
    DedupedBuckets {
        buckets: input.buckets,
        merged: 0,
    }
}
