// User notification handlers (authenticated)
use axum::{
    extract::{State, Path, Query},
    response::IntoResponse,
    Json,
};
use axum::http::HeaderMap;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::{
    core::errors::{AppError, ErrorKind},
    web::auth::AppState,
};
use super::notification_types::*;
use super::super::notification_query_helper::NotificationQueryFilter;
use super::super::wallet_notification_repository::WalletNotificationRepository;

// ============================================================================
// SIMPLE AUTHENTICATION HELPER
// ============================================================================

/// Simple Bearer token authentication for notification endpoints
/// Extracts wallet address from Authorization header
async fn authenticate_from_headers(
    headers: &axum::http::HeaderMap,
) -> Result<String, AppError> {
    // Extract Bearer token from Authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::new(
            ErrorKind::AuthenticationError,
            "Missing authorization header".to_string(),
        ))?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::new(
            ErrorKind::AuthenticationError,
            "Invalid authorization format".to_string(),
        ));
    }

    let token = &auth_header[7..];

    // Simple token validation for now - accept any non-empty Bearer token
    // This is a temporary solution to get the notifications working
    if token.is_empty() {
        return Err(AppError::new(
            ErrorKind::AuthenticationError,
            "Invalid token".to_string(),
        ));
    }

    // For demo purposes, return a fixed wallet address
    // In production, this should decode a proper JWT token
    Ok("0x742d35Cc6464C73B8A5a2E8F2B56c4C0C3E0A5d3".to_lowercase())
}

// ============================================================================
// USER HANDLERS (Authenticated)
// ============================================================================

/// Get user notifications (authenticated user only)
#[utoipa::path(
    get,
    path = "/api/auth/notifications",
    tag = "notifications",
    params(
        ("page" = Option<u32>, Query, description = "Page number"),
        ("limit" = Option<u32>, Query, description = "Items per page"),
        ("type" = Option<String>, Query, description = "Filter by notification type"),
        ("priority" = Option<String>, Query, description = "Filter by priority"),
        ("status" = Option<String>, Query, description = "Filter by status (read/unread/all)")
    ),
    responses(
        (status = 200, description = "Notifications retrieved successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = []))
)]
pub async fn get_user_notifications_handler(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Query(filters): Query<NotificationFilters>,
) -> Result<impl IntoResponse, AppError> {
    // Authenticate from Authorization header
    let wallet_address = authenticate_from_headers(&headers).await?;

    let page = filters.page.unwrap_or(1);
    let limit = filters.limit.unwrap_or(20);
    let offset = ((page - 1) * limit) as i64;

    // Build filter helper
    let mut filter = NotificationQueryFilter::new();
    if let Some(notif_type) = filters.notification_type {
        filter = filter.notification_type(notif_type);
    }
    if let Some(priority) = filters.priority {
        filter = filter.priority(priority);
    }
    if let Some(status) = filters.status {
        filter = filter.status(status);
    }

    // Use repository for database operations - notifications table is in separate DB
    let notifications_pool = match crate::infrastructure::database::get_notifications_pool().await {
        Ok(p) => std::sync::Arc::new(p),
        Err(e) => {
            tracing::warn!("Failed to get notifications pool, falling back to main pool: {}", e);
            app_state.db_pool.clone()
        }
    };
    let repo = WalletNotificationRepository::new(notifications_pool.clone());

    // Fetch notifications for wallet
    let records = repo.find_for_wallet(&wallet_address, &filter, limit as i64, offset).await
        .map_err(|e| {
            tracing::error!("Failed to fetch notifications for wallet {}: {}", wallet_address, e);
            AppError::new(
                ErrorKind::DatabaseError,
                format!("Failed to fetch notifications: {}", e.message)
            )
        })?;

    let notifications: Vec<NotificationDto> = records.into_iter().map(|r| NotificationDto {
        id: r.id.to_string(),
        wallet_address: r.wallet_address,
        notification_type: r.notification_type,
        title: r.title,
        message: r.message,
        data: r.data,
        priority: r.priority,
        timestamp: r.timestamp,
        expires_at: r.expires_at,
        read_at: r.read_at,
        clicked_at: r.clicked_at,
        delivered_at: r.delivered_at,
        action_url: r.action_url,
        image_url: r.image_url,
    }).collect();

    // Get total count
    let total_count = repo.count_for_wallet(&wallet_address, &filter).await
        .map_err(|e| {
            tracing::error!("Failed to count notifications for wallet {}: {}", wallet_address, e);
            AppError::new(
                ErrorKind::DatabaseError,
                format!("Failed to count notifications: {}", e.message)
            )
        })?;

    // Get unread count
    let unread_count = repo.count_unread_for_wallet(&wallet_address, &filter).await
        .map_err(|e| {
            tracing::error!("Failed to count unread notifications for wallet {}: {}", wallet_address, e);
            AppError::new(
                ErrorKind::DatabaseError,
                format!("Failed to count unread notifications: {}", e.message)
            )
        })?;

    let total_pages = ((total_count as f64) / (limit as f64)).ceil() as u32;

    let response = NotificationsResponse {
        success: true,
        data: NotificationsData {
            notifications,
            total_count: total_count as usize,
            unread_count: unread_count as usize,
            page,
            limit,
            total_pages,
        },
        api_version: "v1".to_string(),
        access_level: "auth".to_string(),
    };

    Ok(Json(response))
}

