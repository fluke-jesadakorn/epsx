//! /admin/wallet-management/access — wallet access control manager.
//!
//! Wave 6C Track C — thin composition of the 4 named sub-components
//! extracted into `crate::components::admin::wallet`. The 4 sub-
//! components (WalletAccessManager, PlanSelectorModal, AccessGrantForm,
//! AccessRevokeDialog) live in `components/admin/wallet.rs`.

use crate::auth::AdminAuthGate;
use crate::components::admin::wallet::WalletAccessManager;
use crate::primitives::*;

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};

// ============================================================================
// Page entry
// ============================================================================

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Access control");
    (meta, rsx! {
        AdminAuthGate { user: ctx.user.clone(), feature: Some("access control".to_string()), required_permissions: Some(vec!["wallets:manage".to_string()]), return_url: Some(ctx.path.clone()),
            div { class: "container page-content",
                // Page header
                div { class: "flex items-center justify-between mb-6",
                    div {
                        h1 { class: "text-2xl font-bold", "Wallet access manager" }
                        p { class: "text-muted-foreground", "Manage per-wallet access to plans and permissions" }
                    }
                    button { class: "btn btn-primary", r#type: "button",
                        Icon { name: "user-check".to_string(), size: Some(16) }
                        " Grant access"
                    }
                }
                WalletAccessManager {}
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
            path: "/wallet-management/access".to_string(),
            ..Default::default()
        }
    }

    /// `test_render_smoke`. The page renders non-empty HTML when the
    /// admin is authed and holds `wallets:manage`.
    #[test]
    fn test_render_smoke() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        assert!(!html.is_empty(), "wallet_access page must render non-empty HTML. Got: {}", html);
        assert!(html.contains("Wallet access manager"), "wallet_access page must contain the title. Got: {}", html);
    }

    /// `test_section_markers`. The default page exposes the manager
    /// chrome. The plan selector modal, access grant form, and revoke
    /// dialog are siblings in the same file (their markers are present
    /// in the source but they live behind user interaction — the
    /// modal is hidden, the forms are revealed on click). We assert
    /// the manager marker is visible by default.
    #[test]
    fn test_section_markers() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        for marker in &[
            "wallet-access-manager",
        ] {
            assert!(
                html.contains(marker),
                "wallet_access page should contain section marker `{marker}`. Got: {html}"
            );
        }
    }
}
