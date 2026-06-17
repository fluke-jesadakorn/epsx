use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[component]
pub fn Spinner(size: Option<String>) -> Element {
    let s = size.unwrap_or_else(|| "md".to_string());
    rsx! { div { class: "spinner spinner-{s}", role: "status", "aria-label": "Loading" } }
}

#[component]
pub fn LoadingBlock(text: Option<String>) -> Element {
    rsx! {
        div { class: "loading-block flex flex-col items-center justify-center p-8",
            Spinner {}
            if let Some(t) = text {
                p { class: "text-muted-foreground mt-2", "{t}" }
            }
        }
    }
}
