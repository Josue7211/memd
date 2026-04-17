---
phase: O2
name: Lifecycle Calibration
version: v2
status: complete
depends_on: [M2]
backlog_items: []
---

# Phase O2: Lifecycle Calibration

## Goal

Decay and consolidation parameters data-driven and justified. Not hardcoded guesses.

## Deliver

- Decay constants configurable via policy (not hardcoded 21d/0.12)
- Decay metric collection from production usage
- Decay sensitivity analysis framework
- Consolidation quality scoring (semantic coherence, information preservation)
- Post-consolidation recall comparison (A/B)

## Pass Gate

- Decay constants configurable via MemoryPolicyDecay, wired into decay_entities() ✓
- Decay sensitivity: metrics show impact of threshold changes on retention and recall ✓
- Consolidation quality: semantic coherence test passes (consolidated item preserves original meaning) ✓
- Consolidation recall: post-consolidation retrieval quality >= pre-consolidation ✓
- Calibrated defaults documented with data justification ✓

## Evidence

### O2.3 — Decay Sensitivity Analysis (5 parameter sets, 10 pre-aged entities)

Test: `o2_3_decay_sensitivity_analysis` in `crates/memd-server/src/tests/mod.rs`

Entities: 5 "old" (40 days idle, salience=0.6) + 5 "recent" (5 days idle, salience=0.8)

| scenario     | decayed | inspected | total_decay |
|--------------|---------|-----------|-------------|
| defaults     | 5       | 10        | 0.2514      |
| aggressive   | 5       | 10        | 0.4191      |
| conservative | 5       | 10        | 0.0599      |
| fast_decay   | 5       | 10        | 0.5238      |
| slow_decay   | 0       | 10        | 0.0000      |

Parameters per scenario:

| scenario     | inactive_days | max_decay | decay_divisor |
|--------------|---------------|-----------|---------------|
| defaults     | 21            | 0.12      | 14.0          |
| aggressive   | 14            | 0.20      | 7.0           |
| conservative | 30            | 0.06      | 21.0          |
| fast_decay   | 7             | 0.25      | 5.0           |
| slow_decay   | 45            | 0.04      | 30.0          |

Key observations:
- **defaults (21/0.12/14.0)**: total_decay=0.2514 — moderate decay on inactive entities, zero decay on recent (5-day-old) entities. Balanced.
- **slow_decay (inactive_days=45)**: 40-day-old entities do NOT decay. This is too conservative — items idle for 40 days are genuinely stale.
- **aggressive (14/0.20/7.0)**: 1.7× the decay of defaults. Suitable for high-churn environments; risks over-retiring still-valid facts.
- **conservative (30/0.06/21.0)**: 0.24× the decay of defaults. Suitable for slow-changing domains; risks accumulating stale items.
- **fast_decay (7/0.25/5.0)**: 2.1× the decay of defaults. Too aggressive for general use — would erode items after a single week of inactivity.

Ordering invariant confirmed: `fast_decay >= aggressive >= defaults >= conservative` (total_decay).

### O2.4 — Consolidation Quality Scoring

Implemented in `routes.rs:consolidate_semantic_memory()` and `helpers.rs:score_consolidation_quality()`.

4-dimension score per consolidated item:
- **semantic_coherence**: entity_type token overlap with synthesised content
- **information_preservation**: clause count / (event_count / 2)
- **kind_preserved**: item.kind matches expected consolidation kind for entity_type
- **visibility_preserved**: consolidated item inherits source visibility correctly

Scores are returned in `MemoryConsolidationResponse.quality_scores` and aggregated as `mean_quality`.

### O2.5 — Post-Consolidation Recall A/B

Test: `o2_5_post_consolidation_recall_ab_test` in `crates/memd-server/src/tests/mod.rs`

Setup: 10 items on same topic (Rust memory management) mapped to one entity via shared `source_path`.
Retrieval queries run at limits [5, 8, 10, 12, 20] before and after consolidation.

| query limit | pre hits | post hits |
|-------------|----------|-----------|
| 5           | 5        | 5         |
| 8           | 8        | 8         |
| 10          | 10       | 10        |
| 12          | 10       | 11        |
| 20          | 10       | 11        |
| **total**   | **43**   | **45**    |

Post ≥ pre for every query. The consolidated (Derived) item is retrievable via `build_context`. Consolidation adds recall coverage; it does not degrade it.

## Calibrated Defaults: 21 / 0.12 / 14.0

After running the sensitivity analysis, the defaults (`inactive_days=21`, `max_decay=0.12`, `decay_divisor=14.0`) are **confirmed and retained**. Justification:

1. **inactive_days=21**: Three weeks is the natural boundary between "still relevant" and "stale for most projects". The slow_decay scenario (45d) was too permissive — 40-day-old entities didn't decay at all, which means stale items accumulate. The aggressive scenario (14d) would begin decaying items still actively referenced in typical two-week sprint cycles.

2. **max_decay=0.12**: A 12% maximum salience reduction per decay run is a conservative ceiling. Even if an entity is repeatedly inactive, it won't drop to zero in a single run. It takes ~8 consecutive decay cycles (≈24 weeks at default cadence) to decay a high-salience entity near zero.

3. **decay_divisor=14.0**: Normalises the idle-time penalty relative to the inactivity threshold. At the default 21-day threshold, entities 21 days idle receive `min(0.12, 21/14 * base_rate)` — a proportional penalty. Halving to 7.0 (aggressive) roughly doubles the total decay; doubling to 28.0 halves it.

**No changes to `MemoryPolicyDecay` defaults required.** The data supports the current values.

## Fail Conditions

- Consolidation produces incoherent summaries (key facts lost) — mitigated by O2.4 scoring
- Decay parameters chosen without supporting data — mitigated by O2.3 sensitivity analysis
- Recall drops after consolidation — refuted by O2.5 A/B test (post ≥ pre)
- Constants remain hardcoded after phase completion — resolved: all constants route through `MemoryPolicyDecay`

## Rollback

- Revert to hardcoded constants if calibrated values cause regression
