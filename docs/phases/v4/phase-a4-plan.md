---
phase: A4
name: Read-State Across Compaction
version: v4
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: []
phase_doc: docs/phases/v4/phase-a4-read-state-compaction.md
granularity: "one step = ≤1 agent session of work; each step has explicit acceptance criteria; TDD inside each step (red → green → commit)"
axis: session_continuity
axis_delta_target: "+1 (1 → 2 minimum)"
---

# Phase A4 — Implementation Plan

> Execute against `research/mining`. Rebuild with `cargo build --release --target-dir /tmp/memd-target -p memd-client -p memd-server`. Server runs under `MEMD_RATE_LIMIT_DISABLED=1` for dogfood.

## 0. Executive summary

A4 proves the file-interaction ledger survives the Claude-Code / Codex auto-compaction event, and that the post-compaction turn reloads it **before the first tool call**. Ledger storage already works (`memd-core/src/file_ledger.rs`: seal/load round-trip tested in `tests` module). What is missing:

1. A canonical **PostCompact reload** step — today `memd-precompact-save.sh` seals the ledger, but nothing on the other side binds the sealed ledger back into the session before tools fire.
2. A **hook-ordering contract** that the runtime can enforce and `memd hooks doctor` can verify.
3. A **breach telemetry line** so regressions are observable, not felt.
4. A **CI regression** that simulates the full compaction arc without a live claude-code harness.

Outcome: 10-STAR axis 1 (session continuity) moves 1 → 2 minimum. This is the floor A4 guarantees; E4/G4 push it further.

---

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `docs/contracts/hook-handoff.md` | Normative contract for PreCompact → PostCompact ledger handoff. Source of truth cited by `memd hooks doctor` messages. |
| `.memd/hooks/memd-postcompact-restore.sh` | PostCompact hook: locate newest sealed ledger, copy back to `ledger_path`, emit NDJSON restore record. Non-blocking on failure, but logs breach. |
| `.memd/hooks/memd-postcompact-restore.ps1` | Windows parity (same behavior, PowerShell). |
| `crates/memd-core/src/file_ledger/restore.rs` | Pure-Rust helpers: `locate_latest_sealed`, `restore_ledger`, `LedgerRestoreReport`. Tested at core level, used by CLI + hook runtime. |
| `crates/memd-client/src/cli/cli_hook_doctor.rs` | Dispatcher logic for `memd hook doctor --check ordering`. Reads a synthetic hook trace, asserts fire order matches `hook-handoff.md`. |
| `crates/memd-client/src/main_tests/continuity_compaction_tests/mod.rs` | CI scenario: synthesize PreCompact seal → compaction → PostCompact restore → prime-reads query. Reuses the existing `continuity_foundation_tests` helpers where possible. |
| `crates/memd-client/fixtures/a4/` | New fixture dir. Holds `pre-compact-ledger.json`, `post-compact-expected.json`, 5-file synthetic session transcript. |

### Files to modify

| Path | Change |
| --- | --- |
| `crates/memd-core/src/file_ledger.rs` | Re-export new `restore` submodule. No behavior change to existing fns. |
| `crates/memd-core/src/lib.rs` | `pub mod file_ledger;` already exists — add `pub use file_ledger::restore::*;` at appropriate scope. |
| `crates/memd-client/src/cli/args.rs` | Extend `HookMode` with `Restore(HookRestoreArgs)`. Extend `HookDoctorArgs` with `--check ordering` option. |
| `crates/memd-client/src/cli/cli_hook_runtime.rs` | Dispatch `HookMode::Restore` → `file_ledger::restore::restore_ledger`. |
| `crates/memd-client/src/cli/cli_hook_doctor.rs` (existing or mod.rs extension) | Wire new `--check ordering` branch. |
| `.memd/hooks/MANIFEST.json` | Add SHA256 + event for `memd-postcompact-restore.sh` / `.ps1`. `event: "PostCompact"`, `harness: "claude-code"` and a parallel `harness: "codex"` entry. |
| `.memd/hooks/README.md` | Document PostCompact restore + handoff contract link. |
| `integrations/hooks/` | Auto-synced via `scripts/sync-integration-hooks.sh` after MANIFEST update — not a manual edit. |
| `docs/phases/v4/phase-a4-read-state-compaction.md` | Add `plan_spec: docs/phases/v4/phase-a4-plan.md` line to frontmatter. |

