# memd public leaderboard

- generated_at: 2026-04-14T02:59:53.424218691+00:00
- rows: 4

## Claim Governance
- fixture-backed run; this is not a full MemPalace parity claim
- run mode is benchmark execution mode; claim class is the per-item label
- implemented mini adapters: longmemeval, locomo, convomem, membench
- declared parity targets: longmemeval, locomo, convomem, membench
- real upstream dataset runs use benchmark-shaped metrics with memd's local retrieval backend; do not treat them as full MemPalace parity yet

| Benchmark | Version | Run mode | Item claim classes | Coverage | Parity claim | Primary Metric | Value | Items | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| ConvoMem | upstream | raw | raw | real-dataset | dataset-grade / retrieval-local | accuracy | 1.000 | 10 | primary_metric=accuracy; dataset=.memd/benchmarks/datasets/convomem/convomem-evidence-sample-10-per-category.json; checksum=sha256:65ec7bb06bbbc1bf169b3cb31a722f763c7c0cbce7b837c1b084215ffb9e2de9; source=https://huggingface.co/datasets/Salesforce/ConvoMem/tree/main/core_benchmark/evidence_questions; no MemPalace cross-baseline has been replayed yet; verification=recorded-unpinned; headline accuracy uses benchmark-shaped metrics over memd's local retrieval backend, not full MemPalace parity infrastructure yet |
| LoCoMo | upstream | raw | raw | fixture-backed | dataset-grade / retrieval-local | evidence_hit_rate@5 (retrieval proxy) | 0.415 | 1986 | primary_metric=evidence_hit_rate@5 (retrieval proxy); dataset=.memd/benchmarks/datasets/locomo/locomo10.json; checksum=sha256:79fa87e90f04081343b8c8debecb80a9a6842b76a7aa537dc9fdf651ea698ff4; source=file://.memd/benchmarks/datasets/locomo/locomo10.json; no MemPalace cross-baseline has been replayed yet; verification=manual-path; headline accuracy uses benchmark-shaped metrics over memd's local retrieval backend, not full MemPalace parity infrastructure yet |
| LongMemEval | upstream | raw | raw | fixture-backed | dataset-grade / retrieval-local | session_recall_any@5 (retrieval proxy) | 0.828 | 500 | primary_metric=session_recall_any@5 (retrieval proxy); dataset=.memd/benchmarks/datasets/longmemeval/longmemeval_s_cleaned.json; checksum=sha256:d6f21ea9d60a0d56f34a05b609c79c88a451d2ae03597821ea3d5a9678c3a442; source=file://.memd/benchmarks/datasets/longmemeval/longmemeval_s_cleaned.json; no MemPalace cross-baseline has been replayed yet; verification=manual-path; headline accuracy uses benchmark-shaped metrics over memd's local retrieval backend, not full MemPalace parity infrastructure yet |
| MemBench | upstream | raw | raw | fixture-backed | dataset-grade / retrieval-local | target_hit_rate@5 (retrieval proxy) | 0.346 | 3000 | primary_metric=target_hit_rate@5 (retrieval proxy); dataset=.memd/benchmarks/datasets/membench/membench-firstagent.json; checksum=sha256:54bde8259c10ee1cfe5ff16f35a8a25ca9ad5d79e162e0b3a43034ed64115e5a; source=file://.memd/benchmarks/datasets/membench/membench-firstagent.json; no MemPalace cross-baseline has been replayed yet; verification=manual-path; headline accuracy uses benchmark-shaped metrics over memd's local retrieval backend, not full MemPalace parity infrastructure yet |