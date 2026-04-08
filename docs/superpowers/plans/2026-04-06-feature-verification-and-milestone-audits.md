# Feature Verification And Milestone Audits Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a reusable verification system that tracks features as canonical contracts, audits milestones against those contracts, and marks `v1` through `v3` as unverified until they pass backward audit.

**Architecture:** Introduce a new `docs/verification/` tree that separates feature truth from milestone truth. Keep the first slice documentation-first but grounded in real runtime evidence by seeding the registry from existing requirements, roadmap claims, codebase maps, and the current memory audit findings.

**Tech Stack:** Markdown docs, existing `.planning/` artifacts, existing Rust test commands, existing codebase audit docs.

---

### Task 1: Create Verification Directory And Canonical Doc Skeletons

**Files:**
- Create: `docs/verification/FEATURES.md`
- Create: `docs/verification/AUDIT-RULES.md`
- Create: `docs/verification/RUNBOOK.md`
- Create: `docs/verification/milestones/MILESTONE-v1.md`
- Create: `docs/verification/milestones/MILESTONE-v2.md`
- Create: `docs/verification/milestones/MILESTONE-v3.md`

- [ ] **Step 1: Write the failing structure check**

```bash
test -f docs/verification/FEATURES.md \
  && test -f docs/verification/AUDIT-RULES.md \
  && test -f docs/verification/RUNBOOK.md \
  && test -f docs/verification/milestones/MILESTONE-v1.md \
  && test -f docs/verification/milestones/MILESTONE-v2.md \
  && test -f docs/verification/milestones/MILESTONE-v3.md
```

Expected: exit code `1` because the docs do not exist yet.

- [ ] **Step 2: Create `docs/verification/FEATURES.md`**

```md
# Feature Registry

## Status Vocabulary

- `unverified`
- `auditing`
- `verified`
- `partial`
- `broken`

## Verification Depth Vocabulary

- `minimal`
- `strong`
- `exhaustive`

## Feature Template

### FEATURE-000: Example Feature

- version: `v0`
- milestones: `v0`
- status: `unverified`
- verification_depth: `exhaustive`

#### User Contract

Describe what a user should experience if this feature works.

#### Implementation Surfaces

- `path/to/file`
- `binary command`
- `HTTP route`

#### Dependencies

- `FEATURE-...`

#### Verification Methods

- unit/integration:
  - `command here`
- workflow:
  - `command or manual flow here`
- adversarial:
  - `noise or failure scenario here`
- migration:
  - `if required`
- cross-harness:
  - `if required`

#### Failure Modes

- how this feature usually breaks

#### Notes

- audit notes and caveats
```

- [ ] **Step 3: Create `docs/verification/AUDIT-RULES.md`**

```md
# Audit Rules

## Core Rule

Runtime behavior beats planning status.

## Feature Verdicts

- `verified`: all required checks pass
- `partial`: core contract exists but one or more required checks fail or are missing
- `broken`: user contract fails in a material way
- `unverified`: not yet audited
- `auditing`: currently under active audit

## Milestone Verdicts

- `verified`: all claimed features are verified
- `regressed`: one or more previously verified features are now partial or broken
- `unverified`: not yet audited
- `auditing`: currently under active audit

## Exhaustive Verification Standard

A feature is exhaustive only if it has:

- implementation trace
- direct test proof
- workflow proof
- adversarial proof
- rerun command
- cross-harness proof when relevant
```

- [ ] **Step 4: Create `docs/verification/RUNBOOK.md`**

```md
# Verification Runbook

## Backward Audit Flow

1. Pick milestone.
2. Mark milestone `auditing`.
3. Enumerate claimed features from the feature registry.
4. Run each feature's rerun commands.
5. Record findings.
6. Mark each feature `verified`, `partial`, or `broken`.
7. Mark milestone `verified` only if all claimed features are verified.

## Post-Change Regression Flow

1. Identify touched features.
2. Run feature rerun commands.
3. Re-run milestone audits affected by those features.
4. Mark regressions immediately in milestone files.
```

- [ ] **Step 5: Create milestone audit stubs**

```md
# Milestone v1 Audit

- status: `unverified`
- audit_date: `not started`
- claimed_features: `to be filled from feature registry`
- result: `pending`

## Findings

- none yet
```

Repeat the same structure for:

- `docs/verification/milestones/MILESTONE-v2.md`
- `docs/verification/milestones/MILESTONE-v3.md`

- [ ] **Step 6: Run the structure check again**

Run:

```bash
test -f docs/verification/FEATURES.md \
  && test -f docs/verification/AUDIT-RULES.md \
  && test -f docs/verification/RUNBOOK.md \
  && test -f docs/verification/milestones/MILESTONE-v1.md \
  && test -f docs/verification/milestones/MILESTONE-v2.md \
  && test -f docs/verification/milestones/MILESTONE-v3.md
```

Expected: exit code `0`.

- [ ] **Step 7: Commit**

```bash
git add docs/verification
git commit -m "docs: add verification registry and milestone audit scaffolding"
```

### Task 2: Seed The Feature Registry For `v1`

**Files:**
- Modify: `docs/verification/FEATURES.md`
- Read: `.planning/REQUIREMENTS.md`
- Read: `ROADMAP.md`
- Read: `.planning/codebase/MEMORY-AUDIT.md`

- [ ] **Step 1: Write the failing audit grep**

Run:

```bash
rg -n "FEATURE-V1-" docs/verification/FEATURES.md
```

Expected: no matches.

- [ ] **Step 2: Add `v1` feature entries**

Add concrete entries like:

```md
### FEATURE-V1-CORE-STORE: Durable Typed Memory Storage

- version: `v1`
- milestones: `v1`
- status: `unverified`
- verification_depth: `exhaustive`

#### User Contract

`memd remember` or equivalent store paths create durable typed memory records that survive later retrieval.

#### Implementation Surfaces

- `crates/memd-client/src/main.rs`
- `crates/memd-server/src/main.rs`
- `crates/memd-server/src/store.rs`

#### Dependencies

- none

#### Verification Methods

- unit/integration:
  - `cargo test -p memd-server --quiet`
- workflow:
  - `memd remember ...` followed by resume or search
- adversarial:
  - store amid synced/session noise
- migration:
  - not required in first audit
- cross-harness:
  - deferred unless bridge claim is audited

#### Failure Modes

- store succeeds but hot-path recall fails
- memory is persisted but not surfaced

#### Notes

- tie to current memory audit findings
```

Add at least these `v1` entries:

- `FEATURE-V1-CORE-STORE`
- `FEATURE-V1-CORE-SEARCH`
- `FEATURE-V1-LIFECYCLE-REPAIR`
- `FEATURE-V1-WORKING-CONTEXT`
- `FEATURE-V1-WORKING-MEMORY`
- `FEATURE-V1-EXPLAIN`
- `FEATURE-V1-PROVENANCE`
- `FEATURE-V1-BUNDLE-ATTACH`

- [ ] **Step 3: Mark all `v1` entries unverified**

Ensure each new `v1` entry includes:

```md
- status: `unverified`
```

- [ ] **Step 4: Re-run the grep**

Run:

```bash
rg -n "FEATURE-V1-" docs/verification/FEATURES.md
```

Expected: multiple `FEATURE-V1-...` matches.

- [ ] **Step 5: Commit**

```bash
git add docs/verification/FEATURES.md
git commit -m "docs: seed v1 feature verification contracts"
```

### Task 3: Seed The Feature Registry For `v2`

**Files:**
- Modify: `docs/verification/FEATURES.md`
- Read: `.planning/REQUIREMENTS.md`
- Read: `.planning/phases/02-v2-foundations/02-01-SUMMARY.md`
- Read: `.planning/phases/03-v2-branchable-beliefs/03-01-SUMMARY.md`
- Read: `.planning/phases/04-v2-retrieval-feedback/04-01-SUMMARY.md`
- Read: `.planning/phases/05-v2-trust-weighted-ranking/05-01-SUMMARY.md`
- Read: `.planning/phases/06-v2-contradiction-resolution/06-01-SUMMARY.md`
- Read: `.planning/phases/07-v2-procedural-and-self-model-memory/07-01-SUMMARY.md`
- Read: `.planning/phases/08-v2-reversible-compression-and-rehydration/08-01-SUMMARY.md`
- Read: `.planning/phases/09-v2-obsidian-compiled-evidence-workspace/09-01-SUMMARY.md`

- [ ] **Step 1: Write the failing audit grep**

Run:

```bash
rg -n "FEATURE-V2-" docs/verification/FEATURES.md
```

Expected: no matches.

- [ ] **Step 2: Add `v2` feature entries**

Add at least these entries:

- `FEATURE-V2-TRUST-FRESHNESS-PROVENANCE`
- `FEATURE-V2-BRANCHABLE-BELIEFS`
- `FEATURE-V2-RETRIEVAL-FEEDBACK`
- `FEATURE-V2-TRUST-WEIGHTED-RANKING`
- `FEATURE-V2-CONTRADICTION-RESOLUTION`
- `FEATURE-V2-PROCEDURAL-SELF-MODEL`
- `FEATURE-V2-REVERSIBLE-REHYDRATION`
- `FEATURE-V2-OBSIDIAN-COMPILED-EVIDENCE`

