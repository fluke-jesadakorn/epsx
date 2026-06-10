//! Form primitives — typed form layout + field validation + error display.
//!
//! Mirrors `apps/frontend/components/ui/form.tsx` + react-hook-form usage in
//! the original. Caller owns the validation; the components just lay out
//! the field and render error/help text.

use dioxus::prelude::*;

/// Top-level `<form>` wrapper. Action is forwarded; method defaults to POST.
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

/// Standalone label, rendered as a `<label>`. Useful when the input is not
/// inside a `Field` (e.g. in an `InputGroup`).
#[component]
pub fn Label(
    #[props(default = None)] html_for: Option<String>,
    #[props(default = false)] required: bool,
    #[props(default = None)] class_name: Option<String>,
    children: Element,
) -> Element {
    let mut cls = "label".to_string();
    if let Some(c) = &class_name { cls.push(' '); cls.push_str(c); }
    rsx! {
        label { class: "{cls}", r#for: html_for.as_deref().unwrap_or(""),
            {children}
            if required { span { class: "label-required text-red-500 ml-1", "*" } }
        }
    }
}

/// Field wrapper — renders a label, the control (children), and help/error text.
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

/// Form section — a titled subsection inside a form. Renders a header
/// (title + description) plus a content area. Useful for long forms
/// (settings, onboarding) where you want to group related fields.
#[component]
pub fn FormSection(
    title: Option<String>,
    #[props(default = None)] description: Option<String>,
    #[props(default = None)] class_name: Option<String>,
    children: Element,
) -> Element {
    let mut cls = "form-section space-y-4".to_string();
    if let Some(c) = &class_name { cls.push(' '); cls.push_str(c); }
    rsx! {
        section { class: "{cls}",
            if title.is_some() || description.is_some() {
                div { class: "form-section-header",
                    if let Some(t) = &title {
                        h3 { class: "form-section-title text-base font-semibold", "{t}" }
                    }
                    if let Some(d) = &description {
                        p { class: "form-section-description text-sm text-muted-foreground", "{d}" }
                    }
                }
            }
            div { class: "form-section-body space-y-4", {children} }
        }
    }
}

/// Form row — horizontal layout helper. Renders a 1- or 2-column grid
/// for side-by-side fields. Children are placed inside a CSS grid.
#[component]
pub fn FormRow(
    #[props(default = 2)] columns: usize,
    #[props(default = None)] class_name: Option<String>,
    children: Element,
) -> Element {
    let cols = columns.max(1).to_string();
    let mut cls = format!("form-row grid grid-cols-1 gap-4 md:grid-cols-{cols}");
    if let Some(c) = &class_name { cls.push(' '); cls.push_str(c); }
    rsx! {
        div { class: "{cls}", {children} }
    }
}

/// Input group — label + control + button row. Renders a label, a control
/// (children) and an optional trailing button area. Used for inputs that
/// have an inline action (e.g. copy button, refresh button).
#[component]
pub fn InputGroup(
    label: Option<String>,
    #[props(default = None)] html_for: Option<String>,
    #[props(default = false)] required: bool,
    #[props(default = None)] help: Option<String>,
    #[props(default = None)] error: Option<String>,
    #[props(default = None)] class_name: Option<String>,
    children: Element,
) -> Element {
    let mut cls = "input-group space-y-2".to_string();
    if let Some(c) = &class_name { cls.push(' '); cls.push_str(c); }
    rsx! {
        div { class: "{cls}",
            if let Some(l) = &label {
                label { class: "input-group-label text-sm font-medium", r#for: html_for.as_deref().unwrap_or(""),
                    "{l}"
                    if required { span { class: "text-red-500 ml-1", "*" } }
                }
            }
            div { class: "input-group-control flex gap-2",
                {children}
            }
            if let Some(h) = &help {
                p { class: "input-group-help text-xs text-muted-foreground", "{h}" }
            }
            if let Some(e) = &error {
                p { class: "input-group-error text-xs text-red-500", "{e}" }
            }
        }
    }
}

/// RadioGroup — a vertical stack of radio rows. Each option is rendered as
/// a Checkbox-style row (radio dot + label). Selection is fully controlled
/// via the `value` + `onchange` pair.
#[component]
pub fn RadioGroup(
    name: String,
    options: Vec<(String, String)>,
    /// Currently-selected value. None = no selection.
    #[props(default = None)] value: Option<String>,
    /// Label rendered above the group.
    #[props(default = None)] label: Option<String>,
    /// Inline help text.
    #[props(default = None)] help: Option<String>,
    /// Error message — also flips `aria-invalid` on the group.
    #[props(default = None)] error: Option<String>,
    #[props(default = false)] required: bool,
    #[props(default = false)] disabled: bool,
    #[props(default = None)] class_name: Option<String>,
    onchange: Option<EventHandler<FormEvent>>,
) -> Element {
    let group_id = format!("radio-{}", name);
    let mut cls = "radio-group space-y-2".to_string();
    if let Some(c) = &class_name { cls.push(' '); cls.push_str(c); }
    rsx! {
        div { class: "{cls}",
            role: "radiogroup",
            "aria-labelledby": format!("{}-label", group_id),
            "aria-required": required.to_string(),
            "aria-invalid": error.is_some().to_string(),
            if let Some(l) = &label {
                div { id: format!("{}-label", group_id), class: "radio-group-label text-sm font-medium",
                    "{l}"
                    if required { span { class: "text-red-500 ml-1", "*" } }
                }
            }
            for (val, lbl) in options.iter() {
                {
                    let val_for_id = val.clone();
                    let val_for_value = val.clone();
                    let is_selected = value.as_deref() == Some(val.as_str());
                    rsx! {
                        label {
                            class: if is_selected { "radio-row selected flex items-center gap-2" } else { "radio-row flex items-center gap-2" },
                            r#for: format!("{}-{}", group_id, val_for_id),
                            input {
                                id: format!("{}-{}", group_id, val_for_id),
                                r#type: "radio",
                                name: "{name}",
                                value: "{val_for_value}",
                                checked: is_selected,
                                disabled: disabled,
                                onchange: move |e| if let Some(h) = &onchange { h.call(e); },
                            }
                            span { class: "radio-row-label", "{lbl}" }
                        }
                    }
                }
            }
            if let Some(h) = &help {
                p { class: "radio-group-help text-xs text-muted-foreground", "{h}" }
            }
            if let Some(e) = &error {
                p { class: "radio-group-error text-xs text-red-500", "{e}" }
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
