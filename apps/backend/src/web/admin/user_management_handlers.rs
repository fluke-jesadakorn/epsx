// ============================================================================
// ADMIN USER MANAGEMENT HANDLERS
// Consolidated user operations for admin interface
// ============================================================================

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Json as RequestJson,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::{error, info};
use sqlx::Row;

use crate::web::auth::AppState;
use crate::web::admin::responses::{AdminApiResponse, AdminMetadata, PaginationInfo};
use crate::auth::{PermissionState, HandlerPermissionExt, ValidationContext, PermissionValidator};

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct UserListQuery {
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub search: Option<String>,
    pub tier: Option<String>,
    pub status: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserSummaryResponse {
    pub wallet_address: String,
    pub tier_level: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_auth_at: Option<DateTime<Utc>>,
    pub permissions_count: i32,
    pub groups_count: i32,
    pub last_activity: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct UserDetailResponse {
    pub wallet_address: String,
    pub tier_level: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_auth_at: Option<DateTime<Utc>>,
    pub permissions: Vec<UserPermission>,
    pub groups: Vec<UserGroup>,
    pub activity_summary: UserActivitySummary,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct UserPermission {
    pub permission: String,
    pub source: String,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct UserGroup {
    pub group_id: String,
    pub group_name: String,
    pub group_type: String,
    pub assigned_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct UserActivitySummary {
    pub total_logins: i32,
    pub last_30_days_logins: i32,
    pub total_permissions: i32,
    pub active_permissions: i32,
    pub expired_permissions: i32,
    pub groups_count: i32,
}

#[derive(Debug, Serialize)]
pub struct UserListResponse {
    pub users: Vec<UserSummaryResponse>,
    pub total: i32,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub tier_level: Option<String>,
    pub is_active: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct UserStatsResponse {
    pub total_users: i32,
    pub active_users: i32,
    pub inactive_users: i32,
    pub users_by_tier: serde_json::Value,
    pub new_users_30_days: i32,
    pub active_users_30_days: i32,
    pub growth_rate: f64,
}

// ============================================================================
// USER MANAGEMENT HANDLERS
// ============================================================================

/**
 * List all users with filtering and pagination
 * GET /admin/users
 */
pub async fn list_users_handler(
    Query(query): Query<UserListQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<AdminApiResponse<UserListResponse>>, StatusCode> {
    info!("🔍 Admin: Listing users with filters: {:?}", query);

    let db_pool = app_state.db_pool.as_ref();
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(50);
    let offset = (page - 1) * limit;

    // Build dynamic query
    let mut conditions = Vec::new();
    let mut params: Vec<&str> = Vec::new();

    if let Some(search) = &query.search {
        conditions.push("wu.wallet_address ILIKE $1");
        params.push(search);
    }

    if let Some(tier) = &query.tier {
        conditions.push("wu.tier_level = $2");
        params.push(tier);
    }

    if let Some(status) = &query.status {
        let is_active = status == "active";
        conditions.push("wu.is_active = $3");
        params.push(if is_active { "true" } else { "false" });
    }

    let where_clause = if conditions.is_empty() {
        "".to_string()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let sort_by = query.sort_by.as_deref().unwrap_or("created_at");
    let sort_order = query.sort_order.as_deref().unwrap_or("DESC");

    // Query users with counts
    let query_str = format!(
        "SELECT 
            wu.wallet_address,
            wu.tier_level,
            wu.is_active,
            wu.created_at,
            wu.last_auth_at,
            COALESCE(perms.count, 0) as permissions_count,
            0 as groups_count,
            wu.created_at as last_activity
         FROM wallet_users wu
         LEFT JOIN (
             SELECT wallet_address, COUNT(*) as count 
             FROM wallet_group_memberships 
             WHERE is_active = true 
             GROUP BY wallet_address
         ) perms ON wu.wallet_address = perms.wallet_address
         {}
         ORDER BY wu.{} {}
         LIMIT {} OFFSET {}",
        where_clause, sort_by, sort_order, limit, offset
    );

    let users = match sqlx::query(&query_str)
        .fetch_all(db_pool)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            error!("❌ Admin: Failed to fetch users: {}", e);
            return Ok(Json(AdminApiResponse::server_error()));
        }
    };

    // Get total count
    let count_query = format!(
        "SELECT COUNT(*) as total FROM wallet_users wu {}",
        where_clause
    );

    let total = match sqlx::query(&count_query)
        .fetch_one(db_pool)
        .await
    {
        Ok(row) => row.get::<i64, _>("total") as i32,
        Err(e) => {
            error!("❌ Admin: Failed to count users: {}", e);
            return Ok(Json(AdminApiResponse::server_error()));
        }
    };

    // Convert to response format
    let user_responses: Vec<UserSummaryResponse> = users
        .into_iter()
        .map(|row| UserSummaryResponse {
            wallet_address: row.get("wallet_address"),
            tier_level: row.get("tier_level"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            last_auth_at: row.get("last_auth_at"),
            permissions_count: row.get::<i32, _>("permissions_count"),
            groups_count: row.get::<i32, _>("groups_count"),
            last_activity: row.get("last_activity"),
            metadata: serde_json::json!({}),
        })
        .collect();

    let total_pages = (total as f64 / limit as f64).ceil() as i32;
    let pagination = PaginationInfo {
        page,
        limit,
        total,
        total_pages,
        has_next_page: page < total_pages,
        has_previous_page: page > 1,
    };

    let response = UserListResponse {
        users: user_responses,
        total,
        pagination: pagination.clone(),
    };

    let metadata = AdminMetadata::list_operation("list_users", pagination);

    info!("✅ Admin: Successfully listed {} users", response.users.len());
    Ok(Json(AdminApiResponse::success_with_meta(
        response,
        "Users retrieved successfully",
        metadata,
    )))
}

/**
 * Get detailed user information
 * GET /admin/users/:wallet_address
 */
pub async fn get_user_handler(
    Path(wallet_address): Path<String>,
    State(app_state): State<AppState>,
) -> Result<Json<AdminApiResponse<UserDetailResponse>>, StatusCode> {
    info!("🔍 Admin: Getting user details for: {}", wallet_address);

    let db_pool = app_state.db_pool.as_ref();

    // Get user basic info
    let user = match sqlx::query!(
        "SELECT wallet_address, tier_level, is_active, created_at, last_auth_at 
         FROM wallet_users 
         WHERE wallet_address = $1",
        wallet_address
    )
    .fetch_optional(db_pool)
    .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Ok(Json(AdminApiResponse::not_found("User")));
        }
        Err(e) => {
            error!("❌ Admin: Failed to fetch user: {}", e);
            return Ok(Json(AdminApiResponse::server_error()));
        }
    };

    // Get user permissions
    let permissions = match sqlx::query!(
        "SELECT 'tier_based' as permission, 'system' as source, created_at as granted_at, 
                NULL::timestamptz as expires_at, is_active
         FROM wallet_users 
         WHERE wallet_address = $1",
        wallet_address
    )
    .fetch_all(db_pool)
    .await
    {
        Ok(perms) => perms
            .into_iter()
            .map(|p| UserPermission {
                permission: format!("epsx:{}:view", user.tier_level),
                source: p.source.unwrap_or("system".to_string()),
                granted_at: p.granted_at,
                expires_at: p.expires_at,
                is_active: p.is_active,
            })
            .collect(),
        Err(_) => Vec::new(),
    };

    // Get user groups (placeholder)
    let groups = Vec::new();

    // Calculate activity summary
    let activity_summary = UserActivitySummary {
        total_logins: 1, // TODO: Implement proper login tracking
        last_30_days_logins: if user.last_auth_at.is_some() { 1 } else { 0 },
        total_permissions: permissions.len() as i32,
        active_permissions: permissions.iter().filter(|p| p.is_active).count() as i32,
        expired_permissions: 0,
        groups_count: groups.len() as i32,
    };

    let response = UserDetailResponse {
        wallet_address: user.wallet_address,
        tier_level: user.tier_level,
        is_active: user.is_active,
        created_at: user.created_at,
        last_auth_at: user.last_auth_at,
        permissions,
        groups,
        activity_summary,
        metadata: serde_json::json!({}),
    };

    let metadata = AdminMetadata::crud_operation("get_user", Some("admin".to_string()));

    info!("✅ Admin: Successfully retrieved user details for: {}", wallet_address);
    Ok(Json(AdminApiResponse::success_with_meta(
        response,
        "User details retrieved successfully",
        metadata,
    )))
}

/**
 * Update user information
 * PUT /admin/users/:wallet_address
 */
pub async fn update_user_handler(
    Path(wallet_address): Path<String>,
    State(app_state): State<AppState>,
    RequestJson(request): RequestJson<UpdateUserRequest>,
) -> Result<Json<AdminApiResponse<UserDetailResponse>>, StatusCode> {
    info!("✏️ Admin: Updating user: {}", wallet_address);

    let db_pool = app_state.db_pool.as_ref();

    // Validate that user exists
    let user_exists = match sqlx::query!(
        "SELECT wallet_address FROM wallet_users WHERE wallet_address = $1",
        wallet_address
    )
    .fetch_optional(db_pool)
    .await
    {
        Ok(Some(_)) => true,
        Ok(None) => {
            return Ok(Json(AdminApiResponse::not_found("User")));
        }
        Err(e) => {
            error!("❌ Admin: Failed to check user existence: {}", e);
            return Ok(Json(AdminApiResponse::server_error()));
        }
    };

    if !user_exists {
        return Ok(Json(AdminApiResponse::not_found("User")));
    }

    // Build update query dynamically
    let mut updates = Vec::new();
    let mut params: Vec<String> = Vec::new();

    if let Some(tier) = &request.tier_level {
        updates.push(format!("tier_level = ${}", params.len() + 2));
        params.push(tier.clone());
    }

    if let Some(active) = request.is_active {
        updates.push(format!("is_active = ${}", params.len() + 2));
        params.push(active.to_string());
    }

    if updates.is_empty() {
        return Ok(Json(AdminApiResponse::validation_error("No updates provided")));
    }

    updates.push("updated_at = NOW()".to_string());

    let update_query = format!(
        "UPDATE wallet_users SET {} WHERE wallet_address = $1",
        updates.join(", ")
    );

    match sqlx::query(&update_query)
        .bind(&wallet_address)
        .execute(db_pool)
        .await
    {
        Ok(_) => {},
        Err(e) => {
            error!("❌ Admin: Failed to update user: {}", e);
            return Ok(Json(AdminApiResponse::server_error()));
        }
    }

    // Return updated user details
    get_user_handler(Path(wallet_address.clone()), State(app_state)).await
}

/**
 * Get user statistics
 * GET /admin/users/stats
 */
pub async fn get_user_stats_handler(
    State(app_state): State<AppState>,
) -> Result<Json<AdminApiResponse<UserStatsResponse>>, StatusCode> {
    info!("📊 Admin: Getting user statistics");

    let db_pool = app_state.db_pool.as_ref();

    // Get basic user counts
    let stats = match sqlx::query!(
        "SELECT 
            COUNT(*) as total_users,
            COUNT(*) FILTER (WHERE is_active = true) as active_users,
            COUNT(*) FILTER (WHERE is_active = false) as inactive_users,
            COUNT(*) FILTER (WHERE created_at >= NOW() - INTERVAL '30 days') as new_users_30_days
         FROM wallet_users"
    )
    .fetch_one(db_pool)
    .await
    {
        Ok(stats) => stats,
        Err(e) => {
            error!("❌ Admin: Failed to fetch user stats: {}", e);
            return Ok(Json(AdminApiResponse::server_error()));
        }
    };

    // Get tier distribution
    let tier_distribution = match sqlx::query!(
        "SELECT tier_level, COUNT(*) as count 
         FROM wallet_users 
         GROUP BY tier_level 
         ORDER BY count DESC"
    )
    .fetch_all(db_pool)
    .await
    {
        Ok(tiers) => {
            let mut tier_map = serde_json::Map::new();
            for tier in tiers {
                tier_map.insert(tier.tier_level, serde_json::Value::from(tier.count));
            }
            serde_json::Value::Object(tier_map)
        }
        Err(_) => serde_json::json!({}),
    };

    // Calculate growth rate (simplified)
    let growth_rate = if stats.total_users.unwrap_or(0) > 0 {
        (stats.new_users_30_days.unwrap_or(0) as f64 / stats.total_users.unwrap_or(1) as f64) * 100.0
    } else {
        0.0
    };

    let response = UserStatsResponse {
        total_users: stats.total_users.unwrap_or(0) as i32,
        active_users: stats.active_users.unwrap_or(0) as i32,
        inactive_users: stats.inactive_users.unwrap_or(0) as i32,
        users_by_tier: tier_distribution,
        new_users_30_days: stats.new_users_30_days.unwrap_or(0) as i32,
        active_users_30_days: stats.active_users.unwrap_or(0) as i32,
        growth_rate,
    };

    let metadata = AdminMetadata::crud_operation("get_user_stats", Some("admin".to_string()));

    info!("✅ Admin: Successfully retrieved user statistics");
    Ok(Json(AdminApiResponse::success_with_meta(
        response,
        "User statistics retrieved successfully",
        metadata,
    )))
}

// ============================================================================
// CENTRALIZED PERMISSION SYSTEM INTEGRATION EXAMPLES
// ============================================================================

/**
 * Example handler using centralized permission validation (v2.0)
 * Demonstrates how to integrate the new PermissionState system
 * GET /admin/users/centralized-test
 */
pub async fn list_users_with_centralized_validation(
    Query(query): Query<UserListQuery>,
    State(app_state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<AdminApiResponse<UserListResponse>>, StatusCode> {
    info!("📋 Admin: Listing users with centralized permission validation");

    // Extract admin wallet address from headers
    let admin_wallet = headers.get("x-wallet-address")
        .and_then(|h| h.to_str().ok())
        .map(String::from);

    // Create permission services (in a real implementation, these would be injected)
    let permission_authority = std::sync::Arc::new(
        crate::auth::create_permission_authority(app_state.db_pool.as_ref().clone())
    );
    let permission_registry = std::sync::Arc::new(
        crate::auth::create_permission_registry(app_state.db_pool.as_ref().clone())
    );
    let permission_state = std::sync::Arc::new(
        PermissionState::new(permission_authority, permission_registry)
    );

    // Validate admin permissions using centralized system
    if let Some(admin_addr) = &admin_wallet {
        match permission_state.require_permission("admin:users:read", admin_addr).await {
            Ok(_) => {
                info!("✅ Admin {} authorized for user listing via centralized authority", admin_addr);
            }
            Err(_) => {
                error!("❌ Admin {} not authorized for user listing", admin_addr);
                return Err(StatusCode::FORBIDDEN);
            }
        }
    } else {
        error!("❌ Missing admin wallet address in request headers");
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Proceed with user listing logic (using existing implementation pattern)
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20).min(100); // Cap at 100
    let offset = (page - 1) * limit;

    let db_pool = app_state.db_pool.as_ref();

    // Get total count for pagination
    let total_count = match sqlx::query!(
        "SELECT COUNT(*) as count FROM wallet_users WHERE is_active = true"
    )
    .fetch_one(db_pool)
    .await
    {
        Ok(result) => result.count.unwrap_or(0) as i32,
        Err(e) => {
            error!("❌ Failed to get user count: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Fetch users with pagination
    let users_result = sqlx::query!(
        r#"
        SELECT 
            wu.wallet_address,
            wu.tier_level,
            wu.is_active,
            wu.created_at,
            wu.last_auth_at,
            wu.wallet_metadata,
            COUNT(wgm.group_id) as groups_count,
            0 as permissions_count
        FROM wallet_users wu
        LEFT JOIN wallet_group_memberships wgm ON wu.wallet_address = wgm.wallet_address
        LEFT JOIN permission_groups pg ON wgm.group_id = pg.id
        WHERE wu.is_active = true
        GROUP BY wu.wallet_address, wu.tier_level, wu.is_active, wu.created_at, wu.last_auth_at, wu.wallet_metadata
        ORDER BY wu.created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        limit as i64,
        offset as i64
    )
    .fetch_all(db_pool)
    .await;

    let users = match users_result {
        Ok(rows) => {
            let mut users = Vec::new();
            for row in rows {
                // Test centralized permission validation for each user
                let _user_permissions_valid = if let Some(wallet) = &admin_wallet {
                    permission_state.validate_permission("admin:users:view_details", wallet).await.unwrap_or(false)
                } else {
                    false
                };

                let user = UserSummaryResponse {
                    wallet_address: row.wallet_address,
                    tier_level: row.tier_level,
                    is_active: row.is_active,
                    created_at: row.created_at,
                    last_auth_at: row.last_auth_at,
                    permissions_count: row.permissions_count.unwrap_or(0) as i32,
                    groups_count: row.groups_count.unwrap_or(0) as i32,
                    last_activity: row.last_auth_at,
                    metadata: row.wallet_metadata,
                };
                users.push(user);
            }
            users
        }
        Err(e) => {
            error!("❌ Failed to fetch users: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let pagination = PaginationInfo {
        page,
        limit,
        total: total_count as i32,
        total_pages: ((total_count as f64) / (limit as f64)).ceil() as i32,
        has_next_page: page * limit < total_count,
        has_previous_page: page > 1,
    };

    let response = UserListResponse {
        users,
        total: total_count,
        pagination: pagination.clone(),
    };

    let metadata = AdminMetadata {
        operation: "list_users".to_string(),
        performed_by: admin_wallet.clone(),
        pagination: Some(pagination),
        permissions: None,
        metadata: None,
    };

    info!("✅ Admin: Retrieved {} users using centralized permission validation", response.users.len());

    Ok(Json(AdminApiResponse::success_with_meta(
        response,
        "Users retrieved with centralized permission validation",
        metadata,
    )))
}

/**
 * Utility function to demonstrate bulk permission validation
 */
pub async fn validate_user_permissions_bulk(
    State(permission_state): State<PermissionState>,
    headers: axum::http::HeaderMap,
    RequestJson(request): RequestJson<serde_json::Value>,
) -> Result<Json<AdminApiResponse<serde_json::Value>>, StatusCode> {
    info!("🔐 Admin: Performing bulk permission validation test");

    // Extract admin wallet
    let admin_wallet = headers.get("x-wallet-address")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("0x742d35Cc6AbAAC8b14A3780B5b0E11B2Ce65d695");

    // Validate admin can perform bulk operations
    match permission_state.require_permission("admin:permissions:bulk_validate", admin_wallet).await {
        Ok(_) => info!("✅ Admin authorized for bulk permission validation"),
        Err(_) => return Err(StatusCode::FORBIDDEN),
    }

    // Get test wallet and permissions from request
    let test_wallet = request.get("wallet_address")
        .and_then(|v| v.as_str())
        .unwrap_or("0x742d35Cc6AbAAC8b14A3780B5b0E11B2Ce65d695");

    let test_permissions = vec![
        "admin:users:read".to_string(),
        "admin:users:write".to_string(),
        "epsx:analytics:read".to_string(),
        "epsx:data:access".to_string(),
    ];

    // Create validation context
    let context = ValidationContext {
        request_id: uuid::Uuid::new_v4().to_string(),
        user_agent: headers.get("user-agent").and_then(|h| h.to_str().ok()).map(String::from),
        ip_address: headers.get("x-forwarded-for").and_then(|h| h.to_str().ok()).map(String::from),
        timestamp: chrono::Utc::now(),
        route_path: "/admin/users/validate-permissions-bulk".to_string(),
        http_method: "POST".to_string(),
    };

    // Perform bulk validation
    match permission_state.authority.bulk_validate_permissions(
        test_wallet,
        &test_permissions,
        &context,
    ).await {
        Ok(bulk_result) => {
            info!("✅ Bulk validation completed: {}/{} permissions granted", 
                bulk_result.granted_count, bulk_result.total_permissions);

            let response_data = serde_json::json!({
                "wallet_address": test_wallet,
                "total_permissions": bulk_result.total_permissions,
                "granted_count": bulk_result.granted_count,
                "denied_count": bulk_result.denied_count,
                "validation_time_ms": bulk_result.validation_time_ms,
                "results": bulk_result.results,
                "system_version": "centralized_authority_v2"
            });

            let metadata = AdminMetadata {
                operation: "bulk_permission_validation".to_string(),
                performed_by: Some(admin_wallet.to_string()),
                pagination: None,
                permissions: None,
                metadata: Some(serde_json::json!({
                    "request_id": context.request_id,
                    "validation_time_ms": bulk_result.validation_time_ms
                })),
            };

            Ok(Json(AdminApiResponse::success_with_meta(
                response_data,
                "Bulk permission validation completed",
                metadata,
            )))
        }
        Err(e) => {
            error!("❌ Bulk validation failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}