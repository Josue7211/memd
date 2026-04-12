# Benchmark Registry 10-Star Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship the first usable 10-star benchmark registry slice for `memd` so the app can prove continuity-critical quality, compare `no memd` vs `with memd`, generate evidence-backed scorecards, and surface actionable gaps for a solo maintainer.

**Architecture:** Build on the existing `memd benchmark`, `memd loops`, `memd telemetry`, and feature benchmark artifact path instead of creating a second benchmarking stack. Add a canonical docs-backed registry plus schema, shared serializable registry/report types, a registry-aware benchmark execution path in `crates/memd-client/src/main.rs`, and generated docs/telemetry outputs that start with tier-0 continuity-critical features and journeys before expanding into the full catalog.

**Tech Stack:** Rust, clap, serde/serde_json, existing `memd benchmark` command, existing loop and telemetry artifacts, markdown/json docs in `docs/verification`, existing `#[cfg(test)]` coverage in `crates/memd-client/src/main.rs` and `crates/memd-schema/src/lib.rs`.

---

## File Structure

- `docs/verification/benchmark-registry.schema.json`
  - canonical JSON Schema for the benchmark registry
- `docs/verification/benchmark-registry.json`
  - canonical machine-readable benchmark graph
- `docs/verification/BENCHMARKS.md`
  - generated feature/journey/family/pillar benchmark summary
- `docs/verification/LOOPS.md`
  - generated loop catalog
- `docs/verification/COVERAGE.md`
  - generated coverage and gap summary
- `docs/verification/SCORES.md`
  - generated scorecards and gate summary
- `docs/verification/FEATURES.md`
  - existing feature contract ledger; align feature IDs and references where needed
- `crates/memd-schema/src/lib.rs`
  - shared benchmark registry/report/evidence/scorecard/runtime-policy types
- `crates/memd-client/src/main.rs`
  - benchmark registry loader
  - schema validation shell-out or inline JSON validation glue
  - benchmark command orchestration
  - score resolution rules
  - generated markdown renderers
  - no-memd vs with-memd comparison logic
  - morning summary renderer
  - autoresearch gap proposal integration
- `crates/memd-client/src/lib.rs`
  - only if a shared client helper is needed for registry-aware telemetry fetches

## Task 1: Create the canonical benchmark registry and schema

**Files:**
- Create: `docs/verification/benchmark-registry.schema.json`
- Create: `docs/verification/benchmark-registry.json`
- Modify: `docs/verification/FEATURES.md`
- Modify: `crates/memd-schema/src/lib.rs`
- Test: `crates/memd-schema/src/lib.rs`

**Goal:** Establish the machine-truth benchmark graph and shared types for the minimal executable slice.

- [ ] **Step 1: Write the failing schema/type roundtrip tests**

Add tests in `crates/memd-schema/src/lib.rs`:

```rust
    #[test]
    fn benchmark_registry_roundtrips_minimal_continuity_slice() {
        let registry = BenchmarkRegistry {
            version: "v1".to_string(),
            app_goal: "seamless memory and continuity".to_string(),
            quality_dimensions: vec![
                QualityDimensionRecord { id: "continuity".to_string(), weight: 25 },
                QualityDimensionRecord { id: "correctness".to_string(), weight: 20 },
            ],
            tiers: vec![TierRecord {
                id: "tier-0-continuity-critical".to_string(),
                description: "continuity-critical surfaces".to_string(),
            }],
            pillars: vec![PillarRecord {
                id: "memory-continuity".to_string(),
                description: "core continuity promise".to_string(),
            }],
            families: vec![FamilyRecord {
                id: "bundle-runtime".to_string(),
                pillar: "memory-continuity".to_string(),
                description: "bundle continuity surfaces".to_string(),
            }],
            features: vec![BenchmarkFeatureRecord {
                id: "feature.bundle.resume".to_string(),
                name: "Resume".to_string(),
                pillar: "memory-continuity".to_string(),
                family: "bundle-runtime".to_string(),
                tier: "tier-0-continuity-critical".to_string(),
                continuity_critical: true,
                user_contract: "resume restores usable continuity".to_string(),
                source_contract_refs: vec!["FEATURE-V1-WORKING-CONTEXT".to_string()],
                commands: vec!["memd resume".to_string()],
                routes: vec![],
                files: vec!["crates/memd-client/src/main.rs".to_string()],
                journey_ids: vec!["journey.continuity.resume-handoff-attach".to_string()],
                loop_ids: vec!["loop.resume.correctness".to_string()],
                quality_dimensions: vec!["continuity".to_string(), "correctness".to_string()],
                drift_risks: vec!["continuity-drift".to_string()],
                failure_modes: vec!["resume misses current task state".to_string()],
                coverage_status: "auditing".to_string(),
                last_verified_at: None,
            }],
            journeys: vec![],
            loops: vec![],
            scorecards: vec![],
            evidence: vec![],
            gates: vec![],
            baseline_modes: vec![],
            runtime_policies: vec![],
            generated_at: None,
        };

        let json = serde_json::to_string(&registry).expect("serialize registry");
        let decoded: BenchmarkRegistry =
            serde_json::from_str(&json).expect("deserialize registry");
        assert_eq!(decoded.features[0].id, "feature.bundle.resume");
        assert!(decoded.features[0].continuity_critical);
    }

    #[test]
    fn benchmark_score_resolution_rules_roundtrip() {
        let rules = ScoreResolutionRules {
            cap_on_continuity_failure: "fragile".to_string(),
            cap_on_missing_required_evidence: "fragile".to_string(),
            cap_on_no_memd_loss: "acceptable".to_string(),
        };
        let json = serde_json::to_string(&rules).expect("serialize score rules");
        let decoded: ScoreResolutionRules =
            serde_json::from_str(&json).expect("deserialize score rules");
        assert_eq!(decoded.cap_on_continuity_failure, "fragile");
    }
```

