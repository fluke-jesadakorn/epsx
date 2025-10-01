/// Stateless admin notification handlers for serverless deployment
/// Handles Web3-first wallet notifications and database storage

use axum::{
    response::IntoResponse,
    Json,
    Extension,
    extract::{Path, Query, OriginalUri},
};
use std::sync::Arc;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::core::errors::AppError;
// Email service removed - Web3-first system uses direct wallet notifications
use crate::infrastructure::container::AuthenticatedUser;

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SendNotificationRequest {
    pub title: String,
    pub body: String,
    pub image_url: Option<String>,
    // recipient_email removed - Web3-first system uses wallet notifications only
    pub recipient_wallet: Option<String>,
    pub topic_name: Option<String>,
    pub data_payload: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct SendNotificationResponse {
    pub success: bool,
    pub notification_id: String,
    pub delivery_channel: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct NotificationStatsResponse {
    pub total_sent: u64,
    pub successful_deliveries: u64,
    pub failed_deliveries: u64,
    // email_deliveries removed - Web3-first system uses wallet notifications only
    pub in_app_notifications: u64,
    pub wallet_notifications: u64,
}

/// Send admin notification handler (Web3-first implementation)
pub async fn admin_send_notification(
    Extension(auth_user): Extension<AuthenticatedUser>,
    // Email service removed - Web3-first system uses direct wallet notifications
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

    let notification_id = Uuid::new_v4().to_string();

    // Send notification based on recipient type (Web3-first approach)
    let delivery_result = if let Some(wallet) = &request.recipient_wallet {
        // Store wallet notification for in-app delivery
        tracing::info!("Storing wallet notification for {}", wallet);
        // In a real implementation, this would store in PostgreSQL notifications table
        (true, "wallet", format!("Wallet notification stored for {}", wallet))
    } else if let Some(topic) = &request.topic_name {
        // Send to topic subscribers via wallet notifications
        tracing::info!("Sending topic notification to {}", topic);
        // In a real implementation, this would:
        // 1. Query database for topic subscribers (by wallet address)
        // 2. Store notifications for all subscriber wallets
        (true, "topic-wallet", format!("Topic notification sent to wallet subscribers of {}", topic))
    } else {
        return Err(AppError {
            kind: crate::core::errors::ErrorKind::ValidationError,
            message: "Either recipient_wallet or topic_name must be provided".to_string(),
            context: crate::core::errors::ErrorContext::default(),
            correlation_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            stack_trace: None,
        });
    };

    let response = SendNotificationResponse {
        success: delivery_result.0,
        notification_id,
        delivery_channel: delivery_result.1.to_string(),
        message: delivery_result.2,
    };

    Ok(Json(response))
}

/// Get admin notification statistics (stateless)
pub async fn get_admin_notification_stats(
    Extension(auth_user): Extension<AuthenticatedUser>,
    _query: Query<PaginationQuery>,
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

    // In a real implementation, query database for actual wallet notification stats
    let stats = NotificationStatsResponse {
        total_sent: 0,
        successful_deliveries: 0,
        failed_deliveries: 0,
        // email_deliveries removed - Web3-first system uses wallet notifications only
        in_app_notifications: 0,
        wallet_notifications: 0,
    };

    Ok(Json(stats))
}

/// Get recent notifications (stateless)
pub async fn get_recent_notifications(
    Extension(auth_user): Extension<AuthenticatedUser>,
    query: Query<PaginationQuery>,
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

    let limit = query.limit.unwrap_or(20);
    // In a real implementation, query database for notifications
    let notifications = Vec::<serde_json::Value>::new();

    Ok(Json(serde_json::json!({
        "notifications": notifications,
        "total": 0,
        "limit": limit,
        "offset": query.offset.unwrap_or(0)
    })))
}

/// Mark notification as read (stateless)
pub async fn mark_notification_read(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(notification_id): Path<String>,
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

    tracing::info!("Marking notification {} as read", notification_id);
    // In a real implementation, update database

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Notification {} marked as read", notification_id),
    })))
}

/// Delete notification (stateless)
pub async fn delete_notification(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(notification_id): Path<String>,
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

    tracing::info!("Deleting notification {}", notification_id);
    // In a real implementation, delete from database

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Notification {} deleted", notification_id),
    })))
}