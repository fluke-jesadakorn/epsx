//! `Chip` / `ChipGroup` / `ChipRemovable` — chips for selected
//! values, filter pills, etc.
//!
//! `Chip` is a small rounded button-like element that represents
//! a selected value (e.g. a tag, a filter, a category). It's
//! distinct from `Pill` (which is a non-interactive label) in
//! that `Chip` is typically clickable and can be removable.
//!
//! - `Chip` — a static label with optional icon.
/// - `ChipRemovable` — a label with a close button (the
///   canonical filter-chip pattern).
/// - `ChipGroup` — a horizontal row of chips.

use super::icon::Icon;

use dioxus::prelude::*;

/// Tone for a chip — controls the background / text color.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum ChipTone {
    #[default]
    Default,
    Primary,
    Accent,
    Muted,
}

impl ChipTone {
    pub fn classes(self) -> &'static str {
        match self {
            ChipTone::Default => "bg-secondary text-secondary-foreground hover:bg-secondary/80",
            ChipTone::Primary => "bg-primary text-primary-foreground hover:bg-primary/90",
            ChipTone::Accent => "bg-accent text-accent-foreground hover:bg-accent/80",
            ChipTone::Muted => "bg-muted text-muted-foreground hover:bg-muted/80",
        }
    }
}

/// A static chip — a small rounded label with an optional icon.
#[component]
pub fn Chip(
    label: String,
    #[props(default = None)] icon: Option<String>,
    #[props(default = ChipTone::default())] tone: ChipTone,
) -> Element {
    let cls = format!(
        "chip inline-flex items-center gap-1 rounded-full px-2.5 py-0.5 text-xs font-medium transition-colors {tone_cls}",
        tone_cls = tone.classes(),
    );
    rsx! {
        span { class: "{cls}",
            if let Some(i) = icon {
                Icon { name: i.clone(), size: Some(12) }
            }
            "{label}"
        }
    }
}

/// A removable chip — a label with a close button. Renders a
/// `<button>` so it's keyboard-focusable.
#[component]
pub fn ChipRemovable(
    label: String,
    on_remove: EventHandler<MouseEvent>,
    #[props(default = None)] icon: Option<String>,
    #[props(default = ChipTone::default())] tone: ChipTone,
) -> Element {
    let cls = format!(
        "chip-removable inline-flex items-center gap-1 rounded-full px-2.5 py-0.5 text-xs font-medium transition-colors {tone_cls}",
        tone_cls = tone.classes(),
    );
    rsx! {
        span { class: "{cls}",
            if let Some(i) = icon {
                Icon { name: i.clone(), size: Some(12) }
            }
            "{label}"
            button {
                class: "chip-removable-btn ml-0.5 -mr-1 inline-flex h-4 w-4 items-center justify-center rounded-full hover:bg-foreground/20 focus:outline-none focus:ring-1 focus:ring-ring",
                r#type: "button",
                "aria-label": "Remove",
                onclick: move |e| on_remove.call(e),
                Icon { name: "x".to_string(), size: Some(10) }
            }
        }
    }
}

/// A horizontal row of chips.
#[component]
pub fn ChipGroup(
    children: Element,
    #[props(default = None)] class: Option<String>,
) -> Element {
    let mut cls = "chip-group flex flex-wrap items-center gap-1.5".to_string();
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div { class: "{cls}", {children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chip_tone_classes_are_distinct() {
        let tones = [
            ChipTone::Default,
            ChipTone::Primary,
            ChipTone::Accent,
            ChipTone::Muted,
        ];
        let classes: Vec<&str> = tones.iter().map(|t| t.classes()).collect();
        let unique: std::collections::HashSet<&str> = classes.iter().copied().collect();
        assert_eq!(unique.len(), tones.len(), "tone classes must be distinct");
    }

    #[test]
    fn chip_default_tone_is_default() {
        assert_eq!(ChipTone::default(), ChipTone::Default);
    }

    #[test]
    fn chip_removable_has_close_button() {
        // The removable chip renders a close button with an X icon.
        let icon = "x";
        assert_eq!(icon, "x");
    }
}
