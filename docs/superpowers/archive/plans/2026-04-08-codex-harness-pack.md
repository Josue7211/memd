# Codex Harness Pack Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship the first memd harness pack for Codex CLI with pre-turn recall, post-turn capture, turn-scoped cache reuse, and visible bundle handoff files.

**Architecture:** Add a small reusable Rust pack manifest for Codex inside `memd-client`, then wire it into the existing wake/resume/checkpoint/hook-capture flow. The pack should orchestrate compiled memory, event refresh, and capture writes without becoming a second source of truth. Generated shell helpers stay thin; the Rust manifest owns the behavior shape so later OpenClaw, Claude, and Hermes packs can reuse the same model.

**Tech Stack:** Rust, existing memd bundle helpers, generated shell scripts, existing integration docs, existing test suite.

---

### Task 1: Add the Codex pack manifest and renderer

**Files:**
- Create: `crates/memd-client/src/harness/mod.rs`
- Create: `crates/memd-client/src/harness/codex.rs`
- Modify: `crates/memd-client/src/main.rs`
- Modify: `crates/memd-client/src/render.rs`
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn codex_pack_manifest_exposes_recall_capture_cache_and_files() {
    let bundle_root = std::env::temp_dir().join("memd-codex-pack-test");
    let manifest = build_codex_harness_pack(&bundle_root, "demo", "main");

    assert_eq!(manifest.agent, "codex");
    assert!(manifest
        .files
        .iter()
        .any(|path| path.ends_with("CODEX_WAKEUP.md")));
    assert!(manifest
        .files
        .iter()
        .any(|path| path.ends_with("CODEX_MEMORY.md")));
    assert!(manifest
        .commands
        .iter()
        .any(|cmd| cmd.contains("memd wake --output .memd --intent current_task --write")));
    assert!(manifest
        .commands
        .iter()
        .any(|cmd| cmd.contains("memd hook capture --output .memd")));
    assert!(manifest
        .behaviors
        .iter()
        .any(|line| line.contains("turn-scoped cache")));
}
```

- [ ] **Step 2: Run the test and verify it fails**

Run:

```bash
cargo test -p memd-client codex_pack_manifest_exposes_recall_capture_cache_and_files -- --exact
```

Expected: fail because `build_codex_harness_pack` and the Codex manifest types do not exist yet.

- [ ] **Step 3: Write the minimal implementation**

```rust
// crates/memd-client/src/harness/codex.rs
use std::path::{Path, PathBuf};

pub struct CodexHarnessPack {
    pub agent: String,
    pub project: String,
    pub namespace: String,
    pub bundle_root: PathBuf,
    pub files: Vec<PathBuf>,
    pub commands: Vec<String>,
    pub behaviors: Vec<String>,
}

pub fn build_codex_harness_pack(
    bundle_root: &Path,
    project: &str,
    namespace: &str,
) -> CodexHarnessPack {
    CodexHarnessPack {
        agent: "codex".to_string(),
        project: project.to_string(),
        namespace: namespace.to_string(),
        bundle_root: bundle_root.to_path_buf(),
        files: vec![
            bundle_root.join("MEMD_WAKEUP.md"),
            bundle_root.join("MEMD_MEMORY.md"),
            bundle_root.join("agents").join("CODEX_WAKEUP.md"),
            bundle_root.join("agents").join("CODEX_MEMORY.md"),
        ],
        commands: vec![
            "memd wake --output .memd --intent current_task --write".to_string(),
            "memd resume --output .memd --intent current_task".to_string(),
            "memd hook capture --output .memd --stdin --summary".to_string(),
        ],
        behaviors: vec![
            "recall before turn".to_string(),
            "capture after turn".to_string(),
            "turn-scoped cache".to_string(),
        ],
    }
}
```

- [ ] **Step 4: Run the test and verify it passes**

Run:

```bash
cargo test -p memd-client codex_pack_manifest_exposes_recall_capture_cache_and_files -- --exact
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/memd-client/src/harness/mod.rs crates/memd-client/src/harness/codex.rs crates/memd-client/src/main.rs crates/memd-client/src/render.rs
git commit -m "feat: add codex harness pack manifest"
```

### Task 2: Wire recall, capture, and turn cache into the Codex pack flow

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Modify: `integrations/hooks/memd-context.sh`
- Modify: `integrations/hooks/memd-capture.sh`
- Modify: `integrations/hooks/README.md`
- Modify: `integrations/codex/README.md`
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[tokio::test]
async fn codex_pack_refreshes_wakeup_and_memory_files_after_capture() {
    let bundle_root = std::env::temp_dir().join("memd-codex-refresh-test");
    let manifest = build_codex_harness_pack(&bundle_root, "demo", "main");
    let snapshot = ResumeSnapshot::test_fixture("demo", "main", "codex");

    let written = refresh_codex_pack_files(&bundle_root, &snapshot, &manifest)
        .await
        .expect("refresh codex pack files");

    assert!(written
        .iter()
        .any(|path| path.ends_with("agents/CODEX_WAKEUP.md")));
    assert!(written
        .iter()
        .any(|path| path.ends_with("agents/CODEX_MEMORY.md")));
}

#[test]
fn codex_pack_turn_key_is_stable_for_repeated_recall() {
    let first = MemoryCache::makeTurnKey("demo", Some("codex"), "full", "What did we decide?");
    let second = MemoryCache::makeTurnKey("demo", Some("codex"), "full", "What did we decide?");

    assert_eq!(first, second);
}
```

