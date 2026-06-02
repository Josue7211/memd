# memd Hermes Agent UX Parity Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task. Do not edit Hermes Agent. Hermes/OpenClaw are references only.

**Goal:** Make memd setup/settings/status feel like Hermes Agent CLI: friendly default interactive TTY UX, clear sections, redacted status, useful config/settings surface, script-safe non-TTY behavior, and no `--interactive` flag.

**Architecture:** Keep memd's current Rust/Clap runtime. Add a small shared terminal UX layer for branded boxes, section headers, checkmarks, redaction, and selector rendering. Route `memd setup`, `memd settings`, and `memd status` through that layer while preserving `--summary` and `--json` for scripts.

**Tech Stack:** Rust, clap, serde_json, console, existing memd-client bundle/config/render modules, existing `scripts/memd-cargo-guard.sh` test wrapper.

---

## Non-Negotiables

- **memd only. No Hermes Agent edits.**
- **No `--interactive` flag.** Bare TTY commands are interactive when useful. Script/headless commands are non-interactive.
- **Keep `memd setup` canonical.** Do not rename to init.
- **Keep `memd settings`.** It aliases config but should present as settings in help/title/output when invoked that way if feasible.
- **Idempotent setup.** Existing `.memd` is a normal state, not an error.
- **Proof before claim.** Every phase ends with exact commands and expected outputs.
- **No secrets in output.** Redact token/API-key-looking values; never print full secrets.

---

## Current Gap Snapshot

Observed from live memd on desktop:

- `memd --help` has many commands with blank descriptions. Hermes has short useful descriptions for every command.
- `memd setup --help` exposes many low-level flags with blank descriptions. Hermes setup has simple section grammar.
- `memd settings --help` says `Usage: memd config ...`, not `memd settings ...`.
- `memd status --summary` works but default status should have Hermes-like sections.
- Existing setup menu uses a box and selectors, but it is still thin: only provider + harness, little explanatory guidance, no full setup sections.
- Existing plan file was too vague. This file replaces it.

---

## Target UX Contract

### Command grammar

```text
memd setup [section] [--non-interactive] [--reset]
memd setup provider|harness|memory|voice|hive|proof
memd settings [list|get|set|reset|show-schema] [...]
memd config [list|get|set|reset|show-schema] [...]
memd status [--all] [--deep] [--summary] [--json]
```

Notes:
- `--non-interactive` is allowed because Hermes has it and semantics are the reverse of `--interactive`.
- If adding `--non-interactive` is too much for this pass, use existing explicit flags + non-TTY detection, but do not add `--interactive`.
- `settings` and `config` can share implementation.

### `memd setup` bare TTY

Open a Hermes-like wizard:

```text
┌─────────────────────────────────────────────────────────┐
│                    ◈ memd Setup                         │
└─────────────────────────────────────────────────────────┘

◆ Provider
  Choose where memd connects first.

  ● Local memd server  http://100.104.154.24:8787
  ○ Custom server
  ○ Offline/project bundle only

  Enter select   ↑/↓ move   Esc skip   Ctrl+C exit
```

Sections:
1. Provider — local/default/custom/offline.
2. Harness — Codex, Claude Code, OpenCode, OpenClaw, Hermes bridge surfaces as applicable.
3. Memory bundle — project/global/output path, idempotent existing bundle handling.
4. Voice/defaults — voice mode, route, intent, visibility.
5. Hive/group — optional shared identity/group settings.
6. Proof — writes/refreshes bundle, prints next commands.

### `memd setup` non-TTY/script

- Does not hang.
- With no args, prints non-interactive guidance + exits 0 or performs safe idempotent defaults. Pick one behavior in task below and test it.
- `--summary` remains one-line machine-friendly.
- `--json` remains structured.

### `memd settings`

Default TTY output should be human-friendly:

```text
┌─────────────────────────────────────────────────────────┐
│                   ◈ memd Settings                       │
└─────────────────────────────────────────────────────────┘

◆ Runtime
  Bundle:      /path/.memd
  Project:     memd
  Namespace:   main
  Agent:       codex
  Session:     session-...

◆ Connection
  Base URL:    http://100.104.154.24:8787
  Route:       auto
  Intent:      current_task

◆ Defaults
  Voice:       caveman-ultra
  Authority:   shared
  Auto commit: off
```

- `memd settings --help` should not say only `Usage: memd config` if user typed settings.
- `memd settings list/get/set/reset/show-schema` stays script-friendly.

### `memd status`

Default TTY output should mimic Hermes status sections:

```text
┌─────────────────────────────────────────────────────────┐
│                    ◈ memd Status                        │
└─────────────────────────────────────────────────────────┘

◆ Environment
  Bundle:       ✓ /path/.memd
  Project:      memd
  Namespace:    main
  Agent:        codex
  Voice:        caveman-ultra

◆ Server
  API:          ✓ ok  http://100.104.154.24:8787
  RAG:          ○ off
  Atlas:        ✓ active  edges=1579 regions=16 ratio=12.34

◆ Memory
  Setup:        ✓ ready
  Session:      session-...
  Capabilities: 1280 total, 47 universal, 1227 native
```

- `--summary` remains existing one-line style.
- `--json` should be added if missing.
- `--all` includes extra details but still redacts secrets.
- `--deep` may run slower checks.

---

## Implementation Tasks

### Task 1: Commit or stash current dirty work before new edits

**Objective:** Preserve existing setup/settings fixes before larger UX work.

**Files:** none modified by task.

**Steps:**

1. Inspect dirty state:

```bash
ssh josue@100.97.74.2 'cd /home/josue/Documents/projects/memd && git status --short'
```

Expected: current modified files plus plan files.

2. Review diff quickly:

```bash
ssh josue@100.97.74.2 'cd /home/josue/Documents/projects/memd && git diff --stat && git diff -- crates/memd-client/src/cli/args.rs crates/memd-client/src/cli/args_memory.rs crates/memd-client/src/bundle/admin_runtime.rs crates/memd-client/src/bundle/config_runtime.rs | sed -n "1,240p"'
```

3. Run current proof before commit:

```bash
ssh josue@100.97.74.2 'cd /home/josue/Documents/projects/memd && bash scripts/memd-cargo-guard.sh -- test -p memd-client setup_interactive_tests -- --nocapture && bash scripts/memd-cargo-guard.sh -- test -p memd-client --no-run && ~/.local/bin/memd setup --help | grep -v -- --interactive && ~/.local/bin/memd settings --summary'
```

Expected: tests pass; help has no `--interactive`; settings summary prints ready line.

4. Commit current UX base:

```bash
ssh josue@100.97.74.2 'cd /home/josue/Documents/projects/memd && git add crates/memd-client/src/cli/args.rs crates/memd-client/src/cli/args_memory.rs crates/memd-client/src/bundle/admin_runtime.rs crates/memd-client/src/bundle/config_runtime.rs .hermes/plans && git commit -m "feat: add memd setup and settings UX base"'
```

5. Verify clean or only expected new work:

```bash
ssh josue@100.97.74.2 'cd /home/josue/Documents/projects/memd && git status --short'
```

Expected: clean.

---

### Task 2: Add shared Hermes-like terminal UX renderer

**Objective:** Stop hand-rolling boxes in setup; create reusable status/settings/setup rendering helpers.

**Files:**
- Create: `crates/memd-client/src/cli/terminal_ux.rs`
- Modify: `crates/memd-client/src/cli/mod.rs`

**Implementation:**

Create helpers with no business logic:

```rust
pub(crate) fn boxed_title(title: &str) -> String { /* 57-col Hermes-like box */ }
pub(crate) fn section(title: &str) -> String { format!("
◆ {title}
") }
pub(crate) fn check(ok: bool) -> &'static str { if ok { "✓" } else { "✗" } }
pub(crate) fn radio_line(selected: bool, label: &str) -> String { /* ●/○ line */ }
pub(crate) fn field(label: &str, value: impl AsRef<str>) -> String { /* aligned two-col */ }
pub(crate) fn redact(value: &str) -> String { /* short + secret-looking redaction */ }
```

Keep ANSI/color optional. If existing color support is absent, plain Unicode is enough.

**Tests:**

Add unit tests in same file:

- `boxed_title_contains_title_and_box_edges`
- `redact_hides_long_secret_values`
- `field_aligns_labels`

Run:

```bash
ssh josue@100.97.74.2 'cd /home/josue/Documents/projects/memd && bash scripts/memd-cargo-guard.sh -- test -p memd-client terminal_ux -- --nocapture'
```

Expected: all new tests pass.

---

### Task 3: Replace setup menu renderer with shared UX helpers

**Objective:** Make setup wizard visually match Hermes patterns and keep behavior unchanged.

**Files:**
- Modify: `crates/memd-client/src/bundle/admin_runtime.rs`
- Test: existing `setup_interactive_tests` module in same file

**Steps:**

1. Import terminal UX helpers.
2. Rewrite `render_interactive_menu` to use `boxed_title`, `section`, `radio_line`.
3. Use `●/○` selectors, not `◆/◇` for radio options; reserve `◆` for section headers like Hermes.
4. Keep arrow/Enter/q behavior as currently implemented.
5. Update snapshot-ish assertions to check:
   - title contains `memd Setup`
   - section line starts `◆ Provider` or matching section
   - selected option uses `●`
   - non-selected option uses `○`
   - no `Choice [default]`
   - no `--interactive`

**Verification:**

```bash
ssh josue@100.97.74.2 'cd /home/josue/Documents/projects/memd && bash scripts/memd-cargo-guard.sh -- test -p memd-client setup_interactive_tests -- --nocapture'
```

Expected: setup renderer tests pass.

---

### Task 4: Add explicit setup sections without renaming setup

**Objective:** Match Hermes `setup [section]` UX while keeping memd's domain sections.

**Files:**
- Modify: `crates/memd-client/src/cli/args_memory.rs`
- Modify: `crates/memd-client/src/bundle/admin_runtime.rs`

**CLI shape:**

Add optional positional enum/field:

```rust
#[arg(value_enum)]
pub(crate) section: Option<SetupSection>,

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum SetupSection {
    Provider,
    Harness,
    Memory,
    Voice,
    Hive,
    Proof,
}
```

Do not add `Interactive` section/flag.

**Runtime:**

- If `section` is set and TTY, open only that picker/details step.
- If `section` is set and non-TTY, print section-specific guidance or apply explicit flags.
- Existing `--summary` should bypass interactive section handling.

**Tests:**

Add tests:
- `setup_help_lists_sections_without_interactive_flag`
- `setup_provider_section_renders_provider_picker`
- `setup_proof_section_prints_proof_commands`

**Verification:**

```bash
ssh josue@100.97.74.2 'cd /home/josue/Documents/projects/memd && bash scripts/memd-cargo-guard.sh -- test -p memd-client setup_section -- --nocapture && bash scripts/memd-cargo-guard.sh -- run -p memd-client -- setup --help | tee /tmp/memd-setup-help.txt && ! grep -q -- "--interactive" /tmp/memd-setup-help.txt && grep -q "provider" /tmp/memd-setup-help.txt'
```

Expected: tests pass; setup help shows sections and no `--interactive`.

---

### Task 5: Add `--non-interactive` setup flag with reversed semantics

**Objective:** Provide a clear headless escape hatch like Hermes, without violating no-`--interactive` rule.

**Files:**
- Modify: `crates/memd-client/src/cli/args_memory.rs`
- Modify: `crates/memd-client/src/bundle/admin_runtime.rs`
- Modify: any `SetupArgs` constructors, likely `crates/memd-client/src/bundle/config_runtime.rs`

