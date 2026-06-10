//! Select primitive — single + multi select, optionally labelled.
//!
//! Mirrors `shared/components/ui/select.tsx` (152 lines, Radix Select) and
//! the multi-select pattern used in the original Next.js apps.

use dioxus::prelude::*;

/// One option in a select / multi-select.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectOption { pub value: String, pub label: String }

/// Single-value select. Native `<select>` for now (server-renderable);
/// a richer popover-driven implementation can be layered on later.
#[component]
pub fn Select(
    name: Option<String>,
    value: Option<String>,
    options: Vec<SelectOption>,
    placeholder: Option<String>,
    label: Option<String>,
    disabled: Option<bool>,
    #[props(default = None)] help: Option<String>,
    #[props(default = None)] error: Option<String>,
    #[props(default = false)] required: bool,
    #[props(default = None)] id: Option<String>,
    onchange: Option<EventHandler<FormEvent>>,
) -> Element {
    let id_str = id.clone().unwrap_or_else(|| match &name { Some(n) => format!("field-{}", n), None => "field-select".to_string() });
    let aria_invalid = if error.is_some() { "true" } else { "false" }.to_string();
    rsx! {
        div { class: "form-field",
            if let Some(l) = label {
                label { class: "form-label", r#for: id_str.clone(), "{l}" }
            }
            select {
                class: "input",
                id: id_str.clone(),
                name: name.clone(),
                disabled: disabled.unwrap_or(false),
                required: required,
                "aria-invalid": "{aria_invalid}",
                onchange: move |e| if let Some(h) = &onchange { h.call(e); },
                if let Some(p) = placeholder {
                    option { value: "", "{p}" }
                }
                for opt in options {
                    option { value: "{opt.value}", selected: value.as_deref() == Some(opt.value.as_str()), "{opt.label}" }
                }
            }
            if let Some(h) = help {
                p { class: "form-help text-xs text-muted-foreground mt-1", "{h}" }
            }
            if let Some(e) = error {
                p { class: "form-error text-xs text-red-500 mt-1", "{e}" }
            }
        }
    }
}

/// Multi-select — renders a chip list of the currently-selected values plus
/// a dropdown picker to add or remove items. The value is fully controlled
/// via `value: Vec<String>` + `onchange: EventHandler<Vec<String>>`.
///
/// Mirrors the multi-select patterns used in the admin-frontend. Server-
/// renderable (chips render with no JS) and hydrates to a richer experience
/// when `web` feature is enabled.
#[component]
pub fn MultiSelect(
    name: String,
    options: Vec<(String, String)>,
    /// Currently-selected values.
    value: Vec<String>,
    #[props(default = None)] label: Option<String>,
    #[props(default = None)] placeholder: Option<String>,
    #[props(default = None)] help: Option<String>,
    #[props(default = None)] error: Option<String>,
    #[props(default = false)] required: bool,
    #[props(default = false)] disabled: bool,
    /// Maximum number of selections allowed. None = unlimited.
    #[props(default = None)] max: Option<usize>,
    onchange: Option<EventHandler<Vec<String>>>,
) -> Element {
    // Use a Signal so the closures can read the latest value without moving
    // the original `value` parameter (which would fail to compile because
    // it would be captured by multiple FnMut closures).
    let value_signal = use_signal(|| value.clone());
    let mut open = use_signal(|| false);
    let options_for_render = options.clone();
    let value_for_render = value.clone();
    let placeholder_str = placeholder.unwrap_or_else(|| "Add…".to_string());
    let aria_invalid = if error.is_some() { "true" } else { "false" }.to_string();
    let error_for_help = error.clone();
    let help_for_help = help.clone();

    rsx! {
        div { class: "form-field multiselect",
            if let Some(l) = label.clone() {
                label { class: "form-label", "{l}" if required { span { class: "text-red-500 ml-1", "*" } } }
            }
            div {
                class: "multiselect-control flex flex-wrap items-center gap-2 input",
                role: "listbox",
                "aria-multiselectable": "true",
                "aria-invalid": "{aria_invalid}",
                for v in value_for_render.iter() {
                    {
                        let v_for_chip = v.clone();
                        let v_for_remove = v.clone();
                        let label_for_chip = options_for_render.iter()
                            .find(|(val, _)| val == &v_for_chip)
                            .map(|(_, l)| l.clone())
                            .unwrap_or_else(|| v_for_chip.clone());
                        let value_signal_for_remove = value_signal.clone();
                        rsx! {
                            span { class: "multiselect-chip badge badge-primary flex items-center gap-1", role: "presentation",
                                "{label_for_chip}"
                                button {
                                    r#type: "button",
                                    class: "multiselect-chip-remove",
                                    disabled: disabled,
                                    "aria-label": format!("Remove {}", label_for_chip),
                                    onclick: move |_| {
                                        if disabled { return; }
                                        let next: Vec<String> = value_signal_for_remove.read().iter()
                                            .filter(|x| x.as_str() != v_for_remove.as_str())
                                            .cloned()
                                            .collect();
                                        if let Some(h) = &onchange { h.call(next); }
                                    },
                                    "×"
                                }
                            }
                        }
                    }
                }
                button {
                    r#type: "button",
                    class: "multiselect-trigger",
                    disabled: disabled || max.map_or(false, |m| value_signal.read().len() >= m),
                    "aria-haspopup": "listbox",
                    "aria-expanded": open.read().to_string(),
                    onclick: move |_| {
                        if disabled { return; }
                        let cur = *open.read();
                        open.set(!cur);
                    },
                    "{placeholder_str}"
                }
            }
            if *open.read() {
                ul { class: "multiselect-menu", role: "listbox",
                    for (val, lbl) in options_for_render.iter() {
                        {
                            let val_for_toggle = val.clone();
                            let val_for_click = val_for_toggle.clone();
                            let is_selected = value_signal.read().contains(&val_for_toggle);
                            let at_max = max.map_or(false, |m| value_signal.read().len() >= m);
                            let disabled_for_option = disabled || (at_max && !is_selected);
                            let value_signal_for_toggle = value_signal.clone();
                            rsx! {
                                li {
                                    role: "option",
                                    "aria-selected": is_selected.to_string(),
                                    class: if is_selected { "multiselect-option selected" } else { "multiselect-option" },
                                    onclick: move |_| {
                                        if disabled_for_option { return; }
                                        let mut next: Vec<String> = value_signal_for_toggle.read().iter().cloned().collect();
                                        if next.contains(&val_for_click) {
                                            next.retain(|x| x != &val_for_click);
                                        } else {
                                            next.push(val_for_click.clone());
                                        }
                                        if let Some(h) = &onchange { h.call(next); }
                                    },
                                    "{lbl}"
                                }
                            }
                        }
                    }
                }
            }
            // Native mirror so the form submits selected values as repeated form fields.
            for v in value_for_render.iter() {
                input { r#type: "hidden", name: "{name}", value: "{v}" }
            }
            if let Some(h) = help_for_help {
                p { class: "form-help text-xs text-muted-foreground mt-1", "{h}" }
            }
            if let Some(e) = error_for_help {
                p { class: "form-error text-xs text-red-500 mt-1", "{e}" }
            }
        }
    }
}
