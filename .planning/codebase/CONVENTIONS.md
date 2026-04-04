# Conventions

## General Style

The codebase is pragmatic Rust with strong preference for explicit structs,
serializable request/response types, and straightforward orchestration logic.

## API Shape

- request and response types live in `crates/memd-schema/src/lib.rs`
- server endpoints mirror those types closely
- client methods map directly onto server routes

## Routing and Policy

- retrieval is guided by explicit `route` and `intent`
- compact responses are the default hot path
- policy is encoded in server logic rather than hidden prompt behavior

## Error Handling

- `anyhow::Context` is used heavily for operational clarity
- server handlers convert internal errors through a shared `internal_error` path
- CLI commands usually bubble errors directly with context

## Integration Pattern

- keep external systems behind documented boundaries
- prefer a single adapter path instead of backend-specific branches
- bundle-first config overrides environment fallbacks

## Documentation Pattern

- docs are direct and architectural rather than marketing-heavy
- policy and contract docs are first-class, not afterthoughts

## Current Convention Pressure

- `main.rs` files are carrying both command routing and product logic
- some newer concepts are documented faster than they are normalized into smaller modules
- the repo values explicitness over abstraction, which is good, but the binaries are getting dense
