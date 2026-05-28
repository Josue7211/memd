# 25-Star Hands-On Setup CEO Plan

> **For Hermes:** This is a plan, not implementation. Use `subagent-driven-development` only after Josue approves execution. Score this work by hands-on setup/product experience, not abstract roadmap closure.

**Goal:** Make memd feel like an Apple-level product for first-time setup: easy to understand, hard to break, self-repairing where possible, clearly documented, and reliable enough that a non-expert can install, start, verify, use, debug, update, and recover without live handholding.

**Architecture:** Treat setup as the product. Add a guided onboarding path that owns the full journey from clone/install through first successful memory recall. Keep the existing power-user CLI, but wrap it in a beginner-safe experience: preflight, guided installer, plain-English docs, health checks, fix buttons/commands, smoke tests, recovery, and evidence capture. The score is a user-journey score backed by real machines and real users.

**Tech Stack:** Rust CLI (`crates/memd-client`), existing `scripts/install-memd.sh`, `memd doctor`, `memd dogfood`, docs under `README.md`, `START-HERE.md`, `docs/core/setup.md`, `docs/DOGFOOD.md`, verification scripts under `scripts/verify/`, proof artifacts under `docs/verification/`.

---

## CEO Mode: 25-Star Interpretation

25-star does **not** mean more internal proof scripts. It means the product feels inevitable to a real user.

The bar:

- A normal person can understand what memd is in 60 seconds.
- A developer can install it from scratch in one command after clone.
- A non-expert can follow docs without knowing the architecture.
- Every setup failure says what happened, why it matters, and the exact fix.
- `memd doctor` becomes the single trusted truth surface.
- The first successful moment happens fast: "memd remembered something and used it correctly."
- Updates do not break existing bundles.
- Recovery is clear when something goes wrong.
- Evidence is real: fresh machine runs, fresh user runs, timed setup recordings, failure logs, support questions, and retention.

This is **product maturity work**, not feature inflation.

---

## Current Audit Snapshot

Observed on `josuesdesktop`, repo `/home/josue/Documents/projects/memd`, branch `main`, HEAD `aba9a17`.

### What already exists

- `scripts/install-memd.sh` builds and installs `memd`, initializes `.memd`, runs `memd doctor`, registers a device, and installs Mac Bridge on Darwin.
- README quickstart has the basic install and dogfood enroll commands.
- `docs/core/setup.md` has longer setup details.
- `docs/DOGFOOD.md` has dogfood install, status, and gates.
- `memd doctor --summary` exists and is already the right center of gravity.
- `memd dogfood enroll/status` exists.
- 25-star contract exists, but it is evidence-gate oriented, not user-experience oriented.

### Product gaps against the Apple-level bar

1. **First page is not beginner-first.** `START-HERE.md` is optimized for project recovery, not a first-time user.
2. **Quickstart assumes a checkout and Rust mental model.** Good for maintainers, not "anybody."
3. **No guided setup command.** User must know installer, enroll, doctor, server, status, and next command order.
4. **No setup scorecard.** Existing 25-star score is mostly roadmap/evidence language, not hands-on experience.
5. **Failure messages are not yet a product surface.** Installer has some helpful messages, but not a complete troubleshooting decision tree.
6. **No first-run wow moment.** Setup does not end with a tiny demo proving memory works.
7. **No timed fresh-machine proof loop.** Need repeatable measurement for install time, prompts, failures, and user confusion.
8. **Docs are deep but not layered.** Need "I just want it working," "I hit an error," and "I want internals" lanes.
9. **No public packaging path yet.** Source checkout is okay for internal dogfood, not Apple-level public setup.
10. **No beginner vocabulary contract.** Docs still use internal terms before the user has a mental model.

---

## North Star User Journey

```text
User lands on repo
  -> understands memd in 60 seconds
  -> chooses "Install locally"
  -> runs one command
  -> gets preflight results
  -> installer fixes safe issues automatically
  -> user sees plain-English progress
  -> memd starts or tells exactly what is missing
  -> first memory is captured
  -> first recall proves it worked
  -> doctor shows green
  -> docs say what to do tomorrow
```

