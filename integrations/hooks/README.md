# memd Hook Kit

These scripts are the default agent loop integration for `memd`.

Use them when a client wants:

- compact context before work starts
- durable spill at a compaction boundary
- a single stable path into the memory manager

For per-project bootstrap, use:

```bash
memd init --project <project> --agent <agent>
```

Check bundle health with:

```bash
memd status --output .memd
```

Resume the default memory snapshot from the bundle:

```bash
memd resume --output .memd
```

## Environment

Set:

- `MEMD_BASE_URL` - defaults to `http://127.0.0.1:8787`
- `MEMD_PROJECT` - required for context fetches
- `MEMD_AGENT` - required for context fetches
- `MEMD_ROUTE` - defaults to `auto`
- `MEMD_INTENT` - defaults to `general`
- `MEMD_WORKSPACE` - optional shared workspace lane
- `MEMD_VISIBILITY` - optional `private|workspace|public`
- `MEMD_LIMIT` - defaults to `8`
- `MEMD_MAX_CHARS` - defaults to `280`
- `MEMD_RAG_URL` - optional; bundle backend config can supply this when present

## Context Hook

```bash
./memd-context.sh
```

## Install on Unix

```bash
./install.sh
```

Optional:

- `MEMD_BIN=/path/to/memd ./install.sh`

## Spill Hook

```bash
./memd-spill.sh --stdin --apply < compaction.json
```

## Install on Windows

```powershell
./install.ps1
```

Optional:

- `$env:MEMD_BIN = "C:\\path\\to\\memd.exe"`
