//! A6 integration tests — `--typed-ingest=episodic` round-trip across
//! the four public benches (LME, LoCoMo, MemBench, ConvoMem). Tests
//! 1–10 per `phase-a6-plan.md` §4. Bodies land in tasks A6.2–A6.8.

#[allow(unused_imports)]
use crate::benchmark::typed_ingest::{EpisodicAdapter, EpisodicProvenance};
