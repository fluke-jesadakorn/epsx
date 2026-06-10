use super::icon::Icon;

use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InputKind { Text, Email, Password, Number, Url, Tel, Search, Textarea, Select, Date, Time }

#[component]
pub fn Input(
    r#type: Option<InputKind>,
    name: Option<String>,
    value: Option<String>,
    placeholder: Option<String>,
    label: Option<String>,
    help: Option<String>,
    error: Option<String>,
    required: Option<bool>,
    disabled: Option<bool>,
    icon: Option<String>,
    rows: Option<u32>,
    class_name: Option<String>,
    id: Option<String>,
    oninput: Option<EventHandler<FormEvent>>,
    onchange: Option<EventHandler<FormEvent>>,
) -> Element {
    let r#type = r#type.unwrap_or(InputKind::Text);
    let type_str = match r#type {
        InputKind::Text => "text",
        InputKind::Email => "email",
        InputKind::Password => "password",
        InputKind::Number => "number",
        InputKind::Url => "url",
        InputKind::Tel => "tel",
        InputKind::Search => "search",
        InputKind::Date => "date",
        InputKind::Time => "time",
        _ => "text",
    };
    let required = required.unwrap_or(false);
    let disabled = disabled.unwrap_or(false);
    let mut input_cls = "input".to_string();
    if let Some(c) = &class_name { input_cls.push(' '); input_cls.push_str(c); }
    if error.is_some() { input_cls.push_str(" input-error"); }
    if icon.is_some() { input_cls.push_str(" input-with-icon"); }

    rsx! {
        div { class: "form-field",
            if let Some(l) = &label {
                label { class: "form-label", "{l}" if required { span { class: "text-red-500 ml-1", "*" } } }
            }
            if let Some(i) = &icon {
                div { class: "input-icon-wrap",
                    span { class: "input-icon", Icon { name: i.clone(), size: Some(16) } }
                }
            }
            match r#type {
                InputKind::Textarea => rsx! {
                    textarea {
                        class: "{input_cls}",
                        name: name.clone(),
                        id: id.clone(),
                        placeholder: placeholder.clone().unwrap_or_default(),
                        required: required,
                        disabled: disabled,
                        rows: rows.unwrap_or(4),
                        value: value.clone().unwrap_or_default(),
                        oninput: move |e| if let Some(h) = &oninput { h.call(e); },
                        onchange: move |e| if let Some(h) = &onchange { h.call(e); },
                    }
                },
                _ => rsx! {
                    input {
                        class: "{input_cls}",
                        r#type: "{type_str}",
                        name: name.clone(),
                        id: id.clone(),
                        placeholder: placeholder.clone().unwrap_or_default(),
                        required: required,
                        disabled: disabled,
                        value: value.clone().unwrap_or_default(),
                        oninput: move |e| if let Some(h) = &oninput { h.call(e); },
                        onchange: move |e| if let Some(h) = &onchange { h.call(e); },
                    }
                },
            }
            if let Some(h) = &help {
                p { class: "form-help text-xs text-muted-foreground mt-1", "{h}" }
            }
            if let Some(e) = &error {
                p { class: "form-error text-xs text-red-500 mt-1", "{e}" }
            }
        }
    }
}
