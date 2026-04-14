# Source Quality Ranking Not Enforced in Retrieval

status: open
severity: medium
phase: Phase I
opened: 2026-04-14

## Problem

Theory says canonical > promoted > candidate in retrieval. Code has
confidence field but doesn't use it for ranking. Status (candidate quality)
can outrank Fact (canonical quality) if in right scope. Trust hierarchy
exists in scoring helpers but doesn't override scope-based ranking.

## Fix

1. Add confidence multiplier to context_score()
2. Weight by source_quality field in retrieval ranking
3. Canonical items should always outrank candidate items, regardless of scope
