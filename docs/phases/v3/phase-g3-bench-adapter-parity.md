---
phase: G3
name: Bench Adapter Parity
version: v3
status: pending
opened: 2026-04-21
depends_on: [F3]
backlog_items: []
---

# Phase G3: Bench Adapter Parity

## Goal

Every public benchmark (LongMemEval, LoCoMo, MemBench, ConvoMem) dispatches retrieval through the same configurable backend. A `--backend memd` flag routes all four benches through memd's retrieval path; `--backend lexical` preserves today's word-overlap ranking for M0-parity audits. No bench is hard-wired to lexical.

## Why this phase exists

`build_context_retrieval_run_report` (`crates/memd-client/src/benchmark/public_benchmark.rs:1373`) implements a pure token-intersection ranker and serves LoCoMo + MemBench + ConvoMem. Only LongMemEval has a `LongMemEvalRetrievalBackend` enum (`:1713`, Lexical/Sidecar/Rrf/Memd) and dispatcher. Consequence: every B3/C3/D3 retrieval improvement is invisible in three of four published numbers. Fixing metrics (H3) without fixing this first produces honest zero-lift reports.

## Deliver

1. **Generic `PublicBenchmarkBackend` enum** replacing `LongMemEvalRetrievalBackend`. Variants: `Lexical`, `Memd { base_url }`, `Sidecar { base_url }`, `Rrf`. Lives in `crates/memd-client/src/benchmark/public_benchmark.rs`, exported for all bench adapters.
2. **Per-bench dispatcher** — `rank_locomo_corpus_via_*`, `rank_membench_corpus_via_*`, `rank_convomem_corpus_via_*` — mirror the existing LongMemEval shape. Reuse `rank_longmemeval_corpus_via_memd` as the template; each sibling is a 20-50 line wrapper that passes corpus + corpus_ids to `/v1/retrieve` with the right `namespace` and `source_agent`.
3. **Refactor `build_context_retrieval_run_report`** to accept a `&PublicBenchmarkRetrievalConfig` and dispatch to the right backend, instead of hard-coding token intersection. The token-intersection scorer becomes the `Lexical` variant body.
4. **CLI flag** — `--backend {lexical,memd,sidecar,rrf}` on `benchmark public`, applies to all benches in the run. Manifest records the backend per bench row.
5. **Parity tests** — one test per bench asserting `Backend::Memd` and `Backend::Lexical` return different ordering on a pinned fixture where retrieval quality matters (i.e., a query whose lexical top-1 is wrong by design). Proves the dispatcher actually routes.
6. **Makefile update** — `bench-public` stays lexical for M0-parity; new `bench-public-memd` target runs `--backend memd` against a running memd server.

## Pass Gate

Bench-delta required:

- pre: LoCoMo/MemBench/ConvoMem backend = fixed lexical; B3/C3/D3 retrieval changes invisible. Current scores: LoCoMo 0.4149, MemBench 0.3463, ConvoMem 0.9028 (2026-04-21 run, all lexical).
- post: `cargo run -p memd-client -- benchmark public --all --backend memd --write --out .memd` produces non-identical ordering vs `--backend lexical` on ≥1 fixture query per bench. Diff recorded in phase evidence.
- regression budget: `--backend lexical` numbers reproduce within ±0.001 of today's values (refactor must not change lexical behavior). No-op guarantee.
- evidence: four parity tests green; manifest JSON shows `retrieval_backend` column; diff report attached.

Non-bench gates:

- `cargo test -p memd-client` green
- `cargo clippy -p memd-client -- -D warnings` green
- CI bench run includes one dual-backend row per bench

## Evidence

- Test output showing ordering difference per bench
- Manifest JSON with `retrieval_backend` per row
- Lexical regression audit (±0.001 tolerance)
- Commit-pinned fixture SHAs used for parity test

## Product Win

Every bench measurement is now an honest probe of memd's retrieval path. A contributor changing B3/C3/D3 sees the delta in every bench, not just LongMemEval. Leaderboard edits are caused by code changes, not by silent lexical-only measurement.

## Fail Conditions

- Refactor breaks a lexical number beyond tolerance → revert, do lexical as an explicit no-op first
- Memd-backend dispatch wedges on any bench → `/v1/retrieve` contract gap, file backlog
- Parity test passes but scores match lexical exactly → dispatcher is a pass-through; fix before exit

## Donor Anchors

- **G3-D1**: `rank_longmemeval_corpus_via_memd` (`public_benchmark.rs:2150`) — template for sibling adapters
- **G3-D2**: `LongMemEvalRetrievalBackend` enum (`public_benchmark.rs:1713`) — template for generic enum

## Rollback

Refactor is behind backend default = `Lexical`; if memd backend is flaky, users (and CI) keep the lexical path. Revert is one flag flip, not a code revert.

## Out of scope

- Canonical metric swap (H3 owns LLM-judge / token-F1 / MC accuracy)
- Leaderboard transparency (I3)
- V3 floor verification run (J3)
