# TP Assertion

- scenario: `ed25519_signed_audit_and_tamper_evidence`
- pass: `true`
- score: `8/10`
- evidence: memd_core::audit; canonical-audit-log.ndjson; tampered-export.ndjson
- metric: `{"signed_entries": 4, "tamper_detected": true, "verify_export": true}`
- generated_at: `2026-05-12T18:44:10.634498+00:00`
