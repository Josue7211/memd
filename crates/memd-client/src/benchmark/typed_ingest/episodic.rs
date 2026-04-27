//! Episodic adapter contract (A6).
//!
//! Per-bench loaders convert public-bench turns into typed
//! `EpisodicTurn { content, provenance }` records for ingestion.
//! Schema policy (per `phase-a6-plan.md` §2 + advisor reconcile):
//! `MemoryKind::Episodic` does NOT exist on `memd-schema`. Episodic is an
//! adapter-layer concept; ingestion lands turns as `MemoryKind::Fact` plus
//! the `EpisodicProvenance` sidecar carried in record metadata.

use serde::{Deserialize, Serialize};

/// Provenance carried with every episodic ingest. Required fields per
/// `phase-a6-plan.md` §2.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct EpisodicProvenance {
    pub bench_id: String,
    pub session_id: String,
    pub turn_index: u32,
    pub speaker: String,
    pub source_hash: String,
    pub captured_at: String,
}

/// Single typed turn yielded by an `EpisodicAdapter`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct EpisodicTurn {
    pub content: String,
    pub provenance: EpisodicProvenance,
}

/// Adapter trait implemented per bench under `bench_loaders/`.
pub(crate) trait EpisodicAdapter {
    fn bench_id(&self) -> &'static str;
    fn next_turn(&mut self) -> Option<EpisodicTurn>;
}
