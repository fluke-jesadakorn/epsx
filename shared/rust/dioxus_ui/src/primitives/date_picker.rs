//! DatePicker — server-renderable date / date-time / date-range pickers.
//!
//! Server side: emits text inputs `type=date` and `type=time` (HTML5 native)
//! which are perfectly accessible. Client hydration can swap in a richer
//! picker.

use dioxus::prelude::*;

/// Single date picker. Renders a `<input type="date">`.
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
    #[props(default = None)] id: Option<String>,
) -> Element {
    let id = id.unwrap_or_else(|| format!("field-{}", name));
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
                "aria-invalid": if error.is_some() { "true" } else { "false" },
            }
        }
    }
}

/// Date range picker. Renders two `DatePicker` side-by-side.
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
                DatePicker { name: name_start.clone(), value: value_start, required: required, label: None, min: None, max: None, help: None, error: None, id: Some(format!("field-{}-start", name_start)) }
                DatePicker { name: name_end.clone(), value: value_end, required: required, label: None, min: None, max: None, help: None, error: None, id: Some(format!("field-{}-end", name_end)) }
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

/// DateTime picker. Combines a `<input type="date">` and a `<input type="time">`.
/// Submits two fields: `<name>_date` and `<name>_time`, OR if `combined_name` is
/// provided, a single hidden `<input name="combined_name" value="YYYY-MM-DDTHH:MM">`
/// is emitted as well for convenience.
#[component]
pub fn DateTimePicker(
    name: String,
    label: Option<String>,
    /// Combined datetime value in `YYYY-MM-DDTHH:MM` format. When set, the
    /// date and time fields are pre-filled.
    #[props(default = None)] value: Option<String>,
    #[props(default = None)] min: Option<String>,
    #[props(default = None)] max: Option<String>,
    #[props(default = None)] help: Option<String>,
    #[props(default = None)] error: Option<String>,
    #[props(default = false)] required: bool,
    /// If set, emit a hidden input with this name and a `YYYY-MM-DDTHH:MM` value.
    #[props(default = None)] combined_name: Option<String>,
) -> Element {
    let id_base = format!("field-{}", name);
    // Try to split the combined value into date + time parts.
    let (date_value, time_value) = match value.as_deref() {
        Some(v) if v.len() >= 16 && v.contains('T') => {
            let parts: Vec<&str> = v.splitn(2, 'T').collect();
            let date = parts.first().copied().unwrap_or("").to_string();
            let time = parts.get(1).copied().unwrap_or("").get(..5).unwrap_or("").to_string();
            (Some(date), Some(time))
        }
        Some(v) if v.len() >= 10 => (Some(v[..10].to_string()), None),
        _ => (None, None),
    };
    let combined_value = format!(
        "{}T{}",
        date_value.clone().unwrap_or_default(),
        time_value.clone().unwrap_or_default(),
    );

    rsx! {
        div { class: "field datetime-picker",
            if let Some(l) = &label {
                label { class: "field-label", "{l}" if required { span { class: "text-red-500 ml-1", "*" } } }
            }
            div { class: "flex gap-2",
                input {
                    class: "input",
                    id: format!("{}-date", id_base),
                    name: format!("{}_date", name),
                    r#type: "date",
                    value: date_value.clone().unwrap_or_default(),
                    min: min.as_deref().unwrap_or(""),
                    max: max.as_deref().unwrap_or(""),
                    required: required,
                    "aria-invalid": if error.is_some() { "true" } else { "false" },
                }
                input {
                    class: "input",
                    id: format!("{}-time", id_base),
                    name: format!("{}_time", name),
                    r#type: "time",
                    value: time_value.clone().unwrap_or_default(),
                    required: required,
                    "aria-invalid": if error.is_some() { "true" } else { "false" },
                }
            }
            if let Some(cn) = combined_name.clone() {
                input { r#type: "hidden", name: "{cn}", value: "{combined_value}" }
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
