use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[component]
pub fn FilterBar(children: Element) -> Element { rsx! { div { class: "filter-bar", {children} } } }

#[component]
pub fn SearchInput(value: String, placeholder: Option<String>, oninput: Option<EventHandler<FormEvent>>) -> Element {
    rsx! {
        div { class: "search-input-wrap",
            span { class: "search-input-icon", Icon { name: "search".to_string(), size: Some(16) } }
            input {
                class: "input search-input",
                r#type: "search",
                value: "{value}",
                placeholder: placeholder.unwrap_or_else(|| "Search...".to_string()),
                oninput: move |e| if let Some(h) = &oninput { h.call(e); },
            }
        }
    }
}
