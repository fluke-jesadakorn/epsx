// UserPermission domain entity - separate permission management
// Individual permissions stored in dedicated table for better flexibility and audit

use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::dom::values::UserId;

/// Domain entity for individual user permissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserPermission {
    id: PermissionId,
    user_id: UserId,
    permission: String, // Format: "platform:resource:action" (e.g., "epsx:rankings:view:25")
    granted_at: DateTime<Utc>,
    granted_by: Option<UserId>, // Who granted this permission (nullable for system grants)
    expires_at: Option<DateTime<Utc>>, // When permission expires (nullable for permanent)
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// Permission ID value object
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PermissionId(pub Uuid);

impl PermissionId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
    
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl std::fmt::Display for PermissionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for PermissionId {
    type Err = uuid::Error;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl UserPermission {
    /// Create new permission (typically granted by system)
    pub fn new(
        user_id: UserId,
        permission: String,
        granted_by: Option<UserId>,
        expires_at: Option<DateTime<Utc>>
    ) -> Self {
        let now = Utc::now();
        Self {
            id: PermissionId::generate(),
            user_id,
            permission,
            granted_at: now,
            granted_by,
            expires_at,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Create permission from existing database data
    pub fn from_existing(
        id: PermissionId,
        user_id: UserId,
        permission: String,
        granted_at: DateTime<Utc>,
        granted_by: Option<UserId>,
        expires_at: Option<DateTime<Utc>>,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            user_id,
            permission,
            granted_at,
            granted_by,
            expires_at,
            is_active,
            created_at,
            updated_at,
        }
    }
    
    /// Full constructor for reconstructing from database
    pub fn reconstruct(
        id: PermissionId,
        user_id: UserId,
        permission: String,
        granted_at: DateTime<Utc>,
        granted_by: Option<UserId>,
        expires_at: Option<DateTime<Utc>>,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            user_id,
            permission,
            granted_at,
            granted_by,
            expires_at,
            is_active,
            created_at,
            updated_at,
        }
    }
    
    // Getters
    pub fn id(&self) -> &PermissionId {
        &self.id
    }
    
    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }
    
    pub fn permission(&self) -> &str {
        &self.permission
    }
    
    pub fn granted_at(&self) -> DateTime<Utc> {
        self.granted_at
    }
    
    pub fn granted_by(&self) -> Option<&UserId> {
        self.granted_by.as_ref()
    }
    
    pub fn expires_at(&self) -> Option<DateTime<Utc>> {
        self.expires_at
    }
    
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
    
    // Business methods
    
    /// Check if permission is currently valid (active and not expired)
    pub fn is_valid(&self) -> bool {
        if !self.is_active {
            return false;
        }
        
        if let Some(expires_at) = self.expires_at {
            if Utc::now() > expires_at {
                return false;
            }
        }
        
        true
    }
    
    /// Check if permission is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }
    
    /// Deactivate permission
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }
    
    /// Reactivate permission
    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }
    
    /// Extend expiration date
    pub fn extend_expiration(&mut self, new_expires_at: Option<DateTime<Utc>>) {
        self.expires_at = new_expires_at;
        self.updated_at = Utc::now();
    }
    
    /// Parse permission into components (platform:resource:action)
    pub fn parse_permission(&self) -> Result<PermissionComponents, PermissionError> {
        PermissionComponents::parse(&self.permission)
    }
    
    /// Check if this permission matches a required permission (with wildcard support)
    pub fn matches(&self, required: &str) -> bool {
        if !self.is_valid() {
            return false;
        }
        
        // Exact match
        if self.permission == required {
            return true;
        }
        
        // Try wildcard matching
        if let (Ok(this_perm), Ok(required_perm)) = (
            PermissionComponents::parse(&self.permission),
            PermissionComponents::parse(required)
        ) {
            return this_perm.matches(&required_perm);
        }
        
        false
    }
    
    /// Create system permission (granted by system, no expiration)
    pub fn system_permission(user_id: UserId, permission: String) -> Self {
        Self::new(user_id, permission, None, None)
    }
    
    /// Create temporary permission with expiration
    pub fn temporary_permission(
        user_id: UserId,
        permission: String,
        granted_by: UserId,
        expires_at: DateTime<Utc>
    ) -> Self {
        Self::new(user_id, permission, Some(granted_by), Some(expires_at))
    }
}

/// Permission components for parsing and matching
#[derive(Debug, Clone, PartialEq)]
pub struct PermissionComponents {
    pub platform: String,
    pub resource: String,
    pub action: String,
}

