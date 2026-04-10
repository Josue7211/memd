# memd public leaderboard

- generated_at: 2026-04-10T04:22:14.779616748+00:00
- rows: 4

## Claim Governance
- fixture-backed run; this is not a full MemPalace parity claim
- run mode is benchmark execution mode; claim class is the per-item label
- implemented mini adapters: longmemeval, locomo, convomem, membench
- declared parity targets: longmemeval, locomo, convomem, membench
- real upstream dataset runs use benchmark-shaped metrics with memd's local retrieval backend; do not treat them as full MemPalace parity yet

| Benchmark | Version | Run mode | Item claim classes | Coverage | Parity claim | Accuracy | Items | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| ConvoMem | upstream | raw | raw | real-dataset | dataset-grade / retrieval-local | 1.000 | 20 | dataset=.memd/benchmarks/datasets/convomem/convomem-evidence-sample.json; checksum=sha256:34238ace63b0a5393833bb213696870cef733f3520066f6647f4c7355be5dd07; source=https://huggingface.co/datasets/Salesforce/ConvoMem/tree/main/core_benchmark/evidence_questions; no MemPalace cross-baseline has been replayed yet; verification=recorded-unpinned; headline accuracy uses benchmark-shaped metrics over memd's local retrieval backend, not full MemPalace parity infrastructure yet |
| LoCoMo | upstream | raw | raw | real-dataset | dataset-grade / retrieval-local | 1.000 | 20 | dataset=.memd/benchmarks/datasets/locomo/locomo10.json; checksum=sha256:79fa87e90f04081343b8c8debecb80a9a6842b76a7aa537dc9fdf651ea698ff4; source=https://raw.githubusercontent.com/snap-research/locomo/3eb6f2c585f5e1699204e3c3bdf7adc5c28cb376/data/locomo10.json; no MemPalace cross-baseline has been replayed yet; verification=verified; headline accuracy uses benchmark-shaped metrics over memd's local retrieval backend, not full MemPalace parity infrastructure yet |
| LongMemEval | upstream | raw | raw | real-dataset | dataset-grade / retrieval-local | 0.950 | 20 | dataset=.memd/benchmarks/datasets/longmemeval/longmemeval_s_cleaned.json; checksum=sha256:d6f21ea9d60a0d56f34a05b609c79c88a451d2ae03597821ea3d5a9678c3a442; source=https://huggingface.co/datasets/xiaowu0162/longmemeval-cleaned/resolve/main/longmemeval_s_cleaned.json; no MemPalace cross-baseline has been replayed yet; verification=verified; headline accuracy uses benchmark-shaped metrics over memd's local retrieval backend, not full MemPalace parity infrastructure yet |
| MemBench | upstream | raw | raw | real-dataset | dataset-grade / retrieval-local | 1.000 | 20 | dataset=.memd/benchmarks/datasets/membench/membench-firstagent.json; checksum=sha256:54bde8259c10ee1cfe5ff16f35a8a25ca9ad5d79e162e0b3a43034ed64115e5a; source=https://github.com/import-myself/Membench/tree/f66d8d1028d3f68627d00f77a967b93fbb8694b6/MemData/FirstAgent; no MemPalace cross-baseline has been replayed yet; verification=recorded-unpinned; headline accuracy uses benchmark-shaped metrics over memd's local retrieval backend, not full MemPalace parity infrastructure yet |