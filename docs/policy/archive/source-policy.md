> Secondary/reference doc. For current project truth start with [[ROADMAP]] and [[docs/WHERE-AM-I.md|WHERE-AM-I]].

# Source Policy

`memd` should only ingest source material that is useful, stable, and real.

## Canonical Sources

Good inputs:

- architecture docs
- runbooks
- inventories
- audit reports
- configuration exports
- diagrams or PDFs that are authoritative source-of-truth artifacts
- screenshots only when they encode a real decision, state, or evidence that is not already captured in text

## Not Canonical

Do not ingest:

- synthetic filler
- placeholder examples
- invented entities used only to force graph growth
- duplicate copies of the same fact across multiple files
- decorative images
- weak summaries of summaries

## Multimodal Rule

Enable image, table, and equation processing only when the source is real and canonical.

Examples:

- a network topology PDF
- a firewall export screenshot
- a VM layout diagram
- a vendor manual that is actually referenced in operations

Do not add images just because the pipeline supports them.

## Promotion Rule

Even canonical sources do not become canonical memory automatically.

They still pass through:

- normalization
- dedupe
- scope classification
- promotion gating

If a source later needs correction, repair should stay bounded and auditable:

- verify the item against the current source
- expire stale or invalid material
- supersede replaced material
- contest unresolved claims
- correct metadata without rewriting unrelated history

## Reversible Compression Rule

Compact summaries are fine. Losing the path back to evidence is not.

Every compressed summary should keep a recoverable trail back to:

- the memory item
- the source-memory aggregate
- the recent lifecycle events that touched it

That keeps the hot path cheap without turning compression into deletion.

## Practical Standard

If a source would not help answer a real future question, it does not belong in `memd`.