### Crates affected

- `memd-core` — new `file_ledger::restore` module. Pure logic, no I/O surprises.
- `memd-client` — new CLI verb path, new hook doctor check, new tests.
- `memd-server` — no change.
- `memd-schema` — no change.

### Binaries rebuilt

`memd` (memd-client binary). `memd-server` unchanged.

---

## 2. Schema changes

A4 is **storage-layout additive only**. No wire-format breaks.

### New on-disk artifact

Restore record, one JSON-lines file: `.memd/logs/ledger-restore.ndjson`.

```json
{"ts_ms":1713811200000,"session_id":"sess-abc","sealed_path":".memd/state/session-sess-abc/sealed/1713811100000.json","restored_path":".memd/state/session-sess-abc/file_interactions.json","entries":12,"source":"postcompact-hook","ok":true}
```

One record per restore call. Append-only. No rotation in A4 (V7 owns log rotation).

### New breach log line

`.memd/logs/continuity-breach.log` — plain text, one line per incident:

```
2026-04-22T13:40:01Z sess-abc breach=tool-before-restore tool=Read path=src/foo.rs
```

### Backward-compat posture

- Existing sealed ledgers under `state/session-*/sealed/*.json` are directly restorable — no migration.
- If `MEMD_A4_LEDGER_SURVIVAL=0` the PostCompact hook no-ops; nothing about the on-disk layout changes.
- If the hook is missing (older install), the CLI still works; `memd hook doctor --check ordering` reports `no-postcompact-hook-installed` and exits 1.

---

## 3. API shape

### New CLI verbs

```
memd hook restore \
  --session-id <ID> \
  [--output .memd] \
  [--latest-only]          # default true
  [--dry-run]              # print what would be restored, no write
  [--json]                 # emit LedgerRestoreReport as JSON
```

Exit codes:
- `0` — restore succeeded or dry-run printed plan
- `2` — no sealed ledger found (non-fatal for caller, breach emitted)
- `3` — disk write failure (fatal, caller should alert)

### Extended existing CLI

```
memd hook doctor --check ordering [--trace <path>] [--json]
```

`--trace` defaults to `.memd/logs/hook-trace.ndjson` (written by B4 — until B4 lands, `ordering` check operates on a synthesized trace from `--trace-inline <json>` used by tests only).

A4 ships the flag plumbing and the `ordering` check body that reads whichever trace exists. If the trace is missing, exit 1 with `trace-unavailable` diagnostic.

### New hook contract (normative)

`docs/contracts/hook-handoff.md` codifies:

1. PreCompact writes sealed ledger via `memd hook seal-ledger` (already implemented).
2. PostCompact MUST run `memd hook restore` **before** any PreEdit / PreRead fires.
3. PostCompact hook exit code non-zero triggers a continuity-breach log line and a WARN-level message in `.memd/logs/hook-trace.ndjson` (B4 wires the latter; A4 only writes the breach line).
4. Hook consumers (claude-code, codex) install via `install.sh` / `install.ps1`; auto-install confirms presence.

### New hook contract record structure

```rust
pub struct LedgerRestoreReport {
    pub session_id: String,
    pub sealed_path: Option<PathBuf>,
    pub restored_path: PathBuf,
    pub entries: usize,
    pub source: RestoreSource,   // Postcompact, Manual, Test
    pub ok: bool,
    pub error: Option<String>,
}
```

Serialized to ndjson + returned from CLI with `--json`.

---

## 4. Test matrix

Granularity: every bullet is one test. Red-green-commit per bullet or per small group.

### Unit (crates/memd-core/src/file_ledger/restore.rs)

