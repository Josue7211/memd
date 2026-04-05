# Phase 38 Context: `v5` Branch and Scope Recommendations

## Why This Phase Exists

`memd` now knows who is active, what they own, what is blocked, and what kind
of coordination lane a task belongs to. The next gap is helping sessions choose
clean work boundaries, especially branches and scopes, before implementation
starts.

## Inputs

- shared tasks with coordination modes
- claim scopes and ownership guards
- coordination inbox and stale-session recovery

## Constraints

- keep recommendations lightweight and non-destructive
- avoid turning recommendations into hidden branch management automation
- stay compatible with the existing CLI, MCP, and git workflow model

## Target Outcome

The next phase should recommend cleaner branches and scopes for active shared
tasks so simultaneous sessions split work with less friction.
