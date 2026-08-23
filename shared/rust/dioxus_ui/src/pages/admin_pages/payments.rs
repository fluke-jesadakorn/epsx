//! Backend-authoritative admin payment inventory and lifecycle controls.
//!
//! Payment, permission, plan, and financial policy remains owned by the Rust
//! services. This page renders only canonical admin pay-intent and payment-link
//! responses supplied by the admin BFF. Lifecycle controls are native forms
//! whose policy, validation, optimistic versions, and durable effects remain
//! owned by the pay service.

use chrono::DateTime;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use super::super::{PageContext, PageMeta};
use crate::auth::AuthGate;
use crate::components::admin::data_state_banner::{AdminDataState, AdminDataStateBanner};
use crate::components::admin::page_layout::{PageGradient, PageHeader, PageLayout, PageMaxWidth};
use crate::primitives::Icon;

pub const ADMIN_PAYMENTS_DATA_PARAM: &str = "data_admin_payment_intents";
pub const ADMIN_PAYMENTS_STATE_PARAM: &str = "data_admin_payment_intents_state";
pub const ADMIN_PAYMENTS_TAB_PARAM: &str = "admin_payment_intents_tab";
pub const ADMIN_PAYMENTS_PAYER_PARAM: &str = "admin_payment_intents_payer";
pub const ADMIN_PAYMENTS_STATUS_PARAM: &str = "admin_payment_intents_status";
pub const ADMIN_PAYMENTS_LIMIT_PARAM: &str = "admin_payment_intents_limit";
pub const ADMIN_PAYMENTS_OFFSET_PARAM: &str = "admin_payment_intents_offset";

pub const ADMIN_PAYMENTS_READY: &str = "ready";
pub const ADMIN_PAYMENTS_EMPTY: &str = "empty";
pub const ADMIN_PAYMENTS_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_PAYMENTS_MALFORMED: &str = "malformed";
pub const ADMIN_PAYMENTS_UNAUTHENTICATED: &str = "unauthenticated";
pub const ADMIN_PAYMENTS_UNAUTHORIZED: &str = "unauthorized";

pub const ADMIN_PAYMENT_LINKS_DATA_PARAM: &str = "data_admin_payment_links";
pub const ADMIN_PAYMENT_LINKS_STATE_PARAM: &str = "data_admin_payment_links_state";

pub const ADMIN_PAYMENT_LINKS_READY: &str = "ready";
pub const ADMIN_PAYMENT_LINKS_EMPTY: &str = "empty";
pub const ADMIN_PAYMENT_LINKS_FORBIDDEN: &str = "forbidden";
pub const ADMIN_PAYMENT_LINKS_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_PAYMENT_LINKS_MALFORMED: &str = "malformed";
pub const ADMIN_PAYMENT_LINKS_UNAUTHENTICATED: &str = "unauthenticated";
pub const ADMIN_PAYMENT_LINKS_UNAUTHORIZED: &str = "unauthorized";
pub const ADMIN_PAYMENT_MUTATION_PARAM: &str = "mutation";

pub const ADMIN_PAYMENT_USER_ACCESS_DATA_PARAM: &str = "data_admin_payment_user_access";
pub const ADMIN_PAYMENT_USER_ACCESS_STATE_PARAM: &str = "data_admin_payment_user_access_state";
pub const ADMIN_PAYMENT_USER_ACCESS_READY: &str = "ready";
pub const ADMIN_PAYMENT_USER_ACCESS_EMPTY: &str = "empty";
pub const ADMIN_PAYMENT_USER_ACCESS_FORBIDDEN: &str = "forbidden";
pub const ADMIN_PAYMENT_USER_ACCESS_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_PAYMENT_USER_ACCESS_MALFORMED: &str = "malformed";
pub const ADMIN_PAYMENT_USER_ACCESS_UNAUTHENTICATED: &str = "unauthenticated";
pub const ADMIN_PAYMENT_USER_ACCESS_UNAUTHORIZED: &str = "unauthorized";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdminPaymentUserAccessQuery {
    pub page: i64,
    pub limit: i64,
    pub status: Option<String>,
    pub search: Option<String>,
}

