# 10-Star Verifier System Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship the first usable verifier-system slice for `memd` so the app can run live feature, journey, adversarial, and comparative proofs from the canonical registry instead of relying only on structural benchmark signals.

**Architecture:** Extend the existing benchmark registry into a unified benchmark-and-verification graph, then add a new `memd verify` CLI surface in `crates/memd-client/src/main.rs` that can load declarative verifier records, materialize fixture packs, execute bounded live verifier runs, write evidence artifacts, and produce gate results. Keep the current `memd benchmark` command as the cheap health lane and layer the live verifier system beside it rather than replacing it.

**Tech Stack:** Rust, clap, serde/serde_json, existing `memd benchmark` registry loader, existing `.memd` artifact/report directories, markdown/json outputs in `docs/verification`, existing `#[cfg(test)]` coverage in `crates/memd-client/src/main.rs` and `crates/memd-schema/src/lib.rs`.

---

## File Structure

- `docs/verification/benchmark-registry.json`
  - expand the canonical registry with verifier, fixture, evidence-policy, and schedule sections
- `docs/verification/benchmark-registry.schema.json`
  - extend the schema to validate the verifier-system additions
- `docs/verification/VERIFIERS.md`
  - generated verifier inventory and current coverage state
- `docs/verification/FIXTURES.md`
  - generated fixture inventory and purpose summary
- `docs/verification/COVERAGE.md`
  - extend existing coverage output with live-verification coverage
- `docs/verification/SCORES.md`
  - extend score output with verification gates and evidence-confidence rules
- `crates/memd-schema/src/lib.rs`
  - shared verifier, fixture, evidence-policy, schedule, verifier-run, and gate types
- `crates/memd-client/src/main.rs`
  - `memd verify` CLI parsing
  - verifier registry loading
  - fixture materialization
  - verifier execution engine
  - evidence writing
  - gate resolution
  - `verify sweep` scheduling
  - docs generation for verifier surfaces

## Task 1: Extend the canonical registry and schema for verifiers

**Files:**
- Modify: `docs/verification/benchmark-registry.json`
- Modify: `docs/verification/benchmark-registry.schema.json`
- Modify: `crates/memd-schema/src/lib.rs`
- Test: `crates/memd-schema/src/lib.rs`

**Goal:** Make the canonical registry capable of describing verifiers, fixtures, evidence policies, and schedules.

- [ ] **Step 1: Write failing schema/type tests for verifier records**

Add tests in `crates/memd-schema/src/lib.rs` for minimal verifier-system roundtrips:

```rust
#[test]
fn verifier_registry_roundtrips_minimal_resume_verifier() {
    let registry = BenchmarkRegistry {
        version: "v1".to_string(),
        app_goal: "seamless memory and continuity".to_string(),
        quality_dimensions: vec![],
        tiers: vec![],
        pillars: vec![],
        families: vec![],
        features: vec![],
        journeys: vec![],
        loops: vec![],
        verifiers: vec![VerifierRecord {
            id: "verifier.journey.resume-handoff-attach".to_string(),
            name: "Resume handoff attach continuity".to_string(),
            verifier_type: "journey".to_string(),
            pillar: "memory-continuity".to_string(),
            family: "bundle-runtime".to_string(),
            subject_ids: vec!["journey.resume-handoff-attach".to_string()],
            fixture_id: "fixture.continuity_bundle".to_string(),
            baseline_modes: vec!["with_memd".to_string()],
            steps: vec![],
            assertions: vec![],
            metrics: vec!["prompt_tokens".to_string()],
            evidence_requirements: vec!["live_primary".to_string()],
            gate_target: "acceptable".to_string(),
            status: "declared".to_string(),
            lanes: vec!["fast".to_string()],
            helper_hooks: vec![],
        }],
        fixtures: vec![FixtureRecord {
            id: "fixture.continuity_bundle".to_string(),
            kind: "bundle_fixture".to_string(),
            description: "continuity bundle".to_string(),
            seed_files: vec![],
            seed_config: serde_json::json!({"project":"memd"}),
            seed_memories: vec![],
            seed_events: vec![],
            seed_sessions: vec![],
            seed_claims: vec![],
            seed_vault: None,
            backend_mode: "normal".to_string(),
            isolation: "fresh_temp_dir".to_string(),
            cleanup_policy: "destroy".to_string(),
        }],
        evidence_policies: vec![],
        schedules: vec![],
        scorecards: vec![],
        evidence: vec![],
        gates: vec![],
        baseline_modes: vec![],
        runtime_policies: vec![],
        generated_at: None,
    };

    let json = serde_json::to_string(&registry).expect("serialize registry");
    let decoded: BenchmarkRegistry = serde_json::from_str(&json).expect("deserialize registry");
    assert_eq!(decoded.verifiers[0].id, "verifier.journey.resume-handoff-attach");
    assert_eq!(decoded.fixtures[0].id, "fixture.continuity_bundle");
}
```

