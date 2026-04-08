# Live Event Compiler Hook And Checkpoint Design

## Goal

Make memd feel alive while the agent works by recording task-state events from
the highest-signal surfaces first:

- `hook capture`
- `checkpoint`

Those events must flow into a bundle-local event log, get compiled into visible
event pages, and refresh the memory surfaces that the agent reads next.

The first slice must prove:

- raw state is read once
- a compact event is appended
- compiled event pages update from that log
- memory pages remain the visible object layer
- no other command surfaces need to change yet

## Why This Exists

The current memory model already has:

- visible memory objects
- compiled memory pages
- a bundle-local event lane
- semantic recall behind the control plane

What it does not yet have is a reliable live path from the agent's working
actions into those event pages. Without that, the system still feels like a
snapshot generator instead of a living memory system.

The goal is not to replace memory objects with events. The goal is to make
events the incremental source that keeps the visible memory fresh.

## Scope

### In Scope

- emit bundle events from `hook capture`
- emit bundle events from `checkpoint`
- append those events to the bundle log
- recompile `MEMD_EVENTS.md`
- recompile `compiled/events/latest.md`
- recompile per-kind and per-item event pages
- keep `MEMD_MEMORY.md` and compiled memory pages as the visible memory layer
- expose the event lane through `memd events`
- keep the implementation bundle-local and deterministic

### Out Of Scope

- server-side streaming event bus
- wiring every command surface into the event compiler
- auto-activating skills from events
- replacing memory objects with the event log
- changing the semantic backend contract
- changing the Obsidian or palace browsing model

## Recommended Approach

Use the existing bundle-local compiler path and wire the live event emission
only into the two strongest entry points first:

- `hook capture`
- `checkpoint`

This is the best first step because it:

- proves the event loop end to end
- touches real task-state changes
- minimizes regression surface
- keeps the compiler deterministic
- avoids inventing a second persistence model

## Design

### 1. Event Record Model

Each live event should be stored as a compact JSONL record under:

- `.memd/state/live-events.jsonl`

Every record should carry enough information to be useful later without
re-reading the full raw source:

- `id`
- `event_type`
- `summary`
- `source`
- `recorded_at`
- `project`
- `namespace`
- `workspace`
- `focus`
- `pressure`
- `next_recovery`
- `context_pressure`
- `estimated_prompt_tokens`
- `working_records`
- `inbox_items`
- `rehydration_items`
- `refresh_recommended`
- `event_spine`
- `change_summary`
- `recent_repo_changes`
- `handoff_sources`

The record should be derived from the current bundle snapshot, not from a fresh
scan of the whole workspace. That preserves the read-once rule.

### 2. Event Compiler

The compiler should read the JSONL event log and render:

- `.memd/MEMD_EVENTS.md`
- `.memd/compiled/events/latest.md`
- `.memd/compiled/events/<kind>.md`
- `.memd/compiled/events/items/<kind>/<id>.md`

The compiler should also generate the same content for the agent bridge files:

- `.memd/agents/CODEX_EVENTS.md`
- `.memd/agents/CLAUDE_CODE_EVENTS.md`
- `.memd/agents/OPENCLAW_EVENTS.md`
- `.memd/agents/OPENCODE_EVENTS.md`

The compiled pages are the view layer. The JSONL log is the append-only input.

### 3. Live Event Sources

For this first slice, only two commands emit live events:

- `hook capture`
- `checkpoint`

That keeps the first version focused on the strongest task-state changes:

- hook capture = the agent noticed a live change
- checkpoint = the agent recorded durable current-task state

Each emitted event should be written once, then immediately compiled into the
event pages and the import surface.

### 4. Memory Relationship

Memory pages and event pages are related but distinct:

- memory pages are the visible working truth
- event pages are the live activity trail that updates them

The memory page should point at the event lane, not absorb the entire event log
inline. That preserves token efficiency and keeps the object view readable.

### 5. Read Once Rule

The live path must respect this rule:

- read raw source once
- derive a compact event from it
- append the event
- compile from the event log
- reuse compiled memory and event views afterward

The compiler must not reread the whole source tree just to produce a live event
record.

## Error Handling

- If event append fails, the command should still surface the failure clearly.
- If compilation fails after append, the log should remain intact and the error
  should identify the compile stage.
- If the event log is empty, `memd events` should still work and report zero
  records cleanly.
- If the bundle has no event log yet, the compiled event index should render as
  empty instead of failing.

## Verification

The first slice must be verified with:

- unit tests for event record derivation
- unit tests for event log append and compile output
- command tests for `memd events --summary`
- command tests for `memd events --list`
- command tests for `hook capture` and `checkpoint` event emission paths
- workspace test pass

Useful smoke checks:

```bash
cargo test -p memd-client --quiet
cargo test --workspace --quiet
cargo run -p memd-client --bin memd -- events --summary --root .memd
cargo run -p memd-client --bin memd -- events --list --root .memd
```

## Success Criteria

The slice is successful if:

- `hook capture` and `checkpoint` each emit a bundle event
- the event log persists under `.memd/state/live-events.jsonl`
- `memd events` can inspect the compiled event lane
- the compiled event pages are regenerated from the log
- the existing memory pages stay visible and usable
- tests pass without widening the scope to every command

## Follow-On Work

After this slice ships, the next steps are:

- wire more live command surfaces into the same compiler path
- add provenance and history to the visible memory object pages
- use repeated event patterns to propose skills
- sandbox-test proposed skills before activation
- keep LightRAG as the recall engine behind the visible object layer

