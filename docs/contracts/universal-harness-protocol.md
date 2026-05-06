---
contract: universal-harness-protocol
version: 0.1
owner_phase: E12/G12
status: closed
opened: 2026-05-05
---

# Universal Harness Protocol

V12 binds memd to one protocol-neutral memory envelope. MCP, ACP, and typed
channel shims all translate into the same request/response semantics in
`memd_core::interop`.

## Envelope

Required request fields:

| Field | Rule |
| --- | --- |
| `protocol` | `mcp`, `acp`, `typed_channel`, or `codex_custom`. |
| `harness` | Calling harness label. |
| `workspace_id` | V11 workspace isolation boundary. |
| `operation` | `query`, `read`, or `correction`. |
| `query` | User/harness memory query text. |

Required response fields:

| Field | Rule |
| --- | --- |
| `protocol` | Echoes caller protocol. |
| `harness` | Echoes caller harness. |
| `workspace_id` | Echoes workspace. |
| `content` | Canonical memory answer. |
| `fidelity` | Harness-local confidence, where `1.0` is exact. |

## Parity Rule

For G12, the same workspace query across supported harnesses must have
`max_delta <= 0.02`. The proof runner uses deterministic canonical answers to
prevent protocol-specific formatting from counting as semantic divergence.

## Shim Budget

All V12 shims must remain below 100 LOC. Current core estimates:

- MCP: 68 LOC
- ACP: 72 LOC
- typed channel: 84 LOC
- Codex custom: 76 LOC

## Audit Coupling

Every read/correction/promotion crossing this protocol boundary must be
auditable by H12/J12 signed entries. G12 proves this with two harnesses writing
corrections and both reading the updated value.
