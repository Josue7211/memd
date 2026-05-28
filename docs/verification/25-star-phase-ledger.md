---
doc: 25-star-phase-ledger
status: planned
opened: 2026-05-09
depends_on:
  - 25-star-CONTRACT.md
  - ../strategy/25-star-master-roadmap.md
---

Secondary/reference doc. Start from [[ROADMAP]] for project truth.

# 25-Star Phase Ledger

This is the atomic execution ledger for V21-V35. It does not activate V21+.
Every listed phase is a future commit boundary, and every version remains
blocked until V20 closes through the `1.0.0` contract.

Rules:

- One commit per phase: A, B, C, D, E, F, G.
- Each phase commit must be revertable without reverting unrelated work.
- If a hard gate misses, stop the version and open the listed `.5` recovery.
- Synthetic proof can exercise harnesses, but cannot close real-user,
  customer, org, reviewer, external-product, or elapsed-time gates.

## Version Phase Map

| Version | A capability | B UX | C harness | D dogfood | E external review | F kill/recovery | G proof packet |
| --- | --- | --- | --- | --- | --- | --- | --- |
| V21 | Hosted control plane, backups, restore drills | support bundle and runbooks | uptime/restore/data-loss harness | 30-day hosted window | ops reviewer | V21.5 if uptime/restore/data loss miss | hosted reliability proof |
| V22 | seats, roles, invites, revoke | team onboarding and dashboard | retention/support-load harness | 5-team 30-day window | team admin review | V22.5 if handholding/retention miss | team adoption proof |
| V23 | certification kit and adapter contract | partner examples and badges | adapter compatibility harness | integration dogfood | external maintainer review | V23.5 if adapters fork | integration proof |
| V24 | retention/delete/export/compliance controls | admin compliance reports | delete/export/audit harness | buyer-trust dogfood | security/legal reviewer | V24.5 if blockers remain | enterprise trust proof |
| V25 | billing hooks and package model | customer success loop | revenue/margin ledger harness | pilot/customer cycle | customer review | V25.5 if no payment signal | revenue proof |
| V26 | portable user/org memory identity | grant/revoke UI | identity leakage harness | 3-app dogfood | app maintainer review | V26.5 if identity forks | network identity proof |
| V27 | federation packet protocol | sharing/revoke receipts | cross-org leak harness | 3-org exchange | org reviewer | V27.5 if trust-by-convention remains | federation proof |
| V28 | portable work-unit package | marketplace trust surface | workflow replay harness | 25 workflow dogfood | external builder review | V28.5 if replay fails elsewhere | work market proof |
| V29 | SDKs/plugins for primary toolchains | default-backend guides | product compatibility harness | 5-product dogfood | product maintainer review | V29.5 if integrations stay demos | default backend proof |
| V30 | compatibility registry and signed badges | public badge/replay surface | independent replay harness | 100-item registry window | independent replay review | V30.5 if badges fail reproduce | network trust proof |
| V31 | multi-year personal continuity model | forgetting/migration controls | continuity-loss harness | 12-month cohort | user/tool reviewer | V31.5 if continuity app-bound | personal continuity proof |
| V32 | role-transition org memory model | permissioned handoff surface | access-leak/context-loss harness | 3-org transition | org/security review | V32.5 if hand curation needed | institutional memory proof |
| V33 | legal retention/export/delete/audit pack | buyer-ready compliance room | legal/security artifact harness | buyer review window | legal/security reviewer | V33.5 if blockers remain | compliance proof |
| V34 | human-agent decision ledger | correction/delegation review UX | 100-case replay harness | governance dogfood | governance reviewer | V34.5 if authority cannot replay | governance proof |
| V35 | category substrate packaging | category review surface | 1-year reliability/category harness | paying-org/product window | independent category review | post-V35 reset if still app-like | category completion proof |

## Commit Boundary Ledger

### V21 Hosted Reliability

Artifact root: `docs/verification/v21-proof-runs/`

| Phase | Commit boundary | Evidence artifact | Rollback / recovery note |
| --- | --- | --- | --- |
| A | `v21-a-hosted-control-plane` | `A-capability.md` | revert hosted-control-plane files only |
| B | `v21-b-support-bundle-ux` | `B-ux.md` | revert support bundle/runbook surface |
| C | `v21-c-uptime-restore-harness` | `C-harness.md` | revert harness; keep no status credit |
| D | `v21-d-hosted-dogfood-window` | `D-dogfood.md` | failed window opens V21.5 |
| E | `v21-e-ops-review` | `E-external-review.md` | reviewer blockers open V21.5 |
| F | `v21-f-kill-recovery-decision` | `F-decision.md` | declare pass/fail before G |
| G | `v21-g-proof-packet` | `G-final-proof.md` | updates ROADMAP only if gate passes |

