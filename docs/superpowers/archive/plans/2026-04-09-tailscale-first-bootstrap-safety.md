# Tailscale-First Bootstrap Safety Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make shared Tailscale authority the primary `memd` hive truth plane, make localhost a permission-gated read-only fallback, and surface degraded-authority warnings during bootstrap and every fallback-backed session start.

**Architecture:** Extend the bundle runtime contract to carry authority policy and live fallback state, then enforce that policy at three boundaries: bootstrap/setup/init, command base URL resolution, and shared write paths. Keep the implementation inside the existing `memd-client` runtime file instead of introducing a new subsystem so the warning state, config writing, and CLI rendering all read the same contract.

**Tech Stack:** Rust, clap, serde/serde_json, existing `memd-client` CLI/runtime code, existing tokio async command paths, existing bundle config/env writers and CLI tests.

---

## File Map

- Modify: `crates/memd-client/src/main.rs`
  - Add authority policy and fallback runtime fields to bundle/runtime structs.
  - Add bootstrap-time fallback gating helpers.
  - Update bundle config/env writing to persist policy and current authority mode.
  - Update status/awareness/coordination rendering to surface degraded-authority state.
  - Block shared mutations when fallback mode is active.
  - Add regression tests.
- Reference: `docs/superpowers/specs/2026-04-09-tailscale-first-bootstrap-safety-design.md`
  - Source of truth for requirements while implementing.
- Already added ops file, no code change required in this plan:
  - `deploy/systemd/memd-local.service`

## Task 1: Add Authority Policy To Bundle Runtime

**Files:**
- Modify: `crates/memd-client/src/main.rs:33186-33340`
- Modify: `crates/memd-client/src/main.rs:14379-14595`
- Test: `crates/memd-client/src/main.rs` (existing CLI/unit test module near `43348+`)

- [ ] **Step 1: Write the failing tests for persisted authority policy**

Add tests near the existing bundle config/env tests:

```rust
#[test]
fn write_init_bundle_persists_shared_authority_policy_defaults() {
    let dir = std::env::temp_dir().join(format!("memd-authority-policy-{}", uuid::Uuid::new_v4()));
    let args = InitArgs {
        agent: "codex".to_string(),
        output: Some(dir.clone()),
        base_url: SHARED_MEMD_BASE_URL.to_string(),
        route: "auto".to_string(),
        intent: "current_task".to_string(),
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        workspace: None,
        visibility: None,
        session: Some("session-a".to_string()),
        tab_id: None,
        global: false,
        force: true,
        seed_existing: false,
        project_root: None,
        hive_system: None,
        hive_role: None,
        capability: Vec::new(),
        hive_group: Vec::new(),
        hive_group_goal: None,
        authority: None,
        rag_url: None,
    };

    write_init_bundle(&args).expect("write bundle");
    let config = fs::read_to_string(dir.join("config.json")).expect("read config");
    assert!(config.contains(r#""authority_policy""#));
    assert!(config.contains(r#""shared_primary": true"#));
    assert!(config.contains(r#""localhost_fallback_policy": "deny""#));
}

#[test]
fn write_init_bundle_exports_authority_env_lines() {
    let dir = std::env::temp_dir().join(format!("memd-authority-env-{}", uuid::Uuid::new_v4()));
    let args = InitArgs {
        agent: "codex".to_string(),
        output: Some(dir.clone()),
        base_url: SHARED_MEMD_BASE_URL.to_string(),
        route: "auto".to_string(),
        intent: "current_task".to_string(),
        project: Some("demo".to_string()),
        namespace: Some("main".to_string()),
        workspace: None,
        visibility: None,
        session: Some("session-a".to_string()),
        tab_id: None,
        global: false,
        force: true,
        seed_existing: false,
        project_root: None,
        hive_system: None,
        hive_role: None,
        capability: Vec::new(),
        hive_group: Vec::new(),
        hive_group_goal: None,
        authority: None,
        rag_url: None,
    };

    write_init_bundle(&args).expect("write bundle");
    let env = fs::read_to_string(dir.join("env")).expect("read env");
    let env_ps1 = fs::read_to_string(dir.join("env.ps1")).expect("read env.ps1");
    assert!(env.contains("MEMD_AUTHORITY_MODE=shared"));
    assert!(env.contains("MEMD_LOCALHOST_FALLBACK_POLICY=deny"));
    assert!(env_ps1.contains("$env:MEMD_AUTHORITY_MODE = \"shared\""));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
cargo test -p memd-client --bin memd write_init_bundle_persists_shared_authority_policy_defaults -- --nocapture
cargo test -p memd-client --bin memd write_init_bundle_exports_authority_env_lines -- --nocapture
```

