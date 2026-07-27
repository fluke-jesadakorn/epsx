//! `/wallet-management/access` — truthful wallet-access availability.
//!
//! The legacy page loaded permission plans and exposed plan and permission
//! editing. The Rust admin BFF has no selected backend-authoritative read or
//! mutation contract for this route, so this leaf fails closed. A verified
//! session may see the explicit unavailable state; signed-out visitors see
//! only the session gate. Authorization and access policy remain backend-owned.

use dioxus::prelude::*;

use crate::auth::AuthGate;
use crate::primitives::Icon;

use super::super::{PageContext, PageMeta};

const WALLET_ACCESS_PATH: &str = "/wallet-management/access";
const ADMIN_HOME_PATH: &str = "/";

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Wallet access");

    // Query and route parameters are deliberately ignored. Only a future
    // backend-owned response may establish plans, permissions, assignments,
    // or authorization to mutate them.
    (
        meta,
        rsx! {
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("the private wallet access workspace".to_string()),
                return_url: Some(WALLET_ACCESS_PATH.to_string()),
                WalletAccessUnavailable {}
            }
        },
    )
}

/// This leaf owns route content only. The admin BFF supplies the single
/// authenticated layout, so rendering another shell or `<main>` here would
/// duplicate document structure.
#[component]
fn WalletAccessUnavailable() -> Element {
    rsx! {
        div {
            class: "container page-content admin-wallet-access py-8",
            "data-admin-wallet-access-state": "unavailable",
            section {
                class: "relative overflow-hidden rounded-3xl border border-border/40 bg-card shadow-2xl",
                role: "status",
                aria_labelledby: "admin-wallet-access-unavailable-title",
                div {
                    class: "absolute inset-x-0 top-0 h-1 bg-gradient-to-r from-[#1fc7d4] via-[#7645d9] to-[#ed4b9e]",
                    aria_hidden: "true",
                }
                div { class: "p-8 md:p-12",
                    div { class: "flex flex-col gap-6 sm:flex-row sm:items-start",
                        div {
                            class: "flex h-16 w-16 shrink-0 items-center justify-center rounded-2xl border border-violet-500/20 bg-violet-500/10 text-violet-400",
                            aria_hidden: "true",
                            Icon { name: "shield".to_string(), size: Some(30) }
                        }
                        div { class: "min-w-0 max-w-3xl",
                            p { class: "text-xs font-black uppercase tracking-[0.22em] text-violet-400",
                                "Wallet access"
                            }
                            h1 {
                                id: "admin-wallet-access-unavailable-title",
                                class: "mt-3 text-3xl font-black tracking-tight text-foreground",
                                "Wallet access data is unavailable"
                            }
                            p { class: "mt-4 text-sm leading-6 text-muted-foreground",
                                "Plan assignments and permission definitions are not shown because a verified wallet-access read contract is not connected. Missing data is not presented as an empty access list."
                            }
                            p { class: "mt-3 text-sm leading-6 text-muted-foreground",
                                "Access changes remain unavailable until the backend supplies authoritative policy checks, validation, and auditable operations."
                            }
                            nav {
                                class: "mt-8 flex flex-wrap gap-3",
                                aria_label: "Wallet access recovery",
                                a {
                                    class: "btn btn-primary",
                                    href: WALLET_ACCESS_PATH,
                                    "Retry"
                                }
                                a {
                                    class: "btn btn-outline",
                                    href: ADMIN_HOME_PATH,
                                    "Admin home"
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
    use crate::auth::user::{AuthMethod, User};

    fn authenticated_empty_claims_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "verified-session".to_string(),
                address: "0xsession".to_string(),
                chain_id: "56".to_string(),
                roles: vec![],
                email: None,
                tier: None,
                permissions: vec![],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: None,
            }),
            path: WALLET_ACCESS_PATH.to_string(),
            ..Default::default()
        }
    }

    fn render_html(ctx: &PageContext) -> String {
        let (_, element) = render(ctx);
        dioxus_ssr::render_element(element)
    }

    fn assert_no_sample_access_or_controls(html: &str) {
        let lowered = html.to_ascii_lowercase();
        for forbidden in [
            ">pro<",
            ">enterprise<",
            ">whale<",
            "api starter",
            "api pro",
            ">free<",
            "admin:permissions:read",
            "admin:permissions:manage",
            "available plans",
            "authorized plans",
            "grant access",
            "revoke access",
            "apply changes",
            "discard changes",
            "refresh",
            "bulk assign",
            "bulk remove",
            "plan-selector-modal",
            "access-grant-form",
            "access-revoke-dialog",
            "wallet-access-manager",
            "<form",
            "<input",
            "<textarea",
            "<select",
            "<button",
        ] {
            assert!(
                !lowered.contains(&forbidden.to_ascii_lowercase()),
                "wallet access must not render sample state or control `{forbidden}`. Got: {html}"
            );
        }
    }

    #[test]
    fn signed_out_render_keeps_private_access_state_hidden() {
        let mut ctx = PageContext {
            path: WALLET_ACCESS_PATH.to_string(),
            query: "planId=private-plan&action=grant".to_string(),
            ..Default::default()
        };
        ctx.params
            .insert("wallet".to_string(), "0xPRIVATE_WALLET".to_string());

        let html = render_html(&ctx);

        assert!(html.contains("Sign in required"));
        assert!(html.contains("href=\"/auth?return_url=%2Fwallet-management%2Faccess\""));
        assert!(!html.contains("data-admin-wallet-access-state"));
        assert!(!html.contains("Wallet access data is unavailable"));
        assert!(!html.contains("private-plan"));
        assert!(!html.contains("0xPRIVATE_WALLET"));
        assert_no_sample_access_or_controls(&html);
    }

    #[test]
    fn authenticated_empty_role_session_reaches_unavailable_state() {
        let html = render_html(&authenticated_empty_claims_ctx());

        assert!(html.contains("data-admin-wallet-access-state=\"unavailable\""));
        assert!(html.contains("role=\"status\""));
        assert!(html.contains("Wallet access data is unavailable"));
        assert!(html.contains("Missing data is not presented as an empty access list"));
        assert!(!html.contains("Permission required"));
        assert!(!html.contains("Admin access required"));
        assert_no_sample_access_or_controls(&html);
    }

    #[test]
    fn samples_and_grant_revoke_apply_refresh_bulk_modal_forms_are_absent() {
        let html = render_html(&authenticated_empty_claims_ctx());

        assert_no_sample_access_or_controls(&html);
        assert_eq!(html.matches("<a ").count(), 2);
        assert!(html.contains("href=\"/wallet-management/access\">Retry</a>"));
        assert!(html.contains("href=\"/\">Admin home</a>"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("javascript:"));
    }

    #[test]
    fn hostile_params_and_query_are_ignored() {
        let mut ctx = authenticated_empty_claims_ctx();
        ctx.query =
            "planId=HOSTILE_PLAN&permission=admin%3Apermissions%3Amanage&action=apply".to_string();
        ctx.params.insert(
            "wallet".to_string(),
            "HOSTILE_WALLET\"><script>alert('access')</script>".to_string(),
        );
        ctx.params
            .insert("assignment".to_string(), "HOSTILE_ASSIGNMENT".to_string());

        let html = render_html(&ctx);

        assert!(html.contains("data-admin-wallet-access-state=\"unavailable\""));
        for forbidden in [
            "HOSTILE_PLAN",
            "admin%3Apermissions%3Amanage",
            "HOSTILE_WALLET",
            "alert(&#39;access&#39;)",
            "HOSTILE_ASSIGNMENT",
        ] {
            assert!(
                !html.contains(forbidden),
                "hostile value leaked: {forbidden}"
            );
        }
        assert_no_sample_access_or_controls(&html);
    }

    #[test]
    fn leaf_has_no_admin_shell_or_main() {
        let html = render_html(&authenticated_empty_claims_ctx());

        assert!(html.contains("class=\"container page-content admin-wallet-access py-8\""));
        assert!(!html.contains("admin-shell"));
        assert!(!html.contains("<header"));
        assert!(!html.contains("<aside"));
        assert!(!html.contains("<footer"));
        assert!(!html.contains("<main"));
    }
}
