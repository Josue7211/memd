# Working-Memory Lifecycle Probe (A3-D3)

The lifecycle probe is a self-test that exercises the full memory pipeline:

1. **store** — write a probe record (scope=local, namespace=`lifecycle_probe`,
   tagged `lifecycle_probe` + `probe:<uuid>`, ttl=300s).
2. **recall** — fetch it back via `/memory/search` and confirm id match.
3. **expire** — mark the record `expired` via `/memory/expire`.
4. **verify_expired** — the same search with `statuses=[active]` must *not*
   return the record; with `statuses=[expired]` it must.

If every step succeeds the probe returns `{"status": "green"}` and the CLI
exits 0. Any failure returns `{"status": "red"}` and exits 1.

## Running manually

```bash
# JSON report
memd diagnostics lifecycle-probe

# Human summary
memd diagnostics lifecycle-probe --summary
```

## Running from a hook

```bash
.memd/hooks/memd-lifecycle-probe.sh
```

The hook honors `MEMD_BUNDLE_ROOT` (default `.memd`) and `MEMD_BASE_URL`
(default `http://127.0.0.1:8787`).

## Cron wiring

Part 1 installs the probe as a manually-runnable hook. Users may wire it from
cron to catch server regressions between sessions:

```cron
*/15 * * * * MEMD_BUNDLE_ROOT=$HOME/Documents/projects/memd/.memd \
    /path/to/.memd/hooks/memd-lifecycle-probe.sh >> /tmp/memd-probe.log 2>&1
```

Automatic wiring (SessionStart + checkpoint invocation) is **A3 Part 2**.

## What a red probe means

- `store` red → server down or `/memory/store` broken.
- `recall` red → write succeeded but search index missed it (indexer lag,
  scope mismatch, status filter too strict).
- `expire` red → `/memory/expire` not persisting status.
- `verify_expired` red → expired records still leak into active searches, or
  the expired-status filter drops them.

Each step's `detail` field in the JSON report carries the upstream error string.
