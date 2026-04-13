# Agent Write Helpers Unreachable from Agents

- status: `open`
- found: `2026-04-13`
- scope: memd-client
- severity: high

## Summary

Shell helpers exist (`.memd/agents/remember-long.sh`, `correct-memory.sh`, etc.)
but agents can't use them. wake.md protocol section says `remember-long` — agents
try `memd remember-long` as CLI subcommand which fails. RAG backend disabled
(`MEMD_BUNDLE_BACKEND_ENABLED=false`). Full agent write pipeline is not operational.

## Symptom

- Agent runs `memd remember-long` → "unrecognized subcommand"
- `sync-semantic` shell helper has no backend to sync to
- Agents fall back to `memd remember` without kind/tag semantics
- No codebase structure, decisions, or preferences stored across sessions

## Root Cause

- Wake protocol at `wakeup.rs:261` lists bare names: `remember-short`, `remember-long`, etc.
- These are shell scripts at `.memd/agents/remember-long.sh`, not CLI subcommands
- Agents (Claude Code, Codex) can't execute arbitrary shell scripts from harness context
- `backend.env` has `MEMD_BUNDLE_BACKEND_ENABLED=false` — semantic backend disabled
- Claude Code CLAUDE.md says "use memd lookup, memd checkpoint, and the lane helpers"

## Fix Shape

- Option A: Fix wake.md protocol to show `.memd/agents/remember-long.sh --content "..."` paths
- Option B: Add thin CLI subcommand aliases (`memd remember-long` → `memd remember --kind fact --tag long-term`)
- Option B is better — agents can call CLI commands but not arbitrary shell scripts
- Fix protocol line at `wakeup.rs:261`
- Enable RAG backend or document it as optional

## Evidence

- `crates/memd-client/src/runtime/resume/wakeup.rs:261` — protocol line with bare names
- `.memd/agents/remember-long.sh` — shell helper calls `memd remember --kind fact`
- `.memd/backend.env` — `MEMD_BUNDLE_BACKEND_ENABLED=false`
- `crates/memd-client/src/bundle/init_runtime/mod.rs:2202-2234` — helper generation

## Dependencies

- blocks: [[docs/backlog/2026-04-13-memd-no-cross-session-codebase-memory.md|no-cross-session-codebase-memory]] (agents can't store typed facts without working write helpers)
- independent: can be fixed standalone (protocol line fix or CLI aliases)

## Related

- [[docs/audits/2026-04-13-full-codebase-audit.md]] — full audit findings
- [[docs/theory/locks/2026-04-11-memd-theory-lock-v1.md]] — live loop step 1 (capture raw event)
- [[integrations/codex/README.md:99-105]] — documents shell helper usage pattern
