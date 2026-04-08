# Live Event Compiler Hook Checkpoint Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `hook capture` and `checkpoint` emit live bundle events that are compiled into visible event pages immediately, without changing the existing memory-object model.

**Architecture:** Keep the current memory pages as the visible object layer and treat the bundle event log as the incremental input. The live path stays bundle-local and deterministic: `hook capture` and `checkpoint` write one compact event record, the compiler refreshes `MEMD_EVENTS.md` plus compiled event pages, and memory pages continue to point at the event lane.

**Tech Stack:** Rust, `anyhow`, `serde_json`, existing `memd-client` bundle helpers, existing `memd` test harness, tempfile-based command tests.

---

### Task 1: Wire live event emission into the two highest-signal command surfaces

**Files:**
- Modify: `crates/memd-client/src/main.rs:2588-2655`
- Modify: `crates/memd-client/src/main.rs:3440-3635`
- Modify: `crates/memd-client/src/main.rs:5848-5866`

- [ ] **Step 1: Write the failing test**

```rust
#[tokio::test]
async fn checkpoint_and_hook_capture_refresh_live_event_pages() {
    let dir = std::env::temp_dir().join(format!("memd-live-events-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("create temp bundle");

    let state = Arc::new(Mutex::new(MockState::default()));
    let base_url = spawn_mock_runtime_server(state.clone(), false).await;
    let snapshot = read_bundle_resume(&autoresearch_resume_args(&dir), &base_url)
        .await
        .expect("read bundle resume");

    write_bundle_memory_files(&dir, &snapshot, None, false)
        .await
        .expect("write bundle memory files");

    let events = read_bundle_event_log(&dir).expect("read bundle event log");
    assert_eq!(events.len(), 1);
    assert!(
        events[0].summary.contains("resume_snapshot")
            || events[0].summary.contains("live_snapshot")
    );

    let root_events = fs::read_to_string(dir.join("MEMD_EVENTS.md"))
        .expect("read event log markdown");
    assert!(root_events.contains("# memd event log"));
    assert!(root_events.contains("compiled/events/latest.md"));

    let compiled = fs::read_to_string(dir.join("compiled/events/latest.md"))
        .expect("read compiled event index");
    assert!(compiled.contains("# memd event index"));

    fs::remove_dir_all(dir).expect("cleanup temp bundle");
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
cargo test -p memd-client checkpoint_and_hook_capture_refresh_live_event_pages -- --exact
```

Expected:

- FAIL until the event refresh path is explicitly wired through the command surfaces.

- [ ] **Step 3: Write the minimal implementation**

```rust
fn refresh_live_bundle_event_pages(
    output: &Path,
    snapshot: &ResumeSnapshot,
    handoff: Option<&HandoffSnapshot>,
) -> anyhow::Result<()> {
    write_bundle_event_files(output, snapshot, handoff)?;
    Ok(())
}

// In Commands::Checkpoint after the snapshot write:
refresh_live_bundle_event_pages(&args.output, &snapshot, None)?;

// In Commands::Hook::Capture after the snapshot write:
refresh_live_bundle_event_pages(&args.output, &snapshot, None)?;
```

- [ ] **Step 4: Run the test to verify it passes**

Run:

```bash
cargo test -p memd-client checkpoint_and_hook_capture_refresh_live_event_pages -- --exact
```

Expected:

- PASS
- `MEMD_EVENTS.md` and `compiled/events/latest.md` are regenerated from the live log

- [ ] **Step 5: Commit**

```bash
git add crates/memd-client/src/main.rs
git commit -m "feat: wire live event compiler to hook and checkpoint"
```

### Task 2: Add command-level verification for the new live event lane

**Files:**
- Modify: `crates/memd-client/src/main.rs:3163-3235`
- Modify: `crates/memd-client/src/main.rs:28141-28250`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn events_summary_reports_live_event_lane_state() {
    let root = std::env::temp_dir().join(format!("memd-event-summary-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(root.join("compiled/events/items/live_snapshot"))
        .expect("create compiled event dir");
    fs::write(
        root.join("compiled/events/latest.md"),
        "# memd event index\n\n- [live_snapshot](items/live_snapshot/live_snapshot-01-abcd1234.md)\n",
    )
    .expect("write compiled event index");

    let index = render_compiled_event_index(&root).expect("render event index");
    let summary = render_compiled_event_index_summary(&root, &index);
    assert!(summary.contains("event index"));
    assert!(summary.contains("kinds="));
    assert!(summary.contains("items="));

    fs::remove_dir_all(root).expect("cleanup temp bundle");
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
cargo test -p memd-client events_summary_reports_live_event_lane_state -- --exact
```

Expected:

- FAIL until the `memd events` summary path is fully stable and covered by tests.

- [ ] **Step 3: Write the minimal implementation**

```rust
// Keep the summary path compact and source-linked.
if args.summary {
    println!("{}", render_compiled_event_index_summary(&bundle_root, &index));
} else if args.list {
    println!("{}", render_compiled_event_index_json(&bundle_root, &index));
} else {
    println!("{}", render_compiled_event_index_markdown(&bundle_root, &index));
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run:

```bash
cargo test -p memd-client events_summary_reports_live_event_lane_state -- --exact
```

Expected:

- PASS

- [ ] **Step 5: Commit**

```bash
git add crates/memd-client/src/main.rs
git commit -m "test: cover the compiled event summary surface"
```

### Task 3: Keep the docs and smoke commands aligned with the live event lane

**Files:**
- Modify: `docs/setup.md:54-84`
- Modify: `docs/api.md:173-190`

- [ ] **Step 1: Write the failing doc expectation**

```bash
rg -n "hook capture|checkpoint|memd events|MEMD_EVENTS" docs/setup.md docs/api.md
```

Expected:

- the setup guide should mention that `hook capture` and `checkpoint` refresh the event lane
- the API guide should mention `memd events --summary`, `--list`, `--query`, and `--open`

- [ ] **Step 2: Apply the doc wording**

```md
`hook capture` and `checkpoint` refresh the live event lane.
Use `memd events --summary` to inspect the compiled event index.
Use `memd events --list` for JSON export.
Use `memd events --query <term>` and `memd events --open <target>` for drilldown.
```

- [ ] **Step 3: Run the smoke checks**

Run:

```bash
cargo test -p memd-client --quiet
cargo test --workspace --quiet
cargo run -p memd-client --bin memd -- events --summary --root .memd
cargo run -p memd-client --bin memd -- events --list --root .memd
```

Expected:

- tests pass
- `memd events --summary` prints the event index status
- `memd events --list` prints JSON with `root`, `kind_count`, `item_count`, and `pages`

- [ ] **Step 4: Commit**

```bash
git add docs/setup.md docs/api.md
git commit -m "docs: add live event compiler usage"
```
