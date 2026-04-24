//! Cross-bucket dedupe — D4.3.
//!
//! Two records are duplicates when their canonicalized content payload
//! matches. The highest-priority bucket (per D4.2 ordering) wins; lower
//! priority duplicates are dropped from their bucket. The survivor is
//! annotated with `| also_in=<bucket1>,<bucket2>` so render can show
//! provenance without reopening the source records.
//!
//! Canonicalization extracts the `c=` payload (whitespace-collapsed)
//! and falls back to `id=` for records without explicit content. Records
//! with neither field hash on the trimmed full record string.

use std::collections::HashMap;

use memd_schema::CompactMemoryRecord;

use super::BucketKind;
use super::buckets::{DedupedBuckets, OrderedBuckets};

pub fn merge(input: OrderedBuckets) -> DedupedBuckets {
    let mut first_seen: HashMap<String, BucketKind> = HashMap::new();
    let mut absorbed_by: HashMap<String, Vec<BucketKind>> = HashMap::new();
    let mut merged_count: usize = 0;

    // First pass: walk in priority order, record the bucket of each unique key
    // and which lower-priority buckets contributed an absorbed copy.
    for (kind, records) in &input.buckets {
        for record in records {
            let key = canonical_key(&record.record);
            match first_seen.get(&key) {
                None => {
                    first_seen.insert(key, *kind);
                }
                Some(&survivor_kind) if survivor_kind != *kind => {
                    absorbed_by.entry(key).or_default().push(*kind);
                    merged_count += 1;
                }
                _ => {
                    // Same bucket repeating itself — let it through; intra-bucket
                    // dedupe is the source's responsibility.
                }
            }
        }
    }

    // Second pass: emit each bucket, dropping lower-priority duplicates and
    // annotating the survivor with provenance.
    let mut emitted_survivor: HashMap<String, ()> = HashMap::new();
    let mut output = Vec::with_capacity(input.buckets.len());

    for (kind, records) in input.buckets {
        let mut kept = Vec::with_capacity(records.len());
        for record in records {
            let key = canonical_key(&record.record);
            let winner = first_seen.get(&key).copied();
            match winner {
                Some(winner_kind) if winner_kind != kind => {
                    // lower-priority dupe — drop
                }
                _ => {
                    if !emitted_survivor.contains_key(&key) {
                        emitted_survivor.insert(key.clone(), ());
                        let absorbed = absorbed_by.get(&key);
                        if let Some(buckets) = absorbed {
                            kept.push(annotate_provenance(record, buckets));
                        } else {
                            kept.push(record);
                        }
                    } else {
                        // Same bucket repeating — keep without re-annotating.
                        kept.push(record);
                    }
                }
            }
        }
        output.push((kind, kept));
    }

    DedupedBuckets {
        buckets: output,
        merged: merged_count,
    }
}

fn annotate_provenance(
    record: CompactMemoryRecord,
    absorbed: &[BucketKind],
) -> CompactMemoryRecord {
    let mut labels: Vec<&str> = absorbed.iter().map(|k| k.label()).collect();
    labels.sort_unstable();
    labels.dedup();
    let CompactMemoryRecord { id, record } = record;
    let new_record = format!("{record} | also_in={}", labels.join(","));
    CompactMemoryRecord {
        id,
        record: new_record,
    }
}

fn canonical_key(record: &str) -> String {
    if let Some(payload) = extract_field(record, "c=") {
        return collapse_whitespace(&payload);
    }
    if let Some(id) = extract_field(record, "id=") {
        return id.trim().to_string();
    }
    collapse_whitespace(record)
}

fn extract_field(record: &str, key: &str) -> Option<String> {
    let idx = record.find(key)?;
    let after = &record[idx + key.len()..];
    Some(after.to_string())
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
