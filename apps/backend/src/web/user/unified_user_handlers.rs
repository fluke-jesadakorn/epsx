// ============================================================================
// UNIFIED USER HANDLERS WITH OPENID BEARER AUTH
// Example implementation of unified response format + OpenID authentication
// ============================================================================

/**
 * CORE PRINCIPLES:
 * - Standard OpenID Bearer token validation
 * - Unified API response format for all endpoints
 * - Backend makes ALL authorization decisions
 * - Frontend displays exactly what backend tells it to display
 * - No permission logic in handlers - middleware handles auth
 */

use axum::{
    extract::{Path, Query, Request, State},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, info};
use utoipa::ToSchema;

use crate::{
    web::{
        auth::routes::AppState,
        middleware::{require_user_context, check_user_permission},
        responses::{UnifiedApiResponse, ResponseMeta, PermissionContext},
    },
};

/// User profile information
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserProfile {
    pub wallet_address: String,
    pub tier_level: String,
    pub permissions: Vec<String>,
    pub auth_method: String,
    pub created_at: String,
    pub last_login: String,
}

/// User permissions query
#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct UserPermissionsQuery {
    pub include_expired: Option<bool>,
}

/// Get current user profile
/// Uses OpenID Bearer token for authentication
#[utoipa::path(
    get,
    path = "/api/v1/user/profile",
    responses(
        (status = 200, description = "User profile retrieved successfully", body = UnifiedApiResponse<UserProfile>),
        (status = 401, description = "Authentication required"),
        (status = 500, description = "Internal server error")
    ),
    tag = "user",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_current_user_profile(
    State(_app_state): State<AppState>,
    request: Request,
) -> Result<Json<UnifiedApiResponse<UserProfile>>, Json<UnifiedApiResponse<()>>> {
    // Extract user context from validated Bearer token (handled by middleware)
    let user_context = match require_user_context(&request) {
        Ok(context) => context,
        Err(_auth_error) => return Err(Json(UnifiedApiResponse::auth_error("Valid Bearer token required"))),
    };

    info!(
        "Getting user profile for wallet: {}",
        user_context.wallet_address
    );

    // Create user profile from authenticated context
    let user_profile = UserProfile {
        wallet_address: user_context.wallet_address.clone(),
        tier_level: user_context.tier_level.clone(),
        permissions: user_context.permissions.clone(),
        auth_method: user_context.auth_method.clone(),
        created_at: chrono::Utc::now().to_rfc3339(), // TODO: Get from database
        last_login: chrono::Utc::now().to_rfc3339(),  // TODO: Get from database
    };

    // Create permission context for frontend
    let permission_context = PermissionContext::from_user_tier(
        &user_context.tier_level,
        &user_context.permissions,
    );

    // Create response metadata with permission context
    let meta = ResponseMeta::default().with_permissions(permission_context);

    info!(
        "User profile retrieved successfully for: {}",
        user_context.wallet_address
    );

    Ok(Json(UnifiedApiResponse::success_with_meta(
        user_profile,
        meta,
    )))
}

/// Get user permissions
/// Requires specific permission to view permissions
#[utoipa::path(
    get,
    path = "/api/v1/user/permissions",
    params(UserPermissionsQuery),
    responses(
        (status = 200, description = "User permissions retrieved", body = UnifiedApiResponse<Vec<String>>),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Permission denied"),
        (status = 500, description = "Internal server error")
    ),
    tag = "user",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_user_permissions(
    State(_app_state): State<AppState>,
    Query(query): Query<UserPermissionsQuery>,
    request: Request,
) -> Result<Json<UnifiedApiResponse<Vec<String>>>, Json<UnifiedApiResponse<()>>> {
    // Extract user context from validated Bearer token
    let user_context = match require_user_context(&request) {
        Ok(context) => context,
        Err(_) => return Err(Json(UnifiedApiResponse::auth_error("Valid Bearer token required"))),
    };

    // Check if user can view their own permissions
    // This is an example of backend permission checking
    if !check_user_permission(user_context, "epsx:user:view_permissions") {
        error!(
            "Permission denied for user {} to view permissions",
            user_context.wallet_address
        );
        return Err(Json(UnifiedApiResponse::permission_error("epsx:user:view_permissions")));
    }

    info!(
        "Getting permissions for user: {}",
        user_context.wallet_address
    );

    // Filter permissions based on query parameters
    let permissions = if query.include_expired.unwrap_or(false) {
        // TODO: Include expired permissions from database
        user_context.permissions.clone()
    } else {
        // Return only active permissions
        user_context.permissions.clone()
    };

    // Create permission context
    let permission_context = PermissionContext::from_user_tier(
        &user_context.tier_level,
        &user_context.permissions,
    );

    let meta = ResponseMeta::default().with_permissions(permission_context);

    info!(
        "Permissions retrieved for user: {} (count: {})",
        user_context.wallet_address,
        permissions.len()
    );

    Ok(Json(UnifiedApiResponse::success_with_meta(
        permissions,
        meta,
    )))
}

