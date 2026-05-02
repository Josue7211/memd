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
use memd_schema::{DedupCluster, DedupDuplicate, DedupScanRequest, DedupScanResponse};
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

impl SqliteStore {
    /// E3-D5: single-pass cluster scan for the CLI dry-run. Loads every
    /// vector in scope, walks pairwise cosine and groups anything within
    /// `threshold` into a cluster seeded by the first-seen vector. The
    /// richest (highest confidence, break ties by updated_at desc) wins
    /// the survivor slot.
    ///
    /// This is O(n²) in vectors_in_scope — fine for the current ~10-50k
    /// ceiling on a single bundle. If that changes, swap for an ANN
    /// index; the API contract stays the same.
    pub fn scan_duplicates(
        &self,
        req: &DedupScanRequest,
        embedding_model: &str,
    ) -> anyhow::Result<DedupScanResponse> {
        let threshold = req
            .threshold_cosine_distance
            .unwrap_or(DEFAULT_DEDUP_COSINE_DISTANCE);
        let limit = req.limit.unwrap_or(50).min(500);
        let min_sim = 1.0 - threshold;

        let rows = self
            .list_vectors_for_scope(
                req.project.as_deref(),
                req.namespace.as_deref(),
                embedding_model,
            )
            .context("list vectors for dedup scan")?;
        let vectors: Vec<(Uuid, Vec<f32>)> = rows
            .into_iter()
            .map(|(id, bytes)| (id, bytes_to_vec(&bytes)))
            .collect();
        let vectors_scanned = vectors.len();

        let mut clusters: Vec<Vec<(Uuid, f32)>> = Vec::new();
        let mut assigned: std::collections::HashSet<Uuid> = std::collections::HashSet::new();

        for i in 0..vectors.len() {
            if assigned.contains(&vectors[i].0) {
                continue;
            }
            let mut group: Vec<(Uuid, f32)> = vec![(vectors[i].0, 1.0)];
            assigned.insert(vectors[i].0);
            for j in (i + 1)..vectors.len() {
                if assigned.contains(&vectors[j].0) {
                    continue;
                }
                if vectors[i].1.len() != vectors[j].1.len() {
                    continue;
                }
                let sim = cosine_on_unit(&vectors[i].1, &vectors[j].1);
                if sim >= min_sim {
                    group.push((vectors[j].0, sim));
                    assigned.insert(vectors[j].0);
                }
            }
            if group.len() >= 2 {
                clusters.push(group);
                if clusters.len() >= limit {
                    break;
                }
            }
        }

        let mut out = Vec::with_capacity(clusters.len());
        for group in clusters {
            let mut items: Vec<(memd_schema::MemoryItem, f32)> = Vec::with_capacity(group.len());
            for (id, sim) in group {
                if let Some(item) = self.get(id)? {
                    items.push((item, sim));
                }
            }
            if items.len() < 2 {
                continue;
            }
            items.sort_by(|a, b| {
                b.0.confidence
                    .partial_cmp(&a.0.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| b.0.updated_at.cmp(&a.0.updated_at))
            });
            let survivor = items.remove(0).0;
            let duplicates: Vec<DedupDuplicate> = items
                .into_iter()
                .map(|(it, sim)| DedupDuplicate {
                    id: it.id,
                    similarity: sim,
                    preview: preview_for(&it.content),
                })
                .collect();
            out.push(DedupCluster {
                survivor_id: survivor.id,
                survivor_preview: preview_for(&survivor.content),
                duplicates,
            });
        }

        Ok(DedupScanResponse {
            clusters: out,
            vectors_scanned,
            threshold_cosine_distance: threshold,
        })
    }
}

fn preview_for(content: &str) -> String {
    let trimmed = content.trim();
    let cap = 120;
    if trimmed.chars().count() <= cap {
        trimmed.to_string()
    } else {
        let head: String = trimmed.chars().take(cap).collect();
        format!("{head}…")
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
            correction_meta: None,
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
        if n == 0.0 {
            v.to_vec()
        } else {
            v.iter().map(|x| x / n).collect()
        }
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
