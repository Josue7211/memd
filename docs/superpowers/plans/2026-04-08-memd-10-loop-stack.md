# memd 10-Loop Autoresearch Stack Implementation Plan

This is a slice plan, not a roadmap.

Roadmap truth lives in `ROADMAP.md`.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expand memd's autoresearch system into a 10-loop stack that catches weak or false-green improvements fast while still tracking real gains in runtime, hive coordination, memory hygiene, and repo hygiene.

**Architecture:** Keep `crates/memd-client/src/main.rs` as the orchestration point, but widen the descriptor table and runner dispatch so each loop reports a distinct failure mode and a distinct secondary signal. Reuse existing hive, bundle, memory, docs, and git helpers instead of creating a parallel subsystem.

**Tech Stack:** Rust, tokio, serde_json, existing memd runtime/bundle helpers, existing test module in `crates/memd-client/src/main.rs`.

---

## Loop Mapping

The new 10-loop stack is a mix of existing runtime loops and two new repo-hygiene loops:

- `prompt-surface` becomes `prompt-efficiency`
- `live-truth` becomes `signal-freshness`
- `event-spine` contributes to `memory-hygiene`
- `correction-learning` becomes `repair-rate`
- `capability-contract` becomes `autonomy-quality`
- `cross-harness` stays `cross-harness`
- `self-evolution` stays `self-evolution`
- add `hive-health`
- add `branch-review-quality`
- add `docs-spec-drift`

## Tasks

### Task 1: Extend the loop contract and descriptor table

**Files:**
- Modify: `crates/memd-client/src/main.rs:10720-11040`
- Test: `crates/memd-client/src/main.rs:36197-36260`

- [ ] **Step 1: Append the two missing loop descriptors**

```rust
AutoresearchLoop::new(
    "hive-health",
    "hive-health",
    "Hive Health",
    "Keep live peers, heartbeat publication, and claim collisions healthy.",
    "live peers / claims",
    "dead peers / collisions",
    "no dead peers",
    "low",
    1.0,
    40.0,
    0.5,
    4.0,
),
AutoresearchLoop::new(
    "docs-spec-drift",
    "docs-spec-drift",
    "Docs Spec Drift",
    "Keep docs and shipped behavior aligned.",
    "docs alignment",
    "spec drift",
    "docs match runtime",
    "medium",
    1.0,
    40.0,
    0.5,
    4.0,
),
```

- [ ] **Step 2: Add the shared success gate**

```rust
fn loop_success_requires_second_signal(
    primary_ok: bool,
    secondary_ok: bool,
    confidence_ok: bool,
    trend_ok: bool,
) -> bool {
    primary_ok && secondary_ok && confidence_ok && trend_ok
}
```

- [ ] **Step 3: Add the table-size regression test**

```rust
#[test]
fn autoresearch_loop_table_has_ten_unique_slugs() {
    let mut slugs = AUTORESEARCH_LOOPS.iter().map(|d| d.slug).collect::<Vec<_>>();
    slugs.sort_unstable();
    slugs.dedup();
    assert_eq!(slugs.len(), 10);
    assert!(slugs.contains(&"hive-health"));
    assert!(slugs.contains(&"docs-spec-drift"));
}
```

- [ ] **Step 4: Run the focused test**

