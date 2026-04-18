# B3 Part 2 — Sidecar Wiring + Dual-Mode Bench Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land B3 Part 2 deliverables (7/8/9) — wire `memd-rag` into `memd-server` behind `rag.enabled=true`, emit dual-mode (intrinsic + accelerated) numbers from a single bench invocation, keep default off, then bank the intrinsic LongMemEval number against the phase gate (≥0.92).

**Architecture:**
- `AppState` in `crates/memd-server/src/main.rs` gains an optional `Arc<RagClient>` constructed from `MEMD_RAG_URL` / bundle `backend.rag.enabled`.
- **Identity contract (load-bearing):** when the server ingests to the sidecar, `SidecarIngestSource.id = MemoryItem.id` and `SidecarIngestSource.source_path = MemoryItem.id.to_string()`. On retrieve, `RagRetrieveItem.source` is parsed as `Uuid` and matched back to `MemoryItem.id`. Drop-if-unparseable, don't fabricate.
- `AppState::store_item` best-effort fans out to `RagClient::ingest(...)` when present (log-on-fail, never block the store), so the sidecar mirror is populated automatically. Bench path does **not** need separate sidecar ingest.
- `search_memory` in `crates/memd-server/src/routes.rs` runs the existing FTS5 path unchanged; when the RagClient is present it also fetches dense candidates and injects them into `fts_ranks` with a bonus score (mirroring the atlas-recall injection pattern) so candidates flow through the same `filter_items` pipeline.
- `memd benchmark public longmemeval` gains a `--dual` mode that hits **two** memd-server URLs — intrinsic (rag OFF) and accelerated (`MEMD_RAG_URL` set) — both user-launched per runbook; bench does NOT spawn servers.
- All bench `[bench-probe]` eprintlns gate behind `MEMD_BENCH_PROBES=1` so green runs are quiet.
- Sidecar bench path (`rank_longmemeval_corpus_via_sidecar`) gets the same dedicated-thread fix the memd path already uses.

**Tech Stack:** Rust, axum, reqwest, tokio, rusqlite (FTS5), fastembed (sidecar only), memd-rag (thin wrapper over memd-sidecar).

**Phase ref:** `docs/phases/v3/phase-b3-activate-retrieval.md` — Part 2 deliverables 7/8/9 and pass-gate.

**Prereq state (HEAD `6b8708a`):**
- scoped search (project+namespace) landed — `snapshot_for_scope`
- bench memd client already on dedicated OS thread + current_thread runtime
- `rag.enabled` default `false` in `.memd/config.json`, `memd-server` has zero memd-rag imports today
- LongMemEval intrinsic baseline: **0.860 @10 / 0.828 @5** (sidecar OFF)

---

## File Structure

**Create:**
- `crates/memd-server/src/rag_bridge.rs` — thin module: env resolution + `Option<RagClient>` constructor + `fetch_dense_candidates` helper that maps `RagRetrieveItem` back to memd `MemoryItem` IDs.

**Modify:**
- `crates/memd-server/Cargo.toml` — add `memd-rag` dep.
- `crates/memd-server/src/main.rs` — `AppState` gains `rag: Option<Arc<RagClient>>`; constructor reads `MEMD_RAG_URL`; plumb into `AppState { … }` at startup.
- `crates/memd-server/src/routes.rs` — `search_memory` calls `rag_bridge::fetch_dense_candidates` when `state.rag.is_some()`; injects into `fts_ranks` behind `MEMD_RETRIEVAL_RAG_DENSE=1` (default on when client exists). Gate avoids silent fallback regressions.
- `crates/memd-server/src/routes.rs` — `/healthz` (or dedicated `/status/rag`) reports sidecar health when configured.
- `crates/memd-client/src/benchmark/public_benchmark.rs` —
  - wrap all `[bench-probe]` eprintlns in `if bench_probes_enabled() { … }` (new fn reading `MEMD_BENCH_PROBES`).
  - port `rank_longmemeval_corpus_via_sidecar` to the dedicated-thread / current_thread pattern used by `rank_longmemeval_corpus_via_memd`.
  - add `--dual` arg on the bench CLI; when set, run intrinsic then accelerated per question and attach `mode: "intrinsic"|"accelerated"` to each result row.
