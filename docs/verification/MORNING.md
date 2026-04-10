# memd morning summary

- Current benchmark score: `94/100`

## Continuity Failures
- feature.bundle.wake [bundle-runtime] coverage=auditing drift=continuity-drift|surface-drift
- feature.bundle.resume [bundle-runtime] coverage=auditing drift=continuity-drift|memory-drift
- feature.bundle.handoff [bundle-runtime] coverage=auditing drift=continuity-drift|surface-drift
- feature.bundle.attach [bundle-runtime] coverage=auditing drift=continuity-drift|harness-drift
- feature.capture.checkpoint [capture-compaction] coverage=auditing drift=continuity-drift|capture-drift

## Verification Regressions
- nightly verify lane nightly is green at 16/16

## Verification Pressure
- verifier.adversarial.hive-claim-collision status=passing gate=acceptable target=acceptable continuity_critical=true
- verifier.adversarial.hive-message-lane-collision status=passing gate=acceptable target=acceptable continuity_critical=true
- verifier.adversarial.hive-task-lane-collision status=passing gate=acceptable target=acceptable continuity_critical=true
- verifier.feature.bundle.attach status=passing gate=acceptable target=acceptable continuity_critical=true
- verifier.feature.bundle.handoff status=passing gate=acceptable target=acceptable continuity_critical=true

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

