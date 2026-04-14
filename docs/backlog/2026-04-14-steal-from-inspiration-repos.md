# Steal From Inspiration Repos: mempalace + supermemory

status: open
severity: high
phase: Phase I
opened: 2026-04-14
type: extraction-plan

## Goal

Both repos are local. Extract everything that closes memd's 20 open gaps faster.
Not copying code — stealing patterns, approaches, and proof that memd's theory works.

## Repos

- `/home/josue/Documents/projects/mempalace` — Python, MCP server, 96.6% LongMemEval
- `/home/josue/Documents/projects/supermemory` — TypeScript/Bun, monorepo, plugin model

## What to Extract from mempalace

mempalace solves 5 problems memd has open:

### 1. Recall that actually works (closes #45 — no behavior-changing recall proof)
- `benchmarks/` has working benchmark harnesses for LongMemEval, LoCoMo, MemBench, ConvoMem
- memd has the same datasets in `.memd/benchmarks/datasets/` but no working harness
- **Steal:** benchmark runner architecture, scoring methodology, how they hit 96.6%

### 2. Dedup that works (closes #49 — memory dedup incomplete)
- `mempalace/dedup.py` — dedicated dedup module
- **Steal:** dedup strategy, similarity threshold, how they handle near-duplicates

### 3. Entity detection + knowledge graph (closes #44 — atlas dormant)
- `mempalace/entity_detector.py` — extracts entities from content
- `mempalace/entity_registry.py` — manages entity catalog
- `mempalace/knowledge_graph.py` — builds navigable graph
- `mempalace/palace_graph.py` — palace-specific graph navigation
- memd has atlas (974 lines, 7 routes) but it's completely dormant
- **Steal:** how they wire entity detection to graph population, what makes navigation useful

### 4. Conversation mining (closes gap in ingestion pipeline #37)
- `mempalace/convo_miner.py` — extracts memories from conversations
- `mempalace/miner.py` — general extraction
- `mempalace/general_extractor.py` — content → memory extraction
- **Steal:** extraction pipeline architecture, how raw content becomes structured memory

### 5. Room detection / auto-categorization (informs lane auto-activation)
- `mempalace/room_detector_local.py` — auto-categorizes memories into "rooms"
- This is exactly what memd lanes need — auto-detection of which domain a memory belongs to
- **Steal:** classification approach, signal extraction, threshold tuning

### 6. Repair + normalization (closes gap in correction flow #43)
- `mempalace/repair.py` — memory repair operations
- `mempalace/normalize.py` — content normalization
- `mempalace/spellcheck.py` — text quality
- **Steal:** how repair flows work in practice, what correction UX looks like

### 7. Hooks integration
- `mempalace/hooks/` and `mempalace/hooks_cli.py` — Claude Code hook integration
- **Steal:** hook patterns that actually work in production

## What to Extract from supermemory

supermemory solves 4 problems memd has open:

### 1. Plugin packaging model (informs harness integration)
- `packages/` has per-framework SDKs: openai-sdk, agent-framework, pipecat-sdk, ai-sdk
- `packages/hooks/` — hook system
- `packages/tools/` — tool definitions
- **Steal:** how one memory API serves multiple harnesses via thin adapters

### 2. Memory graph visualization (closes #44 — atlas dormant)
- `apps/memory-graph-playground/` — interactive graph explorer
- `packages/memory-graph/` — graph data structure
- **Steal:** graph rendering approach, what makes memory navigation useful in a UI

### 3. Auto-capture + auto-recall (informs ingestion pipeline #37)
- `packages/hooks/` — auto-capture from conversation events
- **Steal:** capture trigger design, what events produce useful memories vs noise

### 4. Browser/Raycast extensions (informs Phase I dashboard)
- `apps/browser-extension/` — memory from browser context
- `apps/raycast-extension/` — quick memory access
- `apps/web/` — web dashboard
- **Steal:** dashboard UX patterns, what views users actually need

## Extraction Process

For each extraction target:
1. Read the source file(s) completely
2. Document: what pattern they use, why it works, how to adapt for memd's Rust codebase
3. Map to specific memd backlog item it closes or accelerates
4. Write findings as architecture-lane source material in `.memd/lanes/architecture/`
5. Update inspiration lane with deeper extraction notes

## Priority Order

1. mempalace benchmarks → prove memd recall works (or doesn't)
2. mempalace entity detection + graph → wake atlas from dormancy
3. mempalace convo_miner → build ingestion pipeline
4. mempalace dedup → fix memory dedup
5. supermemory memory-graph → dashboard graph view
6. supermemory auto-capture → fix checkpoint noise
7. mempalace room_detector → lane auto-activation
8. supermemory plugin model → harness SDK packaging

## NOT Stealing

- Code verbatim — reimplement patterns in Rust
- Their specific data models — memd's schema is more sophisticated
- Their storage layer — memd's SQLite + atlas is better architecture
- Their test fixtures — memd has its own benchmark datasets
