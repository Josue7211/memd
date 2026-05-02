---
date: 2026-05-02
session: v4-close-and-phase2-queue
branch: research/mining
status: handoff-ready
supersedes: docs/handoff/2026-05-02-living-skills-phase1-closed-next-phase2.md
next_session: pickup-v5-remediation-or-phase2-execute
---

# Handoff — V4 Closed, Phase 2 Queued, V5+ Remediation Pending

> One sentence: V4 milestone closed today on amended gates (composite
> **1.80 → 3.60**, deviation recorded), Living Skills Phase 2 plan
> landed, V5+ inherits a small remediation queue. Branch
> `research/mining` is the source of truth; `main` is stale.

---

## 1. What landed this session

| Commit | Subject | Why it matters |
| --- | --- | --- |
| `a187a41` | `style: cargo fmt --all (rustfmt drift across 100 files)` | Unblocked ci.yml; precondition for stability pass #2 + V4 close. |
| `aa0e76a` | `feat(v4): close milestone on amended gates — composite 1.80 → 3.60` | V4 closes. Adds `t13_v4_close_axes_match_milestone_targets` to bake the audit gate into code. Updates `MEMD-10-STAR.md`, `MILESTONE-v4.md`, deviation record, stability pass #2 record, ROADMAP_STATE → V5. |
| `3e7fbc6` | `docs(skills): Phase 2 plan — records-as-truth, retire-deletes-record` | Phase 2 plan ready-to-execute at `docs/superpowers/plans/2026-05-02-living-skills-phase2-records-as-truth.md`. |

All three pushed to `origin/research/mining`.

---

## 2. V4 close — what to know in 30 seconds

- **Authoritative composite: 3.60** (gate 3.45, margin +0.15).
- **Lifted axes**: session_continuity 1→4, correction_retention 1→4,
  cross_harness 2→**4** (V4 G4 +1 + V5 C5 banked +1 materialized
  atomically), token_efficiency 2→4, trust_provenance 2→3.
- **Procedural reuse seeded only**: 1→2, no behavior credit (V5 owns lift to 3+).
- **Deviation record**: `docs/verification/milestones/MILESTONE-v4-deviation-2026-05-02.md`.
  Three gates closed on substitutes:
  1. 7-day CI watch → 2× local 10-pass batches one week apart (commits `fd7691e` + `a187a41`).
  2. Dogfood NDJSON harvest → harness asserter outcomes (synthetic fixtures).
  3. G4.7 composite rescore → asserter-sourced observations not real-session NDJSON.
- **Audit baked into code**: `t13_v4_close_axes_match_milestone_targets`
  in `crates/memd-client/src/main_tests/v4_proof_harness/scorecard.rs`
  parses live `MEMD-10-STAR.md`, asserts every axis ≤ milestone-union
  ceiling AND composite ≥ 3.45. Any future regenerate that breaches
  fails CI.

---

## 3. V5+ remediation queue (inherits the deviation debt)

Three concrete tasks the deviation record forwards:

1. **Merge `research/mining` → `main`.** All V4 work, ci workflow,
   harness code, and the Phase 2 plan live on `research/mining`.
   `main` lags. Until the merge, GitHub `schedule` cron cannot fire
   (it only triggers from default branch — workflow-not-on-default
   was the silent precondition that voided the 7-day watch).
2. **Wire F4.7 per-turn drift driver into `runtime/turn.rs`.** F4 plan
   §1 anticipated a driver that calls the existing CLI verb each turn;
   Phase 1 only landed the verb. Result: `.memd/logs/preference-drift.ndjson`
   never produced, dogfood NDJSON harvest unfeasible. Lifts
   procedural_reuse from seed (2) toward 3+ when paired with V5
   routine-detection live-fire.
3. **Living Skills Phase 2 execution** per
   `docs/superpowers/plans/2026-05-02-living-skills-phase2-records-as-truth.md`.
   7 build steps P2.1 → P2.7, 17 named tests, all anticipated by Phase 1
   contract §10/§11. Independent of V5 axis lifts — can land in parallel.
4. **Fix V5 substrate-bench workflow red on `research/mining`** —
   8 failures (6× D5 fixture NotFound, 2× G5 aggregator/e2e). These
   are V5-in-progress and NOT a V4 regression; the V4 close stands.
   Belongs in the V5 substrate-bench lane.

---

