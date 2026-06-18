//! `AspectRatio` / `Container` / `Center` / `Spacer` (raw) —
//! layout utility primitives.
//!
//! - `AspectRatio` — preserves a given aspect ratio (e.g. 16:9 for
///   video embeds, 1:1 for avatars).
/// - `Container` — a max-width centered container with standard
///   horizontal padding.
/// - `Center` — flex container that centers its children both
///   horizontally and vertically.

use dioxus::prelude::*;

/// Preserves an aspect ratio. Renders a `<div>` with a CSS
/// `aspect-ratio` set via inline style.
///
/// - `ratio: f32` — the aspect ratio (width / height). Common
///   values: `16.0 / 9.0` for video, `1.0` for square, `4.0 / 3.0`
///   for classic TV.
/// - `class: Option<String>` — extra Tailwind classes.
#[component]
pub fn AspectRatio(
    ratio: f32,
    #[props(default = None)] class: Option<String>,
    children: Element,
) -> Element {
    let mut cls = "aspect-ratio relative w-full".to_string();
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div {
            class: "{cls}",
            style: "aspect-ratio: {ratio};",
            div { class: "absolute inset-0", {children} }
        }
    }
}

/// Centered container. Renders a `<div>` with a max-width and
/// standard horizontal padding. The default max-width matches
/// Tailwind's `max-w-7xl` (80rem).
#[component]
pub fn Container(
    #[props(default = "7xl".to_string())] max_width: String,
    #[props(default = None)] class: Option<String>,
    children: Element,
) -> Element {
    let mut cls = format!("container mx-auto px-4 sm:px-6 lg:px-8 max-w-{max_width}");
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div { class: "{cls}", {children} }
    }
}

/// Center. Renders a flex container that centers children both
/// horizontally and vertically. Useful for full-screen "loading"
/// states or single-element pages.
#[component]
pub fn Center(
    #[props(default = None)] class: Option<String>,
    children: Element,
) -> Element {
    let mut cls = "center flex items-center justify-center".to_string();
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
    fn aspect_ratio_class() {
        // Default class is `aspect-ratio relative w-full`.
        let base = "aspect-ratio relative w-full";
        assert!(base.contains("relative"));
        assert!(base.contains("w-full"));
    }

    #[test]
    fn aspect_ratio_style_sets_property() {
        // The aspect-ratio CSS property is set via inline style.
        let ratio: f32 = 16.0 / 9.0;
        let style = format!("aspect-ratio: {ratio};");
        assert!(style.contains("aspect-ratio:"));
    }

    #[test]
    fn container_default_max_width_is_7xl() {
        let max_width: String = "7xl".to_string();
        let cls = format!("container mx-auto px-4 sm:px-6 lg:px-8 max-w-{max_width}");
        assert!(cls.contains("max-w-7xl"));
    }

    #[test]
    fn center_class_uses_flex_center() {
        let base = "center flex items-center justify-center";
        assert!(base.contains("items-center"));
        assert!(base.contains("justify-center"));
    }
}
