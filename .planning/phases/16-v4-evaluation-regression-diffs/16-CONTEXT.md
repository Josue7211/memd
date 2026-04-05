# Phase 16: `v4` Evaluation Regression Diffs - Context

**Gathered:** 2026-04-04
**Status:** Completed from shipped repo state
**Mode:** Auto-generated from implementation

<domain>
## Phase Boundary

Saved evaluation snapshots need comparison logic or they become passive logs.
The next `v4` slice compares the current bundle evaluation against the latest
saved baseline and reports drift in the main memory-quality dimensions.

</domain>

<decisions>
## Implementation Decisions

### latest snapshot is the baseline

The first regression layer compares against `.memd/evals/latest.json` to keep
the flow local and deterministic.

### report changed dimensions explicitly

The operator surface should show score deltas and changed dimensions such as
working records, rehydration depth, inbox pressure, workspace lanes, and
semantic hits.

</decisions>

<code_context>
## Existing Code Insights

- `memd eval --write` already persists `latest.json` and timestamped snapshots.
- `BundleEvalResponse` is the right place to carry baseline and change data.
- `render_eval_summary` and markdown artifact rendering already provide compact
  operator-facing surfaces.

</code_context>

<specifics>
## Specific Ideas

- read `.memd/evals/latest.json` as baseline
- compute score delta
- compute changed dimensions
- include changes in summary and markdown output

</specifics>

<deferred>
## Deferred Ideas

- multi-snapshot comparisons
- automatic threshold alerts
- policy adjustments triggered by regressions

</deferred>
