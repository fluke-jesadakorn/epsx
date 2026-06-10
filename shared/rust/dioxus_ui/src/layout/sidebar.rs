use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct SidebarItem { pub label: String, pub href: String, pub icon: String, pub badge: Option<String>, pub active_paths: Vec<String> }

#[component]
pub fn Sidebar(items: Vec<SidebarItem>, current_path: String, header: Option<String>) -> Element {
    rsx! {
        aside { class: "admin-sidebar",
            if let Some(h) = header {
                div { class: "sidebar-header", h3 { "{h}" } }
            }
            nav { class: "sidebar-nav",
                for item in items {
                    a {
                        class: if item.active_paths.iter().any(|p| current_path.starts_with(p)) { "sidebar-item active" } else { "sidebar-item" },
                        href: "{item.href}",
                        span { class: "sidebar-icon", Icon { name: item.icon.clone(), size: Some(18) } }
                        span { class: "sidebar-label", "{item.label}" }
                        if let Some(b) = &item.badge {
                            span { class: "sidebar-badge", "{b}" }
                        }
                    }
                }
            }
        }
    }
}
