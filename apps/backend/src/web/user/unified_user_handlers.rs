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
    extract::{Path, Query, Request, State, Extension},
    Json,
};
use crate::web::middleware::OpenIDUserContext;
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

/// Access Overview Data
#[derive(Debug, Serialize, ToSchema)]
pub struct AccessOverviewData {
    pub current_tier: String,
    pub plans: Vec<AccessPlanData>,
    pub direct_permissions: Vec<DirectPermissionData>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AccessPlanData {
    pub name: String,
    pub description: Option<String>,
    pub expires_at: Option<String>,
    pub permissions: Vec<String>,
    pub source_type: String,
    /// When this plan was assigned to the user
    pub assigned_at: Option<String>,
    /// Who assigned this plan (admin wallet address or "system")
    pub assigned_by: Option<String>,
    /// Days remaining until expiration (None if permanent)
    pub days_remaining: Option<i32>,
    /// Whether renewal is available for this plan
    pub can_renew: bool,
    /// Price for renewal (formatted string, e.g., "29.99 USDT")
    pub renewal_price: Option<String>,
    /// Billing cycle (e.g., "monthly", "yearly")
    pub billing_cycle: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DirectPermissionData {
    pub permission: String,
    pub expires_at: Option<String>,
    /// Days remaining until expiration (None if permanent)
    pub days_remaining: Option<i32>,
    /// When this permission was granted
    pub granted_at: Option<String>,
    /// Who granted this permission
    pub granted_by: Option<String>,
    /// Source of the permission: 'manual' | 'system'
    pub source: String,
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

/// Get user access overview
/// Detailed breakdown of permissions by source (Plan/Plan/Direct)
#[utoipa::path(
    get,
    path = "/api/wallet/access-overview",
    responses(
        (status = 200, description = "Access overview retrieved", body = UnifiedApiResponse<AccessOverviewData>),
        (status = 401, description = "Authentication required"),
        (status = 500, description = "Internal server error")
    ),
    tag = "user",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_user_access_overview(
    State(_app_state): State<AppState>,
    request: Request,
) -> Result<Json<UnifiedApiResponse<AccessOverviewData>>, Json<UnifiedApiResponse<()>>> {
    // Extract user context
    let user_context = match require_user_context(&request) {
        Ok(context) => context,
        Err(_) => return Err(Json(UnifiedApiResponse::auth_error("Valid Bearer token required"))),
    };

    info!("Getting access overview for user: {}", user_context.wallet_address);

    let mut conn = match _app_state.db_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(Json(UnifiedApiResponse::error(500, "Database error", "Failed to connect to database")));
        }
    };

    // Define the row structure for the stored procedure
    #[derive(QueryableByName)]
    #[allow(dead_code)]
    struct PermissionDetailRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        pub permission_string: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        pub permission_id: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Text)]
        pub source_type: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        pub source_id: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        pub source_name: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        pub granted_at: chrono::DateTime<chrono::Utc>,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        pub is_permanent: bool,
    }

    // Call the stored procedure
    let rows = diesel::sql_query(
        r#"
        SELECT
            permission_string,
            permission_id::text,
            source_type,
            source_id::text,
            source_name,
            expires_at,
            granted_at,
            is_permanent
        FROM public.get_wallet_permissions_detailed_working($1)
        "#
    )
    .bind::<diesel::sql_types::Text, _>(&user_context.wallet_address.to_lowercase())
    .load::<PermissionDetailRow>(&mut conn)
    .await
    .map_err(|e| {
        error!("Database error fetching detailed permissions: {}", e);
        Json(UnifiedApiResponse::error(500, "Database error", "Failed to fetch permissions"))
    })?;

    // Process results
    use std::collections::HashMap;

    // Helper function to calculate days remaining
    fn calculate_days_remaining(expires_at: Option<chrono::DateTime<chrono::Utc>>) -> Option<i32> {
        expires_at.map(|exp| {
            let now = chrono::Utc::now();
            let duration = exp.signed_duration_since(now);
            duration.num_days() as i32
        })
    }

    // Plan by source_id (or source_name if ID missing) for plans
    let mut plans_map: HashMap<String, AccessPlanData> = HashMap::new();
    let mut direct_permissions: Vec<DirectPermissionData> = Vec::new();

    for row in rows {
        if row.source_type == "plan" {
            let key = row.source_id.clone().unwrap_or_else(|| row.source_name.clone().unwrap_or("Unknown Plan".to_string()));
            let days_remaining = calculate_days_remaining(row.expires_at);
            let is_plan = row.source_name.as_ref().map(|n| 
                n.contains("Plan") || n.contains("Starter") || n.contains("Pro") || n.contains("Enterprise")
            ).unwrap_or(false);
            
            let entry = plans_map.entry(key).or_insert(AccessPlanData {
                name: row.source_name.clone().unwrap_or("Unknown Plan".to_string()),
                description: None, // Will be fetched from plans table if available
                expires_at: row.expires_at.map(|d| d.to_rfc3339()),
                permissions: Vec::new(),
                source_type: "plan".to_string(),
                assigned_at: Some(row.granted_at.to_rfc3339()),
                assigned_by: None, // Would require additional query to get assigned_by from wallet_plan_assignments
                days_remaining,
                can_renew: is_plan && days_remaining.map(|d| d <= 30).unwrap_or(false),
                renewal_price: None, // Will be set if we can get plan price
                billing_cycle: None, // Will be set from plan data
            });
            
            // Don't duplicate permissions
            if !entry.permissions.contains(&row.permission_string) {
                entry.permissions.push(row.permission_string);
            }
        } else {
            // Direct permission
            let days_remaining = calculate_days_remaining(row.expires_at);
            direct_permissions.push(DirectPermissionData {
                permission: row.permission_string,
                expires_at: row.expires_at.map(|d| d.to_rfc3339()),
                days_remaining,
                granted_at: Some(row.granted_at.to_rfc3339()),
                granted_by: None, // Would require additional query
                source: if row.is_permanent { "system".to_string() } else { "manual".to_string() },
            });
        }
    }

    let mut plans: Vec<AccessPlanData> = plans_map.into_values().collect();

    // ALWAYS include the Free Plan as it represents default permissions
    // This is a constant baseline that all users have
    let free_plan = get_free_plan();
    
    // Check if Free Plan already exists in the list (shouldn't happen, but defensive)
    let has_free_plan = plans.iter().any(|p| p.name == "Free");
    if !has_free_plan {
        // Insert Free Plan at the beginning so it's always first
        plans.insert(0, free_plan);
    }

    // Determine current tier based on highest paid plan, or Free if only Free Plan exists
    let current_tier = if plans.len() == 1 && plans[0].name == "Free" {
        "Free User".to_string()
    } else {
        // If user has paid plans, use the first paid plan name
        plans.iter()
            .find(|p| p.name != "Free")
            .map(|p| p.name.clone())
            .unwrap_or("Standard User".to_string())
    };

    let overview_data = AccessOverviewData {
        current_tier,
        plans,
        direct_permissions,
    };

    // Success response
    Ok(Json(UnifiedApiResponse::success(overview_data)))
}

