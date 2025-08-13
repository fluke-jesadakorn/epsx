use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    app::ports::repositories::{TemporaryPermissionQuery},
    dom::entities::{TemporaryPermission, TemporaryPermissionStatus},
    dom::values::UserId,
    core::errors::{AppError, ErrorKind},
    web::auth::AppState,
};

#[derive(Debug, Deserialize)]
pub struct CreateTemporaryPermissionRequest {
    pub user_id: UserId,
    pub permission: String,
    pub resource: String,
    pub action: String,
    pub expires_at: DateTime<Utc>,
    pub reason: Option<String>,
    pub conditions: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTemporaryPermissionRequest {
    pub permission: Option<String>,
    pub resource: Option<String>,
    pub action: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub reason: Option<String>,
    pub conditions: Option<serde_json::Value>,
    pub status: Option<TemporaryPermissionStatus>,
}

#[derive(Debug, Deserialize)]
pub struct ListTemporaryPermissionsQuery {
    pub user_id: Option<UserId>,
    pub permission: Option<String>,
    pub resource: Option<String>,
    pub action: Option<String>,
    pub status: Option<TemporaryPermissionStatus>,
    pub active_only: Option<bool>,
    pub expires_before: Option<DateTime<Utc>>,
    pub expires_after: Option<DateTime<Utc>>,
    pub granted_by: Option<UserId>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct TemporaryPermissionResponse {
    pub id: Uuid,
    pub user_id: UserId,
    pub permission: String,
    pub resource: String,
    pub action: String,
    pub granted_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub auto_revoke: bool,
    pub granted_by: UserId,
    pub reason: Option<String>,
    pub conditions: serde_json::Value,
    pub status: TemporaryPermissionStatus,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_by: Option<UserId>,
    pub revocation_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
    pub is_expired: bool,
}

#[derive(Debug, Serialize)]
pub struct ListTemporaryPermissionsResponse {
    pub permissions: Vec<TemporaryPermissionResponse>,
    pub total: i64,
    pub limit: i32,
    pub offset: i32,
}

#[derive(Debug, Deserialize)]
pub struct RevokePermissionRequest {
    pub reason: Option<String>,
}

impl From<TemporaryPermission> for TemporaryPermissionResponse {
    fn from(permission: TemporaryPermission) -> Self {
        let is_active = permission.is_active();
        let is_expired = permission.is_expired();
        
        Self {
            id: permission.id,
            user_id: UserId(permission.user_id),
            permission: permission.permission,
            resource: permission.resource,
            action: permission.action,
            granted_at: permission.granted_at,
            expires_at: permission.expires_at,
            auto_revoke: permission.auto_revoke,
            granted_by: UserId(permission.granted_by),
            reason: permission.reason,
            conditions: permission.conditions,
            status: permission.status,
            revoked_at: permission.revoked_at,
            revoked_by: permission.revoked_by.map(|id| UserId(id)),
            revocation_reason: permission.revocation_reason,
            created_at: permission.created_at,
            updated_at: permission.updated_at,
            is_active,
            is_expired,
        }
    }
}

async fn verify_admin_access(resource: &str, action: &str) -> Result<(), AppError> {
    // TODO: Extract user ID from authenticated context
    let user_id = "admin_user"; // Placeholder - in production get from session/token

    // Modern JWT-based permission check
    // TODO: Implement modern permission verification logic  
    let permission_granted = true; // Placeholder
    if permission_granted {
        tracing::debug!("Admin access granted for {} on {}/{}", user_id, resource, action);
        Ok(())
    } else {
        tracing::warn!("Admin access denied for {} on {}/{}", user_id, resource, action);
        Err(AppError::new(
            ErrorKind::AuthorizationError,
            format!("Access denied for {}/{}", resource, action),
        ))
    }
}

/// Create a new temporary permission
pub async fn create_temporary_permission_handler(
    State(app_state): State<AppState>,
    Json(request): Json<CreateTemporaryPermissionRequest>,
) -> Result<Json<TemporaryPermissionResponse>, AppError> {
    // Verify admin access
    verify_admin_access("temporary_permissions", "create").await?;

    let temp_permission = TemporaryPermission::new(
        request.user_id.0,
        request.permission,
        request.resource,
        request.action,
        request.expires_at,
        Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(), // TODO: Get from auth context
        request.reason,
    );

    // Set conditions if provided
    let mut temp_permission = temp_permission;
    if let Some(conditions) = request.conditions {
        temp_permission.set_conditions(conditions);
    }

    let repo = &app_state.temporary_permission_repo;
    let created_permission = repo.create(&temp_permission).await
        .map_err(|e| AppError::new(ErrorKind::InternalServerError, format!("Failed to create temporary permission: {}", e)))?;

    Ok(Json(TemporaryPermissionResponse::from(created_permission)))
}

/// Get temporary permission by ID
pub async fn get_temporary_permission_handler(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<TemporaryPermissionResponse>, AppError> {
    verify_admin_access("temporary_permissions", "read").await?;
    let repo = &app_state.temporary_permission_repo;
    let permission = repo.find_by_id(&id).await
        .map_err(|e| AppError::new(ErrorKind::InternalServerError, format!("Failed to get temporary permission: {}", e)))?
        .ok_or(AppError::new(ErrorKind::AggregateNotFound, "Temporary permission not found"))?;

    Ok(Json(TemporaryPermissionResponse::from(permission)))
}

/// List temporary permissions with filtering
pub async fn list_temporary_permissions_handler(
    State(app_state): State<AppState>,
    Query(params): Query<ListTemporaryPermissionsQuery>,
) -> Result<Json<ListTemporaryPermissionsResponse>, AppError> {
    verify_admin_access("temporary_permissions", "read").await?;

    let query = TemporaryPermissionQuery {
        user_id: params.user_id,
        permission: params.permission,
        resource: params.resource,
        action: params.action,
        status: params.status,
        active_only: params.active_only,
        expires_before: params.expires_before,
        expires_after: params.expires_after,
        granted_by: params.granted_by,
        limit: params.limit,
        offset: params.offset,
    };

    let repo = &app_state.temporary_permission_repo;
    
    let permissions = repo.find_by_query(&query).await
        .map_err(|e| AppError::new(ErrorKind::InternalServerError, format!("Failed to list temporary permissions: {}", e)))?;

    let total = repo.count_by_query(&query).await
        .map_err(|e| AppError::new(ErrorKind::InternalServerError, format!("Failed to count temporary permissions: {}", e)))?;

    let response = ListTemporaryPermissionsResponse {
        permissions: permissions.into_iter().map(TemporaryPermissionResponse::from).collect(),
        total,
        limit: query.limit.unwrap_or(100),
        offset: query.offset.unwrap_or(0),
    };

    Ok(Json(response))
}

/// Update a temporary permission
pub async fn update_temporary_permission_handler(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateTemporaryPermissionRequest>,
) -> Result<Json<TemporaryPermissionResponse>, AppError> {
    verify_admin_access("temporary_permissions", "update").await?;

    let repo = &app_state.temporary_permission_repo;
    let mut permission = repo.find_by_id(&id).await
        .map_err(|e| AppError::new(ErrorKind::InternalServerError, format!("Failed to get temporary permission: {}", e)))?
        .ok_or(AppError::new(ErrorKind::AggregateNotFound, "Temporary permission not found"))?;

    // Update fields if provided
    if let Some(new_permission) = request.permission {
        permission.permission = new_permission;
        permission.touch_updated_at();
    }
    
    if let Some(new_resource) = request.resource {
        permission.resource = new_resource;
        permission.touch_updated_at();
    }
    
    if let Some(new_action) = request.action {
        permission.action = new_action;
        permission.touch_updated_at();
    }
    
    if let Some(new_expires_at) = request.expires_at {
        permission.extend_expiry(new_expires_at);
    }
    
    if let Some(new_reason) = request.reason {
        permission.reason = Some(new_reason);
        permission.touch_updated_at();
    }
    
    if let Some(new_conditions) = request.conditions {
        permission.set_conditions(new_conditions);
    }

    if let Some(new_status) = request.status {
        match new_status {
            TemporaryPermissionStatus::Active => permission.activate(),
            TemporaryPermissionStatus::Suspended => permission.suspend(),
            TemporaryPermissionStatus::Expired => permission.expire(),
            TemporaryPermissionStatus::Revoked => {
                permission.revoke(
                    Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(), // TODO: Get from auth context
                    Some("Updated via API".to_string())
                );
            }
        }
    }

    let updated_permission = repo.update(&permission).await
        .map_err(|e| AppError::new(ErrorKind::InternalServerError, format!("Failed to update temporary permission: {}", e)))?;

    Ok(Json(TemporaryPermissionResponse::from(updated_permission)))
}

/// Revoke a temporary permission
pub async fn revoke_temporary_permission_handler(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<RevokePermissionRequest>,
) -> Result<StatusCode, AppError> {
    verify_admin_access("temporary_permissions", "revoke").await?;

    let repo = &app_state.temporary_permission_repo;
    let mut permission = repo.find_by_id(&id).await
        .map_err(|e| AppError::new(ErrorKind::InternalServerError, format!("Failed to get temporary permission: {}", e)))?
        .ok_or(AppError::new(ErrorKind::AggregateNotFound, "Temporary permission not found"))?;

    permission.revoke(
        Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(), // TODO: Get from auth context
        request.reason.or_else(|| Some("Revoked via API".to_string()))
    );

    repo.update(&permission).await
        .map_err(|e| AppError::new(ErrorKind::InternalServerError, format!("Failed to revoke temporary permission: {}", e)))?;

    Ok(StatusCode::NO_CONTENT)
}

/// Delete a temporary permission
pub async fn delete_temporary_permission_handler(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    verify_admin_access("temporary_permissions", "delete").await?;

    let repo = &app_state.temporary_permission_repo;
    let deleted = repo.delete(&id).await
        .map_err(|e| AppError::new(ErrorKind::InternalServerError, format!("Failed to delete temporary permission: {}", e)))?;

    if !deleted {
        return Err(AppError::new(ErrorKind::AggregateNotFound, "Temporary permission not found"));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Get active temporary permissions for a specific user
pub async fn get_user_temporary_permissions_handler(
    State(app_state): State<AppState>,
    Path(user_id): Path<UserId>,
) -> Result<Json<Vec<TemporaryPermissionResponse>>, AppError> {
    verify_admin_access("temporary_permissions", "read").await?;

    let repo = &app_state.temporary_permission_repo;
    let permissions = repo.find_active_for_user(&user_id).await
        .map_err(|e| AppError::new(ErrorKind::InternalServerError, format!("Failed to get user temporary permissions: {}", e)))?;

    let response: Vec<TemporaryPermissionResponse> = permissions
        .into_iter()
        .map(TemporaryPermissionResponse::from)
        .collect();

    Ok(Json(response))
}

/// Clean up expired temporary permissions (admin only)
pub async fn cleanup_expired_permissions_handler(
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    verify_admin_access("temporary_permissions", "cleanup").await?;

    let repo = &app_state.temporary_permission_repo;
    let cleaned_count = repo.cleanup_expired().await
        .map_err(|e| AppError::new(ErrorKind::InternalServerError, format!("Failed to cleanup expired permissions: {}", e)))?;

    Ok(Json(serde_json::json!({
        "message": "Cleanup completed successfully",
        "cleaned_count": cleaned_count
    })))
}

// Bulk Operations Handlers

#[derive(Debug, Deserialize)]
pub struct BulkCreateTemporaryPermissionsRequest {
    pub permissions: Vec<CreateTemporaryPermissionRequest>,
}

#[derive(Debug, Serialize)]
pub struct BulkCreateTemporaryPermissionsResponse {
    pub created: Vec<TemporaryPermissionResponse>,
    pub failed: Vec<BulkOperationError>,
    pub summary: BulkOperationSummary,
}

#[derive(Debug, Deserialize)]
pub struct BulkRevokeTemporaryPermissionsRequest {
    pub permission_ids: Vec<Uuid>,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BulkRevokeTemporaryPermissionsResponse {
    pub revoked: Vec<Uuid>,
    pub failed: Vec<BulkOperationError>,
    pub summary: BulkOperationSummary,
}

#[derive(Debug, Deserialize)]
pub struct BulkUpdateTemporaryPermissionsRequest {
    pub updates: Vec<BulkPermissionUpdate>,
}

#[derive(Debug, Deserialize)]
pub struct BulkPermissionUpdate {
    pub id: Uuid,
    pub updates: UpdateTemporaryPermissionRequest,
}

#[derive(Debug, Serialize)]
pub struct BulkUpdateTemporaryPermissionsResponse {
    pub updated: Vec<TemporaryPermissionResponse>,
    pub failed: Vec<BulkOperationError>,
    pub summary: BulkOperationSummary,
}

#[derive(Debug, Serialize)]
pub struct BulkOperationError {
    pub id: Option<Uuid>,
    pub error: String,
    pub details: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BulkOperationSummary {
    pub total_requested: usize,
    pub successful: usize,
    pub failed: usize,
    pub execution_time_ms: u64,
}

/// Bulk create temporary permissions
pub async fn bulk_create_temporary_permissions_handler(
    State(app_state): State<AppState>,
    Json(request): Json<BulkCreateTemporaryPermissionsRequest>,
) -> Result<Json<BulkCreateTemporaryPermissionsResponse>, AppError> {
    verify_admin_access("temporary_permissions", "bulk_create").await?;

    let start_time = std::time::Instant::now();
    let mut created = Vec::new();
    let mut failed = Vec::new();

    for (index, perm_request) in request.permissions.into_iter().enumerate() {
        let temp_permission = TemporaryPermission::new(
            perm_request.user_id.0,
            perm_request.permission,
            perm_request.resource,
            perm_request.action,
            perm_request.expires_at,
            Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(), // TODO: Get from auth context
            perm_request.reason,
        );

        let mut temp_permission = temp_permission;
        if let Some(conditions) = perm_request.conditions {
            temp_permission.set_conditions(conditions);
        }

        match app_state.temporary_permission_repo.create(&temp_permission).await {
            Ok(created_permission) => {
                created.push(TemporaryPermissionResponse::from(created_permission));
            }
            Err(e) => {
                failed.push(BulkOperationError {
                    id: None,
                    error: format!("Failed to create permission #{}: {}", index + 1, e),
                    details: Some(format!("User ID: {}", temp_permission.user_id)),
                });
            }
        }
    }

    let execution_time = start_time.elapsed();
    let successful_count = created.len();
    let failed_count = failed.len();
    let total_requested = successful_count + failed_count;

    let response = BulkCreateTemporaryPermissionsResponse {
        created,
        failed,
        summary: BulkOperationSummary {
            total_requested,
            successful: successful_count,
            failed: failed_count,
            execution_time_ms: execution_time.as_millis() as u64,
        },
    };

    Ok(Json(response))
}

/// Bulk revoke temporary permissions
pub async fn bulk_revoke_temporary_permissions_handler(
    State(app_state): State<AppState>,
    Json(request): Json<BulkRevokeTemporaryPermissionsRequest>,
) -> Result<Json<BulkRevokeTemporaryPermissionsResponse>, AppError> {
    verify_admin_access("temporary_permissions", "bulk_revoke").await?;

    let start_time = std::time::Instant::now();
    let mut revoked = Vec::new();
    let mut failed = Vec::new();

    let repo = &app_state.temporary_permission_repo;
    
    for permission_id in request.permission_ids {
        match repo.find_by_id(&permission_id).await {
            Ok(Some(mut permission)) => {
                permission.revoke(
                    Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(), // TODO: Get from auth context
                    request.reason.clone().or_else(|| Some("Bulk revoked via API".to_string()))
                );

                match repo.update(&permission).await {
                    Ok(_) => revoked.push(permission_id),
                    Err(e) => {
                        failed.push(BulkOperationError {
                            id: Some(permission_id),
                            error: format!("Failed to update permission: {}", e),
                            details: None,
                        });
                    }
                }
            }
            Ok(None) => {
                failed.push(BulkOperationError {
                    id: Some(permission_id),
                    error: "Permission not found".to_string(),
                    details: None,
                });
            }
            Err(e) => {
                failed.push(BulkOperationError {
                    id: Some(permission_id),
                    error: format!("Failed to find permission: {}", e),
                    details: None,
                });
            }
        }
    }

    let execution_time = start_time.elapsed();
    let successful_count = revoked.len();
    let failed_count = failed.len();
    let total_requested = successful_count + failed_count;

    let response = BulkRevokeTemporaryPermissionsResponse {
        revoked,
        failed,
        summary: BulkOperationSummary {
            total_requested,
            successful: successful_count,
            failed: failed_count,
            execution_time_ms: execution_time.as_millis() as u64,
        },
    };

    Ok(Json(response))
}

/// Bulk update temporary permissions
pub async fn bulk_update_temporary_permissions_handler(
    State(app_state): State<AppState>,
    Json(request): Json<BulkUpdateTemporaryPermissionsRequest>,
) -> Result<Json<BulkUpdateTemporaryPermissionsResponse>, AppError> {
    verify_admin_access("temporary_permissions", "bulk_update").await?;

    let start_time = std::time::Instant::now();
    let mut updated = Vec::new();
    let mut failed = Vec::new();

    let repo = &app_state.temporary_permission_repo;

    for update_request in request.updates {
        match repo.find_by_id(&update_request.id).await {
            Ok(Some(mut permission)) => {
                // Apply updates
                if let Some(new_permission) = update_request.updates.permission {
                    permission.permission = new_permission;
                    permission.touch_updated_at();
                }
                
                if let Some(new_resource) = update_request.updates.resource {
                    permission.resource = new_resource;
                    permission.touch_updated_at();
                }
                
                if let Some(new_action) = update_request.updates.action {
                    permission.action = new_action;
                    permission.touch_updated_at();
                }
                
                if let Some(new_expires_at) = update_request.updates.expires_at {
                    permission.extend_expiry(new_expires_at);
                }
                
                if let Some(new_reason) = update_request.updates.reason {
                    permission.reason = Some(new_reason);
                    permission.touch_updated_at();
                }
                
                if let Some(new_conditions) = update_request.updates.conditions {
                    permission.set_conditions(new_conditions);
                }

                if let Some(new_status) = update_request.updates.status {
                    match new_status {
                        TemporaryPermissionStatus::Active => permission.activate(),
                        TemporaryPermissionStatus::Suspended => permission.suspend(),
                        TemporaryPermissionStatus::Expired => permission.expire(),
                        TemporaryPermissionStatus::Revoked => {
                            permission.revoke(
                                Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(), // TODO: Get from auth context
                                Some("Bulk updated via API".to_string())
                            );
                        }
                    }
                }

                match repo.update(&permission).await {
                    Ok(updated_permission) => {
                        updated.push(TemporaryPermissionResponse::from(updated_permission));
                    }
                    Err(e) => {
                        failed.push(BulkOperationError {
                            id: Some(update_request.id),
                            error: format!("Failed to update permission: {}", e),
                            details: None,
                        });
                    }
                }
            }
            Ok(None) => {
                failed.push(BulkOperationError {
                    id: Some(update_request.id),
                    error: "Permission not found".to_string(),
                    details: None,
                });
            }
            Err(e) => {
                failed.push(BulkOperationError {
                    id: Some(update_request.id),
                    error: format!("Failed to find permission: {}", e),
                    details: None,
                });
            }
        }
    }

    let execution_time = start_time.elapsed();
    let successful_count = updated.len();
    let failed_count = failed.len();
    let total_requested = successful_count + failed_count;

    let response = BulkUpdateTemporaryPermissionsResponse {
        updated,
        failed,
        summary: BulkOperationSummary {
            total_requested,
            successful: successful_count,
            failed: failed_count,
            execution_time_ms: execution_time.as_millis() as u64,
        },
    };

    Ok(Json(response))
}