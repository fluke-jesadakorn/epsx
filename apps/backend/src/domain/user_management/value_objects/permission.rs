use std::fmt;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::domain::shared_kernel::ValueObject;
use crate::domain::shared_kernel::value_object::ValueObjectError;

/// Permission value object
/// Represents a structured permission in the format "platform:resource:action"
/// with optional temporal constraints
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    /// The full permission string (e.g., "admin:users:manage")
    permission: String,
    /// Optional expiration timestamp for time-limited permissions
    expires_at: Option<DateTime<Utc>>,
    /// Platform scope (e.g., "admin", "epsx", "epsx-pay")
    platform: String,
    /// Resource type (e.g., "users", "analytics", "payments")
    resource: String,
    /// Action allowed (e.g., "view", "manage", "create", "delete")
    action: String,
}

impl Permission {
    /// Create a new permission from a permission string
    pub fn new(permission: impl Into<String>) -> Result<Self, ValueObjectError> {
        let permission = permission.into();
        let parts = Self::parse_permission(&permission)?;
        
        let instance = Self {
            permission: permission.clone(),
            expires_at: None,
            platform: parts.0,
            resource: parts.1,
            action: parts.2,
        };
        
        instance.validate()?;
        Ok(instance)
    }
    
    /// Create a new permission with expiration
    pub fn new_with_expiration(
        permission: impl Into<String>,
        expires_at: DateTime<Utc>
    ) -> Result<Self, ValueObjectError> {
        let mut perm = Self::new(permission)?;
        perm.expires_at = Some(expires_at);
        Ok(perm)
    }
    
    /// Create a permission from embedded timestamp format
    /// Format: "platform:resource:action:unix_timestamp"
    pub fn from_embedded_timestamp(permission: impl Into<String>) -> Result<Self, ValueObjectError> {
        let permission = permission.into();
        let parts: Vec<&str> = permission.split(':').collect();
        
        if parts.len() == 4 {
            // Has timestamp
            let timestamp = parts[3].parse::<i64>()
                .map_err(|_| ValueObjectError::InvalidFormat("Invalid timestamp format".to_string()))?;
            
            let expires_at = DateTime::from_timestamp(timestamp, 0)
                .ok_or_else(|| ValueObjectError::InvalidFormat("Invalid timestamp value".to_string()))?;
            
            let base_permission = format!("{}:{}:{}", parts[0], parts[1], parts[2]);
            let mut perm = Self::new(base_permission)?;
            perm.expires_at = Some(expires_at);
            Ok(perm)
        } else {
            // No timestamp, regular permission
            Self::new(permission)
        }
    }
    
    /// Get the full permission string
    pub fn as_str(&self) -> &str {
        &self.permission
    }
    
    /// Get the platform scope
    pub fn platform(&self) -> &str {
        &self.platform
    }
    
    /// Get the resource
    pub fn resource(&self) -> &str {
        &self.resource
    }
    
    /// Get the action
    pub fn action(&self) -> &str {
        &self.action
    }
    
    /// Get the expiration timestamp
    pub fn expires_at(&self) -> Option<DateTime<Utc>> {
        self.expires_at
    }
    
    /// Check if this permission has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }
    
    /// Check if this permission is active (not expired)
    pub fn is_active(&self) -> bool {
        !self.is_expired()
    }
    
    /// Convert to embedded timestamp format if expires_at is set
    pub fn to_embedded_format(&self) -> String {
        if let Some(expires_at) = self.expires_at {
            format!("{}:{}", self.permission, expires_at.timestamp())
        } else {
            self.permission.clone()
        }
    }
    
    /// Check if this permission starts with the given prefix
    pub fn starts_with(&self, prefix: &str) -> bool {
        self.permission.starts_with(prefix)
    }
    
    /// Check if this permission matches another permission pattern
    /// Supports wildcards with "*"
    pub fn matches(&self, pattern: &str) -> bool {
        // Handle wildcard matching
        if pattern.ends_with("*") {
            let prefix = &pattern[..pattern.len() - 1];
            self.permission.starts_with(prefix)
        } else {
            self.permission == pattern
        }
    }
    
