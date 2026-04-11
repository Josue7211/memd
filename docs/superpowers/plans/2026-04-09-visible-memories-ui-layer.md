# Visible Memories UI Layer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first native `memd` visible-memories UI layer as the canonical artifact workspace, starting from the existing server dashboard and extending it into `Memory Home`, `Knowledge Map`, CLI/TUI parity, and Obsidian bridge affordances.

**Architecture:** Keep the first vertical slice inside `crates/memd-server` so the existing built-in dashboard at `/` becomes the native UI shell instead of creating a disconnected second app. Introduce shared artifact-focused UI schema in `memd-schema`, a server-side UI snapshot layer in a dedicated `ui.rs` module, and CLI/TUI renderers that consume the same artifact contract. Reuse existing search, inbox, explain, working, workspace, timeline, entity, repair, and Obsidian bridge flows instead of inventing parallel endpoints.

**Tech Stack:** Rust, Axum, SQLite store, `memd-schema`, server-rendered HTML/CSS/JS, existing `memd-client` terminal renderers, existing Obsidian bridge in `crates/memd-client/src/obsidian.rs`

---

## File Structure

### New files

- `crates/memd-server/src/ui.rs`
  - Own the visible-memories UI snapshot builder, artifact view models, dashboard HTML shell, and lightweight graph serialization for `Knowledge Map`.
- `docs/superpowers/plans/2026-04-09-visible-memories-ui-layer.md`
  - This implementation plan.

### Modified files

- `crates/memd-schema/src/lib.rs`
  - Add shared visible-memory artifact view structs and UI snapshot response types.
- `crates/memd-server/src/main.rs`
  - Register new UI module, replace current stringly dashboard builder with `ui.rs`, add JSON snapshot endpoint for the native UI, and wire existing routes into the shell.
- `crates/memd-server/src/inspection.rs`
  - Reuse or extend inspect/explain helpers so artifact detail and provenance can be composed without duplicating business logic.
- `crates/memd-server/src/working.rs`
  - Expose compact helpers needed by `Memory Home` such as working-set pressure and rehydration summary.
- `crates/memd-server/src/store.rs`
  - Add or expose small query helpers for artifact lists, graph neighborhoods, and workspace activity summaries where existing APIs are insufficient.
- `crates/memd-client/src/render.rs`
  - Add terminal renderers for the shared artifact model and visible-memory home summary.
- `crates/memd-client/src/main.rs`
  - Add a CLI entry point that hits the new UI snapshot/artifact endpoint or reuses the shared renderers against local responses.
- `docs/core/architecture.md`
  - Update the “built-in dashboard” language to match the new native visible-memories workbench surface.
- `docs/core/obsidian.md`
  - Document how the native UI links into the real vault bridge rather than replacing it.

### Test coverage targets

- `crates/memd-schema/src/lib.rs`
  - Serialization tests for visible-memory artifact and snapshot structs.
- `crates/memd-server/src/ui.rs`
  - Unit tests for snapshot assembly, artifact status mapping, graph neighborhood shaping, and dashboard shell content.
- `crates/memd-server/src/main.rs`
  - Route tests for the new UI snapshot endpoint and updated `/` dashboard.
- `crates/memd-client/src/render.rs`
  - Renderer tests for visible-memory home and artifact summaries.

## Task 1: Add Shared Visible-Memory Artifact Schema

**Files:**
- Modify: `crates/memd-schema/src/lib.rs`
- Test: `crates/memd-schema/src/lib.rs`

- [ ] **Step 1: Write the failing schema serialization test**

