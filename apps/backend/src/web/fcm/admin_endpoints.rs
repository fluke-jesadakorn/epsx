// ============================================================================
// ADMIN FCM ENDPOINTS
// ============================================================================
// Admin-only API endpoints for FCM management and push notifications

use axum::{
    extract::{Path, Query, State},
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, error, warn};
use uuid::Uuid;

use crate::dom::values::UserId;
use crate::infra::db::diesel::types::DevicePlatform;
use crate::infra::services::fcm_token_service::FcmTokenInfo;
use crate::infra::services::fcm_push_service::{FcmMessage, FcmPriority, FcmBatchResult};
use super::AuthenticatedUser;
use crate::infra::container::AppContainer;
use crate::core::errors::AppResult;

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AdminPushRequest {
    pub title: String,
    pub body: String,
    pub data: Option<HashMap<String, String>>,
    pub image_url: Option<String>,
    pub priority: Option<FcmPriority>,
    pub ttl: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct BroadcastPushRequest {
    pub title: String,
    pub body: String,
    pub data: Option<HashMap<String, String>>,
    pub image_url: Option<String>,
    pub priority: Option<FcmPriority>,
    pub ttl: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct PlatformPushRequest {
    pub title: String,
    pub body: String,
    pub data: Option<HashMap<String, String>>,
    pub image_url: Option<String>,
    pub priority: Option<FcmPriority>,
    pub ttl: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct TokenQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct AdminFcmTokenResponse {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub platform: DevicePlatform,
    pub device_info: Option<serde_json::Value>,
    pub user_agent: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub last_used_at: String,
}

#[derive(Debug, Serialize)]
pub struct AdminPushResponse {
    pub success: bool,
    pub message: String,
    pub sent_count: u32,
    pub failed_count: u32,
    pub total_sent: u32,
}

#[derive(Debug, Serialize)]
pub struct FcmStatsResponse {
    pub total_tokens: i64,
    pub active_tokens: i64,
    pub inactive_tokens: i64,
    pub platform_breakdown: HashMap<String, i64>,
}

impl From<FcmTokenInfo> for AdminFcmTokenResponse {
    fn from(info: FcmTokenInfo) -> Self {
        Self {
            id: info.id,
            user_id: info.user_id,
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

impl From<FcmBatchResult> for AdminPushResponse {
    fn from(result: FcmBatchResult) -> Self {
        Self {
            success: result.successful > 0,
            message: if result.successful > 0 {
                format!("Sent to {}/{} devices successfully", result.successful, result.total_sent)
            } else {
                "Failed to send to any devices".to_string()
            },
            sent_count: result.successful,
            failed_count: result.failed,
            total_sent: result.total_sent,
        }
    }
}

// ============================================================================
// ADMIN TOKEN MANAGEMENT ENDPOINTS
// ============================================================================

/// Get all FCM tokens (admin only)
pub async fn admin_get_all_tokens(
    State(container): State<AppContainer>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(query): Query<TokenQuery>,
) -> AppResult<Json<Vec<AdminFcmTokenResponse>>> {
    // Check admin permissions
    if !user.has_admin_access() {
        return Err(crate::core::errors::AppError::unauthorized("Admin access required"));
    }

    info!("Admin {} getting all FCM tokens", user.user_id);

    let fcm_token_service = container.fcm_token_service();
    let limit = query.limit.unwrap_or(50);

    match fcm_token_service.get_broadcast_tokens(Some(limit)).await {
        Ok(tokens) => {
            let token_responses: Vec<AdminFcmTokenResponse> = tokens
                .into_iter()
                .map(AdminFcmTokenResponse::from)
                .collect();
            
            info!("Admin {} retrieved {} FCM tokens", user.user_id, token_responses.len());
            Ok(Json(token_responses))
        }
        Err(e) => {
            error!("Admin {} failed to get FCM tokens: {}", user.user_id, e);
            Err(crate::core::errors::AppError::internal_server_error(
                "Failed to retrieve FCM tokens"
            ))
        }
    }
}

/// Get FCM tokens for a specific user (admin only)
pub async fn admin_get_user_tokens(
    State(container): State<AppContainer>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(target_user_id): Path<String>,
) -> AppResult<Json<Vec<AdminFcmTokenResponse>>> {
    // Check admin permissions
    if !user.has_admin_access() {
        return Err(crate::core::errors::AppError::unauthorized("Admin access required"));
    }

    info!("Admin {} getting FCM tokens for user {}", user.user_id, target_user_id);

    let fcm_token_service = container.fcm_token_service();
    let user_id = UserId(Uuid::parse_str(&target_user_id)
        .map_err(|_| crate::core::errors::AppError::validation_error("Invalid user ID"))?);

    match fcm_token_service.get_user_tokens(&user_id).await {
        Ok(tokens) => {
            let token_responses: Vec<AdminFcmTokenResponse> = tokens
                .into_iter()
                .map(AdminFcmTokenResponse::from)
                .collect();
            
            info!("Admin {} retrieved {} FCM tokens for user {}", 
                user.user_id, token_responses.len(), target_user_id);
            Ok(Json(token_responses))
        }
        Err(e) => {
            error!("Admin {} failed to get FCM tokens for user {}: {}", 
                user.user_id, target_user_id, e);
            Err(crate::core::errors::AppError::internal_server_error(
                "Failed to retrieve user FCM tokens"
            ))
        }
    }
}

/// Deactivate any FCM token (admin only)
pub async fn admin_deactivate_token(
    State(container): State<AppContainer>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(token_id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    // Check admin permissions
    if !user.has_admin_access() {
        return Err(crate::core::errors::AppError::unauthorized("Admin access required"));
    }

    info!("Admin {} deactivating FCM token {}", user.user_id, token_id);

    let fcm_token_service = container.fcm_token_service();

    // For admin, we need to search through all tokens to find the one with this ID
    // This is a simplified approach - in production, you might want a more efficient method
    match fcm_token_service.deactivate_token(&token_id).await {
        Ok(success) => {
            if success {
                info!("Admin {} successfully deactivated FCM token {}", user.user_id, token_id);
                Ok(Json(serde_json::json!({
                    "success": true,
                    "message": "FCM token deactivated successfully"
                })))
            } else {
                warn!("Admin {} tried to deactivate non-existent FCM token {}", user.user_id, token_id);
                Ok(Json(serde_json::json!({
                    "success": false,
                    "message": "Token not found or already inactive"
                })))
            }
        }
        Err(e) => {
            error!("Admin {} failed to deactivate FCM token {}: {}", user.user_id, token_id, e);
            Ok(Json(serde_json::json!({
                "success": false,
                "message": format!("Failed to deactivate token: {}", e)
            })))
        }
    }
}

// ============================================================================
// ADMIN PUSH NOTIFICATION ENDPOINTS
// ============================================================================

/// Send push notification to a specific user (admin only)
pub async fn admin_send_to_user(
    State(container): State<AppContainer>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(target_user_id): Path<String>,
    Json(request): Json<AdminPushRequest>,
) -> AppResult<Json<AdminPushResponse>> {
    // Check admin permissions
    if !user.has_admin_access() {
        return Err(crate::core::errors::AppError::unauthorized("Admin access required"));
    }

    info!("Admin {} sending push notification to user {}", user.user_id, target_user_id);

    let fcm_push_service = container.fcm_push_service();
    let user_id = UserId(Uuid::parse_str(&target_user_id)
        .map_err(|_| crate::core::errors::AppError::validation_error("Invalid user ID"))?);

    // Create FCM message
    let mut message = FcmMessage::simple_notification(request.title, request.body);
    message.data = request.data;
    message.image_url = request.image_url;
    message.priority = request.priority.unwrap_or(FcmPriority::Normal);
    if let Some(ttl) = request.ttl {
        message.ttl = Some(ttl);
    }

    match fcm_push_service.send_to_user(&user_id, &message, None).await {
        Ok(result) => {
            info!("Admin {} sent notification to user {}: {}/{} successful", 
                user.user_id, target_user_id, result.successful, result.total_sent);
            Ok(Json(AdminPushResponse::from(result)))
        }
        Err(e) => {
            error!("Admin {} failed to send notification to user {}: {}", 
                user.user_id, target_user_id, e);
            Ok(Json(AdminPushResponse {
                success: false,
                message: format!("Failed to send notification: {}", e),
                sent_count: 0,
                failed_count: 0,
                total_sent: 0,
            }))
        }
    }
}

/// Send broadcast notification to all users (admin only)
pub async fn admin_send_broadcast(
    State(container): State<AppContainer>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(request): Json<BroadcastPushRequest>,
) -> AppResult<Json<AdminPushResponse>> {
    // Check admin permissions
    if !user.has_admin_access() {
        return Err(crate::core::errors::AppError::unauthorized("Admin access required"));
    }

    info!("Admin {} sending broadcast push notification", user.user_id);

    let fcm_push_service = container.fcm_push_service();

    // Create FCM message
    let mut message = FcmMessage::simple_notification(request.title, request.body);
    message.data = request.data;
    message.image_url = request.image_url;
    message.priority = request.priority.unwrap_or(FcmPriority::Normal);
    if let Some(ttl) = request.ttl {
        message.ttl = Some(ttl);
    }

    match fcm_push_service.send_broadcast(&message, request.limit, None).await {
        Ok(result) => {
            info!("Admin {} sent broadcast notification: {}/{} successful", 
                user.user_id, result.successful, result.total_sent);
            Ok(Json(AdminPushResponse::from(result)))
        }
        Err(e) => {
            error!("Admin {} failed to send broadcast notification: {}", user.user_id, e);
            Ok(Json(AdminPushResponse {
                success: false,
                message: format!("Failed to send broadcast notification: {}", e),
                sent_count: 0,
                failed_count: 0,
                total_sent: 0,
            }))
        }
    }
}

/// Send notification to all users on a specific platform (admin only)
pub async fn admin_send_to_platform(
    State(_container): State<AppContainer>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(platform): Path<DevicePlatform>,
    Json(_request): Json<PlatformPushRequest>,
) -> AppResult<Json<AdminPushResponse>> {
    // Check admin permissions
    if !user.has_admin_access() {
        return Err(crate::core::errors::AppError::unauthorized("Admin access required"));
    }

    info!("Admin {} sending push notification to {} platform", user.user_id, 
        match platform {
            DevicePlatform::Web => "web",
            DevicePlatform::Android => "Android",
            DevicePlatform::Ios => "iOS",
        });

    // For now, return a stub response since we need to implement platform-specific broadcasting
    Ok(Json(AdminPushResponse {
        success: false,
        message: "Platform-specific broadcasting not yet implemented".to_string(),
        sent_count: 0,
        failed_count: 0,
        total_sent: 0,
    }))
}

/// Get FCM statistics (admin only)
pub async fn admin_get_fcm_stats(
    State(_container): State<AppContainer>,
    Extension(user): Extension<AuthenticatedUser>,
) -> AppResult<Json<FcmStatsResponse>> {
    // Check admin permissions
    if !user.has_admin_access() {
        return Err(crate::core::errors::AppError::unauthorized("Admin access required"));
    }

    info!("Admin {} getting FCM statistics", user.user_id);

    // For now, return mock stats since we need to implement the stats function
    // TODO: Implement get_fcm_stats in the FCM token service
    Ok(Json(FcmStatsResponse {
        total_tokens: 0,
        active_tokens: 0,
        inactive_tokens: 0,
        platform_breakdown: HashMap::new(),
    }))
}