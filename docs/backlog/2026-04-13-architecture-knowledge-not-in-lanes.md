# Architecture Knowledge Not Stored in Memory Lanes

status: open
severity: high
phase: Phase I
opened: 2026-04-13

## Problem

memd's own architecture, theory locks, and design principles are not stored as
durable truth in memory lanes. When an agent session starts, the model has no
memd-internal knowledge beyond what wake.md provides — it doesn't know:

- Wake vs resume distinction (wake = memory boot, always first; resume = continue after break)
- The live loop stages (capture → working context → continuity → episodic → semantic → procedural → compile)
- Theory locks (working context, session continuity, episodic/semantic/procedural/candidate/canonical)
- Hot vs cold surfaces (wake.md hot, mem.md/events.md cold)
- Correction + provenance model
- Hive coordination model
- Lane architecture (fact, decision, preference, live_truth, status, procedure)

This means every new session must re-read DESIGN.md, THEORY.md, and architecture.md
to understand how memd works — defeating the purpose of memd as a memory substrate.

## Root Cause

The init pipeline and wake compiler don't seed architecture knowledge into lanes.
There's no mechanism to store "meta-knowledge about memd itself" as durable facts
that surface in wake packets.

## Impact

- Agent sessions waste tokens re-reading design docs
- Agents make wrong assumptions about memd behavior (e.g., confusing wake/resume)
- Debugging memd issues requires re-learning the architecture each time
- Users have to manually explain memd to the agent ("you don't understand how it works")

## Fix Plan

1. Store core architecture facts via `memd remember --kind fact --tag memd-architecture`:
   - Wake vs resume semantics
   - Live loop stages
   - Theory lock names and purposes
   - Hot/cold surface definitions
   - Lane types and their purposes
2. Ensure wake packet compiler can surface memd-architecture tagged facts
3. Consider a dedicated `memd-meta` kind or tag for self-referential knowledge
4. Add architecture seeding to the init pipeline so fresh bundles start with this knowledge
