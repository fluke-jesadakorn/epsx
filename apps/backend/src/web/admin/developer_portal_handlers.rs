//! Developer Portal API Handlers
//!
//! REST endpoints for API key and module management.

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Json, Response},
    Extension,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use tracing::{error, info};
use uuid::Uuid;

use crate::domain::developer_portal::{
    ApiKey, ApiKeyStatus, CreateApiKeyRequest, CreateModuleRequest, DeveloperPortalStats,
    ModuleAccessRequest, RevokeApiKeyRequest, UpdateModuleRequest, UsageService,
};
use crate::infrastructure::adapters::repositories::developer_portal::{
    ApiKeyRepository, ModuleRepository,
};
use crate::infrastructure::database::get_analytics_pool;
use crate::infrastructure::services::audit_service::{AuditCtx, AuditEntry};
use crate::web::auth::AppState;
use crate::web::responses::UnifiedApiResponse;

// ============================================================================
// Request/Response DTOs
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListApiKeysQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub status: Option<String>,
    /// Filter by wallet address
    pub wallet: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListModulesQuery {
    pub status: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateApiKeyBody {
    pub client_name: String,
    pub client_description: Option<String>,
    pub client_contact_email: Option<String>,
    pub wallet_address: String,
    #[serde(default)]
    pub allowed_modules: Vec<ModuleAccessInput>,
    pub ip_restrictions: Option<Vec<String>>,
    pub rate_limit_per_minute: Option<i32>,
    pub rate_limit_per_day: Option<i32>,
    pub expires_at: Option<String>,
    /// Optional plan IDs to assign to the API key
    pub plan_ids: Option<Vec<String>>,
    /// Optional direct permissions to assign to the API key
    pub permissions: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleAccessInput {
    pub module_id: String,
    pub access_level: String,
    pub custom_quotas: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevokeApiKeyBody {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateExpirationBody {
    /// New expiration date in ISO 8601 format, or null to remove expiration
    pub expires_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateModuleBody {
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub category: String,
    pub base_path: String,
    pub default_rate_limit: Option<i32>,
    pub access_levels: Option<serde_json::Value>,
    pub endpoints: Option<Vec<crate::domain::developer_portal::ModuleEndpoint>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateModuleBody {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub default_rate_limit: Option<i32>,
    pub access_levels: Option<serde_json::Value>,
    pub endpoints: Option<Vec<crate::domain::developer_portal::ModuleEndpoint>>,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyListResponse {
    pub api_keys: Vec<AdminApiKeySummaryView>,
    pub total: i64,
}

/// The bounded inventory endpoint exposes only fields needed by the
/// read-only developer portal. Management/detail endpoints continue to use
/// `AdminApiKeyView`, but list consumers must not receive wallet ownership,
/// contacts, permissions, rate limits, or revocation metadata by default.
#[derive(Debug, Serialize)]
pub struct AdminApiKeySummaryView {
    pub id: Uuid,
    pub key_prefix: String,
    pub client_name: String,
    pub status: ApiKeyStatus,
    pub total_requests: i64,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<ApiKey> for AdminApiKeySummaryView {
    fn from(value: ApiKey) -> Self {
        Self {
            id: value.id,
            key_prefix: value.key_prefix,
            client_name: value.client_name,
            status: value.status,
            total_requests: value.total_requests,
            expires_at: value.expires_at,
            last_used_at: value.last_used_at,
            created_at: value.created_at,
        }
    }
}

/// A read projection that can never contain the plaintext API-key secret.
/// The secret is carried only by `AdminApiKeyCreatedResponse` on creation.
#[derive(Debug, Serialize)]
pub struct AdminApiKeyView {
    pub id: Uuid,
    pub key_prefix: String,
    pub client_name: String,
    pub client_description: Option<String>,
    pub client_contact_email: Option<String>,
    pub wallet_address: String,
    pub status: ApiKeyStatus,
    pub total_requests: i64,
    pub ip_restrictions: Vec<String>,
    pub rate_limits: crate::domain::developer_portal::RateLimits,
    pub allowed_modules: Vec<crate::domain::developer_portal::ModuleAccess>,
    pub permission_plans: Vec<crate::domain::developer_portal::PlanInfo>,
    pub selected_permissions: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_by: Option<String>,
    pub revocation_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub updated_at: DateTime<Utc>,
}

impl From<ApiKey> for AdminApiKeyView {
    fn from(value: ApiKey) -> Self {
        Self {
            id: value.id,
            key_prefix: value.key_prefix,
            client_name: value.client_name,
            client_description: value.client_description,
            client_contact_email: value.client_contact_email,
            wallet_address: value.wallet_address,
            status: value.status,
            total_requests: value.total_requests,
            ip_restrictions: value.ip_restrictions,
            rate_limits: value.rate_limits,
            allowed_modules: value.allowed_modules,
            permission_plans: value.permission_plans,
            selected_permissions: value.selected_permissions,
            expires_at: value.expires_at,
            last_used_at: value.last_used_at,
            revoked_at: value.revoked_at,
            revoked_by: value.revoked_by,
            revocation_reason: value.revocation_reason,
            created_at: value.created_at,
            created_by: value.created_by,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AdminApiKeyCreatedResponse {
    pub api_key: AdminApiKeyView,
    pub secret: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListExpiringKeysQuery {
    /// Number of days to look ahead for expiring keys (default: 7)
    pub days: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ExpiringKeysResponse {
    pub api_keys: Vec<AdminApiKeyView>,
    pub total: i64,
    pub days_ahead: i64,
}

// ============================================================================
// API Key Handlers
// ============================================================================

fn correlation_id(headers: &HeaderMap) -> Result<String, &'static str> {
    match headers.get("x-request-id") {
        None => Ok(Uuid::new_v4().to_string()),
        Some(value) => {
            let value = value.to_str().map_err(|_| "x-request-id must be ASCII")?;
            Uuid::parse_str(value)
                .map(|id| id.to_string())
                .map_err(|_| "x-request-id must be a UUID")
        }
    }
}

fn idempotency_key(headers: &HeaderMap) -> Result<&str, &'static str> {
    let value = headers
        .get("idempotency-key")
        .ok_or("Idempotency-Key is required for developer portal mutations")?
        .to_str()
        .map_err(|_| "Idempotency-Key must be ASCII")?;
    if value.is_empty()
        || value.chars().count() > 128
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err("Idempotency-Key must be bounded and control-free");
    }
    Ok(value)
}

fn response_with_id<T: Serialize>(
    mut body: UnifiedApiResponse<T>,
    request_id: &str,
    status: Option<StatusCode>,
) -> Response {
    if let Some(meta) = body.meta.as_mut() {
        meta.request_id = Some(request_id.to_string());
    }
    let mut response = body.into_response();
    if let Some(status) = status {
        *response.status_mut() = status;
    }
    if let Ok(value) = HeaderValue::from_str(request_id) {
        response.headers_mut().insert("x-request-id", value);
    }
    response
}

fn error_response<T: Serialize>(
    request_id: &str,
    status: StatusCode,
    message: &str,
    reason: &str,
    error_type: &str,
    details: serde_json::Value,
) -> Response {
    response_with_id(
        UnifiedApiResponse::<T>::error_with_details(
            status.as_u16(),
            message,
            reason,
            error_type,
            details,
        ),
        request_id,
        None,
    )
}

fn authorize<T: Serialize>(
    context: &crate::web::middleware::OpenIDUserContext,
    permission: &str,
    request_id: &str,
) -> Result<(), Response> {
    if !matches!(context.token_audiences.as_deref(), Some([audience]) if audience == "epsx-admin") {
        return Err(error_response::<T>(
            request_id,
            StatusCode::UNAUTHORIZED,
            "Authentication required",
            "A single epsx-admin audience is required",
            "invalid_admin_audience",
            json!({}),
        ));
    }
    if !epsx_contracts::permissions::has_permission(&context.permissions, permission) {
        return Err(error_response::<T>(
            request_id,
            StatusCode::FORBIDDEN,
            "Permission denied",
            "The admin token does not grant the required developer permission",
            "missing_permission",
            json!({"required_permission": permission}),
        ));
    }
    Ok(())
}

fn valid_text(value: &str, max: usize, allow_empty: bool) -> bool {
    (allow_empty || !value.trim().is_empty())
        && value.chars().count() <= max
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

fn valid_wallet(value: &str) -> bool {
    value.len() == 42
        && value.starts_with("0x")
        && value[2..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_expiration(value: Option<&str>) -> Result<Option<DateTime<Utc>>, &'static str> {
    let Some(value) = value else {
        return Ok(None);
    };
    let parsed = DateTime::parse_from_rfc3339(value)
        .map_err(|_| "expires_at must be RFC3339")?
        .with_timezone(&Utc);
    let now = Utc::now();
    if parsed <= now || parsed > now + chrono::Duration::days(3650) {
        return Err("expires_at must be in the future and no more than 10 years away");
    }
    Ok(Some(parsed))
}

fn validate_pagination(
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<(i64, i64), &'static str> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);
    if !(1..=100).contains(&limit) || !(0..=10_000_000).contains(&offset) {
        return Err("limit must be 1..100 and offset must be 0..10,000,000");
    }
    Ok((limit, offset))
}

fn validate_create_body(
    body: &CreateApiKeyBody,
) -> Result<
    (Option<DateTime<Utc>>, Vec<ModuleAccessRequest>, Vec<Uuid>),
    (&'static str, serde_json::Value),
> {
    if !valid_text(&body.client_name, 255, false) {
        return Err((
            "client_name is required and bounded",
            json!({"field":"client_name"}),
        ));
    }
    if body
        .client_description
        .as_deref()
        .is_some_and(|value| !valid_text(value, 2_000, true))
    {
        return Err((
            "client_description is invalid",
            json!({"field":"client_description"}),
        ));
    }
    if body
        .client_contact_email
        .as_deref()
        .is_some_and(|value| !valid_text(value, 320, false) || !value.contains('@'))
    {
        return Err((
            "client_contact_email is invalid",
            json!({"field":"client_contact_email"}),
        ));
    }
    if !valid_wallet(&body.wallet_address) {
        return Err((
            "wallet_address must be a canonical EVM address",
            json!({"field":"wallet_address"}),
        ));
    }
    if body.allowed_modules.len() > 100 {
        return Err((
            "allowed_modules is too large",
            json!({"field":"allowed_modules"}),
        ));
    }
    let mut modules = Vec::with_capacity(body.allowed_modules.len());
    for module in &body.allowed_modules {
        let module_id = Uuid::parse_str(&module.module_id).map_err(|_| {
            (
                "module_id must be a UUID",
                json!({"field":"allowed_modules"}),
            )
        })?;
        if !matches!(
            module.access_level.as_str(),
            "bronze" | "silver" | "gold" | "platinum" | "enterprise"
        ) {
            return Err(("access_level is invalid", json!({"field":"access_level"})));
        }
        if module.custom_quotas.as_ref().is_some_and(|value| {
            serde_json::to_vec(value).map_or(true, |bytes| bytes.len() > 16_384)
        }) {
            return Err((
                "custom_quotas is too large",
                json!({"field":"custom_quotas"}),
            ));
        }
        modules.push(ModuleAccessRequest {
            module_id,
            access_level: module.access_level.clone(),
            custom_quotas: module.custom_quotas.clone(),
        });
    }
    let plan_ids = body
        .plan_ids
        .clone()
        .unwrap_or_default()
        .into_iter()
        .map(|value| {
            Uuid::parse_str(&value)
                .map_err(|_| ("plan_ids must contain UUIDs", json!({"field":"plan_ids"})))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if plan_ids.len() > 32 {
        return Err(("plan_ids is too large", json!({"field":"plan_ids"})));
    }
    let permissions = body.permissions.as_deref().unwrap_or_default();
    if permissions.len() > 100
        || permissions
            .iter()
            .any(|value| !valid_text(value, 128, false))
    {
        return Err((
            "permissions are invalid or too large",
            json!({"field":"permissions"}),
        ));
    }
    if body.ip_restrictions.as_ref().is_some_and(|values| {
        values.len() > 20 || values.iter().any(|value| !valid_text(value, 64, false))
    }) {
        return Err((
            "ip_restrictions are invalid or too large",
            json!({"field":"ip_restrictions"}),
        ));
    }
    if body
        .rate_limit_per_minute
        .is_some_and(|value| !(1..=1_000_000).contains(&value))
        || body
            .rate_limit_per_day
            .is_some_and(|value| !(1..=100_000_000).contains(&value))
    {
        return Err((
            "rate limits are outside their bounds",
            json!({"field":"rate_limit"}),
        ));
    }
    let expires_at = valid_expiration(body.expires_at.as_deref())
        .map_err(|reason| (reason, json!({"field":"expires_at"})))?;
    Ok((expires_at, modules, plan_ids))
}

fn validate_reason(reason: &str) -> Result<(), &'static str> {
    if valid_text(reason, 500, false) {
        Ok(())
    } else {
        Err("reason must be non-empty and at most 500 characters")
    }
}

fn validate_days(days: Option<i64>) -> Result<i64, &'static str> {
    let days = days.unwrap_or(7);
    if (1..=365).contains(&days) {
        Ok(days)
    } else {
        Err("days must be between 1 and 365")
    }
}

pub async fn list_api_keys_handler(
    State(state): State<AppState>,
    Extension(context): Extension<crate::web::middleware::OpenIDUserContext>,
    headers: HeaderMap,
    Query(query): Query<ListApiKeysQuery>,
) -> Response {
    let request_id = match correlation_id(&headers) {
        Ok(id) => id,
        Err(reason) => {
            return error_response::<()>(
                "generated",
                StatusCode::BAD_REQUEST,
                "Invalid request ID",
                reason,
                "invalid_request_id",
                json!({}),
            )
        }
    };
    if let Err(response) =
        authorize::<ApiKeyListResponse>(&context, "admin:developer:read", &request_id)
    {
        return response;
    }
    let (limit, offset) = match validate_pagination(query.limit, query.offset) {
        Ok(value) => value,
        Err(reason) => {
            return error_response::<ApiKeyListResponse>(
                &request_id,
                StatusCode::BAD_REQUEST,
                "Invalid pagination",
                reason,
                "invalid_query",
                json!({}),
            )
        }
    };
    if query
        .status
        .as_deref()
        .is_some_and(|value| !matches!(value, "active" | "revoked" | "expired"))
    {
        return error_response::<ApiKeyListResponse>(
            &request_id,
            StatusCode::BAD_REQUEST,
            "Invalid status",
            "status must be active, revoked, or expired",
            "invalid_query",
            json!({}),
        );
    }
    if query
        .wallet
        .as_deref()
        .is_some_and(|value| !valid_wallet(value))
    {
        return error_response::<ApiKeyListResponse>(
            &request_id,
            StatusCode::BAD_REQUEST,
            "Invalid wallet",
            "wallet must be a canonical EVM address",
            "invalid_identifier",
            json!({}),
        );
    }
    let repo = ApiKeyRepository::new(*state.db_pool);
    let result = if let Some(wallet) = &query.wallet {
        repo.list_by_wallet(wallet, Some(limit), Some(offset), query.status.as_deref())
            .await
    } else {
        repo.list_all(Some(limit), Some(offset), query.status.as_deref())
            .await
    };
    match result {
        Ok((api_keys, total)) => response_with_id(
            UnifiedApiResponse::success(ApiKeyListResponse {
                api_keys: api_keys
                    .into_iter()
                    .map(AdminApiKeySummaryView::from)
                    .collect(),
                total,
            }),
            &request_id,
            None,
        ),
        Err(error) => {
            error!(request_id = %request_id, "api key list failed: {error}");
            error_response::<ApiKeyListResponse>(
                &request_id,
                StatusCode::BAD_GATEWAY,
                "Developer portal unavailable",
                "The key repository did not return an authoritative list",
                "repository_read_failed",
                json!({}),
            )
        }
    }
}

pub async fn create_api_key_handler(
    State(state): State<AppState>,
    Extension(context): Extension<crate::web::middleware::OpenIDUserContext>,
    headers: HeaderMap,
    Json(body): Json<CreateApiKeyBody>,
) -> Response {
    let request_id = match correlation_id(&headers) {
        Ok(id) => id,
        Err(reason) => {
            return error_response::<()>(
                "generated",
                StatusCode::BAD_REQUEST,
                "Invalid request ID",
                reason,
                "invalid_request_id",
                json!({}),
            )
        }
    };
    if let Err(response) =
        authorize::<AdminApiKeyCreatedResponse>(&context, "admin:developer:manage", &request_id)
    {
        return response;
    }
    if let Err(reason) = idempotency_key(&headers) {
        return error_response::<AdminApiKeyCreatedResponse>(
            &request_id,
            StatusCode::BAD_REQUEST,
            "Invalid idempotency key",
            reason,
            "invalid_idempotency_key",
            json!({}),
        );
    }
    let (expires_at, allowed_modules, plan_ids) = match validate_create_body(&body) {
        Ok(value) => value,
        Err((reason, details)) => {
            return error_response::<AdminApiKeyCreatedResponse>(
                &request_id,
                StatusCode::UNPROCESSABLE_ENTITY,
                "Validation failed",
                reason,
                "validation_error",
                details,
            )
        }
    };
    let request = CreateApiKeyRequest {
        client_name: body.client_name.clone(),
        client_description: body.client_description.clone(),
        client_contact_email: body.client_contact_email.clone(),
        wallet_address: body.wallet_address.to_lowercase(),
        allowed_modules,
        plan_ids,
        permissions: body.permissions.clone().unwrap_or_default(),
        ip_restrictions: body.ip_restrictions.clone(),
        rate_limit_per_minute: body.rate_limit_per_minute,
        rate_limit_per_day: body.rate_limit_per_day,
        expires_at,
        created_by: context.wallet_address.to_lowercase(),
    };
    let repo = ApiKeyRepository::new(*state.db_pool);
    let created = match repo.create(request).await {
        Ok(value) => value,
        Err(_) => {
            return error_response::<AdminApiKeyCreatedResponse>(
                &request_id,
                StatusCode::CONFLICT,
                "API key was not created",
                "The key request conflicts with an existing record",
                "repository_conflict",
                json!({}),
            )
        }
    };
    let created_view = AdminApiKeyCreatedResponse {
        api_key: AdminApiKeyView::from(created.api_key.clone()),
        secret: created.full_key.clone(),
    };
    let idempotency_digest = headers
        .get("idempotency-key")
        .and_then(|value| value.to_str().ok())
        .map(|value| {
            let mut hasher = Sha256::new();
            hasher.update(value.as_bytes());
            hex::encode(hasher.finalize())
        });
    if state
        .audit
        .log_sync(
            &AuditCtx::from_wallet(&context.wallet_address, &headers),
            &AuditEntry::new("api_key", "create", "developer")
                .id(&created.api_key.id.to_string())
                .meta(
                    json!({"request_id": request_id, "idempotency_key_digest": idempotency_digest}),
                ),
        )
        .await
        .is_err()
    {
        let _ = repo
            .revoke(
                created.api_key.id,
                RevokeApiKeyRequest {
                    reason: "audit write failed".to_string(),
                    revoked_by: context.wallet_address.clone(),
                },
            )
            .await;
        return error_response::<AdminApiKeyCreatedResponse>(
            &request_id,
            StatusCode::SERVICE_UNAVAILABLE,
            "API key creation not committed",
            "The audit record could not be durably written",
            "audit_write_failed",
            json!({}),
        );
    }
    response_with_id(
        UnifiedApiResponse::success(created_view),
        &request_id,
        Some(StatusCode::CREATED),
    )
}

pub async fn get_api_key_handler(
    State(state): State<AppState>,
    Extension(context): Extension<crate::web::middleware::OpenIDUserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let request_id = match correlation_id(&headers) {
        Ok(id) => id,
        Err(reason) => {
            return error_response::<()>(
                "generated",
                StatusCode::BAD_REQUEST,
                "Invalid request ID",
                reason,
                "invalid_request_id",
                json!({}),
            )
        }
    };
    if let Err(response) =
        authorize::<AdminApiKeyView>(&context, "admin:developer:read", &request_id)
    {
        return response;
    }
    let id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return error_response::<AdminApiKeyView>(
                &request_id,
                StatusCode::BAD_REQUEST,
                "Invalid API key ID",
                "id must be a UUID",
                "invalid_identifier",
                json!({}),
            )
        }
    };
    let repo = ApiKeyRepository::new(*state.db_pool);
    match repo.get_by_id(id).await {
        Ok(Some(key)) => response_with_id(
            UnifiedApiResponse::success(AdminApiKeyView::from(key)),
            &request_id,
            None,
        ),
        Ok(None) => error_response::<AdminApiKeyView>(
            &request_id,
            StatusCode::NOT_FOUND,
            "API key not found",
            "The requested API key does not exist",
            "not_found",
            json!({}),
        ),
        Err(_) => error_response::<AdminApiKeyView>(
            &request_id,
            StatusCode::BAD_GATEWAY,
            "Developer portal unavailable",
            "The key repository did not return the requested key",
            "repository_read_failed",
            json!({}),
        ),
    }
}

pub async fn revoke_api_key_handler(
    State(state): State<AppState>,
    Extension(context): Extension<crate::web::middleware::OpenIDUserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<RevokeApiKeyBody>,
) -> Response {
    let request_id = match correlation_id(&headers) {
        Ok(id) => id,
        Err(reason) => {
            return error_response::<()>(
                "generated",
                StatusCode::BAD_REQUEST,
                "Invalid request ID",
                reason,
                "invalid_request_id",
                json!({}),
            )
        }
    };
    if let Err(response) =
        authorize::<AdminApiKeyView>(&context, "admin:developer:manage", &request_id)
    {
        return response;
    }
    if let Err(reason) = idempotency_key(&headers) {
        return error_response::<AdminApiKeyView>(
            &request_id,
            StatusCode::BAD_REQUEST,
            "Invalid idempotency key",
            reason,
            "invalid_idempotency_key",
            json!({}),
        );
    }
    if let Err(reason) = validate_reason(&body.reason) {
        return error_response::<AdminApiKeyView>(
            &request_id,
            StatusCode::UNPROCESSABLE_ENTITY,
            "Invalid revoke reason",
            reason,
            "validation_error",
            json!({"field":"reason"}),
        );
    }
    let id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return error_response::<AdminApiKeyView>(
                &request_id,
                StatusCode::BAD_REQUEST,
                "Invalid API key ID",
                "id must be a UUID",
                "invalid_identifier",
                json!({}),
            )
        }
    };
    let repo = ApiKeyRepository::new(*state.db_pool);
    match repo
        .revoke(
            id,
            RevokeApiKeyRequest {
                reason: body.reason.clone(),
                revoked_by: context.wallet_address.clone(),
            },
        )
        .await
    {
        Ok(key) => {
            if state
                .audit
                .log_sync(
                    &AuditCtx::from_wallet(&context.wallet_address, &headers),
                    &AuditEntry::new("api_key", "revoke", "developer")
                        .id(&id.to_string())
                        .meta(json!({"request_id": request_id})),
                )
                .await
                .is_err()
            {
                return error_response::<AdminApiKeyView>(
                    &request_id,
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Revoke pending",
                    "The audit record could not be durably written",
                    "audit_write_failed",
                    json!({}),
                );
            }
            response_with_id(
                UnifiedApiResponse::success(AdminApiKeyView::from(key)),
                &request_id,
                None,
            )
        }
        Err(_) => error_response::<AdminApiKeyView>(
            &request_id,
            StatusCode::BAD_GATEWAY,
            "Developer portal unavailable",
            "The API key could not be revoked",
            "repository_write_failed",
            json!({}),
        ),
    }
}

pub async fn update_expiration_handler(
    State(state): State<AppState>,
    Extension(context): Extension<crate::web::middleware::OpenIDUserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<UpdateExpirationBody>,
) -> Response {
    let request_id = match correlation_id(&headers) {
        Ok(id) => id,
        Err(reason) => {
            return error_response::<()>(
                "generated",
                StatusCode::BAD_REQUEST,
                "Invalid request ID",
                reason,
                "invalid_request_id",
                json!({}),
            )
        }
    };
    if let Err(response) =
        authorize::<AdminApiKeyView>(&context, "admin:developer:manage", &request_id)
    {
        return response;
    }
    if let Err(reason) = idempotency_key(&headers) {
        return error_response::<AdminApiKeyView>(
            &request_id,
            StatusCode::BAD_REQUEST,
            "Invalid idempotency key",
            reason,
            "invalid_idempotency_key",
            json!({}),
        );
    }
    let id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return error_response::<AdminApiKeyView>(
                &request_id,
                StatusCode::BAD_REQUEST,
                "Invalid API key ID",
                "id must be a UUID",
                "invalid_identifier",
                json!({}),
            )
        }
    };
    let expires_at = match valid_expiration(body.expires_at.as_deref()) {
        Ok(value) => value,
        Err(reason) => {
            return error_response::<AdminApiKeyView>(
                &request_id,
                StatusCode::UNPROCESSABLE_ENTITY,
                "Invalid expiration",
                reason,
                "validation_error",
                json!({"field":"expires_at"}),
            )
        }
    };
    let repo = ApiKeyRepository::new(*state.db_pool);
    match repo.update_expiration(id, expires_at).await {
        Ok(key) => {
            if state
                .audit
                .log_sync(
                    &AuditCtx::from_wallet(&context.wallet_address, &headers),
                    &AuditEntry::new("api_key", "expiration_update", "developer")
                        .id(&id.to_string())
                        .meta(json!({"request_id": request_id, "expires_at": expires_at})),
                )
                .await
                .is_err()
            {
                return error_response::<AdminApiKeyView>(
                    &request_id,
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Expiration update pending",
                    "The audit record could not be durably written",
                    "audit_write_failed",
                    json!({}),
                );
            }
            response_with_id(
                UnifiedApiResponse::success(AdminApiKeyView::from(key)),
                &request_id,
                None,
            )
        }
        Err(_) => error_response::<AdminApiKeyView>(
            &request_id,
            StatusCode::BAD_GATEWAY,
            "Developer portal unavailable",
            "The API key expiration could not be updated",
            "repository_write_failed",
            json!({}),
        ),
    }
}

pub async fn list_expiring_keys_handler(
    State(state): State<AppState>,
    Extension(context): Extension<crate::web::middleware::OpenIDUserContext>,
    headers: HeaderMap,
    Query(query): Query<ListExpiringKeysQuery>,
) -> Response {
    let request_id = match correlation_id(&headers) {
        Ok(id) => id,
        Err(reason) => {
            return error_response::<()>(
                "generated",
                StatusCode::BAD_REQUEST,
                "Invalid request ID",
                reason,
                "invalid_request_id",
                json!({}),
            )
        }
    };
    if let Err(response) =
        authorize::<ExpiringKeysResponse>(&context, "admin:developer:read", &request_id)
    {
        return response;
    }
    let days = match validate_days(query.days) {
        Ok(days) => days,
        Err(reason) => {
            return error_response::<ExpiringKeysResponse>(
                &request_id,
                StatusCode::BAD_REQUEST,
                "Invalid expiration window",
                reason,
                "invalid_query",
                json!({}),
            )
        }
    };
    let (limit, offset) = match validate_pagination(query.limit, query.offset) {
        Ok(value) => value,
        Err(reason) => {
            return error_response::<ExpiringKeysResponse>(
                &request_id,
                StatusCode::BAD_REQUEST,
                "Invalid pagination",
                reason,
                "invalid_query",
                json!({}),
            )
        }
    };
    let repo = ApiKeyRepository::new(*state.db_pool);
    match repo
        .list_expiring_keys(days, Some(limit), Some(offset))
        .await
    {
        Ok((api_keys, total)) => response_with_id(
            UnifiedApiResponse::success(ExpiringKeysResponse {
                api_keys: api_keys.into_iter().map(AdminApiKeyView::from).collect(),
                total,
                days_ahead: days,
            }),
            &request_id,
            None,
        ),
        Err(_) => error_response::<ExpiringKeysResponse>(
            &request_id,
            StatusCode::BAD_GATEWAY,
            "Developer portal unavailable",
            "The expiring-key inventory is unavailable",
            "repository_read_failed",
            json!({}),
        ),
    }
}

// ============================================================================
// Module Handlers
// ============================================================================

/// GET /api/admin/developer-portal/modules
pub async fn list_modules_handler(
    State(state): State<AppState>,
    Query(query): Query<ListModulesQuery>,
) -> impl IntoResponse {
    let pool = *state.db_pool;
    let repo = ModuleRepository::new(pool);

    match repo
        .list(query.status.as_deref(), query.category.as_deref())
        .await
    {
        Ok(response) => UnifiedApiResponse::success(response),
        Err(e) => {
            error!("Failed to list modules: {}", e);
            UnifiedApiResponse::server_error(&e.to_string())
        }
    }
}

/// GET /api/admin/developer-portal/modules/:id
pub async fn get_module_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let pool = *state.db_pool;
    let repo = ModuleRepository::new(pool);

    let uuid = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => {
            return UnifiedApiResponse::error(
                400,
                "Invalid UUID",
                "The provided ID is not a valid UUID",
            )
        }
    };

    match repo.get_by_id(uuid).await {
        Ok(Some(module)) => UnifiedApiResponse::success(module),
        Ok(None) => UnifiedApiResponse::not_found("Module"),
        Err(e) => {
            error!("Failed to get module: {}", e);
            UnifiedApiResponse::server_error(&e.to_string())
        }
    }
}

/// POST /api/admin/developer-portal/modules
pub async fn create_module_handler(
    State(state): State<AppState>,
    axum::Extension(user_ctx): axum::Extension<
        crate::web::middleware::bearer_middleware::OpenIDUserContext,
    >,
    headers: HeaderMap,
    Json(body): Json<CreateModuleBody>,
) -> impl IntoResponse {
    let pool = *state.db_pool;
    let repo = ModuleRepository::new(pool);

    let request = CreateModuleRequest {
        name: body.name.clone(),
        display_name: body.display_name.clone(),
        description: body.description.clone(),
        category: body.category.clone(),
        base_path: body.base_path.clone(),
        default_rate_limit: body.default_rate_limit,
        access_levels: body.access_levels.clone(),
        endpoints: body.endpoints.clone(),
    };

    match repo.create(request).await {
        Ok(module) => {
            info!("Created module: {}", module.id);

            // Audit log
            let ctx = AuditCtx::from_wallet(&user_ctx.wallet_address, &headers);
            state.audit.log(
                ctx,
                AuditEntry::new("module", "create", "developer")
                    .id(&module.id.to_string())
                    .after(serde_json::json!({
                        "name": body.name,
                        "display_name": body.display_name,
                        "category": body.category,
                        "base_path": body.base_path,
                        "default_rate_limit": body.default_rate_limit,
                    })),
            );

            UnifiedApiResponse::success(module)
        }
        Err(e) => {
            error!("Failed to create module: {}", e);
            UnifiedApiResponse::server_error(&e.to_string())
        }
    }
}

/// PUT /api/admin/developer-portal/modules/:id
pub async fn update_module_handler(
    State(state): State<AppState>,
    axum::Extension(user_ctx): axum::Extension<
        crate::web::middleware::bearer_middleware::OpenIDUserContext,
    >,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<UpdateModuleBody>,
) -> impl IntoResponse {
    let pool = *state.db_pool;
    let repo = ModuleRepository::new(pool);

    let uuid = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => {
            return UnifiedApiResponse::error(
                400,
                "Invalid UUID",
                "The provided ID is not a valid UUID",
            )
        }
    };

    let request = UpdateModuleRequest {
        display_name: body.display_name.clone(),
        description: body.description.clone(),
        status: body.status.clone(),
        default_rate_limit: body.default_rate_limit,
        access_levels: body.access_levels.clone(),
        endpoints: body.endpoints.clone(),
    };

    match repo.update(uuid, request).await {
        Ok(module) => {
            info!("Updated module: {}", uuid);

            // Audit log
            let ctx = AuditCtx::from_wallet(&user_ctx.wallet_address, &headers);
            state.audit.log(
                ctx,
                AuditEntry::new("module", "update", "developer")
                    .id(&uuid.to_string())
                    .after(serde_json::json!({
                        "display_name": body.display_name,
                        "description": body.description,
                        "status": body.status,
                        "default_rate_limit": body.default_rate_limit,
                    })),
            );

            UnifiedApiResponse::success(module)
        }
        Err(e) => {
            error!("Failed to update module: {}", e);
            UnifiedApiResponse::server_error(&e.to_string())
        }
    }
}

// ============================================================================
// Stats Handler
// ============================================================================

/// GET /api/admin/developer-portal/stats
pub async fn get_stats_handler(State(state): State<AppState>) -> impl IntoResponse {
    let core_pool = *state.db_pool;
    let api_key_repo = ApiKeyRepository::new(core_pool);
    let module_repo = ModuleRepository::new(core_pool);

    // Get authoritative counts from the database; do not classify a bounded
    // page of keys as if it were the complete inventory.
    let (total_keys, active_count, revoked_count, expired_count) = match api_key_repo.counts().await
    {
        Ok(result) => result,
        Err(e) => return UnifiedApiResponse::server_error(&e.to_string()),
    };

    // Get module counts
    let modules = match module_repo.list(None, None).await {
        Ok(result) => result,
        Err(e) => return UnifiedApiResponse::server_error(&e.to_string()),
    };

    let active_modules = modules
        .modules
        .iter()
        .filter(|m| m.status == crate::domain::developer_portal::ModuleStatus::Active)
        .count() as i64;

    // Get usage statistics from analytics database
    let (total_requests_today, total_requests_this_month, top_modules_by_usage) =
        match get_analytics_pool().await {
            Ok(analytics_pool) => {
                let usage_service = UsageService::new(core_pool, analytics_pool);

                let today = match usage_service.get_requests_today().await {
                    Ok(value) => value,
                    Err(error) => {
                        error!("Failed to get today's developer usage: {}", error);
                        return UnifiedApiResponse::error(
                            503,
                            "Developer portal unavailable",
                            "Usage analytics are temporarily unavailable",
                        );
                    }
                };
                let month = match usage_service.get_requests_this_month().await {
                    Ok(value) => value,
                    Err(error) => {
                        error!("Failed to get this month's developer usage: {}", error);
                        return UnifiedApiResponse::error(
                            503,
                            "Developer portal unavailable",
                            "Usage analytics are temporarily unavailable",
                        );
                    }
                };
                let top_modules = match usage_service.get_top_modules_by_usage(5).await {
                    Ok(value) => value,
                    Err(error) => {
                        error!("Failed to get top developer modules: {}", error);
                        return UnifiedApiResponse::error(
                            503,
                            "Developer portal unavailable",
                            "Usage analytics are temporarily unavailable",
                        );
                    }
                }
                .into_iter()
                .map(|m| crate::domain::developer_portal::ModuleUsageStats {
                    module_id: m.module_id,
                    module_name: m.module_name,
                    request_count: m.request_count,
                    unique_api_keys: m.unique_api_keys,
                })
                .collect();

                (today, month, top_modules)
            }
            Err(e) => {
                error!("Failed to get analytics pool: {}", e);
                return UnifiedApiResponse::error(
                    503,
                    "Developer portal unavailable",
                    "Usage analytics are temporarily unavailable",
                );
            }
        };

    let stats = DeveloperPortalStats {
        total_api_keys: total_keys,
        active_api_keys: active_count,
        revoked_api_keys: revoked_count,
        expired_api_keys: expired_count,
        total_modules: modules.total,
        active_modules,
        total_requests_today,
        total_requests_this_month,
        top_modules_by_usage,
    };

    UnifiedApiResponse::success(stats)
}
