use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[component]
pub fn AccessDenied(reason: Option<String>, required_permissions: Option<Vec<String>>) -> Element {
    rsx! {
        div { class: "access-denied", role: "alert",
            div { class: "access-denied-icon", Icon { name: "info".to_string(), size: Some(64), class_name: Some("text-destructive".to_string()) } }
            h1 { class: "access-denied-title", "Access Denied" }
            if let Some(r) = reason {
                p { class: "access-denied-reason", "{r}" }
            }
            if let Some(perms) = required_permissions {
                div { class: "access-denied-perms",
                    p { "Required permissions:" }
                    ul { for p in perms { li { "{p}" } } }
                }
            }
            div { class: "access-denied-actions", a { class: "btn btn-primary", href: "/", "Back to home" } }
        }
    }
}
