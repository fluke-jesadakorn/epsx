use crate::prelude::*;
use crate::domain::shared_kernel::{DomainEvent, EventMetadata};
use uuid::Uuid;

/// Event emitted when a plan is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanCreatedEvent {
    pub metadata: EventMetadata,
    pub plan_id: String,
    pub name: String,
    pub slug: String,
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl PlanCreatedEvent {
    pub fn new(
        aggregate_id: String,
        aggregate_version: u64,
        plan_id: String,
        name: String,
        slug: String,
        permissions: Vec<String>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id, aggregate_version),
            plan_id,
            name,
            slug,
            permissions,
            created_at,
        }
    }
}

impl DomainEvent for PlanCreatedEvent {
    fn event_id(&self) -> Uuid { self.metadata.event_id }
    fn event_type(&self) -> &'static str { "PlanCreated" }
    fn aggregate_type(&self) -> &'static str { "Plan" }
    fn occurred_at(&self) -> DateTime<Utc> { self.metadata.occurred_at }
    fn aggregate_version(&self) -> u64 { self.metadata.aggregate_version }
    fn aggregate_id(&self) -> String { self.metadata.aggregate_id.clone() }
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Event emitted when a plan is updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanUpdatedEvent {
    pub metadata: EventMetadata,
    pub plan_id: String,
    pub updated_at: DateTime<Utc>,
}

impl PlanUpdatedEvent {
    pub fn new(aggregate_id: String, aggregate_version: u64, plan_id: String, updated_at: DateTime<Utc>) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id, aggregate_version),
            plan_id,
            updated_at,
        }
    }
}

impl DomainEvent for PlanUpdatedEvent {
    fn event_id(&self) -> Uuid { self.metadata.event_id }
    fn event_type(&self) -> &'static str { "PlanUpdated" }
    fn aggregate_type(&self) -> &'static str { "Plan" }
    fn occurred_at(&self) -> DateTime<Utc> { self.metadata.occurred_at }
    fn aggregate_version(&self) -> u64 { self.metadata.aggregate_version }
    fn aggregate_id(&self) -> String { self.metadata.aggregate_id.clone() }
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Event emitted when a plan is deleted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanDeletedEvent {
    pub metadata: EventMetadata,
    pub plan_id: String,
    pub deleted_at: DateTime<Utc>,
}

impl PlanDeletedEvent {
    pub fn new(aggregate_id: String, aggregate_version: u64, plan_id: String, deleted_at: DateTime<Utc>) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id, aggregate_version),
            plan_id,
            deleted_at,
        }
    }
}

impl DomainEvent for PlanDeletedEvent {
    fn event_id(&self) -> Uuid { self.metadata.event_id }
    fn event_type(&self) -> &'static str { "PlanDeleted" }
    fn aggregate_type(&self) -> &'static str { "Plan" }
    fn occurred_at(&self) -> DateTime<Utc> { self.metadata.occurred_at }
    fn aggregate_version(&self) -> u64 { self.metadata.aggregate_version }
    fn aggregate_id(&self) -> String { self.metadata.aggregate_id.clone() }
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Event emitted when a wallet is assigned to a plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAssignedToPlanEvent {
    pub metadata: EventMetadata,
    pub plan_id: String,
    pub wallet_address: String,
    pub assigned_at: DateTime<Utc>,
}

impl WalletAssignedToPlanEvent {
    pub fn new(
        aggregate_id: String,
        aggregate_version: u64,
        plan_id: String,
        wallet_address: String,
        assigned_at: DateTime<Utc>,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id, aggregate_version),
            plan_id,
            wallet_address,
            assigned_at,
        }
    }
}

impl DomainEvent for WalletAssignedToPlanEvent {
    fn event_id(&self) -> Uuid { self.metadata.event_id }
    fn event_type(&self) -> &'static str { "WalletAssignedToPlan" }
    fn aggregate_type(&self) -> &'static str { "Plan" }
    fn occurred_at(&self) -> DateTime<Utc> { self.metadata.occurred_at }
    fn aggregate_version(&self) -> u64 { self.metadata.aggregate_version }
    fn aggregate_id(&self) -> String { self.metadata.aggregate_id.clone() }
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Event emitted when a wallet is removed from a plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletRemovedFromPlanEvent {
    pub metadata: EventMetadata,
    pub plan_id: String,
    pub wallet_address: String,
    pub removed_at: DateTime<Utc>,
}

impl WalletRemovedFromPlanEvent {
    pub fn new(
        aggregate_id: String,
        aggregate_version: u64,
        plan_id: String,
        wallet_address: String,
        removed_at: DateTime<Utc>,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id, aggregate_version),
            plan_id,
            wallet_address,
            removed_at,
        }
    }
}

impl DomainEvent for WalletRemovedFromPlanEvent {
    fn event_id(&self) -> Uuid { self.metadata.event_id }
    fn event_type(&self) -> &'static str { "WalletRemovedFromPlan" }
    fn aggregate_type(&self) -> &'static str { "Plan" }
    fn occurred_at(&self) -> DateTime<Utc> { self.metadata.occurred_at }
    fn aggregate_version(&self) -> u64 { self.metadata.aggregate_version }
    fn aggregate_id(&self) -> String { self.metadata.aggregate_id.clone() }
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

// Type aliases for backward compatibility during migration
pub type PermissionPlanCreatedEvent = PlanCreatedEvent;
pub type PermissionPlanUpdatedEvent = PlanUpdatedEvent;
pub type PermissionPlanDeletedEvent = PlanDeletedEvent;
