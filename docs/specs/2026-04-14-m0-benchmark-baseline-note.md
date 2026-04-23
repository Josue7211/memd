# M0 Benchmark Baseline Note

Date: `2026-04-14`

## What Ran

### Retrieval baseline

```bash
cargo run -p memd-client --bin memd -- benchmark public longmemeval --limit 25 --json
```

Observed result:

- dataset: `LongMemEval`
- mode: `raw`
- retrieval backend: `lexical`
- sample: `25`
- accuracy: `0.96`
- mean latency: `17.2ms`

Important limit:

- this is retrieval-style benchmark output
- this is not the end-to-end `--full-eval` number used for public parity

### Full-eval dry run

```bash
cargo run -p memd-client --bin memd -- benchmark public --all --full-eval --dry-run --sample 5
```

Observed result:

- command path works
- datasets resolved for `longmemeval`, `locomo`, `convomem`, `membench`
- dry-run cost estimate printed successfully

## Blocker

No `OPENAI_API_KEY` or `ANTHROPIC_API_KEY` was present in the shell during this run.

Because of that:

- real `--full-eval` baseline did not run
- only dry-run validation was possible for M0 parity mode

## Next Command

When API credentials are available, run:

```bash
cargo run -p memd-client --bin memd -- benchmark public --all --full-eval
```

Optional cheaper first pass:

```bash
cargo run -p memd-client --bin memd -- benchmark public --all --full-eval --sample 50
```
