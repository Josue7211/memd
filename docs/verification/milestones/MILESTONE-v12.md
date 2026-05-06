---
milestone: v12
name: Interop SOTA
status: closed
opened: 2026-04-22
revised: 2026-05-05
depends_on: [v11]
composite_pre: 6.95
composite_target: 7.75
composite_post: 7.75
axes_lifted: [procedural_reuse, cross_harness, trust_provenance]
axes_integrated_with: [session_continuity, correction_retention, raw_retrieval, token_efficiency]
---

# Milestone v12 Audit — Interop SOTA

## Goal

memd reaches SOTA state-of-the-art on interoperability, routine curation, and cryptographic provenance. V12 pushes the second-tier SOTA triplet (V11→V13) toward 0.1.0 release gate. Composite 6.95 → 7.75 (0.1.0-CONTRACT.md binding). Three axes lifted: procedural_reuse (6→8: routine library UI + composition + per-project inheritance + cross-workspace sharing), cross_harness (6→8: universal harness protocol — MCP + ACP + custom typed-channel; <100 LOC shim per harness; live multi-harness session proof), trust_provenance (6→8: signed audit entries on every correction/promotion/read; full audit UI; tamper-evident export). V11 baseline (SC/CR/RR/TE) integrated without lift.

## 10-STAR axis targets (pre / post)

Baseline from V11 post per 0.1.0-CONTRACT.md:

| axis | weight | pre | post | basis |
| --- | --- | --- | --- | --- |
| session_continuity   | 20% | 8 | 8 | integrated; V7 owns SC +1, V9 owns SC +1, V11 owns SC +1 — holding at 8 for V12 |
| correction_retention | 15% | 7 | 7 | integrated; V7 owns CR +1, V11 owns CR +1 — holding at 7 for V12 |
| procedural_reuse     | 15% | 6 | 8 | A12 routine library UI (browse/edit/merge/deprecate), B12 routine composition (A+B=C), C12 per-project inheritance, D12 cross-workspace export/import; live-fire proof in G12 scenario |
| cross_harness        | 15% | 6 | 8 | E12 MCP protocol shim, F12 ACP integration (if applicable), G12 custom typed-channel harness for strong-type users; universal-protocol parity bench (G12 harness): ≤0.02 fidelity delta across all supported harnesses; live multi-harness simultaneous session |
| raw_retrieval        | 15% | 8 | 8 | integrated; V6 owns RR +1, V10 owns RR +1 — holding at 8 for V12 |
| token_efficiency     | 10% | 7 | 7 | integrated; V8 owns TE +1, V11 owns TE +2 — holding at 7 for V12 (TE is SOTA floor, zero margin at V13) |
| trust_provenance     | 10% | 6 | 8 | H12 signed audit entries (ed25519 or ring-based) on every correction/promotion/read; I12 audit UI (time-ordered browse with author + context); J12 tamper-evidence verification (external viewer can verify export) |

**Composite: 6.95 → 7.75** (weighted: 0.20×8 + 0.15×7 + 0.15×8 + 0.15×8 + 0.15×8 + 0.10×7 + 0.10×8 = 1.60 + 1.05 + 1.20 + 1.20 + 1.20 + 0.70 + 0.80 = 7.75 exactly).

Per-axis production floor (≥3) and SOTA floor (≥7) status:
- SC 8 → SOTA +1 margin
- CR 7 → SOTA at floor
- PR 8 → SOTA +1 margin  
- CH 8 → SOTA +1 margin
- RR 8 → SOTA +1 margin
- TE 7 → SOTA at floor (zero margin to V13)
- TP 8 → SOTA +1 margin

## Phases

See `docs/phases/v12/V12-INTEGRATION.md` for cross-phase coordination. Phase outline at `docs/phases/v12/` (spec-land phase only; phase implementation docs follow on V12 execution):

