# Inspiration Matrix

File-by-file extraction notes for the main references.

## caveman

- Repo: [JuliusBrussee/caveman](https://github.com/JuliusBrussee/caveman)
- Files:
  - `caveman/SKILL.md`
  - `caveman-compress/README.md`
  - `benchmarks/run.py`
- Extract:
  - output budgets
  - lite/full/ultra modes
  - compression-with-validation flow
  - benchmark table culture
- Reuse in memd:
  - prompt renderer
  - `memd status`
  - memory-file compression

## MinerU

- Repo: [opendatalab/MinerU](https://github.com/opendatalab/MinerU)
- Files:
  - `README.md`
  - `docker/compose.yaml`
  - `mineru.template.json`
  - `docs/requirements.txt`
- Extract:
  - parse pipeline
  - Docker-backed deploy shapes
  - env templating
  - offline support
- Reuse in memd:
  - ingestion pipeline
  - backend setup docs
  - config templates

## LightRAG

- Repo: [HKUDS/LightRAG](https://github.com/HKUDS/LightRAG)
- Files:
  - `docs/LightRAG-API-Server.md`
  - `docs/Algorithm.md`
  - `config.ini.example`
  - `lightrag/lightrag.py`
- Extract:
  - API/server split
  - `.env` precedence
  - workspace isolation
  - retrieval config surface
- Reuse in memd:
  - semantic backend adapter
  - per-machine URL config
  - retrieval tuning and isolation

## claude-peers-mcp

- Repo: [louislva/claude-peers-mcp](https://github.com/louislva/claude-peers-mcp)
- Files:
  - `server.ts`
  - `broker.ts`
  - `shared/types.ts`
- Extract:
  - broker lifecycle
  - peer registration + heartbeat
  - summary propagation
  - message push path
- Reuse in memd:
  - session peer model
  - live handoff layer
  - coordination surface

## supermemory

- Repo: [supermemoryai/supermemory](https://github.com/supermemoryai/supermemory)
- Files:
  - `README.md`
  - `CLAUDE.md`
  - `packages/tools/src/shared/*`
  - `packages/tools/src/openai/*`
  - `packages/tools/src/mastra/*`
  - `packages/tools/src/vercel/*`
  - `packages/tools/src/claude-memory.ts`
  - `packages/ai-sdk/src/*`
  - `packages/openai-sdk-python/src/supermemory_openai/*`
  - `packages/agent-framework-python/src/supermemory_agent_framework/*`
  - `packages/pipecat-sdk-python/README.md`
  - `packages/memory-graph/src/*`
  - `apps/browser-extension/*`
  - `apps/docs/concepts/memory-vs-rag.mdx`
  - `apps/docs/concepts/super-rag.mdx`
  - `apps/docs/concepts/how-it-works.mdx`
- Extract:
  - one shared memory core with thin harness adapters
  - turn-scoped memory cache to avoid repeat calls in one interaction
  - auto-recall before turns and auto-capture after turns
  - profile + search + conversation endpoints as the main memory contract
  - container tags / project routing as a first-class partition key
  - document, memory, and graph surfaces as separate products
  - multi-modal extractors for PDFs, images, video, and code
  - docs/test harnesses that exercise each integration path
- Reuse in memd:
  - harness plugin packaging model
  - memory/profile/RAG context stack
  - auto-capture / auto-recall client behavior
  - per-turn cache for prompt injection
  - project-scoped routing and onboarding
  - graph/browse surface for visible memory objects

## awesome-design-md

- Repo: [VoltAgent/awesome-design-md](https://github.com/VoltAgent/awesome-design-md)
- Files:
  - `README.md`
  - one `DESIGN.md` per style folder
  - `preview.html`
  - `preview-dark.html`
- Extract:
  - inspiration as markdown
  - previewable reference artifacts
  - curated folder structure
- Reuse in memd:
  - inspiration MD lane
  - portable design memory

## mempalace

- Repo: [milla-jovovich/mempalace](https://github.com/milla-jovovich/mempalace)
- Files:
  - `README.md`
  - `mempalace/mcp_server.py`
  - `mempalace/convo_miner.py`
  - `mempalace/searcher.py`
  - `mempalace/room_detector_local.py`
  - `mempalace/entity_registry.py`
- Extract:
  - verbatim storage
  - palace taxonomy
  - search filters
  - local room detection
  - entity registry / onboarding
- Reuse in memd:
  - memory model
  - raw evidence retention
  - structured search and onboarding

## Karpathy KB pattern

- Source:
  - Karpathy's knowledge-base workflow note
- Files:
  - raw ingest folders
  - compiled wiki markdown
  - Obsidian notes and images
- Extract:
  - raw/ directory as canonical input store
  - LLM-maintained wiki compilation
  - markdown-first corpus
  - self-maintenance via linting and health checks
- Reuse in memd:
  - research notebook workflow
  - compiled memory docs
  - self-evolving knowledge surfaces

## farzaa/wiki-gen-skill

- Source:
  - [gist](https://gist.github.com/farzaa/c35ac0cfbeb957788650e36aabea836d)
- Files:
  - `personal_wiki_skill.md`
- Extract:
  - ingest / absorb / query / cleanup / breakdown / status lifecycle
  - wiki as synthesized articles, not raw filing
  - anti-cramming and anti-thinning rules
- Reuse in memd:
  - compiled wiki flow
  - concept-first memory organization

## agent-zero

- Source:
  - [agent0ai/agent-zero](https://github.com/agent0ai/agent-zero)
- Files:
  - prompts folder
  - plugin/tool folders
  - web/terminal/browser integration surfaces
- Extract:
  - prompt-defined behavior
  - dynamic tool surface
  - persistent memory
  - multi-agent cooperation
- Reuse in memd:
  - editable agent policies
  - plugin/tool extension model
  - subagent orchestration

## Hermes Agent

- Source:
  - [NousResearch/hermes-agent](https://github.com/NousResearch/hermes-agent)
- Files:
  - install/docs pages
  - platform integration pages
  - persistent memory docs
- Extract:
  - cloud-first onboarding
  - self-host later
  - persistent memory
  - Obsidian and other platform gateways
- Reuse in memd:
  - onboarding model
  - adoption-friendly packaging
  - multi-surface access

## Follow-Up Pattern

For each file above, ask:

1. What behavior does this file own?
2. What does memd need from it?
3. What would a clean reimplementation look like?
4. What should not be copied?
