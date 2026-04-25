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

// -------- D4.4: budget + demotion ----------

fn admitted_count(admitted: &buckets::AdmittedBuckets, kind: BucketKind) -> usize {
    admitted
        .buckets
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, recs)| recs.len())
        .unwrap_or(0)
}

fn demoted_count(admitted: &buckets::AdmittedBuckets, kind: BucketKind) -> usize {
    admitted
        .demoted
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, n)| *n)
        .unwrap_or(0)
}

fn admitted_total_chars(admitted: &buckets::AdmittedBuckets) -> usize {
    admitted
        .buckets
        .iter()
        .flat_map(|(_, recs)| recs.iter())
        .map(|r| r.record.len())
        .sum()
}

#[test]
fn budget_enforces_hard_cap() {
    // 50 semantic records × 100 chars = 5000; budget 2000 should cap below 2000.
    // Class cap (semantic_episodic = 20%) = 400 chars, so we expect ≤4 records.
    let recs: Vec<_> = (0..50)
        .map(|i| rec(&format!("id={i:02} | c={}", "x".repeat(80))))
        .collect();
    let input = CompilerInput {
        semantic: recs,
        ..Default::default()
    };
    let budget = WakeBudget::default_2000();
    let admitted = budget::admit(dedupe::merge(priority::apply(&input)), &budget);

    let total = admitted_total_chars(&admitted);
    assert!(
        total <= budget.tokens,
        "admitted chars {total} must not exceed budget {}",
        budget.tokens
    );
}

#[test]
fn budget_respects_per_bucket_floor() {
    // 4 canonical records each ~600 chars = 2400 total.
    // Class cap (canonical = 25% of 2000) = 500. Without floor, only ~0 fit.
    // Floor = 4; all four must survive even though total > budget.
    let recs: Vec<_> = (0..4)
        .map(|i| rec(&format!("id={i} | c={}", "y".repeat(580))))
        .collect();
    let input = CompilerInput {
        canonical: recs,
        ..Default::default()
    };
    let budget = WakeBudget::default_2000();
    let admitted = budget::admit(dedupe::merge(priority::apply(&input)), &budget);

    assert_eq!(
        admitted_count(&admitted, BucketKind::Canonical),
        4,
        "canonical floor=4 must override class cap"
    );
}

#[test]
fn budget_demotes_overflow_to_lookup_hint() {
    // 20 semantic records, budget 1000 → class cap 200. Each ~100 chars.
    // ~1-2 admitted, 18+ demoted.
    let recs: Vec<_> = (0..20)
        .map(|i| rec(&format!("id={i:02} | c={}", "z".repeat(95))))
        .collect();
    let input = CompilerInput {
        semantic: recs,
        ..Default::default()
    };
    let mut budget = WakeBudget::default_2000();
    budget.tokens = 1000;
    let admitted = budget::admit(dedupe::merge(priority::apply(&input)), &budget);

    let demoted = demoted_count(&admitted, BucketKind::Semantic);
    let admitted_n = admitted_count(&admitted, BucketKind::Semantic);
    assert_eq!(admitted_n + demoted, 20);
    assert!(
        demoted >= 17,
        "expected most semantic records demoted; got admitted={admitted_n} demoted={demoted}"
    );
}

#[test]
fn token_counter_matches_compute_wake_token_metrics() {
    // The compiler tracks admitted cost by summing record string lengths
    // — same arithmetic compute_wake_token_metrics applies to its inputs.
    let recs = vec![
        rec("id=1 | c=alpha"),
        rec("id=2 | c=beta"),
        rec("id=3 | c=gamma"),
    ];
    let expected: usize = recs.iter().map(|r| r.record.len()).sum();
    let input = CompilerInput {
        canonical: recs,
        ..Default::default()
    };
    let budget = WakeBudget::default_2000();
    let admitted = budget::admit(dedupe::merge(priority::apply(&input)), &budget);
    assert_eq!(admitted_total_chars(&admitted), expected);
}

// -------- D4.6: CLI surface (overrides + ledger) ----------

#[test]
fn budget_override_via_with_tokens_respected() {
    let budget = WakeBudget::default_2000().with_tokens(500);
    assert_eq!(budget.tokens, 500);

    // 0 means "leave default" (CLI default_value_t).
    let unchanged = WakeBudget::default_2000().with_tokens(0);
    assert_eq!(unchanged.tokens, 2000);
}

