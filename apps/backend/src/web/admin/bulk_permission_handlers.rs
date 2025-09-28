use crate::domain::shared_kernel::value_objects::UserId;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
// ============================================================================
// BULK PERMISSION MANAGEMENT HANDLERS
// ============================================================================
// Comprehensive bulk operations for permission management in admin interface

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

use crate::web::auth::routes::AppState;

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct BulkGrantPermissionsRequest {
    pub user_ids: Vec<String>,
    pub permissions: Vec<String>,
    pub reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub notify_users: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct BulkRevokePermissionsRequest {
    pub user_ids: Vec<String>, 
    pub permissions: Vec<String>,
    pub reason: Option<String>,
    pub notify_users: Option<bool>,
}

// BulkRoleAssignmentRequest removed - using permissions-based system only

#[derive(Debug, Deserialize)]
pub struct BulkPermissionTemplateRequest {
    pub user_ids: Vec<String>,
    pub template_id: String,
    pub merge_permissions: Option<bool>,
    pub expires_at: Option<DateTime<Utc>>,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BulkPermissionValidationRequest {
    pub user_ids: Vec<String>,
    pub check_expired: Option<bool>,
    pub check_conflicting: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct BulkOperationResponse {
    pub successful: Vec<BulkUserResult>,
    pub failed: Vec<BulkUserError>,
    pub summary: BulkSummary,
    pub operation: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct BulkUserResult {
    pub wallet_address: String,
    pub email: Option<String>,
    pub permissions_added: Vec<String>,
    pub permissions_removed: Vec<String>,
    pub previous_permissions: Vec<String>,
    pub current_permissions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct BulkUserError {
    pub wallet_address: String,
    pub error: String,
    pub error_code: String,
}

#[derive(Debug, Serialize)]
pub struct BulkSummary {
    pub total_users: i32,
    pub successful_operations: i32,
    pub failed_operations: i32,
    pub permissions_granted: i32,
    pub permissions_revoked: i32,
}

#[derive(Debug, Serialize)]
pub struct BulkValidationResponse {
    pub user_validations: Vec<UserPermissionValidation>,
    pub summary: ValidationSummary,
}

#[derive(Debug, Serialize)]
pub struct UserPermissionValidation {
    pub wallet_address: String,
    pub email: Option<String>,
    pub valid_permissions: Vec<String>,
    pub expired_permissions: Vec<ExpiredPermissionInfo>,
    pub conflicting_permissions: Vec<ConflictingPermissionInfo>,
    pub health_score: f32, // 0-100 score
}

#[derive(Debug, Serialize)]
pub struct ExpiredPermissionInfo {
    pub permission: String,
    pub expired_at: DateTime<Utc>,
    pub expired_since: i64, // milliseconds
}

#[derive(Debug, Serialize)]
pub struct ConflictingPermissionInfo {
    pub permission: String,
    pub conflicts_with: String,
    pub conflict_type: String,
}

#[derive(Debug, Serialize)]
pub struct ValidationSummary {
    pub total_users_checked: i32,
    pub users_with_issues: i32,
    pub total_expired_permissions: i32,
    pub total_conflicting_permissions: i32,
    pub average_health_score: f32,
}

// ============================================================================
// PERMISSION TEMPLATES
// ============================================================================

fn get_permission_templates() -> HashMap<String, Vec<String>> {
    let mut templates = HashMap::new();
    
    templates.insert("basic_user".to_string(), vec![
        "epsx:analytics:view".to_string(),
        "epsx:profile:manage".to_string(),
    ]);
    
    templates.insert("premium_user".to_string(), vec![
        "epsx:analytics:view".to_string(),
        "epsx:analytics:export:basic".to_string(),
        "epsx:profile:manage".to_string(),
        "epsx:realtime:access".to_string(),
    ]);
    
    templates.insert("admin_user".to_string(), vec![
        "admin:*:*".to_string(),
    ]);
    
    templates.insert("moderator".to_string(), vec![
        "admin:users:view".to_string(),
        "admin:users:edit".to_string(),
        "epsx:analytics:view".to_string(),
        "epsx:moderation:access".to_string(),
    ]);
    
    templates.insert("analytics_power_user".to_string(), vec![
        "epsx:analytics:view".to_string(),
        "epsx:analytics:export:advanced".to_string(),
        "epsx:analytics:rankings:manage".to_string(),
        "epsx:realtime:access".to_string(),
        "epsx:api:access".to_string(),
    ]);
    
    templates
}

fn get_role_permissions() -> HashMap<String, Vec<String>> {
    let mut roles = HashMap::new();
    
    roles.insert("admin".to_string(), vec![
        "admin:*:*".to_string(),
    ]);
    
    roles.insert("user".to_string(), vec![
        "epsx:analytics:view".to_string(),
        "epsx:profile:manage".to_string(),
    ]);
    
    roles.insert("guest".to_string(), vec![
        "epsx:analytics:view".to_string(),
    ]);
    
    roles.insert("premium".to_string(), vec![
        "epsx:analytics:view".to_string(),
        "epsx:analytics:export:basic".to_string(),
        "epsx:profile:manage".to_string(),
        "epsx:realtime:access".to_string(),
    ]);
    
    roles
}

// ============================================================================
// HANDLERS
// ============================================================================

/// Bulk grant permissions to multiple users
/// POST /admin/users/bulk/permissions/grant
pub async fn bulk_grant_permissions(
    State(state): State<AppState>,
    Json(request): Json<BulkGrantPermissionsRequest>,
) -> Result<Json<BulkOperationResponse>, (StatusCode, String)> {
    tracing::info!("Bulk granting permissions to {} users", request.user_ids.len());
    
    if request.user_ids.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No user IDs provided".to_string()));
    }
    
    if request.permissions.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No permissions provided".to_string()));
    }
    
    let mut successful = Vec::new();
    let mut failed = Vec::new();
    let mut total_granted = 0;
    
    for user_id in &request.user_ids {
        let user_id_typed = match UserId::from_str(user_id) {
            Ok(id) => id,
            Err(_) => {
                failed.push(BulkUserError {
                    wallet_address: user_id.clone(),
                    error: "Invalid user ID format".to_string(),
                    error_code: "INVALID_USER_ID".to_string(),
                });
                continue;
            }
        };
        
        // Get user from repository first, then access their permissions
        let user = match state.user_repo.find_by_id(&user_id_typed).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                tracing::warn!("User not found: {}", user_id);
                failed.push(BulkUserError {
                    wallet_address: user_id.clone(),
                    error: "User not found".to_string(),
                    error_code: "USER_NOT_FOUND".to_string(),
                });
                continue;
            },
            Err(e) => {
                tracing::error!("Failed to get user {}: {:?}", user_id, e);
                failed.push(BulkUserError {
                    wallet_address: user_id.clone(),
                    error: format!("Failed to get user: {}", e),
                    error_code: "DATABASE_ERROR".to_string(),
                });
                continue;
            }
        };
        
        // Get current permissions from User aggregate
        let current_permissions = user.active_permissions();
        
        let previous_permissions = current_permissions.clone();
        
        // Add new permissions (avoid duplicates)
        let mut new_permissions = current_permissions;
        let mut added_permissions = Vec::new();
        
        for permission in &request.permissions {
            if !new_permissions.contains(permission) {
                new_permissions.push(permission.clone());
                added_permissions.push(permission.clone());
            }
        }
        
        // Convert permission strings to Permission objects and update user using DDD approach
        use crate::domain::user_management::value_objects::Permission;
        use std::collections::HashSet;
        
        let mut permission_objects = HashSet::new();
        for perm_str in &new_permissions {
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
                        total_granted += added_permissions.len();
                        
                        // Get user email for response  
                        let email = Some(updated_user.email().to_string());
                
                successful.push(BulkUserResult {
                    wallet_address: user_id.clone(),
                    email,
                    permissions_added: added_permissions,
                    permissions_removed: vec![],
                    previous_permissions,
                    current_permissions: new_permissions,
                });
                    },
                    Err(e) => {
                        failed.push(BulkUserError {
                            wallet_address: user_id.clone(),
                            error: format!("Failed to save user: {}", e),
                            error_code: "SAVE_FAILED".to_string(),
                        });
                    }
                }
            },
            Err(e) => {
                failed.push(BulkUserError {
                    wallet_address: user_id.clone(),
                    error: format!("Failed to update permissions: {}", e),
                    error_code: "UPDATE_FAILED".to_string(),
                });
            }
        }
    }
    
    let summary = BulkSummary {
        total_users: request.user_ids.len() as i32,
        successful_operations: successful.len() as i32,
        failed_operations: failed.len() as i32,
        permissions_granted: total_granted as i32,
        permissions_revoked: 0,
    };
    
    tracing::info!("Bulk grant completed: {} successful, {} failed, {} permissions granted", 
                   summary.successful_operations, summary.failed_operations, summary.permissions_granted);
    
    Ok(Json(BulkOperationResponse {
        successful,
        failed,
        summary,
        operation: "bulk_grant_permissions".to_string(),
        timestamp: Utc::now(),
    }))
}

