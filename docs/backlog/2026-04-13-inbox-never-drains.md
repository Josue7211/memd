# Inbox Never Drains — Expired Items Accumulate

- status: `open`
- found: `2026-04-13`
- scope: memd-server
- severity: critical

## Summary

No drain/acknowledge/clear endpoints. Expired items still appear in inbox because
the filter at `routes.rs:266` includes all non-Active items (including Expired).
No garbage collection for expired memory items. Currently 6 ghost items from
deleted `.planning/` files persist indefinitely.

## Symptom

- `memd inbox` shows 6 items all `status=expired` referencing `.planning/ROADMAP.md` etc.
- Items have `reasons: [expired, derived, ttl]` but no way to dismiss
- Inbox count never decreases

## Root Cause

- Inbox filter at `routes.rs:263-266`: `entry.item.stage == Candidate || entry.item.status != Active`
  - This includes Expired items (status != Active is true for Expired)
- No `POST /memory/inbox/acknowledge` or `DELETE /memory/inbox/{id}` endpoint
- No batch cleanup or GC for expired items past TTL + grace period
- `apply_lifecycle()` at `keys/mod.rs:110-132` marks items expired but never removes them

## Fix Shape

- Add `.filter(|e| e.item.status != MemoryStatus::Expired)` before inbox filter at `routes.rs:263`
- Or: exclude expired items that have been expired longer than a grace period (e.g., 1h)
- Add drain endpoint: `POST /memory/inbox/dismiss` to remove acknowledged items
- Add GC pass in worker: delete expired items older than 24h

## Evidence

- `crates/memd-server/src/routes.rs:255-327` — `get_inbox()` handler
- `crates/memd-server/src/helpers.rs:1200-1238` — `inbox_reasons()` includes "expired"
- `crates/memd-server/src/keys/mod.rs:110-132` — `apply_lifecycle()` marks expired, doesn't remove
- Current inbox: 6 items all `status=expired` from `.planning/` files

## Dependencies

- blocks: [[docs/backlog/2026-04-13-stale-continuity-ghost-refs.md|stale-continuity-ghost-refs]] (continuity pulls from expired inbox)
- blocks: [[docs/backlog/2026-04-13-status-noise-floods-memory.md|status-noise-floods-memory]] (fix ghosts first or freed working slots fill with ghosts)
- blocks: [[docs/backlog/2026-04-13-wake-packet-kind-coverage.md|wake-packet-kind-coverage]] (ghost inbox poisons continuity before facts can surface)
- **fix this first** — all downstream fixes depend on clean inbox

## Related

- [[docs/audits/2026-04-13-full-codebase-audit.md]] — full audit findings
- [[docs/theory/locks/2026-04-11-memd-theory-lock-v1.md]] — live loop step 3 (update session continuity)
- [[docs/theory/locks/2026-04-11-memd-evaluation-theory-lock-v1.md]] — 10-star axis: session continuity (20%)
