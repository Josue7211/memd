# Session Resume Ignores memd / No Phase Progress in Memory

status: open
severity: high
phase: Phase I
opened: 2026-04-13

## Problem

When starting a new session and asking "where did we leave off on Phase I", the model:

1. Searches `.planning/` (deleted — stale ref in global CLAUDE.md, now fixed)
2. Reads ROADMAP.md, git log, then glob-scans 5-7 source files to reconstruct state
3. Only checks memd when explicitly told "check ur memory"
4. memd lookup returns nothing useful — "No deep Phase I memory stored"

The model rebuilds context from scratch every session instead of resuming from memd.

## Root Causes

### A. Bootstrap hook not firing (FIXED 2026-04-13)
- `UserPromptSubmit` hook was only in project `.claude/settings.json`
- Project-level hooks need approval and silently skip if not approved
- Fix: moved hook to global `~/.claude/settings.json`

### B. Global CLAUDE.md had stale `.planning/` refs (FIXED 2026-04-13)
- `~/.claude/CLAUDE.md` still said `.planning/` is authoritative state
- Fix: updated both refs to `ROADMAP.md`

### C. Phase progress not stored as durable truth (OPEN)
- No memd memory captures Phase I build state (what's done, what's next, what's stubbed)
- Each session must re-scan source files to figure out where it left off
- `memd lookup --query "phase I dashboard"` returns nothing actionable
- The continuity section in wake.md only has item IDs, not human-readable progress

### D. Model doesn't prioritize memd over file scanning (OPEN)
- Even with bootstrap working, the model's instinct is to read files + git log
- CLAUDE.md says "memd wake MUST be first action" but model treats it as optional
- Need stronger enforcement or the wake output needs to contain enough context
  that file scanning becomes unnecessary

## Fix Plan

1. After each Phase I work session, capture progress as durable truth via `memd remember-long`
2. Wake packet should include phase progress summary so model sees it on boot
3. Consider adding phase status to the wake compiler's working memory selection
4. The bootstrap hook (now global) should inject enough context that the model
   can answer "where did we leave off" from wake data alone
