use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[component]
pub fn PageHeader(title: String, description: Option<String>, icon: Option<String>, children: Element) -> Element {
    rsx! {
        header { class: "page-header",
            div { class: "page-header-inner",
                div { class: "page-header-content",
                    if let Some(i) = icon {
                        span { class: "page-header-icon", Icon { name: i.clone(), size: Some(24) } }
                    }
                    div {
                        h1 { class: "page-title", "{title}" }
                        if let Some(d) = description {
                            p { class: "page-description text-muted-foreground", "{d}" }
                        }
                    }
                }
                div { class: "page-header-actions", {children} }
            }
        }
    }
}
