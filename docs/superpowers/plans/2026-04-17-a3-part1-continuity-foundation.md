# A3 Part 1 — memd Continuity Foundation (State Continuity) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make memd's continuity contract real so no continuation session re-Reads files the prior session touched, preferences survive compaction + session restart, and wake/checkpoint/resume guarantees are machine-verifiable.

**Architecture:** Add a **file-interaction ledger** populated by a new memd PostToolUse hook (Read/Edit/Write/NotebookEdit), persisted under `.memd/state/session-<id>/file_interactions.json`, flushed + sealed by the existing precompact hook, surfaced in the wake packet as a `## Files Touched` block, and primed into the next session via a new `memd prime-reads` CLI command. Wire the lifecycle self-test (store → recall → expire → verify) as a cron-style probe. Prove cross-session preference replay with an integration test. Publish a machine-readable `.memd/contract.json` the cross-harness validator consumes.

**Tech Stack:** Rust (crates: `memd-core` for ledger data model + lifecycle probe, `memd-client` for CLI subcommands + wake assembly + contract verifier, `memd-server` only if lifecycle probe needs a server roundtrip), Bash hooks (`.memd/hooks/*.sh`), JSON state files, clap CLI, `anyhow`/`serde`/`serde_json`, existing test harness.

**Scope (A3 Part 1 only — 5 deliverables):**
1. File-interaction ledger + PostToolUse hook + wake `## Files Touched` block (D1)
2. `memd prime-reads` CLI command (D2)
3. Working-memory lifecycle self-test (D3)
4. Cross-session preference replay test (D4)
5. Live memory contract.json + `memd contract verify` (D5)

Part 2 (enforcement, hooks consolidation, drift repair, strict mode) and Part 3 (codebase organization) are separate plans, to be written after Part 1 ships.

---

## File Structure

**Create (new):**
- `crates/memd-core/src/file_ledger.rs` — `FileInteractionLedger`, `FileInteractionEntry`, append/flush/load/summarize functions (pure data model, zero I/O in tests via in-memory round-trips)
- `crates/memd-core/src/lifecycle_probe.rs` — `run_lifecycle_self_test()` that stores → recalls → expires → verifies a probe record, emits green/red JSON
- `crates/memd-core/src/contract.rs` — `MemdContract` struct + `verify_contract()` (guarantees schema + checker)
- `crates/memd-client/src/cli/cli_prime_reads_runtime.rs` — `PrimeReads` subcommand runner
- `crates/memd-client/src/cli/cli_contract_runtime.rs` — `Contract Verify` subcommand runner
- `.memd/hooks/memd-file-interaction.sh` — PostToolUse hook body (reads Claude Code hook JSON from stdin, calls `memd hook file-interaction --stdin`)
- `.memd/hooks/memd-lifecycle-probe.sh` — cron-style probe runner (calls `memd diagnostics lifecycle-probe`)
- `.memd/contract.json` — generated artifact (checked into repo as the reference contract)
- `crates/memd-client/src/main_tests/continuity_foundation_tests/mod.rs` — integration tests for compaction-mid-edit, prime-reads, preference replay, contract verify

**Modify:**
- `crates/memd-core/src/lib.rs` — `pub mod file_ledger; pub mod lifecycle_probe; pub mod contract;`
- `crates/memd-client/src/cli/args.rs` — add `PrimeReads(PrimeReadsArgs)` to `Commands`, `HookCommand::FileInteraction`, `DiagnosticsCommand::LifecycleProbe`, `Contract(ContractArgs)` with `ContractCommand::Verify`
- `crates/memd-client/src/cli/commands.rs` — no structural change; parser helpers only if needed
- `crates/memd-client/src/cli/cli_hook_runtime.rs` — dispatch the new `file-interaction` subcommand to a handler that appends to the ledger
- `crates/memd-client/src/cli/mod.rs` — `pub(crate) mod cli_prime_reads_runtime; pub(crate) mod cli_contract_runtime;` and `Commands::PrimeReads(_) => cli_prime_reads_runtime::run(...)` wiring
- `crates/memd-client/src/runtime/resume/wakeup.rs` — insert `## Files Touched` block between Atlas and Continuity (at ~line 206, before `let continuity = snapshot.continuity_capsule();`)
- `crates/memd-client/src/runtime/resume/mod.rs` — thread the ledger read into `ResumeSnapshot` assembly
- `.memd/hooks/memd-precompact-save.sh` — add "seal ledger" step: copy `current-session` ledger into timestamped session dir before allowing compaction
- `~/.claude/settings.json` — wire the new PostToolUse hook (matcher `Read|Edit|Write|NotebookEdit`) and install the PreCompact hook that's currently unwired
- `Cargo.toml` workspace members — none (all crates already listed)