/// Update user preferences
/// Example of POST endpoint with OpenID auth and unified response
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateUserPreferencesRequest {
    pub notification_preferences: Option<Value>,
    pub display_preferences: Option<Value>,
}

#[utoipa::path(
    post,
    path = "/api/v1/user/preferences",
    request_body = UpdateUserPreferencesRequest,
    responses(
        (status = 200, description = "Preferences updated successfully", body = UnifiedApiResponse<Value>),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Authentication required"),
        (status = 500, description = "Internal server error")
    ),
    tag = "user",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_user_preferences(
    State(_app_state): State<AppState>,
    request: Request,
    Json(update_request): Json<UpdateUserPreferencesRequest>,
) -> Result<Json<UnifiedApiResponse<Value>>, Json<UnifiedApiResponse<()>>> {
    // Extract user context from validated Bearer token
    let user_context = match require_user_context(&request) {
        Ok(context) => context,
        Err(_) => return Err(Json(UnifiedApiResponse::auth_error("Valid Bearer token required"))),
    };

    info!(
        "Updating preferences for user: {}",
        user_context.wallet_address
    );

    // Validate request
    if update_request.notification_preferences.is_none() && update_request.display_preferences.is_none() {
        return Err(Json(UnifiedApiResponse::validation_error(json!({
            "message": "At least one preference type must be provided"
        }))));
    }

    // TODO: Update preferences in database
    // For now, return success with updated preferences
    let updated_preferences = json!({
        "wallet_address": user_context.wallet_address,
        "notification_preferences": update_request.notification_preferences,
        "display_preferences": update_request.display_preferences,
        "updated_at": chrono::Utc::now().to_rfc3339()
    });

    info!(
        "Preferences updated successfully for user: {}",
        user_context.wallet_address
    );

    Ok(Json(UnifiedApiResponse::success(updated_preferences)))
}

/// Admin endpoint - get user by wallet address
/// Example of admin-only endpoint with permission checking
#[utoipa::path(
    get,
    path = "/api/v1/admin/users/{wallet_address}",
    params(
        ("wallet_address" = String, Path, description = "Wallet address of the user")
    ),
    responses(
        (status = 200, description = "User found", body = UnifiedApiResponse<UserProfile>),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Admin access required"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "admin",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_user_by_wallet_address(
    State(_app_state): State<AppState>,
    Path(wallet_address): Path<String>,
    request: Request,
) -> Result<Json<UnifiedApiResponse<UserProfile>>, Json<UnifiedApiResponse<()>>> {
    // Extract user context from validated Bearer token
    let user_context = match require_user_context(&request) {
        Ok(context) => context,
        Err(_) => return Err(Json(UnifiedApiResponse::auth_error("Valid Bearer token required"))),
    };

    // Check admin permission
    if !check_user_permission(user_context, "admin:users:view") {
        error!(
            "Permission denied for user {} to view user data",
            user_context.wallet_address
        );
        return Err(Json(UnifiedApiResponse::permission_error("admin:users:view")));
    }

    info!(
        "Admin {} looking up user: {}",
        user_context.wallet_address, wallet_address
    );

    // TODO: Query user from database by wallet address
    // For now, return mock data
    let user_profile = UserProfile {
        wallet_address: wallet_address.clone(),
        tier_level: "premium".to_string(),
        permissions: vec![
            "epsx:analytics:read".to_string(),
            "epsx:export:csv".to_string(),
        ],
        auth_method: "web3_siwe".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        last_login: chrono::Utc::now().to_rfc3339(),
    };

    // Create admin permission context
    let permission_context = PermissionContext::from_user_tier(
        &user_context.tier_level,
        &user_context.permissions,
    );

    let meta = ResponseMeta::default().with_permissions(permission_context);

    info!(
        "User {} found by admin {}",
        wallet_address, user_context.wallet_address
    );

    Ok(Json(UnifiedApiResponse::success_with_meta(
        user_profile,
        meta,
    )))
}