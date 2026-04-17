---
date: 2026-04-17
from: claude-code@session-7eab5dde
to: next session executing V3 / A3 (memd Continuity Foundation)
branch: research/mining
last_clean_commit: (uncommitted at time of this rewrite — next session commits the V3 reshuffle + new A3 doc + backlog adds)
status: V3 active. A3 reassigned to memd Continuity Foundation. Old A3 (Intrinsic Retrieval) shifted to B3. No source code touched this session.
supersedes: docs/handoff/2026-04-16-V3-milestone-seeded-archive-cleanup.md (still readable for V3-seed history)
self_supersedes: earlier 2026-04-17 revision of this same file (which described A3 as Intrinsic Retrieval). Reshuffle happened mid-session when user flagged memd core-continuity bugs as blocking; this file was rewritten in place.
---

# Handoff Packet — V3 A3 Is Continuity Foundation, Not Retrieval

## TL;DR

V3 phase IDs were reshuffled **twice** on 2026-04-17:

1. **First reshuffle (early session):** renamed old V3 phases so alphabet matched execution order. Old A3 (Bench Honesty) → E3; old B3 (Activate Retrieval) → A3; etc.
2. **Second reshuffle (this session, late):** inserted a **new A3 = memd Continuity Foundation** at V3 entry, pushing everything down one letter: retrieval → B3, reranker → C3, atlas → D3, consolidation → E3, bench honesty → F3.

Why the second reshuffle: user directive *"are you retarded why would A3 come next"* + *"this is a massive issue"* + *"everybacklog issue should be in the roaadmap for a fix"*. Translation: memd is leaking state across compaction, hooks are scattered across three directories, the codebase isn't organized, and backlog items aren't mapped to roadmap phases. **Cannot benchmark a memory OS that loses state across compaction.** Fix the memory OS before tuning the memory OS.

Next concrete action: A3 Part 1 — file-interaction ledger + wake "Files Touched" block, so continuation sessions don't re-Read files the prior session already Read. Everything below is the unpacking.

## V3 framing (read before executing any phase)

V3 is the **FINAL memory OS**. Not a better v1. Not catch-up. The last version anyone needs. Product must be **great without RAG**. Sidecar is an optional accelerator, not load-bearing.

**Four canonical decisions logged to memd on 2026-04-17:**
1. *"not looking for the fastest ship, looking for the best product"* — every phase dual-gated (bench + product).
2. *"all the other services don't rely on rag for better benches and truly we shouldn't either, is supposed to be optional and a great product even without"* — sidecar is flag-gated accelerator, not load-bearing.
3. *"we need at least 70% on ALL benches WITHOUT the sidecar — that's where our competition is at — that's the bare minimum — this is the FINAL memory OS, we need to go above and beyond"* — V3 completion gate is ≥0.70 intrinsic on LongMemEval, LoCoMo, MemBench, and ConvoMem.
4. *"thats not a you messed up youre never allowed to mess up thats waht memd does"* + *"everybacklog issue should be in the roaadmap for a fix"* — memd owns the class of failure, not the assistant's workflow. Every backlog item must map to a roadmap phase. Orphaned issues are unacceptable.

Every V3 phase doc now has a `## Product Win` section alongside `## Pass Gate`. Bench reports carry two columns going forward: **intrinsic** (sidecar off, primary, 0.70 floor on all four) and **accelerated** (sidecar on, bonus, ≥+0.02 delta required).

## V3 phase table (current, post-reshuffle-2)

| Phase | Name | Execution slot | depends_on |
|-------|------|----------------|------------|
| **A3** | **memd Continuity Foundation** | 1 (entry, NEW) | — |
| B3 | Intrinsic Retrieval (was A3 pre-reshuffle-2) | 2 | [A3] |
| C3 | Reranker + Embeddings (was B3) | 3 | [A3, B3] |
| D3 | Atlas at Recall (was C3) | 4 | [A3, B3, C3] |
| E3 | Consolidation + Sessions (was D3) | 5 | [A3, B3, C3, D3] |
| F3 | Bench Honesty (was E3) | 6 | [A3, B3, C3, D3, E3] |

A3 phase doc: [[docs/phases/phase-a3-continuity-foundation.md]] (authoritative).

