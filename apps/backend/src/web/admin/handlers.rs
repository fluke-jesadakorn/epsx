// Admin API handlers for user management with Casbin authorization

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use crate::web::auth::AppState;
use chrono::{DateTime, Utc};
use serde_json::{json, Value};

// Request/Response DTOs for admin endpoints

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminCreateUserRequest {
    pub email: String,
    pub role: String,
    pub fb_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminUpdateUserRequest {
    pub role: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminListUsersQuery {
    pub offset: Option<u32>,
    pub limit: Option<u32>,
    pub role_filter: Option<String>,
    pub page_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminUserStatsQuery {
    pub include_roles: Option<bool>,
    pub include_tiers: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminBulkUpdateRequest {
    pub user_ids: Vec<String>,
    pub new_level: Option<String>,
    pub new_role: Option<String>,
    pub batch_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminLevelHistoryQuery {
    pub user_id: String,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminPermissionProfileAssignRequest {
    pub profile_id: String,
    pub user_ids: Vec<String>,
    pub reason: Option<String>,
    pub merge_permissions: Option<bool>,
    pub expires_at: Option<DateTime<Utc>>,
    pub notify_users: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminPermissionProfileAssignResponse {
    pub profile_id: String,
    pub successful_assignments: Vec<PermissionProfileAssignmentResult>,
    pub failed_assignments: Vec<PermissionProfileAssignmentFailure>,
    pub total_assigned: u32,
    pub total_failed: u32,
    pub applied_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionProfileAssignmentResult {
    pub user_id: String,
    pub features_unlocked: Vec<String>,
    pub permissions_added: Vec<String>,
    pub assignment_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionProfileAssignmentFailure {
    pub user_id: String,
    pub error: String,
    pub error_code: String,
}

// Simplified admin handler implementations for Casbin migration

/// GET /admin/users - List all users with pagination and filtering
pub async fn list_users_handler(
    State(app_state): State<AppState>,
    Query(query): Query<AdminListUsersQuery>,
) -> Result<Json<Value>, StatusCode> {
    let user_id = extract_user_id_from_context()?;
    verify_admin_permissions(&app_state, &user_id, "/api/v1/admin/users", "GET").await?;
    
    tracing::info!("Admin list users handler called with authorization");
    
    let offset = query.offset.unwrap_or(0) as i64;
    let limit = query.limit.unwrap_or(50) as i64;
    
    // Get users from database with pagination
    let users = match app_state.user_repo.list(offset as u32, limit as u32).await {
        Ok(users) => users,
        Err(e) => {
            tracing::error!("Failed to fetch users: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Get total count for pagination
    let total_count = match app_state.user_repo.count().await {
        Ok(count) => count,
        Err(e) => {
            tracing::error!("Failed to count users: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    let user_list: Vec<Value> = users.into_iter().map(|user| {
        json!({
            "id": user.id().to_string(),
            "email": user.email().value(),
            "role": user.role().to_string(),
            "subscription_tier": user.sub().tier().to_string(),
            "is_active": user.is_active(),
            "created_at": user.created_at(),
            "updated_at": user.updated_at()
        })
    }).collect();
    
    Ok(Json(json!({
        "users": user_list,
        "total": total_count,
        "offset": offset,
        "limit": limit
    })))
}

/// POST /admin/users - Create a new user (admin only)
pub async fn create_user_handler(
    State(app_state): State<AppState>,
    Json(req): Json<AdminCreateUserRequest>,
) -> Result<Json<Value>, StatusCode> {
    let user_id = extract_user_id_from_context()?;
    verify_admin_permissions(&app_state, &user_id, "/api/v1/admin/users", "POST").await?;
    
    tracing::info!("Admin create user handler called with authorization for role: {}", req.role);
    
    // Parse and validate email
    let email = match crate::dom::values::Email::new(req.email.clone()) {
        Ok(email) => email,
        Err(_) => {
            tracing::warn!("Invalid email provided: {}", req.email);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    // Check if user already exists
    match app_state.user_repo.find_by_email(&email).await {
        Ok(Some(_)) => {
            tracing::warn!("User with email {} already exists", req.email);
            return Err(StatusCode::CONFLICT);
        },
        Ok(None) => {}, // Good, user doesn't exist
        Err(e) => {
            tracing::error!("Failed to check if user exists: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
    
    // Parse role
    let role = crate::dom::values::Role::from_string(&req.role).unwrap_or(crate::dom::values::Role::User);
    
    // Generate Firebase UID (in a real system, this would come from Firebase)
    let firebase_uid = req.fb_token.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    
    // Create new user
    let user = crate::dom::entities::User::new(firebase_uid, email, role);
    
    // Save user to database
    if let Err(e) = app_state.user_repo.save(&user).await {
        tracing::error!("Failed to create user: {:?}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    tracing::info!("Successfully created user with ID: {}", user.id());
    
    Ok(Json(json!({
        "message": "User created successfully",
        "user_id": user.id().to_string(),
        "email": user.email().value(),
        "role": user.role().to_string(),
        "created_at": user.created_at()
    })))
}

/// GET /admin/users/{user_id} - Get specific user details
pub async fn get_user_handler(
    State(_app_state): State<AppState>,
    Path(_user_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Implement with Casbin during migration
    tracing::info!("Admin get user handler called during migration");
    
    Ok(Json(json!({
        "message": "Get user details - implementation pending during migration"
    })))
}

/// PUT /admin/users/{user_id} - Update user details
pub async fn update_user_handler(
    State(_app_state): State<AppState>,
    Path(_user_id): Path<String>,
    Json(_req): Json<AdminUpdateUserRequest>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Implement with Casbin during migration
    tracing::info!("Admin update user handler called during migration");
    
    Ok(Json(json!({
        "message": "User update - implementation pending during migration"
    })))
}

/// DELETE /admin/users/{user_id} - Soft delete user
pub async fn delete_user_handler(
    State(_app_state): State<AppState>,
    Path(_user_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Implement with Casbin during migration
    tracing::info!("Admin delete user handler called during migration");
    
    Ok(Json(json!({
        "message": "User deletion - implementation pending during migration"
    })))
}

/// GET /admin/analytics/user-statistics - Get user statistics
pub async fn get_user_stats_handler(
    State(_app_state): State<AppState>,
    Query(_query): Query<AdminUserStatsQuery>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Implement with Casbin during migration
    tracing::info!("Admin user stats handler called during migration");
    
    Ok(Json(json!({
        "total_users": 0,
        "by_role": {},
        "by_tier": {},
        "message": "User statistics - implementation pending during migration"
    })))
}

/// POST /admin/users/bulk-update - Bulk update user levels
pub async fn bulk_update_users_handler(
    State(_app_state): State<AppState>,
    Json(_req): Json<AdminBulkUpdateRequest>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Implement with Casbin during migration
    tracing::info!("Admin bulk update handler called during migration");
    
    Ok(Json(json!({
        "message": "Bulk user update - implementation pending during migration"
    })))
}

/// GET /admin/users/{user_id}/level-history - Get user level history
pub async fn get_level_history_handler(
    State(_app_state): State<AppState>,
    Query(_query): Query<AdminLevelHistoryQuery>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Implement with Casbin during migration
    tracing::info!("Admin level history handler called during migration");
    
    Ok(Json(json!({
        "history": [],
        "message": "Level history - implementation pending during migration"
    })))
}

/// POST /admin/permission-profiles/assign - Assign permission profiles to users
pub async fn assign_permission_profiles_handler(
    State(_app_state): State<AppState>,
    Json(_req): Json<AdminPermissionProfileAssignRequest>,
) -> Result<Json<AdminPermissionProfileAssignResponse>, StatusCode> {
    // TODO: Implement with Casbin during migration
    tracing::info!("Admin assign permission profiles handler called during migration");
    
    Ok(Json(AdminPermissionProfileAssignResponse {
        profile_id: "migration_profile".to_string(),
        successful_assignments: vec![],
        failed_assignments: vec![],
        total_assigned: 0,
        total_failed: 0,
        applied_at: chrono::Utc::now(),
    }))
}

/// Helper function to verify admin permissions using Casbin
async fn verify_admin_permissions(
    app_state: &AppState,
    user_id: &str,
    resource: &str,
    action: &str,
) -> Result<(), StatusCode> {
    match app_state.casbin_service.enforce(user_id, resource, action).await {
        Ok(true) => {
            tracing::debug!("Admin permission granted for user {} on {}/{}", user_id, resource, action);
            Ok(())
        }
        Ok(false) => {
            tracing::warn!("Admin permission denied for user {} on {}/{}", user_id, resource, action);
            Err(StatusCode::FORBIDDEN)
        }
        Err(e) => {
            tracing::error!("Failed to check admin permissions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Extract user ID from request context - simplified for migration
/// TODO: Integrate with proper authentication middleware
fn extract_user_id_from_context() -> Result<String, StatusCode> {
    // For migration purposes, return a test admin user
    // In production, this would extract from JWT/session similar to user handlers
    // This function is maintained for backward compatibility with existing admin handlers
    Ok("admin".to_string())
}