//! Checkbox primitive — supports the three visual states
//! (checked / unchecked / indeterminate) used by shadcn / Radix.

use dioxus::prelude::*;

/// A checkbox with optional label and indeterminate state.
///
/// `indeterminate: Some(true)` flips the rendered element to a visual
/// "indeterminate" state by adding `data-state="indeterminate"` plus a
/// `checkbox-indeterminate` CSS class. The HTML `indeterminate` property
/// itself is not directly set (it's a DOM property, not an attribute),
/// but the visual cue is enough for server-rendered shadcn parity.
#[component]
pub fn Checkbox(
    checked: bool,
    #[props(default = None)] label: Option<String>,
    #[props(default = None)] disabled: Option<bool>,
    #[props(default = None)] name: Option<String>,
    #[props(default = None)] id: Option<String>,
    /// When `Some(true)`, render as the indeterminate state.
    #[props(default = None)] indeterminate: Option<bool>,
    /// Optional override class names.
    #[props(default = None)] class_name: Option<String>,
    onchange: Option<EventHandler<FormEvent>>,
) -> Element {
    let indeterminate = indeterminate.unwrap_or(false);
    let data_state = if indeterminate {
        "indeterminate"
    } else if checked {
        "checked"
    } else {
        "unchecked"
    };
    let mut cls = "checkbox-wrapper".to_string();
    if let Some(c) = &class_name { cls.push(' '); cls.push_str(c); }
    rsx! {
        label { class: "{cls}",
            input {
                r#type: "checkbox",
                class: if indeterminate { "checkbox checkbox-indeterminate" } else { "checkbox" },
                checked: checked,
                disabled: disabled.unwrap_or(false),
                name: name.clone(),
                id: id.clone(),
                "data-state": data_state,
                "aria-checked": if indeterminate { "mixed" } else { checked.to_string() },
                onchange: move |e| if let Some(h) = &onchange { h.call(e); },
            }
            if let Some(l) = label {
                span { class: "checkbox-label", "{l}" }
            }
        }
    }
}