#[test]
fn include_bucket_forces_inclusion_even_over_budget() {
    // 4 semantic records, each 1500 chars → would normally be capped at
    // ~400 chars (20% of 2000). With force-include, all 4 land.
    let recs: Vec<_> = (0..4)
        .map(|i| rec(&format!("id={i} | c={}", "s".repeat(1500))))
        .collect();
    let input = CompilerInput {
        semantic: recs,
        ..Default::default()
    };
    let budget = WakeBudget::default_2000().with_includes(&["semantic".to_string()]);
    let admitted = budget::admit(dedupe::merge(priority::apply(&input)), &budget);
    assert_eq!(admitted_count(&admitted, BucketKind::Semantic), 4);
}

#[test]
fn exclude_bucket_drops_records_entirely() {
    let input = CompilerInput {
        canonical: vec![rec("id=A | c=keep")],
        semantic: vec![rec("id=B | c=drop")],
        ..Default::default()
    };
    let budget = WakeBudget::default_2000().with_excludes(&["semantic".to_string()]);
    let compiled = compile_wake(input, budget);
    assert!(compiled.markdown.contains("Durable Truth"));
    assert!(!compiled.markdown.contains("## Semantic"));
}

#[test]
fn parse_bucket_label_is_case_insensitive_and_pluralizes() {
    assert_eq!(parse_bucket_label("Canonical"), Some(BucketKind::Canonical));
    assert_eq!(parse_bucket_label("PREFERENCES"), Some(BucketKind::Preference));
    assert_eq!(parse_bucket_label("corrections"), Some(BucketKind::Correction));
    assert_eq!(parse_bucket_label("nonsense"), None);
}

#[test]
fn ledger_writes_wake_budget_ndjson_line() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let bundle = tempdir.path();

    let input = CompilerInput {
        canonical: vec![rec("id=A | c=durable")],
        semantic: (0..30)
            .map(|i| rec(&format!("id={i:02} | c={}", "x".repeat(95))))
            .collect(),
        ..Default::default()
    };
    let mut budget = WakeBudget::default_2000();
    budget.tokens = 800;
    let compiled = compile_wake(input, budget);

    ledger::write_budget_line(bundle, Some("session-x"), 9999, &compiled)
        .expect("budget line written");
    ledger::write_cost_line(bundle, Some("session-x"), compiled.tokens, 800, "claude-opus-4-7")
        .expect("cost line written");

    let budget_path = bundle.join("logs/wake-budget.ndjson");
    let cost_path = bundle.join("logs/wake-cost.ndjson");
    let budget_body = std::fs::read_to_string(&budget_path).expect("budget file");
    let cost_body = std::fs::read_to_string(&cost_path).expect("cost file");
    assert!(budget_body.contains("session-x"));
    assert!(budget_body.contains("compiled_tokens"));
    assert!(budget_body.contains("bucket_fill_ratio"));
    assert!(budget_body.contains("\"demoted\""));
    assert!(cost_body.contains("\"wake_token_count\""));
    assert!(cost_body.contains("\"estimated_cost_usd\""));
    assert!(cost_body.contains("claude-opus-4-7"));
}

// -------- D4.5: render ----------

#[test]
fn render_emits_section_headers_in_priority_order() {
    let input = CompilerInput {
        canonical: vec![rec("id=A | c=durable")],
        preferences: vec![rec("id=B | c=pref")],
        focus: vec![rec("id=C | c=doing")],
        corrections: vec![rec("id=D | c=corrected")],
        episodic: vec![rec("id=E | c=session-x")],
        semantic: vec![rec("id=F | c=fact")],
        candidates: vec![rec("id=G | c=maybe")],
        drift_notes: Vec::new(),
    };
    let compiled = compile_wake(input, WakeBudget::default_2000());

    let canon_idx = compiled.markdown.find("## Durable Truth").expect("canonical header");
    let pref_idx = compiled.markdown.find("## Preferences").expect("pref header");
    let focus_idx = compiled.markdown.find("## Focus").expect("focus header");
    let corr_idx = compiled.markdown.find("## Corrections").expect("corr header");
    let epi_idx = compiled.markdown.find("## Episodic").expect("epi header");
    let sem_idx = compiled.markdown.find("## Semantic").expect("sem header");
    let cand_idx = compiled.markdown.find("## Candidates").expect("cand header");

    assert!(canon_idx < pref_idx);
    assert!(pref_idx < focus_idx);
    assert!(focus_idx < corr_idx);
    assert!(corr_idx < epi_idx);
    assert!(epi_idx < sem_idx);
    assert!(sem_idx < cand_idx);
}

