# Inspiration Lane

Goal: keep a durable, source-linked memory lane of repos that shape memd.

Rule:
- quick lane = what the repo is for, what to borrow
- deep lane = what to copy, what to avoid, why it matters for memd
- never copy code verbatim; reimplement patterns cleanly
- search it with `memd inspiration --query "..."`

Related docs:
- `INSPIRATION-ARCHITECTURE.md` = how the repos map onto memd subsystems
- `INSPIRATION-BACKLOG.md` = the work items that fall out of the extraction
- `INSPIRATION-MATRIX.md` = file-by-file extraction matrix and notes
- `INSPIRATION-DOCTRINE.md` = the final short policy version

## Quick Recall

| Repo | Role for memd | Borrow |
|---|---|---|
| [JuliusBrussee/caveman](https://github.com/JuliusBrussee/caveman) | token discipline and response compression | intensity levels, brevity defaults, benchmark culture, memory-file compression |
| [opendatalab/MinerU](https://github.com/opendatalab/MinerU) | ingestion and document extraction engine | OCR + structured extraction, CLI/API/Docker shape, env templating, offline deployment |
| [HKUDS/LightRAG](https://github.com/HKUDS/LightRAG) | semantic backend contract | API-first split, `.env`-driven config, workspace isolation, server/UI separation |
| [louislva/claude-peers-mcp](https://github.com/louislva/claude-peers-mcp) | live peer coordination layer | broker daemon, peer discovery, heartbeat, summary propagation, push messages |
| [anthropics/claude-code](https://github.com/anthropics/claude-code) / source build | full terminal-first assistant runtime | session continuity, memory compaction, IDE integration, worktrees, bridge sessions, tool catalogs, background consolidation |
| [supermemoryai/supermemory](https://github.com/supermemoryai/supermemory) | full-context memory product and plugin distribution layer | one API for memory + RAG + profiles, per-harness plugins, auto-recall, auto-capture, container tags, multi-modal extractors |
| [VoltAgent/awesome-design-md](https://github.com/VoltAgent/awesome-design-md) | inspiration MD library | portable `DESIGN.md`, preview artifacts, curated reference structure |
| [milla-jovovich/mempalace](https://github.com/milla-jovovich/mempalace) | memory palace model | verbatim storage, wing/room/drawer taxonomy, MCP search/write, local-first recall |
| Karpathy KB pattern | self-evolving research wiki | raw/ ingest, compiled wiki, Obsidian frontend, lint + repair loops |
| [farzaa/wiki-gen-skill](https://gist.github.com/farzaa/c35ac0cfbeb957788650e36aabea836d) | personal knowledge wiki skill | ingest, absorb, query, cleanup, breakdown, status |
| [agent0ai/agent-zero](https://github.com/agent0ai/agent-zero) | prompt-driven general agent | prompt-owned behavior, dynamic tools, multi-agent cooperation, persistent memory |
| [NousResearch/hermes-agent](https://github.com/NousResearch/hermes-agent) | self-improving multi-platform agent | persistent memory, platform gateways, cloud-first then self-host, Obsidian integration |

## Deep Research

### caveman

- What to copy:
  - explicit output budgets
  - lite/full/ultra modes
  - short, exact technical language
  - benchmark-driven proof that brevity can help
- What to avoid:
  - gimmicky tone if it obscures meaning
  - compression without validation
- Memd fit:
  - use for prompt compression, status output, and memory-file compaction

### MinerU

- What to copy:
  - multi-stage ingestion pipeline
  - clean separation of parser, API, demo UI, and deployment
  - config via env/template files
  - offline-friendly deployment story
- What to avoid:
  - heavy environment assumptions in the core product
  - overcoupling memd to one backend model or hardware profile
- Memd fit:
  - use as the ingestion/extraction spine for files, docs, and mixed formats

### LightRAG

- What to copy:
  - `.env` first configuration
  - `workspace` isolation
  - API base as the real integration point, not the web UI
  - query modes and retrieval tuning
- What to avoid:
  - hardcoded hostnames
  - UI-path confusion like `/webui/` being treated as the backend
  - config sprawl that makes onboarding brittle
- Memd fit:
  - use as the semantic backend pattern for local, Tailscale, and later Docker setups

### claude-peers-mcp

- What to copy:
  - local broker with simple state store
  - peer registration + heartbeat + cleanup
  - summary broadcast so sessions know each other’s context
  - immediate push channel for time-sensitive messages
- What to avoid:
  - localhost-only assumptions once memd needs cross-machine use
  - single-client coupling
- Memd fit:
  - use for live coordination, peer awareness, and session-to-session context handoff

### Claude Code source build

- Source:
  - [anthropics/claude-code](https://github.com/anthropics/claude-code) as reconstructed in `/home/josue/Documents/projects/claude-code-source-build`
- What to copy:
  - session continuity as a first-class runtime concern
  - managed session memory and compaction loops
  - worktree-aware isolation for parallel work
  - bridge / remote-control session lifecycle
  - IDE integration as part of live context
  - explicit tool, command, and skill catalogs
  - background consolidation and auto-maintenance
  - task taxonomy that distinguishes local, remote, teammate, and maintenance flows
- What to avoid:
  - treating agent teams as the whole product
  - letting session state and truth get mixed without explicit roles
  - hidden side effects that are hard to inspect or recover
  - feature-flag sprawl without clear product boundaries
- Memd fit:
  - use as the source for `memd`'s continuity + truth + coordination vision
  - use as the model for visible capability catalogs and lifecycle operations
  - use as the model for background maintenance loops that improve memory over time
  - use as the model for editor-aware, worktree-aware, session-aware memory state

### supermemory

- What to copy:
  - one API for memory, RAG, user profiles, connectors, and file processing
  - plugin-per-harness distribution model
  - auto-recall before turns and auto-capture after turns
  - container tags for work/personal/project partitioning
  - multi-modal extractors for PDFs, images/OCR, video/transcription, and code AST chunking
  - consumer app plus MCP plus CLI-style integrations
- What to avoid:
  - cloud lock-in as the only path
  - API-key friction becoming the default onboarding flow
  - over-centralizing the product so local-first/self-host becomes an afterthought
- Memd fit:
  - use as the packaging model for harness-specific plugins and one-click adoption
  - use as the product shape for memory + recall + profile + connectors in one context stack
  - use as the reference for project/container routing and turnkey client integrations

### awesome-design-md

- What to copy:
  - design inspiration as markdown, not screenshots only
  - preview pages alongside the source doc
  - a curated library with named roles
- What to avoid:
  - treating it like surface styling only
  - mixing inspiration docs with implementation code
- Memd fit:
  - use as the inspiration-MD pattern for project-specific UI/design memory

### mempalace

- What to copy:
  - store raw conversation evidence
  - palace taxonomy: wing / room / hall / drawer
  - MCP tools for search, status, add, delete
  - local-first search with metadata filters
  - onboarding that learns project/person structure
- What to avoid:
  - overclaiming compression wins
  - relying on summaries when raw evidence matters
- Memd fit:
  - use as the “store everything, make it findable” reference

### Karpathy KB pattern

- What to copy:
  - raw/ directory for source ingest
  - compiled wiki as markdown files
  - backlinks and concept pages generated by the model
  - Obsidian as the read/edit frontend
  - health checks, linting, and repair loops
- What to avoid:
  - hand-editing everything manually
  - treating wiki output as a one-shot artifact
  - losing image/source linkage
- Memd fit:
  - use as the self-evolving research and knowledge-base model

### farzaa/wiki-gen-skill

- What to copy:
  - ingest / absorb / query / cleanup / breakdown / status lifecycle
  - raw entries compiled into wiki articles
  - index/backlink discipline
  - explicit anti-cramming and anti-thinning rules
- What to avoid:
  - diary-log articles that never become concepts
  - weak article boundaries
  - missing backlinks
- Memd fit:
  - use as the direct template for a personal/project knowledge wiki workflow

### agent-zero

- What to copy:
  - prompt-owned framework behavior
  - everything configurable through prompts and plugins
  - persistent memory and multi-agent cooperation
  - browser/tool extensibility
- What to avoid:
  - hardcoded rails where the user should control policy
  - hidden behavior that cannot be edited
- Memd fit:
  - use as the “agent behavior comes from prompts, not code” inspiration

### Hermes Agent

- What to copy:
  - cloud-first onboarding with self-host later
  - persistent memory across sessions
  - multi-platform gateways like Obsidian, Slack, and CLI
  - self-improving skill surface
- What to avoid:
  - setup friction as the default path
  - forcing infra complexity before value is proven
- Memd fit:
  - use as the product packaging model for easier adoption without losing self-host control

## Max Extraction Standard

For every reference repo, capture these five things:

1. role
2. borrow
3. avoid
4. fit for memd
5. follow-up question or test

If a repo does not produce all five, it is not fully extracted yet.

## Why These Repos Matter Together

- `caveman` gives memd the token budget mindset.
- `MinerU` gives memd the extraction spine.
- `LightRAG` gives memd the backend contract.
- `claude-peers-mcp` gives memd live coordination.
- `claude-code` gives memd the broader terminal-runtime pattern: continuity, compaction, worktrees, IDE, bridge, and task taxonomy.
- `awesome-design-md` gives memd design inspiration as portable markdown.
- `mempalace` gives memd the memory-palace mental model and raw retention bias.
- Karpathy says keep a raw ingest area, compile a wiki, and let the model evolve it.
- `wiki-gen-skill` says every entry must be absorbed somewhere and every page needs a point.
- `agent-zero` says behavior should live in prompts and plugins, not hidden rails.
- `Hermes` says cloud-first onboarding can coexist with self-host later.

Together they point to the product:
- capture more
- compress less blindly
- retrieve with provenance
- coordinate across sessions
- keep the UI contract portable
- keep the backend swappable
- make the wiki self-maintaining
- make the agent editable through prompts
