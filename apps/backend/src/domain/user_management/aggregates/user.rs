use std::collections::HashSet;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::shared_kernel::{
    AggregateRoot, 
    DomainEvent, 
    DomainError, 
    DomainResult,
    ValueObject,
    aggregate_root::AggregateBase
};

use crate::domain::user_management::value_objects::{
    UserId, Email, FirebaseUid, Permission
};

use crate::domain::user_management::events::{
    UserCreatedEvent, 
    UserEmailUpdatedEvent,
    UserActivatedEvent,
    UserDeactivatedEvent,
    PermissionGrantedEvent,
    PermissionRevokedEvent,
    UserPermissionsUpdatedEvent
};

/// User aggregate root
/// Represents a user in the system with their permissions and status
/// This is the consistency boundary for user-related operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    // Identity
    id: UserId,
    firebase_uid: FirebaseUid,
    email: Email,
    
    // Status
    is_active: bool,
    email_verified: bool,
    
    // Permissions
    permissions: HashSet<Permission>,
    
    // Audit fields
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    last_login_at: Option<DateTime<Utc>>,
    
    // Aggregate infrastructure
    #[serde(flatten)]
    base: AggregateBase,
}

impl User {
    /// Create a new user
    pub fn create(
        id: UserId,
        firebase_uid: FirebaseUid,
        email: Email,
    ) -> DomainResult<Self> {
        // Business rule: Email must be valid
        email.validate()
            .map_err(|e| DomainError::validation_error("email", e.to_string()))?;
        
        // Business rule: Firebase UID must be valid  
        firebase_uid.validate()
            .map_err(|e| DomainError::validation_error("firebase_uid", e.to_string()))?;
        
        let now = Utc::now();
        let base = AggregateBase::new();
        
        let mut user = Self {
            id: id.clone(),
            firebase_uid: firebase_uid.clone(),
            email: email.clone(),
            is_active: true, // New users are active by default
            email_verified: false, // Email verification required
            permissions: HashSet::new(),
            created_at: now,
            updated_at: now,
            last_login_at: None,
            base,
        };
        
        // Raise domain event
        user.base.add_event(Box::new(UserCreatedEvent::new(
            id,
            email,
            firebase_uid,
            user.base.version
        )));
        
        Ok(user)
    }
    
    /// Load existing user (for repository reconstruction)
    pub fn load(
        id: UserId,
        firebase_uid: FirebaseUid,
        email: Email,
        is_active: bool,
        email_verified: bool,
        permissions: HashSet<Permission>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        last_login_at: Option<DateTime<Utc>>,
        version: u64,
    ) -> Self {
        let mut base = AggregateBase::new();
        base.version = version;
        base.created_at = created_at;
        base.updated_at = updated_at;
        
        Self {
            id,
            firebase_uid,
            email,
            is_active,
            email_verified,
            permissions,
            created_at,
            updated_at,
            last_login_at,
            base,
        }
    }
    
    // Getters
    pub fn id(&self) -> &UserId {
        &self.id
    }
    
    pub fn firebase_uid(&self) -> &FirebaseUid {
        &self.firebase_uid
    }
    
    pub fn email(&self) -> &Email {
        &self.email
    }
    
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    
    pub fn is_email_verified(&self) -> bool {
        self.email_verified
    }
    
    pub fn permissions(&self) -> &HashSet<Permission> {
        &self.permissions
    }
    
    pub fn last_login_at(&self) -> Option<DateTime<Utc>> {
        self.last_login_at
    }
    
    // Business operations
    
    /// Update the user's email address
    pub fn update_email(&mut self, new_email: Email) -> DomainResult<()> {
        // Business rule: Cannot update to same email
        if self.email == new_email {
            return Err(DomainError::business_rule_violation(
                "New email must be different from current email"
            ));
        }
        
        // Business rule: Email must be valid
        new_email.validate()
            .map_err(|e| DomainError::validation_error("email", e.to_string()))?;
        
        let old_email = self.email.clone();
        self.email = new_email.clone();
        self.email_verified = false; // Must re-verify new email
        self.base.touch();
        
        // Raise domain event
        self.base.add_event(Box::new(UserEmailUpdatedEvent::new(
            self.id.clone(),
            old_email,
            new_email,
            self.base.version
        )));
        
        Ok(())
    }
    
