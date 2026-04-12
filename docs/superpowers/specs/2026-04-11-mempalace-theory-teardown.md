# mempalace Theory Teardown for memd

## Why This Exists

We need to know what `mempalace` actually proves, what is benchmark theater, and what `memd` should steal or reject.

This is a theory document, not a branding reaction.

## What We Tested

Local repo inspected:

- `../mempalace`

Code paths read:

- `README.md`
- `benchmarks/BENCHMARKS.md`
- `benchmarks/longmemeval_bench.py`
- `mempalace/searcher.py`
- `mempalace/palace_graph.py`
- `mempalace/layers.py`

Local tests run:

- `uv run --project ../mempalace python -m pytest ../mempalace/tests/test_searcher.py ../mempalace/tests/test_palace_graph.py -q`
- result: `38 passed`

- `uv run --project ../mempalace python -m pytest ../mempalace/tests/test_layers.py -q`
- result: `41 passed`

## Hard Findings

### 1. The strongest thing in mempalace is not the palace metaphor

The strongest thing is:

- raw verbatim storage
- simple embedding retrieval
- targeted reranking and query heuristics

Their own README says:

- `96.6%` LongMemEval comes from raw mode
- AAAK compression regresses
- room boost is not moat
- contradiction detection is not fully wired

## 2. Their benchmark gains are mostly retrieval-stage gains

From `benchmarks/BENCHMARKS.md` and `longmemeval_bench.py`:

- raw baseline is already very high
- later gains come from:
  - keyword overlap
  - temporal boosts
  - assistant-turn indexing
  - preference extraction
  - quoted phrase boosts
  - person-name boosts
  - optional rerank

This means the real lesson is:

- keep full signal
- retrieve smarter
- package results better

Not:

- build a cute palace ontology and call it solved

### 2a. Some gains are benchmark-shaped, not universal theory

The benchmark code adds targeted boosts for:

- temporal expressions
- assistant-reference questions
- preference questions
- quoted phrases
- person names

This is useful because it reveals real failure modes.

But it also means:

- some gains come from benchmark-aware heuristics
- not all gains should be treated as universal memory architecture

Lesson for `memd`:

- keep the general retrieval primitive
- reject overfitting as theory

## 3. mempalace has a useful wake-up idea, but weak memory typing

`layers.py` is important.

It has:

- `Layer0` identity
- `Layer1` essential story
- `Layer2` on-demand wing/room retrieval
- `Layer3` deep search

This is good because it proves:

- not all memory should load at once
- small wake packet matters
- deeper recall should be conditional

But it is still weak compared to what `memd` needs because it does not cleanly separate:

- session continuity
- episodic memory
- semantic memory
- procedural memory
- correction flow

### 3a. Their layers are loading-depth layers, not memory-kind layers

`mempalace` answers:

- how much should I load right now?

It does not fully answer:

- what kind of memory is this?

`memd` still needs explicit kinds:

- session continuity
- episodic memory
- semantic memory
- procedural memory

## 4. The palace graph is navigation, not truth

`palace_graph.py` builds graph edges from metadata:

- nodes = rooms
- edges = shared rooms across wings

This is useful as:

- navigation
- tunnel discovery
- cross-domain traversal

But it is not:

- truth repair
- canonical memory logic
- contradiction resolution
- procedural learning

This directly supports our newer theory:

- `memory atlas` is a navigation layer over truth
- it is not the truth itself

### 4a. The graph is metadata-driven and therefore thin

The graph uses:

- room labels
- wing labels
- hall labels
- dates

That makes it:

- practical
- cheap
- useful for traversal

But weak as:

- deep reasoning substrate
- trust system
- correction system

## 5. mempalace is strong at retrieval, weak at live memory

What it does well:

- offline retrieval
- raw storage
- searchable memory
- wake-up layering

What it does not seem to center:

- live correction overwrite
- active session continuity
- multiharness shared memory
- procedural memory as first-class substrate
- hive coordination

So it is much closer to:

- very strong retrieval-first memory

than:

- full second-brain operating system

### 5a. Mining is simple and truth-preserving

`miner.py` is blunt:

- route file
- chunk file
- store verbatim chunks

This is a strength.

It keeps ingestion understandable and truth-preserving.

It is also a limit:

- weak live-state typing
- weak event-native memory model
- weak procedural extraction path

## What memd Should Steal

### Steal 1: Raw-first doctrine

Do not throw away source text, artifacts, or events early.

### Steal 2: Small wake packet doctrine

Load tiny critical context first.

### Steal 3: Retrieval-stage optimization

Real gains come from:

- exact phrase support
- name/entity support
- temporal support
- type-aware retrieval
- optional rerank

Also:

- assistant-turn retrieval when needed
- synthetic recall aids only if source linkage stays intact

### Steal 4: Navigable memory topology

Keep:

- regions
- neighborhoods
- tunnels
- bridges

but under `memory atlas`, not palace cosplay.

### Steal 5: Loading-depth discipline

Keep explicit surfaces for:

- wake packet
- on-demand expansion
- deep search

But map them over typed memory, not instead of typed memory.

### Steal 6: Honest benchmark discipline

Their note correcting README claims is actually good research hygiene.

## What memd Should Reject

### Reject 1: Palace metaphor as architecture

Useful for navigation.
Not enough for a full memory model.

### Reject 2: Compression-first framing

Their own results show lossy compression hurts benchmark recall.

`memd` should compress compiled context, not destroy truth.

### Reject 3: Flat retrieval-only memory

We need more than search.

We need:

- resume
- continuity
- procedural learning
- live correction
- multiharness shared memory

### Reject 4: Semantic search as the whole system

Strong retrieval is necessary.
Not sufficient.

### Reject 5: Benchmark patch pile as product theory

If the model becomes:

- one regex for one benchmark
- one boost for one miss
- one filter for one evaluation set

then theory collapses into overfitting.

`memd` should generalize those lessons into primitives:

- temporal retrieval
- assistant-turn retrieval
- preference retrieval
- entity retrieval
- exact-span retrieval

## Theory Upgrades for memd Triggered by This Teardown

### Upgrade 1

The core moat should be:

- raw event spine
- typed memory
- wake packet compiler
- live correction loop
- multiharness continuity

Not:

- optional semantic backend

### Upgrade 2

`Memory atlas` should replace graph-as-truth thinking.

Atlas is:

- multidimensional navigation
- progressive expansion
- linked traversal

not:

- the canonical layer itself

### Upgrade 3

Session continuity must become first-class.

`mempalace` proves wake packets matter.
`memd` must push further:

- what were we doing
- where did we stop
- what changed
- what next

### Upgrade 4

Procedural memory must be native.

`mempalace` does not solve this deeply.
`Hermes` points here.
`memd` should own it.

## Practical Conclusion

`mempalace` proves that the field has underestimated:

- raw truth retention
- simple retrieval baselines
- small wake-up context

But it does not yet prove the full `memd` vision.

That vision still needs:

- multiharness second brain
- session continuity
- typed memory kinds
- correction and provenance as first-class
- hive memory
- latency briefing
- procedural learning

## Final Verdict

`mempalace` is a very strong retrieval-first memory system.

It is not yet the full memory operating system we are aiming for.

The correct move is:

- steal its strongest retrieval principles
- reject its limiting metaphor as the full model
- build `memd` as the bigger system above that baseline
