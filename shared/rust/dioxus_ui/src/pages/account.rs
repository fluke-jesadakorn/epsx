//! `/account` — production-safe account overview.
//!
//! The page preserves the development UI's section composition while only
//! presenting claims available from the verified local session. Unsupported
//! profile statistics, credit balance, access details, and preferences fail
//! closed as unavailable; they are never inferred from missing or partial
//! compatibility payloads. Payment history retains its separate strict,
//! owner-scoped read contract.

use crate::primitives::*;

use super::PageContext;
use super::PageMeta;
use crate::auth::user::{AuthMethod, User};
use crate::components::account::{
    decode_pay_history, PaymentHistoryLoad, PaymentHistoryTab, ACCOUNT_PAYMENT_HISTORY_DATA_PARAM,
    ACCOUNT_PAYMENT_HISTORY_EMPTY, ACCOUNT_PAYMENT_HISTORY_MALFORMED,
    ACCOUNT_PAYMENT_HISTORY_MAX_ITEMS, ACCOUNT_PAYMENT_HISTORY_READY,
    ACCOUNT_PAYMENT_HISTORY_STATE_PARAM, ACCOUNT_PAYMENT_HISTORY_UNAVAILABLE,
};
use crate::layout::main_layout::MainLayout;
use dioxus::prelude::*;

const ACCOUNT_PATH: &str = "/account";
const ACCOUNT_SIGN_IN_PATH: &str = "/auth?return_url=%2Faccount";

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Account");
    (meta, rsx! { RenderAccount { ctx: ctx.clone() } })
}

#[component]
fn RenderAccount(ctx: PageContext) -> Element {
    let session_user = ctx.user.clone();
    let payment_history_address = ctx.user.as_ref().map(|user| user.address.clone());
    let payment_history_load = payment_history_load(&ctx);

    rsx! {
        MainLayout { ctx: ctx.clone(),
            div { class: "container page-content account-settings-page",
                "data-section": "account-page",
                // 1. Hero
                AccountSettingsHero {}
                // 2. 4 stat cards
                div { class: "mt-8",
                    AccountStatsRow { user: session_user.clone() }
                }
                // 3. 3 quick-action cards
                div { class: "mt-8",
                    AccountQuickActions {}
                }
                // 4. Access & Plans
                div { class: "mt-8",
                    AccessAndPlansSection {}
                }
                // 5. Transaction History
                div { class: "mt-8",
                    PaymentHistorySection {
                        address: payment_history_address,
                        load: payment_history_load,
                    }
                }
                // 6. Notification Preferences
                div { class: "mt-8",
                    NotificationPreferencesSection { signed_in: session_user.is_some() }
                }
                // 7. Privacy & Data Security banner
                div { class: "mt-8",
                    PrivacyBannerSection {}
                }
            }
        }
    }
}

// ----- 1. Hero ----------------------------------------------------------------

/// "Account Settings" gradient title + tagline. Mirrors the OLD
/// prod: h1 with a 4-stop yellow→orange→pink→purple gradient via
/// `bg-clip-text text-transparent` + a small "👤" emoji before.
#[component]
fn AccountSettingsHero() -> Element {
    rsx! {
        div { class: "account-settings-hero text-center mb-12",
            "data-section": "account-settings-hero",
            h1 { class: "text-4xl sm:text-5xl font-bold flex items-center justify-center gap-3",
                span { class: "text-foreground", "👤" }
                span { class: "bg-gradient-to-r from-yellow-600 via-orange-600 via-pink-600 to-purple-600 bg-clip-text text-transparent",
                    "Account Settings"
                }
            }
            p { class: "mt-4 text-base sm:text-lg text-slate-300 max-w-2xl mx-auto font-medium",
                "Manage your account access, payments, and preferences with ease"
            }
        }
    }
}

// ----- 2. Stats row ------------------------------------------------------------

