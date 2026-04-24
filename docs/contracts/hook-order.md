# Hook Order Contract — Fire Order, Budgets, Failure Classes

> Normative contract for the memd hook runtime. Cited by
> `memd hooks enforce` and `memd hooks doctor --check contract`. Extends
> [`hook-handoff.md`](hook-handoff.md) (A4) with budget, failure-class,
> and trace-emission requirements.

Owner: `docs/phases/v4/phase-b4-plan.md`.

Status: **active** — B4 writes this contract and the `hook_runtime`
module. Cited verbatim by doctor diagnostics; changes here require a
matching `contract_version` bump in `.memd/hooks/MANIFEST.json`.

Current contract version: **0.3**.

---

## 1. Canonical fire order

```
 ┌──────────────┐   ┌──────────────────┐   ┌──────────────┐   ┌─────────────┐   ┌───────────────┐   ┌──────────────┐   ┌──────────────┐   ┌──────────────┐
 │ SessionStart │──▶│ UserPromptSubmit │──▶│   PreRead    │──▶│   PreEdit   │──▶│  PostToolUse  │──▶│  PreCompact  │──▶│  PostCompact │──▶│     Stop     │
 │   wake       │   │  caveman guard   │   │  (observe)   │   │  (observe) │   │  file-ledger  │   │  seal-ledger │   │   restore    │   │   capture    │
 └──────────────┘   └──────────────────┘   └──────────────┘   └─────────────┘   └───────────────┘   └──────────────┘   └──────────────┘   └──────────────┘
```

Gaps are permitted — not every turn compacts, not every turn edits.
Swaps are **not**: `PostCompact` before `PreCompact` within the same
session is a halt-class violation.

### Event tokens

| Token               | Harness emitter                        | Inner cmd (when present)                  | Fires when                                       |
|---------------------|----------------------------------------|-------------------------------------------|--------------------------------------------------|
| `SessionStart`      | claude-code, codex                     | `memd wake`                               | New session spins up                             |
| `UserPromptSubmit`  | claude-code                            | `memd hook context`                       | User submits a prompt                            |
| `PreRead`           | claude-code, codex                     | `memd hook context` / noop                | Tool use will read a file                        |
| `PreEdit`           | claude-code, codex                     | `memd hook context` / noop                | Tool use will write/edit a file                  |
| `PreToolUse`        | claude-code, codex (generic pre-tool)  | `memd hook gate` (optional)               | Generic pre-tool probe — file-op subset covered  |
| `PostToolUse`       | claude-code, codex                     | `memd hook file-interaction`              | After tool use; updates ledger                   |
| `PreCompact`        | claude-code                            | `memd hook seal-ledger`                   | Harness is about to compact transcript           |
| `LedgerSeal`        | `memd hook seal-ledger` (inner)        | —                                         | Sealed ledger written (A4)                       |
| `PostCompact`       | claude-code                            | `memd hook restore`                       | Harness finished compaction                      |
| `LedgerRestore`     | `memd hook restore` (inner)            | —                                         | Active ledger restored from sealed snapshot (A4) |
| `Stop`              | claude-code                            | `memd hook capture`                       | End of turn                                      |

Tokens on this list are the **only** strings `memd hooks doctor` and
`memd hooks enforce` will accept. Unknown tokens → `contract-parse`
exit 3.

---

## 2. Per-event contract

| Event              | Budget (ms) | Failure class | Required predecessors             | Required trace fields                            |
|--------------------|-------------|---------------|------------------------------------|--------------------------------------------------|
| `SessionStart`     | `2000`      | `log`         | none                               | `event`, `ts_ms`, `session_id`, `harness`        |
| `UserPromptSubmit` | `5000`      | `log`         | `SessionStart`                     | + `budget_ms`, `elapsed_ms`, `exit_code`         |
| `PreRead`          | `500`       | `log`         | `SessionStart`                     | + `tool`                                         |
| `PreEdit`          | `500`       | `log`         | `SessionStart`                     | + `tool`, `path`                                 |
| `PreToolUse`       | `500`       | `log`         | `SessionStart`                     | + `tool`, `path?`                                |
| `PostToolUse`      | `500`       | `log`         | `PreEdit` or `PreRead` or `PreToolUse` | + `tool`, `path?`                            |
| `PreCompact`       | `5000`      | `halt`        | `SessionStart`                     | + `elapsed_ms`, `exit_code`                      |
| `LedgerSeal`       | —           | `log`         | `PreCompact`                       | + `sealed_path`                                  |
| `PostCompact`      | `2000`      | `halt`        | `PreCompact`                       | + `elapsed_ms`, `exit_code`                      |
| `LedgerRestore`    | —           | `halt`        | `PostCompact`                      | + `restored_path`, `entries`, `ok`               |
| `Stop`             | `3000`      | `log`         | `SessionStart`                     | + `elapsed_ms`, `exit_code`                      |

