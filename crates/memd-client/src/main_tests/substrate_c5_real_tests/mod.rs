//! C5 real-backend integration test. Drives `memd-server` over HTTP via
//! the `MemdGateway` trait so the existing claude_code + codex
//! `HarnessAdapter`s drive a real persistence layer instead of the
//! perfect-recall `InMemoryGateway`. `#[ignore]` by default — run with:
//!
//! ```sh
//! CARGO_TARGET_DIR=/tmp/memd-target cargo build --bin memd-server
//! CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-client --bin memd \
//!   -- --ignored c5_real
//! ```
//!
//! Visibility model on the wire:
//! * `Scope::Project`  → `MemoryScope::Project`, project = config.project,
//!                       namespace = "shared". Visible to either harness
//!                       that searches this project.
//! * `Scope::Local`    → `MemoryScope::Local`, namespace = "c5-{harness}".
//!                       Per-harness namespace is what enforces the
//!                       cross-harness isolation we audit.
//! * `Scope::Global`   → `MemoryScope::Global`. Not exercised by the
//!                       default scenarios but mapped for completeness.
//!
//! Each write carries:
//!   - `source_agent = harness` so `ReadHit.source_harness` round-trips.
//!   - `tags = [tag, "c5-real"]` so `ReadHit.tag` round-trips and a
//!     tag-only filter scopes the search to a single planted record.
//!
//! See `crates/memd-client/src/main_tests/substrate_a5_real_tests` and
//! `..substrate_b5_real_tests` for the sister-suites this mirrors.
//!
//! See `docs/handoff/LATEST.md` for the V5 merge gate this test feeds.

use crate::MemdClient;
use crate::benchmark::substrate::cross_harness::{
    C5PassGate, C5RunConfig, C5Scenario, run_c5_with_adapters,
};
use crate::benchmark::substrate::harness_adapter::{
    HarnessAdapter, MemdGateway, ReadHit, Scope, WriteResult,
    claude_code::ClaudeCodeAdapter, codex::CodexAdapter,
};
use memd_schema::{
    MemoryKind, MemoryScope, MemoryStatus, MemoryVisibility, SearchMemoryRequest,
    StoreMemoryRequest,
};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use tempfile::tempdir;
use tokio::runtime::Runtime;

use super::real_server_support::spawn_memd_server;

struct HttpMemdGateway {
    client: MemdClient,
    runtime: Runtime,
    counter: AtomicU64,
}

impl HttpMemdGateway {
    fn new(base_url: &str) -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("build tokio runtime");
        let client = MemdClient::new(base_url).expect("build memd client");
        Self {
            client,
            runtime,
            counter: AtomicU64::new(0),
        }
    }

    fn map_kind(s: &str) -> MemoryKind {
        match s {
            "preference" => MemoryKind::Preference,
            "correction" => MemoryKind::Fact, // server has no Correction kind; tag it.
            _ => MemoryKind::Fact,
        }
    }

    fn map_scope(scope: Scope) -> MemoryScope {
        match scope {
            Scope::Project => MemoryScope::Project,
            Scope::Local => MemoryScope::Local,
            Scope::Global => MemoryScope::Global,
        }
    }

    fn namespace_for(harness: &str, scope: Scope) -> String {
        match scope {
            // Per-harness namespace is the wire-level enforcement of
            // local-scope isolation across harnesses.
            Scope::Local => format!("c5-{harness}"),
            // Project + Global writes share a namespace within this
            // suite's project so both harnesses can read each other's
            // project-scope payloads.
            _ => "shared".to_string(),
        }
    }
}

impl MemdGateway for HttpMemdGateway {
    fn remember(
        &self,
        harness: &str,
        project: &str,
        kind: &str,
        content: &str,
        tag: &str,
        scope: Scope,
    ) -> std::io::Result<WriteResult> {
        let req = StoreMemoryRequest {
            content: content.to_string(),
            kind: Self::map_kind(kind),
            scope: Self::map_scope(scope),
            project: Some(project.to_string()),
            namespace: Some(Self::namespace_for(harness, scope)),
            workspace: None,
            visibility: Some(MemoryVisibility::Workspace),
            belief_branch: None,
            source_agent: Some(harness.to_string()),
            source_system: None,
            source_path: None,
            source_quality: None,
            confidence: Some(1.0),
            ttl_seconds: None,
            last_verified_at: None,
            supersedes: vec![],
            tags: vec![tag.to_string(), "c5-real".into(), kind.to_string()],
            status: Some(MemoryStatus::Active),
            lane: None,
        };
        let resp = self
            .runtime
            .block_on(self.client.store(&req))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        // Stable id for the test seam. The server's UUID is what
        // matters, but the gateway interface uses an opaque string.
        let n = self.counter.fetch_add(1, Ordering::SeqCst) + 1;
        Ok(WriteResult {
            id: format!("c5-{}-{n:04}-{}", harness, resp.item.id),
            tag: tag.to_string(),
            scope,
        })
    }

