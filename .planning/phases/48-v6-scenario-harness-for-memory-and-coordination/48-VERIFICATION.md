---
phase: "48"
name: "v6 Scenario Harness for Memory and Coordination"
created: 2026-04-06
status: pending
---

# Phase 48: `v6` Scenario Harness for Memory and Coordination — Verification

## Goal-Backward Verification

**Phase Goal:** add stable scenario harnesses for resume, handoff, workspace retrieval,
stale-session recovery, and coworking workflows.

## Checks

| # | Requirement | Status | Evidence |
|---|------------|--------|----------|
| 1 | Unknown scenarios are rejected clearly | pass | scenario parse rejects unsupported values and prints supported list |
| 2 | New scenario workflows execute and emit checks | pass | each workflow name returns non-empty checks and score budget |
| 3 | Bundle-health path still works | pass | existing `bundle_health` assertions still hold |
| 4 | Scenario artifacts are still produced | pass | report artifacts written to `scenarios/latest.*` |

## Result

pending
