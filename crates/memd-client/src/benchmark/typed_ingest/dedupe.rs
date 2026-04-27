//! V6 / B6 — candidate dedupe. Hash-first, cosine when an embedder is
//! plumbed. Contract: `docs/contracts/semantic-distillation.md` §5.

use sha2::{Digest, Sha256};

use super::distiller::DistillCandidate;

/// Stable content hash. Lowercase + trim before hashing so trivial
/// case/whitespace variants collapse.
pub(crate) fn content_hash(content: &str) -> String {
    let normalized = content.trim().to_lowercase();
    let mut h = Sha256::new();
    h.update(normalized.as_bytes());
    format!("{:x}", h.finalize())
}

/// Cosine similarity on unit vectors. Mirror of memd-server's
/// `cosine_on_unit` — kept local to avoid a cross-crate dep just for
/// the math. Inputs assumed normalised; if not, normalise first.
pub(crate) fn cosine_on_unit(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Default cosine-similarity threshold above which two candidates count
/// as near-duplicates. Mirror of the storage-time threshold (0.85).
pub(crate) const COSINE_NEAR_DUPLICATE: f32 = 0.85;

/// Result of running dedupe on a batch.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DedupeReport {
    pub kept: Vec<DistillCandidate>,
    pub collapsed_hash: usize,
    pub collapsed_cosine: usize,
}

/// Hash-only dedupe: drop later candidates whose normalised content
/// hash matches one already kept. O(n) with a HashSet.
pub(crate) fn dedupe_hash(candidates: Vec<DistillCandidate>) -> DedupeReport {
    let mut seen = std::collections::HashSet::<String>::new();
    let mut kept = Vec::with_capacity(candidates.len());
    let mut collapsed = 0usize;
    for c in candidates {
        let h = content_hash(&c.content);
        if seen.insert(h) {
            kept.push(c);
        } else {
            collapsed += 1;
        }
    }
    DedupeReport {
        kept,
        collapsed_hash: collapsed,
        collapsed_cosine: 0,
    }
}

/// Hash + cosine dedupe. `embeddings` must have the same length as
/// `candidates` and be unit-normalised. Drops candidates whose hash
/// matches a previously kept one OR whose cosine similarity to any
/// previously kept embedding ≥ `COSINE_NEAR_DUPLICATE`.
pub(crate) fn dedupe_hash_cosine(
    candidates: Vec<DistillCandidate>,
    embeddings: Vec<Vec<f32>>,
) -> DedupeReport {
    assert_eq!(candidates.len(), embeddings.len(), "embedding count mismatch");
    let mut seen_hash = std::collections::HashSet::<String>::new();
    let mut kept_embeddings: Vec<Vec<f32>> = Vec::new();
    let mut kept = Vec::with_capacity(candidates.len());
    let mut collapsed_hash = 0usize;
    let mut collapsed_cosine = 0usize;

    for (c, emb) in candidates.into_iter().zip(embeddings.into_iter()) {
        let h = content_hash(&c.content);
        if !seen_hash.insert(h) {
            collapsed_hash += 1;
            continue;
        }
        let mut near_dupe = false;
        for prior in &kept_embeddings {
            if cosine_on_unit(prior, &emb) >= COSINE_NEAR_DUPLICATE {
                near_dupe = true;
                break;
            }
        }
        if near_dupe {
            collapsed_cosine += 1;
            continue;
        }
        kept.push(c);
        kept_embeddings.push(emb);
    }
    DedupeReport {
        kept,
        collapsed_hash,
        collapsed_cosine,
    }
}
