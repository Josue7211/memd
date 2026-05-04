//! B6 integration tests — semantic distillation. Tests 1–10 per
//! `phase-b6-plan.md` §4. Bodies land in tasks B6.1–B6.7.
//!
//! The judge call itself is mocked via the cache layer (see
//! `tests/fixtures/typed_ingest/b6/cached-extractions-sample.jsonl`):
//! pre-recorded extractions go in, the distiller reads them back and
//! never touches the network. Real LME run is deferred to post-V5
//! calendar gate (2026-05-02) alongside A6.9.

use std::io::Write as _;
use std::path::PathBuf;

use serde_json::json;

use crate::benchmark::typed_ingest::candidate_store::{
    CANDIDATE_STAGE, CandidateRecord, append_candidates, read_candidates,
};
use crate::benchmark::typed_ingest::dedupe::{
    COSINE_NEAR_DUPLICATE, cosine_on_unit, dedupe_hash, dedupe_hash_cosine,
};
use crate::benchmark::typed_ingest::distiller::{
    CacheOutcome, CacheRecord, CandidateKind, DistillCandidate, DistillOutput, DistillTelemetry,
    PROMPT_CARD_V1, PROMPT_CARD_VERSION, PromptCard, append_distill_telemetry, cache_get,
    cache_key, cache_put, distill_telemetry_path, format_distill_telemetry_line,
    validate_distill_json,
};
use crate::benchmark::typed_ingest::episodic::{EpisodicProvenance, EpisodicTurn};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/typed_ingest/b6")
}

fn read_turns_with_facts() -> Vec<EpisodicTurn> {
    let path = fixtures_dir().join("turns-with-facts.jsonl");
    let body =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    body.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("parse turn"))
        .collect()
}

fn read_cached_extractions() -> Vec<CacheRecord> {
    let path = fixtures_dir().join("cached-extractions-sample.jsonl");
    let body =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    body.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("parse cache record"))
        .collect()
}

