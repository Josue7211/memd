---
phase: "49"
name: "v6 Composite Scoring and Acceptance Gates"
created: 2026-04-06
type: plan
status: in_progress
---

# Phase 49: `v6` Composite Scoring and Acceptance Gates — Plan

## Goal

Add a deterministic composite scorer that merges correctness, memory quality,
coordination quality, latency, and bloat into a single gateable report.

## Plan

1. Add a new composite scoring command and report model.
2. Pull in the latest eval, scenario, and coordination signals.
3. Compute explicit per-dimension scores and an overall weighted score.
4. Mark hard correctness failures distinctly from quality degradations.
5. Persist composite artifacts and add tests for the scoring path.

## Verification

- `cargo test -p memd-client`
- composite artifacts write to `composite/latest.json` and `composite/latest.md`

## Result

The repo gets one explicit acceptance gate instead of multiple disconnected
quality readings.
