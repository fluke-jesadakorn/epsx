//! Owner-scoped Developer Portal HTTP contract.

use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Extension, Json,
};
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use uuid::Uuid;

use crate::domain::developer_portal::{
    DeveloperEntitlement, DeveloperEntitlementService, UsageReport, UsageService,
};
use crate::infrastructure::adapters::repositories::developer_portal::{
    ApiKeyRepository, IdempotentMutation, OwnerApiKeyCreateRequest,
};
use crate::prelude::{AppError, AppResult};
use crate::web::auth::AppState;
use crate::web::middleware::OpenIDUserContext;
use crate::web::responses::UnifiedApiResponse;

const MAX_NAME_CHARS: usize = 255;
const MAX_DESCRIPTION_CHARS: usize = 2_000;
const MAX_SCOPES: usize = 100;
const MAX_REASON_CHARS: usize = 500;

#[derive(Debug, Deserialize)]
pub struct ListMyKeysQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateMyApiKeyBody {
    pub name: String,
    pub description: Option<String>,
    pub scopes: Vec<String>,
    pub expires_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevokeMyApiKeyBody {
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UsageDaysQuery {
    pub days: Option<i32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ApiKeySummary {
    pub id: Uuid,
    pub key_prefix: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub scopes: Vec<String>,
    pub total_requests: i64,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct MyApiKeyListResponse {
    pub api_keys: Vec<ApiKeySummary>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Serialize)]
pub struct CreateMyApiKeyResponse {
    pub api_key: ApiKeySummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    pub replayed: bool,
}

#[derive(Debug, Serialize)]
pub struct RevokeMyApiKeyResponse {
    pub id: Uuid,
    pub status: &'static str,
    pub replayed: bool,
}

#[derive(Debug, Serialize)]
pub struct DeveloperOverviewResponse {
    pub entitlement: DeveloperEntitlement,
    pub api_keys: Vec<ApiKeySummary>,
    pub total_api_keys: i64,
    pub usage: UsageReport,
}

#[derive(Debug, Serialize)]
pub struct AvailablePlansResponse {
    pub plans: Vec<AvailablePlan>,
}

#[derive(Debug, Serialize)]
pub struct AvailablePlan {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub permissions: Vec<String>,
}

fn private_success<T: Serialize>(status: StatusCode, data: T) -> Response {
    let mut response = (status, Json(UnifiedApiResponse::success(data))).into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    response
}

fn private_error(error: AppError) -> Response {
    let mut response = error.into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    response
}

fn valid_text(value: &str, max_chars: usize, allow_empty: bool) -> bool {
    let trimmed = value.trim();
    (allow_empty || !trimmed.is_empty())
        && trimmed.chars().count() <= max_chars
        && !trimmed.chars().any(char::is_control)
}

fn parse_days(value: Option<i32>) -> AppResult<i32> {
    let days = value.unwrap_or(7);
    if matches!(days, 7 | 30 | 90) {
        Ok(days)
    } else {
        Err(AppError::bad_request("days must be one of 7, 30, or 90"))
    }
}

fn require_idempotency_key(headers: &HeaderMap) -> AppResult<&str> {
    let key = headers
        .get("idempotency-key")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::bad_request("Idempotency-Key header is required"))?;
    if !(8..=128).contains(&key.len())
        || !key
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':'))
    {
        return Err(AppError::bad_request("Idempotency-Key is malformed"));
    }
    Ok(key)
}

fn payload_hash(value: &impl Serialize) -> AppResult<String> {
    let encoded = serde_json::to_vec(value)
        .map_err(|error| AppError::bad_request(format!("invalid mutation payload: {error}")))?;
    Ok(hex::encode(Sha256::digest(encoded)))
}

fn api_key_summary(
    key: crate::domain::developer_portal::ApiKey,
    assignable_scopes: &[String],
) -> ApiKeySummary {
    ApiKeySummary {
        id: key.id,
        key_prefix: format!("{}…", key.key_prefix),
        name: key.client_name,
        description: key.client_description,
        status: key.status.to_string(),
        scopes: key
            .selected_permissions
            .into_iter()
            .filter(|scope| assignable_scopes.contains(scope) && !scope.starts_with("admin:"))
            .collect(),
        total_requests: key.total_requests,
        expires_at: key.expires_at,
        last_used_at: key.last_used_at,
        created_at: key.created_at,
    }
}

async fn live_entitlement(state: &AppState, wallet: &str) -> AppResult<DeveloperEntitlement> {
    DeveloperEntitlementService::new(*state.db_pool)
        .resolve(wallet)
        .await
}

