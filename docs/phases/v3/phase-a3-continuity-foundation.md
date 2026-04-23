---
phase: A3
name: memd Continuity Foundation
version: v3
status: pending
depends_on: []
notes: NEW V3 entry phase, inserted 2026-04-17 when user directive (*"are you retarded why would A3 come next"*, *"this is a massive issue"*, *"everybacklog issue should be in the roaadmap for a fix"*) made clear that memd core-continuity bugs supersede retrieval phases. Cannot benchmark a product whose memory state leaks across compaction, whose hooks are scattered, whose backlog doesn't map to the roadmap. Fix the memory OS before tuning the memory OS.
backlog_items:
  - "2026-04-17-memd-read-state-lost-across-compaction"
  - "2026-04-17-hooks-scattered-across-three-dirs"
  - "2026-04-17-codebase-organization-pass"
  - "2026-04-17-memd-process-too-soft-cross-harness"
  - "2026-04-17-memd-file-structure-not-enforced-in-code"
  - "2026-04-16-pipeline-lifecycle-broken"
  - "2026-04-16-working-memory-stale-records"
  - "2026-04-15-memd-preferences-not-persisted-across-sessions"
  - "2026-04-14-no-live-memory-contract"
---

# Phase A3: memd Continuity Foundation

## Goal

Make memd **authoritative, not advisory** before any V3 retrieval work. memd must own continuity of state across compaction, session boundaries, and harness switches — so no assistant ever has to rediscover what a prior session already established. Every backlog item under "memd-core" must map to a fix in this phase (or an explicit deferral); the V3 roadmap must be a **complete** map of known issues → planned fixes.

## Why this phase exists

User directive 2026-04-17, canonical in memd:
> *"are you retarded why would A3 come next"* — (A3 = retrieval then; now moved to B3).
> *"this is a massive issue"* — re: memd losing file Read/Edit state across compaction.
> *"everybacklog issue should be in the roaadmap for a fix"* — roadmap must be exhaustive.
> *"thats not a you messed up youre never allowed to mess up thats waht memd does"* — memd owns the class of failure, not the assistant's workflow.

Today memd fails this contract on several surfaces:

- `2026-04-17-memd-read-state-lost-across-compaction`: after a Claude Code compaction, files the prior session Read are no longer considered Read by the Edit tool. The continuation session must re-Read before any Edit — this is wasted tokens, visible to the user, and a continuity bug in memd's job description.
- `2026-04-17-hooks-scattered-across-three-dirs`: hook scripts live in `.memd/hooks/`, `integrations/hooks/`, and `.claude/hooks/` with near-total duplication. No single source of truth.
- `2026-04-17-codebase-organization-pass`: 84 flat backlog items, phases intermixed across V1/V2/V3 in one directory, `.memd/` top-level has 23 entries of mixed purpose. Finding "where does a new X go" requires a scan.
- `2026-04-17-memd-process-too-soft-cross-harness`: memd process enforcement is advisory, not blocking. Sessions drift into host-harness defaults after wake.
- `2026-04-16-pipeline-lifecycle-broken`: promote/expire/archive lifecycle does not execute. Records accumulate forever.
- `2026-04-16-working-memory-stale-records`: completed phase status occupies working memory weeks after verification. Expiry never runs on phase completion.
- `2026-04-15-memd-preferences-not-persisted-across-sessions`: agents don't retain architecture decisions across sessions.
- `2026-04-14-no-live-memory-contract`: no machine-checkable contract for "what memd guarantees is live".

Each of these is memd failing its core product promise. V3 benchmark work is meaningless on top of a leaky foundation.

## Deliver

### Part 1 — State continuity (the "memd never forgets" contract)

