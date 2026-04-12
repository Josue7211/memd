# Phase A Raw Truth Spine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make raw truth capture first-class so major harness inputs enter one source-linked raw event spine before later promotion, resume, and atlas layers build on top.

**Architecture:** Reuse the existing store event machinery on the server, then add a bundle-local raw spine in `memd-client` that records every meaningful ingest path (`remember`, `checkpoint`, `ingest`, `hook capture`, `hook spill`). The server remains the canonical memory store, while the bundle-local raw spine becomes the read-once evidence trail used by resume, compiled event pages, and later continuity work.

**Tech Stack:** Rust, `clap`, `serde`, `chrono`, `uuid`, `memd-client`, `memd-server`, SQLite-backed `memd-server` store, existing bundle `.memd/state` + `.memd/compiled` artifacts.

---

## File Map

**Create**

- `crates/memd-client/src/runtime/raw_spine.rs`
- `crates/memd-client/src/main_tests/runtime_raw_spine_tests/mod.rs`

**Modify**

- `crates/memd-client/src/runtime/mod.rs`
- `crates/memd-client/src/runtime/checkpoint.rs`
- `crates/memd-client/src/runtime/ingest_runtime.rs`
- `crates/memd-client/src/cli/cli_hook_runtime.rs`
- `crates/memd-client/src/compiled/event.rs`
- `crates/memd-client/src/main_tests/mod.rs`
- `crates/memd-server/src/tests/mod.rs`
- `docs/core/setup.md`
- `docs/core/architecture.md`
- `docs/verification/FEATURES.md`

**Why these files**

- `runtime/raw_spine.rs` becomes the one small unit responsible for raw spine record creation, file persistence, and bundle-local summaries.
- `checkpoint.rs`, `ingest_runtime.rs`, and `cli_hook_runtime.rs` already own the real capture paths. Wire raw spine writes there instead of inventing a second ingestion layer.
- `compiled/event.rs` already owns bundle event artifacts. Extend it to surface raw spine records rather than creating another renderer.
- `memd-server/src/tests/mod.rs` is the easiest place to assert that `/memory/store` and `/memory/candidates` still emit source-linked server-side `memory_events`.
- `runtime_raw_spine_tests` gives us a focused TDD slice instead of dumping more cases into `runtime_memory_tests_core.rs`.

### Task 1: Harden the server-side event spine contract

**Files:**
- Modify: `crates/memd-server/src/tests/mod.rs`

- [ ] **Step 1: Write the failing server test for canonical store events**

Add this test near the other route/state tests in `crates/memd-server/src/tests/mod.rs`:

```rust
#[tokio::test]
async fn store_item_records_source_linked_event_for_canonical_memory() {
    let (_dir, state) = temp_state("memd-store-event-canonical");

    let (item, duplicate) = state
        .store_item(
            StoreMemoryRequest {
                content: "raw truth: user corrected deployment target".to_string(),
                kind: MemoryKind::Decision,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("core".to_string()),
                visibility: Some(MemoryVisibility::Workspace),
                belief_branch: None,
                source_agent: Some("codex@test".to_string()),
                source_system: Some("hook-capture".to_string()),
                source_path: Some(".memd/agents/CODEX_WAKEUP.md".to_string()),
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.91),
                ttl_seconds: Some(86_400),
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec!["raw-spine".to_string(), "correction".to_string()],
                status: Some(MemoryStatus::Active),
            },
            MemoryStage::Canonical,
        )
        .expect("store canonical memory");

    assert!(duplicate.is_none());

    let timeline = state
        .store
        .timeline_for_memory(TimelineMemoryRequest {
            id: item.id,
            limit: Some(10),
        })
        .expect("load timeline");

    assert!(!timeline.events.is_empty(), "expected stored item timeline event");
    let event = &timeline.events[0];
    assert_eq!(event.event_type, "stored_canonical");
    assert_eq!(event.source_system.as_deref(), Some("hook-capture"));
    assert_eq!(
        event.source_path.as_deref(),
        Some(".memd/agents/CODEX_WAKEUP.md")
    );
    assert!(event.tags.iter().any(|tag| tag == "raw-spine"));
}
```

Run:

```bash
cargo test -q -p memd-server store_item_records_source_linked_event_for_canonical_memory -- --nocapture
```

Expected: PASS or close-to-pass. If it already passes, keep the test; it becomes the server-side contract for Phase A.

