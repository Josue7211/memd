# Inspiration Backlog

This is the work that falls out of the repo extraction.

## Implementation Order

1. Add inspiration search.
2. Save durable repo records.
3. Keep a what-we-borrowed summary in project docs.

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

## High Value

4. Turn `caveman` into a memd prompt/output policy.
   - Short status summaries.
   - Explicit output limits.
   - Commanded compression modes.

5. Turn `MinerU` into the ingest spine.
   - Normalize inputs before semantic classification.
   - Keep raw source, parsed text, and metadata separate.

6. Turn `LightRAG` into a backend adapter boundary.
   - Per-machine backend URL.
   - API root only.
   - Workspace support.

7. Turn `claude-peers-mcp` into a memd peer model.
   - Registration.
   - Heartbeat.
   - Summary.
   - Push message flow.

8. Turn `awesome-design-md` into an inspiration MD workflow.
   - One design doc per style.
   - Preview page for verification.
   - Copyable project-specific design contracts.

9. Turn `mempalace` into a raw-evidence storage rule.
   - Preserve source text.
   - Attach structure without discarding truth.
   - Search with wing/room-like filters.

10. Turn the Karpathy KB pattern into a research workspace loop.
   - Raw/ ingest directory.
   - Compiled markdown wiki.
   - Obsidian frontend.
   - Self-evolving lint / repair passes.

11. Turn `wiki-gen-skill` into the wiki compilation contract.
   - Absorb every entry somewhere.
   - Synthesize by concept, not just by file.
   - Keep backlinks and index files current.

12. Turn `agent-zero` into the prompt/plugin policy model.
   - Editable system prompt.
   - Dynamic tool/plugin surface.
   - Persistent memory plus subagents.

13. Turn `Hermes` into the onboarding/adoption model.
    - Cloud-first option.
    - Self-host later.
    - Obsidian and multi-platform gateways.

14. Turn `supermemory` into the harness-plugin packaging model.
    - One API for memory, RAG, profiles, connectors, and file processing.
    - Separate plugins per harness instead of one generic client wrapper.
    - Auto-recall before turns and auto-capture after turns.
    - Add a turn cache so repeated tool loops do not re-fetch the same memory payload.
    - Container tags / project routing for work, personal, and repo scopes.
    - Multi-modal extractors as first-class ingestion surfaces.
    - Keep the graph/browse UI separate from the core memory API.

## Lower Priority

15. Build a repo diff watcher for the inspiration lane.
    - Flag when an upstream repo changes significantly.
    - Re-review extraction notes when it does.

16. Add benchmark notes for each borrowed pattern.
    - Token savings.
    - Recall impact.
    - Setup friction.

17. Add “avoid” notes everywhere.
    - Most inspiration docs only say what looks good.
    - memd should also record what not to copy.

## Supermemory Gap Closure

18. Ship harness-specific plugin packs for memd.
    - One plugin per harness instead of one adapter blob.
    - Keep the shared memory core small and reusable.
    - Recommended shape: full harness packs first, then a generator once the first packs are stable.
    - Build order: OpenAI/Codex first, OpenClaw second, Claude third, Hermes fourth.

19. Push turn-scoped recall caching through every turn-based memory path.
    - Search, profile, and event refresh should reuse the same turn payload.
    - The same turn should never pay the same retrieval cost twice.

20. Keep the graph/browser surface separate from the core memory API.
    - Search should resolve to a visible object/page.
    - The browse layer should stay inspectable even if the backend changes.

## Acceptance Rule

An inspiration item is not done until it has:

- a source link
- one-line role
- one-line borrow note
- one-line avoid note
- one follow-up implementation question
