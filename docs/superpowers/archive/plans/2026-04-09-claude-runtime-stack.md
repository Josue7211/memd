# Claude Code-Inspired Runtime Stack Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the Claude Code source extraction into the first shipped `memd` runtime stack by making live coordination visible, memory truth explicit, and session continuity reliable.

**Architecture:** Keep the work inside `crates/memd-client/src/main.rs` and its existing test module so the runtime, summary rendering, and session bootstrap stay in one control surface. Build the stack in the same order as the roadmap: coordination surface first, truth metadata second, continuity overlay third. Reuse the current awareness, resume, and hive helpers instead of introducing a parallel state model.

**Tech Stack:** Rust, tokio, serde_json, existing memd awareness/resume/hive helpers, existing `#[cfg(test)]` module in `crates/memd-client/src/main.rs`.

---

## Phase Mapping

- **Phase 59:** Live coordination surface
- **Phase 60:** Truth-first memory model
- **Phase 61:** Session continuity overlay

## File Structure

- `crates/memd-client/src/main.rs`
  - render the live session map
  - classify current, active, stale, dead, and shared rows
  - carry truth metadata through awareness and resume paths
  - overlay the live session identity onto repo-local bundles
  - add the regression tests in the existing test module

## Tasks

### Task 1: Make live coordination the primary summary surface

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Test: `crates/memd-client/src/main.rs`

**Goal:** The summary output should make current, active, stale, dead, and shared session state obvious without making the user inspect raw rows.

- [ ] **Step 1: Write the failing tests**

Extend the existing awareness summary tests in `crates/memd-client/src/main.rs` so they prove the summary is grouped and labeled instead of dumped as a flat list:

- `project_awareness_summary_hides_dead_remote_rows_by_default`
- `project_awareness_summary_calls_out_stale_remote_sessions`
- `project_awareness_summary_marks_current_and_active_hive_sessions`
- new `project_awareness_summary_groups_sessions_into_current_active_stale_dead_sections`

The new grouped-state test should assert that the summary starts with the compact counts and then emits separate current, active hive, stale, and dead sections using the existing `ProjectAwarenessResponse` and `ProjectAwarenessEntry` model.

- [ ] **Step 2: Run the focused test and confirm it fails**

Run: `cargo test -p memd-client --bin memd project_awareness_summary_groups_sessions_into_current_active_stale_dead_sections -- --exact`

Expected: FAIL because the summary still renders with only the older row-style grouping.

- [ ] **Step 3: Implement the summary grouping**

Update `render_project_awareness_summary` so it:

- groups entries into current, active hive sessions, stale sessions, dead sessions, and shared sessions
- hides dead remote rows by default in the summary view
- emits a short diagnostic line when stale or dead rows are suppressed
- labels the current session explicitly so it cannot be mistaken for other live rows

The output should still include the existing root, collision, and session identifiers, but the first thing the user sees must be the grouped state.

- [ ] **Step 4: Run the focused test and confirm it passes**

Run: `cargo test -p memd-client --bin memd project_awareness_summary_groups_sessions_into_current_active_stale_dead_sections -- --exact`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/memd-client/src/main.rs
git commit -m "feat: surface live coordination in awareness summary"
```

### Task 2: Make truth metadata drive the visible memory model

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Test: `crates/memd-client/src/main.rs`

**Goal:** The awareness and resume paths should carry freshness and supersession state so the summary can distinguish verified truth from stale or replaced state.

- [ ] **Step 1: Write the failing tests**

Extend the existing merge and summary tests so they prove freshness and supersession are visible from current awareness state instead of inferred from row position:

- `awareness_merge_prefers_fresher_local_session_metadata_over_stale_remote_row`
- `awareness_merge_keeps_distinct_sessions_when_remote_rows_are_not_duplicates`
- new `project_awareness_summary_marks_freshness_and_supersession_from_last_updated`

The new test should assert that stale rows are called out using the existing `presence`, `last_updated`, and suppression diagnostics rather than a second truth store.

- [ ] **Step 2: Run the focused test and confirm it fails**

Run: `cargo test -p memd-client --bin memd project_awareness_summary_marks_freshness_and_supersession_from_last_updated -- --exact`

Expected: FAIL because the current summary does not yet render freshness and supersession as first-class summary signals.

- [ ] **Step 3: Implement truth-aware summary state**

Update the awareness/resume data flow so the summary renderer has explicit truth fields to work with instead of inferring everything from row presence:

- compute freshness from `last_updated`, `presence`, and the current bundle identity
- preserve superseded-row suppression in the summary header
- surface freshness markers in the row labels or diagnostics when a session is stale
- keep the raw row data available for drilldown even when the summary is compact

This task should reuse the existing `read_project_awareness`, `read_project_awareness_shared`, `read_bundle_resume`, and suppression helpers instead of adding a second truth store.

- [ ] **Step 4: Run the focused test and confirm it passes**

Run: `cargo test -p memd-client --bin memd project_awareness_summary_marks_freshness_and_supersession_from_last_updated -- --exact`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/memd-client/src/main.rs
git commit -m "feat: make awareness truth-first"
```

