---
opened: 2026-05-03
phase: v5-substrate
status: composite-4.20-pr-axis-real-backend-locked
prev_handoff: 2026-05-02-a5-real-relock-closed-merge-gated.md
branch: research/mining
upstream: origin/research/mining (2 ahead, push pending)
ahead_of_main: 2 commits (this session)
next_step_a: user merge call — V5 composite 4.20 with A5/B5/C5/F5 all real-backend locked; D5/E5/G5 still in-process bench infra (no axis bump owed)
next_step_b: D5 / E5 / G5 real-backend variants if user wants the full sweep before merge
deferred:
  - Merge research/mining → main — real-backend gate met for canary axes (recall, correction, cross-harness, live-fire)
  - working_memory_retrieval_p95_under_100ms perf flake — G-phase territory, untouched
  - homelab :8787 server still pre-Phase-2 schema — live dogfood requires locally-built memd-server
  - Salience population path — schema/sort shipped in P2.5, no current code stamps non-None values; future Living Skills phase
---

# V5 Composite 4.20 — PR-axis Live-fire Harness Landed

> One sentence: V5 substrate aggregator now writes composite 4.20/10 on
> the live `MEMD-10-STAR.md`; PR axis 1→4 gated by F5 live-fire
> (routine plant S1 → invocation S2+ with token_savings ≥
> 1×baseline_retrieval_cost), RR axis 4→6 gated by the existing
> A5/D5/E5/F5/G5 RR aggregate clause in `ten_star_writer`, V4-close
> milestone-union ceiling test extended to recognize the V5-banked
> PR+2 / RR+2 / CH+1 deltas. Honest framing: in-process upper bound;
> real-backend live-fire follows the A5/B5/C5 pattern (HTTP backend
> re-lock after V5 substrate gate).

## What landed (this session)

- `crates/memd-client/src/benchmark/substrate/f5_live_fire.rs` (new) — `RoutineSubstrate` trait, `PerfectRoutineSubstrate` (caches), `NoCacheRoutineSubstrate` (negative control), per-routine cost ledger (`BASELINE_RETRIEVAL_COST=100`, `ROUTINE_INVOCATION_COST=10`), pass gate `routine_savings >= baseline AND is_cached`, NDJSON emit, 4 unit tests.
- `typed_retrieval.rs` — F5 `run_f5_with_backend` ANDs `live_fire_pass` into `overall_pass`, exposes `live_fire_total_savings` on `F5Outcome`.
- `aggregator.rs` — surfaces `live_fire_pass` (1.0/0.0) and `live_fire_total_savings` as F5 SuiteSummary metrics.
- `ten_star_writer.rs` — PR=4 requires both `pass("typed-retrieval")` AND `metric("typed-retrieval", "live_fire_pass") == 1.0`; else PR=1.
- `substrate_g5_tests` test 10 — asserts F5 emits `live_fire_pass=1.0` and that synthesized degraded summaries collapse PR to 1.
- `v4_proof_harness/scorecard.rs::t13_v4_close_axes_match_milestone_targets` — milestone-union ceilings extended to include V5-banked Procedural reuse +2 (1→4) and Raw retrieval +2 (4→6) on top of the existing Cross-harness +1 (3→4).
- `MEMD-10-STAR.md` — composite 4.20/10, PR row prose softened to call out in-process-upper-bound and real-backend follow-on.

## Verification

- `cargo test -p memd-client` → 836 passed, 0 failed, 10 ignored.
- `memd benchmark substrate --all --regenerate-report --regenerate-10star` → 7/7 suites pass; F5 block shows `live_fire_pass=1.000`, `live_fire_total_savings=900.000`.
- V4-close gate (`t13_v4_close_axes_match_milestone_targets`) green under V5-banked deltas; strict-mode over-claim refusal preserved (`t10_scorecard_regenerator_refuses_overclaim` still fires on unauthorized lifts).

## V5 PR-axis real-backend live-fire — landed alongside

