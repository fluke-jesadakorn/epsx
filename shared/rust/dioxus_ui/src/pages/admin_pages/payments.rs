//! /admin/payments — payment management (Payments Hub).
//!
//! Wave 6C Track C — thin composition of the 6 named sub-components
//! extracted into `crate::components::admin::payments`. The 6 sub-
//! components (with their `// === wave6b-admin-pages-depth-track-c
//! <marker> ===` comments) live in `components/admin/payments.rs`:
//!
//! 1. `PaymentLinkStats`       — top-of-page stat cards.
//! 2. `PaymentsFilterPanel`    — search + status + method + plan filter bar.
//! 3. `PaymentLinksList`       — main payment-intent table.
//! 4. `AccessManagementList`   — users with active plan access.
//! 5. `CreateLinkForm`         — new payment link form.
//! 6. `LinkRevokeConfirm`      — destructive revoke confirm dialog.
//!
//! The page itself retains only the page-entry shell (auth gate +
//! header + tab switcher) and the per-tab body wrappers; the
//! inlined JSX moved to the sub-component file. The
//! `test_section_markers` test still asserts the 3 default-tab
//! markers (`payments-stats`, `payments-filter-panel`,
//! `payment-links-list`) so the section-marker contract is preserved.

use crate::auth::AdminAuthGate;
use crate::components::admin::payments::{
    AccessManagementList, CreateLinkForm, LinkRevokeConfirm, PaymentLinkStats,
    PaymentLinksList, PaymentsFilterPanel,
};
use crate::primitives::*;

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};

// ============================================================================
// Page entry
// ============================================================================

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Payments");
    (meta, rsx! {
        AdminAuthGate {
            user: ctx.user.clone(),
            feature: Some("payment management".to_string()),
            required_permissions: Some(vec!["payments:manage".to_string()]),
            return_url: Some(ctx.path.clone()),
            RenderPaymentsHub { ctx: ctx.clone() }
        }
    })
}

#[component]
fn RenderPaymentsHub(ctx: PageContext) -> Element {
    let _ = ctx;
    let mut active_tab = use_signal(|| "payments".to_string());
    let _ = active_tab.read();

    rsx! {
        div { class: "container page-content",
            // Page header
            div { class: "flex items-center justify-between mb-6",
                div {
                    h1 { class: "text-2xl font-bold", "Payments Hub" }
                    p { class: "text-muted-foreground", "Manage payments, user access, and payment links" }
                }
                div { class: "flex gap-2",
                    button { class: "btn btn-sm btn-outline", r#type: "button",
                        Icon { name: "refresh-cw".to_string(), size: Some(14) }
                        " Refresh"
                    }
                    button { class: "btn btn-sm btn-outline", r#type: "button",
                        Icon { name: "bar-chart-3".to_string(), size: Some(14) }
                        " Export CSV"
                    }
                }
            }

            // Tab switcher
            PaymentsHubTabs { active: active_tab.read().clone() }

            if active_tab.read().as_str() == "payments" {
                PaymentsTab {}
            } else if active_tab.read().as_str() == "user-access" {
                UserAccessTab {}
            } else {
                PaymentLinksTab {}
            }
        }
    }
}

// ============================================================================
// Tab switcher
// ============================================================================

#[component]
fn PaymentsHubTabs(active: String) -> Element {
    let tab_btn = |label: &str, key: &str, active: &str| rsx! {
        button {
            class: if active == key { "btn btn-primary btn-sm" } else { "btn btn-outline btn-sm" },
            r#type: "button",
            "{label}"
        }
    };
    rsx! {
        div { class: "flex gap-2 mb-6",
            {tab_btn("Payments", "payments", &active)}
            {tab_btn("User Access", "user-access", &active)}
            {tab_btn("Payment Links", "payment-links", &active)}
        }
    }
}

// ============================================================================
// Tab bodies (thin compositions of the sub-components)
// ============================================================================

#[component]
fn PaymentsTab() -> Element {
    rsx! {
        div { class: "space-y-6",
            PaymentLinkStats {}
            PaymentsFilterPanel {}
            PaymentLinksList {}
        }
    }
}

#[component]
fn UserAccessTab() -> Element {
    rsx! {
        div { class: "space-y-6",
            // For the user-access tab we re-use the stats with a
            // different label set (active users instead of total revenue).
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 mb-6",
                StatCard { label: "Users with access".to_string(), value: "412".to_string(), icon: Some("users".to_string()) }
                StatCard { label: "Expiring in 7 days".to_string(), value: "8".to_string(), icon: Some("clock".to_string()) }
                StatCard { label: "Expired this month".to_string(), value: "3".to_string(), icon: Some("alert-circle".to_string()) }
            }
            AccessManagementList {}
        }
    }
}

#[component]
fn PaymentLinksTab() -> Element {
    rsx! {
        div { class: "space-y-6",
            // Quick filter / actions row
            div { class: "rounded-xl border border-border/20 bg-card p-4 mb-6",
                div { class: "grid grid-cols-1 sm:grid-cols-3 gap-4",
                    div {
                        label { class: "block text-xs font-bold text-muted-foreground uppercase tracking-[0.15em] mb-2", "Context Type" }
                        select { class: "input",
                            option { value: "", "All Types" }
                            option { value: "plan", "Plan" }
                            option { value: "group", "Group" }
                            option { value: "product", "Product" }
                            option { value: "campaign", "Campaign" }
                            option { value: "custom", "Custom" }
                        }
                    }
                    div {
                        label { class: "block text-xs font-bold text-muted-foreground uppercase tracking-[0.15em] mb-2", "Status" }
                        select { class: "input",
                            option { value: "", "All Status" }
                            option { value: "true", "Active" }
                            option { value: "false", "Inactive" }
                        }
                    }
                    div { class: "flex items-end",
                        button { class: "btn btn-outline w-full", r#type: "button", "Reset" }
                    }
                }
            }
            // Two-column layout: form on the left, list on the right
            div { class: "grid grid-cols-1 lg:grid-cols-2 gap-6",
                CreateLinkForm {}
                PaymentLinksList {}
            }
            LinkRevokeConfirm {}
        }
    }
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
                permissions: vec!["payments:manage".to_string()],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: Some("Admin".to_string()),
            }),
            path: "/payments".to_string(),
            ..Default::default()
        }
    }

    fn render_to_string(ctx: &PageContext) -> String {
        let (_meta, el) = render(ctx);
        dioxus_ssr::render_element(el)
    }

    /// `test_render_smoke`. The page renders non-empty HTML when the
    /// admin is authed and holds `payments:manage`.
    #[test]
    fn test_render_smoke() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        assert!(!html.is_empty(), "payments page must render non-empty HTML. Got: {}", html);
        assert!(html.contains("Payments Hub"), "payments page must contain the hub title. Got: {}", html);
    }

    /// `test_section_markers`. The default tab is "payments", so the
    /// 3 default-tab markers are present. The create-link-form and
    /// link-revoke-confirm live on the Payment Links tab (rendered
    /// when the user clicks that tab).
    #[test]
    fn test_section_markers() {
        let html = render_to_string(&authed_ctx());
        for marker in &[
            "payments-stats",
            "payments-filter-panel",
            "payment-links-list",
        ] {
            assert!(
                html.contains(marker),
                "payments page should contain section marker `{marker}`. Got: {html}"
            );
        }
    }
}
