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
use std::sync::Arc;

use crate::web::auth::AppState;
use crate::web::admin::responses::{AdminApiResponse, AdminMetadata, PaginationInfo};
use crate::auth::unified_permission_service::UnifiedPermissionService;

// CQRS imports for wallet management
use crate::application::shared::{QueryHandler, CommandHandler};
use crate::application::wallet_management::queries::admin_models as query_models;
use crate::application::wallet_management::queries::admin_handlers as query_handlers;
use crate::application::wallet_management::commands::admin_models as command_models;
use crate::application::wallet_management::commands::admin_handlers as command_handlers;

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
pub struct WalletListQuery {
    /// Page number for pagination
    #[param(example = 1)]
    pub page: Option<i32>,
    /// Number of items per page
    #[param(example = 20)]
    pub limit: Option<i32>,
    /// Search term to filter by wallet address
    #[param(example = "0x1234")]
    pub search: Option<String>,
    /// Filter by user tier (deprecated - always shows all tiers)
    #[param(example = "premium")]
    pub tier: Option<String>,
    /// Filter by user status (active/inactive)
    #[param(example = "active")]
    pub status: Option<String>,
    /// Filter by creation date from (RFC3339 format)
    #[param(example = "2024-01-01T00:00:00Z")]
    pub date_from: Option<String>,
    /// Filter by creation date to (RFC3339 format)
    #[param(example = "2024-12-31T23:59:59Z")]
    pub date_to: Option<String>,
    /// Sort field
    #[param(example = "created_at")]
    pub sort_by: Option<String>,
    /// Sort order (asc/desc)
    #[param(example = "desc")]
    pub sort_order: Option<String>,
    /// Exclude members of a specific plan
    #[param(example = "uuid")]
    pub exclude_plan_id: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WalletSummaryResponse {
    /// Wallet address of the user
    #[schema(example = "0x1234567890123456789012345678901234567890")]
    pub wallet_address: String,
    /// Whether the wallet is currently active
    #[schema(example = true)]
    pub is_active: bool,
    /// When the wallet was created
    pub created_at: DateTime<Utc>,
    /// Last authentication timestamp
    pub last_auth_at: Option<DateTime<Utc>>,
    /// Total number of permissions assigned
    #[schema(example = 5)]
    pub permissions_count: i32,
    /// Number of permission plans assigned
    #[schema(example = 2)]
    pub plans_count: i32,
    /// Last activity timestamp
    pub last_activity: Option<DateTime<Utc>>,
    /// Additional wallet metadata
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WalletDetailResponse {
    /// Wallet address of the user
    #[schema(example = "0x1234567890123456789012345678901234567890")]
    pub wallet_address: String,
    /// Whether the wallet is currently active
    #[schema(example = true)]
    pub is_active: bool,
    /// When the wallet was created
    pub created_at: DateTime<Utc>,
    /// Last authentication timestamp
    pub last_auth_at: Option<DateTime<Utc>>,
    /// List of wallet permissions
    pub permissions: Vec<WalletPermission>,
    /// List of wallet plans
    pub plans: Vec<WalletPlan>,
    /// Activity summary for the wallet
    pub activity_summary: WalletActivitySummary,
    /// Additional wallet metadata
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WalletPermission {
    /// Permission string (format: platform:resource:action)
    #[schema(example = "epsx:analytics:read")]
    pub permission: String,
    /// Source of the permission (direct, plan, etc.)
    #[schema(example = "plan")]
    pub source: String,
    /// When the permission was granted
    pub granted_at: DateTime<Utc>,
    /// When the permission expires (if applicable)
    pub expires_at: Option<DateTime<Utc>>,
    /// Whether the permission is currently active
    #[schema(example = true)]
    pub is_active: bool,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WalletPlan {
    /// Plan unique identifier
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub plan_id: String,
    /// Plan display name
    #[schema(example = "Premium Users")]
    pub plan_name: String,
    /// Plan type (system, custom, etc.)
    #[schema(example = "system")]
    pub plan_type: String,
    /// When the wallet was assigned to this plan
    pub assigned_at: DateTime<Utc>,
    /// When the plan assignment expires (if applicable)
    pub expires_at: Option<DateTime<Utc>>,
    /// Whether the plan assignment is currently active
    #[schema(example = true)]
    pub is_active: bool,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WalletActivitySummary {
    /// Total number of logins for this wallet
    #[schema(example = 150)]
    pub total_logins: i32,
    /// Number of logins in the last 30 days
    #[schema(example = 25)]
    pub last_30_days_logins: i32,
    /// Total number of permissions (including expired)
    #[schema(example = 8)]
    pub total_permissions: i32,
    /// Number of currently active permissions
    #[schema(example = 5)]
    pub active_permissions: i32,
    /// Number of expired permissions
    #[schema(example = 3)]
    pub expired_permissions: i32,
    /// Number of plans the wallet belongs to
    #[schema(example = 2)]
    pub plans_count: i32,
}

// ============================================================================
// DTO MAPPING (FROM APPLICATION TO WEB)
// ============================================================================

impl From<query_models::WalletSummaryDto> for WalletSummaryResponse {
    fn from(dto: query_models::WalletSummaryDto) -> Self {
        Self {
            wallet_address: dto.wallet_address,
            is_active: dto.is_active,
            created_at: dto.created_at,
            last_auth_at: dto.last_auth_at,
            permissions_count: dto.permissions_count,
            plans_count: dto.plans_count,
            last_activity: dto.last_activity,
            metadata: dto.metadata.unwrap_or(serde_json::json!({})),
        }
    }
}

impl From<query_models::WalletDetailDto> for WalletDetailResponse {
    fn from(dto: query_models::WalletDetailDto) -> Self {
        Self {
            wallet_address: dto.wallet_address,
            is_active: dto.is_active,
            created_at: dto.created_at,
            last_auth_at: dto.last_auth_at,
            permissions: dto.permissions.into_iter().map(Into::into).collect(),
            plans: dto.plans.into_iter().map(Into::into).collect(),
            activity_summary: dto.activity_summary.into(),
            metadata: dto.metadata.unwrap_or(serde_json::json!({})),
        }
    }
}

impl From<query_models::WalletPermissionDto> for WalletPermission {
    fn from(dto: query_models::WalletPermissionDto) -> Self {
        Self {
            permission: dto.permission,
            source: dto.source,
            granted_at: dto.granted_at,
            expires_at: dto.expires_at,
            is_active: dto.is_active,
        }
    }
}

impl From<query_models::WalletPlanDto> for WalletPlan {
    fn from(dto: query_models::WalletPlanDto) -> Self {
        Self {
            plan_id: dto.plan_id,
            plan_name: dto.plan_name,
            plan_type: dto.plan_type,
            assigned_at: dto.assigned_at,
            expires_at: dto.expires_at,
            is_active: dto.is_active,
        }
    }
}

impl From<query_models::WalletActivitySummaryDto> for WalletActivitySummary {
    fn from(dto: query_models::WalletActivitySummaryDto) -> Self {
        Self {
            total_logins: dto.total_logins,
            last_30_days_logins: dto.last_30_days_logins,
            total_permissions: dto.total_permissions,
            active_permissions: dto.active_permissions,
            expired_permissions: dto.expired_permissions,
            plans_count: dto.plans_count,
        }
    }
}

impl From<query_models::PaginationDto> for PaginationInfo {
    fn from(dto: query_models::PaginationDto) -> Self {
        Self {
            page: dto.page,
            limit: dto.limit,
            total: dto.total,
            total_pages: dto.total_pages,
            has_next_page: dto.has_next_page,
            has_previous_page: dto.has_previous_page,
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WalletListResponse {
    /// List of wallets
    pub wallets: Vec<WalletSummaryResponse>,
    /// Total number of wallets matching the filters
    #[schema(example = 250)]
    pub total: i32,
    /// Pagination information
    pub pagination: PaginationInfo,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateWalletRequest {
    /// Whether the wallet should be active or disabled
    #[schema(example = true)]
    pub is_active: Option<bool>,
    /// Additional metadata stored as JSON
    #[schema(example = json!({"notes": "VIP customer", "tier": "premium"}))]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct BulkPermissionValidationRequest {
    /// Wallet address to validate permissions for
    #[schema(example = "0x1234567890123456789012345678901234567890")]
    pub wallet_address: String,
    /// List of permissions to validate
    #[schema(example = json!(["admin:users:read", "epsx:analytics:read"]))]
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BulkPermissionValidationResponse {
    /// Wallet address that was validated
    #[schema(example = "0x1234567890123456789012345678901234567890")]
    pub wallet_address: String,
    /// Total number of permissions validated
    #[schema(example = 4)]
    pub total_permissions: u32,
    /// Number of permissions granted
    #[schema(example = 3)]
    pub granted_count: u32,
    /// Number of permissions denied
    #[schema(example = 1)]
    pub denied_count: u32,
    /// Time taken for validation in milliseconds
    #[schema(example = 15)]
    pub validation_time_ms: u64,
    /// Detailed results for each permission
    #[schema(example = json!([{"permission": "admin:users:read", "granted": true, "source": "plan"}]))]
    pub results: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WalletStatsResponse {
    /// Total number of registered wallet users
    #[schema(example = 1250)]
    pub total_users: i32,
    /// Number of currently active users
    #[schema(example = 980)]
    pub active_users: i32,
    /// Number of inactive users
    #[schema(example = 270)]
    pub inactive_users: i32,
    /// User distribution by tier (deprecated - shows empty object)
    #[schema(example = json!({}))]
    pub users_by_tier: serde_json::Value,
    /// Number of new users in the last 30 days
    #[schema(example = 85)]
    pub new_users_30_days: i32,
    /// Number of active users in the last 30 days
    #[schema(example = 650)]
    pub active_users_30_days: i32,
    /// Monthly growth rate percentage
    #[schema(example = 7.2)]
    pub growth_rate: f64,
}

impl From<query_models::WalletStatsDto> for WalletStatsResponse {
    fn from(dto: query_models::WalletStatsDto) -> Self {
        Self {
            total_users: dto.total_users,
            active_users: dto.active_users,
            inactive_users: dto.inactive_users,
            users_by_tier: serde_json::json!({}),
            new_users_30_days: dto.new_users_30_days,
            active_users_30_days: dto.active_users_30_days,
            growth_rate: dto.growth_rate,
        }
    }
}

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
        ("sort_order" = Option<String>, Query, description = "Sort order (asc/desc)"),
        ("exclude_plan_id" = Option<String>, Query, description = "Exclude members of a specific plan")
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
        exclude_plan_id: params.exclude_plan_id,
    };

    // 2. Execute CQRS handler
    let handler = query_handlers::GetWalletListQueryHandler::new(app_state.db_pool.clone());
    let response = handler.handle(query).await.map_err(|e| {
        error!("❌ Wallet list query failed: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // 3. Map DTOs to web responses using traits
    let wallets: Vec<WalletSummaryResponse> = response
        .wallets
        .into_iter()
        .map(Into::into)
        .collect();

    let pagination: PaginationInfo = response.pagination.into();

    // 4. Build web response
    let web_response = WalletListResponse {
        wallets,
        total: pagination.total,
        pagination: pagination.clone(),
    };

    let metadata = AdminMetadata::list_operation("list_wallets", pagination);

    info!(
        "✅ Admin: Successfully listed {} wallets",
        web_response.wallets.len()
    );
    Ok(Json(AdminApiResponse::success_with_meta(
        web_response,
        "Wallets retrieved successfully",
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

    // 3. Map to web response using traits
    let web_response: WalletDetailResponse = response.wallet.into();

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

    // 3. Map to web response using traits
    let web_response: WalletDetailResponse = response.wallet.into();

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

    // 3. Map to web response using traits
    let web_response: WalletStatsResponse = response.stats.into();

    let metadata = AdminMetadata::crud_operation("get_user_stats", Some("admin".to_string()));

    info!("✅ Admin: Successfully retrieved user statistics");
    Ok(Json(AdminApiResponse::success_with_meta(
        web_response,
        "User statistics retrieved successfully",
        metadata,
    )))
}

/**
 * Disable a wallet (CQRS-based)
 * POST /admin/wallets/:wallet_address/disable
 */
#[utoipa::path(
    post,
    path = "/admin/wallets/{wallet_address}/disable",
    tag = "admin-wallets",
    request_body = command_models::DisableWalletCommand,
    responses(
        (status = 200, description = "Successfully disabled wallet"),
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
pub async fn disable_user_handler(
    Path(wallet_address): Path<String>,
    State(app_state): State<AppState>,
    RequestJson(request): RequestJson<command_models::DisableWalletCommand>,
) -> Result<Json<AdminApiResponse<command_models::DisableWalletResponse>>, StatusCode> {
    info!("🚫 Admin: Disabling user: {} (CQRS)", wallet_address);

    // Ensure path param matches body logic if needed, but command has it.
    // Overwrite the wallet address in command from path to be safe/consistent
    let command = command_models::DisableWalletCommand {
        wallet_address: wallet_address.clone(),
        ..request
    };

    // Execute CQRS handler
    let handler = command_handlers::DisableWalletCommandHandler::new(app_state.db_pool.clone());
    let response = handler.handle(command).await.map_err(|e| {
        error!("❌ Disable wallet failed: {}", e);
        if e.to_string().contains("not found") {
            return StatusCode::NOT_FOUND;
        }
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let metadata = AdminMetadata::crud_operation("disable_user", Some("admin".to_string()));

    Ok(Json(AdminApiResponse::success_with_meta(
        response,
        "Wallet disabled successfully",
        metadata,
    )))
}

/**
 * Enable a wallet (CQRS-based)
 * POST /admin/wallets/:wallet_address/enable
 */
#[utoipa::path(
    post,
    path = "/admin/wallets/{wallet_address}/enable",
    tag = "admin-wallets",
    request_body = command_models::EnableWalletCommand,
    responses(
        (status = 200, description = "Successfully enabled wallet"),
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
pub async fn enable_user_handler(
    Path(wallet_address): Path<String>,
    State(app_state): State<AppState>,
    RequestJson(request): RequestJson<command_models::EnableWalletCommand>,
) -> Result<Json<AdminApiResponse<command_models::EnableWalletResponse>>, StatusCode> {
    info!("✅ Admin: Enabling user: {} (CQRS)", wallet_address);

    // Ensure path param matches body logic
    let command = command_models::EnableWalletCommand {
        wallet_address: wallet_address.clone(),
        ..request
    };

    // Execute CQRS handler
    let handler = command_handlers::EnableWalletCommandHandler::new(app_state.db_pool.clone());
    let response = handler.handle(command).await.map_err(|e| {
        error!("❌ Enable wallet failed: {}", e);
        if e.to_string().contains("not found") {
            return StatusCode::NOT_FOUND;
        }
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let metadata = AdminMetadata::crud_operation("enable_user", Some("admin".to_string()));

    Ok(Json(AdminApiResponse::success_with_meta(
        response,
        "Wallet enabled successfully",
        metadata,
    )))
}


// ============================================================================
// ADMIN UTILITY HANDLERS
// ============================================================================

/// Bulk permission validation for admin utilities
/// Validates multiple permissions for a wallet in a single request
#[utoipa::path(
    post,
    path = "/admin/wallets/validate-permissions-bulk",
    tag = "admin-wallets",
    request_body = BulkPermissionValidationRequest,
    responses(
        (status = 200, description = "Successfully validated permissions", body = AdminApiResponse<BulkPermissionValidationResponse>),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Admin access required"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
pub async fn validate_user_permissions_bulk(
    State(permission_service): State<Arc<UnifiedPermissionService>>,
    headers: axum::http::HeaderMap,
    RequestJson(request): RequestJson<BulkPermissionValidationRequest>,
) -> Result<Json<AdminApiResponse<BulkPermissionValidationResponse>>, StatusCode> {
    info!("🔐 Admin: Performing bulk permission validation test");
    let start_time = std::time::Instant::now();

    // Extract admin wallet
    let admin_wallet = headers.get("x-wallet-address")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("0x742d35Cc6AbAAC8b14A3780B5b0E11B2Ce65d695");

    // Validate admin can perform bulk operations
    match permission_service.has_permission(admin_wallet, "admin:permissions:bulk_validate").await {
        Ok(true) => info!("✅ Admin authorized for bulk permission validation"),
        Ok(false) => {
            info!("❌ Admin not authorized for bulk permission validation");
            return Err(StatusCode::FORBIDDEN);
        },
        Err(e) => {
            error!("❌ Permission check failed: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    // Get test wallet and permissions from request
    let test_wallet = request.wallet_address.clone();
    let test_permissions = request.permissions.clone();

    // Perform bulk validation using UnifiedPermissionService
    let mut results = Vec::new();
    let mut granted_count = 0u32;
    let mut denied_count = 0u32;

    for permission in &test_permissions {
        match permission_service.has_permission(&test_wallet, permission).await {
            Ok(granted) => {
                if granted {
                    granted_count += 1;
                } else {
                    denied_count += 1;
                }
                results.push(serde_json::json!({
                    "permission": permission,
                    "granted": granted,
                    "source": if granted { "direct_or_plan" } else { "not_found" },
                    "expires_at": null,
                    "reason": null
                }));
            }
            Err(e) => {
                denied_count += 1;
                results.push(serde_json::json!({
                    "permission": permission,
                    "granted": false,
                    "source": "error",
                    "expires_at": null,
                    "reason": e.to_string()
                }));
            }
        }
    }

    let validation_time_ms = start_time.elapsed().as_millis() as u64;

    info!("✅ Bulk validation completed: {}/{} permissions granted in {}ms",
        granted_count, test_permissions.len(), validation_time_ms);

    let response_data = BulkPermissionValidationResponse {
        wallet_address: test_wallet,
        total_permissions: test_permissions.len() as u32,
        granted_count,
        denied_count,
        validation_time_ms,
        results,
    };

    let metadata = AdminMetadata {
        operation: "bulk_permission_validation".to_string(),
        performed_by: Some(admin_wallet.to_string()),
        pagination: None,
        permissions: None,
        metadata: Some(serde_json::json!({
            "request_id": uuid::Uuid::new_v4().to_string(),
            "validation_time_ms": validation_time_ms
        })),
    };

    Ok(Json(AdminApiResponse::success_with_meta(
        response_data,
        "Bulk permission validation completed",
        metadata,
    )))
}