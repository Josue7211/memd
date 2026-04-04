# Branching Model

`memd` uses a branch-first workflow.

## Branch Types

- `main` is the release line.
- `work/<milestone>` is for active milestone work.
- `feat/<area>` is for scoped feature work inside a milestone.
- `fix/<area>` is for targeted bug fixes.
- `release/vX.Y.Z` is for preparing a tagged release.
- `hotfix/<area>` is for urgent release-line fixes.

## Rules

- Do not work directly on `main`.
- Keep each branch focused on one coherent change set.
- Prefer small, atomic commits over large mixed commits.
- Split files only at real seams that improve ownership or reuse.
- Merge or rebase only after the branch is verified.

## Version History

- Use commit messages that describe the user-visible or architectural change.
- Keep milestone work grouped in a readable history.
- Tag releases explicitly so external users can trace stable points.
- Document noteworthy behavior changes in `CHANGELOG.md`.

## Practical Flow

1. create or switch to the appropriate work branch
2. make the smallest coherent change set
3. run formatting and tests
4. update docs if behavior or public contracts changed
5. open a review or merge request with the branch context