- [ ] **Step 2: Write the failing server test for candidate store events**

Add a second test proving candidate writes also retain raw source linkage:

```rust
#[tokio::test]
async fn store_item_records_source_linked_event_for_candidate_memory() {
    let (_dir, state) = temp_state("memd-store-event-candidate");

    let (item, duplicate) = state
        .store_item(
            StoreMemoryRequest {
                content: "checkpoint: parser lane blocked by stale resume packet".to_string(),
                kind: MemoryKind::Status,
                scope: MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("core".to_string()),
                visibility: Some(MemoryVisibility::Workspace),
                belief_branch: None,
                source_agent: Some("codex@test".to_string()),
                source_system: Some("checkpoint".to_string()),
                source_path: Some("checkpoint".to_string()),
                source_quality: Some(SourceQuality::Canonical),
                confidence: Some(0.78),
                ttl_seconds: Some(86_400),
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: vec!["checkpoint".to_string(), "raw-spine".to_string()],
                status: Some(MemoryStatus::Active),
            },
            MemoryStage::Candidate,
        )
        .expect("store candidate memory");

    assert!(duplicate.is_none());

    let timeline = state
        .store
        .timeline_for_memory(TimelineMemoryRequest {
            id: item.id,
            limit: Some(10),
        })
        .expect("load timeline");

    assert!(!timeline.events.is_empty(), "expected candidate timeline event");
    let event = &timeline.events[0];
    assert_eq!(event.event_type, "stored_candidate");
    assert_eq!(event.source_system.as_deref(), Some("checkpoint"));
    assert_eq!(event.source_path.as_deref(), Some("checkpoint"));
}
```

Run:

```bash
cargo test -q -p memd-server store_item_records_source_linked_event_for_candidate_memory -- --nocapture
```

Expected: PASS or near-pass. If it fails, fix server event defaults before moving on.

- [ ] **Step 3: Tighten server defaults only if the tests expose gaps**

If one of the tests fails, patch `crates/memd-server/src/main.rs` in `AppState::store_item(...)` or `record_item_event(...)` with the minimal fix only. The intended event shape is:

```rust
let _ = self.record_item_event(
    &item,
    event_type_for_stage(stage),
    format!(
        "{} memory item stored",
        match stage {
            MemoryStage::Candidate => "candidate",
            MemoryStage::Canonical => "canonical",
        }
    ),
);
```

And the event payload must keep:

```rust
RecordEventArgs {
    event_type: event_type.to_string(),
    summary,
    occurred_at: item.updated_at,
    project: item.project.clone(),
    namespace: item.namespace.clone(),
    workspace: item.workspace.clone(),
    source_agent: item.source_agent.clone(),
    source_system: item.source_system.clone(),
    source_path: item.source_path.clone(),
    related_entity_ids: Vec::new(),
    tags: item.tags.clone(),
    context,
    confidence: item.confidence,
    salience_score: entity.record.salience_score,
}
```

- [ ] **Step 4: Run the server test slice**

Run:

```bash
cargo test -q -p memd-server store_item_records_source_linked_event -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/memd-server/src/tests/mod.rs crates/memd-server/src/main.rs
git commit -m "test: lock source-linked server event spine contract"
```

### Task 2: Add a bundle-local raw spine record model and persistence helpers

**Files:**
- Create: `crates/memd-client/src/runtime/raw_spine.rs`
- Modify: `crates/memd-client/src/runtime/mod.rs`
- Modify: `crates/memd-client/src/main_tests/mod.rs`
- Create: `crates/memd-client/src/main_tests/runtime_raw_spine_tests/mod.rs`

- [ ] **Step 1: Write the failing raw spine helper tests**

Create `crates/memd-client/src/main_tests/runtime_raw_spine_tests/mod.rs`:

```rust
use super::*;

#[test]
fn write_raw_spine_records_merges_and_sorts_latest_first() {
    let dir = std::env::temp_dir().join(format!("memd-raw-spine-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("create raw spine dir");

    let older = RawSpineRecord {
        id: "older".to_string(),
        event_type: "remember".to_string(),
        stage: "canonical".to_string(),
        source_system: Some("remember".to_string()),
        source_path: Some("README.md".to_string()),
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        workspace: Some("core".to_string()),
        confidence: Some(0.8),
        tags: vec!["raw-spine".to_string()],
        content_preview: "older record".to_string(),
        recorded_at: chrono::Utc::now() - chrono::Duration::minutes(5),
    };

    let newer = RawSpineRecord {
        id: "newer".to_string(),
        event_type: "checkpoint".to_string(),
        stage: "candidate".to_string(),
        source_system: Some("checkpoint".to_string()),
        source_path: Some("checkpoint".to_string()),
        project: Some("memd".to_string()),
        namespace: Some("main".to_string()),
        workspace: Some("core".to_string()),
        confidence: Some(0.9),
        tags: vec!["raw-spine".to_string(), "checkpoint".to_string()],
        content_preview: "newer record".to_string(),
        recorded_at: chrono::Utc::now(),
    };

    write_raw_spine_records(&dir, &[older.clone()]).expect("write older");
    write_raw_spine_records(&dir, &[newer.clone()]).expect("write newer");

    let records = read_raw_spine_records(&dir).expect("read raw spine");
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].id, "newer");
    assert_eq!(records[1].id, "older");
}

#[test]
fn derive_raw_spine_record_keeps_source_linkage() {
    let record = derive_raw_spine_record(
        "hook_capture",
        "candidate",
        Some("hook-capture"),
        Some(".memd/agents/CODEX_WAKEUP.md"),
        Some("memd"),
        Some("main"),
        Some("core"),
        Some(0.88),
        &["raw-spine", "correction"],
        "corrected deployment target is local-first",
    );

    assert_eq!(record.event_type, "hook_capture");
    assert_eq!(record.stage, "candidate");
    assert_eq!(record.source_system.as_deref(), Some("hook-capture"));
    assert_eq!(
        record.source_path.as_deref(),
        Some(".memd/agents/CODEX_WAKEUP.md")
    );
    assert!(record.tags.iter().any(|tag| tag == "raw-spine"));
}
```

Register the module in `crates/memd-client/src/main_tests/mod.rs`:

```rust
mod runtime_raw_spine_tests;
```

Run:

```bash
cargo test -q -p memd-client runtime_raw_spine -- --nocapture
```

Expected: FAIL because `RawSpineRecord`, `read_raw_spine_records`, `write_raw_spine_records`, and `derive_raw_spine_record` do not exist yet.

- [ ] **Step 2: Implement the raw spine helper**

Create `crates/memd-client/src/runtime/raw_spine.rs`:

```rust
use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RawSpineRecord {
    pub(crate) id: String,
    pub(crate) event_type: String,
    pub(crate) stage: String,
    pub(crate) source_system: Option<String>,
    pub(crate) source_path: Option<String>,
    pub(crate) project: Option<String>,
    pub(crate) namespace: Option<String>,
    pub(crate) workspace: Option<String>,
    pub(crate) confidence: Option<f32>,
    pub(crate) tags: Vec<String>,
    pub(crate) content_preview: String,
    pub(crate) recorded_at: DateTime<Utc>,
}

fn raw_spine_path(output: &Path) -> PathBuf {
    output.join("state").join("raw-spine.jsonl")
}

pub(crate) fn derive_raw_spine_record(
    event_type: &str,
    stage: &str,
    source_system: Option<&str>,
    source_path: Option<&str>,
    project: Option<&str>,
    namespace: Option<&str>,
    workspace: Option<&str>,
    confidence: Option<f32>,
    tags: &[&str],
    content: &str,
) -> RawSpineRecord {
    let preview = compact_inline(content.trim(), 180);
    let signature = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}",
        event_type,
        stage,
        source_system.unwrap_or("none"),
        source_path.unwrap_or("none"),
        project.unwrap_or("none"),
        namespace.unwrap_or("none"),
        workspace.unwrap_or("none"),
        preview
    );
    RawSpineRecord {
        id: format!("raw-{}", short_hash_text(&signature)),
        event_type: event_type.to_string(),
        stage: stage.to_string(),
        source_system: source_system.map(str::to_string),
        source_path: source_path.map(str::to_string),
        project: project.map(str::to_string),
        namespace: namespace.map(str::to_string),
        workspace: workspace.map(str::to_string),
        confidence,
        tags: tags.iter().map(|value| value.to_string()).collect(),
        content_preview: preview,
        recorded_at: Utc::now(),
    }
}

pub(crate) fn read_raw_spine_records(output: &Path) -> anyhow::Result<Vec<RawSpineRecord>> {
    let path = raw_spine_path(output);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let mut records = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        records.push(serde_json::from_str::<RawSpineRecord>(trimmed)?);
    }
    Ok(records)
}

pub(crate) fn write_raw_spine_records(output: &Path, records: &[RawSpineRecord]) -> anyhow::Result<()> {
    let path = raw_spine_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let mut merged = std::collections::BTreeMap::<String, RawSpineRecord>::new();
    for record in read_raw_spine_records(output)? {
        merged.insert(record.id.clone(), record);
    }
    for record in records {
        merged.insert(record.id.clone(), record.clone());
    }

    let mut merged = merged.into_values().collect::<Vec<_>>();
    merged.sort_by(|left, right| right.recorded_at.cmp(&left.recorded_at));
    merged.truncate(512);

    let mut body = String::new();
    for record in merged {
        body.push_str(&serde_json::to_string(&record)?);
        body.push('\n');
    }
    fs::write(&path, body).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}
```

