//! Unit tests for the wake-context compiler.
//!
//! D4.2 lands tests 1–3 (priority).
//! D4.3 lands tests 4–5 (dedupe).
//! D4.4 lands tests 6–8 + 12 (budget + token counter parity).
//! D4.5 lands tests 9–11 (render).

#![allow(dead_code)]
#![allow(unused_imports)]

use memd_schema::CompactMemoryRecord;
use uuid::Uuid;

use super::*;

fn rec(s: &str) -> CompactMemoryRecord {
    CompactMemoryRecord {
        id: Uuid::new_v4(),
        record: s.to_string(),
    }
}

fn bucket_of(ordered: &buckets::OrderedBuckets, kind: BucketKind) -> &Vec<CompactMemoryRecord> {
    &ordered
        .buckets
        .iter()
        .find(|(k, _)| *k == kind)
        .expect("bucket present")
        .1
}

fn position_of(ordered: &buckets::OrderedBuckets, kind: BucketKind) -> usize {
    ordered
        .buckets
        .iter()
        .position(|(k, _)| *k == kind)
        .expect("bucket present")
}

fn dedup_position(deduped: &buckets::DedupedBuckets, kind: BucketKind) -> &Vec<CompactMemoryRecord> {
    &deduped
        .buckets
        .iter()
        .find(|(k, _)| *k == kind)
        .expect("bucket present")
        .1
}

// -------- D4.2: priority ----------

#[test]
fn priority_order_canonical_first() {
    let input = CompilerInput {
        canonical: vec![rec("kind=fact")],
        preferences: vec![rec("kind=preference")],
        focus: vec![rec("kind=live_truth")],
        ..Default::default()
    };
    let ordered = priority::apply(&input);
    assert_eq!(
        ordered.buckets.first().map(|(k, _)| *k),
        Some(BucketKind::Canonical),
        "canonical must be first priority bucket"
    );
}

#[test]
fn priority_order_preferences_after_canonical_before_focus() {
    let input = CompilerInput {
        canonical: vec![rec("c")],
        preferences: vec![rec("p")],
        focus: vec![rec("f")],
        ..Default::default()
    };
    let ordered = priority::apply(&input);
    let canon = position_of(&ordered, BucketKind::Canonical);
    let pref = position_of(&ordered, BucketKind::Preference);
    let focus = position_of(&ordered, BucketKind::Focus);
    assert!(canon < pref, "canonical before preference");
    assert!(pref < focus, "preference before focus");
}

#[test]
fn priority_order_corrections_before_semantic() {
    let input = CompilerInput {
        semantic: vec![rec("kind=fact | c=user prefers tabs")],
        corrections: vec![rec("kind=correction | c=user prefers spaces (overrides earlier)")],
        ..Default::default()
    };
    let ordered = priority::apply(&input);
    let corr = position_of(&ordered, BucketKind::Correction);
    let sem = position_of(&ordered, BucketKind::Semantic);
    assert!(
        corr < sem,
        "corrections must outrank plain semantic facts on equal topic"
    );
    // Records present in their respective buckets.
    assert_eq!(bucket_of(&ordered, BucketKind::Correction).len(), 1);
    assert_eq!(bucket_of(&ordered, BucketKind::Semantic).len(), 1);
}

// -------- D4.3: dedupe ----------

#[test]
fn dedupe_merges_same_content_across_buckets_with_provenance() {
    // Same payload `c=` in canonical and semantic → keep canonical, annotate
    // with provenance pointer to absorbed bucket.
    let payload = "c=user prefers spaces";
    let input = CompilerInput {
        canonical: vec![rec(&format!("id=A | kind=fact | {payload}"))],
        semantic: vec![rec(&format!("id=B | kind=fact | {payload}"))],
        ..Default::default()
    };
    let ordered = priority::apply(&input);
    let deduped = dedupe::merge(ordered);

    assert_eq!(deduped.merged, 1, "exactly one cross-bucket merge");
    let canon_recs = dedup_position(&deduped, BucketKind::Canonical);
    let sem_recs = dedup_position(&deduped, BucketKind::Semantic);
    assert_eq!(canon_recs.len(), 1, "canonical keeps the survivor");
    assert_eq!(sem_recs.len(), 0, "lower-priority duplicate dropped");

    let survivor = &canon_recs[0].record;
    assert!(
        survivor.contains("also_in="),
        "provenance annotation missing in survivor: {survivor}"
    );
    assert!(
        survivor.contains("semantic"),
        "absorbed bucket not listed in provenance: {survivor}"
    );
}

#[test]
fn dedupe_preserves_highest_priority_source() {
    let payload = "c=user prefers tabs";
    let input = CompilerInput {
        canonical: vec![rec(&format!("id=CANON | kind=fact | {payload}"))],
        episodic: vec![rec(&format!("id=EPI | kind=episode | {payload}"))],
        semantic: vec![rec(&format!("id=SEM | kind=fact | {payload}"))],
        ..Default::default()
    };
    let ordered = priority::apply(&input);
    let deduped = dedupe::merge(ordered);

    assert_eq!(deduped.merged, 2, "two lower-priority duplicates absorbed");
    let canon_recs = dedup_position(&deduped, BucketKind::Canonical);
    assert_eq!(canon_recs.len(), 1);
    assert!(
        canon_recs[0].record.contains("id=CANON"),
        "canonical (highest priority) record must survive"
    );
    assert_eq!(dedup_position(&deduped, BucketKind::Episodic).len(), 0);
    assert_eq!(dedup_position(&deduped, BucketKind::Semantic).len(), 0);
}

#[test]
fn dedupe_preserves_distinct_content() {
    // Records with different payloads do NOT dedupe.
    let input = CompilerInput {
        canonical: vec![rec("id=A | c=fact one")],
        semantic: vec![rec("id=B | c=fact two")],
        ..Default::default()
    };
    let ordered = priority::apply(&input);
    let deduped = dedupe::merge(ordered);
    assert_eq!(deduped.merged, 0);
    assert_eq!(dedup_position(&deduped, BucketKind::Canonical).len(), 1);
    assert_eq!(dedup_position(&deduped, BucketKind::Semantic).len(), 1);
}