/// Helper to get the canonical Free Plan
/// This is ALWAYS returned to users as a baseline plan with default permissions
/// Centralized definition effectively acting as a constant
fn get_free_plan() -> AccessPlanData {
    use crate::core::constants::{FREE_PLAN_NAME, FREE_PLAN_DESCRIPTION, FREE_PLAN_DEFAULT_PERMISSIONS};
    
    AccessPlanData {
        name: FREE_PLAN_NAME.to_string(),
        description: Some(FREE_PLAN_DESCRIPTION.to_string()),
        source_type: "plan".to_string(), // Frontend expects 'plan' for badge logic
        permissions: FREE_PLAN_DEFAULT_PERMISSIONS.iter().map(|s| s.to_string()).collect(),
        expires_at: None,
        assigned_at: None, 
        assigned_by: Some("system".to_string()),
        days_remaining: None,
        can_renew: false,
        renewal_price: None,
        billing_cycle: None,
    }
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
    Extension(user_context): Extension<OpenIDUserContext>,
    Json(update_request): Json<UpdateUserPreferencesRequest>,
) -> Result<Json<UnifiedApiResponse<Value>>, Json<UnifiedApiResponse<()>>> {
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
    .bind::<diesel::sql_types::Text, _>(&wallet_address.to_lowercase())
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
/// Notification preferences structure
#[derive(Debug, Serialize, Deserialize, ToSchema, Default, Clone)]
pub struct NotificationPreferences {
    pub trading: bool,
    pub security: bool,
    pub account: bool,
    pub system: bool,
    pub marketing: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NotificationPreferencesResponse {
    pub preferences: NotificationPreferences,
}

/// Get user notification preferences
#[utoipa::path(
    get,
    path = "/api/notifications/preferences",
    responses(
        (status = 200, description = "Preferences retrieved", body = UnifiedApiResponse<NotificationPreferencesResponse>),
        (status = 401, description = "Authentication required"),
        (status = 500, description = "Internal server error")
    ),
    tag = "user",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_user_notification_preferences(
    State(_app_state): State<AppState>,
    Extension(user_context): Extension<OpenIDUserContext>,
) -> Result<Json<UnifiedApiResponse<NotificationPreferencesResponse>>, Json<UnifiedApiResponse<()>>> {
    // User context already validated by middleware

    let mut conn = match _app_state.db_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(Json(UnifiedApiResponse::error(500, "Database error", "Failed to connect to database")));
        }
    };

    #[derive(QueryableByName)]
    struct MetadataRow {
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
        wallet_metadata: Option<serde_json::Value>,
    }

    let result = diesel::sql_query(
        "SELECT wallet_metadata FROM wallet_users WHERE wallet_address = $1"
    )
    .bind::<diesel::sql_types::Text, _>(&user_context.wallet_address)
    .get_result::<MetadataRow>(&mut conn)
    .await
    .optional();

    let preferences = match result {
        Ok(Some(row)) => {
            if let Some(metadata) = row.wallet_metadata {
                if let Some(prefs) = metadata.get("preferences").and_then(|p| p.get("notification_preferences")) {
                    serde_json::from_value(prefs.clone()).unwrap_or_default()
                } else {
                    NotificationPreferences::default()
                }
            } else {
                NotificationPreferences::default()
            }
        },
        _ => NotificationPreferences::default(),
    };

    Ok(Json(UnifiedApiResponse::success(NotificationPreferencesResponse {
        preferences,
    })))
}

