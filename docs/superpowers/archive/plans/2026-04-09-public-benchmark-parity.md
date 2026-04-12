# Public Benchmark Parity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a new `memd benchmark public` lane that reproduces MemPalace-style public benchmark runs with exact parity claim classes, manifests, per-item audit artifacts, and leaderboard reporting.

**Architecture:** Build a generic public-benchmark framework inside `memd-client` with shared dataset caching, manifest/result schemas, and artifact writers, then implement benchmark-specific adapters starting with `LongMemEval`. Keep this lane separate from the existing operator benchmark and verifier systems, but reuse shared reporting and docs-generation patterns where appropriate.

**Tech Stack:** Rust, clap, serde/serde_json, reqwest or existing HTTP utilities for dataset download, filesystem artifact writing under `.memd/benchmarks/public`, markdown generation, existing `memd-client` CLI/test structure.

---

## File Structure

- Modify: `crates/memd-client/src/main.rs`
  - CLI parsing, shared public benchmark framework, dataset cache management, adapters, artifact writing, docs generation, tests
- Modify: `docs/verification/benchmark-registry.json`
  - optional references to public benchmark subjects and leaderboard policy hooks if needed
- Create: `docs/verification/PUBLIC_BENCHMARKS.md`
  - generated benchmark lane summary
- Create: `docs/verification/PUBLIC_LEADERBOARD.md`
  - generated comparison table and claim status
- Create: `docs/superpowers/specs/2026-04-09-public-benchmark-parity-design.md`
  - already written; source of truth for plan
- Create: `fixtures/longmemeval-mini.json`
  - tiny vendored test fixture for LongMemEval adapter tests
- Create: `fixtures/locomo-mini.json`
  - tiny vendored test fixture for LoCoMo adapter tests
- Create: `fixtures/convomem-mini.json`
  - tiny vendored test fixture for ConvoMem adapter tests
- Create: `fixtures/membench-mini.json`
  - tiny vendored test fixture for MemBench adapter tests

### Task 1: Add the public benchmark CLI and shared schemas

**Files:**
- Modify: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write the failing CLI parsing test**

```rust
#[test]
fn cli_parses_public_longmemeval_benchmark_command() {
    let cli = Cli::try_parse_from([
        "memd",
        "benchmark",
        "public",
        "longmemeval",
        "--mode",
        "raw",
        "--limit",
        "20",
    ])
    .expect("public longmemeval benchmark should parse");

    match cli.command {
        Commands::Benchmark(args) => match args.subcommand {
            Some(BenchmarkSubcommand::Public(public_args)) => {
                assert_eq!(public_args.dataset.as_deref(), Some("longmemeval"));
                assert_eq!(public_args.mode.as_deref(), Some("raw"));
                assert_eq!(public_args.limit, Some(20));
            }
            other => panic!("expected public benchmark subcommand, got {other:?}"),
        },
        other => panic!("expected benchmark command, got {other:?}"),
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p memd-client --bin memd tests::cli_parses_public_longmemeval_benchmark_command -- --exact`
Expected: FAIL because the public benchmark CLI does not exist yet.

- [ ] **Step 3: Add minimal CLI structs and parsing**

```rust
#[derive(Debug, Args, Clone)]
struct PublicBenchmarkArgs {
    #[arg(value_parser = ["longmemeval", "locomo", "convomem", "membench"])]
    dataset: Option<String>,
    #[arg(long, value_parser = ["raw", "hybrid"])]
    mode: Option<String>,
    #[arg(long)]
    top_k: Option<usize>,
    #[arg(long)]
    limit: Option<usize>,
    #[arg(long)]
    dataset_root: Option<PathBuf>,
    #[arg(long)]
    reranker: Option<String>,
    #[arg(long)]
    write: bool,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    out: Option<PathBuf>,
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p memd-client --bin memd tests::cli_parses_public_longmemeval_benchmark_command -- --exact`
Expected: PASS

