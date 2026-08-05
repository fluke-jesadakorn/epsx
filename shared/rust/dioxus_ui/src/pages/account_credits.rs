//! `/account/credits` — truthful credits availability shell.
//!
//! Credits do not yet have a selected authoritative read path in the
//! migration architecture. This page therefore renders only states the
//! server can prove: signed out, or authenticated with credits unavailable.
//! It must not infer a zero balance, an empty ledger, or transaction data
//! from absent, malformed, or compatibility parameters.

use dioxus::prelude::*;

use super::{PageContext, PageMeta};
use crate::layout::main_layout::MainLayout;
use crate::primitives::*;

const ACCOUNT_PATH: &str = "/account";
const CREDITS_SIGN_IN_PATH: &str = "/auth?return_url=%2Faccount%2Fcredits";

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Credits");
    (meta, rsx! { RenderAccountCredits { ctx: ctx.clone() } })
}

#[component]
fn RenderAccountCredits(ctx: PageContext) -> Element {
    let owner = ctx.user.as_ref().map(|user| user.address.clone());

    rsx! {
        MainLayout { ctx: ctx.clone(),
            // The source credits route uses a centered max-w-6xl frame with
            // a 1.5rem inset and top breathing room below the header. Keep
            // that geometry even when the owner-scoped data is unavailable.
            div { class: "page-content credits-ledger-page mx-auto max-w-6xl px-6 pt-6",
                div { class: "mb-6",
                    h1 { class: "text-3xl font-bold text-foreground", "Credit Balance" }
                    p { class: "mt-2 text-slate-400",
                        "Manage your EPSX credits and view transaction history"
                    }
                }

                if let Some(owner) = owner {
                    CreditsUnavailable { owner }
                } else {
                    CreditsSignedOut {}
                }
            }
        }
    }
}

#[component]
fn CreditsSignedOut() -> Element {
    rsx! {
        section {
            class: "credits-access-state card card-glass overflow-hidden",
            "data-credits-state": "signed-out",
            aria_labelledby: "credits-signed-out-title",
            role: "status",
            div { class: "p-10 sm:p-12 text-center",
                div { class: "mx-auto flex h-14 w-14 items-center justify-center rounded-2xl bg-primary/10 text-primary",
                    Icon { name: "coins".to_string(), size: Some(28) }
                }
                p { class: "mt-5 text-xs font-semibold uppercase tracking-widest text-primary",
                    "Private account data"
                }
                h2 {
                    id: "credits-signed-out-title",
                    class: "mt-2 text-2xl font-semibold text-foreground",
                    "Sign in to view your credits"
                }
                p { class: "mx-auto mt-3 max-w-xl text-sm leading-6 text-muted-foreground",
                    "Credit balances and ledger activity are private to the wallet that owns them."
                }
                a {
                    class: "btn btn-primary mt-6",
                    href: CREDITS_SIGN_IN_PATH,
                    "Sign in"
                }
            }
        }
    }
}

