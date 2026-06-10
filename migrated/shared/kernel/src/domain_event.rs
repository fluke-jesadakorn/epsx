//! Domain event trait and metadata.
//!
//! Source: apps/backend/src/domain/shared_kernel/domain_event.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use uuid::Uuid;

/// Base trait for all domain events.
pub trait DomainEvent: Send + Sync + Debug {
    fn event_id(&self) -> Uuid;
    fn event_type(&self) -> &'static str;
    fn aggregate_type(&self) -> &'static str;
    fn occurred_at(&self) -> DateTime<Utc>;
    fn aggregate_version(&self) -> u64;
    fn aggregate_id(&self) -> String;
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>>;
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Base event data that all events should include.
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

/// Domain event bus port — concrete impls live in shared/events.
pub trait DomainEventBus: Send + Sync {
    fn publish(&self, event: &dyn DomainEvent);

    fn publish_batch(&self, events: &[Box<dyn DomainEvent>]) {
        for event in events {
            self.publish(&**event);
        }
    }
}

/// In-memory event bus for testing and simple scenarios.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_metadata_new_sets_fields() {
        let m = EventMetadata::new("agg-1".to_string(), 3);
        assert_eq!(m.aggregate_id, "agg-1");
        assert_eq!(m.aggregate_version, 3);
    }
}
