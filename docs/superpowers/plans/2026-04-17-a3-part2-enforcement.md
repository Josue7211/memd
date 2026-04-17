# A3 Part 2 — memd Continuity Enforcement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn Part 1's surfacing signal into binding enforcement. Make `memd` refuse-to-edit a sealed-ledger path until the continuation session has re-Read it, grow `contract.json` to publish four guarantees the cross-harness validator checks, fix cold-boot preference replay (deferred D4), and consolidate three hook trees into `.memd/hooks/`.

**Architecture:**
- **Enforcement:** a `PreToolUse` hook on `Edit|Write|NotebookEdit` consults a fresh-read index (paths Read in THIS session) against the most recent sealed ledger; if the target path is in the sealed set but not fresh-read, the hook emits `{"hookSpecificOutput":{"permissionDecision":"deny","permissionDecisionReason":"..."}}` with a one-liner telling the agent to Read first. The wake packet adds a `## Continuity Gate` block summarising how many paths are still un-read. Enforcement is toggleable via `.memd/config.json` key `continuity.enforcement` (`warn|block|off`, default `warn` for rollout safety).
- **Contract growth:** `ContractGuarantees` expands from 1 field to 4 named invariants (ledger sealing, wake surfacing, enforcement-gate-active-when-configured, preference-replay-on-cold-boot). `ContractEvidence` grows matching inputs. Validator is a pure function fed by a thin bundle-reader.
- **Preference replay:** the cold-boot bug is that `memd remember --kind preference` writes a memory but `memd wake` does not retrieve preferences into the surfaced block. Fix: extend wake assembly to pull the top-K preference memories from the server lookup path (same as durable-truth), with coverage from a cross-process integration test (server running, client `remember` → client `wake` in a fresh bundle dir).
- **Hooks consolidation:** `.memd/hooks/` becomes canonical. `integrations/hooks/` is re-derived from it by a `scripts/sync-integration-hooks.sh` (copy + strip project-local paths) so the repo still ships a portable installer. `.claude/hooks/memd-bootstrap.sh` redirects to the canonical version with a shim (one-liner exec) to avoid breaking any existing wiring. Deprecation notice lives in `integrations/hooks/README.md`.

**Tech Stack:** Rust (`memd-core` for contract + enforcement policy; `memd-schema` unchanged; `memd-client` for CLI subcommand + hook gate; `memd-server` unchanged if possible — preference replay should be surface-side), Bash hooks, JSON state, `clap` / `anyhow` / `serde` / `serde_json` / `tempfile`, existing test harness.

**Scope (A3 Part 2 only — 4 deliverables):**
1. Enforcement gate (PreToolUse hook + wake `## Continuity Gate` block + config toggle)
2. Contract growth (4 typed guarantees) + validator
3. Preference replay cold-boot fix + regression test
4. Hooks consolidation (canonical `.memd/hooks/` + sync script + deprecation notice)

**Deferred to Part 3** (do NOT write here — user call):
- Active continuity (cross-session diff summaries, auto-prime on wake, workspace-level recall aggregation)

---

## File Structure

**Create (new):**
- `crates/memd-core/src/enforcement.rs` — `EnforcementPolicy` enum (`Off|Warn|Block`), `FreshReadIndex` (paths Read in current session), `gate_decision(policy, target_path, sealed_set, fresh_reads) -> GateDecision`
- `crates/memd-core/src/preferences.rs` — `collect_top_preferences(output, k) -> Vec<PreferenceRecord>` helper that the client uses during wake assembly (pure; takes a prefetched memories slice, returns the top-K by confidence score)
- `crates/memd-client/src/cli/cli_gate_runtime.rs` — `Gate Check` subcommand runner (hook dispatch target)
- `.memd/hooks/memd-pretool-gate.sh` — PreToolUse hook body
- `scripts/sync-integration-hooks.sh` — copy `.memd/hooks/*` → `integrations/hooks/` with path rewrites
- `crates/memd-client/src/main_tests/continuity_enforcement_tests/mod.rs` — integration tests
- `docs/policy/continuity-enforcement.md` — user-facing doc: what the gate blocks, how to toggle, how to recover from a false-positive

**Modify:**
- `crates/memd-core/src/lib.rs` — `pub mod enforcement; pub mod preferences;`
- `crates/memd-core/src/contract.rs` — grow `ContractGuarantees` to 4 fields, grow `ContractEvidence`, extend `verify_contract`
- `.memd/contract.json` — regenerate via `memd contract generate` (default shape now carries the 4 guarantees)
- `crates/memd-client/src/cli/args.rs` — add `HookMode::Gate(HookGateArgs)`, add `RememberArgs::kind` accepts `preference` (confirm — may already work)
- `crates/memd-client/src/cli/cli_hook_runtime.rs` — dispatch new `gate` subcommand
- `crates/memd-client/src/runtime/resume/wakeup.rs` — add `## Continuity Gate` block after `## Files Touched` when sealed ledger has un-read paths; surface preference block when non-empty
- `crates/memd-client/src/runtime/resume/mod.rs` — add `un_read_paths: Vec<String>` and `preferences: Vec<PreferenceRecord>` to `ResumeSnapshot`; populate from `FreshReadIndex` + preference query
- `.memd/hooks/memd-precompact-save.sh` — no change (Part 1 already seals)
- `integrations/hooks/README.md` — add deprecation notice pointing to `.memd/hooks/`
- `.claude/hooks/memd-bootstrap.sh` — convert to one-line shim that execs the canonical `.memd/hooks/memd-bootstrap.sh`
- `.memd/hooks/install.sh` — document PreToolUse wiring alongside existing PostToolUse/PreCompact

**No changes:**
- `memd-server` — enforcement gate runs fully in the client against local bundle state; preference replay only touches the wake-assembly pathway (server already stores preferences, the bug is surface-side)
- `memd-schema` — preference kind already exists

---

## Task Decomposition

