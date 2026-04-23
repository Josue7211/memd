---
version: v8
kind: integration-plan
status: ready-to-execute
opened: 2026-04-22
revised: 2026-04-22
scope: A8..G8
depends_on: [../../verification/0.1.0-CONTRACT.md, ../../verification/0.1.0-AXIS-OWNERSHIP.md, ../../verification/milestones/MILESTONE-v8.md]
---

# V8 Integration — Cross-Phase Plan

> Read after all six `phase-{a8..f8}-plan.md` specs. This doc covers what no single phase plan owns: shared fixtures, execution order, UI-specific G8 harness requirements (headless browser via agent-browser), scorecard regenerator strict-mode, TE-tightest-margin risk tracking, and the commit strategy for the spec-land phase itself.

## 1. Execution-order discipline

Phase-level dependency (strict):

```
A8 ──► B8 ──► C8 ──┐
         │         │
         └────► D8 ──┐
                     │
                  E8 ┤
                     │
              F8 ────┤
                     │
                     ▼
                     G8
```

Rules:

- A8 tasks 1–6 land first (atlas UI foundation + SC integration contract). B8 cannot start until A8 Task A8.5 (SC read API finalized).
- C8 parallelize with B8 after A8 completes; C8 uses A8 atlas foundation + A8 SC read API.
- D8 requires A8 atlas foundation + A8 SC read (integration). E8 requires D8 provenance API. F8 parallelize with E8 after D8 Task D8.4 (provenance depth contract).
- G8 requires all A8–F8 complete. **G8 is dual-deliverable:** (a) `memd configure` CLI (the canonical settings surface — OWNS foundational operator UX; no axis credit) + (b) scorecard regenerator + closing release harness (TE + TP axis credit aggregator). Both land inside G8.
- G8's `memd configure` sub-phase can parallelize against A8–F8 UI work (no UI dependency), but its schema MUST be finalized before E8 ships cost-ledger budget caps (E8 reads defaults via `memd configure get cost_ledger.budget_tokens`).

No phase may short-circuit a prior dependency to hit its own pass gate. If blocked, file a backlog item and surface in the next session's handoff.

## 2. Shared test fixtures

To avoid fixture drift across phases, the following live under a shared dir and are referenced by multiple phase plans:

| Fixture | Owner | Shared with | Purpose |
| --- | --- | --- | --- |
| `crates/memd-web/fixtures/shared/sessions/operator-session-canonical.jsonl` | G8 | A8/B8/C8/D8/F8 (atlas, correction, inspector, provenance read paths) | 3-turn operator workflow recording (budget inquiry → correction → memory search) |
| `crates/memd-web/fixtures/shared/transcripts/budget-dialog-turns.jsonl` | E8 | G8 (cost ledger harness proof) | 5 turns exercising budget cap edit + enforcement |
| `crates/memd-web/fixtures/shared/transcripts/provenance-drilldown-turns.jsonl` | D8 | G8 (TP harness proof) | 3 turns: depth 1 (metadata) → depth 2 (source) → depth 3 (corrections + alts) |
| `crates/memd-web/fixtures/shared/preferences/operator-prefs.jsonl` | A8 | G8, D8 (integration), E8 (budget preference) | Operator preferences including budget-tokens, display theme |

Convention: each phase plan's `fixtures/<phase>/` dir contains **only** fixtures unique to that phase. Shared fixtures move to `fixtures/shared/` the moment a second phase references them, with a compat shim in the original dir (symlink or `pub use`).

First consolidation happens after C8 lands — harvest A8's atlas fixture + C8's searchable records into shared.

## 3. SC integration contract (no axis credit)

V8 **integrates** with session_continuity but claims **zero credit** (V7 owns SC +1 final; V8 only reads the data).

### Integration points (read-only, no SC lift)

- **A8 atlas UI** reads from `session_continuity` fact store (ledger from V4, corrections from V7). Displays graph of canonical memory + recovery paths. No new SC logic.
- **B8 correction UX** reads correction entries populated by V7 C7. Inline capture uses V7 judge client (shared budget). No new SC logic.
- **C8 memory inspector** queries `session_continuity` metadata (session_id, turn_id, wall-clock). Searchable/filterable display. No new SC logic.
- **F8 public leaderboard** surfaces retraction log (corrections visible to anyone) + gaming-audit rule. Reads from V7 correction tables; audit logic is F8-new but **does not modify SC score**.