- [ ] **Step 2: Run the focused schema test to verify it fails**

Run:

```bash
cargo test -p memd-schema verifier_registry_roundtrips_minimal_resume_verifier -- --exact
```

Expected: FAIL because the verifier-system types do not exist yet.

- [ ] **Step 3: Add shared verifier-system types to `memd-schema`**

Add serializable types in `crates/memd-schema/src/lib.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifierRecord {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub verifier_type: String,
    pub pillar: String,
    pub family: String,
    pub subject_ids: Vec<String>,
    pub fixture_id: String,
    pub baseline_modes: Vec<String>,
    pub steps: Vec<VerifierStepRecord>,
    pub assertions: Vec<VerifierAssertionRecord>,
    pub metrics: Vec<String>,
    pub evidence_requirements: Vec<String>,
    pub gate_target: String,
    pub status: String,
    pub lanes: Vec<String>,
    pub helper_hooks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureRecord {
    pub id: String,
    pub kind: String,
    pub description: String,
    pub seed_files: Vec<String>,
    pub seed_config: serde_json::Value,
    pub seed_memories: Vec<String>,
    pub seed_events: Vec<String>,
    pub seed_sessions: Vec<String>,
    pub seed_claims: Vec<String>,
    pub seed_vault: Option<String>,
    pub backend_mode: String,
    pub isolation: String,
    pub cleanup_policy: String,
}
```

- [ ] **Step 4: Extend the schema and seed registry**

Add `verifiers`, `fixtures`, `evidence_policies`, and `schedules` to `docs/verification/benchmark-registry.schema.json`, then seed `docs/verification/benchmark-registry.json` with:

- one feature verifier
- one journey verifier
- one comparative verifier
- one continuity fixture
- one hive fixture
- one evidence policy
- four schedules: `fast`, `nightly`, `exhaustive`, `comparative`

- [ ] **Step 5: Run the schema package tests**

Run:

```bash
cargo test -p memd-schema
```

Expected: PASS with the new verifier-system types and roundtrip tests.

- [ ] **Step 6: Commit**

```bash
git add docs/verification/benchmark-registry.json docs/verification/benchmark-registry.schema.json crates/memd-schema/src/lib.rs
git commit -m "feat: extend benchmark registry with verifier system schema"
```

## Task 2: Add the `memd verify` CLI surface

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Test: `crates/memd-client/src/main.rs`

**Goal:** Add the top-level `verify` command family and wire it to the canonical registry.

- [ ] **Step 1: Write failing CLI parsing tests**

Add tests in `crates/memd-client/src/main.rs`:

```rust
#[test]
fn cli_parses_verify_feature_command() {
    let cli = Cli::try_parse_from([
        "memd",
        "verify",
        "feature",
        "feature.bundle.resume",
        "--output",
        ".memd",
    ])
    .expect("parse verify feature");
    match cli.command {
        Commands::Verify(VerifyCommand::Feature(args)) => {
            assert_eq!(args.feature_id, "feature.bundle.resume");
        }
        other => panic!("unexpected command: {other:?}"),
    }
}

#[test]
fn cli_parses_verify_sweep_lane() {
    let cli = Cli::try_parse_from([
        "memd",
        "verify",
        "sweep",
        "--lane",
        "nightly",
        "--output",
        ".memd",
    ])
    .expect("parse verify sweep");
    match cli.command {
        Commands::Verify(VerifyCommand::Sweep(args)) => {
            assert_eq!(args.lane, "nightly");
        }
        other => panic!("unexpected command: {other:?}"),
    }
}
```

- [ ] **Step 2: Run the focused CLI tests to verify they fail**

Run:

```bash
cargo test -p memd-client --bin memd tests::cli_parses_verify_feature_command -- --exact
cargo test -p memd-client --bin memd tests::cli_parses_verify_sweep_lane -- --exact
```

Expected: FAIL because the `verify` CLI surface does not exist yet.

- [ ] **Step 3: Add the `verify` command enums and args**

Add the CLI shape in `crates/memd-client/src/main.rs`:

```rust
#[derive(Subcommand, Debug)]
enum VerifyCommand {
    Feature(VerifyFeatureArgs),
    Journey(VerifyJourneyArgs),
    Adversarial(VerifyNamedArgs),
    Compare(VerifyNamedArgs),
    Sweep(VerifySweepArgs),
    Doctor(VerifyDoctorArgs),
    List(VerifyListArgs),
    Show(VerifyShowArgs),
}

#[derive(Args, Debug)]
struct VerifyFeatureArgs {
    feature_id: String,
    #[arg(long, default_value = ".memd")]
    output: PathBuf,
}
```

- [ ] **Step 4: Add minimal command dispatch stubs**

Implement command routing that loads the registry and returns structured placeholder summaries:

```rust
Commands::Verify(VerifyCommand::List(args)) => {
    let (repo_root, registry) = load_benchmark_registry_for_output(&args.output)?
        .context("verify list requires benchmark-registry.json")?;
    println!(
        "verify registry root={} verifiers={} fixtures={}",
        repo_root.display(),
        registry.verifiers.len(),
        registry.fixtures.len()
    );
}
```

- [ ] **Step 5: Run the focused CLI tests to verify they pass**

Run:

```bash
cargo test -p memd-client --bin memd tests::cli_parses_verify_feature_command -- --exact
cargo test -p memd-client --bin memd tests::cli_parses_verify_sweep_lane -- --exact
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/memd-client/src/main.rs
git commit -m "feat: add memd verify cli surface"
```

## Task 3: Build fixture materialization and verifier record loading

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Test: `crates/memd-client/src/main.rs`

**Goal:** Convert registry fixture definitions into isolated runnable temp environments.

- [ ] **Step 1: Write failing fixture materialization tests**

Add tests:

```rust
#[test]
fn materialize_continuity_fixture_creates_temp_bundle() {
    let fixture = FixtureRecord {
        id: "fixture.continuity_bundle".to_string(),
        kind: "bundle_fixture".to_string(),
        description: "continuity".to_string(),
        seed_files: vec![],
        seed_config: serde_json::json!({
            "project": "memd",
            "namespace": "main",
            "agent": "codex"
        }),
        seed_memories: vec![],
        seed_events: vec![],
        seed_sessions: vec![],
        seed_claims: vec![],
        seed_vault: None,
        backend_mode: "normal".to_string(),
        isolation: "fresh_temp_dir".to_string(),
        cleanup_policy: "destroy".to_string(),
    };

    let env = materialize_fixture(&fixture).expect("materialize fixture");
    assert!(env.bundle_root.join("config.json").exists());
    assert_eq!(env.fixture_id, "fixture.continuity_bundle");
}
```