- [ ] **Step 2: Run the focused schema tests to verify they fail**

Run:

```bash
cargo test -p memd-schema benchmark_registry_roundtrips_minimal_continuity_slice -- --exact
cargo test -p memd-schema benchmark_score_resolution_rules_roundtrip -- --exact
```

Expected: FAIL because the benchmark registry types do not exist yet.

- [ ] **Step 3: Add the shared benchmark registry types**

Add the shared types in `crates/memd-schema/src/lib.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkRegistry {
    pub version: String,
    pub app_goal: String,
    pub quality_dimensions: Vec<QualityDimensionRecord>,
    pub tiers: Vec<TierRecord>,
    pub pillars: Vec<PillarRecord>,
    pub families: Vec<FamilyRecord>,
    pub features: Vec<BenchmarkFeatureRecord>,
    pub journeys: Vec<BenchmarkJourneyRecord>,
    pub loops: Vec<BenchmarkLoopRecord>,
    pub scorecards: Vec<BenchmarkScorecardRecord>,
    pub evidence: Vec<BenchmarkEvidenceRecord>,
    pub gates: Vec<BenchmarkGateRecord>,
    pub baseline_modes: Vec<BaselineModeRecord>,
    pub runtime_policies: Vec<RuntimePolicyRecord>,
    pub generated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkFeatureRecord {
    pub id: String,
    pub name: String,
    pub pillar: String,
    pub family: String,
    pub tier: String,
    pub continuity_critical: bool,
    pub user_contract: String,
    pub source_contract_refs: Vec<String>,
    pub commands: Vec<String>,
    pub routes: Vec<String>,
    pub files: Vec<String>,
    pub journey_ids: Vec<String>,
    pub loop_ids: Vec<String>,
    pub quality_dimensions: Vec<String>,
    pub drift_risks: Vec<String>,
    pub failure_modes: Vec<String>,
    pub coverage_status: String,
    pub last_verified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreResolutionRules {
    pub cap_on_continuity_failure: String,
    pub cap_on_missing_required_evidence: String,
    pub cap_on_no_memd_loss: String,
}
```

- [ ] **Step 4: Write the canonical schema and seed registry**

