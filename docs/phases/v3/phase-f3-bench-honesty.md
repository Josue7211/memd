---
phase: F3
name: Bench Honesty
version: v3
status: complete
depends_on: [A3, B3, C3, D3, E3]
notes: Renamed E3→F3 on 2026-04-17 when new A3 (memd Continuity Foundation) was inserted at V3 entry. Deliverable 1 (ConvoMem adapter fix) remains parallelizable with B3 Intrinsic Retrieval since it is an adapter bug, not a retrieval-quality problem.
backlog_items:
  - "2026-04-14-no-public-benchmark-parity"
  - "2026-04-14-no-behavior-changing-recall-proof"
  - "2026-04-14-no-data-recovery-procedure"
---

# Phase F3: Bench Honesty

## Goal

Make every leaderboard claim defensible. ConvoMem adapter truth is already repaired upstream, MemPalace cross-baseline is now replayed locally on memd fixtures, and [[docs/verification/PUBLIC_LEADERBOARD.md]] refreshes from first-class replay artifacts instead of note text.

## Why this phase exists

Before this phase, the public leaderboard shipped four bench rows with a "no MemPalace cross-baseline has been replayed yet" disclaimer. The ConvoMem zero was real roadmap debt when this phase was written, but that adapter bug has already been fixed; the remaining credibility gap was that the MemPalace column still came from published notes instead of local replay artifacts.

## Deliver

1. **ConvoMem adapter audit** — read upstream dataset shape (`Salesforce/ConvoMem` evidence_questions), trace memd's adapter end to end, identify mismatch (likely candidate: query → retrieval-route mapping or evidence-id matching). Write failing test reproducing the 0.000 score, fix, prove green.
2. **MemPalace cross-baseline replay** — run mempalace's reference benchmark binary on the same fixture, record its score, ship side-by-side comparison in leaderboard. Removes the "dataset-grade / retrieval-local" disclaimer from claim class.
3. **Per-phase leaderboard refresh** — `make bench` regenerates [[docs/verification/PUBLIC_LEADERBOARD.md]] with timestamp + commit; CI gate on V3 PRs requires leaderboard touch.
4. **Bench claim governance** — every row in leaderboard carries: (a) claim class, (b) verification status (verified / recorded-unpinned / replay-pending), (c) regression-budget against last verified score, (d) link to commit that produced it.
5. **Bench rerun on schema drift** — when retrieval pipeline changes (B3/C3/D3/E3 merges), CI regenerates leaderboard automatically. No silent score regression possible.

## Pass Gate

Bench-delta required (regenerate [[docs/verification/PUBLIC_LEADERBOARD.md]]):

- pre: MemPalace cross-baseline=`missing`, leaderboard rows=`replay-pending`, ConvoMem adapter still not replay-backed locally
- post: **ConvoMem = 0.903 intrinsic** on memd's public fixture and **MemPalace cross-baseline live for all 4 benches** (`LongMemEval=0.966`, `LoCoMo=0.889`, `ConvoMem=0.938`, `MemBench=0.841`)
- regression budget: no other metric drops > 0.02
- evidence: leaderboard regenerated with mempalace side-by-side column and replay artifact paths

Plus:
- `cargo test -p memd-client` ConvoMem regression test green (was failing pre-fix)
- `make bench-public` runs end to end on CI without manual fixture loads
- All four leaderboard rows have `verification: verified` (none `recorded-unpinned`)

## Evidence

- ConvoMem adapter trace before/after fix
- MemPalace replay log + score table
- Regenerated leaderboard with side-by-side cross-baseline column
- CI run showing automated leaderboard refresh

## Product Win

- **Leaderboard is a page a stranger can verify.** Every row links to the commit that produced it + the fixture it ran on + the rerun command. No "trust us" claims.
- **Regressions are loud, not silent.** A score drop shows up in CI before a PR lands; the PR description carries the delta.
- **Cross-baseline is first-class.** MemPalace column sits next to memd column on every row, backed by local replay artifact bundles; honesty beats optics.

Evidence:
- Stranger-test: someone outside the project picks a leaderboard row, reruns it from the commit link, gets the same number within regression budget
- CI log showing a deliberate regression caught and blocked
- Per-row verification-state column visible at a glance (verified / replay-pending / recorded-unpinned)

## Fail Conditions

- ConvoMem stays below 0.70 intrinsic — adapter fix alone wasn't enough; this is now retrieval quality, loop back to B3/C3/D3 for the missing gains
- ConvoMem stays at 0.000 — diagnosis was wrong; likely retrieval issue not adapter; loop back to B3
- MemPalace replay score lower than memd's claim — retract the claim, surface honestly
- Leaderboard refresh breaks CI on every PR — gate scope is wrong; narrow to V3-touched PRs only
- Side-by-side comparison shows mempalace beating memd on a metric memd claims to lead — retract lead claim

## Donor Anchors

- **F3-D1**: mempalace benchmark harness (longmemeval_bench.py, multi-model sweep) — [[.memd/lanes/architecture/A2-01-benchmark-harness.md]]
- **F3-D2**: bench claim governance and verification states — [[docs/verification/PUBLIC_BENCHMARKS.md]]

## Rollback

- ConvoMem adapter fix can be feature-flagged; if regression on other benches, revert
- MemPalace replay is read-only — no rollback needed
- Per-phase leaderboard refresh CI gate can be relaxed without code revert (CI config only)

## Out of scope

- New benchmark adoption (ScrollBench, etc.) — separate future phase
- LLM-graded retrieval evaluation (already covered by existing benchmark-registry)
- Internal dogfood metrics (different surface)

## Why F3 lives last in V3

F3 only makes sense after B3+C3+D3+E3 have moved the numbers. Refreshing the leaderboard before retrieval is fixed just publishes bad scores faster. That is why the phase ended up as a trust layer over already-landed retrieval work: ConvoMem was lifted off zero earlier, and F3 closed the remaining honesty gap by replaying MemPalace locally on the same memd-owned fixtures.
