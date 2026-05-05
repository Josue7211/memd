//! F5 real-backend live-fire test. Drives `memd-server` over HTTP via
//! the `RoutineSubstrate` trait and asserts a routine planted in S1
//! is durably recoverable in S2+ — the wire-side analogue of
//! `PerfectRoutineSubstrate`. `#[ignore]` by default — run with:
//!
//! ```sh
//! CARGO_TARGET_DIR=/tmp/memd-target cargo build --bin memd-server
//! CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-client --bin memd \
//!   -- --ignored f5_real
//! ```
//!
//! Cost model mirrors `PerfectRoutineSubstrate`: first call on a
//! routine_id pays `BASELINE_RETRIEVAL_COST` (planting via
//! `/memory/store` with `MemoryKind::Procedural` + per-routine tag);
//! subsequent calls pay `ROUTINE_INVOCATION_COST` if the server
//! returns a tag-filtered active hit, else baseline (substrate
//! failed to cache → live-fire gate fails by design).
//!
//! See `crates/memd-client/src/main_tests/substrate_a5_real_tests` and
//! `substrate_b5_real_tests` for the sister suites this mirrors.

use crate::MemdClient;
use crate::benchmark::substrate::f5_live_fire::{
    BASELINE_RETRIEVAL_COST, LiveFireConfig, ROUTINE_INVOCATION_COST, RoutineSubstrate,
    run_live_fire_with_substrate,
};
use memd_schema::{
    MemoryKind, MemoryScope, MemoryStatus, MemoryVisibility, SearchMemoryRequest,
    StoreMemoryRequest,
};
use std::path::PathBuf;
use tempfile::tempdir;
use tokio::runtime::Runtime;

use super::real_server_support::spawn_memd_server;

fn routine_tag(routine_id: &str) -> String {
    format!("f5-routine-{routine_id}")
}

struct HttpRoutineSubstrate {
    client: MemdClient,
    project: String,
    namespace: String,
    runtime: Runtime,
}

impl HttpRoutineSubstrate {
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
        }
    }

    fn search_routine(&self, routine_id: &str) -> usize {
        let tag = routine_tag(routine_id);
        let req = SearchMemoryRequest {
            query: None,
            route: None,
            intent: None,
            scopes: vec![MemoryScope::Project],
            kinds: vec![MemoryKind::Procedural],
            statuses: vec![MemoryStatus::Active],
            project: Some(self.project.clone()),
            namespace: Some(self.namespace.clone()),
            workspace: None,
            visibility: None,
            belief_branch: None,
            source_agent: None,
            region: None,
            tags: vec![tag],
            stages: vec![],
            limit: Some(1),
            max_chars_per_item: None,
        };
        self.runtime
            .block_on(self.client.search(&req))
            .map(|r| r.items.len())
            .unwrap_or(0)
    }

    fn plant_routine(&self, routine_id: &str) {
        let tag = routine_tag(routine_id);
        // Content must produce a unique `redundancy_key` token set per
        // routine — server-side dedup tokenizes on non-alphanumeric and
        // sort+dedups, so any two routines whose content reduces to the
        // same multiset will collapse. Substituting the routine id as a
        // single alphanumeric token (`r0`, `r1`, …) puts the entropy in
        // a distinct lexical slot the dedup respects.
        let unique = format!("r{}", routine_id.trim_start_matches("routine-"));
        let req = StoreMemoryRequest {
            content: format!("procedure {unique} alpha bravo charlie delta echo {unique}end"),
            kind: MemoryKind::Procedural,
            scope: MemoryScope::Project,
            project: Some(self.project.clone()),
            namespace: Some(self.namespace.clone()),
            workspace: None,
            visibility: Some(MemoryVisibility::Workspace),
            belief_branch: None,
            source_agent: Some("f5-real-bench".into()),
            source_system: None,
            source_path: None,
            source_quality: None,
            confidence: Some(1.0),
            ttl_seconds: None,
            last_verified_at: None,
            supersedes: vec![],
            tags: vec![tag],
            status: Some(MemoryStatus::Active),
            lane: None,
        };
        self.runtime
            .block_on(self.client.store(&req))
            .expect("memd store routine");
    }
}

