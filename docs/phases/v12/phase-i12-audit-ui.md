---
phase: I12
status: closed
closed: 2026-05-05
axis: trust_provenance
evidence: [crates/memd-core/src/audit/mod.rs, scripts/verify/v12-interop-sota-suite.sh]
---

# I12 Audit UI

Closed at core browse/explain layer. CLI polish can reuse `AuditLog::browse_since`
and `AuditLog::explain`.

Exit criteria:

- Audit log can browse entries since a timestamp.
- Audit log can explain an item chain by `item_id`.
- Export remains NDJSON for external review.
- G12 summary cites signed entries count.
