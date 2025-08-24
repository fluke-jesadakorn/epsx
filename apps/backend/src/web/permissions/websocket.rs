// Permission WebSocket and Real-time Updates
//
// Provides real-time permission updates via WebSocket connections and Server-Sent Events,
// enabling live permission state synchronization across applications with efficient
// change detection and minimal latency.

use axum::{
    extract::ws::{Message, WebSocket},
    extract::State,
    response::sse::{Event, KeepAlive},
    response::Sse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde_json::json;
use std::{
    collections::HashMap,
    sync::Arc,
    time::Duration,
};
use tokio::{
    sync::{broadcast, RwLock, Mutex},
    time::interval,
};
// use tokio_stream::wrappers::BroadcastStream;
use uuid::Uuid;
use chrono::Utc;

use crate::{
    infra::container::AppContainer,
    permissions::*,
    dom::values::UserId,
    web::middleware::auth_monitoring::AuthContext,
};

// ============================================================================
// WebSocket Connection Management
// ============================================================================

/// WebSocket connection manager for real-time permission updates
#[derive(Clone)]
pub struct PermissionWebSocketManager {
    // Active connections mapped by user ID
    connections: Arc<RwLock<HashMap<UserId, Vec<ConnectionInfo>>>>,
    
    // Broadcast channel for permission updates
    update_sender: broadcast::Sender<PermissionUpdate>,
    
    // Permission system reference
    container: AppContainer,
}

#[derive(Debug, Clone)]
struct ConnectionInfo {
    id: Uuid,
    user_id: UserId,
    connected_at: chrono::DateTime<chrono::Utc>,
    last_ping: chrono::DateTime<chrono::Utc>,
    subscription_filters: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PermissionUpdate {
    pub update_type: PermissionUpdateType,
    pub user_id: UserId,
    pub permission: Option<String>,
    pub resource: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone)]
pub enum PermissionUpdateType {
    PermissionGranted,
    PermissionRevoked,
    PermissionElevated,
    TierChanged,
    AdminModuleAssigned,
    AdminModuleRevoked,
    PermissionExpired,
    PolicyUpdated,
    SystemMaintenance,
}

impl PermissionWebSocketManager {
    pub fn new(container: AppContainer) -> Self {
        let (tx, _) = broadcast::channel(1000);
        
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            update_sender: tx,
            container,
        }
    }
    
    /// Handle new WebSocket connection
    pub async fn handle_connection(
        &self,
        socket: WebSocket,
        user_id: UserId,
        _auth_context: AuthContext,
    ) {
        let connection_id = Uuid::new_v4();
        let (sender, mut receiver) = socket.split();
        let sender = Arc::new(Mutex::new(sender));
        
        // Create connection info
        let connection = ConnectionInfo {
            id: connection_id,
            user_id: user_id.clone(),
            connected_at: Utc::now(),
            last_ping: Utc::now(),
            subscription_filters: vec!["*".to_string()], // Default to all updates
        };
        
        // Add connection to manager
        {
            let mut connections = self.connections.write().await;
            connections
                .entry(user_id.clone())
                .or_insert_with(Vec::new)
                .push(connection);
        }
        
        tracing::info!(
            "WebSocket connection established: {} for user {}",
            connection_id,
            user_id
        );
        
        // Send initial connection message
        let welcome_message = json!({
            "type": "welcome",
            "connection_id": connection_id,
            "user_id": user_id,
            "timestamp": Utc::now(),
            "server_info": {
                "version": "1.0.0",
                "capabilities": ["permissions", "real_time_updates", "batch_operations"]
            }
        });
        
        if sender
            .lock()
            .await
            .send(Message::Text(welcome_message.to_string()))
            .await
            .is_err()
        {
            tracing::warn!("Failed to send welcome message to {}", connection_id);
            self.remove_connection(&user_id, connection_id).await;
            return;
        }
        
        // Send initial permissions snapshot
        self.send_permissions_snapshot(&sender, &user_id).await;
        
        // Create broadcast receiver for updates
        let mut update_receiver = self.update_sender.subscribe();
        
        // Handle incoming messages and outgoing updates
        let _connections_clone = self.connections.clone();
        let _container_clone = self.container.clone();
        let _user_id_clone = user_id.clone();
        let sender_clone = sender.clone();
        let sender_clone2 = sender.clone();
        
        tokio::select! {
            // Handle incoming WebSocket messages
            _ = async {
                while let Some(msg) = receiver.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            self.handle_client_message(text, &user_id, connection_id).await;
                        }
                        Ok(Message::Binary(_)) => {
                            // Binary messages not supported
                            let error_msg = json!({
                                "type": "error",
                                "message": "Binary messages not supported",
                                "timestamp": Utc::now()
                            });
                            
                            if sender_clone.lock().await.send(Message::Text(error_msg.to_string())).await.is_err() {
                                break;
                            }
                        }
                        Ok(Message::Ping(data)) => {
                            // Respond to ping
                            if sender_clone.lock().await.send(Message::Pong(data)).await.is_err() {
                                break;
                            }
                            
                            // Update last ping time
                            self.update_connection_ping(&user_id, connection_id).await;
                        }
                        Ok(Message::Close(_)) => {
                            tracing::info!("WebSocket connection closed by client: {}", connection_id);
                            break;
                        }
                        Err(e) => {
                            tracing::error!("WebSocket error for {}: {}", connection_id, e);
                            break;
                        }
                        _ => {}
                    }
                }
            } => {},
            
            // Handle outgoing permission updates
            _ = async {
                while let Ok(update) = update_receiver.recv().await {
                    // Check if this update is relevant to the user
                    if update.user_id == user_id || self.is_global_update(&update) {
                        let update_message = json!({
                            "type": "permission_update",
                            "update_type": format!("{:?}", update.update_type),
                            "user_id": update.user_id,
                            "permission": update.permission,
                            "resource": update.resource,
                            "timestamp": update.timestamp,
                            "metadata": update.metadata
                        });
                        
                        if sender_clone2.lock().await.send(Message::Text(update_message.to_string())).await.is_err() {
                            break;
                        }
                    }
                }
            } => {}
        }
        
        // Cleanup connection
        self.remove_connection(&user_id, connection_id).await;
        tracing::info!("WebSocket connection ended: {} for user {}", connection_id, user_id);
    }
    
    /// Send initial permissions snapshot to client
    async fn send_permissions_snapshot(&self, sender: &Arc<Mutex<futures::stream::SplitSink<WebSocket, Message>>>, user_id: &UserId) {
        // Get permission system
        let permission_system = match self.container.get_permission_system() {
            Ok(system) => system,
            Err(e) => {
                tracing::error!("Failed to get permission system: {}", e);
                return;
            }
        };
        
        // Get user permissions
        match permission_system.get_permissions(user_id).await {
            Ok(permissions) => {
                let snapshot = json!({
                    "type": "permissions_snapshot",
                    "user_id": user_id,
                    "permissions": permissions.iter().map(|p| json!({
                        "name": p.name(),
                        "resource": p.resource,
                        "granted_at": p.granted_at(),
                        "expires_at": p.expires_at,
                        "source": p.source()
                    })).collect::<Vec<_>>(),
                    "timestamp": Utc::now()
                });
                
                if sender
                    .lock()
                    .await
                    .send(Message::Text(snapshot.to_string()))
                    .await
                    .is_err()
                {
                    tracing::warn!("Failed to send permissions snapshot to user {}", user_id);
                }
            }
            Err(e) => {
                tracing::error!("Failed to get permissions for user {}: {}", user_id, e);
                
                let error_msg = json!({
                    "type": "error",
                    "message": "Failed to load permissions",
                    "timestamp": Utc::now()
                });
                
                let _ = sender.lock().await.send(Message::Text(error_msg.to_string())).await;
            }
        }
    }
    
    /// Handle client messages
    async fn handle_client_message(&self, text: String, user_id: &UserId, connection_id: Uuid) {
        let message: serde_json::Value = match serde_json::from_str(&text) {
            Ok(msg) => msg,
            Err(e) => {
                tracing::warn!("Invalid JSON message from {}: {}", connection_id, e);
                return;
            }
        };
        
        let msg_type = message.get("type").and_then(|t| t.as_str());
        
        match msg_type {
            Some("ping") => {
                // Handle client ping
                self.update_connection_ping(user_id, connection_id).await;
            }
            Some("subscribe") => {
                // Handle subscription updates
                if let Some(filters) = message.get("filters").and_then(|f| f.as_array()) {
                    let filter_strings: Vec<String> = filters
                        .iter()
                        .filter_map(|f| f.as_str().map(|s| s.to_string()))
                        .collect();
                    
                    self.update_subscription_filters(user_id, connection_id, filter_strings).await;
                }
            }
            Some("validate_permission") => {
                // Handle real-time permission validation requests
                if let (Some(permission), resource) = (
                    message.get("permission").and_then(|p| p.as_str()),
                    message.get("resource").and_then(|r| r.as_str()),
                ) {
                    self.handle_validation_request(user_id, permission, resource.map(|s| s.to_string()), connection_id).await;
                }
            }
            Some("get_permissions") => {
                // Handle permission refresh requests
                // This will be handled by sending a new snapshot
            }
            _ => {
                tracing::debug!("Unknown message type from {}: {:?}", connection_id, msg_type);
            }
        }
    }
    
    /// Handle real-time permission validation
    async fn handle_validation_request(
        &self,
        user_id: &UserId,
        permission: &str,
        resource: Option<String>,
        connection_id: Uuid,
    ) {
        let permission_system = match self.container.get_permission_system() {
            Ok(system) => system,
            Err(_) => return,
        };
        
        let resource_value = resource.unwrap_or_else(|| "global".to_string());
        let context = PermissionContext {
            user_id: user_id.clone(),
            permission: permission.to_string(),
            resource: resource_value.clone(),
            context_data: {
                let mut data = HashMap::new();
                data.insert("session_id".to_string(), format!("ws:{}", connection_id));
                data.insert("connection_type".to_string(), "websocket".to_string());
                data
            },
            timestamp: Utc::now(),
            ip_address: None,
            user_agent: Some("WebSocket Client".to_string()),
            session_id: Some(format!("ws:{}", connection_id)),
        };
        
        match permission_system.validate_permission(&context).await {
            Ok(result) => {
                // Send validation result back to client
                let validation_result = result.to_result(&context, 0.0);
                let response = json!({
                    "type": "validation_result",
                    "permission": permission,
                    "resource": resource_value,
                    "allowed": validation_result.allowed,
                    "cached": validation_result.cached,
                    "source": validation_result.source,
                    "validation_time_ms": validation_result.validation_time_ms,
                    "timestamp": Utc::now()
                });
                
                self.send_to_connection(user_id, connection_id, response).await;
            }
            Err(e) => {
                let error_response = json!({
                    "type": "validation_error",
                    "permission": permission,
                    "resource": resource_value,
                    "error": e.to_string(),
                    "timestamp": Utc::now()
                });
                
                self.send_to_connection(user_id, connection_id, error_response).await;
            }
        }
    }
    
    /// Broadcast permission update to all relevant connections
    pub async fn broadcast_permission_update(&self, update: PermissionUpdate) {
        // Send update via broadcast channel
        if let Err(e) = self.update_sender.send(update.clone()) {
            tracing::error!("Failed to broadcast permission update: {}", e);
        }
        
        // Also send directly to specific user connections if needed
        if let Some(connections) = self.get_user_connections(&update.user_id).await {
            for connection in connections {
                let update_message = json!({
                    "type": "permission_update",
                    "update_type": format!("{:?}", update.update_type),
                    "user_id": update.user_id,
                    "permission": update.permission,
                    "resource": update.resource,
                    "timestamp": update.timestamp,
                    "metadata": update.metadata
                });
                
                self.send_to_connection(&update.user_id, connection.id, update_message).await;
            }
        }
    }
    
    // Helper methods
    async fn remove_connection(&self, user_id: &UserId, connection_id: Uuid) {
        let mut connections = self.connections.write().await;
        if let Some(user_connections) = connections.get_mut(user_id) {
            user_connections.retain(|c| c.id != connection_id);
            if user_connections.is_empty() {
                connections.remove(user_id);
            }
        }
    }
    
    async fn update_connection_ping(&self, user_id: &UserId, connection_id: Uuid) {
        let mut connections = self.connections.write().await;
        if let Some(user_connections) = connections.get_mut(user_id) {
            for connection in user_connections.iter_mut() {
                if connection.id == connection_id {
                    connection.last_ping = Utc::now();
                    break;
                }
            }
        }
    }
    
    async fn update_subscription_filters(&self, user_id: &UserId, connection_id: Uuid, filters: Vec<String>) {
        let mut connections = self.connections.write().await;
        if let Some(user_connections) = connections.get_mut(user_id) {
            for connection in user_connections.iter_mut() {
                if connection.id == connection_id {
                    connection.subscription_filters = filters;
                    break;
                }
            }
        }
    }
    
    async fn get_user_connections(&self, user_id: &UserId) -> Option<Vec<ConnectionInfo>> {
        let connections = self.connections.read().await;
        connections.get(user_id).cloned()
    }
    
    async fn send_to_connection(&self, _user_id: &UserId, connection_id: Uuid, message: serde_json::Value) {
        // This is a simplified implementation
        // In a real implementation, you'd need to maintain sender channels
        tracing::debug!("Would send message to connection {}: {}", connection_id, message);
    }
    
    fn is_global_update(&self, update: &PermissionUpdate) -> bool {
        matches!(update.update_type, 
            PermissionUpdateType::PolicyUpdated | 
            PermissionUpdateType::SystemMaintenance
        )
    }
}

