use axum::{
    extract::{Query, State},
    response::sse::{Event, KeepAlive, Sse},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use chrono::{DateTime, Utc};

use crate::{
    web::auth::AppState,
    core::errors::{AppError, ErrorKind},
};

// ============================================================================
// SSE NOTIFICATION TYPES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSENotification {
    pub id: String,
    pub wallet_address: String,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub priority: NotificationPriority,
    pub timestamp: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    System,
    Security,
    Permission,
    WalletManagement,
    Wallet,
    Payment,
    General,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Deserialize)]
pub struct SSEQuery {
    pub wallet_address: Option<String>,
    pub types: Option<String>, // comma-separated notification types
    pub timeout: Option<u64>,  // seconds
}

// ============================================================================
// SSE BROADCAST SYSTEM
// ============================================================================

#[derive(Clone)]
pub struct NotificationBroadcaster {
    sender: broadcast::Sender<SSENotification>,
}

impl NotificationBroadcaster {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        Self { sender }
    }

    pub fn broadcast(&self, notification: SSENotification) -> Result<(), AppError> {
        self.sender.send(notification).map_err(|_| {
            AppError::new(
                ErrorKind::InternalError,
                "Failed to broadcast notification".to_string(),
            )
        })?;
        Ok(())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SSENotification> {
        self.sender.subscribe()
    }
}

impl Default for NotificationBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// SSE HANDLERS
// ============================================================================

/// SSE endpoint for real-time notifications
pub async fn sse_notifications_handler(
    State(app_state): State<AppState>,
    Query(query): Query<SSEQuery>,
) -> Result<impl IntoResponse, AppError> {
    // Get wallet address from query or extract from auth context
    let wallet_address = query.wallet_address.unwrap_or_else(|| "anonymous".to_string());
    
    // Parse notification types filter
    let allowed_types: Option<Vec<NotificationType>> = query.types.map(|types_str| {
        types_str.split(',')
            .filter_map(|t| match t.trim() {
                "system" => Some(NotificationType::System),
                "security" => Some(NotificationType::Security),
                "permission" => Some(NotificationType::Permission),
                "user_management" => Some(NotificationType::WalletManagement),
                "wallet" => Some(NotificationType::Wallet),
                "payment" => Some(NotificationType::Payment),
                "general" => Some(NotificationType::General),
                _ => None,
            })
            .collect()
    });

    // Get broadcaster from app state
    let broadcaster = app_state.notification_broadcaster.clone();
    let receiver = broadcaster.subscribe();
    
    let stream = BroadcastStream::new(receiver)
        .filter_map(move |result| -> Option<Result<Event, axum::Error>> {
            let allowed_types = allowed_types.clone();
            let wallet_address = wallet_address.clone();
            
            match result {
                Ok(notification) => {
                    // Filter by wallet address (allow "all" for broadcast notifications)
                    if notification.wallet_address != wallet_address && notification.wallet_address != "all" {
                        return None;
                    }
                    
                    // Filter by notification types if specified
                    if let Some(ref types) = allowed_types {
                        if !types.contains(&notification.notification_type) {
                            return None;
                        }
                    }
                    
                    // Check if notification has expired
                    if let Some(expires_at) = notification.expires_at {
                        if Utc::now() > expires_at {
                            return None;
                        }
                    }
                    
                    // Convert to SSE event
                    match serde_json::to_string(&notification) {
                        Ok(data) => Some(Ok(Event::default()
                            .event(format!("notification_{}", notification.notification_type as u8))
                            .id(notification.id.clone())
                            .data(data))),
                        Err(_) => None,
                    }
                }
                Err(_) => None,
            }
        });

    let keep_alive_duration = Duration::from_secs(query.timeout.unwrap_or(30));
    
    Ok(Sse::new(stream)
        .keep_alive(KeepAlive::new().interval(keep_alive_duration)))
}

/// Send notification via SSE (for admin use)
pub async fn send_sse_notification_handler(
    State(app_state): State<AppState>,
    axum::extract::Json(request): axum::extract::Json<SendNotificationRequest>,
) -> Result<impl IntoResponse, AppError> {
    // TODO: Add authentication check for admin users
    
    let notification = SSENotification {
        id: uuid::Uuid::new_v4().to_string(),
        wallet_address: request.wallet_address.unwrap_or_else(|| "all".to_string()),
        notification_type: request.notification_type,
        title: request.title,
        message: request.message,
        data: request.data,
        priority: request.priority,
        timestamp: Utc::now(),
        expires_at: request.expires_in_seconds.map(|seconds| {
            Utc::now() + chrono::Duration::seconds(seconds as i64)
        }),
    };

    // Broadcast the notification
    let broadcaster = app_state.notification_broadcaster.clone();
    broadcaster.broadcast(notification.clone())?;

    // Store in database for persistence (stateless approach)
    // TODO: Implement database storage for notification history
    
    Ok(axum::response::Json(serde_json::json!({
        "success": true,
        "notification_id": notification.id,
        "message": "Notification sent successfully"
    })))
}

/// Broadcast notification to all users (admin only)
pub async fn broadcast_sse_notification_handler(
    State(app_state): State<AppState>,
    axum::extract::Json(request): axum::extract::Json<BroadcastNotificationRequest>,
) -> Result<impl IntoResponse, AppError> {
    // TODO: Add authentication check for admin users
    
    let notification = SSENotification {
        id: uuid::Uuid::new_v4().to_string(),
        wallet_address: "all".to_string(), // Broadcast to all wallets
        notification_type: request.notification_type,
        title: request.title,
        message: request.message,
        data: request.data,
        priority: request.priority,
        timestamp: Utc::now(),
        expires_at: request.expires_in_seconds.map(|seconds| {
            Utc::now() + chrono::Duration::seconds(seconds as i64)
        }),
    };

    // Broadcast the notification
    let broadcaster = app_state.notification_broadcaster.clone();
    broadcaster.broadcast(notification.clone())?;

    Ok(axum::response::Json(serde_json::json!({
        "success": true,
        "notification_id": notification.id,
        "message": "Broadcast notification sent successfully"
    })))
}

// ============================================================================
// REQUEST TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct SendNotificationRequest {
    pub wallet_address: Option<String>,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub priority: NotificationPriority,
    pub expires_in_seconds: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct BroadcastNotificationRequest {
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub priority: NotificationPriority,
    pub expires_in_seconds: Option<u32>,
}

// ============================================================================
// HEALTH CHECK
// ============================================================================

/// SSE health check endpoint
pub async fn sse_health_handler(
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let broadcaster = app_state.notification_broadcaster.clone();
    
    // Send a test heartbeat notification
    let heartbeat = SSENotification {
        id: uuid::Uuid::new_v4().to_string(),
        wallet_address: "system".to_string(),
        notification_type: NotificationType::System,
        title: "SSE Health Check".to_string(),
        message: "SSE system is operational".to_string(),
        data: Some(serde_json::json!({
            "timestamp": Utc::now(),
            "status": "healthy"
        })),
        priority: NotificationPriority::Low,
        timestamp: Utc::now(),
        expires_at: Some(Utc::now() + chrono::Duration::seconds(5)),
    };

    // Attempt to broadcast
    match broadcaster.broadcast(heartbeat) {
        Ok(_) => Ok(axum::response::Json(serde_json::json!({
            "status": "healthy",
            "sse_active": true,
            "timestamp": Utc::now()
        }))),
        Err(_) => Ok(axum::response::Json(serde_json::json!({
            "status": "degraded",
            "sse_active": false,
            "timestamp": Utc::now()
        })))
    }
}