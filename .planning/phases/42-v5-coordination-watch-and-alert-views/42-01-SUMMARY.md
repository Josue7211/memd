# Summary 42-01: `v5` Coordination Watch and Alert Views

## Goal

Make coordination pressure easier to monitor during active coworking without
turning the output into a transcript feed.

## What Changed

1. Added `--watch` and `--interval-secs` to the CLI coordination surface.
2. Reused the same bounded `view` categories so watch mode can track `all`,
   `inbox`, `requests`, `recovery`, `policy`, or `history`.
3. Added compact alert lines that only print when the selected coordination
   slice changes.
4. Kept watch output bounded by pairing those alert lines with the current
   filtered dashboard summary instead of replaying raw state on every loop.

## Verification

- `cargo test -q`

## Result

Operators can keep coordination pressure visible as it changes instead of
manually rerunning the same static summary over and over.
