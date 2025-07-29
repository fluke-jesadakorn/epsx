// Domain entities with business rules and behavior

pub mod user;
pub mod auth;
pub mod payment;
pub mod stock;
pub mod iam;
pub mod audit;
pub mod permission_profile;

pub use user::*;
pub use auth::*;
pub use payment::*;
pub use stock::*;
pub use iam::*;
pub use audit::*;
pub use permission_profile::*;

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::fmt::Debug;

/// Core aggregate root trait for domain entities
/// Provides common functionality for all aggregates
pub trait AggregateRoot: Send + Sync + Debug {
    type Id: Clone + Debug + Serialize + for<'de> Deserialize<'de>;
    
    /// Get the unique identifier for this aggregate
    fn id(&self) -> &Self::Id;
    
    /// Get the current version for optimistic concurrency control
    fn version(&self) -> u64;
    
    /// Get uncommitted domain events
    fn uncommitted_events(&self) -> &[Box<dyn crate::dom::events::DomainEvent>];
    
    /// Mark all events as committed (called after persistence)
    fn mark_events_as_committed(&mut self);
    
    /// Get when this aggregate was created
    fn created_at(&self) -> DateTime<Utc>;
    
    /// Get when this aggregate was last updated
    fn updated_at(&self) -> DateTime<Utc>;
}

/// Unit of Work pattern for transactional boundaries
#[async_trait::async_trait]
pub trait UnitOfWork: Send + Sync {
    type Error: Send + Sync + Debug;
    
    /// Commit all changes in this unit of work
    async fn commit(&self) -> Result<(), Self::Error>;
    
    /// Rollback all changes in this unit of work
    async fn rollback(&self) -> Result<(), Self::Error>;
    
    /// Check if the unit of work is still active
    fn is_active(&self) -> bool;
}