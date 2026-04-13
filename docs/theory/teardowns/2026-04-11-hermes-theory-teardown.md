# Hermes Theory Teardown for memd

## Why This Exists

`Hermes` matters because it pushes on the side `mempalace` does not:

- always-on operation
- procedural memory
- scheduled automation
- cross-platform continuity
- checkpoint and rollback safety

We need to know what to keep and what to reject for `memd`.

## What We Inspected

Local repo inspected:

- `../hermes-agent`

Docs and code read:

- `README.md`
- `skills/autonomous-ai-agents/hermes-agent/SKILL.md`
- `agent/memory_manager.py`
- `agent/memory_provider.py`
- `cron/__init__.py`
- `tools/checkpoint_manager.py`

## Hard Findings

### 1. Hermes is strongest as an always-on agent shell, not a memory substrate

Its strongest ideas are:

- skills as procedural memory
- cron-driven unattended work
- multi-platform gateway continuity
- checkpoints and rollback
- profile isolation

This is very valuable.

But it is a different center of gravity than `memd`.

Hermes is:

- an agent runtime with memory features

`memd` wants to be:

- the memory substrate under many runtimes

## 2. Hermes treats procedural memory as first-class

This is the biggest thing `memd` should steal.

Hermes explicitly leans on:

- learning reusable procedures as skills
- improving skills over time
- persisting workflow knowledge

That maps directly to our theory:

- procedural memory must be a native memory kind

## 3. Hermes has the right instinct about always-on loops

From docs and cron code:

- scheduled jobs
- isolated sessions
- unattended execution
- gateway/service model

This proves a real architectural point:

- overnight memory work should not be fake branding
- it should run on actual always-on loops

This supports:

- dream
- autodream
- autoresearch
- autoevolve

as real operational systems, not aspirational slogans.

## 4. Hermes memory is provider-oriented, not theory-oriented

`memory_manager.py` and `memory_provider.py` show:

- built-in memory always present
- one external provider allowed
- prefetch before turn
- sync after turn
- optional hooks for compression, delegation, session end

This is good because:

- it makes memory pluggable
- it wires memory into the turn lifecycle

It is weak because:

- memory is framed as provider plumbing
- not as a precise typed-memory model
- not as a canonical truth control plane

Lesson:

- integration hooks are good
- provider abstraction alone is not the theory

## 5. Hermes is good at continuity, but not yet a human-owned second brain

Hermes supports continuity through:

- sessions
- profiles
- gateways
- memory plugins
- skill accumulation

But the conceptual owner is still closer to:

- the agent instance

`memd` needs a stronger stance:

- the human owns the memory substrate
- harnesses route through it

## 6. Checkpoints are a major insight

`tools/checkpoint_manager.py` is important.

It gives:

- automatic snapshots before mutation
- rollback capability
- low-friction safety

This is not only an ops feature.

For `memd` theory it implies:

- important memory systems should preserve recoverability
- memory and action should be linked to reversible state

This likely belongs in:

- procedural memory
- provenance
- correction loops

## What memd Should Steal

### Steal 1: Procedural memory as a first-class thing

Not as side docs.
Not as plugin afterthought.

### Steal 2: Always-on loop reality

Nightly systems must run on real schedules and real background agents.

### Steal 3: Lifecycle hooks

Useful hooks include:

- pre-turn prefetch
- post-turn sync
- session-end extraction
- pre-compress extraction
- delegation observation

### Steal 4: Checkpoint and rollback thinking

Memory should connect to recoverability, not just retrieval.

### Steal 5: Cross-platform continuity

One substrate should support:

- terminal
- messaging
- IDE
- background agents

## What memd Should Reject

### Reject 1: Agent-first ownership model

`memd` should be human-owned substrate first.

### Reject 2: Provider abstraction as the core theory

Provider abstraction is implementation shape.
Not the memory model.

### Reject 3: Memory as mostly profile-local

Profiles and isolated sessions are useful.
But `memd` needs a stronger shared-memory story across harnesses.

### Reject 4: Skills alone as procedural memory

Skills are one surface of procedural memory.
Not the whole thing.

Procedural memory should also include:

- operating policies
- learned routing behavior
- recovery patterns
- task execution habits

## Theory Upgrades for memd Triggered by This Teardown

### Upgrade 1

Procedural memory is mandatory, not optional.

### Upgrade 2

Overnight evolution must be attached to real always-on infrastructure.

### Upgrade 3

Memory lifecycle hooks should be first-class in the substrate:

- prefetch
- sync
- compress-boundary extraction
- session-end extraction
- delegation capture

### Upgrade 4

Checkpoint and rollback should influence how memory tracks risky changes and recovery knowledge.

## Practical Conclusion

`Hermes` proves that:

- procedural learning matters
- always-on systems matter
- scheduled memory work matters
- agent continuity needs real infrastructure

But it does not fully solve:

- typed memory ontology
- canonical truth control plane
- multiharness second-brain ownership
- memory atlas
- raw-truth-first retrieval at `mempalace` strength

## Final Verdict

`Hermes` is strongest as:

- procedural-memory inspiration
- always-on loop inspiration
- runtime-hook inspiration

`memd` should steal those strengths and combine them with:

- raw-first truth retention
- typed memory kinds
- canonical memory
- memory atlas
- human-owned multiharness continuity
