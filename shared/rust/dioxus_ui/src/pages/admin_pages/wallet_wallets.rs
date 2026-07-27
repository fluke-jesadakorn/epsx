//! Authenticated wallet status summary plus truthful detail/disable shells.
//!
//! The wallet-list route may render four backend-authoritative aggregate
//! counts, and the detail route may render a separate redacted wallet read.
//! Rows, balances, plans, permissions, activity, filters, exports, and every
//! mutation remain unavailable. Frontend roles and permissions are never
//! treated as policy authority.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::AuthGate;
use crate::components::admin::page_layout::{PageGradient, PageHeader, PageLayout, PageMaxWidth};
use crate::primitives::Icon;

use super::super::{PageContext, PageMeta};

const WALLETS_PATH: &str = "/wallet-management/wallets";
const MAX_WALLET_LABEL_CHARS: usize = 100;
const MAX_WALLET_ROLE_CHARS: usize = 64;
const MAX_CHAIN_ID_CHARS: usize = 10;

pub const ADMIN_WALLET_STATS_DATA_PARAM: &str = "data_admin_wallet_stats";
pub const ADMIN_WALLET_STATS_STATE_PARAM: &str = "data_admin_wallet_stats_state";
pub const ADMIN_WALLET_LIST_DATA_PARAM: &str = "data_admin_wallet_list";
pub const ADMIN_WALLET_LIST_STATE_PARAM: &str = "data_admin_wallet_list_state";
pub const ADMIN_WALLET_DETAIL_DATA_PARAM: &str = "data_admin_wallet_detail";
pub const ADMIN_WALLET_DETAIL_STATE_PARAM: &str = "data_admin_wallet_detail_state";

pub const ADMIN_WALLET_STATS_READY: &str = "ready";
pub const ADMIN_WALLET_STATS_FORBIDDEN: &str = "forbidden";
pub const ADMIN_WALLET_STATS_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_WALLET_STATS_MALFORMED: &str = "malformed";
pub const ADMIN_WALLET_LIST_READY: &str = "ready";
pub const ADMIN_WALLET_LIST_EMPTY: &str = "empty";
pub const ADMIN_WALLET_LIST_FORBIDDEN: &str = "forbidden";
pub const ADMIN_WALLET_LIST_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_WALLET_LIST_MALFORMED: &str = "malformed";
pub const ADMIN_WALLET_DETAIL_READY: &str = "ready";
pub const ADMIN_WALLET_DETAIL_FORBIDDEN: &str = "forbidden";
pub const ADMIN_WALLET_DETAIL_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_WALLET_DETAIL_MALFORMED: &str = "malformed";
pub const ADMIN_WALLET_DISABLE_STATE_PARAM: &str = "data_admin_wallet_disable_state";
pub const ADMIN_WALLET_DISABLE_FORM: &str = "form";
pub const ADMIN_WALLET_DISABLE_SUCCESS: &str = "success";
pub const ADMIN_WALLET_DISABLE_CONFLICT: &str = "conflict";
pub const ADMIN_WALLET_DISABLE_FORBIDDEN: &str = "forbidden";
pub const ADMIN_WALLET_DISABLE_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_WALLET_DISABLE_MALFORMED: &str = "malformed";

/// Deliberately excludes identities, addresses, balances, tier distribution,
/// activity-window claims, growth calculations, and every row-level field.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminWalletStatsSummary {
    pub total_users: i64,
    pub active_users: i64,
    pub inactive_users: i64,
    pub new_users_30_days: i64,
}

/// Decode the exact aggregate projection and reject impossible count sets.
/// Zero is valid for every count.
pub fn decode_admin_wallet_stats_projection(
    value: serde_json::Value,
) -> Option<AdminWalletStatsSummary> {
    let projection: AdminWalletStatsSummary = serde_json::from_value(value).ok()?;
    if projection.total_users < 0
        || projection.active_users < 0
        || projection.inactive_users < 0
        || projection.new_users_30_days < 0
        || projection
            .active_users
            .checked_add(projection.inactive_users)
            != Some(projection.total_users)
        || projection.new_users_30_days > projection.total_users
    {
        return None;
    }
    Some(projection)
}