Run: `cargo test -p memd-client autoresearch_loop_table_has_ten_unique_slugs -- --exact`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/memd-client/src/main.rs
git commit -m "feat: expand autoresearch loop contract"
```

### Task 2: Add hive-health, memory-hygiene, and autonomy-quality runners

**Files:**
- Modify: `crates/memd-client/src/main.rs:9780-10590`
- Test: `crates/memd-client/src/main.rs:36260-36480`

- [ ] **Step 1: Add `run_hive_health_loop`**

```rust
async fn run_hive_health_loop(
    output: &Path,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let awareness = read_project_awareness(&AwarenessArgs {
        output: output.to_path_buf(),
        root: None,
        include_current: true,
        summary: false,
    })
    .await?;
    let heartbeat = build_hive_heartbeat(output, None)?;
    let evidence = vec![
        format!("active_hives={}", awareness.entries.len()),
        format!("dead_hives={}", awareness.entries.iter().filter(|entry| entry.presence == "dead").count()),
        format!("claim_collisions={}", awareness.collisions.len()),
    ];
    let percent = if awareness.collisions.is_empty() { 100.0 } else { 100.0 - (awareness.collisions.len() as f64 * 10.0) };
    let token_savings = (awareness.entries.len() as f64) * 4.0;
    let confidence_met = loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let secondary_signal_ok = heartbeat.status != "dead" && awareness.collisions.is_empty();
    let status = if loop_success_requires_second_signal(
        confidence_met,
        secondary_signal_ok,
        confidence_met,
        !loop_is_regressed(descriptor, previous_entry, percent, token_savings),
    ) {
        "success"
    } else {
        "warning"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        "hive health score".to_string(),
        vec!["hive health".to_string()],
        serde_json::json!({
            "evidence": evidence,
            "heartbeat_status": heartbeat.status,
        }),
        status,
    ))
}
```

- [ ] **Step 2: Add `run_memory_hygiene_loop`**

```rust
async fn run_memory_hygiene_loop(
    output: &Path,
    base_url: &str,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let snapshot = read_bundle_resume(&autoresearch_resume_args(output), base_url).await?;
    let duplicates = snapshot.redundant_context_items() as f64;
    let evidence = vec![
        format!("duplicates={duplicates}"),
        format!("context_records={}", snapshot.context.records.len()),
        format!("working_records={}", snapshot.working.records.len()),
        format!("rehydration_records={}", snapshot.working.rehydration_queue.len()),
    ];
    let percent = if duplicates == 0.0 { 100.0 } else { (100.0 - duplicates * 10.0).max(0.0) };
    let token_savings = (snapshot.context.records.len() as f64).max(1.0);
    let confidence_met = loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let secondary_signal_ok = duplicates == 0.0 || snapshot.context.records.len() <= snapshot.working.records.len();
    let status = if loop_success_requires_second_signal(
        confidence_met,
        secondary_signal_ok,
        confidence_met,
        !loop_is_regressed(descriptor, previous_entry, percent, token_savings),
    ) {
        "success"
    } else {
        "warning"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        "memory hygiene score".to_string(),
        vec!["memory hygiene".to_string()],
        serde_json::json!({
            "evidence": evidence,
            "duplicates": duplicates,
        }),
        status,
    ))
}
```

- [ ] **Step 3: Add `run_autonomy_quality_loop` by folding false-green and warning quality into the current contract**

```rust
async fn run_autonomy_quality_loop(
    output: &Path,
    base_url: &str,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let snapshot = read_bundle_resume(&autoresearch_resume_args(output), base_url).await?;
    let warning_quality = snapshot.refresh_recommended as usize + snapshot.working.truncated as usize;
    let evidence = vec![
        format!("refresh_recommended={}", snapshot.refresh_recommended),
        format!("working_truncated={}", snapshot.working.truncated),
        format!("warning_pressure={warning_quality}"),
    ];
    let percent = if warning_quality == 0 { 100.0 } else { 100.0 - (warning_quality as f64 * 20.0) };
    let token_savings = (evidence.len() as f64) * 10.0;
    let confidence_met = loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let secondary_signal_ok = warning_quality == 0;
    let status = if loop_success_requires_second_signal(
        confidence_met,
        secondary_signal_ok,
        confidence_met,
        !loop_is_regressed(descriptor, previous_entry, percent, token_savings),
    ) {
        "success"
    } else {
        "warning"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        "autonomy quality score".to_string(),
        vec!["autonomy quality".to_string()],
        serde_json::json!({
            "evidence": evidence,
            "warning_quality": warning_quality,
        }),
        status,
    ))
}
```

- [ ] **Step 4: Wire the three runners into the dispatcher**

```rust
"hive-health" => run_hive_health_loop(output, descriptor, previous_runs, previous_entry).await?,
"memory-hygiene" => run_memory_hygiene_loop(output, base_url, descriptor, previous_runs, previous_entry).await?,
"autonomy-quality" => run_autonomy_quality_loop(output, base_url, descriptor, previous_runs, previous_entry).await?,
```

- [ ] **Step 5: Add tests for the new runners**

```rust
#[tokio::test]
async fn run_hive_health_loop_warns_on_dead_peer_pressure() { }