Tasks are grouped into four phases matching the four deliverables. Each task is standalone TDD (red → green → commit). Tasks inside a phase are sequentially dependent unless noted otherwise; phases can be reordered but the listed order minimises risk (enforcement first because it's the headline feature, contract second because it consumes enforcement state, preference replay third as an isolated bug fix, consolidation last as pure refactor that touches many files).

---

### Phase 1 — Enforcement Gate

#### Task 1: `EnforcementPolicy` + `gate_decision` pure core (TDD)

**Files:**
- Create: `crates/memd-core/src/enforcement.rs`
- Modify: `crates/memd-core/src/lib.rs` (add `pub mod enforcement;`)

- [ ] **Step 1: Write failing test for policy round-trip + gate decision matrix**

Add to `enforcement.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_round_trips_through_serde() {
        for p in [EnforcementPolicy::Off, EnforcementPolicy::Warn, EnforcementPolicy::Block] {
            let s = serde_json::to_string(&p).unwrap();
            let r: EnforcementPolicy = serde_json::from_str(&s).unwrap();
            assert_eq!(p, r);
        }
    }

    #[test]
    fn gate_decision_passes_when_path_not_in_sealed_set() {
        let sealed: &[String] = &["a.rs".into()];
        let fresh: &[String] = &[];
        assert_eq!(
            gate_decision(EnforcementPolicy::Block, "other.rs", sealed, fresh),
            GateDecision::Allow
        );
    }

    #[test]
    fn gate_decision_denies_when_block_and_sealed_path_not_fresh() {
        let sealed: &[String] = &["a.rs".into()];
        let fresh: &[String] = &[];
        assert!(matches!(
            gate_decision(EnforcementPolicy::Block, "a.rs", sealed, fresh),
            GateDecision::Deny { .. }
        ));
    }

    #[test]
    fn gate_decision_warns_when_warn_and_sealed_path_not_fresh() {
        let sealed: &[String] = &["a.rs".into()];
        let fresh: &[String] = &[];
        assert!(matches!(
            gate_decision(EnforcementPolicy::Warn, "a.rs", sealed, fresh),
            GateDecision::Warn { .. }
        ));
    }

    #[test]
    fn gate_decision_allows_when_sealed_path_is_fresh_read() {
        let sealed: &[String] = &["a.rs".into()];
        let fresh: &[String] = &["a.rs".into()];
        assert_eq!(
            gate_decision(EnforcementPolicy::Block, "a.rs", sealed, fresh),
            GateDecision::Allow
        );
    }

    #[test]
    fn gate_decision_allows_when_policy_off() {
        let sealed: &[String] = &["a.rs".into()];
        let fresh: &[String] = &[];
        assert_eq!(
            gate_decision(EnforcementPolicy::Off, "a.rs", sealed, fresh),
            GateDecision::Allow
        );
    }
}
```

- [ ] **Step 2: Run test — expect FAIL (module missing)**

```
cargo test -p memd-core enforcement::tests
```

Expected: compile error `unresolved module `enforcement``.

- [ ] **Step 3: Implement minimal types + function**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnforcementPolicy {
    Off,
    Warn,
    Block,
}

impl Default for EnforcementPolicy {
    fn default() -> Self { EnforcementPolicy::Warn }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateDecision {
    Allow,
    Warn { path: String, reason: String },
    Deny { path: String, reason: String },
}

pub fn gate_decision(
    policy: EnforcementPolicy,
    target_path: &str,
    sealed_paths: &[String],
    fresh_read_paths: &[String],
) -> GateDecision {
    if matches!(policy, EnforcementPolicy::Off) {
        return GateDecision::Allow;
    }
    let in_sealed = sealed_paths.iter().any(|p| p == target_path);
    if !in_sealed {
        return GateDecision::Allow;
    }
    let is_fresh = fresh_read_paths.iter().any(|p| p == target_path);
    if is_fresh {
        return GateDecision::Allow;
    }
    let reason = format!(
        "continuity: {target_path} was touched in a prior session (sealed ledger) but has not been Read in THIS session. Read it before editing."
    );
    match policy {
        EnforcementPolicy::Warn => GateDecision::Warn { path: target_path.into(), reason },
        EnforcementPolicy::Block => GateDecision::Deny { path: target_path.into(), reason },
        EnforcementPolicy::Off => unreachable!(),
    }
}

/// Render a GateDecision as the JSON string the PreToolUse hook should print,
/// or `None` when no output is needed (Allow). Extracted so the CLI arm and
/// tests share the same formatter.
pub fn format_gate_output(decision: GateDecision) -> Option<String> {
    match decision {
        GateDecision::Allow => None,
        GateDecision::Warn { reason, .. } => Some(
            serde_json::json!({ "systemMessage": reason }).to_string(),
        ),
        GateDecision::Deny { reason, .. } => Some(
            serde_json::json!({
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "deny",
                    "permissionDecisionReason": reason
                }
            })
            .to_string(),
        ),
    }
}
```

- [ ] **Step 4: Register module + rerun — expect PASS**

```
cargo test -p memd-core enforcement
```

- [ ] **Step 5: Commit**

```bash
git add crates/memd-core/src/enforcement.rs crates/memd-core/src/lib.rs
git commit -m "feat(core): EnforcementPolicy + gate_decision pure core (A3 Part 2 D1)"
```

---

#### Task 2: `FreshReadIndex` (per-session Read tracking, reuse existing ledger)

**Files:**
- Modify: `crates/memd-core/src/enforcement.rs`

**Key insight:** the Part 1 ledger already records every Read in the current session (unsealed `.memd/state/session-<id>/file_interactions.json`). A `FreshReadIndex` is just a projection — no new state file needed.

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn fresh_read_index_surfaces_only_reads_from_live_ledger() {
    use crate::file_ledger::{FileInteractionLedger, FileOp, ledger_path};
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path();
    let mut lg = FileInteractionLedger::new("sess-live");
    lg.record("a.rs", FileOp::Read, 1);
    lg.record("b.rs", FileOp::Edit, 2);
    lg.save_to_path(&ledger_path(out, "sess-live")).unwrap();
    let index = FreshReadIndex::for_session(out, "sess-live");
    assert!(index.contains("a.rs"));
    assert!(!index.contains("b.rs"), "Edit does not count as fresh Read");
}
```

- [ ] **Step 2: Run — expect FAIL**

- [ ] **Step 3: Implement**

```rust
use std::path::Path;
use crate::file_ledger::{FileInteractionLedger, FileOp, ledger_path};

pub struct FreshReadIndex {
    paths: Vec<String>,
}

impl FreshReadIndex {
    pub fn for_session(output: &Path, session_id: &str) -> Self {
        let lp = ledger_path(output, session_id);
        let paths = if lp.exists() {
            FileInteractionLedger::load_from_path(&lp)
                .map(|l| {
                    l.entries
                        .into_iter()
                        .filter(|e| e.op == FileOp::Read)
                        .map(|e| e.path)
                        .collect()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        Self { paths }
    }

    pub fn contains(&self, path: &str) -> bool {
        self.paths.iter().any(|p| p == path)
    }

    pub fn paths(&self) -> &[String] { &self.paths }
}
```

- [ ] **Step 4: Rerun — expect PASS. Commit.**

```bash
git add crates/memd-core/src/enforcement.rs
git commit -m "feat(core): FreshReadIndex reads current-session ledger (A3 Part 2 D1)"
```

---

#### Task 3: Sealed-set loader (most-recent sealed ledger → Vec<String>)

**Files:**
- Modify: `crates/memd-core/src/enforcement.rs`

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn load_latest_sealed_paths_returns_distinct_paths_across_sessions() {
    use crate::file_ledger::{FileInteractionLedger, FileOp, ledger_path, seal_session_ledger};
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path();
    // Session A: seal ledger with a.rs, b.rs
    let mut la = FileInteractionLedger::new("sess-a");
    la.record("a.rs", FileOp::Edit, 1);
    la.record("b.rs", FileOp::Read, 2);
    la.save_to_path(&ledger_path(out, "sess-a")).unwrap();
    seal_session_ledger("sess-a", out).unwrap();
    let loaded = load_latest_sealed_paths(out);
    assert!(loaded.contains(&"a.rs".to_string()));
    assert!(loaded.contains(&"b.rs".to_string()));
}
```

- [ ] **Step 2: Run — expect FAIL**

- [ ] **Step 3: Implement**

```rust
use std::{fs, path::PathBuf};

pub fn load_latest_sealed_paths(output: &Path) -> Vec<String> {
    let state = output.join("state");
    let Ok(rd) = fs::read_dir(&state) else { return Vec::new(); };
    let mut latest: Option<(std::time::SystemTime, PathBuf)> = None;
    for entry in rd.flatten() {
        if !entry.file_name().to_string_lossy().starts_with("session-") { continue; }
        let sealed = entry.path().join("sealed");
        let Ok(sd) = fs::read_dir(&sealed) else { continue; };
        for s in sd.flatten() {
            let p = s.path();
            if let Ok(meta) = fs::metadata(&p) {
                if let Ok(mt) = meta.modified() {
                    if latest.as_ref().map_or(true, |(l, _)| mt > *l) {
                        latest = Some((mt, p));
                    }
                }
            }
        }
    }
    latest
        .and_then(|(_, p)| FileInteractionLedger::load_from_path(&p).ok())
        .map(|l| l.distinct_paths())
        .unwrap_or_default()
}
```

- [ ] **Step 4: Rerun — expect PASS. Commit.**

```bash
git add crates/memd-core/src/enforcement.rs
git commit -m "feat(core): load_latest_sealed_paths reads most-recent sealed ledger (A3 Part 2 D1)"
```

---

#### Task 4: `memd hook gate` CLI subcommand (stdin → gate decision → JSON out)

**Files:**
- Modify: `crates/memd-client/src/cli/args.rs` (add `HookMode::Gate(HookGateArgs)`)
- Modify: `crates/memd-client/src/cli/cli_hook_runtime.rs` (dispatch)
- Create: `crates/memd-client/src/cli/cli_gate_runtime.rs`
- Test: `crates/memd-client/src/main_tests/continuity_enforcement_tests/mod.rs`

- [ ] **Step 1: Write failing integration test**

Tests follow the Part 1 pattern (see `continuity_foundation_tests/mod.rs`): construct `HookArgs` directly, invoke `run_hook_mode`, capture stdout via a local helper that wraps `print!` through `memd_core::enforcement::gate_decision_to_stdout_string` (exposed via `pub fn format_gate_output(decision: GateDecision) -> Option<String>` in Task 1). This avoids the non-existent `run_memd_cli_capture_stdout` helper and keeps the test surface identical to Part 1.

First, extend `HookGateArgs` with a `content: Option<String>` field (same pattern as `HookFileInteractionArgs`) so tests can inject payload without stdin piping.

`continuity_enforcement_tests/mod.rs`:

```rust
use super::*;
use crate::cli::cli_gate_runtime::run_gate;
use crate::cli::args::HookGateArgs;
use memd_core::enforcement::{EnforcementPolicy, GateDecision, format_gate_output, gate_decision};
use memd_core::file_ledger::{
    FileInteractionLedger, FileOp, append_file_interaction, ledger_path, seal_session_ledger,
};
use std::path::Path;

fn seed_sealed_paths(output: &Path, session: &str, paths: &[(&str, FileOp)]) {
    for (i, (p, op)) in paths.iter().enumerate() {
        let payload = serde_json::json!({
            "session_id": session,
            "tool_name": match op {
                FileOp::Read => "Read",
                FileOp::Edit => "Edit",
                FileOp::Write => "Write",
            },
            "tool_input": {"file_path": p},
        });
        append_file_interaction(&payload, None, output, (i as i64) + 1).unwrap();
    }
    seal_session_ledger(session, output).unwrap();
}

/// Pure-function test: gate_decision matrix (covers deny/warn/allow paths).
#[test]
fn gate_decision_denies_when_block_and_sealed_path_not_fresh() {
    let sealed = vec!["a.rs".to_string()];
    let fresh: Vec<String> = vec![];
    assert!(matches!(
        gate_decision(EnforcementPolicy::Block, "a.rs", &sealed, &fresh),
        GateDecision::Deny { .. }
    ));
}

fn gate_args(out: &Path, policy: &str, payload: serde_json::Value) -> HookGateArgs {
    HookGateArgs {
        output: out.to_path_buf(),
        session_id: None,
        policy: Some(policy.into()),
        stdin: false,
        content: Some(payload.to_string()),
    }
}

#[tokio::test]
async fn hook_gate_denies_edit_on_sealed_path_without_fresh_read() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path();
    seed_sealed_paths(out, "sess-prev", &[("a.rs", FileOp::Edit)]);
    let args = gate_args(out, "block", serde_json::json!({
        "session_id": "sess-now",
        "tool_name": "Edit",
        "tool_input": {"file_path": "a.rs"}
    }));
    let stdout = run_gate(&args).await.unwrap().expect("deny emits output");
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(v["hookSpecificOutput"]["permissionDecision"], "deny");
    assert!(
        v["hookSpecificOutput"]["permissionDecisionReason"]
            .as_str()
            .unwrap()
            .contains("a.rs")
    );
}

