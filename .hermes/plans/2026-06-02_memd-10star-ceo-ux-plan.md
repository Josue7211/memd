# memd 10-Star CEO UX Remediation Plan

> **For Hermes:** Use subagent-driven-development skill. Execute in isolated worktrees, with spec review + quality review per workstream. Do not call UX production-ready until live installed binary proof passes.

**Goal:** Move memd from beta CLI UX to a production-grade, Hermes-family CLI experience for first-run setup, help, settings/config, status, and doctor.

**CEO Read:** Current UX is not 9/10. Treat it as **4/10 beta**. It has working mechanics and some branded rendering, but it still exposes implementation complexity, lacks color/density/interaction quality, and has no golden UX lock.

**Architecture:** Keep Clap and existing Rust command runtime. Add a first-class terminal UX layer that controls command discovery, wizard menus, status panels, and script-safe output. Split user help from internal command catalog. Add golden tests and live binary proof.

**Success bar:** A new user can run `memd`, `memd help`, `memd setup`, `memd settings`, `memd status`, and `memd doctor` and understand what to do in under 30 seconds, without seeing 100 internal commands or raw JSON/markdown unless requested.

---

## Non-Negotiables

1. **No overrating.** Current baseline is 4/10 beta until proof says otherwise.
2. **No command dump in root help.** Root help shows only curated user commands.
3. **Every visible command has a description.** No blank descriptions in curated help.
4. **Advanced commands stay reachable.** Full catalog moves to `memd commands` / `memd help <command>` / `memd commands --all` style surface.
5. **Hermes-like but memd-owned.** Big brand, clear sections, keyboard menus, concise dashboard panels, no Hermes copy/paste identity.
6. **Script modes untouched.** `--summary` one-line and `--json` parseable for settings/status/doctor/setup where supported.
7. **Live installed binary proof required.** Source/cargo-run is not enough.
8. **No secret leaks.** Redact token/password/key-looking values.

---

## Workstream A — Help and command discovery

**Objective:** Make `memd`, `memd --help`, and command discovery beginner-friendly.

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Modify: `crates/memd-client/src/cli/terminal_ux.rs`
- Likely modify: `crates/memd-client/src/cli/command_catalog.rs`
- Tests: add/update terminal UX/help tests under existing `terminal_ux_tests` or a new help test module.

**Tasks:**
1. Root `memd` and `memd --help` render curated help, not Clap full enum dump.
2. Curated help sections:
   - Start here: setup, status, doctor, settings
   - Memory: lookup, teach, remember, wake
   - Explore: memory, skills, commands
   - Advanced: `memd commands`, `memd help <command>`, `--summary`, `--json`
3. Add short descriptions for every curated command.
4. Add `memd commands` UX mode that explains: “Full catalog for power users.”
5. Keep `memd help <command>` exact Clap help for detailed flags.

**Verification:**
```bash
bash scripts/memd-cargo-guard.sh -- test -p memd-client terminal_ux -- --nocapture
bash scripts/memd-cargo-guard.sh -- run -p memd-client -- --help | tee /tmp/memd-help.txt
grep -q "Start here" /tmp/memd-help.txt
grep -q "memd commands" /tmp/memd-help.txt
! grep -q "healthz" /tmp/memd-help.txt
bash scripts/memd-cargo-guard.sh -- run -p memd-client -- help status | grep -q "Usage: memd status"
```

---

## Workstream B — Setup wizard parity

**Objective:** Make `memd setup` feel like Hermes setup: welcome state, mode menu, section flow, proof step.

**Files:**
- Modify: `crates/memd-client/src/bundle/admin_runtime.rs`
- Modify: `crates/memd-client/src/cli/terminal_ux.rs`
- Tests: existing `setup_interactive_tests`, `setup_section`, `setup_non_interactive`

**Tasks:**
1. Bare TTY `memd setup` starts with:
   - brand panel
   - “Welcome Back” when configured, or “Welcome to memd” when fresh
   - readiness checklist
   - menu: Quick Setup, Full Setup, Provider, Harness, Memory Bundle, Voice, Hive, Proof, Exit
2. Section renderers include descriptions and current state, not just choices.
3. Replace thin selector copy with Hermes-style: “Select by number, Enter to confirm.”
4. `setup proof` shows exact next commands and pass/fail checklist.
5. Non-interactive stays safe: no hang, `--summary` one-line.

**Verification:**
```bash
bash scripts/memd-cargo-guard.sh -- test -p memd-client setup_interactive_tests -- --nocapture
bash scripts/memd-cargo-guard.sh -- test -p memd-client setup_section -- --nocapture
bash scripts/memd-cargo-guard.sh -- test -p memd-client setup_non_interactive -- --nocapture
bash scripts/memd-cargo-guard.sh -- run -p memd-client -- setup proof | tee /tmp/memd-setup-proof.txt
grep -q "Quick Setup" /tmp/memd-setup-proof.txt || true
bash scripts/memd-cargo-guard.sh -- run -p memd-client -- setup --non-interactive --summary | grep -q "setup "
```

---

## Workstream C — Settings/config dashboard

**Objective:** Make `memd settings` a useful dashboard, not a raw config dump.

