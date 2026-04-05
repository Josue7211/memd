# Summary 43-01: `v5` Coordination Subscription and Hook Surfaces

## Goal

Expose coordination changes as a reusable signal instead of leaving change
detection trapped inside the watch loop.

## What Changed

1. Added a persisted coordination snapshot under bundle state so coordination
   deltas can be computed across invocations.
2. Added `--changes-only` to the CLI coordination surface for one-shot,
   hook-friendly change inspection.
3. Extracted compact coordination snapshot and alert rendering into reusable
   helpers instead of keeping that logic embedded in watch-only flow.
4. Kept the change feed aligned with the same bounded categories already used by
   dashboard, drilldown, and watch views.

## Verification

- `cargo test -q`

## Result

Coordination change detection is now reusable outside the live watch loop, so
hooks and other downstream surfaces can consume bounded coworking deltas
directly.
