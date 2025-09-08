use async_trait::async_trait;
use chrono::{DateTime, Utc};// Real-time Events Repository Ports
// Repository interfaces for Real-time Events bounded context


use super::{EventId, RealtimeEvent, RealtimeUserId, ConnectionId};
use super::aggregates::EventStatus;
use super::value_objects::{ConnectionInfo, ConnectionType};

/// Repository port for managing real-time events
#[async_trait]
pub trait EventRepositoryPort: Send + Sync {
    async fn save(&self, event: &RealtimeEvent) -> Result<(), String>;
    async fn find_by_id(&self, event_id: &EventId) -> Result<Option<RealtimeEvent>, String>;
    async fn find_pending_events(&self, limit: u32) -> Result<Vec<RealtimeEvent>, String>;
    async fn find_by_channel_and_status(&self, channel: &str, status: EventStatus, limit: u32) -> Result<Vec<RealtimeEvent>, String>;
    async fn find_expired_events(&self, before: DateTime<Utc>) -> Result<Vec<RealtimeEvent>, String>;
    async fn count_by_status(&self, status: EventStatus) -> Result<u64, String>;
    async fn delete_expired(&self, before: DateTime<Utc>) -> Result<u64, String>;
    async fn find_events_for_retry(&self, max_attempts: u32) -> Result<Vec<RealtimeEvent>, String>;
}

/// Repository port for managing real-time connections
#[async_trait]
pub trait ConnectionRepositoryPort: Send + Sync {
    async fn store_connection(&self, connection_id: &ConnectionId, user_id: &RealtimeUserId, connection_info: ConnectionInfo) -> Result<(), String>;
    async fn remove_connection(&self, connection_id: &ConnectionId) -> Result<(), String>;
    async fn find_connection(&self, connection_id: &ConnectionId) -> Result<Option<ConnectionInfo>, String>;
    async fn find_user_connections(&self, user_id: &RealtimeUserId) -> Result<Vec<ConnectionId>, String>;
    async fn find_connections_by_type(&self, connection_type: ConnectionType) -> Result<Vec<ConnectionId>, String>;
    async fn update_heartbeat(&self, connection_id: &ConnectionId) -> Result<(), String>;
    async fn find_stale_connections(&self, stale_after: DateTime<Utc>) -> Result<Vec<ConnectionId>, String>;
    async fn count_active_connections(&self) -> Result<u64, String>;
    async fn count_user_connections(&self, user_id: &RealtimeUserId) -> Result<u32, String>;
}

/// Repository port for managing event subscriptions
#[async_trait]
pub trait SubscriptionRepositoryPort: Send + Sync {
    // TODO: Define subscription management methods
}

/// Repository port for managing event history and analytics
#[async_trait]
pub trait EventHistoryRepositoryPort: Send + Sync {
    // TODO: Define event history and analytics methods
}