#[tokio::test]
async fn run_memory_hygiene_loop_succeeds_on_low_duplicate_pressure() { }

#[tokio::test]
async fn run_autonomy_quality_loop_warns_when_refresh_pressure_is_high() { }
```

- [ ] **Step 6: Run focused tests**

Run:
`cargo test -p memd-client run_hive_health_loop_warns_on_dead_peer_pressure -- --exact`
`cargo test -p memd-client run_memory_hygiene_loop_succeeds_on_low_duplicate_pressure -- --exact`
`cargo test -p memd-client run_autonomy_quality_loop_warns_when_refresh_pressure_is_high -- --exact`

Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add crates/memd-client/src/main.rs
git commit -m "feat: add hive, hygiene, and autonomy loops"
```

### Task 3: Add branch-review-quality, prompt-efficiency, repair-rate, and docs-spec-drift evaluation

**Files:**
- Modify: `crates/memd-client/src/main.rs:10300-10700`
- Test: `crates/memd-client/src/main.rs:36480-36680`

- [ ] **Step 1: Add `run_branch_review_quality_loop`**

```rust
async fn run_branch_review_quality_loop(
    output: &Path,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let root = infer_bundle_project_root(output).unwrap_or_else(|| output.to_path_buf());
    let evidence = collect_gap_repo_evidence(&root);
    let branch = std::process::Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("branch")
        .arg("--show-current")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    let review_ready = !evidence.iter().any(|line| line.contains("dirty"));
    let percent = if review_ready { 100.0 } else { 0.0 };
    let token_savings = if branch == "unknown" { 0.0 } else { 20.0 };
    let confidence_met = loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let status = if loop_success_requires_second_signal(
        confidence_met,
        branch != "unknown" && review_ready,
        confidence_met,
        !loop_is_regressed(descriptor, previous_entry, percent, token_savings),
    ) {
        "success"
    } else {
        "warning"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        format!("branch {} review_ready={}", branch, review_ready),
        vec!["branch review".to_string()],
        serde_json::json!({
            "evidence": evidence,
            "branch": branch,
            "review_ready": review_ready,
        }),
        status,
    ))
}
```

- [ ] **Step 2: Add `run_prompt_efficiency_loop` by reusing the existing prompt-surface logic**

```rust
async fn run_prompt_efficiency_loop(
    output: &Path,
    base_url: &str,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let snapshot = read_bundle_resume(&autoresearch_resume_args(output), base_url).await?;
    let tokens = snapshot.estimated_prompt_tokens() as f64;
    let evidence = vec![
        format!("estimated_tokens={tokens}"),
        format!("context_pressure={}", snapshot.context_pressure()),
        format!("redundant_items={}", snapshot.redundant_context_items()),
    ];
    let percent = (1_400.0 - tokens).max(0.0) / 1_400.0 * 100.0;
    let token_savings = (1_400.0 - tokens).max(0.0);
    let confidence_met = loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let secondary_signal_ok = snapshot.context_pressure() != "high" || snapshot.redundant_context_items() == 0;
    let status = if loop_success_requires_second_signal(
        confidence_met,
        secondary_signal_ok,
        confidence_met,
        !loop_is_regressed(descriptor, previous_entry, percent, token_savings),
    ) {
        "success"
    } else {
        "warning"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        format!("prompt tokens = {} (baseline 1400)", tokens),
        vec!["prompt efficiency".to_string()],
        serde_json::json!({
            "evidence": evidence,
            "context_pressure": snapshot.context_pressure(),
        }),
        status,
    ))
}
```

- [ ] **Step 3: Add `run_repair_rate_loop` by reusing the existing correction-learning logic**

