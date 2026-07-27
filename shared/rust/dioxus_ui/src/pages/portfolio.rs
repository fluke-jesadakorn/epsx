//! `/portfolio` — truthful owner-portfolio availability shell.
//!
//! The development source gates portfolio data behind authentication, but
//! this Rust route does not yet have a frozen owner-scoped holdings/watchlist
//! contract. Signed-out visitors retain the source-shaped sign-in experience.
//! Authenticated visitors receive an explicit unavailable state instead of
//! hard-coded stocks, prices, ranks, entitlement claims, or inert controls.
//! Legacy `data_portfolio` parameters are intentionally ignored.

use super::PageContext;
use super::PageMeta;
use crate::components::auth_access_banner::AuthAccessBanner;
use crate::layout::main_layout::MainLayout;
use crate::primitives::*;
use dioxus::prelude::*;

const PORTFOLIO_SIGN_IN_PATH: &str = "/auth?return_url=%2Fportfolio";

/// Inline CSS rules for Tailwind v2 CDN arbitrary-value classes
/// that the CDN doesn't generate. We inject these into the page so
/// `h-[400px]`-style dimensions render correctly.
const PORTFOLIO_INLINE_CSS: &str = r#"
.portfolio-prod-bg > div[style*="radial-gradient"] { opacity: 1 !important; }
.absolute.-top-40.-left-40 { width: 400px !important; height: 400px !important; }
.absolute.top-1\/3.-right-32 { width: 300px !important; height: 300px !important; }
/* Sign-in card needs prod colors (bg-blue-50 border-blue-200
   dark:bg-blue-900/20 dark:border-blue-700). The page renders in
   dark mode so we use the dark-theme values (blue-900/20 + blue-700)
   — these are what v2 CDN renders vs prod's v3 PostCSS pipeline. */
.portfolio-signin-card {
  background-color: rgb(30 58 138 / 0.2) !important;
  border-color: rgb(29 78 216) !important;
}
/* Wave 28 T2 — Tailwind v2 CDN doesn't generate the arbitrary-
   value `min-h-[300px]` class, so force it on the prod's
   `<RequireSignIn>` wrapper (which reserves 300px of vertical
   space for the signin card). */
.portfolio-prod-require-signin { min-height: 300px !important; }
"#;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Portfolio");
    (
        meta,
        rsx! {
            MainLayout { ctx: ctx.clone(),
                style { "{PORTFOLIO_INLINE_CSS}" }
                // Wave 25 T2 — match prod's bg-gray-50 dark:bg-slate-950
                // shell (we use the dark color directly because Tailwind
                // v2 CDN drops `dark:` variants). The fixed bg layer has
                // 3 gradient orbs + a radial dark overlay.
                div { class: "portfolio-prod-page relative min-h-screen bg-slate-950",
                    div { class: "fixed inset-0 z-0 portfolio-prod-bg",
                        div { class: "absolute inset-0 bg-gradient-to-b from-slate-950 via-slate-900 to-slate-950" }
                        div { class: "absolute -top-40 -left-40 h-[400px] w-[400px] rounded-full bg-emerald-600/15 blur-3xl portfolio-prod-orb-1" }
                        div { class: "absolute top-1/3 -right-32 h-[300px] w-[300px] rounded-full bg-teal-600/10 blur-3xl portfolio-prod-orb-2" }
                        div { class: "absolute inset-0 bg-[radial-gradient(ellipse_at_center,_transparent_0%,_rgba(0,0,0,0.3)_100%)]" }
                    }
                    div { class: "relative z-10",
                        div { class: "mx-auto max-w-7xl px-4 py-6 sm:py-8 portfolio-prod-container",
                            PortfolioHeader { wallet_connected: ctx.wallet.address.is_some() }
                            if ctx.user.is_none() {
                                if ctx.wallet.address.is_none() {
                                    AuthAccessBanner { href: PORTFOLIO_SIGN_IN_PATH.to_string() }
                                }
                                div { class: "flex items-center justify-center min-h-[300px] p-6 portfolio-prod-require-signin",
                                    div { class: "max-w-md w-full",
                                        PortfolioSignInCard {}
                                    }
                                }
                            } else {
                                PortfolioUnavailable { source_shape: true }
                            }
                        }
                    }
                }
            }
        },
    )
}

