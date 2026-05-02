---
milestone: v4
name: Live Loop Repair
status: complete
opened: 2026-04-22
revised: 2026-04-25
closed: 2026-05-02
deviation_record: docs/verification/milestones/MILESTONE-v4-deviation-2026-05-02.md
depends_on: [v3]
composite_pre: 1.80
composite_target: 3.45
composite_observed: 3.60
axes_lifted: [session_continuity, correction_retention, cross_harness, token_efficiency, trust_provenance]
axes_seeded_no_credit: [procedural_reuse]
gates_pending: []
gates_amended:
  - g4_6_seven_day_ci_stability_watch  # 2× local 10-pass batches accepted (see deviation record)
  - d4_8_e4_7_f4_7_dogfood_harvest     # harness asserters accepted in lieu of real-session NDJSON
  - g4_7_composite_rescore_at_3_45     # observed 3.60 ≥ 3.45 (asserter-sourced observations)
---

# Milestone v4 Audit — Live Loop Repair

## Goal

memd used-as-designed in a real claude-code or codex session does not lose
state, does not drop corrections, does not bloat context. Fixes 10-STAR
gaps 1–9 (the pre-V3 "memory OS broken in daily use" set) and seeds
procedural-detection instrumentation without claiming lift. No
public-bench chasing; the win is the dogfood feel.

## 10-STAR axis targets (pre / post)

Scores match the 0.1.0-CONTRACT.md baseline (zero-generosity regrade).

| axis | weight | pre | post | basis |
| --- | --- | --- | --- | --- |
| session_continuity   | 20% | 1 | 4 | A4 ledger survival + B4 enforced hooks + D4 compiler |
| correction_retention | 15% | 1 | 4 | C4 E2E + F4 preference drift; 7-day dogfood precision ≥0.85 |
| procedural_reuse     | 15% | 1 | 2 | F4.7 seed — instrument dead path, no behavior proof (no lift claimed beyond 2) |
| cross_harness        | 15% | 2 | 3 | G4 cross-harness flip test (correction in harness A visible in harness B) |
| raw_retrieval        | 15% | 4 | 4 | no V4 work — V5 substrate bench target |
| token_efficiency     | 10% | 2 | 4 | D4 compiler + E4 depth contract; wake median ≤2000 tokens, cost measured |
| trust_provenance     | 10% | 2 | 3 | C4 provenance visible, B4 trace surfaced; drilldown still partial |

**Composite: 1.80 → 3.45** (weighted arithmetic).

### Why procedural_reuse is capped at 2 not 3

Agent D codebase audit confirmed `RetrievalIntent::Procedural` has no caller
in runtime; the detect path is dead. V4's F4.7 seed wires the instrumentation
and adds a metric but does not prove behavior change. V4 claims **no axis
credit beyond +1** (still below the 0.1.0 per-axis floor of 3). V5 owns the
lift to 3+ via routine-detection live-fire.

Any regeneration of this table that scores procedural_reuse > 2 without V5
work landing is invalid.

## Phases

See `ROADMAP.md` → "V4: Live Loop Repair". Phase docs at
`docs/phases/v4/phase-{a4..g4}-*.md`. F4.7 is the intra-F4 seed task — no
separate phase.

## Completion gate

3-session multi-harness dogfood recording (G4 harness):

- state survives compaction (A4)
- hook order honored under load (B4)
- correction in session 1 honored in session 3 (C4, F4)
- wake context ≤2k tokens with zero continuity loss (D4, E4)
- **Cross-harness flip**: correction issued in claude-code session 1 is
  present in codex session 2 retrieval for the same workspace (G4)
- G4 harness passes end-to-end with negative controls firing as designed

Evidence: recorded trace + G4 harness NDJSON + regenerated 10-STAR
composite in `docs/verification/MEMD-10-STAR.md` via G4 scorecard
regenerator.

## Per-axis harness assertions (required for axis credit)

| axis | concrete assertion | fixture |
| --- | --- | --- |
| session_continuity   | cut 2 wake reconstructs cut 1 focus within budget | shared/sessions/session-1.jsonl |
| correction_retention | T22 answers "ulid" not "uuid"; T21 answers "2026-05-15" not "2026-05-01" | G4 scenario |
| procedural_reuse     | metric "routine_candidates_observed" ≥ 1 after T25 (no behavior assertion) | F4.7 instrumentation |
| cross_harness        | correction from claude-code S1 observable from codex S2 via lookup | G4 cross-harness flip |
| token_efficiency     | wake median ≤ 2000 tokens across 3 sessions; cost ledger written | D4 + E4 metrics |
| trust_provenance     | every correction carries source-turn provenance queryable via explain | C4 + B4 trace |