**Files:**
- Modify: `crates/memd-client/src/bundle/config_runtime.rs`
- Modify: `crates/memd-client/src/cli/terminal_ux.rs`
- Tests: `settings_summary`, add dashboard assertions.

**Tasks:**
1. Keep default `settings` human dashboard.
2. Group sections by user mental model:
   - Identity
   - Connection
   - Memory Bundle
   - Agent Defaults
   - Hive/Authority
   - Edit commands
3. Add “Edit commands” section with examples:
   - `memd settings set voice_mode=caveman-lite`
   - `memd settings get base_url`
   - `memd settings list --summary`
4. Long values truncate cleanly with full value available in `--json`.
5. `config` may remain compatibility alias but title should match invocation if feasible.

**Verification:**
```bash
bash scripts/memd-cargo-guard.sh -- test -p memd-client settings_summary -- --nocapture
bash scripts/memd-cargo-guard.sh -- run -p memd-client -- settings | tee /tmp/memd-settings.txt
grep -q "Edit commands" /tmp/memd-settings.txt
grep -q "Connection" /tmp/memd-settings.txt
bash scripts/memd-cargo-guard.sh -- run -p memd-client -- settings --summary | grep -q "settings "
bash scripts/memd-cargo-guard.sh -- run -p memd-client -- settings --json | python3 -m json.tool >/dev/null
```

---

## Workstream D — Status and doctor production dashboards

**Objective:** Make `status` and `doctor` actionable executive dashboards.

**Files:**
- Modify: `crates/memd-client/src/render/mod.rs`
- Modify: `crates/memd-client/src/bundle/config_runtime.rs`
- Tests: `status_render_tests`, add doctor render test.

**Tasks:**
1. `status` sections:
   - Environment
   - Server
   - Memory
   - Runtime wiring
   - Next action
2. `doctor` sections:
   - Overall verdict: PASS / WARN / FAIL
   - Problems found
   - Repair commands
   - Proof commands
3. Doctor must not just say ready; it must explain what was checked.
4. Add warning styles for degraded/missing values.
5. Keep `--summary` and `--json` untouched.

**Verification:**
```bash
bash scripts/memd-cargo-guard.sh -- test -p memd-client status_render_tests -- --nocapture
bash scripts/memd-cargo-guard.sh -- run -p memd-client -- status | tee /tmp/memd-status.txt
grep -q "Next action" /tmp/memd-status.txt
grep -q "Server" /tmp/memd-status.txt
bash scripts/memd-cargo-guard.sh -- run -p memd-client -- doctor | tee /tmp/memd-doctor.txt
grep -q "Overall" /tmp/memd-doctor.txt
grep -q "Repair" /tmp/memd-doctor.txt
bash scripts/memd-cargo-guard.sh -- run -p memd-client -- status --json | python3 -m json.tool >/dev/null
bash scripts/memd-cargo-guard.sh -- run -p memd-client -- doctor --json | python3 -m json.tool >/dev/null
```

---

## Workstream E — Golden UX lock and live proof

**Objective:** Stop subjective drift. Capture stable UX expectations.

**Files:**
- Add/update tests near terminal UX/render modules.
- Optional: `scripts/verify/memd-ux-proof.sh`

**Tasks:**
1. Add golden-ish tests for:
   - root help excludes internal commands
   - root help includes curated descriptions
   - setup selector uses Hermes-like radio/menu copy
   - settings includes edit examples
   - doctor includes verdict/repair/proof
2. Add proof script that runs source + installed binary checks.
3. Install live binary:
   ```bash
   cargo install --path crates/memd-client --bin memd --root /home/josue/.local --force
   ```
4. Verify live path and output:
   ```bash
   which memd
   memd --help
   memd setup proof
   memd settings
   memd status
   memd doctor
   memd settings --summary
   memd status --summary
   memd doctor --summary
   memd settings --json | python3 -m json.tool >/dev/null
   memd status --json | python3 -m json.tool >/dev/null
   memd doctor --json | python3 -m json.tool >/dev/null
   ```

---

## Final acceptance gate

Run all of this before claiming 10-star:

```bash
cargo fmt --check
git diff --check
bash scripts/memd-cargo-guard.sh -- check -p memd-client
bash scripts/memd-cargo-guard.sh -- test -p memd-client terminal_ux -- --nocapture
bash scripts/memd-cargo-guard.sh -- test -p memd-client setup_interactive_tests -- --nocapture
bash scripts/memd-cargo-guard.sh -- test -p memd-client setup_section -- --nocapture
bash scripts/memd-cargo-guard.sh -- test -p memd-client setup_non_interactive -- --nocapture
bash scripts/memd-cargo-guard.sh -- test -p memd-client settings_summary -- --nocapture
bash scripts/memd-cargo-guard.sh -- test -p memd-client status_render_tests -- --nocapture
cargo install --path crates/memd-client --bin memd --root /home/josue/.local --force
which memd
memd --help
memd setup proof
memd settings
memd status
memd doctor
memd settings --summary
memd status --summary
memd doctor --summary
memd settings --json | python3 -m json.tool >/dev/null
memd status --json | python3 -m json.tool >/dev/null
memd doctor --json | python3 -m json.tool >/dev/null
```

Only after this gate:
- commit
- push `main`
- report actual score with evidence

