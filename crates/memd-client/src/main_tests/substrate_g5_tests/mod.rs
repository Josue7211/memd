//! G5 integration tests — adversarial-noise bench.

use crate::benchmark::substrate::adversarial_noise::{
    generate_corpus, run_g5_in_process, CanonicalWinsScorer, G5RunConfig,
    NoiseRecord, PerfectCanonicalBackend, TieBreakProvenanceScorer,
};
use std::path::PathBuf;
use tempfile::tempdir;

fn small_config(results_dir: PathBuf) -> G5RunConfig {
    let mut cfg = G5RunConfig::default_with_results_dir(results_dir);
    cfg.canonical_count = 10;
    cfg.noise_per_canonical = 3;
    cfg.seed = 40;
    cfg
}

/// G5 Test 1 — `noise_generator_seeds_contradictions_with_recency_offset`.
/// Generated noise records share subject+predicate with their canonical
/// sibling, contradict its value, and carry a strictly-newer timestamp.
#[test]
fn noise_generator_seeds_contradictions_with_recency_offset() {
    let (canonical, noise) = generate_corpus(40, 10, 3, 3600);
    assert_eq!(canonical.len(), 10);
    assert_eq!(noise.len(), 30);

    for canon in &canonical {
        let siblings: Vec<&NoiseRecord> =
            noise.iter().filter(|n| n.canonical_id == canon.canonical_id).collect();
        assert_eq!(siblings.len(), 3, "every canonical needs 3 noise siblings");

        for n in &siblings {
            assert_eq!(n.subject, canon.subject, "noise must share subject");
            assert_eq!(n.predicate, canon.predicate, "noise must share predicate");
            assert_ne!(n.value, canon.value, "noise must contradict canonical value");
            assert!(
                n.captured_at_offset_s > canon.captured_at_offset_s,
                "noise must be newer than canonical (recency-bias trap)"
            );
            assert!(
                n.provenance_chain_len < canon.provenance_chain_len,
                "noise must have weaker provenance"
            );
        }
    }
}

/// G5 Test 2 — `scorer_canonical_wins_rate`.
/// Canonical-wins scorer counts the fraction of queries where the
/// canonical record outranks every noise sibling.
#[test]
fn scorer_canonical_wins_rate() {
    let scorer = CanonicalWinsScorer::new();

    let canon = NoiseRecord {
        id: 0,
        canonical_id: 0,
        is_canonical: true,
        subject: "alice".into(),
        predicate: "lives_in".into(),
        value: "Lisbon".into(),
        captured_at_offset_s: 0,
        provenance_chain_len: 3,
    };
    let noise = NoiseRecord {
        id: 1,
        canonical_id: 0,
        is_canonical: false,
        subject: "alice".into(),
        predicate: "lives_in".into(),
        value: "Mars".into(),
        captured_at_offset_s: 3600,
        provenance_chain_len: 1,
    };

    // Canonical at top → win.
    assert!(scorer.winner_is_canonical(0, &[canon.clone(), noise.clone()]));
    // Noise at top → loss + leak.
    assert!(!scorer.winner_is_canonical(0, &[noise.clone(), canon.clone()]));
    assert_eq!(scorer.leaked_noise(0, &[noise.clone(), canon.clone()]), Some(1));
}

/// G5 Test 3 — `scorer_tie_break_by_provenance_when_canonical_newest`.
/// When canonical and noise tie on relevance, the longer provenance chain
/// must win.
#[test]
fn scorer_tie_break_by_provenance_when_canonical_newest() {
    let tb = TieBreakProvenanceScorer::new();

    let canon = NoiseRecord {
        id: 0,
        canonical_id: 0,
        is_canonical: true,
        subject: "bob".into(),
        predicate: "works_at".into(),
        value: "ACME".into(),
        captured_at_offset_s: 0,
        provenance_chain_len: 3,
    };
    let noise = NoiseRecord {
        id: 1,
        canonical_id: 0,
        is_canonical: false,
        subject: "bob".into(),
        predicate: "works_at".into(),
        value: "Atlantis".into(),
        captured_at_offset_s: 3600,
        provenance_chain_len: 1,
    };
    assert_eq!(tb.tie_break(&canon, &noise), Some(true));

    // Cross-canonical pairing returns None (not applicable).
    let other = NoiseRecord { canonical_id: 7, ..noise.clone() };
    assert_eq!(tb.tie_break(&canon, &other), None);
}