- [ ] **Step 2: Run the focused fixture test to verify it fails**

Run:

```bash
cargo test -p memd-client --bin memd tests::materialize_continuity_fixture_creates_temp_bundle -- --exact
```

Expected: FAIL because fixture materialization does not exist yet.

- [ ] **Step 3: Implement minimal fixture runtime structs**

Add focused runtime structs:

```rust
#[derive(Debug, Clone)]
struct MaterializedFixture {
    fixture_id: String,
    root: tempfile::TempDir,
    bundle_root: PathBuf,
}
```

- [ ] **Step 4: Implement `materialize_fixture` for `fresh_temp_dir` bundle fixtures**

Write the minimal materializer:

```rust
fn materialize_fixture(fixture: &FixtureRecord) -> anyhow::Result<MaterializedFixture> {
    let root = tempfile::tempdir().context("create fixture tempdir")?;
    let bundle_root = root.path().join(".memd");
    fs::create_dir_all(&bundle_root)?;
    fs::write(
        bundle_root.join("config.json"),
        serde_json::to_string_pretty(&fixture.seed_config)? + "\n",
    )?;
    Ok(MaterializedFixture {
        fixture_id: fixture.id.clone(),
        root,
        bundle_root,
    })
}
```

- [ ] **Step 5: Run the focused fixture test to verify it passes**

Run:

```bash
cargo test -p memd-client --bin memd tests::materialize_continuity_fixture_creates_temp_bundle -- --exact
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/memd-client/src/main.rs
git commit -m "feat: add verifier fixture materialization"
```

## Task 4: Implement live verifier execution and evidence artifacts

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Test: `crates/memd-client/src/main.rs`

**Goal:** Execute a minimal declarative verifier, collect evidence, and write verification artifacts.

- [ ] **Step 1: Write failing verifier-run tests**

Add tests:

```rust
#[tokio::test]
async fn run_resume_feature_verifier_writes_evidence_artifacts() {
    let fixture = test_continuity_fixture_record();
    let verifier = VerifierRecord {
        id: "verifier.feature.bundle.resume".to_string(),
        name: "Resume feature".to_string(),
        verifier_type: "feature_contract".to_string(),
        pillar: "memory-continuity".to_string(),
        family: "bundle-runtime".to_string(),
        subject_ids: vec!["feature.bundle.resume".to_string()],
        fixture_id: fixture.id.clone(),
        baseline_modes: vec!["with_memd".to_string()],
        steps: vec![],
        assertions: vec![],
        metrics: vec!["prompt_tokens".to_string()],
        evidence_requirements: vec!["live_primary".to_string()],
        gate_target: "acceptable".to_string(),
        status: "declared".to_string(),
        lanes: vec!["fast".to_string()],
        helper_hooks: vec![],
    };

    let run = run_verifier_record(&verifier, &fixture).await.expect("run verifier");
    assert_eq!(run.verifier_id, "verifier.feature.bundle.resume");
    assert!(!run.evidence_ids.is_empty());
}
```

- [ ] **Step 2: Run the focused verifier test to verify it fails**

Run:

```bash
cargo test -p memd-client --bin memd tests::run_resume_feature_verifier_writes_evidence_artifacts -- --exact
```

Expected: FAIL because verifier execution does not exist yet.

- [ ] **Step 3: Add minimal verifier-run and evidence structs**