fn require_read(entitlement: &DeveloperEntitlement) -> AppResult<()> {
    if entitlement.can_read {
        Ok(())
    } else {
        Err(AppError::forbidden("epsx:api:read is required"))
    }
}

fn require_write(entitlement: &DeveloperEntitlement) -> AppResult<()> {
    require_read(entitlement)?;
    if entitlement.can_write {
        Ok(())
    } else {
        Err(AppError::forbidden("epsx:api:write is required"))
    }
}

pub async fn list_my_keys_handler(
    State(state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
    Query(query): Query<ListMyKeysQuery>,
) -> Response {
    let result = async {
        let entitlement = live_entitlement(&state, &context.wallet_address).await?;
        require_read(&entitlement)?;
        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);
        if !(1..=100).contains(&limit) || !(0..=1_000_000).contains(&offset) {
            return Err(AppError::bad_request("invalid pagination"));
        }
        if query
            .status
            .as_deref()
            .is_some_and(|status| !matches!(status, "active" | "revoked" | "expired"))
        {
            return Err(AppError::bad_request("invalid API key status"));
        }
        let (keys, total) = ApiKeyRepository::new(*state.db_pool)
            .list_by_wallet(
                &context.wallet_address,
                Some(limit),
                Some(offset),
                query.status.as_deref(),
            )
            .await?;
        Ok(MyApiKeyListResponse {
            api_keys: keys
                .into_iter()
                .map(|key| api_key_summary(key, &entitlement.assignable_scopes))
                .collect(),
            total,
            limit,
            offset,
        })
    }
    .await;
    match result {
        Ok(data) => private_success(StatusCode::OK, data),
        Err(error) => private_error(error),
    }
}