The target feeling: "This thing has taste. It respects my time. When it breaks, it helps me."

---

## Scoring Model: Hands-On Setup Score

Use this score before calling any release 15/20/25-star from a product standpoint.

| Lane | Weight | 25-star bar | Current honest read |
| --- | ---: | --- | --- |
| Setup comprehension | 15 | User knows what to do in <60s | Weak. Docs are maintainer-oriented. |
| One-command install | 15 | One command handles preflight, build/install, PATH, bundle, doctor | Medium. Script exists, but checkout/Rust assumptions remain. |
| Guided first run | 15 | User reaches first successful memory proof | Weak. No first-run demo path. |
| Failure recovery | 15 | Every common failure has exact fix and repair command | Medium-low. Doctor exists, recovery map incomplete. |
| Documentation clarity | 15 | Layered beginner docs, plain English, screenshots/examples | Medium-low. Lots of docs, not beginner-shaped. |
| Reliability | 15 | Fresh machine pass rate >=95%, no handholding | Unknown. Needs real evidence. |
| Update/uninstall/reinstall | 5 | Safe update and clean uninstall path | Weak/unknown. |
| Trust/privacy | 5 | Clear data location, privacy, consent, logs | Medium. Dogfood consent exists, beginner trust docs missing. |

**Initial hands-on score: 3.5/10.**

This is not an insult to the engine. The engine has deep capability. The setup product is not yet Apple-level.

Target gates:

- **5-star:** Maintainer can install reliably from checkout, docs are enough for a developer.
- **10-star:** Fresh developer can install, run, verify, and recover without live help.
- **15-star:** Non-expert technical user can complete setup with guided docs and plain errors.
- **20-star:** Public package install works across macOS/Linux with telemetry-backed reliability.
- **25-star:** Setup feels boring in the best way. It just works, explains itself, self-repairs common drift, and earns external user trust.

---

## Implementation Alternatives

### Approach A: Docs-first polish

**Summary:** Rewrite README/START-HERE/setup docs and add troubleshooting tables. No major CLI changes.

**Effort:** S
**Risk:** Low
**Pros:** Fast, low diff, immediately helps.
**Cons:** Does not make product self-guided. Still relies on user reading correctly.
**Completeness:** 5/10

### Approach B: Guided setup wrapper

**Summary:** Add `memd onboarding` or `memd setup --guided` that runs preflight, install checks, bundle init, doctor, dogfood enroll prompt, first memory demo, and writes a setup report.

**Effort:** M
**Risk:** Medium
**Pros:** Turns setup into a product flow. Testable. Keeps existing commands.
**Cons:** Needs careful error taxonomy and OS handling.
**Completeness:** 8/10

### Approach C: Full Apple-level setup system

**Summary:** Build a complete setup product: guided CLI, docs rewrite, setup scorecard, fresh-machine proof harness, failure taxonomy, repair actions, update/uninstall flows, and release evidence gates.

**Effort:** L human, M with CC+subagents
**Risk:** Medium
**Pros:** Actually matches the user bar. Creates durable product maturity, not just docs.
**Cons:** Requires multiple workstreams and honest external/fresh-user validation.
**Completeness:** 10/10

**CEO recommendation:** Choose Approach C. AI makes the lake boilable. The difference between a docs patch and a setup product is the difference between "can be installed" and "people trust it."

---

## Scope Decision

Use **SCOPE EXPANSION**, but focused expansion.

Do not add random advanced features. Expand only along the setup experience path:

1. Understand.
2. Install.
3. Start.
4. Prove.
5. Recover.
6. Update.
7. Trust.
8. Measure.

Anything not improving those steps is out of scope.

---

## Workstreams

## Workstream A: Beginner-first information architecture

### Task A1: Replace `START-HERE.md` with user-lane routing

**Objective:** Make the first doc useful to both new users and maintainers.