    /// Activate the user
    pub fn activate(&mut self) -> DomainResult<()> {
        // Business rule: Cannot activate already active user
        if self.is_active {
            return Err(DomainError::business_rule_violation(
                "User is already active"
            ));
        }
        
        self.is_active = true;
        self.base.touch();
        
        // Raise domain event
        self.base.add_event(Box::new(UserActivatedEvent::new(
            self.id.clone(),
            self.base.version
        )));
        
        Ok(())
    }
    
    /// Deactivate the user
    pub fn deactivate(&mut self, reason: Option<String>) -> DomainResult<()> {
        // Business rule: Cannot deactivate already inactive user
        if !self.is_active {
            return Err(DomainError::business_rule_violation(
                "User is already inactive"
            ));
        }
        
        self.is_active = false;
        self.base.touch();
        
        // Raise domain event
        self.base.add_event(Box::new(UserDeactivatedEvent::new(
            self.id.clone(),
            reason,
            self.base.version
        )));
        
        Ok(())
    }
    
    /// Verify the user's email
    pub fn verify_email(&mut self) -> DomainResult<()> {
        // Business rule: Cannot verify already verified email
        if self.email_verified {
            return Err(DomainError::business_rule_violation(
                "Email is already verified"
            ));
        }
        
        self.email_verified = true;
        self.base.touch();
        
        Ok(())
    }
    
    /// Grant a permission to the user
    pub fn grant_permission(
        &mut self, 
        permission: Permission, 
        granted_by: Option<UserId>
    ) -> DomainResult<()> {
        // Business rule: User must be active to receive permissions
        if !self.is_active {
            return Err(DomainError::business_rule_violation(
                "Cannot grant permissions to inactive user"
            ));
        }
        
        // Business rule: Permission must be valid
        permission.validate()
            .map_err(|e| DomainError::validation_error("permission", e.to_string()))?;
        
        // Business rule: Cannot grant expired permission
        if permission.is_expired() {
            return Err(DomainError::business_rule_violation(
                "Cannot grant expired permission"
            ));
        }
        
        // Business rule: Cannot grant duplicate permission
        if self.has_permission_exact(&permission) {
            return Err(DomainError::business_rule_violation(
                "User already has this permission"
            ));
        }
        
        self.permissions.insert(permission.clone());
        self.base.touch();
        
        // Raise domain event
        self.base.add_event(Box::new(PermissionGrantedEvent::new(
            self.id.clone(),
            permission,
            granted_by,
            self.base.version
        )));
        
        Ok(())
    }
    
    /// Revoke a permission from the user
    pub fn revoke_permission(
        &mut self, 
        permission: &Permission, 
        revoked_by: Option<UserId>,
        reason: Option<String>
    ) -> DomainResult<()> {
        // Business rule: Cannot revoke permission user doesn't have
        if !self.has_permission_exact(permission) {
            return Err(DomainError::business_rule_violation(
                "User doesn't have this permission"
            ));
        }
        
        self.permissions.remove(permission);
        self.base.touch();
        
        // Raise domain event
        self.base.add_event(Box::new(PermissionRevokedEvent::new(
            self.id.clone(),
            permission.clone(),
            revoked_by,
            reason,
            self.base.version
        )));
        
        Ok(())
    }
    
    /// Update all user permissions (bulk operation)
    pub fn update_permissions(
        &mut self,
        new_permissions: HashSet<Permission>,
        updated_by: Option<UserId>
    ) -> DomainResult<()> {
        // Business rule: User must be active to update permissions
        if !self.is_active {
            return Err(DomainError::business_rule_violation(
                "Cannot update permissions for inactive user"
            ));
        }
        
        // Validate all new permissions
        for permission in &new_permissions {
            permission.validate()
                .map_err(|e| DomainError::validation_error("permission", e.to_string()))?;
            
            if permission.is_expired() {
                return Err(DomainError::business_rule_violation(
                    format!("Cannot add expired permission: {}", permission)
                ));
            }
        }
        
        // Calculate changes
        let added_permissions: Vec<Permission> = new_permissions
            .difference(&self.permissions)
            .cloned()
            .collect();
            
        let removed_permissions: Vec<Permission> = self.permissions
            .difference(&new_permissions)
            .cloned()
            .collect();
        
        // Update permissions
        self.permissions = new_permissions;
        self.base.touch();
        
        // Raise domain event if there were changes
        if !added_permissions.is_empty() || !removed_permissions.is_empty() {
            self.base.add_event(Box::new(UserPermissionsUpdatedEvent::new(
                self.id.clone(),
                added_permissions,
                removed_permissions,
                updated_by,
                self.base.version
            )));
        }
        
        Ok(())
    }
    
