//! Preference replay + drift detection (Phase F4).
//!
//! Preferences live in the canonical memory store. F4 adds a drift
//! detector that compares recent agent behavior to a stored preference
//! via the cached LLM judge (shared C4 infrastructure), surfaces drift
//! through the D4 wake compiler, and exposes a `memd preference` CLI.
//!
//! Module layout:
//! - [`drift`]: detector + outstanding-state helpers.

pub mod drift;
pub mod outstanding;
pub mod tick;

use serde::{Deserialize, Serialize};

/// Lightweight preference record consumed by the drift detector.
///
/// Real preferences live as `MemoryKind::Preference` records; this
/// struct is the minimal surface the detector and CLI need.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreferenceRecord {
    pub id: String,
    pub content: String,
}

impl PreferenceRecord {
    pub fn new(id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
        }
    }
}