Export it from `crates/memd-client/src/runtime/mod.rs`:

```rust
mod raw_spine;
pub(crate) use raw_spine::*;
```

- [ ] **Step 3: Run the raw spine helper tests**

Run:

```bash
cargo test -q -p memd-client runtime_raw_spine -- --nocapture
```

Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add crates/memd-client/src/runtime/raw_spine.rs crates/memd-client/src/runtime/mod.rs crates/memd-client/src/main_tests/mod.rs crates/memd-client/src/main_tests/runtime_raw_spine_tests/mod.rs
git commit -m "feat: add bundle raw spine helper"
```

### Task 3: Wire all real capture paths into the raw spine

**Files:**
- Modify: `crates/memd-client/src/runtime/checkpoint.rs`
- Modify: `crates/memd-client/src/runtime/ingest_runtime.rs`
- Modify: `crates/memd-client/src/cli/cli_hook_runtime.rs`
- Modify: `crates/memd-client/src/main_tests/runtime_raw_spine_tests/mod.rs`

- [ ] **Step 1: Write the failing client integration tests**

Extend `crates/memd-client/src/main_tests/runtime_raw_spine_tests/mod.rs`.

Use the **real existing test helpers**, not invented ones:

- for hook capture, mirror the setup shape from `hook_capture_can_supersede_stale_memory_after_promotion` in `crates/memd-client/src/main_tests/runtime_memory_tests/runtime_memory_tests_core.rs`
- use `MockRuntimeState::default()` + `spawn_mock_runtime_server(state.clone(), false).await`
- create a temp bundle dir and call `run_hook_mode(&client, &base_url, HookArgs { ... })`
- after the call, read `.memd/state/raw-spine.jsonl` through `read_raw_spine_records(&dir)` and assert:
  - one `hook_capture` record exists
  - one `checkpoint` record exists when promotion/checkpoint flow runs

Add a second spill test using the compaction packet shape already exercised in `crates/memd-client/src/e2e_tests.rs`:

- build a small `CompactionPacket` inline in the test instead of relying on a non-existent helper
- write it to a temp file
- call `run_hook_mode(&client, &base_url, HookArgs { mode: HookMode::Spill(...) })`
- then assert `read_raw_spine_records(&dir)` contains a `hook_spill` record

The test file should stay self-contained and compile only against helpers that already exist in `main_tests` or are defined locally in the new module.

Run:

```bash
cargo test -q -p memd-client raw_spine -- --nocapture
```

Expected: FAIL because none of these runtimes write raw spine records yet.

- [ ] **Step 2: Wire `remember` / `checkpoint` writes in `checkpoint.rs`**

In `crates/memd-client/src/runtime/checkpoint.rs`, after successful writes, append raw spine records:

```rust
pub(crate) fn append_raw_spine_store_record(
    output: &Path,
    event_type: &str,
    stage: &str,
    project: Option<&str>,
    namespace: Option<&str>,
    workspace: Option<&str>,
    source_system: Option<&str>,
    source_path: Option<&str>,
    confidence: Option<f32>,
    tags: &[String],
    content: &str,
) -> anyhow::Result<()> {
    let tag_refs = tags.iter().map(|value| value.as_str()).collect::<Vec<_>>();
    let record = derive_raw_spine_record(
        event_type,
        stage,
        source_system,
        source_path,
        project,
        namespace,
        workspace,
        confidence,
        &tag_refs,
        content,
    );
    write_raw_spine_records(output, &[record])
}
```

Call it from `remember_with_bundle_defaults(...)` after the store succeeds:

```rust
let response = client
    .store(&memd_schema::StoreMemoryRequest { /* existing payload */ })
    .await?;

