# Scope and Visibility Isolation Not Enforced

status: open
severity: high
phase: Phase I
opened: 2026-04-14

## Problem

Scope (private/workspace/project/shared) and visibility (private/shared)
fields exist on memory items but are never checked at retrieval time.
One agent can read another's private items. No validation at handoff.
Agent identity isolation missing — all agents share same working memory.

## Fix

1. Add scope/visibility check to all retrieval endpoints
2. Per-agent working context isolation
3. Validate scope at handoff time
4. Audit all retrieval paths for enforcement gaps
