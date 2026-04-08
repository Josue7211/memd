#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HarnessPreset {
    pub(crate) pack_id: &'static str,
    pub(crate) name: &'static str,
    pub(crate) role: &'static str,
    pub(crate) default_verbs: &'static [&'static str],
}

#[derive(Debug, Clone)]
pub(crate) struct HarnessPresetRegistry {
    pub(crate) packs: Vec<HarnessPreset>,
}

impl HarnessPresetRegistry {
    pub(crate) fn default_registry() -> Self {
        Self {
            packs: vec![
                HarnessPreset {
                    pack_id: "codex",
                    name: "Codex",
                    role: "turn-first recall/capture pack",
                    default_verbs: &["wake", "resume", "checkpoint"],
                },
                HarnessPreset {
                    pack_id: "agent-zero",
                    name: "Agent Zero",
                    role: "zero-friction resume/remember/handoff pack",
                    default_verbs: &["resume", "remember", "handoff"],
                },
                HarnessPreset {
                    pack_id: "openclaw",
                    name: "OpenClaw",
                    role: "compact context/spill pack",
                    default_verbs: &["context", "resume", "spill"],
                },
                HarnessPreset {
                    pack_id: "hermes",
                    name: "Hermes",
                    role: "adoption-focused wake/capture pack",
                    default_verbs: &["wake", "resume", "capture"],
                },
                HarnessPreset {
                    pack_id: "opencode",
                    name: "OpenCode",
                    role: "resume/remember/handoff pack",
                    default_verbs: &["resume", "remember", "handoff"],
                },
            ],
        }
    }

    pub(crate) fn get(&self, pack_id: &str) -> Option<&HarnessPreset> {
        self.packs.iter().find(|pack| pack.pack_id == pack_id)
    }
}
