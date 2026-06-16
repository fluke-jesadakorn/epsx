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

/// Pick a page size (10 / 25 / 50 / 100). Wave 23 T4 v2: the
/// `onchange` handler used to be a Dioxus `move |e| { … }` closure
/// that was stripped at SSR time (hydration-less), so changing
/// the dropdown did nothing. Now we render the select as raw HTML
/// via `epsx_templates::navigate_select_html`, which sets
/// `data-epsx-navigate="1"` + `data-base-href` attributes; the
/// `global_js` `bindNavigateSelects()` listener picks it up on
/// `DOMContentLoaded` and wires the `change` handler that
/// navigates to `<base_href>?limit=<value>`.
#[component]
pub fn LimitSelector(current: u32, base_href: String) -> Element {
    let limits: Vec<(String, String)> = [10u32, 25, 50, 100]
        .iter()
        .map(|l| (l.to_string(), l.to_string()))
        .collect();
    let current_str = current.to_string();
    let html = epsx_templates::navigate_select_html(&base_href, "limit", &current_str, &limits);
    rsx! {
        div { class: "limit-selector",
            span { "Show" }
            span { class: "limit-selector-select-wrap inline-block",
                dangerous_inner_html: "{html}"
            }
        }
    }
}
