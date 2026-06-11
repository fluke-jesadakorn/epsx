//! Admin breadcrumbs — 1:1 port of
//! `apps-old/admin-frontend/components/layout/breadcrumb.tsx`.
//!
//! Two public components live in this module:
//! - [`Breadcrumbs`] — plural, list-driven (existing public API; legacy
//!   scaffold). Unchanged.
//! - [`Breadcrumb`] — singular, route-aware (new TS-parity port). Reads
//!   `current_path` and generates the chain via
//!   [`generate_breadcrumbs`].
//!
//! Plus the [`Crumb`] / [`BreadcrumbItem`] structs and the
//! `ROUTE_CONFIG` data + `generate_breadcrumbs` helper, all
//! `pub` so pages and other chrome components can reuse them.

use dioxus::prelude::*;

/// One breadcrumb entry. Used by the plural list-driven [`Breadcrumbs`].
#[derive(Clone, Debug, PartialEq)]
pub struct Crumb {
    pub label: String,
    pub href: Option<String>,
}

/// TS-parity breadcrumb item — has an optional emoji `icon` for the
/// single-crumb render and the auto-generation helper.
#[derive(Clone, Debug, PartialEq)]
pub struct BreadcrumbItem {
    pub label: String,
    pub href: String,
    /// Optional emoji icon (e.g. `"🏠"`). Rendered as text in `<span>`.
    pub icon: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────
// Plural (legacy) breadcrumbs — list-driven.
// ─────────────────────────────────────────────────────────────────────────

/// Plural, list-driven breadcrumbs. The legacy public API; kept for
/// backward compatibility with `crate::layout::Breadcrumbs`.
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

// ─────────────────────────────────────────────────────────────────────────
// Singular (new) Breadcrumb — route-aware auto-generated chain.
// ─────────────────────────────────────────────────────────────────────────

/// TS-parity route config. Mirrors the `routeConfig` map in
/// `breadcrumb.tsx` (line 13). The `key` is the path string; the value
/// is the human label + optional emoji icon.
///
/// `static` so it's a single allocation shared across all calls.
pub static ROUTE_CONFIG: std::sync::LazyLock<std::collections::HashMap<&'static str, BreadcrumbItem>> = std::sync::LazyLock::new(|| {
    use std::collections::HashMap;
    let mut m: HashMap<&'static str, BreadcrumbItem> = HashMap::new();
    m.insert("/", BreadcrumbItem { label: "Dashboard".into(), href: "/".into(), icon: Some("🏠".into()) });
    m.insert("/users", BreadcrumbItem { label: "Users".into(), href: "/users".into(), icon: Some("👥".into()) });
    m.insert("/users/create", BreadcrumbItem { label: "Create user".into(), href: "/users/create".into(), icon: None });
    m.insert("/users/bulk", BreadcrumbItem { label: "Bulk Operations".into(), href: "/users/bulk".into(), icon: None });
    m.insert("/analytics", BreadcrumbItem { label: "Analytics".into(), href: "/analytics".into(), icon: Some("📊".into()) });
    m.insert("/analytics/eps", BreadcrumbItem { label: "EPS Analytics".into(), href: "/analytics/eps".into(), icon: None });
    m.insert("/notifications", BreadcrumbItem { label: "Notifications".into(), href: "/notifications".into(), icon: Some("🔔".into()) });
    m.insert("/notifications/manage", BreadcrumbItem { label: "Overview".into(), href: "/notifications/manage".into(), icon: None });
    m.insert("/notifications/create", BreadcrumbItem { label: "Create Notification".into(), href: "/notifications/create".into(), icon: None });
    m.insert("/settings", BreadcrumbItem { label: "Settings".into(), href: "/settings".into(), icon: Some("⚙️".into()) });
    m.insert("/audit-log", BreadcrumbItem { label: "Audit Log".into(), href: "/audit-log".into(), icon: Some("📜".into()) });
    m.insert("/bulk-permissions", BreadcrumbItem { label: "Bulk Permissions".into(), href: "/bulk-permissions".into(), icon: Some("⚡".into()) });
    m.insert("/developer-portal", BreadcrumbItem { label: "Developer Portal".into(), href: "/developer-portal".into(), icon: Some("👨‍💻".into()) });
    m.insert("/wallet-management", BreadcrumbItem { label: "Wallet Management".into(), href: "/wallet-management".into(), icon: Some("👛".into()) });
    m.insert("/wallet-management/wallets", BreadcrumbItem { label: "Wallets".into(), href: "/wallet-management/wallets".into(), icon: None });
    m.insert("/wallet-management/access", BreadcrumbItem { label: "Access Control".into(), href: "/wallet-management/access".into(), icon: None });
    m.insert("/wallet-management/access/plans", BreadcrumbItem { label: "Plans".into(), href: "/wallet-management/access/plans".into(), icon: None });
    m.insert("/payments", BreadcrumbItem { label: "Payments".into(), href: "/payments".into(), icon: Some("💰".into()) });
    m.insert("/chat", BreadcrumbItem { label: "Chat Support".into(), href: "/chat".into(), icon: Some("💬".into()) });
    m
});

