# Handoff — A3 Part 2 (Continuity Enforcement) green, next = Part 3 decision point

**Date:** 2026-04-17
**Branch:** `research/mining`
**Plan:** `docs/superpowers/plans/2026-04-17-a3-part2-enforcement.md`
**Status:** code-green; live-hook wiring is a user action (see below)

## What shipped

Part 2 turns Part 1's **surfacing** signal into **binding enforcement** + grows the machine-readable contract from 1 to 4 guarantees + fixes the cold-boot preference replay bug + consolidates three hook trees onto one canonical source.

| ID | Deliverable | Evidence |
| --- | --- | --- |
| D1 | Enforcement gate: PreToolUse hook + `## Continuity Gate` wake block + policy toggle | `memd-core::enforcement::{EnforcementPolicy, GateDecision, gate_decision, format_gate_output, FreshReadIndex, load_latest_sealed_paths}`, `memd hook gate` CLI, `.memd/hooks/memd-pretool-gate.sh`; tests `continuity_enforcement_tests` (8 passing) incl. `enforcement_end_to_end_seal_deny_read_allow` |
| D2 | Contract grows to 4 typed guarantees + verifier pulls evidence from bundle | `MemdContract` version bumped `0.1.0` → `0.2.0`; `ContractGuarantees` now has `surfaces_files_touched_when_sealed_ledger_exists`, `seals_session_ledger_on_precompact`, `enforces_continuity_gate_when_configured`, `replays_preferences_on_cold_boot`; `ContractEvidence` is tri-state for preference (`Option<bool>`); verifier helpers read bundle state (live-ledger, sealed-dir, config, hook-wired) |
| D3 | Preference replay fix (closes 2026-04-15 backlog) | `memd-client::runtime::resume::ResumeSnapshot::preferences` populated via `RetrievalIntent::Preference` `context_compact` call; `render_preferences_block` helper emits `## Preferences` block between `## Focus` and `## Atlas`; 4 new tests in `continuity_enforcement_tests` |
| D4 | Hooks consolidation: `.memd/hooks/` canonical + sync script + deprecation notice + CI idempotency check | `scripts/sync-integration-hooks.sh`, `scripts/hooks-lint.sh`, top-of-file notices in both READMEs, `.claude/hooks/memd-bootstrap.sh` converted to exec shim |

## Gate results

```
cargo test --workspace
→ 724 tests total (enforcement 8 + foundation 11 + contract 9 + 696 other); 0 failures
```

```
memd contract verify --output .memd
→ contract verify ok — version 0.2.0 (sealed_ledger=true files_touched=53)
```

```
memd diagnostics lifecycle-probe --output .memd
→ status: green — store/recall/expire/verify_expired all ok
```

```
bash scripts/hooks-lint.sh
→ hooks-lint: integrations/hooks/ in sync with .memd/hooks/
```

## Commit manifest

| SHA | Task | Scope |
| --- | --- | --- |
| `add420e` | T1 | enforcement.rs: EnforcementPolicy + GateDecision + gate_decision + format_gate_output |
| `caa276d` | T2 | FreshReadIndex reads current-session ledger |
| `58e61c4` | T3 | load_latest_sealed_paths reads most-recent sealed ledger |
| `1b4a3e3` | T4 | `memd hook gate` CLI subcommand |
| `8c22f00` | T5 | `.memd/hooks/memd-pretool-gate.sh` + install docs |
| `e5342fc` | — | Track Part 1 hook scripts (leftover from gitignore fix) |
| `ef99959` | T6 | `## Continuity Gate` wake block |
| `40db48f` | T7 | acceptance test: seal→deny→read→allow flow |
| `a474d42` | T8 | `ContractGuarantees` grows to 4 fields; version `0.2.0` |
| `3b0ed9c` | T9 | contract verify reads enforcement + preference evidence |
| `4296886` | T10 | preference audit (findings doc) |
| `9d41bd9` | T11 | wake surfaces preference memories (closes backlog) |
| `87ed4c2` | T13 | hooks consolidation audit |
| `e29e95c` | T14 | `scripts/sync-integration-hooks.sh`; `.memd/hooks/` canonical |
| `fc03c8e` | T15 | `.claude/hooks/memd-bootstrap.sh` → exec shim |
| `9bcaa81` | — | sync-fix: integrations/hooks/install.sh aligned |
| `7d1e047` | T16 | `scripts/hooks-lint.sh` CI drift check |