fn read_expected_keywords() -> Vec<String> {
    let path = fixtures_dir().join("expected-facts.json");
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

/// B6 Test 1 — `distiller_prompt_card_loads`.
/// `PromptCard::v1()` returns the frozen card and its version is the
/// stable identifier the cache key depends on.
#[test]
fn distiller_prompt_card_loads() {
    let card = PromptCard::v1();
    assert_eq!(card.version, PROMPT_CARD_VERSION);
    assert_eq!(card.version, "semantic-distillation/v1");
    assert_eq!(card.system_prompt, PROMPT_CARD_V1);
    // Card must constrain the model to JSON-only output.
    assert!(card.system_prompt.contains("ONLY a JSON object"));
    assert!(card.system_prompt.contains("Fact"));
    assert!(card.system_prompt.contains("Decision"));
    assert!(card.system_prompt.contains("Preference"));
}

/// B6 Test 2 — `distiller_emits_valid_schema_on_happy_turn`.
/// A well-formed judge response parses into typed candidates with
/// kind/content/confidence/source_turn_ids/rationale. Malformed input
/// is rejected.
#[test]
fn distiller_emits_valid_schema_on_happy_turn() {
    let raw = json!({
        "candidates": [
            {
                "kind": "Preference",
                "content": "User prefers sourdough bread, particularly rye",
                "confidence": 0.86,
                "source_turn_ids": ["sess_b6::0"],
                "rationale": "explicit pref"
            }
        ]
    })
    .to_string();
    let parsed: DistillOutput = validate_distill_json(&raw).expect("valid schema");
    assert_eq!(parsed.candidates.len(), 1);
    let c = &parsed.candidates[0];
    assert_eq!(c.kind, CandidateKind::Preference);
    assert!(c.content.starts_with("User prefers sourdough"));
    assert!((c.confidence - 0.86).abs() < 1e-6);
    assert_eq!(c.source_turn_ids, vec!["sess_b6::0"]);

    // Reject unknown kind
    let bad_kind = json!({"candidates":[{
        "kind":"Mood","content":"x","confidence":0.5,
        "source_turn_ids":["a"],"rationale":""
    }]})
    .to_string();
    assert!(validate_distill_json(&bad_kind).is_err());

    // Reject confidence out of range
    let bad_conf = json!({"candidates":[{
        "kind":"Fact","content":"x","confidence":1.5,
        "source_turn_ids":["a"],"rationale":""
    }]})
    .to_string();
    assert!(validate_distill_json(&bad_conf).is_err());

    // Reject empty source_turn_ids
    let bad_src = json!({"candidates":[{
        "kind":"Fact","content":"x","confidence":0.5,
        "source_turn_ids":[],"rationale":""
    }]})
    .to_string();
    assert!(validate_distill_json(&bad_src).is_err());

    // Reject extra top-level key
    let bad_top = json!({
        "candidates":[],
        "extra":"nope"
    })
    .to_string();
    assert!(validate_distill_json(&bad_top).is_err());

    // Reject empty content
    let bad_empty = json!({"candidates":[{
        "kind":"Fact","content":"   ","confidence":0.5,
        "source_turn_ids":["a"],"rationale":""
    }]})
    .to_string();
    assert!(validate_distill_json(&bad_empty).is_err());
}

/// B6 Test 3 — `distiller_zero_candidates_on_chat_filler`.
/// Filler turns ("got it", "sounds reasonable") in the curated fixture
/// have empty cached extractions. The distiller surfaces zero
/// candidates for those turns — the validator + cache layer agree.
#[test]
fn distiller_zero_candidates_on_chat_filler() {
    let cached = read_cached_extractions();
    let turns = read_turns_with_facts();
    assert_eq!(cached.len(), turns.len(), "fixture line counts must match");

    // Index-aligned: filler turns are turn_index 1 and 3 in the fixture.
    let filler_indices = [1usize, 3];
    for &i in &filler_indices {
        assert_eq!(turns[i].provenance.speaker, "assistant");
        assert!(
            cached[i].candidates.is_empty(),
            "cached extraction for filler turn_index={} should be empty",
            i
        );
    }
    // Total candidate count across filler turns is zero.
    let filler_total: usize = filler_indices
        .iter()
        .map(|&i| cached[i].candidates.len())
        .sum();
    assert_eq!(filler_total, 0);
}

/// B6 Test 4 — `distiller_caches_by_turn_id_and_prompt_version`.
/// Cache key = sha256(prompt_version || source_hash). Different
/// prompt_versions yield different keys. cache_put → cache_get
/// round-trips the record. Missing key returns Ok(None).
#[test]
fn distiller_caches_by_turn_id_and_prompt_version() {
    let dir = tempfile::tempdir().unwrap();
    let cache_dir = dir.path();

    let source_hash = "0000000000000000000000000000000000000000000000000000000000000001";
    let k1 = cache_key("semantic-distillation/v1", source_hash);
    let k2 = cache_key("semantic-distillation/v2", source_hash);
    let k3 = cache_key("semantic-distillation/v1", "deadbeef");

    assert_eq!(k1.len(), 64);
    assert_ne!(k1, k2, "different prompt_version → different key");
    assert_ne!(k1, k3, "different source_hash → different key");
    assert_eq!(
        k1,
        cache_key("semantic-distillation/v1", source_hash),
        "stable across calls"
    );

    // Miss → None
    assert!(cache_get(cache_dir, &k1).unwrap().is_none());

    let rec = CacheRecord {
        key: k1.clone(),
        model: "gpt-5.4".to_string(),
        milli_usd: 2,
        candidates: vec![DistillCandidate {
            kind: CandidateKind::Fact,
            content: "X".to_string(),
            confidence: 0.9,
            source_turn_ids: vec!["sess::0".to_string()],
            rationale: "r".to_string(),
        }],
        ts: "2026-04-27T00:00:00Z".to_string(),
    };
    cache_put(cache_dir, &rec).expect("put");

    let hit = cache_get(cache_dir, &k1).unwrap().expect("hit");
    assert_eq!(hit, rec);
}

/// B6 Test 5 — `dedupe_collapses_near_duplicate_by_hash`.
/// Two candidates whose normalised content matches exactly collapse to
/// one; case + leading/trailing whitespace differences still collapse.
#[test]
fn dedupe_collapses_near_duplicate_by_hash() {
    let mk = |s: &str| DistillCandidate {
        kind: CandidateKind::Fact,
        content: s.to_string(),
        confidence: 0.9,
        source_turn_ids: vec!["sess::0".to_string()],
        rationale: "r".to_string(),
    };
    let input = vec![
        mk("User likes sourdough"),
        mk("  user likes sourdough  "), // case + whitespace duplicate
        mk("User chose Rust over Go"),
        mk("User likes sourdough"), // exact duplicate
    ];
    let report = dedupe_hash(input);
    assert_eq!(report.kept.len(), 2);
    assert_eq!(report.collapsed_hash, 2);
    assert_eq!(report.collapsed_cosine, 0);
    assert_eq!(report.kept[0].content, "User likes sourdough");
    assert_eq!(report.kept[1].content, "User chose Rust over Go");
}

/// B6 Test 6 — `dedupe_collapses_near_duplicate_by_cosine`.
/// Two candidates whose content hashes differ but whose embeddings are
/// near-parallel (cosine ≥ 0.85) collapse via cosine. Distinct topics
/// stay separate.
#[test]
fn dedupe_collapses_near_duplicate_by_cosine() {
    let mk = |s: &str| DistillCandidate {
        kind: CandidateKind::Fact,
        content: s.to_string(),
        confidence: 0.9,
        source_turn_ids: vec!["sess::0".to_string()],
        rationale: "r".to_string(),
    };
    // Three candidates: A and A' near-parallel, B orthogonal.
    let candidates = vec![
        mk("User likes sourdough bread"),
        mk("User enjoys sourdough loaves"), // same topic, different wording
        mk("User chose Rust over Go"),
    ];
    let a = vec![1.0_f32, 0.0, 0.0];
    let a_prime = {
        // 0.95 cosine with a (well above 0.85 threshold)
        let mut v = vec![0.95_f32, 0.31224989991, 0.0];
        let n: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        v.iter_mut().for_each(|x| *x /= n);
        v
    };
    let b = vec![0.0_f32, 0.0, 1.0];
    assert!(cosine_on_unit(&a, &a_prime) >= COSINE_NEAR_DUPLICATE);
    assert!(cosine_on_unit(&a, &b) < COSINE_NEAR_DUPLICATE);

    let report = dedupe_hash_cosine(candidates, vec![a.clone(), a_prime, b.clone()]);
    assert_eq!(report.kept.len(), 2, "near-dup A' should collapse");
    assert_eq!(report.collapsed_cosine, 1);
    assert_eq!(report.collapsed_hash, 0);
    assert!(report.kept.iter().any(|c| c.content.contains("sourdough")));
    assert!(report.kept.iter().any(|c| c.content.contains("Rust")));
}

/// B6 Test 7 — `candidate_store_persists_as_stage_candidate`.
/// Append → read round-trip preserves stage, kind, content, provenance,
/// distill sidecar. stage is exactly "candidate".
#[test]
fn candidate_store_persists_as_stage_candidate() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("candidates.jsonl");

    let prov = EpisodicProvenance {
        bench_id: "longmemeval".to_string(),
        session_id: "sess_b6".to_string(),
        turn_index: 0,
        speaker: "user".to_string(),
        source_hash: "a".repeat(64),
        captured_at: "2024-01-01".to_string(),
    };
    let cand = DistillCandidate {
        kind: CandidateKind::Preference,
        content: "User prefers sourdough".to_string(),
        confidence: 0.9,
        source_turn_ids: vec!["sess_b6::0".to_string()],
        rationale: "explicit pref".to_string(),
    };
    let rec = CandidateRecord::from_parts(cand, prov, PROMPT_CARD_VERSION, "gpt-5.4");
    append_candidates(&path, &[rec.clone()]).unwrap();

    let round = read_candidates(&path).unwrap();
    assert_eq!(round.len(), 1);
    assert_eq!(round[0], rec);
    assert_eq!(round[0].stage, CANDIDATE_STAGE);
    assert_eq!(round[0].stage, "candidate");
    assert_eq!(round[0].kind, CandidateKind::Preference);

    // Append more — file grows, prior records preserved.
    let cand2 = DistillCandidate {
        kind: CandidateKind::Fact,
        content: "User has a shiba".to_string(),
        confidence: 0.95,
        source_turn_ids: vec!["sess_b6::4".to_string()],
        rationale: "durable fact".to_string(),
    };
    let rec2 = CandidateRecord::from_parts(
        cand2,
        EpisodicProvenance {
            bench_id: "longmemeval".to_string(),
            session_id: "sess_b6".to_string(),
            turn_index: 4,
            speaker: "user".to_string(),
            source_hash: "b".repeat(64),
            captured_at: "2024-01-03".to_string(),
        },
        PROMPT_CARD_VERSION,
        "gpt-5.4",
    );
    append_candidates(&path, &[rec2.clone()]).unwrap();
    let round2 = read_candidates(&path).unwrap();
    assert_eq!(round2.len(), 2);
    assert_eq!(round2[1], rec2);
}

