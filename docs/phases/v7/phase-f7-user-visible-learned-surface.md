---
phase: F7
name: User-Visible "I learned X from Y" Surface
version: v7
status: planned
opened: 2026-04-22
depends_on: [E7]
axis: correction_retention, trust_provenance
plan_spec: docs/phases/v7/phase-f7-plan.md
---

# Phase F7: User-Visible "I learned X from Y" Surface

## Goal

Users can see what memd learned this session. CLI + wake-time hint: "last session you corrected 3 things; here's what changed." Turns silent correction capture into a visible contract.

## Why this phase exists

E7 ships the chain. But users don't CLI-query provenance mid-flow. F7 surfaces recently-promoted corrections on session start — the transparency loop that makes users trust capture.

## Deliver

1. **Wake-time surface.** `.memd/wake.md` gains a `## Recently Learned` section (last 7 days, deduped, top-N).
2. **CLI surface.** `memd learned --since 7d [--since-session <id>] [--json]`.
3. **Digest.** Optional daily digest at `.memd/logs/learned-digest-YYYY-MM-DD.md` — human-readable rollup.
4. **Opt-out.** `MEMD_LEARNED_SURFACE=0` for silent operation.

## Pass Gate

- pre: corrections invisible unless explicitly queried
- post: wake.md surfaces recent; CLI lists; digest committed sample; opt-out works
- evidence: wake.md sample, CLI output, digest sample

## Product Win

Correction capture becomes a visible, auditable loop. User trust compounds.

## Evidence

- wake.md before/after
- CLI tests
- digest sample
- opt-out test

## Fail Conditions

- Surface too noisy (>10 items default): dedupe + summarize, do not dump.
- Surface leaks private-scope corrections: V3 visibility regression; hard fail.

## Non-Goals

- UI rendering (V8)
- push notifications (out of scope)
