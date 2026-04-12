# Ceiling memd Live Truth Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn `memd` from a refresh-oriented memory helper into a truth-first external cortex that updates immediately after repo-local edits and user corrections, suppresses stale beliefs, and never mutates shared runtimes during normal memory operations.

**Architecture:** Add a first-class `live_truth` substrate with strict precedence over older working memory, feed it from explicit local edit/correction events, compile noisy raw events into compact truth items, and inject that lane into every resume/reload/prompt surface. Keep runtime mutation out of the default path by splitting normal memory operations from explicit repair/install actions.

**Tech Stack:** Rust, `memd-client`, `memd-server`, `memd-schema`, existing bundle resume/render paths, git-based repo inspection, current memory store/query APIs.

---

## File Structure

**Core live-truth substrate**
- Modify: `crates/memd-schema/src/lib.rs`
  - Add live-truth item shape, event shape, retrieval request fields, precedence metadata.
- Modify: `crates/memd-server/src/store.rs`
  - Persist live-truth items and event-derived truth records.
- Modify: `crates/memd-server/src/working.rs`
  - Retrieve live-truth items ahead of working memory and apply stale-belief suppression.
- Modify: `crates/memd-server/src/main.rs`
  - Wire new API routes/handlers if needed.

**Client ingestion and rendering**
- Modify: `crates/memd-client/src/main.rs`
  - Emit repo-local edit/correction events, compile them into truth items, request the lane during resume/reload, and render top-of-context truth.
- Modify: `crates/memd-client/src/render.rs`
  - Surface live truth prominently in resume prompts.
- Modify: `crates/memd-client/src/commands.rs`
  - Add explicit commands for truth ingestion/inspection if current CLI parsing gets too large.
- Modify: `crates/memd-client/src/obsidian.rs`
  - Keep fixture/test snapshots compatible with the new resume shape.

**Policy and docs**
- Modify: `.planning/ROADMAP.md`
- Modify: `.planning/STATE.md`
- Modify: `docs/core/architecture.md`
- Create: `docs/strategy/live-truth.md`

**Tests**
- Modify: existing unit tests in `crates/memd-client/src/main.rs`
- Modify: existing tests in `crates/memd-client/src/obsidian.rs`
- Create or modify: server tests in `crates/memd-server/src/store.rs`
- Create or modify: server tests in `crates/memd-server/src/working.rs`

## Task 1: Lock the Product Invariants

**Files:**
- Modify: `.planning/ROADMAP.md`
- Modify: `.planning/STATE.md`
- Modify: `docs/core/architecture.md`
- Create: `docs/strategy/live-truth.md`

- [ ] **Step 1: Write the invariant section in `docs/strategy/live-truth.md`**

Document these exact rules:

```md
# Live Truth

## Hard Invariants

- memd is allowed to observe, summarize, compile, and learn during normal operation.
- memd is not allowed to mutate shared runtime state during `init`, `reload`, `resume`, `refresh`, or normal memory maintenance.
- shared runtime mutation is only allowed in explicit repair/install commands.
- the freshest verified local truth must outrank older working memory and older durable memory.
- raw events are not prompt material; they must be compiled into compact truth items first.
- project-local live truth is the default; cross-project promotion requires explicit validation.
```

- [ ] **Step 2: Add the ceiling phases to the roadmap**

Add these phase headings under the current v6 section:

```md
#### Phase 53: `v6` Live Truth Substrate and Precedence
#### Phase 54: `v6` Event-Driven Repo Edit and Correction Ingestion
#### Phase 55: `v6` Truth Compiler and Stale-Belief Suppression
#### Phase 56: `v6` Cross-Harness Expansion and Safe Self-Evolution Gates
```

- [ ] **Step 3: Update `.planning/STATE.md` with the new north star**

Insert a concise state note:

```md
- the next ceiling target is a truth-first live memory substrate: event-driven local truth, stale-belief suppression, and zero shared-runtime mutation during normal memory operations.
```

- [ ] **Step 4: Verify docs are saved**

Run:

```bash
test -f docs/strategy/live-truth.md && test -f docs/core/architecture.md && test -f .planning/ROADMAP.md && test -f .planning/STATE.md
```

