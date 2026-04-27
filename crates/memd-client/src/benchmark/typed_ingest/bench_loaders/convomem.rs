//! ConvoMem episodic adapter — task A6.5.
//!
//! Reads `convomem-evidence-sample-*.json` (top-level dict with `items`,
//! each item carrying `metadata.conversations: [{id, messages: [{speaker,
//! text}]}]`). Yields one `EpisodicTurn` per message with `session_id =
//! <item_id>::<conversation_id>` and `captured_at` taken from the
//! enclosing item's `metadata.scenario_description` is unsuitable as a
//! timestamp; ConvoMem does not ship per-message dates, so we leave
//! `captured_at` empty (per `phase-a6-plan.md` §2 — provenance is
//! best-effort, missing fields stay empty rather than synthesised).

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::super::episodic::{EpisodicAdapter, EpisodicProvenance, EpisodicTurn};

pub(crate) const BENCH_ID: &str = "convomem";

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ConvomemFile {
    #[serde(default)]
    pub items: Vec<ConvomemItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ConvomemItem {
    #[serde(default)]
    pub item_id: String,
    #[serde(default)]
    pub metadata: ConvomemMetadata,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub(crate) struct ConvomemMetadata {
    #[serde(default)]
    pub conversations: Vec<ConvomemConversation>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ConvomemConversation {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub messages: Vec<ConvomemMessage>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub(crate) struct ConvomemMessage {
    pub speaker: String,
    pub text: String,
}

struct FlatTurn {
    session_id: String,
    turn_index: u32,
    msg: ConvomemMessage,
}

pub(crate) struct ConvomemAdapter {
    queue: std::vec::IntoIter<FlatTurn>,
}

impl ConvomemAdapter {
    pub(crate) fn from_path(path: &Path) -> Result<Self> {
        let bytes = fs::read(path)
            .with_context(|| format!("read ConvoMem dataset {}", path.display()))?;
        let file: ConvomemFile = serde_json::from_slice(&bytes)
            .with_context(|| format!("parse ConvoMem dataset {}", path.display()))?;
        Ok(Self::from_file(file))
    }

    pub(crate) fn from_file(file: ConvomemFile) -> Self {
        let mut flat: Vec<FlatTurn> = Vec::new();
        for item in file.items {
            for conv in item.metadata.conversations {
                let session_id = format!("{}::{}", item.item_id, conv.id);
                for (idx, msg) in conv.messages.into_iter().enumerate() {
                    flat.push(FlatTurn {
                        session_id: session_id.clone(),
                        turn_index: idx as u32,
                        msg,
                    });
                }
            }
        }
        Self { queue: flat.into_iter() }
    }
}

impl EpisodicAdapter for ConvomemAdapter {
    fn bench_id(&self) -> &'static str {
        BENCH_ID
    }

    fn next_turn(&mut self) -> Option<EpisodicTurn> {
        let f = self.queue.next()?;
        let raw = serde_json::to_vec(&f.msg).unwrap_or_default();
        let source_hash = format!("{:x}", Sha256::digest(&raw));
        Some(EpisodicTurn {
            content: f.msg.text.clone(),
            provenance: EpisodicProvenance {
                bench_id: BENCH_ID.to_string(),
                session_id: f.session_id,
                turn_index: f.turn_index,
                speaker: f.msg.speaker.clone(),
                source_hash,
                captured_at: String::new(),
            },
        })
    }
}
