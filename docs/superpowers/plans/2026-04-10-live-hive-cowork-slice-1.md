# Live Hive Cowork Slice 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add canonical live `working` and `touches` state to hive sessions, compute `relationship_state` for peer visibility, and surface that live cowork signal in the roster.

**Architecture:** Extend the canonical hive session payload in `memd-schema`, publish the new fields from the client heartbeat/build path, and derive relationship state in the client roster path so the first slice delivers live cowork visibility without waiting on queen workflow changes. Keep persistence backward-compatible by using optional fields and JSON payload updates only.

**Tech Stack:** Rust, `memd-schema`, `memd-client`, existing mock runtime server tests, Cargo test

---

## File Structure

### Core payload shape

- Modify: `crates/memd-schema/src/lib.rs`
  - Extend `HiveSessionRecord` with canonical cowork fields.

### Client publishing and derivation

- Modify: `crates/memd-client/src/main.rs`
  - Build canonical `working` and normalized `touches` in the heartbeat path.
  - Preserve backward-compatible fallback from existing `topic_claim` and `scope_claims`.
  - Compute `relationship_state`, `relationship_peer`, `relationship_reason`, and `suggested_action` in the roster path.
  - Render the new roster summary.

### Verification

- Modify: `crates/memd-client/src/main.rs`
  - Add unit and integration-style tests near the existing hive tests.

## Task 1: Extend The Canonical Hive Session Payload

**Files:**
- Modify: `crates/memd-schema/src/lib.rs`
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write the failing roster test that expects live cowork fields**

Add a test near `render_hive_roster_summary_prefers_worker_names_and_role_lane_task`:

```rust
#[test]
fn render_hive_roster_summary_surfaces_working_touches_and_relationship() {
    let response = HiveRosterResponse {
        project: "memd".to_string(),
        namespace: "main".to_string(),
        queen_session: None,
        bees: vec![HiveSessionRecord {
            session: "session-memd".to_string(),
            worker_name: Some("Memd".to_string()),
            working: Some("fixing hive group visibility".to_string()),
            touches: vec![
                "file:crates/memd-client/src/main.rs".to_string(),
                "task:hive-awareness".to_string(),
            ],
            relationship_state: Some("near".to_string()),
            relationship_peer: Some("Clawcontrol".to_string()),
            relationship_reason: Some("shared session-merge area".to_string()),
            suggested_action: Some("cowork".to_string()),
            status: "live".to_string(),
            ..test_hive_session_record("session-memd")
        }],
    };

    let summary = render_hive_roster_summary(&response);
    assert!(summary.contains("work=\"fixing hive group visibility\""));
    assert!(summary.contains("touches=file:crates/memd-client/src/main.rs,task:hive-awareness"));
    assert!(summary.contains("relation=near:Clawcontrol"));
    assert!(summary.contains("action=cowork"));
}
```

- [ ] **Step 2: Run the test to verify it fails on missing fields**

Run:

```bash
cargo test -q -p memd-client --bin memd render_hive_roster_summary_surfaces_working_touches_and_relationship -- --nocapture
```

Expected:

- compile failure because `HiveSessionRecord` does not yet have `working`, `touches`, `relationship_state`, `relationship_peer`, `relationship_reason`, and `suggested_action`

- [ ] **Step 3: Extend `HiveSessionRecord` with optional cowork fields**

Update `crates/memd-schema/src/lib.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HiveSessionRecord {
    // existing fields ...
    #[serde(default)]
    pub next_action: Option<String>,
    #[serde(default)]
    pub working: Option<String>,
    #[serde(default)]
    pub touches: Vec<String>,
    #[serde(default)]
    pub relationship_state: Option<String>,
    #[serde(default)]
    pub relationship_peer: Option<String>,
    #[serde(default)]
    pub relationship_reason: Option<String>,
    #[serde(default)]
    pub suggested_action: Option<String>,
    #[serde(default)]
    pub needs_help: bool,
    #[serde(default)]
    pub needs_review: bool,
    // existing fields ...
}
```

- [ ] **Step 4: Run the same test again to reach the next failure**

Run:

```bash
cargo test -q -p memd-client --bin memd render_hive_roster_summary_surfaces_working_touches_and_relationship -- --nocapture
```

Expected:

- compile or assertion failure in `memd-client` until the renderer is updated

- [ ] **Step 5: Commit the schema extension once the client compiles against it**

```bash
git add crates/memd-schema/src/lib.rs crates/memd-client/src/main.rs
git commit -m "feat: add canonical hive cowork session fields"
```

## Task 2: Publish Canonical `working` And Normalized `touches`

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write a failing heartbeat test for canonical live work state**

Add a test near `enrich_hive_heartbeat_with_runtime_intent_prefers_owned_task_state`:

