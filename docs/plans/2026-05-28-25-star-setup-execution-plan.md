# 25-Star Setup Execution Plan, Detailed Build Spec

Secondary/reference doc. Start from [[ROADMAP]] for project truth.

Generated: 2026-05-28
Branch: `work/10-star-setup-experience`
Current base commit: `cc687b8 feat(setup): add guided 10-star setup path`
Owner goal: make memd setup a 25-star Apple-level hands-on setup experience.

## Non-Negotiable Definition

This score is not about architecture depth. It is hands-on setup/product experience.

A true 25-star setup means a real person can:

1. find the right start point
2. install memd
3. pick providers and harnesses in a polished interactive setup center
4. prove first memory works
5. understand where data lives
6. recover from common failures with doctor/docs, no live maintainer
7. update safely
8. uninstall safely without losing memory accidentally
9. trust the product enough to use it for real agent memory

True 25-star requires **external evidence**. Internal proof can only say "25-star implementation complete, external validation pending."

## Current Implementation Inventory

Already on branch:

| Area | File | Status | Gap |
| --- | --- | --- | --- |
| README quickstart | `README.md` | improved | needs guided/demo commands once built |
| first routing | `START-HERE.md` | improved | okay |
| setup docs | `docs/setup/*.md` | added | needs failure registry, distribution, OS matrix |
| scorecard | `docs/verification/setup-experience-scorecard.md` | added | needs axis scoring table and evidence refs |
| smoke proof | `scripts/verify/setup-experience-smoke.sh` | added | currently mutates `.memd`, needs temp isolation |
| interactive setup | `crates/memd-client/src/bundle/admin_runtime.rs` | provider + single harness picker | needs multi-select, config persistence, non-TTY fallback |
| CLI args | `crates/memd-client/src/cli/args_memory.rs` | `--interactive` added | needs `--guided`, `--verbose`, maybe harness args |
| dependency | `crates/memd-client/Cargo.toml` | `console = "0.16"` | okay |
| tests | `setup_interactive_tests` | render-only tests | needs state tests + behavior tests |
| doc lint | `scripts/doc-lint.sh` | failing | pre-existing missing banner in `docs/policy/claude-hooks.md` |

## Final Deliverables Checklist

A 25-star setup implementation is not done until these exist:

- [ ] `memd setup --interactive` supports provider selection and multi-harness toggle.
- [ ] `memd setup --guided --summary` prints step-by-step setup health with fixes.
- [ ] `memd setup --guided --json` emits redacted machine-readable report.
- [ ] `memd setup-demo --summary` proves first memory works in a temp root.
- [ ] `memd doctor --summary` uses beginner issue codes and exact fixes for setup failures.
- [ ] setup smoke script uses temp output and leaves git tree clean by default.
- [ ] clean-room setup proof script exists and passes on Linux.
- [ ] update script or command exists with rollback-safe behavior.
- [ ] uninstall script or command exists with dry-run and memory-preserving default.
- [ ] setup failure registry doc maps every common failure to symptom/cause/fix/verify.
- [ ] data/privacy doc answers where data lives and what leaves machine.
- [ ] distribution plan names public install channels and order.
- [ ] OS matrix doc names supported OS proof rows.
- [ ] human trial template exists.
- [ ] 25-star evidence packet exists.
- [ ] at least 5 external setup trials pass before true 25 claim.

---

# Phase 0: Make Current Branch Green

## Task 0.1, Fix doc-lint blocker

### Problem

`./scripts/doc-lint.sh` fails on a pre-existing docs policy file:

```text
doc-lint: docs/policy/claude-hooks.md missing secondary-doc banner
```

This blocks claiming green docs checks for setup work.

### Files

Modify:

- `docs/policy/claude-hooks.md`

### Exact Change

Insert after title:

```markdown
Secondary/reference doc. Start from [[ROADMAP]] for project truth.
```

### Acceptance Criteria

- `scripts/doc-lint.sh` no longer fails on `docs/policy/claude-hooks.md`.
- No unrelated docs edited.

### Verification

```bash
scripts/doc-lint.sh
git diff --check
git status --short
```

### Commit

```bash
git add docs/policy/claude-hooks.md
git commit -m "docs(policy): mark claude hooks as reference doc"
```

## Task 0.2, Isolate setup smoke proof

### Problem

Current `scripts/verify/setup-experience-smoke.sh` uses `.memd` in the worktree and writes setup-run artifacts. That dirties repo state and makes proof noisy.

