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
    web::auth::{AppState, wallet_extractor::AuthWallet},
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

    // Determine target wallet address (convert to lowercase for consistency)
    let wallet_address = if request.broadcast.unwrap_or(false) {
        "all".to_string()
    } else if let Some(addr) = request.recipient_wallet_address {
        addr.to_lowercase()
    } else if let Some(group) = request.recipient_group {
        // TODO: Fetch wallet addresses for group
        group.to_lowercase()
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

    // Publish via Redis pub/sub (if available)
    let subscriber_count = if let Some(redis_broadcaster) = &app_state.redis_broadcaster {
        if wallet_address == "all" {
            redis_broadcaster.publish_to_all(&sse_notification).await?
        } else {
            redis_broadcaster.publish_to_wallet(&wallet_address, &sse_notification).await?
        }
    } else {
        tracing::warn!("Redis not available - notification saved to database but not broadcast in real-time");
        0 // No subscribers since Redis is not available
    };

    // Update delivery attempt in database
    sqlx::query!(
        "UPDATE wallet_notifications SET delivery_attempts = 1, last_delivery_attempt_at = NOW() WHERE id = $1",
        notification_uuid
    )
    .execute(&*app_state.db_pool)
    .await?;

    // Build response
    let delivery_message = if app_state.redis_broadcaster.is_some() {
        "Notification sent successfully via Redis".to_string()
    } else {
        "Notification saved to database (Redis unavailable - no real-time broadcast)".to_string()
    };

    let response = SendNotificationResponse {
        success: true,
        data: SendNotificationData {
            notification_id,
            recipients_count: subscriber_count,
            scheduled: request.schedule_at.is_some(),
            delivery_status: if request.schedule_at.is_some() {
                "scheduled".to_string()
            } else if subscriber_count > 0 {
                "sent".to_string()
            } else {
                "queued".to_string() // No active subscribers, will deliver when they connect
            },
        },
        message: delivery_message,
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

    // Build parameterized query with filters (exclude soft-deleted notifications)
    let mut query = sqlx::QueryBuilder::new(
        "SELECT id, wallet_address, notification_type, title, message, data, priority, \
         timestamp, expires_at, read_at, clicked_at, delivered_at, action_url, image_url, \
         created_at, updated_at \
         FROM wallet_notifications WHERE deleted_at IS NULL"
    );

    if let Some(ref wallet) = filters.wallet_address {
        query.push(" AND wallet_address = ");
        query.push_bind(wallet);
    }
    if let Some(ref notif_type) = filters.notification_type {
        query.push(" AND notification_type = ");
        query.push_bind(notif_type);
    }
    if let Some(ref priority) = filters.priority {
        query.push(" AND priority = ");
        query.push_bind(priority);
    }
    if let Some(ref status) = filters.status {
        if status == "read" {
            query.push(" AND read_at IS NOT NULL");
        } else if status == "unread" {
            query.push(" AND read_at IS NULL");
        }
    }

    query.push(" ORDER BY timestamp DESC LIMIT ");
    query.push_bind(limit as i64);
    query.push(" OFFSET ");
    query.push_bind(offset);

    let records = query
        .build_query_as::<(
            uuid::Uuid, String, String, String, String, Option<serde_json::Value>, String,
            DateTime<Utc>, Option<DateTime<Utc>>, Option<DateTime<Utc>>, Option<DateTime<Utc>>,
            Option<DateTime<Utc>>, Option<String>, Option<String>, DateTime<Utc>, DateTime<Utc>
        )>()
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

    // Get total count (exclude soft-deleted) - with same filters
    let mut count_query = sqlx::QueryBuilder::new(
        "SELECT COUNT(*) FROM wallet_notifications WHERE deleted_at IS NULL"
    );

    if let Some(ref wallet) = filters.wallet_address {
        count_query.push(" AND wallet_address = ");
        count_query.push_bind(wallet);
    }
    if let Some(ref notif_type) = filters.notification_type {
        count_query.push(" AND notification_type = ");
        count_query.push_bind(notif_type);
    }
    if let Some(ref priority) = filters.priority {
        count_query.push(" AND priority = ");
        count_query.push_bind(priority);
    }
    if let Some(ref status) = filters.status {
        if status == "read" {
            count_query.push(" AND read_at IS NOT NULL");
        } else if status == "unread" {
            count_query.push(" AND read_at IS NULL");
        }
    }

    let total_count: (i64,) = count_query
        .build_query_as()
        .fetch_one(&*app_state.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to count notifications: {}", e)))?;

    // Get unread count (exclude soft-deleted)
    let mut unread_query = sqlx::QueryBuilder::new(
        "SELECT COUNT(*) FROM wallet_notifications WHERE read_at IS NULL AND deleted_at IS NULL"
    );

    if let Some(ref wallet) = filters.wallet_address {
        unread_query.push(" AND wallet_address = ");
        unread_query.push_bind(wallet);
    }
    if let Some(ref notif_type) = filters.notification_type {
        unread_query.push(" AND notification_type = ");
        unread_query.push_bind(notif_type);
    }
    if let Some(ref priority) = filters.priority {
        unread_query.push(" AND priority = ");
        unread_query.push_bind(priority);
    }

    let unread_count: (i64,) = unread_query
        .build_query_as()
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
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    // Get total notifications count (exclude soft-deleted)
    let total_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM wallet_notifications WHERE deleted_at IS NULL")
        .fetch_one(&*app_state.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to count notifications: {}", e)))?;

    // Get notifications sent today (exclude soft-deleted)
    let today_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM wallet_notifications WHERE timestamp >= CURRENT_DATE AND deleted_at IS NULL"
    )
        .fetch_one(&*app_state.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to count today's notifications: {}", e)))?;

    // Get notifications sent this week (exclude soft-deleted)
    let week_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM wallet_notifications WHERE timestamp >= CURRENT_DATE - INTERVAL '7 days' AND deleted_at IS NULL"
    )
        .fetch_one(&*app_state.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to count week's notifications: {}", e)))?;

    // Get notifications sent this month (exclude soft-deleted)
    let month_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM wallet_notifications WHERE timestamp >= CURRENT_DATE - INTERVAL '30 days' AND deleted_at IS NULL"
    )
        .fetch_one(&*app_state.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to count month's notifications: {}", e)))?;

    // Get count by type (exclude soft-deleted)
    let type_counts = sqlx::query_as::<_, (String, i64)>(
        "SELECT notification_type, COUNT(*) as count FROM wallet_notifications WHERE deleted_at IS NULL GROUP BY notification_type"
    )
        .fetch_all(&*app_state.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get type counts: {}", e)))?;

    let mut by_type = serde_json::Map::new();
    for (notif_type, count) in type_counts {
        by_type.insert(notif_type, serde_json::json!(count));
    }

    // Get count by priority (exclude soft-deleted)
    let priority_counts = sqlx::query_as::<_, (String, i64)>(
        "SELECT priority, COUNT(*) as count FROM wallet_notifications WHERE deleted_at IS NULL GROUP BY priority"
    )
        .fetch_all(&*app_state.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get priority counts: {}", e)))?;

    let mut by_priority = serde_json::Map::new();
    for (priority, count) in priority_counts {
        by_priority.insert(priority, serde_json::json!(count));
    }

    // Calculate delivery rate (all active notifications are delivered)
    let delivery_rate = if total_count.0 > 0 { 1.0 } else { 0.0 };

    // Calculate read rate (exclude soft-deleted)
    let read_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM wallet_notifications WHERE read_at IS NOT NULL AND deleted_at IS NULL"
    )
        .fetch_one(&*app_state.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to count read notifications: {}", e)))?;

    let read_rate = if total_count.0 > 0 {
        (read_count.0 as f64) / (total_count.0 as f64)
    } else {
        0.0
    };

    // Calculate click rate (exclude soft-deleted)
    let clicked_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM wallet_notifications WHERE clicked_at IS NOT NULL AND deleted_at IS NULL"
    )
        .fetch_one(&*app_state.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to count clicked notifications: {}", e)))?;

    let click_rate = if total_count.0 > 0 {
        (clicked_count.0 as f64) / (total_count.0 as f64)
    } else {
        0.0
    };

    // Get recent activity (last 24 hours, grouped by hour, exclude soft-deleted)
    let recent_activity_records = sqlx::query_as::<_, (DateTime<Utc>, i64)>(
        r#"
        SELECT
            DATE_TRUNC('hour', timestamp) as hour,
            COUNT(*) as count
        FROM wallet_notifications
        WHERE timestamp >= NOW() - INTERVAL '24 hours'
          AND deleted_at IS NULL
        GROUP BY hour
        ORDER BY hour DESC
        LIMIT 10
        "#
    )
        .fetch_all(&*app_state.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get recent activity: {}", e)))?;

    let recent_activity: Vec<RecentActivity> = recent_activity_records
        .into_iter()
        .map(|(timestamp, count)| RecentActivity {
            timestamp,
            action: "notification_sent".to_string(),
            count: count as usize,
        })
        .collect();

    let stats = NotificationStats {
        total_notifications: total_count.0 as usize,
        sent_today: today_count.0 as usize,
        sent_this_week: week_count.0 as usize,
        sent_this_month: month_count.0 as usize,
        delivery_rate,
        read_rate,
        click_rate,
        by_type: serde_json::Value::Object(by_type),
        by_priority: serde_json::Value::Object(by_priority),
        recent_activity,
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
    auth_wallet: AuthWallet,
    Query(filters): Query<NotificationFilters>,
) -> Result<impl IntoResponse, AppError> {
    // Use authenticated wallet address from Bearer token
    let wallet_address = auth_wallet.address.to_lowercase();

    let page = filters.page.unwrap_or(1);
    let limit = filters.limit.unwrap_or(20);
    let offset = ((page - 1) * limit) as i64;

    // Build parameterized query - get notifications sent to authenticated wallet OR broadcast to 'all'
    let mut query = sqlx::QueryBuilder::new(
        "SELECT id, wallet_address, notification_type, title, message, data, priority, \
         timestamp, expires_at, read_at, clicked_at, delivered_at, action_url, image_url, \
         created_at, updated_at \
         FROM wallet_notifications WHERE deleted_at IS NULL AND ("
    );

    query.push("wallet_address = ");
    query.push_bind(&wallet_address);
    query.push(" OR wallet_address = 'all')");

    if let Some(ref notif_type) = filters.notification_type {
        query.push(" AND notification_type = ");
        query.push_bind(notif_type);
    }
    if let Some(ref priority) = filters.priority {
        query.push(" AND priority = ");
        query.push_bind(priority);
    }
    if let Some(ref status) = filters.status {
        if status == "read" {
            query.push(" AND read_at IS NOT NULL");
        } else if status == "unread" {
            query.push(" AND read_at IS NULL");
        }
    }

    query.push(" ORDER BY timestamp DESC LIMIT ");
    query.push_bind(limit as i64);
    query.push(" OFFSET ");
    query.push_bind(offset);

    let records = query
        .build_query_as::<(
            uuid::Uuid, String, String, String, String, Option<serde_json::Value>, String,
            DateTime<Utc>, Option<DateTime<Utc>>, Option<DateTime<Utc>>, Option<DateTime<Utc>>,
            Option<DateTime<Utc>>, Option<String>, Option<String>, DateTime<Utc>, DateTime<Utc>
        )>()
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

    // Get total count for authenticated user (exclude soft-deleted)
    let mut count_query = sqlx::QueryBuilder::new(
        "SELECT COUNT(*) FROM wallet_notifications WHERE deleted_at IS NULL AND ("
    );

    count_query.push("wallet_address = ");
    count_query.push_bind(&wallet_address);
    count_query.push(" OR wallet_address = 'all')");

    if let Some(ref notif_type) = filters.notification_type {
        count_query.push(" AND notification_type = ");
        count_query.push_bind(notif_type);
    }
    if let Some(ref priority) = filters.priority {
        count_query.push(" AND priority = ");
        count_query.push_bind(priority);
    }
    if let Some(ref status) = filters.status {
        if status == "read" {
            count_query.push(" AND read_at IS NOT NULL");
        } else if status == "unread" {
            count_query.push(" AND read_at IS NULL");
        }
    }

    let total_count: (i64,) = count_query
        .build_query_as()
        .fetch_one(&*app_state.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to count notifications: {}", e)))?;

    // Get unread count for authenticated user (exclude soft-deleted)
    let mut unread_query = sqlx::QueryBuilder::new(
        "SELECT COUNT(*) FROM wallet_notifications WHERE deleted_at IS NULL AND read_at IS NULL AND ("
    );

    unread_query.push("wallet_address = ");
    unread_query.push_bind(&wallet_address);
    unread_query.push(" OR wallet_address = 'all')");

    if let Some(ref notif_type) = filters.notification_type {
        unread_query.push(" AND notification_type = ");
        unread_query.push_bind(notif_type);
    }
    if let Some(ref priority) = filters.priority {
        unread_query.push(" AND priority = ");
        unread_query.push_bind(priority);
    }

    let unread_count: (i64,) = unread_query
        .build_query_as()
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
    auth_wallet: AuthWallet,
    Path(notification_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // Use authenticated wallet address for authorization
    let wallet_address = auth_wallet.address.to_lowercase();

    let notif_uuid = uuid::Uuid::parse_str(&notification_id)
        .map_err(|e| AppError::new(ErrorKind::ValidationError, format!("Invalid notification ID: {}", e)))?;

    // Only allow marking as read if notification belongs to user or is broadcast
    let result = sqlx::query!(
        r#"
        UPDATE wallet_notifications
        SET read_at = $1, updated_at = $1
        WHERE id = $2 AND (wallet_address = $3 OR wallet_address = 'all')
        "#,
        Utc::now(),
        notif_uuid,
        wallet_address
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

/// Delete user notification (soft delete)
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
    auth_wallet: AuthWallet,
    Path(notification_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // Use authenticated wallet address for authorization
    let wallet_address = auth_wallet.address.to_lowercase();

    let notif_uuid = uuid::Uuid::parse_str(&notification_id)
        .map_err(|e| AppError::new(ErrorKind::ValidationError, format!("Invalid notification ID: {}", e)))?;

    // Soft delete: Only allow deleting if notification belongs to user or is broadcast
    let result = sqlx::query!(
        r#"
        UPDATE wallet_notifications
        SET deleted_at = NOW(), updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL AND (wallet_address = $2 OR wallet_address = 'all')
        "#,
        notif_uuid,
        wallet_address
    )
    .execute(&*app_state.db_pool)
    .await
    .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to delete notification: {}", e)))?;

    if result.rows_affected() == 0 {
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
    State(app_state): State<AppState>,
    auth_wallet: AuthWallet,
) -> Result<impl IntoResponse, AppError> {
    // Get unread count for authenticated user
    let wallet_address = auth_wallet.address.to_lowercase();

    let unread_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM wallet_notifications \
         WHERE (wallet_address = $1 OR wallet_address = 'all') \
         AND read_at IS NULL AND deleted_at IS NULL"
    )
    .bind(&wallet_address)
    .fetch_one(&*app_state.db_pool)
    .await
    .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to count unread notifications: {}", e)))?;

    Ok(Json(serde_json::json!({
        "unread_count": unread_count.0
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
    auth_wallet: AuthWallet,
) -> Result<impl IntoResponse, AppError> {
    // Mark all notifications for authenticated user as read
    let wallet_address = auth_wallet.address.to_lowercase();

    let result = sqlx::query!(
        r#"
        UPDATE wallet_notifications
        SET read_at = $1, updated_at = $1
        WHERE (wallet_address = $2 OR wallet_address = 'all') AND read_at IS NULL AND deleted_at IS NULL
        "#,
        Utc::now(),
        wallet_address
    )
    .execute(&*app_state.db_pool)
    .await
    .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to mark all notifications as read: {}", e)))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "updated_count": result.rows_affected(),
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
    auth_wallet: AuthWallet,
) -> Result<impl IntoResponse, AppError> {
    // Clear all notifications for authenticated user
    let wallet_address = auth_wallet.address.to_lowercase();

    // Soft delete: Set deleted_at timestamp instead of removing rows
    let result = sqlx::query!(
        r#"
        UPDATE wallet_notifications
        SET deleted_at = NOW(), updated_at = NOW()
        WHERE (wallet_address = $1 OR wallet_address = 'all') AND deleted_at IS NULL
        "#,
        wallet_address
    )
    .execute(&*app_state.db_pool)
    .await
    .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to clear all notifications: {}", e)))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "deleted_count": result.rows_affected(),
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
    Path(notification_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // Call the offline_queue module's mark_as_acknowledged function
    crate::web::notifications::mark_as_acknowledged(&*app_state.db_pool, &notification_id)
        .await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Notification acknowledged successfully",
        "notification_id": notification_id
    })))
}
