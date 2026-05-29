# Hive/Hivemind Coordination Feature Proof (Feature 25)

[[ROADMAP]] Secondary/reference doc for feature verification.

Feature id: `feature.hive_hivemind_coordination`

## Honest Status

- Current status: partial
- Proof status: partial
- Dogfood status: ad_hoc
- External status: none
- Blocks 25/25 claim: yes

This proof improves the local, replayable evidence for hive/hivemind coordination,
but it is not a production-reliability or external-auditor proof.

## What the Local Proof Validates

Run:

```bash
bash scripts/verify/feature-hive-hivemind-coordination-proof.sh
```

The proof checks three layers:

1. Current hive authority and capability contracts in docs/scripts:
   - `docs/contracts/hive-live-map-guard.md`
   - `scripts/verify/hive-live-map-guard-contract.sh`
   - `scripts/verify/hive-production-proof.sh`
   - `scripts/dev-server-guard.sh`
   - `scripts/live-state-sync-memd.sh`
   - `scripts/live-state-sync-clawcontrol.sh`
   - `crates/memd-client/src/render/render_summary.rs`
2. Archived coordination artifacts in
   `docs/verification/hive-runs/2026-05-26-internal-alpha`:
   - roster/session records for `queen`, `worker-a`, and `worker-b`
   - distinct capabilities (`coordination`, `memory`, `review`)
   - participant authority and isolated `hive-proof-*` namespace scope
   - targeted inbox delivery, acknowledgement, and handoff packet
   - exclusive-write, help-only, and shared-review task lanes
   - handoff/task/dev-server receipts and claim release evidence
3. Checksum integrity for the archived hive run via `SHA256SUMS.json`.

## No Cross-Agent Leakage Assumption

The proof intentionally validates targeted coordination artifacts rather than
assuming that one agent can read another agent's private context. It checks:

- explicit `from_session`/`to_session` message routing;
- acknowledgement clearing only the target inbox artifact;
- handoff content carrying a user-copyable `next_agent_prompt`;
- task receipts and coordination modes as durable shared artifacts;
- guard text that keeps memd and sibling apps separate.

This means the allowed claim is only that shared memd coordination surfaces can
carry roster, authority, capability, handoff, receipt, and task state. It does
not claim invisible cross-agent memory, private transcript access, or ambient
state sharing.

## Staleness Limit

The archived run is dated `2026-05-26` and is therefore evidence of an ad hoc
local/internal-alpha proof, not sustained dogfood. The proof script revalidates
artifact integrity and current contract text, but it does not rerun the full
hive production exercise or an external canary.

## Allowed Claim

A current static/local proof validates hive coordination contracts and archived
ad hoc artifacts for roster, authority, capability, handoff, task ownership,
help/review lanes, dev-server conflict receipts, and claim release.

## Forbidden Claims

Do not claim production hive/hivemind reliability, continuous dogfood, external
verification, or cross-agent leakage/private-context sharing from this proof.
