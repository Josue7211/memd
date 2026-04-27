//! V6 typed-ingest module root.
//!
//! A6 lands episodic adapters across the four public benches (LME, LoCoMo,
//! MemBench, ConvoMem). B6 layers semantic distillation on top; C6 promotes
//! to canonical. See `docs/phases/v6/phase-a6-plan.md` and
//! `docs/phases/v6/V6-INTEGRATION.md`.

pub(crate) mod episodic;
pub(crate) mod bench_loaders;
pub(crate) mod ingest_card;

pub(crate) use episodic::{EpisodicAdapter, EpisodicProvenance};

use std::path::Path;

use anyhow::{anyhow, Result};

use bench_loaders::{
    convomem::ConvomemAdapter, lme::LmeAdapter, locomo::LocomoAdapter,
    membench::MembenchAdapter,
};

/// Outcome of a typed-ingest dispatch — counts and provenance hashes
/// (deterministic enough for ingest-card baseline locks in A6.8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TypedIngestReport {
    pub bench_id: &'static str,
    pub turn_count: usize,
    pub session_count: usize,
}

/// Maps a public-benchmark dataset id to its episodic adapter and walks
/// the full turn stream, producing a `TypedIngestReport`. Pure: no
/// network, no server. Runtime activation (CLI → live ingest) graduates
/// in A6.9 once the calendar gate clears.
pub(crate) fn dispatch_typed_ingest_episodic(
    dataset_id: &str,
    path: &Path,
) -> Result<TypedIngestReport> {
    let mut sessions = std::collections::BTreeSet::<String>::new();
    let mut turn_count = 0usize;
    let bench_id: &'static str = match dataset_id {
        "longmemeval" => {
            let mut a = LmeAdapter::from_path(path)?;
            while let Some(t) = a.next_turn() {
                sessions.insert(t.provenance.session_id);
                turn_count += 1;
            }
            bench_loaders::lme::BENCH_ID
        }
        "locomo" => {
            let mut a = LocomoAdapter::from_path(path)?;
            while let Some(t) = a.next_turn() {
                sessions.insert(t.provenance.session_id);
                turn_count += 1;
            }
            bench_loaders::locomo::BENCH_ID
        }
        "membench" => {
            let mut a = MembenchAdapter::from_path(path)?;
            while let Some(t) = a.next_turn() {
                sessions.insert(t.provenance.session_id);
                turn_count += 1;
            }
            bench_loaders::membench::BENCH_ID
        }
        "convomem" => {
            let mut a = ConvomemAdapter::from_path(path)?;
            while let Some(t) = a.next_turn() {
                sessions.insert(t.provenance.session_id);
                turn_count += 1;
            }
            bench_loaders::convomem::BENCH_ID
        }
        other => {
            return Err(anyhow!(
                "typed-ingest=episodic does not support dataset `{}`",
                other
            ));
        }
    };
    Ok(TypedIngestReport {
        bench_id,
        turn_count,
        session_count: sessions.len(),
    })
}
