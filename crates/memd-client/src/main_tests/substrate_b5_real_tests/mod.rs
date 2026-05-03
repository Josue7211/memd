//! B5 real-backend integration test. Drives `memd-server` over HTTP via
//! the `B5Backend` trait and asserts non-trivial correction propagation
//! and provenance-citation rates on a small synthetic corpus.
//! `#[ignore]` by default — run with:
//!
//! ```sh
//! CARGO_TARGET_DIR=/tmp/memd-target cargo build --bin memd-server
//! CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-client --bin memd \
//!   -- --ignored b5_real
//! ```
//!
//! `seal_session` / `restore_session` are no-ops here: a real
//! memd-server persists records via SQLite, so cross-session recall
//! happens by construction. Provenance citation is detected via a
//! correction-turn tag attached at `/memory/correct` time — the server's
//! `correct_item` does not populate `correction_meta.source_turn`, so
//! tags are the canonical wire-level signal we have.
//!
//! Ranking note: for a fact-stable identity we tag every memory for fact
//! N with `b5-fact-{N}` and search by that tag — corpus subjects /
//! predicates collide across fact_ids by design, so a content-only
//! query is ambiguous. The tag filter scopes the search down to a
//! single fact's active descendants.
//!
//! See `crates/memd-client/src/main_tests/substrate_a5_real_tests` for
//! the sister-suite that this mirrors.
//!
//! See `docs/handoff/LATEST.md` for the V5 merge gate this test feeds.

