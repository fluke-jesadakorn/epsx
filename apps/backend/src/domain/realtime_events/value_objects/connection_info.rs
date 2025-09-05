// Connection Info Value Objects

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Information about a real-time connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub user_agent: Option<String>,
    pub ip_address: String,
    pub connected_at: DateTime<Utc>,
    pub last_ping: DateTime<Utc>,
    pub connection_type: ConnectionType,
}

/// Type of real-time connection
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionType {
    WebSocket,
    ServerSentEvents,
}

/// Connection status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Error,
}