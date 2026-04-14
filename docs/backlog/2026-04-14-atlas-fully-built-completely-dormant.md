# Atlas Fully Built but Completely Dormant

status: open
severity: high
phase: Phase I
opened: 2026-04-14

## Problem

Atlas has 974 lines of code, 7 routes, region clustering, trail tracking,
entity search with multi-factor scoring, 18 passing tests. Never called
from resume/wake path. Entity links table is permanently empty. No client
methods for atlas queries. CLI `memd explore` exists but is never invoked
in any harness integration.

## Evidence

- atlas.rs: 974 lines, 7 routes
- routes.rs: all 7 mapped
- Tests: 18 pass
- Entity links table: created in schema, never written to
- lib.rs: no atlas client methods
- No harness calls atlas

## Fix

1. Wire atlas into resume path — show relevant regions in wake packet
2. Populate entity links table (auto-link from co-occurrence already built)
3. Add client methods for explore/region/trail
4. Surface atlas navigation in dashboard (Phase I)
