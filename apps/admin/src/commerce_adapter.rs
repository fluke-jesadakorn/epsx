//! Route-owned BFF loaders for the admin commerce/wallet read surfaces.
//!
//! This module is intentionally standalone: the central SSR/router wiring is
//! owned outside the commerce slice. Each loader sends the verified bearer and
//! request ID to the service that owns the record, rejects non-canonical route
//! identifiers before network I/O, and projects only the fields accepted by
//! the corresponding Dioxus page.

use epsx_dioxus_ui::pages::admin_pages::{
    payments::{
        decode_admin_payment_link_list_projection, AdminPaymentLinkListProjection,
        AdminPaymentLinkProjection, ADMIN_PAYMENT_LINKS_EMPTY, ADMIN_PAYMENT_LINKS_FORBIDDEN,
        ADMIN_PAYMENT_LINKS_MALFORMED, ADMIN_PAYMENT_LINKS_READY, ADMIN_PAYMENT_LINKS_UNAVAILABLE,
    },
    wallet_access::{
        decode_admin_access_projection, AdminAccessProjection, ADMIN_ACCESS_FORBIDDEN,
        ADMIN_ACCESS_MALFORMED, ADMIN_ACCESS_READY, ADMIN_ACCESS_UNAVAILABLE,
    },
    wallet_credits::{
        decode_admin_credit_stats_projection, AdminCreditStatsProjection, ADMIN_CREDITS_FORBIDDEN,
        ADMIN_CREDITS_MALFORMED, ADMIN_CREDITS_READY, ADMIN_CREDITS_UNAVAILABLE,
    },
    wallet_plans::{
        decode_admin_plan_list_projection, decode_admin_plan_projection, AdminPlanListProjection,
        AdminPlanProjection, ADMIN_PLANS_EMPTY, ADMIN_PLANS_FORBIDDEN, ADMIN_PLANS_MALFORMED,
        ADMIN_PLANS_READY, ADMIN_PLANS_UNAVAILABLE, ADMIN_PLAN_DETAIL_STATE_PARAM,
    },
    wallet_wallets::{
        decode_admin_wallet_detail_projection, AdminWalletDetailProjection,
        ADMIN_WALLET_DETAIL_FORBIDDEN, ADMIN_WALLET_DETAIL_MALFORMED, ADMIN_WALLET_DETAIL_READY,
        ADMIN_WALLET_DETAIL_UNAVAILABLE,
    },
};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;

const MAX_RESPONSE_BYTES: usize = 512 * 1024;
const WALLET_DETAIL_PREFIX: &str = "/api/v1/admin/wallets/";
const CREDIT_STATS_PATH: &str = "/api/v1/admin/credits";
const ACCESS_PATH: &str = "/api/v1/admin/subscription/access?limit=1000&offset=0";
const PLANS_PATH: &str = "/api/v1/admin/subscription/plans?limit=100&offset=0";
const PAYMENT_LINKS_PATH: &str = "/api/v1/admin/pay/links?limit=100&offset=0";
const PLAN_DETAIL_PREFIX: &str = "/api/v1/admin/subscription/plans/";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AdminCommerceLoad<T> {
    Ready(T),
    Empty,
    Forbidden,
    Unavailable,
    Malformed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UpstreamError {
    Forbidden,
    Unavailable,
    Malformed,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendWallet {
    address: String,
    chain_id: String,
    label: Option<String>,
    role: Option<String>,
    status: String,
    metadata: Value,
    version: i64,
    created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendCreditStats {
    outstanding_minor: i64,
    granted_today_minor: i64,
    revoked_today_minor: i64,
    active_accounts: i64,
    correlation_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendAccessList {
    items: Vec<BackendAccessAssignment>,
    correlation_id: String,
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
    assigned_by: String,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendPlanList {
    items: Vec<BackendPlan>,
    total: i64,
    limit: i64,
    offset: i64,
    correlation_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendPlan {
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendLinkList {
    items: Vec<BackendLink>,
    total: i64,
    limit: i64,
    offset: i64,
    correlation_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendLink {
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

pub(crate) fn wallet_detail_path(address: &str) -> Option<String> {
    canonical_wallet(address).map(|address| format!("{WALLET_DETAIL_PREFIX}{address}"))
}

pub(crate) fn plan_detail_path(plan_id: &str) -> Option<String> {
    canonical_uuid(plan_id).map(|plan_id| format!("{PLAN_DETAIL_PREFIX}{plan_id}"))
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
    let projection = serde_json::json!({
        "address": payload.address,
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
                    "plan_id": item.plan_id,
                    "plan_name": item.plan_name,
                    "permission": item.permission,
                    "expires_at": item.expires_at,
                })
            })
            .collect::<Vec<_>>(),
    });
    match decode_admin_access_projection(projection) {
        Some(projection) => AdminCommerceLoad::Ready(projection),
        None => AdminCommerceLoad::Malformed,
    }
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
    let projection = serde_json::json!({
        "items": payload
            .items
            .into_iter()
            .map(|item| {
                serde_json::json!({
                    "slug": item.slug,
                    "max_uses": item.max_uses,
                    "current_uses": item.current_uses,
                    "expires_at": item.expires_at,
                    "status": item.status,
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

async fn get_json<T: DeserializeOwned>(
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
        return Err(match response.status() {
            reqwest::StatusCode::FORBIDDEN => UpstreamError::Forbidden,
            reqwest::StatusCode::BAD_REQUEST => UpstreamError::Malformed,
            _ => UpstreamError::Unavailable,
        });
    }
    let body = read_body_limited(response).await?;
    serde_json::from_slice(&body).map_err(|_| UpstreamError::Malformed)
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
mod tests {
    use super::*;

    const ADDRESS: &str = "0x1111111111111111111111111111111111111111";
    const PLAN_ID: &str = "00000000-0000-0000-0000-000000000001";

    #[test]
    fn dynamic_paths_require_canonical_single_segments() {
        assert_eq!(
            wallet_detail_path("0x1111111111111111111111111111111111111111").as_deref(),
            Some("/api/v1/admin/wallets/0x1111111111111111111111111111111111111111")
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
    fn projection_states_are_explicit_and_redaction_drops_identity_fields() {
        let projection = decode_admin_payment_link_list_projection(serde_json::json!({
            "items": [{
                "slug": "epsx-public",
                "max_uses": 1,
                "current_uses": 0,
                "expires_at": null,
                "status": "active"
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
            slug: projection.items[0].slug.clone(),
            max_uses: projection.items[0].max_uses,
            current_uses: projection.items[0].current_uses,
            expires_at: projection.items[0].expires_at.clone(),
            status: projection.items[0].status.clone(),
        };
    }
}
