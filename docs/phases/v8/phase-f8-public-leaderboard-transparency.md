---
phase: F8
name: Public Leaderboard Transparency Page + V8 Completion Gate
version: v8
status: planned
opened: 2026-04-22
depends_on: [A8, B8, C8, D8, E8]
axis: trust_provenance, V8 completion gate
plan_spec: docs/phases/v8/phase-f8-plan.md
---

# Phase F8: Public Leaderboard Transparency Page + V8 Completion Gate

## Goal

Two jobs: (1) public leaderboard page — live method cards, reproduction commands, retraction log, gaming-audit rule, current canonical numbers. (2) V8 gate — stranger test + composite ≥ 8.5.

## Why this phase exists

V6 F6 shipped PUBLIC_BENCHMARKS.md as markdown. F8 is the web-first surface that competitors' dashboards (mem0, letta) beat us on. Plus V8 gate.

## Deliver

1. **Transparency page.** Rendered from `PUBLIC_BENCHMARKS.md` + method cards via MDX; auto-refreshes on source commit.
2. **Retraction log.** Every retracted score carries a public reason + timestamp.
3. **Gaming-audit rule.** Front-and-center: "any score gaming ends our leaderboard." Linked to I3 rule card.
4. **Stranger test.** Outside reviewer runs memd (sidecar OFF); produces 5 screencasts + write-up.
5. **V8 aggregator.** Regenerates `docs/verification/V8_UI_AUDIT.md` with UI test pass rates + perf numbers.
6. **10-STAR axis writer.** Bumps trust_provenance 8→9, procedural_reuse 5→7, session_continuity 7→8.
7. **MILESTONE-v8 close.**

## Pass Gate

- pre: leaderboard is markdown; no stranger test
- post:
  - transparency page live + auto-regen
  - stranger test rates memd best-in-class on 5 surfaces (wake, correction, atlas, episode readability, leaderboard verifiability) vs mempalace/supermemory/letta/mem0
  - composite ≥ 8.5
- evidence: page URL, 5 screencasts, reviewer write-up, V8 audit doc

## Product Win

memd wins the surfaces comparison. 10-STAR composite 8.5.

## Evidence

- transparency URL
- stranger-test artifacts
- V8 audit
- 10-STAR regen

## Fail Conditions

- Any best-in-class miss: do not ship; fix the surface, re-run stranger.
- Composite < 8.5: writer refuses to publish; root-cause.

## Non-Goals

- Comparing to closed-source competitors we can't reproduce.
- Paid user studies (use volunteer reviewer).
