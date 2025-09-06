use crate::domain::shared_kernel::value_objects::UserId;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
// ============================================================================
// EMBEDDED TIMESTAMP PERMISSION HANDLERS
// ============================================================================
// Handlers for managing permissions with embedded timestamps in the format:
// "platform:resource:action:unix_timestamp"

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

use crate::web::auth::routes::AppState;
use crate::auth::permissions::{
    parse_permission_with_timestamp, is_permission_valid_with_time_check,
    add_timestamp_to_permission, filter_valid_permissions,
    get_expiring_permissions
};

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct EmbeddedPermissionRequest {
    pub embedded_permission: String,
    pub base_permission: String,
    pub platform: String,
    pub resource: String,
    pub action: String,
    pub expiry_timestamp: i64,
    pub reason: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
pub struct BulkEmbeddedPermissionRequest {
    pub user_ids: Vec<String>,
    pub permissions: Vec<EmbeddedPermissionData>,
    pub reason: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
pub struct EmbeddedPermissionData {
    pub base_permission: String,
    pub platform: String,
    pub resource: String,
    pub action: String,
    pub expiry_timestamp: i64,
    #[serde(skip)]
    pub embedded_permission: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExtendPermissionRequest {
    pub permission: String,
    pub new_expiry_timestamp: i64,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RevokePermissionRequest {
    pub permission: String,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ValidatePermissionsRequest {
    pub permissions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CleanupExpiredRequest {
    pub dry_run: Option<bool>,
    pub batch_size: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub action: Option<String>,
    pub platform: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EmbeddedPermissionResponse {
    pub permission: String,
    pub expires_at: i64,
}

#[derive(Debug, Serialize)]
pub struct BulkPermissionResponse {
    pub successful: Vec<UserPermissionResult>,
    pub failed: Vec<UserPermissionError>,
    pub summary: BulkSummary,
}

#[derive(Debug, Serialize)]
pub struct UserPermissionResult {
    pub user_id: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct UserPermissionError {
    pub user_id: String,
    pub error: String,
}

#[derive(Debug, Serialize)]
pub struct BulkSummary {
    pub total: i32,
    pub successful: i32,
    pub failed: i32,
}

#[derive(Debug, Serialize)]
pub struct ValidationResult {
    pub valid: Vec<String>,
    pub expired: Vec<ExpiredPermission>,
    pub expiring_soon: Vec<ExpiringSoonPermission>,
    pub summary: ValidationSummary,
}

#[derive(Debug, Serialize)]
pub struct ExpiredPermission {
    pub permission: String,
    pub base_permission: String,
    pub expired_at: i64,
    pub expired_for: i64, // milliseconds
}

#[derive(Debug, Serialize)]
pub struct ExpiringSoonPermission {
    pub permission: String,
    pub base_permission: String,
    pub expires_at: i64,
    pub expires_in: i64, // milliseconds
}

#[derive(Debug, Serialize)]
pub struct ValidationSummary {
    pub total: i32,
    pub valid_count: i32,
    pub expired_count: i32,
    pub expiring_soon_count: i32,
}

#[derive(Debug, Serialize)]
pub struct ExpiryStatusResponse {
    pub user_id: String,
    pub permissions: Vec<PermissionExpiryInfo>,
    pub health: ExpiryHealthInfo,
}

#[derive(Debug, Serialize)]
pub struct PermissionExpiryInfo {
    pub permission: String,
    pub base_permission: String,
    pub expires_at: Option<i64>,
    pub is_expired: bool,
    pub time_remaining: Option<i64>, // milliseconds
    pub expires_in: Option<String>, // human readable
}

#[derive(Debug, Serialize)]
pub struct ExpiryHealthInfo {
    pub has_expired: bool,
    pub has_expiring_soon: bool,
    pub next_expiry: Option<i64>,
    pub time_until_next_expiry: Option<i64>, // milliseconds
}

#[derive(Debug, Serialize)]
pub struct ExtendPermissionResponse {
    pub old_permission: String,
    pub new_permission: String,
    pub extension: i64, // milliseconds
}

#[derive(Debug, Serialize)]
pub struct ConvertPermissionResponse {
    pub new_permission: String,
    pub old_permission: String,
}

#[derive(Debug, Serialize)]
pub struct CleanupResponse {
    pub cleaned: i32,
    pub failed: i32,
    pub details: Vec<CleanupDetail>,
}

#[derive(Debug, Serialize)]
pub struct CleanupDetail {
    pub user_id: String,
    pub permission: String,
    pub expired_at: i64,
    pub status: String, // "cleaned" or "failed"
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HistoryResponse {
    pub history: Vec<HistoryEntry>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Serialize)]
pub struct HistoryEntry {
    pub id: String,
    pub user_id: String,
    pub action: String,
    pub permission: String,
    pub base_permission: String,
    pub platform: String,
    pub resource: String,
    pub action_type: String,
    pub expiry_timestamp: Option<i64>,
    pub previous_expiry: Option<i64>,
    pub new_expiry: Option<i64>,
    pub reason: Option<String>,
    pub granted_by: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize)]
pub struct PaginationInfo {
    pub total: i32,
    pub limit: i32,
    pub offset: i32,
    pub has_more: bool,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    pub error: String,
    pub message: String,
    pub details: Option<String>,
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn format_time_remaining(timestamp: i64) -> String {
    let now = chrono::Utc::now().timestamp();
    let diff = timestamp - now;
    
    if diff <= 0 {
        return "Expired".to_string();
    }
    
    let seconds = diff;
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let days = hours / 24;
    
    if days > 0 {
        format!("{} day{}", days, if days != 1 { "s" } else { "" })
    } else if hours > 0 {
        format!("{} hour{}", hours, if hours != 1 { "s" } else { "" })
    } else if minutes > 0 {
        format!("{} minute{}", minutes, if minutes != 1 { "s" } else { "" })
    } else {
        format!("{} second{}", seconds, if seconds != 1 { "s" } else { "" })
    }
}

fn create_permission_expiry_info(permission: &str) -> PermissionExpiryInfo {
    let (base_permission, timestamp) = parse_permission_with_timestamp(permission);
    let is_expired = !is_permission_valid_with_time_check(permission);
    
    let (time_remaining, expires_in) = if let Some(ts) = timestamp {
        let now = chrono::Utc::now().timestamp() * 1000; // Convert to milliseconds
        let expiry_ms = ts * 1000;
        let remaining = (expiry_ms - now).max(0);
        (Some(remaining), Some(format_time_remaining(ts)))
    } else {
        (None, None)
    };
    
    PermissionExpiryInfo {
        permission: permission.to_string(),
        base_permission,
        expires_at: timestamp,
        is_expired,
        time_remaining,
        expires_in,
    }
}

// ============================================================================
// HANDLERS
// ============================================================================

/// Grant an embedded timestamp permission to a user
/// POST /admin/users/:user_id/embedded-permissions
pub async fn grant_embedded_permission(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(request): Json<EmbeddedPermissionRequest>,
) -> Result<Json<EmbeddedPermissionResponse>, (StatusCode, Json<ApiErrorResponse>)> {
    tracing::info!("Granting embedded permission to user {}: {}", user_id, request.embedded_permission);
    
    // Get current user permissions - Firebase IDs are valid strings
    let user_id_typed = UserId::from_string_unchecked(user_id.clone());
    
    // Get user from repository first, then access their permissions
    let user = match state.user_repo.find_by_id(&user_id_typed).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ApiErrorResponse {
                    error: "user_not_found".to_string(),
                    message: "User not found".to_string(),
                    details: None,
                }),
            ));
        }
        Err(e) => {
            tracing::error!("Failed to get user: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiErrorResponse {
                    error: "database_error".to_string(),
                    message: "Failed to retrieve user permissions".to_string(),
                    details: Some(e.to_string()),
                }),
            ));
        }
    };
    
    // Get current permissions from User aggregate
    let mut new_permissions: Vec<String> = user.active_permissions();
    new_permissions.push(request.embedded_permission.clone());
    
    // Convert permission strings to Permission objects
    use crate::domain::user_management::value_objects::Permission;
    use std::collections::HashSet;
    
    let mut permission_objects = HashSet::new();
    for perm_str in new_permissions {
        match Permission::new(perm_str) {
            Ok(perm) => { permission_objects.insert(perm); },
            Err(e) => {
                tracing::warn!("Invalid permission format: {:?}", e);
                // Skip invalid permissions
            }
        }
    }
    
    // Update user permissions using DDD approach
    let mut updated_user = user;
    match updated_user.update_permissions(permission_objects, None) {
        Ok(()) => {
            // Save the updated user
            match state.user_repo.save(&updated_user).await {
                Ok(()) => {
                    tracing::info!("Successfully granted embedded permission to user: {}", user_id);
                    Ok(Json(EmbeddedPermissionResponse {
                        permission: request.embedded_permission,
                        expires_at: request.expiry_timestamp,
                    }))
                },
                Err(e) => {
                    tracing::error!("Failed to save updated user permissions: {:?}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiErrorResponse {
                            error: "save_failed".to_string(),
                            message: "Failed to save user permissions".to_string(),
                            details: Some(e.to_string()),
                        }),
                    ))
                }
            }
        },
        Err(e) => {
            tracing::error!("Failed to update user permissions: {:?}", e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ApiErrorResponse {
                    error: "permission_update_failed".to_string(),
                    message: "Failed to update user permissions".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// Grant embedded timestamp permissions to multiple users
/// POST /admin/users/bulk/embedded-permissions
pub async fn grant_bulk_embedded_permissions(
    State(state): State<AppState>,
    Json(request): Json<BulkEmbeddedPermissionRequest>,
) -> Result<Json<BulkPermissionResponse>, (StatusCode, Json<ApiErrorResponse>)> {
    tracing::info!("Granting bulk embedded permissions to {} users", request.user_ids.len());
    
    let mut successful = Vec::new();
    let mut failed = Vec::new();
    
    for user_id in &request.user_ids {
        // Parse user ID to proper type - Firebase IDs are valid strings
        let user_id_typed = UserId::from_string_unchecked(user_id.clone());
        
        // Get user from repository first
        let user = match state.user_repo.find_by_id(&user_id_typed).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                tracing::warn!("User not found: {}", user_id);
                failed.push(UserPermissionError {
                    user_id: user_id.clone(),
                    error: "User not found".to_string(),
                });
                continue;
            },
            Err(e) => {
                tracing::error!("Failed to get user {}: {:?}", user_id, e);
                failed.push(UserPermissionError {
                    user_id: user_id.clone(),
                    error: format!("Failed to get user: {}", e),
                });
                continue;
            }
        };
        
        // Get current permissions from User aggregate
        let current_permissions = user.active_permissions();
        
        // Create embedded permissions
        let embedded_permissions: Vec<String> = request.permissions.iter()
            .map(|p| add_timestamp_to_permission(&p.base_permission, 0) // Will need to use actual timestamp calculation
                .replace(":0", &format!(":{}", p.expiry_timestamp)))
            .collect();
        
        // Combine current permissions with new embedded permissions
        let mut new_permissions: Vec<String> = current_permissions;
        new_permissions.extend(embedded_permissions.clone());
        
        // Convert to Permission objects for DDD approach
        use crate::domain::user_management::value_objects::Permission;
        use std::collections::HashSet;
        
        let mut permission_objects = HashSet::new();
        for perm_str in new_permissions {
            match Permission::new(perm_str) {
                Ok(perm) => { permission_objects.insert(perm); },
                Err(e) => {
                    tracing::warn!("Invalid permission format: {:?}", e);
                    // Skip invalid permissions
                }
            }
        }
        
        // Update user permissions using DDD approach
        let mut updated_user = user;
        match updated_user.update_permissions(permission_objects, None) {
            Ok(()) => {
                // Save the updated user
                match state.user_repo.save(&updated_user).await {
                    Ok(()) => {
                        successful.push(UserPermissionResult {
                            user_id: user_id.clone(),
                            permissions: embedded_permissions,
                        });
                    },
                    Err(e) => {
                        failed.push(UserPermissionError {
                            user_id: user_id.clone(),
                            error: format!("Failed to save user: {}", e),
                        });
                    }
                }
            },
            Err(e) => {
                failed.push(UserPermissionError {
                    user_id: user_id.clone(),
                    error: format!("Failed to update permissions: {}", e),
                });
            }
        }
    }
    
    let summary = BulkSummary {
        total: request.user_ids.len() as i32,
        successful: successful.len() as i32,
        failed: failed.len() as i32,
    };
    
    tracing::info!("Bulk embedded permissions completed: {} successful, {} failed", summary.successful, summary.failed);
    
    Ok(Json(BulkPermissionResponse {
        successful,
        failed,
        summary,
    }))
}

/// Validate embedded permissions for a user
/// POST /admin/users/:user_id/embedded-permissions/validate
pub async fn validate_embedded_permissions(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(request): Json<ValidatePermissionsRequest>,
) -> Result<Json<ValidationResult>, (StatusCode, Json<ApiErrorResponse>)> {
    tracing::info!("Validating embedded permissions for user: {}", user_id);
    
    let user_id_typed = UserId::from_string_unchecked(user_id.clone());
    
    let permissions = if request.permissions.is_empty() {
        // Get user's current permissions using DDD approach
        match state.user_repo.find_by_id(&user_id_typed).await {
            Ok(Some(user)) => user.active_permissions(),
            Ok(None) => vec![], // User not found, return empty permissions
            Err(e) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiErrorResponse {
                        error: "database_error".to_string(),
                        message: "Failed to retrieve user permissions".to_string(),
                        details: Some(e.to_string()),
                    }),
                ));
            }
        }
    } else {
        request.permissions
    };
    
    let valid = filter_valid_permissions(&permissions);
    let expired: Vec<String> = Vec::new(); // TODO: Implement expired permission detection
    let expiring_soon = get_expiring_permissions(&permissions, 24); // Next 24 hours
    
    let summary = ValidationSummary {
        total: permissions.len() as i32,
        valid_count: valid.len() as i32,
        expired_count: expired.len() as i32,
        expiring_soon_count: expiring_soon.len() as i32,
    };
    
    Ok(Json(ValidationResult {
        valid,
        expired: vec![], // Will be populated by TODO above
        expiring_soon: vec![], // Will be populated by converting expiring_soon
        summary,
    }))
}

