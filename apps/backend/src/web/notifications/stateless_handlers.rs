use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::{delete, get, patch, post},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::core::errors::AppError;
use crate::infrastructure::container::AppContainer;

type ApiResult<T> = Result<T, AppError>;
type ApiError = AppError;

/// Notification data structure
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub message: String,
    pub notification_type: String,
    pub category: Option<String>,
    pub priority: String,
    pub is_urgent: bool,
    pub is_read: bool,
    pub read_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_archived: bool,
    pub archived_at: Option<chrono::DateTime<chrono::Utc>>,
    pub data: Option<serde_json::Value>,
    pub action_url: Option<String>,
    pub action_label: Option<String>,
    pub icon: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Request to create a new notification
#[derive(Debug, Deserialize)]
pub struct CreateNotificationRequest {
    pub user_id: Uuid,
    pub title: String,
    pub message: String,
    pub notification_type: String, // 'info', 'warning', 'error', 'success', 'security'
    pub category: Option<String>,   // 'account', 'trading', 'system', 'security', 'promotion'
    pub priority: Option<String>,   // 'low', 'normal', 'high', 'urgent'
    pub is_urgent: Option<bool>,
    pub data: Option<serde_json::Value>,
    pub action_url: Option<String>,
    pub action_label: Option<String>,
    pub icon: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Request to update notification status
#[derive(Debug, Deserialize)]
pub struct UpdateNotificationRequest {
    pub is_read: Option<bool>,
    pub is_archived: Option<bool>,
}

/// Request to bulk update notifications
#[derive(Debug, Deserialize)]
pub struct BulkUpdateRequest {
    pub notification_ids: Vec<Uuid>,
    pub action: String, // 'mark_read', 'mark_unread', 'archive', 'unarchive', 'delete'
}

/// Notification preferences
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct NotificationPreferences {
    pub notifications_enabled: bool,
    pub email_notifications: bool,
    pub account_notifications: bool,
    pub trading_notifications: bool,
    pub system_notifications: bool,
    pub security_notifications: bool,
    pub promotion_notifications: bool,
    pub minimum_priority: String,
    pub urgent_only: bool,
    pub auto_archive_days: i32,
    pub auto_delete_days: i32,
}

/// Query parameters for listing notifications
#[derive(Debug, Deserialize)]
pub struct NotificationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub unread_only: Option<bool>,
    pub category: Option<String>,
    pub notification_type: Option<String>,
    pub priority: Option<String>,
    pub urgent_only: Option<bool>,
    pub include_archived: Option<bool>,
}

/// Response for notification counts
#[derive(Debug, Serialize)]
pub struct NotificationCountResponse {
    pub total: i64,
    pub unread: i64,
    pub urgent: i64,
    pub by_category: HashMap<String, i64>,
    pub by_priority: HashMap<String, i64>,
}

/// Create stateless notification routes
pub fn create_routes() -> Router<AppContainer> {
    Router::new()
        .route("/", get(list_notifications))
        .route("/", post(create_notification))
        .route("/:id", get(get_notification))
        .route("/:id", patch(update_notification))
        .route("/:id", delete(delete_notification))
        .route("/bulk", post(bulk_update_notifications))
        .route("/count", get(get_notification_count))
        .route("/preferences", get(get_preferences))
        .route("/preferences", post(update_preferences))
        .route("/cleanup", post(cleanup_notifications))
}

