# Lane Architecture Gaps

Status: `open`
Created: 2026-04-13
Phase: Phase H / cross-phase
Related: [[2026-04-13-memd-lane-theory-lock-v1]]

Lane theory lock written. Implementation gaps remain.

## Gaps

1. **Only inspiration lane has source files** — Design, Architecture, Research,
   Workflow, Preference lanes have no `.memd/lanes/<name>/` content yet.

2. **`memd inspiration` is file-scan only** — Theory says lane queries should
   hit the server via atlas. File scan is bootstrap fallback only.

3. **No lane activation logic** — Theory says lanes activate automatically from
   task context (topic, scope, working memory signals). Not implemented.

4. **No lane tagging on ingest** — Memory items aren't tagged with lane membership
   at ingest time. Schema has the field, runtime doesn't populate it.

5. **Atlas regions have `lane` field but unused** — `atlas_regions` table has a
   `lane` column. `generate_regions_for_project()` doesn't set it.
