# Queen Operations Dead Code — Routes Without Client

- status: `open`
- found: `2026-04-13`
- scope: memd-server, memd-client
- severity: medium

## Summary

3 queen routes (deny, reroute, handoff) implemented in `routes.rs` but NO
corresponding client methods in `lib.rs`. Coordination modes (exclusive_write,
shared_review) stored on tasks but not enforced at claim or DB level. Overlap
detection is post-hoc (in board/follow views), not preventive.

## Symptom

- `POST /hive/queen/deny` exists but no `client.queen_deny()` method
- Tasks declare `coordination_mode: "exclusive_write"` but two agents can claim same scope
- Overlap detected after the fact in hive board, not blocked during claim acquisition

## Root Cause

- Queen routes were Phase H prep, implemented server-side but client stubs never added
- Coordination modes are advisory metadata, not enforced constraints
- Overlap detection only runs in `hive_follow()` and `hive_board()` views
- `acquire_hive_claim()` does not check coordination mode of related tasks

## Fix Shape

- Add client methods: `queen_deny()`, `queen_reroute()`, `queen_handoff()` in lib.rs
- Or: remove routes if truly Phase H deferred, document in roadmap
- Add coordination mode check in `acquire_hive_claim()` — reject exclusive_write conflicts
- Add overlap detection in `assign_hive_task()` before returning success

## Evidence

- `crates/memd-server/src/routes.rs:842-985` — 3 queen route handlers
- `crates/memd-client/src/lib.rs` — no queen methods
- `crates/memd-server/src/store.rs:1540-1610` — `acquire_hive_claim()` no mode check
- `crates/memd-server/src/store_coordination.rs:1-50` — `hive_board()` does overlap detection

## Dependencies

- independent: can fix standalone (add client methods or remove routes)
- Phase H territory — may defer

## Related

- [[docs/audits/2026-04-13-full-codebase-audit.md]] — full audit findings
- [[docs/theory/locks/2026-04-11-memd-hive-theory-lock-v1.md]] — hive coordination theory
- [[docs/backlog/2026-04-13-hive-deferred-transaction.md|hive-deferred-transaction]] — related hive concurrency issue