Assertion: "operator can read continuity data from prior session" is measured by G8 but credited to V7, not V8.

## 4. UI-specific G8 harness (headless browser automation)

The G8 harness differs from V4–V7 in that it exercises **UI endpoints**, not just backend APIs. Harness must:

1. **Spawn headless browser** via agent-browser (primary) or Playwright (fallback).
2. **Scripted interaction** (click, fill, submit).
3. **Assertion on DOM state** (element visibility, text content, CSS computed style).
4. **Screenshot capture** for every axis proof (TE cost ledger, TP drilldown).
5. **Console error detection** — fail if any `window.onerror`, unhandled promise rejection, or `console.error` fires.

### Tool requirements

- `agent-browser` CLI available on PATH (from homelab-cli repo or shipped in memd-web container).
- Fallback: `npx playwright` if agent-browser unavailable (slower, less reliable for multi-turn scenarios).
- Browser DevTools logs captured to G8 proof NDJSON: `{ "type": "console_error", "level": "error", "text": "...", "turn": N }`.

### Harness architecture

```
G8.1: Start memd-web dev server (port 3000)
G8.2: Launch agent-browser with operator session ID
G8.3: Execute TE proof scenario (§4.1) → log metrics
G8.4: Reset operator session (logout/new profile)
G8.5: Execute TP proof scenario (§4.2) → log metrics
G8.6: Check proof NDJSON for zero console errors
G8.7: Screenshot both TE and TP UI states
G8.8: Regenerate MEMD-10-STAR.md from proof metrics
```

### 4.1 TE proof scenario (cost ledger visible + tunable)

Headless browser interaction:

```
T1: Navigate to http://localhost:3000/operator/cost-ledger
T2: Assert DOM: <h1>Token Budget</h1> visible, cap display shows "4000" (default)
T3: Click <button id="edit-budget">
T4: Fill <input id="budget-cap"> with "2000", submit
T5: Assert DOM: cap display updates to "2000", no console.error
T6: Execute turn 1 in sidecar (memd wake + read file)
T7: Assert DOM: turn 1 burn tracked, displayed as "1200 / 2000"
T8: Execute turn 2 (read different file)
T9: Assert DOM: turn 2 burn added, total "2150 / 2000" (exceeds cap)
T10: Check metrics in proof NDJSON: cost_ledger_visible=true, budget_tunable=true, budget_edit_turns >= 1
```

Screenshot: cost-ledger-ui-before-and-after-budget-edit.png

### 4.2 TP proof scenario (provenance depth 3+ drilldown)

Headless browser interaction:

```
T1: Navigate to http://localhost:3000/operator/provenance-browser
T2: Assert DOM: memory facts listed (e.g., "primary ID: ulid")
T3: Click on fact "primary ID: ulid"
T4: Assert DOM depth 1: metadata panel shows "confidence: 0.95, extraction_method: pattern, source_turn: 3"
T5: Click <button id="show-source-turn">
T6: Assert DOM depth 2: source turn T3 transcript visible, shows "correction: ULID not UUID"
T7: Click <button id="show-correction-history">
T8: Assert DOM depth 3: all corrections citing this fact listed (T3 correction + any later affirming corrections)
T9: Also at depth 3: alternate candidates shown (other extractions from T3 not chosen)
T10: Check metrics: provenance_depth_max >= 3, drilldown_clicks >= 1
```

Screenshot: provenance-browser-depth3-with-history-and-alts.png

### 4.3 Console error check

Harness must fail if any of these appears during harness execution:

- `window.onerror` listener fires with non-null error
- Unhandled promise rejection detected by `window.onunhandledrejection`
- `console.error(...)` called by app code (note: `console.warn` allowed)
- HTTP response status >= 500 from any XHR/fetch

Logged to proof NDJSON:

```json
{ "type": "console_error", "turn": 5, "level": "error", "source": "fetch", "status": 500, "url": "/api/provenance", "text": "Internal server error" }
```

Zero console errors is **mandatory** for axis credit. If any error, harness exits with failure, axis does not lift.

## 5. Scorecard regenerator strict-mode

G8's scorecard regenerator checks:

1. **TE score**: must be ≤ 5 (target). Fails if harness evidence shows cost_ledger_visible=false or budget_tunable=false.
2. **TP score**: must be ≤ 6 (target). Fails if provenance_depth_max < 3.
3. **No over-claim**: if TE harness fails, regenerator writes TE=4 (prior) and stops (does not attempt TP).
4. **SC preserved**: if A8–F8 read SC but make no SC changes, SC stays 5 (V7 post).

Rules:

- Never score an axis higher than harness evidence supports.
- If an axis has no V8 work, preserve its prior score verbatim (copy from V7 post).
- Always link to the proof-run NDJSON (dated).
- Append a one-line delta history entry so prior scorecards are reconstructible.

## 6. TE tightest-margin risk tracking

**Critical reminder:** TE margin is +2 at release (final 5, floor 3). V8 owns +1. If V8 misses TE or V4 fails to deliver +2, TE = 4 and release is blocked (floor violation).

Track this risk in every harness run:

```json
{
  "run_id": "G8-TE-proof-run-2026-04-23",
  "axis": "token_efficiency",
  "pre_v7": 4,
  "target_v8": 5,
  "final_v10": 5,
  "floor_3": 3,
  "margin": 2,
  "margin_owner_v4": 2,
  "margin_owner_v8": 1,
  "risk_assessment": "V8 must deliver +1. If V4 under-delivers (stays at 3 instead of 4), V8 +1 alone = 4 (floor violation). Every V8 commit must include cost-ledger proof.",
  "console_errors": 0,
  "cost_ledger_visible": true,
  "budget_tunable": true
}
```

If any harness run shows `cost_ledger_visible=false` or `budget_tunable=false`, file a blocker backlog item immediately.

## 6a. `memd configure` CLI (G8 sub-phase) — canonical settings surface

G8 ships the first and only canonical runtime-settings CLI. Every runtime toggle the codebase cares about routes through this surface. No axis credit; foundational.

### Surface

```
memd configure list                       # print all keys + current values + defaults
memd configure get <key>                  # single-key read, exit 0 if set, 2 if unknown
memd configure set <key>=<value>          # validate against schema, write .memd/config.json
memd configure reset [<key>]              # restore default (all if <key> omitted)
memd configure show-schema                # emit JSONSchema for config keys (machine-readable)
```

### Storage

- File: `.memd/config.json` (schema version `0.3+`).
- Atomic write via the V7 H7 writer-guard (config changes themselves are dirty-tracked; auto-commit applies unless the caller disables it).
- Back-compat: existing older keys in `.memd/config.json` carried forward; unknown keys produce a warning (not an error) on `list`, a hard error on `set`.

### Initial key surface (V8 close)

| Key | Type | Default | Owner phase | Notes |
| --- | --- | --- | --- | --- |
| `auto_commit.enabled` | bool | `true` | V7 H7 | atomic-commit primitive master toggle |
| `auto_commit.exclude_paths` | string[] | `[]` | V7 H7 | paths auto-commit ignores (e.g. build artefacts) |
| `cost_ledger.budget_tokens` | int | `4000` | V8 E8 | default wake budget cap; operator-tunable via UI or CLI |
| `cost_ledger.per_turn_warn` | int | `1500` | V8 E8 | warn threshold |
| `provenance.drilldown_depth_max` | int | `3` | V8 D8 | depth contract for D8 browser |
| `voice.mode` | string (enum) | `caveman-lite` | prior | voice surface; validated against voice registry |

### Reserved keys (future milestones)

- `federated.visibility.default` (V9 federated-memory defaults; schema stub ships in V8 G8, activation in V9)
- `compiler.mode` (V11 dynamic-compiler toggle; stub ships V8, activation in V11)
- `protocol.mcp.enabled` / `protocol.acp.enabled` / `protocol.custom.enabled` (V12)
- `provenance.export.signed` (V12 cryptographic provenance)

Schema stubs land in V8 G8 so later milestones only need to flip `reserved: true → reserved: false` + implement. No flag-day migrations downstream.

### Validation rules (strict)

1. Unknown key on `set` → error: `unknown key '<k>' — did you mean '<closest>'?` (Levenshtein ≤ 2 match).
2. Type mismatch → error with expected type from schema.
3. Enum out of range → error listing valid values.
4. Reserved key `set` attempt → error: `key reserved for future milestone, not yet active`.
5. Write to read-only key → error (e.g., schema_version).

