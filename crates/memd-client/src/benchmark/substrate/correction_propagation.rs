//! B5 correction-propagation runner.
//!
//! Plant N facts in session 1, correct each in `correct_in_session`,
//! query in `query_sessions` {3, 5, 8} and verify both:
//!   1. value propagation — the lookup returns the corrected value,
//!   2. provenance linkage — the returned record's chain cites the
//!      correction turn.
//!
//! Backend is `B5Backend`. The default `InProcessB5Backend` is a
//! perfect-recall recorder that proves driver+scorer correctness; the
//! HTTP backend is a follow-up.

use crate::benchmark::substrate::fixtures::{Fact, KindMix, generate_corpus};
use crate::benchmark::substrate::report::{ScenarioRecord, append_ndjson};
use crate::benchmark::substrate::scorers::provenance_chain_cites_correction;
use chrono::Utc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
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
    fn query_with_provenance(
        &self,
        session: &str,
        fact_id: u32,
        correction_turn: &str,
    ) -> Option<QueryHit>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct QueryHit {
    pub(crate) value: String,
    pub(crate) cites_correction_turn: bool,
}

/// Events captured by the in-process backend so tests can assert
/// ordering without inspecting backend internals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum B5Event {
    SessionOpened(String),
    FactIngested {
        session: String,
        fact_id: u32,
    },
    CorrectionApplied {
        session: String,
        fact_id: u32,
        value: String,
    },
    SessionSealed(String),
    SessionRestored {
        id: String,
        from: String,
    },
    Query {
        session: String,
        fact_id: u32,
    },
}

/// Per-fact backing store: current value + ordered provenance chain of
/// turn IDs that contributed to the value.
#[derive(Debug, Clone)]
struct FactState {
    value: String,
    chain: Vec<String>,
}

/// Perfect-recall, in-process B5 backend. Used by the default runner +
/// integration tests. Doubles as a recording backend so unit tests can
/// assert event ordering.
#[derive(Default)]
pub(crate) struct InProcessB5Backend {
    state: Mutex<HashMap<u32, FactState>>,
    events: Mutex<Vec<B5Event>>,
}

impl InProcessB5Backend {
    pub(crate) fn events(&self) -> Vec<B5Event> {
        self.events.lock().unwrap().clone()
    }
}

impl B5Backend for InProcessB5Backend {
    fn open_session(&self, id: &str) {
        self.events
            .lock()
            .unwrap()
            .push(B5Event::SessionOpened(id.to_string()));
    }

    fn ingest_fact(&self, session: &str, fact: &Fact) {
        let turn = format!("{session}-ingest-{:03}", fact.id);
        self.state.lock().unwrap().insert(
            fact.id,
            FactState {
                value: fact.value.clone(),
                chain: vec![turn],
            },
        );
        self.events.lock().unwrap().push(B5Event::FactIngested {
            session: session.to_string(),
            fact_id: fact.id,
        });
    }

    fn apply_correction(&self, session: &str, fact_id: u32, corrected_value: &str) {
        let turn = correction_turn_id(session, fact_id);
        let mut st = self.state.lock().unwrap();
        let entry = st.entry(fact_id).or_insert_with(|| FactState {
            value: corrected_value.to_string(),
            chain: Vec::new(),
        });
        entry.value = corrected_value.to_string();
        entry.chain.push(turn);
        drop(st);
        self.events
            .lock()
            .unwrap()
            .push(B5Event::CorrectionApplied {
                session: session.to_string(),
                fact_id,
                value: corrected_value.to_string(),
            });
    }

    fn seal_session(&self, id: &str) {
        self.events
            .lock()
            .unwrap()
            .push(B5Event::SessionSealed(id.to_string()));
    }

    fn restore_session(&self, id: &str, restored_from: &str) {
        self.events.lock().unwrap().push(B5Event::SessionRestored {
            id: id.to_string(),
            from: restored_from.to_string(),
        });
    }