/// List notifications for a user
/// GET /api/notifications?user_id=uuid&limit=20&offset=0&unread_only=true
pub async fn list_notifications(
    State(container): State<AppContainer>,
    Query(query): Query<NotificationQuery>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Json<Vec<Notification>>> {
    let user_id = params
        .get("user_id")
        .ok_or_else(|| ApiError::bad_request("user_id parameter required"))?;
    
    let user_id = Uuid::parse_str(user_id)
        .map_err(|_| ApiError::bad_request("Invalid user_id format"))?;

    info!("Listing notifications for user: {}", user_id);

    let db_pool = container.db_pool();
    let limit = query.limit.unwrap_or(50).min(100); // Max 100 notifications per request
    let offset = query.offset.unwrap_or(0);

    let mut where_conditions = vec![
        "n.user_id = $1".to_string(),
        "(n.expires_at IS NULL OR n.expires_at > CURRENT_TIMESTAMP)".to_string(),
    ];
    let mut param_count = 1;

    if !query.include_archived.unwrap_or(false) {
        where_conditions.push("n.is_archived = false".to_string());
    }

    if query.unread_only.unwrap_or(false) {
        where_conditions.push("n.is_read = false".to_string());
    }

    if let Some(category) = &query.category {
        param_count += 1;
        where_conditions.push(format!("n.category = ${}", param_count));
    }

    if let Some(notification_type) = &query.notification_type {
        param_count += 1;
        where_conditions.push(format!("n.notification_type = ${}", param_count));
    }

    if let Some(priority) = &query.priority {
        param_count += 1;
        where_conditions.push(format!("n.priority = ${}", param_count));
    }

    if query.urgent_only.unwrap_or(false) {
        where_conditions.push("n.is_urgent = true".to_string());
    }

    let where_clause = where_conditions.join(" AND ");
    
    let query_sql = format!(
        r#"
        SELECT n.id, n.user_id, n.title, n.message, n.notification_type, n.category,
               n.priority, n.is_urgent, n.is_read, n.read_at, n.is_archived, n.archived_at,
               n.data, n.action_url, n.action_label, n.icon, n.created_at, n.updated_at, n.expires_at
        FROM notifications n
        JOIN notification_preferences np ON n.user_id = np.user_id
        WHERE {} 
          AND np.notifications_enabled = true
        ORDER BY n.is_urgent DESC, n.created_at DESC
        LIMIT {} OFFSET {}
        "#,
        where_clause, limit, offset
    );

    let mut query_builder = sqlx::query_as(&query_sql).bind(user_id);

    if let Some(category) = &query.category {
        query_builder = query_builder.bind(category);
    }
    if let Some(notification_type) = &query.notification_type {
        query_builder = query_builder.bind(notification_type);
    }
    if let Some(priority) = &query.priority {
        query_builder = query_builder.bind(priority);
    }

    let notifications: Vec<Notification> = query_builder
        .fetch_all(db_pool.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to fetch notifications: {}", e);
            ApiError::internal_server_error("Failed to fetch notifications")
        })?;

    Ok(Json(notifications))
}

