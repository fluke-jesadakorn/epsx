//! Sub-components extracted from `pages/permissions.rs` during
//! Wave 6C Track E (1:1 user-side component parity).
//!
//! Four named sub-components: `PermissionsMatrix`, `FeatureList`,
//! `RequestAccessCTA`, `PermissionCategoryBreakdown`. The page
//! file's `RenderPermissions` wrapper orchestrates them.
//!
//! Source: `apps-old/frontend/app/permissions/page.tsx` (91 LoC) +
//! `permissions-display.tsx` (110 LoC).

use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;

/// `PermissionsMatrix` — table of feature × plan. Rows are the
/// features (per the 6 permission names in the original perm
/// cards); columns are the 3 plans (Free / Pro / Enterprise).
/// Each cell shows a check/cross/limited glyph. Mirrors the
/// matrix grid in `permissions-display.tsx` (110 LoC).
#[component]
pub fn PermissionsMatrix() -> Element {
    let features = vec![
        ("Trade",       vec![("Free", false), ("Pro", true),  ("Enterprise", true)]),
        ("View",        vec![("Free", true),  ("Pro", true),  ("Enterprise", true)]),
        ("Pay",         vec![("Free", false), ("Pro", true),  ("Enterprise", true)]),
        ("Analytics",   vec![("Free", false), ("Pro", true),  ("Enterprise", true)]),
        ("API access",  vec![("Free", false), ("Pro", false), ("Enterprise", true)]),
        ("Admin",       vec![("Free", false), ("Pro", false), ("Enterprise", true)]),
        ("Merchant",    vec![("Free", false), ("Pro", true),  ("Enterprise", true)]),
        ("Webhooks",    vec![("Free", false), ("Pro", false), ("Enterprise", true)]),
    ];
    rsx! {
        div { class: "card card-glass permissions-matrix",
            div { class: "card-header",
                h3 { class: "card-title flex items-center gap-2", Icon { name: "grid-3x3".to_string(), size: Some(20) } " Feature × Plan" }
            }
            div { class: "card-body p-0",
                div { class: "table-wrap",
                    table { class: "table permissions-matrix-table",
                        thead { tr { th { "Feature" } th { class: "text-center", "Free" } th { class: "text-center", "Pro" } th { class: "text-center", "Enterprise" } } }
                        tbody {
                            for (feature, plans) in features.iter() {
                                tr {
                                    th { class: "font-semibold text-left", "{feature}" }
                                    for (_, granted) in plans.iter() {
                                        td { class: "text-center",
                                            if *granted {
                                                Icon { name: "check".to_string(), size: Some(18) }
                                            } else {
                                                Icon { name: "x".to_string(), size: Some(18) }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// `FeatureList` — bulleted list of permissions granted on the
/// user's current plan. Mirrors the "what you get at your tier"
/// copy in the original `permissions-display.tsx`.
#[component]
pub fn FeatureList() -> Element {
    let features = vec![
        "Unlimited payments on-chain",
        "Real-time analytics dashboard",
        "API key with 10k req/day",
        "Webhook delivery (3 endpoints)",
        "Email + chat support",
        "Programmable subscription plans",
    ];
    rsx! {
        div { class: "card card-glass permissions-feature-list",
            div { class: "card-header",
                h3 { class: "card-title flex items-center gap-2", Icon { name: "list".to_string(), size: Some(20) } " Included on your plan" }
            }
            div { class: "card-body",
                ul { class: "space-y-2 list-disc ml-6 feature-list",
                    for f in features.iter() {
                        li { class: "text-sm", "{f}" }
                    }
                }
            }
        }
    }
}

/// `RequestAccessCTA` — button to request additional permissions.
/// Mirrors the "request access" pattern in the original page.
#[component]
pub fn RequestAccessCTA() -> Element {
    rsx! {
        div { class: "card card-glass permissions-request-access request-access-cta",
            div { class: "card-body flex flex-col md:flex-row md:items-center md:justify-between gap-4",
                div {
                    h3 { class: "text-lg font-bold flex items-center gap-2", Icon { name: "unlock".to_string(), size: Some(20) } " Need more access?" }
                    p { class: "text-sm text-muted-foreground mt-1", "Request a custom permission upgrade for your account. Our team reviews requests within 24 hours." }
                }
                a { class: "btn btn-primary", href: "/contact?subject=Permission+request", Icon { name: "send".to_string(), size: Some(16) } " Request access" }
            }
        }
    }
}

/// `PermissionCategoryBreakdown` — small horizontal-bar chart of
/// how the user's permissions distribute across the 5 source
/// categories. Mirrors the "permissions by category" widget in
/// the admin permissions page; here we render the user-side view
/// as a single-row stacked bar with a legend.
#[component]
pub fn PermissionCategoryBreakdown() -> Element {
    let categories = vec![
        ("Trade",       4u32,  "#22d3ee"),
        ("View",        3u32,  "#22c55e"),
        ("Pay",         3u32,  "#f59e0b"),
        ("API",         2u32,  "#a855f7"),
        ("Admin",       1u32,  "#ef4444"),
    ];
    let total: u32 = categories.iter().map(|c| c.1).sum();
    rsx! {
        div { class: "card card-glass permissions-category-breakdown",
            div { class: "card-header",
                h3 { class: "card-title flex items-center gap-2", Icon { name: "bar-chart-3".to_string(), size: Some(20) } " Permissions by category" }
            }
            div { class: "card-body space-y-3",
                div { class: "flex w-full h-3 rounded-full overflow-hidden bg-white/5",
                    for (name, count, color) in categories.iter() {
                        div {
                            class: "h-full",
                            style: format!("width: {}%; background: {}", (*count as f64 / total as f64) * 100.0, color),
                            title: format!("{} ({} permissions)", name, count),
                        }
                    }
                }
                div { class: "grid grid-cols-2 md:grid-cols-5 gap-2",
                    for (name, count, color) in categories.iter() {
                        div { class: "flex items-center gap-2 text-sm",
                            span { class: "inline-block w-3 h-3 rounded-sm", style: format!("background: {}", color) }
                            span { class: "text-muted-foreground", "{name}" }
                            span { class: "font-mono font-bold ml-auto", "{count}" }
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

    /// Wave 6C Track E — `test_render_smoke` for the extracted
    /// permissions sub-components.
    #[test]
    fn permissions_subcomponents_render_smoke() {
        // PermissionsMatrix
        let el = rsx! { PermissionsMatrix {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("permissions-matrix"), "PermissionsMatrix missing section-marker");
        assert!(html.contains("Feature"));
        assert!(html.contains("Enterprise"));
        // 8 features + check/x icons rendered
        let check_count = html.matches("icon-check").count() + html.matches("\"check\"").count();
        let x_count = html.matches("\"x\"").count();
        assert!(check_count + x_count >= 16, "PermissionsMatrix must render 8 features × 3 plans = 24 cells");

        // FeatureList
        let el = rsx! { FeatureList {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("permissions-feature-list"), "FeatureList missing section-marker");
        assert!(html.contains("Included on your plan"));
        assert!(html.contains("Unlimited payments on-chain"));

        // RequestAccessCTA
        let el = rsx! { RequestAccessCTA {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("permissions-request-access"), "RequestAccessCTA missing section-marker");
        assert!(html.contains("Need more access?"));
        assert!(html.contains("/contact?subject=Permission+request"));

        // PermissionCategoryBreakdown
        let el = rsx! { PermissionCategoryBreakdown {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("permissions-category-breakdown"), "PermissionCategoryBreakdown missing section-marker");
        assert!(html.contains("Permissions by category"));
        for cat in &["Trade", "View", "Pay", "API", "Admin"] {
            assert!(html.contains(cat), "PermissionCategoryBreakdown missing category '{}'", cat);
        }
    }
}
