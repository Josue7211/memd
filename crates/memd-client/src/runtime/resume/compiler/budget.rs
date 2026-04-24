//! Budget admission + demotion — D4.4.
//!
//! Two enforcement layers, in order:
//!
//! 1. **Per-bucket entry floor.** `WakeBudget.per_bucket_floor` pins
//!    minimum entries per bucket. Floor wins over class cap and over
//!    the global hard cap — losing canonical truth to fit a budget is
//!    not a tradeoff this compiler ever makes.
//! 2. **Kinds-coverage class caps.** Each bucket maps to one of five
//!    kind classes (Working Context, Session Continuity, Canonical,
//!    Semantic+Episodic, Procedural) with a percentage of the total
//!    budget. Within a class, priority order from D4.2 still decides
//!    who fills the cap first.
//!
//! Records that miss admission are not deleted — they are surfaced as
//! `(BucketKind, count)` demotions for render to convert into
//! `memd lookup` hints.

use std::collections::HashMap;

use super::WakeBudget;
use super::buckets::{AdmittedBuckets, DedupedBuckets};
use super::{BucketKind, KindsCoverage};

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
enum BucketClass {
    WorkingContext,
    SessionContinuity,
    Canonical,
    SemanticEpisodic,
    Procedural,
}

fn class_of(kind: BucketKind) -> BucketClass {
    match kind {
        BucketKind::Focus => BucketClass::WorkingContext,
        BucketKind::Preference => BucketClass::SessionContinuity,
        BucketKind::Canonical | BucketKind::Correction => BucketClass::Canonical,
        BucketKind::Semantic | BucketKind::Episodic => BucketClass::SemanticEpisodic,
        BucketKind::Candidate => BucketClass::Procedural,
    }
}

fn class_caps(total: usize, kc: &KindsCoverage) -> HashMap<BucketClass, usize> {
    let total_f = total as f64;
    let mut caps = HashMap::new();
    caps.insert(
        BucketClass::WorkingContext,
        (total_f * kc.working_context_pct) as usize,
    );
    caps.insert(
        BucketClass::SessionContinuity,
        (total_f * kc.session_continuity_pct) as usize,
    );
    caps.insert(
        BucketClass::Canonical,
        (total_f * kc.canonical_pct) as usize,
    );
    caps.insert(
        BucketClass::SemanticEpisodic,
        (total_f * kc.semantic_episodic_pct) as usize,
    );
    caps.insert(
        BucketClass::Procedural,
        (total_f * kc.procedural_pct) as usize,
    );
    caps
}

pub fn admit(input: DedupedBuckets, budget: &WakeBudget) -> AdmittedBuckets {
    let class_caps = class_caps(budget.tokens, &budget.kinds_coverage);
    let mut class_used: HashMap<BucketClass, usize> = HashMap::new();
    let mut total_used: usize = 0;

    let mut output: Vec<(BucketKind, Vec<memd_schema::CompactMemoryRecord>)> =
        Vec::with_capacity(input.buckets.len());
    let mut demoted: Vec<(BucketKind, usize)> = Vec::new();

    for (kind, records) in input.buckets.into_iter() {
        let class = class_of(kind);
        let class_cap = *class_caps.get(&class).unwrap_or(&0);
        let floor = budget.per_bucket_floor.get(&kind).copied().unwrap_or(0);
        let forced = budget.force_include.contains(&kind);

        let mut admitted = Vec::with_capacity(records.len());
        let mut admitted_n: usize = 0;
        let mut overflow_n: usize = 0;

        for record in records {
            let cost = record.record.len();
            let class_used_now = *class_used.get(&class).unwrap_or(&0);
            let must_meet_floor = admitted_n < floor;
            let class_ok = class_used_now + cost <= class_cap;
            let total_ok = total_used + cost <= budget.tokens;

            if forced || must_meet_floor || (class_ok && total_ok) {
                *class_used.entry(class).or_insert(0) += cost;
                total_used += cost;
                admitted_n += 1;
                admitted.push(record);
            } else {
                overflow_n += 1;
            }
        }

        if overflow_n > 0 {
            demoted.push((kind, overflow_n));
        }
        output.push((kind, admitted));
    }

    AdmittedBuckets {
        buckets: output,
        demoted,
    }
}
