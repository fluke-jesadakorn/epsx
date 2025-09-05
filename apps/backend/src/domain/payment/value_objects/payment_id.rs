use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use uuid::Uuid;

/// Payment ID Value Object
/// Unique identifier for payments with validation and formatting
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PaymentId {
    value: Uuid,
}

impl PaymentId {
    /// Generate new payment ID
    pub fn generate() -> Self {
        Self {
            value: Uuid::new_v4(),
        }
    }

    /// Create from UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self { value: uuid }
    }

    /// Create from string with validation
    pub fn from_string(s: &str) -> Result<Self, String> {
        let uuid = Uuid::parse_str(s)
            .map_err(|_| format!("Invalid payment ID format: {}", s))?;
        Ok(Self { value: uuid })
    }

    /// Get the underlying UUID
    pub fn value(&self) -> Uuid {
        self.value
    }

    /// Get as string
    pub fn as_string(&self) -> String {
        self.value.to_string()
    }

    /// Check if this is a valid payment ID format
    pub fn is_valid_format(s: &str) -> bool {
        Uuid::parse_str(s).is_ok()
    }

    /// Generate short ID for display (first 8 characters)
    pub fn short_id(&self) -> String {
        self.value.to_string()[..8].to_string()
    }

    /// Check if payment ID is nil/empty
    pub fn is_nil(&self) -> bool {
        self.value.is_nil()
    }
}

impl Display for PaymentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl From<Uuid> for PaymentId {
    fn from(uuid: Uuid) -> Self {
        Self::from_uuid(uuid)
    }
}

impl From<PaymentId> for Uuid {
    fn from(payment_id: PaymentId) -> Self {
        payment_id.value
    }
}

impl std::str::FromStr for PaymentId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_string(s)
    }
}

/// Payment Reference Number - Human readable payment identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PaymentReference {
    value: String,
}

impl PaymentReference {
    /// Generate new payment reference (PAY-YYYYMMDD-XXXXXX format)
    pub fn generate() -> Self {
        let now = chrono::Utc::now();
        let date_part = now.format("%Y%m%d").to_string();
        let random_part = format!("{:06}", rand::random::<u32>() % 1_000_000);
        
        Self {
            value: format!("PAY-{}-{}", date_part, random_part),
        }
    }

    /// Create from string with validation
    pub fn from_string(s: &str) -> Result<Self, String> {
        if !Self::is_valid_format(s) {
            return Err(format!("Invalid payment reference format: {}", s));
        }
        
        Ok(Self {
            value: s.to_uppercase(),
        })
    }

    /// Get the value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Check if format is valid (PAY-YYYYMMDD-XXXXXX)
    pub fn is_valid_format(s: &str) -> bool {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 3 {
            return false;
        }
        
        parts[0] == "PAY" 
            && parts[1].len() == 8 
            && parts[1].chars().all(|c| c.is_ascii_digit())
            && parts[2].len() == 6
            && parts[2].chars().all(|c| c.is_ascii_digit())
    }

    /// Get the date part of the reference
    pub fn date_part(&self) -> Option<chrono::NaiveDate> {
        let parts: Vec<&str> = self.value.split('-').collect();
        if parts.len() >= 2 {
            chrono::NaiveDate::parse_from_str(parts[1], "%Y%m%d").ok()
        } else {
            None
        }
    }

    /// Get the sequence number
    pub fn sequence_number(&self) -> Option<u32> {
        let parts: Vec<&str> = self.value.split('-').collect();
        if parts.len() >= 3 {
            parts[2].parse::<u32>().ok()
        } else {
            None
        }
    }
}

impl Display for PaymentReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl std::str::FromStr for PaymentReference {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_string(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_id_generation() {
        let id1 = PaymentId::generate();
        let id2 = PaymentId::generate();
        
        assert_ne!(id1, id2);
        assert!(!id1.is_nil());
        assert!(!id2.is_nil());
    }

    #[test]
    fn test_payment_id_from_string() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let payment_id = PaymentId::from_string(uuid_str).unwrap();
        
        assert_eq!(payment_id.as_string(), uuid_str);
        assert!(PaymentId::is_valid_format(uuid_str));
        assert!(!PaymentId::is_valid_format("invalid-uuid"));
    }

    #[test]
    fn test_payment_id_short_id() {
        let payment_id = PaymentId::generate();
        let short_id = payment_id.short_id();
        
        assert_eq!(short_id.len(), 8);
        assert!(payment_id.as_string().starts_with(&short_id));
    }

    #[test]
    fn test_payment_reference_generation() {
        let ref1 = PaymentReference::generate();
        let ref2 = PaymentReference::generate();
        
        assert_ne!(ref1, ref2);
        assert!(ref1.value().starts_with("PAY-"));
        assert!(PaymentReference::is_valid_format(ref1.value()));
    }

    #[test]
    fn test_payment_reference_validation() {
        let valid_ref = "PAY-20241201-123456";
        let invalid_refs = vec![
            "INVALID-20241201-123456",
            "PAY-2024120-123456", // Wrong date length
            "PAY-20241201-12345", // Wrong sequence length
            "PAY-20241201",       // Missing sequence
            "20241201-123456",    // Missing prefix
        ];

        assert!(PaymentReference::is_valid_format(valid_ref));
        
        for invalid in invalid_refs {
            assert!(!PaymentReference::is_valid_format(invalid), "Should be invalid: {}", invalid);
        }
    }

    #[test]
    fn test_payment_reference_parsing() {
        let reference = PaymentReference::from_string("PAY-20241201-123456").unwrap();
        
        let date = reference.date_part().unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 12);
        assert_eq!(date.day(), 1);
        
        let sequence = reference.sequence_number().unwrap();
        assert_eq!(sequence, 123456);
    }
}