```rust
#[tokio::test]
async fn build_hive_heartbeat_sets_working_and_normalized_touches() {
    let dir = std::env::temp_dir().join(format!(
        "memd-heartbeat-working-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(dir.join(".memd")).expect("create bundle");
    write_test_bundle_config(&dir.join(".memd"), "http://127.0.0.1:8787");

    let snapshot = ResumeSnapshot {
        working: test_working_snapshot(&[
            "Fix hive group visibility",
            "file_edited:crates/memd-client/src/main.rs",
        ]),
        inbox: test_inbox_snapshot(&["scope=task:hive-awareness"]),
        ..test_resume_snapshot()
    };

    let heartbeat = build_hive_heartbeat(&dir.join(".memd"), Some(&snapshot))
        .expect("build heartbeat");

    assert_eq!(heartbeat.working.as_deref(), Some("Fix hive group visibility"));
    assert_eq!(
        heartbeat.touches,
        vec![
            "file:crates/memd-client/src/main.rs".to_string(),
            "task:hive-awareness".to_string(),
        ]
    );
}
```

- [ ] **Step 2: Run the heartbeat test to verify it fails**

Run:

```bash
cargo test -q -p memd-client --bin memd build_hive_heartbeat_sets_working_and_normalized_touches -- --nocapture
```

Expected:

- failure because `BundleHeartbeatState` and `build_hive_heartbeat` do not yet publish canonical `working` and normalized `touches`

- [ ] **Step 3: Add canonical helper functions**

In `crates/memd-client/src/main.rs`, introduce focused helpers near the existing awareness helpers:

```rust
fn normalize_hive_touch(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(path) = trimmed.strip_prefix("file:") {
        return Some(format!("file:{}", path.trim()));
    }
    if let Some(path) = trimmed.strip_prefix("file_edited:") {
        return Some(format!("file:{}", path.trim()));
    }
    if let Some(task) = trimmed.strip_prefix("task:") {
        return Some(format!("task:{}", task.trim()));
    }
    if let Some(topic) = trimmed.strip_prefix("topic:") {
        return Some(format!("topic:{}", topic.trim()));
    }
    Some(trimmed.to_string())
}

fn derive_hive_working(
    focus: Option<&str>,
    next_recovery: Option<&str>,
    pressure: Option<&str>,
    task_title: Option<&str>,
) -> Option<String> {
    task_title
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| compact_inline(value, 120))
        .or_else(|| focus.and_then(simplify_awareness_work_text))
        .or_else(|| next_recovery.and_then(simplify_awareness_work_text))
        .or_else(|| pressure.and_then(simplify_awareness_work_text))
}
```

- [ ] **Step 4: Extend `BundleHeartbeatState` and heartbeat build path**

Update `BundleHeartbeatState`:

```rust
struct BundleHeartbeatState {
    // existing fields ...
    working: Option<String>,
    touches: Vec<String>,
    // existing fields ...
}
```

Then wire canonical state in `build_hive_heartbeat`:

```rust
let touches = derive_hive_touches(
    claims_state.as_ref(),
    focus.as_deref(),
    pressure.as_deref(),
    next_recovery.as_deref(),
);
let working = derive_hive_working(
    focus.as_deref(),
    next_recovery.as_deref(),
    pressure.as_deref(),
    None,
);

Ok(BundleHeartbeatState {
    // existing fields ...
    working,
    touches,
    topic_claim,
    scope_claims,
    // existing fields ...
})
```

And in `enrich_hive_heartbeat_with_runtime_intent`:

```rust
if let Some(task) = current_task {
    if state.working.as_deref().is_none_or(|value| value.trim().is_empty()) {
        state.working = Some(compact_inline(&task.title, 120));
    }
    for scope in &task.claim_scopes {
        if let Some(normalized) = normalize_hive_touch(scope) {
            push_unique_touch_point(&mut state.touches, &normalized);
        }
    }
}
```

- [ ] **Step 5: Keep legacy compatibility by backfilling from old fields**

In `project_awareness_entry_to_hive_session`:

```rust
let working = entry
    .working
    .clone()
    .or_else(|| entry.topic_claim.clone())
    .or_else(|| Some(awareness_work_quickview(entry)));
let touches = if entry.touches.is_empty() {
    awareness_touch_points(entry)
        .into_iter()
        .filter_map(|value| normalize_hive_touch(&value))
        .collect::<Vec<_>>()
} else {
    entry.touches.clone()
};
```

Use those values in the returned `HiveSessionRecord`.

- [ ] **Step 6: Run the focused tests**

Run:

```bash
cargo test -q -p memd-client --bin memd build_hive_heartbeat_sets_working_and_normalized_touches -- --nocapture
cargo test -q -p memd-client --bin memd enrich_hive_heartbeat_with_runtime_intent_prefers_owned_task_state -- --nocapture
```

Expected:

