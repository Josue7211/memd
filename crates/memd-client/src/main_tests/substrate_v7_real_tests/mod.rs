//! V7 real-backend correction behavior-change tests.
//!
//! Ignored by default because they spawn `memd-server`. These prove the V7
//! in-process gate is not just a recorder: `/memory/correct` must create
//! active corrected facts that S2 retrieves by neutral query, and the
//! returned item must expose `correction_meta.source_turn`.

use crate::MemdClient;
use crate::benchmark::substrate::correction_behavior::{
    BehaviorHit, BehaviorQuery, V7BehaviorBackend, V7PassGate, V7RunConfig, corrected_value,
    correction_turn_id, run_v7_correction_behavior_with_backend,
};
use crate::benchmark::substrate::fixtures::{Fact, KindMix};
use memd_schema::{
    CorrectMemoryRequest, ExplainMemoryRequest, MemoryKind, MemoryScope, MemoryStatus,
    MemoryVisibility, SearchMemoryRequest, StoreMemoryRequest,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tempfile::tempdir;
use tokio::runtime::Runtime;
use uuid::Uuid;

use super::real_server_support::spawn_memd_server;

fn fact_tag(fact_id: u32) -> String {
    format!("v7-fact-{fact_id:03}")
}

struct HttpV7BehaviorBackend {
    client: MemdClient,
    project: String,
    namespace: String,
    runtime: Runtime,
    state: Mutex<HashMap<u32, Uuid>>,
}

impl HttpV7BehaviorBackend {
    fn new(base_url: &str, project: impl Into<String>, namespace: impl Into<String>) -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("build tokio runtime");
        let client = MemdClient::new(base_url).expect("build memd client");
        Self {
            client,
            project: project.into(),
            namespace: namespace.into(),
            runtime,
            state: Mutex::new(HashMap::new()),
        }
    }
}

impl V7BehaviorBackend for HttpV7BehaviorBackend {
    fn open_session(&self, _id: &str) {}

    fn ingest_fact(&self, _session: &str, fact: &Fact) {
        let req = StoreMemoryRequest {
            content: format!(
                "f{} {} {} {}",
                fact.id, fact.subject, fact.predicate, fact.value
            ),
            kind: MemoryKind::Fact,
            scope: MemoryScope::Project,
            project: Some(self.project.clone()),
            namespace: Some(self.namespace.clone()),
            workspace: None,
            visibility: Some(MemoryVisibility::Workspace),
            belief_branch: None,
            source_agent: Some("v7-real-bench".into()),
            source_system: None,
            source_path: None,
            source_quality: None,
            confidence: Some(1.0),
            ttl_seconds: None,
            last_verified_at: None,
            supersedes: vec![],
            tags: vec!["v7-real".into(), fact_tag(fact.id)],
            status: Some(MemoryStatus::Active),
            lane: None,
        };
        let resp = self
            .runtime
            .block_on(self.client.store(&req))
            .expect("memd store");
        self.state.lock().unwrap().insert(fact.id, resp.item.id);
    }

    fn apply_correction(&self, session: &str, fact: &Fact, corrected_value: &str) {
        let id = *self
            .state
            .lock()
            .unwrap()
            .get(&fact.id)
            .expect("fact must be ingested before correction");
        let turn = correction_turn_id(session, fact.id);
        let req = CorrectMemoryRequest {
            id,
            content: format!(
                "f{} {} {} {}",
                fact.id, fact.subject, fact.predicate, corrected_value
            ),
            reason: Some("v7-real correction".into()),
            tags: Some(vec!["v7-real".into(), fact_tag(fact.id), turn]),
            confidence: Some(1.0),
        };
        let resp = self
            .runtime
            .block_on(self.client.correct(&req))
            .expect("memd correct");
        self.state.lock().unwrap().insert(fact.id, resp.new_item.id);
    }

    fn seal_session(&self, _id: &str) {}
    fn restore_session(&self, _id: &str, _restored_from: &str) {}

    fn query_behavior(&self, _session: &str, query: &BehaviorQuery) -> Option<BehaviorHit> {
        let req = SearchMemoryRequest {
            query: Some(format!("{} {}", query.subject, query.predicate)),
            statuses: vec![MemoryStatus::Active],
            project: Some(self.project.clone()),
            namespace: Some(self.namespace.clone()),
            tags: vec![fact_tag(query.fact_id)],
            limit: Some(1),
            ..SearchMemoryRequest::default()
        };
        let resp = self
            .runtime
            .block_on(self.client.search(&req))
            .expect("memd search");
        let item = resp.items.into_iter().next()?;
        let explain = self
            .runtime
            .block_on(self.client.explain(&ExplainMemoryRequest {
                id: item.id,
                belief_branch: None,
                route: None,
                intent: None,
            }))
            .expect("memd explain");
        let value = item
            .content
            .split_whitespace()
            .next_back()
            .unwrap_or("")
            .to_string();
        let provenance_chain = item
            .correction_meta
            .and_then(|meta| meta.source_turn)
            .into_iter()
            .chain(
                explain
                    .corrections_chain
                    .into_iter()
                    .filter_map(|entry| entry.correction_source_turn),
            )
            .collect();
        Some(BehaviorHit {
            value,
            provenance_chain,
        })
    }
}

fn v7_real_config(results_dir: PathBuf) -> V7RunConfig {
    V7RunConfig {
        seed: 47,
        fact_count: 5,
        kind_mix: KindMix::default(),
        pass_gate: V7PassGate {
            next_session_behavior_rate: 1.0,
            chain_completeness_rate: 1.0,
            rollback_behavior_rate: 1.0,
            rollback_chain_completeness_rate: 1.0,
        },
        results_dir,
    }
}

#[test]
#[ignore]
fn v7_real_backend_correction_behavior_change_and_meta() {
    let server = spawn_memd_server().expect("spawn memd-server");
    let backend = HttpV7BehaviorBackend::new(&server.base_url, "v7-real", "main");
    let dir = tempdir().expect("tempdir for results");
    let cfg = v7_real_config(dir.path().to_path_buf());

    let outcome =
        run_v7_correction_behavior_with_backend(&cfg, &backend).expect("run v7 real behavior gate");

    assert!(outcome.overall_pass);
    assert_eq!(outcome.next_session_behavior_rate, 1.0);
    assert_eq!(outcome.chain_completeness_rate, 1.0);
    assert_eq!(outcome.rollback_behavior_rate, 1.0);
    assert_eq!(outcome.rollback_chain_completeness_rate, 1.0);
}
