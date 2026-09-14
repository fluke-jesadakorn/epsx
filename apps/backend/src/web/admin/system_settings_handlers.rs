//! System Settings Handlers
//!
//! Provides handlers for managing global admin console settings.
//! Settings are stored in system_settings table and are NOT tied to any specific wallet.

use axum::{
    extract::{Extension, Path, State},
    http::HeaderMap,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, info};
use utoipa::ToSchema;

use crate::web::auth::AppState;
use crate::web::middleware::bearer_middleware::OpenIDUserContext;
use epsx_contracts::errors::{AppError, ErrorKind};

const ADMIN_AUDIENCE: &str = "epsx-admin";
const SETTINGS_READ_PERMISSION: &str = "admin:settings:read";
const SETTINGS_MANAGE_PERMISSION: &str = "admin:settings:manage";
const MAX_SETTINGS: usize = 100;
const MAX_CATEGORY_CHARS: usize = 64;
const MAX_KEY_CHARS: usize = 128;
const MAX_VALUE_BYTES: usize = 32 * 1024;
const MAX_IDEMPOTENCY_KEY_CHARS: usize = 56;

// ============================================================================
// DTOs
// ============================================================================

/// Request to update settings
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateSettingsRequest {
    pub settings: Vec<SettingUpdate>,
}

/// Individual setting update
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SettingUpdate {
    pub category: String,
    pub key: String,
    pub value: Value,
    #[serde(default)]
    pub expected_updated_at: Option<String>,
}

/// Response for a single setting
#[derive(Debug, Serialize)]
pub struct SettingResponse {
    pub category: String,
    pub key: String,
    pub value: Value,
    pub description: Option<String>,
    pub updated_at: String,
}

/// Response for all settings
#[derive(Debug, Serialize)]
pub struct AllSettingsResponse {
    pub settings: std::collections::HashMap<String, std::collections::HashMap<String, Value>>,
}

/// Response for category settings
#[derive(Debug, Serialize)]
pub struct CategorySettingsResponse {
    pub category: String,
    pub settings: std::collections::HashMap<String, Value>,
}

// ============================================================================
// Database Types (inline for simplicity)
// ============================================================================

#[derive(Debug, sqlx::FromRow)]
struct SystemSettingRow {
    #[allow(dead_code)]
    pub id: i32,
    pub category: String,
    pub key: String,
    pub value: Value,
    #[allow(dead_code)]
    pub description: Option<String>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

fn request_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .filter(|value| bounded_text(value, 128))
        .map(str::to_string)
}

fn bounded_text(value: &str, max_chars: usize) -> bool {
    !value.trim().is_empty()
        && value.chars().count() <= max_chars
        && !value.chars().any(char::is_control)
}

