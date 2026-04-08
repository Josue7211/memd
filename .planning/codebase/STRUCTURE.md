# Structure

## Repository Shape

Top-level layout:

- `crates/`
  - Rust workspace crates for client, server, schema, RAG, multimodal, worker
- `docs/`
  - product and setup docs
- `integrations/`
  - harness-specific integration docs and assets
- `.planning/`
  - roadmap, state, phase artifacts, codebase maps
- `.memd/`
  - generated local bundle/memory artifacts for this repo
- `deploy/`
  - deployment config
- `scripts/`
  - support scripts

## Key Runtime Locations

### Client-Orchestrated Memory Paths

- `crates/memd-client/src/main.rs`
  - CLI command enum and dispatch
  - bundle runtime/default resolution
  - `remember`, `checkpoint`, `resume`, `refresh`, `handoff`
  - repo-change live-truth sync
  - bundle artifact writing
- `crates/memd-client/src/render.rs`
  - compact resume/handoff prompt output
- `crates/memd-client/src/obsidian.rs`
  - markdown/vault output helpers

### Server Retrieval and Persistence Paths

- `crates/memd-server/src/main.rs`
  - Axum app, HTTP routes, context assembly
- `crates/memd-server/src/working.rs`
  - working-memory selection and budgeting
- `crates/memd-server/src/store.rs`
  - SQLite schema and persistence operations
- `crates/memd-server/src/inspection.rs`
  - explain/inspection views
- `crates/memd-server/src/repair.rs`
  - repair operations
- `crates/memd-server/src/keys.rs`
  - key generation helpers

### Shared Model Paths

- `crates/memd-schema/src/lib.rs`
  - canonical item/request/response types
- `crates/memd-core/src/lib.rs`
  - shared lower-level support

## Bundle / Generated Memory Structure

Project-local generated bundle:

- `.memd/`
  - `MEMD_MEMORY.md`
  - `agents/`
  - `hooks/`
  - `state/`

Important generated files mentioned in code:

- `.memd/MEMD_MEMORY.md`
  - main generated memory surface for harness consumption
- `.memd/state/last-resume.json`
  - serialized `BundleResumeState`
- `.memd/state/heartbeat.json`
  - bundle heartbeat
- source/capability registries and migration manifests

The bundle is an integration surface, not only storage. A lot of product
behavior depends on these generated files being re-read by external harnesses.

## Crate-Level Boundaries

### `memd-client`

Role:

- user-facing CLI
- bundle manager
- prompt/render generator
- orchestration layer over server APIs

Notable concern:

- `src/main.rs` is the dominant coordination file and contains both runtime code
  and extensive tests

### `memd-server`

Role:

- durable memory store
- search/context/working-memory APIs
- coordination and inspection APIs

Notable concern:

- server routing and context-building logic are concentrated in `src/main.rs`

### `memd-schema`

Role:

- contract crate shared across binaries

Why it matters:

- architectural behavior is often implied by schema richness, but runtime recall
  still depends on how client and server actually use those types

### `memd-rag` / `memd-multimodal` / `memd-sidecar`

Role:

- optional or extended retrieval/ingest paths

These appear adjacent to the core memory system, not the guaranteed hot path.

## Naming and Organizational Patterns

- crate names follow `memd-*`
- most memory behavior terms are explicit in type and function names:
  - `working_memory`
  - `context_compact`
  - `remember_with_bundle_defaults`
  - `sync_recent_repo_live_truth`
  - `write_bundle_memory_files`
- generated project bundle artifacts live under `.memd/`
- planning artifacts live under `.planning/`

## Hot Paths To Read First For Memory Bugs

If debugging “memories are not working,” start here in this order:

1. `crates/memd-client/src/main.rs`
   - `Commands::Remember`
   - `remember_with_bundle_defaults`
   - `read_bundle_resume`
   - `sync_recent_repo_live_truth`
   - `write_bundle_memory_files`
2. `crates/memd-server/src/main.rs`
   - `search_memory`
   - `get_context`
   - `get_compact_context`
   - `build_context`
3. `crates/memd-server/src/working.rs`
   - `working_memory`
4. `crates/memd-client/src/render.rs`
   - `render_resume_prompt`

That sequence matches the practical memory loop most closely.

## Structure-Level Reasons The System Can Feel Broken

### 1. Storage and consumption live in different places

- server persists memory
- client regenerates bundle files
- external harnesses must then consume those files

That multi-hop structure makes failures easy to hide.

### 2. Too much behavior is centralized in giant files

Especially:

- `crates/memd-client/src/main.rs`
- `crates/memd-server/src/main.rs`
- `crates/memd-server/src/store.rs`

This increases the chance that product-critical flows are partially implemented
without one obvious ownership boundary.

### 3. Planning tree is rich, runtime truth is harder to inspect

The repo has a very deep `.planning/phases/` history and regenerated maps under
`.planning/codebase/`, but the actual hot runtime loop is still buried in a few
large Rust files.

## Current Structural Diagnosis

The repository structure is optimized for:

- breadth of capabilities
- planning discipline
- generated artifact surfaces

It is less optimized for:

- proving that a stored memory is later recalled and changes behavior

That mismatch is likely a major reason the product can look sophisticated while
still failing the core “remember what matters” test in real use.
