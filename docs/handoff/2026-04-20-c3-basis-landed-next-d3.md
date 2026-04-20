---
date: 2026-04-20
phase: C3
part: basis
status: checkpoint
next_phase: D3
next_part: atlas-at-recall
branch: research/mining
base_head: 9068771
tests: targeted C3 build + sidecar/server/client tests green; fresh release reruns for LoCoMo/MemBench/ConvoMem
---
# C3 basis landed, next is D3 Atlas at Recall

## TL;DR

C3 implementation basis is in.

- sidecar `/v1/rerank` exists
- server uses rerank after dense blend
- local cross-encoder rerank works
- Anthropic/Haiku rerank path exists when `ANTHROPIC_API_KEY` is present
- embed-model selection + model stamping + re-embed sweep landed
- ConvoMem adapter is honest now

Fresh release-binary reruns:

| benchmark | value |
| --- | --- |
| LoCoMo | 0.7089627391742196 |
| MemBench | 0.9926666666666667 |
| ConvoMem | 0.9980952380952381 |

So C3 is good enough to move on in implementation terms. The one missing
piece is a fresh full `LongMemEval` rerun on the final release C3 runtime.
It was started, but did not finish within session budget. Roadmap-wise,
next move is `D3 Atlas at Recall`, then rerun the full board after real D3
progress.

## What changed

- `crates/memd-sidecar/src/main.rs`
  - added `/v1/rerank`
  - local `bge-reranker-base` runtime
  - Anthropic rerank primary with local fallback
  - per-record embedding cache eviction on update
  - embed model selection from `MEMD_EMBED_MODEL`
- `crates/memd-sidecar/src/lib.rs`
  - rerank request/response contract
- `crates/memd-rag/src/lib.rs`
  - rerank client wrapper
- `crates/memd-server/src/rag_bridge.rs`
  - server->sidecar rerank bridge
- `crates/memd-server/src/routes.rs`
  - rerank after dense blend
- `crates/memd-server/src/embed.rs`
  - query prefix `query: `
  - passage prefix `passage: `
  - model-aware dimensions / codes
- `crates/memd-server/src/main.rs`
  - model-aware vector writes
  - background re-embed sweep
- `crates/memd-server/src/store.rs`
- `crates/memd-server/src/store_migrations.rs`
  - `embedding_model` columns / filtering / migration
- `crates/memd-client/src/bundle/mod.rs`
- `crates/memd-client/src/bundle/init_runtime/mod.rs`
  - bundle backend now carries `embedding_model`
  - `backend.env` exports `MEMD_EMBED_MODEL`
- `crates/memd-client/src/benchmark/public_benchmark.rs`
- `crates/memd-client/src/main_tests/public_benchmark_tests/mod.rs`
  - proper ConvoMem message-level evidence mapping
- `docs/verification/PUBLIC_LEADERBOARD.md`
  - replaced stale LoCoMo/MemBench and fake-zero ConvoMem rows

## Verification that passed

- `cargo build -p memd-rag -p memd-sidecar -p memd-server -p memd-client`
- `cargo test -p memd-sidecar -- --nocapture`
- `cargo test -p memd-server search_memory_uses_sidecar_rerank_when_available -- --nocapture`
- `cargo test -p memd-client build_public_benchmark_item_results_convomem_can_hit_message_evidence -- --nocapture`
- `cargo test -p memd-client write_bundle_backend_env_includes_embed_model_when_configured -- --nocapture`

## Fresh evidence

- LoCoMo fresh release rerun:
  - `.monitor/c3/fast/locomo.log`
  - `accuracy/hit_rate/recall@50 = 0.7089627391742196`
- MemBench fresh release rerun:
  - `.monitor/c3/fast/membench.log`
  - `accuracy/hit_rate/recall@50 = 0.9926666666666667`
- ConvoMem fresh release rerun:
  - `.monitor/c3/fast/convomem.log`
  - `accuracy/hit_rate/recall@50 = 0.9980952380952381`
- LongMemEval:
  - fresh full release rerun was started twice
  - did not complete in session budget
  - leaderboard keeps earlier verified `0.936`

## Checkpoint scope

Base commit before this work: `9068771`

Committed in this checkpoint:

- `Cargo.lock`
- `crates/memd-client/src/benchmark/public_benchmark.rs`
- `crates/memd-client/src/bundle/init_runtime/mod.rs`
- `crates/memd-client/src/bundle/mod.rs`
- `crates/memd-client/src/evaluation_runtime_tests/evaluation_runtime_tests_support.rs`
- `crates/memd-client/src/main_tests/bootstrap_harness_tests/mod.rs`
- `crates/memd-client/src/main_tests/public_benchmark_tests/mod.rs`
- `crates/memd-rag/src/lib.rs`
- `crates/memd-server/src/embed.rs`
- `crates/memd-server/src/main.rs`
- `crates/memd-server/src/rag_bridge.rs`
- `crates/memd-server/src/routes.rs`
- `crates/memd-server/src/store.rs`
- `crates/memd-server/src/store_migrations.rs`
- `crates/memd-server/src/tests/mod.rs`
- `crates/memd-sidecar/Cargo.toml`
- `crates/memd-sidecar/src/lib.rs`
- `crates/memd-sidecar/src/main.rs`
- `docs/verification/PUBLIC_LEADERBOARD.md`

Added docs in this checkpoint:

- `docs/superpowers/plans/2026-04-20-c3-core-rerank.md`
- `docs/superpowers/plans/2026-04-20-convomem-adapter.md`
- `docs/superpowers/specs/2026-04-20-c3-core-rerank-design.md`
- `docs/superpowers/specs/2026-04-20-convomem-adapter-design.md`

## Next session

1. Treat C3 as basis-landed, evidence-partial.
2. Start `D3 Atlas at Recall`.
3. Do not waste time rerunning LoCoMo/MemBench/ConvoMem again first.
4. After real D3 progress, rerun full board.
5. At that point, finish the missing fresh `LongMemEval` release rerun and
   close C3 evidence honestly.

## Important note

Do not answer from the old leaderboard rows. The current live facts are:

- LoCoMo ~= `0.709`
- MemBench ~= `0.993`
- ConvoMem ~= `0.998`

The old `0.415 / 0.346 / 0.000` story is stale and was already corrected in
the leaderboard file.
