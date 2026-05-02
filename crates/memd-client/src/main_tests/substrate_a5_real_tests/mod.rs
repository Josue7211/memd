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
use std::path::PathBuf;
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

const A5_SCENARIOS: &[(usize, usize)] = &[
    (20, 2),
    (20, 4),
    (20, 8),
    (50, 2),
    (50, 4),
    (50, 8),
    (100, 2),
    (100, 4),
    (100, 8),
];

fn run_one_scenario(
    base_url: &str,
    n: usize,
    k: usize,
) -> (f64, f64) {
    let project = format!("a5-real-n{n}-k{k}");
    let backend = HttpMemdBackend::new(base_url, &project, "main");
    let dir = tempdir().expect("tempdir for results");
    let cfg = A5RunConfig {
        seed: 42,
        fact_counts: vec![n],
        cuts: vec![k],
        kind_mix: KindMix::default(),
        pass_gate: PassGate {
            recall_at_3_k2: 0.0,
            recall_at_3_k8: 0.0,
        },
        results_dir: dir.path().to_path_buf(),
    };
    let outcome = run_a5_with_backend(&cfg, &backend).expect("run a5 real scenario");
    let r = &outcome.records[0];
    (r.recall_at_1, r.recall_at_3)
}

/// Sanity floor: a real memd-server should retrieve _something_ for the
/// smallest synthetic corpus. The locked baseline test below tightens
/// this to production thresholds across all 9 scenarios.
#[test]
#[ignore]
fn a5_real_backend_recall_non_trivial() {
    let server = spawn_memd_server().expect("spawn memd-server");
    let (r1, r3) = run_one_scenario(&server.base_url, 20, 2);
    eprintln!("a5-real n=20 k=2: recall@1={r1:.3} recall@3={r3:.3}");
    assert!(
        r3 > 0.0,
        "real backend must achieve non-trivial recall@3 (got {r3:.3})"
    );
}

/// One-shot baseline capture. Prints JSON-ready scenario rows to
/// stdout. Run with `--nocapture` and paste into
/// `docs/verification/substrate-baselines/a5_real-YYYY-MM-DD.json`.
#[test]
#[ignore]
fn a5_real_capture_baseline_numbers() {
    let server = spawn_memd_server().expect("spawn memd-server");
    println!("--- a5_real baseline capture ---");
    for &(n, k) in A5_SCENARIOS {
        let (r1, r3) = run_one_scenario(&server.base_url, n, k);
        let comma = if (n, k) == *A5_SCENARIOS.last().unwrap() {
            ""
        } else {
            ","
        };
        println!(
            "    {{ \"fact_count\": {n}, \"cut_k\": {k}, \"recall_at_1\": {r1}, \"recall_at_3\": {r3} }}{comma}"
        );
    }
    println!("--- end capture ---");
}

/// Locked-baseline regression check. Loads the most-recent
/// `a5_real-*.json` file, replays each scenario against a fresh real
/// backend, and asserts every recall_at_3 stays within `tolerance` of
/// the locked floor.
#[test]
#[ignore]
fn a5_real_baseline_canonical_numbers() {
    let baselines_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/verification/substrate-baselines");
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&baselines_dir)
        .unwrap_or_else(|e| panic!("read baseline dir {baselines_dir:?}: {e}"))
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|n| n.starts_with("a5_real-") && n.ends_with(".json"))
        })
        .collect();
    entries.sort();
    let latest = entries.last().expect("at least one a5_real-*.json baseline");

    #[derive(serde::Deserialize)]
    struct BaselineScenario {
        fact_count: usize,
        cut_k: usize,
        recall_at_3: f64,
    }
    #[derive(serde::Deserialize)]
    struct Baseline {
        tolerance: f64,
        scenarios: Vec<BaselineScenario>,
    }

    let baseline: Baseline = serde_json::from_slice(&std::fs::read(latest).unwrap())
        .unwrap_or_else(|e| panic!("parse {latest:?}: {e}"));

    let server = spawn_memd_server().expect("spawn memd-server");
    for floor in &baseline.scenarios {
        let (_r1, r3) = run_one_scenario(&server.base_url, floor.fact_count, floor.cut_k);
        assert!(
            r3 + baseline.tolerance >= floor.recall_at_3,
            "regression: n={} k={} recall_at_3 {:.3} < floor {:.3} (tol {:.3})",
            floor.fact_count,
            floor.cut_k,
            r3,
            floor.recall_at_3,
            baseline.tolerance,
        );
    }
}