Failure-class semantics:

- `halt` — inner non-zero, timeout, or order violation causes the
  wrapper to exit non-zero. Harnesses that honor exit codes (codex,
  claude-code PreCompact) abort the turn.
- `log` — inner non-zero or timeout is recorded in the trace with
  `failure_class` set (`inner-nonzero`, `timeout`, `bad-output`), but
  the wrapper exits 0. Session continues.

The `LedgerSeal` and `LedgerRestore` rows have no budget because they
are inner-command emissions, not harness-driven events. Their budget
is absorbed by the surrounding `PreCompact` / `PostCompact` row.

---

## 3. Trace line shape

NDJSON, append-only, one line per hook fire. Path defaults to
`<BUNDLE_ROOT>/logs/hook-trace.ndjson`; `MEMD_HOOK_TRACE_PATH` overrides.

```json
{
  "ts_ms": 1713811200000,
  "event": "PostCompact",
  "harness": "claude-code",
  "session_id": "sess-abc",
  "budget_ms": 2000,
  "elapsed_ms": 182,
  "exit_code": 0,
  "failure_class": "none",
  "trace_id": "01HWXSAMPLEULID",
  "tool": null,
  "path": null
}
```

Required fields vary by event (§2 column 5). Unknown fields are
accepted and preserved (forward-compat). `trace_id` is a ULID
generated per fire; pairs PreCompact/PostCompact share a session_id
but have distinct `trace_id`.

### `failure_class` values

| Value            | Meaning                                                             |
|------------------|---------------------------------------------------------------------|
| `none`           | Inner exit 0, within budget, order ok                               |
| `timeout`        | Inner exceeded `budget_ms` (SIGTERM sent, exit_code=124)            |
| `inner-nonzero`  | Inner exited non-zero within budget                                 |
| `bad-output`     | Inner emitted malformed output where a schema was expected          |
| `order-violation`| Predecessor of §2 column 4 missing in observed sequence             |
| `contract-parse` | Manifest or contract file invalid                                   |

---

## 4. Wrapper exit codes

Exit codes returned by `memd hooks enforce` and consumed by harnesses:

| Code | Meaning                                                              |
|------|----------------------------------------------------------------------|
| `0`  | Ok; or log-class failure (check trace for `failure_class`)           |
| `1`  | Halt-class inner failure or order violation                          |
| `2`  | Halt-class budget exceeded                                           |
| `3`  | Contract parse failure (manifest or hook-order.md invalid)           |

Non-wrapped harness path (`MEMD_HOOK_ENFORCE=0`) preserves the inner
command's native exit code for backward compatibility with V3 scripts.

---

## 5. Feature flags

| Flag                         | Default | Effect                                                               |
|------------------------------|---------|----------------------------------------------------------------------|
| `MEMD_HOOK_ENFORCE`          | `0` → `1` after dogfood | Route hook scripts through `memd hooks enforce`. |
| `MEMD_HOOK_TRACE_PATH`       | `<BUNDLE_ROOT>/logs/hook-trace.ndjson` | Override trace destination (tests). |
| `MEMD_HOOK_BUDGET_OVERRIDE`  | unset   | Comma-sep `event=ms` overrides (debugging; bypasses §2 column 2).    |

Default flip from `0` to `1` gated on 7-day dogfood window showing
p99 wrapper latency ≤ 200 ms and zero `failure_class=inner-nonzero`
on `log`-class events (silent-swallow rate = 0).

---

## 6. Worked examples

### Good — canonical turn with compaction