**No changes:**
- `memd-server` — Part 1 avoids server changes; lifecycle probe runs fully via client against existing endpoints
- Other hook directories (`integrations/hooks/`, `.claude/hooks/`) — left alone; Part 2 consolidates them

---

## Task Decomposition

### Task 1: Ledger data model in memd-core (TDD)

**Files:**
- Create: `crates/memd-core/src/file_ledger.rs`
- Test: inline `#[cfg(test)] mod tests` in same file
- Modify: `crates/memd-core/src/lib.rs` (add `pub mod file_ledger;`)

- [ ] **Step 1: Write failing test for FileInteractionEntry round-trip**

```rust
#[test]
fn entry_round_trips_through_json() {
    let entry = FileInteractionEntry {
        path: "crates/memd-core/src/lib.rs".into(),
        op: FileOp::Read,
        count: 3,
        last_ts_ms: 1_700_000_000_000,
    };
    let json = serde_json::to_string(&entry).unwrap();
    let parsed: FileInteractionEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, entry);
}
```

- [ ] **Step 2: Run test — expect FAIL (module missing)**

```
cargo test -p memd-core file_ledger::tests::entry_round_trips_through_json
```
Expected: compile error "unresolved module `file_ledger`".

- [ ] **Step 3: Implement minimal types**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileOp { Read, Edit, Write }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileInteractionEntry {
    pub path: String,
    pub op: FileOp,
    pub count: u32,
    pub last_ts_ms: i64,
}
```

- [ ] **Step 4: Register module + rerun — expect PASS**

Edit `crates/memd-core/src/lib.rs`: add `pub mod file_ledger;` near the top.
Run: `cargo test -p memd-core file_ledger`
Expected: 1 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/memd-core/src/file_ledger.rs crates/memd-core/src/lib.rs
git commit -m "feat(core): add FileInteractionEntry types for A3 continuity ledger"
```

---

### Task 2: Ledger container — append / upsert / load / save

**Files:**
- Modify: `crates/memd-core/src/file_ledger.rs`

- [ ] **Step 1: Write failing test for upsert semantics**

```rust
#[test]
fn upsert_increments_existing_entry_and_updates_ts() {
    let mut ledger = FileInteractionLedger::new("session-x");
    ledger.record("a.rs", FileOp::Read, 1_000);
    ledger.record("a.rs", FileOp::Read, 2_000);
    ledger.record("a.rs", FileOp::Edit, 3_000);
    let read = ledger.find("a.rs", FileOp::Read).unwrap();
    assert_eq!(read.count, 2);
    assert_eq!(read.last_ts_ms, 2_000);
    let edit = ledger.find("a.rs", FileOp::Edit).unwrap();
    assert_eq!(edit.count, 1);
    assert_eq!(edit.last_ts_ms, 3_000);
}
```

- [ ] **Step 2: Run test — expect FAIL**

- [ ] **Step 3: Implement `FileInteractionLedger`**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInteractionLedger {
    pub session_id: String,
    pub entries: Vec<FileInteractionEntry>,
}