- `docs/verification/PUBLIC_LEADERBOARD.md` — regenerate with dual columns once bench is green.
- `docs/phases/v3/phase-b3-activate-retrieval.md` — close-out summary section at bottom, Part 2 landed row.
- `ROADMAP.md` — `phase_status: part2-prereq-green-bench-unblocked` → `part2-active` at start, `b3-part2-landed` on close.

**Test:**
- `crates/memd-server/src/tests/mod.rs` — add `rag_bridge_no_client_is_noop()`, `search_memory_with_rag_mock_injects_candidates()` using an in-proc mock HTTP server (hyper-tungstenite or wiremock-rs — check existing test infra).
- `crates/memd-client/src/benchmark/public_benchmark.rs` tests (`#[test]` at bottom) — add `dual_mode_emits_two_rows_per_question()` against a synthetic corpus + in-proc memd-server fixture (follow the shape of any existing bench-mode test).

---

## Task 1: Port sidecar bench path off `reqwest::blocking`

**Files:**
- Modify: `crates/memd-client/src/benchmark/public_benchmark.rs:1516-1621` (`rank_longmemeval_corpus_via_sidecar`)

- [ ] **Step 1: Add sidecar async roundtrip fn (mirror `bench_memd_roundtrip`)**

Extract the ingest+retrieve HTTP work into `async fn bench_sidecar_roundtrip(...) -> anyhow::Result<RagRetrieveResponse>` using `reqwest::Client` (async), keeping request shapes identical. Keep the old blocking fn name as a thin wrapper that spawns a dedicated thread + current_thread runtime and calls into the async fn — same pattern as `rank_longmemeval_corpus_via_memd`.

- [ ] **Step 2: Build + cargo check**

```sh
CARGO_TARGET_DIR=/tmp/memd-target cargo check -p memd-client
```
Expected: clean.

- [ ] **Step 3: Commit**

```sh
git add crates/memd-client/src/benchmark/public_benchmark.rs
git commit -m "fix(b3-part2): port sidecar bench off reqwest::blocking to dedicated-thread pattern"
```

---

## Task 2: Gate bench probes behind `MEMD_BENCH_PROBES=1`

**Files:**
- Modify: `crates/memd-client/src/benchmark/public_benchmark.rs` (8 `[bench-probe]` eprintln sites)

- [ ] **Step 1: Add helper**

```rust
fn bench_probes_enabled() -> bool {
    matches!(
        std::env::var("MEMD_BENCH_PROBES").as_deref().unwrap_or(""),
        "1" | "true" | "on" | "yes"
    )
}
```

- [ ] **Step 2: Wrap every `[bench-probe]` eprintln call**

Use `if bench_probes_enabled() { eprintln!(...) }` or a macro if cleaner. Do not drop the messages.

- [ ] **Step 3: Build + smoke-run bench with probes off**

```sh
CARGO_TARGET_DIR=/tmp/memd-target cargo build --release -p memd-client -p memd-server
MEMD_BASE_URL=http://127.0.0.1:18787 \
/tmp/memd-target/release/memd benchmark public longmemeval \
  --limit 1 --mode raw --top-k 20 --dry-run --retrieval-backend memd 2>&1 | grep -c '\[bench-probe\]'
```
Expected: `0`.

- [ ] **Step 4: Re-run with probes on**

```sh
MEMD_BENCH_PROBES=1 MEMD_BASE_URL=http://127.0.0.1:18787 \
/tmp/memd-target/release/memd benchmark public longmemeval \
  --limit 1 --mode raw --top-k 20 --dry-run --retrieval-backend memd 2>&1 | grep -c '\[bench-probe\]'
```
Expected: `> 0`.

