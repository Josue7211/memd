---
milestone: v8
name: Operator Surfaces
status: planned
opened: 2026-04-22
revised: 2026-04-22
depends_on: [v7]
composite_pre: 4.90
composite_target: 5.10
axes_lifted: [token_efficiency, trust_provenance]
axes_integrated_with: [session_continuity]
---

# Milestone v8 Audit — Operator Surfaces

## Goal

Operator can see and tune memd. Visible cost ledger (token budget inspector, per-turn burn tracking) enables budget-aware operation. Provenance browser (depth 3+ drilldown to source turn + correction history) replaces partial surfaces from prior milestones. Correction UX, atlas navigation, memory inspector, diff + rollback all updated for operator workflow. Competitor surfaces (mempalace atlas, mem0 dashboard, letta correction UX) are the bar for feature parity.

## 10-STAR axis targets (pre / post)

Scores match the 0.1.0-CONTRACT.md baseline (zero-generosity regrade) and V7 post-state.

| axis | weight | pre | post | basis |
| --- | --- | --- | --- | --- |
| session_continuity   | 20% | 5 | 5 | A8–F8 read continuity data; INT w/ V7 (no credit claimed) |
| correction_retention | 15% | 5 | 5 | stable; no V8 work |
| procedural_reuse     | 15% | 4 | 4 | stable; no V8 work |
| cross_harness        | 15% | 4 | 4 | stable; no V8 work |
| raw_retrieval        | 15% | 7 | 7 | stable; no V8 work |
| token_efficiency     | 10% | 4 | 5 | A8 cost ledger + D8 provenance depth tracking; operator can see and tune budget |
| trust_provenance     | 10% | 5 | 6 | D8 drilldown (depth 3+ to source turn + correction history + alternate candidates) |

**Composite: 4.90 → 5.10** (weighted arithmetic).

## TE margin note (superseded)

Original framing: TE tightest at release. **Superseded by V11+ SOTA push** — V11
Compiler SOTA lifts TE 5→7 (dynamic per-turn compiler), V13 closes at TE=7 against
SOTA floor 7 (zero margin). V8's TE 4→5 is an intermediate lift, not the release-
critical axis. V8 TE assertion still required (cost ledger + tunable budget) but
failure here is recoverable in V11; V13 zero-margin is the real release cliff.

## Phases

See `ROADMAP.md` → "V8: Operator Surfaces". Phase docs at `docs/phases/v8/phase-{a8..f8}-*.md`.

- **A8** Atlas navigation UI — graph view of canonical memory, click to provenance. SC integration.
- **B8** Correction UX — inline capture, live preview of "before/after" retrieval. SC integration.
- **C8** Memory inspector — all records by type, searchable, filterable. SC integration.
- **D8** Provenance browser — click any fact, trace to source turn + extraction reason + correction history. **OWNS TP +1 (5→6).**
- **E8** Cost ledger UI — token budget cap, per-turn burn tracking, cumulative cost graph. **OWNS TE +1 (4→5).**
- **F8** Public leaderboard transparency page — live method cards, reproduction commands, retraction log, gaming-audit rule. SC integration.
- **G8** `memd configure` settings CLI — single canonical entry point for all runtime settings. Subcommands: `memd configure list`, `memd configure get <key>`, `memd configure set <key>=<value>`, `memd configure reset [<key>]`. Writes to `.memd/config.json` (schema v0.3+). Exposes V7 H7 atomic-commit toggle (`auto_commit.enabled`), V8 cost-ledger budget caps, V9 federated-memory visibility defaults, and future V11-V13 feature toggles. Must validate keys against schema (unknown key = error with "did you mean"). TAB-completion for keys in zsh/bash. No axis credit claimed; foundational for operator UX consistency. Phase G8 is the sole canonical config surface — all other "settings" references in the codebase either delegate to it or are deprecated.

## Completion gate

G8 is dual-deliverable: (1) `memd configure` CLI (no axis credit; canonical settings surface) + (2) closing release harness (TE + TP axis credit aggregator + scorecard regenerator). Both land inside G8. The harness (headless browser automation via agent-browser / Playwright) covers:

### TE proof (cost ledger visible, operator can control budget)
- G8.TE.1: dev server running, operator session via agent-browser, calls `memd wake --output ~/.memd --budget-tokens 3000`
- G8.TE.2: cost ledger UI displays 3000 cap + current burn; inspector shows per-turn tokens
- G8.TE.3: operator calls `memd preference set --budget-tokens 2000`, UI refreshes, cap updates
- G8.TE.4: turn executed under new cap; if new turn exceeds 2000, wake stops; if under, wake includes turn
- G8.TE.5: metric `cost_ledger_visible: true` and `budget_tunable: true` logged to G8 proof run