/// Create a new notification
/// POST /api/notifications
pub async fn create_notification(
    State(container): State<AppContainer>,
    Json(request): Json<CreateNotificationRequest>,
) -> ApiResult<Json<Notification>> {
    info!("Creating notification for user: {}", request.user_id);

    let db_pool = container.db_pool();
    let notification_id = Uuid::new_v4();

    let notification = sqlx::query_as::<_, Notification>(
        r#"
        INSERT INTO notifications (
            id, user_id, title, message, notification_type, category, priority, is_urgent,
            data, action_url, action_label, icon, expires_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        RETURNING id, user_id, title, message, notification_type, category, priority, is_urgent,
                  is_read, read_at, is_archived, archived_at, data, action_url, action_label, icon,
                  created_at, updated_at, expires_at
        "#,
    )
    .bind(notification_id)
    .bind(request.user_id)
    .bind(&request.title)
    .bind(&request.message)
    .bind(&request.notification_type)
    .bind(&request.category)
    .bind(request.priority.as_deref().unwrap_or("normal"))
    .bind(request.is_urgent.unwrap_or(false))
    .bind(&request.data)
    .bind(&request.action_url)
    .bind(&request.action_label)
    .bind(&request.icon)
    .bind(request.expires_at)
    .fetch_one(db_pool.as_ref())
    .await
    .map_err(|e| {
        error!("Failed to create notification: {}", e);
        ApiError::internal_server_error("Failed to create notification")
    })?;

    // Log delivery for audit
    let _ = sqlx::query!(
        r#"
        INSERT INTO notification_delivery_log (notification_id, delivery_method, status, delivered_at)
        VALUES ($1, 'database', 'delivered', CURRENT_TIMESTAMP)
        "#,
        notification_id
    )
    .execute(db_pool.as_ref())
    .await;

    info!("Created notification {} for user {}", notification_id, request.user_id);
    Ok(Json(notification))
}

/// Get a specific notification
/// GET /api/notifications/:id
pub async fn get_notification(
    State(container): State<AppContainer>,
    Path(notification_id): Path<Uuid>,
) -> ApiResult<Json<Notification>> {
    let db_pool = container.db_pool();

    let notification = sqlx::query_as::<_, Notification>(
        r#"
        SELECT id, user_id, title, message, notification_type, category, priority, is_urgent,
               is_read, read_at, is_archived, archived_at, data, action_url, action_label, icon,
               created_at, updated_at, expires_at
        FROM notifications
        WHERE id = $1 AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
        "#,
    )
    .bind(notification_id)
    .fetch_optional(db_pool.as_ref())
    .await
    .map_err(|e| {
        error!("Failed to fetch notification: {}", e);
        ApiError::internal_server_error("Failed to fetch notification")
    })?
    .ok_or_else(|| ApiError::not_found("Notification not found"))?;

    Ok(Json(notification))
}

/// Update notification status
/// PATCH /api/notifications/:id
pub async fn update_notification(
    State(container): State<AppContainer>,
    Path(notification_id): Path<Uuid>,
    Json(request): Json<UpdateNotificationRequest>,
) -> ApiResult<Json<Notification>> {
    let db_pool = container.db_pool();

    let mut updates = Vec::new();
    let mut param_count = 1;

    if let Some(is_read) = request.is_read {
        param_count += 1;
        if is_read {
            updates.push(format!("is_read = ${}, read_at = CURRENT_TIMESTAMP", param_count));
        } else {
            updates.push(format!("is_read = ${}, read_at = NULL", param_count));
        }
    }

    if let Some(is_archived) = request.is_archived {
        param_count += 1;
        if is_archived {
            updates.push(format!("is_archived = ${}, archived_at = CURRENT_TIMESTAMP", param_count));
        } else {
            updates.push(format!("is_archived = ${}, archived_at = NULL", param_count));
        }
    }

    if updates.is_empty() {
        return Err(ApiError::bad_request("No updates provided"));
    }

    let update_clause = updates.join(", ");
    let query_sql = format!(
        r#"
        UPDATE notifications 
        SET {}, updated_at = CURRENT_TIMESTAMP
        WHERE id = $1
        RETURNING id, user_id, title, message, notification_type, category, priority, is_urgent,
                  is_read, read_at, is_archived, archived_at, data, action_url, action_label, icon,
                  created_at, updated_at, expires_at
        "#,
        update_clause
    );

    let mut query_builder = sqlx::query_as(&query_sql).bind(notification_id);

    if let Some(is_read) = request.is_read {
        query_builder = query_builder.bind(is_read);
    }
    if let Some(is_archived) = request.is_archived {
        query_builder = query_builder.bind(is_archived);
    }

    let notification: Notification = query_builder
        .fetch_optional(db_pool.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to update notification: {}", e);
            ApiError::internal_server_error("Failed to update notification")
        })?
        .ok_or_else(|| ApiError::not_found("Notification not found"))?;

    Ok(Json(notification))
}

/// Delete a notification
/// DELETE /api/notifications/:id
pub async fn delete_notification(
    State(container): State<AppContainer>,
    Path(notification_id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    let db_pool = container.db_pool();

    let result = sqlx::query!(
        "DELETE FROM notifications WHERE id = $1",
        notification_id
    )
    .execute(db_pool.as_ref())
    .await
    .map_err(|e| {
        error!("Failed to delete notification: {}", e);
        ApiError::internal_server_error("Failed to delete notification")
    })?;

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("Notification not found"));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Notification deleted successfully"
    })))
}

/// Bulk update notifications
/// POST /api/notifications/bulk
pub async fn bulk_update_notifications(
    State(container): State<AppContainer>,
    Json(request): Json<BulkUpdateRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    if request.notification_ids.is_empty() {
        return Err(ApiError::bad_request("No notification IDs provided"));
    }

    let db_pool = container.db_pool();
    let notification_ids: Vec<Uuid> = request.notification_ids;

    let result = match request.action.as_str() {
        "mark_read" => {
            sqlx::query!(
                "UPDATE notifications SET is_read = true, read_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP WHERE id = ANY($1)",
                &notification_ids
            )
            .execute(db_pool.as_ref())
            .await
        }
        "mark_unread" => {
            sqlx::query!(
                "UPDATE notifications SET is_read = false, read_at = NULL, updated_at = CURRENT_TIMESTAMP WHERE id = ANY($1)",
                &notification_ids
            )
            .execute(db_pool.as_ref())
            .await
        }
        "archive" => {
            sqlx::query!(
                "UPDATE notifications SET is_archived = true, archived_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP WHERE id = ANY($1)",
                &notification_ids
            )
            .execute(db_pool.as_ref())
            .await
        }
        "unarchive" => {
            sqlx::query!(
                "UPDATE notifications SET is_archived = false, archived_at = NULL, updated_at = CURRENT_TIMESTAMP WHERE id = ANY($1)",
                &notification_ids
            )
            .execute(db_pool.as_ref())
            .await
        }
        "delete" => {
            sqlx::query!(
                "DELETE FROM notifications WHERE id = ANY($1)",
                &notification_ids
            )
            .execute(db_pool.as_ref())
            .await
        }
        _ => return Err(ApiError::bad_request("Invalid action")),
    };

    let affected_rows = result
        .map_err(|e| {
            error!("Failed to bulk update notifications: {}", e);
            ApiError::internal_server_error("Failed to bulk update notifications")
        })?
        .rows_affected();

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Bulk {} completed", request.action),
        "affected_count": affected_rows
    })))
}

