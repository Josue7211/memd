---
date: 2026-04-17
milestone: V3
phase: A3
part: 1
status: plan_ready
branch: research/mining
head: a64510d
supersedes: 2026-04-17-v3-phase-rename-next-A3.md
next_action: pick execution mode (subagent-driven vs inline) and run plan
---

# A3 Part 1 plan ready — next is execute

## State

- Branch `research/mining`, tree clean, tip `a64510d`.
- Plan landed: `docs/superpowers/plans/2026-04-17-a3-part1-continuity-foundation.md` (11 tasks).
- Phase doc `docs/phases/phase-a3-continuity-foundation.md` unchanged (Part 1 maps to D1–D3 + D5).
- Roadmap: V3 A3 Continuity Foundation active; B3 retrieval blocked until A3 ships.

## What changed since last handoff

- Wrote full Part 1 implementation plan (file-interaction ledger, PostToolUse hook, wake `## Files Touched` block, `memd prime-reads`, lifecycle self-test, `.memd/contract.json` + `memd contract verify`).
- Ran advisor review. Two blockers surfaced:
  1. Task 8 was testing plumbing, not the phase-doc acceptance ("≥10 Edits zero re-Read errors"). **Fix (landed)**: re-scoped Part 1 as the *surfacing* half — ledger, wake block, prime-reads, contract. The enforcement gate ("zero re-Read errors under a blocking validator") moves to Part 2 where the cross-harness validator lives.
  2. Task 10 (cross-session preference replay) assumed a passing test, but backlog `2026-04-15-memd-preferences-not-persisted-across-sessions` says the behavior is already red. **Fix (landed)**: deferred to Part 2 alongside the validator work that will actually exercise preference durability.
- Sharpenings landed: `NotebookEdit` uses `notebook_path` not `file_path`; `collect_files_touched` now falls back to the live unsealed ledger when no sealed copy exists.
- Commit trail:
  - `80e958f docs: A3 Part 1 implementation plan — continuity foundation` (initial)
  - `a64510d docs: A3 Part 1 plan — address advisor blockers` (rework)

## Part 1 scope (final)

4 deliverables:
1. **D1** — File-interaction ledger + PostToolUse hook + wake `## Files Touched` block
2. **D2** — `memd prime-reads` CLI
3. **D3** — Working-memory lifecycle self-test
4. **D5** — Live memory contract.json + `memd contract verify`

**Deferred to Part 2** (explicit):
- D4 cross-session preference replay
- Enforcement of "zero re-Read errors after compaction" (cross-harness validator blocks Edit-without-Read on ledgered paths)
- Hooks consolidation under one canonical tree
- Strict/locked enforcement modes
- Drift detection + repair

## Part 1 Pass Gate

- `cargo test -p memd-core -p memd-client` green incl. `continuity_foundation_tests`
- Real Claude Code session: ≥5 file touches → `/compact` → sealed ledger exists → continuation wake shows `## Files Touched` → `memd prime-reads` lists paths
- `memd diagnostics lifecycle-probe` returns green
- `memd contract verify` exit 0
- `.memd/contract.json` committed

## Next action

User picks execution mode:
- **Subagent-driven** (recommended) — fresh subagent per task + two-stage review; uses `superpowers:subagent-driven-development`
- **Inline** — batched in-session execution with checkpoints; uses `superpowers:executing-plans`

Then start Task 1 (FileInteractionEntry types TDD in `crates/memd-core/src/file_ledger.rs`).

## Open questions for Part 2 (capture now, answer later)

- Validator placement: cross-harness pre-send hook vs memd-server middleware vs CLI wrapper?
- Preference storage: does `memd remember --kind preference` already reach durable substrate, or does it land in working memory only? Test on fresh bundle.
- Hook consolidation: symlinks vs install-generated mirrors — which harness (Codex, Gemini CLI) breaks on symlinks?

## Risks still live

- PostToolUse hook adds per-tool-call latency. If it regresses UX we fall back to precompact-only sealing (lossier, cheap).
- Ledger grows linearly with distinct files × ops. Retention policy is a Part 2 concern; Part 1 lives with linear growth since `/compact` is rare.