impl FileInteractionLedger {
    pub fn new(session_id: impl Into<String>) -> Self {
        Self { session_id: session_id.into(), entries: Vec::new() }
    }
    pub fn record(&mut self, path: impl AsRef<str>, op: FileOp, ts_ms: i64) {
        let path = path.as_ref();
        if let Some(e) = self.entries.iter_mut().find(|e| e.path == path && e.op == op) {
            e.count += 1;
            e.last_ts_ms = ts_ms;
            return;
        }
        self.entries.push(FileInteractionEntry {
            path: path.to_string(),
            op,
            count: 1,
            last_ts_ms: ts_ms,
        });
    }
    pub fn find(&self, path: &str, op: FileOp) -> Option<&FileInteractionEntry> {
        self.entries.iter().find(|e| e.path == path && e.op == op)
    }
    pub fn distinct_paths(&self) -> Vec<String> {
        let mut v: Vec<String> = self.entries.iter().map(|e| e.path.clone()).collect();
        v.sort(); v.dedup(); v
    }
}
```

- [ ] **Step 4: Rerun — expect PASS**

- [ ] **Step 5: Add failing test for `load_from_path` / `save_to_path` round-trip via tempfile**

```rust
#[test]
fn ledger_round_trips_through_disk() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("file_interactions.json");
    let mut ledger = FileInteractionLedger::new("session-1");
    ledger.record("x.rs", FileOp::Read, 10);
    ledger.save_to_path(&path).unwrap();
    let loaded = FileInteractionLedger::load_from_path(&path).unwrap();
    assert_eq!(loaded.session_id, "session-1");
    assert_eq!(loaded.entries.len(), 1);
}
```

Ensure `memd-core/Cargo.toml` has `[dev-dependencies] tempfile = "3"`. If not, add it.

- [ ] **Step 6: Implement `load_from_path` / `save_to_path`**

```rust
use std::{fs, io, path::Path};

