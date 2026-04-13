# 158 Clippy Warnings

- status: `closed`
- found: `2026-04-13`
- scope: all crates

## Summary

158 clippy warnings across the codebase. No correctness bugs, but code quality
debt that makes real issues harder to spot in CI output.

## Symptom

- Clippy output is noisy — real warnings hide in 158 lines of lint
- 6 identical if-blocks suggest duplicated logic
- 14 functions with 8-12 args are hard to call correctly

## Breakdown

- 52 collapsible if-statements
- 16 MutexGuard held across await (all in tests)
- 14 functions with too many arguments (8-12 vs clippy limit of 7)
- 6 inefficient `contains()` → should use `iter().any()`
- 6 derivable impls (could use `#[derive]`)
- 6 identical if-blocks (duplicated logic)
- 5 unnecessary reference creation
- 5 elidable explicit lifetimes
- 5 items after test modules
- Various minor: format-in-format, clone-on-copy, large variant size diff

## Fix Shape

- Run `cargo clippy --fix` for auto-fixable items (collapsible ifs, refs, lifetimes)
- Manual fix for identical if-blocks and too-many-args
- Consider `#[allow(clippy::too_many_arguments)]` where refactoring isn't worth it

## Evidence

- `cargo clippy --all-targets 2>&1 | grep "^warning:" | wc -l` → 158
