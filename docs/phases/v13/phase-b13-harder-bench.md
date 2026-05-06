---
phase: B13
status: closed
opened: 2026-05-05
closed: 2026-05-05
axis: raw_retrieval
depends_on: [phase-a13-public-bench-domination.md]
---

# B13 Harder Bench

## Goal

Make the release proof harder than a single public-bench parity claim by
requiring four named benchmark margins plus SC/CR parity rows in one table.

## Close Evidence

- Required margin table:
  `docs/verification/release-0-1-0/2026-05-05-margin-targets.md`
- Release harness:
  `docs/verification/release-0-1-0/2026-05-05-g13-harness.ndjson`

## Result

Closed. The release proof treats all four RR margins as a single all-or-nothing
gate.