impl FileInteractionLedger {
    pub fn save_to_path(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
        let bytes = serde_json::to_vec_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(path, bytes)
    }
    pub fn load_from_path(path: &Path) -> io::Result<Self> {
        let bytes = fs::read(path)?;
        serde_json::from_slice(&bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}
```

- [ ] **Step 7: Rerun — expect PASS. Commit.**

```bash
git add crates/memd-core/src/file_ledger.rs crates/memd-core/Cargo.toml
git commit -m "feat(core): FileInteractionLedger append/save/load with upsert semantics"
```

---

### Task 3: `memd hook file-interaction` CLI handler (stdin → ledger)

**Files:**
- Modify: `crates/memd-client/src/cli/args.rs` (extend `HookCommand`)
- Modify: `crates/memd-client/src/cli/cli_hook_runtime.rs` (dispatch)
- Create: `crates/memd-client/src/cli/cli_hook_file_interaction.rs`
- Test: `crates/memd-client/src/main_tests/continuity_foundation_tests/mod.rs`

- [ ] **Step 1: Write failing integration test**

`continuity_foundation_tests/mod.rs`:

```rust
#[tokio::test]
async fn hook_file_interaction_appends_ledger_entry() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path();
    let hook_input = serde_json::json!({
        "session_id": "sess-abc",
        "tool_name": "Read",
        "tool_input": {"file_path": "/tmp/foo.rs"}
    });
    run_memd_cli(&[
        "hook", "file-interaction",
        "--output", output.to_str().unwrap(),
        "--stdin",
    ], &hook_input.to_string()).await.unwrap();

    let ledger_path = output
        .join("state/session-sess-abc/file_interactions.json");
    assert!(ledger_path.exists(), "ledger file should be created");
    let ledger: FileInteractionLedger =
        serde_json::from_slice(&std::fs::read(&ledger_path).unwrap()).unwrap();
    assert_eq!(ledger.entries.len(), 1);
    assert_eq!(ledger.entries[0].path, "/tmp/foo.rs");
    assert_eq!(ledger.entries[0].op, FileOp::Read);
}
```

Register module in `crates/memd-client/src/main_tests/mod.rs` (pattern: other `*_tests/mod.rs` entries).

- [ ] **Step 2: Run test — expect FAIL (subcommand doesn't exist)**

- [ ] **Step 3: Add `HookCommand::FileInteraction(FileInteractionArgs)` in `args.rs`**

Find the `HookCommand` enum (search for `pub enum HookCommand`), add a variant with `--stdin`, `--output`, `--session-id` (optional, falls back to JSON payload).

- [ ] **Step 4: Create `cli_hook_file_interaction.rs` handler**

```rust
use std::io::Read;
use memd_core::file_ledger::{FileInteractionLedger, FileOp};

pub(crate) async fn run(args: &FileInteractionArgs) -> anyhow::Result<()> {
    let mut payload = String::new();
    std::io::stdin().read_to_string(&mut payload)?;
    let v: serde_json::Value = serde_json::from_str(&payload)?;
    let session_id = args.session_id.clone()
        .or_else(|| v.get("session_id").and_then(|s| s.as_str().map(String::from)))
        .unwrap_or_else(|| "unknown".into());
    let tool = v.get("tool_name").and_then(|s| s.as_str()).unwrap_or("");
    let op = match tool {
        "Read" => FileOp::Read,
        "Edit" | "NotebookEdit" => FileOp::Edit,
        "Write" => FileOp::Write,
        _ => return Ok(()), // ignore non-file tools
    };
    let path = v.pointer("/tool_input/file_path")
        .and_then(|s| s.as_str())
        .unwrap_or("");
    if path.is_empty() { return Ok(()); }
    let now_ms = chrono::Utc::now().timestamp_millis();
    let ledger_dir = args.output.join("state").join(format!("session-{session_id}"));
    let ledger_path = ledger_dir.join("file_interactions.json");
    let mut ledger = if ledger_path.exists() {
        FileInteractionLedger::load_from_path(&ledger_path).unwrap_or_else(|_| FileInteractionLedger::new(&session_id))
    } else {
        FileInteractionLedger::new(&session_id)
    };
    ledger.record(path, op, now_ms);
    ledger.save_to_path(&ledger_path)?;
    Ok(())
}
```

- [ ] **Step 5: Wire dispatch in `cli_hook_runtime.rs`**

Add match arm for `HookCommand::FileInteraction(args) => cli_hook_file_interaction::run(args).await`.

- [ ] **Step 6: Rerun test — expect PASS**

```
cargo test -p memd-client continuity_foundation_tests::hook_file_interaction_appends_ledger_entry
```

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat(cli): memd hook file-interaction appends session ledger"
```

---

### Task 4: PostToolUse hook script + settings.json wiring (A3-D1)

**Files:**
- Create: `.memd/hooks/memd-file-interaction.sh`
- Modify: `~/.claude/settings.json` (add PostToolUse entry)

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

BASE_URL="${MEMD_BASE_URL:-http://100.104.154.24:8787}"
OUTPUT="${MEMD_BUNDLE_ROOT:-.memd}"

# Claude Code pipes hook JSON on stdin. Forward to memd.
memd --base-url "$BASE_URL" hook file-interaction --output "$OUTPUT" --stdin || true
```

Make executable: `chmod +x .memd/hooks/memd-file-interaction.sh`

- [ ] **Step 2: Add PostToolUse matcher to `~/.claude/settings.json`**

Use the **update-config** skill (deferred). Add under existing `"hooks"` → `"PostToolUse"`:

```json
{
  "matcher": "Read|Edit|Write|NotebookEdit",
  "hooks": [
    { "type": "command", "command": "bash \"$HOME/Documents/projects/memd/.memd/hooks/memd-file-interaction.sh\"", "timeout": 10 }
  ]
}
```

Also ensure existing `PreCompact` wiring exists; if missing, add:

```json
"PreCompact": [
  { "matcher": "", "hooks": [
    { "type": "command", "command": "bash \"$HOME/Documents/projects/memd/.memd/hooks/memd-precompact-save.sh\"", "timeout": 15 }
  ]}
]
```

- [ ] **Step 3: Verify hook fires end-to-end (manual smoke test)**

Trigger an Edit from Claude Code on a scratch file, then:

```bash
ls .memd/state/session-*/file_interactions.json
cat $(ls -t .memd/state/session-*/file_interactions.json | head -1)
```

Expected: JSON with at least one entry for the scratch file.

- [ ] **Step 4: Commit**

```bash
git add .memd/hooks/memd-file-interaction.sh
git commit -m "feat(hooks): PostToolUse memd-file-interaction.sh writes session ledger"
```

(Do NOT commit the global `~/.claude/settings.json` — that's user config. Capture the JSON snippet in `docs/policy/claude-hooks.md` as the install target.)

---

### Task 5: Precompact seal step (ledger finalize)

**Files:**
- Modify: `.memd/hooks/memd-precompact-save.sh`

- [ ] **Step 1: Write a failing test for seal semantics**

Add to `continuity_foundation_tests/mod.rs`:

```rust
#[test]
fn precompact_seal_copies_current_session_ledger_to_sealed_dir() {
    // Arrange: create .memd/state/session-sess-1/file_interactions.json
    // Act: invoke a Rust helper `seal_session_ledger(session_id, output)`
    // Assert: copy exists at .memd/state/session-sess-1/sealed/<ts>.json
}
```

- [ ] **Step 2: Implement `seal_session_ledger` in `memd-core`**

```rust
pub fn seal_session_ledger(session_id: &str, output: &Path) -> io::Result<PathBuf> {
    let src = output.join("state").join(format!("session-{session_id}")).join("file_interactions.json");
    if !src.exists() { return Err(io::Error::new(io::ErrorKind::NotFound, "no ledger")); }
    let dst_dir = src.parent().unwrap().join("sealed");
    fs::create_dir_all(&dst_dir)?;
    let ts = chrono::Utc::now().timestamp_millis();
    let dst = dst_dir.join(format!("{ts}.json"));
    fs::copy(&src, &dst)?;
    Ok(dst)
}
```

- [ ] **Step 3: Add `memd hook seal-ledger --session-id <id> --output <dir>` subcommand**

Mirror Task 3's plumbing.

- [ ] **Step 4: Extend precompact hook to call seal-ledger before emitting the block-decision JSON**

```bash
SESSION_ID="$(printf '%s' "$INPUT" | python3 -c 'import json, sys; print(json.load(sys.stdin).get("session_id", "unknown"))' 2>/dev/null || printf unknown)"
memd hook seal-ledger --session-id "$SESSION_ID" --output "${MEMD_BUNDLE_ROOT:-.memd}" || true
```

- [ ] **Step 5: Rerun test — expect PASS. Commit.**

```bash
git add -A
git commit -m "feat: seal session ledger on precompact"
```

---

### Task 6: Wake packet `## Files Touched` section (A3-D1b)

**Files:**
- Modify: `crates/memd-client/src/runtime/resume/wakeup.rs` (insert block after Atlas, before Continuity)
- Modify: `crates/memd-client/src/runtime/resume/mod.rs` (surface ledger on `ResumeSnapshot`)
- Test: `continuity_foundation_tests/mod.rs`

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn wake_packet_surfaces_files_touched_block_from_prior_session() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path();
    seed_prior_session_ledger(output, "sess-prev", &[
        ("crates/memd-core/src/lib.rs", FileOp::Read),
        ("crates/memd-core/src/lib.rs", FileOp::Edit),
        ("ROADMAP.md", FileOp::Read),
    ]);
    let wake = render_wake_for_test(output);
    assert!(wake.contains("## Files Touched"));
    assert!(wake.contains("crates/memd-core/src/lib.rs"));
    assert!(wake.contains("ROADMAP.md"));
}
```

- [ ] **Step 2: Run test — expect FAIL**

- [ ] **Step 3: In `runtime/resume/mod.rs`, add `files_touched: Vec<String>` to `ResumeSnapshot`; populate from most-recent sealed ledger (or current ledger if no sealed)**

```rust
pub fn collect_files_touched(output: &Path) -> Vec<String> {
    let state = output.join("state");
    let Ok(rd) = fs::read_dir(&state) else { return Vec::new(); };
    let mut latest: Option<(std::time::SystemTime, PathBuf)> = None;
    for entry in rd.flatten() {
        if !entry.file_name().to_string_lossy().starts_with("session-") { continue; }
        let sealed = entry.path().join("sealed");
        if let Ok(sd) = fs::read_dir(&sealed) {
            for s in sd.flatten() {
                let meta = s.metadata().ok();
                let mt = meta.and_then(|m| m.modified().ok());
                if let Some(mt) = mt {
                    if latest.as_ref().map_or(true, |(l, _)| mt > *l) {
                        latest = Some((mt, s.path()));
                    }
                }
            }
        }
    }
    if let Some((_, path)) = latest {
        if let Ok(l) = FileInteractionLedger::load_from_path(&path) {
            return l.distinct_paths();
        }
    }
    Vec::new()
}
```

- [ ] **Step 4: In `runtime/resume/wakeup.rs` after the Atlas block (~line 204), add:**

```rust
if !snapshot.files_touched.is_empty() && !claude_strict {
    prefix.push_str("## Files Touched\n\n");
    prefix.push_str("_Prior session Read/Edit/Write. Bulk-Read before first Edit to avoid re-Read errors after compaction._\n\n");
    let limit = if verbose { 20 } else { 10 };
    for p in snapshot.files_touched.iter().take(limit) {
        prefix.push_str(&format!("- {p}\n"));
    }
    if snapshot.files_touched.len() > limit {
        prefix.push_str(&format!("- + {} more via `memd prime-reads`\n", snapshot.files_touched.len() - limit));
    }
    prefix.push('\n');
}
```

- [ ] **Step 5: Rerun test — expect PASS. Also rerun existing `wake_fallback_*` and `codex_pack_refreshes_*` tests to confirm no regressions.**

```
cargo test -p memd-client wake
cargo test -p memd-client continuity_foundation_tests
```

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat(wake): surface ## Files Touched block from prior session ledger"
```

