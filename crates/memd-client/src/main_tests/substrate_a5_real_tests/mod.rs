//! A5 real-backend integration test. Drives `memd-server` over HTTP via
//! the `BenchBackend` trait and asserts non-trivial recall on a small
//! synthetic corpus. `#[ignore]` by default — run with:
//!
//! ```sh
//! CARGO_TARGET_DIR=/tmp/memd-target cargo build --bin memd-server
//! CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-client --bin memd \
//!   -- --ignored a5_real
//! ```
//!
//! `seal_session` / `restore_session` are no-ops here: a real
//! memd-server persists records via SQLite, so cross-session recall
//! happens by construction. The A4 PostCompact ledger restore lives in
//! the V4 proof harness; this test focuses on the recall pipeline
//! (FTS5 + intrinsic ranking) end-to-end.

use crate::MemdClient;
use crate::benchmark::substrate::cross_session_recall::{
    A5RunConfig, PassGate, run_a5_with_backend,
};
use crate::benchmark::substrate::fixtures::{Fact, KindMix};
use crate::benchmark::substrate::session_driver::BenchBackend;
use memd_schema::{
    MemoryKind, MemoryScope, MemoryStatus, MemoryVisibility, SearchMemoryRequest,
    StoreMemoryRequest,
};
use std::sync::Mutex;
use tempfile::tempdir;
use tokio::runtime::Runtime;

use super::real_server_support::spawn_memd_server;

struct HttpMemdBackend {
    client: MemdClient,
    project: String,
    namespace: String,
    runtime: Runtime,
    current_session: Mutex<String>,
}

impl HttpMemdBackend {
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
            current_session: Mutex::new(String::new()),
        }
    }
}

impl BenchBackend for HttpMemdBackend {
    fn open_session(&self, id: &str) {
        *self.current_session.lock().unwrap() = id.to_string();
    }

    fn ingest_fact(&self, _session: &str, fact: &Fact) {
        let req = StoreMemoryRequest {
            content: format!("{} {} {}", fact.subject, fact.predicate, fact.value),
            kind: MemoryKind::Fact,
            scope: MemoryScope::Project,
            project: Some(self.project.clone()),
            namespace: Some(self.namespace.clone()),
            workspace: None,
            visibility: Some(MemoryVisibility::Workspace),
            belief_branch: None,
            source_agent: Some("a5-real-bench".into()),
            source_system: None,
            source_path: None,
            source_quality: None,
            confidence: Some(1.0),
            ttl_seconds: None,
            last_verified_at: None,
            supersedes: vec![],
            tags: vec!["a5-real".into()],
            status: Some(MemoryStatus::Active),
            lane: None,
        };
        self.runtime
            .block_on(self.client.store(&req))
            .expect("memd store");
    }

    fn seal_session(&self, _id: &str) {}

    fn restore_session(&self, _id: &str, _restored_from: &str) {}

    fn query_top_k(&self, _session: &str, fact: &Fact, k: usize) -> Vec<String> {
        let req = SearchMemoryRequest {
            query: Some(format!("{} {}", fact.subject, fact.predicate)),
            route: None,
            intent: None,
            scopes: vec![MemoryScope::Project],
            kinds: vec![],
            statuses: vec![MemoryStatus::Active],
            project: Some(self.project.clone()),
            namespace: Some(self.namespace.clone()),
            workspace: None,
            visibility: None,
            belief_branch: None,
            source_agent: None,
            region: None,
            tags: vec![],
            stages: vec![],
            limit: Some(k),
            max_chars_per_item: None,
        };
        let resp = self
            .runtime
            .block_on(self.client.search(&req))
            .expect("memd search");
        // Content shape: "{subject} {predicate} {value}". Last token = value.
        resp.items
            .iter()
            .map(|i| {
                i.content
                    .split_whitespace()
                    .next_back()
                    .unwrap_or("")
                    .to_string()
            })
            .collect()
    }
}

/// Sanity floor: a real memd-server should retrieve _something_ for the
/// synthetic corpus. The locked baseline (A5.4) tightens this to
/// production thresholds.
#[test]
#[ignore]
fn a5_real_backend_recall_non_trivial() {
    let server = spawn_memd_server().expect("spawn memd-server");
    let backend = HttpMemdBackend::new(&server.base_url, "a5-real-bench", "main");
    let dir = tempdir().expect("tempdir for results");
    let cfg = A5RunConfig {
        seed: 42,
        fact_counts: vec![20],
        cuts: vec![2],
        kind_mix: KindMix::default(),
        pass_gate: PassGate {
            recall_at_3_k2: 0.0,
            recall_at_3_k8: 0.0,
        },
        results_dir: dir.path().to_path_buf(),
    };
    let outcome = run_a5_with_backend(&cfg, &backend).expect("run a5 real");
    let r1 = outcome.records[0].recall_at_1;
    let r3 = outcome.records[0].recall_at_3;
    eprintln!(
        "a5-real n=20 k=2: recall@1={r1:.3} recall@3={r3:.3} pass={}",
        outcome.records[0].pass
    );
    assert!(
        r3 > 0.0,
        "real backend must achieve non-trivial recall@3 (got {r3:.3})"
    );
}
