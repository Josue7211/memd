---
phase: H12
status: closed
closed: 2026-05-05
axis: trust_provenance
evidence: [crates/memd-core/src/audit/mod.rs, scripts/verify/v12-interop-sota-suite.sh]
---

# H12 Signed Audit Entries

Closed by `memd_core::audit::SignedAuditEntry`.

Exit criteria:

- Entries include actor, action, item id, context, payload hash, public key,
  and signature.
- Signatures use ed25519.
- Verification fails after action tamper.
- G12 emits non-zero signed read/correction/deprecation entries.
