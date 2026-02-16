// ============================================================================
// WALLET DISABLE/ENABLE HANDLERS
// Handlers for temporarily disabling and re-enabling wallets
// ============================================================================

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Json as RequestJson,
};
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use uuid::Uuid;

use crate::web::auth::AppState;
use crate::web::admin::responses::{AdminApiResponse, AdminMetadata};

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct DisableWalletRequest {
    /// Duration in days (null = until manual re-enable)
    pub duration_days: Option<i32>,
    /// Reason category for the disable
    pub reason_category: String,
    /// Detailed reason description
    pub reason_details: String,
    /// Platforms affected by this disable
    pub affected_platforms: Vec<String>,
    /// Whether to block login entirely
    pub block_login: bool,
    /// Whether to pause active subscriptions
    pub pause_subscriptions: bool,
    /// Whether to notify the user
    pub notify_user: bool,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct EnableWalletRequest {
    /// Platforms to re-enable
    pub platforms_to_enable: Vec<String>,
    /// Whether to restore previously disabled permissions
    pub restore_permissions: bool,
    /// Whether to resume paused subscriptions
    pub resume_subscriptions: bool,
    /// Admin's note about the resolution
    pub resolution_note: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DisableWalletResponse {
    pub success: bool,
    pub wallet_address: String,
    pub disabled_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub message: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct EnableWalletResponse {
    pub success: bool,
    pub wallet_address: String,
    pub enabled_at: DateTime<Utc>,
    pub message: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ActivityLogEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub description: String,
    pub performed_by: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ActivityLogResponse {
    pub events: Vec<ActivityLogEntry>,
    pub total: i64,
}

// ============================================================================
// DISABLE/ENABLE HANDLERS
// ============================================================================

/// Temporarily disable a wallet
/// POST /admin/wallets/{wallet_address}/disable
#[utoipa::path(
    post,
    path = "/admin/wallets/{wallet_address}/disable",
    tag = "admin-wallets",
    request_body = DisableWalletRequest,
    responses(
        (status = 200, description = "Wallet disabled successfully"),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Wallet not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("wallet_address" = String, Path, description = "Wallet address to disable")
    ),
    security(("bearerAuth" = []))
)]
pub async fn disable_wallet_handler(
    Path(wallet_address): Path<String>,
    State(app_state): State<AppState>,
    axum::Extension(admin_context): axum::Extension<crate::web::middleware::OpenIDUserContext>,
    RequestJson(request): RequestJson<DisableWalletRequest>,
) -> Result<Json<AdminApiResponse<DisableWalletResponse>>, StatusCode> {
    let admin_wallet = &admin_context.wallet_address;
    info!("Admin {} disabling wallet: {}", admin_wallet, wallet_address);

    let mut conn = app_state.db_pool.get().await.map_err(|e| {
        error!("Failed to get connection: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let now = Utc::now();
    let expires_at = request.duration_days.map(|days| now + Duration::days(days as i64));

    // Build disable_info JSON with actual admin wallet
    let disable_info = serde_json::json!({
        "disabled_at": now.to_rfc3339(),
        "disabled_by": admin_wallet,
        "duration": request.duration_days,
        "expires_at": expires_at.map(|e| e.to_rfc3339()),
        "reason_category": request.reason_category,
        "reason_details": request.reason_details,
        "affected_platforms": request.affected_platforms,
        "block_login": request.block_login,
        "pause_subscriptions": request.pause_subscriptions,
    });

    // Update auth_users table
    let rows_affected = diesel::sql_query(
        "UPDATE auth_users SET is_active = false, disable_info = $1, updated_at = $2 WHERE wallet_address = $3"
    )
    .bind::<diesel::sql_types::Jsonb, _>(&disable_info)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .bind::<diesel::sql_types::Text, _>(&wallet_address)
    .execute(&mut conn)
    .await
    .map_err(|e| {
        error!("Failed to disable wallet: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if rows_affected == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    // Log the activity with actual admin wallet
    let _ = diesel::sql_query(
        "INSERT INTO wallet_activity_logs (wallet_address, event_type, description, performed_by, metadata) 
         VALUES ($1, 'wallet_disabled', $2, $3, $4)"
    )
    .bind::<diesel::sql_types::Text, _>(&wallet_address)
    .bind::<diesel::sql_types::Text, _>(format!("Wallet disabled: {}", request.reason_details))
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some(admin_wallet.as_str()))
    .bind::<diesel::sql_types::Jsonb, _>(&disable_info)
    .execute(&mut conn)
    .await;

    // Send notification if requested
    if request.notify_user {
        // Insert notification into notifications table
        let notification_payload = serde_json::json!({
            "type": "wallet_disabled",
            "title": "Account Temporarily Disabled",
            "message": format!("Your account has been temporarily disabled. Reason: {}", request.reason_category),
            "reason": request.reason_details,
            "expires_at": expires_at.map(|e| e.to_rfc3339()),
        });
        let _ = diesel::sql_query(
            "INSERT INTO notifications (wallet_address, notification_type, title, message, data, created_at) 
             VALUES ($1, 'system', 'Account Disabled', $2, $3, $4)"
        )
        .bind::<diesel::sql_types::Text, _>(&wallet_address)
        .bind::<diesel::sql_types::Text, _>(format!("Your account has been temporarily disabled. Reason: {}", request.reason_category))
        .bind::<diesel::sql_types::Jsonb, _>(&notification_payload)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)
        .await;
    }

    let response = DisableWalletResponse {
        success: true,
        wallet_address: wallet_address.clone(),
        disabled_at: now,
        expires_at,
        message: format!("Wallet {} has been disabled", wallet_address),
    };

    let metadata = AdminMetadata::crud_operation("disable_wallet", Some(admin_wallet.clone()));

    info!("Admin {}: Successfully disabled wallet: {}", admin_wallet, wallet_address);
    Ok(Json(AdminApiResponse::success_with_meta(
        response,
        "Wallet disabled successfully",
        metadata,
    )))
}

/// Re-enable a disabled wallet
/// POST /admin/wallets/{wallet_address}/enable
#[utoipa::path(
    post,
    path = "/admin/wallets/{wallet_address}/enable",
    tag = "admin-wallets",
    request_body = EnableWalletRequest,
    responses(
        (status = 200, description = "Wallet enabled successfully"),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Wallet not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("wallet_address" = String, Path, description = "Wallet address to enable")
    ),
    security(("bearerAuth" = []))
)]
pub async fn enable_wallet_handler(
    Path(wallet_address): Path<String>,
    State(app_state): State<AppState>,
    axum::Extension(admin_context): axum::Extension<crate::web::middleware::OpenIDUserContext>,
    RequestJson(request): RequestJson<EnableWalletRequest>,
) -> Result<Json<AdminApiResponse<EnableWalletResponse>>, StatusCode> {
    let admin_wallet = &admin_context.wallet_address;
    info!("Admin {} re-enabling wallet: {}", admin_wallet, wallet_address);

    let mut conn = app_state.db_pool.get().await.map_err(|e| {
        error!("Failed to get connection: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let now = Utc::now();

    // Update auth_users table
    let rows_affected = diesel::sql_query(
        "UPDATE auth_users SET is_active = true, disable_info = NULL, updated_at = $1 WHERE wallet_address = $2"
    )
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .bind::<diesel::sql_types::Text, _>(&wallet_address)
    .execute(&mut conn)
    .await
    .map_err(|e| {
        error!("Failed to enable wallet: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if rows_affected == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    // Log the activity with actual admin wallet
    let activity_meta = serde_json::json!({
        "enabled_at": now.to_rfc3339(),
        "enabled_by": admin_wallet,
        "platforms_enabled": request.platforms_to_enable,
        "permissions_restored": request.restore_permissions,
        "subscriptions_resumed": request.resume_subscriptions,
        "resolution_note": request.resolution_note,
    });

    let _ = diesel::sql_query(
        "INSERT INTO wallet_activity_logs (wallet_address, event_type, description, performed_by, metadata) 
         VALUES ($1, 'wallet_enabled', $2, $3, $4)"
    )
    .bind::<diesel::sql_types::Text, _>(&wallet_address)
    .bind::<diesel::sql_types::Text, _>(format!("Wallet re-enabled: {}", request.resolution_note))
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some(admin_wallet.as_str()))
    .bind::<diesel::sql_types::Jsonb, _>(&activity_meta)
    .execute(&mut conn)
    .await;

    // Restore permissions if requested
    // This restores any soft-deleted or expired permissions that were set to expire when wallet was disabled
    if request.restore_permissions {
        // Update any permissions that were marked as expired when wallet was disabled
        let _ = diesel::sql_query(
            "UPDATE wallet_permissions 
             SET expires_at = NULL, updated_at = $1 
             WHERE wallet_address = $2 
             AND expires_at < $1 
             AND source_metadata->>'disabled_during_account_disable' = 'true'"
        )
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Text, _>(&wallet_address)
        .execute(&mut conn)
        .await;

        info!("Restored permissions for wallet: {}", wallet_address);
    }

    // Resume subscriptions if requested
    if request.resume_subscriptions {
        // Get payments pool for subscription updates
        if let Ok(payments_pool) = crate::infrastructure::database::get_payments_pool().await {
            if let Ok(mut payments_conn) = payments_pool.get().await {
                // Resume paused subscriptions
                let _ = diesel::sql_query(
                    "UPDATE subscriptions 
                     SET status = 'active', cancelled_at = NULL, metadata = metadata || '{\"resumed_by_admin\": true}'::jsonb
                     WHERE wallet_address = $1 
                     AND status = 'paused'"
                )
                .bind::<diesel::sql_types::Text, _>(&wallet_address)
                .execute(&mut payments_conn)
                .await;

                info!("Resumed subscriptions for wallet: {}", wallet_address);
            }
        }
    }

    let response = EnableWalletResponse {
        success: true,
        wallet_address: wallet_address.clone(),
        enabled_at: now,
        message: format!("Wallet {} has been re-enabled", wallet_address),
    };

    let metadata = AdminMetadata::crud_operation("enable_wallet", Some(admin_wallet.clone()));

    info!("Admin {}: Successfully enabled wallet: {}", admin_wallet, wallet_address);
    Ok(Json(AdminApiResponse::success_with_meta(
        response,
        "Wallet enabled successfully",
        metadata,
    )))
}

// ============================================================================
// ACTIVITY LOG HANDLERS
// ============================================================================

/// Get activity history for a wallet
/// GET /admin/wallets/{wallet_address}/activity
#[utoipa::path(
    get,
    path = "/admin/wallets/{wallet_address}/activity",
    tag = "admin-wallets",
    responses(
        (status = 200, description = "Activity history retrieved successfully"),
        (status = 404, description = "Wallet not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("wallet_address" = String, Path, description = "Wallet address"),
        ("limit" = Option<i32>, Query, description = "Number of events to return (default 20)")
    ),
    security(("bearerAuth" = []))
)]
pub async fn get_wallet_activity_handler(
    Path(wallet_address): Path<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
    State(app_state): State<AppState>,
) -> Result<Json<AdminApiResponse<ActivityLogResponse>>, StatusCode> {
    info!("Admin: Getting activity for wallet: {}", wallet_address);

    let limit: i32 = params.get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(20)
        .min(100);

    let mut conn = app_state.db_pool.get().await.map_err(|e| {
        error!("Failed to get connection: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Query activity logs
    #[derive(QueryableByName)]
    struct ActivityRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        event_type: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        description: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        performed_by: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        created_at: DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Jsonb)]
        metadata: serde_json::Value,
    }

    let activities: Vec<ActivityRow> = diesel::sql_query(
        "SELECT id, event_type, description, performed_by, created_at, metadata 
         FROM wallet_activity_logs 
         WHERE wallet_address = $1 
         ORDER BY created_at DESC 
         LIMIT $2"
    )
    .bind::<diesel::sql_types::Text, _>(&wallet_address)
    .bind::<diesel::sql_types::Integer, _>(limit)
    .load(&mut conn)
    .await
    .unwrap_or_default();

    // Get total count
    #[derive(QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    let total: i64 = diesel::sql_query(
        "SELECT COUNT(*) as count FROM wallet_activity_logs WHERE wallet_address = $1"
    )
    .bind::<diesel::sql_types::Text, _>(&wallet_address)
    .get_result::<CountRow>(&mut conn)
    .await
    .map(|r| r.count)
    .unwrap_or(0);

    let events: Vec<ActivityLogEntry> = activities
        .into_iter()
        .map(|a| ActivityLogEntry {
            id: a.id.to_string(),
            event_type: a.event_type,
            description: a.description,
            performed_by: a.performed_by,
            timestamp: a.created_at,
            metadata: a.metadata,
        })
        .collect();

    let response = ActivityLogResponse { events, total };

    let metadata = AdminMetadata::crud_operation("get_wallet_activity", Some("admin".to_string()));

    info!("Admin: Retrieved {} activity events for {}", response.events.len(), wallet_address);
    Ok(Json(AdminApiResponse::success_with_meta(
        response,
        "Activity history retrieved successfully",
        metadata,
    )))
}
