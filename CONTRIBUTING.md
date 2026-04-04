# Contributing

Thanks for helping improve `memd`.

## Local Workflow

- run `cargo fmt --all`
- run `cargo test`
- keep changes small and scoped
- update docs when behavior changes

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

## Design Rule

If the change makes memory larger without making retrieval better, it is probably the wrong tradeoff.
