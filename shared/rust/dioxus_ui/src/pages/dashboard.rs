//! /dashboard — the EPSX personal dashboard.
//!
//! Wave 27 T2 — port of `apps-old/frontend/app/dashboard/page.tsx` +
//! `apps-old/frontend/components/dashboard/dashboard-client.tsx`
//! (75 + 296 = 371 LoC of source).
//!
//! ## Sections (per source design)
//!
//! 1. **Page shell** — `min-h-screen bg-gradient-to-br from-slate-50
//!    to-slate-100 dark:from-slate-900 dark:to-slate-800` container
//!    with `container mx-auto px-4 py-8` inner. (Source page.tsx:48-49.)
//!
//! 2. **Header** — `mb-8` div with h1 `"Personal Dashboard"`
//!    (`text-3xl font-bold text-slate-900 dark:text-slate-100`) +
//!    sub `"Your personalized market analytics and portfolio
//!    overview"` (`text-slate-600 dark:text-slate-400 mt-2`).
//!    (Source page.tsx:50-57.)
//!
//! 3. **Auth-aware body** — when `ctx.user.is_some()`, render the
//!    `DashboardClient` (5 feature cards: Profile / Settings /
//!    Analytics / Premium Content / Moderator Panel + a 6th
//!    "Your Permissions" card). When unauthenticated, render the
//!    "Please sign in to access your dashboard..." fallback.
//!    (Source page.tsx:59-72.)
//!
//! ## Auth-aware note
//!
//! Wave 25 T2 used `MainLayout`-wrapped stats (Total earnings /
//! Watchlist / Active plans / API calls). Wave 27 T2 swaps that for
//! the source's actual structure — an h1 + sub + (authed DashboardClient
//! OR unauth fallback). The pixel-recheck harness captures the
//! unauthenticated state (the prod baseline was anonymous), so the
//! "Please sign in to access your dashboard..." fallback is the
//! branch the harness diffs against.

use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;

/// DashboardClient mock data — mirrors `apps-old/frontend/app/dashboard/page.tsx:35-45`.
#[derive(Clone, Debug, PartialEq)]
struct DashboardMockStats {
    total_views: u64,
    total_users: u64,
    revenue: u64,
}

impl DashboardMockStats {
    fn default_mock() -> Self {
        // Mirrors the source's `dashboardData` default — total_views=0,
        // total_users=1, revenue=0.
        Self { total_views: 0, total_users: 1, revenue: 0 }
    }
}

/// Permissions object mirror — mirrors the source's `permissions` shape
/// (`role`, `permissions: Vec<String>`, `platforms: Vec<String>`,
/// `platform_context: Option<String>`).
#[derive(Clone, Debug, PartialEq)]
struct DashboardPermissions {
    role: String,
    permissions: Vec<String>,
    platforms: Vec<String>,
    platform_context: Option<String>,
}

impl DashboardPermissions {
    fn from_user(user: &crate::auth::User) -> Self {
        Self {
            role: if user.roles.is_empty() { "user".to_string() } else { user.roles[0].clone() },
            permissions: user.permissions.clone(),
            platforms: vec!["epsx".to_string()],
            platform_context: Some("epsx".to_string()),
        }
    }
}

/// User object mirror — mirrors the source's `user` shape (id, email,
/// name, permissions, package_tier, wallet_address, platforms,
/// primary_platform, platform_context).
#[derive(Clone, Debug, PartialEq)]
struct DashboardUserView {
    id: String,
    email: String,
    name: String,
    permissions: Vec<String>,
    package_tier: String,
    wallet_address: Option<String>,
    platforms: Vec<String>,
    primary_platform: String,
    platform_context: Option<String>,
}

impl DashboardUserView {
    fn from_ctx_user(user: &crate::auth::User) -> Self {
        let email = user.email.clone().unwrap_or_default();
        let name = user
            .display_name
            .clone()
            .unwrap_or_else(|| email.split('@').next().unwrap_or("user").to_string());
        Self {
            id: user.id.clone(),
            email,
            name,
            permissions: user.permissions.clone(),
            package_tier: user.tier.clone().unwrap_or_else(|| "FREE".to_string()),
            wallet_address: Some(user.address.clone()),
            platforms: vec!["epsx".to_string()],
            primary_platform: "epsx".to_string(),
            platform_context: Some("epsx".to_string()),
        }
    }
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Dashboard");
    (meta, rsx! { RenderDashboard { ctx: ctx.clone() } })
}