/// Four source-like cards. The wallet and known authentication method are
/// verified session claims; profile age and credits remain unavailable until
/// authoritative reads are selected. `data_account` is intentionally ignored.
#[component]
fn AccountStatsRow(user: Option<User>) -> Element {
    let signed_in = user.is_some();
    let wallet = user
        .as_ref()
        .map(|user| user.address.trim())
        .filter(|address| !address.is_empty())
        .map(str::to_string);
    let auth_method = user
        .as_ref()
        .and_then(|user| verified_auth_method_label(&user.auth_method));

    rsx! {
        section {
            class: "account-stats-row grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 sm:gap-6",
            aria_label: "Account summary",
            // Current wallet: only the owner carried by the verified session.
            div {
                class: "account-stat-wallet card card-glass p-5 sm:p-6 shadow-xl border-2 border-blue-300/50",
                "data-account-stat-state": if wallet.is_some() { "verified" } else if signed_in { "unavailable" } else { "signed-out" },
                div { class: "flex items-center justify-between mb-4 text-2xl sm:text-3xl",
                    span { "👛" }
                    span { class: "text-xs font-semibold px-2 py-0.5 rounded border border-blue-200 bg-blue-50/50 text-blue-600",
                        if wallet.is_some() { "Session" } else if signed_in { "Unavailable" } else { "Signed out" }
                    }
                }
                div { class: "space-y-1",
                    div { class: "text-sm font-medium text-slate-400", "Current Address" }
                    if let Some(ref wallet) = wallet {
                        div { class: "text-sm font-mono font-bold text-foreground truncate",
                            "{wallet}"
                        }
                    } else if signed_in {
                        div { class: "text-sm font-semibold text-muted-foreground", "Not available" }
                    } else {
                        a { class: "text-sm font-semibold text-blue-500 hover:underline", href: ACCOUNT_SIGN_IN_PATH,
                            "Sign in to view"
                        }
                    }
                }
            }
            // Membership date requires an authoritative profile read.
            div {
                class: "account-stat-member card card-glass p-5 sm:p-6 shadow-xl border-2 border-green-300/50",
                "data-account-stat-state": "unavailable",
                div { class: "flex items-center justify-between mb-4 text-2xl sm:text-3xl",
                    span { "📅" }
                    span { class: "text-xs font-semibold px-2 py-0.5 rounded border border-slate-300/50 bg-secondary text-muted-foreground",
                        "Unavailable"
                    }
                }
                div { class: "space-y-1",
                    div { class: "text-sm font-medium text-slate-400", "Member Since" }
                    div { class: "text-lg font-bold text-muted-foreground", "Not available" }
                }
            }
            // Credit authority is unresolved; link to its truthful detail page.
            a {
                class: "account-stat-balance card card-glass p-5 sm:p-6 shadow-xl border-2 border-orange-300/50 block",
                "data-account-stat-state": "unavailable",
                href: "/account/credits",
                div { class: "flex items-center justify-between mb-4 text-2xl sm:text-3xl",
                    span { "💰" }
                    span { class: "text-xs font-semibold px-2 py-0.5 rounded border border-orange-200 bg-orange-50/50 text-orange-600",
                        "Unavailable"
                    }
                }
                div { class: "space-y-1",
                    div { class: "text-sm font-medium text-slate-400", "Available Balance" }
                    div { class: "text-lg font-bold text-muted-foreground", "Not available" }
                    div { class: "text-xs text-orange-500", "View credit status →" }
                }
            }
            // Authentication method: shown only when the session identifies it.
            div {
                class: "account-stat-method card card-glass p-5 sm:p-6 shadow-xl border-2 border-purple-300/50",
                "data-account-stat-state": if auth_method.is_some() { "verified" } else if signed_in { "unavailable" } else { "signed-out" },
                div { class: "flex items-center justify-between mb-4 text-2xl sm:text-3xl",
                    span { "🛡️" }
                    span { class: "text-xs font-semibold px-2 py-0.5 rounded border border-purple-200 bg-purple-50/50 text-purple-600",
                        if auth_method.is_some() { "Session" } else if signed_in { "Unavailable" } else { "Signed out" }
                    }
                }
                div { class: "space-y-1",
                    div { class: "text-sm font-medium text-slate-400", "Sign-in Method" }
                    if let Some(auth_method) = auth_method {
                        div { class: "text-lg font-bold text-foreground", "{auth_method}" }
                    } else if signed_in {
                        div { class: "text-lg font-bold text-muted-foreground", "Not available" }
                    } else {
                        div { class: "text-lg font-bold text-muted-foreground", "Sign in to view" }
                    }
                }
            }
        }
    }
}

fn verified_auth_method_label(method: &AuthMethod) -> Option<&'static str> {
    match method {
        AuthMethod::Wallet => Some("Wallet"),
        AuthMethod::Email => Some("Email"),
        AuthMethod::Demo => Some("Demo"),
        AuthMethod::OAuth => Some("OAuth"),
        AuthMethod::Siwe => Some("SIWE"),
        AuthMethod::Unknown => None,
    }
}