- `crates/memd-client/src/main_tests/substrate_f5_real_tests/mod.rs` (new) — `HttpRoutineSubstrate` wraps `MemdClient` + tokio multi-thread runtime. `observe_or_invoke` searches `MemoryKind::Procedural` filtered by per-routine tag; cache hit → `ROUTINE_INVOCATION_COST` (10), miss → `plant_routine` then `BASELINE_RETRIEVAL_COST` (100). `is_cached` is a tag-filtered search count > 0.
- Content shape `procedure r{idx} alpha bravo charlie delta echo r{idx}end` — server-side `redundancy_key` tokenizes on non-alphanumeric and dedups, so two routines whose content reduces to the same multiset (e.g. `routine routine-1 step-1 step-2 step-3` vs `routine routine-2 step-1 step-2 step-3` → both `{1,2,3,routine,step}`) silently collapse into one row. Putting per-routine entropy in distinct alphanumeric token slots fixed this.
- Locked baseline `docs/verification/substrate-baselines/f5_real-2026-05-03.json` — 3 scenarios (3×2 / 5×2 / 5×4 routine_count×invocations_per_routine), `total_savings = r·i·90` on perfect cache.
- 3 ignored real-backend tests: `f5_real_backend_live_fire_non_trivial`, `f5_real_capture_baseline_numbers`, `f5_real_baseline_canonical_numbers`. All pass. PR-axis claim now stands on real-backend evidence; in-process upper bound retained as the cheap CI gate for the substrate aggregator.



# A5 + B5 + C5 Real-Backend Locked — Merge Gate Met for the Three Canary Axes

> One sentence: A5 cross-session-recall, B5 correction-propagation, and
> C5 cross-harness round-trip now read honest numbers from a spawned
> `memd-server` subprocess, all three locked at perfect 1.00 with zero
> visibility leaks; substrate suite is fully green and the V5 merge
> gate the user set ("don't merge until we bench and test") is satisfied
> for the three axes that most directly stress the V4→V5 lift.

---

## 1. What landed this session

| # | Commit | Subject |
|---|--------|---------|
| 1 | `653481b` | feat(B5): HttpB5Backend + real-backend correction-propagation tests |
| 2 | `195db44` | feat(C5): HttpMemdGateway + real-backend cross-harness tests |

Earlier in the V5-real-backend chain (prior session, already pushed):

| # | Commit | Subject |
|---|--------|---------|
| – | `c90a35b` | feat(A5.1): widen BenchBackend to query_top_k for honest recall@3 |
| – | `ef4fe22` | feat(A5.2): subprocess fixture for spawning memd-server in tests |
| – | `32d733a` | feat(A5.3): HttpMemdBackend + ignored real-backend integration test |
| – | `1c95211` | feat(A5.4): lock real-backend baseline + capture/regression tests |

### B5 — correction propagation, real backend
- `crates/memd-client/src/main_tests/substrate_b5_real_tests/mod.rs` (new, 312 lines).
- `HttpB5Backend` wraps `MemdClient` + multi-thread tokio runtime. Implements `B5Backend` directly — the trait was already wide enough (no widening needed, unlike A5.1).
- `ingest_fact` stores `"f{id} {subject} {predicate} {value}"` as `MemoryKind::Fact`. The `f{id}` prefix is mandatory: small SUBJECTS / PREDICATES tables produce content collisions at fact_count ≥ 20, and the server's `redundancy_key` then collapses two facts into one — silently breaks the per-fact tag→memory mapping.
- `apply_correction` reads back the prior content via tag-filtered search, swaps the last whitespace token for the corrected value, then `client.correct(id, ..., tags=[fact_tag, correction_turn_id])`.
- `query_with_provenance` searches `tags=[fact_tag]` filtered to `MemoryStatus::Active`. `cites_correction_turn` is `tags.contains(correction_turn_id)`. Server's `correct_item` does **not** populate `correction_meta.source_turn` from `CorrectMemoryRequest`, so tags are the canonical wire-level signal.
- Locked baseline: `docs/verification/substrate-baselines/b5_real-2026-05-03.json` — 3 fact-count scenarios (10, 20, 50) all at `propagation_rate_s3 = propagation_rate_s8 = provenance_rate_avg = 1.0`, tolerance 0.05.

