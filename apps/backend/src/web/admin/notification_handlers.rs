use axum::{
    response::IntoResponse,
    Json,
    Extension,
};
use std::sync::Arc;
use uuid::Uuid;

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

/// Admin notification stats handler (stub implementation) 
pub async fn admin_get_notification_stats() -> Result<impl IntoResponse, AppError> {
    Ok(Json(serde_json::json!({
        "message": "Stub implementation - feature not available"
    })))
}

/// Admin get user notifications handler (stub implementation)
pub async fn admin_get_user_notifications() -> Result<impl IntoResponse, AppError> {
    Ok(Json(serde_json::json!({
        "message": "Stub implementation - feature not available"
    })))
}

/// Admin mark notification as read handler (stub implementation)
pub async fn admin_mark_notification_read() -> Result<impl IntoResponse, AppError> {
    Ok(Json(serde_json::json!({
        "message": "Stub implementation - feature not available" 
    })))
}

/// Admin delete notification handler (stub implementation)
pub async fn admin_delete_notification() -> Result<impl IntoResponse, AppError> {
    Ok(Json(serde_json::json!({
        "message": "Stub implementation - feature not available"
    })))
}