/// Bulk revoke permissions from multiple users
/// POST /admin/users/bulk/permissions/revoke
pub async fn bulk_revoke_permissions(
    State(state): State<AppState>,
    Json(request): Json<BulkRevokePermissionsRequest>,
) -> Result<Json<BulkOperationResponse>, (StatusCode, String)> {
    tracing::info!("Bulk revoking permissions from {} users", request.user_ids.len());
    
    if request.user_ids.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No user IDs provided".to_string()));
    }
    
    if request.permissions.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No permissions provided".to_string()));
    }
    
    let mut successful = Vec::new();
    let mut failed = Vec::new();
    let mut total_revoked = 0;
    
    for user_id in &request.user_ids {
        let user_id_typed = match UserId::from_str(user_id) {
            Ok(id) => id,
            Err(_) => {
                failed.push(BulkUserError {
                    wallet_address: user_id.clone(),
                    error: "Invalid user ID format".to_string(),
                    error_code: "INVALID_USER_ID".to_string(),
                });
                continue;
            }
        };
        
        // Get user from repository first, then access their permissions
        let user = match state.user_repo.find_by_id(&user_id_typed).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                tracing::warn!("User not found: {}", user_id);
                failed.push(BulkUserError {
                    wallet_address: user_id.clone(),
                    error: "User not found".to_string(),
                    error_code: "USER_NOT_FOUND".to_string(),
                });
                continue;
            },
            Err(e) => {
                tracing::error!("Failed to get user {}: {:?}", user_id, e);
                failed.push(BulkUserError {
                    wallet_address: user_id.clone(),
                    error: format!("Failed to get user: {}", e),
                    error_code: "DATABASE_ERROR".to_string(),
                });
                continue;
            }
        };
        
        // Get current permissions from User aggregate
        let current_permissions = user.active_permissions();
        
        let previous_permissions = current_permissions.clone();
        
        // Remove specified permissions
        let mut removed_permissions = Vec::new();
        let new_permissions: Vec<String> = current_permissions.into_iter()
            .filter(|perm| {
                if request.permissions.contains(perm) {
                    removed_permissions.push(perm.clone());
                    false
                } else {
                    true
                }
            })
            .collect();
        
        // Convert permission strings to Permission objects and update user using DDD approach
        use crate::domain::user_management::value_objects::Permission;
        use std::collections::HashSet;
        
        let mut permission_objects = HashSet::new();
        for perm_str in &new_permissions {
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
                        total_revoked += removed_permissions.len();
                        
                        // Get user email for response  
                        let email = Some(updated_user.email().to_string());
                
                successful.push(BulkUserResult {
                    wallet_address: user_id.clone(),
                    email,
                    permissions_added: vec![],
                    permissions_removed: removed_permissions,
                    previous_permissions,
                    current_permissions: new_permissions,
                });
                    },
                    Err(e) => {
                        failed.push(BulkUserError {
                            wallet_address: user_id.clone(),
                            error: format!("Failed to save user: {}", e),
                            error_code: "SAVE_FAILED".to_string(),
                        });
                    }
                }
            },
            Err(e) => {
                failed.push(BulkUserError {
                    wallet_address: user_id.clone(),
                    error: format!("Failed to update permissions: {}", e),
                    error_code: "UPDATE_FAILED".to_string(),
                });
            }
        }
    }
    
    let summary = BulkSummary {
        total_users: request.user_ids.len() as i32,
        successful_operations: successful.len() as i32,
        failed_operations: failed.len() as i32,
        permissions_granted: 0,
        permissions_revoked: total_revoked as i32,
    };
    
    tracing::info!("Bulk revoke completed: {} successful, {} failed, {} permissions revoked", 
                   summary.successful_operations, summary.failed_operations, summary.permissions_revoked);
    
    Ok(Json(BulkOperationResponse {
        successful,
        failed,
        summary,
        operation: "bulk_revoke_permissions".to_string(),
        timestamp: Utc::now(),
    }))
}