/// A bounded row projection for the wallet inventory.  Metadata, timestamps,
/// balances, entitlements, and audit identity stay outside PageContext.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminWalletListItemProjection {
    pub address: String,
    pub chain_id: String,
    pub label: Option<String>,
    pub role: Option<String>,
    pub status: String,
    pub version: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminWalletListProjection {
    pub items: Vec<AdminWalletListItemProjection>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

pub fn decode_admin_wallet_list_projection(
    value: serde_json::Value,
) -> Option<AdminWalletListProjection> {
    let projection: AdminWalletListProjection = serde_json::from_value(value).ok()?;
    if projection.total < 0
        || !(1..=100).contains(&projection.limit)
        || !(0..=10_000_000).contains(&projection.offset)
        || projection.items.len() > projection.limit as usize
    {
        return None;
    }
    if projection.items.iter().any(|item| {
        canonical_wallet_address(&item.address).is_none()
            || !valid_chain_id(&item.chain_id)
            || item
                .label
                .as_deref()
                .is_some_and(|value| !valid_optional_text(value, MAX_WALLET_LABEL_CHARS))
            || item
                .role
                .as_deref()
                .is_some_and(|value| !valid_optional_text(value, MAX_WALLET_ROLE_CHARS))
            || !matches!(item.status.as_str(), "active" | "disabled")
            || item.version < 0
    }) {
        return None;
    }
    Some(projection)
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum WalletStatsLoad {
    Ready(AdminWalletStatsSummary),
    Forbidden,
    Unavailable,
    Malformed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum WalletListLoad {
    Ready(AdminWalletListProjection),
    Empty,
    Forbidden,
    Unavailable,
    Malformed,
}

fn wallet_list_load(ctx: &PageContext) -> Option<WalletListLoad> {
    let state = ctx
        .params
        .get(ADMIN_WALLET_LIST_STATE_PARAM)
        .map(String::as_str)?;
    Some(match state {
        ADMIN_WALLET_LIST_READY => ctx
            .params
            .get(ADMIN_WALLET_LIST_DATA_PARAM)
            .and_then(|raw| serde_json::from_str(raw).ok())
            .and_then(decode_admin_wallet_list_projection)
            .map(|projection| {
                if projection.items.is_empty() && projection.total == 0 {
                    WalletListLoad::Empty
                } else {
                    WalletListLoad::Ready(projection)
                }
            })
            .unwrap_or(WalletListLoad::Malformed),
        ADMIN_WALLET_LIST_EMPTY => WalletListLoad::Empty,
        ADMIN_WALLET_LIST_FORBIDDEN => WalletListLoad::Forbidden,
        ADMIN_WALLET_LIST_UNAVAILABLE => WalletListLoad::Unavailable,
        ADMIN_WALLET_LIST_MALFORMED => WalletListLoad::Malformed,
        _ => WalletListLoad::Malformed,
    })
}

fn wallet_stats_load(ctx: &PageContext) -> WalletStatsLoad {
    match ctx
        .params
        .get(ADMIN_WALLET_STATS_STATE_PARAM)
        .map(String::as_str)
    {
        Some(ADMIN_WALLET_STATS_READY) => {
            let Some(raw) = ctx.params.get(ADMIN_WALLET_STATS_DATA_PARAM) else {
                return WalletStatsLoad::Malformed;
            };
            serde_json::from_str(raw)
                .ok()
                .and_then(decode_admin_wallet_stats_projection)
                .map(WalletStatsLoad::Ready)
                .unwrap_or(WalletStatsLoad::Malformed)
        }
        Some(ADMIN_WALLET_STATS_FORBIDDEN) => WalletStatsLoad::Forbidden,
        Some(ADMIN_WALLET_STATS_MALFORMED) => WalletStatsLoad::Malformed,
        Some(ADMIN_WALLET_STATS_UNAVAILABLE) | None => WalletStatsLoad::Unavailable,
        Some(_) => WalletStatsLoad::Malformed,
    }
}

/// Redacted fields from the backend AdminWallet DTO. Metadata, creation time,
/// and operation/audit evidence never enter PageContext or page HTML.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminWalletDetailProjection {
    pub address: String,
    pub chain_id: String,
    pub label: Option<String>,
    pub role: Option<String>,
    pub status: String,
    pub version: i64,
}

pub fn decode_admin_wallet_detail_projection(
    value: serde_json::Value,
) -> Option<AdminWalletDetailProjection> {
    let projection: AdminWalletDetailProjection = serde_json::from_value(value).ok()?;
    if canonical_wallet_address(&projection.address).is_none()
        || !valid_chain_id(&projection.chain_id)
        || projection
            .label
            .as_deref()
            .is_some_and(|value| !valid_optional_text(value, MAX_WALLET_LABEL_CHARS))
        || projection
            .role
            .as_deref()
            .is_some_and(|value| !valid_optional_text(value, MAX_WALLET_ROLE_CHARS))
        || !matches!(projection.status.as_str(), "active" | "disabled")
        || projection.version < 0
    {
        return None;
    }
    Some(projection)
}

fn valid_optional_text(value: &str, max_chars: usize) -> bool {
    value.chars().count() <= max_chars
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

fn valid_chain_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_CHAIN_ID_CHARS
        && value.bytes().all(|byte| byte.is_ascii_digit())
}

fn canonical_wallet_address(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    if bytes.len() != 42
        || bytes[0] != b'0'
        || !matches!(bytes[1], b'x' | b'X')
        || !bytes[2..].iter().all(u8::is_ascii_hexdigit)
    {
        return None;
    }
    Some(format!("0x{}", value[2..].to_ascii_lowercase()))
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum WalletDetailLoad {
    Ready(AdminWalletDetailProjection),
    Forbidden,
    Unavailable,
    Malformed,
}

fn wallet_detail_load(ctx: &PageContext) -> WalletDetailLoad {
    let Some(route_address) = ctx
        .params
        .get("address")
        .and_then(|value| canonical_wallet_address(value))
    else {
        return WalletDetailLoad::Malformed;
    };

    match ctx
        .params
        .get(ADMIN_WALLET_DETAIL_STATE_PARAM)
        .map(String::as_str)
    {
        Some(ADMIN_WALLET_DETAIL_READY) => {
            let Some(raw) = ctx.params.get(ADMIN_WALLET_DETAIL_DATA_PARAM) else {
                return WalletDetailLoad::Malformed;
            };
            let Some(projection) = serde_json::from_str(raw)
                .ok()
                .and_then(decode_admin_wallet_detail_projection)
            else {
                return WalletDetailLoad::Malformed;
            };
            if canonical_wallet_address(&projection.address) == Some(route_address) {
                WalletDetailLoad::Ready(projection)
            } else {
                WalletDetailLoad::Malformed
            }
        }
        Some(ADMIN_WALLET_DETAIL_FORBIDDEN) => WalletDetailLoad::Forbidden,
        Some(ADMIN_WALLET_DETAIL_MALFORMED) => WalletDetailLoad::Malformed,
        Some(ADMIN_WALLET_DETAIL_UNAVAILABLE) | None => WalletDetailLoad::Unavailable,
        Some(_) => WalletDetailLoad::Malformed,
    }
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Wallets");
    (
        meta,
        rsx! {
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("the private admin wallet workspace".to_string()),
                return_url: Some(WALLETS_PATH.to_string()),
                RenderWalletList { ctx: ctx.clone() }
            }
        },
    )
}

pub fn render_detail(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Wallet detail");
    (
        meta,
        rsx! {
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("the private admin wallet workspace".to_string()),
                return_url: Some(WALLETS_PATH.to_string()),
                RenderWalletDetail { ctx: ctx.clone() }
            }
        },
    )
}

/// The legacy confirmation route is backed by the wallet detail version and
/// submits only a backend-authorized, idempotent status mutation.
pub fn render_disable(ctx: &PageContext) -> (PageMeta, Element) {
    let reference = ctx
        .params
        .get("address")
        .and_then(|value| canonical_wallet_address(value));
    let meta = PageMeta::admin("Disable wallet");
    (
        meta,
        rsx! {
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("the private admin wallet workspace".to_string()),
                return_url: Some(WALLETS_PATH.to_string()),
                RenderWalletDisable { ctx: ctx.clone(), reference }
            }
        },
    )
}

