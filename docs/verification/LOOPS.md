# memd benchmark loops

- Loops: `11`
- Journeys: `1`

| Loop | Type | Family | Baseline | Features | Journeys | Status |
| --- | --- | --- | --- | --- | --- | --- |
| `loop.feature.wake.correctness` | `feature-contract` | `bundle-runtime` | `baseline.with-memd` | `1` | `1` | `active` |
| `loop.feature.resume.correctness` | `feature-contract` | `bundle-runtime` | `baseline.with-memd` | `1` | `1` | `active` |
| `loop.feature.handoff.correctness` | `feature-contract` | `bundle-runtime` | `baseline.with-memd` | `1` | `1` | `active` |
| `loop.feature.attach.correctness` | `feature-contract` | `bundle-runtime` | `baseline.with-memd` | `1` | `1` | `active` |
| `loop.feature.checkpoint.correctness` | `feature-contract` | `capture-compaction` | `baseline.with-memd` | `1` | `1` | `active` |
| `loop.feature.hook-capture.correctness` | `feature-contract` | `capture-compaction` | `baseline.with-memd` | `1` | `1` | `active` |
| `loop.feature.working-context.correctness` | `feature-contract` | `bundle-runtime` | `baseline.with-memd` | `1` | `1` | `active` |
| `loop.feature.working-memory.correctness` | `feature-contract` | `bundle-runtime` | `baseline.with-memd` | `1` | `1` | `active` |
| `loop.journey.resume-handoff-attach.correctness` | `feature-contract` | `bundle-runtime` | `baseline.with-memd` | `8` | `1` | `active` |
| `loop.journey.resume-handoff-attach.no-memd-delta` | `drift-prevention` | `drift-prevention` | `baseline.no-memd` | `8` | `1` | `active` |
| `loop.journey.resume-handoff-attach.drift` | `drift-prevention` | `drift-prevention` | `baseline.no-memd` | `8` | `1` | `active` |

## Coverage Gaps
- `benchmark:unbenchmarked_continuity_feature` [high] bench continuity-critical features before promoting the benchmark registry
  - evidence: feature.bundle.wake [bundle-runtime] coverage=auditing loops=1
  - evidence: feature.bundle.resume [bundle-runtime] coverage=auditing loops=1
  - evidence: feature.bundle.handoff [bundle-runtime] coverage=auditing loops=1
  - evidence: feature.bundle.attach [bundle-runtime] coverage=auditing loops=1
  - evidence: feature.capture.checkpoint [capture-compaction] coverage=auditing loops=1

## Loop Coverage Notes
- `loop.feature.wake.correctness` probes `run wake and inspect the refreshed startup surface` and `change runtime context and confirm wake still surfaces usable state`
- `loop.feature.resume.correctness` probes `run resume and confirm current-task state remains visible` and `add synced noise and confirm durable state survives`
- `loop.feature.handoff.correctness` probes `render a handoff packet and inspect the compact takeover state` and `remove recent context and confirm the packet still has enough evidence`
- `loop.feature.attach.correctness` probes `generate attach output and verify the bundle config is usable` and `change route or workspace and confirm attach output still reflects the bundle`
- `loop.feature.checkpoint.correctness` probes `write a checkpoint and confirm it can be recovered` and `force a noisy session state and confirm the checkpoint still matters`
- `loop.feature.hook-capture.correctness` probes `capture a live turn and inspect the compiled event lane` and `introduce repeated events and confirm the lane stays compact`
- `loop.feature.working-context.correctness` probes `run resume and confirm compact context remains useful` and `add multiple resume-state records and confirm durable facts stay visible`
- `loop.feature.working-memory.correctness` probes `inspect the working memory buffer after resume` and `exceed budget and confirm important records stay visible`
- `loop.journey.resume-handoff-attach.correctness` probes `run the continuity journey and confirm the session resumes cleanly` and `change runtime context and confirm the journey still recovers the current task`
- `loop.journey.resume-handoff-attach.no-memd-delta` probes `compare no-memd and with-memd continuity output for the journey` and `remove live context and confirm memd still improves recovery`
- `loop.journey.resume-handoff-attach.drift` probes `compare current runtime output against the canonical registry` and `mutate docs or runtime defaults and confirm the drift surfaces`

