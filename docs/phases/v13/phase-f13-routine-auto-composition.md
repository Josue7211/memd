---
phase: F13
status: closed
opened: 2026-05-05
closed: 2026-05-05
axis: procedural_reuse
depends_on: [../v13/V13-INTEGRATION.md]
---

# F13 Routine Auto Composition

## Goal

Promote repeated A+B+C behavior into a composed routine and share it to a
second workspace with origin metadata intact.

## Close Evidence

- Fixtures:
  `crates/memd-client/fixtures/shared/release-0-1-0/routines/`
- Core primitive: `memd_core::v13::auto_compose_repeated_routine`
- Axis proof:
  `docs/verification/release-0-1-0/2026-05-05-axis-procedural_reuse.ndjson`

## Result

Closed. PR lifts `8 -> 9`.
