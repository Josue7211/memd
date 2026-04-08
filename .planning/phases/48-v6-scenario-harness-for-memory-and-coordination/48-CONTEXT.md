---
phase: "48"
name: "v6 Scenario Harness for Memory and Coordination"
created: 2026-04-06
---

# Phase 48: v6 Scenario Harness for Memory and Coordination — Context

This phase adds stable scenario-level harnesses for real memd memory and
coordination flows so autoresearch and nightly loops can run repeatable,
low-friction checks.

## Why This Exists

Current scenario command behavior is effectively one generic bundle-health smoke test.
The `v6` roadmap requires reusable, workflow-aligned targets for resume,
handoff, workspace retrieval, stale-session recovery, and coworking.

## Decisions

- Preserve the existing `memd scenario` command surface and extend it with named
  scenario workloads.
- Keep scenario outputs machine-readable and compact while still human-readable.
- Keep scoring logic additive and explicit so improvements and regressions are
  easy to diff.
- Make scenario names explicit and reject unsupported values immediately.

## Discretion Areas

- Exact point weights per check for each workflow.
- Whether future workflows should move to dedicated subcommands versus staying in
  `memd scenario`.
- How hard failures are interpreted by nightly loops (warn vs fail).
