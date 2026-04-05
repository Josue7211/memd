---
phase: 12-v3-workspace-policy-corrections
type: context
status: ready
---

## Goal

Let operators correct workspace and visibility mistakes through the existing
repair path so shared-memory policy changes stay explicit and auditable.

## Current Boundary

- Shared workspace lanes and handoff bundles already exist.
- Repair is already the explicit path for metadata corrections.
- Workspace and visibility changes should not require bypassing memory history.

## Dependencies

- phase 10 shared workspace foundations are complete
- phase 11 workspace handoff bundles are complete
- repair, explain, and inbox surfaces already exist
