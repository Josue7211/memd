---
phase: H13
status: closed
opened: 2026-05-05
closed: 2026-05-05
axis: all
depends_on: [phase-a13-public-bench-domination.md, phase-g13-third-party-provenance-replay.md]
---

# H13 Release Harness

## Goal

Run the full 0.1.0 release battery, regenerate the 10-STAR scorecard in strict
mode, and write release-ready evidence.

## Close Evidence

- Verifier: `scripts/verify/v13-release-suite.sh`
- Harness log:
  `docs/verification/release-0-1-0/2026-05-05-g13-harness.ndjson`
- Release-ready marker:
  `docs/verification/release-0-1-0/2026-05-05-0-1-0-release-ready.txt`
- Scorecard:
  `docs/verification/MEMD-10-STAR.md`

## Result

Closed. Composite regenerates to `8.50/10`; 0.1.0 release gate is marked ready.