    /// Record user login
    pub fn record_login(&mut self) -> DomainResult<()> {
        // Business rule: Only active users can login
        if !self.is_active {
            return Err(DomainError::business_rule_violation(
                "Inactive users cannot login"
            ));
        }
        
        self.last_login_at = Some(Utc::now());
        self.base.touch();
        
        Ok(())
    }
    
    /// Check if user has a specific permission (exact match)
    pub fn has_permission_exact(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }
    
    /// Check if user has access to perform an action on a resource
    pub fn has_access(&self, platform: &str, resource: &str, action: &str) -> bool {
        // Only active users can have access
        if !self.is_active {
            return false;
        }
        
        // Check active permissions
        self.permissions
            .iter()
            .filter(|p| p.is_active()) // Only consider non-expired permissions
            .any(|p| p.grants_access(platform, resource, action))
    }
    
    /// Get all active permissions as strings
    pub fn active_permissions(&self) -> Vec<String> {
        self.permissions
            .iter()
            .filter(|p| p.is_active())
            .map(|p| p.as_str().to_string())
            .collect()
    }
    
    /// Check if user is eligible for automatic permission assignment
    pub fn is_eligible_for_auto_assignment(&self) -> bool {
        self.is_active && self.email_verified
    }
    
    /// Remove expired permissions (cleanup operation)
    pub fn cleanup_expired_permissions(&mut self) -> u32 {
        let expired_count = self.permissions.iter().filter(|p| p.is_expired()).count();
        
        if expired_count > 0 {
            self.permissions.retain(|p| !p.is_expired());
            self.base.touch();
        }
        
        expired_count as u32
    }
}

impl AggregateRoot for User {
    type Id = UserId;
    
    fn id(&self) -> &Self::Id {
        &self.id
    }
    
    fn version(&self) -> u64 {
        self.base.version
    }
    
    fn increment_version(&mut self) {
        self.base.increment_version();
    }
    
    fn uncommitted_events(&self) -> &[Box<dyn DomainEvent>] {
        &self.base.events
    }
    
    fn mark_events_as_committed(&mut self) {
        self.base.clear_events();
    }
    
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    
    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
    
    fn touch(&mut self) {
        self.base.touch();
        self.updated_at = self.base.updated_at;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_user() -> User {
        User::create(
            UserId::new(),
            FirebaseUid::new("test_firebase_uid").unwrap(),
            Email::new("test@example.com").unwrap(),
        ).unwrap()
    }
    
    #[test]
    fn create_user_should_succeed() {
        let user = create_test_user();
        assert!(user.is_active());
        assert!(!user.is_email_verified());
        assert_eq!(user.uncommitted_events().len(), 1);
    }
    
    #[test]
    fn grant_permission_should_succeed() {
        let mut user = create_test_user();
        let permission = Permission::new("admin:users:view").unwrap();
        
        let result = user.grant_permission(permission.clone(), None);
        assert!(result.is_ok());
        assert!(user.has_permission_exact(&permission));
    }
    
    #[test]
    fn grant_duplicate_permission_should_fail() {
        let mut user = create_test_user();
        let permission = Permission::new("admin:users:view").unwrap();
        
        user.grant_permission(permission.clone(), None).unwrap();
        let result = user.grant_permission(permission, None);
        
        assert!(result.is_err());
    }
    
    #[test]
    fn grant_permission_to_inactive_user_should_fail() {
        let mut user = create_test_user();
        user.deactivate(Some("Test".to_string())).unwrap();
        
        let permission = Permission::new("admin:users:view").unwrap();
        let result = user.grant_permission(permission, None);
        
        assert!(result.is_err());
    }
    
    #[test]
    fn has_access_should_work_correctly() {
        let mut user = create_test_user();
        let permission = Permission::new("admin:users:*").unwrap();
        
        user.grant_permission(permission, None).unwrap();
        
        assert!(user.has_access("admin", "users", "view"));
        assert!(user.has_access("admin", "users", "manage"));
        assert!(!user.has_access("epsx", "users", "view"));
    }
    
    #[test]
    fn inactive_user_has_no_access() {
        let mut user = create_test_user();
        let permission = Permission::new("admin:users:view").unwrap();
        
        user.grant_permission(permission, None).unwrap();
        user.deactivate(Some("Test".to_string())).unwrap();
        
        assert!(!user.has_access("admin", "users", "view"));
    }
}