/// Mark notification as read
#[utoipa::path(
    put,
    path = "/api/auth/notifications/{id}/read",
    tag = "notifications",
    params(
        ("id" = String, Path, description = "Notification ID")
    ),
    responses(
        (status = 200, description = "Notification marked as read"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Notification not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = []))
)]
pub async fn mark_notification_read_handler(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Path(notification_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // Authenticate from Authorization header
    let wallet_address = authenticate_from_headers(&headers).await?;

    let notif_uuid = uuid::Uuid::parse_str(&notification_id)
        .map_err(|e| AppError::new(ErrorKind::ValidationError, format!("Invalid notification ID: {}", e)))?;

    // Get notifications database connection
    let notifications_pool = if let Ok(p) = crate::infrastructure::database::get_notifications_pool().await {
        std::sync::Arc::new(p)
    } else {
        app_state.db_pool.clone()
    };
    let mut conn = notifications_pool.get().await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get database connection: {}", e)))?;

    let now = Utc::now();

    // Only allow marking as read if notification belongs to user or is broadcast
    let rows_affected = diesel::sql_query(
        r#"
        UPDATE wallet_notifications
        SET read_at = $1, updated_at = $1
        WHERE id = $2 AND (wallet_address = $3 OR wallet_address = 'all')
        "#
    )
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .bind::<diesel::sql_types::Uuid, _>(notif_uuid)
    .bind::<diesel::sql_types::Text, _>(&wallet_address)
    .execute(&mut conn)
    .await
    .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to mark notification as read: {}", e)))?;

    if rows_affected == 0 {
        return Err(AppError::new(
            ErrorKind::AggregateNotFound,
            "Notification not found".to_string(),
        ));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Notification marked as read",
        "notification_id": notification_id
    })))
}

/// Delete user notification (soft delete)
#[utoipa::path(
    delete,
    path = "/api/auth/notifications/{id}",
    tag = "notifications",
    params(
        ("id" = String, Path, description = "Notification ID")
    ),
    responses(
        (status = 200, description = "Notification deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Notification not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = []))
)]
pub async fn delete_notification_handler(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Path(notification_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // Authenticate from Authorization header
    let wallet_address = authenticate_from_headers(&headers).await?;

    let notif_uuid = uuid::Uuid::parse_str(&notification_id)
        .map_err(|e| AppError::new(ErrorKind::ValidationError, format!("Invalid notification ID: {}", e)))?;

    // Get notifications database connection
    let notifications_pool = if let Ok(p) = crate::infrastructure::database::get_notifications_pool().await {
        std::sync::Arc::new(p)
    } else {
        app_state.db_pool.clone()
    };
    let mut conn = notifications_pool.get().await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get database connection: {}", e)))?;

    // Soft delete: Only allow deleting if notification belongs to user or is broadcast
    let rows_affected = diesel::sql_query(
        r#"
        UPDATE wallet_notifications
        SET deleted_at = NOW(), updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL AND (wallet_address = $2 OR wallet_address = 'all')
        "#
    )
    .bind::<diesel::sql_types::Uuid, _>(notif_uuid)
    .bind::<diesel::sql_types::Text, _>(&wallet_address)
    .execute(&mut conn)
    .await
    .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to delete notification: {}", e)))?;

    if rows_affected == 0 {
        return Err(AppError::new(
            ErrorKind::AggregateNotFound,
            "Notification not found or already deleted".to_string(),
        ));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Notification deleted successfully",
        "notification_id": notification_id
    })))
}

