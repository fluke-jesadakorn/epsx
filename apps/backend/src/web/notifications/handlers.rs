// Notification API handlers for real-time notifications
use uuid::Uuid;

use axum::{

    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;

use serde_json::{json, Value};


use crate::web::{auth::routes::AppState, middleware::AuthCtx};


use super::dto::*;


/// GET /api/v1/notifications - List user notifications with pagination and filtering
pub async fn list_notifications_handler(
    auth_ctx: AuthCtx,
    State(_app_state): State<AppState>,
    Query(query): Query<NotificationQuery>,
) -> Result<Json<NotificationListResponse>, StatusCode> {
    let user_id = auth_ctx.user_id;
    
    // For now, return mock notifications - TODO: integrate with notification service
    let mock_notifications = vec![
        NotificationResponse {
            id: "notif_1".to_string(),
            user_id: user_id.to_string(),
            title: "Test Notification".to_string(),
            message: "This is a test notification".to_string(),
            notification_type: "system".to_string(),
            category: "general".to_string(),
            priority: "normal".to_string(),
            status: "unread".to_string(),
            channel: "in_app".to_string(),
            metadata: None,
            created_at: Utc::now(),
            updated_at: None,
            read_at: None,
            expires_at: None,
            scheduled_for: None,
            sent_at: None,
            delivery_status: None,
            error_message: None,
            retry_count: 0,
        }
    ];

    let per_page = query.per_page.unwrap_or(20);
    let page = query.page.unwrap_or(1);
    let total_count = 1u64;
    let total_pages = 1u64;

    let pagination = PaginationResponse {
        page,
        per_page,
        total_pages,
        has_next: false,
        has_prev: false,
    };

    let response = NotificationListResponse {
        notifications: mock_notifications,
        pagination,
        unread_count: 1,
        total_count,
        fetched_at: Utc::now(),
    };

    Ok(Json(response))
}

/// GET /api/v1/notifications/:id - Get single notification by ID
pub async fn get_notification_handler(
    auth_ctx: AuthCtx,
    State(_app_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<NotificationResponse>, StatusCode> {
    let user_id = auth_ctx.user_id;

    let response = NotificationResponse {
        id,
        user_id: user_id.to_string(),
        title: "Test Notification".to_string(),
        message: "This is a test notification".to_string(),
        notification_type: "system".to_string(),
        category: "general".to_string(),
        priority: "normal".to_string(),
        status: "unread".to_string(),
        channel: "in_app".to_string(),
        metadata: None,
        created_at: Utc::now(),
        updated_at: None,
        read_at: None,
        expires_at: None,
        scheduled_for: None,
        sent_at: None,
        delivery_status: None,
        error_message: None,
        retry_count: 0,
    };

    Ok(Json(response))
}

/// POST /api/v1/notifications/read/:id - Mark notification as read
pub async fn mark_notification_read_handler(
    auth_ctx: AuthCtx,
    State(_app_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let user_id = auth_ctx.user_id;

    Ok(Json(json!({
        "notification_id": id,
        "user_id": user_id.to_string(),
        "marked_at": Utc::now(),
        "status": "read"
    })))
}

/// POST /api/v1/notifications/read-all - Mark all notifications as read
pub async fn mark_all_notifications_read_handler(
    auth_ctx: AuthCtx,
    State(_app_state): State<AppState>,
    Json(req): Json<MarkNotificationsReadRequest>,
) -> Result<Json<MarkNotificationsReadResponse>, StatusCode> {
    let user_id = auth_ctx.user_id;

    let marked_count = if req.mark_all.unwrap_or(false) {
        5 // Mock: mark all notifications as read
    } else {
        req.notification_ids.len()
    };

    let response = MarkNotificationsReadResponse {
        user_id: user_id.to_string(),
        marked_count: marked_count as u64,
        notification_ids: req.notification_ids,
        mark_all: req.mark_all.unwrap_or(false),
        marked_at: Utc::now(),
    };

    Ok(Json(response))
}

/// DELETE /api/v1/notifications/:id - Delete notification
pub async fn delete_notification_handler(
    auth_ctx: AuthCtx,
    State(_app_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let user_id = auth_ctx.user_id;

    Ok(Json(json!({
        "notification_id": id,
        "user_id": user_id.to_string(),
        "deleted_at": Utc::now(),
        "status": "deleted"
    })))
}

/// GET /api/v1/notifications/unread-count - Get unread notification count
pub async fn get_unread_count_handler(
    auth_ctx: AuthCtx,
    State(_app_state): State<AppState>,
) -> Result<Json<UnreadCountResponse>, StatusCode> {
    let user_id = auth_ctx.user_id;

    let response = UnreadCountResponse {
        user_id: user_id.to_string(),
        unread_count: 1,
        by_category: vec![
            CategoryCount {
                category: "system".to_string(),
                count: 1,
            },
        ],
        by_priority: vec![
            PriorityCount {
                priority: "normal".to_string(),
                count: 1,
            },
        ],
        last_checked: Utc::now(),
    };

    Ok(Json(response))
}

/// POST /api/v1/notifications/device-token - Register FCM device token
pub async fn register_device_token_handler(
    auth_ctx: AuthCtx,
    State(_app_state): State<AppState>,
    Json(req): Json<DeviceTokenRequest>,
) -> Result<Json<DeviceTokenResponse>, StatusCode> {
    let user_id = auth_ctx.user_id;

    let response = DeviceTokenResponse {
        user_id: user_id.to_string(),
        token_id: Uuid::new_v4().to_string(),
        device_type: req.device_type,
        registered_at: Utc::now(),
        status: "registered".to_string(),
    };

    Ok(Json(response))
}

/// POST /api/v1/notifications/preferences - Update notification preferences
pub async fn update_preferences_handler(
    auth_ctx: AuthCtx,
    State(_app_state): State<AppState>,
    Json(_req): Json<NotificationPreferencesRequest>,
) -> Result<Json<NotificationPreferencesResponse>, StatusCode> {
    let user_id = auth_ctx.user_id;

    let response = NotificationPreferencesResponse {
        user_id: user_id.to_string(),
        email_enabled: true,
        push_enabled: true,
        in_app_enabled: true,
        categories: vec![],
        quiet_hours: None,
        updated_at: Utc::now(),
    };

    Ok(Json(response))
}

/// GET /api/v1/notifications/preferences - Get notification preferences
pub async fn get_preferences_handler(
    auth_ctx: AuthCtx,
    State(_app_state): State<AppState>,
) -> Result<Json<NotificationPreferencesResponse>, StatusCode> {
    let user_id = auth_ctx.user_id;

    let response = NotificationPreferencesResponse {
        user_id: user_id.to_string(),
        email_enabled: true,
        push_enabled: true,
        in_app_enabled: true,
        categories: vec![],
        quiet_hours: None,
        updated_at: Utc::now(),
    };

    Ok(Json(response))
}

/// POST /api/v1/admin/notifications - Create admin notification (broadcast or targeted)
pub async fn create_admin_notification_handler(
    auth_ctx: AuthCtx,
    State(_app_state): State<AppState>,
    Json(req): Json<CreateNotificationRequest>,
) -> Result<Json<CreateNotificationResponse>, StatusCode> {
    let _admin_user_id = auth_ctx.user_id;

    let target_count = match &req.target_users {
        Some(users) => users.len() as u64,
        None => 1, // Broadcast
    };

    let response = CreateNotificationResponse {
        notification_id: Uuid::new_v4().to_string(),
        target_count,
        created_at: Utc::now(),
        scheduled_for: req.scheduled_for,
        status: "created".to_string(),
    };

    Ok(Json(response))
}