/// B6 Test 8 — `candidate_provenance_references_source_turns`.
/// Every persisted candidate's `distill.source_turn_ids` resolves to a
/// real turn (`session_id::turn_index` form), and the candidate's
/// provenance matches the originating turn.
#[test]
fn candidate_provenance_references_source_turns() {
    let turns = read_turns_with_facts();
    let cached = read_cached_extractions();
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("candidates.jsonl");

    // Build records by zipping cached extractions with their turns,
    // mirroring what the runtime will do post-A6.9.
    let mut records = Vec::new();
    for (turn, cache_rec) in turns.iter().zip(cached.iter()) {
        for cand in &cache_rec.candidates {
            records.push(CandidateRecord::from_parts(
                cand.clone(),
                turn.provenance.clone(),
                PROMPT_CARD_VERSION,
                &cache_rec.model,
            ));
        }
    }
    append_candidates(&path, &records).unwrap();

    let round = read_candidates(&path).unwrap();
    assert!(!round.is_empty(), "expected at least one candidate");

    // Map session_id::turn_index → bool for fast lookup.
    let valid_ids: std::collections::HashSet<String> = turns
        .iter()
        .map(|t| format!("{}::{}", t.provenance.session_id, t.provenance.turn_index))
        .collect();

    for r in &round {
        assert_eq!(r.stage, CANDIDATE_STAGE);
        assert!(!r.distill.source_turn_ids.is_empty());
        for id in &r.distill.source_turn_ids {
            assert!(
                valid_ids.contains(id),
                "candidate references unknown turn id `{id}`"
            );
        }
        // Provenance must match a real turn.
        let key = format!("{}::{}", r.provenance.session_id, r.provenance.turn_index);
        assert!(valid_ids.contains(&key));
    }
}