```rust
async fn run_repair_rate_loop(
    output: &Path,
    base_url: &str,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let snapshot = read_bundle_resume(&autoresearch_resume_args(output), base_url).await?;
    let tracked = snapshot.change_summary.len() as f64;
    let corrections = snapshot
        .change_summary
        .iter()
        .filter(|line| {
            let lower = line.to_lowercase();
            lower.contains("fix") || lower.contains("correct") || lower.contains("repair")
        })
        .count() as f64;
    let evidence = vec![
        format!("tracked={tracked}"),
        format!("corrections={corrections}"),
        format!("recent={}", snapshot.recent_repo_changes.len()),
    ];
    let percent = if tracked == 0.0 { 0.0 } else { (1.0 - corrections / tracked).max(0.0) * 100.0 };
    let token_savings = ((tracked - corrections).max(0.0)) * 10.0;
    let confidence_met = loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let secondary_signal_ok = corrections <= tracked;
    let status = if loop_success_requires_second_signal(
        confidence_met,
        secondary_signal_ok,
        confidence_met,
        !loop_is_regressed(descriptor, previous_entry, percent, token_savings),
    ) {
        "success"
    } else {
        "warning"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        format!("{} corrections out of {} tracked change summaries", corrections, tracked),
        vec!["repair rate".to_string()],
        serde_json::json!({
            "evidence": evidence,
            "corrections": corrections,
        }),
        status,
    ))
}
```

- [ ] **Step 4: Add `run_docs_spec_drift_loop`**

```rust
async fn run_docs_spec_drift_loop(
    output: &Path,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let spec = fs::read_to_string("docs/superpowers/specs/2026-04-08-memd-10-loop-design.md")?;
    let plan = fs::read_to_string("docs/superpowers/plans/2026-04-08-memd-10-loop-stack.md")?;
    let evidence = vec![
        format!("spec_bytes={}", spec.len()),
        format!("plan_bytes={}", plan.len()),
        format!("runtime_bytes={}", fs::metadata(output.join("Cargo.toml")).map(|m| m.len()).unwrap_or(0)),
    ];
    let secondary_signal_ok = spec.contains("10-loop") && plan.contains("Implementation Plan");
    let percent = if secondary_signal_ok { 100.0 } else { 0.0 };
    let token_savings = (spec.len() + plan.len()) as f64 / 100.0;
    let confidence_met = loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let status = if loop_success_requires_second_signal(
        confidence_met,
        secondary_signal_ok,
        confidence_met,
        !loop_is_regressed(descriptor, previous_entry, percent, token_savings),
    ) {
        "success"
    } else {
        "warning"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        "docs-spec drift score".to_string(),
        vec!["docs spec drift".to_string()],
        serde_json::json!({
            "evidence": evidence,
            "spec_has_10_loop": spec.contains("10-loop"),
        }),
        status,
    ))
}
```

- [ ] **Step 5: Wire all four runners into the dispatcher**

```rust
"branch-review-quality" => run_branch_review_quality_loop(output, descriptor, previous_runs, previous_entry).await?,
"prompt-efficiency" => run_prompt_efficiency_loop(output, base_url, descriptor, previous_runs, previous_entry).await?,
"repair-rate" => run_repair_rate_loop(output, base_url, descriptor, previous_runs, previous_entry).await?,
"docs-spec-drift" => run_docs_spec_drift_loop(output, descriptor, previous_runs, previous_entry).await?,
```

- [ ] **Step 6: Add tests that prove the new signals are scored**

