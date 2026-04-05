# Phase 26 Summary: `v4` Claude Native Memory Bridge

## Completed

- added bundle-generated `CLAUDE_IMPORTS.md`
- added bundle-generated `CLAUDE.md.example`
- updated Claude integration docs to use imports and `/memory`
- treated dream/autodream as part of the Claude bridge contract

## Verification

- `cargo test -q -p memd-client`
- `cargo test -q`

## Outcome

Claude Code can load bundle memory through its native memory workflow while
`memd` remains the source of truth.