### TP proof (provenance depth 3+ drilldown to source turn + correction history)
- G8.TP.1: operator clicks on a memory fact in the provenance browser
- G8.TP.2: drilldown shows depth 1: the fact itself + metadata (extraction method, confidence)
- G8.TP.3: drilldown depth 2: source turn (which session turn produced this extraction)
- G8.TP.4: drilldown depth 3: correction history (all corrections that cite this fact) + alternate candidates (other extractions from same turn not chosen)
- G8.TP.5: metric `provenance_depth_max: 3` or higher logged to G8 proof run

### Configure CLI proof (no axis credit; settings-surface stability)
- G8.CFG.1: `memd configure list` prints all 6 V8 keys + defaults (auto_commit.enabled, cost_ledger.budget_tokens, cost_ledger.per_turn_warn, provenance.drilldown_depth_max, voice.mode + reserved stubs)
- G8.CFG.2: `memd configure set cost_ledger.budget_tokens=2000` → writes `.memd/config.json` atomically via V7 H7 writer-guard
- G8.CFG.3: `memd configure get cost_ledger.budget_tokens` → "2000"
- G8.CFG.4: `memd wake` respects new budget (reads config, not env)
- G8.CFG.5: `memd configure set unknown.key=1` → exits 2, emits "did you mean" hint (Levenshtein ≤2)
- G8.CFG.6: `memd configure reset cost_ledger.budget_tokens` → default restored
- G8.CFG.7: schema hash unchanged vs committed snapshot (drift = blocker)
- G8.CFG.8: metric `configure_suite.pass_count=7, fail_count=0` logged to G8 proof run

### Stranger review (outside reviewer, sidecar OFF)
- Reviewer rates memd best-in-class on 5 surfaces (wake quality, correction UX, atlas navigation, memory searchability, cost ledger readability) vs mempalace / supermemory / letta / mem0
- Evidence: reviewer write-up + 5 side-by-side screencasts + zero console errors in browser devtools

Evidence: recorded trace + G8 harness NDJSON (TE + TP assertions) + regenerated 10-STAR composite in `docs/verification/MEMD-10-STAR.md` via G8 scorecard regenerator.

## Per-axis harness assertions (required for axis credit)

| axis | concrete assertion | fixture |
| --- | --- | --- |
| token_efficiency | cost ledger displays budget cap; operator edits cap to 2000; new turn respects cap | E8 headless scenario + agent-browser |
| trust_provenance | click memory fact → depth 1 (metadata) → depth 2 (source turn) → depth 3 (correction history + alts) | D8 headless scenario + agent-browser |
| session_continuity | operator session reads continuity data from prior session (no lift claimed) | G8 shared fixture replay |

Missing TE or TP assertion → axis does not lift, milestone does not close. SC credit flows to V7, not V8.

## Non-goals (does not touch)

- correction_retention (V7 owns)
- procedural_reuse (V5 owns)
- cross_harness (V4 owns; V8 integrates)
- raw_retrieval (V5 owns)
- mobile UI (scoped out)
- IDE integration UI beyond claude-code/codex (scoped out)

## Browser testing mandate (from project CLAUDE.md)

All V8 UI work requires:
1. Dev server running (memd-web or equivalent)
2. Scripted interaction via agent-browser (Playwright fallback if needed)
3. Verify end-to-end feature works (cost ledger updates, provenance drilldown succeeds)
4. **Zero console errors** in browser devtools
5. Screenshot captured for harness proof

No UI change ships without live browser verification + harness proof. This is mandatory for TE axis credit (tightest margin at release).

## Changelog

- 2026-04-22 opened — initial spec (composite_target 8.5, stale ownership).
- 2026-04-22 revised:
  - composite_pre 7.8 → 4.90 (V7 post-state per 0.1.0-CONTRACT.md)
  - composite_target 8.5 → 5.10 (V8 targets per 0.1.0-CONTRACT.md)
  - axes_lifted corrected to TE +1 (4→5), TP +1 (5→6) per 0.1.0-AXIS-OWNERSHIP.md
  - axes_integrated_with SC (operator surfaces read continuity data; no credit)
  - Phase E8 split: atlas/correction/inspector → A8/B8/C8 (SC integration), provenance → D8 (TP owned), cost ledger → E8 (TE owned), transparency → F8 (SC integration)
  - Non-goals CR, PR, CH, RR explicit per axis-ownership table
  - TE margin risk section added (TE +2 total margin, V8 owns +1, floor violation blocker)
  - G8 harness assertions added: TE proof (cost ledger visible + tunable), TP proof (depth 3+ drilldown)
  - Browser testing mandate added; agent-browser required for UI axis credit
  - Per-axis harness assertions table added (no lift without G8 proof)
  - Stranger review moved to completion gate + required
- 2026-04-22 revised (V11-V13 SOTA extension):
  - TE-margin-risk framing superseded — V11 takes TE 5→7, V13 closes at zero margin; V8's TE=5 is intermediate, not release-critical
  - Added G8 phase — `memd configure` settings CLI. Canonical entry point for all runtime settings, exposes V7 H7 atomic-commit toggle plus V8-V13 feature flags. No axis credit; foundational operator UX.