## What happened this session

1. **Picked up V3 handoff** from 2026-04-16 packet. Verified clean tree, ROADMAP_STATE on v3/V3/B3 pending.
2. **User flagged bad phase IDs** — alphabet didn't match execution. Reshuffle 1 executed + committed as `d0c00dd`.
3. **User flagged class of memd bugs** — *"why are you stil having to read these files so much"*, *"thats not a you messed up youre never allowed to mess up thats waht memd does"*. Translation: memd must eliminate re-Read-after-compaction, not blame the assistant. Superseded initial mis-framed preference memory; new decision logged.
4. **User demanded hook consolidation + codebase organization pass.**
5. **Three new backlog items authored:**
   - `docs/backlog/2026-04-17-memd-read-state-lost-across-compaction.md`
   - `docs/backlog/2026-04-17-hooks-scattered-across-three-dirs.md`
   - `docs/backlog/2026-04-17-codebase-organization-pass.md`
6. **User rejected burying new items in M4/N2**: *"ok so wh yeh fuck would a3 come nbext are ytou retarded"*, *"it has to be in the v3 roadmap"*, *"everybacklog issue should be in the roaadmap for a fix"*.
7. **Reshuffle 2 executed:** new A3 = Continuity Foundation inserted at V3 entry. Old A3→B3, B3→C3, C3→D3, D3→E3, E3→F3. Five phase files renamed via git mv in reverse order (collision-safe). Phase frontmatter updated. Body cross-refs shifted via Python regex. Donor-suffix miscounts fixed manually in 4 files.
8. **New phase doc authored:** `docs/phases/phase-a3-continuity-foundation.md` — three-part deliverable spec (continuity, enforcement, organization).
9. **ROADMAP.md updated:** V3 table rewritten to 6 rows, Status Snapshot + ROADMAP_STATE patched, `next_step` set to A3 Part 1, `active_blockers` list includes all 8 memd-core backlog items. Added **Roadmap-coverage rule** demanding every backlog item carry `phase:` frontmatter.
10. **No source code touched.** All infra/docs.

## Current repo state

- branch: `research/mining`
- working tree: **dirty** (this packet rewrite + phase renames + new A3 doc + backlog adds + ROADMAP patches all uncommitted)
- last clean commit before reshuffle-2: `d0c00dd docs: rename V3 phases to execution order`
- `ROADMAP_STATE`: `current_phase=A3`, `phase_status=pending`, `next_step=A3 Part 1 — file-interaction ledger + wake Files-Touched block (memd Continuity Foundation, the new V3 entry phase)`
- memd: new A3 identity + roadmap-coverage rule + class-failure-is-memd's decisions logged

## Next move — A3 Part 1 (state continuity)

Phase doc: [[docs/phases/phase-a3-continuity-foundation.md]] — read it. Three parts; Part 1 is the immediate entry.

**Part 1 deliverables (state continuity — the "memd never forgets" contract):**

1. **File-interaction ledger persisted per session** — pre-compact hook records `{file_path, op: read|edit|write, count, last_ts}` to `.memd/state/session-<id>/file_interactions.json`. Wake packet surfaces `## Files Touched` so continuation session bulk-Reads before any Edit.
   - Acceptance: simulated mid-edit compaction → continuation session runs ≥10 Edits with zero `File has not been read yet` errors.
   - Natural anchor: existing `.memd/hooks/memd-precompact-save.sh`.
2. **`memd prime-reads` command** — emits newline list of paths from prior session ledger. Continuation session mass-Reads them in one parallel batch.
3. **Working-memory lifecycle enforcement** — promote/expire/archive runs on phase-complete, checkpoint, and wake. Self-test (cron-style) reports green/red.
4. **Preferences persist across sessions + compaction** — A→B→A replay test proves it.
5. **Live memory contract** — machine-readable `.memd/contract.json` listing wake/checkpoint/resume guarantees. Cross-harness validators consume it.

Part 2 is **enforcement** (hooks consolidation, cross-harness pre-send validator, drift detection, strict mode). Part 3 is **codebase organization** (backlog INDEX, subfolder grouping, `.memd/` flatten). Read the phase doc.