1. `locate_latest_sealed_returns_newest_by_timestamp` — write 3 sealed files with incrementing ms filenames, assert latest wins.
2. `locate_latest_sealed_returns_none_when_sealed_dir_missing` — absent dir → `None`, not error.
3. `locate_latest_sealed_ignores_non_json_files` — `latest.tmp` ignored.
4. `restore_ledger_copies_sealed_to_active_path` — populate sealed, restore, assert `ledger_path` content == sealed content.
5. `restore_ledger_overwrites_existing_active_ledger` — active ledger present, sealed newer, restore wins, no diff.
6. `restore_ledger_is_idempotent` — two restores same result, no duplicate entries.
7. `restore_ledger_records_source_postcompact` — report.source == `Postcompact` when constructed via hook path.

### Unit (crates/memd-client/src/cli/cli_hook_doctor.rs)

8. `ordering_check_passes_on_canonical_trace` — synthetic trace with correct fire order → ok.
9. `ordering_check_flags_tool_before_restore` — trace where PreEdit fires pre-restore → fail + specific diagnostic string.
10. `ordering_check_flags_missing_restore` — PostCompact event without subsequent restore → fail.
11. `ordering_check_requires_trace_file` — no trace + no `--trace-inline` → exit 1 `trace-unavailable`.

### Integration (crates/memd-client/src/main_tests/continuity_compaction_tests/mod.rs)

12. `seal_then_restore_round_trips_ledger_entries` — CLI `hook seal-ledger` then `hook restore`, assert entries survive byte-for-byte.
13. `restore_noops_when_no_sealed_ledger_present` — exit 2, breach-log line written.
14. `restore_emits_ndjson_record` — one valid JSON line appended.
15. `cli_dry_run_does_not_mutate_disk` — dry-run prints plan; `ledger_path` unchanged.
16. `hook_doctor_ordering_check_with_inline_trace_happy_path` — end-to-end CLI invocation.
17. `hook_doctor_ordering_check_reports_breach_line_count_on_existing_log` — surfaces breach count from breach log.

### E2E scenario (fixture-driven)

18. `a4_compaction_survival_5_files` — synthesize 5 Read hooks (populate ledger), call `hook seal-ledger`, wipe active ledger (simulate compaction reset), call `hook restore`, query via `memd lookup --depth lookup` (E4 placeholder: fallback to direct `memd hook restore --json` parsing for A4), assert all 5 paths retrievable.
19. `a4_compaction_breach_detection` — same as 18 but skip the restore step; assert breach line for the first simulated PreEdit.

### Rebuild + smoke

Every step that touches Rust must end with:

```
cargo build --release --target-dir /tmp/memd-target -p memd-client
cargo test --target-dir /tmp/memd-target -p memd-core file_ledger::
cargo test --target-dir /tmp/memd-target -p memd-client continuity_compaction
```

Acceptance: all green, zero new warnings in touched files.

---

## 5. Fixtures

New dir: `crates/memd-client/fixtures/a4/`. Fixture loader helper: `memd-client/src/main_tests/continuity_compaction_tests/fixtures.rs`.

| File | Contents | Regen command |
| --- | --- | --- |
| `pre-compact-ledger.json` | FileInteractionLedger with 5 synthetic Read entries for `src/a.rs..src/e.rs`. Ts monotonic. | `cargo run --bin memd -- --fixture-emit a4/pre-compact-ledger --target-dir /tmp/memd-target` (new fixture-emit helper added under `--dev-tools`; test-only, behind `cfg(feature = "dev-fixtures")` if desired — simplest: static JSON checked in, no emitter in A4). |
| `post-compact-expected.json` | Byte-equal copy of `pre-compact-ledger.json`. | `cp`. |
| `session-transcript.jsonl` | 20-line synthetic hook payload stream: 5× PreEdit-prime hits, 5× Read, 1× PreCompact, 1× PostCompact, 1× PreEdit on file #1 (expected to NOT breach). | Static checked-in. |
| `breach-transcript.jsonl` | Same as above minus the PostCompact restore hook. Test 19 consumes this. | Static checked-in. |