Expected: FAIL because `BundleRuntimeConfig` / bundle writers do not yet include authority policy fields.

- [ ] **Step 3: Add the minimal runtime policy types and fields**

Extend the runtime contract near `BundleRuntimeConfig`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum LocalhostFallbackPolicy {
    Deny,
    AllowReadOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct BundleAuthorityPolicy {
    #[serde(default = "default_true")]
    shared_primary: bool,
    #[serde(default)]
    localhost_fallback_policy: LocalhostFallbackPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct BundleAuthorityState {
    #[serde(default)]
    mode: String,
    #[serde(default)]
    degraded: bool,
    #[serde(default)]
    shared_base_url: Option<String>,
    #[serde(default)]
    fallback_base_url: Option<String>,
    #[serde(default)]
    warning_acknowledged_at: Option<DateTime<Utc>>,
    #[serde(default)]
    expires_at: Option<DateTime<Utc>>,
}

struct BundleRuntimeConfig {
    // existing fields...
    #[serde(default)]
    authority_policy: BundleAuthorityPolicy,
    #[serde(default)]
    authority_state: BundleAuthorityState,
}
```

Add defaults used by config writing:

```rust
fn default_true() -> bool { true }

impl Default for LocalhostFallbackPolicy {
    fn default() -> Self {
        Self::Deny
    }
}
```

- [ ] **Step 4: Update bundle config/env writers**

In `write_init_bundle`, include policy/state in `BundleConfig` and exported env:

```rust
authority_policy: BundleAuthorityPolicy {
    shared_primary: true,
    localhost_fallback_policy: LocalhostFallbackPolicy::Deny,
},
authority_state: BundleAuthorityState {
    mode: "shared".to_string(),
    degraded: false,
    shared_base_url: Some(args.base_url.clone()),
    fallback_base_url: None,
    warning_acknowledged_at: None,
    expires_at: None,
},
```

Add env lines:

```rust
"MEMD_AUTHORITY_MODE={}\nMEMD_LOCALHOST_FALLBACK_POLICY={}\n"
```

And PowerShell lines:

```rust
"$env:MEMD_AUTHORITY_MODE = \"{}\"\n$env:MEMD_LOCALHOST_FALLBACK_POLICY = \"{}\"\n"
```

- [ ] **Step 5: Re-run tests**

Run:

```bash
cargo test -p memd-client --bin memd write_init_bundle_persists_shared_authority_policy_defaults -- --nocapture
cargo test -p memd-client --bin memd write_init_bundle_exports_authority_env_lines -- --nocapture
```

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add crates/memd-client/src/main.rs
git commit -m "feat: persist memd authority policy state"
```

## Task 2: Enforce Tailscale-First Bootstrap And Session Warnings

**Files:**
- Modify: `crates/memd-client/src/main.rs:9532-9565`
- Modify: `crates/memd-client/src/main.rs:14947-15538`
- Modify: `crates/memd-client/src/main.rs:28115-28544`
- Test: `crates/memd-client/src/main.rs` (existing command/runtime tests near `44135+`, `44787+`)

- [ ] **Step 1: Write failing tests for fallback warnings**

Add tests:

```rust
#[tokio::test]
async fn read_bundle_status_warns_when_localhost_fallback_is_active() {
    let dir = std::env::temp_dir().join(format!("memd-status-fallback-{}", uuid::Uuid::new_v4()));
    write_test_bundle_config(&dir, SHARED_MEMD_BASE_URL);
    let mut runtime = read_bundle_runtime_config_raw(&dir).expect("runtime").expect("config");
    runtime.authority_policy.localhost_fallback_policy = LocalhostFallbackPolicy::AllowReadOnly;
    runtime.authority_state.mode = "localhost_read_only".to_string();
    runtime.authority_state.degraded = true;
    runtime.authority_state.shared_base_url = Some(SHARED_MEMD_BASE_URL.to_string());
    runtime.authority_state.fallback_base_url = Some("http://127.0.0.1:8787".to_string());
    fs::write(dir.join("config.json"), serde_json::to_string_pretty(&runtime).unwrap()).unwrap();

    let status = read_bundle_status(&dir, SHARED_MEMD_BASE_URL).await.expect("status");
    let summary = render_bundle_status_summary(&status);
    assert!(summary.contains("authority=localhost_read_only"));
    assert!(summary.contains("degraded=yes"));
    assert!(summary.contains("warning=shared authority unavailable"));
}

#[tokio::test]
async fn resolve_bundle_command_base_url_does_not_silently_fallback_to_localhost() {
    let dir = std::env::temp_dir().join(format!("memd-resolve-fallback-{}", uuid::Uuid::new_v4()));
    write_test_bundle_config(&dir, SHARED_MEMD_BASE_URL);
    let runtime = read_bundle_runtime_config_raw(&dir).expect("runtime").expect("config");
    let resolved = resolve_bundle_command_base_url(
        SHARED_MEMD_BASE_URL,
        runtime.base_url.as_deref(),
    );
    assert_eq!(resolved, SHARED_MEMD_BASE_URL);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
cargo test -p memd-client --bin memd read_bundle_status_warns_when_localhost_fallback_is_active -- --nocapture
cargo test -p memd-client --bin memd resolve_bundle_command_base_url_does_not_silently_fallback_to_localhost -- --nocapture
```

Expected: FAIL because status does not yet surface authority warnings and bootstrap policy state is not enforced.

- [ ] **Step 3: Add bootstrap and session-start authority helpers**

Add focused helpers near the base URL / bootstrap helpers:

```rust
fn runtime_prefers_shared_authority(runtime: Option<&BundleRuntimeConfig>) -> bool {
    runtime
        .map(|value| value.authority_policy.shared_primary)
        .unwrap_or(true)
}

fn runtime_allows_localhost_read_only(runtime: Option<&BundleRuntimeConfig>) -> bool {
    runtime
        .map(|value| value.authority_policy.localhost_fallback_policy == LocalhostFallbackPolicy::AllowReadOnly)
        .unwrap_or(false)
}

fn authority_warning_lines(runtime: Option<&BundleRuntimeConfig>) -> Vec<String> {
    let Some(runtime) = runtime else { return Vec::new(); };
    if runtime.authority_state.mode != "localhost_read_only" {
        return Vec::new();
    }
    vec![
        "shared authority unavailable".to_string(),
        "localhost fallback active".to_string(),
        "prompt-injection and split-brain risk increased".to_string(),
        "coordination writes blocked".to_string(),
    ]
}
```

- [ ] **Step 4: Thread warning state into bootstrap and status**

Update bootstrap/init/hive entry points to preserve shared base URL and authority mode:

```rust
runtime.authority_state.shared_base_url = Some(SHARED_MEMD_BASE_URL.to_string());
runtime.authority_state.mode = "shared".to_string();
runtime.authority_state.degraded = false;
```

Update `read_bundle_status` / `render_bundle_status_summary` to surface:

```rust
status["authority"] = json!(runtime.authority_state.mode);
status["degraded"] = json!(runtime.authority_state.degraded);
status["shared_base_url"] = json!(runtime.authority_state.shared_base_url);
status["fallback_base_url"] = json!(runtime.authority_state.fallback_base_url);
status["authority_warning"] = json!(authority_warning_lines(runtime.as_ref()));
```

Update bootstrap/session-start output to print the warning lines whenever mode is `localhost_read_only`.

- [ ] **Step 5: Re-run tests**

Run:

```bash
cargo test -p memd-client --bin memd read_bundle_status_warns_when_localhost_fallback_is_active -- --nocapture
cargo test -p memd-client --bin memd resolve_bundle_command_base_url_does_not_silently_fallback_to_localhost -- --nocapture
```

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add crates/memd-client/src/main.rs
git commit -m "feat: warn on degraded memd authority bootstrap"
```

## Task 3: Gate Localhost Read-Only Fallback And Block Shared Writes

**Files:**
- Modify: `crates/memd-client/src/main.rs:17099-17129`
- Modify: `crates/memd-client/src/main.rs:17223-18304`
- Modify: `crates/memd-client/src/main.rs:18767-19280`
- Test: `crates/memd-client/src/main.rs` (near existing offline coordination tests at `44680+`)

- [ ] **Step 1: Write failing tests for localhost read-only enforcement**

Add tests:

```rust
#[tokio::test]
async fn run_coordination_command_blocks_queen_actions_in_localhost_read_only_mode() {
    let dir = std::env::temp_dir().join(format!("memd-localhost-ro-{}", uuid::Uuid::new_v4()));
    write_test_bundle_config(&dir, "http://127.0.0.1:8787");
    let mut runtime = read_bundle_runtime_config_raw(&dir).expect("runtime").expect("config");
    runtime.authority_policy.localhost_fallback_policy = LocalhostFallbackPolicy::AllowReadOnly;
    runtime.authority_state.mode = "localhost_read_only".to_string();
    runtime.authority_state.degraded = true;
    runtime.authority_state.shared_base_url = Some(SHARED_MEMD_BASE_URL.to_string());
    runtime.authority_state.fallback_base_url = Some("http://127.0.0.1:8787".to_string());
    fs::write(dir.join("config.json"), serde_json::to_string_pretty(&runtime).unwrap()).unwrap();

    let err = run_coordination_command(
        &CoordinationArgs {
            output: dir.clone(),
            view: None,
            changes_only: false,
            watch: false,
            interval_secs: 30,
            recover_session: None,
            retire_session: None,
            to_session: None,
            deny_session: Some("bee-b".to_string()),
            reroute_session: None,
            handoff_scope: None,
            summary: false,
        },
        "http://127.0.0.1:8787",
    )
    .await
    .expect_err("queen action should be blocked");

    assert!(err.to_string().contains("localhost read-only fallback active"));
}

#[tokio::test]
async fn read_only_coordination_still_works_in_localhost_read_only_mode() {
    let dir = std::env::temp_dir().join(format!("memd-localhost-ro-read-{}", uuid::Uuid::new_v4()));
    write_test_bundle_config(&dir, "http://127.0.0.1:8787");
    let mut runtime = read_bundle_runtime_config_raw(&dir).expect("runtime").expect("config");
    runtime.authority_policy.localhost_fallback_policy = LocalhostFallbackPolicy::AllowReadOnly;
    runtime.authority_state.mode = "localhost_read_only".to_string();
    runtime.authority_state.degraded = true;
    runtime.authority_state.shared_base_url = Some(SHARED_MEMD_BASE_URL.to_string());
    runtime.authority_state.fallback_base_url = Some("http://127.0.0.1:8787".to_string());
    fs::write(dir.join("config.json"), serde_json::to_string_pretty(&runtime).unwrap()).unwrap();

    let response = run_coordination_command(
        &CoordinationArgs {
            output: dir.clone(),
            view: Some("overview".to_string()),
            changes_only: false,
            watch: false,
            interval_secs: 30,
            recover_session: None,
            retire_session: None,
            to_session: None,
            deny_session: None,
            reroute_session: None,
            handoff_scope: None,
            summary: false,
        },
        "http://127.0.0.1:8787",
    )
    .await
    .expect("read-only coordination");

    assert!(response.receipts.is_empty());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
cargo test -p memd-client --bin memd run_coordination_command_blocks_queen_actions_in_localhost_read_only_mode -- --nocapture
cargo test -p memd-client --bin memd read_only_coordination_still_works_in_localhost_read_only_mode -- --nocapture
```

Expected: FAIL because localhost authority mode is not yet enforced as a separate trust state.

- [ ] **Step 3: Add explicit authority-mode enforcement helper**

Add a helper near the coordination / session command logic:

```rust
fn ensure_shared_authority_write_allowed(
    runtime: Option<&BundleRuntimeConfig>,
    operation: &str,
) -> anyhow::Result<()> {
    let Some(runtime) = runtime else { return Ok(()); };
    if runtime.authority_state.mode == "localhost_read_only" {
        anyhow::bail!(
            "localhost read-only fallback active; {} requires trusted shared authority",
            operation
        );
    }
    Ok(())
}
```

- [ ] **Step 4: Apply enforcement to shared write paths**

Call the helper before write-capable operations in:

```rust
run_coordination_command(...)
run_messages_command(...)
run_tasks_command(...)
run_session_command(...)
publish_bundle_heartbeat(...)
```

Pattern:

```rust
ensure_shared_authority_write_allowed(runtime.as_ref(), "queen actions")?;
```

Use read-only pass-through for:

```rust
status
awareness
coordination --summary
resume / memory inspection
```

- [ ] **Step 5: Re-run tests**

Run:

```bash
cargo test -p memd-client --bin memd run_coordination_command_blocks_queen_actions_in_localhost_read_only_mode -- --nocapture
cargo test -p memd-client --bin memd read_only_coordination_still_works_in_localhost_read_only_mode -- --nocapture
cargo test -p memd-client --bin memd run_coordination_command_fails_fast_for_mutations_when_backend_unreachable -- --nocapture
```

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add crates/memd-client/src/main.rs
git commit -m "feat: enforce localhost read-only memd fallback"
```

## Task 4: Surface Receipts And Fallback State In Awareness/Coordination

**Files:**
- Modify: `crates/memd-client/src/main.rs:29292-29740`
- Modify: `crates/memd-client/src/main.rs:19280-19580`
- Test: `crates/memd-client/src/main.rs` (existing rendering tests around `39565+`, `45109+`)

- [ ] **Step 1: Write failing tests for visible authority state**

Add tests:

```rust
#[test]
fn render_project_awareness_summary_marks_localhost_fallback_sessions() {
    let response = ProjectAwarenessResponse {
        root: "server:http://127.0.0.1:8787".to_string(),
        current_bundle: "/tmp/demo/.memd".to_string(),
        diagnostics: Vec::new(),
        hidden_remote_dead: 0,
        hidden_superseded_stale: 0,
        entries: vec![ProjectAwarenessEntry {
            bundle_root: "/tmp/demo/.memd".to_string(),
            project: "demo".to_string(),
            namespace: Some("main".to_string()),
            session: Some("session-a".to_string()),
            presence: "active".to_string(),
            truth: "current".to_string(),
            authority: Some("participant".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            focus: Some("none".to_string()),
            pressure: Some("none".to_string()),
            // existing fields...
            ..test_awareness_entry_defaults()
        }],
    };
    let summary = render_project_awareness_summary(&response);
    assert!(summary.contains("authority_mode=localhost_read_only"));
}

#[test]
fn render_coordination_summary_shows_degraded_authority_banner() {
    let response = CoordinationResponse {
        bundle_root: ".memd".to_string(),
        current_session: "queen-a".to_string(),
        inbox: HiveCoordinationInboxResponse {
            messages: vec![],
            owned_tasks: vec![],
            help_tasks: vec![],
            review_tasks: vec![],
        },
        recovery: CoordinationRecoverySummary::default(),
        lane_fault: None,
        lane_receipts: vec![],
        policy_conflicts: vec![],
        suggestions: vec![],
        boundary_recommendations: vec![],
        receipts: vec![],
    };
    let summary = render_coordination_summary(&response, Some("overview"));
    assert!(summary.contains("authority degraded"));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
cargo test -p memd-client --bin memd render_project_awareness_summary_marks_localhost_fallback_sessions -- --nocapture
cargo test -p memd-client --bin memd render_coordination_summary_shows_degraded_authority_banner -- --nocapture
```

Expected: FAIL because summaries do not yet display explicit authority-mode state.

- [ ] **Step 3: Add receipt and rendering fields**

Thread authority state into summary builders:

```rust
lines.push(format!(
    "authority mode={} degraded={} shared={} fallback={}",
    runtime.authority_state.mode,
    if runtime.authority_state.degraded { "yes" } else { "no" },
    runtime.authority_state.shared_base_url.as_deref().unwrap_or("none"),
    runtime.authority_state.fallback_base_url.as_deref().unwrap_or("none"),
));
```

Emit or append receipts for:

```rust
authority_fallback_activated
authority_fallback_blocked_write
authority_restored_shared
```

- [ ] **Step 4: Re-run tests**

Run:

```bash
cargo test -p memd-client --bin memd render_project_awareness_summary_marks_localhost_fallback_sessions -- --nocapture
cargo test -p memd-client --bin memd render_coordination_summary_shows_degraded_authority_banner -- --nocapture
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/memd-client/src/main.rs
git commit -m "feat: surface degraded authority state in memd summaries"
```

## Task 5: Final Verification And Cleanup

**Files:**
- Modify: `crates/memd-client/src/main.rs` only if test fixes are needed
- Verify: `docs/superpowers/specs/2026-04-09-tailscale-first-bootstrap-safety-design.md`

- [ ] **Step 1: Run targeted regression suite**

Run:

```bash
cargo test -p memd-client --bin memd write_init_bundle_persists_shared_authority_policy_defaults -- --nocapture
cargo test -p memd-client --bin memd write_init_bundle_exports_authority_env_lines -- --nocapture
cargo test -p memd-client --bin memd read_bundle_status_warns_when_localhost_fallback_is_active -- --nocapture
cargo test -p memd-client --bin memd resolve_bundle_command_base_url_does_not_silently_fallback_to_localhost -- --nocapture
cargo test -p memd-client --bin memd run_coordination_command_blocks_queen_actions_in_localhost_read_only_mode -- --nocapture
cargo test -p memd-client --bin memd read_only_coordination_still_works_in_localhost_read_only_mode -- --nocapture
```

Expected: all PASS

- [ ] **Step 2: Run broader memd command safety checks**

Run:

```bash
cargo test -p memd-client --bin memd run_coordination_command_records_queen_decisions -- --nocapture
cargo test -p memd-client --bin memd run_coordination_command_falls_back_to_local_truth_when_backend_unreachable -- --nocapture
cargo test -p memd-client --bin memd run_coordination_command_fails_fast_for_mutations_when_backend_unreachable -- --nocapture
cargo test -p memd-client --bin memd status_summary_surfaces_lane_reroute_context -- --nocapture
```

Expected: all PASS

- [ ] **Step 3: Run formatting and a final suite checkpoint**

Run:

```bash
cargo fmt --all
cargo test -p memd-client --bin memd
```

Expected:

- `cargo fmt --all` exits `0`
- `cargo test -p memd-client --bin memd` passes, or if the known long-tail stall remains, document the exact stalled tests and keep the targeted authority-policy suite as the release gate for this slice

- [ ] **Step 4: Commit final polish**

```bash
git add crates/memd-client/src/main.rs
git commit -m "test: cover tailscale-first memd authority policy"
```

## Spec Coverage Check

- Shared authority first: Task 2
- Explicit localhost permission and read-only mode: Tasks 1 and 3
- Bootstrap and session-start warnings: Task 2
- Deny-precedence policy model: Task 1
- Blocked shared writes during fallback: Task 3
- Receipts and visibility: Task 4

## Placeholder Scan

- No `TODO`
- No `TBD`
- Every task has exact file paths, commands, and code stubs

## Type Consistency Check

- Policy naming is consistent:
  - `BundleAuthorityPolicy`
  - `BundleAuthorityState`
  - `LocalhostFallbackPolicy`
  - `localhost_read_only`
  - `ensure_shared_authority_write_allowed`

