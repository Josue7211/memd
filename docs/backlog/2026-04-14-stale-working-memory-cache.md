# Stale Working Memory Cache

- status: `open`
- severity: `high`
- phase: `V2-B2`
- opened: `2026-04-14`
- scope: memd-core

## Problem

Working memory is loaded at session start and cached until checkpoint. If session is long (hours), cache becomes stale. Corrections made during session are invisible unless agent explicitly reloads. Silent data inconsistency.

## Fix

- Add cache TTL with background refresh
- Implement invalidation signal on correction
- Test long-session staleness scenarios
- Add to phase-B2 acceptance criteria (cache freshness)
