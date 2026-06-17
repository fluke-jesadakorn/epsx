// kernel extraction wave9 — moved from apps/backend/src/domain/shared_kernel/value_objects/user_id.rs
// Import-path adjustment: the `ValueObject` / `ValueObjectError` / `Identity`
// traits now live in sibling modules of this crate.
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;
use crate::value_object::{ValueObject, ValueObjectError};
use crate::traits::aggregate_root::Identity;

/// User identifier value object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(String);

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl UserId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn from_string(id: String) -> Result<Self, ValueObjectError> {
        if id.is_empty() {
            return Err(ValueObjectError::Required("User ID cannot be empty".to_string()));
        }
        let wallet_address = Self(id);
        wallet_address.validate()?;
        Ok(wallet_address)
    }

    pub fn from_string_unchecked(id: String) -> Self {
        Self(id)
    }

    pub fn parse(id: &str) -> Result<Self, ValueObjectError> {
        Self::from_string(id.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ValueObject for UserId {
    type Error = ValueObjectError;
    
    fn validate(&self) -> Result<(), Self::Error> {
        if self.0.is_empty() {
            return Err(ValueObjectError::Required("User ID cannot be empty".to_string()));
        }
        
        // Basic UUID format validation - could be more strict
        if self.0.len() < 8 {
            return Err(ValueObjectError::InvalidFormat("User ID must be at least 8 characters".to_string()));
        }
        
        Ok(())
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for UserId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl From<i32> for UserId {
    fn from(id: i32) -> Self {
        Self(id.to_string())
    }
}

impl From<&str> for UserId {
    fn from(id: &str) -> Self {
        Self(id.to_string())
    }
}

impl Identity for UserId {
    fn new() -> Self {
        Self::new()
    }

    fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid.to_string())
    }

    fn to_uuid(&self) -> Uuid {
        // Try to parse as UUID, fall back to generating from string hash if invalid
        Uuid::parse_str(&self.0).unwrap_or_else(|_| {
            // Generate a deterministic UUID from the string
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            self.0.hash(&mut hasher);
            let hash = hasher.finish();
            // Convert hash to UUID bytes
            let bytes = hash.to_le_bytes();
            let mut uuid_bytes = [0u8; 16];
            uuid_bytes[..8].copy_from_slice(&bytes);
            uuid_bytes[8..16].copy_from_slice(&bytes);
            Uuid::from_bytes(uuid_bytes)
        })
    }

    fn to_string(&self) -> String {
        self.0.clone()
    }
}
