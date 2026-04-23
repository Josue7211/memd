---
version: v12
kind: integration-plan
status: ready-to-plan
opened: 2026-04-22
revised: 2026-04-22
scope: A12..J12 (outline only — implementation phase plans follow)
depends_on: [../../verification/0.1.0-CONTRACT.md, ../../verification/0.1.0-AXIS-OWNERSHIP.md, ../../verification/milestones/MILESTONE-v12.md, ../../theory/MEMD-SOTA-THEORY.md]
---

# V12 Integration — Cross-Phase Plan

> V12 is the second of three SOTA push milestones (V11-V13). This outline covers the ten-phase scope without implementation detail. Phase-level specs (A12..J12 plan docs) are generated during V12 execution. This doc defines coordination rules, protocol surface, assertion fixtures, and the dual-harness scenario G12 executes.

## 1. Execution-order discipline

Phase dependency tree (strict):

```
A12 ──┬──► B12 ──────┬──► G12 (routine composition, inheritance)
      │              │
      └──► C12 ───────┤
      │              │
      └──► D12 ───────┤
      │              │
      └──► E12 ───────┤ (protocol unification, parity bench)
           │          │
           └─► F12 ───┤
           │          │
           └──► H12 ──┤
                │      │
                └─► I12 ┤
                │       │
                └─► J12 ┘
```

Rules:
- A12 UI foundation must land before B12 (composition reuses CLI infrastructure).
- C12, D12, E12, F12 can parallelize after A12 Task A12.N (CLI foundation doc).
- H12 signed audit requires E12/F12 protocol unification (audit entries must be harness-agnostic).
- I12 (audit UI) requires H12 schema + audit.ndjson populated.
- J12 (tamper-evidence verification) requires I12 + export format finalized.
- G12 requires all phases 1–6 (A12–F12) to land before proof harness runs.

No phase may short-circuit a prior dependency. If blocked, file backlog item and surface in next session.

## 2. Protocol Coverage — memd for Any Harness

V12 binds memd to the three emerging harness communication standards. Per MEMD-SOTA-THEORY.md §4 (cross_harness 7/10 baseline):

> "Two harnesses (claude-code + codex) flip cleanly with ≥0.98 parity on retrieval, corrections survive harness switch, no leak between users within same workspace."

V12 extends this to:

> "Universal harness protocol — memd speaks MCP, ACP, and a custom protocol for harnesses that want typed channels. Any harness plugs in with <100 LOC shim. Live multi-harness session (user runs claude-code AND codex simultaneously, both see same memory state)."

### E12 — Model Context Protocol (MCP)

**Scope:** Expose memd memory store (read queries, write corrections, recall) as a standard MCP resource server.

**Protocol version (if versioned at spec time):** Document the MCP API version memd targets (e.g., MCP v1.0, draft-XX).

**Shim pattern:** Any MCP client (claude-code, gemini, other) implements ~50 LOC wrapper:
```
1. Connect to memd MCP server (stdio or HTTP transport)
2. Register memory query intent ("give me focus context")
3. Decode memory NDJSON response
4. Populate harness context
```

**Scope boundary:** Read path (queries) + correction ingestion (write path). Multi-user isolation handled by harness preset + workspace_id; memd enforces visibility at schema level.

### F12 — Agent Communication Protocol (ACP)

**Status:** TBD pending E12 outcome. If MCP alone satisfies all target harnesses, F12 becomes non-goal and scope is cut. If additional harnesses require ACP, F12 implements it.

**Decision point:** Post-E12 Task E12.N. If gemini/cursor/aider can consume MCP without ACP, F12 is archived and noted as "integration-only" in MILESTONE-v12.md (no axis credit claimed).

**Placeholder scope (if F12 proceeds):** Interop with agent-native federation protocols (if applicable to memd deployment model). Details deferred to F12 plan.

### Custom Typed-Channel Protocol

**Scope:** For memd-native agents (agents built specifically around memd) or agents that want strong static guarantees, a custom Rust protocol with protobuf/msgpack encoding.

**Not in E12 or F12 scope.** G12 harness implementation may use this for the multi-harness proof (claude-code + codex simultaneously); codex-native implementation might choose typed-channel over MCP for latency.

**Serialization:** Define in G12 plan: protobuf v3 or msgpack; versioning strategy; forward-compatibility guarantees.

## 3. Shared Test Fixtures & Scenario

### 3.1 Routine Library Fixtures

