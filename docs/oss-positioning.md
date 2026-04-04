# OSS Positioning

## Goal

`memd` should be usable by anyone, not only by one homelab or one agent stack.

## Requirements

- standalone repo
- cross-platform core binaries for Linux, macOS, and Windows
- clean HTTP API
- optional local-only mode
- optional LAN deployment mode
- pluggable semantic backends
- open client adapters
- reusable Rust SDKs
- a small CLI for humans and agent runners
- examples for common agent setups

## First Shipping Story

A single developer should be able to:

1. run `memd` locally
2. store structured memories
3. retrieve compact context
4. sync active state across machines
5. optionally attach a semantic backend for long-term retrieval

## First-Class Supported Integrations

- Claude Code
- Codex
- Mission Control
- OpenClaw

## Design Constraint

The API and schema must stay generic enough that people can plug in other agents, editors, and runtimes without inheriting your personal setup.

Linux-specific deploy helpers are fine, but the core product must not depend on them.
