/// The canonical shared visible surfaces for all non-wake-only harnesses.
pub(crate) const SHARED_VISIBLE_SURFACES: &[&str] = &["wake.md", "mem.md", "events.md"];

/// Wake-only harnesses (like Claude Code) only import the wake surface by default.
pub(crate) const WAKE_ONLY_SURFACES: &[&str] = &["wake.md"];

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
    pub(crate) wake_char_budget: usize,
    /// If true, this harness only imports wake.md at boot. Deeper surfaces are cold-path.
    pub(crate) wake_only: bool,
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
                    entrypoint: "memd wake --output .memd --write",
                    surface_set: SHARED_VISIBLE_SURFACES,
                    default_verbs: &["wake", "lookup", "checkpoint", "spill"],
                    cache_policy: "turn-scoped recall/capture cache",
                    copy_tone: "turn-first recall/capture/spill pack",
                    wake_char_budget: 1800,
                    wake_only: false,
                },
                HarnessPreset {
                    pack_id: "claude-code",
                    display_name: "Claude Code",
                    entrypoint: "native import bridge",
                    surface_set: WAKE_ONLY_SURFACES,
                    default_verbs: &["wake", "resume", "lookup"],
                    cache_policy: "wake-only import bridge",
                    copy_tone: "native import bridge pack",
                    wake_char_budget: 1200,
                    wake_only: true,
                },
                HarnessPreset {
                    pack_id: "agent-zero",
                    display_name: "Agent Zero",
                    entrypoint: "memd wake --output .memd",
                    surface_set: SHARED_VISIBLE_SURFACES,
                    default_verbs: &["resume", "remember", "handoff", "spill"],
                    cache_policy: "zero-friction startup cache",
                    copy_tone: "minimal ceremony, fresh-session path, and spill",
                    wake_char_budget: 1800,
                    wake_only: false,
                },
                HarnessPreset {
                    pack_id: "openclaw",
                    display_name: "OpenClaw",
                    entrypoint: "memd context --project <project> --agent openclaw --compact",
                    surface_set: SHARED_VISIBLE_SURFACES,
                    default_verbs: &["context", "resume", "spill"],
                    cache_policy: "compact-first spill cache",
                    copy_tone: "compact context and spill at boundaries",
                    wake_char_budget: 1800,
                    wake_only: false,
                },
                HarnessPreset {
                    pack_id: "hermes",
                    display_name: "Hermes",
                    entrypoint: "memd wake --output .memd",
                    surface_set: SHARED_VISIBLE_SURFACES,
                    default_verbs: &["wake", "resume", "capture", "spill"],
                    cache_policy: "onboarding-first startup cache",
                    copy_tone: "adoption-focused wake/capture/spill harness",
                    wake_char_budget: 1800,
                    wake_only: false,
                },
                HarnessPreset {
                    pack_id: "opencode",
                    display_name: "OpenCode",
                    entrypoint: "memd resume --output .memd",
                    surface_set: SHARED_VISIBLE_SURFACES,
                    default_verbs: &["resume", "remember", "handoff", "spill"],
                    cache_policy: "shared-lane continuity cache",
                    copy_tone: "explicit continuity and spill verbs",
                    wake_char_budget: 1800,
                    wake_only: false,
                },
            ],
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn get(&self, pack_id: &str) -> Option<&HarnessPreset> {
        self.packs.iter().find(|pack| pack.pack_id == pack_id)
    }
}

pub(crate) fn wake_char_budget_for_agent(agent: Option<&str>) -> usize {
    let normalized = agent
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("codex")
        .to_ascii_lowercase();
    let registry = HarnessPresetRegistry::default_registry();
    // Try exact match first, then prefix match for session-qualified agents like "claude-code@session-xyz"
    registry
        .get(&normalized)
        .or_else(|| {
            registry
                .packs
                .iter()
                .find(|preset| normalized.starts_with(preset.pack_id))
        })
        .map(|preset| preset.wake_char_budget)
        .unwrap_or(1800)
}

#[allow(dead_code)] // Used in tests, reserved for wake-only routing in Phase H
pub(crate) fn is_wake_only_agent(agent: Option<&str>) -> bool {
    let normalized = agent
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("codex")
        .to_ascii_lowercase();
    let registry = HarnessPresetRegistry::default_registry();
    registry
        .get(&normalized)
        .or_else(|| {
            registry
                .packs
                .iter()
                .find(|preset| normalized.starts_with(preset.pack_id))
        })
        .is_some_and(|preset| preset.wake_only)
}
