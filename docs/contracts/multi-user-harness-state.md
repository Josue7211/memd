---
contract: multi-user-harness-state
version: 0.1
status: active
introduced_in: V9 A9
depends_on: docs/contracts/federated-memory-visibility.md
---

# Multi-User Harness State Contract

V9 separates three identities that earlier milestones often collapsed:

| Field | Meaning | Mutable |
| --- | --- | --- |
| `user_id` | Human/team principal that owns private state. | No after insert. |
| `agent_id` / `source_agent` | Agent instance that wrote the record. | No after insert. |
| `harness_preset` | Runtime harness family, such as `codex` or `claude-code`. | No after insert. |

`user_id` is the privacy boundary. `agent_id` is attribution. `harness_preset`
is compatibility/provenance. No one field can substitute for all three once V9
multi-user mode is active.

## Invariants

1. Private memory is visible only to the same `user_id`. Legacy rows without
   `user_id` fall back to current source-agent ownership rules.
2. Workspace memory is visible only when the caller workspace matches the item
   workspace.
3. Public memory is still bounded by project/namespace route rules.
4. `user_id`, `source_agent`, and `harness_preset` are immutable after insert.
5. Supersede/correct creates a new row and preserves the old row in history.
6. Identical content from two users keeps co-attribution. Dedup must not erase
   either author.
7. Wake and working-memory reconstruction filter by `user_id` before ranking.
8. Lookup never promotes Local/Private state to broader scope as a side effect.

## Required State

Every new persisted memory row SHOULD carry:

```json
{
  "user_id": "user-a",
  "source_agent": "codex@session-123",
  "harness_preset": "codex",
  "workspace": "team-alpha",
  "visibility": "workspace"
}
```

Rows from pre-V9 stores may omit `user_id` and `harness_preset`. Readers must
remain backward-compatible, but V9 tests target new rows with explicit identity.

## Caller Identity

Retrieval and write paths build a caller identity from request/runtime fields:

| Source | Field |
| --- | --- |
| User principal | `user_id` |
| Agent instance | `agent` or `source_agent` |
| Harness family | `harness_preset` |
| Workspace boundary | `workspace` |
| Project boundary | `project` |
| Namespace boundary | `namespace` |

Missing caller identity fails closed for Private records. Missing workspace
fails closed for Workspace records once B9 enforcement is active.

## Wake Contract

Wake, context, and working-memory surfaces must not show another user's private
focus, even if:

- the same agent name is reused,
- the same session id is reused,
- the same project/namespace/workspace is used,
- the other user's record ranks higher by recency or confidence.

Filtering occurs before scoring, FTS fusion, dense expansion, and working-memory
admission.

## Write Contract

Normal writes may only claim the caller identity. A request that tries to write
with another `user_id` or `source_agent` is rejected. Trusted operator override
is out of V9 scope unless a later contract explicitly adds it.

## V9 Proof Hooks

The following fixtures must be replayable:

- `ua-ub-ua-3session.jsonl`
- `flip-ua-ub-ua.jsonl`
- `cross-user-corrections.jsonl`
- `identity-collision-10turn.jsonl`
- `agent-spoofing-negative.jsonl`
- `scope-escalation-negative.jsonl`
- `cross-workspace-leak-negative.jsonl`
- `per-scope-retention-negative.jsonl`

The G9 gate passes only when all fixtures prove zero leaks, zero silent
overwrites, and zero identity escalation.
