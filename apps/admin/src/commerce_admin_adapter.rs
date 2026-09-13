//! Strict read adapters for admin commerce projections.
//!
//! Service ownership stays explicit here: wallet reads come from the wallet
//! service, plans/access from subscription, and payment links from pay. The
//! adapters discard correlation, ownership, metadata, and mutation evidence
//! before values enter PageContext.

use epsx_client::{ClientError, RequestContext, ServiceClient};
use epsx_dioxus_ui::pages::admin_pages::payments::{
    decode_admin_payment_link_list_projection, AdminPaymentLinkListProjection,
};
use epsx_dioxus_ui::pages::admin_pages::wallet_access::{
    decode_admin_access_projection, AdminAccessProjection,
};
use epsx_dioxus_ui::pages::admin_pages::wallet_credits::{
    decode_admin_credit_stats_projection, AdminCreditStatsProjection,
};
use epsx_dioxus_ui::pages::admin_pages::wallet_plans::{
    decode_admin_plan_list_projection, decode_admin_plan_projection, AdminPlanListProjection,
    AdminPlanProjection,
};
use epsx_dioxus_ui::pages::admin_pages::wallet_wallets::{
    decode_admin_wallet_detail_projection, decode_admin_wallet_stats_projection,
    AdminWalletDetailProjection, AdminWalletStatsSummary,
};
use serde::Deserialize;
use serde_json::json;

const MAX_RESPONSE_BYTES: usize = 512 * 1024;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum CommerceLoad<T> {
    Ready(T),
    Empty,
    Forbidden,
    Unavailable,
    Malformed,
}

fn classify_error<T>(error: ClientError) -> CommerceLoad<T> {
    match error {
        ClientError::UpstreamStatus(401 | 403) | ClientError::Unauthorized => {
            CommerceLoad::Forbidden
        }
        _ => CommerceLoad::Unavailable,
    }
}

fn bounded(value: &serde_json::Value) -> bool {
    serde_json::to_vec(value)
        .map(|bytes| bytes.len() <= MAX_RESPONSE_BYTES)
        .unwrap_or(false)
}

fn valid_wallet(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 42
        && bytes[0] == b'0'
        && matches!(bytes[1], b'x' | b'X')
        && bytes[2..].iter().all(u8::is_ascii_hexdigit)
}

fn canonical_wallet(value: &str) -> Option<String> {
    valid_wallet(value).then(|| format!("0x{}", &value[2..].to_ascii_lowercase()))
}

fn valid_uuid(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                byte == b'-'
            } else {
                byte.is_ascii_hexdigit()
            }
        })
}

pub(crate) async fn load_wallet_stats(
    client: &ServiceClient,
    ctx: &RequestContext,
) -> CommerceLoad<AdminWalletStatsSummary> {
    let value = match client
        .get_with_ctx("/api/v1/admin/wallets/stats", ctx)
        .await
    {
        Ok(value) => value,
        Err(error) => return classify_error(error),
    };
    if !bounded(&value) {
        return CommerceLoad::Malformed;
    }
    let raw: WalletStatsResponse = match serde_json::from_value(value) {
        Ok(raw) => raw,
        Err(_) => return CommerceLoad::Malformed,
    };
    let projection = json!({
        "total_users": raw.total,
        "active_users": raw.active,
        "inactive_users": raw.disabled,
        "new_users_30_days": raw.new_30_days,
    });
    match decode_admin_wallet_stats_projection(projection) {
        Some(projection) => CommerceLoad::Ready(projection),
        None => CommerceLoad::Malformed,
    }
}