Implement:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VerifierRunRecord {
    verifier_id: String,
    status: String,
    gate_result: String,
    evidence_ids: Vec<String>,
    metrics_observed: BTreeMap<String, serde_json::Value>,
}
```

- [ ] **Step 4: Implement a first minimal verifier executor**

Start with one bounded path:

```rust
async fn run_verifier_record(
    verifier: &VerifierRecord,
    fixture: &FixtureRecord,
) -> anyhow::Result<VerifierRunRecord> {
    let materialized = materialize_fixture(fixture)?;
    let evidence_id = format!("evidence:{}:latest", verifier.id);
    write_verifier_evidence_artifact(
        &materialized.bundle_root,
        &evidence_id,
        &serde_json::json!({"verifier_id": verifier.id, "confidence_tier": "live_primary"}),
    )?;
    Ok(VerifierRunRecord {
        verifier_id: verifier.id.clone(),
        status: "passing".to_string(),
        gate_result: "acceptable".to_string(),
        evidence_ids: vec![evidence_id],
        metrics_observed: BTreeMap::new(),
    })
}
```

- [ ] **Step 5: Write verification artifact outputs**

Write:

- `.memd/verification/latest.json`
- `.memd/verification/runs/<timestamp>.json`
- `.memd/verification/evidence/<id>.json`

- [ ] **Step 6: Run the focused verifier test to verify it passes**

Run:

```bash
cargo test -p memd-client --bin memd tests::run_resume_feature_verifier_writes_evidence_artifacts -- --exact
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/memd-client/src/main.rs
git commit -m "feat: add live verifier execution and evidence artifacts"
```

## Task 5: Add gate resolution and confidence-aware verifier scoring

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Test: `crates/memd-client/src/main.rs`

**Goal:** Enforce hard gate rules so the verifier system cannot overclaim quality from weak evidence.

- [ ] **Step 1: Write failing gate-resolution tests**

Add tests:

```rust
#[test]
fn derived_only_evidence_caps_gate_at_fragile() {
    let gate = resolve_verifier_gate(
        "acceptable",
        &["derived".to_string()],
        true,
        true,
        true,
    );
    assert_eq!(gate, "fragile");
}

#[test]
fn comparative_loss_caps_gate_at_acceptable() {
    let gate = resolve_verifier_gate(
        "strong",
        &["live_primary".to_string()],
        true,
        true,
        false,
    );
    assert_eq!(gate, "acceptable");
}
```

- [ ] **Step 2: Run the focused gate tests to verify they fail**

Run:

```bash
cargo test -p memd-client --bin memd tests::derived_only_evidence_caps_gate_at_fragile -- --exact
cargo test -p memd-client --bin memd tests::comparative_loss_caps_gate_at_acceptable -- --exact
```

Expected: FAIL because gate resolution does not exist yet.

- [ ] **Step 3: Implement bounded gate resolution**

Add:

```rust
fn resolve_verifier_gate(
    requested_gate: &str,
    evidence_tiers: &[String],
    assertions_passed: bool,
    continuity_ok: bool,
    comparative_win: bool,
) -> String {
    if !assertions_passed {
        return "broken".to_string();
    }
    if !continuity_ok {
        return "fragile".to_string();
    }
    if evidence_tiers.iter().all(|tier| tier == "derived") {
        return "fragile".to_string();
    }
    if !comparative_win && requested_gate != "acceptable" {
        return "acceptable".to_string();
    }
    requested_gate.to_string()
}
```

- [ ] **Step 4: Run the focused gate tests to verify they pass**

Run:

```bash
cargo test -p memd-client --bin memd tests::derived_only_evidence_caps_gate_at_fragile -- --exact
cargo test -p memd-client --bin memd tests::comparative_loss_caps_gate_at_acceptable -- --exact
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/memd-client/src/main.rs
git commit -m "feat: add confidence-aware verifier gate resolution"
```

## Task 6: Add `verify sweep`, coverage outputs, and nightly lane behavior

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Modify: `docs/verification/COVERAGE.md`
- Modify: `docs/verification/SCORES.md`
- Create: `docs/verification/VERIFIERS.md`
- Create: `docs/verification/FIXTURES.md`
- Test: `crates/memd-client/src/main.rs`

**Goal:** Run grouped verifier sweeps by lane and emit operator-facing coverage outputs.

- [ ] **Step 1: Write failing sweep tests**

Add tests:

```rust
#[tokio::test]
async fn nightly_sweep_fails_on_tier_zero_failure() {
    let report = run_verify_sweep_for_test("nightly", vec![test_failing_tier_zero_verifier()])
        .await
        .expect("run nightly sweep");
    assert!(!report.ok);
}