/// Update user notification preferences
#[utoipa::path(
    post,
    path = "/api/notifications/preferences",
    request_body = NotificationPreferences,
    responses(
        (status = 200, description = "Preferences updated", body = UnifiedApiResponse<NotificationPreferencesResponse>),
        (status = 401, description = "Authentication required"),
        (status = 500, description = "Internal server error")
    ),
    tag = "user",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_user_notification_preferences(
    State(_app_state): State<AppState>,
    Extension(user_context): Extension<OpenIDUserContext>,
    Json(preferences): Json<NotificationPreferences>,
) -> Result<Json<UnifiedApiResponse<NotificationPreferencesResponse>>, Json<UnifiedApiResponse<()>>> {
    // User context already validated by middleware

    let mut conn = match _app_state.db_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(Json(UnifiedApiResponse::error(500, "Database error", "Failed to connect to database")));
        }
    };

    // Construct the partial JSON update
    // We want to update wallet_metadata.preferences.notification_preferences
    // simpler to first get current metadata, merge, and update, BUT concurrency...
    // Postgres jsonb_set is better.
    // logical path: wallet_metadata['preferences']['notification_preferences'] = new_prefs

    // We can do a deep merge or just ensure the path exists.
    // For simplicity, let's use a specialized query to ensure structure.
    
    // First ensure 'preferences' object exists
    let _ = diesel::sql_query(
        r#"
        UPDATE wallet_users 
        SET wallet_metadata = jsonb_set(
            COALESCE(wallet_metadata, '{}'::jsonb), 
            '{preferences}', 
            COALESCE(wallet_metadata->'preferences', '{}'::jsonb), 
            true
        )
        WHERE wallet_address = $1
        "#
    )
    .bind::<diesel::sql_types::Text, _>(&user_context.wallet_address)
    .execute(&mut conn)
    .await;

    // Then update notification_preferences
    let update_result = diesel::sql_query(
        r#"
        UPDATE wallet_users
        SET wallet_metadata = jsonb_set(
            wallet_metadata,
            '{preferences,notification_preferences}',
            $2::jsonb,
            true
        ),
        updated_at = NOW()
        WHERE wallet_address = $1
        "#
    )
    .bind::<diesel::sql_types::Text, _>(&user_context.wallet_address)
    .bind::<diesel::sql_types::Jsonb, _>(serde_json::to_value(&preferences).unwrap())
    .execute(&mut conn)
    .await;

    if update_result.is_err() {
        return Err(Json(UnifiedApiResponse::error(500, "Database error", "Failed to update preferences")));
    }

    Ok(Json(UnifiedApiResponse::success(NotificationPreferencesResponse {
        preferences,
    })))
}
