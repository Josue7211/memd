---
phase: D13
status: closed
opened: 2026-05-05
closed: 2026-05-05
axis: session_continuity
depends_on: [phase-c13-cross-device-crdt-sync.md]
---

# D13 Dormant Project Recovery

## Goal

Recover a project after a 30-day gap with focus intact and wake median under
the 1500-token TE floor.

## Close Evidence

- Fixture:
  `crates/memd-client/fixtures/shared/release-0-1-0/sessions/dormant-30d.jsonl`
- Long-session fixture:
  `crates/memd-client/fixtures/shared/release-0-1-0/sessions/long-session-200t-4c.jsonl`
- Core proof: `memd_core::v13::dormant_project_recovery`
- TE check:
  `docs/verification/release-0-1-0/2026-05-05-te-integration-check.ndjson`

## Result

Closed. SC lifts `8 -> 9`; TE holds at `7`.