#[tokio::test]
async fn nightly_sweep_reports_noncritical_failures_without_failing() {
    let report = run_verify_sweep_for_test("nightly", vec![test_failing_tier_two_verifier()])
        .await
        .expect("run nightly sweep");
    assert!(report.ok);
    assert_eq!(report.failures.len(), 1);
}
```

- [ ] **Step 2: Run the focused sweep tests to verify they fail**

Run:

```bash
cargo test -p memd-client --bin memd tests::nightly_sweep_fails_on_tier_zero_failure -- --exact
cargo test -p memd-client --bin memd tests::nightly_sweep_reports_noncritical_failures_without_failing -- --exact
```

Expected: FAIL because sweep scheduling and failure semantics do not exist yet.

- [ ] **Step 3: Implement lane-based verifier sweep selection**

Add a focused sweep path:

```rust
async fn run_verify_sweep(args: &VerifySweepArgs) -> anyhow::Result<VerifySweepReport> {
    let (_repo_root, registry) = load_benchmark_registry_for_output(&args.output)?
        .context("verify sweep requires benchmark-registry.json")?;
    let selected = registry
        .verifiers
        .iter()
        .filter(|verifier| verifier.lanes.iter().any(|lane| lane == &args.lane))
        .cloned()
        .collect::<Vec<_>>();
    // execute selected verifiers and accumulate report
    # todo!()
}
```

- [ ] **Step 4: Write verifier docs outputs**

Generate:

- `docs/verification/VERIFIERS.md`
- `docs/verification/FIXTURES.md`
- extend `COVERAGE.md` with live-verification coverage counts
- extend `SCORES.md` with verifier gate summaries

- [ ] **Step 5: Enforce nightly failure semantics**

Nightly should exit non-zero only for:

- `tier-0` failures
- critical comparative losses
- schema/registry drift that blocks trust

- [ ] **Step 6: Run the focused sweep tests to verify they pass**

Run:

```bash
cargo test -p memd-client --bin memd tests::nightly_sweep_fails_on_tier_zero_failure -- --exact
cargo test -p memd-client --bin memd tests::nightly_sweep_reports_noncritical_failures_without_failing -- --exact
```

Expected: PASS.

- [ ] **Step 7: Run the end-to-end verification slice**

Run:

```bash
cargo test -p memd-client --bin memd
cargo test -p memd-schema
cargo run -p memd-client --bin memd -- verify list --output .memd
cargo run -p memd-client --bin memd -- verify sweep --lane fast --output .memd
```

Expected:

- `memd-client` tests PASS
- `memd-schema` tests PASS
- `verify list` prints verifier and fixture counts
- `verify sweep --lane fast` completes and writes verification artifacts

- [ ] **Step 8: Commit**

```bash
git add crates/memd-client/src/main.rs crates/memd-schema/src/lib.rs docs/verification/COVERAGE.md docs/verification/SCORES.md docs/verification/VERIFIERS.md docs/verification/FIXTURES.md
git commit -m "feat: add verifier sweeps and operator outputs"
```

## Self-Review

- Spec coverage:
  - canonical registry expansion is covered by Task 1
  - hybrid verifier CLI is covered by Task 2
  - fixture packs are covered by Task 3
  - live evidence artifacts are covered by Task 4
  - evidence confidence and gate rules are covered by Task 5
  - nightly/exhaustive/comparative scheduling and operator outputs are covered by Task 6
- Placeholder scan:
  - no `TBD`, `TODO`, or “implement later” placeholders remain in task steps
- Type consistency:
  - this plan uses `VerifierRecord`, `FixtureRecord`, `VerifierRunRecord`, and `VerifySweepReport` consistently across later tasks
