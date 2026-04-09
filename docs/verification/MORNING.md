# memd morning summary

- Current benchmark score: `94/100`

## Continuity Failures
- feature.bundle.wake [bundle-runtime] coverage=auditing drift=continuity-drift|surface-drift
- feature.bundle.resume [bundle-runtime] coverage=auditing drift=continuity-drift|memory-drift
- feature.bundle.handoff [bundle-runtime] coverage=auditing drift=continuity-drift|surface-drift
- feature.bundle.attach [bundle-runtime] coverage=auditing drift=continuity-drift|harness-drift
- feature.capture.checkpoint [capture-compaction] coverage=auditing drift=continuity-drift|capture-drift

## Drift Risks
- capture-drift
- continuity-drift
- event-drift
- harness-drift
- memory-drift

## Token Regressions
- no-memd prompt tokens=8046 with-memd prompt tokens=4806 delta=3240
- no-memd rereads=4 with-memd rereads=1 delta=3

## With memd vs No memd
- with memd beats no memd by 3240 tokens, 3 rereads, and 83 reconstruction steps

## Next Actions
- benchmark the remaining continuity-critical features