Create `docs/verification/benchmark-registry.schema.json` with a top-level shape that requires:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "memd benchmark registry",
  "type": "object",
  "required": [
    "version",
    "app_goal",
    "quality_dimensions",
    "tiers",
    "pillars",
    "families",
    "features",
    "journeys",
    "loops",
    "scorecards",
    "evidence",
    "gates",
    "baseline_modes",
    "runtime_policies"
  ],
  "properties": {
    "version": { "type": "string" },
    "app_goal": { "type": "string" },
    "features": {
      "type": "array",
      "items": {
        "type": "object",
        "required": [
          "id",
          "name",
          "pillar",
          "family",
          "tier",
          "continuity_critical",
          "user_contract",
          "source_contract_refs",
          "commands",
          "journey_ids",
          "loop_ids",
          "quality_dimensions",
          "drift_risks",
          "failure_modes",
          "coverage_status"
        ]
      }
    }
  }
}
```

Create `docs/verification/benchmark-registry.json` seeded with the minimal executable slice:

```json
{
  "version": "v1",
  "app_goal": "memd as a seamless memory control plane with near-perfect continuity",
  "quality_dimensions": [
    { "id": "continuity", "weight": 25 },
    { "id": "correctness", "weight": 20 },
    { "id": "reliability", "weight": 15 },
    { "id": "drift_resistance", "weight": 15 },
    { "id": "token_efficiency", "weight": 10 }
  ],
  "tiers": [
    {
      "id": "tier-0-continuity-critical",
      "description": "continuity-critical surfaces"
    }
  ],
  "pillars": [
    {
      "id": "memory-continuity",
      "description": "seamless continuity surfaces"
    }
  ],
  "families": [
    {
      "id": "bundle-runtime",
      "pillar": "memory-continuity",
      "description": "bundle continuity surfaces"
    },
    {
      "id": "capture-compaction",
      "pillar": "memory-continuity",
      "description": "capture and compaction continuity surfaces"
    },
    {
      "id": "drift-prevention",
      "pillar": "memory-continuity",
      "description": "continuity drift controls"
    }
  ],
  "features": [
    {
      "id": "feature.bundle.wake",
      "name": "Wake",
      "pillar": "memory-continuity",
      "family": "bundle-runtime",
      "tier": "tier-0-continuity-critical",
      "continuity_critical": true,
      "user_contract": "wake refreshes the startup memory surface before a turn",
      "source_contract_refs": [],
      "commands": ["memd wake"],
      "routes": [],
      "files": ["crates/memd-client/src/main.rs"],
      "journey_ids": ["journey.continuity.resume-handoff-attach"],
      "loop_ids": ["loop.feature.wake.correctness"],
      "quality_dimensions": ["continuity", "correctness", "token_efficiency"],
      "drift_risks": ["continuity-drift", "surface-drift"],
      "failure_modes": ["wake file not refreshed"],
      "coverage_status": "auditing",
      "last_verified_at": null
    }
  ],
  "journeys": [],
  "loops": [],
  "scorecards": [],
  "evidence": [],
  "gates": [],
  "baseline_modes": [],
  "runtime_policies": []
}
```

- [ ] **Step 5: Align `FEATURES.md` references with the registry**

Update `docs/verification/FEATURES.md` to add or align stable IDs in notes for tier-0 surfaces where the existing contract already exists. Use the benchmark feature IDs exactly:

```md
#### Notes

