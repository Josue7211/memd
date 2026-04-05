# Phase 27 Summary: `v4` Resume Current-Task Snapshot

## Completed

- added a `Current Task Snapshot` block to prompt-shaped resume output
- surfaced the same snapshot in generated bundle memory markdown
- exposed focus, pressure, next recovery, and lane together

## Verification

- `cargo test -q -p memd-client`
- `cargo test -q`

## Outcome

Operators can see the active task state immediately from the default resume
surfaces.
