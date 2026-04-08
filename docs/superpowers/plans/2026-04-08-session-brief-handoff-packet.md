# Session Brief and Handoff Packet Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make new sessions useful in one read by rendering a compact, identity-aware session brief for `status`, `resume`, `handoff`, and new-session bootstrap.

**Architecture:** Build one shared session-brief packet from existing bundle/runtime state, then render that same packet through all high-signal surfaces. Keep compiled memory pages as the long-form truth and make the brief a fast front door with header/body/proof sections.

**Tech Stack:** Rust, existing `memd-client` render pipeline, current bundle/state JSON, existing docs and integration markdown.

---

### Task 1: Define the session brief data shape and synthesis path

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Modify: `crates/memd-client/src/render.rs`
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write the failing test**

Add a unit test that constructs a minimal resume/handoff state and asserts the synthesized brief contains:

```rust
assert!(brief.contains("project=demo"));
assert!(brief.contains("session=session-alpha"));
assert!(brief.contains("tab=tab-bravo"));
assert!(brief.contains("what we are doing now"));
assert!(brief.contains("last handoff"));
assert!(brief.contains("next 3 actions"));
assert!(brief.contains("proof"));
```

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
cargo test -p memd-client session_brief_contains_identity_and_ranked_sections -- --exact
```

Expected: fail because the session brief type and renderer do not exist yet.

- [ ] **Step 3: Write the minimal implementation**

Add a small shared brief structure in `main.rs` or `render.rs` that gathers:

- `project`
- `namespace`
- `session`
- `tab`
- `goal`
- `freshness`
- `blocker`
- `now`
- `last_handoff`
- `next_actions`
- `open_loops`
- `contradictions`
- `proof_links`

Shape the renderer so it emits the ranked order from the spec:

```rust
pub(crate) fn render_session_brief(brief: &SessionBrief) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "project={} namespace={} session={} tab={} goal={} freshness={} blocker={}\n",
        brief.project, brief.namespace, brief.session, brief.tab, brief.goal, brief.freshness, brief.blocker
    ));
    out.push_str(&format!("now: {}\n", brief.now));
    out.push_str(&format!("last handoff: {}\n", brief.last_handoff));
    out.push_str(&format!("next 3 actions: {}\n", brief.next_actions.join(" | ")));
    out.push_str(&format!("open loops: {}\n", brief.open_loops.join(" | ")));
    out.push_str(&format!("contradictions: {}\n", brief.contradictions.join(" | ")));
    out.push_str(&format!("proof: {}\n", brief.proof_links.join(" | ")));
    out
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run:

```bash
cargo test -p memd-client session_brief_contains_identity_and_ranked_sections -- --exact
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/memd-client/src/main.rs crates/memd-client/src/render.rs
git commit -m "feat: add session brief synthesis"
```

### Task 2: Render the brief from status, resume, and handoff

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Modify: `crates/memd-client/src/render.rs`
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write the failing test**

Add tests for each surface that assert the same brief string appears in:

- `status --summary`
- `resume --summary`
- `handoff --summary`

Example assertion:

```rust
assert!(output.contains("what we are doing now"));
assert!(output.contains("next 3 actions"));
assert!(output.contains("proof"));
```

- [ ] **Step 2: Run the tests to verify they fail**

Run:

```bash
cargo test -p memd-client status_summary_renders_session_brief -- --exact
cargo test -p memd-client resume_summary_renders_session_brief -- --exact
cargo test -p memd-client handoff_summary_renders_session_brief -- --exact
```

Expected: fail because the commands still render their older surface-specific summaries.

- [ ] **Step 3: Write the minimal implementation**

Update command routing so the shared brief renderer is called from the status/resume/handoff summary paths. Keep the surface-specific details only as trailing metadata.

Minimal pattern:

```rust
let brief = build_session_brief(&runtime, &snapshot, handoff.as_ref());
println!("{}", render_session_brief(&brief));
```

Do not remove existing fields yet if they are still needed elsewhere. The plan is to unify the high-signal top section first, not to rewrite the whole command output.

- [ ] **Step 4: Run the tests to verify they pass**