Pass gate summary (from phase-a3-continuity-foundation.md):
- Compaction-mid-edit scenario: zero re-Read errors across ≥10 Edits.
- `memd prime-reads` returns non-empty list after any session with ≥1 file interaction.
- Lifecycle self-test cron green ≥99%.
- `find . -name "memd-*.sh" -path "*/hooks/*"` returns files in exactly one canonical tree.
- Every backlog file has `phase:` set (no unassigned at merge time).
- `docs/backlog/INDEX.md` regenerated by `make backlog-index`.

## Donor anchors (read before code on A3)

- Existing `.memd/hooks/memd-precompact-save.sh` — natural anchor for ledger write hook
- Existing wake-packet assembly code — natural anchor for `## Files Touched` block
- memd lifecycle code (promote/expire/archive) — partially implemented per `2026-04-16-pipeline-lifecycle-broken.md`
- `2026-04-17-memd-process-too-soft-cross-harness.md` — fix spec for cross-harness validator

B3-F3 donor anchors (retrieval/rerank/atlas/consolidation/bench) unchanged from their phase docs; relevant only once A3 is green.

## Outstanding before A3 coding starts

1. **Commit the uncommitted reshuffle-2** — phase renames + new A3 doc + backlog adds + ROADMAP + this handoff. Use `memd checkpoint --auto-commit --roadmap-set current_phase=A3 --roadmap-set phase_status=pending --content "V3 reshuffle-2: new A3 = memd Continuity Foundation"`.
2. **Add `phase:` frontmatter to all 84 existing backlog items** — per roadmap-coverage rule. Most map to M4 phases (I2/M2-evo/N2) or V3 phases (A3/B3/C3/D3/E3/F3); a few may need `phase: unassigned` pending triage. This is itself an A3 Part 3 deliverable.
3. **Write `make backlog-index` target** — generates `docs/backlog/INDEX.md` from frontmatter, grouped by phase/milestone/severity. Also an A3 Part 3 deliverable.
4. **Update `docs/WHERE-AM-I.md`** to point at the new A3 = Continuity Foundation.

## Parallel option: F3.1 (ConvoMem adapter)

`F3 Bench Honesty` deliverable 1 (ConvoMem adapter fix) is an adapter/routing bug, not a retrieval problem. Parallelizable with A3 on a side branch off `main`. Current score 0.000 is a shape mismatch between `Salesforce/ConvoMem evidence_questions` and memd's adapter. Write failing test → trace adapter → fix → prove green. Formal F3 phase merge still sits at end of V3.

## M4 deferred — what's parked

Plan: `docs/plans/M4-EXECUTION-PLAN.md` (authoritative when M4 resumes).

- `I2 Human Dashboard` — 11 substeps; entry is `I2.2 fix EntitySearchResult type mismatch`. See [[docs/handoff/2026-04-16-L2-complete-next-I2.md]].
- `M2-evo Overnight Evolution` — infra; E3 may cherry-pick dream-loop pieces.
- `N2 Integrations Polish` — last in M4 order.

K2 + L2 already done on `main` + `research/mining`.

## Open questions for next session

1. Start A3 Part 1 solo, or queue F3.1 ConvoMem adapter in parallel on a side branch off `main`?
2. File-interaction ledger schema — JSON flat file per session, or SQLite table in the same store as working memory? (Decision deferred to A3 planning phase.)
3. Hooks canonical tree — symlinks from `.claude/hooks/` and `integrations/hooks/` (easy, transparent), or install-generated copies (safer on non-POSIX, heavier maintenance)? (Decision deferred to A3 Part 2.)
4. Backlog frontmatter migration — one commit (big diff) or per-milestone commits? (Recommend: one commit per milestone subfolder to keep reviewable.)

## Boot orientation

`memd wake --output .memd` + reading this packet is the whole orientation. Do not re-grep the codebase to rediscover state.

One git command to confirm branch tip matches this packet's claim:

```bash
git log --oneline -1
```

If last clean commit is still `d0c00dd` and tree is dirty with the reshuffle-2 files, state matches this packet. Commit before coding.

If the commit landed already (next session picking up after a commit), expect a commit message like `memd auto-commit: V3 reshuffle-2 — A3 = Continuity Foundation` or similar.

If that fails, state has drifted — re-read memd, not the codebase.
