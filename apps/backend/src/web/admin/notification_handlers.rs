use axum::{
    response::IntoResponse,
    Json,
    Extension,
    extract::{Path, Query},
};
use std::sync::Arc;
use uuid::Uuid;
use tracing::warn;

use crate::core::errors::AppError;
use crate::infra::services::{FcmService, FcmNotification};
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
    Extension(repo): Extension<Arc<crate::infra::db::diesel::repos::UserNotificationRepository>>,
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

    match repo.get_admin_notification_stats().await {
        Ok(stats) => {
            Ok(Json(serde_json::json!({
                "total_sent": stats.total_sent,
                "delivered": stats.delivered,
                "failed": stats.failed,
                "pending": stats.pending,
                "success_rate": stats.success_rate,
                "todays_sent": stats.todays_sent,
                "todays_delivered": stats.todays_delivered,
                "avg_delivery_time": stats.avg_delivery_time,
                "peak_hour": stats.peak_hour,
                "admin_request_by": auth_user.user_id,
                "generated_at": chrono::Utc::now()
            })))
        },
        Err(e) => {
            warn!("Admin {} failed to fetch notification stats: {}", auth_user.user_id, e);
            Err(AppError {
                kind: crate::core::errors::ErrorKind::DatabaseError,
                message: "Failed to fetch admin notification statistics".to_string(),
                context: crate::core::errors::ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })
        }
    }
}

/// Admin get user notifications handler with real database query
pub async fn admin_get_user_notifications(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Extension(repo): Extension<Arc<crate::infra::db::diesel::repos::UserNotificationRepository>>,
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
    
    match repo.get_all_notifications(Some(limit), Some(offset)).await {
        Ok(notifications) => {
            let total_count = notifications.len() as i64;
            
            Ok(Json(serde_json::json!({
                "notifications": notifications,
                "total_count": total_count,
                "limit": limit,
                "offset": offset,
                "admin_request_by": auth_user.user_id,
                "fetched_at": chrono::Utc::now()
            })))
        },
        Err(e) => {
            warn!("Admin {} failed to fetch user notifications: {}", auth_user.user_id, e);
            Err(AppError {
                kind: crate::core::errors::ErrorKind::DatabaseError,
                message: "Failed to fetch user notifications".to_string(),
                context: crate::core::errors::ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })
        }
    }
}

/// Admin mark notification as read handler with real database operation
pub async fn admin_mark_notification_read(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Extension(repo): Extension<Arc<crate::infra::db::diesel::repos::UserNotificationRepository>>,
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

    // For admin operations, we'll mark the notification as read for all users who received it
    // This is a bulk operation for admin convenience
    match repo.mark_notification_as_read_for_all(notification_id).await {
        Ok(affected_count) => {
            Ok(Json(serde_json::json!({
                "success": true,
                "message": format!("Notification marked as read for {} users", affected_count),
                "notification_id": notification_id,
                "affected_users": affected_count,
                "admin_request_by": auth_user.user_id,
                "marked_at": chrono::Utc::now()
            })))
        },
        Err(e) => {
            warn!("Admin {} failed to mark notification {} as read: {}", auth_user.user_id, notification_id, e);
            Err(AppError {
                kind: crate::core::errors::ErrorKind::DatabaseError,
                message: "Failed to mark notification as read".to_string(),
                context: crate::core::errors::ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })
        }
    }
}

/// Admin delete notification handler with real database operation
pub async fn admin_delete_notification(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Extension(repo): Extension<Arc<crate::infra::db::diesel::repos::UserNotificationRepository>>,
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

    match repo.delete_notification(notification_id).await {
        Ok(deleted) => {
            if deleted {
                Ok(Json(serde_json::json!({
                    "success": true,
                    "message": "Notification deleted successfully",
                    "notification_id": notification_id,
                    "admin_request_by": auth_user.user_id,
                    "deleted_at": chrono::Utc::now()
                })))
            } else {
                Ok(Json(serde_json::json!({
                    "success": false,
                    "message": "Notification not found or already deleted",
                    "notification_id": notification_id,
                    "admin_request_by": auth_user.user_id
                })))
            }
        },
        Err(e) => {
            warn!("Admin {} failed to delete notification {}: {}", auth_user.user_id, notification_id, e);
            Err(AppError {
                kind: crate::core::errors::ErrorKind::DatabaseError,
                message: "Failed to delete notification".to_string(),
                context: crate::core::errors::ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })
        }
    }
}