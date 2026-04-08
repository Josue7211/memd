# Unified Harness Preset Schema Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace pack-specific drift with one shared harness preset schema so Codex, OpenClaw, Hermes, OpenCode, and Agent Zero are thin policy layers over the same memd core loop.

**Architecture:** Add one schema layer that describes each harness pack as data, then render pack metadata, docs, and generated surfaces from that schema. Keep memd core ownership unchanged: routing, cache, compiled memory, and handoff/resume/capture plumbing stay in the engine, while pack differences are limited to defaults, wording, and entrypoints.

**Tech Stack:** Rust, current `memd-client` harness modules, existing integration markdown, existing docs generation/render pipeline, current test harness.

---

### Task 1: Add a shared harness preset schema and registry

**Files:**
- Create: `crates/memd-client/src/harness/preset.rs`
- Modify: `crates/memd-client/src/harness/mod.rs`
- Modify: `crates/memd-client/src/harness/index.rs`
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write the failing test**

Add a registry test that expects all five packs to come from one shared preset source and asserts the registry exposes stable IDs and defaults:

```rust
assert!(registry.packs.iter().any(|pack| pack.pack_id == "codex"));
assert!(registry.packs.iter().any(|pack| pack.pack_id == "openclaw"));
assert!(registry.packs.iter().any(|pack| pack.pack_id == "hermes"));
assert!(registry.packs.iter().any(|pack| pack.pack_id == "opencode"));
assert!(registry.packs.iter().any(|pack| pack.pack_id == "agent-zero"));
assert_eq!(registry.get("codex").unwrap().default_verbs, vec!["wake", "resume", "checkpoint"]);
```

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
cargo test -p memd-client harness_registry_exposes_shared_preset_ids_and_defaults -- --exact
```

Expected: fail because the schema and registry do not exist yet.

- [ ] **Step 3: Write the minimal implementation**

Create a shared preset type that captures the pack contract as data:

```rust
pub struct HarnessPreset {
    pub pack_id: &'static str,
    pub display_name: &'static str,
    pub entrypoint: &'static str,
    pub surface_set: &'static [&'static str],
    pub default_verbs: &'static [&'static str],
    pub cache_policy: &'static str,
    pub copy_tone: &'static str,
}
```

Then create a registry that returns one preset per harness:

```rust
pub struct HarnessPresetRegistry {
    pub packs: Vec<HarnessPreset>,
}

impl HarnessPresetRegistry {
    pub fn get(&self, pack_id: &str) -> Option<&HarnessPreset> { /* ... */ }
}
```

Keep the first implementation minimal: store the pack differences as data, not as custom runtime logic.

- [ ] **Step 4: Run the test to verify it passes**

Run:

```bash
cargo test -p memd-client harness_registry_exposes_shared_preset_ids_and_defaults -- --exact
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/memd-client/src/harness/preset.rs crates/memd-client/src/harness/mod.rs crates/memd-client/src/harness/index.rs crates/memd-client/src/main.rs
git commit -m "feat: add shared harness preset registry"
```

### Task 2: Render pack docs and metadata from the preset schema

**Files:**
- Modify: `crates/memd-client/src/render.rs`
- Modify: `crates/memd-client/src/main.rs`
- Modify: `integrations/codex/README.md`
- Modify: `integrations/openclaw/README.md`
- Modify: `integrations/hermes/README.md`
- Modify: `integrations/opencode/README.md`
- Modify: `integrations/agent-zero/README.md`
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write the failing test**

Add a test that renders each pack README from the registry and checks that each one uses its own pack defaults while sharing the same core loop language:

```rust
assert!(codex.contains("Codex"));
assert!(codex.contains("wake / resume / checkpoint"));
assert!(openclaw.contains("compact context / spill"));
assert!(hermes.contains("adoption-focused"));
assert!(opencode.contains("resume / remember / handoff"));
assert!(agent_zero.contains("zero-friction"));
assert!(codex.contains("shared memory core"));
assert!(hermes.contains("shared memory core"));
```

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
cargo test -p memd-client pack_readmes_render_from_shared_preset_registry -- --exact
```

Expected: fail because the docs still have hand-maintained pack copy.

- [ ] **Step 3: Write the minimal implementation**

Add a single render path that takes a `HarnessPreset` and produces the README/help text for that pack. Use the schema fields for:

- pack title
- entrypoint
- default verbs
- surface set
- pack-specific tone

Keep the pack-specific differences limited to wording and defaults; do not duplicate memory semantics in each README.

- [ ] **Step 4: Run the test to verify it passes**

Run:

```bash
cargo test -p memd-client pack_readmes_render_from_shared_preset_registry -- --exact
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/memd-client/src/render.rs crates/memd-client/src/main.rs integrations/codex/README.md integrations/openclaw/README.md integrations/hermes/README.md integrations/opencode/README.md integrations/agent-zero/README.md
git commit -m "feat: render pack docs from preset schema"
```

### Task 3: Generate command/help surfaces and pack index from the same schema

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Modify: `crates/memd-client/src/harness/index.rs`
- Modify: `docs/setup.md`
- Modify: `docs/api.md`
- Modify: `docs/oss-positioning.md`
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write the failing test**

Add a test that asserts the pack index and command-help surfaces stay in sync with the schema:

```rust
assert!(index.summary().contains("Codex"));
assert!(index.summary().contains("OpenClaw"));
assert!(index.summary().contains("Hermes"));
assert!(index.summary().contains("OpenCode"));
assert!(index.summary().contains("Agent Zero"));
assert!(help_text.contains("memd wake"));
assert!(help_text.contains("memd resume"));
assert!(help_text.contains("memd handoff"));
```

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
cargo test -p memd-client pack_index_and_help_surface_track_the_schema -- --exact
```

Expected: fail because index/help text are still partly hand-maintained.

- [ ] **Step 3: Write the minimal implementation**

Route pack index summaries and command/help text through the preset registry so the same schema powers:

- pack listing output
- startup copy
- docs sections
- pack-specific command wording

This step should remove the last major source of drift between pack metadata and runtime behavior.

- [ ] **Step 4: Run the test to verify it passes**

Run:

```bash
cargo test -p memd-client pack_index_and_help_surface_track_the_schema -- --exact
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/memd-client/src/main.rs crates/memd-client/src/harness/index.rs docs/setup.md docs/api.md docs/oss-positioning.md
git commit -m "feat: route pack index and help through preset schema"
```

### Task 4: Verify end-to-end pack parity and drift elimination

**Files:**
- Modify: none unless a test exposes a small bug
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Run the pack parity checks**

Run:

```bash
cargo test -p memd-client --quiet
cargo test --workspace --quiet
```

- [ ] **Step 2: Smoke test the generated surfaces**

Check the updated pack surfaces:

```bash
cargo run -p memd-client --bin memd -- packs --summary
```

Verify the summary shows the five pack presets with their intended defaults.

- [ ] **Step 3: Confirm drift is gone**

Confirm the generated pack README text, docs sections, and index output all agree on:

- pack names
- pack IDs
- entrypoints
- default verbs
- pack tone

- [ ] **Step 4: Commit any final fix**

If the parity checks expose a mismatch, fix only the mismatch and commit it separately. Do not add unrelated cleanup.

## Coverage Check

This plan covers the spec requirements as follows:

- shared preset contract: Task 1
- pack docs and runtime metadata from same source: Task 2
- pack index and help surfaces from schema: Task 3
- drift elimination verification: Task 4

No spec requirement is left without a task.
