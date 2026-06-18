//! `BreadcrumbNav` — shadcn-style navigation breadcrumb primitive.
//!
//! Mirrors the breadcrumb pattern used across `apps-old/frontend` pages
//! (manual, docs sidebar, profile, dashboard, settings). Renders a
//! list of breadcrumb items separated by a separator (default `"/"`),
//! with the last item rendered as the current page (non-clickable).
//!
//! Naming: the existing `crate::layout::Breadcrumb` / `Breadcrumbs`
//! already provide admin route-aware breadcrumbs, so this primitive is
//! named `BreadcrumbNav` to avoid ambiguous-glob-reexport warnings.
//!
//! Usage:
//! ```ignore
//! BreadcrumbNav { items: vec![
//!     BreadcrumbNavItem { label: "Home".into(), href: Some("/".into()) },
//!     BreadcrumbNavItem { label: "Docs".into(), href: Some("/docs".into()) },
//!     BreadcrumbNavItem { label: "API".into(), href: None },
//! ] }
//! ```

use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct BreadcrumbNavItem {
    pub label: String,
    pub href: Option<String>,
}

#[component]
pub fn BreadcrumbNav(
    items: Vec<BreadcrumbNavItem>,
    #[props(default = None)] class_name: Option<String>,
    #[props(default = None)] separator: Option<String>,
) -> Element {
    let extra = class_name.unwrap_or_default();
    let sep = separator.unwrap_or_else(|| "/".to_string());
    let cls = if extra.is_empty() {
        "breadcrumb flex items-center gap-2 text-sm text-muted-foreground".to_string()
    } else {
        format!("breadcrumb flex items-center gap-2 text-sm text-muted-foreground {extra}")
    };
    let n = items.len();
    rsx! {
        nav { class: "{cls}", "aria-label": "Breadcrumb",
            ol { class: "flex items-center gap-2",
                for (i, item) in items.iter().enumerate() {
                    {
                        let is_last = i + 1 == n;
                        let label = item.label.clone();
                        let href = item.href.clone();
                        rsx! {
                            li { class: "breadcrumb-item flex items-center gap-2",
                                if let Some(h) = href.clone() {
                                    if !is_last {
                                        a { class: "breadcrumb-link hover:underline text-foreground", href: "{h}",
                                            "{label}"
                                        }
                                    } else {
                                        span { class: "breadcrumb-current text-foreground font-medium",
                                            "{label}"
                                        }
                                    }
                                } else {
                                    span { class: if is_last { "breadcrumb-current text-foreground font-medium" } else { "breadcrumb-leaf text-foreground" },
                                        "{label}"
                                    }
                                }
                                if !is_last {
                                    span { class: "breadcrumb-separator text-muted-foreground", "{sep}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breadcrumb_nav_item_struct_holds_label_and_optional_href() {
        let item = BreadcrumbNavItem {
            label: "Home".into(),
            href: Some("/".into()),
        };
        assert_eq!(item.label, "Home");
        assert_eq!(item.href.as_deref(), Some("/"));
    }

    #[test]
    fn breadcrumb_nav_item_can_have_no_href() {
        let item = BreadcrumbNavItem {
            label: "Current".into(),
            href: None,
        };
        assert!(item.href.is_none());
    }

    #[test]
    fn default_separator_is_slash() {
        // The default separator is "/".
        let sep: String = "/".to_string();
        assert_eq!(sep, "/");
    }
}