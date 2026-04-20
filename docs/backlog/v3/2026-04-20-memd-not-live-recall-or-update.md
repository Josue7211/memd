---
status: open
severity: critical
phase: A3
opened: 2026-04-20
scope: memd-live-recall
---
# memd Does Not Do LIVE Recall on User Messages

status: open
severity: critical
phase: A3 (continuity / wake / live recall)
opened: 2026-04-20

## Summary

memd is advertised as LIVE — durable truth surfaced in the working
conversation, not a passive store. In practice it is NOT live: the
only surfacing channel is the UserPromptSubmit hook at
`.memd/hooks/memd-context.sh`, which runs `memd wake` each turn and
emits the SAME fixed `wake.md` bootstrap regardless of what the user
just said. Zero reactivity to user message content, zero content-based
recall, zero inline injection of relevant durable nodes.

When the user says "mempalace", `memd lookup --query "palace"` returns
6 matches with top node at 0.92 confidence — but the agent sees none
of those in context. It either claims ignorance or proactively guesses
which term to lookup. Both break the "LIVE" promise.

## Problem (not just wake — the whole live loop is broken)

### 1. The UserPromptSubmit hook is static

`.memd/hooks/memd-context.sh` is the ONLY surfacing path during a live
conversation. It does:

```
memd wake --output .memd --project ... --agent ... --intent current_task
```

That command is content-agnostic — it ignores the user's actual
message text. Every turn produces the same `wake.md` bootstrap
(durable truth + focus + preferences + continuity, ~30 lines). When
the user types "mempalace", there is no path for memd to react to the
word "mempalace" and inject the 6-match lookup result into the
pre-model context.

### 2. Wake itself is thin

Even the bootstrap that does fire surfaces only `decisions` + `status`
+ `focus` + a short tail. Nodes with `type=semantic+canonical` and
kind `fact` / `topology` — which describe architecture, inspiration
repos, data-flow maps — stay invisible regardless of confidence.

### 3. No message-keyed lookup fanout

The hook could extract salient terms from the user message and fire
`memd lookup` per term before the model sees the prompt. It doesn't.
There is no `memd lookup --from-stdin` or `memd context --message
"..."` mode wired into the hook chain.

### 4. Agent is forced to simulate what memd should do

Because the store is opaque at prompt time, the agent has to either:

- claim ignorance and wait for correction (the 2026-04-20 mempalace
  incident), or
- proactively `memd lookup` on every speculative term each turn —
  noisy, expensive, arbitrary.

Both are workarounds. Neither is "LIVE memd".

## Example: mempalace incident (2026-04-20)

Session on `memd` project. User asks "what does mempalace do". Agent
has no surfaced context. Wake bootstrap did not include mempalace.
Agent says "not a memd term". User pushes back: "if you don't know
mempalace, memd is not working".

After manual `memd lookup --query "palace"`:

`memd lookup --output .memd --query "palace"` does not surface inspiration-repo context even
when it is present as durable truth in project scope. Example: mempalace
(`milla-jovovich/mempalace`) is the upstream repo memd borrows its
embedding strategy, dedup patterns, and longmemeval bench harness shape
from. `memd lookup --query "palace"` returns 6 matches with the top node
at 0.92 confidence:

- `09224ec9` (fact, semantic+canonical, 0.92) — top-level mempalace description
- `b66692cc` (topology, semantic+canonical, 0.80) — A2-10 embedding strategy
- `75d47b00` (topology, semantic+canonical, 0.80) — inspiration architecture

None of these surface on wake. The wake bootstrap only shows 2 durable
truths + 4 "via memd lookup" links to decisions/status/focus. Inspiration
nodes — which are the project's architectural foundation and relevant
on virtually every session — are invisible until the agent explicitly
queries. This forces the agent to either:

1. Claim ignorance ("not a memd term") and get corrected by the user, or
2. Pre-emptively lookup every session on hunches — noisy, slow, arbitrary.

Both violate the wake-protocol claim "Read first. Durable truth beats
transcript recall. Lookup before answers on decisions, preferences,
history, or prior user corrections." The high-confidence inspiration
nodes ARE durable truth and they ARE decisions — they set every
retrieval / embedding / dedup design call — but wake does not elevate
them.

## Reproduction

1. Fresh session on `memd` project. Wake bootstrap runs.
2. User asks "what does mempalace do" / "mem palace".
3. Agent has no surfaced context. Has to invent OR explicitly lookup.
4. `memd lookup --query "palace"` returns high-confidence match that
   should have been surfaced on wake.

## Why wake misses it

- Wake surfaces `decisions` + `status` + `focus` + a small tail. Nodes
  tagged `type=semantic+canonical` with kind=`fact` or `topology` are
  not in any of those channels even at 0.92 confidence.
- The wake budget trims aggressively — "startup trimmed; use memd lookup
  or memd resume for deeper recall" — but the trim heuristic does not
  elevate semantic nodes whose content is referenced across many code
  paths (embedding, dedup, bench).
- No "project-topology snapshot" channel exists in wake output. The
  repo's inspiration map is nowhere in the default recall surface.

## Goalpost

memd must BEAT both mempalace and supermemory. That means:

- **LIVE updates**: every user turn is an ingest opportunity. New facts,
  corrections, preferences, decisions land in the store without the
  agent having to remember to call `memd remember`.
