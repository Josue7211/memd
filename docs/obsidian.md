# Obsidian Vault Bridge

`memd` can import an Obsidian vault directly from the local vault directory.
That makes it compatible with a CouchDB-synced vault without requiring any
special Obsidian plugin or CLI workflow.

It also fits a stronger workflow than simple note import:

- keep raw source material in the vault
- let an agent compile or maintain derived wiki pages as markdown
- use Obsidian as the human and agent workspace for browsing, backlinks, and outputs
- let `memd` preserve typed memory, provenance, contradiction state, and policy outside the vault text itself

That means Obsidian is not only an ingest source. It can also be the markdown
frontend for a compiled knowledge base, while `memd` remains the memory control
plane behind it.

The bridge is filesystem-first:

- markdown notes become candidate memories
- compiled wiki pages can be imported the same way as hand-written notes
- note paths are preserved as source anchors
- unchanged notes are skipped using a local sync state file
- wiki links can be turned into entity links
- backlinks are recorded so the vault has directional note context
- folder paths and depth are captured for graph-aware vault structure
- attachments can be imported from the vault, routed through the multimodal path, and linked back to the note they belong to
- notes that look like secrets are skipped before import
- notes can be mirrored back in place with a small `memd` sync block for round-trip editing

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

Watch a vault and keep it synced automatically:

```bash
cargo run -p memd-client --bin memd -- obsidian watch --vault ~/vault --project notes
```

Inspect vault sync health:

```bash
cargo run -p memd-client --bin memd -- obsidian status --vault ~/vault --project notes --summary
```

Limit sync to specific folders or tags:

```bash
cargo run -p memd-client --bin memd -- obsidian roundtrip --vault ~/vault --project notes --include-folder work --include-tag project --apply
cargo run -p memd-client --bin memd -- obsidian watch --vault ~/vault --project notes --include-folder work --exclude-folder archive
```

Write a memory item back into the vault as a note:

```bash
cargo run -p memd-client --bin memd -- obsidian writeback --vault ~/vault --id <uuid> --apply
```

Write it back and open it in Obsidian immediately:

```bash
cargo run -p memd-client --bin memd -- obsidian writeback --vault ~/vault --id <uuid> --apply --open
```

Open it in a split pane instead of replacing the current tab:

```bash
cargo run -p memd-client --bin memd -- obsidian writeback --vault ~/vault --id <uuid> --apply --open --pane-type split
```

Open an existing vault note directly through the Obsidian URI path:

```bash
cargo run -p memd-client --bin memd -- obsidian open --vault ~/vault --note wiki/topic.md --apply
```

Preview the URI without launching Obsidian:

```bash
cargo run -p memd-client --bin memd -- obsidian open --vault ~/vault --note wiki/topic.md
```

Compile a search query into a markdown wiki page inside the vault:

```bash
cargo run -p memd-client --bin memd -- obsidian compile --vault ~/vault --project notes --query "rust memory patterns" --apply
```

Compile it and open the generated page immediately:

```bash
cargo run -p memd-client --bin memd -- obsidian compile --vault ~/vault --project notes --query "rust memory patterns" --apply --open
```

Each applied compile also updates:

```text
<vault>/.memd/compiled/INDEX.md
```

so generated wiki pages accumulate into a browsable vault index instead of
staying as isolated one-off files.

Round-trip a vault and annotate source notes in place:

```bash
cargo run -p memd-client --bin memd -- obsidian roundtrip --vault ~/vault --project notes --apply
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

At the product level, this supports a compiled-wiki workflow:

- `raw/` source material can stay in the vault or nearby project tree
- agent-maintained summary and concept pages can live in the same vault
- generated outputs such as reports, slides, and analysis notes can be filed back into the vault
- `memd` can index those files without forcing a separate semantic backend for small and medium knowledge bases

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

For desktop open-hand-off, `memd` can also emit and launch an `obsidian://open`
URI that targets the generated writeback file. `obsidian writeback --open`
uses the platform opener (`open`, `xdg-open`, or `start`) so the result lands
back in the vault UI immediately.

Writeback notes are generated under `<vault>/.memd/writeback/` by default.
Pass `--output` to place them somewhere else and `--overwrite` to replace an
existing note.

Writeback pages are meant to be useful compiled-wiki artifacts, not only raw
exports. They include:

- the canonical memory summary
- reasons and policy hooks
- entity state when available
- recent events
- top source lanes
- sibling belief branches
- the compact artifact trail behind the memory

`obsidian roundtrip` also writes a compact `<!-- memd:begin -->` block back
into each synced source note so the vault keeps a local record of the imported
memory item and entity. It also writes mirror notes under
`<vault>/.memd/writeback/notes/` and attachment mirrors under
`<vault>/.memd/writeback/attachments/`.

`obsidian watch` starts the same round-trip sync in a file watcher loop and
reruns it after vault changes settle.

`--include-folder`, `--exclude-folder`, `--include-tag`, and `--exclude-tag`
scope the vault bridge to a slice of the vault instead of the whole tree.

`obsidian status` reports sync entry count, mirror coverage, changed vs
unchanged items, and whether the round-trip path is already live.

## Relationship To RAG

Obsidian is a first-class markdown workspace option, not a replacement for the
optional semantic backend.

Use Obsidian mode when:

- the knowledge base is still small or medium enough to stay markdown-native
- backlinks, local files, and direct agent-written notes are the main workflow
- you want the agent to compile and maintain a wiki in place

Use the optional semantic backend when:

- the vault or source corpus grows beyond comfortable direct file navigation
- semantic recall across larger corpora becomes necessary
- multimodal retrieval pressure is high enough that a dedicated backend pays for itself

The intended shape is:

- raw sources
- compiled markdown wiki in Obsidian
- `memd` typed memory and policy layer
- optional LightRAG-compatible backend when scale demands it