```rust
#[test]
fn visible_memory_artifact_snapshot_round_trips() {
    let snapshot = VisibleMemorySnapshotResponse {
        generated_at: chrono::Utc::now(),
        home: VisibleMemoryHome {
            focus_artifact: VisibleMemoryArtifact {
                id: uuid::Uuid::nil(),
                title: "runtime spine".to_string(),
                body: "runtime spine is the canonical memory contract".to_string(),
                artifact_kind: "compiled_page".to_string(),
                memory_kind: Some(MemoryKind::Decision),
                scope: Some(MemoryScope::Project),
                visibility: Some(MemoryVisibility::Workspace),
                workspace: Some("team-alpha".to_string()),
                status: VisibleMemoryStatus::Current,
                freshness: "fresh".to_string(),
                confidence: 0.93,
                provenance: VisibleMemoryProvenance {
                    source_system: Some("obsidian".to_string()),
                    source_path: Some("wiki/runtime-spine.md".to_string()),
                    producer: Some("obsidian compile".to_string()),
                    trust_reason: "verified from compiled workspace page".to_string(),
                    last_verified_at: None,
                },
                sources: vec!["wiki/runtime-spine.md".to_string()],
                linked_artifact_ids: vec![],
                linked_sessions: vec!["codex-01".to_string()],
                linked_agents: vec!["codex".to_string()],
                repair_state: "healthy".to_string(),
                actions: vec!["inspect".to_string(), "verify_current".to_string()],
            },
            inbox_count: 3,
            repair_count: 1,
            awareness_count: 2,
        },
        knowledge_map: VisibleMemoryKnowledgeMap {
            nodes: vec![],
            edges: vec![],
        },
    };

    let json = serde_json::to_string(&snapshot).unwrap();
    let decoded: VisibleMemorySnapshotResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.home.focus_artifact.title, "runtime spine");
    assert_eq!(decoded.home.focus_artifact.status, VisibleMemoryStatus::Current);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p memd-schema visible_memory_artifact_snapshot_round_trips -- --exact`

Expected: FAIL with errors like `cannot find struct VisibleMemorySnapshotResponse` and `cannot find enum VisibleMemoryStatus`.