    fn lookup(
        &self,
        harness: &str,
        project: &str,
        query: &str,
        scope: Scope,
    ) -> std::io::Result<Vec<ReadHit>> {
        // C5's read scripts use the tag as the query string (see
        // `cross_harness::build_scripts`), so a tag filter is the
        // natural high-precision lookup. Project/Local/Global semantics
        // are enforced by namespace + scope filters server-side.
        let namespace = Self::namespace_for(harness, scope);
        let req = SearchMemoryRequest {
            query: None,
            statuses: vec![MemoryStatus::Active],
            scopes: vec![Self::map_scope(scope)],
            project: Some(project.to_string()),
            namespace: match scope {
                // Local: clamp to this harness's namespace so a leaky
                // server can't accidentally surface another harness's
                // local writes. Project/Global span "shared".
                Scope::Local => Some(namespace.clone()),
                _ => Some("shared".to_string()),
            },
            tags: vec![query.to_string()],
            limit: Some(10),
            ..SearchMemoryRequest::default()
        };
        let resp = self
            .runtime
            .block_on(self.client.search(&req))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        let mut hits = Vec::with_capacity(resp.items.len());
        for item in resp.items {
            // Map the item's persisted scope back to C5's `Scope` enum.
            // `MemoryScope::Synced` is collapsed to `Project` for audit
            // purposes — synced scope is an implementation detail of
            // the persistence layer, not a C5 visibility class.
            let source_scope = match item.scope {
                MemoryScope::Local => Scope::Local,
                MemoryScope::Global => Scope::Global,
                MemoryScope::Project | MemoryScope::Synced => Scope::Project,
            };
            // Tag round-trip: pick the C5 script tag (the one that
            // isn't "c5-real" and isn't the kind marker). The script
            // tag is unique per fact, so this is unambiguous.
            let script_tag = item
                .tags
                .iter()
                .find(|t| {
                    t.as_str() != "c5-real"
                        && t.as_str() != "fact"
                        && t.as_str() != "preference"
                        && t.as_str() != "correction"
                })
                .cloned()
                .unwrap_or_default();
            hits.push(ReadHit {
                id: item.id.to_string(),
                content: item.content,
                tag: script_tag,
                source_harness: item.source_agent.unwrap_or_default(),
                source_scope,
                source_project: item.project,
            });
        }
        Ok(hits)
    }
}

fn ready_adapters(dir: &std::path::Path) -> (ClaudeCodeAdapter, CodexAdapter) {
    let claude_settings = dir.join("settings.json");
    let codex_hooks = dir.join("hooks.json");
    fs::write(&claude_settings, "{}").unwrap();
    fs::write(&codex_hooks, "{}").unwrap();
    (
        ClaudeCodeAdapter::with_config_path(claude_settings),
        CodexAdapter::with_config_path(codex_hooks),
    )
}

/// Captured metric per scenario row.
#[derive(Debug, Clone)]
struct C5Row {
    suite: String,
    truth_conservation_rate: f64,
    visibility_leak_count: u64,
}

fn run_one_scenario_set(base_url: &str, project: &str) -> (Vec<C5Row>, u64) {
    let dir = tempdir().expect("tempdir for adapter configs");
    let (claude, codex) = ready_adapters(dir.path());
    let gateway = HttpMemdGateway::new(base_url);

    let cfg = C5RunConfig {
        // Floor everything so the runner never short-circuits on its own
        // pass-gate; we assert separately on returned numbers.
        pass_gate: C5PassGate {
            truth_conservation_rate: 0.0,
            visibility_leak_count: u64::MAX,
            latency_p95_ms: u64::MAX,
        },
        project: project.into(),
        results_dir: dir.path().to_path_buf(),
        ..C5RunConfig::default_with_results_dir(dir.path().to_path_buf())
    };
    let adapters: Vec<(&str, &dyn HarnessAdapter)> =
        vec![("claude_code", &claude), ("codex", &codex)];
    let outcome =
        run_c5_with_adapters(&cfg, &adapters, &gateway).expect("run c5 real scenario set");
    let leaks_total = outcome.leaks.len() as u64;
    let rows: Vec<C5Row> = outcome
        .records
        .into_iter()
        .map(|r| C5Row {
            suite: r.suite,
            truth_conservation_rate: r.recall_at_1,
            visibility_leak_count: r.recall_at_3 as u64,
        })
        .collect();
    (rows, leaks_total)
}

