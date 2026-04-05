# Promotion Policy

## Principle

Not every memory deserves to become long-term knowledge.

`memd` should treat dream output, session summaries, and agent writebacks as candidates, not truth.

## Promote to Synced Short-Term

Good candidates:

- active project objective
- current blocker
- current machine state
- workstream handoff

Bad candidates:

- old finished work
- broad architectural prose
- facts that already exist in long-term memory

## Promote to Project Long-Term

Good candidates:

- stable architecture decisions
- recurring project rules
- durable runbooks
- important debugging discoveries that are likely to remain true

Bad candidates:

- transient hypotheses
- temporary incidents
- unresolved conflicts

## Promote to Global Long-Term

Good candidates:

- cross-project preferences
- reusable operating rules
- repeatable infra patterns
- standards that matter across tools and codebases

Bad candidates:

- project-specific quirks
- one-off fixes
- unverified habits

## Gate Conditions

Before promotion, check:

- duplicate or near-duplicate existence
- contradiction with newer memories
- source quality
- freshness
- scope correctness
- confidence threshold
- redundancy key collision

If the item is still useful but not yet canonical, it can stay visible in inbox or working memory with explicit policy reasons instead of being silently dropped.

The same rule applies to retrieval policy: expose the visible hooks first, then learn from them later.

See [Source Policy](./source-policy.md) for what counts as acceptable input in the first place.

## Dream Policy

- project dream can propose project and synced memories
- cross-project dream can propose global memories
- neither should write canonical long-term memory directly
