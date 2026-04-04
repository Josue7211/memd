# Maintainer Workflow

`memd` uses a milestone-plus-scope workflow.

## Branch Roles

- `main`
  - release branch only
  - tagged versions come from here
- `work/<milestone>`
  - integration branch for the current capability version or milestone
- `feat/<scope>`
  - scoped implementation branch cut from the active `work/<milestone>` branch
- `fix/<scope>`
  - bounded bugfix branch
- `docs/<scope>`
  - documentation-only or workflow-only branch when the change is independently reviewable

## Merge Policy

- do not work directly on `main`
- do not let one feature branch silently absorb later unrelated scopes
- prefer linear milestone history:
  - rebase or squash scoped branches onto the active `work/<milestone>` branch
- merge `work/<milestone>` into `main` only when the milestone slice is coherent and verified
- if a branch changes public behavior, update docs and `CHANGELOG.md` in the same branch

## Push Policy

- push active milestone and scoped branches upstream once they become real workstreams
- keep remote branches aligned with local project reality so collaborators can follow the actual topology
- delete remote scoped branches after they are merged and no longer needed

## Review Policy

- every scoped branch should answer:
  - what changed
  - why it belongs in this scope
  - how it was verified
- do not mix unrelated docs, refactors, and features in one PR unless they are inseparable

## Release Policy

1. land verified scoped work into the active `work/<milestone>` branch
2. update `CHANGELOG.md`
3. cut `release/vX.Y.Z` if release prep needs its own branch
4. tag from `main`
5. keep milestone summaries and roadmap state consistent with what actually shipped