impl RoutineSubstrate for HttpRoutineSubstrate {
    fn observe_or_invoke(&mut self, routine_id: &str) -> u32 {
        if self.search_routine(routine_id) > 0 {
            ROUTINE_INVOCATION_COST
        } else {
            self.plant_routine(routine_id);
            BASELINE_RETRIEVAL_COST
        }
    }

    fn is_cached(&self, routine_id: &str) -> bool {
        self.search_routine(routine_id) > 0
    }
}

fn run_one_scenario(base_url: &str, routine_count: usize, invocations_per_routine: usize) -> u32 {
    let project = format!("f5-real-r{routine_count}-i{invocations_per_routine}");
    let mut substrate = HttpRoutineSubstrate::new(base_url, &project, "main");
    let dir = tempdir().expect("tempdir for results");
    let cfg = LiveFireConfig {
        routine_count,
        invocations_per_routine,
        results_dir: dir.path().to_path_buf(),
    };
    let outcome =
        run_live_fire_with_substrate(&cfg, &mut substrate).expect("run f5 real live-fire");
    assert!(
        outcome.overall_pass,
        "real-backend live-fire must pass for {project}"
    );
    outcome.total_savings
}

const F5_LIVE_FIRE_SCENARIOS: &[(usize, usize)] = &[(3, 2), (5, 2), (5, 4)];

/// Sanity floor: a real memd-server must durably cache a planted routine
/// across `observe_or_invoke` calls. The locked baseline test below
/// tightens this to the full scenario sweep.
#[test]
#[ignore]
fn f5_real_backend_live_fire_non_trivial() {
    let server = spawn_memd_server().expect("spawn memd-server");
    let savings = run_one_scenario(&server.base_url, 3, 2);
    eprintln!("f5-real r=3 i=2: total_savings={savings}");
    assert!(
        savings >= BASELINE_RETRIEVAL_COST,
        "real backend must achieve ≥ 1×baseline savings (got {savings})"
    );
}

/// One-shot baseline capture. Prints JSON-ready scenario rows to stdout.
/// Run with `--nocapture` and paste into
/// `docs/verification/substrate-baselines/f5_real-YYYY-MM-DD.json`.
#[test]
#[ignore]
fn f5_real_capture_baseline_numbers() {
    let server = spawn_memd_server().expect("spawn memd-server");
    println!("--- f5_real baseline capture ---");
    for &(r, i) in F5_LIVE_FIRE_SCENARIOS {
        let savings = run_one_scenario(&server.base_url, r, i);
        let comma = if (r, i) == *F5_LIVE_FIRE_SCENARIOS.last().unwrap() {
            ""
        } else {
            ","
        };
        println!(
            "    {{ \"routine_count\": {r}, \"invocations_per_routine\": {i}, \"total_savings\": {savings} }}{comma}"
        );
    }
    println!("--- end capture ---");
}

/// Locked-baseline regression check. Loads the most-recent
/// `f5_real-*.json` file, replays each scenario against a fresh real
/// backend, and asserts every total_savings stays at or above the
/// locked floor.
#[test]
#[ignore]
fn f5_real_baseline_canonical_numbers() {
    let baselines_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/verification/substrate-baselines");
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&baselines_dir)
        .unwrap_or_else(|e| panic!("read baseline dir {baselines_dir:?}: {e}"))
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|n| n.starts_with("f5_real-") && n.ends_with(".json"))
        })
        .collect();
    entries.sort();
    let latest = entries
        .last()
        .expect("at least one f5_real-*.json baseline");

    #[derive(serde::Deserialize)]
    struct BaselineScenario {
        routine_count: usize,
        invocations_per_routine: usize,
        total_savings: u32,
    }
    #[derive(serde::Deserialize)]
    struct Baseline {
        scenarios: Vec<BaselineScenario>,
    }

    let baseline: Baseline = serde_json::from_slice(&std::fs::read(latest).unwrap())
        .unwrap_or_else(|e| panic!("parse {latest:?}: {e}"));

    let server = spawn_memd_server().expect("spawn memd-server");
    for floor in &baseline.scenarios {
        let savings = run_one_scenario(
            &server.base_url,
            floor.routine_count,
            floor.invocations_per_routine,
        );
        assert!(
            savings >= floor.total_savings,
            "regression: r={} i={} total_savings {} < floor {}",
            floor.routine_count,
            floor.invocations_per_routine,
            savings,
            floor.total_savings,
        );
    }
}