---

### Task 7: `memd prime-reads` CLI command (A3-D2)

**Files:**
- Modify: `crates/memd-client/src/cli/args.rs` (add `PrimeReads(PrimeReadsArgs)`)
- Create: `crates/memd-client/src/cli/cli_prime_reads_runtime.rs`
- Modify: `crates/memd-client/src/cli/mod.rs` (register + dispatch)
- Test: `continuity_foundation_tests/mod.rs`

- [ ] **Step 1: Write failing test**

```rust
#[tokio::test]
async fn prime_reads_emits_newline_paths_from_most_recent_sealed_ledger() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path();
    seed_prior_session_ledger(output, "sess-prev", &[
        ("a.rs", FileOp::Read), ("b.rs", FileOp::Edit),
    ]);
    let out = run_memd_cli_capture_stdout(&[
        "prime-reads", "--output", output.to_str().unwrap(),
    ]).await.unwrap();
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines.contains(&"a.rs"));
    assert!(lines.contains(&"b.rs"));
}
```

- [ ] **Step 2: Run — expect FAIL (no subcommand)**

- [ ] **Step 3: Define `PrimeReadsArgs`**

```rust
#[derive(Debug, Clone, Args)]
pub(crate) struct PrimeReadsArgs {
    #[arg(long, default_value = ".memd")]
    pub(crate) output: PathBuf,
    #[arg(long)]
    pub(crate) since_session: Option<String>,
}
```