### Task 3: Make repo-local bundles follow the live session identity

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Test: `crates/memd-client/src/main.rs`

**Goal:** A restarted or rebound session should carry the live identity from `~/.memd` into the repo-local bundle so the project never keeps publishing an old session as current.

- [ ] **Step 1: Write the failing tests**

Add tests that prove the repo-local bundle follows the live session identity:

- `project_awareness_includes_current_bundle_when_session_exists`
- `run_hive_command_rebinds_repo_bundle_to_live_session_identity`
- new `project_awareness_summary_follows_live_session_after_rebind`

The new rebind test should use the real `HiveArgs` shape:

- `project_root`
- `output`
- `base_url`
- `hive_group`
- `publish_heartbeat`

and it should assert that the `HiveWireResponse` and awareness summary both reflect the live `session` instead of the stale one.

- [ ] **Step 2: Run the focused test and confirm it fails**

Run: `cargo test -p memd-client --bin memd run_hive_command_rebinds_repo_bundle_to_live_session_identity -- --exact`

Expected: FAIL because repo-local bundle state still needs the live-session overlay.

- [ ] **Step 3: Implement the live-session overlay**

Update `run_hive_command`, `build_hive_heartbeat`, `propagate_hive_metadata_to_active_project_bundles`, and the bundle/session bootstrap path so the repo-local `.memd` state is overwritten with the live session identity when the global bundle already has the current session:

- prefer the live session from `~/.memd` when both exist
- propagate the live identity into the repo-local state before heartbeat publish
- keep stale session rows visible as stale rather than current
- preserve the current work state so continuity is not lost during the rebind

The repo-local bundle should now follow the live session instead of freezing an old one.

- [ ] **Step 4: Run the focused test and confirm it passes**

Run: `cargo test -p memd-client --bin memd run_hive_command_rebinds_repo_bundle_to_live_session_identity -- --exact`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/memd-client/src/main.rs
git commit -m "feat: follow live session identity in repo bundles"
```

### Task 4: Verify the full stack together

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Test: `crates/memd-client/src/main.rs`

**Goal:** Prove the three phases work together in the same runtime: coordination is visible, truth is explicit, and continuity follows the live session.

- [ ] **Step 1: Add an end-to-end regression test**

Add one end-to-end test named `claude_runtime_stack_emits_coordinated_truthful_continuous_summary` that:

- runs `run_hive_command` with the real `HiveArgs`
- renders the project awareness summary from the current bundle state
- asserts the summary includes:
  - `current_session`
  - `active_hive_sessions`
  - `stale_remote_sessions` or `stale_sessions`
  - `hidden_remote_dead`
  - `hidden_superseded_stale`
  - the current live session identity from the overlay path

- [ ] **Step 2: Run the end-to-end test and confirm it fails first**

Run: `cargo test -p memd-client --bin memd claude_runtime_stack_emits_coordinated_truthful_continuous_summary -- --exact`

Expected: FAIL until the three tasks above are implemented together.

- [ ] **Step 3: Run the full focused suite**

Run: `cargo test -p memd-client --bin memd`

Expected: PASS.

- [ ] **Step 4: Capture the live CLI check**

Run:

```bash
cargo run -q -p memd-client --bin memd -- awareness --output .memd --summary
```

Expected:

- current session is clearly labeled
- dead rows are suppressed from the summary
- stale rows are called out explicitly
- the live session identity matches the current runtime

- [ ] **Step 5: Commit**

```bash
git add crates/memd-client/src/main.rs
git commit -m "feat: land claude runtime stack"
```

## Review Checklist

- live coordination is surfaced first, not buried in row dumps
- truth metadata is explicit and drives the summary model
- repo-local bundle identity follows the live session after restart or rebind
- the full `memd-client` suite passes after the changes
- no new top-level files are needed beyond this plan and the runtime changes