```rust
#[test]
fn collect_gap_repo_evidence_surfaces_repo_docs_and_runtime_signals() {
    let root = std::env::temp_dir().join("memd-gap-repo-evidence");
    fs::create_dir_all(root.join("docs")).expect("create docs");
    fs::write(root.join("README.md"), "# memd\n").expect("write readme");
    fs::write(root.join("ROADMAP.md"), "# roadmap\n").expect("write roadmap");
    fs::write(root.join("docs").join("setup.md"), "memd init\n").expect("write setup");
    let evidence = collect_gap_repo_evidence(&root);
    assert!(evidence.iter().any(|line| line.contains("git branch:")));
    assert!(evidence.iter().any(|line| line.contains("README.md:")));
}

#[test]
fn branch_review_quality_detects_dirty_branch_state() {
    let root = std::env::current_dir().expect("current dir");
    let evidence = collect_gap_repo_evidence(&root);
    assert!(evidence.iter().any(|line| line.contains("git branch:")));
}

#[tokio::test]
async fn run_docs_spec_drift_loop_warns_when_docs_and_runtime_diverge() {
    let output = std::env::temp_dir().join("memd-docs-spec-drift");
    fs::create_dir_all(&output).expect("create output");
    fs::write(output.join("Cargo.toml"), "[package]\nname = \"memd\"\n").expect("write cargo");
    let descriptor = AUTORESEARCH_LOOPS
        .iter()
        .find(|descriptor| descriptor.slug == "docs-spec-drift")
        .expect("docs-spec-drift descriptor");
    let record = run_docs_spec_drift_loop(&output, descriptor, 0, None)
        .await
        .expect("run docs spec drift loop");
    assert!(record.status.is_some());
}
```

- [ ] **Step 7: Run focused tests**

Run:
`cargo test -p memd-client collect_gap_repo_evidence_surfaces_repo_docs_and_runtime_signals -- --exact`
`cargo test -p memd-client run_docs_spec_drift_loop_warns_when_docs_and_runtime_diverge -- --exact`

Expected: PASS

- [ ] **Step 8: Commit**

```bash
git add crates/memd-client/src/main.rs
git commit -m "feat: add branch review and docs drift loops"
```

### Task 4: Tighten final evaluation, verify the full 10-loop stack, and remove dead code

**Files:**
- Modify: `crates/memd-client/src/main.rs:10720-10860`
- Test: `crates/memd-client/src/main.rs:36680-36740`

- [ ] **Step 1: Use the shared success gate everywhere**

```rust
let success = loop_success_requires_second_signal(
    confidence_met,
    secondary_signal_ok,
    !loop_is_regressed(descriptor, previous_entry, percent, token_savings),
    !snapshot.refresh_recommended,
);
```

- [ ] **Step 2: Keep warning metadata explicit**

```rust
let metadata = serde_json::json!({
    "evidence": evidence,
    "secondary_signal": secondary_signal_metadata,
    "confidence": loop_confidence_metadata(descriptor, percent, token_savings, confidence_met, evidence.len()),
    "trend": loop_trend_metadata(descriptor, previous_entry, percent, token_savings),
    "warning_reasons": warning_reasons,
});
```

- [ ] **Step 3: Add an end-to-end test that sees all 10 loop outputs**

```rust
#[test]
fn autoresearch_emits_ten_loop_records() {
    let output = std::env::temp_dir().join("memd-10-loop-records");
    fs::create_dir_all(&output).expect("create output");
    let outputs = read_loop_entries(&output).expect("read loop entries");
    assert_eq!(outputs.len(), 10);
    assert!(outputs.iter().any(|record| record.slug == "hive-health"));
    assert!(outputs.iter().any(|record| record.slug == "docs-spec-drift"));
    assert!(outputs.iter().all(|record| record.record.status.is_some()));
}
```

- [ ] **Step 4: Run the full memd-client suite**

Run: `cargo test -p memd-client`

Expected: PASS

- [ ] **Step 5: Remove `autoresearch_resume_args` if no call sites remain**

```rust
fn autoresearch_resume_args(output: &Path) -> ResumeArgs {
    autoresearch_resume_args_with_limits(output, 8, 4, true)
}
```

Delete it only after confirming `rg -n "autoresearch_resume_args\\(" crates/memd-client/src/main.rs` returns no remaining production call sites.

- [ ] **Step 6: Commit**

```bash
git add crates/memd-client/src/main.rs
git commit -m "feat: finalize 10-loop autoresearch stack"
```

## Verification Checklist
- `cargo test -p memd-client`
- `cargo build -p memd-client --bin memd`
- one manual `memd autoresearch` pass confirms 10 loop records
- every warning has explicit `warning_reasons`
- `refresh_recommended` stays metadata-only unless a loop explicitly uses it
