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
    UserStatsRes, GetLevelHistoryReq, GetLevelHistoryRes, SoftDeleteUserReq, SoftDeleteUserRes,
};
use crate::app::use_cases::UserUseCaseError;
use crate::dom::values::{UserId, Role};
use crate::dom::entities::permission_profile::{
    PermissionProfileId, PermissionProfileQuery, ApplyPermissionProfileRequest, PermissionProfileCategory
};
use crate::dom::entities::iam::PackageTier;
use crate::web::middleware::auth_middleware::AuthCtx;
use crate::web::AppState;
use chrono::{DateTime, Utc};

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

// Permission Profile assignment DTOs
#[derive(Debug, Serialize, Deserialize)]
pub struct AdminPermissionProfileListQuery {
    pub category: Option<String>,
    pub package_tier: Option<String>,
    pub active_only: Option<bool>,
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
        .list_users(req)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("Failed to list users: {:?}", e);
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

/// GET /admin/permission-profiles - List available permission profiles for assignment
pub async fn list_permission_profiles_handler(
    State(app_state): State<AppState>,
    auth_ctx: AuthCtx,
    Query(query): Query<AdminPermissionProfileListQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Verify admin permissions
    verify_admin_permissions(&app_state, &auth_ctx.user_id).await?;

    // Convert query parameters to domain objects
    let category = query.category.as_ref().and_then(|c| match c.as_str() {
        "user" => Some(PermissionProfileCategory::User),
        "moderator" => Some(PermissionProfileCategory::Moderator),
        "admin" => Some(PermissionProfileCategory::Admin),
        "custom" => Some(PermissionProfileCategory::Custom),
        _ => None,
    });

    let package_tier = query.package_tier.as_ref().and_then(|t| match t.as_str() {
        "Bronze" => Some(PackageTier::Bronze),
        "Silver" => Some(PackageTier::Silver),
        "Gold" => Some(PackageTier::Gold),
        "Platinum" => Some(PackageTier::Platinum),
        _ => None,
    });

    let profile_query = PermissionProfileQuery::new()
        .with_pagination(
            query.limit.unwrap_or(50),
            query.offset.unwrap_or(0)
        );

    let profile_query = if let Some(cat) = category {
        profile_query.by_category(cat)
    } else {
        profile_query
    };

    let profile_query = if let Some(tier) = package_tier {
        profile_query.by_tier(tier)
    } else {
        profile_query
    };

    let profile_query = if query.active_only.unwrap_or(true) {
        profile_query
    } else {
        profile_query.include_inactive()
    };

    // Get profiles from repository
    let profiles = app_state
        .permission_profile_repo
        .search(&profile_query)
        .await
        .map_err(|e| {
            tracing::error!("Failed to search profiles: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Convert profiles to response format
    let profiles_response: Vec<serde_json::Value> = profiles
        .into_iter()
        .map(|profile| {
            serde_json::json!({
                "id": profile.id().value(),
                "name": profile.name(),
                "description": profile.description(),
                "category": profile.category().to_string(),
                "target_tier": profile.target_tier().to_string(),
                "is_active": profile.is_active(),
                "permissions_count": profile.default_permissions().len(),
                "tags": profile.tags(),
                "created_at": profile.created_at(),
                "version": profile.version(),
                "metadata": {
                    "requires_approval": profile.metadata().requires_approval,
                    "max_assignments": profile.metadata().max_assignments,
                    "use_cases": profile.metadata().use_cases,
                    "warnings": profile.metadata().warnings
                }
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "profiles": profiles_response,
        "total": profiles_response.len(),
        "limit": query.limit.unwrap_or(50),
        "offset": query.offset.unwrap_or(0)
    })))
}

/// POST /admin/permission-profiles/assign - Assign permission profile to users directly (bypasses payment)
pub async fn assign_permission_profile_directly_handler(
    State(app_state): State<AppState>,
    auth_ctx: AuthCtx,
    Json(request): Json<AdminPermissionProfileAssignRequest>,
) -> Result<Json<AdminPermissionProfileAssignResponse>, StatusCode> {
    // Verify admin permissions
    verify_admin_permissions(&app_state, &auth_ctx.user_id).await?;

    let profile_id = PermissionProfileId::new(request.profile_id.clone());
    
    // Validate profile exists and is available
    let profile = app_state
        .permission_profile_repo
        .get(&profile_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get profile: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::error!("Permission profile not found: {}", request.profile_id);
            StatusCode::NOT_FOUND
        })?;

    if !profile.is_active() {
        tracing::warn!("Attempt to assign inactive profile: {}", request.profile_id);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Convert user IDs to UserId objects
    let user_ids: Result<Vec<UserId>, _> = request
        .user_ids
        .iter()
        .map(|id| UserId::from_str(id))
        .collect();

    let user_ids = user_ids.map_err(|e| {
        tracing::error!("Invalid user ID format: {:?}", e);
        StatusCode::BAD_REQUEST
    })?;

    // Create apply profile request
    let apply_request = ApplyPermissionProfileRequest {
        profile_id: profile_id.clone(),
        user_ids: user_ids.clone(),
        permission_overrides: None,
        reason: request.reason.clone(),
        merge_permissions: request.merge_permissions.unwrap_or(true),
        expires_at: request.expires_at,
        applied_by: auth_ctx.user_id.clone(),
    };

    // Process the profile assignment
    let mut successful_assignments = Vec::new();
    let mut failed_assignments = Vec::new();

    for user_id in &user_ids {
        match process_single_permission_profile_assignment(
            &app_state,
            &profile,
            user_id,
            &apply_request,
        ).await {
            Ok(result) => {
                successful_assignments.push(PermissionProfileAssignmentResult {
                    user_id: user_id.value().to_string(),
                    features_unlocked: result.features_unlocked,
                    permissions_added: result.permissions_added,
                    assignment_type: "admin_direct".to_string(),
                });
            }
            Err(error) => {
                failed_assignments.push(PermissionProfileAssignmentFailure {
                    user_id: user_id.value().to_string(),
                    error: error.to_string(),
                    error_code: "ASSIGNMENT_FAILED".to_string(),
                });
            }
        }
    }

    // Log the assignment activity
    tracing::info!(
        "Admin {} assigned profile {} to {} users (success: {}, failed: {})",
        auth_ctx.user_id.value(),
        request.profile_id,
        user_ids.len(),
        successful_assignments.len(),
        failed_assignments.len()
    );

    let total_assigned = successful_assignments.len() as u32;
    let total_failed = failed_assignments.len() as u32;
    
    let response = AdminPermissionProfileAssignResponse {
        profile_id: request.profile_id,
        successful_assignments,
        failed_assignments,
        total_assigned,
        total_failed,
        applied_at: Utc::now(),
    };

    Ok(Json(response))
}

/// GET /admin/permission-profiles/{profile_id} - Get permission profile details
pub async fn get_permission_profile_details_handler(
    State(app_state): State<AppState>,
    auth_ctx: AuthCtx,
    Path(profile_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Verify admin permissions
    verify_admin_permissions(&app_state, &auth_ctx.user_id).await?;

    let profile_id = PermissionProfileId::new(profile_id);
    
    let profile = app_state
        .permission_profile_repo
        .get(&profile_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get profile: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::error!("Permission profile not found: {}", profile_id.value());
            StatusCode::NOT_FOUND
        })?;

    let response = serde_json::json!({
        "id": profile.id().value(),
        "name": profile.name(),
        "description": profile.description(),
        "category": profile.category().to_string(),
        "target_tier": profile.target_tier().to_string(),
        "is_active": profile.is_active(),
        "default_permissions": profile.default_permissions().iter().map(|p| {
            serde_json::json!({
                "resource": p.resource(),
                "action": p.action(),
                "conditions": p.conditions().clone()
            })
        }).collect::<Vec<_>>(),
        "policy_attachments": profile.policy_attachments().iter().map(|p| p.value()).collect::<Vec<_>>(),
        "tags": profile.tags(),
        "metadata": {
            "prerequisites": profile.metadata().prerequisites,
            "warnings": profile.metadata().warnings,
            "use_cases": profile.metadata().use_cases,
            "max_assignments": profile.metadata().max_assignments,
            "requires_approval": profile.metadata().requires_approval,
            "auto_expire_days": profile.metadata().auto_expire_days,
            "custom_fields": profile.metadata().custom_fields
        },
        "created_at": profile.created_at(),
        "updated_at": profile.updated_at(),
        "created_by": profile.created_by().value(),
        "version": profile.version()
    });

    Ok(Json(response))
}

// Helper function to process individual permission profile assignment
async fn process_single_permission_profile_assignment(
    app_state: &AppState,
    profile: &crate::dom::entities::permission_profile::PermissionProfile,
    user_id: &UserId,
    apply_request: &ApplyPermissionProfileRequest,
) -> Result<SingleAssignmentResult, Box<dyn std::error::Error>> {
    // Verify user exists
    let _user = app_state
        .user_repo
        .get(user_id)
        .await?
        .ok_or("User not found")?;

    // For admin direct assignment, we bypass payment requirements
    // and directly apply the profile permissions
    
    let features_unlocked: Vec<String> = profile
        .default_permissions()
        .iter()
        .map(|p| format!("{}:{}", p.resource(), p.action()))
        .collect();

    let permissions_added: Vec<String> = profile
        .default_permissions()
        .iter()
        .map(|p| format!("{}", p.resource()))
        .collect();

    // TODO: Implement actual permission assignment to user
    // This would involve:
    // 1. Adding permissions to user's permission set
    // 2. Creating audit log entry
    // 3. Sending notification if requested
    // 4. Updating user's effective permissions

    tracing::info!(
        "Successfully assigned profile {} to user {} (admin: {})",
        profile.id().value(),
        user_id.value(),
        apply_request.applied_by().value()
    );

    Ok(SingleAssignmentResult {
        features_unlocked,
        permissions_added,
    })
}

/// DELETE /admin/users/{user_id} - Soft delete a user (admin only)
pub async fn soft_delete_user_handler(
    State(app_state): State<AppState>,
    auth_ctx: AuthCtx,
    Path(user_id): Path<String>,
    Json(req): Json<AdminSoftDeleteUserRequest>,
) -> Result<Json<SoftDeleteUserRes>, StatusCode> {
    // Verify admin permissions
    verify_admin_permissions(&app_state, &auth_ctx.user_id).await?;

    let user_id = UserId::from_string(user_id);
    let delete_req = SoftDeleteUserReq {
        usr_id: user_id.to_string(),
        reason: req.reason.clone(),
    };

    let result = app_state
        .user_mgmt_uc
        .soft_delete_user(delete_req, auth_ctx.user_id.clone())
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("Failed to soft delete user: {:?}", e);
            match e {
                UserUseCaseError::ValidationError(_) => StatusCode::BAD_REQUEST,
                UserUseCaseError::UserNotFound(_) => StatusCode::NOT_FOUND,
                UserUseCaseError::PermissionDenied => StatusCode::FORBIDDEN,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    tracing::info!(
        "Admin {} soft deleted user {} with reason: {:?}",
        auth_ctx.user_id.value(),
        user_id.value(),
        req.reason
    );

    Ok(result)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminSoftDeleteUserRequest {
    pub reason: Option<String>,
}

#[derive(Debug)]
struct SingleAssignmentResult {
    features_unlocked: Vec<String>,
    permissions_added: Vec<String>,
}