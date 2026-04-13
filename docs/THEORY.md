# memd Theory

`THEORY.md` is the root entrypoint for the theory side of the project.

`ROADMAP.md` is execution truth.
`THEORY.md` is model truth.

## Core Thesis

`memd` is a multiharness second-brain memory substrate for humans and agents.

It exists to make this true:

- read once
- remember once
- reuse everywhere

The system should reduce repeated context rebuilding without reducing quality,
while keeping provenance, correction state, and trust visible.

Current proving-ground harnesses:

- Codex
- OpenCode
- Hermes
- OpenClaw

## Theory Locks

The locked model is:

1. working context
2. session continuity
3. episodic memory
4. semantic memory
5. procedural memory
6. candidate memory
7. canonical memory
8. correction + provenance
9. wake packet compiler
10. hive coordination

Important separations:

- surfaces are not the substrate
- semantic helpers are not truth
- Obsidian is not canonical truth
- atlas is not canonical truth
- wake packet is not the whole memory system

## Surface Lock

The locked shared visible surfaces are:

- `wake.md`
- `mem.md`
- `events.md`

Rules:

- these are shared system surfaces, not per-harness duplicated payloads
- harnesses are adapters over one brain, not separate memory bundles
- Claude Code should load only `wake.md` by default
- `mem.md` and `events.md` are cold-path surfaces and should load only on demand
- atlas/navigation may point into these surfaces, but is not a fourth truth plane

## Evolution Lock

The 10-star system is not only the three visible files.

Underneath them, `memd` should run a Hermes-style improvement loop:

- collect recent memory and gap signals
- run dream/nightly maintenance or bounded autoresearch passes
- validate or reject candidate improvements
- promote validated gains into canonical, semantic, or procedural memory
- refresh the next `wake.md` so the next session wakes smarter without loading more by default

## Root Theory Docs

- [memd-theory-lock-v1.md](./docs/superpowers/specs/2026-04-11-memd-theory-lock-v1.md)
- [memd-10-star-memory-model-v2.md](./docs/superpowers/specs/2026-04-11-memd-10-star-memory-model-v2.md)
- [memd-canonical-theory-synthesis.md](./docs/superpowers/specs/2026-04-11-memd-canonical-theory-synthesis.md)

## Domain Theory Docs

- [memd-ontology-lock-v1.md](./docs/superpowers/specs/2026-04-11-memd-ontology-lock-v1.md)
- [memd-retrieval-theory-lock-v1.md](./docs/superpowers/specs/2026-04-11-memd-retrieval-theory-lock-v1.md)
- [memd-canonical-promotion-theory-lock-v1.md](./docs/superpowers/specs/2026-04-11-memd-canonical-promotion-theory-lock-v1.md)
- [memd-atlas-theory-lock-v1.md](./docs/superpowers/specs/2026-04-11-memd-atlas-theory-lock-v1.md)
- [memd-hive-theory-lock-v1.md](./docs/superpowers/specs/2026-04-11-memd-hive-theory-lock-v1.md)
- [memd-evaluation-theory-lock-v1.md](./docs/superpowers/specs/2026-04-11-memd-evaluation-theory-lock-v1.md)

## Comparative / Teardown Docs

- [mempalace-theory-teardown.md](./docs/superpowers/specs/2026-04-11-mempalace-theory-teardown.md)
- [hermes-theory-teardown.md](./docs/superpowers/specs/2026-04-11-hermes-theory-teardown.md)

## Working Rule

When theory and execution drift:

- update `ROADMAP.md` for execution truth
- update `THEORY.md` and linked theory docs for model truth
- do not create a second competing roadmap