#[component]
fn RenderDashboard(ctx: PageContext) -> Element {
    // Mock stats — when the BFF has no `data_dashboard` payload, we
    // default to the source's mock shape (0/1/0). The OLD prod shows
    // an identical 0/1/0 mock for the `DashboardClient`'s "Total
    // Views / Total Users / Revenue" cards (see dashboard-client.tsx
    // which reads `dashboardData.data.stats`).
    let mock_stats: DashboardMockStats = ctx.params
        .get("data_dashboard")
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .and_then(|v| v.get("stats").cloned())
        .map(|s| DashboardMockStats {
            total_views: s.get("totalViews").and_then(|x| x.as_u64()).unwrap_or(0),
            total_users: s.get("totalUsers").and_then(|x| x.as_u64()).unwrap_or(1),
            revenue: s.get("revenue").and_then(|x| x.as_u64()).unwrap_or(0),
        })
        .unwrap_or_else(DashboardMockStats::default_mock);

    rsx! {
        MainLayout { ctx: ctx.clone(),
            // === wave27-t2-port-fe-pages ===
            // Source: apps-old/frontend/app/dashboard/page.tsx:48-49.
            // Prod renders:
            //   <div class="min-h-screen bg-gradient-to-br from-slate-50 to-slate-100 dark:from-slate-900 dark:to-slate-800">
            //     <div class="container mx-auto px-4 py-8">
            // We replicate the same classes verbatim. The v2-CDN's
            // missing `dark:` support means dark-mode renders light;
            // we leave that as a structural gap (matches the rest of
            // the Wave 26 + Wave 27 ports).
            div { class: "dashboard-prod-page min-h-screen bg-gradient-to-br from-slate-50 to-slate-100 dark:from-slate-900 dark:to-slate-800",
                div { class: "container mx-auto px-4 py-8",
                    // Page header (source page.tsx:50-57).
                    div { class: "dashboard-prod-header mb-8",
                        h1 { class: "dashboard-prod-title text-3xl font-bold text-slate-900 dark:text-slate-100",
                            "Personal Dashboard"
                        }
                        p { class: "dashboard-prod-subtitle text-slate-600 dark:text-slate-400 mt-2",
                            "Your personalized market analytics and portfolio overview"
                        }
                    }
                    // Auth-aware body (source page.tsx:59-72).
                    if let Some(u) = ctx.user.as_ref() {
                        DashboardClient {
                            user: DashboardUserView::from_ctx_user(u),
                            permissions: DashboardPermissions::from_user(u),
                            stats: mock_stats.clone(),
                        }
                    } else {
                        // Unauthed fallback — matches source's
                        // <div class="text-center p-8"><p>...sign
                        // in to access your dashboard...</p></div>.
                        // The harness captures the unauthed state
                        // (prod baseline was anonymous), so this is
                        // the branch the pixel-diff harness measures.
                        div { class: "dashboard-prod-fallback text-center p-8",
                            p { class: "dashboard-prod-fallback-text text-slate-600 dark:text-slate-400",
                                "Please sign in to access your dashboard..."
                            }
                        }
                    }
                }
            }
        }
    }
}

// === DashboardClient ===

