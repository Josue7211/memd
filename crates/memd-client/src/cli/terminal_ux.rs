use std::fmt::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CheckState {
    Ready,
    Pending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MenuOption<'a> {
    pub(crate) label: &'a str,
    pub(crate) description: Option<&'a str>,
}

impl<'a> MenuOption<'a> {
    pub(crate) const fn new(label: &'a str) -> Self {
        Self {
            label,
            description: None,
        }
    }

    pub(crate) const fn with_description(label: &'a str, description: &'a str) -> Self {
        Self {
            label,
            description: Some(description),
        }
    }
}

pub(crate) fn render_brand_box(title: &str, subtitle: &str, eyebrow: &str) -> String {
    let mut out = String::new();
    let title_line = truncate_to_width(title, 56);
    let subtitle = truncate_to_width(subtitle, 56);
    let eyebrow = truncate_to_width(eyebrow, 56);
    out.push_str("╭────────────────────────────────────────────────────────────────────────╮\n");
    out.push_str("│                                                                        │\n");
    out.push_str("│  ███╗   ███╗ ███████╗ ███╗   ███╗ ██████╗                             │\n");
    out.push_str("│  ████╗ ████║ ██╔════╝ ████╗ ████║ ██╔══██╗                            │\n");
    out.push_str("│  ██╔████╔██║ █████╗   ██╔████╔██║ ██║  ██║                            │\n");
    out.push_str("│  ██║╚██╔╝██║ ██╔══╝   ██║╚██╔╝██║ ██║  ██║                            │\n");
    out.push_str("│  ██║ ╚═╝ ██║ ███████╗ ██║ ╚═╝ ██║ ██████╔╝                            │\n");
    out.push_str("│  ╚═╝     ╚═╝ ╚══════╝ ╚═╝     ╚═╝ ╚═════╝                             │\n");
    out.push_str("│                                                                        │\n");
    let _ = writeln!(out, "│  ⚡ {title_line:<66}│");
    let _ = writeln!(out, "│  ◆ {subtitle:<66}│");
    let _ = writeln!(out, "│  ◇ {eyebrow:<66}│");
    out.push_str("│                                                                        │\n");
    out.push_str("╰────────────────────────────────────────────────────────────────────────╯\n");
    out
}

pub(crate) fn render_section_header(title: &str, body: &str) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "✦ {title}");
    if !body.is_empty() {
        let _ = writeln!(out, "  {body}");
    }
    out
}

pub(crate) fn render_checklist(items: &[(&str, CheckState)]) -> String {
    let mut out = String::new();
    for (label, state) in items {
        let mark = match state {
            CheckState::Ready => "✓",
            CheckState::Pending => "•",
        };
        let _ = writeln!(out, "  {mark} {label}");
    }
    out
}

pub(crate) fn render_selector(prompt: &str, options: &[MenuOption<'_>], selected: usize) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "◆ {prompt}");
    out.push_str("  Select by number, Enter to confirm.\n\n");
    for (idx, option) in options.iter().enumerate() {
        let marker = if idx == selected { "(●)" } else { "(○)" };
        let _ = writeln!(out, "  {marker} {:>2}. {}", idx + 1, option.label);
        if let Some(description) = option.description {
            let _ = writeln!(out, "       — {description}");
        }
    }
    out.push('\n');
    out.push_str("  ↑/↓ move   Enter select   q quit");
    out
}

pub(crate) fn redact_secret(value: &str) -> String {
    if value.is_empty() {
        return "<empty>".to_string();
    }
    if value.len() <= 8 {
        return "<redacted>".to_string();
    }
    format!("{}…{}", &value[..4], &value[value.len() - 4..])
}

fn truncate_to_width(value: &str, width: usize) -> String {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= width {
        value.to_string()
    } else if width <= 1 {
        "…".to_string()
    } else {
        let mut truncated: String = chars.into_iter().take(width - 1).collect();
        truncated.push('…');
        truncated
    }
}

pub(crate) fn render_panel(title: &str, subtitle: &str, sections: &[PanelSection<'_>]) -> String {
    let mut out = String::new();
    out.push_str(&render_brand_box(
        title,
        subtitle,
        &title.to_ascii_uppercase(),
    ));
    for (idx, section) in sections.iter().enumerate() {
        if idx == 0 {
            out.push('\n');
        }
        let _ = writeln!(out, "◆ {}", section.title);
        if let Some(body) = section.body {
            let _ = writeln!(out, "  {body}");
        }
        for row in section.rows {
            let _ = writeln!(out, "  {:<20} {}", row.label, row.value);
        }
        if idx + 1 != sections.len() {
            out.push('\n');
        }
    }
    out.trim_end().to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PanelSection<'a> {
    pub(crate) title: &'a str,
    pub(crate) body: Option<&'a str>,
    pub(crate) rows: &'a [PanelRow<'a>],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PanelRow<'a> {
    pub(crate) label: &'a str,
    pub(crate) value: &'a str,
}

pub(crate) fn ready_mark(ready: bool) -> &'static str {
    if ready { "✓" } else { "✗" }
}