impl PermissionComponents {
    pub fn parse(permission: &str) -> Result<Self, PermissionError> {
        let parts: Vec<&str> = permission.split(':').collect();
        
        if parts.len() < 3 {
            return Err(PermissionError::InvalidFormat(
                format!("Permission must be in format 'platform:resource:action', got: {}", permission)
            ));
        }
        
        Ok(Self {
            platform: parts[0].to_string(),
            resource: parts[1].to_string(),
            action: parts.iter().skip(2).cloned().collect::<Vec<_>>().join(":"), // Handle complex actions like "view:25"
        })
    }
    
    /// Check if this permission matches another with wildcard support
    pub fn matches(&self, other: &PermissionComponents) -> bool {
        (self.platform == "*" || other.platform == "*" || self.platform == other.platform) &&
        (self.resource == "*" || other.resource == "*" || self.resource == other.resource) &&
        (self.action == "*" || other.action == "*" || self.action == other.action)
    }
}

/// Permission-related errors
#[derive(Debug, thiserror::Error)]
pub enum PermissionError {
    #[error("Invalid permission format: {0}")]
    InvalidFormat(String),
    
    #[error("Permission has expired")]
    Expired,
    
    #[error("Permission is not active")]
    NotActive,
    
    #[error("Access denied: insufficient permissions")]
    AccessDenied,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_permission_creation() {
        let user_id = UserId::generate();
        let permission = UserPermission::system_permission(
            user_id.clone(),
            "epsx:rankings:view:25".to_string()
        );
        
        assert_eq!(permission.user_id(), &user_id);
        assert_eq!(permission.permission(), "epsx:rankings:view:25");
        assert!(permission.is_active());
        assert!(permission.is_valid());
        assert!(!permission.is_expired());
        assert_eq!(permission.granted_by(), None); // System permission
        assert_eq!(permission.expires_at(), None); // No expiration
    }
    
    #[test]
    fn test_temporary_permission() {
        let user_id = UserId::generate();
        let granted_by = UserId::generate();
        let expires_at = Utc::now() + chrono::Duration::hours(1);
        
        let permission = UserPermission::temporary_permission(
            user_id.clone(),
            "epsx:admin:temp".to_string(),
            granted_by.clone(),
            expires_at
        );
        
        assert_eq!(permission.granted_by(), Some(&granted_by));
        assert_eq!(permission.expires_at(), Some(expires_at));
        assert!(!permission.is_expired());
        assert!(permission.is_valid());
    }
    
    #[test]
    fn test_permission_expiration() {
        let user_id = UserId::generate();
        let granted_by = UserId::generate();
        let expires_at = Utc::now() - chrono::Duration::hours(1); // Expired
        
        let permission = UserPermission::temporary_permission(
            user_id,
            "epsx:admin:temp".to_string(),
            granted_by,
            expires_at
        );
        
        assert!(permission.is_expired());
        assert!(!permission.is_valid()); // Expired permissions are not valid
    }
    
    #[test]
    fn test_permission_deactivation() {
        let user_id = UserId::generate();
        let mut permission = UserPermission::system_permission(
            user_id,
            "epsx:test".to_string()
        );
        
        assert!(permission.is_valid());
        
        permission.deactivate();
        assert!(!permission.is_active());
        assert!(!permission.is_valid()); // Inactive permissions are not valid
        
        permission.activate();
        assert!(permission.is_active());
        assert!(permission.is_valid());
    }
    
    #[test]
    fn test_permission_matching() {
        let user_id = UserId::generate();
        
        let admin_permission = UserPermission::system_permission(
            user_id.clone(),
            "admin:*:*".to_string()
        );
        
        let specific_permission = UserPermission::system_permission(
            user_id,
            "epsx:rankings:view:25".to_string()
        );
        
        // Admin permission should match any specific permission
        assert!(admin_permission.matches("epsx:rankings:view:25"));
        assert!(admin_permission.matches("epsx:users:manage"));
        
        // Specific permission should match exact requirement
        assert!(specific_permission.matches("epsx:rankings:view:25"));
        assert!(!specific_permission.matches("epsx:users:manage"));
    }
    
    #[test]
    fn test_permission_components_parsing() {
        let components = PermissionComponents::parse("epsx:rankings:view:25").unwrap();
        
        assert_eq!(components.platform, "epsx");
        assert_eq!(components.resource, "rankings");
        assert_eq!(components.action, "view:25"); // Complex action
        
        // Test wildcard matching
        let admin_components = PermissionComponents::parse("admin:*:*").unwrap();
        assert!(admin_components.matches(&components));
        assert!(!components.matches(&admin_components));
    }
    
    #[test]
    fn test_invalid_permission_format() {
        let result = PermissionComponents::parse("invalid-format");
        assert!(result.is_err());
        
        let result = PermissionComponents::parse("only:two");
        assert!(result.is_err());
    }
}