#[test]
fn render_includes_demotion_hint_section_when_overflow() {
    // Force overflow by giving more semantic records than can fit.
    let recs: Vec<_> = (0..20)
        .map(|i| rec(&format!("id={i:02} | c={}", "z".repeat(95))))
        .collect();
    let input = CompilerInput {
        semantic: recs,
        ..Default::default()
    };
    let mut budget = WakeBudget::default_2000();
    budget.tokens = 800;
    let compiled = compile_wake(input, budget);

    assert!(
        !compiled.demotion_hints.is_empty(),
        "demotion hints should populate when overflow occurs"
    );
    assert!(
        compiled.markdown.contains("## Demoted"),
        "render must include demotion section when overflow exists; got:\n{}",
        compiled.markdown
    );
    assert!(
        compiled.markdown.contains("memd lookup"),
        "demotion section must point at `memd lookup`"
    );
}

#[test]
fn render_is_markdown_and_round_trips_token_count() {
    let input = CompilerInput {
        canonical: vec![rec("id=A | c=hello")],
        ..Default::default()
    };
    let compiled = compile_wake(input, WakeBudget::default_2000());

    assert!(compiled.markdown.starts_with("## "), "must start with markdown header");
    assert_eq!(
        compiled.tokens,
        compiled.markdown.len(),
        "tokens must equal markdown char length (parity with compute_wake_token_metrics)"
    );
}

// -------- F4.3: preference non-demotion + drift surface ----------

#[test]
fn budget_preferences_bucket_is_non_demotable() {
    // 10 preference records ~250 chars each = 2500 chars total.
    // Budget 1000, class cap (session_continuity = 20%) = 200.
    // Without the F4.3 rule, almost nothing fits. With it, all 10 land.
    let recs: Vec<_> = (0..10)
        .map(|i| rec(&format!("id={i:02} | c={}", "p".repeat(240))))
        .collect();
    let input = CompilerInput {
        preferences: recs,
        ..Default::default()
    };
    let mut budget = WakeBudget::default_2000();
    budget.tokens = 1000;
    let admitted = budget::admit(dedupe::merge(priority::apply(&input)), &budget);

    assert_eq!(
        admitted_count(&admitted, BucketKind::Preference),
        10,
        "preferences are non-demotable; all 10 must land regardless of class cap"
    );
    assert_eq!(
        demoted_count(&admitted, BucketKind::Preference),
        0,
        "preferences must never appear in demoted set"
    );
}

#[test]
fn render_surfaces_drift_line_inside_preferences_section() {
    let input = CompilerInput {
        preferences: vec![rec("id=P1 | c=user prefers terse")],
        drift_notes: vec![
            "⚠ drift: pref-voice-terse (3 violations in last 10 turns)".to_string(),
        ],
        ..Default::default()
    };
    let compiled = compile_wake(input, WakeBudget::default_2000());

    let pref_idx = compiled
        .markdown
        .find("## Preferences")
        .expect("pref header present");
    let drift_idx = compiled.markdown.find("⚠ drift").expect("drift line present");
    assert!(
        drift_idx > pref_idx,
        "drift line must follow the `## Preferences` header"
    );

    let first_record_idx = compiled
        .markdown
        .find("- id=P1")
        .expect("preference record present");
    assert!(
        drift_idx < first_record_idx,
        "drift line must precede first preference record"
    );

    let drift_line = compiled.markdown[drift_idx..]
        .lines()
        .next()
        .expect("drift line resolves");
    assert!(
        drift_line.chars().count() <= 80,
        "drift line ≤80 chars; got {} chars: {drift_line}",
        drift_line.chars().count()
    );
}

#[test]
fn render_omits_drift_block_when_no_notes() {
    let input = CompilerInput {
        preferences: vec![rec("id=P1 | c=user prefers terse")],
        ..Default::default()
    };
    let compiled = compile_wake(input, WakeBudget::default_2000());
    assert!(
        !compiled.markdown.contains("⚠ drift"),
        "no drift line should render when drift_notes empty"
    );
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
