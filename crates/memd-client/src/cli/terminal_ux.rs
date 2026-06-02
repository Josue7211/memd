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
    out.push_str("╔═════════════════════════════════════════════════════════════════╗\n");
    out.push_str("║  memd                                                            ║\n");
    let subtitle = truncate_to_width(subtitle, 61);
    let eyebrow = truncate_to_width(eyebrow, 61);
    let _ = writeln!(out, "║  ✦  {subtitle:<58} ║");
    let _ = writeln!(out, "║  {eyebrow:<61} ║");
    out.push_str("╚═════════════════════════════════════════════════════════════════╝\n");
    if !title.is_empty() {
        let _ = writeln!(out, "\n◈ {title}");
    }
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
    let _ = writeln!(out, "✦ {prompt}");
    out.push_str("  Choose a route. Enter accepts the highlighted default.\n\n");
    for (idx, option) in options.iter().enumerate() {
        let marker = if idx == selected { "◆" } else { "◇" };
        let _ = writeln!(out, "  {marker} {:>2}. {}", idx + 1, option.label);
        if let Some(description) = option.description {
            let _ = writeln!(out, "       {description}");
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

#[cfg(test)]
mod terminal_ux_tests {
    use super::*;

    #[test]
    fn brand_box_uses_memd_identity_without_hermes_copy() {
        let rendered = render_brand_box("memd Setup", "memory control plane", "SETUP / Provider");
        assert!(rendered.contains("memd"));
        assert!(rendered.contains("memory control plane"));
        assert!(rendered.contains("SETUP / Provider"));
        assert!(rendered.contains("◈ memd Setup"));
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
        assert!(rendered.contains("◇  1. Local only"));
        assert!(rendered.contains("◆  2. Shared memd server"));
        assert!(rendered.contains("Use team memory"));
    }

    #[test]
    fn redact_secret_keeps_shape_not_secret() {
        assert_eq!(redact_secret("1234567890abcdef"), "1234…cdef");
        assert_eq!(redact_secret("short"), "<redacted>");
    }
}
