use std::path::Path;
use std::sync::Mutex;

use anyhow::{Context, anyhow};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

pub(crate) struct Embedder {
    model: Mutex<TextEmbedding>,
    dim: usize,
}

impl Embedder {
    pub(crate) fn try_new(cache_dir: &Path) -> anyhow::Result<Self> {
        std::fs::create_dir_all(cache_dir).ok();
        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::AllMiniLML6V2)
                .with_cache_dir(cache_dir.to_path_buf())
                .with_show_download_progress(false),
        )
        .context("initialize fastembed AllMiniLML6V2")?;
        Ok(Self {
            model: Mutex::new(model),
            dim: 384,
        })
    }

    pub(crate) fn dim(&self) -> usize {
        self.dim
    }

    pub(crate) fn embed_normalized(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Ok(vec![0.0; self.dim]);
        }
        let mut model = self
            .model
            .lock()
            .map_err(|_| anyhow!("fastembed mutex poisoned"))?;
        let mut embeddings = model
            .embed(vec![trimmed.to_string()], None)
            .context("fastembed embed call failed")?;
        let mut vec = embeddings
            .pop()
            .ok_or_else(|| anyhow!("fastembed returned empty batch"))?;
        l2_normalize(&mut vec);
        Ok(vec)
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
    match std::env::var("MEMD_INTRINSIC_DENSE") {
        Ok(v) => {
            let v = v.trim().to_ascii_lowercase();
            matches!(v.as_str(), "1" | "true" | "on" | "yes")
        }
        Err(_) => false,
    }
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
