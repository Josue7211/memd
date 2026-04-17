# Hooks Consolidation Audit — A3 Part 2 Prep (2026-04-17)

## Canonical direction

`.memd/hooks/` is canonical; `integrations/hooks/` re-derives from it. `.claude/hooks/memd-bootstrap.sh` shims to `.memd/hooks/memd-bootstrap.sh`.

## Per-file disposition

| Filename | Canonical | Delta summary | Reason for delta | Rewrite rule for sync script |
|----------|-----------|---------------|------------------|------------------------------|
| install.sh | .memd | integrations missing 3 installs (memd-file-interaction.sh, memd-lifecycle-probe.sh, memd-bootstrap.sh) and 3 wrapper creations | Part 1 added these to .memd after integrations was last synced; integrations is older, stale baseline | delete lines 14–16 and 48–63 from integrations version, replace with .memd version |
| memd-bootstrap.sh | .memd | integrations has session-aware staleness check (SESSION_ID, session_has_live_wake_receipt, stamp_live_wake_receipt); .memd is simpler global check | integrations evolved to per-session caching after .memd baseline; .memd reverted to simpler logic | delete SESSION_ID parsing and session-marker functions (lines 12, 46–62 in integrations), delete session guard from staleness check (line 68 condition), delete stamp call (line 84), delete session-gated fallback (line 86 condition check), keep .memd's global-only logic |
| memd-capture.sh | integrations | integrations ends cleanly; .memd appends obsolete CODEX_MEMORY.md fallback (lines 61–63) | .memd has stale CODEX-specific fallback; integrations removed it as part of agent-neutral refactor | delete .memd lines 61–63, keep integrations version as-is |
| memd-context.sh | integrations | integrations ends cleanly; .memd appends obsolete CODEX_WAKEUP.md fallback (lines 65–67) | .memd has stale CODEX-specific fallback; integrations removed it as part of agent-neutral refactor | delete .memd lines 65–67, keep integrations version as-is |
| memd-file-interaction.sh | .memd | identical in both | no delta | copy .memd version as-is |
| memd-lifecycle-probe.sh | .memd | identical in both | no delta | copy .memd version as-is |
| memd-precompact-save.sh | .memd | identical in both | no delta | copy .memd version as-is |
| memd-spill.sh | .memd | identical in both | no delta | copy .memd version as-is |
| memd-stop-save.sh | .memd | identical in both | no delta | copy .memd version as-is |
| memd-pretool-gate.sh | .memd | only in .memd, not in integrations | new in Part 1 Task 5, unique to .memd | copy .memd version to integrations (not in Task 14 scope — Task 14 only syncs divergent files; Task 15+ will add new files) |
| memd-bootstrap.ps1 | .memd | identical in both | no delta | copy .memd version as-is |
| memd-capture.ps1 | .memd | identical in both | no delta | copy .memd version as-is |
| memd-context.ps1 | .memd | identical in both | no delta | copy .memd version as-is |
| memd-precompact-save.ps1 | .memd | identical in both | no delta | copy .memd version as-is |
| memd-spill.ps1 | .memd | identical in both | no delta | copy .memd version as-is |
| memd-stop-save.ps1 | .memd | identical in both | no delta | copy .memd version as-is |
| install.ps1 | .memd | identical in both | no delta | copy .memd version as-is |
| README.md | integrations | .memd appends obsolete CODEX/CLAUDE_CODE/AGENT_ZERO/OPENCLAW/OPENCODE/HERMES agent-specific refresh list (lines 47–62), .memd has stale Claude Code docs (lines 131–134) | .memd has pre-refactor agent-specific regeneration details and wrong cached bootstrap policy; integrations has cleaner, agent-neutral docs | delete .memd lines 47–62 and replace stale docs (lines 131–134) with integrations version |

## Rewrite rule summary for scripts/sync-integration-hooks.sh

Per-file rewrites Task 14 applies when syncing from canonical `.memd/hooks/` → `integrations/hooks/`:

- `install.sh`: use `.memd` version (contains full install list)
- `memd-bootstrap.sh`: use integrations version (simpler, correct staleness logic)
- `memd-capture.sh`: use integrations version (clean agent-neutral ending)
- `memd-context.sh`: use integrations version (clean agent-neutral ending)
- `README.md`: use integrations version (agent-neutral, correct bootstrap policy)
- `*.ps1`, `memd-*-save.sh`, `memd-spill.sh`, `memd-file-interaction.sh`, `memd-lifecycle-probe.sh`: use `.memd` version (identical or only in .memd)

No path-based sed rules needed; all divergence is semantic (logic/docs), not environment paths.
