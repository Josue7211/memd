---
version: v4
kind: integration-plan
status: ready-to-execute
opened: 2026-04-22
scope: A4..G4
---

# V4 Integration — Cross-Phase Plan

> Read after all seven `phase-{a4..g4}-plan.md` specs. This doc covers what no single phase plan owns: shared fixtures, hook-contract diff vs current layout, the dogfood scenario G4 executes, the 10-STAR regeneration ritual, and the commit strategy for the spec-land phase itself.

## 1. Execution-order discipline

Phase-level dependency (strict):

```
A4 ──► B4 ──► C4 ──┐
              │    │
              └► D4 ──► E4 ──┐
                 │            │
                 └──► F4 ─────┤
                              │
                              ▼
                              G4
```

Rules:

- A4 tasks 1–6 land first (ledger + contract). B4 cannot start until A4 Task A4.6 (handoff contract doc) commits.
- C4 and D4 parallelize after B4 lands Task B4.6 (universal trace).
- E4 requires D4 wake compiler. F4 requires C4 Correction kind + judge client.
- G4 requires everything.

No phase may short-circuit a prior dependency to hit its own pass gate. If blocked, file a backlog item and surface in the next session's handoff.

## 2. Shared test fixtures

To avoid fixture drift across phases, the following live under a shared dir and are referenced by multiple phase plans:

| Fixture | Owner | Shared with |
| --- | --- | --- |
| `crates/memd-client/fixtures/shared/sessions/session-1.jsonl` | G4 | A4 (compaction survival), D4 (20-fixture set includes session-1 anonymized), C4 (turn 2/5 corrections) |
| `crates/memd-client/fixtures/shared/preferences/prefs-5.jsonl` | F4 | G4 proof harness, D4 continuity test |
| `crates/memd-client/fixtures/shared/transcripts/aligned-10turn.jsonl` | F4 | G4 |
| `crates/memd-client/fixtures/shared/transcripts/drift-10turn.jsonl` | F4 | G4 |
| `crates/memd-client/fixtures/shared/hook-traces/canonical-trace.ndjson` | B4 | G4, A4 doctor check |

Convention: each phase plan's `fixtures/<phase>/` dir contains **only** fixtures unique to that phase. Shared fixtures move to `fixtures/shared/` the moment a second phase references them, with a compat shim in the original dir (symlink or `pub use`).

First consolidation happens after C4 lands — harvest A4's ledger fixture + C4's turn transcript into shared.

## 3. Hook-contract diff — `.memd/hooks/` layout

### Current (post V3 K3)

```
.memd/hooks/
├── MANIFEST.json             # contract_version implicit 0.2
├── README.md                 # canonical source
├── install.{sh,ps1}
├── memd-bootstrap.sh
├── memd-capture.{sh,ps1}
├── memd-context.{sh,ps1}
├── memd-file-interaction.sh
├── memd-lifecycle-probe.sh
├── memd-precompact-save.{sh,ps1}   # seals ledger
├── memd-preedit-prime.sh
├── memd-pretool-gate.sh
├── memd-spill.{sh,ps1}
└── memd-stop-save.{sh,ps1}
```

### Post V4

```
.memd/hooks/
├── MANIFEST.json             # contract_version: "0.3"
├── README.md                 # updated section "PostCompact restore"
├── install.{sh,ps1}
├── memd-bootstrap.sh
├── memd-capture.{sh,ps1}                # accepts --kind correction, provenance flags (C4)
├── memd-context.{sh,ps1}
├── memd-file-interaction.sh
├── memd-lifecycle-probe.sh
├── memd-precompact-save.{sh,ps1}
├── memd-postcompact-restore.{sh,ps1}    # NEW (A4)
├── memd-preedit-prime.sh
├── memd-pretool-gate.sh
├── memd-spill.{sh,ps1}
└── memd-stop-save.{sh,ps1}
```

Differences:
- `memd-postcompact-restore.{sh,ps1}` added (A4 Task A4.4).
- Every inner `memd hook …` call routed through `memd hooks enforce --event <NAME> --budget-ms <N> --` when `MEMD_HOOK_ENFORCE=1` (B4 Task B4.9).
- MANIFEST.json bumps `contract_version` 0.2 → 0.3; new entries for restore hook.
- `memd-capture.{sh,ps1}` passes `--kind`, `--corrects-id`, `--source-turn` through to `memd hook capture`.

Harness-side diff (claude-code `~/.claude/settings.json`, codex `~/.codex/hooks.json`): PostCompact event must register `memd-postcompact-restore.sh`. The autosync path (`scripts/sync-integration-hooks.sh`) re-emits the harness configs from `.memd/hooks/MANIFEST.json`; do not hand-edit harness configs.

Note: `docs/backlog/v3/2026-04-22-harness-bridges-report-inverted.md` flags that `docs/HARNESS_BRIDGES.md` currently inverts reality. Ignore that doc during V4 work; read `~/.claude/settings.json` and `~/.codex/hooks.json` directly.