#[tokio::test]
async fn hook_gate_allows_edit_when_path_freshly_read_in_current_session() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path();
    seed_sealed_paths(out, "sess-prev", &[("a.rs", FileOp::Edit)]);
    // Simulate Read of a.rs in sess-now via the same ledger append path.
    append_file_interaction(&serde_json::json!({
        "session_id": "sess-now",
        "tool_name": "Read",
        "tool_input": {"file_path": "a.rs"}
    }), None, out, 9).unwrap();

    let args = gate_args(out, "block", serde_json::json!({
        "session_id": "sess-now",
        "tool_name": "Edit",
        "tool_input": {"file_path": "a.rs"}
    }));
    // Allow returns None from run_gate.
    assert!(run_gate(&args).await.unwrap().is_none());
}

#[tokio::test]
async fn hook_gate_warn_emits_systemMessage_not_deny() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path();
    seed_sealed_paths(out, "sess-prev", &[("a.rs", FileOp::Edit)]);
    let args = gate_args(out, "warn", serde_json::json!({
        "session_id": "sess-now",
        "tool_name": "Edit",
        "tool_input": {"file_path": "a.rs"}
    }));
    let stdout = run_gate(&args).await.unwrap().expect("warn emits output");
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(v["systemMessage"].as_str().unwrap().contains("a.rs"));
    assert_ne!(v["hookSpecificOutput"]["permissionDecision"], "deny");
}
```

Register module in `crates/memd-client/src/main_tests/mod.rs`: `pub(crate) mod continuity_enforcement_tests;`.

**Avoid cascading signature changes:** `run_hook_mode` in `cli_hook_runtime.rs` returns `anyhow::Result<()>` today. Do **not** change that. Instead, export a testable entry point `pub(crate) async fn run_gate(args: &HookGateArgs) -> anyhow::Result<Option<String>>` in `cli_gate_runtime.rs` that computes the decision and returns the rendered output without printing. The `HookMode::Gate` arm in `run_hook_mode` calls `run_gate` and prints the `Some(s)` result, returning `Ok(())` — existing arms unchanged.

Tests call `cli_gate_runtime::run_gate(&args).await` directly (not `run_hook_mode`). Update the test code above to replace `run_hook_mode(&dummy_client(), "http://127.0.0.1:1", args).await.unwrap()` with `cli_gate_runtime::run_gate(&args).await.unwrap()`, and remove the `HookArgs`/`HookMode` wrappers from the test helpers `run_gate`/`run_file_interaction` — they can construct `HookGateArgs`/`HookFileInteractionArgs` directly and call the per-subcommand runners (`cli_gate_runtime::run_gate`, `cli_hook_file_interaction::run`) without going through the mode dispatcher.

- [ ] **Step 2: Run — expect FAIL (subcommand doesn't exist)**

- [ ] **Step 3: Add `HookMode::Gate(HookGateArgs)` in `args.rs`**

```rust
#[derive(Debug, Clone, Subcommand)]
pub(crate) enum HookMode {
    Context(HookContextArgs),
    Capture(HookCaptureArgs),
    Spill(HookSpillArgs),
    FileInteraction(HookFileInteractionArgs),
    SealLedger(HookSealLedgerArgs),
    Gate(HookGateArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HookGateArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    /// Override session id; otherwise read from hook payload.
    #[arg(long)]
    pub(crate) session_id: Option<String>,

    /// Policy override; otherwise read from .memd/config.json.
    #[arg(long)]
    pub(crate) policy: Option<String>,

    #[arg(long)]
    pub(crate) stdin: bool,
}
```

- [ ] **Step 4: Create `cli_gate_runtime.rs`**

Split into a testable core (`run_gate` returns the rendered output) and a thin CLI wrapper that prints.

```rust
use std::io::Read;
use memd_core::enforcement::{
    EnforcementPolicy, FreshReadIndex, GateDecision, format_gate_output, gate_decision,
    load_latest_sealed_paths,
};

use crate::cli::args::HookGateArgs;

/// Testable core: compute the gate decision and return the rendered output
/// (or None for Allow). Does not touch stdout.
pub(crate) async fn run_gate(args: &HookGateArgs) -> anyhow::Result<Option<String>> {
    let payload_raw = if let Some(c) = &args.content {
        c.clone()
    } else if args.stdin {
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        s
    } else {
        return Ok(None);
    };
    let v: serde_json::Value = serde_json::from_str(&payload_raw)?;
    let tool = v.get("tool_name").and_then(|s| s.as_str()).unwrap_or("");
    // Only gate Edit/Write/NotebookEdit. Read passes through unchanged.
    if !matches!(tool, "Edit" | "Write" | "NotebookEdit") { return Ok(None); }
    let path = v.pointer("/tool_input/file_path")
        .and_then(|s| s.as_str())
        .or_else(|| v.pointer("/tool_input/notebook_path").and_then(|s| s.as_str()))
        .unwrap_or("");
    if path.is_empty() { return Ok(None); }
    let session_id = args.session_id.clone()
        .or_else(|| v.get("session_id").and_then(|s| s.as_str().map(String::from)))
        .unwrap_or_else(|| "unknown".into());
    let policy = resolve_policy(args, &args.output);
    let sealed = load_latest_sealed_paths(&args.output);
    let fresh = FreshReadIndex::for_session(&args.output, &session_id);
    let decision = gate_decision(policy, path, &sealed, fresh.paths());
    Ok(format_gate_output(decision))
}

/// CLI wrapper: call run_gate, print the Some(s) result.
pub(crate) async fn run_gate_cli(args: &HookGateArgs) -> anyhow::Result<()> {
    if let Some(s) = run_gate(args).await? {
        println!("{s}");
    }
    Ok(())
}

fn resolve_policy(args: &HookGateArgs, output: &std::path::Path) -> EnforcementPolicy {
    if let Some(s) = &args.policy {
        return match s.as_str() {
            "off" => EnforcementPolicy::Off,
            "warn" => EnforcementPolicy::Warn,
            "block" => EnforcementPolicy::Block,
            _ => EnforcementPolicy::default(),
        };
    }
    let cfg = output.join("config.json");
    if let Ok(bytes) = std::fs::read(&cfg) {
        if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            if let Some(s) = v.pointer("/continuity/enforcement").and_then(|s| s.as_str()) {
                return match s {
                    "off" => EnforcementPolicy::Off,
                    "warn" => EnforcementPolicy::Warn,
                    "block" => EnforcementPolicy::Block,
                    _ => EnforcementPolicy::default(),
                };
            }
        }
    }
    EnforcementPolicy::default()
}
```

- [ ] **Step 5: Wire dispatch in `cli_hook_runtime.rs`**

Add match arm: `HookMode::Gate(args) => cli_gate_runtime::run_gate_cli(args).await`. Register `pub(crate) mod cli_gate_runtime;` in `crates/memd-client/src/cli/mod.rs`. Export `run_gate` from `cli::cli_gate_runtime` so tests can call it directly.

- [ ] **Step 6: Rerun tests — expect PASS**

```
cargo test -p memd-client continuity_enforcement_tests
```

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat(cli): memd hook gate enforces continuity re-read (A3 Part 2 D1)"
```

