use std::fmt;
use serde::{Deserialize, Serialize};
use regex::Regex;

use crate::domain::shared_kernel::ValueObject;
use crate::domain::shared_kernel::value_object::ValueObjectError;

/// Email address value object
/// Represents a validated email address
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Email(String);

impl Email {
    /// Create a new email address
    pub fn new(email: impl Into<String>) -> Result<Self, ValueObjectError> {
        let email = email.into();
        let instance = Self(email);
        instance.validate()?;
        Ok(instance)
    }
    
    /// Get the email address as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Get the domain part of the email
    pub fn domain(&self) -> Option<&str> {
        self.0.split('@').nth(1)
    }
    
    /// Get the local part of the email (before @)
    pub fn local_part(&self) -> Option<&str> {
        self.0.split('@').next()
    }
    
    /// Check if this is a disposable email address
    pub fn is_disposable(&self) -> bool {
        // Common disposable email domains
        let disposable_domains = [
            "tempmail.org",
            "10minutemail.com", 
            "guerrillamail.com",
            "mailinator.com",
            "yopmail.com"
        ];
        
        if let Some(domain) = self.domain() {
            disposable_domains.contains(&domain.to_lowercase().as_str())
        } else {
            false
        }
    }
}

impl ValueObject for Email {
    type Error = ValueObjectError;
    
    fn validate(&self) -> Result<(), Self::Error> {
        // Basic email validation using regex
        let email_regex = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$")
            .map_err(|_| ValueObjectError::ValidationFailed("Regex compilation failed".to_string()))?;
            
        if !email_regex.is_match(&self.0) {
            return Err(ValueObjectError::InvalidFormat("Invalid email format".to_string()));
        }
        
        // Check length constraints
        if self.0.len() > 254 {
            return Err(ValueObjectError::OutOfRange("Email too long (max 254 characters)".to_string()));
        }
        
        if self.0.is_empty() {
            return Err(ValueObjectError::Required("Email cannot be empty".to_string()));
        }
        
        Ok(())
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Email> for String {
    fn from(email: Email) -> Self {
        email.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn valid_email_should_pass() {
        let email = Email::new("test@example.com");
        assert!(email.is_ok());
    }
    
    #[test]
    fn invalid_email_should_fail() {
        let email = Email::new("invalid-email");
        assert!(email.is_err());
    }
    
    #[test]
    fn empty_email_should_fail() {
        let email = Email::new("");
        assert!(email.is_err());
    }
    
    #[test]
    fn email_parts_extraction() {
        let email = Email::new("user@domain.com").unwrap();
        assert_eq!(email.local_part(), Some("user"));
        assert_eq!(email.domain(), Some("domain.com"));
    }
}