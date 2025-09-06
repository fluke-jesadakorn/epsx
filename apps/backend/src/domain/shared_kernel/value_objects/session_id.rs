use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;
use crate::domain::shared_kernel::value_object::{ValueObject, ValueObjectError};

/// Session identifier value object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(String);

impl SessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
    
    pub fn generate() -> Self {
        Self::new()
    }

    pub fn from_string(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl ValueObject for SessionId {
    type Error = ValueObjectError;
    
    fn validate(&self) -> Result<(), Self::Error> {
        if self.0.is_empty() {
            return Err(ValueObjectError::Required("Session ID cannot be empty".to_string()));
        }
        
        // Allow both UUID format and prefixed formats (e.g., "auth_code:xyz", "refresh:xyz")
        if self.0.len() < 8 {
            return Err(ValueObjectError::InvalidFormat("Session ID must be at least 8 characters".to_string()));
        }
        
        // If it's not a prefixed format, validate as UUID
        if !self.0.contains(':') {
            if let Err(_) = uuid::Uuid::parse_str(&self.0) {
                return Err(ValueObjectError::InvalidFormat("Session ID must be a valid UUID or prefixed format".to_string()));
            }
        }
        
        Ok(())
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for SessionId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl From<&str> for SessionId {
    fn from(id: &str) -> Self {
        Self(id.to_string())
    }
}

/// Alias for legacy compatibility
pub type SessId = SessionId;