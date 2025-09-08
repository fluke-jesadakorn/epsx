// Real-time Events User ID Value Object
// Represents a user in the context of real-time event broadcasting

use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// User identifier for real-time events
/// This is separate from domain::user_management::UserId to maintain bounded context isolation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(String);

impl UserId {
    /// Create new user ID from string
    pub fn new(id: String) -> Result<Self, UserIdError> {
        if id.is_empty() {
            return Err(UserIdError::Empty);
        }
        
        if id.len() > 100 {
            return Err(UserIdError::TooLong);
        }
        
        // Validate format - can be numeric ID or Firebase UID
        if !Self::is_valid_format(&id) {
            return Err(UserIdError::InvalidFormat);
        }
        
        Ok(Self(id))
    }
    
    /// Create from numeric user ID
    pub fn from_numeric(id: i32) -> Self {
        Self(id.to_string())
    }
    
    /// Create from Firebase UID
    pub fn from_firebase_uid(uid: String) -> Result<Self, UserIdError> {
        Self::new(uid)
    }
    
    /// Get the user ID as string
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Get the user ID as string (owned)
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
    
    /// Check if this is a numeric user ID
    pub fn is_numeric(&self) -> bool {
        self.0.parse::<i32>().is_ok()
    }
    
    /// Check if this is a Firebase UID format
    pub fn is_firebase_uid(&self) -> bool {
        self.0.len() >= 20 && !self.is_numeric()
    }
    
    /// Validate user ID format
    fn is_valid_format(id: &str) -> bool {
        // Allow numeric IDs (legacy)
        if id.parse::<i32>().is_ok() {
            return true;
        }
        
        // Allow Firebase UID format (alphanumeric with some special chars)
        if id.len() >= 10 && id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return true;
        }
        
        false
    }
}

impl Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<crate::domain::user_management::value_objects::UserId> for UserId {
    fn from(user_id: crate::domain::user_management::value_objects::UserId) -> Self {
        Self(user_id.to_string())
    }
}

/// Errors that can occur when creating a user ID
#[derive(Debug, thiserror::Error)]
pub enum UserIdError {
    #[error("User ID cannot be empty")]
    Empty,
    
    #[error("User ID is too long (max 100 characters)")]
    TooLong,
    
    #[error("Invalid user ID format")]
    InvalidFormat,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_numeric_user_id() {
        let user_id = UserId::from_numeric(123);
        assert_eq!(user_id.as_str(), "123");
        assert!(user_id.is_numeric());
        assert!(!user_id.is_firebase_uid());
    }
    
    #[test]
    fn test_firebase_uid() {
        let uid = "abcd1234efgh5678ijkl9012mnop3456qrst7890";
        let user_id = UserId::from_firebase_uid(uid.to_string()).unwrap();
        assert_eq!(user_id.as_str(), uid);
        assert!(!user_id.is_numeric());
        assert!(user_id.is_firebase_uid());
    }
    
    #[test]
    fn test_invalid_user_id() {
        let result = UserId::new("".to_string());
        assert!(matches!(result, Err(UserIdError::Empty)));
        
        let long_id = "a".repeat(101);
        let result = UserId::new(long_id);
        assert!(matches!(result, Err(UserIdError::TooLong)));
    }
}