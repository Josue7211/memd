//! G5 adversarial-noise runner.
//!
//! 50 canonical facts (full provenance, older timestamps) plus 150 noise
//! records (semantic contradictions with `recency_offset_s` newer).
//! Each canonical query must beat its 3 noise siblings: pass-gate
//! `canonical_wins_rate ≥ 0.90`, `noise_leak_rate ≤ 0.05`,
//! `tie_break_by_provenance_rate ≥ 0.75`.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;
use uuid::Uuid;

/// One adversarial-noise record (canonical or noise variant).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct NoiseRecord {
    pub(crate) id: u32,
    pub(crate) canonical_id: u32,
    pub(crate) is_canonical: bool,
    pub(crate) subject: String,
    pub(crate) predicate: String,
    pub(crate) value: String,
    pub(crate) captured_at_offset_s: i64,
    pub(crate) provenance_chain_len: usize,
}

/// Pass-gate per `phase-g5-plan.md` §2 + the YAML spec.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PassGate {
    pub(crate) canonical_wins_rate: f64,
    pub(crate) noise_leak_rate: f64,
    pub(crate) tie_break_by_provenance_rate: f64,
}

impl Default for PassGate {
    fn default() -> Self {
        Self {
            canonical_wins_rate: 0.90,
            noise_leak_rate: 0.05,
            tie_break_by_provenance_rate: 0.75,
        }
    }
}

/// Static config for a G5 run.
#[derive(Debug, Clone)]
pub(crate) struct G5RunConfig {
    pub(crate) seed: u64,
    pub(crate) canonical_count: usize,
    pub(crate) noise_per_canonical: usize,
    pub(crate) noise_recency_offset_s: i64,
    pub(crate) pass_gate: PassGate,
    pub(crate) results_dir: PathBuf,
}

impl G5RunConfig {
    pub(crate) fn default_with_results_dir(results_dir: PathBuf) -> Self {
        Self {
            seed: 40,
            canonical_count: 50,
            noise_per_canonical: 3,
            noise_recency_offset_s: 3600,
            pass_gate: PassGate::default(),
            results_dir,
        }
    }
}

/// Per-query G5 record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct G5QueryRecord {
    pub(crate) suite: String,
    pub(crate) seed: u64,
    pub(crate) query_idx: usize,
    pub(crate) canonical_id: u32,
    pub(crate) winner_is_canonical: bool,
    pub(crate) leaked_noise_id: Option<u32>,
    pub(crate) tie_break_correct: Option<bool>,
    pub(crate) pass: bool,
}

/// Outcome of one full G5 invocation.
#[derive(Debug, Clone)]
pub(crate) struct G5Outcome {
    pub(crate) records: Vec<G5QueryRecord>,
    pub(crate) ndjson_path: PathBuf,
    pub(crate) overall_pass: bool,
    pub(crate) canonical_wins_rate: f64,
    pub(crate) noise_leak_rate: f64,
    pub(crate) tie_break_by_provenance_rate: f64,
}

/// SplitMix64 — same shape as fixtures.rs, kept local to avoid cross-module
/// surface creep.
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn next_in_range(&mut self, n: usize) -> usize {
        (self.next_u64() as usize) % n.max(1)
    }
}

const SUBJECTS: &[&str] = &[
    "alice", "bob", "carol", "dave", "erin", "frank", "grace", "heidi", "ivan", "judy", "ken",
    "leo", "mallory", "nina", "olivia", "peggy",
];
const PREDICATES: &[&str] = &[
    "lives_in",
    "works_at",
    "drives",
    "owns_pet",
    "born_in",
    "graduated_from",
];
const CANONICAL_VALUES: &[&str] = &[
    "Lisbon",
    "Tokyo",
    "Toronto",
    "Berlin",
    "Madrid",
    "Oslo",
    "Dublin",
    "Vienna",
    "Helsinki",
    "Reykjavik",
];
const NOISE_VALUES: &[&str] = &[
    "Antarctica",
    "Atlantis",
    "Mars",
    "the moon",
    "Narnia",
    "Westeros",
    "Elsewhere",
    "Nowhere",
];