/// Get notification counts and statistics
/// GET /api/notifications/count?user_id=uuid
pub async fn get_notification_count(
    State(container): State<AppContainer>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Json<NotificationCountResponse>> {
    let user_id = params
        .get("user_id")
        .ok_or_else(|| ApiError::bad_request("user_id parameter required"))?;
    
    let user_id = Uuid::parse_str(user_id)
        .map_err(|_| ApiError::bad_request("Invalid user_id format"))?;

    let db_pool = container.db_pool();

    // Get total and unread counts
    let counts = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) as total,
            COUNT(*) FILTER (WHERE is_read = false) as unread,
            COUNT(*) FILTER (WHERE is_urgent = true AND is_read = false) as urgent
        FROM notifications n
        JOIN notification_preferences np ON n.user_id = np.user_id
        WHERE n.user_id = $1 
          AND n.is_archived = false
          AND (n.expires_at IS NULL OR n.expires_at > CURRENT_TIMESTAMP)
          AND np.notifications_enabled = true
        "#,
        user_id
    )
    .fetch_one(db_pool.as_ref())
    .await
    .map_err(|e| {
        error!("Failed to get notification counts: {}", e);
        ApiError::internal_server_error("Failed to get notification counts")
    })?;

    // Get counts by category
    let category_counts = sqlx::query!(
        r#"
        SELECT category, COUNT(*) as count
        FROM notifications n
        JOIN notification_preferences np ON n.user_id = np.user_id
        WHERE n.user_id = $1 
          AND n.is_archived = false
          AND n.is_read = false
          AND (n.expires_at IS NULL OR n.expires_at > CURRENT_TIMESTAMP)
          AND np.notifications_enabled = true
        GROUP BY category
        "#,
        user_id
    )
    .fetch_all(db_pool.as_ref())
    .await
    .map_err(|e| {
        error!("Failed to get category counts: {}", e);
        ApiError::internal_server_error("Failed to get category counts")
    })?;

    // Get counts by priority
    let priority_counts = sqlx::query!(
        r#"
        SELECT priority, COUNT(*) as count
        FROM notifications n
        JOIN notification_preferences np ON n.user_id = np.user_id
        WHERE n.user_id = $1 
          AND n.is_archived = false
          AND n.is_read = false
          AND (n.expires_at IS NULL OR n.expires_at > CURRENT_TIMESTAMP)
          AND np.notifications_enabled = true
        GROUP BY priority
        "#,
        user_id
    )
    .fetch_all(db_pool.as_ref())
    .await
    .map_err(|e| {
        error!("Failed to get priority counts: {}", e);
        ApiError::internal_server_error("Failed to get priority counts")
    })?;

    let mut by_category = HashMap::new();
    for row in category_counts {
        by_category.insert(
            row.category.unwrap_or_else(|| "uncategorized".to_string()),
            row.count.unwrap_or(0) as i64,
        );
    }

    let mut by_priority = HashMap::new();
    for row in priority_counts {
        by_priority.insert(
            row.priority.unwrap_or_else(|| "normal".to_string()),
            row.count.unwrap_or(0) as i64
        );
    }

    let response = NotificationCountResponse {
        total: counts.total.unwrap_or(0) as i64,
        unread: counts.unread.unwrap_or(0) as i64,
        urgent: counts.urgent.unwrap_or(0) as i64,
        by_category,
        by_priority,
    };

    Ok(Json(response))
}