Add `PrimeReads(PrimeReadsArgs)` to `Commands`.

- [ ] **Step 4: Implement runtime**

```rust
use memd_core::file_ledger::FileInteractionLedger;

pub(crate) fn run(args: &PrimeReadsArgs) -> anyhow::Result<()> {
    let paths = if let Some(session) = &args.since_session {
        let p = args.output.join("state").join(format!("session-{session}")).join("file_interactions.json");
        FileInteractionLedger::load_from_path(&p).map(|l| l.distinct_paths()).unwrap_or_default()
    } else {
        crate::runtime::resume::collect_files_touched(&args.output)
    };
    for p in paths { println!("{p}"); }
    Ok(())
}
```

- [ ] **Step 5: Wire dispatch in `cli/mod.rs`**

- [ ] **Step 6: Rerun test — expect PASS. Commit.**

```bash
git add -A
git commit -m "feat(cli): memd prime-reads emits paths from prior session ledger"
```

---

### Task 8: Compaction-mid-edit acceptance test (the real A3 gate)

**Files:**
- Test: `continuity_foundation_tests/mod.rs`

- [ ] **Step 1: Write scenario test that simulates: session A edits 5 files → precompact seals → session B boots cold → wake includes ## Files Touched → `prime-reads` lists those 5 files**

```rust
#[tokio::test]
async fn compaction_mid_edit_flow_lists_prior_session_files() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path();
    // Simulate session A: 5 hook invocations
    for file in ["a.rs", "b.rs", "c.rs", "d.rs", "e.rs"] {
        run_memd_cli(&[
            "hook", "file-interaction",
            "--output", output.to_str().unwrap(),
            "--stdin",
        ], &serde_json::json!({
            "session_id": "sess-A",
            "tool_name": "Edit",
            "tool_input": {"file_path": file}
        }).to_string()).await.unwrap();
    }
    // Seal (precompact)
    run_memd_cli(&[
        "hook", "seal-ledger",
        "--session-id", "sess-A",
        "--output", output.to_str().unwrap(),
    ], "").await.unwrap();
    // Session B: prime-reads
    let listed = run_memd_cli_capture_stdout(&[
        "prime-reads", "--output", output.to_str().unwrap(),
    ]).await.unwrap();
    for f in ["a.rs", "b.rs", "c.rs", "d.rs", "e.rs"] {
        assert!(listed.contains(f), "prime-reads missing {f}: {listed}");
    }
}
```

- [ ] **Step 2: Run — expect PASS (if previous tasks are correct). If FAIL, stop and debug root cause.**

- [ ] **Step 3: Commit as gate test**