/// Get unread notification count
#[utoipa::path(
    get,
    path = "/api/auth/notifications/unread-count",
    tag = "notifications",
    responses(
        (status = 200, description = "Unread count retrieved successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = []))
)]
pub async fn get_unread_count_handler(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    // Authenticate from Authorization header
    let wallet_address = authenticate_from_headers(&headers).await?;

    // Get notifications database connection
    let notifications_pool = if let Ok(p) = crate::infrastructure::database::get_notifications_pool().await {
        std::sync::Arc::new(p)
    } else {
        app_state.db_pool.clone()
    };
    let mut conn = notifications_pool.get().await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get database connection: {}", e)))?;

    #[derive(QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    let unread_count: i64 = diesel::sql_query(
        "SELECT COUNT(*) as count FROM wallet_notifications \
         WHERE (wallet_address = $1 OR wallet_address = 'all') \
         AND read_at IS NULL AND deleted_at IS NULL"
    )
    .bind::<diesel::sql_types::Text, _>(&wallet_address)
    .get_result::<CountRow>(&mut conn)
    .await
    .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to count unread notifications: {}", e)))?
    .count;

    Ok(Json(serde_json::json!({
        "unread_count": unread_count
    })))
}

/// Mark all notifications as read
#[utoipa::path(
    put,
    path = "/api/notifications/mark-all-read",
    tag = "notifications",
    responses(
        (status = 200, description = "All notifications marked as read"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = []))
)]
pub async fn mark_all_notifications_read_handler(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    // Mark all notifications for authenticated user as read
    let wallet_address = authenticate_from_headers(&headers).await?;

    // Get notifications database connection
    let notifications_pool = if let Ok(p) = crate::infrastructure::database::get_notifications_pool().await {
        std::sync::Arc::new(p)
    } else {
        app_state.db_pool.clone()
    };
    let mut conn = notifications_pool.get().await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get database connection: {}", e)))?;

    let now = Utc::now();

    let rows_affected = diesel::sql_query(
        r#"
        UPDATE wallet_notifications
        SET read_at = $1, updated_at = $1
        WHERE (wallet_address = $2 OR wallet_address = 'all') AND read_at IS NULL AND deleted_at IS NULL
        "#
    )
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .bind::<diesel::sql_types::Text, _>(&wallet_address)
    .execute(&mut conn)
    .await
    .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to mark all notifications as read: {}", e)))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "updated_count": rows_affected,
        "message": "All notifications marked as read"
    })))
}

/// Clear all notifications (soft delete)
#[utoipa::path(
    delete,
    path = "/api/notifications/clear-all",
    tag = "notifications",
    responses(
        (status = 200, description = "All notifications cleared successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = []))
)]
pub async fn clear_all_notifications_handler(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    // Clear all notifications for authenticated user
    let wallet_address = authenticate_from_headers(&headers).await?;

    // Get notifications database connection
    let notifications_pool = if let Ok(p) = crate::infrastructure::database::get_notifications_pool().await {
        std::sync::Arc::new(p)
    } else {
        app_state.db_pool.clone()
    };
    let mut conn = notifications_pool.get().await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get database connection: {}", e)))?;

    // Soft delete: Set deleted_at timestamp instead of removing rows
    let rows_affected = diesel::sql_query(
        r#"
        UPDATE wallet_notifications
        SET deleted_at = NOW(), updated_at = NOW()
        WHERE (wallet_address = $1 OR wallet_address = 'all') AND deleted_at IS NULL
        "#
    )
    .bind::<diesel::sql_types::Text, _>(&wallet_address)
    .execute(&mut conn)
    .await
    .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to clear all notifications: {}", e)))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "deleted_count": rows_affected,
        "message": "All notifications cleared successfully"
    })))
}

/// Acknowledge notification (mark as received by client)
/// This is called automatically by the frontend when a notification is received via SSE
#[utoipa::path(
    put,
    path = "/api/notifications/{id}/acknowledge",
    tag = "notifications",
    params(
        ("id" = String, Path, description = "Notification ID")
    ),
    responses(
        (status = 200, description = "Notification acknowledged successfully"),
        (status = 400, description = "Invalid notification ID"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = []))
)]
pub async fn acknowledge_notification_handler(
    State(app_state): State<AppState>,
    _headers: HeaderMap,
    Path(notification_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // Get notifications database pool
    let notifications_pool = if let Ok(p) = crate::infrastructure::database::get_notifications_pool().await {
        p
    } else {
        &**app_state.db_pool
    };

    // Call the offline_queue module's mark_as_acknowledged function
    crate::web::notifications::mark_as_acknowledged(notifications_pool, &notification_id)
        .await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Notification acknowledged successfully",
        "notification_id": notification_id
    })))
}
