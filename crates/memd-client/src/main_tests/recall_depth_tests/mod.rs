use super::*;
use crate::runtime::recall::{
    LOOKUP_DEPTH_RECORD_CAP, RecallDepth, clamp_lookup_limit, depth, dispatch_lookup_with_depth,
    escalation, run_lookup_arm_inner, synth_resume_args, synth_wake_args, telemetry,
};

fn baseline_lookup_args(output: PathBuf, query: &str, depth: RecallDepth) -> LookupArgs {
    LookupArgs {
        output,
        query: query.to_string(),
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        workspace: None,
        region: None,
        visibility: None,
        route: None,
        intent: None,
        kind: Vec::new(),
        tag: Vec::new(),
        include_stale: false,
        limit: None,
        verbose: false,
        json: true,
        depth,
        explain_depth: false,
        explain_route: false,
    }
}

// E4.2 — Test 1: CLI flag parses to the requested RecallDepth variant.
#[test]
fn recall_depth_parses_cli_flag() {
    let cli = Cli::try_parse_from([
        "memd",
        "lookup",
        "--query",
        "the migration plan",
        "--depth",
        "wake",
    ])
    .expect("lookup --depth wake should parse");

    match cli.command {
        Commands::Lookup(args) => {
            assert_eq!(args.depth, RecallDepth::Wake);
            assert!(!args.explain_depth);
        }
        other => panic!("expected lookup command, got {other:?}"),
    }

    let cli = Cli::try_parse_from([
        "memd",
        "lookup",
        "--query",
        "the migration plan",
        "--depth",
        "resume",
        "--explain-depth",
    ])
    .expect("lookup --depth resume --explain-depth should parse");

    match cli.command {
        Commands::Lookup(args) => {
            assert_eq!(args.depth, RecallDepth::Resume);
            assert!(args.explain_depth);
        }
        other => panic!("expected lookup command, got {other:?}"),
    }
}

// E4.2 — Test 2: When --depth is omitted, default is Lookup.
#[test]
fn recall_depth_defaults_to_lookup() {
    let cli = Cli::try_parse_from(["memd", "lookup", "--query", "anything"])
        .expect("lookup with no --depth should parse");

    match cli.command {
        Commands::Lookup(args) => assert_eq!(args.depth, RecallDepth::Lookup),
        other => panic!("expected lookup command, got {other:?}"),
    }
}

// E4.2 — Test 8: Wake-arm synthesis preserves bundle identity so the
// existing wake compiler path is invoked with the same context the user
// supplied to `lookup`. We test the synthesis, not the network round-trip
// (covered by D4 wake tests), so the contract stays focused.
#[test]
fn lookup_depth_wake_returns_compiled_wake() {
    let bundle = std::env::temp_dir().join(format!("memd-recall-wake-{}", uuid::Uuid::new_v4()));
    let args = baseline_lookup_args(bundle.clone(), "session start", RecallDepth::Wake);
    let wake = synth_wake_args(&args);

    assert_eq!(wake.output, bundle);
    assert_eq!(wake.project.as_deref(), Some("demo"));
    assert_eq!(wake.namespace.as_deref(), Some("main"));
    assert!(!wake.raw, "wake arm should always go through the compiler");
    assert_eq!(
        wake.budget_tokens, 0,
        "default budget defers to env/2000-default"
    );
    assert!(!wake.write, "lookup --depth wake is read-only");
    assert!(
        !wake.summary,
        "wake arm should emit the full compiled wake doc"
    );
}

