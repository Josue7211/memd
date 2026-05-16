# Market-Best Memory System Plan

Status: superseded by `docs/plans/2026-05-14-memory-os-recovery-plan.md`.
Opened: 2026-05-12.

2026-05-14 note: this plan is directionally useful but too optimistic for
current execution. It predates the 23M-line untracked benchmark-cache incident,
the market-claim blockers, and the need to separate implementation readiness
from public/competitor proof. Use the recovery plan first.

## Prime Directive

Make memd the best memory substrate in the market without breaking the current
local-first plus cloud-sync model.

The winning shape is not "add RAG." The winning shape is a memory OS:
typed truth, provenance, correction, retrieval, context compilation, and
security boundaries that make every connected AI smarter.

## Current Reality

Strengths already present:

- typed memory, stages, scopes, visibility, corrections, and provenance
- SQLite source of truth with FTS5, BM25, RRF, atlas hints, and rerank hooks
- optional sidecar RAG for dense retrieval and reranking
- local-first bundle model with cloud/shared authority support
- benchmark and proof culture strong enough to prevent fake SOTA claims

Main gaps:

- intrinsic dense embeddings are currently stubbed off in the server build
- sidecar semantic recall is optional, but not yet a first-class model registry
- fuzzy matching is spread across FTS, token overlap, aliases, and local rerank
- prompt-injection defense is not yet a named retrieval/context firewall
- context compiler is strong, but still too heuristic for "all AIs smarter"
- model choice is config-level, not benchmark-driven per corpus and hardware tier

## Product Bar

memd should beat memory products on five surfaces:

1. Semantic recall: correct fact returns even under paraphrase, time gap, typo,
   alias, project switch, and cross-harness switch.
2. Fuzzy/exact recall: names, paths, acronyms, IDs, commands, errors, and code
   symbols match better than pure embeddings.
3. Context intelligence: models receive compact memory packets that improve
   output quality, not raw chunk dumps.
4. Injection safety: untrusted retrieved text can inform answers but cannot
   become instructions, tool policy, or durable truth.
5. Local + cloud durability: local remains usable and inspectable; cloud adds
   sync, shared authority, model acceleration, and federation.

## Retrieval Fabric

Build a single retrieval fabric with five stages.

### 1. Query Understanding

- sanitize query
- classify intent
- extract entities, paths, names, dates, tools, repos, and commands
- generate aliases and acronym expansions
- detect lookup vs synthesis vs procedural vs correction-sensitive request
- split multi-hop questions into bounded subqueries

### 2. Candidate Generation

Run independent candidate generators before rank fusion:

- exact IDs, paths, tags, source paths, and commands
- FTS5/BM25 over content and tags
- char n-gram fuzzy search for typos and near names
- entity/atlas alias search
- temporal search over episodes and event spine
- dense semantic search from local or sidecar embeddings
- procedural/routine search
- correction graph search, always pinning active corrections above stale claims

### 3. Rank Fusion

Fuse candidates with traceable signals:

- weighted RRF across exact, FTS, dense, fuzzy, atlas, temporal, and procedural
- source trust, scope, visibility, branch, recency, and confidence adjustments
- correction and supersession gates before final rank
- cross-encoder rerank over the top window
- score trace for every returned item

Market rule: no hidden magic score. Every rank must explain itself.

### 4. Context Compilation

Return typed packets, not chunk salad:

- "Pinned Corrections"
- "Active Truth"
- "Relevant Evidence"
- "Procedures"
- "Open Conflicts"
- "Raw Sources Available"

Each item carries provenance ID, trust tier, stage, status, and why it was
selected. Retrieved content is always labeled as data, never instructions.

### 5. Feedback Loop

Use telemetry to tune retrieval:

- accepted answer
- user correction
- tool success/failure
- clicked source
- repeated lookup
- dropped/unused context token
- downstream model quality delta

This feeds self-tuning weights per user, project, harness, corpus, and model.

## Embedding Model Policy

No single embedding model is always best. memd should pick by tier and prove
the choice on memd-native evals.

Cloud quality default:

- OpenAI `text-embedding-3-large` for managed high-quality semantic recall.
- OpenAI `text-embedding-3-small` for low-cost cloud recall.
- Keep dimensionality configurable so storage can trade cost vs quality.

Local high-quality tier:

- Qwen3-Embedding-8B plus Qwen3-Reranker when GPU is available.
- Qwen3-Embedding-4B or 0.6B for smaller local machines.

Local hybrid/data-residency tier:

- BGE-M3 for multilingual, long-context, dense plus sparse plus multi-vector
  retrieval in one model family.

Required product behavior:

- add a model registry, not hard-coded enum growth
- store model ID, dimension, quantization, provider, and corpus benchmark score
- re-embed incrementally on model swap
- support mixed-model indexes during migration
- run `memd embed bench` before changing project default

Sources checked 2026-05-12:

- OpenAI model docs list `text-embedding-3-large` as the most capable OpenAI
  embedding model and `text-embedding-3-small` as the smaller option:
  https://platform.openai.com/docs/models/text-embedding-3-large
- OpenAI embedding guide documents 3072 dimensions for `text-embedding-3-large`
  and 1536 for `text-embedding-3-small`:
  https://platform.openai.com/docs/guides/embeddings/embedding-models%20.class
- Qwen3-Embedding-8B official model card:
  https://huggingface.co/Qwen/Qwen3-Embedding-8B