**Files:**
- Modify: `START-HERE.md`
- Modify: `README.md`
- Create: `docs/setup/README.md`

**Content requirements:**

`START-HERE.md` must start with:

```markdown
# Start Here

Choose your lane:

1. I want to install memd for the first time -> README Quickstart
2. I installed it and something failed -> docs/setup/troubleshooting.md
3. I want to dogfood on a real machine -> docs/DOGFOOD.md
4. I am continuing repo development -> ROADMAP.md
```

**Verification:**

```bash
scripts/doc-lint.sh
scripts/lint-links.sh
```

**Commit:** `docs(setup): route first-time users before maintainer recovery`

### Task A2: Rewrite README quickstart around the first successful moment

**Objective:** README gets user to working memory proof, not just installed binary.

**Files:**
- Modify: `README.md`

**Required quickstart shape:**

```markdown
## Quickstart

### 1. Install
scripts/install-memd.sh

### 2. Check health
memd doctor --summary

### 3. Prove memory works
memd setup-demo --summary

### 4. Use it tomorrow
memd resume --output .memd --intent current_task
```

If `memd setup-demo` does not exist yet, mark it as planned in this plan and use current command sequence until implemented.

**Verification:** README has no unexplained internal terms before quickstart.

**Commit:** `docs(readme): make quickstart prove first memory success`

### Task A3: Create layered setup docs

**Objective:** Split docs by user intent.

**Files:**
- Create: `docs/setup/install.md`
- Create: `docs/setup/first-run.md`
- Create: `docs/setup/troubleshooting.md`
- Create: `docs/setup/update.md`
- Create: `docs/setup/uninstall.md`
- Modify: `docs/core/setup.md` to point beginner users to `docs/setup/`

**Rules:**

- Plain English first.
- Commands second.
- Internals last.
- Every error section includes: symptom, cause, fix, verify.

**Verification:**

```bash
scripts/doc-lint.sh
scripts/lint-links.sh
```

**Commit:** `docs(setup): add beginner setup guide layers`

---

## Workstream B: Guided setup product

### Task B1: Add setup journey model

**Objective:** Represent setup as explicit steps with status, message, fix, and verification.

**Files:**
- Create: `crates/memd-client/src/setup_journey.rs`
- Modify: `crates/memd-client/src/lib.rs` or module root as needed
- Add tests near existing CLI/runtime tests

**Model:**

```rust
pub enum SetupStepId {
    Preflight,
    InstallBinary,
    PathCheck,
    BundleInit,
    Doctor,
    DeviceRegister,
    DogfoodEnroll,
    FirstMemoryDemo,
    FinalHealth,
}

pub enum SetupStepStatus {
    Passed,
    Fixed,
    NeedsUserAction,
    Failed,
    Skipped,
}

pub struct SetupStepReport {
    pub id: SetupStepId,
    pub status: SetupStepStatus,
    pub message: String,
    pub fix_command: Option<String>,
    pub verify_command: Option<String>,
}
```

**Test cases:**

- all green
- missing cargo
- PATH missing
- doctor fail
- dogfood skipped without consent

**Verification:**

```bash
cargo test -p memd-client setup_journey -- --nocapture
```

**Commit:** `feat(setup): model guided setup journey`

### Task B2: Add `memd setup --guided --summary`

**Objective:** One command guides the user through setup health without replacing installer internals.

**Files:**
- Modify: CLI args files under `crates/memd-client/src/cli/`
- Modify runtime dispatch under `crates/memd-client/src/cli/mod.rs`
- Use `setup_journey.rs`

**Behavior:**

```bash
memd setup --guided --summary
```

Output shape:

```text
memd setup: checking this machine
✓ Rust/cargo available
✓ memd binary available
✓ PATH can find memd
✓ .memd bundle initialized
✓ doctor passed
! dogfood not enrolled
  next: memd dogfood enroll --user-id <name> --consent --summary
✓ first memory demo passed

Setup score: 8/10
Next: memd resume --output .memd --intent current_task
```

