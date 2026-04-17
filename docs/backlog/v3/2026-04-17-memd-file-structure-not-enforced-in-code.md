---
status: open
severity: high
phase: A3
opened: 2026-04-17
scope: harness-core, process, cli, contract
---
# memd File Structure Not Enforced In Code

- status: `open`
- severity: `high`
- phase: `A3`
- opened: `2026-04-17`
- scope: harness-core, process, cli, contract

## Problem

memd relies on prose conventions (`wake.md`, `docs/WHERE-AM-I.md`, handoff
packets, phase docs) to tell agents *where files go*. Nothing in memd code
enforces that contract. An agent can freely drop a plan into
`docs/superpowers/plans/` when canonical locations are `docs/plans/` or
`docs/specs/`, and memd will not complain. memd file structure must be
hardcoded into memd — as a runtime contract + a gated write path — not relied
on as convention the assistant is expected to remember.

User directive 2026-04-17 (canonical in memd):
> *"the memd file structure should be BUILT and coded into memd"*
> *"if you dont understand memd process that is a bug"*
> *"this is in the backlog"*
> *"this is not the memd process"* — re: plan landing under
> `docs/superpowers/plans/` instead of `docs/plans/` / `docs/specs/`

The class of failure is identical to `memd-process-too-soft-cross-harness.md`
but scoped specifically to **file layout**. Continuity (A3 Part 1/2) built the
ledger + enforcement gate for *file reads*; this item extends the same
enforcement spine to *file writes*, constrained by a canonical layout schema.

## Evidence

- session 2026-04-17 (`claude-code@session-7eab5dde`): user asked for A3 Part 3
  plan; assistant invoked `superpowers:writing-plans` skill; plan landed at
  `docs/superpowers/plans/2026-04-17-a3-part3-codebase-organization.md`; user
  had to catch the wrong location manually and direct deletion. nothing in
  memd's PreToolUse gate flagged the write target.
- `docs/superpowers/plans/` currently holds 9 active + 17 archived plans; A3
  Part 1 and Part 2 plans both landed there; the pattern was inherited, not
  caught, across three phases.
- `docs/WHERE-AM-I.md` line 26 references `docs/superpowers/plans/` as if
  canonical (stale; compounds the drift).
- ROADMAP.md only links `docs/plans/M{1,2,3,4}-EXECUTION-PLAN.md` as
  authoritative execution plans — the canonical location is clear to a human
  reader, but invisible to any runtime guard.
- plan surface sprawl across `docs/plans/`, `docs/superpowers/plans/`,
  `docs/superpowers/specs/`, `docs/specs/`, `docs/reference/archive/` with no
  single machine-readable registry.

## Root Cause

1. `.memd/contract.json` (Part 2, version `0.2.0`) covers continuity guarantees
   only — nothing about *where files are allowed to go*.
2. `memd hook gate` (Part 2) filters Read staleness on ledgered paths but has
   no schema for write targets.
3. wake packet has no `## Layout Map` block — the canonical dirs are nowhere
   surfaced as startup truth.
4. `.memd/agents/` imports ship prose ("use docs/plans/ for execution plans")
   but never as a tested contract.
5. no `memd doctor structure` subcommand auditing repo against a schema.

## Fix

1. Extend `.memd/contract.json` with a `file_layout` section: canonical paths
   per artifact kind (`plans`, `specs`, `backlog`, `phases`, `handoff`,
   `theory`, `policy`, `verification`, `docs_root`, `code`, `hooks`) plus a
   `denylist` (e.g. `docs/superpowers/**`).
2. Grow `ContractGuarantees` with `enforces_file_layout_contract` + evidence
   (`file_layout_registered`, `file_layout_gate_blocks_denylist`). Contract
   version bumps `0.2.0` → `0.3.0`.
3. Extend `memd hook gate` (Part 2 infrastructure) to match Edit/Write
   `file_path` against the layout schema: denylist match → deny; non-canonical
   kind match → warn with pointer to canonical location; exact canonical match
   → allow.
4. `memd doctor structure` subcommand: scan repo, report plans outside
   canonical, backlog items missing `phase:` frontmatter, handoff dir missing
   `LATEST.md`, `.memd/` entries outside schema. CI fails on non-zero drift.
5. Wake packet emits `## Layout Map` block (top-N canonical paths by kind) so
   the assistant never has to infer. Sits alongside `## Files Touched`
   (Part 1) and `## Continuity Gate` (Part 2).
6. `.memd/agents/CLAUDE_IMPORTS.md` + `.memd/agents/codex.sh` refresh: replace
   prose with a short line pointing at the machine contract.

## Acceptance

- `.memd/contract.json` version `0.3.0` shipped; `memd contract verify`
  validates `file_layout` section.
- writing to `docs/superpowers/plans/*.md` (or any denylisted path) via Edit or
  Write is blocked by `memd hook gate` with deny JSON pointing at canonical
  location; verified by integration test mimicking session 2026-04-17's mis-write.
- writing a plan to `docs/plans/` succeeds silently; writing to `docs/specs/`
  succeeds silently.
- `memd doctor structure` exits 0 on clean tree, non-zero after dropping a
  stray `docs/superpowers/plans/test.md`.
- wake packet on fresh boot includes `## Layout Map` block listing canonical
  dirs by kind.
- cross-harness parity: same contract fires identically in Claude Code, Codex,
  OpenCode (policy=warn minimum; policy=block optional per
  `.memd/config.json`).

## Risk

- over-eager gate blocks legitimate authoring outside the schema (e.g. user
  writes to a new dir they intend to canonicalize). Mitigate by shipping
  policy=`warn` default; graduate to `block` only after drift goes to zero.
- canonical set is a small schema but must be agreed upstream; codify in
  `.memd/contract.json` as the one source, never scatter across prose.
- denylist for `docs/superpowers/**` will catch live files that currently
  ship there (Part 1/2 plans, older specs). Migration must land before
  denylist goes hard — schedule as part of A3 Part 3.

## Relationship to other items

- Supports `2026-04-17-memd-process-too-soft-cross-harness.md` — file layout is
  one axis of "process too soft"; the fix here piggy-backs on the same
  enforcement gate.
- Depends on `2026-04-17-codebase-organization-pass.md` — canonical layout
  must land before enforcement can guard it.
- Builds on A3 Part 2 enforcement infrastructure (`gate_decision`,
  `memd hook gate`, `MemdContract`, `EnforcementPolicy`) — no new enforcement
  class needed, just widen the schema.
- Resolves the session 2026-04-17 drift class: the same bug cannot happen
  after this item ships.
