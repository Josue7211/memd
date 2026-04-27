//! Episodic adapter scaffold (A6).
//!
//! Per-bench loaders convert public-bench turns into
//! `MemoryRecord { kind: Episodic, provenance: ... }` for ingestion.
//! Tasks A6.2–A6.6 fill in adapter bodies + provenance + round-trip.

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

/// Adapter trait implemented per bench under `bench_loaders/`.
pub(crate) trait EpisodicAdapter {
    type Turn;

    fn bench_id(&self) -> &'static str;
    fn next_turn(&mut self) -> Option<Self::Turn>;
    fn provenance(&self, turn: &Self::Turn) -> EpisodicProvenance;
}
