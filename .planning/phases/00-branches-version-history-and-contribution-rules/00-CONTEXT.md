# Phase 0: Branches, Version History, and Contribution Rules - Context

**Gathered:** 2026-04-04
**Status:** Ready for planning

<domain>
## Phase Boundary

This phase establishes the OSS-ready project foundations that need to exist
before the memory roadmap keeps expanding: a real branch workflow, explicit
version history conventions, contribution and security rules, and targeted file
splitting in the large entrypoints where seams already exist.

</domain>

<decisions>
## Implementation Decisions

### Branch strategy
- **D-01:** Active development should move onto a dedicated branch rather than
  remaining on `main`.
- **D-02:** Branching should be an operational rule for future phased work, not
  an afterthought once the work is done.
- **D-03:** Release/version history should be documented as part of the repo's
  public workflow so future contributors can follow the same pattern.

### File splitting
- **D-04:** File splitting should happen only where seams already exist and the
  extracted module will be reused or independently understood.
- **D-05:** The first split targets are the large CLI and server entrypoints,
  but only at their existing helper boundaries.
- **D-06:** The goal is maintainability and reuse, not shrinking files for its
  own sake.

### Open source readiness
- **D-07:** Contribution, security, and review expectations must be explicit
  enough for an outside contributor to work without oral context.
- **D-08:** Public project guidance should be separated from internal planning
  artifacts so the repo reads like a maintained OSS project.

### the agent's Discretion
- Exact branch naming convention, as long as it is stable and obvious
- Whether to add a dedicated release/versioning doc or keep the guidance in the
  roadmap and contribution docs
- The smallest useful set of extracted modules for the entrypoint splits

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project strategy
- `ROADMAP.md` — version roadmap and the new `v0` OSS foundation milestone
- `.planning/ROADMAP.md` — execution-facing phase order and milestone split
- `.planning/PROJECT.md` — project framing and current planning boundaries
- `.planning/STATE.md` — current planning state and next command

### OSS guidance
- `README.md` — public project overview and current user-facing status
- `CONTRIBUTING.md` — contribution expectations and local workflow
- `SECURITY.md` — security reporting expectations
- `docs/reference/oss-positioning.md` — the public OSS shape and target audience

### Implementation seams
- `crates/memd-server/src/main.rs` — current server entrypoint with extractable
  helper boundaries
- `crates/memd-server/src/routing.rs` — existing example of a split-out module
- `crates/memd-server/src/keys.rs` — existing example of a split-out module
- `crates/memd-client/src/main.rs` — current CLI entrypoint with extractable
  rendering and command boundaries
- `crates/memd-client/src/obsidian.rs` — existing example of a split-out module

</canonical_refs>

<specifics>
## Specific Ideas

- The repo already has `LICENSE`, `CONTRIBUTING.md`, and `SECURITY.md`, so this
  phase is about making the workflow real and explicit, not inventing the first
  policy docs from scratch.
- `main.rs` files are large enough that extracting the existing helper clusters
  should improve reuse without changing public behavior.
- The branch strategy should be reflected in the roadmap and the working branch
  the agent uses during execution.

</specifics>

<deferred>
## Deferred Ideas

- Deeper memory-model work stays in `v1` and later
- Release automation can be added later if the branch/release conventions are
  stable
- Broader refactors outside the obvious seam boundaries are out of scope

</deferred>

---

*Phase: 00-branches-version-history-and-contribution-rules*
*Context gathered: 2026-04-04*
