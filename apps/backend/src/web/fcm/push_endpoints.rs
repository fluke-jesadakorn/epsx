// ============================================================================
// FCM PUSH NOTIFICATION ENDPOINTS
// ============================================================================
// API endpoints for sending push notifications to users

use axum::{
    extract::State,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use tracing::{info, error};
use uuid::Uuid;

use crate::dom::values::UserId;
use crate::infra::services::fcm_push_service::FcmMessage;
use super::AuthenticatedUser;
use crate::infra::container::AppContainer;
use crate::core::errors::AppResult;

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct SendTestNotificationRequest {
    pub title: Option<String>,
    pub body: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SendTestNotificationResponse {
    pub success: bool,
    pub message: String,
    pub sent_count: u32,
    pub failed_count: u32,
}

// ============================================================================
// PUSH NOTIFICATION ENDPOINTS
// ============================================================================

/// Send a test notification to the authenticated user's devices
pub async fn send_test_notification(
    State(container): State<AppContainer>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(request): Json<SendTestNotificationRequest>,
) -> AppResult<Json<SendTestNotificationResponse>> {
    info!("Sending test notification to user: {}", user.user_id);

    let fcm_push_service = container.fcm_push_service();
    let user_id = UserId(Uuid::parse_str(&user.user_id)
        .map_err(|_| crate::core::errors::AppError::validation_error("Invalid user ID"))?);

    // Create test message
    let title = request.title.unwrap_or_else(|| "Test Notification".to_string());
    let body = request.body.unwrap_or_else(|| {
        format!("This is a test notification for user {}", user.display_name.unwrap_or("User".to_string()))
    });

    let message = FcmMessage::simple_notification(title, body);

    // Send to all user devices
    match fcm_push_service.send_to_user(&user_id, &message, None).await {
        Ok(batch_result) => {
            info!(
                "Test notification sent to user {}: {}/{} successful", 
                user.user_id, 
                batch_result.successful, 
                batch_result.total_sent
            );
            
            Ok(Json(SendTestNotificationResponse {
                success: batch_result.successful > 0,
                message: if batch_result.successful > 0 {
                    format!(
                        "Test notification sent to {} out of {} devices", 
                        batch_result.successful, 
                        batch_result.total_sent
                    )
                } else if batch_result.total_sent == 0 {
                    "No FCM tokens found for your account. Please register a device first.".to_string()
                } else {
                    "Failed to send notifications to any devices".to_string()
                },
                sent_count: batch_result.successful,
                failed_count: batch_result.failed,
            }))
        }
        Err(e) => {
            error!("Failed to send test notification to user {}: {}", user.user_id, e);
            Ok(Json(SendTestNotificationResponse {
                success: false,
                message: format!("Failed to send test notification: {}", e),
                sent_count: 0,
                failed_count: 0,
            }))
        }
    }
}