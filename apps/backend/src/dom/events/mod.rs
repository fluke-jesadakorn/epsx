// Domain events for decoupled communication
use chrono::{DateTime, Utc};
use uuid::Uuid;


use serde::{Serialize, Deserialize};


use crate::dom::values::{UserId};


// Removed: notification_events - will be re-implemented

pub trait DomainEvent: Send + Sync {
    fn event_id(&self) -> &Uuid;
    fn occurred_at(&self) -> DateTime<Utc>;
    fn event_type(&self) -> &'static str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissionChangedEvent {
    event_id: Uuid,
    occurred_at: DateTime<Utc>,
    user_id: UserId,
    permissions_added: Vec<String>,
    permissions_removed: Vec<String>,
    old_package_tier: String,
    new_package_tier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRegisteredEvent {
    event_id: Uuid,
    occurred_at: DateTime<Utc>,
    user_id: UserId,
    email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDeletedEvent {
    event_id: Uuid,
    occurred_at: DateTime<Utc>,
    user_id: UserId,
    deleted_by: UserId,
    reason: Option<String>,
}

impl UserPermissionChangedEvent {
    pub fn new(
        user_id: UserId, 
        permissions_added: Vec<String>,
        permissions_removed: Vec<String>,
        old_package_tier: String, 
        new_package_tier: String
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            user_id,
            permissions_added,
            permissions_removed,
            old_package_tier,
            new_package_tier,
        }
    }
    
    pub fn user_id(&self) -> &UserId { &self.user_id }
    pub fn permissions_added(&self) -> &[String] { &self.permissions_added }
    pub fn permissions_removed(&self) -> &[String] { &self.permissions_removed }
    pub fn old_package_tier(&self) -> &str { &self.old_package_tier }
    pub fn new_package_tier(&self) -> &str { &self.new_package_tier }
}

impl DomainEvent for UserPermissionChangedEvent {
    fn event_id(&self) -> &Uuid { &self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "UserPermissionChanged" }
}


impl UserRegisteredEvent {
    pub fn new(user_id: UserId, email: String) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            user_id,
            email,
        }
    }
    
    pub fn user_id(&self) -> &UserId { &self.user_id }
    pub fn email(&self) -> &str { &self.email }
}

impl DomainEvent for UserRegisteredEvent {
    fn event_id(&self) -> &Uuid { &self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "UserRegistered" }
}

impl UserDeletedEvent {
    pub fn new(user_id: UserId, deleted_by: UserId, reason: Option<String>) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            user_id,
            deleted_by,
            reason,
        }
    }
    
    pub fn user_id(&self) -> &UserId { &self.user_id }
    pub fn deleted_by(&self) -> &UserId { &self.deleted_by }
    pub fn reason(&self) -> &Option<String> { &self.reason }
}

impl DomainEvent for UserDeletedEvent {
    fn event_id(&self) -> &Uuid { &self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "UserDeleted" }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::values::UserId;
    
    #[test]
    fn should_create_user_permission_changed_event() {
        let user_id = UserId::generate();
        let event = UserPermissionChangedEvent::new(
            user_id.clone(),
            vec!["user-management".to_string()],
            vec![],
            "FREE".to_string(),
            "BRONZE".to_string()
        );
        
        assert_eq!(event.user_id(), &user_id);
        assert_eq!(event.permissions_added(), &["user-management"]);
        assert_eq!(event.permissions_removed().len(), 0);
        assert_eq!(event.old_package_tier(), "FREE");
        assert_eq!(event.new_package_tier(), "BRONZE");
        assert_eq!(event.event_type(), "UserPermissionChanged");
    }
    
}