| Fixture | Owner | Shared with |
| --- | --- | --- |
| `fixtures/shared/routines/seed-library.jsonl` | A12 | B12, C12, D12, G12 |
| `fixtures/shared/routines/composition-chain.jsonl` | B12 | C12, D12, G12 |
| `fixtures/shared/routines/project-override.json` | C12 | D12, G12 |
| `fixtures/shared/routines/export-snapshot.tar.gz` | D12 | G12 (import test) |

Convention: Phase-unique fixtures live under `fixtures/v12/<phase>/`; shared fixtures move to `fixtures/shared/routines/` after second reference.

### 3.2 Protocol & Audit Fixtures

| Fixture | Owner | Shared with |
| --- | --- | --- |
| `fixtures/shared/protocols/mcp-responses.ndjson` | E12 | G12 (parity bench) |
| `fixtures/shared/protocols/dual-harness-session.jsonl` | G12 | verification harness |
| `fixtures/shared/audit/canonical-audit-log.ndjson` | H12 | I12, J12, G12 |
| `fixtures/shared/audit/tampered-export.ndjson` | J12 | verify harness (negative control) |

## 4. Dual-Harness Scenario G12 Executes

G12 proof harness establishes three facts:
1. Routine library works end-to-end (A12–D12 assertions).
2. Universal protocol parity holds (E12–F12 assertions).
3. Cryptographic provenance is trustworthy (H12–J12 assertions).

Full canonical script; each turn tagged with the V12 phase it exercises.

### Session 1 — "routine discovery" (claude-code, 15 turns)

| Turn | Actor | Utterance | Phases exercised |
| --- | --- | --- | --- |
| T1 | user | "I often lint and format before commit." | A12 (routine candidate detection begins) |
| T2 | agent | Read config files | A12 |
| T3 | user | "Now format the code." | A12 (pattern observation 1/3) |
| T4 | agent | Format | A12 |
| T5 | user | "Lint it too." | A12 (pattern observation 2/3) |
| T6 | agent | Lint | A12 |
| T7 | user | "Next file — lint and format please." | A12 (pattern observation 3/3 → routine candidate ready) |
| T8 | agent | Lint + format | A12 |
| T9 | system | User browses routines: `memd routines` | A12 (CLI lists "lint-format" candidate) |
| T10 | user | "Edit that routine to include commit." | B12 (routine edit begins) |
| T11 | system | Agent A12 + agent B12 → composition: "lint" + "format" ⇒ new "lint-format" | B12 (composition happens within A12 routine) |
| T12 | user | "Use the merged routine on file.rs" | B12 (invocation of composed routine) |
| T13 | agent | Execute lint-format | B12 |
| T14 | user | Override project default: "Use 4-space indent for this project." | C12 (project-level config write) |
| T15 | system | SessionStop → PreCompact | A12, B12, C12 seal |

### Session 2 — "cross-workspace + codex harness" (codex on different workspace, 10 turns)

**Key:** Session 2 runs on **codex**, **different workspace**. This tests both cross-harness (CH) **and** cross-workspace routine export (D12).

| Turn | Actor | Utterance | Phases |
| --- | --- | --- | --- |
| — | system | Wake via codex preset; load workspace WS-2; import routine library from WS-1 | D12 (cross-workspace export/import), E12 (MCP/custom protocol) |
| T16 | user | "Do we have the lint-format routine?" | A12 (CLI list across imported routines) |
| T17 | system | `memd routines browse --workspace ws-2` shows "lint-format" imported from ws-1 | D12 (import surfaced in CLI) |
| T18 | user | "Apply lint-format to src/main.rs" | B12 (invocation, codex harness) |
| T19 | agent | Execute imported routine | B12 |
| T20 | user | "Deprecate that routine — we switched to prettier." | A12 (deprecation) |
| T21 | system | `memd audit browse` shows deprecation + author (codex) + timestamp | H12, I12 (audit logged) |
| T22 | user | `memd audit explain <deprecation-turn>` | I12 (drilldown: who deprecated, why, when) |
| T23 | system | Full context chain surfaced (predecessor routines, usage count, etc.) | I12 (full audit context) |
| T24 | system | SessionStop → PreCompact | all phases seal |

### Session 3 — "Multi-harness Simultaneous" (claude-code + codex running in parallel, 5 turns)

This is the strongest proof of CH 6→8 lift: both harnesses run **at the same time** on the same workspace, see the same memory state, and atomic writes don't conflict.

