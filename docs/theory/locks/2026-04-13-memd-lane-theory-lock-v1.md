# memd Lane Theory Lock v1

## Canonical Definition

A lane is a domain-scoped memory context that activates on demand based on the current task.

Lanes are not storage. Lanes are not folders. Lanes are not memory kinds.

A lane is a **routing filter** over the memory atlas. When an agent works on frontend, the design lane activates. When an agent researches, the research lane activates. Multiple lanes can be active simultaneously.

## What a Lane Does

1. Activates based on current task signals (topic, scope, working context)
2. Surfaces lane-relevant memory items across all memory kinds
3. Accepts new memory items tagged with the lane
4. Deactivates when the task moves to a different domain

A lane cuts across memory kinds. The design lane holds episodic sessions, semantic rules, procedural workflows, candidate ideas, and canonical truths — all related to design.

## What a Lane Is Not

- Not a folder on disk
- Not a replacement for memory kinds
- Not a separate storage layer
- Not a static collection that must be manually loaded
- Not documentation

## Starter Lanes

| Lane | Activates When | Holds |
|---|---|---|
| Inspiration | studying external repos, papers, prior art that shaped the project | what to borrow, what to avoid, how it fits, research lineage |
| Design | frontend work, UI decisions, visual direction | design rules, component patterns, layout decisions |
| Architecture | system design, API shape, data flow | architecture decisions, module boundaries, integration patterns |
| Research | deep investigation, paper reading, experiments | findings, hypotheses, evidence, conclusions |
| Workflow | process work, CI/CD, deployment, maintenance | operating procedures, checklists, automation patterns |
| Preference | user corrections, style choices, tool preferences | user preferences, correction history, style rules |

Lanes can be extended. These six are the starter set.

## Lane Activation

Lane activation is automatic. The system detects which lanes are relevant from:

- Current working context content
- Topic claims from the hive session
- Scope claims
- Task metadata
- Explicit user lane switches

An agent doing frontend work does not need to say "activate design lane." The design lane activates because the working context signals frontend work.

Multiple lanes can be active at once. An agent researching a design pattern has both the research and design lanes active.

## Lane Storage

Lanes are **atlas-level tags on memory items**, not a separate storage layer.

- Every memory item can belong to zero or more lanes
- Atlas regions have a `lane` field (already in the schema)
- Lane membership is set at ingest time and can be updated by promotion or correction
- The lane tag participates in retrieval routing — active lanes boost their items in working memory ranking

## Lane Source Material

Some lanes have **source material** — external content that seeds the lane with knowledge.

Example: The inspiration lane has source files (research notes on external repos and papers). These files are **ingested** into the memory system as lane-tagged memory items. The files themselves are not the lane.

Source material lives in the `.memd/` bundle under `lanes/<lane-name>/`:

```
.memd/
  lanes/
    inspiration/
      INSPIRATION-LANE.md
      INSPIRATION-ARCHITECTURE.md
      ...
    design/
    architecture/
    research/
```

Source files are ingested on `memd wake` or `memd setup`. After ingestion, the lane query hits the server, not the files.

## Lane Retrieval

Lane retrieval is part of the working memory and atlas retrieval paths.

When lanes are active:

1. Working memory boosts items matching active lanes
2. Atlas explore/navigate respects lane filters
3. `memd inspiration --query "..."` queries the server for inspiration-lane items, not files
4. Wake packet includes lane-relevant procedural and canonical items

When no lanes are explicitly active, the system infers lanes from working context.

## Lane Lifecycle

1. **Seed** — source material is ingested during setup or wake
2. **Activate** — lane activates when task signals match
3. **Surface** — lane-relevant items appear in working memory and atlas
4. **Grow** — new memory items are tagged with active lanes during capture
5. **Consolidate** — dream/nightly passes can promote candidate lane items to canonical
6. **Deactivate** — lane goes quiet when task domain shifts

## Lane vs Memory Kind

Memory kinds (episodic, semantic, procedural, candidate, canonical) describe **what type** of memory something is.

Lanes describe **what domain** the memory belongs to.

These are orthogonal. A procedural memory about CSS grid layout belongs to the design lane. A canonical memory about the DroidSpeak paper belongs to the inspiration lane and the research lane.

## Lane vs Atlas Region

Atlas regions are navigable clusters of related memory items. A region can span multiple lanes or live entirely within one lane.

Lanes are broader than regions. The design lane contains many regions (typography, layout, color, components). A region is a zoom level within a lane.

## Relationship to Hive

In hive coordination, active lanes are part of the session heartbeat. Other sessions can see which lanes a peer has active, enabling:

- Lane-aware handoff (transfer design lane context to a design-focused agent)
- Lane collision detection (two agents with the same lane active on the same scope)
- Lane-scoped procedural sharing

## Locked Decisions

### D1. Lanes are atlas tags, not storage

Lane membership is a tag on memory items. There is no separate lane database or lane file system.

### D2. Lane activation is automatic

The system infers active lanes from task context. Explicit activation is available but not the default path.

### D3. Source material lives in the bundle

Lane source files (inspiration notes, design references) live under `.memd/lanes/<name>/` and are ingested into the server.

### D4. Lane queries hit the server

`memd inspiration` and future lane queries are server-side retrieval operations, not file scans. The file scan path is a bootstrap fallback only.

### D5. Lanes are project-scoped by default

Lanes belong to a project+namespace. Global lanes (user preferences across projects) are a future extension.
