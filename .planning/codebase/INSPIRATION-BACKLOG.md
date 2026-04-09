# Inspiration Backlog

This is the work that falls out of the repo extraction.

## Implementation Order

1. Add inspiration search.
2. Save durable repo records.
3. Keep a what-we-borrowed summary in project docs.
4. Add ranked extraction notes for the Claude Code source build.

## Immediate

1. Add a dedicated inspiration search command.
   - Search the inspiration lane by repo, role, borrow pattern, or tag.
   - This is the fastest way to make the lane useful in real work.

2. Add repo-specific notes to memd memory.
   - One record per repo.
   - Fields: `role`, `borrow`, `avoid`, `fit_for_memd`, `follow_up`.

3. Add a “what we borrowed” section to project docs.
   - Keeps the extraction visible.
   - Reduces drift between inspiration and implementation.

4. Add Claude Code source extraction notes to the lane.
   - Capture the full runtime surface, not just agent teams.
   - Record what to borrow, what to avoid, and where memd fits.

## High Value

5. Turn `caveman` into a memd prompt/output policy.
   - Short status summaries.
   - Explicit output limits.
   - Commanded compression modes.

6. Turn `MinerU` into the ingest spine.
   - Normalize inputs before semantic classification.
   - Keep raw source, parsed text, and metadata separate.

7. Turn `LightRAG` into a backend adapter boundary.
   - Per-machine backend URL.
   - API root only.
   - Workspace support.

8. Turn the live coordination source into a live coordination model.
   - Registration.
   - Heartbeat.
   - Summary.
   - Push message flow.

9. Turn `awesome-design-md` into an inspiration MD workflow.
   - One design doc per style.
   - Preview page for verification.
   - Copyable project-specific design contracts.

10. Turn `mempalace` into a raw-evidence storage rule.
   - Preserve source text.
   - Attach structure without discarding truth.
   - Search with wing/room-like filters.

11. Turn the Karpathy KB pattern into a research workspace loop.
   - Raw/ ingest directory.
   - Compiled markdown wiki.
   - Obsidian frontend.
   - Self-evolving lint / repair passes.

12. Turn `wiki-gen-skill` into the wiki compilation contract.
   - Absorb every entry somewhere.
   - Synthesize by concept, not just by file.
   - Keep backlinks and index files current.

13. Turn `agent-zero` into the prompt/plugin policy model.
   - Editable system prompt.
   - Dynamic tool/plugin surface.
   - Persistent memory plus subagents.

14. Turn `Hermes` into the onboarding/adoption model.
    - Cloud-first option.
    - Self-host later.
    - Obsidian and multi-platform gateways.

15. Turn `supermemory` into the harness-plugin packaging model.
    - One API for memory, RAG, profiles, connectors, and file processing.
    - Separate plugins per harness instead of one generic client wrapper.
    - Auto-recall before turns and auto-capture after turns.
    - Add a turn cache so repeated tool loops do not re-fetch the same memory payload.
    - Container tags / project routing for work, personal, and repo scopes.
    - Multi-modal extractors as first-class ingestion surfaces.
    - Keep the graph/browse UI separate from the core memory API.

16. Turn the Claude Code source build into the runtime model extraction source.
    - Session continuity as a first-class runtime concern.
    - Managed memory compaction and background consolidation loops.
    - Worktree-aware isolation for parallel work.
    - IDE integration as part of the live context model.
    - Bridge / remote-control session lifecycle.
    - Explicit tool, command, skill, and capability catalogs.
    - Task taxonomy that distinguishes local, remote, teammate, workflow, and maintenance flows.
    - Native integrations, analytics, and feature flags as inspectable control planes.

## Lower Priority

17. Build a repo diff watcher for the inspiration lane.
    - Flag when an upstream repo changes significantly.
    - Re-review extraction notes when it does.

18. Add benchmark notes for each borrowed pattern.
    - Token savings.
    - Recall impact.
    - Setup friction.

19. Add “avoid” notes everywhere.
    - Most inspiration docs only say what looks good.
    - memd should also record what not to copy.

## Supermemory Gap Closure

20. Ship harness-specific plugin packs for memd.
    - One plugin per harness instead of one adapter blob.
    - Keep the shared memory core small and reusable.
    - Recommended shape: full harness packs first, then a generator once the first packs are stable.
    - Build order: OpenAI/Codex first, OpenClaw second, Claude third, Hermes fourth.

21. Push turn-scoped recall caching through every turn-based memory path.
    - Search, profile, and event refresh should reuse the same turn payload.
    - The same turn should never pay the same retrieval cost twice.

22. Keep the graph/browser surface separate from the core memory API.
    - Search should resolve to a visible object/page.
    - The browse layer should stay inspectable even if the backend changes.

## Acceptance Rule

An inspiration item is not done until it has:

- a source link
- one-line role
- one-line borrow note
- one-line avoid note
- one follow-up implementation question

## Ranked Memd Backlog From Claude Code

This is the leverage-ranked extraction from `/home/josue/Documents/projects/claude-code-source-build`.

### Near-Term

1. Session continuity overlay
   - Goal: make the current session survive restarts and rebind cleanly.
   - Source areas: session memory, bridge lifecycle, session runner.
   - Memd fit: strongest direct hit on the core promise.

2. Memory maintenance loop
   - Goal: compact, refresh, and clean up memory in the background.
   - Source areas: SessionMemory, autoDream, autoCompact.
   - Memd fit: turns memory into an active system, not a dump.

3. Truth-first memory model
   - Goal: make provenance, freshness, and contradiction state explicit.
   - Source areas: session memory, compaction, resume flows.
   - Memd fit: prevents durable confusion.

4. Live coordination view
   - Goal: show current, active, stale, dead, and shared state at a glance.
   - Source areas: bridge, task lifecycle, awareness surfaces.
   - Memd fit: makes multi-session work inspectable.

5. Worktree-aware isolation
   - Goal: make project/worktree boundaries real in runtime state.
   - Source areas: worktree utilities, session bootstrap.
   - Memd fit: enables safe parallel work without state bleed.

### Strategic Bets

6. IDE-aware context
   - Goal: pull editor state into live memory and feed it back cleanly.
   - Source areas: IDE integration hooks and selection state.
   - Memd fit: strongest practical continuity boost for daily use.

7. Capability catalog
   - Goal: expose tools, commands, skills, and plugins as a searchable catalog.
   - Source areas: tool registries, skill registries, plugin surfaces.
   - Memd fit: makes the runtime legible and extensible.

8. Session lifecycle controls
   - Goal: create, resume, rebind, retire, and reconcile sessions explicitly.
   - Source areas: bridge session creation and runtime session manager.
   - Memd fit: needed for reliable multi-session coordination.

9. Background task taxonomy
   - Goal: distinguish local, remote, teammate, workflow, and maintenance tasks.
   - Source areas: task definitions and orchestration plumbing.
   - Memd fit: makes autonomy inspectable instead of opaque.

10. Packaging and trust plane
   - Goal: keep plugin distribution, feature flags, analytics, and native integrations visible.
   - Source areas: package manifest, feature flags, analytics, native integrations.
   - Memd fit: keeps the product shippable and observable.
