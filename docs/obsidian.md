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

## Tier Position

Obsidian is the first deployment tier of the memory stack:

- Tier 1: Obsidian only
- Tier 2: shared sync on top of the same vault structure
- Tier 3: LightRAG layered on top when scale demands semantic retrieval

The later tiers should extend the same file structure instead of replacing it.

For the working loop, the usual pattern is:

- capture or sync source material into the vault
- use `memd search`, `memd working`, and `memd explain` to inspect state
- emit compact shared handoff bundles with `memd handoff` when another agent or human needs to pick work up
- compile durable evidence pages with `obsidian compile`
- keep writeback pages and compiled evidence pages indexed inside `.memd/`
- add LightRAG only if markdown-native retrieval is no longer enough

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
- compiled query pages and compiled memory pages should be treated as the preferred repeat-entry surface before cold raw rereads
- if a topic is already represented in `.memd/compiled/`, prefer that artifact for new agent sessions and only fall back to raw sources when proof-level drilldown is needed
- when `obsidian compile --apply` writes a compiled query page or compiled memory page, memd also syncs the same fetched items into the configured LightRAG backend when one is available
- repeated scans should reuse `.memd/obsidian-scan-cache.json` when file size and modified time match, so unchanged vault material is not reopened unnecessarily

## Scan

Show what is in the vault without importing anything:

```bash
cargo run -p memd-client --bin memd -- obsidian scan --vault ~/vault --summary
```

The scanner keeps a local fingerprint cache in `.memd/obsidian-scan-cache.json` and reuses parsed note and attachment snapshots when a file has not changed.

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

Target a shared workspace lane instead of the private default:

```bash
cargo run -p memd-client --bin memd -- obsidian compile --vault ~/vault --project notes --workspace team-alpha --visibility workspace --query "rust memory patterns" --apply
```

Compile it and open the generated page immediately:

```bash
cargo run -p memd-client --bin memd -- obsidian compile --vault ~/vault --project notes --query "rust memory patterns" --apply --open
```

Compile a specific memory item into a compiled evidence page inside the vault:

```bash
cargo run -p memd-client --bin memd -- obsidian compile --vault ~/vault --id 12345678-1234-5678-1234-567812345678 --apply
```

This is the preferred shape when you want an item to become part of the
Obsidian knowledge base instead of only a transient writeback note.

Preview a shared workspace handoff bundle in the terminal:

```bash
cargo run -p memd-client --bin memd -- handoff --output .memd --prompt
```

Write a shared workspace handoff bundle into the vault:

```bash
cargo run -p memd-client --bin memd -- obsidian handoff --vault ~/vault --project notes --workspace team-alpha --visibility workspace --apply
```

Write it and open it immediately in Obsidian:

```bash
cargo run -p memd-client --bin memd -- obsidian handoff --vault ~/vault --project notes --workspace team-alpha --visibility workspace --apply --open
```

The Obsidian commands that read or compile memory can also take
`--workspace <name>` and `--visibility <private|workspace|public>` so compiled
pages line up with the same safe shared-memory boundaries as the rest of the
control plane.

Generated writeback and compiled memory pages now include workspace and
visibility metadata in their frontmatter and summary sections so handoff state
stays visible inside the vault, not only in API responses.

Handoff pages land under `<vault>/.memd/handoffs/` by default and package:

- the active resume frame
- working memory
- rehydration queue
- inbox pressure
- workspace lanes
- semantic recall when the bundle has RAG configured
- source lanes

Mirror notes and roundtrip annotations produced by `obsidian sync` /
`obsidian roundtrip` also carry workspace and visibility when those flags are
set, so shared-lane provenance survives inside mirrored vault artifacts too.

Each applied compile also updates:

```text
<vault>/.memd/compiled/INDEX.md
```

so generated wiki pages accumulate into a browsable vault index instead of
staying as isolated one-off files. Query pages land under `.memd/compiled/`
and compiled memory evidence pages land under `.memd/compiled/memory/`.
When the active bundle has RAG enabled, compiled query pages and compiled memory
pages also include a bounded `Semantic Recall` section so the vault view
matches the hybrid markdown-plus-semantic retrieval lane used by `memd resume`,
`memd handoff`, and `obsidian compile`.

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

The intended file shape is:

- `raw/`
- `wiki/`
- `output/`

with `memd` preserving typed memory, provenance, lanes, and handoffs around
that markdown workspace.

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

This is the tier progression:

- Obsidian-only for direct file-native knowledge
- shared sync for cross-agent and cross-device continuity
- LightRAG when semantic depth and scale are needed

The intended shape is:

- raw sources
- compiled markdown wiki in Obsidian
- `memd` typed memory and policy layer
- optional LightRAG-compatible backend when scale demands it
