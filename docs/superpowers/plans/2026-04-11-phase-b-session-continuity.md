# Phase B Session Continuity Implementation Plan

> **For agentic workers:** use mini workers only. Execute task-by-task. Keep the pass gate strict: a fresh session must answer the continuity questions without transcript rebuild.

**Goal:** make session continuity explicit and compact so a new session can answer:

- what are we doing
- where did we leave off
- what changed
- what next

**Architecture:** reuse the existing resume stack in `memd-client` and retrieval APIs already present in `memd-server`. Do not invent a second continuity backend. Lift the implicit signals already present in `ResumeSnapshot` into an explicit continuity capsule that drives resume prompts, handoff prompts, bundle markdown, and verification.

**Tech Stack:** Rust, `memd-client`, `memd-server`, existing resume cache + bundle state + compiled bundle artifacts.

---

## File Map

**Modify**

- `crates/memd-client/src/runtime/resume/mod.rs`
- `crates/memd-client/src/runtime/resume/wakeup.rs`
- `crates/memd-client/src/render/render_summary.rs`
- `crates/memd-client/src/bundle/memory_surface.rs`
- `crates/memd-client/src/evaluation_runtime_tests/evaluation_runtime_tests_bundle_state.rs`
- `docs/core/architecture.md`
- `docs/verification/FEATURES.md`

---

## Task 1: Lock the explicit continuity contract

**Goal**

Turn the current implicit continuity signals into a direct contract.

**Do**

- add failing tests that require resume and handoff prompts to surface:
  - current task
  - resume point / where we left off
  - change summary / what changed
  - next action
- prefer `evaluation_runtime_tests_bundle_state.rs`
- keep assertions short and product-facing

**Pass**

- targeted continuity prompt tests pass

---

## Task 2: Add a continuity capsule to `ResumeSnapshot`

**Goal**

Give the runtime one explicit structured continuity view instead of scattered heuristics.

**Do**

- add helper methods and/or a small continuity struct in `runtime/resume/mod.rs`
- derive fields from existing sources:
  - working memory
  - inbox
  - rehydration queue
  - change summary
  - event spine
  - workspace lane
- avoid new network calls

**Pass**

- one runtime path can directly answer the 4 continuity questions

---

## Task 3: Make resume and handoff prompts continuity-first

**Goal**

Fresh sessions should read continuity answers first, not reconstruct them from mixed sections.

**Do**

- update `render_resume_prompt`
- update `render_handoff_prompt`
- keep prompt compact
- keep old signal sections only if still useful after the continuity block

**Pass**

- resume and handoff prompts start with continuity answers
- prompt still stays compact

---

## Task 4: Surface the same continuity block in bundle wake / memory pages

**Goal**

The visible bundle should match the prompt contract.

**Do**

- update wakeup and memory surfaces so the same continuity block is visible in bundle artifacts
- keep wording aligned with prompts

**Pass**

- fresh human inspection and fresh agent resume see the same continuity story

---

## Task 5: Document and verify the Phase B contract

**Do**

- add Phase B wording to `docs/core/architecture.md`
- add a new `FEATURE-SESSION-CONTINUITY` entry in `docs/verification/FEATURES.md`
- run narrow continuity tests

**Pass**

- docs reflect the contract
- narrow tests pass

---

## Pass Gate

- a fresh session can answer:
  - what are we doing
  - where did we leave off
  - what changed
  - what next
- without transcript rebuild

## Evidence

- resume prompt tests
- handoff prompt tests
- bundle wake / memory surface tests

## Fail Conditions

- continuity still depends on mixed heuristics only
- prompts still require reconstruction by reading multiple sections
- wake pages and prompt surfaces disagree
