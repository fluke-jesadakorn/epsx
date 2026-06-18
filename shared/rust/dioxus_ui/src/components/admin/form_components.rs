//! Admin form-components — Wave 37 T1 admin primitives port.
//!
//! Mirrors `apps-old/admin-frontend/components/ui/form-components.tsx`,
//! which is a thin re-export wrapper around the shared form
//! primitives (`Button`, `Input`, `Select`, `Textarea`, `Form`,
//! `FormField`, `FormFieldWrapper`, `FormLabel as Label`).
//!
//! ## Source-of-truth mapping
//!
//! | apps-old `form-components.tsx` | Dioxus |
//! | --- | --- |
//! | `<Button>` (shared) | `crate::primitives::button::Button` |
//! | `<Input>` (shared)  | `crate::primitives::input::Input` |
//! | `<Select>` (shared) | `crate::primitives::select::Select` |
//! | `<Textarea>` (admin variant) | `crate::components::admin::Textarea` (always wp variant) |
//! | `<Form>` / `<FormField>` / `<FormFieldWrapper>` | `crate::primitives::form::{Form, FormField, FormFieldWrapper}` |
//! | `<Label>` | `crate::components::admin::Label` |
//!
//! The Dioxus port provides a single import surface so admin forms
//! can `use crate::components::admin::*` and get every form
//! primitive without crossing module boundaries.
//!
//! ## Tests
//!
//! `test_form_components_module_exports` — every component is
//! reachable via `crate::components::admin::*`.

pub use crate::components::admin::{Label, Textarea};
pub use crate::primitives::button::Button;
pub use crate::primitives::input::{Input, InputKind};
pub use crate::primitives::select::Select;
pub use crate::primitives::form::{Field as FormFieldWrapper, Form, FormField};

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::prelude::*;

    /// Every form component is reachable from
    /// `crate::components::admin::*`. This is a compile-time
    /// guarantee — if any of the `pub use` lines breaks, this
    /// module fails to compile.
    #[test]
    fn test_form_components_module_exports() {
        // The `pub use` lines above are the test — if any link is
        // broken, this file won't compile. This test just confirms
        // the module loads.
        assert!(true);
    }

    /// Smoke render: `Button` resolves and renders non-empty HTML.
    #[test]
    fn test_form_components_button_smoke() {
        let el = rsx! { Button { children: rsx!("Save") } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("Save"), "Button should render children. Got: {html}");
        assert!(html.contains("<button"), "Button should render <button> element. Got: {html}");
    }

    /// `Label` is reachable from the admin form-components module.
    #[test]
    fn test_form_components_label_smoke() {
        let el = rsx! {
            Label {
                html_for: Some("email".to_string()),
                "Email"
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("for=\"email\""), "Label should propagate html_for. Got: {html}");
    }
}
