use dioxus::prelude::*;

#[component]
pub fn Checkbox(checked: bool, label: Option<String>, disabled: Option<bool>, name: Option<String>, id: Option<String>, onchange: Option<EventHandler<FormEvent>>) -> Element {
    rsx! {
        label { class: "checkbox-wrapper",
            input {
                r#type: "checkbox",
                class: "checkbox",
                checked: checked,
                disabled: disabled.unwrap_or(false),
                name: name.clone(),
                id: id.clone(),
                onchange: move |e| if let Some(h) = &onchange { h.call(e); },
            }
            if let Some(l) = label {
                span { class: "checkbox-label", "{l}" }
            }
        }
    }
}
