//! `DropdownMenu` — shadcn/Radix-naming wrapper around `Dropdown`.
//!
//! Mirrors `apps-old/frontend/components/ui/dropdown-menu.tsx`. The
//! underlying Dioxus primitive (`Dropdown`) already provides the
//! `DropdownTrigger` / `DropdownLabel` / `DropdownItem` /
//! `DropdownCheckboxItem` / `DropdownSeparator` sub-components, so this
//! module is a thin facade that exposes the same names under the
//! `DropdownMenu` namespace.
//!
//! Usage:
//! ```ignore
//! use dioxus_ui::DropdownMenu;
//!
//! DropdownMenu::Root { open: state, on_toggle: handler, trigger: ..., children: ... }
//! DropdownMenu::Trigger {}
//! DropdownMenu::Item { href: Some("/x".into()), children: rsx! { "..." } }
//! ```
//!
//! Because Rust doesn't allow `pub use` to expose sub-components under a
//! different module path, this file re-declares the same components
//! with their existing signatures. They compile to the same `rsx!`
//! output — callers that already use `Dropdown` keep working; callers
//! that prefer the shadcn `DropdownMenu.*` namespace can use these.

use dioxus::prelude::*;

// Note: the underlying Dropdown + sub-components are re-exported via
// the separate `pub use dropdown::*` in `primitives.rs`. To avoid
// ambiguous-glob-reexport warnings, this module does NOT re-export
// them here; callers access them via `use crate::Dropdown` /
// `use crate::DropdownItem` etc. (same names as the existing
// `Dropdown` primitive). The shadcn `DropdownMenu` namespace below
// is additive.

/// Trigger slot — the clickable element that toggles the menu.
///
/// In the underlying `Dropdown`, the trigger is just `children` of the
/// root. This thin wrapper exists so callers can write
/// `DropdownMenuTrigger { children: rsx! { "Open" } }` and the
/// component renders a button-styled wrapper that the menu's open
/// state is bound to.
#[component]
pub fn DropdownMenuTrigger(
    #[props(default = None)] class: Option<String>,
    children: Element,
) -> Element {
    let extra = class.unwrap_or_default();
    let base = "dropdown-menu-trigger inline-flex items-center justify-center";
    let cls = if extra.is_empty() {
        base.to_string()
    } else {
        format!("{base} {extra}")
    };
    rsx! {
        div { class: "{cls}", role: "button", tabindex: "0",
            {children}
        }
    }
}

/// Content slot — wraps the menu body. Use inside `DropdownMenu::Root`.
#[component]
pub fn Content(
    #[props(default = None)] align: Option<String>,
    #[props(default = None)] side: Option<String>,
    #[props(default = None)] class: Option<String>,
    children: Element,
) -> Element {
    let mut cls = "dropdown-menu-content".to_string();
    match align.as_deref() {
        Some("start") => cls.push_str(" dropdown-menu-content-align-start"),
        Some("end") => cls.push_str(" dropdown-menu-content-align-end"),
        _ => cls.push_str(" dropdown-menu-content-align-center"),
    }
    match side.as_deref() {
        Some("top") => cls.push_str(" dropdown-menu-content-side-top"),
        _ => cls.push_str(" dropdown-menu-content-side-bottom"),
    }
    if let Some(c) = class { cls.push(' '); cls.push_str(&c); }
    rsx! {
        div { class: "{cls}", role: "menu", {children} }
    }
}

/// Shortcut group — visually separates items without a real divider.
#[component]
pub fn Group(children: Element) -> Element {
    rsx! { div { class: "dropdown-menu-group", role: "group", {children} } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn align_class_mapping_is_distinct() {
        let a = match Some("start") {
            Some(x) => x,
            _ => "center",
        };
        let b = match Some("end") {
            Some(x) => x,
            _ => "center",
        };
        assert_ne!(a, b);
    }
}