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

use std::collections::{HashMap, HashSet};

use memd_schema::CompactMemoryRecord;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod buckets;
pub mod budget;
pub mod dedupe;
pub mod ledger;
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompilerInput {
    #[serde(default)]
    pub canonical: Vec<CompactMemoryRecord>,
    #[serde(default)]
    pub preferences: Vec<CompactMemoryRecord>,
    #[serde(default)]
    pub focus: Vec<CompactMemoryRecord>,
    #[serde(default)]
    pub episodic: Vec<CompactMemoryRecord>,
    #[serde(default)]
    pub semantic: Vec<CompactMemoryRecord>,
    #[serde(default)]
    pub corrections: Vec<CompactMemoryRecord>,
    #[serde(default)]
    pub candidates: Vec<CompactMemoryRecord>,
    /// F4.3 — drift surface lines (one per outstanding preference). Rendered
    /// inside the `## Preferences` section, above the record list. ≤80 chars
    /// each is enforced upstream; render does not re-truncate.
    #[serde(default)]
    pub drift_notes: Vec<String>,
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
    /// CLI `--include-bucket`: bypass class cap and total cap for these.
    pub force_include: HashSet<BucketKind>,
    /// CLI `--exclude-bucket`: drop from input entirely.
    pub force_exclude: HashSet<BucketKind>,
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
            force_include: HashSet::new(),
            force_exclude: HashSet::new(),
        }
    }

    pub fn with_tokens(mut self, tokens: usize) -> Self {
        if tokens > 0 {
            self.tokens = tokens;
        }
        self
    }

    pub fn with_includes(mut self, names: &[String]) -> Self {
        for name in names {
            if let Some(kind) = parse_bucket_label(name) {
                self.force_include.insert(kind);
            }
        }
        self
    }

    pub fn with_excludes(mut self, names: &[String]) -> Self {
        for name in names {
            if let Some(kind) = parse_bucket_label(name) {
                self.force_exclude.insert(kind);
            }
        }
        self
    }
}

/// `true` when wake should route through the compiler instead of the
/// legacy raw render. Flag default is OFF until D4.8 dogfood completes.
pub fn compiler_enabled() -> bool {
    matches!(
        std::env::var("MEMD_D4_COMPILER")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "on" | "yes"
    )
}

/// Reads the env-var override for `tokens` (chars). `0` = use default.
pub fn env_budget_tokens() -> usize {
    std::env::var("MEMD_WAKE_BUDGET_TOKENS")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .unwrap_or(0)
}

pub fn parse_bucket_label(name: &str) -> Option<BucketKind> {
    match name.trim().to_ascii_lowercase().as_str() {
        "canonical" => Some(BucketKind::Canonical),
        "preference" | "preferences" => Some(BucketKind::Preference),
        "focus" => Some(BucketKind::Focus),
        "episodic" => Some(BucketKind::Episodic),
        "semantic" => Some(BucketKind::Semantic),
        "correction" | "corrections" => Some(BucketKind::Correction),
        "candidate" | "candidates" => Some(BucketKind::Candidate),
        _ => None,
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
    let drift_notes = input.drift_notes.clone();
    let filtered = filter_excluded(input, &budget.force_exclude);
    let ordered = priority::apply(&filtered);
    let deduped = dedupe::merge(ordered);
    let admitted = budget::admit(deduped, &budget);
    render::emit(admitted, &budget, &drift_notes)
}

fn filter_excluded(mut input: CompilerInput, exclude: &HashSet<BucketKind>) -> CompilerInput {
    if exclude.is_empty() {
        return input;
    }
    if exclude.contains(&BucketKind::Canonical) {
        input.canonical.clear();
    }
    if exclude.contains(&BucketKind::Preference) {
        input.preferences.clear();
    }
    if exclude.contains(&BucketKind::Focus) {
        input.focus.clear();
    }
    if exclude.contains(&BucketKind::Episodic) {
        input.episodic.clear();
    }
    if exclude.contains(&BucketKind::Semantic) {
        input.semantic.clear();
    }
    if exclude.contains(&BucketKind::Correction) {
        input.corrections.clear();
    }
    if exclude.contains(&BucketKind::Candidate) {
        input.candidates.clear();
    }
    input
}

/// IO helper: read F4 outstanding drift state for `memd_dir` and render
/// each entry as a one-line note. Kept separate from `input_from_snapshot`
/// to preserve that adapter's pure-transform contract.
///
/// Returns an empty vec when the state file is missing or unreadable —
/// drift surfacing is best-effort, never blocks wake.
pub fn drift_notes_from_outstanding(memd_dir: &std::path::Path) -> Vec<String> {
    let path = memd_core::preference::outstanding::outstanding_state_path(memd_dir);
    match memd_core::preference::outstanding::read_outstanding(&path) {
        Ok(state) => state
            .entries
            .values()
            .map(|entry| entry.render_line())
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Adapter: build a CompilerInput from the existing ResumeSnapshot. Lives
/// here so the snapshot shape is the only thing that ever reaches the
/// compiler — keeps the pure-transform contract.
pub fn input_from_snapshot(snapshot: &super::ResumeSnapshot) -> CompilerInput {
    let preferences: Vec<CompactMemoryRecord> = snapshot
        .preferences
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| CompactMemoryRecord {
            id: Uuid::new_v4(),
            record: line.clone(),
        })
        .collect();
    CompilerInput {
        canonical: snapshot.context.records.clone(),
        preferences,
        focus: snapshot.working.records.clone(),
        episodic: Vec::new(),
        semantic: Vec::new(),
        corrections: Vec::new(),
        candidates: Vec::new(),
        drift_notes: Vec::new(),
    }
}
