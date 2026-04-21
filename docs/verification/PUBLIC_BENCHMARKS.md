# memd public benchmark suite

- latest_verified_release_board: 2026-04-20T20:33:00+00:00
- supported_targets: longmemeval, locomo, convomem, membench
- implemented_adapters: longmemeval, locomo, convomem, membench
- warning: the auto-generated `.memd/benchmarks/public/*` snapshots were later overwritten by the public-harness path and were removed because they were not authoritative for release-board truth. Use the table below for the last verified release board and the F3 replay baselines.

## Target Inventory
- longmemeval: implemented
- locomo: implemented
- convomem: implemented
- membench: implemented

## Latest Verified Release Board
| Benchmark | Mode | Primary Metric | memd | MemPalace Replay | memd Evidence | Replay Artifacts |
| --- | --- | --- | --- | --- | --- | --- |
| LongMemEval | raw | session_recall_any@5 (retrieval diagnostic) | 0.936 | 0.966 | `docs/handoff/2026-04-20-c3-basis-landed-next-d3.md` | `.memd/benchmarks/baselines/mempalace-replays/longmemeval/latest/` |
| LoCoMo | raw | evidence_hit_rate@5 (retrieval diagnostic) | 0.709 | 0.889 | `.monitor/c3/fast/locomo.log` | `.memd/benchmarks/baselines/mempalace-replays/locomo/latest/` |
| ConvoMem | raw | accuracy (retrieval diagnostic) | 0.998 | 0.938 | `.monitor/c3/fast/convomem.log` | `.memd/benchmarks/baselines/mempalace-replays/convomem/latest/` |
| MemBench | raw | target_hit_rate@5 (retrieval diagnostic) | 0.993 | 0.841 | `.monitor/c3/fast/membench.log` | `.memd/benchmarks/baselines/mempalace-replays/membench/latest/` |

## Authoritative Sources
- memd release-board numbers:
  - `docs/handoff/2026-04-20-c3-basis-landed-next-d3.md`
  - `.monitor/c3/fast/locomo.log`
  - `.monitor/c3/fast/convomem.log`
  - `.monitor/c3/fast/membench.log`
- MemPalace replay baselines:
  - `.memd/benchmarks/baselines/mempalace_replays.json`
  - `.memd/benchmarks/baselines/mempalace-replays/*/latest/summary.json`

## Stale Snapshot Warning
- The overwritten public-harness snapshots under `.memd/benchmarks/public/` were removed because they showed stale lexical rows. The generator path still needs a fix before those artifacts can be trusted again.
