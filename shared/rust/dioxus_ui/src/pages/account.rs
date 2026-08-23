//! `/account` — production-safe account overview.
//!
//! The page preserves the development UI's section composition while only
//! presenting claims available from the verified local session. Unsupported
//! profile statistics, credit balance, and access details fail closed as
//! unavailable; owner preferences use a separate strict SSR read and also fail
//! closed on missing or partial compatibility payloads. Payment history retains
//! its separate strict, owner-scoped read contract.

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
use crate::pages::account_credits::{credit_balance_load, CreditBalanceLoad};
use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

const ACCOUNT_PATH: &str = "/account";
const ACCOUNT_SIGN_IN_PATH: &str = "/auth?return_url=%2Faccount";
pub const ACCOUNT_NOTIFICATION_PREFERENCES_DATA_PARAM: &str =
    "data_account_notification_preferences";
pub const ACCOUNT_NOTIFICATION_PREFERENCES_STATE_PARAM: &str =
    "data_account_notification_preferences_state";
pub const ACCOUNT_NOTIFICATION_PREFERENCES_FORM_STATE_PARAM: &str =
    "data_account_notification_preferences_form_state";
const ACCOUNT_NOTIFICATION_PREFERENCES_READY: &str = "ready";
const ACCOUNT_NOTIFICATION_PREFERENCES_UNAVAILABLE: &str = "unavailable";
const ACCOUNT_NOTIFICATION_PREFERENCES_MALFORMED: &str = "malformed";
pub const ACCOUNT_PROFILE_DATA_PARAM: &str = "data_account_profile";
pub const ACCOUNT_PROFILE_STATE_PARAM: &str = "data_account_profile_state";
pub const ACCOUNT_ACCESS_DATA_PARAM: &str = "data_account_access";
pub const ACCOUNT_ACCESS_STATE_PARAM: &str = "data_account_access_state";
pub const ACCOUNT_PLAN_PAYMENTS_DATA_PARAM: &str = "data_account_plan_payments";
pub const ACCOUNT_PLAN_PAYMENTS_STATE_PARAM: &str = "data_account_plan_payments_state";
pub const ACCOUNT_DATA_READY: &str = "ready";
pub const ACCOUNT_DATA_EMPTY: &str = "empty";
pub const ACCOUNT_DATA_UNAVAILABLE: &str = "unavailable";
pub const ACCOUNT_DATA_MALFORMED: &str = "malformed";
pub const ACCOUNT_PLAN_PAYMENTS_MAX_ITEMS: usize = 10;

