//! /permissions — user permissions with matrix + feature list.
//!
//! Wave 6A Track D — port of `apps-old/frontend/app/permissions/page.tsx`
//! (91 LoC, the page is small with no sub-components) + the
//! `permissions-display.tsx` matrix pattern (110 LoC).
//!
//! Section coverage (matches design doc §"Track D — permissions"):
//! - `PermissionsMatrix` — table of feature × plan (the matrix grid)
//! - `FeatureList` — bulleted list of permissions per current plan
//! - `RequestAccessCTA` — button to request additional permissions
//!
//! The current 93 LoC `permissions.rs` already had the
//! `permissions / analytics / history` tabs from Wave 3b gate
//! enrichment; we preserve them as the analytics/history views and
//! add the new `PermissionsMatrix` as the default tab (which
/// replaces the previous simple card grid). The 6 permission
/// cards from the old version are folded into the matrix's rows.

use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::auth::AuthGate;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Permissions");
    (meta, rsx! { RenderPermissions { ctx: ctx.clone() } })
}

#[component]
fn RenderPermissions(ctx: PageContext) -> Element {
    let mut tab = use_signal(|| "permissions".to_string());
    rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("your permissions".to_string()),
                required_permissions: Some(vec!["permissions:read".to_string()]),
                return_url: Some(ctx.path.clone()),
                div { class: "container page-content max-w-6xl",
                    // === wave6-auth-pages-depth-track-d permissions header ===
                    PageHeader { title: "My permissions".to_string(), description: Some("Active permissions, history, and analytics".to_string()), icon: Some("shield".to_string()) }
                    // === wave6-auth-pages-depth-track-d permissions stat cards ===
                    div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 mb-4 permissions-stats",
                        StatCard { label: "Total permissions".to_string(), value: "12".to_string(), icon: Some("key".to_string()) }
                        StatCard { label: "Active".to_string(), value: "10".to_string(), icon: Some("check".to_string()) }
                        StatCard { label: "Expiring soon".to_string(), value: "2".to_string(), icon: Some("clock".to_string()) }
                    }
                    div { class: "card card-glass",
                        div { class: "card-header",
                            h2 { class: "card-title", "Permission details" }
                            // === wave6-auth-pages-depth-track-d permissions tab nav ===
                            div { class: "tabs permissions-tab-nav",
                                button { class: if *tab.read() == "permissions" { "btn btn-sm btn-primary" } else { "btn btn-sm btn-outline" }, onclick: move |_| tab.set("permissions".to_string()), "Permissions" }
                                button { class: if *tab.read() == "analytics" { "btn btn-sm btn-primary" } else { "btn btn-sm btn-outline" }, onclick: move |_| tab.set("analytics".to_string()), "Analytics" }
                                button { class: if *tab.read() == "history" { "btn btn-sm btn-primary" } else { "btn btn-sm btn-outline" }, onclick: move |_| tab.set("history".to_string()), "History" }
                            }
                        }
                        div { class: "card-body",
                            if *tab.read() == "permissions" {
                                // === wave6-auth-pages-depth-track-d permissions matrix ===
                                div { class: "space-y-6 permissions-matrix-panel",
                                    PermissionsMatrix {}
                                    // === wave6-auth-pages-depth-track-d permissions feature list ===
                                    FeatureList {}
                                    // === wave6-auth-pages-depth-track-d permissions request-access ===
                                    RequestAccessCTA {}
                                }
                            } else if *tab.read() == "analytics" {
                                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                                    div { class: "card card-glass", div { class: "card-header", h4 { "By category" } } div { class: "card-body",
                                        crate::charts::ChartDonut { data: vec![("Trade".to_string(), 4.0, "#22d3ee".to_string()), ("View".to_string(), 3.0, "#22c55e".to_string()), ("Pay".to_string(), 3.0, "#f59e0b".to_string())], size: 180, thickness: 28 } } }
                                    div { class: "card card-glass", div { class: "card-header", h4 { "By source" } } div { class: "card-body",
                                        crate::charts::ChartDonut { data: vec![("Plan".to_string(), 8.0, "#3b82f6".to_string()), ("Direct".to_string(), 2.0, "#9ca3af".to_string())], size: 180, thickness: 28 } } }
                                }
                            } else {
                                // === wave6-auth-pages-depth-track-d permissions history ===
                                div { class: "table-wrap permissions-history-table",
                                    table { class: "table", thead { tr { th { "Date" } th { "Action" } th { "Permission" } th { "Source" } } } tbody {
                                        tr { td { "2024-09-15" } td { span { class: "badge badge-success", "Granted" } } td { "trade" } td { "Pro plan" } }
                                        tr { td { "2024-08-01" } td { span { class: "badge badge-info", "Updated" } } td { "view" } td { "Pro plan" } }
                                        tr { td { "2024-07-20" } td { span { class: "badge badge-danger", "Revoked" } } td { "admin" } td { "— " } }
                                    } } }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// `PermissionsMatrix` — table of feature × plan. Rows are the
/// features (per the 6 permission names in the original perm
/// cards); columns are the 3 plans (Free / Pro / Enterprise).
/// Each cell shows a check/cross/limited glyph. Mirrors the
/// matrix grid in `permissions-display.tsx` (110 LoC).
#[component]
fn PermissionsMatrix() -> Element {
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
fn FeatureList() -> Element {
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
                ul { class: "space-y-2 list-disc ml-6",
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
fn RequestAccessCTA() -> Element {
    rsx! {
        div { class: "card card-glass permissions-request-access",
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::User;
    use crate::pages::PageContext;
    use crate::auth::user::AuthMethod;

    fn ctx(path: &str) -> PageContext {
        let user = User {
            id: "u1".to_string(),
            address: "0x1234…abcd".to_string(),
            chain_id: "56".to_string(),
            roles: vec!["user".to_string()],
            email: Some("test@epsx.io".to_string()),
            tier: Some("Pro".to_string()),
            permissions: vec!["permissions:read".to_string()],
            last_login_at: None,
            auth_method: AuthMethod::Wallet,
            display_name: Some("Test".to_string()),
        };
        PageContext { user: Some(user), path: path.to_string(), ..Default::default() }
    }

    #[test]
    fn test_render_smoke() {
        let (_meta, element) = render(&ctx("/permissions"));
        let html = dioxus_ssr::render_element(element);
        assert!(html.contains("My permissions"), "/permissions header must render. Got: {}", html);
    }

    #[test]
    fn test_section_markers() {
        let (_meta, element) = render(&ctx("/permissions"));
        let html = dioxus_ssr::render_element(element);
        for marker in [
            "permissions-stats",
            "permissions-tab-nav",
            "permissions-matrix",
            "permissions-feature-list",
            "permissions-request-access",
        ] {
            assert!(html.contains(marker), "missing section marker: {}", marker);
        }
    }
}
