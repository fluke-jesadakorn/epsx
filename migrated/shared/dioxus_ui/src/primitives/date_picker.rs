//! DatePicker — server-renderable date / date-range picker.
//!
//! Server side: emits a text input `type=date` (HTML5 native) which is
//! perfectly accessible. Client hydration can swap in a richer picker.

use dioxus::prelude::*;

#[component]
pub fn DatePicker(
    name: String,
    label: Option<String>,
    #[props(default = None)] value: Option<String>,
    #[props(default = None)] min: Option<String>,
    #[props(default = None)] max: Option<String>,
    #[props(default = None)] help: Option<String>,
    #[props(default = None)] error: Option<String>,
    #[props(default = false)] required: bool,
) -> Element {
    let id = format!("field-{}", name);
    rsx! {
        crate::primitives::form::Field { label: label.clone(), html_for: Some(id.clone()), help: help.clone(), error: error.clone(), required: required,
            input {
                class: "input",
                id: id.clone(),
                name: "{name}",
                r#type: "date",
                value: value.as_deref().unwrap_or(""),
                min: min.as_deref().unwrap_or(""),
                max: max.as_deref().unwrap_or(""),
                required: required,
            }
        }
    }
}

#[component]
pub fn DateRangePicker(
    name_start: String,
    name_end: String,
    label: Option<String>,
    #[props(default = None)] value_start: Option<String>,
    #[props(default = None)] value_end: Option<String>,
    #[props(default = false)] required: bool,
    #[props(default = None)] help: Option<String>,
    #[props(default = None)] error: Option<String>,
) -> Element {
    rsx! {
        div { class: "field date-range",
            if let Some(l) = &label {
                label { class: "field-label", "{l}" }
            }
            div { class: "flex gap-2",
                DatePicker { name: name_start.clone(), value: value_start, required: required, label: None, min: None, max: None, help: None, error: None }
                DatePicker { name: name_end.clone(), value: value_end, required: required, label: None, min: None, max: None, help: None, error: None }
            }
            if let Some(h) = &help {
                p { class: "field-help text-sm text-muted-foreground", "{h}" }
            }
            if let Some(e) = &error {
                p { class: "field-error text-sm text-danger", "{e}" }
            }
        }
    }
}