Missing any assertion → axis does not lift, milestone does not close.

## Non-goals

- public bench number chasing (V6 owns that)
- operator UI polish (V8)
- multi-user enforcement (V9 — contract published in V4 via
  docs/contracts/federated-memory-visibility.md but not enforced)
- procedural axis to 3+ (V5 scope; V4 only seeds)

## Pass evidence (G4.7 — accumulating)

- G4.1 fixtures (commit `c0f83cc`) — 3-session scenario + expected cuts +
  seed-state + inject-faults README in `crates/memd-client/fixtures/g4/`.
- G4.2 driver (commit `fecaea5`) — `crates/memd-client/src/main_tests/v4_proof_harness/mod.rs`,
  2 tests green (parse + 3-session run with simulated PreCompact).
- G4.3 cross-V4 assertions (commit `445040d`) — 6 asserters (A4/B4/C4/D4/E4/F4)
  + 6 fault-inject fixtures in `crates/memd-client/fixtures/g4/inject-faults/`.
- G4.4 scorecard regenerator (commit `251539d`) — strict mode refuses
  over-claims; updates `## 10-Star Composite Scorecard` table in place.
- G4.5 CI entrypoint + workflow (commit `fd7691e`) — `scripts/ci/v4-proof-harness.sh`
  + `.github/workflows/v4-proof-harness.yml` (push gate + nightly cron 03:00 UTC).
- G4.6 stability pass #1 (2026-04-25) — `docs/verification/v4-proof-runs/2026-04-25-stability-pass-1.md`,
  10/10 local back-to-back green. 7-day CI watch underway.

## Open gates before V4 closes

**All gates resolved 2026-05-02 with deviation amendments — see
`MILESTONE-v4-deviation-2026-05-02.md` for the formal record.**

1. **G4.6 7-day CI stability** ~~`.github/workflows/v4-proof-harness.yml`
   nightly must hit 10/10 by 2026-05-02.~~ → **Amended.** GitHub `schedule`
   cron only fires from default branch; workflow lives on `research/mining`
   only. Substitute: two local 10× back-to-back stability passes one week
   apart on sequential commits (`fd7691e` 2026-04-25 + `a187a41`
   2026-05-02), 10/10 each, zero breach lines.
2. **Dogfood harvest** ~~D4.8 / E4.7 / F4.7 7-day env-flag clocks (running
   since 2026-04-25 for F4) earliest harvest 2026-05-01. Required for
   correction_retention + token_efficiency + procedural_reuse axis evidence.~~
   → **Amended.** F4.7 per-turn drift tick was never wired into the
   runtime hook path; CLI verb exists but no driver invokes it per turn.
   `.memd/logs/preference-drift.ndjson` not produced. Substitute: harness
   asserter outcomes (synthetic fixtures) per `§"Per-axis harness
   assertions"` accepted as axis-credit evidence.
3. **G4.7 composite rescore** — invoked against asserter-sourced axis
   observations (deviation: not real-session NDJSON). Composite **3.60**
   ≥ 3.45 gate satisfied with 0.15 margin. Strict-mode over-claim refusal
   property preserved.
4. **`continuity-breach.log`** — empty across both 10× stability passes.

## Changelog

- 2026-04-22 opened.
- 2026-04-22 revised: composite_pre 2.15 → 1.80 (reconciled with zero-
  generosity 10-STAR regrade); composite_target 4.0 → 3.45; procedural_reuse
  demoted 1→3 to 1→2 (F4.7 seed only, no behavior credit); cross-harness
  flip assertion added to G4 gate; per-axis harness assertions table added
  to enforce "no axis credit without harness proof" rule from 0.1.0-CONTRACT.
- 2026-04-25 G4 harness machinery landed (G4.1–G4.5, commits c0f83cc → fd7691e),
  G4.6 stability pass #1 logged (10/10 local). Status moved planned →
  harness-built-watch-active. Awaiting 7-day CI watch + 2026-05-01 dogfood
  harvest before G4.7 close + composite rescore.
- 2026-05-02 V4 closes on amended gates. Stability pass #2 (10/10 local)
  + composite rescore 3.60 ≥ 3.45 gate. Two preconditions failed silently
  (workflow-not-on-default-branch blocked the 7-day cron; F4.7 per-turn
  driver never wired blocked the dogfood NDJSON harvest). User-authorized
  deviation recorded in `MILESTONE-v4-deviation-2026-05-02.md`. V5+
  inherits remediation: cron infra + per-turn drift driver wiring.
  Status: harness-built-watch-active → complete.
