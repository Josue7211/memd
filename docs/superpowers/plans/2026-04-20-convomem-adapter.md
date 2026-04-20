# ConvoMem Adapter Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rewrite the `ConvoMem` retrieval adapter so message-level evidence ids are normalized, retrieved, and scored honestly.

**Architecture:** Add stable per-message ids during `ConvoMem` normalization, derive `message_evidence_ids` from the same conversation payload, then swap retrieval docs and expected-target lookup to use those ids.

**Tech Stack:** Rust, `serde_json`, existing `public_benchmark.rs` normalization and runtime tests

---

### Task 1: Add failing tests

**Files:**
- Modify: `crates/memd-client/src/main_tests/public_benchmark_tests/mod.rs`

- [ ] Add a normalization test that expects `message_evidence_ids` to be populated from evidence objects.
- [ ] Add a retrieval-report test that proves `ConvoMem` can score a hit when the gold evidence message is present.

### Task 2: Implement normalization helpers

**Files:**
- Modify: `crates/memd-client/src/benchmark/public_benchmark.rs`

- [ ] Add stable message-id helpers for `ConvoMem`.
- [ ] Derive `message_evidence_ids` during normalization.

### Task 3: Switch retrieval contract

**Files:**
- Modify: `crates/memd-client/src/benchmark/public_benchmark.rs`

- [ ] Make `ConvoMem` retrieval docs emit message-level docs keyed by stable ids.
- [ ] Make `ConvoMem` expected targets use `message_evidence_ids`.

### Task 4: Verify and rerun

**Files:**
- Runtime artifacts only

- [ ] Run focused `memd-client` public benchmark tests.
- [ ] Rerun fresh `ConvoMem` benchmark on the updated server.
