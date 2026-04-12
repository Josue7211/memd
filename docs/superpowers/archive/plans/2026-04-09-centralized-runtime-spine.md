# Centralized Runtime Spine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship the first centralized `memd` runtime spine that improves retrieval quality, lowers token use, preserves live session continuity, raises memory quality, and makes coworking inspectable.

**Architecture:** Keep the core changes in the existing runtime path instead of adding parallel systems. Extend the bundle/session/task/awareness contracts in `crates/memd-client/src/main.rs`, `crates/memd-client/src/lib.rs`, `crates/memd-schema/src/lib.rs`, and `crates/memd-server/src/{main.rs,store.rs}` so truth, maintenance, continuity, coordination, tasks, and capabilities all read from the same persisted model. Add thin CLI surfaces over those contracts rather than creating command-local state.

**Tech Stack:** Rust, clap, tokio, serde/serde_json, existing memd awareness/hive/task helpers, existing sqlite-backed server store, existing `#[cfg(test)]` module in `crates/memd-client/src/main.rs`.

---

## File Structure

- `crates/memd-schema/src/lib.rs`
  - shared request/response types for maintenance receipts/reports, session lifecycle summaries, task classification summaries, and capability snapshots
- `crates/memd-client/src/lib.rs`
  - client methods for any new runtime endpoints
- `crates/memd-client/src/main.rs`
  - CLI argument structs
  - runtime orchestration
  - summary rendering
  - improvement/gap integration
  - focused and end-to-end tests
- `crates/memd-server/src/main.rs`
  - route wiring for new runtime endpoints
- `crates/memd-server/src/store.rs`
  - sqlite persistence and query helpers for new runtime contracts

## Task 1: Ship the maintenance spine

**Files:**
- Modify: `crates/memd-schema/src/lib.rs`
- Modify: `crates/memd-client/src/lib.rs`
- Modify: `crates/memd-client/src/main.rs`
- Modify: `crates/memd-server/src/main.rs`
- Modify: `crates/memd-server/src/store.rs`
- Test: `crates/memd-client/src/main.rs`
- Test: `crates/memd-server/src/store.rs`

**Goal:** Add a real `memd maintain` surface that can scan, compact, refresh, and repair memory while persisting receipts and reports.

- [ ] **Step 1: Write the failing client tests**

Add tests in `crates/memd-client/src/main.rs`:

```rust
    #[tokio::test]
    async fn run_maintain_command_persists_scan_report() {
        let dir = std::env::temp_dir().join(format!(
            "memd-maintain-scan-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).expect("create temp bundle");
        fs::write(
            dir.join("config.json"),
            r#"{"project":"demo","namespace":"main","agent":"codex","session":"session-a"}"#,
        )
        .expect("write config");

        let base_url = spawn_mock_memory_server().await;
        let report = run_maintain_command(
            &MaintainArgs {
                output: dir.clone(),
                mode: "scan".to_string(),
                apply: false,
                summary: false,
            },
            &base_url,
        )
        .await
        .expect("run maintain scan");

        assert_eq!(report.mode, "scan");
        assert!(report.receipt_id.is_some());
        assert!(report.findings.iter().any(|value| value.contains("memory")));
    }

    #[tokio::test]
    async fn render_maintain_summary_surfaces_receipt_and_counts() {
        let summary = render_maintain_summary(&MaintainReport {
            mode: "compact".to_string(),
            receipt_id: Some("receipt-1".to_string()),
            compacted_items: 3,
            refreshed_items: 0,
            repaired_items: 1,
            findings: vec!["compacted stale duplicates".to_string()],
            generated_at: Utc::now(),
        });
        assert!(summary.contains("maintain mode=compact"));
        assert!(summary.contains("receipt=receipt-1"));
        assert!(summary.contains("compacted=3"));
        assert!(summary.contains("repaired=1"));
    }
```

- [ ] **Step 2: Run the focused tests to verify they fail**

Run:

```bash
cargo test -p memd-client --bin memd run_maintain_command_persists_scan_report -- --exact
cargo test -p memd-client --bin memd render_maintain_summary_surfaces_receipt_and_counts -- --exact
```

Expected: FAIL because `MaintainArgs`, `run_maintain_command`, `MaintainReport`, and `render_maintain_summary` do not exist yet.

- [ ] **Step 3: Add the shared maintenance contract**

