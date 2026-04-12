# memd benchmark loops

- Loops: `21`
- Journeys: `2`

| Loop | Type | Family | Baseline | Features | Journeys | Status |
| --- | --- | --- | --- | --- | --- | --- |
| `loop.feature.wake.correctness` | `feature-contract` | `bundle-runtime` | `baseline.with-memd` | `1` | `1` | `active` |
| `loop.feature.wake.packet-efficiency` | `feature-contract` | `bundle-runtime` | `baseline.with-memd` | `1` | `1` | `active` |
| `loop.feature.resume.correctness` | `feature-contract` | `bundle-runtime` | `baseline.with-memd` | `1` | `1` | `active` |
| `loop.feature.handoff.correctness` | `feature-contract` | `bundle-runtime` | `baseline.with-memd` | `1` | `1` | `active` |
| `loop.feature.attach.correctness` | `feature-contract` | `bundle-runtime` | `baseline.with-memd` | `1` | `1` | `active` |
| `loop.feature.checkpoint.correctness` | `feature-contract` | `capture-compaction` | `baseline.with-memd` | `1` | `1` | `active` |
| `loop.feature.hook-capture.correctness` | `feature-contract` | `capture-compaction` | `baseline.with-memd` | `1` | `1` | `active` |
| `loop.feature.working-context.correctness` | `feature-contract` | `bundle-runtime` | `baseline.with-memd` | `1` | `1` | `active` |
| `loop.feature.working-memory.correctness` | `feature-contract` | `bundle-runtime` | `baseline.with-memd` | `1` | `1` | `active` |
| `loop.feature.procedural-retrieval.correctness` | `feature-contract` | `bundle-runtime` | `baseline.with-memd` | `1` | `1` | `active` |
| `loop.feature.canonical-retrieval.correctness` | `feature-contract` | `bundle-runtime` | `baseline.with-memd` | `1` | `1` | `active` |
| `loop.feature.hive-messages.correctness` | `feature-contract` | `coordination-hive` | `baseline.with-memd` | `1` | `2` | `active` |
| `loop.feature.hive-claims.correctness` | `feature-contract` | `coordination-hive` | `baseline.with-memd` | `1` | `2` | `active` |
| `loop.feature.hive-tasks.correctness` | `feature-contract` | `coordination-hive` | `baseline.with-memd` | `1` | `2` | `active` |
| `loop.journey.resume-handoff-attach.correctness` | `feature-contract` | `bundle-runtime` | `baseline.with-memd` | `8` | `1` | `active` |
| `loop.journey.resume-handoff-attach.no-memd-delta` | `drift-prevention` | `drift-prevention` | `baseline.no-memd` | `8` | `1` | `active` |
| `loop.journey.resume-handoff-attach.drift` | `drift-prevention` | `drift-prevention` | `baseline.no-memd` | `8` | `1` | `active` |
| `loop.journey.hive-transfer-assign.correctness` | `feature-contract` | `coordination-hive` | `baseline.with-memd` | `3` | `1` | `active` |
| `loop.adversarial.hive-claim-collision.containment` | `feature-contract` | `coordination-hive` | `baseline.with-memd` | `1` | `1` | `active` |
| `loop.adversarial.hive-task-lane-collision.containment` | `feature-contract` | `coordination-hive` | `baseline.with-memd` | `1` | `1` | `active` |
| `loop.adversarial.hive-message-lane-collision.containment` | `feature-contract` | `coordination-hive` | `baseline.with-memd` | `1` | `1` | `active` |

## Coverage Gaps
- `benchmark:unbenchmarked_continuity_feature` [high] bench continuity-critical features before promoting the benchmark registry
  - evidence: feature.session_continuity [bundle-runtime] coverage=auditing loops=1
  - evidence: feature.bundle.handoff [bundle-runtime] coverage=auditing loops=1
  - evidence: feature.bundle.attach [bundle-runtime] coverage=auditing loops=1
  - evidence: feature.capture.checkpoint [capture-compaction] coverage=auditing loops=1
  - evidence: feature.capture.hook-capture [capture-compaction] coverage=auditing loops=1

## Loop Coverage Notes
- `loop.feature.wake.correctness` probes `run wake and inspect the refreshed startup surface` and `change runtime context and confirm wake still surfaces usable state`
- `loop.feature.wake.packet-efficiency` probes `render the wake packet and compare core_prompt_tokens to estimated_prompt_tokens` and `add repeated context and confirm the packet still prefers compact durable truth`
- `loop.feature.resume.correctness` probes `run resume and confirm current-task state remains visible` and `add synced noise and confirm durable state survives`
- `loop.feature.handoff.correctness` probes `render a handoff packet and inspect the compact takeover state` and `remove recent context and confirm the packet still has enough evidence`
- `loop.feature.attach.correctness` probes `generate attach output and verify the bundle config is usable` and `change route or workspace and confirm attach output still reflects the bundle`
- `loop.feature.checkpoint.correctness` probes `write a checkpoint and confirm it can be recovered` and `force a noisy session state and confirm the checkpoint still matters`
- `loop.feature.hook-capture.correctness` probes `capture a live turn and inspect the compiled event lane` and `introduce repeated events and confirm the lane stays compact`
- `loop.feature.working-context.correctness` probes `run resume and confirm compact context remains useful` and `add multiple resume-state records and confirm durable facts stay visible`
- `loop.feature.working-memory.correctness` probes `inspect the working memory buffer after resume` and `exceed budget and confirm important records stay visible`
- `loop.feature.procedural-retrieval.correctness` probes `run procedural retrieval and confirm workflow memory remains explicit` and `inject current-task noise and confirm procedural queries remain retrievable`
- `loop.feature.canonical-retrieval.correctness` probes `run canonical retrieval and confirm durable facts are still surfaced` and `change runtime source artifacts and confirm canonical recall still resolves`
- `loop.feature.hive-messages.correctness` probes `send a hive message, read it from the target inbox, and acknowledge it` and `resolve a target session from a shared hive fixture and confirm ack state remains visible`
- `loop.feature.hive-claims.correctness` probes `acquire a scope claim and transfer it to the target hive session` and `move claim ownership across a shared hive fixture and confirm the holder changes`
- `loop.feature.hive-tasks.correctness` probes `upsert a shared task and assign it to a target hive session` and `handoff task ownership across the fixture and confirm the task holder changes`
- `loop.journey.resume-handoff-attach.correctness` probes `run the continuity journey and confirm the session resumes cleanly` and `change runtime context and confirm the journey still recovers the current task`
- `loop.journey.resume-handoff-attach.no-memd-delta` probes `compare no-memd and with-memd continuity output for the journey` and `remove live context and confirm memd still improves recovery`
- `loop.journey.resume-handoff-attach.drift` probes `compare current runtime output against the canonical registry` and `mutate docs or runtime defaults and confirm the drift surfaces`
- `loop.journey.hive-transfer-assign.correctness` probes `send a hive handoff, acknowledge it, transfer the claim, then assign the task to the target session` and `repeat the transfer flow under shared hive state and confirm ownership stays attributable`
- `loop.adversarial.hive-claim-collision.containment` probes `acquire a claim in one hive session and retry acquisition from a competing session` and `confirm the competing session sees the incumbent holder instead of stealing the scope`
- `loop.adversarial.hive-task-lane-collision.containment` probes `seed a colliding target lane and attempt shared task assignment` and `confirm the target lane collision is rejected before ownership moves`
- `loop.adversarial.hive-message-lane-collision.containment` probes `seed a colliding target lane and attempt a direct handoff message send` and `confirm the target lane collision is rejected before the message routes to the stale target session`

