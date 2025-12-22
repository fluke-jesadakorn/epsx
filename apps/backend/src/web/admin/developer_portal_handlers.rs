//! Developer Portal API Handlers
//!
//! REST endpoints for API key and module management.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    Extension,
};
use serde::{Deserialize, Serialize};
use tracing::{info, error};
use uuid::Uuid;

use crate::domain::developer_portal::{
    CreateApiKeyRequest, RevokeApiKeyRequest,
    CreateModuleRequest, UpdateModuleRequest, ModuleAccessRequest,
    DeveloperPortalStats,
};
use crate::infrastructure::adapters::repositories::developer_portal::{ApiKeyRepository, ModuleRepository};
use crate::web::auth::AppState;
use crate::web::responses::UnifiedApiResponse;

// ============================================================================
// Request/Response DTOs
// ============================================================================

#[derive(Debug, Deserialize)]
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
pub struct CreateApiKeyBody {
    pub client_name: String,
    pub client_description: Option<String>,
    pub client_contact_email: Option<String>,
    pub wallet_address: String,
    pub allowed_modules: Vec<ModuleAccessInput>,
    pub ip_restrictions: Option<Vec<String>>,
    pub rate_limit_per_minute: Option<i32>,
    pub rate_limit_per_day: Option<i32>,
    pub expires_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ModuleAccessInput {
    pub module_id: String,
    pub access_level: String,
    pub custom_quotas: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct RevokeApiKeyBody {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
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
    pub api_keys: Vec<crate::domain::developer_portal::ApiKey>,
    pub total: i64,
}

#[derive(Debug, Deserialize)]
pub struct ListExpiringKeysQuery {
    /// Number of days to look ahead for expiring keys (default: 7)
    pub days: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ExpiringKeysResponse {
    pub api_keys: Vec<crate::domain::developer_portal::ApiKey>,
    pub total: i64,
    pub days_ahead: i64,
}

// ============================================================================
// API Key Handlers
// ============================================================================

/// GET /api/v1/admin/developer-portal/api-keys
/// Supports ?wallet=0x... to filter by wallet address
pub async fn list_api_keys_handler(
    State(state): State<AppState>,
    Query(query): Query<ListApiKeysQuery>,
) -> impl IntoResponse {
    let pool = *state.db_pool;
    let repo = ApiKeyRepository::new(pool);
    
    // If wallet filter is provided, use list_by_wallet, otherwise use list_all
    let result = if let Some(wallet) = &query.wallet {
        repo.list_by_wallet(wallet, query.limit, query.offset, query.status.as_deref()).await
    } else {
        repo.list_all(query.limit, query.offset, query.status.as_deref()).await
    };

    match result {
        Ok((api_keys, total)) => {
            UnifiedApiResponse::success(ApiKeyListResponse { api_keys, total })
        }
        Err(e) => {
            error!("Failed to list API keys: {}", e);
            UnifiedApiResponse::server_error(&e.to_string())
        }
    }
}

/// POST /api/v1/admin/developer-portal/api-keys
pub async fn create_api_key_handler(
    State(state): State<AppState>,
    Json(body): Json<CreateApiKeyBody>,
) -> impl IntoResponse {
    let pool = *state.db_pool;
    let repo = ApiKeyRepository::new(pool);
    
    // Parse expires_at if provided
    let expires_at = if let Some(expires_str) = &body.expires_at {
        chrono::DateTime::parse_from_rfc3339(expires_str)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .ok()
    } else {
        None
    };

    // Convert module access
    let allowed_modules: Vec<ModuleAccessRequest> = body.allowed_modules.iter().map(|m| {
        ModuleAccessRequest {
            module_id: Uuid::parse_str(&m.module_id).unwrap_or_default(),
            access_level: m.access_level.clone(),
            custom_quotas: m.custom_quotas.clone(),
        }
    }).collect();

    let request = CreateApiKeyRequest {
        client_name: body.client_name,
        client_description: body.client_description,
        client_contact_email: body.client_contact_email,
        wallet_address: body.wallet_address.clone(),
        allowed_modules,
        group_ids: vec![], // TODO: Add group_ids to admin create form
        ip_restrictions: body.ip_restrictions,
        rate_limit_per_minute: body.rate_limit_per_minute,
        rate_limit_per_day: body.rate_limit_per_day,
        expires_at,
        created_by: body.wallet_address, // Use wallet as creator for now
    };

    match repo.create(request).await {
        Ok(response) => {
            info!("Created API key: {}", response.api_key.id);
            UnifiedApiResponse::success(response)
        }
        Err(e) => {
            error!("Failed to create API key: {}", e);
            UnifiedApiResponse::server_error(&e.to_string())
        }
    }
}

/// GET /api/v1/admin/developer-portal/api-keys/:id
pub async fn get_api_key_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let pool = *state.db_pool;
    let repo = ApiKeyRepository::new(pool);
    
    let uuid = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return UnifiedApiResponse::error(400, "Invalid UUID", "The provided ID is not a valid UUID"),
    };

    match repo.get_by_id(uuid).await {
        Ok(Some(api_key)) => UnifiedApiResponse::success(api_key),
        Ok(None) => UnifiedApiResponse::not_found("API key"),
        Err(e) => {
            error!("Failed to get API key: {}", e);
            UnifiedApiResponse::server_error(&e.to_string())
        }
    }
}

/// POST /api/v1/admin/developer-portal/api-keys/:id/revoke
pub async fn revoke_api_key_handler(
    State(state): State<AppState>,
    Extension(admin_wallet): Extension<String>,
    Path(id): Path<String>,
    Json(body): Json<RevokeApiKeyBody>,
) -> impl IntoResponse {
    let pool = *state.db_pool;
    let repo = ApiKeyRepository::new(pool);
    
    let uuid = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return UnifiedApiResponse::error(400, "Invalid UUID", "The provided ID is not a valid UUID"),
    };

