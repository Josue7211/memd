# Branching Model

`memd` uses a branch-first workflow.

## Branch Types

- `main` is the release line.
- `work/<milestone>` is for active milestone work.
- `feat/<area>` is for scoped feature work inside a milestone.
- `fix/<area>` is for targeted bug fixes.
- `release/vX.Y.Z` is for preparing a tagged release.
- `hotfix/<area>` is for urgent release-line fixes.

## Phase Mapping

`memd` should not run a whole version on one giant branch.

Use this shape instead:

- `work/v0-*`, `work/v1-*`, `work/v2-*` for the milestone integration line
- `feat/v2-branchable-beliefs`, `feat/v2-retrieval-feedback`, `feat/v2-trust-weighted-ranking`, `feat/v2-contradiction-resolution`, `feat/v2-procedural-self-model` for bounded capability slices
- `feat/obsidian-*` or `feat/rag-*` for cross-cutting integrations that can move independently of one memory phase

If a scope is big enough to need its own summary in the roadmap or changelog, it is big enough to deserve its own branch.

## Rules

- Do not work directly on `main`.
- Keep each branch focused on one coherent change set.
- Prefer small, atomic commits over large mixed commits.
- Split files only at real seams that improve ownership or reuse.
- Merge or rebase only after the branch is verified.
- Push milestone and feature branches upstream when they become active so the remote history reflects the real work topology.
- Do not let one feature branch silently absorb later unrelated phases.

## Version History

- Use commit messages that describe the user-visible or architectural change.
- Keep milestone work grouped in a readable history.
- Tag releases explicitly so external users can trace stable points.
- Document noteworthy behavior changes in `CHANGELOG.md`.

## Recommended Flow

1. cut or switch to the current `work/<milestone>` branch
2. branch a scoped `feat/<area>` or `fix/<area>` branch from that work branch
3. land atomic commits on the scoped branch
4. verify with formatting, tests, and docs updates
5. merge or replay the scoped branch back onto the `work/<milestone>` branch
6. merge the milestone branch to `main` only when the milestone slice is actually ready
