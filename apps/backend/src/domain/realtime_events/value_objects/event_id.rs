// Event ID Value Object

use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use uuid::Uuid;

/// Unique identifier for real-time events
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(String);

impl EventId {
    /// Create new event ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
    
    /// Create event ID from string
    pub fn from_string(id: String) -> Result<Self, EventIdError> {
        if id.is_empty() {
            return Err(EventIdError::Empty);
        }
        
        // Validate UUID format
        Uuid::parse_str(&id)
            .map_err(|_| EventIdError::InvalidFormat)?;
        
        Ok(Self(id))
    }
    
    /// Get the ID as string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Errors that can occur when creating an event ID
#[derive(Debug, thiserror::Error)]
pub enum EventIdError {
    #[error("Event ID cannot be empty")]
    Empty,
    
    #[error("Invalid event ID format (must be UUID)")]
    InvalidFormat,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_event_id() {
        let id1 = EventId::new();
        let id2 = EventId::new();
        
        // Should be different
        assert_ne!(id1, id2);
        
        // Should be valid UUIDs
        assert!(Uuid::parse_str(id1.as_str()).is_ok());
        assert!(Uuid::parse_str(id2.as_str()).is_ok());
    }
    
    #[test]
    fn test_from_string() {
        let uuid_str = Uuid::new_v4().to_string();
        let event_id = EventId::from_string(uuid_str.clone()).unwrap();
        assert_eq!(event_id.as_str(), uuid_str);
        
        // Test invalid format
        let result = EventId::from_string("not-a-uuid".to_string());
        assert!(matches!(result, Err(EventIdError::InvalidFormat)));
    }
}