// ----- 3. Quick actions --------------------------------------------------------

/// 3 quick-action cards: Support Center, Privacy Control, Recent
/// Activity. Each is a gradient-border card with an icon, title,
/// short description, and a coloured chip + arrow on the right.
/// Mirrors `account-client.tsx` lines 187-235.
#[component]
fn AccountQuickActions() -> Element {
    rsx! {
        div { class: "account-quick-actions grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6",
            // Support Center
            a { class: "block group", href: "/contact",
                div { class: "card card-glass p-5 sm:p-6 relative overflow-hidden border-2 border-blue-300/50 hover:scale-105 transition-all duration-300",
                    div { class: "absolute top-4 right-4 w-4 h-4 rounded-full bg-gradient-to-r from-blue-400 to-cyan-500 blur-sm opacity-60" }
                    h3 { class: "text-lg sm:text-xl font-bold flex items-center gap-2",
                        span { class: "text-xl", "🛟" }
                        span { class: "bg-gradient-to-r from-blue-400 to-cyan-500 bg-clip-text text-transparent",
                            "Support Center"
                        }
                    }
                    p { class: "mt-2 text-sm text-slate-300", "Need help? Connect with our team" }
                    div { class: "mt-4 flex items-center justify-between",
                        span { class: "px-3 py-1 rounded-full text-xs font-semibold text-white bg-gradient-to-r from-blue-400 to-cyan-500",
                            "Contact"
                        }
                        span { class: "text-slate-400", "→" }
                    }
                }
            }
            // Privacy Control
            a { class: "block group", href: "/privacy",
                div { class: "card card-glass p-5 sm:p-6 relative overflow-hidden border-2 border-green-300/50 hover:scale-105 transition-all duration-300",
                    div { class: "absolute top-4 right-4 w-4 h-4 rounded-full bg-gradient-to-r from-green-400 to-emerald-500 blur-sm opacity-60" }
                    h3 { class: "text-lg sm:text-xl font-bold flex items-center gap-2",
                        span { class: "text-xl", "🔒" }
                        span { class: "bg-gradient-to-r from-green-400 to-emerald-500 bg-clip-text text-transparent",
                            "Privacy Control"
                        }
                    }
                    p { class: "mt-2 text-sm text-slate-300", "Review how account data is handled" }
                    div { class: "mt-4 flex items-center justify-between",
                        span { class: "px-3 py-1 rounded-full text-xs font-semibold text-white bg-gradient-to-r from-green-400 to-emerald-500",
                            "Settings"
                        }
                        span { class: "text-slate-400", "→" }
                    }
                }
            }
            // Recent Activity
            a { class: "block group", href: "/notifications",
                div { class: "card card-glass p-5 sm:p-6 relative overflow-hidden border-2 border-orange-300/50 hover:scale-105 transition-all duration-300",
                    div { class: "absolute top-4 right-4 w-4 h-4 rounded-full bg-gradient-to-r from-orange-400 to-pink-500 blur-sm opacity-60" }
                    h3 { class: "text-lg sm:text-xl font-bold flex items-center gap-2",
                        span { class: "text-xl", "🔔" }
                        span { class: "bg-gradient-to-r from-orange-400 to-pink-500 bg-clip-text text-transparent",
                            "Recent Activity"
                        }
                    }
                    p { class: "mt-2 text-sm text-slate-300", "Check your latest logs and alerts" }
                    div { class: "mt-4 flex items-center justify-between",
                        span { class: "px-3 py-1 rounded-full text-xs font-semibold text-white bg-gradient-to-r from-orange-400 to-pink-500",
                            "View Logs"
                        }
                        span { class: "text-slate-400", "→" }
                    }
                }
            }
        }
    }
}

// ----- 4. Access & Plans -------------------------------------------------------

