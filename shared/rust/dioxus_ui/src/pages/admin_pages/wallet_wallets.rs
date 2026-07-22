//! Truthful authenticated shells for admin wallet list, detail, and disable routes.
//!
//! The Rust admin BFF does not yet consume a typed authoritative wallet read
//! model, and the disable mutation is not registered. These pages therefore
//! expose no sample addresses, balances, plans, permissions, activity, stats,
//! filters, exports, or mutation controls. Frontend roles and permissions are
//! never treated as policy authority.

use dioxus::prelude::*;

use crate::auth::AuthGate;
use crate::primitives::Icon;

use super::super::{PageContext, PageMeta};

const WALLETS_PATH: &str = "/wallet-management/wallets";
const MAX_ROUTE_REFERENCE_CHARS: usize = 64;

#[derive(Clone, Copy, PartialEq)]
enum WalletSurface {
    List,
    Detail,
    Disable,
}

impl WalletSurface {
    fn marker(self) -> &'static str {
        match self {
            Self::List => "list",
            Self::Detail => "detail",
            Self::Disable => "disable",
        }
    }

    fn meta_title(self) -> &'static str {
        match self {
            Self::List => "Wallets unavailable",
            Self::Detail => "Wallet detail unavailable",
            Self::Disable => "Wallet operation unavailable",
        }
    }

    fn eyebrow(self) -> &'static str {
        match self {
            Self::List => "Wallet inventory",
            Self::Detail => "Wallet workspace",
            Self::Disable => "Wallet operation",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::List => "Wallet inventory is unavailable",
            Self::Detail => "This wallet cannot be verified",
            Self::Disable => "Wallet changes are unavailable",
        }
    }

    fn detail(self) -> &'static str {
        match self {
            Self::List => {
                "No wallet records, counts, balances, platforms, permissions, subscription summaries, or activity are shown because an authoritative wallet list contract is not connected."
            }
            Self::Detail => {
                "No identity, balance, chain, subscription, permission, activity, or transaction data is shown because the backend has not verified the requested wallet."
            }
            Self::Disable => {
                "No status or impact is inferred, and no disable or re-enable action is offered because an authorized, idempotent, audited wallet mutation is not connected."
            }
        }
    }
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    render_surface(ctx, WalletSurface::List, None)
}

/// The route value is a bounded, control-free, escaped diagnostic reference.
/// It never proves that a wallet exists, is canonical, or is authorized.
pub fn render_detail(ctx: &PageContext) -> (PageMeta, Element) {
    let reference = bounded_route_reference(
        ctx.params
            .get("address")
            .map(String::as_str)
            .unwrap_or_default(),
    );
    render_surface(ctx, WalletSurface::Detail, Some(reference))
}

/// The legacy confirmation route remains non-mutating. It cannot derive impact
/// or status from the path and exposes no submit control or mutation endpoint.
pub fn render_disable(ctx: &PageContext) -> (PageMeta, Element) {
    let reference = bounded_route_reference(
        ctx.params
            .get("address")
            .map(String::as_str)
            .unwrap_or_default(),
    );
    render_surface(ctx, WalletSurface::Disable, Some(reference))
}

fn render_surface(
    ctx: &PageContext,
    surface: WalletSurface,
    route_reference: Option<String>,
) -> (PageMeta, Element) {
    let meta = PageMeta::admin(surface.meta_title());
    let retry_href = route_reference
        .as_deref()
        .map(|reference| route_href(surface, reference))
        .unwrap_or_else(|| WALLETS_PATH.to_string());

    (
        meta,
        rsx! {
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("the private admin wallet workspace".to_string()),
                // Never disclose a route identifier in signed-out HTML.
                return_url: Some(WALLETS_PATH.to_string()),
                WalletUnavailable { surface, route_reference, retry_href }
            }
        },
    )
}

fn bounded_route_reference(raw: &str) -> String {
    let cleaned = raw
        .chars()
        .filter(|character| !character.is_control())
        .collect::<String>();
    let cleaned = cleaned.trim();

    if cleaned.is_empty() {
        return "not provided".to_string();
    }

    if cleaned.chars().count() <= MAX_ROUTE_REFERENCE_CHARS {
        return cleaned.to_string();
    }

    let mut bounded = cleaned
        .chars()
        .take(MAX_ROUTE_REFERENCE_CHARS.saturating_sub(1))
        .collect::<String>();
    bounded.push('…');
    bounded
}

