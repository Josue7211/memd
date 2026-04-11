# Phase 0 Summary: Branches, Version History, and Contribution Rules

**Completed:** 2026-04-04
**Status:** Complete

## Outcome

Phase 0 is complete. The repository now has an OSS-first foundation:

- active work moved onto `work/v0-oss-foundations`
- public guidance for branching, release/version history, contribution, and security is documented
- changelog and code-of-conduct files exist for public collaboration
- the CLI entrypoint has reusable render and command helpers split out
- the server entrypoint has reusable inspection, repair, and working-memory helper modules split out

## Files Added

- `docs/policy/release-process.md`
- `docs/policy/branching.md`
- `CHANGELOG.md`
- `CODE_OF_CONDUCT.md`
- `crates/memd-client/src/render.rs`
- `crates/memd-client/src/commands.rs`
- `crates/memd-server/src/inspection.rs`
- `crates/memd-server/src/repair.rs`
- `crates/memd-server/src/working.rs`

## Files Updated

- `ROADMAP.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/REQUIREMENTS.md`
- `README.md`
- `CONTRIBUTING.md`
- `docs/reference/oss-positioning.md`
- `crates/memd-client/src/main.rs`
- `crates/memd-server/src/main.rs`

## Verification

- `cargo test -q -p memd-server`
- `cargo test -q -p memd-client`
- `cargo test -q`

## Notes

- File splitting was limited to the seams already present in the code.
- The public memory routes and CLI surface were kept stable.
