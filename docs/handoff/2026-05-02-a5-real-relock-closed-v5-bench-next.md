---
opened: 2026-05-02
phase: v5-substrate
status: a5-real-relock-closed-merge-gated-on-bench
prev_handoff: 2026-05-02-living-skills-phase2-closed-merge-gated.md
branch: research/mining
upstream: origin/research/mining (synced)
ahead_of_main: 181 commits
next_step_a: bench + test run on real backend (V5 substrate-bench full sweep) before merge to main
next_step_b: pick next V5 lift — B5 correction-propagation re-lock, C5 cross-harness re-lock, or D5/E5/F5/G5 follow-ons (mirror the A5 trait-widening pattern)
deferred:
  - Merge research/mining → main — explicitly gated by user on real-backend bench + test pass
  - working_memory_retrieval_p95_under_100ms perf flake — G-phase territory, untouched
  - homelab :8787 server still pre-Phase-2 schema — live dogfood requires locally-built memd-server
  - Salience population path — schema/sort shipped in P2.5, no current code stamps non-None values; future Living Skills phase
---

# A5 Real-Backend Re-Lock Closed — V5 Bench Next, Merge Still Gated

> One sentence: A5 cross-session-recall now reads honest recall@3 from a
> spawned `memd-server` subprocess, locked at `recall@3 = 1.00` across 9
> scenarios; substrate suite is fully green (119 tests + 4 ignored real-backend);
> next step is a real-backend bench sweep before any merge to `main`.

---

## 1. What landed this session

| # | Commit | Subject |
|---|--------|---------|
| 1 | `c90a35b` | feat(A5.1): widen BenchBackend to query_top_k for honest recall@3 |
| 2 | `ef4fe22` | feat(A5.2): subprocess fixture for spawning memd-server in tests |
| 3 | `32d733a` | feat(A5.3): HttpMemdBackend + ignored real-backend integration test |
| 4 | `1c95211` | feat(A5.4): lock real-backend baseline + capture/regression tests |

All 4 pushed to `origin/research/mining`.

### A5.1 — trait widening
- `BenchBackend::query_for_fact(...) -> Option<String>` → `query_top_k(session, fact, k) -> Vec<String>`.
- `ScenarioOutcome` gained `recall_at_3: f64`. Previously the runner cheated with `r3 = r1` because the trait could not express top-k.
- `RecordingBackend` returns `vec![value]` on match, empty vec otherwise. `DegradedBackend::query_top_k` returns `Vec::new()`.

### A5.2 — subprocess fixture
- `crates/memd-client/src/main_tests/real_server_support.rs` (new, 132 lines).
- `spawn_memd_server()` pre-binds `127.0.0.1:0`, drops the listener, passes the captured port via `MEMD_BIND_ADDR`, points DB at a tempdir via `MEMD_DB_PATH`, polls `/healthz` (not `/health`) up to 10s.
- `MEMD_SERVER_BIN` env override; fallback walks workspace `target/{debug,release}/memd-server`.

### A5.3 — HttpMemdBackend
- `crates/memd-client/src/main_tests/substrate_a5_real_tests/mod.rs` (new).
- Wraps `MemdClient` + multi-thread tokio runtime. `ingest_fact` stores `"{subject} {predicate} {value}"` as `Fact` kind; `query_top_k` searches `"{subject} {predicate}"` and parses the last whitespace token of each result content as the value.
- `seal_session` / `restore_session` are no-ops — real `memd-server` persists via SQLite, so cross-session recall happens by construction. A4 PostCompact ledger restore lives in the V4 proof harness.

### A5.4 — locked baseline
- `docs/verification/substrate-baselines/a5_real-2026-05-02.json` — 9 scenarios, `tolerance: 0.05`.
- recall@3 = 1.00 across the matrix. recall@1 logged for visibility (1.00 @ N=20, 0.96 @ N=50, 0.87 @ N=100). cut_k has zero effect because seal/restore are no-ops.
- `MEMD_RATE_LIMIT_DISABLED=1` added to spawn fixture — capture run does ~9×(40–300 writes) which trips the 100/min soft gate.
- Glob is `a5_real-*.json` (underscore), distinct from `a5-*.json` (hyphen) used by the in-process locked-baseline test → no collision.

---

## 2. Verify-green commands

```bash
# from repo root, branch research/mining

# Substrate suite (in-process, fast)
CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-client --bin memd substrate
# expected: 119 passed, 3 ignored (A5 real-backend), 0 failed

# Workspace
CARGO_TARGET_DIR=/tmp/memd-target cargo test --workspace --no-fail-fast
# expected: 832 + 240 + side suites all ok, 4 ignored, 0 failed

# A5 real-backend smoke (~10s)
CARGO_TARGET_DIR=/tmp/memd-target cargo build --bin memd-server
CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-client --bin memd \
  -- --ignored a5_real_backend_recall_non_trivial

# A5 baseline regression (~5min — full 9-scenario replay)
CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-client --bin memd \
  -- --ignored a5_real_baseline_canonical_numbers

# Re-capture baseline (~5min — overwrite a5_real-YYYY-MM-DD.json)
CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-client --bin memd \
  -- --ignored --nocapture a5_real_capture_baseline_numbers
```

