// ============================================================================
// FCM TOKEN MANAGEMENT ENDPOINTS
// ============================================================================
// API endpoints for FCM token registration and management

use axum::{
    extract::{Path, State},
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use tracing::{info, error, warn};
use uuid::Uuid;

use crate::dom::values::UserId;
use crate::infra::db::diesel::types::DevicePlatform;
use crate::infra::services::fcm_token_service::FcmTokenInfo;
use super::AuthenticatedUser;
use crate::infra::container::AppContainer;
use crate::core::errors::AppResult;

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct RegisterFcmTokenRequest {
    pub token: String,
    pub platform: DevicePlatform,
    pub device_info: Option<serde_json::Value>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterFcmTokenResponse {
    pub success: bool,
    pub token_id: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct FcmTokenResponse {
    pub id: String,
    pub token: String,
    pub platform: DevicePlatform,
    pub device_info: Option<serde_json::Value>,
    pub user_agent: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub last_used_at: String,
}

#[derive(Debug, Serialize)]
pub struct DeactivateTokenResponse {
    pub success: bool,
    pub message: String,
}

impl From<FcmTokenInfo> for FcmTokenResponse {
    fn from(info: FcmTokenInfo) -> Self {
        Self {
            id: info.id,
            token: info.token,
            platform: info.platform,
            device_info: info.device_info,
            user_agent: info.user_agent,
            is_active: info.is_active,
            created_at: info.created_at.to_rfc3339(),
            last_used_at: info.last_used_at.to_rfc3339(),
        }
    }
}

// ============================================================================
// TOKEN MANAGEMENT ENDPOINTS
// ============================================================================

/// Register a new FCM token for the authenticated user
pub async fn register_fcm_token(
    State(container): State<AppContainer>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(request): Json<RegisterFcmTokenRequest>,
) -> AppResult<Json<RegisterFcmTokenResponse>> {
    info!("Registering FCM token for user: {}", user.user_id);

    // Validate request
    if request.token.is_empty() {
        return Ok(Json(RegisterFcmTokenResponse {
            success: false,
            token_id: String::new(),
            message: "Token cannot be empty".to_string(),
        }));
    }

    // Get FCM token service
    let fcm_token_service = container.fcm_token_service();
    let user_id = UserId(Uuid::parse_str(&user.user_id)
        .map_err(|_| crate::core::errors::AppError::validation_error("Invalid user ID"))?);

    match fcm_token_service.register_token(
        &user_id,
        request.token,
        request.platform,
        request.device_info,
        request.user_agent,
    ).await {
        Ok(token_info) => {
            info!("Successfully registered FCM token for user: {}", user.user_id);
            Ok(Json(RegisterFcmTokenResponse {
                success: true,
                token_id: token_info.id,
                message: "FCM token registered successfully".to_string(),
            }))
        }
        Err(e) => {
            error!("Failed to register FCM token for user {}: {}", user.user_id, e);
            Ok(Json(RegisterFcmTokenResponse {
                success: false,
                token_id: String::new(),
                message: format!("Failed to register token: {}", e),
            }))
        }
    }
}

/// Get all FCM tokens for the authenticated user
pub async fn get_my_fcm_tokens(
    State(container): State<AppContainer>,
    Extension(user): Extension<AuthenticatedUser>,
) -> AppResult<Json<Vec<FcmTokenResponse>>> {
    info!("Getting FCM tokens for user: {}", user.user_id);

    let fcm_token_service = container.fcm_token_service();
    let user_id = UserId(Uuid::parse_str(&user.user_id)
        .map_err(|_| crate::core::errors::AppError::validation_error("Invalid user ID"))?);

    match fcm_token_service.get_user_tokens(&user_id).await {
        Ok(tokens) => {
            let token_responses: Vec<FcmTokenResponse> = tokens
                .into_iter()
                .map(FcmTokenResponse::from)
                .collect();
            
            info!("Retrieved {} FCM tokens for user: {}", token_responses.len(), user.user_id);
            Ok(Json(token_responses))
        }
        Err(e) => {
            error!("Failed to get FCM tokens for user {}: {}", user.user_id, e);
            Err(crate::core::errors::AppError::internal_server_error(
                "Failed to retrieve FCM tokens"
            ))
        }
    }
}

/// Deactivate an FCM token (users can only deactivate their own tokens)
pub async fn deactivate_fcm_token(
    State(container): State<AppContainer>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(token_id): Path<String>,
) -> AppResult<Json<DeactivateTokenResponse>> {
    info!("Deactivating FCM token {} for user: {}", token_id, user.user_id);

    let fcm_token_service = container.fcm_token_service();
    let user_id = UserId(Uuid::parse_str(&user.user_id)
        .map_err(|_| crate::core::errors::AppError::validation_error("Invalid user ID"))?);

    // First, verify the token belongs to this user
    match fcm_token_service.get_user_tokens(&user_id).await {
        Ok(user_tokens) => {
            let owns_token = user_tokens.iter().any(|token| token.id == token_id);
            
            if !owns_token {
                warn!("User {} tried to deactivate token {} they don't own", user.user_id, token_id);
                return Ok(Json(DeactivateTokenResponse {
                    success: false,
                    message: "Token not found or not owned by user".to_string(),
                }));
            }

            // Find the actual token string to deactivate
            if let Some(token_info) = user_tokens.iter().find(|token| token.id == token_id) {
                match fcm_token_service.deactivate_token(&token_info.token).await {
                    Ok(success) => {
                        if success {
                            info!("Successfully deactivated FCM token {} for user: {}", token_id, user.user_id);
                            Ok(Json(DeactivateTokenResponse {
                                success: true,
                                message: "FCM token deactivated successfully".to_string(),
                            }))
                        } else {
                            warn!("FCM token {} was already inactive for user: {}", token_id, user.user_id);
                            Ok(Json(DeactivateTokenResponse {
                                success: false,
                                message: "Token was already inactive".to_string(),
                            }))
                        }
                    }
                    Err(e) => {
                        error!("Failed to deactivate FCM token {} for user {}: {}", token_id, user.user_id, e);
                        Ok(Json(DeactivateTokenResponse {
                            success: false,
                            message: format!("Failed to deactivate token: {}", e),
                        }))
                    }
                }
            } else {
                Ok(Json(DeactivateTokenResponse {
                    success: false,
                    message: "Token not found".to_string(),
                }))
            }
        }
        Err(e) => {
            error!("Failed to verify token ownership for user {}: {}", user.user_id, e);
            Ok(Json(DeactivateTokenResponse {
                success: false,
                message: "Failed to verify token ownership".to_string(),
            }))
        }
    }
}