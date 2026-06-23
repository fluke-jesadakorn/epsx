//! /account — the EPSX account settings page.
//!
//! Wave 22 (T2) — pixel-perfect port of
//! `apps-old/frontend/app/account/page.tsx` (33 LoC) +
//! `components/account/account-client.tsx` (363 LoC). Mirrors the OLD
//! prod layout:
//!
//! Sections (render order, matches the OLD prod baseline PNG):
//! 1. `AccountSettingsHero`    — large gradient title "Account Settings"
//!    + tagline, on a yellow→orange→pink→purple gradient.
//! 2. `AccountStatsRow`        — 4 stat cards in a 1/2/4 grid:
//!    Current Address (Wallet), Member Since, Available Balance, Method.
//! 3. `AccountQuickActions`    — 3 gradient-border cards: Support Center,
//!    Privacy Control, Recent Activity — each linking to the relevant
//!    sub-page.
//! 4. `AccessAndPlansSection`  — large rounded card with the
//!    `AccessOverview` slot. In the port this is a static placeholder
//!    showing "Access overview" with an "Unable to load access
//!    details" alert (matches the OLD prod render when the API isn't
//!    wired).
//! 5. `PaymentHistorySection`  — Transaction History card with a
//!    placeholder table or empty state.
//! 6. `NotificationPreferencesSection` — copy of the OLD's grid-12
//!    notification preferences (5 toggles + Browse All Alerts +
//!    Advanced Settings buttons).
//! 7. `PrivacyBannerSection`   — full-width indigo banner with
//!    "Privacy & Data Security" + "Read Policy" CTA.
//!
//! Removed in T2: the Wave-6A-Track-A 6-tab model
//! (`ProfileTab`/`SubscriptionTab`/`UsageTab`/`NotificationsTab`/
//! `ConnectedAccountsTab`/`DangerZoneTab`). The OLD prod does not use
//! tabs; the 7 sections above replace them.
//!
//! All section markers are asserted in the `tests` module below.

use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Account");
    (meta, rsx! { RenderAccount { ctx: ctx.clone() } })
}

