//! Aggregate Root trait and base implementation.
//!
//! Source: apps/backend/src/domain/shared_kernel/aggregate_root.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use uuid::Uuid;

use super::domain_event::DomainEvent;

/// Core aggregate root trait following DDD principles.
/// Aggregates are consistency boundaries and the only way to modify entities.
pub trait AggregateRoot: Send + Sync + Debug + Clone {
    type Id: Clone + Debug + Serialize + for<'de> Deserialize<'de> + PartialEq;

    fn id(&self) -> &Self::Id;
    fn version(&self) -> u64;
    fn increment_version(&mut self);
    fn uncommitted_events(&self) -> &[Box<dyn DomainEvent>];
    fn mark_events_as_committed(&mut self);
    fn created_at(&self) -> DateTime<Utc>;
    fn updated_at(&self) -> DateTime<Utc>;
    fn touch(&mut self);
}

/// Base implementation for aggregates.
#[derive(Debug, Serialize, Deserialize)]
pub struct AggregateBase {
    pub version: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip)]
    pub events: Vec<Box<dyn DomainEvent>>,
}

impl AggregateBase {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            version: 0,
            created_at: now,
            updated_at: now,
            events: Vec::new(),
        }
    }

    pub fn add_event(&mut self, event: Box<dyn DomainEvent>) {
        self.events.push(event);
        self.touch();
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    pub fn increment_version(&mut self) {
        self.version += 1;
        self.touch();
    }

    pub fn clear_events(&mut self) {
        self.events.clear();
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn uncommitted_events(&self) -> &[Box<dyn DomainEvent>] {
        &self.events
    }

    pub fn mark_events_as_committed(&mut self) {
        self.events.clear();
    }

    pub fn take_events(&mut self) -> Vec<Box<dyn DomainEvent>> {
        std::mem::take(&mut self.events)
    }

    pub fn from_persistence(
        version: u64,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            version,
            created_at,
            updated_at,
            events: Vec::new(),
        }
    }
}

impl Clone for AggregateBase {
    fn clone(&self) -> Self {
        Self {
            version: self.version,
            created_at: self.created_at,
            updated_at: self.updated_at,
            events: Vec::new(),
        }
    }
}

impl Default for AggregateBase {
    fn default() -> Self {
        Self::new()
    }
}

/// Identity trait for aggregate IDs.
pub trait Identity:
    Clone + Debug + Serialize + for<'de> Deserialize<'de> + PartialEq + Send + Sync
{
    fn new() -> Self;
    fn from_uuid(uuid: Uuid) -> Self;
    fn to_uuid(&self) -> Uuid;
    fn to_string(&self) -> String;
}

/// Generate a new unique ID.
pub fn new_id() -> Uuid {
    Uuid::new_v4()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregate_base_default_is_zero() {
        let base = AggregateBase::new();
        assert_eq!(base.version, 0);
        assert!(base.events.is_empty());
    }

    #[test]
    fn increment_version() {
        let mut base = AggregateBase::new();
        base.increment_version();
        assert_eq!(base.version, 1);
    }
}
