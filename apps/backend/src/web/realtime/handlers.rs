// Real-time handlers for event broadcasting and management

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use crate::web::middleware::AuthCtx;
use tracing::{info, error};

use crate::dom::values::UserId;
use super::events::{EventMessage, RealtimeEvent, NotificationLevel};
use super::websocket::{ConnectionManager, ConnectionStats};
use super::super::auth::routes::AppState;

/// Request to broadcast a system notification
#[derive(Debug, Deserialize)]
pub struct BroadcastNotificationRequest {
    pub title: String,
    pub message: String,
    pub level: String, // "info", "warning", "error", "success"
    pub target_user: Option<String>, // None for broadcast to all
}

/// Request to simulate a payment event (for testing)
#[derive(Debug, Deserialize)]
pub struct SimulatePaymentRequest {
    pub payment_id: String,
    pub user_id: String,
    pub amount: f64,
    pub currency: String,
    pub event_type: String, // "started", "completed", "failed"
    pub transaction_id: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

/// Request to simulate a stock price update
#[derive(Debug, Deserialize)]
pub struct SimulateStockUpdateRequest {
    pub symbol: String,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub volume: u64,
}

/// Response for event broadcasting
#[derive(Debug, Serialize)]
pub struct BroadcastResponse {
    pub success: bool,
    pub message: String,
    pub event_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Broadcast a system notification to all or specific users
pub async fn broadcast_notification_handler(
    auth_ctx: AuthCtx,
    State(app_state): State<AppState>,
    Json(payload): Json<BroadcastNotificationRequest>,
) -> Result<Json<BroadcastResponse>, StatusCode> {
    // Verify admin access
    let current_user_id = auth_ctx.user_id;
    verify_admin_access(&app_state, &current_user_id).await?;
    
    // Parse notification level
    let level = match payload.level.to_lowercase().as_str() {
        "info" => NotificationLevel::Info,
        "warning" => NotificationLevel::Warning,
        "error" => NotificationLevel::Error,
        "success" => NotificationLevel::Success,
        _ => NotificationLevel::Info,
    };
    
    // Create notification event
    let event = RealtimeEvent::SystemNotification {
        title: payload.title,
        message: payload.message,
        level,
        target_user: payload.target_user.clone(),
        metadata: std::collections::HashMap::new(),
        timestamp: chrono::Utc::now(),
    };
    
    let mut event_msg = EventMessage::new(event, "admin-notification".to_string());
    
    // Set target user if specified
    if let Some(target_user) = payload.target_user {
        event_msg = event_msg.with_user_id(target_user);
    }
    
    // Broadcast event
    let connection_manager = get_connection_manager(&app_state);
    connection_manager.broadcast_event(event_msg.clone()).await;
    
    info!("Admin {} broadcasted notification: {}", current_user_id.to_string(), event_msg.metadata.event_id);
    
    Ok(Json(BroadcastResponse {
        success: true,
        message: "Notification broadcasted successfully".to_string(),
        event_id: event_msg.metadata.event_id,
        timestamp: chrono::Utc::now(),
    }))
}

/// Simulate a payment event (for testing)
pub async fn simulate_payment_handler(
    auth_ctx: AuthCtx,
    State(app_state): State<AppState>,
    Json(payload): Json<SimulatePaymentRequest>,
) -> Result<Json<BroadcastResponse>, StatusCode> {
    // Verify admin access
    let current_user_id = auth_ctx.user_id;
    verify_admin_access(&app_state, &current_user_id).await?;
    
    // Create payment event based on type
    let event = match payload.event_type.as_str() {
        "started" => RealtimeEvent::payment_started(
            payload.payment_id,
            payload.user_id,
            payload.amount,
            payload.currency,
        ),
        "completed" => RealtimeEvent::payment_completed(
            payload.payment_id,
            payload.user_id, 
            payload.amount,
            payload.currency,
            payload.transaction_id.unwrap_or_else(|| format!("txn_{}", uuid::Uuid::new_v4())),
        ),
        "failed" => RealtimeEvent::payment_failed(
            payload.payment_id,
            payload.user_id,
            payload.amount,
            payload.currency,
            payload.error_code.unwrap_or_else(|| "UNKNOWN_ERROR".to_string()),
            payload.error_message.unwrap_or_else(|| "Payment failed".to_string()),
        ),
        _ => return Err(StatusCode::BAD_REQUEST),
    };
    
    let event_msg = EventMessage::new(event, "payment-simulator".to_string());
    
    // Broadcast event
    let connection_manager = get_connection_manager(&app_state);
    connection_manager.broadcast_event(event_msg.clone()).await;
    
    info!("Admin {} simulated payment event: {}", current_user_id.to_string(), event_msg.metadata.event_id);
    
    Ok(Json(BroadcastResponse {
        success: true,
        message: format!("Payment {} event simulated successfully", payload.event_type),
        event_id: event_msg.metadata.event_id,
        timestamp: chrono::Utc::now(),
    }))
}

/// Simulate a stock price update (for testing)
pub async fn simulate_stock_update_handler(
    auth_ctx: AuthCtx,
    State(app_state): State<AppState>,
    Json(payload): Json<SimulateStockUpdateRequest>,
) -> Result<Json<BroadcastResponse>, StatusCode> {
    // Verify admin access
    let current_user_id = auth_ctx.user_id;
    verify_admin_access(&app_state, &current_user_id).await?;
    
    // Create stock price update event
    let event = RealtimeEvent::StockPriceUpdate {
        symbol: payload.symbol,
        price: payload.price,
        change: payload.change,
        change_percent: payload.change_percent,
        volume: payload.volume,
        timestamp: chrono::Utc::now(),
    };
    
    let event_msg = EventMessage::new(event, "stock-simulator".to_string());
    
    // Broadcast event
    let connection_manager = get_connection_manager(&app_state);
    connection_manager.broadcast_event(event_msg.clone()).await;
    
    info!("Admin {} simulated stock update: {}", current_user_id.to_string(), event_msg.metadata.event_id);
    
    Ok(Json(BroadcastResponse {
        success: true,
        message: "Stock price update simulated successfully".to_string(),
        event_id: event_msg.metadata.event_id,
        timestamp: chrono::Utc::now(),
    }))
}

/// Get real-time connection statistics
pub async fn get_connection_stats_handler(
    auth_ctx: AuthCtx,
    State(app_state): State<AppState>,
) -> Result<Json<ConnectionStats>, StatusCode> {
    // Verify admin access
    let current_user_id = auth_ctx.user_id;
    verify_admin_access(&app_state, &current_user_id).await?;
    
    let connection_manager = get_connection_manager(&app_state);
    let stats = connection_manager.get_stats().await;
    
    Ok(Json(stats))
}

/// Send a targeted notification to a specific user
pub async fn send_user_notification_handler(
    Path(user_id): Path<String>,
    auth_ctx: AuthCtx,
    State(app_state): State<AppState>,
    Json(payload): Json<BroadcastNotificationRequest>,
) -> Result<Json<BroadcastResponse>, StatusCode> {
    // Verify admin access
    let current_user_id = auth_ctx.user_id;
    verify_admin_access(&app_state, &current_user_id).await?;
    
    // Parse notification level
    let level = match payload.level.to_lowercase().as_str() {
        "info" => NotificationLevel::Info,
        "warning" => NotificationLevel::Warning,  
        "error" => NotificationLevel::Error,
        "success" => NotificationLevel::Success,
        _ => NotificationLevel::Info,
    };
    
    // Create targeted notification event
    let event = RealtimeEvent::SystemNotification {
        title: payload.title,
        message: payload.message,
        level,
        target_user: Some(user_id.clone()),
        metadata: std::collections::HashMap::new(),
        timestamp: chrono::Utc::now(),
    };
    
    let event_msg = EventMessage::new(event, "admin-targeted-notification".to_string())
        .with_user_id(user_id.clone());
    
    // Send to specific user
    let connection_manager = get_connection_manager(&app_state);
    let target_user_id = UserId::new(user_id);
    connection_manager.send_to_user(&target_user_id, event_msg.clone()).await;
    
    info!("Admin {} sent targeted notification to {}: {}", 
          current_user_id.to_string(), target_user_id.to_string(), event_msg.metadata.event_id);
    
    Ok(Json(BroadcastResponse {
        success: true,
        message: "Targeted notification sent successfully".to_string(),
        event_id: event_msg.metadata.event_id,
        timestamp: chrono::Utc::now(),
    }))
}


/// Verify admin access
async fn verify_admin_access(app_state: &AppState, user_id: &UserId) -> Result<(), StatusCode> {
    let user = app_state.user_repo.get(user_id).await
        .map_err(|e| {
            error!("Failed to get user for admin check: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check if user has admin role
    use crate::auth::roles::Role;
    match user.role() {
        Role::Admin => Ok(()),
        _ => {
            tracing::warn!("Non-admin user {} attempted admin operation", user_id.to_string());
            Err(StatusCode::FORBIDDEN)
        }
    }
}

/// Get connection manager from app state
fn get_connection_manager(_app_state: &AppState) -> ConnectionManager {
    // In a real implementation, this would be stored in AppState
    // For now, create a new one each time (not ideal for production)
    ConnectionManager::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn should_create_broadcast_response() {
        let response = BroadcastResponse {
            success: true,
            message: "Test message".to_string(),
            event_id: "test_event_123".to_string(),
            timestamp: chrono::Utc::now(),
        };
        
        assert!(response.success);
        assert_eq!(response.message, "Test message");
        assert_eq!(response.event_id, "test_event_123");
    }
    
    #[test]
    fn should_deserialize_notification_request() {
        let json = r#"{
            "title": "Test Notification",
            "message": "This is a test message",
            "level": "info",
            "target_user": "user123"
        }"#;
        
        let request: BroadcastNotificationRequest = serde_json::from_str(json).unwrap();
        
        assert_eq!(request.title, "Test Notification");
        assert_eq!(request.message, "This is a test message");
        assert_eq!(request.level, "info");
        assert_eq!(request.target_user, Some("user123".to_string()));
    }
    
    #[test]
    fn should_deserialize_payment_simulation_request() {
        let json = r#"{
            "payment_id": "pay_123",
            "user_id": "user_456",
            "amount": 99.99,
            "currency": "USD",
            "event_type": "completed",
            "transaction_id": "txn_789"
        }"#;
        
        let request: SimulatePaymentRequest = serde_json::from_str(json).unwrap();
        
        assert_eq!(request.payment_id, "pay_123");
        assert_eq!(request.user_id, "user_456");
        assert_eq!(request.amount, 99.99);
        assert_eq!(request.currency, "USD");
        assert_eq!(request.event_type, "completed");
        assert_eq!(request.transaction_id, Some("txn_789".to_string()));
    }
}