    let request = RevokeApiKeyRequest {
        reason: body.reason,
        revoked_by: admin_wallet.clone(),
    };

    match repo.revoke(uuid, request).await {
        Ok(api_key) => {
            info!("Admin {} revoked API key: {}", admin_wallet, uuid);
            UnifiedApiResponse::success(api_key)
        }
        Err(e) => {
            error!("Failed to revoke API key: {}", e);
            UnifiedApiResponse::server_error(&e.to_string())
        }
    }
}

/// PATCH /api/v1/admin/developer-portal/api-keys/:id/expiration
/// Update the expiration date of an API key
pub async fn update_expiration_handler(
    State(state): State<AppState>,
    Extension(admin_wallet): Extension<String>,
    Path(id): Path<String>,
    Json(body): Json<UpdateExpirationBody>,
) -> impl IntoResponse {
    let pool = *state.db_pool;
    let repo = ApiKeyRepository::new(pool);
    
    let uuid = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return UnifiedApiResponse::error(400, "Invalid UUID", "The provided ID is not a valid UUID"),
    };

    // Parse expires_at if provided
    let expires_at = if let Some(expires_str) = &body.expires_at {
        match chrono::DateTime::parse_from_rfc3339(expires_str) {
            Ok(dt) => Some(dt.with_timezone(&chrono::Utc)),
            Err(_) => return UnifiedApiResponse::error(400, "Invalid date format", "expires_at must be in ISO 8601 format"),
        }
    } else {
        None // Null means remove expiration
    };

    match repo.update_expiration(uuid, expires_at).await {
        Ok(api_key) => {
            info!("Admin {} updated API key {} expiration to {:?}", admin_wallet, uuid, expires_at);
            UnifiedApiResponse::success(api_key)
        }
        Err(e) => {
            error!("Failed to update API key expiration: {}", e);
            UnifiedApiResponse::server_error(&e.to_string())
        }
    }
}

/// GET /api/v1/admin/developer-portal/api-keys/expiring
/// List API keys expiring within the specified number of days
pub async fn list_expiring_keys_handler(
    State(state): State<AppState>,
    Query(query): Query<ListExpiringKeysQuery>,
) -> impl IntoResponse {
    let pool = *state.db_pool;
    let repo = ApiKeyRepository::new(pool);
    
    let days = query.days.unwrap_or(7); // Default to 7 days
    
    match repo.list_expiring_keys(days, query.limit, query.offset).await {
        Ok((api_keys, total)) => {
            UnifiedApiResponse::success(ExpiringKeysResponse {
                api_keys,
                total,
                days_ahead: days,
            })
        }
        Err(e) => {
            error!("Failed to list expiring API keys: {}", e);
            UnifiedApiResponse::server_error(&e.to_string())
        }
    }
}

