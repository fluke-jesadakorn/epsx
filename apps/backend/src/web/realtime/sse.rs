use crate::domain::shared_kernel::value_objects::UserId;
// Server-Sent Events implementation for real-time updates

use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        Response,
    },
};
use futures::Stream;
use serde::Deserialize;
use std::{convert::Infallible, time::Duration};
use tokio::sync::broadcast;
use tokio_stream::{wrappers::BroadcastStream, StreamExt as _};
use crate::web::middleware::clean_auth::AuthenticatedUser;
use tracing::{info, error};
use super::events::{EventMessage, RealtimeEvent};
use super::super::auth::routes::AppState;

/// SSE connection parameters
#[derive(Debug, Deserialize)]
pub struct SseParams {
    /// Types of events to subscribe to
    pub events: Option<String>, // comma-separated: "payment,stock,notification"
    /// Specific symbols for stock updates  
    pub symbols: Option<String>, // comma-separated stock symbols
    /// Last event ID for reconnection
    pub last_event_id: Option<String>,
}

/// SSE handler for real-time events
pub async fn sse_handler(
    Query(params): Query<SseParams>,
    auth_ctx: AuthenticatedUser,
    State(_app_state): State<AppState>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    // Extract user from session and convert to UserId
    let user_id = crate::domain::shared_kernel::value_objects::UserId::from_string_unchecked(auth_ctx.user_id);
    
    // Parse subscribed events
    let subscribed_events: Vec<String> = params.events
        .unwrap_or_else(|| "payment,notification".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
        
    // Parse subscribed symbols
    let subscribed_symbols: Vec<String> = params.symbols
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    
    info!("SSE connection from user: {} with events: {:?}", user_id.to_string(), subscribed_events);
    
    // Get event broadcaster (in production, this would be in AppState)
    let (event_sender, event_receiver) = broadcast::channel::<EventMessage>(1000);
    
    // Start background task to simulate events (for testing)
    tokio::spawn(simulate_events(event_sender.clone()));
    
    // Clone data for the stream
    let user_id_clone = user_id.clone();
    let subscribed_events_clone = subscribed_events.clone();
    let subscribed_symbols_clone = subscribed_symbols.clone();
    
    // Create event stream
    let stream = BroadcastStream::new(event_receiver)
        .map(move |result| {
            let user_id = user_id_clone.clone();
            let subscribed_events = subscribed_events_clone.clone();
            let subscribed_symbols = subscribed_symbols_clone.clone();
            
            match result {
                Ok(event) => {
                    // Filter events based on subscription
                    if should_send_sse_event(&event, &user_id, &subscribed_events, &subscribed_symbols) {
                        match create_sse_event(event) {
                            Ok(sse_event) => Ok(sse_event),
                            Err(e) => {
                                error!("Failed to create SSE event: {:?}", e);
                                Ok(Event::default()
                                    .event("error")
                                    .data("Failed to process event"))
                            }
                        }
                    } else {
                        // Send heartbeat to keep connection alive
                        Ok(Event::default()
                            .event("heartbeat")
                            .data("ping"))
                    }
                },
                Err(e) => {
                    error!("Broadcast error in SSE stream: {:?}", e);
                    Ok(Event::default()
                        .event("error")
                        .data("Stream error"))
                }
            }
        });
    
    // Send initial connection event
    let initial_stream = tokio_stream::once(Ok(Event::default()
        .event("connected")
        .data(serde_json::json!({
            "user_id": user_id.to_string(),
            "subscribed_events": subscribed_events,
            "subscribed_symbols": subscribed_symbols,
            "timestamp": chrono::Utc::now()
        }).to_string())));
    
    let combined_stream = initial_stream.chain(stream);
    
    Ok(Sse::new(combined_stream)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(30))
                .text("keep-alive-text")
        ))
}

