# Contributing

Thanks for helping improve `memd`.

## Before You Start

Read these first:

- [README](./README.md)
- [Roadmap](./ROADMAP.md)
- [Branching Model](./docs/branching.md)
- [Release Process](./docs/release-process.md)
- [Maintainer Workflow](./docs/maintainer-workflow.md)
- [Infrastructure Facts](./docs/infra-facts.md)
- [Code of Conduct](./CODE_OF_CONDUCT.md)
- [Security Policy](./SECURITY.md)

`memd` is not a generic note store. Contributions should improve compact,
durable, inspectable memory behavior for agents.

The primary user story is agent continuity: `memd` should help Codex or another
agent persist working state, retrieve evidence, and compile durable knowledge
without depending on chat history alone.

## Good Contribution Areas

- retrieval quality
- provenance and evidence handling
- working-memory control
- contradiction handling
- Obsidian and agent integration quality
- compact API and CLI ergonomics
- cross-platform reliability
- tests, docs, and release hygiene

## Changes That Usually Do Not Fit

- transcript dumping as canonical memory
- hidden vendor lock-in
- features that enlarge memory without improving retrieval
- behavior that weakens provenance or trust boundaries
- project-specific hacks presented as general architecture

## Local Setup

Basic development flow:

```bash
cargo fmt --all
cargo test
```

The main workspace is Rust-first. The default local server runs on
`127.0.0.1:8787`.

Useful commands while developing:

```bash
cargo run -p memd-server
cargo run -p memd-client --bin memd -- healthz
cargo run -p memd-client --bin memd -- status --output .memd
cargo run -p memd-client --bin memd -- resume --output .memd --intent current_task
```

When working on bundle setup or agent integration, use `status --output .memd`
first. It now reports `setup_ready` and any missing bundle files so setup issues
are visible before deeper debugging.

## Local Workflow

- work on a dedicated branch
- start from the current `work/<milestone>` branch, then cut a scoped `feat/<area>` or `fix/<area>` branch
- run `cargo fmt --all`
- run `cargo test`
- keep changes small and scoped
- update docs when behavior changes

If your change affects public behavior, CLI output, docs, or roadmap-facing
capabilities, update those in the same branch.

## Branch and Release

See [Release Process](./docs/release-process.md) for branch-first workflow,
version history, and release conventions.
See [Branching Model](./docs/branching.md) for branch naming and commit
discipline.

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

## Scope Discipline

Prefer one coherent change set per branch.

Good examples:

- `feat/v2-rehydration`
- `fix/obsidian-writeback-uri`
- `docs/release-checklist`

Bad examples:

- mixing unrelated docs, refactors, and features in one branch
- burying two roadmap slices in one PR
- “while I was here” cleanup across unrelated crates

## Testing Expectations

- run `cargo fmt --all`
- run `cargo test`
- add or update tests when behavior changes
- if you change a contract, update the docs in the same branch

If you cannot run a relevant verification step, say so clearly in the PR.

## Documentation Expectations

Update docs when you change:

- public CLI behavior
- API contract
- backend ownership or integration boundaries
- branch/release workflow
- roadmap-visible product direction
- memory substrate workflow and evidence compilation

Relevant docs usually live under `docs/`, but roadmap and maintainer workflow
changes may also require updates to `README.md`, `CHANGELOG.md`, and
`.planning/` artifacts.

If you change bundle bootstrap, Obsidian workflow, or token-efficiency behavior,
also update the release-facing quickstart text so new users can discover the
current happy path without reading source.

## Infra Claims

Do not guess deployment or networking facts.

Before stating anything about:

- Cloudflare tunnels
- domains or subdomains
- VM ownership
- public reachability
- LAN vs Tailscale accessibility

verify locally first and use [Infrastructure Facts](./docs/infra-facts.md) as
the repo truth source. If you cannot verify a claim, mark it `unverified`
instead of filling in the gap from context.

## Pull Requests

Please include:

- a short summary of the change
- tests, if behavior changed
- doc updates for new public behavior
- the branch name and what phase or scope it belongs to
- whether the change lands on a milestone branch or a scoped feature branch

Good PRs also explain:

- why this change belongs in `memd`
- what constraints or tradeoffs were considered
- what was verified
- what remains intentionally out of scope

## Issues

Use the issue templates for:

- bug reports
- feature requests

Please include repro steps, affected commands or APIs, and the branch or
version context when you have it.

## Security

Do not open a public issue with sensitive exploit detail.

Follow [SECURITY.md](./SECURITY.md) instead.

## Design Rule

If the change makes memory larger without making retrieval better, it is probably the wrong tradeoff.