fn encode_path_segment(reference: &str) -> String {
    let mut encoded = String::with_capacity(reference.len());
    for byte in reference.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(char::from(*byte));
            }
            _ => {
                const HEX: &[u8; 16] = b"0123456789ABCDEF";
                encoded.push('%');
                encoded.push(char::from(HEX[(byte >> 4) as usize]));
                encoded.push(char::from(HEX[(byte & 0x0f) as usize]));
            }
        }
    }
    encoded
}

fn route_href(surface: WalletSurface, reference: &str) -> String {
    let encoded = encode_path_segment(reference);
    match surface {
        WalletSurface::List => WALLETS_PATH.to_string(),
        WalletSurface::Detail => format!("/wallet-management/{encoded}"),
        WalletSurface::Disable => {
            format!("/wallet-management/wallets/{encoded}/disable")
        }
    }
}

#[component]
fn WalletUnavailable(
    surface: WalletSurface,
    route_reference: Option<String>,
    retry_href: String,
) -> Element {
    let title_id = format!("admin-wallet-{}-unavailable-title", surface.marker());

    rsx! {
        div {
            class: "container page-content max-w-6xl py-10",
            "data-admin-wallets-state": "unavailable",
            "data-admin-wallets-surface": surface.marker(),
            section {
                class: "relative overflow-hidden rounded-3xl border border-border/40 bg-card shadow-2xl",
                role: "status",
                aria_labelledby: title_id.clone(),
                div {
                    class: "absolute inset-x-0 top-0 h-1 bg-gradient-to-r from-[#1fc7d4] via-[#7645d9] to-[#ed4b9e]",
                    aria_hidden: "true",
                }
                div { class: "grid gap-8 p-8 md:grid-cols-[auto_1fr] md:p-12",
                    div {
                        class: "flex h-16 w-16 items-center justify-center rounded-2xl border border-violet-500/20 bg-violet-500/10 text-violet-400",
                        aria_hidden: "true",
                        Icon { name: "wallet".to_string(), size: Some(30) }
                    }
                    div {
                        p { class: "text-xs font-black uppercase tracking-[0.22em] text-violet-400",
                            {surface.eyebrow()}
                        }
                        h1 { id: title_id, class: "mt-3 text-3xl font-black tracking-tight text-foreground",
                            {surface.title()}
                        }
                        div { class: "mt-5 rounded-2xl border border-amber-500/20 bg-amber-500/10 p-5",
                            p { class: "text-sm font-semibold leading-6 text-foreground",
                                {surface.detail()}
                            }
                        }
                        if let Some(reference) = route_reference {
                            p { class: "mt-4 rounded-xl border border-border/30 bg-background/50 px-4 py-3 text-sm text-muted-foreground",
                                "Unverified route reference: "
                                code { "data-admin-wallet-route-reference": "bounded", "{reference}" }
                            }
                        }
                        p { class: "mt-5 max-w-3xl text-sm leading-6 text-muted-foreground",
                            "The verified session keeps this workspace private. Only the Rust backend may authorize wallet reads or changes and return canonical typed data."
                        }
                        nav { class: "mt-8 flex flex-wrap gap-3", aria_label: "Wallet workspace recovery",
                            a { class: "btn btn-primary", href: retry_href,
                                Icon { name: "refresh-cw".to_string(), size: Some(16) }
                                " Retry wallet availability"
                            }
                            if surface != WalletSurface::List {
                                a { class: "btn btn-outline", href: WALLETS_PATH,
                                    Icon { name: "arrow-left".to_string(), size: Some(16) }
                                    " Wallet list"
                                }
                            }
                            a { class: "btn btn-ghost", href: "/",
                                Icon { name: "home".to_string(), size: Some(16) }
                                " Admin home"
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
    use std::collections::HashMap;

    use super::*;
    use crate::auth::user::{AuthMethod, User};

    fn authenticated_ctx() -> PageContext {
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
            path: WALLETS_PATH.to_string(),
            ..Default::default()
        }
    }

    fn html(element: Element) -> String {
        dioxus_ssr::render_element(element)
    }

    fn assert_no_samples_or_controls(rendered: &str) {
        let lowered = rendered.to_ascii_lowercase();
        for forbidden in [
            "0x1234…5678",
            "0xabcd…ef12",
            "0xdead…beef",
            "1.234 bnb",
            "pro plan ($29/mo)",
            "platform distribution",
            "download transactions csv",
            "add wallet",
            "disable wallet",
            "re-enable wallet",
            "grant access",
            "all status",
            "all platforms",
            "date created",
            "<form",
            "<input",
            "<textarea",
            "<select",
            "<button",
            "<table",
        ] {
            assert!(
                !lowered.contains(&forbidden.to_ascii_lowercase()),
                "wallet UI leaked sample state or a control `{forbidden}`: {rendered}"
            );
        }
    }

    #[test]
    fn signed_out_direct_routes_hide_private_state_and_references() {
        for (path, render_fn) in [
            (
                WALLETS_PATH,
                render as fn(&PageContext) -> (PageMeta, Element),
            ),
            ("/wallet-management/private-reference", render_detail),
            (
                "/wallet-management/wallets/private-reference/disable",
                render_disable,
            ),
        ] {
            let mut ctx = PageContext {
                path: path.to_string(),
                ..Default::default()
            };
            ctx.params
                .insert("address".to_string(), "private-reference".to_string());
            let rendered = html(render_fn(&ctx).1);

            assert!(rendered.contains("Sign in required"), "{path}: {rendered}");
            assert!(
                !rendered.contains("private-reference"),
                "{path}: {rendered}"
            );
            assert!(!rendered.contains("data-admin-wallets-state"));
            assert!(rendered.contains("href=\"/auth?return_url=%2Fwallet-management%2Fwallets\""));
            assert_no_samples_or_controls(&rendered);
        }
    }

    #[test]
    fn empty_role_session_reaches_all_explicit_unavailable_surfaces() {
        let mut ctx = authenticated_ctx();
        let list = html(render(&ctx).1);

        ctx.params
            .insert("address".to_string(), "0xunverified".to_string());
        ctx.path = "/wallet-management/0xunverified".to_string();
        let detail = html(render_detail(&ctx).1);
        ctx.path = "/wallet-management/wallets/0xunverified/disable".to_string();
        let disable = html(render_disable(&ctx).1);

        for (surface, rendered) in [("list", list), ("detail", detail), ("disable", disable)] {
            assert!(rendered.contains("data-admin-wallets-state=\"unavailable\""));
            assert!(rendered.contains(&format!("data-admin-wallets-surface=\"{surface}\"")));
            assert!(!rendered.contains("Permission required"));
            assert!(!rendered.contains("Admin access required"));
            assert_no_samples_or_controls(&rendered);
        }
    }

    #[test]
    fn dynamic_reference_is_bounded_escaped_unverified_and_one_encoded_segment() {
        let mut ctx = authenticated_ctx();
        ctx.path = "/wallet-management/hostile".to_string();
        ctx.params.insert(
            "address".to_string(),
            format!("{}<script>alert(1)</script>\n", "x".repeat(80)),
        );
        let rendered = html(render_detail(&ctx).1);

        assert!(rendered.contains("Unverified route reference"));
        assert!(rendered.contains("data-admin-wallet-route-reference=\"bounded\""));
        assert!(rendered.contains('…'));
        assert!(!rendered.contains("<script>"));
        assert!(!rendered.contains("alert(1)"));
        assert!(rendered.contains("href=\"/wallet-management/"));
        assert!(!rendered.contains("\n"));
        assert_no_samples_or_controls(&rendered);
    }

    #[test]
    fn query_and_unrelated_params_never_create_wallet_state() {
        let mut ctx = authenticated_ctx();
        ctx.query = "balance=999&plan=HOSTILE_PLAN&status=active".to_string();
        ctx.params = HashMap::from([
            ("balance".to_string(), "HOSTILE_BALANCE".to_string()),
            ("permissions".to_string(), "HOSTILE_PERMISSION".to_string()),
        ]);
        let rendered = html(render(&ctx).1);

        for forbidden in [
            "999",
            "HOSTILE_PLAN",
            "HOSTILE_BALANCE",
            "HOSTILE_PERMISSION",
        ] {
            assert!(!rendered.contains(forbidden));
        }
        assert!(rendered.contains("data-admin-wallets-state=\"unavailable\""));
        assert_no_samples_or_controls(&rendered);
    }

    #[test]
    fn leaves_are_body_only_and_disable_surface_has_no_mutation_affordance() {
        let mut ctx = authenticated_ctx();
        ctx.params
            .insert("address".to_string(), "0xunverified".to_string());

        for rendered in [
            html(render(&ctx).1),
            html(render_detail(&ctx).1),
            html(render_disable(&ctx).1),
        ] {
            assert!(!rendered.contains("class=\"admin-shell"));
            assert!(!rendered.contains("<main"));
            assert!(!rendered.contains("<header"));
            assert!(!rendered.contains("<aside"));
            assert!(!rendered.contains("<footer"));
            assert_no_samples_or_controls(&rendered);
        }
    }
}