Each entry must include:

- actual implementation surfaces
- current rerun command candidates
- likely failure modes
- exhaustive verification methods

- [ ] **Step 3: Include current known runtime findings where applicable**

For ranking-related features, include notes such as:

```md
- current-task retrieval previously dropped durable project memory behind synced noise until the global non-live-truth ranking fix landed
```

- [ ] **Step 4: Re-run the grep**

Run:

```bash
rg -n "FEATURE-V2-" docs/verification/FEATURES.md
```

Expected: multiple `FEATURE-V2-...` matches.

- [ ] **Step 5: Commit**

```bash
git add docs/verification/FEATURES.md
git commit -m "docs: seed v2 feature verification contracts"
```

### Task 4: Seed The Feature Registry For `v3`

**Files:**
- Modify: `docs/verification/FEATURES.md`
- Read: `.planning/phases/10-v3-shared-workspace-foundations/10-01-SUMMARY.md`
- Read: `.planning/phases/11-v3-workspace-handoff-bundles/11-01-SUMMARY.md`
- Read: `.planning/phases/12-v3-workspace-policy-corrections/12-01-SUMMARY.md`
- Read: `.planning/phases/13-v3-workspace-aware-retrieval-priorities/13-01-SUMMARY.md`
- Read: `.planning/codebase/MEMORY-AUDIT.md`

- [ ] **Step 1: Write the failing audit grep**

Run:

```bash
rg -n "FEATURE-V3-" docs/verification/FEATURES.md
```

Expected: no matches.

- [ ] **Step 2: Add `v3` feature entries**

Add at least these entries:

- `FEATURE-V3-SHARED-WORKSPACE-LANES`
- `FEATURE-V3-HANDOFF-BUNDLES`
- `FEATURE-V3-WORKSPACE-POLICY-CORRECTIONS`
- `FEATURE-V3-WORKSPACE-AWARE-RETRIEVAL`

For each entry include:

- shared-memory user contract
- workspace scope expectations
- verification commands
- cross-workspace adversarial scenario

- [ ] **Step 3: Capture the current audit reality**

For `FEATURE-V3-WORKSPACE-AWARE-RETRIEVAL`, include notes like:

```md
- matching workspace memory must remain visible under cross-workspace synced noise
- regression is covered by `current_task_context_prefers_matching_workspace_memory_under_cross_workspace_noise`
```

- [ ] **Step 4: Re-run the grep**

Run:

```bash
rg -n "FEATURE-V3-" docs/verification/FEATURES.md
```

Expected: multiple `FEATURE-V3-...` matches.

- [ ] **Step 5: Commit**

```bash
git add docs/verification/FEATURES.md
git commit -m "docs: seed v3 feature verification contracts"
```

### Task 5: Build Initial Milestone Audit Matrices

**Files:**
- Modify: `docs/verification/milestones/MILESTONE-v1.md`
- Modify: `docs/verification/milestones/MILESTONE-v2.md`
- Modify: `docs/verification/milestones/MILESTONE-v3.md`
- Read: `docs/verification/FEATURES.md`

- [ ] **Step 1: Add `v1` claimed feature matrix**

Insert a table like:

```md
## Claimed Features

| Feature | Status | Notes |
|---------|--------|-------|
| `FEATURE-V1-CORE-STORE` | `unverified` | backward audit not started |
| `FEATURE-V1-CORE-SEARCH` | `unverified` | backward audit not started |
| `FEATURE-V1-LIFECYCLE-REPAIR` | `unverified` | backward audit not started |
```

- [ ] **Step 2: Add `v2` claimed feature matrix**

Use the same structure for the `v2` feature set.

- [ ] **Step 3: Add `v3` claimed feature matrix**

Use the same structure for the `v3` feature set.

- [ ] **Step 4: Mark all three milestone audit files as `unverified`**

Ensure each file explicitly contains:

```md
- status: `unverified`
```

- [ ] **Step 5: Commit**

```bash
git add docs/verification/milestones
git commit -m "docs: add initial v1-v3 milestone audit matrices"
```

### Task 6: Capture The First Backward Audit Findings

**Files:**
- Modify: `docs/verification/FEATURES.md`
- Modify: `docs/verification/milestones/MILESTONE-v1.md`
- Modify: `docs/verification/milestones/MILESTONE-v2.md`
- Modify: `docs/verification/milestones/MILESTONE-v3.md`
- Read: `.planning/codebase/MEMORY-AUDIT.md`

- [ ] **Step 1: Add real audit notes for already-confirmed findings**

