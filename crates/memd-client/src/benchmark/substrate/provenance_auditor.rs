//! E5 provenance field-completeness auditor.
//!
//! Validates that every retrieved record carries all required provenance
//! metadata: source_turn, captured_by, captured_at. This module is designed
//! for reuse by B5 (provenance-chain assertions) and G5 (aggregator spot-check).

use serde::{Deserialize, Serialize};

/// Required provenance fields per phase-e5-plan.md §2.
const REQUIRED_FIELDS: &[&str] = &["source_turn", "captured_by", "captured_at"];

/// Audit outcome for a single record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct AuditOutcome {
    pub(crate) passed: bool,
    pub(crate) missing_fields: Vec<String>,
    pub(crate) chain_length: usize,
}

impl AuditOutcome {
    pub(crate) fn passed() -> Self {
        Self {
            passed: true,
            missing_fields: Vec::new(),
            chain_length: 0,
        }
    }

    pub(crate) fn with_chain_length(mut self, length: usize) -> Self {
        self.chain_length = length;
        self
    }

    pub(crate) fn missing(fields: Vec<String>) -> Self {
        Self {
            passed: false,
            missing_fields: fields,
            chain_length: 0,
        }
    }
}

/// Audit a single MemoryRecord-like JSON object for provenance completeness.
/// Returns `AuditOutcome` indicating which fields (if any) are missing.
///
/// Input: `record` is a serde_json::Value representing a MemoryRecord.
/// We check for all REQUIRED_FIELDS in the provenance block.
pub(crate) fn audit_record(record: &serde_json::Value) -> AuditOutcome {
    let missing: Vec<String> = REQUIRED_FIELDS
        .iter()
        .filter(|field| {
            record.get("provenance")
                .and_then(|p| p.get(**field))
                .is_none()
        })
        .map(|s| s.to_string())
        .collect();

    if missing.is_empty() {
        // Try to infer chain length from provenance structure if present.
        let chain_len = record
            .get("provenance")
            .and_then(|p| p.get("chain"))
            .and_then(|c| {
                if c.is_array() {
                    c.as_array().map(|arr| arr.len())
                } else {
                    None
                }
            })
            .unwrap_or(1);

        AuditOutcome::passed().with_chain_length(chain_len)
    } else {
        AuditOutcome::missing(missing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Test 1: auditor passes a fully-sourced record.
    #[test]
    fn auditor_passes_fully_sourced_record() {
        let record = json!({
            "id": "test-1",
            "content": "test fact",
            "provenance": {
                "source_turn": "s1:t1",
                "captured_by": "manual",
                "captured_at": "2026-01-01T00:00:00Z"
            }
        });
        let outcome = audit_record(&record);
        assert!(outcome.passed);
        assert!(outcome.missing_fields.is_empty());
    }

    /// Test 2: auditor fails when source_turn is missing.
    #[test]
    fn auditor_fails_missing_source_turn() {
        let record = json!({
            "id": "test-2",
            "content": "test fact",
            "provenance": {
                "captured_by": "manual",
                "captured_at": "2026-01-01T00:00:00Z"
            }
        });
        let outcome = audit_record(&record);
        assert!(!outcome.passed);
        assert!(outcome.missing_fields.contains(&"source_turn".to_string()));
    }

    /// Test 3: auditor fails when captured_by is missing.
    #[test]
    fn auditor_fails_missing_captured_by() {
        let record = json!({
            "id": "test-3",
            "content": "test fact",
            "provenance": {
                "source_turn": "s1:t1",
                "captured_at": "2026-01-01T00:00:00Z"
            }
        });
        let outcome = audit_record(&record);
        assert!(!outcome.passed);
        assert!(outcome.missing_fields.contains(&"captured_by".to_string()));
    }

    /// Test 4: auditor reports chain length.
    #[test]
    fn auditor_reports_chain_length() {
        let record = json!({
            "id": "test-4",
            "content": "test fact",
            "provenance": {
                "source_turn": "s1:t1",
                "captured_by": "detector",
                "captured_at": "2026-01-01T00:00:00Z",
                "chain": [
                    {"turn": "s1:t1", "operation": "ingest"},
                    {"turn": "s1:t2", "operation": "correct"}
                ]
            }
        });
        let outcome = audit_record(&record);
        assert!(outcome.passed);
        assert_eq!(outcome.chain_length, 2);
    }
}
