//! V6 typed-ingest module root.
//!
//! A6 lands episodic adapters across the four public benches (LME, LoCoMo,
//! MemBench, ConvoMem). B6 layers semantic distillation on top; C6 promotes
//! to canonical. See `docs/phases/v6/phase-a6-plan.md` and
//! `docs/phases/v6/V6-INTEGRATION.md`.

pub(crate) mod episodic;
pub(crate) mod bench_loaders;

pub(crate) use episodic::{EpisodicAdapter, EpisodicProvenance};