/// Bulk assign roles to multiple users
/// POST /admin/users/bulk/roles/assign - DEPRECATED
pub async fn bulk_assign_roles(
    State(_state): State<AppState>,
) -> Result<Json<BulkOperationResponse>, (StatusCode, String)> {
    // Role assignment removed - using permissions-based system
    Err((StatusCode::NOT_IMPLEMENTED, "Role assignment deprecated - use permissions API".to_string()))
}

/// Apply permission template to multiple users
/// POST /admin/users/bulk/templates/apply
pub async fn bulk_apply_permission_template(
    State(state): State<AppState>,
    Json(request): Json<BulkPermissionTemplateRequest>,
) -> Result<Json<BulkOperationResponse>, (StatusCode, String)> {
    tracing::info!("Bulk applying template '{}' to {} users", request.template_id, request.user_ids.len());
    
    if request.user_ids.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No user IDs provided".to_string()));
    }
    
    let templates = get_permission_templates();
    let template_permissions = match templates.get(&request.template_id) {
        Some(perms) => perms,
        None => {
            return Err((StatusCode::BAD_REQUEST, format!("Unknown template: {}", request.template_id)));
        }
    };
    
    // Convert to grant request and reuse the bulk grant logic
    let grant_request = BulkGrantPermissionsRequest {
        user_ids: request.user_ids,
        permissions: template_permissions.clone(),
        reason: request.reason,
        expires_at: request.expires_at,
        notify_users: None,
    };
    
    let response = bulk_grant_permissions(State(state), Json(grant_request)).await?;
    
    // Update operation name to reflect template application
    let Json(mut response_data) = response;
    response_data.operation = format!("bulk_apply_template_{}", request.template_id);
    
    Ok(Json(response_data))
}

