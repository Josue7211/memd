//! LongMemEval episodic adapter — task A6.2.
//!
//! Reads `longmemeval_s_cleaned.json` (list of items with
//! `haystack_session_ids` parallel to `haystack_sessions`, each session a
//! list of `{role, content}` turns). Yields one `EpisodicTurn` per turn
//! across all sessions of all items, in deterministic order.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::super::episodic::{EpisodicAdapter, EpisodicProvenance, EpisodicTurn};

pub(crate) const BENCH_ID: &str = "longmemeval";

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct LmeItem {
    #[serde(default)]
    pub question_date: String,
    #[serde(default)]
    pub haystack_dates: Vec<String>,
    pub haystack_session_ids: Vec<String>,
    pub haystack_sessions: Vec<Vec<LmeTurn>>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub(crate) struct LmeTurn {
    pub role: String,
    pub content: String,
}

pub(crate) struct LmeAdapter {
    items: Vec<LmeItem>,
    item_idx: usize,
    session_idx: usize,
    turn_idx: usize,
}

impl LmeAdapter {
    pub(crate) fn from_path(path: &Path) -> Result<Self> {
        let bytes =
            fs::read(path).with_context(|| format!("read LME dataset {}", path.display()))?;
        let items: Vec<LmeItem> = serde_json::from_slice(&bytes)
            .with_context(|| format!("parse LME dataset {}", path.display()))?;
        Ok(Self::from_items(items))
    }

    pub(crate) fn from_items(items: Vec<LmeItem>) -> Self {
        Self {
            items,
            item_idx: 0,
            session_idx: 0,
            turn_idx: 0,
        }
    }
}

impl EpisodicAdapter for LmeAdapter {
    fn bench_id(&self) -> &'static str {
        BENCH_ID
    }

    fn next_turn(&mut self) -> Option<EpisodicTurn> {
        loop {
            let item = self.items.get(self.item_idx)?;
            let Some(session) = item.haystack_sessions.get(self.session_idx) else {
                self.item_idx += 1;
                self.session_idx = 0;
                self.turn_idx = 0;
                continue;
            };
            let Some(turn) = session.get(self.turn_idx) else {
                self.session_idx += 1;
                self.turn_idx = 0;
                continue;
            };
            let session_id = item
                .haystack_session_ids
                .get(self.session_idx)
                .cloned()
                .unwrap_or_default();
            let captured_at = item
                .haystack_dates
                .get(self.session_idx)
                .cloned()
                .unwrap_or_else(|| item.question_date.clone());
            let raw = serde_json::to_vec(turn).unwrap_or_default();
            let source_hash = format!("{:x}", Sha256::digest(&raw));
            let provenance = EpisodicProvenance {
                bench_id: BENCH_ID.to_string(),
                session_id,
                turn_index: self.turn_idx as u32,
                speaker: turn.role.clone(),
                source_hash,
                captured_at,
            };
            let content = turn.content.clone();
            self.turn_idx += 1;
            return Some(EpisodicTurn {
                content,
                provenance,
            });
        }
    }
}