### Files

Modify:

- `scripts/verify/setup-experience-smoke.sh`

### Exact Behavior

Default run:

```bash
bash scripts/verify/setup-experience-smoke.sh
```

Must:

- create temp dir via `mktemp -d`
- set `MEMD_OUTPUT="$TMP/.memd"` or pass `--output "$TMP/.memd"` to commands that support it
- run setup with `--force` only against temp output, never repo `.memd`
- write report to temp dir by default
- print report path
- delete temp dir at exit unless `MEMD_SETUP_SMOKE_KEEP=1`
- leave `git status --short` unchanged except intended source edits

Optional keep:

```bash
MEMD_SETUP_SMOKE_KEEP=1 bash scripts/verify/setup-experience-smoke.sh
```

Must preserve report and print location.

### Report Fields

Markdown report must include:

- UTC timestamp
- host
- OS
- memd path
- setup command and result
- doctor summary
- status summary
- resume first 40 lines
- final result `setup-experience-smoke=pass`

### Acceptance Criteria

- default run exits 0
- default run does not create `docs/verification/setup-runs/`
- default run does not modify `.memd`
- `MEMD_SETUP_SMOKE_KEEP=1` preserves report outside git by default or under ignored path

### Verification

```bash
before=$(git status --short)
bash scripts/verify/setup-experience-smoke.sh
after=$(git status --short)
test "$before" = "$after"
git diff --check
```

### Commit

```bash
git add scripts/verify/setup-experience-smoke.sh
git commit -m "test(setup): isolate setup smoke proof"
```

---

# Phase 1: Interactive Setup Center

## Task 1.1, Extract setup interactive state model

### Problem

Current interactive code mixes rendering, terminal input, and selection side effects in `admin_runtime.rs`. That makes multi-select hard and TTY tests brittle.

### Files

Create:

- `crates/memd-client/src/setup_interactive.rs`

Modify:

- `crates/memd-client/src/lib.rs` or `crates/memd-client/src/main.rs` module wiring if needed
- `crates/memd-client/src/bundle/admin_runtime.rs`

