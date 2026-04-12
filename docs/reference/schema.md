# Schema

## Memory Item

Every durable memory item should contain:

- `id`
- `content`
- `kind`
- `scope`
- `project`
- `namespace`
- `source_agent`
- `source_system`
- `source_path`
- `redundancy_key`
- `source_quality`
- `confidence`
- `ttl`
- `created_at`
- `updated_at`
- `last_verified_at`
- `supersedes`
- `tags`
- `status`

## Search Filters

`SearchMemoryRequest` supports filtering by:

- scope
- kind
- status
- stage
- project
- namespace
- source agent
- tags
- retrieval route
- retrieval intent

## Inbox / Explain

The control surface adds:

- `MemoryInboxRequest`
- `MemoryInboxResponse`
- `ExplainMemoryRequest`
- `ExplainMemoryResponse`

Inbox items carry reasons like:

- candidate
- claimed
- inferred
- stale
- contested
- superseded
- expired
- derived
- low-confidence
- ttl

## Kinds

- `fact`
- `decision`
- `preference`
- `runbook`
- `topology`
- `status`
- `pattern`
- `constraint`

## Scopes

- `local`
- `synced`
- `project`
- `global`

## Retrieval Routes

- `auto`
- `local_only`
- `synced_only`
- `project_only`
- `global_only`
- `local_first`
- `synced_first`
- `project_first`
- `global_first`
- `all`

## Retrieval Intents

- `general`
- `current_task`
- `decision`
- `runbook`
- `topology`
- `preference`
- `fact`
- `pattern`

## Status Values

- `active`
- `stale`
- `superseded`
- `contested`
- `expired`

## Rules

- long-term items require explicit scope
- global items require stronger promotion rules than project items
- every item should be attributable to a source
- every item should be allowed to decay, expire, or be superseded
- synthetic source input should be rejected
- near-duplicate items should collapse under a redundancy key
- route and intent should bias retrieval toward the smallest useful tier

## Source Quality

- `canonical`
- `derived`
- `synthetic`