/// Preserve the recognizable development-source heading without claiming
/// that market data is live or that a watchlist has been loaded.
#[component]
fn PortfolioHeader(wallet_connected: bool) -> Element {
    rsx! {
        div { class: "portfolio-prod-header mb-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between",
            div { class: "flex items-center gap-3",
                div { class: "portfolio-prod-icon flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-emerald-400 to-teal-500",
                    Icon { name: "heart".to_string(), size: Some(20), class_name: Some("text-white".to_string()) }
                }
                div {
                    h1 { class: "text-2xl font-bold text-white portfolio-prod-title", "Portfolio" }
                    p { class: "text-sm text-slate-400 portfolio-prod-subtitle",
                        "Track your watchlisted stocks"
                    }
                }
            }
            span { class: "inline-flex w-max items-center gap-1.5 self-start rounded-lg border border-emerald-500/20 bg-emerald-500/10 px-3 py-1.5 text-xs font-medium text-emerald-400 sm:self-center",
                Icon { name: "trending-up".to_string(), size: Some(14) }
                "Live"
            }
        }
    }
}

#[component]
fn PortfolioUnavailable(source_shape: bool) -> Element {
    rsx! {
        section {
            class: if source_shape {
                "portfolio-unavailable portfolio-source-preview overflow-hidden rounded-none border-0 bg-transparent shadow-none"
            } else {
                "portfolio-unavailable overflow-hidden rounded-3xl border border-slate-700/80 bg-slate-900/50 shadow-xl shadow-black/20"
            },
            "data-portfolio-state": "unavailable",
            role: "alert",
            aria_labelledby: "portfolio-unavailable-title",
            if !source_shape {
                div { class: "h-1.5 bg-gradient-to-r from-emerald-400 via-teal-400 to-cyan-400" }
            }
            div { class: if source_shape { "space-y-4 p-0 sm:space-y-8 sm:p-8" } else { "space-y-8 p-5 sm:p-8" },
                div {
                    class: if source_shape { "portfolio-watchlist-search flex min-w-0 items-center gap-2 rounded-lg border border-slate-600 bg-slate-800/70 px-3 py-2 text-xs text-slate-400 sm:gap-3 sm:rounded-2xl sm:px-5 sm:py-4 sm:text-xl" } else { "portfolio-watchlist-search flex items-center gap-3 rounded-2xl border border-slate-600 bg-slate-800/70 px-5 py-4 text-base text-slate-400 sm:text-xl" },
                    role: "searchbox",
                    aria_disabled: "true",
                    Icon { name: "search".to_string(), size: Some(if source_shape { 14 } else { 22 }) }
                    span { class: "min-w-0 truncate", "Search stocks to add to watchlist…" }
                }

                div { class: if source_shape { "flex min-h-[220px] flex-col items-center justify-center text-center sm:min-h-[360px]" } else { "flex min-h-[280px] flex-col items-center justify-center text-center sm:min-h-[360px]" },
                    div { class: if source_shape { "flex h-16 w-16 items-center justify-center rounded-2xl bg-slate-800 text-slate-400" } else { "flex h-24 w-24 items-center justify-center rounded-3xl bg-slate-800 text-slate-400" },
                        Icon { name: "heart".to_string(), size: Some(if source_shape { 32 } else { 52 }) }
                    }
                    h2 {
                        id: "portfolio-unavailable-title",
                        class: if source_shape { "mt-4 text-base font-semibold text-white sm:mt-8 sm:text-3xl" } else { "mt-8 text-2xl font-semibold text-white sm:text-3xl" },
                        "No watchlist data available"
                    }
                    p { class: if source_shape { "mt-2 max-w-xs text-[11px] leading-4 text-slate-400 sm:max-w-2xl sm:text-xl sm:leading-relaxed" } else { "mt-3 max-w-2xl text-base leading-relaxed text-slate-400 sm:text-xl" },
                        "The owner-scoped holdings and watchlist response is unavailable. Search and watchlist actions stay disabled until the backend contract is ready."
                    }
                    p { class: "sr-only",
                        "Your portfolio cannot be verified right now. No securities, prices, rankings, plan access, or watchlist membership are being inferred."
                    }
                }

                nav {
                    class: if source_shape {
                        "sr-only"
                    } else {
                        "flex flex-col gap-3 border-t border-slate-700 pt-6 sm:flex-row"
                    },
                    aria_label: "Portfolio alternatives",
                    a {
                        class: "btn btn-primary",
                        href: "/account",
                        Icon { name: "user".to_string(), size: Some(16) }
                        " Return to account"
                    }
                    a {
                        class: "btn btn-ghost",
                        href: "/contact",
                        Icon { name: "circle-help".to_string(), size: Some(16) }
                        " Contact support"
                    }
                }
            }
        }
    }
}

