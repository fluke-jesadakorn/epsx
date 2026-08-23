//! Route-owned BFF loaders for the admin commerce/wallet read surfaces.
//!
//! Central SSR/router wiring calls these loaders. Each loader sends the verified bearer and
//! request ID to the service that owns the record, rejects non-canonical route
//! identifiers before network I/O, and projects only the fields accepted by
//! the corresponding Dioxus page.

use chrono::DateTime;
use epsx_dioxus_ui::pages::admin_pages::{
    payments::{
        decode_admin_payment_link_list_projection, decode_admin_payment_user_access_projection,
        AdminPaymentLinkListProjection, AdminPaymentUserAccessProjection,
        AdminPaymentUserAccessQuery,
    },
    wallet_access::{decode_admin_access_projection, AdminAccessProjection},
    wallet_credits::{decode_admin_credit_stats_projection, AdminCreditStatsProjection},
    wallet_plans::{
        decode_admin_plan_list_projection, decode_admin_plan_projection, AdminPlanListProjection,
        AdminPlanProjection,
    },
    wallet_wallets::{
        decode_admin_wallet_detail_projection, decode_admin_wallet_list_projection,
        decode_admin_wallet_stats_projection, AdminWalletDetailProjection,
        AdminWalletListProjection, AdminWalletListQuery, AdminWalletStatsSummary,
    },
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::upstream::UpstreamFailure;

const MAX_RESPONSE_BYTES: usize = 512 * 1024;
const WALLET_STATS_PATH: &str = "/api/v1/admin/wallets/stats";
const WALLET_DETAIL_PREFIX: &str = "/api/v1/admin/wallets/";
const CREDIT_STATS_PATH: &str = "/api/v1/admin/credits";
const ACCESS_PATH: &str = "/api/v1/admin/subscription/access?limit=100&offset=0";
const PLANS_PATH: &str = "/api/v1/admin/subscription/plans?limit=100&offset=0";
const PAYMENT_LINKS_PATH: &str = "/api/v1/admin/pay/links?limit=100&offset=0";
const MONOLITH_PAYMENT_LINKS_PATH: &str = "/api/admin/payment-links?limit=100&offset=0";
const PLAN_DETAIL_PREFIX: &str = "/api/v1/admin/subscription/plans/";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AdminCommerceLoad<T> {
    Ready(T),
    Empty,
    Forbidden,
    Unauthorized,
    Unavailable,
    Malformed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AdminCommerceMutationLoad<T> {
    Ready(T),
    Forbidden,
    Conflict,
    Unauthorized,
    Unavailable,
    Malformed,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct WalletStatusCommand {
    pub expected_version: i64,
    pub reason: String,
}

pub(crate) fn wallet_status_mutation_path(address: &str, enable: bool) -> Option<String> {
    canonical_wallet(address).map(|address| {
        format!(
            "/api/v1/admin/wallets/{address}/{}",
            if enable { "enable" } else { "disable" }
        )
    })
}

#[cfg(test)]
pub(crate) fn credit_mutation_path(address: &str, operation: &str) -> Option<String> {
    if !matches!(operation, "grant" | "revoke") {
        return None;
    }
    canonical_wallet(address).map(|address| format!("/api/v1/admin/credits/{address}/{operation}"))
}

#[cfg(test)]
pub(crate) fn access_mutation_path(operation: &str) -> Option<&'static str> {
    match operation {
        "assign" => Some("/api/v1/admin/subscription/access/assign"),
        "revoke" => Some("/api/v1/admin/subscription/access/revoke"),
        _ => None,
    }
}

#[cfg(test)]
pub(crate) fn plan_mutation_path(plan_id: Option<&str>) -> Option<String> {
    match plan_id {
        None => Some("/api/v1/admin/subscription/plans".to_string()),
        Some(plan_id) => {
            canonical_uuid(plan_id).map(|plan_id| format!("{PLAN_DETAIL_PREFIX}{plan_id}"))
        }
    }
}

#[cfg(test)]
pub(crate) fn payment_link_mutation_path(link_id: Option<&str>) -> Option<String> {
    match link_id {
        None => Some("/api/v1/admin/pay/links".to_string()),
        Some(link_id) if valid_resource_id(link_id) => {
            Some(format!("/api/v1/admin/pay/links/{link_id}/disable"))
        }
        _ => None,
    }
}

#[cfg(test)]
pub(crate) fn payment_intent_cancel_path(intent_id: &str) -> Option<String> {
    valid_resource_id(intent_id).then(|| format!("/api/v1/admin/pay/intents/{intent_id}/cancel"))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UpstreamError {
    Forbidden,
    Unauthorized,
    Unavailable,
    Malformed,
}

impl From<UpstreamFailure> for UpstreamError {
    fn from(failure: UpstreamFailure) -> Self {
        match failure {
            UpstreamFailure::Unauthorized => Self::Unauthorized,
            UpstreamFailure::Forbidden => Self::Forbidden,
            UpstreamFailure::Malformed => Self::Malformed,
            UpstreamFailure::Unavailable => Self::Unavailable,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendWallet {
    address: String,
    chain_id: String,
    label: Option<String>,
    role: Option<String>,
    status: String,
    #[serde(rename = "metadata")]
    _metadata: Value,
    version: i64,
    #[serde(rename = "created_at")]
    _created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendPaymentUserAccessEnvelope {
    success: bool,
    data: BackendPaymentUserAccessData,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendPaymentUserAccessData {
    users: Vec<BackendPaymentUserAccessItem>,
    pagination: BackendPaymentUserAccessPagination,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendPaymentUserAccessItem {
    wallet_address: String,
    current_plan_id: Option<String>,
    plan_name: Option<String>,
    plan_expires_at: Option<String>,
    days_remaining: i64,
    status: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendPaymentUserAccessPagination {
    page: i64,
    limit: i64,
    #[serde(rename = "total")]
    _total: i64,
    total_pages: i64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendWalletStats {
    total: i64,
    active: i64,
    disabled: i64,
    new_30_days: i64,
    #[serde(rename = "correlation_id")]
    _correlation_id: String,
}

#[cfg(test)]
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AdminApiResponseEnvelope {
    success: bool,
    data: Option<Value>,
    error: Option<String>,
    message: String,
    timestamp: String,
    admin_meta: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendCreditStats {
    outstanding_minor: i64,
    granted_today_minor: i64,
    revoked_today_minor: i64,
    active_accounts: i64,
    #[serde(rename = "correlation_id")]
    _correlation_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendWalletList {
    items: Vec<BackendWallet>,
    total: i64,
    limit: i64,
    offset: i64,
    #[serde(rename = "correlation_id")]
    _correlation_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendWalletMutation {
    wallet: BackendWallet,
    evidence: BackendWalletMutationEvidence,
    correlation_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendWalletMutationEvidence {
    operation_id: String,
    version: i64,
    observed_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendAccessList {
    items: Vec<BackendAccessAssignment>,
    #[serde(rename = "correlation_id")]
    _correlation_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendAccessAssignment {
    wallet_address: String,
    plan_id: String,
    plan_name: String,
    permission: String,
    expires_at: Option<String>,
    version: i64,
    #[serde(rename = "assigned_by")]
    _assigned_by: String,
    #[serde(rename = "updated_at")]
    _updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendPlanList {
    items: Vec<BackendPlan>,
    total: i64,
    limit: i64,
    offset: i64,
    #[serde(rename = "correlation_id")]
    _correlation_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendPlan {
    id: String,
    #[serde(rename = "merchant_id")]
    _merchant_id: String,
    name: String,
    description: Option<String>,
    amount: String,
    currency: String,
    chain_id: String,
    interval: i32,
    active: Option<bool>,
    #[serde(rename = "created_at")]
    _created_at: Option<String>,
    version: i64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendLinkList {
    items: Vec<BackendLink>,
    total: i64,
    limit: i64,
    offset: i64,
    #[serde(rename = "correlation_id")]
    _correlation_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendLink {
    id: String,
    slug: String,
    #[serde(rename = "intent_id")]
    _intent_id: String,
    max_uses: i32,
    current_uses: i32,
    expires_at: Option<String>,
    #[serde(rename = "created_at")]
    _created_at: String,
    status: String,
    version: i64,
}

#[derive(Debug, Deserialize)]
struct MonolithLinkList {
    payment_links: Vec<MonolithLink>,
    total: i64,
    limit: i64,
    offset: i64,
}

#[derive(Debug, Deserialize)]
struct MonolithLink {
    id: String,
    slug: String,
    max_uses: Option<i32>,
    current_uses: i32,
    expires_at: Option<String>,
    is_active: bool,
}

pub(crate) fn wallet_detail_path(address: &str) -> Option<String> {
    canonical_wallet(address).map(|address| format!("{WALLET_DETAIL_PREFIX}{address}"))
}

pub(crate) fn wallet_access_path(address: &str) -> Option<String> {
    let address = canonical_wallet(address)?;
    let encoded_query = {
        let mut query = url::form_urlencoded::Serializer::new(String::new());
        query.append_pair("wallet_address", &address);
        query.append_pair("limit", "100");
        query.append_pair("offset", "0");
        query.finish()
    };
    Some(format!("/api/v1/admin/subscription/access?{encoded_query}"))
}

pub(crate) fn plan_detail_path(plan_id: &str) -> Option<String> {
    canonical_uuid(plan_id).map(|plan_id| format!("{PLAN_DETAIL_PREFIX}{plan_id}"))
}

pub(crate) async fn load_wallet_stats(
    client: &epsx_client::ServiceClient,
    ctx: &epsx_client::RequestContext,
) -> AdminCommerceLoad<AdminWalletStatsSummary> {
    let payload = match get_json::<BackendWalletStats>(client, WALLET_STATS_PATH, ctx).await {
        Ok(payload) => payload,
        Err(error) => return error.into_load(),
    };
    let projection = serde_json::json!({
        "total_users": payload.total,
        "active_users": payload.active,
        "inactive_users": payload.disabled,
        "new_users_30_days": payload.new_30_days,
    });
    decode_admin_wallet_stats_projection(projection)
        .map(AdminCommerceLoad::Ready)
        .unwrap_or(AdminCommerceLoad::Malformed)
}

pub(crate) async fn load_wallet_detail(
    client: &epsx_client::ServiceClient,
    address: &str,
    ctx: &epsx_client::RequestContext,
) -> AdminCommerceLoad<AdminWalletDetailProjection> {
    let Some(requested_address) = canonical_wallet(address) else {
        return AdminCommerceLoad::Malformed;
    };
    let Some(path) = wallet_detail_path(address) else {
        return AdminCommerceLoad::Malformed;
    };
    let payload = match get_json::<BackendWallet>(client, &path, ctx).await {
        Ok(payload) => payload,
        Err(error) => return error.into_load(),
    };
    let returned_address = payload.address.clone();
    let projection = serde_json::json!({
        "address": returned_address,
        "chain_id": payload.chain_id,
        "label": payload.label,
        "role": payload.role,
        "status": payload.status,
        "version": payload.version,
    });
    if canonical_wallet(&payload.address).as_deref() != Some(requested_address.as_str()) {
        return AdminCommerceLoad::Malformed;
    }
    decode_admin_wallet_detail_projection(projection)
        .map(AdminCommerceLoad::Ready)
        .unwrap_or(AdminCommerceLoad::Malformed)
}

pub(crate) async fn load_wallet_list(
    client: &epsx_client::ServiceClient,
    query: &AdminWalletListQuery,
    ctx: &epsx_client::RequestContext,
) -> AdminCommerceLoad<AdminWalletListProjection> {
    let Some(path) = wallet_list_path(query) else {
        return AdminCommerceLoad::Malformed;
    };
    let payload = match get_json::<BackendWalletList>(client, &path, ctx).await {
        Ok(payload) => payload,
        Err(error) => return error.into_load(),
    };
    let projection = serde_json::json!({
        "items": payload.items.into_iter().map(|item| serde_json::json!({
            "address": item.address,
            "chain_id": item.chain_id,
            "label": item.label,
            "role": item.role,
            "status": item.status,
            "version": item.version,
        })).collect::<Vec<_>>(),
        "total": payload.total,
        "limit": payload.limit,
        "offset": payload.offset,
    });
    match decode_admin_wallet_list_projection(projection) {
        Some(projection) if projection.items.is_empty() && projection.total == 0 => {
            AdminCommerceLoad::Empty
        }
        Some(projection) => AdminCommerceLoad::Ready(projection),
        None => AdminCommerceLoad::Malformed,
    }
}

fn wallet_list_path(query: &AdminWalletListQuery) -> Option<String> {
    let offset = query.offset()?;
    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    if let Some(status) = query.status.as_deref() {
        serializer.append_pair("status", status);
    }
    if let Some(search) = query.search.as_deref() {
        serializer.append_pair("search", search);
    }
    serializer.append_pair("limit", &query.limit.to_string());
    serializer.append_pair("offset", &offset.to_string());
    Some(format!("/api/v1/admin/wallets?{}", serializer.finish()))
}

pub(crate) async fn load_payment_user_access(
    client: &epsx_client::ServiceClient,
    query: &AdminPaymentUserAccessQuery,
    ctx: &epsx_client::RequestContext,
) -> AdminCommerceLoad<AdminPaymentUserAccessProjection> {
    let path = payment_user_access_path(query);
    let payload = match get_json::<BackendPaymentUserAccessEnvelope>(client, &path, ctx).await {
        Ok(payload) => payload,
        Err(error) => return error.into_load(),
    };
    if !payload.success {
        return AdminCommerceLoad::Malformed;
    }
    let projection = serde_json::json!({
        "items": payload.data.users.into_iter().map(|item| serde_json::json!({
            "wallet_address": item.wallet_address,
            "current_plan_id": item.current_plan_id,
            "plan_name": item.plan_name,
            "plan_expires_at": item.plan_expires_at,
            "days_remaining": item.days_remaining,
            "status": item.status,
        })).collect::<Vec<_>>(),
        "page": payload.data.pagination.page,
        "limit": payload.data.pagination.limit,
        "total_pages": payload.data.pagination.total_pages,
    });
    match decode_admin_payment_user_access_projection(projection) {
        Some(projection) if projection.items.is_empty() => AdminCommerceLoad::Empty,
        Some(projection) => AdminCommerceLoad::Ready(projection),
        None => AdminCommerceLoad::Malformed,
    }
}

fn payment_user_access_path(query: &AdminPaymentUserAccessQuery) -> String {
    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    serializer.append_pair("page", &query.page.to_string());
    serializer.append_pair("limit", &query.limit.to_string());
    if let Some(status) = query.status.as_deref() {
        serializer.append_pair("status", status);
    }
    if let Some(search) = query.search.as_deref() {
        serializer.append_pair("search", search);
    }
    format!("/api/admin/plans/user-access/list?{}", serializer.finish())
}

pub(crate) async fn load_credit_stats(
    client: &epsx_client::ServiceClient,
    ctx: &epsx_client::RequestContext,
) -> AdminCommerceLoad<AdminCreditStatsProjection> {
    let payload = match get_json::<BackendCreditStats>(client, CREDIT_STATS_PATH, ctx).await {
        Ok(payload) => payload,
        Err(error) => return error.into_load(),
    };
    let projection = serde_json::json!({
        "outstanding_minor": payload.outstanding_minor,
        "granted_today_minor": payload.granted_today_minor,
        "revoked_today_minor": payload.revoked_today_minor,
        "active_accounts": payload.active_accounts,
    });
    decode_admin_credit_stats_projection(projection)
        .map(AdminCommerceLoad::Ready)
        .unwrap_or(AdminCommerceLoad::Malformed)
}

pub(crate) async fn load_access(
    client: &epsx_client::ServiceClient,
    ctx: &epsx_client::RequestContext,
) -> AdminCommerceLoad<AdminAccessProjection> {
    let payload = match get_json::<BackendAccessList>(client, ACCESS_PATH, ctx).await {
        Ok(payload) => payload,
        Err(error) => return error.into_load(),
    };
    let projection = serde_json::json!({
        "items": payload
            .items
            .into_iter()
            .map(|item| {
                serde_json::json!({
                    "wallet_address": item.wallet_address,
                    "plan_id": item.plan_id,
                    "plan_name": item.plan_name,
                    "permission": item.permission,
                    "expires_at": item.expires_at,
                    "version": item.version,
                })
            })
            .collect::<Vec<_>>(),
    });
    match decode_admin_access_projection(projection) {
        Some(projection) => AdminCommerceLoad::Ready(projection),
        None => AdminCommerceLoad::Malformed,
    }
}

/// Load only assignments owned by one canonical wallet. The subscription
/// service performs the filtering; the admin UI never derives a wallet's
/// effective access from a global page of assignments.
pub(crate) async fn load_wallet_access(
    client: &epsx_client::ServiceClient,
    address: &str,
    ctx: &epsx_client::RequestContext,
) -> AdminCommerceLoad<AdminAccessProjection> {
    let Some(address) = canonical_wallet(address) else {
        return AdminCommerceLoad::Malformed;
    };
    let Some(path) = wallet_access_path(&address) else {
        return AdminCommerceLoad::Malformed;
    };
    let payload = match get_json::<BackendAccessList>(client, &path, ctx).await {
        Ok(payload) => payload,
        Err(error) => return error.into_load(),
    };
    if payload
        .items
        .iter()
        .any(|item| canonical_wallet(&item.wallet_address).as_deref() != Some(address.as_str()))
    {
        return AdminCommerceLoad::Malformed;
    }
    let projection = serde_json::json!({
        "items": payload
            .items
            .into_iter()
            .map(|item| {
                serde_json::json!({
                    "wallet_address": item.wallet_address,
                    "plan_id": item.plan_id,
                    "plan_name": item.plan_name,
                    "permission": item.permission,
                    "expires_at": item.expires_at,
                    "version": item.version,
                })
            })
            .collect::<Vec<_>>(),
    });
    decode_admin_access_projection(projection)
        .map(AdminCommerceLoad::Ready)
        .unwrap_or(AdminCommerceLoad::Malformed)
}

pub(crate) async fn load_plans(
    client: &epsx_client::ServiceClient,
    ctx: &epsx_client::RequestContext,
) -> AdminCommerceLoad<AdminPlanListProjection> {
    let payload = match get_json::<BackendPlanList>(client, PLANS_PATH, ctx).await {
        Ok(payload) => payload,
        Err(error) => return error.into_load(),
    };
    let projection = serde_json::json!({
        "items": payload
            .items
            .into_iter()
            .map(redact_plan)
            .collect::<Vec<_>>(),
        "total": payload.total,
        "limit": payload.limit,
        "offset": payload.offset,
    });
    match decode_admin_plan_list_projection(projection) {
        Some(projection) if projection.items.is_empty() && projection.total == 0 => {
            AdminCommerceLoad::Empty
        }
        Some(projection) => AdminCommerceLoad::Ready(projection),
        None => AdminCommerceLoad::Malformed,
    }
}

pub(crate) async fn load_plan_detail(
    client: &epsx_client::ServiceClient,
    plan_id: &str,
    ctx: &epsx_client::RequestContext,
) -> AdminCommerceLoad<AdminPlanProjection> {
    let Some(requested_plan_id) = canonical_uuid(plan_id) else {
        return AdminCommerceLoad::Malformed;
    };
    let Some(path) = plan_detail_path(plan_id) else {
        return AdminCommerceLoad::Malformed;
    };
    let payload = match get_json::<BackendPlan>(client, &path, ctx).await {
        Ok(payload) => payload,
        Err(error) => return error.into_load(),
    };
    if canonical_uuid(&payload.id).as_deref() != Some(requested_plan_id.as_str()) {
        return AdminCommerceLoad::Malformed;
    }
    decode_admin_plan_projection(redact_plan(payload))
        .map(AdminCommerceLoad::Ready)
        .unwrap_or(AdminCommerceLoad::Malformed)
}

pub(crate) async fn load_payment_links(
    client: &epsx_client::ServiceClient,
    ctx: &epsx_client::RequestContext,
) -> AdminCommerceLoad<AdminPaymentLinkListProjection> {
    let payload = match get_json::<BackendLinkList>(client, PAYMENT_LINKS_PATH, ctx).await {
        Ok(payload) => payload,
        Err(error) => return error.into_load(),
    };
    // An empty registry is authoritative when its bounded pagination envelope
    // is valid. There is no item projection to decode in this state, so do not
    // downgrade a strict zero-item response merely because an item-only
    // projection check has nothing to inspect.
    if payload.items.is_empty()
        && payload.total == 0
        && (1..=100).contains(&payload.limit)
        && (0..=10_000_000).contains(&payload.offset)
    {
        return AdminCommerceLoad::Empty;
    }
    let projection = serde_json::json!({
        "items": payload
            .items
            .into_iter()
            .map(|item| {
                serde_json::json!({
                    "id": item.id,
                    "slug": item.slug,
                    "max_uses": item.max_uses,
                    "current_uses": item.current_uses,
                    "expires_at": item.expires_at,
                    "status": item.status,
                    "version": item.version,
                })
            })
            .collect::<Vec<_>>(),
        "total": payload.total,
        "limit": payload.limit,
        "offset": payload.offset,
    });
    match decode_admin_payment_link_list_projection(projection) {
        Some(projection) if projection.items.is_empty() && projection.total == 0 => {
            AdminCommerceLoad::Empty
        }
        Some(projection) => AdminCommerceLoad::Ready(projection),
        None => AdminCommerceLoad::Malformed,
    }
}

pub(crate) async fn load_payment_links_monolith(
    client: &epsx_client::ServiceClient,
    ctx: &epsx_client::RequestContext,
) -> AdminCommerceLoad<AdminPaymentLinkListProjection> {
    let payload = match get_json::<MonolithLinkList>(client, MONOLITH_PAYMENT_LINKS_PATH, ctx).await
    {
        Ok(payload) => payload,
        Err(error) => return error.into_load(),
    };
    if payload.payment_links.is_empty()
        && payload.total == 0
        && (1..=100).contains(&payload.limit)
        && (0..=10_000_000).contains(&payload.offset)
    {
        return AdminCommerceLoad::Empty;
    }
    let projection = serde_json::json!({
        "items": payload
            .payment_links
            .into_iter()
            .map(|item| {
                serde_json::json!({
                    "id": item.id,
                    "slug": item.slug,
                    "max_uses": item.max_uses.unwrap_or(0),
                    "current_uses": item.current_uses,
                    "expires_at": item.expires_at,
                    "status": if item.is_active { "active" } else { "disabled" },
                    "version": 0,
                })
            })
            .collect::<Vec<_>>(),
        "total": payload.total,
        "limit": payload.limit,
        "offset": payload.offset,
    });
    match decode_admin_payment_link_list_projection(projection) {
        Some(projection) if projection.items.is_empty() && projection.total == 0 => {
            AdminCommerceLoad::Empty
        }
        Some(projection) => AdminCommerceLoad::Ready(projection),
        None => AdminCommerceLoad::Malformed,
    }
}

fn redact_plan(plan: BackendPlan) -> Value {
    serde_json::json!({
        "id": plan.id,
        "name": plan.name,
        "description": plan.description,
        "amount": plan.amount,
        "currency": plan.currency,
        "chain_id": plan.chain_id,
        "interval": plan.interval,
        "active": plan.active,
        "version": plan.version,
    })
}

async fn try_get_json<T: DeserializeOwned>(
    client: &epsx_client::ServiceClient,
    path: &str,
    ctx: &epsx_client::RequestContext,
) -> Result<T, UpstreamError> {
    let Some(token) = ctx
        .auth_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
    else {
        return Err(UpstreamError::Unavailable);
    };
    let http_client = reqwest::Client::builder()
        .timeout(client.config().timeout)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| UpstreamError::Unavailable)?;
    let response = http_client
        .get(format!(
            "{}{}",
            client.base_url().trim_end_matches('/'),
            path
        ))
        .header("x-request-id", ctx.request_id.to_string())
        .bearer_auth(token)
        .send()
        .await
        .map_err(|_| UpstreamError::Unavailable)?;
    if !response.status().is_success() {
        return Err(UpstreamFailure::classify(response.status()).into());
    }
    let body = read_body_limited(response).await?;
    serde_json::from_slice(&body).map_err(|_| UpstreamError::Malformed)
}

async fn get_json<T: DeserializeOwned>(
    client: &epsx_client::ServiceClient,
    path: &str,
    ctx: &epsx_client::RequestContext,
) -> Result<T, UpstreamError> {
    let primary = try_get_json(client, path, ctx).await;
    // Production wallet/subscription/pay routes are mounted at `/api/v1/admin/*`
    // on the extracted services but the monolith fallback remains at `/api/admin/*`.
    // When the BFF's service client points at the monolith (the default when
    // `WALLET_SERVICE_URL` etc. are unset) the `/api/v1` prefix 404s as
    // `Unavailable`.  Retry once with the alternate prefix so the same BFF
    // can serve rows whether the extracted service or the monolith is the
    // upstream.  Only retry on `Unavailable` (which includes 404) — permission
    // (`Forbidden`/`Unauthorized`) and contract (`Malformed`) failures are
    // authoritative and must not be masked by a fallback.
    if !matches!(primary, Err(UpstreamError::Unavailable)) {
        return primary;
    }
    let alt = if let Some(rest) = path.strip_prefix("/api/v1") {
        format!("/api{rest}")
    } else if let Some(rest) = path.strip_prefix("/api") {
        format!("/api/v1{rest}")
    } else {
        return primary;
    };
    if alt == path {
        return primary;
    }
    match try_get_json(client, &alt, ctx).await {
        Ok(value) => Ok(value),
        Err(_) => primary,
    }
}

/// Decode the backend-owned admin envelope before any route projection is
/// attempted. Raw DTOs and unsuccessful envelopes are never accepted.
#[cfg(test)]
pub(crate) fn decode_admin_envelope<T: DeserializeOwned>(body: &[u8]) -> Result<T, ()> {
    let envelope: AdminApiResponseEnvelope = serde_json::from_slice(body).map_err(|_| ())?;
    if !envelope.success
        || envelope.data.is_none()
        || envelope.error.is_some()
        || envelope.message.trim().is_empty()
        || envelope.message.chars().count() > 500
        || envelope.message.chars().any(char::is_control)
        || DateTime::parse_from_rfc3339(&envelope.timestamp).is_err()
    {
        return Err(());
    }
    let _admin_meta = envelope.admin_meta;
    serde_json::from_value(envelope.data.unwrap()).map_err(|_| ())
}

/// Send the wallet service's raw, evidence-bearing status mutation. Wallet
/// service admin routes intentionally return their own strict DTO rather than
/// the monolith admin envelope used by legacy mutations.
pub(crate) async fn send_wallet_status_mutation(
    client: &epsx_client::ServiceClient,
    path: &str,
    command: &WalletStatusCommand,
    idempotency_key: &str,
    ctx: &epsx_client::RequestContext,
) -> AdminCommerceMutationLoad<AdminWalletDetailProjection> {
    if path.is_empty()
        || !(1..=128).contains(&idempotency_key.len())
        || !idempotency_key
            .bytes()
            .all(|byte| byte.is_ascii_graphic() && byte != b' ')
        || command.expected_version < 0
        || command.reason.trim().is_empty()
        || command.reason.chars().count() > 500
        || command.reason.chars().any(char::is_control)
    {
        return AdminCommerceMutationLoad::Malformed;
    }
    let Some(token) = ctx
        .auth_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
    else {
        return AdminCommerceMutationLoad::Unavailable;
    };
    let http_client = match reqwest::Client::builder()
        .timeout(client.config().timeout)
        .redirect(reqwest::redirect::Policy::none())
        .build()
    {
        Ok(client) => client,
        Err(_) => return AdminCommerceMutationLoad::Unavailable,
    };
    let request = match http_client
        .post(format!(
            "{}{}",
            client.base_url().trim_end_matches('/'),
            path
        ))
        .header("x-request-id", ctx.request_id.to_string())
        .header("idempotency-key", idempotency_key)
        .bearer_auth(token)
        .json(command)
        .build()
    {
        Ok(request) => request,
        Err(_) => return AdminCommerceMutationLoad::Unavailable,
    };
    let response = match http_client.execute(request).await {
        Ok(response) => response,
        Err(_) => return AdminCommerceMutationLoad::Unavailable,
    };
    let status = response.status();
    if status == reqwest::StatusCode::FORBIDDEN {
        return AdminCommerceMutationLoad::Forbidden;
    }
    if status == reqwest::StatusCode::CONFLICT {
        return AdminCommerceMutationLoad::Conflict;
    }
    if !status.is_success() {
        return if status == reqwest::StatusCode::UNAUTHORIZED {
            AdminCommerceMutationLoad::Unauthorized
        } else {
            AdminCommerceMutationLoad::Malformed
        };
    }
    let body = match read_body_limited(response).await {
        Ok(body) => body,
        Err(_) => return AdminCommerceMutationLoad::Unavailable,
    };
    let response: BackendWalletMutation = match serde_json::from_slice(&body) {
        Ok(response) => response,
        Err(_) => return AdminCommerceMutationLoad::Malformed,
    };
    if response.evidence.operation_id.trim().is_empty()
        || response.evidence.version < 0
        || response.evidence.observed_at.trim().is_empty()
        || DateTime::parse_from_rfc3339(&response.evidence.observed_at).is_err()
        || response.correlation_id.trim().is_empty()
    {
        return AdminCommerceMutationLoad::Malformed;
    }
    let projection = AdminWalletDetailProjection {
        address: response.wallet.address,
        chain_id: response.wallet.chain_id,
        label: response.wallet.label,
        role: response.wallet.role,
        status: response.wallet.status,
        version: response.wallet.version,
    };
    if decode_admin_wallet_detail_projection(
        serde_json::to_value(&projection).unwrap_or(Value::Null),
    )
    .is_none()
    {
        return AdminCommerceMutationLoad::Malformed;
    }
    AdminCommerceMutationLoad::Ready(projection)
}

async fn read_body_limited(mut response: reqwest::Response) -> Result<Vec<u8>, UpstreamError> {
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
    {
        return Err(UpstreamError::Unavailable);
    }
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| UpstreamError::Unavailable)?
    {
        let next_len = body
            .len()
            .checked_add(chunk.len())
            .ok_or(UpstreamError::Unavailable)?;
        if next_len > MAX_RESPONSE_BYTES {
            return Err(UpstreamError::Unavailable);
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

impl UpstreamError {
    fn into_load<T>(self) -> AdminCommerceLoad<T> {
        match self {
            Self::Forbidden => AdminCommerceLoad::Forbidden,
            Self::Unauthorized => AdminCommerceLoad::Unauthorized,
            Self::Unavailable => AdminCommerceLoad::Unavailable,
            Self::Malformed => AdminCommerceLoad::Malformed,
        }
    }
}

fn canonical_wallet(value: &str) -> Option<String> {
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

fn canonical_uuid(value: &str) -> Option<String> {
    let uuid = uuid::Uuid::parse_str(value).ok()?;
    (value.len() == 36 && !value.contains('/') && !value.contains('%')).then(|| uuid.to_string())
}

#[cfg(test)]
fn valid_resource_id(value: &str) -> bool {
    (1..=66).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() || matches!(byte, b'x' | b'-' | b'_'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use epsx_dioxus_ui::pages::admin_pages::{
        payments::{
            AdminPaymentLinkProjection, ADMIN_PAYMENT_LINKS_EMPTY, ADMIN_PAYMENT_LINKS_FORBIDDEN,
            ADMIN_PAYMENT_LINKS_MALFORMED, ADMIN_PAYMENT_LINKS_READY,
            ADMIN_PAYMENT_LINKS_UNAVAILABLE,
        },
        wallet_access::{
            ADMIN_ACCESS_FORBIDDEN, ADMIN_ACCESS_MALFORMED, ADMIN_ACCESS_READY,
            ADMIN_ACCESS_UNAVAILABLE,
        },
        wallet_credits::{
            ADMIN_CREDITS_FORBIDDEN, ADMIN_CREDITS_MALFORMED, ADMIN_CREDITS_READY,
            ADMIN_CREDITS_UNAVAILABLE,
        },
        wallet_plans::{
            ADMIN_PLANS_EMPTY, ADMIN_PLANS_FORBIDDEN, ADMIN_PLANS_MALFORMED, ADMIN_PLANS_READY,
            ADMIN_PLANS_UNAVAILABLE, ADMIN_PLAN_DETAIL_STATE_PARAM,
        },
        wallet_wallets::{
            ADMIN_WALLET_DETAIL_FORBIDDEN, ADMIN_WALLET_DETAIL_MALFORMED,
            ADMIN_WALLET_DETAIL_READY, ADMIN_WALLET_DETAIL_UNAVAILABLE,
        },
    };

    const ADDRESS: &str = "0x1111111111111111111111111111111111111111";
    const PLAN_ID: &str = "00000000-0000-0000-0000-000000000001";

    #[test]
    fn dynamic_paths_require_canonical_single_segments() {
        assert_eq!(
            wallet_detail_path("0x1111111111111111111111111111111111111111").as_deref(),
            Some("/api/v1/admin/wallets/0x1111111111111111111111111111111111111111")
        );
        assert_eq!(
            wallet_access_path(ADDRESS).as_deref(),
            Some("/api/v1/admin/subscription/access?wallet_address=0x1111111111111111111111111111111111111111&limit=100&offset=0")
        );
        assert_eq!(
            plan_detail_path(PLAN_ID).as_deref(),
            Some("/api/v1/admin/subscription/plans/00000000-0000-0000-0000-000000000001")
        );
        for invalid in [
            "0x111111111111111111111111111111111111111", // short
            "0x1111111111111111111111111111111111111111foo",
            "0x1111111111111111111111111111111111111111/../x",
            "0x1111111111111111111111111111111111111111%2f",
        ] {
            assert!(wallet_detail_path(invalid).is_none(), "accepted {invalid}");
            assert!(wallet_access_path(invalid).is_none(), "accepted {invalid}");
        }
        for invalid in [
            "00000000-0000-0000-0000-000000000001/../x",
            "00000000-0000-0000-0000-000000000001%2f",
            "00000000-0000-0000-0000-00000000001",
        ] {
            assert!(plan_detail_path(invalid).is_none(), "accepted {invalid}");
        }
        assert_eq!(canonical_wallet(ADDRESS), Some(ADDRESS.to_string()));
    }

    #[test]
    fn wallet_list_path_forwards_only_the_closed_backend_query() {
        let query = AdminWalletListQuery::from_raw("search=0x1111&status=disabled&limit=25&page=3")
            .expect("valid wallet query");
        assert_eq!(
            wallet_list_path(&query).as_deref(),
            Some("/api/v1/admin/wallets?status=disabled&search=0x1111&limit=25&offset=50")
        );
        assert!(AdminWalletListQuery::from_raw("platform=analytics").is_err());
    }

    #[test]
    fn payment_user_access_path_forwards_the_production_backend_contract() {
        let query = AdminPaymentUserAccessQuery::from_raw(
            "tab=user-access&page=3&limit=20&status=active&search=0x1111",
        )
        .expect("valid user-access query");
        assert_eq!(
            payment_user_access_path(&query),
            "/api/admin/plans/user-access/list?page=3&limit=20&status=active&search=0x1111"
        );
    }

    #[test]
    fn mutation_paths_split_exact_operations_and_reject_boundaries() {
        assert_eq!(
            wallet_status_mutation_path(ADDRESS, false).as_deref(),
            Some("/api/v1/admin/wallets/0x1111111111111111111111111111111111111111/disable")
        );
        assert_eq!(
            credit_mutation_path(ADDRESS, "grant").as_deref(),
            Some("/api/v1/admin/credits/0x1111111111111111111111111111111111111111/grant")
        );
        assert_eq!(
            access_mutation_path("assign"),
            Some("/api/v1/admin/subscription/access/assign")
        );
        assert_eq!(
            plan_mutation_path(None).as_deref(),
            Some("/api/v1/admin/subscription/plans")
        );
        assert_eq!(
            payment_link_mutation_path(None).as_deref(),
            Some("/api/v1/admin/pay/links")
        );
        assert!(wallet_status_mutation_path(
            "0x1111111111111111111111111111111111111111foo",
            false
        )
        .is_none());
        assert!(
            credit_mutation_path("0x1111111111111111111111111111111111111111%2f", "grant")
                .is_none()
        );
        assert!(payment_intent_cancel_path("intent.id").is_none());
        assert!(payment_link_mutation_path(Some("linksfoo")).is_none());
        assert!(access_mutation_path("assignfoo").is_none());
    }

    #[test]
    fn empty_payment_link_inventory_is_authoritative() {
        let projection = serde_json::json!({
            "items": [],
            "total": 0,
            "limit": 100,
            "offset": 0,
        });
        let decoded = decode_admin_payment_link_list_projection(projection);
        assert!(decoded.is_some());
        let decoded = decoded.expect("empty payment-link projection");
        assert!(decoded.items.is_empty());
        assert_eq!(decoded.total, 0);
    }

    #[test]
    fn admin_envelope_requires_success_data_timestamp_and_exact_fields() {
        let body = serde_json::json!({
            "success": true,
            "data": {"value": 7},
            "error": null,
            "message": "Mutation applied",
            "timestamp": "2026-07-27T00:00:00Z",
            "admin_meta": {"operation": "wallet.disable"}
        });
        let decoded: serde_json::Value =
            decode_admin_envelope(body.to_string().as_bytes()).unwrap();
        assert_eq!(decoded["value"], 7);

        let raw = serde_json::json!({"value": 7});
        assert!(decode_admin_envelope::<serde_json::Value>(raw.to_string().as_bytes()).is_err());

        let mut failed = body.clone();
        failed["success"] = serde_json::json!(false);
        failed["data"] = serde_json::Value::Null;
        failed["error"] = serde_json::json!("forbidden");
        assert!(decode_admin_envelope::<serde_json::Value>(failed.to_string().as_bytes()).is_err());

        let mut unknown = body;
        unknown["unexpected"] = serde_json::json!(true);
        assert!(
            decode_admin_envelope::<serde_json::Value>(unknown.to_string().as_bytes()).is_err()
        );
    }

    #[test]
    fn projection_states_are_explicit_and_redaction_drops_identity_fields() {
        let projection = decode_admin_payment_link_list_projection(serde_json::json!({
            "items": [{
                "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                "slug": "epsx-public",
                "max_uses": 1,
                "current_uses": 0,
                "expires_at": null,
                "status": "active",
                "version": 0
            }],
            "total": 1,
            "limit": 100,
            "offset": 0
        }))
        .unwrap();
        assert_eq!(projection.items[0].slug, "epsx-public");
        assert!(!serde_json::to_value(&projection)
            .unwrap()
            .to_string()
            .contains("intent_id"));
        assert_eq!(ADMIN_PAYMENT_LINKS_READY, "ready");
        assert_eq!(ADMIN_PAYMENT_LINKS_EMPTY, "empty");
        assert_eq!(ADMIN_PAYMENT_LINKS_FORBIDDEN, "forbidden");
        assert_eq!(ADMIN_PAYMENT_LINKS_UNAVAILABLE, "unavailable");
        assert_eq!(ADMIN_PAYMENT_LINKS_MALFORMED, "malformed");
        assert_eq!(ADMIN_ACCESS_READY, "ready");
        assert_eq!(ADMIN_ACCESS_FORBIDDEN, "forbidden");
        assert_eq!(ADMIN_ACCESS_UNAVAILABLE, "unavailable");
        assert_eq!(ADMIN_ACCESS_MALFORMED, "malformed");
        assert_eq!(ADMIN_CREDITS_READY, "ready");
        assert_eq!(ADMIN_CREDITS_FORBIDDEN, "forbidden");
        assert_eq!(ADMIN_CREDITS_UNAVAILABLE, "unavailable");
        assert_eq!(ADMIN_CREDITS_MALFORMED, "malformed");
        assert_eq!(ADMIN_PLANS_READY, "ready");
        assert_eq!(ADMIN_PLANS_EMPTY, "empty");
        assert_eq!(ADMIN_PLANS_FORBIDDEN, "forbidden");
        assert_eq!(ADMIN_PLANS_UNAVAILABLE, "unavailable");
        assert_eq!(ADMIN_PLANS_MALFORMED, "malformed");
        assert_eq!(ADMIN_WALLET_DETAIL_READY, "ready");
        assert_eq!(ADMIN_WALLET_DETAIL_FORBIDDEN, "forbidden");
        assert_eq!(ADMIN_WALLET_DETAIL_UNAVAILABLE, "unavailable");
        assert_eq!(ADMIN_WALLET_DETAIL_MALFORMED, "malformed");
        assert_eq!(
            ADMIN_PLAN_DETAIL_STATE_PARAM,
            "data_admin_plan_detail_state"
        );
        let _ = AdminPaymentLinkProjection {
            id: projection.items[0].id.clone(),
            slug: projection.items[0].slug.clone(),
            max_uses: projection.items[0].max_uses,
            current_uses: projection.items[0].current_uses,
            expires_at: projection.items[0].expires_at.clone(),
            status: projection.items[0].status.clone(),
            version: projection.items[0].version,
        };
    }
}