append_raw_spine_store_record(
    &args.output,
    "remember",
    "canonical",
    project.as_deref(),
    namespace.as_deref(),
    workspace.as_deref(),
    Some("memd"),
    args.source_path.as_deref(),
    args.confidence,
    &args.tag,
    &content,
)?;

Ok(response)
```

And from `checkpoint_with_bundle_defaults(...)` after translation:

```rust
let response = remember_with_bundle_defaults(&translated, base_url).await?;
append_raw_spine_store_record(
    &args.output,
    "checkpoint",
    "candidate",
    translated.project.as_deref(),
    translated.namespace.as_deref(),
    translated.workspace.as_deref(),
    translated.source_system.as_deref(),
    translated.source_path.as_deref(),
    translated.confidence,
    &translated.tag,
    translated.content.as_deref().unwrap_or_default(),
)?;
Ok(response)
```

- [ ] **Step 3: Wire `ingest` and `hook` paths**

In `crates/memd-client/src/runtime/ingest_runtime.rs`, after `client.candidate(&req).await?` succeeds inside `ingest_text_memory(...)`:

```rust
append_raw_spine_store_record(
    &resolve_default_bundle_root(args)?,
    "ingest",
    "candidate",
    args.project.as_deref(),
    args.namespace.as_deref(),
    args.workspace.as_deref(),
    args.source_system.as_deref().or(Some("ingest")),
    args.source_path.as_deref(),
    args.confidence,
    &args.tag,
    &req.content,
)?;
```

In `crates/memd-client/src/cli/cli_hook_runtime.rs`, add focused raw spine writes:

```rust
append_raw_spine_store_record(
    &args.output,
    "hook_capture",
    "candidate",
    args.project.as_deref(),
    args.namespace.as_deref(),
    args.workspace.as_deref(),
    Some("hook-capture"),
    args.source_path.as_deref(),
    args.confidence,
    &args.tag,
    &content,
)?;
```

And in `HookMode::Spill` after `candidate_batch(...)` succeeds:

```rust
let summary = serde_json::to_string(&spill).unwrap_or_else(|_| "compaction spill".to_string());
append_raw_spine_store_record(
    &args.input.output,
    "hook_spill",
    "candidate",
    None,
    None,
    None,
    Some("hook-spill"),
    Some("compaction-packet"),
    Some(0.8),
    &vec!["spill".to_string(), "raw-spine".to_string()],
    &summary,
)?;
```

- [ ] **Step 4: Run the client raw spine slice**

Run:

```bash
cargo test -q -p memd-client raw_spine -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/memd-client/src/runtime/checkpoint.rs crates/memd-client/src/runtime/ingest_runtime.rs crates/memd-client/src/cli/cli_hook_runtime.rs crates/memd-client/src/main_tests/runtime_raw_spine_tests/mod.rs
git commit -m "feat: write raw spine records from ingest paths"
```

### Task 4: Surface the raw spine in compiled bundle artifacts

**Files:**
- Modify: `crates/memd-client/src/compiled/event.rs`
- Modify: `crates/memd-client/src/main_tests/runtime_raw_spine_tests/mod.rs`

- [ ] **Step 1: Write the failing rendering test**

Add this to `runtime_raw_spine_tests/mod.rs`:

```rust
#[test]
fn write_bundle_event_files_includes_raw_spine_section() {
    let dir = std::env::temp_dir().join(format!("memd-raw-spine-pages-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("create bundle dir");

    write_raw_spine_records(
        &dir,
        &[derive_raw_spine_record(
            "hook_capture",
            "candidate",
            Some("hook-capture"),
            Some(".memd/agents/CODEX_WAKEUP.md"),
            Some("memd"),
            Some("main"),
            Some("core"),
            Some(0.9),
            &["raw-spine", "correction"],
            "decision: preserve raw truth before promotion",
        )],
    )
    .expect("write raw spine");

    let snapshot = codex_test_snapshot("memd", "main", "codex@test");
    write_bundle_event_files(&dir, &snapshot, None).expect("write bundle event files");

    let latest = std::fs::read_to_string(dir.join("compiled/events/latest.md")).expect("read latest");
    assert!(latest.contains("## Raw Spine"));
    assert!(latest.contains("hook_capture"));
    assert!(latest.contains(".memd/agents/CODEX_WAKEUP.md"));
}
```

Run:

```bash
cargo test -q -p memd-client write_bundle_event_files_includes_raw_spine_section -- --nocapture
```

Expected: FAIL because the event renderer does not read `raw-spine.jsonl` yet.

- [ ] **Step 2: Extend the compiled event renderer**

In `crates/memd-client/src/compiled/event.rs`, add a helper:

```rust
fn render_raw_spine_markdown(output: &Path) -> anyhow::Result<String> {
    let records = read_raw_spine_records(output)?;
    if records.is_empty() {
        return Ok(String::new());
    }

    let mut markdown = String::from("## Raw Spine\n\n");
    for record in records.iter().take(24) {
        markdown.push_str(&format!(
            "- `{}` stage=`{}` source=`{}` path=`{}` preview=`{}`\n",
            record.event_type,
            record.stage,
            record.source_system.as_deref().unwrap_or("none"),
            record.source_path.as_deref().unwrap_or("none"),
            record.content_preview
        ));
    }
    markdown.push('\n');
    Ok(markdown)
}
```

Then append it inside `render_compiled_event_index_markdown(...)` or the function that writes `compiled/events/latest.md`:

```rust
let raw_spine = render_raw_spine_markdown(output)?;
if !raw_spine.is_empty() {
    markdown.push_str(&raw_spine);
}
```

- [ ] **Step 3: Run the rendering test**

Run:

```bash
cargo test -q -p memd-client write_bundle_event_files_includes_raw_spine_section -- --nocapture
```

Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add crates/memd-client/src/compiled/event.rs crates/memd-client/src/main_tests/runtime_raw_spine_tests/mod.rs
git commit -m "feat: surface raw spine in compiled bundle events"
```

### Task 5: Document and verify the Phase A contract

**Files:**
- Modify: `docs/core/setup.md`
- Modify: `docs/core/architecture.md`
- Modify: `docs/verification/FEATURES.md`

- [ ] **Step 1: Document the raw spine commands**

In `docs/core/setup.md`, add the exact inspection loop near the existing hook examples:

````md
Inspect the raw truth spine:

```bash
cat .memd/state/raw-spine.jsonl
```

The raw spine should show source-linked records from:

- `memd remember`
- `memd checkpoint`
- `memd ingest`
- `memd hook capture`
- `memd hook spill --apply`
````
```

- [ ] **Step 2: Update architecture wording**

In `docs/core/architecture.md`, tighten the live loop section so Phase A is explicit:

```md
Phase A raw truth spine:

1. capture raw event, artifact, or correction
2. preserve source linkage immediately
3. write bundle-local raw spine record
4. write candidate or canonical memory
5. keep raw evidence reachable for later resume, atlas, and promotion flows
```

- [ ] **Step 3: Add a verification contract**

In `docs/verification/FEATURES.md`, add a new contract entry:

```md
## feature.raw_truth_spine

- Commands:
  - `memd remember`
  - `memd checkpoint`
  - `memd ingest`
  - `memd hook capture`
  - `memd hook spill --apply`
- Pass:
  - each path writes a source-linked raw spine record
  - server timeline keeps source metadata
  - bundle compiled events surface the raw spine
```

- [ ] **Step 4: Run the targeted verification commands**

Run:

```bash
cargo test -q -p memd-server store_item_records_source_linked_event -- --nocapture
cargo test -q -p memd-client raw_spine -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add docs/core/setup.md docs/core/architecture.md docs/verification/FEATURES.md
git commit -m "docs: lock raw truth spine contract"
```

## Self-Review

### Spec coverage

- `capture once` -> Tasks 2 and 3 add raw spine writes across all ingress paths.
- `keep raw evidence` -> Task 4 surfaces the raw spine in compiled artifacts.
- `never lose source linkage` -> Tasks 1 and 3 assert `source_system` and `source_path` survive both server and bundle layers.
- `all major harness inputs can enter one raw event spine` -> Task 3 covers `remember`, `checkpoint`, `ingest`, `hook capture`, and `hook spill`.

### Placeholder scan

- No `TODO`, `TBD`, or “implement later” placeholders remain.
- Every task contains exact file paths, code blocks, commands, and expected outcomes.

### Type consistency

- Bundle-local raw truth record uses one type: `RawSpineRecord`.
- Server-side source-linked event contract stays on existing `MemoryEventRecord`.
- Phase A does not invent a second canonical store or a new server transport path.

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-04-11-phase-a-raw-truth-spine.md`. Two execution options:

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**