Decision: **static fixtures, no emitter in A4**. Reduces surface. V5 benches can refactor to an emitter if they need generation.

---

## 6. Telemetry

| Signal | Path | Emitter | Consumer |
| --- | --- | --- | --- |
| Restore record (success or no-op) | `.memd/logs/ledger-restore.ndjson` | `memd hook restore` | G4 CI asserts 1 restore per compaction. |
| Breach incident | `.memd/logs/continuity-breach.log` | `memd hook restore` when no sealed found + `cli_hook_doctor.rs` ordering check on trace | A4 pass gate (zero entries in 10-run test). |
| Hook trace (cross-phase) | `.memd/logs/hook-trace.ndjson` | B4 — not written by A4. | A4 doctor check reads if present. |

Counter names introduced (prometheus-shaped, exposed via `memd-server /metrics` in V7; A4 only emits as counters in log consumer, not wired to server):

- `memd_ledger_restore_total{result="ok|noop|fail"}`
- `memd_continuity_breach_total{kind="tool-before-restore|missing-restore|no-sealed-ledger"}`

A4 writes the log lines; V7 wires /metrics. No A4 work on /metrics path — **do not add**.

---

## 7. Feature flags

| Var | Default | Effect | Graduation |
| --- | --- | --- | --- |
| `MEMD_A4_LEDGER_SURVIVAL` | `0` during dogfood week 1; `1` after | Enables PostCompact restore hook body and doctor ordering check. When `0`, hook exits 0 immediately, doctor check returns `disabled`. | Flip to `1` after 7-day dogfood shows zero breach lines under normal use. |
| `MEMD_A4_BREACH_FATAL` | `0` | When `1`, breach promotes to non-zero hook exit (kills tool call). Off by default — observability first. | Flip to `1` only after operator UX is built (V8). |

Both read via `std::env::var` at hook runtime. No central registry required in A4; V6 consolidates flags.

---

## 8. Task list (executable)

Each task is one agent session. Each step inside a task is a TDD micro-cycle when Rust, else a doc / config edit. End every task with a commit on `research/mining`.

### Task A4.1 — `file_ledger::restore` module

- [ ] Read `crates/memd-core/src/file_ledger.rs` to confirm current `seal_session_ledger` path shape.
- [ ] Write failing test `locate_latest_sealed_returns_newest_by_timestamp` in new file `crates/memd-core/src/file_ledger/restore.rs`.
- [ ] Add `pub mod restore;` to `file_ledger.rs` (convert to directory module `file_ledger/mod.rs` + `restore.rs`, or keep `file_ledger.rs` and add sibling `file_ledger_restore.rs` — prefer dir module for clarity; check `grep -n "mod file_ledger" crates/memd-core/src/lib.rs` first).
- [ ] Implement `locate_latest_sealed(output: &Path, session_id: &str) -> Option<PathBuf>`. Green test.
- [ ] Add tests 2–3 (unit). Green.
- [ ] Implement `restore_ledger(session_id, output) -> io::Result<LedgerRestoreReport>`. Green tests 4–7.
- [ ] `cargo test --target-dir /tmp/memd-target -p memd-core file_ledger::` all green.
- [ ] Commit: `feat(memd-core/file_ledger): restore module locates and applies sealed ledger (A4)`.

Acceptance: unit tests 1–7 pass. No warnings in `memd-core`.

### Task A4.2 — `memd hook restore` CLI verb

- [ ] Read `crates/memd-client/src/cli/args.rs` lines 1660–1710 for `HookMode` variants to add `Restore(HookRestoreArgs)`.
- [ ] Add `HookRestoreArgs` struct with session-id, output, latest-only, dry-run, json flags.
- [ ] Write failing integration test `cli_hook_restore_round_trip` in `main_tests/continuity_compaction_tests/mod.rs`.
- [ ] Extend dispatcher in `cli_hook_runtime.rs` to call `file_ledger::restore::restore_ledger`.
- [ ] Implement ndjson append to `.memd/logs/ledger-restore.ndjson`.
- [ ] Green integration tests 12–15.
- [ ] Commit: `feat(memd-client/hook): add memd hook restore verb (A4)`.