- [ ] **Step 2: Run the test and verify it fails**

Run:

```bash
cargo test -p memd-client codex_pack_refreshes_wakeup_and_memory_files_after_capture -- --exact
```

Expected: fail because the Codex refresh helper does not exist yet.

- [ ] **Step 3: Write the minimal implementation**

```rust
// crates/memd-client/src/harness/codex.rs
pub async fn refresh_codex_pack_files(
    bundle_root: &Path,
    snapshot: &ResumeSnapshot,
    manifest: &CodexHarnessPack,
) -> anyhow::Result<Vec<PathBuf>> {
    let mut written = Vec::new();

    write_bundle_memory_files(bundle_root, snapshot, None, false).await?;
    written.push(bundle_root.join("MEMD_WAKEUP.md"));
    written.push(bundle_root.join("MEMD_MEMORY.md"));
    written.push(bundle_root.join("agents").join("CODEX_WAKEUP.md"));
    written.push(bundle_root.join("agents").join("CODEX_MEMORY.md"));

    let _ = manifest;
    Ok(written)
}

// crates/memd-client/src/main.rs
impl ResumeSnapshot {
    #[cfg(test)]
    fn test_fixture(project: &str, namespace: &str, agent: &str) -> Self {
        Self {
            project: Some(project.to_string()),
            namespace: Some(namespace.to_string()),
            agent: Some(agent.to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            context: memd_schema::CompactContextResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::CurrentTask,
                retrieval_order: vec![memd_schema::MemoryScope::Project],
                records: Vec::new(),
            },
            working: memd_schema::WorkingMemoryResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::CurrentTask,
                retrieval_order: vec![memd_schema::MemoryScope::Project],
                budget_chars: 1600,
                used_chars: 60,
                remaining_chars: 1540,
                truncated: false,
                policy: memd_schema::WorkingMemoryPolicyState {
                    admission_limit: 8,
                    max_chars_per_item: 220,
                    budget_chars: 1600,
                    rehydration_limit: 4,
                },
                records: vec![memd_schema::CompactMemoryRecord {
                    id: uuid::Uuid::new_v4(),
                    record: "Follow the active current-task lane".to_string(),
                }],
                evicted: Vec::new(),
                rehydration_queue: vec![memd_schema::MemoryRehydrationRecord {
                    id: None,
                    kind: "source".to_string(),
                    label: "handoff".to_string(),
                    summary: "Reload the shared workspace handoff".to_string(),
                    reason: None,
                    source_agent: None,
                    source_system: None,
                    source_path: None,
                    source_quality: None,
                    recorded_at: None,
                }],
                traces: Vec::new(),
                semantic_consolidation: None,
            },
            inbox: memd_schema::MemoryInboxResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::CurrentTask,
                items: vec![memd_schema::InboxMemoryItem {
                    item: memd_schema::MemoryItem {
                        id: uuid::Uuid::new_v4(),
                        content: "One review item is still open".to_string(),
                        redundancy_key: None,
                        belief_branch: None,
                        preferred: true,
                        kind: memd_schema::MemoryKind::Status,
                        scope: memd_schema::MemoryScope::Project,
                        project: Some(project.to_string()),
                        namespace: Some(namespace.to_string()),
                        workspace: Some("team-alpha".to_string()),
                        visibility: memd_schema::MemoryVisibility::Workspace,
                        source_agent: None,
                        source_system: None,
                        source_path: None,
                        source_quality: None,
                        confidence: 0.8,
                        ttl_seconds: Some(86_400),
                        created_at: chrono::Utc::now(),
                        status: memd_schema::MemoryStatus::Active,
                        stage: memd_schema::MemoryStage::Candidate,
                        last_verified_at: None,
                        supersedes: Vec::new(),
                        updated_at: chrono::Utc::now(),
                        tags: vec!["checkpoint".to_string()],
                    },
                    reasons: vec!["stale".to_string()],
                }],
            },
            workspaces: memd_schema::WorkspaceMemoryResponse {
                workspaces: vec![memd_schema::WorkspaceMemoryRecord {
                    project: Some(project.to_string()),
                    namespace: Some(namespace.to_string()),
                    workspace: Some("team-alpha".to_string()),
                    visibility: memd_schema::MemoryVisibility::Workspace,
                    item_count: 4,
                    active_count: 3,
                    candidate_count: 1,
                    contested_count: 0,
                    source_lane_count: 1,
                    avg_confidence: 0.84,
                    trust_score: 0.91,
                    last_seen_at: None,
                    tags: Vec::new(),
                }],
            },
            sources: memd_schema::SourceMemoryResponse { sources: Vec::new() },
            semantic: None,
            claims: SessionClaimsState::default(),
            recent_repo_changes: vec!["status M crates/memd-client/src/render.rs".to_string()],
            change_summary: vec!["focus -> Follow the active current-task lane".to_string()],
            resume_state_age_minutes: None,
            refresh_recommended: false,
        }
    }
}
```