**Rules:**

- Never dump raw Rust errors at beginner level.
- Show raw detail only with `--verbose` or `--json`.
- Every non-green step has a next command.

**Verification:**

```bash
cargo test -p memd-client setup_guided -- --nocapture
cargo run -p memd-client --bin memd -- setup --guided --summary
```

**Commit:** `feat(setup): add guided setup summary`

### Task B3: Add setup report JSON

**Objective:** Make setup measurable and dogfoodable.

**Command:**

```bash
memd setup --guided --json > docs/verification/setup-runs/<date>-<machine>.json
```

**Schema fields:**

- machine OS
- memd version/commit
- elapsed seconds
- step reports
- setup score
- safe repair actions taken
- user action required
- final doctor status

**Verification:** JSON parses and contains no secrets.

**Commit:** `feat(setup): emit guided setup report json`

---

## Workstream C: First-run demo and wow moment

### Task C1: Add `memd setup-demo --summary`

**Objective:** Prove memory works in under 30 seconds.

**Flow:**

```text
1. Write a harmless demo memory: "memd setup demo favorite color is blue"
2. Compile/refresh bundle
3. Lookup demo query
4. Show result
5. Clean up or mark demo as setup evidence
```

**Files:**
- Add CLI args/runtime for `setup-demo`
- Tests for isolated temp bundle

**Output:**

```text
memd demo: saved a test memory
memd demo: recalled it successfully
Result: favorite color is blue
You are ready.
```

**Safety:**

- Use a clearly tagged setup-demo memory.
- Do not pollute real user memory unless user consents.
- For real bundles, ask or use temp root by default.

**Verification:**

```bash
cargo test -p memd-client setup_demo -- --nocapture
cargo run -p memd-client --bin memd -- setup-demo --summary --root /tmp/memd-demo
```

**Commit:** `feat(setup): add first-run memory demo`

### Task C2: Wire demo into installer end state

**Objective:** Installer ends with proof, not just "ready."

**Files:**
- Modify: `scripts/install-memd.sh`

**Behavior:**

- After `memd doctor --summary`, print:

```text
memd install: next proof step:
  memd setup-demo --summary
```

- If safe and non-interactive mode allows, run demo automatically with temp root.

**Verification:** run script in temp checkout or dry-run mode.

**Commit:** `chore(setup): point installer to first-run proof`

---

## Workstream D: Failure taxonomy and self-repair

### Task D1: Create setup failure registry

**Objective:** Name common failures and fixes.

**Files:**
- Create: `docs/setup/failure-registry.md`
- Create or modify Rust failure mapping module

**Minimum failures:**

| Failure | User sees | Fix | Verify |
| --- | --- | --- | --- |
| Missing Rust/cargo | "Rust is needed to build memd from source" | rustup install link | `cargo --version` |
| PATH missing | "memd installed but shell cannot find it" | add `~/.local/bin` | `command -v memd` |
| Stale binary | "Installed memd is older than checkout" | rerun installer | `memd --version` or capability check |
| Bundle missing | ".memd is not initialized" | `memd setup --summary` | `memd status --summary` |
| Doctor red | "Setup health failed" | exact doctor repair | `memd doctor --repair --summary` |
| Mac Bridge missing | "Apple services bridge is not installed" | install command | launchctl check |
| Server unavailable | "memd server is not reachable" | start server or set URL | health endpoint |
| Permission denied | "memd cannot write bundle path" | chmod/chown guidance | write test |

**Verification:** docs and CLI mappings cover every registry entry.

**Commit:** `docs(setup): define setup failure registry`

### Task D2: Upgrade `memd doctor` beginner output

**Objective:** Doctor becomes the trusted product support surface.

**Output contract:**

```text
memd doctor: RED
Problem: memd cannot write .memd/mem.md
Why it matters: memory cannot refresh after each turn
Fix: chmod u+w .memd/mem.md
Verify: memd doctor --summary
Details: run with --verbose
```

