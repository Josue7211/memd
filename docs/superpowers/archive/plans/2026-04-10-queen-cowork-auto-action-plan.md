# Queen Cowork Auto-Action Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make queen surface explicit cowork actions and optionally auto-dispatch them through the live `hive cowork` protocol when policy allows.

**Architecture:** Queen stays a coordinator, not a second transport. It reads the existing coordination graph, turns `near` / `blocked` / `cowork_active` into cowork-specific action cards, and uses one opt-in flag to dispatch packets through the existing cowork command path. The packet format, receipts, and follow surfaces stay the single source of truth.

**Tech Stack:** Rust, `clap`, `memd-client` CLI, existing coordination helpers, existing mock runtime test harness in `crates/memd-client/src/main.rs`.

---

### Task 1: Add queen cowork card fields and render the exact cowork command

**Files:**
- Modify: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write the failing test**

Add a parser test and a render test alongside the existing queen tests:

```rust
#[test]
fn cli_parses_hive_queen_cowork_auto_send_flag() {
    let cli = Cli::try_parse_from([
        "memd",
        "hive",
        "queen",
        "--output",
        ".memd",
        "--cowork-auto-send",
        "--summary",
    ])
    .expect("hive queen command should parse");

    match cli.command {
        Commands::Hive(args) => match args.command {
            Some(HiveSubcommand::Queen(queen)) => {
                assert!(queen.cowork_auto_send);
                assert_eq!(queen.output, PathBuf::from(".memd"));
            }
            other => panic!("expected hive queen subcommand, got {other:?}"),
        },
        other => panic!("expected hive command, got {other:?}"),
    }
}

#[test]
fn render_hive_queen_summary_surfaces_cowork_commands() {
    let response = HiveQueenResponse {
        queen_session: "session-queen".to_string(),
        suggested_actions: vec!["request_cowork Peer needs live overlap support".to_string()],
        action_cards: vec![HiveQueenActionCard {
            action: "request_cowork".to_string(),
            priority: "high".to_string(),
            target_session: Some("session-peer".to_string()),
            target_worker: Some("Peer".to_string()),
            task_id: Some("parser-refactor".to_string()),
            scope: Some("task:parser-refactor".to_string()),
            reason: "peer is blocked on the same live scope".to_string(),
            follow_command: Some("memd hive follow --session session-peer --summary".to_string()),
            deny_command: None,
            reroute_command: None,
            retire_command: None,
            cowork_command: Some(
                "memd hive cowork request --to-session session-peer --task-id parser-refactor --scope task:parser-refactor --reason peer is blocked on the same live scope --summary".to_string(),
            ),
        }],
        recent_receipts: vec!["queen_handoff session-peer parser-refactor".to_string()],
    };

    let summary = render_hive_queen_summary(&response);
    assert!(summary.contains("hive_queen queen=session-queen"));
    assert!(summary.contains("## Action Cards"));
    assert!(summary.contains("request_cowork"));
    assert!(summary.contains("cowork=memd hive cowork request --to-session session-peer"));
}
```

Run:

```bash
cargo test -q -p memd-client --bin memd queen_cowork -- --nocapture
```

Expected: FAIL with missing `cowork_auto_send` and missing `cowork_command` until the model and renderer are extended.

- [ ] **Step 2: Implement the minimal code**

Update the queen model and summary rendering in `crates/memd-client/src/main.rs`:

```rust
#[derive(Debug, Clone, Args)]
struct HiveQueenArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    output: PathBuf,
    #[arg(long)]
    view: Option<String>,
    #[arg(long)]
    recover_session: Option<String>,
    #[arg(long)]
    retire_session: Option<String>,
    #[arg(long)]
    to_session: Option<String>,
    #[arg(long)]
    deny_session: Option<String>,
    #[arg(long)]
    reroute_session: Option<String>,
    #[arg(long)]
    handoff_scope: Option<String>,
    #[arg(long)]
    cowork_auto_send: bool,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Clone, Serialize)]
struct HiveQueenActionCard {
    action: String,
    priority: String,
    target_session: Option<String>,
    target_worker: Option<String>,
    task_id: Option<String>,
    scope: Option<String>,
    reason: String,
    follow_command: Option<String>,
    deny_command: Option<String>,
    reroute_command: Option<String>,
    retire_command: Option<String>,
    cowork_command: Option<String>,
}
```

Then update `run_hive_queen_command(...)` so action cards set `cowork_command` for `request_cowork` and `ack_cowork` suggestions:

```rust
cowork_command: match suggestion.action.as_str() {
    "request_cowork" | "ack_cowork" => suggestion.target_session.as_ref().map(|session| {
        let cowork_verb = if suggestion.action == "ack_cowork" {
            "ack"
        } else {
            "request"
        };
        let mut command = format!("memd hive cowork {cowork_verb}");
        command.push_str(&format!(" --to-session {session}"));
        if let Some(task_id) = suggestion.task_id.as_deref() {
            command.push_str(&format!(" --task-id {task_id}"));
        }
        if let Some(scope) = suggestion.scope.as_deref() {
            command.push_str(&format!(" --scope {scope}"));
        }
        command.push_str(&format!(" --reason {}", suggestion.reason));
        command
    }),
    _ => None,
},
```

And update `render_hive_queen_summary(...)` to print the cowork command when present:

```rust
if let Some(command) = card.cowork_command.as_deref() {
    lines.push(format!("  cowork={command}"));
}
```

- [ ] **Step 3: Run the test again**

Run:

```bash
cargo test -q -p memd-client --bin memd queen_cowork -- --nocapture
```

Expected: PASS. The queen summary should now render a cowork action card with the exact command string.

