//! /admin — Command Center (admin dashboard).
//!
//! Wave 6C Track B — 1:1 component parity with the Next.js source
//! `apps-old/admin-frontend/app/dashboard-client.tsx` + the 4
//! sub-components in `apps-old/admin-frontend/components/admin/`:
//! - `dashboard-pulse-header.tsx` (70 LoC) — `AdminPulseHeader`
//! - `dashboard-hud-metrics.tsx` (80 LoC) — `AdminStatsCards`
//! - `dashboard-bento-tools.tsx` (121 LoC) — `WalletsByChain`
//!   (the dashboard's wallet-distribution row)
//! - `dashboard-activity-stream.tsx` (140 LoC) — `ActivityStream`
//!
//! Wave 6B's section-level parity is preserved. The page now
//! composes the extracted sub-components from
//! `crate::components::admin::dashboard::*` instead of inlining
//! the JSX. See `components/admin/dashboard.rs` for the
//! sub-component bodies.
//!
//! Section markers (asserted by `test_section_markers`):
//! - `admin-stats-cards`
//! - `wallets-by-chain`
//! - `recent-transactions`
//! - `system-alerts`
//! - `activity-stream`
//! (the `admin-pulse-header` marker is preserved too).

use crate::auth::AdminAuthGate;
use crate::components::admin::dashboard::{
    ActivityStream, AdminPulseHeader, AdminStatsCards, RecentTransactions, SystemAlerts,
    WalletsByChain,
};
use crate::layout::admin_shell::AdminShell;

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Command Center");
    (meta, rsx! { RenderDashboard { ctx: ctx.clone() } })
}

#[component]
fn RenderDashboard(ctx: PageContext) -> Element {
    rsx! {
        AdminAuthGate { user: ctx.user.clone(), feature: Some("the admin dashboard".to_string()), required_permissions: Some(vec!["admin:*".to_string()]), return_url: Some(ctx.path.clone()),
            AdminShell {
                ctx: ctx.clone(),
                page_title: "Command Center".to_string(),
                breadcrumbs: vec![
                    ("Dashboard".to_string(), "/".to_string()),
                    ("Command Center".to_string(), "/".to_string()),
                ],
                div { class: "container page-content admin-dashboard",
                    // Command Center pulse header (mirrors
                    // `dashboard-pulse-header.tsx`).
                    AdminPulseHeader {}
                    // 5-up stat row (`AdminStatsCards`).
                    AdminStatsCards {}
                    // `WalletsByChain` — donut + per-chain legend.
                    WalletsByChain {}
                    // `ActivityStream` + `RecentTransactions` + `SystemAlerts` in a 3-col grid.
                    div { class: "grid grid-cols-1 lg:grid-cols-3 gap-4 mt-6",
                        div { class: "lg:col-span-1",
                            ActivityStream {}
                        }
                        div { class: "lg:col-span-2",
                            RecentTransactions {}
                            div { class: "mt-4",
                                SystemAlerts {}
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
    use crate::auth::user::{AuthMethod, User};

    /// Build an authenticated admin `PageContext` with the
    /// `admin:*` permission the gate checks.
    fn admin_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "u-admin".to_string(),
                address: "0x1234abcd5678ef90".to_string(),
                chain_id: "56".to_string(),
                roles: vec!["admin".to_string()],
                email: Some("admin@epsx.io".to_string()),
                tier: Some("Admin".to_string()),
                permissions: vec!["admin:*".to_string()],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: Some("Admin".to_string()),
            }),
            path: "/".to_string(),
            ..Default::default()
        }
    }

    /// Smoke test: `render()` returns a non-empty Element.
    #[test]
    fn test_render_smoke() {
        let ctx = admin_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(!html.trim().is_empty(), "dashboard must render non-empty HTML. Got: {html}");
        assert!(html.len() > 100, "dashboard HTML is suspiciously short ({} bytes).", html.len());
    }

    /// Section-marker test: the rendered HTML must contain every
    /// design-doc-named section.
    #[test]
    fn test_section_markers() {
        let ctx = admin_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        for marker in &[
            "admin-pulse-header",
            "admin-stats-cards",
            "wallets-by-chain",
            "recent-transactions",
            "system-alerts",
            "activity-stream",
        ] {
            // 4-form matcher (single class, first/middle/last of a
            // class list).
            let needle_a = format!("class=\"{}\"", marker);
            let needle_b = format!("class=\"{mark} ", mark = marker);
            let needle_c = format!(" {}\"", marker);
            let needle_d = format!(" {} ", marker);
            assert!(
                html.contains(&needle_a)
                    || html.contains(&needle_b)
                    || html.contains(&needle_c)
                    || html.contains(&needle_d),
                "dashboard must contain section marker `{marker}`. Got: {html}"
            );
        }
    }
}
