// User domain entity with minimal naming conventions

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::dom::values::{UserId, Email, PermissionSet, Subscription, Role};
use crate::dom::events::UserRoleChangedEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: UserId,
    firebase_uid: String,
    email: Email,
    role: Role,
    permissions: PermissionSet,
    subscription: Subscription,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

impl User {
    pub fn new(firebase_uid: String, email: Email, role: Role) -> Self {
        let id = UserId::generate();
        let now = Utc::now();
        
        Self {
            id: id.clone(),
            firebase_uid,
            email,
            permissions: PermissionSet::for_role(&role),
            subscription: Subscription::free(),
            role,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }
    
    /// Create user from existing database data (basic version)
    pub fn from_existing(
        id: UserId,
        firebase_uid: String,
        email: Email,
        role: Role,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            firebase_uid,
            email,
            permissions: PermissionSet::for_role(&role),
            subscription: Subscription::free(), // Default to free subscription
            role,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }
    
    /// Create user from complete existing database data
    pub fn from_existing_complete(
        id: UserId,
        firebase_uid: String,
        email: Email,
        role: Role,
        subscription: Subscription,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
        deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Self {
        Self {
            id,
            firebase_uid,
            email,
            permissions: PermissionSet::for_role(&role),
            subscription,
            role,
            created_at,
            updated_at,
            deleted_at,
        }
    }
    
    pub fn reconstruct(
        id: UserId,
        firebase_uid: String,
        email: Email, 
        role: Role,
        permissions: PermissionSet,
        subscription: Subscription,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        deleted_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            firebase_uid,
            email,
            role,
            permissions,
            subscription,
            created_at,
            updated_at,
            deleted_at,
        }
    }
    
    // Getters
    pub fn id(&self) -> &UserId { &self.id }
    pub fn firebase_uid(&self) -> &str { &self.firebase_uid }
    pub fn email(&self) -> &Email { &self.email }
    pub fn role(&self) -> &Role { &self.role }
    pub fn permissions(&self) -> &PermissionSet { &self.permissions }
    pub fn subscription(&self) -> &Subscription { &self.subscription }
    
    // Backward compatibility getters (deprecated)
    #[deprecated(note = "Use permissions() instead")]
    pub fn perms(&self) -> &PermissionSet { &self.permissions }
    #[deprecated(note = "Use subscription() instead")]
    pub fn sub(&self) -> &Subscription { &self.subscription }
    pub fn created_at(&self) -> DateTime<Utc> { self.created_at }
    pub fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
    pub fn deleted_at(&self) -> Option<DateTime<Utc>> { self.deleted_at }
    pub fn is_deleted(&self) -> bool { self.deleted_at.is_some() }
    
    // Business methods
    pub fn upgrade_role(&mut self, new_role: Role) -> Result<UserRoleChangedEvent, DomainError> {
        if !self.can_upgrade_to(&new_role) {
            return Err(DomainError::RoleUpgradeNotAllowed {
                current: self.role.clone(),
                target: new_role,
            });
        }
        
        let old_role = self.role.clone();
        self.role = new_role.clone();
        self.permissions = PermissionSet::for_role(&new_role);
        self.updated_at = Utc::now();
        
        Ok(UserRoleChangedEvent::new(self.id.clone(), old_role, new_role))
    }
    
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(permission)
    }
    
    // Backward compatibility method (deprecated)
    #[deprecated(note = "Use has_permission() instead")]
    pub fn has_perm(&self, perm: &str) -> bool {
        self.permissions.contains(perm)
    }
    
    pub fn update_subscription(&mut self, subscription: Subscription) {
        self.subscription = subscription;
        self.updated_at = Utc::now();
    }
    
    // Backward compatibility method (deprecated)
    #[deprecated(note = "Use update_subscription() instead")]
    pub fn update_sub(&mut self, sub: Subscription) {
        self.subscription = sub;
        self.updated_at = Utc::now();
    }
    
    pub fn is_active(&self) -> bool {
        !self.is_deleted() && self.subscription.is_active()
    }
    
    pub fn soft_delete(&mut self) {
        self.deleted_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
    
    pub fn restore(&mut self) {
        self.deleted_at = None;
        self.updated_at = Utc::now();
    }
    
    fn can_upgrade_to(&self, target_role: &Role) -> bool {
        use Role::*;
        
        match (&self.role, target_role) {
            (User, Premium) => true,
            (Premium, Moderator) => true,
            (Moderator, Admin) => true,
            (Admin, SuperAdmin) => true,
            _ => false,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Cannot upgrade from {current:?} to {target:?}")]
    RoleUpgradeNotAllowed { current: Role, target: Role },
    
    #[error("Invalid email: {0}")]
    InvalidEmail(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn should_create_new_user() {
        let email = Email::new("test@example.com").unwrap();
        let firebase_uid = "firebase123".to_string();
        let user = User::new(firebase_uid.clone(), email.clone(), Role::User);
        
        assert_eq!(user.firebase_uid(), &firebase_uid);
        assert_eq!(user.email(), &email);
        assert_eq!(user.role(), &Role::User);
        assert!(user.has_perm("read:own_data"));
    }
    
    #[test]
    fn should_upgrade_user_role() {
        let email = Email::new("test@example.com").unwrap();
        let firebase_uid = "firebase123".to_string();
        let mut user = User::new(firebase_uid, email, Role::User);
        
        let event = user.upgrade_role(Role::Premium).unwrap();
        
        assert_eq!(user.role(), &Role::Premium);
        assert!(user.has_perm("access:premium_features"));
        assert_eq!(event.new_role(), &Role::Premium);
    }
    
    #[test]
    fn should_reject_invalid_upgrade() {
        let email = Email::new("test@example.com").unwrap();
        let firebase_uid = "firebase123".to_string();
        let mut user = User::new(firebase_uid, email, Role::User);
        
        let result = user.upgrade_role(Role::SuperAdmin);
        
        assert!(result.is_err());
    }
}