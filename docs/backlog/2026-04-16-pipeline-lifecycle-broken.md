# Promote / Expire / Archive Lifecycle Does Not Execute Reliably

- status: `open`
- severity: `critical`
- phase: `V2-M2-evo`
- opened: `2026-04-16`
- scope: memd-core

## Problem

The production lifecycle pipeline is incomplete. Candidate promotion, expiry, archival,
and working-memory cleanup are not exercised as one end-to-end contract, so records
accumulate indefinitely. Earlier gates proved single-item store and recall, but not the
full lifecycle under realistic long-running usage.

## Fix

- Add end-to-end lifecycle tests: candidate -> canonical -> archived -> absent from working set
- Run lifecycle maintenance on real completion and retention events, not just ad hoc hooks
- Surface lifecycle health in eval/scenario output so failures are visible before bloat
- Block milestone completion when lifecycle regression leaves stale records active
