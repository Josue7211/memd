# Continuity Fields Reference Deleted Files

- status: `closed`
- found: `2026-04-13`
- scope: memd-client
- severity: critical

## Summary

Continuity capsule `left_off` and `blocker` fields pull from expired inbox items
that reference `.planning/` files no longer on disk. No file existence validation
anywhere in the chain. `memd status` heartbeat shows "editing .planning/ROADMAP.md"
— a file that doesn't exist.

## Symptom

- wake.md shows `left_off=file_edited: .planning/ROADMAP.md`
- `blocker=file_edited: .planning/ROADMAP.md file_edited: .planning/STATE.md`
- `memd status` heartbeat working field references nonexistent paths
- Fresh sessions see stale context from weeks ago

## Root Cause

- `compact_inbox_items()` at `resume/mod.rs:1450-1457` does not filter expired items
- `continuity_left_off()` at `resume/mod.rs:1285-1301` pulls first inbox item without validation
- `blocker` at `resume/mod.rs:1509-1512` pulls first inbox item without validation
- `summarize_repo_event_line()` at `evidence.rs:231-268` creates events without path validation
- No mechanism anywhere checks if referenced file paths still exist on disk

## Fix Shape

- Filter expired items in `compact_inbox_items()`: skip items with `reasons.contains("expired")`
- Or: validate that `file_edited: <path>` references existing files before including
- Heartbeat working field: validate path before storing
- Defense in depth: server-side inbox excludes expired (see #29)

## Evidence

- `crates/memd-client/src/runtime/resume/mod.rs:1450-1457` — `compact_inbox_items()`
- `crates/memd-client/src/runtime/resume/mod.rs:1285-1301` — `continuity_left_off()`
- `crates/memd-client/src/runtime/resume/mod.rs:1509-1512` — blocker from inbox
- `crates/memd-client/src/workflow/improvement/evidence.rs:231-268` — no path validation

## Dependencies

- blocked-by: [[docs/backlog/2026-04-13-inbox-never-drains.md|inbox-never-drains]] (server-side expired exclusion eliminates most ghost sources)
- blocks: [[docs/backlog/2026-04-13-status-reports-healthy-while-broken.md|status-reports-healthy-while-broken]] (heartbeat uses same ghost data)
- blocks: [[docs/backlog/2026-04-13-memd-no-cross-session-codebase-memory.md|no-cross-session-codebase-memory]] (continuity noise hides real content)

## Related

- [[docs/audits/2026-04-13-full-codebase-audit.md]] — full audit findings
- [[docs/theory/locks/2026-04-11-memd-theory-lock-v1.md]] — live loop step 3 (session continuity)
- [[docs/phases/phase-b-session-continuity.md]] — Phase B pass gate claims "continuity explicit"
