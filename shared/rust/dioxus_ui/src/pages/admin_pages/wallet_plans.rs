//! /admin/wallet-management/access + /admin/wallet-management/access/plans + /[planId].
//!
//! Wave 6C Track C — thin composition of the 6 named sub-components
//! extracted into `crate::components::admin::wallet`. The 6 sub-
//! components (PlanListSidebar, PlanEditorPage, PlanEditorDrawer,
//! PlanApiLimits, PlanPromotions, PlanItemCard) live in
//! `components/admin/wallet.rs` along with the `Plan` struct +
//! `sample_plans()` helper (formerly inline in this file).

use crate::auth::AdminAuthGate;
use crate::components::admin::wallet::{
    sample_plans, PlanEditorDrawer, PlanEditorPage, PlanListSidebar,
};
use crate::data_table::{Column, DataTable, Row, SortDir};
use crate::primitives::*;

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};

// ============================================================================
// Page: /wallet-management/access — access control hub
// ============================================================================

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Access control");
    (meta, rsx! {
        AdminAuthGate {
            user: ctx.user.clone(),
            feature: Some("access control".to_string()),
            required_permissions: Some(vec!["wallets:manage".to_string()]),
            return_url: Some(ctx.path.clone()),
            div { class: "container page-content",
                div { class: "mb-6",
                    h1 { class: "text-2xl font-bold", "Access control" }
                    p { class: "text-muted-foreground", "Manage wallet permissions and access plans" }
                }
                div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 mb-6",
                    StatCard { label: "Total wallets".to_string(), value: "1,234".to_string(), icon: Some("users".to_string()) }
                    StatCard { label: "Active plans".to_string(), value: "5".to_string(), icon: Some("zap".to_string()) }
                    StatCard { label: "Permissions".to_string(), value: "12".to_string(), icon: Some("key".to_string()) }
                }
                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                    a { class: "card card-glass p-6", href: "/wallet-management/access/plans",
                        Icon { name: "list".to_string(), size: Some(32) }
                        h3 { class: "text-xl font-bold mt-3", "Plans" }
                        p { class: "text-muted-foreground mt-1", "Subscription plans with permissions, pricing, and billing intervals" }
                    }
                    a { class: "card card-glass p-6", href: "/wallet-management/wallets",
                        Icon { name: "users".to_string(), size: Some(32) }
                        h3 { class: "text-xl font-bold mt-3", "Wallets" }
                        p { class: "text-muted-foreground mt-1", "Wallet-by-wallet permission management" }
                    }
                }
            }
        }
    })
}

// ============================================================================
// Page: /wallet-management/access/plans — plan list + drawer
// ============================================================================

pub fn render_plans(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Plans");
    let columns = vec![
        Column { key: "name".into(), label: "Name".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("25%".into()), class_name: None },
        Column { key: "price".into(), label: "Price".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("15%".into()), class_name: None },
        Column { key: "interval".into(), label: "Interval".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("15%".into()), class_name: None },
        Column { key: "subs".into(), label: "Subscribers".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("15%".into()), class_name: None },
        Column { key: "status".into(), label: "Status".into(), sortable: true, align: crate::primitives::data_table::Align::Center, width: Some("15%".into()), class_name: None },
        Column { key: "actions".into(), label: "Actions".into(), sortable: false, align: crate::primitives::data_table::Align::Right, width: Some("15%".into()), class_name: None },
    ];
    let rows = vec![
        Row { id: "1".into(), cells: vec!["Free".into(), "$0".into(), "monthly".into(), "—".into(), "Active".into(), "Edit".into()] },
        Row { id: "2".into(), cells: vec!["Pro".into(), "$29".into(), "monthly".into(), "412".into(), "Active".into(), "Edit".into()] },
        Row { id: "3".into(), cells: vec!["Enterprise".into(), "$299".into(), "monthly".into(), "32".into(), "Active".into(), "Edit".into()] },
        Row { id: "4".into(), cells: vec!["Whale".into(), "$999".into(), "monthly".into(), "5".into(), "Active".into(), "Edit".into()] },
    ];
    (meta, rsx! {
        AdminAuthGate { user: ctx.user.clone(), feature: Some("plan management".to_string()), required_permissions: Some(vec!["wallets:manage".to_string()]), return_url: Some(ctx.path.clone()),
            div { class: "container page-content",
                div { class: "flex items-center justify-between mb-6",
                    div {
                        h1 { class: "text-2xl font-bold", "Plans" }
                        p { class: "text-muted-foreground", "Subscription plans with permissions and pricing" }
                    }
                    a { class: "btn btn-primary", href: "/wallet-management/access/plans/new",
                        Icon { name: "plus".to_string(), size: Some(16) }
                        " New plan"
                    }
                }
                PlanListSidebar { plans: sample_plans() }
                // The drawer shell — when a plan is selected the
                // PlanEditorDrawer slides in from the right
                PlanEditorDrawer {}
                // The list (DataTable) view is a parallel view used
                // when sidebar is collapsed on smaller screens.
                div { class: "mt-6",
                    DataTable { columns, rows, striped: true, page_size: 20, filter_placeholder: Some("Filter by name, status...".to_string()), initial_sort: Some(("price".to_string(), SortDir::Asc)) }
                }
            }
        }
    })
}

