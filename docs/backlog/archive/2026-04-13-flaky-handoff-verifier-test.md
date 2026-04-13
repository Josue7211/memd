# Flaky Handoff Verifier Test

Status: `closed`
Created: 2026-04-13
Phase: cross-phase

## Test

`runtime_verification_tests/mod.rs:1767` —
`run_verify_feature_command_executes_seeded_handoff_verifier`

## Behavior

Passes when run alone. Fails in full suite with connection refused on
`http://127.0.0.1:59999/memory/search`. Port collision or test ordering
interference with other tests that bind ports.

## Fix Direction

Use dynamic port allocation or mock the server connection in the test.
