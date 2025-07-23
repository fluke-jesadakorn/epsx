// Admin API handlers for user management

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use crate::app::dtos::{
    CreateUserReq, CreateUserRes, GetUserReq, GetUserRes, UpdateRoleReq, UpdateRoleRes,
    ListUsersReq, ListUsersRes, BulkUpdateLevelsReq, BulkUpdateLevelsRes, UserStatsReq,
    UserStatsRes, GetLevelHistoryReq, GetLevelHistoryRes,
};
use crate::app::use_cases::UserUseCaseError;
use crate::dom::values::{UserId, Role};
use crate::web::middleware::auth_middleware::AuthCtx;
use crate::web::AppState;

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

// Admin handler implementations

/// GET /admin/users - List all users with pagination and filtering
pub async fn list_users_handler(
    State(app_state): State<AppState>,
    auth_ctx: AuthCtx,
    Query(query): Query<AdminListUsersQuery>,
) -> Result<Json<ListUsersRes>, StatusCode> {
    // Verify admin permissions
    verify_admin_permissions(&app_state, &auth_ctx.user_id).await?;

    let req = ListUsersReq {
        offset: query.offset.unwrap_or(0),
        limit: query.limit.unwrap_or(50),
        role_filter: query.role_filter,
        page_token: query.page_token,
    };

    app_state
        .user_mgmt_uc
        .list_firebase_users(req)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("Failed to list Firebase users: {:?}", e);
            match e {
                UserUseCaseError::ValidationError(_) => StatusCode::BAD_REQUEST,
                UserUseCaseError::PermissionDenied => StatusCode::FORBIDDEN,
                UserUseCaseError::ExternalServiceError(_) => StatusCode::SERVICE_UNAVAILABLE,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })
}

/// POST /admin/users - Create a new user (admin only)
pub async fn create_user_handler(
    State(app_state): State<AppState>,
    auth_ctx: AuthCtx,
    Json(req): Json<AdminCreateUserRequest>,
) -> Result<Json<CreateUserRes>, StatusCode> {
    // Verify admin permissions
    verify_admin_permissions(&app_state, &auth_ctx.user_id).await?;

    let create_req = CreateUserReq {
        email: req.email,
        role: req.role,
        fb_token: req.fb_token,
    };

    app_state
        .user_mgmt_uc
        .create_user(create_req)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("Failed to create user: {:?}", e);
            match e {
                UserUseCaseError::ValidationError(_) => StatusCode::BAD_REQUEST,
                UserUseCaseError::UserAlreadyExists(_) => StatusCode::CONFLICT,
                UserUseCaseError::PermissionDenied => StatusCode::FORBIDDEN,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })
}

/// GET /admin/users/{user_id} - Get specific user details
pub async fn get_user_handler(
    State(app_state): State<AppState>,
    auth_ctx: AuthCtx,
    Path(user_id): Path<String>,
) -> Result<Json<GetUserRes>, StatusCode> {
    // Verify admin permissions
    verify_admin_permissions(&app_state, &auth_ctx.user_id).await?;

    let user_id = UserId::from_str(&user_id).map_err(|e| {
        tracing::error!("Invalid user ID format: {} - {:?}", user_id, e);
        StatusCode::BAD_REQUEST
    })?;

    let req = GetUserReq { usr_id: user_id };

    app_state
        .user_mgmt_uc
        .get_user(req)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("Failed to get user: {:?}", e);
            match e {
                UserUseCaseError::UserNotFound(_) => StatusCode::NOT_FOUND,
                UserUseCaseError::PermissionDenied => StatusCode::FORBIDDEN,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })
}