- [ ] **Step 5: Commit**

```sh
git commit -am "chore(b3-part2): gate bench probes behind MEMD_BENCH_PROBES=1"
```

---

## Task 3: Server — carry optional RagClient in AppState

**Files:**
- Create: `crates/memd-server/src/rag_bridge.rs`
- Modify: `crates/memd-server/Cargo.toml`
- Modify: `crates/memd-server/src/main.rs` (AppState struct + state construction)

- [ ] **Step 1: Add dep**

Add to `crates/memd-server/Cargo.toml` under `[dependencies]`:
```toml
memd-rag = { path = "../memd-rag" }
```

- [ ] **Step 2: Write failing integration test**

In `crates/memd-server/src/tests/mod.rs`:
```rust
#[tokio::test]
async fn rag_bridge_no_env_is_none() {
    // MEMD_RAG_URL absent → state.rag == None.
    let prior = std::env::var("MEMD_RAG_URL").ok();
    unsafe { std::env::remove_var("MEMD_RAG_URL"); }
    let state = test_state_with_rag();
    assert!(state.rag.is_none());
    if let Some(v) = prior { unsafe { std::env::set_var("MEMD_RAG_URL", v); } }
}
```

Run: `cargo test -p memd-server rag_bridge_no_env_is_none` → FAIL (symbol missing).

- [ ] **Step 3: Implement `rag_bridge.rs` constructor**

```rust
use std::sync::Arc;

pub fn build_rag_client() -> Option<Arc<memd_rag::RagClient>> {
    let url = std::env::var("MEMD_RAG_URL").ok().filter(|s| !s.is_empty())?;
    memd_rag::RagClient::new(&url).ok().map(Arc::new)
}
```

Add `pub(crate) mod rag_bridge;` to `main.rs`; add field `rag: Option<Arc<memd_rag::RagClient>>` to `AppState`; set `rag: rag_bridge::build_rag_client()` in the `AppState { … }` literal at line ~627.

Add `test_state_with_rag` in tests helpers mirroring existing test state builders.

- [ ] **Step 4: Run test**

`cargo test -p memd-server rag_bridge_no_env_is_none` → PASS.

- [ ] **Step 5: Commit**

```sh
git commit -am "feat(b3-part2): AppState carries Option<Arc<RagClient>> from MEMD_RAG_URL"
```

---

## Task 4: Sidecar ingest fan-out + identity contract

**Files:**
- Modify: `crates/memd-server/src/rag_bridge.rs` (+ `ingest_item` async helper)
- Modify: `crates/memd-server/src/main.rs` — `AppState::store_item` best-effort fan-out when `state.rag.is_some()`.
- Test: `crates/memd-server/src/tests/mod.rs`

**Contract (load-bearing; implement EXACTLY):**
- On ingest: `SidecarIngestSource.id = MemoryItem.id` (Uuid), `SidecarIngestSource.source_path = MemoryItem.id.to_string()`, `content = MemoryItem.content`, `tags = MemoryItem.tags`, `kind` maps from `MemoryItem.kind`.
- On retrieve: parse `RagRetrieveItem.source` as `Uuid`; drop any item whose source fails to parse (do NOT fabricate an ID or fall back to content-hash matching).
- Fan-out is **best-effort**: log-on-fail with `tracing::warn!`, never propagate the error up through `store_item`. Store path must not regress under a flaky sidecar.

- [ ] **Step 1: Failing test — store with RAG URL set calls mock ingest endpoint**

Stand up a mock (wiremock-rs or hand-rolled tiny axum) that records POSTs to `/ingest`. Call `AppState::store_item` on a harness state built with the mock URL. Assert the mock saw one call whose `id` field equals the returned `MemoryItem.id` and `source_path` equals that id as a string.

Run: FAIL — fan-out not yet wired.

- [ ] **Step 2: Implement `rag_bridge::ingest_item`**

