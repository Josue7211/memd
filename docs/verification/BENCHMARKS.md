# memd benchmark registry

- Root: ``
- Registry: `docs/verification/benchmark-registry.json`
- Version: `v1`
- App goal: memd as a seamless memory control plane with near-perfect continuity
- Current benchmark score: `94/100`
- Quality dimensions: `5`
- Pillars: `1`
- Families: `3`
- Features: `8`
- Journeys: `1`
- Loops: `11`
- Scorecards: `0`
- Evidence records: `0`
- Gates: `0`
- Baseline modes: `2`
- Runtime policies: `4`

## Pillars
- `memory-continuity`: 3 family surfaces, 8 features

## Feature Coverage Snapshot
- `feature.bundle.wake` [bundle-runtime] auditing | continuity=true | loops=1
- `feature.bundle.resume` [bundle-runtime] auditing | continuity=true | loops=1
- `feature.bundle.handoff` [bundle-runtime] auditing | continuity=true | loops=1
- `feature.bundle.attach` [bundle-runtime] auditing | continuity=true | loops=1
- `feature.capture.checkpoint` [capture-compaction] auditing | continuity=true | loops=1
- `feature.capture.hook-capture` [capture-compaction] auditing | continuity=true | loops=1
- `feature.memory.working-context` [bundle-runtime] auditing | continuity=true | loops=1
- `feature.memory.working-memory` [bundle-runtime] auditing | continuity=true | loops=1

## Quality Dimensions
- `continuity` weight `25`
- `correctness` weight `20`
- `reliability` weight `15`
- `drift_resistance` weight `15`
- `token_efficiency` weight `10`

