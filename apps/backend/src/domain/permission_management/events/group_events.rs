use crate::prelude::*;
use crate::domain::shared_kernel::{DomainEvent, EventMetadata};
use uuid::Uuid;

/// Event emitted when a group is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupCreatedEvent {
    pub metadata: EventMetadata,
    pub group_id: String,
    pub name: String,
    pub slug: String,
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl GroupCreatedEvent {
    pub fn new(
        aggregate_id: String,
        aggregate_version: u64,
        group_id: String,
        name: String,
        slug: String,
        permissions: Vec<String>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id, aggregate_version),
            group_id,
            name,
            slug,
            permissions,
            created_at,
        }
    }
}

impl DomainEvent for GroupCreatedEvent {
    fn event_id(&self) -> Uuid { self.metadata.event_id }
    fn event_type(&self) -> &'static str { "GroupCreated" }
    fn aggregate_type(&self) -> &'static str { "Group" }
    fn occurred_at(&self) -> DateTime<Utc> { self.metadata.occurred_at }
    fn aggregate_version(&self) -> u64 { self.metadata.aggregate_version }
    fn aggregate_id(&self) -> String { self.metadata.aggregate_id.clone() }
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Event emitted when a group is updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupUpdatedEvent {
    pub metadata: EventMetadata,
    pub group_id: String,
    pub updated_at: DateTime<Utc>,
}

impl GroupUpdatedEvent {
    pub fn new(aggregate_id: String, aggregate_version: u64, group_id: String, updated_at: DateTime<Utc>) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id, aggregate_version),
            group_id,
            updated_at,
        }
    }
}

impl DomainEvent for GroupUpdatedEvent {
    fn event_id(&self) -> Uuid { self.metadata.event_id }
    fn event_type(&self) -> &'static str { "GroupUpdated" }
    fn aggregate_type(&self) -> &'static str { "Group" }
    fn occurred_at(&self) -> DateTime<Utc> { self.metadata.occurred_at }
    fn aggregate_version(&self) -> u64 { self.metadata.aggregate_version }
    fn aggregate_id(&self) -> String { self.metadata.aggregate_id.clone() }
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Event emitted when a group is deleted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupDeletedEvent {
    pub metadata: EventMetadata,
    pub group_id: String,
    pub deleted_at: DateTime<Utc>,
}

impl GroupDeletedEvent {
    pub fn new(aggregate_id: String, aggregate_version: u64, group_id: String, deleted_at: DateTime<Utc>) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id, aggregate_version),
            group_id,
            deleted_at,
        }
    }
}

impl DomainEvent for GroupDeletedEvent {
    fn event_id(&self) -> Uuid { self.metadata.event_id }
    fn event_type(&self) -> &'static str { "GroupDeleted" }
    fn aggregate_type(&self) -> &'static str { "Group" }
    fn occurred_at(&self) -> DateTime<Utc> { self.metadata.occurred_at }
    fn aggregate_version(&self) -> u64 { self.metadata.aggregate_version }
    fn aggregate_id(&self) -> String { self.metadata.aggregate_id.clone() }
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Event emitted when a wallet is assigned to a group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAssignedToGroupEvent {
    pub metadata: EventMetadata,
    pub group_id: String,
    pub wallet_address: String,
    pub assigned_at: DateTime<Utc>,
}

impl WalletAssignedToGroupEvent {
    pub fn new(
        aggregate_id: String,
        aggregate_version: u64,
        group_id: String,
        wallet_address: String,
        assigned_at: DateTime<Utc>,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id, aggregate_version),
            group_id,
            wallet_address,
            assigned_at,
        }
    }
}

impl DomainEvent for WalletAssignedToGroupEvent {
    fn event_id(&self) -> Uuid { self.metadata.event_id }
    fn event_type(&self) -> &'static str { "WalletAssignedToGroup" }
    fn aggregate_type(&self) -> &'static str { "Group" }
    fn occurred_at(&self) -> DateTime<Utc> { self.metadata.occurred_at }
    fn aggregate_version(&self) -> u64 { self.metadata.aggregate_version }
    fn aggregate_id(&self) -> String { self.metadata.aggregate_id.clone() }
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Event emitted when a wallet is removed from a group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletRemovedFromGroupEvent {
    pub metadata: EventMetadata,
    pub group_id: String,
    pub wallet_address: String,
    pub removed_at: DateTime<Utc>,
}

impl WalletRemovedFromGroupEvent {
    pub fn new(
        aggregate_id: String,
        aggregate_version: u64,
        group_id: String,
        wallet_address: String,
        removed_at: DateTime<Utc>,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id, aggregate_version),
            group_id,
            wallet_address,
            removed_at,
        }
    }
}

impl DomainEvent for WalletRemovedFromGroupEvent {
    fn event_id(&self) -> Uuid { self.metadata.event_id }
    fn event_type(&self) -> &'static str { "WalletRemovedFromGroup" }
    fn aggregate_type(&self) -> &'static str { "Group" }
    fn occurred_at(&self) -> DateTime<Utc> { self.metadata.occurred_at }
    fn aggregate_version(&self) -> u64 { self.metadata.aggregate_version }
    fn aggregate_id(&self) -> String { self.metadata.aggregate_id.clone() }
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

// Type aliases for backward compatibility during migration
pub type PermissionGroupCreatedEvent = GroupCreatedEvent;
pub type PermissionGroupUpdatedEvent = GroupUpdatedEvent;
pub type PermissionGroupDeletedEvent = GroupDeletedEvent;