    fn query_with_provenance(
        &self,
        session: &str,
        fact_id: u32,
        correction_turn: &str,
    ) -> Option<QueryHit> {
        self.events.lock().unwrap().push(B5Event::Query {
            session: session.to_string(),
            fact_id,
        });
        let st = self.state.lock().unwrap();
        st.get(&fact_id).map(|s| QueryHit {
            value: s.value.clone(),
            cites_correction_turn: provenance_chain_cites_correction(&s.chain, correction_turn),
        })
    }
}

/// A backend that loses corrections (returns original value or None) —
/// used to exercise the pass-gate-miss path in tests.
#[derive(Default)]
pub(crate) struct DegradedB5Backend;

impl B5Backend for DegradedB5Backend {
    fn open_session(&self, _id: &str) {}
    fn ingest_fact(&self, _session: &str, _fact: &Fact) {}
    fn apply_correction(&self, _session: &str, _fact_id: u32, _corrected_value: &str) {}
    fn seal_session(&self, _id: &str) {}
    fn restore_session(&self, _id: &str, _restored_from: &str) {}
    fn query_with_provenance(
        &self,
        _session: &str,
        _fact_id: u32,
        _correction_turn: &str,
    ) -> Option<QueryHit> {
        None
    }
}

pub(crate) fn correction_turn_id(session: &str, fact_id: u32) -> String {
    format!("{session}-correct-{fact_id:03}")
}

pub(crate) fn session_id(seed: u64, session_idx: usize) -> String {
    format!("b5-seed{seed}-s{session_idx}")
}

/// Build a corrected value for `fact` such that it is detectably
/// different from the original — appended `-v2` suffix is enough for
/// exact-match scoring.
pub(crate) fn corrected_value(fact: &Fact) -> String {
    format!("{}-v2", fact.value)
}

/// Run B5 with the in-process perfect-recall backend.
pub(crate) fn run_b5_in_process(config: &B5RunConfig) -> std::io::Result<B5Outcome> {
    let backend = InProcessB5Backend::default();
    run_b5_with_backend(config, &backend)
}

