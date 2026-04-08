# Live Truth

`memd` should behave like a truth-first external cortex, not a refresh-only note generator.

## Hard Invariants

- normal memory operations may observe, summarize, compile, and learn
- normal memory operations may not mutate shared runtime state
- shared runtime mutation is only allowed in explicit repair or install flows
- the freshest verified local truth must outrank older ambiguous memory
- raw events are not prompt material; they must be compiled into compact truth items first
- project-local live truth is the default; cross-project promotion requires explicit validation
- session-local and tab-local scope should stay visible when multiple Codex
  tabs share one project

## Runtime Doctrine

The live policy snapshot should enforce these defaults:

- read raw source once, then prefer compiled memory
- reopen raw only on change, doubt, or repair
- update memory through events and patches, not full rewrites
- keep visible memory objects as the product truth
- keep session and tab identity visible on memory surfaces instead of inferring
  which Codex tab did the work
- treat semantic retrieval as a fallback lane, not the source of truth
- propose repeated patterns as skills, sandbox test them, then gate activation
  by evaluation and policy
- expose skill lifecycle state in a visible policy surface
- write batch artifacts for review, activate, and apply receipts so downstream
  flows stay explicit
- post apply receipts to a durable server sink after local queue drain
- persist applied skill records separately from apply receipts so activation is
  queryable, not just audited
- expose a query lane for stored apply receipts and activation records

## Biological Principles Worth Copying

- small, high-priority working memory
- fresh local truth precedence
- layered memory:
  - live truth
  - task and project memory
  - durable policy
  - promoted abstractions
- selective consolidation instead of replaying full history
- cue-driven retrieval under current task pressure
- compressed cognition for energy and token efficiency

## Biological Failure Modes To Reject

- confabulation without provenance
- hidden belief drift
- contradiction collapse without inspection
- unbounded replay of raw history
- unsafe side effects while "thinking"

## Product Bar

The next answer should be able to run from:

- live truth
- compact task state
- compiled knowledge
- delta context

It should not need to reread raw files just to recover something that changed moments ago.
