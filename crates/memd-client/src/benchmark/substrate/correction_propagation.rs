//! B5 correction-propagation runner.
//!
//! Plant N facts in session 1, correct each in session 2, query in
//! sessions {3, 5, 8} and verify both value propagation (lookup returns
//! corrected value) and provenance linkage (returned record cites the
//! correction turn).
//!
//! Runner stub — full impl lands in B5.3 / B5.4. Trait surface and
//! config types are defined here so the dispatcher arm in `mod.rs` can
//! reference them in B5.4.

use crate::benchmark::substrate::fixtures::{generate_corpus, Fact, KindMix};
use crate::benchmark::substrate::report::{append_ndjson, ScenarioRecord};
use chrono::Utc;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// What pass/fail looks like for B5. Matches `phase-b5-plan.md` §2
/// (`pass_gate.propagation_rate_*` + `provenance_correctness`).
#[derive(Debug, Clone, Copy)]
pub(crate) struct B5PassGate {
    pub(crate) propagation_rate_s3: f64,
    pub(crate) propagation_rate_s8: f64,
    pub(crate) provenance_correctness: f64,
}

impl Default for B5PassGate {
    fn default() -> Self {
        Self {
            propagation_rate_s3: 0.85,
            propagation_rate_s8: 0.80,
            provenance_correctness: 0.95,
        }
    }
}

/// Static config for a B5 invocation. CLI args lower into this.
#[derive(Debug, Clone)]
pub(crate) struct B5RunConfig {
    pub(crate) seed: u64,
    pub(crate) fact_count: usize,
    pub(crate) correct_in_session: usize,
    pub(crate) query_sessions: Vec<usize>,
    pub(crate) kind_mix: KindMix,
    pub(crate) pass_gate: B5PassGate,
    pub(crate) results_dir: PathBuf,
    pub(crate) rollback_enabled: bool,
}

impl B5RunConfig {
    pub(crate) fn default_with_results_dir(results_dir: PathBuf) -> Self {
        Self {
            seed: 43,
            fact_count: 20,
            correct_in_session: 2,
            query_sessions: vec![3, 5, 8],
            kind_mix: KindMix::default(),
            pass_gate: B5PassGate::default(),
            results_dir,
            rollback_enabled: rollback_flag_enabled(),
        }
    }
}

fn rollback_flag_enabled() -> bool {
    std::env::var("MEMD_SUBSTRATE_B5_ROLLBACK")
        .map(|v| v != "0")
        .unwrap_or(true)
}

/// Outcome of a single B5 invocation.
#[derive(Debug, Clone)]
pub(crate) struct B5Outcome {
    pub(crate) records: Vec<ScenarioRecord>,
    pub(crate) ndjson_path: PathBuf,
    pub(crate) overall_pass: bool,
}

/// Backend B5 talks to. Distinct from A5's `BenchBackend` because B5
/// needs (a) a notion of *applying a correction* in a later session and
/// (b) a query that returns provenance metadata, not just the value.
pub(crate) trait B5Backend {
    fn open_session(&self, id: &str);
    fn ingest_fact(&self, session: &str, fact: &Fact);
    fn apply_correction(&self, session: &str, fact_id: u32, corrected_value: &str);
    fn seal_session(&self, id: &str);
    fn restore_session(&self, id: &str, restored_from: &str);
    /// Returns `(value, cites_correction_turn)` if the backend has any
    /// record for `fact_id`. The boolean asserts whether the returned
    /// record's provenance chain references the correction-turn session.
    fn query_with_provenance(&self, session: &str, fact_id: u32) -> Option<QueryHit>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct QueryHit {
    pub(crate) value: String,
    pub(crate) cites_correction_turn: bool,
}

/// Run B5 with the in-process perfect-recall recorder. Used by
/// integration tests + the dispatcher's default path until the HTTP
/// backend lands.
pub(crate) fn run_b5_in_process(_config: &B5RunConfig) -> std::io::Result<B5Outcome> {
    // B5.1 stub — real implementation in B5.3. Returns an empty pass
    // so `cargo test` stays green while tests for the scorer + driver
    // are written in B5.2/B5.3.
    let run_id = Uuid::new_v4().to_string();
    let ts_ms = Utc::now().timestamp_millis();
    let records: Vec<ScenarioRecord> = Vec::new();
    let ndjson_path = ndjson_path_for(&_config.results_dir, ts_ms);
    if !records.is_empty() {
        append_ndjson(&ndjson_path, &records)?;
    }
    write_run_metadata(&_config.results_dir, &run_id, ts_ms, _config)?;
    Ok(B5Outcome {
        records,
        ndjson_path,
        overall_pass: true,
    })
}

fn ndjson_path_for(results_dir: &Path, ts_ms: i64) -> PathBuf {
    let date = chrono::DateTime::<Utc>::from_timestamp_millis(ts_ms)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "unknown-date".into());
    results_dir.join(format!("correction-propagation-{date}.ndjson"))
}

fn write_run_metadata(
    results_dir: &Path,
    run_id: &str,
    ts_ms: i64,
    config: &B5RunConfig,
) -> std::io::Result<()> {
    use std::io::Write;
    std::fs::create_dir_all(results_dir)?;
    let runs_jsonl = results_dir.join("runs.jsonl");
    let row = serde_json::json!({
        "suite": "correction-propagation",
        "run_id": run_id,
        "ts_ms": ts_ms,
        "seed": config.seed,
        "fact_count": config.fact_count,
        "correct_in_session": config.correct_in_session,
        "query_sessions": config.query_sessions,
        "rollback_enabled": config.rollback_enabled,
    });
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&runs_jsonl)?;
    let line = format!("{row}\n");
    f.write_all(line.as_bytes())
}

/// Reference to the deterministic source corpus B5 plants in session 1.
/// Exposed so tests can plant + assert against the same set the runner
/// will use.
pub(crate) fn b5_source_corpus(config: &B5RunConfig) -> Vec<Fact> {
    generate_corpus(config.seed, config.fact_count, &config.kind_mix)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn b5_default_config_matches_yaml_spec() {
        let dir = tempdir().unwrap();
        let cfg = B5RunConfig::default_with_results_dir(dir.path().to_path_buf());
        assert_eq!(cfg.seed, 43);
        assert_eq!(cfg.fact_count, 20);
        assert_eq!(cfg.correct_in_session, 2);
        assert_eq!(cfg.query_sessions, vec![3, 5, 8]);
        assert!((cfg.pass_gate.propagation_rate_s3 - 0.85).abs() < f64::EPSILON);
        assert!((cfg.pass_gate.provenance_correctness - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn b5_stub_runner_writes_runs_metadata() {
        let dir = tempdir().unwrap();
        let cfg = B5RunConfig::default_with_results_dir(dir.path().to_path_buf());
        let outcome = run_b5_in_process(&cfg).unwrap();
        assert!(outcome.overall_pass, "stub returns pass until B5.3");
        let runs = std::fs::read_to_string(dir.path().join("runs.jsonl")).unwrap();
        assert!(runs.contains("correction-propagation"));
    }

    #[test]
    fn b5_source_corpus_is_deterministic() {
        let dir = tempdir().unwrap();
        let cfg = B5RunConfig::default_with_results_dir(dir.path().to_path_buf());
        let a = b5_source_corpus(&cfg);
        let b = b5_source_corpus(&cfg);
        assert_eq!(a, b);
        assert_eq!(a.len(), 20);
    }
}