// E4.2 — Test 9: At depth=Lookup the dispatcher sends a SearchMemoryRequest
// whose limit is clamped to ≤3 (per docs/contracts/recall-depth.md).
#[tokio::test]
async fn lookup_depth_lookup_returns_1_to_3_records() {
    assert_eq!(clamp_lookup_limit(None), LOOKUP_DEPTH_RECORD_CAP);
    assert_eq!(clamp_lookup_limit(Some(0)), 1);
    assert_eq!(clamp_lookup_limit(Some(1)), 1);
    assert_eq!(clamp_lookup_limit(Some(3)), 3);
    assert_eq!(clamp_lookup_limit(Some(99)), LOOKUP_DEPTH_RECORD_CAP);

    let bundle = std::env::temp_dir().join(format!("memd-recall-lookup-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&bundle).expect("create bundle root");

    let state = MockRuntimeState::default();
    let base_url = spawn_mock_runtime_server(state.clone(), false).await;
    let client = MemdClient::new(&base_url).expect("client");

    let mut args = baseline_lookup_args(bundle, "any neutral query", RecallDepth::Lookup);
    args.limit = Some(50);

    dispatch_lookup_with_depth(&client, &base_url, args)
        .await
        .expect("dispatch lookup");

    let requests = state
        .search_requests
        .lock()
        .expect("lock search requests")
        .clone();
    assert!(!requests.is_empty(), "dispatcher should hit /memory/search");
    let first = &requests[0];
    assert!(
        first.limit.unwrap_or(usize::MAX) <= LOOKUP_DEPTH_RECORD_CAP,
        "depth=lookup must clamp limit to ≤{LOOKUP_DEPTH_RECORD_CAP}, got {:?}",
        first.limit
    );
}

// E4.2 — Test 10: Resume-arm synthesis preserves bundle identity so the
// existing resume snapshot path is invoked. End-to-end resume IO is
// covered by resume tests; we assert the synthesis contract here.
#[test]
fn lookup_depth_resume_returns_full_task_state() {
    let bundle = std::env::temp_dir().join(format!("memd-recall-resume-{}", uuid::Uuid::new_v4()));
    let args = baseline_lookup_args(bundle.clone(), "where did i leave off", RecallDepth::Resume);
    let resume = synth_resume_args(&args);

    assert_eq!(resume.output, bundle);
    assert_eq!(resume.project.as_deref(), Some("demo"));
    assert_eq!(resume.namespace.as_deref(), Some("main"));
    assert!(
        !resume.prompt,
        "resume arm returns the full snapshot, not the prompt form"
    );
    assert!(
        !resume.summary,
        "resume arm returns the full snapshot, not the summary form"
    );
}

// E4.3 — Test 3: "the X task/plan/issue/decision/bug/feature" specifier.
#[test]
fn escalation_detector_fires_on_the_x_task_pattern() {
    assert!(escalation::detect("the migration task"));
    assert!(escalation::detect("THE migration TASK"));
    assert!(escalation::detect("my caching plan"));
    assert!(escalation::detect("our recent decision"));
    assert!(escalation::detect("the auth bug we hit yesterday"));
    assert!(escalation::detect("the new search feature"));
}

// E4.3 — Test 4: "what (was|were) (I|we) (doing|working on|trying)".
#[test]
fn escalation_detector_fires_on_what_was_i_doing() {
    assert!(escalation::detect("what was I doing yesterday"));
    assert!(escalation::detect("What were we working on this morning?"));
    assert!(escalation::detect("what was I trying to fix"));
    assert!(escalation::detect("where did I leave off"));
    assert!(escalation::detect("Where did we leave off last night"));
}

// E4.3 — Test 5: Neutral queries do not match the specifier set.
#[test]
fn escalation_detector_ignores_neutral_query() {
    assert!(!escalation::detect("configuration files"));
    assert!(!escalation::detect("how do I parse JSON"));
    assert!(!escalation::detect("server logs"));
    assert!(!escalation::detect("the"));
    assert!(!escalation::detect("task"));
    assert!(!escalation::detect(""));
}

// E4.3 — Test 11: Zero-hit lookup with specifier query → outcome carries
// the canonical hint string (dispatcher prints to stderr; here we assert
// at the runtime layer where stderr capture is unnecessary).
#[tokio::test]
async fn lookup_depth_lookup_zero_hit_emits_escalation_hint_when_specifier() {
    let bundle =
        std::env::temp_dir().join(format!("memd-recall-hint-pos-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&bundle).expect("create bundle root");

    let state = MockRuntimeState::default();
    let base_url = spawn_mock_runtime_server(state.clone(), false).await;
    let client = MemdClient::new(&base_url).expect("client");

    // mock_search returns no items for arbitrary tagless queries by default
    // unless the query matches one of its hardcoded branches; "the migration
    // plan" does not match any → zero-hit.
    let mut args =
        baseline_lookup_args(bundle, "the migration plan we shelved", RecallDepth::Lookup);
    args.tag = vec!["resume_state".to_string()];

    let outcome = run_lookup_arm_inner(&client, &base_url, args)
        .await
        .expect("dispatch lookup inner");

    assert!(outcome.response.items.is_empty(), "expected zero hits");
    let hint = outcome.escalation_hint.expect("expected escalation hint");
    assert!(hint.starts_with("hint: zero results at lookup depth."));
    assert!(hint.contains("--depth resume"));
    assert!(hint.contains("the migration plan we shelved"));
}

// E4.3 — Test 12: Zero-hit lookup with neutral query → no hint.
#[tokio::test]
async fn lookup_depth_lookup_zero_hit_no_hint_on_neutral_query() {
    let bundle =
        std::env::temp_dir().join(format!("memd-recall-hint-neg-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&bundle).expect("create bundle root");

    let state = MockRuntimeState::default();
    let base_url = spawn_mock_runtime_server(state.clone(), false).await;
    let client = MemdClient::new(&base_url).expect("client");

    let mut args = baseline_lookup_args(bundle, "configuration files", RecallDepth::Lookup);
    args.tag = vec!["resume_state".to_string()];

    let outcome = run_lookup_arm_inner(&client, &base_url, args)
        .await
        .expect("dispatch lookup inner");

    assert!(outcome.response.items.is_empty(), "expected zero hits");
    assert!(
        outcome.escalation_hint.is_none(),
        "neutral query must not trigger the escalation hint"
    );
}

fn read_depth_log(bundle_root: &PathBuf) -> Vec<serde_json::Value> {
    let path = telemetry::log_path(bundle_root);
    let raw = fs::read_to_string(&path).unwrap_or_default();
    raw.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str::<serde_json::Value>(l).expect("ndjson parse"))
        .collect()
}

// E4.4 — Test 6: Every dispatcher call writes exactly one NDJSON line.
#[tokio::test]
async fn telemetry_writes_one_ndjson_per_call() {
    let bundle =
        std::env::temp_dir().join(format!("memd-recall-telem-one-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&bundle).expect("create bundle root");

    let state = MockRuntimeState::default();
    let base_url = spawn_mock_runtime_server(state.clone(), false).await;
    let client = MemdClient::new(&base_url).expect("client");

    for query in ["alpha probe", "beta probe", "gamma probe"] {
        let args = baseline_lookup_args(bundle.clone(), query, RecallDepth::Lookup);
        dispatch_lookup_with_depth(&client, &base_url, args)
            .await
            .expect("dispatch lookup");
    }

    let lines = read_depth_log(&bundle);
    assert_eq!(lines.len(), 3, "one telemetry line per dispatch call");
    for line in &lines {
        assert_eq!(line["depth"], "lookup");
        assert!(line["ts_ms"].is_i64());
        assert!(line["records_returned"].is_u64());
        assert!(line["tokens_returned"].is_u64());
        assert!(line["latency_ms"].is_u64());
    }
}

// E4.4 — Test 7: Zero-hit + specifier query records the hint string in
// the `escalation_hint` column.
#[tokio::test]
async fn telemetry_records_zero_hit_with_escalation_hint() {
    let bundle =
        std::env::temp_dir().join(format!("memd-recall-telem-hint-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&bundle).expect("create bundle root");

    let state = MockRuntimeState::default();
    let base_url = spawn_mock_runtime_server(state.clone(), false).await;
    let client = MemdClient::new(&base_url).expect("client");

    let mut args = baseline_lookup_args(
        bundle.clone(),
        "the migration plan we shelved",
        RecallDepth::Lookup,
    );
    args.tag = vec!["resume_state".to_string()];

    dispatch_lookup_with_depth(&client, &base_url, args)
        .await
        .expect("dispatch lookup");

    let lines = read_depth_log(&bundle);
    assert_eq!(lines.len(), 1);
    let hint = lines[0]["escalation_hint"]
        .as_str()
        .expect("escalation_hint should be a string when set");
    assert!(hint.starts_with("hint: zero results at lookup depth."));
    assert_eq!(lines[0]["records_returned"], 0);
}

// E4.5 — Test 13: `--explain-depth` produces a one-line rationale that
// names the depth and the cost/quality tradeoff from the contract.
#[test]
fn cli_explain_depth_prints_rationale() {
    let lookup = depth::explain_line(RecallDepth::Lookup);
    assert!(lookup.starts_with("depth: lookup"));
    assert!(lookup.contains("targeted query"));
    assert!(lookup.contains("1–3 records") || lookup.contains("1-3 records"));

    let wake = depth::explain_line(RecallDepth::Wake);
    assert!(wake.starts_with("depth: wake"));
    assert!(wake.contains("≤2k tokens") || wake.contains("2k tokens"));

    let resume = depth::explain_line(RecallDepth::Resume);
    assert!(resume.starts_with("depth: resume"));
    assert!(resume.contains("full task-state"));

    let cli = Cli::try_parse_from(["memd", "lookup", "--query", "anything", "--explain-depth"])
        .expect("parse --explain-depth");
    match cli.command {
        Commands::Lookup(args) => assert!(args.explain_depth),
        other => panic!("expected lookup command, got {other:?}"),
    }
}

// E4.4 — Test 14: Standalone `memd wake` writes a depth telemetry line
// so wake calls show up in the recall-depth distribution alongside
// `lookup --depth wake`.
#[tokio::test]
async fn wake_cli_writes_depth_telemetry_line() {
    let bundle =
        std::env::temp_dir().join(format!("memd-recall-telem-wake-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&bundle).expect("create bundle root");

    let state = MockRuntimeState::default();
    let base_url = spawn_mock_runtime_server(state.clone(), false).await;
    write_test_bundle_config(&bundle, &base_url);

    let wake = WakeArgs {
        output: bundle.clone(),
        project: None,
        namespace: None,
        agent: None,
        workspace: None,
        visibility: None,
        route: None,
        intent: None,
        limit: None,
        rehydration_limit: None,
        semantic: false,
        verbose: false,
        write: false,
        summary: true,
        raw: true,
        budget_tokens: 0,
        include_bucket: Vec::new(),
        exclude_bucket: Vec::new(),
    };

    crate::run_bundle_wake_command(&wake, &base_url)
        .await
        .expect("run wake");

    let lines = read_depth_log(&bundle);
    assert_eq!(
        lines.len(),
        1,
        "wake CLI must emit one depth-telemetry line"
    );
    assert_eq!(lines[0]["depth"], "wake");
    assert_eq!(lines[0]["query"], "wake");
}

#[derive(serde::Deserialize)]
struct ExpectedDepthRow {
    query: String,
    expected_depth: String,
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/e4")
}

fn parse_jsonl<T: for<'de> serde::Deserialize<'de>>(path: &Path) -> Vec<T> {
    let raw = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("read fixture {}: {}", path.display(), err));
    raw.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("ndjson parse"))
        .collect()
}

fn parse_depth(raw: &str) -> RecallDepth {
    match raw {
        "wake" => RecallDepth::Wake,
        "lookup" => RecallDepth::Lookup,
        "resume" => RecallDepth::Resume,
        other => panic!("unknown depth in fixture: {other}"),
    }
}

fn percentile(latencies: &mut [u64], pct: f64) -> u64 {
    assert!(!latencies.is_empty(), "percentile of empty set");
    latencies.sort_unstable();
    let rank = ((pct / 100.0) * (latencies.len() as f64 - 1.0)).round() as usize;
    latencies[rank.min(latencies.len() - 1)]
}

async fn dispatch_fixture_set(bundle: &Path, base_url: &str, fixtures: &[ExpectedDepthRow]) {
    let client = MemdClient::new(base_url).expect("client");
    for row in fixtures {
        let depth = parse_depth(&row.expected_depth);
        let args = baseline_lookup_args(bundle.to_path_buf(), &row.query, depth);
        let _ = dispatch_lookup_with_depth(&client, base_url, args).await;
    }
}

// E4.6 — Test 15: per-depth latency p50/p95 are bounded and the
// percentile-compute path is exercised. Real-world contract budgets
// (`docs/contracts/recall-depth.md`: wake <100ms p50, lookup <50ms p50,
// resume <500ms p95) are validated by the E4.7 7-day dogfood. Mock-based
// test budgets are loose ceilings that catch unbounded latency regressions
// (e.g. accidental blocking IO in the dispatcher) without flaking under
// `cargo test` parallelism.
#[tokio::test]
async fn latency_budgets_hold_on_fixture_set() {
    let bundle = std::env::temp_dir().join(format!("memd-recall-latency-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&bundle).expect("create bundle root");

    let state = MockRuntimeState::default();
    let base_url = spawn_mock_runtime_server(state.clone(), false).await;
    write_test_bundle_config(&bundle, &base_url);

    let fixtures: Vec<ExpectedDepthRow> = parse_jsonl(&fixture_dir().join("expected-depth.jsonl"));
    assert_eq!(fixtures.len(), 30, "expected 30 fixture queries");

    dispatch_fixture_set(&bundle, &base_url, &fixtures).await;

    let lines = read_depth_log(&bundle);
    assert!(!lines.is_empty(), "dispatcher must emit telemetry");

    let mut by_depth: std::collections::BTreeMap<String, Vec<u64>> = Default::default();
    for line in &lines {
        let depth = line["depth"].as_str().unwrap().to_string();
        let lat = line["latency_ms"].as_u64().unwrap();
        by_depth.entry(depth).or_default().push(lat);
    }

    // Test ceilings are 20× the real-world budgets: tight enough to catch
    // unbounded growth, loose enough that mock-server contention under
    // `cargo test -j` does not flake.
    if let Some(lats) = by_depth.get_mut("wake") {
        let p50 = percentile(lats, 50.0);
        assert!(
            p50 < 2_000,
            "wake p50 {p50}ms must stay <2000ms (real budget 100ms)"
        );
    }
    if let Some(lats) = by_depth.get_mut("lookup") {
        let p50 = percentile(lats, 50.0);
        assert!(
            p50 < 1_000,
            "lookup p50 {p50}ms must stay <1000ms (real budget 50ms)"
        );
    }
    if let Some(lats) = by_depth.get_mut("resume") {
        let p95 = percentile(lats, 95.0);
        assert!(
            p95 < 5_000,
            "resume p95 {p95}ms must stay <5000ms (real budget 500ms)"
        );
    }
}

// E4.6 — Test 16: Running the 30-query fixture set produces a depth
// distribution where lookup share ≥30% (contract pass gate). Each query is
// dispatched at its expected depth; we count NDJSON lines per depth.
#[tokio::test]
async fn depth_distribution_test() {
    let bundle = std::env::temp_dir().join(format!("memd-recall-dist-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&bundle).expect("create bundle root");

    let state = MockRuntimeState::default();
    let base_url = spawn_mock_runtime_server(state.clone(), false).await;
    write_test_bundle_config(&bundle, &base_url);

    let fixtures: Vec<ExpectedDepthRow> = parse_jsonl(&fixture_dir().join("expected-depth.jsonl"));
    assert_eq!(fixtures.len(), 30);

    dispatch_fixture_set(&bundle, &base_url, &fixtures).await;

    let lines = read_depth_log(&bundle);
    let mut counts: std::collections::BTreeMap<String, usize> = Default::default();
    for line in &lines {
        let depth = line["depth"].as_str().unwrap().to_string();
        *counts.entry(depth).or_default() += 1;
    }

    let total: usize = counts.values().sum();
    assert_eq!(
        total, 30,
        "every dispatched query must emit exactly one telemetry line; got {counts:?}"
    );
    let lookup = counts.get("lookup").copied().unwrap_or(0);
    let lookup_share = lookup as f64 / total as f64;
    assert!(
        lookup_share >= 0.30,
        "lookup share {:.2}% must hit contract pass gate of ≥30% (counts={counts:?})",
        lookup_share * 100.0
    );
    assert_eq!(
        counts.get("wake").copied().unwrap_or(0),
        10,
        "fixture has 10 wake queries"
    );
    assert_eq!(
        counts.get("lookup").copied().unwrap_or(0),
        10,
        "fixture has 10 lookup queries"
    );
    assert_eq!(
        counts.get("resume").copied().unwrap_or(0),
        10,
        "fixture has 10 resume queries"
    );
}

// E4.6 — Test 17 (bonus): specifier fixtures positively / negatively match
// the escalation regex set, guarding against future drift in the regex
// patterns when fixtures are updated.
#[test]
fn specifier_fixtures_match_regex_set() {
    #[derive(serde::Deserialize)]
    struct Row {
        query: String,
    }
    let pos: Vec<Row> = parse_jsonl(&fixture_dir().join("specifier-positive.jsonl"));
    let neg: Vec<Row> = parse_jsonl(&fixture_dir().join("specifier-negative.jsonl"));
    assert_eq!(pos.len(), 10, "expected 10 positive specifier fixtures");
    assert_eq!(neg.len(), 10, "expected 10 negative specifier fixtures");
    for row in &pos {
        assert!(
            escalation::detect(&row.query),
            "positive fixture must match specifier set: {:?}",
            row.query
        );
    }
    for row in &neg {
        assert!(
            !escalation::detect(&row.query),
            "negative fixture must NOT match specifier set: {:?}",
            row.query
        );
    }
}
