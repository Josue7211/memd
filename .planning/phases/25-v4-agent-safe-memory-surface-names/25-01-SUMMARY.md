# Phase 25 Summary: `v4` Agent-Safe Memory Surface Names

## Completed

- changed the shared bundle root file to `MEMD_MEMORY.md`
- kept agent-specific memory files intact
- updated README and integration docs to reference the new root file
- updated tests that read the generated memory placeholder

## Verification

- `cargo test -q -p memd-client`
- `cargo test -q`

## Outcome

The bundle memory surface is now safe to use alongside agents that already own
their own `MEMORY.md` conventions.
