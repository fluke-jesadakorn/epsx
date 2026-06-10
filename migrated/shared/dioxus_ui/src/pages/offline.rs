use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;

pub fn render(_ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Offline");
    (meta, rsx! {
        div { class: "offline-page",
            div { class: "offline-card card card-glass",
                div { class: "offline-icon", Icon { name: "info".to_string(), size: Some(48) } }
                h1 { "You're offline" }
                p { class: "text-muted-foreground", "Check your connection and try again." }
                div { class: "offline-actions",
                    a { class: "btn btn-primary", href: "javascript:location.reload()", "Refresh" }
                    a { class: "btn btn-outline", href: "/", "Home" }
                    a { class: "btn btn-outline", href: "/notifications", "Notifications" }
                }
            }
        }
    })
}
