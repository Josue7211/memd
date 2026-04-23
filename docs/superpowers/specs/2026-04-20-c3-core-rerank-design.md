# C3 Core Rerank Design

**Date:** 2026-04-20

**Goal**

Finish the score-moving slice of C3 by adding an intrinsic local reranker on top of the current B3 search ranking, then rerun `ConvoMem` and `MemBench` so bench state is fresh.

**Scope**

- In:
  - local rerank in the current product search path
  - default-on intrinsic behavior with a debug escape hatch
  - rerank traces/test coverage at the ranking seam
  - fresh `ConvoMem` and `MemBench` reruns after implementation
- Out:
  - sidecar `/rerank`
  - `MEMD_EMBED_MODEL`
  - corpus re-embed migration
  - `embedding_model` stamps
  - final C3 evidence bundle

**Design**

The current product search path already builds a ranked candidate list from FTS, optional sidecar dense tail injection, and intrinsic dense blending in [crates/memd-server/src/routes.rs](/home/josue/Documents/projects/memd/crates/memd-server/src/routes.rs:265). The smallest real C3 slice is to rerank only the top candidate window from that path instead of replacing retrieval.

The reranker should be local and cheap. Reuse the same class of signals already proven in `memd-sidecar`: token overlap, keyword overlap, query bigrams, trigram-style semantic features, tags, source path, and source agent. Blend that local rerank score with the existing base rank score so dense retrieval remains the source of candidate recall and rerank only changes ordering.

**Approach**

1. Add a small local rerank module inside `memd-server` for query/item scoring.
2. Apply rerank to the top-N ranked ids after intrinsic dense blending and before `filter_items`.
3. Keep rerank enabled by default, with env-based debug disable for regression isolation.
4. Add tests that prove rerank promotes the stronger candidate when base ranking is tied or misleading.
5. Rerun `ConvoMem` and `MemBench` after the code lands.

**Success**

- C3 is no longer docs-only.
- Search path performs local intrinsic rerank before final top-k selection.
- Existing search behavior stays stable outside the reranked candidate window.
- Fresh `ConvoMem` and `MemBench` numbers exist from post-change runs.

**Known Limits**

- This does not complete full C3.
- This does not guarantee `ConvoMem` improvement because its current adapter/metric contract still looks suspect.
- This does not solve `MemBench` multi-hop/entity recall; that remains partly D3 territory.