```bash
git add -A
git commit -m "test: compaction-mid-edit acceptance test for A3 continuity"
```

---

### Task 9: Lifecycle self-test (A3-D3)

**Files:**
- Create: `crates/memd-core/src/lifecycle_probe.rs`
- Modify: `crates/memd-core/src/lib.rs` (`pub mod lifecycle_probe;`)
- Modify: `crates/memd-client/src/cli/args.rs` (add `DiagnosticsCommand::LifecycleProbe`)
- Create: `crates/memd-client/src/cli/cli_lifecycle_probe_runtime.rs`
- Create: `.memd/hooks/memd-lifecycle-probe.sh`
- Test: `continuity_foundation_tests/mod.rs`

- [ ] **Step 1: Write failing test for probe return shape**

```rust
#[tokio::test]
async fn lifecycle_probe_reports_green_on_healthy_server() {
    let report: LifecycleProbeReport = run_probe_against_test_server().await;
    assert_eq!(report.status, "green");
    assert!(report.steps.iter().all(|s| s.ok));
}
```

- [ ] **Step 2: Implement probe: store a probe record → immediate recall → expire → confirm expired → emit JSON**

The probe record carries a tag `lifecycle_probe` and a UUID so concurrent probes don't clobber each other.

- [ ] **Step 3: Wire `memd diagnostics lifecycle-probe` subcommand → runs probe, prints JSON, exits 0 on green / 1 on red**

- [ ] **Step 4: Write hook script**

```bash
#!/usr/bin/env bash
set -euo pipefail
memd diagnostics lifecycle-probe --output "${MEMD_BUNDLE_ROOT:-.memd}"
```

- [ ] **Step 5: Document cron wiring in `docs/policy/lifecycle-probe.md` — the probe is invoked from wake, checkpoint, and a cron entry (user installs manually; automation is Part 2)**

- [ ] **Step 6: Rerun tests. Commit.**

```bash
git add -A
git commit -m "feat: working-memory lifecycle self-test probe (A3-D3)"
```

---

### Task 10: Cross-session preference replay test (A3-D4)

**Files:**
- Test: `continuity_foundation_tests/mod.rs`
- Possibly modify: whatever code layer fails the test (leave for discovery)

- [ ] **Step 1: Write failing integration test**

```rust
#[tokio::test]
async fn preference_stored_in_session_a_is_retrieved_in_session_b_after_cold_boot() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path();
    // Session A stores a preference
    run_memd_cli(&[
        "remember",
        "--kind", "preference",
        "--content", "voice-mode=caveman-ultra",
        "--output", output.to_str().unwrap(),
    ], "").await.unwrap();
    // Simulate cold boot: wipe in-memory caches, re-init bundle
    simulate_cold_boot(output);
    // Session B wakes, looks up
    let lookup = run_memd_cli_capture_stdout(&[
        "lookup", "--query", "voice-mode",
        "--output", output.to_str().unwrap(),
    ]).await.unwrap();
    assert!(lookup.contains("caveman-ultra"), "preference not retrieved: {lookup}");
}
```

