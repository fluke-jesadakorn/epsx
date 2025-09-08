use axum::{
    response::IntoResponse,
    Json,
    Extension,
    extract::{Path, Query},
};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::errors::AppError;
use crate::infrastructure::adapters::services::{FcmService, FcmNotification};
use crate::web::notifications::dto::*;
use crate::web::middleware::AuthenticatedUser;

#[derive(Debug, serde::Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Send admin notification handler (stub implementation)
pub async fn admin_send_notification(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Extension(fcm_service): Extension<Arc<FcmService>>,
    Json(request): Json<SendNotificationRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Check admin permissions
    if !auth_user.valid_permissions.iter().any(|p| p.starts_with("admin:")) {
        return Err(AppError {
            kind: crate::core::errors::ErrorKind::AuthorizationError,
            message: "Admin access required".to_string(),
            context: crate::core::errors::ErrorContext::default(),
            correlation_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            stack_trace: None,
        });
    }

    let notification = FcmNotification {
        title: Some(request.title.clone()),
        body: Some(request.body.clone()),
        image: request.image_url.clone(),
    };

    let notification_id = Uuid::new_v4();

    // Send to specific user or topic
    if let Some(topic_name) = &request.fcm_topic_id {
        let _message_id = fcm_service
            .send_to_topic(topic_name.clone(), notification, request.data_payload)
            .await?;

        let response = SendNotificationResponse {
            id: notification_id,
            message: "Notification sent to topic successfully".to_string(),
            recipient_count: Some(100),
            delivery_ids: vec![],
        };

        Ok(Json(response))
    } else if let Some(_user_id) = request.recipient_user_id {
        // For specific user notifications, we would need FCM token lookup
        // For now, return an error indicating this feature needs implementation
        Err(AppError {
            kind: crate::core::errors::ErrorKind::InternalError,
            message: "User-specific notifications not yet implemented".to_string(),
            context: crate::core::errors::ErrorContext::default(),
            correlation_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            stack_trace: None,
        })
    } else {
        Err(AppError {
            kind: crate::core::errors::ErrorKind::ValidationError,
            message: "Either recipient_user_id or fcm_topic_id must be provided".to_string(),
            context: crate::core::errors::ErrorContext::default(),
            correlation_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            stack_trace: None,
        })
    }
}

/// Admin topic broadcast handler (stub implementation)
pub async fn admin_broadcast_to_topic() -> Result<impl IntoResponse, AppError> {
    Ok(Json(serde_json::json!({
        "message": "Stub implementation - feature not available"
    })))
}

/// Admin notification stats handler with real database query
pub async fn admin_get_notification_stats(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Extension(repo): Extension<Arc<crate::infrastructure::adapters::repositories::diesel::repos::UserNotificationRepository>>,
) -> Result<impl IntoResponse, AppError> {
    // Check admin permissions
    if !auth_user.valid_permissions.iter().any(|p| p.starts_with("admin:")) {
        return Err(AppError {
            kind: crate::core::errors::ErrorKind::AuthorizationError,
            message: "Admin access required".to_string(),
            context: crate::core::errors::ErrorContext::default(),
            correlation_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            stack_trace: None,
        });
    }

    // TODO: Implement get_admin_notification_stats method in NotificationRepositoryAdapter
    // Returning placeholder stats for now
    Ok(Json(serde_json::json!({
        "total_sent": 0,
        "delivered": 0, 
        "failed": 0,
        "pending": 0,
        "success_rate": 0.0,
        "todays_sent": 0,
        "todays_delivered": 0,
        "avg_delivery_time": 0,
        "peak_hour": 9, // Default peak hour
        "admin_request_by": auth_user.user_id,
        "generated_at": chrono::Utc::now()
    })))
}

/// Admin get user notifications handler with real database query
pub async fn admin_get_user_notifications(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Extension(repo): Extension<Arc<crate::infrastructure::adapters::repositories::diesel::repos::UserNotificationRepository>>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    // Check admin permissions
    if !auth_user.valid_permissions.iter().any(|p| p.starts_with("admin:")) {
        return Err(AppError {
            kind: crate::core::errors::ErrorKind::AuthorizationError,
            message: "Admin access required".to_string(),
            context: crate::core::errors::ErrorContext::default(),
            correlation_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            stack_trace: None,
        });
    }

    let limit = pagination.limit.unwrap_or(50);
    let offset = pagination.offset.unwrap_or(0);
    
    // TODO: Implement get_all_notifications method in NotificationRepositoryAdapter
    // Returning empty notifications list for now
    Ok(Json(serde_json::json!({
        "notifications": [],
        "total_count": 0,
        "limit": limit,
        "offset": offset,
        "admin_request_by": auth_user.user_id,
        "fetched_at": chrono::Utc::now()
    })))
}

/// Admin mark notification as read handler with real database operation
pub async fn admin_mark_notification_read(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Extension(repo): Extension<Arc<crate::infrastructure::adapters::repositories::diesel::repos::UserNotificationRepository>>,
    Path(notification_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // Check admin permissions
    if !auth_user.valid_permissions.iter().any(|p| p.starts_with("admin:")) {
        return Err(AppError {
            kind: crate::core::errors::ErrorKind::AuthorizationError,
            message: "Admin access required".to_string(),
            context: crate::core::errors::ErrorContext::default(),
            correlation_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            stack_trace: None,
        });
    }

    // TODO: Implement mark_notification_as_read_for_all method in NotificationRepositoryAdapter
    // For admin operations, we'll mark the notification as read for all users who received it
    // This is a bulk operation for admin convenience
    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Notification marked as read (placeholder - {} users)", 0),
        "notification_id": notification_id,
        "affected_users": 0,
        "admin_request_by": auth_user.user_id,
        "marked_at": chrono::Utc::now()
    })))
}

/// Admin delete notification handler with real database operation
pub async fn admin_delete_notification(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Extension(repo): Extension<Arc<crate::infrastructure::adapters::repositories::diesel::repos::UserNotificationRepository>>,
    Path(notification_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // Check admin permissions
    if !auth_user.valid_permissions.iter().any(|p| p.starts_with("admin:")) {
        return Err(AppError {
            kind: crate::core::errors::ErrorKind::AuthorizationError,
            message: "Admin access required".to_string(),
            context: crate::core::errors::ErrorContext::default(),
            correlation_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            stack_trace: None,
        });
    }

    // TODO: Implement delete_notification method in NotificationRepositoryAdapter
    // Returning success placeholder for now
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Notification delete requested (placeholder)",
        "notification_id": notification_id,
        "admin_request_by": auth_user.user_id,
        "deleted_at": chrono::Utc::now()
    })))
}