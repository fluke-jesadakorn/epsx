//! System Settings Handlers
//!
//! Provides handlers for managing global admin console settings.
//! Settings are stored in system_settings table and are NOT tied to any specific wallet.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, info};
use utoipa::ToSchema;

use crate::web::auth::AppState;

// ============================================================================
// DTOs
// ============================================================================

/// Request to update settings
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSettingsRequest {
    pub settings: Vec<SettingUpdate>,
}

/// Individual setting update
#[derive(Debug, Deserialize, ToSchema)]
pub struct SettingUpdate {
    pub category: String,
    pub key: String,
    pub value: Value,
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

#[derive(Debug, QueryableByName)]
struct SystemSettingRow {
    #[allow(dead_code)]
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub id: i32,
    #[diesel(sql_type = diesel::sql_types::Varchar)]
    pub category: String,
    #[diesel(sql_type = diesel::sql_types::Varchar)]
    pub key: String,
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    pub value: Value,
    #[allow(dead_code)]
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub description: Option<String>,
    #[allow(dead_code)]
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// Default Settings
// ============================================================================

fn get_default_settings() -> std::collections::HashMap<String, std::collections::HashMap<String, Value>> {
    let mut defaults = std::collections::HashMap::new();
    
    // General settings
    let mut general = std::collections::HashMap::new();
    general.insert("systemName".to_string(), json!("EPSX Admin Console"));
    general.insert("adminEmail".to_string(), json!("admin@epsx.com"));
    general.insert("maintenanceMode".to_string(), json!(false));
    defaults.insert("general".to_string(), general);
    
    // Notification settings
    let mut notifications = std::collections::HashMap::new();
    notifications.insert("emailNotifications".to_string(), json!(true));
    notifications.insert("pushNotifications".to_string(), json!(false));
    notifications.insert("smsNotifications".to_string(), json!(true));
    notifications.insert("securityAlerts".to_string(), json!(true));
    defaults.insert("notifications".to_string(), notifications);
    
    // Security settings
    let mut security = std::collections::HashMap::new();
    security.insert("sessionTimeout".to_string(), json!(30));
    defaults.insert("security".to_string(), security);
    
    // Appearance settings
    let mut appearance = std::collections::HashMap::new();
    appearance.insert("theme".to_string(), json!("light"));
    appearance.insert("primaryColor".to_string(), json!("#FF8C00"));
    defaults.insert("appearance".to_string(), appearance);
    
    defaults
}

// ============================================================================
// Handlers
// ============================================================================

/// Get all system settings
/// GET /api/v1/admin/settings
#[utoipa::path(
    get,
    path = "/api/v1/admin/settings",
    responses(
        (status = 200, description = "All system settings", body = Value),
        (status = 500, description = "Internal server error")
    ),
    tag = "admin-settings"
)]
pub async fn get_all_settings_handler(
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    info!("📋 Getting all system settings");
    
    let mut conn = app_state.db_pool.get().await.map_err(|e| {
        error!("❌ Failed to get DB connection: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Query all settings from database
    let rows: Vec<SystemSettingRow> = diesel::sql_query(
        "SELECT id, category, key, value, description, updated_at FROM system_settings ORDER BY category, key"
    )
    .load(&mut conn)
    .await
    .map_err(|e| {
        error!("❌ Failed to query settings: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Group settings by category
    let mut settings: std::collections::HashMap<String, std::collections::HashMap<String, Value>> = 
        std::collections::HashMap::new();
    
    for row in rows {
        let category_settings = settings.entry(row.category.clone()).or_insert_with(std::collections::HashMap::new);
        category_settings.insert(row.key, row.value);
    }
    
    // If no settings in DB, return defaults
    if settings.is_empty() {
        settings = get_default_settings();
    }
    
    info!("✅ Retrieved {} categories of settings", settings.len());
    
    Ok(Json(json!({
        "success": true,
        "data": settings
    })))
}

/// Get settings by category
/// GET /api/v1/admin/settings/:category
#[utoipa::path(
    get,
    path = "/api/v1/admin/settings/{category}",
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
    Path(category): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    info!("📋 Getting settings for category: {}", category);
    
    let mut conn = app_state.db_pool.get().await.map_err(|e| {
        error!("❌ Failed to get DB connection: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Query settings for specific category
    let rows: Vec<SystemSettingRow> = diesel::sql_query(
        "SELECT id, category, key, value, description, updated_at FROM system_settings WHERE category = $1"
    )
    .bind::<diesel::sql_types::Varchar, _>(&category)
    .load(&mut conn)
    .await
    .map_err(|e| {
        error!("❌ Failed to query settings: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Build response
    let mut settings: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
    for row in rows {
        settings.insert(row.key, row.value);
    }
    
    // If no settings found, check defaults
    if settings.is_empty() {
        let defaults = get_default_settings();
        if let Some(default_category) = defaults.get(&category) {
            settings = default_category.clone();
        }
    }
    
    info!("✅ Retrieved {} settings for category: {}", settings.len(), category);
    
    Ok(Json(json!({
        "success": true,
        "data": {
            "category": category,
            "settings": settings
        }
    })))
}

/// Update system settings (bulk)
/// PUT /api/v1/admin/settings
#[utoipa::path(
    put,
    path = "/api/v1/admin/settings",
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
    Json(request): Json<UpdateSettingsRequest>,
) -> Result<Json<Value>, StatusCode> {
    info!("📝 Updating {} settings", request.settings.len());
    
    let mut conn = app_state.db_pool.get().await.map_err(|e| {
        error!("❌ Failed to get DB connection: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let mut updated_count = 0;
    let mut errors: Vec<String> = Vec::new();
    
    for setting in &request.settings {
        let result = diesel::sql_query(
            "INSERT INTO system_settings (category, key, value, updated_at) 
             VALUES ($1, $2, $3, NOW()) 
             ON CONFLICT (category, key) 
             DO UPDATE SET value = $3, updated_at = NOW()"
        )
        .bind::<diesel::sql_types::Varchar, _>(&setting.category)
        .bind::<diesel::sql_types::Varchar, _>(&setting.key)
        .bind::<diesel::sql_types::Jsonb, _>(&setting.value)
        .execute(&mut conn)
        .await;
        
        match result {
            Ok(_) => {
                updated_count += 1;
                info!("✅ Updated setting: {}.{}", setting.category, setting.key);
            }
            Err(e) => {
                error!("❌ Failed to update {}.{}: {}", setting.category, setting.key, e);
                errors.push(format!("{}.{}: {}", setting.category, setting.key, e));
            }
        }
    }
    
    if errors.is_empty() {
        Ok(Json(json!({
            "success": true,
            "message": format!("Updated {} settings", updated_count),
            "updated_count": updated_count
        })))
    } else {
        Ok(Json(json!({
            "success": false,
            "message": "Some settings failed to update",
            "updated_count": updated_count,
            "errors": errors
        })))
    }
}

/// Reset settings to defaults
/// POST /api/v1/admin/settings/reset
#[utoipa::path(
    post,
    path = "/api/v1/admin/settings/reset",
    responses(
        (status = 200, description = "Settings reset to defaults", body = Value),
        (status = 500, description = "Internal server error")
    ),
    tag = "admin-settings"
)]
pub async fn reset_settings_handler(
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    info!("🔄 Resetting all settings to defaults");
    
    let mut conn = app_state.db_pool.get().await.map_err(|e| {
        error!("❌ Failed to get DB connection: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Delete all existing settings
    diesel::sql_query("DELETE FROM system_settings")
        .execute(&mut conn)
        .await
        .map_err(|e| {
            error!("❌ Failed to delete settings: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Insert default settings
    let defaults = get_default_settings();
    let mut inserted_count = 0;
    
    for (category, settings) in &defaults {
        for (key, value) in settings {
            let result = diesel::sql_query(
                "INSERT INTO system_settings (category, key, value, updated_at) VALUES ($1, $2, $3, NOW())"
            )
            .bind::<diesel::sql_types::Varchar, _>(category)
            .bind::<diesel::sql_types::Varchar, _>(key)
            .bind::<diesel::sql_types::Jsonb, _>(value)
            .execute(&mut conn)
            .await;
            
            if result.is_ok() {
                inserted_count += 1;
            }
        }
    }
    
    info!("✅ Reset {} settings to defaults", inserted_count);
    
    Ok(Json(json!({
        "success": true,
        "message": format!("Reset {} settings to defaults", inserted_count),
        "data": defaults
    })))
}
