---
phase: J3
name: V3 Floor Verification
version: v3
status: pending
opened: 2026-04-21
depends_on: [G3, H3, I3]
backlog_items: []
---

# Phase J3: V3 Floor Verification

## Goal

Run the paired intrinsic (sidecar-OFF) and accelerated (sidecar-ON) benchmark on all four public benches using the canonical metrics wired in H3 and the memd-backend dispatcher from G3. Publish the honest result — passing or failing the ≥0.70 floor — with full method-card transparency per I3. V3 ships on the truth of this run, not on pre-run optimism.

## Why this phase exists

V3's completion gate is "≥0.70 intrinsic on ALL four benches without the sidecar." That claim is meaningless until (a) the bench harness actually exercises memd retrieval (G3), (b) the metrics match the industry canon (H3), and (c) the leaderboard is audit-clean (I3). J3 is the measurement that all three enable.

## Deliver

1. **Paired run.** One invocation produces two passes over the four benches:
   - **Intrinsic**: `--backend memd --sidecar off`. memd-server running locally with RAG sidecar URL unset. This is the floor measurement.
   - **Accelerated**: `--backend memd --sidecar on` with RAG sidecar running. Secondary column.
2. **Canonical metric primary.** LongMemEval `qa_accuracy` (GPT-4o), LoCoMo `token_f1_avg`, MemBench `mc_accuracy`, ConvoMem `accuracy` per H3.
3. **Manifest stamp.** Both passes stamp: git SHA, fixture SHA, judge model, backend, sidecar state, total runtime, judge cost, `MEMD_RAG_TIMEOUT_MS`, `MEMD_RETRIEVAL_RAG_DENSE`. Machine-readable in `.memd/benchmarks/history/benchmark-runs.jsonl`.
4. **Pass/fail verdict per bench.** Each bench row gets `floor_status: pass | fail | borderline` where pass = ≥0.70 intrinsic, borderline = 0.67-0.70 (margin for noise), fail = <0.67.
5. **V3 completion decision.** If all four `pass`, V3 ships; bench-honesty roadmap closes. If any `fail` or `borderline`, ship the run anyway, mark V3 completion as `floor-missed-shipped-honestly`, file per-bench backlog items with recovery plans. **No silent reruns.** One run, one verdict, one artifact.
6. **Stranger-reproducibility check.** One outside reviewer (or a fresh clone + the reproduction command from the leaderboard) produces the same number within ±0.03. Evidence attached.

## Pass Gate

This phase's pass gate is meta: the **measurement** passes, not the **number**.

- pre: no reproducible paired-intrinsic/accelerated run on canonical metrics exists.
- post: one paired run completed end-to-end with both passes, manifest stamped, leaderboard regenerated with floor verdicts, stranger-reproduction confirmed.
- evidence: JSONL manifest entries for both passes; leaderboard diff showing verdicts; stranger-reproduction note signed off.
- regression budget: not applicable — this phase defines the canonical baseline going forward. Subsequent phases (post-V3) gate against these numbers.

**V3 completion gate** (owned by this phase per roadmap):
- Floor: ≥0.70 intrinsic on all four benches = V3 passes the competition floor.
- Stretch: LongMemEval ≥0.92, LoCoMo ≥0.80, MemBench ≥0.75, ConvoMem ≥0.75 = above-and-beyond.
- Accelerated delta: sidecar ON ≥ intrinsic + 0.02 per bench (or sidecar is not earning its keep).
- Dogfood: 5 surfaces still pass the stranger-test from ROADMAP V3 block, run with sidecar OFF.

## Evidence

- Paired manifest (intrinsic + accelerated) in JSONL
- Regenerated leaderboard with `floor_status` column
- Stranger-reproduction note (who, when, delta)
- Judge-cost ledger for the run
- If floor missed: recovery backlog items filed per failed bench

## Product Win

memd publishes a score it stands behind. Whether that score clears the 70% competition floor or misses it, the publication itself is the product win — the AI memory space is full of retracted/contested numbers; memd's number doesn't join that pile.

## Fail Conditions

- Paired run completes but floor is missed AND the team reruns silently looking for a better number → fail; ship the first complete run or explicitly document why (e.g., discovered bug mid-run requiring fix).
- Stranger-reproduction disagrees by >0.03 on any bench → harness is not deterministic; fix (seed, cache, fixture pinning) before shipping.
- Accelerated ≤ intrinsic on any bench → sidecar is degrading quality, not helping; file blocker and investigate before V3 ships.
- Judge cost exceeds budget → reduce item_count and re-run transparently with smaller-N note.

## Donor Anchors

- **J3-D1**: `make bench-public` + new `bench-public-memd` targets (G3 output)
- **J3-D2**: `PUBLIC_LEADERBOARD.md` method-card format (I3 output)
- **J3-D3**: existing paired intrinsic/accelerated harness scaffolding (`resolve_public_benchmark_dual_memd_base_urls` in `public_benchmark.rs:2432`)

## Rollback

This is a measurement run, not a code ship. Rollback = retract the published leaderboard row and re-run. The whole point of I3's retraction log is making this rollback cheap.

## Out of scope

- Retrieval quality improvements to clear a missed floor — loops back to B3/C3/D3, not part of J3
- New benchmarks beyond the four — separate phase
- Post-V3 continuous-gate design — future milestone
