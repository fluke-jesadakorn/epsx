//! Value Object base trait and validation error.
//!
//! Source: apps/backend/src/domain/shared_kernel/value_object.rs

use std::fmt::Debug;
use serde::{Deserialize, Serialize};

/// Marker trait for value objects.
/// Value objects are immutable and are defined by their attributes.
pub trait ValueObject:
    Clone + Debug + PartialEq + Eq + Send + Sync + Serialize + for<'de> Deserialize<'de>
{
    type Error: std::error::Error + Send + Sync;

    /// Validate the value object
    fn validate(&self) -> Result<(), Self::Error>;

    /// Check if this value object is valid
    fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }
}

/// Base validation error for value objects.
#[derive(Debug, thiserror::Error)]
pub enum ValueObjectError {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Value out of range: {0}")]
    OutOfRange(String),

    #[error("Required field missing: {0}")]
    Required(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct AlwaysValid;

    impl ValueObject for AlwaysValid {
        type Error = ValueObjectError;
        fn validate(&self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[test]
    fn value_object_is_valid() {
        assert!(AlwaysValid.is_valid());
    }
}
