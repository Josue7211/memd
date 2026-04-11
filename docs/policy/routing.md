# Routing

`memd` uses a typed retrieval router so clients can ask for the right memory tier without pulling the entire corpus.

## Routes

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

## Intents

- `general`
- `current_task`
- `decision`
- `runbook`
- `topology`
- `preference`
- `fact`
- `pattern`

## Default Resolution

When the route is `auto`, `memd` resolves it from intent:

- `general` -> `all`
- `current_task` -> `local_first`
- `decision` -> `project_first`
- `runbook` -> `project_first`
- `topology` -> `project_first`
- `preference` -> `global_first`
- `fact` -> `all`
- `pattern` -> `global_first`

## Ranking

The router affects:

- scope ordering
- hard scope filtering for `*_only` routes
- retrieval scoring
- compact context ordering

The intent is not to duplicate memory in multiple layers. It is to ask the manager for the smallest useful slice from the right tier.
