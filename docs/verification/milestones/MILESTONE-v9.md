---
milestone: v9
name: Multi-User / Team
status: closed
opened: 2026-04-22
revised: 2026-04-22
closed: 2026-05-05
depends_on: [v8]
composite_pre: 5.10
composite_target: 5.60
axes_lifted: [session_continuity, cross_harness]
axes_integrated_with: []
---

# Milestone v9 Audit — Multi-User / Team

## Goal

Federated memory with per-user isolation and cross-user assertion enforcement.
V9 owns SC +1 (5→6) via session continuity across user context switches; CH +2
(4→6) via multi-user harness isolation + adversarial suite proving zero visibility
leaks, zero identity collision, zero scope escalation. Contract enforcement site:
`docs/contracts/federated-memory-visibility.md` (published V4, enforced V9 with
full adversarial suite). Opens pre-ship window (G9 harness is release-candidate
dry-run).

## 10-STAR axis targets (pre / post)

Scores match 0.1.0-CONTRACT.md per-milestone delta.

| axis | weight | pre | post | basis |
| --- | --- | --- | --- | --- |
| session_continuity   | 20% | 5 | 6 | A9 user context switch isolation; B9 per-user harness state; cross-user wake bleed test negative control |
| correction_retention | 15% | 5 | 5 | no V9 work — V10 scope |
| procedural_reuse     | 15% | 4 | 4 | no V9 work — V10 scope |
| cross_harness        | 15% | 4 | 6 | C9 multi-user multi-harness flip; D9 identity collision test; E9 scope boundary adversarial suite (8 scenarios) |
| raw_retrieval        | 15% | 7 | 7 | no V9 work — stable |
| token_efficiency     | 10% | 5 | 5 | no V9 work — stable |
| trust_provenance     | 10% | 6 | 6 | no V9 work — stable |

**Composite: 5.10 → 5.60** (weighted arithmetic, zero-generosity regrade).

## Axis ownership and non-goals

Per 0.1.0-AXIS-OWNERSHIP.md:
- **Owns**: SC +1 (5→6), CH +2 (4→6)
- **Integrates**: (none)
- **Non-goals**: CR, PR, RR, TE, TP (explicit non-touch)

## Phases

See `ROADMAP.md` → "V9: Multi-user / Team". Phase docs at `docs/phases/v9/phase-{a9..g9}-*.md`.

- **A9** Per-user harness state isolation — wake reconstructs only user A's focus, not user B's.
- **B9** Cross-user negative tests — read path: user A memory invisible to B unless explicitly shared; write path: agent cannot write on behalf of other agent.
- **C9** Multi-user multi-harness flip — user A (claude-code) → user B (codex) → user A (claude-code), truth conserved, zero A/B bleed.
- **D9** Identity collision + adversarial suite — agent_id spoofing test, scope escalation test, per-scope retention override test.
- **E9** Correction provenance across users — correction by user A in shared scope, user B sees it with A's attribution; correction scope boundary test.
- **F9** Pre-ship harness dry-run — G9 runs full adversarial suite; passes = V9 closed and pre-ship window opened.
- **G9** Multi-user adversarial gate harness — 8-scenario suite from federated-memory-visibility contract, zero leaks, zero collisions.

## 10-STAR axis harness assertions (required for axis credit)

| axis | concrete assertion | fixture | phase |
| --- | --- | --- | --- |
| session_continuity   | wake in user B context does not surface user A's focus bucket; cross-user switch survives round-trip without bleed | shared/multi-user/session-ua-ub-ua.jsonl | A9 + C9 |
| cross_harness        | user A (claude-code) correction visible in user B (codex) lookup within same workspace; user B (codex) correction visible in user A (claude-code) after round-trip | shared/multi-user/flip-ua-ub-ua.jsonl | C9 |

## Completion gate

Multi-user adversarial suite (G9 harness, federated-memory-visibility.md, 8 scenarios):

1. **Cross-user read leak** — agent B queries shared scope, user A private memory invisible.
2. **Cross-user write escape** — agent B cannot write with user A's agent_id.
3. **Correction provenance preservation** — correction by user A shows A's name in user B's audit trail.
4. **ID collision** — two users, same claim hash, dedup correctly to both; no silent overwrite.
5. **Agent_id spoofing** — agent_id field immutable post-insert; cannot laterally escalate to other agent.
6. **Scope escalation** — Local→Project promotion requires explicit API, never automatic on read/write.
7. **Per-scope retention override** — Workspace-scoped correction does not leak to Global scope.
8. **Multi-user flip test** — user A (harness 1) → user B (harness 2) → user A (harness 1), canonical consistent, zero A/B bleed on wake.

Evidence: recorded trace + G9 harness NDJSON + 8 adversarial scenario results + regenerated composite in `docs/verification/MEMD-10-STAR.md` via G9 scorecard regenerator.

## Evidence

- A9/B9/D9 substrate tests: `scripts/verify/v9-adversarial-suite.sh` reruns
  `cargo test -p memd-server a9`, `b9`, and `d9`.
- F9 dry-run: `docs/verification/v9-runs/f9-dry-run.ndjson`
- G9 proof: `docs/verification/v9-proof-runs/2026-05-05-adversarial-suite.ndjson`
- G9 summary: `docs/verification/v9-proof-runs/2026-05-05-adversarial-suite.md`
- Shared fixtures: `crates/memd-client/fixtures/shared/multi-user/`
- Matrix: `docs/contracts/federated-visibility-matrix.json`

## Pre-ship checklist (V9 close = release-candidate dry-run entry)

- [x] G9 runs full adversarial suite with zero failures (all 8 scenarios pass).
- [x] G9 cross-user flip test: user A → B → A round-trip, canonical never branches.
- [x] G9 negative controls firing: inject scope escalation → test catches it; drop agent_id validation → test catches it.
- [x] Scorecard regenerator strict-mode: cannot score CH > 6 without G9 adversarial proof.
- [x] federated-memory-visibility.md contract in force during G9; any spec contradiction fails review.
- [x] Release artifact: `docs/verification/v9-proof-runs/2026-05-05-adversarial-suite.ndjson` signed by G9 gate.

## Non-goals

- enterprise SSO (V11+ scope)
- end-to-end encryption at rest (V10+ scope)
- ACLs beyond 3-axis model (scope, visibility, agent_id)
- cross-organization sharing (0.1.0 non-goal)

## Changelog

- 2026-04-22 opened.
- 2026-04-22 revised: composite_pre 8.5 → 5.10 (aligned to contract), composite_target 9.0 → 5.60; axes_lifted SC +1, CH +2 (contract binding); non-goals explicit (CR, PR, RR, TE, TP); axis ownership table added; adversarial suite 8-scenario list added; pre-ship checklist added; per-axis harness assertions added; phases a9..g9 outline added.