// ============================================================================
// Server-Sent Events Implementation
// ============================================================================

/// Create SSE stream for permission updates
pub async fn create_permission_sse_stream(
    container: AppContainer,
    user_id: UserId,
    _auth_context: AuthContext,
) -> Sse<impl futures::Stream<Item = Result<Event, axum::BoxError>>> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    
    // Send initial permissions
    tokio::spawn(async move {
        let permission_system = match container.get_permission_system() {
            Ok(system) => system,
            Err(e) => {
                tracing::error!("Failed to get permission system for SSE: {}", e);
                return;
            }
        };
        
        // Send initial snapshot
        match permission_system.get_permissions(&user_id).await {
            Ok(permissions) => {
                let snapshot = json!({
                    "type": "permissions_snapshot",
                    "user_id": user_id,
                    "permissions": permissions.iter().map(|p| json!({
                        "name": p.name(),
                        "resource": p.resource,
                        "source": p.source()
                    })).collect::<Vec<_>>(),
                    "timestamp": Utc::now()
                });
                
                let event = Event::default()
                    .event("permissions_snapshot")
                    .data(snapshot.to_string());
                
                let _ = tx.send(Ok(event));
            }
            Err(e) => {
                tracing::error!("Failed to get initial permissions for SSE: {}", e);
            }
        }
        
        // Send periodic heartbeat
        let mut heartbeat = interval(Duration::from_secs(30));
        
        loop {
            heartbeat.tick().await;
            
            let heartbeat_event = Event::default()
                .event("heartbeat")
                .data(json!({
                    "timestamp": Utc::now(),
                    "user_id": user_id
                }).to_string());
            
            if tx.send(Ok(heartbeat_event)).is_err() {
                break;
            }
        }
    });
    
    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
    
    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive-text"),
    )
}

