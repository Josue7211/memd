#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct HarnessPreset {
    pub(crate) pack_id: &'static str,
    pub(crate) display_name: &'static str,
    pub(crate) entrypoint: &'static str,
    pub(crate) surface_set: &'static [&'static str],
    pub(crate) default_verbs: &'static [&'static str],
    pub(crate) cache_policy: &'static str,
    pub(crate) copy_tone: &'static str,
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
                    display_name: "Codex",
                    entrypoint: "memd wake --output .memd --intent current_task --write",
                    surface_set: &[
                        "MEMD_WAKEUP.md",
                        "MEMD_MEMORY.md",
                        "agents/CODEX_WAKEUP.md",
                        "agents/CODEX_MEMORY.md",
                    ],
                    default_verbs: &["wake", "resume", "checkpoint"],
                    cache_policy: "turn-scoped recall/capture cache",
                    copy_tone: "turn-first recall/capture pack",
                },
                HarnessPreset {
                    pack_id: "agent-zero",
                    display_name: "Agent Zero",
                    entrypoint: "memd wake --output .memd --intent current_task",
                    surface_set: &[
                        "MEMD_WAKEUP.md",
                        "MEMD_MEMORY.md",
                        "agents/AGENT_ZERO_WAKEUP.md",
                        "agents/AGENT_ZERO_MEMORY.md",
                    ],
                    default_verbs: &["resume", "remember", "handoff"],
                    cache_policy: "zero-friction startup cache",
                    copy_tone: "minimal ceremony and fresh-session path",
                },
                HarnessPreset {
                    pack_id: "openclaw",
                    display_name: "OpenClaw",
                    entrypoint: "memd context --project <project> --agent openclaw --compact",
                    surface_set: &[
                        "MEMD_WAKEUP.md",
                        "MEMD_MEMORY.md",
                        "agents/OPENCLAW_WAKEUP.md",
                        "agents/OPENCLAW_MEMORY.md",
                    ],
                    default_verbs: &["context", "resume", "spill"],
                    cache_policy: "compact-first spill cache",
                    copy_tone: "compact context and spill at boundaries",
                },
                HarnessPreset {
                    pack_id: "hermes",
                    display_name: "Hermes",
                    entrypoint: "memd wake --output .memd --intent current_task",
                    surface_set: &[
                        "MEMD_WAKEUP.md",
                        "MEMD_MEMORY.md",
                        "agents/HERMES_WAKEUP.md",
                        "agents/HERMES_MEMORY.md",
                    ],
                    default_verbs: &["wake", "resume", "capture"],
                    cache_policy: "onboarding-first startup cache",
                    copy_tone: "adoption-focused harness",
                },
                HarnessPreset {
                    pack_id: "opencode",
                    display_name: "OpenCode",
                    entrypoint: "memd resume --output .memd --intent current_task",
                    surface_set: &[
                        "MEMD_WAKEUP.md",
                        "MEMD_MEMORY.md",
                        "agents/OPENCODE_WAKEUP.md",
                        "agents/OPENCODE_MEMORY.md",
                    ],
                    default_verbs: &["resume", "remember", "handoff"],
                    cache_policy: "shared-lane continuity cache",
                    copy_tone: "explicit continuity verbs",
                },
            ],
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn get(&self, pack_id: &str) -> Option<&HarnessPreset> {
        self.packs.iter().find(|pack| pack.pack_id == pack_id)
    }
}
