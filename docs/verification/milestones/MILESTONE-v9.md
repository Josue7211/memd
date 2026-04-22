---
milestone: v9
name: Multi-User / Team
status: planned
opened: 2026-04-22
depends_on: [v8]
composite_pre: 8.5
composite_target: 9.0
axes_lifted: [cross_harness]
---

# Milestone v9 Audit — Multi-User / Team

## Goal

Shared-namespace memory. Two users, three agents, one truth — without silent overwrites, without visibility leaks, with divergence surfaced. The features already exist in schema (workspace, visibility, hive, merge governor) per MILESTONE-v3 findings; V9 proves they're trustworthy end-to-end.

## 10-STAR axis targets (pre / post)

| axis | weight | pre | post |
| --- | --- | --- | --- |
| cross-harness | 15% | 6 | 9 |
| trust + provenance | 10% | 9 | 9 |
| session continuity | 20% | 8 | 9 |
| other axes | — | stable | stable |

composite: 8.5 → 9.0

## Phases

- **A9** Shared namespace semantics — ns precedence, project vs global, clear rules.
- **B9** Visibility / ACL honored by retrieval — private records never leak cross-user.
- **C9** Merge collision governor live — two agents write conflicting canonical, governor resolves or surfaces.
- **D9** Hive divergence receipts — user sees "agent A said X, agent B said Y, canonical is Z because...".
- **E9** Multi-agent handoff quality — claude-code → codex → cursor → back, truth conserved.
- **F9** Team-wide correction propagation — one user corrects, team's canonical updates, provenance shows who.

## Completion gate

2-user 3-agent 10-session dogfood: truth conserved (V5 C5 suite passes), divergences surfaced (D9), zero visibility leaks, zero silent overwrites. Audit trail on every cross-user canonical mutation.

## Non-goals

- enterprise SSO (separate milestone, post-V10)
- billing / quotas