- [ ] **Step 2: Run — diagnose whether it passes or fails. If it fails, root-cause before fixing (don't add hacks).**

If it passes on first run, the contract is already holding; note that in the commit and skip to Task 11. If it fails, file a sub-task to fix the specific layer and keep the test red until the real fix lands.

- [ ] **Step 3: Commit the (possibly red, then green) test**

```bash
git add -A
git commit -m "test: cross-session preference replay (A3-D4)"
```

---

### Task 11: Live memory contract.json + `memd contract verify` (A3-D5)

**Files:**
- Create: `crates/memd-core/src/contract.rs`
- Modify: `crates/memd-core/src/lib.rs`
- Modify: `crates/memd-client/src/cli/args.rs` (add `Contract(ContractArgs)` + `ContractCommand::Verify`)
- Create: `crates/memd-client/src/cli/cli_contract_runtime.rs`
- Create: `.memd/contract.json` (generated artifact; checked in)
- Test: `continuity_foundation_tests/mod.rs`

- [ ] **Step 1: Define contract shape**

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct MemdContract {
    pub version: String,              // semver
    pub guarantees: ContractGuarantees,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContractGuarantees {
    pub wake: WakeGuarantees,
    pub checkpoint: CheckpointGuarantees,
    pub resume: ResumeGuarantees,
    pub ledger: LedgerGuarantees,
}
// Each sub-struct lists named booleans / enum values, e.g.:
pub struct WakeGuarantees {
    pub emits_durable_truth: bool,
    pub emits_continuity: bool,
    pub emits_files_touched_when_prior_session_exists: bool,
    pub char_budget_enforced: bool,
}
```

- [ ] **Step 2: Write default contract to `.memd/contract.json`**

- [ ] **Step 3: Implement `verify_contract(snapshot: &ResumeSnapshot, contract: &MemdContract) -> Vec<ContractViolation>`**

- [ ] **Step 4: Write failing test**

```rust
#[test]
fn contract_verify_flags_missing_files_touched_when_prior_session_exists() {
    // Arrange snapshot where files_touched is empty BUT sealed ledger exists
    // Act: verify
    // Assert: violation returned
}
```

- [ ] **Step 5: Wire `memd contract verify --output <dir>` subcommand → prints violations, exits 1 if any**

- [ ] **Step 6: Rerun tests. Commit.**

```bash
git add -A
git commit -m "feat: live memory contract.json + memd contract verify (A3-D5)"
```

---

### Task 12: Gate verification — run the full A3 Part 1 gate

**Files:**
- None — pure verification + handoff update

- [ ] **Step 1: Run full test suite**

```
cargo test -p memd-core -p memd-client -p memd-server
```

Expected: green.

- [ ] **Step 2: Run compaction-mid-edit gate check manually**

Start fresh Claude Code session, Edit 10 files, trigger compaction, verify continuation session's wake packet includes `## Files Touched` and subsequent Edits succeed with zero `File has not been read yet` errors.

- [ ] **Step 3: Run lifecycle probe**

```
memd diagnostics lifecycle-probe --output .memd
```

Expected: `{"status":"green", ...}` exit 0.

- [ ] **Step 4: Run contract verify**

```
memd contract verify --output .memd
```

Expected: exit 0.

- [ ] **Step 5: Update ROADMAP_STATE**

```
memd checkpoint --auto-commit \
  --roadmap-set current_phase=A3 \
  --roadmap-set phase_status=part1_green \
  --content "A3 Part 1 green — continuity foundation live"
```

- [ ] **Step 6: Write handoff packet**

`docs/handoff/2026-04-17-a3-part1-green-next-part2.md` — includes gate evidence, contract artifact path, and the open question Part 2 must answer (validator design, hooks consolidation strategy, strict mode).

- [ ] **Step 7: Final commit + roadmap update**

```bash
git add -A
git commit -m "docs: A3 Part 1 handoff — continuity foundation green, next Part 2"
```

---

## Non-goals (explicit)

- No server schema changes (Part 2 may need some for validator endpoints)
- No hook consolidation under one tree (Part 2)
- No backlog frontmatter migration (Part 3)
- No retrieval quality work (B3)
- No score changes on any bench (F3)
- No new UI surfaces

## Risks & rollback

- **Risk:** PostToolUse hook latency slows every tool call. **Mitigation:** hook body is a single fast CLI call with `|| true`; if latency regresses, fall back to the precompact-only path (lossier but cheap).
- **Risk:** Ledger grows unbounded on long sessions. **Mitigation:** upsert semantics cap entries at one per (path, op); total size bounded by distinct file count.
- **Risk:** Cross-session preference replay test reveals a deeper bug than Part 1 can fix. **Mitigation:** file the failing test as a Part 2 blocker; don't land a hack.
- **Rollback:** every deliverable is flag-gated or removable — ledger behind `memd.continuity.ledger=true`, PostToolUse hook removable from settings, wake block conditional on non-empty `files_touched`. Revert by dropping the commits.

## Pass Gate (must all be true before Part 2 starts)

- [ ] `cargo test -p memd-core -p memd-client` green, including `continuity_foundation_tests`
- [ ] Compaction-mid-edit smoke: real Claude Code session edits N files, compacts, continuation shows `## Files Touched`, 10 consecutive Edits succeed with zero re-Read errors
- [ ] `memd prime-reads` emits non-empty list after a session with ≥1 file interaction
- [ ] `memd diagnostics lifecycle-probe` returns green
- [ ] `memd contract verify` returns exit 0 on a populated bundle
- [ ] Cross-session preference replay test green
- [ ] `.memd/contract.json` committed and matches code
- [ ] Handoff packet written pointing at Part 2