Add the shared types in `crates/memd-schema/src/lib.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintainReportRequest {
    pub mode: String,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub session: Option<String>,
    pub apply: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintainReport {
    pub mode: String,
    pub receipt_id: Option<String>,
    pub compacted_items: usize,
    pub refreshed_items: usize,
    pub repaired_items: usize,
    pub findings: Vec<String>,
    pub generated_at: DateTime<Utc>,
}
```

- [ ] **Step 4: Add the client/server/store path**

Implement:

- in `crates/memd-client/src/lib.rs`:

```rust
pub async fn maintain_runtime(
    &self,
    req: &MaintainReportRequest,
) -> anyhow::Result<MaintainReport> {
    self.post_json("/runtime/maintain", req).await
}
```

- in `crates/memd-server/src/main.rs`:

```rust
.route("/runtime/maintain", post(post_runtime_maintain))
```

- in `crates/memd-server/src/store.rs`, add a minimal persisted implementation that:
  - computes duplicate/stale signals from current memory state
  - returns a `MaintainReport`
  - does not mutate data unless `apply=true`

- [ ] **Step 5: Add the CLI surface**

Add in `crates/memd-client/src/main.rs`:

```rust
#[derive(Debug, Clone, Args)]
struct MaintainArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,
    #[arg(long, default_value = "scan")]
    mode: String,
    #[arg(long)]
    apply: bool,
    #[arg(long)]
    summary: bool,
}
```

Implement:

- `run_maintain_command(...)`
- `render_maintain_summary(...)`
- command dispatch branch for `memd maintain`

- [ ] **Step 6: Run the focused tests to verify they pass**

Run:

```bash
cargo test -p memd-client --bin memd run_maintain_command_persists_scan_report -- --exact
cargo test -p memd-client --bin memd render_maintain_summary_surfaces_receipt_and_counts -- --exact
```

Expected: PASS.

- [ ] **Step 7: Run the relevant suites**

Run:

```bash
cargo test -p memd-client --bin memd
cargo test -p memd-server
```

Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add crates/memd-schema/src/lib.rs crates/memd-client/src/lib.rs crates/memd-client/src/main.rs crates/memd-server/src/main.rs crates/memd-server/src/store.rs
git commit -m "feat: add runtime maintenance spine"
```

## Task 2: Make retrieval and memory truth compact and explicit

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Modify: `crates/memd-schema/src/lib.rs`
- Test: `crates/memd-client/src/main.rs`

**Goal:** Add explicit truth metadata to memory-facing runtime summaries and make retrieval prefer compact canonical state before raw fallback.

- [ ] **Step 1: Write the failing tests**

Add tests in `crates/memd-client/src/main.rs`:

```rust
    #[test]
    fn truth_summary_marks_stale_superseded_and_confident_items() {
        let summary = render_truth_summary(&[
            TruthRecordSummary {
                id: "a".to_string(),
                freshness: "fresh".to_string(),
                confidence: 0.92,
                superseded: false,
                contradicted: false,
                provenance: "resume".to_string(),
                compact_summary: "current task owns checkout flow".to_string(),
            },
            TruthRecordSummary {
                id: "b".to_string(),
                freshness: "stale".to_string(),
                confidence: 0.41,
                superseded: true,
                contradicted: false,
                provenance: "inbox".to_string(),
                compact_summary: "old checkout task state".to_string(),
            },
        ]);
        assert!(summary.contains("freshness=fresh"));
        assert!(summary.contains("superseded=true"));
        assert!(summary.contains("confidence=0.41"));
    }

    #[test]
    fn retrieval_prefers_compact_truth_before_raw_fallback() {
        let decision = choose_retrieval_tier(
            true,
            true,
            true,
            900,
        );
        assert_eq!(decision, RetrievalTier::CompactTruth);
    }
