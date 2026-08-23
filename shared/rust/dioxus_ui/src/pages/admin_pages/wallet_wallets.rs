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
use crate::components::admin::data_state_banner::{AdminDataState, AdminDataStateBanner};
use crate::primitives::Icon;

use super::super::{PageContext, PageMeta};
use super::wallet_access::{
    decode_admin_access_projection, AdminAccessProjection, ADMIN_ACCESS_DATA_PARAM,
    ADMIN_ACCESS_FORBIDDEN, ADMIN_ACCESS_MALFORMED, ADMIN_ACCESS_READY, ADMIN_ACCESS_STATE_PARAM,
    ADMIN_ACCESS_UNAUTHENTICATED, ADMIN_ACCESS_UNAUTHORIZED, ADMIN_ACCESS_UNAVAILABLE,
};
use super::wallet_hub::WalletManagementHub;
use super::wallet_plans::{
    decode_admin_plan_list_projection, AdminPlanListProjection, ADMIN_PLANS_DATA_PARAM,
    ADMIN_PLANS_EMPTY, ADMIN_PLANS_FORBIDDEN, ADMIN_PLANS_MALFORMED, ADMIN_PLANS_READY,
    ADMIN_PLANS_STATE_PARAM, ADMIN_PLANS_UNAUTHENTICATED, ADMIN_PLANS_UNAUTHORIZED,
    ADMIN_PLANS_UNAVAILABLE,
};

const WALLETS_PATH: &str = "/wallet-management/wallets";
const MAX_WALLET_LABEL_CHARS: usize = 100;
const MAX_WALLET_ROLE_CHARS: usize = 64;
const MAX_CHAIN_ID_CHARS: usize = 10;
const DEFAULT_WALLET_PAGE_SIZE: i64 = 10;
const MAX_WALLET_OFFSET: i64 = 10_000_000;

/// Closed wallet-list query contract shared by the admin BFF and page.
/// Filtering and pagination are executed by the wallet service; the UI only
/// serializes validated user intent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdminWalletListQuery {
    pub search: Option<String>,
    pub status: Option<String>,
    pub page: i64,
    pub limit: i64,
}

impl Default for AdminWalletListQuery {
    fn default() -> Self {
        Self {
            search: None,
            status: None,
            page: 1,
            limit: DEFAULT_WALLET_PAGE_SIZE,
        }
    }
}

impl AdminWalletListQuery {
    #[allow(clippy::result_unit_err)]
    pub fn from_raw(raw: &str) -> Result<Self, ()> {
        let mut query = Self::default();
        let mut seen = std::collections::HashSet::new();

        for (key, value) in url::form_urlencoded::parse(raw.as_bytes()) {
            if !seen.insert(key.to_string()) {
                return Err(());
            }
            match key.as_ref() {
                "search" => {
                    let value = value.into_owned();
                    if value.is_empty() {
                        query.search = None;
                    } else if value.len() <= 42
                        && value.bytes().all(|byte| {
                            byte.is_ascii_alphanumeric() || matches!(byte, b'x' | b'X' | b'_')
                        })
                    {
                        query.search = Some(value);
                    } else {
                        return Err(());
                    }
                }
                "status" => match value.as_ref() {
                    "" | "all" => query.status = None,
                    "active" | "disabled" => query.status = Some(value.into_owned()),
                    _ => return Err(()),
                },
                "page" => {
                    query.page = value.parse().map_err(|_| ())?;
                    if query.page < 1 {
                        return Err(());
                    }
                }
                "limit" => {
                    query.limit = value.parse().map_err(|_| ())?;
                    if !matches!(query.limit, 10 | 25 | 50) {
                        return Err(());
                    }
                }
                _ => return Err(()),
            }
        }

        query.offset().map(|_| query).ok_or(())
    }

