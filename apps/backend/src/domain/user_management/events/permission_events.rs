use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::shared_kernel::{DomainEvent, domain_event::EventMetadata};
use crate::domain::shared_kernel::value_objects::UserId;
use crate::domain::user_management::value_objects::Permission;

/// Event raised when a permission is granted to a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionGrantedEvent {
    pub metadata: EventMetadata,
    pub user_id: UserId,
    pub permission: Permission,
    pub granted_by: Option<UserId>,
}

impl PermissionGrantedEvent {
    pub fn new(
        user_id: UserId, 
        permission: Permission, 
        granted_by: Option<UserId>,
        aggregate_version: u64
    ) -> Self {
        Self {
            metadata: EventMetadata::new(user_id.to_string(), aggregate_version),
            user_id,
            permission,
            granted_by,
        }
    }
}

impl DomainEvent for PermissionGrantedEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }
    
    fn event_type(&self) -> &'static str {
        "PermissionGranted"
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

/// Event raised when a permission is revoked from a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRevokedEvent {
    pub metadata: EventMetadata,
    pub user_id: UserId,
    pub permission: Permission,
    pub revoked_by: Option<UserId>,
    pub reason: Option<String>,
}

impl PermissionRevokedEvent {
    pub fn new(
        user_id: UserId, 
        permission: Permission, 
        revoked_by: Option<UserId>,
        reason: Option<String>,
        aggregate_version: u64
    ) -> Self {
        Self {
            metadata: EventMetadata::new(user_id.to_string(), aggregate_version),
            user_id,
            permission,
            revoked_by,
            reason,
        }
    }
}

impl DomainEvent for PermissionRevokedEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }
    
    fn event_type(&self) -> &'static str {
        "PermissionRevoked"
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

/// Event raised when a permission expires
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionExpiredEvent {
    pub metadata: EventMetadata,
    pub user_id: UserId,
    pub permission: Permission,
}

impl PermissionExpiredEvent {
    pub fn new(user_id: UserId, permission: Permission, aggregate_version: u64) -> Self {
        Self {
            metadata: EventMetadata::new(user_id.to_string(), aggregate_version),
            user_id,
            permission,
        }
    }
}

impl DomainEvent for PermissionExpiredEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }
    
    fn event_type(&self) -> &'static str {
        "PermissionExpired"
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

/// Event raised when multiple permissions are updated for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissionsUpdatedEvent {
    pub metadata: EventMetadata,
    pub user_id: UserId,
    pub added_permissions: Vec<Permission>,
    pub removed_permissions: Vec<Permission>,
    pub updated_by: Option<UserId>,
}

impl UserPermissionsUpdatedEvent {
    pub fn new(
        user_id: UserId,
        added_permissions: Vec<Permission>,
        removed_permissions: Vec<Permission>,
        updated_by: Option<UserId>,
        aggregate_version: u64
    ) -> Self {
        Self {
            metadata: EventMetadata::new(user_id.to_string(), aggregate_version),
            user_id,
            added_permissions,
            removed_permissions,
            updated_by,
        }
    }
}

impl DomainEvent for UserPermissionsUpdatedEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }
    
    fn event_type(&self) -> &'static str {
        "UserPermissionsUpdated"
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