impl AdminPaymentUserAccessQuery {
    #[allow(clippy::result_unit_err)]
    pub fn from_raw(raw: &str) -> Result<Self, ()> {
        let mut query = Self {
            page: 1,
            limit: 20,
            status: None,
            search: None,
        };
        let mut seen = std::collections::HashSet::new();
        for (key, value) in url::form_urlencoded::parse(raw.as_bytes()) {
            if key == "tab" {
                continue;
            }
            if !seen.insert(key.to_string()) {
                return Err(());
            }
            match key.as_ref() {
                "page" => {
                    query.page = value.parse().map_err(|_| ())?;
                    if !(1..=500_001).contains(&query.page) {
                        return Err(());
                    }
                }
                "limit" => {
                    query.limit = value.parse().map_err(|_| ())?;
                    if !matches!(query.limit, 10 | 20 | 50 | 100) {
                        return Err(());
                    }
                }
                "status" => match value.as_ref() {
                    "" | "all" => query.status = None,
                    "active" | "expired" | "expiring_soon" | "no_plan" => {
                        query.status = Some(value.into_owned())
                    }
                    _ => return Err(()),
                },
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
                _ => return Err(()),
            }
        }
        Ok(query)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminPaymentUserAccessItem {
    pub wallet_address: String,
    pub current_plan_id: Option<String>,
    pub plan_name: Option<String>,
    pub plan_expires_at: Option<String>,
    pub days_remaining: i64,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminPaymentUserAccessProjection {
    pub items: Vec<AdminPaymentUserAccessItem>,
    pub page: i64,
    pub limit: i64,
    pub total_pages: i64,
}

pub fn decode_admin_payment_user_access_projection(
    value: serde_json::Value,
) -> Option<AdminPaymentUserAccessProjection> {
    let projection: AdminPaymentUserAccessProjection = serde_json::from_value(value).ok()?;
    if projection.page < 1
        || !(1..=100).contains(&projection.limit)
        || projection.total_pages < 1
        || projection.items.len() > usize::try_from(projection.limit).ok()?
        || projection.items.iter().any(|item| {
            !valid_wallet_address(&item.wallet_address)
                || item
                    .current_plan_id
                    .as_deref()
                    .is_some_and(|value| uuid::Uuid::parse_str(value).is_err())
                || item.plan_name.as_deref().is_some_and(|value| {
                    value.trim().is_empty()
                        || value.trim() != value
                        || value.chars().count() > 100
                        || value.chars().any(char::is_control)
                })
                || item.plan_expires_at.as_deref().is_some_and(|value| {
                    value.len() > 64 || DateTime::parse_from_rfc3339(value).is_err()
                })
                || !(0..=365_000).contains(&item.days_remaining)
                || !matches!(
                    item.status.as_str(),
                    "active" | "expiring_soon" | "expired" | "no_plan"
                )
        })
    {
        return None;
    }
    Some(projection)
}

fn valid_wallet_address(value: &str) -> bool {
    value.len() == 42
        && value.starts_with("0x")
        && value[2..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// Exact service-owned fields returned by `GET /api/v1/admin/pay/intents`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminPaymentIntent {
    pub id: String,
    pub chain_id: String,
    pub payer: String,
    pub payee: String,
    pub amount: String,
    pub token_address: String,
    pub status: String,
    pub escrow_id: Option<String>,
    pub tx_hash: Option<String>,
    pub description: Option<String>,
    pub expires_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminPaymentIntentList {
    pub items: Vec<AdminPaymentIntent>,
    pub total: i64,
}

/// Decode and semantically validate the service boundary before any value is
/// presented as an authoritative payment record.
pub fn decode_admin_payment_intent_list(
    value: serde_json::Value,
) -> Option<AdminPaymentIntentList> {
    let payload: AdminPaymentIntentList = serde_json::from_value(value).ok()?;
    let total = usize::try_from(payload.total).ok()?;
    if total < payload.items.len() || payload.items.iter().any(|item| !item.is_well_formed()) {
        return None;
    }
    Some(payload)
}

impl AdminPaymentIntent {
    fn is_well_formed(&self) -> bool {
        [
            self.id.as_str(),
            self.chain_id.as_str(),
            self.payer.as_str(),
            self.payee.as_str(),
            self.amount.as_str(),
            self.token_address.as_str(),
            self.status.as_str(),
            self.created_at.as_str(),
            self.updated_at.as_str(),
        ]
        .iter()
        .all(|value| !value.trim().is_empty())
    }
}

/// Safe fields from the backend AdminPayLink DTO needed to identify a link and
/// submit an optimistic disable transition. Intent identity and audit details
/// remain service-owned and are not carried into PageContext.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminPaymentLinkProjection {
    pub id: String,
    pub slug: String,
    pub max_uses: i32,
    pub current_uses: i32,
    pub expires_at: Option<String>,
    pub status: String,
    pub version: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminPaymentLinkListProjection {
    pub items: Vec<AdminPaymentLinkProjection>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

pub fn decode_admin_payment_link_projection(
    value: serde_json::Value,
) -> Option<AdminPaymentLinkProjection> {
    let projection: AdminPaymentLinkProjection = serde_json::from_value(value).ok()?;
    projection.is_well_formed().then_some(projection)
}

pub fn decode_admin_payment_link_list_projection(
    value: serde_json::Value,
) -> Option<AdminPaymentLinkListProjection> {
    let projection: AdminPaymentLinkListProjection = serde_json::from_value(value).ok()?;
    if !(1..=100).contains(&projection.limit)
        || !(0..=10_000_000).contains(&projection.offset)
        || projection.total < 0
        || projection.items.len() > usize::try_from(projection.limit).ok()?
        || projection
            .offset
            .checked_add(i64::try_from(projection.items.len()).ok()?)?
            > projection.total
        || projection.items.iter().any(|item| !item.is_well_formed())
    {
        return None;
    }
    Some(projection)
}

impl AdminPaymentLinkProjection {
    fn is_well_formed(&self) -> bool {
        valid_resource_id(&self.id)
            && valid_link_slug(&self.slug)
            && (0..=1_000_000).contains(&self.max_uses)
            && self.current_uses >= 0
            && (self.max_uses == 0 || self.current_uses <= self.max_uses)
            && matches!(self.status.as_str(), "active" | "disabled")
            && self.version >= 0
            && self.expires_at.as_deref().is_none_or(|value| {
                !value.is_empty()
                    && value.len() <= 64
                    && !value.chars().any(char::is_control)
                    && DateTime::parse_from_rfc3339(value).is_ok()
            })
    }
}

fn valid_link_slug(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn valid_resource_id(value: &str) -> bool {
    (1..=66).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() || matches!(byte, b'x' | b'-' | b'_'))
}

fn payment_intent_expected_version(value: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|timestamp| timestamp.timestamp_micros())
        .filter(|version| *version > 0)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PaymentFilters {
    payer: Option<String>,
    status: Option<String>,
    limit: usize,
    offset: usize,
}

impl PaymentFilters {
    fn from_ctx(ctx: &PageContext) -> Self {
        Self {
            payer: ctx
                .params
                .get(ADMIN_PAYMENTS_PAYER_PARAM)
                .and_then(|value| safe_filter(value, 128)),
            status: ctx
                .params
                .get(ADMIN_PAYMENTS_STATUS_PARAM)
                .and_then(|value| safe_filter(value, 32)),
            limit: ctx
                .params
                .get(ADMIN_PAYMENTS_LIMIT_PARAM)
                .and_then(|value| value.parse().ok())
                .unwrap_or(20)
                .clamp(1, 100),
            offset: ctx
                .params
                .get(ADMIN_PAYMENTS_OFFSET_PARAM)
                .and_then(|value| value.parse().ok())
                .unwrap_or(0),
        }
    }

    fn page_url(&self, offset: usize) -> String {
        let mut pairs = vec![
            "tab=payments".to_string(),
            format!("limit={}", self.limit),
            format!("offset={offset}"),
        ];
        if let Some(payer) = &self.payer {
            pairs.push(format!("payer={payer}"));
        }
        if let Some(status) = &self.status {
            pairs.push(format!("status={status}"));
        }
        format!("/payments?{}", pairs.join("&"))
    }
}

fn safe_filter(value: &str, max_len: usize) -> Option<String> {
    (!value.is_empty()
        && value.len() <= max_len
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'.' | b'_' | b'-')))
    .then(|| value.to_string())
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PaymentLoad {
    Ready(AdminPaymentIntentList),
    Empty,
    Unauthenticated,
    Unauthorized,
    Unavailable,
    Malformed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PaymentLinksLoad {
    Ready(AdminPaymentLinkListProjection),
    Empty,
    Unauthenticated,
    Unauthorized,
    Forbidden,
    Unavailable,
    Malformed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PaymentUserAccessLoad {
    Ready(AdminPaymentUserAccessProjection),
    Empty,
    Unauthenticated,
    Unauthorized,
    Forbidden,
    Unavailable,
    Malformed,
}

fn payment_user_access_load(ctx: &PageContext) -> PaymentUserAccessLoad {
    let state = ctx
        .params
        .get(ADMIN_PAYMENT_USER_ACCESS_STATE_PARAM)
        .map(String::as_str);
    match state {
        Some(ADMIN_PAYMENT_USER_ACCESS_READY) | Some(ADMIN_PAYMENT_USER_ACCESS_EMPTY) => {
            let Some(raw) = ctx.params.get(ADMIN_PAYMENT_USER_ACCESS_DATA_PARAM) else {
                return PaymentUserAccessLoad::Malformed;
            };
            let Some(projection) = serde_json::from_str(raw)
                .ok()
                .and_then(decode_admin_payment_user_access_projection)
            else {
                return PaymentUserAccessLoad::Malformed;
            };
            match (state, projection.items.is_empty()) {
                (Some(ADMIN_PAYMENT_USER_ACCESS_READY), false) => {
                    PaymentUserAccessLoad::Ready(projection)
                }
                (Some(ADMIN_PAYMENT_USER_ACCESS_EMPTY), true) => PaymentUserAccessLoad::Empty,
                _ => PaymentUserAccessLoad::Malformed,
            }
        }
        Some(ADMIN_PAYMENT_USER_ACCESS_FORBIDDEN) => PaymentUserAccessLoad::Forbidden,
        Some(ADMIN_PAYMENT_USER_ACCESS_MALFORMED) => PaymentUserAccessLoad::Malformed,
        Some(ADMIN_PAYMENT_USER_ACCESS_UNAUTHENTICATED) => PaymentUserAccessLoad::Unauthenticated,
        Some(ADMIN_PAYMENT_USER_ACCESS_UNAUTHORIZED) => PaymentUserAccessLoad::Unauthorized,
        Some(ADMIN_PAYMENT_USER_ACCESS_UNAVAILABLE) | None => PaymentUserAccessLoad::Unavailable,
        Some(_) => PaymentUserAccessLoad::Malformed,
    }
}

fn payment_links_load(ctx: &PageContext) -> PaymentLinksLoad {
    match ctx
        .params
        .get(ADMIN_PAYMENT_LINKS_STATE_PARAM)
        .map(String::as_str)
    {
        Some(ADMIN_PAYMENT_LINKS_READY) | Some(ADMIN_PAYMENT_LINKS_EMPTY) => {
            let Some(raw) = ctx.params.get(ADMIN_PAYMENT_LINKS_DATA_PARAM) else {
                return PaymentLinksLoad::Malformed;
            };
            let Some(projection) = serde_json::from_str(raw)
                .ok()
                .and_then(decode_admin_payment_link_list_projection)
            else {
                return PaymentLinksLoad::Malformed;
            };
            match (
                ctx.params
                    .get(ADMIN_PAYMENT_LINKS_STATE_PARAM)
                    .map(String::as_str),
                projection.items.is_empty(),
                projection.total,
            ) {
                (Some(ADMIN_PAYMENT_LINKS_READY), false, _) => PaymentLinksLoad::Ready(projection),
                (Some(ADMIN_PAYMENT_LINKS_READY), true, total) if total > 0 => {
                    PaymentLinksLoad::Ready(projection)
                }
                (Some(ADMIN_PAYMENT_LINKS_EMPTY), true, 0) => PaymentLinksLoad::Empty,
                _ => PaymentLinksLoad::Malformed,
            }
        }
        Some(ADMIN_PAYMENT_LINKS_FORBIDDEN) => PaymentLinksLoad::Forbidden,
        Some(ADMIN_PAYMENT_LINKS_MALFORMED) => PaymentLinksLoad::Malformed,
        Some(ADMIN_PAYMENT_LINKS_UNAUTHENTICATED) => PaymentLinksLoad::Unauthenticated,
        Some(ADMIN_PAYMENT_LINKS_UNAUTHORIZED) => PaymentLinksLoad::Unauthorized,
        Some(ADMIN_PAYMENT_LINKS_UNAVAILABLE) | None => PaymentLinksLoad::Unavailable,
        Some(_) => PaymentLinksLoad::Malformed,
    }
}

fn payment_load(ctx: &PageContext) -> PaymentLoad {
    let state = ctx
        .params
        .get(ADMIN_PAYMENTS_STATE_PARAM)
        .map(String::as_str);
    match state {
        Some(ADMIN_PAYMENTS_READY) | Some(ADMIN_PAYMENTS_EMPTY) => {
            let Some(raw) = ctx.params.get(ADMIN_PAYMENTS_DATA_PARAM) else {
                return PaymentLoad::Malformed;
            };
            let Some(payload) = serde_json::from_str(raw)
                .ok()
                .and_then(decode_admin_payment_intent_list)
            else {
                return PaymentLoad::Malformed;
            };
            match (state, payload.items.is_empty(), payload.total) {
                (Some(ADMIN_PAYMENTS_READY), false, _) => PaymentLoad::Ready(payload),
                (Some(ADMIN_PAYMENTS_READY), true, total) if total > 0 => {
                    PaymentLoad::Ready(payload)
                }
                (Some(ADMIN_PAYMENTS_EMPTY), true, 0) => PaymentLoad::Empty,
                _ => PaymentLoad::Malformed,
            }
        }
        Some(ADMIN_PAYMENTS_MALFORMED) => PaymentLoad::Malformed,
        Some(ADMIN_PAYMENTS_UNAUTHENTICATED) => PaymentLoad::Unauthenticated,
        Some(ADMIN_PAYMENTS_UNAUTHORIZED) => PaymentLoad::Unauthorized,
        Some(ADMIN_PAYMENTS_UNAVAILABLE) | None => PaymentLoad::Unavailable,
        Some(_) => PaymentLoad::Malformed,
    }
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Payments");
    let active_tab = ctx
        .params
        .get(ADMIN_PAYMENTS_TAB_PARAM)
        .map(String::as_str)
        .unwrap_or("payments");
    let required_permissions = match active_tab {
        "payment-links" => Some(vec!["admin:payment-links:view".to_string()]),
        // The subscription backend is the authority for this tab. Do not
        // duplicate its plan-access decision in the UI gate.
        "user-access" => None,
        _ => Some(vec!["admin:payments:view".to_string()]),
    };
    (
        meta,
        rsx! {
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("payment intent visibility".to_string()),
                required_permissions,
                return_url: Some(ctx.path.clone()),
                RenderPaymentsHub { ctx: ctx.clone() }
            }
        },
    )
}

#[component]
fn RenderPaymentsHub(ctx: PageContext) -> Element {
    let active_tab = ctx
        .params
        .get(ADMIN_PAYMENTS_TAB_PARAM)
        .map(String::as_str)
        .unwrap_or("payments");
    let active_tab = match active_tab {
        "user-access" => "user-access",
        "payment-links" => "payment-links",
        _ => "payments",
    };
    let filters = PaymentFilters::from_ctx(&ctx);
    let mutation = match ctx
        .params
        .get(ADMIN_PAYMENT_MUTATION_PARAM)
        .map(String::as_str)
    {
        Some("success") | Some("conflict") | Some("forbidden") | Some("unavailable")
        | Some("malformed") => ctx.params.get(ADMIN_PAYMENT_MUTATION_PARAM).cloned(),
        _ => None,
    };

    rsx! {
        PageLayout {
            max_width: Some(PageMaxWidth::SevenXl),
            PageHeader {
                title: "Payments Hub".to_string(),
                subtitle: Some("Manage payments, user access, and payment links".to_string()),
                icon: Some("credit-card".to_string()),
                gradient: Some(PageGradient::Primary),
                centered: Some(true),
                extra_actions: None,
                class_name: None,
            }
            if let Some(state) = mutation.clone() {
                p { class: "mb-5 rounded-xl border border-amber-500/30 bg-amber-500/5 px-4 py-3 text-sm", role: if state == "forbidden" { "alert" } else { "status" },
                    "data-admin-payment-mutation-state": state,
                    "Payment mutation: {state}"
                }
            }
            if active_tab == "payments" {
                PaymentsTab { load: payment_load(&ctx), filters }
            } else if active_tab == "user-access" {
                PaymentUserAccessTab {
                    load: payment_user_access_load(&ctx),
                    query: AdminPaymentUserAccessQuery::from_raw(&ctx.query).unwrap_or(AdminPaymentUserAccessQuery {
                        page: 1,
                        limit: 20,
                        status: None,
                        search: None,
                    }),
                }
            } else {
                PaymentLinksTab { load: payment_links_load(&ctx) }
            }
        }
    }
}

#[component]
fn PaymentsTab(load: PaymentLoad, filters: PaymentFilters) -> Element {
    rsx! {
        div { class: "space-y-6 sm:space-y-8",
            PaymentsActionBar { refresh_url: filters.page_url(filters.offset) }
            PaymentsSummaryGrid {}
            PaymentFilterForm { filters: filters.clone() }
            match load {
                PaymentLoad::Ready(payload) => rsx! {
                    PaymentIntentList { payload, filters: filters.clone() }
                },
                PaymentLoad::Empty => rsx! {
                    section { class: "rounded-2xl border border-border/30 bg-card p-10 text-center", role: "status",
                        h2 { class: "text-lg font-semibold", "No payment intents found" }
                        p { class: "mt-2 text-sm text-muted-foreground", "No authoritative payment intents match the current filters." }
                    }
                },
                PaymentLoad::Unauthenticated | PaymentLoad::Unauthorized => {
                    let state = if matches!(load, PaymentLoad::Unauthenticated) {
                        AdminDataState::Unauthenticated
                    } else {
                        AdminDataState::Unauthorized
                    };
                    rsx! {
                        AdminDataStateBanner {
                            state,
                            subject: "Payments".to_string(),
                            return_path: "/payments".to_string(),
                            retry_href: "/payments".to_string(),
                        }
                    }
                }
                PaymentLoad::Unavailable => rsx! {
                    LoadProblem {
                        title: "Payment intents unavailable".to_string(),
                        detail: "The payment service could not provide an authoritative response. No records are being shown.".to_string(),
                        retry_url: filters.page_url(filters.offset),
                    }
                },
                PaymentLoad::Malformed => rsx! {
                    LoadProblem {
                        title: "Payment data could not be verified".to_string(),
                        detail: "The service response did not match the payment-intent contract. No records are being shown.".to_string(),
                        retry_url: filters.page_url(filters.offset),
                    }
                },
            }
        }
    }
}

#[component]
fn PaymentsActionBar(refresh_url: String) -> Element {
    rsx! {
        div { class: "flex flex-wrap items-center gap-3", aria_label: "Payment actions",
            a { class: "btn btn-sm bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white", href: refresh_url,
                Icon { name: "refresh-cw".to_string(), size: Some(15) }
                " Refresh"
            }
            button { class: "btn btn-sm btn-outline", r#type: "button", disabled: true, title: "CSV export requires a backend-owned redacted export contract",
                Icon { name: "bar-chart-3".to_string(), size: Some(15) }
                " Export CSV"
            }
        }
    }
}

#[component]
fn PaymentsSummaryGrid() -> Element {
    const CARDS: [(&str, &str, &str); 4] = [
        (
            "Total Revenue",
            "Platform total is not exposed",
            "text-[#1fc7d4]",
        ),
        (
            "Successful",
            "Verified completion summary",
            "text-[#31d0aa]",
        ),
        ("Pending", "In-progress summary", "text-[#ffb237]"),
        ("Today", "Current revenue summary", "text-[#ed4b9e]"),
    ];
    rsx! {
        div { class: "grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4", aria_label: "Payment summary",
            for (title, subtitle, accent) in CARDS {
                article { class: "overflow-hidden rounded-xl border border-border/20 bg-card p-5 shadow-xl",
                    p { class: "text-[10px] font-bold uppercase tracking-[0.2em] text-muted-foreground", "{title}" }
                    p { class: "mt-3 text-2xl font-black tracking-tight {accent}", "Unavailable" }
                    p { class: "mt-1 text-xs text-muted-foreground", "{subtitle}" }
                }
            }
        }
    }
}

#[component]
fn PaymentUserAccessTab(
    load: PaymentUserAccessLoad,
    query: AdminPaymentUserAccessQuery,
) -> Element {
    let refresh_url = payment_user_access_url(&query, query.page);
    rsx! {
        div { class: "space-y-6 sm:space-y-8",
            div { class: "flex items-center gap-3",
                a { class: "btn btn-sm bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white", href: refresh_url,
                    Icon { name: "refresh-cw".to_string(), size: Some(14) }
                    " Refresh"
                }
            }
            match load {
                PaymentUserAccessLoad::Ready(projection) => rsx! { PaymentUserAccessReady { projection, query } },
                PaymentUserAccessLoad::Empty => rsx! { PaymentUserAccessEmpty {} },
                PaymentUserAccessLoad::Forbidden => rsx! {
                    LoadProblem {
                        title: "User-access read was denied".to_string(),
                        detail: "The subscription backend did not authorize this session to read plan access.".to_string(),
                        retry_url: payment_user_access_url(&query, query.page),
                    }
                },
                PaymentUserAccessLoad::Unauthenticated | PaymentUserAccessLoad::Unauthorized => {
                    let state = if matches!(load, PaymentUserAccessLoad::Unauthenticated) {
                        AdminDataState::Unauthenticated
                    } else {
                        AdminDataState::Unauthorized
                    };
                    rsx! {
                        AdminDataStateBanner {
                            state,
                            subject: "User access".to_string(),
                            return_path: "/payments".to_string(),
                            retry_href: "/payments".to_string(),
                        }
                    }
                }
                PaymentUserAccessLoad::Unavailable => rsx! {
                    LoadProblem {
                        title: "User access is unavailable".to_string(),
                        detail: "The subscription backend could not provide an authoritative user-access response.".to_string(),
                        retry_url: payment_user_access_url(&query, query.page),
                    }
                },
                PaymentUserAccessLoad::Malformed => rsx! {
                    LoadProblem {
                        title: "User-access data could not be verified".to_string(),
                        detail: "The backend response did not match the strict plan-access contract.".to_string(),
                        retry_url: payment_user_access_url(&query, query.page),
                    }
                },
            }
        }
    }
}

#[component]
fn PaymentUserAccessReady(
    projection: AdminPaymentUserAccessProjection,
    query: AdminPaymentUserAccessQuery,
) -> Element {
    let count = projection.items.len();
    let previous = (projection.page > 1)
        .then(|| payment_user_access_url(&query, projection.page.saturating_sub(1)));
    let next = (projection.items.len() == projection.limit as usize)
        .then(|| payment_user_access_url(&query, projection.page.saturating_add(1)));
    rsx! {
        section { class: "overflow-hidden rounded-2xl border border-border/20 bg-card shadow-xl", "data-admin-payment-user-access-state": ADMIN_PAYMENT_USER_ACCESS_READY,
            div { class: "h-[3px] bg-gradient-to-r from-[#31d0aa] to-[#1fc7d4]", aria_hidden: "true" }
            div { class: "p-4 sm:p-6 lg:p-8",
                div { class: "mb-6 flex flex-col items-start justify-between gap-3 sm:flex-row sm:items-center",
                    h2 { class: "text-xs font-bold uppercase tracking-[0.2em] text-[#31d0aa]", "All Users with Plan Access" }
                    span { class: "rounded-full border border-border/40 bg-muted/50 px-3 py-1 text-xs font-bold text-muted-foreground", "{count} users" }
                }
                div { class: "space-y-4 sm:hidden",
                    for item in projection.items.iter() {
                        PaymentUserAccessCard { item: item.clone() }
                    }
                }
                div { class: "hidden overflow-x-auto sm:block",
                    table { class: "min-w-full",
                        thead {
                            tr { class: "border-b border-border/50",
                                for label in ["Wallet", "Plan", "Status", "Days Left", "Expires", "Actions"] {
                                    th { class: "px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-muted-foreground", "{label}" }
                                }
                            }
                        }
                        tbody { class: "divide-y divide-border/50",
                            for item in projection.items.iter() {
                                PaymentUserAccessRow { item: item.clone() }
                            }
                        }
                    }
                }
                nav { class: "mt-6 flex items-center justify-between", aria_label: "User access pagination",
                    if let Some(href) = previous {
                        a { class: "btn btn-sm btn-outline", href, "Previous" }
                    } else {
                        span { class: "btn btn-sm btn-outline pointer-events-none opacity-40", "Previous" }
                    }
                    span { class: "text-sm text-muted-foreground", "Page {projection.page}" }
                    if let Some(href) = next {
                        a { class: "btn btn-sm btn-outline", href, "Next" }
                    } else {
                        span { class: "btn btn-sm btn-outline pointer-events-none opacity-40", "Next" }
                    }
                }
            }
        }
    }
}

#[component]
fn PaymentUserAccessRow(item: AdminPaymentUserAccessItem) -> Element {
    let detail_url = format!("/wallet-management/{}", item.wallet_address);
    let plan = item.plan_name.unwrap_or_else(|| "No Plan".to_string());
    let expires = item.plan_expires_at.unwrap_or_else(|| "Never".to_string());
    let days = if item.days_remaining > 0 {
        format!("{} days", item.days_remaining)
    } else {
        "-".to_string()
    };
    rsx! {
        tr { class: "transition-colors hover:bg-muted/30",
            td { class: "px-4 py-4 font-mono text-xs text-muted-foreground", "{item.wallet_address}" }
            td { class: "px-4 py-4 text-sm font-semibold text-foreground", "{plan}" }
            td { class: "px-4 py-4", PaymentUserAccessStatus { status: item.status } }
            td { class: "px-4 py-4 text-sm text-secondary", "{days}" }
            td { class: "px-4 py-4 text-sm text-muted-foreground", "{expires}" }
            td { class: "px-4 py-4",
                a { class: "btn btn-sm btn-outline", href: detail_url, "View" }
            }
        }
    }
}

#[component]
fn PaymentUserAccessCard(item: AdminPaymentUserAccessItem) -> Element {
    let detail_url = format!("/wallet-management/{}", item.wallet_address);
    let plan = item.plan_name.unwrap_or_else(|| "No Plan".to_string());
    let expires = item.plan_expires_at.unwrap_or_else(|| "Never".to_string());
    let days = if item.days_remaining > 0 {
        format!("{} days", item.days_remaining)
    } else {
        "-".to_string()
    };
    rsx! {
        article { class: "rounded-2xl border border-border/50 bg-muted/30 p-4",
            div { class: "flex items-center justify-between gap-3",
                p { class: "break-all font-mono text-xs text-muted-foreground", "{item.wallet_address}" }
                PaymentUserAccessStatus { status: item.status }
            }
            dl { class: "mt-3 grid grid-cols-2 gap-3",
                div { class: "rounded-xl border border-border/50 bg-card p-3",
                    dt { class: "text-sm font-medium text-muted-foreground", "Plan" }
                    dd { class: "mt-1 text-lg font-bold text-primary", "{plan}" }
                }
                div { class: "rounded-xl border border-border/50 bg-card p-3",
                    dt { class: "text-sm font-medium text-muted-foreground", "Days Left" }
                    dd { class: "mt-1 text-lg font-bold text-secondary", "{days}" }
                }
            }
            p { class: "mt-3 text-xs text-muted-foreground", "Expires: {expires}" }
            a { class: "btn btn-sm btn-outline mt-3", href: detail_url, "View wallet" }
        }
    }
}

#[component]
fn PaymentUserAccessStatus(status: String) -> Element {
    let class = match status.as_str() {
        "active" => "border-success/20 bg-success/10 text-success",
        "expiring_soon" => "border-warning/20 bg-warning/10 text-warning",
        "expired" => "border-destructive/20 bg-destructive/10 text-destructive",
        _ => "border-border/50 bg-muted text-muted-foreground",
    };
    let label = match status.as_str() {
        "no_plan" => "No Plan",
        "expiring_soon" => "Expiring Soon",
        _ => status.as_str(),
    };
    rsx! {
        span { class: "inline-flex rounded-full border px-2.5 py-1 text-xs font-semibold {class}", "{label}" }
    }
}

#[component]
fn PaymentUserAccessEmpty() -> Element {
    rsx! {
        section { class: "overflow-hidden rounded-2xl border border-border/20 bg-card shadow-xl", role: "status", "data-admin-payment-user-access-state": ADMIN_PAYMENT_USER_ACCESS_EMPTY,
            div { class: "h-[3px] bg-gradient-to-r from-[#31d0aa] to-[#1fc7d4]", aria_hidden: "true" }
            div { class: "px-6 py-16 text-center",
                div { class: "mx-auto flex h-20 w-20 items-center justify-center rounded-2xl bg-primary/10 text-primary",
                    Icon { name: "users".to_string(), size: Some(40) }
                }
                h2 { class: "mt-4 text-xl font-semibold", "No users with plan access found" }
                p { class: "mt-2 text-muted-foreground", "Users with active subscriptions will appear here" }
            }
        }
    }
}

fn payment_user_access_url(query: &AdminPaymentUserAccessQuery, page: i64) -> String {
    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    serializer.append_pair("tab", "user-access");
    serializer.append_pair("page", &page.to_string());
    serializer.append_pair("limit", &query.limit.to_string());
    if let Some(status) = query.status.as_deref() {
        serializer.append_pair("status", status);
    }
    if let Some(search) = query.search.as_deref() {
        serializer.append_pair("search", search);
    }
    format!("/payments?{}", serializer.finish())
}

#[component]
fn PaymentLinksTab(load: PaymentLinksLoad) -> Element {
    let available = matches!(&load, PaymentLinksLoad::Ready(_) | PaymentLinksLoad::Empty);
    rsx! {
        div { class: "space-y-6 sm:space-y-8",
            PaymentLinksActions { available }
            PaymentLinksFilters {}
            if available {
                PaymentLinkCreateForm {}
            }
            match load {
                PaymentLinksLoad::Ready(projection) => rsx! { PaymentLinksReady { projection } },
                PaymentLinksLoad::Empty => rsx! {
                    PaymentLinksEmpty {}
                },
                PaymentLinksLoad::Forbidden => rsx! {
                    PaymentLinksProblem {
                        state: ADMIN_PAYMENT_LINKS_FORBIDDEN,
                        title: "Payment-link access was denied".to_string(),
                        detail: "The backend did not authorize this session to read payment links.".to_string(),
                    }
                },
                PaymentLinksLoad::Unauthenticated | PaymentLinksLoad::Unauthorized => {
                    let state = if matches!(load, PaymentLinksLoad::Unauthenticated) {
                        AdminDataState::Unauthenticated
                    } else {
                        AdminDataState::Unauthorized
                    };
                    rsx! {
                        AdminDataStateBanner {
                            state,
                            subject: "Payment links".to_string(),
                            return_path: "/payments".to_string(),
                            retry_href: "/payments".to_string(),
                        }
                    }
                }
                PaymentLinksLoad::Unavailable => rsx! {
                    PaymentLinksProblem {
                        state: ADMIN_PAYMENT_LINKS_UNAVAILABLE,
                        title: "Payment links are unavailable".to_string(),
                        detail: "The payment backend could not provide an authoritative link response. No links are being shown.".to_string(),
                    }
                },
                PaymentLinksLoad::Malformed => rsx! {
                    PaymentLinksProblem {
                        state: ADMIN_PAYMENT_LINKS_MALFORMED,
                        title: "Payment-link data could not be verified".to_string(),
                        detail: "The backend response did not match the strict redacted payment-link contract. No links are being shown.".to_string(),
                    }
                },
            }
        }
    }
}

#[component]
fn PaymentLinksActions(available: bool) -> Element {
    rsx! {
        div { class: "flex flex-wrap items-center gap-3", aria_label: "Payment-link actions",
            if available {
                a { class: "btn btn-sm bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white", href: "#create-payment-link",
                    Icon { name: "plus".to_string(), size: Some(15) }
                    " New Link"
                }
            } else {
                button { class: "btn btn-sm bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white opacity-50", r#type: "button", disabled: true, title: "Creation is unavailable until the backend returns an authoritative link inventory",
                    Icon { name: "plus".to_string(), size: Some(15) }
                    " New Link"
                }
            }
            a { class: "btn btn-sm btn-outline", href: "/payments?tab=payment-links",
                Icon { name: "refresh-cw".to_string(), size: Some(15) }
                " Refresh"
            }
        }
    }
}

#[component]
fn PaymentLinksFilters() -> Element {
    rsx! {
        section { class: "rounded-xl border border-border/20 bg-card p-4", aria_label: "Payment-link filters",
            div { class: "grid grid-cols-1 gap-4 sm:grid-cols-3",
                label { class: "block text-xs font-bold uppercase tracking-[0.15em] text-muted-foreground",
                    "Context Type"
                    select { class: "input mt-2 w-full", disabled: true, title: "Context type is not exposed by the current payment-link projection",
                        option { "All Types" }
                    }
                }
                label { class: "block text-xs font-bold uppercase tracking-[0.15em] text-muted-foreground",
                    "Status"
                    select { class: "input mt-2 w-full", disabled: true, title: "Server-side status filtering is not exposed by the current payment-link endpoint",
                        option { "All Status" }
                    }
                }
                div { class: "flex items-end",
                    button { class: "btn btn-sm btn-outline w-full", r#type: "button", disabled: true, "Reset" }
                }
            }
        }
    }
}

#[component]
fn PaymentLinksEmpty() -> Element {
    rsx! {
        section { class: "overflow-hidden rounded-2xl border border-border/20 bg-card shadow-xl", role: "status", "data-admin-payment-links-state": ADMIN_PAYMENT_LINKS_EMPTY,
            div { class: "h-[3px] bg-gradient-to-r from-[#7645d9] to-[#ed4b9e]", aria_hidden: "true" }
            div { class: "p-4 sm:p-6 lg:p-8",
                div { class: "mb-6 flex items-center justify-between gap-3",
                    h2 { class: "text-xs font-bold uppercase tracking-[0.2em] text-[#7645d9]", "All Payment Links" }
                    span { class: "rounded-full border border-border/40 bg-muted/50 px-3 py-1 text-xs font-bold text-muted-foreground", "0 links" }
                }
                div { class: "py-12 text-center sm:py-16",
                    div { class: "mx-auto flex h-20 w-20 items-center justify-center rounded-2xl bg-primary/10 text-primary",
                        Icon { name: "link-2".to_string(), size: Some(40) }
                    }
                    h3 { class: "mt-4 text-xl font-semibold text-foreground", "No payment links yet" }
                    p { class: "mt-2 text-muted-foreground", "Create your first payment link to get started" }
                }
            }
        }
    }
}

#[component]
fn PaymentLinksReady(projection: AdminPaymentLinkListProjection) -> Element {
    rsx! {
        section {
            class: "overflow-hidden rounded-2xl border border-border/20 bg-card shadow-xl",
            aria_labelledby: "admin-payment-links-title",
            "data-admin-payment-links-state": ADMIN_PAYMENT_LINKS_READY,
            div { class: "h-[3px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]", aria_hidden: "true" }
            div { class: "flex flex-wrap items-center justify-between gap-3 p-6",
                div {
                    h2 { id: "admin-payment-links-title", class: "text-lg font-semibold", "Payment links" }
                    p { class: "text-sm text-muted-foreground", "{projection.total} authoritative records" }
                }
                p { class: "text-sm text-muted-foreground", "Backend-authoritative lifecycle" }
            }
            if projection.items.is_empty() {
                div { class: "border-t border-border/30 p-8 text-center", role: "status",
                    "No links are present on this bounded page; additional continuation is unavailable."
                }
            } else {
                div { class: "hidden overflow-x-auto md:block",
                    table { class: "min-w-full",
                        caption { class: "sr-only", "Backend-authoritative payment links" }
                        thead { tr { class: "border-y border-border/30",
                            th { class: "px-4 py-3 text-left text-xs uppercase text-muted-foreground", scope: "col", "Link" }
                            th { class: "px-4 py-3 text-left text-xs uppercase text-muted-foreground", scope: "col", "Uses" }
                            th { class: "px-4 py-3 text-left text-xs uppercase text-muted-foreground", scope: "col", "Expires" }
                            th { class: "px-4 py-3 text-left text-xs uppercase text-muted-foreground", scope: "col", "Status" }
                            th { class: "px-4 py-3 text-left text-xs uppercase text-muted-foreground", scope: "col", "Action" }
                        } }
                        tbody { class: "divide-y divide-border/30",
                            for link in projection.items.iter() {
                                PaymentLinkRow { link: link.clone() }
                            }
                        }
                    }
                }
                div { class: "space-y-3 p-4 md:hidden",
                    for link in projection.items.iter() {
                        PaymentLinkCard { link: link.clone() }
                    }
                }
            }
            p { class: "border-t border-border/30 px-6 py-4 text-xs leading-5 text-muted-foreground",
                "Intent identity and audit details remain service-owned; lifecycle actions use the displayed backend version."
            }
        }
    }
}

#[component]
fn PaymentLinkRow(link: AdminPaymentLinkProjection) -> Element {
    let expiry = link
        .expires_at
        .clone()
        .unwrap_or_else(|| "No expiration reported".to_string());
    let uses = if link.max_uses == 0 {
        format!("{} / unlimited", link.current_uses)
    } else {
        format!("{} / {}", link.current_uses, link.max_uses)
    };
    rsx! {
        tr {
            td { class: "px-4 py-4 align-top", code { class: "text-xs", "{link.slug}" } }
            td { class: "px-4 py-4 align-top text-sm", "{uses}" }
            td { class: "px-4 py-4 align-top text-sm text-muted-foreground", "{expiry}" }
            td { class: "px-4 py-4 align-top", StatusBadge { status: link.status.clone() } }
            td { class: "px-4 py-4 align-top",
                if link.status == "active" {
                    form { method: "post", action: "/payments", class: "flex flex-wrap gap-2",
                        input { r#type: "hidden", name: "operation", value: "payment_link_disable" }
                        input { r#type: "hidden", name: "link_id", value: link.id.clone() }
                        input { r#type: "hidden", name: "expected_version", value: link.version.to_string() }
                        input { r#type: "hidden", name: "idempotency_key", value: format!("admin.payment-links.disable.{}", uuid::Uuid::new_v4()) }
                        button { r#type: "submit", class: "btn btn-sm btn-outline", "Disable link" }
                    }
                } else {
                    span { class: "text-xs text-muted-foreground", "Disabled" }
                }
            }
        }
    }
}

#[component]
fn PaymentLinkCard(link: AdminPaymentLinkProjection) -> Element {
    let expiry = link
        .expires_at
        .unwrap_or_else(|| "No expiration reported".to_string());
    let uses = if link.max_uses == 0 {
        format!("{} / unlimited", link.current_uses)
    } else {
        format!("{} / {}", link.current_uses, link.max_uses)
    };
    rsx! {
        article { class: "rounded-xl border border-border/30 p-4",
            div { class: "flex items-start justify-between gap-3",
                code { class: "break-all text-xs", "{link.slug}" }
                StatusBadge { status: link.status.clone() }
            }
            dl { class: "mt-4 grid grid-cols-[auto_1fr] gap-x-3 gap-y-2 text-sm",
                dt { class: "text-muted-foreground", "Uses" }
                dd { "{uses}" }
                dt { class: "text-muted-foreground", "Expires" }
                dd { "{expiry}" }
            }
            if link.status == "active" {
                form { method: "post", action: "/payments", class: "mt-4",
                    input { r#type: "hidden", name: "operation", value: "payment_link_disable" }
                    input { r#type: "hidden", name: "link_id", value: link.id }
                    input { r#type: "hidden", name: "expected_version", value: link.version.to_string() }
                    input { r#type: "hidden", name: "idempotency_key", value: format!("admin.payment-links.disable.{}", uuid::Uuid::new_v4()) }
                    button { r#type: "submit", class: "btn btn-sm btn-outline", "Disable link" }
                }
            }
        }
    }
}

#[component]
fn PaymentLinkCreateForm() -> Element {
    rsx! {
        form { id: "create-payment-link", method: "post", action: "/payments", class: "grid gap-3 rounded-xl border border-border/20 bg-card p-4 shadow-xl md:grid-cols-4 md:items-end",
            input { r#type: "hidden", name: "operation", value: "payment_link_create" }
            input { r#type: "hidden", name: "idempotency_key", value: format!("admin.payment-links.create.{}", uuid::Uuid::new_v4()) }
            label { class: "space-y-2 text-sm font-medium",
                span { "Intent ID" }
                input { class: "input w-full font-mono", name: "intent_id", maxlength: 128, required: true, placeholder: "Existing intent ID" }
            }
            label { class: "space-y-2 text-sm font-medium",
                span { "Maximum uses" }
                input { class: "input w-full", name: "max_uses", r#type: "number", min: 0, max: 1000000, placeholder: "Unlimited" }
            }
            label { class: "space-y-2 text-sm font-medium",
                span { "Expires in seconds" }
                input { class: "input w-full", name: "expires_in", r#type: "number", min: 1, max: 31536000, placeholder: "Optional" }
            }
            button { class: "btn btn-primary", r#type: "submit", "Create payment link" }
        }
    }
}

#[component]
fn PaymentLinksProblem(state: &'static str, title: String, detail: String) -> Element {
    rsx! {
        div {
            "data-admin-payment-links-state": state,
            section {
                class: "rounded-xl border border-amber-500/30 bg-amber-500/10 px-5 py-4",
                role: if state == ADMIN_PAYMENT_LINKS_FORBIDDEN { "alert" } else { "status" },
                div { class: "flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between",
                    div {
                        h2 { class: "font-semibold", "{title}" }
                        p { class: "mt-1 text-sm text-muted-foreground", "{detail}" }
                    }
                    nav { class: "flex shrink-0 flex-wrap gap-2", aria_label: "Payment-link recovery",
                        a { class: "btn btn-sm btn-outline", href: "/payments?tab=payment-links", "Retry payment links" }
                        a { class: "btn btn-sm btn-ghost", href: "/", "Admin home" }
                    }
                }
            }
            section { class: "mt-6 overflow-hidden rounded-2xl border border-border/20 bg-card shadow-xl", aria_label: "Payment-link inventory unavailable",
                div { class: "h-[3px] bg-gradient-to-r from-[#7645d9] to-[#ed4b9e]", aria_hidden: "true" }
                div { class: "p-4 sm:p-6 lg:p-8",
                    div { class: "mb-6 flex items-center justify-between gap-3",
                        h3 { class: "text-xs font-bold uppercase tracking-[0.2em] text-[#7645d9]", "All Payment Links" }
                        span { class: "rounded-full border border-border/40 bg-muted/50 px-3 py-1 font-mono text-xs text-amber-400", "Unavailable" }
                    }
                    div { class: "rounded-xl border border-dashed border-border/40 px-6 py-12 text-center",
                        Icon { name: "link-2".to_string(), size: Some(36) }
                        p { class: "mt-3 text-sm font-semibold text-foreground", "No verified link inventory" }
                        p { class: "mt-1 text-xs text-muted-foreground", "Creation and lifecycle actions remain disabled until the backend responds." }
                    }
                }
            }
        }
    }
}

#[component]
fn PaymentFilterForm(filters: PaymentFilters) -> Element {
    let payer = filters.payer.clone().unwrap_or_default();
    let status = filters.status.clone().unwrap_or_default();
    rsx! {
        form { class: "payments-filter-panel rounded-xl border border-border/20 bg-card p-4", method: "GET", action: "/payments",
            input { r#type: "hidden", name: "tab", value: "payments" }
            input { r#type: "hidden", name: "offset", value: "0" }
            div { class: "grid grid-cols-1 gap-4 md:grid-cols-4 md:items-end",
                label { class: "space-y-2 text-sm font-medium",
                    span { "Payer" }
                    input { class: "input w-full font-mono", r#type: "text", name: "payer", value: payer, maxlength: "128", placeholder: "Wallet address" }
                }
                label { class: "space-y-2 text-sm font-medium",
                    span { "Status" }
                    input { class: "input w-full", r#type: "text", name: "status", value: status, maxlength: "32", placeholder: "All statuses" }
                }
                label { class: "space-y-2 text-sm font-medium",
                    span { "Rows per page" }
                    select { class: "input w-full", name: "limit",
                        for value in [10usize, 20, 50, 100] {
                            option { value: "{value}", selected: filters.limit == value, "{value}" }
                        }
                    }
                }
                button { class: "btn btn-primary", r#type: "submit", "Apply filters" }
            }
        }
    }
}

#[component]
fn PaymentIntentList(payload: AdminPaymentIntentList, filters: PaymentFilters) -> Element {
    let total = usize::try_from(payload.total).unwrap_or(0);
    let current_page = filters.offset / filters.limit + 1;
    let total_pages = total.div_ceil(filters.limit).max(1);
    let previous_offset = filters.offset.saturating_sub(filters.limit);
    let next_offset = filters.offset.saturating_add(filters.limit);
    let has_previous = filters.offset > 0;
    let has_next = next_offset < total;

    rsx! {
        section { class: "payment-intents-list rounded-2xl border border-border/20 overflow-hidden bg-card",
            div { class: "h-[3px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]" }
            div { class: "flex flex-wrap items-center justify-between gap-3 p-6",
                div {
                    h2 { class: "text-lg font-semibold", "Payment intents" }
                    p { class: "text-sm text-muted-foreground", "{payload.total} authoritative records" }
                }
                p { class: "text-sm text-muted-foreground", "Page {current_page} of {total_pages}" }
            }
            if payload.items.is_empty() {
                div { class: "border-t border-border/30 p-8 text-center", role: "status",
                    h3 { class: "font-semibold", "No payment intents on this page" }
                    p { class: "mt-2 text-sm text-muted-foreground", "The filtered inventory still contains records. Return to the first page or use Previous." }
                    a { class: "btn btn-sm btn-outline mt-4", href: filters.page_url(0), "Return to first page" }
                }
            } else {
                div { class: "hidden overflow-x-auto md:block",
                    table { class: "min-w-full",
                        caption { class: "sr-only", "Backend-authoritative payment intents" }
                        thead {
                            tr { class: "border-y border-border/30",
                                th { class: "px-4 py-3 text-left text-xs uppercase text-muted-foreground", scope: "col", "Intent" }
                                th { class: "px-4 py-3 text-left text-xs uppercase text-muted-foreground", scope: "col", "Payer / payee" }
                                th { class: "px-4 py-3 text-left text-xs uppercase text-muted-foreground", scope: "col", "Amount / token" }
                                th { class: "px-4 py-3 text-left text-xs uppercase text-muted-foreground", scope: "col", "Chain" }
                                th { class: "px-4 py-3 text-left text-xs uppercase text-muted-foreground", scope: "col", "Status" }
                                th { class: "px-4 py-3 text-left text-xs uppercase text-muted-foreground", scope: "col", "Created" }
                                th { class: "px-4 py-3 text-left text-xs uppercase text-muted-foreground", scope: "col", "Action" }
                            }
                        }
                        tbody { class: "divide-y divide-border/30",
                            for intent in payload.items.iter() {
                                PaymentIntentRow { intent: intent.clone() }
                            }
                        }
                    }
                }
                div { class: "space-y-3 p-4 md:hidden",
                    for intent in payload.items.iter() {
                        PaymentIntentCard { intent: intent.clone() }
                    }
                }
            }
            nav { class: "flex items-center justify-between border-t border-border/30 p-4", aria_label: "Payment intent pagination",
                if has_previous {
                    a { class: "btn btn-sm btn-outline", href: filters.page_url(previous_offset), rel: "prev", "Previous" }
                } else {
                    span { class: "btn btn-sm btn-outline opacity-50", aria_disabled: "true", "Previous" }
                }
                if has_next {
                    a { class: "btn btn-sm btn-outline", href: filters.page_url(next_offset), rel: "next", "Next" }
                } else {
                    span { class: "btn btn-sm btn-outline opacity-50", aria_disabled: "true", "Next" }
                }
            }
        }
    }
}

#[component]
fn PaymentIntentRow(intent: AdminPaymentIntent) -> Element {
    rsx! {
        tr {
            td { class: "px-4 py-4 align-top",
                code { class: "text-xs", "{intent.id}" }
                if let Some(hash) = &intent.tx_hash {
                    p { class: "mt-1 font-mono text-xs text-muted-foreground", "Tx {hash}" }
                }
            }
            td { class: "px-4 py-4 align-top font-mono text-xs",
                p { "From {intent.payer}" }
                p { class: "mt-1 text-muted-foreground", "To {intent.payee}" }
            }
            td { class: "px-4 py-4 align-top",
                p { class: "font-semibold", "{intent.amount}" }
                code { class: "text-xs text-muted-foreground", "{intent.token_address}" }
            }
            td { class: "px-4 py-4 align-top", "{intent.chain_id}" }
            td { class: "px-4 py-4 align-top", StatusBadge { status: intent.status.clone() } }
            td { class: "px-4 py-4 align-top text-sm text-muted-foreground", "{intent.created_at}" }
            td { class: "px-4 py-4 align-top",
                if intent.status == "pending" {
                    if let Some(version) = payment_intent_expected_version(&intent.updated_at) {
                        form { method: "post", action: "/payments", class: "flex flex-wrap gap-2",
                            input { r#type: "hidden", name: "operation", value: "payment_intent_cancel" }
                            input { r#type: "hidden", name: "intent_id", value: intent.id.clone() }
                            input { r#type: "hidden", name: "expected_version", value: version.to_string() }
                            input { r#type: "hidden", name: "idempotency_key", value: format!("admin.payment-intents.cancel.{}", uuid::Uuid::new_v4()) }
                            button { r#type: "submit", class: "btn btn-sm btn-outline", "Cancel intent" }
                        }
                    } else {
                        span { class: "text-xs text-muted-foreground", "Version unavailable" }
                    }
                } else {
                    span { class: "text-xs text-muted-foreground", "No action" }
                }
            }
        }
    }
}

#[component]
fn PaymentIntentCard(intent: AdminPaymentIntent) -> Element {
    rsx! {
        article { class: "rounded-xl border border-border/30 p-4",
            div { class: "flex items-start justify-between gap-3",
                code { class: "break-all text-xs", "{intent.id}" }
                StatusBadge { status: intent.status.clone() }
            }
            dl { class: "mt-4 grid grid-cols-[auto_1fr] gap-x-3 gap-y-2 text-sm",
                dt { class: "text-muted-foreground", "Payer" }
                dd { class: "break-all font-mono text-xs", "{intent.payer}" }
                dt { class: "text-muted-foreground", "Payee" }
                dd { class: "break-all font-mono text-xs", "{intent.payee}" }
                dt { class: "text-muted-foreground", "Amount" }
                dd { "{intent.amount}" }
                dt { class: "text-muted-foreground", "Token" }
                dd { class: "break-all font-mono text-xs", "{intent.token_address}" }
                dt { class: "text-muted-foreground", "Chain" }
                dd { "{intent.chain_id}" }
                dt { class: "text-muted-foreground", "Created" }
                dd { "{intent.created_at}" }
            }
            if intent.status == "pending" {
                if let Some(version) = payment_intent_expected_version(&intent.updated_at) {
                    form { method: "post", action: "/payments", class: "mt-4",
                        input { r#type: "hidden", name: "operation", value: "payment_intent_cancel" }
                        input { r#type: "hidden", name: "intent_id", value: intent.id }
                        input { r#type: "hidden", name: "expected_version", value: version.to_string() }
                        input { r#type: "hidden", name: "idempotency_key", value: format!("admin.payment-intents.cancel.{}", uuid::Uuid::new_v4()) }
                        button { r#type: "submit", class: "btn btn-sm btn-outline", "Cancel intent" }
                    }
                }
            }
        }
    }
}

#[component]
fn StatusBadge(status: String) -> Element {
    rsx! {
        span { class: "inline-flex rounded-full border border-border/40 bg-muted/40 px-2.5 py-1 text-xs font-semibold", "{status}" }
    }
}

#[component]
fn LoadProblem(title: String, detail: String, retry_url: String) -> Element {
    rsx! {
        section { class: "rounded-2xl border border-amber-500/30 bg-amber-500/5 p-8", role: "alert",
            h2 { class: "text-lg font-semibold", "{title}" }
            p { class: "mt-2 text-sm text-muted-foreground", "{detail}" }
            a { class: "btn btn-sm btn-outline mt-5", href: retry_url, "Try again" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{user::AuthMethod, User};

    fn authed_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "admin-1".to_string(),
                address: "0xadmin".to_string(),
                chain_id: "56".to_string(),
                roles: vec![],
                email: None,
                tier: None,
                permissions: vec![
                    "admin:payments:view".to_string(),
                    "admin:payment-links:view".to_string(),
                ],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: None,
            }),
            path: "/payments".to_string(),
            ..Default::default()
        }
    }

    fn intent(id: &str) -> AdminPaymentIntent {
        AdminPaymentIntent {
            id: id.to_string(),
            chain_id: "56".to_string(),
            payer: "0x1111111111111111111111111111111111111111".to_string(),
            payee: "0x2222222222222222222222222222222222222222".to_string(),
            amount: "1000000000000000000".to_string(),
            token_address: "0x3333333333333333333333333333333333333333".to_string(),
            status: "pending".to_string(),
            escrow_id: None,
            tx_hash: Some("0xtransaction".to_string()),
            description: None,
            expires_at: None,
            created_at: "2026-07-22T10:00:00Z".to_string(),
            updated_at: "2026-07-22T10:00:00Z".to_string(),
        }
    }

    fn with_load(state: &str, payload: Option<AdminPaymentIntentList>) -> PageContext {
        let mut ctx = authed_ctx();
        ctx.params
            .insert(ADMIN_PAYMENTS_STATE_PARAM.to_string(), state.to_string());
        if let Some(payload) = payload {
            ctx.params.insert(
                ADMIN_PAYMENTS_DATA_PARAM.to_string(),
                serde_json::to_string(&payload).unwrap(),
            );
        }
        ctx.params
            .insert(ADMIN_PAYMENTS_LIMIT_PARAM.to_string(), "20".to_string());
        ctx.params
            .insert(ADMIN_PAYMENTS_OFFSET_PARAM.to_string(), "0".to_string());
        ctx
    }

    fn link(slug: &str) -> AdminPaymentLinkProjection {
        AdminPaymentLinkProjection {
            id: "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".to_string(),
            slug: slug.to_string(),
            max_uses: 3,
            current_uses: 1,
            expires_at: Some("2026-12-31T00:00:00Z".to_string()),
            status: "active".to_string(),
            version: 0,
        }
    }

    fn with_links(state: &str, payload: Option<AdminPaymentLinkListProjection>) -> PageContext {
        let mut ctx = authed_ctx();
        ctx.params.insert(
            ADMIN_PAYMENTS_TAB_PARAM.to_string(),
            "payment-links".to_string(),
        );
        ctx.params.insert(
            ADMIN_PAYMENT_LINKS_STATE_PARAM.to_string(),
            state.to_string(),
        );
        if let Some(payload) = payload {
            ctx.params.insert(
                ADMIN_PAYMENT_LINKS_DATA_PARAM.to_string(),
                serde_json::to_string(&payload).unwrap(),
            );
        }
        ctx
    }

    fn with_user_access(
        state: &str,
        payload: Option<AdminPaymentUserAccessProjection>,
    ) -> PageContext {
        let mut ctx = authed_ctx();
        ctx.query = "tab=user-access&page=1&limit=20".to_string();
        ctx.params.insert(
            ADMIN_PAYMENTS_TAB_PARAM.to_string(),
            "user-access".to_string(),
        );
        ctx.params.insert(
            ADMIN_PAYMENT_USER_ACCESS_STATE_PARAM.to_string(),
            state.to_string(),
        );
        if let Some(payload) = payload {
            ctx.params.insert(
                ADMIN_PAYMENT_USER_ACCESS_DATA_PARAM.to_string(),
                serde_json::to_string(&payload).unwrap(),
            );
        }
        ctx
    }

    fn render_html(ctx: &PageContext) -> String {
        let (_, element) = render(ctx);
        dioxus_ssr::render_element(element)
    }

    #[test]
    fn ready_renders_only_service_owned_fields_and_escapes_text() {
        let mut row = intent("intent-1");
        row.status = "<script>alert(1)</script>".to_string();
        let html = render_html(&with_load(
            ADMIN_PAYMENTS_READY,
            Some(AdminPaymentIntentList {
                items: vec![row],
                total: 1,
            }),
        ));

        assert!(html.contains("intent-1"), "rendered HTML: {html:?}");
        assert!(html.contains("1000000000000000000"));
        assert!(html.contains("&#60;script&#62;alert(1)&#60;/script&#62;"));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("Total Revenue"));
        assert!(html.contains("Export CSV"));
        for invented in ["$45,231", "pi_abc123", "Pro Plan Monthly"] {
            assert!(
                !html.contains(invented),
                "invented value leaked: {invented}"
            );
        }
    }

    #[test]
    fn empty_unavailable_and_malformed_are_distinct() {
        let empty = render_html(&with_load(
            ADMIN_PAYMENTS_EMPTY,
            Some(AdminPaymentIntentList {
                items: vec![],
                total: 0,
            }),
        ));
        let unavailable = render_html(&with_load(ADMIN_PAYMENTS_UNAVAILABLE, None));
        let malformed = render_html(&with_load(ADMIN_PAYMENTS_MALFORMED, None));

        assert!(empty.contains("No payment intents found"));
        assert!(unavailable.contains("Payment intents unavailable"));
        assert!(unavailable.contains("Total Revenue"));
        assert!(unavailable.contains("Export CSV"));
        assert!(!unavailable.contains("No payment intents found"));
        assert!(malformed.contains("Payment data could not be verified"));
        assert!(!malformed.contains("No payment intents found"));
    }

    #[test]
    fn inconsistent_state_or_payload_fails_closed_as_malformed() {
        let ctx = with_load(
            ADMIN_PAYMENTS_READY,
            Some(AdminPaymentIntentList {
                items: vec![],
                total: 0,
            }),
        );
        assert!(render_html(&ctx).contains("Payment data could not be verified"));

        let invalid = serde_json::json!({"items": [intent("intent-1")], "total": 0});
        assert!(decode_admin_payment_intent_list(invalid).is_none());
    }

    #[test]
    fn nonzero_total_empty_page_preserves_truth_and_recovery_navigation() {
        let mut ctx = with_load(
            ADMIN_PAYMENTS_READY,
            Some(AdminPaymentIntentList {
                items: vec![],
                total: 41,
            }),
        );
        ctx.params
            .insert(ADMIN_PAYMENTS_OFFSET_PARAM.to_string(), "40".to_string());

        let html = render_html(&ctx);
        assert!(html.contains("41 authoritative records"));
        assert!(html.contains("No payment intents on this page"));
        assert!(html.contains("Return to first page"));
        assert!(html.contains("offset=0"));
        assert!(html.contains("Previous"));
        assert!(!html.contains("No payment intents found"));
    }

    #[test]
    fn view_permission_is_required_without_manage_substitution() {
        let mut denied = with_load(ADMIN_PAYMENTS_UNAVAILABLE, None);
        denied.user.as_mut().unwrap().permissions = vec!["admin:payments:manage".to_string()];
        let html = render_html(&denied);
        assert!(html.contains("Permission required"));
        assert!(html.contains("admin:payments:view"));
        assert!(!html.contains("Payments Hub"));
    }

    #[test]
    fn unavailable_tabs_have_no_mutation_controls_or_sample_records() {
        for (tab, expected) in [
            ("user-access", "User access is unavailable"),
            ("payment-links", "Payment links are unavailable"),
        ] {
            let mut ctx = with_load(ADMIN_PAYMENTS_UNAVAILABLE, None);
            ctx.params
                .insert(ADMIN_PAYMENTS_TAB_PARAM.to_string(), tab.to_string());
            let html = render_html(&ctx);
            assert!(html.contains(expected));
            for forbidden in [
                "Create Payment Link",
                "Revoke link",
                "Pro Plan Monthly",
                "Users with access",
            ] {
                assert!(
                    !html.contains(forbidden),
                    "forbidden control/sample leaked: {forbidden}"
                );
            }
        }
    }

    #[test]
    fn payment_link_projection_is_strict_and_exposes_versioned_lifecycle_controls() {
        let payload = AdminPaymentLinkListProjection {
            items: vec![link("epsx-link_1")],
            total: 1,
            limit: 20,
            offset: 0,
        };
        let html = render_html(&with_links(ADMIN_PAYMENT_LINKS_READY, Some(payload)));
        assert!(html.contains("data-admin-payment-links-state=\"ready\""));
        assert!(html.contains("epsx-link_1"));
        assert!(html.contains("1 / 3"));
        assert!(html.contains("Create payment link"));
        assert!(html.contains("Disable link"));
        assert!(html.contains("expected_version"));
        assert!(html.contains("idempotency_key"));

        assert!(decode_admin_payment_link_projection(serde_json::json!({
            "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            "slug": "epsx-link_1",
            "max_uses": 3,
            "current_uses": 1,
            "expires_at": null,
            "status": "active",
            "version": 0,
        }))
        .is_some());
        assert!(decode_admin_payment_link_projection(serde_json::json!({
            "slug": "epsx-link_1",
            "max_uses": 3,
            "current_uses": 1,
            "expires_at": null,
            "status": "active",
            "id": "0xprivate-intent",
            "intent_id": "private-intent",
            "version": 4,
            "correlation_id": "private-request",
        }))
        .is_none());
        assert!(decode_admin_payment_link_projection(serde_json::json!({
            "id": "link-1",
            "slug": "epsx/link",
            "max_uses": 3,
            "current_uses": 1,
            "expires_at": null,
            "status": "active",
            "version": 0,
        }))
        .is_none());
    }

    #[test]
    fn payment_link_states_preserve_empty_forbidden_unavailable_and_malformed() {
        let empty = render_html(&with_links(
            ADMIN_PAYMENT_LINKS_EMPTY,
            Some(AdminPaymentLinkListProjection {
                items: vec![],
                total: 0,
                limit: 20,
                offset: 0,
            }),
        ));
        assert!(empty.contains("No payment links yet"));
        assert!(empty.contains("All Payment Links"));
        for (state, title) in [
            (
                ADMIN_PAYMENT_LINKS_FORBIDDEN,
                "Payment-link access was denied",
            ),
            (
                ADMIN_PAYMENT_LINKS_UNAVAILABLE,
                "Payment links are unavailable",
            ),
            (
                ADMIN_PAYMENT_LINKS_MALFORMED,
                "Payment-link data could not be verified",
            ),
        ] {
            let html = render_html(&with_links(
                state,
                Some(AdminPaymentLinkListProjection {
                    items: vec![link("epsx-link_1")],
                    total: 1,
                    limit: 20,
                    offset: 0,
                }),
            ));
            assert!(html.contains(&format!("data-admin-payment-links-state=\"{state}\"")));
            assert!(html.contains(title));
            assert!(html.contains("All Payment Links"));
            assert!(html.contains("No verified link inventory"));
            assert!(!html.contains("epsx-link_1"));
        }
    }

    #[test]
    fn payment_link_read_permission_is_not_payment_manage_or_generic_read() {
        let mut ctx = with_links(ADMIN_PAYMENT_LINKS_UNAVAILABLE, None);
        ctx.user.as_mut().unwrap().permissions = vec!["admin:payments:manage".to_string()];
        let html = render_html(&ctx);
        assert!(html.contains("Permission required"));
        assert!(html.contains("admin:payment-links:view"));
        assert!(!html.contains("Payment links are unavailable"));
    }

    #[test]
    fn user_access_query_projection_and_responsive_views_are_backend_owned() {
        let query = AdminPaymentUserAccessQuery::from_raw(
            "tab=user-access&page=2&limit=20&status=expiring_soon&search=0x1111",
        )
        .expect("valid user-access query");
        assert_eq!(query.page, 2);
        assert_eq!(query.status.as_deref(), Some("expiring_soon"));
        assert!(AdminPaymentUserAccessQuery::from_raw("tab=user-access&page=0").is_err());
        assert!(AdminPaymentUserAccessQuery::from_raw(
            "tab=user-access&status=active&status=expired"
        )
        .is_err());

        let projection = AdminPaymentUserAccessProjection {
            items: vec![AdminPaymentUserAccessItem {
                wallet_address: "0x1111111111111111111111111111111111111111".to_string(),
                current_plan_id: Some("00000000-0000-0000-0000-000000000001".to_string()),
                plan_name: Some("Lifetime".to_string()),
                plan_expires_at: None,
                days_remaining: 365,
                status: "active".to_string(),
            }],
            page: 1,
            limit: 20,
            total_pages: 1,
        };
        assert!(decode_admin_payment_user_access_projection(
            serde_json::to_value(&projection).unwrap()
        )
        .is_some());
        let html = render_html(&with_user_access(
            ADMIN_PAYMENT_USER_ACCESS_READY,
            Some(projection),
        ));
        assert!(html.contains("All Users with Plan Access"));
        assert!(html.contains("Lifetime"));
        assert!(html.contains("sm:hidden"));
        assert!(html.contains("sm:block"));
        assert!(html.contains("/wallet-management/0x1111111111111111111111111111111111111111"));
        assert!(!html.contains("Permission required"));
    }

    #[test]
    fn user_access_states_are_distinct_and_never_reuse_stale_rows() {
        let empty = AdminPaymentUserAccessProjection {
            items: vec![],
            page: 1,
            limit: 20,
            total_pages: 1,
        };
        assert!(render_html(&with_user_access(
            ADMIN_PAYMENT_USER_ACCESS_EMPTY,
            Some(empty)
        ))
        .contains("No users with plan access found"));
        for (state, title) in [
            (
                ADMIN_PAYMENT_USER_ACCESS_FORBIDDEN,
                "User-access read was denied",
            ),
            (
                ADMIN_PAYMENT_USER_ACCESS_UNAVAILABLE,
                "User access is unavailable",
            ),
            (
                ADMIN_PAYMENT_USER_ACCESS_MALFORMED,
                "User-access data could not be verified",
            ),
        ] {
            let html = render_html(&with_user_access(state, None));
            assert!(html.contains(title));
            assert!(!html.contains("Lifetime"));
        }
    }

    #[test]
    fn unauthenticated_and_unauthorized_decode_and_render_the_shared_banner() {
        let intents_unauthenticated = with_load(ADMIN_PAYMENTS_UNAUTHENTICATED, None);
        assert_eq!(
            payment_load(&intents_unauthenticated),
            PaymentLoad::Unauthenticated
        );
        let html = render_html(&intents_unauthenticated);
        assert!(html.contains("data-admin-data-state=\"unauthenticated\""));
        assert!(html.contains("Sign in required"));

        let intents_unauthorized = with_load(ADMIN_PAYMENTS_UNAUTHORIZED, None);
        assert_eq!(
            payment_load(&intents_unauthorized),
            PaymentLoad::Unauthorized
        );
        assert!(
            render_html(&intents_unauthorized).contains("data-admin-data-state=\"unauthorized\"")
        );

        let links_unauthenticated = with_links(ADMIN_PAYMENT_LINKS_UNAUTHENTICATED, None);
        assert_eq!(
            payment_links_load(&links_unauthenticated),
            PaymentLinksLoad::Unauthenticated
        );
        let links_unauthorized = with_links(ADMIN_PAYMENT_LINKS_UNAUTHORIZED, None);
        assert_eq!(
            payment_links_load(&links_unauthorized),
            PaymentLinksLoad::Unauthorized
        );
        assert!(render_html(&links_unauthorized).contains("Session expired"));

        let access_unauthenticated =
            with_user_access(ADMIN_PAYMENT_USER_ACCESS_UNAUTHENTICATED, None);
        assert_eq!(
            payment_user_access_load(&access_unauthenticated),
            PaymentUserAccessLoad::Unauthenticated
        );
        let access_unauthorized = with_user_access(ADMIN_PAYMENT_USER_ACCESS_UNAUTHORIZED, None);
        assert_eq!(
            payment_user_access_load(&access_unauthorized),
            PaymentUserAccessLoad::Unauthorized
        );
        assert!(render_html(&access_unauthorized).contains("Session expired"));
    }

    #[test]
    fn pagination_preserves_only_normalized_filters() {
        let mut ctx = with_load(
            ADMIN_PAYMENTS_READY,
            Some(AdminPaymentIntentList {
                items: vec![intent("intent-1")],
                total: 41,
            }),
        );
        ctx.params.insert(
            ADMIN_PAYMENTS_PAYER_PARAM.to_string(),
            "0x1111111111111111111111111111111111111111".to_string(),
        );
        ctx.params.insert(
            ADMIN_PAYMENTS_STATUS_PARAM.to_string(),
            "pending".to_string(),
        );
        let html = render_html(&ctx);
        assert!(html.contains("offset=20"));
        assert!(html.contains("payer=0x1111111111111111111111111111111111111111"));
        assert!(html.contains("status=pending"));
        assert!(!html.contains("admin:payments:manage"));
    }
}
