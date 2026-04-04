# Release Process

`memd` uses a branch-first workflow.

## Branching

- do active work on a dedicated branch
- keep the branch focused on one phase or one small set of related changes
- avoid doing cleanup directly on `main`
- merge only after the phase or change set is verified
- see [Branching Model](./branching.md) for branch names and commit discipline

## Version History

- record meaningful changes as small, reviewable commits
- keep roadmap phases aligned with versioned capabilities
- prefer explicit release notes over silent behavior drift
- maintain `CHANGELOG.md` as part of release prep

## Contributor Workflow

1. branch from the current development line
2. keep the change scoped
3. run formatting and tests
4. update docs when behavior changes
5. open a review with the branch and the relevant version context

## Review Expectations

- explain what changed and why
- mention any file splits that were made to improve reuse or maintainability
- call out any user-visible behavior changes

## Security Reporting

Security issues should follow [`SECURITY.md`](../SECURITY.md).

## Project Rule

File splitting should happen only when the seam is real. Do not split code just
to make the tree look smaller.
