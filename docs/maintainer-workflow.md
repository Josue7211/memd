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

Practical rule:

- merge scoped work into the milestone branch
- merge milestone work into `main`
- do not use `main` as an integration sandbox

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

Reviewers should check:

- roadmap and scope alignment
- public contract drift
- missing docs or tests
- whether the branch should have been split further
- whether the change weakens provenance, retrieval compactness, or control-plane boundaries

## Infra Verification Policy

Infrastructure claims are evidence-bound.

Maintainers should not state or approve claims about:

- tunnels
- DNS
- domains or subdomains
- public accessibility
- LAN or Tailscale reachability
- VM ownership or service location

unless they were verified locally first. Use [Infrastructure Facts](./infra-facts.md)
as the repo truth source for environment-specific facts.

If a fact was not checked, call it `unverified`. Do not let review comments,
docs, or assistant output invent URLs or accessibility claims from context.

## Release Policy

1. land verified scoped work into the active `work/<milestone>` branch
2. update `CHANGELOG.md`
3. cut `release/vX.Y.Z` if release prep needs its own branch
4. tag from `main`
5. keep milestone summaries and roadmap state consistent with what actually shipped

## Remote Hygiene

- push active milestone and scoped branches upstream early
- delete stale remote branches after merge
- keep remote branch names aligned with the actual roadmap slices
- avoid leaving significant local-only history that collaborators cannot see
