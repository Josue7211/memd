# Phase 17 Context: `v4` Evaluation Failure Gates

## Why This Phase Exists

`memd eval` can now score bundle-backed memory quality, persist snapshots, and
compare against the latest baseline. That still leaves one operational gap:
automation cannot act on the result without parsing output manually.

This phase turns evaluation into a usable control surface for hooks, cron jobs,
and CI by adding deterministic failure gates.

## Inputs

- Phase 14: bundle memory evaluation foundations
- Phase 15: evaluation snapshot persistence
- Phase 16: evaluation regression diffs
- `README.md` bundle evaluation workflow

## Constraints

- keep the first gate slice local and deterministic
- fail only on explicit operator-provided gates
- preserve the current human-readable summary and JSON output
- do not add remote services or adaptive policy reactions yet

## Target Outcome

`memd eval` should be able to fail fast when:

- score drops below a required floor
- current evaluation regresses from the latest saved baseline
