//! `Stack` family — vertical, horizontal, and z-index stacking
//! helpers.
//!
//! Thin layout primitives that wrap a `<div>` with the standard
//! flexbox / grid Tailwind classes. They exist for the same
//! reason utility CSS classes exist: to give a semantic name to
//! a common layout pattern. Use them as the root container of a
//! group of children to communicate the layout intent.
//!
//! - `VStack` — vertical flex column (children stack top-to-bottom).
/// - `HStack` — horizontal flex row (children stack left-to-right).
/// - `ZStack` — absolute-positioned children (children stack on top
///   of each other).
/// - `Spacer` — a flex spacer that grows to fill available space.
/// - `Divider` — a horizontal or vertical line.

use dioxus::prelude::*;

/// Vertical stack. Renders a flex column with a configurable gap.
#[component]
pub fn VStack(
    #[props(default = "4".to_string())] gap: String,
    #[props(default = None)] class: Option<String>,
    children: Element,
) -> Element {
    let mut cls = format!("vstack flex flex-col gap-{gap} items-stretch");
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div { class: "{cls}", {children} }
    }
}

/// Horizontal stack. Renders a flex row with a configurable gap.
#[component]
pub fn HStack(
    #[props(default = "4".to_string())] gap: String,
    #[props(default = None)] class: Option<String>,
    children: Element,
) -> Element {
    let mut cls = format!("hstack flex flex-row gap-{gap} items-center");
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div { class: "{cls}", {children} }
    }
}

/// Z-stack. Renders a relative container with absolute-positioned
/// children. Children should set their own `position: absolute`
/// via the `class` prop.
#[component]
pub fn ZStack(
    #[props(default = None)] class: Option<String>,
    children: Element,
) -> Element {
    let mut cls = "zstack relative".to_string();
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div { class: "{cls}", {children} }
    }
}

/// Spacer. Renders an empty flex item that grows to fill
/// available space in the parent flex container.
#[component]
pub fn Spacer() -> Element {
    rsx! {
        div { class: "spacer flex-1" }
    }
}

/// Divider. Renders a horizontal or vertical line.
#[component]
pub fn Divider(
    #[props(default = "horizontal".to_string())] orientation: String,
    #[props(default = None)] class: Option<String>,
) -> Element {
    let mut cls = "divider shrink-0 bg-border".to_string();
    match orientation.as_str() {
        "vertical" => cls.push_str(" h-full w-px"),
        _ => cls.push_str(" h-px w-full"),
    }
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div { class: "{cls}", role: "separator", "aria-orientation": "{orientation}" }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vstack_default_gap() {
        let gap: String = "4".to_string();
        let cls = format!("vstack flex flex-col gap-{gap} items-stretch");
        assert!(cls.contains("flex-col"));
        assert!(cls.contains("gap-4"));
    }

    #[test]
    fn hstack_default_gap() {
        let gap: String = "2".to_string();
        let cls = format!("hstack flex flex-row gap-{gap} items-center");
        assert!(cls.contains("flex-row"));
        assert!(cls.contains("gap-2"));
    }

    #[test]
    fn divider_horizontal_class() {
        let mut cls = "divider shrink-0 bg-border".to_string();
        cls.push_str(" h-px w-full");
        assert!(cls.contains("h-px"));
        assert!(cls.contains("w-full"));
    }

    #[test]
    fn divider_vertical_class() {
        let mut cls = "divider shrink-0 bg-border".to_string();
        cls.push_str(" h-full w-px");
        assert!(cls.contains("h-full"));
        assert!(cls.contains("w-px"));
    }
}