**Behavior:**

- `memd setup` in TTY: wizard.
- `memd setup --non-interactive`: no wizard; safe defaults/idempotent update.
- `memd setup --summary`: no wizard, summary only.
- non-TTY bare setup: no wizard; print guidance or safe defaults. Prefer safe idempotent defaults if current behavior already does that.

**Tests:**

- `setup_non_interactive_flag_skips_picker`
- `setup_summary_skips_picker`
- `setup_non_tty_skips_picker`
- grep test for no `--interactive`

**Verification:**

```bash
ssh josue@100.97.74.2 'cd /home/josue/Documents/projects/memd && bash scripts/memd-cargo-guard.sh -- test -p memd-client setup_non_interactive -- --nocapture && bash scripts/memd-cargo-guard.sh -- run -p memd-client -- setup --non-interactive --summary && bash scripts/memd-cargo-guard.sh -- run -p memd-client -- setup --help | grep -- --non-interactive && ! bash scripts/memd-cargo-guard.sh -- run -p memd-client -- setup --help | grep -- --interactive'
```

Expected: `--non-interactive` exists; `--interactive` absent.

---

### Task 6: Make settings render as settings, not config

**Objective:** User typed `memd settings`; help and default UI should not look like a hidden config alias.

**Files:**
- Modify: `crates/memd-client/src/cli/args.rs`
- Modify: `crates/memd-client/src/bundle/config_runtime.rs`
- Possibly modify command dispatch file if separate from args.

**Approach:**

Best option: make `settings` its own Clap subcommand variant that reuses `ConfigArgs` internally.

Example shape:

```rust
Config(ConfigArgs),
Settings(ConfigArgs),
```

Dispatch both to same runtime with an invocation label:

```rust
run_bundle_config_command(args, ConfigSurface::Settings)
run_bundle_config_command(args, ConfigSurface::Config)
```

If Clap alias cannot show proper usage for alias, separate variant is required.

**Runtime output:**

- `memd config --help`: can say config.
- `memd settings --help`: must say settings.
- `memd settings --summary`: can keep one-line `config ...` or switch to `settings ...`; prefer `settings ...` for user-facing parity.
- `memd settings` default TTY renders settings sections.

**Tests:**

- `settings_help_uses_settings_name`
- `settings_summary_uses_settings_prefix`
- `config_summary_still_works`

**Verification:**

```bash
ssh josue@100.97.74.2 'cd /home/josue/Documents/projects/memd && bash scripts/memd-cargo-guard.sh -- test -p memd-client settings_surface -- --nocapture && bash scripts/memd-cargo-guard.sh -- run -p memd-client -- settings --help | grep "Usage: memd settings" && bash scripts/memd-cargo-guard.sh -- run -p memd-client -- settings --summary'
```

Expected: help says `memd settings`; summary works.

---

### Task 7: Add human-friendly `memd settings` default output

**Objective:** Match Hermes config/status style for settings without breaking script modes.

**Files:**
- Modify: `crates/memd-client/src/bundle/config_runtime.rs`
- Use: `crates/memd-client/src/cli/terminal_ux.rs`

**Behavior:**

- `memd settings` in TTY/default prints boxed title and sections.
- `memd settings --summary` stays one-line.
- `memd settings --json` stays JSON.
- Values are aligned and redacted.

**Sections:**

- Runtime: bundle, project, namespace, agent, session.
- Connection: base_url, rag_url if present, route, intent.
- Defaults: voice, authority, visibility, workspace, auto_commit.
- Health: ready/degraded.

**Tests:**

- `settings_default_renders_sections`
- `settings_redacts_secret_like_values`
- `settings_summary_stays_machine_friendly`
- `settings_json_stays_valid_json`

**Verification:**

