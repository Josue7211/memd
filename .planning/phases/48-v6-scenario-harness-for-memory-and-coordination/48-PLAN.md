---
phase: "48"
name: "v6 Scenario Harness for Memory and Coordination"
created: 2026-04-06
type: plan
status: in_progress
---

# Phase 48: `v6` Scenario Harness for Memory and Coordination — Plan

## Goal

Add named, reproducible scenario runs for high-value memd workflows so the system
can compare baseline versus candidate behavior with stable scoring and artifacts.

## Plan

1. Extend `memd scenario` to accept only supported, replayable scenario names.
2. Add workflow-specific checks for:
   - `resume_after_pause`
   - `handoff`
   - `workspace_retrieval`
   - `stale_session_recovery`
   - `coworking`
3. Preserve existing bundle-health behavior for compatibility.
4. Write test coverage for scenario routing and unsupported names.
5. Capture phase-level verification that scenario artifacts are produced and
   actionable findings are present.

## Verification

- `cargo fmt --all`
- `cargo test -p memd-client --test ???` (or full crate test target)
- `memd scenario --scenario <workflow>` completes and writes scenario artifacts

## Result

Scenario execution becomes workflow-aware, with explicit failure modes and stable
scenario names.
