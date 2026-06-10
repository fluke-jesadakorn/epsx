use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct SelectOption { pub value: String, pub label: String }

#[component]
pub fn Select(name: Option<String>, value: Option<String>, options: Vec<SelectOption>, placeholder: Option<String>, label: Option<String>, disabled: Option<bool>, onchange: Option<EventHandler<FormEvent>>) -> Element {
    rsx! {
        div { class: "form-field",
            if let Some(l) = label {
                label { class: "form-label", "{l}" }
            }
            select {
                class: "input",
                name: name.clone(),
                disabled: disabled.unwrap_or(false),
                onchange: move |e| if let Some(h) = &onchange { h.call(e); },
                if let Some(p) = placeholder {
                    option { value: "", "{p}" }
                }
                for opt in options {
                    option { value: "{opt.value}", selected: value.as_deref() == Some(opt.value.as_str()), "{opt.label}" }
                }
            }
        }
    }
}
