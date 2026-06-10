use super::icon::Icon;

use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct TabItem {
    pub key: String,
    pub label: String,
    pub icon: Option<String>,
}

#[component]
pub fn Tabs(
    items: Vec<TabItem>,
    active: String,
    on_select: EventHandler<String>,
    class_name: Option<String>,
) -> Element {
    let mut cls = "tabs".to_string();
    if let Some(c) = class_name { cls.push(' '); cls.push_str(&c); }
    rsx! {
        div { class: "{cls}", role: "tablist",
            for item in items {
                button {
                    class: if item.key == active { "tab tab-active" } else { "tab" },
                    role: "tab",
                    "aria-selected": item.key == active,
                    onclick: move |_| on_select.call(item.key.clone()),
                    if let Some(i) = &item.icon {
                        span { class: "tab-icon", Icon { name: i.clone(), size: Some(16) } }
                    }
                    span { "{item.label}" }
                }
            }
        }
    }
}