1. **File-interaction ledger persisted per session** — pre-compact hook records `{file_path, op: read|edit|write, count, last_ts}` per session to `.memd/state/session-<id>/file_interactions.json`. Wake packet surfaces `## Files Touched` block so continuation session can bulk-Read before any Edit. Acceptance: simulated mid-edit compaction runs N subsequent Edits with zero `File has not been read yet` errors.
2. **`memd prime-reads` command** — emits newline list of paths from the prior session's ledger; continuation session mass-Reads them in one parallel batch. Optional `--since-session <id>` flag.
3. **Working-memory lifecycle enforcement** — promote/expire/archive runs on every phase-complete, checkpoint, and wake. Pipeline has a self-test (cron-style) that stores → recalls → expires a probe record and reports green/red.
4. **Preferences persist across sessions** — architecture decisions, style preferences, voice mode survive session restart AND compaction. Cross-session A→B→A replay test proves it.
5. **Live memory contract** — machine-readable `.memd/contract.json` lists: what wake guarantees, what checkpoint guarantees, what resume guarantees. Cross-harness validators check the contract, block output if it is violated.

### Part 2 — Enforcement (authoritative, not advisory)

6. **Hooks consolidation under one canonical tree** — `.memd/hooks/` becomes the single source. `.memd/hooks/shared/`, `.memd/hooks/harness/<name>/`. `integrations/hooks/` and `.claude/hooks/` become install-generated mirrors or symlinks. MANIFEST + `memd hooks list|install|doctor` subcommands.
7. **Cross-harness pre-send validator** — blocks output when required memd lifecycle steps (wake / checkpoint / handoff / roadmap-set) are missing or stale. Hard-fail mode, not reminder.
8. **Drift detection + repair** — memd detects when a session has left the memd process (no checkpoints, stale wake, no handoff before completion); auto-runs repair flow before allowing finish.
9. **Strict / locked enforcement modes** — config flag promotes advisory gates to hard-fail gates.

### Part 3 — Codebase organization

