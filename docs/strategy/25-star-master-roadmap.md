---
doc: 25-star-master-roadmap
status: strategy
opened: 2026-05-09
depends_on:
  - ../verification/1.0.0-CONTRACT.md
  - ../verification/25-star-CONTRACT.md
  - v21-v25-ceo-mode.md
---

# 25-Star memd Master Roadmap

> Strategy contract. For active project truth, start with `ROADMAP.md`.
> This document does not activate V21+. V20 evidence ops remains current until
> honest `1.0.0` close.

## Meaning of the Stars

The 10-star scorecard remains the substrate ceiling. Stars beyond 10 do not
raise any 10-STAR axis above 10/10. They measure deployment, adoption,
network, governance, and category maturity.

| Star band | Meaning | Version band | Completion claim |
| --- | --- | --- | --- |
| 10-star | Perfect memory substrate | V20 / `1.0.0` | Any harness remembers right, with proof. |
| 15-star | Perfect product/company wedge | V21-V25 | Teams can run, trust, integrate, and pay for memd. |
| 20-star | Memory network | V26-V30 | Tools, orgs, and external builders build on memd. |
| 25-star | Durable memory layer | V31-V35 | memd becomes portable human/agent institutional memory. |

## Current Rule

Do not start V21 before `1.0.0` closes with dated V14-V20 evidence. Until
then, all engineering remains limited to evidence-blocker fixes.

V20 close means:

- 3 real users and 3 harness-user pairs.
- 3 devices on current `main`.
- V14/V15/V16/V17/V18 elapsed dogfood windows complete.
- V19 external auditor artifacts complete.
- V20 third-party replay complete.
- Weekly evidence notes present.
- `1.0.0` tag cut only after all real gates pass.

## Atomic Execution Shape

Every V21-V35 version uses the same A-G phase shell. One commit per phase.
Each phase must be independently revertable.

| Phase | Purpose | Commit rule |
| --- | --- | --- |
| A | Substrate/product capability | Adds the minimum durable primitive. |
| B | Operator/user experience | Makes the primitive usable without maintainer help. |
| C | Evidence harness | Adds repeatable proof and failure fixtures. |
| D | Dogfood window | Starts or closes real usage window with dated artifacts. |
| E | External review | Adds outside reviewer, maintainer, customer, or org proof. |
| F | Kill/recovery decision | Explicit pass/fail, recovery plan if gate misses. |
| G | Final proof packet | Freezes artifacts, updates roadmap, prepares next version. |

No version advances on synthetic proof alone if its gate requires users,
customers, third parties, devices, orgs, or elapsed time.

## V21-V25: 15-Star Product Company

### V21 Hosted Reliability

Goal: memd becomes boring to run.

Build:

- Hosted control plane.
- Backup/restore drill automation.
- Migration runbooks.
- Incident review template.
- Support bundle export.

Gate:

- 30-day hosted reliability window.
- 99.9% monthly uptime.
- Restore under 15 minutes.
- Zero data loss.

Kill criterion: manual SSH babysitting blocks V22.

Recovery: V21.5 reliability recovery, scoped only to uptime, restore, and data
loss blockers.

### V22 Team Adoption

Goal: small teams use memd daily without founder support.

Build:

- Guided team onboarding.
- Admin seats and roles.
- Invite/revoke flow.
- Team evidence dashboard.
- Support docs.

Gate:

- 5 real teams onboarded.
- 30 active days.
- >=70% weekly retention.
- 3 teams complete correction + replay + sync workflow.
- Support load <30 minutes per team per week.

Kill criterion: setup requiring live handholding blocks V23.

Recovery: V22.5 onboarding recovery, scoped to setup, docs, and support load.

### V23 Ecosystem Integrations

Goal: memd becomes the default memory backend for harnesses.

Build:

- Integration certification kit.
- Harness compatibility matrix.
- Versioned adapter contract.
- Partner examples.
- Public replay badges.

Gate:

- 10 certified integrations.
- 3 external maintainers contribute fixes.
- Adapter upgrade does not break prior certified integrations.

Kill criterion: adapter behavior forks block V24.

Recovery: V23.5 adapter contract recovery, scoped to certification and
compatibility.

### V24 Enterprise Trust

Goal: buyers can approve memd.

Build:

- Security review packet.
- Data retention controls.
- Delete/export workflows.
- Admin compliance reports.
- Threat model refresh.

Gate:

- SOC2-lite pack accepted by an external reviewer.
- Retention/delete/export verified.
- Audit replay accepted by reviewer.
- No blocker-severity external findings.

Kill criterion: trust story needing promises instead of artifacts blocks V25.

Recovery: V24.5 trust recovery, scoped to reviewer findings.

### V25 Revenue Engine

Goal: prove memd can sustain itself.

Build:

- Pricing/package validation.
- Billing hooks.
- Customer success loop.
- Churn reason capture.
- Sales demo proof pack.

Gate:

- 10 paying customers or signed pilots.
- 3 renew or expand after first cycle.
- Support + infra margin documented.

Kill criterion: users will not pay for the package.

Recovery: V25.5 packaging/pricing reset, no feature expansion.

## V26-V30: 20-Star Memory Network

### V26 Network Identity

Goal: one portable user/org memory identity works across apps.

Build:

- User/org identity model.
- App authorization grants.
- Scoped memory identity resolver.
- Revocation and transfer receipts.

Gate:

- 3 independent apps authenticate and resolve the same memory identity.
- Zero cross-app private leakage in adversarial test.
- Revocation takes effect in every app within the proof window.

Kill criterion: identity requires app-specific forks.

Recovery: V26.5 identity contract recovery.

### V27 Federation Protocol

