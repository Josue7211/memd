---
phase: A4
name: Read-State Across Compaction
version: v4
status: planned
opened: 2026-04-22
depends_on: []
backlog_items: [read-state-lost-across-compaction]
axis: session_continuity
---

# Phase A4: Read-State Across Compaction

## Goal

When a claude-code or codex session hits auto-compaction, memd's view of what the agent has already read must survive. Current behavior: PreCompact hook captures working state but read-ledger pointer gets dropped on the other side, agent re-reads the same files cold. This phase makes the ledger survive round-trip.

## Why this phase exists

V3 A3 Part 1 landed the file-interaction ledger + prime-reads + PreCompact non-blocking. The ledger writes. The prime-reads fire. But the cross-compaction continuity is not yet proven: on a real compaction event the post-compaction session does not always reload the ledger before the first tool call, so the agent re-reads files it already knew. 10-STAR axis 1 (session continuity) is stuck at 1/10 until this is closed.

## Deliver

1. **Compaction survival test.** Scripted claude-code session that:
   - reads 5 files (ledger populates)
   - triggers compaction
   - queries for the 5 files — expected: ledger returns cached summary, not cold re-read
2. **PreCompact → PostCompact handoff contract.** Written in `docs/contracts/hook-handoff.md`: PreCompact writes ledger checkpoint, PostCompact prime-reads MUST fire before first tool call, ledger pointer MUST be loaded.
3. **Hook ordering guard.** `memd hooks doctor` extended to verify PostCompact hooks fire in the documented order on a test run.
4. **Failure telemetry.** If a post-compaction tool call fires before ledger reload, emit a one-line warning to `.memd/logs/continuity-breach.log`.
5. **Regression test.** In CI: spin up a mock claude-code session, simulate compaction, assert ledger survives.

## Pass Gate

- pre: compaction survival test fails (agent re-reads) on ≥50% of runs
- post: compaction survival test passes 10/10 runs, zero `continuity-breach.log` entries
- evidence: test run log + `.memd/logs/continuity-breach.log` empty across 10 runs + 10-STAR session-continuity axis re-scored (target +1)
- regression budget: zero new hook failures; `memd hooks doctor` stays green

## Product Win

User runs a long session, hits compaction, picks back up — memd already knows what the agent read. No cold re-read of the same files. Felt as "memd remembered, I didn't have to re-explain."

## Evidence

- Compaction survival test script + 10-run log
- Updated `docs/contracts/hook-handoff.md`
- `memd hooks doctor` output showing ordering check
- 10-STAR scorecard delta (axis 1: 1 → 2 minimum)

## Fail Conditions

- If handoff contract requires breaking change to hook API: file contract-breakage backlog, pause A4, coordinate with hook consumers (codex, claude-code agents).
- If telemetry shows breaches are config-dependent (some users ok, some not): ship fix but mark A4 partial until all configs covered.

## Rollback

Feature-flagged behind `MEMD_A4_LEDGER_SURVIVAL=1`. Off by default for the first dogfood week.