10. **Backlog → Roadmap mapping index** — `docs/backlog/INDEX.md` regenerated from frontmatter, grouped by milestone/phase/severity. Every item has a `phase:` field pointing at the V3/M4 phase that owns its fix, OR `phase: unassigned` with a TODO to assign. Script: `make backlog-index`. CI gate: new backlog items must have `phase:` set.
11. **docs/backlog/** re-grouped under `m1/ m2/ m3/ m4/ v3/ unassigned/` subfolders.
12. **docs/phases/** re-grouped under `v1/ v2/ v3/` subfolders.
13. **docs/handoff/LATEST.md** symlink to live packet (or `INDEX.md` regenerated on each new handoff).
14. **.memd/ top-level** grouped under `state/ config/ artifacts/ docs/ code/` — flatten goes to 6-ish entries from 23.
15. **docs/plans/archive/** for superseded execution plans, `status: superseded` frontmatter.

## Pass Gate

V3 cannot start B3 until every item below is true:

**Continuity:**
- Simulated compaction-mid-edit scenario: continuation session runs ≥10 consecutive Edits across prior-session files with zero `File has not been read yet` errors.
- `memd prime-reads` returns a non-empty newline list after any session with ≥1 file interaction.
- Lifecycle self-test (store → recall → expire → verify-expired) runs on a cron hook and reports green ≥99% of the time.
- Cross-session preference replay: A stores a preference → B session boots from cold → B retrieves it via wake. Automated test.
- `.memd/contract.json` exists, is machine-readable, and cross-harness validators consume it.

**Enforcement:**
- `find . -name "memd-*.sh" -path "*/hooks/*"` returns files in exactly one canonical tree (symlinks from harness-specific install targets allowed).
- `.memd/hooks/MANIFEST.json` covers every shipped hook.
- `memd hooks doctor` returns green on fresh install and loud red on tampered install.
- Pre-send validator blocks a deliberately-missing-checkpoint completion attempt (negative test).

**Organization:**
- `docs/backlog/INDEX.md` exists, is regenerated by `make backlog-index`, and every backlog file has `phase:` set (no unassigned items at merge time — each must be assigned or explicitly marked deferred).
- ROADMAP.md V3 section links every open memd-core backlog item to a phase (this phase or a named successor).
- `docs/phases/v{1,2,3}/` subfolders exist; wiki-links updated; `make lint-links` green.
- `docs/handoff/LATEST.md` resolves to the live packet.

Plus:
- `cargo test -p memd-core -p memd-server -p memd-client` green.
- No backlog item remains in "unassigned" state at merge.

## Evidence

- Compaction-continuity test log (before: Edit fails → Read needed; after: Edits succeed directly)
- `.memd/state/session-<id>/file_interactions.json` sample from a real session
- `memd hooks doctor` output on clean vs tampered install
- `docs/backlog/INDEX.md` regeneration diff
- Roadmap ↔ backlog coverage audit (every open backlog item maps to a phase)
- Cross-session preference replay test output
- Lifecycle self-test cron output (7-day window)

## Product Win

- **memd never forgets.** A continuation session picks up without the assistant rediscovering anything the prior session established. User never sees redundant Read/grep/check sequences.
- **memd is authoritative.** Sessions cannot drift into host-harness defaults; drift is detected and repaired, not accepted. Voice mode, lifecycle, checkpoints, roadmap state are enforced, not hoped.
- **Roadmap is exhaustive.** Every backlog item has a phase that will fix it. No orphaned issues.
- **Codebase is legible.** A new contributor reads `docs/README.md` once and knows where to put a new backlog / phase / handoff / plan / theory note.

Evidence:
- Stranger test: hand a fresh checkout to someone who has never worked in memd. They write a backlog item, a phase doc, and a handoff packet in the correct locations with correct frontmatter, from reading `docs/README.md` alone.
- Session-continuity demo: record a session mid-edit → compaction → continuation → zero redundant Reads.
- Leaderboard of compliance: `memd hooks doctor`, `memd contract verify`, `make backlog-index` all green on CI.

## Fail Conditions

- Continuation session still has to re-Read files the prior session touched — the core continuity contract is not fixed; this phase is not done.
- Hooks remain in three directories — consolidation didn't land.
- Any open backlog item lacks a `phase:` field — roadmap coverage is incomplete.
- Memd process still advisory (assistant can complete work without checkpoint/handoff) — enforcement didn't land.
- Preferences still lost across session restart — core value prop unfixed.

## Donor Anchors

- **A3-D1**: existing `.memd/hooks/memd-precompact-save.sh` — the natural anchor for the file-interaction ledger write hook
- **A3-D2**: existing wake-packet assembly code — natural anchor for the `## Files Touched` block emission
- **A3-D3**: memd lifecycle code (promote/expire/archive) — already partially implemented per `2026-04-16-pipeline-lifecycle-broken.md`, needs its self-test and cron wiring
- **A3-D4**: `2026-04-17-memd-process-too-soft-cross-harness.md` fix spec — the cross-harness validator design

## Rollback

- File-interaction ledger behind `memd.continuity.ledger=true`; revert to current behavior if it regresses checkpoint latency
- Hooks consolidation is fully reversible from git (no data migration)
- Strict enforcement mode flag-gated; can drop back to advisory mode if a harness breaks
- Directory reorg is a single commit; revertable as a whole unit if wiki-link breakage is too wide

## Out of scope

- Retrieval quality work (B3)
- Reranker (C3)
- Atlas traversal (D3)
- Episode consolidation (E3)
- Bench honesty (F3)
- Actual score changes on any benchmark — A3 is infra, not quality

## Why A3 is V3 entry, not a deferred chore

memd's product is continuity. V3 promises "FINAL memory OS, above and beyond 70% intrinsic floor." You cannot benchmark a memory OS that loses state across compaction; any bench number is unreliable because the system itself is leaky. Fix the memory OS before measuring it. Retrieval tuning in B3-F3 is the skin; continuity in A3 is the skeleton.
