---
phase: "49"
name: "v6 Composite Scoring and Acceptance Gates"
created: 2026-04-06
status: complete
---

# Phase 49: v6 Composite Scoring and Acceptance Gates — Summary

## Goal

Create a deterministic composite scorer that blends eval, scenario, coordination,
latency, and bloat into a single acceptance gate.

## What Changed

1. Added a new `memd composite` command and report model.
2. Reused the latest eval and scenario artifacts plus live coordination sampling.
3. Scored five explicit dimensions:
   - correctness
   - memory quality
   - coordination quality
   - latency
   - bloat
4. Added persisted composite artifacts under `composite/latest.*`.
5. Added tests that verify the composite report combines existing saved signals.

## Verification

- `cargo test -p memd-client run_composite_command`
- `cargo test -p memd-client`

## Result

`memd` now has a single explicit composite gate that can feed later experiment
runner work.