T12 (backlog status flip + regression test) was completed inline with T11.

## Scope deviation notes

- **Hooks consolidation direction inversion on 3 files.** The T13 audit found `.memd/hooks/memd-bootstrap.sh`, `memd-capture.sh`, `memd-context.sh` had stale CODEX fallbacks that `integrations/hooks/` already cleaned up. The sync script now preserves the `integrations/hooks/` version for those 3 files (reverse of the plan's default direction) via a backup-restore dance. Cleaner long-term fix: port those cleanups INTO `.memd/hooks/` so the sync can be a straight copy. Filed as technical debt — not blocking.
- **Preference evidence marker wiring (T9).** The contract-evidence helper reads `state/preference-replay.{green,red}`. The marker is dropped by the T11 test `preference_replay_marker_green_when_render_path_works` when `MEMD_BUNDLE_ROOT` is set, so `cargo test` runs against the real bundle produce the green marker as a byproduct. Operator smoke paths don't need to care.
- **T12 absorbed into T11.** Backlog flip + closing note happened in the T11 commit (`9d41bd9`); no separate T12 commit.

## Known gap — live PreToolUse hook not wired in user settings (same as Part 1 pattern)

Part 1's handoff already noted that the committed hook script `.memd/hooks/memd-file-interaction.sh` wasn't referenced from `~/.claude/settings.json` at handoff time. The same applies to Part 2's new `.memd/hooks/memd-pretool-gate.sh`:

```
$ jq '.hooks.PreToolUse' ~/.claude/settings.json
null   # or missing memd-pretool-gate entry
```

### User action to close the gap

Add to `~/.claude/settings.json`:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Edit|Write|NotebookEdit",
        "hooks": [
          { "type": "command", "command": "bash \"$HOME/Documents/projects/memd/.memd/hooks/memd-pretool-gate.sh\"", "timeout": 10 }
        ]
      }
    ]
  }
}
```

Plus create `.memd/config.json` in the bundle root (or add to an existing one):

```json
{
  "continuity": {
    "enforcement": "warn"
  }
}
```

Default is `warn` (surface the continuity message, don't block). Flip to `"block"` once you're confident it won't produce false positives on paths you no longer need to re-Read. `"off"` fully disables the gate.

Then in a new session: Edit a path listed in `.memd/state/session-<prior-id>/sealed/*.json` → Claude Code should show the gate message. Read the file first and the Edit passes unchanged.

## Next — Part 3 decision point (user call)

Per Part 1's handoff, Part 3 was "still undefined — candidates include:"

1. **Cross-session diff summaries** — when a new session boots, show what changed in the repo since the last session's sealed snapshot (git diff against prior seal).
2. **Auto-prime on wake** — the continuation session auto-Reads the top-N files from the sealed ledger before the agent's first turn, so the `## Continuity Gate` block is empty on arrival.
3. **Workspace-level recall aggregation** — merge ledgers across multiple project bundles when the user hops workspaces, so global state (preferences, decisions) surfaces everywhere.

Other Part 3 candidates the plan deferred:
- Default policy flip from `warn` to `block` (contingent on telemetry showing low false-positive rate).
- Retention policy on sealed ledgers (currently unbounded).
- Hooks consolidation finish — port the `integrations/hooks/` cleanups (bootstrap/capture/context) back into `.memd/hooks/` so the sync script can be a straight copy.

**Options:**
1. Pick one Part 3 shape and open a plan for it.
2. Close A3 here — Part 2 delivers the full enforcement contract; Part 3 ambitions can live in backlog.
3. Pivot to the next V3 milestone (B3 Intrinsic Retrieval) and return to Part 3 later.

Awaiting user direction.

## Pointers

- Plan: `docs/superpowers/plans/2026-04-17-a3-part2-enforcement.md`
- Audits produced this phase: `docs/handoff/2026-04-17-a3-part2-preference-audit.md`, `docs/handoff/2026-04-17-a3-part2-hooks-audit.md`
- Contract source of truth: `.memd/contract.json` (version `0.2.0`, 4 guarantees)
- Hook scripts canonical source: `.memd/hooks/`; regenerate `integrations/hooks/` via `scripts/sync-integration-hooks.sh`; lint via `scripts/hooks-lint.sh`
- Prior handoff (this phase's predecessor): `docs/handoff/2026-04-17-a3-part1-green-next-part2.md`
