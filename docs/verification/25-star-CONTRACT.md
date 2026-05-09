---
doc: 25-star-contract
status: active
opened: 2026-05-09
depends_on:
  - 1.0.0-CONTRACT.md
  - ../strategy/25-star-master-roadmap.md
  - 25-star-phase-ledger.md
---

# memd 25-Star Contract

> Evidence contract for V21-V35. This does not modify the 10-STAR substrate
> scorecard. V20 remains the maximum substrate score: 10.00 / 10.00.

## Prime Directive

V21-V35 cannot start until honest `1.0.0` close. Synthetic proof may unblock
engineering, but it cannot close any gate that requires elapsed time, real
users, real teams, paying customers, external reviewers, external products, or
independent replay.

## Star Bands

| Band | Versions | Proof type | Close condition |
| --- | --- | --- | --- |
| 10-star | V20 | substrate proof | `1.0.0` gate in `1.0.0-CONTRACT.md` passes. |
| 15-star | V21-V25 | product/company proof | Reliability, teams, integrations, trust, revenue gates pass. |
| 20-star | V26-V30 | network proof | External apps/orgs/builders depend on memd with replayable trust. |
| 25-star | V31-V35 | category proof | memd is durable human/agent institutional memory. |

## Global Close Rules

- Each version must declare one owner and one reviewer before work starts.
- Each version uses A-G phase commits: capability, UX, harness, dogfood,
  external review, kill/recovery decision, final proof packet.
- Every gate needs dated artifacts and commands or reviewer/customer evidence.
- Any missed hard gate creates a `.5` recovery version scoped to the failed
  gate. Recovery versions do not claim expansion credit.
- Roadmap status must distinguish `planned`, `active`, `code_complete`,
  `dogfood_pending`, `external_review_pending`, `closed`, and `recovery`.
- V21+ may not be marked `active` while V20 real gates are open.

## Version Gate Table

| Version | Metric | Artifact path | Kill criterion | Recovery |
| --- | --- | --- | --- | --- |
| V21 Hosted Reliability | 99.9% monthly uptime, restore <15 min, zero data loss | `docs/verification/v21-proof-runs/<date>-hosted-uptime.md` | Manual SSH babysitting required | V21.5 reliability recovery |
| V22 Team Adoption | 5 teams, 30 active days, >=70% weekly retention | `docs/verification/v22-proof-runs/<date>-team-adoption.md` | Setup needs live handholding | V22.5 onboarding recovery |
| V23 Ecosystem Integrations | 10 certified integrations, 3 external maintainers | `docs/verification/v23-proof-runs/<date>-integration-certification.md` | Adapters fork behavior | V23.5 adapter recovery |
| V24 Enterprise Trust | SOC2-lite pack accepted by external reviewer | `docs/verification/v24-proof-runs/<date>-enterprise-trust.md` | Trust story needs promises | V24.5 trust recovery |
| V25 Revenue Engine | 10 paying customers or signed pilots, 3 renew/expand | `docs/verification/v25-proof-runs/<date>-revenue-engine.md` | Users will not pay | V25.5 packaging recovery |
| V26 Network Identity | 3 independent apps resolve same memory identity | `docs/verification/v26-proof-runs/<date>-network-identity.md` | Identity forks per app | V26.5 identity recovery |
| V27 Federation Protocol | 3 orgs exchange approved packets, zero leak findings | `docs/verification/v27-proof-runs/<date>-federation-protocol.md` | Sharing relies on convention | V27.5 isolation recovery |
| V28 Agent Work Market | 25 workflows replay from proof packets | `docs/verification/v28-proof-runs/<date>-agent-work-market.md` | Work packets cannot replay elsewhere | V28.5 replay recovery |
| V29 Default Backend Push | 5 external products use memd as primary memory layer | `docs/verification/v29-proof-runs/<date>-default-backend.md` | Integrations remain demos | V29.5 SDK recovery |
| V30 Network Trust | 100 certified integrations/routines, replay pass >=95% | `docs/verification/v30-proof-runs/<date>-network-trust.md` | Badges cannot be reproduced | V30.5 registry recovery |
| V31 Personal Memory Continuity | 12-month cohort, continuity loss <=2% | `docs/verification/v31-proof-runs/<date>-personal-continuity.md` | Continuity depends on one app | V31.5 portability recovery |
| V32 Institutional Memory | 3 org role transitions, no context loss/access leak | `docs/verification/v32-proof-runs/<date>-institutional-memory.md` | Departing human must curate by hand | V32.5 handoff recovery |
| V33 Legal/Compliance Grade | Legal/security review accepts artifacts without blocker | `docs/verification/v33-proof-runs/<date>-legal-compliance.md` | Compliance uses promises/screenshots | V33.5 artifact recovery |
| V34 Human-Agent Governance | 100 decisions/corrections/delegations replay cleanly | `docs/verification/v34-proof-runs/<date>-governance-replay.md` | Agents act without replayable authority | V34.5 governance recovery |
| V35 Category Completion | 25 paying orgs, 10 external products, 1 year reliability | `docs/verification/v35-proof-runs/<date>-category-completion.md` | memd remains app, not substrate | post-V35 category reset |

