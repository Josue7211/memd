---
milestone: v16
name: Cross-Device Sync at Scale
status: planned
opened: 2026-04-22
depends_on: [v15, ../../verification/1.0.0-CONTRACT.md, ../../verification/1.0.0-AXIS-OWNERSHIP.md]
composite_pre: 8.70
composite_target: 9.05
axes_lifted: [session_continuity, cross_harness]
axes_integrated_with: []
---

# Milestone v16 Audit — Cross-Device Sync at Scale

## Goal

Same user, multiple devices (desktop + laptop + mobile + CLI on server)
see the same memory state with CRDT merge resolution on conflicts.
Dormant-project recovery at months-long horizon (6+ months) with zero
measurable quality delta vs same-session. Cross-device replay produces
identical behavior turn-for-turn. Ships SC 9→10 and CH 8→9.

## 10-STAR axis targets (pre / post)

| axis | weight | pre | post | basis |
| --- | --- | --- | --- | --- |
| session_continuity   | 20% | 9  | 10 | **OWNS +1** — multi-month dormant resume + cross-device replay identical |
| correction_retention | 15% | 8  | 8  | — |
| procedural_reuse     | 15% | 9  | 9  | — |
| cross_harness        | 15% | 8  | 9  | **OWNS +1** — cross-device scaled CH dimension |
| raw_retrieval        | 15% | 9  | 9  | — |
| token_efficiency     | 10% | 9  | 9  | — |
| trust_provenance     | 10% | 9  | 9  | — |

**Composite: 8.70 → 9.05**.

## Phases (planned)

- **A16** CRDT memory layer (conflict-free replicated datatype for `MemoryRecord`)
- **B16** Sync protocol (optional self-hosted relay or peer-to-peer; opt-in)
- **C16** Multi-device wake (any device reads current state within ≤2s of sync event)
- **D16** Dormant-project recovery (6+ month gap; wake re-hydrates focus with no quality delta)
- **E16** Cross-device replay harness (same turn sequence on device A and device B → identical behavior)
- **F16** `memd configure sync.enabled/relay_url/conflict_policy`
- **G16** V16 gate harness (3-device dogfood ≥90 days; dormant-project replay proof)

## Completion gate

1. ≥90-day 3-device dogfood (desktop + laptop + mobile or SSH'd server).
2. Dormant-project recovery: ≥1 project with ≥6-month gap, wake re-hydrates focus with ≤0.02 fidelity delta.
3. Cross-device replay: same turn sequence on two devices produces identical memory state.
4. CRDT conflict resolution verified (synthetic conflict scenario: device A + B edit same record; merge resolves without data loss).
5. 10-STAR composite regenerated ≥9.05 with SC=10, CH=9.

## Non-goals

- Cross-user sync (V17 owns federation)
- Cloud-managed sync-as-a-service (V16 is self-hosted / peer-to-peer)
- Real-time collaborative editing (async CRDT is enough)

## Changelog

- 2026-04-22 opened.