use crate::MemdClient;
use crate::benchmark::substrate::correction_propagation::{
    B5Backend, B5PassGate, B5RunConfig, QueryHit, correction_turn_id, run_b5_with_backend,
};
use crate::benchmark::substrate::fixtures::{Fact, KindMix};
use memd_schema::{
    CorrectMemoryRequest, MemoryKind, MemoryScope, MemoryStatus, MemoryVisibility,
    SearchMemoryRequest, StoreMemoryRequest,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tempfile::tempdir;
use tokio::runtime::Runtime;
use uuid::Uuid;

use super::real_server_support::spawn_memd_server;

fn fact_tag(fact_id: u32) -> String {
    format!("b5-fact-{fact_id:03}")
}

struct HttpB5Backend {
    client: MemdClient,
    project: String,
    namespace: String,
    runtime: Runtime,
    /// fact_id → latest active memory id for that fact. Updated on each
    /// correction — `/memory/correct` returns a new item; the old one
    /// is marked Superseded server-side.
    state: Mutex<HashMap<u32, Uuid>>,
}

impl HttpB5Backend {
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

impl B5Backend for HttpB5Backend {
    fn open_session(&self, _id: &str) {}

    fn ingest_fact(&self, _session: &str, fact: &Fact) {
        let req = StoreMemoryRequest {
            // Prefix with fact.id so canonical/redundancy dedup never
            // collapses two synthetic facts that happen to share a
            // subject/predicate/value triple — corpus tables are small
            // and collisions are common at fact_count >= 20.
            content: format!("f{} {} {} {}", fact.id, fact.subject, fact.predicate, fact.value),
            kind: MemoryKind::Fact,
            scope: MemoryScope::Project,
            project: Some(self.project.clone()),
            namespace: Some(self.namespace.clone()),
            workspace: None,
            visibility: Some(MemoryVisibility::Workspace),
            belief_branch: None,
            source_agent: Some("b5-real-bench".into()),
            source_system: None,
            source_path: None,
            source_quality: None,
            confidence: Some(1.0),
            ttl_seconds: None,
            last_verified_at: None,
            supersedes: vec![],
            tags: vec!["b5-real".into(), fact_tag(fact.id)],
            status: Some(MemoryStatus::Active),
            lane: None,
        };
        let resp = self
            .runtime
            .block_on(self.client.store(&req))
            .expect("memd store");
        self.state.lock().unwrap().insert(fact.id, resp.item.id);
    }

    fn apply_correction(&self, session: &str, fact_id: u32, corrected_value: &str) {
        let id = *self
            .state
            .lock()
            .unwrap()
            .get(&fact_id)
            .expect("fact must be ingested before correction");
        // Look up subject+predicate from a search by fact tag so we can
        // rebuild the corrected content. Cheaper than caching here and
        // keeps the truth in the server.
        let prior = self
            .runtime
            .block_on(self.client.search(&SearchMemoryRequest {
                query: None,
                statuses: vec![MemoryStatus::Active],
                project: Some(self.project.clone()),
                namespace: Some(self.namespace.clone()),
                tags: vec![fact_tag(fact_id)],
                limit: Some(1),
                ..SearchMemoryRequest::default()
            }))
            .expect("memd search (apply_correction lookup)");
        let head = prior.items.first().expect("prior fact must exist");
        // Content shape: "{subject} {predicate} {value}". Replace last
        // token with corrected value.
        let mut tokens: Vec<&str> = head.content.split_whitespace().collect();
        if let Some(last) = tokens.last_mut() {
            *last = corrected_value;
        }
        let new_content = tokens.join(" ");
        let turn = correction_turn_id(session, fact_id);
        let req = CorrectMemoryRequest {
            id,
            content: new_content,
            reason: Some("b5-real correction".into()),
            tags: Some(vec!["b5-real".into(), fact_tag(fact_id), turn]),
            confidence: Some(1.0),
        };
        let resp = self
            .runtime
            .block_on(self.client.correct(&req))
            .expect("memd correct");
        self.state.lock().unwrap().insert(fact_id, resp.new_item.id);
    }

    fn seal_session(&self, _id: &str) {}

    fn restore_session(&self, _id: &str, _restored_from: &str) {}

    fn query_with_provenance(
        &self,
        _session: &str,
        fact_id: u32,
        correction_turn: &str,
    ) -> Option<QueryHit> {
        let req = SearchMemoryRequest {
            query: None,
            statuses: vec![MemoryStatus::Active],
            project: Some(self.project.clone()),
            namespace: Some(self.namespace.clone()),
            tags: vec![fact_tag(fact_id)],
            limit: Some(1),
            ..SearchMemoryRequest::default()
        };
        let resp = self
            .runtime
            .block_on(self.client.search(&req))
            .expect("memd search");
        let item = resp.items.into_iter().next()?;
        let value = item
            .content
            .split_whitespace()
            .next_back()
            .unwrap_or("")
            .to_string();
        let cites_correction_turn = item.tags.iter().any(|t| t == correction_turn);
        Some(QueryHit {
            value,
            cites_correction_turn,
        })
    }
}

const B5_SCENARIOS: &[usize] = &[10, 20, 50];

fn run_one_scenario(base_url: &str, fact_count: usize) -> (f64, f64, f64) {
    let project = format!("b5-real-n{fact_count}");
    let backend = HttpB5Backend::new(base_url, &project, "main");
    let dir = tempdir().expect("tempdir for results");
    let cfg = B5RunConfig {
        seed: 43,
        fact_count,
        correct_in_session: 2,
        query_sessions: vec![3, 5, 8],
        kind_mix: KindMix::default(),
        // Floor pass-gate to 0.0 so the runner never fails the harness;
        // we assert separately on returned rates.
        pass_gate: B5PassGate {
            propagation_rate_s3: 0.0,
            propagation_rate_s8: 0.0,
            provenance_correctness: 0.0,
        },
        results_dir: dir.path().to_path_buf(),
        rollback_enabled: true,
    };
    let outcome = run_b5_with_backend(&cfg, &backend).expect("run b5 real scenario");
    // records[0] = qs=3, records[1] = qs=5, records[2] = qs=8.
    // recall_at_1 carries propagation_rate; recall_at_3 carries provenance_rate.
    let prop_s3 = outcome.records[0].recall_at_1;
    let prop_s8 = outcome.records[2].recall_at_1;
    let prov_avg = outcome
        .records
        .iter()
        .map(|r| r.recall_at_3)
        .sum::<f64>()
        / outcome.records.len() as f64;
    (prop_s3, prop_s8, prov_avg)
}

/// Sanity floor: a real memd-server must propagate corrections forward
/// through later sessions and tag the active item with the correction
/// turn. The locked baseline tightens this to production thresholds.
#[test]
#[ignore]
fn b5_real_backend_propagation_non_trivial() {
    let server = spawn_memd_server().expect("spawn memd-server");
    let (prop_s3, prop_s8, prov) = run_one_scenario(&server.base_url, 10);
    eprintln!("b5-real n=10: prop_s3={prop_s3:.3} prop_s8={prop_s8:.3} prov_avg={prov:.3}");
    assert!(
        prop_s3 > 0.0,
        "real backend must propagate corrections by s3 (got {prop_s3:.3})"
    );
    assert!(
        prov > 0.0,
        "real backend must cite correction turn at least once (got {prov:.3})"
    );
}

/// One-shot baseline capture. Prints JSON-ready scenario rows to
/// stdout. Run with `--nocapture` and paste into
/// `docs/verification/substrate-baselines/b5_real-YYYY-MM-DD.json`.
#[test]
#[ignore]
fn b5_real_capture_baseline_numbers() {
    let server = spawn_memd_server().expect("spawn memd-server");
    println!("--- b5_real baseline capture ---");
    for (idx, &n) in B5_SCENARIOS.iter().enumerate() {
        let (prop_s3, prop_s8, prov_avg) = run_one_scenario(&server.base_url, n);
        let comma = if idx + 1 == B5_SCENARIOS.len() {
            ""
        } else {
            ","
        };
        println!(
            "    {{ \"fact_count\": {n}, \"propagation_rate_s3\": {prop_s3}, \"propagation_rate_s8\": {prop_s8}, \"provenance_rate_avg\": {prov_avg} }}{comma}"
        );
    }
    println!("--- end capture ---");
}

/// Locked-baseline regression check. Loads the most-recent
/// `b5_real-*.json` file and asserts every metric stays within
/// `tolerance` of the locked floor.
#[test]
#[ignore]
fn b5_real_baseline_canonical_numbers() {
    let baselines_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/verification/substrate-baselines");
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&baselines_dir)
        .unwrap_or_else(|e| panic!("read baseline dir {baselines_dir:?}: {e}"))
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|n| n.starts_with("b5_real-") && n.ends_with(".json"))
        })
        .collect();
    entries.sort();
    let latest = entries.last().expect("at least one b5_real-*.json baseline");

    #[derive(serde::Deserialize)]
    struct BaselineScenario {
        fact_count: usize,
        propagation_rate_s3: f64,
        propagation_rate_s8: f64,
        provenance_rate_avg: f64,
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
        let (prop_s3, prop_s8, prov_avg) = run_one_scenario(&server.base_url, floor.fact_count);
        assert!(
            prop_s3 + baseline.tolerance >= floor.propagation_rate_s3,
            "regression: n={} propagation_rate_s3 {:.3} < floor {:.3} (tol {:.3})",
            floor.fact_count,
            prop_s3,
            floor.propagation_rate_s3,
            baseline.tolerance,
        );
        assert!(
            prop_s8 + baseline.tolerance >= floor.propagation_rate_s8,
            "regression: n={} propagation_rate_s8 {:.3} < floor {:.3} (tol {:.3})",
            floor.fact_count,
            prop_s8,
            floor.propagation_rate_s8,
            baseline.tolerance,
        );
        assert!(
            prov_avg + baseline.tolerance >= floor.provenance_rate_avg,
            "regression: n={} provenance_rate_avg {:.3} < floor {:.3} (tol {:.3})",
            floor.fact_count,
            prov_avg,
            floor.provenance_rate_avg,
            baseline.tolerance,
        );
    }
}