**Rules:**

- Beginner `--summary` never prints noisy stack traces.
- `--json` includes stable machine-readable issue codes.
- `--repair` only performs safe, reversible fixes.

**Verification:** doctor tests for red/yellow/green states.

**Commit:** `feat(doctor): make setup failures actionable`

### Task D3: Add repair confidence levels

**Objective:** Avoid dangerous auto-fixes.

**Levels:**

- `auto_safe`: can fix without asking
- `ask_first`: show command and ask/require explicit flag
- `manual_only`: explain, never run automatically

**Verification:** tests prove no destructive repair runs under `--summary` alone.

**Commit:** `feat(doctor): classify setup repairs by safety`

---

## Workstream E: Setup scorecard and proof harness

### Task E1: Add setup scorecard doc

**Objective:** Make scoring honest and user-experience based.

**Files:**
- Create: `docs/verification/setup-experience-scorecard.md`

**Axes:**

1. comprehension
2. install success
3. first-run success
4. failure recovery
5. docs clarity
6. update safety
7. trust/privacy clarity
8. support burden

Each axis has 0/1/3/5/10/15/20/25 bands.

**Commit:** `docs(setup): add hands-on setup scorecard`

### Task E2: Add fresh-machine setup proof script

**Objective:** Reproduce setup from clean-ish environment and capture evidence.

**Files:**
- Create: `scripts/verify/setup-experience-smoke.sh`

**Script steps:**

```bash
scripts/install-memd.sh
memd doctor --summary
memd setup --guided --json
memd setup-demo --summary
memd dogfood status --summary || true
```

**Artifacts:**

- `docs/verification/setup-runs/<date>-setup-smoke.md`
- `docs/verification/setup-runs/<date>-setup-smoke.json`

**Verification:** script runs on maintainer machine first, then fresh machine.

**Commit:** `test(setup): add setup experience smoke proof`

### Task E3: Add human setup trial template

**Objective:** Capture real user friction, not model self-review.

**Files:**
- Create: `docs/verification/setup-runs/HUMAN-TRIAL-TEMPLATE.md`

**Fields:**

- user role
- OS
- starting knowledge
- elapsed time
- commands run
- where they paused
- errors hit
- did they need live help?
- first successful memory proof?
- confidence score
- notes in user's words

**Commit:** `docs(setup): add human setup trial template`

---

## Workstream F: Packaging and distribution path

### Task F1: Decide install channel roadmap

**Objective:** Source checkout is not enough for Apple-level public setup.

**Options to evaluate:**

- `cargo install memd` after crate packaging
- GitHub release binary tarballs
- Homebrew tap for macOS
- shell installer that downloads signed release
- Nix/devcontainer optional lanes

**Deliverable:** `docs/setup/distribution-plan.md`

**CEO recommendation:** Start with GitHub release binary + Homebrew tap plan. Keep source installer for devs.

**Commit:** `docs(setup): plan public install channels`

### Task F2: Add update/uninstall commands or scripts

**Objective:** Setup product includes lifecycle, not only install.

**Commands/docs:**

```bash
memd update --summary
memd uninstall --dry-run
```

If full commands are too much for first pass, create scripts and docs first:

- `scripts/update-memd.sh`
- `scripts/uninstall-memd.sh --dry-run`

**Verification:** dry-run shows files touched and never deletes user memory by default.

**Commit:** `feat(setup): add safe update and uninstall path`

---

## Workstream G: Trust, privacy, and beginner safety

### Task G1: Add "Where your data lives" doc

**Objective:** Build trust before user stores memory.

**Files:**
- Create: `docs/setup/data-and-privacy.md`
- Link from README quickstart and dogfood docs

**Must answer:**

- Where is `.memd`?
- What gets written?
- What leaves the machine by default?
- What changes when server/RAG/sync is enabled?
- How to inspect/delete/export data?
- What dogfood consent means?

**Commit:** `docs(setup): explain data location and privacy plainly`

