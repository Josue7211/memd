---
milestone: v8
name: Operator Surfaces
status: planned
opened: 2026-04-22
depends_on: [v7]
composite_pre: 7.8
composite_target: 8.5
axes_lifted: [trust_provenance, procedural_reuse]
---

# Milestone v8 Audit — Operator Surfaces

## Goal

User can see memd. Atlas navigation, correction UX, memory inspector, provenance browser, diff + rollback UI, public leaderboard transparency page. Competitor surfaces (mempalace atlas, mem0 dashboard, letta correction UX) are the bar; memd clears it.

## 10-STAR axis targets (pre / post)

| axis | weight | pre | post |
| --- | --- | --- | --- |
| trust + provenance | 10% | 8 | 9 |
| procedural reuse | 15% | 5 | 7 |
| session continuity | 20% | 7 | 8 |
| other axes | — | stable | stable |

composite: 7.8 → 8.5

## Phases

- **A8** Atlas navigation UI — graph view of canonical memory, click to provenance.
- **B8** Correction UX — inline correction capture, live preview of "before/after" retrieval.
- **C8** Memory inspector — all records by type, searchable, filterable.
- **D8** Provenance browser — click any fact, trace to source turn + extraction reason.
- **E8** Diff + rollback UI — see what changed, undo with one click.
- **F8** Public leaderboard transparency page — live method cards, reproduction commands, retraction log, gaming-audit rule.

## Completion gate

Stranger test: outside reviewer (sidecar OFF) rates memd best-in-class on 5 surfaces (wake quality, correction UX, atlas navigation, episode readability, leaderboard verifiability) vs mempalace / supermemory / letta / mem0. Evidence: reviewer write-up + 5 side-by-side screencasts.

## Non-goals

- mobile UI (scoped out)
- IDE integration UI beyond claude-code/codex (scoped out)
