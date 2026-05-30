# Hive/Hivemind Coordination Feature Proof (Feature 25)

[[ROADMAP]] Secondary/reference doc for feature verification.

Feature id: `feature.hive_hivemind_coordination`

## Honest Status

- Current status: partial
- Proof status: strong
- Dogfood status: ad_hoc
- External status: none
- Blocks 25/25 claim: yes

This proof provides strong local, replayable evidence for hive/hivemind coordination by combining static contract checks, archived-run integrity checks, executable-mode validation, and the isolated local hive production proof. It is still not an external-auditor proof or a sustained production-reliability claim.

## What the Local Proof Validates

Run:

```bash
bash scripts/verify/feature-hive-hivemind-coordination-proof.sh
```

The proof checks five local layers:

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
3. Executable/run-mode hygiene for hive proof and guard scripts.
4. Checksum integrity for the archived hive run via `SHA256SUMS.json`.
5. Local production rehearsal through `scripts/verify/hive-production-proof.sh` (without the optional external/shared Tailscale canary).

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

The archived run is dated `2026-05-26` and remains evidence of an ad hoc
local/internal-alpha run, not sustained dogfood. The local proof now also
reruns the isolated hive production exercise and current static guards, but it
does not run the optional external/shared Tailscale canary and does not replace
independent external review.

## Allowed Claim

A current strong local proof validates hive coordination contracts, executable
script modes, live-map separation guards, an isolated hive production rehearsal,
and archived ad hoc artifacts for roster, authority, capability, handoff, task
ownership, help/review lanes, dev-server conflict receipts, and claim release.

## Forbidden Claims

Do not claim sustained production hive/hivemind reliability, continuous
dogfood, external verification, optional shared/Tailscale canary coverage, or
cross-agent leakage/private-context sharing from this local proof.
