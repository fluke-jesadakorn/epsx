// ============================================================================
// UNIFIED USER HANDLERS WITH OPENID BEARER AUTH
// Example implementation of unified response format + OpenID authentication
// ============================================================================

//! CORE PRINCIPLES:
//! - Standard OpenID Bearer token validation
//! - Unified API response format for all endpoints
//! - Backend makes ALL authorization decisions
//! - Frontend displays exactly what backend tells it to display
//! - No permission logic in handlers - middleware handles auth

use axum::{
    extract::{Path, Query, Request, State},
    Json,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, info};
use utoipa::ToSchema;

use crate::{
    web::{
        auth::AppState,
        middleware::{require_user_context, check_user_permission},
        responses::{UnifiedApiResponse, ResponseMeta, PermissionContext},
    },
};

/// User profile information
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserProfile {
    pub wallet_address: String,
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
    path = "/api/wallet/profile",
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

    // Query user data from database
    let mut conn = match _app_state.db_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            let now = chrono::Utc::now().to_rfc3339();
            return Ok(Json(UnifiedApiResponse::success_with_meta(
                UserProfile {
                    wallet_address: user_context.wallet_address.clone(),
                    permissions: user_context.permissions.clone(),
                    auth_method: user_context.auth_method.clone(),
                    created_at: now.clone(),
                    last_login: now,
                },
                ResponseMeta::default().with_permissions(PermissionContext::from_permissions(&user_context.permissions)),
            )));
        }
    };

    #[derive(QueryableByName)]
    struct UserRow {
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        created_at: Option<chrono::DateTime<chrono::Utc>>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        last_auth_at: Option<chrono::DateTime<chrono::Utc>>,
    }

    let (created_at, last_login) = match diesel::sql_query(
        "SELECT created_at, last_auth_at FROM wallet_users WHERE wallet_address = $1"
    )
    .bind::<diesel::sql_types::Text, _>(&user_context.wallet_address)
    .get_result::<UserRow>(&mut conn)
    .await
    .optional()
    {
        Ok(Some(row)) => (
            row.created_at.unwrap_or_else(chrono::Utc::now).to_rfc3339(),
            row.last_auth_at.unwrap_or_else(chrono::Utc::now).to_rfc3339(),
        ),
        _ => {
            // Fallback to current time if user not found in database
            let now = chrono::Utc::now().to_rfc3339();
            (now.clone(), now)
        }
    };

    // Create user profile from authenticated context with database data
    let user_profile = UserProfile {
        wallet_address: user_context.wallet_address.clone(),
        permissions: user_context.permissions.clone(),
        auth_method: user_context.auth_method.clone(),
        created_at,
        last_login,
    };

    // Create permission context for frontend from permissions only
    let permission_context = PermissionContext::from_permissions(&user_context.permissions);

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
    path = "/api/wallet/permissions",
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
        // Include all permissions (active and expired) from database
        let mut conn = match _app_state.db_pool.get().await {
            Ok(c) => c,
            Err(_) => {
                return Ok(Json(UnifiedApiResponse::success_with_meta(
                    user_context.permissions.clone(),
                    ResponseMeta::default().with_permissions(PermissionContext::from_permissions(&user_context.permissions)),
                )));
            }
        };

        #[derive(QueryableByName)]
        struct PermissionRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            permission_string: String,
        }

        diesel::sql_query(
            r#"
            SELECT DISTINCT permission_string
            FROM user_effective_permissions
            WHERE wallet_address = $1
            ORDER BY permission_string
            "#
        )
        .bind::<diesel::sql_types::Text, _>(&user_context.wallet_address)
        .load::<PermissionRow>(&mut conn)
        .await
        .map(|rows| rows.into_iter().map(|r| r.permission_string).collect())
        .unwrap_or_else(|_| user_context.permissions.clone())
    } else {
        // Return only active (non-expired) permissions
        user_context.permissions.clone()
    };

    // Create permission context from permissions only
    let permission_context = PermissionContext::from_permissions(&user_context.permissions);

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
    path = "/api/wallet/preferences",
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

    // Build preferences JSON object
    let mut preferences_json = json!({});
    if let Some(notif_prefs) = &update_request.notification_preferences {
        preferences_json["notification_preferences"] = notif_prefs.clone();
    }
    if let Some(display_prefs) = &update_request.display_preferences {
        preferences_json["display_preferences"] = display_prefs.clone();
    }

    // Update preferences in database (store in wallet_metadata.preferences)
    let mut conn = match _app_state.db_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(Json(UnifiedApiResponse::error(
                500,
                "Database error",
                "Failed to get database connection",
            )));
        }
    };

    let update_result = diesel::sql_query(
        r#"
        UPDATE wallet_users
        SET wallet_metadata = jsonb_set(
            COALESCE(wallet_metadata, '{}'::jsonb),
            '{preferences}',
            $2::jsonb,
            true
        ),
        updated_at = NOW()
        WHERE wallet_address = $1
        "#
    )
    .bind::<diesel::sql_types::Text, _>(&user_context.wallet_address)
    .bind::<diesel::sql_types::Jsonb, _>(&preferences_json)
    .execute(&mut conn)
    .await;

    // Check if update succeeded
    if update_result.is_err() {
        error!("Failed to update preferences for user: {}", user_context.wallet_address);
        return Err(Json(UnifiedApiResponse::error(
            500,
            "Database error",
            "Failed to update preferences",
        )));
    }

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

