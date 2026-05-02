//! A5 cross-session-recall runner.
//!
//! Wires fixtures + session_driver + scorers + report into a single
//! callable. The bench-time backend defaults to the in-process
//! perfect-recall recorder so the runner is fully reproducible without
//! booting memd-server. A `BackendKind::Http` variant lands in a
//! follow-up that talks to a spawned `memd-server` process; the
//! call-site here is identical, only the trait impl changes.

use crate::benchmark::substrate::fixtures::{KindMix, generate_corpus};
use crate::benchmark::substrate::report::{ScenarioRecord, append_ndjson};
use crate::benchmark::substrate::session_driver::{A5Scenario, BenchBackend, RecordingBackend};
use chrono::Utc;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// What pass/fail looks like for A5. Matches `phase-a5-plan.md` §2
/// (`pass_gate.recall_at_3_*` keys).
#[derive(Debug, Clone, Copy)]
pub(crate) struct PassGate {
    pub(crate) recall_at_3_k2: f64,
    pub(crate) recall_at_3_k8: f64,
}

impl Default for PassGate {
    fn default() -> Self {
        Self {
            recall_at_3_k2: 0.90,
            recall_at_3_k8: 0.80,
        }
    }
}

/// Static config for an A5 invocation. CLI args lower into this.
#[derive(Debug, Clone)]
pub(crate) struct A5RunConfig {
    pub(crate) seed: u64,
    pub(crate) fact_counts: Vec<usize>,
    pub(crate) cuts: Vec<usize>,
    pub(crate) kind_mix: KindMix,
    pub(crate) pass_gate: PassGate,
    pub(crate) results_dir: PathBuf,
}

impl A5RunConfig {
    /// Default config matching the YAML spec — used by the CLI when no
    /// `--spec` is provided.
    pub(crate) fn default_with_results_dir(results_dir: PathBuf) -> Self {
        Self {
            seed: 42,
            fact_counts: vec![20, 50, 100],
            cuts: vec![2, 4, 8],
            kind_mix: KindMix::default(),
            pass_gate: PassGate::default(),
            results_dir,
        }
    }
}

/// Outcome of one full A5 invocation.
#[derive(Debug, Clone)]
pub(crate) struct A5Outcome {
    pub(crate) records: Vec<ScenarioRecord>,
    pub(crate) ndjson_path: PathBuf,
    pub(crate) overall_pass: bool,
}

/// Run the A5 suite using the perfect-recall in-process backend.
/// Returns the records that were written to NDJSON + the file path.
pub(crate) fn run_a5_in_process(config: &A5RunConfig) -> std::io::Result<A5Outcome> {
    let backend = RecordingBackend::default();
    run_a5_with_backend(config, &backend)
}

/// Backend-generic entry point. Useful for tests that want to inject a
/// degraded backend to exercise the pass-gate-miss code path.
pub(crate) fn run_a5_with_backend<B: BenchBackend>(
    config: &A5RunConfig,
    backend: &B,
) -> std::io::Result<A5Outcome> {
    let run_id = Uuid::new_v4().to_string();
    let ts_ms = Utc::now().timestamp_millis();

    let mut records = Vec::with_capacity(config.fact_counts.len() * config.cuts.len());
    let mut overall_pass = true;

    for &n in &config.fact_counts {
        for &cut_k in &config.cuts {
            let facts = generate_corpus(config.seed, n, &config.kind_mix);
            let scenario = A5Scenario {
                suite: "cross-session-recall".into(),
                seed: config.seed,
                facts,
                cut_k,
            };
            let outcome = scenario.run(backend);
            let r1 = outcome.recall_at_1;
            // recall@3 ≈ recall@1 for this synthetic backend; in the
            // real-backend impl (follow-up) the driver will record
            // top-k retrieval and compute distinct values.
            let r3 = r1;
            let pass = match cut_k {
                k if k <= 2 => r3 >= config.pass_gate.recall_at_3_k2,
                _ => r3 >= config.pass_gate.recall_at_3_k8,
            };
            if !pass {
                overall_pass = false;
            }
            records.push(ScenarioRecord {
                suite: "cross-session-recall".into(),
                run_id: run_id.clone(),
                ts_ms,
                seed: config.seed,
                fact_count: n,
                cut_k,
                recall_at_1: r1,
                recall_at_3: r3,
                answer_exact_match: r1,
                tokens_per_recall: 0,
                latency_ms_p50: 0,
                latency_ms_p95: 0,
                pass,
            });
        }
    }

    let ndjson_path = ndjson_path_for(&config.results_dir, ts_ms);
    append_ndjson(&ndjson_path, &records)?;
    write_run_metadata(&config.results_dir, &run_id, ts_ms, config)?;

    Ok(A5Outcome {
        records,
        ndjson_path,
        overall_pass,
    })
}

fn ndjson_path_for(results_dir: &Path, ts_ms: i64) -> PathBuf {
    let date = chrono::DateTime::<Utc>::from_timestamp_millis(ts_ms)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "unknown-date".into());
    results_dir.join(format!("cross-session-recall-{date}.ndjson"))
}

fn write_run_metadata(
    results_dir: &Path,
    run_id: &str,
    ts_ms: i64,
    config: &A5RunConfig,
) -> std::io::Result<()> {
    use std::io::Write;
    std::fs::create_dir_all(results_dir)?;
    let runs_jsonl = results_dir.join("runs.jsonl");
    let row = serde_json::json!({
        "suite": "cross-session-recall",
        "run_id": run_id,
        "ts_ms": ts_ms,
        "seed": config.seed,
        "fact_counts": config.fact_counts,
        "cuts": config.cuts,
    });
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&runs_jsonl)?;
    let line = format!("{row}\n");
    f.write_all(line.as_bytes())
}

/// A backend that always returns the wrong value — used to exercise the
/// pass-gate-miss path in tests. Real bench runs never use this.
#[derive(Default)]
pub(crate) struct DegradedBackend;

impl BenchBackend for DegradedBackend {
    fn open_session(&self, _id: &str) {}
    fn ingest_fact(&self, _session: &str, _fact: &crate::benchmark::substrate::fixtures::Fact) {}
    fn seal_session(&self, _id: &str) {}
    fn restore_session(&self, _id: &str, _restored_from: &str) {}
    fn query_for_fact(&self, _session: &str, _fact_id: u32) -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn small_config(results_dir: PathBuf) -> A5RunConfig {
        A5RunConfig {
            seed: 42,
            fact_counts: vec![20],
            cuts: vec![2],
            kind_mix: KindMix::default(),
            pass_gate: PassGate::default(),
            results_dir,
        }
    }

    #[test]
    fn run_a5_in_process_writes_ndjson_and_passes() {
        let dir = tempdir().unwrap();
        let cfg = small_config(dir.path().to_path_buf());
        let outcome = run_a5_in_process(&cfg).unwrap();
        assert!(outcome.overall_pass);
        assert_eq!(outcome.records.len(), 1);
        let body = std::fs::read_to_string(&outcome.ndjson_path).unwrap();
        assert!(body.contains("cross-session-recall"));
        assert!(dir.path().join("runs.jsonl").exists());
    }

    #[test]
    fn run_a5_with_degraded_backend_fails_pass_gate() {
        let dir = tempdir().unwrap();
        let cfg = small_config(dir.path().to_path_buf());
        let outcome = run_a5_with_backend(&cfg, &DegradedBackend).unwrap();
        assert!(!outcome.overall_pass);
        for r in &outcome.records {
            assert_eq!(r.recall_at_1, 0.0);
            assert!(!r.pass);
        }
    }
}
