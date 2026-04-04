use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use memd_sidecar::{
    SidecarIngestRequest, SidecarIngestSource, SidecarRetrieveMode, SidecarRetrieveRequest,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MultimodalAssetKind {
    Pdf,
    Image,
    Video,
    Table,
    Equation,
    Text,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExtractionBackend {
    Mineru,
    RagAnything,
    TextFallback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalAsset {
    pub path: PathBuf,
    pub kind: MultimodalAssetKind,
    pub mime: Option<String>,
    pub bytes: Option<u64>,
    pub backend: ExtractionBackend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalIngestPlan {
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub assets: Vec<MultimodalAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalChunk {
    pub id: Uuid,
    pub asset_path: PathBuf,
    pub kind: MultimodalAssetKind,
    pub backend: ExtractionBackend,
    pub mime: Option<String>,
    pub bytes: Option<u64>,
    pub content: String,
    pub confidence: f32,
}

pub fn build_ingest_plan(
    paths: impl IntoIterator<Item = impl AsRef<Path>>,
    project: Option<String>,
    namespace: Option<String>,
) -> anyhow::Result<MultimodalIngestPlan> {
    let mut assets = Vec::new();
    for path in paths {
        assets.push(classify_asset(path.as_ref())?);
    }

    Ok(MultimodalIngestPlan {
        project,
        namespace,
        assets,
    })
}

pub fn extract_chunks(plan: &MultimodalIngestPlan) -> anyhow::Result<Vec<MultimodalChunk>> {
    let mut chunks = Vec::with_capacity(plan.assets.len());
    for asset in &plan.assets {
        chunks.push(extract_chunk(asset)?);
    }
    Ok(chunks)
}

pub fn to_sidecar_requests(
    plan: &MultimodalIngestPlan,
    chunks: &[MultimodalChunk],
) -> Vec<SidecarIngestRequest> {
    let project = plan.project.clone();
    let namespace = plan.namespace.clone();
    chunks
        .iter()
        .map(|chunk| SidecarIngestRequest {
            project: project.clone(),
            namespace: namespace.clone(),
            source: SidecarIngestSource {
                id: chunk.id,
                kind: format!("{:?}", chunk.kind).to_lowercase(),
                content: chunk.content.clone(),
                mime: chunk.mime.clone(),
                bytes: chunk.bytes,
                source_quality: Some(memd_schema::SourceQuality::Derived),
                source_agent: Some(match chunk.backend {
                    ExtractionBackend::Mineru => "mineru".to_string(),
                    ExtractionBackend::RagAnything => "raganything".to_string(),
                    ExtractionBackend::TextFallback => "multimodal".to_string(),
                }),
                source_path: Some(chunk.asset_path.display().to_string()),
                tags: vec![
                    format!("asset_kind={:?}", chunk.kind).to_lowercase(),
                    format!("backend={:?}", chunk.backend).to_lowercase(),
                ],
            },
        })
        .collect()
}

fn classify_asset(path: &Path) -> anyhow::Result<MultimodalAsset> {
    let metadata = fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;
    let mime = mime_guess::from_path(path)
        .first_raw()
        .map(|value| value.to_string());
    let kind = classify_kind(path, mime.as_deref());
    let backend = match kind {
        MultimodalAssetKind::Pdf => ExtractionBackend::Mineru,
        MultimodalAssetKind::Image
        | MultimodalAssetKind::Video
        | MultimodalAssetKind::Table
        | MultimodalAssetKind::Equation => ExtractionBackend::RagAnything,
        MultimodalAssetKind::Text | MultimodalAssetKind::Unknown => ExtractionBackend::TextFallback,
    };

    Ok(MultimodalAsset {
        path: path.to_path_buf(),
        kind,
        mime,
        bytes: Some(metadata.len()),
        backend,
    })
}

fn extract_chunk(asset: &MultimodalAsset) -> anyhow::Result<MultimodalChunk> {
    let content = match asset.kind {
        MultimodalAssetKind::Text => fs::read_to_string(&asset.path)
            .with_context(|| format!("read {}", asset.path.display()))?,
        _ => format!(
            "multimodal_asset path={} kind={:?} backend={:?} mime={}",
            asset.path.display(),
            asset.kind,
            asset.backend,
            asset.mime.as_deref().unwrap_or("unknown")
        ),
    };

    Ok(MultimodalChunk {
        id: Uuid::new_v4(),
        asset_path: asset.path.clone(),
        kind: asset.kind,
        backend: asset.backend,
        mime: asset.mime.clone(),
        bytes: asset.bytes,
        content,
        confidence: match asset.kind {
            MultimodalAssetKind::Text => 0.95,
            MultimodalAssetKind::Pdf => 0.9,
            MultimodalAssetKind::Image => 0.86,
            MultimodalAssetKind::Video => 0.82,
            MultimodalAssetKind::Table => 0.88,
            MultimodalAssetKind::Equation => 0.84,
            MultimodalAssetKind::Unknown => 0.5,
        },
    })
}

fn classify_kind(path: &Path, mime: Option<&str>) -> MultimodalAssetKind {
    let ext = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.trim().to_ascii_lowercase());

    match (ext.as_deref(), mime) {
        (Some("pdf"), _) | (_, Some("application/pdf")) => MultimodalAssetKind::Pdf,
        (Some("png"), _)
        | (Some("jpg"), _)
        | (Some("jpeg"), _)
        | (Some("webp"), _)
        | (Some("heic"), _) => MultimodalAssetKind::Image,
        (Some("mp4"), _)
        | (Some("mov"), _)
        | (Some("mkv"), _)
        | (Some("webm"), _)
        | (_, Some("video/mp4"))
        | (_, Some("video/webm")) => MultimodalAssetKind::Video,
        (Some("csv"), _) | (Some("tsv"), _) | (Some("xlsx"), _) | (_, Some("text/csv")) => {
            MultimodalAssetKind::Table
        }
        (Some("tex"), _) | (Some("mml"), _) | (_, Some("application/mathml+xml")) => {
            MultimodalAssetKind::Equation
        }
        (Some("txt"), _)
        | (Some("md"), _)
        | (Some("json"), _)
        | (Some("yaml"), _)
        | (Some("yml"), _)
        | (_, Some("text/plain")) => MultimodalAssetKind::Text,
        _ => MultimodalAssetKind::Unknown,
    }
}

pub fn build_retrieve_request(
    query: impl Into<String>,
    project: Option<String>,
    namespace: Option<String>,
    limit: Option<usize>,
    include_cross_modal: bool,
) -> SidecarRetrieveRequest {
    SidecarRetrieveRequest {
        query: query.into(),
        project,
        namespace,
        mode: SidecarRetrieveMode::Multimodal,
        limit,
        include_cross_modal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_file(name: &str, contents: &str) -> PathBuf {
        let dir = std::env::temp_dir();
        let file_name = match name.rsplit_once('.') {
            Some((stem, ext)) if !stem.is_empty() && !ext.is_empty() => {
                format!("memd-multimodal-{}-{}.{}", stem, Uuid::new_v4(), ext)
            }
            _ => format!("memd-multimodal-{}-{}", name, Uuid::new_v4()),
        };
        let path = dir.join(file_name);
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
        path
    }

    #[test]
    fn classifies_pdf_image_and_text() {
        let pdf = temp_file("report.pdf", "%PDF-1.7");
        let image = temp_file("diagram.png", "fake");
        let text = temp_file("notes.md", "# heading");

        assert_eq!(classify_asset(&pdf).unwrap().kind, MultimodalAssetKind::Pdf);
        assert_eq!(
            classify_asset(&image).unwrap().kind,
            MultimodalAssetKind::Image
        );
        assert_eq!(
            classify_asset(&text).unwrap().kind,
            MultimodalAssetKind::Text
        );
    }

    #[test]
    fn builds_sidecar_requests_with_backend_hints() {
        let text = temp_file("notes.md", "hello");
        let plan = build_ingest_plan([&text], Some("memd".into()), Some("main".into())).unwrap();
        let chunks = extract_chunks(&plan).unwrap();
        let requests = to_sidecar_requests(&plan, &chunks);

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].project.as_deref(), Some("memd"));
        assert_eq!(
            requests[0].source.source_quality,
            Some(memd_schema::SourceQuality::Derived)
        );
        assert!(requests[0].source.bytes.is_some());
        assert_eq!(requests[0].source.mime.as_deref(), Some("text/markdown"));
        assert!(
            requests[0]
                .source
                .tags
                .iter()
                .any(|tag| tag.contains("backend"))
        );
    }
}
