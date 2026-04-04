---
phase: 03-v2-branchable-beliefs
plan: 01
subsystem: api
tags: [memory, contradiction, belief-branch, explain]
requires:
  - phase: 02-v2-foundations
    provides: trust floors, artifact trails, explicit policy hooks
provides:
  - durable belief branches on memory records
  - branch-aware duplicate and canonical separation
  - sibling branch inspection in explain responses
affects:
  - future contradiction resolution work
  - future retrieval feedback and branch arbitration
tech-stack:
  added: []
  patterns:
    - explicit contradiction lanes through payload-level branch ids
    - explain surfaces sibling belief records directly
key-files:
  created:
    - .planning/phases/03-v2-branchable-beliefs/03-CONTEXT.md
    - .planning/phases/03-v2-branchable-beliefs/03-01-PLAN.md
    - .planning/phases/03-v2-branchable-beliefs/03-01-SUMMARY.md
  modified:
    - crates/memd-schema/src/lib.rs
    - crates/memd-server/src/keys.rs
    - crates/memd-server/src/main.rs
    - crates/memd-server/src/inspection.rs
    - crates/memd-client/src/main.rs
    - crates/memd-client/src/render.rs
    - docs/api.md
key-decisions:
  - "Belief branches are explicit durable fields, not inferred from repo branch."
  - "Duplicate control separates competing beliefs by belief branch."
  - "Explain surfaces sibling records directly instead of hiding contradiction structure."
patterns-established:
  - "Pattern 1: contradiction lanes stay visible through branch siblings."
  - "Pattern 2: new durable semantics ride in the payload contract before schema expansion."
requirements-completed: [SUPR-03]
completed: 2026-04-04
---

# Phase 3: `v2` Branchable Beliefs Summary

`memd` now has an explicit first contradiction lane.

## Accomplishments
- Added `belief_branch` to durable memory and the request surfaces that need it.
- Separated canonical and redundancy keys across belief branches so competing beliefs can coexist.
- Extended explain with bounded sibling branch inspection and surfaced branch state in the CLI summary.
- Added branch-aware filtering to search and inbox and documented the new contract.

## Verification
- `cargo test -q`

## Next Phase Readiness

Phase 4 can now add retrieval outcome feedback on top of explicit belief lanes instead of hidden contradiction flattening.

---
*Phase: 03-v2-branchable-beliefs*
*Completed: 2026-04-04*