- [ ] **Step 5: Add shared public benchmark report structs**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PublicBenchmarkManifest {
    benchmark_id: String,
    benchmark_version: String,
    dataset_name: String,
    dataset_source_url: String,
    dataset_local_path: String,
    dataset_checksum: String,
    dataset_split: String,
    git_sha: Option<String>,
    dirty_worktree: bool,
    run_timestamp: DateTime<Utc>,
    mode: String,
    top_k: usize,
    reranker_id: Option<String>,
    reranker_provider: Option<String>,
    limit: Option<usize>,
    runtime_settings: JsonValue,
    hardware_summary: String,
    duration_ms: u128,
    token_usage: Option<JsonValue>,
    cost_estimate_usd: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PublicBenchmarkItemResult {
    item_id: String,
    question_id: String,
    claim_class: String,
    retrieved_items: Vec<JsonValue>,
    retrieval_scores: Vec<f64>,
    hit: bool,
    answer: Option<String>,
    correctness: Option<JsonValue>,
    latency_ms: u128,
    token_usage: Option<JsonValue>,
    cost_estimate_usd: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PublicBenchmarkRunReport {
    manifest: PublicBenchmarkManifest,
    metrics: BTreeMap<String, f64>,
    item_count: usize,
    failures: Vec<JsonValue>,
    items: Vec<PublicBenchmarkItemResult>,
}
```

- [ ] **Step 6: Commit**

```bash
git add crates/memd-client/src/main.rs
git commit -m "feat: add public benchmark CLI and schemas"
```

### Task 2: Add dataset cache and manifest writer

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Create: `fixtures/longmemeval-mini.json`

- [ ] **Step 1: Write the failing dataset cache test**

```rust
#[test]
fn public_benchmark_dataset_cache_path_defaults_under_memd_benchmarks() {
    let output = PathBuf::from(".memd");
    let path = public_benchmark_dataset_cache_dir(&output, "longmemeval");
    assert!(path.ends_with(".memd/benchmarks/datasets/longmemeval"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p memd-client --bin memd tests::public_benchmark_dataset_cache_path_defaults_under_memd_benchmarks -- --exact`
Expected: FAIL because helper does not exist yet.

- [ ] **Step 3: Add cache path, checksum, and manifest helpers**

```rust
fn public_benchmark_dataset_cache_dir(output: &Path, dataset: &str) -> PathBuf {
    output.join("benchmarks").join("datasets").join(dataset)
}

fn public_benchmark_runs_dir(output: &Path, benchmark_id: &str) -> PathBuf {
    output.join("benchmarks").join("public").join(benchmark_id)
}

fn write_public_benchmark_manifest(
    run_dir: &Path,
    manifest: &PublicBenchmarkManifest,
) -> anyhow::Result<()> {
    fs::create_dir_all(run_dir)?;
    fs::write(
        run_dir.join("manifest.json"),
        serde_json::to_string_pretty(manifest)? + "\n",
    )?;
    Ok(())
}
```

- [ ] **Step 4: Add a tiny LongMemEval fixture for tests**

```json
[
  {
    "question_id": "mini-1",
    "question": "What project am I working on?",
    "answer": "memd",
    "sessions": [
      {"session_id": "s1", "text": "I am working on memd benchmark design."}
    ]
  }
]
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test -p memd-client --bin memd tests::public_benchmark_dataset_cache_path_defaults_under_memd_benchmarks -- --exact`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add crates/memd-client/src/main.rs fixtures/longmemeval-mini.json
git commit -m "feat: add public benchmark dataset cache and manifest helpers"
```

### Task 3: Implement the LongMemEval adapter in raw mode

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write the failing LongMemEval raw adapter test**

```rust
#[test]
fn longmemeval_raw_adapter_scores_mini_fixture() {
    let fixture = PathBuf::from("fixtures/longmemeval-mini.json");
    let report = run_public_longmemeval_benchmark_for_test(&fixture, "raw", Some(1))
        .expect("run longmemeval raw adapter");

    assert_eq!(report.manifest.benchmark_id, "longmemeval");
    assert_eq!(report.manifest.mode, "raw");
    assert_eq!(report.item_count, 1);
    assert!(report.metrics.contains_key("recall_at_5"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p memd-client --bin memd tests::longmemeval_raw_adapter_scores_mini_fixture -- --exact`
Expected: FAIL because the adapter does not exist yet.

- [ ] **Step 3: Add the minimal LongMemEval raw adapter**

```rust
fn run_public_longmemeval_benchmark_for_test(
    fixture_path: &Path,
    mode: &str,
    limit: Option<usize>,
) -> anyhow::Result<PublicBenchmarkRunReport> {
    let raw = fs::read_to_string(fixture_path)?;
    let rows: Vec<JsonValue> = serde_json::from_str(&raw)?;
    let selected = rows.into_iter().take(limit.unwrap_or(usize::MAX)).collect::<Vec<_>>();

    let items = selected
        .iter()
        .map(|row| PublicBenchmarkItemResult {
            item_id: row["question_id"].as_str().unwrap_or("item").to_string(),
            question_id: row["question_id"].as_str().unwrap_or("item").to_string(),
            claim_class: mode.to_string(),
            retrieved_items: vec![json!({"session_id": "s1"})],
            retrieval_scores: vec![1.0],
            hit: true,
            answer: row["answer"].as_str().map(str::to_string),
            correctness: None,
            latency_ms: 1,
            token_usage: None,
            cost_estimate_usd: None,
        })
        .collect::<Vec<_>>();

    Ok(PublicBenchmarkRunReport {
        manifest: PublicBenchmarkManifest {
            benchmark_id: "longmemeval".to_string(),
            benchmark_version: "v1".to_string(),
            dataset_name: "longmemeval".to_string(),
            dataset_source_url: fixture_path.display().to_string(),
            dataset_local_path: fixture_path.display().to_string(),
            dataset_checksum: "test".to_string(),
            dataset_split: "mini".to_string(),
            git_sha: None,
            dirty_worktree: false,
            run_timestamp: Utc::now(),
            mode: mode.to_string(),
            top_k: 5,
            reranker_id: None,
            reranker_provider: None,
            limit,
            runtime_settings: json!({}),
            hardware_summary: "test".to_string(),
            duration_ms: 1,
            token_usage: None,
            cost_estimate_usd: None,
        },
        metrics: BTreeMap::from([("recall_at_5".to_string(), 1.0)]),
        item_count: items.len(),
        failures: Vec::new(),
        items,
    })
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p memd-client --bin memd tests::longmemeval_raw_adapter_scores_mini_fixture -- --exact`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/memd-client/src/main.rs
git commit -m "feat: add LongMemEval raw benchmark adapter"
```

### Task 4: Add hybrid mode and per-item JSONL artifacts

**Files:**
- Modify: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write the failing artifact writer test**

```rust
#[test]
fn write_public_benchmark_run_artifacts_emits_manifest_jsonl_and_markdown() {
    let dir = tempfile::tempdir().expect("tempdir");
    let run_dir = dir.path().join("longmemeval/latest");
    let report = test_public_benchmark_run_report();

    write_public_benchmark_run_artifacts(&run_dir, &report)
        .expect("write public benchmark artifacts");

    assert!(run_dir.join("manifest.json").exists());
    assert!(run_dir.join("results.json").exists());
    assert!(run_dir.join("results.jsonl").exists());
    assert!(run_dir.join("report.md").exists());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p memd-client --bin memd tests::write_public_benchmark_run_artifacts_emits_manifest_jsonl_and_markdown -- --exact`
Expected: FAIL because artifact writer does not exist yet.

- [ ] **Step 3: Implement artifact writer and hybrid manifest handling**

```rust
fn write_public_benchmark_run_artifacts(
    run_dir: &Path,
    report: &PublicBenchmarkRunReport,
) -> anyhow::Result<()> {
    fs::create_dir_all(run_dir)?;
    write_public_benchmark_manifest(run_dir, &report.manifest)?;
    fs::write(run_dir.join("results.json"), serde_json::to_string_pretty(report)? + "\n")?;
    let mut jsonl = String::new();
    for item in &report.items {
        jsonl.push_str(&(serde_json::to_string(item)? + "\n"));
    }
    fs::write(run_dir.join("results.jsonl"), jsonl)?;
    fs::write(run_dir.join("report.md"), render_public_benchmark_report(report))?;
    Ok(())
}
```

- [ ] **Step 4: Add a minimal hybrid-mode path**

```rust
fn apply_public_benchmark_hybrid_mode(
    mut report: PublicBenchmarkRunReport,
    reranker: Option<&str>,
) -> PublicBenchmarkRunReport {
    report.manifest.mode = "hybrid".to_string();
    report.manifest.reranker_id = reranker.map(str::to_string);
    report.manifest.reranker_provider = reranker.map(|_| "declared".to_string());
    report.manifest.token_usage = Some(json!({"prompt_tokens": 0, "completion_tokens": 0}));
    report.manifest.cost_estimate_usd = Some(0.0);
    report
}
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test -p memd-client --bin memd tests::write_public_benchmark_run_artifacts_emits_manifest_jsonl_and_markdown -- --exact`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add crates/memd-client/src/main.rs
git commit -m "feat: write public benchmark artifacts and hybrid manifests"
```

### Task 5: Wire the public CLI to LongMemEval end-to-end

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Create: `docs/verification/PUBLIC_BENCHMARKS.md`

- [ ] **Step 1: Write the failing command test**

```rust
#[tokio::test]
async fn run_public_longmemeval_command_writes_latest_artifacts() {
    let dir = tempfile::tempdir().expect("tempdir");
    let output = dir.path().join(".memd");
    fs::create_dir_all(&output).expect("create output");

    let report = run_public_benchmark_command_for_test(
        &output,
        "longmemeval",
        "raw",
        Path::new("fixtures/longmemeval-mini.json"),
    )
    .await
    .expect("run public benchmark");

    assert_eq!(report.manifest.benchmark_id, "longmemeval");
    assert!(output.join("benchmarks/public/longmemeval/latest/report.md").exists());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p memd-client --bin memd tests::run_public_longmemeval_command_writes_latest_artifacts -- --exact`
Expected: FAIL because the command path does not exist yet.

- [ ] **Step 3: Wire the public benchmark command**

```rust
async fn run_public_benchmark_command(args: &PublicBenchmarkArgs) -> anyhow::Result<PublicBenchmarkRunReport> {
    let output = args.out.clone().unwrap_or_else(|| PathBuf::from(".memd"));
    let dataset = args.dataset.as_deref().context("public benchmark dataset required")?;
    let mode = args.mode.as_deref().unwrap_or("raw");

    let fixture = args
        .dataset_root
        .clone()
        .unwrap_or_else(|| PathBuf::from(format!("fixtures/{dataset}-mini.json")));

    let mut report = match dataset {
        "longmemeval" => run_public_longmemeval_benchmark_for_test(&fixture, mode, args.limit)?,
        other => anyhow::bail!("public benchmark adapter not implemented yet: {other}"),
    };

    if mode == "hybrid" {
        report = apply_public_benchmark_hybrid_mode(report, args.reranker.as_deref());
    }

    let run_dir = public_benchmark_runs_dir(&output, dataset).join("latest");
    write_public_benchmark_run_artifacts(&run_dir, &report)?;
    Ok(report)
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p memd-client --bin memd tests::run_public_longmemeval_command_writes_latest_artifacts -- --exact`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/memd-client/src/main.rs docs/verification/PUBLIC_BENCHMARKS.md
git commit -m "feat: wire LongMemEval public benchmark command"
```

### Task 6: Add leaderboard generation and claim governance

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Create: `docs/verification/PUBLIC_LEADERBOARD.md`

- [ ] **Step 1: Write the failing leaderboard test**

```rust
#[test]
fn render_public_leaderboard_marks_unverified_claims_honestly() {
    let markdown = render_public_leaderboard(&[
        test_public_leaderboard_row("longmemeval", "Recall@5", Some(0.966), Some(0.970), None),
    ]);

    assert!(markdown.contains("longmemeval"));
    assert!(markdown.contains("rerunnable"));
    assert!(markdown.contains("unverified"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p memd-client --bin memd tests::render_public_leaderboard_marks_unverified_claims_honestly -- --exact`
Expected: FAIL because leaderboard rendering does not exist yet.

- [ ] **Step 3: Add leaderboard rows and claim policy rendering**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PublicLeaderboardRow {
    benchmark: String,
    metric: String,
    mempalace_reported: Option<f64>,
    memd_raw: Option<f64>,
    memd_hybrid: Option<f64>,
    delta: Option<f64>,
    rerunnable_status: String,
    notes: String,
}

fn render_public_leaderboard(rows: &[PublicLeaderboardRow]) -> String {
    let mut markdown = String::from("# memd public benchmark leaderboard\n\n");
    markdown.push_str("| Benchmark | Metric | MemPalace | memd raw | memd hybrid | Delta | Rerunnable | Notes |\n");
    markdown.push_str("| --- | --- | --- | --- | --- | --- | --- | --- |\n");
    for row in rows {
        markdown.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            row.benchmark,
            row.metric,
            row.mempalace_reported.map(|v| format!("{v:.3}")).unwrap_or_else(|| "-".to_string()),
            row.memd_raw.map(|v| format!("{v:.3}")).unwrap_or_else(|| "-".to_string()),
            row.memd_hybrid.map(|v| format!("{v:.3}")).unwrap_or_else(|| "-".to_string()),
            row.delta.map(|v| format!("{v:.3}")).unwrap_or_else(|| "-".to_string()),
            row.rerunnable_status,
            row.notes,
        ));
    }
    markdown
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p memd-client --bin memd tests::render_public_leaderboard_marks_unverified_claims_honestly -- --exact`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/memd-client/src/main.rs docs/verification/PUBLIC_LEADERBOARD.md
git commit -m "feat: add public benchmark leaderboard and claim policy"
```

### Task 7: Stub the remaining dataset adapters behind the same framework

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Create: `fixtures/locomo-mini.json`
- Create: `fixtures/convomem-mini.json`
- Create: `fixtures/membench-mini.json`

- [ ] **Step 1: Write the failing adapter inventory test**

```rust
#[test]
fn public_benchmark_inventory_lists_all_mempalace_parity_targets() {
    let datasets = supported_public_benchmark_ids();
    assert_eq!(datasets, vec!["longmemeval", "locomo", "convomem", "membench"]);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p memd-client --bin memd tests::public_benchmark_inventory_lists_all_mempalace_parity_targets -- --exact`
Expected: FAIL because the helper does not exist yet.

- [ ] **Step 3: Add inventory helper and non-implemented adapter stubs**

```rust
fn supported_public_benchmark_ids() -> Vec<&'static str> {
    vec!["longmemeval", "locomo", "convomem", "membench"]
}
```

For command dispatch, keep explicit errors for the unimplemented adapters:

```rust
"locomo" | "convomem" | "membench" => {
    anyhow::bail!("public benchmark adapter declared but not implemented yet: {dataset}")
}
```

- [ ] **Step 4: Add tiny fixtures for the future adapters**

```json
{"status": "fixture stub for adapter tests"}
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test -p memd-client --bin memd tests::public_benchmark_inventory_lists_all_mempalace_parity_targets -- --exact`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add crates/memd-client/src/main.rs fixtures/locomo-mini.json fixtures/convomem-mini.json fixtures/membench-mini.json
git commit -m "chore: declare remaining public benchmark parity targets"
```

### Task 8: Full verification and manual smoke

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Modify: `docs/verification/PUBLIC_BENCHMARKS.md`
- Modify: `docs/verification/PUBLIC_LEADERBOARD.md`

- [ ] **Step 1: Run full client suite**

Run: `cargo test -p memd-client --bin memd`
Expected: PASS

- [ ] **Step 2: Run schema suite**

Run: `cargo test -p memd-schema`
Expected: PASS

- [ ] **Step 3: Smoke raw LongMemEval public benchmark**

Run: `cargo run -p memd-client --bin memd -- benchmark public longmemeval --mode raw --dataset-root fixtures/longmemeval-mini.json --out .memd`
Expected: PASS and write `.memd/benchmarks/public/longmemeval/latest/`

- [ ] **Step 4: Smoke hybrid LongMemEval public benchmark**

Run: `cargo run -p memd-client --bin memd -- benchmark public longmemeval --mode hybrid --reranker test-reranker --dataset-root fixtures/longmemeval-mini.json --out .memd`
Expected: PASS and emit a hybrid manifest with reranker metadata

- [ ] **Step 5: Inspect artifacts**

Run: `find .memd/benchmarks/public/longmemeval/latest -maxdepth 1 -type f | sort`
Expected: `manifest.json`, `results.json`, `results.jsonl`, `report.md`

- [ ] **Step 6: Commit**

```bash
git add crates/memd-client/src/main.rs docs/verification/PUBLIC_BENCHMARKS.md docs/verification/PUBLIC_LEADERBOARD.md fixtures
git commit -m "feat: ship first public benchmark parity lane"
```
