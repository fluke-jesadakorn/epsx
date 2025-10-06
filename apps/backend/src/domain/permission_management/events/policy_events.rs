use crate::prelude::*;
use crate::domain::shared_kernel::{DomainEvent, EventMetadata};
use uuid::Uuid;

/// Event emitted when a policy is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCreatedEvent {
    pub metadata: EventMetadata,
    pub policy_id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

impl PolicyCreatedEvent {
    pub fn new(
        aggregate_id: String,
        aggregate_version: u64,
        policy_id: String,
        name: String,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id, aggregate_version),
            policy_id,
            name,
            created_at,
        }
    }
}

impl DomainEvent for PolicyCreatedEvent {
    fn event_id(&self) -> Uuid { self.metadata.event_id }
    fn event_type(&self) -> &'static str { "PolicyCreated" }
    fn aggregate_type(&self) -> &'static str { "Policy" }
    fn occurred_at(&self) -> DateTime<Utc> { self.metadata.occurred_at }
    fn aggregate_version(&self) -> u64 { self.metadata.aggregate_version }
    fn aggregate_id(&self) -> String { self.metadata.aggregate_id.clone() }
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Event emitted when a policy is updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyUpdatedEvent {
    pub metadata: EventMetadata,
    pub policy_id: String,
    pub updated_at: DateTime<Utc>,
}

impl PolicyUpdatedEvent {
    pub fn new(aggregate_id: String, aggregate_version: u64, policy_id: String, updated_at: DateTime<Utc>) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id, aggregate_version),
            policy_id,
            updated_at,
        }
    }
}

impl DomainEvent for PolicyUpdatedEvent {
    fn event_id(&self) -> Uuid { self.metadata.event_id }
    fn event_type(&self) -> &'static str { "PolicyUpdated" }
    fn aggregate_type(&self) -> &'static str { "Policy" }
    fn occurred_at(&self) -> DateTime<Utc> { self.metadata.occurred_at }
    fn aggregate_version(&self) -> u64 { self.metadata.aggregate_version }
    fn aggregate_id(&self) -> String { self.metadata.aggregate_id.clone() }
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
}
