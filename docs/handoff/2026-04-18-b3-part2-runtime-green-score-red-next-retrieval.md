---
date: 2026-04-19
phase: B3
part: 2
status: yellow
next_phase: B3
next_part: retrieval-quality
branch: research/mining
head: c5b451d
tests: cargo build --release -p memd-server (clean, 4 pre-existing warnings)
---
# B3 Part 2 — chunking + batch ort wired, next is 500Q rerun

## TL;DR

Inlined fastembed @ 9699ddc landed LongMemEval `session_recall_any@5 = 0.882`
(baseline 0.828, gate 0.92). Missed gate by 3.8pts but `@10 = 0.934` — the
answer is in top-10 94% of the time, rank is wrong. Weakest per-type:
`single-session-preference @10 = 0.667` — single per-session vector blurs
preferences buried deep in long sessions.

@ c5b451d: per-turn chunking added (char-window w/ overlap, MiniLM 256-tok cap
in mind) + batch ort session call so ingest doesn't pay N mutex-lock round
trips per doc. Untested on the full 500Q. Build is clean.

## What shipped this session

- `crates/memd-server/src/embed.rs`
  - `chunk_text(text, max_chars, overlap)` — char-window chunker (default 1500
    chars, 200 overlap; env overrides `MEMD_DENSE_CHUNK_CHARS`,
    `MEMD_DENSE_CHUNK_OVERLAP`).
  - `embed_batch_normalized(&[String]) -> Vec<Vec<f32>>` — single
    `TextEmbedding::embed` call over the whole chunk batch, L2-normalizes
    each, filters empty. One mutex lock per doc, not per chunk.

- `crates/memd-server/src/store.rs` + `store_migrations.rs`
  - `memory_vectors` composite PK `(memory_id, chunk_idx)` + scope index.
  - `replace_memory_vector_chunks(memory_id, project, ns, dim, chunks)` —
    delete-all + insert-all in a single tx.
  - `list_vectors_for_scope(project, ns)` — scoped brute-force fetch (not
    global scan).
  - `migrate_memory_vectors_chunk_idx` — if legacy table lacks `chunk_idx`,
    drops and recreates (vectors are derived cache, safe to rebuild).

- `crates/memd-server/src/main.rs`
  - `maybe_upsert_vector` chunks item content, batch-embeds in one ort call,
    zips with indices, writes rows in one tx. Called from the 3 ingest paths
    (store, revive, reinforce).

- `crates/memd-server/src/routes.rs`
  - Dense search groups chunk cosines by `memory_id` taking MAX.
  - Blend: `score = dense + 0.1 * fts` (dense dominates, FTS tiebreaker).
  - Truncates to 200 candidates before ranking.

## Where the gate stands

| metric                           | pre-chunk (9699ddc) | target |
| -------------------------------- | ------------------- | ------ |
| `session_recall_any@5`           | 0.882               | ≥0.92  |
| `session_recall_any@10`          | 0.934               | —      |
| `per_type::single-session-pref@10` | 0.667             | —      |
| mean_latency_ms                  | 9671                | —      |

Per-chunk serial embed tripled smoke latency (2.1s → 5.7s per Q). Batch-embed
is the fix that should hold the run under ~30 min. Not yet measured.

## Next steps

1. Smoke 20Q against a clean db with the chunking server. Expect latency
   ~2s/Q (back to baseline) and recall@5 up from 0.882.
   ```
   pkill -f 'target/release/memd-server'
   rm -rf /tmp/memd-bench && mkdir -p /tmp/memd-bench
   MEMD_RATE_LIMIT_DISABLED=1 MEMD_STORE_AUTO_LINK_DISABLED=1 \
     MEMD_INTRINSIC_DENSE=1 MEMD_DB_PATH=/tmp/memd-bench/memd.db \
     nohup /tmp/memd-target/release/memd-server > /tmp/memd-server.log 2>&1 &
   ```
   Then invoke `memd benchmark public longmemeval --retrieval-backend memd
   --memd-url http://127.0.0.1:8787 --sample 20 --write`.

2. If smoke looks right, run the full 500Q. If recall@5 ≥ 0.92 — B3 gate
   cleared, close part 2. If not, inspect per-type breakdown — the chunk
   granularity knobs (`MEMD_DENSE_CHUNK_CHARS`, `_OVERLAP`) are the first
   dial; blend weight `0.1 * fts` in `routes.rs:search_memory` is second.

3. If still short: reranker experiment (MiniLM-L6 cross-encoder for the
   top-20 candidates). Not yet scaffolded.

## Landmines / gotchas

- There is a stale `memd-server` binary in `target/release/` (from before
  `CARGO_TARGET_DIR=/tmp/memd-target` convention). `pkill -f memd-server`
  matches this session's shell process text and nukes the shell. Kill by
  PID from `ps aux | grep -E 'memd-server($| )'`.
- Default DB path is `.memd/memd.db` (`MEMD_DB_PATH` env override).
  Pre-existing repo db has 97 items — always bench against a clean `/tmp`
  path.
- Rate limit is on by default → set `MEMD_RATE_LIMIT_DISABLED=1` for bench.
- `MEMD_STORE_AUTO_LINK_DISABLED=1` too — auto-link is O(N) scan per write
  and kills bulk-ingest throughput.