/// PUT /admin/users/{user_id} - Update user role
pub async fn update_user_role_handler(
    State(app_state): State<AppState>,
    auth_ctx: AuthCtx,
    Path(user_id): Path<String>,
    Json(req): Json<AdminUpdateUserRequest>,
) -> Result<Json<UpdateRoleRes>, StatusCode> {
    // Verify admin permissions
    verify_admin_permissions(&app_state, &auth_ctx.user_id).await?;

    let user_id = UserId::from_str(&user_id).map_err(|e| {
        tracing::error!("Invalid user ID format: {} - {:?}", user_id, e);
        StatusCode::BAD_REQUEST
    })?;

    let new_role = req.role.ok_or_else(|| {
        tracing::error!("Missing role in update request");
        StatusCode::BAD_REQUEST
    })?;

    let update_req = UpdateRoleReq {
        usr_id: user_id,
        new_role,
        admin_id: auth_ctx.user_id,
    };

    app_state
        .user_mgmt_uc
        .update_role(update_req)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("Failed to update user role: {:?}", e);
            match e {
                UserUseCaseError::ValidationError(_) => StatusCode::BAD_REQUEST,
                UserUseCaseError::UserNotFound(_) => StatusCode::NOT_FOUND,
                UserUseCaseError::PermissionDenied => StatusCode::FORBIDDEN,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })
}

/// POST /admin/users/bulk-update-levels - Bulk update user levels
pub async fn bulk_update_levels_handler(
    State(app_state): State<AppState>,
    auth_ctx: AuthCtx,
    Json(req): Json<BulkUpdateLevelsReq>,
) -> Result<Json<BulkUpdateLevelsRes>, StatusCode> {
    // Verify admin permissions
    verify_admin_permissions(&app_state, &auth_ctx.user_id).await?;

    // Ensure the admin_id in the request matches the current user
    let mut bulk_req = req;
    bulk_req.admin_id = auth_ctx.user_id;

    app_state
        .user_mgmt_uc
        .bulk_update_levels(bulk_req)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("Failed to bulk update user levels: {:?}", e);
            match e {
                UserUseCaseError::ValidationError(_) => StatusCode::BAD_REQUEST,
                UserUseCaseError::PermissionDenied => StatusCode::FORBIDDEN,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })
}

/// GET /admin/stats - Get user statistics
pub async fn get_user_stats_handler(
    State(app_state): State<AppState>,
    auth_ctx: AuthCtx,
    Query(query): Query<AdminUserStatsQuery>,
) -> Result<Json<UserStatsRes>, StatusCode> {
    // Verify admin permissions
    verify_admin_permissions(&app_state, &auth_ctx.user_id).await?;

    let req = UserStatsReq {
        include_roles: query.include_roles.unwrap_or(true),
        include_tiers: query.include_tiers.unwrap_or(true),
        start_date: None,
        end_date: None,
    };

    app_state
        .user_mgmt_uc
        .get_user_statistics(req)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("Failed to get user statistics: {:?}", e);
            match e {
                UserUseCaseError::ValidationError(_) => StatusCode::BAD_REQUEST,
                UserUseCaseError::PermissionDenied => StatusCode::FORBIDDEN,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })
}

/// GET /admin/users/{user_id}/level-history - Get user level change history
pub async fn get_level_history_handler(
    State(app_state): State<AppState>,
    auth_ctx: AuthCtx,
    Path(user_id): Path<String>,
) -> Result<Json<GetLevelHistoryRes>, StatusCode> {
    // Verify admin permissions
    verify_admin_permissions(&app_state, &auth_ctx.user_id).await?;

    let user_id = UserId::from_str(&user_id).map_err(|e| {
        tracing::error!("Invalid user ID format: {} - {:?}", user_id, e);
        StatusCode::BAD_REQUEST
    })?;

    let req = GetLevelHistoryReq {
        usr_id: user_id,
        limit: Some(50),
        offset: Some(0),
    };

    app_state
        .user_mgmt_uc
        .get_level_history(req)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("Failed to get level history: {:?}", e);
            match e {
                UserUseCaseError::UserNotFound(_) => StatusCode::NOT_FOUND,
                UserUseCaseError::PermissionDenied => StatusCode::FORBIDDEN,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })
}

// Helper function to verify admin permissions
async fn verify_admin_permissions(
    app_state: &AppState,
    user_id: &UserId,
) -> Result<(), StatusCode> {
    let user = app_state
        .user_repo
        .get(user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get user for permission check: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::error!("User not found for permission check: {}", user_id);
            StatusCode::UNAUTHORIZED
        })?;

    match user.role() {
        Role::Admin | Role::SuperAdmin => Ok(()),
        _ => {
            tracing::warn!(
                "User {} attempted to access admin endpoint without permissions",
                user_id
            );
            Err(StatusCode::FORBIDDEN)
        }
    }
}