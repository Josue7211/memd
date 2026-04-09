# memd 10-Loop Autoresearch Stack

## Goal
Build a 10-loop autoresearch stack that catches weak or false-green improvements fast while still rewarding real gains in memd runtime, hive coordination, memory quality, and repo hygiene.

## Design Principles
- Each loop must measure a distinct failure mode.
- A loop only counts as `success` when it passes all gates, beats its previous best, and improves or preserves a second signal.
- Warnings should be explicit about the failed gate or gates.
- `refresh_recommended` is metadata, not an automatic failure.
- The stack should optimize for aggressive failure detection, not shallow green rates.

## Loop Set

### 1. `hive-health`
Tracks live peers, heartbeat publication, dead sessions, and claim collisions.

### 2. `memory-hygiene`
Tracks stale memories, duplicate memories, orphaned entries, and compression wins.

### 3. `autonomy-quality`
Tracks false-green rate, warning rate, and real delta versus noise.

### 4. `prompt-efficiency`
Tracks prompt token burn, reuse rate, and bundle shrink.

### 5. `repair-rate`
Tracks how often the system fixes real problems instead of churning on superficial changes.

### 6. `signal-freshness`
Tracks stale snapshot rate, live-truth drift, and refresh pressure.

### 7. `cross-harness`
Tracks portability across sessions, bundles, and harnesses.

### 8. `self-evolution`
Tracks accepted experiments that stick and keep improving after acceptance.

### 9. `branch-review-quality`
Tracks branch cleanliness, diff quality, and review readiness.

### 10. `docs-spec-drift`
Tracks whether docs and specs still match shipped behavior.

## Success Rules
Each loop must:
- pass its absolute evidence floor
- avoid regression versus the previous same-loop run
- beat its previous best when a new green is accepted
- keep `warning_reasons` explicit

The system-level stack is considered healthy when:
- at least 8 of 10 loops are `success`
- no loop is green due only to stale or empty data
- any warning has an explicit reason and an evidence trail

## Metrics and Gates
All loops use the same shared gate pattern:
- freshness check
- trend check against the prior same-loop run
- absolute floor check on percent, token savings, and evidence count

Loop-specific score formulas may differ, but they must produce:
- `percent_improvement`
- `token_savings`
- `evidence`
- `confidence`
- `trend`
- `warning_reasons`

## Data Sources
Primary sources:
- `.memd/` runtime state
- live memd server bundle snapshots
- repo memory and claims
- recent loop history
- docs/spec artifacts
- git branch and review state

## Rollout Plan
1. Add descriptors and scoring for the 10 loops.
2. Wire the new loops into the autoresearch runner.
3. Add tests for each new loop’s warning and success paths.
4. Run the stack on the current project and tune the floors from actual measurements.
5. Keep the background loop running so the stack can continue self-calibrating.

## Testing
Coverage must include:
- one success-path test per loop
- one warning-path test per loop
- trend regression coverage
- absolute-floor coverage
- saved-record validation for all loop outputs

## Notes
- `refresh_recommended` should remain visible in metadata.
- It should not by itself force a warning when the measured signal is clearly good.
- The goal is to prevent false-green loops, not to punish healthy runs that also need a fresh session.
