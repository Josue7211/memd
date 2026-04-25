---
date: 2026-04-25
phase: G4.6
run_kind: local-back-to-back
runs: 10
passes: 10
fails: 0
script: scripts/ci/v4-proof-harness.sh
commit: fd7691e
---

# G4.6 — V4 Proof Harness Stability Pass #1

First of the 7-day stability watch series (per `phase-g4-plan.md §G4.6`).
Ran the CI entrypoint 10× back-to-back on `research/mining` at commit
`fd7691e` (G4.5 just landed).

## Results

```
run 1: exit=0
run 2: exit=0
run 3: exit=0
run 4: exit=0
run 5: exit=0
run 6: exit=0
run 7: exit=0
run 8: exit=0
run 9: exit=0
run 10: exit=0
```

10/10 green. Each run executes `cargo test -p memd-client v4_proof_harness`
which exercises 12 tests (G4.2 driver, G4.3 asserters t3-t8, G4.4
scorecard regenerator t9/t10, G4.5 CI helpers t11/t12).

No infra-flake retries needed. No `continuity-breach.log` lines emitted
across any of the 10 runs (driver test asserts this per loop).

## Watch schedule

| date       | source     | status |
| ---------- | ---------- | ------ |
| 2026-04-25 | local 10×  | 10/10 green (this file) |
| 2026-04-26 | CI nightly | pending — workflow `.github/workflows/v4-proof-harness.yml` cron 03:00 UTC |
| 2026-04-27 | CI nightly | pending |
| 2026-04-28 | CI nightly | pending |
| 2026-04-29 | CI nightly | pending |
| 2026-04-30 | CI nightly | pending |
| 2026-05-01 | CI nightly | pending — earliest harvest for D4.8 / E4.7 / F4.7 7-day clocks |
| 2026-05-02 | CI nightly | pending — 7-day watch closes; G4.7 milestone-close gate eligible if 10/10 |

If any nightly fails: root-cause the source phase per `phase-g4-plan.md §G4.6`
(no retry — surface the regression). File a recovery phase if needed.

## Composite-rescore gate (G4.7 prerequisite)

Per `MILESTONE-v4.md`:
- composite_target = **3.45** (not 4.0 — regraded 2026-04-22 vs.
  `expected-cut-3.json::scorecard.composite_min` which still says 4.0;
  fixture-vs-milestone drift to reconcile in G4.7)
- regenerator must be invoked against real harness-observed axis scores
  from the dogfood window — not from this stability pass (which only
  proves the harness is deterministic, not that any axis lifted)
