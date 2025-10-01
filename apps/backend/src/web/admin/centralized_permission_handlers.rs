// Centralized Permission Admin Handlers (v2.0)
// Demonstrates integration with CentralizedPermissionAuthority and DatabasePermissionRegistry
// Shows best practices for using the new validation system

use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::Json,
};
use axum::http::HeaderMap;
use chrono::Timelike;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info, warn, debug};
use uuid::Uuid;

use crate::auth::{
    PermissionState, HandlerPermissionExt, RequirePermission, PermissionValidator,
    RoutePermissionResolver, ValidationContext, PermissionResult,
    Permission,
};
use crate::web::admin::responses::{AdminApiResponse, AdminMetadata};

// ============================================================================
// REQUEST/RESPONSE TYPES FOR CENTRALIZED PERMISSION SYSTEM
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct PermissionValidationRequest {
    pub wallet_address: String,
    pub permissions: Vec<String>,
    pub context: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct RoutePermissionRequest {
    pub route_pattern: String,
    pub http_method: String,
    pub required_permission: String,
    pub priority: Option<i32>,
    pub is_public: Option<bool>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PermissionValidationResponse {
    pub wallet_address: String,
    pub results: HashMap<String, PermissionResult>,
    pub total_permissions: usize,
    pub granted_count: usize,
    pub denied_count: usize,
    pub validation_time_ms: u64,
    pub cache_hit_ratio: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct RoutePermissionResponse {
    pub id: String,
    pub route_pattern: String,
    pub http_method: String,
    pub required_permission: String,
    pub priority: i32,
    pub is_active: bool,
    pub is_public: bool,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct PermissionCacheStats {
    pub permission_hits: u64,
    pub permission_misses: u64,
    pub route_hits: u64,
    pub route_misses: u64,
    pub cache_invalidations: u64,
    pub hit_ratio: f64,
}

#[derive(Debug, Serialize)]
pub struct SystemHealthResponse {
    pub system_version: String,
    pub permission_authority_status: String,
    pub permission_registry_status: String,
    pub cache_status: String,
    pub total_route_mappings: usize,
    pub cache_stats: PermissionCacheStats,
}

// ============================================================================
// PERMISSION VALIDATION HANDLERS
// ============================================================================

/// Validate permissions for a wallet using centralized authority
pub async fn validate_wallet_permissions(
    State(permission_state): State<PermissionState>,
    headers: HeaderMap,
    Json(request): Json<PermissionValidationRequest>,
) -> Result<Json<AdminApiResponse<PermissionValidationResponse>>, StatusCode> {
    info!(
        "🔐 Admin: Validating {} permissions for wallet: {}",
        request.permissions.len(),
        request.wallet_address
    );

    // Extract requesting admin wallet from headers for audit
    let admin_wallet = extract_admin_wallet_address(&headers);
    
    // Validate admin has permission to perform this action
    if let Some(admin_addr) = &admin_wallet {
        match permission_state.require_permission("admin:permissions:validate", admin_addr).await {
            Ok(_) => {
                debug!("Admin {} authorized for permission validation", admin_addr);
            }
            Err(_response) => {
                warn!("Admin {} not authorized for permission validation", admin_addr);
                return Err(StatusCode::FORBIDDEN);
            }
        }
    }

    // Create validation context
    let context = ValidationContext {
        request_id: Uuid::new_v4().to_string(),
        user_agent: headers.get("user-agent")
            .and_then(|h| h.to_str().ok())
            .map(String::from),
        ip_address: headers.get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .map(String::from),
        timestamp: chrono::Utc::now(),
        route_path: "/admin/permissions/validate".to_string(),
        http_method: "POST".to_string(),
    };

    // Perform bulk validation using centralized authority
    match permission_state.authority.bulk_validate_permissions(
        &request.wallet_address,
        &request.permissions,
        &context,
    ).await {
        Ok(bulk_result) => {
            let cache_stats = permission_state.authority.get_cache_stats().await;
            let hit_ratio = if cache_stats.permission_hits + cache_stats.permission_misses > 0 {
                cache_stats.permission_hits as f64 / 
                (cache_stats.permission_hits + cache_stats.permission_misses) as f64
            } else {
                0.0
            };

            info!(
                "✅ Permission validation completed: {}/{} granted for wallet: {} ({}ms)",
                bulk_result.granted_count,
                bulk_result.total_permissions,
                request.wallet_address,
                bulk_result.validation_time_ms
            );

            let response = PermissionValidationResponse {
                wallet_address: bulk_result.wallet_address,
                results: bulk_result.results,
                total_permissions: bulk_result.total_permissions,
                granted_count: bulk_result.granted_count,
                denied_count: bulk_result.denied_count,
                validation_time_ms: bulk_result.validation_time_ms,
                cache_hit_ratio: Some(hit_ratio),
            };

            Ok(Json(AdminApiResponse::success_with_meta(
                response,
                "Permission validation completed",
                AdminMetadata {
                    operation: "validate_permissions".to_string(),
                    performed_by: admin_wallet,
                    pagination: None,
                    permissions: None,
                    metadata: Some(serde_json::json!({
                        "request_id": context.request_id,
                        "validation_time_ms": bulk_result.validation_time_ms
                    })),
                },
            )))
        }
        Err(e) => {
            error!(
                "❌ Permission validation failed for wallet {}: {}",
                request.wallet_address, e
            );

            Ok(Json(AdminApiResponse::error(
                &format!("Permission validation failed: {}", e),
                "Permission validation failed"
            )))
        }
    }
}

/// Get all permissions for a wallet using centralized authority
pub async fn get_wallet_permissions(
    State(permission_state): State<PermissionState>,
    headers: HeaderMap,
    Path(wallet_address): Path<String>,
) -> Result<Json<AdminApiResponse<Vec<Permission>>>, StatusCode> {
    info!("📋 Admin: Fetching permissions for wallet: {}", wallet_address);

    // Extract requesting admin wallet from headers
    let admin_wallet = extract_admin_wallet_address(&headers);
    
    // Validate admin permissions
    if let Some(admin_addr) = &admin_wallet {
        if let Err(_) = permission_state.require_permission("admin:permissions:read", admin_addr).await {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    match permission_state.authority.get_wallet_permissions(&wallet_address).await {
        Ok(permissions) => {
            info!(
                "✅ Retrieved {} permissions for wallet: {}",
                permissions.len(),
                wallet_address
            );

            Ok(Json(AdminApiResponse::success_with_meta(
                permissions,
                "Permissions retrieved successfully",
                AdminMetadata {
                    operation: "get_wallet_permissions".to_string(),
                    performed_by: admin_wallet,
                    pagination: None,
                    permissions: None,
                    metadata: Some(serde_json::json!({
                        "request_id": Uuid::new_v4().to_string()
                    })),
                },
            )))
        }
        Err(e) => {
            error!("❌ Failed to get permissions for wallet {}: {}", wallet_address, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// ROUTE PERMISSION MANAGEMENT HANDLERS
// ============================================================================

/// Register a new route permission mapping
pub async fn register_route_permission(
    State(permission_state): State<PermissionState>,
    headers: HeaderMap,
    Json(request): Json<RoutePermissionRequest>,
) -> Result<Json<AdminApiResponse<String>>, StatusCode> {
    info!(
        "🛣️ Admin: Registering route permission: {} {} -> {}",
        request.http_method, request.route_pattern, request.required_permission
    );

    // Extract admin wallet
    let admin_wallet = extract_admin_wallet_address(&headers);
    
    // Validate admin permissions
    if let Some(admin_addr) = &admin_wallet {
        if let Err(_) = permission_state.require_permission("admin:routes:manage", admin_addr).await {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    match permission_state.registry.register_route_permission(
        &request.route_pattern,
        &request.http_method,
        &request.required_permission,
    ).await {
        Ok(()) => {
            info!(
                "✅ Route permission registered: {} {} -> {}",
                request.http_method, request.route_pattern, request.required_permission
            );

            Ok(Json(AdminApiResponse::success_with_meta(
                "Route permission registered successfully".to_string(),
                "Route permission registered successfully",
                AdminMetadata {
                    operation: "register_route_permission".to_string(),
                    performed_by: admin_wallet,
                    pagination: None,
                    permissions: None,
                    metadata: Some(serde_json::json!({
                        "request_id": Uuid::new_v4().to_string()
                    })),
                },
            )))
        }
        Err(e) => {
            error!("❌ Failed to register route permission: {}", e);

            Ok(Json(AdminApiResponse::error(
                &format!("Failed to register route permission: {}", e),
                "Failed to register route permission"
            )))
        }
    }
}

/// Get all route permission mappings
pub async fn get_route_permissions(
    State(permission_state): State<PermissionState>,
    headers: HeaderMap,
) -> Result<Json<AdminApiResponse<Vec<crate::auth::RoutePermissionMapping>>>, StatusCode> {
    info!("📋 Admin: Fetching all route permission mappings");

    // Extract admin wallet
    let admin_wallet = extract_admin_wallet_address(&headers);
    
    // Validate admin permissions
    if let Some(admin_addr) = &admin_wallet {
        if let Err(_) = permission_state.require_permission("admin:routes:read", admin_addr).await {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    match permission_state.registry.get_all_mappings().await {
        Ok(mappings) => {
            info!("✅ Retrieved {} route permission mappings", mappings.len());

            Ok(Json(AdminApiResponse::success_with_meta(
                mappings,
                "Route permissions retrieved successfully",
                AdminMetadata {
                    operation: "get_route_permissions".to_string(),
                    performed_by: admin_wallet,
                    pagination: None,
                    permissions: None,
                    metadata: Some(serde_json::json!({
                        "request_id": Uuid::new_v4().to_string()
                    })),
                },
            )))
        }
        Err(e) => {
            error!("❌ Failed to get route permissions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// SYSTEM MONITORING HANDLERS
// ============================================================================

/// Get centralized permission system health and statistics
pub async fn get_system_health(
    State(permission_state): State<PermissionState>,
    headers: HeaderMap,
) -> Result<Json<AdminApiResponse<SystemHealthResponse>>, StatusCode> {
    info!("📊 Admin: Fetching centralized permission system health");

    // Extract admin wallet
    let admin_wallet = extract_admin_wallet_address(&headers);
    
    // Validate admin permissions
    if let Some(admin_addr) = &admin_wallet {
        if let Err(_) = permission_state.require_permission("admin:system:monitor", admin_addr).await {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    // Get cache statistics
    let cache_stats = permission_state.authority.get_cache_stats().await;
    let registry_stats = permission_state.registry.get_cache_stats().await;
    
    // Get route mappings count
    let route_mappings_count = match permission_state.registry.get_all_mappings().await {
        Ok(mappings) => mappings.len(),
        Err(_) => 0,
    };

    let hit_ratio = if cache_stats.permission_hits + cache_stats.permission_misses > 0 {
        cache_stats.permission_hits as f64 / 
        (cache_stats.permission_hits + cache_stats.permission_misses) as f64
    } else {
        0.0
    };

    let system_health = SystemHealthResponse {
        system_version: "centralized_authority_v2".to_string(),
        permission_authority_status: "operational".to_string(),
        permission_registry_status: "operational".to_string(),
        cache_status: if hit_ratio > 0.7 { "optimal" } else if hit_ratio > 0.3 { "good" } else { "needs_warmup" }.to_string(),
        total_route_mappings: route_mappings_count,
        cache_stats: PermissionCacheStats {
            permission_hits: cache_stats.permission_hits,
            permission_misses: cache_stats.permission_misses,
            route_hits: registry_stats.hits,
            route_misses: registry_stats.misses,
            cache_invalidations: cache_stats.cache_invalidations + registry_stats.cache_invalidations,
            hit_ratio,
        },
    };

    info!("✅ System health retrieved with {:.2}% cache hit ratio", hit_ratio * 100.0);

    Ok(Json(AdminApiResponse::success_with_meta(
        system_health,
        "System health retrieved successfully",
        AdminMetadata {
            operation: "get_system_health".to_string(),
            performed_by: admin_wallet,
            pagination: None,
            permissions: None,
            metadata: Some(serde_json::json!({
                "request_id": Uuid::new_v4().to_string()
            })),
        },
    )))
}

/// Clear permission caches for performance optimization
pub async fn clear_permission_caches(
    State(permission_state): State<PermissionState>,
    headers: HeaderMap,
) -> Result<Json<AdminApiResponse<String>>, StatusCode> {
    info!("🗑️ Admin: Clearing permission caches");

    // Extract admin wallet
    let admin_wallet = extract_admin_wallet_address(&headers);
    
    // Validate admin permissions (requires high-level admin access)
    if let Some(admin_addr) = &admin_wallet {
        if let Err(_) = permission_state.require_permission("admin:system:manage", admin_addr).await {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    // Clear all caches
    permission_state.authority.clear_caches().await;
    
    info!("✅ Permission caches cleared successfully");

    Ok(Json(AdminApiResponse::success_with_meta(
        "Permission caches cleared successfully".to_string(),
        "Permission caches cleared successfully",
        AdminMetadata {
            operation: "clear_permission_caches".to_string(),
            performed_by: admin_wallet,
            pagination: None,
            permissions: None,
            metadata: Some(serde_json::json!({
                "request_id": Uuid::new_v4().to_string()
            })),
        },
    )))
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Extract admin wallet address from request headers
fn extract_admin_wallet_address(headers: &HeaderMap) -> Option<String> {
    headers.get("x-wallet-address")
        .or_else(|| headers.get("x-admin-wallet"))
        .and_then(|h| h.to_str().ok())
        .map(|addr| addr.to_lowercase())
        .filter(|addr| addr.len() == 42 && addr.starts_with("0x"))
}

// ============================================================================
// EXAMPLE IMPLEMENTATION OF RequirePermission TRAIT
// ============================================================================

/// Example handler that implements the RequirePermission trait
pub struct AdminUserManagementHandler;

#[async_trait::async_trait]
impl RequirePermission for AdminUserManagementHandler {
    fn required_permission() -> &'static str {
        "admin:users:manage"
    }
    
    async fn custom_validation(
        &self,
        wallet_address: &str,
        context: &ValidationContext,
    ) -> Result<bool, crate::core::errors::AppError> {
        // Custom validation logic - e.g., time-based restrictions
        let hour = context.timestamp.hour();
        if hour < 6 || hour > 22 {
            warn!("Admin access attempted outside business hours by {}", wallet_address);
            return Ok(false);
        }
        
        Ok(true)
    }
}