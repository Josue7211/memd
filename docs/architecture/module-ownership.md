# Module Ownership

This map keeps feature work out of oversized catch-all files. Prefer adding a
small sibling module over extending a large parent module.

## Server

- `crates/memd-server/src/routes.rs`: route registration and thin handlers.
- `crates/memd-server/src/routes_tests.rs`: route-level tests.
- `crates/memd-server/src/store.rs`: persistence contracts and database
  behavior. Split by store domain before adding large new behavior.
- `crates/memd-server/src/tests/`: integration-style server behavior grouped
  by route or memory domain.

## Client

- `crates/memd-client/src/cli/args.rs`: top-level command enum and small
  shared CLI args only.
- `crates/memd-client/src/cli/args_skill.rs`: skill command argument shapes.
- `crates/memd-client/src/benchmark/`: benchmark runtime, dataset loading, and
  report rendering split by responsibility.
- `crates/memd-client/src/bundle/`: bundle creation, initialization, and
  maintenance runtime code. Add new bridge/runtime domains as sibling modules.
- `crates/memd-client/src/runtime/resume/`: resume and wake packet assembly.

## Schema

- `crates/memd-schema/src/lib.rs`: public schema types and constants.
- `crates/memd-schema/src/tests.rs`: schema tests and serialization fixtures.

## Dashboard

- `apps/dashboard/app/routes/`: route-level data orchestration.
- `apps/dashboard/app/components/ui/`: reusable UI controls shared across
  routes.

## Guardrails

- Keep source files below `MEMD_HYGIENE_MAX_SOURCE_LINES`, default `3000`.
- Run `scripts/verify/repo-hygiene-guard.sh` before broad cleanup commits.
- Move repeated route UI into `components/ui` before adding another route-local
  copy.
- Move repeated CLI arg groups into `args_*.rs` before extending `args.rs`.
