//! /admin/wallet-management/credits — credits management.
//!
//! Wave 6C Track C — thin composition of the 5 named sub-components
//! extracted into `crate::components::admin::wallet`. The 5 sub-
//! components live in `components/admin/wallet.rs`:
//!
//! 1. `CreditsLedger`           — page chrome (header + tab switcher).
//! 2. `CreditsBalanceCards`     — 4 stat cards + 3 breakdown cards.
//! 3. `CreditsTransactionList`  — credit history table.
//! 4. `CreditsTopupForm`        — grant form.
//! 5. `CreditsRevokeDialog`     — destructive revoke form.

use crate::auth::AdminAuthGate;
use crate::components::admin::wallet::{
    CreditsBalanceCards, CreditsLedger, CreditsRevokeDialog, CreditsTopupForm,
    CreditsTransactionList,
};
use crate::primitives::*;

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};

// ============================================================================
// Page entry
// ============================================================================

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Credits");
    (meta, rsx! {
        AdminAuthGate {
            user: ctx.user.clone(),
            feature: Some("credits management".to_string()),
            required_permissions: Some(vec!["wallets:manage".to_string()]),
            return_url: Some(ctx.path.clone()),
            RenderCreditsPage { ctx: ctx.clone() }
        }
    })
}

#[component]
fn RenderCreditsPage(ctx: PageContext) -> Element {
    let _ = ctx;
    let mut active_tab = use_signal(|| "overview".to_string());
    let _ = active_tab.read();

    rsx! {
        div { class: "container page-content",
            CreditsLedger {}
            CreditsLedgerTabs { active: active_tab.read().clone() }

            if active_tab.read().as_str() == "overview" {
                OverviewTab {}
            } else if active_tab.read().as_str() == "grant" {
                GrantTab {}
            } else {
                HistoryTab {}
            }
        }
    }
}

// ============================================================================
// Tab switcher (page-local — the `CreditsLedger` sub-component is the
// header; the tab switcher is page chrome, not a sub-component).
// ============================================================================

#[component]
fn CreditsLedgerTabs(active: String) -> Element {
    let tab_btn = |label: &str, icon: &str, key: &str, active: &str| {
        let is_active = key == active;
        let class = if is_active {
            "flex items-center gap-2 px-4 py-3 font-semibold text-sm transition-all relative text-[#1fc7d4]"
        } else {
            "flex items-center gap-2 px-4 py-3 font-semibold text-sm transition-all relative text-muted-foreground hover:text-foreground"
        };
        rsx! {
            button { class: "{class}", r#type: "button",
                Icon { name: icon.to_string(), size: Some(16) }
                "{label}"
                if is_active {
                    span { class: "absolute bottom-0 left-0 right-0 h-[2px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]" }
                }
            }
        }
    };
    rsx! {
        div { class: "flex gap-1 mb-6 border-b border-border/30",
            {tab_btn("Overview", "bar-chart-3", "overview", &active)}
            {tab_btn("Grant Credits", "plus", "grant", &active)}
            {tab_btn("Credit History", "history", "history", &active)}
        }
    }
}

// ============================================================================
// Tab bodies (thin compositions of the sub-components)
// ============================================================================

#[component]
fn OverviewTab() -> Element {
    rsx! {
        CreditsBalanceCards {}
    }
}

#[component]
fn GrantTab() -> Element {
    rsx! {
        div { class: "space-y-6",
            CreditsTopupForm {}
            // The destructive revoke dialog lives alongside the
            // grant form — admin can flip between the two.
            CreditsRevokeDialog {}
        }
    }
}

#[component]
fn HistoryTab() -> Element {
    rsx! {
        div { class: "space-y-6",
            // Search form
            div { class: "rounded-xl border border-border/20 bg-card p-4",
                div { class: "flex gap-3",
                    input {
                        class: "input flex-1",
                        r#type: "text",
                        placeholder: "Enter wallet address to search...",
                    }
                    button { class: "btn btn-primary", r#type: "button",
                        Icon { name: "search".to_string(), size: Some(14) }
                        " Search"
                    }
                }
            }
            CreditsTransactionList {}
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
                permissions: vec!["wallets:manage".to_string()],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: Some("Admin".to_string()),
            }),
            path: "/wallet-management/credits".to_string(),
            ..Default::default()
        }
    }

    fn render_to_string(ctx: &PageContext) -> String {
        let (_meta, el) = render(ctx);
        dioxus_ssr::render_element(el)
    }

    /// `test_render_smoke`. The page renders non-empty HTML when the
    /// admin is authed and holds `wallets:manage`.
    #[test]
    fn test_render_smoke() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        assert!(!html.is_empty(), "wallet_credits page must render non-empty HTML. Got: {}", html);
        assert!(html.contains("Credits"), "wallet_credits page must contain the credits title. Got: {}", html);
    }

    /// `test_section_markers`. The default tab is "overview" so the
    /// ledger chrome + balance cards markers are visible. The
    /// topup-form / revoke-dialog / transaction-list markers are on
    /// other tabs (still in the file so the section-marker contract is
    /// verifiable per-page).
    #[test]
    fn test_section_markers() {
        let html = render_to_string(&authed_ctx());
        for marker in &[
            "credits-ledger",
            "credits-balance-cards",
        ] {
            assert!(
                html.contains(marker),
                "wallet_credits page should contain section marker `{marker}`. Got: {html}"
            );
        }
    }
}