/// Large rounded card with the "Access & Plans" header + a placeholder
/// for the `AccessOverview` slot. Mirrors
/// `account-client.tsx` lines 237-246 + `access-overview.tsx`. The
/// placeholder matches the OLD prod render when the API returns the
/// "Unable to load access details" error.
#[component]
fn AccessAndPlansSection() -> Element {
    rsx! {
        div { class: "account-access-plans card card-glass p-6 sm:p-8 lg:p-10 shadow-2xl border-2 border-indigo-200/50",
            "data-section": "account-access-plans",
            div { class: "flex items-center gap-3 mb-8",
                div { class: "p-3 bg-indigo-100 dark:bg-indigo-900/30 rounded-2xl",
                    Icon { name: "shield".to_string(), size: Some(24), class_name: Some("text-indigo-600 dark:text-indigo-400".to_string()) }
                }
                h2 { class: "text-2xl sm:text-3xl font-bold text-foreground", "Access & Plans" }
            }
            div {
                class: "p-6 rounded-2xl border border-red-200 bg-red-50/30 dark:bg-red-900/10",
                "data-access-state": "unavailable",
                role: "alert",
                div { class: "flex items-center gap-3",
                    Icon { name: "alert-triangle".to_string(), size: Some(20), class_name: Some("text-red-500".to_string()) }
                    p { class: "text-sm text-red-600 dark:text-red-400",
                        "Access and plan details are unavailable. No access level is being inferred."
                    }
                }
            }
        }
    }
}

// ----- 5. Transaction History --------------------------------------------------

/// Transaction History section. Mirrors
/// `account-client.tsx` lines 248-257 + `payment-history-tab.tsx`.
///
/// The BFF supplies an explicit state and an owner-scoped, bounded JSON
/// payload. Missing data remains unavailable; it is never treated as empty.
#[component]
fn PaymentHistorySection(address: Option<String>, load: PaymentHistoryLoad) -> Element {
    rsx! {
        PaymentHistoryTab {
            address,
            load,
            class: Some("account-payment-history".to_string()),
        }
    }
}

fn payment_history_load(ctx: &PageContext) -> PaymentHistoryLoad {
    let Some(user) = ctx.user.as_ref() else {
        return PaymentHistoryLoad::SignedOut;
    };

    match ctx
        .params
        .get(ACCOUNT_PAYMENT_HISTORY_STATE_PARAM)
        .map(String::as_str)
    {
        None | Some(ACCOUNT_PAYMENT_HISTORY_UNAVAILABLE) => PaymentHistoryLoad::Unavailable,
        Some(ACCOUNT_PAYMENT_HISTORY_MALFORMED) => PaymentHistoryLoad::Malformed,
        Some(ACCOUNT_PAYMENT_HISTORY_READY) | Some(ACCOUNT_PAYMENT_HISTORY_EMPTY) => {
            let expected_empty = ctx
                .params
                .get(ACCOUNT_PAYMENT_HISTORY_STATE_PARAM)
                .is_some_and(|state| state == ACCOUNT_PAYMENT_HISTORY_EMPTY);
            let Some(value) = ctx
                .params
                .get(ACCOUNT_PAYMENT_HISTORY_DATA_PARAM)
                .and_then(|raw| serde_json::from_str(raw).ok())
            else {
                return PaymentHistoryLoad::Malformed;
            };
            let Some(history) =
                decode_pay_history(value, &user.address, ACCOUNT_PAYMENT_HISTORY_MAX_ITEMS)
            else {
                return PaymentHistoryLoad::Malformed;
            };
            let is_empty = history.intents.is_empty()
                && history.escrows.is_empty()
                && history.total_intents == 0
                && history.total_escrows == 0;

            match (expected_empty, is_empty) {
                (true, true) => PaymentHistoryLoad::Empty,
                (false, false) => PaymentHistoryLoad::Ready(history),
                _ => PaymentHistoryLoad::Malformed,
            }
        }
        Some(_) => PaymentHistoryLoad::Malformed,
    }
}

// ----- 6. Notification Preferences ---------------------------------------------