pub(crate) async fn load_wallet_detail(
    client: &ServiceClient,
    address: &str,
    ctx: &RequestContext,
) -> CommerceLoad<AdminWalletDetailProjection> {
    let Some(address) = canonical_wallet(address) else {
        return CommerceLoad::Malformed;
    };
    let path = format!("/api/v1/admin/wallets/{address}");
    let value = match client.get_with_ctx(&path, ctx).await {
        Ok(value) => value,
        Err(error) => return classify_error(error),
    };
    if !bounded(&value) {
        return CommerceLoad::Malformed;
    }
    let raw: WalletDetailResponse = match serde_json::from_value(value) {
        Ok(raw) => raw,
        Err(_) => return CommerceLoad::Malformed,
    };
    if canonical_wallet(&raw.address) != Some(address) {
        return CommerceLoad::Malformed;
    }
    match decode_admin_wallet_detail_projection(json!({
        "address": raw.address,
        "chain_id": raw.chain_id,
        "label": raw.label,
        "role": raw.role,
        "status": raw.status,
        "version": raw.version,
    })) {
        Some(projection) => CommerceLoad::Ready(projection),
        None => CommerceLoad::Malformed,
    }
}

pub(crate) async fn load_credit_stats(
    client: &ServiceClient,
    ctx: &RequestContext,
) -> CommerceLoad<AdminCreditStatsProjection> {
    let value = match client.get_with_ctx("/api/v1/admin/credits", ctx).await {
        Ok(value) => value,
        Err(error) => return classify_error(error),
    };
    if !bounded(&value) {
        return CommerceLoad::Malformed;
    }
    let raw: CreditStatsResponse = match serde_json::from_value(value) {
        Ok(raw) => raw,
        Err(_) => return CommerceLoad::Malformed,
    };
    match decode_admin_credit_stats_projection(json!({
        "outstanding_minor": raw.outstanding_minor,
        "granted_today_minor": raw.granted_today_minor,
        "revoked_today_minor": raw.revoked_today_minor,
        "active_accounts": raw.active_accounts,
    })) {
        Some(projection) => CommerceLoad::Ready(projection),
        None => CommerceLoad::Malformed,
    }
}

pub(crate) async fn load_access(
    client: &ServiceClient,
    ctx: &RequestContext,
) -> CommerceLoad<AdminAccessProjection> {
    let value = match client
        .get_with_ctx("/api/v1/admin/subscription/access?limit=100&offset=0", ctx)
        .await
    {
        Ok(value) => value,
        Err(error) => return classify_error(error),
    };
    if !bounded(&value) {
        return CommerceLoad::Malformed;
    }
    let raw: AccessResponse = match serde_json::from_value(value) {
        Ok(raw) => raw,
        Err(_) => return CommerceLoad::Malformed,
    };
    let items = raw
        .items
        .into_iter()
        .map(|item| {
            json!({
                "plan_id": item.plan_id,
                "plan_name": item.plan_name,
                "permission": item.permission,
                "expires_at": item.expires_at,
            })
        })
        .collect::<Vec<_>>();
    match decode_admin_access_projection(json!({ "items": items })) {
        Some(projection) => CommerceLoad::Ready(projection),
        None => CommerceLoad::Malformed,
    }
}

pub(crate) async fn load_plans(
    client: &ServiceClient,
    ctx: &RequestContext,
) -> CommerceLoad<AdminPlanListProjection> {
    let value = match client
        .get_with_ctx("/api/v1/admin/subscription/plans?limit=100&offset=0", ctx)
        .await
    {
        Ok(value) => value,
        Err(error) => return classify_error(error),
    };
    if !bounded(&value) {
        return CommerceLoad::Malformed;
    }
    let raw: PlanListResponse = match serde_json::from_value(value) {
        Ok(raw) => raw,
        Err(_) => return CommerceLoad::Malformed,
    };
    let items = raw.items.into_iter().map(project_plan).collect::<Vec<_>>();
    let projection = json!({
        "items": items,
        "total": raw.total,
        "limit": raw.limit,
        "offset": raw.offset,
    });
    match decode_admin_plan_list_projection(projection) {
        Some(projection) if projection.items.is_empty() && projection.total == 0 => {
            CommerceLoad::Empty
        }
        Some(projection) => CommerceLoad::Ready(projection),
        None => CommerceLoad::Malformed,
    }
}

