---
phase: D4
name: Working-Context Compiler
version: v4
status: planned
opened: 2026-04-22
depends_on: [B4]
backlog_items: [wake-context-bloat, token-efficiency-untested]
axis: token_efficiency
---

# Phase D4: Working-Context Compiler

## Goal

Wake context today is a raw dump of top-k retrieved records. It bloats tokens and buries the agent under noise. D4 ships a compiler: retrieved records → compressed, typed, prioritized working context. Target: <2k tokens wake, zero continuity loss.

## Why this phase exists

Token-efficiency axis at 1/10. Current `memd wake` output is verbose, flat, untyped. Competitor surfaces (mempalace atlas, mem0 dashboard) compile tighter. Without D4, V6 public-bench lift is handicapped — the generator gets paid in noise.

## Deliver

1. **Compiler pipeline.** Input: retrieved records (episodic + semantic + canonical + candidate + preferences + focus). Output: single structured wake doc, <2k tokens, sections by type.
2. **Priority rules.**
   - canonical truths first
   - preferences next
   - active focus (what I'm doing)
   - recent episodic (last 3 turns)
   - semantic facts (deduplicated)
   - candidates (flagged as uncertain)
3. **Deduplication.** Same fact from multiple records → single line, provenance list.
4. **Token budget enforcer.** Hard cap at 2000 tokens; overflow demoted to `memd lookup` depth.
5. **Continuity-loss test.** Before/after wake on 20 recorded session-resume scenarios. "After" wake must answer the same "what was I doing?" / "what did I learn?" / "what did user prefer?" queries the raw wake answered.

## Pass Gate

- pre: mean wake size ~5k tokens; no continuity-loss test
- post: mean wake size <2k tokens; continuity-loss test 20/20 pass
- evidence: wake-size histogram from 7-day dogfood + continuity-loss test output
- regression budget: zero queries that succeeded pre-D4 may fail post-D4 on the continuity test

## Product Win

Agent wakes into a clean brief, not a transcript. Feels like "I know what's going on" instead of "let me parse this wall of text." Token cost per session drops measurably.

## Evidence

- Wake-size histogram (pre vs post)
- Continuity-loss test output
- Sample wake-doc diffs (before/after) for 5 sessions

## Fail Conditions

- Compressor strips a fact that the continuity test needs: bump priority rule, rerun.
- Token cap forces useful info out: make `MEMD_WAKE_BUDGET_TOKENS` env-configurable, default 2000.

## Rollback

Behind `MEMD_D4_COMPILER=1`. Raw wake preserved as `memd wake --raw`.
