//! Admin `Label` component — Wave 37 T1 admin primitives port.
//!
//! Mirrors `apps-old/admin-frontend/components/ui/label.tsx`, which is a
//! direct re-export from `@/shared/components/ui/label`. In the Dioxus
//! port we re-export the existing `primitives::form::Textarea`-style
//! form label primitive so admin pages can do
//! `use crate::components::admin::Label` without pulling from the
//! generic `primitives` namespace.
//!
//! The Next.js `label.tsx` exports `* from '@/shared/components/ui/label'`.
//! Since the shared path doesn't exist in this repo (the production
//! app's `@/shared/...` is an alias not present here), we expose the
//! Dioxus form `<Label>` primitive under the `admin::Label` name.
//!
//! ## Source-of-truth mapping
//!
//! | apps-old `label.tsx` | Dioxus |
//! | --- | --- |
//! | `<Label>` (Radix UI label primitive) | `crate::primitives::form::Label` |
//!
//! ## Tests
//!
//! `test_label_renders_for_id` — the rendered HTML contains the
//! `for="..."` attribute wiring the label to its target input.
//! `test_label_propagates_class_name` — the `class_name` slot is
//! applied to the `<label>` element.

use dioxus::prelude::*;

/// Dioxus form `<label>` element. Wraps `primitives::form::Label`
/// so admin pages can `use crate::components::admin::Label` and
/// avoid importing the generic primitives module.
///
/// `html_for` matches Radix's `<Label htmlFor="…">` prop and is
/// rendered as the standard `<label for="…">` HTML attribute.
#[component]
pub fn Label(
    html_for: Option<String>,
    class_name: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        label {
            r#for: html_for.clone().unwrap_or_default(),
            class: class_name.unwrap_or_default(),
            {children}
        }
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// The `html_for` slot propagates to the standard `<label for="...">`
    /// attribute. This is the contract admin forms depend on for
    /// click-to-focus semantics.
    #[test]
    fn test_label_renders_for_id() {
        let el = rsx! {
            Label {
                html_for: Some("email-input".to_string()),
                "Email"
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("<label"), "label should render <label> element");
        assert!(
            html.contains("for=\"email-input\""),
            "label should propagate `html_for` to `for` attribute. Got: {html}"
        );
        assert!(html.contains("Email"), "label should render its children");
    }

    /// The `class_name` slot is applied to the rendered `<label>`.
    #[test]
    fn test_label_propagates_class_name() {
        let el = rsx! {
            Label {
                class_name: Some("text-sm font-medium".to_string()),
                "Email"
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("class=\"text-sm font-medium\""),
            "label should apply `class_name`. Got: {html}"
        );
    }

    /// `Label` renders without a `html_for` value (for use cases where
    /// the wrapping input is implicit / the label is decorative).
    #[test]
    fn test_label_renders_without_html_for() {
        let el = rsx! { Label { "Display only" } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("<label"), "label should render <label> element");
        assert!(html.contains("Display only"), "label should render children");
    }
}
