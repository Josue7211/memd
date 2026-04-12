> Secondary/reference doc. For current project truth start with [[ROADMAP]] and [[docs/WHERE-AM-I.md|WHERE-AM-I]].

# memd verifiers

- Lane: `nightly`
- Total: `18`

- `verifier.feature.bundle.wake` [feature_contract] fixture=`fixture.continuity_bundle` lanes=`fast,nightly`
- `verifier.feature.session_continuity` [feature_contract] fixture=`fixture.continuity_bundle` lanes=`fast,nightly`
- `verifier.feature.bundle.handoff` [feature_contract] fixture=`fixture.continuity_bundle` lanes=`fast,nightly`
- `verifier.feature.bundle.attach` [feature_contract] fixture=`fixture.continuity_bundle` lanes=`fast,nightly`
- `verifier.feature.capture.checkpoint` [feature_contract] fixture=`fixture.continuity_bundle` lanes=`fast,nightly`
- `verifier.feature.memory.working-context` [feature_contract] fixture=`fixture.continuity_bundle` lanes=`fast,nightly`
- `verifier.feature.memory.working-memory` [feature_contract] fixture=`fixture.continuity_bundle` lanes=`fast,nightly`
- `verifier.feature.memory.procedural-retrieval` [feature_contract] fixture=`fixture.continuity_bundle` lanes=`fast,nightly`
- `verifier.feature.memory.canonical-retrieval` [feature_contract] fixture=`fixture.continuity_bundle` lanes=`nightly,exhaustive`
- `verifier.journey.resume-handoff-attach` [journey] fixture=`fixture.continuity_bundle` lanes=`fast,nightly,exhaustive`
- `verifier.compare.resume-no-memd-vs-with-memd` [comparative] fixture=`fixture.continuity_bundle` lanes=`nightly,comparative,exhaustive`
- `verifier.journey.hive-transfer-assign` [journey] fixture=`fixture.hive-two-session-bundle` lanes=`nightly,exhaustive`
- `verifier.feature.hive.messages-send-ack` [feature_contract] fixture=`fixture.hive-two-session-bundle` lanes=`nightly,exhaustive`
- `verifier.feature.hive.claims-transfer` [feature_contract] fixture=`fixture.hive-two-session-bundle` lanes=`nightly,exhaustive`
- `verifier.feature.hive.tasks-assign` [feature_contract] fixture=`fixture.hive-two-session-bundle` lanes=`nightly,exhaustive`
- `verifier.adversarial.hive-claim-collision` [adversarial] fixture=`fixture.hive-two-session-bundle` lanes=`nightly,exhaustive`
- `verifier.adversarial.hive-task-lane-collision` [adversarial] fixture=`fixture.hive-two-session-bundle` lanes=`nightly,exhaustive`
- `verifier.adversarial.hive-message-lane-collision` [adversarial] fixture=`fixture.hive-two-session-bundle` lanes=`nightly,exhaustive`