```rust
pub(crate) async fn ingest_item(
    client: &memd_rag::RagClient,
    item: &memd_schema::MemoryItem,
) -> anyhow::Result<()> {
    let src = memd_sidecar::SidecarIngestSource {
        id: item.id,
        source_path: item.id.to_string(),
        content: item.content.clone(),
        tags: item.tags.clone(),
        kind: map_kind(item.kind),
        ..Default::default()
    };
    client.ingest(vec![src]).await.map(|_| ())
}
```

In `AppState::store_item` (after the existing sqlite insert), if `self.rag.is_some()` spawn a detached task:
```rust
if let Some(rag) = self.rag.clone() {
    let snap = item.clone();
    tokio::spawn(async move {
        if let Err(e) = rag_bridge::ingest_item(&rag, &snap).await {
            tracing::warn!(target: "memd::rag_bridge", ?e, "sidecar ingest failed (non-fatal)");
        }
    });
}
```

Detached spawn keeps store latency flat; failure never blocks the caller.

- [ ] **Step 3: Run test → PASS**

- [ ] **Step 4: Commit**

```sh
git commit -am "feat(b3-part2): best-effort sidecar ingest fan-out from AppState::store_item"
```

---

## Task 5: Server — inject dense candidates into search_memory ranking

**Files:**
- Modify: `crates/memd-server/src/rag_bridge.rs` (+ `fetch_dense_candidates`)
- Modify: `crates/memd-server/src/routes.rs` (`search_memory`, around the atlas-recall block ~292-312)
- Test: `crates/memd-server/src/tests/mod.rs`

- [ ] **Step 1: Failing test — mock RAG injects a candidate that filter_items surfaces**

Stand up a `wiremock` (or hand-rolled tiny axum) mock server that returns a single known `RagRetrieveResponse` item pointing to a memory already stored via the test harness. Assert the item appears in the `SearchMemoryResponse` ranked ahead of unrelated items.

Run: FAIL — bridge not yet calling out.

- [ ] **Step 2: Implement `fetch_dense_candidates`**

```rust
pub(crate) async fn fetch_dense_candidates(
    client: &memd_rag::RagClient,
    query: &str,
    project: Option<&str>,
    namespace: Option<&str>,
    limit: usize,
) -> Vec<(uuid::Uuid, f32)> {
    // Map RagRetrieveItem.source (MemoryItem id as string) → uuid. Drop items that
    // fail to parse. Return at most `limit` pairs in response order.
}
```

In `routes.rs::search_memory`, after the atlas-recall block, gate on `state.rag.is_some() && dense_enabled()` (where `dense_enabled()` reads `MEMD_RETRIEVAL_RAG_DENSE` — default on when the client exists, off otherwise). Call `fetch_dense_candidates(...).await` and append any new IDs into `fts_ranks` with `tail_score` (same bonus shape as atlas-recall neighbors).

- [ ] **Step 3: Run test**

`cargo test -p memd-server search_memory_with_rag_mock_injects_candidates` → PASS.

- [ ] **Step 4: Full server test suite**

```sh
CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-server
```
Expected: all green (no regressions).

- [ ] **Step 5: Commit**

```sh
git commit -am "feat(b3-part2): inject dense RAG candidates into search_memory ranking"
```

---

## Task 6: Dual-mode bench — single invocation emits intrinsic + accelerated rows

**Files:**
- Modify: `crates/memd-client/src/benchmark/public_benchmark.rs` (CLI flag, per-question dispatch, result row shape)

- [ ] **Step 1: Add `--dual` CLI flag**

Add `dual: bool` to the CLI args struct. When set, overrides `--retrieval-backend` and forces dual runs.

- [ ] **Step 2: Design the dispatch**