/// TS-parity breadcrumb generator. Splits `pathname` into segments and
/// walks `ROUTE_CONFIG`, falling back to a title-cased label for any
/// segment that isn't in the map. The dashboard crumb is always the
/// first entry — when the user is already on `/`, only the dashboard
/// crumb is returned.
pub fn generate_breadcrumbs(pathname: &str) -> Vec<BreadcrumbItem> {
    let mut breadcrumbs: Vec<BreadcrumbItem> = Vec::new();
    let dashboard_item = ROUTE_CONFIG
        .get("/")
        .cloned()
        .unwrap_or_else(|| BreadcrumbItem { label: "Dashboard".into(), href: "/".into(), icon: Some("🏠".into()) });
    breadcrumbs.push(dashboard_item);

    let segments: Vec<&str> = pathname.split('/').filter(|s| !s.is_empty()).collect();
    let mut current_path = String::new();
    for segment in segments {
        current_path.push('/');
        current_path.push_str(segment);
        if let Some(item) = ROUTE_CONFIG.get(current_path.as_str()) {
            breadcrumbs.push(item.clone());
        } else {
            // Fallback: title-cased label with hyphens converted to spaces.
            let label = segment
                .split('-')
                .map(|w| {
                    let mut chars = w.chars();
                    match chars.next() {
                        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                        None => String::new(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            breadcrumbs.push(BreadcrumbItem {
                label,
                href: current_path.clone(),
                icon: None,
            });
        }
    }

    // Remove duplicate dashboard if current page is dashboard.
    if pathname == "/" && breadcrumbs.len() > 1 {
        let first = breadcrumbs[0].clone();
        return vec![first];
    }
    breadcrumbs
}

/// Singular, route-aware breadcrumb. The single-crumb render branch
/// (when the chain has only the dashboard) uses a compact layout; the
/// multi-crumb render chain links every non-last entry.
///
/// Mirrors the `Breadcrumb` component in `breadcrumb.tsx` line 78.
#[component]
pub fn Breadcrumb(current_path: String) -> Element {
    let items = generate_breadcrumbs(&current_path);
    if items.len() <= 1 {
        // Single crumb — compact layout.
        return rsx! {
            div { class: "flex items-center gap-1.5 sm:gap-2 text-xs sm:text-sm min-w-0",
                span { class: "text-muted-foreground flex-shrink-0",
                    "{items.first().and_then(|i| i.icon.clone()).unwrap_or_else(|| \"🏠\".to_string())}"
                }
                span { class: "font-semibold bg-gradient-to-r from-yellow-600 to-orange-600 dark:from-purple-400 dark:to-pink-400 bg-clip-text text-transparent truncate",
                    "{items.first().map(|i| i.label.clone()).unwrap_or_else(|| \"Dashboard\".to_string())}"
                }
            }
        };
    }

    // Multi-crumb render.
    rsx! {
        div { class: "flex items-center gap-1 sm:gap-1.5 lg:gap-2 text-xs sm:text-sm min-w-0 overflow-hidden",
            for (index, item) in items.iter().enumerate() {
                {
                    let is_last = index == items.len() - 1;
                    let is_first = index == 0;
                    rsx! {
                        div { class: "flex items-center gap-1 sm:gap-1.5 lg:gap-2 flex-shrink-0",
                            if is_first {
                                if let Some(icon) = &item.icon {
                                    span { class: "text-muted-foreground flex-shrink-0", "{icon}" }
                                } else {
                                    span { class: "text-muted-foreground flex-shrink-0", "🏠" }
                                }
                            }
                            if !is_last {
                                a {
                                    class: "text-muted-foreground hover:text-gray-800 dark:hover:text-gray-100 truncate max-w-[100px] sm:max-w-[150px] lg:max-w-none",
                                    href: "{item.href}",
                                    title: "{item.label}",
                                    "{item.label}"
                                }
                                span { class: "text-muted-foreground flex-shrink-0", "/" }
                            } else {
                                span {
                                    class: "font-semibold bg-gradient-to-r from-yellow-600 to-orange-600 dark:from-purple-400 dark:to-pink-400 bg-clip-text text-transparent truncate max-w-[150px] sm:max-w-[200px] lg:max-w-none",
                                    title: "{item.label}",
                                    "{item.label}"
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
    fn generate_breadcrumbs_for_dashboard_only() {
        let crumbs = generate_breadcrumbs("/");
        assert_eq!(crumbs.len(), 1);
        assert_eq!(crumbs[0].label, "Dashboard");
    }

    #[test]
    fn generate_breadcrumbs_for_nested_path() {
        let crumbs = generate_breadcrumbs("/wallet-management/access/plans");
        // Dashboard + Wallet Management + Access Control + Plans
        assert_eq!(crumbs.len(), 4);
        assert_eq!(crumbs[0].label, "Dashboard");
        assert_eq!(crumbs[1].label, "Wallet Management");
        assert_eq!(crumbs[2].label, "Access Control");
        assert_eq!(crumbs[3].label, "Plans");
    }

    #[test]
    fn generate_breadcrumbs_falls_back_for_unknown_segment() {
        let crumbs = generate_breadcrumbs("/some/unknown-route");
        // Dashboard + Some + Unknown Route
        assert_eq!(crumbs.len(), 3);
        assert_eq!(crumbs[1].label, "Some");
        assert_eq!(crumbs[2].label, "Unknown Route");
    }
}
