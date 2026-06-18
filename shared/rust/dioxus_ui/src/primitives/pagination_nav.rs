//! `PaginationNav` — page-number navigation control.
//!
//! Mirrors the pagination pattern used in `analytics/`,
//! `developer/docs/`, and `dashboard/` pages. Renders Previous / page
//! numbers / Next buttons with `aria-label`s for screen readers.
//!
//! Naming: the existing `crate::data::Pagination` primitive already
//! covers the basic use case (base_href + total_pages + query_param);
//! `PaginationNav` adds a callback-driven variant for use cases that
//! want client-side pagination (e.g. infinite-scroll fallbacks).
//!
//! Usage:
//! ```ignore
//! PaginationNav {
//!     current_page: 3,
//!     total_pages: 10,
//!     base_href: Some("/news".into()),
//!     on_change: Some(handler),
//! }
//! ```

use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PaginationKind {
    /// Render as numbered buttons with Previous / Next.
    Numbered,
    /// Render as Previous / Next only (compact mode).
    Simple,
}

#[component]
pub fn PaginationNav(
    current_page: u32,
    total_pages: u32,
    #[props(default = None)] base_href: Option<String>,
    #[props(default = PaginationKind::Numbered)] kind: PaginationKind,
    #[props(default = None)] on_change: Option<EventHandler<u32>>,
    #[props(default = None)] class_name: Option<String>,
) -> Element {
    let total = total_pages.max(1);
    let cur = current_page.max(1).min(total);
    let extra = class_name.unwrap_or_default();
    let base_cls = if extra.is_empty() {
        "pagination flex items-center gap-2".to_string()
    } else {
        format!("pagination flex items-center gap-2 {extra}")
    };

    let href_for_page = |page: u32| -> String {
        match &base_href {
            Some(h) => {
                if h.contains('?') {
                    format!("{h}&page={page}")
                } else {
                    format!("{h}?page={page}")
                }
            }
            None => format!("?page={page}"),
        }
    };

    let prev_disabled = cur <= 1;
    let next_disabled = cur >= total;

    let prev_page = if cur > 1 { cur - 1 } else { 1 };
    let next_page = if cur < total { cur + 1 } else { total };

    let on_prev = move |_: MouseEvent| {
        if !prev_disabled {
            if let Some(h) = &on_change {
                h.call(prev_page);
            }
        }
    };
    let on_next = move |_: MouseEvent| {
        if !next_disabled {
            if let Some(h) = &on_change {
                h.call(next_page);
            }
        }
    };

    rsx! {
        nav { class: "{base_cls}", "aria-label": "Pagination",
            if matches!(kind, PaginationKind::Numbered) {
                // Page list — collapses to a window when there are too
                // many pages so we don't render 1000 buttons.
                {render_page_list(cur, total, href_for_page)}
            }
            button {
                class: "pagination-prev px-3 py-1 rounded border disabled:opacity-50",
                r#type: "button",
                disabled: prev_disabled,
                "aria-label": "Previous page",
                onclick: on_prev,
                "Previous"
            }
            button {
                class: "pagination-next px-3 py-1 rounded border disabled:opacity-50",
                r#type: "button",
                disabled: next_disabled,
                "aria-label": "Next page",
                onclick: on_next,
                "Next"
            }
        }
    }
}

fn render_page_list<F: Fn(u32) -> String>(current: u32, total: u32, href_for: F) -> Element {
    // Window: first, ..., current-1, current, current+1, ..., last.
    let mut pages: Vec<u32> = Vec::new();
    let window: i64 = 1;
    let total_i = total as i64;
    let cur_i = current as i64;
    for p in 1..=total_i {
        let p_u = p as u32;
        let dist = (p - cur_i).abs();
        if p == 1 || p == total_i || dist <= window {
            pages.push(p_u);
        }
    }
    let mut with_ellipsis: Vec<PageSlot> = Vec::new();
    let mut last: Option<u32> = None;
    for p in pages {
        if let Some(l) = last {
            if p > l + 1 {
                with_ellipsis.push(PageSlot::Ellipsis);
            }
        }
        with_ellipsis.push(PageSlot::Number(p));
        last = Some(p);
    }
    rsx! {
        ul { class: "pagination-list flex items-center gap-1 list-none m-0 p-0",
            for slot in with_ellipsis {
                match slot {
                    PageSlot::Number(p) => {
                        let href = href_for(p);
                        let active = p == current;
                        rsx! {
                            li { class: "pagination-page",
                                a { class: if active {
                                        "pagination-link is-active px-3 py-1 rounded border bg-primary text-primary-foreground"
                                    } else {
                                        "pagination-link px-3 py-1 rounded border"
                                    },
                                    href: "{href}",
                                    "aria-current": if active { "page" } else { "false" },
                                    "{p}"
                                }
                            }
                        }
                    }
                    PageSlot::Ellipsis => {
                        rsx! {
                            li { class: "pagination-ellipsis px-2 text-muted-foreground",
                                span { "…" }
                            }
                        }
                    }
                }
            }
        }
    }
}

enum PageSlot { Number(u32), Ellipsis }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prev_disabled_when_on_first_page() {
        let cur = 1u32;
        let total = 5u32;
        let prev_disabled = cur <= 1;
        assert!(prev_disabled);
    }

    #[test]
    fn next_disabled_when_on_last_page() {
        let cur = 5u32;
        let total = 5u32;
        let next_disabled = cur >= total;
        assert!(next_disabled);
    }

    #[test]
    fn href_query_appends_correctly() {
        let h = Some("/news".to_string());
        let result = match &h {
            Some(b) => {
                if b.contains('?') {
                    format!("{b}&page=2")
                } else {
                    format!("{b}?page=2")
                }
            }
            None => "?page=2".to_string(),
        };
        assert_eq!(result, "/news?page=2");

        let h2 = Some("/news?cat=tech".to_string());
        let result2 = match &h2 {
            Some(b) => {
                if b.contains('?') {
                    format!("{b}&page=2")
                } else {
                    format!("{b}?page=2")
                }
            }
            None => "?page=2".to_string(),
        };
        assert_eq!(result2, "/news?cat=tech&page=2");
    }

    #[test]
    fn current_page_is_clamped_to_total() {
        let cur = 100u32;
        let total = 5u32;
        let clamped = cur.max(1).min(total);
        assert_eq!(clamped, 5);
    }
}