    pub fn offset(&self) -> Option<i64> {
        self.page
            .checked_sub(1)?
            .checked_mul(self.limit)
            .filter(|offset| *offset <= MAX_WALLET_OFFSET)
    }
}

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
pub const ADMIN_WALLET_STATS_UNAUTHENTICATED: &str = "unauthenticated";
pub const ADMIN_WALLET_STATS_UNAUTHORIZED: &str = "unauthorized";
pub const ADMIN_WALLET_LIST_READY: &str = "ready";
pub const ADMIN_WALLET_LIST_EMPTY: &str = "empty";
pub const ADMIN_WALLET_LIST_FORBIDDEN: &str = "forbidden";
pub const ADMIN_WALLET_LIST_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_WALLET_LIST_MALFORMED: &str = "malformed";
pub const ADMIN_WALLET_LIST_UNAUTHENTICATED: &str = "unauthenticated";
pub const ADMIN_WALLET_LIST_UNAUTHORIZED: &str = "unauthorized";
pub const ADMIN_WALLET_DETAIL_READY: &str = "ready";
pub const ADMIN_WALLET_DETAIL_FORBIDDEN: &str = "forbidden";
pub const ADMIN_WALLET_DETAIL_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_WALLET_DETAIL_MALFORMED: &str = "malformed";
pub const ADMIN_WALLET_DETAIL_UNAUTHENTICATED: &str = "unauthenticated";
pub const ADMIN_WALLET_DETAIL_UNAUTHORIZED: &str = "unauthorized";
pub const ADMIN_WALLET_DISABLE_STATE_PARAM: &str = "data_admin_wallet_disable_state";
pub const ADMIN_WALLET_DISABLE_FORM: &str = "form";
pub const ADMIN_WALLET_DISABLE_SUCCESS: &str = "success";
pub const ADMIN_WALLET_DISABLE_CONFLICT: &str = "conflict";
pub const ADMIN_WALLET_DISABLE_FORBIDDEN: &str = "forbidden";
pub const ADMIN_WALLET_DISABLE_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_WALLET_DISABLE_MALFORMED: &str = "malformed";
pub const ADMIN_WALLET_DISABLE_UNAUTHENTICATED: &str = "unauthenticated";
pub const ADMIN_WALLET_DISABLE_UNAUTHORIZED: &str = "unauthorized";

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
    Unauthenticated,
    Unauthorized,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum WalletListLoad {
    Ready(AdminWalletListProjection),
    Empty,
    Forbidden,
    Unavailable,
    Malformed,
    Unauthenticated,
    Unauthorized,
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
        ADMIN_WALLET_LIST_UNAUTHENTICATED => WalletListLoad::Unauthenticated,
        ADMIN_WALLET_LIST_UNAUTHORIZED => WalletListLoad::Unauthorized,
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
        Some(ADMIN_WALLET_STATS_UNAUTHENTICATED) => WalletStatsLoad::Unauthenticated,
        Some(ADMIN_WALLET_STATS_UNAUTHORIZED) => WalletStatsLoad::Unauthorized,
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
    Unauthenticated,
    Unauthorized,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum WalletDetailAccessLoad {
    Ready(AdminAccessProjection),
    Forbidden,
    Unavailable,
    Malformed,
    Unauthenticated,
    Unauthorized,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum WalletDetailPlansLoad {
    Ready(AdminPlanListProjection),
    Empty,
    Forbidden,
    Unavailable,
    Malformed,
    Unauthenticated,
    Unauthorized,
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
        Some(ADMIN_WALLET_DETAIL_UNAUTHENTICATED) => WalletDetailLoad::Unauthenticated,
        Some(ADMIN_WALLET_DETAIL_UNAUTHORIZED) => WalletDetailLoad::Unauthorized,
        Some(ADMIN_WALLET_DETAIL_UNAVAILABLE) | None => WalletDetailLoad::Unavailable,
        Some(_) => WalletDetailLoad::Malformed,
    }
}

fn wallet_detail_access_load(ctx: &PageContext, address: Option<&str>) -> WalletDetailAccessLoad {
    match ctx.params.get(ADMIN_ACCESS_STATE_PARAM).map(String::as_str) {
        Some(ADMIN_ACCESS_READY) => {
            let Some(raw) = ctx.params.get(ADMIN_ACCESS_DATA_PARAM) else {
                return WalletDetailAccessLoad::Malformed;
            };
            let Some(projection) = serde_json::from_str(raw)
                .ok()
                .and_then(decode_admin_access_projection)
            else {
                return WalletDetailAccessLoad::Malformed;
            };
            let Some(address) = address.and_then(canonical_wallet_address) else {
                return WalletDetailAccessLoad::Malformed;
            };
            if projection.items.iter().all(|item| {
                canonical_wallet_address(&item.wallet_address).as_deref() == Some(address.as_str())
            }) {
                WalletDetailAccessLoad::Ready(projection)
            } else {
                WalletDetailAccessLoad::Malformed
            }
        }
        Some(ADMIN_ACCESS_FORBIDDEN) => WalletDetailAccessLoad::Forbidden,
        Some(ADMIN_ACCESS_MALFORMED) => WalletDetailAccessLoad::Malformed,
        Some(ADMIN_ACCESS_UNAUTHENTICATED) => WalletDetailAccessLoad::Unauthenticated,
        Some(ADMIN_ACCESS_UNAUTHORIZED) => WalletDetailAccessLoad::Unauthorized,
        Some(ADMIN_ACCESS_UNAVAILABLE) | None => WalletDetailAccessLoad::Unavailable,
        Some(_) => WalletDetailAccessLoad::Malformed,
    }
}

fn wallet_detail_plans_load(ctx: &PageContext) -> WalletDetailPlansLoad {
    match ctx.params.get(ADMIN_PLANS_STATE_PARAM).map(String::as_str) {
        Some(ADMIN_PLANS_READY) | Some(ADMIN_PLANS_EMPTY) => {
            let Some(raw) = ctx.params.get(ADMIN_PLANS_DATA_PARAM) else {
                return WalletDetailPlansLoad::Malformed;
            };
            let Some(projection) = serde_json::from_str(raw)
                .ok()
                .and_then(decode_admin_plan_list_projection)
            else {
                return WalletDetailPlansLoad::Malformed;
            };
            match (
                ctx.params.get(ADMIN_PLANS_STATE_PARAM).map(String::as_str),
                projection.items.is_empty(),
                projection.total,
            ) {
                (Some(ADMIN_PLANS_READY), false, _) => WalletDetailPlansLoad::Ready(projection),
                (Some(ADMIN_PLANS_READY), true, total) if total > 0 => {
                    WalletDetailPlansLoad::Ready(projection)
                }
                (Some(ADMIN_PLANS_EMPTY), true, 0) => WalletDetailPlansLoad::Empty,
                _ => WalletDetailPlansLoad::Malformed,
            }
        }
        Some(ADMIN_PLANS_FORBIDDEN) => WalletDetailPlansLoad::Forbidden,
        Some(ADMIN_PLANS_MALFORMED) => WalletDetailPlansLoad::Malformed,
        Some(ADMIN_PLANS_UNAUTHENTICATED) => WalletDetailPlansLoad::Unauthenticated,
        Some(ADMIN_PLANS_UNAUTHORIZED) => WalletDetailPlansLoad::Unauthorized,
        Some(ADMIN_PLANS_UNAVAILABLE) | None => WalletDetailPlansLoad::Unavailable,
        Some(_) => WalletDetailPlansLoad::Malformed,
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
    rsx! {
        WalletManagementHub {
            ctx: ctx.clone(),
            WalletDisableWorkspace { ctx, reference }
        }
    }
}

#[component]
fn WalletDisableWorkspace(ctx: PageContext, reference: Option<String>) -> Element {
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
                | ADMIN_WALLET_DISABLE_UNAUTHENTICATED
                | ADMIN_WALLET_DISABLE_UNAUTHORIZED
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
            ADMIN_WALLET_DISABLE_UNAUTHENTICATED | ADMIN_WALLET_DISABLE_UNAUTHORIZED => {
                let banner_state = if state == ADMIN_WALLET_DISABLE_UNAUTHENTICATED {
                    AdminDataState::Unauthenticated
                } else {
                    AdminDataState::Unauthorized
                };
                rsx! {
                    AdminDataStateBanner {
                        state: banner_state,
                        subject: "Wallet disable".to_string(),
                        return_path: WALLETS_PATH.to_string(),
                        retry_href: WALLETS_PATH.to_string(),
                    }
                }
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
            let short_address = format!(
                "{}...{}",
                projection.address.chars().take(6).collect::<String>(),
                projection
                    .address
                    .chars()
                    .rev()
                    .take(4)
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect::<String>()
            );
            rsx! {
                div { class: "space-y-6",
                    "data-admin-wallet-disable-state": ADMIN_WALLET_DISABLE_FORM,
                    WalletDisableHeader {}
                    form { method: "post", action,
                        input { type: "hidden", name: "expected_version", value: projection.version.to_string() }
                        input { type: "hidden", name: "idempotency_key", value: idempotency_key }
                        div { class: "grid grid-cols-1 gap-6 lg:grid-cols-3",
                            div { class: "space-y-6 lg:col-span-2",
                                section { class: "rounded-2xl border border-border/20 bg-white/[0.02] p-6",
                                    div { class: "mb-4 flex items-center gap-2",
                                        Icon { name: "clock".to_string(), size: Some(20), class_name: Some("text-[#1fc7d4]".to_string()) }
                                        h2 { class: "text-lg font-semibold", "Disable Duration" }
                                    }
                                    div { class: "grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-5",
                                        for (label, detail) in [("24 hours", "Short-term"), ("7 days", "Review"), ("30 days", "Investigation"), ("90 days", "Extended"), ("Indefinite", "Backend-owned")] {
                                            button { class: if label == "Indefinite" { "flex cursor-not-allowed flex-col items-center gap-1 rounded-xl border border-amber-500/50 bg-amber-500/10 px-3 py-3 text-center text-amber-400" } else { "flex cursor-not-allowed flex-col items-center gap-1 rounded-xl border border-border/20 px-3 py-3 text-center text-muted-foreground opacity-50" }, r#type: "button", disabled: true, title: "The wallet mutation currently owns an indefinite disable only",
                                                span { class: "text-sm font-bold", "{label}" }
                                                span { class: "text-[10px] opacity-60", "{detail}" }
                                            }
                                        }
                                    }
                                }
                                section { class: "rounded-2xl border border-border/20 bg-white/[0.02] p-6",
                                    div { class: "mb-4 flex items-center justify-between gap-3",
                                        div { class: "flex items-center gap-2",
                                            Icon { name: "bar-chart-3".to_string(), size: Some(20), class_name: Some("text-[#7645d9]".to_string()) }
                                            h2 { class: "text-lg font-semibold", "Affected Platforms" }
                                        }
                                        span { class: "text-xs text-muted-foreground", "All backend-authorized access" }
                                    }
                                    div { class: "grid grid-cols-1 gap-3 sm:grid-cols-2",
                                        for label in ["EPSX Analytics", "EPSX Pay", "EPSX Token", "EPSX Markets"] {
                                            div { class: "flex items-center gap-3 rounded-xl border border-border/20 px-4 py-3 text-sm text-muted-foreground opacity-60",
                                                input { r#type: "checkbox", checked: true, disabled: true, aria_label: label }
                                                "{label}"
                                            }
                                        }
                                    }
                                    p { class: "mt-3 text-xs leading-5 text-muted-foreground", "The current wallet status mutation applies globally; platform-scoped disabling is not exposed by the backend." }
                                }
                                section { class: "space-y-4 rounded-2xl border border-border/20 bg-white/[0.02] p-6",
                                    div { class: "flex items-center gap-2",
                                        Icon { name: "file-text".to_string(), size: Some(20), class_name: Some("text-[#ed4b9e]".to_string()) }
                                        h2 { class: "text-lg font-semibold", "Reason & Details" }
                                    }
                                    label { class: "block text-sm text-muted-foreground", "Category"
                                        select { class: "select select-bordered mt-1.5 w-full cursor-not-allowed bg-white/[0.02]", disabled: true,
                                            option { "Administrative action" }
                                        }
                                    }
                                    label { class: "block text-sm text-muted-foreground", r#for: "wallet-disable-reason", "Details (Required)"
                                        textarea { id: "wallet-disable-reason", name: "reason", required: true, maxlength: "500", rows: "5", class: "textarea textarea-bordered mt-1.5 min-h-[120px] w-full resize-none bg-white/[0.02] text-foreground", placeholder: "Explain why this wallet is being disabled. Be specific for audit purposes..." }
                                    }
                                }
                                section { class: "rounded-2xl border border-border/20 bg-white/[0.02] p-6",
                                    div { class: "mb-4 flex items-center gap-2",
                                        Icon { name: "shield".to_string(), size: Some(20), class_name: Some("text-muted-foreground".to_string()) }
                                        h2 { class: "text-lg font-semibold", "Additional Actions" }
                                    }
                                    div { class: "space-y-3",
                                        for (label, detail) in [("Block login across all platforms", "The wallet status contract controls access globally"), ("Pause active subscriptions", "No idempotent subscription pause contract is exposed"), ("Send notification to user", "No wallet-status notification contract is exposed")] {
                                            div { class: "flex items-start gap-3 rounded-xl border border-border/20 px-4 py-3 opacity-60",
                                                input { r#type: "checkbox", checked: label.starts_with("Block login"), disabled: true, aria_label: label }
                                                div {
                                                    p { class: "text-sm font-medium", "{label}" }
                                                    p { class: "mt-0.5 text-xs text-muted-foreground", "{detail}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            aside { class: "h-fit rounded-2xl border border-amber-500/20 bg-amber-500/5 p-6 lg:sticky lg:top-6",
                                h2 { class: "text-sm font-bold uppercase tracking-wider text-amber-400", "Action Summary" }
                                dl { class: "mt-4 space-y-4",
                                    div {
                                        dt { class: "text-xs text-muted-foreground", "Wallet" }
                                        dd { class: "mt-0.5 font-mono text-sm text-foreground", "{short_address}" }
                                    }
                                    div {
                                        dt { class: "text-xs text-muted-foreground", "Duration" }
                                        dd { class: "mt-0.5 text-sm font-medium text-foreground", "Indefinite" }
                                    }
                                    div {
                                        dt { class: "text-xs text-muted-foreground", "Scope" }
                                        dd { class: "mt-0.5 text-sm font-medium text-foreground", "Global wallet status" }
                                    }
                                    div {
                                        dt { class: "text-xs text-muted-foreground", "Read version" }
                                        dd { class: "mt-0.5 font-mono text-sm text-foreground", "{projection.version}" }
                                    }
                                }
                                div { class: "mt-5 rounded-xl border border-amber-500/20 bg-amber-500/10 p-3",
                                    div { class: "flex gap-2",
                                        Icon { name: "triangle-alert".to_string(), size: Some(16), class_name: Some("mt-0.5 shrink-0 text-amber-400".to_string()) }
                                        p { class: "text-xs leading-5 text-amber-300/90", "The wallet service rechecks permission, current status, and version before committing this audited change." }
                                    }
                                }
                                button { class: "btn mt-5 w-full bg-gradient-to-r from-amber-600 to-red-600 font-bold text-white", type: "submit",
                                    Icon { name: "shield".to_string(), size: Some(16) }
                                    "Disable Wallet"
                                }
                                a { class: "btn btn-ghost mt-2 w-full", href: format!("/wallet-management/{}", projection.address), "Cancel" }
                            }
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
        WalletDetailLoad::Unauthenticated => {
            let disable_href = format!("/wallet-management/wallets/{reference}/disable");
            rsx! {
                AdminDataStateBanner {
                    state: AdminDataState::Unauthenticated,
                    subject: "Wallet detail".to_string(),
                    return_path: disable_href.clone(),
                    retry_href: disable_href,
                }
            }
        }
        WalletDetailLoad::Unauthorized => {
            let disable_href = format!("/wallet-management/wallets/{reference}/disable");
            rsx! {
                AdminDataStateBanner {
                    state: AdminDataState::Unauthorized,
                    subject: "Wallet detail".to_string(),
                    return_path: disable_href.clone(),
                    retry_href: disable_href,
                }
            }
        }
    }
}

#[component]
fn WalletDisableNotice(state: &'static str, title: String, detail: String) -> Element {
    rsx! {
        div { class: "space-y-6", "data-admin-wallet-disable-state": state,
            WalletDisableHeader {}
            section { class: "rounded-2xl border border-amber-500/30 bg-amber-500/5 p-5 sm:p-6", role: "status",
                div { class: "flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between",
                    div { class: "flex items-start gap-3",
                        Icon { name: "triangle-alert".to_string(), size: Some(20), class_name: Some("mt-0.5 shrink-0 text-amber-400".to_string()) }
                        div {
                            h2 { class: "font-semibold text-foreground", "{title}" }
                            p { class: "mt-1 text-sm leading-6 text-muted-foreground", "{detail}" }
                        }
                    }
                    a { class: "btn btn-sm btn-outline shrink-0", href: "/wallet-management/wallets", "Wallet inventory" }
                }
            }
            WalletDisableScaffold {}
        }
    }
}

#[component]
fn WalletDisableHeader() -> Element {
    rsx! {
        header { class: "flex items-center gap-4",
            a { class: "rounded-xl border border-border/20 bg-muted/30 p-2.5 text-muted-foreground transition-colors hover:bg-muted/50 hover:text-foreground", href: WALLETS_PATH, aria_label: "Back to wallets",
                Icon { name: "arrow-left".to_string(), size: Some(20) }
            }
            div {
                h1 { class: "flex items-center gap-3 text-2xl font-bold sm:text-3xl",
                    span { class: "rounded-xl border border-amber-500/20 bg-amber-500/10 p-2",
                        Icon { name: "shield".to_string(), size: Some(24), class_name: Some("text-amber-500".to_string()) }
                    }
                    span { class: "text-amber-400", "Disable Wallet" }
                }
                p { class: "mt-1 text-sm text-muted-foreground", "Restrict access through the backend-authorized wallet status contract" }
            }
        }
    }
}

#[component]
fn WalletDisableScaffold() -> Element {
    rsx! {
        div { class: "grid grid-cols-1 gap-6 lg:grid-cols-3", aria_hidden: "true",
            div { class: "space-y-6 lg:col-span-2",
                section { class: "rounded-2xl border border-border/20 bg-white/[0.02] p-6",
                    div { class: "mb-4 flex items-center gap-2",
                        Icon { name: "clock".to_string(), size: Some(20), class_name: Some("text-[#1fc7d4]".to_string()) }
                        h2 { class: "text-lg font-semibold", "Disable Duration" }
                    }
                    div { class: "grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-5",
                        for label in ["24 hours", "7 days", "30 days", "90 days", "Indefinite"] {
                            button { class: "cursor-not-allowed rounded-xl border border-border/20 px-3 py-3 text-center text-sm font-bold text-muted-foreground opacity-50", r#type: "button", disabled: true, "{label}" }
                        }
                    }
                }
                section { class: "rounded-2xl border border-border/20 bg-white/[0.02] p-6",
                    div { class: "mb-4 flex items-center gap-2",
                        Icon { name: "bar-chart-3".to_string(), size: Some(20), class_name: Some("text-[#7645d9]".to_string()) }
                        h2 { class: "text-lg font-semibold", "Affected Platforms" }
                    }
                    div { class: "grid grid-cols-1 gap-3 sm:grid-cols-2",
                        for label in ["EPSX Analytics", "EPSX Pay", "EPSX Token", "EPSX Markets"] {
                            div { class: "rounded-xl border border-border/20 px-4 py-3 text-sm text-muted-foreground opacity-50", "{label}" }
                        }
                    }
                }
                section { class: "space-y-4 rounded-2xl border border-border/20 bg-white/[0.02] p-6",
                    div { class: "flex items-center gap-2",
                        Icon { name: "file-text".to_string(), size: Some(20), class_name: Some("text-[#ed4b9e]".to_string()) }
                        h2 { class: "text-lg font-semibold", "Reason & Details" }
                    }
                    div { class: "h-10 rounded-xl border border-border/20 bg-background/40 opacity-50" }
                    div { class: "h-28 rounded-xl border border-border/20 bg-background/40 opacity-50" }
                }
                section { class: "rounded-2xl border border-border/20 bg-white/[0.02] p-6",
                    h2 { class: "text-lg font-semibold", "Additional Actions" }
                    div { class: "mt-4 space-y-3",
                        for label in ["Block login across all platforms", "Pause active subscriptions", "Send notification to user"] {
                            div { class: "rounded-xl border border-border/20 px-4 py-3 text-sm text-muted-foreground opacity-50", "{label}" }
                        }
                    }
                }
            }
            aside { class: "h-fit rounded-2xl border border-amber-500/20 bg-amber-500/5 p-6 lg:sticky lg:top-6",
                h2 { class: "text-sm font-bold uppercase tracking-wider text-amber-400", "Action Summary" }
                p { class: "mt-4 text-sm text-muted-foreground", "Wallet status and version must be verified before this action can be configured." }
                button { class: "btn mt-6 w-full cursor-not-allowed bg-gradient-to-r from-amber-600 to-red-600 text-white opacity-50", r#type: "button", disabled: true, "Disable Wallet" }
                a { class: "btn btn-ghost mt-2 w-full", href: WALLETS_PATH, tabindex: "-1", "Cancel" }
            }
        }
    }
}

#[component]
fn RenderWalletDetail(ctx: PageContext) -> Element {
    let route_address = ctx
        .params
        .get("address")
        .and_then(|address| canonical_wallet_address(address))
        .unwrap_or_default();
    let detail = wallet_detail_load(&ctx);
    let access = wallet_detail_access_load(&ctx, Some(route_address.as_str()));
    let plans = wallet_detail_plans_load(&ctx);

    rsx! {
        WalletManagementHub {
            ctx: ctx.clone(),
            WalletDetailWorkspace {
                route_address,
                detail,
                access,
                plans,
            }
        }
    }
}

#[component]
fn WalletDetailWorkspace(
    route_address: String,
    detail: WalletDetailLoad,
    access: WalletDetailAccessLoad,
    plans: WalletDetailPlansLoad,
) -> Element {
    let wallet = match detail.clone() {
        WalletDetailLoad::Ready(projection) => Some(projection),
        _ => None,
    };
    let auth_state = match detail.clone() {
        WalletDetailLoad::Unauthenticated => Some(AdminDataState::Unauthenticated),
        WalletDetailLoad::Unauthorized => Some(AdminDataState::Unauthorized),
        _ => None,
    };
    let problem = match detail {
        WalletDetailLoad::Ready(_) => None,
        WalletDetailLoad::Forbidden => Some((
            ADMIN_WALLET_DETAIL_FORBIDDEN,
            "Wallet detail access was denied",
            "The backend did not authorize this session to read the requested wallet.",
        )),
        WalletDetailLoad::Unavailable => Some((
            ADMIN_WALLET_DETAIL_UNAVAILABLE,
            "Wallet detail is unavailable",
            "The wallet backend could not provide an authoritative wallet response. No wallet fields are being shown.",
        )),
        WalletDetailLoad::Malformed => Some((
            ADMIN_WALLET_DETAIL_MALFORMED,
            "Wallet detail could not be verified",
            "The route address or backend response did not match the strict wallet contract. No wallet fields are being shown.",
        )),
        WalletDetailLoad::Unauthenticated | WalletDetailLoad::Unauthorized => None,
    };

    rsx! {
        section { class: "mx-auto max-w-6xl space-y-6", "data-admin-wallet-detail-surface": "workspace",
            header { class: "flex flex-col gap-4 sm:flex-row sm:items-center",
                a { class: "flex h-10 w-10 shrink-0 items-center justify-center rounded-xl border border-border/40 bg-card transition-colors hover:bg-muted/30", href: WALLETS_PATH, aria_label: "Back to wallet inventory",
                    Icon { name: "arrow-left".to_string(), size: Some(20) }
                }
                div { class: "min-w-0 flex-1",
                    h1 { class: "flex items-center gap-2 text-2xl font-bold text-foreground",
                        span { aria_hidden: "true", "👛" }
                        "Wallet Details"
                    }
                    p { class: "mt-1 text-sm text-muted-foreground", "Manage wallet access and plans" }
                }
                a { class: "btn btn-outline gap-2 self-start sm:self-auto", href: format!("/wallet-management/{route_address}"),
                    Icon { name: "refresh-cw".to_string(), size: Some(16) }
                    "Refresh"
                }
            }
            div { class: "border-t border-border/20 pt-6",
                div { class: "mb-6",
                    h2 { class: "flex items-center gap-2 text-xl font-bold text-foreground",
                        span { class: "text-purple-400", Icon { name: "shield-check".to_string(), size: Some(21) } }
                        "Wallet Identity & Access Management"
                    }
                    p { class: "mt-1 text-sm text-muted-foreground", "Manage wallet identification, subscription plans, and access plans" }
                }
                if let Some(state) = auth_state {
                    AdminDataStateBanner {
                        state,
                        subject: "Wallet detail".to_string(),
                        return_path: format!("/wallet-management/{route_address}"),
                        retry_href: format!("/wallet-management/{route_address}"),
                    }
                }
                if let Some((state, title, message)) = problem {
                    WalletDetailProblemBanner {
                        state,
                        title: title.to_string(),
                        detail: message.to_string(),
                        route_address: route_address.clone(),
                    }
                }
                div { class: "grid grid-cols-1 gap-6 lg:grid-cols-3",
                    div { class: "lg:col-span-1",
                        WalletAvailablePlansPanel { plans, address: route_address.clone() }
                    }
                    div { class: "space-y-6 lg:col-span-2",
                        WalletMetadataPanel { wallet, address: route_address.clone() }
                        WalletSubscriptionPanel {}
                        WalletAssignedPlansPanel { access }
                    }
                }
            }
        }
    }
}

#[component]
fn WalletMetadataPanel(wallet: Option<AdminWalletDetailProjection>, address: String) -> Element {
    let label = wallet
        .as_ref()
        .and_then(|wallet| wallet.label.clone())
        .unwrap_or_default();
    let status = wallet
        .as_ref()
        .map(|wallet| wallet.status.as_str())
        .unwrap_or("Unavailable");
    let chain = wallet
        .as_ref()
        .map(|wallet| wallet.chain_id.as_str())
        .unwrap_or("Unavailable");
    let role = wallet
        .as_ref()
        .and_then(|wallet| wallet.role.as_deref())
        .unwrap_or("Not reported");
    let status_class = if status == "active" {
        "border-green-500/30 bg-green-500/10 text-green-400"
    } else if status == "disabled" {
        "border-amber-500/30 bg-amber-500/10 text-amber-400"
    } else {
        "border-border/30 bg-muted/30 text-muted-foreground"
    };

    rsx! {
        section { class: "overflow-hidden rounded-xl border border-border/20 bg-card shadow-lg", "data-admin-wallet-detail-state": if wallet.is_some() { ADMIN_WALLET_DETAIL_READY } else { ADMIN_WALLET_DETAIL_UNAVAILABLE },
            div { class: "h-1 bg-gradient-to-r from-amber-500 to-orange-500" }
            header { class: "border-b border-border/20 bg-muted/30 p-5",
                div { class: "flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between",
                    h3 { class: "text-sm font-semibold uppercase tracking-wider text-amber-400", "Wallet Details" }
                    div { class: "flex min-w-0 flex-wrap items-center gap-2",
                        span { class: "rounded-full border px-2 py-0.5 text-[10px] font-bold uppercase {status_class}", "{status}" }
                        span { class: "max-w-full break-all rounded-full border border-border/20 bg-black/20 px-3 py-1 font-mono text-[10px] text-muted-foreground", "{address}" }
                    }
                }
            }
            div { class: "grid grid-cols-1 gap-4 p-5 md:grid-cols-2",
                div {
                    label { class: "text-xs text-muted-foreground", r#for: "wallet-detail-label", "Label" }
                    input { id: "wallet-detail-label", class: "mt-2 h-9 w-full rounded-lg border border-border/20 bg-background/50 px-3 text-sm", value: label, readonly: true, placeholder: "No label reported" }
                }
                div {
                    label { class: "text-xs text-muted-foreground", r#for: "wallet-detail-note", "Private Note" }
                    input { id: "wallet-detail-note", class: "mt-2 h-9 w-full rounded-lg border border-border/20 bg-background/50 px-3 text-sm text-muted-foreground", readonly: true, placeholder: "Metadata note projection unavailable" }
                }
                dl { class: "grid grid-cols-2 gap-3 md:col-span-2",
                    div { class: "rounded-lg border border-border/20 bg-background/30 p-3",
                        dt { class: "text-[10px] uppercase tracking-wider text-muted-foreground", "Chain" }
                        dd { class: "mt-1 text-sm font-semibold text-foreground", "{chain}" }
                    }
                    div { class: "rounded-lg border border-border/20 bg-background/30 p-3",
                        dt { class: "text-[10px] uppercase tracking-wider text-muted-foreground", "Role" }
                        dd { class: "mt-1 text-sm font-semibold text-foreground", "{role}" }
                    }
                }
            }
            footer { class: "flex flex-wrap justify-end gap-2 px-5 pb-5",
                button { class: "btn btn-sm btn-ghost", disabled: true, title: "No editable metadata patch projection is exposed to this page", "Discard" }
                button { class: "btn btn-sm btn-primary", disabled: true, title: "No editable metadata patch projection is exposed to this page",
                    Icon { name: "save".to_string(), size: Some(14) }
                    " Update Details"
                }
                if wallet.as_ref().is_some_and(|wallet| wallet.status == "active") {
                    a { class: "btn btn-sm btn-outline text-amber-400", href: format!("/wallet-management/wallets/{address}/disable"), "Disable wallet" }
                }
            }
        }
    }
}

#[component]
fn WalletSubscriptionPanel() -> Element {
    rsx! {
        section { class: "rounded-xl border border-purple-500/20 bg-card p-4 shadow-lg", aria_label: "Active subscription",
            div { class: "flex items-center gap-4",
                span { class: "flex h-10 w-10 items-center justify-center rounded-lg border border-purple-500/20 bg-purple-500/10 text-purple-400",
                    Icon { name: "package".to_string(), size: Some(20) }
                }
                div { class: "min-w-0 flex-1",
                    h3 { class: "text-sm font-bold uppercase tracking-wider text-foreground", "Active Subscription" }
                    p { class: "mt-1 text-xs text-muted-foreground", "Subscription detail projection unavailable" }
                }
                span { class: "rounded-full border border-border/20 px-2 py-1 text-[10px] font-bold uppercase text-muted-foreground", "Unavailable" }
            }
        }
    }
}

#[component]
fn WalletAvailablePlansPanel(plans: WalletDetailPlansLoad, address: String) -> Element {
    rsx! {
        section { class: "flex h-full min-h-[420px] flex-col overflow-hidden rounded-xl border border-border/20 bg-card shadow-lg", aria_label: "Available plans",
            header { class: "border-b border-border/20 bg-muted/30 p-5",
                div { class: "flex items-start justify-between gap-3",
                    div {
                        h3 { class: "text-sm font-semibold text-foreground", "Available Plans" }
                        p { class: "mt-1 text-xs text-muted-foreground", "Assign backend-owned plan permissions" }
                    }
                    span { class: "rounded-full border border-blue-500/30 px-2 py-1 text-[10px] text-blue-400",
                        match &plans {
                            WalletDetailPlansLoad::Ready(projection) => format!("{} available", projection.items.len()),
                            WalletDetailPlansLoad::Empty => "0 available".to_string(),
                            _ => "Unavailable".to_string(),
                        }
                    }
                }
                input { class: "mt-4 h-8 w-full rounded-lg border border-border/20 bg-background/50 px-3 text-xs text-muted-foreground", placeholder: "Search plans...", disabled: true, title: "Wallet detail plan search requires a URL query contract" }
            }
            div { class: "flex-1 divide-y divide-border/20 overflow-y-auto",
                match plans {
                    WalletDetailPlansLoad::Ready(projection) => rsx! {
                        for plan in projection.items {
                            article { class: "space-y-3 p-4",
                                div { class: "flex items-start justify-between gap-3",
                                    div { class: "min-w-0",
                                        h4 { class: "font-semibold text-foreground", "{plan.name}" }
                                        p { class: "mt-1 line-clamp-2 text-xs text-muted-foreground", {plan.description.clone().unwrap_or_else(|| "No plan description".to_string())} }
                                    }
                                    span { class: "rounded border border-border/20 px-2 py-1 text-[10px] uppercase text-muted-foreground", if plan.active == Some(false) { "Inactive" } else { "Active" } }
                                }
                                form { class: "flex flex-col gap-2 sm:flex-row", method: "post", action: "/wallet-management/access",
                                    input { type: "hidden", name: "operation", value: "access_assign" }
                                    input { type: "hidden", name: "wallet_address", value: address.clone() }
                                    input { type: "hidden", name: "plan_id", value: plan.id.clone() }
                                    input { type: "hidden", name: "expected_version", value: "0" }
                                    input { type: "hidden", name: "idempotency_key", value: format!("admin.wallet-detail.assign.{}", Uuid::new_v4()) }
                                    input { class: "h-8 min-w-0 flex-1 rounded-lg border border-border/20 bg-background/50 px-3 font-mono text-xs", name: "permission", required: true, maxlength: "128", pattern: "[A-Za-z0-9:_-]+", placeholder: "permission:key" }
                                    button { class: "btn btn-sm btn-outline", type: "submit", disabled: plan.active == Some(false), "Assign" }
                                }
                            }
                        }
                    },
                    WalletDetailPlansLoad::Empty => rsx! { WalletDetailPanelEmpty { icon: "package", title: "No plans available", detail: "The backend returned an authoritative empty plan catalog." } },
                    WalletDetailPlansLoad::Forbidden => rsx! { WalletDetailPanelEmpty { icon: "shield", title: "Plan access denied", detail: "The backend did not authorize this plan catalog read." } },
                    WalletDetailPlansLoad::Unavailable => rsx! { WalletDetailPanelEmpty { icon: "package", title: "Plans unavailable", detail: "No unverified plan definitions are shown." } },
                    WalletDetailPlansLoad::Malformed => rsx! { WalletDetailPanelEmpty { icon: "triangle-alert", title: "Plans could not be verified", detail: "The plan catalog did not match its strict contract." } },
                    WalletDetailPlansLoad::Unauthenticated => rsx! {
                        AdminDataStateBanner {
                            state: AdminDataState::Unauthenticated,
                            subject: "Wallet plans".to_string(),
                            return_path: WALLETS_PATH.to_string(),
                            retry_href: WALLETS_PATH.to_string(),
                        }
                    },
                    WalletDetailPlansLoad::Unauthorized => rsx! {
                        AdminDataStateBanner {
                            state: AdminDataState::Unauthorized,
                            subject: "Wallet plans".to_string(),
                            return_path: WALLETS_PATH.to_string(),
                            retry_href: WALLETS_PATH.to_string(),
                        }
                    },
                }
            }
        }
    }
}

#[component]
fn WalletAssignedPlansPanel(access: WalletDetailAccessLoad) -> Element {
    rsx! {
        section { class: "overflow-hidden rounded-xl border border-border/20 bg-card shadow-lg", aria_label: "Assigned plans",
            div { class: "h-1 bg-gradient-to-r from-blue-500 to-cyan-500" }
            header { class: "border-b border-border/20 bg-muted/30 p-5",
                div { class: "flex items-center justify-between gap-3",
                    h3 { class: "text-sm font-semibold uppercase tracking-wider text-blue-400", "Assigned Plans" }
                    span { class: "rounded-full border border-blue-500/30 px-2 py-1 text-[10px] text-blue-400",
                        match &access {
                            WalletDetailAccessLoad::Ready(projection) => projection.items.len().to_string(),
                            _ => "Unavailable".to_string(),
                        }
                    }
                }
                input { class: "mt-4 h-8 w-full rounded-lg border border-border/20 bg-background/50 px-3 text-xs text-muted-foreground", placeholder: "Search within assignments...", disabled: true, title: "Assignment search requires a URL query contract" }
            }
            div { class: "divide-y divide-border/20",
                match access {
                    WalletDetailAccessLoad::Ready(projection) if projection.items.is_empty() => rsx! { WalletDetailPanelEmpty { icon: "shield-check", title: "No assigned plans", detail: "The backend returned an authoritative empty assignment set." } },
                    WalletDetailAccessLoad::Ready(projection) => rsx! {
                        for item in projection.items {
                            article { class: "flex flex-col gap-3 p-4 sm:flex-row sm:items-center sm:justify-between",
                                div { class: "min-w-0",
                                    h4 { class: "font-semibold text-foreground", "{item.plan_name}" }
                                    p { class: "mt-1 break-all font-mono text-xs text-muted-foreground", "{item.permission}" }
                                    p { class: "mt-1 text-[10px] uppercase tracking-wider text-muted-foreground/60", "Version {item.version}" }
                                }
                                form { method: "post", action: "/wallet-management/access",
                                    input { type: "hidden", name: "operation", value: "access_revoke" }
                                    input { type: "hidden", name: "wallet_address", value: item.wallet_address.clone() }
                                    input { type: "hidden", name: "plan_id", value: item.plan_id.clone() }
                                    input { type: "hidden", name: "permission", value: item.permission.clone() }
                                    input { type: "hidden", name: "expected_version", value: item.version.to_string() }
                                    input { type: "hidden", name: "idempotency_key", value: format!("admin.wallet-detail.revoke.{}", Uuid::new_v4()) }
                                    button { class: "btn btn-sm btn-outline", type: "submit", "Remove" }
                                }
                            }
                        }
                    },
                    WalletDetailAccessLoad::Forbidden => rsx! { WalletDetailPanelEmpty { icon: "shield", title: "Assignments denied", detail: "The backend did not authorize this wallet assignment read." } },
                    WalletDetailAccessLoad::Unavailable => rsx! { WalletDetailPanelEmpty { icon: "shield-check", title: "Assignments unavailable", detail: "No unverified wallet assignments are shown." } },
                    WalletDetailAccessLoad::Malformed => rsx! { WalletDetailPanelEmpty { icon: "triangle-alert", title: "Assignments could not be verified", detail: "The wallet assignment response did not match its strict contract." } },
                    WalletDetailAccessLoad::Unauthenticated => rsx! {
                        AdminDataStateBanner {
                            state: AdminDataState::Unauthenticated,
                            subject: "Wallet access".to_string(),
                            return_path: WALLETS_PATH.to_string(),
                            retry_href: WALLETS_PATH.to_string(),
                        }
                    },
                    WalletDetailAccessLoad::Unauthorized => rsx! {
                        AdminDataStateBanner {
                            state: AdminDataState::Unauthorized,
                            subject: "Wallet access".to_string(),
                            return_path: WALLETS_PATH.to_string(),
                            retry_href: WALLETS_PATH.to_string(),
                        }
                    },
                }
            }
        }
    }
}

#[component]
fn WalletDetailPanelEmpty(
    icon: &'static str,
    title: &'static str,
    detail: &'static str,
) -> Element {
    rsx! {
        div { class: "flex min-h-44 flex-col items-center justify-center p-8 text-center",
            span { class: "text-muted-foreground", Icon { name: icon.to_string(), size: Some(30) } }
            h4 { class: "mt-3 font-semibold text-foreground", "{title}" }
            p { class: "mt-1 max-w-sm text-xs leading-5 text-muted-foreground", "{detail}" }
        }
    }
}

#[component]
fn WalletDetailProblemBanner(
    state: &'static str,
    title: String,
    detail: String,
    route_address: String,
) -> Element {
    rsx! {
        section { class: "mb-6 rounded-xl border border-amber-500/25 bg-amber-500/10 px-5 py-4", role: if state == ADMIN_WALLET_DETAIL_FORBIDDEN { "alert" } else { "status" }, "data-admin-wallet-detail-state": state,
            div { class: "flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between",
                div { class: "min-w-0",
                    h3 { class: "font-semibold text-foreground", "{title}" }
                    p { class: "mt-1 max-w-3xl text-sm text-muted-foreground", "{detail}" }
                }
                nav { class: "flex shrink-0 flex-wrap gap-2", aria_label: "Wallet detail recovery",
                    a { class: "btn btn-sm btn-outline", href: format!("/wallet-management/{route_address}"), "Retry wallet read" }
                    a { class: "btn btn-sm btn-ghost", href: WALLETS_PATH, "Wallet list" }
                }
            }
        }
    }
}

#[component]
fn RenderWalletList(ctx: PageContext) -> Element {
    let inventory = wallet_list_load(&ctx);
    let has_inventory = inventory.is_some();
    let query = AdminWalletListQuery::from_raw(&ctx.query).unwrap_or_default();
    let wallet_total = match inventory.as_ref() {
        Some(WalletListLoad::Ready(projection)) => Some(projection.total),
        Some(WalletListLoad::Empty) => Some(0),
        _ => None,
    };
    let load = wallet_stats_load(&ctx);

    rsx! {
        div {
            "data-admin-wallets-surface": "list",
            WalletManagementHub {
                ctx: ctx.clone(),
                WalletSectionHeader { total: wallet_total }
                WalletFilterBar { query: query.clone() }
                match inventory {
                    Some(WalletListLoad::Ready(projection)) => rsx! { WalletListReady { projection, query: query.clone() } },
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
                    Some(WalletListLoad::Unauthenticated) => rsx! {
                        AdminDataStateBanner {
                            state: AdminDataState::Unauthenticated,
                            subject: "Wallet inventory".to_string(),
                            return_path: WALLETS_PATH.to_string(),
                            retry_href: WALLETS_PATH.to_string(),
                        }
                    },
                    Some(WalletListLoad::Unauthorized) => rsx! {
                        AdminDataStateBanner {
                            state: AdminDataState::Unauthorized,
                            subject: "Wallet inventory".to_string(),
                            return_path: WALLETS_PATH.to_string(),
                            retry_href: WALLETS_PATH.to_string(),
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
                        WalletStatsLoad::Unauthenticated => rsx! {
                            AdminDataStateBanner {
                                state: AdminDataState::Unauthenticated,
                                subject: "Wallet stats".to_string(),
                                return_path: WALLETS_PATH.to_string(),
                                retry_href: WALLETS_PATH.to_string(),
                            }
                        },
                        WalletStatsLoad::Unauthorized => rsx! {
                            AdminDataStateBanner {
                                state: AdminDataState::Unauthorized,
                                subject: "Wallet stats".to_string(),
                                return_path: WALLETS_PATH.to_string(),
                                retry_href: WALLETS_PATH.to_string(),
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
fn WalletFilterBar(query: AdminWalletListQuery) -> Element {
    let search = query.search.clone().unwrap_or_default();
    let status = query.status.as_deref().unwrap_or("all");

    rsx! {
        form {
            class: "flex flex-col gap-3 rounded-2xl border border-border/20 bg-card p-4 shadow-lg sm:flex-row",
            method: "get",
            action: WALLETS_PATH,
            aria_label: "Wallet filters",
            div { class: "relative min-w-0 flex-1",
                span { class: "pointer-events-none absolute left-3.5 top-1/2 -translate-y-1/2 text-muted-foreground", aria_hidden: "true",
                    Icon { name: "search".to_string(), size: Some(16) }
                }
                input {
                    class: "h-10 w-full rounded-xl border border-border/30 bg-muted/30 pl-10 pr-3 text-sm text-foreground placeholder:text-muted-foreground/40 focus:border-[#1fc7d4]/50 focus:outline-none",
                    name: "search",
                    value: search,
                    maxlength: "42",
                    placeholder: "Search address or label...",
                    autocomplete: "off",
                }
            }
            div { class: "flex flex-wrap items-center gap-2",
                select { class: "h-10 w-[120px] rounded-xl border border-border/30 bg-muted/30 px-3 text-sm", name: "status", aria_label: "Wallet status",
                    option { value: "all", selected: status == "all", "All Status" }
                    option { value: "active", selected: status == "active", "Active" }
                    option { value: "disabled", selected: status == "disabled", "Disabled" }
                }
                select { class: "h-10 w-[130px] rounded-xl border border-border/30 bg-muted/20 px-3 text-sm text-muted-foreground", disabled: true, aria_label: "Wallet platform filter unavailable", title: "Platform filtering is not exposed by the wallet service yet",
                    option { "All Platforms" }
                }
                select { class: "h-10 w-[140px] rounded-xl border border-border/30 bg-muted/20 px-3 text-sm text-muted-foreground", disabled: true, aria_label: "Wallet sort order",
                    option { "Date Created" }
                }
                input { r#type: "hidden", name: "limit", value: query.limit.to_string() }
                input { r#type: "hidden", name: "page", value: "1" }
                button { class: "flex h-10 w-10 items-center justify-center rounded-full border border-border/30 bg-muted/20 text-muted-foreground transition-colors hover:border-[#1fc7d4]/30 hover:text-[#1fc7d4]", r#type: "submit", aria_label: "Apply wallet filters", title: "Apply wallet filters",
                    Icon { name: "arrow-up-down".to_string(), size: Some(17) }
                }
            }
        }
    }
}

#[component]
fn WalletSectionHeader(total: Option<i64>) -> Element {
    rsx! {
        section { class: "overflow-hidden rounded-2xl border border-border/20 bg-card shadow-xl",
            div { class: "h-[3px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]", aria_hidden: "true" }
            div { class: "flex items-center justify-between px-5 py-4",
                div { class: "flex items-center gap-3",
                    span { class: "h-[3px] w-8 rounded-full bg-[#1fc7d4]", aria_hidden: "true" }
                    span { class: "text-[#1fc7d4]", aria_hidden: "true",
                        Icon { name: "wallet".to_string(), size: Some(20) }
                    }
                    h2 { class: "text-xl font-bold text-foreground", "Wallets" }
                }
                if let Some(total) = total {
                    span { class: "rounded-full border border-[#1fc7d4]/20 bg-[#1fc7d4]/10 px-2.5 py-1 text-xs font-bold text-[#1fc7d4]",
                        "{total} "
                        if total == 1 { "wallet" } else { "wallets" }
                    }
                }
            }
        }
    }
}

#[component]
fn WalletListReady(projection: AdminWalletListProjection, query: AdminWalletListQuery) -> Element {
    let pagination = projection.clone();
    rsx! {
        div { class: "space-y-3", "data-admin-wallet-list-state": ADMIN_WALLET_LIST_READY,
            section {
                class: "overflow-hidden rounded-2xl border border-border/30 bg-card shadow-xl",
                div { class: "h-[3px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]", aria_hidden: "true" }
                div { class: "flex items-center justify-between border-b border-border/20 px-4 py-3",
                    h2 { class: "text-xs font-bold uppercase tracking-[0.2em] text-[#1fc7d4]", "Wallets" }
                    span { class: "rounded-full border border-border/20 bg-muted/20 px-2.5 py-1 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground", "{projection.total} records" }
                }
                div { class: "overflow-x-auto",
                    table { class: "w-full min-w-[820px] text-left", aria_label: "Wallet inventory",
                        thead { class: "border-b border-border/20 bg-muted/10",
                            tr {
                                th { class: "px-5 py-3 text-[10px] font-bold uppercase tracking-[0.2em] text-muted-foreground", scope: "col", "Wallet" }
                                th { class: "px-4 py-3 text-[10px] font-bold uppercase tracking-[0.2em] text-muted-foreground", scope: "col", "Role" }
                                th { class: "px-4 py-3 text-[10px] font-bold uppercase tracking-[0.2em] text-muted-foreground", scope: "col", "Network" }
                                th { class: "px-4 py-3 text-[10px] font-bold uppercase tracking-[0.2em] text-muted-foreground", scope: "col", "Status" }
                                th { class: "px-4 py-3 text-[10px] font-bold uppercase tracking-[0.2em] text-muted-foreground", scope: "col", "Revision" }
                                th { class: "px-5 py-3 text-right text-[10px] font-bold uppercase tracking-[0.2em] text-muted-foreground", scope: "col", "Actions" }
                            }
                        }
                        tbody { class: "divide-y divide-border/20",
                            for wallet in projection.items { WalletListRow { wallet } }
                        }
                    }
                }
            }
            WalletPagination { projection: pagination, query }
        }
    }
}

#[component]
fn WalletPagination(projection: AdminWalletListProjection, query: AdminWalletListQuery) -> Element {
    let page = projection.offset / projection.limit + 1;
    let total_pages = ((projection.total + projection.limit - 1) / projection.limit).max(1);
    let start = if projection.total == 0 {
        0
    } else {
        projection.offset + 1
    };
    let end = (projection.offset + projection.items.len() as i64).min(projection.total);
    let previous = (page > 1).then(|| wallet_query_href(&query, page - 1));
    let next = (page < total_pages).then(|| wallet_query_href(&query, page + 1));

    rsx! {
        nav { class: "flex flex-col items-start justify-between gap-3 border-t border-border/30 pt-3 sm:flex-row sm:items-center", aria_label: "Wallet pagination",
            p { class: "text-xs text-muted-foreground", "{start}-{end} of {projection.total}" }
            div { class: "flex items-center gap-2",
                if let Some(href) = previous {
                    a { class: "btn btn-sm btn-outline", href, aria_label: "Previous wallet page",
                        Icon { name: "arrow-left".to_string(), size: Some(14) }
                    }
                } else {
                    span { class: "btn btn-sm btn-outline pointer-events-none opacity-40", aria_hidden: "true",
                        Icon { name: "arrow-left".to_string(), size: Some(14) }
                    }
                }
                span { class: "px-1.5 text-xs text-muted-foreground", "{page} / {total_pages}" }
                if let Some(href) = next {
                    a { class: "btn btn-sm btn-outline", href, aria_label: "Next wallet page",
                        Icon { name: "arrow-right".to_string(), size: Some(14) }
                    }
                } else {
                    span { class: "btn btn-sm btn-outline pointer-events-none opacity-40", aria_hidden: "true",
                        Icon { name: "arrow-right".to_string(), size: Some(14) }
                    }
                }
            }
        }
    }
}

fn wallet_query_href(query: &AdminWalletListQuery, page: i64) -> String {
    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    if let Some(search) = query.search.as_deref() {
        serializer.append_pair("search", search);
    }
    if let Some(status) = query.status.as_deref() {
        serializer.append_pair("status", status);
    }
    serializer.append_pair("limit", &query.limit.to_string());
    serializer.append_pair("page", &page.to_string());
    format!("{WALLETS_PATH}?{}", serializer.finish())
}

#[component]
fn WalletListRow(wallet: AdminWalletListItemProjection) -> Element {
    let detail_href = format!(
        "/wallet-management/{}",
        encode_path_segment(&wallet.address)
    );
    let is_active = wallet.status == "active";
    let status = if is_active { "Active" } else { "Disabled" };
    let status_class = if is_active {
        "border-emerald-500/25 bg-emerald-500/10 text-emerald-400"
    } else {
        "border-rose-500/25 bg-rose-500/10 text-rose-400"
    };
    let short_address = format!(
        "{}...{}",
        wallet.address.chars().take(6).collect::<String>(),
        wallet
            .address
            .chars()
            .rev()
            .take(4)
            .collect::<String>()
            .chars()
            .rev()
            .collect::<String>()
    );
    let avatar = wallet
        .address
        .chars()
        .skip(2)
        .take(2)
        .collect::<String>()
        .to_uppercase();
    let label = wallet.label.clone();
    let role = wallet
        .role
        .clone()
        .unwrap_or_else(|| "Unassigned".to_string());
    rsx! {
        tr { class: "transition-colors hover:bg-muted/10",
            td { class: "px-5 py-4",
                div { class: "flex items-center gap-3",
                    div { class: "relative flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl bg-gradient-to-br from-[#1fc7d4] to-[#7645d9] text-xs font-black text-white shadow-lg shadow-cyan-500/10", "{avatar}" }
                    div { class: "min-w-0",
                        p { class: "font-mono text-sm font-bold text-foreground", title: "{wallet.address}", "{short_address}" }
                        if let Some(label) = label { p { class: "mt-0.5 max-w-[220px] truncate text-xs text-muted-foreground", "{label}" } }
                    }
                }
            }
            td { class: "px-4 py-4 text-sm text-muted-foreground", "{role}" }
            td { class: "px-4 py-4",
                span { class: "inline-flex items-center gap-1.5 text-sm text-muted-foreground",
                    Icon { name: "link".to_string(), size: Some(14) }
                    "Chain {wallet.chain_id}"
                }
            }
            td { class: "px-4 py-4",
                span { class: "inline-flex rounded-full border px-2.5 py-1 text-xs font-semibold {status_class}", "{status}" }
            }
            td { class: "px-4 py-4 font-mono text-sm text-muted-foreground", "v{wallet.version}" }
            td { class: "px-5 py-4 text-right",
                a { class: "inline-flex items-center gap-1.5 rounded-xl border border-[#7645d9]/30 px-3 py-1.5 text-sm font-semibold text-purple-400 transition-colors hover:bg-purple-500/10", href: detail_href,
                    "View"
                    Icon { name: "chevron-right".to_string(), size: Some(14) }
                }
            }
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
            "re-enable wallet",
            "grant access",
            "<textarea",
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
        assert!(rendered.contains("aria-label=\"Wallet inventory\""));
        assert!(rendered.contains("Search address or label..."));
        assert!(rendered.contains("aria-label=\"Apply wallet filters\""));
        assert!(!rendered.contains(">10 rows<"));
        assert!(!rendered.contains("secret"));
        assert!(rendered.contains("method=\"get\""));
        assert!(!rendered.contains("method=\"post\""));
    }

    #[test]
    fn wallet_list_query_is_closed_bounded_and_backend_pageable() {
        let query = AdminWalletListQuery::from_raw("search=0x1111&status=disabled&limit=25&page=3")
            .expect("valid wallet-list query");
        assert_eq!(query.search.as_deref(), Some("0x1111"));
        assert_eq!(query.status.as_deref(), Some("disabled"));
        assert_eq!(query.limit, 25);
        assert_eq!(query.page, 3);
        assert_eq!(query.offset(), Some(50));
        assert!(AdminWalletListQuery::from_raw("status=all&limit=10&page=1").is_ok());

        for invalid in [
            "unknown=1",
            "status=active&status=disabled",
            "status=pending",
            "limit=100",
            "page=0",
            "page=1000002&limit=10",
            "search=wallet%20with%20spaces",
        ] {
            assert!(
                AdminWalletListQuery::from_raw(invalid).is_err(),
                "accepted {invalid}"
            );
        }
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
        assert!(rendered.contains("Wallet Management Hub"));
        assert_eq!(rendered.matches(">0<").count(), 7, "{rendered}");
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
    fn stats_unauthenticated_and_unauthorized_render_shared_banner_without_totals() {
        for state in [
            ADMIN_WALLET_STATS_UNAUTHENTICATED,
            ADMIN_WALLET_STATS_UNAUTHORIZED,
        ] {
            let ctx = ctx_with_stats(state, Some(stats_json(777, 700, 77, 7)));
            let rendered = html(render(&ctx).1);

            assert!(rendered.contains(&format!("data-admin-data-state=\"{state}\"")));
            assert!(rendered.contains("Sign in"));
            assert!(!rendered.contains(">777<"));
            assert!(!rendered.contains("data-admin-wallets-state"));
            assert_no_samples_or_controls(&rendered);
        }
    }

    #[test]
    fn list_unauthenticated_and_unauthorized_render_shared_banner_without_rows() {
        for state in [
            ADMIN_WALLET_LIST_UNAUTHENTICATED,
            ADMIN_WALLET_LIST_UNAUTHORIZED,
        ] {
            let mut ctx = authenticated_ctx();
            ctx.params
                .insert(ADMIN_WALLET_LIST_STATE_PARAM.to_string(), state.to_string());
            let rendered = html(render(&ctx).1);

            assert!(rendered.contains(&format!("data-admin-data-state=\"{state}\"")));
            assert!(rendered.contains("Sign in"));
            assert!(!rendered.contains(TEST_ADDRESS));
            assert_no_samples_or_controls(&rendered);
        }
    }

    #[test]
    fn detail_unauthenticated_and_unauthorized_render_shared_banner() {
        for state in [
            ADMIN_WALLET_DETAIL_UNAUTHENTICATED,
            ADMIN_WALLET_DETAIL_UNAUTHORIZED,
        ] {
            let ctx =
                ctx_with_wallet_detail(state, TEST_ADDRESS, Some(wallet_detail_json(TEST_ADDRESS)));
            let rendered = html(render_detail(&ctx).1);

            assert!(rendered.contains(&format!("data-admin-data-state=\"{state}\"")));
            assert!(!rendered.contains("Read-only wallet"));
            assert_no_samples_or_controls(&rendered);
        }
    }

    #[test]
    fn disable_unauthenticated_and_unauthorized_render_shared_banner() {
        for state in [
            ADMIN_WALLET_DISABLE_UNAUTHENTICATED,
            ADMIN_WALLET_DISABLE_UNAUTHORIZED,
        ] {
            let mut ctx = authenticated_ctx();
            ctx.path = format!("/wallet-management/wallets/{TEST_ADDRESS}/disable");
            ctx.params
                .insert("address".to_string(), TEST_ADDRESS.to_string());
            ctx.params.insert(
                ADMIN_WALLET_DISABLE_STATE_PARAM.to_string(),
                state.to_string(),
            );
            let rendered = html(render_disable(&ctx).1);

            assert!(rendered.contains(&format!("data-admin-data-state=\"{state}\"")));
            assert!(!rendered.contains("name=\"reason\""));
            assert_no_samples_or_controls(&rendered);
        }
    }

    #[test]
    fn detail_panels_surface_new_access_and_plan_auth_states() {
        let mut ctx = authenticated_ctx();
        ctx.path = format!("/wallet-management/{TEST_ADDRESS}");
        ctx.params
            .insert("address".to_string(), TEST_ADDRESS.to_string());
        ctx.params.insert(
            ADMIN_ACCESS_STATE_PARAM.to_string(),
            ADMIN_ACCESS_UNAUTHENTICATED.to_string(),
        );
        ctx.params.insert(
            ADMIN_PLANS_STATE_PARAM.to_string(),
            ADMIN_PLANS_UNAUTHORIZED.to_string(),
        );
        let rendered = html(render_detail(&ctx).1);

        assert!(rendered.contains("data-admin-data-state=\"unauthenticated\""));
        assert!(rendered.contains("data-admin-data-state=\"unauthorized\""));
        assert_no_samples_or_controls(&rendered);
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
        assert!(detail.contains("Wallet Management Hub"));
        assert!(detail.contains("1,234"));
        assert_no_samples_or_controls(&detail);

        let disable = html(render_disable(&ctx).1);
        assert!(disable.contains("data-admin-wallet-disable-state=\"unavailable\""));
        assert!(disable.contains("Wallet Management Hub"));
        assert!(disable.contains("1,234"));
        assert!(disable.contains("Action Summary"));
        assert_no_samples_or_controls(&disable);
    }

    #[test]
    fn wallet_detail_projection_is_strict_redacted_and_route_bound() {
        let mut ctx = ctx_with_wallet_detail(
            ADMIN_WALLET_DETAIL_READY,
            TEST_ADDRESS,
            Some(wallet_detail_json(TEST_ADDRESS)),
        );
        ctx.params.insert(
            ADMIN_ACCESS_STATE_PARAM.to_string(),
            ADMIN_ACCESS_READY.to_string(),
        );
        ctx.params.insert(
            ADMIN_ACCESS_DATA_PARAM.to_string(),
            serde_json::json!({
                "items": [{
                    "wallet_address": TEST_ADDRESS,
                    "plan_id": "00000000-0000-0000-0000-000000000001",
                    "plan_name": "Professional",
                    "permission": "epsx:analytics:read",
                    "expires_at": null,
                    "version": 2
                }]
            })
            .to_string(),
        );
        ctx.params.insert(
            ADMIN_PLANS_STATE_PARAM.to_string(),
            ADMIN_PLANS_READY.to_string(),
        );
        ctx.params.insert(
            ADMIN_PLANS_DATA_PARAM.to_string(),
            serde_json::json!({
                "items": [{
                    "id": "00000000-0000-0000-0000-000000000001",
                    "name": "Professional",
                    "description": "Production analytics plan",
                    "amount": "2900",
                    "currency": "USD",
                    "chain_id": "56",
                    "interval": 30,
                    "active": true,
                    "version": 4
                }],
                "total": 1,
                "limit": 100,
                "offset": 0
            })
            .to_string(),
        );
        let rendered = html(render_detail(&ctx).1);
        assert!(rendered.contains("data-admin-wallet-detail-state=\"ready\""));
        assert!(rendered.contains(TEST_ADDRESS));
        assert!(rendered.contains("Read-only wallet"));
        assert!(rendered.contains("Wallet Management Hub"));
        assert!(rendered.contains("Wallet Identity"));
        assert!(rendered.contains("Access Management"));
        assert!(rendered.contains("Available Plans"));
        assert!(rendered.contains("Active Subscription"));
        assert!(rendered.contains("Assigned Plans"));
        assert!(rendered.contains("Update Details"));
        assert!(rendered.contains("Disable wallet"));
        assert!(rendered.contains("epsx:analytics:read"));
        assert!(rendered.contains("value=\"access_assign\""));
        assert!(rendered.contains("value=\"access_revoke\""));
        assert!(rendered.contains("name=\"idempotency_key\""));
        assert!(!rendered.contains("secret"));
        assert!(rendered.contains("<form"));
        assert!(rendered.contains("<button"));

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
        assert!(rendered.contains("Disable Wallet"));
        assert!(rendered.contains("Disable Duration"));
        assert!(rendered.contains("Action Summary"));
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
            assert!(!rendered.contains("data-admin-sidebar"));
            assert_no_samples_or_controls(&rendered);
        }
    }
}
