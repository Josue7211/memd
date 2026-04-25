//! A5 deterministic fixture generator.
//!
//! Substrate benches must be reproducible across machines and across CI
//! runs. We pin the corpus shape to a single `seed: u64` and an inline
//! SplitMix64 PRNG (no `rand` crate dep — keeping the bench surface
//! self-contained). Same seed + same `KindMix` + same `count` ⇒ identical
//! `Vec<Fact>` byte-for-byte.

use serde::{Deserialize, Serialize};

/// Three fact kinds A5 cares about. The mix ratios let us tune what kind
/// of recall pressure the bench applies (canonical = stable, preference
/// = drifty, semantic = paraphrase).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum FactKind {
    Canonical,
    Semantic,
    Preference,
}

impl FactKind {
    fn as_str(self) -> &'static str {
        match self {
            FactKind::Canonical => "canonical",
            FactKind::Semantic => "semantic",
            FactKind::Preference => "preference",
        }
    }
}

/// Mix ratios. Sum need not be exactly 1.0 — generator normalises.
/// Defaults match `phase-a5-plan.md` §2: canonical 0.5, semantic 0.3,
/// preference 0.2.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct KindMix {
    pub(crate) canonical: f64,
    pub(crate) semantic: f64,
    pub(crate) preference: f64,
}