// ============================================================================
// Permission Change Detection
// ============================================================================

/// Service that monitors permission changes and triggers real-time updates
pub struct PermissionChangeDetector {
    websocket_manager: PermissionWebSocketManager,
    container: AppContainer,
}

impl PermissionChangeDetector {
    pub fn new(websocket_manager: PermissionWebSocketManager, container: AppContainer) -> Self {
        Self {
            websocket_manager,
            container,
        }
    }
    
    /// Start monitoring for permission changes
    pub async fn start_monitoring(&self) {
        let mut check_interval = interval(Duration::from_secs(5));
        
        loop {
            check_interval.tick().await;
            
            // Check for permission changes
            // This is a simplified implementation
            // In practice, you'd use database triggers, event sourcing, or message queues
            
            self.detect_and_broadcast_changes().await;
        }
    }
    
    async fn detect_and_broadcast_changes(&self) {
        // TODO: Implement actual change detection logic
        // This could involve:
        // 1. Polling database for changes
        // 2. Listening to message queue events
        // 3. Using database triggers
        // 4. Event sourcing integration
        
        tracing::debug!("Checking for permission changes...");
    }
    
    /// Manually trigger a permission update broadcast
    pub async fn broadcast_update(&self, update: PermissionUpdate) {
        self.websocket_manager.broadcast_permission_update(update).await;
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Create permission update from permission change event
pub fn create_permission_update(
    update_type: PermissionUpdateType,
    user_id: UserId,
    permission: Option<String>,
    resource: Option<String>,
    metadata: serde_json::Value,
) -> PermissionUpdate {
    PermissionUpdate {
        update_type,
        user_id,
        permission,
        resource,
        timestamp: Utc::now(),
        metadata,
    }
}

// Temporary permission event enum for WebSocket handling
#[derive(Debug, Clone)]
pub enum PermissionEvent {
    PermissionGranted { user_id: String, permission: String },
    PermissionRevoked { user_id: String, permission: String },
}

/// Convert domain events to WebSocket updates
pub async fn handle_permission_event(
    event: &PermissionEvent,
    websocket_manager: &PermissionWebSocketManager,
) {
    let update = match event {
        PermissionEvent::PermissionGranted { user_id, permission } => {
            create_permission_update(
                PermissionUpdateType::PermissionGranted,
                UserId::new(user_id.clone()),
                Some(permission.clone()),
                None,
                json!({ "event": "permission_granted" }),
            )
        }
        PermissionEvent::PermissionRevoked { user_id, permission } => {
            create_permission_update(
                PermissionUpdateType::PermissionRevoked,
                UserId::new(user_id.clone()),
                Some(permission.clone()),
                None,
                json!({ "event": "permission_revoked" }),
            )
        }
    };
    
    websocket_manager.broadcast_permission_update(update).await;
}

// ============================================================================
// Integration with Permission System
// ============================================================================

/// Initialize WebSocket manager and integrations
pub fn init_realtime_permissions(container: AppContainer) -> PermissionWebSocketManager {
    let manager = PermissionWebSocketManager::new(container.clone());
    
    // Start change detector
    let detector = PermissionChangeDetector::new(manager.clone(), container);
    tokio::spawn(async move {
        detector.start_monitoring().await;
    });
    
    manager
}

/// WebSocket handler wrapper for easy integration
pub async fn handle_permission_websocket(
    ws: axum::extract::WebSocketUpgrade,
    State(_container): State<AppContainer>,
    user_id: UserId,
    auth_context: AuthContext,
) -> axum::response::Response {
    let manager = init_realtime_permissions(_container);
    
    ws.on_upgrade(move |socket| async move {
        manager.handle_connection(socket, user_id, auth_context).await;
    })
}

/// SSE handler wrapper
pub async fn handle_permission_sse(
    State(_container): State<AppContainer>,
    user_id: UserId,
    auth_context: AuthContext,
) -> Sse<impl futures::Stream<Item = Result<Event, axum::BoxError>>> {
    create_permission_sse_stream(_container, user_id, auth_context).await
}