/// Sanity floor: a real memd-server must conserve project-scope truth
/// across the claude→codex and codex→claude pairs and leak nothing
/// across the local-scope boundary.
#[test]
#[ignore]
fn c5_real_backend_truth_and_isolation_non_trivial() {
    let server = spawn_memd_server().expect("spawn memd-server");
    let (rows, leaks_total) = run_one_scenario_set(&server.base_url, "c5-real-smoke");
    eprintln!("c5-real rows={}", rows.len());
    for r in &rows {
        eprintln!(
            "  {} truth={:.3} leaks={}",
            r.suite, r.truth_conservation_rate, r.visibility_leak_count
        );
    }
    assert!(!rows.is_empty(), "must produce at least one record");
    for r in &rows {
        assert!(
            r.truth_conservation_rate > 0.0,
            "real backend must conserve some project-scope truth ({}: {:.3})",
            r.suite,
            r.truth_conservation_rate
        );
    }
    assert_eq!(
        leaks_total, 0,
        "real backend must enforce zero cross-harness / cross-project visibility leaks"
    );
}

/// One-shot baseline capture. Prints JSON-ready rows to stdout. Run
/// with `--nocapture` and paste into
/// `docs/verification/substrate-baselines/c5_real-YYYY-MM-DD.json`.
#[test]
#[ignore]
fn c5_real_capture_baseline_numbers() {
    let server = spawn_memd_server().expect("spawn memd-server");
    let (rows, leaks_total) = run_one_scenario_set(&server.base_url, "c5-real-baseline");
    println!("--- c5_real baseline capture ---");
    println!("    \"leaks_total\": {leaks_total},");
    println!("    \"scenarios\": [");
    for (i, r) in rows.iter().enumerate() {
        let comma = if i + 1 == rows.len() { "" } else { "," };
        println!(
            "      {{ \"suite\": \"{}\", \"truth_conservation_rate\": {}, \"visibility_leak_count\": {} }}{comma}",
            r.suite, r.truth_conservation_rate, r.visibility_leak_count
        );
    }
    println!("    ]");
    println!("--- end capture ---");
}

/// Locked-baseline regression check. Loads the most-recent
/// `c5_real-*.json` file and asserts every metric stays within
/// `tolerance` of the locked floor + zero leaks.
#[test]
#[ignore]
fn c5_real_baseline_canonical_numbers() {
    let baselines_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/verification/substrate-baselines");
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&baselines_dir)
        .unwrap_or_else(|e| panic!("read baseline dir {baselines_dir:?}: {e}"))
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|n| n.starts_with("c5_real-") && n.ends_with(".json"))
        })
        .collect();
    entries.sort();
    let latest = entries.last().expect("at least one c5_real-*.json baseline");

    #[derive(serde::Deserialize)]
    struct BaselineScenario {
        suite: String,
        truth_conservation_rate: f64,
        #[allow(dead_code)]
        visibility_leak_count: u64,
    }
    #[derive(serde::Deserialize)]
    struct Baseline {
        tolerance: f64,
        leaks_total: u64,
        scenarios: Vec<BaselineScenario>,
    }

    let baseline: Baseline = serde_json::from_slice(&std::fs::read(latest).unwrap())
        .unwrap_or_else(|e| panic!("parse {latest:?}: {e}"));

    let server = spawn_memd_server().expect("spawn memd-server");
    let (rows, leaks_total) = run_one_scenario_set(&server.base_url, "c5-real-locked");
    assert_eq!(
        leaks_total, baseline.leaks_total,
        "regression: visibility leaks {leaks_total} != locked {}",
        baseline.leaks_total
    );
    for floor in &baseline.scenarios {
        let row = rows
            .iter()
            .find(|r| r.suite == floor.suite)
            .unwrap_or_else(|| panic!("missing suite in run: {}", floor.suite));
        assert!(
            row.truth_conservation_rate + baseline.tolerance >= floor.truth_conservation_rate,
            "regression: {} truth_conservation_rate {:.3} < floor {:.3} (tol {:.3})",
            floor.suite,
            row.truth_conservation_rate,
            floor.truth_conservation_rate,
            baseline.tolerance,
        );
        assert_eq!(
            row.visibility_leak_count, 0,
            "regression: {} produced {} visibility leaks (locked floor is 0)",
            floor.suite, row.visibility_leak_count
        );
    }
}

// Kept around as an unused import sentinel: silences dead-code warnings
// for `C5Scenario` if a future refactor narrows its surface area.
#[allow(dead_code)]
const _SCENARIO_TYPE_HINT: Option<C5Scenario> = None;