impl Default for KindMix {
    fn default() -> Self {
        Self {
            canonical: 0.5,
            semantic: 0.3,
            preference: 0.2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Fact {
    pub(crate) id: u32,
    pub(crate) kind: String,
    pub(crate) subject: String,
    pub(crate) predicate: String,
    pub(crate) value: String,
}

/// SplitMix64 — small, fast, well-distributed, deterministic.
/// Used because we don't want a `rand` crate dependency for one bench.
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
        // Modulo bias is acceptable for word-table indices (small `n`).
        (self.next_u64() as usize) % n
    }

    fn next_f64(&mut self) -> f64 {
        // 53-bit mantissa via shift, divides into [0, 1).
        let bits = self.next_u64() >> 11;
        bits as f64 / (1u64 << 53) as f64
    }
}

const SUBJECTS: &[&str] = &[
    "alice", "bob", "carol", "dave", "erin", "frank", "grace", "heidi",
    "ivan", "judy", "ken", "leo", "mallory", "nina", "olivia", "peggy",
];

const PREDICATES_CANONICAL: &[&str] = &[
    "lives_in",
    "works_at",
    "drives",
    "owns_pet",
    "born_in",
    "graduated_from",
];

const PREDICATES_SEMANTIC: &[&str] = &[
    "describes",
    "summarises",
    "narrates",
    "explains",
    "frames",
];

const PREDICATES_PREFERENCE: &[&str] = &[
    "prefers",
    "likes",
    "dislikes",
    "avoids",
    "tolerates",
];

const VALUES_PLACE: &[&str] = &[
    "berlin", "tokyo", "lima", "lisbon", "oslo", "kyoto", "porto", "ankara",
];

const VALUES_VEHICLE: &[&str] = &["volvo", "tesla", "ford", "audi", "honda", "bmw", "fiat"];

const VALUES_PET: &[&str] = &["beagle", "tabby", "parakeet", "ferret", "iguana"];

const VALUES_TOPIC: &[&str] = &[
    "logistics",
    "biochemistry",
    "macroeconomics",
    "carpentry",
    "linguistics",
];

const VALUES_FOOD: &[&str] = &[
    "udon", "kimchi", "halloumi", "tempeh", "seitan", "kombucha", "miso",
];

fn pick<'a, R>(rng: &mut SplitMix64, table: &'a [&'a str]) -> &'a str
where
    R: ?Sized,
{
    table[rng.next_in_range(table.len())]
}

fn build_fact(rng: &mut SplitMix64, id: u32, kind: FactKind) -> Fact {
    let subject = pick::<()>(rng, SUBJECTS).to_string();
    let (predicate, value) = match kind {
        FactKind::Canonical => {
            let p = pick::<()>(rng, PREDICATES_CANONICAL);
            let v = match p {
                "lives_in" | "born_in" | "graduated_from" => pick::<()>(rng, VALUES_PLACE),
                "drives" => pick::<()>(rng, VALUES_VEHICLE),
                "owns_pet" => pick::<()>(rng, VALUES_PET),
                "works_at" => pick::<()>(rng, VALUES_PLACE),
                _ => pick::<()>(rng, VALUES_PLACE),
            };
            (p.to_string(), v.to_string())
        }
        FactKind::Semantic => (
            pick::<()>(rng, PREDICATES_SEMANTIC).to_string(),
            pick::<()>(rng, VALUES_TOPIC).to_string(),
        ),
        FactKind::Preference => (
            pick::<()>(rng, PREDICATES_PREFERENCE).to_string(),
            pick::<()>(rng, VALUES_FOOD).to_string(),
        ),
    };
    Fact {
        id,
        kind: kind.as_str().to_string(),
        subject,
        predicate,
        value,
    }
}

/// Returns `count` facts for `(seed, mix)`. Same inputs ⇒ same `Vec<Fact>`.
///
/// The kind for fact `i` is drawn from a normalised `mix` via the same
/// PRNG stream that fills the fact's fields, so reordering or changing
/// `count` shifts the corpus — that is intentional: scenario size is part
/// of the fixture identity.
pub(crate) fn generate_corpus(seed: u64, count: usize, mix: &KindMix) -> Vec<Fact> {
    let total = (mix.canonical + mix.semantic + mix.preference).max(f64::EPSILON);
    let canonical_p = mix.canonical / total;
    let semantic_p = mix.semantic / total;
    // preference fills the remainder.

    let mut rng = SplitMix64::new(seed);
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let r = rng.next_f64();
        let kind = if r < canonical_p {
            FactKind::Canonical
        } else if r < canonical_p + semantic_p {
            FactKind::Semantic
        } else {
            FactKind::Preference
        };
        out.push(build_fact(&mut rng, i as u32, kind));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test 1 — `fixtures_generate_deterministic_corpus_for_fixed_seed`.
    /// Same seed + count + mix ⇒ byte-identical JSON serialisation.
    #[test]
    fn fixtures_generate_deterministic_corpus_for_fixed_seed() {
        let mix = KindMix::default();
        let a = generate_corpus(42, 50, &mix);
        let b = generate_corpus(42, 50, &mix);
        assert_eq!(a, b, "same (seed, count, mix) must produce identical Vec<Fact>");

        let a_bytes = serde_json::to_vec(&a).unwrap();
        let b_bytes = serde_json::to_vec(&b).unwrap();
        assert_eq!(
            a_bytes, b_bytes,
            "JSON serialisation must be byte-identical across calls"
        );

        // Different seed should change the corpus.
        let c = generate_corpus(43, 50, &mix);
        assert_ne!(a, c, "different seed must produce a different corpus");
    }

    /// Test 2 — `fixtures_mix_respects_kind_ratios`.
    /// At a large N, kind frequencies should be within a generous
    /// tolerance of the configured mix ratios.
    #[test]
    fn fixtures_mix_respects_kind_ratios() {
        let mix = KindMix {
            canonical: 0.5,
            semantic: 0.3,
            preference: 0.2,
        };
        let n = 4000;
        let corpus = generate_corpus(42, n, &mix);

        let mut canonical = 0;
        let mut semantic = 0;
        let mut preference = 0;
        for f in &corpus {
            match f.kind.as_str() {
                "canonical" => canonical += 1,
                "semantic" => semantic += 1,
                "preference" => preference += 1,
                other => panic!("unknown kind {other}"),
            }
        }

        let n_f = n as f64;
        let canonical_ratio = canonical as f64 / n_f;
        let semantic_ratio = semantic as f64 / n_f;
        let preference_ratio = preference as f64 / n_f;
        let tol = 0.04; // ±4 percentage points at N=4000.

        assert!(
            (canonical_ratio - mix.canonical).abs() < tol,
            "canonical ratio {canonical_ratio:.3} not within {tol} of {}",
            mix.canonical
        );
        assert!(
            (semantic_ratio - mix.semantic).abs() < tol,
            "semantic ratio {semantic_ratio:.3} not within {tol} of {}",
            mix.semantic
        );
        assert!(
            (preference_ratio - mix.preference).abs() < tol,
            "preference ratio {preference_ratio:.3} not within {tol} of {}",
            mix.preference
        );
    }

    #[test]
    fn fact_ids_are_dense_and_sequential() {
        let corpus = generate_corpus(42, 16, &KindMix::default());
        for (i, f) in corpus.iter().enumerate() {
            assert_eq!(f.id, i as u32);
        }
    }
}
