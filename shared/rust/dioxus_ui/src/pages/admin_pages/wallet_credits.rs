//! `/wallet-management/credits` — truthful admin credits availability.
//!
//! The legacy page read aggregate credit statistics and wallet history and
//! exposed credit mutations. The Rust admin BFF does not provide a selected,
//! backend-authoritative credits contract, so this page fails closed: it shows
//! neither inferred financial data nor mutation controls. Authentication is a
//! session-presence concern here; roles and permissions remain backend-owned.

use dioxus::prelude::*;

use crate::auth::AuthGate;
use crate::primitives::Icon;

use super::super::{PageContext, PageMeta};

const WALLET_CREDITS_PATH: &str = "/wallet-management/credits";
const ADMIN_HOME_PATH: &str = "/";

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Credits");
    (meta, rsx! { RenderWalletCredits { ctx: ctx.clone() } })
}

/// This leaf deliberately owns only route content. The admin BFF wraps this
/// route in its single `AdminLayout::Auth` shell, so adding `AdminShell` here
/// would duplicate the header, sidebar, and footer.
#[component]
fn RenderWalletCredits(ctx: PageContext) -> Element {
    rsx! {
        AuthGate {
            user: ctx.user.clone(),
            feature: Some("the private wallet credits workspace".to_string()),
            return_url: Some(WALLET_CREDITS_PATH.to_string()),
            WalletCreditsUnavailable {}
        }
    }
}

#[component]
fn WalletCreditsUnavailable() -> Element {
    rsx! {
        div {
            class: "container page-content admin-wallet-credits py-8",
            "data-admin-wallet-credits-state": "unavailable",
            section {
                class: "relative overflow-hidden rounded-3xl border border-border/40 bg-card shadow-2xl",
                role: "status",
                aria_labelledby: "admin-wallet-credits-unavailable-title",
                div {
                    class: "absolute inset-x-0 top-0 h-1 bg-gradient-to-r from-[#1fc7d4] via-[#7645d9] to-[#ffb237]",
                    aria_hidden: "true",
                }
                div { class: "p-8 md:p-12",
                    div { class: "flex flex-col gap-6 sm:flex-row sm:items-start",
                        div {
                            class: "flex h-16 w-16 shrink-0 items-center justify-center rounded-2xl border border-cyan-500/20 bg-cyan-500/10 text-[#1fc7d4]",
                            aria_hidden: "true",
                            Icon { name: "coins".to_string(), size: Some(30) }
                        }
                        div { class: "min-w-0 max-w-3xl",
                            p { class: "text-xs font-black uppercase tracking-[0.22em] text-[#1fc7d4]",
                                "Wallet credits"
                            }
                            h1 {
                                id: "admin-wallet-credits-unavailable-title",
                                class: "mt-3 text-3xl font-black tracking-tight text-foreground",
                                "Credit data is unavailable"
                            }
                            p { class: "mt-4 text-sm leading-6 text-muted-foreground",
                                "Balances, aggregate metrics, and wallet history are not shown because a verified credits read contract is not connected. Missing data is not presented as a zero balance or an empty ledger."
                            }
                            p { class: "mt-3 text-sm leading-6 text-muted-foreground",
                                "Credit changes remain paused until the backend owns authorization, validation, idempotency, and audit recording for every operation."
                            }
                            nav {
                                class: "mt-8 flex flex-wrap gap-3",
                                aria_label: "Wallet credits recovery",
                                a {
                                    class: "btn btn-primary",
                                    href: WALLET_CREDITS_PATH,
                                    Icon { name: "refresh-cw".to_string(), size: Some(16) }
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
            path: WALLET_CREDITS_PATH.to_string(),
            ..Default::default()
        }
    }

    fn render_html(ctx: &PageContext) -> String {
        let (_, element) = render(ctx);
        dioxus_ssr::render_element(element)
    }

    fn assert_no_financial_fabrication_or_controls(html: &str) {
        let lowered = html.to_ascii_lowercase();
        for forbidden in [
            "$45,200.00",
            "$8,400.00",
            "$3,250.00",
            "1,234",
            "tx_1",
            "promotional credit for early adopter",
            "api call charges",
            "beta tester reward",
            "refund reversal",
            "total credits outstanding",
            "credits granted today",
            "credits used today",
            "active users with credits",
            "grant credits",
            "revoke credits",
            "award credits",
            "deduct credits",
            "credit history",
            "<form",
            "<input",
            "<textarea",
            "<button",
            "<table",
        ] {
            assert!(
                !lowered.contains(&forbidden.to_ascii_lowercase()),
                "wallet credits must not render fabricated state or a financial control `{forbidden}`. Got: {html}"
            );
        }
    }

    #[test]
    fn signed_out_render_keeps_private_credit_state_hidden() {
        let html = render_html(&PageContext {
            path: WALLET_CREDITS_PATH.to_string(),
            ..Default::default()
        });

        assert!(html.contains("Sign in required"));
        assert!(html.contains("href=\"/auth?return_url=%2Fwallet-management%2Fcredits\""));
        assert!(!html.contains("data-admin-wallet-credits-state"));
        assert!(!html.contains("Credit data is unavailable"));
        assert_no_financial_fabrication_or_controls(&html);
    }

    #[test]
    fn authenticated_empty_claims_reach_explicit_unavailable_state() {
        let html = render_html(&authenticated_empty_claims_ctx());

        assert!(html.contains("data-admin-wallet-credits-state=\"unavailable\""));
        assert!(html.contains("role=\"status\""));
        assert!(html.contains("Credit data is unavailable"));
        assert!(html.contains("Missing data is not presented as a zero balance"));
        assert!(!html.contains("Permission required"));
        assert!(!html.contains("Admin access required"));
        assert_no_financial_fabrication_or_controls(&html);
    }

    #[test]
    fn unavailable_state_has_only_safe_native_recovery_navigation() {
        let html = render_html(&authenticated_empty_claims_ctx());

        assert!(html.contains("href=\"/wallet-management/credits\">"));
        assert!(html.contains("Retry</a>"));
        assert!(html.contains("href=\"/\">Admin home</a>"));
        assert_eq!(html.matches("<a ").count(), 2);
        assert_no_financial_fabrication_or_controls(&html);
    }

    #[test]
    fn hostile_params_and_query_are_not_reflected_or_treated_as_data() {
        let mut ctx = authenticated_empty_claims_ctx();
        ctx.query = "tab=grant&balance=HOSTILE_CREDIT_QUERY".to_string();
        ctx.params.insert(
            "transactions".to_string(),
            "HOSTILE_CREDIT_PARAM<script>alert('credit')</script>".to_string(),
        );

        let html = render_html(&ctx);

        assert!(html.contains("data-admin-wallet-credits-state=\"unavailable\""));
        assert!(!html.contains("HOSTILE_CREDIT_QUERY"));
        assert!(!html.contains("HOSTILE_CREDIT_PARAM"));
        assert!(!html.contains("alert(&#39;credit&#39;)"));
        assert_no_financial_fabrication_or_controls(&html);
    }

    #[test]
    fn leaf_is_body_only_because_the_admin_bff_owns_the_shell() {
        let html = render_html(&authenticated_empty_claims_ctx());

        assert!(html.contains("class=\"container page-content admin-wallet-credits py-8\""));
        assert!(!html.contains("admin-shell"));
        assert!(!html.contains("<header"));
        assert!(!html.contains("<aside"));
        assert!(!html.contains("<footer"));
        assert!(!html.contains("<main"));
    }
}
