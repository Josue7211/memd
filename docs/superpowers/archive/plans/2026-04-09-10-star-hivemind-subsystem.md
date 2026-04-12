# 10-Star Hivemind Subsystem Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn `memd hive` into a coherent team runtime with named bees, roster/follow/queen surfaces, automatic stale-bee hygiene, structured handoffs, and an actionable hive board.

**Architecture:** Keep the existing runtime spine: schema types in `memd-schema`, persistence and query support in `memd-server`, and most CLI/render logic in `memd-client`. Add first-class bee identity and hivemind views on top of the current awareness/coordination/task/message primitives instead of inventing a parallel subsystem.

**Tech Stack:** Rust, clap, serde/serde_json, tokio async command paths, existing `memd-client` CLI/runtime, existing `memd-server` persistence/API surface, existing inline test suites in `main.rs`, existing hive task/message/coordination flows.

---

## File Map

- Modify: `crates/memd-schema/src/lib.rs`
  - Add first-class bee identity/runtime fields and any new hive response/view types needed by client/server.
- Modify: `crates/memd-server/src/store.rs`
  - Persist/read new bee identity fields, add roster/follow/handoff query support, and implement automatic stale-session retirement rules.
- Modify: `crates/memd-server/src/main.rs`
  - Expose any new server endpoints needed for roster/follow/handoff/dashboard data without changing the existing shared authority model.
- Modify: `crates/memd-client/src/main.rs`
  - Add `memd hive` subcommands (`roster`, `follow`, `queen`, `lane`, `handoff`), enrich heartbeat/runtime intent, render the new board/roster/follow views, and wire queen automation.
- Reference: `docs/superpowers/specs/2026-04-09-10-star-hivemind-subsystem-design.md`
  - Source of truth for the subsystem contract and rollout order.

## Task 1: Add First-Class Bee Identity Contract

**Files:**
- Modify: `crates/memd-schema/src/lib.rs`
- Modify: `crates/memd-server/src/store.rs`
- Modify: `crates/memd-client/src/main.rs`
- Test: `crates/memd-schema/src/lib.rs`
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write the failing schema roundtrip test**

Add a schema test near the existing hive session serde coverage:

```rust
#[test]
fn hive_session_roundtrips_worker_identity_fields() {
    let session = HiveSessionRecord {
        session_id: "session-lorentz".to_string(),
        project: "memd".to_string(),
        namespace: "main".to_string(),
        workspace: Some("shared".to_string()),
        agent: "codex".to_string(),
        harness: "codex".to_string(),
        worker_name: Some("Lorentz".to_string()),
        display_name: Some("Parser Reviewer".to_string()),
        role: Some("reviewer".to_string()),
        capabilities: vec!["review".to_string(), "coordination".to_string()],
        lane_id: Some("lane-render-review".to_string()),
        repo_root: Some("/repo".to_string()),
        worktree_root: Some("/repo-review".to_string()),
        branch: Some("review/render".to_string()),
        base_branch: Some("main".to_string()),
        task_id: Some("review-parser-handoff".to_string()),
        topic_claim: Some("Review parser handoff".to_string()),
        scope_claims: vec!["crates/memd-client/src/main.rs".to_string()],
        next_action: Some("Review overlap guard output".to_string()),
        status: Some("active".to_string()),
        needs_help: false,
        needs_review: false,
        handoff_state: Some("none".to_string()),
        confidence: Some("high".to_string()),
        risk: Some("low".to_string()),
        updated_at: Utc::now(),
    };

    let json = serde_json::to_string(&session).expect("serialize session");
    let decoded: HiveSessionRecord = serde_json::from_str(&json).expect("deserialize session");
    assert_eq!(decoded.worker_name.as_deref(), Some("Lorentz"));
    assert_eq!(decoded.display_name.as_deref(), Some("Parser Reviewer"));
    assert_eq!(decoded.role.as_deref(), Some("reviewer"));
    assert_eq!(decoded.lane_id.as_deref(), Some("lane-render-review"));
    assert_eq!(decoded.next_action.as_deref(), Some("Review overlap guard output"));
    assert_eq!(decoded.risk.as_deref(), Some("low"));
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
cargo test -p memd-schema hive_session_roundtrips_worker_identity_fields -- --nocapture
```

Expected: FAIL because `HiveSessionRecord` does not yet expose the full identity/runtime contract.

- [ ] **Step 3: Extend the schema types**

