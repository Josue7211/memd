# memd benchmark coverage

- Current benchmark score: pending live benchmark run
- Feature coverage records: `8`
- Journey coverage records: `1`

## Continuity-Critical Features
- `feature.bundle.wake` [auditing] loops=1 drift=continuity-drift|surface-drift
- `feature.bundle.resume` [auditing] loops=1 drift=continuity-drift|memory-drift
- `feature.bundle.handoff` [auditing] loops=1 drift=continuity-drift|surface-drift
- `feature.bundle.attach` [auditing] loops=1 drift=continuity-drift|harness-drift
- `feature.capture.checkpoint` [auditing] loops=1 drift=continuity-drift|capture-drift
- `feature.capture.hook-capture` [auditing] loops=1 drift=continuity-drift|event-drift
- `feature.memory.working-context` [auditing] loops=1 drift=continuity-drift|memory-drift
- `feature.memory.working-memory` [auditing] loops=1 drift=continuity-drift|memory-drift

## Journeys
- `journey.continuity.startup-to-continuation` [a fresh or resumed session continues without manual reconstruction] features=8 loops=3 gate=acceptable

## Current Benchmark Areas
- `core_memory`: pending
- `retrieval_context`: pending
- `visible_memory`: pending
- `bundle_session`: pending
- `capture_compaction_events`: pending
- `coordination_hive`: pending
- `obsidian`: pending
- `semantic_multimodal`: pending
- `policy_skills_evolution`: pending
- `diagnostics_admin`: pending
