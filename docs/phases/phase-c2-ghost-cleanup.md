---
phase: C2
name: Ghost Cleanup
version: v2
status: pending
depends_on: [B2]
backlog_items: [46, 50, 76]
---

# Phase C2: Ghost Cleanup

## Goal

Continuity fields reference only live, valid items. DB has no accumulated dead weight.

## Deliver

- Expired item filter in continuity capsule
- Source path validation before inclusion
- GC pass: delete expired items older than grace period
- Session orphan detection
- SQLITE_BUSY retry with exponential backoff
- Incomplete transaction rollback protection

## Pass Gate

- `memd status` shows 0 ghost refs
- Continuity capsule fields resolve to live items only
- Expired items cleaned from DB within 1 worker cycle
- Concurrent write test: 3 agents writing simultaneously, 0 SQLITE_BUSY errors
- No incomplete transaction artifacts in DB after simulated crash

## Evidence

- Ghost ref count before/after
- GC log showing expired items removed
- Concurrent write stress test results
- Transaction integrity test after kill -9

## Fail Conditions

- Ghost refs still appear in continuity
- GC removes active items
- Concurrent writes still deadlock

## Rollback

- Revert GC if it removes active items
- Revert transaction changes if data corruption detected
