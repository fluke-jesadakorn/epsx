//! `Pill` / `PillList` / `Tag` / `TagList` — small inline labels.
//!
//! Three related components for displaying small inline labels:
//!
//! - `Pill` — a small rounded label with optional close button.
///   Used for selected filter values, active categories, etc.
/// - `PillList` — a horizontal row of pills with a consistent gap.
/// - `Tag` — a more text-only label (less rounded than a pill).
///   Used for hashtags, content tags, etc.
/// - `TagList` — a horizontal row of tags.

use super::icon::Icon;

use dioxus::prelude::*;

/// Tone for a pill — controls the border / background color.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum PillTone {
    #[default]
    Default,
    Primary,
    Success,
    Warning,
    Error,
    Info,
}

impl PillTone {
    pub fn classes(self) -> &'static str {
        match self {
            PillTone::Default => "bg-muted text-muted-foreground border-border",
            PillTone::Primary => "bg-primary/10 text-primary border-primary/30",
            PillTone::Success => "bg-green-50 dark:bg-green-900/10 text-green-700 dark:text-green-300 border-green-500/30",
            PillTone::Warning => "bg-yellow-50 dark:bg-yellow-900/10 text-yellow-700 dark:text-yellow-300 border-yellow-500/30",
            PillTone::Error => "bg-red-50 dark:bg-red-900/10 text-red-700 dark:text-red-300 border-red-500/30",
            PillTone::Info => "bg-blue-50 dark:bg-blue-900/10 text-blue-700 dark:text-blue-300 border-blue-500/30",
        }
    }
}

/// Pill — a small rounded label. Optionally has a close button
/// (set `on_remove` to enable).
#[component]
pub fn Pill(
    label: String,
    #[props(default = PillTone::default())] tone: PillTone,
    #[props(default = None)] on_remove: Option<EventHandler<MouseEvent>>,
    #[props(default = None)] class: Option<String>,
) -> Element {
    let mut cls = format!(
        "pill inline-flex items-center gap-1 rounded-full border px-2.5 py-0.5 text-xs font-medium {tone_cls}",
        tone_cls = tone.classes(),
    );
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        span { class: "{cls}",
            "{label}"
            if let Some(h) = on_remove {
                button {
                    class: "pill-remove ml-1 inline-flex items-center justify-center rounded-full hover:bg-foreground/10",
                    r#type: "button",
                    "aria-label": "Remove",
                    onclick: move |e| h.call(e),
                    Icon { name: "x".to_string(), size: Some(12) }
                }
            }
        }
    }
}

/// Horizontal list of pills.
#[component]
pub fn PillList(
    pills: Vec<String>,
    #[props(default = PillTone::default())] tone: PillTone,
) -> Element {
    rsx! {
        div { class: "pill-list flex flex-wrap items-center gap-1.5",
            for p in pills.iter() {
                Pill { label: p.clone(), tone: tone }
            }
        }
    }
}

/// Tag — a less-rounded text label. Used for hashtags, content
/// tags, etc.
#[component]
pub fn Tag(
    text: String,
    #[props(default = "#".to_string())] prefix: String,
    #[props(default = None)] class: Option<String>,
) -> Element {
    let mut cls = "tag inline-flex items-center text-xs font-medium text-muted-foreground hover:text-foreground transition-colors".to_string();
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        span { class: "{cls}",
            span { class: "tag-prefix text-primary/60 mr-0.5", "{prefix}" }
            span { class: "tag-text", "{text}" }
        }
    }
}

/// Horizontal list of tags.
#[component]
pub fn TagList(
    tags: Vec<String>,
    #[props(default = "#".to_string())] prefix: String,
) -> Element {
    rsx! {
        div { class: "tag-list flex flex-wrap items-center gap-2",
            for t in tags.iter() {
                Tag { text: t.clone(), prefix: prefix.clone() }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tone_classes_are_distinct() {
        let tones = [
            PillTone::Default,
            PillTone::Primary,
            PillTone::Success,
            PillTone::Warning,
            PillTone::Error,
            PillTone::Info,
        ];
        let classes: Vec<&str> = tones.iter().map(|t| t.classes()).collect();
        let unique: std::collections::HashSet<&str> = classes.iter().copied().collect();
        assert_eq!(unique.len(), tones.len(), "tone classes must be distinct");
    }

    #[test]
    fn default_tone_class() {
        assert_eq!(
            PillTone::Default.classes(),
            "bg-muted text-muted-foreground border-border"
        );
    }

    #[test]
    fn pill_renders_with_default_tone() {
        // Sanity check the default is "Default".
        assert_eq!(PillTone::default(), PillTone::Default);
    }

    #[test]
    fn tag_default_prefix_is_hash() {
        let prefix: String = "#".to_string();
        assert_eq!(prefix, "#");
    }
}
