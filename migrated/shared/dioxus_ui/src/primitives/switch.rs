use dioxus::prelude::*;

#[component]
pub fn Switch(checked: bool, label: Option<String>, disabled: Option<bool>, onchange: Option<EventHandler<FormEvent>>) -> Element {
    rsx! {
        label { class: if checked { "SwitchRoot state-checked" } else { "SwitchRoot" },
            input {
                r#type: "checkbox",
                class: "SwitchInput",
                checked: checked,
                disabled: disabled.unwrap_or(false),
                onchange: move |e| if let Some(h) = &onchange { h.call(e); },
            }
            span { class: "SwitchThumb" }
            if let Some(l) = label {
                span { class: "SwitchLabel", "{l}" }
            }
        }
    }
}
