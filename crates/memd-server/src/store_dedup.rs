//! E3-D1: storage-time near-duplicate detection via embedding cosine.
//!
//! Complements the canonical/redundancy-key dedup in `insert_or_get_duplicate`:
//! those catch exact-match dups; this catches paraphrases (e.g. "used Cargo"
//! vs. "we use Cargo for builds") whose hashes diverge but whose embeddings
//! collide.
//!
//! Contract:
//!   - vectors in `memory_vectors` are unit-normalized (see `embed_batch_normalized`),
//!     so cosine similarity == dot product.
//!   - "0.15 cosine distance" means similarity ≥ 0.85.
//!   - scope filter is (project, namespace, embedding_model) — dedup never
//!     crosses projects or embedding models.
//!
//! Used by:
//!   - store_item ingest path when `MEMD_STORE_DEDUP=1`
//!   - `memd dedup --dry-run` CLI (E3-D5) via `scan_duplicates`

use anyhow::Context;
use uuid::Uuid;

use crate::embed::{bytes_to_vec, cosine_on_unit};
use crate::store::SqliteStore;

/// Default cosine-distance threshold for storage-time dedup.
/// Tuned on LoCoMo paraphrase pairs; lower → stricter.
pub const DEFAULT_DEDUP_COSINE_DISTANCE: f32 = 0.15;

#[derive(Debug, Clone)]
pub struct NearDuplicate {
    pub existing_id: Uuid,
    pub similarity: f32,
}

impl SqliteStore {
    /// Return the highest-similarity existing vector in scope whose cosine
    /// distance to `incoming` is ≤ `threshold`. Returns None if nothing in
    /// scope clears the bar.
    pub fn find_near_duplicate(
        &self,
        project: Option<&str>,
        namespace: Option<&str>,
        embedding_model: &str,
        incoming: &[f32],
        threshold_distance: f32,
    ) -> anyhow::Result<Option<NearDuplicate>> {
        let min_sim = 1.0 - threshold_distance;
        let rows = self
            .list_vectors_for_scope(project, namespace, embedding_model)
            .context("list candidate vectors for dedup")?;

        let mut best: Option<NearDuplicate> = None;
        for (id, bytes) in rows {
            let v = bytes_to_vec(&bytes);
            if v.len() != incoming.len() {
                continue;
            }
            let sim = cosine_on_unit(incoming, &v);
            if sim < min_sim {
                continue;
            }
            if best.as_ref().is_none_or(|b| sim > b.similarity) {
                best = Some(NearDuplicate {
                    existing_id: id,
                    similarity: sim,
                });
            }
        }
        Ok(best)
    }
}