```bash
ssh josue@100.97.74.2 'cd /home/josue/Documents/projects/memd && bash scripts/memd-cargo-guard.sh -- test -p memd-client settings_render -- --nocapture && bash scripts/memd-cargo-guard.sh -- run -p memd-client -- settings | sed -n "1,80p" && bash scripts/memd-cargo-guard.sh -- run -p memd-client -- settings --json >/tmp/memd-settings.json && python3 -m json.tool /tmp/memd-settings.json >/dev/null'
```

Expected: sections visible; JSON valid.

---

### Task 8: Add Hermes-like default `memd status` renderer

**Objective:** Make status readable by humans like Hermes, while preserving summary.

**Files:**
- Modify: `crates/memd-client/src/cli/args_memory.rs` or status arg file
- Modify: status runtime file; locate with `grep -R "render_bundle_status_summary\|run_.*status" crates/memd-client/src`
- Modify: `crates/memd-client/src/render/mod.rs`

**Behavior:**

- Default `memd status`: boxed title + sections.
- `memd status --summary`: existing one-line output.
- Add `--json` if missing and wire to existing status JSON object.
- Add `--all` and `--deep` if not present. If deep checks already exist elsewhere, wire them. If not, add flags as no-op documented placeholders only if tests assert no behavior beyond accepted parsing.

**Sections:**

- Environment: bundle, project, namespace, agent, voice.
- Server: API, RAG, Atlas.
- Memory: setup ready, session, capabilities.
- Warnings: degraded/missing setup/atlas warning.

**Tests:**

- `status_default_renders_hermes_sections`
- `status_summary_unchanged`
- `status_json_valid`
- `status_redacts_secrets`

**Verification:**

```bash
ssh josue@100.97.74.2 'cd /home/josue/Documents/projects/memd && bash scripts/memd-cargo-guard.sh -- test -p memd-client status_render -- --nocapture && bash scripts/memd-cargo-guard.sh -- run -p memd-client -- status | sed -n "1,100p" && bash scripts/memd-cargo-guard.sh -- run -p memd-client -- status --summary && bash scripts/memd-cargo-guard.sh -- run -p memd-client -- status --json >/tmp/memd-status.json && python3 -m json.tool /tmp/memd-status.json >/dev/null'
```

Expected: default human sections; summary unchanged; JSON valid.

---

### Task 9: Clean up top-level help descriptions

**Objective:** Make `memd --help` look deliberate like Hermes instead of a command dump with blanks.

**Files:**
- Modify: `crates/memd-client/src/cli/args.rs`
- Possibly `crates/memd-client/src/cli/args_memory.rs`

**Rules:**

- Add one-line descriptions for high-value commands: setup, settings, status, doctor, wake, refresh, lookup, remember, teach, config, memory, skills, hooks, init.
- Do not fully rewrite every obscure/internal command in this pass unless easy.
- Put setup/settings/status near discoverable positions if Clap order can be adjusted safely.
- Keep aliases visible where useful.

**Tests:**

- `top_level_help_describes_setup_settings_status`
- `top_level_help_has_no_blank_for_core_commands`

**Verification:**

```bash
ssh josue@100.97.74.2 'cd /home/josue/Documents/projects/memd && bash scripts/memd-cargo-guard.sh -- test -p memd-client help_surface -- --nocapture && bash scripts/memd-cargo-guard.sh -- run -p memd-client -- --help | sed -n "1,140p"'
```

Expected: core commands have descriptions.

---

### Task 10: Golden/snapshot tests for UX contract

**Objective:** Lock the new UX so it does not regress.

**Files:**
- Create/modify tests under existing Rust unit test modules, or create `crates/memd-client/tests/ux_parity.rs` if integration tests already exist.

**Test cases:**

1. `setup_help_has_no_interactive_and_has_sections`
2. `settings_help_uses_settings_usage`
3. `status_default_has_environment_server_memory_sections`
4. `summary_outputs_are_single_line`
5. `json_outputs_parse`
6. `secret_redaction_does_not_print_full_secret`

**Verification:**