const ACCOUNT_MAX_TEXT: usize = 512;
const ACCOUNT_MAX_WALLET: usize = 128;
const ACCOUNT_MAX_PERMISSIONS: usize = 256;
const ACCOUNT_MAX_GROUPS: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AccountProfileProjection {
    pub wallet_address: String,
    pub created_at: String,
    pub last_login: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AccountProfileWire {
    wallet_address: String,
    permissions: Vec<String>,
    auth_method: String,
    created_at: String,
    last_login: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AccountAccessGroupProjection {
    pub id: String,
    pub name: String,
    pub expires_at: Option<String>,
    pub permissions: Vec<String>,
    pub source_type: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AccountAccessProjection {
    pub current_tier: String,
    pub groups: Vec<AccountAccessGroupProjection>,
    pub direct_permissions: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AccountAccessWire {
    current_tier: String,
    groups: Vec<AccountAccessGroupWire>,
    direct_permissions: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AccountAccessGroupWire {
    id: String,
    name: String,
    description: Option<String>,
    expires_at: Option<String>,
    permissions: Vec<String>,
    source_type: String,
    assigned_at: Option<String>,
    assigned_by: Option<String>,
    days_remaining: Option<i64>,
    can_renew: bool,
    renewal_price: Option<Value>,
    billing_cycle: Option<String>,
    tier_level: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AccountPlanPaymentProjection {
    pub id: String,
    pub amount: String,
    pub currency: String,
    pub status: String,
    pub tx_hash: Option<String>,
    pub plan_name: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub payment_reference: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AccountPlanPaymentsProjection {
    pub payments: Vec<AccountPlanPaymentProjection>,
    pub total: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AccountPlanPaymentsDataWire {
    payments: Vec<AccountPlanPaymentWire>,
    pagination: AccountPlanPaymentsPaginationWire,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AccountPlanPaymentWire {
    id: String,
    amount: serde_json::Number,
    currency: String,
    status: String,
    tx_hash: Option<String>,
    plan_name: Option<String>,
    permissions_granted: Vec<String>,
    created_at: String,
    completed_at: Option<String>,
    payment_reference: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AccountPlanPaymentsPaginationWire {
    page: usize,
    per_page: usize,
    total: usize,
    total_pages: usize,
}

pub fn decode_account_profile(
    value: Value,
    expected_owner: &str,
) -> Option<AccountProfileProjection> {
    if !value.get("success")?.as_bool()? {
        return None;
    }
    let profile: AccountProfileWire = serde_json::from_value(value.get("data")?.clone()).ok()?;
    if !profile.wallet_address.eq_ignore_ascii_case(expected_owner)
        || !valid_account_text(&profile.wallet_address, ACCOUNT_MAX_WALLET)
        || profile.permissions.len() > ACCOUNT_MAX_PERMISSIONS
        || !profile
            .permissions
            .iter()
            .all(|permission| valid_account_text(permission, ACCOUNT_MAX_TEXT))
        || !valid_account_text(&profile.auth_method, ACCOUNT_MAX_TEXT)
        || !valid_account_timestamp(&profile.created_at)
        || !valid_account_timestamp(&profile.last_login)
    {
        return None;
    }
    Some(AccountProfileProjection {
        wallet_address: profile.wallet_address,
        created_at: profile.created_at,
        last_login: profile.last_login,
    })
}

pub fn decode_account_access(value: Value) -> Option<AccountAccessProjection> {
    if !value.get("success")?.as_bool()? {
        return None;
    }
    let access: AccountAccessWire = serde_json::from_value(value.get("data")?.clone()).ok()?;
    if !valid_account_text(&access.current_tier, ACCOUNT_MAX_TEXT)
        || access.groups.len() > ACCOUNT_MAX_GROUPS
        || access.direct_permissions.len() > ACCOUNT_MAX_PERMISSIONS
        || !access
            .direct_permissions
            .iter()
            .all(|permission| valid_account_text(permission, ACCOUNT_MAX_TEXT))
        || !access.groups.iter().all(valid_account_access_group)
    {
        return None;
    }
    Some(AccountAccessProjection {
        current_tier: access.current_tier,
        groups: access
            .groups
            .into_iter()
            .map(|group| AccountAccessGroupProjection {
                id: group.id,
                name: group.name,
                expires_at: group.expires_at,
                permissions: group.permissions,
                source_type: group.source_type,
            })
            .collect(),
        direct_permissions: access.direct_permissions,
    })
}

fn valid_account_access_group(group: &AccountAccessGroupWire) -> bool {
    valid_account_text(&group.id, ACCOUNT_MAX_TEXT)
        && valid_account_text(&group.name, ACCOUNT_MAX_TEXT)
        && group
            .description
            .as_deref()
            .is_none_or(|value| valid_account_text(value, ACCOUNT_MAX_TEXT))
        && group
            .expires_at
            .as_deref()
            .is_none_or(valid_account_timestamp)
        && group.permissions.len() <= ACCOUNT_MAX_PERMISSIONS
        && group
            .permissions
            .iter()
            .all(|permission| valid_account_text(permission, ACCOUNT_MAX_TEXT))
        && valid_account_text(&group.source_type, ACCOUNT_MAX_TEXT)
        && group
            .assigned_at
            .as_deref()
            .is_none_or(valid_account_timestamp)
        && group
            .assigned_by
            .as_deref()
            .is_none_or(|value| valid_account_text(value, ACCOUNT_MAX_WALLET))
        && group.days_remaining.is_none_or(|days| days >= 0)
        && group
            .billing_cycle
            .as_deref()
            .is_none_or(|value| valid_account_text(value, ACCOUNT_MAX_TEXT))
        && group.tier_level >= 0
        && (!group.can_renew || group.renewal_price.is_some())
}

pub fn decode_account_plan_payments(
    value: Value,
    max_items: usize,
) -> Option<AccountPlanPaymentsProjection> {
    if max_items == 0 || !value.get("success")?.as_bool()? {
        return None;
    }
    let data: AccountPlanPaymentsDataWire =
        serde_json::from_value(value.get("data")?.clone()).ok()?;
    if data.pagination.page != 1
        || data.pagination.per_page != max_items
        || data.payments.len() > max_items
        || data.pagination.total < data.payments.len()
        || (data.payments.is_empty() && data.pagination.total != 0)
        || data.pagination.total_pages
            != data
                .pagination
                .total
                .div_ceil(data.pagination.per_page.max(1))
        || !data.payments.iter().all(valid_account_plan_payment)
    {
        return None;
    }
    Some(AccountPlanPaymentsProjection {
        total: data.pagination.total,
        payments: data
            .payments
            .into_iter()
            .map(|payment| AccountPlanPaymentProjection {
                id: payment.id,
                amount: payment.amount.to_string(),
                currency: payment.currency,
                status: payment.status,
                tx_hash: payment.tx_hash,
                plan_name: payment.plan_name,
                created_at: payment.created_at,
                completed_at: payment.completed_at,
                payment_reference: payment.payment_reference,
            })
            .collect(),
    })
}

fn valid_account_plan_payment(payment: &AccountPlanPaymentWire) -> bool {
    valid_account_text(&payment.id, ACCOUNT_MAX_TEXT)
        && payment.amount.as_f64().is_some_and(f64::is_finite)
        && valid_account_text(&payment.currency, 32)
        && valid_account_text(&payment.status, 64)
        && payment
            .tx_hash
            .as_deref()
            .is_none_or(|value| valid_account_text(value, ACCOUNT_MAX_TEXT))
        && payment
            .plan_name
            .as_deref()
            .is_none_or(|value| valid_account_text(value, ACCOUNT_MAX_TEXT))
        && payment.permissions_granted.len() <= ACCOUNT_MAX_PERMISSIONS
        && payment
            .permissions_granted
            .iter()
            .all(|permission| valid_account_text(permission, ACCOUNT_MAX_TEXT))
        && valid_account_timestamp(&payment.created_at)
        && payment
            .completed_at
            .as_deref()
            .is_none_or(valid_account_timestamp)
        && valid_account_text(&payment.payment_reference, ACCOUNT_MAX_TEXT)
}

fn valid_account_text(value: &str, max_len: usize) -> bool {
    !value.is_empty()
        && value.trim() == value
        && value.len() <= max_len
        && !value.chars().any(char::is_control)
}

fn valid_account_timestamp(value: &str) -> bool {
    valid_account_text(value, 64) && DateTime::parse_from_rfc3339(value).is_ok()
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum AccountProfileLoad {
    Ready(AccountProfileProjection),
    Unavailable,
    Malformed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum AccountAccessLoad {
    Ready(AccountAccessProjection),
    Unavailable,
    Malformed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum AccountPlanPaymentsLoad {
    Ready(AccountPlanPaymentsProjection),
    Empty,
    Unavailable,
    Malformed,
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Account");
    (meta, rsx! { RenderAccount { ctx: ctx.clone() } })
}

#[component]
fn RenderAccount(ctx: PageContext) -> Element {
    let session_user = ctx.user.clone();
    let profile_load = account_profile_load(&ctx);
    let access_load = account_access_load(&ctx);
    let credit_balance = credit_balance_load(&ctx);
    let plan_payments_load = account_plan_payments_load(&ctx);
    let payment_history_address = ctx.user.as_ref().map(|user| user.address.clone());
    let payment_history_load = payment_history_load(&ctx);
    let notification_preferences_load = notification_preferences_load(&ctx);
    let notification_preferences_form_state = notification_preferences_form_state(&ctx);

    rsx! {
        MainLayout { ctx: ctx.clone(),
            div { class: "account-prod-page relative min-h-screen overflow-hidden px-3 pb-20 sm:px-6",
                div { class: "pointer-events-none fixed inset-0 overflow-hidden", aria_hidden: "true",
                    div { class: "absolute left-20 top-20 h-32 w-32 rounded-full bg-gradient-to-r from-yellow-400/20 to-orange-500/20 blur-xl" }
                    div { class: "absolute right-32 top-40 h-24 w-24 rounded-full bg-gradient-to-r from-pink-400/20 to-purple-500/20 blur-lg" }
                    div { class: "absolute bottom-32 left-1/3 h-28 w-28 rounded-full bg-gradient-to-r from-orange-400/15 to-yellow-500/15 blur-xl" }
                }
                // Match the source account frame: the page owns the 1.5rem
                // outer inset, while the content frame contributes another
                // 1.5rem and starts 1.5rem below the sticky header.
                div { class: "page-content account-settings-page relative z-10 px-6 pt-6",
                    "data-section": "account-page",
                    // 1. Hero
                    AccountSettingsHero {}
                    // 2. 4 stat cards
                    div { class: "mt-8",
                        AccountStatsRow { user: session_user.clone(), profile_load, credit_balance }
                    }
                    // 3. 3 quick-action cards
                    div { class: "mt-12",
                        AccountQuickActions {}
                    }
                    // 4. Access & Plans
                    div { class: "mt-12",
                        AccessAndPlansSection { load: access_load }
                    }
                    // 5. Plan payments confirmed by the subscription backend.
                    div { class: "mt-8",
                        PlanPaymentsSection { load: plan_payments_load }
                    }
                    // 6. Pay intents and escrow activity.
                    div { class: "mt-8",
                        PaymentHistorySection {
                            address: payment_history_address,
                            load: payment_history_load,
                        }
                    }
                    // 7. Notification Preferences
                    div { class: "mt-8",
                        NotificationPreferencesSection {
                            load: notification_preferences_load,
                            form_state: notification_preferences_form_state,
                        }
                    }
                    // 8. Privacy & Data Security banner
                    div { class: "mt-8",
                        PrivacyBannerSection {}
                    }
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
        div { class: "account-settings-hero text-center mb-6 sm:mb-12",
            "data-section": "account-settings-hero",
            h1 { class: "text-3xl sm:text-5xl font-bold flex items-center justify-center gap-2 sm:gap-3",
                span { class: "text-foreground", "👤" }
                span { class: "bg-gradient-to-r from-yellow-600 via-orange-600 via-pink-600 to-purple-600 bg-clip-text text-transparent",
                    "Account Settings"
                }
            }
            p { class: "mt-3 sm:mt-4 text-sm sm:text-lg text-slate-300 max-w-2xl mx-auto font-medium",
                "Manage your account access, payments, and preferences with ease"
            }
        }
    }
}

// ----- 2. Stats row ------------------------------------------------------------

/// Four source-like cards. The wallet and known authentication method are
/// verified session claims; profile age and credits are independent strict
/// owner-scoped reads so either card can remain truthful if the other fails.
#[component]
fn AccountStatsRow(
    user: Option<User>,
    profile_load: AccountProfileLoad,
    credit_balance: CreditBalanceLoad,
) -> Element {
    let signed_in = user.is_some();
    let wallet = user
        .as_ref()
        .map(|user| user.address.trim())
        .filter(|address| !address.is_empty())
        .map(str::to_string);
    let auth_method = user
        .as_ref()
        .and_then(|user| verified_auth_method_label(&user.auth_method));
    let member_since = match &profile_load {
        AccountProfileLoad::Ready(profile) => Some(profile.created_at[..10].to_string()),
        _ => None,
    };
    let available_credits = match &credit_balance {
        CreditBalanceLoad::Ready(balance) => Some(balance.available_balance.clone()),
        _ => None,
    };

    rsx! {
        section {
            class: "account-stats-row grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 sm:gap-6",
            aria_label: "Account summary",
            // Current wallet: only the owner carried by the verified session.
            div {
                class: "account-stat-wallet card card-glass p-3 sm:p-6 shadow-xl border-2 border-blue-300/50",
                "data-account-stat-state": if wallet.is_some() { "verified" } else if signed_in { "unavailable" } else { "signed-out" },
                div { class: "flex items-center justify-between mb-2 sm:mb-4 text-xl sm:text-3xl",
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
            // Membership date is sourced from the owner profile endpoint.
            div {
                class: "account-stat-member card card-glass p-3 sm:p-6 shadow-xl border-2 border-green-300/50",
                "data-account-stat-state": if member_since.is_some() { "verified" } else if matches!(profile_load, AccountProfileLoad::Malformed) { "malformed" } else { "unavailable" },
                div { class: "flex items-center justify-between mb-2 sm:mb-4 text-xl sm:text-3xl",
                    span { "📅" }
                    span { class: "text-xs font-semibold px-2 py-0.5 rounded border border-green-200 bg-green-50/50 text-green-600",
                        if member_since.is_some() { "Verified" } else { "Unavailable" }
                    }
                }
                div { class: "space-y-1",
                    div { class: "text-sm font-medium text-slate-400", "Member Since" }
                    if let Some(ref member_since) = member_since {
                        time { class: "text-lg font-bold text-foreground", datetime: member_since.clone(), "{member_since}" }
                    } else {
                        div { class: "text-lg font-bold text-muted-foreground", "Not available" }
                    }
                }
            }
            // Credit authority is the payments database owner endpoint.
            a {
                class: "account-stat-balance card card-glass p-3 sm:p-6 shadow-xl border-2 border-orange-300/50 block",
                "data-account-stat-state": if available_credits.is_some() { "verified" } else if matches!(credit_balance, CreditBalanceLoad::Malformed) { "malformed" } else { "unavailable" },
                href: "/account/credits",
                div { class: "flex items-center justify-between mb-2 sm:mb-4 text-xl sm:text-3xl",
                    span { "💰" }
                    span { class: "text-xs font-semibold px-2 py-0.5 rounded border border-orange-200 bg-orange-50/50 text-orange-600",
                        if available_credits.is_some() { "Verified" } else { "Unavailable" }
                    }
                }
                div { class: "space-y-1",
                    div { class: "text-sm font-medium text-slate-400", "Available Balance" }
                    if let Some(ref available_credits) = available_credits {
                        div { class: "text-lg font-bold text-foreground", "{available_credits} credits" }
                    } else {
                        div { class: "text-lg font-bold text-muted-foreground", "Not available" }
                    }
                    // Keep the truthful navigation affordance available to
                    // assistive technology without adding a fourth visible
                    // line that changes the source card's height.
                    div { class: "sr-only", "View credit status →" }
                }
            }
            // Authentication method: shown only when the session identifies it.
            div {
                class: "account-stat-method card card-glass p-3 sm:p-6 shadow-xl border-2 border-purple-300/50",
                "data-account-stat-state": if auth_method.is_some() { "verified" } else if signed_in { "unavailable" } else { "signed-out" },
                div { class: "flex items-center justify-between mb-2 sm:mb-4 text-xl sm:text-3xl",
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
                div { class: "card card-glass p-3 sm:p-6 relative overflow-hidden border-2 border-blue-300/50 hover:scale-105 transition-all duration-300",
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
                div { class: "card card-glass p-3 sm:p-6 relative overflow-hidden border-2 border-green-300/50 hover:scale-105 transition-all duration-300",
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
                div { class: "card card-glass p-3 sm:p-6 relative overflow-hidden border-2 border-orange-300/50 hover:scale-105 transition-all duration-300",
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

/// Backend-authoritative access groups and their granted permissions.
#[component]
fn AccessAndPlansSection(load: AccountAccessLoad) -> Element {
    let access_state = match &load {
        AccountAccessLoad::Ready(_) => "ready",
        AccountAccessLoad::Unavailable => "unavailable",
        AccountAccessLoad::Malformed => "malformed",
    };
    rsx! {
        div { class: "account-access-plans card card-glass p-4 sm:p-8 lg:p-10 shadow-2xl border-2 border-indigo-200/50",
            "data-section": "account-access-plans",
            "data-access-state": access_state,
            div { class: "flex items-center gap-3 mb-4 sm:mb-8",
                div { class: "p-3 bg-indigo-100 dark:bg-indigo-900/30 rounded-2xl",
                    Icon { name: "shield".to_string(), size: Some(24), class_name: Some("text-indigo-600 dark:text-indigo-400".to_string()) }
                }
                h2 { class: "text-2xl sm:text-3xl font-bold text-foreground", "Access & Plans" }
                a { class: "btn btn-primary ml-auto", href: "/plans", "Manage plans" }
            }
            match load {
                AccountAccessLoad::Ready(access) => rsx! {
                    div { class: "rounded-2xl border border-indigo-300/30 bg-indigo-500/5 p-4 sm:p-6",
                        p { class: "text-xs font-semibold uppercase tracking-widest text-indigo-500", "Current access" }
                        h3 { class: "mt-2 text-2xl font-bold text-foreground", "{access.current_tier}" }
                        p { class: "mt-2 text-sm text-muted-foreground",
                            "{access.groups.len()} active access source(s) · {access.direct_permissions.len()} direct permission(s)"
                        }
                    }
                    div { class: "mt-5 grid gap-4 lg:grid-cols-2", aria_label: "Active plans and access groups",
                        for group in access.groups {
                            article { class: "rounded-2xl border border-border bg-secondary/30 p-4 sm:p-5",
                                div { class: "flex flex-wrap items-start justify-between gap-3",
                                    div {
                                        h3 { class: "font-semibold text-foreground", "{group.name}" }
                                        p { class: "mt-1 text-xs uppercase tracking-wide text-muted-foreground", "{group.source_type}" }
                                    }
                                    if let Some(expires_at) = group.expires_at {
                                        time { class: "rounded-full border border-amber-500/30 bg-amber-500/10 px-2 py-1 text-xs text-amber-500", datetime: expires_at.clone(),
                                            "Expires {expires_at}"
                                        }
                                    } else {
                                        span { class: "rounded-full border border-emerald-500/30 bg-emerald-500/10 px-2 py-1 text-xs text-emerald-500", "No expiry" }
                                    }
                                }
                                if group.permissions.is_empty() {
                                    p { class: "mt-4 text-sm text-muted-foreground", "No explicit permissions in this access source." }
                                } else {
                                    ul { class: "mt-4 flex flex-wrap gap-2", aria_label: "Granted permissions",
                                        for permission in group.permissions {
                                            li { class: "rounded-full border border-border bg-background/50 px-2.5 py-1 font-mono text-xs text-muted-foreground", "{permission}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                AccountAccessLoad::Unavailable => rsx! {
                    AccountAccessProblem { state: "unavailable", detail: "Access and plan details are temporarily unavailable. No access level is being inferred." }
                },
                AccountAccessLoad::Malformed => rsx! {
                    AccountAccessProblem { state: "malformed", detail: "Access details could not be verified safely. No plan or permission data was shown." }
                },
            }
        }
    }
}

#[component]
fn AccountAccessProblem(state: &'static str, detail: &'static str) -> Element {
    rsx! {
        div { class: "p-3 sm:p-6 rounded-2xl border border-red-200 bg-red-50/30 dark:bg-red-900/10", "data-access-problem": state, role: "alert",
            div { class: "flex items-center gap-3",
                Icon { name: "alert-triangle".to_string(), size: Some(20), class_name: Some("text-red-500".to_string()) }
                p { class: "text-sm text-red-600 dark:text-red-400", "{detail}" }
            }
        }
    }
}

// ----- 5. Plan payments ---------------------------------------------------------

#[component]
fn PlanPaymentsSection(load: AccountPlanPaymentsLoad) -> Element {
    let state = match &load {
        AccountPlanPaymentsLoad::Ready(_) => "ready",
        AccountPlanPaymentsLoad::Empty => "empty",
        AccountPlanPaymentsLoad::Unavailable => "unavailable",
        AccountPlanPaymentsLoad::Malformed => "malformed",
    };
    rsx! {
        section {
            class: "account-plan-payments card card-glass p-6 sm:p-8 lg:p-10 shadow-2xl border-2 border-emerald-200/50",
            "data-section": "account-plan-payments",
            "data-plan-payments-state": state,
            div { class: "flex flex-wrap items-center gap-3 mb-6",
                div { class: "p-3 bg-emerald-100 dark:bg-emerald-900/30 rounded-2xl",
                    Icon { name: "receipt".to_string(), size: Some(24), class_name: Some("text-emerald-600 dark:text-emerald-400".to_string()) }
                }
                div {
                    h2 { class: "text-2xl sm:text-3xl font-bold text-foreground", "Plan Payments" }
                    p { class: "mt-1 text-sm text-muted-foreground", "Confirmed checkout and subscription transactions" }
                }
                a { class: "btn btn-outline ml-auto", href: "/plans", "View plans" }
            }
            match load {
                AccountPlanPaymentsLoad::Ready(history) => rsx! {
                    p { class: "mb-4 text-sm text-muted-foreground", "Showing {history.payments.len()} of {history.total} payment(s)" }
                    ol { class: "space-y-3", aria_label: "Plan payment history",
                        for payment in history.payments {
                            PlanPaymentRow { payment }
                        }
                    }
                },
                AccountPlanPaymentsLoad::Empty => rsx! {
                    PlanPaymentsMessage { state: "empty", title: "No plan payments yet", detail: "Completed plan checkouts owned by this wallet will appear here." }
                },
                AccountPlanPaymentsLoad::Unavailable => rsx! {
                    PlanPaymentsMessage { state: "unavailable", title: "Plan payments are temporarily unavailable", detail: "The subscription payment history could not be reached. No empty history was assumed." }
                },
                AccountPlanPaymentsLoad::Malformed => rsx! {
                    PlanPaymentsMessage { state: "malformed", title: "Plan payments could not be displayed safely", detail: "The backend returned an unexpected payment response. No transactions were shown." }
                },
            }
        }
    }
}

#[component]
fn PlanPaymentRow(payment: AccountPlanPaymentProjection) -> Element {
    let plan_name = payment
        .plan_name
        .clone()
        .unwrap_or_else(|| "Plan payment".to_string());
    let payment_time = payment
        .completed_at
        .clone()
        .unwrap_or_else(|| payment.created_at.clone());
    rsx! {
        li {
            article { class: "rounded-2xl border border-border bg-secondary/30 p-4 sm:p-5",
                div { class: "flex flex-wrap items-start justify-between gap-3",
                    div {
                        h3 { class: "font-semibold text-foreground", "{plan_name}" }
                        p { class: "mt-1 font-mono text-xs text-muted-foreground break-all", "{payment.payment_reference}" }
                    }
                    span { class: "rounded-full border border-emerald-500/30 bg-emerald-500/10 px-2.5 py-1 text-xs font-semibold text-emerald-500", "{payment.status}" }
                }
                div { class: "mt-4 flex flex-wrap items-end justify-between gap-3",
                    div {
                        p { class: "text-lg font-bold text-foreground", "{payment.amount} {payment.currency}" }
                        time { class: "mt-1 block text-xs text-muted-foreground", datetime: payment_time.clone(), "{payment_time}" }
                    }
                    if let Some(tx_hash) = payment.tx_hash {
                        span { class: "max-w-full truncate font-mono text-xs text-muted-foreground", title: tx_hash.clone(), "{tx_hash}" }
                    }
                }
            }
        }
    }
}

#[component]
fn PlanPaymentsMessage(state: &'static str, title: &'static str, detail: &'static str) -> Element {
    let role = if matches!(state, "unavailable" | "malformed") {
        "alert"
    } else {
        "status"
    };
    rsx! {
        div { class: "p-8 text-center", "data-plan-payments-message": state, role,
            Icon { name: "receipt".to_string(), size: Some(40), class_name: Some("text-muted-foreground".to_string()) }
            h3 { class: "mt-3 font-semibold text-foreground", "{title}" }
            p { class: "mt-1 text-sm text-muted-foreground", "{detail}" }
            a { class: "btn btn-outline mt-5", href: ACCOUNT_PATH, "Refresh" }
        }
    }
}

// ----- 6. Pay and escrow activity ---------------------------------------------

/// Payment intent and escrow activity section. Mirrors
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
            class: Some("account-payment-history card card-glass p-6 sm:p-8 lg:p-10 shadow-2xl border-2 border-blue-200/50".to_string()),
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

fn account_profile_load(ctx: &PageContext) -> AccountProfileLoad {
    let Some(user) = ctx.user.as_ref() else {
        return AccountProfileLoad::Unavailable;
    };
    match ctx
        .params
        .get(ACCOUNT_PROFILE_STATE_PARAM)
        .map(String::as_str)
    {
        Some(ACCOUNT_DATA_READY) => ctx
            .params
            .get(ACCOUNT_PROFILE_DATA_PARAM)
            .and_then(|raw| serde_json::from_str(raw).ok())
            .and_then(|value| decode_account_profile(value, &user.address))
            .map(AccountProfileLoad::Ready)
            .unwrap_or(AccountProfileLoad::Malformed),
        Some(ACCOUNT_DATA_MALFORMED) => AccountProfileLoad::Malformed,
        Some(ACCOUNT_DATA_UNAVAILABLE) | None => AccountProfileLoad::Unavailable,
        Some(_) => AccountProfileLoad::Malformed,
    }
}

fn account_access_load(ctx: &PageContext) -> AccountAccessLoad {
    match ctx
        .params
        .get(ACCOUNT_ACCESS_STATE_PARAM)
        .map(String::as_str)
    {
        Some(ACCOUNT_DATA_READY) => ctx
            .params
            .get(ACCOUNT_ACCESS_DATA_PARAM)
            .and_then(|raw| serde_json::from_str(raw).ok())
            .and_then(decode_account_access)
            .map(AccountAccessLoad::Ready)
            .unwrap_or(AccountAccessLoad::Malformed),
        Some(ACCOUNT_DATA_MALFORMED) => AccountAccessLoad::Malformed,
        Some(ACCOUNT_DATA_UNAVAILABLE) | None => AccountAccessLoad::Unavailable,
        Some(_) => AccountAccessLoad::Malformed,
    }
}

fn account_plan_payments_load(ctx: &PageContext) -> AccountPlanPaymentsLoad {
    match ctx
        .params
        .get(ACCOUNT_PLAN_PAYMENTS_STATE_PARAM)
        .map(String::as_str)
    {
        Some(ACCOUNT_DATA_READY) | Some(ACCOUNT_DATA_EMPTY) => {
            let expected_empty = ctx
                .params
                .get(ACCOUNT_PLAN_PAYMENTS_STATE_PARAM)
                .is_some_and(|state| state == ACCOUNT_DATA_EMPTY);
            let history = ctx
                .params
                .get(ACCOUNT_PLAN_PAYMENTS_DATA_PARAM)
                .and_then(|raw| serde_json::from_str(raw).ok())
                .and_then(|value| {
                    decode_account_plan_payments(value, ACCOUNT_PLAN_PAYMENTS_MAX_ITEMS)
                });
            match history {
                Some(history) if expected_empty == history.payments.is_empty() => {
                    if expected_empty {
                        AccountPlanPaymentsLoad::Empty
                    } else {
                        AccountPlanPaymentsLoad::Ready(history)
                    }
                }
                _ => AccountPlanPaymentsLoad::Malformed,
            }
        }
        Some(ACCOUNT_DATA_MALFORMED) => AccountPlanPaymentsLoad::Malformed,
        Some(ACCOUNT_DATA_UNAVAILABLE) | None => AccountPlanPaymentsLoad::Unavailable,
        Some(_) => AccountPlanPaymentsLoad::Malformed,
    }
}

// ----- 6. Notification Preferences ---------------------------------------------

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct NotificationQuietHours {
    start: String,
    end: String,
    #[serde(default)]
    enabled: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct NotificationPreferencesPayload {
    channels: BTreeMap<String, bool>,
    quiet_hours: Option<NotificationQuietHours>,
    timezone: Option<String>,
    updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum NotificationPreferencesLoad {
    SignedOut,
    Ready(NotificationPreferencesPayload),
    Unavailable,
    Malformed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NotificationPreferencesFormState {
    None,
    Saved,
    Error,
}

fn notification_preferences_form_state(ctx: &PageContext) -> NotificationPreferencesFormState {
    match ctx
        .params
        .get(ACCOUNT_NOTIFICATION_PREFERENCES_FORM_STATE_PARAM)
        .map(String::as_str)
    {
        Some("saved") => NotificationPreferencesFormState::Saved,
        Some("error") => NotificationPreferencesFormState::Error,
        _ => NotificationPreferencesFormState::None,
    }
}

fn valid_preference_clock(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 5
        || bytes[2] != b':'
        || !bytes[0].is_ascii_digit()
        || !bytes[1].is_ascii_digit()
        || !bytes[3].is_ascii_digit()
        || !bytes[4].is_ascii_digit()
    {
        return false;
    }
    let hour = (bytes[0] - b'0') * 10 + bytes[1] - b'0';
    let minute = (bytes[3] - b'0') * 10 + bytes[4] - b'0';
    hour < 24 && minute < 60
}

fn valid_notification_preferences(payload: &NotificationPreferencesPayload) -> bool {
    payload
        .channels
        .keys()
        .all(|key| matches!(key.as_str(), "email" | "in_app" | "push"))
        && payload.quiet_hours.as_ref().is_none_or(|quiet| {
            valid_preference_clock(&quiet.start) && valid_preference_clock(&quiet.end)
        })
        && payload.timezone.as_deref().is_none_or(|timezone| {
            !timezone.is_empty() && timezone.len() <= 64 && !timezone.chars().any(char::is_control)
        })
}

fn notification_preferences_load(ctx: &PageContext) -> NotificationPreferencesLoad {
    if ctx.user.is_none() {
        return NotificationPreferencesLoad::SignedOut;
    }
    match ctx
        .params
        .get(ACCOUNT_NOTIFICATION_PREFERENCES_STATE_PARAM)
        .map(String::as_str)
    {
        Some(ACCOUNT_NOTIFICATION_PREFERENCES_READY) => {
            let Some(payload) = ctx
                .params
                .get(ACCOUNT_NOTIFICATION_PREFERENCES_DATA_PARAM)
                .and_then(|raw| serde_json::from_str::<NotificationPreferencesPayload>(raw).ok())
            else {
                return NotificationPreferencesLoad::Malformed;
            };
            if valid_notification_preferences(&payload) {
                NotificationPreferencesLoad::Ready(payload)
            } else {
                NotificationPreferencesLoad::Malformed
            }
        }
        Some(ACCOUNT_NOTIFICATION_PREFERENCES_MALFORMED) => NotificationPreferencesLoad::Malformed,
        Some(ACCOUNT_NOTIFICATION_PREFERENCES_UNAVAILABLE) | None => {
            NotificationPreferencesLoad::Unavailable
        }
        Some(_) => NotificationPreferencesLoad::Malformed,
    }
}

/// Owner-scoped preferences are rendered only after the strict SSR read has
/// succeeded. The native form is a bounded server-side adapter; it does not
/// claim browser push permission or provider delivery outcome.
#[component]
fn NotificationPreferencesSection(
    load: NotificationPreferencesLoad,
    form_state: NotificationPreferencesFormState,
) -> Element {
    let signed_in = !matches!(&load, NotificationPreferencesLoad::SignedOut);
    let ready = match &load {
        NotificationPreferencesLoad::Ready(payload) => Some(payload),
        _ => None,
    };
    let state = match &load {
        NotificationPreferencesLoad::SignedOut => "signed-out",
        NotificationPreferencesLoad::Ready(_) => "ready",
        NotificationPreferencesLoad::Unavailable => "unavailable",
        NotificationPreferencesLoad::Malformed => "malformed",
    };
    let alert = matches!(
        &load,
        NotificationPreferencesLoad::Unavailable | NotificationPreferencesLoad::Malformed
    );
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
                        match ready {
                            Some(_) => "Saved notification choices are shown below. Submit the bounded form to update wallet-owned delivery settings.",
                            None if signed_in => "Saved notification choices could not be loaded from the notification service.",
                            None => "Sign in before viewing wallet-owned notification preferences.",
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
                        "data-preferences-state": state,
                        role: if alert { "alert" } else { "status" },
                        div { class: "flex items-start gap-3",
                            Icon {
                                name: if ready.is_some() { "check-circle".to_string() } else if signed_in { "alert-circle".to_string() } else { "lock".to_string() },
                                size: Some(22),
                                class_name: Some("mt-0.5 text-purple-400".to_string()),
                            }
                            div { class: "min-w-0 flex-1",
                                h3 { class: "font-semibold text-foreground",
                                    match ready {
                                        Some(_) => "Notification preferences loaded",
                                        None if signed_in && alert => "Notification preferences are unavailable",
                                        None if signed_in => "Notification preferences are unavailable",
                                        None => "Sign in to view notification preferences",
                                    }
                                }
                                p { class: "mt-1 text-sm leading-6 text-muted-foreground",
                                    match form_state {
                                        NotificationPreferencesFormState::Saved if ready.is_some() => "Preferences saved. The values below were reloaded from the notification service.".to_string(),
                                        NotificationPreferencesFormState::Saved => "Preferences were submitted, but the notification service values could not be reloaded.".to_string(),
                                        NotificationPreferencesFormState::Error => "Preferences could not be saved. Review the bounded fields and try again.".to_string(),
                                        NotificationPreferencesFormState::None => match ready {
                                        Some(payload) => match payload.updated_at {
                                            Some(updated_at) => "Last saved at ".to_string() + &updated_at.to_rfc3339(),
                                            None => "Saved values are shown; the service did not provide a save timestamp.".to_string(),
                                        },
                                        None if signed_in => "No saved values were loaded, and no changes can be made until the service is available.".to_string(),
                                        None => "Preferences are private to the wallet that owns them.".to_string(),
                                        },
                                    }
                                }
                                if let Some(payload) = ready {
                                    if let Some(timezone) = payload.timezone.as_deref() {
                                        p { class: "mt-1 text-xs text-muted-foreground", "Timezone: {timezone}" }
                                    }
                                }
                                div { class: "mt-4 flex flex-wrap gap-3",
                                    if signed_in && ready.is_none() {
                                        a { class: "btn btn-sm btn-outline", href: ACCOUNT_PATH, "Retry" }
                                    } else if !signed_in {
                                        a { class: "btn btn-sm btn-primary", href: ACCOUNT_SIGN_IN_PATH, "Sign in" }
                                    }
                                }
                            }
                        }
                    }
                    if let Some(payload) = ready {
                        NotificationPreferencesForm { payload: payload.clone() }
                    }
                    NotificationPushSection { signed_in }
                }
            }
        }
    }
}

/// Browser push is an optional capability of the notification service. The
/// server-rendered control starts disabled and makes no provider-delivery
/// claim; the authenticated account runtime enables it only after the strict
/// push-status contract and an explicit browser permission gesture succeed.
#[component]
fn NotificationPushSection(signed_in: bool) -> Element {
    let state = if signed_in { "checking" } else { "signed-out" };
    rsx! {
        div {
            class: "mt-5 rounded-2xl border border-border bg-card p-5",
            "data-section": "account-browser-push",
            "data-epsx-notification-push": "true",
            "data-push-state": state,
            h3 { class: "font-semibold text-foreground", "Browser notifications" }
            p {
                class: "mt-1 text-sm leading-6 text-muted-foreground",
                "data-push-status": "true",
                aria_live: "polite",
                if signed_in {
                    "Checking whether browser push is available…"
                } else {
                    "Sign in to check browser notification availability."
                }
            }
            p {
                class: "mt-1 text-xs leading-5 text-muted-foreground",
                "Browser permission and subscription status are shown here; this does not confirm provider delivery."
            }
            div { class: "mt-4 flex flex-wrap gap-3",
                if signed_in {
                    button {
                        r#type: "button",
                        class: "btn btn-sm btn-primary",
                        "data-push-action": "enable",
                        disabled: true,
                        "Enable browser notifications"
                    }
                    button {
                        r#type: "button",
                        class: "btn btn-sm btn-outline",
                        "data-push-action": "disable",
                        hidden: true,
                        "Disable browser notifications"
                    }
                } else {
                    a {
                        class: "btn btn-sm btn-outline",
                        href: ACCOUNT_SIGN_IN_PATH,
                        "Sign in"
                    }
                }
            }
        }
    }
}

#[component]
fn NotificationPreferencesForm(payload: NotificationPreferencesPayload) -> Element {
    let email_enabled = payload.channels.get("email").copied().unwrap_or(false);
    let in_app_enabled = payload.channels.get("in_app").copied().unwrap_or(false);
    let push_enabled = payload.channels.get("push").copied().unwrap_or(false);
    let quiet = payload.quiet_hours.as_ref();
    let quiet_enabled = quiet.and_then(|value| value.enabled).unwrap_or(false);
    let quiet_start = quiet.map(|value| value.start.as_str()).unwrap_or("22:00");
    let quiet_end = quiet.map(|value| value.end.as_str()).unwrap_or("07:00");
    let timezone = payload.timezone.as_deref().unwrap_or("UTC");
    rsx! {
        form {
            class: "mt-4 grid gap-5 rounded-2xl border border-border bg-card p-5",
            method: "post",
            action: "/account/notification-preferences",
            "data-preferences-form": "true",
            aria_label: "Notification preference settings",
            div { class: "grid gap-4 sm:grid-cols-3",
                PreferenceSelect { name: "email", label: "Email Alerts", enabled: email_enabled }
                PreferenceSelect { name: "in_app", label: "In-app Alerts", enabled: in_app_enabled }
                PreferenceSelect { name: "push", label: "Push Alerts", enabled: push_enabled }
            }
            div { class: "grid gap-4 sm:grid-cols-2",
                label { class: "grid gap-2 text-sm font-medium text-foreground",
                    span { "Quiet hours" }
                    select { name: "quiet_enabled", class: "rounded-xl border border-border bg-background px-3 py-2 text-foreground",
                        option { value: "true", selected: quiet_enabled, "Enabled" }
                        option { value: "false", selected: !quiet_enabled, "Disabled" }
                    }
                }
                label { class: "grid gap-2 text-sm font-medium text-foreground",
                    span { "Timezone" }
                    input { name: "timezone", value: timezone, maxlength: "64", autocomplete: "off", class: "rounded-xl border border-border bg-background px-3 py-2 text-foreground" }
                }
                label { class: "grid gap-2 text-sm font-medium text-foreground",
                    span { "Quiet start" }
                    input { type: "time", name: "quiet_start", value: quiet_start, required: true, class: "rounded-xl border border-border bg-background px-3 py-2 text-foreground" }
                }
                label { class: "grid gap-2 text-sm font-medium text-foreground",
                    span { "Quiet end" }
                    input { type: "time", name: "quiet_end", value: quiet_end, required: true, class: "rounded-xl border border-border bg-background px-3 py-2 text-foreground" }
                }
            }
            div { class: "flex items-center justify-between gap-4 pt-1",
                p { class: "text-xs text-muted-foreground", "Changes apply to future owner-bound notification delivery." }
                button { type: "submit", class: "btn btn-sm btn-primary", "Save preferences" }
            }
        }
    }
}

#[component]
fn PreferenceSelect(name: &'static str, label: &'static str, enabled: bool) -> Element {
    rsx! {
        label { class: "grid gap-2 text-sm font-medium text-foreground",
            span { "{label}" }
            select { name, class: "rounded-xl border border-border bg-background px-3 py-2 text-foreground",
                option { value: "true", selected: enabled, "Enabled" }
                option { value: "false", selected: !enabled, "Disabled" }
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
        assert!(
            html.contains("account-payment-history card card-glass p-6 sm:p-8 lg:p-10 shadow-2xl border-2 border-blue-200/50"),
            "transaction history must retain the source card frame. Got: {}",
            html
        );
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
        let html = dioxus_ssr::render_element(rsx! { AccountStatsRow {
            user: Some(user),
            profile_load: AccountProfileLoad::Unavailable,
            credit_balance: CreditBalanceLoad::Unavailable,
        } });

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
        let escaped = dioxus_ssr::render_element(rsx! { AccountStatsRow {
            user: Some(user),
            profile_load: AccountProfileLoad::Unavailable,
            credit_balance: CreditBalanceLoad::Unavailable,
        } });
        assert!(!escaped.contains("<script>alert('owner')</script>"));
        assert!(escaped.contains("&#60;script&#62;alert(&#39;owner&#39;)&#60;/script&#62;"));

        let signed_out = dioxus_ssr::render_element(rsx! { AccountStatsRow {
            user: None,
            profile_load: AccountProfileLoad::Unavailable,
            credit_balance: CreditBalanceLoad::SignedOut,
        } });
        assert!(signed_out.contains("data-account-stat-state=\"signed-out\""));
        assert!(signed_out.contains("href=\"/auth?return_url=%2Faccount\""));
        assert!(!signed_out.contains("Not Connected"));
        assert_account_stats_fail_closed(&signed_out);
    }

    #[test]
    fn verified_account_sources_render_profile_access_credit_and_plan_payment() {
        let mut ctx = authed_ctx();
        ctx.params.insert(
            ACCOUNT_PROFILE_STATE_PARAM.to_string(),
            ACCOUNT_DATA_READY.to_string(),
        );
        ctx.params.insert(
            ACCOUNT_PROFILE_DATA_PARAM.to_string(),
            serde_json::json!({
                "success": true,
                "data": {
                    "wallet_address": "0x1234ABCD",
                    "permissions": ["epsx:analytics:view"],
                    "auth_method": "web3_siwe",
                    "created_at": "2026-01-08T15:43:39Z",
                    "last_login": "2026-08-21T18:56:22Z"
                },
                "meta": {}
            })
            .to_string(),
        );
        ctx.params.insert(
            crate::pages::account_credits::ACCOUNT_CREDIT_BALANCE_STATE_PARAM.to_string(),
            crate::pages::account_credits::ACCOUNT_CREDIT_READY.to_string(),
        );
        ctx.params.insert(
            crate::pages::account_credits::ACCOUNT_CREDIT_BALANCE_DATA_PARAM.to_string(),
            serde_json::json!({
                "wallet_address": "0x1234abcd",
                "balance": "0",
                "pending_balance": "0",
                "available_balance": "0",
                "lifetime_earned": "0",
                "lifetime_spent": "0",
                "last_transaction_at": null
            })
            .to_string(),
        );
        ctx.params.insert(
            ACCOUNT_ACCESS_STATE_PARAM.to_string(),
            ACCOUNT_DATA_READY.to_string(),
        );
        ctx.params.insert(
            ACCOUNT_ACCESS_DATA_PARAM.to_string(),
            serde_json::json!({
                "success": true,
                "data": {
                    "current_tier": "1 Day Package",
                    "groups": [{
                        "id": "plan-1",
                        "name": "1 Day Package",
                        "description": null,
                        "expires_at": "2026-08-22T18:17:17Z",
                        "permissions": ["epsx:analytics:view", "epsx:trading:basic"],
                        "source_type": "plan",
                        "assigned_at": "2026-08-21T18:17:17Z",
                        "assigned_by": null,
                        "days_remaining": 0,
                        "can_renew": false,
                        "renewal_price": null,
                        "billing_cycle": null,
                        "tier_level": 0
                    }],
                    "direct_permissions": []
                },
                "meta": {}
            })
            .to_string(),
        );
        ctx.params.insert(
            ACCOUNT_PLAN_PAYMENTS_STATE_PARAM.to_string(),
            ACCOUNT_DATA_READY.to_string(),
        );
        ctx.params.insert(
            ACCOUNT_PLAN_PAYMENTS_DATA_PARAM.to_string(),
            serde_json::json!({
                "success": true,
                "data": {
                    "payments": [{
                        "id": "payment-1",
                        "amount": 1.0,
                        "currency": "USDT",
                        "status": "confirmed",
                        "tx_hash": "0xabc",
                        "plan_name": "1 Day Package",
                        "permissions_granted": [],
                        "created_at": "2026-08-21T18:17:16Z",
                        "completed_at": "2026-08-21T18:17:17Z",
                        "payment_reference": "PAY-123"
                    }],
                    "pagination": {"page": 1, "per_page": 10, "total": 1, "total_pages": 1}
                }
            })
            .to_string(),
        );

        let html = render_html(&ctx);
        assert!(html.contains(">2026-01-08<"));
        assert!(html.contains(">0 credits<"));
        assert!(html.contains("data-access-state=\"ready\""));
        assert!(html.contains("1 Day Package"));
        assert!(html.contains("epsx:analytics:view"));
        assert!(html.contains("data-plan-payments-state=\"ready\""));
        assert!(html.contains("confirmed"));
        assert!(html.contains("PAY-123"));
    }

    #[test]
    fn account_decoders_reject_wrong_owner_and_inconsistent_payment_pagination() {
        assert!(decode_account_profile(
            serde_json::json!({
                "success": true,
                "data": {
                    "wallet_address": "0xother",
                    "permissions": [],
                    "auth_method": "web3_siwe",
                    "created_at": "2026-01-08T15:43:39Z",
                    "last_login": "2026-08-21T18:56:22Z"
                }
            }),
            "0x1234abcd"
        )
        .is_none());
        assert!(decode_account_plan_payments(
            serde_json::json!({
                "success": true,
                "data": {
                    "payments": [],
                    "pagination": {"page": 1, "per_page": 10, "total": 1, "total_pages": 1}
                }
            }),
            10
        )
        .is_none());
    }

    #[test]
    fn unavailable_preferences_fail_closed_without_mutation_controls() {
        let html = dioxus_ssr::render_element(rsx! { NotificationPreferencesSection {
            load: NotificationPreferencesLoad::Unavailable,
            form_state: NotificationPreferencesFormState::None,
        } });

        assert!(html.contains("data-preferences-state=\"unavailable\""));
        assert!(html.contains("role=\"alert\""));
        assert!(html.contains("No saved values were loaded"));
        assert!(html.contains("href=\"/account\">Retry</a>"));
        assert!(html.contains("data-epsx-notification-push=\"true\""));
        assert!(html.contains("data-push-state=\"checking\""));
        assert!(html.contains("Enable browser notifications"));
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
        let html = dioxus_ssr::render_element(rsx! { NotificationPreferencesSection {
            load: NotificationPreferencesLoad::SignedOut,
            form_state: NotificationPreferencesFormState::None,
        } });

        assert!(html.contains("data-preferences-state=\"signed-out\""));
        assert!(html.contains("role=\"status\""));
        assert!(html.contains("Sign in to view notification preferences"));
        assert!(html.contains("href=\"/auth?return_url=%2Faccount\">Sign in</a>"));
        assert!(html.contains("data-push-state=\"signed-out\""));
        assert!(html.contains("Sign in to check browser notification availability."));
        assert!(!html.contains("preference-read-only-row"));
        assert!(!html.contains("<input"));
    }

    #[test]
    fn loaded_preferences_render_a_bounded_native_form_without_client_mutation() {
        let payload = NotificationPreferencesPayload {
            channels: BTreeMap::from([
                ("email".to_string(), true),
                ("in_app".to_string(), false),
                ("push".to_string(), true),
            ]),
            quiet_hours: Some(NotificationQuietHours {
                start: "22:00".to_string(),
                end: "07:00".to_string(),
                enabled: Some(true),
            }),
            timezone: Some("Asia/Bangkok".to_string()),
            updated_at: Some(
                "2026-07-24T00:00:00Z"
                    .parse::<DateTime<Utc>>()
                    .expect("valid fixture timestamp"),
            ),
        };
        let html = dioxus_ssr::render_element(rsx! {
            NotificationPreferencesSection {
                load: NotificationPreferencesLoad::Ready(payload),
                form_state: NotificationPreferencesFormState::None,
            }
        });

        assert!(html.contains("data-preferences-state=\"ready\""));
        assert!(html.contains("Notification preferences loaded"));
        assert!(html.contains("Email Alerts"));
        assert!(html.contains("Enabled"));
        assert!(html.contains("In-app Alerts"));
        assert!(html.contains("Disabled"));
        assert!(html.contains("name=\"quiet_start\""));
        assert!(html.contains("value=\"22:00\""));
        assert!(html.contains("name=\"quiet_end\""));
        assert!(html.contains("value=\"07:00\""));
        assert!(html.contains("Asia/Bangkok"));
        assert!(html.contains("data-preferences-form=\"true\""));
        assert!(html.contains("action=\"/account/notification-preferences\""));
        assert!(html.contains("name=\"email\""));
        assert!(html.contains("name=\"quiet_start\""));
        assert!(html.contains("Save preferences"));
        assert!(html.contains("data-epsx-notification-push=\"true\""));
        assert!(html.contains("disabled"));
        assert!(html.contains("Browser permission and subscription status are shown here"));
        assert!(!html.contains("onchange"));
    }

    #[test]
    fn preference_projection_rejects_unknown_channels_and_invalid_quiet_hours() {
        let mut ctx = authed_ctx();
        ctx.params.insert(
            ACCOUNT_NOTIFICATION_PREFERENCES_STATE_PARAM.to_string(),
            ACCOUNT_NOTIFICATION_PREFERENCES_READY.to_string(),
        );
        for payload in [
            serde_json::json!({
                "channels": {"webhook": true},
                "quiet_hours": null,
                "timezone": "UTC",
                "updated_at": null
            }),
            serde_json::json!({
                "channels": {"email": true},
                "quiet_hours": {"start": "25:00", "end": "07:00"},
                "timezone": "UTC",
                "updated_at": null
            }),
            serde_json::json!({
                "channels": {"email": true},
                "quiet_hours": null,
                "timezone": "UTC\nforged",
                "updated_at": null
            }),
        ] {
            ctx.params.insert(
                ACCOUNT_NOTIFICATION_PREFERENCES_DATA_PARAM.to_string(),
                payload.to_string(),
            );
            assert_eq!(
                notification_preferences_load(&ctx),
                NotificationPreferencesLoad::Malformed
            );
        }
    }

    #[test]
    fn payment_history_missing_payload_is_unavailable_not_empty() {
        let (_meta, element) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(element);
        assert!(html.contains("Payment intents and escrows are temporarily unavailable"));
        assert!(!html.contains("No payment intents or escrows yet"));
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
        assert!(!html.contains("No payment intents or escrows yet"));
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
