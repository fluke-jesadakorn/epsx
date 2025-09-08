use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;
use crate::domain::shared_kernel::value_object::{ValueObject, ValueObjectError};

/// User identifier value object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(String);

impl UserId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn from_string(id: String) -> Result<Self, ValueObjectError> {
        if id.is_empty() {
            return Err(ValueObjectError::Required("User ID cannot be empty".to_string()));
        }
        let user_id = Self(id);
        user_id.validate()?;
        Ok(user_id)
    }

    pub fn from_string_unchecked(id: String) -> Self {
        Self(id)
    }

    pub fn from_str(id: &str) -> Result<Self, ValueObjectError> {
        Self::from_string(id.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
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