    /// Check if this permission grants access to a specific action on a resource
    pub fn grants_access(&self, platform: &str, resource: &str, action: &str) -> bool {
        if self.is_expired() {
            return false;
        }
        
        // Exact match
        if self.platform == platform && self.resource == resource && self.action == action {
            return true;
        }
        
        // Wildcard matching
        if self.action == "*" && self.platform == platform && self.resource == resource {
            return true;
        }
        
        if self.resource == "*" && self.platform == platform {
            return true;
        }
        
        false
    }
    
    fn parse_permission(permission: &str) -> Result<(String, String, String), ValueObjectError> {
        let parts: Vec<&str> = permission.split(':').collect();
        
        if parts.len() < 3 {
            return Err(ValueObjectError::InvalidFormat(
                "Permission must be in format 'platform:resource:action'".to_string()
            ));
        }
        
        Ok((
            parts[0].to_string(),
            parts[1].to_string(),
            parts[2].to_string(),
        ))
    }
}

impl ValueObject for Permission {
    type Error = ValueObjectError;
    
    fn validate(&self) -> Result<(), Self::Error> {
        if self.permission.is_empty() {
            return Err(ValueObjectError::Required("Permission cannot be empty".to_string()));
        }
        
        // Validate format
        Self::parse_permission(&self.permission)?;
        
        // Validate individual parts
        if self.platform.is_empty() {
            return Err(ValueObjectError::Required("Platform cannot be empty".to_string()));
        }
        
        if self.resource.is_empty() {
            return Err(ValueObjectError::Required("Resource cannot be empty".to_string()));
        }
        
        if self.action.is_empty() {
            return Err(ValueObjectError::Required("Action cannot be empty".to_string()));
        }
        
        // Validate allowed characters
        let valid_chars = |s: &str| s.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '*');
        
        if !valid_chars(&self.platform) || !valid_chars(&self.resource) || !valid_chars(&self.action) {
            return Err(ValueObjectError::InvalidFormat(
                "Permission parts can only contain alphanumeric characters, underscores, hyphens, and asterisks".to_string()
            ));
        }
        
        Ok(())
    }
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.permission)
    }
}

impl From<Permission> for String {
    fn from(permission: Permission) -> Self {
        permission.permission
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn valid_permission_should_pass() {
        let perm = Permission::new("admin:users:manage");
        assert!(perm.is_ok());
        
        let perm = perm.unwrap();
        assert_eq!(perm.platform(), "admin");
        assert_eq!(perm.resource(), "users");
        assert_eq!(perm.action(), "manage");
    }
    
    #[test]
    fn invalid_permission_format_should_fail() {
        let perm = Permission::new("invalid");
        assert!(perm.is_err());
    }
    
    #[test]
    fn permission_with_expiration() {
        let expires_at = Utc::now() + chrono::Duration::hours(1);
        let perm = Permission::new_with_expiration("admin:users:view", expires_at);
        assert!(perm.is_ok());
        
        let perm = perm.unwrap();
        assert!(!perm.is_expired());
        assert!(perm.is_active());
    }
    
    #[test]
    fn embedded_timestamp_format() {
        let timestamp = (Utc::now() + chrono::Duration::hours(1)).timestamp();
        let embedded = format!("admin:users:view:{}", timestamp);
        
        let perm = Permission::from_embedded_timestamp(&embedded);
        assert!(perm.is_ok());
        
        let perm = perm.unwrap();
        assert!(perm.expires_at().is_some());
        assert!(!perm.is_expired());
    }
    
    #[test]
    fn permission_matching() {
        let perm = Permission::new("admin:users:manage").unwrap();
        
        assert!(perm.matches("admin:users:manage"));
        assert!(perm.matches("admin:users:*"));
        assert!(perm.matches("admin:*"));
        assert!(!perm.matches("epsx:users:manage"));
    }
    
    #[test]
    fn grants_access_logic() {
        let perm = Permission::new("admin:users:manage").unwrap();
        
        assert!(perm.grants_access("admin", "users", "manage"));
        assert!(!perm.grants_access("admin", "users", "delete"));
        assert!(!perm.grants_access("epsx", "users", "manage"));
        
        let wildcard_perm = Permission::new("admin:users:*").unwrap();
        assert!(wildcard_perm.grants_access("admin", "users", "manage"));
        assert!(wildcard_perm.grants_access("admin", "users", "delete"));
    }
}