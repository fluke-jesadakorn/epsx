use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[component]
pub fn ErrorView(title: Option<String>, description: Option<String>, retry_href: Option<String>) -> Element {
    rsx! {
        div { class: "error-view flex flex-col items-center text-center p-12",
            div { class: "error-view-icon", Icon { name: "info".to_string(), size: Some(48), class_name: Some("text-destructive".to_string()) } }
            h3 { class: "error-view-title", {title.unwrap_or_else(|| "Something went wrong".to_string())} }
            if let Some(d) = description {
                p { class: "error-view-description text-muted-foreground", "{d}" }
            }
            if let Some(h) = retry_href {
                a { class: "btn btn-primary mt-4", href: "{h}", "Try again" }
            }
        }
    }
}
