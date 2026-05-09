use std::path::Path;

use anyhow::anyhow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConfiguredEmbeddingModel {
    AllMiniLML6V2,
    BGEBaseENV15,
    BGELargeENV15,
}

pub(crate) struct Embedder {
    configured_model: ConfiguredEmbeddingModel,
}

impl Embedder {
    pub(crate) fn try_new(_cache_dir: &Path) -> anyhow::Result<Self> {
        Err(anyhow!(
            "intrinsic dense embedding is unavailable in this server build; use MEMD_RAG_URL sidecar retrieval"
        ))
    }

    pub(crate) fn dim(&self) -> usize {
        self.configured_model.dim()
    }

    pub(crate) fn model_code(&self) -> &'static str {
        self.configured_model.code()
    }

    pub(crate) fn embed_query_normalized(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Ok(vec![0.0; self.dim()]);
        }
        Err(anyhow!(
            "intrinsic dense embedding is unavailable in this server build"
        ))
    }

    /// Embed a batch of texts in a single ort session call. Empty inputs
    /// are skipped entirely (not padded) — callers get a 1:1 mapping
    /// between the returned Vec and the non-empty inputs only. That
    /// keeps the ingest hot path off a per-chunk mutex dance and cuts
    /// wall-clock per document by roughly Nx for N chunks.
    pub(crate) fn embed_batch_normalized(&self, texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>> {
        let prepared: Vec<String> = texts
            .iter()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .map(|t| format!("passage: {t}"))
            .collect();
        if prepared.is_empty() {
            return Ok(Vec::new());
        }
        Err(anyhow!(
            "intrinsic dense embedding is unavailable in this server build"
        ))
    }
}

impl ConfiguredEmbeddingModel {
    pub(crate) fn code(self) -> &'static str {
        match self {
            Self::AllMiniLML6V2 => "all-minilm-l6-v2",
            Self::BGEBaseENV15 => "bge-base-en-v1.5",
            Self::BGELargeENV15 => "bge-large-en-v1.5",
        }
    }

    fn dim(self) -> usize {
        match self {
            Self::AllMiniLML6V2 => 384,
            Self::BGEBaseENV15 => 768,
            Self::BGELargeENV15 => 1024,
        }
    }
}

pub(crate) fn configured_embedding_model_from_env() -> ConfiguredEmbeddingModel {
    match std::env::var("MEMD_EMBED_MODEL")
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .as_deref()
    {
        Some("bge-base-en-v1.5") => ConfiguredEmbeddingModel::BGEBaseENV15,
        Some("bge-large-en-v1.5") => ConfiguredEmbeddingModel::BGELargeENV15,
        _ => ConfiguredEmbeddingModel::AllMiniLML6V2,
    }
}

fn l2_normalize(vec: &mut [f32]) {
    let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 1e-12 {
        for v in vec.iter_mut() {
            *v /= norm;
        }
    }
}

pub(crate) fn vec_to_bytes(vec: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(vec.len() * 4);
    for v in vec {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

pub(crate) fn bytes_to_vec(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

pub(crate) fn cosine_on_unit(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

pub(crate) fn intrinsic_dense_enabled() -> bool {
    false
}

/// Split text into overlapping character windows. Tuned for MiniLM's
/// 256-token cap: ~1500 chars stays under the cap for English prose,
/// and the 200-char overlap keeps answer phrases from being split
/// across chunk boundaries (which silently demote their cosine score).
pub(crate) fn chunk_text(text: &str, max_chars: usize, overlap: usize) -> Vec<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    if trimmed.chars().count() <= max_chars {
        return vec![trimmed.to_string()];
    }
    let chars: Vec<char> = trimmed.chars().collect();
    let step = max_chars.saturating_sub(overlap).max(1);
    let mut chunks = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let end = (i + max_chars).min(chars.len());
        chunks.push(chars[i..end].iter().collect::<String>());
        if end == chars.len() {
            break;
        }
        i += step;
    }
    chunks
}

pub(crate) fn chunk_max_chars() -> usize {
    std::env::var("MEMD_DENSE_CHUNK_CHARS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|n| *n > 0)
        .unwrap_or(1500)
}

pub(crate) fn chunk_overlap_chars() -> usize {
    std::env::var("MEMD_DENSE_CHUNK_OVERLAP")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(200)
}

pub(crate) fn default_cache_dir() -> std::path::PathBuf {
    if let Ok(dir) = std::env::var("MEMD_FASTEMBED_CACHE") {
        return std::path::PathBuf::from(dir);
    }
    if let Ok(home) = std::env::var("HOME") {
        return std::path::PathBuf::from(home).join(".memd/fastembed");
    }
    std::path::PathBuf::from("/tmp/memd-fastembed")
}