Update relevant features with notes such as:

```md
- confirmed fix: global non-live-truth ranking in `build_context`
- confirmed regression coverage:
  - `current_task_context_keeps_project_fact_visible_under_synced_noise`
  - `current_task_context_prefers_matching_workspace_memory_under_cross_workspace_noise`
  - `read_bundle_resume_keeps_recalled_project_fact_visible_in_bundle_memory`
  - `superseded_memory_drops_out_after_manual_correction_loop`
```

- [ ] **Step 2: Mark the corresponding feature statuses**

For the features directly supported by these regressions, update status from:

```md
- status: `unverified`
```

to one of:

```md
- status: `partial`
```

or:

```md
- status: `verified`
```

Choose `partial` unless the full user-visible contract has been audited.

- [ ] **Step 3: Record milestone-level findings**

Add findings like:

```md
## Findings

- durable project memory was previously dropped from `current_task` retrieval under synced session-state noise
- matching workspace memory is now regression-covered
- low-level correction suppression works when the correction lifecycle is driven explicitly
- user-facing zero-friction correction ingest is still missing
```

- [ ] **Step 4: Leave milestone verdicts conservative**

Ensure `v1`, `v2`, and `v3` do **not** move to `verified` yet unless every claimed feature is truly audited.

- [ ] **Step 5: Commit**

```bash
git add docs/verification/FEATURES.md docs/verification/milestones
git commit -m "docs: record initial backward audit findings for v1-v3"
```

### Task 7: Add A Reusable Verification Entry Point

**Files:**
- Modify: `docs/verification/RUNBOOK.md`
- Modify: `README.md`
- Modify: `docs/setup.md`

- [ ] **Step 1: Add rerun command conventions to the runbook**

Add a section like:

```md
## Standard Rerun Command Style

Every feature contract should prefer:

- exact `cargo test` command
- exact CLI command
- exact scenario/manual flow
```

- [ ] **Step 2: Add a README pointer**

Add a short section to `README.md`:

```md
## Verification

Milestone truth is tracked in `docs/verification/`.
Previously completed milestones are treated as unverified until they pass audit.
```

- [ ] **Step 3: Add setup docs for operators**

Add a short section to `docs/setup.md`:

```md
## Backward Audit Workflow

Use `docs/verification/FEATURES.md` and `docs/verification/milestones/` as the source of truth for feature and milestone verification.
```

- [ ] **Step 4: Run doc grep verification**

Run:

```bash
rg -n "docs/verification|unverified until audited|Feature Registry|Milestone v1 Audit" README.md docs/setup.md docs/verification
```

Expected: matches across the new docs.

- [ ] **Step 5: Commit**

```bash
git add README.md docs/setup.md docs/verification
git commit -m "docs: wire verification system into project guidance"
```

### Task 8: Full Verification

**Files:**
- Modify as needed: touched files above only

- [ ] **Step 1: Verify the verification docs exist**

Run:

```bash
test -f docs/verification/FEATURES.md \
  && test -f docs/verification/AUDIT-RULES.md \
  && test -f docs/verification/RUNBOOK.md \
  && test -f docs/verification/milestones/MILESTONE-v1.md \
  && test -f docs/verification/milestones/MILESTONE-v2.md \
  && test -f docs/verification/milestones/MILESTONE-v3.md
```

Expected: exit code `0`.

- [ ] **Step 2: Verify feature coverage presence**

Run:

```bash
rg -n "FEATURE-V1-|FEATURE-V2-|FEATURE-V3-" docs/verification/FEATURES.md
```

Expected: entries for all three versions.

- [ ] **Step 3: Verify milestone status discipline**

Run:

```bash
rg -n "status: `unverified`|status: `partial`|status: `verified`|status: `regressed`" docs/verification/milestones
```

Expected: explicit milestone statuses in all audit files.

- [ ] **Step 4: Verify current technical regressions still pass**

Run:

```bash
cargo test -p memd-server --quiet
cargo test -p memd-client --quiet
```

Expected: all tests pass.

## Self-Review

### Spec coverage

- covers canonical feature registry
- covers milestone audit files
- covers audit rules and runbook
- covers backward audit seeding for `v1`-`v3`
- covers integration into README and setup docs
- keeps milestones conservative until actually audited

### Placeholder scan

- no `TODO` / `TBD` placeholders remain
- each task names exact files and commands
- each task contains concrete content rather than abstract instructions

### Type consistency

- one status vocabulary:
  - `unverified`
  - `auditing`
  - `verified`
  - `partial`
  - `broken`
  - `regressed`
- one doc tree:
  - `docs/verification/`
