use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::web::auth::routes::AppState;

/// Role assignment request from admin frontend
#[derive(Debug, Deserialize)]
pub struct AdminRoleAssignmentRequest {
    pub role: String,
    pub reason: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Permission update request
#[derive(Debug, Deserialize)]
pub struct PermissionUpdateRequest {
    pub permissions: Vec<String>,
}

/// Role query parameters
#[derive(Debug, Deserialize)]
pub struct RoleQuery {
    pub role: Option<String>,
    pub include_expired: Option<bool>,
}

/// User role response
#[derive(Debug, Serialize)]
pub struct UserRoleResponse {
    pub firebase_uid: String,
    pub role: String,
    pub permissions: Vec<String>,
    pub access_level: String,
    pub is_admin: bool,
    pub is_premium: bool,
    pub role_assigned_by: Option<String>,
    pub role_assigned_at: String,
    pub expires_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Role assignment audit response
#[derive(Debug, Serialize)]
pub struct RoleAuditResponse {
    pub id: String,
    pub firebase_uid: String,
    pub old_role: Option<String>,
    pub new_role: String,
    pub assigned_by: Option<String>,
    pub reason: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub timestamp: String,
}

/// Users by role response
#[derive(Debug, Serialize)]
pub struct UsersByRoleResponse {
    pub role: String,
    pub users: Vec<String>,
    pub count: usize,
}

/// Standard API error response
#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    pub error: String,
    pub message: String,
    pub details: Option<String>,
}

/// Get user role and permissions from database
/// GET /admin/roles/users/:firebase_uid
pub async fn get_user_role(
    State(_state): State<AppState>,
    Path(firebase_uid): Path<String>,
) -> Result<Json<UserRoleResponse>, (StatusCode, Json<ApiErrorResponse>)> {
    tracing::info!("Admin: Getting database role for Firebase user: {}", firebase_uid);
    
    // TODO: Get database pool from AppState
    // For now, return error indicating database not available
    Err((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ApiErrorResponse {
            error: "service_unavailable".to_string(),
            message: "Database role service not available".to_string(),
            details: Some("Database pool not configured in AppState".to_string()),
        }),
    ))
}

/// Assign role to user in database
/// POST /admin/roles/users/:firebase_uid/assign
pub async fn assign_user_role(
    State(_state): State<AppState>,
    Path(firebase_uid): Path<String>,
    Json(request): Json<AdminRoleAssignmentRequest>,
) -> Result<Json<UserRoleResponse>, (StatusCode, Json<ApiErrorResponse>)> {
    tracing::info!("Admin: Assigning role '{}' to Firebase user: {}", request.role, firebase_uid);
    
    // TODO: Extract admin Firebase UID from authentication context
    let _assigned_by = "admin_user_uid".to_string(); // Placeholder
    
    // TODO: Get database pool from AppState and create role service
    // For now, return error indicating database not available
    Err((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ApiErrorResponse {
            error: "service_unavailable".to_string(),
            message: "Database role service not available".to_string(),
            details: Some("Database pool not configured in AppState".to_string()),
        }),
    ))
}

/// Update user permissions directly (supports embedded timestamp permissions)
/// PUT /admin/roles/users/:firebase_uid/permissions
pub async fn update_user_permissions(
    State(state): State<AppState>,
    Path(firebase_uid): Path<String>,
    Json(request): Json<PermissionUpdateRequest>,
) -> Result<StatusCode, (StatusCode, Json<ApiErrorResponse>)> {
    tracing::info!("Admin: Updating permissions for Firebase user: {}", firebase_uid);
    
    match state.permission_application_service.update_user_permissions(&firebase_uid, request.permissions).await {
        Ok(()) => {
            tracing::info!("Successfully updated permissions for user: {}", firebase_uid);
            Ok(StatusCode::OK)
        },
        Err(e) => {
            tracing::error!("Failed to update permissions for user {}: {:?}", firebase_uid, e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ApiErrorResponse {
                    error: "update_failed".to_string(),
                    message: format!("Failed to update user permissions: {}", e),
                    details: Some(firebase_uid),
                }),
            ))
        }
    }
}

/// Revoke user role (reset to basic user)
/// DELETE /admin/roles/users/:firebase_uid
pub async fn revoke_user_role(
    State(_state): State<AppState>,
    Path(firebase_uid): Path<String>,
    Json(reason): Json<HashMap<String, String>>,
) -> Result<StatusCode, (StatusCode, Json<ApiErrorResponse>)> {
    tracing::info!("Admin: Revoking role for Firebase user: {}", firebase_uid);
    
    // TODO: Extract admin Firebase UID from authentication context
    let _revoked_by = "admin_user_uid".to_string(); // Placeholder
    let _revocation_reason = reason.get("reason").cloned();
    
    // TODO: Implement with database pool from AppState
    Err((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ApiErrorResponse {
            error: "service_unavailable".to_string(),
            message: "Database role service not available".to_string(),
            details: Some("Database pool not configured in AppState".to_string()),
        }),
    ))
}

/// List users by role
/// GET /admin/roles/users-by-role
pub async fn list_users_by_role(
    State(_state): State<AppState>,
    Query(query): Query<RoleQuery>,
) -> Result<Json<UsersByRoleResponse>, (StatusCode, Json<ApiErrorResponse>)> {
    let role = query.role.unwrap_or_else(|| "user-basic-001".to_string());
    tracing::info!("Admin: Listing users with role: {}", role);
    
    // TODO: Implement with database pool from AppState
    Err((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ApiErrorResponse {
            error: "service_unavailable".to_string(),
            message: "Database role service not available".to_string(),
            details: Some("Database pool not configured in AppState".to_string()),
        }),
    ))
}

/// Get role assignment history for user
/// GET /admin/roles/users/:firebase_uid/history
pub async fn get_role_assignment_history(
    State(_state): State<AppState>,
    Path(firebase_uid): Path<String>,
) -> Result<Json<Vec<RoleAuditResponse>>, (StatusCode, Json<ApiErrorResponse>)> {
    tracing::info!("Admin: Getting role assignment history for: {}", firebase_uid);
    
    // TODO: Implement with database pool from AppState
    Err((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ApiErrorResponse {
            error: "service_unavailable".to_string(),
            message: "Database role service not available".to_string(),
            details: Some("Database pool not configured in AppState".to_string()),
        }),
    ))
}

/// Clean up expired roles
/// POST /admin/roles/cleanup-expired
pub async fn cleanup_expired_roles(
    State(_state): State<AppState>,
) -> Result<Json<HashMap<String, u32>>, (StatusCode, Json<ApiErrorResponse>)> {
    tracing::info!("Admin: Cleaning up expired roles");
    
    // TODO: Implement with database pool from AppState
    Err((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ApiErrorResponse {
            error: "service_unavailable".to_string(),
            message: "Database role service not available".to_string(),
            details: Some("Database pool not configured in AppState".to_string()),
        }),
    ))
}

// Helper functions