In `rank_longmemeval_corpus`, when dual mode is active:
1. Run with `LongMemEvalRetrievalBackend::Memd` against `MEMD_BASE_URL` (intrinsic).
2. Run with `LongMemEvalRetrievalBackend::Memd` against `MEMD_BASE_URL_ACCELERATED` (sidecar-enabled server). Fall back to `MEMD_BASE_URL` if unset and assume `MEMD_RAG_URL` flips dense on in the same server.
3. Emit both result rows with `mode: "intrinsic" | "accelerated"`.

- [ ] **Step 3: Failing test**

`dual_mode_emits_two_rows_per_question` — synthetic corpus, in-proc memd-server with and without RAG URL; assert the result JSON shows 2 rows per question with distinct `mode` labels.

Run → FAIL.

- [ ] **Step 4: Implement + pass**

- [ ] **Step 5: Smoke**

Two servers needed for full accelerated. For Step 5 we only smoke-verify the *shape*: `--dual` with sidecar off on both should still produce 2 rows (accelerated == intrinsic numerically).

```sh
/tmp/memd-target/release/memd benchmark public longmemeval \
  --limit 2 --mode raw --top-k 20 --dry-run --dual
```
Expected: 4 rows (2 questions × 2 modes).

- [ ] **Step 6: Commit**

```sh
git commit -am "feat(b3-part2): --dual bench emits intrinsic + accelerated rows per question"
```

---

## Task 7: Sidecar health surfaces cleanly on `/healthz`

**Files:**
- Modify: `crates/memd-server/src/routes.rs` — extend healthz to include `{ "rag": { "enabled": bool, "reachable": bool, "name": Option<String> } }`.

- [ ] **Step 1: Failing test**

Assert healthz JSON includes `"rag"` key; with `MEMD_RAG_URL` unset → `enabled=false`. With a mock rag endpoint → `enabled=true, reachable=true`.

- [ ] **Step 2: Implement**

Call `RagClient::healthz()` with a short timeout when state.rag.is_some(). Never block response on it — 500ms cap, reachable=false on timeout.

- [ ] **Step 3: Run test → PASS**

- [ ] **Step 4: Commit**

```sh
git commit -am "feat(b3-part2): /healthz surfaces rag sidecar enabled+reachable state"
```

---

## Task 8: Full intrinsic sweep — bank the ≥0.92 LME number

**Files:** (no code — eval-only)
- Output: `.memd/benchmarks/history/benchmark-runs.jsonl` (gitignored; capture headline to plan)
- Output: `docs/verification/PUBLIC_LEADERBOARD.md` regenerated

- [ ] **Step 1: Build release**

```sh
CARGO_TARGET_DIR=/tmp/memd-target cargo build --release -p memd-client -p memd-server
```

- [ ] **Step 2: Launch bench server (intrinsic mode, rag OFF)**

```sh
MEMD_BIND_ADDR=127.0.0.1:18787 \
MEMD_DB_PATH=/tmp/memd-bench-intrinsic.db \
MEMD_RATE_LIMIT_DISABLED=1 \
MEMD_STORE_AUTO_LINK_DISABLED=1 \
MEMD_RETRIEVAL_FTS5_TUNED=1 \
MEMD_RETRIEVAL_ATLAS_EXPANSION=1 \
MEMD_RETRIEVAL_ATLAS_RECALL=1 \
/tmp/memd-target/release/memd-server &
```

All the intrinsic flags from Part 1 flipped ON — this is the "intrinsic wins first" claim.

- [ ] **Step 3: Full 500-Q sweep**

```sh
MEMD_BASE_URL=http://127.0.0.1:18787 \
/tmp/memd-target/release/memd benchmark public longmemeval \
  --mode raw --top-k 20 --record --retrieval-backend memd
```

Expected: headline `session_recall_any@5 ≥ 0.92` (gate). Capture the number. If under: investigate one more round before flipping phase status. Do **not** advance with a failed gate.

- [ ] **Step 4: Regenerate leaderboard with intrinsic column**

```sh
/tmp/memd-target/release/memd benchmark public leaderboard --write docs/verification/PUBLIC_LEADERBOARD.md
```

- [ ] **Step 5: Commit**

