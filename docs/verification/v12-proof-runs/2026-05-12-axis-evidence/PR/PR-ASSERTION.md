# PR Assertion

- scenario: `routine_library_compose_inherit_export_import`
- pass: `true`
- score: `8/10`
- evidence: memd_core::routine::library; seed-library.jsonl
- metric: `{"composed_routine": "lint-format", "cross_workspace_imported": true, "tampered_export_rejected": true}`
- generated_at: `2026-05-12T18:44:10.634498+00:00`