- both pass

- [ ] **Step 7: Commit the canonical publish path**

```bash
git add crates/memd-client/src/main.rs
git commit -m "feat: publish canonical hive working state"
```

## Task 3: Compute Relationship State In The Roster Path

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write a failing test for `near` relationship detection**

Add:

```rust
#[test]
fn annotate_hive_relationships_marks_near_for_related_touches() {
    let bees = vec![
        HiveSessionRecord {
            session: "memd".to_string(),
            worker_name: Some("Memd".to_string()),
            working: Some("fixing hive group visibility".to_string()),
            touches: vec![
                "file:crates/memd-client/src/main.rs".to_string(),
                "topic:session-merge".to_string(),
            ],
            ..test_hive_session_record("memd")
        },
        HiveSessionRecord {
            session: "clawcontrol".to_string(),
            worker_name: Some("Clawcontrol".to_string()),
            working: Some("adjusting session merge semantics".to_string()),
            touches: vec![
                "area:coordination-runtime".to_string(),
                "topic:session-merge".to_string(),
            ],
            ..test_hive_session_record("clawcontrol")
        },
    ];

    let annotated = annotate_hive_relationships(bees, Some("memd"));
    let current = annotated
        .into_iter()
        .find(|bee| bee.session == "memd")
        .expect("current bee");

    assert_eq!(current.relationship_state.as_deref(), Some("near"));
    assert_eq!(current.relationship_peer.as_deref(), Some("Clawcontrol"));
    assert_eq!(current.suggested_action.as_deref(), Some("cowork"));
}
```

- [ ] **Step 2: Run the relationship test to verify it fails**

Run:

```bash
cargo test -q -p memd-client --bin memd annotate_hive_relationships_marks_near_for_related_touches -- --nocapture
```

Expected:

- failure because relationship derivation does not yet exist

- [ ] **Step 3: Add a small relationship annotation function**

In `crates/memd-client/src/main.rs`:

```rust
fn annotate_hive_relationships(
    bees: Vec<HiveSessionRecord>,
    current_session: Option<&str>,
) -> Vec<HiveSessionRecord> {
    let Some(current_session) = current_session else {
        return bees;
    };

    let snapshot = bees.clone();
    bees.into_iter()
        .map(|mut bee| {
            if bee.session != current_session {
                return bee;
            }

            let mut best: Option<(String, String, String, String)> = None;
            for peer in snapshot.iter().filter(|peer| peer.session != bee.session) {
                if let Some((state, reason, action)) =
                    derive_hive_relationship(&bee, peer)
                {
                    best = Some((
                        state,
                        hive_actor_label(
                            peer.display_name.as_deref(),
                            peer.worker_name.as_deref(),
                            peer.agent.as_deref(),
                            Some(peer.session.as_str()),
                        ),
                        reason,
                        action,
                    ));
                    break;
                }
            }

            if let Some((state, peer, reason, action)) = best {
                bee.relationship_state = Some(state);
                bee.relationship_peer = Some(peer);
                bee.relationship_reason = Some(reason);
                bee.suggested_action = Some(action);
            } else {
                bee.relationship_state = Some("clear".to_string());
                bee.suggested_action = Some("continue".to_string());
            }

            bee
        })
        .collect()
}
```

- [ ] **Step 4: Implement `derive_hive_relationship` with exact-first severity**

```rust
fn derive_hive_relationship(
    current: &HiveSessionRecord,
    peer: &HiveSessionRecord,
) -> Option<(String, String, String)> {
    let current_touches = current.touches.iter().collect::<std::collections::BTreeSet<_>>();
    let peer_touches = peer.touches.iter().collect::<std::collections::BTreeSet<_>>();

    let exact = current_touches
        .intersection(&peer_touches)
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
    if !exact.is_empty() {
        return Some((
            "conflict".to_string(),
            format!("shared touch {}", exact.join(",")),
            "stop_and_cowork".to_string(),
        ));
    }

    let current_topics = current
        .touches
        .iter()
        .filter(|value| value.starts_with("topic:") || value.starts_with("area:"))
        .collect::<std::collections::BTreeSet<_>>();
    let peer_topics = peer
        .touches
        .iter()
        .filter(|value| value.starts_with("topic:") || value.starts_with("area:"))
        .collect::<std::collections::BTreeSet<_>>();
    let nearby = current_topics
        .intersection(&peer_topics)
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
    if !nearby.is_empty() {
        return Some((
            "near".to_string(),
            format!("shared area {}", nearby.join(",")),
            "cowork".to_string(),
        ));
    }

    if current.needs_help || current.needs_review {
        return Some((
            "blocked".to_string(),
            "waiting on peer coordination".to_string(),
            "handoff_or_review".to_string(),
        ));
    }

    None
}
```

- [ ] **Step 5: Apply relationship annotation in `run_hive_roster_command`**

