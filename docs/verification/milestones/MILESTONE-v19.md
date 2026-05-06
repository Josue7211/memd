---
milestone: v19
name: Zero-Knowledge Provenance + Full Crypto Audit
status: code_complete_external_auditor_pending
opened: 2026-04-22
depends_on: [v18, ../../verification/1.0.0-CONTRACT.md, ../../verification/1.0.0-AXIS-OWNERSHIP.md]
composite_pre: 9.50
composite_target: 9.75
axes_lifted: [correction_retention, trust_provenance]
axes_integrated_with: []
---

# Milestone v19 Audit — Zero-Knowledge Provenance + Full Crypto Audit

## Goal

ZK provenance proofs: user can prove a correction was applied without
revealing the correction content. Tamper-evident audit trail signed end-
to-end (extending V12 ed25519 foundation). Compliance-grade audit UI
with multi-party attestation. Ships TP 9→10 and CR 9→10 (V19 closes
both axes at ceiling).

## 10-STAR axis targets (pre / post)

| axis | weight | pre | post | basis |
| --- | --- | --- | --- | --- |
| session_continuity   | 20% | 10 | 10 | — |
| correction_retention | 15% | 9  | 10 | **OWNS +1** — ZK replayable correction proofs |
| procedural_reuse     | 15% | 10 | 10 | — |
| cross_harness        | 15% | 10 | 10 | — |
| raw_retrieval        | 15% | 9  | 9  | — |
| token_efficiency     | 10% | 9  | 9  | — |
| trust_provenance     | 10% | 9  | 10 | **OWNS +1** — ZK provenance + multi-party attestation |

**Composite: 9.50 → 9.75**.

## Phases (code complete; external auditor gate open)

- **A19** ZK proof system selection (groth16 / plonk / custom; circuit for "correction was applied")
- **B19** Circuit implementation + proof generation for correction-applied claim
- **C19** Verifier tool (`memd audit verify-zk <proof>` — no memd instance needed; standalone)
- **D19** Multi-party attestation (two-of-three signing for high-stakes corrections)
- **E19** Compliance-grade audit UI (time-ordered; signed; query-able; exportable for auditors)
- **F19** Third-party ZK replay (external auditor verifies corrections without seeing content)
- **G19** V19 gate harness (≥10 ZK proofs verified by external tool; compliance dogfood)

## Completion gate

1. ≥10 correction-applied ZK proofs generated and verified by external standalone tool — **local standalone verifier passed; external auditor still pending**.
2. Multi-party attestation flow operational (two-of-three signing proven end-to-end) — **passed**.
3. Compliance audit UI passes external auditor smoke test (rich enough for SOC2-lite scenario) — **pending external auditor**.
4. Tamper-evidence: post-hoc audit-log modification detected by external verifier — **passed**.
5. 10-STAR composite regenerated ≥9.75 with TP=10, CR=10 — **provisional passed; final close waits on auditor evidence**.

## Evidence

- Core substrate: `crates/memd-core/src/v19.rs`
- CLI verifier: `memd audit verify-zk <proof>`
- Proof script: `scripts/verify/v19-zk-provenance-suite.sh`
- Summary: `docs/verification/v19-proof-runs/2026-05-06-zk-provenance-suite.md`
- Artifact: `docs/verification/v19-proof-runs/2026-05-06-zk-provenance-suite.ndjson`

## Non-goals

- Regulated-industry certification (HIPAA/SOC2/PCI) — 2.0.0 deployment-partner work
- ZK proofs for non-correction claims (V19 scope is corrections only)

## Changelog

- 2026-04-22 opened.
- 2026-05-06 code complete. Pragmatic commitment proof substrate, `audit verify-zk`, two-of-three attestation, tamper evidence, and V19 proof artifacts landed. External auditor smoke gate remains open.
