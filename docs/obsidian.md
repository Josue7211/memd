# Obsidian Vault Bridge

`memd` can import an Obsidian vault directly from the local vault directory.
That makes it compatible with a CouchDB-synced vault without requiring any
special Obsidian plugin or CLI workflow.

The bridge is filesystem-first:

- markdown notes become candidate memories
- note paths are preserved as source anchors
- unchanged notes are skipped using a local sync state file
- wiki links can be turned into entity links
- backlinks are recorded so the vault has directional note context
- folder paths and depth are captured for graph-aware vault structure
- attachments can be imported from the vault, routed through the multimodal path, and linked back to the note they belong to
- notes that look like secrets are skipped before import

## Scan

Show what is in the vault without importing anything:

```bash
cargo run -p memd-client --bin memd -- obsidian scan --vault ~/vault --summary
```

## Import

Import notes into `memd` as candidates:

```bash
cargo run -p memd-client --bin memd -- obsidian import --vault ~/vault --project notes --apply
```

Import notes and also create associative links for wiki links:

```bash
cargo run -p memd-client --bin memd -- obsidian import --vault ~/vault --project notes --apply --link-notes
```

Sync a vault in one pass:

```bash
cargo run -p memd-client --bin memd -- obsidian sync --vault ~/vault --project notes
```

Write a memory item back into the vault as a note:

```bash
cargo run -p memd-client --bin memd -- obsidian writeback --vault ~/vault --id <uuid> --apply
```

Import notes and vault attachments together:

```bash
cargo run -p memd-client --bin memd -- obsidian import --vault ~/vault --project notes --include-attachments --apply
```

Review only the sensitive notes that were skipped:

```bash
cargo run -p memd-client --bin memd -- obsidian scan --vault ~/vault --review-sensitive --summary
```

## What Gets Stored

Each note becomes a compact candidate memory with:

- note title
- vault-relative path
- folder path and folder depth
- tags and aliases from frontmatter when present
- a short excerpt
- backlinks when the vault contains inbound wiki references
- `source_system=obsidian`
- `source_path` anchored to the note file

If `--link-notes` is enabled, wiki links like `[[Other Note]]` are resolved
against imported note titles and written as entity links.

Notes that contain obvious credential markers such as API keys, private keys,
or secret tokens are excluded from import by default.

`--review-sensitive` prints only filenames and reasons for skipped sensitive
notes. It does not print note bodies, excerpts, or candidate content.

When `--include-attachments` is enabled, `memd` scans non-markdown vault files,
skips unchanged assets using the same sync state, routes changed attachments
through the multimodal sidecar, and stores a compact attachment memory record
so the graph can link the attachment back to its note when the filename or
folder strongly matches. Text-like attachments are also screened for obvious
secret markers before import. Attachments also carry folder metadata so vault
structure stays visible in the graph.

Incremental sync stores a small state file under the vault by default:

```text
<vault>/.memd/obsidian-sync.json
```

That file keeps a per-note hash, size, modified time, and last imported item
ID so unchanged notes can be skipped and changed notes can be marked as
superseding the previous import.

## Obsidian CLI

This bridge does not depend on Obsidian's CLI. If you already use a CLI to
open or manage the vault, you can keep doing that. `memd` only needs the
local vault path.

Writeback notes are generated under `<vault>/.memd/writeback/` by default.
Pass `--output` to place them somewhere else and `--overwrite` to replace an
existing note.