/// B6 Test 9 — `flag_routing_episodic_plus_semantic`.
/// CLI accepts `--typed-ingest=episodic+semantic`, rejects unknown
/// values, and the runtime notice surfaces the distill model + budget
/// when (and only when) the semantic mode is on.
#[test]
fn flag_routing_episodic_plus_semantic() {
    use crate::benchmark::typed_ingest::distiller::{cache_enabled, effective_distill_model};
    use crate::benchmark::typed_ingest::typed_ingest_runtime_notice;
    use crate::cli::args::PublicBenchmarkArgs;
    use clap::Parser;

    #[derive(Parser)]
    struct Wrap {
        #[command(flatten)]
        a: PublicBenchmarkArgs,
    }

    // Episodic+semantic accepted with model + budget overrides.
    let args = Wrap::try_parse_from([
        "memd",
        "longmemeval",
        "--typed-ingest=episodic+semantic",
        "--distill-model",
        "gpt-5.4",
        "--distill-budget-milli-usd",
        "50",
    ])
    .expect("parse")
    .a;
    assert_eq!(args.typed_ingest.as_deref(), Some("episodic+semantic"));
    assert_eq!(args.distill_model, "gpt-5.4");
    assert_eq!(args.distill_budget_milli_usd, 50);

    // Episodic-only still works (A6 surface preserved).
    let a6 = Wrap::try_parse_from(["memd", "longmemeval", "--typed-ingest=episodic"])
        .expect("parse")
        .a;
    assert_eq!(a6.typed_ingest.as_deref(), Some("episodic"));

    // Bad value rejected.
    let bad = Wrap::try_parse_from(["memd", "longmemeval", "--typed-ingest=semantic-only"]);
    assert!(bad.is_err());

    // Notice formatting:
    // - episodic+semantic notice mentions the distill model + budget + cache state.
    let n_full =
        typed_ingest_runtime_notice("episodic+semantic", false, "gpt-5.4", 50, true, false);
    assert!(n_full.contains("--typed-ingest=episodic+semantic"));
    assert!(n_full.contains("distill_model=gpt-5.4"));
    assert!(n_full.contains("budget_milli_usd=50"));
    assert!(n_full.contains("cache=on"));
    assert!(n_full.contains("ACTIVE"));
    // B6 mode does not surface the C6 rule version or dry-run flag.
    assert!(!n_full.contains("promotion_rule"));
    assert!(!n_full.contains("dry_run"));
    // - episodic-only notice does not mention distill.
    let n_a6 = typed_ingest_runtime_notice("episodic", false, "gpt-5.4", 50, true, false);
    assert!(!n_a6.contains("distill_model"));
    assert!(!n_a6.contains("budget_milli_usd"));
    assert!(!n_a6.contains("cache="));
    // - env-active keeps the active phrase; V6 close no longer requires
    //   MEMD_V6_TYPED_INGEST for the notice surface.
    let n_active =
        typed_ingest_runtime_notice("episodic+semantic", true, "gpt-5.4", 50, true, false);
    assert!(n_active.contains("ACTIVE"));
    // - cache=off surfaces when MEMD_V6_DISTILL_CACHE=0.
    let n_no_cache =
        typed_ingest_runtime_notice("episodic+semantic", false, "gpt-5.4", 50, false, false);
    assert!(n_no_cache.contains("cache=off"));

    // Env-var read sites (mirrors A6.8 notice/eprintln pattern):
    // MEMD_V6_DISTILL_MODEL overrides the CLI default; MEMD_V6_DISTILL_CACHE=0
    // disables the cache. Save/restore the old values so parallel tests
    // don't see the override leak. Rust 2024 edition: env mutators are
    // unsafe because process-global state is racy across threads.
    let prev_model = std::env::var("MEMD_V6_DISTILL_MODEL").ok();
    let prev_cache = std::env::var("MEMD_V6_DISTILL_CACHE").ok();

    unsafe { std::env::remove_var("MEMD_V6_DISTILL_MODEL") };
    assert_eq!(effective_distill_model("gpt-5.4"), "gpt-5.4");

    unsafe { std::env::set_var("MEMD_V6_DISTILL_MODEL", "gpt-foo") };
    assert_eq!(effective_distill_model("gpt-5.4"), "gpt-foo");

    unsafe { std::env::set_var("MEMD_V6_DISTILL_MODEL", "   ") };
    assert_eq!(
        effective_distill_model("gpt-5.4"),
        "gpt-5.4",
        "whitespace-only override falls back to CLI default"
    );

    unsafe { std::env::remove_var("MEMD_V6_DISTILL_CACHE") };
    assert!(cache_enabled(), "cache enabled by default");
    unsafe { std::env::set_var("MEMD_V6_DISTILL_CACHE", "0") };
    assert!(!cache_enabled(), "MEMD_V6_DISTILL_CACHE=0 disables cache");
    unsafe { std::env::set_var("MEMD_V6_DISTILL_CACHE", "1") };
    assert!(cache_enabled(), "non-zero values keep cache on");

    // Restore prior environment.
    unsafe {
        match prev_model {
            Some(v) => std::env::set_var("MEMD_V6_DISTILL_MODEL", v),
            None => std::env::remove_var("MEMD_V6_DISTILL_MODEL"),
        }
        match prev_cache {
            Some(v) => std::env::set_var("MEMD_V6_DISTILL_CACHE", v),
            None => std::env::remove_var("MEMD_V6_DISTILL_CACHE"),
        }
    }
}

