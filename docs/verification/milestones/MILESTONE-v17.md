---
milestone: v17
name: Cross-User Routine Economy
status: code_complete_dogfood_pending
opened: 2026-04-22
depends_on: [v16, ../../verification/1.0.0-CONTRACT.md, ../../verification/1.0.0-AXIS-OWNERSHIP.md]
composite_pre: 9.05
composite_target: 9.35
axes_lifted: [procedural_reuse, cross_harness]
axes_integrated_with: [correction_retention]
---

# Milestone v17 Audit — Cross-User Routine Economy

## Goal

Routine marketplace: discover routines contributed by other memd users
with trust + provenance + per-user reputation. Parameterized routine
generalization (infer variable bindings from example traces). Federation
at scale: thousands of users, per-user isolation preserved + explicit
sharing + zero data leakage. Ships PR 9→10 and CH 9→10.

## 10-STAR axis targets (pre / post)

| axis | weight | pre | post | basis |
| --- | --- | --- | --- | --- |
| session_continuity   | 20% | 10 | 10 | — |
| correction_retention | 15% | 8  | 8  | INT (routines may embed corrections; no credit) |
| procedural_reuse     | 15% | 9  | 10 | **OWNS +1** — marketplace + generalization |
| cross_harness        | 15% | 9  | 10 | **OWNS +1** — cross-user federation at scale |
| raw_retrieval        | 15% | 9  | 9  | — |
| token_efficiency     | 10% | 9  | 9  | — |
| trust_provenance     | 10% | 9  | 9  | — |

**Composite: 9.05 → 9.35**.

## Phases (code complete; dogfood gate open)

- **A17** Routine marketplace schema (routine as content-addressed object + author + version + reputation-signals)
- **B17** Trust layer (per-user reputation; user can block/allowlist authors)
- **C17** Parameterized routine generalization (infer variable bindings from ≥3 example traces)
- **D17** Discovery UI (`memd routines marketplace search/browse/install`)
- **E17** Federation scale test (≥1000 users; per-user isolation preserved)
- **F17** Zero-data-leakage proof (adversarial: user A shares routine citing private memory; shared version strips citations)
- **G17** V17 gate harness (≥30-day marketplace dogfood; ≥5 installed routines across ≥3 users)

## Completion gate

1. Marketplace live with ≥N user accounts (N from V14 telemetry + V16 sync cohort) — **pending real dogfood**.
2. ≥1000-user federation scale test passes (per-user isolation preserved under load) — **passed synthetic**.
3. ≥10 parameterized routines validated (variable bindings inferred from example traces) — **passed**.
4. Zero data leakage in adversarial audit (shared routine strips private citations) — **passed**.
5. Reputation system prevents spam/abuse routines from ranking — **passed in trust policy fixture**.
6. 10-STAR composite regenerated ≥9.35 with PR=10, CH=10 — **provisional passed; final close waits on dogfood**.

## Evidence

- Core substrate: `crates/memd-core/src/v17.rs`
- CLI surface: `memd routines marketplace search|browse|install`
- Proof script: `scripts/verify/v17-routine-marketplace-suite.sh`
- Summary: `docs/verification/v17-proof-runs/2026-05-06-routine-marketplace-suite.md`
- Artifact: `docs/verification/v17-proof-runs/2026-05-06-routine-marketplace-suite.ndjson`

## Non-goals

- Monetization / paid routines (substrate-only; marketplace economics is harness territory)
- Routine execution in sandbox (V17 is discovery + sharing; execution trust is user's harness)
- Routine auto-merge across users (explicit opt-in only)

## Changelog

- 2026-04-22 opened.
- 2026-05-06 code complete. Marketplace schema, trust policy, parameterization, CLI search/install, 1000-user synthetic federation, leakage proof, and V17 proof artifacts landed. Real 30-day marketplace dogfood gate remains open.
