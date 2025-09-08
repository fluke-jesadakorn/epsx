// Connection ID Value Object

use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use uuid::Uuid;

/// Unique identifier for WebSocket/SSE connections
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConnectionId(String);

impl ConnectionId {
    /// Create new connection ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
    
    /// Create connection ID from string
    pub fn from_string(id: String) -> Result<Self, ConnectionIdError> {
        if id.is_empty() {
            return Err(ConnectionIdError::Empty);
        }
        
        // Validate UUID format
        Uuid::parse_str(&id)
            .map_err(|_| ConnectionIdError::InvalidFormat)?;
        
        Ok(Self(id))
    }
    
    /// Get the ID as string
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Get the ID as string (owned)
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl Default for ConnectionId {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for ConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Errors that can occur when creating a connection ID
#[derive(Debug, thiserror::Error)]
pub enum ConnectionIdError {
    #[error("Connection ID cannot be empty")]
    Empty,
    
    #[error("Invalid connection ID format (must be UUID)")]
    InvalidFormat,
}