/// B6 Test 10 — `b6_baseline_lifts_lme_qa_accuracy_at_least_0_02`.
/// Deterministic fixture-driven lift: with cached extractions surfaced
/// as candidates, a substring-recall proxy over the curated keyword set
/// improves by ≥ 0.02 vs the episodic-only baseline. Real LME canonical
/// run is deferred to post-V5 calendar gate (2026-05-02) alongside A6.9.
#[test]
fn b6_baseline_lifts_lme_qa_accuracy_at_least_0_02() {
    let turns = read_turns_with_facts();
    let cached = read_cached_extractions();
    let expected = read_expected_keywords();

    fn proxy_acc(haystack: &[String], expected: &[String]) -> f64 {
        let lower: Vec<String> = haystack.iter().map(|h| h.to_lowercase()).collect();
        let hits = expected
            .iter()
            .filter(|kw| {
                let kw = kw.to_lowercase();
                lower.iter().any(|h| h.contains(&kw))
            })
            .count();
        hits as f64 / expected.len() as f64
    }

    let baseline_haystack: Vec<String> = turns.iter().map(|t| t.content.clone()).collect();
    let mut distill_haystack = baseline_haystack.clone();
    for rec in &cached {
        for c in &rec.candidates {
            distill_haystack.push(c.content.clone());
        }
    }
    let baseline = proxy_acc(&baseline_haystack, &expected);
    let distill = proxy_acc(&distill_haystack, &expected);
    let lift = distill - baseline;

    eprintln!(
        "[b6] qa_proxy baseline={:.3} distill={:.3} lift={:.3}",
        baseline, distill, lift
    );
    assert!(
        lift >= 0.02,
        "B6 lift {lift:.3} below 0.02 threshold (baseline={baseline:.3} distill={distill:.3})"
    );
    // distill must reach the expected keyword set fully on this fixture.
    assert!((distill - 1.0).abs() < f64::EPSILON);
}