## Evidence Rules

### Real-user and real-team gates

Accepted evidence:

- dated cohort table
- consent state
- usage window start/end
- retention metric
- support-load ledger
- anonymized telemetry export when applicable

Not accepted:

- synthetic fixture alone
- local maintainer-only run
- unreviewed screenshots

### External reviewer gates

Accepted evidence:

- reviewer identity or role
- reviewed artifact path
- transcript or written finding list
- pass/fail statement
- blocker list, if any

Not accepted:

- internal self-review
- reviewer-free checklist
- unverifiable verbal claim

### Customer and revenue gates

Accepted evidence:

- signed pilot/customer ledger
- renewal or expansion record
- pricing/package snapshot
- margin model

Not accepted:

- interest list
- demo feedback without commitment
- unpaid internal usage

### Network and category gates

Accepted evidence:

- external app/org/product list
- certification/replay transcript
- signed compatibility badge
- revocation and leakage audit
- independent category review

Not accepted:

- first-party demo integration
- adapter with custom behavior fork
- badge without independent replay

## Atomicity Rules

- Each version opens with A-phase planning commit and closes with G-phase proof
  packet commit.
- Phase boundaries and rollback notes are enumerated in
  `25-star-phase-ledger.md`.
- No A-G phase commit may mix unrelated runtime cleanup.
- If a phase touches code, docs must name rollback command or revert scope.
- If a phase touches docs only, docs must name the evidence gap it resolves.
- Every G-phase packet updates `ROADMAP.md` and links the proof artifact.

## Status Audit

Automated or manual review must fail if:

- V21+ is marked active before V20 closes.
- A gate says `closed` without artifact path.
- A gate closes on synthetic proof where real evidence is required.
- A version lacks kill criterion or recovery rule.
- A phase lacks commit boundary.

## Relationship to Existing Contracts

- `1.0.0-CONTRACT.md` remains binding for V20 and 10-star closure.
- `25-star-master-roadmap.md` explains product/network/category intent.
- This contract defines the proof bar for V21-V35.
- `25-star-phase-ledger.md` defines the atomic A-G execution ledger.

## Audit Command

Run:

```bash
scripts/verify/25-star-roadmap-audit.sh
```

The audit fails on missing V21-V35 metrics, artifact paths, kill criteria,
recovery rules, active-status leaks, synthetic-proof close claims, or missing
A-G rollback boundaries.

## Changelog

- 2026-05-09 opened. Converts 15/20/25-star strategy into evidence-gated
  roadmap contract without changing runtime APIs or 10-STAR substrate scoring.