Replace:

```rust
bees: visible_entries
    .into_iter()
    .map(project_awareness_entry_to_hive_session)
    .collect(),
```

With:

```rust
let bees = visible_entries
    .into_iter()
    .map(project_awareness_entry_to_hive_session)
    .collect::<Vec<_>>();
let bees = annotate_hive_relationships(
    bees,
    runtime.as_ref().and_then(|config| config.session.as_deref()),
);

bees,
```

- [ ] **Step 6: Run focused tests**

Run:

```bash
cargo test -q -p memd-client --bin memd annotate_hive_relationships_marks_near_for_related_touches -- --nocapture
cargo test -q -p memd-client --bin memd run_hive_roster_command_uses_group_awareness_for_cross_project_members -- --nocapture
```

Expected:

- both pass

- [ ] **Step 7: Commit the relationship engine slice**

```bash
git add crates/memd-client/src/main.rs
git commit -m "feat: derive live hive cowork relationships"
```

## Task 4: Surface The New Live Cowork Signal In Roster Output

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Update roster rendering to show work, touches, and relation**

Replace the existing roster line shape with:

```rust
lines.push(format!(
    "- {} ({}) role={} lane={} task={} work=\"{}\" touches={} relation={} action={} status={}",
    worker,
    bee.session,
    bee.role
        .as_deref()
        .or(bee.hive_role.as_deref())
        .unwrap_or("worker"),
    lane,
    bee.task_id.as_deref().unwrap_or("none"),
    bee.working.as_deref().unwrap_or("none"),
    if bee.touches.is_empty() {
        "none".to_string()
    } else {
        bee.touches.join(",")
    },
    match (
        bee.relationship_state.as_deref(),
        bee.relationship_peer.as_deref(),
    ) {
        (Some(state), Some(peer)) => format!("{state}:{peer}"),
        (Some(state), None) => state.to_string(),
        _ => "clear".to_string(),
    },
    bee.suggested_action.as_deref().unwrap_or("continue"),
    bee.status,
));
```

- [ ] **Step 2: Add a regression for conflict output**

```rust
#[test]
fn render_hive_roster_summary_surfaces_conflict_action() {
    let response = HiveRosterResponse {
        project: "memd".to_string(),
        namespace: "main".to_string(),
        queen_session: None,
        bees: vec![HiveSessionRecord {
            session: "memd".to_string(),
            worker_name: Some("Memd".to_string()),
            working: Some("editing parser lane".to_string()),
            touches: vec!["file:crates/memd-client/src/main.rs".to_string()],
            relationship_state: Some("conflict".to_string()),
            relationship_peer: Some("Clawcontrol".to_string()),
            suggested_action: Some("stop_and_cowork".to_string()),
            status: "live".to_string(),
            ..test_hive_session_record("memd")
        }],
    };

    let summary = render_hive_roster_summary(&response);
    assert!(summary.contains("relation=conflict:Clawcontrol"));
    assert!(summary.contains("action=stop_and_cowork"));
}
```

- [ ] **Step 3: Run the roster tests**

Run:

```bash
cargo test -q -p memd-client --bin memd render_hive_roster_summary_surfaces_working_touches_and_relationship -- --nocapture
cargo test -q -p memd-client --bin memd render_hive_roster_summary_surfaces_conflict_action -- --nocapture
```

Expected:

- both pass

- [ ] **Step 4: Run compile coverage for the full client binary**

Run:

```bash
cargo test -q -p memd-client --bin memd --no-run
```

Expected:

- exit code `0`

- [ ] **Step 5: Commit the roster UX slice**

```bash
git add crates/memd-client/src/main.rs
git commit -m "feat: surface live cowork state in hive roster"
```

## Spec Coverage Check

- canonical live `working`: covered by Task 2
- normalized `touches`: covered by Task 2
- `relationship_state`: covered by Task 3
- roster visibility of live cowork signal: covered by Task 4
- backward-compatible cross-project hive-group visibility: protected by existing and rerun roster/awareness tests in Tasks 3 and 4

## Final Verification Commands

Run:

```bash
cargo test -q -p memd-client --bin memd build_hive_heartbeat_sets_working_and_normalized_touches -- --nocapture
cargo test -q -p memd-client --bin memd annotate_hive_relationships_marks_near_for_related_touches -- --nocapture
cargo test -q -p memd-client --bin memd render_hive_roster_summary_surfaces_working_touches_and_relationship -- --nocapture
cargo test -q -p memd-client --bin memd render_hive_roster_summary_surfaces_conflict_action -- --nocapture
cargo test -q -p memd-client --bin memd run_hive_roster_command_uses_group_awareness_for_cross_project_members -- --nocapture
cargo test -q -p memd-client --bin memd --no-run
```

Expected:

- all commands pass

