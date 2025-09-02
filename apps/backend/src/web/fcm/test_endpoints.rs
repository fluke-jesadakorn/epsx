// ============================================================================
// FCM TEST ENDPOINTS (NO AUTH REQUIRED)
// ============================================================================
// Test endpoints for FCM functionality without authentication requirements

use axum::{
    extract::State,
    response::Json,
};
use serde::{Deserialize, Serialize};
use tracing::{info, error};
use uuid::Uuid;

use crate::dom::values::UserId;
use crate::infra::db::diesel::types::DevicePlatform;
use crate::infra::services::fcm_push_service::FcmMessage;
use crate::infra::container::AppContainer;
use crate::core::errors::AppResult;

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct TestRegisterTokenRequest {
    pub user_id: String,
    pub token: String,
    pub platform: DevicePlatform,
    pub device_info: Option<serde_json::Value>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TestSendNotificationRequest {
    pub user_id: String,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Serialize)]
pub struct TestResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

// ============================================================================
// TEST ENDPOINTS
// ============================================================================

/// Test FCM token registration (no auth required)
pub async fn test_register_token(
    State(container): State<AppContainer>,
    Json(request): Json<TestRegisterTokenRequest>,
) -> AppResult<Json<TestResponse>> {
    info!("Testing FCM token registration for user: {}", request.user_id);

    let fcm_token_service = container.fcm_token_service();
    let user_id = UserId(Uuid::parse_str(&request.user_id)
        .map_err(|_| crate::core::errors::AppError::validation_error("Invalid user ID"))?);

    match fcm_token_service.register_token(
        &user_id,
        request.token,
        request.platform,
        request.device_info,
        request.user_agent,
    ).await {
        Ok(token_info) => {
            info!("Successfully registered test FCM token for user: {}", request.user_id);
            Ok(Json(TestResponse {
                success: true,
                message: "FCM token registered successfully".to_string(),
                data: Some(serde_json::json!({
                    "token_id": token_info.id,
                    "platform": token_info.platform,
                    "is_active": token_info.is_active
                })),
            }))
        }
        Err(e) => {
            error!("Failed to register test FCM token for user {}: {}", request.user_id, e);
            Ok(Json(TestResponse {
                success: false,
                message: format!("Failed to register token: {}", e),
                data: None,
            }))
        }
    }
}

/// Test FCM notification sending (no auth required)  
pub async fn test_send_notification(
    State(container): State<AppContainer>,
    Json(request): Json<TestSendNotificationRequest>,
) -> AppResult<Json<TestResponse>> {
    info!("Testing FCM notification send to user: {}", request.user_id);

    let fcm_push_service = container.fcm_push_service();
    let user_id = UserId(Uuid::parse_str(&request.user_id)
        .map_err(|_| crate::core::errors::AppError::validation_error("Invalid user ID"))?);

    let message = FcmMessage::simple_notification(request.title, request.body);

    match fcm_push_service.send_to_user(&user_id, &message, None).await {
        Ok(batch_result) => {
            info!(
                "Test notification sent to user {}: {}/{} successful", 
                request.user_id, 
                batch_result.successful, 
                batch_result.total_sent
            );
            
            Ok(Json(TestResponse {
                success: batch_result.successful > 0,
                message: if batch_result.successful > 0 {
                    format!(
                        "Test notification sent to {} out of {} devices", 
                        batch_result.successful, 
                        batch_result.total_sent
                    )
                } else if batch_result.total_sent == 0 {
                    "No FCM tokens found for user. Register a token first.".to_string()
                } else {
                    "Failed to send notifications to any devices".to_string()
                },
                data: Some(serde_json::json!({
                    "sent_count": batch_result.successful,
                    "failed_count": batch_result.failed,
                    "total_sent": batch_result.total_sent
                })),
            }))
        }
        Err(e) => {
            error!("Failed to send test notification to user {}: {}", request.user_id, e);
            Ok(Json(TestResponse {
                success: false,
                message: format!("Failed to send test notification: {}", e),
                data: None,
            }))
        }
    }
}

/// Get test user FCM tokens (no auth required)
pub async fn test_get_user_tokens(
    State(container): State<AppContainer>,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> AppResult<Json<TestResponse>> {
    info!("Testing FCM token retrieval for user: {}", user_id);

    let fcm_token_service = container.fcm_token_service();
    let user_id_parsed = UserId(Uuid::parse_str(&user_id)
        .map_err(|_| crate::core::errors::AppError::validation_error("Invalid user ID"))?);

    match fcm_token_service.get_user_tokens(&user_id_parsed).await {
        Ok(tokens) => {
            let token_data: Vec<serde_json::Value> = tokens
                .into_iter()
                .map(|token| serde_json::json!({
                    "id": token.id,
                    "token": token.token,
                    "platform": token.platform,
                    "is_active": token.is_active,
                    "created_at": token.created_at,
                    "last_used_at": token.last_used_at
                }))
                .collect();
            
            info!("Retrieved {} FCM tokens for test user: {}", token_data.len(), user_id);
            Ok(Json(TestResponse {
                success: true,
                message: format!("Found {} FCM tokens", token_data.len()),
                data: Some(serde_json::json!({
                    "tokens": token_data,
                    "count": token_data.len()
                })),
            }))
        }
        Err(e) => {
            error!("Failed to get test FCM tokens for user {}: {}", user_id, e);
            Ok(Json(TestResponse {
                success: false,
                message: format!("Failed to retrieve FCM tokens: {}", e),
                data: None,
            }))
        }
    }
}

/// FCM system health check (no auth required)
pub async fn test_fcm_health() -> Json<TestResponse> {
    Json(TestResponse {
        success: true,
        message: "FCM system is operational".to_string(),
        data: Some(serde_json::json!({
            "timestamp": chrono::Utc::now(),
            "endpoints": [
                "/test/register-token",
                "/test/send-notification", 
                "/test/user/{user_id}/tokens",
                "/test/health"
            ]
        })),
    })
}