//! Form primitives — typed form layout + field validation + error display.
//!
//! Mirrors `apps/frontend/components/ui/form.tsx` + react-hook-form usage in
//! the original. Caller owns the validation; the components just lay out
//! the field and render error/help text.

use dioxus::prelude::*;

#[component]
pub fn Form(
    on_submit: Option<EventHandler<FormEvent>>,
    #[props(default = "post".to_string())] method: String,
    #[props(default = None)] action: Option<String>,
    #[props(default = None)] class_name: Option<String>,
    children: Element,
) -> Element {
    let cls = format!("form{}", class_name.as_ref().map(|c| format!(" {}", c)).unwrap_or_default());
    rsx! {
        form {
            class: "{cls}",
            method: "{method}",
            action: action.as_deref().unwrap_or(""),
            onsubmit: move |e| if let Some(h) = &on_submit { h.call(e); },
            {children}
        }
    }
}

#[component]
pub fn Field(
    label: Option<String>,
    #[props(default = None)] html_for: Option<String>,
    #[props(default = None)] help: Option<String>,
    #[props(default = None)] error: Option<String>,
    #[props(default = false)] required: bool,
    children: Element,
) -> Element {
    rsx! {
        div { class: "field",
            if let Some(l) = &label {
                label { class: "field-label", r#for: html_for.as_deref().unwrap_or(""),
                    "{l}"
                    if required { span { class: "field-required", " *" } }
                }
            }
            {children}
            if let Some(h) = &help {
                p { class: "field-help text-sm text-muted-foreground", "{h}" }
            }
            if let Some(e) = &error {
                p { class: "field-error text-sm text-danger", "{e}" }
            }
        }
    }
}

#[component]
pub fn FormField(
    name: String,
    label: Option<String>,
    #[props(default = None)] placeholder: Option<String>,
    #[props(default = None)] value: Option<String>,
    #[props(default = None)] help: Option<String>,
    #[props(default = None)] error: Option<String>,
    #[props(default = false)] required: bool,
    #[props(default = "text".to_string())] kind: String,
    #[props(default = None)] autocomplete: Option<String>,
    #[props(default = false)] disabled: bool,
    #[props(default = None)] pattern: Option<String>,
    #[props(default = None)] min: Option<String>,
    #[props(default = None)] max: Option<String>,
    #[props(default = None)] maxlength: Option<String>,
    #[props(default = None)] oninput: Option<EventHandler<FormEvent>>,
) -> Element {
    let id = format!("field-{}", name);
    rsx! {
        Field { label: label.clone(), html_for: Some(id.clone()), help: help.clone(), error: error.clone(), required: required,
            input {
                class: "input",
                id: id.clone(),
                name: "{name}",
                r#type: "{kind}",
                placeholder: placeholder.as_deref().unwrap_or(""),
                value: value.as_deref().unwrap_or(""),
                disabled: disabled,
                required: required,
                autocomplete: autocomplete.as_deref().unwrap_or("off"),
                pattern: pattern.as_deref().unwrap_or(""),
                min: min.as_deref().unwrap_or(""),
                max: max.as_deref().unwrap_or(""),
                maxlength: maxlength.as_deref().unwrap_or(""),
                oninput: move |e| if let Some(h) = &oninput { h.call(e); },
            }
        }
    }
}

#[component]
pub fn Textarea(
    name: String,
    label: Option<String>,
    #[props(default = None)] placeholder: Option<String>,
    #[props(default = None)] value: Option<String>,
    #[props(default = None)] help: Option<String>,
    #[props(default = None)] error: Option<String>,
    #[props(default = false)] required: bool,
    #[props(default = 4)] rows: usize,
    #[props(default = None)] oninput: Option<EventHandler<FormEvent>>,
) -> Element {
    let id = format!("field-{}", name);
    rsx! {
        Field { label: label.clone(), html_for: Some(id.clone()), help: help.clone(), error: error.clone(), required: required,
            textarea {
                class: "input",
                id: id.clone(),
                name: "{name}",
                placeholder: placeholder.as_deref().unwrap_or(""),
                rows: "{rows}",
                required: required,
                value: value.as_deref().unwrap_or(""),
                oninput: move |e| if let Some(h) = &oninput { h.call(e); },
            }
        }
    }
}

#[component]
pub fn SelectField(
    name: String,
    label: Option<String>,
    options: Vec<(String, String)>,
    #[props(default = None)] value: Option<String>,
    #[props(default = None)] help: Option<String>,
    #[props(default = None)] error: Option<String>,
    #[props(default = false)] required: bool,
    #[props(default = None)] placeholder: Option<String>,
    #[props(default = None)] onchange: Option<EventHandler<FormEvent>>,
) -> Element {
    let id = format!("field-{}", name);
    let value_str = value.clone().unwrap_or_default();
    rsx! {
        Field { label: label.clone(), html_for: Some(id.clone()), help: help.clone(), error: error.clone(), required: required,
            select {
                class: "input",
                id: id.clone(),
                name: "{name}",
                required: required,
                onchange: move |e| if let Some(h) = &onchange { h.call(e); },
                if let Some(ph) = &placeholder {
                    option { value: "", disabled: true, selected: value_str.is_empty(), "{ph}" }
                }
                for (val, lbl) in &options {
                    option { value: "{val}", selected: *val == value_str, "{lbl}" }
                }
            }
        }
    }
}

#[component]
pub fn CheckboxField(
    name: String,
    label: String,
    #[props(default = false)] checked: bool,
    #[props(default = None)] help: Option<String>,
    #[props(default = None)] onchange: Option<EventHandler<FormEvent>>,
) -> Element {
    let id = format!("field-{}", name);
    rsx! {
        div { class: "field field-checkbox",
            label { class: "flex items-center gap-2", r#for: id.clone(),
                input {
                    class: "checkbox",
                    id: id.clone(),
                    name: "{name}",
                    r#type: "checkbox",
                    checked: checked,
                    onchange: move |e| if let Some(h) = &onchange { h.call(e); },
                }
                span { "{label}" }
            }
            if let Some(h) = &help {
                p { class: "field-help text-sm text-muted-foreground", "{h}" }
            }
        }
    }
}

#[component]
pub fn FormActions(children: Element) -> Element {
    rsx! { div { class: "form-actions flex gap-2 justify-end", {children} } }
}