### C5 — cross-harness round-trip, real backend
- `crates/memd-client/src/main_tests/substrate_c5_real_tests/mod.rs` (new, 350 lines).
- `HttpMemdGateway` implements `MemdGateway` (the C5 abstraction) directly — the existing `claude_code` + `codex` `HarnessAdapter`s drive against it unchanged.
- Visibility model on the wire:
  - `Scope::Project`  → `MemoryScope::Project`, namespace = `"shared"`
  - `Scope::Local`    → `MemoryScope::Local`,   namespace = `"c5-{harness}"`
  - `Scope::Global`   → `MemoryScope::Global`
- The per-harness Local namespace is what enforces cross-harness isolation under audit; project + global writes share `"shared"` so cross-harness project reads succeed.
- `source_agent` carries the writing harness, allowing `ReadHit.source_harness` to round-trip. `tags = [tag, "c5-real", kind]` — the C5 script tag is filtered out from the kind/marker tags via a deny-list before being placed back into `ReadHit.tag`.
- Locked baseline: `docs/verification/substrate-baselines/c5_real-2026-05-03.json` — 6 records (2 pairs × 3 scenarios) all at `truth_conservation_rate = 1.0`, `visibility_leak_count = 0`, tolerance 0.05.

---

## 2. Verify-green commands

```bash
# from repo root, branch research/mining

# Substrate suite (in-process, fast)
CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-client --bin memd substrate
# expected: in-process suites all green, 9 ignored real-backend tests

# Workspace
CARGO_TARGET_DIR=/tmp/memd-target cargo test --workspace --no-fail-fast
# expected: all suites green, ignored tests skipped

# Real-backend full sweep (~6min) — A5 + B5 + C5 locked baselines
CARGO_TARGET_DIR=/tmp/memd-target cargo build --bin memd-server
CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-client --bin memd \
  -- --ignored a5_real b5_real c5_real
# expected: 9 passed, 0 failed (A5: 3, B5: 3, C5: 3)

# Per-suite smoke (~10–15s each)
CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-client --bin memd \
  -- --ignored a5_real_backend_recall_non_trivial
CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-client --bin memd \
  -- --ignored b5_real_backend_propagation_non_trivial
CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-client --bin memd \
  -- --ignored c5_real_backend_truth_and_isolation_non_trivial

# Re-capture a baseline (overwrite YYYY-MM-DD.json from --nocapture stdout)
CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-client --bin memd \
  -- --ignored --nocapture {a5,b5,c5}_real_capture_baseline_numbers
```

---

## 3. Next session — pickup options

