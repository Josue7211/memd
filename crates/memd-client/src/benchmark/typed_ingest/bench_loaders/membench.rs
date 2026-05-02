//! MemBench episodic adapter — task A6.4.
//!
//! Reads `membench-firstagent.json` (top-level dict keyed by category, each
//! value a list of items with `tid` and `message_list: Vec<Vec<turn>>` where
//! a turn carries `mid`, `time`, `place`, and both `user` /
//! `user_message` and `assistant` / `assistant_message` /
//! `agent` fields). Each turn yields up to two `EpisodicTurn`s — one
//! `user`, one `assistant` — preserving FirstAgent ordering.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::super::episodic::{EpisodicAdapter, EpisodicProvenance, EpisodicTurn};

pub(crate) const BENCH_ID: &str = "membench";

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct MembenchItem {
    #[serde(default)]
    pub tid: String,
    #[serde(default)]
    pub message_list: Vec<Vec<Value>>,
}

struct FlatTurn {
    session_id: String,
    captured_at: String,
    turn_index: u32,
    speaker: String,
    content: String,
    raw: Vec<u8>,
}

pub(crate) struct MembenchAdapter {
    queue: std::vec::IntoIter<FlatTurn>,
}

impl MembenchAdapter {
    pub(crate) fn from_path(path: &Path) -> Result<Self> {
        let bytes =
            fs::read(path).with_context(|| format!("read MemBench dataset {}", path.display()))?;
        let raw: BTreeMap<String, Vec<MembenchItem>> = serde_json::from_slice(&bytes)
            .with_context(|| format!("parse MemBench dataset {}", path.display()))?;
        Ok(Self::from_categories(raw))
    }

    pub(crate) fn from_categories(raw: BTreeMap<String, Vec<MembenchItem>>) -> Self {
        let mut flat: Vec<FlatTurn> = Vec::new();
        for (category, items) in raw {
            for item in items {
                for (list_idx, msg_list) in item.message_list.into_iter().enumerate() {
                    let session_id = format!("{}::{}::list_{}", category, item.tid, list_idx);
                    for turn in msg_list {
                        let mid = turn.get("mid").and_then(Value::as_u64).unwrap_or(0) as u32;
                        let captured_at = turn
                            .get("time")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string();
                        let user = turn
                            .get("user_message")
                            .and_then(Value::as_str)
                            .or_else(|| turn.get("user").and_then(Value::as_str))
                            .map(str::to_string);
                        let assistant = turn
                            .get("assistant_message")
                            .and_then(Value::as_str)
                            .or_else(|| turn.get("assistant").and_then(Value::as_str))
                            .or_else(|| turn.get("agent").and_then(Value::as_str))
                            .map(str::to_string);
                        let raw_bytes = serde_json::to_vec(&turn).unwrap_or_default();
                        if let Some(content) = user.filter(|s| !s.is_empty()) {
                            flat.push(FlatTurn {
                                session_id: session_id.clone(),
                                captured_at: captured_at.clone(),
                                turn_index: mid * 2,
                                speaker: "user".to_string(),
                                content,
                                raw: raw_bytes.clone(),
                            });
                        }
                        if let Some(content) = assistant.filter(|s| !s.is_empty()) {
                            flat.push(FlatTurn {
                                session_id: session_id.clone(),
                                captured_at: captured_at.clone(),
                                turn_index: mid * 2 + 1,
                                speaker: "assistant".to_string(),
                                content,
                                raw: raw_bytes,
                            });
                        }
                    }
                }
            }
        }
        Self {
            queue: flat.into_iter(),
        }
    }
}

impl EpisodicAdapter for MembenchAdapter {
    fn bench_id(&self) -> &'static str {
        BENCH_ID
    }

    fn next_turn(&mut self) -> Option<EpisodicTurn> {
        let f = self.queue.next()?;
        let role_bytes = f.speaker.as_bytes();
        let mut hasher = Sha256::new();
        hasher.update(&f.raw);
        hasher.update(b"::");
        hasher.update(role_bytes);
        let source_hash = format!("{:x}", hasher.finalize());
        Some(EpisodicTurn {
            content: f.content,
            provenance: EpisodicProvenance {
                bench_id: BENCH_ID.to_string(),
                session_id: f.session_id,
                turn_index: f.turn_index,
                speaker: f.speaker,
                source_hash,
                captured_at: f.captured_at,
            },
        })
    }
}
