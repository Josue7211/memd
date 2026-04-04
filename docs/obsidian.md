# Obsidian Vault Bridge

`memd` can import an Obsidian vault directly from the local vault directory.
That makes it compatible with a CouchDB-synced vault without requiring any
special Obsidian plugin or CLI workflow.

The bridge is filesystem-first:

- markdown notes become candidate memories
- note paths are preserved as source anchors
- wiki links can be turned into entity links
- attachments can still go through the multimodal path separately
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

Review only the sensitive notes that were skipped:

```bash
cargo run -p memd-client --bin memd -- obsidian scan --vault ~/vault --review-sensitive --summary
```

## What Gets Stored

Each note becomes a compact candidate memory with:

- note title
- vault-relative path
- tags and aliases from frontmatter when present
- a short excerpt
- `source_system=obsidian`
- `source_path` anchored to the note file

If `--link-notes` is enabled, wiki links like `[[Other Note]]` are resolved
against imported note titles and written as entity links.

Notes that contain obvious credential markers such as API keys, private keys,
or secret tokens are excluded from import by default.

`--review-sensitive` prints only filenames and reasons for skipped sensitive
notes. It does not print note bodies, excerpts, or candidate content.

## Obsidian CLI

This bridge does not depend on Obsidian's CLI. If you already use a CLI to
open or manage the vault, you can keep doing that. `memd` only needs the
local vault path.
