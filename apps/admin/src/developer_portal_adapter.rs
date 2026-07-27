//! Strict route-owned adapter for the admin developer-portal read projection.
//!
//! Reads are projected into a bounded redacted inventory. Mutations use the
//! backend-owned validation/audit boundary; the only secret-bearing value this
//! adapter can return is the one-time creation result.

use chrono::DateTime;
use epsx_dioxus_ui::pages::admin_pages::developer_portal::{
    decode_admin_developer_key_summary, decode_admin_developer_projection,
    AdminDeveloperApiKeySummary, AdminDeveloperModuleUsage, AdminDeveloperPortalProjection,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

const API_KEYS_PATH: &str = "/api/admin/developer-portal/api-keys?limit=100&offset=0";
const API_KEYS_MUTATION_PATH: &str = "/api/admin/developer-portal/api-keys";
const STATS_PATH: &str = "/api/admin/developer-portal/stats";
const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
const MAX_IDEMPOTENCY_KEY_CHARS: usize = 128;
const MAX_REASON_CHARS: usize = 500;
const MAX_SECRET_CHARS: usize = 512;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AdminDeveloperPortalLoad {
    Ready(AdminDeveloperPortalProjection),
    Empty(AdminDeveloperPortalProjection),
    Forbidden,
    Unavailable,
    Malformed,
}

pub(crate) type AdminDeveloperLoad = AdminDeveloperPortalLoad;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AdminDeveloperMutationError {
    Forbidden,
    Invalid,
    Conflict,
    Unavailable,
    Malformed,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct AdminDeveloperModuleGrant {
    pub(crate) module_id: String,
    pub(crate) access_level: String,
    pub(crate) custom_quotas: Option<Value>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct AdminDeveloperCreateInput {
    pub(crate) client_name: String,
    pub(crate) client_description: Option<String>,
    pub(crate) client_contact_email: Option<String>,
    pub(crate) wallet_address: String,
    pub(crate) allowed_modules: Vec<AdminDeveloperModuleGrant>,
    pub(crate) ip_restrictions: Option<Vec<String>>,
    pub(crate) rate_limit_per_minute: Option<i32>,
    pub(crate) rate_limit_per_day: Option<i32>,
    pub(crate) expires_at: Option<String>,
    pub(crate) plan_ids: Option<Vec<String>>,
    pub(crate) permissions: Option<Vec<String>>,
}

/// Secret-bearing creation result. It intentionally has no Debug or Serialize
/// implementation so a secret cannot enter logs, list/read projections, or
/// audit payloads by an accidental formatting/serialization call.
pub(crate) struct AdminDeveloperCreatedKey {
    pub(crate) key: AdminDeveloperApiKeySummary,
    pub(crate) secret: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResponseMeta {
    timestamp: String,
    request_id: Option<String>,
    version: Option<String>,
    message: Option<String>,
    pagination: Option<Value>,
    permissions: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope<T> {
    success: bool,
    data: Option<T>,
    error: Option<Value>,
    meta: Option<ResponseMeta>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ApiKeyList {
    api_keys: Vec<RawApiKey>,
    total: i64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRateLimits {
    per_minute: i32,
    per_day: i32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawApiKey {
    id: String,
    key_prefix: String,
    // The list/read contract must omit this field entirely. The custom
    // deserializer rejects both a plaintext value and an explicit null.
    #[serde(default, deserialize_with = "reject_secret_field")]
    full_key: Option<String>,
    client_name: String,
    client_description: Option<String>,
    client_contact_email: Option<String>,
    wallet_address: String,
    status: String,
    total_requests: i64,
    ip_restrictions: Vec<String>,
    rate_limits: RawRateLimits,
    allowed_modules: Vec<Value>,
    permission_plans: Vec<Value>,
    selected_permissions: Vec<String>,
    expires_at: Option<String>,
    last_used_at: Option<String>,
    revoked_at: Option<String>,
    revoked_by: Option<String>,
    revocation_reason: Option<String>,
    created_at: String,
    created_by: String,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeveloperStats {
    total_api_keys: i64,
    active_api_keys: i64,
    revoked_api_keys: i64,
    expired_api_keys: i64,
    total_modules: i64,
    active_modules: i64,
    total_requests_today: i64,
    total_requests_this_month: i64,
    top_modules_by_usage: Vec<RawModuleUsage>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawModuleUsage {
    module_id: String,
    module_name: String,
    request_count: i64,
    unique_api_keys: i64,
}

fn reject_secret_field<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let _: Option<String> = Option::deserialize(deserializer)?;
    Err(serde::de::Error::custom(
        "full_key is not allowed in a list/read response",
    ))
}

enum FetchResult {
    Json(Value),
    Forbidden,
    Unavailable,
    Malformed,
}

pub(crate) async fn load_admin_developer_portal(
    client: &epsx_client::ServiceClient,
    ctx: &epsx_client::RequestContext,
) -> AdminDeveloperPortalLoad {
    let Some(token) = ctx
        .auth_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
    else {
        return AdminDeveloperPortalLoad::Unavailable;
    };

    let http_client = match reqwest::Client::builder()
        .timeout(client.config().timeout)
        .redirect(reqwest::redirect::Policy::none())
        .build()
    {
        Ok(client) => client,
        Err(_) => return AdminDeveloperPortalLoad::Unavailable,
    };

    let keys = fetch_json(&http_client, client, API_KEYS_PATH, token, ctx).await;
    let stats = fetch_json(&http_client, client, STATS_PATH, token, ctx).await;
    match (keys, stats) {
        (FetchResult::Forbidden, _) | (_, FetchResult::Forbidden) => {
            AdminDeveloperPortalLoad::Forbidden
        }
        (FetchResult::Malformed, _) | (_, FetchResult::Malformed) => {
            AdminDeveloperPortalLoad::Malformed
        }
        (FetchResult::Json(keys), FetchResult::Json(stats)) => {
            classify_developer_payload(keys, stats)
        }
        _ => AdminDeveloperPortalLoad::Unavailable,
    }
}

pub(crate) async fn create_admin_api_key(
    client: &epsx_client::ServiceClient,
    ctx: &epsx_client::RequestContext,
    input: AdminDeveloperCreateInput,
    idempotency_key: &str,
) -> Result<AdminDeveloperCreatedKey, AdminDeveloperMutationError> {
    validate_idempotency_key(idempotency_key)?;
    validate_create_input(&input)?;
    let value = send_mutation(
        client,
        ctx,
        reqwest::Method::POST,
        API_KEYS_MUTATION_PATH,
        serde_json::to_value(input).map_err(|_| AdminDeveloperMutationError::Malformed)?,
        idempotency_key,
    )
    .await?;
    let payload: CreatedPayload =
        decode_envelope(value).ok_or(AdminDeveloperMutationError::Malformed)?;
    if payload.secret.is_empty()
        || payload.secret.chars().count() > MAX_SECRET_CHARS
        || payload.secret.chars().any(char::is_control)
    {
        return Err(AdminDeveloperMutationError::Malformed);
    }
    let key = project_key(payload.api_key).ok_or(AdminDeveloperMutationError::Malformed)?;
    Ok(AdminDeveloperCreatedKey {
        key,
        secret: payload.secret,
    })
}

pub(crate) async fn revoke_admin_api_key(
    client: &epsx_client::ServiceClient,
    ctx: &epsx_client::RequestContext,
    id: &str,
    reason: &str,
    idempotency_key: &str,
) -> Result<AdminDeveloperApiKeySummary, AdminDeveloperMutationError> {
    let id = canonical_key_id(id)?;
    validate_reason(reason)?;
    validate_idempotency_key(idempotency_key)?;
    decode_mutation_summary(
        send_mutation(
            client,
            ctx,
            reqwest::Method::POST,
            &format!("{API_KEYS_MUTATION_PATH}/{id}/revoke"),
            json!({"reason": reason}),
            idempotency_key,
        )
        .await?,
    )
}

pub(crate) async fn update_admin_api_key_expiration(
    client: &epsx_client::ServiceClient,
    ctx: &epsx_client::RequestContext,
    id: &str,
    expires_at: Option<&str>,
    idempotency_key: &str,
) -> Result<AdminDeveloperApiKeySummary, AdminDeveloperMutationError> {
    let id = canonical_key_id(id)?;
    validate_expiration(expires_at)?;
    validate_idempotency_key(idempotency_key)?;
    decode_mutation_summary(
        send_mutation(
            client,
            ctx,
            reqwest::Method::PATCH,
            &format!("{API_KEYS_MUTATION_PATH}/{id}/expiration"),
            json!({"expires_at": expires_at}),
            idempotency_key,
        )
        .await?,
    )
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreatedPayload {
    api_key: RawApiKey,
    secret: String,
}

fn decode_mutation_summary(
    value: Value,
) -> Result<AdminDeveloperApiKeySummary, AdminDeveloperMutationError> {
    let raw: RawApiKey = decode_envelope(value).ok_or(AdminDeveloperMutationError::Malformed)?;
    let projected = project_key(raw).ok_or(AdminDeveloperMutationError::Malformed)?;
    decode_admin_developer_key_summary(
        serde_json::to_value(projected).map_err(|_| AdminDeveloperMutationError::Malformed)?,
    )
    .ok_or(AdminDeveloperMutationError::Malformed)
}

async fn send_mutation(
    client: &epsx_client::ServiceClient,
    ctx: &epsx_client::RequestContext,
    method: reqwest::Method,
    path: &str,
    body: Value,
    idempotency_key: &str,
) -> Result<Value, AdminDeveloperMutationError> {
    let Some(token) = ctx
        .auth_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
    else {
        return Err(AdminDeveloperMutationError::Unavailable);
    };
    let http_client = reqwest::Client::builder()
        .timeout(client.config().timeout)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| AdminDeveloperMutationError::Unavailable)?;
    let response = http_client
        .request(
            method,
            format!("{}{}", client.base_url().trim_end_matches('/'), path),
        )
        .header("x-request-id", ctx.request_id.to_string())
        .header("idempotency-key", idempotency_key)
        .bearer_auth(token)
        .json(&body)
        .send()
        .await
        .map_err(|_| AdminDeveloperMutationError::Unavailable)?;
    let status = response.status();
    let body = read_response_body_limited(response, MAX_RESPONSE_BYTES)
        .await
        .map_err(|_| AdminDeveloperMutationError::Unavailable)?;
    match status {
        reqwest::StatusCode::OK | reqwest::StatusCode::CREATED => {
            serde_json::from_slice(&body).map_err(|_| AdminDeveloperMutationError::Malformed)
        }
        reqwest::StatusCode::BAD_REQUEST | reqwest::StatusCode::UNPROCESSABLE_ENTITY => {
            Err(AdminDeveloperMutationError::Invalid)
        }
        reqwest::StatusCode::FORBIDDEN => Err(AdminDeveloperMutationError::Forbidden),
        reqwest::StatusCode::CONFLICT => Err(AdminDeveloperMutationError::Conflict),
        _ => Err(AdminDeveloperMutationError::Unavailable),
    }
}

fn validate_create_input(
    input: &AdminDeveloperCreateInput,
) -> Result<(), AdminDeveloperMutationError> {
    if !bounded_text(&input.client_name, 255, false)
        || input
            .client_description
            .as_deref()
            .is_some_and(|value| !bounded_text(value, 2_000, true))
        || input
            .client_contact_email
            .as_deref()
            .is_some_and(|value| !bounded_text(value, 320, false) || !value.contains('@'))
        || !valid_wallet(&input.wallet_address)
        || input.allowed_modules.len() > 100
        || input.ip_restrictions.as_ref().is_some_and(|values| {
            values.len() > 20 || values.iter().any(|value| !bounded_text(value, 64, false))
        })
        || input
            .rate_limit_per_minute
            .is_some_and(|value| !(1..=1_000_000).contains(&value))
        || input
            .rate_limit_per_day
            .is_some_and(|value| !(1..=100_000_000).contains(&value))
        || input
            .expires_at
            .as_deref()
            .is_some_and(|value| validate_expiration(Some(value)).is_err())
    {
        return Err(AdminDeveloperMutationError::Invalid);
    }
    if input.allowed_modules.iter().any(|module| {
        Uuid::parse_str(&module.module_id).is_err()
            || !matches!(
                module.access_level.as_str(),
                "bronze" | "silver" | "gold" | "platinum" | "enterprise"
            )
            || module.custom_quotas.as_ref().is_some_and(|value| {
                serde_json::to_vec(value).map_or(true, |bytes| bytes.len() > 16_384)
            })
    }) || input
        .plan_ids
        .as_ref()
        .is_some_and(|ids| ids.len() > 32 || ids.iter().any(|id| Uuid::parse_str(id).is_err()))
        || input.permissions.as_ref().is_some_and(|permissions| {
            permissions.len() > 100
                || permissions
                    .iter()
                    .any(|permission| !bounded_text(permission, 128, false))
        })
    {
        return Err(AdminDeveloperMutationError::Invalid);
    }
    Ok(())
}

fn canonical_key_id(value: &str) -> Result<String, AdminDeveloperMutationError> {
    Uuid::parse_str(value)
        .map(|id| id.to_string())
        .map_err(|_| AdminDeveloperMutationError::Invalid)
}

fn validate_idempotency_key(value: &str) -> Result<(), AdminDeveloperMutationError> {
    if bounded_text(value, MAX_IDEMPOTENCY_KEY_CHARS, false) {
        Ok(())
    } else {
        Err(AdminDeveloperMutationError::Invalid)
    }
}

fn validate_reason(value: &str) -> Result<(), AdminDeveloperMutationError> {
    if bounded_text(value, MAX_REASON_CHARS, false) {
        Ok(())
    } else {
        Err(AdminDeveloperMutationError::Invalid)
    }
}

fn validate_expiration(value: Option<&str>) -> Result<(), AdminDeveloperMutationError> {
    let Some(value) = value else {
        return Ok(());
    };
    if !bounded_text(value, 64, false) || DateTime::parse_from_rfc3339(value).is_err() {
        return Err(AdminDeveloperMutationError::Invalid);
    }
    Ok(())
}

fn bounded_text(value: &str, max_chars: usize, allow_empty: bool) -> bool {
    (allow_empty || !value.trim().is_empty())
        && value.chars().count() <= max_chars
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

fn valid_wallet(value: &str) -> bool {
    value.len() == 42
        && value.starts_with("0x")
        && value[2..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

async fn fetch_json(
    http_client: &reqwest::Client,
    client: &epsx_client::ServiceClient,
    path: &str,
    token: &str,
    ctx: &epsx_client::RequestContext,
) -> FetchResult {
    let response = match http_client
        .get(format!(
            "{}{}",
            client.base_url().trim_end_matches('/'),
            path
        ))
        .header("x-request-id", ctx.request_id.to_string())
        .bearer_auth(token)
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => return FetchResult::Unavailable,
    };

    match response.status() {
        reqwest::StatusCode::OK => {}
        reqwest::StatusCode::FORBIDDEN => return FetchResult::Forbidden,
        reqwest::StatusCode::BAD_REQUEST | reqwest::StatusCode::UNPROCESSABLE_ENTITY => {
            return FetchResult::Malformed
        }
        _ => return FetchResult::Unavailable,
    }

    let body = match read_response_body_limited(response, MAX_RESPONSE_BYTES).await {
        Ok(body) => body,
        Err(()) => return FetchResult::Unavailable,
    };
    match serde_json::from_slice(&body) {
        Ok(value) => FetchResult::Json(value),
        Err(_) => FetchResult::Malformed,
    }
}

async fn read_response_body_limited(
    mut response: reqwest::Response,
    limit: usize,
) -> Result<Vec<u8>, ()> {
    if response
        .content_length()
        .is_some_and(|length| length > limit as u64)
    {
        return Err(());
    }

    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| ())? {
        let next = body.len().checked_add(chunk.len()).ok_or(())?;
        if next > limit {
            return Err(());
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

fn decode_envelope<T: DeserializeOwned>(value: Value) -> Option<T> {
    let envelope: Envelope<T> = serde_json::from_value(value).ok()?;
    if !envelope.success
        || envelope.error.is_some()
        || !envelope.meta.as_ref().is_some_and(valid_response_meta)
    {
        return None;
    }
    envelope.data
}

fn valid_response_meta(meta: &ResponseMeta) -> bool {
    meta.timestamp.len() <= 64
        && chrono::DateTime::parse_from_rfc3339(&meta.timestamp).is_ok()
        && meta
            .request_id
            .as_deref()
            .is_none_or(|id| uuid::Uuid::parse_str(id).is_ok())
        && meta.version.as_deref() == Some("v1")
        && meta.message.is_none()
        && meta.pagination.is_none()
        && meta.permissions.is_none()
}

fn classify_developer_payload(keys: Value, stats: Value) -> AdminDeveloperPortalLoad {
    let key_data: ApiKeyList = match decode_envelope(keys) {
        Some(data) => data,
        None => return AdminDeveloperPortalLoad::Malformed,
    };
    let stats_data: DeveloperStats = match decode_envelope(stats) {
        Some(data) => data,
        None => return AdminDeveloperPortalLoad::Malformed,
    };
    let api_keys = key_data
        .api_keys
        .into_iter()
        .map(project_key)
        .collect::<Option<Vec<_>>>();
    let top_modules = stats_data
        .top_modules_by_usage
        .into_iter()
        .map(project_module)
        .collect::<Option<Vec<_>>>();
    let (Some(api_keys), Some(top_modules)) = (api_keys, top_modules) else {
        return AdminDeveloperPortalLoad::Malformed;
    };
    if stats_data.total_api_keys < 0 || stats_data.total_api_keys != key_data.total {
        return AdminDeveloperPortalLoad::Malformed;
    }
    let _ = (
        stats_data.active_api_keys,
        stats_data.revoked_api_keys,
        stats_data.expired_api_keys,
        stats_data.total_modules,
        stats_data.active_modules,
    );
    let projection = AdminDeveloperPortalProjection {
        api_keys,
        total_api_keys: stats_data.total_api_keys,
        total_requests_today: stats_data.total_requests_today,
        total_requests_this_month: stats_data.total_requests_this_month,
        top_modules_by_usage: top_modules,
    };
    let encoded = match serde_json::to_value(&projection) {
        Ok(value) => value,
        Err(_) => return AdminDeveloperPortalLoad::Malformed,
    };
    let Some(projection) = decode_admin_developer_projection(encoded) else {
        return AdminDeveloperPortalLoad::Malformed;
    };
    if projection.api_keys.is_empty() && projection.total_api_keys == 0 {
        AdminDeveloperPortalLoad::Empty(projection)
    } else {
        AdminDeveloperPortalLoad::Ready(projection)
    }
}

fn project_key(raw: RawApiKey) -> Option<AdminDeveloperApiKeySummary> {
    if raw.full_key.is_some() {
        return None;
    }
    let _ = (
        raw.client_description,
        raw.client_contact_email,
        raw.wallet_address,
        raw.ip_restrictions,
        raw.rate_limits,
        raw.allowed_modules,
        raw.permission_plans,
        raw.selected_permissions,
        raw.revoked_at,
        raw.revoked_by,
        raw.revocation_reason,
        raw.created_by,
        raw.updated_at,
    );
    let summary = AdminDeveloperApiKeySummary {
        id: raw.id,
        key_prefix: raw.key_prefix,
        client_name: raw.client_name,
        status: raw.status,
        total_requests: raw.total_requests,
        expires_at: raw.expires_at,
        last_used_at: raw.last_used_at,
        created_at: raw.created_at,
    };
    decode_admin_developer_key_summary(serde_json::to_value(summary).ok()?)
}

fn project_module(raw: RawModuleUsage) -> Option<AdminDeveloperModuleUsage> {
    Some(AdminDeveloperModuleUsage {
        module_id: raw.module_id,
        module_name: raw.module_name,
        request_count: raw.request_count,
        unique_api_keys: raw.unique_api_keys,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    fn valid_key() -> Value {
        serde_json::json!({
            "id": "01234567-89ab-4cde-8fab-0123456789ab",
            "key_prefix": "epsx_abc123",
            "client_name": "Production",
            "client_description": null,
            "client_contact_email": null,
            "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
            "status": "active",
            "total_requests": 12,
            "ip_restrictions": [],
            "rate_limits": {"per_minute": 60, "per_day": 10000},
            "allowed_modules": [],
            "permission_plans": [],
            "selected_permissions": [],
            "expires_at": null,
            "last_used_at": null,
            "revoked_at": null,
            "revoked_by": null,
            "revocation_reason": null,
            "created_at": "2026-07-27T00:00:00Z",
            "created_by": "admin",
            "updated_at": "2026-07-27T00:00:00Z"
        })
    }

    fn envelope(data: Value) -> Value {
        serde_json::json!({
            "success": true,
            "data": data,
            "error": null,
            "meta": {"timestamp": "2026-07-27T00:00:00Z", "version": "v1"}
        })
    }

    fn stats() -> Value {
        serde_json::json!({
            "total_api_keys": 1,
            "active_api_keys": 1,
            "revoked_api_keys": 0,
            "expired_api_keys": 0,
            "total_modules": 1,
            "active_modules": 1,
            "total_requests_today": 2,
            "total_requests_this_month": 12,
            "top_modules_by_usage": [{
                "module_id": "11234567-89ab-4cde-8fab-0123456789ab",
                "module_name": "Market data",
                "request_count": 12,
                "unique_api_keys": 1
            }]
        })
    }

    #[test]
    fn read_and_mutation_decoders_require_the_backend_envelope() {
        let wrapped = envelope(serde_json::json!({
            "api_keys": [valid_key()],
            "total": 1
        }));
        assert!(decode_envelope::<ApiKeyList>(wrapped).is_some());
        assert!(decode_envelope::<ApiKeyList>(serde_json::json!({
            "api_keys": [valid_key()],
            "total": 1
        }))
        .is_none());

        let mut invalid_meta = envelope(stats());
        invalid_meta["meta"]["version"] = serde_json::json!("v2");
        assert!(decode_envelope::<DeveloperStats>(invalid_meta).is_none());
    }

    fn context() -> epsx_client::RequestContext {
        epsx_client::RequestContext {
            request_id: uuid::Uuid::parse_str("d9dbcc48-7f46-46cb-9b87-7cda68cb3af2").unwrap(),
            auth_token: Some("verified-admin-token".to_string()),
            user_id: None,
            address: None,
        }
    }

    #[test]
    fn projection_is_redacted_and_secret_once_is_rejected_on_reads() {
        let keys = envelope(serde_json::json!({
            "api_keys": [valid_key()],
            "total": 1
        }));
        let projection = classify_developer_payload(keys, envelope(stats())).clone();
        let AdminDeveloperPortalLoad::Ready(projection) = projection else {
            panic!("expected ready projection");
        };
        let encoded = serde_json::to_value(projection).unwrap();
        assert!(encoded["api_keys"][0].get("full_key").is_none());

        let mut secret = valid_key();
        secret["full_key"] = serde_json::json!("epsx_live_secret");
        let result = classify_developer_payload(
            envelope(serde_json::json!({"api_keys": [secret], "total": 1})),
            envelope(stats()),
        );
        assert!(matches!(result, AdminDeveloperPortalLoad::Malformed));
    }

    #[test]
    fn secret_once_is_only_accepted_from_creation_and_mutation_inputs_are_bounded() {
        let created = envelope(serde_json::json!({
            "api_key": valid_key(),
            "secret": "epsx_live_secret"
        }));
        let payload: CreatedPayload = decode_envelope(created).unwrap();
        assert_eq!(payload.secret, "epsx_live_secret");
        assert!(project_key(payload.api_key).is_some());

        let mut key_with_read_secret = valid_key();
        key_with_read_secret["full_key"] = serde_json::json!(null);
        assert!(serde_json::from_value::<RawApiKey>(key_with_read_secret).is_err());

        let mut input = AdminDeveloperCreateInput {
            client_name: "Production".to_string(),
            client_description: None,
            client_contact_email: None,
            wallet_address: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            allowed_modules: vec![AdminDeveloperModuleGrant {
                module_id: "11234567-89ab-4cde-8fab-0123456789ab".to_string(),
                access_level: "gold".to_string(),
                custom_quotas: None,
            }],
            ip_restrictions: None,
            rate_limit_per_minute: Some(60),
            rate_limit_per_day: Some(10_000),
            expires_at: None,
            plan_ids: None,
            permissions: Some(vec!["admin:developer:read".to_string()]),
        };
        assert!(validate_create_input(&input).is_ok());
        input.allowed_modules[0].module_id = "not-a-uuid".to_string();
        assert_eq!(
            validate_create_input(&input),
            Err(AdminDeveloperMutationError::Invalid)
        );
        assert_eq!(
            canonical_key_id("not-a-uuid"),
            Err(AdminDeveloperMutationError::Invalid)
        );
        assert_eq!(
            validate_idempotency_key(" bad"),
            Err(AdminDeveloperMutationError::Invalid)
        );
    }

    #[test]
    fn malformed_stats_and_empty_inventory_are_classified_truthfully() {
        let empty = classify_developer_payload(
            envelope(serde_json::json!({"api_keys": [], "total": 0})),
            envelope(serde_json::json!({
                "total_api_keys": 0,
                "active_api_keys": 0,
                "revoked_api_keys": 0,
                "expired_api_keys": 0,
                "total_modules": 0,
                "active_modules": 0,
                "total_requests_today": 0,
                "total_requests_this_month": 0,
                "top_modules_by_usage": []
            })),
        );
        assert!(matches!(empty, AdminDeveloperPortalLoad::Empty(_)));

        let mut malformed = stats();
        malformed["top_modules_by_usage"][0]["unique_api_keys"] = serde_json::json!(99);
        let result = classify_developer_payload(
            envelope(serde_json::json!({"api_keys": [valid_key()], "total": 1})),
            envelope(malformed),
        );
        assert!(matches!(result, AdminDeveloperPortalLoad::Malformed));
    }

    #[tokio::test]
    async fn loader_forwards_only_bearer_and_request_id_without_redirects() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            for index in 0..2 {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut request = Vec::new();
                let mut chunk = [0_u8; 1024];
                while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                    let read = stream.read(&mut chunk).await.unwrap();
                    assert!(read > 0);
                    request.extend_from_slice(&chunk[..read]);
                }
                let request = String::from_utf8(request).unwrap();
                assert!(request
                    .to_ascii_lowercase()
                    .contains("authorization: bearer verified-admin-token"));
                assert!(request
                    .to_ascii_lowercase()
                    .contains("x-request-id: d9dbcc48-7f46-46cb-9b87-7cda68cb3af2"));
                assert!(!request.to_ascii_lowercase().contains("x-user-address:"));
                let body = if index == 0 {
                    envelope(serde_json::json!({"api_keys": [valid_key()], "total": 1}))
                } else {
                    envelope(stats())
                }
                .to_string();
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream.write_all(response.as_bytes()).await.unwrap();
            }
        });
        let client = epsx_client::ServiceClient::new(epsx_client::ClientConfig {
            base_url: format!("http://{address}"),
            timeout: Duration::from_secs(2),
        });
        let load = load_admin_developer_portal(&client, &context()).await;
        server.await.unwrap();
        assert!(matches!(load, AdminDeveloperPortalLoad::Ready(_)));
    }
}