```bash
ssh josue@100.97.74.2 'cd /home/josue/Documents/projects/memd && bash scripts/memd-cargo-guard.sh -- test -p memd-client ux_parity -- --nocapture'
```

Expected: all UX parity tests pass.

---

### Task 11: Install live binary and verify user shell behavior

**Objective:** Avoid stale `/home/josue/.local/bin/memd` issue recurring.

**Files:** none.

**Commands:**

```bash
ssh josue@100.97.74.2 'cd /home/josue/Documents/projects/memd && cargo install --path crates/memd-client --locked --force && cp ~/.cargo/bin/memd ~/.local/bin/memd && hash -r && which memd && memd --help | sed -n "1,120p" && memd setup --help | sed -n "1,120p" && memd settings --help | sed -n "1,120p" && memd status | sed -n "1,120p"'
```

Expected:
- `which memd` is `/home/josue/.local/bin/memd`.
- setup help has no `--interactive`.
- settings help says `Usage: memd settings`.
- status default has Hermes-like sections.

---

### Task 12: Final full proof and commit

**Objective:** Land only after tests and live proof pass.

**Commands:**

```bash
ssh josue@100.97.74.2 'cd /home/josue/Documents/projects/memd && bash scripts/memd-cargo-guard.sh -- test -p memd-client --no-run && bash scripts/memd-cargo-guard.sh -- test -p memd-client setup -- --nocapture && bash scripts/memd-cargo-guard.sh -- test -p memd-client settings -- --nocapture && bash scripts/memd-cargo-guard.sh -- test -p memd-client status -- --nocapture && bash scripts/memd-cargo-guard.sh -- run -p memd-client -- setup --help | grep -v -- --interactive && bash scripts/memd-cargo-guard.sh -- run -p memd-client -- settings --summary && bash scripts/memd-cargo-guard.sh -- run -p memd-client -- status --summary && git status --short'
```

Expected: all tests pass; no `--interactive`; summaries work; only intended files dirty.

Commit:

```bash
ssh josue@100.97.74.2 'cd /home/josue/Documents/projects/memd && git add crates/memd-client/src .hermes/plans && git commit -m "feat: match Hermes-style memd UX" && git status --short'
```

Expected: commit created; status clean.

---

## Acceptance Criteria

- `memd setup --help` contains `--non-interactive` if implemented and never contains `--interactive`.
- Bare TTY `memd setup` opens a branded wizard.
- `memd setup --summary` is non-interactive and one-line.
- `memd setup` is idempotent when `.memd` exists.
- `memd settings --help` says `Usage: memd settings`, not just config.
- `memd settings` default output has Hermes-like boxed title and sections.
- `memd settings --summary` and `--json` remain script-safe.
- `memd status` default output has Hermes-like boxed title and Environment/API/Memory sections.
- `memd status --summary` remains one-line.
- Top-level help has useful descriptions for setup/settings/status/doctor/config/memory basics.
- Live installed `/home/josue/.local/bin/memd` matches source behavior.
- No Hermes Agent files changed.

---

## Risk Controls

- If Clap alias cannot customize `settings` help text, use a separate `Settings(ConfigArgs)` variant.
- If status JSON does not exist yet, add it by printing the existing status `serde_json::Value` before rendering.
- If a full TTY wizard is hard to test, test pure renderer functions and `should_run_setup_picker` logic separately.
- If `--non-interactive` creates scope churn, leave it for a follow-up but still do not add `--interactive`.
- Keep all secret values redacted by default; `--all` must still avoid raw token output unless project already has a safe redaction contract.

---

## Execution Mode Recommendation

Use 4 isolated workstreams after Task 1 commit:

1. **Setup UX:** Tasks 2-5.
2. **Settings UX:** Tasks 6-7.
3. **Status/help UX:** Tasks 8-9.
4. **Parity tests/install:** Tasks 10-12.

Merge only branches that pass their task-specific proof. Then run Task 12 final proof on main.