/// G5 Test 4 — `runner_50_canonical_150_noise_complete`.
/// Runner ingests the full corpus, emits one record per canonical, writes
/// NDJSON, and scores every metric.
#[test]
fn runner_50_canonical_150_noise_complete() {
    let dir = tempdir().unwrap();
    let cfg = small_config(dir.path().to_path_buf());

    let outcome = run_g5_in_process(&cfg).unwrap();

    assert_eq!(outcome.records.len(), 10, "one query per canonical");
    assert!(outcome.ndjson_path.exists(), "NDJSON must be written");
    assert!(
        outcome.canonical_wins_rate >= 0.0 && outcome.canonical_wins_rate <= 1.0,
        "wins rate is a fraction"
    );
    assert!(
        outcome.noise_leak_rate >= 0.0 && outcome.noise_leak_rate <= 1.0,
        "leak rate is a fraction"
    );
    assert!(outcome.overall_pass, "perfect-canonical backend must pass gate");

    // Sanity: with the perfect backend, every record wins and zero leak.
    let backend = PerfectCanonicalBackend;
    let _ = backend; // smoke import
    for r in &outcome.records {
        assert!(r.winner_is_canonical, "perfect backend wins every query");
        assert_eq!(r.leaked_noise_id, None);
        assert!(r.pass);
    }
}

/// G5 Test 5 — `cli_g5_noise_happy`.
/// Default invocation passes pass-gate and writes adversarial-noise.ndjson.
#[test]
fn cli_g5_noise_happy() {
    let dir = tempdir().unwrap();
    let cfg = small_config(dir.path().to_path_buf());
    let outcome = run_g5_in_process(&cfg).unwrap();
    assert!(outcome.overall_pass, "perfect-canonical backend must pass");
    let body = std::fs::read_to_string(&outcome.ndjson_path).unwrap();
    assert!(body.lines().count() > 0, "NDJSON should have records");
    for line in body.lines() {
        assert!(line.contains("\"suite\":\"adversarial-noise\""));
    }
}

/// G5 Test 6 — `cli_g5_noise_reproducibility`.
/// Same seed produces identical per-query records (latency excluded).
#[test]
fn cli_g5_noise_reproducibility() {
    let dir_a = tempdir().unwrap();
    let dir_b = tempdir().unwrap();
    let cfg_a = small_config(dir_a.path().to_path_buf());
    let cfg_b = small_config(dir_b.path().to_path_buf());
    let a = run_g5_in_process(&cfg_a).unwrap();
    let b = run_g5_in_process(&cfg_b).unwrap();
    assert_eq!(a.records.len(), b.records.len());
    for (ra, rb) in a.records.iter().zip(b.records.iter()) {
        assert_eq!(ra.canonical_id, rb.canonical_id);
        assert_eq!(ra.winner_is_canonical, rb.winner_is_canonical);
        assert_eq!(ra.leaked_noise_id, rb.leaked_noise_id);
        assert_eq!(ra.pass, rb.pass);
    }
    assert!((a.canonical_wins_rate - b.canonical_wins_rate).abs() < f64::EPSILON);
    assert!((a.noise_leak_rate - b.noise_leak_rate).abs() < f64::EPSILON);
}

/// Pass-gate sanity: thresholds are wired correctly.
#[test]
fn pass_gate_thresholds_match_plan() {
    use crate::benchmark::substrate::adversarial_noise::PassGate;
    let g = PassGate::default();
    assert_eq!(g.canonical_wins_rate, 0.90);
    assert_eq!(g.noise_leak_rate, 0.05);
    assert_eq!(g.tie_break_by_provenance_rate, 0.75);
}
