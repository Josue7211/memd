# SC Assertion

- scenario: `project_a_b_a_isolation_and_compaction_recovery`
- pass: `true`
- score: `8/10`
- evidence: memd_core::isolation; memd_core::compaction::recovery; docs/contracts/project-isolation.md
- metric: `{"corrections_recovered": 1, "project_a_focus_restored": true, "project_b_items_hidden": true}`
- generated_at: `2026-05-12T03:18:42.859471+00:00`
