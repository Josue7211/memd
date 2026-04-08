# Testing

## Overview

Testing is mostly crate-local Rust unit testing driven by `cargo test`. Coverage is strongest for deterministic helpers, scoring functions, storage edge cases, and Obsidian import/output behavior.

The major weakness is end-to-end proof of the core product contract:

- store a memory
- resume or retrieve later
- surface that memory prominently
- change later behavior because of that recall

That gap is directly relevant to the current debugging session.

## Primary Test Commands

- `cargo test`
- `cargo test -p memd-client --quiet`
- `cargo test -p memd-server --quiet`

Planning docs and summaries repeatedly reference focused commands such as:

- `cargo test -p memd-client`
- `cargo test -p memd-server`
- `cargo fmt --all -- --check`

The PR template at `.github/PULL_REQUEST_TEMPLATE.md` explicitly calls out `cargo test`.

## Where Tests Live

### Server tests

- `crates/memd-server/src/main.rs`
- `crates/memd-server/src/store.rs`
- `crates/memd-server/src/working.rs`

These cover:

- retrieval scoring and precedence
- workspace ranking
- selected `live_truth` ordering behavior
- sqlite store concurrency and migration behavior
- peer session identity behavior

Concrete examples:

- `live_truth_precedes_project_memory` in `crates/memd-server/src/main.rs`
- `active_recent_canonical_items_rank_above_stale_contested_items` in `crates/memd-server/src/working.rs`
- `concurrent_write_and_cross_workspace_reads_complete` in `crates/memd-server/src/store.rs`

### Client tests

- `crates/memd-client/src/main.rs`
- `crates/memd-client/src/obsidian.rs`

Observed emphasis:

- `obsidian.rs` has broad unit coverage for vault scanning, duplication control, annotation roundtrips, compiled-note paths, and handoff markdown generation.
- `main.rs` contains some targeted tests, but the current suite is much lighter than the size of the file suggests.

### Render-path coverage

- `crates/memd-client/src/render.rs` contains critical prompt rendering logic.
- There is no visible dedicated test block in the inspected region for the most important resume prompt layout guarantees.

This matters because prompt structure is part of the product behavior, not just presentation.

## What The Tests Prove Well

- DTO-heavy request/response flows compile and basic behaviors are stable.
- Ranking helpers and storage internals work in isolation.
- Obsidian import/export and compiled markdown helpers are relatively well protected.
- Some truth-first behavior exists at a narrow level, especially for repo-change `LiveTruth` ordering.

## What The Tests Do Not Prove Well

### 1. Durable memory recall in real agent workflows

There is no strong end-to-end test proving:

- a memory is ingested through the normal user-facing path
- a later resume/retrieval path surfaces it
- the surfaced memory displaces stale assumptions
- the next response changes accordingly

This is the central gap behind the current "memd does not remember" failure.

### 2. User correction durability

The planning/docs language discusses correction learning, but the inspected test surfaces do not show strong named tests for:

- automatic storage of user corrections
- precedence of corrected memory over stale prior memory
- repeated correction promotion into stable policy

There is repo-change `live_truth` coverage, but not equivalent proof for conversational corrections.

### 3. General memory ingest -> recall loop

There is a distinction between:

- storing memory records via routes and commands
- retrieving memory records
- meaningfully injecting them into resume context

The tests appear to validate pieces of that pipeline, not the whole loop.

### 4. Prompt-surface effectiveness

`render_resume_prompt` in `crates/memd-client/src/render.rs` is a high-leverage behavior surface, but the current mapping did not find matching prominent tests that guarantee critical memories appear early and clearly enough to guide the agent.

## Likely Testing Anti-Pattern

The repo has drifted toward "unit confidence without product confidence":

- storage tests prove records can exist
- ranking tests prove isolated precedence math
- snapshot/output tests prove formatting
- but no hard acceptance test proves `memd` actually remembers what matters in a later real task

## Recommended Missing Tests

These are the highest-value additions for the memory failure being debugged now:

- end-to-end test: `remember` a project fact, run resume/context, verify the fact appears in the hot path
- end-to-end test: inject a user correction, then verify the next resume/context prefers the correction over stale memory
- end-to-end test: store multiple memories, restart the retrieval path, and verify current-task recall remains stable
- render-path test: critical recalled memories appear before lower-priority background context in `render_resume_prompt`
- regression test: a planning/status artifact cannot outrank fresher contradictory memory without explicit provenance

## Practical Read For This Debugging Session

The current suite is good enough to keep refactors from instantly breaking the server and client internals.

It is not good enough to justify the product claim that `memd` functions as a reliable memory system / memory OS. The missing proof is not compile-time or unit-level; it is the absence of workflow tests for actual recall and behavior change.