Update the hive session/request types in `crates/memd-schema/src/lib.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HiveSessionRecord {
    pub session_id: String,
    pub project: String,
    pub namespace: String,
    pub workspace: Option<String>,
    pub agent: String,
    pub harness: String,
    #[serde(default)]
    pub worker_name: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub lane_id: Option<String>,
    #[serde(default)]
    pub repo_root: Option<String>,
    #[serde(default)]
    pub worktree_root: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub base_branch: Option<String>,
    #[serde(default)]
    pub task_id: Option<String>,
    #[serde(default)]
    pub topic_claim: Option<String>,
    #[serde(default)]
    pub scope_claims: Vec<String>,
    #[serde(default)]
    pub next_action: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub needs_help: bool,
    #[serde(default)]
    pub needs_review: bool,
    #[serde(default)]
    pub handoff_state: Option<String>,
    #[serde(default)]
    pub confidence: Option<String>,
    #[serde(default)]
    pub risk: Option<String>,
    pub updated_at: DateTime<Utc>,
}
```

Apply the same fields to the heartbeat request/update types so the client can publish them and the server can preserve them.

- [ ] **Step 4: Persist the new fields in the server store**

Update the server-side upsert/load mapping in `crates/memd-server/src/store.rs`:

```rust
worker_name: request.worker_name.clone().filter(|value| !value.trim().is_empty()),
display_name: request.display_name.clone().filter(|value| !value.trim().is_empty()),
role: request.role.clone().filter(|value| !value.trim().is_empty()),
lane_id: request.lane_id.clone().filter(|value| !value.trim().is_empty()),
next_action: request.next_action.clone().filter(|value| !value.trim().is_empty()),
status: request.status.clone().filter(|value| !value.trim().is_empty()),
needs_help: request.needs_help,
needs_review: request.needs_review,
handoff_state: request.handoff_state.clone().filter(|value| !value.trim().is_empty()),
confidence: request.confidence.clone().filter(|value| !value.trim().is_empty()),
risk: request.risk.clone().filter(|value| !value.trim().is_empty()),
```

Do not create a new table for this task. Reuse the existing session payload storage shape and keep the migration additive.

- [ ] **Step 5: Publish the new fields from the client heartbeat**

Update the heartbeat builder in `crates/memd-client/src/main.rs`:

```rust
let worker_name = runtime
    .as_ref()
    .and_then(|value| value.hive_worker_name.clone())
    .or_else(|| derive_worker_name_from_agent_id(&args.agent, &args.session));

let display_name = runtime
    .as_ref()
    .and_then(|value| value.hive_display_name.clone());

let role = runtime
    .as_ref()
    .and_then(|value| value.hive_role.clone())
    .or_else(|| Some("worker".to_string()));

let next_action = derive_hive_next_action(focus.as_deref(), pressure.as_deref());
let status = Some(if snapshot_idle { "idle" } else { "active" }.to_string());
let risk = derive_hive_risk(&scope_claims, pressure.as_deref());
```

- [ ] **Step 6: Re-run the targeted tests**

Run:

```bash
cargo test -p memd-schema hive_session_roundtrips_worker_identity_fields -- --nocapture
cargo test -p memd-client --bin memd build_hive_heartbeat_derives_first_class_intent_fields -- --nocapture
```

Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add crates/memd-schema/src/lib.rs crates/memd-server/src/store.rs crates/memd-client/src/main.rs
git commit -m "feat: add first-class hive bee identity contract"
```

## Task 2: Add `memd hive roster`

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Modify: `crates/memd-schema/src/lib.rs`
- Modify: `crates/memd-server/src/main.rs`
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write the failing roster render test**

Add a client test:

```rust
#[test]
fn render_hive_roster_summary_prefers_worker_names_and_role_lane_task() {
    let response = HiveRosterResponse {
        project: "memd".to_string(),
        namespace: "main".to_string(),
        queen_session: Some("session-queen".to_string()),
        bees: vec![HiveSessionRecord {
            session_id: "session-lorentz".to_string(),
            project: "memd".to_string(),
            namespace: "main".to_string(),
            workspace: Some("shared".to_string()),
            agent: "codex".to_string(),
            harness: "codex".to_string(),
            worker_name: Some("Lorentz".to_string()),
            display_name: None,
            role: Some("reviewer".to_string()),
            capabilities: vec!["review".to_string(), "coordination".to_string()],
            lane_id: Some("lane-review".to_string()),
            repo_root: Some("/repo".to_string()),
            worktree_root: Some("/repo-review".to_string()),
            branch: Some("review/parser".to_string()),
            base_branch: Some("main".to_string()),
            task_id: Some("review-parser".to_string()),
            topic_claim: Some("Review parser handoff".to_string()),
            scope_claims: vec!["crates/memd-client/src/main.rs".to_string()],
            next_action: Some("Review overlap guard output".to_string()),
            status: Some("active".to_string()),
            needs_help: false,
            needs_review: true,
            handoff_state: Some("none".to_string()),
            confidence: Some("high".to_string()),
            risk: Some("low".to_string()),
            updated_at: Utc::now(),
        }],
    };

    let summary = render_hive_roster_summary(&response);
    assert!(summary.contains("Lorentz (session-lorentz)"));
    assert!(summary.contains("role=reviewer"));
    assert!(summary.contains("lane=lane-review"));
    assert!(summary.contains("task=review-parser"));
    assert!(summary.contains("caps=review,coordination"));
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
cargo test -p memd-client --bin memd render_hive_roster_summary_prefers_worker_names_and_role_lane_task -- --nocapture
```

Expected: FAIL because the roster surface and renderer do not exist.

- [ ] **Step 3: Add the CLI subcommand and response type**

Extend the hive command shape in `crates/memd-client/src/main.rs`:

```rust
#[derive(Subcommand, Debug, Clone)]
enum HiveSubcommand {
    Board(HiveArgs),
    Roster(HiveRosterArgs),
    Follow(HiveFollowArgs),
    Queen(HiveQueenArgs),
    Lane(HiveLaneArgs),
    Handoff(HiveHandoffArgs),
}

#[derive(Args, Debug, Clone)]
struct HiveRosterArgs {
    #[arg(long, default_value = ".memd")]
    output: PathBuf,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    summary: bool,
}
```

Add a minimal response type in `crates/memd-schema/src/lib.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HiveRosterResponse {
    pub project: String,
    pub namespace: String,
    pub queen_session: Option<String>,
    pub bees: Vec<HiveSessionRecord>,
}
```

- [ ] **Step 4: Implement roster query + renderer**

In `crates/memd-client/src/main.rs`, reuse `read_project_awareness` and normalize it into roster rows:

```rust
async fn run_hive_roster_command(args: &HiveRosterArgs, base_url: &str) -> anyhow::Result<HiveRosterResponse> {
    let awareness = read_project_awareness(&AwarenessArgs {
        output: args.output.clone(),
        json: false,
        summary: false,
        local_only: false,
    }).await?;

    Ok(HiveRosterResponse {
        project: awareness.project.clone().unwrap_or_else(|| "unknown".to_string()),
        namespace: awareness.namespace.clone().unwrap_or_else(|| "default".to_string()),
        queen_session: awareness.entries.iter().find(|entry| entry.role.as_deref() == Some("queen")).map(|entry| entry.session_id.clone()),
        bees: project_awareness_visible_entries(&awareness)
            .into_iter()
            .map(project_awareness_entry_to_hive_session)
            .collect(),
    })
}
```

Render format:

```rust
format!(
    "{} ({}) role={} lane={} task={} caps={}",
    bee.worker_name.as_deref().unwrap_or("unnamed"),
    bee.session_id,
    bee.role.as_deref().unwrap_or("worker"),
    bee.lane_id.as_deref().unwrap_or("none"),
    bee.task_id.as_deref().unwrap_or("none"),
    compact_inline(&bee.capabilities.join(","), 48),
)
```

- [ ] **Step 5: Re-run the targeted test**

Run:

```bash
cargo test -p memd-client --bin memd render_hive_roster_summary_prefers_worker_names_and_role_lane_task -- --nocapture
```

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add crates/memd-client/src/main.rs crates/memd-schema/src/lib.rs crates/memd-server/src/main.rs
git commit -m "feat: add hive roster command"
```

## Task 3: Add `memd hive follow`

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Modify: `crates/memd-schema/src/lib.rs`
- Modify: `crates/memd-server/src/store.rs`
- Modify: `crates/memd-server/src/main.rs`
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write the failing follow summary test**

Add:

```rust
#[test]
fn render_hive_follow_summary_surfaces_messages_receipts_and_overlap_risk() {
    let response = HiveFollowResponse {
        bee: sample_hive_follow_bee("Lorentz", "session-lorentz", "review-parser"),
        messages: vec!["Bee B requested parser review".to_string()],
        receipts: vec!["queen_assign review-parser -> Lorentz".to_string()],
        task_transitions: vec!["task none -> review-parser".to_string()],
        overlap_with_current: Some("shares crates/memd-client/src/main.rs".to_string()),
        recommended_action: "coordinate".to_string(),
    };

    let summary = render_hive_follow_summary(&response);
    assert!(summary.contains("Lorentz (session-lorentz)"));
    assert!(summary.contains("messages=1"));
    assert!(summary.contains("receipts=1"));
    assert!(summary.contains("overlap=shares crates/memd-client/src/main.rs"));
    assert!(summary.contains("recommended=coordinate"));
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
cargo test -p memd-client --bin memd render_hive_follow_summary_surfaces_messages_receipts_and_overlap_risk -- --nocapture
```

Expected: FAIL because follow types/renderer do not exist.

- [ ] **Step 3: Add the follow CLI + response types**

Add to schema:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HiveFollowResponse {
    pub bee: HiveSessionRecord,
    pub messages: Vec<String>,
    pub receipts: Vec<String>,
    pub task_transitions: Vec<String>,
    pub overlap_with_current: Option<String>,
    pub recommended_action: String,
}
```

Add CLI args:

```rust
#[derive(Args, Debug, Clone)]
struct HiveFollowArgs {
    #[arg(long, default_value = ".memd")]
    output: PathBuf,
    #[arg(long)]
    worker: Option<String>,
    #[arg(long)]
    session: Option<String>,
    #[arg(long)]
    watch: bool,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    summary: bool,
    #[arg(long, value_delimiter = ',')]
    show: Vec<String>,
    #[arg(long)]
    overlap_with: Option<String>,
}
```

- [ ] **Step 4: Implement follow resolution**

In the client:

```rust
async fn run_hive_follow_command(args: &HiveFollowArgs, base_url: &str) -> anyhow::Result<HiveFollowResponse> {
    let awareness = read_project_awareness(&AwarenessArgs {
        output: args.output.clone(),
        json: false,
        summary: false,
        local_only: false,
    }).await?;
    let coordination = run_coordination_command(&CoordinationArgs::summary_only(args.output.clone()), base_url).await?;
    let bee = resolve_follow_target(&awareness, args.worker.as_deref(), args.session.as_deref())?;
    let overlap = derive_follow_overlap_reason(&bee, awareness.current_session.as_deref(), &awareness.entries);

    Ok(HiveFollowResponse {
        bee: project_awareness_entry_to_hive_session(bee),
        messages: summarize_follow_messages(&coordination, &bee.session_id),
        receipts: summarize_follow_receipts(&coordination, &bee.session_id),
        task_transitions: summarize_follow_task_transitions(&coordination, &bee.session_id),
        overlap_with_current: overlap,
        recommended_action: recommend_follow_action(&bee, &coordination),
    })
}
```

Do not implement live streaming in this task. `--watch` should re-run the same follow snapshot on an interval, using the existing async runtime.

- [ ] **Step 5: Re-run the targeted test**

Run:

```bash
cargo test -p memd-client --bin memd render_hive_follow_summary_surfaces_messages_receipts_and_overlap_risk -- --nocapture
```

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add crates/memd-client/src/main.rs crates/memd-schema/src/lib.rs crates/memd-server/src/store.rs crates/memd-server/src/main.rs
git commit -m "feat: add hive follow command"
```

## Task 4: Add `memd hive queen`

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Modify: `crates/memd-schema/src/lib.rs`
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write the failing queen summary test**

Add:

```rust
#[test]
fn render_hive_queen_summary_surfaces_explicit_actions() {
    let response = HiveQueenResponse {
        queen_session: "session-queen".to_string(),
        suggested_actions: vec![
            "reroute Lorentz off crates/memd-client/src/main.rs".to_string(),
            "retire stale bee session-old".to_string(),
        ],
        recent_receipts: vec![
            "queen_assign session-lorentz review-parser".to_string(),
            "queen_deny session-avicenna overlap-main-rs".to_string(),
        ],
    };

    let summary = render_hive_queen_summary(&response);
    assert!(summary.contains("queen=session-queen"));
    assert!(summary.contains("reroute Lorentz"));
    assert!(summary.contains("queen_deny session-avicenna"));
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
cargo test -p memd-client --bin memd render_hive_queen_summary_surfaces_explicit_actions -- --nocapture
```

Expected: FAIL because there is no first-class queen surface yet.

- [ ] **Step 3: Add the queen CLI wrapper**

Add:

```rust
#[derive(Args, Debug, Clone)]
struct HiveQueenArgs {
    #[arg(long, default_value = ".memd")]
    output: PathBuf,
    #[arg(long)]
    assign_task: Option<String>,
    #[arg(long)]
    to_session: Option<String>,
    #[arg(long)]
    deny_session: Option<String>,
    #[arg(long)]
    reroute_session: Option<String>,
    #[arg(long)]
    retire_session: Option<String>,
    #[arg(long)]
    handoff_scope: Option<String>,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    summary: bool,
}
```

Keep the implementation as a thin wrapper around the existing `CoordinationArgs` path so there is still one queen authority flow.

- [ ] **Step 4: Implement queen summary + action dispatch**

```rust
async fn run_hive_queen_command(args: &HiveQueenArgs, base_url: &str) -> anyhow::Result<HiveQueenResponse> {
    let coordination = run_coordination_command(
        &CoordinationArgs {
            output: args.output.clone(),
            to_session: args.to_session.clone(),
            assign_task: args.assign_task.clone(),
            deny_session: args.deny_session.clone(),
            reroute_session: args.reroute_session.clone(),
            retire_session: args.retire_session.clone(),
            handoff_scope: args.handoff_scope.clone(),
            json: false,
            summary: false,
            ..CoordinationArgs::default()
        },
        base_url,
    ).await?;

    Ok(HiveQueenResponse {
        queen_session: coordination.current_session.clone(),
        suggested_actions: suggest_coordination_actions(
            &coordination.active_hives,
            &coordination.tasks,
            &coordination.policy_conflicts,
            &coordination.current_session,
        )
        .into_iter()
        .map(|value| value.title)
        .collect(),
        recent_receipts: coordination
            .receipts
            .iter()
            .filter(|receipt| receipt.kind.starts_with("queen_"))
            .map(|receipt| format!("{} {} {}", receipt.kind, receipt.actor_session, receipt.summary))
            .collect(),
    })
}
```

- [ ] **Step 5: Re-run the targeted test**

Run:

```bash
cargo test -p memd-client --bin memd render_hive_queen_summary_surfaces_explicit_actions -- --nocapture
```

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add crates/memd-client/src/main.rs crates/memd-schema/src/lib.rs
git commit -m "feat: add hive queen surface"
```

## Task 5: Polish the Default `memd hive` Team Board

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Modify: `crates/memd-schema/src/lib.rs`
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write the failing hive board summary test**

Add:

```rust
#[test]
fn render_hive_board_summary_surfaces_board_sections() {
    let response = HiveBoardResponse {
        queen_session: Some("session-queen".to_string()),
        active_bees: vec![sample_hive_follow_bee("Lorentz", "session-lorentz", "review-parser")],
        blocked_bees: vec!["Avicenna overlap on crates/memd-client/src/main.rs".to_string()],
        stale_bees: vec!["session-old".to_string()],
        review_queue: vec!["review-parser -> Lorentz".to_string()],
        overlap_risks: vec!["Lorentz vs Avicenna on crates/memd-client/src/main.rs".to_string()],
        lane_faults: vec!["lane_fault session-avicenna shared worktree".to_string()],
        recommended_actions: vec!["reroute Avicenna".to_string()],
    };

    let summary = render_hive_board_summary(&response);
    assert!(summary.contains("## Active Bees"));
    assert!(summary.contains("## Review Queue"));
    assert!(summary.contains("## Recommended Actions"));
    assert!(summary.contains("Lorentz (session-lorentz)"));
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
cargo test -p memd-client --bin memd render_hive_board_summary_surfaces_board_sections -- --nocapture
```

Expected: FAIL because the main hive command is still not a dedicated board.

- [ ] **Step 3: Build the board response from existing primitives**

Add a response type:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HiveBoardResponse {
    pub queen_session: Option<String>,
    pub active_bees: Vec<HiveSessionRecord>,
    pub blocked_bees: Vec<String>,
    pub stale_bees: Vec<String>,
    pub review_queue: Vec<String>,
    pub overlap_risks: Vec<String>,
    pub lane_faults: Vec<String>,
    pub recommended_actions: Vec<String>,
}
```

Build it by composing roster + coordination:

```rust
async fn run_hive_board_command(args: &HiveArgs, base_url: &str) -> anyhow::Result<HiveBoardResponse> {
    let roster = run_hive_roster_command(&HiveRosterArgs::from_output(args.output.clone()), base_url).await?;
    let coordination = run_coordination_command(&CoordinationArgs::summary_only(args.output.clone()), base_url).await?;

    Ok(HiveBoardResponse {
        queen_session: roster.queen_session.clone().or_else(|| Some(coordination.current_session.clone())),
        active_bees: roster.bees.into_iter().filter(|bee| bee.status.as_deref() == Some("active")).collect(),
        blocked_bees: coordination.policy_conflicts.iter().map(|value| value.summary.clone()).collect(),
        stale_bees: coordination.stale_sessions.iter().map(|value| value.session_id.clone()).collect(),
        review_queue: coordination.tasks.iter().filter(|task| task.coordination_mode == "shared_review").map(|task| format!("{} -> {}", task.id, task.owner_session.as_deref().unwrap_or("unassigned"))).collect(),
        overlap_risks: coordination.policy_conflicts.iter().filter(|value| value.kind.contains("overlap")).map(|value| value.summary.clone()).collect(),
        lane_faults: coordination.receipts.iter().filter(|receipt| receipt.kind == "lane_fault").map(|receipt| receipt.summary.clone()).collect(),
        recommended_actions: suggest_coordination_actions(&coordination.active_hives, &coordination.tasks, &coordination.policy_conflicts, &coordination.current_session).into_iter().map(|value| value.title).collect(),
    })
}
```

- [ ] **Step 4: Re-run the targeted test**

Run:

```bash
cargo test -p memd-client --bin memd render_hive_board_summary_surfaces_board_sections -- --nocapture
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/memd-client/src/main.rs crates/memd-schema/src/lib.rs
git commit -m "feat: make memd hive a team board"
```

## Task 6: Auto-Retire Stale Bees

**Files:**
- Modify: `crates/memd-server/src/store.rs`
- Modify: `crates/memd-server/src/main.rs`
- Modify: `crates/memd-client/src/main.rs`
- Test: `crates/memd-client/src/main.rs`
- Test: `crates/memd-server/src/store.rs`

- [ ] **Step 1: Write the failing stale-retirement tests**

Add a server test:

```rust
#[test]
fn hive_coordination_auto_retires_stale_session_without_owned_work() {
    let store = test_store();
    seed_stale_session(&store, "session-old", Duration::minutes(45));
    let sessions = store.list_hive_sessions("memd", "main").expect("list sessions");
    assert!(sessions.iter().any(|session| session.session_id == "session-old"));

    let retired = store.auto_retire_stale_hive_sessions("memd", "main", Utc::now()).expect("auto retire");
    assert_eq!(retired, vec!["session-old".to_string()]);
}
```

Add a client test:

```rust
#[tokio::test]
async fn run_hive_board_command_prunes_retired_stale_bees_from_default_view() {
    let output = test_bundle_dir("memd-hive-board-retire");
    seed_test_bundle(&output, "session-current");
    seed_stale_sibling_bundle(&output, "session-stale", None, None);

    let board = run_hive_board_command(&HiveArgs::summary_only(output.clone()), "http://127.0.0.1:8787")
        .await
        .expect("board");

    assert!(board.stale_bees.is_empty());
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run:

```bash
cargo test -p memd-server hive_coordination_auto_retires_stale_session_without_owned_work -- --nocapture
cargo test -p memd-client --bin memd run_hive_board_command_prunes_retired_stale_bees_from_default_view -- --nocapture
```

Expected: FAIL because retirement is still advisory/manual.

- [ ] **Step 3: Implement server-side stale retirement helper**

In `crates/memd-server/src/store.rs`:

```rust
pub fn auto_retire_stale_hive_sessions(
    &self,
    project: &str,
    namespace: &str,
    now: DateTime<Utc>,
) -> anyhow::Result<Vec<String>> {
    let sessions = self.list_hive_sessions(project, namespace)?;
    let tasks = self.hive_tasks(project, namespace)?;
    let claims = self.hive_claims(project, namespace)?;

    let retired = sessions
        .into_iter()
        .filter(|session| is_retireable_stale_session(session, &tasks, &claims, now))
        .map(|session| session.session_id)
        .collect::<Vec<_>>();

    for session_id in &retired {
        self.retire_hive_session(project, namespace, session_id)?;
    }

    Ok(retired)
}
```

Retire only if:
- last heartbeat is stale
- no owned active task
- no active claim
- no pending handoff receipt

- [ ] **Step 4: Call auto-retirement from the board/coordination read path**

In the client, before rendering the board:

```rust
let _ = timeout_ok(client.auto_retire_hive_sessions(&HiveAutoRetireRequest {
    project: project.clone(),
    namespace: namespace.clone(),
}));
```

If the backend does not support the route yet, keep the client readable and non-fatal. On the shared path, it should be best-effort cleanup, not a board blocker.

- [ ] **Step 5: Re-run the targeted tests**

Run:

```bash
cargo test -p memd-server hive_coordination_auto_retires_stale_session_without_owned_work -- --nocapture
cargo test -p memd-client --bin memd run_hive_board_command_prunes_retired_stale_bees_from_default_view -- --nocapture
```

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add crates/memd-server/src/store.rs crates/memd-server/src/main.rs crates/memd-client/src/main.rs
git commit -m "feat: auto-retire stale hive bees"
```

## Task 7: Add Structured `memd hive handoff`

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Modify: `crates/memd-schema/src/lib.rs`
- Modify: `crates/memd-server/src/store.rs`
- Modify: `crates/memd-server/src/main.rs`
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write the failing handoff test**

Add:

```rust
#[tokio::test]
async fn run_hive_handoff_command_builds_compact_handoff_packet() {
    let output = test_bundle_dir("memd-hive-handoff");
    seed_test_bundle(&output, "session-lorentz");

    let response = run_hive_handoff_command(
        &HiveHandoffArgs {
            output: output.clone(),
            to_session: Some("session-avicenna".to_string()),
            task_id: Some("review-parser".to_string()),
            scope: vec!["crates/memd-client/src/main.rs".to_string()],
            blocker: Some("Need reviewer takeover".to_string()),
            next_action: Some("Validate overlap policy".to_string()),
            json: false,
            summary: true,
        },
        "http://127.0.0.1:8787",
    )
    .await
    .expect("handoff");

    assert_eq!(response.to_session.as_deref(), Some("session-avicenna"));
    assert_eq!(response.task_id.as_deref(), Some("review-parser"));
    assert!(response.scope_claims.iter().any(|value| value == "crates/memd-client/src/main.rs"));
    assert_eq!(response.blocker.as_deref(), Some("Need reviewer takeover"));
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
cargo test -p memd-client --bin memd run_hive_handoff_command_builds_compact_handoff_packet -- --nocapture
```

Expected: FAIL because there is no first-class hive handoff command.

- [ ] **Step 3: Add the handoff response/args types**

Add to schema:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HiveHandoffPacket {
    pub from_session: String,
    pub to_session: Option<String>,
    pub task_id: Option<String>,
    pub topic_claim: Option<String>,
    pub scope_claims: Vec<String>,
    pub status: Option<String>,
    pub blocker: Option<String>,
    pub next_action: Option<String>,
}
```

Add CLI args:

```rust
#[derive(Args, Debug, Clone)]
struct HiveHandoffArgs {
    #[arg(long, default_value = ".memd")]
    output: PathBuf,
    #[arg(long)]
    to_session: Option<String>,
    #[arg(long)]
    task_id: Option<String>,
    #[arg(long, value_delimiter = ',')]
    scope: Vec<String>,
    #[arg(long)]
    blocker: Option<String>,
    #[arg(long)]
    next_action: Option<String>,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    summary: bool,
}
```

- [ ] **Step 4: Implement the handoff command on top of existing receipts/messages**

```rust
async fn run_hive_handoff_command(args: &HiveHandoffArgs, base_url: &str) -> anyhow::Result<HiveHandoffPacket> {
    let runtime = read_bundle_runtime_config(&args.output)?;
    let packet = HiveHandoffPacket {
        from_session: runtime.session.clone(),
        to_session: args.to_session.clone(),
        task_id: args.task_id.clone(),
        topic_claim: derive_hive_topic_claim_from_task(args.task_id.as_deref()),
        scope_claims: args.scope.clone(),
        status: Some("handoff_requested".to_string()),
        blocker: args.blocker.clone(),
        next_action: args.next_action.clone(),
    };

    emit_coordination_receipt(
        base_url,
        "queen_handoff",
        &runtime.session,
        packet.to_session.clone(),
        packet.task_id.clone(),
        Some(format!("handoff {}", compact_inline(&packet.scope_claims.join(","), 72))),
    ).await?;

    send_handoff_message(base_url, &packet).await?;
    Ok(packet)
}
```

Keep packets compact. Do not attach transcript text.

- [ ] **Step 5: Re-run the targeted test**

Run:

```bash
cargo test -p memd-client --bin memd run_hive_handoff_command_builds_compact_handoff_packet -- --nocapture
```

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add crates/memd-client/src/main.rs crates/memd-schema/src/lib.rs crates/memd-server/src/store.rs crates/memd-server/src/main.rs
git commit -m "feat: add hive handoff packets"
```

## Task 8: Add Dashboard Parity Data Surface

**Files:**
- Modify: `crates/memd-schema/src/lib.rs`
- Modify: `crates/memd-server/src/main.rs`
- Modify: `crates/memd-client/src/main.rs`
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write the failing dashboard payload test**

Add:

```rust
#[test]
fn hive_board_response_includes_dashboard_panels() {
    let response = HiveBoardResponse {
        queen_session: Some("session-queen".to_string()),
        active_bees: vec![sample_hive_follow_bee("Lorentz", "session-lorentz", "review-parser")],
        blocked_bees: vec!["Avicenna overlap".to_string()],
        stale_bees: vec!["session-old".to_string()],
        review_queue: vec!["review-parser -> Lorentz".to_string()],
        overlap_risks: vec!["Lorentz vs Avicenna".to_string()],
        lane_faults: vec!["lane_fault session-avicenna".to_string()],
        recommended_actions: vec!["reroute Avicenna".to_string()],
    };

    let json = serde_json::to_value(&response).expect("serialize board");
    assert!(json.get("active_bees").is_some());
    assert!(json.get("review_queue").is_some());
    assert!(json.get("lane_faults").is_some());
    assert!(json.get("recommended_actions").is_some());
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
cargo test -p memd-client --bin memd hive_board_response_includes_dashboard_panels -- --nocapture
```

Expected: FAIL because the board response is not yet formalized as the dashboard contract.

- [ ] **Step 3: Expose a stable hive board JSON route**

In `crates/memd-server/src/main.rs`, add a route that returns the same board contract used by the CLI:

```rust
Router::new()
    .route("/hive/board", get(get_hive_board))
    .route("/hive/roster", get(get_hive_roster))
    .route("/hive/follow/:session_id", get(get_hive_follow))
```

The handler should reuse the same store/query helpers built in earlier tasks rather than recalculating ad hoc.

- [ ] **Step 4: Make the CLI `--json` path use the stable response objects**

Keep the CLI and dashboard in sync by serializing the same response structs:

```rust
if args.json {
    println!("{}", serde_json::to_string_pretty(&board)?);
    return Ok(());
}
println!("{}", render_hive_board_summary(&board));
```

- [ ] **Step 5: Re-run the targeted test**

Run:

```bash
cargo test -p memd-client --bin memd hive_board_response_includes_dashboard_panels -- --nocapture
```

Expected: PASS

- [ ] **Step 6: Run the final focused verification set**

Run:

```bash
cargo test -p memd-schema hive_session_roundtrips_worker_identity_fields -- --nocapture
cargo test -p memd-client --bin memd render_hive_roster_summary_prefers_worker_names_and_role_lane_task -- --nocapture
cargo test -p memd-client --bin memd render_hive_follow_summary_surfaces_messages_receipts_and_overlap_risk -- --nocapture
cargo test -p memd-client --bin memd render_hive_queen_summary_surfaces_explicit_actions -- --nocapture
cargo test -p memd-client --bin memd render_hive_board_summary_surfaces_board_sections -- --nocapture
cargo test -p memd-client --bin memd run_hive_board_command_prunes_retired_stale_bees_from_default_view -- --nocapture
cargo test -p memd-client --bin memd run_hive_handoff_command_builds_compact_handoff_packet -- --nocapture
cargo test -p memd-client --bin memd hive_board_response_includes_dashboard_panels -- --nocapture
cargo test -p memd-server hive_coordination_auto_retires_stale_session_without_owned_work -- --nocapture
```

Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add crates/memd-schema/src/lib.rs crates/memd-server/src/main.rs crates/memd-server/src/store.rs crates/memd-client/src/main.rs
git commit -m "feat: add 10-star hivemind board contract"
```

## Spec Coverage Check

- Roster and identity model: covered by Task 1 and Task 2.
- `memd hive follow`: covered by Task 3.
- Queen-first action surface: covered by Task 4.
- Team board polish: covered by Task 5.
- Automatic stale-bee retirement: covered by Task 6.
- Structured handoff flow: covered by Task 7.
- Dashboard parity: covered by Task 8.

## Placeholder Scan

- No `TODO`, `TBD`, or “implement later” placeholders remain.
- Every task includes exact files, commands, and concrete code snippets.

## Type Consistency Check

- Shared runtime record: `HiveSessionRecord`
- Board response: `HiveBoardResponse`
- Roster response: `HiveRosterResponse`
- Follow response: `HiveFollowResponse`
- Handoff packet: `HiveHandoffPacket`
- Identity fields consistently use:
  - `worker_name`
  - `display_name`
  - `lane_id`
  - `task_id`
  - `topic_claim`
  - `scope_claims`
