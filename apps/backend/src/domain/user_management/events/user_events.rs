use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::shared_kernel::{DomainEvent, domain_event::EventMetadata};
use crate::domain::shared_kernel::value_objects::UserId;
use crate::domain::user_management::value_objects::Email;

/// Event raised when a new user is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCreatedEvent {
    pub metadata: EventMetadata,
    pub user_id: UserId,
    pub email: Email,
}

impl UserCreatedEvent {
    pub fn new(user_id: UserId, email: Email, aggregate_version: u64) -> Self {
        Self {
            metadata: EventMetadata::new(user_id.to_string(), aggregate_version),
            user_id,
            email,
        }
    }
}

impl DomainEvent for UserCreatedEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }
    
    fn event_type(&self) -> &'static str {
        "UserCreated"
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }
    
    fn aggregate_version(&self) -> u64 {
        self.metadata.aggregate_version
    }
    
    fn aggregate_id(&self) -> String {
        self.metadata.aggregate_id.clone()
    }
    
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

/// Event raised when a user's email is updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEmailUpdatedEvent {
    pub metadata: EventMetadata,
    pub user_id: UserId,
    pub old_email: Email,
    pub new_email: Email,
}

impl UserEmailUpdatedEvent {
    pub fn new(user_id: UserId, old_email: Email, new_email: Email, aggregate_version: u64) -> Self {
        Self {
            metadata: EventMetadata::new(user_id.to_string(), aggregate_version),
            user_id,
            old_email,
            new_email,
        }
    }
}

impl DomainEvent for UserEmailUpdatedEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }
    
    fn event_type(&self) -> &'static str {
        "UserEmailUpdated"
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }
    
    fn aggregate_version(&self) -> u64 {
        self.metadata.aggregate_version
    }
    
    fn aggregate_id(&self) -> String {
        self.metadata.aggregate_id.clone()
    }
    
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

/// Event raised when a user is activated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivatedEvent {
    pub metadata: EventMetadata,
    pub user_id: UserId,
}

impl UserActivatedEvent {
    pub fn new(user_id: UserId, aggregate_version: u64) -> Self {
        Self {
            metadata: EventMetadata::new(user_id.to_string(), aggregate_version),
            user_id,
        }
    }
}

impl DomainEvent for UserActivatedEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }
    
    fn event_type(&self) -> &'static str {
        "UserActivated"
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }
    
    fn aggregate_version(&self) -> u64 {
        self.metadata.aggregate_version
    }
    
    fn aggregate_id(&self) -> String {
        self.metadata.aggregate_id.clone()
    }
    
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

/// Event raised when a user is deactivated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDeactivatedEvent {
    pub metadata: EventMetadata,
    pub user_id: UserId,
    pub reason: Option<String>,
}

impl UserDeactivatedEvent {
    pub fn new(user_id: UserId, reason: Option<String>, aggregate_version: u64) -> Self {
        Self {
            metadata: EventMetadata::new(user_id.to_string(), aggregate_version),
            user_id,
            reason,
        }
    }
}

impl DomainEvent for UserDeactivatedEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }
    
    fn event_type(&self) -> &'static str {
        "UserDeactivated"
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }
    
    fn aggregate_version(&self) -> u64 {
        self.metadata.aggregate_version
    }
    
    fn aggregate_id(&self) -> String {
        self.metadata.aggregate_id.clone()
    }
    
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}