## 4. 3-session dogfood scenario G4 executes

Full canonical script; each turn tagged with the V4 phase it exercises.

### Session 1 — "establish" (10 turns)

| Turn | Actor | Utterance | Phases exercised |
| --- | --- | --- | --- |
| T1 | user | "We use Postgres for the ledger." | C4 (fact ingestion) |
| T2 | user | "The primary ID is uuid." | C4 (prior claim for correction) |
| T3 | user | "No wait, actually the primary ID is ulid." | C4 (correction detector, judge confirm) |
| T4 | user | "Prefer terse replies." | F4 (preference P1) |
| T5 | user | "No emojis ever." | F4 (preference P2) |
| T6 | user | "Focus: finish the Q1 migration." | D4 (focus bucket) |
| T7 | user | "The migration deadline is 2026-05-01." | C4 |
| T8 | agent | Read `src/ledger.rs` | A4 (ledger populates) |
| T9 | agent | Read `migrations/0001.sql` + `src/lib.rs` | A4 |
| T10 | system | SessionStop → PreCompact | A4 seal, B4 trace |

Cut 1 state snapshot assertions — see `phase-g4-plan.md` §4 Cut 1.

### Session 2 — "continue" (10 turns)

| Turn | Actor | Utterance | Phases |
| --- | --- | --- | --- |
| — | system | Wake via D4 compiler | D4 + F4 + A4 (PostCompact restore before T11) |
| T11 | agent | Read `migrations/0002.sql` | A4 |
| T12 | user | "What's the primary ID?" → `memd lookup --query "primary ID"` | E4 + C4 |
| T13 | agent | Touch file from session 1 | A4 |
| T14 | agent | Touch another | A4 |
| T15 | agent | Touch another | A4 |
| T16 | user | "How are we doing?" | D4 (wake brief recall) |
| T17 | user | "Summarize what you found." | — |
| T18 | user | "No, migration deadline is 2026-05-15." | C4 (second correction) |
| T19 | agent | (seeded) verbose reply | F4 (drift trigger) |
| T20 | system | SessionStop → PreCompact | A4 + B4 |

Cut 2 state snapshot assertions — see G4 plan §4 Cut 2.

### Session 3 — "honor" (5 turns)

| Turn | Actor | Utterance | Phases |
| --- | --- | --- | --- |
| — | system | Wake (should include drift surface + corrections) | D4 + F4 |
| T21 | user | "What's the migration deadline?" | E4 + C4 (must answer 2026-05-15) |
| T22 | user | "And the primary ID?" | E4 + C4 (must answer ulid) |
| T23 | user | `memd preference confirm P1` | F4 |
| T24 | agent | terse reply | F4 |
| T25 | system | SessionStop | B4 |

Cut 3 assertions + scorecard regeneration — see G4 plan §4 Cut 3.

### Scenario self-tests (harness fault-injection)

G4 Task G4.3 includes fault-injection variants: skip A4 restore → assert cut 2 wake fails; inject B4 silent swallow → assert harness surfaces it; drop C4 provenance → assert C4 assertion fires. These are negative controls that prove the harness itself is honest.

## 5. 10-STAR scorecard regeneration template

G4 Task G4.4 writes to `docs/verification/MEMD-10-STAR.md`. The regenerator replaces the composite scorecard table in place and appends a dated evidence block.

Target table post-V4 (aspirational — actual numbers come from harness):

```markdown
## 10-Star Composite Scorecard

| Axis | Weight | Score | Status |
|------|--------|-------|--------|
| Session continuity | 20% | 4/10 | A4 ledger survival + B4 enforced hooks; G4 proof runs 10/10 |
| Correction retention | 15% | 4/10 | C4 E2E + F4 preference drift; 7d dogfood precision ≥0.85 |
| Procedural reuse | 15% | 1/10 | unchanged — V5+ scope |
| Cross-harness continuity | 15% | 4/10 | E4 depth contract + B4 trace normalized cross-harness |
| Raw retrieval strength | 15% | 4/10 | unchanged — V5 substrate bench target |
| Token efficiency | 10% | 7/10 | D4 compiler + E4 depth; wake median ≤2000 tokens |
| Trust + provenance | 10% | 6/10 | B4 trace + C4 provenance visible |

**Composite: ≥4.0 (V4 gate requirement) — regenerated YYYY-MM-DD by G4 harness run <id>**

Evidence: docs/verification/v4-proof-runs/YYYY-MM-DD.ndjson
```

Regeneration rules:
- Never score an axis higher than the harness evidence supports.
- If an axis has no V4 work, preserve its prior score verbatim.
- Always link to the proof-run NDJSON.
- Append a one-line delta history entry so prior scorecards are reconstructible.

## 6. Feature-flag graduation calendar

Flag-flip ordering (each flip = its own commit, each after a 7-day clean window):