pub(crate) async fn load_plan_detail(
    client: &ServiceClient,
    plan_id: &str,
    ctx: &RequestContext,
) -> CommerceLoad<AdminPlanProjection> {
    if !valid_uuid(plan_id) {
        return CommerceLoad::Malformed;
    }
    let path = format!(
        "/api/v1/admin/subscription/plans/{}",
        plan_id.to_ascii_lowercase()
    );
    let value = match client.get_with_ctx(&path, ctx).await {
        Ok(value) => value,
        Err(error) => return classify_error(error),
    };
    if !bounded(&value) {
        return CommerceLoad::Malformed;
    }
    let raw: PlanResponse = match serde_json::from_value(value) {
        Ok(raw) => raw,
        Err(_) => return CommerceLoad::Malformed,
    };
    let projection = project_plan(raw);
    match decode_admin_plan_projection(projection) {
        Some(projection) => CommerceLoad::Ready(projection),
        None => CommerceLoad::Malformed,
    }
}

pub(crate) async fn load_payment_links(
    client: &ServiceClient,
    ctx: &RequestContext,
) -> CommerceLoad<AdminPaymentLinkListProjection> {
    let value = match client
        .get_with_ctx("/api/v1/admin/pay/links?limit=100&offset=0", ctx)
        .await
    {
        Ok(value) => value,
        Err(error) => return classify_error(error),
    };
    if !bounded(&value) {
        return CommerceLoad::Malformed;
    }
    let raw: LinkListResponse = match serde_json::from_value(value) {
        Ok(raw) => raw,
        Err(_) => return CommerceLoad::Malformed,
    };
    let items = raw
        .items
        .into_iter()
        .map(|item| {
            json!({
                "id": item.id,
                "slug": item.slug,
                "max_uses": item.max_uses,
                "current_uses": item.current_uses,
                "expires_at": item.expires_at,
                "status": item.status,
                "version": item.version,
            })
        })
        .collect::<Vec<_>>();
    let projection = json!({
        "items": items,
        "total": raw.total,
        "limit": raw.limit,
        "offset": raw.offset,
    });
    match decode_admin_payment_link_list_projection(projection) {
        Some(projection) if projection.items.is_empty() && projection.total == 0 => {
            CommerceLoad::Empty
        }
        Some(projection) => CommerceLoad::Ready(projection),
        None => CommerceLoad::Malformed,
    }
}

fn project_plan(raw: PlanResponse) -> serde_json::Value {
    json!({
        "id": raw.id,
        "name": raw.name,
        "description": raw.description,
        "amount": raw.amount,
        "currency": raw.currency,
        "chain_id": raw.chain_id,
        "interval": raw.interval,
        "active": raw.active,
        "version": raw.version,
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WalletStatsResponse {
    total: i64,
    active: i64,
    disabled: i64,
    new_30_days: i64,
    correlation_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WalletDetailResponse {
    address: String,
    chain_id: String,
    label: Option<String>,
    role: Option<String>,
    status: String,
    metadata: serde_json::Value,
    version: i64,
    created_at: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreditStatsResponse {
    outstanding_minor: i64,
    granted_today_minor: i64,
    revoked_today_minor: i64,
    active_accounts: i64,
    correlation_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AccessResponse {
    items: Vec<AccessItem>,
    correlation_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AccessItem {
    wallet_address: String,
    plan_id: String,
    plan_name: String,
    permission: String,
    expires_at: Option<String>,
    version: i64,
    assigned_by: String,
    updated_at: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlanListResponse {
    items: Vec<PlanResponse>,
    total: i64,
    limit: i64,
    offset: i64,
    correlation_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlanResponse {
    id: String,
    merchant_id: String,
    name: String,
    description: Option<String>,
    amount: String,
    currency: String,
    chain_id: String,
    interval: i32,
    active: Option<bool>,
    created_at: Option<String>,
    version: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LinkListResponse {
    items: Vec<LinkItem>,
    total: i64,
    limit: i64,
    offset: i64,
    correlation_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LinkItem {
    id: String,
    slug: String,
    intent_id: String,
    max_uses: i32,
    current_uses: i32,
    expires_at: Option<String>,
    created_at: String,
    status: String,
    version: i64,
}