#[component]
fn CreditsUnavailable(owner: String) -> Element {
    rsx! {
        section {
            class: "credits-unavailable-content",
            "data-credits-state": "unavailable",
            aria_labelledby: "credits-unavailable-title",
            role: "alert",

            div { class: "credits-balance-row grid grid-cols-1 gap-4 md:grid-cols-3",
                UnavailableBalanceCard {
                    marker: "credits-balance-available",
                    icon: "coins",
                    label: "Available Balance",
                    highlighted: true,
                }
                UnavailableBalanceCard {
                    marker: "credits-balance-earned",
                    icon: "trending-up",
                    label: "Lifetime Earned",
                    highlighted: false,
                }
                UnavailableBalanceCard {
                    marker: "credits-balance-spent",
                    icon: "trending-down",
                    label: "Lifetime Spent",
                    highlighted: false,
                }
            }

            div { class: "credits-transaction-list card card-glass mt-6 overflow-hidden",
                div { class: "border-b border-border p-6",
                    h2 { class: "text-xl font-semibold text-foreground", "Transaction History" }
                }
                div { class: "p-8 sm:p-10",
                    div { class: "flex flex-col items-start gap-5 sm:flex-row",
                        div { class: "flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-amber-500/10 text-amber-500",
                            Icon { name: "alert-circle".to_string(), size: Some(24) }
                        }
                        div { class: "min-w-0 flex-1",
                            p { class: "text-xs font-semibold uppercase tracking-widest text-amber-500",
                                "Credits unavailable"
                            }
                            h3 {
                                id: "credits-unavailable-title",
                                class: "mt-2 text-xl font-semibold text-foreground",
                                "Your credit data cannot be verified right now"
                            }
                            p { class: "mt-2 text-sm leading-6 text-muted-foreground",
                                "We cannot verify your credit balance or transaction history right now. No balance or ledger activity is being inferred."
                            }
                            p { class: "mt-3 text-xs text-muted-foreground",
                                "Signed-in wallet: "
                                span { class: "font-mono break-all text-foreground", "{owner}" }
                            }
                            nav {
                                class: "mt-6 flex flex-wrap gap-3",
                                "aria-label": "Credit page alternatives",
                                a { class: "btn btn-primary", href: ACCOUNT_PATH, "Back to account" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn UnavailableBalanceCard(
    marker: &'static str,
    icon: &'static str,
    label: &'static str,
    highlighted: bool,
) -> Element {
    let card_class = if highlighted {
        format!(
            "{marker} card card-glass border-primary/20 bg-gradient-to-br from-primary/10 to-transparent"
        )
    } else {
        format!("{marker} card card-glass")
    };

    rsx! {
        div { class: "{card_class}",
            div { class: "card-body",
                div { class: "mb-3 flex items-center justify-between",
                    div { class: "rounded-lg bg-primary/10 p-2 text-primary",
                        Icon { name: icon.to_string(), size: Some(20) }
                    }
                    span {
                        class: "rounded-full border border-amber-500/30 bg-amber-500/10 px-2 py-1 text-xs font-medium text-amber-500",
                        "Unavailable"
                    }
                }
                p { class: "text-sm text-muted-foreground", "{label}" }
                p {
                    class: "mt-1 text-2xl font-semibold text-foreground",
                    "data-credit-value": "unavailable",
                    "Not available"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::user::{AuthMethod, User};

    fn signed_out_ctx() -> PageContext {
        PageContext {
            user: None,
            path: "/account/credits".to_string(),
            ..Default::default()
        }
    }

    fn authed_ctx_with_address(address: &str) -> PageContext {
        PageContext {
            user: Some(User {
                id: "u-1".to_string(),
                address: address.to_string(),
                chain_id: "1".to_string(),
                roles: vec!["user".to_string()],
                email: Some("test@epsx.io".to_string()),
                tier: Some("pro".to_string()),
                permissions: vec!["profile:read".to_string()],
                last_login_at: None,
                auth_method: AuthMethod::default(),
                display_name: Some("EPSX tester".to_string()),
            }),
            path: "/account/credits".to_string(),
            ..Default::default()
        }
    }

    fn render_html(ctx: &PageContext) -> String {
        let (_meta, element) = render(ctx);
        dioxus_ssr::render_element(element)
    }

    fn assert_no_inferred_financial_state(html: &str) {
        for forbidden in [
            "$0",
            "No transactions found",
            "credits-filter-chip",
            "Admin Grant",
            "Admin Revoke",
            "Proration Credit",
            "Daily credit grant",
            "$250",
            "$1,250",
        ] {
            assert!(
                !html.contains(forbidden),
                "credits page must not render inferred, canned, or inert content `{forbidden}`. Got: {html}"
            );
        }
    }

    #[test]
    fn signed_out_is_a_status_with_native_return_link() {
        let html = render_html(&signed_out_ctx());

        assert!(html.contains("data-credits-state=\"signed-out\""));
        assert!(html.contains("role=\"status\""));
        assert!(html.contains("Sign in to view your credits"));
        assert!(html.contains("href=\"/auth?return_url=%2Faccount%2Fcredits\""));
        assert!(!html.contains("data-credits-state=\"unavailable\""));
        assert_no_inferred_financial_state(&html);
    }

    #[test]
    fn authenticated_state_is_an_alert_with_meaningful_account_navigation() {
        let html = render_html(&authed_ctx_with_address("0x1234abcd"));

        assert!(html.contains("data-credits-state=\"unavailable\""));
        assert!(html.contains("role=\"alert\""));
        assert!(html.contains("Your credit data cannot be verified right now"));
        assert!(html.contains("No balance or ledger activity is being inferred."));
        assert!(html.contains("aria-label=\"Credit page alternatives\""));
        assert!(html.contains("href=\"/account\">Back to account</a>"));
        assert!(!html.contains("href=\"/account/credits\""));
        assert!(!html.contains(">Retry</a>"));
        assert_no_inferred_financial_state(&html);
    }

    #[test]
    fn canned_partial_and_malformed_params_are_never_financial_data() {
        let mut canned = authed_ctx_with_address("0x1234abcd");
        canned.params.insert(
            "data_credits".to_string(),
            r#"{"available_balance":250,"lifetime_earned":1250,"transactions":[{"title":"Daily credit grant","amount":250}]}"#.to_string(),
        );
        let canned_html = render_html(&canned);

        let mut malformed = authed_ctx_with_address("0x1234abcd");
        malformed
            .params
            .insert("data_credits".to_string(), "{not-json".to_string());
        let malformed_html = render_html(&malformed);

        for html in [&canned_html, &malformed_html] {
            assert!(html.contains("data-credits-state=\"unavailable\""));
            assert!(html.contains("data-credit-value=\"unavailable\""));
            assert_no_inferred_financial_state(html);
        }
    }

    #[test]
    fn unavailable_layout_stays_source_like_without_inert_filter_controls() {
        let html = render_html(&authed_ctx_with_address("0x1234abcd"));

        for marker in [
            "credits-ledger-page",
            "credits-balance-row",
            "credits-balance-available",
            "credits-balance-earned",
            "credits-balance-spent",
            "credits-transaction-list",
        ] {
            assert!(
                html.contains(marker),
                "truthful unavailable page must retain source-like section `{marker}`. Got: {html}"
            );
        }
        assert_eq!(html.matches("data-credit-value=\"unavailable\"").count(), 3);
        assert_no_inferred_financial_state(&html);
    }

    #[test]
    fn dynamic_owner_is_html_escaped() {
        let html = render_html(&authed_ctx_with_address("<script>alert('owner')</script>"));

        assert!(!html.contains("<script>alert('owner')</script>"));
        assert!(
            html.contains("&#60;script&#62;alert(&#39;owner&#39;)&#60;/script&#62;"),
            "wallet content must be escaped by RSX. Got: {html}"
        );
        assert_no_inferred_financial_state(&html);
    }
}
