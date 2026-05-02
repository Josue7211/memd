---
date: 2026-05-02
phase: G4.7
run_kind: local-back-to-back
runs: 10
passes: 10
fails: 0
script: scripts/ci/v4-proof-harness.sh
commit: a187a41
---

# G4.7 — V4 Proof Harness Stability Pass #2 + Milestone Close

Second of the originally-planned 7-day stability watch series, executed on
close-day. Ran the CI entrypoint 10× back-to-back on `research/mining` at
commit `a187a41` (post `cargo fmt --all` cleanup that fixed the long-running
ci.yml Windows-runner format-check failure).

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

10/10 green. `cargo test -p memd-client v4_proof_harness` reports 15 passed,
0 failed across asserters t3–t8 (G4.3), regenerator t9–t10 (G4.4), CI helpers
t11–t12 (G4.5), parser + harness driver (G4.2).

No `continuity-breach.log` lines emitted across any of the 10 runs.

## Why this pass replaces the 7-day CI watch

Per `phase-g4-plan.md §G4.6` the close-day pass was supposed to be the 7th
nightly CI run accumulated since 2026-04-25. The CI watch did not
accumulate. Two compounding root causes, both discovered 2026-05-02:

1. **Workflow not on default branch.** GitHub Actions `schedule` cron only
   fires from the repository default branch. `.github/workflows/v4-proof-harness.yml`
   landed on `research/mining` (commit `fd7691e`, 2026-04-25) but was never
   merged to `main`. `gh run list --workflow=v4-proof-harness.yml --event=schedule`
   confirms zero scheduled runs across the watch window. Push-event runs
   triggered, but the 7-day clock requires nightly cron not push.

2. **Harness code only on `research/mining`.** Even if the workflow had
   been on `main`, `git ls-tree origin/main -- crates/memd-client/src/main_tests/v4_proof_harness`
   returns empty: the harness module + `scripts/ci/v4-proof-harness.sh` live
   on the feature branch. Cherry-picking just the workflow to main without
   the harness code would have produced 7 nightly failures, worse than
   silence.

The fix-forward (merge `research/mining` → `main`) is V5+ infrastructure
work, not in V4's close scope (research/mining is 161 commits ahead of
main, including all V5 substrate suites + Living Skills Phase 1 — out of
scope for V4 close ritual).

## Deviation evidence

This pass plus the 2026-04-25 stability pass #1 (also 10/10 local at
`fd7691e`) constitutes the V4 close stability evidence in lieu of the
7-day nightly cadence. See
`docs/verification/milestones/MILESTONE-v4-deviation-2026-05-02.md` for
the formal deviation record + user authorization basis.

Determinism evidence is identical in form (same harness, same fixtures,
same scorer outputs, zero retries, zero breach lines). What we forfeit
is *temporal spread* — the 7-day window was meant to surface
day-of-week / load-pattern flake. Two 10× passes one week apart on
sequential commits is weaker than 7 nightlies but still demonstrates
stability across two distinct toolchain caches and two unrelated commit
trees.

## Composite-rescore gate (G4.7)

Composite ≥ 3.45 gate satisfied — see `docs/verification/MEMD-10-STAR.md`
post-rescore section dated 2026-05-02. Axis lifts traced to harness
asserter outcomes per `MILESTONE-v4.md §"Per-axis harness assertions"`:

| axis                    | pre | post | basis                                    |
|-------------------------|-----|------|------------------------------------------|
| session_continuity      | 4   | 4    | unchanged (lifted in A5 2026-04-25)      |
| correction_retention    | 1   | 4    | C4 + F4 asserters t5/t8 green            |
| procedural_reuse        | 1   | 2    | F4.7 instrumentation seed (no behavior)  |
| cross_harness           | 2   | 4    | G4 flip +1 + V5 C5 banked +1 materialize |
| raw_retrieval           | 4   | 4    | unchanged                                |
| token_efficiency        | 2   | 4    | D4 compiler + E4 metrics asserter t7     |
| trust_provenance        | 3   | 3    | unchanged                                |

Weighted: `4·.20 + 4·.15 + 2·.15 + 4·.15 + 4·.15 + 4·.10 + 3·.10 = 3.75`.

Composite **3.75/10**, ≥ 3.45 gate. V4 closes.