pub async fn get_my_key_handler(
    State(state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
    Path(id): Path<Uuid>,
) -> Response {
    let result = async {
        let entitlement = live_entitlement(&state, &context.wallet_address).await?;
        require_read(&entitlement)?;
        let key = ApiKeyRepository::new(*state.db_pool)
            .get_by_id(id)
            .await?
            .filter(|key| {
                key.wallet_address
                    .eq_ignore_ascii_case(&context.wallet_address)
            })
            .ok_or_else(|| AppError::not_found("API key not found"))?;
        Ok(api_key_summary(key, &entitlement.assignable_scopes))
    }
    .await;
    match result {
        Ok(data) => private_success(StatusCode::OK, data),
        Err(error) => private_error(error),
    }
}

pub async fn create_my_key_handler(
    State(state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
    headers: HeaderMap,
    Json(body): Json<CreateMyApiKeyBody>,
) -> Response {
    let result = async {
        let entitlement = live_entitlement(&state, &context.wallet_address).await?;
        require_write(&entitlement)?;
        if !entitlement.has_active_api_entitlement {
            return Err(AppError::forbidden("no active API entitlement"));
        }
        if !valid_text(&body.name, MAX_NAME_CHARS, false)
            || body
                .description
                .as_deref()
                .is_some_and(|value| !valid_text(value, MAX_DESCRIPTION_CHARS, true))
        {
            return Err(AppError::bad_request("invalid API key name or description"));
        }
        let requested = body
            .scopes
            .iter()
            .map(|scope| scope.trim().to_string())
            .collect::<BTreeSet<_>>();
        if body.scopes.is_empty()
            || body.scopes.len() > MAX_SCOPES
            || requested.len() != body.scopes.len()
            || requested.is_empty()
            || requested.len() > MAX_SCOPES
            || requested.iter().any(|scope| {
                scope.is_empty()
                    || scope.starts_with("admin:")
                    || !entitlement.assignable_scopes.contains(scope)
            })
        {
            return Err(AppError::forbidden(
                "requested scopes are not a subset of the live API entitlement",
            ));
        }
        let expires_at = body
            .expires_at
            .as_deref()
            .map(DateTime::parse_from_rfc3339)
            .transpose()
            .map_err(|_| AppError::bad_request("expires_at must be RFC3339"))?
            .map(|value| value.with_timezone(&Utc));
        if expires_at.is_some_and(|expiry| {
            expiry <= Utc::now() || expiry > Utc::now() + Duration::days(3_653)
        }) {
            return Err(AppError::bad_request(
                "expires_at must be in the future and no more than 10 years away",
            ));
        }
        let idempotency_key = require_idempotency_key(&headers)?;
        #[derive(Serialize)]
        struct CanonicalPayload<'a> {
            name: &'a str,
            description: &'a Option<String>,
            scopes: &'a BTreeSet<String>,
            expires_at: Option<DateTime<Utc>>,
        }
        let hash = payload_hash(&CanonicalPayload {
            name: body.name.trim(),
            description: &body.description,
            scopes: &requested,
            expires_at,
        })?;
        let repo = ApiKeyRepository::new(*state.db_pool);
        let (mutation, secret) = repo
            .create_for_owner(
                OwnerApiKeyCreateRequest {
                    client_name: body.name.trim().to_string(),
                    client_description: body.description,
                    wallet_address: context.wallet_address.to_ascii_lowercase(),
                    scopes: requested.into_iter().collect(),
                    rate_limit_per_minute: entitlement.rate_limits.per_minute.min(i32::MAX as u32)
                        as i32,
                    rate_limit_per_day: entitlement.rate_limits.per_day.min(i32::MAX as u32) as i32,
                    expires_at,
                },
                idempotency_key,
                &hash,
            )
            .await?;
        let id = match mutation {
            IdempotentMutation::Applied(id) | IdempotentMutation::Replayed(id) => id,
        };
        let key = repo
            .get_by_id(id)
            .await?
            .filter(|key| {
                key.wallet_address
                    .eq_ignore_ascii_case(&context.wallet_address)
            })
            .ok_or_else(|| AppError::not_found("API key not found"))?;
        Ok(CreateMyApiKeyResponse {
            api_key: api_key_summary(key, &entitlement.assignable_scopes),
            secret,
            replayed: matches!(mutation, IdempotentMutation::Replayed(_)),
        })
    }
    .await;
    match result {
        Ok(data) => private_success(
            if data.replayed {
                StatusCode::OK
            } else {
                StatusCode::CREATED
            },
            data,
        ),
        Err(error) => private_error(error),
    }
}

pub async fn revoke_my_key_handler(
    State(state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(body): Json<RevokeMyApiKeyBody>,
) -> Response {
    let result = async {
        require_write(&live_entitlement(&state, &context.wallet_address).await?)?;
        let reason = body
            .reason
            .unwrap_or_else(|| "Revoked by owner".to_string());
        if !valid_text(&reason, MAX_REASON_CHARS, false) {
            return Err(AppError::bad_request("invalid revocation reason"));
        }
        let idempotency_key = require_idempotency_key(&headers)?;
        let hash = payload_hash(&serde_json::json!({"id": id, "reason": reason}))?;
        let mutation = ApiKeyRepository::new(*state.db_pool)
            .revoke_for_owner(
                id,
                &context.wallet_address.to_ascii_lowercase(),
                &reason,
                idempotency_key,
                &hash,
            )
            .await?;
        Ok(RevokeMyApiKeyResponse {
            id,
            status: "revoked",
            replayed: matches!(mutation, IdempotentMutation::Replayed(_)),
        })
    }
    .await;
    match result {
        Ok(data) => private_success(StatusCode::OK, data),
        Err(error) => private_error(error),
    }
}

pub async fn list_available_plans_handler(State(state): State<AppState>) -> Response {
    let result: AppResult<AvailablePlansResponse> = async {
        use crate::schemas::primary::{permissions, plan_permissions, plans};
        let mut conn =
            state.db_pool.acquire().await.map_err(|error| {
                AppError::database_error(format!("available plans pool: {error}"))
            })?;
        let plan_rows = plans::table
            .filter(plans::is_active.eq(true))
            .filter(plans::is_public.eq(true))
            .filter(plans::plan_type.ne("admin"))
            .select((plans::id, plans::name, plans::slug, plans::description))
            .order(plans::name.asc())
            .load::<(Uuid, String, String, String)>(&mut *conn)
            .await?;
        let mut result = Vec::with_capacity(plan_rows.len());
        for (id, name, slug, description) in plan_rows {
            let plan_scopes = plan_permissions::table
                .inner_join(
                    permissions::table.on(permissions::id.eq(plan_permissions::permission_id)),
                )
                .filter(plan_permissions::plan_id.eq(id))
                .filter(permissions::is_active.eq(true))
                .filter(permissions::api_assignable.eq(true))
                .filter(permissions::permission_string.not_like("admin:%"))
                .select(permissions::permission_string)
                .order(permissions::permission_string.asc())
                .load::<String>(&mut *conn)
                .await?;
            result.push(AvailablePlan {
                id,
                name,
                slug,
                description,
                permissions: plan_scopes,
            });
        }
        Ok(AvailablePlansResponse { plans: result })
    }
    .await;
    match result {
        Ok(data) => {
            let mut response = Json(UnifiedApiResponse::success(data)).into_response();
            response.headers_mut().insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=60"),
            );
            response
        }
        Err(error) => error.into_response(),
    }
}

pub async fn get_my_plans_handler(
    State(state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
) -> Response {
    match live_entitlement(&state, &context.wallet_address).await {
        Ok(entitlement) if entitlement.can_read => private_success(StatusCode::OK, entitlement),
        Ok(_) => private_error(AppError::forbidden("epsx:api:read is required")),
        Err(error) => private_error(error),
    }
}

async fn usage_report(state: &AppState, wallet: &str, days: Option<i32>) -> AppResult<UsageReport> {
    require_read(&live_entitlement(state, wallet).await?)?;
    let days = parse_days(days)?;
    let analytics_pool = state.analytics_db_pool.as_ref().ok_or_else(|| {
        AppError::new(
            epsx_contracts::errors::ErrorKind::ServiceUnavailable,
            "usage analytics unavailable",
        )
    })?;
    UsageService::new(*state.db_pool, **analytics_pool)
        .get_report(wallet, days)
        .await
        .map_err(|error| {
            AppError::new(
                epsx_contracts::errors::ErrorKind::ServiceUnavailable,
                format!("usage analytics unavailable: {error}"),
            )
        })
}

pub async fn get_usage_stats_handler(
    State(state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
) -> Response {
    match usage_report(&state, &context.wallet_address, Some(7)).await {
        Ok(report) => private_success(StatusCode::OK, report),
        Err(error) => private_error(error),
    }
}

pub async fn get_usage_history_handler(
    State(state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
    Query(query): Query<UsageDaysQuery>,
) -> Response {
    match usage_report(&state, &context.wallet_address, query.days).await {
        Ok(report) => private_success(StatusCode::OK, report.daily),
        Err(error) => private_error(error),
    }
}

pub async fn get_top_endpoints_handler(
    State(state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
    Query(query): Query<UsageDaysQuery>,
) -> Response {
    match usage_report(&state, &context.wallet_address, query.days).await {
        Ok(report) => private_success(StatusCode::OK, report.top_endpoints),
        Err(error) => private_error(error),
    }
}

pub async fn developer_overview_handler(
    State(state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
    Query(query): Query<UsageDaysQuery>,
) -> Response {
    let result = async {
        let days = parse_days(query.days)?;
        let entitlement = live_entitlement(&state, &context.wallet_address).await?;
        require_read(&entitlement)?;
        let repo = ApiKeyRepository::new(*state.db_pool);
        let (keys, total_api_keys) = repo
            .list_by_wallet(&context.wallet_address, Some(100), Some(0), None)
            .await?;
        let usage = usage_report(&state, &context.wallet_address, Some(days)).await?;
        let api_keys = keys
            .into_iter()
            .map(|key| api_key_summary(key, &entitlement.assignable_scopes))
            .collect();
        Ok(DeveloperOverviewResponse {
            entitlement,
            api_keys,
            total_api_keys,
            usage,
        })
    }
    .await;
    match result {
        Ok(data) => private_success(StatusCode::OK, data),
        Err(error) => private_error(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_contract_rejects_client_authority_fields() {
        let body = serde_json::json!({
            "name": "client",
            "description": null,
            "scopes": ["epsx:analytics:view"],
            "expires_at": null,
            "plan_ids": [Uuid::nil()],
            "wallet_address": "0x0000000000000000000000000000000000000000",
            "rate_limit_per_minute": 999999
        });
        assert!(serde_json::from_value::<CreateMyApiKeyBody>(body).is_err());
    }

    #[test]
    fn read_summary_has_no_plaintext_secret_field() {
        let fields = serde_json::to_value(ApiKeySummary {
            id: Uuid::nil(),
            key_prefix: "epsx_deadbeef…".to_string(),
            name: "test".to_string(),
            description: None,
            status: "active".to_string(),
            scopes: vec!["epsx:analytics:view".to_string()],
            total_requests: 0,
            expires_at: None,
            last_used_at: None,
            created_at: Utc::now(),
        })
        .unwrap();
        assert!(fields.get("secret").is_none());
        assert!(fields.get("full_key").is_none());
    }

    #[test]
    fn allowed_usage_windows_are_explicit() {
        for days in [7, 30, 90] {
            assert_eq!(parse_days(Some(days)).unwrap(), days);
        }
        for days in [0, 1, 31, 365] {
            assert!(parse_days(Some(days)).is_err());
        }
    }
}
