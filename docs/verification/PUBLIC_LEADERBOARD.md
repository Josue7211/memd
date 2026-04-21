# memd public leaderboard

- generated_at: 2026-04-20T20:33:00+00:00
- rows: 4
- note: `make bench` / `benchmark public --all` currently rewrites `.memd/benchmarks/public/*` from the public-harness path and can overwrite the verified release board with stale lexical rows. Those stale generated snapshots were removed from `.memd/benchmarks/public/`. The `memd` column below restores the last verified release reruns from C3/D3 evidence; the `MemPalace` column is the F3 local same-fixture replay baseline.

## Claim Governance
- claim class, verification, regression budget, and replay evidence are first-class row fields
- run mode is benchmark execution mode; claim class stays dataset-native
- implemented adapters: `longmemeval`, `locomo`, `convomem`, `membench`
- declared parity targets: `longmemeval`, `locomo`, `convomem`, `membench`
- default regression budget: `0.020`
- stale overwritten public-harness rows `0.415 / 0.346 / 0.000` are not authoritative for the release board

| Benchmark | Version | Run mode | Coverage | Claim Class | Verification | Primary Metric | memd | MemPalace | Regression | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| ConvoMem | upstream | raw | real-dataset | cross-replayed | verified | accuracy (retrieval diagnostic) | 0.998 | 0.938 (replayed) | +0.000 / 0.020 | memd=`.monitor/c3/fast/convomem.log`; MemPalace=`.memd/benchmarks/baselines/mempalace-replays/convomem/latest/summary.json` | dataset=`.memd/benchmarks/datasets/convomem/convomem-evidence-sample.json`; stale public-harness snapshot removed from `.memd/benchmarks/public/` |
| LoCoMo | upstream | raw | real-dataset | cross-replayed | verified | evidence_hit_rate@5 (retrieval diagnostic) | 0.709 | 0.889 (replayed) | +0.000 / 0.020 | memd=`.monitor/c3/fast/locomo.log`; MemPalace=`.memd/benchmarks/baselines/mempalace-replays/locomo/latest/summary.json` | dataset=`.memd/benchmarks/datasets/locomo/locomo10.json`; restored from the verified release rerun called out in `docs/handoff/2026-04-20-c3-basis-landed-next-d3.md` |
| LongMemEval | upstream | raw | real-dataset | cross-replayed | verified-earlier | session_recall_any@5 (retrieval diagnostic) | 0.936 | 0.966 (replayed) | +0.000 / 0.020 | memd=`docs/handoff/2026-04-20-c3-basis-landed-next-d3.md`; MemPalace=`.memd/benchmarks/baselines/mempalace-replays/longmemeval/latest/summary.json` | dataset=`.memd/benchmarks/datasets/longmemeval/longmemeval_s_cleaned.json`; handoff explicitly says the leaderboard kept the earlier verified `0.936` because the fresh full rerun did not finish within session budget |
| MemBench | upstream | raw | real-dataset | cross-replayed | verified | target_hit_rate@5 (retrieval diagnostic) | 0.993 | 0.841 (replayed) | +0.000 / 0.020 | memd=`.monitor/c3/fast/membench.log`; MemPalace=`.memd/benchmarks/baselines/mempalace-replays/membench/latest/summary.json` | dataset=`.memd/benchmarks/datasets/membench/membench-firstagent.json`; stale public-harness snapshot removed from `.memd/benchmarks/public/` |
