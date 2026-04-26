//! F5 typed-retrieval runner.
//!
//! Measures whether query shape routes to the correct MemoryKind.
//! 50 queries × 11 kinds = 550 invocations. Reports confusion matrix +
//! correct-type-rate@1.

use crate::benchmark::substrate::session_driver::{F5Scenario, BenchBackend, RecordingBackend};
use chrono::Utc;
use memd_schema::MemoryKind;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Pass-gate thresholds for F5. Matches `phase-f5-plan.md` §2.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PassGate {
    pub(crate) correct_type_rate_at_1: f64,
    pub(crate) wrong_type_ratio: f64,
    pub(crate) per_kind_min_rate: f64,
}

impl Default for PassGate {
    fn default() -> Self {
        Self {
            correct_type_rate_at_1: 0.85,
            wrong_type_ratio: 0.05,
            per_kind_min_rate: 0.75,
        }
    }
}

/// Static config for F5 invocation.
#[derive(Debug, Clone)]
pub(crate) struct F5RunConfig {
    pub(crate) seed: u64,
    pub(crate) queries_per_kind: usize,
    pub(crate) pass_gate: PassGate,
    pub(crate) results_dir: PathBuf,
}

impl F5RunConfig {
    /// Default config matching the YAML spec.
    pub(crate) fn default_with_results_dir(results_dir: PathBuf) -> Self {
        Self {
            seed: 42,
            queries_per_kind: 50,
            pass_gate: PassGate::default(),
            results_dir,
        }
    }
}

/// Per-query record for F5.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct F5QueryRecord {
    pub(crate) suite: String,
    pub(crate) seed: u64,
    pub(crate) query_idx: usize,
    pub(crate) query: String,
    pub(crate) expected_kind: String,
    pub(crate) actual_kind: String,
    pub(crate) correct_at_1: bool,
    pub(crate) pass: bool,
}

/// Confusion matrix: 12×12 tracking per-kind performance.
#[derive(Debug, Clone)]
pub(crate) struct ConfusionMatrix {
    matrix: BTreeMap<String, BTreeMap<String, usize>>,
    kinds: Vec<String>,
}

impl ConfusionMatrix {
    pub(crate) fn new() -> Self {
        let kinds = vec![
            "Fact", "Decision", "Preference", "Runbook", "Procedural",
            "SelfModel", "Topology", "Status", "LiveTruth", "Pattern",
            "Constraint", "Correction",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

        let mut matrix = BTreeMap::new();
        for k in &kinds {
            let mut row = BTreeMap::new();
            for k2 in &kinds {
                row.insert(k2.clone(), 0);
            }
            matrix.insert(k.clone(), row);
        }

        Self { matrix, kinds }
    }

    pub(crate) fn record(&mut self, expected: &str, actual: &str) {
        if let Some(row) = self.matrix.get_mut(expected) {
            if let Some(cell) = row.get_mut(actual) {
                *cell += 1;
            }
        }
    }

    pub(crate) fn kinds(&self) -> &[String] {
        &self.kinds
    }

    pub(crate) fn to_csv(&self) -> String {
        let mut csv = String::new();
        csv.push_str("expected,");
        for kind in &self.kinds {
            csv.push_str(&format!("{},", kind));
        }
        csv.pop(); // remove trailing comma
        csv.push('\n');

        for expected in &self.kinds {
            csv.push_str(&format!("{},", expected));
            if let Some(row) = self.matrix.get(expected) {
                for actual in &self.kinds {
                    let count = row.get(actual).copied().unwrap_or(0);
                    csv.push_str(&format!("{},", count));
                }
            }
            csv.pop(); // remove trailing comma
            csv.push('\n');
        }

        csv
    }
}

/// Scorer for correct-type-at-1 metric.
#[derive(Debug)]
pub(crate) struct CorrectTypeScorer;

impl CorrectTypeScorer {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) fn score_result(&self, expected: &str, actual: &str) -> f64 {
        if expected == actual {
            1.0
        } else {
            0.0
        }
    }
}

/// Outcome of one full F5 invocation.
#[derive(Debug, Clone)]
pub(crate) struct F5Outcome {
    pub(crate) records: Vec<F5QueryRecord>,
    pub(crate) ndjson_path: PathBuf,
    pub(crate) overall_pass: bool,
    pub(crate) confusion_matrix_csv: String,
}

/// Run F5 suite using perfect-recall in-process backend.
pub(crate) fn run_f5_in_process(config: &F5RunConfig) -> std::io::Result<F5Outcome> {
    let backend = RecordingBackend::default();
    run_f5_with_backend(config, &backend)
}

/// Backend-generic entry point.
pub(crate) fn run_f5_with_backend<B: BenchBackend>(
    config: &F5RunConfig,
    backend: &B,
) -> std::io::Result<F5Outcome> {
    let run_id = Uuid::new_v4().to_string();
    let ts_ms = Utc::now().timestamp_millis();

    // Kinds to query (11 kinds, excluding Correction which is B5 scope)
    let query_kinds = vec![
        ("Fact", "What is X?"),
        ("Decision", "Why did we choose A over B?"),
        ("Preference", "What do I like?"),
        ("Runbook", "How do I execute Y?"),
        ("Procedural", "What are the steps for Z?"),
        ("SelfModel", "What am I good at?"),
        ("Topology", "How is the system organized?"),
        ("Status", "What is the current state?"),
        ("LiveTruth", "Is X currently true?"),
        ("Pattern", "What pattern does X follow?"),
        ("Constraint", "What are the limits on X?"),
    ];

    let mut records = Vec::new();
    let mut matrix = ConfusionMatrix::new();
    let scorer = CorrectTypeScorer::new();
    let mut overall_pass = true;

    for (expected_kind, query_template) in query_kinds {
        for query_idx in 0..config.queries_per_kind {
            let query = format!("{} [query {}-{}]", query_template, expected_kind, query_idx);
            let scenario = F5Scenario {
                suite: "typed-retrieval".into(),
                seed: config.seed,
                query: query.clone(),
                expected_kind: expected_kind.to_string(),
            };

            let outcome = scenario.run(backend);
            let actual_kind = &outcome.routed_kind;
            let correct = scorer.score_result(expected_kind, actual_kind) > 0.0;

            matrix.record(expected_kind, actual_kind);

            let record = F5QueryRecord {
                suite: "typed-retrieval".into(),
                seed: config.seed,
                query_idx,
                query: query.clone(),
                expected_kind: expected_kind.to_string(),
                actual_kind: actual_kind.clone(),
                correct_at_1: correct,
                pass: correct,
            };

            if !correct {
                overall_pass = false;
            }

            records.push(record);
        }
    }

    // Write NDJSON
    let ndjson_path = config.results_dir.join("typed-retrieval.ndjson");
    if let Some(parent) = ndjson_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&ndjson_path)?;
    for r in &records {
        let line = serde_json::to_string(r).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
        })?;
        writeln!(f, "{}", line)?;
    }

    // Apply pass-gate thresholds
    if records.is_empty() {
        overall_pass = false;
    } else {
        let correct_count = records.iter().filter(|r| r.correct_at_1).count();
        let correct_rate = correct_count as f64 / records.len() as f64;
        if correct_rate < config.pass_gate.correct_type_rate_at_1 {
            overall_pass = false;
        }
    }

    Ok(F5Outcome {
        records,
        ndjson_path,
        overall_pass,
        confusion_matrix_csv: matrix.to_csv(),
    })
}
