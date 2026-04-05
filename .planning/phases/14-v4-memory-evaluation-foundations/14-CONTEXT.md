# Phase 14: `v4` Memory Evaluation Foundations - Context

**Gathered:** 2026-04-04
**Status:** Completed from shipped repo state
**Mode:** Auto-generated from strategic roadmap and implementation

<domain>
## Phase Boundary

The first `v4` slice should evaluate the real bundle-backed memory loop instead
of inventing abstract policy metrics. The harness must score the actual resume
path and surface its weak points compactly for operators.

</domain>

<decisions>
## Implementation Decisions

### evaluate the real startup path

The evaluation command reuses `read_bundle_resume` so it measures the same path
that Codex and other agents actually consume.

### keep the first score deterministic

The first harness uses bounded heuristic scoring over working records, context,
rehydration, inbox pressure, workspace coverage, and semantic recall instead of
jumping directly to adaptive policy tuning.

</decisions>

<code_context>
## Existing Code Insights

- `crates/memd-client/src/main.rs` already owns bundle resume, handoff, status,
  and agent profile flows.
- bundle startup quality can be evaluated without changing server contracts by
  reusing the existing client snapshot model.
- `crates/memd-client/src/render.rs` already has compact summary renderers that
  match the intended operator-facing CLI style.

</code_context>

<specifics>
## Specific Ideas

- add `memd eval --output .memd`
- return JSON and summary output
- score the real bundle resume path
- emit compact findings operators can act on

</specifics>

<deferred>
## Deferred Ideas

- learned scoring and policy tuning
- time-series evaluation history
- A/B policy comparison
- automatic regression baselines

</deferred>
