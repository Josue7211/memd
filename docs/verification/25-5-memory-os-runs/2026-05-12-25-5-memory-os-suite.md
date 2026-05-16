# 25/5 Memory OS Focused Proof

- gates: 19/19 pass
- claim: implemented gates pass, including live FastEmbed RAG lift, process-level harness replay, and upstream LongMemEval/LoCoMo/MemBench/ConvoMem external smoke; full 25/5 market-best claim remains open until competitor-scale full-corpus runs pass.

| Pillar | Gate | Status | Log |
|---|---|---|---|
| recall | server-search-fabric | pass | `docs/verification/25-5-memory-os-runs/2026-05-12-server-search-fabric.log` |
| recall | server-no-rag-acceptance | pass | `docs/verification/25-5-memory-os-runs/2026-05-12-server-no-rag-acceptance.log` |
| recall | server-no-rag-public-corpus | pass | `docs/verification/25-5-memory-os-runs/2026-05-12-server-no-rag-public-corpus.log` |
| rag_booster | server-with-rag-acceptance | pass | `docs/verification/25-5-memory-os-runs/2026-05-12-server-with-rag-acceptance.log` |
| rag_booster | server-with-rag-public-corpus | pass | `docs/verification/25-5-memory-os-runs/2026-05-12-server-with-rag-public-corpus.log` |
| continuity | server-cross-harness-ollama | pass | `docs/verification/25-5-memory-os-runs/2026-05-12-server-cross-harness-ollama.log` |
| continuity | server-cross-harness-matrix | pass | `docs/verification/25-5-memory-os-runs/2026-05-12-server-cross-harness-matrix.log` |
| continuity | harness-process-replay | pass | `docs/verification/25-5-memory-os-runs/2026-05-12-harness-process-replay.log` |
| offline_sync | client-offline-store-queue | pass | `docs/verification/25-5-memory-os-runs/2026-05-12-client-offline-store-queue.log` |
| safety | ollama-prompt-firewall | pass | `docs/verification/25-5-memory-os-runs/2026-05-12-ollama-prompt-firewall.log` |
| rag_booster | server-rag-bridge | pass | `docs/verification/25-5-memory-os-runs/2026-05-12-server-rag-bridge.log` |
| rag_booster | live-server-sidecar-rag | pass | `docs/verification/25-5-memory-os-runs/2026-05-12-live-server-sidecar-rag.log` |
| model_selection | core-embedding-registry | pass | `docs/verification/25-5-memory-os-runs/2026-05-12-core-embedding-registry.log` |
| model_selection | client-embed-bench | pass | `docs/verification/25-5-memory-os-runs/2026-05-12-client-embed-bench.log` |
| model_selection | live-sidecar-embed-bench | pass | `docs/verification/25-5-memory-os-runs/2026-05-12-live-sidecar-embed-bench.log` |
| model_selection | live-sidecar-fastembed-bench | pass | `docs/verification/25-5-memory-os-runs/2026-05-12-live-sidecar-fastembed-bench.log` |
| rag_booster | live-rag-lift-corpus | pass | `docs/verification/25-5-memory-os-runs/2026-05-12-live-rag-lift-corpus.log` |
| public_benchmarks | public-benchmark-fixtures | pass | `docs/verification/25-5-memory-os-runs/2026-05-12-public-benchmark-fixtures.log` |
| public_benchmarks | external-public-smoke | pass | `docs/verification/25-5-memory-os-runs/2026-05-12-external-public-smoke.log` |
