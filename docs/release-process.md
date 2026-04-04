# Release Process

`memd` uses a branch-first workflow.

## Branching

- do active work on a dedicated branch
- use a `work/<milestone>` branch as the integration line for the active version
- cut `feat/<scope>` and `fix/<scope>` branches from that work branch for bounded implementation slices
- keep each scoped branch focused on one phase or one small set of related changes
- avoid doing cleanup directly on `main`
- merge only after the phase or change set is verified
- see [Branching Model](./branching.md) for branch names and commit discipline

## Version History

- record meaningful changes as small, reviewable commits
- keep roadmap phases aligned with versioned capabilities
- prefer explicit release notes over silent behavior drift
- maintain `CHANGELOG.md` as part of release prep

## Contributor Workflow

1. branch from the current `work/<milestone>` line
2. keep the change scoped to one `feat/<scope>` or `fix/<scope>` branch
3. run formatting and tests
4. update docs when behavior changes
5. merge back into the milestone branch once verified
6. release from `main`, not from a feature branch

## Review Expectations

- explain what changed and why
- mention any file splits that were made to improve reuse or maintainability
- call out any user-visible behavior changes

## Security Reporting

Security issues should follow [`SECURITY.md`](../SECURITY.md).

## Project Rule

File splitting should happen only when the seam is real. Do not split code just
to make the tree look smaller.
