---
date: 2026-04-18
phase: B3
part: 2-prereq
status: green
next_phase: B3
next_part: 2
branch: research/mining
head: 99a758b
tests: n/a (no new tests; prereq unblock only)
---
# B3 Part 2 Prereq Green — bench unblocked, next Part 2 sidecar wiring + dual-mode bench

## TL;DR

Part 2 prereq clears. LongMemEval bench runs end-to-end against a real
memd-server (intrinsic path, not in-proc). Two root causes killed:
search no longer does a full-corpus scan, and the bench client no
longer wedges inside reqwest::blocking under a caller-owned tokio
runtime. Q0 completes with session + turn metrics. Ready to wire
sidecar + dual-mode bench (Part 2 proper).

## What shipped

1. **Scoped search** — `347a27a` `fix(b3-part2-prereq): scope search to project+namespace (avoids global scan; 28ms vs 1825ms measured)`.
   - `SqliteStore::list_for_scope(project, namespace)` — pushes filters into SQL via existing `idx_memory_project` / `idx_memory_namespace`.
   - `AppState::snapshot_for_scope` — hydrated counterpart, keeps the lifecycle-apply step.
   - `search_memory` route now calls `snapshot_for_scope(req.project, req.namespace)` instead of `snapshot()`.
   - Measured at stall point: **1825ms → 28ms** per search.
   - Files: `crates/memd-server/{store.rs,main.rs,routes.rs}`.

2. **Bench HTTP I/O isolated from caller runtime** — `99a758b` `fix(b3-part2-prereq): unblock longmemeval bench by isolating memd HTTP I/O from caller runtime`.
   - `rank_longmemeval_corpus_via_memd` was using `reqwest::blocking::Client` while the bench CLI itself already runs inside tokio. `blocking::Client` starts its own worker runtime and parks the caller via Condvar; that dual-handoff wedged mid-ingest (reproduced: main R + 100% CPU, all reqwest-internal threads S 0 ticks, stuck inside `.send()` at Q0 idx=72/273).
   - Fix: spawn dedicated OS thread owning a fresh `new_current_thread` tokio runtime, drive async `reqwest::Client` there. Fully isolates bench I/O from whatever runtime the caller owns.
   - Probes (`[bench-probe] store-iter / store-json-built / store-send-returned / store-reply`) retained to aid future diagnosis.
   - Files: `crates/memd-client/src/benchmark/public_benchmark.rs`.

## Runbook — reproduce the green run

```sh
# Build
CARGO_TARGET_DIR=/tmp/memd-target cargo build --release -p memd-client -p memd-server

# Launch bench-only server (separate from main dev server on :8787)
MEMD_BIND_ADDR=127.0.0.1:18787 \
MEMD_DB_PATH=/tmp/memd-bench.db \
MEMD_RATE_LIMIT_DISABLED=1 \
MEMD_STORE_AUTO_LINK_DISABLED=1 \
/tmp/memd-target/release/memd-server &

# Run Q0 only (dry-run, top-k 20, intrinsic backend)
MEMD_BASE_URL=http://127.0.0.1:18787 \
/tmp/memd-target/release/memd benchmark public longmemeval \
  --limit 1 --mode raw --top-k 20 --dry-run \
  --retrieval-backend memd
```

Observed: 53 sessions + 273 turns ingest clean, 1 search returns, full
session + turn ndcg/recall metrics land. Latency 1349ms end-to-end.

## Gotchas future-you WILL hit

- The env var is `MEMD_BASE_URL` (see `public_benchmark.rs:1462`), **not** `MEMD_PUBLIC_BENCH_BASE_URL`. Setting the wrong one silently sends traffic to whatever dev server is on `:8787`, which has real rate-limits and a polluted corpus. Cost me ~an hour of diagnosing a "rate limit at iter 46" red herring.
- `pkill -f 'memd...'` from an interactive zsh matches the shell's own command line and kills the shell (exit 144). Use `kill -9 <pid>` or the Bash tool's `run_in_background`.
- There's still one more `reqwest::blocking` user in this file (line ~1525, sidecar path `rank_longmemeval_sidecar`). It did **not** fire the same bug in this run because we ran `--retrieval-backend memd`, not sidecar. If sidecar wiring in Part 2 hits the same stall, port it the same way.

## What's next — B3 Part 2 proper

Unblocked. Next work, in order:

1. **Sidecar wiring** — `rank_longmemeval_sidecar` path needs to drive the RAG-optional sidecar (already behind a flag in Part 1). Port its blocking client the same way if it stalls.
2. **Dual-mode bench** — single `memd benchmark public longmemeval` invocation should emit both `intrinsic` (memd-backed) and `sidecar` numbers side by side, so regressions in either path are caught at once.
3. **Full sweep** — run all 500 LongMemEval questions against the intrinsic path now that the per-question namespace pattern is fast. Current target: LME 0.86 → **≥0.92** (phase doc).
4. **Retire bench probes** — once sweep is green, drop the `[bench-probe]` eprintln! lines (they're useful but noisy). Or gate them behind `MEMD_BENCH_PROBES=1`.

## Pointer

- Phase doc: `docs/phases/v3/phase-b3-activate-retrieval.md` (check for Part 2 sub-goals / pass-gate).
- ROADMAP line 11-12: `current_phase: B3` / `phase_status: part2-prereq-landed` — flip to `part2-active` when sidecar wiring starts.