### V22 Team Adoption

Artifact root: `docs/verification/v22-proof-runs/`

| Phase | Commit boundary | Evidence artifact | Rollback / recovery note |
| --- | --- | --- | --- |
| A | `v22-a-team-roles-invites` | `A-capability.md` | revert seats/roles/invite primitive |
| B | `v22-b-team-onboarding-dashboard` | `B-ux.md` | revert onboarding/dashboard surface |
| C | `v22-c-retention-support-harness` | `C-harness.md` | revert harness; no adoption credit |
| D | `v22-d-team-dogfood-window` | `D-dogfood.md` | failed cohort opens V22.5 |
| E | `v22-e-team-admin-review` | `E-external-review.md` | reviewer blockers open V22.5 |
| F | `v22-f-kill-recovery-decision` | `F-decision.md` | decide handholding/retention fate |
| G | `v22-g-proof-packet` | `G-final-proof.md` | updates ROADMAP only if gate passes |

### V23 Ecosystem Integrations

Artifact root: `docs/verification/v23-proof-runs/`

| Phase | Commit boundary | Evidence artifact | Rollback / recovery note |
| --- | --- | --- | --- |
| A | `v23-a-certification-kit-contract` | `A-capability.md` | revert kit/contract changes |
| B | `v23-b-partner-examples-badges` | `B-ux.md` | revert examples/badge surface |
| C | `v23-c-adapter-compat-harness` | `C-harness.md` | revert harness; no certification credit |
| D | `v23-d-integration-dogfood` | `D-dogfood.md` | failed integrations open V23.5 |
| E | `v23-e-maintainer-review` | `E-external-review.md` | maintainer blockers open V23.5 |
| F | `v23-f-kill-recovery-decision` | `F-decision.md` | decide adapter-fork fate |
| G | `v23-g-proof-packet` | `G-final-proof.md` | updates ROADMAP only if gate passes |

### V24 Enterprise Trust

Artifact root: `docs/verification/v24-proof-runs/`

| Phase | Commit boundary | Evidence artifact | Rollback / recovery note |
| --- | --- | --- | --- |
| A | `v24-a-retention-delete-export` | `A-capability.md` | revert trust controls only |
| B | `v24-b-compliance-report-ux` | `B-ux.md` | revert report surface |
| C | `v24-c-trust-harness` | `C-harness.md` | revert harness; no trust credit |
| D | `v24-d-buyer-trust-dogfood` | `D-dogfood.md` | failed review prep opens V24.5 |
| E | `v24-e-security-legal-review` | `E-external-review.md` | blocker findings open V24.5 |
| F | `v24-f-kill-recovery-decision` | `F-decision.md` | decide blocker severity |
| G | `v24-g-proof-packet` | `G-final-proof.md` | updates ROADMAP only if gate passes |

### V25 Revenue Engine

Artifact root: `docs/verification/v25-proof-runs/`

| Phase | Commit boundary | Evidence artifact | Rollback / recovery note |
| --- | --- | --- | --- |
| A | `v25-a-billing-packaging` | `A-capability.md` | revert billing/package hooks |
| B | `v25-b-customer-success-loop` | `B-ux.md` | revert success-loop surface |
| C | `v25-c-revenue-margin-harness` | `C-harness.md` | revert ledger harness |
| D | `v25-d-pilot-customer-cycle` | `D-dogfood.md` | no pay signal opens V25.5 |
| E | `v25-e-customer-review` | `E-external-review.md` | customer blockers open V25.5 |
| F | `v25-f-kill-recovery-decision` | `F-decision.md` | decide package viability |
| G | `v25-g-proof-packet` | `G-final-proof.md` | updates ROADMAP only if gate passes |

### V26 Network Identity

Artifact root: `docs/verification/v26-proof-runs/`

