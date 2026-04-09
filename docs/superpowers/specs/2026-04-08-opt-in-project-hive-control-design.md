# Opt-In Project Hive Control

## Goal

Make `memd` support an explicit project-level hive mode that can be enabled and
disabled per repo/workspace, while keeping live session join and cross-project
pairing as separate actions.

The product shape is:

- `memd hive-project` controls whether a project is hive-enabled.
- `memd hive` joins/publishes the current live session.
- `memd hive-link` remains the manual safe link between different projects.
- `memd hive-fix` remains the repair path for stale bundles and drifted base URLs.

## Why

Right now the live hive can be repaired and published, but project membership is
still too implicit. The user wants:

- opt-in project hiveminds
- enable/disable controls per project
- a manual safe-link path for different projects
- a clear separation between live membership and persistent project anchoring

## User-Facing Commands

### `memd hive-project --enable`

Enables hive mode for the current project bundle.

Expected behavior:

- writes persistent project-hive state into the bundle config
- ensures the project is anchored for future sessions
- refreshes the current bundle heartbeat
- makes future sessions in that project auto-join the shared hive on startup
- keeps the shared hive URL behavior intact

### `memd hive-project --disable`

Disables hive mode for the current project bundle.

Expected behavior:

- turns off automatic project hive join for future sessions
- preserves the rest of the bundle config
- does not delete unrelated memory files
- leaves manual `memd hive` available for explicit one-off joins

### `memd hive-project --status`

Reports whether the project hive is enabled and whether the current bundle is
actively joined.

### `memd hive`

Keeps its current meaning:

- join/publish the current session
- ensure the live heartbeat is visible on the shared server
- respect the project hive setting if it is enabled

### `memd hive-link`

Keeps its current meaning:

- temporary safe pairing between two live sessions
- use when the sessions are from different projects or need short-lived trust

### `memd hive-group-link`

Keeps its current meaning:

- persistent trust anchor for a project/workspace
- used by `hive-project --enable` as the underlying anchor mechanism

### `memd hive-fix`

Keeps its current meaning:

- repair local bundles onto the shared hive URL
- rewrite stale localhost base URLs
- regenerate launchers
- re-publish heartbeat

## Data Model

Add a project-hive toggle to bundle runtime/config state.

Proposed fields:

- `hive_project_enabled: bool`
- `hive_project_anchor: Option<String>`
- `hive_project_joined_at: Option<DateTime<Utc>>`

Notes:

- `hive_project_enabled` is the opt-in gate.
- `hive_project_anchor` identifies the persistent project hive anchor.
- `hive_project_joined_at` is for audit/debugging.
- Existing hive/session fields stay intact.

## Behavior

### Enable Flow

When `memd hive-project --enable` runs:

1. Resolve the current bundle.
2. Create or refresh the persistent project hive anchor.
3. Set `hive_project_enabled=true`.
4. Ensure the bundle is pointed at the shared hive base URL when needed.
5. Publish the current session heartbeat.
6. Regenerate launcher/env files so future sessions inherit the hive behavior.

### Disable Flow

When `memd hive-project --disable` runs:

1. Resolve the current bundle.
2. Set `hive_project_enabled=false`.
3. Leave the rest of the bundle state alone.
4. Keep manual `hive` available for explicit one-off session join.

### Startup Flow

If a project hive is enabled:

- new sessions in that project should auto-join the hive on startup
- startup launchers should keep the shared hive URL
- heartbeats should continue to publish to the shared server

If a project hive is disabled:

- automatic project hive join should not happen
- explicit `memd hive` still works when a user wants to join manually

### Cross-Project Flow

If two different projects want to collaborate:

- use `memd hive-link`
- do not use the project hive anchor as a cross-project shortcut
- keep the safe-link handshake explicit and auditable

## Implementation Shape

The implementation should stay split into small responsibilities:

- command parsing and summaries in `crates/memd-client/src/main.rs`
- bundle config/runtime persistence in the same file where the bundle config
  helpers live
- anchor lifecycle in the hive-group-link path
- startup launcher generation in the agent profile renderers
- discovery and live visibility in awareness/status

Prefer additive changes over renaming existing compat surfaces:

- keep `hive`
- keep `hive-link`
- keep `hive-group-link`
- keep `hive-fix`

## Error Handling

- If the bundle cannot be read, fail with a clear bundle-specific message.
- If the project hive is enabled but the bundle is still pointed at localhost,
  repair it before publishing.
- If the anchor cannot be created, report the failure and do not claim the
  project hive is live.
- If disable succeeds, report that the project hive is off even if the session
  heartbeat remains temporarily visible until it expires.

## Tests

Add coverage for:

- enabling a project hive writes the new toggle and publishes a heartbeat
- disabling a project hive clears the toggle without breaking the rest of the bundle
- status surfaces enabled vs disabled project hive state
- startup launcher output includes the shared hive URL only when enabled
- `hive-link` remains distinct from project hive enablement
- `hive-fix` still repairs stale localhost bundles

## Non-Goals

- Replacing live session join with the persistent anchor
- Auto-enabling project hive for every repo by default
- Removing the manual safe-link workflow
- Breaking the current compatibility aliases for existing hive commands