Expected: command exits `0`

## Task 2: Introduce First-Class Live Truth Types

**Files:**
- Modify: `crates/memd-schema/src/lib.rs`
- Modify: `crates/memd-server/src/store.rs`
- Test: server unit tests in `crates/memd-server/src/store.rs`

- [ ] **Step 1: Add failing schema/storage tests for live truth**

Add tests covering:

```rust
#[test]
fn stores_live_truth_item_with_precedence_metadata() {}

#[test]
fn rejects_runtime_mutation_flags_on_normal_live_truth_ingest() {}
```

- [ ] **Step 2: Add schema types**

Define concrete types like:

```rust
pub enum MemoryLane {
    LiveTruth,
    Working,
    Durable,
}

pub enum LiveTruthKind {
    RepoEdit,
    UserCorrection,
    CommandOutcome,
}

pub struct LiveTruthItem {
    pub id: Uuid,
    pub project: Option<String>,
    pub workspace: Option<String>,
    pub kind: LiveTruthKind,
    pub summary: String,
    pub evidence_paths: Vec<String>,
    pub confidence: f32,
    pub recorded_at: DateTime<Utc>,
    pub supersedes_keys: Vec<String>,
}
```

- [ ] **Step 3: Add minimal server persistence**

Implement storage path in `store.rs` that can accept a live truth item through the existing store substrate rather than inventing a second database.

- [ ] **Step 4: Run server tests**

Run:

```bash
cargo test -p memd-server store_live_truth --quiet
```

Expected: targeted live-truth storage tests pass

## Task 3: Make Retrieval Truth-First

**Files:**
- Modify: `crates/memd-server/src/working.rs`
- Modify: `crates/memd-schema/src/lib.rs`
- Test: `crates/memd-server/src/working.rs`

- [ ] **Step 1: Write failing retrieval-order tests**

Add tests covering:

```rust
#[test]
fn live_truth_precedes_older_working_memory() {}

#[test]
fn conflicting_older_memory_is_suppressed_when_fresh_truth_exists() {}
```

- [ ] **Step 2: Extend retrieval request/response shapes**

Add fields that let the client request live truth explicitly and inspect whether suppression occurred.

- [ ] **Step 3: Implement truth-first merge**

Use this merge rule:

```rust
// precedence order
live_truth -> working -> inbox/workspace -> durable -> semantic
```

If a truth item supersedes a lower-priority record, omit the older record from prompt-facing output.

- [ ] **Step 4: Run retrieval tests**

Run:

```bash
cargo test -p memd-server working_live_truth --quiet
```

Expected: retrieval precedence tests pass

## Task 4: Add Event-Driven Repo Edit and Correction Ingestion

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Modify: `crates/memd-client/src/commands.rs`
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write failing client tests**

Add tests for:

```rust
#[test]
fn compiles_repo_edit_event_from_git_diff_summary() {}

#[test]
fn compiles_user_correction_event_into_live_truth_candidate() {}
```

- [ ] **Step 2: Add a bounded event collector**

Implement a collector that reads:

```text
git status --short
git diff --stat --compact-summary
```

and converts that into raw repo edit events for the current project root.

- [ ] **Step 3: Add correction-ingest entrypoint**

Add a client path that can store a user correction as a high-priority live truth item instead of leaving it as transient conversation state.

- [ ] **Step 4: Keep the path safe**

Do not call:

```text
apply_capability_bridges()
~/.local/bin writes
shared skill link writes
PATH edits
```

from the ingestion path.

- [ ] **Step 5: Run client tests**

Run:

```bash
cargo test -p memd-client live_truth_ingest --quiet
```

Expected: repo-edit and correction-ingest tests pass

## Task 5: Build the Truth Compiler

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Modify: `crates/memd-server/src/store.rs`
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write failing compiler tests**

Add tests covering:

```rust
#[test]
fn collapses_multiple_same_file_edits_into_one_truth_item() {}

#[test]
fn keeps_latest_verified_meaning_instead_of_edit_churn() {}
```

- [ ] **Step 2: Implement compiler rules**

Rules:

```text
- group by file and intent
- keep latest verified summary
- cap output size aggressively
- preserve evidence paths
- never emit raw patch text into the hot lane
```

- [ ] **Step 3: Store compiled truth instead of raw event noise**

Only compiled truth items should be visible to resume/reload/prompt rendering.

- [ ] **Step 4: Run compiler tests**

Run:

```bash
cargo test -p memd-client truth_compiler --quiet
```

Expected: truth compiler tests pass

## Task 6: Inject Live Truth into Resume, Reload, and Prompt Surfaces

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Modify: `crates/memd-client/src/render.rs`
- Modify: `crates/memd-client/src/obsidian.rs`
- Test: `crates/memd-client/src/main.rs`
- Test: `crates/memd-client/src/obsidian.rs`

- [ ] **Step 1: Write failing render tests**

Add tests covering:

```rust
#[test]
fn resume_prompt_surfaces_live_truth_before_since_last_resume() {}

#[test]
fn bundle_memory_markdown_shows_live_truth_section() {}
```

- [ ] **Step 2: Extend `ResumeSnapshot`**

Add explicit fields:

```rust
live_truth: Vec<String>,
suppressed_items: Vec<String>,
```

Keep `change_summary` separate from `live_truth`.

- [ ] **Step 3: Render the lane at the top**

Target order:

```text
Context Budget
Live Truth
Current Task Snapshot
Since Last Resume
Working Memory
...
```

- [ ] **Step 4: Update fixture snapshots**

Patch `obsidian.rs` and any test snapshot constructors so the new fields are explicit.

- [ ] **Step 5: Run render tests**

Run:

```bash
cargo test -p memd-client resume_prompt_surfaces_live_truth_before_since_last_resume --quiet
cargo test -p memd-client bundle_memory_markdown_shows_live_truth_section --quiet
```

Expected: prompt and markdown tests pass

## Task 7: Add Safe Self-Evolution Gates

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Modify: `crates/memd-server/src/working.rs`
- Modify: `docs/strategy/live-truth.md`
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write failing policy-gate tests**

Add tests:

```rust
#[test]
fn repeated_validated_corrections_promote_to_project_policy() {}

#[test]
fn single_session_noise_does_not_promote_to_policy() {}
```

- [ ] **Step 2: Add promotion gates**

Promotion requirements:

```text
- repeated evidence
- validation outcome
- provenance retained
- rollback possible
```

- [ ] **Step 3: Keep runtime mutation outside the loop**

Document and enforce:

```text
self-evolution may update policy and memory layers
self-evolution may not mutate shared runtimes or install surfaces
```

- [ ] **Step 4: Run policy tests**

Run:

```bash
cargo test -p memd-client policy_promotion --quiet
```

Expected: promotion tests pass

## Task 8: Full Verification

**Files:**
- Modify as needed: touched files above only

- [ ] **Step 1: Run the full client suite**

Run:

```bash
cargo test -p memd-client --quiet
```

Expected: all tests pass

- [ ] **Step 2: Run the full server suite**

Run:

```bash
cargo test -p memd-server --quiet
```

Expected: all tests pass

- [ ] **Step 3: Verify roadmap health**

Run:

```bash
node ~/.codex/get-shit-done/bin/gsd-tools.cjs validate health
```

Expected: `healthy`

- [ ] **Step 4: Manual product verification**

Verify this sequence works:

```text
1. edit a repo file
2. trigger memd-backed resume/reload surface
3. confirm live truth names the change without reopening the file
4. inject a user correction
5. confirm the next prompt uses the correction instead of the stale belief
6. confirm no shared runtime files were modified as a side effect
```

## Self-Review

**Spec coverage**
- Covers live truth substrate, truth-first retrieval, repo-edit ingestion, correction ingestion, truth compilation, stale-belief suppression, self-evolution gates, and runtime non-interference.

**Placeholder scan**
- No `TODO`, `TBD`, or “handle appropriately” placeholders remain.

**Type consistency**
- Uses one naming set throughout:
  - `live_truth`
  - `LiveTruthItem`
  - `LiveTruthKind`
  - `suppressed_items`

