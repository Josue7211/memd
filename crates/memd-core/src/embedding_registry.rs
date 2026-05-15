use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmbeddingProvider {
    OpenAi,
    Local,
    Sidecar,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmbeddingRole {
    Dense,
    Sparse,
    Rerank,
    Hybrid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddingModelProfile {
    pub id: String,
    pub provider: EmbeddingProvider,
    pub role: EmbeddingRole,
    pub dimensions: Option<usize>,
    pub local: bool,
    pub cloud: bool,
    pub quality_tier: u8,
    pub cost_tier: u8,
    pub latency_tier: u8,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddingModelRegistry {
    pub schema_version: u8,
    pub default_cloud: String,
    pub default_local: String,
    pub default_hybrid: String,
    pub profiles: Vec<EmbeddingModelProfile>,
}

impl EmbeddingModelRegistry {
    pub fn builtin() -> Self {
        let profiles = vec![
            profile(
                "text-embedding-3-large",
                EmbeddingProvider::OpenAi,
                EmbeddingRole::Dense,
                Some(3072),
                false,
                true,
                (5, 3, 3),
                "cloud quality default",
            ),
            profile(
                "text-embedding-3-small",
                EmbeddingProvider::OpenAi,
                EmbeddingRole::Dense,
                Some(1536),
                false,
                true,
                (4, 5, 4),
                "cloud cost default",
            ),
            profile(
                "qwen3-embedding-8b",
                EmbeddingProvider::Local,
                EmbeddingRole::Dense,
                Some(4096),
                true,
                false,
                (5, 4, 2),
                "local high-quality GPU tier",
            ),
            profile(
                "qwen3-reranker-8b",
                EmbeddingProvider::Local,
                EmbeddingRole::Rerank,
                None,
                true,
                false,
                (5, 4, 2),
                "local top-window rerank tier",
            ),
            profile(
                "bge-m3",
                EmbeddingProvider::Local,
                EmbeddingRole::Hybrid,
                Some(1024),
                true,
                false,
                (4, 5, 4),
                "local hybrid dense/sparse/multivector tier",
            ),
            profile(
                "rag-sidecar:sparse",
                EmbeddingProvider::Sidecar,
                EmbeddingRole::Sparse,
                None,
                true,
                false,
                (3, 5, 5),
                "sidecar fallback profile; no dense model required",
            ),
            profile(
                "rag-sidecar:fastembed",
                EmbeddingProvider::Sidecar,
                EmbeddingRole::Dense,
                None,
                true,
                false,
                (4, 5, 4),
                "sidecar dense local profile",
            ),
        ];
        Self {
            schema_version: 1,
            default_cloud: "text-embedding-3-large".to_string(),
            default_local: "qwen3-embedding-8b".to_string(),
            default_hybrid: "bge-m3".to_string(),
            profiles,
        }
    }

    pub fn select(&self, id: &str) -> Option<&EmbeddingModelProfile> {
        self.profiles.iter().find(|profile| profile.id == id)
    }

    pub fn recommended_for(&self, target: &str) -> Option<&EmbeddingModelProfile> {
        match target {
            "cloud" => self.select(&self.default_cloud),
            "local" => self.select(&self.default_local),
            "hybrid" => self.select(&self.default_hybrid),
            _ => None,
        }
    }

    pub fn merge_profiles(&mut self, profiles: Vec<EmbeddingModelProfile>) {
        for profile in profiles {
            if let Some(existing) = self
                .profiles
                .iter_mut()
                .find(|existing| existing.id == profile.id)
            {
                *existing = profile;
            } else {
                self.profiles.push(profile);
            }
        }
    }
}

fn profile(
    id: &str,
    provider: EmbeddingProvider,
    role: EmbeddingRole,
    dimensions: Option<usize>,
    local: bool,
    cloud: bool,
    tiers: (u8, u8, u8),
    notes: &str,
) -> EmbeddingModelProfile {
    EmbeddingModelProfile {
        id: id.to_string(),
        provider,
        role,
        dimensions,
        local,
        cloud,
        quality_tier: tiers.0,
        cost_tier: tiers.1,
        latency_tier: tiers.2,
        notes: notes.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_registry_names_cloud_local_and_hybrid_defaults() {
        let registry = EmbeddingModelRegistry::builtin();

        assert_eq!(
            registry
                .recommended_for("cloud")
                .map(|profile| profile.id.as_str()),
            Some("text-embedding-3-large")
        );
        assert_eq!(
            registry
                .recommended_for("local")
                .map(|profile| profile.id.as_str()),
            Some("qwen3-embedding-8b")
        );
        assert_eq!(
            registry
                .recommended_for("hybrid")
                .map(|profile| profile.id.as_str()),
            Some("bge-m3")
        );
    }

    #[test]
    fn custom_profiles_replace_by_id() {
        let mut registry = EmbeddingModelRegistry::builtin();
        registry.merge_profiles(vec![EmbeddingModelProfile {
            id: "bge-m3".to_string(),
            provider: EmbeddingProvider::Local,
            role: EmbeddingRole::Hybrid,
            dimensions: Some(1024),
            local: true,
            cloud: false,
            quality_tier: 5,
            cost_tier: 5,
            latency_tier: 4,
            notes: "bench upgraded".to_string(),
        }]);

        assert_eq!(
            registry
                .select("bge-m3")
                .map(|profile| profile.notes.as_str()),
            Some("bench upgraded")
        );
    }
}
