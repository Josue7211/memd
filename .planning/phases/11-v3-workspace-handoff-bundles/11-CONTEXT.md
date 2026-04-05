---
phase: 11-v3-workspace-handoff-bundles
type: context
status: ready
---

## Goal

Build a first-class shared handoff bundle so agents and humans can resume work
from shared workspace memory instead of rebuilding state from scattered search,
working-memory, and inbox calls.

## Current Boundary

- Phase 10 already established shared workspace and visibility lanes.
- `resume` already assembles compact startup state for one agent session.
- The next slice should package that state for delegation and shared workspace
  handoff without turning it into transcript storage.

## Dependencies

- shared workspace foundations are complete
- Obsidian compiled evidence workspace is complete
- bundle-backed resume and remember flows are already in place
