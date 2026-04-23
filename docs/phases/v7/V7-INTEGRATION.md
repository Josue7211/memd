---
version: v7
kind: integration-plan
status: ready-to-execute
opened: 2026-04-22
scope: A7..G7
---

# V7 Integration — Cross-Phase Plan

## 1. Execution-order discipline

```
A7 ──► B7 ──► C7 ──┐
              │    │
              └► D7, E7 ──► F7 ──► G7
```

Rules:
- A7 verifies V4 C4 capture at scale. B7 cannot open until A7 miss-rate ≤ 5% confirmed on real 30-day trace.
- C7/D7/E7 parallelize after B7 lands promotion rules + retraction stage.
- F7 requires E7 chain (to render "from turn X" surface).
- G7 is strict sequential gate.

## 2. Shared fixtures

| Fixture | Owner | Shared |
| --- | --- | --- |
| `tests/fixtures/correction/shared/30-correction-corpus.jsonl` | A7 | B7 (promotion replay), C7 (scenario turns), D7 (FP corpus baseline) |
| `tests/fixtures/correction/shared/chain-schema-examples.jsonl` | E7 | F7 (surface render), G7 (rollback chain) |
| `.memd/benchmarks/substrate/fixtures/shared/correction-behavior-scenario.jsonl` | C7 | G7 (dogfood extends with rollback) |

## 3. Schema migrations (ordered)

1. B7: `Stage::Retracted`.
2. D7: `Stage::ContradictedPending`.
3. E7: `MemoryRecord.correction_chain JSONB`.
4. G7: `Stage::RetractedByRollback`.

Every migration reversible; down-migration tested before forward-run lands in CI.

## 4. Per-phase axis ownership

| Phase | Owns | Integrates | Non-goals |
| --- | --- | --- | --- |
| A7 | — (validation only) | — | PR, CH, RR, TE, TP, CR behavior-change |
| B7 | CR lift measurement (promotion_correctness_rate) | — | SC, PR, CH, RR, TE, TP |
| C7 | CR behavior-change lift (next_session_behavior_rate) | — | SC, PR, CH, RR, TE, TP |
| D7 | — (safety check only) | — | all axes |
| E7 | TP lift (chain_completeness), SC continuity data | — | CR behavior (C7 owns), PR, CH, RR, TE |
| F7 | TP surface (user-visible correction log) | E7 (uses chain data) | SC, CR, PR, CH, RR, TE |
| G7 | SC +1 aggregator (correction focus context), V7 composite writer | all phases (aggregates their harness proofs) | — |
| H7 | — (durability primitive; no axis credit) | A7, B7, C7, E7 (every correction write path routes through H7 auto-commit), V8 G8 (configure surface) | SC/CR/TP lifts do not depend on H7 for *credit* but depend on H7 for *durability* — if a correction is captured but not committed to disk atomically, a crash can drop the correction and invalidate the lift retroactively |

## 4a. Feature-flag graduation calendar

| Flag | Phase | Day | Condition |
| --- | --- | --- | --- |
| `MEMD_B7_CORRECTION_PROMOTE` | B7.7 | 7 | after 7-day clean, promotion_correctness_rate ≥ 0.90 |
| `MEMD_LEARNED_SURFACE` | F7 | N/A | production default, no gate |
| `MEMD_V7_ALLOW_BELOW_TARGET` | G7 | N/A | permanent = 0 (hard floor) |

## 5. 3-session dogfood scenario (G7)

- S1 (10 turns): plant 5 corrections, 2 contradictions, 1 rollback-target.
- S2 (5 turns): query each corrected fact; C7 metrics accumulate.
- S3 (5 turns): query again; rollback 1 correction mid-session; re-query; F7 surface validated.

Cut assertions spelled in G7 Task G7.5.

## 6. V5 B5 evolution

V5 B5 CorrectionPropagation suite gains sub-metrics across V7:
- B7: promotion_correctness_rate lifts ≥ 0.05.
- C7: adds `next_session_behavior_rate`.
- D7: adds `contradiction_resolve_success_rate`.
- E7: chain_completeness = 1.000 (hard).
- G7: B5 composite ≥ 1.00.

## 7. 10-STAR regeneration (strict-mode scorecard writer)

G7 Task G7.4 writes axis deltas. **Regenerator MUST run in strict-mode:**