```

- [ ] **Step 2: Run the focused tests to verify they fail**

Run:

```bash
cargo test -p memd-client --bin memd truth_summary_marks_stale_superseded_and_confident_items -- --exact
cargo test -p memd-client --bin memd retrieval_prefers_compact_truth_before_raw_fallback -- --exact
```

Expected: FAIL because the truth summary structures and retrieval-tier decision helper do not exist yet.

- [ ] **Step 3: Add compact truth types and helpers**

Add in `crates/memd-client/src/main.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
enum RetrievalTier {
    HotSummary,
    CompactTruth,
    CanonicalEvidence,
    RawFallback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TruthRecordSummary {
    id: String,
    freshness: String,
    confidence: f32,
    superseded: bool,
    contradicted: bool,
    provenance: String,
    compact_summary: String,
}
```

Add:

- `choose_retrieval_tier(...)`
- `build_truth_record_summaries(...)`
- `render_truth_summary(...)`

- [ ] **Step 4: Thread truth summaries into existing surfaces**

Update:

- `read_bundle_status(...)`
- `render_status_summary(...)`
- any existing memory summary path already used by resume/status/gap

The rule is:

- prefer compact canonical summaries when available
- include freshness/provenance/confidence in rendered diagnostics
- only fall back to raw detail when compact truth is unavailable

- [ ] **Step 5: Run the focused tests to verify they pass**

Run:

```bash
cargo test -p memd-client --bin memd truth_summary_marks_stale_superseded_and_confident_items -- --exact
cargo test -p memd-client --bin memd retrieval_prefers_compact_truth_before_raw_fallback -- --exact
```

Expected: PASS.

- [ ] **Step 6: Run the client suite**

Run:

```bash
cargo test -p memd-client --bin memd
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/memd-client/src/main.rs crates/memd-schema/src/lib.rs
git commit -m "feat: make retrieval truth-first and compact"
```

## Task 3: Finish the continuity spine with explicit session surfaces

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Modify: `crates/memd-schema/src/lib.rs`
- Modify: `crates/memd-client/src/lib.rs`
- Modify: `crates/memd-server/src/main.rs`
- Modify: `crates/memd-server/src/store.rs`
- Test: `crates/memd-client/src/main.rs`
- Test: `crates/memd-server/src/store.rs`

**Goal:** Add a dedicated `memd session` surface for summary, rebind, retire, and reconcile so continuity is explicit and shared across commands.

- [ ] **Step 1: Write the failing tests**

Add tests in `crates/memd-client/src/main.rs`:

```rust
    #[tokio::test]
    async fn run_session_command_summary_surfaces_live_bundle_and_rebased_from() {
        let dir = std::env::temp_dir().join(format!("memd-session-summary-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp dir");
        fs::write(
            dir.join("config.json"),
            r#"{"project":"demo","agent":"codex","session":"stale-session"}"#,
        )
        .expect("write config");

        let summary = run_session_command(
            &SessionArgs {
                output: dir.clone(),
                action: "summary".to_string(),
                target_session: None,
                summary: false,
            },
            "http://127.0.0.1:8787",
        )
        .await
        .expect("run session summary");

        assert!(summary.live_session.is_some());
    }
```

- [ ] **Step 2: Run the focused test to verify it fails**

Run:

```bash
cargo test -p memd-client --bin memd run_session_command_summary_surfaces_live_bundle_and_rebased_from -- --exact
```

Expected: FAIL because `SessionArgs` and `run_session_command` do not exist yet.

- [ ] **Step 3: Add the session contract and client/server/store hooks**

Add shared session lifecycle types in `crates/memd-schema/src/lib.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLifecycleSummary {
    pub bundle_session: Option<String>,
    pub live_session: Option<String>,
    pub rebased_from: Option<String>,
    pub retired_sessions: Vec<String>,
}
```

Wire any required endpoint through:

- `crates/memd-client/src/lib.rs`
- `crates/memd-server/src/main.rs`
- `crates/memd-server/src/store.rs`

Reuse the existing retirement and hive-session persistence paths where possible.

- [ ] **Step 4: Add the CLI surface**

Add to `crates/memd-client/src/main.rs`:

```rust
#[derive(Debug, Clone, Args)]
struct SessionArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,
    #[arg(long, default_value = "summary")]
    action: String,
    #[arg(long)]
    target_session: Option<String>,
    #[arg(long)]
    summary: bool,
}
```

Implement:

- `run_session_command(...)`
- `render_session_summary(...)`
- dispatch branch for `memd session`

Support:

- `summary`
- `rebind`
- `retire`
- `reconcile`

- [ ] **Step 5: Run the focused test to verify it passes**

Run:

```bash
cargo test -p memd-client --bin memd run_session_command_summary_surfaces_live_bundle_and_rebased_from -- --exact
```

Expected: PASS.

- [ ] **Step 6: Run the relevant suites**

Run:

```bash
cargo test -p memd-client --bin memd
cargo test -p memd-server
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/memd-schema/src/lib.rs crates/memd-client/src/lib.rs crates/memd-client/src/main.rs crates/memd-server/src/main.rs crates/memd-server/src/store.rs
git commit -m "feat: add explicit session continuity surface"
```

## Task 4: Finish the coordination and task spine

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Modify: `crates/memd-schema/src/lib.rs`
- Test: `crates/memd-client/src/main.rs`

**Goal:** Make coworking and duplicate-work pressure explicit through task classification and improved coordination summaries.

- [ ] **Step 1: Write the failing tests**

Add tests in `crates/memd-client/src/main.rs`:

```rust
    #[test]
    fn classify_task_runtime_mode_distinguishes_workflow_review_help_and_maintenance() {
        assert_eq!(
            classify_task_runtime_mode("refresh docs", "maintenance sweep", "maintenance"),
            "maintenance"
        );
        assert_eq!(
            classify_task_runtime_mode("review patch", "shared review lane", "shared_review"),
            "review"
        );
    }

