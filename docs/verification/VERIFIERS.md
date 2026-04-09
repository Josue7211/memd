# memd verifiers

- Lane: `fast`
- Total: `3`

- `verifier.feature.bundle.resume` [feature_contract] fixture=`fixture.continuity_bundle` lanes=`fast,nightly`
- `verifier.journey.resume-handoff-attach` [journey] fixture=`fixture.continuity_bundle` lanes=`fast,nightly,exhaustive`
- `verifier.compare.resume-no-memd-vs-with-memd` [comparative] fixture=`fixture.continuity_bundle` lanes=`nightly,comparative,exhaustive`
