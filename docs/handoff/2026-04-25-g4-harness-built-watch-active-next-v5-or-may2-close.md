---
opened: 2026-04-25
phase: G4
status: harness-built-watch-active
prev_handoff: 2026-04-24-f4-code-complete-next-f4-7.md
next_step_a: V5 phase docs review (user-signaled direction; calendar-safe — does not consume V4 close)
next_step_b: 2026-05-02 G4.7 close (cron job e96a5ac3 scheduled, session-only — re-/schedule on next wake if session died)
v4_close_gates:
  - 7-day CI nightly watch closes 2026-05-02 (.github/workflows/v4-proof-harness.yml)
  - D4.8 / E4.7 / F4.7 dogfood harvest earliest 2026-05-01
  - composite rescore via G4.4 regenerator, gate ≥3.45 (NOT 4.0)
---

# G4 harness machinery built, V4 in calendar watch — next is V5 prep or May 2 close

V4 Live Loop Repair is **code-complete on `research/mining`**. All G4
sub-tasks merged. V4 cannot CLOSE today — closing requires the 7-day
stability watch + 2026-05-01 dogfood harvest. User signaled V5 as the
next direction; that work is calendar-safe (does not consume V4 close
budget).

## What landed (this session, 2026-04-25)

| Task | Commit | Notes |
|------|--------|-------|
| ROADMAP truth → 2026-04-25 + F4 code-complete | `5d04278` | F4.7 dogfood clock started in `~/.zshrc` (`MEMD_F4_PREF_DRIFT=1`) |
| G4.1 — 3-session fixtures + 6 fault-inject overrides | `c0f83cc` | `crates/memd-client/fixtures/g4/{session-{1,2,3}.jsonl, expected-cut-{1,2,3}.json, seed-state.json, inject-faults/}` |
| G4.2 — in-process driver, 2 tests | `fecaea5` | `crates/memd-client/src/main_tests/v4_proof_harness/mod.rs`. In-process, NOT spawned binary (advisor call) |
| G4.3 — 6 cross-V4 asserters, tests 3–8 | `445040d` | `assertions.rs` — A4/B4/C4/D4/E4/F4 |
| G4.4 — strict-mode 10-STAR regenerator, tests 9+10 | `251539d` | `scorecard.rs` — refuses over-claim, threads NDJSON pointer |
| G4.5 — bash CI entrypoint + GH workflow + tests 11+12 | `fd7691e` | `scripts/ci/v4-proof-harness.sh` + `.github/workflows/v4-proof-harness.yml` (push gate + nightly cron 03:00 UTC) |
| G4.6 stability pass #1 + G4.7 watch-active scaffold | `1127ae8` | `docs/verification/v4-proof-runs/2026-04-25-stability-pass-1.md`, MILESTONE-v4 status → `harness-built-watch-active` |
| Fixture composite_min 4.0 → 3.45 | `41924ae` | Reconciles fixture-vs-milestone drift |
| G4.2.3 cross-harness flip + G4.2.4 F4.7 counter asserters | `904e15f` | 3 new tests; cross_harness axis can't lift past 2 without G4.2.3 evidence at close |

## Verify-green commands

```sh
CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-client v4_proof_harness
CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-client
bash scripts/ci/v4-proof-harness.sh
```

Last run: **memd-client 596 pass** (+15 vs session start 581). CI script exit 0.

## V4 close gates (CANNOT touch before 2026-05-02)

1. **7-day CI nightly watch** — `.github/workflows/v4-proof-harness.yml`
   cron `0 3 * * *`. Closes 2026-05-02. Any flake = root-cause source phase
   per `phase-g4-plan.md §G4.6`, no retry.
2. **Dogfood harvest** — D4.8 / E4.7 / F4.7 7-day env-flag clocks.
   F4.7 started 2026-04-25 in `~/.zshrc`. Earliest harvest 2026-05-01.
   Read `<.memd>/logs/preference-drift.ndjson` for restate-rate, wake
   token telemetry, lookup depth + escalation hints.
3. **Composite rescore** — invoke `scorecard::regenerate_scorecard()`
   against harvested NDJSON. Gate ≥ **3.45** (per MILESTONE-v4.md
   composite_target — NOT the legacy 4.0 still floating in
   `expected-cut-3.json` until the close commit reconciles it).

## Cron scheduled

Job `e96a5ac3` — fires 2026-05-02 09:47 local — runs G4.7 close attempt.
**Session-only** (despite durable: true flag — scheduler reported
"not written to disk"). If this Claude session dies before May 2,
re-/schedule on next wake.

## Next direction (user-signaled this session)

**V5: Substrate-Native Benchmark Suite** — axis lift PR 2→4, CH 3→4, RR 4→6.

V5 phase docs already drafted at milestone-open:

```
docs/phases/v5/
├── V5-INTEGRATION.md
├── phase-a5-cross-session-recall.md  + plan
├── phase-b5-correction-propagation.md + plan
├── phase-c5-cross-harness-continuity.md + plan
├── phase-d5-progressive-depth.md + plan
├── phase-e5-provenance-integrity.md + plan
├── phase-f5-typed-retrieval.md + plan
└── phase-g5-adversarial-noise.md + plan
```

V5 completion gate (per ROADMAP.md:289): all 7 bench suites run in CI,
numbers in `docs/verification/SUBSTRATE_BENCHMARKS.md`, any memd
competitor can run them.

**Recommended pickup sequence:**
1. Sanity-pass V5 phase docs (drafted at V4 open — may have stale
   refs to V4 surfaces that drifted during execution).
2. Pick V5 entry phase (A5 cross-session-recall is the natural
   start per integration doc).
3. **Do not commit V4 → complete in ROADMAP.md** until G4.7 close
   commit lands on or after 2026-05-02 with composite ≥3.45.

## Other unblocked (not V4, not V5)

- **V3 tail** — canonical rerun via codex-lb (`OPENAI_BASE_URL=http://127.0.0.1:2455/v1`,
  `OPENAI_API_KEY=$CODEX_LB_API_KEY`). LongMemEval + LoCoMo + ConvoMem.
  Listed in ROADMAP `v3_tail_followups`. Standalone, separable from V4/V5.

## Repo hygiene note

`crates/memd-client/.memd/state/raw-spine.jsonl` modified since session
start, never staged. Probably benign dogfood writes from local memd
verbs; investigate before next clean checkpoint.

## Voice

caveman-ultra. Terse, fix don't explain, no trailing summaries.