/// Get permission expiry status for a user
/// GET /admin/users/:user_id/permissions/expiry-status
pub async fn get_permission_expiry_status(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<ExpiryStatusResponse>, (StatusCode, Json<ApiErrorResponse>)> {
    tracing::info!("Getting permission expiry status for user: {}", user_id);
    
    let user_id_typed = UserId::from_string_unchecked(user_id.clone());
    
    let permissions = match state.user_repo.find_by_id(&user_id_typed).await {
        Ok(Some(user)) => user.active_permissions(),
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ApiErrorResponse {
                    error: "user_not_found".to_string(),
                    message: "User not found".to_string(),
                    details: None,
                }),
            ));
        },
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiErrorResponse {
                    error: "database_error".to_string(),
                    message: "Failed to retrieve user".to_string(),
                    details: Some(e.to_string()),
                }),
            ));
        }
    };
    
    let permission_infos: Vec<PermissionExpiryInfo> = permissions.iter()
        .map(|p| create_permission_expiry_info(p))
        .collect();
    
    let has_expired = permission_infos.iter().any(|p| p.is_expired);
    let has_expiring_soon = permission_infos.iter().any(|p| {
        if let Some(remaining) = p.time_remaining {
            remaining <= 24 * 60 * 60 * 1000 // 24 hours in milliseconds
        } else {
            false
        }
    });
    
    let next_expiry = permission_infos.iter()
        .filter_map(|p| p.expires_at)
        .filter(|&ts| {
            let now = chrono::Utc::now().timestamp();
            ts > now
        })
        .min();
    
    let time_until_next_expiry = next_expiry.map(|ts| {
        let now = chrono::Utc::now().timestamp() * 1000;
        ((ts * 1000) - now).max(0)
    });
    
    let health = ExpiryHealthInfo {
        has_expired,
        has_expiring_soon,
        next_expiry,
        time_until_next_expiry,
    };
    
    Ok(Json(ExpiryStatusResponse {
        user_id,
        permissions: permission_infos,
        health,
    }))
}