| Phase | Commit boundary | Evidence artifact | Rollback / recovery note |
| --- | --- | --- | --- |
| A | `v26-a-portable-identity` | `A-capability.md` | revert identity resolver/grants |
| B | `v26-b-grant-revoke-ux` | `B-ux.md` | revert grant/revoke surface |
| C | `v26-c-leakage-harness` | `C-harness.md` | revert leakage harness |
| D | `v26-d-three-app-dogfood` | `D-dogfood.md` | app forks open V26.5 |
| E | `v26-e-app-maintainer-review` | `E-external-review.md` | maintainer blockers open V26.5 |
| F | `v26-f-kill-recovery-decision` | `F-decision.md` | decide identity contract fate |
| G | `v26-g-proof-packet` | `G-final-proof.md` | updates ROADMAP only if gate passes |

### V27 Federation Protocol

Artifact root: `docs/verification/v27-proof-runs/`

| Phase | Commit boundary | Evidence artifact | Rollback / recovery note |
| --- | --- | --- | --- |
| A | `v27-a-federation-packets` | `A-capability.md` | revert protocol files |
| B | `v27-b-sharing-receipts-ux` | `B-ux.md` | revert sharing UI/CLI surface |
| C | `v27-c-cross-org-leak-harness` | `C-harness.md` | revert harness; no federation credit |
| D | `v27-d-three-org-exchange` | `D-dogfood.md` | leak findings open V27.5 |
| E | `v27-e-org-review` | `E-external-review.md` | org blockers open V27.5 |
| F | `v27-f-kill-recovery-decision` | `F-decision.md` | decide isolation fate |
| G | `v27-g-proof-packet` | `G-final-proof.md` | updates ROADMAP only if gate passes |

### V28 Agent Work Market

Artifact root: `docs/verification/v28-proof-runs/`

| Phase | Commit boundary | Evidence artifact | Rollback / recovery note |
| --- | --- | --- | --- |
| A | `v28-a-work-unit-package` | `A-capability.md` | revert work-unit format |
| B | `v28-b-marketplace-trust-ux` | `B-ux.md` | revert market trust surface |
| C | `v28-c-workflow-replay-harness` | `C-harness.md` | revert replay harness |
| D | `v28-d-workflow-dogfood` | `D-dogfood.md` | replay failures open V28.5 |
| E | `v28-e-builder-review` | `E-external-review.md` | builder blockers open V28.5 |
| F | `v28-f-kill-recovery-decision` | `F-decision.md` | decide portability fate |
| G | `v28-g-proof-packet` | `G-final-proof.md` | updates ROADMAP only if gate passes |

### V29 Default Backend Push

Artifact root: `docs/verification/v29-proof-runs/`

| Phase | Commit boundary | Evidence artifact | Rollback / recovery note |
| --- | --- | --- | --- |
| A | `v29-a-sdks-plugins` | `A-capability.md` | revert SDK/plugin surface |
| B | `v29-b-default-backend-guides` | `B-ux.md` | revert guides/examples |
| C | `v29-c-product-compat-harness` | `C-harness.md` | revert compatibility harness |
| D | `v29-d-five-product-dogfood` | `D-dogfood.md` | demo-only result opens V29.5 |
| E | `v29-e-product-maintainer-review` | `E-external-review.md` | maintainer blockers open V29.5 |
| F | `v29-f-kill-recovery-decision` | `F-decision.md` | decide default-backend viability |
| G | `v29-g-proof-packet` | `G-final-proof.md` | updates ROADMAP only if gate passes |

### V30 Network Trust

Artifact root: `docs/verification/v30-proof-runs/`

| Phase | Commit boundary | Evidence artifact | Rollback / recovery note |
| --- | --- | --- | --- |
| A | `v30-a-compat-registry-badges` | `A-capability.md` | revert registry/badge schema |
| B | `v30-b-public-replay-surface` | `B-ux.md` | revert public replay surface |
| C | `v30-c-independent-replay-harness` | `C-harness.md` | revert replay harness |
| D | `v30-d-registry-window` | `D-dogfood.md` | pass rate miss opens V30.5 |
| E | `v30-e-independent-replay-review` | `E-external-review.md` | reviewer blockers open V30.5 |
| F | `v30-f-kill-recovery-decision` | `F-decision.md` | decide badge reproducibility |
| G | `v30-g-proof-packet` | `G-final-proof.md` | updates ROADMAP only if gate passes |

### V31 Personal Memory Continuity

Artifact root: `docs/verification/v31-proof-runs/`

