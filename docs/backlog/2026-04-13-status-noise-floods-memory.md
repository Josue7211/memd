# Status Noise Floods Working Memory

- status: `closed`
- found: `2026-04-13`
- scope: memd-client, memd-server
- severity: critical

## Summary

15+ auto-checkpoint triggers each create `kind=status` records with no deduplication.
24h TTL means 10-20 accumulate per day. Working memory budget (1600 chars, 8 items)
is consumed 80-90% by status noise. User facts, decisions, and preferences are evicted.

## Symptom

- `memd working` returns 7/7 status records, zero user content
- `memd resume` shows only checkpoint metadata in working memory
- facts stored via `memd remember` never appear in wake packet or working memory

## Root Cause

- `checkpoint_as_remember_args()` at `checkpoint.rs:1031` forces `kind = "status"` on all checkpoints
- No `redundancy_key` set on status records — each checkpoint creates a NEW record instead of superseding
- `working_item_priority()` at `working/mod.rs:308-448` is kind-blind — status scores same as facts
- Freshness bias (+0.06) always favors most-recent status record
- TTL = 86,400 seconds (24h) at `checkpoint.rs:809,868,911` keeps status records active too long
- 15+ trigger points: resume, compaction, wake, coordination, workspace watch, maintenance, hooks

## Fix Shape

- Add `redundancy_key` computation in `checkpoint_as_remember_args()` so status records dedup by (project, namespace, source_path)
- Reduce TTL from 86,400 to 3,600 (1 hour)
- Add kind-based preference in `working_item_priority()`: Status gets -0.15, Fact/Decision/Procedure gets +0.10
- Or add max-status-items cap (e.g., 2) in working memory admission at `working/mod.rs`

## Evidence

- `crates/memd-client/src/runtime/checkpoint.rs:1017-1044` — `checkpoint_as_remember_args()` forces kind=status
- `crates/memd-client/src/runtime/checkpoint.rs:809,868,911` — 86,400 TTL on all auto-checkpoints
- `crates/memd-server/src/working/mod.rs:308-448` — `working_item_priority()` kind-blind scoring
- `crates/memd-server/src/working/mod.rs:72-113` — 1600 char budget, 8 item admission limit

## Dependencies

- blocks: [[docs/backlog/2026-04-13-wake-packet-kind-coverage.md|wake-packet-kind-coverage]] (even if wake scoring fixed, status still floods)
- blocks: [[docs/backlog/2026-04-13-memd-no-cross-session-codebase-memory.md|no-cross-session-codebase-memory]] (facts evicted by status noise)
- blocked-by: [[docs/backlog/2026-04-13-inbox-never-drains.md|inbox-never-drains]] (fix ghosts first, or freed slots fill with ghost inbox items)

## Related

- [[docs/audits/2026-04-13-full-codebase-audit.md]] — full audit findings
- [[docs/theory/locks/2026-04-11-memd-theory-lock-v1.md]] — live loop step 2 (update working context)
- [[docs/theory/locks/2026-04-11-memd-evaluation-theory-lock-v1.md]] — 10-star scorecard axis: working-memory control