```sh
git add docs/verification/PUBLIC_LEADERBOARD.md
git commit -m "bench(b3-part2): intrinsic 500-Q sweep — LME <0.XX> with Part 1 flags on"
```

---

## Task 9: Phase close-out + roadmap flip + handoff packet

**Files:**
- Modify: `docs/phases/v3/phase-b3-activate-retrieval.md` — append "Part 2 — tasks-landed summary" section mirroring Part 1's.
- Modify: `ROADMAP.md` — flip `phase_status` to `b3-part2-landed` (or `b3-green` if all intrinsic gates clear).
- Create: `docs/handoff/2026-04-18-b3-part2-landed.md` via `scripts/handoff-latest.sh` refresh.
- Run: `memd checkpoint --roadmap-set phase_status=b3-part2-landed --auto-commit`

- [ ] **Step 1: Append Part 2 summary to phase doc**

Table rows for deliverables 7/8/9, commits linked. Note any gotchas (sidecar still blocking-path-caveat gone, dense candidate gate, dual-mode shape).

- [ ] **Step 2: Flip roadmap**

```sh
memd checkpoint --roadmap-set phase_status=b3-part2-landed --auto-commit \
  --content "B3 Part 2 landed: sidecar wiring + dual-bench + intrinsic LME <value>"
```

- [ ] **Step 3: Write handoff packet**

Follow `docs/handoff/2026-04-18-b3-part2-prereq-green-next-part2.md` as template. TL;DR, what shipped, runbook, gotchas, what's next (C3 reranker).

- [ ] **Step 4: Refresh LATEST symlink**

```sh
./scripts/handoff-latest.sh
```

- [ ] **Step 5: Commit**

```sh
git add docs/phases/v3/phase-b3-activate-retrieval.md docs/handoff/
git commit -m "docs(b3-part2): close-out — part2 landed, next C3 reranker"
```

---

## Execution Notes

- **Rigid TDD where fast tests exist, integration-style where they don't.** Tasks 3/4/5/7 warrant mock-server integration tests; bench tasks (1/2/6/8) lean on smoke runs since a strict unit test for a 500-question sweep is not the shape of the problem.
- **Commit after every task, not every step.** Step-level commits exist where explicit; otherwise one commit per task at the end.
- **Flag philosophy.** Every new behavior lands behind an env flag (`MEMD_RETRIEVAL_RAG_DENSE`, `MEMD_BENCH_PROBES`) so rollback is env-toggle, not revert. Defaults: dense on-when-client-exists, probes off.
- **Test env var hygiene.** Cargo runs unit tests in parallel threads within a process; `std::env::set_var` / `remove_var` mutate shared state and will race. For any test that toggles `MEMD_RAG_URL` or `MEMD_RETRIEVAL_RAG_DENSE`, use the `serial_test` crate (`#[serial]`) or construct state via a dedicated builder that accepts the URL directly — DO NOT rely on process env inside parallel tests.
- **If Task 8 fails the ≥0.92 gate:** stop. Diagnose which Part 1 flag (or combo) is under-performing, or whether dense candidates are pulling *down* the intrinsic number (they shouldn't — intrinsic run has rag OFF). 0.828 → 0.92 is aspirational; may require one budget-tuning pass on BM25 weights / recall-k / atlas neighbors. Do not ship a red gate.
- **Dual-mode operator UX.** The bench does NOT spawn servers. User launches two memd-server instances (intrinsic on `:18787` with rag OFF, accelerated on `:18788` with `MEMD_RAG_URL` set). Runbook in Task 6 close-out must spell this out.

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-04-18-b3-part2-sidecar-dual-bench.md`. Two execution options:

**1. Subagent-Driven** — dispatch a fresh subagent per task, review between tasks.
**2. Inline Execution (recommended here)** — we're already in-context with orientation done; batch inline with a checkpoint after Task 5 (mid-way, server wiring done) and after Task 8 (bench banked).

Which approach?