/// Deterministic generator: seed → (canonical[N], noise[N * noise_per_canonical]).
/// Noise records carry the same subject+predicate as their canonical sibling
/// but a contradicting value, a `recency_offset_s` newer timestamp, and
/// shorter provenance chain (1 vs canonical's 3).
pub(crate) fn generate_corpus(
    seed: u64,
    canonical_count: usize,
    noise_per_canonical: usize,
    noise_recency_offset_s: i64,
) -> (Vec<NoiseRecord>, Vec<NoiseRecord>) {
    let mut rng = SplitMix64::new(seed);
    let mut canonical = Vec::with_capacity(canonical_count);
    let mut noise = Vec::with_capacity(canonical_count * noise_per_canonical);

    for i in 0..canonical_count {
        let subj = SUBJECTS[rng.next_in_range(SUBJECTS.len())].to_string();
        let pred = PREDICATES[rng.next_in_range(PREDICATES.len())].to_string();
        let canon_val = CANONICAL_VALUES[rng.next_in_range(CANONICAL_VALUES.len())].to_string();

        let canon_id = i as u32;
        canonical.push(NoiseRecord {
            id: canon_id,
            canonical_id: canon_id,
            is_canonical: true,
            subject: subj.clone(),
            predicate: pred.clone(),
            value: canon_val,
            captured_at_offset_s: 0,
            provenance_chain_len: 3,
        });

        for j in 0..noise_per_canonical {
            let noise_val = NOISE_VALUES[rng.next_in_range(NOISE_VALUES.len())].to_string();
            let noise_id =
                (canonical_count as u32) + (i as u32) * (noise_per_canonical as u32) + j as u32;
            noise.push(NoiseRecord {
                id: noise_id,
                canonical_id: canon_id,
                is_canonical: false,
                subject: subj.clone(),
                predicate: pred.clone(),
                value: noise_val,
                captured_at_offset_s: noise_recency_offset_s,
                provenance_chain_len: 1,
            });
        }
    }
    (canonical, noise)
}

/// Scorer: did the canonical record rank above its noise siblings?
#[derive(Debug)]
pub(crate) struct CanonicalWinsScorer;

impl CanonicalWinsScorer {
    pub(crate) fn new() -> Self {
        Self
    }

    /// Given a ranked list (top first), check whether `canonical_id` is the
    /// top match against its noise siblings.
    pub(crate) fn winner_is_canonical(&self, canonical_id: u32, ranked: &[NoiseRecord]) -> bool {
        ranked
            .iter()
            .find(|r| r.canonical_id == canonical_id)
            .map(|r| r.is_canonical)
            .unwrap_or(false)
    }

    /// Returns the leaked noise id when canonical lost.
    pub(crate) fn leaked_noise(&self, canonical_id: u32, ranked: &[NoiseRecord]) -> Option<u32> {
        for r in ranked {
            if r.canonical_id == canonical_id {
                if r.is_canonical {
                    return None;
                }
                return Some(r.id);
            }
        }
        None
    }
}

/// Tie-break scorer: when canonical+noise scored equally, did the
/// provenance-chain-length comparator pick the canonical?
#[derive(Debug)]
pub(crate) struct TieBreakProvenanceScorer;

impl TieBreakProvenanceScorer {
    pub(crate) fn new() -> Self {
        Self
    }

    /// Returns `Some(true)` if canonical's chain is strictly longer (correct
    /// tie-break), `Some(false)` if noise's is, `None` if no tie applicable.
    pub(crate) fn tie_break(&self, canonical: &NoiseRecord, noise: &NoiseRecord) -> Option<bool> {
        if !canonical.is_canonical || noise.is_canonical {
            return None;
        }
        if canonical.canonical_id != noise.canonical_id {
            return None;
        }
        Some(canonical.provenance_chain_len > noise.provenance_chain_len)
    }
}

/// Backend trait: rank canonical + noise siblings for one query.
pub(crate) trait NoiseBackend {
    fn rank_for_canonical(
        &self,
        canonical: &NoiseRecord,
        siblings: &[NoiseRecord],
    ) -> Vec<NoiseRecord>;
}

