# Hive Session Upsert Uses DEFERRED Transaction

- status: `open`
- found: `2026-04-13`
- scope: memd-server

## Summary

Hive session upsert uses `.transaction()` which defaults to DEFERRED in rusqlite.
Under concurrent multi-harness writes, the second writer gets SQLITE_BUSY → 500.
Not data corruption, but user-facing errors under the exact workload Phase H enables.

## Symptom

- Two harnesses upsert to the same session_key simultaneously → one gets 500
- Error is transient but confusing and unhandled

## Root Cause

- `store.rs:2256` — `.transaction()` uses DEFERRED behavior
- DEFERRED acquires write lock on first write, second connection blocks or gets BUSY
- No retry logic on BUSY

## Fix Shape

- Use `transaction_with_behavior(TransactionBehavior::Immediate)` for write transactions
- Or add retry with backoff on SQLITE_BUSY
- Check other transaction sites (`store.rs:519` duplicate merge) for same pattern

## Evidence

- `store.rs:2254-2316` — hive session upsert
- `store.rs:519` — duplicate merge transaction (same pattern)
