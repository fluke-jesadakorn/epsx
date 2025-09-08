use std::fmt;
use serde::{Deserialize, Serialize};

use crate::domain::shared_kernel::ValueObject;
use crate::domain::shared_kernel::value_object::ValueObjectError;

/// Firebase User ID value object
/// Represents a Firebase authentication user identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FirebaseUid(String);

impl FirebaseUid {
    /// Create a new Firebase UID
    pub fn new(uid: impl Into<String>) -> Result<Self, ValueObjectError> {
        let uid = uid.into();
        let instance = Self(uid);
        instance.validate()?;
        Ok(instance)
    }
    
    /// Get the Firebase UID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ValueObject for FirebaseUid {
    type Error = ValueObjectError;
    
    fn validate(&self) -> Result<(), Self::Error> {
        // Firebase UIDs are typically 28 characters long and alphanumeric
        if self.0.is_empty() {
            return Err(ValueObjectError::Required("Firebase UID cannot be empty".to_string()));
        }
        
        if self.0.len() < 10 || self.0.len() > 128 {
            return Err(ValueObjectError::OutOfRange("Firebase UID length must be between 10 and 128 characters".to_string()));
        }
        
        // Check that it contains only valid characters
        if !self.0.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(ValueObjectError::InvalidFormat("Firebase UID contains invalid characters".to_string()));
        }
        
        Ok(())
    }
}

impl fmt::Display for FirebaseUid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<FirebaseUid> for String {
    fn from(uid: FirebaseUid) -> Self {
        uid.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn valid_firebase_uid_should_pass() {
        let uid = FirebaseUid::new("abcd1234efgh5678ijkl9012");
        assert!(uid.is_ok());
    }
    
    #[test]
    fn empty_firebase_uid_should_fail() {
        let uid = FirebaseUid::new("");
        assert!(uid.is_err());
    }
    
    #[test]
    fn too_short_firebase_uid_should_fail() {
        let uid = FirebaseUid::new("abc123");
        assert!(uid.is_err());
    }
    
    #[test]
    fn invalid_characters_should_fail() {
        let uid = FirebaseUid::new("invalid@uid");
        assert!(uid.is_err());
    }
}