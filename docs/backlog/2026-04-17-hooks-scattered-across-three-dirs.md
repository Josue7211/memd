# Hooks Scattered Across Three Directories

- status: `open`
- severity: `medium`
- phase: `V2-N2` (Integrations Polish) or a new dedicated organization pass
- opened: `2026-04-17`
- scope: repo-structure, harness-integrations

## Problem

memd hook scripts currently live in three separate directories with near-total
duplication, and the count is growing. There is no single source of truth for
"what hooks does memd ship and where do they live." User quote (2026-04-17):
*"we also need to make a folder in memd for all the hooks we have, were
getting to have a LOT of hooks"* and *"the codebase has to be more organized"*.

## Evidence

Current layout:

- `.memd/hooks/` (14 files): `install.{sh,ps1}`, `memd-bootstrap.sh`,
  `memd-capture.{sh,ps1}`, `memd-context.{sh,ps1}`, `memd-precompact-save.{sh,ps1}`,
  `memd-spill.{sh,ps1}`, `memd-stop-save.{sh,ps1}`, `README.md`
- `integrations/hooks/` (14 files): byte-identical or near-identical copies
  of the same scripts (appears to be a mirror)
- `.claude/hooks/` (1 file): `memd-bootstrap.sh` — the harness-entry point
  for Claude Code specifically

Issues:
1. Duplication between `.memd/hooks/` and `integrations/hooks/` — one must be
   canonical, the other should be a link, generator output, or deleted
2. No per-harness subdirectory, so adding Codex/Grok/Kimi/OpenCode/Gemini
   hooks will worsen the sprawl
3. `.claude/hooks/memd-bootstrap.sh` is the only file in its dir and
   duplicates an `.memd/hooks/memd-bootstrap.sh` — harness-specific entry
   vs canonical installer role is unclear
4. README.md exists in two of the three dirs with potentially-divergent content

## Root Cause Hypotheses

1. `.memd/hooks` is the product, `integrations/hooks` is the shipped bundle
   from older integration-layer work — never reconciled
2. `.claude/hooks` was added when Claude Code harness integration was bolted
   on, separate from the canonical set
3. no canonical "hooks manifest" lists shipped hooks + install targets +
   which harnesses consume which

## Fix

- pick ONE canonical directory. Likely `.memd/hooks/` since it already carries
  the install scripts and README
- every harness gets a subdirectory: `.memd/hooks/harness/claude-code/`,
  `.memd/hooks/harness/codex/`, `.memd/hooks/harness/gemini/`, etc
- shared hooks (pre-compact save, capture, spill, stop-save) live at
  `.memd/hooks/shared/`
- `integrations/hooks/` is deleted or becomes a generated mirror (regenerated
  by `cargo run --bin memd-install-hooks`)
- `.claude/hooks/` becomes a symlink or install-generated directory pointing
  into `.memd/hooks/harness/claude-code/`
- new manifest file `.memd/hooks/MANIFEST.json` or `.toml` lists every hook,
  purpose, target harness, install target path, whether it is shared
- README at `.memd/hooks/README.md` is the single source of truth; other
  READMEs are deleted or redirected
- stretch: `memd hooks list` / `memd hooks install` / `memd hooks doctor`
  subcommands surface the manifest and verify installation state

## Acceptance

- `find . -name "memd-*.sh" -path "*/hooks/*"` returns files in exactly
  one canonical tree (plus harness-specific symlinks if applicable)
- `.memd/hooks/MANIFEST.json` lists every hook with harness + purpose
- adding a Codex hook means adding one file in `.memd/hooks/harness/codex/`
  and one manifest entry — no duplication work
- README explains the layout in ≤ 20 lines

## Relationship to other items

- `2026-04-17-memd-process-too-soft-cross-harness.md` — the hooks layout
  fix unblocks the cross-harness enforcement work, since the enforcement
  surface is the hook set
- `2026-04-17-memd-read-state-lost-across-compaction.md` — the fix touches
  `memd-precompact-save.sh`, so the hooks layout must be stable first
