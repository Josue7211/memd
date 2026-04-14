---
phase: M2
name: Overnight Evolution
version: v2
status: pending
depends_on: [F2, G2, K2]
backlog_items: [62, 63, 65]
---

# Phase M2: Overnight Evolution

## Goal

Memory improves itself. Skills proposed from patterns. Decay calibrated.

## Deliver

- Dream/autodream/autoevolve loops
- Procedure detection in runtime (not just worker)
- Skill proposal from repeated patterns
- Skill gating with evaluation gate
- Decay calibration from real usage data
- Consolidation quality measurement

## Pass Gate

- Do something 3 times → memd proposes a procedure (automated test)
- Proposed skill blocked until eval score ≥ threshold
- Decay thresholds tuned: old unused items fade, frequently used items persist
- Consolidation output quality measured and above baseline

## Evidence

- Procedure proposal test
- Skill gating test
- Decay calibration data (before/after curves)
- Consolidation quality metrics

## Fail Conditions

- False positive skill proposals
- Decay removes active items
- Consolidation degrades memory quality

## Donor Extraction (from inspiration repos)

- **M2-D1** (Omegon `decay.rs` — **DIRECT RUST LIFT**): Reinforcement-extended half-life. Each access extends half-life: `halfLife = base × (factor ^ (count-1))`, capped at 90 days. Frequently used facts persist longer. Unused facts decay faster. Replace memd's flat 21d/0.12 decay.
- **M2-D2** (Omegon `types.rs` — **DIRECT RUST LIFT**): Episode narrative extraction. `Episode { id, mind, title, narrative, date, session_id }` + `episode_facts` junction table + FTS5 on narratives. Evolution loop consolidates session events into episodic narratives.

See: `docs/theory/2026-04-14-donor-extraction-to-v2-phases.md` for full details.

## Rollback

- Revert evolution changes if recall score drops
