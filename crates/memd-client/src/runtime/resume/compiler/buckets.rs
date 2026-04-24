//! Typed bucket views consumed by the priority/dedupe/budget pipeline.
//!
//! Filled in D4.2 (priority) — D4.1 only defines the carrier types so
//! the pipeline signatures compile.

#![allow(dead_code)]

use memd_schema::CompactMemoryRecord;

use super::BucketKind;

#[derive(Debug, Clone)]
pub struct OrderedBuckets {
    /// `(kind, records)` in priority order. First entry = highest priority.
    pub buckets: Vec<(BucketKind, Vec<CompactMemoryRecord>)>,
}

#[derive(Debug, Clone)]
pub struct DedupedBuckets {
    pub buckets: Vec<(BucketKind, Vec<CompactMemoryRecord>)>,
    pub merged: usize,
}

#[derive(Debug, Clone)]
pub struct AdmittedBuckets {
    pub buckets: Vec<(BucketKind, Vec<CompactMemoryRecord>)>,
    pub demoted: Vec<(BucketKind, usize)>,
}
