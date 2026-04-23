---
phase: C2
name: Ghost Cleanup
version: v2
status: reopened
depends_on: [B2]
backlog_items: [46, 50, 76]
verified_at: 2026-04-14
reopened_at: 2026-04-15
reopened_reason: GC mechanics work in tests but stale records still accumulate in production. Working memory holds B2 phase status weeks after completion. TTL enforcement and lifecycle management not running in real use.
---

# Phase C2: Ghost Cleanup

Current status: `reopened` — GC mechanics verified in test but production DB still accumulates dead weight. Stale completed-phase records persist indefinitely.

## Reopened Scope

- **TTL enforcement in production**: expired items must actually be removed, not just flagged
- **Phase-completion expiry**: when a phase flips to verified/complete, its status records expire
- **Lifecycle management**: the consolidation loop must run and clean up

## Node Verification (from [[docs/verification/NODE-VERIFICATION-MATRIX.md]])

This phase owns M1-tier verification for:
- P2 (session continuity): answers what/where/changed/next without ghosts
- M2 (session continuity): clean resume without ghost refs

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

### Verification (2026-04-14)

- **Gate 1** (0 ghost refs in context): `context_items=3, ghost_refs=0` — expired/superseded items filtered by `compact_inbox_items()` and server-side `filter_items()` (default excludes Expired status).
- **Gate 2** (expired cleaned in 1 cycle): `gc_expired_items(3600)` removed 1 expired item past grace period. 0 expired remaining after single `maintain --mode full --apply true`.
- **Gate 3** (concurrent writes): `concurrent_writes_no_sqlite_busy` test — 3 threads × 50 inserts, 0 SQLITE_BUSY errors, 150/150 items stored. WAL mode + `busy_timeout=5000` + `TransactionBehavior::Immediate`.

## Fail Conditions

- Ghost refs still appear in continuity
- GC removes active items
- Concurrent writes still deadlock

## Donor Extraction (from inspiration repos)

- **C2-D1** (Omegon `sqlite.rs`): Lifecycle-driven expiration — TTL check on every access, not separate GC pass. `created_at + ttl_seconds <= now → Expired`.
- **C2-D2** (Smriti `TurnEvent.sequence_number`): Monotonic sequence isolation. Capture `history_base_seq` on resume, filter to `seq > base`. Prevents pre-mount data contaminating current context. Eliminates ghost refs by construction.

See: `docs/theory/2026-04-14-donor-extraction-to-v2-phases.md` for full details.

## Rollback

- Revert GC if it removes active items
- Revert transaction changes if data corruption detected
