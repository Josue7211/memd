# Correction Flow Has No User Pathway

status: open
severity: high
phase: Phase I
opened: 2026-04-14

## Problem

Theory says correction is first-class: user says "wrong" → supersede →
stale belief replaced in future recall. Mechanics exist (supersede field,
repair endpoints, belief branches, trust hierarchy) but there is zero
user-facing pathway. No CLI command, no API call path from "that's wrong"
to a supersede operation.

## Evidence

- MemoryStatus::Superseded exists
- Repair endpoint with 6 modes exists
- Trust scoring exists (helpers.rs:649-663)
- No `memd correct` CLI command
- No `correct-memory` lane helper wired to supersede
- No E2E test: store fact → correct it → verify future recall reflects correction

## Fix

1. Add `memd correct --id <uuid> --content "corrected content"` CLI command
2. Wire `correct-memory` lane helper to call supersede with belief branch
3. Add E2E test proving correction flows through to recall
4. Dashboard: correction UI in Phase I
