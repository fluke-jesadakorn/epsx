use serde::{Deserialize, Serialize};
use std::fmt;
use crate::domain::shared_kernel::value_object::{ValueObject, ValueObjectError};
use crate::core::errors::AppError;

/// Email address value object with validation
///
/// **IMPORTANT**: This is used ONLY for notification delivery (email topics, sending emails).
/// It is NOT used for user identity or authentication in the Web3-first system.
/// User identity is based on wallet_address only.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Email(String);

impl Email {
    pub fn new(email: String) -> Result<Self, AppError> {
        if Self::is_valid(&email) {
            Ok(Self(email.to_lowercase()))
        } else {
            Err(AppError::validation_error(format!("Invalid email format: {}", email)))
        }
    }

    pub fn from_trusted(email: String) -> Self {
        Self(email.to_lowercase())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn is_valid(email: &str) -> bool {
        // Basic email validation - could be enhanced with regex
        email.contains('@') && email.contains('.') && email.len() > 5
    }
}

impl ValueObject for Email {
    type Error = ValueObjectError;
    
    fn validate(&self) -> Result<(), Self::Error> {
        if !Self::is_valid(&self.0) {
            return Err(ValueObjectError::InvalidFormat(format!("Invalid email format: {}", self.0)));
        }
        Ok(())
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for Email {
    type Error = AppError;

    fn try_from(email: String) -> Result<Self, Self::Error> {
        Self::new(email)
    }
}

impl TryFrom<&str> for Email {
    type Error = AppError;

    fn try_from(email: &str) -> Result<Self, Self::Error> {
        Self::new(email.to_string())
    }
}