#[component]
fn RenderWalletDisable(ctx: PageContext, reference: Option<String>) -> Element {
    let state = ctx
        .params
        .get(ADMIN_WALLET_DISABLE_STATE_PARAM)
        .map(String::as_str);
    if let Some(state) = state.filter(|state| {
        matches!(
            *state,
            ADMIN_WALLET_DISABLE_SUCCESS
                | ADMIN_WALLET_DISABLE_CONFLICT
                | ADMIN_WALLET_DISABLE_FORBIDDEN
                | ADMIN_WALLET_DISABLE_UNAVAILABLE
                | ADMIN_WALLET_DISABLE_MALFORMED
        )
    }) {
        return match state {
            ADMIN_WALLET_DISABLE_SUCCESS => {
                rsx! { WalletDisableNotice { state: ADMIN_WALLET_DISABLE_SUCCESS, title: "Wallet disabled".to_string(), detail: "The wallet service committed the status change and returned an operation receipt.".to_string() } }
            }
            ADMIN_WALLET_DISABLE_CONFLICT => {
                rsx! { WalletDisableNotice { state: ADMIN_WALLET_DISABLE_CONFLICT, title: "Wallet status changed elsewhere".to_string(), detail: "The submitted version was stale. Reload the backend-authoritative wallet detail before retrying.".to_string() } }
            }
            ADMIN_WALLET_DISABLE_FORBIDDEN => {
                rsx! { WalletDisableNotice { state: ADMIN_WALLET_DISABLE_FORBIDDEN, title: "Wallet change access was denied".to_string(), detail: "The wallet service did not authorize this session to change the requested resource.".to_string() } }
            }
            ADMIN_WALLET_DISABLE_UNAVAILABLE => {
                rsx! { WalletDisableNotice { state: ADMIN_WALLET_DISABLE_UNAVAILABLE, title: "Wallet change is unavailable".to_string(), detail: "The wallet service did not provide a committed mutation result. No success is inferred.".to_string() } }
            }
            _ => {
                rsx! { WalletDisableNotice { state: ADMIN_WALLET_DISABLE_MALFORMED, title: "Wallet change could not be verified".to_string(), detail: "The mutation response or route state did not match the strict contract.".to_string() } }
            }
        };
    }

    let Some(reference) = reference else {
        return rsx! { WalletDisableNotice { state: ADMIN_WALLET_DISABLE_MALFORMED, title: "Wallet address could not be verified".to_string(), detail: "The route must contain one canonical wallet address.".to_string() } };
    };
    match wallet_detail_load(&ctx) {
        WalletDetailLoad::Ready(projection) if projection.status == "active" => {
            let action = format!("/wallet-management/wallets/{reference}/disable");
            let idempotency_key = Uuid::new_v4().to_string();
            rsx! {
                section {
                    class: "container page-content max-w-3xl py-10",
                    "data-admin-wallet-disable-state": ADMIN_WALLET_DISABLE_FORM,
                    h1 { class: "text-3xl font-black tracking-tight text-foreground", "Disable wallet" }
                    p { class: "mt-3 break-all font-mono text-sm text-muted-foreground", "{projection.address}" }
                    p { class: "mt-4 text-sm leading-6 text-muted-foreground", "The wallet service will recheck ownership, permission, current status, and version before committing this audited change." }
                    form { class: "mt-6 space-y-4 rounded-2xl border border-border/30 bg-card p-6 shadow-xl", method: "post", action,
                        input { type: "hidden", name: "expected_version", value: projection.version.to_string() }
                        input { type: "hidden", name: "idempotency_key", value: idempotency_key }
                        label { class: "block text-sm font-medium text-foreground", r#for: "wallet-disable-reason", "Reason" }
                        textarea { id: "wallet-disable-reason", name: "reason", required: true, maxlength: "500", rows: "4", class: "mt-2 w-full rounded-xl border border-border/40 bg-background p-3 text-sm", placeholder: "Describe the administrative reason" }
                        div { class: "flex flex-wrap gap-3",
                            button { class: "btn btn-primary", type: "submit", "Disable wallet" }
                            a { class: "btn btn-outline", href: format!("/wallet-management/{}", projection.address), "Cancel" }
                        }
                    }
                }
            }
        }
        WalletDetailLoad::Ready(_) => {
            rsx! { WalletDisableNotice { state: ADMIN_WALLET_DISABLE_MALFORMED, title: "Wallet is already disabled".to_string(), detail: "The backend-authoritative wallet state does not require this operation.".to_string() } }
        }
        WalletDetailLoad::Forbidden => {
            rsx! { WalletDisableNotice { state: ADMIN_WALLET_DISABLE_FORBIDDEN, title: "Wallet detail access was denied".to_string(), detail: "The current wallet status could not be authorized.".to_string() } }
        }
        WalletDetailLoad::Unavailable => {
            rsx! { WalletDisableNotice { state: ADMIN_WALLET_DISABLE_UNAVAILABLE, title: "Wallet status is unavailable".to_string(), detail: "The wallet service did not provide the current version, so no mutation form is shown.".to_string() } }
        }
        WalletDetailLoad::Malformed => {
            rsx! { WalletDisableNotice { state: ADMIN_WALLET_DISABLE_MALFORMED, title: "Wallet status could not be verified".to_string(), detail: "The route or backend detail response did not match the strict wallet contract.".to_string() } }
        }
    }
}

#[component]
fn WalletDisableNotice(state: &'static str, title: String, detail: String) -> Element {
    rsx! {
        section { class: "container page-content max-w-3xl py-10", role: "status", "data-admin-wallet-disable-state": state,
            h1 { class: "text-3xl font-black tracking-tight text-foreground", "{title}" }
            p { class: "mt-3 text-sm leading-6 text-muted-foreground", "{detail}" }
            nav { class: "mt-6 flex flex-wrap gap-3", aria_label: "Wallet disable recovery",
                a { class: "btn btn-primary", href: "/wallet-management/wallets", "Wallet inventory" }
                a { class: "btn btn-outline", href: "/", "Admin home" }
            }
        }
    }
}

#[component]
fn RenderWalletDetail(ctx: PageContext) -> Element {
    match wallet_detail_load(&ctx) {
        WalletDetailLoad::Ready(projection) => rsx! { WalletDetailReady { projection } },
        WalletDetailLoad::Forbidden => rsx! {
            WalletDetailProblem {
                state: ADMIN_WALLET_DETAIL_FORBIDDEN,
                title: "Wallet detail access was denied".to_string(),
                detail: "The backend did not authorize this session to read the requested wallet.".to_string(),
            }
        },
        WalletDetailLoad::Unavailable => rsx! {
            WalletDetailProblem {
                state: ADMIN_WALLET_DETAIL_UNAVAILABLE,
                title: "Wallet detail is unavailable".to_string(),
                detail: "The wallet backend could not provide an authoritative wallet response. No wallet fields are being shown.".to_string(),
            }
        },
        WalletDetailLoad::Malformed => rsx! {
            WalletDetailProblem {
                state: ADMIN_WALLET_DETAIL_MALFORMED,
                title: "Wallet detail could not be verified".to_string(),
                detail: "The route address or backend response did not match the strict wallet contract. No wallet fields are being shown.".to_string(),
            }
        },
    }
}

#[component]
fn WalletDetailReady(projection: AdminWalletDetailProjection) -> Element {
    let label = projection
        .label
        .unwrap_or_else(|| "Not reported".to_string());
    let role = projection
        .role
        .unwrap_or_else(|| "Not reported".to_string());
    let status = projection.status.clone();
    let status_label = if status == "active" {
        "Active"
    } else {
        "Disabled"
    };

    rsx! {
        PageLayout {
            max_width: Some(PageMaxWidth::FourXl),
            PageHeader {
                title: "Wallet detail".to_string(),
                subtitle: Some("Backend-authoritative read-only wallet projection".to_string()),
                icon: Some("wallet".to_string()),
                gradient: Some(PageGradient::Primary),
                centered: Some(false),
                extra_actions: None,
                class_name: None,
            }
            section {
                class: "rounded-2xl border border-border/30 bg-card p-6 shadow-xl sm:p-8",
                aria_labelledby: "admin-wallet-detail-title",
                "data-admin-wallet-detail-state": ADMIN_WALLET_DETAIL_READY,
                h2 { id: "admin-wallet-detail-title", class: "text-2xl font-bold text-foreground", "{label}" }
                p { class: "mt-2 break-all font-mono text-sm text-muted-foreground", "{projection.address}" }
                dl { class: "mt-6 grid gap-4 sm:grid-cols-2",
                    WalletDetailField { label: "Status", value: status_label.to_string() }
                    WalletDetailField { label: "Chain", value: projection.chain_id }
                    WalletDetailField { label: "Role", value: role }
                    WalletDetailField { label: "Read version", value: projection.version.to_string() }
                }
                p { class: "mt-6 border-t border-border/30 pt-4 text-xs leading-5 text-muted-foreground",
                    "Metadata, balances, entitlements, permissions, audit identity, and wallet operations are not part of this redacted read projection."
                }
                nav { class: "mt-6 flex flex-wrap gap-3", aria_label: "Wallet detail recovery",
                    a { class: "btn btn-outline", href: WALLETS_PATH, "Wallet list" }
                    a { class: "btn btn-ghost", href: "/", "Admin home" }
                }
            }
        }
    }
}

#[component]
fn WalletDetailField(label: &'static str, value: String) -> Element {
    rsx! {
        div { class: "rounded-xl border border-border/20 bg-background/40 p-4",
            dt { class: "text-xs font-medium uppercase tracking-wide text-muted-foreground", "{label}" }
            dd { class: "mt-1 break-words text-sm font-semibold text-foreground", "{value}" }
        }
    }
}

#[component]
fn WalletDetailProblem(state: &'static str, title: String, detail: String) -> Element {
    rsx! {
        PageLayout {
            max_width: Some(PageMaxWidth::FourXl),
            PageHeader {
                title: "Wallet detail".to_string(),
                subtitle: Some("Read-only backend projection".to_string()),
                icon: Some("wallet".to_string()),
                gradient: Some(PageGradient::Primary),
                centered: Some(false),
                extra_actions: None,
                class_name: None,
            }
            section {
                class: "rounded-2xl border border-amber-500/25 bg-amber-500/10 p-6 sm:p-8",
                role: if state == ADMIN_WALLET_DETAIL_FORBIDDEN { "alert" } else { "status" },
                aria_labelledby: "admin-wallet-detail-problem-title",
                "data-admin-wallet-detail-state": state,
                h2 { id: "admin-wallet-detail-problem-title", class: "text-xl font-bold text-foreground", "{title}" }
                p { class: "mt-2 max-w-3xl text-sm leading-6 text-muted-foreground", "{detail}" }
                nav { class: "mt-5 flex flex-wrap gap-3", aria_label: "Wallet detail recovery",
                    a { class: "btn btn-sm btn-outline", href: WALLETS_PATH, "Retry wallet read" }
                    a { class: "btn btn-sm btn-ghost", href: "/", "Admin home" }
                }
            }
        }
    }
}

#[component]
fn RenderWalletList(ctx: PageContext) -> Element {
    let inventory = wallet_list_load(&ctx);
    let has_inventory = inventory.is_some();
    let load = wallet_stats_load(&ctx);

    rsx! {
        div {
            "data-admin-wallets-surface": "list",
            PageLayout {
                max_width: Some(PageMaxWidth::SevenXl),
                PageHeader {
                    title: "Wallets".to_string(),
                    subtitle: Some("Review backend-authoritative user status totals".to_string()),
                    icon: Some("wallet".to_string()),
                    gradient: Some(PageGradient::Primary),
                    centered: Some(false),
                    extra_actions: None,
                    class_name: None,
                }
                match inventory {
                    Some(WalletListLoad::Ready(projection)) => rsx! { WalletListReady { projection } },
                    Some(WalletListLoad::Empty) => rsx! { WalletListEmpty {} },
                    Some(WalletListLoad::Forbidden) => rsx! {
                        WalletListProblem {
                            state: ADMIN_WALLET_LIST_FORBIDDEN,
                            title: "Wallet inventory access was denied".to_string(),
                            detail: "The backend did not authorize this session to read wallet rows.".to_string(),
                        }
                    },
                    Some(WalletListLoad::Unavailable) => rsx! {
                        WalletListProblem {
                            state: ADMIN_WALLET_LIST_UNAVAILABLE,
                            title: "Wallet inventory is unavailable".to_string(),
                            detail: "The wallet backend could not provide an authoritative wallet list. No rows are being shown.".to_string(),
                        }
                    },
                    Some(WalletListLoad::Malformed) => rsx! {
                        WalletListProblem {
                            state: ADMIN_WALLET_LIST_MALFORMED,
                            title: "Wallet inventory could not be verified".to_string(),
                            detail: "The backend response did not match the strict bounded wallet-list contract. No rows are being shown.".to_string(),
                        }
                    },
                    None => match load {
                        WalletStatsLoad::Ready(projection) => rsx! { WalletStatsReady { projection } },
                        WalletStatsLoad::Forbidden => rsx! {
                            WalletStatsProblem {
                                state: ADMIN_WALLET_STATS_FORBIDDEN,
                                title: "Wallet summary access was denied".to_string(),
                                detail: "The backend did not authorize this session to read wallet status totals.".to_string(),
                            }
                        },
                        WalletStatsLoad::Unavailable => rsx! {
                            WalletStatsProblem {
                                state: ADMIN_WALLET_STATS_UNAVAILABLE,
                                title: "Wallet summary is unavailable".to_string(),
                                detail: "The wallet backend could not provide an authoritative status summary. No totals are being shown.".to_string(),
                            }
                        },
                        WalletStatsLoad::Malformed => rsx! {
                            WalletStatsProblem {
                                state: ADMIN_WALLET_STATS_MALFORMED,
                                title: "Wallet summary could not be verified".to_string(),
                                detail: "The backend response did not match the strict aggregate contract. No totals are being shown.".to_string(),
                            }
                        },
                    },
                }
                if !has_inventory {
                    WalletInventoryUnavailableNotice {}
                }
            }
        }
    }
}

#[component]
fn WalletListReady(projection: AdminWalletListProjection) -> Element {
    rsx! {
        section {
            class: "overflow-hidden rounded-2xl border border-border/30 bg-card shadow-xl",
            "data-admin-wallet-list-state": ADMIN_WALLET_LIST_READY,
            h2 { class: "p-5 text-lg font-semibold text-foreground", "Wallet inventory" }
            p { class: "px-5 pb-4 text-sm text-muted-foreground", "{projection.total} backend-authoritative records in this bounded response." }
            ul { class: "divide-y divide-border/30 border-t border-border/30", aria_label: "Wallet inventory",
                for wallet in projection.items { WalletListRow { wallet } }
            }
        }
    }
}

#[component]
fn WalletListRow(wallet: AdminWalletListItemProjection) -> Element {
    let detail_href = format!(
        "/wallet-management/{}",
        encode_path_segment(&wallet.address)
    );
    let status = if wallet.status == "active" {
        "Active"
    } else {
        "Disabled"
    };
    rsx! {
        li { class: "grid gap-3 p-5 sm:grid-cols-[minmax(0,1fr)_auto_auto] sm:items-center",
            div {
                p { class: "break-all font-mono text-sm text-foreground", "{wallet.address}" }
                p { class: "mt-1 text-xs text-muted-foreground", "Chain {wallet.chain_id} · {status}" }
            }
            p { class: "text-sm text-muted-foreground", "Version {wallet.version}" }
            a { class: "btn btn-sm btn-outline", href: detail_href, "Read detail" }
        }
    }
}

#[component]
fn WalletListEmpty() -> Element {
    rsx! {
        section { class: "rounded-2xl border border-border/30 bg-card p-8 text-center", role: "status", "data-admin-wallet-list-state": ADMIN_WALLET_LIST_EMPTY,
            h2 { class: "text-xl font-semibold text-foreground", "No wallets returned" }
            p { class: "mt-2 text-sm text-muted-foreground", "The backend returned an authoritative empty wallet inventory." }
        }
    }
}

#[component]
fn WalletListProblem(state: &'static str, title: String, detail: String) -> Element {
    rsx! {
        section { class: "rounded-2xl border border-amber-500/25 bg-amber-500/10 p-6", role: "status", "data-admin-wallet-list-state": state,
            h2 { class: "text-xl font-bold text-foreground", "{title}" }
            p { class: "mt-2 text-sm leading-6 text-muted-foreground", "{detail}" }
        }
    }
}

#[component]
fn WalletStatsReady(projection: AdminWalletStatsSummary) -> Element {
    let total = format_count(projection.total_users);
    let active = format_count(projection.active_users);
    let inactive = format_count(projection.inactive_users);
    let new_30_days = format_count(projection.new_users_30_days);

    rsx! {
        section {
            class: "overflow-hidden rounded-2xl border border-border/30 bg-card shadow-xl",
            aria_labelledby: "admin-wallet-stats-title",
            "data-admin-wallets-state": ADMIN_WALLET_STATS_READY,
            div {
                class: "h-1 bg-gradient-to-r from-[#1fc7d4] via-[#7645d9] to-[#31d0aa]",
                aria_hidden: "true",
            }
            div { class: "p-5 sm:p-6",
                h2 {
                    id: "admin-wallet-stats-title",
                    class: "text-lg font-semibold text-foreground",
                    "User status summary"
                }
                p { class: "mt-1 max-w-3xl text-sm leading-6 text-muted-foreground",
                    "These counts describe backend user records. Active and inactive are stored account statuses, not recent activity measurements."
                }
            }
            dl {
                class: "grid grid-cols-1 gap-px border-t border-border/30 bg-border/30 sm:grid-cols-2 xl:grid-cols-4",
                WalletCount {
                    label: "Total users".to_string(),
                    value: total,
                    detail: "All registered user records".to_string(),
                }
                WalletCount {
                    label: "Users marked active".to_string(),
                    value: active,
                    detail: "Records with active status".to_string(),
                }
                WalletCount {
                    label: "Users marked inactive".to_string(),
                    value: inactive,
                    detail: "Records with inactive status".to_string(),
                }
                WalletCount {
                    label: "New users, past 30 days".to_string(),
                    value: new_30_days,
                    detail: "Records created in the last 30 days".to_string(),
                }
            }
        }
    }
}

#[component]
fn WalletCount(label: String, value: String, detail: String) -> Element {
    rsx! {
        div { class: "min-w-0 bg-card p-5 sm:p-6",
            dt { class: "text-sm font-medium text-muted-foreground", "{label}" }
            dd { class: "mt-2 break-words text-3xl font-black tracking-tight text-foreground", "{value}" }
            dd { class: "mt-2 text-xs leading-5 text-muted-foreground", "{detail}" }
        }
    }
}

#[component]
fn WalletStatsProblem(state: &'static str, title: String, detail: String) -> Element {
    rsx! {
        section {
            class: "rounded-2xl border border-amber-500/25 bg-amber-500/10 p-6 sm:p-8",
            role: "status",
            aria_labelledby: "admin-wallet-stats-problem-title",
            "data-admin-wallets-state": state,
            div { class: "flex flex-col gap-5 sm:flex-row sm:items-start",
                div {
                    class: "flex h-12 w-12 shrink-0 items-center justify-center rounded-xl border border-amber-500/25 bg-background/60 text-amber-700 dark:text-amber-300",
                    aria_hidden: "true",
                    Icon { name: "shield-alert".to_string(), size: Some(24) }
                }
                div { class: "min-w-0",
                    h2 {
                        id: "admin-wallet-stats-problem-title",
                        class: "text-xl font-bold text-foreground",
                        "{title}"
                    }
                    p { class: "mt-2 max-w-3xl text-sm leading-6 text-muted-foreground", "{detail}" }
                    nav { class: "mt-5 flex flex-wrap gap-3", aria_label: "Wallet summary recovery",
                        a { class: "btn btn-sm btn-outline", href: WALLETS_PATH,
                            Icon { name: "refresh-cw".to_string(), size: Some(15) }
                            " Retry summary"
                        }
                        a { class: "btn btn-sm btn-ghost", href: "/",
                            Icon { name: "home".to_string(), size: Some(15) }
                            " Admin home"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn WalletInventoryUnavailableNotice() -> Element {
    rsx! {
        section {
            class: "mt-6 rounded-2xl border border-border/40 bg-muted/20 p-5 sm:p-6",
            aria_labelledby: "admin-wallet-inventory-unavailable-title",
            "data-admin-wallet-inventory-state": "unavailable",
            div { class: "flex flex-col gap-4 sm:flex-row sm:items-start",
                div {
                    class: "flex h-10 w-10 shrink-0 items-center justify-center rounded-xl border border-border/40 bg-background/60 text-muted-foreground",
                    aria_hidden: "true",
                    Icon { name: "lock".to_string(), size: Some(19) }
                }
                div {
                    h2 {
                        id: "admin-wallet-inventory-unavailable-title",
                        class: "font-semibold text-foreground",
                        "Wallet inventory remains unavailable"
                    }
                    p { class: "mt-2 max-w-4xl text-sm leading-6 text-muted-foreground",
                        "Wallet rows, filters, details, balances, plans, permissions, activity, and controls are not connected to an authorized typed contract and are not shown."
                    }
                }
            }
        }
    }
}

fn format_count(value: i64) -> String {
    let digits = value.to_string();
    let mut formatted = String::with_capacity(digits.len() + digits.len() / 3);
    let first_group = digits.len() % 3;
    for (index, byte) in digits.bytes().enumerate() {
        if index > 0 && index % 3 == first_group {
            formatted.push(',');
        }
        formatted.push(char::from(byte));
    }
    formatted
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::auth::user::{AuthMethod, User};

    const TEST_ADDRESS: &str = "0x1111111111111111111111111111111111111111";

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

    fn stats_json(total: i64, active: i64, inactive: i64, new_30_days: i64) -> String {
        serde_json::json!({
            "total_users": total,
            "active_users": active,
            "inactive_users": inactive,
            "new_users_30_days": new_30_days,
        })
        .to_string()
    }

    fn ctx_with_stats(state: &str, data: Option<String>) -> PageContext {
        let mut ctx = authenticated_ctx();
        ctx.params.insert(
            ADMIN_WALLET_STATS_STATE_PARAM.to_string(),
            state.to_string(),
        );
        if let Some(data) = data {
            ctx.params
                .insert(ADMIN_WALLET_STATS_DATA_PARAM.to_string(), data);
        }
        ctx
    }

    fn wallet_detail_json(address: &str) -> String {
        serde_json::json!({
            "address": address,
            "chain_id": "56",
            "label": "Read-only wallet",
            "role": "user",
            "status": "active",
            "version": 3,
        })
        .to_string()
    }

    fn ctx_with_wallet_detail(state: &str, address: &str, data: Option<String>) -> PageContext {
        let mut ctx = authenticated_ctx();
        ctx.path = format!("/wallet-management/{address}");
        ctx.params
            .insert("address".to_string(), address.to_string());
        ctx.params.insert(
            ADMIN_WALLET_DETAIL_STATE_PARAM.to_string(),
            state.to_string(),
        );
        if let Some(data) = data {
            ctx.params
                .insert(ADMIN_WALLET_DETAIL_DATA_PARAM.to_string(), data);
        }
        ctx
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
    fn strict_projection_accepts_zero_and_rejects_extra_or_impossible_counts() {
        let zero = serde_json::json!({
            "total_users": 0,
            "active_users": 0,
            "inactive_users": 0,
            "new_users_30_days": 0,
        });
        assert_eq!(
            decode_admin_wallet_stats_projection(zero),
            Some(AdminWalletStatsSummary {
                total_users: 0,
                active_users: 0,
                inactive_users: 0,
                new_users_30_days: 0,
            })
        );

        for malformed in [
            serde_json::json!({
                "total_users": -1,
                "active_users": 0,
                "inactive_users": 0,
                "new_users_30_days": 0,
            }),
            serde_json::json!({
                "total_users": 10,
                "active_users": 8,
                "inactive_users": 1,
                "new_users_30_days": 1,
            }),
            serde_json::json!({
                "total_users": 10,
                "active_users": 8,
                "inactive_users": 2,
                "new_users_30_days": 11,
            }),
            serde_json::json!({
                "total_users": 10,
                "active_users": 8,
                "inactive_users": 2,
                "new_users_30_days": 1,
                "growth_rate": 10.0,
            }),
        ] {
            assert!(decode_admin_wallet_stats_projection(malformed).is_none());
        }
    }

    #[test]
    fn wallet_list_projection_is_bounded_redacted_and_route_safe() {
        let value = serde_json::json!({
            "items": [{
                "address": TEST_ADDRESS,
                "chain_id": "56",
                "label": "Treasury",
                "role": "user",
                "status": "active",
                "version": 4
            }],
            "total": 1,
            "limit": 100,
            "offset": 0
        });
        let projection = decode_admin_wallet_list_projection(value).unwrap();
        assert_eq!(projection.items[0].address, TEST_ADDRESS);
        assert_eq!(projection.items[0].version, 4);

        let mut hostile = serde_json::json!({
            "items": [{
                "address": format!("{TEST_ADDRESS}/../x"),
                "chain_id": "56",
                "label": "Treasury",
                "role": "user",
                "status": "active",
                "version": 4
            }],
            "total": 1,
            "limit": 100,
            "offset": 0
        });
        assert!(decode_admin_wallet_list_projection(hostile.take()).is_none());
    }

    #[test]
    fn wallet_list_page_renders_only_verified_rows_and_detail_links() {
        let mut ctx = authenticated_ctx();
        ctx.params.insert(
            ADMIN_WALLET_LIST_STATE_PARAM.to_string(),
            ADMIN_WALLET_LIST_READY.to_string(),
        );
        ctx.params.insert(
            ADMIN_WALLET_LIST_DATA_PARAM.to_string(),
            serde_json::json!({
                "items": [{
                    "address": TEST_ADDRESS,
                    "chain_id": "56",
                    "label": null,
                    "role": "user",
                    "status": "disabled",
                    "version": 2
                }],
                "total": 1,
                "limit": 100,
                "offset": 0
            })
            .to_string(),
        );
        let rendered = html(render(&ctx).1);
        assert!(rendered.contains("data-admin-wallet-list-state=\"ready\""));
        assert!(rendered.contains(TEST_ADDRESS));
        assert!(rendered.contains("/wallet-management/0x1111111111111111111111111111111111111111"));
        assert!(!rendered.contains("metadata"));
        assert!(!rendered.contains("<form"));
    }

    #[test]
    fn ready_summary_is_accessible_responsive_neutral_and_inventory_stays_unavailable() {
        let ctx = ctx_with_stats(
            ADMIN_WALLET_STATS_READY,
            Some(stats_json(1_234, 900, 334, 56)),
        );
        let rendered = html(render(&ctx).1);

        assert!(rendered.contains("data-admin-wallets-state=\"ready\""));
        assert!(rendered.contains("data-admin-wallets-surface=\"list\""));
        assert!(rendered.contains("aria-labelledby=\"admin-wallet-stats-title\""));
        assert!(rendered.contains("sm:grid-cols-2"));
        assert!(rendered.contains("xl:grid-cols-4"));
        assert!(rendered.contains("Total users"));
        assert!(rendered.contains("1,234"));
        assert!(rendered.contains("Users marked active"));
        assert!(rendered.contains(">900<"));
        assert!(rendered.contains("Users marked inactive"));
        assert!(rendered.contains(">334<"));
        assert!(rendered.contains("New users, past 30 days"));
        assert!(rendered.contains(">56<"));
        assert!(rendered.contains("not recent activity measurements"));
        assert!(rendered.contains("data-admin-wallet-inventory-state=\"unavailable\""));
        assert!(rendered.contains("Wallet inventory remains unavailable"));
        assert!(rendered.contains(
            "Wallet rows, filters, details, balances, plans, permissions, activity, and controls"
        ));
        for forbidden in [
            "active users, past 30 days",
            "growth rate",
            "tier distribution",
            "users_by_tier",
            "active_users_30_days",
        ] {
            assert!(!rendered.to_ascii_lowercase().contains(forbidden));
        }
        assert_no_samples_or_controls(&rendered);
    }

    #[test]
    fn all_zero_counts_are_a_ready_summary_not_an_empty_or_unavailable_state() {
        let ctx = ctx_with_stats(ADMIN_WALLET_STATS_READY, Some(stats_json(0, 0, 0, 0)));
        let rendered = html(render(&ctx).1);

        assert!(rendered.contains("data-admin-wallets-state=\"ready\""));
        assert_eq!(rendered.matches(">0<").count(), 4, "{rendered}");
        assert!(!rendered.contains("Wallet summary is unavailable"));
        assert!(!rendered.contains("No users"));
    }

    #[test]
    fn forbidden_unavailable_and_malformed_are_distinct_and_hide_stale_data() {
        for (state, title) in [
            (
                ADMIN_WALLET_STATS_FORBIDDEN,
                "Wallet summary access was denied",
            ),
            (
                ADMIN_WALLET_STATS_UNAVAILABLE,
                "Wallet summary is unavailable",
            ),
            (
                ADMIN_WALLET_STATS_MALFORMED,
                "Wallet summary could not be verified",
            ),
        ] {
            let ctx = ctx_with_stats(state, Some(stats_json(777, 700, 77, 7)));
            let rendered = html(render(&ctx).1);

            assert!(rendered.contains(&format!("data-admin-wallets-state=\"{state}\"")));
            assert!(rendered.contains(title));
            assert!(!rendered.contains(">777<"));
            assert!(!rendered.contains(">700<"));
            assert!(rendered.contains("Wallet inventory remains unavailable"));
            assert_no_samples_or_controls(&rendered);
        }
    }

    #[test]
    fn unknown_or_identity_fields_fail_closed_without_reaching_html() {
        let ctx = ctx_with_stats(
            ADMIN_WALLET_STATS_READY,
            Some(
                serde_json::json!({
                    "total_users": 1,
                    "active_users": 1,
                    "inactive_users": 0,
                    "new_users_30_days": 1,
                    "wallet_address": "0xprivate-wallet",
                    "email": "private@example.test",
                    "balance": "999999",
                    "plan": "private-enterprise-plan",
                    "permissions": ["admin:all"],
                    "growth_rate": 99.9,
                    "active_users_30_days": 1,
                    "users_by_tier": {"private": 1},
                })
                .to_string(),
            ),
        );
        let rendered = html(render(&ctx).1);

        assert!(rendered.contains("data-admin-wallets-state=\"malformed\""));
        for private in [
            "0xprivate-wallet",
            "private@example.test",
            "999999",
            "private-enterprise-plan",
            "admin:all",
            "99.9",
            "users_by_tier",
        ] {
            assert!(
                !rendered.contains(private),
                "leaked `{private}`: {rendered}"
            );
        }
        assert_no_samples_or_controls(&rendered);
    }

    #[test]
    fn list_projection_never_changes_detail_or_disable_unavailable_surfaces() {
        let mut ctx = ctx_with_stats(
            ADMIN_WALLET_STATS_READY,
            Some(stats_json(1_234, 900, 334, 56)),
        );
        ctx.params
            .insert("address".to_string(), TEST_ADDRESS.to_string());

        let detail = html(render_detail(&ctx).1);
        assert!(detail.contains("data-admin-wallet-detail-state=\"unavailable\""));
        assert!(!detail.contains("User status summary"));
        assert!(!detail.contains("1,234"));
        assert_no_samples_or_controls(&detail);

        let disable = html(render_disable(&ctx).1);
        assert!(disable.contains("data-admin-wallet-disable-state=\"unavailable\""));
        assert!(!disable.contains("1,234"));
        assert_no_samples_or_controls(&disable);
    }

    #[test]
    fn wallet_detail_projection_is_strict_redacted_and_route_bound() {
        let mut ctx = ctx_with_wallet_detail(
            ADMIN_WALLET_DETAIL_READY,
            TEST_ADDRESS,
            Some(wallet_detail_json(TEST_ADDRESS)),
        );
        let rendered = html(render_detail(&ctx).1);
        assert!(rendered.contains("data-admin-wallet-detail-state=\"ready\""));
        assert!(rendered.contains(TEST_ADDRESS));
        assert!(rendered.contains("Read-only wallet"));
        assert!(!rendered.contains("metadata"));
        assert!(!rendered.contains("<form"));
        assert!(!rendered.contains("<button"));

        ctx.params.insert(
            ADMIN_WALLET_DETAIL_DATA_PARAM.to_string(),
            serde_json::json!({
                "address": TEST_ADDRESS,
                "chain_id": "56",
                "label": "Read-only wallet",
                "role": "user",
                "status": "active",
                "version": 3,
                "metadata": {"secret": "redacted"},
            })
            .to_string(),
        );
        let malformed = html(render_detail(&ctx).1);
        assert!(malformed.contains("data-admin-wallet-detail-state=\"malformed\""));
        assert!(!malformed.contains("redacted"));

        let mismatched = ctx_with_wallet_detail(
            ADMIN_WALLET_DETAIL_READY,
            TEST_ADDRESS,
            Some(wallet_detail_json(
                "0x2222222222222222222222222222222222222222",
            )),
        );
        assert!(html(render_detail(&mismatched).1)
            .contains("data-admin-wallet-detail-state=\"malformed\""));
    }

    #[test]
    fn signed_out_ready_projection_is_private() {
        let mut ctx = ctx_with_stats(
            ADMIN_WALLET_STATS_READY,
            Some(stats_json(1_234, 900, 334, 56)),
        );
        ctx.user = None;
        let rendered = html(render(&ctx).1);

        assert!(rendered.contains("Sign in required"));
        assert!(!rendered.contains("data-admin-wallets-state"));
        assert!(!rendered.contains("User status summary"));
        assert!(!rendered.contains("1,234"));
        assert!(!rendered.contains("Users marked active"));
        assert_no_samples_or_controls(&rendered);
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
            .insert("address".to_string(), TEST_ADDRESS.to_string());
        ctx.path = format!("/wallet-management/{TEST_ADDRESS}");
        let detail = html(render_detail(&ctx).1);
        ctx.path = format!("/wallet-management/wallets/{TEST_ADDRESS}/disable");
        let disable = html(render_disable(&ctx).1);

        assert!(list.contains("data-admin-wallets-state=\"unavailable\""));
        assert!(list.contains("data-admin-wallets-surface=\"list\""));
        for (surface, rendered) in [("detail", detail), ("disable", disable)] {
            let state_marker = if surface == "detail" {
                "data-admin-wallet-detail-state=\"unavailable\""
            } else {
                "data-admin-wallet-disable-state=\"unavailable\""
            };
            assert!(rendered.contains(state_marker));
            assert!(!rendered.contains("Permission required"));
            assert!(!rendered.contains("Admin access required"));
            assert_no_samples_or_controls(&rendered);
        }
    }

    #[test]
    fn invalid_dynamic_wallet_address_is_malformed_without_reflection() {
        let mut ctx = authenticated_ctx();
        ctx.path = "/wallet-management/hostile".to_string();
        ctx.params.insert(
            "address".to_string(),
            format!("{}<script>alert(1)</script>\n", "x".repeat(80)),
        );
        let rendered = html(render_detail(&ctx).1);

        assert!(rendered.contains("data-admin-wallet-detail-state=\"malformed\""));
        assert!(!rendered.contains("<script>"));
        assert!(!rendered.contains("alert(1)"));
        assert!(!rendered.contains("Unverified route reference"));
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
    fn disable_surface_requires_backend_detail_and_emits_bounded_form() {
        let mut ctx = ctx_with_wallet_detail(
            ADMIN_WALLET_DETAIL_READY,
            TEST_ADDRESS,
            Some(wallet_detail_json(TEST_ADDRESS)),
        );
        ctx.path = format!("/wallet-management/wallets/{TEST_ADDRESS}/disable");
        ctx.params.insert(
            ADMIN_WALLET_DISABLE_STATE_PARAM.to_string(),
            ADMIN_WALLET_DISABLE_FORM.to_string(),
        );
        let rendered = html(render_disable(&ctx).1);
        assert!(rendered.contains("data-admin-wallet-disable-state=\"form\""));
        assert!(rendered.contains("method=\"post\""));
        assert!(rendered.contains("name=\"expected_version\""));
        assert!(rendered.contains("name=\"idempotency_key\""));
        assert!(rendered.contains("name=\"reason\""));
        assert!(rendered.contains("Disable wallet"));
    }

    #[test]
    fn leaves_are_body_only_and_disable_surface_has_no_mutation_affordance() {
        let mut ctx = ctx_with_stats(
            ADMIN_WALLET_STATS_READY,
            Some(stats_json(1_234, 900, 334, 56)),
        );
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
