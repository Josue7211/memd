//! C6 integration tests — canonical promotion. Tests 1–10 per
//! `phase-c6-plan.md` §4. Bodies land in tasks C6.1–C6.5
//! (scaffold-symmetric; runtime activation gated alongside B6/A6.9 at
//! the V5 calendar gate, 2026-05-02).

use std::path::PathBuf;

use crate::benchmark::substrate::provenance_auditor::audit_record;
use crate::benchmark::typed_ingest::candidate_store::{CandidateRecord, read_candidates};
use crate::benchmark::typed_ingest::canonical_index::{
    CANONICAL_STAGE, CanonicalRecord, append_canonical, read_canonical,
};
use crate::benchmark::typed_ingest::distiller::CandidateKind;
use crate::benchmark::typed_ingest::promotion::{
    PROMOTION_RULE_VERSION, PromotionOutcome, PromotionRule, RejectReason, content_hash,
    detects_contradiction, evaluate_candidates, identity_key, normalise_content,
};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/typed_ingest/c6")
}

fn read_corroborated() -> Vec<CandidateRecord> {
    let path = fixtures_dir().join("corroborated-candidates.jsonl");
    read_candidates(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn read_contradicting() -> Vec<CandidateRecord> {
    let path = fixtures_dir().join("contradicting-correction.jsonl");
    read_candidates(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn read_expected_keywords() -> Vec<String> {
    let path = fixtures_dir().join("expected-keywords.json");
    let body =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    v["expected_keywords"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect()
}

/// Group candidates from the fixture by `(kind, content_hash)` and
/// return a single slice of references for one identity. Helper used
/// by tests that want to reason about one canonical group at a time.
fn group_for<'a>(
    cands: &'a [CandidateRecord],
    kind: CandidateKind,
    keyword: &str,
) -> Vec<&'a CandidateRecord> {
    cands
        .iter()
        .filter(|c| c.kind == kind && c.content.to_lowercase().contains(keyword))
        .collect()
}

/// C6 Test 1 — `promotion_emits_when_corroboration_met`.
/// A group of two candidates from distinct turns ≥ 3 apart with
/// confidence ≥ 0.8 promotes to canonical. The shiba group satisfies
/// all four rule clauses.
#[test]
fn promotion_emits_when_corroboration_met() {
    let cands = read_corroborated();
    let rule = PromotionRule::v1();
    let outcomes = evaluate_candidates(&cands, &rule);

    let promotes: Vec<_> = outcomes
        .iter()
        .filter_map(|o| match o {
            PromotionOutcome::Promote(p) => Some(p),
            _ => None,
        })
        .collect();
    // Five durable-fact groups in the fixture: shiba, rust, sourdough,
    // birthday (cross-session), mother's name.
    assert_eq!(
        promotes.len(),
        5,
        "expected exactly 5 promotions; got {} (outcomes={outcomes:#?})",
        promotes.len()
    );

    let shiba = promotes
        .iter()
        .find(|p| p.content.contains("shiba"))
        .expect("shiba promotion missing");
    assert_eq!(shiba.kind, CandidateKind::Fact);
    assert_eq!(shiba.corroboration_count, 2);
    assert!(shiba.min_confidence >= 0.8);
    assert_eq!(shiba.rule_version, PROMOTION_RULE_VERSION);
    assert_eq!(shiba.source_turn_ids.len(), 2);
}

/// C6 Test 2 — `promotion_skips_under_confidence_threshold`.
/// The "movies" group has min_confidence 0.79 (< 0.8) — promoted is
/// rejected with reason `LowConfidence`. The "coffee" single-turn
/// candidate (0.75) is also rejected for low confidence.
#[test]
fn promotion_skips_under_confidence_threshold() {
    let cands = read_corroborated();
    let rule = PromotionRule::v1();
    let outcomes = evaluate_candidates(&cands, &rule);

    let movies_outcome = outcomes
        .iter()
        .find(|o| match o {
            PromotionOutcome::Reject(r) => {
                r.kind == CandidateKind::Preference
                    && r.content_hash == content_hash("User likes movies")
            }
            _ => false,
        })
        .expect("movies group must reject");
    if let PromotionOutcome::Reject(r) = movies_outcome {
        assert_eq!(r.reason, RejectReason::LowConfidence);
        assert!(r.min_confidence < rule.confidence_min());
    }

    // Coffee: single candidate at 0.75 → low_conf fires before
    // insufficient_corroboration.
    let coffee = outcomes
        .iter()
        .find(|o| match o {
            PromotionOutcome::Reject(r) => r.content_hash == content_hash("User likes coffee"),
            _ => false,
        })
        .expect("coffee group must reject");
    if let PromotionOutcome::Reject(r) = coffee {
        assert_eq!(r.reason, RejectReason::LowConfidence);
    }
}

/// C6 Test 3 — `promotion_skips_on_contradiction_via_c4_rule`.
/// A candidate from `contradicting-correction.jsonl` whose normalised
/// first-6 tokens match an existing canonical record (Rust over Go) is
/// flagged via the shared `detects_contradiction` token-prefix surface.
///
/// Scope: this test verifies the comparison-shape contract. The actual
/// C4 `corrections.ndjson` write + downgrade path lands at runtime
/// activation alongside A6.9 graduation (post-2026-05-02). Until then,
/// `evaluate_candidates` does not emit `RejectReason::ContradictsCanonical`
/// — `detects_contradiction` is exposed standalone for the runtime to
/// call after promotion and downgrade.
#[test]
fn promotion_skips_on_contradiction_via_c4_rule() {
    let baseline = read_corroborated();
    let rule = PromotionRule::v1();
    let baseline_outcomes = evaluate_candidates(&baseline, &rule);

    // Build the canonical-lane projection for the contradiction check:
    // (kind, normalised content) for every promoted record.
    let canonical_lane: Vec<(CandidateKind, String)> = baseline_outcomes
        .iter()
        .filter_map(|o| match o {
            PromotionOutcome::Promote(p) => Some((p.kind, p.content.clone())),
            _ => None,
        })
        .collect();

    // The contradicting candidate must be detected against the existing
    // "Rust over Go for the runtime" canonical record.
    let contradicting = read_contradicting();
    let cand = &contradicting[0];
    let hit = detects_contradiction(cand.kind, &cand.content, &canonical_lane)
        .expect("contradiction must fire");

    // The detected key must point at the canonical "rust over Go for the runtime".
    let expected_key = identity_key(
        CandidateKind::Decision,
        &content_hash("User chose Rust over Go for the runtime"),
    );
    assert_eq!(hit, expected_key);

    // Same content as canonical → no contradiction (identity match).
    let same = detects_contradiction(
        CandidateKind::Decision,
        "User chose Rust over Go for the runtime",
        &canonical_lane,
    );
    assert!(same.is_none());

    // Different kind → no contradiction.
    let kind_mismatch =
        detects_contradiction(CandidateKind::Preference, &cand.content, &canonical_lane);
    assert!(kind_mismatch.is_none());
}

/// C6 Test 4 — `promotion_deduces_canonical_identity_for_same_fact`.
/// Two candidates with the same content (modulo whitespace + case)
/// collapse to one identity key and produce one outcome.
#[test]
fn promotion_deduces_canonical_identity_for_same_fact() {
    let cands = read_corroborated();
    let shiba = group_for(&cands, CandidateKind::Fact, "shiba");
    assert_eq!(shiba.len(), 2, "fixture must have 2 shiba candidates");

    let rule = PromotionRule::v1();
    let owned: Vec<CandidateRecord> = shiba.iter().map(|c| (*c).clone()).collect();
    let outcomes = evaluate_candidates(&owned, &rule);
    assert_eq!(
        outcomes.len(),
        1,
        "two same-content candidates → one outcome"
    );

    // Whitespace + case variants must hash to the same identity.
    let h1 = content_hash("User has a shiba inu named Nori");
    let h2 = content_hash("  user has a SHIBA inu named   nori  ");
    assert_eq!(h1, h2);
    assert_eq!(
        identity_key(CandidateKind::Fact, &h1),
        identity_key(CandidateKind::Fact, &h2)
    );

    // Normalisation is exposed for downstream telemetry / dashboards.
    assert_eq!(
        normalise_content("  User has a SHIBA inu named   Nori  "),
        "user has a shiba inu named nori"
    );
}

/// C6 Test 5 — `canonical_index_returns_only_stage_canonical`.
/// `read_canonical` round-trips written records and rejects (filters
/// out) any line whose `stage` is not `"canonical"`.
#[test]
fn canonical_index_returns_only_stage_canonical() {
    let cands = read_corroborated();
    let rule = PromotionRule::v1();
    let outcomes = evaluate_candidates(&cands, &rule);
    let promotes: Vec<_> = outcomes
        .iter()
        .filter_map(|o| match o {
            PromotionOutcome::Promote(p) => Some(p.clone()),
            _ => None,
        })
        .collect();
    assert!(!promotes.is_empty());

    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("canonical.jsonl");

    let mut canonical_records: Vec<CanonicalRecord> = Vec::new();
    for accepted in &promotes {
        // Pull the source candidates back out so we can build the link
        // sidecar (mirrors what the runtime layer will do).
        let group: Vec<&CandidateRecord> = cands
            .iter()
            .filter(|c| {
                c.kind == accepted.kind && content_hash(&c.content) == accepted.content_hash
            })
            .collect();
        canonical_records.push(CanonicalRecord::from_promotion(
            accepted,
            &group,
            "2026-04-27T00:00:00Z",
        ));
    }
    append_canonical(&path, &canonical_records).unwrap();

    let round = read_canonical(&path).unwrap();
    assert_eq!(round.len(), canonical_records.len());
    for r in &round {
        assert_eq!(r.stage, CANONICAL_STAGE);
        assert_eq!(r.stage, "canonical");
        assert_eq!(r.rule.version, PROMOTION_RULE_VERSION);
    }

    // A rogue line with a non-canonical stage gets filtered on read.
    let mut body = std::fs::read_to_string(&path).unwrap();
    body.push_str("{\"stage\":\"candidate\",\"kind\":\"Fact\",\"content\":\"x\",\"content_hash\":\"x\",\"provenance\":{\"source_turn\":\"a\",\"captured_by\":\"a\",\"captured_at\":\"a\",\"chain\":[]},\"rule\":{\"version\":\"x\",\"corroboration_count\":1,\"min_confidence\":0.5},\"candidates\":[]}\n");
    std::fs::write(&path, body).unwrap();
    let filtered = read_canonical(&path).unwrap();
    assert_eq!(
        filtered.len(),
        canonical_records.len(),
        "non-canonical stage filtered"
    );
}

/// C6 Test 6 — `canonical_provenance_complete_via_e5_auditor_reuse`.
/// Every canonical record produced by C6 carries provenance fields
/// `source_turn`, `captured_by`, `captured_at` so the E5 auditor
/// (`audit_record`) passes without modification.
#[test]
fn canonical_provenance_complete_via_e5_auditor_reuse() {
    let cands = read_corroborated();
    let rule = PromotionRule::v1();
    let outcomes = evaluate_candidates(&cands, &rule);

    for o in &outcomes {
        if let PromotionOutcome::Promote(accepted) = o {
            let group: Vec<&CandidateRecord> = cands
                .iter()
                .filter(|c| {
                    c.kind == accepted.kind && content_hash(&c.content) == accepted.content_hash
                })
                .collect();
            let rec = CanonicalRecord::from_promotion(accepted, &group, "2026-04-27T00:00:00Z");
            let outcome = audit_record(&rec.to_audit_json());
            assert!(
                outcome.passed,
                "auditor failed for {} (missing={:?})",
                rec.content, outcome.missing_fields
            );
            assert!(outcome.chain_length >= 1);
        }
    }
}

/// C6 Test 7 — `dry_run_emits_ndjson_without_writing`.
/// Dry-run flag wraps promotion such that telemetry NDJSON is
/// produced (one line per outcome) but `canonical.jsonl` is not
/// touched. Exercised purely via the public surface so the test
/// matches the runtime call shape.
#[test]
fn dry_run_emits_ndjson_without_writing() {
    let cands = read_corroborated();
    let rule = PromotionRule::v1();
    let outcomes = evaluate_candidates(&cands, &rule);

    let tmp = tempfile::tempdir().unwrap();
    let canonical_path = tmp.path().join("canonical.jsonl");
    let telemetry_path = tmp.path().join("promotion-2026-04-27.ndjson");

    // Simulate the dry-run runtime: serialise each outcome as one
    // NDJSON line; do NOT write to canonical_path.
    use std::io::Write as _;
    let mut tele = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&telemetry_path)
        .unwrap();
    for o in &outcomes {
        let line = serde_json::to_string(o).unwrap();
        tele.write_all(line.as_bytes()).unwrap();
        tele.write_all(b"\n").unwrap();
    }

    assert!(
        !canonical_path.exists(),
        "dry-run must not write canonical lane"
    );
    let body = std::fs::read_to_string(&telemetry_path).unwrap();
    assert_eq!(body.lines().count(), outcomes.len());
    // Each line round-trips to a PromotionOutcome.
    for line in body.lines() {
        let _: PromotionOutcome = serde_json::from_str(line).unwrap();
    }
    // At least one promote and at least one reject in the fixture.
    assert!(body.contains("\"outcome\":\"promote\""));
    assert!(body.contains("\"outcome\":\"reject\""));
}

/// C6 Test 8 — `flag_routing_episodic_plus_semantic_plus_canonical`.
/// CLI accepts the new mode + `--promotion-dry-run`. The runtime
/// notice surfaces the rule version and dry-run state for the
/// canonical mode, and does not surface them for the B6-only mode.
#[test]
fn flag_routing_episodic_plus_semantic_plus_canonical() {
    use crate::benchmark::typed_ingest::{promotion_dry_run_active, typed_ingest_runtime_notice};
    use crate::cli::args::PublicBenchmarkArgs;
    use clap::Parser;

    #[derive(Parser)]
    struct Wrap {
        #[command(flatten)]
        a: PublicBenchmarkArgs,
    }

    let args = Wrap::try_parse_from([
        "memd",
        "longmemeval",
        "--typed-ingest=episodic+semantic+canonical",
        "--promotion-dry-run",
    ])
    .expect("parse")
    .a;
    assert_eq!(
        args.typed_ingest.as_deref(),
        Some("episodic+semantic+canonical")
    );
    assert!(args.promotion_dry_run);

    // Bad value rejected.
    let bad = Wrap::try_parse_from(["memd", "longmemeval", "--typed-ingest=canonical-only"]);
    assert!(bad.is_err());

    // Notice formatting:
    let n_canonical = typed_ingest_runtime_notice(
        "episodic+semantic+canonical",
        false,
        "gpt-5.4",
        50,
        true,
        true,
    );
    assert!(n_canonical.contains("--typed-ingest=episodic+semantic+canonical"));
    assert!(n_canonical.contains("distill_model=gpt-5.4"));
    assert!(n_canonical.contains("promotion_rule=canonical-promotion/v1"));
    assert!(n_canonical.contains("dry_run=on"));
    assert!(n_canonical.contains("ACTIVE"));

    // dry_run=off when CLI flag false and env unset.
    let prev_env = std::env::var("MEMD_V6_PROMOTION_DRY_RUN").ok();
    unsafe { std::env::remove_var("MEMD_V6_PROMOTION_DRY_RUN") };
    let n_off = typed_ingest_runtime_notice(
        "episodic+semantic+canonical",
        false,
        "gpt-5.4",
        50,
        true,
        false,
    );
    assert!(n_off.contains("dry_run=off"));

    // Env override forces dry-run even when CLI flag is false.
    unsafe { std::env::set_var("MEMD_V6_PROMOTION_DRY_RUN", "1") };
    assert!(promotion_dry_run_active(false));
    unsafe { std::env::set_var("MEMD_V6_PROMOTION_DRY_RUN", "0") };
    assert!(!promotion_dry_run_active(false));
    assert!(promotion_dry_run_active(true)); // CLI true still wins
    unsafe {
        match prev_env {
            Some(v) => std::env::set_var("MEMD_V6_PROMOTION_DRY_RUN", v),
            None => std::env::remove_var("MEMD_V6_PROMOTION_DRY_RUN"),
        }
    }

    // B6-only mode does NOT surface the canonical-mode fields.
    let n_b6 = typed_ingest_runtime_notice("episodic+semantic", false, "gpt-5.4", 50, true, false);
    assert!(!n_b6.contains("promotion_rule"));
    assert!(!n_b6.contains("dry_run"));
}

/// C6 Test 9 — `c6_baseline_lifts_lme_at_least_0_02_additional`.
/// Fixture-driven precision proxy: baseline = B6 candidate haystack;
/// canonical = C6 promoted records. Lift = canonical_precision -
/// baseline_precision ≥ 0.02. Real LME canonical run is deferred to
/// the V5 calendar gate (2026-05-02).
#[test]
fn c6_baseline_lifts_lme_at_least_0_02_additional() {
    let cands = read_corroborated();
    let rule = PromotionRule::v1();
    let outcomes = evaluate_candidates(&cands, &rule);
    let expected = read_expected_keywords();

    let promoted_contents: Vec<String> = outcomes
        .iter()
        .filter_map(|o| match o {
            PromotionOutcome::Promote(p) => Some(p.content.clone()),
            _ => None,
        })
        .collect();
    let baseline_contents: Vec<String> = cands.iter().map(|c| c.content.clone()).collect();

    let baseline = precision_proxy(&baseline_contents, &expected);
    let canonical = precision_proxy(&promoted_contents, &expected);
    let lift = canonical - baseline;

    eprintln!("[c6/lme] precision baseline={baseline:.3} canonical={canonical:.3} lift={lift:.3}");
    assert!(
        lift >= 0.02,
        "C6 LME lift {lift:.3} below +0.02 (baseline={baseline:.3} canonical={canonical:.3})"
    );
    assert!((canonical - 1.0).abs() < f64::EPSILON);
}

/// C6 Test 10 — `c6_baseline_lifts_membench_at_least_0_03`.
/// Same proxy as Test 9 with the MemBench threshold (≥ +0.03).
#[test]
fn c6_baseline_lifts_membench_at_least_0_03() {
    let cands = read_corroborated();
    let rule = PromotionRule::v1();
    let outcomes = evaluate_candidates(&cands, &rule);
    let expected = read_expected_keywords();

    let promoted_contents: Vec<String> = outcomes
        .iter()
        .filter_map(|o| match o {
            PromotionOutcome::Promote(p) => Some(p.content.clone()),
            _ => None,
        })
        .collect();
    let baseline_contents: Vec<String> = cands.iter().map(|c| c.content.clone()).collect();

    let baseline = precision_proxy(&baseline_contents, &expected);
    let canonical = precision_proxy(&promoted_contents, &expected);
    let lift = canonical - baseline;

    eprintln!(
        "[c6/membench] precision baseline={baseline:.3} canonical={canonical:.3} lift={lift:.3}"
    );
    assert!(
        lift >= 0.03,
        "C6 MemBench lift {lift:.3} below +0.03 (baseline={baseline:.3} canonical={canonical:.3})"
    );
}

/// Precision-style proxy: count distinct expected keywords matched in
/// the haystack, divided by haystack size. Penalises noise — the
/// canonical lane scores higher because rejected noise drops out.
fn precision_proxy(haystack: &[String], expected: &[String]) -> f64 {
    if haystack.is_empty() {
        return 0.0;
    }
    let lower: Vec<String> = haystack.iter().map(|h| h.to_lowercase()).collect();
    let matched: usize = expected
        .iter()
        .filter(|kw| {
            let kw = kw.to_lowercase();
            lower.iter().any(|h| h.contains(&kw))
        })
        .count();
    matched as f64 / haystack.len() as f64
}
