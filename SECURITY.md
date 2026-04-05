# Security

If you find a security issue in `memd`, do not file it in a public issue with exploit details.

## Report Process

- use GitHub Security Advisories if enabled for the repository
- otherwise contact the maintainers through the private channel configured by the project owner

If you are unsure whether something is a security issue, report it privately
first.

## What To Include

- what component is affected
- what version or commit you tested
- the minimal reproduction
- whether the issue leaks memory, secrets, or access control
- whether the issue crosses project, namespace, or source-trust boundaries
- whether the issue depends on a non-default backend or integration

## Disclosure Expectations

- give maintainers time to reproduce and patch the issue
- avoid publishing exploit details before a fix or mitigation exists
- if a public write-up is planned, coordinate timing with the maintainers

## High-Risk Areas

Pay extra attention to:

- source provenance and trust handling
- cross-project or cross-namespace memory leakage
- unsafe file handling in Obsidian, hooks, and multimodal ingest
- backend sidecar boundaries
- shell hook execution and environment propagation
- deserialization and request shaping across CLI, server, and sidecar boundaries

## Scope

Security issues include:

- data exposure
- auth or access control mistakes
- unsafe file handling
- injection or deserialization issues
- privilege boundary violations

Memory correctness bugs are not always security bugs, but they can become one fast if they leak private source or cross-project state.
