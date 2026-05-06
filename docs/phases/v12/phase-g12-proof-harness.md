---
phase: G12
status: closed
closed: 2026-05-05
axis: procedural_reuse,cross_harness,trust_provenance
evidence: [crates/memd-core/src/v12.rs, scripts/verify/v12-interop-sota-suite.sh, docs/verification/v12-proof-runs/2026-05-05-interop-sota-suite.ndjson]
---

# G12 Proof Harness

Closed by `memd_core::v12::run_v12_proof` and
`scripts/verify/v12-interop-sota-suite.sh`.

Exit criteria:

- Routine library composition, inheritance, export/import pass.
- Protocol parity has `max_delta <= 0.02`.
- Dual-harness correction session sees atomic state updates.
- Signed audit export verifies and tamper negative control fires.
- 10-STAR composite regenerates to `7.75/10`.
