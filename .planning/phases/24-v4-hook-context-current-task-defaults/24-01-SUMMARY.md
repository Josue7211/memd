# Phase 24 Summary: `v4` Hook Context Current-Task Defaults

## Completed

- defaulted `memd hook context` to `current_task` intent
- updated Unix and PowerShell hook scripts to use `current_task`
- updated hook documentation to reflect the new default

## Verification

- `cargo test -q -p memd-client`
- `cargo test -q`

## Outcome

The default hook-context path now uses the same short-term retrieval lane as
the rest of the bundle launch surface.
