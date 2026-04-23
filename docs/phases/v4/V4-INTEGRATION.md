---
version: v4
kind: integration-plan
status: ready-to-execute
opened: 2026-04-22
revised: 2026-04-22
scope: A4..G4 (F4 includes F4.7 seed)
depends_on: [../../verification/0.1.0-CONTRACT.md, ../../verification/milestones/MILESTONE-v4.md]
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

### Session 2 — "continue" on **codex harness** (10 turns — cross-harness flip)

Session 2 runs on **codex**, not claude-code. Same workspace, same user,
different harness preset. This is what makes `cross_harness 2→3` a real
axis lift and not a phantom one (per 0.1.0-CONTRACT.md "no axis credit
without harness proof").

| Turn | Actor | Utterance | Phases |
| --- | --- | --- | --- |
| — | system | Wake via D4 compiler in codex preset | D4 + F4 + A4 (PostCompact restore before T11) + **CH**: beliefs from claude-code S1 observable |
| T11 | agent | Read `migrations/0002.sql` | A4 |
| T12 | user | "What's the primary ID?" → `memd lookup --query "primary ID"` | E4 + C4 + **CH flip**: must answer "ulid" (correction T3 was issued in claude-code) |
| T13 | agent | Touch file from session 1 | A4 (cross-harness ledger handoff) |
| T14 | agent | Touch another | A4 |
| T15 | agent | Touch another | A4 |
| T16 | user | "How are we doing?" | D4 (wake brief recall) |
| T16.5 | system | **F4.7 assertion**: `routine_candidates_observed` metric incremented ≥1 from T13-T15 file-touch pattern (instrumentation only, no behavior) | F4.7 |
| T17 | user | "Summarize what you found." | — |
| T18 | user | "No, migration deadline is 2026-05-15." | C4 (second correction, codex-originated) |
| T19 | agent | (seeded) verbose reply | F4 (drift trigger) |
| T20 | system | SessionStop → PreCompact | A4 + B4 |

Cut 2 state snapshot assertions — see G4 plan §4 Cut 2.

### Session 3 — "honor" on **claude-code** (5 turns — round-trip)

Session 3 returns to claude-code (original harness). This proves
corrections made in codex (S2) survive the cross-harness round-trip back
to claude-code without re-export or manual sync.

| Turn | Actor | Utterance | Phases |
| --- | --- | --- | --- |
| — | system | Wake (should include drift surface + both corrections, both harnesses) | D4 + F4 + CH round-trip |
| T21 | user | "What's the migration deadline?" | E4 + C4 (must answer 2026-05-15 — **correction issued in codex must hold in claude-code**) |
| T22 | user | "And the primary ID?" | E4 + C4 (must answer ulid) |
| T23 | user | `memd preference confirm P1` | F4 |
| T24 | agent | terse reply | F4 |
| T25 | system | SessionStop | B4 |
| T25.5 | system | **F4.7 assertion**: metric `routine_candidates_observed` total across 3 sessions ≥ 3 (instrumentation live, zero behavior credit) | F4.7 |

Cut 3 assertions + scorecard regeneration — see G4 plan §4 Cut 3.

### Scenario self-tests (harness fault-injection)

G4 Task G4.3 includes fault-injection variants: skip A4 restore → assert cut 2 wake fails; inject B4 silent swallow → assert harness surfaces it; drop C4 provenance → assert C4 assertion fires. These are negative controls that prove the harness itself is honest.

## 5. 10-STAR scorecard regeneration template

G4 Task G4.4 writes to `docs/verification/MEMD-10-STAR.md`. The regenerator replaces the composite scorecard table in place and appends a dated evidence block.

Target table post-V4 (aligned to MILESTONE-v4.md axis contract — harness
produces actual numbers and the regenerator fails if any score exceeds
this table):

```markdown
## 10-Star Composite Scorecard

| Axis | Weight | Score | Status |
|------|--------|-------|--------|
| Session continuity | 20% | 4/10 | A4 ledger survival + B4 enforced hooks + D4 compiler |
| Correction retention | 15% | 4/10 | C4 E2E + F4 preference drift; 7d dogfood precision ≥0.85 |
| Procedural reuse | 15% | 2/10 | F4.7 seed — instrumentation live, no behavior credit |
| Cross-harness continuity | 15% | 3/10 | G4 cross-harness flip: correction in claude-code observable in codex |
| Raw retrieval strength | 15% | 4/10 | unchanged — V5 substrate bench target |
| Token efficiency | 10% | 4/10 | D4 compiler + E4 depth; wake median ≤2000 tokens, cost measured |
| Trust + provenance | 10% | 3/10 | B4 trace + C4 provenance visible; drilldown still partial |

**Composite: 3.45 (V4 gate requirement) — regenerated YYYY-MM-DD by G4 harness run <id>**

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
5. `MEMD_F4_PREF_DRIFT` = 1 (Task F4.8)   — F4.7 seed ships flag-off always
6. E4 flags ship on by default; no graduation needed.

**Calendar spillover:** 5 graduations × 7-day clean window = 35 days of
post-G4 observation. V4 code-complete and G4 harness pass are the
milestone-close bar; flag-graduation runs into the V5 planning window.
V5 planning must account for the flag-ops work, but V5 phase A5 is **not**
blocked on graduation completion — only on the handoff commit from V4's
last code phase.

F4.7 instrumentation ships always-on (flag-off would mean metric never
populates; defeats the measurement). Instrumentation is zero-cost, zero-
behavior, so no graduation needed.

G4 runs with all flags at production defaults (on). A graduation rollback
does not re-open V4 — file a recovery phase instead. If a flag flip
surfaces a regression during the 7-day window, the recovery phase targets
the specific axis that regressed and is named `v4-recovery-<axis>-<date>`.

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

## 11. Schema + ordering locks (V4 substrate plumbing)

Three schema-level locks land in V4 to unblock multi-harness and multi-
agent use-cases in V5+. These are **structural**, not axis-credit-bearing.
All three are A4 scope (read-state plumbing).

### 11.1 Lamport vector clock per memory row

Every write to `memory_items` stamps a `(node_id, sequence)` Lamport pair.
`node_id` is derived from harness preset + agent id; `sequence` is monotonic
per node. Two writes to the same claim from different harnesses are ordered
by Lamport rule (max node_seen + 1), breaking ties by canonical node_id sort.

Columns added (A4 Task A4.2 migration):
- `memory_items.lamport_node_id` TEXT NOT NULL
- `memory_items.lamport_seq`     INTEGER NOT NULL
- `memory_items.lamport_vector`  TEXT (JSON-encoded observed-clock snapshot)

Rationale: prevents the "codex supersede accidentally overwritten by stale
claude-code write" class of bug in the V4 cross-harness flip scenario.
Donor: Omegon (Rust reference implementation of Lamport-versioned memory).

### 11.2 Sequence-based session isolation

Each `memory_items` row carries `session_seq` (monotonic per session). All
read paths accept a `cutoff_seq` parameter to replay-past-a-point for
debugging and to prevent "future state" from leaking into reconstruction.

Column added (A4 Task A4.3):
- `memory_items.session_seq` INTEGER NOT NULL (monotonic per session_id)

Rationale: debugging a corruption post-compaction without `cutoff_seq` means
manually filtering by timestamp, which is lossy. Donor: Smriti sequence
isolation.

### 11.3 Content-hash deduplication

Every `memory_items` row carries a normalized content hash. Dedup rule:
normalize (trim + collapse whitespace + lowercase) → SHA256 → first 16 hex
chars. Insert-time uniqueness index on `(agent_id, content_hash)` prevents
same-agent dup; cross-agent dup falls through but gets co-attribution in
`memory_item_co_authors` (new table, D4 scope).

Column added (A4 Task A4.2):
- `memory_items.content_hash` TEXT NOT NULL

Rationale: the 3-session scenario has T2/T3 correction that could double-
insert under noisy hook retries. Content hash makes dedup correctness a
constraint not a best-effort. Donor: Omegon + mempalace.

### 11.4 Why V4 owns these

All three are read-state plumbing. A4 is the "read state across compaction"
phase; plumbing lives where the schema migrations land. Later phases
consume the new columns without owning the migration, which keeps phase
boundaries clean.

## 12. Exit criteria for V4 as a milestone

All seven phase exit criteria met AND G4 exit criteria met AND:

- 10-STAR composite ≥ 3.45 written to `docs/verification/MEMD-10-STAR.md` by the G4 scorecard regenerator.
- No axis score above the targets in MILESTONE-v4.md (regenerator fails loud on over-claim).
- `docs/verification/milestones/MILESTONE-v4.md` filled in with evidence paths.
- `ROADMAP.md` V4 → closed, V5 → in progress.
- No open backlog items tagged `axis: session_continuity`, `axis: correction_retention`, or `axis: cross_harness` at severity `blocker`.
- Cross-harness flip test passing (claude-code S1 → codex S2 → claude-code S3 round-trip).
- F4.7 metric `routine_candidates_observed` populated non-zero (instrumentation proven live, no behavior credit claimed).
- Schema locks 11.1–11.3 landed and covered by unit tests.
- `docs/contracts/federated-memory-visibility.md` referenced from V5 A5 kickoff (V9 enforcement precondition visible).
- Final handoff doc points at `docs/phases/v5/` (to be created in the V5 plan-spec phase).

## 13. Changelog

- 2026-04-22 initial spec.
- 2026-04-22 revision:
  - Session 2 switched from claude-code to codex (real cross-harness flip).
  - F4.7 seed added inside F4 (procedural instrumentation, no axis credit).
  - Scorecard template aligned to MILESTONE-v4 post-axis targets (3.45 not 4.0).
  - Flag-calendar spillover note added (5 × 7-day runs into V5 window).
  - Schema locks section added (Lamport, sequence isolation, content hash)
    — all A4 scope structural plumbing.
  - Cross-harness, F4.7, and schema-lock exit criteria added.
