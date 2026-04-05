# Phase 33 Context: `v5` Peer Coordination MCP Foundations

## Why This Phase Exists

The repo now has backend-brokered peer messages, claims, claim transfer,
heartbeats, and awareness. The next gap is exposing that coordination substrate
as a first-class MCP/agent integration surface instead of only CLI and raw HTTP
primitives.

## Inputs

- backend-brokered peer messages and claims
- targeted session handoffs and assignment-friendly claim transfer
- user goal of Claude-peers-style simultaneous coworking across terminals

## Constraints

- keep the backend as the coordination source of truth
- expose only stable primitives through MCP
- preserve session identity and claim safety

## Target Outcome

The next phase should expose the existing coordination substrate through a
peer-oriented MCP contract so multiple agents can coordinate natively.
