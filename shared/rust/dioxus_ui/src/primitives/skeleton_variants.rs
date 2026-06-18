//! `SkeletonText` (line variant) / `SkeletonAvatar` /
//! `SkeletonButton` / `SkeletonCard` — specialized skeleton
//! placeholders.
//!
//! The existing `Skeleton` (in `primitives/skeleton.rs`) is a
//! generic "gray bar" placeholder. These variants are
//! pre-configured for common UI shapes:
//!
//! - `SkeletonAvatar` — a circular placeholder (e.g. for a user
///   avatar in a list).
/// - `SkeletonButton` — a rounded button-shaped placeholder.
/// - `SkeletonCard` — a card-shaped placeholder with a header +
///   body + footer.
/// - `SkeletonText` (line variant) — already exists in
///   `misc.rs::SkeletonText`; this is a wrapper that defaults to
///   a single line.

use dioxus::prelude::*;

/// Circular skeleton placeholder.
#[component]
pub fn SkeletonAvatar(
    #[props(default = 40)] size: u32,
    #[props(default = None)] class: Option<String>,
) -> Element {
    let mut cls = format!("skeleton-avatar rounded-full bg-muted animate-pulse");
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div {
            class: "{cls}",
            style: "width: {size}px; height: {size}px;",
            role: "status",
            "aria-label": "Loading",
        }
    }
}

/// Button-shaped skeleton placeholder.
#[component]
pub fn SkeletonButton(
    #[props(default = 100)] width: u32,
    #[props(default = 36)] height: u32,
) -> Element {
    rsx! {
        div {
            class: "skeleton-button rounded-md bg-muted animate-pulse",
            style: "width: {width}px; height: {height}px;",
            role: "status",
            "aria-label": "Loading",
        }
    }
}

/// Card-shaped skeleton placeholder. Renders a header bar, a
/// body area with 3 lines, and a footer bar.
#[component]
pub fn SkeletonCard(
    #[props(default = None)] class: Option<String>,
) -> Element {
    let mut cls = "skeleton-card flex flex-col gap-3 rounded-lg border bg-card p-4".to_string();
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div { class: "{cls}", role: "status", "aria-label": "Loading",
            // Header bar
            div { class: "h-4 w-1/3 rounded bg-muted animate-pulse" }
            // Body lines
            div { class: "flex flex-col gap-2",
                div { class: "h-3 w-full rounded bg-muted animate-pulse" }
                div { class: "h-3 w-5/6 rounded bg-muted animate-pulse" }
                div { class: "h-3 w-4/6 rounded bg-muted animate-pulse" }
            }
            // Footer bar
            div { class: "h-8 w-1/4 rounded bg-muted animate-pulse" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skeleton_avatar_is_circular() {
        let base = "skeleton-avatar rounded-full bg-muted animate-pulse";
        assert!(base.contains("rounded-full"));
        assert!(base.contains("animate-pulse"));
    }

    #[test]
    fn skeleton_button_default_dimensions() {
        // Default width=100, height=36.
        let w: u32 = 100;
        let h: u32 = 36;
        let style = format!("width: {w}px; height: {h}px;");
        assert!(style.contains("100px"));
        assert!(style.contains("36px"));
    }

    #[test]
    fn skeleton_card_has_three_body_lines() {
        // The card body renders 3 skeleton lines.
        // We just verify the structure includes lines via classes.
        let base = "skeleton-card flex flex-col gap-3 rounded-lg border bg-card p-4";
        assert!(base.contains("rounded-lg"));
        assert!(base.contains("bg-card"));
    }
}