#[component]
fn RenderAccount(ctx: PageContext) -> Element {
    // T2: pull live data from `data_account` when the BFF has wired
    // it up. When absent (the common dev case), we use the
    // placeholder defaults that match the OLD prod render: a
    // connected address, a "Join Now" member-since, $0 balance, and
    // "Web3 Vault" auth method.
    let data: Option<AccountData> = ctx.params.get("data_account")
        .and_then(|s| serde_json::from_str(s).ok());

    let wallet = data.as_ref()
        .and_then(|d| d.wallet_address.clone())
        .or_else(|| ctx.user.as_ref().map(|u| u.address.clone()))
        .unwrap_or_else(|| "Not Connected".to_string());
    let member_since = data.as_ref()
        .and_then(|d| d.member_since.clone())
        .unwrap_or_else(|| "Join Now".to_string());
    let available_balance = data.as_ref()
        .map(|d| d.available_balance)
        .unwrap_or(0.0);
    let method = data.as_ref()
        .and_then(|d| d.method.clone())
        .unwrap_or_else(|| "Web3 Vault".to_string());
    let method_pretty = if method == "wallet" || method.is_empty() {
        "Web3 Vault".to_string()
    } else {
        method
    };

    rsx! {
        MainLayout { ctx: ctx.clone(),
            // T2: removed the `<AuthGate>` wrapper — the OLD prod
            // page is public-readable (see apps-old/frontend/middleware.ts
            // publicRoutes: '/account*'). For anonymous visitors we
            // render the OLD prod layout with the placeholder set
            // (Not Connected / Join Now / $0 / Web3 Vault). Authed
            // users get the same layout but with the wallet address
            // + member-since + balance filled in from `data_account`.
            // Wave 49 T2 (Plan 13) — use the `page-bg-app` body
            // class (set via PageMeta::app) to reproduce prod's
            // purple/magenta radial-glow body gradient. The full
            // MarketingBackground wrapper would render visible orbs
            // that prod does NOT show.
            div { class: "container page-content account-settings-page",
                "data-section": "account-page",
                // 1. Hero
                AccountSettingsHero {}
                // 2. 4 stat cards
                div { class: "mt-8",
                    AccountStatsRow {
                        wallet: wallet.clone(),
                        member_since: member_since.clone(),
                        available_balance,
                        method: method_pretty.clone(),
                    }
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
                    PaymentHistorySection {}
                }
                // 6. Notification Preferences
                div { class: "mt-8",
                    NotificationPreferencesSection {}
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

/// 4 stat cards: Current Address (Wallet), Member Since, Available
/// Balance, Method. Each card has a coloured border (blue/green/
/// orange/purple) + a small badge in the top-right (Wallet/Active/
/// Credits/Secure) + a primary icon (emoji or lucide). Mirrors the
/// OLD prod render in `account-client.tsx` lines 142-184.
#[component]
fn AccountStatsRow(
    wallet: String,
    member_since: String,
    available_balance: f64,
    method: String,
) -> Element {
    let balance_str = if available_balance == 0.0 {
        "$0".to_string()
    } else {
        format!("${:.2}", available_balance)
    };
    rsx! {
        div { class: "account-stats-row grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 sm:gap-6",
            // Current Address (Wallet)
            div { class: "account-stat-wallet card card-glass p-5 sm:p-6 shadow-xl border-2 border-blue-300/50",
                div { class: "flex items-center justify-between mb-4 text-2xl sm:text-3xl",
                    span { "👛" }
                    span { class: "text-xs font-semibold px-2 py-0.5 rounded border border-blue-200 bg-blue-50/50 text-blue-600",
                        "Wallet"
                    }
                }
                div { class: "space-y-1",
                    div { class: "text-sm font-medium text-slate-400", "Current Address" }
                    div { class: "text-sm font-mono font-bold text-foreground truncate",
                        "{wallet}"
                    }
                }
            }
            // Member Since (Active)
            div { class: "account-stat-member card card-glass p-5 sm:p-6 shadow-xl border-2 border-green-300/50",
                div { class: "flex items-center justify-between mb-4 text-2xl sm:text-3xl",
                    span { "✅" }
                    span { class: "text-xs font-semibold px-2 py-0.5 rounded border border-green-200 bg-green-50/50 text-green-600",
                        "Active"
                    }
                }
                div { class: "space-y-1",
                    div { class: "text-sm font-medium text-slate-400", "Member Since" }
                    div { class: "text-lg font-bold text-foreground",
                        "{member_since}"
                    }
                }
            }
            // Available Balance (Credits)
            div { class: "account-stat-balance card card-glass p-5 sm:p-6 shadow-xl border-2 border-orange-300/50",
                div { class: "flex items-center justify-between mb-4 text-2xl sm:text-3xl",
                    span { "💰" }
                    span { class: "text-xs font-semibold px-2 py-0.5 rounded border border-orange-200 bg-orange-50/50 text-orange-600",
                        "Credits"
                    }
                }
                div { class: "space-y-1",
                    div { class: "text-sm font-medium text-slate-400", "Available Balance" }
                    div { class: "text-lg font-bold text-foreground",
                        "{balance_str}"
                    }
                }
            }
            // Method (Secure)
            div { class: "account-stat-method card card-glass p-5 sm:p-6 shadow-xl border-2 border-purple-300/50",
                div { class: "flex items-center justify-between mb-4 text-2xl sm:text-3xl",
                    span { "🛡️" }
                    span { class: "text-xs font-semibold px-2 py-0.5 rounded border border-purple-200 bg-purple-50/50 text-purple-600",
                        "Secure"
                    }
                }
                div { class: "space-y-1",
                    div { class: "text-sm font-medium text-slate-400", "Method" }
                    div { class: "text-lg font-bold text-foreground capitalize",
                        "{method}"
                    }
                }
            }
        }
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
                    p { class: "mt-2 text-sm text-slate-300", "Manage your data and visibility settings" }
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
            div { class: "p-6 rounded-2xl border border-red-200 bg-red-50/30 dark:bg-red-900/10",
                div { class: "flex items-center gap-3",
                    Icon { name: "alert-triangle".to_string(), size: Some(20), class_name: Some("text-red-500".to_string()) }
                    p { class: "text-sm text-red-600 dark:text-red-400",
                        "Unable to load access details."
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
/// wave49(slice-5): now renders the `PaymentHistoryTab` from
/// `crate::components::account::*` (the slice-4 shared component).
/// Passes the SSR-resolved `static_history` prop when the
/// caller pre-fetched it; falls through to the empty state when
/// not. The page-level rendering keeps the
/// `data-section="account-payment-history"` marker that the
/// existing pixel-diff tests assert on.
#[component]
fn PaymentHistorySection() -> Element {
    use crate::components::account::{PaymentHistoryTab, PayHistory};
    // The BFF should pre-fetch the history server-side via
    // `GET /api/v1/pay/history/{address}` and inject the JSON
    // here. Slice-5 ships the shell; slice-6+ will wire the
    // BFF-side fetch (see apps/frontend/src/ssr.rs::fetch_page_data).
    let static_history: Option<PayHistory> = None;
    rsx! {
        PaymentHistoryTab {
            address: None,
            static_history,
            class: Some("account-payment-history".to_string()),
        }
    }
}

// ----- 6. Notification Preferences ---------------------------------------------

/// 5 notification preference toggles + 2 secondary buttons
/// ("Browse All Alerts" / "Advanced Settings"). Mirrors
/// `account-client.tsx` lines 259-340. Toggling is local-state-only
/// (no BFF integration); the BFF POST is a Wave-22 follow-up.
#[component]
fn NotificationPreferencesSection() -> Element {
    // Default to the same set the OLD initializes: analytics/security/
    // account ON, system/marketing OFF.
    let mut analytics_on = use_signal(|| true);
    let mut security_on = use_signal(|| true);
    let mut account_on = use_signal(|| true);
    let mut system_on = use_signal(|| false);
    let mut marketing_on = use_signal(|| false);
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
                        "Choose exactly what you want to be notified about. We'll send alerts via web push to keep you updated."
                    }
                    div { class: "flex flex-col gap-3 pt-2",
                        a { class: "btn btn-outline w-full justify-between group hover:border-purple-300 font-bold",
                            href: "/notifications",
                            span { "Browse All Alerts" }
                            span { "→" }
                        }
                        a { class: "btn btn-outline w-full justify-between group font-bold",
                            href: "/notifications/preferences",
                            span { "Advanced Settings" }
                            span { "✨" }
                        }
                    }
                }
                div { class: "lg:col-span-8",
                    div { class: "grid sm:grid-cols-2 gap-4",
                        NotifToggleRow { icon: "💹", label: "Analytics Alerts", desc: "Price movements & portfolio", on_signal: analytics_on }
                        NotifToggleRow { icon: "🛡️", label: "Security Alerts", desc: "Auth & security warnings", on_signal: security_on }
                        NotifToggleRow { icon: "👤", label: "Account Updates", desc: "Profile & subscription", on_signal: account_on }
                        NotifToggleRow { icon: "⚙️", label: "System Status", desc: "Maintenance & features", on_signal: system_on }
                        NotifToggleRow { icon: "🎁", label: "Promotions", desc: "News & special offers", on_signal: marketing_on }
                    }
                }
            }
        }
    }
}

#[component]
fn NotifToggleRow(
    icon: String,
    label: String,
    desc: String,
    on_signal: Signal<bool>,
) -> Element {
    let mut sig = on_signal;
    rsx! {
        label { class: "cursor-pointer group notif-toggle-row block",
            div { class: "flex items-center justify-between p-4 rounded-2xl bg-card border-2 border-border group-hover:border-purple-200 transition-all duration-200",
                div { class: "flex gap-4",
                    div { class: "text-2xl mt-1", "{icon}" }
                    div {
                        div { class: "font-bold text-foreground group-hover:text-purple-400 transition-colors", "{label}" }
                        div { class: "text-xs text-slate-400", "{desc}" }
                    }
                }
                input {
                    r#type: "checkbox",
                    checked: *sig.read(),
                    onchange: move |e| sig.set(e.checked()),
                    class: "h-5 w-5 rounded-lg border-2 border-slate-300 text-purple-600 focus:ring-purple-500 cursor-pointer",
                }
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
                    "Your account data is secured with industrial-grade encryption and protocol-level security."
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

// ----- Data model --------------------------------------------------------------

/// T2: data model for the account BFF fetch. Parsed from the
/// `data_account` query param. When the BFF doesn't supply it, the
/// page renders the OLD "Not Connected / Join Now / $0 / Web3 Vault"
/// placeholder set.
#[derive(Clone, Debug, serde::Deserialize, PartialEq)]
struct AccountData {
    #[serde(default)]
    wallet_address: Option<String>,
    #[serde(default)]
    member_since: Option<String>,
    #[serde(default)]
    available_balance: f64,
    #[serde(default)]
    method: Option<String>,
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
    use crate::auth::user::User;
    use crate::auth::user::AuthMethod;

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

    fn needle(marker: &str) -> [String; 4] {
        [
            format!("class=\"{}\"", marker),
            format!("class=\"{mark} ", mark = marker),
            format!(" {}\"", marker),
            format!(" {} ", marker),
        ]
    }

    #[test]
    fn test_render_smoke() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        assert!(!html.is_empty(), "account page must render non-empty HTML. Got: {}", html);
        assert!(html.len() > 100, "account HTML is suspiciously short ({} bytes).", html.len());
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
                html.contains(&n[0]) || html.contains(&n[1]) || html.contains(&n[2]) || html.contains(&n[3]),
                "account page must contain section marker '{}'. Got: {}",
                marker, html
            );
        }
    }

    #[test]
    fn test_hero_present() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("Account Settings"),
            "page must render the 'Account Settings' hero title. Got: {}", html);
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
                !(html.contains(&n[0]) || html.contains(&n[1]) || html.contains(&n[2]) || html.contains(&n[3])),
                "T2 removed the old 6-tab model; section marker '{}' must not render. Got: {}",
                old_marker, html
            );
        }
    }

    #[test]
    fn test_default_wallet_is_not_connected() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        // When the BFF hasn't wired `data_account` (the default
        // dev case), the wallet field shows "Not Connected" —
        // matching the OLD prod render.
        assert!(html.contains("Not Connected"),
            "default wallet field must show 'Not Connected'. Got: {}", html);
    }
}