    #[test]
    fn render_tasks_summary_surfaces_duplicate_work_pressure() {
        let summary = render_tasks_summary(&TaskRuntimeSummary {
            total_tasks: 4,
            maintenance_tasks: 1,
            review_tasks: 1,
            help_tasks: 1,
            duplicate_pressure: 2,
        });
        assert!(summary.contains("duplicate_pressure=2"));
        assert!(summary.contains("maintenance=1"));
    }
```

- [ ] **Step 2: Run the focused tests to verify they fail**

Run:

```bash
cargo test -p memd-client --bin memd classify_task_runtime_mode_distinguishes_workflow_review_help_and_maintenance -- --exact
cargo test -p memd-client --bin memd render_tasks_summary_surfaces_duplicate_work_pressure -- --exact
```

Expected: FAIL because task classification helpers and summary types do not exist yet.

- [ ] **Step 3: Add task classification and summary helpers**

Add in `crates/memd-client/src/main.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TaskRuntimeSummary {
    total_tasks: usize,
    maintenance_tasks: usize,
    review_tasks: usize,
    help_tasks: usize,
    duplicate_pressure: usize,
}
```

Implement:

- `classify_task_runtime_mode(...)`
- `build_task_runtime_summary(...)`
- `render_tasks_summary(...)`

- [ ] **Step 4: Add or expand the `memd tasks` CLI surface**

Add a `TasksRuntimeArgs` path or extend the existing task surface so it supports:

- `summary`
- `list`
- `classify`

The summary should show:

- task counts by class
- duplicate-work pressure
- ownership shape

- [ ] **Step 5: Run the focused tests to verify they pass**

Run:

```bash
cargo test -p memd-client --bin memd classify_task_runtime_mode_distinguishes_workflow_review_help_and_maintenance -- --exact
cargo test -p memd-client --bin memd render_tasks_summary_surfaces_duplicate_work_pressure -- --exact
```

Expected: PASS.

- [ ] **Step 6: Run the client suite**

Run:

```bash
cargo test -p memd-client --bin memd
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/memd-client/src/main.rs crates/memd-schema/src/lib.rs
git commit -m "feat: add task runtime coordination surface"
```

## Task 5: Ship the capability runtime surface

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Test: `crates/memd-client/src/main.rs`

**Goal:** Let operators inspect what the runtime can do right now through a dedicated capability summary/list/search surface.

- [ ] **Step 1: Write the failing tests**

Add tests in `crates/memd-client/src/main.rs`:

```rust
    #[test]
    fn render_capabilities_summary_surfaces_runtime_and_hive_counts() {
        let summary = render_capabilities_summary(&CapabilityRuntimeSummary {
            total_capabilities: 5,
            runtime_capabilities: 3,
            hive_capabilities: 2,
            search_hits: 0,
        });
        assert!(summary.contains("capabilities total=5"));
        assert!(summary.contains("runtime=3"));
        assert!(summary.contains("hive=2"));
    }

