# Code Conventions

## Overview

`memd` is a Rust workspace built around request/response DTOs in `memd-schema`, an HTTP server in `crates/memd-server`, and a large CLI/client in `crates/memd-client`.

The codebase follows consistent Rust patterns, but the main client and server entrypoints have grown very large, so local consistency is better than global modularity.

## Workspace Conventions

- Workspace members are declared centrally in `Cargo.toml`.
- Each major capability lives in a crate:
  - `crates/memd-schema` for shared types
  - `crates/memd-server` for persistence and retrieval routes
  - `crates/memd-client` for CLI, resume flows, bundle logic, and rendering
  - `crates/memd-core`, `crates/memd-rag`, `crates/memd-sidecar`, `crates/memd-worker`, `crates/memd-multimodal` for narrower supporting concerns

## Coding Patterns

### Typed request/response boundaries

- Shared request/response structs and enums are centralized in `crates/memd-schema/src/lib.rs`.
- Client calls generally mirror server routes 1:1 through methods on `MemdClient` in `crates/memd-client/src/lib.rs`.
- Server endpoints typically deserialize a schema request, call store/working helpers, then return a schema response.

This keeps transport shapes explicit, but it also means behavior can feel "implemented" once the DTO and route exist even if the product loop is still weak.

### Large entrypoint files

- `crates/memd-client/src/main.rs` is the dominant orchestration file.
- `crates/memd-server/src/main.rs` contains routing, retrieval scoring, and several tests.
- `crates/memd-client/src/render.rs` and `crates/memd-client/src/obsidian.rs` are also substantial.

The prevailing style is to add bounded helper functions near the command or flow instead of extracting a new module early.

### Error handling

- The repo consistently uses `anyhow::Result` and `Context` for filesystem/network errors.
- Axum handlers typically map failures through `internal_error` style adapters in `crates/memd-server/src/main.rs`.
- Errors are usually surfaced with file/path context rather than swallowed.

### Compact output style

- Prompt and summary rendering favors short compressed sections over prose.
- Example: `render_resume_prompt` in `crates/memd-client/src/render.rs` emits compact headings like `## T`, `## E+LT`, `## W`, `## RI`.

This is aligned with token-efficiency goals, but it makes behavior harder to inspect casually and easier to mistake for correctness if not verified with real flows.

## Naming Conventions

- Rust naming is conventional:
  - `CamelCase` for types
  - `snake_case` for functions/fields
  - enum variants serialized with `snake_case` where needed
- CLI parsing helpers are usually named `parse_*` and live in `crates/memd-client/src/commands.rs` or `crates/memd-client/src/main.rs`.
- Retrieval and rendering helpers are named by product surface:
  - `build_context`
  - `working_memory`
  - `render_resume_prompt`
  - `sync_recent_repo_live_truth`

## Architectural Conventions That Matter For Debugging

### Memory is represented as items plus policy metadata

- Memory items carry `kind`, `scope`, `status`, `source_quality`, `confidence`, `last_verified_at`, `supersedes`, and tags in `crates/memd-schema/src/lib.rs`.
- Ranking logic depends on these fields in `crates/memd-server/src/main.rs` and `crates/memd-server/src/working.rs`.

### Retrieval is assembled before working-memory ranking

- `build_context` in `crates/memd-server/src/main.rs` assembles candidate items.
- `working_memory` in `crates/memd-server/src/working.rs` ranks and compacts them into prompt-facing records.

This split is important: many memory bugs can come from either stage, even if tests cover only the ranking helper.

### Hot-path memory UX is client-rendered

- Resume/handoff behavior depends on how snapshots are rendered in `crates/memd-client/src/render.rs`.
- A memory can exist in storage and still be functionally useless if the resume/render path never surfaces it clearly enough for the agent loop to consume it.

## Observed Convention Gaps

- The repo often treats "route exists" or "record is stored" as a meaningful milestone, but the stronger product convention, "stored memory changes future behavior," is not consistently enforced in tests.
- There is explicit repo-change live-truth syncing in `crates/memd-client/src/main.rs`, but no equally obvious convention for automatic ingestion of user corrections or general conversational memories on the hot path.
- Planning docs use completion language aggressively relative to observed product behavior, so debugging should prefer code and repros over roadmap text.