#[component]
fn PortfolioBoundaryItem(icon: &'static str, title: &'static str, body: &'static str) -> Element {
    rsx! {
        div { class: "rounded-xl border border-slate-700 bg-slate-800/50 p-4",
            div { class: "flex items-center gap-2 font-semibold text-white",
                Icon { name: icon.to_string(), size: Some(18) }
                "{title}"
            }
            p { class: "mt-2 text-sm leading-6 text-slate-300", "{body}" }
            span { class: "mt-3 inline-flex rounded-full border border-amber-500/30 bg-amber-500/10 px-2 py-1 text-xs font-medium text-amber-400",
                "Unavailable"
            }
        }
    }
}

/// "Sign In Required" blue card. Mirrors prod's
/// `p-6 bg-blue-50 border border-blue-200 rounded-lg
/// dark:bg-blue-900/20 dark:border-blue-700` panel with a 🔐
/// emoji icon, "Sign In Required" heading, "To view your
/// portfolio, you need basic authentication." subtext, a bright
/// blue "Sign In" button, a blue "Learn More" link, and a small
/// blue "Need help?" footer.
///
/// Wave 28 T2 — replaced the gold 40px lock SVG with the prod's
/// `🔐` emoji span (the prod uses the literal emoji, not an SVG),
/// and changed the inner wrapper from `flex flex-col items-center
/// text-center` to the prod's `text-center space-y-4` shape.
#[component]
fn PortfolioSignInCard() -> Element {
    rsx! {
        div { class: "portfolio-prod-signin portfolio-signin-card p-6 bg-blue-900/20 border border-blue-700 rounded-lg",
            div { class: "text-center space-y-4",
                // 🔐 emoji icon (prod's actual markup — no SVG)
                div { class: "flex justify-center",
                    span { class: "text-3xl", role: "img", aria_label: "Sign in required", "🔐" }
                }
                // Heading
                h3 { class: "portfolio-prod-signin-title text-lg font-medium text-blue-100",
                    "Sign In Required"
                }
                // Subtext
                p { class: "portfolio-prod-signin-sub text-sm text-blue-300",
                    "To view your portfolio, you need basic authentication."
                }
                // Primary "Sign In" button — bright blue
                a { class: "portfolio-prod-signin-btn w-full px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 font-medium text-center block",
                    href: PORTFOLIO_SIGN_IN_PATH,
                    "Sign In"
                }
                // "Learn More" link — blue text
                a { class: "portfolio-prod-signin-link w-full px-4 py-2 text-blue-400 hover:text-blue-300 font-medium text-sm text-center block",
                    href: "/contact",
                    "Learn More"
                }
                // Footer — "Need help?"
                p { class: "portfolio-prod-signin-footer text-xs text-blue-400",
                    "Need help? Check our support documentation or contact support."
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::user::AuthMethod;
    use crate::auth::User;

    fn authed_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "u1".to_string(),
                address: "0x1234…abcd".to_string(),
                chain_id: "56".to_string(),
                roles: vec!["user".to_string()],
                email: Some("test@epsx.io".to_string()),
                tier: Some("Pro".to_string()),
                permissions: vec![],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: Some("Test".to_string()),
            }),
            path: "/portfolio".to_string(),
            ..Default::default()
        }
    }

    fn anon_ctx() -> PageContext {
        PageContext {
            user: None,
            path: "/portfolio".to_string(),
            ..Default::default()
        }
    }

    fn connected_anon_ctx() -> PageContext {
        PageContext {
            wallet: crate::auth::wallet_button::ConnectedWalletState {
                address: Some("0x1234567890abcdef1234567890abcdef12345678".to_string()),
                connector_id: Some("metaMask".to_string()),
                chain_id: Some(56),
                ..Default::default()
            },
            ..anon_ctx()
        }
    }

    #[test]
    fn authenticated_portfolio_fails_closed_with_meaningful_alternatives() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);

        for marker in [
            "data-portfolio-state=\"unavailable\"",
            "Your portfolio cannot be verified right now",
            "No securities, prices, rankings, plan access, or watchlist membership are being inferred.",
            "aria-label=\"Portfolio alternatives\"",
            "href=\"/account\"",
            "href=\"/contact\"",
        ] {
            assert!(html.contains(marker), "missing truthful marker `{marker}`: {html}");
        }
        assert!(!html.contains("href=\"/portfolio\""));
        assert!(!html.contains("> Retry</a>"));
    }

    #[test]
    fn canned_and_malformed_portfolio_payloads_are_ignored() {
        for payload in [
            r#"{"holdings":[{"symbol":"CANNED_TICKER","price":"$987.65","rank":"Premium","eps":"EPS ▲"}]}"#,
            r#"{"watchlist":["CANNED_WATCHLIST_ITEM"],"live":true}"#,
            "{malformed",
        ] {
            let mut ctx = authed_ctx();
            ctx.params
                .insert("data_portfolio".to_string(), payload.to_string());
            let (_meta, el) = render(&ctx);
            let html = dioxus_ssr::render_element(el);

            assert!(html.contains("data-portfolio-state=\"unavailable\""));
            for forbidden in [
                "CANNED_TICKER",
                "$987.65",
                "Premium",
                "EPS ▲",
                "CANNED_WATCHLIST_ITEM",
                "portfolio-prod-stock-card",
                "portfolio-prod-search-input",
            ] {
                assert!(
                    !html.contains(forbidden),
                    "legacy payload or unsupported control `{forbidden}` must not render: {html}"
                );
            }
        }
    }

    #[test]
    fn authenticated_portfolio_has_no_sample_financial_or_entitlement_claims() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);

        for forbidden in [
            "AAPL",
            "MSFT",
            "NVDA",
            "$189.45",
            "+2.34%",
            "Your Watchlist",
            "Unlock Full Analytics Access",
            "Top 100 stock rankings",
            "Real-time EPS data",
            "AI-powered insights",
            "Sign In Free",
        ] {
            assert!(
                !html.contains(forbidden),
                "unsupported portfolio or entitlement claim `{forbidden}` must not render: {html}"
            );
        }
    }

    #[test]
    fn signed_out_portfolio_keeps_truthful_require_sign_in_state() {
        let (_meta, el) = render(&anon_ctx());
        let html = dioxus_ssr::render_element(el);

        for marker in [
            "portfolio-prod-require-signin",
            "portfolio-prod-signin",
            "Sign In Required",
            "To view your portfolio, you need basic authentication.",
            "href=\"/auth?return_url=%2Fportfolio\"",
            "href=\"/contact\"",
            "aria-label=\"Sign in required\"",
        ] {
            assert!(
                html.contains(marker),
                "missing signed-out marker `{marker}`: {html}"
            );
        }

        assert_eq!(html.matches(PORTFOLIO_SIGN_IN_PATH).count(), 2);
        assert!(html.contains("Unlock Full Analytics Access"));
        assert!(!html.contains("href=\"/auth\""));
        assert!(!html.contains("data-portfolio-state=\"unavailable\""));
        assert!(!html.contains("portfolio-prod-stock-card"));
        assert!(!html.contains("portfolio-prod-search-input"));
        assert!(!html.contains("portfolio-prod-upsell"));
    }

    #[test]
    fn connected_wallet_without_session_still_uses_the_source_sign_in_gate() {
        let (_meta, el) = render(&connected_anon_ctx());
        let html = dioxus_ssr::render_element(el);

        assert!(html.contains("portfolio-prod-signin"));
        assert!(html.contains("Sign In Required"));
        assert!(!html.contains("data-portfolio-state=\"unavailable\""));
        assert!(!html.contains("Live preview"));
    }
}
