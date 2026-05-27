---
contract: selective-expansion-ceo
phase: 25-star
status: normative
opened: 2026-05-27
applies_to:
  - memd lookup --depth lookup
  - agent recall policy
  - 25/5-star CEO-mode synthesis
---

# Selective Expansion CEO Mode — Contract

Selective expansion CEO mode is a recall-policy overlay for high-leverage user
asks such as `CEO mode`, `25/5 star`, `think bigger`, or strategic quality
questions. It preserves the E4 progressive-depth contract: lookup remains cheap
and bounded, and the agent receives explicit guidance for when and how to expand.

## Goal

When the user asks for executive-quality synthesis, `memd lookup --depth lookup`
should not dump a broad resume or raw transcript. It should return the normal
needle-level records plus a compact instruction block that tells the agent to
synthesize from durable evidence and escalate only when needed.

The mode is designed for questions like:

- `CEO mode: make this 25/5 star`
- `how can we make this better`
- `what are we missing`
- `what is the bottleneck`
- `think bigger about the launch plan`

## Non-goals

- It is not a RAG-sidecar path and must not depend on RAG files.
- It is not silent auto-resume. Resume/thread reconstruction remains opt-in.
- It is not permission to include raw chat noise, stale scratch notes, or
  untrusted artifacts.
- It is not a replacement for the `wake|lookup|resume` depth set.

## Trigger taxonomy

| Mode | Label | Meaning | Examples |
| ---- | ----- | ------- | -------- |
| Normal | `normal` | Ordinary lookup; no CEO overlay. | `configuration files`, `json schema` |
| Explicit CEO | `ceo_explicit` | User directly asks for CEO / star / bigger mode. | `CEO mode`, `25/5 star`, `think bigger` |
| Inferred CEO | `ceo_inferred` | User asks a strategy or quality question that needs synthesis. | `what are we missing`, `what is the bottleneck` |

Explicit triggers win over inferred triggers when both match.

## Output contract

At `lookup` depth, CEO mode appends a compact markdown guidance block headed:

```markdown
## Selective expansion: CEO mode
```

The block must include:

1. the trigger label (`ceo_explicit` or `ceo_inferred`),
2. the expansion ladder: `needle -> thread -> CEO -> forensics only if needed`,
3. the answer shape: `Read, Prize, Bottleneck, Moves, Recommendation, Proof`,
4. the memory rule: use approved decisions/preferences/outcomes; avoid raw chat noise.

If lookup records are insufficient, the dispatcher may print a hint telling the
agent to rerun with `--depth resume`. That hint is advisory and must not silently
re-dispatch.

## Agent behavior

An agent receiving CEO-mode guidance should answer in this order:

1. **Read** — state the current situation from the returned evidence.
2. **Prize** — define the upside / desired 25/5-star outcome.
3. **Bottleneck** — identify the highest-leverage constraint.
4. **Moves** — list concrete next moves, ordered by leverage.
5. **Recommendation** — make one clear call.
6. **Proof** — cite the memory evidence used, or state that evidence was thin.

The agent should stay compact. It should escalate to `resume` only when the
1–3 lookup records cannot reconstruct the relevant thread.

## Evaluation fixtures

Fixture rows live under `crates/memd-client/fixtures/e4/`:

- `selective-expansion-ceo-positive.jsonl` — explicit and inferred CEO-mode
  queries with expected labels.
- `selective-expansion-ceo-negative.jsonl` — neutral queries that must remain
  `normal` and must not produce CEO guidance or hints.

Each row is NDJSON with this stable shape:

```json
{"query":"CEO mode: make this 25/5 star","expected_mode":"ceo_explicit","expected_guidance":true,"expected_hint":true}
```

The fixtures are intentionally query-level and do not require RAG state.

## Pass gate

CEO selective expansion passes when:

1. every positive fixture maps to the expected CEO label,
2. every negative fixture maps to `normal`,
3. positive rows produce the CEO guidance block and advisory hint,
4. negative rows produce neither CEO guidance nor CEO hint,
5. existing recall-depth escalation behavior still passes.

## Stability

The labels `normal`, `ceo_explicit`, and `ceo_inferred` are stable for 25-star
evals. New trigger phrases may be added, but existing negative fixtures must not
begin matching unless the fixture is intentionally reclassified with a contract
update.
