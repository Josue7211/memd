# memd morning summary

- Current benchmark score: pending live benchmark run
- Registry: `docs/verification/benchmark-registry.json`
- Continuity-critical features: `8`
- Runtime policies: `4`

## Continuity Failures
- `feature.bundle.wake` `[bundle-runtime]` coverage=`auditing` drift=`continuity-drift|surface-drift`
- `feature.bundle.resume` `[bundle-runtime]` coverage=`auditing` drift=`continuity-drift|memory-drift`
- `feature.bundle.handoff` `[bundle-runtime]` coverage=`auditing` drift=`continuity-drift|surface-drift`
- `feature.bundle.attach` `[bundle-runtime]` coverage=`auditing` drift=`continuity-drift|harness-drift`
- `feature.capture.checkpoint` `[capture-compaction]` coverage=`auditing` drift=`continuity-drift|capture-drift`
- `feature.capture.hook-capture` `[capture-compaction]` coverage=`auditing` drift=`continuity-drift|event-drift`
- `feature.memory.working-context` `[bundle-runtime]` coverage=`auditing` drift=`continuity-drift|memory-drift`
- `feature.memory.working-memory` `[bundle-runtime]` coverage=`auditing` drift=`continuity-drift|memory-drift`

## Drift Risks
- `continuity-drift`
- `memory-drift`
- `surface-drift`
- `capture-drift`
- `event-drift`
- `harness-drift`

## Token Regressions
- benchmark token efficiency is pending live benchmark run
- no-memd vs with-memd comparison is pending live benchmark run
- benchmark budget policies are registered but not yet exercised by a live run

## With memd vs No memd
- no-memd comparison not available yet
- run `memd benchmark --output .memd --write` to populate comparative evidence

## Next Actions
- benchmark the remaining continuity-critical features
- run live benchmark write to refresh `MORNING.md`
- compare `with memd` against `no memd` on the resume journey
- validate benchmark cost budgets in the next live run