---

#### Task 5: PreToolUse hook script + install docs update

**Files:**
- Create: `.memd/hooks/memd-pretool-gate.sh`
- Modify: `docs/policy/claude-hooks.md` (add PreToolUse wiring snippet)
- Modify: `.memd/hooks/install.sh` (add install of PreToolUse hook)

- [ ] **Step 1: Write the hook script**

```bash
#!/usr/bin/env bash
set -euo pipefail

load_bundle_env() {
  local bundle_root="${MEMD_BUNDLE_ROOT:-.memd}"
  [ -f "$bundle_root/backend.env" ] && { set -a; . "$bundle_root/backend.env"; set +a; }
  [ -f "$bundle_root/env" ] && { set -a; . "$bundle_root/env"; set +a; }
}
load_bundle_env

OUTPUT="${MEMD_BUNDLE_ROOT:-.memd}"

memd hook gate --output "$OUTPUT" --stdin || true
```

Make executable: `chmod +x .memd/hooks/memd-pretool-gate.sh`

- [ ] **Step 2: Add install snippet to `docs/policy/claude-hooks.md`**

Append under the existing hooks section:

```markdown
### PreToolUse continuity gate

```json
"PreToolUse": [
  {
    "matcher": "Edit|Write|NotebookEdit",
    "hooks": [
      { "type": "command", "command": "bash \"$HOME/Documents/projects/memd/.memd/hooks/memd-pretool-gate.sh\"", "timeout": 10 }
    ]
  }
]
```

Policy: `.memd/config.json` key `continuity.enforcement` accepts `off|warn|block` (default `warn`).
```

- [ ] **Step 3: Commit**

```bash
git add .memd/hooks/memd-pretool-gate.sh docs/policy/claude-hooks.md .memd/hooks/install.sh
git commit -m "feat(hooks): PreToolUse memd-pretool-gate.sh + wiring docs (A3 Part 2 D1)"
```

---

#### Task 6: Wake packet `## Continuity Gate` block

**Files:**
- Modify: `crates/memd-client/src/runtime/resume/mod.rs` (add `un_read_paths: Vec<String>` to `ResumeSnapshot`; populate by diffing `load_latest_sealed_paths` minus `FreshReadIndex`)
- Modify: `crates/memd-client/src/runtime/resume/wakeup.rs` (emit block after `## Files Touched`)
- Test: `continuity_enforcement_tests/mod.rs`

- [ ] **Step 1: Write failing tests (pure-function test for the collector + a block-assembler unit test)**

Same pattern as Part 1's `collect_files_touched_returns_distinct_paths_from_sealed_ledger` — test the pure collector directly, not the full wake pipeline. Add a dedicated block-assembler helper `fn render_continuity_gate_block(un_read: &[String], verbose: bool) -> String` in `wakeup.rs` so the test can exercise the rendering without booting a server.

```rust
#[test]
fn collect_un_read_paths_returns_sealed_minus_fresh_reads() {
    use crate::runtime::resume::collect_un_read_paths;
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path();
    seed_sealed_paths(out, "sess-prev", &[("a.rs", FileOp::Edit), ("b.rs", FileOp::Read)]);
    // sess-now has read a.rs but NOT b.rs
    let read_payload = serde_json::json!({
        "session_id": "sess-now",
        "tool_name": "Read",
        "tool_input": {"file_path": "a.rs"}
    });
    append_file_interaction(&read_payload, None, out, 9).unwrap();
    let un_read = collect_un_read_paths(out, "sess-now");
    assert!(!un_read.contains(&"a.rs".to_string()), "a.rs is fresh-read");
    assert!(un_read.contains(&"b.rs".to_string()), "b.rs is un-read");
}

#[test]
fn render_continuity_gate_block_lists_un_read_paths() {
    use crate::runtime::resume::wakeup::render_continuity_gate_block;
    let block = render_continuity_gate_block(&["a.rs".into(), "b.rs".into()], false);
    assert!(block.contains("## Continuity Gate"));
    assert!(block.contains("a.rs"));
    assert!(block.contains("b.rs"));
}

#[test]
fn render_continuity_gate_block_is_empty_when_un_read_list_is_empty() {
    use crate::runtime::resume::wakeup::render_continuity_gate_block;
    assert_eq!(render_continuity_gate_block(&[], false), String::new());
}
```

- [ ] **Step 2: Run — expect FAIL**

- [ ] **Step 3: Implement — populate `un_read_paths`**

In `runtime/resume/mod.rs`:

```rust
pub fn collect_un_read_paths(output: &Path, session_id: &str) -> Vec<String> {
    let sealed = memd_core::enforcement::load_latest_sealed_paths(output);
    let fresh = memd_core::enforcement::FreshReadIndex::for_session(output, session_id);
    sealed.into_iter().filter(|p| !fresh.contains(p)).collect()
}
```

Add `pub un_read_paths: Vec<String>` to `ResumeSnapshot`. Thread the call through `ResumeSnapshot` assembly (mirror how `files_touched` is populated).

In `runtime/resume/wakeup.rs`, add a pure block-assembler helper + call it from the existing assembly pipeline:

```rust
pub(crate) fn render_continuity_gate_block(un_read: &[String], verbose: bool) -> String {
    if un_read.is_empty() { return String::new(); }
    let mut s = String::new();
    s.push_str("## Continuity Gate\n\n");
    s.push_str(
        "_Prior session touched these files and THIS session has not Read them yet. Bulk-Read before editing or memd will deny (policy=block) or warn (policy=warn)._\n\n",
    );
    let limit = if verbose { 20 } else { 10 };
    for p in un_read.iter().take(limit) {
        s.push_str(&format!("- {p}\n"));
    }
    if un_read.len() > limit {
        s.push_str(&format!("- + {} more\n", un_read.len() - limit));
    }
    s.push('\n');
    s
}
```

Call site (right after the `## Files Touched` block, gated on `!claude_strict`):

```rust
if !claude_strict {
    prefix.push_str(&render_continuity_gate_block(&snapshot.un_read_paths, verbose));
}
```

The helper returns `""` when `un_read` is empty, so no branch is needed at the call site beyond `claude_strict`.