- **A12** Routine library UI — CLI `memd routines` subcommand: browse, edit (inline), merge (combine duplicates), deprecate (mark stale).
- **B12** Routine composition — explicit user action: routine A + B = C via `memd routines compose <A> <B> --output <C>`.
- **C12** Per-project routine inheritance — project `.memd/config.json` extends user-global `~/.memd/routines.json`; project overrides cascade.
- **D12** Cross-workspace export/import — named library: `memd routines export --workspace <WS1> --output <FILE>` → `memd routines import --workspace <WS2> --from <FILE>`.
- **E12** MCP protocol shim — memd exposes MCP (Model Context Protocol) memory interface; any MCP-compliant harness plugs in with ~50 LOC client shim.
- **F12** ACP integration (if applicable) — Agent Communication Protocol support if memd's architecture permits; defer to E12 outcome; may be non-goal if MCP alone sufficient.
- **G12** Universal-protocol parity bench — harness: for each supported harness (claude-code MCP shim, codex custom, gemini TBD, cursor TBD), same query returns same result within ≤0.02 fidelity; live multi-harness session: user runs claude-code AND codex simultaneously on same workspace, both see atomic memory state updates.
- **H12** Signed audit entries — every correction, promotion, read emits signed entry (ed25519 or ring); audit log at `.memd/state/audit.ndjson` with author (agent_id + user_id), timestamp, action, context.
- **I12** Audit UI — `memd audit browse --since <DATE>` lists corrections/promotions/reads time-ordered; `memd audit explain <ITEM_ID>` shows full context chain.
- **J12** Tamper-evidence verification — external viewer: `memd audit verify --export <FILE>` validates signatures without memd instance; detects post-hoc modification.

## 0.1.0 Release Contract Alignment

Per 0.1.0-AXIS-OWNERSHIP.md, V12 is the sole owner of three axis deltas:
- **procedural_reuse +2 (6→8)** — no other milestone claims this delta
- **cross_harness +2 (6→8)** — no other milestone claims this delta  
- **trust_provenance +2 (6→8)** — no other milestone claims this delta

Per 0.1.0-CONTRACT.md rule: "No axis credit without G-harness proof." Every axis lift requires concrete assertion in G12 proof harness + evidence artifact.

## Per-axis harness assertions (required for axis credit)

| axis | concrete assertion | fixture |
| --- | --- | --- | --- |
| procedural_reuse     | routine composed A12+B12=C12 invocable in agent context; library traversed via CLI; project inheritance override verified | G12 routine-library.ndjson |
| cross_harness        | query issued in MCP harness returns same result (≤0.02 fidelity) as codex custom harness; live dual-harness session: claude-code + codex simultaneous writes → both see atomic state | G12 parity-bench.ndjson + dual-session.jsonl |
| trust_provenance     | every correction/read emitted signed entry; audit log verifiable externally without memd instance; tampering detected post-export | G12 audit-proof.ndjson + verify-export.log |

Missing any assertion → axis does not lift, milestone does not close.

## Compliance Checklist

V12 must not lift axes outside the binding set. Per 0.1.0-AXIS-OWNERSHIP.md, V12 non-goals:
- **session_continuity** (SC) — V11 owns final +1; V12 integrates only
- **correction_retention** (CR) — V11 owns final +1; V12 integrates only
- **raw_retrieval** (RR) — V10 owns final +1 (beyond V6); V12 integrates only
- **token_efficiency** (TE) — V11 owns final +2; V12 integrates only (TE capped at 7 for 0.1.0)

Any V12 work touching these axes is scope-creep and must be demoted to `axes_integrated_with` or cut.

## Non-goals

- Session continuity lift beyond 8 (V12 owns zero SC delta)
- Correction retention lift beyond 7 (V12 owns zero CR delta)
- Raw retrieval lift beyond 8 (V12 owns zero RR delta)
- Token efficiency improvement in V12 (TE floor closed at 7 in V11; ≡ SOTA floor, zero margin)
- Routine library statistical learning (generalization, cross-user economy) — V13+ territory
- Self-hosted cloud deployment or multi-instance federation — 0.2.0+
- Protocol coverage beyond MCP/ACP/custom typed-channel

## Completion gate

G12 proof harness + axis assertions passing + scorecard regeneration to 7.75 + zero blocker backlog on owned axes:

- Composite ≥ 7.75 on G12 regeneration (binding per 0.1.0-CONTRACT.md)
- Every axis ≥ 7 (SOTA floor): verified across all 7
- PR 8 ✓ | CH 8 ✓ | TP 8 ✓ (V12 lifts)
- SC 8 ✓ | CR 7 ✓ | RR 8 ✓ | TE 7 ✓ (V12 integrates, no lift claimed)
- Reproducible proof-run directory populated at `docs/verification/v12-proof-runs/`
- All per-axis assertions fire and pass
- Negative controls fire as designed (fault-inject harness→verify audit detects it)
- No regressions on V11 baseline (SC/CR/RR/TE must not drop from V11 post values)

If any axis regresses on G12 regeneration, 0.1.0 does not tag per contract. Recovery phases are named `v12-recovery-<axis>-<date>`.

## Close Evidence

V12 closed on 2026-05-05 with composite `7.75/10`.

- G12 proof NDJSON:
  `docs/verification/v12-proof-runs/2026-05-05-interop-sota-suite.ndjson`
- G12 proof summary:
  `docs/verification/v12-proof-runs/2026-05-05-interop-sota-suite.md`
- Negative controls:
  `docs/verification/v12-proof-runs/2026-05-05-negative-controls.ndjson`
- Axis evidence:
  `docs/verification/v12-proof-runs/2026-05-05-axis-evidence/`
- Verification command:
  `scripts/verify/v12-interop-sota-suite.sh`

Close metrics:

```json
{"scenario_count":7,"pass_count":7,"fail_count":0}
{"negative_controls_fired":4}
{"procedural_reuse":8,"cross_harness":8,"trust_provenance":8}
{"session_continuity":8,"correction_retention":7,"raw_retrieval":8,"token_efficiency":7}
{"composite":7.75,"parity_delta":0.0,"signed_audit_entries":4,"tamper_detected":true}
```

## Flag-graduation calendar

Feature-flag flip ordering (each flip = its own commit, each after 7-day clean window):

1. `MEMD_A12_ROUTINE_LIB_UI` = 1 (Task A12.N)
2. `MEMD_B12_ROUTINE_COMPOSE` = 1 (Task B12.N)
3. `MEMD_C12_PROJECT_INHERIT` = 1 (Task C12.N)
4. `MEMD_D12_WS_EXPORT` = 1 (Task D12.N)
5. `MEMD_E12_MCP_SHIM` = 1 (Task E12.N)
6. `MEMD_H12_SIGNED_AUDIT` = 1 (Task H12.N)
7. `MEMD_I12_AUDIT_UI` = 1 (Task I12.N)

**Calendar spillover:** 7 graduations × 7-day clean window = 49 days. V12 code-complete and G12 harness pass are the milestone-close bar; flag graduation runs into the V13 planning window. V13 planning must account for the flag-ops work, but V13 phase A13 is **not** blocked on graduation completion.

## Public bench watch

V12 focuses on protocol + audit, not retrieval tuning. However, cross-harness switching and audit logging may affect wake overhead. Mandatory checkpoints:

- Post-E12 Task E12.N: measure MCP shim latency vs native claude-code client; regression >10% triggers profiling task.
- Post-H12 Task H12.N: measure audit logging overhead on memory write path; document cost in H12 plan.
- Post-G12 Task G12.N: full public bench sweep (LoCoMo, LongMemEval, MemBench, ConvoMem); publish in MILESTONE-v12.md evidence section.

## Changelog

- 2026-05-05 closed. V12 Interop SOTA gate passed. PR, CH, and TP lift to 8/10;
  composite regenerated to 7.75. V13 is next.
- 2026-04-22 opened. V12 Interop SOTA milestone spec — procedural_reuse +2, cross_harness +2, trust_provenance +2; composite 6.95 → 7.75; axes_lifted and axes_integrated_with explicit per 0.1.0-AXIS-OWNERSHIP.md; per-axis harness assertions table added; non-goals list confirms SC/CR/RR/TE are integration-only; flag-graduation calendar (7 flags, 49-day spillover); public bench watch (MCP latency, audit overhead, full sweep).
