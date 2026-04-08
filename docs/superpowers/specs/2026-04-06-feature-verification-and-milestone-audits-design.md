# Feature Verification And Milestone Audits Design

## Goal

Create a reusable verification system for `memd` so milestone completion is
measured by audited product truth rather than planning status. The system must:

- track features as reusable verification contracts
- track milestones as collections of claimed features
- treat previously achieved milestones as `unverified until audited`
- support exhaustive verification by default
- make future regression sweeps practical after changes

## Why This Exists

The current repo has planning artifacts, roadmap phases, and test coverage, but
that has not been enough to guarantee product truth. A milestone can be marked
complete while runtime behavior is still weak. We need two distinct layers:

- feature truth:
  - does this feature actually work?
- milestone truth:
  - does this milestone's claimed feature set actually work together?

Without that split, later changes can silently break earlier product behavior.

## Recommended Approach

Use a hybrid system:

- a canonical feature registry
- milestone audit files that point at those features

This is the primary design because it gives both:

- milestone-level closeout discipline
- reusable feature-level regression contracts

## Design

### 1. Canonical Feature Registry

Create a single source of truth for reusable feature contracts:

- `docs/verification/FEATURES.md`

Each feature entry should include:

- `id`
- `name`
- `version`
- `milestones`
- `status`
  - `unverified`
  - `auditing`
  - `verified`
  - `partial`
  - `broken`
- `user_contract`
  - what the user should experience if it works
- `implementation_surfaces`
  - key files, commands, routes, or generated artifacts
- `verification_depth`
  - exhaustive by default
- `verification_methods`
  - unit/integration tests
  - E2E workflow tests
  - adversarial/regression scenarios
  - migration checks
  - cross-harness checks
- `rerun_commands`
  - the exact commands or scripts to re-verify later
- `dependencies`
  - prerequisite features or subsystems
- `failure_modes`
  - how this feature usually breaks
- `notes`
  - audit findings and caveats

Example feature classes:

- durable memory store
- compact context retrieval
- working-memory budget enforcement
- contradiction resolution
- workspace-aware recall
- handoff bundle generation
- correction durability
- Obsidian compiled evidence workspace
- attach/import bridge surfaces

### 2. Milestone Audit Files

Create one audit file per milestone:

- `docs/verification/milestones/MILESTONE-v1.md`
- `docs/verification/milestones/MILESTONE-v2.md`
- `docs/verification/milestones/MILESTONE-v3.md`
- later milestones follow the same pattern

Each milestone audit file should include:

- milestone goal
- claimed features
- audit status
  - `unverified`
  - `auditing`
  - `verified`
  - `regressed`
- audit date
- feature result matrix
- regressions found
- blockers
- follow-up fixes required
- overall milestone verdict

Important rule:

- a milestone is never considered product-complete just because roadmap phases
  landed
- a milestone becomes `verified` only when its claimed features pass audit

### 3. Audit Rules

Define a shared audit standard:

- `docs/verification/AUDIT-RULES.md`

This file should define:

- what counts as passing
- what counts as partial
- what counts as broken
- what “exhaustive” means
- when cross-harness verification is required
- when migration verification is required
- when adversarial scenarios are mandatory
- how to handle planning/runtime disagreement

Core rule:

- runtime behavior beats planning status

### 4. Runbook

Create an operator runbook:

- `docs/verification/RUNBOOK.md`

This file should explain:

- how to audit an existing milestone backward
- how to verify a new milestone before calling it done
- how to run post-change regression sweeps
- how to mark a feature or milestone as regressed
- how to convert audit failures into missing tests and fixes

## Initial Status Policy

All previously claimed milestones start as:

- `unverified until audited`

This applies especially to:

- `v1`
- `v2`
- `v3`

The current repo state should not assume old planning completion equals feature
truth.

## Verification Depth

Default verification depth is exhaustive.

That means each important feature should eventually have:

- implementation trace:
  - code path exists and is identifiable
- direct test proof:
  - unit or integration proof for key behavior
- workflow proof:
  - user-visible E2E validation
- adversarial proof:
  - realistic failure or noise scenarios
- regression proof:
  - rerunnable commands after future updates
- harness proof:
  - where relevant, confirm behavior across supported harness surfaces

Not every feature needs the same cost immediately, but the system should be
designed to support this level by default.

## Migration Plan

### Phase 1: Establish Verification Structure

Create:

- `docs/verification/FEATURES.md`
- `docs/verification/AUDIT-RULES.md`
- `docs/verification/RUNBOOK.md`
- `docs/verification/milestones/MILESTONE-v1.md`
- `docs/verification/milestones/MILESTONE-v2.md`
- `docs/verification/milestones/MILESTONE-v3.md`

Initial milestone statuses:

- `unverified`

### Phase 2: Build The `v1`-`v3` Feature Registry

Extract features from:

- `.planning/REQUIREMENTS.md`
- `ROADMAP.md`
- `.planning/phases/01-*` through `.planning/phases/13-*`
- the current runtime codepaths

Feature registry entries should reflect actual user-facing contracts, not only
 phase labels.

### Phase 3: Audit Backward

Audit in this order:

1. `v1` core memory loop
2. `v2` trust, contradiction, and evidence features
3. `v3` workspace/shared-memory behavior

For each failed audit:

- record the failure in the milestone audit file
- record the feature as `partial` or `broken`
- add or extend regression tests
- fix behavior before advancing milestone status

### Phase 4: Use The Same System Going Forward

Future milestone closeout requires:

- feature matrix complete
- required tests present
- milestone audit verdict recorded

## Benefits

This system gives:

- a reusable feature truth model
- milestone closeout discipline
- backward auditing for already-claimed work
- future regression confidence after changes
- a way to detect when the API or product behavior breaks even if unit tests
  still pass

## Risks

### Risk 1: Documentation Drift

If the registry is not maintained, it becomes another stale planning layer.

Mitigation:

- treat failed audits and new regressions as required updates to verification
  docs

### Risk 2: Overly Granular Features

If the registry is too fine-grained, it becomes hard to use.

Mitigation:

- define features at user-visible contract level, not every internal helper

### Risk 3: Verification Cost Explosion

Exhaustive verification can become expensive if applied blindly.

Mitigation:

- keep exhaustive as the design default
- prioritize by product risk when building out coverage

## Recommendation

Adopt the hybrid verification model immediately and start with `v1`-`v3`.

Do not trust previous milestone completion claims until the backward audit is
done. The current memory debugging work should feed directly into the first
feature registry and milestone audit files.