Run:

```bash
cargo test -p memd-client status_summary_renders_session_brief -- --exact
cargo test -p memd-client resume_summary_renders_session_brief -- --exact
cargo test -p memd-client handoff_summary_renders_session_brief -- --exact
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/memd-client/src/main.rs crates/memd-client/src/render.rs
git commit -m "feat: render session brief across hot paths"
```

### Task 3: Add the durable session brief artifact and bootstrap surface

**Files:**
- Modify: `crates/memd-client/src/main.rs`
- Modify: `crates/memd-client/src/render.rs`
- Modify: `integrations/hooks/README.md`
- Modify: `docs/setup.md`
- Modify: `docs/api.md`
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Write the failing test**

Add a test that verifies the session brief artifact is written to disk and contains the ranked sections:

```rust
assert!(brief_path.exists());
let brief = std::fs::read_to_string(&brief_path).unwrap();
assert!(brief.contains("what we are doing now"));
assert!(brief.contains("last handoff"));
assert!(brief.contains("next 3 actions"));
```

Also add a bootstrap test that asserts the first-session surface includes the session brief before drilldown links.

- [ ] **Step 2: Run the tests to verify they fail**

Run:

```bash
cargo test -p memd-client session_brief_artifact_is_written -- --exact
cargo test -p memd-client new_session_bootstrap_prints_session_brief_first -- --exact
```

Expected: fail because the artifact and bootstrap hook are not yet wired.

- [ ] **Step 3: Write the minimal implementation**

Write `MEMD_SESSION_BRIEF.md` from the same synthesized brief used by the hot-path renderers. Then update bootstrap wiring so a fresh session reads that artifact first, after project/namespace/session/tab are known.

If a bootstrap path already writes wakeup memory, have it link to the session brief rather than duplicating all of it.

- [ ] **Step 4: Update docs**

Document the new role of the brief in the existing docs:

- `docs/setup.md` for the user-facing workflow
- `docs/api.md` for the command contract
- `integrations/hooks/README.md` for hook output expectations

Add a short note that:

- the session brief is the fast front door
- compiled memory pages remain the durable truth
- proof links are how the user drills down

- [ ] **Step 5: Run the tests to verify they pass**

Run:

```bash
cargo test -p memd-client session_brief_artifact_is_written -- --exact
cargo test -p memd-client new_session_bootstrap_prints_session_brief_first -- --exact
cargo test -p memd-client --quiet
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/memd-client/src/main.rs crates/memd-client/src/render.rs integrations/hooks/README.md docs/setup.md docs/api.md
git commit -m "feat: add durable session brief artifact"
```

### Task 4: Verify end-to-end session quality on a real bundle

**Files:**
- Modify: none unless a test exposes a small bug
- Test: `crates/memd-client/src/main.rs`

- [ ] **Step 1: Run the hot-path commands**

Use the current bundle and a real tab ID:

```bash
MEMD_TAB_ID=tab-alpha cargo run -p memd-client --bin memd -- status --summary --output .memd
MEMD_TAB_ID=tab-alpha cargo run -p memd-client --bin memd -- resume --output .memd --intent current_task --summary
MEMD_TAB_ID=tab-alpha cargo run -p memd-client --bin memd -- handoff --output .memd --summary
```

- [ ] **Step 2: Verify the outputs**

Check that each output includes:

- project / namespace / session / tab
- current task
- last handoff
- next actions
- proof links
- freshness or blocker if present

- [ ] **Step 3: Run the workspace tests**

Run:

```bash
cargo test -p memd-client --quiet
cargo test --workspace --quiet
```

Expected: green.

- [ ] **Step 4: Commit any final fix**

If the smoke run exposes a specific bug, fix only that bug and commit it separately. Do not bundle unrelated cleanup.

## Coverage Check

This plan covers the spec requirements as follows:

- shared session brief packet: Task 1
- same packet across status/resume/handoff: Task 2
- durable artifact and bootstrap surface: Task 3
- freshness, blockers, proof links: Tasks 1-3
- real-world verification: Task 4

No spec requirement is left without a task.
