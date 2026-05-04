//! V7 correction behavior-change runner.
//!
//! B5 proves corrected values can propagate. V7 tightens that into the
//! product behavior gate: a correction made in S1 must change S2 retrieval
//! without the S2 query repeating the corrected value, and the returned
//! record must cite the correction turn.

use crate::benchmark::substrate::fixtures::{Fact, KindMix, generate_corpus};
use crate::benchmark::substrate::report::{ScenarioRecord, append_ndjson};
use crate::benchmark::substrate::scorers::provenance_chain_cites_correction;
use chrono::Utc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub(crate) struct V7PassGate {
    pub(crate) next_session_behavior_rate: f64,
    pub(crate) chain_completeness_rate: f64,
}

impl Default for V7PassGate {
    fn default() -> Self {
        Self {
            next_session_behavior_rate: 0.05,
            chain_completeness_rate: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct V7RunConfig {
    pub(crate) seed: u64,
    pub(crate) fact_count: usize,
    pub(crate) kind_mix: KindMix,
    pub(crate) pass_gate: V7PassGate,
    pub(crate) results_dir: PathBuf,
}

impl V7RunConfig {
    pub(crate) fn default_with_results_dir(results_dir: PathBuf) -> Self {
        Self {
            seed: 47,
            fact_count: 5,
            kind_mix: KindMix::default(),
            pass_gate: V7PassGate::default(),
            results_dir,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct V7Outcome {
    pub(crate) records: Vec<ScenarioRecord>,
    pub(crate) ndjson_path: PathBuf,
    pub(crate) overall_pass: bool,
    pub(crate) next_session_behavior_rate: f64,
    pub(crate) chain_completeness_rate: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BehaviorQuery {
    pub(crate) fact_id: u32,
    pub(crate) subject: String,
    pub(crate) predicate: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BehaviorHit {
    pub(crate) value: String,
    pub(crate) provenance_chain: Vec<String>,
}

pub(crate) trait V7BehaviorBackend {
    fn open_session(&self, id: &str);
    fn ingest_fact(&self, session: &str, fact: &Fact);
    fn apply_correction(&self, session: &str, fact: &Fact, corrected_value: &str);
    fn seal_session(&self, id: &str);
    fn restore_session(&self, id: &str, restored_from: &str);
    fn query_behavior(&self, session: &str, query: &BehaviorQuery) -> Option<BehaviorHit>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum V7Event {
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
    BehaviorQuery {
        session: String,
        fact_id: u32,
        subject: String,
        predicate: String,
    },
}

#[derive(Debug, Clone)]
struct FactState {
    subject: String,
    predicate: String,
    value: String,
    chain: Vec<String>,
}

#[derive(Default)]
pub(crate) struct InProcessV7BehaviorBackend {
    state: Mutex<HashMap<u32, FactState>>,
    events: Mutex<Vec<V7Event>>,
}

impl InProcessV7BehaviorBackend {
    #[cfg(test)]
    pub(crate) fn events(&self) -> Vec<V7Event> {
        self.events.lock().unwrap().clone()
    }
}

impl V7BehaviorBackend for InProcessV7BehaviorBackend {
    fn open_session(&self, id: &str) {
        self.events
            .lock()
            .unwrap()
            .push(V7Event::SessionOpened(id.to_string()));
    }

    fn ingest_fact(&self, session: &str, fact: &Fact) {
        self.state.lock().unwrap().insert(
            fact.id,
            FactState {
                subject: fact.subject.clone(),
                predicate: fact.predicate.clone(),
                value: fact.value.clone(),
                chain: vec![ingest_turn_id(session, fact.id)],
            },
        );
        self.events.lock().unwrap().push(V7Event::FactIngested {
            session: session.to_string(),
            fact_id: fact.id,
        });
    }

    fn apply_correction(&self, session: &str, fact: &Fact, corrected_value: &str) {
        let mut state = self.state.lock().unwrap();
        let entry = state.entry(fact.id).or_insert_with(|| FactState {
            subject: fact.subject.clone(),
            predicate: fact.predicate.clone(),
            value: fact.value.clone(),
            chain: vec![ingest_turn_id(session, fact.id)],
        });
        entry.value = corrected_value.to_string();
        entry.chain.push(correction_turn_id(session, fact.id));
        drop(state);
        self.events
            .lock()
            .unwrap()
            .push(V7Event::CorrectionApplied {
                session: session.to_string(),
                fact_id: fact.id,
                value: corrected_value.to_string(),
            });
    }

    fn seal_session(&self, id: &str) {
        self.events
            .lock()
            .unwrap()
            .push(V7Event::SessionSealed(id.to_string()));
    }

    fn restore_session(&self, id: &str, restored_from: &str) {
        self.events.lock().unwrap().push(V7Event::SessionRestored {
            id: id.to_string(),
            from: restored_from.to_string(),
        });
    }

    fn query_behavior(&self, session: &str, query: &BehaviorQuery) -> Option<BehaviorHit> {
        self.events.lock().unwrap().push(V7Event::BehaviorQuery {
            session: session.to_string(),
            fact_id: query.fact_id,
            subject: query.subject.clone(),
            predicate: query.predicate.clone(),
        });
        let state = self.state.lock().unwrap();
        let entry = state.get(&query.fact_id)?;
        if entry.subject == query.subject && entry.predicate == query.predicate {
            Some(BehaviorHit {
                value: entry.value.clone(),
                provenance_chain: entry.chain.clone(),
            })
        } else {
            None
        }
    }
}

#[derive(Default)]
#[cfg(test)]
pub(crate) struct OriginalValueV7Backend {
    originals: Mutex<HashMap<u32, FactState>>,
}

#[cfg(test)]
impl V7BehaviorBackend for OriginalValueV7Backend {
    fn open_session(&self, _id: &str) {}

    fn ingest_fact(&self, session: &str, fact: &Fact) {
        self.originals.lock().unwrap().insert(
            fact.id,
            FactState {
                subject: fact.subject.clone(),
                predicate: fact.predicate.clone(),
                value: fact.value.clone(),
                chain: vec![ingest_turn_id(session, fact.id)],
            },
        );
    }

    fn apply_correction(&self, _session: &str, _fact: &Fact, _corrected_value: &str) {}
    fn seal_session(&self, _id: &str) {}
    fn restore_session(&self, _id: &str, _restored_from: &str) {}

    fn query_behavior(&self, _session: &str, query: &BehaviorQuery) -> Option<BehaviorHit> {
        let state = self.originals.lock().unwrap();
        state.get(&query.fact_id).map(|entry| BehaviorHit {
            value: entry.value.clone(),
            provenance_chain: entry.chain.clone(),
        })
    }
}

pub(crate) fn run_v7_correction_behavior_in_process(
    config: &V7RunConfig,
) -> std::io::Result<V7Outcome> {
    let backend = InProcessV7BehaviorBackend::default();
    run_v7_correction_behavior_with_backend(config, &backend)
}

pub(crate) fn run_v7_correction_behavior_with_backend<B: V7BehaviorBackend>(
    config: &V7RunConfig,
    backend: &B,
) -> std::io::Result<V7Outcome> {
    let run_id = Uuid::new_v4().to_string();
    let ts_ms = Utc::now().timestamp_millis();
    let facts = generate_corpus(config.seed, config.fact_count, &config.kind_mix);
    let s1 = session_id(config.seed, 1);
    let s2 = session_id(config.seed, 2);

    backend.open_session(&s1);
    for fact in &facts {
        backend.ingest_fact(&s1, fact);
        backend.apply_correction(&s1, fact, &corrected_value(fact));
    }
    backend.seal_session(&s1);
    backend.open_session(&s2);
    backend.restore_session(&s2, &s1);

    let mut behavior_hits = 0usize;
    let mut chain_hits = 0usize;
    for fact in &facts {
        let query = BehaviorQuery {
            fact_id: fact.id,
            subject: fact.subject.clone(),
            predicate: fact.predicate.clone(),
        };
        if let Some(hit) = backend.query_behavior(&s2, &query) {
            if hit.value == corrected_value(fact) {
                behavior_hits += 1;
            }
            if provenance_chain_cites_correction(
                &hit.provenance_chain,
                &correction_turn_id(&s1, fact.id),
            ) {
                chain_hits += 1;
            }
        }
    }

    let n = facts.len().max(1) as f64;
    let next_session_behavior_rate = behavior_hits as f64 / n;
    let chain_completeness_rate = chain_hits as f64 / n;
    let pass = next_session_behavior_rate >= config.pass_gate.next_session_behavior_rate
        && chain_completeness_rate >= config.pass_gate.chain_completeness_rate;
    let records = vec![ScenarioRecord {
        suite: "correction-behavior-change".into(),
        run_id,
        ts_ms,
        seed: config.seed,
        fact_count: config.fact_count,
        cut_k: 2,
        recall_at_1: next_session_behavior_rate,
        recall_at_3: chain_completeness_rate,
        answer_exact_match: next_session_behavior_rate,
        tokens_per_recall: 0,
        latency_ms_p50: 0,
        latency_ms_p95: 0,
        pass,
    }];

    let ndjson_path = ndjson_path_for(&config.results_dir, ts_ms);
    append_ndjson(&ndjson_path, &records)?;
    write_run_metadata(&config.results_dir, &records[0].run_id, ts_ms, config)?;

    Ok(V7Outcome {
        records,
        ndjson_path,
        overall_pass: pass,
        next_session_behavior_rate,
        chain_completeness_rate,
    })
}

pub(crate) fn corrected_value(fact: &Fact) -> String {
    format!("{}-corrected", fact.value)
}

pub(crate) fn correction_turn_id(session: &str, fact_id: u32) -> String {
    format!("{session}-correct-{fact_id:03}")
}

fn ingest_turn_id(session: &str, fact_id: u32) -> String {
    format!("{session}-ingest-{fact_id:03}")
}

fn session_id(seed: u64, session_idx: usize) -> String {
    format!("v7-seed{seed}-s{session_idx}")
}

fn ndjson_path_for(results_dir: &Path, ts_ms: i64) -> PathBuf {
    let date = chrono::DateTime::<Utc>::from_timestamp_millis(ts_ms)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "unknown-date".into());
    results_dir.join(format!("correction-behavior-change-{date}.ndjson"))
}

fn write_run_metadata(
    results_dir: &Path,
    run_id: &str,
    ts_ms: i64,
    config: &V7RunConfig,
) -> std::io::Result<()> {
    use std::io::Write;
    std::fs::create_dir_all(results_dir)?;
    let row = serde_json::json!({
        "suite": "correction-behavior-change",
        "run_id": run_id,
        "ts_ms": ts_ms,
        "seed": config.seed,
        "fact_count": config.fact_count,
        "pass_gate": {
            "next_session_behavior_rate": config.pass_gate.next_session_behavior_rate,
            "chain_completeness_rate": config.pass_gate.chain_completeness_rate,
        },
    });
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(results_dir.join("runs.jsonl"))?;
    writeln!(file, "{row}")?;
    Ok(())
}
