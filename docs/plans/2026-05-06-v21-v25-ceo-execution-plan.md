# V21-V25 CEO Execution Plan

Status: strategy plan; deferred until honest `1.0.0` close.

## Prime Directive

Do not start V21 before `1.0.0` closes with real V14-V20 evidence. Until then,
all engineering remains evidence-blocker fixes only.

## CEO Dashboard

| Version | Outcome | CEO metric | Earliest start | Kill criterion |
| --- | --- | --- | --- | --- |
| V21 | Hosted reliability | 99.9% monthly uptime | after 1.0.0 | manual SSH babysitting required |
| V22 | Team adoption | 5 teams, 30 active days, >=70% weekly retention | after V21 gate | setup needs live handholding |
| V23 | Ecosystem integrations | 10 certified integrations with replay proof | after V22 gate | adapters fork behavior |
| V24 | Enterprise trust | SOC2-lite pack accepted by external reviewer | after V23 gate | trust story needs promises |
| V25 | Revenue engine | 10 paying customers or signed pilots | after V24 gate | users will not pay |

## V21 Plan: Hosted Reliability

Owner: platform/ops.

Build:
- hosted control plane
- backup/restore drill automation
- migration runbooks
- incident review template
- support bundle export

Proof artifacts:
- `docs/verification/v21-proof-runs/<date>-hosted-uptime.md`
- uptime log export
- restore drill transcript
- incident review packet, if any incident occurs

Gate:
- 30-day hosted window
- 99.9% uptime
- restore under 15 minutes
- zero data loss

Decision:
- pass -> V22
- fail -> V21.5 reliability recovery

## V22 Plan: Team Adoption

Owner: product/adoption.

Build:
- guided team onboarding
- admin seats and roles
- invite/revoke flow
- team evidence dashboard
- support docs

Proof artifacts:
- `docs/verification/v22-proof-runs/<date>-team-adoption.md`
- cohort table for 5 teams
- retention export
- support-load ledger

Gate:
- 5 teams onboarded
- 30 active days
- >=70% weekly retention
- 3 teams complete correction + replay + sync workflow
- support load <30 minutes/team/week

Decision:
- pass -> V23
- fail -> onboarding recovery before integrations

## V23 Plan: Ecosystem Integrations

Owner: ecosystem.

Build:
- certification kit
- harness compatibility matrix
- versioned adapter contract
- partner examples
- public replay badges

Proof artifacts:
- `docs/verification/v23-proof-runs/<date>-integration-certification.md`
- compatibility matrix
- replay transcripts for each certified integration
- external maintainer contribution log

Gate:
- 10 integrations certified
- 3 external maintainers contribute fixes
- adapter upgrade does not break prior certified integrations

Decision:
- pass -> V24
- fail -> freeze adapters and harden contract

## V24 Plan: Enterprise Trust

Owner: security/compliance.

Build:
- security review packet
- data retention controls
- delete/export workflows
- admin compliance reports
- threat model refresh

Proof artifacts:
- `docs/verification/v24-proof-runs/<date>-enterprise-trust.md`
- threat model
- retention/delete/export transcripts
- external review letter or issue list

Gate:
- external review has no blocker severity findings
- retention/delete/export verified
- audit replay accepted by reviewer

Decision:
- pass -> V25
- fail -> trust recovery before revenue

## V25 Plan: Revenue Engine

Owner: GTM/customer success.

Build:
- pricing/package validation
- billing hooks
- customer success loop
- churn reason capture
- sales demo proof pack

Proof artifacts:
- `docs/verification/v25-proof-runs/<date>-revenue-engine.md`
- customer/pilot ledger
- renewal/expansion evidence
- infra/support margin model

Gate:
- 10 paying customers or signed pilots
- 3 renew or expand after first cycle
- support + infra margin documented

Decision:
- pass -> post-V25 scale plan
- fail -> packaging/pricing reset, no more feature expansion

## Operating Cadence

- Weekly CEO review: evidence, blockers, metric trend, kill criteria.
- Monthly strategy review: keep / stop / recover / advance.
- Every version gets one owner and one reviewer before work starts.
- Every gate needs real artifacts, not synthetic proof.

## Staffing Order

1. Platform owner for V21.
2. Adoption owner for V22.
3. Ecosystem owner for V23.
4. Security/compliance reviewer for V24.
5. GTM/customer success owner for V25.

## Current Action

Stay on V20 evidence ops. Assign V21-V25 provisional owners only after V19
auditor and V20 replay reviewer are staffed.