/// Validate permissions for multiple users
/// POST /admin/users/bulk/permissions/validate
pub async fn bulk_validate_permissions(
    State(state): State<AppState>,
    Json(request): Json<BulkPermissionValidationRequest>,
) -> Result<Json<BulkValidationResponse>, (StatusCode, String)> {
    tracing::info!("Bulk validating permissions for {} users", request.user_ids.len());
    
    if request.user_ids.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No user IDs provided".to_string()));
    }
    
    let mut user_validations = Vec::new();
    let _check_expired = request.check_expired.unwrap_or(true);
    let _check_conflicting = request.check_conflicting.unwrap_or(true);
    
    for user_id in &request.user_ids {
        let user_id_typed = match UserId::from_str(user_id) {
            Ok(id) => id,
            Err(_) => {
                // Skip invalid user IDs
                continue;
            }
        };
        
        // Get user from repository
        let user = match state.user_repo.find_by_id(&user_id_typed).await {
            Ok(Some(user)) => user,
            Ok(None) | Err(_) => continue, // Skip users that can't be retrieved
        };
        
        // Get user permissions from User aggregate
        let permission_strings = user.active_permissions();
        
        // Get user email
        let email = Some(user.email().to_string());
        
        // Validate permissions
        let mut valid_permissions = Vec::new();
        let expired_permissions = Vec::new();
        let conflicting_permissions = Vec::new();
        
        for permission in &permission_strings {
            // Check if permission is valid (basic format validation)
            if permission.matches(':').count() >= 2 {
                valid_permissions.push(permission.clone());
            }
            
            // TODO: Check for expired permissions if check_expired is true
            // This would require parsing timestamp-embedded permissions
            
            // TODO: Check for conflicting permissions if check_conflicting is true
            // This would require permission conflict rules
        }
        
        // Calculate health score (simple implementation)
        let total_permissions = permission_strings.len() as f32;
        let valid_count = valid_permissions.len() as f32;
        let health_score = if total_permissions > 0.0 {
            (valid_count / total_permissions) * 100.0
        } else {
            100.0
        };
        
        user_validations.push(UserPermissionValidation {
            wallet_address: user_id.clone(),
            email,
            valid_permissions,
            expired_permissions,
            conflicting_permissions,
            health_score,
        });
    }
    
    // Calculate summary statistics
    let total_users = user_validations.len() as i32;
    let users_with_issues = user_validations.iter()
        .filter(|v| !v.expired_permissions.is_empty() || !v.conflicting_permissions.is_empty())
        .count() as i32;
    
    let total_expired = user_validations.iter()
        .map(|v| v.expired_permissions.len())
        .sum::<usize>() as i32;
    
    let total_conflicting = user_validations.iter()
        .map(|v| v.conflicting_permissions.len())
        .sum::<usize>() as i32;
    
    let average_health_score = if total_users > 0 {
        user_validations.iter()
            .map(|v| v.health_score)
            .sum::<f32>() / total_users as f32
    } else {
        100.0
    };
    
    let summary = ValidationSummary {
        total_users_checked: total_users,
        users_with_issues,
        total_expired_permissions: total_expired,
        total_conflicting_permissions: total_conflicting,
        average_health_score,
    };
    
    tracing::info!("Bulk validation completed: {} users checked, {} with issues, avg health: {:.1}%", 
                   summary.total_users_checked, summary.users_with_issues, summary.average_health_score);
    
    Ok(Json(BulkValidationResponse {
        user_validations,
        summary,
    }))
}