/// Return true when the operator has opted into storage-time cosine dedup.
/// Default off — the feature requires a live embedder and adds a vector
/// scan per write, so we gate it until the CLI dry-run (D5) has been used
/// to inspect impact on an existing bundle.
pub(crate) fn store_dedup_enabled() -> bool {
    match std::env::var("MEMD_STORE_DEDUP") {
        Ok(v) => {
            let v = v.trim().to_ascii_lowercase();
            matches!(v.as_str(), "1" | "true" | "on" | "yes")
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::SqliteStore;
    use chrono::Utc;
    use memd_schema::{
        MemoryItem, MemoryKind, MemoryScope, MemoryStage, MemoryStatus, MemoryVisibility,
        SourceQuality,
    };
    use tempfile::NamedTempFile;

    fn seed_item(store: &SqliteStore, content: &str) -> MemoryItem {
        let now = Utc::now();
        let item = MemoryItem {
            id: Uuid::new_v4(),
            content: content.into(),
            redundancy_key: None,
            belief_branch: None,
            preferred: false,
            kind: MemoryKind::Fact,
            scope: MemoryScope::Project,
            project: Some("p".into()),
            namespace: Some("n".into()),
            workspace: None,
            visibility: MemoryVisibility::default(),
            source_agent: None,
            source_system: None,
            source_path: None,
            confidence: 0.7,
            ttl_seconds: None,
            created_at: now,
            updated_at: now,
            last_verified_at: None,
            supersedes: Vec::new(),
            tags: Vec::new(),
            status: MemoryStatus::Active,
            source_quality: Some(SourceQuality::Canonical),
            stage: MemoryStage::Canonical,
            lane: None,
            version: 1,
        };
        let ck = crate::keys::canonical_key(&item);
        let rk = crate::keys::redundancy_key(&item);
        store
            .insert_or_get_duplicate(&item, &ck, &rk)
            .expect("insert seed");
        item
    }

    fn unit(v: &[f32]) -> Vec<f32> {
        let n: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        if n == 0.0 { v.to_vec() } else { v.iter().map(|x| x / n).collect() }
    }

    #[test]
    fn near_duplicate_returns_highest_similarity_match() {
        let tmp = NamedTempFile::new().unwrap();
        let store = SqliteStore::open(tmp.path()).unwrap();

        let seed = seed_item(&store, "cargo is the build tool");
        let v_seed = unit(&[1.0, 0.0, 0.0, 0.0]);
        store
            .replace_memory_vector_chunks(
                seed.id,
                Some("p"),
                Some("n"),
                "test-model",
                v_seed.len(),
                &[(0, crate::embed::vec_to_bytes(&v_seed))],
            )
            .expect("seed vec");

        // incoming: tiny angle vs. seed → high cosine, ~0 distance
        let incoming = unit(&[0.99, 0.05, 0.0, 0.0]);
        let hit = store
            .find_near_duplicate(Some("p"), Some("n"), "test-model", &incoming, 0.15)
            .expect("find");
        let hit = hit.expect("expected near-dup");
        assert_eq!(hit.existing_id, seed.id);
        assert!(hit.similarity > 0.99);
    }

    #[test]
    fn far_vector_is_not_a_duplicate() {
        let tmp = NamedTempFile::new().unwrap();
        let store = SqliteStore::open(tmp.path()).unwrap();

        let seed = seed_item(&store, "unrelated");
        let v_seed = unit(&[1.0, 0.0, 0.0, 0.0]);
        store
            .replace_memory_vector_chunks(
                seed.id,
                Some("p"),
                Some("n"),
                "test-model",
                v_seed.len(),
                &[(0, crate::embed::vec_to_bytes(&v_seed))],
            )
            .expect("seed vec");

        // Orthogonal → similarity 0 → distance 1.0, well past 0.15
        let incoming = unit(&[0.0, 1.0, 0.0, 0.0]);
        let hit = store
            .find_near_duplicate(Some("p"), Some("n"), "test-model", &incoming, 0.15)
            .expect("find");
        assert!(hit.is_none());
    }

    #[test]
    fn threshold_zero_allows_only_exact_match() {
        let tmp = NamedTempFile::new().unwrap();
        let store = SqliteStore::open(tmp.path()).unwrap();

        let seed = seed_item(&store, "exact");
        let v_seed = unit(&[0.6, 0.8, 0.0, 0.0]);
        store
            .replace_memory_vector_chunks(
                seed.id,
                Some("p"),
                Some("n"),
                "test-model",
                v_seed.len(),
                &[(0, crate::embed::vec_to_bytes(&v_seed))],
            )
            .expect("seed vec");

        // similarity < 1.0 (small perturbation)
        let incoming = unit(&[0.6, 0.8, 0.01, 0.0]);
        let hit = store
            .find_near_duplicate(Some("p"), Some("n"), "test-model", &incoming, 0.0)
            .unwrap();
        assert!(hit.is_none());

        // exact match → similarity == 1.0
        let hit = store
            .find_near_duplicate(Some("p"), Some("n"), "test-model", &v_seed, 0.0)
            .unwrap()
            .expect("exact dup");
        assert!((hit.similarity - 1.0).abs() < 1e-4);
    }
}