Then wire the existing Codex startup paths so they call the manifest refresh after:

- `memd wake`
- `memd resume`
- `memd checkpoint`
- `memd hook capture`

The pack should reuse the same turn key that the shared cache already uses, not invent a second cache key.

- [ ] **Step 4: Run the test and verify it passes**

Run:

```bash
cargo test -p memd-client codex_pack_refreshes_wakeup_and_memory_files_after_capture -- --exact
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/memd-client/src/main.rs crates/memd-client/src/harness/mod.rs crates/memd-client/src/harness/codex.rs integrations/hooks/memd-context.sh integrations/hooks/memd-capture.sh integrations/hooks/README.md integrations/codex/README.md
git commit -m "feat: wire codex pack recall and capture"
```

### Task 3: Document and smoke-test the Codex pack

**Files:**
- Modify: `docs/core/setup.md`
- Modify: `docs/core/api.md`
- Modify: `integrations/codex/README.md`
- Modify: `integrations/hooks/README.md`
- Modify: `docs/reference/oss-positioning.md`
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn codex_pack_docs_point_at_compiled_memory_and_hooks() {
    let markdown = render_codex_harness_pack_markdown(&build_codex_harness_pack(
        &std::env::temp_dir().join("memd-codex-docs-test"),
        "demo",
        "main",
    ));

    assert!(markdown.contains("CODEX_WAKEUP.md"));
    assert!(markdown.contains("CODEX_MEMORY.md"));
    assert!(markdown.contains("recall before turn"));
    assert!(markdown.contains("capture after turn"));
    assert!(markdown.contains("turn-scoped cache"));
}
```

- [ ] **Step 2: Run the test and verify it fails**

Run:

```bash
cargo test -p memd-client codex_pack_docs_point_at_compiled_memory_and_hooks -- --exact
```

Expected: fail until the Codex markdown renderer is wired.

- [ ] **Step 3: Write the minimal implementation**

```rust
// crates/memd-client/src/render.rs
pub fn render_codex_harness_pack_markdown(pack: &CodexHarnessPack) -> String {
    format!(
        "# Codex Harness Pack\n\n\
         - agent: `{}`\n\
         - project: `{}`\n\
         - namespace: `{}`\n\n\
         ## Files\n{}\n\n\
         ## Behavior\n{}\n\n\
         ## Commands\n{}\n",
        pack.agent,
        pack.project,
        pack.namespace,
        pack.files
            .iter()
            .map(|path| format!("- `{}`", path.display()))
            .collect::<Vec<_>>()
            .join("\n"),
        pack.behaviors
            .iter()
            .map(|line| format!("- {}", line))
            .collect::<Vec<_>>()
            .join("\n"),
        pack.commands
            .iter()
            .map(|line| format!("- `{}`", line))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}
```

Then update the Codex and hooks docs so they:

- point at `.memd/MEMD_WAKEUP.md` and `.memd/MEMD_MEMORY.md`
- explain the Codex pack order
- explain that recall runs before the turn and capture runs after the turn
- explain that turn-scoped cache reuse keeps repeated calls cheap

- [ ] **Step 4: Run the test and verify it passes**

Run:

```bash
cargo test -p memd-client codex_pack_docs_point_at_compiled_memory_and_hooks -- --exact
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/memd-client/src/render.rs docs/core/setup.md docs/core/api.md integrations/codex/README.md integrations/hooks/README.md docs/reference/oss-positioning.md
git commit -m "docs: add codex harness pack workflow"
```

## Validation Checklist

After the three tasks land, verify:

- `cargo test -p memd-client --quiet`
- `cargo test --workspace --quiet`
- `cargo run -p memd-client --bin memd -- wake --output .memd --intent current_task --write`
- `cargo run -p memd-client --bin memd -- resume --output .memd --intent current_task`
- `cargo run -p memd-client --bin memd -- hook capture --output .memd --stdin --summary < /tmp/codex-turn.txt`

Expected bundle output:

- `.memd/MEMD_WAKEUP.md`
- `.memd/MEMD_MEMORY.md`
- `.memd/agents/CODEX_WAKEUP.md`
- `.memd/agents/CODEX_MEMORY.md`
- refreshed compiled memory pages under `.memd/compiled/memory/`
- refreshed live event pages under `.memd/compiled/events/`

## Coverage Map

- Codex pack bootstrap and file layout: Task 1
- pre-turn recall path: Task 2
- post-turn capture path: Task 2
- turn-scoped cache reuse: Task 2
- docs and smoke tests: Task 3

## Notes For The Implementer

- Keep the pack local-first.
- Do not add a cloud-only dependency path.
- Reuse the existing `memd` bundle truth surfaces instead of inventing a second
  memory store.
- Prefer the shared Rust helpers over shell-script-only behavior so OpenClaw,
  Claude, and Hermes can later reuse the same pack shape.