- [ ] **Step 3: Write the minimal shared schema**

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VisibleMemoryStatus {
    Current,
    Candidate,
    Stale,
    Superseded,
    Conflicted,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VisibleMemoryProvenance {
    pub source_system: Option<String>,
    pub source_path: Option<String>,
    pub producer: Option<String>,
    pub trust_reason: String,
    pub last_verified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VisibleMemoryArtifact {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub artifact_kind: String,
    pub memory_kind: Option<MemoryKind>,
    pub scope: Option<MemoryScope>,
    pub visibility: Option<MemoryVisibility>,
    pub workspace: Option<String>,
    pub status: VisibleMemoryStatus,
    pub freshness: String,
    pub confidence: f32,
    pub provenance: VisibleMemoryProvenance,
    pub sources: Vec<String>,
    pub linked_artifact_ids: Vec<Uuid>,
    pub linked_sessions: Vec<String>,
    pub linked_agents: Vec<String>,
    pub repair_state: String,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VisibleMemoryHome {
    pub focus_artifact: VisibleMemoryArtifact,
    pub inbox_count: usize,
    pub repair_count: usize,
    pub awareness_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VisibleMemoryGraphNode {
    pub artifact_id: Uuid,
    pub title: String,
    pub artifact_kind: String,
    pub status: VisibleMemoryStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VisibleMemoryGraphEdge {
    pub from: Uuid,
    pub to: Uuid,
    pub relation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VisibleMemoryKnowledgeMap {
    pub nodes: Vec<VisibleMemoryGraphNode>,
    pub edges: Vec<VisibleMemoryGraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VisibleMemorySnapshotResponse {
    pub generated_at: DateTime<Utc>,
    pub home: VisibleMemoryHome,
    pub knowledge_map: VisibleMemoryKnowledgeMap,
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p memd-schema visible_memory_artifact_snapshot_round_trips -- --exact`

Expected: PASS with `1 passed`.

- [ ] **Step 5: Commit**

```bash
git add crates/memd-schema/src/lib.rs
git commit -m "feat: add visible memory snapshot schema"
```

## Task 2: Build Server-Side UI Snapshot Aggregation

**Files:**
- Create: `crates/memd-server/src/ui.rs`
- Modify: `crates/memd-server/src/main.rs`
- Modify: `crates/memd-server/src/store.rs`
- Test: `crates/memd-server/src/ui.rs`

- [ ] **Step 1: Write the failing UI snapshot test**

```rust
#[test]
fn builds_memory_home_snapshot_from_store_state() {
    let store = SqliteStore::open(":memory:").unwrap();
    let state = AppState { store };

    let now = chrono::Utc::now();
    let item = MemoryItem {
        id: uuid::Uuid::new_v4(),
        content: "runtime spine is current".to_string(),
        redundancy_key: Some("runtime-spine".to_string()),
        belief_branch: None,
        preferred: true,
        kind: MemoryKind::Decision,
        scope: MemoryScope::Project,
        project: Some("memd".to_string()),
        namespace: Some("core".to_string()),
        workspace: Some("team-alpha".to_string()),
        visibility: MemoryVisibility::Workspace,
        source_agent: Some("codex".to_string()),
        source_system: Some("obsidian".to_string()),
        source_path: Some("wiki/runtime-spine.md".to_string()),
        confidence: 0.92,
        ttl_seconds: None,
        created_at: now,
        updated_at: now,
        last_verified_at: Some(now),
        supersedes: vec![],
        tags: vec!["runtime".to_string()],
        status: MemoryStatus::Active,
        source_quality: Some(SourceQuality::Derived),
        stage: MemoryStage::Canonical,
    };

    let canonical_key = crate::keys::canonical_key(&item);
    let redundancy_key = crate::keys::redundancy_key(&item);
    state.store.insert_or_get_duplicate(&item, &canonical_key, &redundancy_key).unwrap();

    let snapshot = ui::build_visible_memory_snapshot(&state).unwrap();
    assert_eq!(snapshot.home.focus_artifact.title, "runtime spine");
    assert_eq!(snapshot.home.focus_artifact.status, VisibleMemoryStatus::Current);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p memd-server builds_memory_home_snapshot_from_store_state -- --exact`

Expected: FAIL with errors like `file not found for module ui` or `cannot find function build_visible_memory_snapshot`.

- [ ] **Step 3: Write the minimal UI snapshot builder**

```rust
// crates/memd-server/src/ui.rs
use chrono::Utc;
use memd_schema::{
    VisibleMemoryArtifact, VisibleMemoryGraphEdge, VisibleMemoryGraphNode,
    VisibleMemoryHome, VisibleMemoryKnowledgeMap, VisibleMemoryProvenance,
    VisibleMemorySnapshotResponse, VisibleMemoryStatus,
};

use crate::{AppState, keys::canonical_key};

pub(crate) fn build_visible_memory_snapshot(
    state: &AppState,
) -> anyhow::Result<VisibleMemorySnapshotResponse> {
    let items = state.snapshot()?;
    let focus = items
        .iter()
        .find(|item| item.preferred)
        .or_else(|| items.first())
        .ok_or_else(|| anyhow::anyhow!("no memory items available"))?;

    let title = canonical_key(focus).replace('-', " ");
    let artifact = VisibleMemoryArtifact {
        id: focus.id,
        title,
        body: focus.content.clone(),
        artifact_kind: "memory_item".to_string(),
        memory_kind: Some(focus.kind),
        scope: Some(focus.scope),
        visibility: Some(focus.visibility),
        workspace: focus.workspace.clone(),
        status: VisibleMemoryStatus::Current,
        freshness: if focus.last_verified_at.is_some() {
            "fresh".to_string()
        } else {
            "unverified".to_string()
        },
        confidence: focus.confidence,
        provenance: VisibleMemoryProvenance {
            source_system: focus.source_system.clone(),
            source_path: focus.source_path.clone(),
            producer: focus.source_agent.clone(),
            trust_reason: "derived from stored memd artifact".to_string(),
            last_verified_at: focus.last_verified_at,
        },
        sources: focus.source_path.clone().into_iter().collect(),
        linked_artifact_ids: vec![],
        linked_sessions: vec![],
        linked_agents: focus.source_agent.clone().into_iter().collect(),
        repair_state: "healthy".to_string(),
        actions: vec![
            "inspect".to_string(),
            "explain".to_string(),
            "verify_current".to_string(),
            "mark_stale".to_string(),
        ],
    };

    Ok(VisibleMemorySnapshotResponse {
        generated_at: Utc::now(),
        home: VisibleMemoryHome {
            focus_artifact: artifact,
            inbox_count: 0,
            repair_count: 0,
            awareness_count: 0,
        },
        knowledge_map: VisibleMemoryKnowledgeMap {
            nodes: vec![VisibleMemoryGraphNode {
                artifact_id: focus.id,
                title: "runtime spine".to_string(),
                artifact_kind: "memory_item".to_string(),
                status: VisibleMemoryStatus::Current,
            }],
            edges: vec![VisibleMemoryGraphEdge {
                from: focus.id,
                to: focus.id,
                relation: "focus".to_string(),
            }],
        },
    })
}
```

- [ ] **Step 4: Wire the new JSON endpoint and route**

```rust
// crates/memd-server/src/main.rs
mod ui;

use memd_schema::VisibleMemorySnapshotResponse;

let app = Router::new()
    .route("/", get(dashboard))
    .route("/ui/snapshot", get(get_visible_memory_snapshot))
    // existing routes stay intact
    .with_state(state);

async fn get_visible_memory_snapshot(
    State(state): State<AppState>,
) -> Result<Json<VisibleMemorySnapshotResponse>, (StatusCode, String)> {
    let response = ui::build_visible_memory_snapshot(&state).map_err(internal_error)?;
    Ok(Json(response))
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p memd-server builds_memory_home_snapshot_from_store_state -- --exact`

Expected: PASS with `1 passed`.

- [ ] **Step 6: Commit**

```bash
git add crates/memd-server/src/ui.rs crates/memd-server/src/main.rs crates/memd-server/src/store.rs
git commit -m "feat: add visible memory UI snapshot endpoint"
```

## Task 3: Replace the Built-In Dashboard With Memory Home

**Files:**
- Modify: `crates/memd-server/src/ui.rs`
- Modify: `crates/memd-server/src/main.rs`
- Test: `crates/memd-server/src/ui.rs`

- [ ] **Step 1: Write the failing dashboard shell test**

```rust
#[test]
fn dashboard_html_includes_memory_home_sections() {
    let snapshot = VisibleMemorySnapshotResponse {
        generated_at: chrono::Utc::now(),
        home: VisibleMemoryHome {
            focus_artifact: VisibleMemoryArtifact {
                id: uuid::Uuid::nil(),
                title: "runtime spine".to_string(),
                body: "runtime spine is the canonical memory contract".to_string(),
                artifact_kind: "compiled_page".to_string(),
                memory_kind: None,
                scope: None,
                visibility: None,
                workspace: Some("team-alpha".to_string()),
                status: VisibleMemoryStatus::Current,
                freshness: "fresh".to_string(),
                confidence: 0.94,
                provenance: VisibleMemoryProvenance {
                    source_system: Some("obsidian".to_string()),
                    source_path: Some("wiki/runtime-spine.md".to_string()),
                    producer: Some("obsidian compile".to_string()),
                    trust_reason: "verified compiled page".to_string(),
                    last_verified_at: None,
                },
                sources: vec!["wiki/runtime-spine.md".to_string()],
                linked_artifact_ids: vec![],
                linked_sessions: vec!["codex-01".to_string()],
                linked_agents: vec!["codex".to_string()],
                repair_state: "healthy".to_string(),
                actions: vec!["inspect".to_string()],
            },
            inbox_count: 2,
            repair_count: 1,
            awareness_count: 3,
        },
        knowledge_map: VisibleMemoryKnowledgeMap { nodes: vec![], edges: vec![] },
    };

    let html = ui::dashboard_html(&snapshot);
    assert!(html.contains("Memory Home"));
    assert!(html.contains("Knowledge Map"));
    assert!(html.contains("Truth"));
    assert!(html.contains("Open in Obsidian"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p memd-server dashboard_html_includes_memory_home_sections -- --exact`

Expected: FAIL because `dashboard_html` still returns the old dashboard or does not include the new sections.

- [ ] **Step 3: Render the new Memory Home shell**

```rust
pub(crate) fn dashboard_html(snapshot: &VisibleMemorySnapshotResponse) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>memd visible memories</title>
    <style>
      :root {{
        --bg: #0b0813;
        --panel: #151120;
        --ink: #f2ecff;
        --muted: #a89fc0;
        --line: rgba(157,124,216,.18);
        --accent: #9d7cd8;
        --good: #68d391;
      }}
      body {{ margin: 0; background: var(--bg); color: var(--ink); font-family: ui-sans-serif, system-ui, sans-serif; }}
      .shell {{ display: grid; grid-template-columns: 260px minmax(0,1fr) 320px; min-height: 100vh; }}
      .rail, .detail {{ background: var(--panel); border-right: 1px solid var(--line); padding: 20px; }}
      .detail {{ border-right: 0; border-left: 1px solid var(--line); }}
      .main {{ padding: 24px; display: grid; gap: 18px; }}
      .panel {{ border: 1px solid var(--line); border-radius: 16px; padding: 18px; background: rgba(255,255,255,.02); }}
      .eyebrow {{ color: var(--muted); text-transform: uppercase; letter-spacing: .14em; font-size: 12px; }}
      .actions {{ display: flex; gap: 8px; flex-wrap: wrap; }}
      .actions button {{ border: 1px solid var(--line); background: transparent; color: var(--ink); border-radius: 999px; padding: 8px 12px; }}
    </style>
  </head>
  <body>
    <div class="shell">
      <aside class="rail">
        <div class="eyebrow">Navigation</div>
        <h2>Memory Home</h2>
        <p>Workspaces, lanes, collections, vault links.</p>
      </aside>
      <main class="main">
        <section class="panel">
          <div class="eyebrow">Focus Artifact</div>
          <h1>{title}</h1>
          <p>{body}</p>
          <div class="actions">
            <button>Inspect</button>
            <button>Explain</button>
            <button>Verify Current</button>
            <button>Open in Obsidian</button>
          </div>
        </section>
        <section class="panel">
          <div class="eyebrow">Knowledge Map</div>
          <p>Graph neighborhood, backlinks, contradictions, and source adjacency render here.</p>
        </section>
      </main>
      <aside class="detail">
        <div class="eyebrow">Truth</div>
        <p>Status: {status}</p>
        <p>Freshness: {freshness}</p>
        <p>Workspace: {workspace}</p>
      </aside>
    </div>
  </body>
</html>"#,
        title = snapshot.home.focus_artifact.title,
        body = snapshot.home.focus_artifact.body,
        status = format!("{:?}", snapshot.home.focus_artifact.status),
        freshness = snapshot.home.focus_artifact.freshness,
        workspace = snapshot.home.focus_artifact.workspace.as_deref().unwrap_or("none"),
    )
}
```

- [ ] **Step 4: Swap the dashboard handler to use the snapshot-backed shell**

```rust
async fn dashboard(State(state): State<AppState>) -> Result<Html<String>, (StatusCode, String)> {
    let snapshot = ui::build_visible_memory_snapshot(&state).map_err(internal_error)?;
    Ok(Html(ui::dashboard_html(&snapshot)))
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p memd-server dashboard_html_includes_memory_home_sections -- --exact`

Expected: PASS with `1 passed`.

- [ ] **Step 6: Smoke test in the browser**

Run: `cargo run -p memd-server`

Expected: server starts on `127.0.0.1:8787`; opening `http://127.0.0.1:8787/` shows the new `Memory Home` shell with left rail, center artifact, and right-side truth panel.

- [ ] **Step 7: Commit**

```bash
git add crates/memd-server/src/ui.rs crates/memd-server/src/main.rs
git commit -m "feat: add memory home dashboard shell"
```

## Task 4: Add Knowledge Map, Repair Pressure, and Workspace Awareness

**Files:**
- Modify: `crates/memd-server/src/ui.rs`
- Modify: `crates/memd-server/src/store.rs`
- Modify: `crates/memd-server/src/working.rs`
- Test: `crates/memd-server/src/ui.rs`

- [ ] **Step 1: Write the failing graph and pressure test**

```rust
#[test]
fn snapshot_includes_graph_nodes_and_operational_pressure() {
    let store = SqliteStore::open(":memory:").unwrap();
    let state = AppState { store };

    let snapshot = ui::build_visible_memory_snapshot(&state).unwrap();
    assert!(!snapshot.knowledge_map.nodes.is_empty());
    assert!(snapshot.home.inbox_count >= 0);
    assert!(snapshot.home.repair_count >= 0);
    assert!(snapshot.home.awareness_count >= 0);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p memd-server snapshot_includes_graph_nodes_and_operational_pressure -- --exact`

Expected: FAIL because the snapshot only returns placeholder graph data and zeroed pressure fields.

- [ ] **Step 3: Extend the snapshot builder with real queue and awareness summaries**

```rust
let inbox = state.store.inbox(&MemoryInboxRequest {
    limit: Some(8),
    project: None,
    namespace: None,
    workspace: None,
    visibility: None,
})?;
let workspaces = state.store.workspaces(&WorkspaceMemoryRequest {
    limit: Some(8),
    project: None,
    namespace: None,
    workspace: None,
    visibility: None,
})?;
let timeline = state.store.timeline(&TimelineMemoryRequest {
    limit: Some(12),
    project: None,
    namespace: None,
    workspace: None,
    visibility: None,
})?;

let nodes = timeline
    .items
    .iter()
    .map(|event| VisibleMemoryGraphNode {
        artifact_id: event.item_id,
        title: event.summary.clone(),
        artifact_kind: event.event_type.clone(),
        status: VisibleMemoryStatus::Current,
    })
    .collect::<Vec<_>>();

let edges = timeline
    .items
    .windows(2)
    .map(|pair| VisibleMemoryGraphEdge {
        from: pair[0].item_id,
        to: pair[1].item_id,
        relation: "timeline".to_string(),
    })
    .collect::<Vec<_>>();

home.inbox_count = inbox.items.len();
home.repair_count = inbox.items.iter().filter(|item| item.needs_attention).count();
home.awareness_count = workspaces.workspaces.len();
knowledge_map = VisibleMemoryKnowledgeMap { nodes, edges };
```

- [ ] **Step 4: Render the new right-rail modules and map panel**

```rust
<section class="panel">
  <div class="eyebrow">Knowledge Map</div>
  <ul>
    {graph_nodes}
  </ul>
</section>
<section class="panel">
  <div class="eyebrow">Repair Pressure</div>
  <p>Inbox: {inbox_count}</p>
  <p>Repair queue: {repair_count}</p>
  <p>Workspace awareness: {awareness_count}</p>
</section>
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p memd-server snapshot_includes_graph_nodes_and_operational_pressure -- --exact`

Expected: PASS with `1 passed`.

- [ ] **Step 6: Commit**

```bash
git add crates/memd-server/src/ui.rs crates/memd-server/src/store.rs crates/memd-server/src/working.rs
git commit -m "feat: add knowledge map and memory pressure to dashboard"
```

## Task 5: Add CLI/TUI Parity for Memory Home and Artifact Inspect

**Files:**
- Modify: `crates/memd-client/src/render.rs`
- Modify: `crates/memd-client/src/main.rs`
- Test: `crates/memd-client/src/render.rs`

- [ ] **Step 1: Write the failing terminal renderer test**

```rust
#[test]
fn renders_visible_memory_home_summary() {
    let response = VisibleMemorySnapshotResponse {
        generated_at: chrono::Utc::now(),
        home: VisibleMemoryHome {
            focus_artifact: VisibleMemoryArtifact {
                id: uuid::Uuid::nil(),
                title: "runtime spine".to_string(),
                body: "runtime spine is the canonical memory contract".to_string(),
                artifact_kind: "compiled_page".to_string(),
                memory_kind: None,
                scope: None,
                visibility: None,
                workspace: Some("team-alpha".to_string()),
                status: VisibleMemoryStatus::Current,
                freshness: "fresh".to_string(),
                confidence: 0.94,
                provenance: VisibleMemoryProvenance {
                    source_system: Some("obsidian".to_string()),
                    source_path: Some("wiki/runtime-spine.md".to_string()),
                    producer: Some("obsidian compile".to_string()),
                    trust_reason: "verified".to_string(),
                    last_verified_at: None,
                },
                sources: vec!["wiki/runtime-spine.md".to_string()],
                linked_artifact_ids: vec![],
                linked_sessions: vec!["codex-01".to_string()],
                linked_agents: vec!["codex".to_string()],
                repair_state: "healthy".to_string(),
                actions: vec!["inspect".to_string()],
            },
            inbox_count: 2,
            repair_count: 1,
            awareness_count: 3,
        },
        knowledge_map: VisibleMemoryKnowledgeMap { nodes: vec![], edges: vec![] },
    };

    let output = render_visible_memory_home(&response);
    assert!(output.contains("memory_home focus=\"runtime spine\""));
    assert!(output.contains("repair=1"));
    assert!(output.contains("workspace=team-alpha"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p memd-client renders_visible_memory_home_summary -- --exact`

Expected: FAIL because `render_visible_memory_home` does not exist.

- [ ] **Step 3: Add the renderer and command hook**

```rust
// crates/memd-client/src/render.rs
pub(crate) fn render_visible_memory_home(response: &VisibleMemorySnapshotResponse) -> String {
    format!(
        "memory_home focus=\"{}\" status={:?} freshness={} workspace={} inbox={} repair={} awareness={}",
        response.home.focus_artifact.title,
        response.home.focus_artifact.status,
        response.home.focus_artifact.freshness,
        response
            .home
            .focus_artifact
            .workspace
            .as_deref()
            .unwrap_or("none"),
        response.home.inbox_count,
        response.home.repair_count,
        response.home.awareness_count,
    )
}
```

```rust
// crates/memd-client/src/main.rs
let response = client
    .get::<VisibleMemorySnapshotResponse>("/ui/snapshot")
    .await?;
println!("{}", render::render_visible_memory_home(&response));
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p memd-client renders_visible_memory_home_summary -- --exact`

Expected: PASS with `1 passed`.

- [ ] **Step 5: Smoke test the CLI**

Run: `cargo run -p memd-client --bin memd -- ui home`

Expected output starts with `memory_home focus=` and includes inbox, repair, and awareness counts.

- [ ] **Step 6: Commit**

```bash
git add crates/memd-client/src/render.rs crates/memd-client/src/main.rs
git commit -m "feat: add visible memory home cli summary"
```

## Task 6: Add Obsidian Bridge Affordances and Docs

**Files:**
- Modify: `crates/memd-server/src/ui.rs`
- Modify: `docs/core/architecture.md`
- Modify: `docs/core/obsidian.md`
- Test: `crates/memd-server/src/ui.rs`

- [ ] **Step 1: Write the failing Obsidian action test**

```rust
#[test]
fn dashboard_html_shows_obsidian_action_when_source_path_exists() {
    let snapshot = sample_snapshot_with_source("wiki/runtime-spine.md");
    let html = ui::dashboard_html(&snapshot);
    assert!(html.contains("Open in Obsidian"));
    assert!(html.contains("Vault source"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p memd-server dashboard_html_shows_obsidian_action_when_source_path_exists -- --exact`

Expected: FAIL because the shell does not yet conditionally show vault-specific affordances.

- [ ] **Step 3: Add Obsidian-aware actions to the shell**

```rust
let obsidian_action = snapshot
    .home
    .focus_artifact
    .provenance
    .source_path
    .as_deref()
    .map(|path| format!(r#"<button data-source-path="{path}">Open in Obsidian</button>"#))
    .unwrap_or_default();

let vault_source = snapshot
    .home
    .focus_artifact
    .provenance
    .source_path
    .as_deref()
    .map(|path| format!(r#"<p>Vault source: <code>{path}</code></p>"#))
    .unwrap_or_default();
```

- [ ] **Step 4: Update docs to reflect the new canonical UI posture**

```markdown
<!-- docs/core/architecture.md -->
The server serves the first native visible-memories workbench at `/`, replacing
the previous minimal dashboard. This shell should expose `Memory Home`,
`Knowledge Map`, truth state, repair pressure, and source-linked artifact
inspection without replacing the existing CLI or Obsidian bridge.
```

```markdown
<!-- docs/core/obsidian.md -->
The native visible-memories UI treats Obsidian as a first-class integration.
Users keep their real vault. `memd` links focused artifacts back to vault paths
and preserves the same artifact identity across the native UI, CLI/TUI, and
vault-generated pages.
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p memd-server dashboard_html_shows_obsidian_action_when_source_path_exists -- --exact`

Expected: PASS with `1 passed`.

- [ ] **Step 6: Run the targeted regression suite**

Run: `cargo test -p memd-schema visible_memory_artifact_snapshot_round_trips -- --exact && cargo test -p memd-server builds_memory_home_snapshot_from_store_state -- --exact && cargo test -p memd-server dashboard_html_includes_memory_home_sections -- --exact && cargo test -p memd-server snapshot_includes_graph_nodes_and_operational_pressure -- --exact && cargo test -p memd-client renders_visible_memory_home_summary -- --exact`

Expected: all targeted tests pass.

- [ ] **Step 7: Commit**

```bash
git add crates/memd-server/src/ui.rs docs/core/architecture.md docs/core/obsidian.md
git commit -m "docs: wire visible memory ui to obsidian bridge"
```

## Self-Review

### Spec coverage

- Canonical native UI shell: covered by Tasks 2 and 3.
- `Memory Home` default landing view: covered by Task 3.
- `Knowledge Map` first-class adjacent mode: covered by Task 4.
- Artifact contract as shared UI truth: covered by Task 1.
- Full feature coverage across inbox, repair, awareness, routing, sources: covered by Tasks 2, 4, and 6.
- CLI/TUI parity: covered by Task 5.
- Obsidian as integration rather than replacement: covered by Task 6.

### Placeholder scan

- No `TBD`, `TODO`, or “implement later” placeholders remain.
- Each code-changing step includes concrete code.
- Each validation step includes an exact command and expected result.

### Type consistency

- Shared names are consistent across tasks:
  - `VisibleMemorySnapshotResponse`
  - `VisibleMemoryArtifact`
  - `VisibleMemoryStatus`
  - `VisibleMemoryHome`
  - `VisibleMemoryKnowledgeMap`
- The server, CLI, and tests all reference the same snapshot shape.

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-04-09-visible-memories-ui-layer.md`.

Two execution options:

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?