/// Admin endpoint - get wallet by wallet address
/// Example of admin-only endpoint with permission checking
#[utoipa::path(
    get,
    path = "/api/admin/wallets/{wallet_address}",
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

    // Query user from database by wallet address
    let mut conn = match _app_state.db_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(Json(UnifiedApiResponse::error(500, "Database error", "Failed to get database connection")));
        }
    };

    #[derive(QueryableByName)]
    struct WalletUserRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        wallet_address: String,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_active: bool,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        created_at: Option<chrono::DateTime<chrono::Utc>>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        last_auth_at: Option<chrono::DateTime<chrono::Utc>>,
    }

    let user_data = diesel::sql_query(
        "SELECT wallet_address, is_active, created_at, last_auth_at FROM wallet_users WHERE wallet_address = $1"
    )
    .bind::<diesel::sql_types::Text, _>(&wallet_address)
    .get_result::<WalletUserRow>(&mut conn)
    .await
    .optional();

    // Check if user exists
    let (wallet_addr, _is_active, created_at, last_auth) = match user_data {
        Ok(Some(data)) => (data.wallet_address, data.is_active, data.created_at, data.last_auth_at),
        Ok(None) => {
            error!("User not found: {}", wallet_address);
            return Err(Json(UnifiedApiResponse::not_found(&format!("User {} not found", wallet_address))));
        },
        Err(e) => {
            error!("Database error looking up user {}: {}", wallet_address, e);
            return Err(Json(UnifiedApiResponse::error(500, "Database error", "Failed to query user")));
        }
    };

    // Get user permissions from user_effective_permissions read model
    #[derive(QueryableByName)]
    struct UserPermissionRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        permission_string: String,
    }

    let user_permissions = diesel::sql_query(
        r#"
        SELECT DISTINCT permission_string
        FROM user_effective_permissions
        WHERE wallet_address = $1
          AND (expires_at IS NULL OR expires_at > NOW())
        "#
    )
    .bind::<diesel::sql_types::Text, _>(&wallet_addr)
    .load::<UserPermissionRow>(&mut conn)
    .await
    .map(|rows| rows.into_iter().map(|r| r.permission_string).collect())
    .unwrap_or_default();

    let user_profile = UserProfile {
        wallet_address: wallet_addr,
        permissions: user_permissions,
        auth_method: "web3_siwe".to_string(),
        created_at: created_at.unwrap_or_else(chrono::Utc::now).to_rfc3339(),
        last_login: last_auth.unwrap_or_else(chrono::Utc::now).to_rfc3339(),
    };

    // Create admin permission context from permissions only
    let permission_context = PermissionContext::from_permissions(&user_context.permissions);

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