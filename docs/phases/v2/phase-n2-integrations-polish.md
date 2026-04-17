---
phase: N2
name: Integrations Polish
version: v2
status: pending
depends_on: [F2, I2, M2-evo]
backlog_items:
  - "2026-04-14-skill-gating-config-flags-only"
  - "2026-04-14-rag-sidecar-disabled-no-fallback"
  - "2026-04-14-no-data-recovery-procedure"
  - "2026-04-14-no-admission-control-rate-limiting"
---

# Phase N2: Integrations Polish

## Goal

Integrations and runtime-safety surfaces are productized without breaking the core
memory path.

## Deliver

- Obsidian import with conflict resolution
- Two-way sync (file watcher or periodic poll)
- Harness SDK packaging model (from supermemory extraction)
- Runtime skill gating enforcement
- RAG sidecar with timeout, retry, fallback
- Data recovery procedure with restore proof
- Admission control / rate limiting at the product boundary

## Pass Gate

- Edit note in Obsidian → appears in memd within 1 cycle
- Store fact in memd → appears in Obsidian vault within 1 cycle
- Conflict: both sides edit → resolution documented and working
- Proposed skill remains blocked until policy gate allows it
- RAG query with 5s timeout → fallback to non-RAG retrieval
- Backup/restore runbook succeeds on a seeded bundle
- A noisy client cannot flood working memory unchecked

## Evidence

- Two-way sync test
- Conflict resolution test
- Skill gating enforcement test
- RAG fallback test
- Backup/restore walkthrough
- Admission control test

## Fail Conditions

- Sync loop (both sides endlessly updating)
- Conflict loses data
- Skill gate never actually blocks
- RAG timeout blocks main retrieval
- Restore procedure loses data
- Rate limiter can be bypassed by one noisy agent

## Donor Extraction (from inspiration repos)

- **N2-D1** (supermemory `packages/tools/`): Thin harness adapter pattern. 3-step for every framework: accept config → wrap core → return framework wrapper. Each adapter <200 lines. Core centralized.
- **N2-D2** (supermemory `shared/cache.ts`): Turn-scoped LRU cache (max 100 entries). Key: `container:thread:mode:normalizedMessage`. Prevents duplicate API calls within same agent turn.
- **N2-D3** (Smriti `SKILL.md`): Skill pack as versioned instruction file. Teaches agents: when to remember, when NOT to checkpoint, multi-agent etiquette.
- **N2-D4** (Omegon `plugins/mcp.rs` — **DIRECT RUST LIFT**): MCP server with multiple transport modes. Local process, OCI container, Docker gateway, Styrene mesh. Config per server: url, command, args, env, timeout.

See: `docs/theory/2026-04-14-donor-extraction-to-v2-phases.md` for full details.

## Rollback

- Disable sync if data loss detected
