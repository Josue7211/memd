---
phase: E4
name: Progressive-Depth Recall
version: v4
status: planned
opened: 2026-04-22
depends_on: [D4]
backlog_items: [progressive-depth-not-wired]
axis: token_efficiency
---

# Phase E4: Progressive-Depth Recall

## Goal

Three recall depths, each with a clear cost/quality contract: **wake** (<2k tokens, overview), **lookup** (targeted query, 1-3 records), **resume** (full state reconstruction, bounded). Agent picks depth; wake is cheap and constant, resume is expensive and precise, lookup is the middle. Today: wake and resume exist; lookup is weak; depth selection is manual and opaque.

## Why this phase exists

Progressive-depth is a core memd thesis per ROADMAP and MEMD-10-STAR. Without it agents either under-retrieve (wake-only → miss context) or over-retrieve (resume → blow tokens). Both axes (session continuity, token efficiency) cap here.

## Deliver

1. **Depth contracts.** Documented in `docs/contracts/recall-depth.md`:
   - wake: sync, ≤2k tokens, O(1) cost, returns compiled brief
   - lookup: sync, 1-3 records, O(query), returns typed records
   - resume: async-ok, bounded tokens, O(session-history), returns compiled task state
2. **`memd lookup --depth {wake|lookup|resume}`** flag exposed. Default lookup = `lookup`.
3. **Auto-escalation.** If lookup returns zero hits and query has specifier ("the X task", "what I was doing"), prompt agent to escalate to resume. No automatic escalation — explicit cost.
4. **Measurement.** Every depth call logs `.memd/logs/recall-depth.ndjson`: query, depth, token-cost, records-returned, latency.
5. **Progressive-depth bench (stub).** V5 owns the full bench; E4 lands the harness hooks so the bench can be wired later without schema change.

## Pass Gate

- pre: lookup depth exists but is rarely used; wake or resume is what agents reach for
- post: in 7-day dogfood, lookup is ≥30% of recall calls; token cost per turn drops vs pre
- evidence: `recall-depth.ndjson` distribution + per-turn token cost delta
- regression budget: no increase in session-continuity failures (agent misses context because it chose wrong depth)

## Product Win

Agent spends the right amount of tokens for the job. Cheap queries stay cheap; expensive queries only fire when needed.

## Evidence

- [`docs/contracts/recall-depth.md`](../../contracts/recall-depth.md) — normative depth + telemetry contract
- 7-day depth distribution histogram
- Per-turn token cost pre vs post

## Fail Conditions

- Agent defaults to resume regardless: add soft nudge in wake doc ("use lookup for targeted queries").
- Auto-escalation detects wrong cases: tune rule, or disable and require explicit.

## Rollback

Depth flag is additive; default depth unchanged until 7-day dogfood validates.