---

## 3. Next session — pickup options

### Option A: V5 substrate-bench real-backend sweep (gating merge)
User said "we dont merge until we bench and test." The current locked
baseline is **A5 only** against a real backend. Other suites still bench
against in-process/perfect-recall backends:

- B5 correction-propagation: `RecordingBackend` based — apply A5.1-style
  trait widening if needed, then add HttpMemdBackend variant.
- C5 cross-harness: synthetic; real-backend variant unblocks visibility-leak
  asserts under live SQLite.
- D5 progressive-depth, E5 provenance-integrity, F5 typed-retrieval,
  G5 adversarial-noise: same pattern.

Cheapest first sweep: just spawn one server, run all suites with a
real-backend variant, capture their numbers as locked baselines under
`docs/verification/substrate-baselines/{b5..g5}_real-2026-05-02.json`.
Then merge.

### Option B: Treat A5 alone as sufficient bench pass and merge
A5 is the pipeline canary — recall + persistence end-to-end. If user
intent was "make sure A5 isn't lying," this is closed. Merge gate is
user judgment call; defer to user.

### Option C: V5 axis lift
V5 milestone targets per `docs/verification/milestones/MILESTONE-v5.md`:
- session_continuity 4 → 5
- correction_retention 4 → 5
- cross_harness 4 → 5
- procedural_reuse 2 → 4 (the +2)

These need **real-session NDJSON harvest** + asserter rescore, not just
substrate-bench wins. Different lane than the bench sweep above.

### Recommended pickup
**Option A**, smallest scope: B5 + C5 real-backend variants (mirror A5.1–A5.4
pattern) → bake into `a5_real_baseline_canonical_numbers`-style regression
tests → re-evaluate merge gate.

Why B5/C5 first: they're the suites that most directly stress
"correction propagates across sessions" and "no leak across harnesses" —
both axis-credit gates for V4 → V5 lift. D5/E5/F5/G5 can lag.

---

## 4. State of the world

- **Branch**: `research/mining`, clean tree, synced with origin.
- **Ahead of `main`**: 181 commits.
- **CI**: green per V5 substrate-bench workflow on this branch (last
  recorded green commit before this session was 07b68ac; A5.1–A5.4 only
  added `#[ignore]` tests so workflow path unchanged).
- **Substrate**: 119 in-process tests green. 3 ignored (`a5_real_*`) +
  1 ignored (`real_server_spawns_and_responds_to_healthz`) = 4 total
  real-backend gates that require pre-built `memd-server` binary.
- **Living Skills**: Phase 2 closed (P2.1–P2.7), records-as-truth.
- **V4 milestone**: closed on amended gates (composite 3.60).
- **V5 milestone**: planned, suites built but axis lifts pending real-session NDJSON.

---

## 5. Files of interest

- `crates/memd-client/src/benchmark/substrate/session_driver.rs:39` — `BenchBackend` trait
- `crates/memd-client/src/benchmark/substrate/cross_session_recall.rs:77` — `run_a5_with_backend` entry point
- `crates/memd-client/src/main_tests/real_server_support.rs:64` — `spawn_memd_server`
- `crates/memd-client/src/main_tests/substrate_a5_real_tests/mod.rs:60` — `HttpMemdBackend` impl
- `crates/memd-server/src/rate_limit.rs:128` — `MEMD_RATE_LIMIT_DISABLED` knob
- `docs/verification/substrate-baselines/a5_real-2026-05-02.json` — locked floor
- `docs/phases/v5/V5-INTEGRATION.md` — cross-phase plan
- `docs/verification/milestones/MILESTONE-v5.md` — axis targets

---

## 6. Pitfalls re-discovered this session

1. **Trait too narrow swallows recall@3** — A5.1 caught the synthetic
   `r3 = r1` cheat that hid behind `Option<String>`. When a benchmark
   metric is statically expressible from the trait surface, suspect a
   cheat. Mirror this audit on B5–G5.
2. **Rate-limit gates multi-scenario captures** — soft gate is 100
   writes/min; full A5 capture is ~720 writes across ~6min. `MEMD_RATE_LIMIT_DISABLED=1`
   in the spawn fixture is the right knob; never bypass it in prod paths.
3. **Filename glob collisions** — existing `a5-*.json` test reads
   `a5_real-*.json` by accident if filename starts `a5-`. Underscore
   prefix sidesteps.
4. **`rsplit_whitespace` doesn't exist** — use
   `.split_whitespace().next_back()` which is iterator-trait based.
5. **`memd-server` is binary-only** — no lib.rs, no exported `Router`.
   Subprocess spawn is the only integration path. Documented in
   `real_server_support.rs:1` header.
