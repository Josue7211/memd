# Opt-In Project Hive Control Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add opt-in project hive controls so a repo can explicitly enable or disable persistent hive membership, while keeping live session join, safe cross-project pairing, and hive repair separate.

**Architecture:** Store the project-hive toggle in bundle runtime/config state, then thread that toggle through startup launcher generation, join/publish flows, and awareness/status reporting. Keep the current live-session commands intact, add a new project-level control surface, and use the existing shared hive base URL as the transport for any enabled project hive.

**Tech Stack:** Rust (`clap`, `serde`, `tokio`), existing memd bundle config/runtime helpers, existing heartbeat/awareness server APIs, existing launcher generation in `crates/memd-client/src/main.rs`.

---

### Task 1: Add project hive state to bundle config/runtime

**Files:**
- Modify: `crates/memd-schema/src/lib.rs`
- Modify: `crates/memd-client/src/main.rs`
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn hive_project_state_round_trips_through_bundle_runtime_config() {
    let runtime = BundleRuntimeConfig {
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        agent: Some("codex".to_string()),
        session: Some("codex-a".to_string()),
        tab_id: Some("tab-alpha".to_string()),
        hive_system: Some("codex".to_string()),
        hive_role: Some("agent".to_string()),
        capabilities: vec!["claim".to_string()],
        hive_groups: vec!["openclaw-stack".to_string()],
        hive_group_goal: Some("coordinate the project hive".to_string()),
        authority: Some("participant".to_string()),
        base_url: Some("http://100.104.154.24:8787".to_string()),
        route: Some("auto".to_string()),
        intent: Some("current_task".to_string()),
        workspace: Some("shared".to_string()),
        visibility: Some("workspace".to_string()),
        heartbeat_model: Some("gpt-4.1-mini".to_string()),
        auto_short_term_capture: true,
        hive_project_enabled: true,
        hive_project_anchor: Some("project:demo".to_string()),
        hive_project_joined_at: Some(Utc::now()),
    };
    let json = serde_json::to_string(&runtime).unwrap();
    assert!(json.contains("\"hive_project_enabled\":true"));
    assert!(json.contains("\"hive_project_anchor\":\"project:demo\""));
}
```

- [ ] **Step 2: Run the failing test**

Run:
```bash
cargo test -p memd-client hive_project_state_round_trips_through_bundle_runtime_config -- --nocapture
```

Expected: fail until `BundleRuntimeConfig` and the bundle config structs accept the new hive-project fields.

- [ ] **Step 3: Write the minimal implementation**

Add the new fields to the bundle runtime/config structs in `crates/memd-schema/src/lib.rs`, then propagate them through the bundle read/write helpers in `crates/memd-client/src/main.rs`:

```rust
pub hive_project_enabled: bool,
pub hive_project_anchor: Option<String>,
pub hive_project_joined_at: Option<DateTime<Utc>>,
```

Make sure the bundle writer preserves these values, and the runtime reader defaults them sensibly when absent.

- [ ] **Step 4: Run the test to verify it passes**

Run:
```bash
cargo test -p memd-client hive_project_state_round_trips_through_bundle_runtime_config -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/memd-schema/src/lib.rs crates/memd-client/src/main.rs
git commit -m "feat: add project hive runtime state"
```

### Task 2: Add `memd hive-project` enable/disable/status command

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn hive_project_enable_disable_and_status_are_exposed() {
    use clap::CommandFactory;

    let help = Cli::command().render_long_help().to_string();
    assert!(help.contains("hive-project"));
    assert!(help.contains("--enable"));
    assert!(help.contains("--disable"));
    assert!(help.contains("--status"));
}
```

- [ ] **Step 2: Run the failing test**

Run:
```bash
cargo test -p memd-client hive_project_enable_disable_and_status_are_exposed -- --nocapture
```

Expected: fail until the new subcommand and flags exist.

- [ ] **Step 3: Write the minimal implementation**

Add a new `HiveProject` subcommand in the `Commands` enum and implement:

```rust
match cli.command {
    Commands::HiveProject(args) => {
        let response = run_hive_project_command(&args).await?;
        if args.summary {
            println!("{}", render_hive_project_summary(&response));
        } else {
            print_json(&response)?;
        }
    }
    _ => { /* existing commands */ }
}
```

The command should:

- `--enable`: set `hive_project_enabled=true`, set `hive_project_anchor`, publish the current heartbeat, and regenerate launchers
- `--disable`: set `hive_project_enabled=false`, leave other bundle state intact
- `--status`: report enabled/disabled plus current joined state

- [ ] **Step 4: Run the test to verify it passes**

Run:
```bash
cargo test -p memd-client hive_project_enable_disable_and_status_are_exposed -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/memd-client/src/main.rs
git commit -m "feat: add project hive command"
```

