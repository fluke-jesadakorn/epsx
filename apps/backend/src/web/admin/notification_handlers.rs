use axum::{
    extract::{State, Path, Query},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx;

use crate::{
    core::errors::{AppError, ErrorKind},
    web::auth::AppState,
    web::notifications::{SSENotification, NotificationType, NotificationPriority},
};

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SendNotificationRequest {
    pub recipient_wallet_address: Option<String>,
    pub recipient_group: Option<String>,
    pub broadcast: Option<bool>,
    pub notification_type: NotificationType,
    pub priority: NotificationPriority,
    pub title: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub action_url: Option<String>,
    pub image_url: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub schedule_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct SendNotificationResponse {
    pub success: bool,
    pub data: SendNotificationData,
    pub message: String,
    pub api_version: String,
}

#[derive(Debug, Serialize)]
pub struct SendNotificationData {
    pub notification_id: String,
    pub recipients_count: usize,
    pub scheduled: bool,
    pub delivery_status: String,
}

#[derive(Debug, Deserialize)]
pub struct NotificationFilters {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    #[serde(rename = "type")]
    pub notification_type: Option<String>,
    pub priority: Option<String>,
    pub status: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub wallet_address: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct NotificationsResponse {
    pub success: bool,
    pub data: NotificationsData,
    pub api_version: String,
    pub access_level: String,
}

#[derive(Debug, Serialize)]
pub struct NotificationsData {
    pub notifications: Vec<NotificationDto>,
    pub total_count: usize,
    pub unread_count: usize,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct NotificationDto {
    pub id: String,
    pub wallet_address: String,
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub priority: String,
    pub timestamp: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>,
    pub clicked_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub action_url: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct NotificationStatsResponse {
    pub success: bool,
    pub data: NotificationStats,
    pub api_version: String,
    pub access_level: String,
}

#[derive(Debug, Serialize)]
pub struct NotificationStats {
    pub total_notifications: usize,
    pub sent_today: usize,
    pub sent_this_week: usize,
    pub sent_this_month: usize,
    pub delivery_rate: f64,
    pub read_rate: f64,
    pub click_rate: f64,
    pub by_type: serde_json::Value,
    pub by_priority: serde_json::Value,
    pub recent_activity: Vec<RecentActivity>,
}

#[derive(Debug, Serialize)]
pub struct RecentActivity {
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub count: usize,
}

// ============================================================================
// ADMIN HANDLERS
// ============================================================================

/// Send notification to specific user, group, or broadcast
#[utoipa::path(
    post,
    path = "/api/v1/admin/notifications/send",
    tag = "admin-notifications",
    request_body = SendNotificationRequest,
    responses(
        (status = 200, description = "Notification sent successfully"),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = []))
)]
pub async fn send_notification_handler(
    State(app_state): State<AppState>,
    Json(request): Json<SendNotificationRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate request
    if request.title.trim().is_empty() {
        return Err(AppError::new(
            ErrorKind::ValidationError,
            "Notification title cannot be empty".to_string(),
        ));
    }

    if request.message.trim().is_empty() {
        return Err(AppError::new(
            ErrorKind::ValidationError,
            "Notification message cannot be empty".to_string(),
        ));
    }

    // Determine target wallet address
    let wallet_address = if request.broadcast.unwrap_or(false) {
        "all".to_string()
    } else if let Some(addr) = request.recipient_wallet_address {
        addr
    } else if let Some(group) = request.recipient_group {
        // TODO: Fetch wallet addresses for group
        group
    } else {
        return Err(AppError::new(
            ErrorKind::ValidationError,
            "Must specify recipient_wallet_address, recipient_group, or broadcast=true".to_string(),
        ));
    };

    // Create notification ID
    let notification_id = uuid::Uuid::new_v4().to_string();

    // Create SSE notification
    let sse_notification = SSENotification {
        id: notification_id.clone(),
        wallet_address: wallet_address.clone(),
        notification_type: request.notification_type.clone(),
        title: request.title.clone(),
        message: request.message.clone(),
        data: request.data.clone(),
        priority: request.priority.clone(),
        timestamp: Utc::now(),
        expires_at: request.expires_at,
    };

    // Convert notification type and priority to string for database
    let notif_type = format!("{:?}", request.notification_type).to_lowercase();
    let notif_priority = format!("{:?}", request.priority).to_lowercase();

    // Persist to database
    let notification_uuid = uuid::Uuid::parse_str(&notification_id)
        .map_err(|e| AppError::new(ErrorKind::InternalError, format!("Invalid UUID: {}", e)))?;

    sqlx::query!(
        r#"
        INSERT INTO wallet_notifications
        (id, wallet_address, notification_type, title, message, data, priority, timestamp, expires_at, delivered_at, action_url, image_url)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#,
        notification_uuid,
        wallet_address,
        notif_type,
        request.title,
        request.message,
        request.data,
        notif_priority,
        Utc::now(),
        request.expires_at,
        Some(Utc::now()),
        request.action_url,
        request.image_url
    )
    .execute(&*app_state.db_pool)
    .await
    .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to save notification: {}", e)))?;

    // Broadcast via SSE
    let broadcaster = app_state.notification_broadcaster.clone();

    // Attempt to broadcast, but don't fail if no listeners
    let _ = broadcaster.broadcast(sse_notification.clone());

    // Build response
    let response = SendNotificationResponse {
        success: true,
        data: SendNotificationData {
            notification_id,
            recipients_count: if wallet_address == "all" { 1000 } else { 1 }, // TODO: Get actual count
            scheduled: request.schedule_at.is_some(),
            delivery_status: if request.schedule_at.is_some() {
                "scheduled".to_string()
            } else {
                "sent".to_string()
            },
        },
        message: "Notification sent successfully".to_string(),
        api_version: "v1".to_string(),
    };

    Ok(Json(response))
}

/// Get all notifications (admin view with filters)
#[utoipa::path(
    get,
    path = "/api/v1/admin/notifications",
    tag = "admin-notifications",
    params(
        ("page" = Option<u32>, Query, description = "Page number"),
        ("limit" = Option<u32>, Query, description = "Items per page"),
        ("type" = Option<String>, Query, description = "Filter by notification type"),
        ("priority" = Option<String>, Query, description = "Filter by priority"),
        ("status" = Option<String>, Query, description = "Filter by status"),
        ("wallet_address" = Option<String>, Query, description = "Filter by wallet address")
    ),
    responses(
        (status = 200, description = "Notifications retrieved successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = []))
)]
pub async fn get_all_notifications_handler(
    State(app_state): State<AppState>,
    Query(filters): Query<NotificationFilters>,
) -> Result<impl IntoResponse, AppError> {
    let page = filters.page.unwrap_or(1);
    let limit = filters.limit.unwrap_or(20);
    let offset = ((page - 1) * limit) as i64;

    // Build query with filters
    let mut query = String::from("SELECT * FROM wallet_notifications WHERE 1=1");

    if let Some(ref wallet) = filters.wallet_address {
        query.push_str(&format!(" AND wallet_address = '{}'", wallet));
    }
    if let Some(ref notif_type) = filters.notification_type {
        query.push_str(&format!(" AND notification_type = '{}'", notif_type));
    }
    if let Some(ref priority) = filters.priority {
        query.push_str(&format!(" AND priority = '{}'", priority));
    }
    if let Some(ref status) = filters.status {
        match status.as_str() {
            "read" => query.push_str(" AND read_at IS NOT NULL"),
            "unread" => query.push_str(" AND read_at IS NULL"),
            _ => {}
        }
    }

    query.push_str(" ORDER BY timestamp DESC");
    query.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

    let records = sqlx::query_as::<_, (
        uuid::Uuid, String, String, String, String, Option<serde_json::Value>, String,
        DateTime<Utc>, Option<DateTime<Utc>>, Option<DateTime<Utc>>, Option<DateTime<Utc>>,
        Option<DateTime<Utc>>, Option<String>, Option<String>, DateTime<Utc>, DateTime<Utc>
    )>(&query)
        .fetch_all(&*app_state.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to fetch notifications: {}", e)))?;

    let notifications: Vec<NotificationDto> = records.into_iter().map(|r| NotificationDto {
        id: r.0.to_string(),
        wallet_address: r.1,
        notification_type: r.2,
        title: r.3,
        message: r.4,
        data: r.5,
        priority: r.6,
        timestamp: r.7,
        expires_at: r.8,
        read_at: r.9,
        clicked_at: r.10,
        delivered_at: r.11,
        action_url: r.12,
        image_url: r.13,
    }).collect();

    // Get total count
    let total_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM wallet_notifications")
        .fetch_one(&*app_state.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to count notifications: {}", e)))?;

    // Get unread count
    let unread_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM wallet_notifications WHERE read_at IS NULL")
        .fetch_one(&*app_state.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to count unread: {}", e)))?;

    let total_pages = ((total_count.0 as f64) / (limit as f64)).ceil() as u32;

    let response = NotificationsResponse {
        success: true,
        data: NotificationsData {
            notifications,
            total_count: total_count.0 as usize,
            unread_count: unread_count.0 as usize,
            page,
            limit,
            total_pages,
        },
        api_version: "v1".to_string(),
        access_level: "admin".to_string(),
    };

    Ok(Json(response))
}

/// Get notification statistics
#[utoipa::path(
    get,
    path = "/api/v1/admin/notifications/stats",
    tag = "admin-notifications",
    responses(
        (status = 200, description = "Statistics retrieved successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = []))
)]
pub async fn get_notification_stats_handler(
    State(_app_state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    // TODO: Query database for real statistics
    // For now, return sample data to demonstrate the system

    let stats = NotificationStats {
        total_notifications: 156,
        sent_today: 12,
        sent_this_week: 47,
        sent_this_month: 156,
        delivery_rate: 0.98,
        read_rate: 0.75,
        click_rate: 0.42,
        by_type: serde_json::json!({
            "system": 45,
            "security": 23,
            "permission": 12,
            "wallet": 34,
            "payment": 18,
            "general": 24
        }),
        by_priority: serde_json::json!({
            "low": 89,
            "normal": 45,
            "high": 18,
            "urgent": 4
        }),
        recent_activity: vec![
            RecentActivity {
                timestamp: Utc::now() - chrono::Duration::hours(2),
                action: "broadcast_sent".to_string(),
                count: 1,
            },
            RecentActivity {
                timestamp: Utc::now() - chrono::Duration::hours(5),
                action: "notification_sent".to_string(),
                count: 3,
            },
        ],
    };

    let response = NotificationStatsResponse {
        success: true,
        data: stats,
        api_version: "v1".to_string(),
        access_level: "admin".to_string(),
    };

    Ok(Json(response))
}

// ============================================================================
// USER HANDLERS (Authenticated)
// ============================================================================

/// Get user notifications (authenticated user only)
#[utoipa::path(
    get,
    path = "/api/v1/auth/notifications",
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
    Query(filters): Query<NotificationFilters>,
) -> Result<impl IntoResponse, AppError> {
    // TODO: Extract wallet_address from Bearer token/session
    // For now, return all notifications (broadcast to 'all')

    let page = filters.page.unwrap_or(1);
    let limit = filters.limit.unwrap_or(20);
    let offset = ((page - 1) * limit) as i64;

    // Build query - get notifications sent to 'all' (broadcast)
    // In production, you'd also filter by specific wallet_address from auth
    let mut query = String::from("SELECT * FROM wallet_notifications WHERE wallet_address = 'all'");

    if let Some(ref notif_type) = filters.notification_type {
        query.push_str(&format!(" AND notification_type = '{}'", notif_type));
    }
    if let Some(ref priority) = filters.priority {
        query.push_str(&format!(" AND priority = '{}'", priority));
    }
    if let Some(ref status) = filters.status {
        match status.as_str() {
            "read" => query.push_str(" AND read_at IS NOT NULL"),
            "unread" => query.push_str(" AND read_at IS NULL"),
            _ => {}
        }
    }

    query.push_str(" ORDER BY timestamp DESC");
    query.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

    let records = sqlx::query_as::<_, (
        uuid::Uuid, String, String, String, String, Option<serde_json::Value>, String,
        DateTime<Utc>, Option<DateTime<Utc>>, Option<DateTime<Utc>>, Option<DateTime<Utc>>,
        Option<DateTime<Utc>>, Option<String>, Option<String>, DateTime<Utc>, DateTime<Utc>
    )>(&query)
        .fetch_all(&*app_state.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to fetch notifications: {}", e)))?;

    let notifications: Vec<NotificationDto> = records.into_iter().map(|r| NotificationDto {
        id: r.0.to_string(),
        wallet_address: r.1,
        notification_type: r.2,
        title: r.3,
        message: r.4,
        data: r.5,
        priority: r.6,
        timestamp: r.7,
        expires_at: r.8,
        read_at: r.9,
        clicked_at: r.10,
        delivered_at: r.11,
        action_url: r.12,
        image_url: r.13,
    }).collect();

    // Get total count for this user
    let total_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM wallet_notifications WHERE wallet_address = 'all'")
        .fetch_one(&*app_state.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to count notifications: {}", e)))?;

    // Get unread count for this user
    let unread_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM wallet_notifications WHERE wallet_address = 'all' AND read_at IS NULL")
        .fetch_one(&*app_state.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to count unread: {}", e)))?;

    let total_pages = ((total_count.0 as f64) / (limit as f64)).ceil() as u32;

    let response = NotificationsResponse {
        success: true,
        data: NotificationsData {
            notifications,
            total_count: total_count.0 as usize,
            unread_count: unread_count.0 as usize,
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
    path = "/api/v1/auth/notifications/{id}/read",
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
    Path(notification_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // TODO: Extract wallet_address from Bearer token/session for authorization

    let notif_uuid = uuid::Uuid::parse_str(&notification_id)
        .map_err(|e| AppError::new(ErrorKind::ValidationError, format!("Invalid notification ID: {}", e)))?;

    let result = sqlx::query!(
        r#"
        UPDATE wallet_notifications
        SET read_at = $1, updated_at = $1
        WHERE id = $2
        "#,
        Utc::now(),
        notif_uuid
    )
    .execute(&*app_state.db_pool)
    .await
    .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to mark notification as read: {}", e)))?;

    if result.rows_affected() == 0 {
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

/// Delete user notification
#[utoipa::path(
    delete,
    path = "/api/v1/auth/notifications/{id}",
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
    Path(notification_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // TODO: Extract wallet_address from Bearer token/session for authorization

    let notif_uuid = uuid::Uuid::parse_str(&notification_id)
        .map_err(|e| AppError::new(ErrorKind::ValidationError, format!("Invalid notification ID: {}", e)))?;

    let result = sqlx::query!(
        r#"
        DELETE FROM wallet_notifications
        WHERE id = $1
        "#,
        notif_uuid
    )
    .execute(&*app_state.db_pool)
    .await
    .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to delete notification: {}", e)))?;

    if result.rows_affected() == 0 {
        return Err(AppError::new(
            ErrorKind::AggregateNotFound,
            "Notification not found".to_string(),
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
    path = "/api/v1/auth/notifications/unread-count",
    tag = "notifications",
    responses(
        (status = 200, description = "Unread count retrieved successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = []))
)]
pub async fn get_unread_count_handler(
    State(_app_state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    // TODO: Extract wallet_address from Bearer token/session
    // TODO: Query database for unread notification count

    Ok(Json(serde_json::json!({
        "unread_count": 0
    })))
}