pub(crate) fn render_home_help() -> String {
    let start_rows = [
        PanelRow {
            label: "setup",
            value: "First-run setup wizard and local bundle onboarding",
        },
        PanelRow {
            label: "status",
            value: "Readable health, server, memory, and next-action dashboard",
        },
        PanelRow {
            label: "doctor",
            value: "Diagnose bundle readiness and repair drift",
        },
        PanelRow {
            label: "settings",
            value: "View or edit project, route, voice, hive, and authority config",
        },
    ];
    let memory_rows = [
        PanelRow {
            label: "lookup",
            value: "Ask memory what this agent should know before answering",
        },
        PanelRow {
            label: "teach",
            value: "Save a canonical fact, preference, or procedure",
        },
        PanelRow {
            label: "remember",
            value: "Save a durable memory item with metadata",
        },
        PanelRow {
            label: "wake",
            value: "Refresh the startup memory surface",
        },
    ];
    let explore_rows = [
        PanelRow {
            label: "memory",
            value: "Browse lanes, memory items, and local memory artifacts",
        },
        PanelRow {
            label: "skills",
            value: "Inspect installed skills and the discovered skill catalog",
        },
        PanelRow {
            label: "commands",
            value: "Open the full command catalog for power users",
        },
    ];
    let advanced_rows = [
        PanelRow {
            label: "memd commands",
            value: "Browse the full command catalog for power users",
        },
        PanelRow {
            label: "memd help <command>",
            value: "Open detailed Clap help and flags for one command",
        },
        PanelRow {
            label: "--summary",
            value: "Print compact one-line output where supported",
        },
        PanelRow {
            label: "--json",
            value: "Print parseable machine output where supported",
        },
    ];
    let sections = [
        PanelSection {
            title: "Start here",
            body: Some("Most users only need these setup and health commands."),
            rows: &start_rows,
        },
        PanelSection {
            title: "Memory",
            body: Some("Recall, teach, and refresh durable memory."),
            rows: &memory_rows,
        },
        PanelSection {
            title: "Explore",
            body: Some("Discover more without dumping every internal command."),
            rows: &explore_rows,
        },
        PanelSection {
            title: "Advanced",
            body: Some("The full CLI is still available, just not dumped on first help."),
            rows: &advanced_rows,
        },
    ];
    let mut out = render_panel("memd Help", "memory control plane", &sections);
    out.push_str(
        "

Usage: memd [OPTIONS] <COMMAND>

Options:
  --base-url <BASE_URL>    override shared server URL
  -h, --help               show this help
",
    );
    out
}

#[cfg(test)]
mod terminal_ux_tests {
    use super::*;

    #[test]
    fn brand_box_uses_memd_identity_without_hermes_copy() {
        let rendered = render_brand_box("memd Setup", "memory control plane", "SETUP / Provider");
        assert!(rendered.contains("memd"));
        assert!(rendered.contains("memory control plane"));
        assert!(rendered.contains("SETUP / Provider"));
        assert!(!rendered.contains("Hermes Agent Setup Wizard"));
    }

    #[test]
    fn selector_marks_current_option() {
        let rendered = render_selector(
            "Pick one",
            &[
                MenuOption::new("Local only"),
                MenuOption::with_description("Shared memd server", "Use team memory"),
            ],
            1,
        );
        assert!(rendered.contains("(○)  1. Local only"));
        assert!(rendered.contains("(●)  2. Shared memd server"));
        assert!(rendered.contains("Use team memory"));
    }

    #[test]
    fn redact_secret_keeps_shape_not_secret() {
        assert_eq!(redact_secret("1234567890abcdef"), "1234…cdef");
        assert_eq!(redact_secret("short"), "<redacted>");
    }

    #[test]
    fn home_help_is_short_and_points_to_command_catalog() {
        let rendered = render_home_help();
        assert!(rendered.contains("memd Help"));
        assert!(rendered.contains("Start here"));
        assert!(rendered.contains("Memory"));
        assert!(rendered.contains("Explore"));
        assert!(rendered.contains("Advanced"));
        for command in [
            "setup",
            "status",
            "doctor",
            "settings",
            "lookup",
            "teach",
            "remember",
            "wake",
            "memory",
            "skills",
            "commands",
            "memd commands",
            "memd help <command>",
            "--summary",
            "--json",
        ] {
            assert!(
                rendered.contains(command),
                "missing curated command: {command}"
            );
        }
        assert!(!rendered.contains("healthz"));
    }

    #[test]
    fn panel_renders_sections_and_rows() {
        let rows = [PanelRow {
            label: "Ready",
            value: "✓ true",
        }];
        let sections = [PanelSection {
            title: "Runtime",
            body: Some("Current state"),
            rows: &rows,
        }];
        let rendered = render_panel("memd Settings", "memory control plane", &sections);
        assert!(rendered.contains("memd Settings"));
        assert!(rendered.contains("◆ Runtime"));
        assert!(rendered.contains("Ready"));
    }
}