### Task 3: Wire project-hive enablement into startup, join, and repair flows

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[tokio::test]
async fn enabled_project_hive_sets_shared_base_url_in_launchers() {
    let dir = std::env::temp_dir().join(format!("memd-hive-project-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("config.json"),
        r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "current_task"
}
"#,
    )
    .unwrap();
    run_hive_project_command(&HiveProjectArgs {
        output: dir.clone(),
        enable: true,
        disable: false,
        status: false,
        summary: false,
    })
    .await
    .unwrap();
    let shell = render_agent_shell_profile(&dir, Some("codex"));
    let ps1 = render_agent_ps1_profile(&dir, Some("codex"));
    let attach = render_attach_snippet("bash", &dir).unwrap();
    assert!(shell.contains("http://100.104.154.24:8787"));
    assert!(ps1.contains("http://100.104.154.24:8787"));
    assert!(attach.contains("http://100.104.154.24:8787"));
}
```

- [ ] **Step 2: Run the failing test**

Run:
```bash
cargo test -p memd-client enabled_project_hive_sets_shared_base_url_in_launchers -- --nocapture
```

Expected: fail until the launcher renderers and join logic respect the project-hive toggle.

- [ ] **Step 3: Write the minimal implementation**

Update these helpers:

- `render_agent_shell_profile(...)`
- `render_agent_ps1_profile(...)`
- `render_attach_snippet(...)`
- `run_hive_command(...)`
- `run_hive_join_command(...)`
- `read_bundle_status(...)`
- `read_project_awareness(...)`
- `resolve_hive_join_base_url(...)`

Behavior to implement:

- if project hive is enabled, startup launchers force the shared hive base URL when the bundle is stale or loopback
- `memd hive` still publishes the live session heartbeat
- `memd hive-fix` can repair stale bundles
- `memd status` and `memd awareness` continue to self-register the current live bundle

- [ ] **Step 4: Run the test to verify it passes**

Run:
```bash
cargo test -p memd-client enabled_project_hive_sets_shared_base_url_in_launchers -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/memd-client/src/main.rs
git commit -m "feat: wire project hive into startup and repair flows"
```

### Task 4: Document the project-hive model and keep cross-project safe-linking explicit

**Files:**
- Modify: `docs/setup.md`
- Modify: `docs/api.md`
- Modify: `README.md`
- Modify: `integrations/mcp-hive/README.md`
- Modify: `/home/josue/.codex/skills/memd/SKILL.md`
- Modify: `/home/josue/.codex/skills/memd-hive/SKILL.md`
- Modify: `/home/josue/.codex/skills/memd-hive-group-link/SKILL.md`
- Modify: `/home/josue/.codex/skills/memd-hive-link/SKILL.md`

- [ ] **Step 1: Write the failing test**

```bash
rg -n "project hive|hive-project|hive-fix|hive-group-link|hive-link" \
  docs/setup.md docs/api.md README.md integrations/mcp-hive/README.md
```

Expected: the old copy still conflates live hive join with persistent project anchoring.

- [ ] **Step 2: Run the failing check**

Run:
```bash
rg -n "project hive|hive-project|hive-fix|hive-group-link|hive-link" \
  docs/setup.md docs/api.md README.md integrations/mcp-hive/README.md
```

Expected: identify the places that still need copy updates.

- [ ] **Step 3: Write the minimal implementation**

Update the visible copy so it says:

- `hive-project` is opt-in per project
- `hive` joins/publishes the live session
- `hive-group-link` is the persistent project anchor
- `hive-link` is the manual safe link between different projects
- `hive-fix` repairs drifted bundles onto the shared hive URL

Also update the memd skill docs so the command surfaces match the new model.

- [ ] **Step 4: Run the check to verify it passes**

Run:
```bash
rg -n "project hive|hive-project|hive-fix|hive-group-link|hive-link" \
  docs/setup.md docs/api.md README.md integrations/mcp-hive/README.md
```

Expected: only the intended terminology remains, with no contradictory copy.

- [ ] **Step 5: Commit**

```bash
git add docs/setup.md docs/api.md README.md integrations/mcp-hive/README.md \
  /home/josue/.codex/skills/memd/SKILL.md \
  /home/josue/.codex/skills/memd-hive/SKILL.md \
  /home/josue/.codex/skills/memd-hive-group-link/SKILL.md \
  /home/josue/.codex/skills/memd-hive-link/SKILL.md
git commit -m "docs: clarify project hive controls"
```

### Task 5: Verify the whole project hive flow end to end

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[tokio::test]
async fn hive_project_enable_then_hive_join_then_hive_fix_all_work_together() {
    let dir = std::env::temp_dir().join(format!("memd-hive-project-e2e-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("config.json"),
        r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "current_task"
}
"#,
    )
    .unwrap();
    run_hive_project_command(&HiveProjectArgs {
        output: dir.clone(),
        enable: true,
        disable: false,
        status: false,
        summary: false,
    })
    .await
    .unwrap();
    run_hive_join_command(&HiveJoinArgs {
        output: dir.clone(),
        base_url: "http://100.104.154.24:8787".to_string(),
        all_active: false,
        all_local: false,
        publish_heartbeat: true,
        summary: false,
    })
    .await
    .unwrap();
    let status = read_bundle_status(&dir, "http://100.104.154.24:8787")
        .await
        .unwrap();
    assert_eq!(
        status.get("defaults").and_then(|v| v.get("hive_project_enabled")),
        Some(&serde_json::Value::Bool(true))
    );
}
```

- [ ] **Step 2: Run the failing test**

Run:
```bash
cargo test -p memd-client hive_project_enable_then_hive_join_then_hive_fix_all_work_together -- --nocapture
```

Expected: fail until the whole control surface is wired together.

- [ ] **Step 3: Write the minimal implementation**

Make the project-hive commands and repair commands agree on the same bundle state and the same shared hive URL, and ensure the current bundle keeps self-registering in awareness/status after enablement.

- [ ] **Step 4: Run the test to verify it passes**

Run:
```bash
cargo test -p memd-client hive_project_enable_then_hive_join_then_hive_fix_all_work_together -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/memd-client/src/main.rs
git commit -m "feat: complete project hive control flow"
```