```ndjson
{"ts_ms":1000,"event":"SessionStart","harness":"claude-code","session_id":"s1","exit_code":0,"failure_class":"none"}
{"ts_ms":1100,"event":"UserPromptSubmit","harness":"claude-code","session_id":"s1","budget_ms":5000,"elapsed_ms":42,"exit_code":0,"failure_class":"none"}
{"ts_ms":1200,"event":"PreEdit","harness":"claude-code","session_id":"s1","tool":"Edit","path":"src/lib.rs","budget_ms":500,"elapsed_ms":8,"exit_code":0,"failure_class":"none"}
{"ts_ms":1250,"event":"PostToolUse","harness":"claude-code","session_id":"s1","tool":"Edit","path":"src/lib.rs","budget_ms":500,"elapsed_ms":11,"exit_code":0,"failure_class":"none"}
{"ts_ms":2000,"event":"PreCompact","harness":"claude-code","session_id":"s1","budget_ms":5000,"elapsed_ms":182,"exit_code":0,"failure_class":"none"}
{"ts_ms":2010,"event":"LedgerSeal","session_id":"s1","sealed_path":"state/session-s1/sealed/1713811200.json","failure_class":"none"}
{"ts_ms":3000,"event":"PostCompact","harness":"claude-code","session_id":"s1","budget_ms":2000,"elapsed_ms":91,"exit_code":0,"failure_class":"none"}
{"ts_ms":3010,"event":"LedgerRestore","session_id":"s1","restored_path":"state/session-s1/file_interactions.json","entries":5,"ok":true,"failure_class":"none"}
{"ts_ms":4000,"event":"Stop","harness":"claude-code","session_id":"s1","budget_ms":3000,"elapsed_ms":220,"exit_code":0,"failure_class":"none"}
```

`memd hooks doctor --check contract` → exit 0.

### Bad — PostCompact budget exceeded (halt)

```ndjson
{"ts_ms":2000,"event":"PreCompact","session_id":"s2","budget_ms":5000,"elapsed_ms":42,"exit_code":0,"failure_class":"none"}
{"ts_ms":3000,"event":"PostCompact","session_id":"s2","budget_ms":2000,"elapsed_ms":8021,"exit_code":124,"failure_class":"timeout"}
```

Wrapper exits 2 on the offending line; doctor surfaces
`memd_hook_budget_exceeded_total{event="PostCompact"}` and exits 1.

### Bad — silent swallow on log-class event

```ndjson
{"ts_ms":4000,"event":"Stop","session_id":"s3","budget_ms":3000,"elapsed_ms":2900,"exit_code":1,"failure_class":"inner-nonzero"}
```

Wrapper returns 0 (log class), but doctor flags
`memd_hook_silent_swallow_total{event="Stop"}` and exits 1.

### Bad — order violation (PostCompact without PreCompact)

```ndjson
{"ts_ms":1000,"event":"SessionStart","session_id":"s4","exit_code":0,"failure_class":"none"}
{"ts_ms":2000,"event":"PostCompact","session_id":"s4","budget_ms":2000,"elapsed_ms":20,"exit_code":0,"failure_class":"order-violation"}
```

Wrapper exits 1; doctor surfaces
`memd_hook_contract_violation_total{kind="order-violation"}`.

---

## 7. Retention + rotation

- B4 caps `logs/hook-trace.ndjson` at **100 MiB**. Above that, the
  enforcer appends one `{"event":"truncation-required",...}` line per
  session and stops writing until manual rotation.
- V7 owns scheduled rotation + retention policy. B4 intentionally
  keeps the ops surface minimal.

---

## 8. Consumers

- `crates/memd-core/src/hook_runtime/` — pure primitives
  (`HookEvent`, `HookBudget`, `HookTrace`, `FireOrderValidator`).
- `crates/memd-client/src/cli/cli_hook_enforce.rs` — wrapper CLI.
- `crates/memd-client/src/cli/cli_hook_runtime.rs::run_hook_doctor_contract`
  — doctor branch reading trace + manifest + this contract.
- `.memd/hooks/memd-*.sh` + `.ps1` — prefix inner `memd hook …` calls
  with `memd hooks enforce` when `MEMD_HOOK_ENFORCE=1`.
- `docs/phases/v4/phase-g4-plan.md` — Continuity proof gate asserts
  every memd action is user-visible via this contract.

---

## 9. Changelog

| Version | Date       | Change                                                       |
|---------|------------|--------------------------------------------------------------|
| 0.2     | 2026-04-24 | A4 — sealed ledger + ordering check (hook-handoff.md).       |
| 0.3     | 2026-04-24 | B4 — enforcer, budgets, failure classes, universal trace.    |
