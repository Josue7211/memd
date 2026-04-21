---
status: open
severity: medium
phase: H3
opened: 2026-04-21
scope: memd-client / public-benchmark
---
# MemBench MQI Composite Weights Undisclosed

- status: `open`
- severity: `medium`
- phase: `H3 (Canonical Metrics)`
- opened: `2026-04-21`
- scope: memd-client / public-benchmark
- upstream: arXiv 2506.21605 (MemBench, ACL Findings 2025)

## Problem

MemBench's headline metric is the **Memory Quality Index** composite:

```
MQI = ω₁·Accuracy + ω₂·Efficiency + ω₃·Capacity
```

The three weights `ω₁`, `ω₂`, `ω₃` are not disclosed in the public
MemBench paper, repository, or dataset card as of 2026-04-21. Without
pinned weights, memd cannot reproduce MQI deterministically and cannot
claim an MQI score comparable to upstream's own reporting.

## Why It Matters

- Phase H3 replaces retrieval-proxy metrics with industry-canonical
  primary metrics on every public bench. MemBench's canonical is MQI.
- Without MQI, memd's MemBench leaderboard row falls back to
  `mc_accuracy` as a sanity metric. That keeps us in the game but does
  not satisfy the H3 pass-gate for "measured by the same yardstick
  upstream and competitors use".

## Workaround Shipped in H3

- MemBench leaderboard row reports `mc_accuracy` as primary.
- Row carries an explicit disclaimer: "MQI composite deferred pending
  upstream weight disclosure (see
  docs/backlog/v3/2026-04-21-membench-mqi-weights-undisclosed.md)."
- No memd score >0.90 may be claimed on MemBench until MQI is
  reproducible.

## Resolution Paths

1. **Direct upstream contact** — email / GitHub issue to the MemBench
   authors requesting the three weights. Preferred path.
2. **Reverse-engineer from paper numbers** — if the paper reports both
   MQI and its three sub-metrics per system, solve the 3×3 linear system
   for (ω₁, ω₂, ω₃).
3. **Pick-and-document defaults** — last resort: adopt (1/3, 1/3, 1/3)
   equal weights, disclose prominently in method card, do not claim
   apples-to-apples MemBench parity.

## Acceptance Criteria for Closing

- Weights pinned in `crates/memd-client/src/benchmark/public_benchmark.rs`
  with a primary-source citation (upstream confirmation, published
  paper table, or documented default).
- MemBench row primary metric switches to `mqi` with
  `mc_accuracy` demoted to diagnostic.
- Backlog entry gets `status: resolved` stamp and the close commit
  references the pinning commit.
