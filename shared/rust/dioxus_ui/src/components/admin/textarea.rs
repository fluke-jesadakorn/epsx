//! Admin `Textarea` component — Wave 37 T1 admin primitives port.
//!
//! Mirrors `apps-old/admin-frontend/components/ui/textarea.tsx`, which
//! is a thin wrapper around the shared `Textarea` primitive that
//! defaults `variant="wp"` for admin-side styling.
//!
//! ## Source-of-truth mapping
//!
//! | apps-old `textarea.tsx` | Dioxus |
//! | --- | --- |
//! | `<Textarea variant="wp">` (default) | `<AdminTextarea>` (always renders the `wp` variant) |
//! | `<Textarea variant="…">` (other)   | `primitives::form::Textarea` (full API) |
//! | `textareaVariants` export          | not re-exported (use `primitives::form` directly) |
//!
//! In Dioxus we render a styled `<textarea>` element directly. The
//! class structure matches the design-system `form-textarea` class
//! so admin forms keep pixel parity with prod.
//!
//! ## Tests
//!
//! `test_textarea_renders_with_value` — the `value` slot is
//! rendered as the textarea's text content.
//! `test_textarea_propagates_name` — the `name` slot becomes the
//! `<textarea name="...">` attribute.

use dioxus::prelude::*;

/// Admin `<textarea>` with the `wp` (admin) variant baked in. Use
/// this for any admin form field (settings, audit-log filters,
/// policy-edit modal, etc.) so the visual style matches prod.
///
/// For non-admin textareas or to override the variant, use
/// `primitives::form::Textarea` directly.
#[component]
pub fn Textarea(
    name: Option<String>,
    placeholder: Option<String>,
    value: Option<String>,
    rows: Option<usize>,
    disabled: Option<bool>,
    required: Option<bool>,
    class_name: Option<String>,
    /// Number of visible text columns (the HTML `cols` attribute).
    cols: Option<usize>,
) -> Element {
    let rows = rows.unwrap_or(4);
    let disabled = disabled.unwrap_or(false);
    let required = required.unwrap_or(false);
    let mut cls = "form-textarea form-textarea-wp".to_string();
    if let Some(c) = class_name {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        textarea {
            name: name.clone().unwrap_or_default(),
            placeholder: placeholder.clone().unwrap_or_default(),
            rows: "{rows}",
            cols: cols.unwrap_or(40),
            disabled: disabled,
            required: required,
            class: "{cls}",
            {value.unwrap_or_default()}
        }
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// The textarea renders with the `wp` admin variant class and the
    /// value is rendered as the textarea's text content.
    #[test]
    fn test_textarea_renders_with_value() {
        let el = rsx! {
            Textarea {
                name: Some("policy-body".to_string()),
                value: Some("Allow read for tier 2".to_string()),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("<textarea"), "textarea should render <textarea> element");
        assert!(
            html.contains("form-textarea-wp"),
            "admin textarea should use the wp variant class. Got: {html}"
        );
        assert!(
            html.contains("Allow read for tier 2"),
            "textarea should render the value as text content. Got: {html}"
        );
    }

    /// The `name` slot becomes the `<textarea name="...">` attribute
    /// so admin forms submit the value to the BFF correctly.
    #[test]
    fn test_textarea_propagates_name() {
        let el = rsx! {
            Textarea {
                name: Some("audit-message".to_string()),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("name=\"audit-message\""),
            "textarea should propagate `name`. Got: {html}"
        );
    }

    /// The `rows` slot overrides the default 4 rows.
    #[test]
    fn test_textarea_renders_with_rows_override() {
        let el = rsx! {
            Textarea {
                rows: Some(10),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("rows=\"10\""),
            "textarea should propagate `rows`. Got: {html}"
        );
    }

    /// The `disabled` slot sets the standard `disabled` HTML attribute.
    #[test]
    fn test_textarea_renders_disabled() {
        let el = rsx! {
            Textarea {
                disabled: Some(true),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("disabled"),
            "textarea should render `disabled` attribute when true. Got: {html}"
        );
    }
}