/// Get notification preferences
/// GET /api/notifications/preferences?user_id=uuid
pub async fn get_preferences(
    State(container): State<AppContainer>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Json<NotificationPreferences>> {
    let user_id = params
        .get("user_id")
        .ok_or_else(|| ApiError::bad_request("user_id parameter required"))?;
    
    let user_id = Uuid::parse_str(user_id)
        .map_err(|_| ApiError::bad_request("Invalid user_id format"))?;

    let db_pool = container.db_pool();

    let preferences = sqlx::query_as::<_, NotificationPreferences>(
        "SELECT notifications_enabled, email_notifications, account_notifications, trading_notifications, system_notifications, security_notifications, promotion_notifications, minimum_priority, urgent_only, auto_archive_days, auto_delete_days FROM notification_preferences WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(db_pool.as_ref())
    .await
    .map_err(|e| {
        error!("Failed to get notification preferences: {}", e);
        ApiError::internal_server_error("Failed to get preferences")
    })?
    .unwrap_or(NotificationPreferences {
        notifications_enabled: true,
        email_notifications: true,
        account_notifications: true,
        trading_notifications: true,
        system_notifications: true,
        security_notifications: true,
        promotion_notifications: false,
        minimum_priority: "normal".to_string(),
        urgent_only: false,
        auto_archive_days: 30,
        auto_delete_days: 90,
    });

    Ok(Json(preferences))
}

/// Update notification preferences
/// POST /api/notifications/preferences?user_id=uuid
pub async fn update_preferences(
    State(container): State<AppContainer>,
    Query(params): Query<HashMap<String, String>>,
    Json(preferences): Json<NotificationPreferences>,
) -> ApiResult<Json<NotificationPreferences>> {
    let user_id = params
        .get("user_id")
        .ok_or_else(|| ApiError::bad_request("user_id parameter required"))?;
    
    let user_id = Uuid::parse_str(user_id)
        .map_err(|_| ApiError::bad_request("Invalid user_id format"))?;

    let db_pool = container.db_pool();

    let updated_preferences = sqlx::query_as::<_, NotificationPreferences>(
        r#"
        INSERT INTO notification_preferences (
            user_id, notifications_enabled, email_notifications, account_notifications,
            trading_notifications, system_notifications, security_notifications,
            promotion_notifications, minimum_priority, urgent_only, auto_archive_days, auto_delete_days
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        ON CONFLICT (user_id) DO UPDATE SET
            notifications_enabled = EXCLUDED.notifications_enabled,
            email_notifications = EXCLUDED.email_notifications,
            account_notifications = EXCLUDED.account_notifications,
            trading_notifications = EXCLUDED.trading_notifications,
            system_notifications = EXCLUDED.system_notifications,
            security_notifications = EXCLUDED.security_notifications,
            promotion_notifications = EXCLUDED.promotion_notifications,
            minimum_priority = EXCLUDED.minimum_priority,
            urgent_only = EXCLUDED.urgent_only,
            auto_archive_days = EXCLUDED.auto_archive_days,
            auto_delete_days = EXCLUDED.auto_delete_days,
            updated_at = CURRENT_TIMESTAMP
        RETURNING notifications_enabled, email_notifications, account_notifications,
                  trading_notifications, system_notifications, security_notifications,
                  promotion_notifications, minimum_priority, urgent_only, auto_archive_days, auto_delete_days
        "#,
    )
    .bind(user_id)
    .bind(preferences.notifications_enabled)
    .bind(preferences.email_notifications)
    .bind(preferences.account_notifications)
    .bind(preferences.trading_notifications)
    .bind(preferences.system_notifications)
    .bind(preferences.security_notifications)
    .bind(preferences.promotion_notifications)
    .bind(&preferences.minimum_priority)
    .bind(preferences.urgent_only)
    .bind(preferences.auto_archive_days)
    .bind(preferences.auto_delete_days)
    .fetch_one(db_pool.as_ref())
    .await
    .map_err(|e| {
        error!("Failed to update notification preferences: {}", e);
        ApiError::internal_server_error("Failed to update preferences")
    })?;

    Ok(Json(updated_preferences))
}

/// Run notification cleanup
/// POST /api/notifications/cleanup
pub async fn cleanup_notifications(
    State(container): State<AppContainer>,
) -> ApiResult<Json<serde_json::Value>> {
    let db_pool = container.db_pool();

    let result = sqlx::query_scalar!(
        "SELECT cleanup_notifications()"
    )
    .fetch_one(db_pool.as_ref())
    .await
    .map_err(|e| {
        error!("Failed to run notification cleanup: {}", e);
        ApiError::internal_server_error("Failed to run cleanup")
    })?;

    let cleaned_count = result.unwrap_or(0);

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Notification cleanup completed",
        "cleaned_count": cleaned_count
    })))
}