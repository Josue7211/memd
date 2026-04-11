# Compaction

## Goal

Compaction should preserve working context without collapsing it into a vague summary.

The output must be small, but it must not drop the state needed to continue work correctly.

## Principles

1. Preserve exact anchors
- file paths
- commands
- IDs
- branch names
- hostnames
- IPs
- model names

2. Preserve open loops
- unresolved questions
- pending actions
- blocked items
- verification steps

3. Preserve hard constraints
- things that must not change
- assumptions that control behavior
- system limits

4. Compress only low-value chatter
- repetition
- intermediate dead ends
- conversational filler

5. Prefer structure over prose
- use fields, not paragraphs
- keep machine-readable records
- avoid lossy paraphrase when exact wording matters

## Suggested Packet Shape

```json
{
  "session": {
    "project": "memd",
    "agent": "codex",
    "task": "build memory manager"
  },
  "goal": "Preserve memory without token waste",
  "hard_constraints": [
    "compact retrieval only",
    "no transcript dumps",
    "cross-project reuse must stay scoped"
  ],
  "active_work": [
    "verification worker scans stale canonical items",
    "Claude and Codex share the same memd control plane"
  ],
  "decisions": [
    {
      "id": "decision-1",
      "text": "Use Rust for the memory manager and client SDKs"
    }
  ],
  "open_loops": [
    {
      "id": "loop-1",
      "text": "Should compaction store raw candidate packets or only promoted summaries?",
      "status": "open"
    }
  ],
  "exact_refs": [
    {
      "type": "file",
      "value": "crates/memd-server/src/main.rs"
    },
    {
      "type": "command",
      "value": "cargo check"
    }
  ],
  "next_actions": [
    "Define the promotion boundary for compaction output",
    "Add a compact packet serializer",
    "Keep client integrations off until the packet contract is stable"
  ],
  "do_not_drop": [
    "scope",
    "project",
    "exact refs",
    "open loops",
    "hard constraints"
  ],
  "memory": {
    "retrieval_order": ["local", "synced", "project", "global"],
    "records": [
      {
        "id": "00000000-0000-0000-0000-000000000000",
        "record": "id=... | stage=canonical | scope=project | kind=fact | status=active | ... | c=..."
      }
    ]
  }
}
```

## What Must Never Happen

- turn the packet into a prose summary
- drop open loops
- lose exact paths or commands
- merge project-local and global context
- rewrite concrete values into approximate language
- drop the compact memory payload itself

## Where It Fits

- compaction output feeds the memory manager
- the memory manager decides whether to store it as candidate memory, synced state, or canonical memory
- clients should not invent their own compaction rules

## Runtime Forms

- `memd compact` defaults to a JSON inspection packet for debugging
- `memd compact --wire` emits the smaller runtime wire format
- `memd compact --spill` emits the durable memory spill batch
- `memd compact --spill --spill-transient` also includes short-lived session state
- `memd compact --spill --apply` writes spill candidates directly into `memd`
- the wire format is the one that should be optimized for token and KV-cache efficiency

## Budget Rules

- preserve exact refs before anything else
- preserve open loops before long lists
- truncate long lists with explicit `... +N` markers
- cap individual line length
- keep the wire format deterministic
- never silently drop state
- spill durable state before truncation when possible
- keep transient state out of spill output unless explicitly requested
