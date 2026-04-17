# Handoff — A3 Part 1 (Continuity Foundation) green, next = Part 2 decision point

**Date:** 2026-04-17
**Branch:** `research/mining`
**Plan:** `docs/superpowers/plans/2026-04-17-a3-part1-continuity-foundation.md`
**Status:** code-green; live-hook wiring is a user action (see below)

## What shipped

Part 1 delivers the **surfacing** half of A3 Continuity Foundation. Four deliverables:

| ID | Deliverable | Evidence |
| --- | --- | --- |
| D1 | File-interaction ledger + PostToolUse hook + `## Files Touched` in wake | `memd-core::file_ledger`, `memd hook file-interaction`, `memd hook seal-ledger`; tests `hook_file_interaction_appends_ledger_entry`, `hook_seal_ledger_copies_current_to_sealed_dir`, `compaction_mid_edit_flow_surfaces_prior_session_files` |
| D2 | `memd prime-reads` CLI | tests `prime_reads_runs_with_populated_ledger`, `prime_reads_since_session_reads_live_ledger` |
| D3 | `memd diagnostics lifecycle-probe` store→recall→expire→verify-expired self-test | `memd-core::lifecycle_probe`, `cli_lifecycle_probe_runtime`; live test `lifecycle_probe_reports_green_on_healthy_server` |
| D5 | `memd contract verify` + tracked `.memd/contract.json` | `memd-core::contract` (ONE typed guarantee: `surfaces_files_touched_when_sealed_ledger_exists`); tests `contract_generate_writes_default_shape_and_verify_passes_on_clean_bundle`, `contract_verify_errors_when_sealed_ledger_exists_but_files_touched_missing`, `contract_verify_green_when_sealed_ledger_surfaces_files` |

**Part 1 is surfacing-only — enforcement (refuse-to-proceed if sealed ledger shows files not yet re-read) is Part 2.**

## Gate results

```
cargo test -p memd-client --bin memd continuity_foundation
→ 11 passed; 0 failed
```

```
./target/debug/memd diagnostics lifecycle-probe --output .memd --summary
→ lifecycle-probe green probe_id=... steps=4
   - ok store / ok recall / ok expire / ok verify_expired
```

```
./target/debug/memd contract verify --output .memd
→ contract verify ok — version 0.1.0 (sealed_ledger=false files_touched=0)
```

Workspace test (prior session): 660 tests, 0 failures.

## Commit manifest

| SHA | Task | Scope |
| --- | --- | --- |
| `22ffbb9` | 2 | `memd hook file-interaction` + `seal-ledger` subcommands |
| `055166d` | 3 | PostToolUse + precompact hook scripts + policy doc |
| `c41c496` | 4 | wired hook scripts into mirror + install.sh |
| `7fd729d` | 5 | `## Files Touched` block in wake-up |
| `f817f53` | 6 | `memd prime-reads` CLI |
| `623a0ff` | 7 | acceptance test: compaction-mid-edit flow |
| `f3dc8fb` | 9 | working-memory lifecycle self-test (D3) |
| `efacd09` | 10 | live memory contract.json + `memd contract verify` (D5) |

## Known gap — live PostToolUse hook not wired in user settings

Advisor blocker (pre-handoff): the committed hook script `.memd/hooks/memd-file-interaction.sh` exists, but nothing in `~/.claude/settings.json` invokes it, so THIS live Claude Code session is not populating `.memd/state/session-<id>/file_interactions.json`. Verified empty:

```
$ ls .memd/state/session-*/file_interactions.json
(no matches)
$ ./target/debug/memd prime-reads --output .memd
(empty)
```

Tests cover the hook end-to-end (Tasks 2, 4, 7 drive the same code path the runtime wrapper invokes), so Part 1 code is correct. The gap is **install-side**, not code-side.

### User action to close the gap

Add to `~/.claude/settings.json` (new `hooks` block, or merge if one exists):

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Read|Edit|Write|NotebookEdit",
        "hooks": [
          { "type": "command", "command": "/home/josue/Documents/projects/memd/.memd/hooks/memd-file-interaction.sh" }
        ]
      }
    ],
    "PreCompact": [
      {
        "hooks": [
          { "type": "command", "command": "/home/josue/Documents/projects/memd/.memd/hooks/memd-precompact-save.sh" }
        ]
      }
    ]
  }
}
```

Then in a new session: do some file edits → `/compact` → next session's wake-up should show `## Files Touched` populated. Part 2's enforcement layer is what will actually pressure-test this wiring end-to-end, so manual verification here is optional but cheap.

## Next — Parts 2/3 decision point (user call)

Per advisor's scoping feedback: **do not autonomously write Parts 2/3 plans**. Surface as decision point.

- **Part 2 (Enforcement):** sealed ledger → explicit "re-read these files before acting" gate. Likely shape: wake-up or session-start fails loudly if `files_touched` non-empty and user hasn't issued a re-read. Cross-harness contract validator consumes `contract.json` (ONE guarantee today, grows).
- **Part 3 (Active continuity):** still undefined — candidates include cross-session diff summaries, auto-prime on wake, or workspace-level recall aggregation.

**Options:**
1. Write Part 2 plan now (advisor recommends this path once Part 1 is live-verified).
2. Pause A3, pivot to another milestone.
3. User picks their own shape for Part 2 first.

Awaiting user direction before touching plans.

## Pointers

- Plan: `docs/superpowers/plans/2026-04-17-a3-part1-continuity-foundation.md`
- Policy: `docs/policy/lifecycle-probe.md`
- Contract source of truth: `.memd/contract.json` (tracked via `.gitignore` `!` exception)
- Prior handoff (this phase's predecessor): `docs/handoff/2026-04-17-a3-part1-plan-ready-next-execute.md`
