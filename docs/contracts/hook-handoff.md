# Hook Handoff Contract — PreCompact → PostCompact → Tool Use

> Normative contract for memd's read-state-across-compaction loop. Cited by
> `memd hook doctor --check ordering` messages and by hook script comments.

Owners: `phase-a4-plan.md`, `phase-b4-plan.md`.

Status: **active** — A4 writes this contract; B4 wires the full hook-trace
emitter that powers the ordering check end-to-end.

---

## 1. Fire order (normative)

```
 ┌──────────────┐     ┌─────────────────┐     ┌──────────────┐     ┌─────────────┐
 │  PreCompact  │ ──▶ │  (compaction)   │ ──▶ │ PostCompact  │ ──▶ │  PreToolUse │
 │  seal-ledger │     │  transcript     │     │   restore    │     │  Read/Edit/ │
 │              │     │  collapse       │     │  sealed →    │     │  Write/...  │
 │              │     │                 │     │  active      │     │             │
 └──────────────┘     └─────────────────┘     └──────────────┘     └─────────────┘
```

### MUSTs

1. **M-SEAL** — PreCompact MUST invoke `memd hook seal-ledger --session-id <SID>`
   before compaction begins. The sealed ledger lives at
   `<BUNDLE_ROOT>/state/session-<SID>/sealed/<ts>.json`.
2. **M-RESTORE** — PostCompact MUST invoke
   `memd hook restore --session-id <SID> --output <BUNDLE_ROOT>` **before** any
   PreToolUse event fires for file-op tools (`Read`, `Edit`, `Write`,
   `NotebookEdit`).
3. **M-IDEMPOTENT** — Restore is byte-for-byte copy of the newest sealed
   ledger into `file_interactions.json`. Running it twice for the same session
   MUST converge to the same on-disk state.
4. **M-NON-BLOCKING** — PostCompact hook scripts MUST exit 0 even when the
   restore CLI reports `no-sealed-ledger`. The breach is logged; the turn
   continues. Blocking here would brick sessions that legitimately have no
   prior ledger (fresh compaction cycle on a brand-new session).
5. **M-TRACE** — Hook runners that emit a trace MUST use the event tokens
   below; the ordering check recognizes only these tokens.

### Event tokens

| Token                   | Emitter              | Meaning                                     |
|-------------------------|----------------------|---------------------------------------------|
| `PreCompact`            | harness hook runner  | PreCompact hook entry                       |
| `LedgerSeal`            | `memd hook seal-ledger` | Sealed ledger written                     |
| `PostCompact`           | harness hook runner  | PostCompact hook entry                      |
| `LedgerRestore`         | `memd hook restore`  | Active ledger restored from sealed snapshot |
| `PreToolUse`            | harness hook runner  | About to invoke a tool (`tool` field set)   |

---

## 2. Breach categories

Written as single append-only lines in
`<BUNDLE_ROOT>/logs/continuity-breach.log` with the form:

```
<rfc3339-utc> <session_id> breach=<kind>[ key=value]*
```

| Kind                 | Trigger                                                                 | Severity |
|----------------------|-------------------------------------------------------------------------|----------|
| `no-sealed-ledger`   | PostCompact restore ran but no sealed ledger existed for the session.   | WARN     |
| `tool-before-restore`| A file-op PreToolUse fired after PostCompact but before LedgerRestore.  | ERROR    |
| `missing-restore`    | PostCompact event was seen in the trace with no matching LedgerRestore. | ERROR    |

`tool-before-restore` and `missing-restore` indicate a broken handoff and
should cause the `ordering` check to exit non-zero. `no-sealed-ledger` is
advisory — a fresh session with no prior seal is expected to hit it once.

---

## 3. Observability

| File                                                      | Writer                        | Shape                                  |
|-----------------------------------------------------------|-------------------------------|----------------------------------------|
| `<BUNDLE_ROOT>/logs/continuity-breach.log`                | `memd hook restore`, doctor   | append-only text, one line per breach  |
| `<BUNDLE_ROOT>/logs/ledger-restore.ndjson`                | `memd hook restore`           | append-only NDJSON `LedgerRestoreReport` |
| `<BUNDLE_ROOT>/logs/hook-trace.ndjson`                    | B4 hook runner (future)       | append-only NDJSON event trace         |

`LedgerRestoreReport` shape:

```json
{
  "session_id": "sess-abc",
  "sealed_path": "<BUNDLE_ROOT>/state/session-sess-abc/sealed/1712345678.json",
  "restored_path": "<BUNDLE_ROOT>/state/session-sess-abc/file_interactions.json",
  "entries": 7,
  "source": "postcompact-hook",
  "ok": true,
  "error": null,
  "ts_ms": 1712345678901
}
```

---

## 4. Feature flags

| Flag                          | Default | Effect                                                                 |
|-------------------------------|---------|------------------------------------------------------------------------|
| `MEMD_A4_LEDGER_SURVIVAL`     | `0`     | `0` → PostCompact hook exits immediately (no-op). `1` → restore runs.  |
| `MEMD_BUNDLE_ROOT`            | auto    | Overrides bundle root used by hook scripts.                            |
| `MEMD_HOOK_STATE_DIR`         | `$HOME/.memd/hook_state` | Hook log directory for script-level diagnostics.      |

Default flip to `1` is gated on a 7-day dogfood window showing zero
`tool-before-restore` / `missing-restore` breach lines under normal use.

---

## 5. Example traces

### Good — canonical fire order

```ndjson
{"event":"PreCompact","ts_ms":1000,"session_id":"s1"}
{"event":"LedgerSeal","ts_ms":1001,"session_id":"s1"}
{"event":"PostCompact","ts_ms":2000,"session_id":"s1"}
{"event":"LedgerRestore","ts_ms":2001,"session_id":"s1"}
{"event":"PreToolUse","ts_ms":2010,"tool":"Read","path":"src/lib.rs","session_id":"s1"}
{"event":"PreToolUse","ts_ms":2020,"tool":"Edit","path":"src/lib.rs","session_id":"s1"}
```

`memd hook doctor --check ordering` exits 0 on this trace.

### Bad — tool-before-restore

```ndjson
{"event":"PostCompact","ts_ms":2000,"session_id":"s2"}
{"event":"PreToolUse","ts_ms":2001,"tool":"Edit","path":"src/lib.rs","session_id":"s2"}
{"event":"LedgerRestore","ts_ms":2002,"session_id":"s2"}
```

Doctor exits non-zero; writes:

```
2026-04-24T00:00:00Z s2 breach=tool-before-restore tool=Edit path=src/lib.rs
```

### Bad — missing-restore

```ndjson
{"event":"PostCompact","ts_ms":2000,"session_id":"s3"}
```

(no LedgerRestore event follows)

Doctor exits non-zero; writes:

```
2026-04-24T00:00:00Z s3 breach=missing-restore
```

---

## 6. Consumers

- `crates/memd-client/src/cli/cli_hook_runtime.rs::run_hook_doctor_ordering`
  — state machine enforcing §1 + §2.
- `.memd/hooks/memd-postcompact-restore.{sh,ps1}` — canonical PostCompact
  hook scripts. Cite this contract in header comments.
- `integrations/hooks/` — auto-synced mirror for external installers.
- `docs/phases/v4/phase-b4-plan.md` — Hook runner emits §1 tokens into
  `logs/hook-trace.ndjson`.
- `docs/phases/v4/phase-g4-plan.md` — Continuity proof gate verifies this
  contract end-to-end.