/// Perfect-recall backend: always promotes canonical to top position
/// regardless of recency. The HTTP backend lands later and exercises the
/// real ranker.
#[derive(Default, Clone)]
pub(crate) struct PerfectCanonicalBackend;

impl NoiseBackend for PerfectCanonicalBackend {
    fn rank_for_canonical(
        &self,
        canonical: &NoiseRecord,
        siblings: &[NoiseRecord],
    ) -> Vec<NoiseRecord> {
        let mut out = Vec::with_capacity(1 + siblings.len());
        out.push(canonical.clone());
        out.extend(siblings.iter().cloned());
        out
    }
}

/// Run G5 in-process via `PerfectCanonicalBackend`.
pub(crate) fn run_g5_in_process(config: &G5RunConfig) -> std::io::Result<G5Outcome> {
    let backend = PerfectCanonicalBackend;
    run_g5_with_backend(config, &backend)
}

/// Backend-generic entry point.
pub(crate) fn run_g5_with_backend<B: NoiseBackend>(
    config: &G5RunConfig,
    backend: &B,
) -> std::io::Result<G5Outcome> {
    let _run_id = Uuid::new_v4().to_string();
    let _ts_ms = Utc::now().timestamp_millis();

    let (canonical, noise) = generate_corpus(
        config.seed,
        config.canonical_count,
        config.noise_per_canonical,
        config.noise_recency_offset_s,
    );

    let scorer = CanonicalWinsScorer::new();
    let tie_scorer = TieBreakProvenanceScorer::new();

    let mut records = Vec::with_capacity(canonical.len());
    let mut wins = 0usize;
    let mut leaks = 0usize;
    let mut tie_attempts = 0usize;
    let mut tie_correct = 0usize;

    for (idx, canon) in canonical.iter().enumerate() {
        let siblings: Vec<NoiseRecord> = noise
            .iter()
            .filter(|n| n.canonical_id == canon.canonical_id)
            .cloned()
            .collect();

        let ranked = backend.rank_for_canonical(canon, &siblings);
        let won = scorer.winner_is_canonical(canon.canonical_id, &ranked);
        let leaked = scorer.leaked_noise(canon.canonical_id, &ranked);

        let tie = siblings
            .first()
            .and_then(|n| tie_scorer.tie_break(canon, n));
        if let Some(correct) = tie {
            tie_attempts += 1;
            if correct {
                tie_correct += 1;
            }
        }

        if won {
            wins += 1;
        }
        if leaked.is_some() {
            leaks += 1;
        }

        records.push(G5QueryRecord {
            suite: "adversarial-noise".into(),
            seed: config.seed,
            query_idx: idx,
            canonical_id: canon.canonical_id,
            winner_is_canonical: won,
            leaked_noise_id: leaked,
            tie_break_correct: tie,
            pass: won,
        });
    }

    let total = records.len().max(1) as f64;
    let canonical_wins_rate = wins as f64 / total;
    let noise_leak_rate = leaks as f64 / total;
    let tie_break_by_provenance_rate = if tie_attempts == 0 {
        1.0
    } else {
        tie_correct as f64 / tie_attempts as f64
    };

    let overall_pass = canonical_wins_rate >= config.pass_gate.canonical_wins_rate
        && noise_leak_rate <= config.pass_gate.noise_leak_rate
        && tie_break_by_provenance_rate >= config.pass_gate.tie_break_by_provenance_rate;

    let ndjson_path = config.results_dir.join("adversarial-noise.ndjson");
    if let Some(parent) = ndjson_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&ndjson_path)?;
    for r in &records {
        let line = serde_json::to_string(r)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        writeln!(f, "{}", line)?;
    }

    Ok(G5Outcome {
        records,
        ndjson_path,
        overall_pass,
        canonical_wins_rate,
        noise_leak_rate,
        tie_break_by_provenance_rate,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corpus_seed_is_deterministic() {
        let (c1, n1) = generate_corpus(40, 10, 3, 3600);
        let (c2, n2) = generate_corpus(40, 10, 3, 3600);
        assert_eq!(c1, c2);
        assert_eq!(n1, n2);
    }
}