1. `MEMD_A4_LEDGER_SURVIVAL` = 1 (Task A4.9)
2. `MEMD_HOOK_ENFORCE` = 1 (Task B4.10)
3. `MEMD_C4_CORRECTION_DETECT` = 1 (Task C4.9)
4. `MEMD_D4_COMPILER` = 1 (Task D4.8)
5. `MEMD_F4_PREF_DRIFT` = 1 (Task F4.7)
6. E4 flags ship on by default; no graduation needed.

G4 runs with all flags at production defaults. A graduation rollback does not re-open V4 — file a recovery phase instead.

## 7. Bench regression watch

V4 does not directly target public-bench numbers, but ingest-path changes in C4 can move LME / ConvoMem. Mandatory checkpoints:

- Post-C4 Task C4.6: run canonical regression suite on LME, ConvoMem, MemBench. Document delta in C4 plan's Task C4.8 precision-review doc.
- Post-D4 Task D4.8: same.
- Post-G4 Task G4.6: full public + substrate bench sweep; publish in MILESTONE-v4.md.

If any public bench regresses >3% canonical score, hold the flag flip and root-cause.

## 8. Commit strategy

### Plan-spec land phase (this task)

Eight atomic commits on `research/mining`, one per file:

1. `docs(v4): phase-a4-plan implementation spec`
2. `docs(v4): phase-b4-plan implementation spec`
3. `docs(v4): phase-c4-plan implementation spec`
4. `docs(v4): phase-d4-plan implementation spec`
5. `docs(v4): phase-e4-plan implementation spec`
6. `docs(v4): phase-f4-plan implementation spec`
7. `docs(v4): phase-g4-plan implementation spec`
8. `docs(v4): V4-INTEGRATION cross-phase plan`

### Execution commits per phase

Each phase plan has its own internal task list that commits per task (A4 = 9 commits, B4 = 11, etc.). Those execution commits are produced by future agents — **not** by the plan-spec-land task. The spec-land task produces only the 8 docs commits + 1 handoff commit.

### Handoff commit

After the 8 docs commits, one more commit:

```
docs(handoff): V4 plan specs landed, next agent executes A4
```

Content: new file `docs/handoff/YYYY-MM-DD-v4-plan-spec-complete-next-execute.md`.

## 9. Cross-phase API surface summary

| Introduced in | Symbol / Path | Consumed by |
| --- | --- | --- |
| A4 | `memd_core::file_ledger::restore::*` | B4 (enforce wraps it), D4 (ledger counts into wake budget), G4 |
| A4 | `docs/contracts/hook-handoff.md` | B4 (enforce reads handoff rules) |
| B4 | `memd_core::hook_runtime::*` | all |
| B4 | `.memd/logs/hook-trace.ndjson` | A4 doctor, C4, D4, F4, G4 |
| B4 | `docs/contracts/hook-order.md` | G4 (fire-order assertions) |
| C4 | `MemoryKind::Correction` + provenance fields | D4 (compiler priority), E4 (lookup filter), F4 (preference promote), G4 |
| C4 | `memd-core::correction::judge` client + cache | F4 (reuse client, shared budget), future V5/V6 |
| C4 | `.memd/logs/corrections.ndjson` | G4 |
| D4 | `runtime::resume::compiler::*` | E4 (wake depth), F4 (non-demotable bucket), G4 |
| D4 | `.memd/logs/wake-budget.ndjson` | G4 |
| E4 | `docs/contracts/recall-depth.md` | G4 |
| E4 | `.memd/logs/recall-depth.ndjson` | G4 |
| F4 | `.memd/logs/preference-drift.ndjson`, `.memd/state/preference-drift-outstanding.json` | G4 (cut 2 + cut 3 assertions) |
| G4 | `docs/verification/v4-proof-runs/*.ndjson` | V5 entry gate |

## 10. Open questions for next executor

Surface these in TodoWrite or phase kickoff notes; do not silently assume:

- Token counter for D4 budget — reuse `compute_wake_token_metrics` or reimplement? Investigate stability and tokenizer alignment with codex-lb.
- `wait_timeout` dep for B4 budget — check `cargo tree` before adding; prefer stdlib if Rust ≥ 1.76 provides equivalent.
- Fixture anonymizer for D4 — does `scripts/dev/` already hold a scrub script? Check before Task D4.7.
- CI substrate — `.github/workflows/` vs other CI (e.g., Forgejo, Woodpecker). Confirm at G4 Task G4.5.
- `MemoryKind` enum non-exhaustive annotation — may already be the case; verify before C4.1.

## 11. Exit criteria for V4 as a milestone

All seven phase exit criteria met AND G4 exit criteria met AND:

- 10-STAR composite ≥ 4.0 written to `docs/verification/MEMD-10-STAR.md` by the G4 scorecard regenerator.
- `docs/verification/milestones/MILESTONE-v4.md` filled in.
- `ROADMAP.md` V4 → closed, V5 → in progress.
- No open backlog items tagged `axis: session_continuity` or `axis: correction_retention` at severity `blocker`.
- Final handoff doc points at `docs/phases/v5/` (to be created in the V5 plan-spec phase).