// ============================================================================
// Module Handlers
// ============================================================================

/// GET /api/v1/admin/developer-portal/modules
pub async fn list_modules_handler(
    State(state): State<AppState>,
    Query(query): Query<ListModulesQuery>,
) -> impl IntoResponse {
    let pool = *state.db_pool;
    let repo = ModuleRepository::new(pool);
    
    match repo.list(query.status.as_deref(), query.category.as_deref()).await {
        Ok(response) => UnifiedApiResponse::success(response),
        Err(e) => {
            error!("Failed to list modules: {}", e);
            UnifiedApiResponse::server_error(&e.to_string())
        }
    }
}

/// GET /api/v1/admin/developer-portal/modules/:id
pub async fn get_module_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let pool = *state.db_pool;
    let repo = ModuleRepository::new(pool);
    
    let uuid = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return UnifiedApiResponse::error(400, "Invalid UUID", "The provided ID is not a valid UUID"),
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

/// POST /api/v1/admin/developer-portal/modules
pub async fn create_module_handler(
    State(state): State<AppState>,
    Json(body): Json<CreateModuleBody>,
) -> impl IntoResponse {
    let pool = *state.db_pool;
    let repo = ModuleRepository::new(pool);
    
    let request = CreateModuleRequest {
        name: body.name,
        display_name: body.display_name,
        description: body.description,
        category: body.category,
        base_path: body.base_path,
        default_rate_limit: body.default_rate_limit,
        access_levels: body.access_levels,
        endpoints: body.endpoints,
    };

    match repo.create(request).await {
        Ok(module) => {
            info!("Created module: {}", module.id);
            UnifiedApiResponse::success(module)
        }
        Err(e) => {
            error!("Failed to create module: {}", e);
            UnifiedApiResponse::server_error(&e.to_string())
        }
    }
}

/// PUT /api/v1/admin/developer-portal/modules/:id
pub async fn update_module_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateModuleBody>,
) -> impl IntoResponse {
    let pool = *state.db_pool;
    let repo = ModuleRepository::new(pool);
    
    let uuid = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return UnifiedApiResponse::error(400, "Invalid UUID", "The provided ID is not a valid UUID"),
    };

    let request = UpdateModuleRequest {
        display_name: body.display_name,
        description: body.description,
        status: body.status,
        default_rate_limit: body.default_rate_limit,
        access_levels: body.access_levels,
        endpoints: body.endpoints,
    };

    match repo.update(uuid, request).await {
        Ok(module) => {
            info!("Updated module: {}", uuid);
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

/// GET /api/v1/admin/developer-portal/stats
pub async fn get_stats_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let pool = *state.db_pool;
    let api_key_repo = ApiKeyRepository::new(pool);
    let module_repo = ModuleRepository::new(pool);

    // Get API key counts
    let (all_keys, total_keys) = match api_key_repo.list_all(Some(1000), None, None).await {
        Ok(result) => result,
        Err(e) => return UnifiedApiResponse::server_error(&e.to_string()),
    };
    
    let active_count = all_keys.iter().filter(|k| k.status == crate::domain::developer_portal::ApiKeyStatus::Active).count() as i64;
    let revoked_count = all_keys.iter().filter(|k| k.status == crate::domain::developer_portal::ApiKeyStatus::Revoked).count() as i64;
    let expired_count = all_keys.iter().filter(|k| k.status == crate::domain::developer_portal::ApiKeyStatus::Expired).count() as i64;

    // Get module counts
    let modules = match module_repo.list(None, None).await {
        Ok(result) => result,
        Err(e) => return UnifiedApiResponse::server_error(&e.to_string()),
    };
    
    let active_modules = modules.modules.iter().filter(|m| m.status == crate::domain::developer_portal::ModuleStatus::Active).count() as i64;

    let stats = DeveloperPortalStats {
        total_api_keys: total_keys,
        active_api_keys: active_count,
        revoked_api_keys: revoked_count,
        expired_api_keys: expired_count,
        total_modules: modules.total,
        active_modules,
        total_requests_today: 0, // TODO: Implement from usage logs
        total_requests_this_month: 0, // TODO: Implement from usage logs
        top_modules_by_usage: vec![], // TODO: Implement from usage logs
    };

    UnifiedApiResponse::success(stats)
}