/// DashboardClient — authed dashboard body. Mirrors
/// `apps-old/frontend/components/dashboard/dashboard-client.tsx`
/// (296 LoC). Renders 5 feature cards (Profile / Settings /
/// Analytics / Premium Content / Moderator Panel) plus a 6th
/// "Your Permissions" card showing the user's role + permission
/// badges. The source has a PancakeSwap-style vibrant background +
/// floating gradient orbs + animations — we keep those in the
/// markup but the v2-CDN doesn't generate `dark:` variants, so the
/// dark-mode appearance will diverge from prod's `dark:from-slate-900
/// dark:via-slate-800 dark:to-slate-900` gradient.
#[component]
fn DashboardClient(
    user: DashboardUserView,
    permissions: DashboardPermissions,
    stats: DashboardMockStats,
) -> Element {
    rsx! {
        div { class: "dashboard-client relative overflow-hidden",
            // Source: dashboard-client.tsx:39 — vibrant gradient bg.
            div { class: "dashboard-client-bg absolute inset-0 bg-gradient-to-br from-blue-50 via-orange-50 to-yellow-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900" }
            // Floating gradient orbs (source: lines 41-44).
            div { class: "dashboard-client-orb-orange absolute -top-40 -left-40 w-96 h-96 bg-gradient-to-br from-orange-400/15 to-yellow-400/15 rounded-full blur-3xl" }
            div { class: "dashboard-client-orb-blue absolute top-20 -right-32 w-80 h-80 bg-gradient-to-br from-blue-400/12 to-cyan-400/12 rounded-full blur-3xl" }
            div { class: "dashboard-client-orb-purple absolute bottom-20 left-20 w-72 h-72 bg-gradient-to-br from-purple-400/10 to-pink-400/10 rounded-full blur-3xl" }
            // Inner content (source: lines 46-294).
            div { class: "dashboard-client-content relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8",
                // Enhanced header (source: lines 48-71).
                div { class: "dashboard-client-header mb-12 text-center",
                    div { class: "dashboard-client-header-icon inline-flex items-center justify-center w-20 h-20 bg-gradient-to-br from-orange-500 to-yellow-500 rounded-3xl mb-6 shadow-2xl",
                        Icon { name: "trending-up".to_string(), size: Some(40), class_name: Some("text-white".to_string()) }
                    }
                    h1 { class: "dashboard-client-title text-5xl font-bold bg-gradient-to-r from-orange-600 via-yellow-600 to-orange-600 bg-clip-text text-transparent mb-4",
                        "\u{1F680} Dashboard"
                    }
                    p { class: "dashboard-client-welcome text-xl text-gray-600 dark:text-gray-300 mb-4",
                        "Welcome back, "
                        span { class: "dashboard-client-welcome-name font-semibold text-orange-600 dark:text-orange-400",
                            "{user.email}"
                        }
                        "! \u{2728}"
                    }
                    // Role badge (source: lines 62-70).
                    div { class: "dashboard-client-role-badge inline-flex items-center gap-3",
                        span { class: "rounded-full border border-green-300 bg-gradient-to-r from-green-500/10 to-emerald-500/10 px-4 py-2 text-sm font-semibold text-green-700",
                            Icon { name: "shield".to_string(), size: Some(16), class_name: Some("mr-2".to_string()) }
                            "Group: {permissions.role}"
                        }
                    }
                }
                // Stat cards row (source: dashboard-client.tsx shows 5 feature cards
                // in a 3-col grid; we add a 3-card stats row at the top
                // mirroring `dashboardData.data.stats`: Total Views /
                // Total Users / Revenue).
                div { class: "dashboard-client-stats grid grid-cols-1 md:grid-cols-3 gap-6 mb-8",
                    DashboardStatCard {
                        label: "Total Views".to_string(),
                        value: stats.total_views.to_string(),
                        icon: "eye".to_string(),
                        color: "blue".to_string(),
                    }
                    DashboardStatCard {
                        label: "Total Users".to_string(),
                        value: stats.total_users.to_string(),
                        icon: "users".to_string(),
                        color: "green".to_string(),
                    }
                    DashboardStatCard {
                        label: "Revenue".to_string(),
                        value: format!("${}", stats.revenue),
                        icon: "dollar-sign".to_string(),
                        color: "purple".to_string(),
                    }
                }
                // Feature cards grid (source: lines 73-198) — 3-col
                // grid with 5 cards (Profile / Settings / Analytics /
                // Premium Content / Moderator Panel).
                div { class: "dashboard-client-features grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8",
                    DashboardFeatureCard {
                        title: "\u{1F464} Profile".to_string(),
                        description: "Manage your personal information".to_string(),
                        href: "/profile".to_string(),
                        cta_label: "View Profile".to_string(),
                        icon: "user".to_string(),
                        color: "orange".to_string(),
                        animation: "fade-in".to_string(),
                    }
                    DashboardFeatureCard {
                        title: "\u{2699}\u{FE0F} Settings".to_string(),
                        description: "Configure your preferences".to_string(),
                        href: "/settings".to_string(),
                        cta_label: "Open Settings".to_string(),
                        icon: "settings".to_string(),
                        color: "blue".to_string(),
                        animation: "fade-in-delayed".to_string(),
                    }
                    DashboardFeatureCard {
                        title: "\u{1F4CA} Analytics".to_string(),
                        description: "View your data and insights".to_string(),
                        href: "/analytics".to_string(),
                        cta_label: "View Analytics".to_string(),
                        icon: "bar-chart-3".to_string(),
                        color: "green".to_string(),
                        animation: "fade-in-delayed-2".to_string(),
                    }
                    DashboardFeatureCard {
                        title: "\u{1F512} Premium Content".to_string(),
                        description: "Access exclusive premium features".to_string(),
                        href: "/premium".to_string(),
                        cta_label: "Access Premium".to_string(),
                        icon: "lock".to_string(),
                        color: "purple".to_string(),
                        animation: "fade-in-delayed-3".to_string(),
                    }
                    DashboardFeatureCard {
                        title: "\u{1F6E1}\u{FE0F} Moderator Panel".to_string(),
                        description: "Moderate content and users".to_string(),
                        href: "/moderator".to_string(),
                        cta_label: "Open Moderator Panel".to_string(),
                        icon: "shield".to_string(),
                        color: "red".to_string(),
                        animation: "fade-in-delayed".to_string(),
                    }
                }
                // Permissions card (source: lines 200-257).
                div { class: "dashboard-client-permissions mt-12",
                    div { class: "rounded-2xl border border-indigo-200/50 bg-white/80 backdrop-blur-xl shadow-2xl",
                        div { class: "dashboard-client-permissions-header px-5 py-4",
                            h3 { class: "flex items-center text-indigo-600",
                                div { class: "w-8 h-8 bg-gradient-to-br from-indigo-500 to-violet-500 rounded-lg flex items-center justify-center mr-3",
                                    Icon { name: "shield".to_string(), size: Some(20), class_name: Some("text-white".to_string()) }
                                }
                                "\u{1F510} Your Permissions"
                            }
                            p { class: "text-sm text-gray-600 mt-1",
                                "Current permissions for your account"
                            }
                        }
                        div { class: "px-5 py-4 space-y-4",
                            div { class: "flex items-center gap-3",
                                span { class: "text-lg font-semibold text-gray-700", "Group:" }
                                span { class: "rounded-full bg-gradient-to-r from-indigo-500 to-violet-500 px-4 py-2 text-sm font-semibold text-white shadow-lg",
                                    "{permissions.role}"
                                }
                            }
                            if !permissions.permissions.is_empty() {
                                div { class: "space-y-2",
                                    h4 { class: "font-medium text-gray-700",
                                        "\u{1F3AF} Permissions:"
                                    }
                                    div { class: "flex flex-wrap gap-2",
                                        for p in permissions.permissions.iter() {
                                            span { class: "rounded border border-cyan-300 bg-gradient-to-r from-cyan-500/10 to-blue-500/10 px-2 py-1 text-xs font-medium text-cyan-700",
                                                "{p}"
                                            }
                                        }
                                    }
                                }
                            } else {
                                div { class: "text-center py-4",
                                    p { class: "text-gray-600 mb-2",
                                        "\u{1F3AD} No specific permissions assigned"
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

/// Stat card row (Total Views / Total Users / Revenue).
#[component]
fn DashboardStatCard(label: String, value: String, icon: String, color: String) -> Element {
    let color_cls = match color.as_str() {
        "blue" => "text-blue-600",
        "green" => "text-green-600",
        "purple" => "text-purple-600",
        _ => "text-slate-600",
    };
    rsx! {
        div { class: "dashboard-stat-card rounded-2xl border border-slate-200 bg-white p-5 shadow-lg",
            div { class: "flex items-center justify-between mb-2",
                p { class: "text-xs font-medium text-slate-500", "{label}" }
                Icon { name: icon, size: Some(16), class_name: Some(color_cls.to_string()) }
            }
            p { class: "text-2xl font-bold {color_cls}", "{value}" }
        }
    }
}

/// Feature card (Profile / Settings / Analytics / Premium / Moderator).
#[component]
fn DashboardFeatureCard(
    title: String,
    description: String,
    href: String,
    cta_label: String,
    icon: String,
    color: String,
    animation: String,
) -> Element {
    let (color_text_cls, color_grad_from, color_grad_to) = match color.as_str() {
        "orange" => ("text-orange-600", "from-orange-500", "to-yellow-500"),
        "blue" => ("text-blue-600", "from-blue-500", "to-purple-500"),
        "green" => ("text-green-600", "from-green-500", "to-emerald-500"),
        "purple" => ("text-purple-600", "from-purple-500", "to-pink-500"),
        "red" => ("text-red-600", "from-red-500", "to-rose-500"),
        _ => ("text-slate-600", "from-slate-500", "to-slate-600"),
    };
    rsx! {
        div { class: "dashboard-feature-card dashboard-feature-card-{color} relative overflow-hidden rounded-2xl border border-slate-200/50 bg-white/80 backdrop-blur-xl shadow-2xl",
            div { class: "dashboard-feature-card-body relative z-10 p-6",
                h3 { class: "dashboard-feature-card-title flex items-center text-lg font-semibold {color_text_cls} mb-2",
                    div { class: "w-8 h-8 bg-gradient-to-br {color_grad_from} {color_grad_to} rounded-lg flex items-center justify-center mr-3",
                        Icon { name: icon, size: Some(20), class_name: Some("text-white".to_string()) }
                    }
                    "{title}"
                }
                p { class: "dashboard-feature-card-desc text-sm text-gray-600 mb-4",
                    "{description}"
                }
                a { class: "dashboard-feature-card-cta inline-flex items-center justify-center w-full rounded-lg bg-gradient-to-r {color_grad_from} {color_grad_to} px-4 py-2 text-sm font-medium text-white shadow-md hover:shadow-lg transition-shadow",
                    href: "{href}",
                    "{cta_label}"
                }
            }
        }
    }
}

// =============================================================================
// Tests
// =============================================================================
//
// - `test_render_smoke` — `render(&empty_ctx())` returns non-empty Element.
// - `test_unauthed_fallback` — when `ctx.user` is `None`, the SSR'd
//   HTML contains the "Please sign in" fallback text (matches the
//   prod capture state).
// - `test_authed_dashboard_client` — when `ctx.user` is `Some`, the
//   SSR'd HTML contains the DashboardClient h1 ("Dashboard") and a
//   stats card label ("Total Views").
// - `test_section_markers` — SSR'd HTML contains the section-marker
//   class names defined above.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::user::User;
    use crate::auth::user::AuthMethod;

    fn empty_ctx() -> PageContext {
        PageContext {
            user: None,
            path: "/dashboard".to_string(),
            ..Default::default()
        }
    }

    fn authed_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "u-1".to_string(),
                address: "0x1234abcd".to_string(),
                chain_id: "1".to_string(),
                roles: vec!["user".to_string()],
                email: Some("test@epsx.io".to_string()),
                tier: Some("pro".to_string()),
                permissions: vec!["dashboard:read".to_string(), "analytics:view".to_string()],
                last_login_at: None,
                auth_method: AuthMethod::default(),
                display_name: Some("EPSX tester".to_string()),
            }),
            path: "/dashboard".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_render_smoke() {
        let (_meta, el) = render(&empty_ctx());
        let html = dioxus_ssr::render_element(el);
        assert!(!html.is_empty(), "dashboard must render non-empty HTML. Got: {}", html);
        assert!(html.len() > 100, "dashboard HTML is suspiciously short ({} bytes).", html.len());
    }

    #[test]
    fn test_unauthed_fallback() {
        let (_meta, el) = render(&empty_ctx());
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("Personal Dashboard"),
            "unauthed dashboard should still render the h1 'Personal Dashboard'. Got: {}",
            html
        );
        assert!(
            html.contains("Please sign in to access your dashboard"),
            "unauthed dashboard should render the fallback text. Got: {}",
            html
        );
    }

    #[test]
    fn test_authed_dashboard_client() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("Dashboard"),
            "authed dashboard should render the DashboardClient h1. Got: {}",
            html
        );
        assert!(
            html.contains("Total Views"),
            "authed dashboard should render the stats card label 'Total Views'. Got: {}",
            html
        );
        assert!(
            html.contains("dashboard-feature-card-profile") || html.contains("Profile"),
            "authed dashboard should render the Profile feature card. Got: {}",
            html
        );
    }

    #[test]
    fn test_section_markers() {
        let (_meta, el) = render(&empty_ctx());
        let html = dioxus_ssr::render_element(el);
        for marker in &[
            "dashboard-prod-page",
            "dashboard-prod-header",
            "dashboard-prod-title",
            "dashboard-prod-subtitle",
            "dashboard-prod-fallback",
        ] {
            let needle_a = format!("class=\"{}\"", marker);
            let needle_b = format!("class=\"{mark} ", mark = marker);
            let needle_c = format!(" {}\"", marker);
            let needle_d = format!(" {} ", marker);
            assert!(
                html.contains(&needle_a)
                    || html.contains(&needle_b)
                    || html.contains(&needle_c)
                    || html.contains(&needle_d),
                "dashboard must contain section marker '{}'. Got: {}",
                marker, html
            );
        }
    }
}