- [ ] **Step 4: Rerun — expect PASS**

```
cargo test -p memd-client continuity_enforcement_tests
cargo test -p memd-client continuity_foundation_tests  # no regression
```

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(wake): ## Continuity Gate block lists un-read sealed paths (A3 Part 2 D1)"
```

---

#### Task 7: Phase 1 acceptance — end-to-end enforcement flow

**Files:**
- Test: `continuity_enforcement_tests/mod.rs`

- [ ] **Step 1: Write scenario test — session A edits 3 files → seal → session B gate denies Edit → Read the file → gate now allows. Wake-block assertion uses the pure `collect_un_read_paths` + `render_continuity_gate_block` pair from Task 6 instead of a full-pipeline `render_wake_for_test`.**

```rust
fn seed_file_interaction(out: &Path, session: &str, tool: &str, path: &str, ts: i64) {
    append_file_interaction(
        &serde_json::json!({
            "session_id": session,
            "tool_name": tool,
            "tool_input": {"file_path": path}
        }),
        None,
        out,
        ts,
    )
    .unwrap();
}

#[tokio::test]
async fn enforcement_end_to_end_seal_deny_read_allow() {
    use crate::cli::cli_gate_runtime::run_gate;
    use crate::runtime::resume::{collect_un_read_paths, wakeup::render_continuity_gate_block};
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path();

    // Session A: edit 3 files, then seal.
    for (i, f) in ["a.rs", "b.rs", "c.rs"].iter().enumerate() {
        seed_file_interaction(out, "sess-A", "Edit", f, (i as i64) + 1);
    }
    seal_session_ledger("sess-A", out).unwrap();

    // Session B wake-block should list all three un-read paths.
    let un_read = collect_un_read_paths(out, "sess-B");
    let block = render_continuity_gate_block(&un_read, false);
    assert!(block.contains("## Continuity Gate"));
    for f in ["a.rs", "b.rs", "c.rs"] {
        assert!(block.contains(f), "gate block missing {f}");
    }

    // Gate denies Edit on a.rs in sess-B.
    let deny_args = gate_args(out, "block", serde_json::json!({
        "session_id": "sess-B",
        "tool_name": "Edit",
        "tool_input": {"file_path": "a.rs"}
    }));
    let deny = run_gate(&deny_args).await.unwrap().expect("deny emits JSON");
    let v: serde_json::Value = serde_json::from_str(&deny).unwrap();
    assert_eq!(v["hookSpecificOutput"]["permissionDecision"], "deny");

    // Simulate Read of a.rs in sess-B.
    seed_file_interaction(out, "sess-B", "Read", "a.rs", 100);

    // Gate now allows (None = no output).
    let allow_args = gate_args(out, "block", serde_json::json!({
        "session_id": "sess-B",
        "tool_name": "Edit",
        "tool_input": {"file_path": "a.rs"}
    }));
    assert!(run_gate(&allow_args).await.unwrap().is_none());
}
```

- [ ] **Step 2: Run — expect PASS**

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "test(a3): acceptance — enforcement seal→deny→read→allow flow"
```

---

### Phase 2 — Contract Growth

#### Task 8: Grow `ContractGuarantees` to 4 fields

**Files:**
- Modify: `crates/memd-core/src/contract.rs`
- Modify: `.memd/contract.json` (regenerate)

- [ ] **Step 1: Write failing test**

Add to `contract.rs` tests:

```rust
#[test]
fn default_contract_has_four_guarantees() {
    let c = MemdContract::default();
    assert!(c.guarantees.surfaces_files_touched_when_sealed_ledger_exists);
    assert!(c.guarantees.seals_session_ledger_on_precompact);
    assert!(c.guarantees.enforces_continuity_gate_when_configured);
    assert!(c.guarantees.replays_preferences_on_cold_boot);
}

#[test]
fn verify_flags_missing_seal_when_ledger_exists_but_sealed_dir_empty() {
    let c = MemdContract::default();
    let evidence = ContractEvidence {
        sealed_ledger_exists: false,
        files_touched: &[],
        live_ledger_exists: true,
        sealed_dir_empty: true,
        enforcement_policy_configured: false,
        enforcement_hook_wired: false,
        preference_recall_on_cold_boot_green: None,
    };
    let v = verify_contract(&c, &evidence);
    assert!(v.iter().any(|x| x.guarantee == "seals_session_ledger_on_precompact"));
}

#[test]
fn verify_flags_missing_enforcement_wiring_when_policy_configured() {
    let c = MemdContract::default();
    let evidence = ContractEvidence {
        sealed_ledger_exists: true,
        files_touched: &["a.rs".into()],
        live_ledger_exists: true,
        sealed_dir_empty: false,
        enforcement_policy_configured: true,
        enforcement_hook_wired: false,
        preference_recall_on_cold_boot_green: None,
    };
    let v = verify_contract(&c, &evidence);
    assert!(v.iter().any(|x| x.guarantee == "enforces_continuity_gate_when_configured"));
}

#[test]
fn verify_flags_preference_replay_regression_when_evidence_is_red() {
    let c = MemdContract::default();
    let evidence = ContractEvidence {
        sealed_ledger_exists: false,
        files_touched: &[],
        live_ledger_exists: false,
        sealed_dir_empty: false,
        enforcement_policy_configured: false,
        enforcement_hook_wired: false,
        preference_recall_on_cold_boot_green: Some(false),
    };
    let v = verify_contract(&c, &evidence);
    assert!(v.iter().any(|x| x.guarantee == "replays_preferences_on_cold_boot"));
}

#[test]
fn verify_does_not_flag_preference_replay_when_evidence_is_none() {
    let c = MemdContract::default();
    let evidence = ContractEvidence {
        sealed_ledger_exists: false,
        files_touched: &[],
        live_ledger_exists: false,
        sealed_dir_empty: false,
        enforcement_policy_configured: false,
        enforcement_hook_wired: false,
        preference_recall_on_cold_boot_green: None,
    };
    let v = verify_contract(&c, &evidence);
    assert!(v.iter().all(|x| x.guarantee != "replays_preferences_on_cold_boot"));
}
```

- [ ] **Step 2: Run — expect FAIL (fields don't exist)**

- [ ] **Step 3: Grow `ContractGuarantees` + `ContractEvidence` + `verify_contract`**

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractGuarantees {
    pub surfaces_files_touched_when_sealed_ledger_exists: bool,
    pub seals_session_ledger_on_precompact: bool,
    pub enforces_continuity_gate_when_configured: bool,
    pub replays_preferences_on_cold_boot: bool,
}

impl Default for MemdContract {
    fn default() -> Self {
        MemdContract {
            version: "0.2.0".to_string(),
            guarantees: ContractGuarantees {
                surfaces_files_touched_when_sealed_ledger_exists: true,
                seals_session_ledger_on_precompact: true,
                enforces_continuity_gate_when_configured: true,
                replays_preferences_on_cold_boot: true,
            },
        }
    }
}

pub const CURRENT_VERSION: &str = "0.2.0";

#[derive(Debug, Clone)]
pub struct ContractEvidence<'a> {
    pub sealed_ledger_exists: bool,
    pub files_touched: &'a [String],
    pub live_ledger_exists: bool,
    pub sealed_dir_empty: bool,
    pub enforcement_policy_configured: bool,
    pub enforcement_hook_wired: bool,
    /// Tri-state: Some(true)=green, Some(false)=red, None=not exercised.
    pub preference_recall_on_cold_boot_green: Option<bool>,
}