fn safe_setting_token(value: &str, max_chars: usize) -> bool {
    (1..=max_chars).contains(&value.chars().count())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn valid_idempotency_key(value: &str) -> bool {
    (1..=MAX_IDEMPOTENCY_KEY_CHARS).contains(&value.chars().count())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn authorize(
    context: &OpenIDUserContext,
    permission: &str,
    operation: &'static str,
    headers: &HeaderMap,
) -> Result<(), AppError> {
    let correlation_id = request_id(headers);
    if !matches!(
        context.token_audiences.as_deref(),
        Some([audience]) if audience == ADMIN_AUDIENCE
    ) {
        return Err(AppError::with_full_context(
            ErrorKind::AuthenticationError,
            "A valid admin audience is required",
            Some(context.wallet_address.clone()),
            correlation_id,
            operation,
            "admin-settings",
        ));
    }
    if !epsx_contracts::permissions::has_permission(&context.permissions, permission) {
        return Err(AppError::with_full_context(
            ErrorKind::AuthorizationError,
            "The required settings permission is missing",
            Some(context.wallet_address.clone()),
            request_id(headers),
            operation,
            "admin-settings",
        ));
    }
    Ok(())
}

fn validate_updates(request: &UpdateSettingsRequest) -> Result<(), AppError> {
    if request.settings.is_empty() || request.settings.len() > MAX_SETTINGS {
        return Err(AppError::bad_request(
            "settings must contain 1 to 100 entries",
        ));
    }
    for setting in &request.settings {
        if !safe_setting_token(&setting.category, MAX_CATEGORY_CHARS)
            || !safe_setting_token(&setting.key, MAX_KEY_CHARS)
        {
            return Err(AppError::bad_request(
                "setting category and key are invalid",
            ));
        }
        if serde_json::to_vec(&setting.value)
            .map(|value| value.len() > MAX_VALUE_BYTES)
            .unwrap_or(true)
        {
            return Err(AppError::bad_request(
                "setting value exceeds the size limit",
            ));
        }
        if let Some(expected) = setting.expected_updated_at.as_deref() {
            if !bounded_text(expected, 64)
                || expected.parse::<chrono::DateTime<chrono::Utc>>().is_err()
            {
                return Err(AppError::bad_request("expected_updated_at must be RFC3339"));
            }
        }
    }
    Ok(())
}

// ============================================================================
// Handlers
// ============================================================================

/// Get all system settings
/// GET /api/admin/settings
#[utoipa::path(
    get,
    path = "/api/admin/settings",
    responses(
        (status = 200, description = "All system settings", body = Value),
        (status = 500, description = "Internal server error")
    ),
    tag = "admin-settings"
)]
pub async fn get_all_settings_handler(
    State(app_state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
    headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
    authorize(
        &context,
        SETTINGS_READ_PERMISSION,
        "admin.settings.read",
        &headers,
    )?;
    info!("Getting all system settings");

    // Query all settings from database
    let rows: Vec<SystemSettingRow> = sqlx::query_as::<_, SystemSettingRow>(
        "SELECT id, category, key, value, description, updated_at FROM system_settings ORDER BY category, key"
    )
    .fetch_all(app_state.db_pool.as_ref())
    .await
    .map_err(|e| {
        error!("Failed to query settings: {}", e);
        AppError::new(ErrorKind::DatabaseError, format!("Failed to query settings: {}", e))
    })?;

    // Plan settings by category
    let mut settings: std::collections::HashMap<String, std::collections::HashMap<String, Value>> =
        std::collections::HashMap::new();

    for row in rows {
        let category_settings = settings.entry(row.category.clone()).or_default();
        category_settings.insert(row.key, row.value);
    }

    info!("Retrieved {} categories of settings", settings.len());

    Ok(Json(json!({
        "success": true,
        "data": settings
    })))
}

/// Get settings by category
/// GET /api/admin/settings/:category
#[utoipa::path(
    get,
    path = "/api/admin/settings/{category}",
    params(
        ("category" = String, Path, description = "Settings category (general, notifications, security, appearance)")
    ),
    responses(
        (status = 200, description = "Category settings", body = Value),
        (status = 404, description = "Category not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "admin-settings"
)]
pub async fn get_settings_by_category_handler(
    State(app_state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
    Path(category): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
    authorize(
        &context,
        SETTINGS_READ_PERMISSION,
        "admin.settings.read_category",
        &headers,
    )?;
    if !safe_setting_token(&category, MAX_CATEGORY_CHARS) {
        return Err(AppError::bad_request("setting category is invalid"));
    }
    info!("Getting settings for category: {}", category);

    // Query settings for specific category
    let rows: Vec<SystemSettingRow> = sqlx::query_as::<_, SystemSettingRow>(
        "SELECT id, category, key, value, description, updated_at FROM system_settings WHERE category = $1"
    )
    .bind(&category)
    .fetch_all(app_state.db_pool.as_ref())
    .await
    .map_err(|e| {
        error!("Failed to query settings: {}", e);
        AppError::new(ErrorKind::DatabaseError, format!("Failed to query settings: {}", e))
    })?;

    // Build response
    let mut settings: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
    for row in rows {
        settings.insert(row.key, row.value);
    }

    info!(
        "Retrieved {} settings for category: {}",
        settings.len(),
        category
    );

    Ok(Json(json!({
        "success": true,
        "data": {
            "category": category,
            "settings": settings
        }
    })))
}

/// Update system settings (bulk)
/// PUT /api/admin/settings
#[utoipa::path(
    put,
    path = "/api/admin/settings",
    request_body = UpdateSettingsRequest,
    responses(
        (status = 200, description = "Settings updated successfully", body = Value),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "admin-settings"
)]
pub async fn update_settings_handler(
    State(app_state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
    headers: HeaderMap,
    Json(request): Json<UpdateSettingsRequest>,
) -> Result<Json<Value>, AppError> {
    authorize(
        &context,
        SETTINGS_MANAGE_PERMISSION,
        "admin.settings.update",
        &headers,
    )?;
    validate_updates(&request)?;
    let idempotency_key = headers
        .get("idempotency-key")
        .and_then(|value| value.to_str().ok())
        .filter(|value| valid_idempotency_key(value))
        .ok_or_else(|| AppError::bad_request("a valid Idempotency-Key header is required"))?;
    let response_request_id =
        request_id(&headers).unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    info!("Updating {} settings", request.settings.len());

    let mut tx = app_state.db_pool.begin().await.map_err(|e| {
        error!("Failed to get DB transaction: {}", e);
        AppError::database_error("Failed to get DB transaction")
    })?;

    sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1))")
        .bind(idempotency_key)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::database_error(format!("Lock error: {}", e)))?;

    let mut updated_count = 0;
    for setting in request.settings {
        let existing: Option<SystemSettingRow> = sqlx::query_as::<_, SystemSettingRow>(
            "SELECT id, category, key, value, description, updated_at
             FROM system_settings WHERE category = $1 AND key = $2 FOR UPDATE",
        )
        .bind(&setting.category)
        .bind(&setting.key)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| AppError::database_error(format!("Failed to query setting: {}", e)))?;

        let expected = setting
            .expected_updated_at
            .as_deref()
            .map(|value| value.parse::<chrono::DateTime<chrono::Utc>>())
            .transpose()
            .map_err(|_| {
                AppError::new(
                    ErrorKind::ConcurrencyConflict,
                    "settings changed; reload before retrying",
                )
            })?;

        match (&existing, expected) {
            (Some(row), Some(expected)) if row.updated_at != expected => {
                return Err(AppError::new(
                    ErrorKind::ConcurrencyConflict,
                    "settings changed; reload before retrying",
                ));
            }
            (Some(_), None) | (None, Some(_)) => {
                return Err(AppError::new(
                    ErrorKind::ConcurrencyConflict,
                    "settings changed; reload before retrying",
                ));
            }
            _ => {}
        }

        if existing.is_some() {
            sqlx::query(
                "UPDATE system_settings SET value = $3, updated_at = NOW()
                 WHERE category = $1 AND key = $2",
            )
            .bind(&setting.category)
            .bind(&setting.key)
            .bind(&setting.value)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to update setting: {}", e)))?;
        } else {
            sqlx::query(
                "INSERT INTO system_settings (category, key, value, updated_at)
                 VALUES ($1, $2, $3, NOW())",
            )
            .bind(&setting.category)
            .bind(&setting.key)
            .bind(&setting.value)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to insert setting: {}", e)))?;
        }
        updated_count += 1;
    }

    tx.commit()
        .await
        .map_err(|e| AppError::database_error(format!("Failed to commit transaction: {}", e)))?;

    Ok(Json(json!({
        "success": true,
        "message": "Settings updated",
        "updated_count": updated_count,
        "request_id": response_request_id
    })))
}

