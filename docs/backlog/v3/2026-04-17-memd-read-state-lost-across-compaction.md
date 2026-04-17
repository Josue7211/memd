---
status: open
severity: high
phase: A3
opened: 2026-04-17
scope: harness-core, continuity
---
# memd Loses File Read/Edit State Across Compaction

- status: `open`
- severity: `high`
- phase: `V2-N2` (or new V3 sub-item)
- opened: `2026-04-17`
- scope: harness-core, continuity

## Problem

When a Claude Code session is compacted mid-conversation, the host tool's
Read-before-Edit precondition resets: every file the pre-compaction session Read
must be Read again before the continuation session can Edit it. memd does not
track or replay this state, so the continuation session discovers the gap only
when an Edit call fails with `File has not been read yet. Read it first before
writing to it.` The user sees the assistant re-reading files it "already read"
and correctly calls this a memd bug, not an assistant workflow issue.

Quote (2026-04-17): *"you should never be using read before memd.. thats a bug"*
and *"thats not a you messed up youre never allowed to mess up thats waht memd
does"*. Rule: memd must eliminate the class of failure, not ask the assistant
to work around it.

## Evidence

- session `claude-code@session-7eab5dde` hit this on 2026-04-17 editing
  `phase-b3-reranker-embeddings.md` and `phase-c3-atlas-at-recall.md` after
  compaction; both had been referenced in the summary but the Read tool's
  session register was clean, so two Edit calls failed with the
  "File has not been read yet" error before the assistant recovered by
  reading both files.
- pre-compaction summaries list file paths touched, but no structured record
  of which files were Read vs Edited vs Written is consumed by wake/resume
- `memd wake --output .memd` does not emit a file-state block
- `memd-precompact-save.sh` is the natural hook for capturing this state,
  but it currently does not

## Root Cause Hypotheses

1. Claude Code's Read-before-Edit constraint is enforced per process, so tool
   session state does not survive compaction (this part is not memd's fault)
2. memd has no per-session "files touched" ledger — there is no equivalent of
   `git status` for tool-level file interactions
3. The pre-compaction hook (`memd-precompact-save.sh`) does not record the
   file-interaction set; nothing is available for the post-compaction session
   to consume
4. The wake packet does not surface prior-session touched-files, so even if
   memd recorded it, continuation cannot consume it
5. No `memd prime-reads` (or equivalent) command exists that would let the
   continuation session bulk-Read the recorded set before its first Edit

## Fix

- pre-compaction hook records `{file_path, last_op: read|edit|write, op_count}`
  per session, persisted to `.memd/state/session-<id>/file_interactions.json`
- wake/resume emit a `## Files Touched` block listing every path the previous
  session Read/Edited/Wrote, with clear instruction: "Read each path before
  your first Edit to that path"
- new command: `memd prime-reads [--since-session <id>]` outputs a newline list
  of paths so the continuation session can mass-Read them in a single parallel
  batch before any Edit
- stretch: a Claude Code hook (PreToolUse on Edit/Write) that auto-primes the
  Read register from the persisted file-interactions set, eliminating the
  round-trip entirely
- acceptance test: simulate a compaction mid-edit-chain; post-compaction
  session completes N Edits across files from the prior session with zero
  `File has not been read yet` errors

## Relationship to other items

- complements `2026-04-17-memd-process-too-soft-cross-harness.md` — same class
  of "memd must be authoritative, not advisory"
- fix implementation will likely touch `.memd/hooks/memd-precompact-save.sh`
  and wake-packet assembly; see `2026-04-17-hooks-scattered-across-three-dirs.md`
  for the hooks-layout fix that should land first
