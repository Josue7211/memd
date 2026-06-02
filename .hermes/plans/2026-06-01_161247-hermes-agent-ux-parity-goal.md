# Goal: memd 100% Hermes Agent UX parity

## Goal

Make `memd` setup/settings/status/config surfaces feel like Hermes Agent end-to-end, not a rough imitation.

Target: when a user runs `memd setup`, `memd settings`, `memd status`, or related first-run commands, the terminal UX should match Hermes Agent command grammar, visual rhythm, language, and beginner flow closely enough that the user recognizes it as the same product family.

## Scope

This is memd work only.

- Do not modify Hermes Agent.
- Do not add `--interactive`.
- Do not rename canonical `memd setup` to `memd init`.
- Do not fake readiness; every done/ready claim needs proof command output.

## UX contract

### Command grammar

- `memd setup` is the main setup surface.
- `memd settings` is the settings/config surface.
- Bare `memd setup` in a TTY opens the Hermes-style guided setup flow.
- Non-TTY or explicit/scripted modes run direct and do not hang:
  - `memd setup --summary`
  - `memd setup --json`
  - `memd setup --agent hermes --summary`
  - `memd settings --summary`
- No `--interactive` flag. Default behavior owns interactivity when appropriate.

### Visual parity

Replicate Hermes Agent terminal patterns:

- branded boxed header
- gold/orange/cyan-style hierarchy where supported
- memd-branded iconography using the same Hermes/OpenClaw terminal rhythm
- section headers with diamond glyphs
- `✓` and `✗` readiness markers
- status rows aligned like Hermes
- setup menu with selected/default route marker
- helper text matching Hermes tone: short, direct, beginner-friendly
- no generic CRUD wizard language
- no `Choice [default]` generic fallback if a branded prompt can be used

### Flow parity

`memd setup` should feel like Hermes Agent setup:

1. Welcome / existing install detection
2. Quick Setup — configure missing items only
3. Full Setup — reconfigure everything
4. Provider / server selection
5. Harness / agent surface selection
6. Memory behavior / voice / route / intent defaults
7. Settings surface
8. Doctor/proof summary
9. Clear next commands

### Content parity

`memd status` / `memd doctor` / `memd settings` should use Hermes-style sections:

- Environment
- Memory Bundle
- Runtime / Server
- Harnesses
- Authority / Hive
- Sessions / Wake / Recall readiness
- Diagnostics / next action

## Current context

Current memd work already includes:

- `memd settings` visible alias/surface
- `memd setup` idempotent over existing `.memd`
- live binary rebuilt in `/home/josue/.local/bin/memd`
- `--interactive` being removed from `memd setup --help`
- early setup menu styling started in `crates/memd-client/src/bundle/admin_runtime.rs`

But this is not enough. Need full parity pass across all relevant memd UX surfaces.

## Files likely to change

- `crates/memd-client/src/bundle/admin_runtime.rs`
- `crates/memd-client/src/bundle/config_runtime.rs`
- `crates/memd-client/src/render/mod.rs`
- `crates/memd-client/src/cli/args.rs`
- `crates/memd-client/src/cli/args_memory.rs`
- docs under `docs/setup/` if copy/examples need updating
- command catalog output if it includes setup/settings help text

## Implementation phases

### Phase 1 — UX inventory

Capture Hermes reference outputs and current memd outputs, then make a delta checklist.

Reference commands:

```bash
hermes status
hermes setup
hermes setup --help
```

memd commands:

```bash
memd setup --help
memd setup
memd setup --summary
memd settings --summary
memd status --summary
memd doctor --summary
```

### Phase 2 — Shared memd UX renderer

Create or consolidate renderer helpers for Hermes-style memd CLI output:

- brand header
- section header
- status row
- yes/no readiness row
- route menu
- summary footer / next commands

### Phase 3 — Setup parity

Make `memd setup` match Hermes flow:

- detect existing config and show Welcome Back equivalent
- show Quick Setup / Full Setup / section choices
- keep direct execution for non-TTY and explicit args
- write/update bundle idempotently
- after setup, show proof summary and next command list

### Phase 4 — Settings parity

Make `memd settings` feel like Hermes settings/config:

- if missing `.memd`, create enough bundle/config to show settings
- show aligned config status by default or summary mode
- support existing config subcommands without breaking scripts
- use Hermes-style headers and row sections

### Phase 5 — Status/doctor parity

Make status and doctor outputs share Hermes-style structure:

- boxed brand header
- sections with `◆`
- aligned labels
- `✓` / `✗` markers
- final next-action guidance

### Phase 6 — Tests

Add/extend tests for:

- no `--interactive` in help/guided docs
- bare setup render contains Hermes-style brand/menu markers
- setup render does not contain generic wizard/copy
- settings alias works
- setup idempotent with existing `.memd`
- status/doctor render has Hermes-style section structure
- non-TTY setup does not block

### Phase 7 — Verification

Run:

```bash
bash scripts/memd-cargo-guard.sh -- test -p memd-client setup_interactive_tests -- --nocapture
bash scripts/memd-cargo-guard.sh -- test -p memd-client --no-run
bash scripts/memd-cargo-guard.sh -- run -p memd-client -- setup --help
bash scripts/memd-cargo-guard.sh -- run -p memd-client -- setup --summary
bash scripts/memd-cargo-guard.sh -- run -p memd-client -- settings --summary
bash scripts/memd-cargo-guard.sh -- run -p memd-client -- status --summary
bash scripts/memd-cargo-guard.sh -- run -p memd-client -- doctor --summary
```

Then install live binary and verify:

```bash
cargo install --path crates/memd-client --locked --force
cp ~/.cargo/bin/memd ~/.local/bin/memd
hash -r
which memd
memd setup --help
memd setup --summary
memd settings --summary
memd status --summary
memd doctor --summary
```

## Acceptance criteria

- `memd setup --help` has no `--interactive`.
- `memd setup` in TTY opens a Hermes-style guided flow by default.
- `memd settings` works live.
- Existing `.memd` does not make setup fail.
- Output language/style is visibly Hermes Agent family, not generic CRUD wizard.
- Tests and live binary checks pass.
- No Hermes Agent repo edits.
