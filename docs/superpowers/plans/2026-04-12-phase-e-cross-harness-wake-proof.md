# Phase E Cross-Harness Wake Proof Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the wake-packet audit tail by proving the lightweight shared-surface model works across harnesses, especially for Claude Code.

**Architecture:** Treat this as a bounded Phase E follow-up, not a new architectural branch. Keep the visible surface model unified (`wake.md`, `mem.md`, `events.md`), keep the wake packet small by construction, and prove Claude Code remains on a wake-only boot path with explicit deeper recall.

**Tech Stack:** Rust, `memd-client`, bundle `.memd` surfaces, harness manifest/build code, wake renderer, Cargo tests, roadmap/phase docs.

---

## Why This Plan Exists

Phase E already proved the wake packet compiler in principle. The remaining gap
is not “invent wake packets”; it is “prove the compact wake model survives
across harnesses without regrowing duplicate context or Claude Code token
bloat.”

This plan closes that gap in one architectural loop:

1. shared visible surfaces
2. hard wake budget behavior
3. Claude Code light-boot proof

## Workstreams

### 1. Shared Visible Surface Contract

- enforce `wake.md`, `mem.md`, `events.md` as the only visible shared surfaces
- keep harness-specific files bridge-only
- remove any duplicate visible payload generation or stale docs that imply it
- add tests that fail if duplicate visible surfaces return

### 2. Layered Wake Packet Contract

- keep `wake.md` as `L0 + L1`, not a deep recall surface
- preserve protocol and correction rules under pressure
- make deeper recall explicit through `memd lookup` and `memd resume`
- enforce stricter wake budgets by harness, with Claude Code strictest

### 3. Claude Code Proof

- prove Claude imports only `wake.md` by default
- keep `mem.md` and `events.md` cold-path for Claude
- verify bridge wording matches real behavior
- expose wake-size regressions through tests and later observability surfaces

## Files

**Modify**

- `crates/memd-client/src/harness/*.rs`
- `crates/memd-client/src/harness/preset.rs`
- `crates/memd-client/src/runtime/resume/wakeup.rs`
- `crates/memd-client/src/main_tests/bootstrap_harness_tests/mod.rs`
- `crates/memd-client/src/evaluation_runtime_tests/*`
- `integrations/*/README.md`
- `docs/core/setup.md`
- `docs/phases/phase-e-wake-packet-compiler.md`
- `ROADMAP.md`
- `docs/WHERE-AM-I.md`

## Pass Gate

- non-Claude harnesses expose only shared visible surfaces
- Claude Code stays on a wake-only boot path
- Claude wake packet stays within the strict budget
- docs no longer imply duplicated visible surfaces or false Claude parity
- no old visible-surface filenames remain in active runtime/docs

## Evidence

- harness manifest tests
- wake budget tests
- Claude bridge tests
- `cargo check -p memd-client`
- targeted boot-path tests

## Fail Conditions

- any harness reintroduces duplicate visible surfaces
- Claude boot silently loads `mem.md` or `events.md`
- wake packet budgets are bypassed by docs/runtime drift
- roadmap wording diverges from actual harness behavior

## Rollback

- revert any compression or bridge change that preserves smaller packets by hiding corrections, provenance, or required protocol behavior

## Result

Phase E closes with a real cross-harness proof instead of a Codex-only packet
win, and Claude Code stops being the main boot-bloat failure mode.
