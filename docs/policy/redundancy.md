> Secondary/reference doc. For current project truth start with [[ROADMAP]] and [[docs/WHERE-AM-I.md|WHERE-AM-I]].

# Redundancy Policy

`memd` should prevent duplicate memory at the source, not after the fact.

## Goal

Avoid storing the same fact in multiple forms across:

- session memory
- auto-dream output
- project long-term memory
- global long-term memory
- downstream RAG backends

## Collapse Rules

- exact duplicates collapse immediately
- near-duplicates collapse under a redundancy key
- paraphrases that do not add new meaning should not become new long-term memories
- derived writeback should carry `source_quality=derived`

## Auto-Dream Rule

Dream output is not canonical memory.

It should:

- compress repeated content
- propose candidate memories
- inherit or infer a redundancy key
- avoid re-emitting stable facts unless they changed

## RAG Rule

When `memd` writes into LightRAG or another long-term backend:

- send only durable records
- preserve the redundancy key
- avoid writing candidate spam
- prefer canonical summaries over raw transcript chunks

## Source Rule

- canonical source material can be ingested
- derived spill/writeback can be stored
- synthetic source material must be rejected

## Outcome

The platform should behave like a memory manager with compaction, not a transcript archive with extra steps.