Goal: orgs can exchange approved memory packets safely.

Build:

- Cross-org sharing protocol.
- Scoped revocation.
- Audit receipts.
- Federation compatibility tests.

Gate:

- 3 orgs exchange approved memory packets.
- Zero private leak findings.
- Revoked packet is rejected by every participant.

Kill criterion: cross-org sharing depends on trust-by-convention.

Recovery: V27.5 federation isolation recovery.

### V28 Agent Work Market

Goal: routines, claims, and tasks become portable agent work units.

Build:

- Work-unit package format.
- Claim/task replay proof.
- Routine execution receipts.
- Marketplace trust metadata.

Gate:

- 25 agent-delivered workflows replay from proof packets.
- At least 5 workflows are contributed by external builders.
- Failed workflow leaves auditable reason and rollback path.

Kill criterion: work packets cannot replay outside the original harness.

Recovery: V28.5 work-unit replay recovery.

### V29 Default Backend Push

Goal: external tools can choose memd as the default memory backend.

Build:

- SDKs/plugins for priority toolchains.
- Default-backend integration guides.
- Compatibility fixtures.
- Upgrade policy.

Gate:

- 5 external products integrate memd as primary memory layer.
- Each product passes replay and visibility tests.
- One minor upgrade preserves all certified behavior.

Kill criterion: integrations remain demos, not primary backends.

Recovery: V29.5 SDK/product integration recovery.

### V30 Network Trust

Goal: the memory network has public trust infrastructure.

Build:

- Public compatibility registry.
- Signed replay badges.
- Integration/routine audit records.
- Independent replay service.

Gate:

- 100 certified integrations or routines.
- Independent replay pass rate >=95%.
- Registry entries carry signed provenance and revocation state.

Kill criterion: badges cannot be independently reproduced.

Recovery: V30.5 registry/replay recovery.

## V31-V35: 25-Star Durable Memory Layer

### V31 Personal Memory Continuity

Goal: personal memory survives tool changes over years.

Build:

- Multi-year personal memory model.
- Explicit forgetting and retention policy.
- Tool migration proof.
- Personal export/import guarantees.

Gate:

- 12-month user cohort migrates tools with continuity loss <=2%.
- Forgetting requests remove memory from every active surface.
- Export/import reproduces recall behavior within tolerance.

Kill criterion: continuity depends on a single app staying installed.

Recovery: V31.5 continuity portability recovery.

### V32 Institutional Memory

Goal: org memory survives turnover with permissions intact.

Build:

- Role transition workflows.
- Org memory stewardship roles.
- Offboarding/onboarding replay.
- Permission-preserving knowledge transfer.

Gate:

- 3 orgs complete role transition.
- No context loss beyond documented intentional redaction.
- No access leak during offboarding/onboarding replay.

Kill criterion: org continuity requires the departing human to curate by hand.

Recovery: V32.5 institutional handoff recovery.

### V33 Legal and Compliance Grade

Goal: deletion, export, retention, and audit satisfy real buyer review.

Build:

- Legal-grade retention controls.
- Deletion proof.
- Export proof.
- Buyer audit packet.

Gate:

- External legal/security review accepts artifacts without blocker.
- Deletion/export/retention proofs replay from clean environment.
- Compliance report maps every claim to evidence.

Kill criterion: compliance rests on promises or manual screenshots.

Recovery: V33.5 compliance artifact recovery.

### V34 Human-Agent Governance

Goal: humans and agents share a decision ledger.

Build:

- Contested-decision ledger.
- Delegation receipts.
- Correction and override workflow.
- Governance replay harness.

Gate:

- 100 contested decisions, corrections, and delegations replay cleanly.
- Human override always wins within scoped authority.
- Agent delegation chain is explainable end to end.

Kill criterion: agents can act without replayable authority.

Recovery: V34.5 governance replay recovery.

### V35 Category Completion

Goal: memd is a memory substrate, not a memory app.

Build:

- Category proof packet.
- Long-window reliability export.
- External product and org proof.
- Independent category review.

Gate:

- 25 paying orgs.
- 10 external products use memd as primary memory layer.
- 1 year reliability record meets published SLO.
- Independent category review accepts the substrate claim.

Kill criterion: memd remains a strong app instead of default substrate.

Recovery: post-V35 category reset; no more expansion until positioning and
proof mismatch is resolved.

## Operating Cadence

- Weekly evidence review during active version windows.
- Monthly CEO review across adoption, reliability, trust, and revenue.
- Every version has exactly one owner and one reviewer.
- Every gate has artifact paths before work starts.
- Every miss creates a `.5` recovery phase instead of lowering the bar.

## Artifact Layout

Use these directories for future proof packets:

- `docs/verification/v21-proof-runs/`
- `docs/verification/v22-proof-runs/`
- `docs/verification/v23-proof-runs/`
- `docs/verification/v24-proof-runs/`
- `docs/verification/v25-proof-runs/`
- `docs/verification/v26-proof-runs/`
- `docs/verification/v27-proof-runs/`
- `docs/verification/v28-proof-runs/`
- `docs/verification/v29-proof-runs/`
- `docs/verification/v30-proof-runs/`
- `docs/verification/v31-proof-runs/`
- `docs/verification/v32-proof-runs/`
- `docs/verification/v33-proof-runs/`
- `docs/verification/v34-proof-runs/`
- `docs/verification/v35-proof-runs/`

Do not create empty proof directories until a version starts.

## First Action After `1.0.0`

Open V21 only after V20 final proof bundle lands. The first V21 commit should
create `docs/verification/v21-proof-runs/`, name the V21 owner/reviewer, and
start the 30-day hosted reliability clock.
