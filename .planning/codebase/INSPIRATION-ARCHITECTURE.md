# Inspiration Architecture

This is the product map from external repos to memd.

## 1. Output discipline

Source:
- [JuliusBrussee/caveman](https://github.com/JuliusBrussee/caveman)

Memd subsystem:
- prompt rendering
- status output
- memory-file compaction

Borrow:
- explicit brevity modes
- token budget awareness
- benchmark-first justification for shorter output

## 2. Ingestion and extraction

Source:
- [opendatalab/MinerU](https://github.com/opendatalab/MinerU)

Memd subsystem:
- file ingest
- doc parsing
- mixed-format normalization
- offline / local deployment

Borrow:
- layered parse pipeline
- API / CLI / UI separation
- env template workflow
- local-first deployment shape

## 3. Semantic backend contract

Source:
- [HKUDS/LightRAG](https://github.com/HKUDS/LightRAG)

Memd subsystem:
- optional RAG backend
- per-machine backend URL
- workspace isolation
- retrieval modes

Borrow:
- `.env`-first config
- API root as the integration boundary
- workspace-scoped data separation
- query mode / retrieval tuning

## 4. Live coordination

Source:
- [live coordination source](https://github.com/louislva/claude-peers-mcp)

Memd subsystem:
- live session groups
- session handoff
- live memory sharing
- presence / heartbeat

Borrow:
- broker daemon pattern
- registration + heartbeat + cleanup
- push notifications for urgent messages
- summarized live session state

## 5. Turnkey memory + plugin distribution

Source:
- [supermemoryai/supermemory](https://github.com/supermemoryai/supermemory)

Memd subsystem:
- client packaging
- harness-specific plugins
- memory/profile/RAG context stack
- container/project routing

Borrow:
- one API for memory, user profile, connectors, and file processing
- per-harness plugin distribution instead of one generic integration
- auto-recall before turns and auto-capture after turns
- turn-scoped cache so a loop does not pay the same retrieval cost twice
- container tags to route by project, work, or personal scope
- multi-modal extractors as first-class ingestion surfaces
- graph package as a separate browse/view layer instead of hiding everything behind search

## 6. Design inspiration

Source:
- [VoltAgent/awesome-design-md](https://github.com/VoltAgent/awesome-design-md)

Memd subsystem:
- UI inspiration docs
- project design memory
- portable style references

Borrow:
- markdown as design source of truth
- previewable reference bundles
- curated library structure

## 7. Memory palace model

Source:
- [milla-jovovich/mempalace](https://github.com/milla-jovovich/mempalace)

Memd subsystem:
- memory taxonomy
- search / recall
- local project state
- MCP access

Borrow:
- store raw evidence
- structured location model
- verbatim retrieval
- onboarding that learns project/person structure

## 8. Self-evolving knowledge base

Source:
- Karpathy's LLM knowledge-base workflow

Memd subsystem:
- research workspace
- compiled markdown wiki
- Obsidian frontend
- self-check / self-repair loops

Borrow:
- raw ingest directory
- compiled wiki maintained by the model
- markdown + images as the primary corpus
- lint/health-check tooling that improves the corpus over time

## 9. Prompt-defined agent framework

Source:
- [agent0ai/agent-zero](https://github.com/agent0ai/agent-zero)

Memd subsystem:
- behavior policy
- prompt templates
- tool/plugin extensibility
- multi-agent orchestration

Borrow:
- behavior defined in prompts and editable templates
- plugin-based extensibility
- persistent memory with cooperative subagents
- browser and terminal as first-class tools

## 10. Cloud-first self-hostable agent

Source:
- [NousResearch/hermes-agent](https://github.com/NousResearch/hermes-agent)

Memd subsystem:
- adoption path
- onboarding UX
- multi-platform gateway
- self-host deployment

Borrow:
- cloud-first onboarding
- self-host later
- persistent memory across sessions
- Obsidian as one of several working surfaces

## 11. Full terminal-first assistant runtime

Source:
- [anthropics/claude-code](https://github.com/anthropics/claude-code) as reconstructed in `/home/josue/Documents/projects/claude-code-source-build`

Memd subsystem:
- session continuity
- memory truth / freshness
- live coordination
- worktree isolation
- IDE integration
- capability catalogs
- session lifecycle controls
- background maintenance loops

Borrow:
- managed session memory and compaction
- auto-maintenance / consolidation loops
- bridge / remote-control lifecycle
- worktree-aware isolation for parallel work
- IDE integration as part of the live context model
- explicit tool, command, skill, and task catalogs
- native integrations and analytics as visible control planes

## Cross-Repo Synthesis

The product shape these repos imply:

- `caveman` says compress output carefully.
- `MinerU` says extract input carefully.
- `LightRAG` says keep the backend swappable.
- the live coordination source says make memory and coordination live.
- `supermemory` says package the memory system as harness-specific plugins over one API.
- `awesome-design-md` says keep design inspiration portable.
- `mempalace` says store the original truth first, then organize it.
- Karpathy says keep a raw ingest area, compile a wiki, and let the model evolve it.
- `wiki-gen-skill` says every entry must be absorbed somewhere and every page needs a point.
- `agent-zero` says behavior should live in prompts and plugins, not hidden rails.
- `Hermes` says cloud-first onboarding can coexist with self-host later.
- `supermemory` says memory should ship as a shared core plus thin per-harness adapters, with a separate graph surface for inspection.
- Claude Code says the full runtime should keep sessions continuous, memory maintained, worktrees isolated, and live context visible.

Memd should combine all of them:

- short outputs
- strong ingest
- clean API boundary
- live session context
- design memory lanes
- raw evidence retention
- Obsidian-first wiki workflow
- prompt-defined agent behavior
- cloud-first adoption path
