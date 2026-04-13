# Wake Packet Structurally Excludes Non-Status Memory Kinds

- status: `open`
- found: `2026-04-13`
- scope: memd-server, memd-client
- severity: critical

## Summary

Wake packets only surface Status + LiveTruth. Facts, Decisions, Preferences, Procedures,
Runbooks, Patterns, Constraints, and Topology are structurally excluded. Root cause:
fixed `intent=current_task` in wake gives Project scope +1.15 and Global scope -0.2.
`context_score()` is kind-blind. Even with status noise fixed (#27), facts still
won't surface because the intent system penalizes their scope.

## Symptom

- Wake packet `## Durable Truth` section shows only status/checkpoint records
- Facts stored via `memd remember --kind fact` never appear in wake
- Decisions, preferences, procedures invisible in wake
- Agent has no cross-session recall of user-created content

## Root Cause

- Wake uses fixed `intent=current_task` at `resume/mod.rs:113-174`
- `context_score()` at `helpers.rs:537-583` does NOT examine `item.kind`
- `intent_scope_bonus()` at `routing/mod.rs:77-139`:
  - CurrentTask: Project=+1.15, Global=-0.2
  - Facts typically stored in Global scope → penalized
  - Decisions/Preferences also Global → penalized
- Context/working/inbox APIs have NO kind filter parameter
- `build_context()` at `helpers.rs:27-113` separates LiveTruth (Phase A carve-out) from
  all other kinds — treats Status == Fact in scoring

## Fix Shape

- Add kind bonus in `context_score()`: Fact/Decision/Procedure +0.3, Status -0.2
- Or: add multi-intent sweep in wake compilation (one pass per intent family)
- Or: add `wake_kinds` list to harness preset — always include specified kinds
- Or: add explicit "canonical facts" carve-out in `build_context()` like LiveTruth has

## Evidence

- `crates/memd-server/src/helpers.rs:537-583` — `context_score()` kind-blind
- `crates/memd-server/src/routing/mod.rs:77-139` — intent scope bonuses
- `crates/memd-client/src/runtime/resume/mod.rs:113-174` — fixed intent in resume
- `crates/memd-client/src/runtime/resume/wakeup.rs:130-143` — Durable Truth renders context.records
- Audit table: Status=surfaces, Fact=excluded, Decision=excluded, Procedure=excluded

## Dependencies

- blocked-by: [[docs/backlog/2026-04-13-status-noise-floods-memory.md|status-noise-floods-memory]] (even if scoring fixed, status flood fills slots)
- blocked-by: [[docs/backlog/2026-04-13-inbox-never-drains.md|inbox-never-drains]] (ghost inbox items poison continuity before facts can surface)
- blocks: [[docs/backlog/2026-04-13-memd-no-cross-session-codebase-memory.md|no-cross-session-codebase-memory]] (facts must surface in wake for cross-session recall)
- blocks: [[docs/backlog/2026-04-13-dogfood-verification-gap.md|dogfood-verification-gap]] (eval assertion "wake contains fact" depends on this)

## Related

- [[docs/audits/2026-04-13-full-codebase-audit.md]] — full audit findings
- [[docs/theory/locks/2026-04-11-memd-theory-lock-v1.md]] — live loop step 7 (compile wake packet)
- [[docs/theory/locks/2026-04-11-memd-retrieval-theory-lock-v1.md]] — retrieval order and intent system
- [[docs/theory/locks/2026-04-11-memd-evaluation-theory-lock-v1.md]] — 10-star scorecard axis: raw retrieval strength
