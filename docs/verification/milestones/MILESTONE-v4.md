---
milestone: v4
name: Live Loop Repair
status: planned
opened: 2026-04-22
depends_on: [v3]
composite_pre: 2.15
composite_target: 4.0
axes_lifted: [session_continuity, correction_retention, procedural_reuse, token_efficiency]
---

# Milestone v4 Audit — Live Loop Repair

## Goal

memd used-as-designed in a real claude-code or codex session does not lose state, does not drop corrections, does not bloat context. Fixes 10-STAR gaps 1–9 (the pre-V3 "memory OS broken in daily use" set). No public-bench chasing; the win is the dogfood feel.

## 10-STAR axis targets (pre / post)

| axis | weight | pre | post |
| --- | --- | --- | --- |
| session continuity | 20% | 1 | 4 |
| correction retention | 15% | 2 | 4 |
| procedural reuse | 15% | 1 | 3 |
| token efficiency | 10% | 1 | 4 |
| cross-harness | 15% | 2 | 3 |
| raw retrieval | 15% | 6 | 6 |
| trust + provenance | 10% | 2 | 3 |

composite: 2.15 → 4.0

## Phases

See `ROADMAP.md` → "V4: Live Loop Repair". Phase docs at `docs/phases/v4/phase-{a4..g4}-*.md`.

## Completion gate

3-session claude-code dogfood recording:
- state survives compaction (A4)
- hook order honored under load (B4)
- correction in session 1 honored in session 3 (C4, F4)
- wake context <2k tokens with zero continuity loss (D4, E4)
- G4 harness passes end-to-end

Evidence: recorded trace + G4 harness JSONL + updated 10-STAR composite scorecard in `docs/verification/MEMD-10-STAR.md`.

## Non-goals

- public bench number chasing (V6 owns that)
- operator UI polish (V8)
- multi-user (V9)
