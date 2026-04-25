use super::*;
use crate::runtime::recall::{
    clamp_lookup_limit, dispatch_lookup_with_depth, escalation, run_lookup_arm_inner,
    synth_resume_args, synth_wake_args, RecallDepth, LOOKUP_DEPTH_RECORD_CAP,
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
    assert_eq!(wake.budget_tokens, 0, "default budget defers to env/2000-default");
    assert!(!wake.write, "lookup --depth wake is read-only");
    assert!(!wake.summary, "wake arm should emit the full compiled wake doc");
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

    let bundle =
        std::env::temp_dir().join(format!("memd-recall-lookup-{}", uuid::Uuid::new_v4()));
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
    assert!(!resume.prompt, "resume arm returns the full snapshot, not the prompt form");
    assert!(!resume.summary, "resume arm returns the full snapshot, not the summary form");
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
    let mut args = baseline_lookup_args(
        bundle,
        "the migration plan we shelved",
        RecallDepth::Lookup,
    );
    args.tag = vec!["resume_state".to_string()];

    let outcome = run_lookup_arm_inner(&client, args)
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

    let outcome = run_lookup_arm_inner(&client, args)
        .await
        .expect("dispatch lookup inner");

    assert!(outcome.response.items.is_empty(), "expected zero hits");
    assert!(
        outcome.escalation_hint.is_none(),
        "neutral query must not trigger the escalation hint"
    );
}