### Task G2: Add secret-redaction check to setup reports

**Objective:** Setup evidence must be shareable.

**Behavior:**

- JSON report redacts tokens, private keys, session values, URLs with credentials.
- Tests include fake secret strings.

**Verification:** test fails if secret-looking value appears in report.

**Commit:** `test(setup): redact secrets from setup reports`

---

## Workstream H: Real user validation gates

### Task H1: Define 5-star setup gate

**Gate:** Maintainer fresh checkout passes without manual repair.

**Evidence:**

- one macOS run
- one Linux run if available
- setup smoke artifact
- zero secret leaks

### Task H2: Define 10-star setup gate

**Gate:** Fresh developer who did not build memd completes install and first demo using docs only.

**Evidence:**

- timed setup trial
- no live handholding
- all errors resolved from docs/doctor

### Task H3: Define 15-star setup gate

**Gate:** Non-expert technical user completes guided setup with at most one doc search.

**Evidence:**

- human trial template
- confusion log
- docs patch after trial

### Task H4: Define 20-star setup gate

**Gate:** Public install channel works across supported OS targets.

**Evidence:**

- package checks
- clean VM runs
- update/uninstall proof

### Task H5: Define 25-star setup gate

**Gate:** 5 external users install, run first demo, recover from at least one induced failure, and say they would trust it for real agent memory.

**Evidence:**

- 5 setup trial records
- median time-to-first-memory <= 10 minutes
- install success >= 95% across trial attempts
- zero live handholding for happy path
- all common failures have doctor/docs fixes
- external written review of docs clarity

**Commit:** `docs(setup): define setup experience star gates`

---

## Error & Rescue Registry

| Codepath | What can go wrong | Rescue action | User sees | Test? |
| --- | --- | --- | --- | --- |
| installer cargo check | `cargo` missing | explain Rust install | exact install URL/command | yes |
| installer build | compile fails | show log path and issue summary | "build failed" with next step | yes |
| PATH setup | shell rc unknown | print exact PATH line | "add this line" | yes |
| bundle init | write denied | no auto chmod unless safe | path + permission fix | yes |
| doctor | multiple failures | grouped issue codes | top 3 fixes first | yes |
| device add | evidence path missing | create dirs safely | "registered" or exact blocker | yes |
| Mac Bridge install | launchctl fail | show launchctl command | bridge not ready + fix | yes/mac only |
| setup demo | recall fails | show capture/compile/lookup stage | demo failed at stage X | yes |
| setup report | secret present | redact before write | no secret output | yes |
| update | old binary in use | preserve old binary until new passes | rollback command | yes |
| uninstall | user data risk | dry-run default, never delete memory by default | file list | yes |

Any row without a test is a critical gap before claiming 10-star setup.

---

## Data Flow Diagram

```text
User
  |
  v
README / START-HERE
  |
  v
scripts/install-memd.sh
  |
  +--> preflight: cargo, OS, PATH, repo
  |
  +--> cargo install memd
  |
  +--> memd setup --summary
  |
  +--> memd doctor --summary / --repair
  |
  +--> memd device add --summary
  |
  +--> optional Mac Bridge install
  |
  v
memd setup --guided --summary
  |
  +--> step report
  +--> fix commands
  +--> setup score
  +--> JSON artifact
  |
  v
memd setup-demo --summary
  |
  +--> capture demo memory
  +--> compile/refresh
  +--> lookup demo memory
  +--> show success
  |
  v
User trusts product enough to dogfood
```

Shadow paths:

```text
nil input      -> use defaults, explain them
empty config   -> initialize bundle
bad PATH       -> print shell-specific fix
server missing -> local-only mode or exact start command
doctor red     -> issue code + repair guidance
recall fail    -> stage-specific setup-demo failure
```

---

## Test Plan

### Unit tests

- setup journey model status transitions
- failure registry maps every issue to fix/verify text
- secret redaction
- setup score calculation
- demo memory lifecycle

### Integration tests