/// B6.7 — `distill_telemetry_line_appends_ndjson`.
/// Telemetry helper formats one NDJSON line and appends it to a
/// per-day file under the configured results dir. Aggregator reads
/// these alongside the A6 ingest card. (Beyond the plan's 10 enumerated
/// tests; locks the on-disk format B6.7 promised.)
#[test]
fn distill_telemetry_line_appends_ndjson() {
    let tmp = tempfile::tempdir().unwrap();
    let results = tmp.path();
    let t = DistillTelemetry {
        ts: "2026-04-27T00:00:00Z".to_string(),
        bench_id: "longmemeval".to_string(),
        turn_id: "sess_b6::0".to_string(),
        judge_model: "gpt-5.4".to_string(),
        prompt_tokens: 410,
        completion_tokens: 48,
        milli_usd: 2,
        candidate_count: 1,
        cache: CacheOutcome::Miss,
    };
    let line = format_distill_telemetry_line(&t);
    assert!(line.starts_with("{"));
    assert!(line.contains("\"bench_id\":\"longmemeval\""));
    assert!(line.contains("\"cache\":\"miss\""));

    append_distill_telemetry(results, "2026-04-27", &t).unwrap();
    let path = distill_telemetry_path(results, "2026-04-27");
    let body = std::fs::read_to_string(&path).unwrap();
    assert_eq!(body.lines().count(), 1);
    let parsed: DistillTelemetry = serde_json::from_str(body.lines().next().unwrap()).unwrap();
    assert_eq!(parsed, t);

    // Append a second line — file grows, prior preserved.
    let mut t2 = t.clone();
    t2.turn_id = "sess_b6::4".to_string();
    t2.cache = CacheOutcome::Hit;
    append_distill_telemetry(results, "2026-04-27", &t2).unwrap();
    let body2 = std::fs::read_to_string(&path).unwrap();
    assert_eq!(body2.lines().count(), 2);
    assert!(body2.contains("\"cache\":\"hit\""));
}

/// Avoid unused-import warnings if any of the sub-tests get gated out.
#[allow(dead_code)]
fn _silence_unused() {
    let _ = json!({});
    let _ = std::io::sink().write_all(b"");
}
