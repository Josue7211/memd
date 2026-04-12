> Secondary/reference doc. For current project truth start with [[ROADMAP]] and [[docs/WHERE-AM-I.md|WHERE-AM-I]].

# memd public benchmark suite

- latest_runs: 4
- supported_targets: longmemeval, locomo, convomem, membench
- implemented_adapters: longmemeval, locomo, convomem, membench
- newest_run: locomo mode=hybrid at 2026-04-10T16:39:12.404137064+00:00

## Target Inventory
- longmemeval: implemented
- locomo: implemented
- convomem: implemented
- membench: implemented
- implemented adapters: longmemeval, locomo, convomem, membench

## Latest Runs
| Benchmark | Version | Mode | Headline metric | Value | Items | Dataset | Checksum | Artifacts |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| ConvoMem | upstream | raw | Recall Fraction@5 | 1.000 | 10 | .memd/benchmarks/datasets/convomem/convomem-evidence-sample-10-per-category.json | sha256:65ec7bb06bbbc1bf169b3cb31a722f763c7c0cbce7b837c1b084215ffb9e2de9 | `.memd/benchmarks/public/convomem/latest/` |
| LoCoMo | upstream | hybrid | Recall Fraction@5 | 0.750 | 10 | .memd/benchmarks/datasets/locomo/locomo10.json | sha256:79fa87e90f04081343b8c8debecb80a9a6842b76a7aa537dc9fdf651ea698ff4 | `.memd/benchmarks/public/locomo/latest/` |
| LongMemEval | upstream | hybrid | Accuracy | 1.000 | 10 | .memd/benchmarks/datasets/longmemeval/longmemeval_s_cleaned.json | sha256:d6f21ea9d60a0d56f34a05b609c79c88a451d2ae03597821ea3d5a9678c3a442 | `.memd/benchmarks/public/longmemeval/latest/` |
| MemBench | upstream | hybrid | Grounded Choice Accuracy | 1.000 | 10 | .memd/benchmarks/datasets/membench/membench-firstagent.json | sha256:54bde8259c10ee1cfe5ff16f35a8a25ca9ad5d79e162e0b3a43034ed64115e5a | `.memd/benchmarks/public/membench/latest/` |

## Artifacts
- convomem: `.memd/benchmarks/public/convomem/latest/manifest.json`, `.memd/benchmarks/public/convomem/latest/results.json`, `.memd/benchmarks/public/convomem/latest/results.jsonl`, `.memd/benchmarks/public/convomem/latest/report.md`
- locomo: `.memd/benchmarks/public/locomo/latest/manifest.json`, `.memd/benchmarks/public/locomo/latest/results.json`, `.memd/benchmarks/public/locomo/latest/results.jsonl`, `.memd/benchmarks/public/locomo/latest/report.md`
- longmemeval: `.memd/benchmarks/public/longmemeval/latest/manifest.json`, `.memd/benchmarks/public/longmemeval/latest/results.json`, `.memd/benchmarks/public/longmemeval/latest/results.jsonl`, `.memd/benchmarks/public/longmemeval/latest/report.md`
- membench: `.memd/benchmarks/public/membench/latest/manifest.json`, `.memd/benchmarks/public/membench/latest/results.json`, `.memd/benchmarks/public/membench/latest/results.jsonl`, `.memd/benchmarks/public/membench/latest/report.md`

## Latest Run Detail: LoCoMo

### Category Breakdown
| Category | Metric | Value |
| --- | --- | --- |
| Single-hop | recall_any@5 | 0.667 |
| Single-hop | recall_fraction@5 | 0.500 |
| Temporal | recall_any@5 | 1.000 |
| Temporal | recall_fraction@5 | 1.000 |
| Temporal-inference | recall_any@5 | 0.000 |
| Temporal-inference | recall_fraction@5 | 0.000 |

| Item | Question | Claim | Hit | Answer | Latency ms |
| --- | --- | --- | --- | --- | --- |
| conv-26::0 | When did Caroline go to the LGBTQ support group? | raw | true | 7 May 2023 | 3717 |
| conv-26::1 | When did Melanie paint a sunrise? | raw | true | 2022 | 3646 |
| conv-26::2 | What fields would Caroline be likely to pursue in her educaton? | raw | false | Psychology, counseling certification | 786 |
| conv-26::3 | What did Caroline research? | raw | true | Adoption agencies | 3249 |
| conv-26::4 | What is Caroline's identity? | raw | false | Transgender woman | 679 |
| conv-26::5 | When did Melanie run a charity race? | raw | true | The sunday before 25 May 2023 | 3702 |
| conv-26::6 | When is Melanie planning on going camping? | raw | true | June 2023 | 3599 |
| conv-26::7 | What is Caroline's relationship status? | raw | true | Single | 891 |
| conv-26::8 | When did Caroline give a speech at a school? | raw | true | The week before 9 June 2023 | 3418 |
| conv-26::9 | When did Caroline meet up with her friends, family, and mentors? | raw | true | The week before 9 June 2023 | 3462 |