---
opened: 2026-05-04
phase: v6-f6-scorecard
status: f6-scorecard-regenerator-landed-merge-ready
prev_handoff: 2026-05-03-a5-b5-c5-real-locked-merge-eligible.md
branch: research/mining
upstream: origin/research/mining (synced at b90a4e7 before this handoff refresh)
merge_status: origin/main is ancestor of research/mining; fast-forward merge ready
next_step_a: merge research/mining -> main and push
next_step_b: continue V6 canonical gates / close from main
deferred:
  - D5 / E5 / G5 real-backend variants remain optional bench-depth work; no V5 axis bump owed
  - homelab :8787 server may still need local memd-server build for full dogfood schema parity
  - working_memory_retrieval_p95_under_100ms perf flake remains G-phase territory
---

# F6 Scorecard Regenerator Landed - Merge Ready

One sentence: desktop truth is now on `research/mining` at `b90a4e7`,
the F6 V6 scorecard writer/regenerator path is landed, and `main` can
fast-forward to this branch before V6 close work continues.

## Current Truth

- Source branch: `research/mining`
- Last synced commit before this handoff refresh: `b90a4e7 test(F6): unit tests for V6 scorecard helpers (contract-key pin + splice)`
- Merge shape: `origin/main` is ancestor of `research/mining` by 7 commits, so the merge is a fast-forward.
- V5 merge gate: met. A5/B5/C5/F5 are real-backend locked; D5/E5/G5 remain in-process bench infra and do not block merge.
- V6 path: F6 scorecard/regenerator work is active. Composite path is 4.20 -> 4.45 per current V6 milestone/status; older `phase-f6-plan.md` references to composite >=7.0 are stale against the milestone/status truth.
- memd runtime: `memd lookup` recalls this tailnet/backend setup: this Mac joined the tailnet, shared backend `http://100.104.154.24:8787` was reachable, and bundle voice authority is `caveman-ultra`.

## Landed After Previous Handoff

| Commit | Subject | Notes |
| --- | --- | --- |
| `12880b7` | `feat(F6): V6 10-STAR writer + regen wiring (Branch 1 - composite 4.20->4.45 path)` | Adds V6 scorecard writer and regeneration wiring. |
| `f3c9265` | `fix(F6): pin V6 scorecard value to contract metric key` | Pins scorecard value to the contract metric key instead of prose/label drift. |
| `b90a4e7` | `test(F6): unit tests for V6 scorecard helpers (contract-key pin + splice)` | Adds helper tests for contract-key pinning and splice behavior. |

Primary code surfaces touched by those commits:

- `crates/memd-client/src/benchmark/substrate/star_writer_v6.rs`
- `crates/memd-client/src/benchmark/substrate/runtime.rs`
- `crates/memd-client/src/benchmark/typed_ingest/mod.rs`
- `crates/memd-client/src/benchmark/public_benchmark.rs`
- `scripts/public-bench-reproduce.sh`

## Verification Carried Forward

From the previous real-backend lock handoff:

- `cargo test -p memd-client` -> 836 passed, 0 failed, 10 ignored.
- `memd benchmark substrate --all --regenerate-report --regenerate-10star` -> 7/7 suites pass; F5 live-fire block shows `live_fire_pass=1.000`.

This sync pass refreshed docs/handoff state and merge truth only. If the
next agent wants a quick F6 code check before deeper V6 work, run the
targeted F6 substrate tests first, then the full substrate benchmark when
ready to regenerate public artifacts.

## Exact Next Move

1. Commit this handoff refresh on `research/mining`.
2. Push `origin/research/mining`.
3. `git switch main`
4. `git pull --ff-only origin main`
5. `git merge --ff-only research/mining`
6. `git push origin main`
7. Continue V6 from `main` or create a fresh `codex/v6-*` branch from `main`.