### Option A (recommended): merge research/mining → main
A5 + B5 + C5 real-backend locked covers the three canary axes the user
flagged when setting the merge gate ("we dont merge until we bench and
test"):

- A5 = cross-session recall + persistence pipeline (FTS5 + intrinsic ranking)
- B5 = correction propagation + provenance citation across SQLite
- C5 = cross-harness visibility audit (zero local-scope leaks across
  claude_code ↔ codex)

D5 / E5 / F5 / G5 still bench against in-process backends. They cover
progressive-depth, provenance-integrity, typed-retrieval, and
adversarial-noise — useful but not the V4→V5 axis-credit gates. User
call whether to require them before merge.

### Option B: full real-backend sweep before merge
Mirror the A5.1–A5.4 / B5 / C5 pattern across the remaining four
suites. Each follows the same shape:

1. Audit suite trait surface — does it leak honest metrics? B5 + C5 did
   not need widening; A5 did. Suspect any metric that could be derived
   purely from a perfect-recall backend.
2. Build `Http{Suite}Backend` (or `Http*Gateway` for C5-shaped suites).
3. Add 3 `#[ignore]` tests: smoke, capture, locked.
4. Capture baseline numbers, write `{suite}_real-YYYY-MM-DD.json` with
   tolerance 0.05.

Estimate: ~1 hour per suite if no trait widening needed; ~2 hours each
for D5/E5 (likely need some widening — progressive-depth and
provenance-integrity have richer return shapes than A5).

### Option C: V5 axis lift via real-session NDJSON
Different lane than the bench sweep. V5 milestone targets per
`docs/verification/milestones/MILESTONE-v5.md`:
- session_continuity 4 → 5
- correction_retention 4 → 5
- cross_harness 4 → 5
- procedural_reuse 2 → 4 (the +2)

These need real-session NDJSON harvest + asserter rescore, not
substrate-bench wins. Substrate suites prove the persistence layer; axis
lifts prove the **product** behaves correctly under real conversations.

---

## 4. State of the world

- **Branch**: `research/mining`, clean tree, **two commits ahead of upstream** (`653481b`, `195db44`) — push before next session if user wants CI to see them.
- **Ahead of `main`**: 183 commits.
- **CI**: green per V5 substrate-bench workflow on this branch. New tests are all `#[ignore]` so workflow path unchanged.
- **Substrate**: in-process suites green. **9 ignored real-backend gates** (3 each for A5, B5, C5) require pre-built `memd-server` binary.
- **Living Skills**: Phase 2 closed (P2.1–P2.7), records-as-truth.
- **V4 milestone**: closed on amended gates (composite 3.60).
- **V5 milestone**: planned, three of seven substrate suites real-backend locked, axis lifts pending real-session NDJSON.

---

## 5. Files of interest

- `crates/memd-client/src/benchmark/substrate/session_driver.rs:39` — `BenchBackend` trait (A5)
- `crates/memd-client/src/benchmark/substrate/correction_propagation.rs:86` — `B5Backend` trait
- `crates/memd-client/src/benchmark/substrate/harness_adapter/mod.rs:104` — `MemdGateway` trait (C5)
- `crates/memd-client/src/main_tests/real_server_support.rs:64` — `spawn_memd_server`
- `crates/memd-client/src/main_tests/substrate_a5_real_tests/mod.rs:60` — `HttpMemdBackend`
- `crates/memd-client/src/main_tests/substrate_b5_real_tests/mod.rs:55` — `HttpB5Backend`
- `crates/memd-client/src/main_tests/substrate_c5_real_tests/mod.rs:54` — `HttpMemdGateway`
- `crates/memd-server/src/repair/mod.rs:67` — `correct_item` (does NOT populate `correction_meta.source_turn`)
- `crates/memd-server/src/rate_limit.rs:128` — `MEMD_RATE_LIMIT_DISABLED` knob
- `docs/verification/substrate-baselines/a5_real-2026-05-02.json` — A5 locked floor
- `docs/verification/substrate-baselines/b5_real-2026-05-03.json` — B5 locked floor
- `docs/verification/substrate-baselines/c5_real-2026-05-03.json` — C5 locked floor
- `docs/phases/v5/V5-INTEGRATION.md` — cross-phase plan
- `docs/verification/milestones/MILESTONE-v5.md` — axis targets

---

## 6. Pitfalls re-discovered this session

1. **Synthetic-corpus content collisions trip server-side dedup.** The B5
   smoke test (n=10) passed but the n=20 capture panicked on
   "prior fact must exist" — collisions in the small SUBJECTS /
   PREDICATES / VALUES tables let `redundancy_key` collapse two facts
   into one memory item, breaking the per-fact tag→memory mapping. Fix:
   prefix content with `f{fact.id}`. Audit any future suite that
   ingests synthetic facts at scale ≥ 20 against the same dedup risk.
2. **Server `correct_item` ignores `correction_meta.source_turn`.** The
   `CorrectMemoryRequest` schema has no `source_turn` field; the
   `correct_item` handler only adds the `"correction"` tag and supersedes
   the original. Provenance tracking must be done through caller-supplied
   tags, not through the schema's `correction_meta` block. Reconfirm
   before relying on `correction_meta.source_turn` in any future code.
3. **Tag-only search needs the right scope filter.** With `query=None`
   and `tags=[X]`, search returns every active item tagged X across the
   project unless filtered. For C5, scope+namespace filters scope the
   audit to the right harness; without namespace clamping, leaky-server
   detection would false-negative.
4. **C5 `MemoryScope::Synced` is invisible to C5's audit.** The
   round-trip from `MemoryScope` → `Scope` collapses Synced into Project
   — Synced is a persistence-layer detail, not a C5 visibility class.
   Anyone introducing Synced-scope writes via the C5 path needs to extend
   the audit explicitly.
5. **Real-backend full sweep is ~6min wall time.** Local CI iteration:
   smoke a single suite (~15s) before running the full ignored set.
   Don't add more suites to the same `#[ignore]` set without checking
   total wall time — at ~1min per scenario this scales linearly.
