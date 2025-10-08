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

use crate::web::auth::AppState;
use crate::web::admin::responses::{AdminApiResponse, AdminMetadata, PaginationInfo};
use crate::auth::{PermissionState, HandlerPermissionExt, ValidationContext, PermissionValidator};

// CQRS imports for wallet management
use crate::application::shared::{QueryHandler, CommandHandler};
use crate::application::wallet_management::queries::admin_models as query_models;
use crate::application::wallet_management::queries::admin_handlers as query_handlers;
use crate::application::wallet_management::commands::admin_models as command_models;
use crate::application::wallet_management::commands::admin_handlers as command_handlers;

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct WalletListQuery {
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
pub struct WalletSummaryResponse {
    pub wallet_address: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_auth_at: Option<DateTime<Utc>>,
    pub permissions_count: i32,
    pub groups_count: i32,
    pub last_activity: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct WalletDetailResponse {
    pub wallet_address: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_auth_at: Option<DateTime<Utc>>,
    pub permissions: Vec<WalletPermission>,
    pub groups: Vec<WalletGroup>,
    pub activity_summary: WalletActivitySummary,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct WalletPermission {
    pub permission: String,
    pub source: String,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct WalletGroup {
    pub group_id: String,
    pub group_name: String,
    pub group_type: String,
    pub assigned_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct WalletActivitySummary {
    pub total_logins: i32,
    pub last_30_days_logins: i32,
    pub total_permissions: i32,
    pub active_permissions: i32,
    pub expired_permissions: i32,
    pub groups_count: i32,
}

#[derive(Debug, Serialize)]
pub struct WalletListResponse {
    pub users: Vec<WalletSummaryResponse>,
    pub total: i32,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateWalletRequest {
    pub is_active: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct WalletStatsResponse {
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
 * List all users with filtering and pagination (CQRS-based)
 * GET /admin/wallets
 */
#[utoipa::path(
    get,
    path = "/admin/wallets",
    tag = "admin-wallets",
    responses(
        (status = 200, description = "Successfully retrieved wallet list"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("page" = Option<i32>, Query, description = "Page number"),
        ("limit" = Option<i32>, Query, description = "Items per page"),
        ("search" = Option<String>, Query, description = "Search by wallet address"),
        ("tier" = Option<String>, Query, description = "Filter by tier"),
        ("status" = Option<String>, Query, description = "Filter by status (active/inactive)"),
        ("date_from" = Option<String>, Query, description = "Filter by creation date from"),
        ("date_to" = Option<String>, Query, description = "Filter by creation date to"),
        ("sort_by" = Option<String>, Query, description = "Sort field"),
        ("sort_order" = Option<String>, Query, description = "Sort order (asc/desc)")
    ),
    security(("bearerAuth" = []))
)]
pub async fn list_users_handler(
    Query(params): Query<WalletListQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<AdminApiResponse<WalletListResponse>>, StatusCode> {
    info!("🔍 Admin: Listing users with filters (CQRS): {:?}", params);

    // 1. Create query (parameter pass-through)
    let query = query_models::GetWalletListQuery {
        page: params.page,
        limit: params.limit,
        search: params.search,
        status: params.status,
        date_from: params.date_from,
        date_to: params.date_to,
        sort_by: params.sort_by,
        sort_order: params.sort_order,
    };

    // 2. Execute CQRS handler
    let handler = query_handlers::GetWalletListQueryHandler::new(app_state.db_pool.clone());
    let response = handler.handle(query).await.map_err(|e| {
        error!("❌ Wallet list query failed: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // 3. Map DTOs to web responses
    let users: Vec<WalletSummaryResponse> = response
        .wallets
        .into_iter()
        .map(|dto| WalletSummaryResponse {
            wallet_address: dto.wallet_address,
            is_active: dto.is_active,
            created_at: dto.created_at,
            last_auth_at: dto.last_auth_at,
            permissions_count: dto.permissions_count,
            groups_count: dto.groups_count,
            last_activity: dto.last_activity,
            metadata: serde_json::json!({}),
        })
        .collect();

    // 4. Build pagination info
    let pagination = PaginationInfo {
        page: response.pagination.page,
        limit: response.pagination.limit,
        total: response.pagination.total,
        total_pages: response.pagination.total_pages,
        has_next_page: response.pagination.has_next_page,
        has_previous_page: response.pagination.has_previous_page,
    };

    // 5. Build web response
    let web_response = WalletListResponse {
        users,
        total: response.pagination.total,
        pagination: pagination.clone(),
    };

    let metadata = AdminMetadata::list_operation("list_users", pagination);

    info!(
        "✅ Admin: Successfully listed {} users",
        web_response.users.len()
    );
    Ok(Json(AdminApiResponse::success_with_meta(
        web_response,
        "Users retrieved successfully",
        metadata,
    )))
}

/**
 * Get detailed user information (CQRS-based)
 * GET /admin/wallets/:wallet_address
 */
#[utoipa::path(
    get,
    path = "/admin/wallets/{wallet_address}",
    tag = "admin-wallets",
    responses(
        (status = 200, description = "Successfully retrieved wallet details"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Wallet not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("wallet_address" = String, Path, description = "Wallet address")
    ),
    security(("bearerAuth" = []))
)]
pub async fn get_user_handler(
    Path(wallet_address): Path<String>,
    State(app_state): State<AppState>,
) -> Result<Json<AdminApiResponse<WalletDetailResponse>>, StatusCode> {
    info!("🔍 Admin: Getting user details for: {} (CQRS)", wallet_address);

    // 1. Create query
    let query = query_models::GetWalletDetailQuery {
        wallet_address: wallet_address.clone(),
    };

    // 2. Execute CQRS handler
    let handler = query_handlers::GetWalletDetailQueryHandler::new(app_state.db_pool.clone());
    let response = handler.handle(query).await.map_err(|e| {
        error!("❌ Wallet detail query failed: {}", e);
        if e.to_string().contains("not found") {
            return StatusCode::NOT_FOUND;
        }
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // 3. Map to web response
    let web_response = map_wallet_detail_dto(response.wallet);

    let metadata = AdminMetadata::crud_operation("get_user", Some("admin".to_string()));

    info!(
        "✅ Admin: Successfully retrieved user details for: {}",
        wallet_address
    );
    Ok(Json(AdminApiResponse::success_with_meta(
        web_response,
        "User details retrieved successfully",
        metadata,
    )))
}

/**
 * Update user information (CQRS-based)
 * PUT /admin/wallets/:wallet_address
 */
#[utoipa::path(
    put,
    path = "/admin/wallets/{wallet_address}",
    tag = "admin-wallets",
    request_body = UpdateWalletRequest,
    responses(
        (status = 200, description = "Successfully updated wallet"),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Wallet not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("wallet_address" = String, Path, description = "Wallet address")
    ),
    security(("bearerAuth" = []))
)]
pub async fn update_user_handler(
    Path(wallet_address): Path<String>,
    State(app_state): State<AppState>,
    RequestJson(request): RequestJson<UpdateWalletRequest>,
) -> Result<Json<AdminApiResponse<WalletDetailResponse>>, StatusCode> {
    info!("✏️ Admin: Updating user: {} (CQRS)", wallet_address);

    // 1. Create command
    let command = command_models::UpdateWalletCommand {
        wallet_address: wallet_address.clone(),
        is_active: request.is_active,
        metadata: request.metadata,
    };

    // 2. Execute CQRS handler
    let handler = command_handlers::UpdateWalletCommandHandler::new(app_state.db_pool.clone());
    let response = handler.handle(command).await.map_err(|e| {
        error!("❌ Update wallet failed: {}", e);
        if e.to_string().contains("not found") {
            return StatusCode::NOT_FOUND;
        }
        if e.to_string().contains("validation") {
            return StatusCode::BAD_REQUEST;
        }
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // 3. Map to web response
    let web_response = map_wallet_detail_dto(response.wallet);

    let metadata = AdminMetadata::crud_operation("update_user", Some("admin".to_string()));

    info!("✅ Admin: Successfully updated user: {}", wallet_address);
    Ok(Json(AdminApiResponse::success_with_meta(
        web_response,
        &response.message,
        metadata,
    )))
}

/**
 * Get user statistics (CQRS-based)
 * GET /admin/wallets/stats
 */
#[utoipa::path(
    get,
    path = "/admin/wallets/stats",
    tag = "admin-wallets",
    responses(
        (status = 200, description = "Successfully retrieved wallet statistics"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = []))
)]
pub async fn get_user_stats_handler(
    State(app_state): State<AppState>,
) -> Result<Json<AdminApiResponse<WalletStatsResponse>>, StatusCode> {
    info!("📊 Admin: Getting user statistics (CQRS)");

    // 1. Create query
    let query = query_models::GetWalletStatsQuery {};

    // 2. Execute CQRS handler
    let handler = query_handlers::GetWalletStatsQueryHandler::new(app_state.db_pool.clone());
    let response = handler.handle(query).await.map_err(|e| {
        error!("❌ Stats query failed: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // 3. Map to web response
    let web_response = WalletStatsResponse {
        total_users: response.stats.total_users,
        active_users: response.stats.active_users,
        inactive_users: response.stats.inactive_users,
        users_by_tier: serde_json::json!({}), // Tier system removed
        new_users_30_days: response.stats.new_users_30_days,
        active_users_30_days: response.stats.active_users_30_days,
        growth_rate: response.stats.growth_rate,
    };

    let metadata = AdminMetadata::crud_operation("get_user_stats", Some("admin".to_string()));

    info!("✅ Admin: Successfully retrieved user statistics");
    Ok(Json(AdminApiResponse::success_with_meta(
        web_response,
        "User statistics retrieved successfully",
        metadata,
    )))
}

// ============================================================================
// DTO MAPPING HELPERS (CQRS → Web Responses)
// ============================================================================

/// Map CQRS WalletDetailDto to web WalletDetailResponse
fn map_wallet_detail_dto(dto: query_models::WalletDetailDto) -> WalletDetailResponse {
    WalletDetailResponse {
        wallet_address: dto.wallet_address,
        is_active: dto.is_active,
        created_at: dto.created_at,
        last_auth_at: dto.last_auth_at,
        permissions: dto
            .permissions
            .into_iter()
            .map(|p| WalletPermission {
                permission: p.permission,
                source: p.source,
                granted_at: p.granted_at,
                expires_at: p.expires_at,
                is_active: p.is_active,
            })
            .collect(),
        groups: dto
            .groups
            .into_iter()
            .map(|g| WalletGroup {
                group_id: g.group_id,
                group_name: g.group_name,
                group_type: g.group_type,
                assigned_at: g.assigned_at,
                expires_at: g.expires_at,
                is_active: g.is_active,
            })
            .collect(),
        activity_summary: WalletActivitySummary {
            total_logins: dto.activity_summary.total_logins,
            last_30_days_logins: dto.activity_summary.last_30_days_logins,
            total_permissions: dto.activity_summary.total_permissions,
            active_permissions: dto.activity_summary.active_permissions,
            expired_permissions: dto.activity_summary.expired_permissions,
            groups_count: dto.activity_summary.groups_count,
        },
        metadata: serde_json::json!({}),
    }
}

// ============================================================================
// ADMIN UTILITY HANDLERS
// ============================================================================

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
        route_path: "/admin/wallets/validate-permissions-bulk".to_string(),
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