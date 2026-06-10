use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Crumb { pub label: String, pub href: Option<String> }

#[component]
pub fn Breadcrumbs(items: Vec<Crumb>) -> Element {
    rsx! {
        nav { class: "breadcrumbs", "aria-label": "breadcrumb",
            ol { class: "breadcrumbs-list",
                for (i, item) in items.iter().enumerate() {
                    li { class: "breadcrumbs-item",
                        if let Some(h) = &item.href {
                            a { href: "{h}", "{item.label}" }
                        } else {
                            span { "{item.label}" }
                        }
                        if i < items.len() - 1 {
                            span { class: "breadcrumbs-sep", "/" }
                        }
                    }
                }
            }
        }
    }
}
