# Memory Audit

## Purpose

Track the `v1`-`v3` functionality audit for the core product claim:

- a memory can be ingested
- it persists durably
- later resume/retrieval surfaces it on the hot path
- it overrides stale assumptions when relevant
- later behavior changes because of that recall

This file records runtime findings, not planning optimism.

## Product-Truth Baseline

- `ROADMAP.md` already says `v1` through `v5` are still `in progress` in product-truth terms until end-to-end user behavior is verified.
- `.planning/REQUIREMENTS.md` marks `v1` requirements complete and `v2` mostly complete, but those statuses are not sufficient proof of runtime behavior.
- Current debugging standard:
  - planning completion is not proof of functioning memory
  - the audit bar is real recall under real hot-path usage

## Hot Path Traced

### Store path

- `crates/memd-client/src/main.rs`
  - `Commands::Remember`
  - `remember_with_bundle_defaults`
- `crates/memd-server/src/main.rs`
  - `/memory/store`
- `crates/memd-server/src/store.rs`
  - durable SQLite persistence

Observed:

- `memd remember` does issue a durable store request.
- after storing, the client immediately re-runs `read_bundle_resume` and rewrites bundle memory files.

### Resume / recall path

- `crates/memd-client/src/main.rs`
  - `read_bundle_resume`
- `crates/memd-server/src/main.rs`
  - `build_context`
- `crates/memd-server/src/working.rs`
  - `working_memory`
- `crates/memd-client/src/render.rs`
  - `render_resume_prompt`

Observed:

- resume first syncs repo-change live truth
- then requests compact context, working memory, inbox, workspace memory, and source memory
- then writes a derived `resume_state` record back into memory
- then renders a highly compressed prompt

## Findings So Far

### Finding 1: Storage exists, but recall is not guaranteed

- durable store behavior appears implemented
- the failure is more likely in retrieval priority and prompt surfacing than in raw persistence alone

Why this matters:

- a memory OS fails if stored memories do not reliably reach the next relevant prompt

### Finding 2: `current_task` retrieval is biased toward local and synced state

Files:

- `crates/memd-server/src/routing.rs`
- `crates/memd-client/src/main.rs`

Observed:

- `read_bundle_resume` defaults to `intent=current_task`
- `current_task` defaults to `route=local_first`
- route order is:
  - local
  - synced
  - project
  - global
- `intent_scope_bonus` for `current_task` gives:
  - local `1.1`
  - synced `0.95`
  - project `0.5`
  - global `-0.2`

Hypothesis:

- durable project memory is underweighted in the main resume path, especially when local live-truth or synced session-state items exist

### Finding 3: automatic ingest is strongest for repo changes, not general memory

Files:

- `crates/memd-client/src/main.rs`

Observed:

- `sync_recent_repo_live_truth` is a concrete automatic ingest path
- it captures repo changes into `LiveTruth`
- there is no equally strong automatic general-memory ingest path for conversationally important facts

Hypothesis:

- the system is better at remembering file edits than remembering user-important project facts

### Finding 4: resume writes derived session memory back into the store

Files:

- `crates/memd-client/src/main.rs`
  - `sync_resume_state_record`

Observed:

- every resume stores or repairs a `MemoryKind::Status` item tagged `resume_state` and `session_state`
- this derived memory is project or synced scoped
- it is written with high confidence and fresh verification timestamps

Hypothesis:

- derived session-state memory may compete with or crowd out durable operator-authored memory in the same hot path

### Finding 5: prompt rendering is extremely compact

Files:

- `crates/memd-client/src/render.rs`

Observed:

- prompt output prioritizes:
  - context budget
  - current task capsule
  - event spine / live truth
  - top working records
  - inbox / rehydration
- only a narrow slice of retrieved memory is shown explicitly

Hypothesis:

- even if a desired memory survives retrieval, it can still disappear from the visible prompt if it is not near the very top

### Finding 6: tests do not prove the real product contract

Files:

- `crates/memd-server/src/main.rs`
- `crates/memd-client/src/main.rs`
- `.planning/codebase/TESTING.md`

Observed:

- there is a narrow test proving `LiveTruth` can precede an older project fact
- there are tests proving resume publishes `resume_state`
- there is no strong end-to-end regression proving:
  - `memd remember` stores a durable project fact
  - later `resume` retrieves it
  - it remains visible in working memory or prompt output
  - it wins over fresher but less important session-state noise

### Finding 7: the project-fact crowd-out bug is now reproduced in code

Files:

- `crates/memd-server/src/main.rs`

Reproducer:

- added test:
  - `current_task_context_keeps_project_fact_visible_under_synced_noise`