| Phase | Commit boundary | Evidence artifact | Rollback / recovery note |
| --- | --- | --- | --- |
| A | `v31-a-personal-continuity-model` | `A-capability.md` | revert continuity model |
| B | `v31-b-forgetting-migration-ux` | `B-ux.md` | revert forgetting/migration surface |
| C | `v31-c-continuity-loss-harness` | `C-harness.md` | revert continuity harness |
| D | `v31-d-twelve-month-cohort` | `D-dogfood.md` | app-bound continuity opens V31.5 |
| E | `v31-e-user-tool-review` | `E-external-review.md` | reviewer blockers open V31.5 |
| F | `v31-f-kill-recovery-decision` | `F-decision.md` | decide portability fate |
| G | `v31-g-proof-packet` | `G-final-proof.md` | updates ROADMAP only if gate passes |

### V32 Institutional Memory

Artifact root: `docs/verification/v32-proof-runs/`

| Phase | Commit boundary | Evidence artifact | Rollback / recovery note |
| --- | --- | --- | --- |
| A | `v32-a-org-transition-model` | `A-capability.md` | revert role-transition model |
| B | `v32-b-permissioned-handoff-ux` | `B-ux.md` | revert handoff surface |
| C | `v32-c-access-context-harness` | `C-harness.md` | revert leak/context harness |
| D | `v32-d-three-org-transition` | `D-dogfood.md` | access/context miss opens V32.5 |
| E | `v32-e-org-security-review` | `E-external-review.md` | reviewer blockers open V32.5 |
| F | `v32-f-kill-recovery-decision` | `F-decision.md` | decide org-memory fate |
| G | `v32-g-proof-packet` | `G-final-proof.md` | updates ROADMAP only if gate passes |

### V33 Legal/Compliance Grade

Artifact root: `docs/verification/v33-proof-runs/`

| Phase | Commit boundary | Evidence artifact | Rollback / recovery note |
| --- | --- | --- | --- |
| A | `v33-a-compliance-artifact-pack` | `A-capability.md` | revert compliance artifact generation |
| B | `v33-b-buyer-compliance-room` | `B-ux.md` | revert buyer room surface |
| C | `v33-c-legal-security-harness` | `C-harness.md` | revert artifact harness |
| D | `v33-d-buyer-review-window` | `D-dogfood.md` | promise-only proof opens V33.5 |
| E | `v33-e-legal-security-review` | `E-external-review.md` | blocker findings open V33.5 |
| F | `v33-f-kill-recovery-decision` | `F-decision.md` | decide compliance fate |
| G | `v33-g-proof-packet` | `G-final-proof.md` | updates ROADMAP only if gate passes |

### V34 Human-Agent Governance

Artifact root: `docs/verification/v34-proof-runs/`

| Phase | Commit boundary | Evidence artifact | Rollback / recovery note |
| --- | --- | --- | --- |
| A | `v34-a-decision-ledger` | `A-capability.md` | revert governance ledger |
| B | `v34-b-governance-review-ux` | `B-ux.md` | revert review surface |
| C | `v34-c-100-case-replay-harness` | `C-harness.md` | revert governance harness |
| D | `v34-d-governance-dogfood` | `D-dogfood.md` | authority replay miss opens V34.5 |
| E | `v34-e-governance-reviewer` | `E-external-review.md` | reviewer blockers open V34.5 |
| F | `v34-f-kill-recovery-decision` | `F-decision.md` | decide authority fate |
| G | `v34-g-proof-packet` | `G-final-proof.md` | updates ROADMAP only if gate passes |

### V35 Category Completion

Artifact root: `docs/verification/v35-proof-runs/`

| Phase | Commit boundary | Evidence artifact | Rollback / recovery note |
| --- | --- | --- | --- |
| A | `v35-a-substrate-packaging` | `A-capability.md` | revert category packaging |
| B | `v35-b-category-review-surface` | `B-ux.md` | revert category review surface |
| C | `v35-c-category-reliability-harness` | `C-harness.md` | revert category harness |
| D | `v35-d-paying-org-product-window` | `D-dogfood.md` | missing org/product/year proof blocks close |
| E | `v35-e-independent-category-review` | `E-external-review.md` | reviewer says app-not-substrate blocks close |
| F | `v35-f-kill-recovery-decision` | `F-decision.md` | post-V35 reset if category miss |
| G | `v35-g-proof-packet` | `G-final-proof.md` | updates ROADMAP only if gate passes |
