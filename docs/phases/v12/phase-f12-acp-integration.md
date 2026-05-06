---
phase: F12
status: closed
closed: 2026-05-05
axis: cross_harness
evidence: [crates/memd-core/src/interop/mod.rs, docs/contracts/universal-harness-protocol.md, scripts/verify/v12-interop-sota-suite.sh]
---

# F12 ACP Integration

Closed as protocol-parity integration. ACP uses the same V12 envelope and shim
budget as MCP/custom typed-channel.

Exit criteria:

- ACP protocol variant exists in core.
- ACP shim estimate stays below 100 LOC.
- ACP does not add a separate axis claim.
- G12 universal parity covers protocol-neutral behavior.