    #[test]
    fn search_capability_runtime_summary_filters_by_term() {
        let hits = search_capability_runtime_summary(
            &["memory".to_string(), "coordination".to_string(), "repair".to_string()],
            "coord",
        );
        assert_eq!(hits, vec!["coordination".to_string()]);
    }
```

- [ ] **Step 2: Run the focused tests to verify they fail**

Run:

```bash
cargo test -p memd-client --bin memd render_capabilities_summary_surfaces_runtime_and_hive_counts -- --exact
cargo test -p memd-client --bin memd search_capability_runtime_summary_filters_by_term -- --exact
```

Expected: FAIL because capability runtime summary helpers do not exist yet.

- [ ] **Step 3: Add capability summary helpers**

Add in `crates/memd-client/src/main.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CapabilityRuntimeSummary {
    total_capabilities: usize,
    runtime_capabilities: usize,
    hive_capabilities: usize,
    search_hits: usize,
}
```

Implement:

- `build_capability_runtime_summary(...)`
- `render_capabilities_summary(...)`
- `search_capability_runtime_summary(...)`

- [ ] **Step 4: Add the `memd capabilities` surface**

Add a dedicated capability command with:

- `summary`
- `list`
- `search`

The command should build from existing capability and hive metadata instead of introducing a new store.

- [ ] **Step 5: Run the focused tests to verify they pass**

Run:

```bash
cargo test -p memd-client --bin memd render_capabilities_summary_surfaces_runtime_and_hive_counts -- --exact
cargo test -p memd-client --bin memd search_capability_runtime_summary_filters_by_term -- --exact
```

Expected: PASS.

- [ ] **Step 6: Run the client suite**

Run:

```bash
cargo test -p memd-client --bin memd
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/memd-client/src/main.rs
git commit -m "feat: add capability runtime surface"
```

## Task 6: Verify the full centralized runtime spine

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Test: `crates/memd-client/src/main.rs`

**Goal:** Prove the five runtime pillars work together as one compact, inspectable system.

- [ ] **Step 1: Add the end-to-end regression**

Add this test in `crates/memd-client/src/main.rs`:

```rust
    #[tokio::test]
    async fn centralized_runtime_spine_emits_compact_truthful_continuous_coworking_summary() {
        let dir = std::env::temp_dir().join(format!(
            "memd-central-runtime-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).expect("create temp dir");
        fs::write(
            dir.join("config.json"),
            r#"{"project":"demo","namespace":"main","agent":"codex","session":"session-a"}"#,
        )
        .expect("write config");

        let base_url = spawn_mock_memory_server().await;
        let _ = run_maintain_command(
            &MaintainArgs {
                output: dir.clone(),
                mode: "scan".to_string(),
                apply: false,
                summary: false,
            },
            &base_url,
        )
        .await
        .expect("run maintain");
        let session = run_session_command(
            &SessionArgs {
                output: dir.clone(),
                action: "summary".to_string(),
                target_session: None,
                summary: false,
            },
            &base_url,
        )
        .await
        .expect("run session summary");
        let awareness = read_project_awareness(&AwarenessArgs {
            output: dir.clone(),
            root: None,
            include_current: true,
            summary: false,
        })
        .await
        .expect("read awareness");

        let rendered = render_project_awareness_summary(&awareness);
        assert!(session.live_session.is_some() || session.bundle_session.is_some());
        assert!(rendered.contains("current_session"));
    }
```

- [ ] **Step 2: Run the focused regression first**

Run:

```bash
cargo test -p memd-client --bin memd centralized_runtime_spine_emits_compact_truthful_continuous_coworking_summary -- --exact
```

Expected: FAIL until the previous tasks are complete.

- [ ] **Step 3: Run the full suites**

Run:

```bash
cargo test -p memd-client --bin memd
cargo test -p memd-server
```

Expected: PASS.

- [ ] **Step 4: Run the real CLI checks**

Run:

```bash
cargo run -q -p memd-client --bin memd -- maintain --output .memd --mode scan --summary
cargo run -q -p memd-client --bin memd -- session --output .memd --action summary --summary
cargo run -q -p memd-client --bin memd -- awareness --output .memd --summary
```

Expected:

- maintenance output shows mode, receipt, and counts
- session output shows bundle/live/rebased state
- awareness output shows current, active, stale, and hidden noise cleanly

- [ ] **Step 5: Commit**

```bash
git add crates/memd-schema/src/lib.rs crates/memd-client/src/lib.rs crates/memd-client/src/main.rs crates/memd-server/src/main.rs crates/memd-server/src/store.rs
git commit -m "feat: land centralized runtime spine"
```

## Review Checklist

- maintenance is real, inspectable, and persisted
- retrieval prefers compact truth over raw fallback
- continuity is explicit and shared across surfaces
- coworking state shows ownership and duplicate-work pressure
- capabilities are inspectable from runtime state
- no new parallel truth model was introduced
- `memd-client` and `memd-server` suites pass
