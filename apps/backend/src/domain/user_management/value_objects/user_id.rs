use std::fmt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::shared_kernel::{ValueObject, Identity, new_id};
use crate::domain::shared_kernel::value_object::ValueObjectError;

/// User identifier value object
/// Represents a unique identifier for a user in the system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(Uuid);

impl UserId {
    /// Create a new user ID with the given UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
    
    /// Parse a user ID from a string
    pub fn from_string(s: &str) -> Result<Self, ValueObjectError> {
        let uuid = Uuid::parse_str(s)
            .map_err(|_| ValueObjectError::InvalidFormat("Invalid UUID format".to_string()))?;
        Ok(Self(uuid))
    }
    
    /// Get the inner UUID
    pub fn inner(&self) -> Uuid {
        self.0
    }
}

impl ValueObject for UserId {
    type Error = ValueObjectError;
    
    fn validate(&self) -> Result<(), Self::Error> {
        // UUID is always valid if it was constructed properly
        Ok(())
    }
}

impl Identity for UserId {
    fn new() -> Self {
        Self(new_id())
    }
    
    fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
    
    fn to_uuid(&self) -> Uuid {
        self.0
    }
    
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for UserId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<UserId> for Uuid {
    fn from(user_id: UserId) -> Self {
        user_id.0
    }
}

impl From<UserId> for String {
    fn from(user_id: UserId) -> Self {
        user_id.0.to_string()
    }
}