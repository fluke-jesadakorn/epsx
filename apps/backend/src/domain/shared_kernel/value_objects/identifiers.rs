use serde::{Deserialize, Serialize};
use crate::domain::shared_kernel::value_object::{ValueObject, ValueObjectError};

// Re-export commonly used identifiers
pub use super::user_id::UserId;

/// Stock symbol identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StockSymbol(String);

impl StockSymbol {
    pub fn new(symbol: String) -> Self {
        Self(symbol.to_uppercase())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ValueObject for StockSymbol {
    type Error = ValueObjectError;
    
    fn validate(&self) -> Result<(), Self::Error> {
        if self.0.is_empty() {
            return Err(ValueObjectError::Required("Stock symbol cannot be empty".to_string()));
        }
        
        if self.0.len() > 10 {
            return Err(ValueObjectError::InvalidFormat("Stock symbol too long (max 10 chars)".to_string()));
        }
        
        Ok(())
    }
}

impl From<String> for StockSymbol {
    fn from(symbol: String) -> Self {
        Self::new(symbol)
    }
}

impl From<&str> for StockSymbol {
    fn from(symbol: &str) -> Self {
        Self::new(symbol.to_string())
    }
}

/// Notification identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NotificationId(String);

impl Default for NotificationId {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    pub fn from_string(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ValueObject for NotificationId {
    type Error = ValueObjectError;
    
    fn validate(&self) -> Result<(), Self::Error> {
        if self.0.is_empty() {
            return Err(ValueObjectError::Required("Notification ID cannot be empty".to_string()));
        }
        
        // Basic UUID format validation
        if self.0.len() < 8 {
            return Err(ValueObjectError::InvalidFormat("Notification ID must be at least 8 characters".to_string()));
        }
        
        Ok(())
    }
}

/// Connection identifier for real-time connections
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConnectionId(String);

impl Default for ConnectionId {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    pub fn from_string(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ValueObject for ConnectionId {
    type Error = ValueObjectError;
    
    fn validate(&self) -> Result<(), Self::Error> {
        if self.0.is_empty() {
            return Err(ValueObjectError::Required("Connection ID cannot be empty".to_string()));
        }
        
        // Basic UUID format validation
        if self.0.len() < 8 {
            return Err(ValueObjectError::InvalidFormat("Connection ID must be at least 8 characters".to_string()));
        }
        
        Ok(())
    }
}