- **LIVE surfacing**: every user turn is a recall opportunity. Relevant
  durable nodes are injected into the pre-model context BEFORE the
  model sees the message, keyed on the actual message content.
- **No manual lookup dance**: the agent should never have to
  `memd lookup` on a speculative term. If the store has a match above
  the confidence floor, it's already in context.

Benchmarks that matter: LongMemEval intrinsic ≥0.92 (mempalace ships
0.966 on its harness — we must match or beat), plus a LIVE-recall
harness that scores on "did memd surface node X without being asked".

## Fix direction

1. **Message-keyed recall in the UserPromptSubmit hook.**
   `.memd/hooks/memd-context.sh` must consume the user message text on
   stdin, extract salient terms (noun phrases, code identifiers,
   project names), fanout `memd lookup` per term, and inject top-N
   matches above a confidence floor into the pre-model context. This
   is the single biggest gap.
2. **Bidirectional hook: capture on the way in, surface on the way in.**
   Same hook can detect correction patterns ("you're wrong", "actually
   X", "no, it's Y") and auto-ingest via `memd remember --kind
   correction` without waiting for the agent. mempalace does not do
   this; supermemory does it shallowly.
3. **Wake pins for semantic+canonical nodes ≥0.85 confidence.** Add a
   dedicated `## Topology` / `## Inspiration` block in `.memd/wake.md`
   generation so topology / architecture / inspiration nodes surface
   every session, not just on explicit query.
4. **Cross-reference follow-through.** When a decision or preference
   cites a node ID (e.g. "see b66692cc"), follow the reference and
   surface the cited node's summary inline so the reference resolves.
5. **Session-entry query fanout.** On wake, proactively run a small
   fanout against project-level terms in `ROADMAP.md` current_phase +
   focus and inline the top-3 results regardless of kind.
6. **Regression harness.** Fixture sessions where the agent must
   recall a known node (mempalace, atlas, dedup, etc.) WITHOUT being
   asked. Agent hallucinating or claiming ignorance fails the harness.
   Run against every release candidate.
7. **LIVE-recall benchmark.** A dedicated metric: given a scripted
   conversation with embedded references to durable nodes, count how
   often memd surfaces the correct node before the agent is forced to
   lookup. Target: 100% for nodes ≥0.85 confidence.

## Vision: memd is the working-memory cache

The deeper frame: **working memory/context should update live as the
conversation moves on**. memd is not a retrieval-store-on-the-side; it
IS the agent's working memory. The conversation transcript is a thin
replayable log; the durable state lives in memd.

Consequences if done right:

- **No compaction needed.** KV cache stays small because the conversation
  stays small. Durable state is not re-stated in the transcript — it
  lives in memd and is re-surfaced on demand. The model sees only what
  changed recently + what memd injected for this turn.
- **KV cache stays warm.** Small context = cache hits across turns.
  Large compaction rewrites invalidate cache; we avoid them entirely.
- **New session = perfect continuity.** Start a fresh conversation and
  memd replays the live state into it. No "summary of previous
  conversation" wall of text, no agent rehydrating from a compaction
  summary. memd is the source of truth; the conversation is a cache of
  the latest deltas.
- **Search is the exception.** Live injection handles the moving edge.
  `memd lookup` is reserved for genuinely old context that aged out of
  the live loop — not for reconstructing every turn.

This shape is strictly better than mempalace and supermemory: both
treat recall as query-driven, coarse, and detached from turn-by-turn
flow. Ours must be conversation-driven, fine-grained, and inline. The
agent should not be reasoning about WHEN to recall; memd should already
have surfaced the right node before the model runs.

## Why this is a wiring bug, not a design gap

memd already has the substrate: active memory lanes, confidence
scores, kind taxonomy, namespace scoping, lookup index. The missing
piece is the live hook that USES the lanes during a conversation.
When the hook detects "mempalace" in the user message, memd should
not only surface the existing nodes — it should also UPDATE:

- bump recency / access count on node 09224ec9
- promote it toward the active lane if it wasn't already
- record that this session referenced it (episodic trace)
- detect if the user's follow-up contradicts or extends the node, and
  queue a correction candidate without the agent calling `remember`

Net effect: the agent doesn't burn 5 lookup queries per turn to
reconstruct context. memd detects and updates as the conversation
flows. That is the LIVE promise. Right now that loop is empty — the
hook runs `memd wake` and stops. All the lane machinery is idle during
the actual conversation.

## Evidence / links

- User correction 2026-04-20 (this backlog): agent did not know
  mempalace on wake, claimed "not a memd term", had to lookup after
  pushback.
- Preference saved this session: "Before claiming a term is unknown,
  run memd lookup" — this is a workaround, not a fix. The core bug is
  that high-confidence inspiration nodes should not need manual lookup.

## Related

- `docs/backlog/v3/2026-04-14-no-behavior-changing-recall-proof.md` —
  wider theme: memd stores memory but doesn't prove behavior change.
  This bug is a concrete instance: recall exists in the store but does
  not change the agent's response because wake doesn't surface it.
- `docs/backlog/v3/2026-04-17-memd-process-too-soft-cross-harness.md` —
  related pattern: memd's protocol is soft; agents route around it.
