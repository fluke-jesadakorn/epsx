// Real-time Events Service Integration
// Orchestrates Real-time Events bounded context for web layer

use std::sync::Arc;

use crate::domain::realtime_events::{
    RealtimeEvent, EventStatus, EventPriority, RealtimeEventError,
    EventPayload, NotificationLevel, RealtimeUserId,
    ConnectionId, ConnectionRepositoryPort, EventRepositoryPort,
    value_objects::{ConnectionInfo, ConnectionType}
};
use crate::infrastructure::adapters::repositories::{
    RealtimeEventRepositoryAdapter, ConnectionRepositoryAdapter
};

/// Integration service for Real-time Events bounded context
/// Provides high-level operations for the web layer
pub struct RealtimeEventsServiceIntegration {
    event_repository: Arc<RealtimeEventRepositoryAdapter>,
    connection_repository: Arc<ConnectionRepositoryAdapter>,
}

impl RealtimeEventsServiceIntegration {
    pub fn new(
        event_repository: Arc<RealtimeEventRepositoryAdapter>,
        connection_repository: Arc<ConnectionRepositoryAdapter>,
    ) -> Self {
        Self {
            event_repository,
            connection_repository,
        }
    }
}

/// High-level operations for Real-time Events
impl RealtimeEventsServiceIntegration {
    /// Broadcast a system notification to all users or a specific user
    pub async fn broadcast_system_notification(
        &self,
        title: String,
        message: String,
        level: NotificationLevel,
        target_user: Option<String>,
        source: String,
    ) -> Result<EventBroadcastResult, RealtimeEventError> {
        // Convert target user to domain type if provided
        let target_users = if let Some(user_id) = target_user {
            vec![RealtimeUserId::new(user_id.to_string()).map_err(|e| RealtimeEventError::InvalidPayload(format!("Invalid user ID: {:?}", e)))?]
        } else {
            vec![]
        };

        // Create event payload
        let payload = EventPayload::system_notification(
            title,
            message,
            level,
            target_users.first().cloned(),
        );

        // Create real-time event
        let event = if target_users.is_empty() {
            // Broadcast to all users
            RealtimeEvent::create_broadcast(payload, "notifications".to_string())?
        } else {
            // Target specific users
            RealtimeEvent::create(payload, target_users, "notifications".to_string())?
        };

        // Save event
        self.event_repository.save(&event).await
            .map_err(|e| RealtimeEventError::InvalidPayload(e))?;

        tracing::info!(
            event_id = %event.id().to_string(),
            source = source,
            channel = event.channel(),
            "Broadcasted system notification"
        );

        Ok(EventBroadcastResult {
            event_id: event.id().to_string(),
            status: event.status().clone(),
            channel: event.channel().to_string(),
            target_user_count: event.target_users().len() as u32,
        })
    }

    /// Simulate a payment event (for testing/admin purposes)
    pub async fn simulate_payment_event(
        &self,
        payment_id: String,
        user_id: String,
        amount: f64,
        currency: String,
        event_type: PaymentEventType,
        transaction_id: Option<String>,
        error_code: Option<String>,
        error_message: Option<String>,
    ) -> Result<EventBroadcastResult, RealtimeEventError> {
        let target_user = RealtimeUserId::new(user_id.to_string())
            .map_err(|e| RealtimeEventError::InvalidPayload(format!("Invalid user ID: {:?}", e)))?;
        let payload = match event_type {
            PaymentEventType::Started => {
                EventPayload::payment_started(payment_id.clone(), target_user.clone(), amount, currency)
            },
            PaymentEventType::Completed => {
                let tx_id = transaction_id.unwrap_or_else(|| format!("txn_{}", uuid::Uuid::new_v4()));
                EventPayload::payment_completed(payment_id.clone(), target_user.clone(), amount, currency, tx_id)
            },
            PaymentEventType::Failed => {
                let error_code = error_code.unwrap_or_else(|| "UNKNOWN_ERROR".to_string());
                let error_msg = error_message.unwrap_or_else(|| "Payment failed".to_string());
                EventPayload::payment_failed(payment_id.clone(), target_user.clone(), amount, currency, error_code, error_msg)
            },
        };

        // Create targeted event
        let event = RealtimeEvent::create(
            payload,
            vec![target_user],
            "payments".to_string(),
        )?;

        // Save event
        self.event_repository.save(&event).await
            .map_err(|e| RealtimeEventError::InvalidPayload(e))?;

        tracing::info!(
            event_id = %event.id().to_string(),
            payment_id = payment_id,
            user_id = user_id,
            event_type = ?event_type,
            "Simulated payment event"
        );

        Ok(EventBroadcastResult {
            event_id: event.id().to_string(),
            status: event.status().clone(),
            channel: event.channel().to_string(),
            target_user_count: event.target_users().len() as u32,
        })
    }

    /// Simulate a stock price update (for testing/admin purposes)
    pub async fn simulate_stock_price_update(
        &self,
        symbol: String,
        price: f64,
        change: f64,
        change_percent: f64,
        volume: u64,
    ) -> Result<EventBroadcastResult, RealtimeEventError> {
        // Create stock price update payload
        let payload = EventPayload::stock_price_update(symbol.clone(), price, change, change_percent, volume);

        // Create broadcast event (stock updates go to all users)
        let event = RealtimeEvent::create_broadcast(payload, "trading".to_string())?;

        // Save event
        self.event_repository.save(&event).await
            .map_err(|e| RealtimeEventError::InvalidPayload(e))?;

        tracing::info!(
            event_id = %event.id().to_string(),
            symbol = symbol,
            price = price,
            "Simulated stock price update"
        );

        Ok(EventBroadcastResult {
            event_id: event.id().to_string(),
            status: event.status().clone(),
            channel: event.channel().to_string(),
            target_user_count: 0, // Broadcast events don't have specific target count
        })
    }