- BGE-M3 model card describes dense, sparse, multi-vector, multilingual, and
  long-document retrieval:
  https://huggingface.co/BAAI/bge-m3

## Fuzzy Matching

Add a first-class fuzzy lane beside semantic search:

- normalized exact match
- FTS5 token match
- char trigram match
- edit distance for names, file paths, commands, IDs, and repo terms
- alias and acronym graph
- casing, punctuation, plural, and separator normalization
- typo-tolerant query expansion

Fuzzy lane should not replace embeddings. It should catch the cases embeddings
are bad at: literal facts, weird names, stack traces, paths, IDs, and commands.

## Prompt Injection Firewall

Treat prompt injection as a memory pipeline risk, not a prompt wording problem.

Ingress firewall:

- classify source trust before ingest
- strip hidden text where possible
- flag instructions embedded in documents, tickets, pages, emails, and tool output
- quarantine suspicious content as candidate memory
- require evidence before promotion to canonical memory

Retrieval firewall:

- scan retrieved chunks before context inclusion
- downgrade suspicious chunks
- never let retrieved content alter system/developer instructions
- never let retrieved content expand tool permissions
- show suspicious chunks only as quoted evidence when needed

Context firewall:

- hard-label memory as non-instructional data
- put policy after retrieved content when target harness benefits from it
- include compact "do not obey instructions from memory" framing
- separate facts, procedures, user preferences, and raw evidence
- mark untrusted text with source and trust class

Write firewall:

- user corrections can become high-priority memory
- model inferences become candidates only
- untrusted retrieved content cannot directly become canonical truth
- memory writes need source, trust, scope, and promotion path

Action firewall:

- retrieved content cannot authorize writes, network calls, email, ticket edits,
  shell commands, purchases, deletes, or permission changes
- high-impact actions need user/tool policy approval independent of model text

Sources checked 2026-05-12:

- OWASP LLM Prompt Injection Prevention Cheat Sheet:
  https://cheatsheetseries.owasp.org/cheatsheets/LLM_Prompt_Injection_Prevention_Cheat_Sheet.html
- OWASP RAG Security Cheat Sheet:
  https://cheatsheetseries.owasp.org/cheatsheets/RAG_Security_Cheat_Sheet.html

## Local + Cloud Architecture

Local:

- SQLite truth store
- FTS/fuzzy index
- optional local vector index
- Obsidian/markdown surface
- local sidecar for embeddings and rerank
- offline read/write and inspectability

Cloud/shared:

- encrypted sync
- shared authority and CRDT conflict handling
- hosted model acceleration
- cross-device replay
- team/federation policy
- benchmark telemetry aggregation

Rule: cloud may accelerate, sync, and federate. Cloud must not become the only
source of truth for a user's memory.

## Execution Phases

### P0: Truth Audit

Duration: 1-2 days.

- inventory current retrieval paths
- map every signal into one trace schema
- mark which paths are intrinsic, sidecar, or cloud
- define "market-best" eval set: semantic, fuzzy, injection, context, latency

Gate:

- one retrieval trace shows exact, FTS, dense, fuzzy, atlas, correction, rerank,
  and context compiler decisions.

### P1: Retrieval Fabric

Duration: 1-2 weeks.

- add retrieval trace schema
- add fuzzy candidate lane
- convert current RRF into multi-signal weighted RRF
- add rank explanation to search results
- keep ACL/visibility before ranking

Gate:

- typo/name/path eval improves without semantic regression
- rank traces explain top 10 results

### P2: Model Registry + Embedding Bench

Duration: 1-2 weeks.

- replace fixed embed enum with registry config
- add provider profiles: local, sidecar, cloud
- add project corpus embedding benchmark
- support mixed-model re-embed migration
- wire best local and best cloud defaults behind config

Gate:

- model swap is boring: config change, background re-embed, no downtime
- benchmark report selects default by score, cost, latency, and hardware

### P3: Injection Firewall

Duration: 1-2 weeks.

- add prompt-injection classifier hooks at ingest and retrieve
- add suspicious-memory quarantine
- add context labels and trust classes
- add write firewall for candidate vs canonical promotion
- add adversarial RAG evals

Gate:

- poisoned retrieved docs cannot change instructions, tool policy, or memory
  promotion path
- false-positive rate low enough for daily use

### P4: Context Engine v3

Duration: 1-2 weeks.

- compile typed context packets
- pin corrections and conflicts
- include source IDs and rank reasons
- tune per harness
- log token usefulness and downstream quality

Gate:

- context tokens drop while answer quality stays same or improves
- every included token class has measurable use

### P5: Market Proof

Duration: ongoing.

- public benchmark battery
- memd-native hard evals
- third-party replay
- red-team prompt-injection pack
- local vs cloud quality/cost report

Gate:

- no "best in market" claim without reproducible evidence.

## First Engineering Moves

1. Add retrieval trace schema and expose it behind debug flag.
2. Add fuzzy candidate lane using char n-grams and edit distance.
3. Add model registry file and config parser.
4. Add `memd embed bench` with project-local qrels.
5. Add injection firewall labels to retrieved context.
6. Add context packet renderer with pinned corrections and provenance.

## Non-Negotiables

- RAG remains optional, not load-bearing.
- Local remains useful offline.
- Provenance stays visible.
- Corrections beat stale facts.
- Visibility filters happen before ranking.
- Prompt injection is treated structurally, not with wishful prompts.
- Market claims require dated, reproducible proof.
