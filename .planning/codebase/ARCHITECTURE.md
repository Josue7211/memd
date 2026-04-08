# Architecture

## Overview

`memd` is a Rust workspace that splits the product into a CLI/client, an HTTP
server with SQLite persistence, shared schema types, and optional sidecars for
RAG and multimodal ingestion.

The practical memory loop is:

1. CLI command gathers runtime defaults and bundle metadata
2. client calls server APIs for memory storage or retrieval
3. server loads a snapshot from SQLite and ranks memory into context/working sets
4. client renders prompt and markdown bundle artifacts
5. external agent harnesses consume those generated files or prompts

The current architectural risk is that `memd` is strong at generating memory
artifacts, but weaker at guaranteeing that stored memories actually affect later
agent behavior.

## Main Components

### Shared Types

- `crates/memd-schema/src/lib.rs`
  - shared API request/response types
  - memory item model, scope/status/kind enums
  - working-memory and policy response types
  - retrieval request shapes used by client and server

### Client / CLI

- `crates/memd-client/src/main.rs`
  - CLI command parsing and most orchestration logic
  - bundle runtime/default resolution
  - `remember`, `checkpoint`, `resume`, `refresh`, `handoff`, eval, experiment
  - repo-change live-truth sync via `sync_recent_repo_live_truth`
  - bundle artifact generation via `write_bundle_memory_files`
- `crates/memd-client/src/render.rs`
  - compact prompt rendering for resume and handoff
- `crates/memd-client/src/obsidian.rs`
  - vault-oriented output and markdown fixture support

### Server / Persistence

- `crates/memd-server/src/main.rs`
  - Axum HTTP routes
  - retrieval endpoints: `/memory/context`, `/memory/context/compact`,
    `/memory/working`, `/memory/search`
  - `build_context` is the main context-assembly seam
- `crates/memd-server/src/working.rs`
  - working-memory budgeting, ranking, eviction, rehydration queue
- `crates/memd-server/src/store.rs`
  - SQLite schema, persistence, snapshots, trust/entity/coordination storage
- `crates/memd-server/src/inspection.rs`
  - explain/inspection-oriented views

### Optional Sidecars

- `crates/memd-rag`
  - semantic retrieval path used only when explicitly enabled
- `crates/memd-multimodal`
  - multimodal support
- `crates/memd-sidecar`
  - additional supporting services
- `crates/memd-worker`
  - background/worker binary

## Runtime Entry Points

### Storage Path

- `memd remember`
  - command handled in `crates/memd-client/src/main.rs`
  - calls `remember_with_bundle_defaults`
  - sends `StoreMemoryRequest` to server
  - then re-runs bundle resume and rewrites generated memory files

Relevant code:

- `crates/memd-client/src/main.rs` (`Commands::Remember`)
- `crates/memd-client/src/main.rs` (`remember_with_bundle_defaults`)
- `crates/memd-server/src/main.rs` (store route handling)
- `crates/memd-server/src/store.rs` (SQLite persistence)

### Resume / Recall Path

- `memd resume`
  - orchestrated by `read_bundle_resume`
  - first syncs repo-change live truth
  - then requests:
    - compact context
    - working memory
    - inbox
    - workspace memory
    - source memory
    - optional semantic retrieval
  - packages all of that into `ResumeSnapshot`
  - writes bundle markdown and state files

Relevant code:

- `crates/memd-client/src/main.rs` (`read_bundle_resume`)
- `crates/memd-server/src/main.rs` (`get_context`, `get_compact_context`)
- `crates/memd-server/src/working.rs` (`working_memory`)
- `crates/memd-client/src/render.rs` (`render_resume_prompt`)

## End-to-End Memory Loop

### Ingest

There are multiple ingest surfaces:

- explicit memory writes:
  - `remember`
  - `checkpoint`
  - raw `store`, `candidate`, `promote`, `repair`
- bootstrap/project import:
  - bundle generation and source registry refresh
- repo-change sync:
  - `sync_recent_repo_live_truth`

The strongest implemented automatic ingest path in current code is repo-change
live truth, not general conversational correction capture.

### Store

All durable item persistence is centralized through SQLite in
`crates/memd-server/src/store.rs`.

Key persisted classes:

- `memory_items`
- `memory_entities`
- `memory_events`
- coordination/session/task tables

This means the storage substrate is broad, but it also means many product claims
share one giant persistence surface with a lot of responsibilities.

### Retrieve

The main retrieval assembly path is `build_context` in
`crates/memd-server/src/main.rs`.

Observed behavior:

- loads a full snapshot
- enriches items with entity/trust metadata
- prepends active `LiveTruth` items
- then iterates retrieval scopes and ranks the remaining items

The working-memory path in `crates/memd-server/src/working.rs` then:

- takes the context output
- reranks for working-memory priority
- applies token/char budgets
- emits compact records and a rehydration queue

### Render

Prompt output is generated from `ResumeSnapshot` in
`crates/memd-client/src/render.rs`.

Important detail:

- rendered prompts are highly compressed
- they reflect the retrieved snapshot
- they do not themselves guarantee that a harness or model actually obeys the
  intended memory precedence

### Behavior Change

This is the weakest architectural seam.

`memd` can:

- store memories
- retrieve and compact them
- write markdown artifacts like `.memd/MEMD_MEMORY.md`
- render prompt text

But later agent behavior only changes if the active harness actually reloads and
uses that generated output. That dependency sits outside the core storage code.

## Architectural Seams Related To Recall Failure

### 1. Store and recall are decoupled by generated artifacts

`remember` writes memory, then regenerates bundle files. This is not the same as
proving the next answer consumed that memory.

Files:

- `crates/memd-client/src/main.rs`
- `.memd/MEMD_MEMORY.md`
- `.memd/state/last-resume.json`

### 2. Automatic live-truth ingest is narrow

The code clearly syncs recent repo changes into a live-truth record, but that
does not cover general “user corrected me” flows by default.

File:

- `crates/memd-client/src/main.rs` (`sync_recent_repo_live_truth`)

### 3. Retrieval is context assembly, not a proven recall contract

`build_context` and `working_memory` produce a ranked snapshot, but the system
still depends on downstream harnesses consuming it correctly.

Files:

- `crates/memd-server/src/main.rs`
- `crates/memd-server/src/working.rs`

### 4. Client orchestration is concentrated in one very large file

`crates/memd-client/src/main.rs` owns command parsing, bundle sync, resume
orchestration, rendering triggers, eval flows, experiment flows, bridge writing,
and tests. That makes root-cause analysis and behavioral guarantees harder.

### 5. Planning/status can drift from runtime truth

The repo has extensive planning and evaluation machinery in `.planning/`, but
those documents can say a phase is complete while the runtime memory contract is
still weak in real use.

## Likely Architecture-Level Explanation For Current Failure

The product behaves more like:

- a memory database
- a prompt/bundle generator
- a harness-bridge layer

than a fully closed-loop memory OS.

The missing guarantee is:

- when a memory is stored, the next relevant interaction must reliably retrieve
  and use it

Until that guarantee is enforced on the hot path, the architecture can look rich
while users still experience “no memory.”