/// Revoke an embedded timestamp permission
/// POST /admin/users/:user_id/embedded-permissions/revoke
pub async fn revoke_embedded_permission(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(request): Json<RevokePermissionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ApiErrorResponse>)> {
    tracing::info!("Revoking embedded permission for user {}: {}", user_id, request.permission);
    
    // Get current user permissions - Firebase IDs are valid strings
    let user_id_typed = UserId::from_string_unchecked(user_id.clone());
    
    let user: crate::domain::user_management::aggregates::User = match state.user_repo.find_by_id(&user_id_typed).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ApiErrorResponse {
                    error: "user_not_found".to_string(),
                    message: "User not found".to_string(),
                    details: None,
                }),
            ));
        },
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiErrorResponse {
                    error: "database_error".to_string(),
                    message: "Failed to retrieve user".to_string(),
                    details: Some(e.to_string()),
                }),
            ));
        }
    };
    
    let current_permissions = user.active_permissions();
    if current_permissions.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiErrorResponse {
                error: "user_not_found".to_string(),
                message: "User has no permissions".to_string(),
                details: Some(user_id),
            }),
        ));
    }
    
    // Remove the specific permission  
    let new_permissions: Vec<String> = current_permissions.into_iter()
        .filter(|p| p != &request.permission)
        .collect();
    
    // Convert permission strings to Permission objects and update user using DDD approach
    use crate::domain::user_management::value_objects::Permission;
    use std::collections::HashSet;
    
    let mut permission_objects = HashSet::new();
    for perm_str in new_permissions {
        match Permission::new(perm_str) {
            Ok(perm) => { permission_objects.insert(perm); },
            Err(e) => {
                tracing::warn!("Invalid permission format: {:?}", e);
                // Skip invalid permissions
            }
        }
    }
    
    // Update user permissions using DDD approach
    let mut updated_user = user;
    match updated_user.update_permissions(permission_objects, None) {
        Ok(()) => {
            // Save the updated user
            match state.user_repo.save(&updated_user).await {
                Ok(()) => {
                    tracing::info!("Successfully revoked embedded permission from user: {}", user_id);
                    Ok((StatusCode::OK, Json(serde_json::json!({"message": "Permission revoked successfully"}))))
                },
                Err(e) => {
                    tracing::error!("Failed to save user after permission revocation: {:?}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiErrorResponse {
                            error: "save_failed".to_string(),
                            message: "Failed to save user permissions".to_string(),
                            details: Some(e.to_string()),
                        }),
                    ))
                }
            }
        },
        Err(e) => {
            tracing::error!("Failed to update user permissions: {:?}", e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ApiErrorResponse {
                    error: "permission_update_failed".to_string(),
                    message: "Failed to update user permissions".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// Extend an embedded timestamp permission's expiry
/// POST /admin/users/:user_id/embedded-permissions/extend
pub async fn extend_embedded_permission(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(request): Json<ExtendPermissionRequest>,
) -> Result<Json<ExtendPermissionResponse>, (StatusCode, Json<ApiErrorResponse>)> {
    tracing::info!("Extending embedded permission for user {}: {}", user_id, request.permission);
    
    let (base_permission, old_timestamp) = parse_permission_with_timestamp(&request.permission);
    let new_permission = add_timestamp_to_permission(&base_permission, 0) // Placeholder for calculation
        .replace(":0", &format!(":{}", request.new_expiry_timestamp));
    
    // Get current user permissions - Firebase IDs are valid strings
    let user_id_typed = UserId::from_string_unchecked(user_id.clone());
    
    let user: crate::domain::user_management::aggregates::User = match state.user_repo.find_by_id(&user_id_typed).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ApiErrorResponse {
                    error: "user_not_found".to_string(),
                    message: "User not found".to_string(),
                    details: None,
                }),
            ));
        },
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiErrorResponse {
                    error: "database_error".to_string(),
                    message: "Failed to retrieve user".to_string(),
                    details: Some(e.to_string()),
                }),
            ));
        }
    };
    
    let current_permissions = user.active_permissions();
    if current_permissions.is_empty() {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ApiErrorResponse {
                    error: "user_not_found".to_string(),
                    message: "User has no permissions".to_string(),
                    details: Some(user_id),
                }),
            ));
    }
    
    // Replace old permission with new one  
    let new_permissions: Vec<String> = current_permissions.into_iter()
        .map(|p| if p == request.permission { new_permission.clone() } else { p })
        .collect();
    
    // Convert permission strings to Permission objects and update user using DDD approach
    use crate::domain::user_management::value_objects::Permission;
    use std::collections::HashSet;
    
    let mut permission_objects = HashSet::new();
    for perm_str in new_permissions {
        match Permission::new(perm_str) {
            Ok(perm) => { permission_objects.insert(perm); },
            Err(e) => {
                tracing::warn!("Invalid permission format: {:?}", e);
                // Skip invalid permissions
            }
        }
    }
    
    // Update user permissions using DDD approach
    let mut updated_user = user;
    let update_result = updated_user.update_permissions(permission_objects, None);
    
    match update_result {
        Ok(()) => {
            // Save the updated user
            match state.user_repo.save(&updated_user).await {
                Ok(()) => {
                    let extension = if let Some(old_ts) = old_timestamp {
                        (request.new_expiry_timestamp - old_ts) * 1000 // Convert to milliseconds
                    } else {
                        0
                    };
                    
                    tracing::info!("Successfully extended embedded permission for user: {}", user_id);
                    Ok(Json(ExtendPermissionResponse {
                        old_permission: request.permission,
                        new_permission,
                        extension,
                    }))
                },
                Err(e) => {
                    tracing::error!("Failed to save user after permission extension: {:?}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiErrorResponse {
                            error: "save_failed".to_string(),
                            message: "Failed to save user permissions".to_string(),
                            details: Some(e.to_string()),
                        }),
                    ))
                }
            }
        },
        Err(e) => {
            tracing::error!("Failed to update user permissions: {:?}", e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ApiErrorResponse {
                    error: "permission_update_failed".to_string(),
                    message: "Failed to update user permissions".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// Cleanup expired embedded permissions across all users
/// POST /admin/embedded-permissions/cleanup-expired
pub async fn cleanup_expired_permissions(
    State(_state): State<AppState>,
    Json(request): Json<CleanupExpiredRequest>,
) -> Result<Json<CleanupResponse>, (StatusCode, Json<ApiErrorResponse>)> {
    let dry_run = request.dry_run.unwrap_or(false);
    tracing::info!("Cleaning up expired embedded permissions (dry_run: {})", dry_run);
    
    // TODO: Implement cleanup logic
    // This would need to:
    // 1. Query all users with permissions
    // 2. Filter expired permissions for each user  
    // 3. Update user permissions to remove expired ones
    // 4. Track success/failure for each operation
    
    Ok(Json(CleanupResponse {
        cleaned: 0,
        failed: 0,
        details: vec![],
    }))
}