// ============================================================================
// Page: /wallet-management/access/plans/[planId] — full-page editor
// ============================================================================

pub fn render_editor(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Plan editor");
    let plan_id = ctx.params.get("planId").cloned();
    (meta, rsx! {
        AdminAuthGate { user: ctx.user.clone(), feature: Some("plan editing".to_string()), required_permissions: Some(vec!["wallets:manage".to_string()]), return_url: Some(ctx.path.clone()),
            div { class: "container page-content max-w-3xl",
                a { class: "btn btn-sm btn-ghost mb-4", href: "/wallet-management/access/plans",
                    Icon { name: "arrow-left".to_string(), size: Some(16) }
                    " Back"
                }
                PlanEditorPage { plan_id: plan_id.unwrap_or_default() }
            }
        }
    })
}

// ============================================================================
// Tests — Wave 6C Track C
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::User;
    use crate::auth::user::AuthMethod;
    use crate::pages::PageContext;

    fn authed_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "admin-1".to_string(),
                address: "0xadmin".to_string(),
                chain_id: "56".to_string(),
                roles: vec!["admin".to_string()],
                email: Some("admin@epsx.io".to_string()),
                tier: Some("Admin".to_string()),
                permissions: vec!["wallets:manage".to_string()],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: Some("Admin".to_string()),
            }),
            path: "/wallet-management/access/plans".to_string(),
            ..Default::default()
        }
    }

    fn editor_ctx() -> PageContext {
        let mut ctx = authed_ctx();
        ctx.path = "/wallet-management/access/plans/pro".to_string();
        ctx.params.insert("planId".to_string(), "pro".to_string());
        ctx
    }

    /// `test_render_smoke`. The plans list page renders non-empty HTML
    /// when the admin is authed and holds `wallets:manage`.
    #[test]
    fn test_render_smoke() {
        let (_meta, el) = render_plans(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        assert!(!html.is_empty(), "wallet_plans page must render non-empty HTML. Got: {}", html);
        assert!(html.contains("Plans"), "wallet_plans page must contain the plans title. Got: {}", html);
    }

    /// `test_section_markers`. The plans list page contains the
    /// sidebar (always visible) and the plan item card. The
    /// editor-only sections are on the `[planId]` route, tested
    /// separately below.
    #[test]
    fn test_section_markers() {
        let (_meta, el) = render_plans(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        for marker in &[
            "plan-list-sidebar",
            "plan-item-card",
        ] {
            assert!(
                html.contains(marker),
                "wallet_plans page should contain section marker `{marker}`. Got: {html}"
            );
        }
    }

    /// The `[planId]` route renders the full editor with the page,
    /// api-limits, and promotions sub-components.
    #[test]
    fn test_section_markers_editor() {
        let (_meta, el) = render_editor(&editor_ctx());
        let html = dioxus_ssr::render_element(el);
        for marker in &[
            "plan-editor-page",
            "plan-api-limits",
            "plan-promotions",
        ] {
            assert!(
                html.contains(marker),
                "wallet_plans editor page should contain section marker `{marker}`. Got: {html}"
            );
        }
    }

    /// The plans list page renders the sidebar + the drawer chrome.
    #[test]
    fn test_section_markers_list_includes_drawer() {
        let (_meta, el) = render_plans(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("plan-editor-drawer"),
            "wallet_plans list page should contain plan-editor-drawer marker. Got: {html}"
        );
    }
}
