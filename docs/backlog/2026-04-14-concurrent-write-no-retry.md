# Concurrent Write: No Retry

- status: `open`
- severity: `critical`
- phase: `V2-C2`
- opened: `2026-04-14`
- scope: memd-core

## Problem

SQLITE_BUSY errors on concurrent writes with no retry or backoff logic. Multi-agent scenarios cause immediate failure. Deadlock risk—no serialization or transaction queuing.

## Fix

- Add exponential backoff on SQLITE_BUSY
- Implement write queue with serialization
- Add lock contention metrics
- Add to phase-C2 acceptance criteria (concurrency safety)
