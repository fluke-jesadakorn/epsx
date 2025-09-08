// Payment Method ID Value Object
// Unique identifier for payment methods in the payment domain

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::fmt::{self, Display};

use crate::domain::shared_kernel::{ValueObject, value_object::ValueObjectError};

/// Unique identifier for payment methods
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PaymentMethodId {
    id: Uuid,
}

impl PaymentMethodId {
    /// Create a new payment method ID
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
        }
    }
    
    /// Create from existing UUID
    pub fn from_uuid(id: Uuid) -> Self {
        Self { id }
    }
    
    /// Create from string representation
    pub fn from_string(id: String) -> Result<Self, PaymentMethodIdError> {
        let uuid = Uuid::parse_str(&id)
            .map_err(|_| PaymentMethodIdError::InvalidFormat(id))?;
        Ok(Self { id: uuid })
    }
    
    /// Get the underlying UUID
    pub fn as_uuid(&self) -> Uuid {
        self.id
    }
    
    /// Get string representation
    pub fn as_string(&self) -> String {
        self.id.to_string()
    }
}

impl ValueObject for PaymentMethodId {
    type Error = PaymentMethodIdError;
    
    fn validate(&self) -> Result<(), Self::Error> {
        // UUID is always valid once created
        Ok(())
    }
}

impl Display for PaymentMethodId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl Default for PaymentMethodId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for PaymentMethodId {
    fn from(id: Uuid) -> Self {
        Self::from_uuid(id)
    }
}

impl From<PaymentMethodId> for Uuid {
    fn from(payment_method_id: PaymentMethodId) -> Self {
        payment_method_id.id
    }
}

impl From<PaymentMethodId> for String {
    fn from(payment_method_id: PaymentMethodId) -> Self {
        payment_method_id.as_string()
    }
}

impl std::str::FromStr for PaymentMethodId {
    type Err = PaymentMethodIdError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_string(s.to_string())
    }
}

/// Errors that can occur when working with payment method IDs
#[derive(Debug, thiserror::Error)]
pub enum PaymentMethodIdError {
    #[error("Invalid payment method ID format: {0}")]
    InvalidFormat(String),
    
    #[error("Payment method ID validation failed: {0}")]
    ValidationFailed(String),
}

impl From<PaymentMethodIdError> for ValueObjectError {
    fn from(error: PaymentMethodIdError) -> Self {
        ValueObjectError::ValidationFailed(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_new_payment_method_id() {
        let id1 = PaymentMethodId::new();
        let id2 = PaymentMethodId::new();
        
        // Should be unique
        assert_ne!(id1, id2);
        assert!(id1.is_valid());
        assert!(id2.is_valid());
    }
    
    #[test]
    fn test_payment_method_id_from_string() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let id = PaymentMethodId::from_string(uuid_str.to_string()).unwrap();
        
        assert_eq!(id.as_string(), uuid_str);
    }
    
    #[test]
    fn test_invalid_payment_method_id_format() {
        let result = PaymentMethodId::from_string("invalid-uuid".to_string());
        assert!(result.is_err());
    }
    
    #[test]
    fn test_payment_method_id_conversions() {
        let uuid = Uuid::new_v4();
        let id = PaymentMethodId::from(uuid);
        
        assert_eq!(Uuid::from(id.clone()), uuid);
        assert_eq!(id.as_uuid(), uuid);
    }
    
    #[test]
    fn test_payment_method_id_display() {
        let id = PaymentMethodId::new();
        let display_str = format!("{}", id);
        let uuid_str = id.as_string();
        
        assert_eq!(display_str, uuid_str);
    }
}