- benchmark_feature_id: `feature.bundle.resume`
- benchmark_tier: `tier-0-continuity-critical`
```

- [ ] **Step 6: Run the schema tests to verify they pass**

Run:

```bash
cargo test -p memd-schema benchmark_registry_roundtrips_minimal_continuity_slice -- --exact
cargo test -p memd-schema benchmark_score_resolution_rules_roundtrip -- --exact
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add docs/verification/benchmark-registry.schema.json docs/verification/benchmark-registry.json docs/verification/FEATURES.md crates/memd-schema/src/lib.rs
git commit -m "feat: add canonical benchmark registry backbone"
```

## Task 2: Make `memd benchmark` registry-aware and generate docs-backed outputs

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Test: `crates/memd-client/src/main.rs`

**Goal:** Teach the existing benchmark command to load the canonical registry, validate minimal expectations, and write generated benchmark summaries.

- [ ] **Step 1: Write the failing benchmark loader/render tests**

Add tests in `crates/memd-client/src/main.rs`:

```rust
    #[test]
    fn load_benchmark_registry_reads_docs_registry() {
        let dir = std::env::temp_dir().join(format!(
            "memd-benchmark-registry-load-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(dir.join("docs/verification")).expect("create docs dir");
        fs::write(
            dir.join("docs/verification/benchmark-registry.json"),
            r#"{
              "version":"v1",
              "app_goal":"demo",
              "quality_dimensions":[],
              "tiers":[],
              "pillars":[],
              "families":[],
              "features":[],
              "journeys":[],
              "loops":[],
              "scorecards":[],
              "evidence":[],
              "gates":[],
              "baseline_modes":[],
              "runtime_policies":[]
            }"#,
        )
        .expect("write registry");

        let registry = load_benchmark_registry(&dir).expect("load registry");
        assert_eq!(registry.version, "v1");
    }

    #[test]
    fn write_benchmark_registry_docs_writes_all_expected_outputs() {
        let dir = std::env::temp_dir().join(format!(
            "memd-benchmark-registry-docs-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(dir.join("docs/verification")).expect("create docs dir");
        let registry = BenchmarkRegistry {
            version: "v1".to_string(),
            app_goal: "demo".to_string(),
            quality_dimensions: Vec::new(),
            tiers: Vec::new(),
            pillars: Vec::new(),
            families: Vec::new(),
            features: Vec::new(),
            journeys: Vec::new(),
            loops: Vec::new(),
            scorecards: Vec::new(),
            evidence: Vec::new(),
            gates: Vec::new(),
            baseline_modes: Vec::new(),
            runtime_policies: Vec::new(),
            generated_at: None,
        };
        let report = BenchmarkRegistryReport {
            registry,
            summary_markdown: "# Benchmarks\n".to_string(),
            loops_markdown: "# Loops\n".to_string(),
            coverage_markdown: "# Coverage\n".to_string(),
            scores_markdown: "# Scores\n".to_string(),
            morning_markdown: "# Morning\n".to_string(),
        };

        write_benchmark_registry_docs(&dir, &report).expect("write benchmark docs");
        assert!(dir.join("docs/verification/BENCHMARKS.md").exists());
        assert!(dir.join("docs/verification/LOOPS.md").exists());
        assert!(dir.join("docs/verification/COVERAGE.md").exists());
        assert!(dir.join("docs/verification/SCORES.md").exists());
    }
```

- [ ] **Step 2: Run the focused tests to verify they fail**

Run:

```bash
cargo test -p memd-client --bin memd load_benchmark_registry_reads_docs_registry -- --exact
cargo test -p memd-client --bin memd write_benchmark_registry_docs_writes_all_expected_outputs -- --exact
```

Expected: FAIL because the benchmark registry loader/writer does not exist yet.

- [ ] **Step 3: Add registry-aware benchmark report structs and helpers**

Add in `crates/memd-client/src/main.rs`:

```rust
struct BenchmarkRegistryReport {
    registry: BenchmarkRegistry,
    summary_markdown: String,
    loops_markdown: String,
    coverage_markdown: String,
    scores_markdown: String,
    morning_markdown: String,
}

fn load_benchmark_registry(repo_root: &Path) -> anyhow::Result<BenchmarkRegistry> {
    let path = repo_root.join("docs/verification/benchmark-registry.json");
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("read {}", path.display()))?;
    let registry: BenchmarkRegistry = serde_json::from_str(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(registry)
}

fn write_benchmark_registry_docs(
    repo_root: &Path,
    report: &BenchmarkRegistryReport,
) -> anyhow::Result<()> {
    fs::write(
        repo_root.join("docs/verification/BENCHMARKS.md"),
        &report.summary_markdown,
    )?;
    fs::write(
        repo_root.join("docs/verification/LOOPS.md"),
        &report.loops_markdown,
    )?;
    fs::write(
        repo_root.join("docs/verification/COVERAGE.md"),
        &report.coverage_markdown,
    )?;
    fs::write(
        repo_root.join("docs/verification/SCORES.md"),
        &report.scores_markdown,
    )?;
    Ok(())
}
```

- [ ] **Step 4: Extend `run_feature_benchmark_command` to load the registry**

Update `run_feature_benchmark_command` in `crates/memd-client/src/main.rs` so it:

- loads the registry from the repo root inferred from `args.output`
- verifies the tier-0 feature IDs and journey IDs are present
- carries the registry into the benchmark response path
- reuses the existing feature benchmark artifacts under `.memd/benchmarks/features/`

Use this shape inside the command:

```rust
let repo_root = args
    .output
    .parent()
    .context("benchmark output must live under a repo root")?;
let registry = load_benchmark_registry(repo_root)?;
let continuity_features = registry
    .features
    .iter()
    .filter(|feature| feature.tier == "tier-0-continuity-critical")
    .collect::<Vec<_>>();
```

- [ ] **Step 5: Generate the docs-backed benchmark markdown outputs**

Add render helpers in `crates/memd-client/src/main.rs`:

```rust
fn render_benchmark_registry_summary(report: &BenchmarkRegistryReport) -> String {
    format!(
        "benchmark_registry version={} features={} journeys={} loops={}",
        report.registry.version,
        report.registry.features.len(),
        report.registry.journeys.len(),
        report.registry.loops.len()
    )
}

fn render_benchmarks_markdown(registry: &BenchmarkRegistry) -> String {
    format!(
        "# Benchmarks\n\n- features: {}\n- journeys: {}\n- loops: {}\n",
        registry.features.len(),
        registry.journeys.len(),
        registry.loops.len()
    )
}
```

Wire `args.write` so the benchmark command writes the generated docs:

```rust
if args.write {
    let registry_report = build_benchmark_registry_report(&registry, &response)?;
    write_benchmark_registry_docs(repo_root, &registry_report)?;
}
```

- [ ] **Step 6: Run the focused tests to verify they pass**

Run:

```bash
cargo test -p memd-client --bin memd load_benchmark_registry_reads_docs_registry -- --exact
cargo test -p memd-client --bin memd write_benchmark_registry_docs_writes_all_expected_outputs -- --exact
```

Expected: PASS.

- [ ] **Step 7: Run the benchmark command manually**

Run:

```bash
cargo run -p memd-client --bin memd -- benchmark --output .memd --summary
cargo run -p memd-client --bin memd -- benchmark --output .memd --write
```

Expected:

- summary prints a benchmark registry summary line
- `docs/verification/BENCHMARKS.md`
- `docs/verification/LOOPS.md`
- `docs/verification/COVERAGE.md`
- `docs/verification/SCORES.md`

are written or refreshed.

- [ ] **Step 8: Commit**

```bash
git add crates/memd-client/src/main.rs docs/verification/BENCHMARKS.md docs/verification/LOOPS.md docs/verification/COVERAGE.md docs/verification/SCORES.md
git commit -m "feat: make benchmark command registry-aware"
```

## Task 3: Implement tier-0 continuity journeys, evidence, and hard score resolution

**Files:**
- Modify: `docs/verification/benchmark-registry.json`
- Modify: `crates/memd-schema/src/lib.rs`
- Modify: `crates/memd-client/src/main.rs`
- Test: `crates/memd-client/src/main.rs`
- Test: `crates/memd-schema/src/lib.rs`

**Goal:** Turn the minimal slice into a real continuity-quality benchmark with explicit evidence and gate caps.

- [ ] **Step 1: Write failing continuity gate tests**

Add tests in `crates/memd-client/src/main.rs`:

```rust
    #[test]
    fn continuity_failure_caps_gate_at_fragile() {
        let scorecard = resolve_benchmark_scorecard(
            &BenchmarkSubjectMetrics {
                correctness: 92,
                continuity: 35,
                reliability: 88,
                token_efficiency: 70,
                no_memd_delta: Some(12),
            },
            &BenchmarkEvidenceSummary {
                has_contract_evidence: true,
                has_workflow_evidence: true,
                has_continuity_evidence: false,
                has_comparative_evidence: true,
                has_drift_failure: false,
            },
            true,
        );
        assert_eq!(scorecard.gate, "fragile");
    }

    #[test]
    fn no_memd_loss_caps_feature_at_acceptable() {
        let scorecard = resolve_benchmark_scorecard(
            &BenchmarkSubjectMetrics {
                correctness: 95,
                continuity: 90,
                reliability: 90,
                token_efficiency: 65,
                no_memd_delta: Some(-4),
            },
            &BenchmarkEvidenceSummary {
                has_contract_evidence: true,
                has_workflow_evidence: true,
                has_continuity_evidence: true,
                has_comparative_evidence: true,
                has_drift_failure: false,
            },
            true,
        );
        assert_eq!(scorecard.gate, "acceptable");
    }
```

- [ ] **Step 2: Run the continuity gate tests to verify they fail**

Run:

```bash
cargo test -p memd-client --bin memd continuity_failure_caps_gate_at_fragile -- --exact
cargo test -p memd-client --bin memd no_memd_loss_caps_feature_at_acceptable -- --exact
```

Expected: FAIL because the score resolution helper does not exist.

- [ ] **Step 3: Seed the registry with tier-0 journeys and loops**

Expand `docs/verification/benchmark-registry.json` with the continuity-critical features and journeys:

```json
{
  "journeys": [
    {
      "id": "journey.continuity.resume-handoff-attach",
      "name": "Resume to Handoff to Attach",
      "goal": "a resumed or attached session continues without manual reconstruction",
      "feature_ids": [
        "feature.bundle.wake",
        "feature.bundle.resume",
        "feature.bundle.handoff",
        "feature.bundle.attach",
        "feature.capture.checkpoint",
        "feature.capture.hook-capture"
      ],
      "loop_ids": [
        "loop.journey.resume-handoff-attach.correctness",
        "loop.journey.resume-handoff-attach.no-memd-delta",
        "loop.journey.resume-handoff-attach.drift"
      ],
      "quality_dimensions": [
        "continuity",
        "correctness",
        "token_efficiency",
        "drift_resistance"
      ],
      "baseline_mode_ids": [
        "baseline.no-memd",
        "baseline.with-memd"
      ],
      "drift_risks": ["continuity-drift", "surface-drift"],
      "gate_target": "acceptable"
    }
  ]
}
```

- [ ] **Step 4: Add evidence summaries and gate resolution helpers**

Add in `crates/memd-client/src/main.rs`:

```rust
struct BenchmarkSubjectMetrics {
    correctness: u8,
    continuity: u8,
    reliability: u8,
    token_efficiency: u8,
    no_memd_delta: Option<i16>,
}

struct BenchmarkEvidenceSummary {
    has_contract_evidence: bool,
    has_workflow_evidence: bool,
    has_continuity_evidence: bool,
    has_comparative_evidence: bool,
    has_drift_failure: bool,
}

fn resolve_benchmark_scorecard(
    metrics: &BenchmarkSubjectMetrics,
    evidence: &BenchmarkEvidenceSummary,
    continuity_critical: bool,
) -> DerivedBenchmarkScorecard {
    let mut gate = if metrics.correctness >= 90 && metrics.continuity >= 90 {
        "strong"
    } else if metrics.correctness >= 70 && metrics.continuity >= 70 {
        "acceptable"
    } else {
        "fragile"
    };

    if continuity_critical && !evidence.has_continuity_evidence {
        gate = "fragile";
    }
    if evidence.has_drift_failure {
        gate = "fragile";
    }
    if metrics.no_memd_delta.unwrap_or_default() < 0 {
        gate = "acceptable";
    }

    DerivedBenchmarkScorecard {
        gate: gate.to_string(),
    }
}
```

- [ ] **Step 5: Add journey evidence artifacts to the benchmark output**

Extend the benchmark artifact writer to emit evidence for the continuity journey under `.memd/telemetry/`:

```rust
fn write_continuity_journey_artifacts(
    output: &Path,
    report: &ContinuityJourneyReport,
) -> anyhow::Result<()> {
    let dir = output.join("telemetry").join("continuity");
    fs::create_dir_all(&dir)?;
    fs::write(
        dir.join("latest.json"),
        serde_json::to_string_pretty(report)? + "\n",
    )?;
    Ok(())
}
```

- [ ] **Step 6: Run the continuity gate tests to verify they pass**

Run:

```bash
cargo test -p memd-client --bin memd continuity_failure_caps_gate_at_fragile -- --exact
cargo test -p memd-client --bin memd no_memd_loss_caps_feature_at_acceptable -- --exact
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add docs/verification/benchmark-registry.json crates/memd-schema/src/lib.rs crates/memd-client/src/main.rs
git commit -m "feat: add continuity-critical score resolution and journeys"
```

## Task 4: Add `no memd` vs `with memd` comparative benchmarking and token deltas

**Files:**
- Modify: `docs/verification/benchmark-registry.json`
- Modify: `crates/memd-client/src/main.rs`
- Test: `crates/memd-client/src/main.rs`

**Goal:** Make the benchmark system prove product value by comparing continuity and token cost against a no-memd baseline.

- [ ] **Step 1: Write the failing comparative benchmark test**

Add in `crates/memd-client/src/main.rs`:

```rust
    #[test]
    fn build_no_memd_delta_report_surfaces_token_and_reconstruction_improvement() {
        let report = build_no_memd_delta_report(
            &BaselineMetrics {
                prompt_tokens: 2200,
                reread_count: 5,
                reconstruction_steps: 4,
            },
            &BaselineMetrics {
                prompt_tokens: 1200,
                reread_count: 2,
                reconstruction_steps: 1,
            },
        );

        assert_eq!(report.token_delta, 1000);
        assert_eq!(report.reread_delta, 3);
        assert_eq!(report.reconstruction_delta, 3);
        assert!(report.with_memd_better);
    }
```

- [ ] **Step 2: Run the comparative test to verify it fails**

Run:

```bash
cargo test -p memd-client --bin memd build_no_memd_delta_report_surfaces_token_and_reconstruction_improvement -- --exact
```

Expected: FAIL because the delta report helper does not exist.

- [ ] **Step 3: Add baseline metric structs and comparative helper**

Add in `crates/memd-client/src/main.rs`:

```rust
struct BaselineMetrics {
    prompt_tokens: usize,
    reread_count: usize,
    reconstruction_steps: usize,
}

struct NoMemdDeltaReport {
    token_delta: isize,
    reread_delta: isize,
    reconstruction_delta: isize,
    with_memd_better: bool,
}

fn build_no_memd_delta_report(
    no_memd: &BaselineMetrics,
    with_memd: &BaselineMetrics,
) -> NoMemdDeltaReport {
    let token_delta = no_memd.prompt_tokens as isize - with_memd.prompt_tokens as isize;
    let reread_delta = no_memd.reread_count as isize - with_memd.reread_count as isize;
    let reconstruction_delta =
        no_memd.reconstruction_steps as isize - with_memd.reconstruction_steps as isize;
    NoMemdDeltaReport {
        token_delta,
        reread_delta,
        reconstruction_delta,
        with_memd_better: token_delta > 0 && reread_delta > 0 && reconstruction_delta > 0,
    }
}
```

- [ ] **Step 4: Add comparative evidence to benchmark output**

Update the benchmark command to include comparative evidence strings:

```rust
evidence.push(format!(
    "no_memd_delta tokens={} rereads={} reconstruction={}",
    delta.token_delta,
    delta.reread_delta,
    delta.reconstruction_delta
));
```

Add a generated markdown section in `docs/verification/SCORES.md`:

```md
## No memd vs With memd

- token delta: `+1000`
- reread delta: `+3`
- reconstruction delta: `+3`
- with memd better: `true`
```

- [ ] **Step 5: Run the comparative test to verify it passes**

Run:

```bash
cargo test -p memd-client --bin memd build_no_memd_delta_report_surfaces_token_and_reconstruction_improvement -- --exact
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add docs/verification/benchmark-registry.json crates/memd-client/src/main.rs docs/verification/SCORES.md
git commit -m "feat: add no-memd comparative benchmark deltas"
```

## Task 5: Add benchmark cost budgets and the morning operator summary

**Files:**
- Modify: `docs/verification/benchmark-registry.json`
- Modify: `crates/memd-client/src/main.rs`
- Test: `crates/memd-client/src/main.rs`

**Goal:** Make the benchmark system practical for one maintainer by tracking benchmark cost and generating a morning summary.

- [ ] **Step 1: Write the failing morning summary test**

Add in `crates/memd-client/src/main.rs`:

```rust
    #[test]
    fn render_morning_operator_summary_surfaces_top_regressions() {
        let summary = render_morning_operator_summary(&MorningOperatorSummary {
            top_continuity_failures: vec!["resume continuity drift".to_string()],
            top_drift_risks: vec!["surface drift in MEMD_MEMORY.md".to_string()],
            top_token_regressions: vec!["handoff packet +420 tokens".to_string()],
            top_no_memd_losses: vec!["resume still loses to no-memd baseline".to_string()],
            proposed_next_actions: vec!["fix resume journey before expanding registry".to_string()],
        });

        assert!(summary.contains("resume continuity drift"));
        assert!(summary.contains("handoff packet +420 tokens"));
        assert!(summary.contains("fix resume journey before expanding registry"));
    }
```

- [ ] **Step 2: Run the morning summary test to verify it fails**

Run:

```bash
cargo test -p memd-client --bin memd render_morning_operator_summary_surfaces_top_regressions -- --exact
```

Expected: FAIL because the morning summary type and renderer do not exist.

- [ ] **Step 3: Add runtime policies for benchmark cost budgets**

Extend `docs/verification/benchmark-registry.json` with:

```json
{
  "runtime_policies": [
    {
      "id": "policy.benchmark.always_on.max_latency_ms",
      "name": "Always-on latency budget",
      "cli_surface": "memd benchmark --policy always-on-latency",
      "default_value": 500,
      "allowed_range": "50-5000",
      "quality_dimensions_affected": ["token_efficiency", "continuity"],
      "risk_level": "medium",
      "loop_ids": ["loop.journey.resume-handoff-attach.correctness"]
    }
  ]
}
```

- [ ] **Step 4: Add the morning summary renderer and writer**

Add in `crates/memd-client/src/main.rs`:

```rust
struct MorningOperatorSummary {
    top_continuity_failures: Vec<String>,
    top_drift_risks: Vec<String>,
    top_token_regressions: Vec<String>,
    top_no_memd_losses: Vec<String>,
    proposed_next_actions: Vec<String>,
}

fn render_morning_operator_summary(summary: &MorningOperatorSummary) -> String {
    format!(
        "# Morning Summary\n\n## Continuity\n- {}\n\n## Drift\n- {}\n\n## Tokens\n- {}\n\n## With memd < No memd\n- {}\n\n## Next Actions\n- {}\n",
        summary.top_continuity_failures.join("\n- "),
        summary.top_drift_risks.join("\n- "),
        summary.top_token_regressions.join("\n- "),
        summary.top_no_memd_losses.join("\n- "),
        summary.proposed_next_actions.join("\n- ")
    )
}
```

Write the rendered summary into:

```rust
fs::write(
    repo_root.join("docs/verification/MORNING.md"),
    render_morning_operator_summary(&summary),
)?;
```

- [ ] **Step 5: Run the morning summary test to verify it passes**

Run:

```bash
cargo test -p memd-client --bin memd render_morning_operator_summary_surfaces_top_regressions -- --exact
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add docs/verification/benchmark-registry.json crates/memd-client/src/main.rs docs/verification/MORNING.md
git commit -m "feat: add benchmark cost budgets and morning summary"
```

## Task 6: Connect benchmark gaps to autoresearch and telemetry

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Modify: `docs/verification/LOOPS.md`
- Modify: `docs/verification/COVERAGE.md`
- Test: `crates/memd-client/src/main.rs`

**Goal:** Let overnight systems discover missing coverage and rank next actions from the benchmark registry instead of only from ad hoc heuristics.

- [ ] **Step 1: Write the failing gap discovery test**

Add in `crates/memd-client/src/main.rs`:

```rust
    #[test]
    fn build_benchmark_gap_candidates_surfaces_unbenchmarked_continuity_feature() {
        let registry = BenchmarkRegistry {
            version: "v1".to_string(),
            app_goal: "demo".to_string(),
            quality_dimensions: Vec::new(),
            tiers: Vec::new(),
            pillars: Vec::new(),
            families: Vec::new(),
            features: vec![BenchmarkFeatureRecord {
                id: "feature.bundle.resume".to_string(),
                name: "Resume".to_string(),
                pillar: "memory-continuity".to_string(),
                family: "bundle-runtime".to_string(),
                tier: "tier-0-continuity-critical".to_string(),
                continuity_critical: true,
                user_contract: "resume restores continuity".to_string(),
                source_contract_refs: Vec::new(),
                commands: vec!["memd resume".to_string()],
                routes: Vec::new(),
                files: vec!["crates/memd-client/src/main.rs".to_string()],
                journey_ids: Vec::new(),
                loop_ids: Vec::new(),
                quality_dimensions: vec!["continuity".to_string()],
                drift_risks: vec!["continuity-drift".to_string()],
                failure_modes: vec!["resume misses task state".to_string()],
                coverage_status: "unbenchmarked".to_string(),
                last_verified_at: None,
            }],
            journeys: Vec::new(),
            loops: Vec::new(),
            scorecards: Vec::new(),
            evidence: Vec::new(),
            gates: Vec::new(),
            baseline_modes: Vec::new(),
            runtime_policies: Vec::new(),
            generated_at: None,
        };

        let gaps = build_benchmark_gap_candidates(&registry);
        assert!(gaps.iter().any(|gap| gap.id == "benchmark:unbenchmarked_continuity_feature"));
    }
```

- [ ] **Step 2: Run the gap discovery test to verify it fails**

Run:

```bash
cargo test -p memd-client --bin memd build_benchmark_gap_candidates_surfaces_unbenchmarked_continuity_feature -- --exact
```

Expected: FAIL because the benchmark gap candidate helper does not exist.

- [ ] **Step 3: Add registry-driven gap discovery**

Add in `crates/memd-client/src/main.rs`:

```rust
fn build_benchmark_gap_candidates(registry: &BenchmarkRegistry) -> Vec<GapCandidate> {
    let mut gaps = Vec::new();
    if registry.features.iter().any(|feature| {
        feature.continuity_critical && feature.coverage_status == "unbenchmarked"
    }) {
        gaps.push(GapCandidate {
            id: "benchmark:unbenchmarked_continuity_feature".to_string(),
            area: "benchmark".to_string(),
            severity: "high".to_string(),
            summary: "continuity-critical feature has no benchmark coverage".to_string(),
        });
    }
    gaps
}
```

Call it from `run_autoresearch` before loop selection and append its output to the existing gap signal set.

- [ ] **Step 4: Surface registry coverage gaps in telemetry and docs**

Update generated docs so:

- `docs/verification/COVERAGE.md` lists unbenchmarked continuity-critical features first
- `docs/verification/LOOPS.md` lists missing loop IDs per feature
- `memd telemetry --json` includes a benchmark coverage section

Add this telemetry fragment:

```json
{
  "benchmark": {
    "continuity_critical_total": 8,
    "continuity_critical_benchmarked": 5,
    "missing_loop_count": 3,
    "with_memd_losses": 1
  }
}
```

- [ ] **Step 5: Run the gap discovery test to verify it passes**

Run:

```bash
cargo test -p memd-client --bin memd build_benchmark_gap_candidates_surfaces_unbenchmarked_continuity_feature -- --exact
```

Expected: PASS.

- [ ] **Step 6: Run the relevant suites**

Run:

```bash
cargo test -p memd-schema
cargo test -p memd-client --bin memd
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/memd-client/src/main.rs docs/verification/LOOPS.md docs/verification/COVERAGE.md
git commit -m "feat: connect benchmark coverage gaps to autoresearch"
```

## Spec Coverage Check

- Canonical registry + schema: covered in Task 1
- Pillars/families/features/journeys/loops structure: covered in Tasks 1-3
- Evidence and scorecards: covered in Tasks 2-4
- Hard score resolution rules: covered in Task 3
- `no memd` vs `with memd`: covered in Task 4
- Benchmark cost budgets: covered in Task 5
- Morning operator summary: covered in Task 5
- Overnight gap discovery and suggestions: covered in Task 6
- Minimal executable slice first: covered by Tasks 1-4 before broader expansion

## Notes

- This plan intentionally implements the minimal executable slice first. It does **not** try to operationalize the entire full-catalog benchmark matrix in one pass.
- `autodream`/`self-evolution` integration is limited to proposal and telemetry surfaces in this phase. Auto-promotion of registry truth stays gated.
- If the existing `memd benchmark` output format proves too rigid, add small adapters rather than replacing the command wholesale.
