// Real-time handlers for event broadcasting and management

use axum::{

    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

use crate::web::middleware::AuthCtx;
use crate::infrastructure::integration::PaymentEventType;

use tracing::{info, error};

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

/// Connection statistics for monitoring
#[derive(Debug, Serialize)]
pub struct ConnectionStats {
    pub total_connections: usize,
    pub unique_users: usize,
    pub connections_by_event: std::collections::HashMap<String, usize>,
}

/// Broadcast a system notification to all or specific users
pub async fn broadcast_notification_handler(
    auth_ctx: AuthCtx,
    State(app_state): State<AppState>,
    Json(payload): Json<BroadcastNotificationRequest>,
) -> Result<Json<BroadcastResponse>, StatusCode> {
    // Verify admin access
    let currentuser_id = auth_ctx.user_id;
    verify_admin_access(&app_state, &currentuser_id).await?;
    
    // Parse notification level for DDD
    let level = match payload.level.to_lowercase().as_str() {
        "info" => crate::domain::realtime_events::value_objects::NotificationLevel::Info,
        "warning" => crate::domain::realtime_events::value_objects::NotificationLevel::Warning,
        "error" => crate::domain::realtime_events::value_objects::NotificationLevel::Error,
        "success" => crate::domain::realtime_events::value_objects::NotificationLevel::Success,
        _ => crate::domain::realtime_events::value_objects::NotificationLevel::Info,
    };
    
    // Use DDD Real-time Events service
    let realtime_service = &app_state.ddd_container.realtime_events_service;
    let result = realtime_service.broadcast_system_notification(
        payload.title,
        payload.message,
        level,
        payload.target_user,
        "admin-notification".to_string(),
    ).await.map_err(|e| {
        error!("Failed to broadcast notification: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Log event
    info!("Admin {} broadcasted notification: {}", 
          currentuser_id.to_string(), result.event_id);
    
    Ok(Json(BroadcastResponse {
        success: true,
        message: "Notification broadcasted successfully".to_string(),
        event_id: result.event_id,
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
    let currentuser_id = auth_ctx.user_id;
    verify_admin_access(&app_state, &currentuser_id).await?;
    
    // Map event type to DDD enum
    let event_type = match payload.event_type.as_str() {
        "started" => PaymentEventType::Started,
        "completed" => PaymentEventType::Completed,
        "failed" => PaymentEventType::Failed,
        _ => return Err(StatusCode::BAD_REQUEST),
    };
    
    // Use DDD Real-time Events service
    let realtime_service = &app_state.ddd_container.realtime_events_service;
    let result = realtime_service.simulate_payment_event(
        payload.payment_id,
        payload.user_id,
        payload.amount,
        payload.currency,
        event_type,
        payload.transaction_id,
        payload.error_code,
        payload.error_message,
    ).await.map_err(|e| {
        error!("Failed to simulate payment event: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Log event
    info!("Admin {} simulated payment event: {}", 
          currentuser_id.to_string(), result.event_id);
    
    Ok(Json(BroadcastResponse {
        success: true,
        message: format!("Payment {} event simulated successfully", payload.event_type),
        event_id: result.event_id,
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
    let currentuser_id = auth_ctx.user_id;
    verify_admin_access(&app_state, &currentuser_id).await?;
    
    // Use DDD Real-time Events service
    let realtime_service = &app_state.ddd_container.realtime_events_service;
    let result = realtime_service.simulate_stock_price_update(
        payload.symbol,
        payload.price,
        payload.change,
        payload.change_percent,
        payload.volume,
    ).await.map_err(|e| {
        error!("Failed to simulate stock price update: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Log event
    info!("Admin {} simulated stock update: {}", 
          currentuser_id.to_string(), result.event_id);
    
    Ok(Json(BroadcastResponse {
        success: true,
        message: "Stock price update simulated successfully".to_string(),
        event_id: result.event_id,
        timestamp: chrono::Utc::now(),
    }))
}

/// Get real-time connection statistics
pub async fn get_connection_stats_handler(
    auth_ctx: AuthCtx,
    State(app_state): State<AppState>,
) -> Result<Json<ConnectionStats>, StatusCode> {
    // Verify admin access
    let currentuser_id = auth_ctx.user_id;
    verify_admin_access(&app_state, &currentuser_id).await?;
    
    // Use DDD Real-time Events service to get stats
    let realtime_service = &app_state.ddd_container.realtime_events_service;
    let ddd_stats = realtime_service.get_connection_stats().await
        .map_err(|e| {
            error!("Failed to get connection stats: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Convert DDD stats to API response format
    let stats = ConnectionStats {
        total_connections: ddd_stats.total_connections,
        unique_users: ddd_stats.unique_users,
        connections_by_event: std::collections::HashMap::new(), // TODO: Implement event-based grouping
    };
    
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
    let currentuser_id = auth_ctx.user_id;
    verify_admin_access(&app_state, &currentuser_id).await?;
    
    // Parse notification level for DDD
    let level = match payload.level.to_lowercase().as_str() {
        "info" => crate::domain::realtime_events::value_objects::NotificationLevel::Info,
        "warning" => crate::domain::realtime_events::value_objects::NotificationLevel::Warning,
        "error" => crate::domain::realtime_events::value_objects::NotificationLevel::Error,
        "success" => crate::domain::realtime_events::value_objects::NotificationLevel::Success,
        _ => crate::domain::realtime_events::value_objects::NotificationLevel::Info,
    };
    
    // Use DDD Real-time Events service
    let realtime_service = &app_state.ddd_container.realtime_events_service;
    let result = realtime_service.broadcast_system_notification(
        payload.title,
        payload.message,
        level,
        Some(user_id.clone()),
        "admin-targeted-notification".to_string(),
    ).await.map_err(|e| {
        error!("Failed to send targeted notification: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Log event
    info!("Admin {} sent targeted notification to {}: {}", 
          currentuser_id.to_string(), user_id, result.event_id);
    
    Ok(Json(BroadcastResponse {
        success: true,
        message: "Targeted notification sent successfully".to_string(),
        event_id: result.event_id,
        timestamp: chrono::Utc::now(),
    }))
}


/// Verify admin access using DDD User aggregate
async fn verify_admin_access(app_state: &AppState, user_id: &crate::domain::shared_kernel::value_objects::UserId) -> Result<(), StatusCode> {
    // Use DDD User Repository Port through DDDContainer
    let user_repository = app_state.ddd_container.user_repository();
    
    let user = user_repository.find_by_id(user_id).await
        .map_err(|e| {
            error!("Failed to get user for admin check: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check if user has admin permissions
    // TODO: Implement proper permission checking when PermissionApplicationService is ready
    // For now, use User aggregate's active_permissions method
    let user_permissions = user.active_permissions();
    
    if user_permissions.iter().any(|p| p.starts_with("admin:")) {
        Ok(())
    } else {
        tracing::warn!("Non-admin user {} attempted admin operation", user_id.to_string());
        Err(StatusCode::FORBIDDEN)
    }
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