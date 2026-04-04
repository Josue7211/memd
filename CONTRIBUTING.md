# Contributing

Thanks for helping improve `memd`.

## Local Workflow

- work on a dedicated branch
- start from the current `work/<milestone>` branch, then cut a scoped `feat/<area>` or `fix/<area>` branch
- run `cargo fmt --all`
- run `cargo test`
- keep changes small and scoped
- update docs when behavior changes

## Branch and Release

See [Release Process](./docs/release-process.md) for branch-first workflow,
version history, and release conventions.
See [Branching Model](./docs/branching.md) for branch naming and commit
discipline.
See [Code of Conduct](./CODE_OF_CONDUCT.md) for project interaction standards.

## What We Care About

- token efficiency
- compact retrieval
- stable APIs
- source provenance
- duplicate collapse
- clear scope boundaries

## What Not To Add

- synthetic memory filler
- transcript dumps as canonical truth
- personal environment assumptions
- hidden coupling between clients and storage backends

## Pull Requests

Please include:

- a short summary of the change
- tests, if behavior changed
- doc updates for new public behavior
- the branch name and what phase or scope it belongs to
- whether the change lands on a milestone branch or a scoped feature branch

## Design Rule

If the change makes memory larger without making retrieval better, it is probably the wrong tradeoff.