| Turn | Actor | Utterance | Phases |
| --- | --- | --- | --- |
| — | system | Start terminal 1 (claude-code WS-1) and terminal 2 (codex WS-1) **simultaneously** | E12/F12/custom protocol (both harnesses connect) |
| T25 | harness-A (claude-code) | Write correction: "primary key is ulid not uuid" | H12 (signed audit entry emitted) |
| T26 | harness-B (codex) | Query: "what's our primary key?" | E12/F12 (parity: both harnesses, same query) |
| T27 | system | harness-B receives "ulid" (T25 correction visible in codex immediately, ≤0.02 fidelity) | E12/F12, CH (cross-harness consistency) |
| T28 | harness-B (codex) | Write: "Actually, use uuid for backward compat." | H12 (correction override, counter-signed) |
| T29 | harness-A (claude-code) | Query again: "primary key?" | E12/F12 (eventual consistency: claude-code sees codex's write) |
| T30 | system | harness-A receives "uuid"; audit log shows both corrections + timestamps + signers (codex then claude-code) | H12, I12 (audit chain; J12 verifies externally) |

Cut-3 assertions + tamper-evidence check — see G12 plan §4 Cut 3.

### Scenario self-tests (fault-injection)

G12 Task G12.N includes negative controls:
- Skip A12 routine library → assert CLI fails gracefully.
- Inject H12 bad signature → assert J12 verify detects tampering.
- Drop E12 MCP connection → assert protocol fallback (if any) or error is clean.
- Corrupt D12 export file → assert import rejects it.

These prove the harness is honest.

## 5. Per-Harness Shim Estimation

**Time-budget for <100 LOC shim per harness:**

| Harness | Shim scope | Estimated LOC |
| --- | --- | --- |
| claude-code (MCP target) | MCP client + context-injection hook | 60–80 |
| codex (custom typed-channel target) | Custom protobuf unmarshaler + recall router | 70–90 |
| gemini (MCP fallback) | MCP client + special-token handling | 50–70 |
| cursor (MCP fallback) | MCP client + editor-specific routing | 60–80 |

If any shim exceeds 120 LOC, the protocol is over-engineered and must be simplified before harness launch.

## 6. Cross-Phase API Surface Summary

| Introduced in | Symbol / Path | Consumed by |
| --- | --- | --- |
| A12 | `memd_core::routine_lib::*` | B12, C12, D12, G12 |
| A12 | `memd::cli::routines` subcommand | all phases + user |
| B12 | `routine_compose(a, b) -> Routine` | C12, D12, G12 |
| C12 | `.memd/config.json` schema: `[routines.inherit_from]` | D12, G12 |
| D12 | `routine_export(workspace) -> tarball` + `routine_import(workspace, tarball)` | G12 |
| E12 | `memd_core::mcp_server::*` (MCP resource server) | all MCP harnesses |
| F12 | `memd_core::acp_adapter::*` (if F12 proceeds) | ACP-capable harnesses |
| H12 | `memd_core::audit::signed_entry::*` (ed25519 or ring) | I12, J12, G12 |
| H12 | `.memd/state/audit.ndjson` | I12 (browse), J12 (verify), G12 (proof) |
| I12 | `memd::cli::audit browse --since <DATE>` | users + G12 |
| I12 | `memd::cli::audit explain <ITEM_ID>` | users + G12 |
| J12 | `audit_verify(export_file) -> bool` | external verification tool |
| G12 | `docs/verification/v12-proof-runs/*.ndjson` | V13 entry gate |

## 7. Feature-Flag Graduation Calendar

Seven flags, 7-day clean window each = 49-day spillover into V13 window.

1. `MEMD_A12_ROUTINE_LIB_UI` = 1 (Task A12.N)
2. `MEMD_B12_ROUTINE_COMPOSE` = 1 (Task B12.N)
3. `MEMD_C12_PROJECT_INHERIT` = 1 (Task C12.N)
4. `MEMD_D12_WS_EXPORT` = 1 (Task D12.N)
5. `MEMD_E12_MCP_SHIM` = 1 (Task E12.N)
6. `MEMD_H12_SIGNED_AUDIT` = 1 (Task H12.N)
7. `MEMD_I12_AUDIT_UI` = 1 (Task I12.N)

**Calendar spillover:** V12 code-complete and G12 harness pass are the milestone-close bar. Flag graduation happens post-close and runs into V13 planning window (see MILESTONE-v12.md).

## 8. Bench Regression Watch

V12 focuses on interop + audit, not retrieval. However:

- Post-E12 Task E12.N: measure MCP shim latency vs native client; regression >10% triggers profiling.
- Post-H12 Task H12.N: measure audit logging overhead on memory write; document in H12 plan.
- Post-G12 Task G12.N: full public bench sweep (LoCoMo, LongMemEval, MemBench, ConvoMem); no regression >2% expected.

If any public bench regresses >2%, root-cause and file recovery task.

## 9. Commit Strategy

### Plan-spec land phase (this task)

One atomic commit:

```
docs(v12): V12-INTEGRATION cross-phase plan
```

Implementation phase-spec commits (A12–J12) produced by future agents during V12 execution:

```
docs(v12): phase-a12-plan implementation spec
docs(v12): phase-b12-plan implementation spec
...
docs(v12): phase-j12-plan implementation spec
```

### Execution commits per phase

Each phase plan has its own internal task list. Execution commits are produced during phase execution, not now.

### Handoff commit

After all 10 implementation phase specs land:

```
docs(handoff): V12 plan specs landed, next agent executes A12
```

## 10. Open Questions for V12 Executor

Surface these in plan-spec phase kickoff:

- **MCP version lock:** Which MCP API version does memd target at V12? (e.g., MCP v1.0, draft-XX)
- **ACP viability:** Post-E12, are all target harnesses (gemini, cursor, aider) reachable via MCP, or does ACP add value? (F12 scope decision point)
- **Custom protocol serialization:** Protobuf v3 or msgpack for typed-channel? Forward-compatibility strategy?
- **Audit log retention:** How long to keep `.memd/state/audit.ndjson`? Rotation policy? (H12 question)
- **Signing key management:** Ed25519 or ring-based? Per-agent key or shared workspace key? (H12 question)
- **Cross-workspace routine name collisions:** If WS-1 exports "lint-format" and WS-2 already has "lint-format", what happens? Override, rename, merge? (D12 question)

## 11. Why V12 owns these three axes

Per MEMD-SOTA-THEORY.md and 0.1.0-AXIS-OWNERSHIP.md, V12's three lifts are the remaining blockers on the SOTA floor:

- **Procedural_reuse (PR) 6→8:** V5 detected routines, V10 measured them. V12 curates the library — user edits, composes, deprecates. No routine economy without curation.
- **Cross_harness (CH) 6→8:** V4 proved flip-parity. V12 scales to universal protocol + simultaneous multi-harness. No multi-agent parity without this.
- **Trust_provenance (TP) 6→8:** V7 surfaced corrections, V8 added browser. V12 adds cryptographic proof — every entry signed, every read logged, external verification possible. No compliance-grade trust without this.

All three are non-cryptographic until V12, non-curated until V12, single-harness until V12. V12 closes the gaps.

## 12. Exit Criteria for V12 as a Milestone

All ten phase exit criteria met AND G12 exit criteria met AND:

- 10-STAR composite ≥ 7.75 written to `docs/verification/MEMD-10-STAR.md` by G12 scorecard regenerator.
- No axis score above targets in MILESTONE-v12.md (regenerator fails loud on over-claim).
- `docs/verification/milestones/MILESTONE-v12.md` filled with evidence paths.
- `ROADMAP.md` V12 → closed, V13 → in progress.
- No open blocker backlog on procedural_reuse, cross_harness, trust_provenance.
- Universal-protocol parity bench passing (≤0.02 fidelity across all harnesses).
- Live dual-harness simultaneous session proof passing (T25–T30 scenario, both harnesses live, atomic writes).
- Signed audit entries populated non-zero; verification tool functional.
- Per-axis harness assertions all passing.
- Schema locks for audit + protocol versioning landed and tested.
- `docs/contracts/universal-harness-protocol.md` written (protocol binding).
- Final handoff doc points at `docs/phases/v13/` (to be created in V13 plan-spec phase).

## 13. Changelog

- 2026-04-22 opened. V12 Interop SOTA integration plan — ten-phase scope (A12–J12 outline only); dual-harness scenario with three cuts (routine discovery + cross-ws export + simultaneous multi-harness); protocol coverage (MCP + ACP TBD + custom typed-channel); shared fixtures for routines + protocols + audit; seven-flag graduation calendar (49-day spillover); bench regression watch; exit criteria including dual-harness proof + universal-protocol parity; questions for executor.