- [ ] **Step 4: Commit**

```bash
git add crates/memd-client/src/main.rs
git commit -m "feat: render queen cowork action cards"
```

### Task 2: Add queen auto-send dispatch gated by an explicit flag

**Files:**
- Modify: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write the failing test**

Add a runtime test that proves queen can dispatch cowork packets when `--cowork-auto-send` is enabled:

```rust
#[tokio::test]
async fn run_hive_queen_command_auto_sends_cowork_when_enabled() {
    // Build a mock runtime with two live sessions:
    // - current queen session
    // - a peer session that is `blocked` or `near`
    // Invoke run_hive_queen_command with cowork_auto_send = true.
    // Assert the queen response includes a cowork receipt summary.
    // Assert the mock runtime recorded a `cowork_request` or `cowork_ack` message.
}
```

Run:

```bash
cargo test -q -p memd-client --bin memd run_hive_queen_command_auto_sends_cowork_when_enabled -- --nocapture
```

Expected: FAIL because the queen has no auto-send helper yet and the response does not carry dispatched cowork receipts.

- [ ] **Step 2: Implement the minimal code**

Add a small helper in `crates/memd-client/src/main.rs` that turns queen cowork suggestions into real cowork dispatches:

```rust
async fn dispatch_queen_cowork_actions(
    args: &HiveQueenArgs,
    base_url: &str,
    suggestions: &[CoordinationSuggestion],
) -> anyhow::Result<Vec<String>> {
    let mut receipts = Vec::new();

    for suggestion in suggestions.iter().filter(|suggestion| {
        matches!(suggestion.action.as_str(), "request_cowork" | "ack_cowork")
    }) {
        let Some(target_session) = suggestion.target_session.as_deref() else {
            continue;
        };
        let cowork_args = HiveCoworkArgs {
            output: args.output.clone(),
            to_session: Some(target_session.to_string()),
            to_worker: None,
            task_id: suggestion.task_id.clone(),
            scope: suggestion.scope.clone().into_iter().collect(),
            reason: Some(suggestion.reason.clone()),
            note: None,
            json: false,
            summary: false,
        };
        let response = run_hive_cowork_command(
            &cowork_args,
            base_url,
            if suggestion.action == "ack_cowork" { "ack" } else { "request" },
        )
        .await?;
        receipts.push(response.receipt_summary);
    }

    Ok(receipts)
}
```

Then update `run_hive_queen_command(...)` so:

```rust
    let mut recent_receipts = coordination.recent_receipts.clone();
    if args.cowork_auto_send {
        let cowork_receipts = dispatch_queen_cowork_actions(
            args,
            base_url,
            &coordination.suggestions,
        )
        .await?;
        recent_receipts.extend(cowork_receipts);
    }
```
```

Keep the policy explicit:
- do not auto-send unless `args.cowork_auto_send` is true
- only dispatch `request_cowork` and `ack_cowork`
- leave `decline` as a manual queen action

- [ ] **Step 3: Run the test again**

Run:

```bash
cargo test -q -p memd-client --bin memd run_hive_queen_command_auto_sends_cowork_when_enabled -- --nocapture
```

Expected: PASS. The mock runtime should record the cowork packet and the queen response should include the cowork receipt summary.

- [ ] **Step 4: Commit**

```bash
git add crates/memd-client/src/main.rs
git commit -m "feat: auto-send queen cowork actions"
```

### Task 3: Regression coverage for near, blocked, and cowork-active queen behavior

**Files:**
- Modify: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write the failing tests**

Add compact tests for the coordination suggestion engine and queen summary:

```rust
#[test]
fn suggest_coordination_actions_emits_cowork_actions_for_near_blocked_and_active_peers() {
    // Build three awareness entries:
    // - one `near` peer -> request_cowork
    // - one `blocked` peer -> request_cowork high priority
    // - one `cowork_active` peer -> ack_cowork
    // Assert all three suggestion actions are present.
}

#[test]
fn render_hive_queen_summary_includes_cowork_card_commands() {
    // Build a queen response with cowork_command and assert the summary contains it.
}
```

Run:

```bash
cargo test -q -p memd-client --bin memd queen_cowork -- --nocapture
```

Expected: FAIL until the test fixtures cover all three relationship states and the queen render path prints cowork commands.

- [ ] **Step 2: Implement the minimal code**

Use the existing coordination graph and `suggest_coordination_actions(...)` to verify:
- `near` => `request_cowork`
- `blocked` => `request_cowork` with higher priority
- `cowork_active` => `ack_cowork`

If the queen auto-send path already exists from Task 2, make the test assert that:
- `--cowork-auto-send` yields a cowork receipt in `HiveQueenResponse.recent_receipts`
- the received summary matches the packet sent through `run_hive_cowork_command(...)`

- [ ] **Step 3: Run the test again**

Run:

```bash
cargo test -q -p memd-client --bin memd queen_cowork -- --nocapture
```

Expected: PASS. The queen summary, suggestion engine, and live auto-send behavior should all line up.

- [ ] **Step 4: Live dogfood**

Use the rebuilt binary in a shared-hive bundle and verify:

```bash
target/debug/memd hive queen --output /home/josue/.memd --cowork-auto-send --summary
target/debug/memd hive follow --output /home/josue/.memd --session session-008f3488 --summary
```

Expected:
- queen emits cowork action cards for `near`/`blocked`/`cowork_active`
- the follow surface shows the resulting cowork receipt when auto-send is enabled
- existing handoff / deny behavior is unchanged

- [ ] **Step 5: Commit**

```bash
git add crates/memd-client/src/main.rs
git commit -m "test: cover queen cowork auto-action flow"
```
