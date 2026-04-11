# Visible Memories UI Layer Design

Date: 2026-04-09
Topic: visible memories UI layer
Status: proposed

## Summary

`memd` should ship a native UI layer that acts as the canonical workspace for the
whole memory control plane.

This UI layer is not a replacement for Obsidian and not a generic note app.
It is the place where notes, compiled knowledge, runtime memory, provenance,
truth status, repair state, workspace awareness, handoff, routing, and source
drilldown all become visible and operable through one artifact model.

The product thesis is:

- `memd` replaces the mess between notes, docs, scratchpads, handoff files,
  hidden agent memory blobs, and half-trusted summaries
- `memd` turns that sprawl into living memory artifacts
- a living memory artifact stays readable like a note, linked like knowledge,
  and inspectable and repairable like runtime state
- Obsidian remains a first-class integration surface, not something `memd`
  rebuilds from scratch

The 10-star framing is:

`memd` is where notes become living memory.

## Product Goal

Build a universal OSS note, knowledge, and memory workspace that can answer:

- what do we know
- why do we believe it
- what changed
- what is stale or contested
- what is currently active in the workspace
- how do we repair the wrong thing without leaving the page

The visible memories UI layer must cover the full product surface, not only a
memory browser. It is the canonical shell for the whole system.

## Product Position

### What `memd` is

- a memory control plane
- a truth-first workspace
- a universal OSS memory operating surface
- a shared artifact model across web, CLI/TUI, and integrations

### What `memd` is not

- not only a storage layer
- not only a semantic retrieval layer
- not only a prettier Obsidian clone
- not only a dashboard over backend APIs

### Obsidian role

Obsidian is a first-class integration and human workspace, not the core product
model.

That means:

- users can keep using their actual Obsidian vault
- `memd` can import, sync, watch, write back, compile into, and open vault
  artifacts
- the native web UI and CLI/TUI remain the canonical `memd` surfaces
- the same artifact should stay coherent across the native UI, CLI/TUI, and
  Obsidian-linked representations

This matches the repo direction:

- `docs/core/architecture.md` places Obsidian in the visible memory story
- `docs/core/obsidian.md` defines the vault bridge as a real deployment tier
- `crates/memd-client/src/obsidian.rs` already owns scan, import, sync, watch,
  writeback, open, compile, and handoff behavior

## Core Object: Memory Artifact

Everything visible in `memd` should render as a first-class memory artifact.

This is the unavoidable core product object.

It is not a plain note, not a vector hit, and not a backend record dump.

It must be:

- readable like a note
- traversable like knowledge
- stateful like memory
- inspectable like evidence
- repairable like truth

### Artifact kinds

The UI must support at least these artifact kinds:

- note
- compiled knowledge page
- working-memory item
- inbox candidate
- handoff packet
- awareness object
- source bundle
- repair target
- semantic recall result
- lane page
- event page
- pack page
- skill page

### Required fields

Every artifact must expose:

- `id`
- `title`
- `body`
- `artifact_kind`
- `memory_kind`
- `scope`
- `visibility`
- `workspace`
- `status`
- `freshness`
- `confidence`
- `provenance`
- `sources`
- `linked_artifacts`
- `linked_sessions`
- `linked_agents`
- `repair_state`
- `history`
- `actions`

### Required statuses

- `current`
- `candidate`
- `stale`
- `superseded`
- `conflicted`
- `archived`

### Required provenance fields

- source path or origin
- source system
- compile, import, or writeback path
- who or what produced it
- last verification point
- current trust reason

### Required actions

- inspect
- explain
- open source
- open linked artifacts
- verify current
- mark stale
- supersede
- promote
- demote to candidate
- handoff
- open in Obsidian when applicable

## Native UI Strategy

The native UI should feel like a serious knowledge workspace first, with live
control-plane state always visible.

It should not feel like:

- a pure note editor
- a pure graph toy
- a pure admin console

### Recommended structure

Use a hybrid knowledge-workbench shell.

#### Left rail

Persistent navigation for:

- workspaces
- lanes
- saved views
- artifact collections
- vault-linked sources
- packs and integrations

#### Center workspace

Default mode is `Memory Home`.

Adjacent first-class mode is `Knowledge Map`.

Other center modes should include:

- artifact document view
- working-memory view
- source view
- recall neighborhood view

The center always anchors on the currently selected artifact or current working
set.

#### Right rail

Persistent side context for:

- truth state
- provenance
- repair actions
- workspace awareness
- routing context
- source drilldown
- contradictions and freshness pressure

#### Global command layer

Fast universal actions:

- search
- explain
- handoff
- resume
- source open
- repair actions
- open in Obsidian when applicable

## Home View

### Default landing view: Memory Home

`Memory Home` is the default landing surface because it answers the immediate
question:

- what matters right now

It should prioritize:

1. selected artifact document
2. active truth and repair pressure
3. workspace activity and cowork awareness

The first screen should make it obvious:

- what `memd` currently believes
- why it believes it
- whether that belief is safe
- which sessions and agents depend on it

### First-class adjacent view: Knowledge Map