/// Convert EventMessage to SSE Event
fn create_sse_event(event_msg: EventMessage) -> Result<Event, serde_json::Error> {
    let event_type = match &event_msg.event {
        RealtimeEvent::PaymentStarted { .. } => "payment_started",
        RealtimeEvent::PaymentCompleted { .. } => "payment_completed", 
        RealtimeEvent::PaymentFailed { .. } => "payment_failed",
        RealtimeEvent::StockPriceUpdate { .. } => "stock_price_update",
        RealtimeEvent::TradeExecuted { .. } => "trade_executed",
        RealtimeEvent::SystemNotification { .. } => "notification",
        RealtimeEvent::SubscriptionUpgraded { .. } => "subscription_upgraded",
        RealtimeEvent::SubscriptionExpired { .. } => "subscription_expired",
        RealtimeEvent::HealthAlert { .. } => "health_alert",
        RealtimeEvent::FeatureExpirationWarning { .. } => "feature_expiration_warning",
        RealtimeEvent::FeatureExpired { .. } => "feature_expired",
        RealtimeEvent::GracePeriodStarted { .. } => "grace_period_started",
        RealtimeEvent::GracePeriodEnding { .. } => "grace_period_ending",
    };
    
    let data = serde_json::to_string(&event_msg)?;
    
    Ok(Event::default()
        .id(event_msg.metadata.event_id)
        .event(event_type)
        .data(data))
}

/// Determine if an SSE event should be sent to a user
fn should_send_sse_event(
    event: &EventMessage,
    user_id: &UserId,
    subscribed_events: &[String],
    subscribed_symbols: &[String],
) -> bool {
    // Check if user is specifically targeted
    if let Some(target_user) = &event.metadata.user_id {
        if target_user != &user_id.to_string() {
            return false;
        }
    }
    
    // Check event type subscription
    let event_type = match &event.event {
        RealtimeEvent::PaymentStarted { .. } | 
        RealtimeEvent::PaymentCompleted { .. } | 
        RealtimeEvent::PaymentFailed { .. } => "payment",
        RealtimeEvent::StockPriceUpdate { .. } | 
        RealtimeEvent::TradeExecuted { .. } => "stock",
        RealtimeEvent::SystemNotification { .. } => "notification",
        RealtimeEvent::SubscriptionUpgraded { .. } |
        RealtimeEvent::SubscriptionExpired { .. } => "subscription",
        RealtimeEvent::HealthAlert { .. } => "health",
        RealtimeEvent::FeatureExpirationWarning { .. } => "feature_expiration_warning",
        RealtimeEvent::FeatureExpired { .. } => "feature_expired",
        RealtimeEvent::GracePeriodStarted { .. } => "grace_period_started",
        RealtimeEvent::GracePeriodEnding { .. } => "grace_period_ending",
    };
    
    if !subscribed_events.contains(&event_type.to_string()) {
        return false;
    }
    
    // For stock events, check symbol subscription
    if event_type == "stock" {
        if let RealtimeEvent::StockPriceUpdate { symbol, .. } = &event.event {
            if !subscribed_symbols.is_empty() && 
               !subscribed_symbols.contains(symbol) {
                return false;
            }
        }
    }
    
    true
}

/// Simulate events for testing (remove in production)
async fn simulate_events(sender: broadcast::Sender<EventMessage>) {
    let mut interval = tokio::time::interval(Duration::from_secs(10));
    let mut counter = 0u32;
    
    loop {
        interval.tick().await;
        counter += 1;
        
        let event = match counter % 4 {
            0 => RealtimeEvent::payment_started(
                format!("pay_{}", counter),
                "test_user".to_string(),
                99.99,
                "USD".to_string()
            ),
            1 => RealtimeEvent::StockPriceUpdate {
                symbol: "AAPL".to_string(),
                price: 150.0 + (counter as f64 * 0.1),
                change: 0.5,
                change_percent: 0.33,
                volume: 1000000,
                timestamp: chrono::Utc::now(),
            },
            2 => RealtimeEvent::SystemNotification {
                title: "Test Notification".to_string(),
                message: format!("This is test notification #{}", counter),
                level: super::events::NotificationLevel::Info,
                target_user: None,
                metadata: std::collections::HashMap::new(),
                timestamp: chrono::Utc::now(),
            },
            _ => RealtimeEvent::payment_completed(
                format!("pay_{}", counter - 3), 
                "test_user".to_string(),
                99.99,
                "USD".to_string(),
                format!("txn_{}", counter)
            ),
        };
        
        let event_msg = EventMessage::new(event, "sse-simulator".to_string());
        
        if let Err(e) = sender.send(event_msg) {
            error!("Failed to send simulated event: {:?}", e);
            break;
        }
    }
}

