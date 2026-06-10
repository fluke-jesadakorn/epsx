use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[component]
pub fn Pagination(current_page: u32, total_pages: u32, base_href: String, query_param: Option<String>) -> Element {
    let qp = query_param.unwrap_or_else(|| "page".to_string());
    let prev = if current_page > 1 { current_page - 1 } else { 1 };
    let next = if current_page < total_pages { current_page + 1 } else { total_pages };
    rsx! {
        nav { class: "pagination", "aria-label": "Pagination",
            a { class: "pagination-btn", href: "{base_href}?{qp}={prev}",
                span { Icon { name: "chevron-down".to_string(), size: Some(16) } }
                span { "Prev" }
            }
            span { class: "pagination-info", "Page {current_page} of {total_pages}" }
            a { class: "pagination-btn", href: "{base_href}?{qp}={next}",
                span { "Next" }
                span { Icon { name: "chevron-right".to_string(), size: Some(16) } }
            }
        }
    }
}

#[component]
pub fn LimitSelector(current: u32, base_href: String) -> Element {
    let limits = [10u32, 25, 50, 100];
    rsx! {
        div { class: "limit-selector",
            span { "Show" }
            select { class: "input input-sm", onchange: move |_| {},
                for l in limits {
                    option { value: "{l}", selected: l == current, "{l}" }
                }
            }
        }
    }
}
