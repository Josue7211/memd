> Secondary/reference doc. For current project truth start with [[ROADMAP]] and [[docs/WHERE-AM-I.md|WHERE-AM-I]].

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
- tag releases from `main`
- keep milestone and feature branch names readable enough to reconstruct history later

## Contributor Workflow

1. branch from the current `work/<milestone>` line
2. keep the change scoped to one `feat/<scope>` or `fix/<scope>` branch
3. run formatting and tests
4. update docs when behavior changes
5. merge back into the milestone branch once verified
6. release from `main`, not from a feature branch

## Release Checklist

Before tagging a release:

1. verify the intended milestone branch is coherent
2. ensure public docs match shipped behavior
3. verify bundle bootstrap still works from the README quickstart:
   - `memd setup --agent codex`
   - `memd status --output .memd`
   - one agent launch surface
4. verify user-facing status/setup signals are accurate:
   - `setup_ready`
   - `missing`
   - backend reachability
   - resume preview for the hot lane
5. run the local release-claim honesty gates:
   - `bash scripts/verify/feature-registry-audit.sh`
   - `bash scripts/verify/feature-release-claim-honesty-gates-proof.sh`
   - `bash scripts/verify/local-25-5-release-claim-honesty-gate.sh`
6. confirm local `25/5` wording stays distinct from any unsupported `25/25`, production-ready, external-verification, or benchmark/scorecard claim; every stronger claim must be supported by registry status and linked proof artifacts; keep unsupported claims blocked
7. update `CHANGELOG.md`
8. confirm CI is green
9. merge the release-ready state to `main`
10. create and push the release tag
11. only then announce the release

## Review Expectations

- explain what changed and why
- mention any file splits that were made to improve reuse or maintainability
- call out any user-visible behavior changes

## Security Reporting

Security issues should follow [`SECURITY.md`](../SECURITY.md).

## Project Rule

File splitting should happen only when the seam is real. Do not split code just
to make the tree look smaller.
