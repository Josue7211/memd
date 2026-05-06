# TP Assertion

- scenario: `ed25519_signed_audit_and_tamper_evidence`
- pass: `true`
- score: `8/10`
- evidence: memd_core::audit; canonical-audit-log.ndjson; tampered-export.ndjson
- metric: `{"signed_entries": 4, "tamper_detected": true, "verify_export": true}`
- generated_at: `2026-05-06T00:09:52.233394+00:00`