/// Health endpoint for SSE connections
pub async fn sse_health_handler() -> Result<Response, StatusCode> {
    let health_data = serde_json::json!({
        "status": "healthy",
        "service": "sse",
        "timestamp": chrono::Utc::now(),
        "connections": 0, // TODO: Track actual connections
    });
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(health_data.to_string().into())
        .unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::shared_kernel::value_objects::UserId;
    
    #[test]
    fn should_create_sse_event() {
        let event_msg = EventMessage::new(
            RealtimeEvent::payment_started(
                "pay123".to_string(),
                "user123".to_string(),
                100.0,
                "USD".to_string()
            ),
            "test".to_string()
        );
        
        let sse_event = create_sse_event(event_msg).unwrap();
        
        // Event should have proper type and contain payment data
        let data = sse_event.data.unwrap();
        assert!(data.contains("payment_started") || data.contains("PaymentStarted"));
        assert!(data.contains("pay123"));
        assert!(data.contains("user123"));
    }
    
    #[test]
    fn should_filter_sse_events_by_subscription() {
        let user_id = UserId::new("user123".to_string());
        let subscribed_events = vec!["payment".to_string()];
        let subscribed_symbols = vec![];
        
        let payment_event = EventMessage::new(
            RealtimeEvent::payment_started(
                "pay123".to_string(),
                "user123".to_string(),
                100.0,
                "USD".to_string()
            ),
            "test".to_string()
        );
        
        let stock_event = EventMessage::new(
            RealtimeEvent::StockPriceUpdate {
                symbol: "AAPL".to_string(),
                price: 150.0,
                change: 2.5,
                change_percent: 1.7,
                volume: 1000000,
                timestamp: chrono::Utc::now(),
            },
            "test".to_string()
        );
        
        assert!(should_send_sse_event(&payment_event, &user_id, &subscribed_events, &subscribed_symbols));
        assert!(!should_send_sse_event(&stock_event, &user_id, &subscribed_events, &subscribed_symbols));
    }
    
    #[test]
    fn should_filter_stock_events_by_symbol() {
        let user_id = UserId::new("user123".to_string());
        let subscribed_events = vec!["stock".to_string()];
        let subscribed_symbols = vec!["AAPL".to_string(), "GOOGL".to_string()];
        
        let aapl_event = EventMessage::new(
            RealtimeEvent::StockPriceUpdate {
                symbol: "AAPL".to_string(),
                price: 150.0,
                change: 2.5,
                change_percent: 1.7,
                volume: 1000000,
                timestamp: chrono::Utc::now(),
            },
            "test".to_string()
        );
        
        let tsla_event = EventMessage::new(
            RealtimeEvent::StockPriceUpdate {
                symbol: "TSLA".to_string(),
                price: 200.0,
                change: -5.0,
                change_percent: -2.4,
                volume: 500000,
                timestamp: chrono::Utc::now(),
            },
            "test".to_string()
        );
        
        assert!(should_send_sse_event(&aapl_event, &user_id, &subscribed_events, &subscribed_symbols));
        assert!(!should_send_sse_event(&tsla_event, &user_id, &subscribed_events, &subscribed_symbols));
    }
}