- `memd setup --guided --summary` on temp root
- `memd setup --guided --json` schema
- `memd setup-demo --summary --root <tmp>`
- `memd doctor --repair --summary` safe repairs only

### Script tests

- `scripts/install-memd.sh` dry/temp run
- `scripts/verify/setup-experience-smoke.sh`
- docs link/lint checks

### Human tests

- maintainer fresh run
- fresh developer run
- brother-style/non-expert run when available
- induced failure recovery run

---

## Verification Commands

Run after implementation:

```bash
cargo fmt --check
cargo test -p memd-client setup_journey -- --nocapture
cargo test -p memd-client setup_guided -- --nocapture
cargo test -p memd-client setup_demo -- --nocapture
cargo test -p memd-client doctor -- --nocapture
cargo check -p memd-client
scripts/doc-lint.sh
scripts/lint-links.sh
scripts/verify/setup-experience-smoke.sh
scripts/verify/25-star-roadmap-audit.sh
git diff --check
```

Fresh-machine proof, not optional before high score:

```bash
scripts/verify/setup-experience-smoke.sh 2>&1 | tee docs/verification/setup-runs/$(date +%F)-fresh-machine.log
```

---

## NOT in scope

- New memory algorithms unrelated to setup.
- New RAG benchmark lifts.
- New multi-org federation gates.
- Revenue/customer claims.
- Calling 25-star externally closed without external users.
- Hiding complexity by deleting power-user docs.

---

## Dream State Delta

```text
CURRENT
Deep engine, many docs, install script, doctor, dogfood gates,
but setup still feels like a maintainer workflow.

THIS PLAN
Turns setup into a guided product with proof, repair, docs, score,
and real setup evidence.

12-MONTH IDEAL
memd is installed by external developers the way they install trusted devtools:
one command, clear health, no mystery, safe updates, visible trust, boring reliability.
```

---

## Phase Order

1. **Docs routing first** so the next person is less lost immediately.
2. **Guided setup model** so product behavior has a spine.
3. **First-run demo** so setup ends with proof.
4. **Doctor/failure registry** so recovery is productized.
5. **Scorecard/proof harness** so claims are honest.
6. **Update/uninstall/privacy** so trust is complete.
7. **Human trials** so scoring leaves the maintainer bubble.

---

## Atomic Commit Plan

1. `docs(setup): route first-time users before maintainer recovery`
2. `docs(readme): make quickstart prove first memory success`
3. `docs(setup): add beginner setup guide layers`
4. `feat(setup): model guided setup journey`
5. `feat(setup): add guided setup summary`
6. `feat(setup): emit guided setup report json`
7. `feat(setup): add first-run memory demo`
8. `chore(setup): point installer to first-run proof`
9. `docs(setup): define setup failure registry`
10. `feat(doctor): make setup failures actionable`
11. `feat(doctor): classify setup repairs by safety`
12. `docs(setup): add hands-on setup scorecard`
13. `test(setup): add setup experience smoke proof`
14. `docs(setup): add human setup trial template`
15. `docs(setup): plan public install channels`
16. `feat(setup): add safe update and uninstall path`
17. `docs(setup): explain data location and privacy plainly`
18. `test(setup): redact secrets from setup reports`
19. `docs(setup): define setup experience star gates`
20. `docs(setup): add final Apple-level setup proof packet`

---

## Kill Criteria

Stop and revise if any of these happen:

- Guided setup adds more confusion than docs alone.
- `memd setup --guided` duplicates installer logic instead of orchestrating it.
- Repair commands risk user data.
- Setup score improves by self-grading but not by human trials.
- Fresh-machine install still needs live handholding after Workstream D.
- Docs become longer but not clearer.

---

## CEO Verdict

This is the right next layer.

The product already has serious engine work. The bottleneck is trust at the front door. If setup feels fragile, users will never reach the memory OS. If setup feels inevitable, all the deep work finally has a path into real hands.

Build the setup product. Score the lived experience. Do not claim the star until fresh users prove it.
