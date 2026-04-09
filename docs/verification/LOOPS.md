# memd benchmark loops

- Loops: `9`
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
| `loop.journey.startup-to-continuation.drift` | `drift-prevention` | `drift-prevention` | `baseline.no-memd` | `8` | `1` | `active` |

## Loop Coverage Notes
- `loop.feature.wake.correctness` probes `run wake and inspect the refreshed startup surface` and `change runtime context and confirm wake still surfaces usable state`
- `loop.feature.resume.correctness` probes `restore usable continuity after turn restart` and `inject stale state and ensure resume rejects it`
- `loop.feature.handoff.correctness` probes `emit a compact takeover packet` and `drop key state to ensure the packet still flags the gap`
- `loop.feature.attach.correctness` probes `configure a supported client from the bundle` and `compare the snippet against live runtime behavior`
- `loop.feature.checkpoint.correctness` probes `write current task state into the live backend` and `verify the checkpoint state survives resume`
- `loop.feature.hook-capture.correctness` probes `record live turn changes and refresh bundle truth` and `introduce a stale turn event and confirm the compiled truth updates`
- `loop.feature.working-context.correctness` probes `keep compact context usable` and `verify current-task facts are not crowded out`
- `loop.feature.working-memory.correctness` probes `keep budgeted state useful under pressure` and `stress eviction to ensure important records survive`
- `loop.journey.startup-to-continuation.drift` probes `a fresh or resumed session continues without manual reconstruction` and `the registry, runtime, and compiled surfaces stay aligned`