Acceptance: integration tests 12–15 pass. `memd hook restore --help` lists flags.

### Task A4.3 — breach telemetry

- [ ] Write failing test `restore_noops_when_no_sealed_ledger_present` (test 13) if not already green — this verifies breach line format.
- [ ] In `file_ledger::restore`, when `locate_latest_sealed` returns `None`, open `.memd/logs/continuity-breach.log` append-only and write one line `{ts} {session_id} breach=no-sealed-ledger`.
- [ ] Add matching case for `tool-before-restore` fed by the doctor check (stub — actual emission in Task A4.5).
- [ ] Green.
- [ ] Commit: `feat(memd-core): breach log line for missing sealed ledger (A4)`.

Acceptance: test 13 passes; log file format matches §2.

### Task A4.4 — PostCompact hook scripts + manifest

- [ ] Read `.memd/hooks/memd-precompact-save.sh` for canonical style + error handling.
- [ ] Write `.memd/hooks/memd-postcompact-restore.sh` that:
  - reads session_id from payload (same helper pattern as memd-precompact-save)
  - calls `memd hook restore --session-id "$SID" --output "$MEMD_OUTPUT"` with `MEMD_A4_LEDGER_SURVIVAL` gate
  - non-blocking on non-zero exit, logs warning
- [ ] Mirror into `memd-postcompact-restore.ps1`.
- [ ] Update `.memd/hooks/MANIFEST.json` — add entries for both scripts (event PostCompact, harnesses claude-code + codex). Recompute SHA256.
- [ ] Run `scripts/sync-integration-hooks.sh` (or re-verify its trigger logic).
- [ ] Update `.memd/hooks/README.md` — new section "PostCompact restore".
- [ ] Manual smoke: bash script handles missing `memd` on PATH gracefully (exit 0 with warning).
- [ ] Commit: `feat(hooks): PostCompact restore hook + MANIFEST entries (A4)`.

Acceptance: `memd hook doctor` (existing, without `--check ordering`) still exits 0. MANIFEST validates.

### Task A4.5 — `memd hook doctor --check ordering`

- [ ] Read existing `HookDoctorArgs` at `crates/memd-client/src/cli/args.rs:1672`.
- [ ] Extend `HookDoctorArgs` with `check: Option<HookDoctorCheck>` (enum `Ordering`), `trace: Option<PathBuf>`, `trace_inline: Option<String>`.
- [ ] New file `crates/memd-client/src/cli/cli_hook_doctor.rs` (or extend existing) implementing `check_ordering(trace, breach_log) -> CheckReport`.
- [ ] Write failing tests 8–11.
- [ ] Implement trace parsing (expects NDJSON lines with fields `event`, `ts_ms`, optional `tool`). If trace missing → exit 1 diagnostic `trace-unavailable`.
- [ ] Green tests 8–11.
- [ ] Commit: `feat(memd-client/hook): doctor --check ordering (A4)`.

Acceptance: doctor tests pass. `memd hook doctor --check ordering --trace-inline '[]'` returns deterministic exit.

### Task A4.6 — `docs/contracts/hook-handoff.md`

- [ ] `mkdir -p docs/contracts`.
- [ ] Write `hook-handoff.md` covering: fire-order diagram, normative MUSTs, breach categories, observability outputs, flag behavior, examples of correct + incorrect traces.
- [ ] Link from `docs/phases/v4/phase-a4-read-state-compaction.md` + `README.md` phases section.
- [ ] Commit: `docs(contracts): hook-handoff contract for A4 (ledger survival across compaction)`.

Acceptance: doc passes markdownlint if configured. No broken internal links.

### Task A4.7 — E2E scenario + CI regression