    /// Register a new real-time connection
    pub async fn register_connection(
        &self,
        user_id: String,
        connection_type: ConnectionType,
        user_agent: Option<String>,
        ip_address: String,
    ) -> Result<ConnectionRegistrationResult, String> {
        let connection_id = ConnectionId::new();
        let realtime_user_id = RealtimeUserId::new(user_id.to_string())
            .map_err(|e| format!("Invalid user ID: {:?}", e))?;

        let connection_info = ConnectionInfo {
            user_agent,
            ip_address,
            connected_at: chrono::Utc::now(),
            last_ping: chrono::Utc::now(),
            connection_type: connection_type.clone(),
        };

        // Store connection
        self.connection_repository
            .store_connection(&connection_id, &realtime_user_id, connection_info)
            .await?;

        tracing::info!(
            connection_id = %connection_id.to_string(),
            user_id = user_id,
            connection_type = ?connection_type,
            "Registered new real-time connection"
        );

        Ok(ConnectionRegistrationResult {
            connection_id: connection_id.to_string(),
            user_id,
            connection_type,
        })
    }

    /// Remove a real-time connection
    pub async fn remove_connection(&self, connection_id: String) -> Result<(), String> {
        let conn_id = ConnectionId::from_string(connection_id.clone())
            .map_err(|e| format!("Invalid connection ID: {:?}", e))?;
        self.connection_repository.remove_connection(&conn_id).await?;

        tracing::info!(
            connection_id = connection_id,
            "Removed real-time connection"
        );

        Ok(())
    }

    /// Get connection statistics
    pub async fn get_connection_stats(&self) -> Result<ConnectionStatistics, String> {
        let total_connections = self.connection_repository.count_active_connections().await?;
        
        // Count unique users (simplified implementation)
        let unique_users = total_connections; // TODO: Implement proper unique user counting

        Ok(ConnectionStatistics {
            total_connections: total_connections as usize,
            unique_users: unique_users as usize,
            websocket_connections: 0, // TODO: Implement connection type counting
            sse_connections: 0,
        })
    }

    /// Find pending events that need to be delivered
    pub async fn find_pending_events(&self, limit: u32) -> Result<Vec<EventDeliveryInfo>, String> {
        let events = self.event_repository.find_pending_events(limit).await?;
        
        let delivery_info: Vec<EventDeliveryInfo> = events.into_iter()
            .map(|event| EventDeliveryInfo {
                event_id: event.id().to_string(),
                channel: event.channel().to_string(),
                priority: event.priority(),
                target_users: event.target_users().iter()
                    .map(|u| u.to_string())
                    .collect(),
                created_at: event.created_at(),
            })
            .collect();

        Ok(delivery_info)
    }
}

/// Result of broadcasting an event
#[derive(Debug)]
pub struct EventBroadcastResult {
    pub event_id: String,
    pub status: EventStatus,
    pub channel: String,
    pub target_user_count: u32,
}

/// Result of registering a connection
#[derive(Debug)]
pub struct ConnectionRegistrationResult {
    pub connection_id: String,
    pub user_id: String,
    pub connection_type: ConnectionType,
}

/// Connection statistics
#[derive(Debug)]
pub struct ConnectionStatistics {
    pub total_connections: usize,
    pub unique_users: usize,
    pub websocket_connections: usize,
    pub sse_connections: usize,
}

/// Information about events pending delivery
#[derive(Debug)]
pub struct EventDeliveryInfo {
    pub event_id: String,
    pub channel: String,
    pub priority: EventPriority,
    pub target_users: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Types of payment events that can be simulated
#[derive(Debug, Clone)]
pub enum PaymentEventType {
    Started,
    Completed,
    Failed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::adapters::repositories::create_test_pool;

    fn create_test_service() -> RealtimeEventsServiceIntegration {
        let db_pool = Arc::new(create_test_pool());
        let event_repository = Arc::new(RealtimeEventRepositoryAdapter::new(db_pool));
        let connection_repository = Arc::new(ConnectionRepositoryAdapter::new());
        
        RealtimeEventsServiceIntegration::new(event_repository, connection_repository)
    }

    #[tokio::test]
    async fn test_broadcast_system_notification() {
        let service = create_test_service();
        
        let result = service.broadcast_system_notification(
            "Test Notification".to_string(),
            "This is a test".to_string(),
            NotificationLevel::Info,
            None,
            "test".to_string(),
        ).await;
        
        assert!(result.is_ok());
        let broadcast_result = result.unwrap();
        assert!(!broadcast_result.event_id.is_empty());
        assert_eq!(broadcast_result.channel, "notifications");
    }

    #[tokio::test]
    async fn test_register_connection() {
        let service = create_test_service();
        
        let result = service.register_connection(
            "user123".to_string(),
            ConnectionType::WebSocket,
            Some("Mozilla/5.0".to_string()),
            "127.0.0.1".to_string(),
        ).await;
        
        assert!(result.is_ok());
        let conn_result = result.unwrap();
        assert_eq!(conn_result.user_id, "user123");
        assert_eq!(conn_result.connection_type, ConnectionType::WebSocket);
    }
}