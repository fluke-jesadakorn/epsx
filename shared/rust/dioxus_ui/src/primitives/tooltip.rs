use dioxus::prelude::*;

#[component]
pub fn Tooltip(text: String, children: Element) -> Element {
    rsx! {
        span { class: "tooltip-wrapper",
            {children}
            span { class: "tooltip-content", role: "tooltip", "{text}" }
        }
    }
}
