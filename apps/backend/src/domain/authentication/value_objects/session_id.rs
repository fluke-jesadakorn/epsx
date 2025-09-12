// Session ID Value Object
// Unique identifier for authentication sessions

use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Session identifier with validation and generation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(String);

impl SessionId {
    /// Generate a new cryptographically secure session ID
    pub fn generate() -> Self {
        let uuid = Uuid::new_v4();
        Self(format!("sess_{}", uuid.simple()))
    }
    
    /// Create from existing string with validation
    pub fn from_string(id: String) -> Result<Self, SessionIdError> {
        Self::validate(&id)?;
        Ok(Self(id))
    }
    
    /// Get string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Get inner string value
    pub fn into_inner(self) -> String {
        self.0
    }
    
    // Validation
    fn validate(id: &str) -> Result<(), SessionIdError> {
        if id.is_empty() {
            return Err(SessionIdError::Empty);
        }
        
        if id.len() < 8 {
            return Err(SessionIdError::TooShort);
        }
        
        if id.len() > 128 {
            return Err(SessionIdError::TooLong);
        }
        
        // Must start with sess_ prefix for our sessions
        if !id.starts_with("sess_") {
            return Err(SessionIdError::InvalidFormat);
        }
        
        // Must contain only alphanumeric characters and underscores
        if !id.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(SessionIdError::InvalidCharacters);
        }
        
        Ok(())
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for SessionId {
    type Err = SessionIdError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_string(s.to_string())
    }
}

impl TryFrom<String> for SessionId {
    type Error = SessionIdError;
    
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_string(value)
    }
}

impl From<SessionId> for String {
    fn from(session_id: SessionId) -> Self {
        session_id.0
    }
}

/// Session ID validation errors
#[derive(Debug, thiserror::Error)]
pub enum SessionIdError {
    #[error("Session ID cannot be empty")]
    Empty,
    
    #[error("Session ID is too short (minimum 8 characters)")]
    TooShort,
    
    #[error("Session ID is too long (maximum 128 characters)")]
    TooLong,
    
    #[error("Session ID must start with 'sess_' prefix")]
    InvalidFormat,
    
    #[error("Session ID contains invalid characters")]
    InvalidCharacters,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn generates_validsession_id() {
        let session_id = SessionId::generate();
        assert!(session_id.as_str().starts_with("sess_"));
        assert!(session_id.as_str().len() > 8);
    }
    
    #[test]
    fn validatessession_id_format() {
        // Valid session ID
        assert!(SessionId::from_string("sess_abc123def456".to_string()).is_ok());
        
        // Invalid cases
        assert!(SessionId::from_string("".to_string()).is_err());
        assert!(SessionId::from_string("abc".to_string()).is_err());
        assert!(SessionId::from_string("invalid_prefix_123".to_string()).is_err());
        assert!(SessionId::from_string("sess_with@invalid#chars".to_string()).is_err());
    }
    
    #[test]
    fn session_ids_are_unique() {
        let id1 = SessionId::generate();
        let id2 = SessionId::generate();
        assert_ne!(id1, id2);
    }
}