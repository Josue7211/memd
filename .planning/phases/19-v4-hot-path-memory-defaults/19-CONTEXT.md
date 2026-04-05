# Phase 19 Context: `v4` Hot-Path Memory Defaults

## Why This Phase Exists

Short-term memory has to feel instant or the whole system loses credibility.
The previous resume flow still pulled semantic fallback automatically when a RAG
backend was configured, which put long-term retrieval on the hot path.

This phase restores the intended contract:

- short-term memory first
- semantic recall second
- opt in when deeper recall is actually needed

## Inputs

- bundle-backed resume and handoff flows
- LightRAG-compatible semantic fallback support
- user requirement that short-term memory stay fast

## Constraints

- do not remove semantic recall support
- keep evaluation able to inspect semantic health
- update generated bundle docs so new bundles teach the fast-default workflow

## Target Outcome

Default `resume` and `handoff` should stay local and fast, with semantic recall
available only through explicit flags.
