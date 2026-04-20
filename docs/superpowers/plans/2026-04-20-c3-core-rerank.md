# C3 Core Rerank Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the benchmark-moving intrinsic rerank slice of C3 to the current search path and rerun `ConvoMem` and `MemBench`.

**Architecture:** Keep B3 recall as-is, then rerank only the top candidate window locally inside `memd-server` using sidecar-style lexical and semantic heuristics blended with the existing base score. This is intentionally smaller than full C3 and avoids sidecar `/rerank` and embed-model migration work.

**Tech Stack:** Rust, axum, existing `memd-server` search route and test harness, public benchmark CLI

---

### Task 1: Add rerank tests first

**Files:**
- Modify: `crates/memd-server/src/tests/mod.rs`
- Verify: `cargo test -p memd-server rerank -- --nocapture`

- [ ] Add a failing unit/integration test proving rerank promotes the better semantic/phrase match over a weaker base-ranked candidate.
- [ ] Run the targeted test and confirm it fails for the expected reason.

### Task 2: Implement local rerank helper

**Files:**
- Modify: `crates/memd-server/src/routes.rs`

- [ ] Add a small local rerank scorer and candidate reorder helper.
- [ ] Gate it with a debug env flag that defaults to enabled.
- [ ] Blend rerank score with current base score for the top candidate window only.

### Task 3: Verify server behavior

**Files:**
- Modify: `crates/memd-server/src/tests/mod.rs` if needed

- [ ] Run the new rerank-focused tests.
- [ ] Run nearby search-route tests to catch regressions.

### Task 4: Refresh benches

**Files:**
- Update runtime artifacts under `.monitor/c3/` and `.memd/benchmarks/c3/` as produced by the CLI

- [ ] Run fresh `ConvoMem` benchmark on the current server.
- [ ] Run fresh `MemBench` benchmark on the current server.
- [ ] Record the resulting values and note whether improvements are real or just fresh state.

### Task 5: Close out

**Files:**
- No required file writes beyond code/tests/artifacts unless benchmark docs are updated by necessity

- [ ] Summarize what part of C3 is now done.
- [ ] Call out what remains out of scope for this slice.