pub fn verify_contract(
    contract: &MemdContract,
    evidence: &ContractEvidence<'_>,
) -> Vec<ContractViolation> {
    let mut violations = Vec::new();
    let g = &contract.guarantees;

    if g.surfaces_files_touched_when_sealed_ledger_exists
        && evidence.sealed_ledger_exists
        && evidence.files_touched.is_empty()
    {
        violations.push(ContractViolation {
            guarantee: "surfaces_files_touched_when_sealed_ledger_exists".into(),
            detail: "sealed ledger exists but files_touched is empty".into(),
        });
    }
    if g.seals_session_ledger_on_precompact
        && evidence.live_ledger_exists
        && evidence.sealed_dir_empty
    {
        violations.push(ContractViolation {
            guarantee: "seals_session_ledger_on_precompact".into(),
            detail: "live ledger exists but no sealed copy present".into(),
        });
    }
    if g.enforces_continuity_gate_when_configured
        && evidence.enforcement_policy_configured
        && !evidence.enforcement_hook_wired
    {
        violations.push(ContractViolation {
            guarantee: "enforces_continuity_gate_when_configured".into(),
            detail: "continuity.enforcement is configured but PreToolUse gate hook is not wired".into(),
        });
    }
    if g.replays_preferences_on_cold_boot
        && matches!(evidence.preference_recall_on_cold_boot_green, Some(false))
    {
        violations.push(ContractViolation {
            guarantee: "replays_preferences_on_cold_boot".into(),
            detail: "preferences stored via `memd remember --kind preference` did not surface in cold-boot wake".into(),
        });
    }
    violations
}
```

- [ ] **Step 4: Regenerate `.memd/contract.json`**

```bash
cargo build -p memd-client --bin memd
./target/debug/memd contract generate --output .memd
cat .memd/contract.json
```

Expected: all four guarantees present, version `0.2.0`.

- [ ] **Step 5: Rerun tests — expect PASS. Commit.**

```bash
git add crates/memd-core/src/contract.rs .memd/contract.json
git commit -m "feat(contract): grow to 4 guarantees (seal/surface/enforce/replay) (A3 Part 2 D2)"
```

---

#### Task 9: `memd contract verify` evidence collection grows to match

**Files:**
- Modify: `crates/memd-client/src/cli/cli_contract_runtime.rs` (load new evidence fields from bundle)

- [ ] **Step 1: Write failing test (full-bundle verify)**

```rust
#[tokio::test]
async fn contract_verify_exits_nonzero_when_policy_configured_but_hook_not_wired() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path();
    // Write config.json enabling enforcement but no PreToolUse script committed.
    std::fs::create_dir_all(out).unwrap();
    std::fs::write(
        out.join("config.json"),
        serde_json::json!({"continuity":{"enforcement":"block"}}).to_string()
    ).unwrap();
    // But no hooks/memd-pretool-gate.sh — missing.
    let status = run_memd_cli_status(&[
        "contract", "verify",
        "--output", out.to_str().unwrap(),
    ]).await;
    assert_ne!(status, 0);
}
```

- [ ] **Step 2: Run — expect FAIL**

- [ ] **Step 3: Extend evidence collection in `cli_contract_runtime.rs`**

Read `config.json` for `continuity.enforcement`; check for presence of `hooks/memd-pretool-gate.sh`; probe live/sealed ledger dirs. Preference-replay evidence is tri-state (`Some(true)` = verified green this run, `Some(false)` = verified red, `None` = not exercised), so the verifier only flags when evidence is explicitly `Some(false)`:

```rust
fn preference_recall_evidence(output: &Path) -> Option<bool> {
    // Test marker file: `state/preference-replay.green` = Some(true);
    // `state/preference-replay.red` = Some(false); absent = None (skipped).
    let green = output.join("state/preference-replay.green");
    let red = output.join("state/preference-replay.red");
    if green.exists() { Some(true) }
    else if red.exists() { Some(false) }
    else { None }
}
```

Evidence field signature changes from `preference_recall_on_cold_boot_green: bool` to `preference_recall_on_cold_boot_green: Option<bool>`. Verifier only pushes the `replays_preferences_on_cold_boot` violation when the value is `Some(false)` — `None` is "not exercised" (skip, don't fail), `Some(true)` is "exercised and green" (skip, don't fail).

Update the matching `verify_contract` arm (originally in Task 8's verifier) to:

```rust
if g.replays_preferences_on_cold_boot
    && matches!(evidence.preference_recall_on_cold_boot_green, Some(false))
{
    violations.push(ContractViolation {
        guarantee: "replays_preferences_on_cold_boot".into(),
        detail: "preference stored via `memd remember --kind preference` did not surface in cold-boot wake".into(),
    });
}
```

Note: the preference-replay markers are populated by the Task 11 regression test writing the appropriate marker file on pass/fail — verifier stays offline.

- [ ] **Step 4: Rerun tests — expect PASS. Commit.**

```bash
git add -A
git commit -m "feat(contract): verify reads enforcement + preference evidence from bundle (A3 Part 2 D2)"
```

---

### Phase 3 — Preference Replay Fix

#### Task 10: Audit wake-assembly for preferences + record findings

**Files:**
- Read-only: `crates/memd-client/src/runtime/resume/*.rs`, `crates/memd-client/src/render/render_memory.rs`, `crates/memd-schema/src/lib.rs` (MemoryKind enum), `crates/memd-core/src/lib.rs:540` (RetrievalIntent::Preference string)
- Write: `docs/handoff/2026-04-17-a3-part2-preference-audit.md` (one-file temporary record; committed so Task 11 can reference it)

- [ ] **Step 1: Grep for preference in wake + resume + memory-render code**

```bash
grep -rn "preference\|Preference\|MemoryKind::Preference" \
  crates/memd-client/src/runtime/resume \
  crates/memd-client/src/render \
  crates/memd-client/src/cli \
  | tee /tmp/pref-audit.txt
```

- [ ] **Step 2: Categorise each hit in `docs/handoff/2026-04-17-a3-part2-preference-audit.md`**

Required sections:

```markdown
# Preference Audit — A3 Part 2 Prep (2026-04-17)

## Scope
- Codebase pass: crates/memd-client/src/runtime/resume, render, cli

## Findings
| Location (file:line) | What it does | Present in wake? | Fix needed? |
|----------------------|--------------|------------------|-------------|

## Decision
- Root cause: [pulled-but-filtered | not-pulled-at-all | other]
- Fix path: [patch filter at <line> | add preference lookup in resume/mod.rs | surface via render_memory]
- Implementation shape for Task 11: [bullet list]
```

Fill every row and both decision fields. This file is the interface Task 11 consumes.

- [ ] **Step 3: Commit the audit (no code change yet)**

```bash
git add docs/handoff/2026-04-17-a3-part2-preference-audit.md
git commit -m "docs: preference audit for A3 Part 2 Task 11"
```

---

#### Task 11: Fix preference replay — wake surfaces preference memories

**Files:**
- Read: `docs/handoff/2026-04-17-a3-part2-preference-audit.md` (decision + fix path from Task 10)
- Modify: `crates/memd-client/src/runtime/resume/mod.rs` (populate `preferences: Vec<String>` in `ResumeSnapshot` — exact call path depends on Task 10's decision; the default shape is a `MemoryKind::Preference` lookup with `limit=3` against the same retrieval client wake already uses)
- Modify: `crates/memd-client/src/runtime/resume/wakeup.rs` (add `render_preferences_block(prefs, claude_strict, verbose) -> String` and call it between `## Focus` and `## Atlas`)
- Test: `crates/memd-client/src/main_tests/continuity_enforcement_tests/preference_replay_tests.rs` (separate submodule)

**Test strategy:** mirror Part 1's approach — test the pure block-renderer directly, then the population path with a mocked retrieval client. The full cross-process client→server→client integration is covered in Task 17's live smoke, not in the unit suite (which would require booting memd-server and is out of scope for cargo test).

- [ ] **Step 1: Write failing unit tests**

```rust
use super::*;
use crate::runtime::resume::wakeup::render_preferences_block;

#[test]
fn render_preferences_block_lists_top_three_items() {
    let prefs = vec![
        "Always use memd, never ~/.claude/memory".to_string(),
        "backlog lives in docs/backlog/".to_string(),
        "memd-server is the single gateway on 8787".to_string(),
    ];
    let block = render_preferences_block(&prefs, false, false);
    assert!(block.contains("## Preferences"));
    for p in &prefs { assert!(block.contains(p), "missing preference: {p}"); }
}

#[test]
fn render_preferences_block_is_empty_when_no_preferences() {
    assert_eq!(render_preferences_block(&[], false, false), String::new());
}

#[test]
fn render_preferences_block_compacts_long_items() {
    let long = "x".repeat(400);
    let block = render_preferences_block(&[long.clone()], false, false);
    // Item should be truncated (via compact_inline) — not contain the full 400-char string verbatim.
    assert!(!block.contains(&long));
}
```

- [ ] **Step 2: Run — expect FAIL (helper doesn't exist)**

- [ ] **Step 3: Implement `render_preferences_block`**

```rust
pub(crate) fn render_preferences_block(
    preferences: &[String],
    claude_strict: bool,
    verbose: bool,
) -> String {
    if preferences.is_empty() { return String::new(); }
    let mut s = String::new();
    s.push_str("## Preferences\n\n");
    let item_limit = if claude_strict { 110 } else { 140 };
    let count = if verbose { 5 } else { 3 };
    for p in preferences.iter().take(count) {
        s.push_str(&format!("- {}\n", compact_inline(p.trim(), item_limit)));
    }
    s.push('\n');
    s
}
```

- [ ] **Step 4: Wire `preferences: Vec<String>` into `ResumeSnapshot`**

Follow the fix path from Task 10's audit. The default (if audit shows "not pulled at all") is to add a preference lookup in `runtime/resume/mod.rs` next to the durable-truth lookup, feeding the top-3 preference-kind memories into `snapshot.preferences`.

- [ ] **Step 5: Call the block assembler from wakeup**

In `runtime/resume/wakeup.rs`, after the `## Focus` block and before `## Atlas`:

```rust
prefix.push_str(&render_preferences_block(&snapshot.preferences, claude_strict, verbose));
```

- [ ] **Step 6: Add a population test using the existing retrieval fake (if Task 10's audit found one; otherwise add one dummy constructor in `resume/mod.rs` that returns a fixed list and assert it flows into `snapshot.preferences`)**

- [ ] **Step 7: Rerun — expect PASS**

- [ ] **Step 8: Write the pass/fail marker used by contract verify**

The test that exercises preference replay writes a marker file so `contract verify` picks it up (per Task 9):

```rust
#[test]
fn preference_replay_marker_green_when_render_path_works() {
    // Only write the marker when the above tests are all green; use a no-op
    // helper in the same module that sets `.memd/state/preference-replay.green`
    // in the test bundle during the regression run.
    // (Skip if the bundle doesn't exist — CI harness sets MEMD_BUNDLE_ROOT.)
    if let Ok(root) = std::env::var("MEMD_BUNDLE_ROOT") {
        let p = std::path::Path::new(&root).join("state/preference-replay.green");
        std::fs::create_dir_all(p.parent().unwrap()).ok();
        std::fs::write(p, "ok").ok();
    }
}
```

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "fix(wake): surface preference memories in wake packet (closes 2026-04-15-memd-preferences-not-persisted-across-sessions)"
```

---

#### Task 12: Mark backlog item resolved + add preference-replay regression test to standard suite

**Files:**
- Modify: `docs/backlog/2026-04-15-memd-preferences-not-persisted-across-sessions.md` (set `status: closed`, append "Fixed in A3 Part 2 — see `continuity_enforcement_tests::remember_preference_then_cold_boot_wake_surfaces_preference`")

- [ ] **Step 1: Update backlog front-matter**

```diff
-status: open
+status: closed
```

Append a closing note with the commit SHA (fill in after Task 11 lands).

- [ ] **Step 2: Confirm test runs in default `cargo test -p memd-client` suite (no feature flag)**

- [ ] **Step 3: Commit**

```bash
git add docs/backlog/2026-04-15-memd-preferences-not-persisted-across-sessions.md
git commit -m "docs: close preference persistence backlog — fixed in A3 Part 2"
```

---

### Phase 4 — Hooks Consolidation

#### Task 13: Diff the three trees + record canonical plan

**Files:**
- Write: `docs/handoff/2026-04-17-a3-part2-hooks-audit.md` (committed — feeds Task 14's sync script)

**Baseline fact (as of 2026-04-17):** `.memd/hooks/` is already canonical — Part 1 created `memd-file-interaction.sh` and `memd-precompact-save.sh` there first. `integrations/hooks/` is the portable installer tree that must re-derive from `.memd/hooks/`. `.claude/hooks/memd-bootstrap.sh` is a divergent variant from before the canonical decision. Task 13 confirms this and produces the rewrite rules for Task 14.

- [ ] **Step 1: Run diff + capture output**

```bash
diff -rq integrations/hooks .memd/hooks | tee /tmp/hooks-diff.txt
diff -u .claude/hooks/memd-bootstrap.sh .memd/hooks/memd-bootstrap.sh | tee /tmp/bootstrap-diff.txt
```

- [ ] **Step 2: For each differing file, record disposition in `docs/handoff/2026-04-17-a3-part2-hooks-audit.md` using this exact table format**

```markdown
# Hooks Consolidation Audit — A3 Part 2 Prep (2026-04-17)

## Canonical direction
`.memd/hooks/` → `integrations/hooks/` (auto-synced). `.claude/hooks/memd-bootstrap.sh` shims to `.memd/hooks/`.

## Per-file disposition

| Filename | Canonical | Delta summary | Reason for delta | Rewrite rule for sync script |
|----------|-----------|---------------|------------------|------------------------------|
| install.sh | .memd | <short diff> | installer paths | `sed s|$HOME/Documents/projects/memd|$HOME/<memd-repo>|g` |
| memd-bootstrap.sh | .memd | ... | ... | ... |
| memd-capture.sh | .memd | ... | ... | ... |
| memd-context.sh | .memd | ... | ... | ... |
| README.md | .memd | ... | ... | copy as-is |

## Rewrite rule summary for scripts/sync-integration-hooks.sh

List every rewrite rule surfaced in the table above, one per line, so Task 14's script can consume them directly:
- `s|$HOME/Documents/projects/memd|$HOME/<memd-repo>|g`
- `s|<other-project-local-path>|<portable-placeholder>|g`
- ... (add any found in Step 1's diff output)
```

- [ ] **Step 3: Commit the audit**

```bash
git add docs/handoff/2026-04-17-a3-part2-hooks-audit.md
git commit -m "docs: hooks consolidation audit for A3 Part 2 Task 14"
```

---

#### Task 14: Make `.memd/hooks/` canonical + write sync script

**Files:**
- Create: `scripts/sync-integration-hooks.sh` — copies `.memd/hooks/*.sh` → `integrations/hooks/`, rewriting project-local paths to portable forms (e.g. `$HOME/Documents/projects/memd` → `$HOME/<memd-repo>` with a comment saying the user edits at install time)
- Modify: files under `integrations/hooks/` — replaced by sync-script output
- Modify: `integrations/hooks/README.md` — add deprecation notice: "These scripts are generated from `.memd/hooks/` via `scripts/sync-integration-hooks.sh`. Edit `.memd/hooks/` and re-run the sync."

- [ ] **Step 1: Write the sync script using the rewrite rules from Task 13's audit**

```bash
#!/usr/bin/env bash
set -euo pipefail
SRC=".memd/hooks"
DST="integrations/hooks"
[ -d "$SRC" ] || { echo "no $SRC"; exit 1; }
[ -d "$DST" ] || mkdir -p "$DST"

# Rewrite rules — keep in sync with docs/handoff/2026-04-17-a3-part2-hooks-audit.md.
# Add one -e per rule; every project-local path surfaced by Task 13's diff must be
# covered here or the sync produces drift and Task 16's CI check fails.
REWRITE=(
  -e 's|\$HOME/Documents/projects/memd|$HOME/<memd-repo>|g'
  # extend here from the audit's rewrite-rule summary
)

for f in "$SRC"/*.sh "$SRC"/*.ps1 "$SRC"/README.md "$SRC"/install.sh "$SRC"/install.ps1; do
  [ -f "$f" ] || continue
  name="$(basename "$f")"
  sed "${REWRITE[@]}" "$f" > "$DST/$name"
  chmod +x "$DST/$name" 2>/dev/null || true
done
echo "synced: $DST"
```

- [ ] **Step 2: Run it**

```bash
bash scripts/sync-integration-hooks.sh
git diff integrations/hooks/
```

Inspect diff — any file under `integrations/hooks/` that differs only by path rewrite is now aligned.

- [ ] **Step 3: Add deprecation notice to `integrations/hooks/README.md`** at the top:

```markdown
> **Generated file.** These scripts are synced from `.memd/hooks/` by `scripts/sync-integration-hooks.sh`.
> Edit the source at `.memd/hooks/` and re-run the script. Do not edit files in this directory directly.
```

- [ ] **Step 4: Commit**

```bash
git add scripts/sync-integration-hooks.sh integrations/hooks/
git commit -m "refactor(hooks): .memd/hooks/ is canonical; integrations/hooks/ generated via sync script"
```

---

#### Task 15: `.claude/hooks/memd-bootstrap.sh` becomes a shim

**Files:**
- Modify: `.claude/hooks/memd-bootstrap.sh`

- [ ] **Step 1: Replace body with one-line shim that execs the canonical script**

```bash
#!/usr/bin/env bash
# Shim — canonical implementation lives in .memd/hooks/memd-bootstrap.sh
exec bash "${MEMD_REPO:-$HOME/Documents/projects/memd}/.memd/hooks/memd-bootstrap.sh" "$@"
```

- [ ] **Step 2: Confirm existing `UserPromptSubmit` hook in `~/.claude/settings.json` still points at `$HOME/.claude/hooks/memd-bootstrap.sh` — yes, shim preserves compatibility.**

- [ ] **Step 3: Commit**

```bash
git add .claude/hooks/memd-bootstrap.sh
git commit -m "refactor(hooks): .claude/hooks/memd-bootstrap.sh is now a shim to .memd/hooks/"
```

---

#### Task 16: Add CI check — sync script is idempotent

**Files:**
- Modify: `scripts/doc-lint.sh` or create `scripts/hooks-lint.sh`

- [ ] **Step 1: Add a check that running the sync script produces no diff**

```bash
#!/usr/bin/env bash
set -euo pipefail
bash scripts/sync-integration-hooks.sh
if ! git diff --quiet -- integrations/hooks/; then
  echo "integrations/hooks/ is out of sync with .memd/hooks/. Run scripts/sync-integration-hooks.sh and commit." >&2
  git diff --stat -- integrations/hooks/ >&2
  exit 1
fi
```

- [ ] **Step 2: Run it — expect exit 0 on a clean tree**

- [ ] **Step 3: Commit**

```bash
git add scripts/hooks-lint.sh
git commit -m "ci: hooks-lint.sh fails if integrations/hooks/ drifts from .memd/hooks/"
```

---

### Phase 5 — Part 2 Gate + Handoff

#### Task 17: Gate verification — run the full Part 2 gate

**Files:**
- None — pure verification + handoff packet.

- [ ] **Step 1: Run full test suite**

```
cargo test -p memd-core -p memd-client -p memd-server
```

Expected: green.

- [ ] **Step 2: Run contract verify on populated bundle**

```
./target/debug/memd contract verify --output .memd
```

Expected: exit 0. If the current bundle's `config.json` enables enforcement, confirm the PreToolUse hook script exists and the wiring doc shows the install snippet.

- [ ] **Step 3: Live smoke (operator)**

1. Open a Claude Code session with the PreToolUse hook wired.
2. Trigger an Edit on a path listed in `.memd/state/session-*/sealed/*.json`.
3. Confirm Claude Code displays the deny message (policy=block) or systemMessage (policy=warn).
4. Read the file in the session.
5. Re-attempt Edit — confirm allow.

(If the settings-watcher caveat from Part 1 still bites — run `/hooks` once or restart.)

- [ ] **Step 4: Update ROADMAP_STATE + checkpoint**

```
memd checkpoint --auto-commit \
  --roadmap-set current_phase=A3 \
  --roadmap-set phase_status=part2_green \
  --content "A3 Part 2 green — enforcement, contract@0.2, preference replay, hooks consolidation"
```

- [ ] **Step 5: Write handoff packet**

`docs/handoff/2026-04-17-a3-part2-green-next-part3.md` — include:
- Commit manifest (SHA per task)
- Evidence blocks (test output, contract verify output, smoke screenshot description)
- Known gaps (any tasks deferred mid-plan)
- Part 3 candidate list from Part 1 handoff, verbatim — this is the user-call.

- [ ] **Step 6: Final commit**

```bash
git add docs/handoff/2026-04-17-a3-part2-green-next-part3.md
git commit -m "docs: A3 Part 2 handoff — enforcement live, next Part 3 user-call"
```

---

## Non-goals (explicit)

- No server schema changes. Preference replay fix is surface-side (wake assembly).
- No new retrieval intents. Preferences use the existing `RetrievalIntent::Preference`.
- No Part 3 work. Active continuity, cross-session diff summaries, auto-prime on wake, and workspace-level recall aggregation are explicitly deferred — user decides after Part 2 ships.
- No overhaul of `~/.claude/settings.json` structure. Existing hooks (promptbook, read-once, no-coauthor) stay untouched; Part 2 adds one PreToolUse entry alongside them.
- No removal of `integrations/hooks/` directory. Just re-derive it from the canonical source.
- No default policy flip to `block`. Default `warn` keeps rollout safe; the flip is a Part 3 question once we have telemetry.

## Risks & rollback

- **Risk:** false-positive denies block real work when the sealed ledger contains paths the user doesn't need to re-Read (e.g. paths long since deleted). **Mitigation:** default policy is `warn`, not `block`; user can toggle `continuity.enforcement=off` for a single session. Task 11 adds a `## Continuity Gate` block users see before hitting the deny.
- **Risk:** PreToolUse hook latency slows every Edit/Write. **Mitigation:** the hook is a single CLI call with `|| true`; fast path returns Allow as soon as the target path is not in the sealed set (cheap string comparison on ≤ N paths).
- **Risk:** contract evidence collection becomes brittle in fresh bundles (no server, no state). **Mitigation:** evidence fields default to unknown/true rather than false — verifier only fails on *observed* violations.
- **Risk:** hooks consolidation breaks an existing install that points at `integrations/hooks/`. **Mitigation:** `.claude/hooks/memd-bootstrap.sh` shim preserves the old entry point; `integrations/hooks/` still exists, just generated.
- **Risk:** preference-replay fix diverges between wake-assembly code path and resume/checkpoint code path. **Mitigation:** test is cross-process (store via remember, recall via fresh wake) — covers the user-visible surface.
- **Rollback:** every deliverable is flag-gated or removable. Enforcement gate: `continuity.enforcement=off` in config.json. Contract growth: old contract.json with 1 guarantee still parses (serde tolerates missing fields via Default). Preference block: conditional on non-empty lookup result. Consolidation: revert the sync-script commit.

## Pass Gate (must all be true before handoff)

**Part 2 is the enforcement gate.** It must actually refuse-to-proceed on sealed paths under a real Claude Code session, not just in cargo tests.

- [ ] `cargo test -p memd-core -p memd-client` green, including `continuity_enforcement_tests` + `continuity_foundation_tests` (no regression)
- [ ] `memd contract verify --output .memd` exit 0 on a populated bundle
- [ ] `.memd/contract.json` version `0.2.0` with 4 guarantees
- [ ] Live smoke: Edit on a sealed-ledger path in a new session surfaces deny (policy=block) or warn (policy=warn); same Edit after an in-session Read is allowed
- [ ] `memd remember --kind preference` → cold boot → wake contains the preference text (cross-process test green)
- [ ] `scripts/hooks-lint.sh` exit 0 (integrations/hooks/ == .memd/hooks/ after sync)
- [ ] `.claude/hooks/memd-bootstrap.sh` is a shim; repo still works end-to-end (existing UserPromptSubmit hook fires)
- [ ] Handoff packet written pointing at Part 3; candidate list preserved verbatim from Part 1 handoff; user-call noted explicitly

**Not gated by Part 2** (explicitly Part 3, user-call):
- Active continuity (diff summaries, auto-prime on wake)
- Workspace-level recall aggregation
- Default policy flip from `warn` to `block`
