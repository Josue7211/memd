//! LoCoMo episodic adapter — task A6.3.
//!
//! Reads `locomo10.json` (list of items with a `conversation` map keyed by
//! `session_N` lists of `{speaker, dia_id, text}` turns plus parallel
//! `session_N_date_time` strings). Yields one `EpisodicTurn` per turn in
//! deterministic numeric session order.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::super::episodic::{EpisodicAdapter, EpisodicProvenance, EpisodicTurn};

pub(crate) const BENCH_ID: &str = "locomo";

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct LocomoItem {
    #[serde(default)]
    pub sample_id: String,
    pub conversation: Value,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub(crate) struct LocomoTurn {
    pub speaker: String,
    #[serde(default)]
    pub dia_id: String,
    pub text: String,
}

struct FlatTurn {
    session_id: String,
    captured_at: String,
    turn_index: u32,
    turn: LocomoTurn,
}

pub(crate) struct LocomoAdapter {
    queue: std::vec::IntoIter<FlatTurn>,
}

impl LocomoAdapter {
    pub(crate) fn from_path(path: &Path) -> Result<Self> {
        let bytes = fs::read(path)
            .with_context(|| format!("read LoCoMo dataset {}", path.display()))?;
        let items: Vec<LocomoItem> = serde_json::from_slice(&bytes)
            .with_context(|| format!("parse LoCoMo dataset {}", path.display()))?;
        Ok(Self::from_items(items))
    }

    pub(crate) fn from_items(items: Vec<LocomoItem>) -> Self {
        let mut flat: Vec<FlatTurn> = Vec::new();
        for item in items.into_iter() {
            let Some(conv) = item.conversation.as_object() else { continue };
            let mut sessions: BTreeMap<u32, &str> = BTreeMap::new();
            for k in conv.keys() {
                if let Some(rest) = k.strip_prefix("session_") {
                    if rest.ends_with("_date_time") {
                        continue;
                    }
                    if let Ok(n) = rest.parse::<u32>() {
                        sessions.insert(n, k.as_str());
                    }
                }
            }
            for (n, key) in sessions {
                let session_id = format!("{}::session_{}", item.sample_id, n);
                let captured_at = conv
                    .get(&format!("session_{}_date_time", n))
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                let Some(turns) = conv.get(key).and_then(Value::as_array) else { continue };
                for (turn_idx, raw) in turns.iter().enumerate() {
                    let Ok(turn) = serde_json::from_value::<LocomoTurn>(raw.clone()) else {
                        continue;
                    };
                    flat.push(FlatTurn {
                        session_id: session_id.clone(),
                        captured_at: captured_at.clone(),
                        turn_index: turn_idx as u32,
                        turn,
                    });
                }
            }
        }
        Self { queue: flat.into_iter() }
    }
}

impl EpisodicAdapter for LocomoAdapter {
    fn bench_id(&self) -> &'static str {
        BENCH_ID
    }

    fn next_turn(&mut self) -> Option<EpisodicTurn> {
        let f = self.queue.next()?;
        let raw = serde_json::to_vec(&f.turn).unwrap_or_default();
        let source_hash = format!("{:x}", Sha256::digest(&raw));
        Some(EpisodicTurn {
            content: f.turn.text.clone(),
            provenance: EpisodicProvenance {
                bench_id: BENCH_ID.to_string(),
                session_id: f.session_id,
                turn_index: f.turn_index,
                speaker: f.turn.speaker.clone(),
                source_hash,
                captured_at: f.captured_at,
            },
        })
    }
}
