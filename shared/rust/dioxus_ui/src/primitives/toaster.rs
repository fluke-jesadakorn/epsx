//! `Toaster` — alias / thin expansion of `ToastProvider`.
//!
//! Mirrors `apps-old/frontend/components/ui/toaster.tsx` (which
//! re-exports `sonner`'s `Toaster`). The Dioxus implementation already
//! has a `ToastProvider` in `feedback/toast.rs` that mounts a toast
//! container. This component:
//!
//! 1. Acts as a `pub use` alias so callers can `use dioxus_ui::Toaster;`
//!    matching the Next.js import shape.
//! 2. Adds an optional `position` prop (`"bottom-right"`, `"top-right"`,
//!    `"top-left"`, `"bottom-left"`, `"top-center"`, `"bottom-center"`)
//!    that maps to a container class name.
//! 3. Wraps the existing `ToastProvider` so the
//!    `push_toast` / `push_info` / `push_success` / `push_warning` /
//!    `push_error` API stays unchanged.
//!
//! The implementation is intentionally tiny — the heavy lifting lives
//! in `feedback::toast`.

use crate::feedback::toast::ToastProvider;

use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ToasterPosition {
    TopLeft, TopCenter, TopRight,
    BottomLeft, BottomCenter, BottomRight,
}

impl ToasterPosition {
    pub fn class(&self) -> &'static str {
        match self {
            ToasterPosition::TopLeft => "toast-container toast-container-top-left",
            ToasterPosition::TopCenter => "toast-container toast-container-top-center",
            ToasterPosition::TopRight => "toast-container toast-container-top-right",
            ToasterPosition::BottomLeft => "toast-container toast-container-bottom-left",
            ToasterPosition::BottomCenter => "toast-container toast-container-bottom-center",
            ToasterPosition::BottomRight => "toast-container toast-container-bottom-right",
        }
    }
}

impl Default for ToasterPosition {
    fn default() -> Self {
        ToasterPosition::BottomRight
    }
}

/// Container for toast notifications. Drop one of these into the root
/// layout (usually right before `</body>`) to mount the toast UI.
///
/// The component re-uses `ToastProvider` under the hood, so all the
/// existing `push_toast` / `push_info` / `push_success` / `push_warning`
/// / `push_error` helpers continue to work.
#[component]
pub fn Toaster(
    #[props(default = ToasterPosition::BottomRight)] position: ToasterPosition,
) -> Element {
    let pos_cls = position.class();
    rsx! {
        div { class: "{pos_cls}",
            ToastProvider {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_positions_have_unique_class_strings() {
        let positions = [
            ToasterPosition::TopLeft,
            ToasterPosition::TopCenter,
            ToasterPosition::TopRight,
            ToasterPosition::BottomLeft,
            ToasterPosition::BottomCenter,
            ToasterPosition::BottomRight,
        ];
        let classes: Vec<&str> = positions.iter().map(|p| p.class()).collect();
        let unique: std::collections::HashSet<&str> = classes.iter().copied().collect();
        assert_eq!(unique.len(), positions.len(), "positions must be distinct");
    }

    #[test]
    fn default_position_is_bottom_right() {
        assert_eq!(ToasterPosition::default(), ToasterPosition::BottomRight);
    }

    #[test]
    fn class_strings_contain_toast_container_prefix() {
        for p in [
            ToasterPosition::TopLeft,
            ToasterPosition::BottomRight,
        ] {
            assert!(p.class().starts_with("toast-container"));
        }
    }
}