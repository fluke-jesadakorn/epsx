use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::fmt::Debug;
use uuid::Uuid;

/// Base trait for all domain events
/// Events represent things that have happened in the domain
pub trait DomainEvent: Send + Sync + Debug {
    /// Unique identifier for this event instance
    fn event_id(&self) -> Uuid;

    /// The type of event (used for routing and serialization)
    fn event_type(&self) -> &'static str;

    /// The type of aggregate that raised this event
    /// Used for event store partitioning and routing
    /// Examples: "WalletUser", "Subscription", "PermissionGroup"
    fn aggregate_type(&self) -> &'static str;

    /// When this event occurred
    fn occurred_at(&self) -> DateTime<Utc>;

    /// Version of the aggregate when this event was raised
    fn aggregate_version(&self) -> u64;

    /// The aggregate ID that raised this event
    fn aggregate_id(&self) -> String;

    /// Serialize event to JSON for storage/transport
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>>;

    /// Downcast to Any for type introspection
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Base event data that all events should include
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub aggregate_version: u64,
    pub aggregate_id: String,
}

impl EventMetadata {
    pub fn new(aggregate_id: String, aggregate_version: u64) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            aggregate_version,
            aggregate_id,
        }
    }
}

/// Simple event bus interface for domain events
pub trait DomainEventBus: Send + Sync {
    /// Publish a domain event
    fn publish(&self, event: &dyn DomainEvent);

    /// Publish multiple events as a batch
    fn publish_batch(&self, events: &[Box<dyn DomainEvent>]) {
        for event in events {
            self.publish(&**event);
        }
    }
}

/// In-memory event bus for testing and simple scenarios
pub struct InMemoryEventBus {
    events: std::sync::RwLock<Vec<String>>,
}

impl InMemoryEventBus {
    pub fn new() -> Self {
        Self {
            events: std::sync::RwLock::new(Vec::new()),
        }
    }
    
    pub fn published_events(&self) -> Vec<String> {
        let events = self.events.read().unwrap();
        events.clone()
    }
    
    pub fn clear_events(&self) {
        let mut events = self.events.write().unwrap();
        events.clear();
    }
}

impl DomainEventBus for InMemoryEventBus {
    fn publish(&self, event: &dyn DomainEvent) {
        let mut events = self.events.write().unwrap();
        events.push(event.event_type().to_string());
    }
}

impl Default for InMemoryEventBus {
    fn default() -> Self {
        Self::new()
    }
}