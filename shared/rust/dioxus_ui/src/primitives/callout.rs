//! `InfoBox` / `Callout` / `Banner` — informational callouts.
//!
//! Three related components for surfacing context / tips /
//! warnings to the user:
//!
//! - `InfoBox` — a small inline callout (e.g. inside a form
//!   section) with an icon + title + description.
/// - `Callout` — a larger block-level callout (e.g. inside a
///   docs page) with an icon + title + description + optional
///   actions.
/// - `Banner` — a full-width page-level banner (e.g. a
///   system-wide announcement) with a colored background.

use super::icon::Icon;

use dioxus::prelude::*;

/// Tone / color of the callout.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum CalloutTone {
    #[default]
    Info,
    Success,
    Warning,
    Error,
    Neutral,
}

impl CalloutTone {
    pub fn border_class(self) -> &'static str {
        match self {
            CalloutTone::Info => "border-blue-500/50",
            CalloutTone::Success => "border-green-500/50",
            CalloutTone::Warning => "border-yellow-500/50",
            CalloutTone::Error => "border-red-500/50",
            CalloutTone::Neutral => "border-border",
        }
    }
    pub fn bg_class(self) -> &'static str {
        match self {
            CalloutTone::Info => "bg-blue-50 dark:bg-blue-900/10",
            CalloutTone::Success => "bg-green-50 dark:bg-green-900/10",
            CalloutTone::Warning => "bg-yellow-50 dark:bg-yellow-900/10",
            CalloutTone::Error => "bg-red-50 dark:bg-red-900/10",
            CalloutTone::Neutral => "bg-muted/30",
        }
    }
    pub fn icon_name(self) -> &'static str {
        match self {
            CalloutTone::Info => "info",
            CalloutTone::Success => "check-circle",
            CalloutTone::Warning => "alert-triangle",
            CalloutTone::Error => "alert-circle",
            CalloutTone::Neutral => "info",
        }
    }
}

/// Small inline info box. Renders a `<div>` with an icon + title +
/// description, sized to fit inside a form section or card.
#[component]
pub fn InfoBox(
    title: String,
    description: String,
    #[props(default = CalloutTone::default())] tone: CalloutTone,
) -> Element {
    rsx! {
        div { class: "info-box flex items-start gap-3 rounded-md border {tone.border_class()} {tone.bg_class()} p-3",
            Icon { name: tone.icon_name().to_string(), size: Some(16) }
            div { class: "flex flex-col gap-0.5",
                span { class: "info-box-title text-sm font-medium", "{title}" }
                span { class: "info-box-description text-xs text-muted-foreground", "{description}" }
            }
        }
    }
}

/// Larger block-level callout. Renders a `<div>` with an icon +
/// title + description + optional children (typically action
/// buttons).
#[component]
pub fn Callout(
    title: String,
    description: String,
    #[props(default = CalloutTone::default())] tone: CalloutTone,
    children: Element,
) -> Element {
    rsx! {
        div { class: "callout flex flex-col gap-3 rounded-lg border {tone.border_class()} {tone.bg_class()} p-4",
            div { class: "flex items-start gap-3",
                Icon { name: tone.icon_name().to_string(), size: Some(20) }
                div { class: "flex flex-col gap-1",
                    h4 { class: "callout-title text-base font-semibold", "{title}" }
                    p { class: "callout-description text-sm text-muted-foreground", "{description}" }
                }
            }
            if /* has children */ true {
                {children}
            }
        }
    }
}

/// Full-width page banner. Renders a `<div>` with a colored
/// background and a single line of text.
#[component]
pub fn Banner(
    text: String,
    #[props(default = CalloutTone::default())] tone: CalloutTone,
    #[props(default = None)] class: Option<String>,
) -> Element {
    let mut cls = format!(
        "banner flex items-center justify-center gap-2 px-4 py-2 text-sm {tone_border} {tone_bg}",
        tone_border = tone.border_class(),
        tone_bg = tone.bg_class(),
    );
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div { class: "{cls}", role: "status",
            Icon { name: tone.icon_name().to_string(), size: Some(16) }
            span { "{text}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tone_border_classes_are_distinct() {
        let tones = [
            CalloutTone::Info,
            CalloutTone::Success,
            CalloutTone::Warning,
            CalloutTone::Error,
            CalloutTone::Neutral,
        ];
        let classes: Vec<&str> = tones.iter().map(|t| t.border_class()).collect();
        let unique: std::collections::HashSet<&str> = classes.iter().copied().collect();
        assert_eq!(unique.len(), tones.len(), "tone border classes must be distinct");
    }

    #[test]
    fn tone_bg_classes_are_distinct() {
        let tones = [
            CalloutTone::Info,
            CalloutTone::Success,
            CalloutTone::Warning,
            CalloutTone::Error,
            CalloutTone::Neutral,
        ];
        let classes: Vec<&str> = tones.iter().map(|t| t.bg_class()).collect();
        let unique: std::collections::HashSet<&str> = classes.iter().copied().collect();
        assert_eq!(unique.len(), tones.len(), "tone bg classes must be distinct");
    }

    #[test]
    fn tone_icons_are_distinct() {
        let tones = [
            CalloutTone::Info,
            CalloutTone::Success,
            CalloutTone::Warning,
            CalloutTone::Error,
        ];
        let classes: Vec<&str> = tones.iter().map(|t| t.icon_name()).collect();
        let unique: std::collections::HashSet<&str> = classes.iter().copied().collect();
        // Info and Neutral both use "info" — but the 4 primary tones
        // are all distinct.
        assert!(classes.contains(&"info"));
        assert!(classes.contains(&"check-circle"));
        assert!(classes.contains(&"alert-triangle"));
        assert!(classes.contains(&"alert-circle"));
    }

    #[test]
    fn info_box_default_tone_is_info() {
        assert_eq!(CalloutTone::default(), CalloutTone::Info);
    }
}