/// Backend-generic entry point.
pub(crate) fn run_b5_with_backend<B: B5Backend>(
    config: &B5RunConfig,
    backend: &B,
) -> std::io::Result<B5Outcome> {
    let run_id = Uuid::new_v4().to_string();
    let ts_ms = Utc::now().timestamp_millis();
    let facts = generate_corpus(config.seed, config.fact_count, &config.kind_mix);

    // Determine highest session we need to drive (max of correct + queries).
    let max_session = config
        .query_sessions
        .iter()
        .copied()
        .chain(std::iter::once(config.correct_in_session))
        .max()
        .unwrap_or(1);

    // Session 1: ingest.
    let s1 = session_id(config.seed, 1);
    backend.open_session(&s1);
    for f in &facts {
        backend.ingest_fact(&s1, f);
    }
    backend.seal_session(&s1);

    // Sessions 2..=max_session: open+restore from previous, apply
    // correction in `correct_in_session`. Seal between hops.
    let mut prev = s1.clone();
    for s_idx in 2..=max_session {
        let sid = session_id(config.seed, s_idx);
        backend.open_session(&sid);
        backend.restore_session(&sid, &prev);
        if s_idx == config.correct_in_session {
            for f in &facts {
                backend.apply_correction(&sid, f.id, &corrected_value(f));
            }
        }
        if s_idx < max_session {
            backend.seal_session(&sid);
        }
        prev = sid;
    }

    // Per query_session: query each fact, score propagation + provenance.
    let correction_session = session_id(config.seed, config.correct_in_session);
    let mut records = Vec::with_capacity(config.query_sessions.len());
    let mut overall_pass = true;
    let mut prov_correct_total = 0usize;
    let mut prov_correct_hits = 0usize;

    for &qs in &config.query_sessions {
        let qsid = session_id(config.seed, qs);
        let mut prop_hits = 0usize;
        let mut prov_hits = 0usize;
        for f in &facts {
            let want = corrected_value(f);
            let turn = correction_turn_id(&correction_session, f.id);
            if let Some(hit) = backend.query_with_provenance(&qsid, f.id, &turn) {
                if hit.value == want {
                    prop_hits += 1;
                }
                if hit.cites_correction_turn {
                    prov_hits += 1;
                }
            }
        }
        let n = facts.len().max(1) as f64;
        let prop_rate = prop_hits as f64 / n;
        let prov_rate = prov_hits as f64 / n;
        prov_correct_total += facts.len();
        prov_correct_hits += prov_hits;

        let prop_floor = if qs <= 3 {
            config.pass_gate.propagation_rate_s3
        } else {
            config.pass_gate.propagation_rate_s8
        };
        let pass = prop_rate >= prop_floor && prov_rate >= config.pass_gate.provenance_correctness;
        if !pass {
            overall_pass = false;
        }

        records.push(ScenarioRecord {
            suite: "correction-propagation".into(),
            run_id: run_id.clone(),
            ts_ms,
            seed: config.seed,
            fact_count: config.fact_count,
            cut_k: qs,
            recall_at_1: prop_rate,
            recall_at_3: prov_rate,
            answer_exact_match: prop_rate,
            tokens_per_recall: 0,
            latency_ms_p50: 0,
            latency_ms_p95: 0,
            pass,
        });
    }

    // Belt-and-braces: also enforce the aggregate provenance rate floor.
    let agg_prov = if prov_correct_total > 0 {
        prov_correct_hits as f64 / prov_correct_total as f64
    } else {
        0.0
    };
    if agg_prov < config.pass_gate.provenance_correctness {
        overall_pass = false;
    }

    let ndjson_path = ndjson_path_for(&config.results_dir, ts_ms);
    if !records.is_empty() {
        append_ndjson(&ndjson_path, &records)?;
    }
    write_run_metadata(&config.results_dir, &run_id, ts_ms, config)?;

    Ok(B5Outcome {
        records,
        ndjson_path,
        overall_pass,
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
    fn b5_runner_writes_runs_metadata() {
        let dir = tempdir().unwrap();
        let cfg = B5RunConfig::default_with_results_dir(dir.path().to_path_buf());
        let outcome = run_b5_in_process(&cfg).unwrap();
        assert!(outcome.overall_pass);
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

    /// B5 Test 3 — `runner_applies_correction_in_session_2_via_c4_path`.
    /// The driver must emit a CorrectionApplied event in session 2
    /// after restore, and only in session 2.
    #[test]
    fn runner_applies_correction_in_session_2_via_c4_path() {
        let dir = tempdir().unwrap();
        let cfg = B5RunConfig {
            fact_count: 5,
            ..B5RunConfig::default_with_results_dir(dir.path().to_path_buf())
        };
        let backend = InProcessB5Backend::default();
        run_b5_with_backend(&cfg, &backend).unwrap();
        let events = backend.events();

        let s2 = session_id(cfg.seed, 2);
        let corrections_in_s2: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                B5Event::CorrectionApplied {
                    session, fact_id, ..
                } if session == &s2 => Some(*fact_id),
                _ => None,
            })
            .collect();
        assert_eq!(
            corrections_in_s2.len(),
            cfg.fact_count,
            "every fact must be corrected in s2"
        );

        // No corrections outside s2.
        let other_corrections = events.iter().any(|e| {
            matches!(
                e,
                B5Event::CorrectionApplied { session, .. } if session != &s2
            )
        });
        assert!(
            !other_corrections,
            "corrections must only fire in correct_in_session"
        );

        // Restore must precede correction.
        let restore_pos = events
            .iter()
            .position(|e| matches!(e, B5Event::SessionRestored { id, .. } if id == &s2))
            .expect("s2 restore missing");
        let first_correction = events
            .iter()
            .position(|e| matches!(e, B5Event::CorrectionApplied { session, .. } if session == &s2))
            .expect("s2 correction missing");
        assert!(restore_pos < first_correction);
    }

    /// B5 Test 4 — `runner_queries_each_target_session`.
    /// Every configured query_session must produce exactly N queries
    /// (one per fact) and emit a ScenarioRecord per session.
    #[test]
    fn runner_queries_each_target_session() {
        let dir = tempdir().unwrap();
        let cfg = B5RunConfig {
            fact_count: 4,
            query_sessions: vec![3, 5],
            ..B5RunConfig::default_with_results_dir(dir.path().to_path_buf())
        };
        let backend = InProcessB5Backend::default();
        let outcome = run_b5_with_backend(&cfg, &backend).unwrap();
        let events = backend.events();

        for &qs in &cfg.query_sessions {
            let qsid = session_id(cfg.seed, qs);
            let n_queries = events
                .iter()
                .filter(|e| matches!(e, B5Event::Query { session, .. } if session == &qsid))
                .count();
            assert_eq!(
                n_queries, cfg.fact_count,
                "wrong query count for session {qs}"
            );
        }
        assert_eq!(outcome.records.len(), cfg.query_sessions.len());
        // Perfect backend should pass every gate.
        assert!(outcome.overall_pass);
        for r in &outcome.records {
            assert!((r.recall_at_1 - 1.0).abs() < f64::EPSILON);
            assert!((r.recall_at_3 - 1.0).abs() < f64::EPSILON);
        }
    }

    /// B5 Test 9 — `b5_rollback_reassert_preserves_chain`.
    /// After a user corrects in s2 and then re-asserts the original
    /// value in s5, the provenance chain must contain BOTH the
    /// correction turn and the re-assertion turn (forward-only, no
    /// rewrite). This is the "user changed their mind back" case and
    /// guards against backends that silently drop the in-between state.
    #[test]
    fn b5_rollback_reassert_preserves_chain() {
        let backend = InProcessB5Backend::default();
        let s1 = session_id(43, 1);
        let s2 = session_id(43, 2);
        let s5 = session_id(43, 5);

        let fact = Fact {
            id: 7,
            kind: "canonical".into(),
            subject: "alice".into(),
            predicate: "lives_in".into(),
            value: "berlin".into(),
        };

        backend.open_session(&s1);
        backend.ingest_fact(&s1, &fact);
        backend.seal_session(&s1);

        backend.open_session(&s2);
        backend.restore_session(&s2, &s1);
        backend.apply_correction(&s2, fact.id, "tokyo");
        backend.seal_session(&s2);

        // Re-assert the original value in s5. Chain must keep both
        // correction-s2 and re-assert-s5 turns.
        backend.open_session(&s5);
        backend.restore_session(&s5, &s2);
        backend.apply_correction(&s5, fact.id, "berlin");

        let correction_turn = correction_turn_id(&s2, fact.id);
        let reassert_turn = correction_turn_id(&s5, fact.id);

        let hit = backend
            .query_with_provenance(&s5, fact.id, &correction_turn)
            .expect("fact must be retrievable");
        assert_eq!(
            hit.value, "berlin",
            "current value must reflect re-assertion"
        );
        assert!(
            hit.cites_correction_turn,
            "chain must still cite the s2 correction turn"
        );

        // Re-asking with reassert turn must also pass.
        let hit2 = backend
            .query_with_provenance(&s5, fact.id, &reassert_turn)
            .expect("fact must be retrievable");
        assert!(
            hit2.cites_correction_turn,
            "chain must cite the s5 re-assertion turn"
        );
    }

    /// Degraded backend (returns None for every query) must miss every
    /// pass-gate axis.
    #[test]
    fn runner_pass_gate_misses_with_degraded_backend() {
        let dir = tempdir().unwrap();
        let cfg = B5RunConfig::default_with_results_dir(dir.path().to_path_buf());
        let outcome = run_b5_with_backend(&cfg, &DegradedB5Backend).unwrap();
        assert!(!outcome.overall_pass);
        for r in &outcome.records {
            assert!(!r.pass);
            assert!((r.recall_at_1 - 0.0).abs() < f64::EPSILON);
            assert!((r.recall_at_3 - 0.0).abs() < f64::EPSILON);
        }
    }
}
