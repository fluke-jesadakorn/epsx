// User domain entity with minimal naming conventions

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::dom::values::{UserId, Email, PermSet, Subscription, Role};
use crate::dom::events::UserRoleChangedEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: UserId,
    email: Email,
    role: Role,
    perms: PermSet,
    sub: Subscription,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(email: Email, role: Role) -> Self {
        let id = UserId::generate();
        let now = Utc::now();
        
        Self {
            id: id.clone(),
            email,
            perms: PermSet::for_role(&role),
            sub: Subscription::free(),
            role,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn reconstruct(
        id: UserId,
        email: Email, 
        role: Role,
        perms: PermSet,
        sub: Subscription,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            email,
            role,
            perms,
            sub,
            created_at,
            updated_at,
        }
    }
    
    // Getters
    pub fn id(&self) -> &UserId { &self.id }
    pub fn email(&self) -> &Email { &self.email }
    pub fn role(&self) -> &Role { &self.role }
    pub fn perms(&self) -> &PermSet { &self.perms }
    pub fn sub(&self) -> &Subscription { &self.sub }
    pub fn created_at(&self) -> DateTime<Utc> { self.created_at }
    pub fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
    
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
        self.perms = PermSet::for_role(&new_role);
        self.updated_at = Utc::now();
        
        Ok(UserRoleChangedEvent::new(self.id.clone(), old_role, new_role))
    }
    
    pub fn has_perm(&self, perm: &str) -> bool {
        self.perms.contains(perm)
    }
    
    pub fn update_sub(&mut self, sub: Subscription) {
        self.sub = sub;
        self.updated_at = Utc::now();
    }
    
    pub fn is_active(&self) -> bool {
        self.sub.is_active()
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
        let user = User::new(email.clone(), Role::User);
        
        assert_eq!(user.email(), &email);
        assert_eq!(user.role(), &Role::User);
        assert!(user.has_perm("read:own_data"));
    }
    
    #[test]
    fn should_upgrade_user_role() {
        let email = Email::new("test@example.com").unwrap();
        let mut user = User::new(email, Role::User);
        
        let event = user.upgrade_role(Role::Premium).unwrap();
        
        assert_eq!(user.role(), &Role::Premium);
        assert!(user.has_perm("access:premium_features"));
        assert_eq!(event.new_role(), &Role::Premium);
    }
    
    #[test]
    fn should_reject_invalid_upgrade() {
        let email = Email::new("test@example.com").unwrap();
        let mut user = User::new(email, Role::User);
        
        let result = user.upgrade_role(Role::SuperAdmin);
        
        assert!(result.is_err());
    }
}