/// Reset settings to defaults
/// POST /api/admin/settings/reset
#[utoipa::path(
    post,
    path = "/api/admin/settings/reset",
    responses(
        (status = 200, description = "Settings reset to defaults", body = Value),
        (status = 500, description = "Internal server error")
    ),
    tag = "admin-settings"
)]
pub async fn reset_settings_handler(
    State(app_state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
    headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
    authorize(
        &context,
        SETTINGS_MANAGE_PERMISSION,
        "admin.settings.reset",
        &headers,
    )?;
    // These values are the forward-compatible defaults seeded by the core
    // migration. Reset delegates to the same versioned, idempotent update
    // transaction as an ordinary write, so it cannot report success before
    // the database commit.
    let request = UpdateSettingsRequest {
        settings: vec![
            SettingUpdate {
                category: "general".into(),
                key: "systemName".into(),
                value: json!("EPSX Admin Console"),
                expected_updated_at: None,
            },
            SettingUpdate {
                category: "general".into(),
                key: "adminEmail".into(),
                value: json!("admin@epsx.com"),
                expected_updated_at: None,
            },
            SettingUpdate {
                category: "general".into(),
                key: "maintenanceMode".into(),
                value: json!(false),
                expected_updated_at: None,
            },
            SettingUpdate {
                category: "notifications".into(),
                key: "emailNotifications".into(),
                value: json!(true),
                expected_updated_at: None,
            },
            SettingUpdate {
                category: "notifications".into(),
                key: "pushNotifications".into(),
                value: json!(false),
                expected_updated_at: None,
            },
            SettingUpdate {
                category: "notifications".into(),
                key: "smsNotifications".into(),
                value: json!(true),
                expected_updated_at: None,
            },
            SettingUpdate {
                category: "notifications".into(),
                key: "securityAlerts".into(),
                value: json!(true),
                expected_updated_at: None,
            },
            SettingUpdate {
                category: "security".into(),
                key: "sessionTimeout".into(),
                value: json!(30),
                expected_updated_at: None,
            },
            SettingUpdate {
                category: "appearance".into(),
                key: "theme".into(),
                value: json!("light"),
                expected_updated_at: None,
            },
            SettingUpdate {
                category: "appearance".into(),
                key: "primaryColor".into(),
                value: json!("#FF8C00"),
                expected_updated_at: None,
            },
        ],
    };
    update_settings_handler(State(app_state), Extension(context), headers, Json(request)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_request() -> UpdateSettingsRequest {
        UpdateSettingsRequest {
            settings: vec![SettingUpdate {
                category: "general".into(),
                key: "systemName".into(),
                value: json!("EPSX"),
                expected_updated_at: None,
            }],
        }
    }

    fn context(audiences: Option<Vec<&str>>, permissions: Vec<&str>) -> OpenIDUserContext {
        OpenIDUserContext {
            sub: "0xadmin".into(),
            wallet_address: "0xadmin".into(),
            permissions: permissions.into_iter().map(str::to_string).collect(),
            token_audiences: audiences
                .map(|values| values.into_iter().map(str::to_string).collect()),
            api_key: None,
            auth_method: "jwt".into(),
            jti: "jti".into(),
            exp: 2,
            iat: 1,
            auth_time: 1,
        }
    }

    #[test]
    fn settings_validation_rejects_unbounded_or_malformed_writes() {
        assert!(validate_updates(&valid_request()).is_ok());
        assert!(validate_updates(&UpdateSettingsRequest { settings: vec![] }).is_err());

        let mut invalid = valid_request();
        invalid.settings[0].category = "general/settings".into();
        assert!(validate_updates(&invalid).is_err());
        invalid = valid_request();
        invalid.settings[0].expected_updated_at = Some("not-a-timestamp".into());
        assert!(validate_updates(&invalid).is_err());
        invalid = valid_request();
        invalid.settings[0].value = json!("x".repeat(MAX_VALUE_BYTES + 1));
        assert!(validate_updates(&invalid).is_err());
    }

    #[test]
    fn settings_authorization_requires_exact_admin_audience_and_granular_permission() {
        let headers = HeaderMap::new();
        assert!(authorize(
            &context(Some(vec!["epsx-admin"]), vec![SETTINGS_READ_PERMISSION]),
            SETTINGS_READ_PERMISSION,
            "test.read",
            &headers,
        )
        .is_ok());
        assert!(authorize(
            &context(Some(vec!["epsx-frontend"]), vec![SETTINGS_READ_PERMISSION]),
            SETTINGS_READ_PERMISSION,
            "test.read",
            &headers,
        )
        .is_err());
        assert!(authorize(
            &context(Some(vec!["epsx-admin"]), vec![]),
            SETTINGS_READ_PERMISSION,
            "test.read",
            &headers,
        )
        .is_err());
        assert!(authorize(
            &context(
                Some(vec!["epsx-admin", "epsx-frontend"]),
                vec![SETTINGS_READ_PERMISSION]
            ),
            SETTINGS_READ_PERMISSION,
            "test.read",
            &headers,
        )
        .is_err());
    }
}