### API

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupProviderChoice {
    LocalOnly,
    SharedServer,
    CustomBaseUrl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SetupHarnessChoice {
    Codex,
    ClaudeCode,
    Hermes,
    OpenClaw,
    OpenCode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupInteractiveState {
    pub provider: SetupProviderChoice,
    pub harnesses: BTreeSet<SetupHarnessChoice>,
    pub cursor: usize,
    pub screen: SetupInteractiveScreen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupInteractiveScreen {
    Provider,
    Harnesses,
    Confirm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupInteractiveAction {
    Up,
    Down,
    ToggleOrSelect,
    Back,
    Quit,
}
```

### Functions

```rust
pub fn reduce_setup_interactive_state(
    state: &SetupInteractiveState,
    action: SetupInteractiveAction,
) -> SetupInteractiveState;

pub fn render_setup_interactive_state(state: &SetupInteractiveState) -> String;

pub fn selected_harness_slugs(state: &SetupInteractiveState) -> Vec<&'static str>;
```

### UX Rules

Provider screen:

```text
                  memd setup

        Pick memory provider

          › Local only, no network needed
            Shared memd server
            Custom MEMD_BASE_URL

        ↑/↓ move   Enter select   q quit
```

Harness screen:

```text
                  memd setup

        Pick agent harnesses

          ✓ Codex
          □ Claude Code
        › □ Hermes
          □ OpenClaw
          □ OpenCode

        ↑/↓ move   Space toggle   Enter continue   q quit
```

Confirm screen:

```text
                  memd setup

        Ready to configure

        Provider: Local only
        Harnesses: Codex, Hermes

        Enter start   ← back   q quit
```

### Tests

Add tests in module:

- provider down moves cursor from LocalOnly to SharedServer
- provider Enter moves to Harnesses screen
- harness Space toggles selected harness
- harness Enter moves to Confirm only if at least one harness selected
- no harness selected defaults to detected agent or Codex, with message
- render includes arrows, Enter, Space, centered title
- selected harness slugs include `codex`, `hermes`, `openclaw`, `opencode`, `claude-code`

### Verification

```bash
cargo test -p memd-client setup_interactive -- --nocapture
```

### Commit

```bash
git add crates/memd-client/src/setup_interactive.rs crates/memd-client/src/bundle/admin_runtime.rs crates/memd-client/src/lib.rs
git commit -m "feat(setup): model interactive setup center"
```

## Task 1.2, Wire interactive state to terminal input

### Files

Modify:

- `crates/memd-client/src/bundle/admin_runtime.rs`

### Behavior

`memd setup --interactive` must:

1. detect if stdout/stdin are TTY
2. if not TTY, print:

```text
interactive setup needs a terminal
Run: memd setup --guided --summary
```

3. if TTY, use arrow keys, Space, Enter
4. return a configured `SetupArgs` equivalent
5. set selected provider
6. set selected harnesses in the bundle profile or setup report path supported by current code

### Non-TTY Test

If testable without TTY, create pure function:

```rust
pub fn interactive_requires_tty(is_tty: bool) -> Option<&'static str>
```

Test expected fallback copy.

### Acceptance Criteria

- existing `memd setup --summary` unchanged
- `memd setup --interactive` does not panic in non-TTY
- TTY path manually verified once

### Verification

```bash
cargo test -p memd-client setup_interactive -- --nocapture
cargo run -p memd-client --bin memd -- setup --summary --force --output /tmp/memd-setup-check
```

Manual TTY check:

```bash
cargo run -p memd-client --bin memd -- setup --interactive --force --output /tmp/memd-setup-interactive
```

Record result in:

- `docs/verification/setup-runs/manual-tty-check.md`

### Commit

```bash
git add crates/memd-client/src/bundle/admin_runtime.rs docs/verification/setup-runs/manual-tty-check.md
git commit -m "feat(setup): wire interactive setup center"
```

---

# Phase 2: Guided Setup Journey

## Task 2.1, Add setup journey module

### Files

Create:

- `crates/memd-client/src/setup_journey.rs`

Modify:

- module exports

### Data Model

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SetupStepId {
    Preflight,
    BinaryAvailable,
    PathVisible,
    BundleInitialized,
    ProviderConfigured,
    HarnessConfigured,
    DoctorPassed,
    FirstMemoryProof,
    DataPrivacyAcknowledged,
    FinalHealth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SetupStepStatus {
    Passed,
    Fixed,
    NeedsUserAction,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SetupStepReport {
    pub id: SetupStepId,
    pub status: SetupStepStatus,
    pub issue_code: Option<String>,
    pub message: String,
    pub why_it_matters: Option<String>,
    pub fix_command: Option<String>,
    pub verify_command: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SetupJourneyReport {
    pub os: String,
    pub shell: Option<String>,
    pub memd_path: Option<String>,
    pub provider: Option<String>,
    pub harnesses: Vec<String>,
    pub elapsed_ms: u128,
    pub score: u8,
    pub steps: Vec<SetupStepReport>,
    pub redacted: bool,
}
```

### Issue Codes

Use stable strings:

- `SETUP_CARGO_MISSING`
- `SETUP_MEMD_NOT_ON_PATH`
- `SETUP_BUNDLE_MISSING`
- `SETUP_BUNDLE_WRITE_DENIED`
- `SETUP_DOCTOR_RED`
- `SETUP_SERVER_UNREACHABLE`
- `SETUP_RAG_UNAVAILABLE`
- `SETUP_NO_HARNESS_SELECTED`
- `SETUP_DEMO_WRITE_FAILED`
- `SETUP_DEMO_LOOKUP_FAILED`
- `SETUP_REPORT_REDACTED_SECRET`

### Score Formula

25-point internal implementation score:

- preflight binary/path: 3
- bundle initialized: 3
- provider/harness configured: 4
- doctor green: 5
- setup-demo passed: 5
- privacy/docs acknowledged or linked: 2
- no user action required: 3

Report both:

- `score_25_internal`
- `external_validation_pending: true`

### Tests

- all passed returns 25 internal
- doctor red caps score at 14
- setup-demo failed caps score at 16
- missing path adds issue code and fix command
- redaction flag set when fake secret appears

### Verification

```bash
cargo test -p memd-client setup_journey -- --nocapture
```

### Commit

```bash
git add crates/memd-client/src/setup_journey.rs
git commit -m "feat(setup): model guided setup journey"
```

## Task 2.2, Add `memd setup --guided`

### Files

Modify:

- `crates/memd-client/src/cli/args_memory.rs`
- `crates/memd-client/src/bundle/admin_runtime.rs`
- `crates/memd-client/src/cli/mod.rs` if dispatch needs change
- `crates/memd-client/src/setup_journey.rs`

### CLI Args

In `SetupArgs` add:

```rust
#[arg(long, default_value_t = false)]
pub(crate) guided: bool,

#[arg(long, default_value_t = false)]
pub(crate) verbose: bool,
```

### Summary Output Contract

```text
memd setup: checking this machine
✓ binary available: /home/josue/.local/bin/memd
✓ bundle initialized: .memd
✓ provider configured: local
✓ harnesses configured: codex, hermes
✓ doctor passed
✓ first memory proof passed

Setup score: 25/25 internal
External validation: pending
Next: memd resume --output .memd --intent current_task
```

For non-green:

```text
! path visible: memd is installed but not on PATH
  why: your shell cannot run memd after install
  fix: export PATH="$HOME/.local/bin:$PATH"
  verify: command -v memd
```

### JSON Output Contract

```json
{
  "score_25_internal": 25,
  "external_validation_pending": true,
  "steps": [
    {
      "id": "PathVisible",
      "status": "Passed",
      "issue_code": null,
      "message": "memd is on PATH",
      "fix_command": null,
      "verify_command": "command -v memd"
    }
  ]
}
```

### Redaction Rules

Before JSON print or report write, redact values matching:

- `BW_SESSION=`
- `OPENAI_API_KEY=`
- `ANTHROPIC_API_KEY=`
- `TOKEN=`
- `PASSWORD=`
- PEM private key blocks
- URLs containing `user:pass@`

Replacement:

```text
[REDACTED]
```

### Acceptance Criteria

- `--guided --summary` exits 0 when setup is usable
- `--guided --json` parses with `jq`
- non-green steps have `fix_command` and `verify_command`
- no secret-looking output in reports

### Verification

```bash
cargo test -p memd-client setup_guided -- --nocapture
cargo run -p memd-client --bin memd -- setup --guided --summary --output /tmp/memd-guided
cargo run -p memd-client --bin memd -- setup --guided --json --output /tmp/memd-guided | jq .
```

### Commit

```bash
git add crates/memd-client/src/cli/args_memory.rs crates/memd-client/src/bundle/admin_runtime.rs crates/memd-client/src/setup_journey.rs
git commit -m "feat(setup): add guided setup summary and json"
```

---

# Phase 3: First Memory Proof Demo

## Task 3.1, Add setup-demo command args

### Files

Modify:

- `crates/memd-client/src/cli/args.rs`
- `crates/memd-client/src/cli/args_memory.rs`
- `crates/memd-client/src/cli/mod.rs`

Create:

- `crates/memd-client/src/setup_demo.rs`

### CLI Contract

```bash
memd setup-demo --summary
memd setup-demo --json
memd setup-demo --output /tmp/memd-demo --keep
```

### Args

```rust
pub(crate) struct SetupDemoArgs {
    #[arg(long)]
    pub(crate) output: Option<PathBuf>,

    #[arg(long, default_value_t = false)]
    pub(crate) keep: bool,

    #[arg(long, default_value_t = false)]
    pub(crate) summary: bool,

    #[arg(long, default_value_t = false)]
    pub(crate) json: bool,
}
```

### Behavior

Default:

- create temp output root
- initialize bundle if needed
- write demo memory with tag `setup-demo`
- run enough compile/status path for lookup to see it
- lookup query: `setup demo favorite color`
- print recalled text
- delete temp output unless `--keep`

Summary:

```text
memd demo: using temp bundle /tmp/memd-demo-abc/.memd
memd demo: saved test memory
memd demo: recalled test memory
Result: setup demo favorite color is blue
You are ready.
```

Failure:

```text
memd demo: failed at lookup
Why: setup demo memory was written but lookup returned no records
Fix: run memd doctor --summary
Verify: memd setup-demo --summary
```

### Tests

- temp root is cleaned by default
- `--keep` preserves output
- summary includes `You are ready`
- JSON includes stages and no secrets
- lookup failure maps to `SETUP_DEMO_LOOKUP_FAILED`

### Verification

```bash
cargo test -p memd-client setup_demo -- --nocapture
cargo run -p memd-client --bin memd -- setup-demo --summary
cargo run -p memd-client --bin memd -- setup-demo --json | jq .
```

### Commit

```bash
git add crates/memd-client/src/setup_demo.rs crates/memd-client/src/cli/args.rs crates/memd-client/src/cli/args_memory.rs crates/memd-client/src/cli/mod.rs
git commit -m "feat(setup): add first memory proof demo"
```

## Task 3.2, Wire demo into README and installer

### Files

Modify:

- `README.md`
- `scripts/install-memd.sh`
- `docs/setup/first-run.md`
- `docs/setup/install.md`

### README Quickstart Final Shape

```markdown
### 4. Prove first memory

```bash
memd setup-demo --summary
```
```

Installer final lines:

```text
memd install: next: run 'memd setup --interactive' to pick providers and harnesses
memd install: proof: run 'memd setup-demo --summary'
memd install: health: run 'memd doctor --summary'
```

### Verification

```bash
scripts/doc-lint.sh
git diff --check
grep -R "setup-demo" -n README.md docs/setup scripts/install-memd.sh
```

### Commit

```bash
git add README.md scripts/install-memd.sh docs/setup/first-run.md docs/setup/install.md
git commit -m "docs(setup): route quickstart to first memory proof"
```

---

# Phase 4: Doctor as Product Support

## Task 4.1, Setup failure registry doc

### Files

Create:

- `docs/setup/failure-registry.md`

Link from:

- `docs/setup/troubleshooting.md`
- `docs/setup/README.md`

### Required Rows

| Issue code | Symptom | Cause | Fix | Verify | Repair class |
| --- | --- | --- | --- | --- | --- |
| `SETUP_CARGO_MISSING` | installer cannot build | Rust missing | install Rust | `cargo --version` | manual_only |
| `SETUP_MEMD_NOT_ON_PATH` | `memd: command not found` | bin dir missing from PATH | export PATH line | `command -v memd` | ask_first |
| `SETUP_STALE_BINARY` | old commands missing | installed binary older than checkout | rerun installer | `memd capabilities sync --help` | auto_safe |
| `SETUP_BUNDLE_MISSING` | status says setup=false | `.memd` absent | `memd setup --summary` | `test -d .memd` | auto_safe |
| `SETUP_BUNDLE_WRITE_DENIED` | permission denied | wrong ownership/mode | owner-specific command | write test | manual_only |
| `SETUP_SERVER_UNREACHABLE` | server=down | base URL unavailable | start server or set URL | health check | ask_first |
| `SETUP_RAG_UNAVAILABLE` | rag=off | RAG disabled/unreachable | optional setup | status | manual_only |
| `SETUP_MAC_BRIDGE_MISSING` | bridge absent | launch agent missing | bridge installer | launchctl list | ask_first |
| `SETUP_REPORT_SECRET_DETECTED` | report redacted | secret-like text in context | no action | inspect redaction | auto_safe |

### Verification

```bash
scripts/doc-lint.sh
grep -R "SETUP_CARGO_MISSING" -n docs/setup
```

### Commit

```bash
git add docs/setup/failure-registry.md docs/setup/troubleshooting.md docs/setup/README.md
git commit -m "docs(setup): add setup failure registry"
```

## Task 4.2, Doctor issue code model

### Files

Modify or create:

- `crates/memd-client/src/bundle/status_runtime.rs`
- `crates/memd-client/src/bundle/admin_runtime.rs`
- `crates/memd-client/src/setup_journey.rs`
- maybe `crates/memd-client/src/setup_failure.rs`

### Model

```rust
pub enum SetupIssueCode { ... }
pub enum RepairClass { AutoSafe, AskFirst, ManualOnly }
pub struct SetupIssue {
    code: SetupIssueCode,
    problem: String,
    why_it_matters: String,
    fix_command: Option<String>,
    verify_command: Option<String>,
    repair_class: RepairClass,
}
```

### Summary Output

For each issue:

```text
Problem: memd is installed but your shell cannot find it
Why it matters: tomorrow your agent cannot run memory commands
Fix: export PATH="$HOME/.local/bin:$PATH"
Verify: command -v memd
```

### JSON Output

`memd doctor --json` includes:

```json
{
  "issues": [
    {
      "code": "SETUP_MEMD_NOT_ON_PATH",
      "repair_class": "ask_first",
      "fix_command": "export PATH=...",
      "verify_command": "command -v memd"
    }
  ]
}
```

### Tests

- missing bundle maps to `SETUP_BUNDLE_MISSING`
- permission denied maps to `SETUP_BUNDLE_WRITE_DENIED`
- server unavailable maps to `SETUP_SERVER_UNREACHABLE`
- summary output contains Problem/Why/Fix/Verify
- `--repair` runs only auto_safe fixes
- manual_only never runs commands

### Verification

```bash
cargo test -p memd-client doctor_setup_issues -- --nocapture
cargo run -p memd-client --bin memd -- doctor --summary --output /tmp/memd-doctor-empty
```

### Commit

```bash
git add crates/memd-client/src/bundle/status_runtime.rs crates/memd-client/src/bundle/admin_runtime.rs crates/memd-client/src/setup_failure.rs
git commit -m "feat(doctor): add setup issue codes and repair classes"
```

---

# Phase 5: Setup Proof Harnesses

## Task 5.1, Clean-room setup proof

### Files

Create:

- `scripts/verify/setup-clean-room.sh`

### Behavior

```bash
bash scripts/verify/setup-clean-room.sh
```

Steps:

1. create temp HOME
2. copy repo to temp checkout using `git archive` or `rsync --exclude .git --exclude target`
3. run `scripts/install-memd.sh`
4. run `memd setup --guided --json`
5. run `memd setup-demo --summary`
6. run `memd doctor --summary`
7. redact report
8. cleanup by default

Keep mode:

```bash
MEMD_SETUP_CLEAN_KEEP=1 bash scripts/verify/setup-clean-room.sh
```

### Acceptance Criteria

- default exits 0 on Linux desktop
- leaves current repo clean
- reports elapsed seconds
- if cargo missing in temp HOME, still uses system cargo from PATH, not fake failure

### Verification

```bash
bash scripts/verify/setup-clean-room.sh
git status --short
git diff --check
```

### Commit

```bash
git add scripts/verify/setup-clean-room.sh
git commit -m "test(setup): add clean-room setup proof"
```

## Task 5.2, Setup report redaction tests

### Files

Modify:

- `crates/memd-client/src/setup_journey.rs`
- `scripts/verify/setup-experience-smoke.sh`
- `scripts/verify/setup-clean-room.sh`

### Fake Secrets for Tests

Use only fake values:

```text
OPENAI_API_KEY=sk-test-should-redact
BW_SESSION=fake-session-should-redact
https://user:pass@example.test/path
-----BEGIN PRIVATE KEY----- fake -----END PRIVATE KEY-----
```

### Tests

- report redacts all fake secrets
- raw secret strings absent from JSON and markdown
- redaction count > 0 when fake secrets supplied

### Verification

```bash
cargo test -p memd-client setup_redaction -- --nocapture
```

### Commit

```bash
git add crates/memd-client/src/setup_journey.rs scripts/verify/setup-experience-smoke.sh scripts/verify/setup-clean-room.sh
git commit -m "test(setup): redact secrets from setup reports"
```

---

# Phase 6: Lifecycle, Update, Uninstall

## Task 6.1, Safe update script

### Files

Create:

- `scripts/update-memd.sh`

### Behavior

```bash
scripts/update-memd.sh
scripts/update-memd.sh --dry-run
```

Steps:

1. detect current binary path
2. detect repo root
3. build new binary into temp prefix
4. run `memd --help` on temp binary
5. run `memd doctor --summary` using temp binary if possible
6. backup old binary to `~/.local/share/memd/backups/memd-<timestamp>`
7. atomically install new binary
8. print rollback command

Rollback command:

```bash
cp ~/.local/share/memd/backups/memd-<timestamp> ~/.local/bin/memd
```

### Acceptance Criteria

- `--dry-run` prints all paths and touches nothing
- backup exists before replacement
- failed build leaves old binary in place

### Verification

```bash
scripts/update-memd.sh --dry-run
bash -n scripts/update-memd.sh
```

### Commit

```bash
git add scripts/update-memd.sh docs/setup/update.md
git commit -m "feat(setup): add rollback-safe update script"
```

## Task 6.2, Safe uninstall script

### Files

Create:

- `scripts/uninstall-memd.sh`

Modify:

- `docs/setup/uninstall.md`

### Behavior

```bash
scripts/uninstall-memd.sh --dry-run
scripts/uninstall-memd.sh --yes
scripts/uninstall-memd.sh --yes --delete-memory
```

Rules:

- dry-run default
- remove binary only with `--yes`
- do not delete `.memd` by default
- delete `.memd` only with both `--yes --delete-memory`
- print exact memory path preserved/deleted

### Acceptance Criteria

- dry-run touches nothing
- no command deletes `.memd` unless `--delete-memory`
- shellcheck if available, otherwise `bash -n`

### Verification

```bash
scripts/uninstall-memd.sh --dry-run
bash -n scripts/uninstall-memd.sh
```

### Commit

```bash
git add scripts/uninstall-memd.sh docs/setup/uninstall.md
git commit -m "feat(setup): add memory-safe uninstall script"
```

---

# Phase 7: Distribution, OS Matrix, Trust Docs

## Task 7.1, Distribution plan

### Files

Create:

- `docs/setup/distribution-plan.md`

### Required Sections

- Current internal channel: source checkout installer
- Public channel 1: GitHub release binary tarballs
- Public channel 2: Homebrew tap
- Public channel 3: `cargo install`, only after crate packaging audit
- Not now: Nix, apt, Chocolatey, Windows installer
- Signing/checksum plan
- Rollback story
- Support matrix

### Acceptance Criteria

- every channel has owner, prerequisites, verification command, rollback path
- no channel marked complete without proof artifact

### Verification

```bash
scripts/doc-lint.sh
grep -n "GitHub release" docs/setup/distribution-plan.md
grep -n "Homebrew" docs/setup/distribution-plan.md
```

### Commit

```bash
git add docs/setup/distribution-plan.md docs/setup/README.md
git commit -m "docs(setup): define public distribution path"
```

## Task 7.2, OS validation matrix

### Files

Create:

- `docs/verification/setup-os-matrix.md`

### Rows

- Linux desktop, current CachyOS/Arch-like
- Ubuntu 24.04 fresh VM
- Debian 12 fresh VM
- macOS Apple Silicon
- macOS Intel, optional if hardware unavailable

### Columns

- install script
- `setup --interactive`
- `setup --guided --json`
- `setup-demo --summary`
- `doctor --repair --summary`
- update dry-run
- uninstall dry-run
- report artifact
- pass/fail/date

### Acceptance Criteria

- unknown rows marked `pending`, not pass
- each pass has artifact path

### Verification

```bash
scripts/doc-lint.sh
grep -n "Ubuntu 24.04" docs/verification/setup-os-matrix.md
```

### Commit

```bash
git add docs/verification/setup-os-matrix.md
git commit -m "docs(setup): add setup os validation matrix"
```

## Task 7.3, Data and privacy upgrade

### Files

Modify:

- `docs/setup/data-and-privacy.md`
- `README.md`
- `docs/setup/README.md`

### Must Answer in First 20 Lines

- Where is my data?
- Does setup send anything over network?
- What changes when I use shared server/RAG/sync?
- How do I delete local state?
- How do I avoid posting secrets in bug reports?

### Acceptance Criteria

- non-expert can answer "what leaves my machine?" from doc alone
- README links data/privacy before dogfood consent

### Verification

```bash
grep -n "What leaves" docs/setup/data-and-privacy.md
grep -n "Data and privacy" README.md
scripts/doc-lint.sh
```

### Commit

```bash
git add docs/setup/data-and-privacy.md README.md docs/setup/README.md
git commit -m "docs(setup): make data and privacy plain"
```

---

# Phase 8: Human Trial and 25-Star Evidence

## Task 8.1, Human trial template

### Files

Create:

- `docs/verification/setup-runs/HUMAN-TRIAL-TEMPLATE.md`

### Template Fields

```markdown
# Human Setup Trial

- trial id:
- date:
- user role:
- OS:
- machine type:
- starting knowledge:
- live help allowed: no for happy path
- starting URL/doc:
- elapsed time to install:
- elapsed time to first memory proof:
- commands run:
- errors hit:
- where user paused:
- words user used when confused:
- did docs/doctor resolve it?
- live help needed?
- confidence 1-5:
- trust quote:
- would use for real agent memory? yes/no/why:
- follow-up fixes required:
```

### Acceptance Criteria

- template is specific enough for repeatable trials
- no secrets requested
- captures user words, not assistant summary only

### Verification

```bash
test -f docs/verification/setup-runs/HUMAN-TRIAL-TEMPLATE.md
grep -n "live help needed" docs/verification/setup-runs/HUMAN-TRIAL-TEMPLATE.md
```

### Commit

```bash
git add docs/verification/setup-runs/HUMAN-TRIAL-TEMPLATE.md
git commit -m "docs(setup): add human setup trial template"
```

## Task 8.2, 25-star evidence packet

### Files

Create:

- `docs/verification/25-star-setup-evidence.md`

### Required Sections

- Claim status: `not-yet-25`, `implementation-complete-validation-pending`, or `true-25`
- Gate checklist
- Trial summary table
- OS matrix links
- setup report links
- confusion log
- fixes made from trials
- unresolved gaps
- final yes/no judgment

### Gate Checklist

True 25 only if:

- [ ] 5 external users completed happy path with zero live help
- [ ] median time to first memory <= 10 min
- [ ] at least 4/5 recovered from induced failure using docs/doctor
- [ ] at least 4/5 understood data/privacy doc
- [ ] at least 4/5 said they would trust memd for real agent memory
- [ ] Linux and macOS proof rows passed
- [ ] update/uninstall dry-run passed
- [ ] no known secret leak in reports

### Acceptance Criteria

- default status is not `true-25`
- explicitly prevents inflated claim
- links every evidence artifact

### Verification

```bash
grep -n "not-yet-25" docs/verification/25-star-setup-evidence.md
grep -n "5 external users" docs/verification/25-star-setup-evidence.md
scripts/doc-lint.sh
```

### Commit

```bash
git add docs/verification/25-star-setup-evidence.md
git commit -m "docs(setup): define 25-star evidence packet"
```

## Task 8.3, Confusion-to-fix loop protocol

### Files

Create:

- `docs/setup/confusion-to-fix.md`

### Protocol

For every human trial confusion:

1. quote exact user words
2. classify as docs, CLI copy, product bug, environment, or expectation gap
3. patch docs or product
4. rerun the relevant proof
5. update evidence packet
6. commit with `trial:` or `docs(setup):` prefix

### Acceptance Criteria

- no confusion can be closed as "user error" without evidence
- every closed confusion links a commit

### Verification

```bash
grep -n "user error" docs/setup/confusion-to-fix.md
scripts/doc-lint.sh
```

### Commit

```bash
git add docs/setup/confusion-to-fix.md
git commit -m "docs(setup): add confusion-to-fix trial protocol"
```

---

# Final 25-Star Verification Script

Create after implementation:

- `scripts/verify/25-star-setup-audit.sh`

It must run:

```bash
set -euo pipefail
scripts/doc-lint.sh
git diff --check
cargo test -p memd-client setup_interactive -- --nocapture
cargo test -p memd-client setup_journey -- --nocapture
cargo test -p memd-client setup_demo -- --nocapture
cargo test -p memd-client doctor_setup_issues -- --nocapture
bash scripts/verify/setup-experience-smoke.sh
bash scripts/verify/setup-clean-room.sh
bash -n scripts/update-memd.sh
bash -n scripts/uninstall-memd.sh
grep -n "not-yet-25\|implementation-complete-validation-pending\|true-25" docs/verification/25-star-setup-evidence.md
```

Acceptance:

- exits 0
- prints final audit report path
- refuses `true-25` status if fewer than 5 trial files exist

Commit:

```bash
git add scripts/verify/25-star-setup-audit.sh
git commit -m "test(setup): add 25-star setup audit gate"
```

---

# Atomic Commit Order

Use this exact order unless implementation proves a dependency is wrong:

1. `docs(policy): mark claude hooks as reference doc`
2. `test(setup): isolate setup smoke proof`
3. `feat(setup): model interactive setup center`
4. `feat(setup): wire interactive setup center`
5. `feat(setup): model guided setup journey`
6. `feat(setup): add guided setup summary and json`
7. `feat(setup): add first memory proof demo`
8. `docs(setup): route quickstart to first memory proof`
9. `docs(setup): add setup failure registry`
10. `feat(doctor): add setup issue codes and repair classes`
11. `test(setup): add clean-room setup proof`
12. `test(setup): redact secrets from setup reports`
13. `feat(setup): add rollback-safe update script`
14. `feat(setup): add memory-safe uninstall script`
15. `docs(setup): define public distribution path`
16. `docs(setup): add setup os validation matrix`
17. `docs(setup): make data and privacy plain`
18. `docs(setup): add human setup trial template`
19. `docs(setup): define 25-star evidence packet`
20. `docs(setup): add confusion-to-fix trial protocol`
21. `test(setup): add 25-star setup audit gate`
22. `docs(setup): publish final 25-star setup audit`

---

# Claim Rules

Allowed after commit 8:

```text
15-star implementation in progress, guided setup and first proof exist.
```

Allowed after commit 17:

```text
20-star-ready internally on verified OS rows only.
```

Allowed after commit 22 but before human trials:

```text
25-star implementation complete, external validation pending.
```

Allowed only after 5 successful human trial files:

```text
True 25-star setup experience validated.
```

Never say Apple-level until the 25-star evidence packet says `true-25` and the audit script agrees.
