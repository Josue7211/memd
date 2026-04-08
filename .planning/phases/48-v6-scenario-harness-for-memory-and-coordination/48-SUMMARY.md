---
phase: "48"
name: "v6 Scenario Harness for Memory and Coordination"
created: 2026-04-06
status: in_progress
---

# Phase 48: `v6` Scenario Harness for Memory and Coordination — Summary

## Goal

Add stable scenario checks for real memory and coordination workflows on top of the
existing bundle health scaffold.

## What Changed

1. Added explicit scenario handling to `run_scenario_command` and normalized
   scenario names.
2. Added workflow checks for:
   - resume recovery continuity
   - workspace retrieval posture
   - handoff readiness
   - stale-session recovery posture
   - coworking visibility and actionability
3. Updated scenario scoring to track `max_score` dynamically and reject unknown
   scenario names with a clear supported list.
4. Added scenario test coverage for supported workflows and unsupported input.

## Verification

- `cargo test -p memd-client` with new scenario tests
- scenario report artifacts still written to `scenarios/latest.json|latest.md`

## Result

In progress. The scenario command now supports named workflow targets, and the
remaining work is to tighten the score model around richer real-workflow data.
