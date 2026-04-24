//! Wake-Context Compiler (D4)
//!
//! Pure transform inserted between retrieval and render. Takes typed
//! buckets, applies priority, dedupes across buckets, enforces a hard
//! budget, demotes overflow to `memd lookup` hints. No storage changes.
//!
//! Budget unit: chars (matches existing `compute_wake_token_metrics`
//! infra). The plan and env vars use the word "tokens" for legacy
//! compatibility; under the hood it is a char count.

#![allow(dead_code)] // scaffolded D4.1; filled in D4.2..D4.6

use std::collections::HashMap;

use memd_schema::CompactMemoryRecord;

pub mod buckets;
pub mod budget;
pub mod dedupe;
pub mod priority;
pub mod render;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BucketKind {
    Canonical,
    Preference,
    Focus,
    Episodic,
    Semantic,
    Correction,
    Candidate,
}

impl BucketKind {
    pub const ALL: [BucketKind; 7] = [
        BucketKind::Canonical,
        BucketKind::Preference,
        BucketKind::Focus,
        BucketKind::Correction,
        BucketKind::Episodic,
        BucketKind::Semantic,
        BucketKind::Candidate,
    ];

    pub fn label(self) -> &'static str {
        match self {
            BucketKind::Canonical => "canonical",
            BucketKind::Preference => "preference",
            BucketKind::Focus => "focus",
            BucketKind::Episodic => "episodic",
            BucketKind::Semantic => "semantic",
            BucketKind::Correction => "correction",
            BucketKind::Candidate => "candidate",
        }
    }

    pub fn section_header(self) -> &'static str {
        match self {
            BucketKind::Canonical => "Durable Truth",
            BucketKind::Preference => "Preferences",
            BucketKind::Focus => "Focus",
            BucketKind::Episodic => "Episodic",
            BucketKind::Semantic => "Semantic",
            BucketKind::Correction => "Corrections",
            BucketKind::Candidate => "Candidates",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CompilerInput {
    pub canonical: Vec<CompactMemoryRecord>,
    pub preferences: Vec<CompactMemoryRecord>,
    pub focus: Vec<CompactMemoryRecord>,
    pub episodic: Vec<CompactMemoryRecord>,
    pub semantic: Vec<CompactMemoryRecord>,
    pub corrections: Vec<CompactMemoryRecord>,
    pub candidates: Vec<CompactMemoryRecord>,
}

impl CompilerInput {
    pub fn bucket(&self, kind: BucketKind) -> &Vec<CompactMemoryRecord> {
        match kind {
            BucketKind::Canonical => &self.canonical,
            BucketKind::Preference => &self.preferences,
            BucketKind::Focus => &self.focus,
            BucketKind::Episodic => &self.episodic,
            BucketKind::Semantic => &self.semantic,
            BucketKind::Correction => &self.corrections,
            BucketKind::Candidate => &self.candidates,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WakeBudget {
    pub tokens: usize,
    pub per_bucket_floor: HashMap<BucketKind, usize>,
    pub kinds_coverage: KindsCoverage,
}

#[derive(Debug, Clone)]
pub struct KindsCoverage {
    pub working_context_pct: f64,
    pub session_continuity_pct: f64,
    pub canonical_pct: f64,
    pub semantic_episodic_pct: f64,
    pub procedural_pct: f64,
}

impl Default for KindsCoverage {
    fn default() -> Self {
        Self {
            working_context_pct: 0.25,
            session_continuity_pct: 0.20,
            canonical_pct: 0.25,
            semantic_episodic_pct: 0.20,
            procedural_pct: 0.10,
        }
    }
}

impl WakeBudget {
    pub fn default_2000() -> Self {
        let mut floor = HashMap::new();
        floor.insert(BucketKind::Canonical, 4);
        floor.insert(BucketKind::Preference, 3);
        floor.insert(BucketKind::Focus, 1);
        Self {
            tokens: 2000,
            per_bucket_floor: floor,
            kinds_coverage: KindsCoverage::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompiledWake {
    pub markdown: String,
    pub tokens: usize,
    pub bucket_report: HashMap<BucketKind, BucketReport>,
    pub demotion_hints: Vec<DemotionHint>,
}

#[derive(Debug, Clone, Default)]
pub struct BucketReport {
    pub admitted: usize,
    pub demoted: usize,
    pub fill_ratio: f64,
}

#[derive(Debug, Clone)]
pub struct DemotionHint {
    pub bucket: BucketKind,
    pub count: usize,
    pub reason: String,
}

/// D4 entry point. Pure transform: no IO, deterministic per input.
///
/// Pipeline: priority order → cross-bucket dedupe → budget+kinds-coverage
/// admission → markdown render with demotion hints.
pub fn compile_wake(input: CompilerInput, budget: WakeBudget) -> CompiledWake {
    let ordered = priority::apply(&input);
    let deduped = dedupe::merge(ordered);
    let admitted = budget::admit(deduped, &budget);
    render::emit(admitted, &budget)
}
