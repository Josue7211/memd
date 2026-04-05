# Summary 46-01: `v5` Policy-Aware Coordination Action Suggestions

## Goal

Help richer operator surfaces move from coordination pressure views to bounded next actions by
surfacing policy-aware suggestions derived from the same state model.

## What Changed

1. Extended the CLI coordination response with policy-aware `suggestions`.
   - Suggestion generation now considers inbox messages, stale-session recovery opportunities, policy conflicts, and missing review/help requests.
   - Suggested actions remain bounded and explicitly do not auto-execute.
2. Added suggestion-aware rendering across coordination summaries and change snapshots.
   - Suggestion slices are included in `--summary` output and are visible in `--view suggestions`.
   - Watch/change alert surfaces now track suggestion count deltas.
3. Added MCP support for suggestion retrieval.
   - Added `coordination_suggestions` tool to expose suggestion lists to richer coworking surfaces.
   - Updated MCP docs for new view options and tool coverage.

## Verification

- `cargo test -p memd-client`
- `node --check integrations/mcp-peer/server.js`

Note:
- `cargo fmt --all -- --check` currently reports pre-existing repository-wide formatting churn, so full repo formatting was intentionally not applied in this pass.

## Result

Richer operator surfaces now receive bounded, policy-aware coordination action suggestions
and can still keep the existing explicit action execution model unchanged.