/// Source-like notification categories without fabricated values or mutation.
/// Until an owner-scoped read/write contract exists, authenticated users see
/// an explicit unavailable state and signed-out users receive a native return
/// link. There are deliberately no inputs, signals, or success messages.
#[component]
fn NotificationPreferencesSection(signed_in: bool) -> Element {
    rsx! {
        div { class: "account-notification-prefs card card-glass p-6 sm:p-8 lg:p-10 shadow-2xl border-2 border-purple-200/50",
            "data-section": "account-notification-prefs",
            div { class: "flex items-center gap-3 mb-8",
                div { class: "p-3 bg-purple-100 dark:bg-purple-900/30 rounded-2xl",
                    Icon { name: "bell".to_string(), size: Some(24), class_name: Some("text-purple-600 dark:text-purple-400".to_string()) }
                }
                h2 { class: "text-2xl sm:text-3xl font-bold text-foreground", "Notification Preferences" }
            }
            div { class: "grid lg:grid-cols-12 gap-8",
                div { class: "lg:col-span-4 space-y-4",
                    p { class: "text-slate-300 text-base leading-relaxed",
                        if signed_in {
                            "Saved notification choices cannot be read or changed right now."
                        } else {
                            "Sign in before viewing wallet-owned notification preferences."
                        }
                    }
                    div { class: "flex flex-col gap-3 pt-2",
                        a { class: "btn btn-outline w-full justify-between group hover:border-purple-300 font-bold",
                            href: "/notifications",
                            span { "Browse All Alerts" }
                            span { "→" }
                        }
                    }
                }
                div { class: "lg:col-span-8",
                    div {
                        class: "rounded-2xl border-2 border-purple-200/50 bg-purple-500/5 p-5",
                        "data-preferences-state": if signed_in { "unavailable" } else { "signed-out" },
                        role: if signed_in { "alert" } else { "status" },
                        div { class: "flex items-start gap-3",
                            Icon {
                                name: if signed_in { "alert-circle".to_string() } else { "lock".to_string() },
                                size: Some(22),
                                class_name: Some("mt-0.5 text-purple-400".to_string()),
                            }
                            div { class: "min-w-0 flex-1",
                                h3 { class: "font-semibold text-foreground",
                                    if signed_in {
                                        "Notification preferences are unavailable"
                                    } else {
                                        "Sign in to view notification preferences"
                                    }
                                }
                                p { class: "mt-1 text-sm leading-6 text-muted-foreground",
                                    if signed_in {
                                        "No saved values were loaded, and no changes can be made from this read-only view."
                                    } else {
                                        "Preferences are private to the wallet that owns them."
                                    }
                                }
                                div { class: "mt-4 flex flex-wrap gap-3",
                                    if signed_in {
                                        a { class: "btn btn-sm btn-outline", href: ACCOUNT_PATH, "Retry" }
                                    } else {
                                        a { class: "btn btn-sm btn-primary", href: ACCOUNT_SIGN_IN_PATH, "Sign in" }
                                    }
                                }
                            }
                        }
                    }
                    if signed_in {
                        ul { class: "mt-4 grid gap-3 sm:grid-cols-2", aria_label: "Preference categories",
                            PreferenceReadOnlyRow { icon: "💹", label: "Analytics Alerts", description: "Price movements and portfolio alerts" }
                            PreferenceReadOnlyRow { icon: "🛡️", label: "Security Alerts", description: "Authentication and security warnings" }
                            PreferenceReadOnlyRow { icon: "👤", label: "Account Updates", description: "Account profile updates" }
                            PreferenceReadOnlyRow { icon: "⚙️", label: "System Status", description: "Maintenance and feature notices" }
                            PreferenceReadOnlyRow { icon: "🎁", label: "Promotions", description: "News and special offers" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PreferenceReadOnlyRow(
    icon: &'static str,
    label: &'static str,
    description: &'static str,
) -> Element {
    rsx! {
        li { class: "preference-read-only-row flex items-start justify-between gap-3 rounded-2xl border border-border bg-card p-4",
            div { class: "flex min-w-0 gap-3",
                span { class: "text-xl", aria_hidden: "true", "{icon}" }
                div {
                    p { class: "font-semibold text-foreground", "{label}" }
                    p { class: "text-xs text-muted-foreground", "{description}" }
                }
            }
            span { class: "shrink-0 rounded-full border border-border px-2 py-1 text-xs text-muted-foreground",
                "Not loaded"
            }
        }
    }
}

// ----- 7. Privacy & Data Security banner ---------------------------------------

/// Full-width indigo banner at the bottom of the OLD prod page.
/// Mirrors `account-client.tsx` lines 342-359.
#[component]
fn PrivacyBannerSection() -> Element {
    rsx! {
        div { class: "account-privacy-banner flex flex-col sm:flex-row items-center justify-between gap-6 p-8 rounded-3xl bg-indigo-600 text-white shadow-xl relative overflow-hidden",
            "data-section": "account-privacy-banner",
            div { class: "relative z-10 space-y-2 text-center sm:text-left",
                h3 { class: "text-xl font-bold flex items-center gap-2 justify-center sm:justify-start",
                    Icon { name: "lock".to_string(), size: Some(20) }
                    " Privacy & Data Security"
                }
                p { class: "text-indigo-100 text-sm max-w-lg",
                    "Review the privacy policy to understand how EPSX handles account data."
                }
            }
            a { class: "relative z-10 bg-white text-indigo-600 hover:bg-white/90 font-bold px-8 py-3 rounded-xl",
                href: "/privacy",
                "Read Policy"
            }
            // Decorative blur orbs (matches the OLD's decorative
            // background visuals).
            div { class: "absolute top-0 right-0 -mr-16 -mt-16 h-48 w-48 rounded-full bg-white/10 blur-3xl" }
            div { class: "absolute bottom-0 left-0 -ml-16 -mb-16 h-32 w-32 rounded-full bg-indigo-400/20 blur-2xl" }
        }
    }
}

// =============================================================================
// Tests
// =============================================================================
//
// - `test_render_smoke` — `render(&ctx)` returns non-empty HTML.
// - `test_section_markers` — every design-doc section marker is
//   present in the SSR'd HTML (7 markers).
// - `test_default_tab_gone` — the Wave-6A-Track-A 6-tab model has
//   been removed; the page no longer renders "account-tabs" or
//   "account-profile-tab" (those were tab-marker strings).
// - `test_hero_present` — the gradient "Account Settings" title
//   renders.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::user::AuthMethod;
    use crate::auth::user::User;
    use serde_json::json;

    fn authed_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "u-1".to_string(),
                address: "0x1234abcd".to_string(),
                chain_id: "1".to_string(),
                roles: vec!["user".to_string()],
                email: Some("test@epsx.io".to_string()),
                tier: Some("pro".to_string()),
                permissions: vec!["profile:read".to_string()],
                last_login_at: None,
                auth_method: AuthMethod::default(),
                display_name: Some("EPSX tester".to_string()),
            }),
            path: "/account".to_string(),
            ..Default::default()
        }
    }

    fn needle(marker: &str) -> [String; 5] {
        [
            format!("class=\"{}\"", marker),
            format!("class=\"{mark} ", mark = marker),
            format!(" {}\"", marker),
            format!(" {} ", marker),
            format!("data-section=\"{}\"", marker),
        ]
    }

    fn history_payload() -> serde_json::Value {
        json!({
            "address": "0x1234ABCD",
            "intents": [{
                "id": "intent-account-1",
                "chain_id": "1",
                "payer": "0x1234abcd",
                "payee": "0xmerchant",
                "amount": "20.00",
                "token_address": "0xtoken",
                "status": "confirmed",
                "escrow_id": null,
                "tx_hash": null,
                "description": null,
                "expires_at": null,
                "created_at": "2026-07-22T10:11:12Z",
                "updated_at": "2026-07-22T10:12:13Z"
            }],
            "escrows": [],
            "total_intents": 1,
            "total_escrows": 0
        })
    }

    fn render_html(ctx: &PageContext) -> String {
        let (_meta, element) = render(ctx);
        dioxus_ssr::render_element(element)
    }

    fn assert_account_stats_fail_closed(html: &str) {
        for unsupported_success in [
            "$0",
            "$0.00",
            "Join Now",
            "Web3 Vault",
            ">Active<",
            ">Secure<",
            "FABRICATED_MEMBER_DATE",
            "FABRICATED_METHOD",
            "FABRICATED_WALLET",
            "$9876.54",
        ] {
            assert!(
                !html.contains(unsupported_success),
                "unsupported account data must not render as success `{unsupported_success}`. Got: {html}"
            );
        }
        assert!(html.contains("data-account-stat-state=\"unavailable\""));
        assert!(html.contains("Not available"));
    }

    #[test]
    fn test_render_smoke() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        assert!(
            !html.is_empty(),
            "account page must render non-empty HTML. Got: {}",
            html
        );
        assert!(
            html.len() > 100,
            "account HTML is suspiciously short ({} bytes).",
            html.len()
        );
    }

    #[test]
    fn test_section_markers() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        for marker in &[
            "account-page",
            "account-settings-hero",
            "account-stats-row",
            "account-stat-wallet",
            "account-quick-actions",
            "account-access-plans",
            "account-payment-history",
            "account-notification-prefs",
            "account-privacy-banner",
        ] {
            let n = needle(marker);
            assert!(
                html.contains(&n[0])
                    || html.contains(&n[1])
                    || html.contains(&n[2])
                    || html.contains(&n[3])
                    || html.contains(&n[4]),
                "account page must contain section marker '{}'. Got: {}",
                marker,
                html
            );
        }
    }

    #[test]
    fn test_hero_present() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("Account Settings"),
            "page must render the 'Account Settings' hero title. Got: {}",
            html
        );
    }

    #[test]
    fn test_old_6_tab_model_removed() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        // The old 6-tab markers must NOT be present (T2 replaced
        // them with the OLD prod's section model).
        for old_marker in &[
            "account-tabs",
            "account-profile-tab",
            "account-subscription-tab",
            "account-usage-tab",
            "account-connected-tab",
            "account-danger-tab",
        ] {
            let n = needle(old_marker);
            assert!(
                !(html.contains(&n[0])
                    || html.contains(&n[1])
                    || html.contains(&n[2])
                    || html.contains(&n[3])),
                "T2 removed the old 6-tab model; section marker '{}' must not render. Got: {}",
                old_marker,
                html
            );
        }
    }

    #[test]
    fn test_wallet_comes_only_from_authenticated_session() {
        let html = render_html(&authed_ctx());
        assert!(
            html.contains("0x1234abcd"),
            "wallet field must show the authenticated session owner. Got: {}",
            html
        );
    }

    #[test]
    fn missing_malformed_and_canned_account_payloads_never_become_stats_success() {
        let missing_html = render_html(&authed_ctx());
        assert_account_stats_fail_closed(&missing_html);

        for raw in [
            "{not-json",
            r#"{"wallet_address":"FABRICATED_WALLET","member_since":"FABRICATED_MEMBER_DATE","available_balance":9876.54,"method":"FABRICATED_METHOD"}"#,
            r#"{"available_balance":0}"#,
            "{}",
        ] {
            let mut ctx = authed_ctx();
            ctx.params
                .insert("data_account".to_string(), raw.to_string());
            let html = render_html(&ctx);
            assert!(html.contains("0x1234abcd"));
            assert_account_stats_fail_closed(&html);
        }
    }

    #[test]
    fn stats_render_only_verified_session_owner_and_known_auth_method() {
        let user = User {
            id: "PRIVATE_USER_ID".to_string(),
            address: "0xverified-owner".to_string(),
            chain_id: "PRIVATE_CHAIN".to_string(),
            roles: vec!["INVENTED_ROLE".to_string()],
            email: Some("INVENTED_EMAIL@example.test".to_string()),
            tier: Some("INVENTED_TIER".to_string()),
            permissions: vec!["INVENTED_PERMISSION".to_string()],
            last_login_at: Some("INVENTED_PROFILE_DATE".to_string()),
            auth_method: AuthMethod::Siwe,
            display_name: Some("INVENTED_PROFILE_NAME".to_string()),
        };
        let html = dioxus_ssr::render_element(rsx! { AccountStatsRow { user: Some(user) } });

        assert!(html.contains("0xverified-owner"));
        assert!(html.contains(">SIWE<"));
        assert_eq!(
            html.matches("data-account-stat-state=\"verified\"").count(),
            2
        );
        for unsupported_claim in [
            "PRIVATE_USER_ID",
            "PRIVATE_CHAIN",
            "INVENTED_ROLE",
            "INVENTED_EMAIL",
            "INVENTED_TIER",
            "INVENTED_PERMISSION",
            "INVENTED_PROFILE_DATE",
            "INVENTED_PROFILE_NAME",
        ] {
            assert!(
                !html.contains(unsupported_claim),
                "stats must not expose unsupported session/profile claim `{unsupported_claim}`. Got: {html}"
            );
        }
        assert_account_stats_fail_closed(&html);
    }

    #[test]
    fn stats_escape_the_verified_owner_and_signed_out_navigation_is_native() {
        let mut user = authed_ctx().user.expect("test session");
        user.address = "<script>alert('owner')</script>".to_string();
        let escaped = dioxus_ssr::render_element(rsx! { AccountStatsRow { user: Some(user) } });
        assert!(!escaped.contains("<script>alert('owner')</script>"));
        assert!(escaped.contains("&#60;script&#62;alert(&#39;owner&#39;)&#60;/script&#62;"));

        let signed_out = dioxus_ssr::render_element(rsx! { AccountStatsRow { user: None } });
        assert!(signed_out.contains("data-account-stat-state=\"signed-out\""));
        assert!(signed_out.contains("href=\"/auth?return_url=%2Faccount\""));
        assert!(!signed_out.contains("Not Connected"));
        assert_account_stats_fail_closed(&signed_out);
    }

    #[test]
    fn preferences_are_explicitly_read_only_without_fake_mutation() {
        let html =
            dioxus_ssr::render_element(rsx! { NotificationPreferencesSection { signed_in: true } });

        assert!(html.contains("data-preferences-state=\"unavailable\""));
        assert!(html.contains("role=\"alert\""));
        assert!(html.contains("No saved values were loaded"));
        assert_eq!(html.matches("Not loaded").count(), 5);
        assert!(html.contains("href=\"/account\">Retry</a>"));
        for fake_mutation in [
            "<input",
            "type=\"checkbox\"",
            "checked=",
            "onchange",
            "updated successfully",
            "Advanced Settings",
            "notif-toggle-row",
        ] {
            assert!(
                !html.contains(fake_mutation),
                "read-only preferences must not expose fake mutation `{fake_mutation}`. Got: {html}"
            );
        }
    }

    #[test]
    fn signed_out_preferences_use_status_and_native_return_link() {
        let html = dioxus_ssr::render_element(
            rsx! { NotificationPreferencesSection { signed_in: false } },
        );

        assert!(html.contains("data-preferences-state=\"signed-out\""));
        assert!(html.contains("role=\"status\""));
        assert!(html.contains("Sign in to view notification preferences"));
        assert!(html.contains("href=\"/auth?return_url=%2Faccount\">Sign in</a>"));
        assert!(!html.contains("preference-read-only-row"));
        assert!(!html.contains("<input"));
    }

    #[test]
    fn payment_history_missing_payload_is_unavailable_not_empty() {
        let (_meta, element) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(element);
        assert!(html.contains("Payment history is temporarily unavailable"));
        assert!(!html.contains("No payment history yet"));
    }

    #[test]
    fn payment_history_signed_out_precedes_injected_state() {
        let mut ctx = PageContext {
            path: "/account".to_string(),
            ..Default::default()
        };
        ctx.params.insert(
            ACCOUNT_PAYMENT_HISTORY_STATE_PARAM.to_string(),
            "empty".to_string(),
        );
        ctx.params.insert(
            ACCOUNT_PAYMENT_HISTORY_DATA_PARAM.to_string(),
            json!({
                "address": "0x1234abcd",
                "intents": [],
                "escrows": [],
                "total_intents": 0,
                "total_escrows": 0
            })
            .to_string(),
        );

        assert_eq!(payment_history_load(&ctx), PaymentHistoryLoad::SignedOut);
        let (_meta, element) = render(&ctx);
        let html = dioxus_ssr::render_element(element);
        assert!(html.contains("Sign in to view payment history"));
        assert!(!html.contains("No payment history yet"));
    }

    #[test]
    fn payment_history_ready_and_empty_states_require_matching_payloads() {
        let mut ready = authed_ctx();
        ready.params.insert(
            ACCOUNT_PAYMENT_HISTORY_STATE_PARAM.to_string(),
            "ready".to_string(),
        );
        ready.params.insert(
            ACCOUNT_PAYMENT_HISTORY_DATA_PARAM.to_string(),
            history_payload().to_string(),
        );
        let (_meta, element) = render(&ready);
        let html = dioxus_ssr::render_element(element);
        assert!(html.contains("intent-account-1"));
        assert!(html.contains("20.00"));

        let mut empty = authed_ctx();
        empty.params.insert(
            ACCOUNT_PAYMENT_HISTORY_STATE_PARAM.to_string(),
            "empty".to_string(),
        );
        empty.params.insert(
            ACCOUNT_PAYMENT_HISTORY_DATA_PARAM.to_string(),
            json!({
                "address": "0x1234abcd",
                "intents": [],
                "escrows": [],
                "total_intents": 0,
                "total_escrows": 0
            })
            .to_string(),
        );
        assert_eq!(payment_history_load(&empty), PaymentHistoryLoad::Empty);

        empty.params.insert(
            ACCOUNT_PAYMENT_HISTORY_STATE_PARAM.to_string(),
            "ready".to_string(),
        );
        assert_eq!(payment_history_load(&empty), PaymentHistoryLoad::Malformed);
    }
}