- scenario:
  - one canonical project fact
  - five synced `resume_state` style status items
  - `current_task` + `local_first`
  - context limit `4`

Observed:

- the durable project fact is dropped from `build_context`
- the new test fails

Implication:

- the current retrieval contract is objectively wrong for the product promise
- this is no longer just a conversational hypothesis

## Audit Status By Version

### v1

Status: not verified

Open questions:

- do stored canonical project facts reliably appear in later resume output?
- does explain / repair functionality help real recall, or just state inspection?
- is compact context actually good enough for hot-path use when multiple memory classes compete?

### v2

Status: partially traced, not verified

Open questions:

- do trust, contradiction, and branch mechanics improve real recall in the hot path?
- are branchable beliefs and trust floors helping users, or are they mostly internal metadata?
- does reversible compression preserve enough important detail to matter in actual sessions?

### v3

Status: not verified

Open questions:

- do shared workspace and handoff memories get recalled in real multi-session use?
- does workspace-aware retrieval improve real recall, or is it hidden by local/synced bias?
- are shared-memory corrections and handoffs behavior-changing or just stored artifacts?

## Highest-Value Next Tests

1. Store a durable project fact with `memd remember`, then run `read_bundle_resume`, and assert that the fact appears in `working.records` or `context.records`.
2. Add repo live-truth and resume-state noise, then assert the remembered project fact still survives the `current_task` hot path.
3. Add a workspace-scoped shared memory, then assert `v3` retrieval keeps it visible when the active workspace matches.
4. Add a contradiction or supersede scenario, then assert stale memory loses in hot-path retrieval instead of only in inspection views.

## Current Root-Cause Hypothesis

`memd` is functioning more like:

- a memory database
- a derived resume-state generator
- a compact prompt builder

than like a fully closed-loop memory OS.

The most likely failure is:

- durable memories are stored
- but the `current_task` retrieval contract overweights local freshness and derived session state
- and the render path compresses the result so aggressively that durable memory often fails to influence later behavior

## Confirmed Root Cause And First Fix

Confirmed in code:

- `crates/memd-server/src/main.rs`
  - `build_context` previously:
    - grouped non-live-truth items by scope
    - sorted only inside each scope bucket
    - appended whole buckets in route order
    - truncated after concatenation

Effect:

- enough `Synced` noise could exclude `Project` memory before score-based ranking mattered

Fix applied:

- `build_context` now globally ranks all eligible non-live-truth items using `context_score`
- scope and intent still matter through the scoring function
- scope order no longer acts like a hard bucket wall for non-live-truth items

Verification:

- new regression test passes:
  - `current_task_context_keeps_project_fact_visible_under_synced_noise`
- additional regression test passes:
  - `current_task_context_prefers_matching_workspace_memory_under_cross_workspace_noise`
- full `cargo test -p memd-server --quiet` passes after the change

What this does **not** prove yet:

- full client-side `remember -> resume -> prompt -> behavior` correctness
- workspace/shared-memory recall under realistic mixed noise
- correction durability as a general product behavior

## Client-Side Recall Coverage Added

Confirmed in code:

- `crates/memd-client/src/main.rs`
  - the test harness now supports injected compact-context and working-memory responses

Verification:

- new client-side regression test passes:
  - `read_bundle_resume_keeps_recalled_project_fact_visible_in_bundle_memory`
- full `cargo test -p memd-client --quiet` passes after the harness extension

Meaning:

- when retrieval returns the right durable memory, `read_bundle_resume` and
  generated bundle memory output can carry that memory into the hot path

Remaining gap:

- we still need stronger proof that corrections and stale-belief overrides are
  ingested and retrieved correctly in realistic flows

## Correction Loop Findings

### Finding 8: low-level correction suppression works when driven explicitly

Confirmed in code:

- added regression test:
  - `superseded_memory_drops_out_after_manual_correction_loop`

Scenario:

- store stale active fact
- supersede it through audited repair
- store corrected active fact that references the stale item
- retrieve current-task context

Observed:

- corrected fact remains visible
- superseded stale belief is excluded from active current-task retrieval

Meaning:

- the lower-layer lifecycle model is capable of correction and stale-belief suppression
- the remaining product gap is likely the lack of a first-class, automatic, user-facing correction ingest path

### Finding 9: correction UX is still too manual

Observed:

- the CLI supports `repair` and repair modes including `supersede`
- but the normal hot path does not appear to convert user corrections into:
  - a supersede action on the stale belief
  - a new corrected durable memory

Likely implication:

- users experience “memd does not remember corrections” not because suppression is impossible, but because the product does not automatically execute the correction lifecycle from normal interaction
