# memd benchmark coverage

- Current benchmark score: `95/100`
- Feature coverage records: `11`
- Journey coverage records: `2`

## Coverage Summary
- continuity-critical total: `11`
- continuity-critical benchmarked: `0`
- missing loop count: `11`
- with-memd losses: `0`

## Continuity-Critical Features
- `feature.bundle.attach` [auditing] loops=1 drift=continuity-drift|harness-drift
- `feature.bundle.handoff` [auditing] loops=1 drift=continuity-drift|surface-drift
- `feature.bundle.resume` [auditing] loops=1 drift=continuity-drift|memory-drift
- `feature.bundle.wake` [auditing] loops=1 drift=continuity-drift|surface-drift
- `feature.capture.checkpoint` [auditing] loops=1 drift=continuity-drift|capture-drift
- `feature.capture.hook-capture` [auditing] loops=1 drift=continuity-drift|event-drift
- `feature.hive.claims` [auditing] loops=1 drift=continuity-drift|surface-drift
- `feature.hive.messages` [auditing] loops=1 drift=continuity-drift|surface-drift
- `feature.hive.tasks` [auditing] loops=1 drift=continuity-drift|surface-drift
- `feature.memory.working-context` [auditing] loops=1 drift=continuity-drift|memory-drift
- `feature.memory.working-memory` [auditing] loops=1 drift=continuity-drift|memory-drift

## Benchmark Gaps
- `benchmark:unbenchmarked_continuity_feature` [high] bench continuity-critical features before promoting the benchmark registry

## Missing Loop IDs
- none

## Journeys
- `journey.continuity.resume-handoff-attach` [a resumed or attached session continues without manual reconstruction] features=8 loops=3 gate=acceptable
- `journey.hive.transfer-assign` [a live hive handoff moves message intent, scope ownership, and task ownership onto the target session] features=3 loops=2 gate=acceptable

## Current Benchmark Areas
- `core_memory`: `93/100`
- `retrieval_context`: `97/100`
- `visible_memory`: `100/100`
- `bundle_session`: `95/100`
- `shared_continuity`: `100/100`
- `capture_compaction_events`: `83/100`
- `coordination_hive`: `88/100`
- `obsidian`: `95/100`
- `semantic_multimodal`: `97/100`
- `policy_skills_evolution`: `100/100`
- `diagnostics_admin`: `100/100`

