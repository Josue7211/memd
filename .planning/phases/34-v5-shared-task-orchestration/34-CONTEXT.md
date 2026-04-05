# Phase 34 Context: `v5` Shared Task Orchestration

## Why This Phase Exists

The repo now has brokered coordination primitives across the backend, CLI, and
MCP layers. The next gap is moving from low-level peer tools to richer
coworking flows so multiple sessions can coordinate on the same project without
manual claim juggling and message choreography.

## Inputs

- backend-brokered peer messages, claims, claim transfer, and assignments
- MCP-native peer coordination bridge under `integrations/mcp-peer`
- user goal of Claude-peers-style simultaneous coworking without stepping on
  each other

## Constraints

- keep `memd-server` as the coordination source of truth
- preserve session-qualified identity and claim safety
- avoid collapsing separate hot memory lanes into one shared transcript
- keep the first orchestration slice narrow and agent-usable

## Target Outcome

The next phase should turn coordination primitives into a shared-task
orchestration layer with clearer task ownership, help/review flows, and
operator-visible coworking state.
