# Docs Conventions

This repo treats docs as part of the recovery and memory system. A fresh-session
agent should be able to recover quickly without guessing which file is truth.

## Primary Docs

These are the first-class recovery surfaces and must stay short, current, and
high-signal:

- [[START-HERE]]
- [[ROADMAP]]
- [[docs/WHERE-AM-I.md|WHERE-AM-I]]
- [[docs/verification/milestones/MILESTONE-v1.md|MILESTONE-v1]]
- phase summary notes in `docs/phases/`
- backlog notes in `docs/backlog/`

### Rules For Primary Docs

- include a compact snapshot near the top
- keep one-screen readability where possible
- use wiki links for detail
- avoid giant prose dumps
- never duplicate conflicting status

## Secondary Docs

Everything in `docs/core/`, `docs/policy/`, `docs/reference/`, `docs/strategy/`,
`docs/verification/`, and `docs/codebase/` is secondary unless explicitly promoted.

### Rules For Secondary Docs

- start with a banner pointing back to [[ROADMAP]] and [[docs/WHERE-AM-I.md|WHERE-AM-I]]
- do not pretend to be the current source of truth
- link to primary docs instead of restating project state
- keep an `INDEX.md` in each major area so agents have a safe entrypoint

## Root Docs

Root should only contain:

- repo entrypoints: `README.md`, `START-HERE.md`, `ROADMAP.md`
- OSS standard docs: `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, `CHANGELOG.md`, `LICENSE`
- harness discovery/config docs: `AGENTS.md`, `CLAUDE.md`
- tiny redirect stubs when a historic root path must stay discoverable

## Status Vocabulary

Allowed status words for roadmap-facing docs:

- `pending`
- `in_progress`
- `blocked`
- `verified`
- `verified_with_audit_tail`
- `complete`

Meanings:

- `verified`: engineering verification passed
- `verified_with_audit_tail`: engineering verification passed, cleanup/audit still open
- `complete`: human-tested and accepted

## Lint Targets

The doc lint should fail when:

- a primary doc is missing its snapshot/start header
- a secondary doc lacks a banner
- root contains deep docs that belong under `docs/`
- roadmap-facing docs use non-standard status words
