use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[component]
pub fn EmptyState(title: String, description: Option<String>, icon: Option<String>, action: Option<String>, action_href: Option<String>) -> Element {
    rsx! {
        div { class: "empty-state flex flex-col items-center text-center p-12",
            if let Some(i) = icon {
                div { class: "empty-state-icon", Icon { name: i.clone(), size: Some(48), class_name: Some("text-muted-foreground".to_string()) } }
            }
            h3 { class: "empty-state-title", "{title}" }
            if let Some(d) = description {
                p { class: "empty-state-description text-muted-foreground", "{d}" }
            }
            if let (Some(a), Some(h)) = (action, action_href) {
                a { class: "btn btn-primary mt-4", href: "{h}", "{a}" }
            }
        }
    }
}
