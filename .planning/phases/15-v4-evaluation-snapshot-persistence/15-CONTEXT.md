# Phase 15: `v4` Evaluation Snapshot Persistence - Context

**Gathered:** 2026-04-04
**Status:** Completed from shipped repo state
**Mode:** Auto-generated from implementation

<domain>
## Phase Boundary

The evaluation harness should not disappear after one terminal run. Operators
need a simple local artifact trail so bundle memory quality can be compared over
time.

</domain>

<decisions>
## Implementation Decisions

### keep persistence local and cheap

The first slice writes markdown and JSON snapshots under the bundle itself.

### favor latest plus timestamped history

Operators get a stable `latest.*` pair and a timestamped history without
needing a database or background service.

</decisions>

<code_context>
## Existing Code Insights

- the new `memd eval` command already computes deterministic bundle quality
  scores from `read_bundle_resume`.
- bundle directories already own generated artifacts like memory files and
  agent profiles, so `evals/` fits the existing model.

</code_context>

<specifics>
## Specific Ideas

- add `memd eval --write`
- write `.memd/evals/latest.json`
- write `.memd/evals/latest.md`
- also write timestamped snapshots for history

</specifics>

<deferred>
## Deferred Ideas

- automatic diffing between snapshots
- threshold-triggered regressions
- scheduled evaluation runs

</deferred>
