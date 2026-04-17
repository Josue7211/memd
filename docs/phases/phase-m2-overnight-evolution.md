---
phase: M2-evo
name: Overnight Evolution
version: v2
status: pending
depends_on: [F2, G2, K2]
backlog_items:
  - "2026-04-14-no-overnight-evolution-loop"
  - "2026-04-14-no-live-memory-contract"
  - "2026-04-16-working-memory-stale-records"
  - "2026-04-16-pipeline-lifecycle-broken"
---

# Phase M2-evo: Overnight Evolution

## Goal

Memory improves itself, stays live while the agent works, and sheds stale lifecycle
noise before it crowds out durable truth.

## Deliver

- Dream/autodream/autoevolve loops
- Live memory refresh contract for mid-session changes and resume/wake
- Lifecycle cleanup that archives stale phase/status records out of working memory
- Decay calibration from real usage data
- Consolidation quality measurement

## Pass Gate

- Mid-session correction or preference write becomes visible without a manual full reload
- Completed-phase status records archive out of working memory after lifecycle maintenance
- Overnight evolution runs without regressing recall quality
- Decay thresholds tuned: old unused items fade, frequently used items persist
- Consolidation output quality measured and above baseline

## Evidence

- Live memory contract test
- Stale-record lifecycle cleanup test
- Evolution loop artifact / regression sweep
- Decay calibration data (before/after curves)
- Consolidation quality metrics

## Fail Conditions

- Mid-session updates stay invisible until restart
- Stale records remain active after phase completion
- Decay removes active items
- Consolidation degrades memory quality

## Donor Extraction (from inspiration repos)

- **M2-D1** (Omegon `decay.rs` — **DIRECT RUST LIFT**): Reinforcement-extended half-life. Each access extends half-life: `halfLife = base × (factor ^ (count-1))`, capped at 90 days. Frequently used facts persist longer. Unused facts decay faster. Replace memd's flat 21d/0.12 decay.
- **M2-D2** (Omegon `types.rs` — **DIRECT RUST LIFT**): Episode narrative extraction. `Episode { id, mind, title, narrative, date, session_id }` + `episode_facts` junction table + FTS5 on narratives. Evolution loop consolidates session events into episodic narratives.

See: `docs/theory/2026-04-14-donor-extraction-to-v2-phases.md` for full details.

## Rollback

- Revert evolution changes if recall score drops
