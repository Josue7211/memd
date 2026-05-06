---
phase: E12
status: closed
closed: 2026-05-05
axis: cross_harness
evidence: [crates/memd-core/src/interop/mod.rs, docs/contracts/universal-harness-protocol.md, scripts/verify/v12-interop-sota-suite.sh]
---

# E12 MCP Protocol Shim

Closed by the protocol-neutral envelope in `memd_core::interop`.

Exit criteria:

- MCP request/response maps to canonical memory envelope.
- Same query returns same canonical answer as custom typed channel.
- Shim estimate stays below 100 LOC.
- G12 parity report asserts `max_delta <= 0.02`.
