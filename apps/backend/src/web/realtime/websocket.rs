// WebSocket implementation for real-time communication

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State, Query,
    },
    http::StatusCode,
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
};
use tokio::sync::{broadcast, RwLock};
use crate::web::middleware::AuthCtx;
use tracing::{info, warn, error};

use crate::dom::values::UserId;
use super::events::{EventMessage, RealtimeEvent};
use super::super::auth::routes::AppState;

/// WebSocket connection parameters
#[derive(Debug, Deserialize)]
pub struct WsParams {
    /// Types of events to subscribe to
    pub events: Option<String>, // comma-separated: "payment,stock,notification"
    /// Specific symbols for stock updates
    pub symbols: Option<String>, // comma-separated stock symbols
}

/// WebSocket connection info
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub user_id: UserId,
    pub session_id: String,
    pub subscribed_events: Vec<String>,
    pub subscribed_symbols: Vec<String>,
    pub connected_at: chrono::DateTime<chrono::Utc>,
}

/// Connection manager for WebSocket clients
#[derive(Debug, Clone)]
pub struct ConnectionManager {
    connections: Arc<RwLock<HashMap<String, ConnectionInfo>>>,
    event_sender: broadcast::Sender<EventMessage>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
        }
    }
    
    /// Add a new connection
    pub async fn add_connection(&self, session_id: String, info: ConnectionInfo) {
        let mut connections = self.connections.write().await;
        connections.insert(session_id.clone(), info);
        info!("WebSocket connection added: {}", session_id);
    }
    
    /// Remove a connection
    pub async fn remove_connection(&self, session_id: &str) {
        let mut connections = self.connections.write().await;
        if connections.remove(session_id).is_some() {
            info!("WebSocket connection removed: {}", session_id);
        }
    }
    
    /// Broadcast event to all connected clients
    pub async fn broadcast_event(&self, event: EventMessage) {
        if let Err(e) = self.event_sender.send(event.clone()) {
            error!("Failed to broadcast event: {:?}", e);
        }
    }
    
    /// Send event to specific user
    pub async fn send_to_user(&self, user_id: &UserId, event: EventMessage) {
        let connections = self.connections.read().await;
        let user_connections: Vec<_> = connections
            .values()
            .filter(|conn| conn.user_id == *user_id)
            .collect();
            
        if !user_connections.is_empty() {
            if let Err(e) = self.event_sender.send(event) {
                error!("Failed to send event to user {}: {:?}", user_id.to_string(), e);
            }
        }
    }
    
    /// Get connection stats
    pub async fn get_stats(&self) -> ConnectionStats {
        let connections = self.connections.read().await;
        let total_connections = connections.len();
        let unique_users = connections.values()
            .map(|conn| &conn.user_id)
            .collect::<std::collections::HashSet<_>>()
            .len();
            
        ConnectionStats {
            total_connections,
            unique_users,
            connections_by_event: HashMap::new(), // TODO: Calculate this
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ConnectionStats {
    pub total_connections: usize,
    pub unique_users: usize,
    pub connections_by_event: HashMap<String, usize>,
}

/// WebSocket upgrade handler
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WsParams>,
    auth_ctx: AuthCtx,
    State(app_state): State<AppState>,
) -> Result<Response, StatusCode> {
    let user_id = auth_ctx.user_id;
    let session_id = uuid::Uuid::new_v4().to_string();
    
    // Parse subscribed events
    let subscribed_events = params.events
        .unwrap_or_else(|| "payment,notification".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
        
    // Parse subscribed symbols  
    let subscribed_symbols = params.symbols
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    
    let connection_info = ConnectionInfo {
        user_id: user_id.clone(),
        session_id: session_id.clone(),
        subscribed_events,
        subscribed_symbols,
        connected_at: chrono::Utc::now(),
    };
    
    info!("WebSocket connection request from user: {}", user_id.to_string());
    
    Ok(ws.on_upgrade(move |socket| {
        handle_websocket(socket, session_id, connection_info, app_state)
    }))
}

/// Handle individual WebSocket connection
async fn handle_websocket(
    socket: WebSocket,
    session_id: String,
    connection_info: ConnectionInfo,
    app_state: AppState,
) {
    let (mut sender, mut receiver) = socket.split();
    let user_id = connection_info.user_id.clone();
    
    // Get connection manager from app state
    let connection_manager = get_connection_manager(&app_state);
    
    // Add connection to manager
    connection_manager.add_connection(session_id.clone(), connection_info.clone()).await;
    
    // Subscribe to broadcast events
    let mut event_receiver = connection_manager.event_sender.subscribe();
    
    // Send welcome message
    let welcome_msg = serde_json::json!({
        "type": "connection_established",
        "session_id": session_id,
        "subscribed_events": connection_info.subscribed_events,
        "subscribed_symbols": connection_info.subscribed_symbols,
        "timestamp": chrono::Utc::now()
    });
    
    if let Err(e) = sender.send(Message::Text(welcome_msg.to_string())).await {
        error!("Failed to send welcome message: {:?}", e);
        return;
    }
    
    // Handle incoming messages and outgoing events concurrently
    let session_id_clone = session_id.clone();
    let connection_manager_clone = connection_manager.clone();
    let connection_info_clone = connection_info.clone();
    let user_id_clone = user_id.clone();
    
    let receive_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    info!("Received WebSocket message from {}: {}", user_id_clone.to_string(), text);
                    // Handle client messages (ping, subscribe, etc.)
                    if let Err(e) = handle_client_message(text, &connection_info_clone).await {
                        warn!("Error handling client message: {:?}", e);
                    }
                },
                Message::Close(_) => {
                    info!("WebSocket connection closed by client: {}", user_id_clone.to_string());
                    break;
                },
                _ => {
                    // Handle other message types (binary, ping, pong)
                }
            }
        }
        
        // Clean up connection
        connection_manager_clone.remove_connection(&session_id_clone).await;
    });
    
    let send_task = tokio::spawn(async move {
        while let Ok(event) = event_receiver.recv().await {
            // Filter events based on subscription
            if should_send_event(&event, &connection_info) {
                let message = serde_json::to_string(&event).unwrap_or_else(|_| {
                    r#"{"type": "error", "message": "Failed to serialize event"}"#.to_string()
                });
                
                if let Err(e) = sender.send(Message::Text(message)).await {
                    error!("Failed to send event to WebSocket: {:?}", e);
                    break;
                }
            }
        }
    });
    
    // Wait for either task to complete
    tokio::select! {
        _ = receive_task => {
            info!("WebSocket receive task completed for user: {}", user_id.to_string());
        },
        _ = send_task => {
            info!("WebSocket send task completed for user: {}", user_id.to_string());
        }
    }
    
    // Clean up
    connection_manager.remove_connection(&session_id).await;
}