- [ ] Add fixture files under `crates/memd-client/fixtures/a4/` per §5.
- [ ] Write `main_tests/continuity_compaction_tests/mod.rs` scenarios 18 + 19.
- [ ] Ensure scenario runs without network, without LLM, deterministic.
- [ ] Add `#[ignore]`-free variant that CI picks up. Tag with `cfg(test)` only — no dogfood dependency.
- [ ] `cargo test --target-dir /tmp/memd-target -p memd-client continuity_compaction -- --nocapture` green.
- [ ] Commit: `test(memd-client): A4 compaction-survival and breach-detection scenarios`.

Acceptance: scenarios 18 + 19 pass 10/10 local runs. No tempdir leakage.

### Task A4.8 — 10-STAR axis 1 rescoring

- [ ] Run the E2E 10 times via a wrapper script `scripts/verify/a4-loop.sh`. Record pass count.
- [ ] Read `docs/verification/MEMD-10-STAR.md`, bump axis 1 score with evidence pointer to the test and breach-log-empty artifact.
- [ ] Recompute composite.
- [ ] Commit: `docs(10-star): axis 1 rescored after A4 pass gate`.

Acceptance: composite moves ≥ +0.2 on the axis; evidence block cites this phase's artifacts.

### Task A4.9 — enable flag

- [ ] After 7 days of dogfood with `MEMD_A4_LEDGER_SURVIVAL=1` set per-user, flip default in hook scripts.
- [ ] Document flip in `docs/handoff/YYYY-MM-DD-a4-default-on.md`.
- [ ] Commit: `feat(a4): default MEMD_A4_LEDGER_SURVIVAL=1 after dogfood window`.

Acceptance: flip is separate commit; traceable; reversible via one-line env override.

---

## 9. Bench impact

A4 does not directly move public-bench numbers (LME / MemBench / ConvoMem / LoCoMo). It unblocks V5 substrate benches:

- **V5 A5 (Session-Continuity Bench, planned).** Scenario generator will seed 5-file read history in session 1, assert survival via `memd hook restore` round-trip in session 2. Pass criterion: 10/10 restore records present, zero breach lines. A4 is the only phase that lights this bench up.
- **V5 D5 (Token-Efficiency Bench, planned).** Prerequisite: wake budget holds after compaction. A4 is necessary but not sufficient (D4 compiler does the arithmetic).

Public bench regression watch: none expected. LME / MemBench / ConvoMem do not exercise compaction. If a regression appears, the cause is elsewhere.

---

## 10. Dependency graph

A4 entry edges: none (roadmap root of V4).
A4 exit edges: B4 (hook-contract enforcement reads `hook-handoff.md`), C4 (correction-capture runs inside sessions that may compact), D4 (wake compiler relies on restored ledger), G4 (integration gate exercises the full path).

Parallelizable? A4 ↔ other V4 phases:
- B4 depends on A4's `hook-handoff.md` — sequential.
- C4 depends on A4 only insofar as corrections mid-session must survive compaction → weak dep, can start in parallel after Task A4.6 commits the contract.
- D4 can scaffold in parallel; wake compiler needs ledger semantics, not A4 implementation.
- F4 depends on C4, indirectly on A4.
- G4 depends on all.

Execution recommendation: finish A4 tasks 1–6 before B4 starts. Tasks 7–9 can run while B4 executes.

---

## Exit criteria for A4

Ordered, each must hold:

1. All unit + integration tests (1–19) pass 10/10 runs.
2. `.memd/logs/continuity-breach.log` is empty across the 10-run E2E scenario.
3. `memd hook doctor --check ordering` exits 0 on canonical trace, non-zero with specific diagnostic on breach trace.
4. `docs/contracts/hook-handoff.md` exists and is linked from phase doc + README.
5. `.memd/hooks/MANIFEST.json` validates (SHA256s match on disk).
6. 10-STAR axis 1 bumped in `docs/verification/MEMD-10-STAR.md` with evidence pointer.
7. All commits are on `research/mining`, atomic, one per task.

If any fails: see `fail_conditions` in the phase doc. Do not close A4.