| axis | pre | post | regenerator check |
| --- | --- | --- | --- |
| session_continuity | 4 | 5 | assert E7 chain_completeness == 1.000 AND G7 dogfood focus-context lift observed |
| correction_retention | 4 | 5 | assert C7 next_session_behavior_rate delta ≥ 0.05 AND B5 composite ≥ 1.00 |
| trust_provenance | 4 | 5 | assert E7 chain_completeness == 1.000 AND F7 surface provenance queryable within 2 clicks |
| (all others) | (maintained from V6) | (maintained from V6) | no change; read-only check against V6 post |

**Strict-mode rules:**
- Composite 4.45 → 4.90 exactly; refuses write if computed ≠ 4.90 ± 0.01.
- CR lift: REQUIRES C7 behavior-change metric (next_session_behavior_rate). Pure B7 promotion lift does NOT count toward V7 CR credit; B7 owns measurement only, C7 owns behavior-change lift. Contract violation → block.
- TP lift: REQUIRES E7 chain_completeness = 1.000 hard assertion. Partial provenance = no lift.
- SC lift: REQUIRES E7 chain data + G7 dogfood focus-context evidence. Drilldown alone insufficient.
- `MEMD_V7_ALLOW_BELOW_TARGET = 0` enforced; any write with axis below pre-value fails hard.

## 8. Commit strategy

Plan-spec land phase (this task): 15 atomic commits (7 phase docs + 7 plan specs + V7-INTEGRATION).

Execution commits per phase: A7=6, B7=7, C7=6, D7=7, E7=6, F7=6, G7=7 → 45 execution commits.

Handoff commit after docs commits.

## 9. Cross-phase API surface

| In | Symbol | Out |
| --- | --- | --- |
| A7 | `correction::verifier` | B7 (promotion uses miss-rate signal), G7 (aggregator) |
| B7 | `correction::promotion::promote_from_correction`; `Stage::Retracted` | C7, D7, E7, G7 |
| C7 | B5 sub-metric feed | G7 |
| D7 | `ContradictionReceipt` + `Stage::ContradictedPending`; resolve CLI | G7, V8 E8 (UI consumes) |
| E7 | `correction_chain` JSONB + audit; `memd fact provenance` | F7, G7, V8 D8 |
| F7 | Wake insert + `memd learned` | G7 dogfood surface-check |
| G7 | Rollback engine + V7 aggregator + 10-STAR writer | V8 entry gate |

## 10. Critical distinction: V4 ingestion vs V7 behavior-change

**Non-negotiable scope boundary. Contract violation risk.**

- **V4 C4 (CR lift 1→4)**: User says "X is Y"; correction captured, stored, API-queryable.
  Proof: V4 G4 harness demonstrates `correction_by_ulid()` returns the stored value.
- **V7 C7 (CR lift 4→5)**: FUTURE SESSION uses corrected value in retrieval
  **without re-prompting**. This is a behavior-change lift, not an API-availability lift.
  Proof: V7 G7 harness demonstrates S1 correction appears in S2 query result and S2 session
  does not re-ask the user about it.

If C7 does not provide `next_session_behavior_rate` metric proof, CR does not lift.
If B7 measures promotion without C7 proving behavior-change, B7 gets zero credit on CR axis
(B7 is a measurement phase per 0.1.0-AXIS-OWNERSHIP.md overlap 1).

## 10a. Open questions

- V4 C4 capture trace log field stability — confirm before A7.1.
- V6 C6 promotion engine public shape — reuse vs fork for correction source?
- `Stage` enum `#[non_exhaustive]` — confirm before migrations.

## 11. Exit criteria (V7 milestone)

1. All phase exit criteria + G7 exit criteria.
2. 10-STAR composite ≥ 4.90 (per 0.1.0-CONTRACT.md), zero axis below pre-value.
3. V5 B5 CorrectionPropagation ≥ 1.00.
4. 3-session dogfood 10/10 over 7 days (5 corrections + 2 contradictions + 1 rollback-target).
5. A7 V4 C4 capture validated (miss-rate ≤ 5%).
6. E7 chain_completeness = 1.000 hard gate.
7. C7 next_session_behavior_rate delta ≥ 0.05.
8. `MILESTONE-v7.md` filled.
9. ROADMAP V7 closed, V8 ready to open.
10. No blocker backlog on `axis: correction_retention` or `axis: session_continuity`.