/// Handle client messages (ping, subscribe updates, etc.)
async fn handle_client_message(
    message: String,
    _connection_info: &ConnectionInfo,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let parsed: serde_json::Value = serde_json::from_str(&message)?;
    
    match parsed["type"].as_str() {
        Some("ping") => {
            // Handle ping - could send pong back
            info!("Received ping from client");
        },
        Some("subscribe") => {
            // Handle subscription updates
            info!("Client subscription update: {}", message);
        },
        _ => {
            warn!("Unknown client message type: {}", message);
        }
    }
    
    Ok(())
}

/// Determine if an event should be sent to a specific connection
fn should_send_event(event: &EventMessage, connection_info: &ConnectionInfo) -> bool {
    // Check if user is specifically targeted
    if let Some(target_user) = &event.metadata.user_id {
        if target_user != &connection_info.user_id.to_string() {
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
    };
    
    if !connection_info.subscribed_events.contains(&event_type.to_string()) {
        return false;
    }
    
    // For stock events, check symbol subscription
    if event_type == "stock" {
        if let RealtimeEvent::StockPriceUpdate { symbol, .. } = &event.event {
            if !connection_info.subscribed_symbols.is_empty() && 
               !connection_info.subscribed_symbols.contains(symbol) {
                return false;
            }
        }
    }
    
    true
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
    use crate::dom::values::UserId;
    
    #[test]
    fn should_filter_events_by_subscription() {
        let connection_info = ConnectionInfo {
            user_id: UserId::new("user123".to_string()),
            session_id: "session123".to_string(),
            subscribed_events: vec!["payment".to_string()],
            subscribed_symbols: vec![],
            connected_at: chrono::Utc::now(),
        };
        
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
        
        assert!(should_send_event(&payment_event, &connection_info));
        assert!(!should_send_event(&stock_event, &connection_info));
    }
    
    #[test]
    fn should_filter_events_by_user_targeting() {
        let connection_info = ConnectionInfo {
            user_id: UserId::new("user123".to_string()),
            session_id: "session123".to_string(),
            subscribed_events: vec!["payment".to_string()],
            subscribed_symbols: vec![],
            connected_at: chrono::Utc::now(),
        };
        
        let targeted_event = EventMessage::new(
            RealtimeEvent::payment_started(
                "pay123".to_string(),
                "user456".to_string(), // Different user
                100.0,
                "USD".to_string()
            ),
            "test".to_string()
        ).with_user_id("user456".to_string());
        
        assert!(!should_send_event(&targeted_event, &connection_info));
    }
}