`Knowledge Map` is the first-class alternate mode because it answers:

- how does this artifact connect to the rest of the system

It is not a vanity graph.

It must show useful structure such as:

- backlinks
- entity and wiki-link relationships
- lane and workspace neighborhoods
- source adjacency
- contradictions and drift edges
- semantic neighbors when relevant

Selecting in the graph should update the main artifact context instead of
throwing the user into a separate product.

## Full Feature Coverage

The visible memories UI layer must explicitly cover all major `memd` features.

### Notes, knowledge, and compiled memory

The UI must show:

- raw imported notes
- compiled pages
- lane pages
- evidence pages
- writeback and handoff artifacts

### Working memory, inbox, explain, and search

The UI must expose:

- active working memory
- inbox pressure and candidate promotion
- explain views that justify retrieval and trust
- search with bounded, inspectable results

### Truth, freshness, contradiction, and repair

The UI must make visible:

- freshness
- verification state
- contradiction state
- promotion history
- supersession
- repair queues and actions

This is core product value, not admin metadata.

### Routing and source drilldown

The UI must show:

- route intent and selected retrieval lane
- why a memory was surfaced
- source anchors
- linked raw evidence
- compact-to-source drilldown

### Workspace lanes, visibility, awareness, and shared state

The UI must support:

- private, workspace, and public visibility lanes
- shared workspace browsing
- cowork and session awareness
- session identity and tab identity where relevant
- collision and stale-state signals

### Resume and handoff

The UI must surface:

- active resume frame
- handoff packet artifacts
- rehydration queue
- active source lanes
- compact shared context

### Obsidian integration

The UI must treat Obsidian as:

- a source lane
- a destination for compiled and writeback artifacts
- a real workspace bridge
- a direct open target

It should never imply that users must abandon their real vault.

### Semantic recall

Semantic retrieval should appear as an attached lane, not the product center.

The UI must preserve the repo doctrine:

- semantic retrieval is fallback and neighborhood expansion
- semantic hits must point back to compiled or source-linked artifacts
- semantic recall must not become the only visible representation of memory

## Unforgettable Product Loop

The first unforgettable loop should be:

1. user opens `Memory Home`
2. user sees the currently important artifact
3. user immediately sees:
   - what `memd` believes
   - why
   - where it came from
   - who or what is using it
   - whether it is stale or conflicted
4. user repairs, promotes, or verifies it in place
5. user sees that change reflected in working memory, handoff, and linked
   surfaces
6. user optionally opens the corresponding vault artifact in Obsidian

That is the first magic moment.

Not “I stored a memory.”
The magic moment is:

“I can control belief drift in my knowledge system.”

## UX Principles

### Principle 1: readable first

Every important artifact must be readable before it is inspectable.

### Principle 2: truth always visible

Truth state, freshness, and conflict pressure should never be hidden behind a
separate admin mode.

### Principle 3: source-linked by default

Every important claim should have a visible path back to source.

### Principle 4: repair in place

Users should repair the selected artifact without navigating to a separate
operations product.

### Principle 5: one artifact model, many lenses

Web, CLI/TUI, and Obsidian-linked representations should differ in density, not
in product truth.

### Principle 6: workspaces stay visible

Session, tab, workspace, and lane identity should remain visible on artifact
surfaces when relevant.

### Principle 7: semantic is support, not center

Semantic recall helps find things but should not replace the visible memory
surface.

## Surface Definitions

### Web app

The richest canonical surface.

Responsibilities:

- artifact reading
- knowledge navigation
- truth and provenance inspection
- repair workflows
- workspace awareness
- routing and source drilldown
- handoff and resume inspection

### CLI/TUI

The compact operator surface.

Responsibilities:

- fast inspect
- fast explain
- search and route inspection
- repair actions
- awareness and queue summaries
- handoff and resume review

### Obsidian

The markdown-native human workspace integration.

Responsibilities:

- browse and author vault content
- read compiled pages and handoff artifacts
- leverage backlinks and graph browsing
- receive writebacks and compiled artifacts
- serve as a bridge to human workflows without replacing `memd`’s artifact model

## Non-Goals

This design does not propose:

- rebuilding Obsidian inside `memd`
- replacing the existing CLI with a web-only product
- making semantic recall the primary memory representation
- collapsing all artifacts into plain markdown notes
- hiding repair and trust semantics behind backend-only endpoints

## Success Criteria

The visible memories UI layer succeeds when:

- users can understand the current memory state without reading raw backend data
- every major feature has a visible home in the UI shell
- truth, provenance, freshness, and repair state are first-class
- the same artifact stays coherent across web, CLI/TUI, and Obsidian-linked
  views
- users can keep using their real Obsidian vault while adopting `memd`
- the first user impression is clearly bigger than “notes app” and clearly more
  usable than “memory black box”

## Open Implementation Direction

The next planning step should define:

- the concrete artifact schema in code
- the canonical view models for web and CLI/TUI
- the first thin vertical slice of `Memory Home`
- the first thin vertical slice of `Knowledge Map`
- feature-by-feature rollout order without breaking existing Obsidian and CLI
  flows