### TAB-completion

- `memd completions zsh` / `bash` emit shell completion script.
- Completion enumerates keys from `memd configure show-schema`; no hard-coded key list.

### `memd configure` integration with other V8 phases

| Phase | Integration | How |
| --- | --- | --- |
| A8 atlas UI | — | no config consumption; pure UI |
| B8 correction UX | — | reads V7 correction tables; no V8 config |
| C8 memory inspector | — | — |
| D8 provenance browser | `provenance.drilldown_depth_max` | UI reads this key; if operator sets to 5, UI must honor |
| E8 cost ledger | `cost_ledger.budget_tokens`, `cost_ledger.per_turn_warn` | UI edit routes through `memd configure set cost_ledger.budget_tokens=<n>`; no duplicate write path |
| F8 leaderboard | — | — |
| G8 harness | validates schema stability | G8.schema-lock: snapshot schema hash, compare on every commit; drift = blocker |

Every other "setting" in the codebase (env vars, hard-coded defaults, ad-hoc prefs in V4–V7) is either:
- **(a) deprecated** in V8 G8 with a shim that reads the new key, OR
- **(b) explicitly out-of-scope** (one-off session flags like `MEMD_DEBUG_*` stay env-only).

No parallel "settings" system ships. `memd configure` is **the** surface. This is a hard rule — violations block V8 close.

### G8 configure sub-phase harness assertion

G8 harness adds one non-UI assertion block (parallel to TE + TP):

```
G8.CFG.1: memd configure list prints all 6 V8 keys + defaults
G8.CFG.2: memd configure set cost_ledger.budget_tokens=2000 → writes .memd/config.json
G8.CFG.3: memd configure get cost_ledger.budget_tokens → "2000"
G8.CFG.4: memd wake --output .memd respects new budget (reads config, not env)
G8.CFG.5: memd configure set unknown.key=1 → exits 2, prints "did you mean" hint
G8.CFG.6: memd configure reset cost_ledger.budget_tokens → defaults restored
G8.CFG.7: schema hash unchanged vs snapshot (no drift)
```

Metric logged to G8 proof NDJSON: `{ "type": "configure_suite", "pass_count": 7, "fail_count": 0 }`. Any fail → V8 does not close.

## 7. Feature-flag graduation calendar

Flag-flip ordering (each flip = its own commit, each after a 7-day clean window):

1. `MEMD_A8_ATLAS_UI` = 1 (Task A8.8)
2. `MEMD_B8_CORRECTION_UX` = 1 (Task B8.8)
3. `MEMD_C8_MEMORY_INSPECTOR` = 1 (Task C8.7)
4. `MEMD_D8_PROVENANCE_BROWSER` = 1 (Task D8.8)
5. `MEMD_E8_COST_LEDGER` = 1 (Task E8.8) — **TE axis credit block: do not flip until G8 TE proof lands**
6. `MEMD_F8_TRANSPARENCY_PAGE` = 1 (Task F8.7)

**Critical ordering:** E8 flag-flip must not happen until G8 TE harness passes and writes to MEMD-10-STAR.md with TE=5 confirmed. If E8 ships but TE stays 4, the feature is live but axis credit is lost.

**Calendar spillover:** 6 graduations × 7-day clean window = 42 days post-G8. V8 code-complete and G8 harness pass are the milestone-close bar; flag-graduation runs into the V9 planning window. V9 planning must account for the flag-ops work, but V9 phase A9 is **not** blocked on graduation completion — only on the handoff commit from V8's last code phase.

G8 runs with all flags at production defaults (on). A graduation rollback does not re-open V8 — file a recovery phase instead.

## 8. Browser testing checkpoint (mandatory for UI axes)

Every phase with UI work (A8, B8, C8, D8, E8, F8) must include:

- **Dev server running** at checkpoint
- **agent-browser** or Playwright session exercising the feature
- **Zero console errors** in harness logs
- **Screenshot** captured for review

This is not optional. Missing browser test proof → phase does not pass gate → V8 does not close.

Checkpoints:

- Post-A8.5: atlas navigation interactive (click node → provenance visible)
- Post-B8.6: correction capture works, preview updates live
- Post-C8.6: memory inspector searchable, filters work
- Post-D8.7: provenance drilldown depth 3+ proven (agent-browser turn by turn)
- Post-E8.7: cost ledger displays + operator can edit budget (agent-browser headless)
- Post-F8.6: leaderboard page loads, retraction log visible, no console errors

## 9. Commit strategy

### Plan-spec land phase (this task)

Eight atomic commits on `research/mining`, one per file + integration doc:

1. `docs(v8): phase-a8-plan implementation spec`
2. `docs(v8): phase-b8-plan implementation spec`
3. `docs(v8): phase-c8-plan implementation spec`
4. `docs(v8): phase-d8-plan implementation spec`
5. `docs(v8): phase-e8-plan implementation spec`
6. `docs(v8): phase-f8-plan implementation spec`
7. `docs(v8): phase-g8-plan implementation spec (memd configure CLI + release harness)`
8. `docs(v8): V8-INTEGRATION cross-phase plan`

### Execution commits per phase

Each phase plan has its own internal task list that commits per task. Those execution commits are produced by future agents — **not** by the plan-spec-land task. The spec-land task produces only the phase docs + 1 integration commit + 1 handoff commit.

### Handoff commit

After the 7 docs commits, one more commit:

```
docs(handoff): V8 plan specs landed, next agent executes A8
```

Content: new file `docs/handoff/YYYY-MM-DD-v8-plan-spec-complete-next-execute.md`.

## 10. Cross-phase API surface summary (UI-specific extensions)

| Introduced in | Symbol / Path | Consumed by | V8 note |
| --- | --- | --- | --- |
| A8 | `memd_web::atlas::*` UI components + SC read API | B8–F8 (all read atlas foundation) | reads session_continuity ledger (integration, no credit) |
| D8 | `memd_web::provenance::drilldown::*` API (depth contract) | E8 (error handling), F8 (transparency page links), G8 | **TP axis credit depends on depth ≥ 3** |
| E8 | `memd_web::cost_ledger::*` UI + budget mutation API | F8 (leaderboard budget disclosures), G8 | **TE axis credit depends on visible + tunable** |
| G8 | `docs/verification/v8-proof-runs/*.ndjson` (TE + TP + console logs) | V9 entry gate | must include cost_ledger_visible, budget_tunable, provenance_depth_max, console_error_count |

## 11. Exit criteria for V8 as a milestone

All six phase exit criteria met (A8–F8) AND G8 exit criteria met AND:

- 10-STAR composite ≥ 5.10 written to `docs/verification/MEMD-10-STAR.md` by the G8 scorecard regenerator.
- TE score = 5 (cost ledger visible + tunable; budget cap edit proven by agent-browser harness).
- TP score = 6 (provenance drilldown depth 3+ proven; alternate candidates visible; correction history included).
- SC score = 5 (unchanged; V7 owns; V8 integrates read-only).
- No axis score above the targets in MILESTONE-v8.md (regenerator fails loud on over-claim).
- All five A8–E8 phase screenshots included in G8 proof bundle (zero console errors in each).
- `docs/verification/milestones/MILESTONE-v8.md` filled in with evidence paths.
- `ROADMAP.md` V8 → closed, V9 → in progress.
- No open backlog items tagged `axis: token_efficiency` or `axis: trust_provenance` at severity `blocker`.
- G8 harness proof NDJSON includes metrics: `cost_ledger_visible`, `budget_tunable`, `provenance_depth_max`, `console_error_count=0`, `configure_suite.pass_count=7`, `configure_suite.fail_count=0`.
- `memd configure` CLI ships with all 6 V8 keys + reserved stubs for V9/V11/V12; schema hash recorded in G8 proof bundle; no parallel settings system exists in the codebase (grep audit for `env::var`, ad-hoc prefs, etc. passes).
- Stranger review write-up + 5 side-by-side screencasts (TE budget UI + TP drilldown + atlas nav + correction UX + leaderboard) committed.
- **TE margin risk assessment** documented and signed off (V4 delivered TE +2, V8 delivered TE +1, margin +2 held).
- E8 cost-ledger flag-flip blocked until post-G8 TE proof regeneration (not flipped during spec-land or execution, only after 7-day clean window post-G8).
- Final handoff doc points at `docs/phases/v9/` (to be created in the V9 plan-spec phase).

## 12. Changelog

- 2026-04-22 initial spec.
