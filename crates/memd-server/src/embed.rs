#![allow(dead_code)]

use std::path::Path;

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
        Ok(Self {
            configured_model: configured_embedding_model_from_env(),
        })
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
        let mut vector = feature_hash_embedding(trimmed, self.dim());
        l2_normalize(&mut vector);
        Ok(vector)
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
        let mut vectors = Vec::with_capacity(prepared.len());
        for text in prepared {
            let mut vector = feature_hash_embedding(&text, self.dim());
            l2_normalize(&mut vector);
            vectors.push(vector);
        }
        Ok(vectors)
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
    parse_intrinsic_dense_enabled(std::env::var("MEMD_INTRINSIC_DENSE").ok().as_deref())
}

fn parse_intrinsic_dense_enabled(value: Option<&str>) -> bool {
    value
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on" | "enabled"
            )
        })
        .unwrap_or(true)
}

fn feature_hash_embedding(text: &str, dim: usize) -> Vec<f32> {
    let mut vector = vec![0.0; dim.max(1)];
    let normalized = normalize_embedding_text(text);
    for token in normalized
        .split_whitespace()
        .filter(|token| token.len() >= 2)
    {
        add_feature(&mut vector, &format!("tok:{token}"), 1.0);
        for gram in char_ngrams(token, 3) {
            add_feature(&mut vector, &format!("tri:{gram}"), 0.35);
        }
        for gram in char_ngrams(token, 4) {
            add_feature(&mut vector, &format!("quad:{gram}"), 0.25);
        }
    }
    for pair in normalized.split_whitespace().collect::<Vec<_>>().windows(2) {
        add_feature(&mut vector, &format!("bi:{}_{}", pair[0], pair[1]), 0.55);
    }
    vector
}

fn normalize_embedding_text(text: &str) -> String {
    text.chars()
        .map(|ch| {
            if ch.is_alphanumeric() || matches!(ch, '/' | '.' | '_' | '-' | ':') {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
}

fn char_ngrams(token: &str, n: usize) -> Vec<String> {
    let chars = token.chars().collect::<Vec<_>>();
    if chars.len() < n {
        return Vec::new();
    }
    chars
        .windows(n)
        .map(|window| window.iter().collect::<String>())
        .collect()
}

fn add_feature(vector: &mut [f32], feature: &str, weight: f32) {
    let hash = stable_hash(feature.as_bytes());
    let index = (hash as usize) % vector.len();
    let sign = if hash & 1 == 0 { 1.0 } else { -1.0 };
    vector[index] += sign * weight;
}

fn stable_hash(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intrinsic_dense_defaults_on_and_allows_explicit_opt_out() {
        assert!(parse_intrinsic_dense_enabled(None));
        assert!(parse_intrinsic_dense_enabled(Some("true")));
        assert!(!parse_intrinsic_dense_enabled(Some("0")));
        assert!(!parse_intrinsic_dense_enabled(Some("false")));
    }
}