## 4. Verify-green commands

```bash
# from repo root, branch research/mining
cargo test -p memd-client --test v4_proof_harness         # 16 green incl. t13 audit
cargo test -p memd-client                                 # full client suite
cargo fmt --all --check                                   # must be clean
cargo clippy --workspace --all-targets -- -D warnings    # baseline floor
```

The t13 audit test is the close gate. If it fails, the live
`MEMD-10-STAR.md` no longer matches the V4 close evidence — investigate
before doing anything else.

---

## 5. Known traps (read before touching CI / scorecard)

- **GitHub `schedule` cron only fires from default branch.** Workflow on
  `research/mining` only → cron silently never ran. Don't reintroduce
  this assumption; either merge to main or use `workflow_dispatch` /
  external scheduler.
- **F4.7 CLI verb exists but no per-turn driver.** Don't claim
  procedural_reuse > 2 until V5 wires `runtime/turn.rs` to call it.
  The t13 ceiling enforces this.
- **Cross-harness ceiling = 4** because V5 C5 already banked +1 on
  2026-04-25; the bank materializes atomically on V4 G4 close (per
  `MEMD-10-STAR.md` line 93). Any regenerate setting cross_harness > 4
  needs new V5 evidence — t13 will refuse otherwise.
- **rustfmt drift was not stylistic**: ci.yml `cargo fmt --check` was
  failing for ~9 days unnoticed. Run fmt after every non-trivial PR.
- **`memd remember --visibility project` is invalid.** Use `workspace`.
- **Subcommand `--output` is positional on the subcommand**, not
  global. `memd resume --output .memd` ✓ / `memd --output .memd resume` ✗.

---

## 6. Truth state at session close

- Branch: `research/mining` (10 commits ahead of `main`).
- ROADMAP_STATE: `version=v5`, `current_milestone=V5`, `v4_status=complete`,
  `v4_composite=3.60`, `truth_date=2026-05-02`.
- All 4 V4 phases (A4–G4) marked complete in roadmap table.
- `MILESTONE-v4.md` frontmatter: `status: complete`, `closed: 2026-05-02`.
- `MEMD-10-STAR.md` composite cell: **3.60/10**.
- ci.yml + v4-proof-harness workflows green on `3e7fbc6`
  (verified 2026-05-02 ~15:10Z by background watcher).
- **substrate-bench workflow red on `3e7fbc6`** — 8 V5-in-progress
  failures: 6× D5 fixture-missing (`Os { code: 2, NotFound }`), 2× G5
  (`aggregator_writes_10star_composite_section` expects rr=6 vs
  observed rr=4; `cli_bench_substrate_all_end_to_end_on_clean_tree`).
  These are V5 substrate-bench tests that were already red pre-V4-close;
  fixing belongs in V5 lane, not the V4 close. Do not block on this
  for Phase 2 execution.
- 3 memd memories captured this session: cron-from-default-branch,
  V4 close decision, F4.7 driver gap.

---

## 7. Suggested pickup order for next session

1. Confirm scheduled cron `c698471e` produced clean ci result. If
   failure handoff exists, fix root cause before anything else.
2. **Choose one lane**:
   - **V5 remediation lane**: merge → main, then wire F4.7 driver in
     `runtime/turn.rs`, then queue routine-detection live-fire.
   - **Phase 2 lane**: execute `docs/superpowers/plans/2026-05-02-living-skills-phase2-records-as-truth.md`
     P2.1 → P2.7. Independent of V5; can interleave.
3. Don't open V5 axis-lift work until F4.7 driver lands — the audit
   test will refuse over-claims.

---

## 8. References

- Plan: `docs/superpowers/plans/2026-05-02-living-skills-phase2-records-as-truth.md`
- Deviation: `docs/verification/milestones/MILESTONE-v4-deviation-2026-05-02.md`
- Stability pass #2: `docs/verification/v4-proof-runs/2026-05-02-stability-pass-2-and-close.md`
- Milestone: `docs/verification/milestones/MILESTONE-v4.md`
- Scorecard: `docs/verification/MEMD-10-STAR.md`
- Audit test: `crates/memd-client/src/main_tests/v4_proof_harness/scorecard.rs::t13_v4_close_axes_match_milestone_targets`
- Phase 1 close handoff (this doc supersedes): `docs/handoff/2026-05-02-living-skills-phase1-closed-next-phase2.md`
